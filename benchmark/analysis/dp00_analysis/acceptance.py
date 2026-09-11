"""Frozen DP-00 calibration stability acceptance and bootstrap diagnostics."""

from __future__ import annotations

import random
from collections import Counter, defaultdict
from statistics import median
from typing import Any, Iterable

from .qa01 import linear_percentile, nearest_rank

ACCEPTANCE_CONTRACT_VERSION = "dp00-calibration-acceptance-v1"
MARGIN_FRACTION = 0.05
MEASURED_CYCLES = 16
EARLY_CYCLES = tuple(range(8))
LATE_CYCLES = tuple(range(8, 16))
PREDICTED_SHIFT_CYCLE_SPAN = 15
BOOTSTRAP_RESAMPLES = 10_000
BOOTSTRAP_SEED = 20_260_911
QA01_SCENARIOS = {"P01", "P02", "P03"}


def _percentiles(values: Iterable[int | float]) -> dict[str, float]:
    samples = list(values)
    return {
        f"p{percentile}": linear_percentile(samples, percentile / 100)
        for percentile in (50, 95, 99)
    }


def theil_sen_slope(cycle_p50: dict[int, float]) -> float:
    """Median of all pairwise slopes across the 16 explicit cycle p50s."""
    if set(cycle_p50) != set(range(MEASURED_CYCLES)):
        raise ValueError("Theil-Sen requires exactly measured cycles 0..15")
    slopes = [
        (cycle_p50[right] - cycle_p50[left]) / (right - left)
        for left in range(MEASURED_CYCLES)
        for right in range(left + 1, MEASURED_CYCLES)
    ]
    return float(median(slopes))


def _bootstrap_ci(values: list[float]) -> dict[str, float | int]:
    return {
        "confidence_level": 0.95,
        "lower": nearest_rank(values, 0.025),
        "upper": nearest_rank(values, 0.975),
        "resamples": len(values),
    }


def _bootstrap(rows: list[dict[str, Any]], resamples: int, seed: int) -> dict[str, Any]:
    # Sampling units remain whole pairs. Stratification keeps early/late and first-mode
    # population sizes fixed while a CAPTURE and MINIMAL observation always travel together.
    strata: dict[tuple[str, str], list[dict[str, Any]]] = defaultdict(list)
    for row in rows:
        half = "early" if row["cycle"] in EARLY_CYCLES else "late"
        strata[(half, row["first_mode"])].append(row)
    rng = random.Random(seed)
    draws: dict[str, list[float]] = defaultdict(list)
    for _ in range(resamples):
        sample = [
            rng.choice(population)
            for key in sorted(strata)
            for population in (strata[key],)
            for _ in range(len(population))
        ]
        capture = [row["capture"] for row in sample]
        minimal = [row["minimal"] for row in sample]
        delta = [row["delta"] for row in sample]
        early = [row for row in sample if row["cycle"] in EARLY_CYCLES]
        late = [row for row in sample if row["cycle"] in LATE_CYCLES]
        capture_first = [row["delta"] for row in sample if row["first_mode"] == "CAPTURE"]
        minimal_first = [row["delta"] for row in sample if row["first_mode"] == "MINIMAL"]
        draws["pooled_capture_p50"].append(linear_percentile(capture, 0.5))
        draws["pooled_minimal_p50"].append(linear_percentile(minimal, 0.5))
        draws["pooled_paired_delta_p50"].append(linear_percentile(delta, 0.5))
        draws["capture_early_late_difference"].append(
            linear_percentile([row["capture"] for row in late], 0.5)
            - linear_percentile([row["capture"] for row in early], 0.5)
        )
        draws["minimal_early_late_difference"].append(
            linear_percentile([row["minimal"] for row in late], 0.5)
            - linear_percentile([row["minimal"] for row in early], 0.5)
        )
        draws["mode_order_interaction_signed"].append(
            float(median(capture_first) - median(minimal_first))
        )
    return {
        "diagnostic_only": True,
        "method": "paired-stratified-by-cycle-half-and-first-mode",
        "sampling_unit": "complete-QA01-qualified-CAPTURE-MINIMAL-pair",
        "observed_pair_count": len(rows),
        "pair_structure_preserved": True,
        "random_seed": seed,
        "confidence_intervals": {
            key: _bootstrap_ci(values) for key, values in sorted(draws.items())
        },
    }


def _group_diagnostics(rows: list[dict[str, Any]], key: str) -> dict[str, Any]:
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in rows:
        grouped[str(row[key])].append(row)
    return {
        group: {
            "pair_count": len(items),
            "capture": _percentiles(row["capture"] for row in items),
            "minimal": _percentiles(row["minimal"] for row in items),
            "paired_delta": _percentiles(row["delta"] for row in items),
        }
        for group, items in sorted(grouped.items())
    }


def derive_calibration_acceptance(
    paired: dict[str, Any],
    manifest: dict[str, Any] | None,
    *,
    schedule_valid: bool,
    bootstrap_resamples: int = BOOTSTRAP_RESAMPLES,
    bootstrap_seed: int = BOOTSTRAP_SEED,
) -> dict[str, Any]:
    """Derive gates from qualified QA-01 pairs; malformed evidence is rejected."""
    if manifest is None:
        return {"present": False, "contract_version": ACCEPTANCE_CONTRACT_VERSION}

    rejection_reasons = []
    if manifest.get("measured_cycle_count") != MEASURED_CYCLES:
        rejection_reasons.append("EXACTLY_16_MEASURED_CYCLES_REQUIRED")
    for key in ("missing_pairs", "duplicate_pairs", "semantic_mismatches", "provenance_mismatches"):
        if paired.get(key):
            rejection_reasons.append(f"MALFORMED_PAIR_{key.upper()}")
    if not schedule_valid:
        rejection_reasons.append("CALIBRATION_SCHEDULE_INVALID")

    rows = []
    for item in paired.get("pairs", []):
        if not item.get("qualified") or item.get("scenario") not in QA01_SCENARIOS:
            continue
        if item.get("capture_ftol_nanos") is None or item.get("minimal_ftol_nanos") is None:
            continue
        rows.append({
            "pair_id": item["pair_id"],
            "cycle": item["cycle"],
            "scenario": item["scenario"],
            "alternative": item["alternative"],
            "position": item["position"],
            "first_mode": item["first_mode"],
            "capture": item["capture_ftol_nanos"],
            "minimal": item["minimal_ftol_nanos"],
            "delta": item["ftol_delta_nanos"],
        })
    cycles = {row["cycle"] for row in rows}
    if cycles != set(range(MEASURED_CYCLES)):
        rejection_reasons.append("QA01_QUALIFIED_CYCLE_COVERAGE_MISSING")
    identity_order: dict[tuple[str, str], Counter[str]] = defaultdict(Counter)
    for row in rows:
        identity_order[(row["scenario"], row["alternative"])][row["first_mode"]] += 1
    if any(
        counts != Counter({"CAPTURE": 8, "MINIMAL": 8})
        for counts in identity_order.values()
    ):
        rejection_reasons.append("QA01_IDENTICAL_PAIR_MODE_ORDER_NOT_8_8")
    first_mode_counts = Counter(row["first_mode"] for row in rows)
    if not rows or not first_mode_counts["CAPTURE"] or not first_mode_counts["MINIMAL"]:
        rejection_reasons.append("QA01_MODE_ORDER_POPULATION_MISSING")
    if rejection_reasons:
        return {
            "present": True,
            "contract_version": ACCEPTANCE_CONTRACT_VERSION,
            "accepted_input": False,
            "rejection_reasons": sorted(set(rejection_reasons)),
            "qualified_qa01_pair_count": len(rows),
            "cycle_drift_acceptance": "FAIL",
            "mode_order_interaction_acceptance": "FAIL",
        }

    minimal_values = [row["minimal"] for row in rows]
    baseline = linear_percentile(minimal_values, 0.5)
    margin = MARGIN_FRACTION * baseline
    drift = {}
    for mode, field in (("CAPTURE", "capture"), ("MINIMAL", "minimal")):
        early_p50 = linear_percentile(
            [row[field] for row in rows if row["cycle"] in EARLY_CYCLES], 0.5
        )
        late_p50 = linear_percentile(
            [row[field] for row in rows if row["cycle"] in LATE_CYCLES], 0.5
        )
        shift = late_p50 - early_p50
        cycle_p50 = {
            cycle: linear_percentile(
                [row[field] for row in rows if row["cycle"] == cycle], 0.5
            )
            for cycle in range(MEASURED_CYCLES)
        }
        slope = theil_sen_slope(cycle_p50)
        predicted = slope * PREDICTED_SHIFT_CYCLE_SPAN
        shift_pass = abs(shift) <= margin
        slope_pass = abs(predicted) <= margin
        drift[mode] = {
            "early_p50_nanos": early_p50,
            "late_p50_nanos": late_p50,
            "early_late_shift_nanos": shift,
            "early_late_absolute_shift_nanos": abs(shift),
            "early_late_normalized_fraction": abs(shift) / baseline,
            "early_late_pass": shift_pass,
            "cycle_p50_nanos": {str(key): value for key, value in cycle_p50.items()},
            "theil_sen_slope_nanos_per_cycle": slope,
            "predicted_cycle0_to_15_shift_nanos": predicted,
            "predicted_absolute_shift_nanos": abs(predicted),
            "predicted_normalized_fraction": abs(predicted) / baseline,
            "theil_sen_pass": slope_pass,
            "pass": shift_pass and slope_pass,
        }

    capture_first = [row["delta"] for row in rows if row["first_mode"] == "CAPTURE"]
    minimal_first = [row["delta"] for row in rows if row["first_mode"] == "MINIMAL"]
    d_cf, d_mf = float(median(capture_first)), float(median(minimal_first))
    signed_interaction = d_cf - d_mf
    interaction = abs(signed_interaction)

    return {
        "present": True,
        "contract_version": ACCEPTANCE_CONTRACT_VERSION,
        "accepted_input": True,
        "rejection_reasons": [],
        "qualified_qa01_pair_count": len(rows),
        "percentile_estimator": "linear-interpolated-(n-1)",
        "baseline_M_nanos": baseline,
        "margin_fraction": MARGIN_FRACTION,
        "absolute_margin_nanos": margin,
        "pooled": {
            "capture": _percentiles(row["capture"] for row in rows),
            "minimal": _percentiles(row["minimal"] for row in rows),
            "paired_delta": _percentiles(row["delta"] for row in rows),
        },
        "by_alternative": _group_diagnostics(rows, "alternative"),
        "by_scenario": _group_diagnostics(rows, "scenario"),
        "by_position": _group_diagnostics(rows, "position"),
        "by_cycle": _group_diagnostics(rows, "cycle"),
        "cycle_drift": drift,
        "cycle_drift_acceptance": (
            "PASS" if all(item["pass"] for item in drift.values()) else "FAIL"
        ),
        "mode_order_interaction": {
            "capture_first_count": len(capture_first),
            "minimal_first_count": len(minimal_first),
            "capture_first_median_delta_nanos": d_cf,
            "minimal_first_median_delta_nanos": d_mf,
            "signed_difference_nanos": signed_interaction,
            "magnitude_nanos": interaction,
            "normalized_fraction": interaction / baseline,
            "pass": interaction <= margin,
        },
        "mode_order_interaction_acceptance": "PASS" if interaction <= margin else "FAIL",
        "bootstrap": _bootstrap(rows, bootstrap_resamples, bootstrap_seed),
    }
