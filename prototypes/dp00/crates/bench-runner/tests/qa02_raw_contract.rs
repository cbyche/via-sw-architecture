use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use bench_core::{ArchitectureEvent, ModelStatus, TaskRelation};
use bench_events::{
    CanonicalEvent, CanonicalEventKind, FailureOutcomeReason, InstrumentationMode,
    ModelSemanticValueKind, ObservableEffectType, project_actual_semantic_trace,
};
use bench_fixtures::pilot_assets::{
    BehaviorPlanRegistry, FixtureRegistry, OracleRegistry, PilotCorpus, PilotCorpusIndex,
    RuntimeScenario, load_pilot_corpus,
};
use bench_runner::pilot_runtime::{EpisodeExecution, execute_episode};
use bench_runner::{Alternative, ControlledLatencyProfile, reload_events, reload_model_calls};

fn corpus() -> PilotCorpus {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    load_pilot_corpus(&root).expect("Pilot corpus")
}

fn runtime_corpus_without_oracles() -> PilotCorpus {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let index: PilotCorpusIndex = serde_json::from_slice(
        &fs::read(root.join("benchmark/scenarios/pilot-v0/index.json")).expect("index"),
    )
    .expect("index parse");
    let scenarios = index
        .scenarios
        .iter()
        .map(|entry| {
            serde_json::from_slice(&fs::read(root.join(&entry.path)).expect("scenario"))
                .expect("scenario parse")
        })
        .collect();
    let behavior_plans: BehaviorPlanRegistry = serde_json::from_slice(
        &fs::read(root.join(&index.behavior_plan_registry_path)).expect("behavior plans"),
    )
    .expect("behavior plans parse");
    let fixtures: FixtureRegistry = serde_json::from_slice(
        &fs::read(root.join(&index.fixture_registry_path)).expect("fixtures"),
    )
    .expect("fixtures parse");
    PilotCorpus {
        index,
        scenarios,
        behavior_plans,
        fixtures,
        oracles: OracleRegistry {
            schema_version: "oracle-registry-pilot-v0".into(),
            asset_id: "NOT_LOADED".into(),
            asset_version: "NOT_LOADED".into(),
            visibility: "EVALUATOR_ONLY".into(),
            oracles: Vec::new(),
        },
    }
}

fn is_decreased(before: Option<i64>, after: Option<i64>) -> bool {
    matches!((before, after), (Some(before), Some(after)) if after < before)
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

#[test]
fn coverage_manifest_matches_every_pilot_qa02_constraint() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let oracle: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("benchmark/oracles/pilot-v0/oracles.json")).expect("oracle registry"),
    )
    .expect("oracle parse");
    let map: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("benchmark/contracts/pilot-v0-constraint-evidence-map.json"))
            .expect("coverage map"),
    )
    .expect("coverage map parse");
    let expected: HashSet<_> = oracle["oracles"]
        .as_array()
        .expect("oracles")
        .iter()
        .flat_map(|oracle| {
            oracle["constraint_manifest"]["constraints"]
                .as_array()
                .unwrap()
        })
        .map(|constraint| constraint["constraint_id"].as_str().unwrap())
        .collect();
    let expected_metadata: HashMap<_, _> = oracle["oracles"]
        .as_array()
        .expect("oracles")
        .iter()
        .flat_map(|oracle| {
            let scenario_id = oracle["scenario_id"].as_str().unwrap();
            oracle["constraint_manifest"]["constraints"]
                .as_array()
                .unwrap()
                .iter()
                .map(move |constraint| {
                    (
                        constraint["constraint_id"].as_str().unwrap(),
                        (scenario_id, constraint["dimension"].as_str().unwrap()),
                    )
                })
        })
        .collect();
    let rows = map["rows"].as_array().expect("coverage rows");
    let actual: HashSet<_> = rows
        .iter()
        .map(|row| row["constraint_id"].as_str().unwrap())
        .collect();
    assert_eq!(rows.len(), 36);
    assert_eq!(actual.len(), rows.len(), "duplicate coverage row");
    assert_eq!(actual, expected);
    for row in rows {
        assert_eq!(
            expected_metadata[row["constraint_id"].as_str().unwrap()],
            (
                row["scenario_id"].as_str().unwrap(),
                row["dimension"].as_str().unwrap()
            )
        );
        assert_eq!(row["independently_derivable"], true);
        assert_eq!(row["gap_type"], "NONE");
        assert_eq!(row["status"], "PASS");
        assert!(!row["authority"].as_str().unwrap().is_empty());
        assert!(!row["derivation_rule"].as_str().unwrap().is_empty());
        assert!(
            row["actual_fields"]
                .as_array()
                .is_some_and(|fields| !fields.is_empty())
        );
    }
}

#[test]
fn p01_transition_is_raw_directional_and_preserves_wrong_actual() {
    let corpus = runtime_corpus_without_oracles();
    let trace =
        project_actual_semantic_trace(&execute(&corpus, "P01", Alternative::C).evidence.events);
    assert_eq!(
        trace.execution_invocations[0].capability_id,
        "volume.decrease"
    );
    let effect = &trace.observable_effects[0];
    assert_eq!(effect.capability_id.as_deref(), Some("volume.decrease"));
    assert_eq!(
        (effect.before_value, effect.after_value),
        (Some(50), Some(35))
    );
    assert!(is_decreased(effect.before_value, effect.after_value));

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .join("benchmark/fixtures/pilot-v0/golden/raw-events/valid-observable-effect-event.json");
    let mut wrong: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture).expect("effect fixture")).expect("effect parse");
    wrong["event"]["UsefulOutcomeObserved"]["effect"]["before_value"] = 20.into();
    let raw: CanonicalEvent =
        serde_json::from_value(wrong).expect("wrong actual remains valid raw");
    let wrong_trace = project_actual_semantic_trace(&[raw]);
    let wrong_effect = &wrong_trace.observable_effects[0];
    assert_eq!(
        (wrong_effect.before_value, wrong_effect.after_value),
        (Some(20), Some(35))
    );
    assert!(!is_decreased(
        wrong_effect.before_value,
        wrong_effect.after_value
    ));
}

#[test]
fn p01_through_p10_actuals_reconstruct_without_loading_oracles() {
    let corpus = runtime_corpus_without_oracles();

    let p01 =
        project_actual_semantic_trace(&execute(&corpus, "P01", Alternative::C).evidence.events);
    assert_eq!(
        p01.execution_invocations[0].capability_id,
        "volume.decrease"
    );
    assert!(is_decreased(
        p01.observable_effects[0].before_value,
        p01.observable_effects[0].after_value
    ));

    let p02 =
        project_actual_semantic_trace(&execute(&corpus, "P02", Alternative::A).evidence.events);
    assert_eq!(
        p02.execution_invocations[0].capability_id,
        "downloads.inspect"
    );
    assert!(!p02.result_bindings.is_empty());
    assert_eq!(
        p02.observable_effects[0].effect_type,
        ObservableEffectType::FileInspected
    );

    let p03 =
        project_actual_semantic_trace(&execute(&corpus, "P03", Alternative::A).evidence.events);
    assert_eq!(p03.execution_invocations[0].executor_id, "NetworkAgent");
    assert_eq!(
        p03.observable_effects[0].effect_type,
        ObservableEffectType::WifiStatusObserved
    );

    let p04 =
        project_actual_semantic_trace(&execute(&corpus, "P04", Alternative::D).evidence.events);
    assert_eq!(p04.referent_bindings[0].resolved_referent_id, "doc-right");
    assert_eq!(
        p04.observable_effects[0].effect_type,
        ObservableEffectType::DocumentOpened
    );

    let p05 =
        project_actual_semantic_trace(&execute(&corpus, "P05", Alternative::A).evidence.events);
    assert_eq!(
        p05.task_associations[0].task_relation,
        TaskRelation::FollowUp
    );
    assert_eq!(p05.result_bindings[0].task_id.as_deref(), Some("T1"));
    assert_eq!(
        p05.observable_effects[0].effect_type,
        ObservableEffectType::DnsCheckObserved
    );

    let p06 =
        project_actual_semantic_trace(&execute(&corpus, "P06", Alternative::D).evidence.events);
    assert!(p06.clarification_actions[0].requested && p06.clarification_actions[0].resolved);
    assert_eq!(p06.referent_bindings[0].resolved_referent_id, "doc-right");
    assert_eq!(
        p06.observable_effects[0].effect_type,
        ObservableEffectType::DocumentOpened
    );

    let p07 =
        project_actual_semantic_trace(&execute(&corpus, "P07", Alternative::A).evidence.events);
    assert_eq!(p07.execution_invocations[0].executor_id, "ARGO");
    assert_eq!(
        p07.observable_effects[0].effect_type,
        ObservableEffectType::DownloadsOrganized
    );

    let p08_execution = execute(&corpus, "P08", Alternative::A);
    let p08 = project_actual_semantic_trace(&p08_execution.evidence.events);
    assert_eq!(p08.failure_outcomes, [FailureOutcomeReason::ModelMalformed]);
    assert!(
        p08_execution
            .evidence
            .model_calls
            .iter()
            .any(|call| call.status == ModelStatus::Malformed)
    );

    let p09_execution = execute(&corpus, "P09", Alternative::A);
    let p09 = project_actual_semantic_trace(&p09_execution.evidence.events);
    assert!(p09.committed_routes.is_empty() && p09.execution_invocations.is_empty());
    assert!(p09.observable_effects.is_empty());
    assert_eq!(p09.failure_outcomes, [FailureOutcomeReason::InvalidRoute]);
    assert!(p09_execution.evidence.model_calls.iter().any(|call| {
        call.semantic_output_reference
            .as_ref()
            .is_some_and(|value| value.value == "MailAgent")
    }));

    let p10_execution = execute(&corpus, "P10", Alternative::A);
    let p10 = project_actual_semantic_trace(&p10_execution.evidence.events);
    assert_eq!(
        p10.failure_outcomes,
        [FailureOutcomeReason::ModelNoResponse]
    );
    assert!(
        p10_execution
            .evidence
            .model_calls
            .iter()
            .any(|call| call.status == ModelStatus::NoResponse)
    );
}
