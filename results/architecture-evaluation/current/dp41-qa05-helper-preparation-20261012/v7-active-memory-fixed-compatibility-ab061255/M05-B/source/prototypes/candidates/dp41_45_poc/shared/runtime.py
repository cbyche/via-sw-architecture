"""Shared 44A dispatcher, 42A local adoption and 45A owner composition.

Only a DP41 candidate supplies request meaning. The dispatcher validates authority,
lineage and actual receipts; it neither sees an oracle nor repairs semantic output.
The external Agent is a payload-driven stub, with separately observable ingress,
receipt, result readiness and user delivery.
"""
from __future__ import annotations

import asyncio
import copy
import re
import uuid

from .evidence import canonical, digest
from .owners import OwnerRepository, identity, permission_allowed, reference, revision


class PolicyFenceError(RuntimeError):
    pass


class Ports:
    def __init__(self, runtime, request_id, input_id, transcript, revision_number=1, held_return=None):
        self.runtime = runtime
        self.request_id, self.input_id = request_id, input_id
        self.transcript, self.revision = transcript, revision_number
        self.held_return = held_return
        self.last_model_end = None
        self.frontier = list(runtime.request_frontiers.get(request_id, []))
        self.read_nodes = []

    @property
    def read_count(self):
        return self.runtime.read_ledger.get(self.request_id, 0)

    def emit(self, event_type, payload):
        return self.runtime.emit(event_type, {"request_id": self.request_id, "input_id": self.input_id, **payload})

    async def model(self, messages, role="CINTERPRET"):
        refs = self.runtime.request_read_refs.get(self.request_id, set())
        result, node = await self.runtime.call_model(messages, role, request_id=self.request_id, refs=refs,
            parents=[*self.frontier, *self.read_nodes], return_node=True)
        self.frontier, self.read_nodes = [node], []
        if self.held_return is not None:
            await self.held_return.wait()
            release_parent = getattr(self.held_return, "causal_node_id", None)
            parents = [node, *([release_parent] if release_parent else [])]
            node = self.runtime.timeline.point("MODEL_RETURN_RELEASED", parents=parents, request_id=self.request_id)
            self.frontier = [node]
        try:
            self.runtime.check_sources(refs, roles=("read", "model"), context="model_return")
        except PolicyFenceError:
            self.emit("MODEL_DISCARDED", {"role": role, "reason": "POLICY_REVOCATION_AFTER_HOLD", "output": result})
            raise
        self.emit("MODEL_RETURN_AVAILABLE", {"role": role, "payload": result, "input_revision": self.revision, "causal_node_id": node})
        return result

    async def read(self, spec):
        count = self.runtime.read_ledger.get(self.request_id, 0) + 1
        self.runtime.read_ledger[self.request_id] = count
        if count > 16:
            self.emit("READ_BUDGET_REJECTED", {"logical_request_reads": count})
            raise RuntimeError("Logical Request read budget exhausted")
        start = self.runtime.timeline.now()
        result = await self.runtime.owners.read(spec)
        node = self.runtime.timeline.add_node("OWNER_READ", "LOCAL_VIA", start, self.runtime.timeline.now(),
            parents=self.frontier, request_id=self.request_id, evidence=[result.get("read_id")], metadata={"spec": spec})
        self.read_nodes.append(node)
        refs = [item["ref"] for item in result.get("items", []) if item.get("ref")]
        self.runtime.request_read_refs.setdefault(self.request_id, set()).update(refs)
        return result

    async def clarify(self, question, scope=None):
        return await self.runtime.clarify(self, question, scope)


class LocalVIA:
    def __init__(self, initial, model_client, evidence, variant, *, stub_profile=None,
                 answer_source=None, voice_profile=None, scene_key=""):
        from .adapters import AgentWireAdapter, CalendarSourceAdapter, QuestionAuthority, PresentedQuestionBinding
        self.evidence, self.model_client, self.variant = evidence, model_client, variant
        self.scene_key = scene_key  # Telemetry only. Never placed in model context.
        self.owners = OwnerRepository(initial, self.emit)
        # Current-scene execution/event slots are separate from retained owner
        # history. A later scene may reuse the owner repository without reviving
        # every unfinished Task in the prior, now stopped scene scope.
        self.scene_task_ids = {identity(task) for task in initial.get("tasks", [])
                               if task.get("status") not in {"COMPLETED", "FAILED", "CANCELLED"}}
        self.pending_task_ids = set()
        self.agent_wire = AgentWireAdapter()
        self.calendar_source = CalendarSourceAdapter()
        self.question_authority = QuestionAuthority()
        self.presented_questions = PresentedQuestionBinding()
        self.voice_on = initial.get("voice", {}).get("on", True)
        self.voice_connection = initial.get("voice", {}).get("connection_id", "V1")
        self.policy = copy.deepcopy(initial.get("policy", {}))
        self.generation = initial.get("output", {}).get("generation", 0)
        self.playing = initial.get("output", {}).get("playing", False)
        self.initial_output = copy.deepcopy(initial.get("output", {}))
        self.user_speaking = False
        self.hold = asyncio.Event()
        self.hold.set()
        self.playback_lock = asyncio.Lock()
        self.profile = voice_profile or {"transcript_delay_seconds": 0., "output_onset_seconds": .25,
                                        "characters_per_second": 6., "stop_seconds": .005}
        self.stub_profile = stub_profile or {}
        self.answer_source = answer_source
        self.current_input_generation = 0
        self.requests, self.read_ledger, self.request_read_refs = {}, {}, {}
        self.effects, self.deliveries, self.publications = [], [], []
        self.seen_events, self.sent_commands, self.jobs = set(), {}, set()
        self.artifacts, self.goal_results, self.result_ready = {}, {}, {}
        self.memory_records = {}
        self.memory_registrations = {}
        for source in self.owners.sources:
            if (source.get("kind") == "USER_MEMORY" and
                    source.get("user_id", source.get("metadata", {}).get("user_id")) == self.owners.current_user_id):
                source = self.owners.lookup(identity(source)) or source
                metadata = source.get("metadata", {})
                self.memory_records[source.get("key", metadata.get("key", identity(source)))] = {
                    "value": copy.deepcopy(source.get("value", metadata.get("value", source.get("text")))),
                    "source_id": identity(source), "revision": revision(source),
                    "user_id": self.owners.current_user_id, "scope": source.get("scope", metadata.get("scope")),
                    "persistent": source.get("persistent", True),
                    "source_refs": copy.deepcopy(source.get("source_refs", [])), "policy_epoch": self.owners.policy_epoch}
        self.model_contexts = {}
        self.state_changed = asyncio.Event()
        self.output_counter, self.event_counter = 0, 0
        self.origin = evidence.seconds()
        from .timing import CausalTimeline
        self.timeline = CausalTimeline(evidence=evidence)
        self.request_frontiers, self.goal_frontiers, self.goal_result_nodes = {}, {}, {}
        self.speech_starts, self.speech_nodes, self.transcript_nodes = {}, {}, {}
        self.last_speech_node = self.previous_playback_node = self.current_voice_onset_node = None
        self.external_intervals = []
        self.input_evidence = {}
        self.question_focus, self.input_question_focus = None, {}
        self.question_followup_keys = set()
        self.last_delivery_s = None
        for question_id, stored_question in list(self.owners.questions.items()):
            question = self.question_authority.issue(stored_question, task_id=stored_question.get("task_id"),
                execution_id=stored_question.get("execution_id"))
            self.owners.questions[question_id] = question
            owner_task = self.owners.tasks.get(question.get("task_id"))
            if owner_task:
                question.setdefault("execution_id", owner_task.get("execution_id"))
                question.setdefault("request_revision", owner_task.get("request_revision", 1))
            if question.get("delivered_receipt") and question.get("text_presented"):
                receipt = {"publication_id": "OWNER_RECEIPT:" + identity(question),
                           "text_presented": True, "voice_on": bool(question.get("voice_presented")),
                           "voice_presented": bool(question.get("voice_presented")), "delivered_receipt": True}
                self.presented_questions.present(question, receipt=receipt)

    def now(self):
        return self.evidence.seconds() - self.origin

    def emit(self, kind, payload):
        row = dict(payload)
        if "kind" in row:
            row["payload_kind"] = row["kind"]
        # Event identity belongs to the runtime even when a delivery/owner payload
        # carries its own domain kind or telemetry-shaped fields.
        row.update({"scene_key": self.scene_key, "scene_seconds": self.now(),
                    "kind": kind, "owner": "LOCAL_VIA"})
        return self.evidence.write("events", row)

    def track(self, coroutine):
        task = asyncio.create_task(coroutine)
        self.jobs.add(task)
        def completed(done):
            self.jobs.discard(done)
            if not done.cancelled() and done.exception() is not None:
                error = done.exception()
                self.emit("ASYNC_JOB_FAILED", {"error": type(error).__name__, "detail": str(error)})
        task.add_done_callback(completed)
        return task

    def _provided(self, ref):
        if isinstance(ref, dict):
            source = self.owners.lookup(ref)
            ref = reference(source) if source else ref.get("ref")
        if ref in self.owners.provided:
            return True
        source = self.owners.lookup(ref)
        return bool(source and reference(source) in self.owners.provided)

    def check_sources(self, refs, *, roles=("read",), context="consume", require_provided=False):
        for ref in refs or []:
            source = self.owners.lookup(ref)
            if source is None or (require_provided and not self._provided(ref)):
                self.emit("REFERENCE_REJECTED", {"reference": ref, "context": context, "reason": "NOT_PROVIDED"})
                raise PolicyFenceError(f"Reference unavailable or not provided: {ref}")
            # Revocation applies to all historical revisions and derived uses.
            latest = self.owners.current(ref) or source
            if any(not permission_allowed(item, role) for item in (source, latest) for role in roles):
                self.emit("POLICY_CONSUMER_REJECTED", {"reference": ref, "roles": list(roles),
                           "context": context, "policy_epoch": self.owners.policy_epoch})
                raise PolicyFenceError(f"Source permission denied at {context}: {ref}")

    async def call_model(self, messages, role, *, request_id, refs=(), max_tokens=None,
                         parents=None, return_node=False):
        refs = tuple(refs)
        self.check_sources(refs, roles=("read", "model"), context="model_start")
        context_id = str(uuid.uuid4())
        self.model_contexts[context_id] = {"refs": refs, "messages": messages, "request_id": request_id}
        start, observed_nodes = self.timeline.now(), []
        parents = list(parents or [])
        node = None
        try:
            kwargs = {"request_id": request_id, "case_id": self.scene_key, "causal_timeline": self.timeline,
                      "causal_parents": parents, "causal_callback": observed_nodes.append}
            if max_tokens is not None:
                kwargs["max_tokens"] = max_tokens
            result = await self.model_client.call(messages, role, **kwargs)
            if observed_nodes:
                node = self.timeline.point("MODEL_OUTPUT_PARSED", parents=[observed_nodes[-1]], request_id=request_id,
                                          metadata={"role": role})
            else:
                node = self.timeline.add_node("MODEL_CALL", "LOCAL_VIA", start, self.timeline.now(),
                                          parents=parents, request_id=request_id, metadata={"role": role, "mock_provider": True})
            try:
                self.check_sources(refs, roles=("read", "model"), context="model_return")
            except PolicyFenceError:
                self.emit("MODEL_DISCARDED", {"request_id": request_id, "role": role,
                           "reason": "POLICY_REVOCATION", "policy_epoch": self.owners.policy_epoch,
                           "output": result})
                raise
            return (result, node) if return_node else result
        finally:
            if node is None and observed_nodes:
                self.timeline.point("MODEL_CALL_TERMINAL", parents=[observed_nodes[-1]], request_id=request_id,
                                    metadata={"role": role, "failed_or_cancelled": True})
            self.model_contexts.pop(context_id, None)

    async def input_started(self, input_id, *, advance_generation=True, causal_parents=None):
        self.user_speaking = True
        self.hold.clear()
        if advance_generation:
            self.current_input_generation += 1
        if self.question_focus:
            self.input_question_focus[input_id] = copy.deepcopy(self.question_focus)
        onset = self.now()
        absolute_onset = self.timeline.now()
        input_node = self.timeline.point("INPUT_STARTED", absolute_onset, parents=causal_parents,
                                        owner="USER", metadata={"input_id": input_id})
        self.speech_starts[input_id] = (absolute_onset, input_node)
        self.emit("INPUT_STARTED", {"input_id": input_id, "generation": self.current_input_generation, "causal_node_id": input_node})
        if self.playing:
            old_generation = self.generation
            self.generation += 1
            await asyncio.sleep(self.profile["stop_seconds"])
            self.playing = False
            stop_node = self.timeline.add_node("LOCAL_OUTPUT_STOP", "LOCAL_VIA", absolute_onset, self.timeline.now(),
                parents=[input_node, *([self.current_voice_onset_node] if self.current_voice_onset_node else [])],
                metadata={"input_id": input_id})
            record = {"kind": "LOCAL_STOP", "input_id": input_id, "generation": old_generation,
                      "start_s": onset, "end_s": self.now(), "text": self.initial_output.get("delivered_voice_text", ""),
                      "causal_node_id": stop_node, "stop_node_id": stop_node,
                      "evidence_level": "MEASURED_MOCK_E2E"}
            self.deliveries.append(record)
            self.evidence.write("deliveries", {"scene_key": self.scene_key, **record})

    async def speech_end(self, input_id):
        self.user_speaking = False
        start, parent = self.speech_starts.get(input_id, (self.timeline.now(), None))
        node = self.timeline.add_node("EXTERNAL_USER_SPEECH", "USER", start, self.timeline.now(),
            parents=[parent] if parent else [], metadata={"input_id": input_id})
        self.speech_nodes[input_id] = self.last_speech_node = node
        self.emit("SPEECH_ENDED", {"input_id": input_id, "causal_node_id": node})
        self.hold.set()

    def interpretation_input(self, ports):
        spans = [{"id": "whole-input", "start": 0, "end": len(ports.transcript), "text": ports.transcript,
                  "input_revision": ports.revision}]
        for index, mention in enumerate(self.input_evidence.get("mentions", [])):
            text = mention["text"]
            offset = ports.transcript.find(text)
            if offset >= 0:
                spans.append({"id": f"input-span-{index}", "start": offset, "end": offset + len(text),
                              "text": text, "input_revision": ports.revision, **mention})
        metadata = self.owners.metadata()
        metadata["clock"] = {"date": "2026-10-12", "time_zone": "Asia/Seoul"}
        focus = self.input_question_focus.get(ports.input_id, self.question_focus)
        if focus:
            current = self.owners.questions.get(focus["question_id"])
            if current and current.get("status") == "VALID" and revision(current) == focus["question_revision"]:
                metadata["question_focus"] = copy.deepcopy(focus)
        return {"transcript": ports.transcript, "input_id": ports.input_id,
                "request_id": ports.request_id, "revision": ports.revision, "input_revision": ports.revision,
                "conversation_id": "C현재", "spans": spans, "metadata": metadata,
                "current_adopted_meaning": self.requests.get(ports.request_id, {}).get("meaning"),
                "budgets": {"model_calls": 8, "reads": 16, "cloud_concurrency": 4}}

    def validate_adoption(self, meaning, ports):
        from ..dp41.contract import validate_document
        validate_document(meaning)
        self.check_sources(self.request_read_refs.get(ports.request_id, ()),
                           roles=("read", "model"), context="meaning_adoption")
        for goal in meaning.get("goals", []):
            target = goal.get("target")
            if target is not None and (target not in self.owners.tasks or not self._provided(target)):
                raise ValueError("Task binding not supplied by owner metadata or an actual read")
            args = goal.get("arguments", {})
            for ref in [*args.get("source_refs", []), *[args[k] for k in ("recipient_ref", "question_ref") if args.get(k)]]:
                if not self._provided(ref):
                    raise ValueError(f"Binding reference was not provided: {ref}")

    async def resolve(self, transcript, *, input_id=None, request_id=None, revision_number=1,
                      held_return=None, generation=None, adopt=True):
        input_id, request_id = input_id or str(uuid.uuid4()), request_id or str(uuid.uuid4())
        generation = self.current_input_generation if generation is None else generation
        ports = Ports(self, request_id, input_id, transcript, revision_number, held_return)
        parent = self.speech_nodes.get(input_id)
        start = self.timeline.nodes[parent]["end_s"] if parent else self.timeline.now()
        node = self.timeline.add_node("TRANSCRIPT_DELIVERY", "LOCAL_VIA", start, self.timeline.now(),
            parents=[parent] if parent else [], request_id=request_id, metadata={"input_id": input_id})
        self.transcript_nodes[input_id], self.request_frontiers[request_id], ports.frontier = node, [node], [node]
        self.emit("TRANSCRIPT_FINAL_RECEIVED", {"input_id": input_id, "request_id": request_id,
                                                "transcript": transcript, "generation": generation, "causal_node_id": node})
        candidate = (__import__("prototypes.candidates.dp41_45_poc.dp41.a", fromlist=["resolve"])
                     if self.variant == "A" else __import__("prototypes.candidates.dp41_45_poc.dp41.b", fromlist=["resolve"]))
        try:
            meaning = await candidate.resolve(self.interpretation_input(ports), ports)
            self.emit("MEANING_PROPOSED", {"request_id": request_id, "meaning": meaning, "generation": generation})
            if generation != self.current_input_generation:
                self.emit("MEANING_DISCARDED", {"request_id": request_id, "reason": "SUPERSEDED_INPUT", "meaning": meaning})
                return {"status": "DISCARDED", "meaning": meaning}
            self.validate_adoption(meaning, ports)
            if adopt:
                adoption_node = self.timeline.point("MEANING_ADOPTED", parents=[*ports.frontier, *ports.read_nodes], request_id=request_id)
                self.request_frontiers[request_id] = [adoption_node]
                self.requests[request_id] = {"meaning": meaning, "ports": ports, "generation": generation}
                self.emit("MEANING_ADOPTED", {"request_id": request_id, "meaning": meaning})
                await self.execute(meaning, ports)
            return {"status": "ADOPTED" if adopt else "PROPOSED", "meaning": meaning}
        except Exception as error:
            self.emit("REQUEST_FAILED", {"request_id": request_id, "error": type(error).__name__, "detail": str(error)})
            return {"status": "FAILED", "error": str(error)}

    async def clarify(self, ports, question, scope):
        requested = (scope or {}).get("question_refs", [])
        if len(requested) == 1:
            spec = requested[0]
            qid = spec.get("question_ref", spec.get("question_id"))
            record = self.owners.questions.get(qid)
            if not record or record.get("status") != "VALID" or revision(record) != spec.get("revision"):
                return {"no_answer": True, "question_id": qid, "reason": "QUESTION_NOT_CURRENT"}
            question = record["text"]
        else:
            qid = str(uuid.uuid4())
            record = {"question_id": qid, "text": question, "scope": scope, "request_id": ports.request_id,
                      "revision": 1, "status": "VALID", "presentation_status": "PRESENTATION_PENDING",
                      "origin": "VIA_CLARIFICATION"}
            self.owners.questions[qid] = record
        self.owners.provided.add(qid)
        delivery = await self.deliver(question, facts={"question": record,
            "_source_refs": list(self.request_read_refs.get(ports.request_id, ()))}, request_id=ports.request_id,
            kind="QUESTION", causal_parents=[*ports.frontier, *ports.read_nodes], question_answer_mode="CLARIFICATION")
        if not delivery.get("delivered_receipt"):
            return {"no_answer": True, "question_id": qid, "reason": delivery.get("discard_reason")}
        if self.answer_source is None:
            return {"no_answer": True, "question_id": qid}
        answer = await self._question_answer(delivery, clarification_scope=scope)
        if answer is None:
            return {"no_answer": True, "question_id": qid}
        self.emit("CLARIFICATION_ANSWER_RECEIVED", {"question_id": qid, **answer})
        speech_node = self.speech_nodes.get(answer.get("input_id"))
        answer_parent = delivery.get("voice_end_node_id", delivery.get("text_node_id"))
        answer_parents = [*ports.frontier, *ports.read_nodes,
            *([speech_node] if speech_node else []), *([answer_parent] if answer_parent else [])]
        start = self.timeline.nodes[speech_node]["end_s"] if speech_node else self.timeline.now()
        answer_node = self.timeline.add_node("CLARIFICATION_TRANSCRIPT_DELIVERY", "LOCAL_VIA", start,
            self.timeline.now(), parents=answer_parents, request_id=ports.request_id,
            metadata={"input_id": answer.get("input_id"), "question_id": qid})
        ports.frontier, ports.read_nodes = [answer_node], []
        self.transcript_nodes[answer.get("input_id")] = answer_node
        # Plan matching belongs to replay telemetry, never to the candidate context.
        return {"transcript": answer["transcript"], "input_id": answer["input_id"],
                "question_id": qid, "question_revision": revision(record)}

    def _current_question_delivery(self, delivery):
        question = (delivery.get("facts") or {}).get("question", {})
        current = self.owners.questions.get(identity(question))
        if (not current or current.get("status") != "VALID" or revision(current) != revision(question)
                or current.get("scope") != question.get("scope") or current.get("kind") != question.get("kind")
                or delivery.get("delivered_receipt") is not True or current.get("delivered_receipt") is not True):
            return None
        if delivery.get("text_presented_s") is None or not delivery.get("text"):
            return None
        if delivery.get("voice_on") and (delivery.get("voice_onset_s") is None
                or delivery.get("voice_text") != delivery.get("text")):
            return None
        if current.get("task_id") is not None:
            task = self.owners.tasks.get(current["task_id"])
            if (not task or task.get("execution_id") != current.get("execution_id")
                    or task.get("request_revision", 1) != current.get("request_revision", task.get("request_revision", 1))):
                return None
        return current

    async def _question_answer(self, delivery, *, clarification_scope=None, new_turn=False):
        question = self._current_question_delivery(delivery)
        callback = getattr(self.answer_source, "on_question_presented", None) if new_turn else self.answer_source
        if question is None or callback is None:
            return None
        key = (identity(question), revision(question))
        if key in self.question_followup_keys:
            self.emit("QUESTION_FOLLOWUP_SUPPRESSED", {"question_id": key[0], "question_revision": key[1],
                "publication_id": delivery["publication_id"], "reason": "ALREADY_REQUESTED"})
            return None
        self.question_followup_keys.add(key)
        try:
            if new_turn:
                answer = await callback(delivery, self)
            else:
                answer = await callback(delivery["text"], clarification_scope, delivery, self)
            if answer is None:
                self.question_followup_keys.discard(key)
            return answer
        except BaseException:
            # Reserve only while acquisition is in flight. A rejected display
            # or failed callback did not supply this question's actual user turn.
            self.question_followup_keys.discard(key)
            raise

    async def _followup_from_question(self, delivery):
        answer = await self._question_answer(delivery, new_turn=True)
        if answer is None:
            return
        request_id = answer.get("request_id") or "R:" + str(uuid.uuid4())
        self.emit("QUESTION_FOLLOWUP_RECEIVED", {**answer, "request_id": request_id,
            "generation": self.current_input_generation})
        # The actual user transcript starts a normal new interpretation. No
        # answer/permission/Task binding is supplied by the replay callback.
        await self.resolve(answer["transcript"], input_id=answer["input_id"], request_id=request_id)

    @staticmethod
    def _fact_refs(facts):
        refs = []
        for item in (facts or {}).get("sources", []):
            refs.append(reference(item))
        for key in ("summary", "artifact"):
            item = (facts or {}).get(key, {})
            if isinstance(item, dict):
                refs.extend(item.get("source_refs", []))
        refs.extend((facts or {}).get("_source_refs", []))
        def raw_dependencies(value):
            if isinstance(value, dict):
                if "source_id" in value and any(k in value for k in ("text", "content")):
                    refs.append(reference(value))
                if "artifact_id" in value and any(k in value for k in ("text", "content")):
                    refs.extend(value.get("source_refs", []))
                if "question_id" in value and any(k in value for k in ("text", "content")):
                    refs.extend(value.get("source_refs", []))
                for child in value.values():
                    raw_dependencies(child)
            elif isinstance(value, list):
                for child in value:
                    raw_dependencies(child)
        raw_dependencies(facts or {})
        return refs

    async def response(self, facts, ports=None, *, kind="RESPONSE", source_event=None):
        request_id = ports.request_id if ports else f"event:{source_event or uuid.uuid4()}"
        refs = self._fact_refs(facts)
        parents = facts.get("_causal_parents", self.request_frontiers.get(request_id, []))
        result, model_node = await self.call_model([
            {"role": "system", "content": "VIA의 한국어 응답 문장을 생성한다. 제공된 실제 사실만 간결히 말한다. "
             "일반 답변은 100자 이내로 핵심 사실을 전달한다. 실제 receipt가 있으면 어떤 대상 업무/operation의 "
             "어떤 상태인지 밝혀라. ACCEPTED는 접수이며 완료가 아니다. CANCEL_ACCEPTED는 취소 명령 접수이며 "
             "실제 실행 종료 확인이 아니다. completion_confirmed와 actual 외부 status가 완료를 확인할 때만 "
             "완료했다고 말한다. SENT가 실제 확인되면 실제 recipient와 어떤 결과가 발송됐는지 포함하라. "
             "원문/결과 body만 낭독하여 접수나 완료 상태를 빠뜨리지 마라. 질문의 선택지를 합치거나 임의 "
             "답변하지 않는다. 출처와 대상 업무를 구별한다. 거절 사실이면 그 거절을 명확히 전달한다. "
             '반환은 JSON {"text": "사용자에게 전달할 문장"}뿐이다.'},
            {"role": "user", "content": canonical(facts)}], "CRESPONSE", request_id=request_id,
            refs=refs, max_tokens=700, parents=parents, return_node=True)
        return await self.deliver(result.get("text", ""), facts=facts, request_id=request_id,
                                  kind=kind, source_event=source_event, causal_parents=[model_node])

    def _delivery_guard(self, facts, created_generation):
        if created_generation != self.generation:
            return "SUPERSEDED_OUTPUT"
        question = (facts or {}).get("question")
        if question:
            current = self.owners.questions.get(identity(question))
            if not current or current.get("status") != "VALID" or revision(current) != revision(question):
                return "QUESTION_WITHDRAWN_OR_STALE"
            if current.get("task_id") is not None:
                task = self.owners.tasks.get(current["task_id"])
                if (not task or task.get("execution_id") != current.get("execution_id")
                        or task.get("request_revision", 1) != current.get("request_revision", task.get("request_revision", 1))):
                    return "QUESTION_OWNER_OR_REVISION_MISMATCH"
        try:
            self.check_sources(self._fact_refs(facts), roles=("read", "model"), context="publication")
        except PolicyFenceError:
            return "POLICY_REVOCATION"
        return None

    async def deliver(self, text, *, facts=None, request_id="", kind="RESPONSE", source_event=None, causal_parents=None,
                      question_answer_mode="NEW_REQUEST"):
        self.output_counter += 1
        publication = f"P:{self.output_counter}:{uuid.uuid4()}"
        created_generation = self.generation
        record = {"kind": kind, "publication_id": publication, "text": text, "facts": facts,
                  "request_id": request_id, "source_event_id": source_event, "generation": created_generation,
                  "text_presented_s": None, "voice_on": self.voice_on, "voice_onset_s": None,
                  "voice_end_s": None, "voice_text": "", "voice_char_events": [], "delivered_receipt": False,
                  "evidence_level": "MEASURED_MOCK_E2E"}
        playback_start = None
        playback_absolute_start = None
        parents = list(causal_parents or self.request_frontiers.get(request_id, []))
        text_node = voice_node = None
        try:
            await self.hold.wait()
            guard = self._delivery_guard(facts, created_generation)
            if guard:
                record["discard_reason"] = guard
            else:
                text_at = self.timeline.now()
                text_parents = [*parents, *([self.last_speech_node] if self.last_speech_node else [])]
                text_node = self.timeline.point("TEXT_PRESENTED", text_at, parents=text_parents,
                                               request_id=request_id, metadata={"publication_id": publication})
                record["text_presented_s"] = text_at - self.origin
                record["text_node_id"] = record["causal_node_id"] = text_node
                record["text_presented"] = True
                self.emit("TEXT_PRESENTED", record)
                # Text publication is actual evidence even when voice is subsequently interrupted.
                question = (facts or {}).get("question")
                if question:
                    current = self.owners.questions[identity(question)]
                    current["text_presented"] = True
                if self.voice_on and text:
                    async with self.playback_lock:
                        await self.hold.wait()
                        guard = self._delivery_guard(facts, created_generation)
                        if not guard:
                            setup_start = self.timeline.now()
                            setup_parents = [text_node, *([self.previous_playback_node] if self.previous_playback_node else []),
                                             *([self.last_speech_node] if self.last_speech_node else [])]
                            await asyncio.sleep(self.profile["output_onset_seconds"])
                            setup_node = self.timeline.add_node("VOICE_SETUP", "LOCAL_VIA", setup_start,
                                self.timeline.now(), parents=setup_parents, request_id=request_id,
                                metadata={"publication_id": publication})
                            await self.hold.wait()
                            guard = self._delivery_guard(facts, created_generation)
                        if guard:
                            record["discard_reason"] = guard
                        else:
                            voice_at = self.timeline.now()
                            voice_node = self.timeline.point("VOICE_ONSET", voice_at,
                                parents=[setup_node, *([self.last_speech_node] if self.last_speech_node else [])],
                                request_id=request_id, metadata={"publication_id": publication})
                            self.current_voice_onset_node = voice_node
                            record["voice_onset_s"] = voice_at - self.origin
                            record["voice_onset_node_id"] = record["causal_node_id"] = voice_node
                            record["voice_char_events"].append({"char_index": 0,
                                "onset_s": voice_at - self.origin, "causal_node_id": voice_node})
                            record["voice_text"] = text[:1]
                            self.emit("VOICE_ONSET", {"publication_id": publication, "request_id": request_id, "causal_node_id": voice_node})
                            self.playing = True
                            rate = max(.001, self.profile["characters_per_second"])
                            duration = len(text) / rate
                            start = self.now()
                            playback_start = start
                            playback_absolute_start = self.timeline.now()
                            while self.now() - start < duration:
                                if self._delivery_guard(facts, created_generation):
                                    break
                                await asyncio.sleep(min(.02, max(0., duration - (self.now() - start))))
                                if self._delivery_guard(facts, created_generation):
                                    break
                                emitted_at = self.timeline.now()
                                available = min(len(text), int((emitted_at - playback_absolute_start) * rate) + 1)
                                # The mock backend emits at its observed tick. A
                                # scheduled character time is not a measured onset.
                                for index in range(len(record["voice_char_events"]), available):
                                    char_node = self.timeline.point("VOICE_CHARACTER", emitted_at, parents=[voice_node],
                                        request_id=request_id, metadata={"publication_id": publication, "char_index": index})
                                    record["voice_char_events"].append({"char_index": index,
                                        "onset_s": emitted_at - self.origin, "causal_node_id": char_node})
                                record["voice_text"] = text[:len(record["voice_char_events"])]
                            record["voice_end_s"] = self.now()
                            playback_node = self.timeline.add_node("VOICE_PLAYBACK", "LOCAL_VIA", playback_absolute_start,
                                self.timeline.now(), parents=[voice_node], request_id=request_id,
                                metadata={"publication_id": publication, "audible_characters": len(record["voice_text"])})
                            self.previous_playback_node = playback_node
                            record["voice_end_node_id"] = playback_node
                            self.playing = False
                            guard = self._delivery_guard(facts, created_generation)
                            if guard:
                                record["discard_reason"] = guard
                guard = self._delivery_guard(facts, created_generation)
                # For Voice ON question scope becomes answerable only after actual voice onset.
                record["delivered_receipt"] = not guard and bool(record["text_presented_s"] is not None) and (
                    not record["voice_on"] or bool(record["voice_onset_s"] is not None))
                if question and record["delivered_receipt"]:
                    current = self.owners.questions[identity(question)]
                    current.update({"presentation_status": "PRESENTED", "delivered_receipt": True,
                                    "voice_presented": bool(record["voice_onset_s"] is not None)})
                    self.presented_questions.present(current, receipt=record)
                    self.question_focus = {"question_id": identity(current), "question_revision": revision(current),
                        "task_id": current.get("task_id"), "execution_id": current.get("execution_id"),
                        "question_publication_id": publication}
                    self.emit("QUESTION_PRESENTED", {"question": copy.deepcopy(current), "delivery": record})
        except asyncio.CancelledError:
            record["discard_reason"] = "LOCAL_CANCELLED"
            record["delivered_receipt"] = False
            if playback_start is not None:
                record["voice_text"] = text[:len(record["voice_char_events"])]
                record["voice_end_s"] = self.now()
                if voice_node is not None:
                    node = self.timeline.add_node("VOICE_PLAYBACK_CANCELLED", "LOCAL_VIA", playback_absolute_start,
                        self.timeline.now(), parents=[voice_node], request_id=request_id,
                        metadata={"publication_id": publication, "audible_characters": len(record["voice_text"])})
                    self.previous_playback_node = node
                    record["voice_end_node_id"] = node
                self.playing = False
            self.emit("OUTPUT_CANCELLED", {"publication_id": publication, "request_id": request_id, "voice_text": record["voice_text"]})
            raise
        finally:
            record["end_s"] = max(record.get("text_presented_s") or self.now(), record.get("voice_onset_s") or 0.)
            self.deliveries.append(record)
            self.publications.append(record)
            self.last_delivery_s = self.now()
            if record.get("text_presented_s") is not None:
                self.owners.interactions.append({"publication_id": publication, "text": text,
                     "delivered_voice_text": record["voice_text"], "delivered_receipt": record["delivered_receipt"],
                     "text_presented": True, "voice_on": record["voice_on"],
                     "text_presented_s": record["text_presented_s"], "voice_onset_s": record["voice_onset_s"],
                     "interrupted": bool(record["voice_on"] and record["voice_text"] != text),
                     "source_refs": self._fact_refs(facts), "generation": created_generation})
                self.owners.provided.add(publication)
            self.evidence.write("deliveries", {"scene_key": self.scene_key, **record})
        # Playback lock and final delivery evidence are complete before a new
        # user turn can acquire the common input/output paths.
        if question_answer_mode == "NEW_REQUEST" and getattr(self.answer_source, "on_question_presented", None):
            if self._current_question_delivery(record) is not None:
                self.track(self._followup_from_question(record))
        return record

    def source_payload(self, references, *, roles=("read", "agent"), context="agent_payload"):
        self.check_sources(references, roles=roles, context=context, require_provided=True)
        result = [copy.deepcopy(self.owners.lookup(ref)) for ref in references or []]
        self.emit("SOURCE_COMPOSITION_READ", {"references": references or [], "roles": list(roles),
                  "context": context, "policy_epoch": self.owners.policy_epoch, "payload_sha256": digest(result)})
        return result

    def _ready(self, request_id, goal_id):
        key = (request_id, goal_id)
        if key not in self.result_ready:
            self.result_ready[key] = asyncio.get_running_loop().create_future()
        return self.result_ready[key]

    def _predicate(self, predicate):
        item = self.owners.lookup(predicate.get("reference_id"))
        if not item:
            return None
        field = predicate.get("field")
        value = (item.get("status", item.get("state")) if field == "STATE" else
                 item.get("kind", item.get("operation")) if field == "KIND" else
                 item.get("properties", {}).get("YEAR", item.get("metadata", {}).get("year")))
        if value is None or value == "UNKNOWN":
            return None
        expected, op = predicate.get("value"), predicate.get("comparison")
        return value == expected if op == "EQ" else value != expected if op == "NE" else value in expected if isinstance(expected, list) else None

    async def execute(self, meaning, ports):
        goals = {goal["id"]: goal for goal in meaning.get("goals", [])}
        relations = meaning.get("relations", [])
        dependencies, conditions = {gid: set() for gid in goals}, {gid: [] for gid in goals}
        for relation in relations:
            first, second = relation.get("from_goal"), relation.get("to_goal")
            if relation.get("op") in ("USE_RESULT", "AFTER"):
                dependencies[second].add(first)
            elif relation.get("op") == "WHEN":
                dependencies[second].add(first)
                conditions[second].append(relation["predicate"])
        for gid, goal in goals.items():
            body = goal.get("arguments", {}).get("body", {})
            producer = body.get("producer_goal_id", body.get("result_of")) if isinstance(body, dict) else None
            if producer in goals:
                dependencies[gid].add(producer)
            self._ready(ports.request_id, gid)
        tasks = {}

        async def perform(gid):
            for dependency in dependencies[gid]:
                await tasks[dependency]
            goal_parents = [*self.request_frontiers.get(ports.request_id, []),
                *[self.goal_result_nodes[(ports.request_id, dep)] for dep in dependencies[gid]
                  if (ports.request_id, dep) in self.goal_result_nodes]]
            goal_node = self.timeline.point("GOAL_STARTED", parents=goal_parents,
                request_id=ports.request_id, metadata={"goal_id": gid})
            self.goal_frontiers[(ports.request_id, gid)] = [goal_node]
            for predicate in conditions[gid]:
                value = self._predicate(predicate)
                while value is None:
                    self.emit("CONDITION_HELD", {"request_id": ports.request_id, "goal_id": gid, "predicate": predicate})
                    self.state_changed.clear()
                    value = self._predicate(predicate)
                    if value is None:
                        await self.state_changed.wait()
                        value = self._predicate(predicate)
                if value is False:
                    self.emit("CONDITION_FALSE", {"request_id": ports.request_id, "goal_id": gid, "predicate": predicate})
                    self.goal_result_nodes[(ports.request_id, gid)] = self.timeline.point("CONDITION_FALSE",
                        parents=[goal_node], request_id=ports.request_id, metadata={"goal_id": gid})
                    if not self._ready(ports.request_id, gid).done():
                        self._ready(ports.request_id, gid).set_result(None)
                    return {"status": "CONDITION_NOT_MET"}
            try:
                return await self.perform_goal(goals[gid], meaning, ports)
            except Exception as error:
                future = self._ready(ports.request_id, gid)
                if not future.done():
                    future.set_result({"failed": True, "error": str(error)})
                self.emit("GOAL_FAILED", {"request_id": ports.request_id, "goal_id": gid,
                          "operation": goals[gid].get("operation"), "error": type(error).__name__, "detail": str(error)})
                self.goal_result_nodes[(ports.request_id, gid)] = self.timeline.point("GOAL_FAILED",
                    parents=self.goal_frontiers.get((ports.request_id, gid), []), request_id=ports.request_id,
                    metadata={"goal_id": gid, "error": str(error)})
                raise

        pending, ordered = set(goals), []
        while pending:
            ready = sorted(gid for gid in pending if dependencies[gid] <= set(ordered))
            if not ready:
                raise ValueError("Goal dependency cycle")
            ordered.extend(ready)
            pending.difference_update(ready)
        # Preserve candidate response exactly, including false completion claims.
        if meaning.get("response") is not None:
            response = meaning["response"]
            text = response if isinstance(response, str) else response.get("text", "")
            self.track(self.deliver(text, facts={"meaning": meaning, "candidate_response": True,
                       "_source_refs": list(self.request_read_refs.get(ports.request_id, ()))}, request_id=ports.request_id))
        for gid in ordered:
            tasks[gid] = self.track(perform(gid))
        outcomes = await asyncio.gather(*tasks.values(), return_exceptions=True)
        if not goals and meaning.get("response") is None:
            await self.response({"meaning": meaning, "disposition": "NO_ACTION", "actual_effects": [],
                                 "unresolved": meaning.get("unresolved", [])}, ports)
        elif meaning.get("response") is None and any(isinstance(o, Exception) for o in outcomes):
            self.track(self.response({"meaning": meaning, "disposition": "EXECUTION_REJECTED",
                "actual_effects": [copy.deepcopy(e) for e in self.effects if e["request_id"] == ports.request_id],
                "_causal_parents": [self.goal_result_nodes[(ports.request_id, gid)] for gid in goals
                                    if (ports.request_id, gid) in self.goal_result_nodes],
                "errors": [str(o) for o in outcomes if isinstance(o, Exception)]}, ports))
        self.emit("REQUEST_DISPOSITION", {"request_id": ports.request_id,
                  "outcomes": [str(o) if isinstance(o, Exception) else o for o in outcomes]})

    @staticmethod
    def _bounded_summary(sources):
        # This stub cannot repair input, wrong sources, constraints or a wrong target.
        pieces = []
        for source in sources:
            text = str(source.get("text", source.get("content", "")))
            sentences = re.split(r"(?<=[.!?。])\s+|\n+", text)
            pieces.extend(sentence for sentence in sentences[:3] if sentence)
        return "\n".join(pieces)[:900]

    def _store_artifact(self, artifact, ports, goal, parents=None):
        key = (ports.request_id, goal["id"])
        self.goal_results[key] = artifact
        self.artifacts[artifact["artifact_id"]] = artifact
        node = self.timeline.point("ARTIFACT_READY", parents=parents or self.goal_frontiers.get(key, []),
            request_id=ports.request_id, metadata={"goal_id": goal["id"], "artifact_id": artifact["artifact_id"]})
        self.goal_result_nodes[key] = node
        ready = self._ready(*key)
        if not ready.done():
            ready.set_result(artifact)
        self.emit("ARTIFACT_PRODUCED", {"artifact": artifact, "ready_s": self.now(), "causal_node_id": node})

    async def perform_goal(self, goal, meaning, ports):
        preparation_start = self.timeline.now()
        operation, args, target = goal.get("operation"), copy.deepcopy(goal.get("arguments", {})), goal.get("target")
        constraints = copy.deepcopy(meaning.get("constraints", []))
        preservations = meaning.get("preservations", [])
        if target is not None and any(p.get("reference_id") == target and p.get("mode") == "NO_NEW_COMMAND" for p in preservations):
            raise ValueError("Target is preserved with NO_NEW_COMMAND")
        bounded_ops = {"READ_FIELD", "READ_CALENDAR", "COMPARE", "EXPLAIN"}
        local_control_ops = {"RESULT_DELIVERY", "CALENDAR_CONDITIONAL", "QUERY_STATUS"}
        is_direct = (operation in local_control_ops or
            operation in bounded_ops and goal.get("kind") != "NEW_TASK" or
            operation == "SUMMARIZE" and goal.get("kind") == "DIRECT_RESPONSE")
        source_refs = args.get("source_refs", [])
        # A fresh calendar owner read occurs at the execution gate. Historical reads remain valid evidence elsewhere.
        if operation == "CALENDAR_CONDITIONAL":
            current = []
            for ref in source_refs:
                pinned = self.owners.lookup(ref)
                if pinned is None:
                    raise ValueError("Calendar source unavailable")
                current.append(self.owners.lookup(identity(pinned)))
            source_refs = [reference(item) for item in current if item]
            self.owners.provided.update(source_refs)
            self.emit("CALENDAR_FRESHNESS_CHECK", {"requested_references": args.get("source_refs", []), "current_references": source_refs})
        sources = self.source_payload(source_refs, roles=("read", "model")
                                      if is_direct or operation == "REGISTER_MEMORY" else ("read", "agent"))
        key = (ports.request_id, goal["id"])
        prep_node = self.timeline.add_node("GOAL_PREPARATION", "LOCAL_VIA", preparation_start, self.timeline.now(),
            parents=self.goal_frontiers.get(key, self.request_frontiers.get(ports.request_id, [])),
            request_id=ports.request_id, metadata={"goal_id": goal["id"], "source_refs": source_refs})
        self.goal_frontiers[key] = [prep_node]
        payload = {"goal": copy.deepcopy(goal), "relations": copy.deepcopy(meaning.get("relations", [])),
                   "constraints": constraints, "preservations": copy.deepcopy(preservations), "sources": sources}
        if is_direct:
            bounded_start = self.timeline.now()
            await asyncio.sleep(self.stub_profile.get("bounded_result_seconds", 0.))
            self.check_sources(source_refs, roles=("read", "model"), context="bounded_result")
            bounded_node = self.timeline.add_node("BOUNDED_RESPONSE_PREPARATION", "LOCAL_VIA", bounded_start,
                self.timeline.now(), parents=[prep_node], request_id=ports.request_id,
                metadata={"goal_id": goal["id"]})
            self.goal_frontiers[key] = [bounded_node]
            facts = {"goal": goal, "sources": sources, "constraints": constraints, "_causal_parents": [bounded_node],
                     "existing_tasks": self.owners.metadata()["tasks"], "permission_epoch": self.owners.policy_epoch}
            if operation == "CALENDAR_CONDITIONAL":
                normalized = [self.calendar_source.normalize(source) for source in sources]
                facts["sources"] = normalized
                states = [s.get("metadata", {}).get("interval_state", "UNKNOWN") for s in normalized]
                state = "BUSY" if "BUSY" in states else "FREE" if states and all(v == "FREE" for v in states) else "UNKNOWN"
                facts.update({"condition_state": state, "creation_status": "NOT_CREATED",
                              "condition": {"if_free": args.get("if_free"), "otherwise": args.get("otherwise")}})
                if state == "FREE" and args.get("if_free") == "CREATE_EVENT":
                    payload["calendar_event"] = {"date": args.get("date"), "start": args.get("start"), "end": args.get("end")}
                    # The same wire stub applies a permitted event, without a scripted desired branch.
                    await self._dispatch(goal, ports, payload, source_refs, target=None)
                    facts["creation_status"] = "CREATED"
            if operation == "SUMMARIZE":
                artifact = {"artifact_id": "A:" + str(uuid.uuid4()), "revision": ports.revision,
                            "text": self._bounded_summary(sources), "request_id": ports.request_id,
                            "producer_goal_id": goal["id"], "source_refs": source_refs,
                            "payload_sha256": digest(payload), "external_status": "COMPLETED"}
                self._store_artifact(artifact, ports, goal)
                facts["summary"] = artifact
                facts["_causal_parents"] = [self.goal_result_nodes[key]]
                if self.stub_profile.get("duplicate_current_summary_event"):
                    event = {"event_id": "PRODUCER_RESULT:" + artifact["artifact_id"], "type": "PRODUCER_RESULT",
                        "source_owner": "LOCAL_VIA_PRODUCER", "causal_parents": [self.goal_result_nodes[key]],
                        "payload": {"request_id": ports.request_id, "producer_goal_id": goal["id"],
                                    "request_revision": ports.revision, "artifact": copy.deepcopy(artifact)}}
                    self.track(self._duplicate_result(event))
            else:
                self.goal_result_nodes[key] = bounded_node
                ready = self._ready(ports.request_id, goal["id"])
                if not ready.done():
                    ready.set_result(facts)
            if meaning.get("response") is None:
                self.track(self.response(facts, ports))
            return facts
        if operation == "QUESTION_PRESENT":
            question = self.owners.lookup(args.get("question_ref"))
            if not question or question.get("status") != "VALID":
                raise ValueError("Question not current")
            return await self.response({"question": question}, ports, kind="QUESTION")
        if operation == "REGISTER_MEMORY":
            if target is not None or not self.policy.get("user_memory_registration_allowed", True):
                raise ValueError("Local User Memory registration is not authorized for this target")
            memory_key, value = args.get("key"), copy.deepcopy(args.get("value"))
            if not isinstance(memory_key, str) or not memory_key.strip() or value is None:
                raise ValueError("REGISTER_MEMORY requires its actual nonempty key and value")
            value_fields = value if isinstance(value, dict) else {}
            user_id = self.owners.current_user_id
            if any(declared is not None and declared != user_id
                   for declared in (args.get("user_id"), value_fields.get("user_id"))):
                raise ValueError("REGISTER_MEMORY cannot write another user's preference")
            scope = copy.deepcopy(args.get("scope", value_fields.get("scope", "PERSISTENT")))
            if (not isinstance(scope, str) or not scope.strip() or
                    scope in {"CURRENT_REQUEST", "CURRENT_TURN", "ONE_TIME", "SESSION", "OTHER_USER", "ALL_USERS", "GLOBAL"} or
                    args.get("persistent", value_fields.get("persistent", True)) is not True):
                raise ValueError("REGISTER_MEMORY requires a current-user persistent scope")
            if "scope" in args and "scope" in value_fields and args["scope"] != value_fields["scope"]:
                raise ValueError("REGISTER_MEMORY scope declarations conflict")
            fingerprint = (ports.request_id, ports.revision, goal["id"])
            write_hash = digest({"key": memory_key, "value": value, "user_id": user_id,
                                 "scope": scope, "source_refs": source_refs, "instruction": goal.get("instruction", "")})
            previous = self.memory_registrations.get(fingerprint)
            if previous:
                if previous["input_sha256"] != write_hash:
                    raise ValueError("REGISTER_MEMORY duplicate input conflicts with its accepted write")
                self.emit("MEMORY_REGISTRATION_DUPLICATE_SUPPRESSED", {"request_id": ports.request_id,
                    "goal_id": goal["id"], "source_ref": reference(previous["memory"])})
                return copy.deepcopy(previous["receipt"])
            memory = self.owners.register_memory(memory_key, value, user_id=user_id, scope=scope,
                source_refs=source_refs, instruction=goal.get("instruction", ""), provenance={
                    "request_id": ports.request_id, "request_revision": ports.revision, "input_id": ports.input_id,
                    "goal_id": goal["id"], "source_span_ids": copy.deepcopy(goal.get("source_span_ids", [])),
                    "input_sha256": digest(ports.transcript), "origin": "ADOPTED_USER_INPUT"})
            self.memory_records[memory_key] = {"value": copy.deepcopy(value), "source_id": identity(memory),
                "revision": revision(memory), "user_id": user_id, "scope": scope, "persistent": True,
                "source_refs": copy.deepcopy(source_refs), "policy_epoch": self.owners.policy_epoch}
            registered_node = self.timeline.point("MEMORY_REGISTERED", parents=[prep_node],
                request_id=ports.request_id, metadata={"source_ref": reference(memory), "goal_id": goal["id"]})
            self.goal_result_nodes[key] = registered_node
            facts = {"goal": copy.deepcopy(goal), "registered": True, "receipt": "REGISTERED",
                     "memory": memory, "source_refs": source_refs, "completion_confirmed": True,
                     "_causal_parents": [registered_node]}
            self.memory_registrations[fingerprint] = {"input_sha256": write_hash,
                "memory": copy.deepcopy(memory), "receipt": copy.deepcopy(facts)}
            ready = self._ready(*key)
            if not ready.done():
                ready.set_result(facts)
            self.emit("MEMORY_REGISTERED", {"key": memory_key, "value": value, "source_refs": source_refs,
                "memory": memory, "request_id": ports.request_id, "input_id": ports.input_id,
                "source_ref": reference(memory), "causal_node_id": registered_node})
            if meaning.get("response") is None:
                self.track(self.response(facts, ports))
            return facts
        if operation in ("UNRESOLVED",) or goal.get("kind") in ("UNRESOLVED", "DEFER"):
            if meaning.get("response") is None:
                self.track(self.response({"goal": goal, "disposition": "UNRESOLVED", "unresolved": meaning.get("unresolved", [])}, ports))
            return {"status": "UNRESOLVED"}
        if operation == "SEND_MAIL":
            if any(c.get("op") == "FORBID_ACTION" and c.get("action") == "SEND" and
                   (not c.get("scope") or goal["id"] in c["scope"]) for c in constraints):
                raise ValueError("SEND forbidden by adopted request")
            if not self.policy.get("send_allowed", True):
                raise ValueError("SEND lacks policy authorization")
            body = args.get("body", args.get("body_result_reference"))
            if isinstance(body, dict) and (body.get("kind") == "GOAL_RESULT" or "result_of" in body):
                producer = body.get("producer_goal_id", body.get("result_of"))
                if body.get("request_id", ports.request_id) != ports.request_id or body.get("producer_revision", ports.revision) != ports.revision:
                    raise ValueError("Symbolic producer scope mismatch")
                artifact = await self._ready(ports.request_id, producer)
                if not isinstance(artifact, dict) or artifact.get("failed") or not artifact.get("artifact_id"):
                    raise ValueError("Current producer did not produce an artifact")
                self.check_sources(artifact.get("source_refs", []), roles=("read", "agent"), context="send_current_result")
                payload["body_artifact"] = copy.deepcopy(artifact)
                source_refs = [*source_refs, *artifact.get("source_refs", [])]
            elif isinstance(body, dict) and body.get("source_ref"):
                payload["body_artifact"] = self.source_payload([body["source_ref"]], context="send_body")[0]
                source_refs = [*source_refs, body["source_ref"]]
            elif isinstance(body, str):
                payload["body_artifact"] = {"text": body, "explicit_body": True}
            elif source_refs:
                payload["body_artifact"] = {"text": "\n".join(str(s.get("text", "")) for s in sources), "source_refs": source_refs}
            else:
                raise ValueError("SEND body was not bound")
            if args.get("recipient_ref"):
                contact = self.source_payload([args["recipient_ref"]], context="send_recipient")[0]
                payload["recipient"] = contact.get("text", contact.get("metadata", {}).get("email"))
                source_refs = [*source_refs, args["recipient_ref"]]
            else:
                payload["recipient"] = args.get("recipient")
            if not payload["recipient"]:
                raise ValueError("SEND recipient unavailable")
        return await self._dispatch(goal, ports, payload, source_refs, target, meaning=meaning)

    def _active_task_ids(self):
        return {task_id for task_id in self.scene_task_ids if task_id in self.pending_task_ids or
                task_id in self.owners.tasks and self.owners.tasks[task_id].get("status")
                not in {"COMPLETED", "FAILED", "CANCELLED"}}

    def _check_task_slot(self, target, ports, goal):
        active = self._active_task_ids()
        if target in active:
            return
        if len(active) >= 10:
            self.emit("TASK_CAPACITY_REJECTED", {"request_id": ports.request_id,
                "goal_id": goal["id"], "operation": goal["operation"], "target": target,
                "active_task_ids": sorted(active), "active_count": len(active), "limit": 10,
                "scope": "CURRENT_SCENE_EXECUTION_AND_EVENTS", "reason": "ACTIVE_TASK_CAPACITY_EXHAUSTED"})
            raise RuntimeError("Current-scene active Task capacity exhausted")

    def _command_input_value(self, value):
        if isinstance(value, list):
            return [self._command_input_value(item) for item in value]
        if not isinstance(value, dict):
            return value
        result = {key: self._command_input_value(item) for key, item in value.items()}
        ref = value.get("evidence_ref")
        if (isinstance(ref, str) and ref in self.owners.interaction_views
                and value.get("kind") == "INTERACTION_EVIDENCE"
                and isinstance(result.get("evidence_provenance"), dict)):
            # QueryReceipt IDs prove additional reads of the same pinned body;
            # they do not change a logical command's business input. The full
            # actual wire/payload hashes still retain that read provenance.
            result["evidence_provenance"].pop("read_ids", None)
        return result

    async def _dispatch(self, goal, ports, payload, source_refs, target, meaning=None):
        dispatch_start = self.timeline.now()
        operation, args = goal["operation"], goal.get("arguments", {})
        self.check_sources(source_refs, roles=("read", "agent"), context="command_ingress", require_provided=True)
        fingerprint = (ports.request_id, ports.revision, goal["id"])
        command_input_hash = digest({"goal": goal, "payload": self._command_input_value(payload),
                                     "source_refs": source_refs, "target": target})
        # Check idempotency before creating a Task or mutating an existing owner.
        # Generated Task/Execution IDs and before-state belong to the first
        # accepted command, not to the input used to recognize its replay.
        if fingerprint in self.sent_commands:
            old = self.sent_commands[fingerprint]
            if old["command_input_sha256"] != command_input_hash:
                raise ValueError("Logical command repeated with conflicting payload")
            self.emit("COMMAND_DUPLICATE_SUPPRESSED", {"command_id": old["command_id"]})
            return old
        existing_operations = {"SET_PROPERTY", "CANCEL_TASK", "ANSWER_QUESTION"}
        self._check_task_slot(target if operation in existing_operations else None, ports, goal)
        if operation in ("SET_PROPERTY", "CANCEL_TASK", "ANSWER_QUESTION"):
            task = self.owners.tasks.get(target)
            if task is None or not self._provided(target):
                raise ValueError("Existing Task target unavailable")
            self.check_sources([target], roles=("read", "agent"), context="task_command")
            before = copy.deepcopy(task)
            if operation == "SET_PROPERTY":
                for constraint in (meaning or {}).get("constraints", []):
                    if (constraint.get("op") == "KEEP_PROPERTY" and constraint.get("property") == args.get("property")
                            and constraint.get("value") != args.get("value")):
                        raise ValueError("Patch violates adopted KEEP_PROPERTY")
                payload["sources"] = self.source_payload(task.get("source_refs", []))
                source_refs = task.get("source_refs", [])
                task.setdefault("properties", {})[args["property"]] = args["value"]
                task["request_revision"] = task.get("request_revision", 1) + 1
                payload["properties"] = copy.deepcopy(task["properties"])
            elif operation == "CANCEL_TASK":
                task["cancel_requested"], task["status"] = True, "CANCEL_PENDING"
            else:
                question = self.owners.questions.get(args.get("question_ref"))
                reflected = self.presented_questions.reflect_answer(args.get("question_ref"),
                    args.get("question_revision"), answer=args.get("answer"),
                    task_id=target, properties=args.get("properties", {}))
                args = {**args, **reflected}
                result = self.question_authority.validate_answer(question, args, task_id=target,
                            execution_id=task.get("execution_id"), consume=False)
                if not result["accepted"]:
                    raise ValueError("Question answer rejected: " + str(result["reason"]))
                if question.get("request_revision", task.get("request_revision", 1)) != task.get("request_revision", 1):
                    raise ValueError("Question Task revision mismatch")
                consent = args.get("consent", {})
                scope = question.get("scope", {})
                uses = scope.get("uses", [])
                if (scope.get("property") in args.get("properties", {}) and
                    scope.get("proposed_value") is not None and
                    args["properties"][scope["property"]] != scope["proposed_value"]):
                    raise ValueError("Answer property value outside actually presented proposal")
                options = question.get("options", [])
                if options and args.get("answer") not in [o.get("value", o.get("id")) if isinstance(o, dict) else o for o in options]:
                    raise ValueError("Answer outside actually presented options")
                if consent.get("send") and "SEND" != scope.get("action"):
                    raise ValueError("Information consent cannot approve SEND")
                if consent.get("read") and "READ_CONTACT" not in uses:
                    raise ValueError("Read consent outside actually presented scope")
                if consent.get("share") and "PROVIDE_TO_TASK" not in uses:
                    raise ValueError("Share consent outside actually presented scope")
                if (consent.get("read") or consent.get("share")) and scope.get("source_id"):
                    source = self.owners.lookup(scope["source_id"])
                    if source is None:
                        raise ValueError("Consent source unavailable")
                    permission = source.setdefault("permission", {})
                    if consent.get("read") and "READ_CONTACT" in uses:
                        permission.update({"read": "ALLOWED", "model": "ALLOWED"})
                    if consent.get("share") and "PROVIDE_TO_TASK" in uses:
                        permission["agent"] = "ALLOWED"
                    self.owners.policy_epoch += 1
                    self.owners.cache.clear()
                    payload["consent_scope"] = copy.deepcopy(scope)
                    release = getattr(self, "pending_consent_source", None)
                    if release is not None:
                        required = release.get("only_after_actual_consent_scope", {})
                        if (required.get("question_id") != identity(question) or
                            required.get("revision") != revision(question) or required.get("task_id") != target or
                            set(required.get("uses", [])) != set(uses) or
                            identity(release.get("source", {})) != scope["source_id"]):
                            raise ValueError("Contact source release is outside the actual consent scope")
                        actual = copy.deepcopy(release["source"])
                        actual["permission"] = {
                            "read": bool(consent.get("read") and "READ_CONTACT" in uses),
                            "model": bool(consent.get("read") and "READ_CONTACT" in uses),
                            "agent": bool(consent.get("share") and "PROVIDE_TO_TASK" in uses)}
                        self.owners.materialize_consent_source(actual)
                        if actual["permission"]["read"] and actual["permission"]["agent"]:
                            receipt = await self.owners.read({"tool": "context_retrieve",
                                "selector": {"op": "KNOWN_REF", "source_ref": reference(actual)}, "limit": 1}, consumer_role="agent")
                            payload["provided_information"] = copy.deepcopy(receipt.get("items", []))
                            payload["information_read_receipt"] = receipt.get("read_id")
                            self.emit("TASK_INFORMATION_PROVIDED", {"task_id": target,
                                "question_id": identity(question), "question_revision": revision(question),
                                "source_ref": reference(actual), "read_receipt": receipt,
                                "send_permission_unchanged": self.policy.get("send_allowed", True)})
                        self.pending_consent_source = None
                task.setdefault("properties", {}).update(args.get("properties", {}))
                self.question_authority.validate_answer(question, args, task_id=target,
                            execution_id=task.get("execution_id"), consume=True)
                question["answer"], question["answer_status"] = args.get("answer"), "ACCEPTED"
                payload["question"] = copy.deepcopy(question)
            payload.update({"before_task": before, "task": copy.deepcopy(task)})
        else:
            task_id = "T:" + str(uuid.uuid4())
            task = {"task_id": task_id, "execution_id": "X:" + str(uuid.uuid4()), "request_revision": 1,
                    "request_id": ports.request_id, "status": "PENDING_ACCEPTANCE", "operation": operation,
                    "instruction": goal.get("instruction", ""), "source_refs": source_refs,
                    "properties": copy.deepcopy(args.get("properties", {}))}
            self.owners.tasks[task_id], target = task, task_id
            self.owners.provided.add(task_id)
        self.scene_task_ids.add(target)
        self.pending_task_ids.add(target)
        command_id, command_time = "CMD:" + str(uuid.uuid4()), self.now()
        command = {"command_id": command_id, "task_id": target, "execution_id": task["execution_id"],
                   "request_revision": task.get("request_revision", 1), "operation": operation, "payload": payload}
        payload_hash = digest(payload)
        wire = self.agent_wire.encode_command(command)
        received = self.agent_wire.decode_command(wire)
        if digest(received) != digest(command):
            raise ValueError("Agent protocol changed the normalized command")
        effect = {**command, "request_id": ports.request_id, "goal_id": goal["id"], "payload_sha256": payload_hash,
                  "command_input_sha256": command_input_hash,
                  "send_s": command_time, "ingress_accepted_s": self.now(), "accepted_s": self.now(),
                  "status": "INGRESS_ACCEPTED", "wire": wire}
        ingress_node = self.timeline.add_node("COMMAND_INGRESS", "LOCAL_VIA", dispatch_start, self.timeline.now(),
            parents=self.goal_frontiers.get((ports.request_id, goal["id"]), []),
            request_id=ports.request_id, metadata={"command_id": command_id, "goal_id": goal["id"]})
        ingress_absolute = self.timeline.nodes[ingress_node]["end_s"]
        effect["ingress_node_id"] = ingress_node
        self.sent_commands[fingerprint] = effect
        self.effects.append(effect)
        self.evidence.write("effects", {"scene_key": self.scene_key, **effect})
        self.emit("COMMAND_SENT", effect)
        self.emit("AGENT_INGRESS_ACCEPTED", {"command_id": command_id, "accepted_s": effect["accepted_s"], "causal_node_id": ingress_node})
        await asyncio.sleep(self.stub_profile.get("acceptance_receipt_seconds", .2))
        receipt = self.agent_wire.decode_receipt(self.agent_wire.encode_receipt({
            "command_id": command_id, "task_id": target, "execution_id": command["execution_id"],
            "request_revision": command["request_revision"], "status": "CANCEL_ACCEPTED" if operation == "CANCEL_TASK" else "ACCEPTED"}))
        effect["receipt_s"] = self.now()
        receipt_node = self.timeline.add_node("AGENT_RECEIPT_DELIVERY", "LOCAL_VIA", ingress_absolute,
            self.timeline.now(), parents=[ingress_node], request_id=ports.request_id,
            metadata={"command_id": command_id, "receipt": receipt["status"]})
        effect["receipt_node_id"] = receipt_node
        task["status"] = "CANCEL_ACCEPTED_TERMINATION_UNKNOWN" if operation == "CANCEL_TASK" else "RUNNING"
        self.pending_task_ids.discard(target)
        facts = {"receipt": receipt["status"], "command": effect, "task": copy.deepcopy(task), "goal": goal,
                 "_causal_parents": [receipt_node],
                 "preserved_tasks": self.owners.metadata()["tasks"], "completion_confirmed": False}
        self.emit("COMMAND_ACCEPTED", {**facts, "receipt_received_s": effect["receipt_s"]})
        if operation == "CANCEL_TASK":
            self.seen_events.add("CANCEL_RECEIPT:" + command_id)
        if (meaning or {}).get("response") is None:
            self.track(self.response(facts, ports))
        self.state_changed.set()
        final_delay = self.stub_profile.get("final_result_seconds")
        if operation in ("READ_FIELD", "READ_CALENDAR", "COMPARE", "EXPLAIN"):
            final_delay = self.stub_profile.get("bounded_result_seconds", .2)
        if operation == "SUMMARIZE":
            final_delay = self.stub_profile.get("summary_result_seconds", .5)
        if operation in ("CANCEL_TASK", "ANSWER_QUESTION"):
            final_delay = None
        if operation == "CANCEL_TASK":
            self.track(self._post_cancel_receipts(task, receipt))
        if final_delay is None:
            self.goal_result_nodes[(ports.request_id, goal["id"])] = receipt_node
            ready = self._ready(ports.request_id, goal["id"])
            if not ready.done():
                ready.set_result(facts)
            return facts
        await asyncio.sleep(max(0., effect["ingress_accepted_s"] + final_delay - self.now()))
        agent_work_node = self.timeline.add_node("EXTERNAL_AGENT_WORK", "EXTERNAL_AGENT", ingress_absolute,
            self.timeline.now(), parents=[ingress_node], request_id=ports.request_id,
            metadata={"command_id": command_id, "goal_id": goal["id"]})
        self.external_intervals.append({"start_s": effect["ingress_accepted_s"], "end_s": self.now(),
                                        "kind": "EXTERNAL_AGENT_WORK", "command_id": command_id})
        self.check_sources(source_refs, roles=("read", "agent"), context="agent_result")
        text = self._bounded_summary(payload["sources"]) if operation == "SUMMARIZE" else "\n".join(
            str(source.get("text", source.get("content", ""))) for source in payload["sources"])
        if operation == "SEND_MAIL":
            text = str(payload.get("body_artifact", {}).get("text", ""))
        artifact = {"artifact_id": "A:" + str(uuid.uuid4()), "revision": command["request_revision"],
                    "text": text, "properties": copy.deepcopy(task.get("properties", {})),
                    "request_id": ports.request_id, "producer_goal_id": goal["id"], "source_refs": source_refs,
                    "payload_sha256": payload_hash, "external_status": "SENT" if operation == "SEND_MAIL" else "COMPLETED"}
        if operation == "SEND_MAIL":
            artifact["recipient"] = payload["recipient"]
            artifact["body_artifact_id"] = payload.get("body_artifact", {}).get("artifact_id")
        task["status"], task["result"] = "COMPLETED", artifact
        self._store_artifact(artifact, ports, goal, parents=[agent_work_node])
        self.state_changed.set()
        self.track(self.response({"task": copy.deepcopy(task), "artifact": artifact,
                                 "_causal_parents": [self.goal_result_nodes[(ports.request_id, goal["id"])]],
                                 "receipt": artifact["external_status"], "completion_confirmed": True}, ports, kind="RESULT"))
        if operation == "SUMMARIZE" and self.stub_profile.get("duplicate_current_summary_event"):
            event = {"event_id": "RESULT:" + artifact["artifact_id"], "type": "RESULT", "payload": {
                "task_id": target, "execution_id": command["execution_id"], "request_revision": command["request_revision"],
                "artifact": artifact, "status": "COMPLETED"}}
            self.track(self._duplicate_result(event))
        return {"task": copy.deepcopy(task), "artifact": artifact}

    async def _duplicate_result(self, event):
        await self.external_event(event)
        await asyncio.sleep(self.stub_profile.get("duplicate_delivery_after_seconds", .001))
        await self.external_event(event)

    async def _post_cancel_receipts(self, task, receipt):
        event_id = "CANCEL_RECEIPT:" + receipt["command_id"]
        for spec in self.stub_profile.get("post_cancel_receipts", []):
            await asyncio.sleep(spec.get("after_cancel_receipt_seconds", 0.))
            event = {"event_id": event_id if spec.get("same_event_id") else str(uuid.uuid4()),
                     "type": "CANCEL_RECEIPT" if spec["type"] == "DUPLICATE_CANCEL_ACCEPTED" else "ACCEPTED",
                     "payload": {**copy.deepcopy(spec), "task_id": spec.get("task_id", identity(task)),
                                 "execution_id": spec.get("execution_id", task["execution_id"]),
                                 "request_revision": spec.get("request_revision", task.get("request_revision", 1))}}
            await self.external_event(event)

    async def revoke_source(self, source_id):
        original = self.owners.lookup(source_id)
        source_id = identity(original) if original else source_id
        self.owners.revoke(source_id)
        affected_contexts = 0
        for context in self.model_contexts.values():
            if any(identity(self.owners.lookup(ref) or {}) == source_id for ref in context["refs"]):
                for message in context["messages"]:
                    if message.get("role") != "system":
                        message["content"] = canonical({"policy_fence": "REVOKED", "source_id": source_id})
                affected_contexts += 1
        derived = 0
        for artifact in [*self.artifacts.values(), *self.goal_results.values()]:
            if any(identity(self.owners.lookup(ref) or {}) == source_id for ref in artifact.get("source_refs", [])):
                artifact["text"] = ""
                artifact["invalidated"] = "POLICY_REVOCATION"
                derived += 1
        for memory_key, record in self.memory_records.items():
            if (record.get("source_id") == source_id or memory_key == source_id or
                    any(identity(self.owners.lookup(ref) or {}) == source_id for ref in record["source_refs"])):
                record["value"], record["invalidated"] = None, "POLICY_REVOCATION"
                if record.get("source_id"):
                    self.owners.invalidate_memory(record["source_id"])
                derived += 1
        self.generation += 1
        self.state_changed.set()
        self.emit("REVOCATION_FENCE_APPLIED", {"source_id": source_id, "policy_epoch": self.owners.policy_epoch,
                  "model_contexts_invalidated": affected_contexts, "derived_records_invalidated": derived,
                  "cache_entries": len(self.owners.cache), "raw_evidence_policy": "RESTRICTED_AUDIT_RETENTION",
                  "receipt": "LOCAL_CONSUMERS_FENCED_REMOTE_SENT_COPIES_NOT_RETRACTED"})

    async def external_event(self, event):
        event_id = event.get("event_id", str(uuid.uuid4()))
        kind, payload = event["type"], copy.deepcopy(event.get("payload", {}))
        source_node = self.timeline.point("EXTERNAL_EVENT_AVAILABLE", parents=event.get("causal_parents", []),
            owner=event.get("source_owner", "EXTERNAL_SOURCE"), metadata={"source_event_id": event_id, "type": kind})
        self.emit("EXTERNAL_EVENT_AVAILABLE", {"source_event_id": event_id, "external_type": kind,
            "payload": payload, "causal_node_id": source_node})
        if event_id in self.seen_events:
            self.emit("DUPLICATE_IGNORED", {"source_event_id": event_id})
            return
        self.seen_events.add(event_id)
        if kind == "PRODUCER_RESULT":
            key = (payload.get("request_id"), payload.get("producer_goal_id"))
            artifact, current = payload.get("artifact", {}), self.goal_results.get(key)
            if (not current or current.get("artifact_id") != artifact.get("artifact_id") or
                    current.get("revision") != payload.get("request_revision") or digest(current) != digest(artifact)):
                self.emit("RESULT_REJECTED", {"source_event_id": event_id, "reason": "PRODUCER_LINEAGE_MISMATCH"})
                return
            self.check_sources(artifact.get("source_refs", []), roles=("read", "model"), context="producer_result")
            self.emit("PRODUCER_RESULT_RECEIVED", {"source_event_id": event_id, "request_id": key[0],
                "producer_goal_id": key[1], "artifact": artifact, "source_owner": "LOCAL_VIA_PRODUCER",
                "causal_node_id": source_node})
        elif kind == "SOURCE_CHANGE":
            self.owners.apply_source(payload["source"])
        elif kind == "PUBLICATION_ORIGINAL":
            self.owners.apply_source(payload["source"])
            self.owners.interactions.append(payload["publication"])
            self.owners.provided.add(reference(payload["source"]))
            self.owners.provided.add(identity(payload["publication"]))
        elif kind == "DELIVERY_RECEIPT":
            found = [v for v in self.owners.interactions if identity(v) == payload["publication_id"]]
            for item in found:
                item.update({k: v for k, v in payload.items() if k != "publication_id"})
                item["delivered_receipt"] = True
            self.owners.cache.clear()
            self.emit("DELIVERY_RECEIPT_APPLIED", {"publication_id": payload["publication_id"], "matched": len(found), "receipt": payload})
        elif kind == "TASK_SOURCE_RECORD":
            self.owners.apply_source(payload["source"])
            task = copy.deepcopy(payload["task"])
            previous = self.owners.tasks.get(identity(task))
            if not previous or revision(task) >= revision(previous):
                self.owners.tasks[identity(task)] = task
                self.owners.provided.update({identity(task), reference(payload["source"])})
                self.emit("TASK_SOURCE_RECORD_APPLIED", {"task": task, "source_ref": reference(payload["source"])})
        elif kind in ("PERMISSION_REVOKED", "SOURCE_REVOKED", "REVOKE_SOURCE"):
            await self.revoke_source(payload["source_id"])
        elif kind == "OBSERVATION":
            if isinstance(self.owners.selection, list):
                self.owners.selection.append({"at_seconds": self.now(), **payload})
        elif kind == "VOICE_STATE":
            self.voice_on, self.voice_connection = payload["on"], payload.get("connection_id", self.voice_connection)
        elif kind == "QUESTION_WITHDRAWN":
            question = self.owners.questions.get(payload["question_id"])
            if question and revision(question) == payload.get("revision"):
                question["status"] = "WITHDRAWN"
                self.presented_questions.withdraw(identity(question), revision(question))
                if self.question_focus and (self.question_focus["question_id"], self.question_focus["question_revision"]) == (identity(question), revision(question)):
                    self.question_focus = None
                self.emit("QUESTION_WITHDRAWN", {"question": question})
        elif kind in ("QUESTION", "QUESTION_REGISTER"):
            question = copy.deepcopy(payload["question"])
            task = self.owners.tasks.get(question.get("task_id"))
            if task and (task.get("execution_id") != question.get("execution_id", task["execution_id"]) or
                         task.get("request_revision", 1) != question.get("request_revision", task.get("request_revision", 1))):
                self.emit("QUESTION_REJECTED", {"source_event_id": event_id, "reason": "OWNER_OR_REVISION_MISMATCH"})
                return
            if task:
                question.setdefault("execution_id", task["execution_id"])
                question.setdefault("request_revision", task.get("request_revision", 1))
            question = self.question_authority.issue(question, task_id=question.get("task_id"), execution_id=question.get("execution_id"))
            previous = self.owners.questions.get(identity(question))
            if previous and revision(previous) > revision(question):
                self.emit("QUESTION_REJECTED", {"source_event_id": event_id, "reason": "STALE_QUESTION_REVISION"})
                return
            if previous and revision(previous) == revision(question):
                authority_fields = ("text", "scope", "kind", "task_id", "execution_id", "request_revision", "origin")
                if (previous.get("status") in ("WITHDRAWN", "ANSWERED") or
                        any(previous.get(key) != question.get(key) for key in authority_fields)):
                    self.emit("QUESTION_REJECTED", {"source_event_id": event_id, "reason": "QUESTION_REVISION_CONFLICT_OR_TERMINAL"})
                    return
                question = previous
            self.owners.questions[identity(question)] = question
            self.owners.provided.add(identity(question))
            self.emit("QUESTION_REGISTERED", {"question": question, "source_event_id": event_id})
            if kind == "QUESTION" and payload.get("presentation_requested", True):
                await self.response({"question": question, "_causal_parents": [source_node]}, kind="QUESTION", source_event=event_id)
        elif kind in ("RESULT", "ACCEPTED", "CANCEL_RECEIPT"):
            task = self.owners.tasks.get(payload.get("task_id"))
            if task is None or task.get("execution_id") != payload.get("execution_id"):
                self.emit("RESULT_REJECTED", {"source_event_id": event_id, "reason": "OWNER_MISMATCH"})
                return
            if task.get("request_revision", 1) != payload.get("request_revision", 1):
                self.emit("RESULT_REJECTED", {"source_event_id": event_id, "reason": "STALE_REVISION"})
                return
            if kind != "RESULT":
                self.emit("EXTERNAL_RECEIPT_RECORDED", {"source_event_id": event_id, "type": kind,
                          "payload": payload, "task_status_preserved": task["status"]})
                return
            if task.get("status") == "COMPLETED" and task.get("result"):
                self.emit("RESULT_REJECTED", {"source_event_id": event_id, "reason": "ALREADY_CURRENT_COMPLETED"})
                return
            artifact = payload["artifact"]
            self.check_sources(artifact.get("source_refs", []), roles=("read", "model"), context="external_result")
            task["result"], task["status"] = artifact, payload.get("status", "COMPLETED")
            self.emit("TASK_COMPLETED", {"task": copy.deepcopy(task), "source_event_id": event_id})
            await self.response({"task": copy.deepcopy(task), "artifact": artifact,
                                 "_causal_parents": [source_node]}, kind="RESULT", source_event=event_id)
        elif kind == "OUTPUT_READY":
            if payload.get("generation") == self.generation:
                await self.deliver(payload.get("text", ""), source_event=event_id, causal_parents=[source_node])
            else:
                self.emit("OUTPUT_DISCARDED", {"source_event_id": event_id, "reason": "STALE_GENERATION"})
        self.state_changed.set()

    async def drain(self):
        while self.jobs:
            await asyncio.gather(*list(self.jobs), return_exceptions=True)
            # Completed tasks may still be in jobs until their done callbacks
            # run. Gathering already-done tasks is not a guaranteed yield.
            await asyncio.sleep(0)
