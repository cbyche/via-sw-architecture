"""Structural runtime checks with generic owner data and no oracle/case identifiers."""
import asyncio
import copy
import time
import unittest
from unittest.mock import patch

from prototypes.candidates.dp41_45_poc.shared.runtime import LocalVIA, Ports, PolicyFenceError
from prototypes.candidates.dp41_45_poc.dp41.contract import MEANING_VERSION


class Evidence:
    def __init__(self):
        self.rows = []
        self.origin = time.monotonic()
    def seconds(self):
        return time.monotonic() - self.origin
    def write(self, table, row):
        self.rows.append((table, copy.deepcopy(row)))
        return row


class Model:
    def __init__(self, text="실제 사실 전달", wait=None):
        self.text, self.wait, self.calls = text, wait, []
    async def call(self, messages, role, **kwargs):
        self.calls.append((copy.deepcopy(messages), role, kwargs))
        if self.wait:
            await self.wait.wait()
        return {"text": self.text}


def source(source_id="doc", revision=1, text="첫 문장입니다. 둘째 문장입니다.", permission=None, kind="DOCUMENT", metadata=None):
    return {"source_id": source_id, "revision": revision, "kind": kind, "title": source_id,
            "text": text, "permission": permission or {"read": True, "model": True, "agent": True},
            "metadata": metadata or {}}


def task(task_id="work", **extra):
    return {"task_id": task_id, "execution_id": "exec-" + task_id, "request_revision": 1,
            "status": "RUNNING", "instruction": "원래 업무", "source_refs": ["doc@v1"],
            "properties": {"FORMAT": "PARAGRAPH", "PERIOD": "2026-Q3"}, **extra}


def goal(gid="g1", operation="READ_FIELD", target=None, arguments=None, kind="DIRECT_RESPONSE"):
    return {"id": gid, "kind": kind, "operation": operation, "target": target,
            "arguments": arguments if arguments is not None else {"source_refs": ["doc@v1"], "field": "VALUE"},
            "instruction": "주어진 요청"}


def meaning(goals, **extra):
    return {"version": MEANING_VERSION, "goals": goals, "relations": [], "constraints": [],
            "preservations": [], "response": None, "evidence_refs": [], "unresolved": [], **extra}


class RuntimeTests(unittest.IsolatedAsyncioTestCase):
    def build(self, initial=None, model=None, **kwargs):
        self.evidence = Evidence()
        self.runtime = LocalVIA(initial or {"sources": [source()], "tasks": [task()], "voice": {"on": False}},
                                model or Model(), self.evidence, "A", voice_profile={
                                    "output_onset_seconds": .001, "characters_per_second": 10000., "stop_seconds": .001},
                                stub_profile={"acceptance_receipt_seconds": .001, "bounded_result_seconds": .001,
                                              "summary_result_seconds": .005, "final_result_seconds": .01}, **kwargs)
        return self.runtime, Ports(self.runtime, "request", "input", "사용자 입력")

    async def asyncTearDown(self):
        for pending in list(getattr(self, "runtime", None).jobs if getattr(self, "runtime", None) else []):
            pending.cancel()
        if getattr(self, "runtime", None):
            await asyncio.gather(*list(self.runtime.jobs), return_exceptions=True)

    async def test_logical_request_read_budget_survives_new_ports(self):
        runtime, p1 = self.build()
        p2 = Ports(runtime, "request", "revision-input", "수정", 2)
        spec = {"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": "doc@v1"}}
        for index in range(16):
            await (p1 if index < 8 else p2).read(spec)
        with self.assertRaises(RuntimeError):
            await p1.read(spec)
        self.assertEqual(17, p2.read_count)

    async def test_event_identity_is_preserved_with_domain_kind_and_payload_fields(self):
        runtime, ports = self.build()
        runtime.scene_key = "actual-scene"
        await runtime.deliver("실제 결과", kind="RESULT", request_id="request")
        await ports.model([{"role": "user", "content": "실제 입력"}])
        row = runtime.emit("DOMAIN_EVENT", {"kind": "CALENDAR", "owner": "forged-owner",
            "scene_key": "forged-scene", "scene_seconds": -999})
        text_event = next(row for table, row in self.evidence.rows
                          if table == "events" and row["kind"] == "TEXT_PRESENTED")
        self.assertEqual("RESULT", text_event["payload_kind"])
        model_event = next(row for table, row in self.evidence.rows
                           if table == "events" and row["kind"] == "MODEL_RETURN_AVAILABLE")
        self.assertEqual("CINTERPRET", model_event["role"])
        self.assertEqual("DOMAIN_EVENT", row["kind"])
        self.assertEqual("CALENDAR", row["payload_kind"])
        self.assertEqual("LOCAL_VIA", row["owner"])
        self.assertEqual("actual-scene", row["scene_key"])
        self.assertGreaterEqual(row["scene_seconds"], 0)

    async def test_source_response_hold_joins_actual_source_and_user_speech_nodes(self):
        runtime, _ = self.build()
        await runtime.input_started("speaking")
        start, input_node = runtime.speech_starts["speaking"]
        work_start = runtime.timeline.now()
        await asyncio.sleep(.005)
        work_node = runtime.timeline.add_node("EXTERNAL_AGENT_WORK", "EXTERNAL_AGENT", work_start,
            runtime.timeline.now(), parents=[input_node])
        event = {"event_id": "actual-result", "type": "RESULT", "causal_parents": [work_node],
                 "payload": {"task_id": "work", "execution_id": "exec-work", "request_revision": 1,
                     "artifact": {"artifact_id": "actual-artifact", "text": "실제 결과", "source_refs": ["doc@v1"]}}}
        pending = asyncio.create_task(runtime.external_event(event))
        await asyncio.sleep(.02)
        self.assertFalse(runtime.deliveries)
        await runtime.speech_end("speaking")
        await pending
        delivery = runtime.deliveries[0]
        attributed = runtime.timeline.attribute(start, delivery["text_node_id"])
        kinds = {node["kind"] for node in attributed["nodes"]}
        self.assertIn("EXTERNAL_EVENT_AVAILABLE", kinds)
        self.assertIn("EXTERNAL_AGENT_WORK", kinds)
        self.assertIn("EXTERNAL_USER_SPEECH", kinds)
        self.assertLess(attributed["attributed_seconds"], attributed["wall_clock_seconds"] - .01)
        self.assertAlmostEqual(attributed["actual_recomputed_seconds"], attributed["wall_clock_seconds"])
        text_event = next(row for table, row in self.evidence.rows if table == "events" and row["kind"] == "TEXT_PRESENTED")
        self.assertEqual("RESULT", text_event["payload_kind"])

    async def test_voice_queue_depends_on_previous_actual_playback_without_double_count(self):
        runtime, _ = self.build({"sources": [], "tasks": [], "voice": {"on": True}})
        runtime.profile["characters_per_second"] = 200.
        start = runtime.timeline.now()
        first = asyncio.create_task(runtime.deliver("앞선 음성 결과", request_id="first", kind="RESULT"))
        await asyncio.sleep(.003)
        second = asyncio.create_task(runtime.deliver("다음 결과", request_id="second", kind="RESULT"))
        first_record, second_record = await asyncio.gather(first, second)
        second_onset = runtime.timeline.nodes[second_record["voice_onset_node_id"]]
        setup = runtime.timeline.nodes[second_onset["parents"][0]]
        self.assertIn(first_record["voice_end_node_id"], setup["parents"])
        attributed = runtime.timeline.attribute(start, second_record["voice_onset_node_id"])
        self.assertAlmostEqual(attributed["attributed_seconds"], attributed["wall_clock_seconds"])
        self.assertGreaterEqual(second_record["voice_onset_s"], first_record["voice_end_s"])

    async def test_duplicate_command_has_one_effect_and_one_task_without_orphan_or_patch(self):
        runtime, ports = self.build()
        report = goal(operation="CREATE_REPORT", kind="NEW_TASK", arguments={"source_refs": ["doc@v1"]})
        document = meaning([report], response="접수 예정")
        await runtime.perform_goal(report, document, ports)
        tasks_after_first = copy.deepcopy(runtime.owners.tasks)
        await runtime.perform_goal(report, document, ports)
        self.assertEqual(1, len(runtime.effects))
        self.assertEqual(tasks_after_first, runtime.owners.tasks)
        self.assertEqual(2, len(runtime.owners.tasks))
        altered = copy.deepcopy(report)
        altered["instruction"] = "서로 다른 명령"
        with self.assertRaisesRegex(ValueError, "conflicting payload"):
            await runtime.perform_goal(altered, meaning([altered], response="다른 요청"), ports)
        self.assertEqual(tasks_after_first, runtime.owners.tasks)
        self.assertEqual(1, len(runtime.effects))
        self.assertTrue(any(table == "events" and row["kind"] == "COMMAND_DUPLICATE_SUPPRESSED"
                            for table, row in self.evidence.rows))

    async def test_duplicate_patch_does_not_increment_task_revision_twice(self):
        runtime, ports = self.build()
        change = goal(operation="SET_PROPERTY", kind="UPDATE_TASK", target="work",
                      arguments={"property": "FORMAT", "value": "TABLE"})
        document = meaning([change], response="접수 예정")
        await runtime.perform_goal(change, document, ports)
        tasks_after_first = copy.deepcopy(runtime.owners.tasks)
        await runtime.perform_goal(change, document, ports)
        self.assertEqual(tasks_after_first, runtime.owners.tasks)
        self.assertEqual(2, runtime.owners.tasks["work"]["request_revision"])
        self.assertEqual(1, len(runtime.effects))

    async def test_drain_yields_when_gathered_jobs_are_already_done(self):
        runtime, _ = self.build()
        async def completed():
            return None
        job = asyncio.create_task(completed())
        await job
        runtime.jobs.add(job)
        asyncio.get_running_loop().call_soon(runtime.jobs.discard, job)
        await asyncio.wait_for(runtime.drain(), .1)
        self.assertFalse(runtime.jobs)

    async def test_bounded_new_task_operations_dispatch_actual_agent_and_bounded_results(self):
        for operation in ("READ_FIELD", "READ_CALENDAR", "COMPARE", "EXPLAIN"):
            with self.subTest(operation=operation):
                runtime, ports = self.build()
                runtime.stub_profile["bounded_result_seconds"] = .01
                runtime.stub_profile["final_result_seconds"] = 8
                selected = goal(operation=operation, kind="NEW_TASK")
                outcome = await runtime.perform_goal(selected, meaning([selected], response="접수 예정"), ports)
                effect = runtime.effects[0]
                self.assertEqual(operation, effect["operation"])
                self.assertEqual("doc", effect["payload"]["sources"][0]["source_id"])
                self.assertEqual("request", runtime.owners.tasks[effect["task_id"]]["request_id"])
                self.assertEqual("COMPLETED", outcome["task"]["status"])
                self.assertEqual("첫 문장입니다. 둘째 문장입니다.", outcome["artifact"]["text"])
                ready_at = runtime.timeline.nodes[runtime.goal_result_nodes[("request", "g1")]]["end_s"] - runtime.origin
                self.assertGreaterEqual(ready_at - effect["ingress_accepted_s"], .01)
                self.assertLess(ready_at - effect["ingress_accepted_s"], .3)
                await runtime.drain()

    async def test_bounded_direct_operations_keep_no_tracked_task_route(self):
        runtime, ports = self.build()
        original_tasks = copy.deepcopy(runtime.owners.tasks)
        for operation in ("READ_FIELD", "READ_CALENDAR", "COMPARE", "EXPLAIN"):
            selected = goal(gid=operation, operation=operation, kind="DIRECT_RESPONSE")
            await runtime.perform_goal(selected, meaning([selected], response="실제 직접 답변"), ports)
        self.assertFalse(runtime.effects)
        self.assertEqual(original_tasks, runtime.owners.tasks)

    async def test_direct_summary_publishes_actual_producer_duplicate_and_sends_once(self):
        runtime, ports = self.build({"sources": [source(), source("person", text="person@example.invalid", kind="CONTACT")],
                                    "tasks": [], "voice": {"on": False}})
        runtime.stub_profile["duplicate_current_summary_event"] = True
        runtime.stub_profile["duplicate_delivery_after_seconds"] = .001
        summary = goal("summary", "SUMMARIZE", kind="DIRECT_RESPONSE", arguments={"source_refs": ["doc@v1"]})
        send = goal("send", "SEND_MAIL", kind="NEW_TASK", arguments={
            "recipient_ref": "person@v1", "body": {"result_of": "summary"}})
        document = meaning([summary, send], relations=[{"op": "USE_RESULT", "from_goal": "summary", "to_goal": "send"}],
                           response="접수 예정")
        await runtime.execute(document, ports)
        await runtime.drain()
        artifact = runtime.goal_results[("request", "summary")]
        result_event = next(row for table, row in self.evidence.rows
                            if table == "events" and row["kind"] == "PRODUCER_RESULT_RECEIVED")
        duplicate = next(row for table, row in self.evidence.rows
                         if table == "events" and row["kind"] == "DUPLICATE_IGNORED")
        self.assertEqual(artifact, result_event["artifact"])
        self.assertEqual(result_event["source_event_id"], duplicate["source_event_id"])
        self.assertEqual(["SEND_MAIL"], [effect["operation"] for effect in runtime.effects])
        self.assertEqual(artifact["artifact_id"], runtime.effects[0]["payload"]["body_artifact"]["artifact_id"])

    async def test_voice_core_character_onset_has_its_actual_emission_node(self):
        runtime, _ = self.build({"sources": [], "tasks": [], "voice": {"on": True}})
        runtime.profile["characters_per_second"] = 100.
        text = "안내: 실제 접수"
        record = await runtime.deliver(text, request_id="request")
        events = record["voice_char_events"]
        self.assertEqual(list(range(len(text))), [event["char_index"] for event in events])
        self.assertEqual(record["voice_onset_node_id"], events[0]["causal_node_id"])
        core_index = text.index("실제")
        core = events[core_index]
        self.assertGreater(core["onset_s"], record["voice_onset_s"])
        self.assertGreaterEqual(core["onset_s"] - record["voice_onset_s"], core_index / 100.)
        core_node = runtime.timeline.nodes[core["causal_node_id"]]
        self.assertEqual("VOICE_CHARACTER", core_node["kind"])
        self.assertEqual([record["voice_onset_node_id"]], core_node["parents"])
        self.assertAlmostEqual(core["onset_s"], core_node["end_s"] - runtime.origin)
        attributed = runtime.timeline.attribute(record["voice_onset_s"] + runtime.origin, core["causal_node_id"])
        self.assertAlmostEqual(core["onset_s"] - record["voice_onset_s"], attributed["wall_clock_seconds"])
        self.assertEqual(text, record["voice_text"])

    async def test_interruption_retains_only_characters_actually_emitted_by_backend(self):
        runtime, _ = self.build({"sources": [], "tasks": [], "voice": {"on": True}})
        runtime.profile["characters_per_second"] = 1000.
        pending = asyncio.create_task(runtime.deliver("긴 실제 결과 " * 100, request_id="request"))
        await asyncio.sleep(.008)
        pending.cancel()
        with self.assertRaises(asyncio.CancelledError):
            await pending
        record = runtime.deliveries[0]
        self.assertEqual([0], [event["char_index"] for event in record["voice_char_events"]])
        self.assertEqual(record["text"][:1], record["voice_text"])
        self.assertEqual("LOCAL_CANCELLED", record["discard_reason"])

    async def test_voice_off_keeps_text_endpoint_without_voice_characters(self):
        runtime, _ = self.build({"sources": [], "tasks": [], "voice": {"on": False}})
        record = await runtime.deliver("실제 결과", request_id="request")
        self.assertEqual([], record["voice_char_events"])
        self.assertEqual(record["text_node_id"], record["causal_node_id"])
        self.assertIsNone(record["voice_onset_s"])

    async def test_candidate_response_is_not_repaired_when_goals_exist(self):
        runtime, ports = self.build()
        await runtime.execute(meaning([goal()], response="완료하지 않았는데 완료했다고 주장"), ports)
        await runtime.drain()
        self.assertEqual("완료하지 않았는데 완료했다고 주장", runtime.deliveries[0]["text"])
        self.assertFalse(runtime.model_client.calls)

    async def test_model_prohibition_cannot_be_bypassed_by_execution_source_read(self):
        runtime, ports = self.build({"sources": [source(permission={"read": True, "model": False, "agent": True})],
                                    "tasks": [], "voice": {"on": False}})
        with self.assertRaises(PolicyFenceError):
            await runtime.perform_goal(goal(), meaning([goal()]), ports)
        self.assertFalse(runtime.model_client.calls)

    async def test_answer_cannot_modify_another_tasks_question(self):
        q = {"question_id": "q", "revision": 2, "task_id": "work", "execution_id": "exec-work",
             "status": "VALID", "text": "기간을 상반기로 할까요?", "scope": {"property": "PERIOD", "proposed_value": "H1"},
             "delivered_receipt": True, "text_presented": True, "voice_presented": True}
        runtime, ports = self.build({"sources": [source()], "tasks": [task(questions=[q]), task("other")], "voice": {"on": False}})
        g = goal(operation="ANSWER_QUESTION", kind="ANSWER_QUESTION", target="other",
                 arguments={"question_ref": "q", "question_revision": 2, "answer": "네", "properties": {"PERIOD": "H1"}})
        with self.assertRaises(ValueError):
            await runtime.perform_goal(g, meaning([g]), ports)
        self.assertEqual("2026-Q3", runtime.owners.tasks["other"]["properties"]["PERIOD"])
        self.assertFalse(runtime.effects)

    async def test_question_withdrawn_during_input_hold_has_no_presentation_receipt(self):
        runtime, _ = self.build({"sources": [source()], "tasks": [task()], "voice": {"on": True}})
        q = {"question_id": "q", "revision": 1, "task_id": "work", "execution_id": "exec-work", "text": "질문입니다", "status": "VALID"}
        runtime.owners.questions["q"] = q
        await runtime.input_started("input")
        pending = asyncio.create_task(runtime.deliver(q["text"], facts={"question": q}, kind="QUESTION"))
        await asyncio.sleep(.001)
        self.assertFalse(any(row[1].get("kind") == "TEXT_PRESENTED" for row in self.evidence.rows))
        await runtime.external_event({"event_id": "withdraw", "type": "QUESTION_WITHDRAWN", "payload": {"question_id": "q", "revision": 1}})
        await runtime.speech_end("input")
        result = await pending
        self.assertIsNone(result["text_presented_s"])
        self.assertIsNone(result["voice_onset_s"])
        self.assertFalse(result["delivered_receipt"])

    async def test_current_producer_send_does_not_wait_for_summary_voice_completion(self):
        runtime, ports = self.build({"sources": [source(), source("person", text="person@example.invalid", kind="CONTACT")],
                                    "tasks": [], "voice": {"on": True}}, model=Model("요약 실제 사실 " * 25))
        runtime.profile.update({"output_onset_seconds": .005, "characters_per_second": 1000.})
        summarize = goal("summary", "SUMMARIZE", arguments={"source_refs": ["doc@v1"]})
        send = goal("send", "SEND_MAIL", arguments={"recipient_ref": "person@v1", "body": {"result_of": "summary"}}, kind="NEW_TASK")
        await runtime.execute(meaning([summarize, send], relations=[{"op": "USE_RESULT", "from_goal": "summary", "to_goal": "send"}]), ports)
        send_effect = next(e for e in runtime.effects if e["operation"] == "SEND_MAIL")
        artifact = runtime.goal_results[("request", "summary")]
        self.assertEqual(artifact["artifact_id"], send_effect["payload"]["body_artifact"]["artifact_id"])
        await runtime.drain()
        summary_voice = next(d for d in runtime.deliveries if d.get("facts", {}).get("summary"))
        self.assertLess(send_effect["send_s"], summary_voice["voice_end_s"])
        self.assertNotEqual("", artifact["text"])

    async def test_calendar_guard_uses_actual_current_owner_after_source_change(self):
        initial_source = source("calendar", kind="CALENDAR", text="10시 FREE", metadata={"date": "2026-10-12", "interval_state": "FREE", "events": []})
        runtime, ports = self.build({"sources": [initial_source], "tasks": [], "voice": {"on": False}})
        await runtime.external_event({"event_id": "change", "type": "SOURCE_CHANGE", "payload": {"source":
            source("calendar", 2, "10시 BUSY", kind="CALENDAR", metadata={"date": "2026-10-12", "interval_state": "BUSY", "events": []})}})
        g = goal(operation="CALENDAR_CONDITIONAL", arguments={"source_refs": ["calendar@v1"], "date": "2026-10-12", "start": "10:00", "end": "11:00", "if_free": "CREATE_EVENT", "otherwise": "AVAILABLE_ONLY"})
        facts = await runtime.perform_goal(g, meaning([g]), ports)
        self.assertEqual("BUSY", facts["condition_state"])
        self.assertEqual(2, facts["sources"][0]["revision"])
        self.assertFalse(runtime.effects)
        await runtime.drain()

    async def test_revocation_fences_late_model_and_clears_local_derived_content(self):
        wait = asyncio.Event()
        runtime, ports = self.build(model=Model(wait=wait))
        await ports.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": "doc@v1"}})
        runtime.artifacts["artifact"] = {"artifact_id": "artifact", "text": "파생 내용", "source_refs": ["doc@v1"]}
        messages = [{"role": "system", "content": "규칙"}, {"role": "user", "content": "원문 내용 doc@v1"}]
        pending = asyncio.create_task(ports.model(messages))
        await asyncio.sleep(.001)
        await runtime.revoke_source("doc")
        wait.set()
        with self.assertRaises(PolicyFenceError):
            await pending
        self.assertEqual("", runtime.artifacts["artifact"]["text"])
        self.assertIn("REVOKED", messages[1]["content"])
        self.assertTrue(any(row[1].get("kind") == "MODEL_DISCARDED" for row in self.evidence.rows))

    async def test_preparation_keeps_original_audible_receipt_and_task_revisions_distinct(self):
        runtime, _ = self.build({"sources": [source(), source("unused-slot", text="", kind="SOURCE_SLOT", metadata={"unpopulated": True})],
                                "tasks": [task()], "voice": {"on": False}})
        await runtime.external_event({"event_id": "original", "type": "PUBLICATION_ORIGINAL", "payload": {
            "source": source("publication", text="들린 내용. 미청취 내용."),
            "publication": {"publication_id": "pub", "text": "들린 내용. 미청취 내용.", "delivered_voice_text": ""}}})
        await runtime.external_event({"event_id": "receipt", "type": "DELIVERY_RECEIPT", "payload": {
            "publication_id": "pub", "delivered_voice_text": "들린 내용.", "interrupted": True}})
        await runtime.external_event({"event_id": "task-source", "type": "TASK_SOURCE_RECORD", "payload": {
            "task": task("done", request_revision=2, status="COMPLETED"), "source": source("doc", 2, "새 원문")}})
        publication = next(p for p in runtime.owners.interactions if p["publication_id"] == "pub")
        self.assertIn("미청취", runtime.owners.lookup("publication@v1")["text"])
        self.assertNotIn("미청취", publication["delivered_voice_text"])
        self.assertEqual(2, runtime.owners.tasks["done"]["request_revision"])
        self.assertIsNotNone(runtime.owners.lookup("doc@v1"))
        self.assertIsNotNone(runtime.owners.lookup("doc@v2"))

    async def test_superseded_interpretation_is_discarded_before_command_execution(self):
        runtime, _ = self.build()
        adopted = meaning([goal(operation="CREATE_REPORT", arguments={"source_refs": ["doc@v1"]}, kind="NEW_TASK")])
        async def fake_resolve(inp, ports):
            runtime.current_input_generation += 1
            return adopted
        with patch("prototypes.candidates.dp41_45_poc.dp41.a.resolve", fake_resolve):
            result = await runtime.resolve("보고서 작성", generation=0)
        self.assertEqual("DISCARDED", result["status"])
        self.assertFalse(runtime.effects)

    async def test_general_finite_when_false_does_not_dispatch_dependent_goal(self):
        runtime, ports = self.build()
        query = goal("status", "QUERY_STATUS", target="work", arguments={})
        report = goal("new", "CREATE_REPORT", arguments={"source_refs": ["doc@v1"]}, kind="NEW_TASK")
        relation = {"op": "WHEN", "from_goal": "status", "to_goal": "new",
                    "predicate": {"reference_id": "work", "field": "STATE", "comparison": "EQ", "value": "FAILED"}}
        await runtime.execute(meaning([query, report], relations=[relation]), ports)
        self.assertFalse(runtime.effects)
        self.assertTrue(any(row[1].get("kind") == "CONDITION_FALSE" for row in self.evidence.rows))
        await runtime.drain()

    async def test_general_finite_when_unknown_holds_then_observes_actual_owner_event(self):
        runtime, ports = self.build({"sources": [source()], "tasks": [task(status="UNKNOWN")], "voice": {"on": False}})
        query = goal("status", "QUERY_STATUS", target="work", arguments={})
        report = goal("new", "CREATE_REPORT", arguments={"source_refs": ["doc@v1"]}, kind="NEW_TASK")
        relation = {"op": "WHEN", "from_goal": "status", "to_goal": "new",
                    "predicate": {"reference_id": "work", "field": "STATE", "comparison": "EQ", "value": "FAILED"}}
        pending = asyncio.create_task(runtime.execute(meaning([query, report], relations=[relation]), ports))
        await asyncio.sleep(.005)
        self.assertFalse(runtime.effects)
        await runtime.external_event({"event_id": "owner-state", "type": "TASK_SOURCE_RECORD", "payload": {
            "task": task(status="FAILED"), "source": source()}})
        await pending
        await runtime.drain()
        self.assertEqual(["CREATE_REPORT"], [e["operation"] for e in runtime.effects])

    async def test_post_cancel_original_receipt_does_not_claim_termination_or_restore_running(self):
        runtime, ports = self.build()
        runtime.stub_profile["post_cancel_receipts"] = [
            {"type": "ACCEPTED", "after_cancel_receipt_seconds": .001, "task_id": "work", "execution_id": "exec-work", "request_revision": 1},
            {"type": "DUPLICATE_CANCEL_ACCEPTED", "after_cancel_receipt_seconds": .001, "same_event_id": True}]
        cancel = goal(operation="CANCEL_TASK", target="work", arguments={}, kind="CANCEL_TASK")
        await runtime.execute(meaning([cancel]), ports)
        await runtime.drain()
        self.assertEqual("CANCEL_ACCEPTED_TERMINATION_UNKNOWN", runtime.owners.tasks["work"]["status"])
        self.assertTrue(any(row[1].get("kind") == "DUPLICATE_IGNORED" for row in self.evidence.rows))
        self.assertFalse(any(row[1].get("kind") == "TASK_COMPLETED" for row in self.evidence.rows))

    async def test_send_body_reference_cannot_bypass_actual_permission_gate(self):
        runtime, ports = self.build({"sources": [source(permission={"read": False, "model": False, "agent": False}),
            source("person", text="person@example.invalid", kind="CONTACT")], "tasks": [], "voice": {"on": False}})
        send = goal(operation="SEND_MAIL", arguments={"recipient_ref": "person@v1", "body": {"source_ref": "doc@v1"}}, kind="NEW_TASK")
        with self.assertRaises(PolicyFenceError):
            await runtime.perform_goal(send, meaning([send]), ports)
        self.assertFalse(runtime.effects)

    async def test_actual_scoped_information_consent_materializes_contact_without_send_approval(self):
        q = {"question_id": "q", "revision": 2, "task_id": "work", "execution_id": "exec-work",
             "status": "VALID", "text": "연락처를 조회해 업무에 제공할까요?",
             "scope": {"source_id": "person", "uses": ["READ_CONTACT", "PROVIDE_TO_TASK"], "task_id": "work"},
             "delivered_receipt": True, "text_presented": True, "voice_presented": True}
        runtime, ports = self.build({"sources": [source(), source("person", text="", kind="CONTACT",
            permission={"read": False, "model": False, "agent": False})],
            "tasks": [task(questions=[q])], "voice": {"on": False}, "policy": {"send_allowed": False}})
        runtime.pending_consent_source = {"source": source("person", text="person@example.invalid", kind="CONTACT"),
            "only_after_actual_consent_scope": {"question_id": "q", "revision": 2, "task_id": "work",
                "uses": ["READ_CONTACT", "PROVIDE_TO_TASK"]}, "send_permission": "DENIED"}
        answer = goal(operation="ANSWER_QUESTION", target="work", kind="ANSWER_QUESTION", arguments={
            "question_ref": "q", "question_revision": 2, "answer": "네", "consent": {"read": True, "share": True, "send": False}})
        await runtime.perform_goal(answer, meaning([answer]), ports)
        await runtime.drain()
        self.assertEqual("person@example.invalid", runtime.owners.lookup("person@v1")["text"])
        payload = runtime.effects[0]["payload"]
        self.assertEqual("person@example.invalid", payload["provided_information"][0]["content"])
        self.assertFalse(runtime.policy["send_allowed"])
        self.assertEqual(["ANSWER_QUESTION"], [e["operation"] for e in runtime.effects])

    async def test_cancellation_preserves_actual_onset_prefix_and_incomplete_receipt(self):
        runtime, _ = self.build({"sources": [], "tasks": [], "voice": {"on": True}})
        text = "재생 중인 실제 문장 " * 100
        pending = asyncio.create_task(runtime.deliver(text, request_id="request"))
        await asyncio.sleep(.012)
        pending.cancel()
        with self.assertRaises(asyncio.CancelledError):
            await pending
        record = runtime.deliveries[0]
        self.assertIsNotNone(record["text_presented_s"])
        self.assertIsNotNone(record["voice_onset_s"])
        self.assertGreater(len(record["voice_text"]), 0)
        self.assertLess(len(record["voice_text"]), len(text))
        self.assertEqual("LOCAL_CANCELLED", record["discard_reason"])
        self.assertFalse(record["delivered_receipt"])
        self.assertTrue(any(table == "deliveries" for table, row in self.evidence.rows))

if __name__ == "__main__":
    unittest.main()
