#!/usr/bin/env python3
"""Static and evaluator qualification for the single active DP-06 campaign."""

from __future__ import annotations

import json
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
CONTRACT = ROOT / "benchmark/architecture/contracts/via-dp-06-evaluation-v4.json"
RUNNER = ROOT / "benchmark/architecture/run_dp06_evaluation.py"
ANALYZER = ROOT / "benchmark/architecture/analyze_dp06_evaluation.py"


class Dp06EvaluationV4Tests(unittest.TestCase):
    def test_contract_is_self_contained_and_names_only_v4_inputs(self) -> None:
        contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
        self.assertEqual(contract["contract_id"], "VIA-DP-06-EVALUATION-v4")
        self.assertNotIn("base_contract", contract)
        self.assertTrue(contract["substantive_response_oracle"])
        self.assertEqual(contract["physical_endpoint_timeout_ms"]["QA-01"], 60000)
        self.assertEqual(contract["latency_case_overrides"]["TC-E02"], "QA-02")

    def test_all_nineteen_qas_have_an_explicit_measurement_disposition(self) -> None:
        contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
        dispositions = contract["qa_applicability"]
        self.assertEqual(
            set(dispositions),
            {
                "QA-01", "QA-02", "QA-03", "QA-04", "QA-05",
                "QA-11", "QA-12", "QA-13", "QA-14", "QA-15",
                "QA-21", "QA-22", "QA-23", "QA-31", "QA-32", "QA-41",
                "QA-51", "QA-61", "QA-62",
            },
        )
        self.assertTrue(all(
            state.startswith(("APPLICABLE", "N/A", "BLOCKED"))
            for state in dispositions.values()
        ))

    def test_runner_and_analyzer_have_no_superseded_contract_reference(self) -> None:
        text = RUNNER.read_text(encoding="utf-8") + ANALYZER.read_text(encoding="utf-8")
        for forbidden in ("reference-v1", "reference-v2", "reference-v3", "evaluation-v3", "pilot-v"):
            self.assertNotIn(forbidden, text)


if __name__ == "__main__":
    unittest.main()
