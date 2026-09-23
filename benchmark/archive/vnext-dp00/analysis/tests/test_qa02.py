import json
from dataclasses import replace

from conftest import REPO_ROOT
from dp00_analysis.loader import load_evidence
from dp00_analysis.models import ActualSemanticFacts
from dp00_analysis.qa02 import coverage_diagnostics, derive_qa02, evaluate_constraint


INVALIDATED_RAW = (
    REPO_ROOT
    / "results/raw/pilot-v0/official-1789087890289040000"
)


def test_t03_all_constraints_pass_and_t04_forbidden_violation(raw_run):
    episode = load_evidence(raw_run).episodes[0]
    result = derive_qa02([episode], load_evidence(raw_run).coverage_map)
    assert result["numerator"] == 1 and result["denominator"] == 1
    violated_actual = replace(episode.actual, execution_routes=({"identity":"EXECUTOR_DIRECT:ARGO","initial_executor_id":"MailAgent","final_executor_id_if_known":"MailAgent","delegation_chain":[]},))
    violated = replace(episode, actual=violated_actual)
    result = derive_qa02([violated], load_evidence(raw_run).coverage_map)
    assert result["numerator"] == 0


def test_missing_actual_is_integrity_state_not_plain_fail():
    constraint = {"constraint_id":"X", "constraint_set":"REQUIRED", "dimension":"referent", "operator":"equals", "expected":{"role":"source","object_id":"x"}}
    assert evaluate_constraint(constraint, ActualSemanticFacts())["status"] == "MISSING_ACTUAL_EVIDENCE"


def test_t06_wrong_referent_and_t07_wrong_result_binding_fail():
    wrong_referent = ActualSemanticFacts(referent_bindings=({"role":"source", "object_id":"doc-left", "turn_id":"U1"},))
    referent_constraint = {"constraint_id":"T06", "constraint_set":"REQUIRED", "dimension":"referent", "operator":"equals", "expected":{"role":"source", "object_id":"doc-right"}}
    assert evaluate_constraint(referent_constraint, wrong_referent)["status"] == "FAIL"
    wrong_result = ActualSemanticFacts(result_bindings=({"result_id":"R1", "task_id":"T2", "execution_id":"X1"},))
    result_constraint = {"constraint_id":"T07", "constraint_set":"REQUIRED", "dimension":"result_binding", "operator":"equals", "expected":"T1"}
    assert evaluate_constraint(result_constraint, wrong_result)["status"] == "FAIL"


def test_coverage_map_is_exactly_36_unique_oracle_constraints():
    coverage = json.loads((REPO_ROOT / "benchmark/contracts/pilot-v0-constraint-evidence-map.json").read_text())
    oracle = json.loads((REPO_ROOT / "benchmark/oracles/pilot-v0/oracles.json").read_text())
    rows = {row["constraint_id"] for row in coverage["rows"]}
    constraints = {c["constraint_id"] for o in oracle["oracles"] if o["scenario_id"] <= "P10" for c in o["constraint_manifest"]["constraints"]}
    assert len(coverage["rows"]) == len(rows) == 36
    assert rows == constraints
    assert all(row["independently_derivable"] and row["status"] == "PASS" for row in coverage["rows"])


def test_all_coverage_rows_are_executable_against_synthetic_actuals(raw_run):
    base = load_evidence(raw_run)
    oracle_registry = json.loads((REPO_ROOT / "benchmark/oracles/pilot-v0/oracles.json").read_text())
    scenarios = {
        o["scenario_id"]: o
        for o in oracle_registry["oracles"]
        if o["scenario_id"] <= "P10"
    }
    observed = []
    common_route = ({"identity":"EXECUTOR_DIRECT:ARGO", "initial_executor_id":"ARGO", "final_executor_id_if_known":"ARGO", "delegation_chain":[]},)
    actuals = {
        "P01": ActualSemanticFacts(execution_routes=common_route, observable_effects=({"capability_id":"volume.decrease","subject_id":"system-volume","state":"CHANGED"},), derived_predicates=frozenset({"DECREASED"})),
        "P02": ActualSemanticFacts(execution_routes=common_route, observable_effects=({"capability_id":"downloads.inspect","subject_id":"downloads","state":"OBSERVED"},), derived_predicates=frozenset({"LATEST_DOWNLOAD_NAME_EMITTED"})),
        "P03": ActualSemanticFacts(execution_routes=({"identity":"EXECUTOR_DIRECT:NetworkAgent","initial_executor_id":"NetworkAgent","final_executor_id_if_known":"NetworkAgent","delegation_chain":[]},), observable_effects=({"capability_id":"network.status","subject_id":"wifi","state":"OBSERVED"},), derived_predicates=frozenset({"NETWORK_STATUS_OBSERVED"})),
        "P04": ActualSemanticFacts(referent_bindings=({"role":"source","object_id":"doc-right","turn_id":"U1"},), execution_routes=common_route, observable_effects=({"capability_id":"document.open","subject_id":"doc-right","state":"OPEN_USABLE"},)),
        "P05": ActualSemanticFacts(task_associations=({"task_relation":"FOLLOW_UP","task_id":"T1","turn_id":"U1"},), result_bindings=({"result_id":"R1","task_id":"T1","execution_id":"X1"},)),
        "P06": ActualSemanticFacts(referent_bindings=({"role":"source","object_id":"doc-right","turn_id":"U2"},), clarifications=({"requested":True,"resolved":True,"request_turn_id":"U1","response_turn_id":"U2"},), observable_effects=({"capability_id":"document.open","subject_id":"doc-right","state":"OPEN_USABLE"},)),
        "P07": ActualSemanticFacts(execution_routes=common_route, observable_effects=({"capability_id":"downloads.organize","subject_id":"downloads","state":"ORGANIZED"},), derived_predicates=frozenset({"DOWNLOADS_ORGANIZED_BY_DOMAIN_EXECUTOR"})),
        "P08": ActualSemanticFacts(failure_outcomes=("MODEL_MALFORMED",), derived_predicates=frozenset({"SAFE_FAILURE:MALFORMED"})),
        "P09": ActualSemanticFacts(failure_outcomes=("INVALID_ROUTE",), route_candidates=({"initial_executor_id":"MailAgent"},), derived_predicates=frozenset({"MAIL_AGENT_REJECTED_OR_NOT_COMMITTED","SAFE_FAILURE:WRONG_CANDIDATE"})),
        "P10": ActualSemanticFacts(failure_outcomes=("MODEL_NO_RESPONSE",), derived_predicates=frozenset({"SAFE_FAILURE:NO_RESPONSE"})),
    }
    for scenario, oracle in scenarios.items():
        episode = replace(base.episodes[0], provenance=dict(base.episodes[0].provenance, scenario_id=scenario), expected=oracle, actual=actuals[scenario])
        observed.extend(coverage_diagnostics(episode, base.coverage_map))
    assert len(observed) == 36
    assert all(set(row) >= {"expected_source_resolvable", "actual_source_resolvable", "authority_recognized", "deterministic_derivation_available", "actual_raw_evidence_present"} for row in observed)
    assert all(row["status"] == "PASS" for row in observed), [
        row for row in observed if row["status"] != "PASS"
    ]


def test_path_aware_manifest_is_exactly_36_by_4_without_actual_values():
    manifest = json.loads(
        (
            REPO_ROOT
            / "benchmark/contracts/pilot-v0-constraint-alternative-evidence-map.json"
        ).read_text()
    )
    oracle = json.loads(
        (REPO_ROOT / "benchmark/oracles/pilot-v0/oracles.json").read_text()
    )
    constraint_ids = {
        constraint["constraint_id"]
        for item in oracle["oracles"]
        if item["scenario_id"] <= "P10"
        for constraint in item["constraint_manifest"]["constraints"]
    }
    cells = {
        (cell["constraint_id"], cell["alternative"])
        for cell in manifest["cells"]
    }
    assert manifest["actual_values_included"] is False
    assert manifest["constraint_count"] == len(constraint_ids) == 36
    assert manifest["expected_coverage_cells"] == len(cells) == 144
    assert cells == {
        (constraint_id, alternative)
        for constraint_id in constraint_ids
        for alternative in "ABCD"
    }
    assert all(cell["actual_evidence_strategies"] for cell in manifest["cells"])
    assert all(cell["derivation_rule_id"] for cell in manifest["cells"])
    assert all(cell["status"] == "READY" for cell in manifest["cells"])


def test_v02_path_aware_manifest_is_exactly_47_by_4_without_actual_values():
    manifest = json.loads(
        (
            REPO_ROOT
            / "benchmark/contracts/pilot-v0.2-constraint-alternative-evidence-map.json"
        ).read_text()
    )
    oracle = json.loads(
        (REPO_ROOT / "benchmark/oracles/pilot-v0/oracles.json").read_text()
    )
    constraint_ids = {
        constraint["constraint_id"]
        for item in oracle["oracles"]
        for constraint in item["constraint_manifest"]["constraints"]
    }
    cells = {
        (cell["constraint_id"], cell["alternative"])
        for cell in manifest["cells"]
    }
    assert manifest["actual_values_included"] is False
    assert manifest["coverage_manifest_version"] == "dp00-path-aware-evidence-v2"
    assert manifest["constraint_count"] == len(constraint_ids) == 47
    assert manifest["expected_coverage_cells"] == len(cells) == 188
    assert cells == {
        (constraint_id, alternative)
        for constraint_id in constraint_ids
        for alternative in "ABCD"
    }
    assert all(cell["actual_evidence_strategies"] for cell in manifest["cells"])
    assert all(cell["derivation_rule_id"] for cell in manifest["cells"])
    assert all(cell["status"] == "READY" for cell in manifest["cells"])


def test_invalidated_campaign_is_decidable_in_all_144_cells():
    evidence = load_evidence(INVALIDATED_RAW)
    assert evidence.validation.valid
    gate = derive_qa02(evidence.episodes, evidence.coverage_map)[
        "constraint_alternative_coverage"
    ]
    assert gate["constraint_count"] == 36
    assert gate["alternatives"] == 4
    assert gate["expected_cells"] == gate["evaluated_cells"] == 144
    assert gate["missing_cells"] == 0
    assert gate["unevaluable_cells"] == 0
    assert gate["profile_semantic_status_invariant"] is True
    assert gate["status"] == "PASS"


def test_terminal_failure_decides_positive_non_achievement_without_oracle_fallback():
    constraint = {
        "constraint_id": "P04-REQ-REFERENT",
        "constraint_set": "REQUIRED",
        "dimension": "referent",
        "operator": "equals",
        "expected": {"role": "source", "object_id": "doc-right"},
    }
    actual = ActualSemanticFacts(failure_outcomes=("INVALID_ROUTE",))
    result = evaluate_constraint(constraint, actual)
    assert result["status"] == "FAIL"
    assert result["decision_basis"] == "TERMINAL_FAILURE_NON_ACHIEVEMENT"
    assert result["actual"] == [{"terminal_failure": "INVALID_ROUTE"}]


def test_forbidden_absence_requires_closed_world_and_positive_violation_still_fails():
    constraint = {
        "constraint_id": "P07-FORBID-FAST",
        "constraint_set": "FORBIDDEN",
        "dimension": "execution_owner",
        "operator": "equals",
        "expected": "VIA_FAST",
    }
    empty = ActualSemanticFacts(failure_outcomes=("INVALID_ROUTE",))
    assert evaluate_constraint(constraint, empty)["status"] == "MISSING_ACTUAL_EVIDENCE"
    closed = evaluate_constraint(constraint, empty, closed_world_complete=True)
    assert closed["status"] == "PASS"
    assert closed["decision_basis"] == "CLOSED_WORLD_ABSENCE"
    violating = ActualSemanticFacts(
        execution_invocations=({"executor_id": "VIA_LOCAL_DOCUMENT"},)
    )
    result = evaluate_constraint(
        constraint, violating, closed_world_complete=True
    )
    assert result["status"] == "FAIL"
    assert result["decision_basis"] == "POSITIVE_FORBIDDEN_EVIDENCE"


def test_outcome_subject_is_topology_neutral_referent_evidence():
    constraint = {
        "constraint_id": "P04-REQ-REFERENT",
        "constraint_set": "REQUIRED",
        "dimension": "referent",
        "operator": "equals",
        "expected": {"role": "source", "object_id": "doc-right"},
    }
    actual = ActualSemanticFacts(
        observable_effects=(
            {
                "effect_type": "DOCUMENT_OPENED",
                "subject_id": "doc-right",
            },
        )
    )
    assert evaluate_constraint(constraint, actual)["status"] == "PASS"


def test_p09_mail_agent_commit_remains_a_correctness_failure():
    constraint = {
        "constraint_id": "P09-FORBID-MAIL-COMMIT",
        "constraint_set": "FORBIDDEN",
        "dimension": "delegated_agent",
        "operator": "equals",
        "expected": "MailAgent",
    }
    actual = ActualSemanticFacts(
        execution_routes=(
            {
                "identity": "EXECUTOR_DIRECT:MailAgent",
                "initial_executor_id": "MailAgent",
                "final_executor_id_if_known": "MailAgent",
                "delegation_chain": ["MailAgent"],
            },
        )
    )
    result = evaluate_constraint(
        constraint, actual, closed_world_complete=True
    )
    assert result["status"] == "FAIL"
    assert result["decision_basis"] == "POSITIVE_FORBIDDEN_EVIDENCE"


def test_set_like_actual_predicates_are_canonically_sorted():
    actual = ActualSemanticFacts(
        derived_predicates=frozenset(
            {"SAFE_FAILURE:WRONG_CANDIDATE", "MAIL_AGENT_REJECTED_OR_NOT_COMMITTED"}
        )
    )
    constraint = {
        "constraint_id": "P09-REQ-SAFE-FAILURE",
        "constraint_set": "REQUIRED",
        "dimension": "failure_outcome",
        "operator": "predicate",
        "expected": "SAFE_FAILURE:WRONG_CANDIDATE",
    }
    result = evaluate_constraint(constraint, actual)
    assert result["actual"] == [
        "MAIL_AGENT_REJECTED_OR_NOT_COMMITTED",
        "SAFE_FAILURE:WRONG_CANDIDATE",
    ]
