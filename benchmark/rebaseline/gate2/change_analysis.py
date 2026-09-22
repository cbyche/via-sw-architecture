#!/usr/bin/env python3
"""Expand the frozen 24-change design rules into candidate-specific raw M/A/R ledgers.

This is pre-Measurement-Freeze DESIGN_ANALYSIS. It deliberately does not calculate
W-07/W-08 representative means, scores, rankings, or a winner.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

from catalog_check import CHANGES, load, make_report


EXPECTED_FAMILY_COUNTS = {"A": 9, "M": 9, "C": 6}
ALLOWED_CLASSIFICATIONS = {"DESIGN_CHANGE", "CONFIGURATION_ONLY"}
EVIDENCE = "DESIGN_ANALYSIS"
NOT_COMPUTED = "NOT_COMPUTED_PRE_MEASUREMENT_FREEZE"


def read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _choice_items(section: dict[str, Any], choices: dict[str, str]) -> list[str]:
    out = list(section.get("common", []))
    by_choice = section.get("by_choice", {})
    if not isinstance(by_choice, dict):
        raise ValueError("by_choice must be an object")
    for key, ids in by_choice.items():
        if ":" not in key:
            raise ValueError(f"invalid conditional choice key: {key}")
        dp, choice = key.rsplit(":", 1)
        if choices.get(dp) == choice:
            out.extend(ids)
    return out


def _element_type(
    element_id: str,
    catalog: dict[str, Any],
    new_element_types: dict[str, str],
) -> str:
    if element_id in catalog["elements"]:
        return catalog["elements"][element_id]["type"]
    kind = new_element_types.get(element_id)
    if kind not in {"C", "I", "S", "D"}:
        raise ValueError(f"unknown element type for added id {element_id}")
    return kind


def validate_rules(rules: dict[str, Any]) -> None:
    if rules.get("version") != "G2-CHANGE-ANALYSIS-v1":
        raise ValueError("unexpected change-analysis rule version")
    if rules.get("policy", {}).get("aggregate_metric") != NOT_COMPUTED:
        raise ValueError("pre-freeze rules must forbid aggregate metric computation")
    if rules.get("policy", {}).get("score") != "NOT_RUN":
        raise ValueError("pre-freeze rules must keep score NOT_RUN")
    if rules.get("policy", {}).get("winner") != "NOT_RUN":
        raise ValueError("pre-freeze rules must keep winner NOT_RUN")

    changes = rules.get("changes")
    if not isinstance(changes, list):
        raise ValueError("changes must be an array")
    ids = [row.get("id") for row in changes]
    if len(ids) != len(set(ids)):
        raise ValueError("duplicate change id")
    if set(ids) != set(CHANGES):
        raise ValueError(f"change set mismatch: expected {sorted(CHANGES)}, got {sorted(ids)}")

    family_counts = Counter(row.get("family") for row in changes)
    if dict(family_counts) != EXPECTED_FAMILY_COUNTS:
        raise ValueError(f"change-family counts changed: {dict(family_counts)}")

    for row in changes:
        cid = row["id"]
        if row["family"] != cid[0]:
            raise ValueError(f"{cid} family mismatch")
        if row.get("classification") not in ALLOWED_CLASSIFICATIONS:
            raise ValueError(f"{cid} has invalid classification")
        for op in ("modified", "added", "removed"):
            section = row.get(op)
            if not isinstance(section, dict):
                raise ValueError(f"{cid} missing {op} section")
            if not isinstance(section.get("common"), list):
                raise ValueError(f"{cid} {op}.common must be a list")
            if not isinstance(section.get("by_choice"), dict):
                raise ValueError(f"{cid} {op}.by_choice must be an object")
        if not row.get("rationale") or not row.get("preservation") or not row.get("regression"):
            raise ValueError(f"{cid} needs rationale/preservation/regression evidence")


def expand(
    root: Path,
    rules: dict[str, Any],
    rules_sha256: str,
) -> dict[str, Any]:
    validate_rules(rules)
    catalog = load(root)
    design = make_report(catalog)
    new_element_types = rules.get("new_element_types", {})
    rule_by_id = {row["id"]: row for row in rules["changes"]}

    rows: list[dict[str, Any]] = []
    for pair in design["pair_candidates"]:
        candidate_id = pair["candidate_id"]
        configuration_id = pair["configuration_id"]
        config = design["configurations"][configuration_id]
        choices = config["choices"]
        active = set(config["element_ids"])

        for change_id in CHANGES:
            rule = rule_by_id[change_id]
            modified = set(_choice_items(rule["modified"], choices))
            added = set(_choice_items(rule["added"], choices))
            removed = set(_choice_items(rule["removed"], choices))

            if modified - active:
                raise ValueError(
                    f"{candidate_id}/{change_id} modifies inactive ids: {sorted(modified-active)}"
                )
            if removed - active:
                raise ValueError(
                    f"{candidate_id}/{change_id} removes inactive ids: {sorted(removed-active)}"
                )
            if added & active:
                raise ValueError(
                    f"{candidate_id}/{change_id} adds already-active ids: {sorted(added & active)}"
                )
            if modified & added or modified & removed or added & removed:
                raise ValueError(f"{candidate_id}/{change_id} M/A/R sets overlap")

            changed = modified | added | removed
            if rule["classification"] == "CONFIGURATION_ONLY" and changed:
                raise ValueError(f"{change_id} is CONFIGURATION_ONLY but changes architecture ids")
            if rule["classification"] == "DESIGN_CHANGE" and not changed:
                raise ValueError(f"{change_id} is DESIGN_CHANGE but has zero architecture ids")

            type_counts = {kind: 0 for kind in "CISD"}
            for eid in changed:
                type_counts[_element_type(eid, catalog, new_element_types)] += 1

            rows.append(
                {
                    "candidate_id": candidate_id,
                    "configuration_id": configuration_id,
                    "configuration_choices": choices,
                    "baseline_source_revision": design["source_revision"],
                    "catalog_fingerprint": design["catalog_fingerprint"],
                    "rules_sha256": rules_sha256,
                    "change_id": change_id,
                    "family": rule["family"],
                    "metric_owner": "W-07" if rule["family"] == "A" else "W-08",
                    "classification": rule["classification"],
                    "modified": sorted(modified),
                    "added": sorted(added),
                    "removed": sorted(removed),
                    "unique_count": len(changed),
                    "type_counts": type_counts,
                    "functional_preservation": "DESIGN_ARGUMENT_ONLY",
                    "preservation_basis": rule["preservation"],
                    "regression_scope": rule["regression"],
                    "rationale": rule["rationale"],
                    "scope_equivalence": rule.get("scope_equivalence", []),
                    "evidence": EVIDENCE,
                    "candidate_execution": "NOT_RUN",
                    "representative_metric_value": None,
                    "score": None,
                }
            )

    if len(rows) != 8 * 24:
        raise ValueError(f"expected 192 expanded rows, got {len(rows)}")

    per_candidate = Counter(row["candidate_id"] for row in rows)
    if set(per_candidate.values()) != {24}:
        raise ValueError(f"each pair candidate must have 24 rows: {dict(per_candidate)}")

    # Pair labels that resolve to the same complete configuration must never diverge.
    config_change_signatures: dict[tuple[str, str], tuple[Any, ...]] = {}
    for row in rows:
        key = (row["configuration_id"], row["change_id"])
        signature = (
            tuple(row["modified"]),
            tuple(row["added"]),
            tuple(row["removed"]),
            row["unique_count"],
            tuple(sorted(row["type_counts"].items())),
            row["classification"],
        )
        prior = config_change_signatures.setdefault(key, signature)
        if prior != signature:
            raise ValueError(f"same configuration diverged across pair labels: {key}")

    # Keep raw family counts visible without computing the representative mean.
    raw_family_counts: dict[str, dict[str, list[int]]] = defaultdict(lambda: defaultdict(list))
    for row in rows:
        raw_family_counts[row["candidate_id"]][row["family"]].append(row["unique_count"])

    candidate_raw = {}
    for candidate_id, families in sorted(raw_family_counts.items()):
        candidate_raw[candidate_id] = {
            family: {
                "change_ids": [
                    row["change_id"]
                    for row in rows
                    if row["candidate_id"] == candidate_id and row["family"] == family
                ],
                "raw_counts": counts,
                "count": len(counts),
            }
            for family, counts in sorted(families.items())
        }

    return {
        "version": "G2-CHANGE-RAW-v1",
        "status": "COMPLETE_RAW_DESIGN_ANALYSIS_PRE_MEASUREMENT_FREEZE",
        "baseline_source_revision": design["source_revision"],
        "catalog_fingerprint": design["catalog_fingerprint"],
        "rules_sha256": rules_sha256,
        "pair_candidates": len(design["pair_candidates"]),
        "unique_configurations": len(design["configurations"]),
        "raw_rows": rows,
        "candidate_raw_families": candidate_raw,
        "aggregate_metrics": {
            "W-07": NOT_COMPUTED,
            "W-08": NOT_COMPUTED,
        },
        "scores": None,
        "winner": None,
        "candidate_execution": "NOT_RUN",
        "warning": (
            "Raw C/I/S/D design-change counts only. Do not compute or publish W-07/W-08 "
            "representative means, 0-5 scores, rankings, or a winner before Measurement Freeze approval."
        ),
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument(
        "--rules",
        type=Path,
        default=Path(__file__).with_name("w07-w08-change-rules.json"),
    )
    parser.add_argument("--output", type=Path)
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()

    root = args.root.resolve()
    rules_path = args.rules.resolve()
    result = expand(root, read_json(rules_path), sha256(rules_path))

    summary = {
        "status": result["status"],
        "baseline_source_revision": result["baseline_source_revision"],
        "catalog_fingerprint": result["catalog_fingerprint"],
        "rules_sha256": result["rules_sha256"],
        "raw_rows": len(result["raw_rows"]),
        "pair_candidates": result["pair_candidates"],
        "unique_configurations": result["unique_configurations"],
        "W-07": NOT_COMPUTED,
        "W-08": NOT_COMPUTED,
        "scores": None,
        "winner": None,
        "candidate_execution": "NOT_RUN",
    }

    if args.output:
        args.output.mkdir(parents=True, exist_ok=True)
        (args.output / "w07-w08-raw-change-ledger.json").write_text(
            json.dumps(result, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )
        (args.output / "w07-w08-validation-summary.json").write_text(
            json.dumps(summary, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )

    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
