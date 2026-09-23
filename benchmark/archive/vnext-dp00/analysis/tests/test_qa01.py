from dataclasses import replace
import json

import pytest

from conftest import REPO_ROOT, provenance, success_events
from dp00_analysis.loader import load_evidence
from dp00_analysis.qa01 import derive_qa01, linear_percentile, nearest_rank
from dp00_analysis.validation import (
    FROZEN_REALISTIC_PROFILES,
    REALISTIC_PROFILE_STATUS,
    REALISTIC_PROFILE_VERSION,
    validate_run_provenance,
)


def test_t01_known_ftol_and_t02_failed_exclusion(raw_run):
    episode = load_evidence(raw_run).episodes[0]
    failed_events = tuple(e for e in episode.canonical_events if e["event"] != "EpisodeCompleted" and "UsefulOutcomeObserved" not in (e["event"] if isinstance(e["event"], dict) else {}))
    failed_events += (dict(episode.canonical_events[-1], event={"EpisodeFailed": {"reason": "EXECUTION_FAILURE"}}),)
    failed = replace(episode, provenance=dict(episode.provenance, run_id="run-failed"), canonical_events=failed_events)
    result = derive_qa01([episode, failed])["profile-z:A"]
    assert result["counts"] == {"eligible_count": 2, "successful_count": 1, "failed_count": 1}
    assert result["statistics"]["min"] == 1_000_000
    assert result["statistics"]["milliseconds"]["mean"] == 1.0
    decomposition = result["dependency_decomposition"]
    assert decomposition["model_invocations"]["mean"] == 1
    assert decomposition["tool_executions"]["mean"] == 1
    assert decomposition["configured_total_dependency_budget"]["mean"] == 0
    assert decomposition["observed_framework_timer_residual"]["mean"] == 1_000_000


def test_explicit_percentile_estimators_and_small_n_difference():
    samples = [1, 2, 3, 4]
    assert nearest_rank(samples, .95) == 4
    assert linear_percentile(samples, .95) == 3.8499999999999996


def test_realistic_profile_ids_are_strictly_accepted():
    values = {
        "R1": (50_000, 50_000, 50_000),
        "R2": (100_000, 20_000, 20_000),
        "R3": (20_000, 100_000, 20_000),
        "R4": (20_000, 20_000, 100_000),
    }
    for profile_id, delays in values.items():
        record = provenance(profile=profile_id)
        record["latency_profile"]["version"] = "dp00-realistic-sensitivity-v1"
        (
            record["latency_profile"]["model_delay_micros"],
            record["latency_profile"]["agent_delay_micros"],
            record["latency_profile"]["tool_delay_micros"],
        ) = delays
        record["latency_profile"]["calibration_status"] = (
            "FROZEN_SYNTHETIC_SENSITIVITY_NOT_PRODUCTION_MEASUREMENT"
        )
        validate_run_provenance(record)

    record["latency_profile"]["tool_delay_micros"] += 1
    with pytest.raises(ValueError, match="frozen realistic profile mismatch"):
        validate_run_provenance(record)


def test_analysis_constants_match_machine_readable_profile_contract():
    contract = json.loads(
        (REPO_ROOT / "benchmark/contracts/dp00-realistic-execution-profiles-v1.json")
        .read_text(encoding="utf-8")
    )
    assert {
        profile["profile_id"]: (
            profile["model_delay_micros"],
            profile["agent_delay_micros"],
            profile["tool_delay_micros"],
        )
        for profile in contract["profiles"]
    } == FROZEN_REALISTIC_PROFILES
    assert {profile["version"] for profile in contract["profiles"]} == {
        REALISTIC_PROFILE_VERSION
    }
    assert REALISTIC_PROFILE_STATUS == (
        "FROZEN_SYNTHETIC_SENSITIVITY_NOT_PRODUCTION_MEASUREMENT"
    )
