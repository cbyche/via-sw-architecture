use bench_core::{
    DecisionOwner, ModelPort, ModelProfile, ModelRequest, ModelStatus, SemanticResponsibility,
};
use bench_replay::{
    ReplayAdapter, ReplayAttempt, ReplayContext, ReplayError, ReplayOperation,
    ResponsibilityMapping,
};

fn request(owner: &str, responsibility: SemanticResponsibility) -> ModelRequest {
    ModelRequest {
        decision_owner: DecisionOwner(owner.into()),
        semantic_responsibilities: vec![responsibility],
        semantic_input: "user turn".into(),
        expected_output_schema: "decision.v1".into(),
        model_profile: ModelProfile {
            id: "replay".into(),
            version: "v0".into(),
        },
    }
}

fn context() -> ReplayContext {
    ReplayContext {
        run_id: "run-1".into(),
        episode_id: "episode-1".into(),
        scenario_id: "hidden-scenario".into(),
        scenario_version: "v1".into(),
        alternative_id: "A".into(),
        current_turn_fixture: "turn-1".into(),
        semantic_behavior_plan_id: "hidden-plan".into(),
        semantic_behavior_plan_version: "v1".into(),
        owner_responsibility_mapping_version: "dp00-base-v0".into(),
        replay_payload_registry_version: "v0".into(),
    }
}

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

#[test]
fn frozen_dp00_mapping_rejects_cross_owner_responsibilities() {
    let adapter = ReplayAdapter::new(
        context(),
        ResponsibilityMapping::dp00_base(),
        [
            ReplayOperation::new(
                "turn.intent",
                SemanticResponsibility::IntentInterpretation,
                vec![ReplayAttempt {
                    output: "intent".into(),
                    status: ModelStatus::Completed,
                }],
            ),
            ReplayOperation::new(
                "turn.referent",
                SemanticResponsibility::ReferentResolution,
                vec![ReplayAttempt {
                    output: "referent".into(),
                    status: ModelStatus::Completed,
                }],
            ),
        ],
    );

    assert_eq!(
        adapter.generate(request(
            "A.AgentRouter",
            SemanticResponsibility::IntentInterpretation
        )),
        Err(ReplayError::ResponsibilityViolation)
    );
    assert_eq!(
        adapter.generate(request(
            "D.ExecutionPathSelector",
            SemanticResponsibility::ReferentResolution
        )),
        Err(ReplayError::ResponsibilityViolation)
    );
    assert_eq!(
        adapter.generate(ModelRequest {
            semantic_responsibilities: Vec::new(),
            ..request(
                "B.ARGOPrimary",
                SemanticResponsibility::IntentInterpretation
            )
        }),
        Err(ReplayError::ResponsibilityViolation)
    );
}

#[test]
fn malformed_then_retry_consumes_two_logical_attempts_in_order() {
    let adapter = ReplayAdapter::new(
        context(),
        ResponsibilityMapping::dp00_base(),
        [ReplayOperation::new(
            "turn.route",
            SemanticResponsibility::AgentSelection,
            vec![
                ReplayAttempt {
                    output: "not valid structured output".into(),
                    status: ModelStatus::Malformed,
                },
                ReplayAttempt {
                    output: "ARGO".into(),
                    status: ModelStatus::Completed,
                },
            ],
        )],
    );

    let first = adapter
        .generate(request(
            "A.AgentRouter",
            SemanticResponsibility::AgentSelection,
        ))
        .expect("first logical generation is recorded");
    assert_eq!(first.model_status, ModelStatus::Malformed);
    let second = adapter
        .generate(request(
            "A.AgentRouter",
            SemanticResponsibility::AgentSelection,
        ))
        .expect("architecture retry generation consumes attempt two");
    assert_eq!(second.model_status, ModelStatus::Completed);
    assert_eq!(second.composite_output, "ARGO");

    let audit = adapter.audit_snapshot();
    assert_eq!(audit.len(), 2);
    assert_eq!(audit[0].attempt, 1);
    assert_eq!(audit[0].status, ModelStatus::Malformed);
    assert_eq!(audit[1].attempt, 2);
    assert_eq!(audit[1].status, ModelStatus::Completed);
    assert_eq!(audit[0].decision_owner.0, "A.AgentRouter");
    assert_eq!(audit[0].operation_key, "turn.route");
}
