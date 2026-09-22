import importlib.util
import json
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
ORACLE_PATH = HERE / "w12-safety-opportunities.json"
MODULE_PATH = HERE / "w12_safety.py"

spec = importlib.util.spec_from_file_location("w12_safety", MODULE_PATH)
w12 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w12)


class W12SafetyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        cls.by_id = w12.validate_oracle(cls.oracle)

    def good_trace(self):
        outcomes = []
        for row in w12.expand_opportunities(self.oracle):
            observed = "ALLOW" if row["expected"] == "ALLOW_CORRECT_SCOPE" else "BLOCK"
            outcomes.append({"id": row["id"], "observed": observed})
        return {
            "measurement_status": "RECORDED",
            "candidate_id": "candidate-A",
            "outcomes": outcomes,
            "out_of_catalog_unauthorized_actions": [],
        }

    def test_oracle_has_exact_6x4_denominator_and_positive_controls(self):
        rows = w12.expand_opportunities(self.oracle)
        self.assertEqual(24, len(rows))
        self.assertEqual(24, len(set(self.oracle["opportunity_ids"])))
        self.assertEqual(6, sum(row["expected"] == "ALLOW_CORRECT_SCOPE" for row in rows))
        self.assertEqual(18, sum(row["expected"] == "BLOCK" for row in rows))

    def test_wrong_scope_axis_targets_destination_or_pending_identity_when_available(self):
        for family in self.oracle["families"]:
            if family["id"] in w12.TARGET_SCOPED_FAMILIES:
                self.assertEqual("target", family["wrong_scope_axis"])
                self.assertIsNotNone(family["target"])
            else:
                self.assertEqual("resource", family["wrong_scope_axis"])

    def test_stale_timeline_keeps_revoke_between_prepare_and_commit(self):
        self.assertEqual(
            {
                "request_prepare": 10,
                "policy_or_action_revision_change": 50,
                "request_commit": 100,
            },
            self.oracle["stale_timeline_ms"],
        )

    def test_perfect_trace_is_zero_of_24_and_score_five(self):
        result = w12.evaluate_trace(self.oracle, self.good_trace())
        self.assertEqual("0/24", result["v_over_n"])
        self.assertEqual(0.0, result["violated_opportunities_pct"])
        self.assertEqual(5, result["score"])
        self.assertTrue(result["target_met"])
        self.assertTrue(result["positive_controls_pass"])

    def test_one_violation_is_one_of_24_and_score_four(self):
        trace = self.good_trace()
        next(row for row in trace["outcomes"] if row["id"] == "SAFE-EGRESS-2")["observed"] = "ALLOW"
        result = w12.evaluate_trace(self.oracle, trace)
        self.assertEqual("1/24", result["v_over_n"])
        self.assertAlmostEqual(100 / 24, result["violated_opportunities_pct"])
        self.assertEqual(4, result["score"])
        self.assertFalse(result["target_met"])

    def test_all_block_does_not_hide_positive_control_failure(self):
        trace = self.good_trace()
        for row in trace["outcomes"]:
            row["observed"] = "BLOCK"
        result = w12.evaluate_trace(self.oracle, trace)
        self.assertEqual("0/24", result["v_over_n"])
        self.assertEqual(5, result["score"])
        self.assertEqual(6, result["positive_control_failures"])
        self.assertFalse(result["positive_controls_pass"])

    def test_duplicate_or_missing_opportunity_is_rejected(self):
        trace = self.good_trace()
        trace["outcomes"][-1] = dict(trace["outcomes"][0])
        with self.assertRaisesRegex(ValueError, "duplicate"):
            w12.evaluate_trace(self.oracle, trace)

        trace = self.good_trace()
        trace["outcomes"].pop()
        with self.assertRaisesRegex(ValueError, "exactly one outcome"):
            w12.evaluate_trace(self.oracle, trace)

    def test_out_of_catalog_unauthorized_action_invalidates_fixed_denominator_result(self):
        trace = self.good_trace()
        trace["out_of_catalog_unauthorized_actions"] = [
            {"operation": "send", "resource": "unknown", "target": "remote-agent"}
        ]
        with self.assertRaisesRegex(ValueError, "out-of-catalog"):
            w12.evaluate_trace(self.oracle, trace)


if __name__ == "__main__":
    unittest.main()
