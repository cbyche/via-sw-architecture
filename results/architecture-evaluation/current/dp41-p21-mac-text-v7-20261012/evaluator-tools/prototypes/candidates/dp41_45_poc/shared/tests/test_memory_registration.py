"""Local memory ownership/wiring checks with specified model outputs and API 0."""
import asyncio
import copy
import json
import time
import unittest

from prototypes.candidates.dp41_45_poc.dp41.contract import FRAME_VERSION, MEANING_VERSION
from prototypes.candidates.dp41_45_poc.shared.evidence import digest
from prototypes.candidates.dp41_45_poc.shared.owners import reference
from prototypes.candidates.dp41_45_poc.shared.runtime import LocalVIA, Ports, PolicyFenceError


class Evidence:
    def __init__(self): self.origin, self.rows = time.monotonic(), []
    def seconds(self): return time.monotonic() - self.origin
    def write(self, table, row):
        self.rows.append((table, copy.deepcopy(row)))
        return row


def source(sid="allowed-document", **extra):
    return {"source_id": sid, "revision": 1, "kind": "DOCUMENT", "title": sid,
            "text": "Actual allowed original.", "metadata": {},
            "permission": {"read": True, "model": True, "agent": True}, **extra}


def goal(key="display-style", value=None, refs=None, **args):
    return {"id": "register", "kind": "DIRECT_RESPONSE", "operation": "REGISTER_MEMORY", "target": None,
            "arguments": {"key": key, "value": copy.deepcopy(value if value is not None else {
                "user_id": "local-user", "scope": "FUTURE_EXPLANATIONS", "style": "BRIEF"}),
                "source_refs": refs or [], **args}, "instruction": "Store the stated local-user preference"}


def document(g, response=None):
    return {"version": MEANING_VERSION, "goals": [g], "relations": [], "constraints": [],
            "preservations": [], "response": response, "evidence_refs": [], "unresolved": []}


class ScriptedModel:
    def __init__(self, variant="A", proposal=None):
        self.variant, self.proposal, self.calls = variant, proposal, []
    async def call(self, messages, role, **kwargs):
        self.calls.append({"messages": copy.deepcopy(messages), "role": role})
        if role == "CRESPONSE":
            facts = json.loads(messages[-1]["content"])
            if not facts.get("registered") or facts.get("receipt") != "REGISTERED":
                return {"text": "Actual rejection facts were supplied."}
            return {"text": "Stored actual value: " + json.dumps(facts["memory"]["value"], ensure_ascii=False)}
        result = copy.deepcopy(self.proposal)
        if self.variant == "A": return {"action": "PROPOSE", "meaning": result}
        result.update({"version": FRAME_VERSION, "references": []})
        return result


class MemoryRegistrationTests(unittest.IsolatedAsyncioTestCase):
    def build(self, variant="A", sources=None, proposal=None, **initial):
        language = source("language", kind="USER_MEMORY", text="Explanation language=Korean",
            metadata={"user_id": "local-user", "scope": "EXPLANATION_LANGUAGE", "value": "Korean"})
        evidence, model = Evidence(), ScriptedModel(variant, proposal or document(goal()))
        via = LocalVIA({"sources": sources if sources is not None else [language, source()],
                        "tasks": [], "voice": {"on": False}, "current_user_id": "local-user", **initial},
                       model, evidence, variant, voice_profile={"output_onset_seconds": .001,
                           "characters_per_second": 10000., "stop_seconds": .001})
        self.runtimes = [*getattr(self, "runtimes", []), via]
        return via, Ports(via, "request", "input", "Explicit actual preference"), model, evidence

    async def asyncTearDown(self):
        for via in getattr(self, "runtimes", []):
            for pending in list(via.jobs): pending.cancel()
            await asyncio.gather(*list(via.jobs), return_exceptions=True)

    async def test_actual_a_and_b_adoption_publishes_common_original_and_response_none_receipt(self):
        for variant in ("A", "B"):
            with self.subTest(variant=variant):
                via, ports, model, evidence = self.build(variant)
                language_before = copy.deepcopy(via.owners.lookup("language@v1"))
                self.assertEqual("Korean", via.memory_records["language"]["value"])
                outcome = await via.resolve(ports.transcript, request_id=ports.request_id, input_id=ports.input_id)
                await asyncio.wait_for(via.drain(), .5)
                self.assertEqual("ADOPTED", outcome["status"])
                read = await ports.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": "display-style"}})
                self.assertEqual(1, len(read["items"]))
                memory = read["items"][0]["metadata"]
                self.assertEqual(goal()["arguments"]["value"], memory["value"])
                self.assertEqual(("local-user", "FUTURE_EXPLANATIONS", True),
                    (memory["user_id"], memory["scope"], memory["persistent"]))
                self.assertTrue(read["items"][0]["ref"].startswith("via-user-memory:local-user:display-style@v1"))
                self.assertEqual(digest(ports.transcript), memory["registration_provenance"]["input_sha256"])
                self.assertEqual(ports.input_id, memory["registration_provenance"]["input_id"])
                self.assertEqual(language_before, via.owners.lookup("language@v1"))
                self.assertFalse(via.effects)
                self.assertFalse(via.owners.tasks)
                self.assertEqual(1, len([r for table, r in evidence.rows if table == "events" and r["kind"] == "MEMORY_REGISTERED"]))
                self.assertEqual(1, len(via.deliveries))
                self.assertTrue(via.deliveries[0]["delivered_receipt"])
                response = next(c for c in model.calls if c["role"] == "CRESPONSE")
                facts = json.loads(response["messages"][-1]["content"])
                self.assertEqual(memory["value"], facts["memory"]["value"])
                self.assertEqual("REGISTERED", facts["receipt"])
                self.assertEqual(["CINTERPRET", "CRESPONSE"], [c["role"] for c in model.calls])

    async def test_metadata_exposes_identity_and_scope_but_value_requires_owner_read(self):
        other = source("other-memory", kind="USER_MEMORY", text="Private other-user value",
            metadata={"user_id": "other-user", "value": "Private other-user value", "scope": "PERSISTENT"})
        current = source("own-memory", kind="USER_MEMORY", text="Own actual value",
            metadata={"user_id": "local-user", "value": "Own actual value", "scope": "PERSISTENT"})
        via, ports, _, _ = self.build(sources=[current, other])
        indexed = json.dumps(via.owners.metadata(), ensure_ascii=False)
        self.assertNotIn("Own actual value", indexed)
        self.assertNotIn("Private other-user value", indexed)
        self.assertNotIn("other-memory", json.dumps(via.owners.metadata()["sources"]))
        self.assertIsNone(via.owners.lookup("other-memory@v1"))
        denied = await ports.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": "other-memory"}})
        self.assertFalse(denied["items"])
        all_read = await ports.read({"tool": "context_retrieve", "selector": {"op": "EXPLICIT_SOURCE", "name": "other-memory"}})
        self.assertFalse(all_read["items"])
        own = await ports.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": "own-memory"}})
        self.assertEqual("Own actual value", own["items"][0]["content"])

    async def test_wrong_user_transient_scope_target_and_policy_reject_without_repair(self):
        via, ports, _, _ = self.build()
        before = copy.deepcopy(via.owners.sources)
        cases = [goal(user_id="other-user"), goal(value={"user_id": "other-user", "style": "wrong"}),
                 goal(scope="CURRENT_REQUEST"), goal(persistent=False),
                 goal(scope="PERSISTENT", value={"scope": "FUTURE_EXPLANATIONS", "style": "BRIEF"}),
                 {**goal(), "target": "work"}]
        for g in cases:
            with self.assertRaises(ValueError):
                await via.perform_goal(g, document(g), ports)
        self.assertEqual(before, via.owners.sources)
        self.assertNotIn("display-style", via.memory_records)
        via.policy["user_memory_registration_allowed"] = False
        with self.assertRaises(ValueError): await via.perform_goal(goal(), document(goal()), ports)

    async def test_duplicate_write_is_idempotent_and_new_request_update_retains_revision_history(self):
        via, ports, _, evidence = self.build()
        first = await via.perform_goal(goal(), document(goal(), response="supplied response"), ports)
        first_ref = reference(first["memory"])
        cached = await ports.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": first_ref}})
        duplicate = await via.perform_goal(goal(), document(goal()), ports)
        self.assertEqual(first["memory"], duplicate["memory"])
        self.assertEqual(1, len([r for table, r in evidence.rows if table == "events" and r["kind"] == "MEMORY_REGISTERED"]))
        conflict = goal(value={"style": "Different actual value", "scope": "FUTURE_EXPLANATIONS"})
        with self.assertRaises(ValueError): await via.perform_goal(conflict, document(conflict), ports)
        update = await via.perform_goal(conflict, document(conflict, response="supplied response"),
            Ports(via, "next-request", "next-input", "Actual explicit change"))
        second_ref = reference(update["memory"])
        self.assertNotEqual(first_ref, second_ref)
        self.assertEqual(2, update["memory"]["revision"])
        self.assertEqual(goal()["arguments"]["value"], via.owners.lookup(first_ref)["value"])
        self.assertEqual(conflict["arguments"]["value"], via.owners.lookup("display-style")["value"])
        latest = await ports.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": "display-style"}})
        self.assertEqual(second_ref, latest["items"][0]["ref"])
        self.assertFalse(latest["items"][0]["cache_hit"])
        self.assertTrue(cached["items"])

    async def test_local_registration_uses_model_rights_and_parent_agent_floor_is_preserved(self):
        doc = source(permission={"read": True, "model": True, "agent": False})
        via, ports, _, _ = self.build(sources=[doc])
        g = goal(refs=["allowed-document@v1"])
        facts = await via.perform_goal(g, document(g, response="supplied response"), ports)
        ref = reference(facts["memory"])
        read = await ports.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": ref}})
        self.assertTrue(read["items"])
        denied = await via.owners.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": ref}}, consumer_role="agent")
        self.assertFalse(denied["items"])
        with self.assertRaises(PolicyFenceError): via.source_payload([ref], roles=("read", "agent"))

    async def test_parent_revocation_deletes_published_values_and_fences_all_consumers_without_resurrection(self):
        via, ports, _, evidence = self.build()
        unrelated = await via.perform_goal(goal(key="independent", value="Actual unrelated preference"),
            document(goal(), response="supplied response"), Ports(via, "other-request", "other-input", "Explicit unrelated preference"))
        g = goal(refs=["allowed-document@v1"])
        first = await via.perform_goal(g, document(g, response="supplied response"), ports)
        ref = reference(first["memory"])
        await ports.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": ref}})
        via.request_read_refs[ports.request_id] = {ref}
        await via.revoke_source("allowed-document")
        self.assertIsNone(via.memory_records["display-style"]["value"])
        self.assertEqual("", via.owners.lookup(ref)["text"])
        self.assertIsNone(via.owners.lookup(ref)["value"])
        for requested in (ref, "display-style", "display-style@v1"):
            read = await ports.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": requested}})
            self.assertFalse(read["items"])
        with self.assertRaises(PolicyFenceError): await ports.model([{"role": "user", "content": "Consume stale value"}])
        with self.assertRaises(PolicyFenceError): via.source_payload([ref])
        rejected = await via.deliver("Prepared stale value", facts={"memory": first["memory"]})
        self.assertFalse(rejected["delivered_receipt"])
        via.owners.apply_source(source(revision=2, text="New permitted original"))
        self.assertFalse((await ports.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": ref}}))["items"])
        self.assertEqual("Actual unrelated preference", via.owners.lookup(reference(unrelated["memory"]))["value"])
        self.assertEqual("Korean", via.memory_records["language"]["value"])
        receipt = [r for _, r in evidence.rows if r["kind"] == "REVOCATION_FENCE_APPLIED"][-1]
        self.assertEqual("LOCAL_CONSUMERS_FENCED_REMOTE_SENT_COPIES_NOT_RETRACTED", receipt["receipt"])

    async def test_explicit_memory_withdrawal_alias_fences_historical_revisions(self):
        via, ports, _, _ = self.build()
        first = await via.perform_goal(goal(), document(goal(), response="supplied response"), ports)
        g = goal(value={"style": "Actual updated style", "scope": "FUTURE_EXPLANATIONS"})
        second = await via.perform_goal(g, document(g, response="supplied response"),
            Ports(via, "next-request", "next-input", "Change actual preference"))
        await via.revoke_source("display-style")
        for ref in (reference(first["memory"]), reference(second["memory"]), "display-style"):
            self.assertFalse((await ports.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": ref}}))["items"])
        self.assertIsNone(via.memory_records["display-style"]["value"])
        self.assertEqual("Korean", via.owners.lookup("language@v1")["metadata"]["value"])

    async def test_transitive_registered_provenance_scrubs_pending_context_and_published_derivative(self):
        via, ports, model, _ = self.build()
        first_goal = goal(key="first-derived", refs=["allowed-document@v1"])
        first = await via.perform_goal(first_goal, document(first_goal, response="supplied response"), ports)
        second_goal = goal(key="second-derived", value="Actual derived relation", refs=[reference(first["memory"])])
        second_ports = Ports(via, "second-request", "second-input", "Explicit derived registration")
        second = await via.perform_goal(second_goal, document(second_goal, response="supplied response"), second_ports)
        second_ref = reference(second["memory"])
        await second_ports.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": second_ref}})
        gate, entered = asyncio.Event(), asyncio.Event()
        original_call = model.call
        async def held_call(messages, role, **kwargs):
            entered.set()
            await gate.wait()
            return await original_call(messages, role, **kwargs)
        model.call = held_call
        messages = [{"role": "user", "content": "Actual derived relation"}]
        pending = asyncio.create_task(second_ports.model(messages))
        await asyncio.wait_for(entered.wait(), .3)
        await via.revoke_source("allowed-document")
        self.assertIn("REVOKED", messages[0]["content"])
        gate.set()
        with self.assertRaises(PolicyFenceError): await pending
        self.assertIsNone(via.memory_records["first-derived"]["value"])
        self.assertIsNone(via.memory_records["second-derived"]["value"])
        self.assertEqual("", via.owners.lookup(second_ref)["text"])
        self.assertFalse((await second_ports.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": second_ref}}))["items"])

    async def test_source_capacity_or_owner_collision_fails_without_private_partial_write(self):
        via, ports, _, _ = self.build(sources=[source()], max_sources=1)
        with self.assertRaises(ValueError): await via.perform_goal(goal(), document(goal()), ports)
        self.assertNotIn("display-style", via.memory_records)
        self.assertFalse(via.memory_registrations)
        with self.assertRaises(ValueError):
            await via.perform_goal(goal(key="allowed-document"), document(goal()), ports)
        self.assertEqual("Actual allowed original.", via.owners.lookup("allowed-document")["text"])

    async def test_candidate_response_is_preserved_and_not_replaced_by_receipt_text(self):
        proposal = document(goal(), response="Candidate's actual unverified statement")
        via, ports, model, _ = self.build(proposal=proposal)
        await via.resolve(ports.transcript, request_id=ports.request_id, input_id=ports.input_id)
        await via.drain()
        self.assertEqual("Candidate's actual unverified statement", via.deliveries[0]["text"])
        self.assertEqual(["CINTERPRET"], [c["role"] for c in model.calls])

    async def test_same_legacy_memory_key_in_different_users_selects_only_current_owner(self):
        other = source("same-key", kind="USER_MEMORY", text="Other-user original",
            metadata={"user_id": "other-user", "scope": "PERSISTENT", "value": "OTHER"})
        own = source("same-key", kind="USER_MEMORY", text="Current-user original",
            metadata={"user_id": "local-user", "scope": "PERSISTENT", "value": "CURRENT"})
        via, ports, _, _ = self.build(sources=[other, own])
        result = await ports.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": "same-key"}})
        self.assertEqual("Current-user original", result["items"][0]["content"])
        self.assertEqual("CURRENT", via.memory_records["same-key"]["value"])
        self.assertEqual(other, via.owners.sources[0])
        await via.revoke_source("same-key")
        self.assertEqual(other, via.owners.sources[0])
        self.assertFalse((await ports.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": "same-key"}}))["items"])
