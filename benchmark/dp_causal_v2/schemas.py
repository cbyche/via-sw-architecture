from __future__ import annotations

from dataclasses import dataclass
from hashlib import sha256
import json
from types import MappingProxyType
from typing import Any, Mapping


def _identity(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return sha256(encoded).hexdigest()


@dataclass(frozen=True)
class CandidateVisibleInput:
    """An explicit allow-list projection; unknown source keys cannot cross the boundary."""

    qa_id: str
    case_id: str
    modality: str | None
    committed_representation: str | None
    temporal_evidence: tuple[Mapping[str, Any], ...]
    capabilities: tuple[str, ...]
    consent_state: str
    principal: str | None
    task_id: str
    operation: str
    parameters: Mapping[str, Any]
    input_hash: str

    @classmethod
    def from_case(cls, qa_id: str, case: Mapping[str, Any]) -> "CandidateVisibleInput":
        case_id = str(case.get("goal_id", case.get("case_id")))
        user = case.get("user_input", {})
        semantics = case.get("expected_task_semantics", {})
        actors = case.get("actors", {})
        # Every field below is named explicitly. In particular, required/expected/oracle
        # values are never copied wholesale into candidate input.
        params: dict[str, Any] = {}
        allow_by_qa = {
            "QA-03": ("event_interleaving", "actors"),
            "QA-04": ("change_request", "family_id"),
            "QA-05": ("change_request", "family_id"),
            "QA-06": ("device_family", "requirement"),
            "QA-08": ("fault_type", "fault_timing", "family_id"),
            "QA-09": ("purpose", "scope_version", "expiry_event", "hard_gate_probe"),
            "QA-10": ("execution_fixture", "outcome_class"),
            "QA-11": ("event_class", "required_channel", "truthful_reportable_event"),
            "QA-12": ("playback_condition", "playback_buffer_depth_ms", "concurrent_state"),
        }
        for key in allow_by_qa.get(qa_id, ()):
            if key in case:
                params[key] = case[key]
        if qa_id in {"QA-01", "QA-02"}:
            params = {
                "intent_hint": semantics.get("intent"),
                "referent_version": semantics.get("referent_version"),
                "constraints": tuple(semantics.get("constraints", ())),
                "read_only": bool(semantics.get("read_only", False)),
                "consent_required": bool(semantics.get("consent_required", False)),
                "semantic_signature": case.get("semantic_signature", case_id),
                "deadline_seconds": case.get("deadline_seconds"),
                "result_fact_count": len(case.get("required_text_detail_fields", ())),
            }
        seed = {"qa_id": qa_id, "case_id": case_id, "user": user, "params": params}
        return cls(
            qa_id=qa_id,
            case_id=case_id,
            modality=case.get("modality"),
            committed_representation=user.get("committed_representation"),
            temporal_evidence=tuple(MappingProxyType(dict(x)) for x in case.get("temporal_interaction_events", ())),
            capabilities=tuple(case.get("agent_capability_requirements", ())),
            consent_state="required" if semantics.get("consent_required") else "not-required",
            principal=case.get("principal"),
            task_id=str(actors.get("task", case.get("task", f"task:{case_id}"))),
            operation=str(case.get("purpose", semantics.get("intent", case.get("change_request", qa_id)))),
            parameters=MappingProxyType(params),
            input_hash=_identity(seed),
        )


@dataclass(frozen=True)
class FrozenPrimitiveReplay:
    primitive_type: str
    input_identity: str
    output: Mapping[str, Any]
    version: str = "dp00-offline-replay-v2"
    provenance: str = "repository-frozen deterministic derivation"
    deterministic_seed: str | None = None

    @property
    def replay_id(self) -> str:
        return f"replay:{self.primitive_type}:{_identity([self.input_identity, self.output, self.version])[:16]}"


@dataclass(frozen=True)
class EvaluatorOracle:
    """Evaluator-only envelope. Runtime APIs do not accept this type."""

    qa_id: str
    case_id: str
    values: Mapping[str, Any]

    @classmethod
    def from_values(cls, qa_id: str, case_id: str, values: Mapping[str, Any]) -> "EvaluatorOracle":
        return cls(qa_id, case_id, MappingProxyType(dict(values)))
