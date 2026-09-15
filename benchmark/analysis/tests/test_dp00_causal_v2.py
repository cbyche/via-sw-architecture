import hashlib
import json
from pathlib import Path

from benchmark.dp_causal_v2.architecture import CausalCandidateRuntime, RuntimeMutation
from benchmark.dp_causal_v2.observation import ObservationAdapter
from benchmark.dp_causal_v2.primitives import OPERATION_COST_SOURCE
from benchmark.dp_causal_v2.schemas import CandidateVisibleInput, EvaluatorOracle
from benchmark.dp_causal_v2.state_machine import LifecycleStateMachine
from benchmark.dp_vnext.controls import load_cases, load_oracles


ROOT = Path(__file__).resolve().parents[3]


def _execute(qa_id, candidate="R1", mutation=None, index=0):
    case = load_cases(qa_id)[index]
    identity = case.get("goal_id", case.get("case_id"))
    visible = CandidateVisibleInput.from_case(qa_id, case)
    evidence = CausalCandidateRuntime(candidate, mutation=mutation).execute(visible)
    oracle = EvaluatorOracle.from_values(qa_id, identity, load_oracles(qa_id)[identity])
    return evidence, ObservationAdapter().adapt(qa_id, case, evidence, oracle)


def test_explicit_candidate_schema_does_not_project_oracle_fields():
    visible = CandidateVisibleInput.from_case("QA-02", load_cases("QA-02")[0])
    serialized = json.dumps({"parameters": dict(visible.parameters), "fields": list(visible.__dataclass_fields__)})
    assert "goal_oracle" not in serialized
    assert "required_result_facts" not in serialized
    assert "expected_graph" not in serialized
    assert "required_scopes" not in serialized


def test_every_event_has_typed_causal_metadata_and_existing_parents():
    evidence, _ = _execute("QA-02", "R3")
    seen = set()
    required = {"event_id", "event_type", "logical_timestamp_ms", "owning_component", "authority",
                "user_task_id", "execution_id", "result_id", "result_version",
                "causal_parent_event_ids", "interface_boundary_crossed", "data"}
    for event in evidence.events:
        assert set(event) == required
        assert set(event["causal_parent_event_ids"]) <= seen
        seen.add(event["event_id"])


def test_all_official_realizations_pass_candidate_specific_conformance():
    for candidate in ("R1", "R3", "R1+@"):
        for qa_id in ("QA-01", "QA-02", "QA-03", "QA-04", "QA-05", "QA-06", "QA-08", "QA-09", "QA-10", "QA-11", "QA-12"):
            case = load_cases(qa_id)[0]
            evidence = CausalCandidateRuntime(candidate).execute(CandidateVisibleInput.from_case(qa_id, case))
            assert evidence.conformance["passed"], (candidate, qa_id, evidence.conformance)


def test_lifecycle_state_machine_handles_concurrency_versions_late_results_voice_and_conflicts():
    machine = LifecycleStateMachine()
    machine.accept_task("t1"); machine.accept_task("t2")
    machine.attach_execution("t1", "e1"); machine.attach_execution("t2", "e2")
    assert machine.bind_result("t2", "e2", "r2", 1) is True  # out of order across tasks
    assert machine.bind_result("t1", "e1", "r1", 1) is True
    assert machine.correct_result("t1", "e1", "r1-v2", 2) is True
    assert machine.bind_result("t1", "e1", "late-v1", 1) is False
    machine.cancel_task("t2")
    assert machine.bind_result("t2", "e2", "late", 2) is False
    follow_up = machine.associate_follow_up("t1", "t3")
    assert follow_up.parent_task_id == "t1"
    machine.start_voice("v1", "t1"); machine.barge_in("v1")
    assert machine.voice_sessions["v1"]["status"] == "stopped" and machine.tasks["t1"].status == "completed"
    assert machine.acquire_exclusive("camera", "t1") is True
    assert machine.acquire_exclusive("camera", "t2") is False


def test_break_r3_specialist_result_correlation_lowers_qa02_and_qa03():
    specialist = next(i for i, c in enumerate(load_cases("QA-02")) if c["agent_capability_requirements"] == ["specialist-v1"])
    _, good02 = _execute("QA-02", "R3", index=specialist)
    _, bad02 = _execute("QA-02", "R3", RuntimeMutation(break_result_correlation=True), specialist)
    _, good03 = _execute("QA-03", "R3")
    _, bad03 = _execute("QA-03", "R3", RuntimeMutation(break_result_correlation=True))
    assert all(good02["required_conditions"].values()) and not all(bad02["required_conditions"].values())
    assert all(good03["required_relations"].values()) and not all(bad03["required_relations"].values())


def test_remove_r1_task_result_binding_lowers_qa02_and_qa03():
    _, bad02 = _execute("QA-02", "R1", RuntimeMutation(remove_task_binding=True))
    _, bad03 = _execute("QA-03", "R1", RuntimeMutation(remove_task_binding=True))
    assert bad02["required_conditions"]["task_result_binding"] is False
    assert bad03["required_relations"]["task_execution"] is True
    assert bad03["required_relations"]["execution_result"] is False
    assert bad03["required_relations"]["result_response"] is True
    assert bad03["required_relations"]["response_delivery"] is True
    assert bad03["required_relations"]["approval_task"] is True
    assert bad03["required_relations"]["cancel_task"] is True
    assert bad03["required_relations"]["follow_up_task"] is True
    assert bad03["required_relations"]["turn_task"] is True
    assert bad03["required_relations"]["task_execution"] is True


def test_unauthorized_scope_propagation_triggers_qa09_hard_failure():
    _, bad = _execute("QA-09", mutation=RuntimeMutation(unauthorized_scope=True))
    assert bad["confirmed_hard_gate_triggers"] == ["scope:unrelated-principal:*"]


def test_removed_trace_edge_lowers_qa10():
    _, good = _execute("QA-10")
    _, bad = _execute("QA-10", mutation=RuntimeMutation(remove_trace_edge=True))
    assert len(bad["reconstructed_causal_graph"]["edges"]) < len(good["reconstructed_causal_graph"]["edges"])


def test_delayed_feedback_worsens_qa11():
    _, good = _execute("QA-11")
    _, bad = _execute("QA-11", mutation=RuntimeMutation(feedback_delay_ms=700))
    assert bad["feedback_received_seconds"] - bad["event_available_seconds"] > good["feedback_received_seconds"] - good["event_available_seconds"]


def test_increased_playback_buffering_worsens_qa12():
    _, good = _execute("QA-12")
    _, bad = _execute("QA-12", mutation=RuntimeMutation(playback_buffer_extra_ms=200))
    assert bad["last_audible_sample_ms"] - bad["user_speech_onset_ms"] > good["last_audible_sample_ms"] - good["user_speech_onset_ms"]


def test_duplicate_recovery_action_triggers_qa08_hard_failure():
    _, bad = _execute("QA-08", mutation=RuntimeMutation(duplicate_recovery_action=True))
    assert bad["unsafe_or_incorrect_recovery"] is True
    assert bad["safe_recovery_reached"] is False


def test_semantic_dependency_leak_lowers_qa05():
    _, good = _execute("QA-05")
    _, bad = _execute("QA-05", mutation=RuntimeMutation(semantic_dependency_leak=True))
    assert good["semantic_dependency_leaks"] == [] and bad["semantic_dependency_leaks"] == ["Z3"]


def test_forced_core_change_on_agent_replacement_lowers_qa04():
    index = next(i for i, c in enumerate(load_cases("QA-04")) if c["change_request"].startswith("agent-replacement"))
    _, bad = _execute("QA-04", mutation=RuntimeMutation(force_agent_core_change=True), index=index)
    assert set(bad["actual_core_semantic_zone_changes"]) & set(bad["forbidden_core_semantic_zones"])


def test_real_extra_boundary_increases_qa01_by_common_primitive():
    good, good_observation = _execute("QA-01")
    bad, bad_observation = _execute("QA-01", mutation=RuntimeMutation(extra_boundary=True))
    added = bad.events[-1]["logical_timestamp_ms"] - good.events[-1]["logical_timestamp_ms"]
    assert added == 5.0
    assert OPERATION_COST_SOURCE["BOUNDARY_HOP"] == "H"
    assert abs((bad_observation["text_details_available_seconds"] - good_observation["text_details_available_seconds"]) - 0.005) < 1e-12


def test_frozen_v1_contract_and_campaign_evidence_match_base_commit():
    import subprocess
    protected = [ROOT / "benchmark/contracts/qa-v1", ROOT / "benchmark/scenarios/qa-v1",
                 ROOT / "benchmark/contracts/dp00-qa01-design-reference-latency-v1.json",
                 ROOT / "results/raw/dp00-vnext-qa-v1/dp00-vnext-qa-v1-campaign-v1"]
    tracked = subprocess.run(["git", "ls-tree", "-r", "--name-only", "fdcbf6e3537a66a13e77496daed5b3fa1bd94020"], cwd=ROOT, check=True, capture_output=True, text=True).stdout.splitlines()
    prefixes = [str(path.relative_to(ROOT)) for path in protected]
    for relative in tracked:
        if any(relative == prefix or relative.startswith(prefix + "/") for prefix in prefixes):
            path = ROOT / relative
            frozen = subprocess.run(["git", "show", f"fdcbf6e3537a66a13e77496daed5b3fa1bd94020:{relative}"], cwd=ROOT, check=True, capture_output=True).stdout
            assert hashlib.sha256(path.read_bytes()).digest() == hashlib.sha256(frozen).digest(), relative
