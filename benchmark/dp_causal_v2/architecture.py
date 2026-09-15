from __future__ import annotations

from dataclasses import asdict, dataclass, field
from typing import Any

from .dependency import ArchitectureDependencyGraph
from .events import EventGraph
from .primitives import OPERATION_COST_SOURCE, PrimitiveCatalog
from .schemas import CandidateVisibleInput
from .state_machine import LifecycleStateMachine


CANDIDATE_IDS = ("R1", "R3", "R1+@")


@dataclass(frozen=True)
class RuntimeMutation:
    break_result_correlation: bool = False
    remove_task_binding: bool = False
    unauthorized_scope: bool = False
    remove_trace_edge: bool = False
    feedback_delay_ms: float = 0.0
    playback_buffer_extra_ms: float = 0.0
    duplicate_recovery_action: bool = False
    semantic_dependency_leak: bool = False
    force_agent_core_change: bool = False
    extra_boundary: bool = False


@dataclass
class RuntimeState:
    InteractionState: dict[str, Any] = field(default_factory=dict)
    GroundingState: dict[str, Any] = field(default_factory=dict)
    UserTaskState: dict[str, Any] = field(default_factory=dict)
    ExecutionState: dict[str, Any] = field(default_factory=dict)
    AgentSessionState: dict[str, Any] = field(default_factory=dict)
    ApprovalState: dict[str, Any] = field(default_factory=dict)
    ResultState: dict[str, Any] = field(default_factory=dict)
    ResponseState: dict[str, Any] = field(default_factory=dict)
    DeliveryState: dict[str, Any] = field(default_factory=dict)


@dataclass
class ExecutionEvidence:
    candidate_id: str
    case_id: str
    input_hash: str
    primitive_replay_ids: list[str]
    primitive_invocations: list[dict[str, Any]]
    events: list[dict[str, Any]]
    state: dict[str, Any]
    state_transitions: list[dict[str, Any]]
    architecture_path: list[str]
    dependency_observation: dict[str, Any]
    conformance: dict[str, Any]

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


class CausalCandidateRuntime:
    """Executable state/event realization. It has no evaluator or filesystem access."""

    def __init__(self, candidate_id: str, profile_id: str = "QWEN3_MEDIUM_REFERENCE", mutation: RuntimeMutation | None = None) -> None:
        if candidate_id not in CANDIDATE_IDS:
            raise ValueError(f"unregistered candidate: {candidate_id}")
        self.candidate_id = candidate_id
        self.base_family = "R1" if candidate_id == "R1+@" else candidate_id
        self.primitives = PrimitiveCatalog(profile_id)
        self.mutation = mutation or RuntimeMutation()
        self.tasks: dict[str, RuntimeState] = {}
        self.resource_owner: str | None = None
        self.lifecycle = LifecycleStateMachine()

    def _local_eligible(self, item: CandidateVisibleInput) -> bool:
        return (self.candidate_id == "R1+@" and item.parameters.get("read_only") is True
                and item.consent_state == "not-required" and item.capabilities == ("bounded-read",))

    def _invoke(self, graph: EventGraph, invocations: list[dict[str, Any]], operation: str, signature: str) -> float:
        elapsed = self.primitives.latency_ms(operation, signature)
        invocations.append({"primitive_type": operation, "semantic_signature": signature,
                            "cost_source": OPERATION_COST_SOURCE[operation], "latency_ms": elapsed})
        graph.clock_ms += elapsed
        return elapsed

    @staticmethod
    def _transition(log: list[dict[str, Any]], state_name: str, old: Any, new: Any, cause: str) -> None:
        log.append({"state": state_name, "from": old, "to": new, "cause_event_id": cause})

    def execute(self, item: CandidateVisibleInput) -> ExecutionEvidence:
        graph = EventGraph(item.task_id)
        state = RuntimeState()
        self.tasks[item.task_id] = state
        self.lifecycle.accept_task(item.task_id)
        transitions: list[dict[str, Any]] = []
        invocations: list[dict[str, Any]] = []
        replay_ids: list[str] = []
        dependency: dict[str, Any] = {}
        signature = str(item.parameters.get("semantic_signature", item.case_id))
        execution_id = f"execution:{item.case_id}"

        accepted = graph.emit("INPUT_ACCEPTED", "VIA.Interaction", "interaction-boundary", data={"input_hash": item.input_hash})
        state.InteractionState.update({"input_id": item.input_hash, "status": "accepted", "turn_id": f"turn:{item.case_id}"})
        state.UserTaskState.update({"task_id": item.task_id, "status": "accepted", "turn_id": f"turn:{item.case_id}", "follow_up_task_id": item.task_id})
        self._transition(transitions, "UserTaskState", None, "accepted", accepted.event_id)

        if item.qa_id in {"QA-01", "QA-02"}:
            path = self._execute_goal(item, graph, state, transitions, invocations, replay_ids, signature, execution_id)
        else:
            path = self._execute_structural(item, graph, state, transitions, invocations, replay_ids, dependency, signature, execution_id)

        conformance = self._conformance(graph, state)
        return ExecutionEvidence(self.candidate_id, item.case_id, item.input_hash, replay_ids,
                                 invocations, graph.serialize(), asdict(state), transitions,
                                 path, dependency, conformance)

    def _execute_goal(self, item, graph, state, transitions, invocations, replay_ids, signature, execution_id):
        evidence = graph.emit("TEMPORAL_EVIDENCE_READ", "VIA.Grounding", "context-temporal",
                              parents=(graph.events[-1].event_id,), data={"versions": [dict(x) for x in item.temporal_evidence]})
        local = self._local_eligible(item)
        specialist = item.capabilities == ("specialist-v1",)
        if self.base_family == "R1":
            semantic = self.primitives.semantic(item)
            replay_ids.append(semantic.replay_id)
            if not local:
                self._invoke(graph, invocations, "SEMANTIC_INFERENCE", signature)
            grounded = graph.emit("GROUNDED_GOAL_CREATED", "VIA.Grounding", "goal-authority", parents=(evidence.event_id,), data=dict(semantic.output))
            decision = graph.emit("SEMANTIC_DECISION_CREATED", "VIA.ControlPlane", "agent-neutral-routing", parents=(grounded.event_id,))
            state.GroundingState.update(dict(semantic.output))
            if local:
                owner, authority = "VIA.BoundedRead", "bounded-deterministic-read-only"
                path = ["Input", "VIA grounding", "bounded local deterministic read", "VIA result binding", "Response"]
            else:
                owner = "SpecialistAgent" if specialist else "SelectedGeneralAgent"
                authority = "downstream-domain-workflow"
                selected = graph.emit("AGENT_SELECTED", "VIA.ControlPlane", "agent-neutral-routing", parents=(decision.event_id,), data={"agent": owner})
                self._invoke(graph, invocations, "BOUNDARY_HOP", signature)
                decision = selected
                path = ["Input", "VIA grounding", "VIA agent-neutral selection", owner,
                        "Specialist execution" if specialist else "Agent workflow/execution", "VIA result binding", "Response"]
        else:
            self._invoke(graph, invocations, "BOUNDARY_HOP", signature)
            admitted = graph.emit("PRIMARY_RUNTIME_ADMITTED", "PrimaryAgentRuntime", "primary-runtime", parents=(evidence.event_id,), boundary="VIA→PrimaryRuntime")
            semantic = self.primitives.semantic(item)
            replay_ids.append(semantic.replay_id)
            self._invoke(graph, invocations, "SEMANTIC_INFERENCE", signature)
            decision = graph.emit("SEMANTIC_DECISION_CREATED", "PrimaryAgentRuntime", "substantive-interpretation-planning", parents=(admitted.event_id,), data=dict(semantic.output))
            grounded = graph.emit("GROUNDED_GOAL_CREATED", "PrimaryAgentRuntime", "substantive-interpretation-planning", parents=(decision.event_id,), data=dict(semantic.output))
            state.GroundingState.update(dict(semantic.output))
            if specialist:
                self._invoke(graph, invocations, "SPECIALIST_DELEGATION", signature)
                decision = graph.emit("SPECIALIST_DELEGATED", "PrimaryAgentRuntime", "specialist-delegation", parents=(grounded.event_id,), boundary="PrimaryRuntime→SpecialistAgent")
                owner = "SpecialistAgent"
                path = ["Input", "VIA", "Primary Agent Runtime", "Primary planning", "Specialist delegation", "Specialist Agent", "Primary workflow/result state", "VIA Task/result correlation", "Response"]
            else:
                owner = "PrimaryAgentRuntime"
                path = ["Input", "VIA interaction boundary", "Primary Agent Runtime", "Primary interpretation/planning", "Primary execution", "VIA Task/result correlation", "Response"]
            authority = "primary-runtime-workflow"

        start = graph.emit("EXECUTION_STARTED", owner, authority, parents=(decision.event_id,), execution_id=execution_id)
        state.ExecutionState.update({"execution_id": execution_id, "task_id": item.task_id, "status": "running", "owner": owner})
        self.lifecycle.attach_execution(item.task_id, execution_id)
        if owner == "VIA.BoundedRead":
            self._invoke(graph, invocations, "TOOL_EXECUTION", signature)
        else:
            self._invoke(graph, invocations, "AGENT_EXECUTION_SHORT" if item.parameters.get("deadline_seconds", 0) <= 20 else "AGENT_EXECUTION_LONG", signature)
        tool = graph.emit("TOOL_REQUESTED", owner, authority, parents=(start.event_id,), execution_id=execution_id, data={"read_only": bool(item.parameters.get("read_only"))})
        completed = graph.emit("TOOL_COMPLETED", owner, authority, parents=(tool.event_id,), execution_id=execution_id)
        result_replay = self.primitives.specialist_result(item, semantic) if specialist else self.primitives.agent_result(item, semantic)
        replay_ids.append(result_replay.replay_id)
        created = graph.emit("RESULT_CREATED", owner, authority, parents=(completed.event_id,), execution_id=execution_id,
                             result_id=f"result:{item.case_id}", result_version=1, data=dict(result_replay.output))
        if self.base_family == "R3" or (self.base_family == "R1" and not local):
            self._invoke(graph, invocations, "RESULT_RETURN", signature)
        received = graph.emit("RESULT_RECEIVED", "VIA.TaskCorrelation", "user-task-correlation", parents=(created.event_id,), boundary=f"{owner}→VIA", execution_id=execution_id, result_id=f"result:{item.case_id}", result_version=1)
        result_task = "wrong-task" if self.mutation.break_result_correlation or self.mutation.remove_task_binding else item.task_id
        state.ResultState.update({"result_id": f"result:{item.case_id}", "version": 1, "execution_id": execution_id, "task_id": result_task, "facts": list(result_replay.output["facts"]), "accepted": result_task == item.task_id})
        accepted_result = result_task == item.task_id and self.lifecycle.bind_result(item.task_id, execution_id, f"result:{item.case_id}", 1)
        state.ResultState["accepted"] = accepted_result
        if not self.mutation.remove_task_binding and accepted_result:
            if self.mutation.extra_boundary:
                self._invoke(graph, invocations, "BOUNDARY_HOP", signature)
            self._invoke(graph, invocations, "RESPONSE_BIND", signature)
            bound = graph.emit("RESULT_BOUND_TO_TASK", "VIA.TaskCorrelation", "result-binding", parents=(received.event_id,), execution_id=execution_id, result_id=f"result:{item.case_id}", result_version=1)
            state.UserTaskState.update({"status": "completed", "execution_id": execution_id, "result_id": f"result:{item.case_id}"})
            response = graph.emit("RESPONSE_CREATED", "VIA.Response", "result-delivery", parents=(bound.event_id,), result_id=f"result:{item.case_id}", result_version=1, data={"facts": list(result_replay.output["facts"])})
            text = graph.emit("TEXT_AVAILABLE", "VIA.Delivery", "delivery", parents=(response.event_id,), result_id=f"result:{item.case_id}", result_version=1)
            state.ResponseState.update({"response_id": f"response:{item.case_id}", "result_id": f"result:{item.case_id}", "facts": list(result_replay.output["facts"])})
            state.DeliveryState.update({"delivery_id": f"delivery:{item.case_id}", "response_id": f"response:{item.case_id}", "text_event": text.event_id, "voice_event": text.event_id})
        return path

    def _execute_structural(self, item, graph, state, transitions, invocations, replay_ids, dependency, signature, execution_id):
        qa = item.qa_id
        dep = ArchitectureDependencyGraph(self.candidate_id)
        parent = graph.events[-1]
        path = ["Input", "VIA interaction boundary"]
        if qa == "QA-03":
            actors = dict(item.parameters["actors"])
            execution_id = actors["execution"]
            state.UserTaskState.update({"task_id": actors["task"], "turn_id": actors["turn"], "follow_up_task_id": actors["task"], "cancel_id": actors["cancel"]})
            state.ExecutionState.update({"execution_id": execution_id, "task_id": actors["task"]})
            state.ApprovalState.update({"approval_id": actors["approval"], "task_id": actors["task"]})
            state.ResultState.update({"result_id": actors["result"], "execution_id": execution_id, "task_id": actors["task"], "version": 1})
            state.ResponseState.update({"response_id": actors["response"], "result_id": actors["result"]})
            state.DeliveryState.update({"delivery_id": actors["delivery"], "response_id": actors["response"]})
            if self.mutation.break_result_correlation: state.ResultState["execution_id"] = "wrong-execution"
            if self.mutation.remove_task_binding: state.ResultState["task_id"] = "wrong-task"
            graph.emit("EXECUTION_STARTED", self._workflow_owner(), self._workflow_authority(), parents=(parent.event_id,), execution_id=execution_id, data={"interleaving": item.parameters["event_interleaving"]})
        elif qa == "QA-04":
            impact = dep.propagate_agent_change(str(item.parameters["change_request"]))
            zones = ("Z3",) if self.mutation.force_agent_core_change else impact.zones
            dependency.update({"components": list(impact.components), "zones": list(zones), "seams": list(impact.seams), "ownership_area": dep.agent_area, "regressions_pass": True})
            graph.emit("PROGRESS_AVAILABLE", dep.agent_component, "integration-change", parents=(parent.event_id,), data=dependency)
        elif qa == "QA-05":
            impact = dep.propagate_product_change(str(item.parameters["change_request"]), str(item.parameters["family_id"]))
            leaks = ("Z3",) if self.mutation.semantic_dependency_leak else impact.leaks
            dependency.update({"components": list(impact.components), "zones": list(impact.zones), "seams": list(impact.seams), "leaks": list(leaks), "regressions_pass": True})
            graph.emit("PROGRESS_AVAILABLE", impact.components[0], "change-propagation", parents=(parent.event_id,), data=dependency)
        elif qa == "QA-06":
            impact = dep.adapt_device(str(item.parameters["device_family"]))
            dependency.update({"components": list(impact.components), "zones": [], "core_changes": [], "functionality_delivered": True})
            graph.emit("PROGRESS_AVAILABLE", impact.components[0], "device-adaptation", parents=(parent.event_id,), data=dependency)
        elif qa == "QA-08":
            fault_owner = self._fault_owner(str(item.parameters["fault_type"]))
            graph.clock_ms = float(item.parameters["fault_timing"]["offset_ms"])
            fault = graph.emit("FAULT_INJECTED", fault_owner, "owned-state-fault", parents=(parent.event_id,), execution_id=execution_id)
            recovery = graph.emit("RECOVERY_STARTED", fault_owner, "recovery", parents=(fault.event_id,), execution_id=execution_id)
            self._invoke(graph, invocations, "RECOVERY_RECONNECT", signature)
            self._invoke(graph, invocations, "RECOVERY_RESTORE_STATE", signature)
            complete = graph.emit("RECOVERY_COMPLETED", fault_owner, "safe-continuation", parents=(recovery.event_id,), execution_id=execution_id, data={"same_task": True, "duplicate_action": self.mutation.duplicate_recovery_action})
            state.ExecutionState.update({"execution_id": execution_id, "task_id": item.task_id, "status": "continuation-ready", "reconciled": True, "duplicate_action": self.mutation.duplicate_recovery_action, "fault_event": fault.event_id, "recovery_event": complete.event_id})
        elif qa == "QA-09":
            scope = self.primitives.scope_decision(item); replay_ids.append(scope.replay_id)
            requested = list(scope.output["requested_scopes"])
            if self.mutation.unauthorized_scope: requested.append("scope:unrelated-principal:*")
            state.ApprovalState.update({"principal": item.principal, "requested_scopes": requested, "granted_scopes": requested, "task_id": item.task_id})
            graph.emit("APPROVAL_GRANTED", "PolicyEngine", "least-privilege-policy", parents=(parent.event_id,), data={"scopes": requested})
        elif qa == "QA-10":
            self._emit_trace_case(graph, parent, execution_id)
        elif qa == "QA-11":
            event_type = {"accepted_queued": "INPUT_ACCEPTED", "meaningful_progress": "PROGRESS_AVAILABLE", "approval_needed": "APPROVAL_REQUIRED", "completion": "RESULT_CREATED", "blocked_failure": "FAULT_INJECTED"}[item.parameters["event_class"]]
            available = float(item.parameters["truthful_reportable_event"]["available_offset_ms"])
            graph.clock_ms = available
            truthful = graph.emit(event_type, self._workflow_owner(), self._workflow_authority(), parents=(parent.event_id,), data={"truthful": True, "version": 1})
            self._invoke(graph, invocations, "EVENT_PROPAGATION", signature)
            graph.clock_ms += self.mutation.feedback_delay_ms
            delivered = graph.emit("TEXT_AVAILABLE", "VIA.Delivery", "feedback-delivery", parents=(truthful.event_id,), data={"meaningful": True})
            state.DeliveryState.update({"truth_event": truthful.event_id, "feedback_event": delivered.event_id})
        elif qa == "QA-12":
            onset = 1000.0
            graph.clock_ms = onset
            detected = graph.emit("BARGE_IN_DETECTED", "VIA.VoicePlaybackController", "interaction-control", parents=(parent.event_id,))
            self._invoke(graph, invocations, "PLAYBACK_CANCEL", signature)
            requested = graph.emit("PLAYBACK_STOP_REQUESTED", "VIA.VoicePlaybackController", "interaction-control", parents=(detected.event_id,))
            drain = float(item.parameters["playback_buffer_depth_ms"]) + self.mutation.playback_buffer_extra_ms
            stopped = graph.emit("VOICE_PLAYBACK_STOPPED", "VIA.VoicePlaybackController", "interaction-control", parents=(requested.event_id,), elapsed_ms=drain, data={"last_audible_sample": True})
            state.DeliveryState.update({"speech_onset_ms": onset, "stop_event": stopped.event_id})
        return path

    def _emit_trace_case(self, graph, parent, execution_id):
        specs = [
            ("TEMPORAL_EVIDENCE_READ", "evidence"), ("SEMANTIC_DECISION_CREATED", "decision"),
            ("AGENT_SELECTED", "agent_selection"), ("EXECUTION_STARTED", "execution"),
            ("PROGRESS_AVAILABLE", "task"), ("RESULT_CREATED", "approval_progress_result"),
            ("RESPONSE_CREATED", "response"), ("TEXT_AVAILABLE", "delivery"),
            ("RESULT_RECEIVED", "attribution"),
        ]
        previous = parent
        for index, (event_type, trace_role) in enumerate(specs):
            parents = () if self.mutation.remove_trace_edge and index == 4 else (previous.event_id,)
            owner = "VIA.ControlPlane" if event_type == "AGENT_SELECTED" and self.base_family == "R1" else self._workflow_owner()
            authority = "agent-neutral-routing" if event_type == "AGENT_SELECTED" and self.base_family == "R1" else self._workflow_authority()
            previous = graph.emit(event_type, owner, authority, parents=parents,
                                  execution_id=execution_id, data={"trace_role": trace_role})

    def _workflow_owner(self): return "PrimaryAgentRuntime" if self.base_family == "R3" else "SelectedGeneralAgent"
    def _workflow_authority(self): return "primary-runtime-workflow" if self.base_family == "R3" else "downstream-domain-workflow"

    def _fault_owner(self, fault_type: str) -> str:
        if "agent" in fault_type: return self._workflow_owner()
        if "delivery" in fault_type or "frontend" in fault_type: return "VIA.Delivery"
        if "context" in fault_type: return "VIA.ContextBoundary"
        return "PrimaryAgentRuntime" if self.base_family == "R3" else "VIA.TaskStore"

    def _conformance(self, graph: EventGraph, state: RuntimeState) -> dict[str, Any]:
        events = graph.events
        owners = {e.owning_component for e in events}
        if self.base_family == "R3":
            checks = {
                "primary_runtime_owns_workflow_truth": not state.ExecutionState or state.ExecutionState.get("owner", "PrimaryAgentRuntime") in {"PrimaryAgentRuntime", "SpecialistAgent"},
                "specialist_delegation_through_primary": all(e.owning_component == "PrimaryAgentRuntime" for e in events if e.event_type == "SPECIALIST_DELEGATED"),
                "no_competing_via_domain_workflow": "VIA.DomainWorkflow" not in owners,
                "user_task_correlation_preserved": bool(state.UserTaskState.get("task_id")),
            }
        else:
            checks = {
                "via_not_general_runtime": "VIA.GeneralAgentRuntime" not in owners,
                "state_changes_downstream": all(e.owning_component != "VIA.BoundedRead" or e.data.get("read_only", True) for e in events if e.event_type == "TOOL_REQUESTED"),
                "arbitrary_tools_downstream": all(e.owning_component != "VIA.BoundedRead" or e.data.get("read_only") is True for e in events if e.event_type == "TOOL_REQUESTED"),
                "initial_selection_agent_neutral": all(e.authority == "agent-neutral-routing" for e in events if e.event_type == "AGENT_SELECTED"),
                "via_owns_task_result_binding": all(e.owning_component == "VIA.TaskCorrelation" for e in events if e.event_type == "RESULT_BOUND_TO_TASK"),
            }
            if self.candidate_id == "R1+@":
                local = [e for e in events if e.owning_component == "VIA.BoundedRead"]
                checks.update({"local_bounded": all(e.authority == "bounded-deterministic-read-only" for e in local),
                               "local_deterministic": all(e.authority == "bounded-deterministic-read-only" for e in local),
                               "local_read_only": all(e.data.get("read_only", True) for e in local),
                               "local_non_planning": all(e.event_type != "SEMANTIC_DECISION_CREATED" for e in local),
                               "local_no_arbitrary_tool_selection": all(e.event_type != "AGENT_SELECTED" for e in local),
                               "local_no_durable_workflow": state.ExecutionState.get("status") != "durable",
                               "local_no_agent_execution_state": all(state.ExecutionState.get("owner") != "VIA.BoundedRead" or state.ExecutionState.get("status") != "durable" for _ in [0])})
        return {"checks": checks, "passed": all(checks.values())}
