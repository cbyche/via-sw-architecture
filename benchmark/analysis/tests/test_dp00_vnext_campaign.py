import hashlib
import json
from pathlib import Path

from benchmark.analysis.qa_v1 import ContractRepository, EvidenceMode, evaluate_canonical_observations
from benchmark.dp_vnext.candidate_runtime import CANDIDATE_IDS, CandidateInput, CandidateRuntime
from benchmark.dp_vnext.campaign import EXPECTED_COUNTS, REGISTRATION_PATH
from benchmark.dp_vnext.controls import LatencyReference, candidate_input, contains_hidden_key, load_cases


ROOT = Path(__file__).resolve().parents[3]


def _load(path):
    return json.loads(path.read_text(encoding="utf-8"))


def _walk_keys(value):
    if isinstance(value, dict):
        for key, child in value.items():
            yield key
            yield from _walk_keys(child)
    elif isinstance(value, list):
        for child in value:
            yield from _walk_keys(child)


def _eligible_input(**semantic_overrides):
    semantic = {"read_only": True, "consent_required": False, **semantic_overrides}
    return CandidateInput.from_sanitized(
        "QA-01",
        "GOAL-TEST-01",
        {
            "expected_task_semantics": semantic,
            "agent_capability_requirements": ["bounded-read"],
            "user_input": {"committed_representation": "device-status-read:fixture-1:v1"},
            "required_result_facts": ["fact-1"]
        }
    )


def test_all_three_candidate_identities_were_preregistered_before_evaluation():
    registration = _load(REGISTRATION_PATH)
    assert registration["status"] == "FROZEN_BEFORE_COMPARATIVE_EXECUTION"
    assert tuple(item["candidate_id"] for item in registration["candidates"]) == CANDIDATE_IDS == ("R1", "R3", "R1+@")
    assert [item["classification"] for item in registration["candidates"]] == ["base_family", "base_family", "tactic_realization"]
    assert registration["candidates"][2]["inherits"] == "R1"
    assert registration["all_candidates_run_together"] is True
    assert registration["post_result_tuning_forbidden"] is True


def test_candidate_execution_receives_no_hidden_evaluator_labels():
    raw = {
        "goal_id": "GOAL-TEST-01",
        "goal_oracle": {"correct_goal": "secret"},
        "ground_truth_fault_onset_event": "secret",
        "expected_task_semantics": {"read_only": True, "memory_oracle": {"operation": "secret"}},
        "user_input": {"committed_representation": "device-status-read:fixture-1:v1"},
        "agent_capability_requirements": ["bounded-read"],
        "required_result_facts": ["fact-1"]
    }
    visible = candidate_input("QA-01", raw)
    assert not contains_hidden_key(dict(visible.payload))
    serialized = json.dumps(dict(visible.payload)).lower()
    assert "secret" not in serialized
    source = (ROOT / "benchmark/dp_vnext/candidate_runtime.py").read_text(encoding="utf-8").lower()
    assert "oracles.json" not in source
    assert "goal_oracle" not in source
    assert CandidateRuntime("R1").execute_goal(visible)["goal"] == "device-status-read"


def test_exact_frozen_population_coverage_is_enforced_by_source_counts():
    for qa_id, expected in EXPECTED_COUNTS.items():
        cases = load_cases(qa_id)
        identities = [case["goal_id"] if qa_id in {"QA-01", "QA-02"} else case["case_id"] for case in cases]
        assert len(cases) == expected
        assert len(set(identities)) == expected
    assert [phase["phase_id"] for phase in load_cases("QA-07")] == ["P0", "P1", "P2", "P3", "P4", "P5"]


def test_candidates_and_campaign_define_no_qa_target_or_score_table():
    paths = list((ROOT / "benchmark/contracts/dp-vnext").glob("dp00-*-candidate-v1.json"))
    paths.append(ROOT / "benchmark/contracts/dp-vnext/dp00-r1-readonly-fastpath-tactic-v1.json")
    paths.extend((ROOT / "benchmark/contracts/dp-vnext/evaluation").glob("*.json"))
    forbidden = {"target", "targets", "score_band", "score_bands", "scoring_table", "population_override"}
    for path in paths:
        assert not (set(_walk_keys(_load(path))) & forbidden), path


def test_comparable_candidates_consume_identical_frozen_reference_samples():
    latency = LatencyReference()
    signature = "candidate-neutral-semantic-signature"
    r1 = latency.qa01_latency_seconds("QWEN3_MEDIUM_REFERENCE", signature, local_tactic=False, bounded_read=True)
    r3 = latency.qa01_latency_seconds("QWEN3_MEDIUM_REFERENCE", signature, local_tactic=False, bounded_read=True)
    r1_at_delegated = latency.qa01_latency_seconds("QWEN3_MEDIUM_REFERENCE", signature, local_tactic=False, bounded_read=True)
    assert r1 == r3 == r1_at_delegated
    assert latency.model_ms("QWEN3_MEDIUM_REFERENCE", signature) == latency.model_ms("QWEN3_MEDIUM_REFERENCE", signature)


def test_r1_readonly_tactic_rejects_every_forbidden_behavior():
    runtime = CandidateRuntime("R1+@")
    assert runtime.readonly_tactic_eligible(_eligible_input()) is True
    assert runtime.readonly_tactic_eligible(_eligible_input(read_only=False)) is False
    assert runtime.readonly_tactic_eligible(_eligible_input(arbitrary_tool_selection=True)) is False
    assert runtime.readonly_tactic_eligible(_eligible_input(planning_required=True)) is False
    assert runtime.readonly_tactic_eligible(_eligible_input(durable_workflow=True)) is False
    assert runtime.readonly_tactic_eligible(_eligible_input(independent_agent_execution_state=True)) is False


def test_qa07_missing_calibration_cannot_receive_a_fabricated_denominator():
    status = _load(ROOT / "benchmark/contracts/dp-vnext/evaluation/qa07-evidence-status-v1.json")
    environment = _load(ROOT / "benchmark/contracts/qa-v1/reference-environment-v1.json")
    workload = _load(ROOT / "benchmark/fixtures/qa-v1/resource/qa07-resource-workload-v1.json")
    assert status["official_score_allowed"] is False
    assert environment["memory"]["calibration_status"] == "PLANNED_BEFORE_QA07_QUALIFICATION"
    assert "no measured device memory" in workload["claim_limit"]
    contracts = ContractRepository(ROOT)
    provenance = contracts.build_provenance(
        architecture_commit="a" * 40,
        dp_id="DP-00",
        alternative_id="R1",
        tactic_package=None,
        run_id="qa07-no-fabrication-test",
        evidence_mode=EvidenceMode.SEMANTIC_REPLAY
    )
    result = evaluate_canonical_observations("QA-07", {"population_id": "qa07-resource-v1", "complete": False}, [], provenance, contracts)
    assert result.raw_metric is None
    assert result.score is None
    assert result.eligibility_status == "INCOMPLETE_POPULATION"


def test_historical_abcd_evidence_identity_files_are_byte_unchanged():
    expected = {
        "benchmark/contracts/qa-v1/historical-evidence-registry-v1.json": "c6c3583f3012a46f582e7f6669eaee1e5ff175e0ec4309fcbdbae63209eb0921",
        "results/reports/pilot-v0/dp00-architecture-20260911T060300Z-83d199d/sha256-inventory.json": "55d9cd4be75f36c326fccfc422429b6d194f3b5ae41efe0bf9f5fcfbefac9789",
        "results/reports/qa03-v1/dp00-qa03-20260911T091004Z-752c6ac/sha256-inventory.json": "7a45053d3d7b308c3be6d2121fdd512c04ba737b788c570dddece060dbd2285d"
    }
    for relative, digest in expected.items():
        assert hashlib.sha256((ROOT / relative).read_bytes()).hexdigest() == digest
