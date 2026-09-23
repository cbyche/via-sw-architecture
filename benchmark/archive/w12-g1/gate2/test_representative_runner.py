import unittest

from representative_runner import assemble_w09, assemble_w10
from score import load_baseline


class RepresentativeRunnerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.baseline = load_baseline()

    def test_w09_combines_four_task_and_two_exec_strata(self):
        fingerprint = "a" * 64
        whole = {
            "profile": "frozen",
            "freeze_fingerprint": fingerprint,
            "w09_strata_metric_eligible": True,
            "candidates": [
                {"task_candidate": candidate, "strata": [{"p95_recovery_ms": value}] * 4}
                for candidate, value in (("shared", 600), ("per_task", 900))
            ],
        }
        fatal = {
            "profile": "frozen",
            "freeze_fingerprint": fingerprint,
            "w09_strata_metric_eligible": True,
            "candidates": [
                {"exec_candidate": candidate, "strata": [{"p95_recovery_ms": value}] * 2}
                for candidate, value in (("shared", 1200), ("isolated", 300))
            ],
        }
        rows = {row["configuration_id"]: row for row in assemble_w09(whole, fatal, self.baseline)["rows"]}
        self.assertEqual(16, len(rows))
        self.assertEqual(800, rows["AAAA"]["metric_value"])
        self.assertEqual(1000, rows["ABAA"]["metric_value"])
        self.assertEqual(500, rows["AAAB"]["metric_value"])
        self.assertEqual(700, rows["ABAB"]["metric_value"])

    def test_w10_maps_exec_choice_to_all_configurations(self):
        cells = [{"pass": True}] * 24
        fatal = [{"pass": True}] * 4
        payload = {
            "profile": "frozen",
            "freeze_fingerprint": "b" * 64,
            "w10_metric_eligible": True,
            "candidates": [
                {"exec_candidate": "shared", "passed_cells": 27, "controller_retention_pct": 27 / 28 * 100,
                 "external_cells": cells, "fatal_cells": fatal},
                {"exec_candidate": "isolated", "passed_cells": 28, "controller_retention_pct": 100,
                 "external_cells": cells, "fatal_cells": fatal},
            ],
        }
        rows = {row["configuration_id"]: row for row in assemble_w10(payload, self.baseline)["rows"]}
        self.assertEqual(16, len(rows))
        self.assertEqual(27, rows["AAAA"]["passed_cells"])
        self.assertEqual(28, rows["AAAB"]["passed_cells"])
        self.assertEqual(5, rows["AAAB"]["score"])


if __name__ == "__main__":
    unittest.main()
