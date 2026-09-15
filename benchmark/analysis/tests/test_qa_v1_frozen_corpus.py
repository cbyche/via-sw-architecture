import hashlib
import json
from pathlib import Path
import subprocess

import pytest

from qa_v1 import (
    ContractRepository,
    EVALUATOR_TYPES,
    EvidenceMode,
    FrozenCorpus,
    evaluate_canonical_observations,
    evaluate_qa,
)


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "benchmark/contracts/qa-v1/corpus-manifest-v1.json"
MATERIALIZER = ROOT / "benchmark/scenarios/qa-v1/materialize_corpus.py"


def load(path):
    return json.loads(path.read_text())


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def population_by_qa():
    return {item["qa_id"]: item for item in load(MANIFEST_PATH)["populations"]}


def population_artifacts(qa_id):
    population = population_by_qa()[qa_id]
    manifest = load(ROOT / population["population_manifest"])
    return (
        manifest,
        load(ROOT / manifest["artifacts"]["families"]),
        load(ROOT / manifest["artifacts"]["instances"]),
        load(ROOT / manifest["artifacts"]["oracles"]),
    )


def test_exact_population_counts_and_family_counts_are_frozen():
    expected = {
        "QA-01": (150, 30), "QA-02": (600, 60), "QA-03": (400, 20),
        "QA-04": (60, 10), "QA-05": (60, 8), "QA-06": (60, 3),
        "QA-07": (None, 1), "QA-08": (200, 10), "QA-09": (300, 15),
        "QA-10": (200, 20), "QA-11": (200, 5), "QA-12": (200, 10),
    }
    populations = population_by_qa()
    assert set(populations) == set(expected)
    for qa_id, (count, families) in expected.items():
        assert populations[qa_id]["status"] == "frozen"
        assert populations[qa_id]["actual_size"] == count
        assert populations[qa_id]["semantic_family_count"] == families


def test_master_goal_catalog_is_unique_and_qa01_is_an_exact_fast_subset():
    families = load(ROOT / "benchmark/scenarios/qa-v1/goals/families/catalog.json")["families"]
    instances = load(ROOT / "benchmark/scenarios/qa-v1/goals/instances/catalog.json")["instances"]
    assert len(families) == 60
    assert len(instances) == 600
    assert len({item["goal_id"] for item in instances}) == 600
    assert len({item["semantic_signature"] for item in instances}) == 600
    assert all(item["goal_oracle"]["required_result_facts"] for item in instances)
    qa01 = [item for item in instances if "QA-01" in item["qa_population_memberships"]]
    assert len(qa01) == 150
    counts = {}
    for item in qa01:
        counts[item["qa02_stratum"]] = counts.get(item["qa02_stratum"], 0) + 1
    assert counts == {"fast_bounded": 50, "short_general_agent": 50, "specialist_agent": 50}


def test_population_hashes_and_oracle_identities_resolve():
    manifest_schema = load(ROOT / "benchmark/schemas/qa-v1-population-manifest.schema.json")
    oracle_schema = load(ROOT / "benchmark/schemas/qa-v1-oracle-set.schema.json")
    for qa_id, population in population_by_qa().items():
        manifest_path = ROOT / population["population_manifest"]
        assert sha256(manifest_path) == population["population_manifest_sha256"]
        manifest, families, instances, oracles = population_artifacts(qa_id)
        assert sha256(ROOT / manifest["artifacts"]["families"]) == manifest["family_manifest_hash"]
        assert sha256(ROOT / manifest["artifacts"]["instances"]) == manifest["materialized_instance_hash"]
        assert sha256(ROOT / manifest["artifacts"]["oracles"]) == manifest["oracle_hash"]
        assert families["population_id"] == population["population_id"]
        assert oracles["population_id"] == population["population_id"]
        assert set(manifest) <= set(manifest_schema["properties"])
        assert set(manifest_schema["required"]) <= set(manifest)
        assert set(oracles) == set(oracle_schema["properties"])
        assert oracles["oracles"]
        if qa_id != "QA-07":
            assert len(instances["instances"]) == population["actual_size"]
            assert len(oracles["oracles"]) == population["actual_size"]


def test_goal_fixture_references_resolve_and_have_meaningful_variations():
    fixture = load(ROOT / "benchmark/fixtures/qa-v1/reference-fixtures-v1.json")
    goals = load(ROOT / "benchmark/scenarios/qa-v1/goals/instances/catalog.json")["instances"]
    agent_profiles = load(ROOT / "benchmark/agent-stubs/qa-v1/capability-profiles-v1.json")["profiles"]
    for goal in goals:
        for reference in goal["context_fixture_references"]:
            path, pointer = reference.split("#/")
            assert (ROOT / path).is_file()
            section, key = pointer.split("/")
            assert key in fixture[section]
        assert goal["modality"] in {"voice", "text"}
        assert goal["required_voice_summary_facts"]
        assert goal["required_text_detail_fields"]
        profile_index = int(goal["agent_profile_reference"].rsplit("/", 1)[1])
        assert 0 <= profile_index < len(agent_profiles)


def test_goal_population_references_are_valid_json_pointers():
    for qa_id in ("QA-01", "QA-02"):
        _, families, instances, oracles = population_artifacts(qa_id)
        for reference in [item["catalog_ref"] for item in families["families"]] + [item["catalog_ref"] for item in instances["instances"]] + list(oracles["oracles"].values()):
            path, pointer = reference.split("#/")
            value = load(ROOT / path)
            for part in pointer.split("/"):
                value = value[int(part)] if isinstance(value, list) else value[part]
            assert value


def test_canonical_event_schema_is_dp_neutral_and_covers_qa_observability():
    schema = load(ROOT / "benchmark/schemas/qa-v1-canonical-event.schema.json")
    event_types = set(schema["properties"]["event_type"]["enum"])
    required = {
        "UserSpeechOnset", "AcousticEOS", "CanonicalTurnCommitted",
        "EvidenceVersionObserved", "TaskEventAvailable", "ResponseFactCommitted",
        "TextDetailVisible", "AudioSegmentPlaybackStarted", "AudioFactDelivered",
        "AudioPlaybackStopped", "SensitiveScopeGranted", "MemoryResidencyChanged",
        "TraceCausalLink",
    }
    assert required <= event_types
    assert "dp_id" not in schema["properties"]


def test_stratum_specific_coverage_invariants():
    _, _, qa03_instances, qa03_oracles = population_artifacts("QA-03")
    assert len({item["event_interleaving"] for item in qa03_instances["instances"]}) == 20
    assert all(
        set(oracle["required_relations"]) == {
            "turn_task", "task_execution", "execution_result", "result_response",
            "response_delivery", "approval_task", "cancel_task", "follow_up_task",
        }
        for oracle in qa03_oracles["oracles"].values()
    )

    _, _, qa05_instances, _ = population_artifacts("QA-05")
    zone_counts = {}
    for item in qa05_instances["instances"]:
        zone = item["expected_ownership_zones"][0]
        zone_counts[zone] = zone_counts.get(zone, 0) + 1
    assert list(zone_counts.values()) == [8, 8, 8, 8, 7, 7, 7, 7]

    _, _, qa06_instances, _ = population_artifacts("QA-06")
    assert {device: sum(item["device_family"] == device for item in qa06_instances["instances"]) for device in ("mobile", "tv", "robot")} == {"mobile": 20, "tv": 20, "robot": 20}

    _, _, qa09_instances, _ = population_artifacts("QA-09")
    triggers = {trigger for item in qa09_instances["instances"] for trigger in item["hard_gate_probe"]}
    assert triggers == {"explicitly forbidden disclosure", "unauthorized state-changing action", "wrong-principal approval use"}
    assert all(item["required_scopes"] for item in qa09_instances["instances"])

    _, _, qa10_instances, _ = population_artifacts("QA-10")
    assert sum(item["outcome_class"] == "success" for item in qa10_instances["instances"]) == 100
    assert sum(item["outcome_class"] == "controlled_fault_or_edge" for item in qa10_instances["instances"]) == 100

    _, _, qa11_instances, _ = population_artifacts("QA-11")
    event_counts = {}
    for item in qa11_instances["instances"]:
        event_counts[item["event_class"]] = event_counts.get(item["event_class"], 0) + 1
    assert event_counts == {"accepted_queued": 40, "meaningful_progress": 40, "approval_needed": 40, "completion": 40, "blocked_failure": 40}

    _, _, qa12_instances, _ = population_artifacts("QA-12")
    assert all(item["previous_response_audible_timeline"] for item in qa12_instances["instances"])
    assert all(item["required_stop_event"]["event_type"] == "AudioPlaybackStopped" for item in qa12_instances["instances"])


def test_qa07_is_frozen_by_timeline_identity_without_inventing_a_count():
    population = population_by_qa()["QA-07"]
    workload = load(ROOT / "benchmark/fixtures/qa-v1/resource/qa07-resource-workload-v1.json")
    assert population["population_id"] == "qa07-resource-v1"
    assert population["actual_size"] is None
    assert population["population_kind"] == "fixed_workload_timeline"
    assert population["workload_identity"] == workload["workload_id"] == "qa07-resource-workload-v1"
    assert workload["case_count"] is None
    assert [phase["phase_id"] for phase in workload["phases"]] == ["P0", "P1", "P2", "P3", "P4", "P5"]
    assert workload["phases"][4]["active_user_tasks"] == 3
    assert "provisional-state" in workload["phases"][5]["required_resident_components"]


def test_frozen_corpus_loader_verifies_every_population_and_identity():
    corpus = FrozenCorpus(ContractRepository(ROOT))
    for qa_id in EVALUATOR_TYPES:
        population = corpus.population(qa_id)
        assert population["complete"] is True
        assert population["population_id"] == population_by_qa()[qa_id]["population_id"]
        if qa_id == "QA-07":
            assert population["case_ids"] == ["P0", "P1", "P2", "P3", "P4", "P5"]
        else:
            assert len(population["case_ids"]) == population_by_qa()[qa_id]["actual_size"]


def test_all_evaluators_execute_the_frozen_self_test_fixture():
    contracts = ContractRepository(ROOT)
    fixture = load(ROOT / "benchmark/fixtures/qa-v1/evaluator-self-test-v1.json")
    provenance = contracts.build_provenance(
        architecture_commit="d" * 40,
        dp_id="DP-00",
        alternative_id="EVALUATOR-SELF-TEST-NOT-A-CANDIDATE",
        tactic_package=None,
        run_id=fixture["fixture_id"],
        evidence_mode=EvidenceMode.SEMANTIC_REPLAY,
    )
    for qa_id, observations in fixture["observations"].items():
        population_id = contracts.qa(qa_id)["population_id"]
        contracts.population(population_id)["planned_size"] = None if qa_id == "QA-07" else 1
        result = evaluate_qa(
            qa_id,
            {"population_id": population_id, "complete": True},
            observations,
            provenance,
            contracts,
        )
        assert result.raw_metric is not None
        assert result.score is not None
        assert result.provenance.evidence_mode is EvidenceMode.SEMANTIC_REPLAY


def test_complete_population_requires_exact_case_identity_coverage():
    contracts = ContractRepository(ROOT)
    provenance = contracts.build_provenance(
        architecture_commit="e" * 40,
        dp_id="DP-00",
        alternative_id="IDENTITY-TEST",
        tactic_package=None,
        run_id="identity-test",
        evidence_mode=EvidenceMode.SEMANTIC_REPLAY,
    )
    contracts.population("qa01-fast-v1")["planned_size"] = 1
    observation = {
        "observation_id": "wrong-id", "start_seconds": 0,
        "voice_facts_audible_seconds": 1, "text_details_available_seconds": 1,
        "useful_outcome_correct": True,
    }
    with pytest.raises(ValueError, match="exactly cover"):
        evaluate_qa(
            "QA-01",
            {"population_id": "qa01-fast-v1", "complete": True, "case_ids": ["expected-id"]},
            [observation],
            provenance,
            contracts,
        )


def test_canonical_observation_envelope_has_one_validated_evaluation_path():
    contracts = ContractRepository(ROOT)
    contracts.population("qa01-fast-v1")["planned_size"] = 1
    provenance = contracts.build_provenance(
        architecture_commit="f" * 40,
        dp_id="DP-00",
        alternative_id="ENVELOPE-SELF-TEST",
        tactic_package=None,
        run_id="envelope-self-test",
        evidence_mode=EvidenceMode.DESIGN_TIME_SIMULATION,
    )
    result = evaluate_canonical_observations(
        "QA-01",
        {"population_id": "qa01-fast-v1", "complete": True, "case_ids": ["case-1"]},
        [{
            "observation_id": "case-1",
            "qa_id": "QA-01",
            "population_id": "qa01-fast-v1",
            "evidence_mode": "design_time_simulation",
            "measurement": {
                "start_seconds": 0,
                "voice_facts_audible_seconds": 1,
                "text_details_available_seconds": 1,
                "useful_outcome_correct": True,
            },
        }],
        provenance,
        contracts,
    )
    assert result.raw_metric == 1.0
    assert result.provenance.evidence_mode is EvidenceMode.DESIGN_TIME_SIMULATION


def test_materialization_is_byte_identical_and_matches_checked_in_artifacts(tmp_path):
    roots = [tmp_path / "run-a", tmp_path / "run-b"]
    for output in roots:
        subprocess.run(
            [str(ROOT / ".venv/bin/python"), str(MATERIALIZER), "--source-root", str(ROOT), "--output-root", str(output)],
            cwd=ROOT,
            check=True,
        )
    hashes = []
    for output in roots:
        hashes.append({
            path.relative_to(output).as_posix(): sha256(path)
            for path in output.rglob("*")
            if path.is_file()
        })
    assert hashes[0] == hashes[1]
    for relative, digest in hashes[0].items():
        assert sha256(ROOT / relative) == digest, relative


def test_materializer_and_fixtures_are_offline_and_probability_free():
    source = MATERIALIZER.read_text()
    forbidden = ("requests.", "urllib.", "http://", "https://api.", "random.", "api_key")
    assert not any(token in source.lower() for token in forbidden)
    replay = load(ROOT / "benchmark/fixtures/qa-v1/semantic-replay-v1.json")
    assert all("probability" not in variant for variant in replay["variants"])


def test_historical_results_remain_numerically_untouched():
    completed = subprocess.run(
        ["git", "diff", "--name-only", "--", "results/raw", "results/derived", "results/reports"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=True,
    )
    assert completed.stdout == ""


def test_use_case_coverage_matrix_covers_uc01_through_uc16_and_is_bound_to_manifest():
    coverage_path = ROOT / "benchmark/contracts/qa-v1/use-case-coverage-v1.json"
    coverage = load(coverage_path)
    manifest = load(MANIFEST_PATH)
    assert set(coverage["use_cases"]) == {f"UC-{number:02d}" for number in range(1, 17)}
    assert manifest["use_case_coverage"]["path"] == coverage_path.relative_to(ROOT).as_posix()
    assert manifest["use_case_coverage"]["sha256"] == sha256(coverage_path)
    assert (ROOT / coverage["source_requirements"]).is_file()
    for entry in coverage["use_cases"].values():
        assert entry["coverage_status"] == "required_covered"
        assert entry["semantic_family_count"] >= 1
        assert entry["frozen_instance_count"] >= 1


def test_use_case_matrix_references_real_populations_and_families():
    coverage = load(ROOT / "benchmark/contracts/qa-v1/use-case-coverage-v1.json")
    populations = population_by_qa()
    valid_population_ids = {entry["population_id"] for entry in populations.values()}
    families_by_population = {}
    for qa_id, population in populations.items():
        _, families, _, _ = population_artifacts(qa_id)
        families_by_population[population["population_id"]] = {
            family["family_id"] for family in families["families"]
        }
    for use_case in coverage["use_cases"].values():
        assert set(use_case["qa_populations"]) <= valid_population_ids
        for entry in use_case["population_coverage"]:
            assert entry["population_id"] in valid_population_ids
            assert set(entry["semantic_family_ids"]) <= families_by_population[entry["population_id"]]


def test_every_frozen_family_and_case_has_valid_use_case_traceability():
    valid = {f"UC-{number:02d}" for number in range(1, 17)}
    for qa_id in EVALUATOR_TYPES:
        _, families, instances, _ = population_artifacts(qa_id)
        for family in families["families"]:
            assert set(family["use_case_tags"]) <= valid
            assert family["use_case_tags"]
            provenance = family["use_case_provenance"]
            assert provenance["source_use_case"] in family["use_case_tags"]
            assert (ROOT / provenance["historical_source_path"]).is_file()
            assert provenance["architecture_assumptions_removed"]
        cases = instances["phases"] if qa_id == "QA-07" else instances["instances"]
        for case in cases:
            assert set(case["use_case_tags"]) <= valid
            assert case["use_case_tags"]
            if qa_id != "QA-07":
                provenance = case["use_case_provenance"]
                assert provenance["source_use_case"] in case["use_case_tags"]


def test_strengthened_use_cases_have_nontrivial_family_and_instance_coverage():
    use_cases = load(ROOT / "benchmark/contracts/qa-v1/use-case-coverage-v1.json")["use_cases"]
    minimum_families = {
        "UC-02": 8, "UC-05": 8, "UC-11": 4, "UC-12": 6,
        "UC-14": 7, "UC-15": 4, "UC-16": 3,
    }
    for uc_id, minimum in minimum_families.items():
        assert use_cases[uc_id]["strengthened_coverage_gate"] is True
        assert use_cases[uc_id]["semantic_family_count"] >= minimum
        assert use_cases[uc_id]["frozen_instance_count"] >= 20


def test_qa02_covers_all_sixteen_product_use_cases_without_changing_600_denominator():
    _, _, instances, _ = population_artifacts("QA-02")
    cases = instances["instances"]
    assert len(cases) == 600
    assert {tag for case in cases for tag in case["use_case_tags"]} == {
        f"UC-{number:02d}" for number in range(1, 17)
    }


def test_uc01_oracles_are_architecture_neutral_and_do_not_require_fast_path():
    goals = load(ROOT / "benchmark/scenarios/qa-v1/goals/instances/catalog.json")["instances"]
    uc01_goals = [goal for goal in goals if "UC-01" in goal["use_case_tags"]]
    assert uc01_goals
    for goal in uc01_goals:
        semantics = goal["expected_task_semantics"]
        assert semantics["architecture_route"] == "unconstrained"
        assert semantics["candidate_may_use_any_contract_compliant_topology"] is True
        scoring_text = json.dumps({
            "task": semantics,
            "oracle": goal["goal_oracle"],
        })
        assert not any(value in scoring_text for value in ("Fast Path", "Agent Router", "R1+@", '"R1"', '"R3"'))


def test_uc05_has_multiple_concrete_action_domains():
    goals = load(ROOT / "benchmark/scenarios/qa-v1/goals/instances/catalog.json")["instances"]
    action_domains = {
        goal["expected_task_semantics"]["action_domain"]
        for goal in goals
        if "UC-05" in goal["use_case_tags"] and "action_domain" in goal["expected_task_semantics"]
    }
    assert {
        "navigation_open", "search_candidate_selection", "local_file_content",
        "communication_send", "media_system_control", "external_transaction",
        "document_transformation", "creative_ui_automation",
    } <= action_domains


def test_uc14_covers_memory_lifecycle_isolation_expiry_conflict_and_consent():
    goals = load(ROOT / "benchmark/scenarios/qa-v1/goals/instances/catalog.json")["instances"]
    memory_oracles = [
        goal["expected_task_semantics"]["memory_oracle"]
        for goal in goals
        if "memory_oracle" in goal["expected_task_semantics"]
    ]
    operations = {oracle["operation"] for oracle in memory_oracles}
    assert {
        "store", "retrieve_and_use", "modify", "delete", "expiry",
        "conflict_resolution", "provenance_consent",
    } <= operations
    assert any(oracle["wrong_principal_probe"] for oracle in memory_oracles)
    assert any(oracle["wrong_task_probe"] for oracle in memory_oracles)
    assert all(oracle["principal_and_task_isolation_required"] for oracle in memory_oracles)


def test_uc16_has_explicit_exclusive_resource_conflict_without_internal_tool_orchestration():
    goals = load(ROOT / "benchmark/scenarios/qa-v1/goals/instances/catalog.json")["instances"]
    resource_oracles = [
        goal["expected_task_semantics"]["resource_oracle"]
        for goal in goals
        if "resource_oracle" in goal["expected_task_semantics"]
    ]
    assert resource_oracles
    for oracle in resource_oracles:
        assert oracle["tasks"]["T1"] == {"resource": "mouse-keyboard", "mode": "exclusive"}
        assert oracle["tasks"]["T2"] == {"resource": "mouse-keyboard", "mode": "exclusive"}
        assert oracle["tasks"]["T3"] == {"resource": None, "mode": "read-only"}
        assert oracle["agent_internal_tool_calls_are_not_via_scheduled"] is True


def test_uc08_physical_interruption_and_semantic_correction_are_separate_qa_observations():
    coverage = load(ROOT / "benchmark/contracts/qa-v1/use-case-coverage-v1.json")["use_cases"]["UC-08"]
    by_qa = {entry["qa_id"] for entry in coverage["population_coverage"]}
    assert {"QA-02", "QA-03", "QA-12"} <= by_qa
    _, _, barge_cases, barge_oracles = population_artifacts("QA-12")
    assert all("previous_response_audible_timeline" in case for case in barge_cases["instances"])
    assert all("measurement_end" in oracle for oracle in barge_oracles["oracles"].values())
    goals = load(ROOT / "benchmark/scenarios/qa-v1/goals/instances/catalog.json")["instances"]
    correction = [
        goal for goal in goals
        if "UC-08" in goal["use_case_tags"]
        and goal["family_id"].endswith(("transcript-revision-pointer", "explicit-pointer-correction", "voice-to-text-correction"))
    ]
    assert correction
    assert all("previous_response_audible_timeline" not in goal for goal in correction)
