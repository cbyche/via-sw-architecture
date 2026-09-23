from __future__ import annotations

import json
from copy import deepcopy
from pathlib import Path

import pytest

from dp00_analysis.acceptance import (
    ACCEPTANCE_CONTRACT_VERSION,
    BOOTSTRAP_SEED,
    derive_calibration_acceptance,
    theil_sen_slope,
)


def manifest(cycles=16):
    return {"measured_cycle_count": cycles}


def paired(*, capture=lambda cycle, scenario, alternative: 1_000.0,
           minimal=lambda cycle, scenario, alternative: 1_000.0):
    rows = []
    for cycle in range(16):
        for scenario_ordinal, scenario in enumerate(("P01", "P02", "P03")):
            for alternative_ordinal, alternative in enumerate("ABCD"):
                first = (
                    "CAPTURE"
                    if (scenario_ordinal + alternative_ordinal + cycle) % 2 == 0
                    else "MINIMAL"
                )
                capture_value = capture(cycle, scenario, alternative)
                minimal_value = minimal(cycle, scenario, alternative)
                rows.append({
                    "pair_id": f"{cycle}:{scenario}:{alternative}",
                    "complete": True,
                    "qualified": True,
                    "cycle": cycle,
                    "scenario": scenario,
                    "alternative": alternative,
                    "position": (alternative_ordinal - cycle) % 4,
                    "first_mode": first,
                    "capture_ftol_nanos": capture_value,
                    "minimal_ftol_nanos": minimal_value,
                    "ftol_delta_nanos": capture_value - minimal_value,
                })
    return {
        "pairs": rows,
        "incomplete_pairs": [],
        "duplicate_mode_pairs": [],
        "semantic_mismatch_pairs": [],
        "qa02_mismatch_pairs": [],
        "qa04_mismatch_pairs": [],
        "provenance_mismatch_pairs": [],
    }


def derive(value, **kwargs):
    return derive_calibration_acceptance(
        value,
        manifest(),
        schedule_valid=True,
        bootstrap_resamples=kwargs.pop("resamples", 40),
        **kwargs,
    )


def test_contract_file_matches_analyzer_constants():
    path = Path(__file__).resolve().parents[2] / "contracts" / "dp00-calibration-acceptance-v1.json"
    contract = json.loads(path.read_text())
    assert contract["contract_version"] == ACCEPTANCE_CONTRACT_VERSION
    assert contract["margin"]["fraction"] == 0.05
    assert contract["bootstrap"]["random_seed"] == BOOTSTRAP_SEED
    assert contract["bootstrap"]["resamples"] == 10_000


def test_M_and_five_percent_margin_are_pooled_minimal_p50():
    result = derive(paired(minimal=lambda cycle, scenario, alternative: 900 + cycle * 10))
    assert result["baseline_M_nanos"] == 975
    assert result["absolute_margin_nanos"] == 48.75


@pytest.mark.parametrize("mode", ["CAPTURE", "MINIMAL"])
def test_early_late_shift_pass_fixture(mode):
    values = lambda cycle, scenario, alternative: 1_000 + (20 if cycle >= 8 else 0)
    value = paired(capture=values) if mode == "CAPTURE" else paired(minimal=values)
    assert derive(value)["cycle_drift"][mode]["early_late_pass"] is True


@pytest.mark.parametrize("mode", ["CAPTURE", "MINIMAL"])
def test_early_late_shift_fail_fixture(mode):
    values = lambda cycle, scenario, alternative: 1_000 + (100 if cycle >= 8 else 0)
    value = paired(capture=values) if mode == "CAPTURE" else paired(minimal=values)
    assert derive(value)["cycle_drift"][mode]["early_late_pass"] is False


@pytest.mark.parametrize("step", [10, -10])
def test_theil_sen_positive_and_negative_drift_fail(step):
    values = lambda cycle, scenario, alternative: 1_000 + step * cycle
    result = derive(paired(capture=values))["cycle_drift"]["CAPTURE"]
    assert result["theil_sen_slope_nanos_per_cycle"] == step
    assert result["theil_sen_pass"] is False


def test_theil_sen_within_margin_pass():
    values = lambda cycle, scenario, alternative: 1_000 + 2 * cycle
    assert derive(paired(capture=values))["cycle_drift"]["CAPTURE"]["theil_sen_pass"] is True
    assert theil_sen_slope({cycle: 1_000 + 2 * cycle for cycle in range(16)}) == 2


@pytest.mark.parametrize("difference, expected", [(40, True), (60, False)])
def test_mode_order_interaction_gate(difference, expected):
    value = paired()
    for row in value["pairs"]:
        row["capture_ftol_nanos"] += difference if row["first_mode"] == "CAPTURE" else 0
        row["ftol_delta_nanos"] = row["capture_ftol_nanos"] - row["minimal_ftol_nanos"]
    assert derive(value)["mode_order_interaction"]["pass"] is expected


def test_absolute_value_semantics_for_negative_shift():
    values = lambda cycle, scenario, alternative: 1_100 - (100 if cycle >= 8 else 0)
    result = derive(paired(capture=values))
    assert result["cycle_drift"]["CAPTURE"]["early_late_shift_nanos"] == -100
    assert result["cycle_drift"]["CAPTURE"]["early_late_pass"] is False


def test_exactly_16_measured_cycles_required():
    result = derive_calibration_acceptance(
        paired(), manifest(15), schedule_valid=True, bootstrap_resamples=2
    )
    assert "EXACTLY_16_MEASURED_CYCLES_REQUIRED" in result["rejection_reasons"]


def test_missing_cycle_is_rejected():
    value = paired()
    value["pairs"] = [row for row in value["pairs"] if row["cycle"] != 15]
    result = derive(value)
    assert "QA01_QUALIFIED_CYCLE_COVERAGE_MISSING" in result["rejection_reasons"]


def test_identical_pair_requires_eight_eight_mode_order_balance():
    value = paired()
    for row in value["pairs"]:
        if row["scenario"] == "P01" and row["alternative"] == "A":
            row["first_mode"] = "CAPTURE"
    result = derive(value)
    assert "QA01_IDENTICAL_PAIR_MODE_ORDER_NOT_8_8" in result["rejection_reasons"]


@pytest.mark.parametrize(
    "key",
    [
        "incomplete_pairs",
        "duplicate_mode_pairs",
        "semantic_mismatch_pairs",
        "qa02_mismatch_pairs",
        "qa04_mismatch_pairs",
        "provenance_mismatch_pairs",
    ],
)
def test_malformed_pair_is_rejected(key):
    value = paired()
    value[key] = [{"pair_id": "bad"}]
    result = derive(value)
    assert result["accepted_input"] is False
    assert any(reason.startswith("MALFORMED_PAIR_") for reason in result["rejection_reasons"])


def test_bootstrap_is_deterministically_reproducible():
    value = paired(capture=lambda cycle, scenario, alternative: 1_000 + cycle * 3)
    left = derive(value, resamples=100)["bootstrap"]
    right = derive(value, resamples=100)["bootstrap"]
    assert left == right
    assert left["random_seed"] == BOOTSTRAP_SEED


def test_bootstrap_declares_and_preserves_pair_sampling_unit():
    bootstrap = derive(paired())["bootstrap"]
    assert bootstrap["pair_structure_preserved"] is True
    assert bootstrap["sampling_unit"] == "complete-QA01-qualified-CAPTURE-MINIMAL-pair"
    assert bootstrap["observed_pair_count"] == 192


def test_acceptance_result_serializes_deterministically():
    value = paired(capture=lambda cycle, scenario, alternative: 1_000 + cycle)
    left = json.dumps(derive(value), indent=2, sort_keys=True)
    right = json.dumps(derive(deepcopy(value)), indent=2, sort_keys=True)
    assert left == right
