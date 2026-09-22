#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

EXPECTED_CASE_COUNT = 94
EXPECTED_MEMBERSHIP = {
    "ASR-02": 48,
    "ASR-03": 30,
    "ASR-06": 27,
}
ALLOWED_DISPOSITIONS = {"GOAL_COMPLETED", "HOLD_OR_FAILURE_HANDLING"}
FROZEN_CLARIFICATION_TERMINAL_CASES = {
    "TC-06.2": ("후속 답", "요약"),
    "TC-06.3": ("후속 답", "문서 생성", "위임"),
    "TC-14.5": ("후속 답", "T-MAIL", "취소 처리"),
}


def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _case_row(catalog_text, case_id):
    prefix = f"| **{case_id} "
    matches = [line for line in catalog_text.splitlines() if line.startswith(prefix)]
    if len(matches) != 1:
        raise ValueError(f"expected exactly one 11-B source row for {case_id}, found {len(matches)}")
    return matches[0]


def audit(cases, oracles, obligation_catalog, catalog_text=None, enforce_cardinality=True):
    case_ids = [case.get("id") for case in cases]
    if len(case_ids) != len(set(case_ids)):
        raise ValueError("duplicate TC id in generated cases")
    if set(case_ids) != set(oracles):
        missing = sorted(set(case_ids) - set(oracles))
        extra = sorted(set(oracles) - set(case_ids))
        raise ValueError(f"case/oracle set mismatch; missing_oracle={missing}, extra_oracle={extra}")

    if enforce_cardinality and len(cases) != EXPECTED_CASE_COUNT:
        raise ValueError(f"expected {EXPECTED_CASE_COUNT} canonical TCs, found {len(cases)}")

    if enforce_cardinality:
        for asr, expected in EXPECTED_MEMBERSHIP.items():
            actual = sum(asr in case.get("primary_asrs", []) for case in cases)
            if actual != expected:
                raise ValueError(f"{asr} canonical membership changed: expected {expected}, found {actual}")

    flattened = []
    clarification_terminal_cases = set()
    for case in cases:
        cid = case["id"]
        oracle = oracles[cid]
        disposition = oracle.get("expected_disposition")
        if disposition not in ALLOWED_DISPOSITIONS:
            raise ValueError(f"{cid} has invalid expected_disposition {disposition!r}")

        obligations = oracle.get("obligations", [])
        obligation_ids = [row.get("id") for row in obligations]
        if len(obligation_ids) != len(set(obligation_ids)):
            raise ValueError(f"{cid} has duplicate obligation ids")

        primary_asrs = set(case.get("primary_asrs", []))
        for tracked in ("ASR-02", "ASR-03"):
            if tracked in primary_asrs and not any(tracked in row.get("asrs", []) for row in obligations):
                raise ValueError(f"{cid} is in {tracked} membership but has no tagged obligation")

        scripted_followups = case.get("input", {}).get("scripted_followups", [])
        if "ASR-02" in primary_asrs and scripted_followups:
            clarification_terminal_cases.add(cid)
            positive_categories = {
                row.get("category")
                for row in obligations
                if row.get("kind") == "REQUIRED_PRESENT" and "ASR-02" in row.get("asrs", [])
            }
            if "CLARIFICATION" not in positive_categories:
                raise ValueError(f"{cid} scripted clarification lacks ASR-02 CLARIFICATION obligation")
            if "OUTCOME" not in positive_categories:
                raise ValueError(
                    f"{cid} scripted clarification lacks ASR-02 OUTCOME obligation; "
                    "a correct question alone cannot receive full completion credit"
                )
            if disposition != "GOAL_COMPLETED":
                raise ValueError(f"{cid} scripted clarification must evaluate the final post-followup outcome")

        for row in obligations:
            flattened.append({"tc": cid, **row})

    if enforce_cardinality and clarification_terminal_cases != set(FROZEN_CLARIFICATION_TERMINAL_CASES):
        raise ValueError(
            "ASR-02 scripted clarification membership changed: "
            f"{sorted(clarification_terminal_cases)}"
        )

    if catalog_text is not None:
        for cid, markers in FROZEN_CLARIFICATION_TERMINAL_CASES.items():
            row = _case_row(catalog_text, cid)
            for marker in markers:
                if marker not in row:
                    raise ValueError(f"{cid} 11-B source row lacks terminal-outcome marker {marker!r}")

    catalog_by_id = {}
    for row in obligation_catalog:
        oid = row.get("id")
        if not oid or oid in catalog_by_id:
            raise ValueError(f"duplicate or missing obligation id in catalog: {oid!r}")
        catalog_by_id[oid] = row

    flat_by_id = {row["id"]: row for row in flattened}
    if set(catalog_by_id) != set(flat_by_id):
        missing = sorted(set(flat_by_id) - set(catalog_by_id))
        extra = sorted(set(catalog_by_id) - set(flat_by_id))
        raise ValueError(f"oracle/catalog obligation set mismatch; missing={missing}, extra={extra}")

    for oid, expected in flat_by_id.items():
        actual = catalog_by_id[oid]
        for field in ("tc", "kind", "text", "asrs", "category"):
            if actual.get(field) != expected.get(field):
                raise ValueError(f"{oid} differs between oracle and obligation catalog at {field}")

    return {
        "status": "PASS_TERMINAL_OUTCOME_AUDIT",
        "canonical_cases": len(cases),
        "membership": {
            asr: sum(asr in case.get("primary_asrs", []) for case in cases)
            for asr in EXPECTED_MEMBERSHIP
        },
        "scripted_clarification_terminal_cases": sorted(clarification_terminal_cases),
        "obligations": len(flattened),
        "candidate_execution": "NOT_RUN",
        "scores": None,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    args = parser.parse_args()
    root = args.root.resolve()
    base = root / "benchmark/rebaseline"
    result = audit(
        load_json(base / "cases.json"),
        load_json(base / "oracle/expected.json"),
        load_json(base / "oracle/obligation-catalog.json"),
        (root / "docs/rebaseline/11b-test-case-catalog.md").read_text(encoding="utf-8"),
    )
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
