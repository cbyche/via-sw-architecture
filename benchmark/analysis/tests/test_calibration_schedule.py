from __future__ import annotations

from copy import deepcopy

from dp00_analysis.calibration import (
    calibration_schedule_diagnostics,
    validate_calibration_schedule,
)
from dp00_analysis.models import ActualSemanticFacts, EpisodeEvidence
from dp00_analysis.validation import validate_calibration_manifest


ALTERNATIVES = "ABCD"


def mode_order(scenario_ordinal: int, alternative_ordinal: int, cycle: int):
    if (scenario_ordinal + alternative_ordinal + cycle) % 2 == 0:
        return ("CAPTURE", "MINIMAL")
    return ("MINIMAL", "CAPTURE")


def manifest(run_ids):
    covered_paths = [
        {
            "scenario_id": f"P{scenario + 1:02}",
            "alternative": alternative,
            "instrumentation_mode": mode,
        }
        for scenario in range(12)
        for alternative in ALTERNATIVES
        for mode in ("CAPTURE", "MINIMAL")
    ]
    return {
        "provenance_schema_version": "dp00-calibration-manifest-v1",
        "calibration_protocol_version": "dp00-calibration-protocol-v1",
        "calibration_id": "calibration-next",
        "source_sha": "frozen-sha",
        "full_prewarm_cycle_count": 2,
        "completed_full_prewarm_cycle_count": 2,
        "prewarm_cycles": [
            {"cycle_id": "W0", "completed": True, "expected_path_count": 96, "completed_path_count": 96, "covered_paths": covered_paths},
            {"cycle_id": "W1", "completed": True, "expected_path_count": 96, "completed_path_count": 96, "covered_paths": deepcopy(covered_paths)},
        ],
        "invocation_warmup_count": 1,
        "measured_cycle_count": 16,
        "mode_order_policy": "SCENARIO_ALTERNATIVE_PARITY_INVERT_V1",
        "alternative_rotation_policy": "DETERMINISTIC_CYCLIC_V1",
        "repetition_policy": "FIXED_16_MEASURED_CYCLES_V1",
        "adaptive_stopping": False,
        "expected_measured_execution_count": 1536,
        "completed_measured_execution_count": 1536,
        "expected_pair_count": 768,
        "completed_pair_count": 768,
        "expected_qa01_pair_count": 192,
        "measured_run_ids": run_ids,
    }


def episodes():
    rows = []
    ordinal = 0
    for cycle in range(16):
        rotation = ALTERNATIVES[cycle % 4:] + ALTERNATIVES[:cycle % 4]
        for scenario_ordinal in range(12):
            scenario = f"P{scenario_ordinal + 1:02}"
            for position, alternative in enumerate(rotation):
                alternative_ordinal = ALTERNATIVES.index(alternative)
                pair_id = f"calibration-next:cycle-{cycle}:repetition-{cycle}:{scenario}:{alternative}"
                for slot, mode in enumerate(mode_order(scenario_ordinal, alternative_ordinal, cycle)):
                    run_id = f"run-{ordinal:04}"
                    provenance = {
                        "provenance_schema_version": "dp00-pilot-provenance-v4",
                        "run_id": run_id,
                        "calibration_id": "calibration-next",
                        "source_sha": "frozen-sha",
                        "calibration_protocol_version": "dp00-calibration-protocol-v1",
                        "warmup_count": 1,
                        "measured_repetition_count": 16,
                        "order_policy": "DETERMINISTIC_CYCLIC_V1",
                        "measurement_population": True,
                        "calibration_execution_ordinal": ordinal,
                        "pair_id": pair_id,
                        "mode_order_slot": slot,
                        "instrumentation_mode": mode,
                        "alternative": alternative,
                        "scenario_id": scenario,
                        "order_cycle": cycle,
                        "order_slot": position,
                        "cycle_id": f"cycle-{cycle}",
                        "repetition_id": f"repetition-{cycle}",
                        "repetition_index": cycle,
                    }
                    rows.append(EpisodeEvidence(
                        raw_directory=None,
                        provenance=provenance,
                        scenario={},
                        canonical_events=(),
                        model_calls=(),
                        fixture_events=(),
                        actual=ActualSemanticFacts(),
                    ))
                    ordinal += 1
    return rows


def test_analyzer_accepts_complete_counterbalanced_schedule():
    rows = episodes()
    value = manifest([row.provenance["run_id"] for row in rows])
    assert validate_calibration_manifest(value) == value
    assert validate_calibration_schedule(value, rows) == []
    assert calibration_schedule_diagnostics(value, rows) == {
        "present": True,
        "protocol_version": "dp00-calibration-protocol-v1",
        "full_prewarm_cycles": 2,
        "measured_cycles": 16,
        "measured_executions": 1536,
        "measured_pairs": 768,
        "qa01_pairs": 192,
        "adaptive_stopping": False,
        "mode_order_policy": "SCENARIO_ALTERNATIVE_PARITY_INVERT_V1",
        "alternative_rotation_policy": "DETERMINISTIC_CYCLIC_V1",
        "repetition_policy": "FIXED_16_MEASURED_CYCLES_V1",
    }


def test_analyzer_rejects_non_counterbalanced_schedule():
    rows = episodes()
    value = manifest([row.provenance["run_id"] for row in rows])
    rows[0].provenance["instrumentation_mode"] = "MINIMAL"
    codes = {issue.code for issue in validate_calibration_schedule(value, rows)}
    assert "CALIBRATION_PAIR_ADJACENCY" in codes
    assert "CALIBRATION_MODE_BALANCE" in codes


def test_analyzer_rejects_incomplete_full_prewarm():
    rows = episodes()
    value = manifest([row.provenance["run_id"] for row in rows])
    value["prewarm_cycles"][1]["completed_path_count"] = 95
    codes = {issue.code for issue in validate_calibration_schedule(value, rows)}
    assert "CALIBRATION_PREWARM_INVALID" in codes


def test_analyzer_rejects_duplicate_prewarm_path_masking_missing_coverage():
    rows = episodes()
    value = manifest([row.provenance["run_id"] for row in rows])
    value["prewarm_cycles"][0]["covered_paths"][-1] = deepcopy(
        value["prewarm_cycles"][0]["covered_paths"][0]
    )
    codes = {issue.code for issue in validate_calibration_schedule(value, rows)}
    assert "CALIBRATION_PREWARM_INVALID" in codes


def test_analyzer_excludes_and_rejects_prewarm_observation_in_raw_population():
    rows = episodes()
    value = manifest([row.provenance["run_id"] for row in rows])
    leaked = deepcopy(rows[0])
    leaked.provenance["provenance_schema_version"] = "dp00-pilot-provenance-v3"
    leaked.provenance["measurement_population"] = False
    codes = {issue.code for issue in validate_calibration_schedule(value, [*rows, leaked])}
    assert "CALIBRATION_PREWARM_LEAK" in codes
