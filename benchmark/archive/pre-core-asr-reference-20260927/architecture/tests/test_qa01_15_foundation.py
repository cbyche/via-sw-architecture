#!/usr/bin/env python3
"""Failure-sentinel qualification for the shared QA-01~QA-15 evaluator."""

from __future__ import annotations

import copy
import importlib.util
import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MODULE_PATH = ROOT / "benchmark/architecture/qa01_15_evaluator.py"
SPEC = importlib.util.spec_from_file_location("qa_evaluator", MODULE_PATH)
assert SPEC and SPEC.loader
EVALUATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(EVALUATOR)
CONTRACT = json.loads(
    (ROOT / "benchmark/architecture/contracts/qa01-15-foundation-v1.json").read_text(
        encoding="utf-8"
    )
)


def exact(predicate_id: str, path: str, expected: object) -> dict[str, object]:
    return {
        "id": predicate_id,
        "operator": "EXACT",
        "actual_path": path,
        "expected": expected,
    }


def group(predicate: dict[str, object]) -> dict[str, object]:
    return {"applicability": "APPLICABLE", "predicates": [predicate]}


def baseline_trace() -> dict[str, object]:
    return {
        "events": {
            "user_input_end": {"timestamp_ns": 1_000_000_000, "provenance": "annotated_capture_wav"},
            "agent_request_available_at_agent_ingress": {"timestamp_ns": 2_000_000_000, "provenance": "reference_agent_source"},
            "agent_result_available_at_source": {"timestamp_ns": 8_000_000_000, "provenance": "reference_agent_source"},
            "first_meaningful_audible_result_audio": {"timestamp_ns": 10_000_000_000, "provenance": "audio_loopback"},
            "first_meaningful_audible_direct_response": {"timestamp_ns": 3_000_000_000, "provenance": "audio_loopback"},
            "agent_status_available_at_source": {"timestamp_ns": 4_000_000_000, "provenance": "reference_agent_source"},
            "first_meaningful_audible_status_audio": {"timestamp_ns": 4_500_000_000, "provenance": "audio_loopback"},
            "barge_in_speech_onset": {"timestamp_ns": 5_000_000_000, "provenance": "annotated_capture_wav"},
            "interrupted_response_last_audible_sample": {"timestamp_ns": 5_120_000_000, "provenance": "audio_loopback"},
            "task_control_input_end": {"timestamp_ns": 6_000_000_000, "provenance": "annotated_capture_wav"},
            "correct_task_control_disposition_presented": {"timestamp_ns": 6_700_000_000, "provenance": "audio_loopback"}
        },
        "path_stages": [
            "input_finalization", "semantic_target_resolution", "task_binding",
            "control_delivery", "truthful_disposition"
        ],
        "validity": {
            "result_valid": True,
            "direct_route": True,
            "status_valid": True,
            "waveform_correlated": True,
            "disposition_truthful": True
        },
        "semantic": {
            "goal": "create_deck", "constraints": ["five_slides"],
            "deliverables": ["pptx"], "referents": ["doc-budget"],
            "decomposition": [["summarize", "create"]], "clarification": False,
            "task_relation": "existing", "handling": "delegate",
            "agent_capability": "presentation"
        },
        "binding": {
            "conversation": "C-1", "request": "R-2", "task": "T-PPT",
            "agent_execution": "X-7", "pending_interaction": "Q-3",
            "result_artifact": "A-PPT", "cardinality": 1
        },
        "state": {
            "accepted_revision": 8, "duplicate_suppression": True,
            "stale_rejection": True, "race_resolution": "completed_wins",
            "terminality": "completed", "result_reference": "A-PPT",
            "pending_interaction": None, "allowed_control": ["query"]
        },
        "continuity": {
            "conversation_relation": ["voice-turn-1", "text-turn-2", "C-1"],
            "referent_relation": ["doc-budget", "doc-budget"],
            "task_relation": ["T-PPT", "T-PPT"],
            "channel_transition": "voice_to_text_to_voice",
            "connection_transition": "new_voice_connection_same_conversation"
        },
        "integrated": {
            "response_or_result": True, "semantic": True, "binding": True,
            "state": True, "continuity": True, "cardinality": True,
            "mandatory_gate": True
        }
    }


def baseline_oracle() -> dict[str, object]:
    correctness: dict[str, object] = {
        "QA-12": {
            name: group(exact(name, f"semantic.{name}", expected))
            for name, expected in {
                "goal": "create_deck", "constraints": ["five_slides"],
                "deliverables": ["pptx"], "referents": ["doc-budget"],
                "decomposition": [["summarize", "create"]], "clarification": False,
                "task_relation": "existing", "handling": "delegate",
                "agent_capability": "presentation"
            }.items()
        },
        "QA-13": {
            name: group(exact(name, f"binding.{name}", expected))
            for name, expected in {
                "conversation": "C-1", "request": "R-2", "task": "T-PPT",
                "agent_execution": "X-7", "pending_interaction": "Q-3",
                "result_artifact": "A-PPT", "cardinality": 1
            }.items()
        },
        "QA-14": {
            name: group(exact(name, f"state.{name}", expected))
            for name, expected in {
                "accepted_revision": 8, "duplicate_suppression": True,
                "stale_rejection": True, "race_resolution": "completed_wins",
                "terminality": "completed", "result_reference": "A-PPT",
                "pending_interaction": None, "allowed_control": ["query"]
            }.items()
        },
        "QA-15": {
            name: group(exact(name, f"continuity.{name}", expected))
            for name, expected in {
                "conversation_relation": ["voice-turn-1", "text-turn-2", "C-1"],
                "referent_relation": ["doc-budget", "doc-budget"],
                "task_relation": ["T-PPT", "T-PPT"],
                "channel_transition": "voice_to_text_to_voice",
                "connection_transition": "new_voice_connection_same_conversation"
            }.items()
        },
        "QA-11": {
            name: group(exact(name, f"integrated.{name}", True))
            for name in CONTRACT["correctness"]["QA-11"]["required_groups"]
        }
    }
    return {
        "latency_applicability": {
            "QA-01": "APPLICABLE", "QA-02": "APPLICABLE", "QA-03": "APPLICABLE",
            "QA-04": "APPLICABLE", "QA-05": "APPLICABLE"
        },
        "latency": {
            qa_id: {
                "predicates": [exact(qa_id, path, True)]
            }
            for qa_id, path in {
                "QA-01": "validity.result_valid",
                "QA-02": "validity.direct_route",
                "QA-03": "validity.status_valid",
                "QA-04": "validity.waveform_correlated",
                "QA-05": "validity.disposition_truthful"
            }.items()
        },
        "correctness": correctness,
        "qa11_driver_applicability": {
            "QA-12": "APPLICABLE", "QA-13": "APPLICABLE",
            "QA-14": "APPLICABLE", "QA-15": "APPLICABLE"
        }
    }


class FoundationQualificationTests(unittest.TestCase):
    def test_normal_trace_passes_every_qa(self) -> None:
        trace, oracle = baseline_trace(), baseline_oracle()
        evaluated = EVALUATOR.evaluate_trial(CONTRACT, trace, oracle)
        latency = evaluated["latency"]
        correctness = evaluated["correctness"]
        self.assertTrue(all(result["pass"] for result in latency.values()))
        self.assertTrue(all(result["pass"] for result in correctness.values()))
        self.assertEqual(latency["QA-01"]["sample_ns"], 3_000_000_000)

    def test_voice_proxy_cannot_end_qa01_or_qa04(self) -> None:
        trace, oracle = baseline_trace(), baseline_oracle()
        trace["events"]["first_meaningful_audible_result_audio"]["provenance"] = "renderer_callback"
        trace["events"]["interrupted_response_last_audible_sample"]["provenance"] = "queue_operation"
        result = EVALUATOR.evaluate_latency(CONTRACT, trace, oracle)
        self.assertIn("QA-01:first_meaningful_audible_result_audio:PROVENANCE_INVALID", result["QA-01"]["failures"])
        self.assertIn("QA-04:interrupted_response_last_audible_sample:PROVENANCE_INVALID", result["QA-04"]["failures"])

    def test_missing_ingress_and_invalid_direct_or_status_endpoints_fail(self) -> None:
        trace, oracle = baseline_trace(), baseline_oracle()
        del trace["events"]["agent_request_available_at_agent_ingress"]
        trace["validity"]["direct_route"] = False
        trace["events"]["agent_status_available_at_source"]["provenance"] = "via_receive"
        trace["validity"]["status_valid"] = False
        result = EVALUATOR.evaluate_latency(CONTRACT, trace, oracle)
        self.assertIn("QA-01:agent_request_available_at_agent_ingress:EVENT_MISSING", result["QA-01"]["failures"])
        self.assertIn("QA-02:VALIDITY:QA-02", result["QA-02"]["failures"])
        self.assertIn("QA-03:agent_status_available_at_source:PROVENANCE_INVALID", result["QA-03"]["failures"])
        self.assertIn("QA-03:VALIDITY:QA-03", result["QA-03"]["failures"])

    def test_qa05_cannot_skip_semantic_target_resolution(self) -> None:
        trace, oracle = baseline_trace(), baseline_oracle()
        trace["path_stages"].remove("semantic_target_resolution")
        result = EVALUATOR.evaluate_latency(CONTRACT, trace, oracle)["QA-05"]
        self.assertFalse(result["pass"])
        self.assertIn("QA-05:semantic_target_resolution:PATH_STAGE_MISSING", result["failures"])

    def test_untruthful_qa05_disposition_cannot_be_fast_success(self) -> None:
        trace, oracle = baseline_trace(), baseline_oracle()
        trace["validity"]["disposition_truthful"] = False
        result = EVALUATOR.evaluate_latency(CONTRACT, trace, oracle)["QA-05"]
        self.assertFalse(result["pass"])
        self.assertEqual(result["sample_ns"], 5_000_000_000)
        self.assertTrue(result["censored"])
        self.assertIn("QA-05:VALIDITY:QA-05", result["failures"])

    def test_semantic_constraint_failure_propagates_to_qa11(self) -> None:
        trace, oracle = baseline_trace(), baseline_oracle()
        trace["semantic"]["constraints"] = []
        result = EVALUATOR.evaluate_correctness(CONTRACT, trace, oracle)
        self.assertIn("QA-12:constraints:constraints", result["QA-12"]["failures"])
        self.assertIn("QA-11:QA-12:DRIVER_FAILED", result["QA-11"]["failures"])
        latency = EVALUATOR.evaluate_trial(CONTRACT, trace, oracle)["latency"]
        self.assertIn("QA-01:QA-11:CORRECTNESS_FAILED", latency["QA-01"]["failures"])
        self.assertIn("QA-02:QA-11:CORRECTNESS_FAILED", latency["QA-02"]["failures"])
        self.assertIn("QA-05:QA-11:CORRECTNESS_FAILED", latency["QA-05"]["failures"])
        self.assertEqual(latency["QA-01"]["sample_ns"], 3_000_000_000)
        self.assertEqual(latency["QA-01"]["observed_sample_ns"], 3_000_000_000)
        self.assertFalse(latency["QA-01"]["censored"])
        self.assertEqual(
            latency["QA-01"]["status"],
            "RESPONSE_OBSERVED_CORRECTNESS_FAILED",
        )

    def test_explicitly_non_applicable_driver_does_not_fail_qa05(self) -> None:
        trace, oracle = baseline_trace(), baseline_oracle()
        oracle["correctness"].pop("QA-14")
        oracle["correctness"].pop("QA-15")
        oracle["qa11_driver_applicability"]["QA-14"] = "N/A"
        oracle["qa11_driver_applicability"]["QA-15"] = "N/A"
        result = EVALUATOR.evaluate_trial(CONTRACT, trace, oracle)["latency"]["QA-05"]
        self.assertTrue(result["correctness_pass"])
        self.assertEqual(result["status"], "PASS")

    def test_qa11_own_integrated_cardinality_failure_is_not_hidden(self) -> None:
        trace, oracle = baseline_trace(), baseline_oracle()
        trace["integrated"]["cardinality"] = False
        result = EVALUATOR.evaluate_correctness(CONTRACT, trace, oracle)["QA-11"]
        self.assertIn("QA-11:cardinality:cardinality", result["failures"])

    def test_wrong_binding_and_duplicate_delivery_fail_qa13(self) -> None:
        trace, oracle = baseline_trace(), baseline_oracle()
        trace["binding"]["task"] = "T-MAIL"
        trace["binding"]["cardinality"] = 2
        result = EVALUATOR.evaluate_correctness(CONTRACT, trace, oracle)["QA-13"]
        self.assertIn("QA-13:task:task", result["failures"])
        self.assertIn("QA-13:cardinality:cardinality", result["failures"])

    def test_stale_or_reopened_state_fails_qa14(self) -> None:
        trace, oracle = baseline_trace(), baseline_oracle()
        trace["state"]["stale_rejection"] = False
        trace["state"]["terminality"] = "running"
        result = EVALUATOR.evaluate_correctness(CONTRACT, trace, oracle)["QA-14"]
        self.assertIn("QA-14:stale_rejection:stale_rejection", result["failures"])
        self.assertIn("QA-14:terminality:terminality", result["failures"])

    def test_cross_channel_task_loss_fails_qa15(self) -> None:
        trace, oracle = baseline_trace(), baseline_oracle()
        trace["continuity"]["task_relation"] = ["T-PPT", "T-NEW"]
        result = EVALUATOR.evaluate_correctness(CONTRACT, trace, oracle)["QA-15"]
        self.assertIn("QA-15:task_relation:task_relation", result["failures"])

    def test_missing_group_is_not_default_pass(self) -> None:
        trace, oracle = baseline_trace(), baseline_oracle()
        del oracle["correctness"]["QA-15"]["connection_transition"]
        result = EVALUATOR.evaluate_correctness(CONTRACT, trace, oracle)["QA-15"]
        self.assertIn("QA-15:connection_transition:GROUP_MISSING", result["failures"])

    def test_missing_latency_applicability_is_not_inferred(self) -> None:
        trace, oracle = baseline_trace(), baseline_oracle()
        del oracle["latency_applicability"]["QA-03"]
        result = EVALUATOR.evaluate_latency(CONTRACT, trace, oracle)["QA-03"]
        self.assertEqual(result["status"], "INVALID")
        self.assertIn("QA-03:APPLICABILITY_MISSING", result["failures"])

    def test_aggregation_is_case_p95_then_worst_case(self) -> None:
        samples = {"easy": list(range(1, 101)), "hard": list(range(101, 201))}
        self.assertEqual(EVALUATOR.aggregate_latency(samples), 195)
        self.assertEqual(
            EVALUATOR.aggregate_correctness({"easy": [True, True], "hard": [True, False]}),
            75.0,
        )

    def test_every_frozen_oracle_operator_is_executable(self) -> None:
        trace = {
            "v": "x", "set": ["a", "b"], "ordered": [["a", "b"]],
            "props": ["required", "safe"], "clarify": True, "count": 1
        }
        predicates = [
            {"id":"exact", "operator":"EXACT", "actual_path":"v", "expected":"x"},
            {"id":"one", "operator":"ONE_OF", "actual_path":"v", "expected":["x","y"]},
            {"id":"set", "operator":"SET_EQUAL", "actual_path":"set", "expected":["b","a"]},
            {"id":"ordered", "operator":"ORDERED_RELATION", "actual_path":"ordered", "expected":[["a","b"]]},
            {"id":"required", "operator":"REQUIRED_PROPOSITION", "actual_path":"props", "expected":["required"]},
            {"id":"forbidden", "operator":"FORBIDDEN_PROPOSITION", "actual_path":"props", "expected":["secret"]},
            {"id":"clarify", "operator":"CLARIFICATION_REQUIRED", "actual_path":"clarify", "expected":True},
            {"id":"binding", "operator":"BINDING", "actual_path":"v", "expected":"x"},
            {"id":"cardinality", "operator":"CARDINALITY", "actual_path":"count", "expected":1},
            {"id":"state", "operator":"FINAL_STATE_EQUAL", "actual_path":"v", "expected":"x"},
            {"id":"continuity", "operator":"CONTINUITY_RELATION", "actual_path":"v", "expected":"x"},
        ]
        self.assertEqual(
            {predicate["operator"] for predicate in predicates},
            set(CONTRACT["oracle_operators"]),
        )
        self.assertTrue(all(EVALUATOR.predicate_passes(predicate, trace) for predicate in predicates))


if __name__ == "__main__":
    unittest.main()
