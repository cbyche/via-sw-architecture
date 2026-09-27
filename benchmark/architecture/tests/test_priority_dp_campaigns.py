from __future__ import annotations

import json
from pathlib import Path
import sys
import tempfile
import unittest


REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO))

from prototypes.candidates.reference_harness.candidates import run_candidate


QA_IDS = {
    "QA-01", "QA-02", "QA-03", "QA-04", "QA-05",
    "QA-11", "QA-12", "QA-13", "QA-14", "QA-15",
    "QA-21", "QA-22", "QA-23", "QA-31", "QA-32", "QA-41",
    "QA-51", "QA-61", "QA-62",
}


class PriorityDpCampaignTests(unittest.TestCase):
    def contracts(self):
        root = REPO / "benchmark" / "architecture" / "contracts"
        for number in ("02", "05", "09", "12", "13"):
            version = "v2" if number == "02" else "v1"
            yield json.loads((root / f"via-dp-{number}-evaluation-{version}.json").read_text())

    def test_contracts_have_exact_active_qa_catalog(self):
        for contract in self.contracts():
            self.assertEqual(set(contract["qa_map"]), QA_IDS)
            self.assertEqual(contract["candidates"], ["A", "B"])
            self.assertTrue(contract["cases"])

    def test_every_candidate_executes_every_frozen_case(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for contract in self.contracts():
                for candidate in contract["candidates"]:
                    for case in contract["cases"]:
                        result = run_candidate(
                            contract["dp"], candidate, case,
                            root / contract["dp"] / candidate / case["id"],
                        )
                        self.assertEqual(result["candidate"], candidate)
                        self.assertEqual(result["case_id"], case["id"])
                        self.assertIn("strict_success", result)
                        self.assertIn("trace_complete", result)

    def test_mutually_exclusive_paths_emit_distinct_trace_events(self):
        by_dp = {contract["dp"]: contract for contract in self.contracts()}
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            expected = {
                "VIA-DP-02": ({"joint_commit_completed", "joint_commit_recovered"}, {"relation_reconciliation_completed"}),
                "VIA-DP-05": ({"context_manifest_committed"}, {"context_capability_issued"}),
                "VIA-DP-09": ({"edge_semantic_normalized"}, {"core_lifecycle_interpreted"}),
                "VIA-DP-12": ({"response_withheld", "evidence_durable"}, {"evidence_enqueued"}),
                "VIA-DP-13": ({"control_execution_started"}, {"control_execution_started"}),
            }
            for dp, contract in by_dp.items():
                case = contract["cases"][0]
                traces = {}
                for candidate in ("A", "B"):
                    result = run_candidate(dp, candidate, case, root / dp / candidate)
                    traces[candidate] = {event["event"] for event in result["trace"]}
                self.assertTrue(traces["A"].intersection(expected[dp][0]))
                self.assertTrue(traces["B"].intersection(expected[dp][1]))
                if dp != "VIA-DP-13":
                    self.assertNotEqual(traces["A"], traces["B"])


if __name__ == "__main__":
    unittest.main()
