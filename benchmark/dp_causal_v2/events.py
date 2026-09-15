from __future__ import annotations

from dataclasses import asdict, dataclass, field
from typing import Any


EVENT_TYPES = frozenset({
    "INPUT_ACCEPTED", "TEMPORAL_EVIDENCE_READ", "GROUNDED_GOAL_CREATED",
    "SEMANTIC_DECISION_CREATED", "AGENT_SELECTED", "PRIMARY_RUNTIME_ADMITTED",
    "SPECIALIST_DELEGATED", "EXECUTION_STARTED", "TOOL_REQUESTED", "TOOL_COMPLETED",
    "APPROVAL_REQUIRED", "APPROVAL_GRANTED", "PROGRESS_AVAILABLE", "RESULT_CREATED",
    "RESULT_RECEIVED", "RESULT_BOUND_TO_TASK", "RESPONSE_CREATED", "TEXT_AVAILABLE",
    "VOICE_PLAYBACK_STARTED", "BARGE_IN_DETECTED", "PLAYBACK_STOP_REQUESTED",
    "VOICE_PLAYBACK_STOPPED", "FAULT_INJECTED", "RECOVERY_STARTED", "RECOVERY_COMPLETED",
})


@dataclass(frozen=True)
class CausalEvent:
    event_id: str
    event_type: str
    logical_timestamp_ms: float
    owning_component: str
    authority: str
    user_task_id: str
    execution_id: str | None
    result_id: str | None
    result_version: int | None
    causal_parent_event_ids: tuple[str, ...]
    interface_boundary_crossed: str | None
    data: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        value = asdict(self)
        value["causal_parent_event_ids"] = list(self.causal_parent_event_ids)
        return value


class EventGraph:
    def __init__(self, task_id: str) -> None:
        self.task_id = task_id
        self.events: list[CausalEvent] = []
        self._ids: set[str] = set()
        self.clock_ms = 0.0

    def emit(self, event_type: str, owner: str, authority: str, *, parents: tuple[str, ...] = (),
             boundary: str | None = None, execution_id: str | None = None,
             result_id: str | None = None, result_version: int | None = None,
             elapsed_ms: float = 0.0, data: dict[str, Any] | None = None) -> CausalEvent:
        if event_type not in EVENT_TYPES:
            raise ValueError(f"unregistered causal event: {event_type}")
        if any(parent not in self._ids for parent in parents):
            raise ValueError("causal parent must already exist")
        self.clock_ms += elapsed_ms
        event_id = f"{self.task_id}:event:{len(self.events) + 1:03d}"
        event = CausalEvent(event_id, event_type, self.clock_ms, owner, authority,
                            self.task_id, execution_id, result_id, result_version,
                            parents, boundary, data or {})
        self.events.append(event)
        self._ids.add(event_id)
        return event

    def serialize(self) -> list[dict[str, Any]]:
        return [event.to_dict() for event in self.events]
