from __future__ import annotations

from hashlib import sha256
from typing import Any

from benchmark.dp_vnext.controls import LatencyReference

from .schemas import CandidateVisibleInput, FrozenPrimitiveReplay


OPERATION_COST_SOURCE = {
    "SEMANTIC_INFERENCE": "COMBINED_EXECUTION_DECISION",
    "BOUNDARY_HOP": "H", "AGENT_ADMISSION": "H", "SPECIALIST_DELEGATION": "H",
    "AGENT_EXECUTION_SHORT": "X3", "AGENT_EXECUTION_LONG": "X3", "TOOL_EXECUTION": "X2",
    "RESULT_RETURN": "H", "STATE_PERSIST": "H", "EVENT_PROPAGATION": "H",
    "RESPONSE_BIND": "H", "PLAYBACK_CANCEL": "X1", "RECOVERY_RECONNECT": "X3",
    "RECOVERY_RESTORE_STATE": "X2",
}


class PrimitiveCatalog:
    """Common deterministic capability and latency source shared by every candidate."""

    def __init__(self, profile_id: str = "QWEN3_MEDIUM_REFERENCE") -> None:
        self.profile_id = profile_id
        self.reference = LatencyReference()

    def latency_ms(self, operation: str, signature: str) -> float:
        source = OPERATION_COST_SOURCE[operation]
        if operation == "SEMANTIC_INFERENCE":
            return self.reference.model_ms(self.profile_id, signature, source)
        value = self.reference.primitive_ms(source, signature)
        if operation == "AGENT_EXECUTION_LONG":
            return value * 2.0
        return value

    @staticmethod
    def semantic(item: CandidateVisibleInput) -> FrozenPrimitiveReplay:
        parts = (item.committed_representation or f"{item.operation}:{item.case_id}:v1").split(":")
        output = {
            "goal": parts[0],
            "referent": ":".join(parts[1:3]),
            "constraints": list(item.parameters.get("constraints", ())),
            "consent_required": item.consent_state == "required",
        }
        return FrozenPrimitiveReplay("SEMANTIC_INTERPRETATION", item.input_hash, output,
                                     deterministic_seed=sha256(item.input_hash.encode()).hexdigest())

    @staticmethod
    def agent_result(item: CandidateVisibleInput, semantic: FrozenPrimitiveReplay) -> FrozenPrimitiveReplay:
        count = int(item.parameters.get("result_fact_count", 0))
        facts = [f"{item.case_id}:fact:{name}" for name in ("identity", "state", "version")[:count]]
        output = {"facts": facts, "goal": semantic.output["goal"], "referent": semantic.output["referent"], "version": 1}
        return FrozenPrimitiveReplay("AGENT_DOMAIN_RESULT", semantic.replay_id, output)

    @staticmethod
    def specialist_result(item: CandidateVisibleInput, semantic: FrozenPrimitiveReplay) -> FrozenPrimitiveReplay:
        base = PrimitiveCatalog.agent_result(item, semantic)
        return FrozenPrimitiveReplay("SPECIALIST_RESULT", base.input_identity, base.output)

    @staticmethod
    def scope_decision(item: CandidateVisibleInput) -> FrozenPrimitiveReplay:
        domain = item.operation.removeprefix("complete-").removesuffix("-goal")
        task_suffix = item.task_id.removeprefix("task:")
        output = {"requested_scopes": [f"scope:{domain}:object-{(int(task_suffix[-2:]) - 1) % 5}", f"scope:task:{task_suffix}"]}
        return FrozenPrimitiveReplay("SCOPE_DERIVATION", item.input_hash, output)
