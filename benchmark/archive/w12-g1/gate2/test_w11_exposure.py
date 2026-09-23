import json
import unittest
from pathlib import Path

from w11_exposure import FROZEN_WORKLOADS, evaluate_trace, validate_oracle

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
ORACLE_PATH = HERE / "w11-protected-units.json"
GENERATOR_PATH = ROOT / "benchmark" / "rebaseline" / "build_assets.py"


class W11PrivacyOracleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))

    def good_trace(self):
        return {
            "candidate_id": "candidate-test",
            "measurement_status": "RECORDED",
            "functional_success_by_workload": {
                workload: True for workload in FROZEN_WORKLOADS
            },
            "egress_events": [],
        }

    def test_oracle_has_exact_frozen_denominator_and_workloads(self):
        unit_ids = validate_oracle(self.oracle)
        self.assertEqual(20, len(unit_ids))
        self.assertEqual(FROZEN_WORKLOADS, set(self.oracle["workloads"]))

    def test_every_unit_is_grounded_in_checked_in_synthetic_generator(self):
        generator = GENERATOR_PATH.read_text(encoding="utf-8")
        for unit in self.oracle["protected_units"]:
            for literal in unit["evidence_literals"]:
                self.assertIn(literal, generator, f"{unit['id']} missing {literal!r}")

    def test_five_unique_remote_units_equal_25_percent_and_score_three(self):
        trace = self.good_trace()
        trace["egress_events"] = [
            {
                "boundary": "outside_user_pc",
                "destination": "remote-model",
                "payload_unit_ids": ["W11-P01", "W11-P02", "W11-P03"],
                "reachable_handle_unit_ids": ["W11-P04", "W11-P05"],
            }
        ]
        result = evaluate_trace(self.oracle, trace)
        self.assertEqual(5, result["exposed_units"])
        self.assertEqual(25.0, result["remote_sensitive_context_exposure_pct"])
        self.assertEqual(3, result["score"])
        self.assertTrue(result["target_met"])

    def test_retransmission_and_multiple_recipients_do_not_expand_union(self):
        trace = self.good_trace()
        trace["egress_events"] = [
            {
                "boundary": "outside_user_pc",
                "destination": "remote-model",
                "payload_unit_ids": ["W11-P07"],
            },
            {
                "boundary": "outside_user_pc",
                "destination": "agent-doc",
                "payload_unit_ids": ["W11-P07"],
            },
            {
                "boundary": "local",
                "destination": "local-model",
                "payload_unit_ids": ["W11-P08", "W11-P09"],
            },
        ]
        result = evaluate_trace(self.oracle, trace)
        self.assertEqual(["W11-P07"], result["exposed_unit_ids"])
        self.assertEqual(5.0, result["remote_sensitive_context_exposure_pct"])
        self.assertEqual(4, result["score"])

    def test_blocking_a_frozen_workload_is_not_valid_zero_exposure(self):
        trace = self.good_trace()
        trace["functional_success_by_workload"]["TC-16.4"] = False
        with self.assertRaisesRegex(ValueError, "functionality failed"):
            evaluate_trace(self.oracle, trace)

    def test_unknown_unit_is_rejected(self):
        trace = self.good_trace()
        trace["egress_events"] = [{
            "boundary": "outside_user_pc",
            "destination": "remote-model",
            "payload_unit_ids": ["W11-UNKNOWN"],
        }]
        with self.assertRaisesRegex(ValueError, "unknown W-11 units"):
            evaluate_trace(self.oracle, trace)


if __name__ == "__main__":
    unittest.main()
