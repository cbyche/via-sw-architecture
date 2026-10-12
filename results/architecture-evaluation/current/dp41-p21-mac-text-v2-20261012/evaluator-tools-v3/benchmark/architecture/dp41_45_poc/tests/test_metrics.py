"""Failure-sensitive aggregation tests; these are not PoC measurements."""
import importlib.util
import json
from pathlib import Path
import unittest


MODULE = Path(__file__).resolve().parents[1] / "metrics.py"
spec = importlib.util.spec_from_file_location("poc_metrics_under_test", MODULE)
metrics = importlib.util.module_from_spec(spec)
spec.loader.exec_module(metrics)


def trial(variant="A", repeat=1, case_id="S1-01", **updates):
    record = {"variant": variant, "repeat": repeat, "case_id": case_id,
              "conditions": {f"C{i}": {"passed": True, "evidence": [f"trace-C{i}"]} for i in range(1, 7)},
              "reactions": [{"id": "I-S1-01-1", "start_s": 0, "end_s": 1, "valid": True,
                             "first_actual_s": 1, "wrong_first": False}],
              "cycle": {"id": "T-S1-01", "start_s": 0, "valid_end_s": 1, "completed": True,
                        "attributed_seconds": 1, "actual_seconds": 1}}
    record.update(updates)
    return record


def manifest(**updates):
    record = {"case_ids": ["S1-01"], "repeat_ids": [1], "variants": ["A"], "strict_coverage": True}
    record.update(updates)
    return record


class MetricsTests(unittest.TestCase):
    def test_wrong_first_corrected_stays_cap_and_excluded_from_correct_only(self):
        row = trial(reactions=[{"id": "I-S1-01-1", "start_s": 2, "end_s": 3.4,
                                "valid": True, "first_actual_s": 2.2, "wrong_first": True}])
        qa = metrics.aggregate([row], [], [], [], manifest())["variants"]["A"]["qa02"]
        self.assertEqual(qa["cap_mean"]["value"], 5)
        self.assertEqual(qa["correct_only_mean"]["n"], 0)
        self.assertAlmostEqual(qa["actual_including_wrong_mean"]["value"], .2)
        self.assertAlmostEqual(qa["observations"][0]["correction_seconds"], 1.4)

    def test_valid_six_second_response_is_never_clipped(self):
        row = trial(reactions=[{"id": "I-S1-01-1", "start_s": 2, "end_s": 8,
                                "valid": True, "first_actual_s": 8, "wrong_first": False}])
        qa = metrics.aggregate([row], [], [], [], manifest())["variants"]["A"]["qa02"]
        self.assertEqual(qa["cap_mean"]["value"], 6)
        self.assertEqual(qa["correct_only_mean"]["value"], 6)
        self.assertEqual(qa["actual_including_wrong_mean"]["value"], 6)
        self.assertEqual(qa["guards"]["ordinary_within_2"], 0)

    def test_missing_response_remains_cap_without_fake_actual_time(self):
        row = trial(reactions=[{"id": "I-S1-01-1", "start_s": 2, "end_s": None,
                                "valid": False, "first_actual_s": None}])
        qa = metrics.aggregate([row], [], [], [], manifest())["variants"]["A"]["qa02"]
        self.assertEqual(qa["cap_mean"], {"value": 5, "numerator": 5, "n": 1})
        self.assertIsNone(qa["actual_including_wrong_mean"]["value"])
        self.assertEqual(qa["missing_count"], 1)

    def test_attributed_wrong_time_cannot_replace_missing_actual_first_timestamp(self):
        row = trial(reactions=[{"id": "I-S1-01-1", "start_s": 2, "end_s": None,
                                "valid": False, "wrong_first": True, "first_actual_s": None,
                                "first_actual_attributed_seconds": .01}])
        qa = metrics.aggregate([row], [], [], [], manifest())["variants"]["A"]["qa02"]
        self.assertEqual(qa["cap_mean"]["value"], 5)
        self.assertEqual(qa["actual_including_wrong_mean"]["n"], 0)

    def test_late_completed_cycle_keeps_actual_time_in_completed_subset(self):
        late = trial(cycle={"id": "T-S1-01", "start_s": 0, "valid_end_s": 49,
                            "completed": True, "attributed_seconds": 31, "actual_seconds": 49})
        qa = metrics.aggregate([late], [], [], [], manifest())["variants"]["A"]["qa03"]
        self.assertEqual(qa["cap_mean"]["value"], 30)
        self.assertEqual(qa["completed_only_mean"]["value"], 31)
        self.assertEqual(qa["late_completed_count"], 1)
        incomplete = trial(cycle={"id": "T-S1-01", "completed": False})
        qa = metrics.aggregate([incomplete], [], [], [], manifest())["variants"]["A"]["qa03"]
        self.assertEqual(qa["cap_mean"]["value"], 30)
        self.assertEqual(qa["completed_only_mean"]["n"], 0)

    def test_memory_peak_is_same_round_sum_and_missing_is_not_zero(self):
        rows = [{"variant": "A", "repeat": 1, "sample_id": s, "process_id": p, "generation": 1,
                 "phys_footprint_bytes": f, "resident_size_bytes": r, "expected_process_ids": [1, 2]}
                for s, p, f, r in [(1, 1, 100, 80), (1, 2, 20, 10),
                                   (2, 1, 10, 8), (2, 2, 200, 180),
                                   (3, 1, 999, 900)]]
        qa = metrics.aggregate([trial()], rows, [], [], manifest())["variants"]["A"]["qa06"]
        self.assertEqual(qa["phys_footprint"]["peak_bytes"], 210)
        self.assertEqual(qa["resident_size_rss"]["peak_bytes"], 188)
        self.assertEqual(qa["phys_footprint"]["per_repeat"][0]["incomplete_samples"], 1)
        self.assertIsNone(qa["windows_commit_score"])

    def test_cost_keeps_judge_outside_via_and_deduplicates_call_updates(self):
        calls = [{"variant": "A", "repeat": 1, "call_id": "semantic", "provider": "Qwen", "role": "INTERPRET", "billed_usd": .02},
                 {"variant": "A", "repeat": 1, "call_id": "semantic", "provider": "Qwen", "role": "INTERPRET", "billed_usd": .02},
                 {"variant": "A", "repeat": 1, "call_id": "judge", "provider": "OpenAI", "role": "JUDGE", "billed_usd": .03},
                 {"variant": "A", "repeat": 1, "call_id": "speech", "provider": "speech_profile", "estimated": True, "cost_usd": .1}]
        result = metrics.aggregate([trial()], [], [], calls, manifest())
        qa = result["variants"]["A"]["qa04"]
        self.assertAlmostEqual(qa["value"], .02)
        self.assertAlmostEqual(qa["simulated_speech_inclusive_value"], .12)
        self.assertAlmostEqual(qa["actual_via_account_usd"], .02)
        self.assertAlmostEqual(qa["judge_account_usd"], .03)
        self.assertEqual(result["provider_spend_usd"], {"Qwen": .02, "OpenAI": .03})

    def test_condition_without_evidence_fails_and_population_must_be_complete(self):
        row = trial()
        row["conditions"]["C1"]["evidence"] = []
        result = metrics.aggregate([row], [], [], [], manifest())
        self.assertEqual(result["variants"]["A"]["qa01"]["passed"], 5)
        with self.assertRaisesRegex(ValueError, "missing trial"):
            metrics.aggregate([row], [], [], [], manifest(repeat_ids=[1, 2]))
        with self.assertRaisesRegex(ValueError, "duplicate trial"):
            metrics.aggregate([row, row], [], [], [], manifest())

    def test_predicted_changed_units_are_not_a_measured_qa05_value(self):
        changes = [{"variant": "A", "task_id": f"M{i:02d}", "changed_units": ["U15"]} for i in range(1, 7)]
        qa = metrics.aggregate([trial()], [], changes, [], manifest())["variants"]["A"]["qa05"]
        self.assertIsNone(qa["value"])
        self.assertFalse(qa["ready"])

    def test_usage_msrp_fallback_is_not_an_invoice_and_unknown_cost_not_zero(self):
        calls = [{"variant": "A", "repeat": 1, "call_id": "semantic", "provider": "qwen", "role": "INTERPRET",
                  "billed_usd": None, "cost_usd": .12, "cost_basis": "PROVIDER_USAGE_X_STANDARD_PRICE"}]
        result = metrics.aggregate([trial()], [], [], calls, manifest())
        qa = result["variants"]["A"]["qa04"]
        self.assertEqual(qa["value"], .12)
        self.assertEqual(qa["evidence_level"], "MEASURED_MODEL")
        self.assertIsNone(qa["invoice_reconciled_via_usd"])
        self.assertEqual(result["provider_spend_usd"], {"qwen": .12})
        calls.append({"variant": "A", "repeat": 1, "call_id": "unknown", "provider": "qwen", "cost_usd": None})
        qa = metrics.aggregate([trial()], [], [], calls, manifest())["variants"]["A"]["qa04"]
        self.assertIsNone(qa["value"])
        self.assertEqual(qa["known_cost_mean_lower_bound"], .12)
        self.assertEqual(qa["missing_usage_or_cost_call_ids"], ["unknown"])
        self.assertIsNone(qa["score"])

    def test_full_p21_fixed_denominators_and_non_applicable_cycles(self):
        workload = json.loads((metrics.QA_DIRECTORY / "03-02-common-workload.json").read_text())
        rows = []
        for repeat in range(1, 6):
            for case in workload["cases"]:
                reactions = [{"id": r["id"], "start_s": 0, "end_s": .05, "first_actual_s": .05, "valid": True}
                             for r in case["reactions"]]
                cycle = {"id": case["cycle"]["id"], "start_s": 0, "valid_end_s": 2, "completed": True} if case.get("cycle") else None
                rows.append(trial(repeat=repeat, case_id=case["case_id"], reactions=reactions, cycle=cycle))
        result = metrics.aggregate(rows, [], [], [], {"variants": ["A"], "strict_coverage": True})["variants"]["A"]
        self.assertEqual(result["qa01"]["denominator"], 630)
        self.assertEqual(result["qa02"]["cap_mean"]["n"], 135)
        self.assertEqual(result["qa03"]["cap_mean"]["n"], 95)
        self.assertEqual(result["qa02"]["guards"]["ordinary_n"], 105)
        self.assertEqual(result["qa02"]["guards"]["burst_n"], 30)
        self.assertEqual(result["qa02"]["guards"]["local_stop_n"], 10)
        self.assertEqual(result["qa01"]["score"], 6)
        self.assertTrue(result["coverage"]["final_scoring_allowed"])
        with self.assertRaisesRegex(ValueError, "unexpected repeat"):
            metrics.aggregate([*rows, trial(repeat=6)], [], [], [], {"variants": ["A"], "strict_coverage": True})

    def test_pilot_values_are_not_final_p21_scores(self):
        result = metrics.aggregate([trial()], [], [], [], manifest())["variants"]["A"]
        self.assertEqual(result["qa01"]["value"], 100)
        self.assertEqual(result["coverage"]["scope"], "PILOT_SUBSET")
        self.assertFalse(result["coverage"]["final_scoring_allowed"])
        self.assertIsNone(result["qa01"]["score"])

    def test_computed_attribution_cannot_replace_missing_actual_endpoint(self):
        row = trial(reactions=[{"id": "I-S1-01-1", "start_s": 0, "end_s": None, "valid": True,
                                "attributed_seconds": .1}],
                    cycle={"id": "T-S1-01", "start_s": 0, "valid_end_s": None, "completed": True,
                           "attributed_seconds": 2})
        result = metrics.aggregate([row], [], [], [], manifest())["variants"]["A"]
        self.assertEqual(result["qa02"]["cap_mean"]["value"], 5)
        self.assertEqual(result["qa02"]["timestamp_missing_count"], 1)
        self.assertEqual(result["qa03"]["cap_mean"]["value"], 30)
        self.assertEqual(result["qa03"]["incomplete_count"], 1)

    def test_memory_primary_is_overall_peak_and_collection_failure_is_missing(self):
        rows = [{"variant": "A", "repeat": r, "sample_id": 1, "process_id": 1, "generation": r,
                 "phys_footprint_bytes": r * 1048576, "resident_size_bytes": r * 2097152,
                 "phase": "IDLE", "success": True} for r in range(1, 6)]
        rows.append({"variant": "A", "repeat": 1, "sample_id": 2, "process_id": 1, "generation": 1,
                     "phys_footprint_bytes": 999 * 1048576, "resident_size_bytes": 999 * 1048576, "success": False})
        result = metrics.aggregate([trial(repeat=r) for r in range(1, 6)], rows, [], [], manifest(repeat_ids=list(range(1, 6))))["variants"]["A"]["qa06"]
        self.assertEqual(result["phys_footprint"]["mean_of_repeat_peaks_mib"], 3)
        self.assertEqual(result["phys_footprint"]["peak_mib"], 5)
        self.assertEqual(result["value"], 5)
        self.assertEqual(result["resident_size_rss"]["mean_of_repeat_peaks_mib"], 6)
        self.assertEqual(result["phys_footprint"]["per_repeat"][0]["incomplete_samples"], 1)

    def test_campaign_mac_score_does_not_claim_windows_commit_equivalence(self):
        workload = json.loads((metrics.QA_DIRECTORY / "03-02-common-workload.json").read_text())
        rows = []
        for repeat in range(1, 6):
            for case in workload["cases"]:
                rows.append(trial(repeat=repeat, case_id=case["case_id"],
                    reactions=[{"id": r["id"], "start_s": 0, "end_s": .05, "first_actual_s": .05, "valid": True}
                               for r in case["reactions"]],
                    cycle={"id": case["cycle"]["id"], "start_s": 0, "valid_end_s": 2, "completed": True} if case.get("cycle") else None))
        memory = [{"variant": "A", "repeat": r, "sample_id": 1, "process_id": 1, "generation": r,
                   "phys_footprint_bytes": 1024 * 1048576, "resident_size_bytes": 1300 * 1048576} for r in range(1, 6)]
        result = metrics.aggregate(rows, memory, [], [], {"variants": ["A"], "strict_coverage": True,
                                                       "memory_scoring_metric": "phys_footprint"})["variants"]["A"]["qa06"]
        self.assertEqual(result["value"], 1024)
        self.assertEqual(result["mac_poc_score"], 6)
        self.assertIsNone(result["windows_commit_score"])
        self.assertFalse(result["metric_basis"]["windows_commit_equivalence"])

    def test_change_count_rejects_mixed_original_snapshots(self):
        jobs = [{"variant": "A", "task_id": f"M{i:02d}", "changed_unit_set": ["U15"],
                 "evidence_level": "MEASURED_CHANGE_EXPERIMENT", "completion_passed": True,
                 "regression_passed": True, "baseline_source_hash": "same" if i < 6 else "different",
                 "patch_verified": True, "responsibility_review_passed": True, "original_baseline_unchanged": True}
                for i in range(1, 7)]
        qa = metrics.aggregate([trial()], [], jobs, [], manifest())["variants"]["A"]["qa05"]
        self.assertFalse(qa["ready"])
        self.assertIsNone(qa["value"])
        self.assertEqual(qa["baseline_source_hashes"], ["different", "same"])


if __name__ == "__main__":
    unittest.main()
