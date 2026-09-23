import json
from dataclasses import replace

from conftest import model_call, provenance, success_events, write_run
from dp00_analysis.diagnostics import derive_summary
from dp00_analysis.loader import load_evidence
from dp00_analysis.paired import derive_paired_calibration
from dp00_analysis.qa02 import derive_qa02
from dp00_analysis.validation import StrictValidationError, validate_run_provenance


def paired_provenance(mode, slot, run_id):
    value = provenance(event_count=8)
    value.update({
        "run_id": run_id,
        "instrumentation_mode": mode,
        "calibration_id": "calibration-1",
        "cycle_id": "cycle-1",
        "pair_id": "calibration-1:cycle-1:rotation-1:P01:A",
        "order_slot": 0,
        "mode_order_slot": slot,
        "repetition_id": "rotation-1",
    })
    return value


def paired_root(tmp_path, *, mismatch=False, minimal=True):
    root = tmp_path / "paired"
    modes = [("CAPTURE", 0, "capture")]
    if minimal:
        modes.append(("MINIMAL", 1, "minimal"))
    for mode, slot, run_id in modes:
        events = success_events()
        calls = [model_call()]
        for row in events + calls:
            row["run_id"] = run_id
        calls[0]["model_call_id"] = f"{run_id}:model:1"
        if mismatch and mode == "MINIMAL":
            events[6]["event"]["UsefulOutcomeObserved"]["effect"]["state"] = "MISMATCH"
        write_run(
            root / run_id,
            events=events,
            calls=calls,
            prov=paired_provenance(mode, slot, run_id),
        )
    return root


def test_explicit_capture_minimal_pair_is_complete_and_semantically_qualified(tmp_path):
    evidence = load_evidence(paired_root(tmp_path))
    assert evidence.validation.valid, evidence.validation.errors
    result = derive_summary(evidence)["paired_calibration"]
    assert result["measured_execution_count"] == 2
    assert result["complete_pair_count"] == 1
    assert result["qualified_pair_count"] == 1, {
        key: result["pairs"][0].get(key)
        for key in (
            "provenance_match", "semantic_match", "qa02_match", "qa04_match",
            "qa04_qualified", "ftol_boundaries_present", "qualified",
        )
    }
    assert result["semantic_mismatch_pairs"] == []
    assert result["qa02_mismatch_pairs"] == []
    assert result["qa04_mismatch_pairs"] == []
    assert result["provenance_mismatch_pairs"] == []
    assert result["pairs"][0]["ftol_boundaries_present"] is True
    assert result["pairs"][0]["capture_ftol_nanos"] == 1_000_000
    assert result["pairs"][0]["minimal_ftol_nanos"] == 1_000_000
    assert result["pairs"][0]["ftol_delta_nanos"] == 0


def test_missing_mode_and_semantic_mismatch_are_detected(tmp_path):
    missing = derive_summary(load_evidence(paired_root(tmp_path / "missing", minimal=False)))["paired_calibration"]
    assert missing["complete_pair_count"] == 0
    assert missing["incomplete_pairs"][0]["missing_modes"] == ["MINIMAL"]

    mismatch = derive_summary(load_evidence(paired_root(tmp_path / "mismatch", mismatch=True)))["paired_calibration"]
    assert mismatch["complete_pair_count"] == 1
    assert mismatch["qualified_pair_count"] == 0
    assert mismatch["semantic_mismatch_pairs"]
    assert mismatch["qa02_mismatch_pairs"] == []


def test_duplicate_mode_is_detected(tmp_path):
    root = paired_root(tmp_path)
    events = success_events()
    calls = [model_call()]
    for row in events + calls:
        row["run_id"] = "capture-duplicate"
    calls[0]["model_call_id"] = "capture-duplicate:model:1"
    write_run(
        root / "capture-duplicate",
        events=events,
        calls=calls,
        prov=paired_provenance("CAPTURE", 0, "capture-duplicate"),
    )

    result = derive_summary(load_evidence(root))["paired_calibration"]
    assert result["complete_pair_count"] == 0
    assert result["duplicate_mode_pairs"][0] == {
        "pair_id": "calibration-1:cycle-1:rotation-1:P01:A",
        "duplicate_modes": ["CAPTURE"],
    }


def test_source_sha_and_paired_provenance_are_strict():
    value = paired_provenance("CAPTURE", 0, "capture")
    assert validate_run_provenance(value) == value
    value["source_sha"] = "wrong"
    try:
        validate_run_provenance(value)
    except StrictValidationError as error:
        assert "source_sha/source_git_commit mismatch" in str(error)
    else:
        raise AssertionError("source mismatch accepted")


def _derive_modified_pair(tmp_path, mutate):
    evidence = load_evidence(paired_root(tmp_path))
    capture, minimal = sorted(
        evidence.episodes,
        key=lambda item: item.provenance["instrumentation_mode"],
    )
    assert capture.provenance["instrumentation_mode"] == "CAPTURE"
    minimal = mutate(minimal)
    episodes = (capture, minimal)
    qa02 = derive_qa02(episodes, evidence.coverage_map)
    return derive_paired_calibration(episodes, qa02)


def test_actual_semantic_mismatch_is_classified_only_as_semantic(tmp_path):
    def mutate(episode):
        association = dict(episode.actual.task_associations[0], task_relation="FOLLOW_UP")
        return replace(
            episode,
            actual=replace(episode.actual, task_associations=(association,)),
        )

    result = _derive_modified_pair(tmp_path, mutate)
    assert result["semantic_mismatch_pairs"]
    assert result["qa02_mismatch_pairs"] == []
    assert result["qa04_mismatch_pairs"] == []
    assert result["provenance_mismatch_pairs"] == []


def test_qa02_only_mismatch_is_classified_only_as_qa02(tmp_path):
    result = _derive_modified_pair(
        tmp_path,
        lambda episode: replace(
            episode,
            provenance=dict(episode.provenance, measurement_spine_event_count=7),
        ),
    )
    assert result["semantic_mismatch_pairs"] == []
    assert result["qa02_mismatch_pairs"]
    assert result["qa04_mismatch_pairs"] == []
    assert result["provenance_mismatch_pairs"] == []


def test_qa04_only_mismatch_is_classified_only_as_qa04(tmp_path):
    def mutate(episode):
        route = dict(episode.actual.execution_routes[0], timestamp=1_000_000)
        return replace(
            episode,
            actual=replace(episode.actual, execution_routes=(route,)),
        )

    result = _derive_modified_pair(tmp_path, mutate)
    assert result["semantic_mismatch_pairs"] == []
    assert result["qa02_mismatch_pairs"] == []
    assert result["qa04_mismatch_pairs"]
    assert result["provenance_mismatch_pairs"] == []


def test_provenance_only_mismatch_is_classified_only_as_provenance(tmp_path):
    result = _derive_modified_pair(
        tmp_path,
        lambda episode: replace(
            episode,
            provenance=dict(episode.provenance, source_sha="different-source"),
        ),
    )
    assert result["semantic_mismatch_pairs"] == []
    assert result["qa02_mismatch_pairs"] == []
    assert result["qa04_mismatch_pairs"] == []
    assert result["provenance_mismatch_pairs"]


def test_multi_cause_rejection_preserves_every_reason_and_is_deterministic(tmp_path):
    def mutate(episode):
        association = dict(episode.actual.task_associations[0], task_relation="FOLLOW_UP")
        route = dict(episode.actual.execution_routes[0], timestamp=1_000_000)
        return replace(
            episode,
            provenance=dict(
                episode.provenance,
                source_sha="different-source",
                measurement_spine_event_count=7,
            ),
            actual=replace(
                episode.actual,
                task_associations=(association,),
                execution_routes=(route,),
            ),
        )

    result = _derive_modified_pair(tmp_path, mutate)
    pair_id = result["rejected_pairs"][0]
    assert {
        "PROVENANCE_MISMATCH",
        "SEMANTIC_MISMATCH",
        "QA02_MISMATCH",
        "QA04_MISMATCH",
    } <= set(result["rejection_reasons_by_pair"][pair_id])
    repeated = _derive_modified_pair(tmp_path / "repeated", mutate)
    assert json.dumps(result, sort_keys=True) == json.dumps(repeated, sort_keys=True)
