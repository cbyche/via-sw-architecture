"""QA-04 logical calls-to-route-commit derivation and candidate aggregates."""

from __future__ import annotations

from collections import defaultdict
from statistics import fmean
from typing import Iterable

from .models import EpisodeEvidence


def episode_calls_to_commit(episode: EpisodeEvidence) -> dict:
    routes = episode.actual.execution_routes
    route_observed = len(routes) == 1
    commit_ts = routes[0]["timestamp"] if route_observed else None
    included = []
    mismatches = []
    for call in episode.model_calls:
        derived = bool(route_observed and call["call_class"] in {"ORCHESTRATION", "MIXED"} and call["decision_owner"] and call["semantic_responsibilities"] and call["completion"] <= commit_ts and not call["route_committed_before_call"])
        if derived:
            included.append(call["model_call_id"])
        if derived != call["qa04_primary_included"]:
            mismatches.append(call["model_call_id"])
    failures = list(episode.actual.failure_outcomes)
    return {
        "run_id": episode.provenance["run_id"], "scenario": episode.provenance["scenario_id"],
        "scenario_class": episode.scenario["scenario_class"], "alternative": episode.provenance["alternative"],
        "latency_profile": episode.provenance["latency_profile"]["profile_id"],
        "route_commit_observed": route_observed, "no_route_commit": not route_observed,
        "calls_to_route_commit": len(included) if route_observed else None, "included_model_call_ids": included,
        "raw_derived_inclusion_mismatches": mismatches,
        "model_calls_before_terminal_failure": len(episode.model_calls) if not route_observed else None,
        "terminal_failure_reason": failures[0] if failures else None,
        "route_commit_expectation": episode.scenario["qa_eligibility"]["qa04"]["route_commit_expectation"],
    }


def derive_qa04(episodes: Iterable[EpisodeEvidence], qa02: dict | None = None) -> dict:
    qa02_rows = [] if qa02 is None else qa02["scenario_diagnostics"]
    qa02_by_run = {x["run_id"]: x["exact_conformant"] for x in qa02_rows}
    per_episode = []
    for episode in episodes:
        if not episode.provenance["measurement_population"]:
            continue
        if not episode.scenario["qa_eligibility"]["qa04"]["eligible"]:
            continue
        item = episode_calls_to_commit(episode)
        item["qa02_conformance"] = qa02_by_run.get(item["run_id"])
        per_episode.append(item)
    groups = defaultdict(list)
    for item in per_episode:
        if item["route_commit_observed"]:
            groups[(item["latency_profile"], item["alternative"])].append(item)
    aggregates = {}
    for key, group in sorted(groups.items()):
        values = [x["calls_to_route_commit"] for x in group]
        classes = defaultdict(list)
        for item in group:
            classes[item["scenario_class"]].append(item["calls_to_route_commit"])
        per_class = {}
        for scenario_class, class_values in sorted(classes.items()):
            class_mean = fmean(class_values)
            per_class[scenario_class] = {
                "sample_count": len(class_values), "class_mean": class_mean,
                "population_share": len(class_values) / len(values),
                "overall_contribution": sum(class_values) / len(values),
            }
        overall = fmean(values)
        macro = fmean(x["class_mean"] for x in per_class.values())
        aggregates[f"profile-{key[0].lower()}:{key[1]}"] = {
            "included_route_committed_episodes": len(values), "overall_mean": overall,
            "macro_average": macro, "absolute_difference": abs(overall - macro),
            "relative_difference": None if overall == 0 else abs(overall - macro) / abs(overall),
            "per_class": per_class,
        }
    return {"per_episode": per_episode, "aggregates": aggregates, "no_route_commit": [x for x in per_episode if x["no_route_commit"]]}
