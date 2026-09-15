import hashlib
import inspect
import json
import math
from pathlib import Path
import re
import subprocess

import pytest

from qa_v1 import ContractRepository, EVALUATOR_TYPES, EvidenceMode, ScoreEngine, evaluate_qa


ROOT = Path(__file__).resolve().parents[3]
REGISTRY_PATH = ROOT / "benchmark/contracts/qa-v1/historical-evidence-registry-v1.json"
REGISTRY_SCHEMA_PATH = ROOT / "benchmark/schemas/historical-evidence-registry-v1.schema.json"
RESULT_SCHEMA_PATH = ROOT / "benchmark/schemas/qa-v1-result-envelope.schema.json"
PLAN_SCHEMA_PATH = ROOT / "benchmark/schemas/qa-v1-population-plan.schema.json"
USE_CASE_COVERAGE_PATH = ROOT / "benchmark/contracts/qa-v1/use-case-coverage-v1.json"
USE_CASE_COVERAGE_SCHEMA_PATH = ROOT / "benchmark/schemas/use-case-coverage-v1.schema.json"


def _load(path):
    return json.loads(path.read_text())


def _resolve_ref(ref, root_schema):
    assert ref.startswith("#/")
    value = root_schema
    for part in ref[2:].split("/"):
        value = value[part]
    return value


def _validate(instance, schema, root_schema=None):
    root_schema = root_schema or schema
    if "$ref" in schema:
        return _validate(instance, _resolve_ref(schema["$ref"], root_schema), root_schema)
    if "const" in schema and instance != schema["const"]:
        raise ValueError(f"expected const {schema['const']!r}")
    if "enum" in schema and instance not in schema["enum"]:
        raise ValueError(f"not in enum: {instance!r}")
    allowed = schema.get("type")
    if allowed:
        allowed = [allowed] if isinstance(allowed, str) else allowed
        checks = {
            "object": lambda value: isinstance(value, dict),
            "array": lambda value: isinstance(value, list),
            "string": lambda value: isinstance(value, str),
            "number": lambda value: isinstance(value, (int, float)) and not isinstance(value, bool),
            "integer": lambda value: isinstance(value, int) and not isinstance(value, bool),
            "boolean": lambda value: isinstance(value, bool),
            "null": lambda value: value is None,
        }
        if not any(checks[name](instance) for name in allowed):
            raise ValueError(f"wrong type: {instance!r}")
    if isinstance(instance, dict):
        properties = schema.get("properties", {})
        if len(instance) < schema.get("minProperties", 0):
            raise ValueError("too few properties")
        if "maxProperties" in schema and len(instance) > schema["maxProperties"]:
            raise ValueError("too many properties")
        missing = set(schema.get("required", [])) - set(instance)
        if missing:
            raise ValueError(f"missing: {sorted(missing)}")
        if schema.get("additionalProperties") is False:
            extras = set(instance) - set(properties)
            if extras:
                raise ValueError(f"extra: {sorted(extras)}")
        elif isinstance(schema.get("additionalProperties"), dict):
            for key in set(instance) - set(properties):
                _validate(instance[key], schema["additionalProperties"], root_schema)
        for key, value in instance.items():
            if key in properties:
                _validate(value, properties[key], root_schema)
    if isinstance(instance, list):
        if len(instance) < schema.get("minItems", 0):
            raise ValueError("too few items")
        if "maxItems" in schema and len(instance) > schema["maxItems"]:
            raise ValueError("too many items")
        if schema.get("uniqueItems") and len({json.dumps(value, sort_keys=True) for value in instance}) != len(instance):
            raise ValueError("duplicate items")
        for value in instance:
            if "items" in schema:
                _validate(value, schema["items"], root_schema)
    if isinstance(instance, str):
        if len(instance) < schema.get("minLength", 0):
            raise ValueError("string too short")
        if "pattern" in schema and not re.search(schema["pattern"], instance):
            raise ValueError("pattern mismatch")
    if isinstance(instance, (int, float)) and not isinstance(instance, bool):
        if "minimum" in schema and instance < schema["minimum"]:
            raise ValueError("below minimum")
        if "maximum" in schema and instance > schema["maximum"]:
            raise ValueError("above maximum")


def _sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _passing_gates(qa):
    return {gate["gate_id"]: True for gate in qa.get("hard_eligibility_gates", [])}


def test_frozen_contract_artifacts_are_byte_unchanged_from_foundation_start():
    assert _sha256(ROOT / "benchmark/contracts/qa-v1/qa-evaluation-contract-v1.json") == "23d1ede2aebed95a5fb8d444b3c31878977d0bc5579d3091338ecff50fd415d4"
    assert _sha256(ROOT / "benchmark/contracts/qa-v1/reference-environment-v1.json") == "5064aee19d148702f8630be6ec00342e862082859c69481309d7bf62105d7eb2"
    manifest = _load(ROOT / "benchmark/contracts/qa-v1/corpus-manifest-v1.json")
    assert manifest["status"] == "FROZEN_CORPUS"
    assert manifest["empirical_evaluation_status"] == "NOT_EVALUATED"


def test_historical_registry_schema_and_paths_are_valid():
    registry = _load(REGISTRY_PATH)
    schema = _load(REGISTRY_SCHEMA_PATH)
    _validate(registry, schema)
    ids = [artifact["artifact_id"] for artifact in registry["artifacts"]]
    assert len(ids) == len(set(ids)) == 20
    for artifact in registry["artifacts"]:
        assert (ROOT / artifact["path"]).exists(), artifact["artifact_id"]


def test_use_case_coverage_registry_validates_against_schema():
    _validate(_load(USE_CASE_COVERAGE_PATH), _load(USE_CASE_COVERAGE_SCHEMA_PATH))


def test_registry_classification_counts_are_deliberately_conservative():
    artifacts = _load(REGISTRY_PATH)["artifacts"]
    counts = {code: 0 for code in ("E0", "E1", "E2", "E3", "E4", "E5")}
    for artifact in artifacts:
        counts[artifact["evidence_class"]] += 1
    assert counts == {"E0": 4, "E1": 0, "E2": 4, "E3": 4, "E4": 6, "E5": 2}


def test_score_engine_reads_every_mapping_from_contract():
    contracts = ContractRepository(ROOT)
    qa01 = contracts.qa("QA-01")
    qa01["score_bands"][-1]["score"] = 2
    outcome = ScoreEngine(contracts).score("QA-01", 0.0, unit="seconds", direction="lower_is_better")
    assert outcome.score == 2
    source = inspect.getsource(ScoreEngine)
    assert "score_bands" in source
    assert not re.search(r'"DP-[0-9]', source)


@pytest.mark.parametrize("qa_id", [f"QA-{number:02d}" for number in range(1, 13)])
def test_all_score_band_boundaries(qa_id):
    contracts = ContractRepository(ROOT)
    qa = contracts.qa(qa_id)
    engine = ScoreEngine(contracts)
    gates = _passing_gates(qa)
    for band in qa["score_bands"]:
        candidates = []
        lower, upper = band["min"], band["max"]
        if lower is not None:
            candidates.append(lower if band["min_inclusive"] else math.nextafter(lower, math.inf))
        if upper is not None:
            candidates.append(upper if band["max_inclusive"] else math.nextafter(upper, -math.inf))
        if lower is not None and upper is not None and lower != upper:
            candidates.append((lower + upper) / 2)
        if upper is None:
            candidates.append(lower + 1)
        for value in candidates:
            result = engine.score(qa_id, value, unit=qa["unit"], direction=qa["direction"], gate_results=gates)
            assert result.score == band["score"], (qa_id, band, value)


def test_score_engine_enforces_unit_direction_target_and_gate():
    engine = ScoreEngine(ContractRepository(ROOT))
    with pytest.raises(ValueError, match="unit"):
        engine.score("QA-01", 3.0, unit="milliseconds")
    with pytest.raises(ValueError, match="direction"):
        engine.score("QA-01", 3.0, unit="seconds", direction="higher_is_better")
    with pytest.raises(ValueError, match="hard-gate"):
        engine.score("QA-09", 100.0, unit="percent")
    outcome = engine.score("QA-09", 100.0, unit="percent", gate_results={"QA09-SECURITY": False})
    assert outcome.score == 5
    assert outcome.target_met is True
    assert outcome.qualification_status == "DISQUALIFIED"


def test_all_twelve_evaluator_interfaces_exist_and_refuse_partial_scores():
    contracts = ContractRepository(ROOT)
    assert set(EVALUATOR_TYPES) == {f"QA-{number:02d}" for number in range(1, 13)}
    provenance = contracts.build_provenance(
        architecture_commit="a" * 40,
        dp_id="DP-00",
        alternative_id="FOUNDATION-NOT-A-CANDIDATE",
        tactic_package=None,
        run_id="foundation-interface-test",
        evidence_mode=EvidenceMode.STRUCTURAL_ANALYSIS,
    )
    result_schema = _load(RESULT_SCHEMA_PATH)
    for qa_id in EVALUATOR_TYPES:
        population_id = contracts.qa(qa_id)["population_id"]
        result = evaluate_qa(qa_id, {"population_id": population_id, "complete": False}, [], provenance, contracts)
        assert result.raw_metric is None
        assert result.score is None
        assert result.eligibility_status == "INCOMPLETE_POPULATION"
        assert result.provenance.evidence_mode is EvidenceMode.STRUCTURAL_ANALYSIS
        _validate(result.to_dict(), result_schema)


def test_all_twelve_evaluators_have_one_complete_observation_to_scalar_path():
    contracts = ContractRepository(ROOT)
    provenance = contracts.build_provenance(
        architecture_commit="c" * 40,
        dp_id="DP-00",
        alternative_id="UNIT-TEST-FIXTURE",
        tactic_package=None,
        run_id="foundation-complete-path-test",
        evidence_mode=EvidenceMode.SEMANTIC_REPLAY,
    )
    observations = {
        "QA-01": {"start_seconds": 0, "voice_facts_audible_seconds": 1, "text_details_available_seconds": 1, "useful_outcome_correct": True},
        "QA-02": {"required_conditions": {"goal": True, "binding": True}},
        "QA-03": {"required_relations": {"turn_task": True, "result_delivery": True}},
        "QA-04": {"requested_change_completed": True, "common_regressions_pass": True, "actual_semantic_ownership_changes": ["Agent adapter"], "allowed_agent_integration_ownership_areas": ["Agent adapter"], "actual_extension_seams": [], "approved_extension_seams": [], "actual_core_semantic_zone_changes": [], "forbidden_core_semantic_zones": ["Z1"]},
        "QA-05": {"requested_functionality_completed": True, "common_regressions_pass": True, "actual_changed_semantic_zones": ["Z2"], "expected_ownership_zones": ["Z2"], "actual_extension_seams": [], "approved_extension_seams": [], "semantic_dependency_leaks": []},
        "QA-06": {"required_functionality_delivered": True, "actual_core_semantic_changes": [], "device_family": "Mobile"},
        "QA-07": {"candidate_resident_bytes": 110, "minimum_mandatory_bytes": 100},
        "QA-08": {"safe_recovery_reached": True, "safe_recovery_seconds": 1, "unsafe_or_incorrect_recovery": False},
        "QA-09": {"required_scopes": ["scope-a"], "granted_or_exposed_scopes": ["scope-a"], "confirmed_hard_gate_triggers": []},
        "QA-10": {"required_chain": {"input": True, "delivery": True}, "hidden_oracle_or_fault_labels_exposed": False},
        "QA-11": {"event_available_seconds": 0, "feedback_received_seconds": 1, "useful_feedback": True, "event_class": "completion"},
        "QA-12": {"user_speech_onset_ms": 0, "last_audible_sample_ms": 100},
    }
    result_schema = _load(RESULT_SCHEMA_PATH)
    for qa_id, observation in observations.items():
        population_id = contracts.qa(qa_id)["population_id"]
        contracts.population(population_id)["planned_size"] = 1
        result = evaluate_qa(qa_id, {"population_id": population_id, "complete": True}, [observation], provenance, contracts)
        assert isinstance(result.raw_metric, float)
        assert result.score is not None
        assert result.official_scalar_metric == contracts.qa(qa_id)["official_scalar_metric"]
        _validate(result.to_dict(), result_schema)


def test_provenance_envelope_exposes_all_required_identities_and_mode():
    contracts = ContractRepository(ROOT)
    provenance = contracts.build_provenance(
        architecture_commit="b" * 40,
        dp_id="DP-16",
        alternative_id="ALT-A",
        tactic_package="read-only-@",
        run_id="run-001",
        evidence_mode=EvidenceMode.SEMANTIC_REPLAY,
    ).to_dict()
    required = {
        "qa_contract_id", "qa_contract_hash", "reference_environment_id", "reference_environment_hash",
        "corpus_manifest_id", "corpus_manifest_hash", "architecture_commit", "dp_id", "alternative_id",
        "tactic_package", "run_id", "evidence_mode",
    }
    assert set(provenance) == required
    assert provenance["evidence_mode"] == "semantic_replay"


def test_population_directories_and_plans_match_frozen_manifest():
    contracts = ContractRepository(ROOT)
    schema = _load(PLAN_SCHEMA_PATH)
    for population in contracts.manifest["populations"]:
        directory = ROOT / population["ownership_directory"]
        plan_path = directory / "population-plan.json"
        assert directory.is_dir()
        assert plan_path.is_file()
        plan = _load(plan_path)
        _validate(plan, schema)
        assert plan["population_id"] == population["population_id"]
        assert plan["qa_id"] == population["qa_id"]
        assert plan["official_target_size"] == population["planned_size"]
        assert plan["status"] == "frozen"
        assert plan["frozen_instance_count"] == population["actual_size"]
        assert plan["population_manifest"] == population["population_manifest"]
        assert plan["results_exist"] is False
        for seed in plan["legacy_seed_sources"]:
            assert (ROOT / seed["path"]).exists()


def test_no_historical_json_claims_qa_v1_without_explicit_provenance():
    command = [
        "rg", "-l", '"qa_contract_id"\\s*:\\s*"qa-evaluation-contract-v1"',
        "results", "-g", "*.json",
    ]
    completed = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)
    assert completed.returncode in (0, 1)
    claimed = [ROOT / line for line in completed.stdout.splitlines() if line]
    for path in claimed:
        document = _load(path)
        provenance = document.get("provenance", document)
        assert set(("qa_contract_hash", "reference_environment_id", "reference_environment_hash", "corpus_manifest_id", "corpus_manifest_hash", "evidence_mode")) <= set(provenance), path


def test_tracked_historical_result_content_has_not_changed():
    completed = subprocess.run(
        ["git", "diff", "--name-only", "--", "results/raw", "results/derived", "results/reports"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=True,
    )
    assert completed.stdout == ""


def test_untracked_qa01_reference_inventory_still_matches_numerical_inputs_and_outputs():
    inventory_path = ROOT / "results/reports/qa01-reference-v1/sha256-inventory.json"
    inventory = _load(inventory_path)
    for relative_path, expected in inventory["files"].items():
        assert _sha256(ROOT / relative_path) == expected, relative_path
