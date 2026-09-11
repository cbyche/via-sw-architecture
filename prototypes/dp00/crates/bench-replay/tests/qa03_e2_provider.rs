use bench_core::{
    DecisionOwner, ModelPort, ModelProfile, ModelRequest, ModelStatus, SemanticResponsibility,
};
use bench_replay::{
    ConfiguredReplayProvider, ReplayAttempt, ReplayContext, ReplayOperation, ReplayProviderId,
    ResponsibilityMapping,
};

fn provider(selection: &str) -> ConfiguredReplayProvider {
    ConfiguredReplayProvider::new(
        selection,
        ReplayContext {
            run_id: "qa03-e2".into(),
            episode_id: "cell".into(),
            scenario_id: "QA03-E2-MODEL-PROVIDER-001".into(),
            scenario_version: "v1".into(),
            alternative_id: "TEST".into(),
            current_turn_fixture: "turn".into(),
            semantic_behavior_plan_id: "plan".into(),
            semantic_behavior_plan_version: "v1".into(),
            owner_responsibility_mapping_version: "v1".into(),
            replay_payload_registry_version: "v1".into(),
        },
        ResponsibilityMapping::dp00_base(),
        [ReplayOperation::new(
            "intent",
            SemanticResponsibility::IntentInterpretation,
            vec![ReplayAttempt {
                output: "GENERAL_FILE_WORK".into(),
                status: ModelStatus::Completed,
            }],
        )],
    )
    .expect("configured provider")
}

fn request() -> ModelRequest {
    ModelRequest {
        decision_owner: DecisionOwner::from("A.IntentRefiner"),
        semantic_responsibilities: vec![SemanticResponsibility::IntentInterpretation],
        semantic_input: "request".into(),
        expected_output_schema: "intent.v1".into(),
        model_profile: ModelProfile {
            id: "qualification".into(),
            version: "v1".into(),
        },
    }
}

#[test]
fn replay_model_v2_is_selected_and_preserves_semantics_and_provenance() {
    let provider = provider("REPLAY_MODEL_V2");
    assert_eq!(provider.provider_id(), ReplayProviderId::ReplayModelV2);
    assert_eq!(provider.context().run_id, "qa03-e2");
    let response = provider.generate(request()).expect("generation");
    assert_eq!(response.composite_output, "GENERAL_FILE_WORK");
    assert_eq!(response.model_status, ModelStatus::Completed);
    assert_eq!(provider.audit_snapshot()[0].decision_owner.0, "A.IntentRefiner");
}

#[test]
fn existing_replay_provider_remains_selectable_and_unchanged() {
    let provider = provider("REPLAY_MODEL_V1");
    assert_eq!(provider.provider_id(), ReplayProviderId::ReplayModelV1);
    assert_eq!(
        provider.generate(request()).unwrap().composite_output,
        "GENERAL_FILE_WORK"
    );
}

