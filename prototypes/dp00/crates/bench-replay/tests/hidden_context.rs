use bench_core::{
    DecisionOwner, ModelPort, ModelProfile, ModelRequest, ModelStatus, SemanticResponsibility,
};
use bench_replay::{
    ReplayAdapter, ReplayAttempt, ReplayContext, ReplayOperation, ResponsibilityMapping,
};

#[test]
fn adapter_resolves_hidden_context_out_of_band() {
    let owner = DecisionOwner("future.intent-owner".into());
    let adapter = ReplayAdapter::new(
        ReplayContext {
            run_id: "run-1".into(),
            episode_id: "episode-1".into(),
            scenario_id: "hidden-scenario".into(),
            scenario_version: "v1".into(),
            alternative_id: "future-aut".into(),
            current_turn_fixture: "turn-1".into(),
            semantic_behavior_plan_id: "hidden-plan".into(),
            semantic_behavior_plan_version: "v1".into(),
            owner_responsibility_mapping_version: "v0".into(),
            replay_payload_registry_version: "v0".into(),
        },
        ResponsibilityMapping::default().allow(
            owner.clone(),
            [SemanticResponsibility::IntentInterpretation],
        ),
        [ReplayOperation::new(
            "turn1.intent",
            SemanticResponsibility::IntentInterpretation,
            vec![ReplayAttempt {
                output: "normalized intent".into(),
                status: ModelStatus::Completed,
            }],
        )],
    );

    let response = adapter
        .generate(ModelRequest {
            decision_owner: owner,
            semantic_responsibilities: vec![SemanticResponsibility::IntentInterpretation],
            semantic_input: "user turn".into(),
            expected_output_schema: "intent.v1".into(),
            model_profile: ModelProfile {
                id: "replay".into(),
                version: "v0".into(),
            },
        })
        .expect("responsibility resolves through hidden context");

    assert_eq!(response.composite_output, "normalized intent");
    assert_eq!(adapter.context().scenario_id, "hidden-scenario");
    assert_eq!(
        adapter.resolved_operation_key(SemanticResponsibility::IntentInterpretation),
        Some("turn1.intent")
    );
}
