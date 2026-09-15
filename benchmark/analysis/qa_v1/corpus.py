from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

from .contract import ContractRepository


class FrozenCorpus:
    """Read and verify the materialized population identities used by evaluators."""

    def __init__(self, contracts: ContractRepository | None = None) -> None:
        self.contracts = contracts or ContractRepository()
        self.root = self.contracts.root

    @staticmethod
    def _sha256(path: Path) -> str:
        return hashlib.sha256(path.read_bytes()).hexdigest()

    @staticmethod
    def _load(path: Path) -> dict[str, Any]:
        with path.open(encoding="utf-8") as handle:
            return json.load(handle)

    def population(self, qa_id: str) -> dict[str, Any]:
        qa = self.contracts.qa(qa_id)
        entry = self.contracts.population(qa["population_id"])
        if entry.get("status") != "frozen":
            return {"population_id": qa["population_id"], "complete": False, "case_ids": []}

        manifest_path = self.root / entry["population_manifest"]
        if self._sha256(manifest_path) != entry["population_manifest_sha256"]:
            raise ValueError(f"{qa_id} population manifest hash mismatch")
        manifest = self._load(manifest_path)
        if manifest["population_id"] != qa["population_id"] or manifest["qa_id"] != qa_id:
            raise ValueError(f"{qa_id} population manifest identity mismatch")
        for key, artifact in manifest["artifacts"].items():
            path = self.root / artifact
            expected = {
                "families": manifest["family_manifest_hash"],
                "instances": manifest["materialized_instance_hash"],
                "oracles": manifest["oracle_hash"],
            }[key]
            if self._sha256(path) != expected:
                raise ValueError(f"{qa_id} {key} hash mismatch")

        instance_document = self._load(self.root / manifest["artifacts"]["instances"])
        if manifest["population_kind"] == "fixed_workload_timeline":
            case_ids = [phase["phase_id"] for phase in instance_document["phases"]]
        else:
            instances = instance_document["instances"]
            key = "goal_id" if qa_id in {"QA-01", "QA-02"} else "case_id"
            case_ids = [item[key] for item in instances]
        if len(case_ids) != len(set(case_ids)):
            raise ValueError(f"{qa_id} contains duplicate case identities")
        actual_size = manifest["actual_size"]
        if actual_size is not None and len(case_ids) != actual_size:
            raise ValueError(f"{qa_id} materialized count does not match its population manifest")
        return {
            "population_id": qa["population_id"],
            "complete": True,
            "population_manifest_hash": entry["population_manifest_sha256"],
            "case_ids": case_ids,
            "population_kind": manifest["population_kind"],
        }
