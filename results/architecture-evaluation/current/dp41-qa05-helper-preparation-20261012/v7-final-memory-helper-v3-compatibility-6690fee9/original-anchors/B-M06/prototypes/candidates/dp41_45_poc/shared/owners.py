"""45A owner reads and local state; no evaluator/oracle access."""
from __future__ import annotations

import copy
import re
import uuid
from urllib.parse import quote, unquote

from .evidence import canonical, digest


_REFERENCE_KEYS = ("question_id", "artifact_id", "result_id", "source_id", "publication_id", "task_id",
                   "handle", "source_ref", "question_ref", "ref", "id")


def _wire_ref(ref):
    if isinstance(ref, dict):
        value = next((ref[key] for key in _REFERENCE_KEYS if ref.get(key) is not None), None)
        return _wire_ref(value)
    return ref


def identity(item):
    return str(item.get("question_id") or item.get("artifact_id") or item.get("result_id")
               or item.get("source_id") or item.get("publication_id") or item.get("task_id")
               or item.get("id") or item.get("ref", ""))


def revision(item):
    return item.get("revision", item.get("request_revision", 1))


def reference(item):
    if item.get("evidence_ref"):
        return item["evidence_ref"]
    item_id = identity(item)
    is_task = item.get("task_id") == item_id and not any(
        key in item for key in ("question_id", "artifact_id", "result_id", "source_id", "publication_id"))
    return item_id if is_task or "question_id" in item else f"{item_id}@v{revision(item)}"


def _ref_parts(ref):
    """Normalize wire aliases without selecting a semantic referent."""
    if isinstance(ref, dict):
        key = next((key for key in _REFERENCE_KEYS
                    if ref.get(key) is not None), None)
        if key is None:
            return None, None
        name, requested = _ref_parts(ref[key])
        return name, ref.get("revision", requested)
    if not isinstance(ref, str):
        return None, None
    scoped = re.fullmatch(r"via-interaction:(PAST_DELIVERED|RECENT_INTERACTION):(.+)@v(\d+):([0-9a-f]{64})", ref)
    if scoped:
        return unquote(scoped[2]), int(scoped[3])
    match = re.fullmatch(r"(.+)@v(\d+)", ref)
    return (match[1], int(match[2])) if match else (ref, None)


def _metadata_record(item):
    """Owner index data must not carry original response or result bodies."""
    if isinstance(item, dict):
        return {key: _metadata_record(value) for key, value in item.items()
                if key not in {"text", "content", "delivered_voice_text", "voice_text", "body"}}
    if isinstance(item, list):
        return [_metadata_record(value) for value in item]
    return item


def permission_allowed(item, role="read"):
    permission = item.get("permission", {})
    value = permission.get(role, True) if isinstance(permission, dict) else permission
    return value not in (False, "DENIED", "REVOKED", "NOT_ALLOWED")


class OwnerRepository:
    def __init__(self, initial, emit):
        self.emit = emit
        self.sources = copy.deepcopy(initial.get("sources", []))
        self.current_user_id = initial.get("current_user_id", "LOCAL_CURRENT_USER")
        if not isinstance(self.current_user_id, str) or not self.current_user_id:
            raise ValueError("Current local user ID must be a nonempty string")
        self.memory_aliases = {s.get("key", s.get("metadata", {}).get("key", identity(s))): identity(s)
                               for s in self.sources if s.get("kind") == "USER_MEMORY"
                               and s.get("user_id", s.get("metadata", {}).get("user_id")) == self.current_user_id}
        self.source_capacity = initial.get("max_sources", 30)
        if type(self.source_capacity) is not int or self.source_capacity < 1:
            raise ValueError("Source capacity must be a positive integer")
        self.tasks = {identity(item): copy.deepcopy(item) for item in initial.get("tasks", [])}
        self.interactions = copy.deepcopy(initial.get("interactions", []))
        self.questions = {}
        for task in self.tasks.values():
            for question in task.get("questions", []):
                self.questions[identity(question)] = {"task_id": identity(task), **question}
        self.questions.update({identity(q): copy.deepcopy(q) for q in initial.get("questions", [])})
        self.selection = copy.deepcopy(initial.get("selection", initial.get("observations", [])))
        self.cache = {}
        self.policy_epoch = 1
        self.provided = set(self.tasks) | set(self.questions) | {reference(s) for s in self.sources}
        self.body_read_refs = set()
        self.source_slot_assignments = {}
        # Only actual owner reads issue these scoped handles. They are not an
        # alias for an original source that happens to share a publication ID.
        self.interaction_views = {}

    @staticmethod
    def _interaction_record_digest(record):
        # Permissions are checked live; the actual delivered body/range is pinned.
        return digest({key: value for key, value in record.items() if key != "permission"})

    def _interaction_view(self, record, scope, read_id):
        record_hash = self._interaction_record_digest(record)
        ref = f"via-interaction:{scope}:{quote(identity(record), safe='')}@v{revision(record)}:{record_hash}"
        entry = self.interaction_views.setdefault(ref, {
            "record": copy.deepcopy(record), "scope": scope, "record_hash": record_hash, "read_ids": []})
        if read_id not in entry["read_ids"]:
            entry["read_ids"].append(read_id)
        return ref, self._materialize_interaction_view(ref)

    def _materialize_interaction_view(self, ref):
        entry = self.interaction_views.get(ref)
        if entry is None:
            return None
        recorded = entry["record"]
        retained = [record for record in self.interactions
                    if identity(record) == identity(recorded) and revision(record) == revision(recorded)
                    and self._interaction_record_digest(record) == entry["record_hash"]]
        if not retained:
            # A deleted or changed same-revision record cannot expand an earlier read.
            return None
        latest = [record for record in self.interactions if identity(record) == identity(recorded)]
        newest_revision = max(revision(record) for record in latest)
        permission_records = [*retained, *[record for record in latest if revision(record) == newest_revision]]
        # A publication original and declared backing sources retain their
        # current access floor without replacing the actual delivered content.
        backing = [source for source in self.sources if identity(source) == identity(recorded)]
        if any(source.get("kind") == "PUBLICATION_ORIGINAL" for source in backing):
            newest = max(revision(source) for source in backing)
            permission_records.extend(source for source in backing if revision(source) == newest)
        for source_ref in recorded.get("source_refs", []):
            source = self.lookup(source_ref)
            current = self.current(source_ref)
            if source is None or current is None:
                return None
            permission_records.extend((source, current))
        result = copy.deepcopy(recorded)
        voice = recorded.get("delivered_voice_text", "")
        text = recorded.get("text", recorded.get("content", ""))
        actual_scope = "INTERACTION_CHANNELS"
        if entry["scope"] == "PAST_DELIVERED":
            # This selector supplies what reached the speech sink, never the
            # unplayed tail of the prepared/Text publication.
            actual_scope = "ACTUAL_TEXT" if recorded.get("voice_on") is False else "ACTUAL_VOICE"
            result["text"] = (text if recorded.get("text_presented") else "") if actual_scope == "ACTUAL_TEXT" else voice
            result.pop("content", None)
        else:
            result["text"] = text
        result.update({"evidence_ref": ref, "kind": "INTERACTION_EVIDENCE",
                       "permission": {role: all(permission_allowed(record, role) for record in permission_records)
                                      for role in ("read", "model", "agent")},
                       "evidence_provenance": {"owner": "Response Manager", "type": "INTERACTION",
                           "scope": actual_scope,
                           "selector": entry["scope"], "publication_id": identity(recorded),
                           "revision": revision(recorded), "record_sha256": entry["record_hash"],
                           "read_ids": list(entry["read_ids"]), "source_refs": copy.deepcopy(recorded.get("source_refs", [])),
                           "channels": {"text": {"presented": bool(recorded.get("text_presented")),
                                                   "actual_text_sha256": digest(text) if recorded.get("text_presented") else None},
                                        "voice": {"actual_text_sha256": digest(voice),
                                                  "delivered_receipt": recorded.get("delivered_receipt"),
                                                  "interrupted": bool(recorded.get("interrupted"))}}}})
        return result

    def current(self, ref):
        """Current policy record in the same owner namespace as the supplied ref."""
        found = self.lookup(ref)
        if found is None:
            return None
        if found.get("evidence_ref"):
            # Materialization already includes the current owner/backing-source floor.
            return found
        return self.lookup(identity(found))

    def metadata(self):
        source_keys = ("source_id", "id", "revision", "kind", "title", "name", "permission", "metadata")
        active = [t for t in self.tasks.values() if t.get("status") != "COMPLETED"]
        completed = [t for t in self.tasks.values() if t.get("status") == "COMPLETED"]
        vectors = {}
        for source in self.sources:
            vectors[identity(source)] = max(vectors.get(identity(source), 0), revision(source))
        self.provided.update(reference(s) for s in self.sources)
        self.provided.update(identity(t) for t in [*active, *completed[-10:]])
        self.provided.update(self.questions)
        visible_sources = [s for s in self.sources if s.get("kind") != "USER_MEMORY" or
                           s.get("user_id", s.get("metadata", {}).get("user_id")) == self.current_user_id]
        result = {"sources": [_metadata_record({k: ({mk: mv for mk, mv in v.items() if mk != "value"}
                               if k == "metadata" and item.get("kind") == "USER_MEMORY" else v)
                               for k, v in item.items() if k in source_keys
                               and (k != "metadata" or permission_allowed(item, "read")
                                    and permission_allowed(item, "model"))}) for item in visible_sources],
                  "tasks": [_metadata_record(task) for task in [*active, *completed[-10:]]],
                  "questions": [_metadata_record(q) for q in self.questions.values()],
                  "interactions": [_metadata_record(item) for item in self.interactions[-10:]],
                  "selection": _metadata_record(self.selection), "policy_epoch": self.policy_epoch,
                  "current_user_id": self.current_user_id,
                  "source_vectors": vectors,
                  "metadata_scope": {"tasks_total": len(self.tasks), "active_tasks_included": len(active),
                                     "completed_tasks_total": len(completed), "completed_tasks_recent_limit": 10,
                                     "interactions_total": len(self.interactions), "interactions_recent_limit": 10,
                                     "sources_logical_total": len({identity(s) for s in self.sources}),
                                     "source_version_records": len(self.sources),
                                     "source_capacity": self.source_capacity,
                                     "unpopulated_source_slots": sum(self._empty_source_slot(s) for s in self.sources),
                                     "bodies": "OWNER_READ_REQUIRED"}}
        return copy.deepcopy(result)

    def lookup(self, ref, *, _memory_path=None):
        wire_ref = _wire_ref(ref)
        if isinstance(wire_ref, str) and wire_ref.startswith("via-interaction:"):
            result = self._materialize_interaction_view(wire_ref)
            if result is not None and _ref_parts(ref)[1] != revision(result):
                return None
            return result
        item_id, requested = _ref_parts(ref)
        if item_id is None:
            return None
        item_id = self.memory_aliases.get(item_id, item_id)
        # Questions and Tasks retain their own identity even when their envelopes include both IDs.
        owned = self.questions.get(item_id) or self.tasks.get(item_id)
        if owned is not None:
            return owned if requested is None or revision(owned) == requested else None
        artifacts = [{**task["result"], "permission": {
            role: permission_allowed(task["result"], role) and permission_allowed(task, role)
            for role in ("read", "model", "agent")}}
            for task in self.tasks.values() if isinstance(task.get("result"), dict)]
        choices = [s for s in [*self.sources, *self.interactions, *artifacts]
                   if identity(s) == item_id and (requested is None or revision(s) == requested)
                   and (s.get("kind") != "USER_MEMORY" or
                        s.get("user_id", s.get("metadata", {}).get("user_id")) == self.current_user_id)]
        if not choices:
            return None
        result = max(choices, key=lambda s: revision(s))
        if result.get("kind") != "USER_MEMORY":
            return result
        if result.get("user_id", result.get("metadata", {}).get("user_id")) != self.current_user_id:
            return None
        path = set(_memory_path or ())
        if item_id in path:
            return None
        path.add(item_id)
        backing = []
        for source_ref in result.get("source_refs", []):
            source = self.lookup(source_ref, _memory_path=path)
            latest = self.lookup(identity(source), _memory_path=path) if source else None
            if source is None or latest is None:
                return {**result, "permission": {"read": False, "model": False, "agent": False}}
            backing.extend((source, latest))
        if backing:
            return {**result, "permission": {role: permission_allowed(result, role)
                    and all(permission_allowed(parent, role) for parent in backing)
                    for role in ("read", "model", "agent")}}
        return result

    async def read(self, spec, *, consumer_role="model"):
        if consumer_role not in ("model", "agent", "read"):
            raise ValueError("Unsupported owner read consumer role")
        selector = spec.get("selector", {})
        op = selector.get("op", "")
        tool = spec.get("tool", "context_retrieve")
        limit = min(10, max(1, int(spec.get("limit", 10))))
        if tool == "task_retrieve":
            values = list(self.tasks.values())
        elif tool == "interaction_retrieve":
            values = [*self.interactions, *self.questions.values()]
        else:
            values = list(self.sources)
        if op == "KNOWN_REF":
            ref = next((selector[k] for k in ("handle", "source_ref", "question_ref", "ref", "id")
                        if selector.get(k) is not None), None)
            found = self.lookup(ref)
            values = [found] if found is not None else []
        elif op == "EXPLICIT_SOURCE":
            name = selector.get("name", "")
            kind = selector.get("source_kind")
            values = [v for v in values if name in (identity(v), v.get("name"), v.get("title"),
                                                    v.get("metadata", {}).get("name"))
                      and (not kind or v.get("kind") in (kind, None))]
        elif op == "SOURCE_AT_REV":
            ref = next((selector[k] for k in ("source_id", "source_ref", "ref", "name")
                        if selector.get(k) is not None), None)
            source_id, encoded_revision = _ref_parts(ref)
            requested = selector.get("revision", encoded_revision)
            values = [v for v in self.sources if identity(v) == source_id and revision(v) == requested]
        elif op == "INPUT_SELECTION":
            selected = self.selection
            if isinstance(selected, dict):
                selected = selected.get("observations", selected.get("items", []))
            values = []
            for observation in selected:
                source = self.lookup(observation)
                values.append({**(source or {}), **observation,
                               "metadata": {**(source or {}).get("metadata", {}),
                                            "observation": copy.deepcopy(observation)}})
        elif op in ("CURRENT_QUESTION", "QUESTION"):
            values = list(self.questions.values())
            requested = next((selector[k] for k in ("question_id", "question_ref", "source_ref", "ref")
                              if selector.get(k) is not None), None)
            if requested is not None:
                item_id, requested_revision = _ref_parts(requested)
                values = [v for v in values if identity(v) == item_id
                          and (requested_revision is None or revision(v) == requested_revision)]
        elif op in ("PAST_DELIVERED", "RECENT_INTERACTION"):
            values = list(self.interactions)
            requested = next((selector[k] for k in ("publication_id", "source_ref", "ref")
                              if selector.get(k) is not None), None)
            if requested is not None:
                item_id, requested_revision = _ref_parts(requested)
                explicit_revision = selector.get("revision")
                wire_ref = _wire_ref(requested)
                if isinstance(wire_ref, str) and wire_ref.startswith("via-interaction:"):
                    pinned = self.lookup(requested)
                    if pinned is None or explicit_revision not in (None, revision(pinned)):
                        values = []
                publication_ref = selector.get("publication_id") is not None or any(identity(v) == item_id for v in self.interactions)
                if publication_ref and requested_revision is not None and explicit_revision not in (None, requested_revision):
                    values = []
                values = [v for v in values if (identity(v) == item_id or any(
                    _ref_parts(ref)[0] == item_id and
                    (requested_revision is None or _ref_parts(ref)[1] == requested_revision)
                    for ref in v.get("source_refs", [])))
                    and (identity(v) != item_id or requested_revision is None or revision(v) == requested_revision)
                    and (explicit_revision is None or revision(v) == explicit_revision)]
            elif selector.get("revision") is not None:
                values = [v for v in values if revision(v) == selector["revision"]]
        elif op == "TASK_RESULT":
            requested = next((selector[k] for k in ("task_id", "source_ref", "ref")
                              if selector.get(k) is not None), None)
            item_id, requested_revision = _ref_parts(requested)
            tasks = list(self.tasks.values()) if requested is None else (
                [self.tasks[item_id]] if item_id in self.tasks else [])
            if requested_revision is not None and item_id in self.tasks:
                tasks = [task for task in tasks if revision(task) == requested_revision]
            values = []
            for task in tasks:
                result = task.get("result")
                refs = task.get("result_refs", [])
                if isinstance(result, dict):
                    refs = [*refs, result]
                for ref in refs:
                    actual = self.lookup(ref)
                    if actual is not None:
                        # Result content belongs to the actual current artifact, with the Task's access floor.
                        values.append({**actual, "permission": {role: permission_allowed(actual, role)
                                       and permission_allowed(task, role) for role in ("read", "model", "agent")}})
            if requested is not None and item_id not in self.tasks:
                values = []
                for task in self.tasks.values():
                    result = task.get("result")
                    if isinstance(result, dict) and identity(result) == item_id and (
                            requested_revision is None or revision(result) == requested_revision):
                        actual = self.lookup(result)
                        if actual is not None:
                            values.append({**actual, "permission": {role: permission_allowed(actual, role)
                                           and permission_allowed(task, role) for role in ("read", "model", "agent")}})
        elif op == "CALENDAR_WINDOW":
            values = [v for v in self.sources if v.get("kind") == "CALENDAR"]
        elif op not in ("RECENT_TASK", "RECENT_INTERACTION", "ALL", ""):
            result = {"status": "NOT_SUPPORTED", "read_id": str(uuid.uuid4()), "items": [],
                      "coverage": "NONE", "gaps": [op], "scope": tool,
                      "consumer_role": consumer_role, "limit": limit, "candidate_count": 0}
            self.emit("OWNER_READ", {"spec": spec, "receipt": result})
            return result
        query = spec.get("query", selector.get("query", ""))
        # Retrieval returns scored candidates; it never adopts a semantic target.
        if query:
            terms = re.findall(r"[\w가-힣]+", query.lower())
            values.sort(key=lambda v: sum(term in str(v).lower() for term in terms), reverse=True)
        if op not in ("SOURCE_AT_REV", "PAST_DELIVERED", "INPUT_SELECTION"):
            latest = {}
            for value in values:
                key = identity(value)
                if key not in latest or revision(value) > revision(latest[key]):
                    latest[key] = value
            values = list(latest.values())
        read_id = str(uuid.uuid4())
        items, denied = [], []
        for value in values[:limit]:
            if value.get("kind") == "USER_MEMORY":
                value = self.lookup(reference(value))
                if value is None:
                    continue
            ref = reference(value)
            if op in ("PAST_DELIVERED", "RECENT_INTERACTION"):
                ref, value = self._interaction_view(value, op, read_id)
                if value is None:
                    denied.append(ref)
                    continue
            elif value.get("evidence_ref") in self.interaction_views:
                entry = self.interaction_views[value["evidence_ref"]]
                if read_id not in entry["read_ids"]:
                    entry["read_ids"].append(read_id)
                value = self._materialize_interaction_view(value["evidence_ref"])
            if not permission_allowed(value, "read") or not permission_allowed(value, consumer_role):
                denied.append(ref)
                continue
            cache_value = value
            if value.get("evidence_ref"):
                cache_value = {**value, "evidence_provenance": {
                    key: item for key, item in value["evidence_provenance"].items() if key != "read_ids"}}
            cache_key = (ref, digest(cache_value), self.policy_epoch)
            hit = cache_key in self.cache
            if not hit:
                self.cache[cache_key] = copy.deepcopy(value)
            v = self.cache[cache_key]
            if value.get("evidence_ref"):
                # A new QueryReceipt is evidence of another read, not a content
                # change. Keep current read provenance outside the content key.
                v = {**v, "evidence_provenance": {**v["evidence_provenance"],
                     "read_ids": list(value["evidence_provenance"]["read_ids"])}}
            content = v.get("text", v.get("content", v.get("instruction", "")))
            if op in ("PAST_DELIVERED", "RECENT_INTERACTION"):
                # Both text visibility and actual audible spans remain visible as distinct evidence.
                content = {"text": v.get("text", v.get("content", "")),
                           "delivered_voice_text": v.get("delivered_voice_text", ""),
                           "delivery_receipt": v.get("delivered_receipt")}
            read_metadata = copy.deepcopy(v)
            if "task_id" in v and not any(key in v for key in ("question_id", "artifact_id", "source_id")):
                # Task lookup supplies state and instructions; its result body has a separate read boundary.
                read_metadata = _metadata_record(v)
            items.append({"ref": ref, "kind": v.get("kind", "TASK" if "task_id" in v else "INTERACTION"),
                          "revision": revision(v), "content": content,
                          "metadata": read_metadata, "cache_hit": hit, "policy_epoch": self.policy_epoch})
            self.provided.add(ref)
            self.body_read_refs.add(ref)
        result = {"status": "OK" if items else "DENIED" if denied else "NO_MATCH", "read_id": read_id,
                  "items": items, "coverage": "BOUNDED", "gaps": denied,
                  "scope": tool, "consumer_role": consumer_role, "limit": limit, "candidate_count": len(values)}
        self.emit("OWNER_READ", {"spec": spec, "receipt": result})
        return result

    def apply_source(self, item):
        if not isinstance(item, dict) or not item.get("source_id") or "revision" not in item:
            raise ValueError("Source publication requires an actual source ID and revision")
        item_id = identity(item)
        known = any(identity(source) == item_id for source in self.sources)
        if known:
            # Retained historical revisions do not consume another logical source slot.
            self.sources.append(copy.deepcopy(item))
            consumed_slot = None
        else:
            index = next((i for i, source in enumerate(self.sources) if self._empty_source_slot(source)), None)
            if index is None:
                logical_count = len({identity(s) for s in self.sources})
                if logical_count >= self.source_capacity:
                    self.emit("SOURCE_CAPACITY_REJECTED", {"source_ref": reference(item),
                              "logical_source_count": logical_count, "source_capacity": self.source_capacity,
                              "reason": "LOGICAL_SOURCE_CAPACITY_EXHAUSTED"})
                    raise ValueError("Logical source capacity exhausted for a new source ID")
                # Small fixtures need not materialize empty placeholders. They
                # still share the canonical logical capacity limit.
                self.sources.append(copy.deepcopy(item))
                consumed_slot = None
            else:
                consumed_slot = identity(self.sources[index])
                self.provided.discard(reference(self.sources[index]))
                self.sources[index] = copy.deepcopy(item)
                self.source_slot_assignments[consumed_slot] = item_id
        self.cache.clear()
        if item.get("kind") == "USER_MEMORY" and item.get("user_id", item.get("metadata", {}).get("user_id")) == self.current_user_id:
            self.memory_aliases[item.get("key", item.get("metadata", {}).get("key", identity(item)))] = item_id
        self.emit("SOURCE_PUBLISHED", {"source": item, "consumed_slot_id": consumed_slot,
                  "logical_source_count": len({identity(s) for s in self.sources}),
                  "source_version_records": len(self.sources)})

    def register_memory(self, key, value, *, user_id, scope, source_refs, provenance, instruction):
        """Publish the actual local-user write through the ordinary source port."""
        if user_id != self.current_user_id:
            raise ValueError("User Memory belongs to a different local user")
        source_id = self.memory_aliases.get(key) or (
            "via-user-memory:" + quote(user_id, safe="") + ":" + quote(key, safe=""))
        collision = self.lookup(key)
        if collision is not None and collision.get("kind") != "USER_MEMORY":
            raise ValueError("Memory key collides with another owner reference")
        old = self.lookup(source_id)
        if old is not None and (old.get("kind") != "USER_MEMORY" or not permission_allowed(old, "read")):
            raise ValueError("Existing User Memory is unavailable or withdrawn")
        if any(identity(self.lookup(ref) or {}) == source_id for ref in source_refs):
            raise ValueError("User Memory cannot derive from its own stored version")
        item = {"source_id": source_id, "revision": revision(old) + 1 if old else 1,
                "kind": "USER_MEMORY", "title": key, "key": key, "value": copy.deepcopy(value),
                "text": value if isinstance(value, str) else canonical(value),
                "user_id": user_id, "scope": copy.deepcopy(scope), "persistent": True,
                "source_refs": copy.deepcopy(source_refs), "instruction": instruction,
                "registration_provenance": copy.deepcopy(provenance),
                "metadata": {"key": key, "user_id": user_id, "scope": copy.deepcopy(scope), "persistent": True},
                "permission": {"read": True, "model": True, "agent": True}}
        self.apply_source(item)
        self.provided.add(reference(item))
        return copy.deepcopy(item)

    def invalidate_memory(self, source_id):
        """Fence every stored revision after explicit deletion or source withdrawal."""
        for source in self.sources:
            if (identity(source) == source_id and source.get("kind") == "USER_MEMORY" and
                    source.get("user_id", source.get("metadata", {}).get("user_id")) == self.current_user_id):
                source["text"], source["value"], source["invalidated"] = "", None, "POLICY_REVOCATION"
                source.get("metadata", {}).pop("value", None)
                source["permission"] = {"read": False, "model": False, "agent": False}
        self.cache.clear()

    @staticmethod
    def _empty_source_slot(source):
        return bool(source.get("kind") == "SOURCE_SLOT" and source.get("metadata", {}).get("unpopulated")
                    and not source.get("text", source.get("content", "")))

    def materialize_consent_source(self, source):
        """Publish actual owner content after the caller validates the consent scope.

        This API does not resolve a Request or broaden permissions. The authority
        caller supplies only the permitted roles and actual source content.
        """
        if not isinstance(source, dict) or not source.get("source_id") or "revision" not in source:
            raise ValueError("Consent materialization requires an existing versioned source")
        indexes = [index for index, old in enumerate(self.sources)
                   if identity(old) == identity(source) and revision(old) == revision(source)]
        if not indexes:
            raise ValueError("Consent source placeholder unavailable")
        for index in indexes:
            old = self.sources[index]
            old_body = old.get("text", old.get("content", ""))
            new_body = source.get("text", source.get("content", ""))
            if old_body and old_body != new_body:
                raise ValueError("Consent materialization cannot change an existing source revision")
        before = [copy.deepcopy(self.sources[index]) for index in indexes]
        materialized = copy.deepcopy(source)
        if all(old == materialized for old in before):
            self.emit("CONSENT_SOURCE_MATERIALIZATION_REUSED", {"source_ref": reference(materialized),
                      "source_sha256": digest(materialized), "policy_epoch": self.policy_epoch})
            return copy.deepcopy(materialized)
        # Replace every matching placeholder so lookup cannot select a stale empty record on a revision tie.
        for index in indexes:
            self.sources[index] = copy.deepcopy(materialized)
        self.policy_epoch += 1
        self.cache.clear()
        self.provided.add(reference(materialized))
        self.emit("CONSENT_SOURCE_MATERIALIZED", {"source": materialized,
                  "source_ref": reference(materialized), "source_sha256": digest(materialized),
                  "previous_record_sha256": [digest(old) for old in before],
                  "policy_epoch": self.policy_epoch, "permissions": copy.deepcopy(materialized.get("permission", {})),
                  "body_read_evidence": False})
        return copy.deepcopy(materialized)

    def revoke(self, source_id):
        for value in [*self.sources, *self.interactions]:
            if (identity(value) == source_id and (value.get("kind") != "USER_MEMORY" or
                    value.get("user_id", value.get("metadata", {}).get("user_id")) == self.current_user_id)):
                value["permission"] = {"read": False, "model": False, "agent": False}
        self.policy_epoch += 1
        self.cache.clear()
        self.emit("SOURCE_REVOKED", {"source_id": source_id, "policy_epoch": self.policy_epoch})
