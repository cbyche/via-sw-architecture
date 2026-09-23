import importlib.util
import json
import unittest
from copy import deepcopy
from pathlib import Path

HERE = Path(__file__).resolve().parent
CONTRACT_PATH = HERE / "w01-foreground-strata.json"
MODULE_PATH = HERE / "w04_foreground_contract.py"

spec = importlib.util.spec_from_file_location("w04_foreground_contract", MODULE_PATH)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class W01W04ForegroundContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.contract = json.loads(CONTRACT_PATH.read_text(encoding="utf-8"))

    def test_frozen_contract_validates(self):
        result = module.validate_contract(self.contract)
        self.assertEqual("PASS_W01_W04_FOREGROUND_CONTRACT", result["status"])
        self.assertEqual(module.EXPECTED_CASES, result["w01_cases"])
        self.assertEqual("NOT_RUN", result["representative_metrics"]["W-04"])

    def test_case_membership_cannot_drift(self):
        broken = deepcopy(self.contract)
        broken["w01"]["cases"].pop()
        with self.assertRaisesRegex(ValueError, "case set/order changed"):
            module.validate_contract(broken)

    def test_one_vs_four_is_the_only_background_comparison(self):
        broken = deepcopy(self.contract)
        broken["w04"]["background_task_counts"] = [0, 4]
        with self.assertRaisesRegex(ValueError, "background Task counts changed"):
            module.validate_contract(broken)

    def test_hosted_ci_cannot_become_metric_evidence(self):
        broken = deepcopy(self.contract)
        broken["w04"]["metric_eligible_in_hosted_ci"] = True
        with self.assertRaisesRegex(ValueError, "metric-ineligible"):
            module.validate_contract(broken)

    def test_clarification_latency_does_not_replace_terminal_completion(self):
        broken = deepcopy(self.contract)
        row = next(case for case in broken["w01"]["cases"] if case["case_id"] == "TC-06.2")
        row["output_contract"] = "first question"
        with self.assertRaisesRegex(ValueError, "clarification"):
            module.validate_contract(broken)


if __name__ == "__main__":
    unittest.main()
