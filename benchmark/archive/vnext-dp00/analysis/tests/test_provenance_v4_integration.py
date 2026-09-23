from __future__ import annotations

import shutil
from dataclasses import replace
from pathlib import Path

import pytest

from conftest import REPO_ROOT, event, model_call, provenance, success_events, write_run
from dp00_analysis.acceptance import derive_calibration_acceptance
from dp00_analysis.diagnostics import derive_summary
from dp00_analysis.loader import load_evidence
from dp00_analysis.qa02 import derive_qa02, supports_measurement_spine
from dp00_analysis.qa04 import episode_calls_to_commit
from dp00_analysis.validation import StrictValidationError, validate_run_provenance


REJECTED_ROOT = (
    REPO_ROOT
    / "results/raw/pilot-v0/dp00-calibration-20260911T050356Z-28aedd0"
)
P12_CAPTURE = (
    "calibration-1789103045924899000-measured-cycle-0-P12-A-capture-89"
)
P12_MINIMAL = (
    "calibration-1789103045924899000-measured-cycle-0-P12-A-minimal-88"
)


@pytest.mark.parametrize(
    ("version", "mode", "expected"),
    [
        ("dp00-pilot-provenance-v3", "CAPTURE", True),
        ("dp00-pilot-provenance-v3", "MINIMAL", True),
        ("dp00-pilot-provenance-v4", "CAPTURE", True),
        ("dp00-pilot-provenance-v4", "MINIMAL", True),
        ("unknown-provenance", "CAPTURE", False),
        ("unknown-provenance", "MINIMAL", False),
    ],
)
def test_measurement_spine_support_is_explicit(version, mode, expected):
    assert supports_measurement_spine(version, mode) is expected


def test_unknown_provenance_is_strictly_rejected():
    value = provenance()
    value["provenance_schema_version"] = "unknown-provenance"
    with pytest.raises(StrictValidationError, match="unsupported schema version"):
        validate_run_provenance(value)


def v4_provenance(mode: str, slot: int, run_id: str, *, event_count: int) -> dict:
    value = provenance(event_count=event_count)
    value.update({
        "provenance_schema_version": "dp00-pilot-provenance-v4",
        "run_id": run_id,
        "instrumentation_mode": mode,
        "calibration_id": "synthetic-v4-integration",
        "cycle_id": "cycle-0",
        "pair_id": "synthetic-v4-integration:cycle-0:repetition-0:P01:A",
        "order_slot": 0,
        "mode_order_slot": slot,
        "repetition_id": "repetition-0",
        "calibration_protocol_version": "dp00-calibration-protocol-v1",
        "calibration_execution_ordinal": slot,
        "attempted_event_count": 10,
        "measurement_spine_event_count": 8,
        "measured_repetition_count": 16,
    })
    return value


def v4_generic_pair_root(tmp_path: Path) -> Path:
    root = tmp_path / "v4-pair"
    for mode, slot, run_id in (
        ("CAPTURE", 0, "v4-capture"),
        ("MINIMAL", 1, "v4-minimal"),
    ):
        events = success_events()
        if mode == "CAPTURE":
            events[2:2] = [
                event(0, "ModelGenerationStarted", emitter="MODEL_FIXTURE", timestamp=1_150_000),
                event(0, "ModelGenerationCompleted", emitter="MODEL_FIXTURE", timestamp=1_175_000),
            ]
        calls = [model_call()]
        for sequence, row in enumerate(events):
            row["sequence_number"] = sequence
            row["event_id"] = f"{run_id}:event:{sequence}"
            row["run_id"] = run_id
            row["episode_id"] = f"{run_id}:episode"
        for row in calls:
            row["run_id"] = run_id
            row["episode_id"] = f"{run_id}:episode"
            row["model_call_id"] = f"{run_id}:model:1"
            row["route_commit_event_id"] = f"{run_id}:event:{5 if mode == 'CAPTURE' else 3}"
        write_run(
            root / run_id,
            events=events,
            calls=calls,
            prov=v4_provenance(mode, slot, run_id, event_count=len(events)),
        )
    return root


def test_v4_generic_pair_runs_loader_qa02_qa04_and_pair_qualification(tmp_path):
    evidence = load_evidence(v4_generic_pair_root(tmp_path))
    assert len(evidence.episodes) == 2
    assert {issue.code for issue in evidence.validation.errors} == {
        "CALIBRATION_MANIFEST_MISSING"
    }
    summary = derive_summary(evidence)
    assert summary["qa02"]["numerator"] == summary["qa02"]["denominator"] == 2
    assert len(summary["qa04"]["per_episode"]) == 2
    pair = summary["paired_calibration"]
    assert pair["qualified_pair_count"] == 1
    assert pair["semantic_mismatch_pairs"] == []
    assert pair["qa02_mismatch_pairs"] == []
    assert pair["qa04_mismatch_pairs"] == []
    assert pair["provenance_mismatch_pairs"] == []
    assert pair["pairs"][0]["ftol_boundaries_present"] is True
    assert pair["pairs"][0]["capture_ftol_nanos"] == 1_000_000
    assert pair["pairs"][0]["minimal_ftol_nanos"] == 1_000_000


def test_v4_minimal_closed_world_absence_passes_without_diagnostic_events(tmp_path):
    evidence = load_evidence(v4_generic_pair_root(tmp_path))
    minimal = next(
        item for item in evidence.episodes
        if item.provenance["instrumentation_mode"] == "MINIMAL"
    )
    assert all(
        not isinstance(row["event"], str)
        or row["event"] not in {"ModelGenerationStarted", "ModelGenerationCompleted"}
        for row in minimal.canonical_events
    )
    result = derive_qa02([minimal], evidence.coverage_map)
    forbidden = next(
        item
        for item in result["scenario_diagnostics"][0]["constraints"]
        if item["constraint_id"] == "P01-FORBID-AGENT"
    )
    assert forbidden["status"] == "PASS"
    assert forbidden["decision_basis"] == "CLOSED_WORLD_ABSENCE"


def test_v4_minimal_actual_missing_lifecycle_evidence_still_fails(tmp_path):
    root = v4_generic_pair_root(tmp_path)
    evidence = load_evidence(root)
    minimal = next(
        item for item in evidence.episodes
        if item.provenance["instrumentation_mode"] == "MINIMAL"
    )
    incomplete = replace(
        minimal,
        canonical_events=minimal.canonical_events[:-1],
        integrity_issues=(minimal.integrity_issues or ()) + (object(),),
    )
    result = derive_qa02([incomplete], evidence.coverage_map)
    forbidden = next(
        item
        for item in result["scenario_diagnostics"][0]["constraints"]
        if item["constraint_id"] == "P01-FORBID-AGENT"
    )
    assert forbidden["status"] == "MISSING_ACTUAL_EVIDENCE"
    assert forbidden["decision_basis"] == "OPEN_WORLD_ABSENCE"


def p12_pair_root(tmp_path: Path) -> Path:
    root = tmp_path / "p12-v4-pair"
    root.mkdir()
    for directory in (P12_CAPTURE, P12_MINIMAL):
        shutil.copytree(REJECTED_ROOT / directory, root / directory)
    return root


@pytest.mark.parametrize("mode", ["CAPTURE", "MINIMAL"])
def test_v4_p12_mode_full_loader_qa02_qa04_integration(tmp_path, mode):
    evidence = load_evidence(p12_pair_root(tmp_path))
    episode = next(
        item for item in evidence.episodes
        if item.provenance["instrumentation_mode"] == mode
    )
    qa02 = derive_qa02([episode], evidence.coverage_map)
    assert qa02["numerator"] == qa02["denominator"] == 1
    routes = sorted(episode.actual.execution_routes, key=lambda item: item["timestamp"])
    assert [route["subgoal_id"] for route in routes] == ["S1", "S2"]
    assert routes[0]["parent_task_id"] == routes[1]["parent_task_id"]
    assert routes[0]["child_task_id"] != routes[1]["child_task_id"]
    assert routes[0]["event_id"] != routes[1]["event_id"]
    assert {effect["effect_type"] for effect in episode.actual.observable_effects} == {
        "MEDIA_PAUSED", "DOWNLOADS_ORGANIZED"
    }
    assert min(item["timestamp"] for item in episode.actual.execution_invocations) > routes[1]["timestamp"]
    qa04 = episode_calls_to_commit(episode)
    assert qa04["observed_subgoal_route_commits"] == ["S1", "S2"]
    assert qa04["final_route_commit_event_id"] == routes[1]["event_id"]
    assert qa04["route_contract_status"] == "ROUTE_COMMITTED"


def test_v4_p12_pair_qualifies_end_to_end(tmp_path):
    evidence = load_evidence(p12_pair_root(tmp_path))
    summary = derive_summary(evidence)
    assert summary["qa02"]["numerator"] == summary["qa02"]["denominator"] == 2
    pair = summary["paired_calibration"]
    assert pair["qualified_pair_count"] == 1
    assert pair["semantic_mismatch_pairs"] == []
    assert pair["qa02_mismatch_pairs"] == []
    assert pair["qa04_mismatch_pairs"] == []
    assert pair["provenance_mismatch_pairs"] == []


def test_rejected_pair_cannot_enter_acceptance_population(tmp_path):
    evidence = load_evidence(v4_generic_pair_root(tmp_path))
    capture, minimal = evidence.episodes
    minimal = replace(
        minimal,
        provenance=dict(minimal.provenance, measurement_spine_event_count=7),
    )
    qa02 = derive_qa02((capture, minimal), evidence.coverage_map)
    from dp00_analysis.paired import derive_paired_calibration

    paired = derive_paired_calibration((capture, minimal), qa02)
    result = derive_calibration_acceptance(
        paired,
        {"measured_cycle_count": 16},
        schedule_valid=True,
        bootstrap_resamples=1,
    )
    assert paired["qualified_pair_count"] == 0
    assert result["accepted_input"] is False
    assert "MALFORMED_PAIR_QA02_MISMATCH_PAIRS" in result["rejection_reasons"]
