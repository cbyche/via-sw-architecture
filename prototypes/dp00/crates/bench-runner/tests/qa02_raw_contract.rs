use std::fs;
use std::path::{Path, PathBuf};

use bench_core::{ArchitectureEvent, ModelStatus, TaskRelation};
use bench_events::{
    CanonicalEventKind, FailureOutcomeReason, InstrumentationMode, ModelSemanticValueKind,
    ObservableEffectType, project_actual_semantic_trace,
};
use bench_fixtures::pilot_assets::{PilotCorpus, RuntimeScenario, load_pilot_corpus};
use bench_runner::pilot_runtime::{EpisodeExecution, execute_episode};
use bench_runner::{Alternative, ControlledLatencyProfile, reload_events, reload_model_calls};

fn corpus() -> PilotCorpus {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    load_pilot_corpus(&root).expect("Pilot corpus")
}

fn scenario<'a>(corpus: &'a PilotCorpus, id: &str) -> &'a RuntimeScenario {
    corpus
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario_id == id)
        .expect("scenario")
}

fn execute(corpus: &PilotCorpus, id: &str, alternative: Alternative) -> EpisodeExecution {
    execute_episode(
        corpus,
        scenario(corpus, id),
        alternative,
        &format!("raw-contract-{id}-{}", alternative.id()),
        "test-sha",
        InstrumentationMode::Capture,
        ControlledLatencyProfile::profile_z(),
    )
    .expect("episode executes")
}

fn round_trip_evidence(
    root: &Path,
    id: &str,
    execution: &EpisodeExecution,
) -> (
    Vec<bench_events::CanonicalEvent>,
    Vec<bench_events::LogicalModelCall>,
) {
    let event_path = root.join(format!("{id}-events.jsonl"));
    let call_path = root.join(format!("{id}-model-calls.jsonl"));
    let mut event_bytes = Vec::new();
    bench_events::serialization::write_jsonl(&execution.evidence.events, &mut event_bytes)
        .expect("serialize raw events");
    let mut call_bytes = Vec::new();
    for call in &execution.evidence.model_calls {
        serde_json::to_writer(&mut call_bytes, call).expect("serialize raw model call");
        call_bytes.push(b'\n');
    }
    fs::write(&event_path, event_bytes).expect("persist development raw events");
    fs::write(&call_path, call_bytes).expect("persist development raw model calls");
    (
        reload_events(&event_path).expect("reload development raw events"),
        reload_model_calls(&call_path).expect("reload development raw model calls"),
    )
}

#[test]
fn p04_p05_p09_actuals_survive_raw_persistence_without_oracle_input() {
    let corpus = corpus();
    let root = std::env::temp_dir().join(format!("dp00-qa02-raw-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale temp evidence");
    }
    fs::create_dir(&root).expect("temp evidence root");

    let p04 = execute(&corpus, "P04", Alternative::D);
    let (p04_events, _) = round_trip_evidence(&root, "P04", &p04);
    let p04_trace = project_actual_semantic_trace(&p04_events);
    assert_eq!(
        p04_trace.referent_bindings[0].resolved_referent_id,
        "doc-right"
    );
    assert_eq!(
        p04_trace.observable_effects[0].effect_type,
        ObservableEffectType::DocumentOpened
    );
    assert_eq!(
        p04_trace.observable_effects[0].subject_id.as_deref(),
        Some("doc-right")
    );

    let p05 = execute(&corpus, "P05", Alternative::A);
    let (p05_events, _) = round_trip_evidence(&root, "P05", &p05);
    let p05_trace = project_actual_semantic_trace(&p05_events);
    assert_eq!(
        p05_trace.task_associations[0].task_relation,
        TaskRelation::FollowUp
    );
    assert_eq!(
        p05_trace.task_associations[0].task_id.as_deref(),
        Some("T1")
    );
    assert_eq!(p05_trace.result_bindings[0].task_id.as_deref(), Some("T1"));
    assert_eq!(
        p05_trace.observable_effects[0].subject_id.as_deref(),
        Some("T1")
    );

    let p09 = execute(&corpus, "P09", Alternative::A);
    let (p09_events, p09_calls) = round_trip_evidence(&root, "P09", &p09);
    let p09_trace = project_actual_semantic_trace(&p09_events);
    assert!(p09_trace.committed_routes.is_empty());
    assert_eq!(
        p09_trace.failure_outcomes,
        [FailureOutcomeReason::InvalidRoute]
    );
    let wrong_candidate = p09_calls
        .iter()
        .find_map(|call| call.semantic_output_reference.as_ref())
        .expect("actual model candidate evidence");
    assert_eq!(
        wrong_candidate.value_kind,
        ModelSemanticValueKind::ExecutorCandidate
    );
    assert_eq!(wrong_candidate.value, "MailAgent");

    fs::remove_dir_all(&root).expect("remove temp evidence");
}

#[test]
fn clarification_route_result_effect_and_failure_dimensions_are_raw_projectable() {
    let corpus = corpus();
    let p06 = execute(&corpus, "P06", Alternative::A);
    let trace = project_actual_semantic_trace(&p06.evidence.events);
    assert_eq!(trace.clarification_actions.len(), 1);
    assert!(trace.clarification_actions[0].requested);
    assert!(trace.clarification_actions[0].resolved);
    assert_eq!(
        trace.clarification_actions[0]
            .request_turn_id
            .as_ref()
            .map(|id| id.0.as_str()),
        Some("U1")
    );
    assert_eq!(
        trace.clarification_actions[0]
            .response_turn_id
            .as_ref()
            .map(|id| id.0.as_str()),
        Some("U2")
    );
    assert_eq!(trace.referent_bindings[0].resolved_referent_id, "doc-right");
    assert!(trace.committed_routes.is_empty());

    for (id, expected) in [
        ("P08", FailureOutcomeReason::ModelMalformed),
        ("P10", FailureOutcomeReason::ModelNoResponse),
    ] {
        let execution = execute(&corpus, id, Alternative::A);
        let trace = project_actual_semantic_trace(&execution.evidence.events);
        assert_eq!(trace.failure_outcomes, [expected]);
        assert!(trace.committed_routes.is_empty());
        assert!(
            execution
                .evidence
                .model_calls
                .iter()
                .any(|call| call.status != ModelStatus::Completed)
        );
    }
}

#[test]
fn p09_committed_executor_would_be_preserved_verbatim_by_route_contract() {
    let corpus = corpus();
    let p03 = execute(&corpus, "P03", Alternative::A);
    let route = project_actual_semantic_trace(&p03.evidence.events)
        .committed_routes
        .pop()
        .expect("committed route");
    assert_eq!(route.initial_executor_id.0, "NetworkAgent");
    assert_eq!(
        route.final_executor_id_if_known.expect("final executor").0,
        "NetworkAgent"
    );
}

#[test]
fn result_binding_uses_raw_product_correlation_not_behavior_plan() {
    let corpus = corpus();
    let p05 = execute(&corpus, "P05", Alternative::B);
    let bound = p05
        .evidence
        .events
        .iter()
        .find(|event| {
            matches!(
                event.event(),
                CanonicalEventKind::Architecture(ArchitectureEvent::ResultBound)
            )
        })
        .expect("result bound event");
    let correlation = bound.product_correlation();
    assert_eq!(
        correlation.task_id.as_ref().map(|id| id.0.as_str()),
        Some("T1")
    );
    assert!(correlation.execution_id.is_some());
    assert!(correlation.result_id.is_some());
}

#[test]
fn distinct_pilot_effects_and_wrong_local_claim_remain_observable() {
    let corpus = corpus();
    let p02 =
        project_actual_semantic_trace(&execute(&corpus, "P02", Alternative::A).evidence.events);
    assert_eq!(
        p02.observable_effects[0].effect_type,
        ObservableEffectType::FileInspected
    );
    assert_eq!(
        p02.observable_effects[0].subject_id.as_deref(),
        Some("downloads")
    );

    let p07 =
        project_actual_semantic_trace(&execute(&corpus, "P07", Alternative::D).evidence.events);
    assert_eq!(
        p07.committed_routes[0].initial_executor_id.0,
        "VIA_LOCAL_VOLUME"
    );
    assert_eq!(
        p07.observable_effects[0].effect_type,
        ObservableEffectType::DownloadsOrganized
    );
    assert_eq!(
        p07.observable_effects[0].executor_id.as_deref(),
        Some("VIA_LOCAL_VOLUME")
    );
}
