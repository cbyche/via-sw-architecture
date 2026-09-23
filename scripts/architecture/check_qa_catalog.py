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
    migrations = catalog["legacy_id_migration"]
    legacy_retired = catalog["legacy_retired_definitions"]
    violations: list[str] = []
    category_bounds = {
        "Responsiveness": range(1, 10),
        "Correctness & Continuity": range(11, 20),
        "Modifiability": range(21, 30),
        "Reliability & Availability": range(31, 40),
        "Resource Efficiency": range(41, 50),
        "Privacy & Security": range(51, 60),
    }

    entry_ids = [entry["id"] for entry in entries]
    if active != entry_ids:
        violations.append("active_qa_ids must match quality_attributes order exactly")
    if len(active) != len(set(active)):
        violations.append("active QA IDs must be unique")

    migration_old_ids = [entry["old_id"] for entry in migrations]
    migration_new_ids = [entry["new_id"] for entry in migrations]
    if len(migration_old_ids) != len(set(migration_old_ids)):
        violations.append("legacy migration old IDs must be unique")
    if not set(migration_new_ids).issubset(set(active)):
        violations.append("legacy migration new IDs must all be active")

    quality_model = QUALITY_MODEL.read_text(encoding="utf-8")
    scoring = SCORING.read_text(encoding="utf-8")
    for entry in entries:
        qa_id = entry["id"]
        metric_keys = [key for key in entry if key in {"metric", "metrics"}]
        if metric_keys != ["metric"] or not isinstance(entry["metric"], str):
            violations.append(f"{qa_id} must define exactly one string metric")
        category = entry.get("category")
        qa_number = int(qa_id.removeprefix("QA-"))
        if category not in category_bounds or qa_number not in category_bounds[category]:
            violations.append(f"{qa_id} is outside its declared category range: {category}")
        if f"| {qa_id} | {entry['name']} |" not in quality_model:
            violations.append(f"{qa_id} name or row missing from quality-model active table")
        if f"| {qa_id} |" not in scoring:
            violations.append(f"{qa_id} missing from scoring-contract metric table")
    for entry in migrations:
        row = f"| {entry['old_id']} | {entry['new_id']} |"
        if row not in quality_model:
            violations.append(f"migration row missing from quality-model: {row}")

    for entry in legacy_retired:
        if entry["old_id"] not in {"QA-04", "QA-06", "QA-10", "QA-12"}:
            violations.append(f"unexpected legacy retired ID: {entry['old_id']}")

    if catalog["status"] != "USER_REVIEW_DRAFT":
        violations.append("catalog status must remain USER_REVIEW_DRAFT until user approval")
    if catalog["asr_selection"] != "NONE_UNASSESSED":
        violations.append("ASR selection changed without a catalog rebaseline")

    if violations:
        print("Draft QA catalog consistency errors:")
        print("\n".join(violations))
        return 1
    print(
        "PASS: draft QA registry has "
        f"{len(active)} active IDs, {len(migrations)} migrations, "
        f"and {len(legacy_retired)} legacy retired definitions"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
