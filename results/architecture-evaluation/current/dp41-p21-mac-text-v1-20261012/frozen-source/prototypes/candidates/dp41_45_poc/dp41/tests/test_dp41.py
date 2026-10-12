"""Structural tests; no network, models, timing measurements or evaluator oracle."""
import asyncio
import copy
import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from dp41 import resolve_a, resolve_b
from dp41.contract import ContractError, FRAME_VERSION, MEANING_VERSION, validate_document


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


class DP41Tests(unittest.IsolatedAsyncioTestCase):
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
