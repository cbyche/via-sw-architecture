use std::path::{Path, PathBuf};

use bench_fixtures::pilot_assets::{
    ConstraintSet, RouteCommitExpectation, load_behavior_plan, load_pilot_corpus,
    load_runtime_scenario,
};
use bench_fixtures::pilot_materialization::materialize_scenario;

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
fn every_pilot_scenario_materializes_to_typed_runtime_values() {
    let corpus = load_pilot_corpus(&repository_root()).expect("Pilot-v0 corpus must validate");
    for scenario in &corpus.scenarios {
        let materialized = materialize_scenario(&corpus, scenario)
            .unwrap_or_else(|error| panic!("{} materialization: {error}", scenario.scenario_id));
        let expected_turns = scenario.stimulus.interaction.len()
            + scenario
                .stimulus
                .interaction
                .iter()
                .map(|turn| turn.deterministic_user_replies.len())
                .sum::<usize>();
        assert_eq!(materialized.turns.len(), expected_turns);
        assert_eq!(
            materialized.acoustic_eos_offsets_micros.len(),
            expected_turns
        );
        assert_eq!(
            materialized.clarification_triggered_turns.len(),
            expected_turns
        );
        assert!(!materialized.replay_operations.is_empty());
    }
}

#[test]
fn typed_fixture_materialization_rejects_unknown_payload_fields() {
    let mut corpus = load_pilot_corpus(&repository_root()).expect("Pilot-v0 corpus must validate");
    let audio = corpus
        .fixtures
        .fixtures
        .iter_mut()
        .find(|fixture| fixture.asset_id == "AUDIO-P01-v1")
        .expect("P01 audio fixture");
    audio
        .data
        .as_object_mut()
        .expect("audio data object")
        .insert("runtime_guess".into(), serde_json::json!(true));
    let scenario = corpus
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario_id == "P01")
        .expect("P01")
        .clone();
    let error = materialize_scenario(&corpus, &scenario).expect_err("unknown field must fail");
    assert!(error.to_string().contains("runtime_guess"));
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

#[test]
fn machine_readable_golden_expectation_manifest_matches_rust_validation() {
    let root = repository_root().join("benchmark/fixtures/pilot-v0/golden");
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("validation-expectations.json")).expect("manifest bytes"),
    )
    .expect("manifest JSON");
    for case in manifest["cases"].as_array().expect("cases") {
        let path = root.join(case["fixture_path"].as_str().expect("fixture path"));
        let accepted = match case["asset_kind"].as_str().expect("asset kind") {
            "RUNTIME_SCENARIO" => load_runtime_scenario(&path).is_ok(),
            "BEHAVIOR_PLAN" => load_behavior_plan(&path).is_ok(),
            other => panic!("unsupported golden asset kind {other}"),
        };
        assert_eq!(
            accepted,
            case["expected"] == "ACCEPT",
            "{} ({})",
            path.display(),
            case["reason_category"]
        );
    }
}
