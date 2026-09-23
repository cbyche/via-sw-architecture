import unittest

from factorial_analysis import build


class FactorialAnalysisTests(unittest.TestCase):
    def make_rows(self, metric, value):
        rows = []
        for ir in "AB":
            for task in "AB":
                for agent in "AB":
                    for execution in "AB":
                        config = ir + task + agent + execution
                        measured = value(config) if callable(value) else value
                        rows.append({"configuration_id": config, "metric_value": measured, "score": 5, "evidence": "TEST"})
        return {"rows": rows}

    def test_build_requires_and_preserves_all_sixteen_configurations(self):
        reference = {"freeze_fingerprint": "r" * 64, "metrics": {}}
        for metric in ("W-01", "W-02", "W-03", "W-04", "W-06", "W-11", "W-12"):
            reference["metrics"][metric] = self.make_rows(metric, 1)
        representative = {"freeze_fingerprint": "s" * 64, "metrics": {
            "W-09": self.make_rows("W-09", 1), "W-10": self.make_rows("W-10", 100)}}
        changes = {"change_metrics": {
            f"{dp}/{choice}": {
                "W-07": {"metric_value": 1 if choice == "A" or dp != "AGENT-DP01" else 2, "score": 4 if choice == "A" or dp != "AGENT-DP01" else 3},
                "W-08": {"metric_value": 1.9, "score": 4},
            }
            for dp in ("IR-DP01", "TASK-DP01", "AGENT-DP01", "EXEC-DP01") for choice in "AB"
        }}
        baseline = {"metrics": {f"W-{i:02d}": {"direction": "higher" if i in {6, 10} else "lower"} for i in range(1, 13)}}
        result = build(reference, representative, changes, baseline)
        self.assertEqual(16, result["configuration_count"])
        self.assertEqual(16, len(result["configurations"]))
        self.assertEqual("BLOCKED_OPENROUTER_KEY", result["W-05"]["status"])
        self.assertIsNone(result["global_winner"])

    def test_factorial_analysis_recovers_a_frozen_interaction(self):
        reference = {"freeze_fingerprint": "r" * 64, "metrics": {}}
        for metric in ("W-01", "W-02", "W-03", "W-06", "W-11", "W-12"):
            reference["metrics"][metric] = self.make_rows(metric, 1)
        reference["metrics"]["W-04"] = self.make_rows(
            "W-04", lambda config: 1.0 - (0.01 if config[1] == "B" and config[3] == "B" else 0.0)
        )
        representative = {"freeze_fingerprint": "s" * 64, "metrics": {
            "W-09": self.make_rows("W-09", 1), "W-10": self.make_rows("W-10", 100)}}
        changes = {"change_metrics": {
            f"{dp}/{choice}": {"W-07": {"metric_value": 1, "score": 4}, "W-08": {"metric_value": 1.9, "score": 4}}
            for dp in ("IR-DP01", "TASK-DP01", "AGENT-DP01", "EXEC-DP01") for choice in "AB"
        }}
        baseline = {"metrics": {f"W-{i:02d}": {"direction": "higher" if i in {6, 10} else "lower"} for i in range(1, 13)}}
        result = build(reference, representative, changes, baseline)
        interaction = result["metric_analysis"]["W-04"]["pairwise_interaction_difference_of_differences"]
        self.assertAlmostEqual(-0.01, interaction["TASK-DP01×EXEC-DP01"])


if __name__ == "__main__":
    unittest.main()
