"""Structural tests; no network, models, timing measurements or evaluator oracle."""
import asyncio
import copy
import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from dp41 import resolve_a, resolve_b
from dp41.contract import (ContractError, FRAME_VERSION, MEANING_VERSION,
                           reference_spec, validate_document, validate_read)
from dp41.prompts import A_SYSTEM, B_SYSTEM, SELECT_SYSTEM
from shared.owners import OwnerRepository


def meaning(goals=None):
    return {"version": MEANING_VERSION, "goals": goals or [], "relations": [], "constraints": [],
            "preservations": [], "response": None, "evidence_refs": [], "unresolved": []}


def frame():
    value = meaning([{"id": "g-read", "kind": "DIRECT_RESPONSE", "operation": "READ_FIELD", "target_ref": None,
                      "arguments": {"source_refs": ["r-document"], "field": "DEADLINE"}, "instruction": "read deadline"}])
    value["version"] = FRAME_VERSION
    value["references"] = [{"id": "r-document", "selector": {"op": "EXPLICIT_SOURCE", "source_kind": "DOCUMENT", "name": "DocA"}}]
    return value


class Ports:
    def __init__(self, outputs, reads=None, answer=None):
        self.outputs = copy.deepcopy(outputs)
        self.reads = copy.deepcopy(reads or [{"status": "OK", "read_id": "read-a",
                       "items": [{"ref": "source-a", "kind": "DOCUMENT", "revision": 1, "content": "deadline source"}]}])
        self.answer = answer or {"no_answer": True}
        self.messages, self.events, self.specs, self.questions = [], [], [], []

    async def model(self, messages, role="CINTERPRET"):
        self.messages.append(copy.deepcopy(messages))
        return self.outputs.pop(0)

    async def read(self, spec):
        self.specs.append(copy.deepcopy(spec))
        return self.reads.pop(0)

    async def clarify(self, question, scope=None):
        self.questions.append((question, scope))
        return self.answer

    def emit(self, kind, payload):
        self.events.append((kind, copy.deepcopy(payload)))


class OwnerPorts(Ports):
    """Scripted models with the real bounded owner read and receipt shape."""
    def __init__(self, outputs, initial):
        super().__init__(outputs)
        self.repository = OwnerRepository(initial, self.emit)
        self.receipts = []

    async def read(self, spec):
        self.specs.append(copy.deepcopy(spec))
        receipt = await self.repository.read(spec)
        self.receipts.append(copy.deepcopy(receipt))
        return receipt


class DP41Tests(unittest.IsolatedAsyncioTestCase):
    def test_reference_query_precedence_preserves_explicit_empty_query(self):
        for reference, expected in (
            ({"query": "explicit", "selector": {"op": "RECENT_TASK", "query": "nested"}, "mention": "mention"}, "explicit"),
            ({"query": "", "selector": {"op": "RECENT_TASK", "query": "nested"}, "mention": "mention"}, ""),
            ({"selector": {"op": "RECENT_TASK", "query": "nested"}, "mention": "mention"}, "nested"),
            ({"selector": {"op": "RECENT_TASK", "query": ""}, "mention": "mention"}, ""),
            ({"selector": {"op": "RECENT_TASK"}, "mention": "mention"}, "mention"),
            ({"selector": {"op": "RECENT_TASK"}}, ""),
        ):
            with self.subTest(reference=reference):
                spec = reference_spec(reference)
                self.assertEqual(expected, spec["query"])
                self.assertEqual(reference["selector"], spec["selector"])
                validate_read(spec)

    def test_selector_query_type_is_checked_even_with_explicit_top_query(self):
        for invalid in (None, [], {}, 1, False):
            with self.subTest(query=invalid):
                selector = {"op": "RECENT_TASK", "query": invalid}
                with self.assertRaisesRegex(ContractError, "selector.query must be a string"):
                    reference_spec({"query": "explicit", "selector": selector})
                with self.assertRaisesRegex(ContractError, "selector.query must be a string"):
                    validate_read({"tool": "task_retrieve", "query": "explicit", "selector": selector})
                value = frame()
                value["references"][0].update(query="explicit", selector=selector)
                with self.assertRaisesRegex(ContractError, "selector.query must be a string"):
                    validate_document(value, frame=True)

    async def test_b_nested_query_reaches_real_owner_instead_of_mention(self):
        for op in ("RECENT_TASK", "TASK_RESULT", "RECENT_INTERACTION", "PAST_DELIVERED"):
            with self.subTest(op=op):
                initial = {"tasks": [], "sources": [], "interactions": []}
                if op in ("RECENT_TASK", "TASK_RESULT"):
                    initial["tasks"] = [{"task_id": name, "request_revision": 1, "status": "COMPLETED",
                        "instruction": label, "result": {"artifact_id": name + "-result", "revision": 1,
                        "kind": "ARTIFACT", "title": label, "text": label}}
                        for name, label in (("older-task", "일정 기록"), ("requested-task", "분기보고 근거"))]
                else:
                    initial["interactions"] = [{"publication_id": name, "revision": 1,
                        "text": label, "delivered_voice_text": label, "voice_on": True,
                        "text_presented": True, "delivered_receipt": True}
                        for name, label in (("older-publication", "일정 기록"), ("requested-publication", "분기보고 근거"))]
                value = frame()
                value["goals"][0].update(operation="EXPLAIN", arguments={"source_refs": ["r-document"]})
                value["references"] = [{"id": "r-document", "mention": "방금 그것",
                    "selector": {"op": op, "query": "분기보고"}, "limit": 1}]
                ports = OwnerPorts([value], initial)
                result = await resolve_b({"transcript": "관련 자료를 설명해 줘."}, ports)
                self.assertEqual("분기보고", ports.specs[0]["query"])
                self.assertEqual("분기보고", ports.specs[0]["selector"]["query"])
                selected = ports.receipts[0]["items"][0]
                self.assertEqual([selected["ref"]], result["goals"][0]["arguments"]["source_refs"])
                record = ports.repository.lookup(selected["ref"])
                self.assertIn("분기보고", record.get("instruction", record.get("text", "")))
                self.assertEqual(1, len(ports.messages))
                self.assertEqual([], ports.questions)

    async def test_b_clarification_uses_actual_owner_metadata_and_content_labels(self):
        initial = {"sources": [{"source_id": "internal-source", "revision": 1, "kind": "DOCUMENT",
            "title": "사용자가 읽은 문서 제목", "text": "실제 문서 본문"}],
            "tasks": [{"task_id": "internal-task", "request_revision": 1, "status": "RUNNING",
                "instruction": "분기 보고서 작성 업무"}],
            "interactions": [{"publication_id": "internal-publication", "revision": 1,
                "text": "실제 표시한 전체 내용", "delivered_voice_text": "실제로 들은 기준 설명",
                "voice_on": True, "text_presented": True, "delivered_receipt": True}]}
        value = frame()
        value["goals"][0].update(operation="EXPLAIN", arguments={"source_refs": ["r-source"]})
        value["references"] = [
            {"id": "r-source", "selector": {"op": "KNOWN_REF", "source_ref": "internal-source@v1"}},
            {"id": "r-task", "selector": {"op": "RECENT_TASK", "query": "보고서"}},
            {"id": "r-history", "selector": {"op": "PAST_DELIVERED", "publication_id": "internal-publication"}}]
        value["unresolved"] = [{"field_ref": "r-history", "reason": "MISSING_CHANGE_VALUE"}]
        ports = OwnerPorts([value], initial)
        result = await resolve_b({"transcript": "그것을 변경해 줘."}, ports)
        self.assertEqual([], result["goals"])
        self.assertEqual(1, len(ports.questions))
        question, scope = ports.questions[0]
        for label in ("사용자가 읽은 문서 제목", "분기 보고서 작성 업무", "실제로 들은 기준 설명"):
            self.assertIn(label, question)
        for internal in ("internal-source", "internal-task", "internal-publication", "via-interaction:"):
            self.assertNotIn(internal, question)
        for field, receipt in zip(("r-source", "r-task", "r-history"), ports.receipts):
            self.assertEqual([receipt["items"][0]["ref"]], scope["candidate_refs"][field])
        self.assertEqual(["r-history"], scope["field_refs"])
        self.assertEqual(1, len(ports.messages))

    async def test_b_clarification_without_title_is_bounded_and_keeps_candidate_numbers(self):
        initial = {"sources": [{"source_id": "opaque-source", "revision": 1, "kind": "DOCUMENT",
            "text": "실제 조회한 긴 내용 " * 50}],
            "tasks": [{"task_id": "opaque-task", "request_revision": 1, "status": "RUNNING", "instruction": ""}],
            "interactions": []}
        value = frame()
        value["references"] = [
            {"id": "r-document", "selector": {"op": "KNOWN_REF", "source_ref": "opaque-source@v1"}},
            {"id": "r-task", "selector": {"op": "RECENT_TASK", "query": ""}}]
        value["unresolved"] = [{"reason": "MISSING_CHANGE_VALUE"}]
        ports = OwnerPorts([value], initial)
        await resolve_b({"transcript": "변경해 줘."}, ports)
        question, scope = ports.questions[0]
        self.assertIn("후보 1: 실제 조회한 긴 내용", question)
        self.assertIn("…; 후보 2", question)
        self.assertLess(len(question), 180)
        self.assertNotIn("opaque-source", question)
        self.assertNotIn("opaque-task", question)
        self.assertEqual(["opaque-task"], scope["candidate_refs"]["r-task"])

    async def test_a_consumes_read_results_in_next_model_context(self):
        read = {"tool": "context_retrieve", "selector": {"op": "EXPLICIT_SOURCE", "source_kind": "DOCUMENT", "name": "DocA"}}
        proposed = meaning([{"id": "g", "kind": "DIRECT_RESPONSE", "operation": "READ_FIELD", "target": None,
                             "arguments": {"source_refs": ["source-a"], "field": "DEADLINE"}, "instruction": "read"}])
        ports = Ports([{"action": "READ", "reads": [read]}, {"action": "PROPOSE", "meaning": proposed}])
        result = await resolve_a({"transcript": "read deadline", "input_id": "input-a", "case_id": "MUST_NOT_BE_MODEL_INPUT"}, ports)
        self.assertEqual(result, proposed)
        self.assertEqual(len(ports.messages), 2)
        serialized = json.dumps(ports.messages[1])
        self.assertIn("deadline source", serialized)
        self.assertNotIn("MUST_NOT_BE_MODEL_INPUT", serialized)

    async def test_b_code_binds_frame_without_final_whole_meaning_model_call(self):
        ports = Ports([frame()])
        result = await resolve_b({"transcript": "read deadline", "input_id": "input-a"}, ports)
        self.assertEqual(result["goals"][0]["arguments"]["source_refs"], ["source-a"])
        self.assertEqual(len(ports.messages), 1)
        self.assertIn(("dp41_field_state", {"field_id": "r-document", "state": "BOUND", "binding": "source-a", "read_id": "read-a"}), ports.events)

    async def test_b_partial_cannot_replace_whole_graph(self):
        read = {"status": "OK", "read_id": "read-a", "items": [{"ref": "a"}, {"ref": "b"}]}
        ports = Ports([frame(), {"field_ref": "r-document", "candidate_ref": "b", "uncertain": False,
                                "evidence_refs": ["read-a"], "goals": [{"id": "injected"}]}], [read])
        result = await resolve_b({"transcript": "unspecified source"}, ports)
        self.assertTrue(result["unresolved"])
        self.assertEqual(len(ports.questions), 1)
        self.assertEqual(result["goals"], [])

    async def test_b_partial_selection_request_contains_explicit_json_instruction(self):
        read = {"status": "OK", "read_id": "read-a", "items": [{"ref": "a"}, {"ref": "b"}]}
        selection = {"field_ref": "r-document", "candidate_ref": "b", "uncertain": False,
                     "evidence_refs": ["read-a"]}
        ports = Ports([frame(), selection], [read])
        result = await resolve_b({"transcript": "use the second source"}, ports)
        self.assertEqual(result["goals"][0]["arguments"]["source_refs"], ["b"])
        self.assertEqual(len(ports.messages), 2)
        self.assertIn("one JSON object", ports.messages[1][0]["content"])
        self.assertEqual(ports.questions, [])

    async def test_b_partial_nested_type_error_uses_existing_unresolved_path(self):
        read = {"status": "OK", "read_id": "read-a", "items": [{"ref": "a"}, {"ref": "b"}]}
        invalid = [{"candidate_ref": []}, {"candidate_ref": {}}, {"evidence_refs": None},
                   {"evidence_refs": [{"read_id": "read-a"}]}]
        for changed in invalid:
            with self.subTest(changed=changed):
                selection = {"field_ref": "r-document", "candidate_ref": "b", "uncertain": False,
                             "evidence_refs": ["read-a"], **changed}
                ports = Ports([frame(), selection], [read])
                result = await resolve_b({"transcript": "unspecified source"}, ports)
                self.assertEqual(result["goals"], [])
                self.assertEqual(result["unresolved"], [{"reason": "NO_CLARIFICATION_ANSWER"}])
                self.assertEqual(len(ports.questions), 1)
                returned = [payload for kind, payload in ports.events if kind == "dp41_b_partial_returned"]
                self.assertEqual(returned[0]["output"], selection)
                self.assertFalse(any(kind == "dp41_resolution_failed" for kind, _ in ports.events))

    async def test_b_unresolved_prose_is_rejected_before_reads_and_retried_as_model_output(self):
        malformed = frame()
        malformed["unresolved"] = ["the request is unclear"]
        ports = Ports([malformed, frame()])
        result = await resolve_b({"transcript": "read deadline"}, ports)
        self.assertEqual(result["goals"][0]["arguments"]["source_refs"], ["source-a"])
        self.assertEqual(result["unresolved"], [])
        self.assertEqual(len(ports.specs), 1)
        self.assertEqual(len(ports.messages), 2)
        self.assertEqual(ports.questions, [])
        rejected = [payload for kind, payload in ports.events if kind == "dp41_output_rejected"]
        self.assertEqual(rejected[0]["output"]["unresolved"], ["the request is unclear"])
        self.assertIn("unresolved entries must be objects", rejected[0]["reason"])
        self.assertIn("validation_error", ports.messages[1][-1]["content"])

    async def test_a_unresolved_nested_type_is_rejected_for_model_self_repair(self):
        malformed = meaning()
        malformed["unresolved"] = [{"reason": ["missing target"]}]
        corrected = meaning()
        corrected["unresolved"] = [{"reason": "MISSING_TARGET", "field_ref": "target", "source_ref": None}]
        ports = Ports([{"action": "PROPOSE", "meaning": malformed},
                       {"action": "PROPOSE", "meaning": corrected}])
        result = await resolve_a({"transcript": "unclear request"}, ports)
        self.assertEqual(result, corrected)
        self.assertEqual(ports.specs, [])
        self.assertEqual(ports.questions, [])
        self.assertEqual(len(ports.messages), 2)
        rejected = [payload for kind, payload in ports.events if kind == "dp41_output_rejected"]
        self.assertEqual(rejected[0]["output"]["meaning"]["unresolved"], malformed["unresolved"])

    def test_all_interpretation_roles_explicitly_request_json(self):
        for prompt in (A_SYSTEM, B_SYSTEM, SELECT_SYSTEM):
            self.assertIn("JSON object", prompt)

    def test_unresolved_is_a_closed_object_schema_without_prose_conversion(self):
        invalid = ["free prose", None, {}, {"reason": ""}, {"reason": []},
                   {"reason": "MISSING", "field_ref": []}, {"reason": "MISSING", "source_ref": {}},
                   {"reason": "MISSING", "instruction": "select a target"}]
        for item in invalid:
            for is_frame in (False, True):
                with self.subTest(item=item, frame=is_frame):
                    value = frame() if is_frame else meaning()
                    value["unresolved"] = [item]
                    with self.assertRaises(ContractError):
                        validate_document(value, frame=is_frame)
        value = frame()
        value["unresolved"] = [{"reason": "DENIED", "field_ref": "r-document", "source_ref": None}]
        self.assertIs(validate_document(value, frame=True), value)

    def test_malformed_nested_condition_constraint_preservation_is_contract_error(self):
        other = {"id": "other", "kind": "DIRECT_RESPONSE", "operation": "EXPLAIN", "target_ref": None,
                 "arguments": {"source_refs": ["r-document"]}, "instruction": "explain"}
        for predicate in ("STATE FAILED", [], {"field": [], "comparison": "EQ"},
                          {"field": "STATE", "comparison": {}}):
            with self.subTest(predicate=predicate):
                value = frame()
                value["goals"].append(other)
                value["relations"] = [{"op": "WHEN", "from_goal": "g-read", "to_goal": "other", "predicate": predicate}]
                with self.assertRaises(ContractError):
                    validate_document(value, frame=True)
        for key in ("constraints", "preservations"):
            for item in ("prose", None, []):
                for is_frame in (False, True):
                    with self.subTest(key=key, item=item, frame=is_frame):
                        value = frame() if is_frame else meaning()
                        value[key] = [item]
                        with self.assertRaises(ContractError):
                            validate_document(value, frame=is_frame)

    def test_malformed_read_and_frame_reference_types_raise_contract_error(self):
        for malformed in (None, [], "text", {"op": []}, {"op": {}}, {"op": None}):
            with self.subTest(selector=malformed):
                with self.assertRaises(ContractError):
                    reference_spec({"selector": malformed})
                with self.assertRaises(ContractError):
                    validate_read({"tool": "context_retrieve", "selector": malformed})
                value = frame()
                value["references"][0]["selector"] = malformed
                with self.assertRaises(ContractError):
                    validate_document(value, frame=True)
        for invalid in ([], {}):
            with self.subTest(invalid=invalid):
                with self.assertRaises(ContractError):
                    validate_read({"tool": invalid, "selector": {"op": "KNOWN_REF", "source_ref": "a"}})
                for field in ("target_ref", "recipient_ref", "question_ref", "source_refs", "cardinality", "preservation", "predicate"):
                    value = frame()
                    if field == "cardinality":
                        value["references"][0][field] = invalid
                    elif field == "target_ref":
                        value["goals"][0][field] = invalid
                    elif field == "preservation":
                        value["preservations"] = [{"reference_id": invalid, "mode": "NO_NEW_COMMAND"}]
                    elif field == "predicate":
                        value["goals"].append({"id": "other", "kind": "DIRECT_RESPONSE", "operation": "EXPLAIN",
                            "target_ref": None, "arguments": {}, "instruction": "explain"})
                        value["relations"] = [{"op": "WHEN", "from_goal": "g-read", "to_goal": "other",
                            "predicate": {"reference_id": invalid, "field": "STATE", "comparison": "EQ", "value": "RUNNING"}}]
                    elif field == "source_refs":
                        value["goals"][0]["arguments"][field] = [{"ref": invalid}]
                    else:
                        value["goals"][0]["arguments"][field] = invalid
                    with self.subTest(field=field):
                        with self.assertRaises(ContractError):
                            validate_document(value, frame=True)
                for operation, field in (("SET_PROPERTY", "property"), ("READ_FIELD", "field")):
                    value = frame()
                    value["goals"][0]["operation"] = operation
                    value["goals"][0]["arguments"] = {field: invalid, "value": "unchanged"}
                    with self.assertRaises(ContractError):
                        validate_document(value, frame=True)

    async def test_b_object_source_reference_and_symbolic_result_remain_supported(self):
        value = frame()
        value["goals"][0]["arguments"]["source_refs"] = [{"ref": "r-document"}]
        value["goals"].append({"id": "g-send", "kind": "NEW_TASK", "operation": "SEND_MAIL",
            "target_ref": None, "arguments": {"recipient": "person@example.invalid",
            "body": {"kind": "GOAL_RESULT", "producer_goal_id": "g-read"}}, "instruction": "send result"})
        ports = Ports([value])
        result = await resolve_b({"request_id": "request", "revision": 2}, ports)
        self.assertEqual(result["goals"][0]["arguments"]["source_refs"], ["source-a"])
        self.assertEqual(result["goals"][1]["arguments"]["body"], {
            "kind": "GOAL_RESULT", "producer_goal_id": "g-read", "request_id": "request", "producer_revision": 2})
        self.assertEqual(len(ports.messages), 1)

    async def test_b_current_producer_stays_symbolic_before_actual_result(self):
        value = frame()
        value["goals"][0].update(id="g-summary", operation="SUMMARIZE", kind="NEW_TASK")
        value["goals"][0]["arguments"] = {"source_refs": ["r-document"]}
        value["goals"].append({"id": "g-send", "kind": "NEW_TASK", "operation": "SEND_MAIL", "target_ref": None,
                              "arguments": {"recipient": "person@example.invalid", "body": {"result_of": "g-summary"}}, "instruction": "send summary"})
        value["relations"] = [{"op": "USE_RESULT", "from_goal": "g-summary", "to_goal": "g-send"}]
        ports = Ports([value])
        result = await resolve_b({"request_id": "current-request", "revision": 2}, ports)
        body = result["goals"][1]["arguments"]["body"]
        self.assertEqual(body, {"kind": "GOAL_RESULT", "producer_goal_id": "g-summary", "request_id": "current-request", "producer_revision": 2})
        self.assertNotIn("content", body)
        self.assertEqual(len(ports.messages), 1)

    async def test_b_denied_read_is_not_replaced_with_hidden_source_content(self):
        ports = Ports([frame()], [{"status": "DENIED", "read_id": "denied-read", "items": [{"ref": "source-a", "kind": "DOCUMENT"}]}])
        result = await resolve_b({"transcript": "deadline"}, ports)
        self.assertEqual(result["unresolved"][0]["reason"], "DENIED")
        self.assertEqual(len(ports.specs), 1)
        self.assertEqual(len(ports.messages), 1)

    async def test_a_parallel_read_batch_preserves_both_results(self):
        specs = [{"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": ref}} for ref in ["a", "b"]]
        ports = Ports([{"action": "READ", "reads": specs}, {"action": "PROPOSE", "meaning": meaning()}],
                      [{"status": "OK", "read_id": "read-a", "items": []}, {"status": "OK", "read_id": "read-b", "items": []}])
        await resolve_a({"transcript": "two reads"}, ports)
        self.assertEqual(len(ports.specs), 2)
        self.assertIn("read-a", json.dumps(ports.messages[1]))
        self.assertIn("read-b", json.dumps(ports.messages[1]))

    async def test_b_patch_preserves_unmentioned_task_and_mail_command(self):
        value = meaning([{"id": "g-update", "kind": "UPDATE_TASK", "operation": "SET_PROPERTY",
                          "target_ref": "r-report", "arguments": {"property": "FORMAT", "value": "TABLE"},
                          "instruction": "change format only"}])
        value.update(version=FRAME_VERSION, references=[
            {"id": "r-report", "selector": {"op": "KNOWN_REF", "source_ref": "report-task"}},
            {"id": "r-mail", "selector": {"op": "KNOWN_REF", "source_ref": "mail-task"}}])
        value["preservations"] = [{"reference_id": "r-mail", "mode": "NO_NEW_COMMAND"}]
        ports = Ports([value])
        inp = {"metadata": {"tasks": [{"task_id": "report-task", "properties": {"PERIOD": "past-period"}},
                                        {"task_id": "mail-task"}]}}
        result = await resolve_b(inp, ports)
        self.assertEqual(result["goals"][0]["target"], "report-task")
        self.assertEqual(result["goals"][0]["arguments"], {"property": "FORMAT", "value": "TABLE"})
        self.assertEqual(result["preservations"], [{"reference_id": "mail-task", "mode": "NO_NEW_COMMAND"}])
        self.assertEqual(len(ports.specs), 0)
        self.assertNotIn("PERIOD", result["goals"][0]["arguments"])

    async def test_b_cannot_bind_withdrawn_question_even_if_reference_is_known(self):
        value = meaning([{"id": "g-answer", "kind": "ANSWER_QUESTION", "operation": "ANSWER_QUESTION",
                          "target_ref": None, "arguments": {"question_ref": "r-q", "answer": "yes"}, "instruction": "answer"}])
        value.update(version=FRAME_VERSION, references=[{"id": "r-q", "selector": {"op": "KNOWN_REF", "source_ref": "q-old"}}])
        ports = Ports([value])
        result = await resolve_b({"metadata": {"questions": [{"question_id": "q-old", "status": "WITHDRAWN", "presented": True}]}}, ports)
        self.assertTrue(result["unresolved"])
        self.assertEqual(result["goals"], [])
        self.assertEqual(len(ports.questions), 1)

    async def test_b_pending_question_binds_for_presentation_only_from_actual_owner_receipt(self):
        question = {"question_id": "q", "revision": 2, "task_id": "work", "execution_id": "exec",
                    "status": "VALID", "text": "현재 질문을 제시할까요?", "kind": "INFORMATION_CONSENT",
                    "delivered_receipt": False, "scope": {"task_id": "work"}}
        value = meaning([{"id": "display", "kind": "DIRECT_RESPONSE", "operation": "QUESTION_PRESENT",
                          "target_ref": None, "arguments": {"question_ref": "r-q"}, "instruction": "실제 질문 제시"}])
        value.update(version=FRAME_VERSION, references=[{"id": "r-q", "selector": {"op": "KNOWN_REF", "source_ref": "q@v2"}}])
        ports = OwnerPorts([value], {"tasks": [{"task_id": "work", "execution_id": "exec", "request_revision": 1,
                                                 "questions": [question]}]})
        result = await resolve_b({"transcript": "질문을 보여줘"}, ports)
        self.assertEqual("QUESTION_PRESENT", result["goals"][0]["operation"])
        self.assertEqual("q", result["goals"][0]["arguments"]["question_ref"])
        self.assertEqual([], ports.questions)
        self.assertEqual(1, len(ports.receipts))
        self.assertFalse(ports.receipts[0]["items"][0]["metadata"]["delivered_receipt"])

    async def test_b_pending_question_is_not_bound_for_answer_or_mixed_consumption(self):
        for operation, mixed in (("ANSWER_QUESTION", False), ("QUESTION_PRESENT", True)):
            with self.subTest(operation=operation, mixed=mixed):
                question = {"question_id": "q", "revision": 2, "task_id": "work", "execution_id": "exec",
                            "status": "VALID", "text": "현재 질문", "kind": "INFORMATION_CONSENT", "delivered_receipt": False}
                args = {"question_ref": "r-q"}
                if operation == "ANSWER_QUESTION":
                    args.update(question_revision=2, answer="yes")
                value = meaning([{"id": "display-or-answer", "kind": "DIRECT_RESPONSE", "operation": operation,
                                  "target_ref": None, "arguments": args, "instruction": "지정 연산"}])
                if mixed:
                    value["goals"].append({"id": "other-use", "kind": "DIRECT_RESPONSE", "operation": "EXPLAIN",
                        "target_ref": None, "arguments": {"source_refs": ["r-q"]}, "instruction": "다른 소비"})
                value.update(version=FRAME_VERSION, references=[{"id": "r-q", "selector": {"op": "KNOWN_REF", "source_ref": "q@v2"}}])
                ports = OwnerPorts([value], {"tasks": [{"task_id": "work", "execution_id": "exec", "request_revision": 1,
                                                         "questions": [question]}]})
                result = await resolve_b({"transcript": "질문 처리"}, ports)
                self.assertEqual([], result["goals"])
                self.assertTrue(result["unresolved"])
                self.assertEqual(1, len(ports.questions))

    async def test_b_pending_question_presentation_ignores_same_named_literal_domain_values(self):
        question = {"question_id": "q", "revision": 2, "status": "VALID", "delivered_receipt": False,
                    "kind": "INFORMATION_CONSENT", "text": "현재 질문"}
        value = meaning([
            {"id": "display", "kind": "DIRECT_RESPONSE", "operation": "QUESTION_PRESENT", "target_ref": None,
                "arguments": {"question_ref": "r-q"}, "instruction": "제시"},
            {"id": "literal-update", "kind": "UPDATE_TASK", "operation": "SET_PROPERTY", "target_ref": "r-work",
                "arguments": {"property": "FORMAT", "value": "r-q"}, "instruction": "문자열 값 유지"},
            {"id": "literal-mail", "kind": "NEW_TASK", "operation": "DRAFT_MAIL", "target_ref": None,
                "arguments": {"recipient": "person@example.invalid", "body": {"text": "r-q"}}, "instruction": "본문 문자열 유지"}])
        value.update(version=FRAME_VERSION, references=[
            {"id": "r-q", "selector": {"op": "KNOWN_REF", "source_ref": "q@v2"}},
            {"id": "r-work", "selector": {"op": "KNOWN_REF", "source_ref": "work"}}])
        ports = OwnerPorts([value], {"tasks": [{"task_id": "work", "execution_id": "exec", "status": "RUNNING",
                                                "request_revision": 1, "questions": [question]}]})
        result = await resolve_b({"transcript": "명시 연산"}, ports)
        self.assertEqual("q", result["goals"][0]["arguments"]["question_ref"])
        self.assertEqual("r-q", result["goals"][1]["arguments"]["value"])
        self.assertEqual({"text": "r-q"}, result["goals"][2]["arguments"]["body"])
        self.assertEqual([], ports.questions)

    async def test_b_presentation_cannot_bind_terminal_or_stale_question(self):
        for status in ("WITHDRAWN", "ANSWERED", "CANCELLED", "STALE"):
            with self.subTest(status=status):
                value = meaning([{"id": "display", "kind": "DIRECT_RESPONSE", "operation": "QUESTION_PRESENT",
                    "target_ref": None, "arguments": {"question_ref": "r-q"}, "instruction": "제시"}])
                value.update(version=FRAME_VERSION, references=[{"id": "r-q", "selector": {"op": "KNOWN_REF", "source_ref": "q"}}])
                ports = Ports([value])
                result = await resolve_b({"metadata": {"questions": [{"question_id": "q", "status": status,
                    "delivered_receipt": False, "revision": 2}]}}, ports)
                self.assertEqual([], result["goals"])
                self.assertTrue(result["unresolved"])

    async def test_unsupported_action_consumes_shared_budget_and_leaves_failure(self):
        ports = Ports([{"action": "RUN_CODE"}] * 2)
        result = await resolve_a({"transcript": "x", "budgets": {"model_calls": 2}}, ports)
        self.assertEqual(len(ports.messages), 2)
        self.assertIn("BudgetExceeded", result["unresolved"][0]["reason"])
        self.assertEqual(sum(kind == "dp41_output_rejected" for kind, _ in ports.events), 2)

    def test_frame_rejects_literal_actual_binding_and_unknown_expression(self):
        value = frame()
        value["goals"][0]["arguments"]["source_refs"] = ["actual-source-id"]
        with self.assertRaises(ContractError):
            validate_document(value, frame=True)
        value = frame()
        value["goals"][0]["arguments"]["expression"] = "compute whatever"
        with self.assertRaises(ContractError):
            validate_document(value, frame=True)

    def test_cyclic_relations_are_rejected(self):
        value = frame()
        value["goals"].append({"id": "g-two", "kind": "DIRECT_RESPONSE", "operation": "EXPLAIN", "target_ref": None,
                              "arguments": {"source_refs": ["r-document"]}, "instruction": "explain"})
        value["relations"] = [{"op": "AFTER", "from_goal": "g-read", "to_goal": "g-two"},
                              {"op": "AFTER", "from_goal": "g-two", "to_goal": "g-read"}]
        with self.assertRaises(ContractError):
            validate_document(value, frame=True)


if __name__ == "__main__":
    unittest.main()
