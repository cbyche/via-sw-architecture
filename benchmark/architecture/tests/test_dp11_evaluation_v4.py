from __future__ import annotations

import json
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
CONTRACT = ROOT / "benchmark/architecture/contracts/via-dp-11-evaluation-v4.json"


class Dp11EvaluationV4Tests(unittest.TestCase):
    def test_all_active_qas_have_a_fail_closed_disposition(self) -> None:
        contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
        expected = {
            "QA-01", "QA-02", "QA-03", "QA-04", "QA-05",
            "QA-11", "QA-12", "QA-13", "QA-14", "QA-15",
            "QA-21", "QA-22", "QA-23", "QA-31", "QA-32", "QA-41",
            "QA-51", "QA-61", "QA-62",
        }
        dispositions = contract["qa_applicability"]
        self.assertEqual(set(dispositions), expected)
        self.assertTrue(all(
            value.startswith(("APPLICABLE", "N/A", "BLOCKED"))
            for value in dispositions.values()
        ))

    def test_only_executable_active_endpoints_are_scored(self) -> None:
        contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
        scored = {
            qa_id for qa_id, value in contract["qa_applicability"].items()
            if value.startswith("APPLICABLE")
        }
        self.assertEqual(scored, {"QA-31", "QA-32", "QA-41", "QA-61", "QA-62"})


if __name__ == "__main__":
    unittest.main()
