from __future__ import annotations

from dataclasses import asdict, dataclass
from enum import Enum
import re
from typing import Any


SHA256_RE = re.compile(r"^[a-f0-9]{64}$")
COMMIT_RE = re.compile(r"^[a-f0-9]{40}$")
DP_RE = re.compile(r"^DP-[0-9]{2,}$")


class EvidenceMode(str, Enum):
    EXECUTED = "executed"
    SEMANTIC_REPLAY = "semantic_replay"
    DESIGN_TIME_SIMULATION = "design_time_simulation"
    STRUCTURAL_ANALYSIS = "structural_analysis"


@dataclass(frozen=True)
class ProvenanceEnvelope:
    qa_contract_id: str
    qa_contract_hash: str
    reference_environment_id: str
    reference_environment_hash: str
    corpus_manifest_id: str
    corpus_manifest_hash: str
    architecture_commit: str
    dp_id: str
    alternative_id: str
    tactic_package: str | None
    run_id: str
    evidence_mode: EvidenceMode

    def validate(self) -> None:
        for name in ("qa_contract_hash", "reference_environment_hash", "corpus_manifest_hash"):
            if not SHA256_RE.fullmatch(getattr(self, name)):
                raise ValueError(f"{name} must be a lowercase SHA-256 hex digest")
        if not COMMIT_RE.fullmatch(self.architecture_commit):
            raise ValueError("architecture_commit must be a full 40-character lowercase Git SHA")
        if not DP_RE.fullmatch(self.dp_id):
            raise ValueError("dp_id must use DP-NN form")
        if not self.alternative_id or not self.run_id:
            raise ValueError("alternative_id and run_id are required")
        if not isinstance(self.evidence_mode, EvidenceMode):
            raise ValueError("evidence_mode must be an EvidenceMode")

    def to_dict(self) -> dict[str, Any]:
        value = asdict(self)
        value["evidence_mode"] = self.evidence_mode.value
        return value


@dataclass(frozen=True)
class ScoreOutcome:
    score: int
    target_met: bool
    qualification_status: str
    failed_gates: tuple[str, ...]

    def to_dict(self) -> dict[str, Any]:
        value = asdict(self)
        value["failed_gates"] = list(self.failed_gates)
        return value


@dataclass(frozen=True)
class EvaluationResult:
    qa_id: str
    population_id: str
    population_manifest_hash: str
    official_scalar_metric: str
    raw_metric: float | None
    unit: str
    direction: str
    score: int | None
    target_met: bool | None
    eligibility_status: str
    qualification_status: str
    failed_gates: tuple[str, ...]
    diagnostics: dict[str, Any]
    provenance: ProvenanceEnvelope

    def to_dict(self) -> dict[str, Any]:
        value = asdict(self)
        value["failed_gates"] = list(self.failed_gates)
        value["provenance"] = self.provenance.to_dict()
        return value
