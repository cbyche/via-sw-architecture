"""Shared, DP-neutral QA Evaluation Contract v1 evaluation foundation."""

from .contract import ContractRepository
from .corpus import FrozenCorpus
from .evaluators import EVALUATOR_TYPES, evaluate_canonical_observations, evaluate_qa
from .models import EvidenceMode, EvaluationResult, ProvenanceEnvelope, ScoreOutcome
from .scoring import ScoreEngine

__all__ = [
    "ContractRepository",
    "EVALUATOR_TYPES",
    "EvidenceMode",
    "EvaluationResult",
    "FrozenCorpus",
    "ProvenanceEnvelope",
    "ScoreEngine",
    "ScoreOutcome",
    "evaluate_canonical_observations",
    "evaluate_qa",
]
