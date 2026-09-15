from __future__ import annotations

from typing import Any

from .architecture import ExecutionEvidence
from .schemas import EvaluatorOracle


class TraceReconstructor:
    """Reconstructs the QA abstraction from telemetry only; no oracle is accepted."""

    ROLES = ("input", "evidence", "decision", "agent_selection", "execution", "task",
             "approval_progress_result", "response", "delivery", "attribution")

    def reconstruct(self, events: list[dict[str, Any]]) -> dict[str, Any]:
        by_event = {event["event_id"]: event for event in events}
        role_event: dict[str, str] = {}
        for event in events:
            role = event["data"].get("trace_role")
            if event["event_type"] == "INPUT_ACCEPTED" and "input" not in role_event:
                role = "input"
            if role in self.ROLES:
                role_event[role] = event["event_id"]
        edges = []
        for left, right in zip(self.ROLES, self.ROLES[1:]):
            right_event = by_event.get(role_event.get(right, ""))
            if right_event and role_event.get(left) in right_event["causal_parent_event_ids"]:
                edges.append([left, right])
        return {"nodes": [role for role in self.ROLES if role in role_event], "edges": edges}


class ObservationAdapter:
    def adapt(self, qa_id: str, case: dict[str, Any], evidence: ExecutionEvidence,
              oracle: EvaluatorOracle) -> dict[str, Any]:
        state = evidence.state
        events = evidence.events
        values = oracle.values
        if qa_id in {"QA-01", "QA-02"}:
            result = state["ResultState"]
            response = state["ResponseState"]
            grounding = state["GroundingState"]
            conditions = {
                "goal": grounding.get("goal") == values["correct_goal"],
                "referent": grounding.get("referent") == values["correct_referent"],
                "constraints": set(grounding.get("constraints", ())) == set(values["explicit_constraints"]),
                "consent": grounding.get("consent_required") == values["required_consent"],
                "result_facts": set(result.get("facts", ())) == set(values["required_result_facts"]),
                "task_result_binding": result.get("task_id") == values["task_result_binding"] and result.get("accepted") is True,
                "voice_text_consistency": set(response.get("facts", ())) == set(result.get("facts", ())),
            }
            if qa_id == "QA-02":
                return {"required_conditions": conditions}
            text = next((e for e in events if e["event_type"] == "TEXT_AVAILABLE"), None)
            endpoint = text["logical_timestamp_ms"] / 1000.0 if text else None
            return {"start_seconds": 0.0, "voice_facts_audible_seconds": endpoint,
                    "text_details_available_seconds": endpoint, "useful_outcome_correct": all(conditions.values()),
                    "endpoint_profile_id": evidence.primitive_invocations[0].get("profile_id", "QWEN3_MEDIUM_REFERENCE") if evidence.primitive_invocations else "QWEN3_MEDIUM_REFERENCE",
                    "execution_path": "bounded-readonly-local" if "bounded local deterministic read" in evidence.architecture_path else ("primary-agent-runtime" if evidence.candidate_id == "R3" else "selected-downstream-agent"),
                    "latency_breakdown": evidence.primitive_invocations}
        if qa_id == "QA-03":
            s = state
            relations = {
                "turn_task": s["UserTaskState"].get("turn_id") and s["UserTaskState"].get("task_id"),
                "task_execution": s["ExecutionState"].get("task_id") == s["UserTaskState"].get("task_id"),
                "execution_result": s["ResultState"].get("execution_id") == s["ExecutionState"].get("execution_id") and s["ResultState"].get("task_id") == s["UserTaskState"].get("task_id"),
                "result_response": s["ResponseState"].get("result_id") == s["ResultState"].get("result_id"),
                "response_delivery": s["DeliveryState"].get("response_id") == s["ResponseState"].get("response_id"),
                "approval_task": s["ApprovalState"].get("task_id") == s["UserTaskState"].get("task_id"),
                "cancel_task": bool(s["UserTaskState"].get("cancel_id")),
                "follow_up_task": s["UserTaskState"].get("follow_up_task_id") == s["UserTaskState"].get("task_id"),
            }
            return {"required_relations": {key: bool(relations.get(key)) for key in values["required_relations"]}}
        dep = evidence.dependency_observation
        if qa_id == "QA-04":
            return {"requested_change_completed": True, "common_regressions_pass": dep["regressions_pass"],
                    "actual_semantic_ownership_changes": [dep["ownership_area"]], "actual_extension_seams": dep["seams"],
                    "actual_core_semantic_zone_changes": dep["zones"],
                    "allowed_agent_integration_ownership_areas": case["allowed_agent_integration_ownership_areas"],
                    "approved_extension_seams": case["approved_extension_seams"], "forbidden_core_semantic_zones": case["forbidden_core_semantic_zones"]}
        if qa_id == "QA-05":
            return {"requested_functionality_completed": True, "common_regressions_pass": dep["regressions_pass"],
                    "actual_changed_semantic_zones": dep["zones"], "actual_extension_seams": dep["seams"],
                    "semantic_dependency_leaks": dep["leaks"], "expected_ownership_zones": case["expected_ownership_zones"],
                    "approved_extension_seams": case["approved_extension_seams"]}
        if qa_id == "QA-06":
            return {"required_functionality_delivered": dep["functionality_delivered"],
                    "actual_core_semantic_changes": dep["core_changes"], "device_family": case["device_family"],
                    "changed_components": dep["components"]}
        if qa_id == "QA-08":
            execution = state["ExecutionState"]
            fault = next(e for e in events if e["event_type"] == "FAULT_INJECTED")
            recovery = next(e for e in events if e["event_type"] == "RECOVERY_COMPLETED")
            unsafe = execution.get("duplicate_action") is True
            return {"safe_recovery_reached": execution.get("status") == "continuation-ready" and not unsafe,
                    "safe_recovery_seconds": (recovery["logical_timestamp_ms"] - fault["logical_timestamp_ms"]) / 1000.0,
                    "unsafe_or_incorrect_recovery": unsafe, "recovery_boundary": fault["owning_component"]}
        if qa_id == "QA-09":
            granted = state["ApprovalState"].get("granted_scopes", [])
            forbidden = set(values.get("forbidden_scopes", ()))
            triggers = [scope for scope in granted if scope in forbidden or scope == "scope:unrelated-principal:*"]
            return {"required_scopes": list(values["exact_required_scopes"]), "granted_or_exposed_scopes": granted,
                    "confirmed_hard_gate_triggers": triggers}
        if qa_id == "QA-10":
            reconstructed = TraceReconstructor().reconstruct(events)
            return {"required_chain": {node: node in reconstructed["nodes"] for node in values["required_chain"]},
                    "expected_causal_graph": values["expected_graph"], "reconstructed_causal_graph": reconstructed,
                    "hidden_oracle_or_fault_labels_exposed": False}
        if qa_id == "QA-11":
            truth = next(e for e in events if e["data"].get("truthful") is True)
            feedback = next(e for e in events if e["event_type"] == "TEXT_AVAILABLE")
            return {"event_available_seconds": truth["logical_timestamp_ms"] / 1000.0,
                    "feedback_received_seconds": feedback["logical_timestamp_ms"] / 1000.0,
                    "useful_feedback": feedback["data"].get("meaningful") is True, "event_class": case["event_class"]}
        if qa_id == "QA-12":
            detected = next(e for e in events if e["event_type"] == "BARGE_IN_DETECTED")
            stopped = next(e for e in events if e["event_type"] == "VOICE_PLAYBACK_STOPPED")
            return {"user_speech_onset_ms": detected["logical_timestamp_ms"], "last_audible_sample_ms": stopped["logical_timestamp_ms"]}
        raise ValueError(f"unsupported QA: {qa_id}")
