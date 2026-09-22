import importlib.util
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
MODULE_PATH = HERE / "terminal_outcome_audit.py"

spec = importlib.util.spec_from_file_location("terminal_outcome_audit", MODULE_PATH)
audit_mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit_mod)


def mini_case(obligations):
    case = {
        "id": "TC-06.2",
        "primary_asrs": ["ASR-02"],
        "input": {"scripted_followups": ["첫 번째 거"]},
    }
    oracle = {
        "TC-06.2": {
            "expected_disposition": "GOAL_COMPLETED",
            "obligations": obligations,
        }
    }
    catalog = [{"tc": "TC-06.2", **row} for row in obligations]
    return [case], oracle, catalog


class TerminalOutcomeAuditTests(unittest.TestCase):
    def test_clarification_only_cannot_receive_full_completion_credit(self):
        obligations = [
            {
                "id": "TC-06.2-O01",
                "kind": "REQUIRED_PRESENT",
                "text": "후속 답 binding",
                "asrs": ["ASR-02"],
                "category": "CLARIFICATION",
                "result": None,
            }
        ]
        cases, oracles, catalog = mini_case(obligations)
        with self.assertRaisesRegex(ValueError, "lacks ASR-02 OUTCOME"):
            audit_mod.audit(cases, oracles, catalog, enforce_cardinality=False)

    def test_clarification_plus_outcome_is_accepted(self):
        obligations = [
            {
                "id": "TC-06.2-O01",
                "kind": "REQUIRED_PRESENT",
                "text": "후속 답 binding",
                "asrs": ["ASR-02"],
                "category": "CLARIFICATION",
                "result": None,
            },
            {
                "id": "TC-06.2-O02",
                "kind": "REQUIRED_PRESENT",
                "text": "선택된 견적서 요약 결과 전달",
                "asrs": ["ASR-02"],
                "category": "OUTCOME",
                "result": None,
            },
        ]
        cases, oracles, catalog = mini_case(obligations)
        result = audit_mod.audit(cases, oracles, catalog, enforce_cardinality=False)
        self.assertEqual("PASS_TERMINAL_OUTCOME_AUDIT", result["status"])
        self.assertEqual(["TC-06.2"], result["scripted_clarification_terminal_cases"])

    def test_oracle_and_flat_catalog_must_match(self):
        obligations = [
            {
                "id": "TC-06.2-O01",
                "kind": "REQUIRED_PRESENT",
                "text": "후속 답 binding",
                "asrs": ["ASR-02"],
                "category": "CLARIFICATION",
                "result": None,
            },
            {
                "id": "TC-06.2-O02",
                "kind": "REQUIRED_PRESENT",
                "text": "선택된 견적서 요약 결과 전달",
                "asrs": ["ASR-02"],
                "category": "OUTCOME",
                "result": None,
            },
        ]
        cases, oracles, catalog = mini_case(obligations)
        catalog[1] = dict(catalog[1], text="different")
        with self.assertRaisesRegex(ValueError, "differs between oracle and obligation catalog"):
            audit_mod.audit(cases, oracles, catalog, enforce_cardinality=False)


if __name__ == "__main__":
    unittest.main()
