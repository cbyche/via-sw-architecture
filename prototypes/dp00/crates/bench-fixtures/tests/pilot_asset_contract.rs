use std::path::{Path, PathBuf};

use bench_fixtures::pilot_assets::{
    ConstraintSet, RouteCommitExpectation, load_behavior_plan, load_pilot_corpus,
    load_runtime_scenario,
};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../..")
}

#[test]
fn pilot_v0_corpus_is_complete_and_cross_referenced() {
    let corpus = load_pilot_corpus(&repository_root()).expect("Pilot-v0 corpus must validate");
    assert_eq!(corpus.scenarios.len(), 10);
    assert_eq!(corpus.behavior_plans.plans.len(), 10);
    assert_eq!(corpus.oracles.oracles.len(), 10);
}

#[test]
fn p09_keeps_route_commit_optional_and_semantic_oracle_authoritative() {
    let corpus = load_pilot_corpus(&repository_root()).expect("Pilot-v0 corpus must validate");
    let scenario = corpus
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario_id == "P09")
        .expect("P09 scenario");
    assert_eq!(
        scenario.qa_eligibility.qa04.route_commit_expectation,
        RouteCommitExpectation::Optional
    );

    let oracle = corpus
        .oracles
        .oracles
        .iter()
        .find(|oracle| oracle.scenario_id == "P09")
        .expect("P09 oracle");
    let forbidden: Vec<_> = oracle
        .constraint_manifest
        .constraints
        .iter()
        .filter(|constraint| constraint.constraint_set == ConstraintSet::Forbidden)
        .map(|constraint| constraint.constraint_id.as_str())
        .collect();
    assert!(forbidden.contains(&"P09-FORBID-MAIL-COMMIT"));
    assert!(forbidden.contains(&"P09-FORBID-MAIL-EFFECT"));
    assert_eq!(
        oracle
            .ground_truth
            .get("correct_executor")
            .and_then(serde_json::Value::as_str),
        Some("NetworkAgent")
    );
}

#[test]
fn golden_runtime_scenario_and_behavior_plan_are_accepted() {
    let root = repository_root().join("benchmark/fixtures/pilot-v0/golden");
    load_runtime_scenario(&root.join("valid-runtime-scenario.json"))
        .expect("valid scenario golden");
    load_behavior_plan(&root.join("valid-behavior-plan.json")).expect("valid behavior-plan golden");
}

#[test]
fn missing_required_field_is_rejected() {
    let path = repository_root()
        .join("benchmark/fixtures/pilot-v0/golden/invalid-missing-required-field.json");
    let error = load_runtime_scenario(&path).expect_err("missing required field must fail");
    assert!(error.to_string().contains("scenario_version"));
}

#[test]
fn unknown_field_and_invalid_enum_are_rejected() {
    let root = repository_root().join("benchmark/fixtures/pilot-v0/golden");
    let unknown = load_runtime_scenario(&root.join("invalid-unknown-field.json"))
        .expect_err("unknown field must fail");
    assert!(unknown.to_string().contains("surprise_answer"));

    let invalid_enum = load_behavior_plan(&root.join("invalid-behavior-enum.json"))
        .expect_err("invalid enum must fail");
    assert!(invalid_enum.to_string().contains("LUCKY_GUESS"));
}
