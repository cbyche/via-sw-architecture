from __future__ import annotations

import math
from typing import Mapping

from .contract import ContractRepository
from .models import ScoreOutcome


class ScoreEngine:
    """The single DP-neutral route from a QA-v1 scalar to its contract score."""

    def __init__(self, contracts: ContractRepository | None = None) -> None:
        self.contracts = contracts or ContractRepository()

    @staticmethod
    def _contains(band: dict, value: float) -> bool:
        lower = band["min"]
        upper = band["max"]
        lower_ok = lower is None or value > lower or (band["min_inclusive"] and value == lower)
        upper_ok = upper is None or value < upper or (band["max_inclusive"] and value == upper)
        return lower_ok and upper_ok

    @staticmethod
    def _target_met(target: dict, value: float) -> bool:
        if target["operator"] == "<=":
            return value <= target["value"]
        if target["operator"] == ">=":
            return value >= target["value"]
        raise ValueError(f"unsupported target operator: {target['operator']}")

    def score(
        self,
        qa_id: str,
        raw_metric: float,
        *,
        unit: str,
        direction: str | None = None,
        gate_results: Mapping[str, bool] | None = None,
    ) -> ScoreOutcome:
        qa = self.contracts.qa(qa_id)
        if unit != qa["unit"]:
            raise ValueError(f"{qa_id} requires unit {qa['unit']!r}, got {unit!r}")
        if direction is not None and direction != qa["direction"]:
            raise ValueError(f"{qa_id} requires direction {qa['direction']!r}, got {direction!r}")
        if not isinstance(raw_metric, (int, float)) or isinstance(raw_metric, bool) or not math.isfinite(raw_metric):
            raise ValueError("raw_metric must be a finite number")
        if raw_metric < 0 or (unit == "percent" and raw_metric > 100):
            raise ValueError(f"raw_metric {raw_metric} is outside the {unit} domain")

        matches = [band for band in qa["score_bands"] if self._contains(band, float(raw_metric))]
        if len(matches) != 1:
            raise ValueError(f"{qa_id} value {raw_metric} matched {len(matches)} score bands")

        required_gates = qa.get("hard_eligibility_gates", [])
        supplied = dict(gate_results or {})
        missing = [gate["gate_id"] for gate in required_gates if gate["gate_id"] not in supplied]
        if missing:
            raise ValueError(f"missing hard-gate results for {qa_id}: {missing}")
        unknown = set(supplied) - {gate["gate_id"] for gate in required_gates}
        if unknown:
            raise ValueError(f"unknown hard-gate results for {qa_id}: {sorted(unknown)}")
        failed = tuple(gate["gate_id"] for gate in required_gates if not supplied[gate["gate_id"]])

        return ScoreOutcome(
            score=matches[0]["score"],
            target_met=self._target_met(qa["target"], float(raw_metric)),
            qualification_status="DISQUALIFIED" if failed else "QUALIFIED",
            failed_gates=failed,
        )
