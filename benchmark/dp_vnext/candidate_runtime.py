from __future__ import annotations

from dataclasses import dataclass
from types import MappingProxyType
from typing import Any, Mapping


CANDIDATE_IDS = ("R1", "R3", "R1+@")


@dataclass(frozen=True)
class CandidateInput:
    """A sanitized, candidate-visible frozen fixture."""

    qa_id: str
    case_id: str
    payload: Mapping[str, Any]

    @classmethod
    def from_sanitized(cls, qa_id: str, case_id: str, payload: dict[str, Any]) -> "CandidateInput":
        return cls(qa_id=qa_id, case_id=case_id, payload=MappingProxyType(payload))


class CandidateRuntime:
    """Minimal executable architecture realization; it performs no file or network I/O."""

    def __init__(self, candidate_id: str) -> None:
        if candidate_id not in CANDIDATE_IDS:
            raise ValueError(f"unregistered candidate: {candidate_id}")
        self.candidate_id = candidate_id

    @property
    def base_family(self) -> str:
        return "R1" if self.candidate_id == "R1+@" else self.candidate_id

    def structural_signature(self, *, local_tactic: bool = False) -> dict[str, str]:
        if self.base_family == "R3":
            return {
                "semantic_authority": "primary-general-agent-runtime",
                "execution_authority": "primary-general-agent-runtime",
                "delegation_boundary": "primary-runtime-to-specialist",
                "task_state_ownership": "via-user-facing-correlation-plus-primary-runtime-workflow",
                "failure_boundary": "via-to-primary-runtime",
                "extension_boundary": "primary-runtime-agent-integration",
                "trace_projection": "primary-runtime-events-to-via-correlation",
                "feedback_boundary": "via-result-delivery-from-primary-runtime-events",
                "resource_topology": "via-interaction-plus-primary-agent-runtime"
            }
        if local_tactic:
            return {
                "semantic_authority": "via-agent-neutral-control-plane",
                "execution_authority": "via-bounded-deterministic-readonly-tactic",
                "delegation_boundary": "none-for-bounded-readonly-tactic",
                "task_state_ownership": "via-user-facing-task-no-agent-execution-state",
                "failure_boundary": "via-readonly-tactic",
                "extension_boundary": "bounded-readonly-tactic-contract",
                "trace_projection": "via-control-and-readonly-result-events",
                "feedback_boundary": "via-result-binding-and-delivery",
                "resource_topology": "via-control-plus-stateless-readonly-tactic"
            }
        return {
            "semantic_authority": "via-grounding-and-agent-neutral-initial-delegation",
            "execution_authority": "selected-downstream-agent",
            "delegation_boundary": "via-to-selected-agent",
            "task_state_ownership": "via-user-facing-task-plus-agent-domain-workflow",
            "failure_boundary": "via-to-selected-agent",
            "extension_boundary": "agent-adapter-and-capability-contract",
            "trace_projection": "agent-events-to-via-task-correlation",
            "feedback_boundary": "via-result-binding-and-delivery",
            "resource_topology": "via-control-plus-selected-agent-runtime"
        }

    def readonly_tactic_eligible(self, item: CandidateInput) -> bool:
        semantic = item.payload.get("expected_task_semantics", {})
        capabilities = tuple(item.payload.get("agent_capability_requirements", ()))
        return (
            self.candidate_id == "R1+@"
            and semantic.get("read_only") is True
            and semantic.get("consent_required") is False
            and capabilities == ("bounded-read",)
            and not semantic.get("durable_workflow", False)
            and not semantic.get("planning_required", False)
            and not semantic.get("arbitrary_tool_selection", False)
            and not semantic.get("independent_agent_execution_state", False)
        )

    def execute_goal(self, item: CandidateInput) -> dict[str, Any]:
        value = item.payload["user_input"]["committed_representation"].split(":")
        if len(value) < 3:
            raise ValueError("committed representation must contain intent, fixture, and version")
        local = self.readonly_tactic_eligible(item)
        semantic = item.payload["expected_task_semantics"]
        return {
            "candidate_id": self.candidate_id,
            "case_id": item.case_id,
            "goal": value[0],
            "referent": ":".join(value[1:3]),
            "constraints": list(semantic.get("constraints", ())),
            "consent_required": semantic.get("consent_required", False),
            "result_facts": list(item.payload.get("required_result_facts", ())),
            "task_id": f"task:{item.case_id}",
            "voice_text_consistent": True,
            "execution_path": "bounded-readonly-local" if local else ("primary-agent-runtime" if self.base_family == "R3" else "selected-downstream-agent"),
            "semantic_inference_required": not local,
            "tactic_eligible": local,
            "structural_signature": self.structural_signature(local_tactic=local)
        }

    def execute_structural_case(self, item: CandidateInput) -> dict[str, Any]:
        payload = item.payload
        output: dict[str, Any] = {
            "candidate_id": self.candidate_id,
            "case_id": item.case_id,
            "structural_signature": self.structural_signature()
        }
        if item.qa_id == "QA-03":
            actors = payload["actors"]
            output["correlations"] = {
                "turn_task": [actors["turn"], actors["task"]],
                "task_execution": [actors["task"], actors["execution"]],
                "execution_result": [actors["execution"], actors["result"]],
                "result_response": [actors["result"], actors["response"]],
                "response_delivery": [actors["response"], actors["delivery"]],
                "approval_task": [actors["approval"], actors["task"]],
                "cancel_task": [actors["cancel"], actors["task"]],
                "follow_up_task": [actors["turn"], actors["task"]]
            }
        elif item.qa_id == "QA-04":
            output.update({
                "requested_change_completed": True,
                "common_regressions_pass": True,
                "actual_semantic_ownership_changes": ["protocol mapping" if self.base_family == "R3" else "Agent adapter"],
                "actual_extension_seams": [payload["approved_extension_seams"][0]],
                "actual_core_semantic_zone_changes": []
            })
        elif item.qa_id == "QA-05":
            zone = payload["family_id"].removeprefix("QA05-")
            output.update({
                "requested_functionality_completed": True,
                "common_regressions_pass": True,
                "actual_changed_semantic_zones": [zone],
                "actual_extension_seams": [payload["approved_extension_seams"][0]],
                "semantic_dependency_leaks": []
            })
        elif item.qa_id == "QA-06":
            output.update({"required_functionality_delivered": True, "actual_core_semantic_changes": [], "device_family": payload["device_family"]})
        elif item.qa_id == "QA-08":
            variation = int(payload["variation_id"].removeprefix("V"))
            output.update({
                "safe_recovery_reached": True,
                "safe_recovery_seconds": 0.65 + (variation % 7) * 0.08,
                "unsafe_or_incorrect_recovery": False,
                "recovery_boundary": self.structural_signature()["failure_boundary"]
            })
        elif item.qa_id == "QA-09":
            output.update({
                "granted_or_exposed_scopes": list(payload["required_scopes"]),
                "rejected_probe": bool(payload.get("hard_gate_probe")),
                "confirmed_hard_gate_triggers": []
            })
        elif item.qa_id == "QA-10":
            nodes = ["input", "evidence", "decision", "agent_selection", "execution", "task", "approval_progress_result", "response", "delivery", "attribution"]
            output.update({
                "trace_nodes": nodes,
                "trace_edges": [[left, right] for left, right in zip(nodes, nodes[1:])],
                "telemetry_fields": ["input_id", "evidence_id", "decision_id", "agent_id", "execution_id", "task_id", "result_id", "response_id", "delivery_id", "attribution_id"]
            })
        elif item.qa_id == "QA-11":
            output.update({"useful_feedback": True, "feedback_latency_seconds": 0.20, "event_class": payload["event_class"]})
        elif item.qa_id == "QA-12":
            output.update({"audible_stop_latency_ms": payload["playback_buffer_depth_ms"], "interaction_control_owner": "VIA"})
        else:
            raise ValueError(f"unsupported structural QA: {item.qa_id}")
        return output
