"""Explicit CAPTURE/MINIMAL pair qualification for calibration evidence."""

from __future__ import annotations

from collections import defaultdict
from typing import Any, Iterable

from .models import EpisodeEvidence
from .qa01 import episode_ftol_nanos
from .qa04 import episode_calls_to_commit


def _without(value: dict[str, Any], *keys: str) -> dict[str, Any]:
    return {key: item for key, item in value.items() if key not in keys}


def _model_call_signature(call: dict[str, Any]) -> dict[str, Any]:
    return _without(
        call,
        "model_call_id",
        "run_id",
        "episode_id",
        "scenario_id",
        "alternative_id",
        "logical_start",
        "first_output",
        "completion",
        "failure",
        "qa04_primary_included",
        "route_commit_event_id",
    )


def _semantic_signature(episode: EpisodeEvidence) -> dict[str, Any]:
    actual = episode.actual
    return {
        "referent_bindings": list(actual.referent_bindings),
        "task_associations": list(actual.task_associations),
        "execution_invocations": [
            _without(item, "execution_id", "timestamp")
            for item in actual.execution_invocations
        ],
        "execution_routes": [
            _without(item, "event_id", "timestamp") for item in actual.execution_routes
        ],
        "clarifications": [
            _without(item, "request_timestamp", "resolve_timestamp")
            for item in actual.clarifications
        ],
        "result_bindings": [
            _without(item, "execution_id", "result_id") for item in actual.result_bindings
        ],
        "observable_effects": [
            _without(item, "timestamp") for item in actual.observable_effects
        ],
        "failure_outcomes": list(actual.failure_outcomes),
        "route_candidates": list(actual.route_candidates),
        "rejected_routes": list(actual.rejected_routes),
        "derived_predicates": sorted(actual.derived_predicates),
        "logical_model_calls": [
            _model_call_signature(call) for call in episode.model_calls
        ],
    }


def _shared_provenance(episode: EpisodeEvidence) -> dict[str, Any]:
    provenance = episode.provenance
    return {
        "calibration_id": provenance["calibration_id"],
        "cycle_id": provenance["cycle_id"],
        "pair_id": provenance["pair_id"],
        "order_slot": provenance["order_slot"],
        "repetition_id": provenance["repetition_id"],
        "alternative": provenance["alternative"],
        "scenario_id": provenance["scenario_id"],
        "scenario_version": provenance["scenario_version"],
        "semantic_behavior_plan_id": provenance["semantic_behavior_plan_id"],
        "semantic_behavior_plan_version": provenance["semantic_behavior_plan_version"],
        "latency_profile": provenance["latency_profile"],
        "warmup_count": provenance["warmup_count"],
        "measured_repetition_count": provenance["measured_repetition_count"],
        "measurement_population": provenance["measurement_population"],
        "order_policy": provenance["order_policy"],
        "order_cycle": provenance["order_cycle"],
        "sequence_position": provenance["sequence_position"],
        "repetition_index": provenance["repetition_index"],
        "pilot_corpus_id": provenance["pilot_corpus_id"],
        "pilot_corpus_version": provenance["pilot_corpus_version"],
        "source_sha": provenance["source_sha"],
        "model_profile": provenance["model_profile"],
        "prompt_profile": provenance["prompt_profile"],
        "cache_policy": provenance["cache_policy"],
        "rust_toolchain": provenance["rust_toolchain"],
        "rustc_version": provenance["rustc_version"],
        "cargo_version": provenance["cargo_version"],
        "target": provenance["target"],
        "build_profile": provenance["build_profile"],
        "tokio_resolved_version": provenance["tokio_resolved_version"],
        "runtime_worker_policy": provenance["runtime_worker_policy"],
        "cargo_lock_identity": provenance["cargo_lock_identity"],
        "os": provenance["os"],
        "machine_architecture": provenance["machine_architecture"],
        "canonical_event_schema_version": provenance["canonical_event_schema_version"],
        "model_call_schema_version": provenance["model_call_schema_version"],
    }


def derive_paired_calibration(
    episodes: Iterable[EpisodeEvidence],
    qa02: dict[str, Any],
) -> dict[str, Any]:
    calibration = [
        episode
        for episode in episodes
        if episode.provenance.get("calibration_id") is not None
    ]
    grouped: dict[str, list[EpisodeEvidence]] = defaultdict(list)
    for episode in calibration:
        grouped[episode.provenance["pair_id"]].append(episode)
    qa02_by_run = {
        row["run_id"]: row["exact_conformant"]
        for row in qa02["scenario_diagnostics"]
    }
    pairs = []
    missing_pairs = []
    duplicate_pairs = []
    semantic_mismatches = []
    provenance_mismatches = []
    for pair_id, rows in sorted(grouped.items()):
        by_mode: dict[str, list[EpisodeEvidence]] = defaultdict(list)
        for row in rows:
            by_mode[row.provenance["instrumentation_mode"]].append(row)
        missing_modes = sorted({"CAPTURE", "MINIMAL"} - set(by_mode))
        duplicate_modes = sorted(mode for mode, items in by_mode.items() if len(items) != 1)
        if missing_modes:
            missing_pairs.append({"pair_id": pair_id, "missing_modes": missing_modes})
        if duplicate_modes:
            duplicate_pairs.append({"pair_id": pair_id, "duplicate_modes": duplicate_modes})
        complete = not missing_modes and not duplicate_modes and len(rows) == 2
        item: dict[str, Any] = {
            "pair_id": pair_id,
            "complete": complete,
            "missing_modes": missing_modes,
            "duplicate_modes": duplicate_modes,
        }
        if complete:
            capture = by_mode["CAPTURE"][0]
            minimal = by_mode["MINIMAL"][0]
            provenance_match = _shared_provenance(capture) == _shared_provenance(minimal)
            mode_slots = {
                capture.provenance["mode_order_slot"],
                minimal.provenance["mode_order_slot"],
            }
            provenance_match = provenance_match and mode_slots == {0, 1}
            semantic_match = _semantic_signature(capture) == _semantic_signature(minimal)
            qa02_match = (
                qa02_by_run.get(capture.provenance["run_id"])
                == qa02_by_run.get(minimal.provenance["run_id"])
            )
            capture_qa04 = episode_calls_to_commit(capture)
            minimal_qa04 = episode_calls_to_commit(minimal)
            qa04_match = all(
                capture_qa04[key] == minimal_qa04[key]
                for key in (
                    "route_commit_observed",
                    "observed_subgoal_route_commits",
                    "calls_to_route_commit",
                    "route_contract_status",
                )
            )
            expectation = capture_qa04["route_commit_expectation"]
            qa04_qualified = {
                "REQUIRED": capture_qa04["route_contract_status"] == "ROUTE_COMMITTED",
                "OPTIONAL": capture_qa04["route_contract_status"] in {
                    "ROUTE_COMMITTED", "ROUTE_OPTIONAL_NOT_COMMITTED"
                },
                "FORBIDDEN": capture_qa04["route_contract_status"]
                == "ROUTE_FORBIDDEN_NOT_COMMITTED",
            }[expectation]
            capture_ftol = episode_ftol_nanos(capture)
            minimal_ftol = episode_ftol_nanos(minimal)
            ftol_boundaries_present = (
                capture_ftol is not None and minimal_ftol is not None
            ) or (
                not capture.scenario["qa_eligibility"]["qa01"]["eligible"]
                and not minimal.scenario["qa_eligibility"]["qa01"]["eligible"]
            )
            if not provenance_match:
                provenance_mismatches.append(pair_id)
            if not (semantic_match and qa02_match and qa04_match):
                semantic_mismatches.append(pair_id)
            item.update({
                "provenance_match": provenance_match,
                "semantic_match": semantic_match,
                "qa02_match": qa02_match,
                "qa04_match": qa04_match,
                "qa04_qualified": qa04_qualified,
                "ftol_boundaries_present": ftol_boundaries_present,
                "capture_ftol_nanos": capture_ftol,
                "minimal_ftol_nanos": minimal_ftol,
                "ftol_delta_nanos": (
                    capture_ftol - minimal_ftol
                    if capture_ftol is not None and minimal_ftol is not None
                    else None
                ),
                "capture_mode_order_slot": capture.provenance["mode_order_slot"],
                "minimal_mode_order_slot": minimal.provenance["mode_order_slot"],
                "qualified": (
                    provenance_match
                    and semantic_match
                    and qa02_match
                    and qa04_match
                    and qa04_qualified
                    and ftol_boundaries_present
                    and qa02_by_run.get(capture.provenance["run_id"]) is True
                ),
            })
        pairs.append(item)
    return {
        "measured_execution_count": len(calibration),
        "observed_pair_count": len(grouped),
        "complete_pair_count": sum(item["complete"] for item in pairs),
        "qualified_pair_count": sum(item.get("qualified", False) for item in pairs),
        "missing_pairs": missing_pairs,
        "duplicate_pairs": duplicate_pairs,
        "provenance_mismatches": provenance_mismatches,
        "semantic_mismatches": semantic_mismatches,
        "pairs": pairs,
    }
