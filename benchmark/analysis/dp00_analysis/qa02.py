"""QA-02 exact conformance and 36×4 evidence-coverage diagnostics."""

from __future__ import annotations

from collections import defaultdict
from typing import Any, Iterable

from .models import ActualSemanticFacts, EpisodeEvidence
from .semantics import event_variant
from .validation import CANONICAL_EVENT_VERSION


def _deduplicate(values: list[Any]) -> list[Any]:
    result: list[Any] = []
    for value in values:
        if value not in result:
            result.append(value)
    return result


def _actual_values(actual: ActualSemanticFacts, dimension: str) -> tuple[list[Any], bool]:
    if dimension == "referent":
        values = [
            {"role": item["role"], "object_id": item["object_id"]}
            for item in actual.referent_bindings
        ]
        values.extend(
            {"role": "source", "object_id": effect["subject_id"]}
            for effect in actual.observable_effects
            if effect.get("effect_type") == "DOCUMENT_OPENED"
            and effect.get("subject_id")
        )
        values = _deduplicate(values)
        return values, bool(values)
    if dimension == "task_association":
        values: list[Any] = []
        for item in actual.task_associations:
            values.extend(
                [
                    item["task_relation"],
                    {"task_relation": item["task_relation"]},
                    {"task_id": item["task_id"]},
                ]
            )
        return values, bool(actual.task_associations)
    if dimension == "result_binding":
        return [
            item["task_id"] if item["task_id"] is not None else "UNBOUND"
            for item in actual.result_bindings
        ], bool(actual.result_bindings)
    if dimension == "route_commit":
        values = ["ROUTE_COMMITTED"] if actual.execution_routes else []
        return values, bool(values)
    if dimension == "clarification":
        values = [
            {
                "requested": item.get("requested", False),
                "resolved": item.get("resolved", False),
                "request_turn_id": item.get("request_turn_id"),
                "response_turn_id": item.get("response_turn_id"),
            }
            for item in actual.clarifications
        ]
        return values, bool(values)
    if dimension == "execution_path":
        values = [route["identity"] for route in actual.execution_routes]
        if not values:
            for invocation in actual.execution_invocations:
                executor = invocation["executor_id"]
                if executor.startswith("VIA_LOCAL"):
                    values.append("LOCAL_DIRECT:VIA_FAST")
                else:
                    values.append(f"EXECUTOR_DIRECT:{executor}")
        values = _deduplicate(values)
        return values, bool(values)
    if dimension == "execution_owner":
        owners = []
        for route in actual.execution_routes:
            initial = route["initial_executor_id"]
            owners.append("VIA_FAST" if initial.startswith("VIA_LOCAL") else initial)
        if not owners:
            owners.extend(
                "VIA_FAST"
                if item["executor_id"].startswith("VIA_LOCAL")
                else item["executor_id"]
                for item in actual.execution_invocations
            )
        owners = _deduplicate(owners)
        return owners, bool(owners)
    if dimension == "delegated_agent":
        values = []
        for route in actual.execution_routes:
            values.extend(route["delegation_chain"])
            final = route["final_executor_id_if_known"]
            if final:
                values.append(final)
            values.append(route["initial_executor_id"])
        values.extend(item["executor_id"] for item in actual.execution_invocations)
        values = _deduplicate(values)
        return values, bool(values)
    if dimension == "observable_effect":
        values: list[Any] = []
        for effect in actual.observable_effects:
            values.append(
                {
                    "capability_id": effect["capability_id"],
                    "state": (
                        "DECREASED"
                        if "DECREASED" in actual.derived_predicates
                        else effect["state"]
                    ),
                }
            )
            values.append(
                {"object_id": effect["subject_id"], "state": effect["state"]}
            )
        values.extend(sorted(actual.derived_predicates))
        return values, bool(values)
    if dimension == "failure_outcome":
        values = sorted(actual.derived_predicates)
        return _deduplicate(values), bool(values)
    if dimension == "routing":
        values = sorted(actual.derived_predicates)
        values.extend(route["identity"] for route in actual.execution_routes)
        return _deduplicate(values), bool(values)
    return [], False


def _value_matches(expected: Any, value: Any) -> bool:
    if isinstance(expected, dict) and isinstance(value, dict):
        return all(
            value.get(key) == expected_value
            for key, expected_value in expected.items()
        )
    return expected == value


def evaluate_constraint(
    constraint: dict[str, Any],
    actual: ActualSemanticFacts,
    *,
    closed_world_complete: bool = False,
    closed_world_conditions: dict[str, bool] | None = None,
    terminal_failure_authoritative: bool = True,
) -> dict[str, Any]:
    """Evaluate Actual independently of Oracle fallback or topology identity."""
    values, resolvable = _actual_values(actual, constraint["dimension"])
    expected = constraint["expected"]
    operator = constraint["operator"]
    if operator in {"equals", "predicate"}:
        matched = any(_value_matches(expected, value) for value in values)
    else:
        matched = any(
            any(_value_matches(option, value) for option in expected)
            for value in values
        )

    common = {
        "constraint_id": constraint["constraint_id"],
        "dimension": constraint["dimension"],
        "constraint_set": constraint["constraint_set"],
        "operator": operator,
        "expected": expected,
    }
    if constraint["constraint_set"] == "FORBIDDEN":
        if matched:
            return {
                **common,
                "status": "FAIL",
                "evidence_complete": True,
                "decision_basis": "POSITIVE_FORBIDDEN_EVIDENCE",
                "actual": values,
            }
        if closed_world_complete:
            return {
                **common,
                "status": "PASS",
                "evidence_complete": True,
                "decision_basis": "CLOSED_WORLD_ABSENCE",
                "closed_world_conditions": closed_world_conditions or {},
                "actual": values,
            }
        return {
            **common,
            "status": "MISSING_ACTUAL_EVIDENCE",
            "evidence_complete": False,
            "decision_basis": "OPEN_WORLD_ABSENCE",
            "closed_world_conditions": closed_world_conditions or {},
            "actual": values,
        }

    if not resolvable:
        if actual.failure_outcomes and terminal_failure_authoritative:
            return {
                **common,
                "status": "FAIL",
                "evidence_complete": True,
                "decision_basis": "TERMINAL_FAILURE_NON_ACHIEVEMENT",
                "actual": [
                    {"terminal_failure": reason}
                    for reason in actual.failure_outcomes
                ],
            }
        return {
            **common,
            "status": "MISSING_ACTUAL_EVIDENCE",
            "evidence_complete": False,
            "decision_basis": "NO_AUTHORITATIVE_POSITIVE_EVIDENCE",
            "actual": values,
        }

    if (
        constraint["constraint_set"] == "ALLOWED"
        and operator == "in"
        and constraint["dimension"] != "failure_outcome"
    ):
        passed = bool(values) and all(
            any(_value_matches(option, value) for option in expected)
            for value in values
        )
    else:
        passed = matched
    return {
        **common,
        "status": "PASS" if passed else "FAIL",
        "evidence_complete": True,
        "decision_basis": "AUTHORITATIVE_SEMANTIC_VALUES",
        "actual": values,
    }


def _closed_world_conditions(
    episode: EpisodeEvidence, cell: dict[str, Any]
) -> dict[str, bool]:
    variants = [event_variant(event)[0] for event in episode.canonical_events]
    terminals = [
        variant
        for variant in variants
        if variant in {"EpisodeCompleted", "EpisodeFailed"}
    ]
    absence = cell["absence_semantics"]
    return {
        "TERMINAL_EVENT_UNIQUE_AND_FINAL": len(terminals) == 1
        and bool(variants)
        and variants[-1] in {"EpisodeCompleted", "EpisodeFailed"},
        "EVENT_COUNT_MATCHES_CAPTURED_STREAM": episode.provenance["event_count"]
        == len(episode.canonical_events),
        "CAPTURE_MODE_AND_SCHEMA_RECOGNIZED": episode.provenance[
            "instrumentation_mode"
        ]
        == "CAPTURE"
        and episode.provenance["canonical_event_schema_version"]
        == CANONICAL_EVENT_VERSION,
        "RELEVANT_BOUNDARY_DECLARED": bool(absence.get("relevant_stream")),
        "NO_EPISODE_INTEGRITY_ERROR": not episode.integrity_issues,
    }


def _coverage_cells(
    coverage_map: dict[str, Any],
) -> dict[tuple[str, str], dict[str, Any]]:
    return {
        (cell["constraint_id"], cell["alternative"]): cell
        for cell in coverage_map["cells"]
    }


def _evaluate_episode_constraint(
    constraint: dict[str, Any],
    episode: EpisodeEvidence,
    cell: dict[str, Any],
) -> dict[str, Any]:
    conditions = _closed_world_conditions(episode, cell)
    closed_world = cell["absence_semantics"]["allowed"] and all(
        conditions.values()
    )
    result = evaluate_constraint(
        constraint,
        episode.actual,
        closed_world_complete=closed_world,
        closed_world_conditions=conditions,
        terminal_failure_authoritative=all(
            conditions[name]
            for name in (
                "TERMINAL_EVENT_UNIQUE_AND_FINAL",
                "EVENT_COUNT_MATCHES_CAPTURED_STREAM",
                "CAPTURE_MODE_AND_SCHEMA_RECOGNIZED",
                "NO_EPISODE_INTEGRITY_ERROR",
            )
        ),
    )
    result["coverage_cell"] = {
        "alternative": cell["alternative"],
        "derivation_rule_id": cell["derivation_rule_id"],
        "absence_rule_id": cell["absence_semantics"]["rule_id"],
    }
    return result


def coverage_diagnostics(
    episode: EpisodeEvidence, coverage_map: dict[str, Any]
) -> list[dict[str, Any]]:
    scenario = episode.provenance["scenario_id"]
    alternative = episode.provenance["alternative"]
    constraints = (
        {
            constraint["constraint_id"]: constraint
            for constraint in episode.expected["constraint_manifest"]["constraints"]
        }
        if episode.expected
        else {}
    )
    cells = _coverage_cells(coverage_map)
    results = []
    for constraint_id, constraint in constraints.items():
        cell = cells.get((constraint_id, alternative))
        if cell is None or cell["scenario_id"] != scenario:
            results.append(
                {
                    "constraint_id": constraint_id,
                    "alternative": alternative,
                    "status": "UNEVALUABLE",
                    "reason": "CONSTRAINT_ALTERNATIVE_CELL_MISSING",
                }
            )
            continue
        evaluated = _evaluate_episode_constraint(constraint, episode, cell)
        results.append(
            {
                "constraint_id": constraint_id,
                "alternative": alternative,
                "expected_source_resolvable": True,
                "actual_source_resolvable": bool(
                    cell["actual_evidence_strategies"]
                ),
                "authority_recognized": all(
                    bool(strategy["authority"])
                    for strategy in cell["actual_evidence_strategies"]
                ),
                "deterministic_derivation_available": bool(
                    cell["derivation_rule_id"]
                ),
                "actual_raw_evidence_present": evaluated["evidence_complete"],
                "status": evaluated["status"],
                "decision_basis": evaluated["decision_basis"],
                "actual": evaluated["actual"],
            }
        )
    return results


def _derive_coverage_gate(
    episodes: list[EpisodeEvidence], coverage_map: dict[str, Any]
) -> dict[str, Any]:
    by_path_profile = {
        (
            episode.provenance["scenario_id"],
            episode.provenance["alternative"],
            episode.provenance["latency_profile"]["profile_id"],
        ): episode
        for episode in episodes
        if episode.provenance["measurement_population"]
    }
    statuses = {
        "PASS": 0,
        "FAIL": 0,
        "MISSING_ACTUAL_EVIDENCE": 0,
        "UNEVALUABLE": 0,
    }
    cells = []
    invariant = True
    for cell in coverage_map["cells"]:
        path_key = (cell["scenario_id"], cell["alternative"])
        episode = by_path_profile.get((*path_key, "Z")) or by_path_profile.get(
            (*path_key, "C")
        )
        if episode is None or episode.expected is None:
            status = "UNEVALUABLE"
            result = {"decision_basis": "REPRESENTATIVE_TRACE_MISSING"}
        else:
            constraints = {
                constraint["constraint_id"]: constraint
                for constraint in episode.expected["constraint_manifest"]["constraints"]
            }
            constraint = constraints.get(cell["constraint_id"])
            if constraint is None:
                status = "UNEVALUABLE"
                result = {"decision_basis": "ORACLE_CONSTRAINT_MISSING"}
            else:
                result = _evaluate_episode_constraint(constraint, episode, cell)
                status = result["status"]
                peer = (
                    by_path_profile.get((*path_key, "C"))
                    if episode.provenance["latency_profile"]["profile_id"] == "Z"
                    else None
                )
                if peer is not None:
                    peer_status = _evaluate_episode_constraint(
                        constraint, peer, cell
                    )["status"]
                    invariant = invariant and peer_status == status
        statuses[status] = statuses.get(status, 0) + 1
        cells.append(
            {
                "scenario": cell["scenario_id"],
                "constraint_id": cell["constraint_id"],
                "alternative": cell["alternative"],
                "status": status,
                "decision_basis": result["decision_basis"],
            }
        )
    expected = coverage_map["expected_coverage_cells"]
    evaluated = statuses["PASS"] + statuses["FAIL"]
    return {
        "coverage_manifest_version": coverage_map["coverage_manifest_version"],
        "constraint_count": coverage_map["constraint_count"],
        "alternatives": len(coverage_map["alternatives"]),
        "expected_cells": expected,
        "evaluated_cells": evaluated,
        "pass_correctness_cells": statuses["PASS"],
        "fail_correctness_cells": statuses["FAIL"],
        "missing_cells": statuses["MISSING_ACTUAL_EVIDENCE"],
        "unevaluable_cells": statuses["UNEVALUABLE"],
        "profile_semantic_status_invariant": invariant,
        "status": (
            "PASS"
            if evaluated == expected
            and statuses["MISSING_ACTUAL_EVIDENCE"] == 0
            and statuses["UNEVALUABLE"] == 0
            and invariant
            else "FAIL"
        ),
        "cells": cells,
    }


def derive_qa02(
    episodes: Iterable[EpisodeEvidence], coverage_map: dict[str, Any]
) -> dict[str, Any]:
    episodes = list(episodes)
    per_episode = []
    dimensions = defaultdict(
        lambda: {
            "eligible_constraints": 0,
            "pass": 0,
            "fail": 0,
            "missing_evidence": 0,
        }
    )
    numerator = denominator = 0
    coverage_cells = _coverage_cells(coverage_map)
    for episode in episodes:
        if not episode.provenance["measurement_population"]:
            continue
        if not episode.scenario["qa_eligibility"]["qa02"]["eligible"]:
            continue
        denominator += 1
        if episode.expected is None:
            results = []
            conformant = False
        else:
            results = []
            for constraint in episode.expected["constraint_manifest"]["constraints"]:
                cell = coverage_cells.get(
                    (
                        constraint["constraint_id"],
                        episode.provenance["alternative"],
                    )
                )
                if cell is None:
                    results.append(
                        {
                            "constraint_id": constraint["constraint_id"],
                            "dimension": constraint["dimension"],
                            "status": "UNEVALUABLE",
                            "evidence_complete": False,
                            "decision_basis": "CONSTRAINT_ALTERNATIVE_CELL_MISSING",
                            "actual": [],
                            "expected": constraint["expected"],
                        }
                    )
                else:
                    results.append(
                        _evaluate_episode_constraint(constraint, episode, cell)
                    )
            conformant = bool(results) and all(
                result["status"] == "PASS" for result in results
            )
        numerator += int(conformant)
        for result in results:
            diag = dimensions[result["dimension"]]
            diag["eligible_constraints"] += 1
            if result["status"] == "PASS":
                diag["pass"] += 1
            elif result["status"] == "FAIL":
                diag["fail"] += 1
            else:
                diag["missing_evidence"] += 1
        per_episode.append(
            {
                "run_id": episode.provenance["run_id"],
                "scenario": episode.provenance["scenario_id"],
                "alternative": episode.provenance["alternative"],
                "exact_conformant": conformant,
                "evidence_complete": all(
                    result["status"] in {"PASS", "FAIL"}
                    for result in results
                ),
                "constraints": results,
                "coverage": coverage_diagnostics(episode, coverage_map),
            }
        )
    return {
        "numerator": numerator,
        "denominator": denominator,
        "aecr_fraction": f"{numerator}/{denominator}",
        "aecr": None if denominator == 0 else numerator / denominator,
        "percentage": None if denominator == 0 else 100 * numerator / denominator,
        "dimension_diagnostics": dict(sorted(dimensions.items())),
        "scenario_diagnostics": per_episode,
        "constraint_alternative_coverage": _derive_coverage_gate(
            episodes, coverage_map
        ),
    }
