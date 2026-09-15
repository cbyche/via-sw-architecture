from __future__ import annotations

import copy
import hashlib
import json
import math
from pathlib import Path
import re
from statistics import NormalDist
from typing import Any

from .candidate_runtime import CandidateInput


ROOT = Path(__file__).resolve().parents[2]

POPULATION_PATHS = {
    "QA-01": ROOT / "benchmark/scenarios/qa-v1/qa01-fast-v1",
    "QA-02": ROOT / "benchmark/scenarios/qa-v1/qa02-goals-v1",
    "QA-03": ROOT / "benchmark/scenarios/qa-v1/qa03-continuity-v1",
    "QA-04": ROOT / "benchmark/evolution/qa-v1/qa04-agent-evolution-v1",
    "QA-05": ROOT / "benchmark/evolution/qa-v1/qa05-product-evolution-v1",
    "QA-06": ROOT / "benchmark/evolution/qa-v1/qa06-device-adaptation-v1",
    "QA-07": ROOT / "benchmark/scenarios/qa-v1/qa07-resource-v1",
    "QA-08": ROOT / "benchmark/failure-injection/qa-v1/qa08-recovery-v1",
    "QA-09": ROOT / "benchmark/scenarios/qa-v1/qa09-sensitive-scope-v1",
    "QA-10": ROOT / "benchmark/scenarios/qa-v1/qa10-trace-v1",
    "QA-11": ROOT / "benchmark/scenarios/qa-v1/qa11-feedback-v1",
    "QA-12": ROOT / "benchmark/scenarios/qa-v1/qa12-barge-in-v1"
}

GOAL_CATALOG_PATH = ROOT / "benchmark/scenarios/qa-v1/goals/instances/catalog.json"
LATENCY_REFERENCE_PATH = ROOT / "benchmark/contracts/dp00-qa01-design-reference-latency-v1.json"
HIDDEN_KEY_FRAGMENTS = ("oracle", "ground_truth", "expected_graph", "injected_fault_label")


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sanitize_candidate_value(value: Any) -> Any:
    """Remove evaluator-only labels recursively before candidate execution."""
    if isinstance(value, dict):
        return {
            key: sanitize_candidate_value(child)
            for key, child in value.items()
            if not any(fragment in key.lower() for fragment in HIDDEN_KEY_FRAGMENTS)
            and key != "candidate_telemetry_must_exclude"
        }
    if isinstance(value, list):
        return [sanitize_candidate_value(child) for child in value]
    return copy.deepcopy(value)


def contains_hidden_key(value: Any) -> bool:
    if isinstance(value, dict):
        return any(
            any(fragment in key.lower() for fragment in HIDDEN_KEY_FRAGMENTS)
            or key == "candidate_telemetry_must_exclude"
            or contains_hidden_key(child)
            for key, child in value.items()
        )
    if isinstance(value, list):
        return any(contains_hidden_key(child) for child in value)
    return False


def case_id(qa_id: str, case: dict[str, Any]) -> str:
    return case["goal_id"] if qa_id in {"QA-01", "QA-02"} else case["case_id"]


def load_cases(qa_id: str) -> list[dict[str, Any]]:
    if qa_id == "QA-07":
        return load_json(ROOT / "benchmark/fixtures/qa-v1/resource/qa07-resource-workload-v1.json")["phases"]
    instances = load_json(POPULATION_PATHS[qa_id] / "instances.json")["instances"]
    if qa_id not in {"QA-01", "QA-02"}:
        return instances
    catalog = load_json(GOAL_CATALOG_PATH)["instances"]
    resolved = []
    for membership in instances:
        match = re.search(r"#/instances/(\d+)$", membership["catalog_ref"])
        if not match:
            raise ValueError(f"invalid goal catalog reference: {membership['catalog_ref']}")
        goal = copy.deepcopy(catalog[int(match.group(1))])
        if goal["goal_id"] != membership["goal_id"]:
            raise ValueError("goal membership reference identity mismatch")
        resolved.append(goal)
    return resolved


def load_oracles(qa_id: str) -> dict[str, Any]:
    if qa_id in {"QA-01", "QA-02"}:
        return {item["goal_id"]: item["goal_oracle"] for item in load_json(GOAL_CATALOG_PATH)["instances"]}
    if qa_id == "QA-07":
        return load_json(POPULATION_PATHS[qa_id] / "oracles.json")["oracles"]
    return load_json(POPULATION_PATHS[qa_id] / "oracles.json")["oracles"]


def candidate_input(qa_id: str, case: dict[str, Any]) -> CandidateInput:
    payload = sanitize_candidate_value(case)
    if contains_hidden_key(payload):
        raise AssertionError("candidate-visible projection retained an evaluator-only key")
    return CandidateInput.from_sanitized(qa_id, case_id(qa_id, case), payload)


class LatencyReference:
    """Candidate-neutral deterministic samples from the frozen reference contract."""

    def __init__(self) -> None:
        contract = load_json(LATENCY_REFERENCE_PATH)
        self.operations = contract["operations"]
        self.profiles = {profile["profile_id"]: profile for profile in contract["models"]}
        self.primitives = contract["common_primitives"]

    @staticmethod
    def _uniform(key: str) -> float:
        raw = int.from_bytes(hashlib.sha256(key.encode("utf-8")).digest()[:8], "big")
        return (raw + 0.5) / (2**64)

    @classmethod
    def _lognormal(cls, key: str, p50: float, p95: float) -> float:
        mu = math.log(p50)
        sigma = math.log(p95 / p50) / 1.6448536269514722
        return math.exp(mu + sigma * NormalDist().inv_cdf(cls._uniform(key)))

    def model_ms(self, profile_id: str, semantic_signature: str, operation: str = "COMBINED_EXECUTION_DECISION") -> float:
        profile = self.profiles[profile_id]
        key = f"{profile_id}|{semantic_signature}|{operation}"
        ttft = self._lognormal(f"{key}|ttft", profile["ttft_p50_ms"], profile["ttft_p95_ms"])
        tpot = self._lognormal(f"{key}|tpot", profile["tpot_p50_ms"], profile["tpot_p95_ms"])
        return ttft + (self.operations[operation]["output_tokens"] - 1) * tpot

    def primitive_ms(self, primitive_id: str, semantic_signature: str) -> float:
        primitive = self.primitives[primitive_id]
        if primitive["distribution"] == "CONSTANT":
            return float(primitive["distribution_parameters"]["value_ms"])
        return self._lognormal(f"{primitive_id}|{semantic_signature}", primitive["p50_ms"], primitive["p95_ms"])

    def qa01_latency_seconds(self, profile_id: str, semantic_signature: str, *, local_tactic: bool, bounded_read: bool) -> float:
        response = self.primitive_ms("H", semantic_signature)
        if local_tactic:
            return (self.primitive_ms("X2", semantic_signature) + response) / 1000.0
        execution = self.primitive_ms("X2" if bounded_read else "X3", semantic_signature)
        dispatch = self.primitive_ms("H", semantic_signature)
        return (self.model_ms(profile_id, semantic_signature) + dispatch + execution + response) / 1000.0
