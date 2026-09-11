"""QA-02 exact conformance and 36-row coverage diagnostics."""

from __future__ import annotations

from collections import defaultdict
from typing import Any, Iterable

from .models import ActualSemanticFacts, EpisodeEvidence


def _actual_values(actual: ActualSemanticFacts, dimension: str) -> tuple[list[Any], bool]:
    if dimension == "referent":
        return list(actual.referent_bindings), bool(actual.referent_bindings)
    if dimension == "task_association":
        values: list[Any] = []
        for item in actual.task_associations:
            values.extend([item["task_relation"], {"task_relation": item["task_relation"]}, {"task_id": item["task_id"]}])
        return values, bool(actual.task_associations)
    if dimension == "result_binding":
        return [item["task_id"] for item in actual.result_bindings], bool(actual.result_bindings)
    if dimension == "clarification":
        return list(actual.clarifications), bool(actual.clarifications)
    if dimension == "execution_path":
        return [r["identity"] for r in actual.execution_routes], bool(actual.execution_routes)
    if dimension == "execution_owner":
        owners = []
        for route in actual.execution_routes:
            initial = route["initial_executor_id"]
            owners.append("VIA_FAST" if initial.startswith("VIA_LOCAL") else initial)
        return owners, bool(actual.execution_routes)
    if dimension == "delegated_agent":
        values = []
        for route in actual.execution_routes:
            values.extend(route["delegation_chain"])
            final = route["final_executor_id_if_known"]
            if final and final != route["initial_executor_id"]:
                values.append(final)
            if route["initial_executor_id"].endswith("Agent"):
                values.append(route["initial_executor_id"])
        return values, True  # route absence is meaningful negative evidence for forbidden-agent rules
    if dimension == "observable_effect":
        values: list[Any] = []
        for effect in actual.observable_effects:
            values.append({"capability_id": effect["capability_id"], "state": "DECREASED" if "DECREASED" in actual.derived_predicates else effect["state"]})
            values.append({"object_id": effect["subject_id"], "state": effect["state"]})
        values.extend(actual.derived_predicates)
        return values, bool(actual.observable_effects) or bool(actual.failure_outcomes)
    if dimension == "failure_outcome":
        return list(actual.derived_predicates), bool(actual.failure_outcomes) or "RECOVERED_TO_NETWORK" in actual.derived_predicates or "CLARIFICATION_REQUESTED" in actual.derived_predicates
    if dimension == "routing":
        return list(actual.derived_predicates), bool(actual.failure_outcomes) or bool(actual.execution_routes) or bool(actual.route_candidates)
    return [], False


def evaluate_constraint(constraint: dict[str, Any], actual: ActualSemanticFacts) -> dict[str, Any]:
    values, resolvable = _actual_values(actual, constraint["dimension"])
    if not resolvable:
        return {"constraint_id": constraint["constraint_id"], "dimension": constraint["dimension"], "status": "MISSING_ACTUAL_EVIDENCE", "actual": values, "expected": constraint["expected"]}
    expected = constraint["expected"]
    operator = constraint["operator"]
    matched = expected in values if operator in {"equals", "predicate"} else any(value in expected for value in values)
    if constraint["constraint_set"] == "FORBIDDEN":
        passed = not matched
    elif constraint["constraint_set"] == "ALLOWED" and operator == "in":
        passed = bool(values) and all(value in expected for value in values)
    else:
        passed = matched
    return {"constraint_id": constraint["constraint_id"], "dimension": constraint["dimension"], "constraint_set": constraint["constraint_set"], "operator": operator, "status": "PASS" if passed else "FAIL", "actual": values, "expected": expected}


def coverage_diagnostics(episode: EpisodeEvidence, coverage_map: dict[str, Any]) -> list[dict[str, Any]]:
    scenario = episode.provenance["scenario_id"]
    constraints = {c["constraint_id"]: c for c in episode.expected["constraint_manifest"]["constraints"]} if episode.expected else {}
    results = []
    for row in coverage_map["rows"]:
        if row["scenario_id"] != scenario:
            continue
        actual_values, present = _actual_values(episode.actual, row["dimension"])
        results.append({
            "constraint_id": row["constraint_id"], "expected_source_resolvable": row["constraint_id"] in constraints,
            "actual_source_resolvable": bool(row["actual_fields"]), "authority_recognized": bool(row["authority"]),
            "deterministic_derivation_available": row["independently_derivable"] and bool(row["derivation_rule"]),
            "actual_raw_evidence_present": present, "status": "PASS" if present else "MISSING_ACTUAL_EVIDENCE",
            "actual": actual_values,
        })
    return results


def derive_qa02(episodes: Iterable[EpisodeEvidence], coverage_map: dict[str, Any]) -> dict:
    per_episode, dimensions = [], defaultdict(lambda: {"eligible_constraints": 0, "pass": 0, "fail": 0, "missing_evidence": 0})
    numerator = denominator = 0
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
            results = [evaluate_constraint(c, episode.actual) for c in episode.expected["constraint_manifest"]["constraints"]]
            conformant = bool(results) and all(r["status"] == "PASS" for r in results)
        numerator += int(conformant)
        for result in results:
            diag = dimensions[result["dimension"]]
            diag["eligible_constraints"] += 1
            if result["status"] == "PASS": diag["pass"] += 1
            elif result["status"] == "FAIL": diag["fail"] += 1
            else: diag["missing_evidence"] += 1
        per_episode.append({
            "run_id": episode.provenance["run_id"], "scenario": episode.provenance["scenario_id"],
            "alternative": episode.provenance["alternative"], "exact_conformant": conformant,
            "constraints": results, "coverage": coverage_diagnostics(episode, coverage_map),
        })
    return {
        "numerator": numerator, "denominator": denominator,
        "aecr_fraction": f"{numerator}/{denominator}", "aecr": None if denominator == 0 else numerator / denominator,
        "percentage": None if denominator == 0 else 100 * numerator / denominator,
        "dimension_diagnostics": dict(sorted(dimensions.items())), "scenario_diagnostics": per_episode,
    }
