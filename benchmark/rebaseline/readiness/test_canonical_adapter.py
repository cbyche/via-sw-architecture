import unittest

from canonical_adapter import build_plan


class CanonicalAdapterTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.plan = build_plan()
        cls.by_id = {row["case_id"]: row for row in cls.plan["cases"]}

    def test_all_94_reviewed_cases_have_one_plan(self):
        self.assertEqual(self.plan["canonical_case_count"], 94)
        self.assertEqual(len(self.by_id), 94)
        self.assertTrue(all(row["responsibilities"] for row in self.plan["cases"]))

    def test_adapter_never_fabricates_candidate_results(self):
        self.assertEqual(self.plan["candidate_observations"], "NOT_RUN")
        self.assertEqual(self.plan["scoring"], "NOT_RUN")
        self.assertTrue(
            all(row["candidate_observation"] == "NOT_RUN" for row in self.plan["cases"])
        )
        self.assertTrue(all(row["score"] is None for row in self.plan["cases"]))

    def test_restart_case_requires_real_exec_task_and_agent_slices(self):
        row = self.by_id["TC-18.6"]
        self.assertEqual(row["patch_key"], "restart")
        self.assertTrue(
            {"EXEC_HOST", "TASK_RUNTIME", "AGENT_FIXTURE"}.issubset(
                set(row["responsibilities"])
            )
        )

    def test_multitask_pending_interaction_keeps_policy_boundary(self):
        row = self.by_id["TC-14.3"]
        self.assertIn("POLICY_SERVICE", row["responsibilities"])
        self.assertIn("TASK_RUNTIME", row["responsibilities"])

    def test_grounded_voice_cases_do_not_hide_missing_runtime(self):
        row = self.by_id["TC-04.2"]
        self.assertIn("VOICE_RUNTIME", row["runtime_blockers"])
        self.assertIn("PRESENTATION", row["runtime_blockers"])
        self.assertEqual(row["adapter_readiness"], "BLOCKED_RUNTIME_GAP")

    def test_actual_model_cases_are_dependency_pending_not_measured(self):
        rows = [
            row
            for row in self.plan["cases"]
            if "IR_MODEL" in row["responsibilities"]
            and not row["runtime_blockers"]
        ]
        self.assertTrue(rows)
        self.assertTrue(
            all(
                row["adapter_readiness"] == "EXECUTION_DEPENDENCY_PENDING"
                for row in rows
            )
        )


if __name__ == "__main__":
    unittest.main()
