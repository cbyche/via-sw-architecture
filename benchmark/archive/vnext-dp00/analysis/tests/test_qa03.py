import copy
import json
from pathlib import Path

from dp00_analysis.qa03 import (
    calculate_ccr,
    calculate_comparative_ccr,
    classify_run,
    main,
    roles_for_path,
    validate_contracts,
)


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "benchmark/contracts/dp00-qa03-evolution-catalog-v1.json"
ROLE_MAP_PATH = ROOT / "benchmark/contracts/dp00-qa03-role-map-v1.json"
SCHEMA_PATH = ROOT / "benchmark/schemas/dp00-qa03-execution-manifest-schema-v1.json"
TEMPLATE_PATH = ROOT / "benchmark/evolution/dp00-qa03-v1/execution-manifest.template.json"


def load(path):
    return json.loads(path.read_text())


def contracts():
    return load(CATALOG_PATH), load(ROLE_MAP_PATH)


def scenario(category="E1"):
    catalog, _ = contracts()
    return next(item for item in catalog["scenarios"] if item["category"] == category)


def manifest(category="E1", alternative="A"):
    item = scenario(category)
    baseline = ["qa02:P04", "qa02:P11", "qa04:P11_FORBIDDEN"]
    return {
        "protocol_version": "dp00-qa03-flexibility-protocol-v1",
        "catalog_version": "dp00-qa03-evolution-catalog-v1",
        "role_map_version": "dp00-qa03-role-map-v1",
        "baseline_git_commit": "d4059eccd2883b3029b1e8c94fc86d092a5041f8",
        "scenario_id": item["id"],
        "alternative": alternative,
        "isolated_from_baseline": True,
        "frozen_before_implementation": True,
        "contract_artifact_hashes_match": True,
        "contract_hashes": {
            "catalog_git_blob": "d4de658fbcf096d0a9aa3269f751bdf4796ae316",
            "role_map_git_blob": "ecc77066c3540191d74c707bdc7e6bb5dc3f4e9f",
            "baseline_regression_git_blob": "ac8dabf096ca14326061f10e003fe1dfdcc3946d",
        },
        "equal_scenario_semantics": True,
        "alternative_definition_preserved": True,
        "evidence_complete": True,
        "acceptance": {"pass": True, "executed_ids": ["acceptance"], "failed_ids": []},
        "baseline_regression": {"failed_required_ids": list(baseline)},
        "observed_regression": {
            "failed_required_ids": list(baseline),
            "previously_passing_failed_ids": [],
            "worsened_required_ids": [],
        },
        "changes": [],
    }


def full_comparative_matrix(classification="CONTAINED"):
    catalog, _ = contracts()
    return [
        {
            "scenario_id": item["id"],
            "alternative": alternative,
            "classification": classification,
        }
        for item in catalog["scenarios"]
        for alternative in "ABCD"
    ]


def test_frozen_contracts_are_complete_and_cover_e1_to_e5():
    catalog, role_map = contracts()
    validate_contracts(catalog, role_map)
    assert [item["category"] for item in catalog["scenarios"]] == ["E1", "E2", "E3", "E4", "E5"]


def test_all_alternative_role_maps_resolve_representative_paths():
    _, role_map = contracts()
    representative = {
        "A": ("prototypes/dp00/crates/alternative-a/src/agent_harness.rs", "AGENT_DELEGATION_ADAPTER"),
        "B": ("prototypes/dp00/crates/alternative-b/src/argo_primary_controller.rs", "ROUTE_DECISION"),
        "C": ("prototypes/dp00/crates/alternative-c/src/fast_executor.rs", "LOCAL_CAPABILITY_EXECUTION"),
        "D": ("prototypes/dp00/crates/alternative-d/src/execution_path_selector.rs", "EXECUTION_POLICY_CONTROL_PLANE"),
    }
    for alternative, (path, expected) in representative.items():
        assert expected in roles_for_path(path, alternative, role_map)


def test_in_area_change_is_contained_and_baseline_failures_do_not_regress():
    _, role_map = contracts()
    run = manifest()
    run["changes"] = [{
        "path": "prototypes/dp00/crates/alternative-a/src/agents/document_summary.rs",
        "change_type": "ADDED",
        "change_class": "PRODUCTION_SOURCE",
    }]
    result = classify_run(run, scenario(), role_map)
    assert result["classification"] == "CONTAINED"
    assert not result["unexpected_changed_roles"]


def test_deliberate_out_of_area_change_is_not_contained():
    _, role_map = contracts()
    run = manifest()
    run["changes"] = [{
        "path": "prototypes/dp00/crates/alternative-a/src/context_engine.rs",
        "change_type": "MODIFIED",
        "change_class": "PRODUCTION_SOURCE",
    }]
    result = classify_run(run, scenario(), role_map)
    assert result["classification"] == "NOT_CONTAINED"
    assert result["unexpected_changed_roles"] == ["CONTEXT_HANDLING"]


def test_e4_valid_architecture_boundary_pressure_is_not_contained_not_invalid():
    _, role_map = contracts()
    run = manifest("E4", "A")
    run["changes"] = [{
        "path": "prototypes/dp00/crates/alternative-a/src/architecture.rs",
        "change_type": "MODIFIED",
        "change_class": "PRODUCTION_SOURCE",
    }]
    result = classify_run(run, scenario("E4"), role_map)
    assert result["classification"] == "NOT_CONTAINED"
    assert result["reasons"] == ["out_of_area_propagation"]


def test_unmapped_architecture_change_is_out_of_area():
    _, role_map = contracts()
    run = manifest()
    run["changes"] = [{"path": "src/surprise.rs", "change_type": "ADDED", "change_class": "PRODUCTION_SOURCE"}]
    result = classify_run(run, scenario(), role_map)
    assert result["classification"] == "NOT_CONTAINED"
    assert "UNMAPPED_ARCHITECTURE_CHANGE" in result["unexpected_changed_roles"]


def test_acceptance_failure_is_not_contained():
    _, role_map = contracts()
    run = manifest()
    run["acceptance"]["pass"] = False
    run["acceptance"]["failed_ids"] = ["ACC-E1-ROUTE"]
    result = classify_run(run, scenario(), role_map)
    assert result["classification"] == "NOT_CONTAINED"
    assert "acceptance_failed" in result["reasons"]


def test_new_and_worsened_regressions_are_not_contained():
    _, role_map = contracts()
    run = manifest()
    run["observed_regression"]["failed_required_ids"].append("qa02:P05")
    run["observed_regression"]["worsened_required_ids"] = ["qa04:P11_FORBIDDEN"]
    result = classify_run(run, scenario(), role_map)
    assert result["classification"] == "NOT_CONTAINED"
    assert result["details"] == ["qa02:P05", "qa04:P11_FORBIDDEN"]


def test_test_only_documentation_and_generated_outputs_are_non_scoring():
    _, role_map = contracts()
    run = manifest()
    run["changes"] = [
        {"path": "tests/new_acceptance.rs", "change_type": "ADDED", "change_class": "TEST"},
        {"path": "docs/note.md", "change_type": "ADDED", "change_class": "DOCUMENTATION"},
        {"path": "target/generated.rs", "change_type": "ADDED", "change_class": "GENERATED_OUTPUT"},
    ]
    assert classify_run(run, scenario(), role_map)["classification"] == "CONTAINED"


def test_multi_role_hunk_assertion_must_be_from_frozen_candidates():
    _, role_map = contracts()
    run = manifest()
    run["changes"] = [{
        "path": "prototypes/dp00/crates/alternative-a/src/agent_router.rs",
        "change_type": "MODIFIED",
        "change_class": "REGISTRATION",
        "asserted_roles": ["CAPABILITY_REGISTRATION"],
    }]
    assert classify_run(run, scenario(), role_map)["classification"] == "CONTAINED"
    run["changes"][0].pop("asserted_roles")
    assert classify_run(run, scenario(), role_map)["classification"] == "INCONCLUSIVE"
    run["changes"][0]["asserted_roles"] = ["VOICE_RUNTIME"]
    assert classify_run(run, scenario(), role_map)["classification"] == "INCONCLUSIVE"


def test_wrong_baseline_or_isolation_is_invalid():
    _, role_map = contracts()
    run = manifest()
    run["baseline_git_commit"] = "wrong"
    run["isolated_from_baseline"] = False
    run["alternative_definition_preserved"] = False
    result = classify_run(run, scenario(), role_map)
    assert result["classification"] == "INVALID_EXPERIMENT"


def test_silent_alternative_redefinition_is_invalid():
    _, role_map = contracts()
    run = manifest("E4", "A")
    run["alternative_definition_preserved"] = False
    result = classify_run(run, scenario("E4"), role_map)
    assert result["classification"] == "INVALID_EXPERIMENT"
    assert "alternative_definition_changed" in result["reasons"]


def test_incomplete_or_ambiguous_evidence_is_inconclusive():
    _, role_map = contracts()
    run = manifest()
    run["evidence_complete"] = False
    assert classify_run(run, scenario(), role_map)["classification"] == "INCONCLUSIVE"
    run = manifest()
    run["changes"] = [{"path": "x", "change_type": "MODIFIED", "change_class": "AMBIGUOUS"}]
    assert classify_run(run, scenario(), role_map)["classification"] == "INCONCLUSIVE"


def test_classification_and_ccr_are_deterministic():
    _, role_map = contracts()
    run = manifest()
    first = classify_run(copy.deepcopy(run), scenario(), role_map)
    second = classify_run(copy.deepcopy(run), scenario(), role_map)
    assert first == second
    results = [first, {**first, "classification": "NOT_CONTAINED"}, {**first, "classification": "INCONCLUSIVE"}]
    assert calculate_ccr(results) == {"contained": 1, "valid_evaluated": 2, "excluded": 1, "ccr_percent": 50.0}


def test_equal_denominator_when_one_cell_is_invalid():
    catalog, _ = contracts()
    cells = full_comparative_matrix()
    cells[0]["classification"] = "INVALID_EXPERIMENT"
    comparison = calculate_comparative_ccr(cells, catalog)
    assert {value["denominator"] for value in comparison["alternatives"].values()} == {4}
    assert comparison["common_denominator"] == 4


def test_invalid_or_inconclusive_cell_excludes_scenario_for_all_alternatives():
    catalog, _ = contracts()
    for classification in ["INVALID_EXPERIMENT", "INCONCLUSIVE"]:
        cells = full_comparative_matrix()
        excluded_id = cells[6]["scenario_id"]
        cells[6]["classification"] = classification
        comparison = calculate_comparative_ccr(cells, catalog)
        assert excluded_id not in comparison["s_primary"]
        exclusion = next(item for item in comparison["excluded_scenarios"] if item["scenario_id"] == excluded_id)
        assert exclusion["excluded_for_all_alternatives"] is True


def test_losing_one_single_category_scenario_prevents_complete_disposition():
    catalog, _ = contracts()
    cells = full_comparative_matrix()
    cells[0]["classification"] = "INCONCLUSIVE"
    comparison = calculate_comparative_ccr(cells, catalog)
    assert comparison["taxonomy_complete"] is False
    assert comparison["campaign_status"] == "QA-03 COMPARATIVE CAMPAIGN INCOMPLETE — TAXONOMY COVERAGE LOST"
    assert comparison["campaign_status"] != "QA-03 EVALUATION COMPLETE"


def test_normal_twenty_cell_matrix_uses_denominator_five_for_every_alternative():
    catalog, _ = contracts()
    cells = full_comparative_matrix()
    cells[1]["classification"] = "NOT_CONTAINED"
    comparison = calculate_comparative_ccr(cells, catalog)
    assert comparison["common_denominator"] == 5
    assert {value["denominator"] for value in comparison["alternatives"].values()} == {5}
    assert comparison["campaign_status"] == "QA-03 EVALUATION COMPLETE"


def test_execution_manifest_schema_and_template_versions_are_aligned():
    schema = load(SCHEMA_PATH)
    template = load(TEMPLATE_PATH)
    for field in schema["required"]:
        assert field in template
    assert template["schema_version"] == schema["properties"]["schema_version"]["const"]
    assert template["protocol_version"] == schema["properties"]["protocol_version"]["const"]


def test_cli_classifies_template_deterministically_as_incomplete(capsys):
    assert main([
        "--catalog", str(CATALOG_PATH),
        "--role-map", str(ROLE_MAP_PATH),
        "--manifest", str(TEMPLATE_PATH),
    ]) == 0
    assert json.loads(capsys.readouterr().out)["classification"] == "INCONCLUSIVE"
