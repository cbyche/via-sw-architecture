from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

ANALYSIS_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = ANALYSIS_ROOT.parents[1]
sys.path.insert(0, str(ANALYSIS_ROOT))


def correlation(turn="U1", task="T1", execution=None, result=None, clarification=None):
    return {"turn_id": turn, "task_id": task, "execution_id": execution, "dispatch_id": None, "result_id": result, "clarification_id": clarification}


def event(seq, value, *, emitter="ARCHITECTURE_UNDER_TEST", timestamp=None, scenario="P01", alternative="A", corr=None):
    return {
        "schema_version": "canonical-event-v2", "event_id": f"E{seq}", "run_id": "run-1",
        "episode_id": "episode-1", "scenario_id": scenario, "scenario_version": "v0.1",
        "alternative_id": alternative, "benchmark_version": "dp00-runtime-pilot-v0",
        "source_git_commit": "test-sha", "sequence_number": seq,
        "monotonic_timestamp": timestamp if timestamp is not None else seq * 10,
        "emitter": emitter, "product_correlation": corr or correlation(), "event": value,
    }


def route(kind="EXECUTOR_DIRECT", initial="ARGO", final="ARGO", chain=None):
    return {"route_kind": kind, "initial_executor_id": initial, "final_executor_id_if_known": final, "delegation_chain": chain or []}


def effect(effect_type="VOLUME_CHANGED", capability="volume.decrease", before=50, after=35, state="CHANGED", executor="ARGO", subject="system-volume"):
    return {"effect_type": effect_type, "capability_id": capability, "subject_id": subject, "target_id": None, "value": str(after) if after is not None else None, "before_value": before, "after_value": after, "state": state, "executor_id": executor, "authoritative_source": "OUTCOME_PROBE"}


def success_events(*, scenario="P01", alternative="A", before=50, after=35):
    return [
        event(1, "AcousticEos", emitter="INTERACTION_FIXTURE", timestamp=1_000_000, scenario=scenario, alternative=alternative),
        event(2, {"Architecture": "ProcessingStarted"}, timestamp=1_100_000, scenario=scenario, alternative=alternative),
        event(3, {"Architecture": {"TaskAssociated": {"task_relation": "NEW"}}}, timestamp=1_200_000, scenario=scenario, alternative=alternative),
        event(4, {"Architecture": {"RouteCommitted": {"route": route()}}}, timestamp=1_300_000, scenario=scenario, alternative=alternative),
        event(5, {"ExecutionStarted": {"invocation": {"capability_id": "volume.decrease", "executor_id": "ARGO"}}}, emitter="TOOL_FIXTURE", timestamp=1_400_000, scenario=scenario, alternative=alternative, corr=correlation(execution="X1")),
        event(6, {"Architecture": "ResultBound"}, timestamp=1_500_000, scenario=scenario, alternative=alternative, corr=correlation(execution="X1", result="R1")),
        event(7, {"UsefulOutcomeObserved": {"effect": effect(before=before, after=after)}}, emitter="OUTCOME_PROBE", timestamp=2_000_000, scenario=scenario, alternative=alternative, corr=correlation(execution="X1", result="R1")),
        event(8, "EpisodeCompleted", emitter="BENCHMARK", timestamp=2_100_000, scenario=scenario, alternative=alternative),
    ]


def model_call(seq=1, *, start=1_150_000, included=True, status="COMPLETED", candidate=None, scenario="P01", alternative="A", before=False):
    return {
        "schema_version": "model-call-v1", "model_call_id": f"M{seq}", "run_id": "run-1",
        "episode_id": "episode-1", "scenario_id": scenario, "alternative_id": alternative,
        "logical_sequence": seq, "attempt": seq, "decision_owner": "A.AgentRouter",
        "semantic_responsibilities": ["AGENT_SELECTION"], "status": status,
        "semantic_output_reference": None if candidate is None else {"value_kind": "EXECUTOR_CANDIDATE", "value": candidate},
        "route_committed_before_call": before, "route_committed_after_call": included,
        "logical_start": start, "first_output": start + 10 if status == "COMPLETED" else None,
        "completion": start + 20, "failure": None if status == "COMPLETED" else start + 20,
        "call_class": "ORCHESTRATION", "classification_reason": "ROUTE_SELECTION",
        "qa04_primary_included": included, "route_commit_event_id": "E4" if included else None,
    }


def provenance(*, scenario="P01", alternative="A", profile="Z", official=False, event_count=8):
    return {
        "provenance_schema_version": "dp00-pilot-provenance-v2", "run_id": "run-1", "campaign_id": "campaign-1" if official else None,
        "campaign_profile_sequence_index": 0 if official else None, "official": official, "source_git_commit": "test-sha", "working_tree_clean": True,
        "pilot_corpus_id": "DP00-PILOT-V0", "pilot_corpus_version": "v0.1", "alternative": alternative,
        "scenario_id": scenario, "scenario_version": "v0.1", "semantic_behavior_plan_id": f"SBP-{scenario}-v1", "semantic_behavior_plan_version": "v1",
        "latency_profile": {"profile_id": profile, "version": f"pilot-{profile.lower()}-v0", "model_delay_micros": 0, "agent_delay_micros": 0, "tool_delay_micros": 0, "calibration_status": "PILOT_TBD"},
        "warmup_count": 0, "measured_repetition_count": 1, "measurement_population": True, "order_policy": "COUNTERBALANCED_ROTATION_V1",
        "order_cycle": 0, "sequence_position": 0, "repetition_index": 0, "instrumentation_mode": "CAPTURE",
        "episode_elapsed_nanos": 2_100_000, "event_count": event_count, "capture_append_cost_nanos": 1,
        "model_profile": "dp00-base@v0", "prompt_profile": "pilot-v0-payload-v1", "cache_policy": "DISABLED",
        "rust_toolchain": "1.89.0", "rustc_version": "rustc test", "cargo_version": "cargo test", "target": "test-target",
        "build_profile": "qualification-or-release", "tokio_resolved_version": "1.53.1", "runtime_worker_policy": "tokio-current-thread-v0",
        "cargo_lock_identity": "fnv1a64-v1:test", "os": "macos", "machine_architecture": "aarch64",
        "canonical_event_schema_version": "canonical-event-v2", "model_call_schema_version": "model-call-v1",
    }


def write_run(path: Path, *, events=None, calls=None, prov=None):
    path.mkdir(parents=True)
    events = events or success_events()
    calls = calls if calls is not None else [model_call()]
    prov = prov or provenance(event_count=len(events))
    (path / "provenance.json").write_text(json.dumps(prov), encoding="utf-8")
    (path / "canonical-events.jsonl").write_text("".join(json.dumps(x) + "\n" for x in events), encoding="utf-8")
    (path / "model-calls.jsonl").write_text("".join(json.dumps(x) + "\n" for x in calls), encoding="utf-8")
    return path


@pytest.fixture
def raw_run(tmp_path):
    return write_run(tmp_path / "raw")
