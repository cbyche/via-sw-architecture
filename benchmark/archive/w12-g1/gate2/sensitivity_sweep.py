#!/usr/bin/env python3
"""Build the Gate 2 12-A sensitivity and Differentiation Gate ledger.

The evaluator consumes only frozen scores and representative summaries. Missing
measurements remain missing; it never imputes zero or creates a weighted winner.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


DP_METRICS = {
    "IR-DP01": ("W-01", "W-02", "W-05", "W-08"),
    "TASK-DP01": ("W-02", "W-04", "W-08", "W-09"),
    "AGENT-DP01": ("W-02", "W-03", "W-07", "W-08"),
    "EXEC-DP01": ("W-01", "W-04", "W-09", "W-10"),
}

CONFIG_ROWS = {
    ("TASK-DP01", "A"): "AAAA",
    ("TASK-DP01", "B"): "ABAA",
    ("EXEC-DP01", "A"): "AABA",
    ("EXEC-DP01", "B"): "AAAB",
}


def read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"expected JSON object: {path}")
    return value


def target_pass(value: float, spec: dict[str, Any]) -> bool:
    if spec["direction"] == "lower":
        return value <= float(spec["target"])
    return value >= float(spec["target"])


def measured_rows(
    dp: str,
    metric: str,
    change_scores: dict[str, Any],
    representative: dict[str, Any],
) -> dict[str, dict[str, Any]]:
    if metric in {"W-07", "W-08"}:
        return {
            candidate: change_scores["change_metrics"][f"{dp}/{candidate}"][metric]
            for candidate in ("A", "B")
        }
    rows = representative["metrics"][metric]["rows"]
    indexed = {row["configuration_id"]: row for row in rows}
    return {
        candidate: indexed[CONFIG_ROWS[(dp, candidate)]]
        for candidate in ("A", "B")
    }


def evaluate_metric(
    dp: str,
    metric: str,
    coverage: dict[str, Any],
    baseline: dict[str, Any],
    change_scores: dict[str, Any],
    representative: dict[str, Any],
) -> dict[str, Any]:
    status = coverage["metrics"][metric]
    row: dict[str, Any] = {
        "working_asr": metric,
        "measurement_status": status["status"],
        "reason": status["reason"],
        "candidates": None,
        "score_band_split": False,
        "target_pass_split": False,
        "opposite_direction_tradeoff": False,
        "g3_observable_sensitivity": False,
    }
    if status["status"] != "MEASURED":
        return row

    values = measured_rows(dp, metric, change_scores, representative)
    spec = baseline["metrics"][metric]
    candidates = {}
    for candidate, measured in values.items():
        value = float(measured["metric_value"])
        candidates[candidate] = {
            "metric_value": value,
            "score": int(measured["score"]),
            "target_pass": target_pass(value, spec),
        }
    score_split = abs(candidates["A"]["score"] - candidates["B"]["score"]) >= 1
    pass_split = candidates["A"]["target_pass"] != candidates["B"]["target_pass"]
    row.update(
        {
            "candidates": candidates,
            "score_band_split": score_split,
            "target_pass_split": pass_split,
            "g3_observable_sensitivity": score_split or pass_split,
        }
    )
    return row


def build_sweep(
    baseline: dict[str, Any],
    coverage: dict[str, Any],
    change_scores: dict[str, Any],
    representative: dict[str, Any],
) -> dict[str, Any]:
    fingerprints = {
        coverage.get("freeze_fingerprint"),
        change_scores.get("freeze_fingerprint"),
        representative.get("freeze_fingerprint"),
    }
    if len(fingerprints) != 1 or None in fingerprints:
        raise ValueError("all sensitivity inputs must share one freeze fingerprint")
    expected_metrics = {f"W-{index:02d}" for index in range(1, 13)}
    if set(coverage.get("metrics", {})) != expected_metrics:
        raise ValueError("coverage must contain exactly W-01..W-12")

    decisions = {}
    for dp, metrics in DP_METRICS.items():
        rows = [
            evaluate_metric(dp, metric, coverage, baseline, change_scores, representative)
            for metric in metrics
        ]
        primary = [row["working_asr"] for row in rows if row["g3_observable_sensitivity"]]
        selected = None
        if dp == "AGENT-DP01" and primary == ["W-07"]:
            w07 = next(row for row in rows if row["working_asr"] == "W-07")
            scores = {candidate: values["score"] for candidate, values in w07["candidates"].items()}
            if scores["A"] != scores["B"]:
                selected = max(scores, key=scores.get)
        decisions[dp] = {
            "metrics": rows,
            "differentiation_gate": {
                "primary_metrics": primary,
                "passed": bool(primary),
                "g1_product_relevance": "PASS",
                "g2_natural_structural_causality": "PASS",
                "g3_observable_sensitivity": "PASS" if primary else "FAIL",
                "g4_non_redundancy": "PASS" if primary else "NOT_APPLICABLE",
                "g5_evidence_traceability": "PASS" if primary else "NOT_APPLICABLE",
            },
            "decision_status": "ACCEPTED" if selected else "DEFERRED_INSUFFICIENT_DIFFERENTIATION",
            "selected_candidate": selected,
        }

    return {
        "version": "G2-SENSITIVITY-v1",
        "freeze_fingerprint": fingerprints.pop(),
        "weighted_total": None,
        "global_winner": None,
        "missing_values_imputed": False,
        "coverage": coverage["metrics"],
        "decisions": decisions,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--coverage", type=Path, required=True)
    parser.add_argument("--change-scores", type=Path, required=True)
    parser.add_argument("--representative", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    payload = build_sweep(
        read_json(args.baseline),
        read_json(args.coverage),
        read_json(args.change_scores),
        read_json(args.representative),
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(args.output)


if __name__ == "__main__":
    main()
