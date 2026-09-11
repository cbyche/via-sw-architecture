"""Read-only domain models for DP-00 offline analysis."""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class ValidationIssue:
    code: str
    message: str
    path: str | None = None
    severity: str = "ERROR"


@dataclass(frozen=True)
class ValidationReport:
    errors: tuple[ValidationIssue, ...] = ()
    warnings: tuple[ValidationIssue, ...] = ()

    @property
    def valid(self) -> bool:
        return not self.errors


@dataclass(frozen=True)
class ActualSemanticFacts:
    referent_bindings: tuple[dict[str, Any], ...] = ()
    task_associations: tuple[dict[str, Any], ...] = ()
    execution_invocations: tuple[dict[str, Any], ...] = ()
    execution_routes: tuple[dict[str, Any], ...] = ()
    clarifications: tuple[dict[str, Any], ...] = ()
    result_bindings: tuple[dict[str, Any], ...] = ()
    observable_effects: tuple[dict[str, Any], ...] = ()
    failure_outcomes: tuple[str, ...] = ()
    route_candidates: tuple[dict[str, Any], ...] = ()
    rejected_routes: tuple[dict[str, Any], ...] = ()
    derived_predicates: frozenset[str] = frozenset()


@dataclass(frozen=True)
class EpisodeEvidence:
    raw_directory: Path
    provenance: dict[str, Any]
    scenario: dict[str, Any]
    canonical_events: tuple[dict[str, Any], ...]
    model_calls: tuple[dict[str, Any], ...]
    fixture_events: tuple[dict[str, Any], ...]
    actual: ActualSemanticFacts
    expected: dict[str, Any] | None = None
    integrity_issues: tuple[ValidationIssue, ...] = ()


@dataclass(frozen=True)
class EvidenceSet:
    raw_root: Path
    campaign_provenance: dict[str, Any] | None
    episodes: tuple[EpisodeEvidence, ...]
    validation: ValidationReport
    coverage_map: dict[str, Any]
    method_config: dict[str, Any] = field(default_factory=dict)
