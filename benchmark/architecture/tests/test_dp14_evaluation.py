#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
CONTRACT = ROOT / "benchmark/architecture/contracts/via-dp-14-evaluation-v1.json"
ANALYZER = ROOT / "benchmark/architecture/analyze_dp14_evaluation.py"


class Dp14EvaluationTests(unittest.TestCase):
    def test_contract_freezes_mutually_exclusive_task_writers(self) -> None:
        contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
        self.assertEqual(contract["candidates"], ["shared_service", "per_task_supervisor"])
        self.assertIn("only Task state transition ownership", contract["fixed_context"]["changed_axis"])

    def test_contract_has_repetition_and_all_qa_dispositions(self) -> None:
        contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
        self.assertEqual(contract["repetitions"]["latency_scored_per_case_per_candidate"], 20)
        self.assertEqual(contract["repetitions"]["recovery_faults_per_candidate"], 10)
        self.assertEqual(len(contract["qa_applicability"]), 19)
        self.assertTrue(all(value.startswith(("APPLICABLE", "N/A", "BLOCKED")) for value in contract["qa_applicability"].values()))

    def test_p95_is_nearest_rank(self) -> None:
        spec = importlib.util.spec_from_file_location("dp14_analyzer", ANALYZER)
        assert spec and spec.loader
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        self.assertEqual(module.p95([float(value) for value in range(1, 21)]), 19.0)


if __name__ == "__main__":
    unittest.main()
