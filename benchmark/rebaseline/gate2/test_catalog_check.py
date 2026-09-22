"""Unit checks for the Gate 2 review metadata, not for VIA candidates."""
import copy
import unittest
from pathlib import Path

from catalog_check import (CORE, CHANGES, WORKING, bindings, catalog_fingerprint, compose,
                           current_revision, elements, factorial_vectors, load, make_report)

ROOT = Path(__file__).resolve().parents[3]


class ReviewContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.catalog = load(ROOT)
        cls.report = make_report(cls.catalog)
        cls.reference = cls.report["reference_choices"]

    def test_four_scored_dp_pairs(self):
        self.assertEqual(set(self.catalog["cards"]), set(CORE))
        self.assertEqual(set(CORE), {"IR-DP01", "TASK-DP01", "AGENT-DP01", "EXEC-DP01"})

    def test_pair_candidates_remain_one_factor_contrasts(self):
        for pair in self.report["pair_candidates"]:
            dp = pair["candidate_id"].split("/")[0]
            vector = self.report["configurations"][pair["configuration_id"]]["choices"]
            self.assertTrue(all(vector[k] == self.reference[k] for k in CORE if k != dp))

    def test_complete_factorial_and_pair_contrasts_are_both_present(self):
        self.assertEqual(len(self.report["configurations"]), 16)
        self.assertEqual(len(self.report["pair_candidates"]), 8)
        self.assertEqual(
            [configuration_id for configuration_id, _ in factorial_vectors()],
            list(self.report["configurations"]),
        )

    def test_all_working_asrs_present(self):
        for pair in self.report["pair_candidates"]:
            actual = {r["working_asr"] for r in self.report["metrics"] if r["candidate_id"] == pair["candidate_id"]}
            self.assertEqual(actual, set(WORKING))

    def test_all_changes_present(self):
        for pair in self.report["pair_candidates"]:
            actual = {r["change_id"] for r in self.report["changes"] if r["candidate_id"] == pair["candidate_id"]}
            self.assertEqual(actual, set(CHANGES))

    def test_source_revision_tracks_current_checkout(self):
        self.assertEqual(self.report["source_revision"], current_revision(ROOT))
        self.assertNotEqual(self.report["source_revision"], "UNAVAILABLE")

    def test_catalog_fingerprint_is_structural_and_mutation_sensitive(self):
        self.assertEqual(self.report["catalog_fingerprint"], catalog_fingerprint(self.catalog))
        self.assertEqual(len(self.report["catalog_fingerprint"]), 64)
        mutated = copy.deepcopy(self.catalog)
        eid = next(iter(mutated["elements"]))
        mutated["elements"][eid]["responsibility"] += " changed"
        self.assertNotEqual(catalog_fingerprint(mutated), self.report["catalog_fingerprint"])

    def test_unmeasured_is_not_zero(self):
        self.assertTrue(all(r["score"] is None and r["metric_value"] is None and r["evidence"] == "NOT_RUN" for r in self.report["metrics"]))

    def test_unanalyzed_is_not_zero_change(self):
        self.assertTrue(all(r["unique_count"] is None and r["modified"] is None for r in self.report["changes"]))

    def test_count_types_exist_for_all_configs(self):
        for config in self.report["configurations"].values():
            self.assertTrue(all(config["element_type_counts"][t] > 0 for t in "CISD"))

    def test_fixed_voice_fast_path_is_common(self):
        for config in self.report["configurations"].values():
            self.assertEqual(config["bindings"]["voice_direct_release_owner"], "G2-C-VOICE")

    def test_fixed_agent_sync_is_common(self):
        for config in self.report["configurations"].values():
            self.assertEqual(config["bindings"]["normal_sync_owner"], "G2-C-AGENTSYNC")

    def test_task_actor_not_a_process_change(self):
        vector = dict(self.reference, **{"TASK-DP01": "B"})
        self.assertEqual(bindings(vector)["task_host"], "G2-D-VIA")
        self.assertNotIn("G2-D-INTEGRATION", compose(self.catalog, vector))

    def test_exec_changes_host_not_task_writer(self):
        a = dict(self.reference, **{"EXEC-DP01": "A"})
        b = dict(self.reference, **{"EXEC-DP01": "B"})
        self.assertEqual(bindings(a)["task_state_writer"], bindings(b)["task_state_writer"])
        self.assertNotEqual(bindings(a)["integration_host"], bindings(b)["integration_host"])
        self.assertFalse(bindings(b)["integration_db_write_permission"])

    def test_missing_dp_rejected(self):
        vector = dict(self.reference)
        vector.pop(CORE[0])
        with self.assertRaises(ValueError):
            compose(self.catalog, vector)

    def test_invalid_choice_rejected(self):
        with self.assertRaises(ValueError):
            compose(self.catalog, dict(self.reference, **{"IR-DP01": "C"}))

    def test_duplicate_element_rejected(self):
        row = "| G2-C-X | responsibility | owner/lifetime | trigger |\n"
        with self.assertRaises(ValueError):
            elements(row + row, "test")

    def test_inactive_binding_rejected(self):
        broken = copy.deepcopy(self.catalog)
        broken["cards"]["EXEC-DP01"]["alternatives"]["B"].remove("G2-D-INTEGRATION")
        with self.assertRaises(ValueError):
            make_report(broken)


if __name__ == "__main__":
    unittest.main()
