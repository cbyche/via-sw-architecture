import unittest
from combined_sweep import build


class CombinedSweepTests(unittest.TestCase):
    def test_current_shape_selects_three_supported_decisions(self):
        def rows(wid, values):
            ids = ["AAAA", "BAAA", "ABAA", "AABA", "AAAB"]
            return {"rows": [{"configuration_id": cid, "metric_value": value, "score": score, "evidence": "REFERENCE"} for cid, (value, score) in zip(ids, values)]}
        reference = {"freeze_fingerprint": "r" * 64, "metrics": {
            "W-01": rows("W-01", [(300,5)]*5), "W-02": rows("W-02", [(70,5)]*5),
            "W-03": rows("W-03", [(29,5)]*5), "W-04": rows("W-04", [(1.06,4),(1.06,4),(1.05,5),(1.06,4),(1.05,5)]),
            "W-06": rows("W-06", [(100,5)]*5), "W-11": rows("W-11", [(25,3)]*5), "W-12": rows("W-12", [(0,5)]*5)}}
        changes = {"change_metrics": {f"{dp}/{c}": {"W-07": {"metric_value": 1.4 if c=="A" or dp!="AGENT-DP01" else 1.8, "score": 4 if c=="A" or dp!="AGENT-DP01" else 3}, "W-08": {"metric_value": 1.9, "score": 4}} for dp in ("IR-DP01","TASK-DP01","AGENT-DP01","EXEC-DP01") for c in "AB"}}
        representative = {"freeze_fingerprint": "s" * 64, "metrics": {"W-09": rows("W-09", [(550,5)]*5), "W-10": rows("W-10", [(100,5)]*5)}}
        result = build(reference, changes, representative)
        self.assertEqual("B", result["decisions"]["TASK-DP01"]["selected_candidate"])
        self.assertEqual("A", result["decisions"]["AGENT-DP01"]["selected_candidate"])
        self.assertEqual("B", result["decisions"]["EXEC-DP01"]["selected_candidate"])
        self.assertIsNone(result["decisions"]["IR-DP01"]["selected_candidate"])
        self.assertIsNone(result["weighted_total"])


if __name__ == "__main__":
    unittest.main()
