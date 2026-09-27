#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[3]
CONTRACT = ROOT / "benchmark/architecture/contracts/via-dp-06-evaluation-v5.json"
ANALYZER = ROOT / "benchmark/architecture/analyze_dp06_evaluation_v5.py"


def load_module():
    spec = importlib.util.spec_from_file_location("dp06_v5_analyzer", ANALYZER)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class Dp06EvaluationV5Tests(unittest.TestCase):
    def test_contract_has_breadth_and_repeated_latency_strata(self) -> None:
        contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
        self.assertEqual(contract["contract_id"], "VIA-DP-06-EVALUATION-v5")
        self.assertEqual(contract["strata"]["correctness_breadth"]["cases"], "ALL_24")
        latency = contract["strata"]["latency_repetition"]
        self.assertEqual(latency["warmup_trials_per_case"], 2)
        self.assertEqual(latency["scored_trials_per_case"], 20)
        self.assertEqual(set(latency["qa_mapping"].values()), {"QA-01", "QA-02", "QA-05"})

    def test_all_nineteen_qas_have_dispositions(self) -> None:
        contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
        self.assertEqual(len(contract["qa_applicability"]), 19)
        self.assertTrue(all(
            value.startswith(("APPLICABLE", "N/A", "BLOCKED"))
            for value in contract["qa_applicability"].values()
        ))

    def test_nearest_rank_p95_uses_tail_sample(self) -> None:
        analyzer = load_module()
        values = [float(index) for index in range(1, 21)]
        self.assertEqual(analyzer.nearest_rank(values, 0.95), 19.0)

    def test_runner_failure_is_preserved_as_timeout(self) -> None:
        analyzer = load_module()
        sample, censored = analyzer.latency_sample(
            {"runner_failure": "boom"}, "QA-01", 60000.0
        )
        self.assertEqual(sample, 60000.0)
        self.assertTrue(censored)


if __name__ == "__main__":
    unittest.main()
