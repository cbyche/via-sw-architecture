import json
import unittest
from pathlib import Path

from reference_campaign import CONFIGS, build_campaign
from score import load_baseline

HERE = Path(__file__).parent


class ReferenceCampaignTests(unittest.TestCase):
    def test_complete_reference_campaign(self):
        w11 = json.loads((HERE / "w11-protected-units.json").read_text())
        w12 = json.loads((HERE / "w12-safety-opportunities.json").read_text())
        result = build_campaign("f" * 64, load_baseline(), w11, w12)
        self.assertEqual({"W-01", "W-02", "W-03", "W-04", "W-06", "W-11", "W-12"}, set(result["metrics"]))
        for metric in result["metrics"].values():
            self.assertEqual(16, len(metric["rows"]))
            self.assertEqual(set(CONFIGS), {row["configuration_id"] for row in metric["rows"]})
        self.assertEqual(16, len(result["integrated_traces"]))
        self.assertTrue(all(row["score"] == 5 for row in result["metrics"]["W-12"]["rows"]))
        self.assertTrue(all(row["score"] == 3 for row in result["metrics"]["W-11"]["rows"]))
        self.assertFalse(result["product_absolute_latency_claim"])

    def test_pre_frozen_interaction_terms_only_activate_on_joint_choices(self):
        w11 = json.loads((HERE / "w11-protected-units.json").read_text())
        w12 = json.loads((HERE / "w12-safety-opportunities.json").read_text())
        result = build_campaign("f" * 64, load_baseline(), w11, w12)
        w02 = {row["configuration_id"]: row for row in result["metrics"]["W-02"]["rows"]}
        self.assertEqual({}, w02["AAAA"]["active_interactions"])
        self.assertEqual({"IR_B_TASK_B": 4.0}, w02["BBAA"]["active_interactions"])
        self.assertEqual(
            {"IR_B_TASK_B": 4.0, "TASK_B_AGENT_B": 3.0, "AGENT_B_EXEC_B": 4.0},
            w02["BBBB"]["active_interactions"],
        )

    def test_integrated_trace_preserves_runtime_causality(self):
        w11 = json.loads((HERE / "w11-protected-units.json").read_text())
        w12 = json.loads((HERE / "w12-safety-opportunities.json").read_text())
        result = build_campaign("f" * 64, load_baseline(), w11, w12)
        trace = next(row for row in result["integrated_traces"] if row["configuration_id"] == "BBBB")
        self.assertEqual("STAGED", trace["events"][1]["owner"])
        self.assertEqual("TASK_SUPERVISOR", trace["events"][2]["owner"])
        self.assertEqual("INTEGRATION_WORKER", trace["events"][3]["owner"])
        self.assertEqual("TYPED_CORE", trace["events"][4]["owner"])


if __name__ == "__main__":
    unittest.main()
