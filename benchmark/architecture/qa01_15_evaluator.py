#!/usr/bin/env python3
"""Candidate-neutral evaluator for the QA-01~QA-15 foundation contract."""

from __future__ import annotations

import math
from typing import Any


def value_at(document: dict[str, Any], path: str) -> Any:
    value: Any = document
    for segment in path.split("."):
        if not isinstance(value, dict) or segment not in value:
            raise KeyError(path)
        value = value[segment]
    return value


def predicate_passes(predicate: dict[str, Any], trace: dict[str, Any]) -> bool:
    actual = value_at(trace, predicate["actual_path"])
    expected = predicate.get("expected")
    operator = predicate["operator"]
    if operator in {"EXACT", "BINDING", "FINAL_STATE_EQUAL", "CONTINUITY_RELATION"}:
        return actual == expected
    if operator == "ONE_OF":
        return actual in expected
    if operator == "SET_EQUAL":
        return set(actual) == set(expected)
    if operator == "ORDERED_RELATION":
        return list(actual) == list(expected)
    if operator == "REQUIRED_PROPOSITION":
        return set(expected).issubset(set(actual))
    if operator == "FORBIDDEN_PROPOSITION":
        return set(expected).isdisjoint(set(actual))
    if operator == "CLARIFICATION_REQUIRED":
        return bool(actual) is bool(expected)
    if operator == "CARDINALITY":
        return int(actual) == int(expected)
    raise ValueError(f"unsupported oracle operator: {operator}")


def evaluate_predicate_groups(
    *,
    qa_id: str,
    required_groups: list[str],
    oracle_groups: dict[str, Any] | None,
    trace: dict[str, Any],
) -> dict[str, Any]:
    failures: list[str] = []
    if oracle_groups is None:
        return {"pass": False, "failures": [f"{qa_id}:ORACLE_MISSING"]}
    for group in required_groups:
        definition = oracle_groups.get(group)
        if definition is None:
            failures.append(f"{qa_id}:{group}:GROUP_MISSING")
            continue
        if definition.get("applicability") == "N/A":
            continue
        predicates = definition.get("predicates")
        if not predicates:
            failures.append(f"{qa_id}:{group}:PREDICATE_MISSING")
            continue
        for predicate in predicates:
            try:
                passed = predicate_passes(predicate, trace)
            except (KeyError, TypeError, ValueError):
                passed = False
            if not passed:
                failures.append(f"{qa_id}:{group}:{predicate['id']}")
    return {"pass": not failures, "failures": failures}


def evaluate_correctness(
    contract: dict[str, Any], trace: dict[str, Any], oracle: dict[str, Any]
) -> dict[str, dict[str, Any]]:
    results: dict[str, dict[str, Any]] = {}
    for qa_id in ("QA-12", "QA-13", "QA-14", "QA-15"):
        results[qa_id] = evaluate_predicate_groups(
            qa_id=qa_id,
            required_groups=contract["correctness"][qa_id]["required_groups"],
            oracle_groups=oracle.get("correctness", {}).get(qa_id),
            trace=trace,
        )

    integrated = evaluate_predicate_groups(
        qa_id="QA-11",
        required_groups=contract["correctness"]["QA-11"]["required_groups"],
        oracle_groups=oracle.get("correctness", {}).get("QA-11"),
        trace=trace,
    )
    driver_applicability = oracle.get("qa11_driver_applicability")
    if driver_applicability is None:
        integrated["pass"] = False
        integrated["failures"].append("QA-11:DRIVER_APPLICABILITY_MISSING")
    else:
        for qa_id in ("QA-12", "QA-13", "QA-14", "QA-15"):
            state = driver_applicability.get(qa_id)
            if state not in {"APPLICABLE", "N/A"}:
                integrated["pass"] = False
                integrated["failures"].append(f"QA-11:{qa_id}:APPLICABILITY_MISSING")
            elif state == "APPLICABLE" and not results[qa_id]["pass"]:
                integrated["pass"] = False
                integrated["failures"].append(f"QA-11:{qa_id}:DRIVER_FAILED")
    results["QA-11"] = integrated
    return results


def evaluate_latency(
    contract: dict[str, Any], trace: dict[str, Any], oracle: dict[str, Any]
) -> dict[str, dict[str, Any]]:
    results: dict[str, dict[str, Any]] = {}
    latency_oracle = oracle.get("latency", {})
    applicability = oracle.get("latency_applicability", {})
    for qa_id, definition in contract["latency"].items():
        failures: list[str] = []
        applicability_state = applicability.get(qa_id)
        if applicability_state == "N/A":
            results[qa_id] = {
                "pass": True,
                "status": "N/A",
                "sample_ns": None,
                "observed_sample_ns": None,
                "censored": False,
                "correctness_pass": None,
                "failures": [],
            }
            continue
        if applicability_state != "APPLICABLE":
            results[qa_id] = {
                "pass": False,
                "status": "INVALID",
                "sample_ns": int(definition["timeout_ms"] * 1_000_000),
                "observed_sample_ns": None,
                "censored": True,
                "correctness_pass": None,
                "failures": [f"{qa_id}:APPLICABILITY_MISSING"],
            }
            continue
        events = trace.get("events", {})
        for event_id in definition["required_events"]:
            event = events.get(event_id)
            if event is None:
                failures.append(f"{qa_id}:{event_id}:EVENT_MISSING")
                continue
            if not isinstance(event.get("timestamp_ns"), int):
                failures.append(f"{qa_id}:{event_id}:TIMESTAMP_INVALID")
            allowed = definition.get("required_endpoint_provenance", {}).get(event_id)
            if allowed and event.get("provenance") not in allowed:
                failures.append(f"{qa_id}:{event_id}:PROVENANCE_INVALID")
            if event.get("provenance") in definition.get("forbidden_endpoint_provenance", []):
                failures.append(f"{qa_id}:{event_id}:FORBIDDEN_PROXY")

        required_stages = definition.get("required_path_stages", [])
        actual_stages = set(trace.get("path_stages", []))
        for stage in required_stages:
            if stage not in actual_stages:
                failures.append(f"{qa_id}:{stage}:PATH_STAGE_MISSING")

        validity = latency_oracle.get(qa_id)
        if validity is None or not validity.get("predicates"):
            failures.append(f"{qa_id}:VALIDITY_ORACLE_MISSING")
        else:
            for predicate in validity["predicates"]:
                try:
                    passed = predicate_passes(predicate, trace)
                except (KeyError, TypeError, ValueError):
                    passed = False
                if not passed:
                    failures.append(f"{qa_id}:VALIDITY:{predicate['id']}")

        sample_ns: int | None = None
        if not failures:
            timestamp = {key: events[key]["timestamp_ns"] for key in definition["required_events"]}
            if qa_id == "QA-01":
                sample_ns = (
                    timestamp["agent_request_available_at_agent_ingress"]
                    - timestamp["user_input_end"]
                    + timestamp["first_meaningful_audible_result_audio"]
                    - timestamp["agent_result_available_at_source"]
                )
            else:
                start, end = {
                    "QA-02": ("user_input_end", "first_meaningful_audible_direct_response"),
                    "QA-03": ("agent_status_available_at_source", "first_meaningful_audible_status_audio"),
                    "QA-04": ("barge_in_speech_onset", "interrupted_response_last_audible_sample"),
                    "QA-05": ("task_control_input_end", "correct_task_control_disposition_presented"),
                }[qa_id]
                sample_ns = timestamp[end] - timestamp[start]
            if sample_ns < 0:
                failures.append(f"{qa_id}:EVENT_ORDER_INVALID")
                sample_ns = None
        observed_sample_ns = sample_ns
        censored = bool(failures)
        if censored:
            sample_ns = int(definition["timeout_ms"] * 1_000_000)
        results[qa_id] = {
            "pass": not failures,
            "status": "PASS" if not failures else "FAIL",
            "sample_ns": sample_ns,
            "observed_sample_ns": observed_sample_ns,
            "censored": censored,
            "correctness_pass": None,
            "failures": failures,
        }
    return results


def evaluate_trial(
    contract: dict[str, Any], trace: dict[str, Any], oracle: dict[str, Any]
) -> dict[str, dict[str, dict[str, Any]]]:
    correctness = evaluate_correctness(contract, trace, oracle)
    latency = evaluate_latency(contract, trace, oracle)
    latency_dependencies = {
        "QA-01": ["QA-11"],
        "QA-02": ["QA-11"],
        "QA-03": ["QA-13", "QA-14"],
        "QA-04": [],
        "QA-05": ["QA-11", "QA-12", "QA-13", "QA-14"],
    }
    driver_applicability = oracle.get("qa11_driver_applicability", {})
    for qa_id, dependencies in latency_dependencies.items():
        if latency[qa_id]["status"] == "N/A":
            continue
        applicable_dependencies = [
            dependency
            for dependency in dependencies
            if dependency == "QA-11" or driver_applicability.get(dependency) != "N/A"
        ]
        dependency_pass = all(
            correctness[dependency]["pass"] for dependency in applicable_dependencies
        )
        latency[qa_id]["correctness_pass"] = dependency_pass
        for dependency in applicable_dependencies:
            if not correctness[dependency]["pass"]:
                latency[qa_id]["pass"] = False
                if latency[qa_id]["status"] == "PASS":
                    latency[qa_id]["status"] = "RESPONSE_OBSERVED_CORRECTNESS_FAILED"
                latency[qa_id]["failures"].append(
                    f"{qa_id}:{dependency}:CORRECTNESS_FAILED"
                )
    return {"latency": latency, "correctness": correctness}


def nearest_rank_p95(values: list[int]) -> int:
    if not values:
        raise ValueError("p95 requires at least one value")
    ordered = sorted(values)
    return ordered[math.ceil(0.95 * len(ordered)) - 1]


def aggregate_latency(samples_by_case: dict[str, list[int]]) -> int:
    if not samples_by_case or any(not values for values in samples_by_case.values()):
        raise ValueError("every frozen case needs scored samples")
    return max(nearest_rank_p95(values) for values in samples_by_case.values())


def aggregate_correctness(pass_by_case: dict[str, list[bool]]) -> float:
    if not pass_by_case or any(not values for values in pass_by_case.values()):
        raise ValueError("every frozen case needs scored trials")
    case_rates = [sum(values) / len(values) for values in pass_by_case.values()]
    return 100.0 * sum(case_rates) / len(case_rates)
