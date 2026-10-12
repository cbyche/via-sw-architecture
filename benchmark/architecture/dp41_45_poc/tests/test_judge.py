"""Reject invented provenance while accepting real mixed stop/publication receipts."""
import copy
import json
import unittest

from benchmark.architecture.dp41_45_poc.judge import evidence_repair_feedback, validate_judgement


class JudgeProvenanceTests(unittest.TestCase):
    def setUp(self):
        self.trial = {
            "events": [{"kind": "READ_RETURNED", "evidence_id": "E1", "source_id": "D1", "causal_node_id": "TL:N1"}],
            "effects": [], "meanings": [],
            "deliveries": [{"kind": "LOCAL_STOP", "text": "old"},
                {"kind": "RESPONSE", "publication_id": "P1", "text": "기한은 5일입니다.",
                 "voice_text": "기한은 5일입니다.", "voice_on": True,
                 "text_presented_s": 1.0, "voice_onset_s": 1.1}],
            "reaction_blueprint": [{"id": "stop", "endpoint_kind": "LOCAL_OUTPUT_STOP"}],
            "initial": {}, "observed_inputs": [], "final_tasks": {}, "final_questions": {},
            "cycle_blueprint": None,
        }
        self.result = {"conditions": {f"C{i}": {"passed": False, "evidence": ["E1"], "reason": "test"} for i in range(1, 7)},
            "reactions": [{"id": "stop", "first_actual_publication_id": None, "valid_publication_id": None,
                "first_actual_core_quote": None, "valid_core_quote": None, "wrong_first": False, "evidence": ["E1"]}],
            "cycle": {"valid_publication_id": "P1", "valid_core_quote": "기한은 5일입니다.", "evidence": ["P1"]},
            "critical_violations": [], "uncertain": False}

    def test_mixed_stop_and_actual_publication_validates(self):
        self.assertIs(validate_judgement(self.result, self.trial), self.result)

    def test_fabricated_publication_is_rejected(self):
        self.result["cycle"]["valid_publication_id"] = "P_INVENTED"
        with self.assertRaises(ValueError):
            validate_judgement(self.result, self.trial)

    def test_unspoken_quote_is_rejected(self):
        self.result["cycle"]["valid_core_quote"] = "기한은 6일입니다."
        with self.assertRaises(ValueError):
            validate_judgement(self.result, self.trial)

    def test_local_stop_cannot_use_another_publication(self):
        self.result["reactions"][0].update(valid_publication_id="P1", valid_core_quote="기한은 5일입니다.")
        with self.assertRaises(ValueError):
            validate_judgement(self.result, self.trial)

    def test_reference_repair_only_points_to_actual_enclosing_evidence(self):
        self.result["conditions"]["C1"]["evidence"] = ["D1", "TL:N1", "D_INVENTED"]
        original = copy.deepcopy(self.result)
        feedback = evidence_repair_feedback(self.result, self.trial, ValueError("invalid evidence"))
        body = json.loads(feedback.split("\n", 1)[1])
        self.assertEqual(self.result, original)
        self.assertEqual(body["actual_supplied_records_containing_those_values"]["D1"], [{"evidence_id": "E1"}])
        self.assertEqual(body["actual_supplied_records_containing_those_values"]["TL:N1"], [{"evidence_id": "E1"}])
        self.assertEqual(body["actual_supplied_records_containing_those_values"]["D_INVENTED"], [])
        self.assertEqual(body["allowed_evidence_references"], ["E1", "P1"])


if __name__ == "__main__":
    unittest.main()
