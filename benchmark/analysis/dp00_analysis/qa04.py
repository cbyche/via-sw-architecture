"""QA-04 logical calls-to-route-commit derivation and candidate aggregates."""

from __future__ import annotations

from collections import defaultdict
from statistics import fmean
from typing import Iterable

from .models import EpisodeEvidence


def final_required_route_commit(episode: EpisodeEvidence) -> dict | None:
    routes = list(episode.actual.execution_routes)
    required = episode.scenario["qa_eligibility"]["qa04"]["required_subgoal_ids"]
    if not required:
        root_routes = [route for route in routes if route.get("subgoal_id") is None]
        return root_routes[0] if len(root_routes) == 1 else None
    scoped_routes = [route for route in routes if route.get("subgoal_id") is not None]
    by_subgoal = {route["subgoal_id"]: route for route in scoped_routes}
    if len(scoped_routes) != len(required) or set(by_subgoal) != set(required):
        return None
    return max(by_subgoal.values(), key=lambda route: route["timestamp"])


def episode_calls_to_commit(episode: EpisodeEvidence) -> dict:
    routes = episode.actual.execution_routes
    final_commit = final_required_route_commit(episode)
    route_observed = final_commit is not None
    commit_ts = final_commit["timestamp"] if final_commit else None
    included = []
    mismatches = []
    for call in episode.model_calls:
        derived = bool(route_observed and call["call_class"] in {"ORCHESTRATION", "MIXED"} and call["decision_owner"] and call["semantic_responsibilities"] and call["completion"] <= commit_ts and not call["route_committed_before_call"])
        if derived:
            included.append(call["model_call_id"])
        if derived != call["qa04_primary_included"]:
            mismatches.append(call["model_call_id"])
    failures = list(episode.actual.failure_outcomes)
    expectation = episode.scenario["qa_eligibility"]["qa04"][
        "route_commit_expectation"
    ]
    contract_status = {
        ("REQUIRED", True): "ROUTE_COMMITTED",
        ("REQUIRED", False): "ROUTE_REQUIRED_NOT_COMMITTED",
        ("OPTIONAL", True): "ROUTE_COMMITTED",
        ("OPTIONAL", False): "ROUTE_OPTIONAL_NOT_COMMITTED",
        ("FORBIDDEN", True): "ROUTE_FORBIDDEN_BUT_COMMITTED",
        ("FORBIDDEN", False): "ROUTE_FORBIDDEN_NOT_COMMITTED",
    }[(expectation, route_observed)]
    return {
        "run_id": episode.provenance["run_id"], "scenario": episode.provenance["scenario_id"],
        "scenario_class": episode.scenario["scenario_class"], "alternative": episode.provenance["alternative"],
        "latency_profile": episode.provenance["latency_profile"]["profile_id"],
        "route_commit_observed": route_observed, "no_route_commit": not route_observed,
        "observed_subgoal_route_commits": sorted(
            route["subgoal_id"] for route in routes if route.get("subgoal_id") is not None
        ),
        "final_route_commit_event_id": final_commit["event_id"] if final_commit else None,
        "calls_to_route_commit": len(included) if route_observed else None, "included_model_call_ids": included,
        "raw_derived_inclusion_mismatches": mismatches,
        "model_calls_before_terminal_failure": len(episode.model_calls) if not route_observed else None,
        "model_calls_before_terminal_resolution": len(episode.model_calls),
        "terminal_failure_reason": failures[0] if failures else None,
        "route_commit_expectation": expectation,
        "route_contract_status": contract_status,
    }


def _contract_candidate(per_episode: list[dict]) -> dict:
    grouped: dict[tuple[str, str], list[dict]] = defaultdict(list)
    for item in per_episode:
        grouped[(item["latency_profile"], item["alternative"])].append(item)

    results = {}
    for key, group in sorted(grouped.items()):
        breakdown = {}
        for expectation in ("REQUIRED", "OPTIONAL", "FORBIDDEN"):
            rows = [
                item
                for item in group
                if item["route_commit_expectation"] == expectation
            ]
            breakdown[expectation] = {
                "total": len(rows),
                "committed": sum(item["route_commit_observed"] for item in rows),
                "not_committed": sum(not item["route_commit_observed"] for item in rows),
            }

        required = [
            item for item in group if item["route_commit_expectation"] == "REQUIRED"
        ]
        primary = [
            item
            for item in required
            if item["route_contract_status"] == "ROUTE_COMMITTED"
            and item["qa02_conformance"] is True
            and item["calls_to_route_commit"] is not None
        ]
        qualification_pass = bool(required) and len(primary) == len(required)
        values = [item["calls_to_route_commit"] for item in primary]
        results[f"profile-{key[0].lower()}:{key[1]}"] = {
            "policy_version": "qa04-route-contract-policy-v1",
            "contract_breakdown": breakdown,
            "required_population": len(required),
            "qualified_primary_observations": len(primary),
            "primary_comparability_qualification": (
                "PASS" if qualification_pass else "FAIL"
            ),
            "qualified_primary_mean": fmean(values) if qualification_pass else None,
            "required_route_not_committed": sum(
                item["route_contract_status"]
                == "ROUTE_REQUIRED_NOT_COMMITTED"
                for item in required
            ),
            "required_qa02_nonconformant": sum(
                item["qa02_conformance"] is not True for item in required
            ),
            "optional_stratum": [
                {
                    "run_id": item["run_id"],
                    "status": item["route_contract_status"],
                    "calls_to_route_commit": item["calls_to_route_commit"],
                }
                for item in group
                if item["route_commit_expectation"] == "OPTIONAL"
            ],
            "forbidden_route_violations": sum(
                item["route_contract_status"] == "ROUTE_FORBIDDEN_BUT_COMMITTED"
                for item in group
            ),
        }
    return results


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
    return {
        "per_episode": per_episode,
        "aggregates": aggregates,
        "no_route_commit": [x for x in per_episode if x["no_route_commit"]],
        "qa04_contract_diagnostics": _contract_candidate(per_episode),
    }
