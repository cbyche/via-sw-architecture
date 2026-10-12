"""Exercise real temporary source deltas, isolation, and honest zero changes."""
import importlib.util
from pathlib import Path
import tempfile
import shutil
import unittest
from unittest.mock import patch as mock_patch


MODULE = Path(__file__).resolve().parents[1] / "change_experiments.py"
spec = importlib.util.spec_from_file_location("poc_changes_under_test", MODULE)
changes = importlib.util.module_from_spec(spec)
spec.loader.exec_module(changes)


def passing(task):
    spec = changes.TASKS[task]
    return {name: {"passed": True, "evidence": ["test-fixture-observed-behaviour"]}
            for name in spec["completion"] + spec["regressions"]}


def reviewers(source, record):
    units = sorted(record.get("responsibility_claims", {}))
    return [{"reviewer_id": name, "accepted": True, "changed_unit_set": units,
             "evidence": ["unit-test-review-fixture"]} for name in ("fixture-reviewer-1", "fixture-reviewer-2")]


class ChangeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.base = Path(self.temp.name) / "baseline"
        self.base.mkdir()
        (self.base / "adapter.py").write_text("wire_key = 'old'\n")
        self.output = Path(self.temp.name) / "jobs"

    def tearDown(self):
        self.temp.cleanup()

    def test_specs_are_twelve_unrun_jobs_without_predicted_counts(self):
        jobs = changes.prepare_jobs()
        self.assertEqual(len(jobs), 12)
        self.assertEqual(len({j["job_id"] for j in jobs}), 12)
        self.assertTrue(all(j["changed_unit_set"] is None and j["status"] == "NOT_RUN" for j in jobs))

    def test_independent_real_diff_and_regression_evidence(self):
        def patch(source):
            self.assertEqual((source / "adapter.py").read_text(), "wire_key = 'old'\n")
            (source / "adapter.py").write_text("wire_key = 'new'\n")
            return {"responsibility_claims": {"U15": {
                "reason": "provider wire envelope differs", "before_contract": "old-key envelope",
                "after_contract": "new-key envelope with same normalized port", "changed_paths": ["adapter.py"],
                "functional_evidence": ["new envelope round-trip fixture"]}}}
        result = changes.run_change_job(self.base, self.output, "A", "M01", patch,
                                        lambda source: passing("M01"), reviewers)
        self.assertEqual(result["status"], "COMPLETED")
        self.assertEqual(result["count"], 1)
        self.assertEqual(result["changed_unit_set"], ["U15"])
        self.assertEqual((self.base / "adapter.py").read_text(), "wire_key = 'old'\n")
        self.assertIn("+wire_key = 'new'", Path(result["patch_ref"]).read_text())
        # A second independent job starts from exactly the same baseline.
        other = changes.run_change_job(self.base, self.output, "B", "M01", patch,
                                       lambda source: passing("M01"), reviewers)
        self.assertEqual(result["baseline_source_hash"], other["baseline_source_hash"])
        with self.assertRaises(FileExistsError):
            changes.run_change_job(self.base, self.output, "A", "M01", patch, lambda source: passing("M01"), reviewers)

    def test_zero_change_requires_complete_baseline_audit(self):
        result = changes.run_change_job(self.base, self.output, "A", "M06", lambda source: {},
                                        lambda source: passing("M06"), reviewers,
                                        lambda source: {"supports_requirement": True, "evidence": ["all-use-fences-already-tested"]})
        self.assertEqual(result["status"], "COMPLETED")
        self.assertEqual(result["count"], 0)
        self.assertEqual(result["changed_paths"], [])
        failed = changes.run_change_job(self.base, self.output, "B", "M06", lambda source: {},
                                        lambda source: passing("M06"), reviewers,
                                        lambda source: {"supports_requirement": False, "evidence": []})
        self.assertEqual(failed["status"], "FAILED")
        self.assertIsNone(failed["count"])

    def test_count_claim_must_have_actual_diff_and_functional_reason(self):
        def patch(source):
            (source / "adapter.py").write_text("wire_key = 'new'\n")
            return {"responsibility_claims": {"U15": {
                "reason": "changed", "before_contract": "old", "after_contract": "new",
                "changed_paths": ["file-not-changed.py"], "functional_evidence": ["test"]}}}
        result = changes.run_change_job(self.base, self.output, "A", "M01", patch,
                                        lambda source: passing("M01"), reviewers)
        self.assertEqual(result["status"], "FAILED")
        self.assertIn("actual source diff", result["error"])
        self.assertTrue(Path(result["result_ref"]).exists())

    def test_regression_failure_preserved_and_does_not_get_small_count_success(self):
        checks = passing("M04")
        checks["SEND_zero"] = {"passed": False, "evidence": ["unexpected send effect"]}
        result = changes.run_change_job(self.base, self.output, "A", "M04", lambda source: {},
                                        lambda source: checks, reviewers,
                                        lambda source: {"supports_requirement": True, "evidence": ["baseline-test"]})
        self.assertEqual(result["status"], "FAILED")
        self.assertIsNone(result["count"])
        self.assertFalse(result["completion_passed"])

    def test_u18_cannot_double_count_private_cache_invalidation(self):
        claim = {"U18": {"reason": "cache changed", "before_contract": "old", "after_contract": "new",
                           "changed_paths": ["cache.py"], "functional_evidence": ["test"]}}
        with self.assertRaisesRegex(ValueError, "distinct cross-owner"):
            changes.validate_responsibility_review(claim, [], ["cache.py"])

    def test_baseline_audit_cannot_modify_source_before_patch(self):
        def audit(source):
            (source / "adapter.py").write_text("wire_key = 'new'\n")
            return {"supports_requirement": True, "evidence": ["claimed-test"]}
        result = changes.run_change_job(self.base, self.output, "A", "M06", lambda source: {},
                                        lambda source: passing("M06"), reviewers, audit)
        self.assertEqual(result["status"], "FAILED")
        self.assertIn("baseline audit modified", result["error"])
        self.assertEqual((self.base / "adapter.py").read_text(), "wire_key = 'old'\n")

    def test_regression_cannot_test_a_different_delta(self):
        def regression(source):
            (source / "adapter.py").write_text("wire_key = 'different-after-patch'\n")
            return passing("M06")
        result = changes.run_change_job(self.base, self.output, "A", "M06", lambda source: {}, regression, reviewers,
                                        lambda source: {"supports_requirement": True, "evidence": ["baseline-test"]})
        self.assertEqual(result["status"], "FAILED")
        self.assertIn("regression runner modified", result["error"])

    def test_all_eighteen_responsibilities_have_review_records(self):
        result = changes.run_change_job(self.base, self.output, "A", "M06", lambda source: {},
                                        lambda source: passing("M06"), reviewers,
                                        lambda source: {"supports_requirement": True, "evidence": ["all-use-fences-tested"]})
        self.assertEqual(set(result["responsibility_assessment"]), set(changes.RESPONSIBILITIES))
        self.assertTrue(all(row["status"] == "UNCHANGED" and row["reason"] for row in result["responsibility_assessment"].values()))

    def test_actual_alternate_adapter_requirements_fail_before_and_pass_after_patch(self):
        original = Path(__file__).resolve().parents[4] / "prototypes/candidates/dp41_45_poc/shared/adapters.py"
        for task in ("M01", "M02", "M03", "M05"):
            source = Path(self.temp.name) / f"source-{task}"
            (source / "shared").mkdir(parents=True)
            shutil.copy2(original, source / "shared/adapters.py")
            with self.assertRaises((AssertionError, KeyError, AttributeError)):
                changes.execute_adapter_requirement(source, task)
            claims = changes.patch_adapter_requirement(source, task)
            result = changes.execute_adapter_requirement(source, task)
            self.assertTrue(result["passed"])
            self.assertTrue(result["observations"])
            self.assertEqual(result["evidence_level"], "SCRIPTED_FUNCTIONAL_REGRESSION")
            self.assertTrue(claims["responsibility_claims"])

    def test_final_review_preserves_first_execution_and_blocks_tampering(self):
        original = Path(__file__).resolve().parents[4] / "prototypes/candidates/dp41_45_poc/shared/adapters.py"
        (self.base / "shared").mkdir()
        shutil.copy2(original, self.base / "shared/adapters.py")
        record = changes.run_change_job(self.base, self.output, "A", "M01",
            lambda source: changes.patch_adapter_requirement(source, "M01"),
            lambda source: passing("M01") if changes.execute_adapter_requirement(source, "M01")["passed"] else {},
            lambda source, record: [])
        self.assertEqual(record["status"], "AWAITING_RESPONSIBILITY_REVIEW")
        preserved = Path(record["result_ref"]).read_bytes()
        reviewed = changes.finalize_responsibility_review(record["result_ref"], reviewers(None, record))
        self.assertEqual(reviewed["status"], "COMPLETED")
        self.assertEqual(reviewed["count"], 1)
        self.assertEqual(Path(record["result_ref"]).read_bytes(), preserved)
        with self.assertRaises(FileExistsError):
            changes.finalize_responsibility_review(record["result_ref"], reviewers(None, record))

    def copy_candidate_package(self, name):
        original = Path(__file__).resolve().parents[4] / "prototypes/candidates/dp41_45_poc"
        source = Path(self.temp.name) / name
        (source / "prototypes/candidates").mkdir(parents=True)
        shutil.copytree(original, source / "prototypes/candidates/dp41_45_poc",
                        ignore=shutil.ignore_patterns("__pycache__"))
        return source

    def test_m04_requires_real_failure_chain_and_independent_actual_delta(self):
        for variant in ("A", "B"):
            source = self.copy_candidate_package("failure-" + variant)
            before = changes.snapshot_digest(changes.source_snapshot(source))
            baseline = changes.execute_runtime_requirement(source, variant, "M04", audit=True)
            self.assertFalse(baseline["failure_continues_draft_once"]["passed"])
            self.assertEqual(before, changes.snapshot_digest(changes.source_snapshot(source)))
            claims = changes.patch_adapter_requirement(source, "M04", variant)["responsibility_claims"]
            self.assertEqual(set(claims), {"U01", "U03", "U07", "U08"} | ({"U02"} if variant == "B" else set()))
            actual = changes.execute_runtime_requirement(source, variant, "M04")
            self.assertTrue(all(actual[name]["passed"] for name in changes.TASKS["M04"]["completion"]
                                + [n for n in changes.TASKS["M04"]["regressions"] if n != "P21"]))

    def test_m06_zero_requires_pending_model_publication_derived_and_restore_probe(self):
        source = self.copy_candidate_package("use-fence")
        before = changes.snapshot_digest(changes.source_snapshot(source))
        checks = changes.execute_runtime_requirement(source, "A", "M06", audit=True)
        required = changes.TASKS["M06"]["completion"] + [n for n in changes.TASKS["M06"]["regressions"] if n != "P21"]
        self.assertTrue(all(checks[name]["passed"] for name in required))
        self.assertEqual(before, changes.snapshot_digest(changes.source_snapshot(source)))

    def test_unaccepted_review_is_preserved_and_can_be_reconciled(self):
        record = changes.run_change_job(self.base, self.output, "A", "M06", lambda source: {},
                                        lambda source: passing("M06"), lambda source, record: [],
                                        lambda source: {"supports_requirement": True, "evidence": ["baseline-test"]})
        pending = changes.finalize_responsibility_review(record["result_ref"], [])
        self.assertEqual(pending["status"], "AWAITING_RESPONSIBILITY_REVIEW")
        self.assertTrue(Path(pending["review_attempt_ref"]).is_file())
        reviewed = changes.finalize_responsibility_review(record["result_ref"], reviewers(None, record))
        self.assertEqual(reviewed["count"], 0)

    def test_callback_audit_keyword_and_evaluator_hashes_are_recorded(self):
        calls = []
        def replay(source, variant, task, *, audit=False):
            calls.append({"variant": variant, "task": task, "audit": audit})
            return passing(task)
        with mock_patch.object(changes, "prepare_jobs", return_value=[{"variant": "A", "task_id": "M06"}]), \
             mock_patch.object(changes, "execute_adapter_requirement", return_value={"passed": True}), \
             mock_patch.object(changes, "execute_runtime_requirement", return_value=passing("M06")):
            rows = changes.run_dp41_change_jobs(self.base, self.output, replay)
        self.assertEqual(calls, [{"variant": "A", "task": "M06", "audit": True},
                                 {"variant": "A", "task": "M06", "audit": False}])
        self.assertEqual(rows[0]["status"], "AWAITING_RESPONSIBILITY_REVIEW")
        self.assertTrue(rows[0]["baseline_support_audit_passed"])
        self.assertTrue(all(len(h) == 64 for h in rows[0]["evaluator_source_hashes"].values()))

    def test_legacy_three_argument_callback_still_runs_baseline_and_after_checks(self):
        calls = []
        def replay(source, variant, task):
            calls.append(task)
            return passing(task)
        with mock_patch.object(changes, "prepare_jobs", return_value=[{"variant": "A", "task_id": "M06"}]), \
             mock_patch.object(changes, "execute_adapter_requirement", return_value={"passed": True}), \
             mock_patch.object(changes, "execute_runtime_requirement", return_value=passing("M06")):
            rows = changes.run_dp41_change_jobs(self.base, self.output, replay)
        self.assertEqual(calls, ["M06", "M06"])
        self.assertEqual(rows[0]["status"], "AWAITING_RESPONSIBILITY_REVIEW")


if __name__ == "__main__":
    unittest.main()
