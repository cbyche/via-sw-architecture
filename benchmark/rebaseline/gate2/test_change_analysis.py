"""Checks for the pre-freeze W-07/W-08 raw change design analysis."""
import unittest
from pathlib import Path

from change_analysis import NOT_COMPUTED, expand, read_json, sha256
from catalog_check import current_revision

ROOT = Path(__file__).resolve().parents[3]
RULES = ROOT / "benchmark/rebaseline/gate2/w07-w08-change-rules.json"


class ChangeAnalysisTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.rules = read_json(RULES)
        cls.result = expand(ROOT, cls.rules, sha256(RULES))
        cls.rows = cls.result["raw_rows"]

    def row(self, candidate_id, change_id):
        matches = [
            row
            for row in self.rows
            if row["candidate_id"] == candidate_id and row["change_id"] == change_id
        ]
        self.assertEqual(1, len(matches))
        return matches[0]

    def test_all_24_changes_expand_to_all_8_pair_labels(self):
        self.assertEqual(24, len(self.rules["changes"]))
        self.assertEqual(192, len(self.rows))
        self.assertEqual(
            {"A": 9, "M": 9, "C": 6},
            {
                family: sum(row["family"] == family for row in self.rules["changes"])
                for family in ("A", "M", "C")
            },
        )
        for candidate in {row["candidate_id"] for row in self.rows}:
            self.assertEqual(24, sum(row["candidate_id"] == candidate for row in self.rows))

    def test_baseline_revision_and_fingerprint_are_present(self):
        self.assertEqual(current_revision(ROOT), self.result["baseline_source_revision"])
        self.assertNotEqual("UNAVAILABLE", self.result["baseline_source_revision"])
        self.assertEqual(64, len(self.result["catalog_fingerprint"]))
        self.assertEqual(64, len(self.result["rules_sha256"]))

    def test_configuration_only_zeroes_are_explicit_not_missing(self):
        zero_ids = {
            row["change_id"]
            for row in self.rows
            if row["candidate_id"] == "IR-DP01/A" and row["unique_count"] == 0
        }
        self.assertEqual({"M-03", "A-01", "A-04", "A-05"}, zero_ids)
        for row in self.rows:
            if row["unique_count"] == 0:
                self.assertEqual("CONFIGURATION_ONLY", row["classification"])

    def test_agent_boundary_difference_is_limited_to_reviewed_lifecycle_changes(self):
        a_counts = {
            cid: self.row("AGENT-DP01/A", cid)["unique_count"]
            for cid in [f"A-{i:02d}" for i in range(1, 10)]
        }
        b_counts = {
            cid: self.row("AGENT-DP01/B", cid)["unique_count"]
            for cid in [f"A-{i:02d}" for i in range(1, 10)]
        }
        for cid in ("A-03", "A-08", "A-09"):
            self.assertEqual(a_counts[cid] + 1, b_counts[cid])
        for cid in set(a_counts) - {"A-03", "A-08", "A-09"}:
            self.assertEqual(a_counts[cid], b_counts[cid])

    def test_non_agent_raw_analysis_does_not_invent_candidate_difference(self):
        reference = {
            row["change_id"]: row["unique_count"]
            for row in self.rows
            if row["candidate_id"] == "IR-DP01/A" and row["family"] in {"M", "C"}
        }
        for candidate in {row["candidate_id"] for row in self.rows}:
            actual = {
                row["change_id"]: row["unique_count"]
                for row in self.rows
                if row["candidate_id"] == candidate and row["family"] in {"M", "C"}
            }
            self.assertEqual(reference, actual)

    def test_same_complete_configuration_is_identical_across_pair_labels(self):
        signatures = {}
        for row in self.rows:
            key = (row["configuration_id"], row["change_id"])
            signature = (
                tuple(row["modified"]),
                tuple(row["added"]),
                tuple(row["removed"]),
                row["unique_count"],
                tuple(sorted(row["type_counts"].items())),
            )
            if key in signatures:
                self.assertEqual(signatures[key], signature)
            else:
                signatures[key] = signature

    def test_added_protocol_contract_has_interface_type(self):
        row = self.row("AGENT-DP01/A", "A-03")
        self.assertEqual(["G2-I-NATIVE-R"], row["added"])
        self.assertEqual(1, row["type_counts"]["I"])

    def test_raw_analysis_cannot_emit_metric_score_or_winner(self):
        self.assertEqual(NOT_COMPUTED, self.result["aggregate_metrics"]["W-07"])
        self.assertEqual(NOT_COMPUTED, self.result["aggregate_metrics"]["W-08"])
        self.assertIsNone(self.result["scores"])
        self.assertIsNone(self.result["winner"])
        self.assertTrue(
            all(
                row["representative_metric_value"] is None
                and row["score"] is None
                and row["candidate_execution"] == "NOT_RUN"
                and row["evidence"] == "DESIGN_ANALYSIS"
                for row in self.rows
            )
        )


if __name__ == "__main__":
    unittest.main()
