#!/usr/bin/env python3
"""Validate the draft QA registry against its two normative Markdown views."""

from pathlib import Path
import json


ROOT = Path(__file__).resolve().parents[2]
REGISTRY = ROOT / "docs/architecture/08-quality-attributes/qa-catalog-draft.json"
QUALITY_MODEL = ROOT / "docs/architecture/08-quality-attributes/quality-model.md"
SCORING = ROOT / "docs/architecture/11-measurement/scoring-contract.md"


def main() -> int:
    catalog = json.loads(REGISTRY.read_text(encoding="utf-8"))
    active = catalog["active_qa_ids"]
    entries = catalog["quality_attributes"]
    retired = catalog["retired_draft_ids"]
    violations: list[str] = []

    entry_ids = [entry["id"] for entry in entries]
    retired_ids = [entry["id"] for entry in retired]
    if active != entry_ids:
        violations.append("active_qa_ids must match quality_attributes order exactly")
    if len(active) != len(set(active)):
        violations.append("active QA IDs must be unique")
    if set(active) & set(retired_ids):
        violations.append("active and retired QA IDs must be disjoint")

    quality_model = QUALITY_MODEL.read_text(encoding="utf-8")
    scoring = SCORING.read_text(encoding="utf-8")
    for entry in entries:
        qa_id = entry["id"]
        if f"| {qa_id} | {entry['name']} |" not in quality_model:
            violations.append(f"{qa_id} name or row missing from quality-model active table")
        if f"| {qa_id} |" not in scoring:
            violations.append(f"{qa_id} missing from scoring-contract metric table")
    for qa_id in retired_ids:
        if f"| {qa_id} " not in quality_model:
            violations.append(f"{qa_id} missing from quality-model retired table")

    if catalog["status"] != "USER_REVIEW_DRAFT":
        violations.append("catalog status must remain USER_REVIEW_DRAFT until user approval")
    if catalog["asr_selection"] != "NONE_UNASSESSED":
        violations.append("ASR selection changed without a catalog rebaseline")

    if violations:
        print("Draft QA catalog consistency errors:")
        print("\n".join(violations))
        return 1
    print(f"PASS: draft QA registry has {len(active)} active and {len(retired_ids)} retired IDs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
