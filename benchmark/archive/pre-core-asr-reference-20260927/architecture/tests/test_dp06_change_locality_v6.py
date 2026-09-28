#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[3]
CONTRACT = ROOT / "benchmark/architecture/contracts/via-dp-06-change-locality-v6.json"
LEDGER = ROOT / "benchmark/architecture/fixtures/dp06-architecture-elements-v6.json"
PACK = ROOT / "benchmark/architecture/fixtures/dp06-change-locality-pack-v6.json"
RUNNER = ROOT / "benchmark/architecture/run_dp06_change_locality_v6.py"
ANALYZER = ROOT / "benchmark/architecture/analyze_dp06_change_locality_v6.py"


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class Dp06ChangeLocalityV6Tests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
        cls.ledger = json.loads(LEDGER.read_text(encoding="utf-8"))
        cls.pack = json.loads(PACK.read_text(encoding="utf-8"))
        cls.runner = load_module("dp06_change_runner", RUNNER)

    def test_pack_has_exact_twenty_nine_changes(self) -> None:
        expected = self.contract["required_change_ids"]
        actual = {qa_id: [] for qa_id in expected}
        for change in self.pack["changes"]:
            actual[change["qa_id"]].append(change["id"])
        self.assertEqual(actual, expected)
        self.assertEqual(sum(map(len, actual.values())), 29)

    def test_ledger_registration_and_source_anchors_are_valid(self) -> None:
        self.runner.validate_inputs(self.contract, self.ledger, self.pack)

    def test_b_prime_has_same_frozen_architecture_as_b(self) -> None:
        b = self.runner.candidate_elements(self.ledger, "B")
        b_prime = self.runner.candidate_elements(self.ledger, "B_PRIME")
        self.assertEqual(b, b_prime)

    def test_each_change_starts_from_identical_candidate_baseline(self) -> None:
        baseline = self.runner.candidate_elements(self.ledger, "A")
        records = [
            self.runner.apply_change(
                baseline, "A", change, self.ledger["shared_capabilities"]
            )
            for change in self.pack["changes"]
        ]
        self.assertEqual(len({record["before_ledger_sha256"] for record in records}), 1)
        self.assertTrue(all(record["baseline_reset"] for record in records))

    def test_expected_primary_values_follow_frozen_impact_ledger(self) -> None:
        expected = {
            "A": {"QA-21": 25 / 9, "QA-22": 29 / 15, "QA-23": 14 / 5},
            "B": {"QA-21": 25 / 9, "QA-22": 29 / 15, "QA-23": 18 / 5},
            "B_PRIME": {"QA-21": 25 / 9, "QA-22": 29 / 15, "QA-23": 18 / 5},
        }
        for candidate, qa_values in expected.items():
            for qa_id, expected_mean in qa_values.items():
                rows = [
                    change for change in self.pack["changes"] if change["qa_id"] == qa_id
                ]
                counts = [len(change["impacts"][candidate]) for change in rows]
                self.assertAlmostEqual(sum(counts) / len(counts), expected_mean)

    def test_runner_emits_eighty_seven_unique_records(self) -> None:
        keys = []
        for candidate in self.contract["candidates"]:
            baseline = self.runner.candidate_elements(self.ledger, candidate)
            for change in self.pack["changes"]:
                record = self.runner.apply_change(
                    baseline, candidate, change, self.ledger["shared_capabilities"]
                )
                keys.append(record["source_execution_key"])
                self.assertEqual(
                    sum(record["element_type_breakdown"].values()),
                    record["changed_element_count"],
                )
        self.assertEqual(len(keys), 87)
        self.assertEqual(len(set(keys)), 87)


if __name__ == "__main__":
    unittest.main()
