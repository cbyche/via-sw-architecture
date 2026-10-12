"""Common evidence-scope and current-scene capacity regressions, without model APIs."""
import asyncio
import copy
import json
import unittest

from prototypes.candidates.dp41_45_poc.dp41 import a, b
from prototypes.candidates.dp41_45_poc.dp41.contract import FRAME_VERSION, MEANING_VERSION
from prototypes.candidates.dp41_45_poc.shared.runtime import LocalVIA, Ports, PolicyFenceError
from prototypes.candidates.dp41_45_poc.shared.tests.test_runtime import Evidence, Model, source, task, goal, meaning


class CommonEvidenceScopeTests(unittest.IsolatedAsyncioTestCase):
    def build(self, *, interactions=None, tasks=None, sources=None, variant="A"):
        full = "실제로 전달한 앞부분. 재생하지 않은 뒷부분."
        audible = "실제로 전달한 앞부분."
        initial = {"sources": sources if sources is not None else [source("same-id", text=full, kind="PUBLICATION_ORIGINAL")],
                   "tasks": tasks or [], "voice": {"on": False},
                   "interactions": interactions if interactions is not None else [{"publication_id": "same-id",
                       "revision": 1, "text": full, "delivered_voice_text": audible, "voice_on": True,
                       "text_presented": True, "delivered_receipt": True, "interrupted": True}]}
        self.evidence = Evidence()
        self.runtime = LocalVIA(initial, Model(), self.evidence, variant,
            voice_profile={"output_onset_seconds": .001, "characters_per_second": 10000., "stop_seconds": .001},
            stub_profile={"acceptance_receipt_seconds": .001, "final_result_seconds": None})
        return self.runtime, Ports(self.runtime, "request", "input", "이전에 실제 전달된 자료로 문서를 만들어 줘.")

    async def asyncTearDown(self):
        if hasattr(self, "runtime"):
            for pending in list(self.runtime.jobs):
                pending.cancel()
            await asyncio.gather(*list(self.runtime.jobs), return_exceptions=True)

    async def read(self, ports, op="PAST_DELIVERED"):
        return await ports.read({"tool": "interaction_retrieve",
            "selector": {"op": op, "publication_id": "same-id"}, "limit": 1})

    async def test_namespace_collision_preserves_actual_scope_through_both_candidate_wire_paths(self):
        for variant, candidate in (("A", a), ("B", b)):
            runtime, ports = self.build(variant=variant)
            model_steps = []
            async def interpretation_model(messages, role="CINTERPRET"):
                model_steps.append(copy.deepcopy(messages))
                if variant == "B":
                    return {"version": FRAME_VERSION, "input_ref": "input", "references": [{"id": "heard",
                        "selector": {"op": "PAST_DELIVERED", "publication_id": "same-id"}, "cardinality": "ONE"}],
                        "goals": [{**goal(operation="CREATE_REPORT", kind="NEW_TASK", arguments={"source_refs": ["heard"]}),
                                   "target_ref": None}], "relations": [], "constraints": [], "preservations": [],
                        "response": None, "unresolved": []}
                if len(model_steps) == 1:
                    return {"action": "READ", "reads": [{"tool": "interaction_retrieve",
                        "selector": {"op": "PAST_DELIVERED", "publication_id": "same-id"}, "limit": 1}]}
                receipt = json.loads(messages[-1]["content"])["bounded_read_results"][0]
                return {"action": "PROPOSE", "meaning": meaning([goal(operation="CREATE_REPORT", kind="NEW_TASK",
                    arguments={"source_refs": [receipt["items"][0]["ref"]]})], evidence_refs=[receipt["read_id"]])}
            ports.model = interpretation_model
            proposed = await candidate.resolve(runtime.interpretation_input(ports), ports)
            self.assertEqual(1, len(proposed["goals"]))
            runtime.validate_adoption(proposed, ports)
            await runtime.execute(proposed, ports)
            await runtime.drain()
            wire_source = runtime.agent_wire.decode_command(runtime.effects[0]["wire"])["payload"]["sources"][0]
            self.assertEqual("실제로 전달한 앞부분.", wire_source["text"])
            self.assertNotIn("재생하지 않은 뒷부분", json.dumps(wire_source, ensure_ascii=False))
            self.assertEqual("Response Manager", wire_source["evidence_provenance"]["owner"])
            self.assertEqual("ACTUAL_VOICE", wire_source["evidence_provenance"]["scope"])
            self.assertEqual(1, wire_source["evidence_provenance"]["revision"])
            self.assertNotEqual("same-id@v1", wire_source["evidence_ref"])
            self.assertEqual("PUBLICATION_ORIGINAL", runtime.owners.lookup("same-id@v1")["kind"])
            # A new actual read receipt must not become a conflicting command
            # or allocate another Task when its body/scope is unchanged.
            await self.read(ports)
            duplicate = await runtime.perform_goal(proposed["goals"][0], proposed, ports)
            self.assertEqual(runtime.effects[0]["command_id"], duplicate["command_id"])
            self.assertEqual(1, len(runtime.effects))
            self.assertEqual(1, len(runtime.owners.tasks))
            recent = (await self.read(ports, "RECENT_INTERACTION"))["items"][0]["ref"]
            changed_scope = copy.deepcopy(proposed["goals"][0])
            changed_scope["arguments"]["source_refs"] = [recent]
            with self.assertRaisesRegex(ValueError, "conflicting payload"):
                await runtime.perform_goal(changed_scope, proposed, ports)
            runtime.owners.interactions.append({**runtime.owners.interactions[0], "revision": 2,
                "text": "새 전체 문장", "delivered_voice_text": "새로 실제 전달된 내용"})
            new_read = await ports.read({"tool": "interaction_retrieve", "selector": {
                "op": "PAST_DELIVERED", "publication_id": "same-id", "revision": 2}, "limit": 1})
            changed_body = copy.deepcopy(proposed["goals"][0])
            changed_body["arguments"]["source_refs"] = [new_read["items"][0]["ref"]]
            with self.assertRaisesRegex(ValueError, "conflicting payload"):
                await runtime.perform_goal(changed_body, proposed, ports)
            self.assertEqual(1, len(runtime.effects))
            self.assertEqual(1, len(runtime.owners.tasks))

    async def test_recent_interaction_retains_distinct_text_and_voice_and_scope_identity(self):
        runtime, ports = self.build()
        past = (await self.read(ports))["items"][0]
        recent = (await self.read(ports, "RECENT_INTERACTION"))["items"][0]
        self.assertNotEqual(past["ref"], recent["ref"])
        record = runtime.source_payload([recent["ref"]])[0]
        self.assertIn("재생하지 않은 뒷부분", record["text"])
        self.assertNotIn("재생하지 않은 뒷부분", record["delivered_voice_text"])
        self.assertEqual("INTERACTION_CHANNELS", record["evidence_provenance"]["scope"])
        self.assertEqual(record["text"], recent["content"]["text"])
        self.assertEqual(record["delivered_voice_text"], recent["content"]["delivered_voice_text"])

    async def test_actual_voice_off_text_and_zero_voice_on_are_not_interchanged(self):
        for voice_on in (False, True):
            runtime, ports = self.build(interactions=[{"publication_id": "same-id", "revision": 1,
                "text": "실제 표시한 Text", "voice_on": voice_on, "text_presented": True,
                "delivered_voice_text": "", "delivered_receipt": not voice_on}])
            item = (await self.read(ports))["items"][0]
            actual = runtime.source_payload([item["ref"]])[0]
            self.assertEqual("" if voice_on else "실제 표시한 Text", actual["text"])
            self.assertEqual("ACTUAL_VOICE" if voice_on else "ACTUAL_TEXT", actual["evidence_provenance"]["scope"])
        runtime, ports = self.build(interactions=[])
        delivery = await runtime.deliver("새로 실제 표시한 Text", request_id="request")
        actual = runtime.owners.interactions[-1]
        self.assertFalse(actual["voice_on"])
        self.assertEqual(delivery["text_presented_s"], actual["text_presented_s"])
        self.assertIsNone(actual["voice_onset_s"])
        receipt = await ports.read({"tool": "interaction_retrieve", "selector": {
            "op": "PAST_DELIVERED", "publication_id": delivery["publication_id"]}, "limit": 1})
        self.assertEqual("새로 실제 표시한 Text", runtime.source_payload([receipt["items"][0]["ref"]])[0]["text"])

    async def test_scoped_handle_requires_actual_read_and_cannot_change_revision_or_body(self):
        runtime, ports = self.build()
        item = (await self.read(ports))["items"][0]
        handle = item["ref"]
        self.assertIsNone(runtime.owners.lookup({"ref": handle, "revision": 2}))
        forged = handle[:-1] + ("0" if handle[-1] != "0" else "1")
        self.assertIsNone(runtime.owners.lookup(forged))
        for alias in ("question_id", "artifact_id", "result_id", "source_id", "publication_id", "task_id",
                      "handle", "source_ref", "question_ref", "ref", "id"):
            self.assertEqual(handle, runtime.owners.lookup({alias: handle, "revision": 1})["evidence_ref"])
            self.assertIsNone(runtime.owners.lookup({alias: forged, "revision": 1}))
            self.assertIsNone(runtime.owners.lookup({alias: handle, "revision": 2}))
            with self.assertRaises(PolicyFenceError):
                runtime.source_payload([{alias: forged, "revision": 1}])
        self.assertIsNone(runtime.owners.lookup({"source_id": {"ref": forged}, "revision": 1}))
        self.assertIsNone(runtime.owners.lookup({"source_id": {"ref": handle, "revision": 2}}))
        with self.assertRaises(PolicyFenceError):
            runtime.source_payload([forged])
        runtime.owners.interactions[0]["delivered_voice_text"] = "같은 revision을 변조한 범위"
        self.assertIsNone(runtime.owners.lookup(handle))
        with self.assertRaises(PolicyFenceError):
            runtime.source_payload([handle])

    async def test_historical_scope_survives_new_revision_but_current_permissions_fence_consumers(self):
        runtime, ports = self.build()
        handle = (await self.read(ports))["items"][0]["ref"]
        newer = {**runtime.owners.interactions[0], "revision": 2, "text": "새 전체 문장", "delivered_voice_text": "새 실제 범위"}
        runtime.owners.interactions.append(newer)
        self.assertEqual("실제로 전달한 앞부분.", runtime.source_payload([handle])[0]["text"])
        newer["permission"] = {"read": True, "model": True, "agent": False}
        runtime.check_sources([handle], roles=("read", "model"))
        with self.assertRaises(PolicyFenceError):
            runtime.source_payload([handle])
        newer["permission"] = {"read": False, "model": False, "agent": False}
        with self.assertRaises(PolicyFenceError):
            runtime.check_sources([handle], roles=("read", "model"))

    async def test_publication_revision_pin_does_not_confuse_backing_source_revision(self):
        runtime, ports = self.build(sources=[source("same-id", kind="PUBLICATION_ORIGINAL"), source("backing-doc")])
        runtime.owners.interactions.append({**runtime.owners.interactions[0], "revision": 2,
            "text": "새 전체 Text", "delivered_voice_text": "새 실제 전달 범위"})
        for selector in ({"publication_id": "same-id@v1", "revision": 2}, {"ref": "same-id@v1", "revision": 2}):
            read = await ports.read({"tool": "interaction_retrieve", "selector": {"op": "PAST_DELIVERED", **selector}, "limit": 1})
            self.assertEqual("NO_MATCH", read["status"])
        actual = await ports.read({"tool": "interaction_retrieve", "selector": {
            "op": "PAST_DELIVERED", "publication_id": "same-id", "revision": 2}, "limit": 1})
        self.assertEqual(2, actual["items"][0]["revision"])
        self.assertEqual("새 실제 전달 범위", runtime.source_payload([actual["items"][0]["ref"]])[0]["text"])
        runtime.owners.interactions.append({"publication_id": "backing-publication", "revision": 2,
            "text": "관련 source의 표시", "delivered_voice_text": "관련 source의 실제 발화", "text_presented": True,
            "delivered_receipt": True, "source_refs": ["backing-doc@v1"]})
        for selector in ({"source_ref": "backing-doc@v1"}, {"source_ref": "backing-doc@v1", "revision": 2}):
            read = await ports.read({"tool": "interaction_retrieve", "selector": {"op": "PAST_DELIVERED", **selector}, "limit": 1})
            record = runtime.source_payload([read["items"][0]["ref"]])[0]
            self.assertEqual("backing-publication", record["publication_id"])
            self.assertEqual(2, record["revision"])
            self.assertEqual(["backing-doc@v1"], record["source_refs"])

    async def test_latest_backing_original_permission_fences_historical_publication_body(self):
        runtime, ports = self.build()
        handle = (await self.read(ports))["items"][0]["ref"]
        runtime.owners.apply_source(source("same-id", revision=2, kind="PUBLICATION_ORIGINAL",
            permission={"read": False, "model": False, "agent": False}))
        self.assertEqual("실제로 전달한 앞부분.", runtime.owners.lookup(handle)["text"])
        for roles in (("read", "model"), ("read", "agent")):
            with self.assertRaises(PolicyFenceError):
                runtime.check_sources([handle], roles=roles)

    async def test_backing_original_and_declared_source_revocation_reach_scoped_view(self):
        for revoke_id in ("same-id", "backing-doc"):
            runtime, ports = self.build(sources=[source("same-id", kind="PUBLICATION_ORIGINAL"), source("backing-doc")])
            runtime.owners.interactions[0]["source_refs"] = ["backing-doc@v1"]
            handle = (await self.read(ports))["items"][0]["ref"]
            runtime.owners.revoke(revoke_id)
            for roles in (("read", "model"), ("read", "agent")):
                with self.assertRaises(PolicyFenceError):
                    runtime.check_sources([handle], roles=roles)

    async def test_scoped_content_cache_reuses_body_and_retains_distinct_read_receipts(self):
        runtime, ports = self.build()
        first = await self.read(ports)
        second = await self.read(ports)
        self.assertFalse(first["items"][0]["cache_hit"])
        self.assertTrue(second["items"][0]["cache_hit"])
        self.assertNotEqual(first["read_id"], second["read_id"])
        self.assertEqual(first["items"][0]["ref"], second["items"][0]["ref"])
        actual = runtime.source_payload([second["items"][0]["ref"]])[0]
        self.assertEqual([first["read_id"], second["read_id"]], actual["evidence_provenance"]["read_ids"])
        runtime.owners.policy_epoch += 1
        third = await self.read(ports)
        self.assertFalse(third["items"][0]["cache_hit"])
        known = await ports.read({"tool": "interaction_retrieve", "selector": {
            "op": "KNOWN_REF", "source_ref": second["items"][0]["ref"]}, "limit": 1})
        self.assertIn(known["read_id"], known["items"][0]["metadata"]["evidence_provenance"]["read_ids"])

    async def test_same_id_unrelated_source_does_not_replace_view_current_owner(self):
        runtime, ports = self.build(sources=[source("same-id", permission={"read": False, "model": False, "agent": False})])
        item = (await self.read(ports))["items"][0]
        self.assertEqual("실제로 전달한 앞부분.", runtime.source_payload([item["ref"]])[0]["text"])


    async def test_scene_seven_plus_three_rejects_fourth_without_orphan_task_or_receipt(self):
        active = [task(f"current-{index}") for index in range(7)]
        history = [task(f"completed-{index}", status="COMPLETED") for index in range(200)]
        runtime, ports = self.build(tasks=[*active, *history])
        # A previous stopped scene may still own an unfinished Task. It is
        # retained evidence, not an execution slot in this scene.
        runtime.owners.tasks["retained-prior"] = task("retained-prior")
        self.assertEqual(7, len(runtime._active_task_ids()))
        commands = []
        for index in range(3):
            item = goal(gid=f"new-{index}", operation="CREATE_REPORT", kind="NEW_TASK", arguments={"source_refs": []})
            commands.append(await runtime._dispatch(item, ports, {}, [], None))
        await runtime.drain()
        before_tasks = copy.deepcopy(runtime.owners.tasks)
        before_receipts = sum(row["kind"] == "COMMAND_ACCEPTED" for table, row in self.evidence.rows if table == "events")
        fourth = goal(gid="extra-new", operation="CREATE_REPORT", kind="NEW_TASK", arguments={"source_refs": []})
        with self.assertRaisesRegex(RuntimeError, "capacity exhausted"):
            await runtime._dispatch(fourth, ports, {}, [], None)
        self.assertEqual(before_tasks, runtime.owners.tasks)
        self.assertEqual(3, len(runtime.effects))
        self.assertEqual(before_receipts, sum(row["kind"] == "COMMAND_ACCEPTED" for table, row in self.evidence.rows if table == "events"))
        replay = goal(gid="new-0", operation="CREATE_REPORT", kind="NEW_TASK", arguments={"source_refs": []})
        duplicate = await runtime._dispatch(replay, ports, {}, [], None)
        self.assertEqual(commands[0]["command"]["command_id"], duplicate["command_id"])
        self.assertEqual(before_tasks, runtime.owners.tasks)
        self.assertEqual(10, len(runtime._active_task_ids()))
        malicious_target = {**fourth, "target": "current-1"}
        with self.assertRaisesRegex(RuntimeError, "capacity exhausted"):
            await runtime._dispatch(malicious_target, ports, {}, [], "current-1")
        self.assertEqual(before_tasks, runtime.owners.tasks)
        self.assertEqual(3, len(runtime.effects))
        rejected = [row for table, row in self.evidence.rows if table == "events" and row["kind"] == "TASK_CAPACITY_REJECTED"]
        self.assertEqual(2, len(rejected))
        self.assertEqual(10, rejected[0]["active_count"])
        self.assertEqual("CURRENT_SCENE_EXECUTION_AND_EVENTS", rejected[0]["scope"])
        cancel = goal(gid="cancel-current", operation="CANCEL_TASK", kind="CANCEL_TASK", target="current-0", arguments={})
        await runtime._dispatch(cancel, ports, {}, [], "current-0")
        await runtime.drain()
        self.assertEqual("CANCEL_ACCEPTED_TERMINATION_UNKNOWN", runtime.owners.tasks["current-0"]["status"])
        self.assertEqual(10, len(runtime._active_task_ids()))
        with self.assertRaisesRegex(RuntimeError, "capacity exhausted"):
            await runtime._dispatch(fourth, ports, {}, [], None)

    async def test_actual_terminal_owner_status_releases_one_current_scene_slot(self):
        for terminal in ("COMPLETED", "FAILED"):
            runtime, ports = self.build(tasks=[task(f"current-{index}") for index in range(10)])
            item = runtime.owners.tasks["current-0"]
            await runtime.external_event({"event_id": "actual-terminal", "type": "RESULT", "payload": {
                "task_id": item["task_id"], "execution_id": item["execution_id"], "request_revision": 1,
                "status": terminal, "artifact": {"artifact_id": "terminal-record", "revision": 1,
                "text": "실제 terminal 사건 본문", "source_refs": []}}})
            self.assertEqual(terminal, runtime.owners.tasks["current-0"]["status"])
            self.assertEqual(9, len(runtime._active_task_ids()))
            created = goal(operation="CREATE_REPORT", kind="NEW_TASK", arguments={"source_refs": []})
            await runtime._dispatch(created, ports, {}, [], None)
            await runtime.drain()
            self.assertEqual(10, len(runtime._active_task_ids()))
            self.assertEqual(1, len(runtime.effects))
