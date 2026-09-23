"""Episode construction and invariant checks."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .models import EpisodeEvidence, ValidationIssue
from .semantics import reconstruct_actual
from .qa04 import final_required_route_commit
from .validation import cross_stream_issues


def build_episode(
    raw_directory: Path,
    provenance: dict[str, Any],
    scenario: dict[str, Any],
    events: list[dict[str, Any]],
    calls: list[dict[str, Any]],
    fixtures: list[dict[str, Any]],
    oracle: dict[str, Any] | None,
) -> EpisodeEvidence:
    actual = reconstruct_actual(events, calls)
    issues = cross_stream_issues(provenance, events, calls, fixtures)
    episode = EpisodeEvidence(
        raw_directory=raw_directory,
        provenance=provenance,
        scenario=scenario,
        canonical_events=tuple(events),
        model_calls=tuple(calls),
        fixture_events=tuple(fixtures),
        actual=actual,
        expected=oracle,
        integrity_issues=tuple(issues),
    )
    final_commit = final_required_route_commit(episode)
    if final_commit is not None:
        commit_id = final_commit["event_id"]
        commit_ts = final_commit["timestamp"]
        for call in calls:
            independently_included = (
                call["call_class"] in {"ORCHESTRATION", "MIXED"}
                and bool(call["decision_owner"])
                and bool(call["semantic_responsibilities"])
                and call["completion"] <= commit_ts
                and not call["route_committed_before_call"]
            )
            if independently_included != call["qa04_primary_included"]:
                issues.append(ValidationIssue(
                    "QA04_INCLUSION_MISMATCH",
                    f"{call['model_call_id']}: raw={call['qa04_primary_included']} derived={independently_included}",
                ))
            if call["route_commit_event_id"] is not None and call["route_commit_event_id"] != commit_id:
                issues.append(ValidationIssue("QA04_ROUTE_COMMIT_REFERENCE_MISMATCH", call["model_call_id"]))
    return EpisodeEvidence(
        raw_directory=episode.raw_directory,
        provenance=episode.provenance,
        scenario=episode.scenario,
        canonical_events=episode.canonical_events,
        model_calls=episode.model_calls,
        fixture_events=episode.fixture_events,
        actual=episode.actual,
        expected=episode.expected,
        integrity_issues=tuple(issues),
    )
