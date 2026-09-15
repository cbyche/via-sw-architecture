from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

from .models import EvidenceMode, ProvenanceEnvelope


class ContractRepository:
    """Loads immutable QA-v1 definitions and computes their artifact identities."""

    def __init__(self, repository_root: Path | None = None) -> None:
        self.root = repository_root or Path(__file__).resolve().parents[3]
        self.contract_path = self.root / "benchmark/contracts/qa-v1/qa-evaluation-contract-v1.json"
        self.environment_path = self.root / "benchmark/contracts/qa-v1/reference-environment-v1.json"
        self.manifest_path = self.root / "benchmark/contracts/qa-v1/corpus-manifest-v1.json"
        self.contract = self._load(self.contract_path)
        self.environment = self._load(self.environment_path)
        self.manifest = self._load(self.manifest_path)
        self._qas = {qa["qa_id"]: qa for qa in self.contract["quality_attributes"]}
        self._populations = {item["population_id"]: item for item in self.manifest["populations"]}

    @staticmethod
    def _load(path: Path) -> dict[str, Any]:
        with path.open("r", encoding="utf-8") as handle:
            return json.load(handle)

    @staticmethod
    def sha256(path: Path) -> str:
        digest = hashlib.sha256()
        with path.open("rb") as handle:
            for chunk in iter(lambda: handle.read(1024 * 1024), b""):
                digest.update(chunk)
        return digest.hexdigest()

    def qa(self, qa_id: str) -> dict[str, Any]:
        try:
            return self._qas[qa_id]
        except KeyError as exc:
            raise ValueError(f"unknown QA-v1 id: {qa_id}") from exc

    def population(self, population_id: str) -> dict[str, Any]:
        try:
            return self._populations[population_id]
        except KeyError as exc:
            raise ValueError(f"unknown QA-v1 population: {population_id}") from exc

    def build_provenance(
        self,
        *,
        architecture_commit: str,
        dp_id: str,
        alternative_id: str,
        tactic_package: str | None,
        run_id: str,
        evidence_mode: EvidenceMode,
    ) -> ProvenanceEnvelope:
        envelope = ProvenanceEnvelope(
            qa_contract_id=self.contract["contract_id"],
            qa_contract_hash=self.sha256(self.contract_path),
            reference_environment_id=self.environment["environment_id"],
            reference_environment_hash=self.sha256(self.environment_path),
            corpus_manifest_id=self.manifest["manifest_id"],
            corpus_manifest_hash=self.sha256(self.manifest_path),
            architecture_commit=architecture_commit,
            dp_id=dp_id,
            alternative_id=alternative_id,
            tactic_package=tactic_package,
            run_id=run_id,
            evidence_mode=evidence_mode,
        )
        envelope.validate()
        return envelope
