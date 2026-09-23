import unittest

from sensitivity_sweep import build_sweep


FINGERPRINT = "f" * 64


class SensitivitySweepTests(unittest.TestCase):
    def setUp(self):
        self.baseline = {
            "metrics": {
                f"W-{index:02d}": {
                    "direction": "lower" if index != 10 else "higher",
                    "target": 2 if index not in {9, 10} else (3000 if index == 9 else 100),
                }
                for index in range(1, 13)
            }
        }
        self.coverage = {
            "freeze_fingerprint": FINGERPRINT,
            "metrics": {
                f"W-{index:02d}": {
                    "status": "MEASURED" if index in {7, 8, 9, 10} else "NOT_RUN",
                    "reason": "fixture",
                }
                for index in range(1, 13)
            },
        }
        metrics = {}
        for dp in ("IR-DP01", "TASK-DP01", "AGENT-DP01", "EXEC-DP01"):
            for candidate in ("A", "B"):
                metrics[f"{dp}/{candidate}"] = {
                    "W-07": {"metric_value": 1.4, "score": 4},
                    "W-08": {"metric_value": 1.9, "score": 4},
                }
        metrics["AGENT-DP01/B"]["W-07"] = {"metric_value": 1.8, "score": 3}
        self.change = {"freeze_fingerprint": FINGERPRINT, "change_metrics": metrics}
        self.representative = {
            "freeze_fingerprint": FINGERPRINT,
            "metrics": {
                "W-09": {
                    "rows": [
                        {"configuration_id": cid, "metric_value": value, "score": 5}
                        for cid, value in (("AAAA", 550), ("ABAA", 551), ("AABA", 550), ("AAAB", 375))
                    ]
                },
                "W-10": {
                    "rows": [
                        {"configuration_id": cid, "metric_value": 100, "score": 5}
                        for cid in ("AAAA", "ABAA", "AABA", "AAAB")
                    ]
                },
            },
        }

    def test_only_agent_w07_passes_observable_sensitivity(self):
        sweep = build_sweep(self.baseline, self.coverage, self.change, self.representative)
        self.assertEqual("A", sweep["decisions"]["AGENT-DP01"]["selected_candidate"])
        self.assertEqual(
            ["W-07"],
            sweep["decisions"]["AGENT-DP01"]["differentiation_gate"]["primary_metrics"],
        )
        for dp in ("IR-DP01", "TASK-DP01", "EXEC-DP01"):
            self.assertFalse(sweep["decisions"][dp]["differentiation_gate"]["passed"])
            self.assertEqual(
                "FAIL",
                sweep["decisions"][dp]["differentiation_gate"]["g3_observable_sensitivity"],
            )
            self.assertIsNone(sweep["decisions"][dp]["selected_candidate"])

    def test_missing_measurements_are_not_scored_or_imputed(self):
        sweep = build_sweep(self.baseline, self.coverage, self.change, self.representative)
        row = sweep["decisions"]["IR-DP01"]["metrics"][0]
        self.assertEqual("NOT_RUN", row["measurement_status"])
        self.assertIsNone(row["candidates"])
        self.assertFalse(sweep["missing_values_imputed"])
        self.assertIsNone(sweep["weighted_total"])
        self.assertIsNone(sweep["global_winner"])

    def test_agent_selection_follows_the_measured_score_not_candidate_name(self):
        self.change["change_metrics"]["AGENT-DP01/A"]["W-07"] = {
            "metric_value": 1.8,
            "score": 3,
        }
        self.change["change_metrics"]["AGENT-DP01/B"]["W-07"] = {
            "metric_value": 1.4,
            "score": 4,
        }
        sweep = build_sweep(self.baseline, self.coverage, self.change, self.representative)
        self.assertEqual("B", sweep["decisions"]["AGENT-DP01"]["selected_candidate"])

    def test_mixed_fingerprints_are_rejected(self):
        self.representative["freeze_fingerprint"] = "e" * 64
        with self.assertRaisesRegex(ValueError, "one freeze fingerprint"):
            build_sweep(self.baseline, self.coverage, self.change, self.representative)


if __name__ == "__main__":
    unittest.main()
