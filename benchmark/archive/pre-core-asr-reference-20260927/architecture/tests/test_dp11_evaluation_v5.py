#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
CONTRACT = ROOT / "benchmark/architecture/contracts/via-dp-11-evaluation-v5.json"
ANALYZER = ROOT / "benchmark/architecture/analyze_dp11_evaluation_v5.py"


def load_analyzer():
    spec = importlib.util.spec_from_file_location("dp11_v5_analyzer", ANALYZER)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class Dp11EvaluationV5Tests(unittest.TestCase):
    def test_contract_changes_only_process_boundary(self) -> None:
        contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
        self.assertEqual(contract["contract_id"], "VIA-DP-11-EVALUATION-v5")
        self.assertEqual(contract["candidates"], ["isolated_worker", "same_process"])
        self.assertIn("only the OS process boundary", contract["fixed_context"]["changed_axis"])

    def test_repeated_physical_latency_and_fault_strata_are_frozen(self) -> None:
        contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
        latency = contract["strata"]["latency_repetition"]
        self.assertEqual(latency["scored_trials_per_case_per_candidate"], 20)
        self.assertEqual(set(latency["case_qa_mapping"].values()), {"QA-01", "QA-03", "QA-05"})
        self.assertTrue(all(value >= 10 for value in contract["strata"]["faults"].values()))

    def test_all_nineteen_qas_have_fail_closed_dispositions(self) -> None:
        contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
        self.assertEqual(len(contract["qa_applicability"]), 19)
        self.assertTrue(all(
            value.startswith(("APPLICABLE", "N/A", "BLOCKED"))
            for value in contract["qa_applicability"].values()
        ))

    def test_nearest_rank_p95(self) -> None:
        analyzer = load_analyzer()
        self.assertEqual(analyzer.p95([float(value) for value in range(1, 21)]), 19.0)


if __name__ == "__main__":
    unittest.main()
