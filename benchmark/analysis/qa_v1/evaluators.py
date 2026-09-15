from __future__ import annotations

from abc import ABC, abstractmethod
import math
import re
from typing import Any, Iterable, Mapping

from .contract import ContractRepository
from .models import EvaluationResult, ProvenanceEnvelope
from .scoring import ScoreEngine


def _nearest_rank_p95(values: Iterable[float]) -> float:
    ordered = sorted(float(value) for value in values)
    if not ordered:
        raise ValueError("p95 requires at least one value")
    if any(not math.isfinite(value) or value < 0 for value in ordered):
        raise ValueError("latency values must be finite and non-negative")
    return ordered[math.ceil(0.95 * len(ordered)) - 1]


def _percent(passing: int, total: int) -> float:
    if total <= 0:
        raise ValueError("rate metrics require a non-empty population")
    return 100.0 * passing / total


def _contract_penalty(qa: Mapping[str, Any]) -> float:
    match = re.search(r"(\d+(?:\.\d+)?)\s*(?:-|\s)?(?:seconds?|ms)\s+penalty", qa["formula"])
    if not match:
        raise ValueError(f"{qa['qa_id']} formula does not expose its latency penalty")
    return float(match.group(1))


class BaseEvaluator(ABC):
    qa_id: str

    def __init__(self, contracts: ContractRepository | None = None) -> None:
        self.contracts = contracts or ContractRepository()
        self.scoring = ScoreEngine(self.contracts)
        self.qa = self.contracts.qa(self.qa_id)

    def _validate_provenance(self, provenance: ProvenanceEnvelope) -> None:
        provenance.validate()
        expected = {
            "qa_contract_id": self.contracts.contract["contract_id"],
            "qa_contract_hash": self.contracts.sha256(self.contracts.contract_path),
            "reference_environment_id": self.contracts.environment["environment_id"],
            "reference_environment_hash": self.contracts.sha256(self.contracts.environment_path),
            "corpus_manifest_id": self.contracts.manifest["manifest_id"],
            "corpus_manifest_hash": self.contracts.sha256(self.contracts.manifest_path),
        }
        for field, value in expected.items():
            if getattr(provenance, field) != value:
                raise ValueError(f"provenance {field} does not match the loaded QA-v1 artifact")

    def evaluate(
        self,
        population: Mapping[str, Any],
        canonical_observations: Iterable[Mapping[str, Any]],
        provenance: ProvenanceEnvelope,
    ) -> EvaluationResult:
        self._validate_provenance(provenance)
        population_id = population.get("population_id")
        if population_id != self.qa["population_id"]:
            raise ValueError(f"{self.qa_id} requires population {self.qa['population_id']!r}")
        manifest_population = self.contracts.population(population_id)
        observations = [dict(observation) for observation in canonical_observations]

        if population.get("complete") is not True:
            return EvaluationResult(
                qa_id=self.qa_id,
                population_id=population_id,
                population_manifest_hash=manifest_population["population_manifest_sha256"],
                official_scalar_metric=self.qa["official_scalar_metric"],
                raw_metric=None,
                unit=self.qa["unit"],
                direction=self.qa["direction"],
                score=None,
                target_met=None,
                eligibility_status="INCOMPLETE_POPULATION",
                qualification_status="NOT_EVALUATED",
                failed_gates=(),
                diagnostics={"observed_count": len(observations), "population_complete": False},
                provenance=provenance,
            )

        planned_size = manifest_population["planned_size"]
        if planned_size is not None and len(observations) != planned_size:
            raise ValueError(
                f"complete {population_id} requires {planned_size} observations, got {len(observations)}"
            )
        expected_case_ids = population.get("case_ids")
        if expected_case_ids is not None:
            observation_ids = [item.get("observation_id") for item in observations]
            if any(value is None for value in observation_ids):
                raise ValueError("canonical observations require observation_id for frozen-population evaluation")
            if len(observation_ids) != len(set(observation_ids)):
                raise ValueError("canonical observation identities must be unique")
            if set(observation_ids) != set(expected_case_ids):
                raise ValueError(f"{population_id} observations do not exactly cover the frozen case identities")
        raw_metric, diagnostics, gates = self._compute(observations)
        outcome = self.scoring.score(
            self.qa_id,
            raw_metric,
            unit=self.qa["unit"],
            direction=self.qa["direction"],
            gate_results=gates,
        )
        return EvaluationResult(
            qa_id=self.qa_id,
            population_id=population_id,
            population_manifest_hash=manifest_population["population_manifest_sha256"],
            official_scalar_metric=self.qa["official_scalar_metric"],
            raw_metric=raw_metric,
            unit=self.qa["unit"],
            direction=self.qa["direction"],
            score=outcome.score,
            target_met=outcome.target_met,
            eligibility_status="ELIGIBLE",
            qualification_status=outcome.qualification_status,
            failed_gates=outcome.failed_gates,
            diagnostics={**diagnostics, "observed_count": len(observations), "population_complete": True},
            provenance=provenance,
        )

    @abstractmethod
    def _compute(self, observations: list[dict[str, Any]]) -> tuple[float, dict[str, Any], dict[str, bool]]:
        raise NotImplementedError


class QA01Evaluator(BaseEvaluator):
    qa_id = "QA-01"

    def _compute(self, observations):
        penalty = _contract_penalty(self.qa)
        latencies = []
        penalties = 0
        for item in observations:
            if not item.get("useful_outcome_correct", False):
                latencies.append(penalty)
                penalties += 1
                continue
            endpoints = (item.get("voice_facts_audible_seconds"), item.get("text_details_available_seconds"))
            if any(value is None for value in endpoints):
                latencies.append(penalty)
                penalties += 1
                continue
            latency = max(float(value) for value in endpoints) - float(item["start_seconds"])
            if latency > penalty:
                latency = penalty
                penalties += 1
            latencies.append(latency)
        return _nearest_rank_p95(latencies), {"penalty_count": penalties, "latencies_seconds": latencies}, {}


class QA02Evaluator(BaseEvaluator):
    qa_id = "QA-02"

    def _compute(self, observations):
        passing = sum(bool(item.get("required_conditions")) and all(value is True for value in item["required_conditions"].values()) for item in observations)
        return _percent(passing, len(observations)), {"passing": passing, "failing": len(observations) - passing}, {}


class QA03Evaluator(BaseEvaluator):
    qa_id = "QA-03"

    def _compute(self, observations):
        passing = sum(bool(item.get("required_relations")) and all(value is True for value in item["required_relations"].values()) for item in observations)
        return _percent(passing, len(observations)), {"passing": passing, "failing": len(observations) - passing}, {}


class QA04Evaluator(BaseEvaluator):
    qa_id = "QA-04"

    def _compute(self, observations):
        def contained(item):
            changed_areas = set(item.get("actual_semantic_ownership_changes", []))
            allowed_areas = set(item.get("allowed_agent_integration_ownership_areas", []))
            used_seams = set(item.get("actual_extension_seams", []))
            approved_seams = set(item.get("approved_extension_seams", []))
            changed_zones = set(item.get("actual_core_semantic_zone_changes", []))
            forbidden_zones = set(item.get("forbidden_core_semantic_zones", []))
            return (
                item.get("requested_change_completed") is True
                and item.get("common_regressions_pass") is True
                and bool(changed_areas)
                and changed_areas <= allowed_areas
                and used_seams <= approved_seams
                and not changed_zones.intersection(forbidden_zones)
            )

        passing = sum(contained(item) for item in observations)
        return _percent(passing, len(observations)), {"contained": passing, "not_contained": len(observations) - passing}, {}


class QA05Evaluator(BaseEvaluator):
    qa_id = "QA-05"

    def _compute(self, observations):
        def contained(item):
            changed_zones = set(item.get("actual_changed_semantic_zones", []))
            expected_zones = set(item.get("expected_ownership_zones", []))
            used_seams = set(item.get("actual_extension_seams", []))
            approved_seams = set(item.get("approved_extension_seams", []))
            leaked_zones = set(item.get("semantic_dependency_leaks", []))
            return (
                item.get("requested_functionality_completed") is True
                and item.get("common_regressions_pass") is True
                and bool(changed_zones)
                and changed_zones <= expected_zones
                and used_seams <= approved_seams
                and not leaked_zones
            )

        passing = sum(contained(item) for item in observations)
        return _percent(passing, len(observations)), {"contained": passing, "not_contained": len(observations) - passing}, {}


class QA06Evaluator(BaseEvaluator):
    qa_id = "QA-06"

    def _compute(self, observations):
        passing = sum(
            item.get("required_functionality_delivered") is True
            and not item.get("actual_core_semantic_changes", [])
            for item in observations
        )
        by_device: dict[str, dict[str, int]] = {}
        for item in observations:
            bucket = by_device.setdefault(item.get("device_family", "UNKNOWN"), {"total": 0, "passing": 0})
            bucket["total"] += 1
            bucket["passing"] += int(
                item.get("required_functionality_delivered") is True
                and not item.get("actual_core_semantic_changes", [])
            )
        return _percent(passing, len(observations)), {"passing": passing, "by_device": by_device}, {}


class QA07Evaluator(BaseEvaluator):
    qa_id = "QA-07"

    def _compute(self, observations):
        if not observations:
            raise ValueError("QA-07 requires a complete memory event timeline")
        denominators = {int(item["minimum_mandatory_bytes"]) for item in observations}
        if len(denominators) != 1 or next(iter(denominators)) <= 0:
            raise ValueError("QA-07 requires one positive frozen denominator across the timeline")
        peak = max(int(item["candidate_resident_bytes"]) for item in observations)
        if peak < 0:
            raise ValueError("candidate resident memory must be non-negative")
        denominator = next(iter(denominators))
        return peak / denominator, {"peak_candidate_bytes": peak, "minimum_mandatory_bytes": denominator}, {}


class QA08Evaluator(BaseEvaluator):
    qa_id = "QA-08"

    def _compute(self, observations):
        penalty = _contract_penalty(self.qa)
        latencies = []
        penalties = 0
        unsafe = 0
        for item in observations:
            unsafe += int(item.get("unsafe_or_incorrect_recovery") is True)
            if item.get("safe_recovery_reached") is not True or item.get("safe_recovery_seconds") is None:
                latencies.append(penalty)
                penalties += 1
            else:
                latency = float(item["safe_recovery_seconds"])
                if latency > penalty:
                    latency = penalty
                    penalties += 1
                latencies.append(latency)
        gate = {self.qa["hard_eligibility_gates"][0]["gate_id"]: unsafe == 0}
        return _nearest_rank_p95(latencies), {"penalty_count": penalties, "unsafe_or_incorrect_count": unsafe}, gate


class QA09Evaluator(BaseEvaluator):
    qa_id = "QA-09"

    def _compute(self, observations):
        tp = fp = fn = 0
        hard_violations = 0
        for item in observations:
            required = set(item.get("required_scopes", []))
            granted = set(item.get("granted_or_exposed_scopes", []))
            tp += len(required & granted)
            fp += len(granted - required)
            fn += len(required - granted)
            hard_violations += len(item.get("confirmed_hard_gate_triggers", []))
        if tp == fp == fn == 0:
            raise ValueError("QA-09 empty-scope behavior must be fixed by the frozen scenario oracle")
        precision = tp / (tp + fp) if tp + fp else 0.0
        recall = tp / (tp + fn) if tp + fn else 0.0
        f1 = 2 * precision * recall / (precision + recall) if precision + recall else 0.0
        gate = {self.qa["hard_eligibility_gates"][0]["gate_id"]: hard_violations == 0}
        return 100.0 * f1, {"true_positive": tp, "false_positive": fp, "false_negative": fn, "precision": precision, "recall": recall, "hard_violation_count": hard_violations}, gate


class QA10Evaluator(BaseEvaluator):
    qa_id = "QA-10"

    def _compute(self, observations):
        def reconstructable(item):
            chain_ok = bool(item.get("required_chain")) and all(
                value is True for value in item["required_chain"].values()
            )
            expected = item.get("expected_causal_graph")
            reconstructed = item.get("reconstructed_causal_graph")
            graph_ok = expected is None or (
                reconstructed is not None
                and set(map(tuple, expected.get("edges", []))) == set(map(tuple, reconstructed.get("edges", [])))
                and set(expected.get("nodes", [])) == set(reconstructed.get("nodes", []))
            )
            return chain_ok and graph_ok and item.get("hidden_oracle_or_fault_labels_exposed") is False

        passing = sum(reconstructable(item) for item in observations)
        return _percent(passing, len(observations)), {"reconstructable": passing, "not_reconstructable": len(observations) - passing}, {}


class QA11Evaluator(BaseEvaluator):
    qa_id = "QA-11"

    def _compute(self, observations):
        penalty = _contract_penalty(self.qa)
        latencies = []
        penalties = 0
        by_event_class: dict[str, list[float]] = {}
        for item in observations:
            if item.get("useful_feedback") is not True or item.get("feedback_received_seconds") is None:
                latency = penalty
                penalties += 1
            else:
                latency = float(item["feedback_received_seconds"]) - float(item["event_available_seconds"])
            latencies.append(latency)
            by_event_class.setdefault(item.get("event_class", "UNKNOWN"), []).append(latency)
        diagnostics = {"penalty_count": penalties, "event_class_p95_seconds": {key: _nearest_rank_p95(values) for key, values in by_event_class.items()}}
        return _nearest_rank_p95(latencies), diagnostics, {}


class QA12Evaluator(BaseEvaluator):
    qa_id = "QA-12"

    def _compute(self, observations):
        penalty = _contract_penalty(self.qa)
        latencies = []
        penalties = 0
        for item in observations:
            stop = item.get("last_audible_sample_ms")
            if stop is None:
                latency = penalty
                penalties += 1
            else:
                latency = float(stop) - float(item["user_speech_onset_ms"])
                if latency > penalty:
                    latency = penalty
                    penalties += 1
            latencies.append(latency)
        return _nearest_rank_p95(latencies), {"penalty_count": penalties, "latencies_ms": latencies}, {}


EVALUATOR_TYPES = {
    "QA-01": QA01Evaluator,
    "QA-02": QA02Evaluator,
    "QA-03": QA03Evaluator,
    "QA-04": QA04Evaluator,
    "QA-05": QA05Evaluator,
    "QA-06": QA06Evaluator,
    "QA-07": QA07Evaluator,
    "QA-08": QA08Evaluator,
    "QA-09": QA09Evaluator,
    "QA-10": QA10Evaluator,
    "QA-11": QA11Evaluator,
    "QA-12": QA12Evaluator,
}


def evaluate_qa(
    qa_id: str,
    population: Mapping[str, Any],
    canonical_observations: Iterable[Mapping[str, Any]],
    provenance: ProvenanceEnvelope,
    contracts: ContractRepository | None = None,
) -> EvaluationResult:
    try:
        evaluator_type = EVALUATOR_TYPES[qa_id]
    except KeyError as exc:
        raise ValueError(f"unknown QA-v1 evaluator: {qa_id}") from exc
    return evaluator_type(contracts).evaluate(population, canonical_observations, provenance)


def evaluate_canonical_observations(
    qa_id: str,
    population: Mapping[str, Any],
    observation_envelopes: Iterable[Mapping[str, Any]],
    provenance: ProvenanceEnvelope,
    contracts: ContractRepository | None = None,
) -> EvaluationResult:
    """Validate DP-neutral observation envelopes, then run the sole QA path."""
    measurements = []
    for envelope in observation_envelopes:
        if envelope.get("qa_id") != qa_id:
            raise ValueError("canonical observation qa_id mismatch")
        if envelope.get("population_id") != population.get("population_id"):
            raise ValueError("canonical observation population_id mismatch")
        if envelope.get("evidence_mode") != provenance.evidence_mode.value:
            raise ValueError("canonical observation evidence_mode mismatch")
        measurement = dict(envelope.get("measurement", {}))
        measurement["observation_id"] = envelope.get("observation_id")
        measurements.append(measurement)
    return evaluate_qa(qa_id, population, measurements, provenance, contracts)
