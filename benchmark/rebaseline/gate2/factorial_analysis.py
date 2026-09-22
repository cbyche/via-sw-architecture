#!/usr/bin/env python3
"""Assemble and analyse the frozen 2^4 Gate 2 campaign without a global winner."""
from __future__ import annotations

import argparse
import json
from itertools import combinations
from pathlib import Path
from typing import Any

from catalog_check import CORE, factorial_vectors

SHORT = {"IR-DP01": "IR", "TASK-DP01": "TASK", "AGENT-DP01": "AGENT", "EXEC-DP01": "EXEC"}
METRIC_AXES = {
    "W-01": ("IR-DP01", "EXEC-DP01"),
    "W-02": ("IR-DP01", "TASK-DP01", "AGENT-DP01", "EXEC-DP01"),
    "W-03": ("AGENT-DP01", "EXEC-DP01"),
    "W-04": ("TASK-DP01", "EXEC-DP01"),
    "W-05": ("IR-DP01",),
    "W-06": CORE,
    "W-07": ("AGENT-DP01",),
    "W-08": ("IR-DP01", "TASK-DP01", "AGENT-DP01"),
    "W-09": ("TASK-DP01", "EXEC-DP01"),
    "W-10": ("EXEC-DP01",),
    "W-11": CORE,
    "W-12": CORE,
}


def load(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"expected object: {path}")
    return value


def indexed_rows(payload: dict[str, Any], metric: str) -> dict[str, dict[str, Any]]:
    rows = payload["metrics"][metric]["rows"]
    indexed = {row["configuration_id"]: row for row in rows}
    expected = {config_id for config_id, _ in factorial_vectors()}
    if set(indexed) != expected or len(rows) != 16:
        raise ValueError(f"{metric}: expected exactly the 16 full-factorial configurations")
    return indexed


def change_rows(changes: dict[str, Any], metric: str) -> dict[str, dict[str, Any]]:
    if metric == "W-07":
        return {
            config_id: changes["change_metrics"][f"AGENT-DP01/{vector['AGENT-DP01']}"][metric]
            for config_id, vector in factorial_vectors()
        }
    values = [
        changes["change_metrics"][f"{dp}/{choice}"][metric]
        for dp in ("IR-DP01", "TASK-DP01", "AGENT-DP01")
        for choice in "AB"
    ]
    signatures = {(float(row["metric_value"]), int(row["score"])) for row in values}
    if len(signatures) != 1:
        raise ValueError("W-08 full-configuration aggregation needs a new frozen change model")
    value, score = signatures.pop()
    return {
        config_id: {"metric_value": value, "score": score, "evidence": "DESIGN_ANALYSIS"}
        for config_id, _ in factorial_vectors()
    }


def factorial_effect(rows: dict[str, dict[str, Any]], axes: tuple[str, ...]) -> float:
    total = 0.0
    for config_id, vector in factorial_vectors():
        sign = 1
        for axis in axes:
            sign *= 1 if vector[axis] == "B" else -1
        total += sign * float(rows[config_id]["metric_value"])
    return (2 ** len(axes)) * total / 16


def contextual_contrasts(rows: dict[str, dict[str, Any]], axis: str, direction: str) -> dict[str, Any]:
    position = CORE.index(axis)
    contrasts = []
    for config_id, vector in factorial_vectors():
        if vector[axis] != "A":
            continue
        other = list(config_id)
        other[position] = "B"
        other_id = "".join(other)
        a, b = rows[config_id], rows[other_id]
        av, bv = float(a["metric_value"]), float(b["metric_value"])
        if av == bv:
            preferred = "TIE"
        elif direction == "lower":
            preferred = "A" if av < bv else "B"
        else:
            preferred = "A" if av > bv else "B"
        contrasts.append(
            {
                "context": "".join(vector[dp] if dp != axis else "-" for dp in CORE),
                "A_configuration": config_id,
                "B_configuration": other_id,
                "A_value": av,
                "B_value": bv,
                "preferred": preferred,
                "score_band_split": int(a["score"]) != int(b["score"]),
            }
        )
    counts = {choice: sum(row["preferred"] == choice for row in contrasts) for choice in ("A", "B", "TIE")}
    return {
        "contrasts": contrasts,
        "preference_counts": counts,
        "score_split_contexts": sum(row["score_band_split"] for row in contrasts),
        "context_count": len(contrasts),
    }


def build(
    reference: dict[str, Any],
    representative: dict[str, Any],
    changes: dict[str, Any],
    baseline: dict[str, Any],
) -> dict[str, Any]:
    rows_by_metric: dict[str, dict[str, dict[str, Any]]] = {}
    for metric in ("W-01", "W-02", "W-03", "W-04", "W-06", "W-11", "W-12"):
        rows_by_metric[metric] = indexed_rows(reference, metric)
    for metric in ("W-09", "W-10"):
        rows_by_metric[metric] = indexed_rows(representative, metric)
    for metric in ("W-07", "W-08"):
        rows_by_metric[metric] = change_rows(changes, metric)

    analyses = {}
    for metric, rows in rows_by_metric.items():
        axes = METRIC_AXES[metric]
        direction = baseline["metrics"][metric]["direction"]
        main_effects = {axis: factorial_effect(rows, (axis,)) for axis in axes}
        pairwise = {
            f"{left}×{right}": factorial_effect(rows, (left, right))
            for left, right in combinations(axes, 2)
        }
        analyses[metric] = {
            "status": "MEASURED_FULL_FACTORIAL",
            "applicable_axes": list(axes),
            "main_effect_B_minus_A": main_effects,
            "pairwise_interaction_difference_of_differences": pairwise,
            "contextual_contrasts": {
                axis: contextual_contrasts(rows, axis, direction) for axis in axes
            },
            "rows": [rows[config_id] for config_id, _ in factorial_vectors()],
        }

    configurations = []
    for config_id, vector in factorial_vectors():
        configurations.append(
            {
                "configuration_id": config_id,
                "choices": vector,
                "metrics": {
                    metric: {
                        "metric_value": rows[config_id]["metric_value"],
                        "score": rows[config_id]["score"],
                    }
                    for metric, rows in rows_by_metric.items()
                },
                "W-05": {"status": "BLOCKED_OPENROUTER_KEY", "metric_value": None, "score": None},
            }
        )

    return {
        "version": "G2-FULL-FACTORIAL-v1",
        "status": "COMPLETE_EXCEPT_W05",
        "design": "2^4_FULL_FACTORIAL",
        "configuration_count": 16,
        "global_winner": None,
        "weighted_total": None,
        "missing_values_imputed": False,
        "source_fingerprints": {
            "reference": reference["freeze_fingerprint"],
            "structural": representative["freeze_fingerprint"],
        },
        "W-05": {
            "status": "BLOCKED_OPENROUTER_KEY",
            "reason": "The frozen hosted Qwen3-8B comparison requires OPENROUTER_API_KEY.",
        },
        "metric_analysis": analyses,
        "configurations": configurations,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("reference", "representative", "changes", "baseline", "output"):
        parser.add_argument(f"--{name}", type=Path, required=True)
    args = parser.parse_args()
    payload = build(load(args.reference), load(args.representative), load(args.changes), load(args.baseline))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    if args.output.exists():
        parser.error(f"refusing to overwrite: {args.output}")
    args.output.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(args.output)


if __name__ == "__main__":
    main()
