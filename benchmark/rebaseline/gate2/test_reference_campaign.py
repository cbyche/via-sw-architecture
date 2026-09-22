import json
import unittest
from pathlib import Path

from reference_campaign import build_campaign
from score import load_baseline

HERE = Path(__file__).parent


class ReferenceCampaignTests(unittest.TestCase):
    def test_complete_reference_campaign(self):
        w11 = json.loads((HERE / "w11-protected-units.json").read_text())
        w12 = json.loads((HERE / "w12-safety-opportunities.json").read_text())
        result = build_campaign("f" * 64, load_baseline(), w11, w12)
        self.assertEqual({"W-01", "W-02", "W-03", "W-04", "W-06", "W-11", "W-12"}, set(result["metrics"]))
        for metric in result["metrics"].values():
            self.assertEqual(5, len(metric["rows"]))
        self.assertTrue(all(row["score"] == 5 for row in result["metrics"]["W-12"]["rows"]))
        self.assertTrue(all(row["score"] == 3 for row in result["metrics"]["W-11"]["rows"]))
        self.assertFalse(result["product_absolute_latency_claim"])


if __name__ == "__main__":
    unittest.main()
