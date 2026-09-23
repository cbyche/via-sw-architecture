"""Generate deterministic Session 5 report data from committed DP-00 evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
from collections import Counter, defaultdict
from pathlib import Path
from statistics import fmean
from typing import Any


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical_actual(values: list[Any]) -> list[Any]:
    """Compare set-like diagnostic Actual values without changing metric semantics."""
    return sorted(values, key=lambda value: json.dumps(value, sort_keys=True))


def qa02_semantics(summary: dict[str, Any]) -> dict[str, Any]:
    qa02 = summary["qa02"]
    episodes = {}
    for row in qa02["scenario_diagnostics"]:
        episodes[row["run_id"]] = {
            "exact_conformant": row["exact_conformant"],
            "evidence_complete": row["evidence_complete"],
            "constraints": {
                item["constraint_id"]: {
                    "dimension": item["dimension"],
                    "status": item["status"],
                    "decision_basis": item["decision_basis"],
                    "expected": item["expected"],
                    "actual": canonical_actual(item["actual"]),
                }
                for item in row["constraints"]
            },
        }
    coverage = qa02["constraint_alternative_coverage"]
    return {
        "numerator": qa02["numerator"],
        "denominator": qa02["denominator"],
        "aecr_fraction": qa02["aecr_fraction"],
        "aecr": qa02["aecr"],
        "percentage": qa02["percentage"],
        "dimension_diagnostics": qa02["dimension_diagnostics"],
        "episodes": episodes,
        "coverage": {
            key: value
            for key, value in coverage.items()
            if key != "cells"
        },
        "coverage_cells": sorted(
            coverage["cells"],
            key=lambda item: (
                item["scenario"], item["constraint_id"], item["alternative"]
            ),
        ),
    }


def group_key(row: dict[str, Any]) -> str:
    run_id = row["run_id"]
    profile = "Z" if "-Z-" in run_id else "C"
    return f"profile-{profile.lower()}:{row['alternative']}"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--v1", type=Path, required=True)
    parser.add_argument("--v2", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    raw_root = args.raw
    v1 = load_json(args.v1)
    v2 = load_json(args.v2)
    campaign = load_json(raw_root / "campaign-provenance.json")
    run_provenance = [
        load_json(path)
        for path in sorted(raw_root.glob("profile-*/*/provenance.json"))
    ]

    qa01_rows = []
    for key, item in sorted(v2["qa01"].items()):
        stats = item["statistics"]
        nearest = stats["nearest_rank"]
        linear = stats["linear"]
        sensitivity = stats["small_n_sensitivity"]
        qa01_rows.append(
            {
                "group": key,
                "eligible": item["counts"]["eligible_count"],
                "success": item["counts"]["successful_count"],
                "failure": item["counts"]["failed_count"],
                "min_ms": stats["min"] / 1_000_000,
                "mean_ms": stats["mean"] / 1_000_000,
                "max_ms": stats["max"] / 1_000_000,
                "nearest_rank_ms": {
                    name: value / 1_000_000 for name, value in nearest.items()
                },
                "linear_ms": {
                    name: value / 1_000_000 for name, value in linear.items()
                },
                "p95_absolute_delta_ms": sensitivity["absolute_difference"]
                / 1_000_000,
                "p95_relative_delta": sensitivity["relative_difference"],
            }
        )

    qa02_groups: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in v2["qa02"]["scenario_diagnostics"]:
        qa02_groups[group_key(row)].append(row)
    qa02_rows = []
    detail = []
    missing = unevaluable = 0
    for key, rows in sorted(qa02_groups.items()):
        statuses = Counter(
            constraint["status"]
            for row in rows
            for constraint in row["constraints"]
        )
        missing += statuses["MISSING_ACTUAL_EVIDENCE"]
        unevaluable += statuses["UNEVALUABLE"]
        conformant = sum(row["exact_conformant"] for row in rows)
        qa02_rows.append(
            {
                "group": key,
                "numerator": conformant,
                "denominator": len(rows),
                "percentage": 100 * conformant / len(rows),
                "constraint_pass": statuses["PASS"],
                "constraint_fail": statuses["FAIL"],
                "missing": statuses["MISSING_ACTUAL_EVIDENCE"],
                "unevaluable": statuses["UNEVALUABLE"],
                "non_conformant_scenarios": [
                    row["scenario"] for row in rows if not row["exact_conformant"]
                ],
            }
        )
        for row in rows:
            if row["scenario"] in {"P04", "P06", "P07", "P09"}:
                detail.append(
                    {
                        "group": key,
                        "scenario": row["scenario"],
                        "exact_conformant": row["exact_conformant"],
                        "failed_constraints": [
                            {
                                "constraint_id": item["constraint_id"],
                                "dimension": item["dimension"],
                                "decision_basis": item["decision_basis"],
                                "actual": item["actual"],
                            }
                            for item in row["constraints"]
                            if item["status"] != "PASS"
                        ],
                    }
                )

    qa04_eligible = Counter(
        group_key(row) for row in v2["qa04"]["per_episode"]
    )
    qa04_no_route = Counter(
        group_key(row) for row in v2["qa04"]["no_route_commit"]
    )
    qa04_rows = []
    for key, aggregate in sorted(v2["qa04"]["aggregates"].items()):
        eligible = qa04_eligible[key]
        no_route = qa04_no_route[key]
        group_episodes = [
            row for row in v2["qa04"]["per_episode"] if group_key(row) == key
        ]
        expectation_totals = Counter(
            row["route_commit_expectation"] for row in group_episodes
        )
        expectation_no_route = Counter(
            row["route_commit_expectation"]
            for row in group_episodes
            if row["no_route_commit"]
        )
        qa04_rows.append(
            {
                "group": key,
                **aggregate,
                "eligible_episodes": eligible,
                "no_route_count": no_route,
                "no_route_rate": no_route / eligible,
                "no_route_by_expectation": {
                    expectation: {
                        "count": expectation_no_route[expectation],
                        "eligible": expectation_totals[expectation],
                        "rate": expectation_no_route[expectation]
                        / expectation_totals[expectation],
                    }
                    for expectation in sorted(expectation_totals)
                },
            }
        )

    instrumentation = []
    for profile in ("Z", "C"):
        rows = [
            row for row in run_provenance if row["latency_profile"]["profile_id"] == profile
        ]
        events = [row["event_count"] for row in rows]
        costs = [row["capture_append_cost_nanos"] for row in rows]
        instrumentation.append(
            {
                "profile": profile,
                "episode_count": len(rows),
                "event_count": {
                    "min": min(events), "mean": fmean(events), "max": max(events)
                },
                "append_cost_nanos": {
                    "min": min(costs), "mean": fmean(costs), "max": max(costs)
                },
            }
        )

    runtime = run_provenance[0]
    profile_counts = Counter(
        row["latency_profile"]["profile_id"] for row in run_provenance
    )
    position_map = {
        alternative: sorted(
            {
                row["sequence_position"]
                for row in run_provenance
                if row["alternative"] == alternative
            }
        )
        for alternative in "ABCD"
    }
    result = {
        "report_data_schema_version": "dp00-calibration-report-data-v1",
        "source": {
            "campaign_id": campaign["campaign_id"],
            "raw_path": str(raw_root),
            "v1_summary_path": str(args.v1),
            "v2_summary_path": str(args.v2),
            "v1_summary_sha256": sha256(args.v1),
            "v2_summary_sha256": sha256(args.v2),
        },
        "configuration": {
            "source_git_commit": campaign["source_git_commit"],
            "corpus_id": campaign["pilot_corpus_id"],
            "corpus_version": campaign["pilot_corpus_version"],
            "analysis_version": v2["analysis_version"],
            "rustc_version": runtime["rustc_version"].splitlines()[0],
            "cargo_version": runtime["cargo_version"],
            "target": runtime["target"],
            "build_profile": runtime["build_profile"],
            "tokio_version": runtime["tokio_resolved_version"],
            "runtime_worker_policy": runtime["runtime_worker_policy"],
            "warmup_count": campaign["campaign_configuration"]["warmup_count"],
            "measured_repetition_count": campaign["campaign_configuration"]["measured_repetition_count"],
            "order_policy": campaign["campaign_configuration"]["order_policy"],
            "position_map": position_map,
            "instrumentation_mode": campaign["campaign_configuration"]["instrumentation_mode"],
            "profiles": campaign["profile_sequence"],
            "schema_versions": {
                "campaign": campaign["provenance_schema_version"],
                "run": runtime["provenance_schema_version"],
                "canonical_event": runtime["canonical_event_schema_version"],
                "model_call": runtime["model_call_schema_version"],
            },
            "coverage_manifest": v2["qa02"]["constraint_alternative_coverage"]["coverage_manifest_version"],
        },
        "validity": {
            "validation": v2["validation"],
            "measured_episodes": len(run_provenance),
            "profile_counts": dict(sorted(profile_counts.items())),
            "missing_episode_count": max(0, 80 - len(run_provenance)),
            "duplicate_episode_count": len(run_provenance)
            - len({row["run_id"] for row in run_provenance}),
            "missing_actual_evidence": missing,
            "unevaluable": unevaluable,
            "coverage_gate": v2["qa02"]["constraint_alternative_coverage"],
        },
        "semantic_identity_gate": {
            "qa01_equal": v1["qa01"] == v2["qa01"],
            "qa02_equal": qa02_semantics(v1) == qa02_semantics(v2),
            "qa04_equal": v1["qa04"] == v2["qa04"],
        },
        "qa01": qa01_rows,
        "qa02": qa02_rows,
        "qa02_discriminating_scenarios": sorted(
            detail, key=lambda row: (row["scenario"], row["group"])
        ),
        "qa04": qa04_rows,
        "instrumentation": instrumentation,
        "coverage": {
            "covered_classes": ["R1", "R2", "R3", "R4", "R5", "R6", "R7", "R10"],
            "not_covered_classes": ["R8", "R9"],
        },
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
