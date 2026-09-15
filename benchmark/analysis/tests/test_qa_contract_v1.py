import json
import re
from pathlib import Path

import pytest


ROOT = Path(__file__).resolve().parents[3]
CONTRACT_PATH = ROOT / "benchmark/contracts/qa-v1/qa-evaluation-contract-v1.json"
ENVIRONMENT_PATH = ROOT / "benchmark/contracts/qa-v1/reference-environment-v1.json"
MANIFEST_PATH = ROOT / "benchmark/contracts/qa-v1/corpus-manifest-v1.json"
SCHEMA_PATH = ROOT / "benchmark/schemas/qa-evaluation-contract-v1.schema.json"

CONTRACT = json.loads(CONTRACT_PATH.read_text())
ENVIRONMENT = json.loads(ENVIRONMENT_PATH.read_text())
MANIFEST = json.loads(MANIFEST_PATH.read_text())
SCHEMA = json.loads(SCHEMA_PATH.read_text())


def _resolve_ref(ref):
    assert ref.startswith("#/")
    value = SCHEMA
    for part in ref[2:].split("/"):
        value = value[part]
    return value


def _validate(instance, schema):
    if "$ref" in schema:
        return _validate(instance, _resolve_ref(schema["$ref"]))
    if "const" in schema and instance != schema["const"]:
        raise ValueError(f"expected const {schema['const']!r}")
    if "enum" in schema and instance not in schema["enum"]:
        raise ValueError(f"not in enum: {instance!r}")

    allowed_types = schema.get("type")
    if allowed_types:
        allowed_types = [allowed_types] if isinstance(allowed_types, str) else allowed_types
        predicates = {
            "object": lambda value: isinstance(value, dict),
            "array": lambda value: isinstance(value, list),
            "string": lambda value: isinstance(value, str),
            "number": lambda value: isinstance(value, (int, float)) and not isinstance(value, bool),
            "integer": lambda value: isinstance(value, int) and not isinstance(value, bool),
            "boolean": lambda value: isinstance(value, bool),
            "null": lambda value: value is None,
        }
        if not any(predicates[type_name](instance) for type_name in allowed_types):
            raise ValueError(f"wrong type for {instance!r}: {allowed_types}")

    if isinstance(instance, dict):
        required = schema.get("required", [])
        missing = set(required) - set(instance)
        if missing:
            raise ValueError(f"missing required properties: {sorted(missing)}")
        properties = schema.get("properties", {})
        if schema.get("additionalProperties") is False:
            extras = set(instance) - set(properties)
            if extras:
                raise ValueError(f"additional properties: {sorted(extras)}")
        for key, value in instance.items():
            if key in properties:
                _validate(value, properties[key])

    if isinstance(instance, list):
        if len(instance) < schema.get("minItems", 0):
            raise ValueError("too few items")
        if "maxItems" in schema and len(instance) > schema["maxItems"]:
            raise ValueError("too many items")
        if schema.get("uniqueItems") and len({json.dumps(item, sort_keys=True) for item in instance}) != len(instance):
            raise ValueError("duplicate array items")
        if "items" in schema:
            for value in instance:
                _validate(value, schema["items"])

    if isinstance(instance, str):
        if len(instance) < schema.get("minLength", 0):
            raise ValueError("string too short")
        if "pattern" in schema and not re.search(schema["pattern"], instance):
            raise ValueError(f"string does not match {schema['pattern']}")

    if isinstance(instance, (int, float)) and not isinstance(instance, bool):
        if "minimum" in schema and instance < schema["minimum"]:
            raise ValueError("number below minimum")


def _validate_fragment(fragment_name, instance):
    _validate(instance, SCHEMA["$defs"][fragment_name])


def test_contract_validates_against_schema():
    _validate(CONTRACT, SCHEMA)


def test_exactly_twelve_unique_active_qas_with_one_scalar_metric():
    qas = CONTRACT["quality_attributes"]
    assert len(qas) == 12
    assert {qa["qa_id"] for qa in qas} == {f"QA-{i:02d}" for i in range(1, 13)}
    assert all(isinstance(qa["official_scalar_metric"], str) and qa["official_scalar_metric"] for qa in qas)
    assert all("official_metrics" not in qa for qa in qas)


def test_every_qa_has_target_and_gapless_nonoverlapping_score_domain():
    for qa in CONTRACT["quality_attributes"]:
        assert qa["target"]["operator"] in {"<=", ">="}
        bands = sorted(qa["score_bands"], key=lambda band: band["min"])
        assert bands[0]["min"] == 0
        assert bands[0]["min_inclusive"] is True
        if qa["direction"] == "higher_is_better":
            assert bands[-1]["max"] == 100
            assert bands[-1]["max_inclusive"] is True
        else:
            assert bands[-1]["max"] is None
        assert {band["score"] for band in bands} == set(range(6))
        for left, right in zip(bands, bands[1:]):
            assert left["max"] == right["min"], qa["qa_id"]
            assert left["max_inclusive"] != right["min_inclusive"], qa["qa_id"]


def test_direction_and_score_mapping_are_consistent():
    for qa in CONTRACT["quality_attributes"]:
        bands = sorted(qa["score_bands"], key=lambda band: band["min"])
        scores = [band["score"] for band in bands]
        if qa["direction"] == "higher_is_better":
            assert scores == sorted(scores)
            assert qa["target"]["operator"] == ">="
        else:
            assert scores == sorted(scores, reverse=True)
            assert qa["target"]["operator"] == "<="


def test_every_population_is_declared_once_and_bound_to_the_same_qa():
    populations = MANIFEST["populations"]
    by_id = {population["population_id"]: population for population in populations}
    assert len(by_id) == len(populations) == 12
    for qa in CONTRACT["quality_attributes"]:
        assert qa["population_id"] in by_id
        assert by_id[qa["population_id"]]["qa_id"] == qa["qa_id"]
        assert by_id[qa["population_id"]]["status"] in {"planned", "frozen", "complete"}


def test_population_manifest_declares_size_strata_owner_and_no_fake_completion():
    assert MANIFEST["empirical_evaluation_status"] == "NOT_EVALUATED"
    assert MANIFEST["status"] == "FROZEN_CORPUS"
    for population in MANIFEST["populations"]:
        assert "planned_size" in population
        assert isinstance(population["strata"], list) and population["strata"]
        assert population["ownership_directory"].startswith("benchmark/")
        assert population["status"] == "frozen"
        assert "population_manifest_sha256" in population
        numeric_strata = [stratum["planned_size"] for stratum in population["strata"]]
        if population["planned_size"] is not None and all(size is not None for size in numeric_strata):
            assert sum(numeric_strata) == population["planned_size"]


def test_active_dp_primary_qa_references_are_valid_v1_ids():
    valid_ids = {qa["qa_id"] for qa in CONTRACT["quality_attributes"]}
    marker = re.compile(r"Primary QAs \(QA Contract v1\):\s*(.*)")
    for path in (ROOT / "docs/architecture/decision-points").glob("*.md"):
        for match in marker.finditer(path.read_text()):
            referenced = set(re.findall(r"QA-\d{2}", match.group(1)))
            assert referenced, path
            assert referenced <= valid_ids, path


def test_qa09_non_offsettable_security_gate_is_executable():
    qa09 = next(qa for qa in CONTRACT["quality_attributes"] if qa["qa_id"] == "QA-09")
    gate = next(gate for gate in qa09["hard_eligibility_gates"] if gate["gate_id"] == "QA09-SECURITY")
    assert gate["offsettable"] is False
    assert set(gate["triggers"]) == {
        "explicitly forbidden disclosure",
        "unauthorized state-changing action",
        "wrong-principal approval use",
    }


def test_result_metadata_carries_versioned_hashed_contract_identities():
    identity = lambda artifact_id: {"id": artifact_id, "version": "1.0.0", "sha256": "a" * 64}
    metadata = {
        "qa_contract": identity(CONTRACT["contract_id"]),
        "reference_environment": identity(ENVIRONMENT["environment_id"]),
        "corpus_manifest": identity(MANIFEST["manifest_id"]),
    }
    _validate_fragment("resultMetadata", metadata)


def test_dp_binding_schema_forbids_custom_targets_metrics_scores_and_populations():
    binding = {
        "dp_id": "DP-00",
        "qa_contract_id": CONTRACT["contract_id"],
        "reference_environment_id": ENVIRONMENT["environment_id"],
        "corpus_manifest_id": MANIFEST["manifest_id"],
        "primary_qa_ids": ["QA-01", "QA-02"],
    }
    _validate_fragment("decisionPointEvaluationBinding", binding)
    for forbidden in ("metric", "target", "score_bands", "population"):
        with pytest.raises(ValueError):
            _validate_fragment("decisionPointEvaluationBinding", {**binding, forbidden: "DP-specific override"})


def test_reference_environment_uses_only_medium_for_official_scoring():
    assert ENVIRONMENT["official_scoring_profile"]["profile_id"] == "QWEN3_MEDIUM_REFERENCE"
    assert ENVIRONMENT["semantic_generation"]["architecture_scoring_uses_cloud_wall_clock_latency"] is False
    assert {profile["role"] for profile in ENVIRONMENT["sensitivity_profiles"]} == {"sensitivity_analysis_only"}


def test_contract_declares_dp_override_prohibition():
    policy = CONTRACT["decision_point_policy"]
    assert policy == {
        "may_nominate_primary_qa_ids": True,
        "may_define_custom_metric": False,
        "may_define_custom_target": False,
        "may_define_custom_score_bands": False,
        "may_define_custom_population": False,
        "required_binding_schema": "#/$defs/decisionPointEvaluationBinding",
    }


def test_human_qa_documents_have_required_structure_and_match_executable_identity():
    docs_dir = ROOT / "docs/evaluation/qa-contracts/v1"
    qa_docs = sorted(path for path in docs_dir.glob("QA-*.md"))
    assert len(qa_docs) == 12
    required_sections = [
        "Purpose / Product Concern",
        "Official Scalar Metric",
        "Unit / Direction",
        "Measurement Formula",
        "Measurement Boundary",
        "Frozen Population Reference",
        "Failure / Missing-data Treatment",
        "Product Target",
        "0–5 Score Mapping",
        "Rationale",
        "What This QA Does Not Measure",
        "Typical Architecture Sensitivity",
        "Evidence / Claim Limits",
    ]
    by_id = {qa["qa_id"]: qa for qa in CONTRACT["quality_attributes"]}
    for path in qa_docs:
        text = path.read_text()
        qa_id = re.match(r"(QA-\d{2})-", path.name).group(1)
        assert all(f"## {section}" in text for section in required_sections), path
        assert by_id[qa_id]["official_scalar_metric"] in text, path
        assert f"`{by_id[qa_id]['population_id']}`" in text, path


def test_legacy_quality_attribute_paths_are_redirect_stubs():
    legacy_dir = ROOT / "docs/evaluation/quality-attributes"
    for path in legacy_dir.glob("QA-*.md"):
        text = path.read_text()
        assert text.startswith("# SUPERSEDED")
        assert "docs/evaluation/qa-contracts/v1/" in text
        assert "docs/archive/legacy-evaluation/pre-qa-contract-v1/" in text
