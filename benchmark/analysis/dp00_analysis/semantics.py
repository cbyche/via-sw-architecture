"""Oracle-free reconstruction of actual semantic facts."""

from __future__ import annotations

from typing import Any

from .models import ActualSemanticFacts


def event_variant(event: dict[str, Any]) -> tuple[str, Any]:
    value = event["event"]
    if isinstance(value, str):
        return value, None
    outer, payload = next(iter(value.items()))
    if outer == "Architecture":
        if isinstance(payload, str):
            return payload, None
        return next(iter(payload.items()))
    return outer, payload


def route_identity(route: dict[str, Any]) -> str:
    kind = route["route_kind"]
    initial = route["initial_executor_id"]
    final = route["final_executor_id_if_known"]
    if kind == "LOCAL_DIRECT":
        owner = "VIA_FAST" if initial.startswith("VIA_LOCAL") else initial
        return f"LOCAL_DIRECT:{owner}"
    if kind == "EXECUTOR_DELEGATED":
        return f"EXECUTOR_DELEGATED:{initial}->{final}"
    return f"EXECUTOR_DIRECT:{initial}"


def reconstruct_actual(events: list[dict[str, Any]], model_calls: list[dict[str, Any]] | None = None) -> ActualSemanticFacts:
    """Project actual values only. This function intentionally accepts no oracle."""
    referents: list[dict[str, Any]] = []
    tasks: list[dict[str, Any]] = []
    invocations: list[dict[str, Any]] = []
    routes: list[dict[str, Any]] = []
    clarifications: list[dict[str, Any]] = []
    results: list[dict[str, Any]] = []
    effects: list[dict[str, Any]] = []
    failures: list[str] = []
    candidates: list[dict[str, Any]] = []
    rejected: list[dict[str, Any]] = []
    pending: list[dict[str, Any]] = []
    predicates: set[str] = set()

    for event in events:
        variant, payload = event_variant(event)
        corr = event["product_correlation"]
        if variant == "ReferentBound" and event["emitter"] == "ARCHITECTURE_UNDER_TEST":
            referents.append({"role": payload["referent_role"].lower(), "object_id": payload["resolved_referent_id"], "turn_id": corr["turn_id"]})
        elif variant == "TaskAssociated" and event["emitter"] == "ARCHITECTURE_UNDER_TEST":
            tasks.append({"task_relation": payload["task_relation"], "task_id": corr["task_id"], "turn_id": corr["turn_id"]})
        elif variant == "RouteCandidateObserved":
            candidates.append(payload["route"])
        elif variant == "RouteCandidateRejected":
            rejected.append({"route": payload["route"], "reason": payload["reason"]})
        elif variant == "RouteCommitted" and event["emitter"] == "ARCHITECTURE_UNDER_TEST":
            route = dict(payload["route"])
            route["identity"] = route_identity(route)
            route["event_id"] = event["event_id"]
            route["timestamp"] = event["monotonic_timestamp"]
            routes.append(route)
        elif variant == "ExecutionStarted":
            item = dict(payload["invocation"])
            item.update({"task_id": corr["task_id"], "execution_id": corr["execution_id"], "timestamp": event["monotonic_timestamp"]})
            invocations.append(item)
        elif variant == "ClarificationRequested":
            item = {"requested": True, "resolved": False, "request_turn_id": corr["turn_id"], "response_turn_id": None, "reason": payload["reason"], "request_timestamp": event["monotonic_timestamp"], "resolve_timestamp": None}
            pending.append(item)
            clarifications.append(item)
        elif variant == "ClarificationResolved":
            item = pending[-1] if pending else {"requested": False}
            item.update({"resolved": True, "request_turn_id": payload["request_turn_id"], "response_turn_id": payload["response_turn_id"], "resolve_timestamp": event["monotonic_timestamp"]})
        elif variant == "ResultBound":
            results.append({"result_id": corr["result_id"], "task_id": corr["task_id"], "execution_id": corr["execution_id"]})
        elif variant == "UsefulOutcomeObserved" and event["emitter"] == "OUTCOME_PROBE":
            effect = dict(payload["effect"])
            effect["timestamp"] = event["monotonic_timestamp"]
            effects.append(effect)
            if effect["effect_type"] == "VOLUME_CHANGED" and effect["after_value"] < effect["before_value"]:
                predicates.add("DECREASED")
            mapping = {
                "FILE_INSPECTED": "LATEST_DOWNLOAD_NAME_EMITTED",
                "WIFI_STATUS_OBSERVED": "NETWORK_STATUS_OBSERVED",
                "DNS_CHECK_OBSERVED": "DNS_CHECK_OBSERVED",
            }
            if effect["effect_type"] in mapping:
                predicates.add(mapping[effect["effect_type"]])
            if effect["effect_type"] == "DOWNLOADS_ORGANIZED" and effect["executor_id"] == "ARGO":
                predicates.add("DOWNLOADS_ORGANIZED_BY_DOMAIN_EXECUTOR")
        elif variant == "EpisodeFailed":
            failures.append(payload["reason"])

    calls = model_calls or []
    proposed = [c["semantic_output_reference"]["value"] for c in calls if c["semantic_output_reference"]]
    committed_executors = {r["initial_executor_id"] for r in routes} | {r["final_executor_id_if_known"] for r in routes}
    executed = {i["executor_id"] for i in invocations}
    if "MailAgent" in proposed and "MailAgent" not in committed_executors:
        predicates.add("MAIL_AGENT_REJECTED_OR_NOT_COMMITTED")
    if any(r == "MODEL_MALFORMED" for r in failures) and not routes:
        predicates.add("SAFE_FAILURE:MALFORMED")
    if any(r == "MODEL_TIMEOUT" for r in failures) and not routes:
        predicates.add("SAFE_FAILURE:TIMEOUT")
    if any(r == "MODEL_NO_RESPONSE" for r in failures) and not routes:
        predicates.add("SAFE_FAILURE:NO_RESPONSE")
    if any(r == "INVALID_ROUTE" for r in failures) and "MailAgent" in proposed:
        predicates.add("SAFE_FAILURE:WRONG_CANDIDATE")
    if clarifications:
        predicates.add("CLARIFICATION_REQUESTED")
    if any(i["executor_id"] == "NetworkAgent" for i in invocations):
        predicates.add("RECOVERED_TO_NETWORK")
    if "MailAgent" in executed and any(e["effect_type"] in {"WIFI_STATUS_OBSERVED", "DIAGNOSIS_STARTED"} for e in effects):
        predicates.add("MAIL_AGENT_EXECUTED_WIFI_REQUEST")
    if any(i["executor_id"].startswith("VIA_LOCAL") for i in invocations) and any(e["effect_type"] == "DOWNLOADS_ORGANIZED" for e in effects):
        predicates.add("LOCAL_FAST_CLAIMED_DOMAIN_PLANNING_SUCCESS")
    if any(e["effect_type"] == "DOCUMENT_OPENED" for e in effects):
        resolved_at = max((c.get("resolve_timestamp") or -1 for c in clarifications), default=-1)
        if any(e["timestamp"] < resolved_at for e in effects if e["effect_type"] == "DOCUMENT_OPENED"):
            predicates.add("FILE_OPEN_BEFORE_CLARIFICATION_RESOLVED")
    if routes and any(c["status"] == "MALFORMED" for c in calls):
        predicates.add("COMMIT_DERIVED_FROM_MALFORMED_OUTPUT")
    if routes and calls and all(c["status"] in {"TIMED_OUT", "NO_RESPONSE"} for c in calls):
        predicates.add("ROUTE_COMMIT_WITHOUT_VALID_MODEL_OUTPUT")
    if effects and (failures or any(c["status"] in {"MALFORMED", "TIMED_OUT", "NO_RESPONSE"} for c in calls)):
        predicates.add("UNAUTHORIZED_DOMAIN_EFFECT")

    return ActualSemanticFacts(
        tuple(referents), tuple(tasks), tuple(invocations), tuple(routes), tuple(clarifications),
        tuple(results), tuple(effects), tuple(failures), tuple(candidates), tuple(rejected), frozenset(predicates)
    )
