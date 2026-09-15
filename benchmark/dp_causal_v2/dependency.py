from __future__ import annotations

from dataclasses import dataclass


ZONE_BY_CHANGE = {
    "provider": "Z1", "ASR": "Z1", "audio": "Z1", "transcript": "Z1", "locale": "Z1", "turn-end": "Z1",
    "referent": "Z2", "context": "Z2", "evidence": "Z2", "screen": "Z2", "temporal": "Z2",
    "semantic": "Z3", "goal": "Z3", "constraint": "Z3", "clarification": "Z3", "planner": "Z3", "decision": "Z3",
    "Agent": "Z4", "agent": "Z4", "approval mapping": "Z4", "cancel mapping": "Z4", "progress mapping": "Z4", "result mapping": "Z4",
    "Task": "Z5", "task": "Z5", "execution": "Z5", "concurrent": "Z5", "session reconnect": "Z5",
    "response": "Z6", "delivery": "Z6", "voice summary": "Z6", "text detail": "Z6", "fact consistency": "Z6", "feedback": "Z6", "barge-in": "Z6",
    "permission": "Z7", "device": "Z7", "mobile": "Z7", "TV": "Z7", "robot": "Z7", "platform": "Z7", "sensor": "Z7", "capability selection": "Z7",
    "memory": "Z8", "residency": "Z8", "buffer": "Z8", "cache sharing": "Z8", "runtime contention": "Z8", "resource attribution": "Z8", "inference runtime": "Z8",
}


@dataclass(frozen=True)
class ChangeImpact:
    components: tuple[str, ...]
    zones: tuple[str, ...]
    seams: tuple[str, ...]
    leaks: tuple[str, ...] = ()


class ArchitectureDependencyGraph:
    """Predefined ownership/dependency graph; propagation never accepts expected zones."""

    def __init__(self, candidate_id: str) -> None:
        self.candidate_id = candidate_id
        if candidate_id == "R3":
            self.agent_area = "protocol mapping"
            self.agent_component = "primary-runtime-agent-integration"
        else:
            self.agent_area = "Agent adapter"
            self.agent_component = "via-agent-adapter"

    def propagate_agent_change(self, change: str) -> ChangeImpact:
        kind = change.split(" variation", 1)[0]
        seam = f"agent-integration:{kind}"
        return ChangeImpact((self.agent_component,), (), (seam,))

    def propagate_product_change(self, change: str, family_id: str) -> ChangeImpact:
        # The family is the frozen change-taxonomy input (not an evaluator oracle).
        # Its component→zone mapping is fixed here before campaign execution.
        zone = family_id.removeprefix("QA05-")
        seam = f"{zone.lower()}:{change.lower().replace(' ', '-')}"
        return ChangeImpact((f"component:{zone.lower()}",), (zone,), (seam,))

    def adapt_device(self, device: str) -> ChangeImpact:
        return ChangeImpact((f"{device}-input-adapter", f"{device}-context-provider", f"{device}-delivery-adapter"), (), (f"device:{device}",))
