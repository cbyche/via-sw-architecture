"""QA-01 FTOL derivation and explicit percentile estimators."""

from __future__ import annotations

import math
from collections import defaultdict
from statistics import fmean
from typing import Iterable

from .models import EpisodeEvidence
from .semantics import event_variant


def nearest_rank(values: Iterable[int | float], percentile: float) -> float:
    ordered = sorted(values)
    if not ordered:
        raise ValueError("percentile requires samples")
    rank = max(1, math.ceil(percentile * len(ordered)))
    return float(ordered[rank - 1])


def linear_percentile(values: Iterable[int | float], percentile: float) -> float:
    ordered = sorted(values)
    if not ordered:
        raise ValueError("percentile requires samples")
    index = (len(ordered) - 1) * percentile
    low, high = math.floor(index), math.ceil(index)
    if low == high:
        return float(ordered[low])
    return float(ordered[low] + (ordered[high] - ordered[low]) * (index - low))


def episode_ftol_nanos(episode: EpisodeEvidence) -> int | None:
    if not episode.scenario["qa_eligibility"]["qa01"]["eligible"]:
        return None
    variants = [(event_variant(e)[0], e) for e in episode.canonical_events]
    if not any(v == "EpisodeCompleted" for v, _ in variants):
        return None
    starts = [e["monotonic_timestamp"] for v, e in variants if v == "AcousticEos" and e["emitter"] == "INTERACTION_FIXTURE"]
    ends = [e["monotonic_timestamp"] for v, e in variants if v == "UsefulOutcomeObserved" and e["emitter"] == "OUTCOME_PROBE"]
    if len(starts) != 1 or not ends:
        raise ValueError("QA-01 eligible success lacks authoritative FTOL boundary")
    delta = min(ends) - starts[0]
    if delta < 0:
        raise ValueError("negative FTOL")
    return delta


def _stats(samples: list[int]) -> dict:
    if not samples:
        return {"sample_count": 0, "unit": "nanoseconds", "human_unit": "milliseconds", "nearest_rank": None, "linear": None}
    result = {
        "sample_count": len(samples), "unit": "nanoseconds", "human_unit": "milliseconds",
        "min": min(samples), "max": max(samples), "mean": fmean(samples),
        "nearest_rank": {f"p{p}": nearest_rank(samples, p / 100) for p in (50, 95, 99)},
        "linear": {f"p{p}": linear_percentile(samples, p / 100) for p in (50, 95, 99)},
    }
    nr, linear = result["nearest_rank"]["p95"], result["linear"]["p95"]
    result["small_n_sensitivity"] = {
        "sample_count": len(samples), "nearest_rank_p95": nr, "linear_p95": linear,
        "absolute_difference": abs(nr - linear),
        "relative_difference": None if nr == 0 else abs(nr - linear) / abs(nr),
    }
    result["milliseconds"] = {"min": result["min"] / 1_000_000, "max": result["max"] / 1_000_000, "mean": result["mean"] / 1_000_000}
    return result


def _count_stats(samples: list[int]) -> dict:
    if not samples:
        return {"sample_count": 0, "unit": "semantic invocations"}
    return {
        "sample_count": len(samples),
        "unit": "semantic invocations",
        "min": min(samples),
        "max": max(samples),
        "mean": fmean(samples),
    }


def derive_qa01(episodes: Iterable[EpisodeEvidence]) -> dict:
    groups: dict[tuple[str, str], list[EpisodeEvidence]] = defaultdict(list)
    for episode in episodes:
        if not episode.provenance["measurement_population"]:
            continue
        profile = episode.provenance["latency_profile"]["profile_id"]
        groups[(profile, episode.provenance["alternative"])].append(episode)
    output = {}
    for (profile, alternative), group in sorted(groups.items()):
        eligible = [e for e in group if e.scenario["qa_eligibility"]["qa01"]["eligible"]]
        values, per_scenario = [], defaultdict(list)
        model_counts, agent_counts, tool_counts = [], [], []
        model_budgets, agent_budgets, tool_budgets, total_budgets, residuals = (
            [], [], [], [], []
        )
        for episode in eligible:
            value = episode_ftol_nanos(episode)
            if value is not None:
                values.append(value)
                per_scenario[episode.provenance["scenario_id"]].append(value)
                model_count = len(episode.model_calls)
                agent_count = sum(
                    1
                    for event in episode.canonical_events
                    if event_variant(event)[0] == "ExecutionStarted"
                    and event["emitter"] == "AGENT_FIXTURE"
                )
                tool_count = sum(
                    1
                    for event in episode.canonical_events
                    if event_variant(event)[0] == "ExecutionStarted"
                    and event["emitter"] == "TOOL_FIXTURE"
                )
                profile_config = episode.provenance["latency_profile"]
                model_budget = model_count * profile_config["model_delay_micros"] * 1_000
                agent_budget = agent_count * profile_config["agent_delay_micros"] * 1_000
                tool_budget = tool_count * profile_config["tool_delay_micros"] * 1_000
                total_budget = model_budget + agent_budget + tool_budget
                model_counts.append(model_count)
                agent_counts.append(agent_count)
                tool_counts.append(tool_count)
                model_budgets.append(model_budget)
                agent_budgets.append(agent_budget)
                tool_budgets.append(tool_budget)
                total_budgets.append(total_budget)
                residuals.append(value - total_budget)
        key = f"profile-{profile.lower()}:{alternative}"
        output[key] = {
            "counts": {"eligible_count": len(eligible), "successful_count": len(values), "failed_count": len(eligible) - len(values)},
            "statistics": _stats(values),
            "percentile_methods": ["nearest-rank", "linear-interpolated-(n-1)"],
            "per_scenario": {s: _stats(v) for s, v in sorted(per_scenario.items())},
            "dependency_decomposition": {
                "budget_unit": "nanoseconds",
                "model_invocations": _count_stats(model_counts),
                "agent_delegations": _count_stats(agent_counts),
                "tool_executions": _count_stats(tool_counts),
                "configured_model_budget": _stats(model_budgets),
                "configured_agent_budget": _stats(agent_budgets),
                "configured_tool_budget": _stats(tool_budgets),
                "configured_total_dependency_budget": _stats(total_budgets),
                "observed_framework_timer_residual": _stats(residuals),
                "residual_interpretation": "FTOL minus configured constant dependency budget; includes framework/runtime and timer/scheduler effects",
            },
        }
    return output
