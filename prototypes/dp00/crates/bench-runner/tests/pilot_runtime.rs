use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use bench_core::{
    ArchitectureEvent, ArchitectureObservation, Clock, DecisionOwner, ModelStatus,
    MonotonicTimestamp, ObservationPort, ProductCorrelation, SemanticResponsibility,
};
use bench_events::{
    CanonicalEventKind, EventEmitter, InMemoryObservationCollector, InstrumentationMode,
    LogicalModelCall, ObservationContext, validate_episode,
};
use bench_fixtures::pilot_assets::load_pilot_corpus;
use bench_runner::pilot_runtime::execute_episode;
use bench_runner::{
    Alternative, ControlledLatencyProfile, PilotRunnerConfig, RawEvidence, RunMode, RunProvenance,
    SourceState, build_episode_plan, counterbalanced_order, guard_run_mode, persist_raw_run,
    reload_events, reload_model_calls,
};

struct TickClock(AtomicU64);

impl Clock for TickClock {
    fn now(&self) -> MonotonicTimestamp {
        MonotonicTimestamp(self.0.fetch_add(10, Ordering::Relaxed))
    }
}

fn config() -> PilotRunnerConfig {
    PilotRunnerConfig {
        pilot_corpus: "pilot-v0".into(),
        alternatives: Alternative::ALL.to_vec(),
        scenario_ids: vec!["P01".into()],
        latency_profile: ControlledLatencyProfile::profile_z(),
        warmup_count: 1,
        measured_repetition_count: 2,
        order_policy: "DETERMINISTIC_CYCLIC_V1".into(),
        instrumentation_mode: InstrumentationMode::Capture,
        raw_output_root: PathBuf::from("results/raw/pilot-v0"),
        run_mode: RunMode::Development,
    }
}

fn provenance(run_id: &str) -> RunProvenance {
    RunProvenance {
        provenance_schema_version: "dp00-pilot-provenance-v1".into(),
        run_id: run_id.into(),
        official: false,
        source_git_commit: "test-sha".into(),
        working_tree_clean: false,
        pilot_corpus_id: "dp00-pilot-v0".into(),
        pilot_corpus_version: "v0.1".into(),
        alternative: Alternative::A,
        scenario_id: "P01".into(),
        scenario_version: "v0.1".into(),
        semantic_behavior_plan_id: "SBP-P01-v1".into(),
        semantic_behavior_plan_version: "v1".into(),
        latency_profile: ControlledLatencyProfile::profile_z(),
        warmup_count: 1,
        measured_repetition_count: 1,
        measurement_population: true,
        order_policy: "DETERMINISTIC_CYCLIC_V1".into(),
        order_cycle: 0,
        sequence_position: 0,
        repetition_index: 0,
        instrumentation_mode: InstrumentationMode::Capture,
        episode_elapsed_nanos: 100,
        event_count: 1,
        capture_append_cost_nanos: 5,
        model_profile: "dp00-base@v0".into(),
        prompt_profile: "pilot-v0-payload-v1".into(),
        cache_policy: "DISABLED".into(),
        rust_toolchain: "1.94.0".into(),
        rustc_version: "rustc test".into(),
        cargo_version: "cargo test".into(),
        target: "test-target".into(),
        build_profile: "test".into(),
        tokio_resolved_version: "1.53.1".into(),
        runtime_worker_policy: "tokio-current-thread-v0".into(),
        cargo_lock_identity: "test-hash".into(),
        os: std::env::consts::OS.into(),
        machine_architecture: std::env::consts::ARCH.into(),
        canonical_event_schema_version: "canonical-event-v0".into(),
        model_call_schema_version: "model-call-v0".into(),
    }
}

fn evidence() -> RawEvidence {
    let collector = InMemoryObservationCollector::with_capacity(
        ObservationContext {
            run_id: "raw-roundtrip".into(),
            episode_id: "episode-1".into(),
            scenario_id: "P01".into(),
            scenario_version: "v0.1".into(),
            alternative_id: "A".into(),
            benchmark_version: "pilot-v0".into(),
            schema_version: "canonical-event-v0".into(),
            source_git_commit: "test".into(),
        },
        TickClock(AtomicU64::new(0)),
        4,
    );
    collector.emit(ArchitectureObservation {
        event: ArchitectureEvent::ProcessingStarted,
        product_correlation: ProductCorrelation::default(),
    });
    RawEvidence {
        events: collector.canonical_snapshot(),
        model_calls: vec![LogicalModelCall {
            schema_version: "model-call-v0".into(),
            model_call_id: "call-1".into(),
            run_id: "raw-roundtrip".into(),
            episode_id: "episode-1".into(),
            scenario_id: "P01".into(),
            alternative_id: "A".into(),
            logical_sequence: 1,
            attempt: 1,
            decision_owner: DecisionOwner::from("A.IntentRefiner"),
            semantic_responsibilities: vec![SemanticResponsibility::IntentInterpretation],
            status: ModelStatus::Completed,
            route_committed_before_call: false,
            route_committed_after_call: true,
            logical_start: MonotonicTimestamp(10),
            first_output: Some(MonotonicTimestamp(15)),
            completion: MonotonicTimestamp(20),
            failure: None,
            call_class: "ORCHESTRATION".into(),
            classification_reason: "TEST".into(),
            qa04_primary_included: true,
            route_commit_event_id: Some("route-1".into()),
        }],
        fixture_events: Vec::new(),
    }
}

#[test]
fn cyclic_order_rotates_deterministically() {
    assert_eq!(
        counterbalanced_order(&Alternative::ALL, 0),
        Alternative::ALL
    );
    assert_eq!(
        counterbalanced_order(&Alternative::ALL, 1),
        [
            Alternative::B,
            Alternative::C,
            Alternative::D,
            Alternative::A
        ]
    );
    assert_eq!(
        counterbalanced_order(&Alternative::ALL, 3),
        [
            Alternative::D,
            Alternative::A,
            Alternative::B,
            Alternative::C
        ]
    );
}

#[test]
fn warmup_is_excluded_from_measured_population() {
    let plan = build_episode_plan(&config());
    assert_eq!(plan.len(), 12);
    assert_eq!(
        plan.iter()
            .filter(|item| !item.measurement_population)
            .count(),
        4
    );
    assert_eq!(
        plan.iter()
            .filter(|item| item.measurement_population)
            .count(),
        8
    );
}

#[test]
fn latency_profiles_are_common_per_dependency_and_z_is_zero() {
    let z1 = ControlledLatencyProfile::profile_z();
    let z2 = ControlledLatencyProfile::profile_z();
    assert_eq!(z1, z2);
    assert_eq!(
        (
            z1.model_delay_micros,
            z1.agent_delay_micros,
            z1.tool_delay_micros
        ),
        (0, 0, 0)
    );
    let c = ControlledLatencyProfile::profile_c();
    for _alternative in Alternative::ALL {
        assert_eq!(c.model_delay_micros, 1_000);
        assert_eq!(c.agent_delay_micros, 1_000);
        assert_eq!(c.tool_delay_micros, 1_000);
    }
    assert!(c.calibration_status.contains("NOT_PRODUCTION_OR_SCORING"));
}

#[test]
fn capture_and_minimal_use_the_same_lifecycle_with_different_sinks() {
    for (mode, expected_stored) in [
        (InstrumentationMode::Capture, 1),
        (InstrumentationMode::Minimal, 0),
    ] {
        let collector = InMemoryObservationCollector::with_mode(
            ObservationContext {
                run_id: "mode".into(),
                episode_id: "episode".into(),
                scenario_id: "P01".into(),
                scenario_version: "v0.1".into(),
                alternative_id: "A".into(),
                benchmark_version: "pilot-v0".into(),
                schema_version: "canonical-event-v0".into(),
                source_git_commit: "test".into(),
            },
            TickClock(AtomicU64::new(0)),
            1,
            mode,
        );
        collector.emit(ArchitectureObservation {
            event: ArchitectureEvent::ProcessingStarted,
            product_correlation: ProductCorrelation::default(),
        });
        assert_eq!(collector.capture_diagnostics().0, 1);
        assert_eq!(collector.canonical_snapshot().len(), expected_stored);
    }
}

#[test]
fn raw_persistence_is_create_new_and_round_trips_both_primary_streams() {
    let root = std::env::temp_dir().join(format!("dp00-raw-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale process-specific test directory");
    }
    fs::create_dir(&root).expect("test output root");
    let provenance = provenance("raw-roundtrip");
    let evidence = evidence();
    let run = persist_raw_run(&root, &provenance, &evidence).expect("first write");
    assert_eq!(
        reload_events(&run.join("canonical-events.jsonl")).expect("event reload"),
        evidence.events
    );
    assert_eq!(
        reload_model_calls(&run.join("model-calls.jsonl")).expect("call reload"),
        evidence.model_calls
    );
    assert!(persist_raw_run(&root, &provenance, &evidence).is_err());
    fs::remove_dir_all(&root).expect("clean test directory");
}

#[test]
fn official_mode_rejects_dirty_source_but_development_records_it() {
    let dirty = SourceState {
        source_git_commit: "test".into(),
        working_tree_clean: false,
    };
    assert!(guard_run_mode(RunMode::Official, &dirty).is_err());
    assert!(guard_run_mode(RunMode::Development, &dirty).is_ok());
}

#[test]
fn provenance_serialization_contains_required_reconstruction_fields() {
    let value = serde_json::to_value(provenance("provenance-fields")).expect("serialize");
    for field in [
        "source_git_commit",
        "working_tree_clean",
        "pilot_corpus_id",
        "semantic_behavior_plan_version",
        "latency_profile",
        "sequence_position",
        "instrumentation_mode",
        "rustc_version",
        "cargo_version",
        "target",
        "tokio_resolved_version",
        "runtime_worker_policy",
        "cargo_lock_identity",
        "canonical_event_schema_version",
        "model_call_schema_version",
    ] {
        assert!(value.get(field).is_some(), "missing {field}");
    }
}

#[test]
fn pilot_episode_reset_isolates_replay_fixture_and_event_state() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let corpus = load_pilot_corpus(&root).expect("Pilot corpus");
    for scenario_id in ["P05", "P06", "P08", "P10"] {
        let scenario = corpus
            .scenarios
            .iter()
            .find(|scenario| scenario.scenario_id == scenario_id)
            .expect("scenario");
        let first = execute_episode(
            &corpus,
            scenario,
            Alternative::A,
            &format!("reset-{scenario_id}-first"),
            "test",
            InstrumentationMode::Capture,
            ControlledLatencyProfile::profile_z(),
        )
        .expect("first execution");
        let second = execute_episode(
            &corpus,
            scenario,
            Alternative::A,
            &format!("reset-{scenario_id}-second"),
            "test",
            InstrumentationMode::Capture,
            ControlledLatencyProfile::profile_z(),
        )
        .expect("second execution");
        assert_eq!(first.evidence.events[0].sequence_number(), 0);
        assert_eq!(second.evidence.events[0].sequence_number(), 0);
        assert!(
            first.evidence.model_calls[0]
                .model_call_id
                .ends_with(":model:1")
        );
        assert!(
            second.evidence.model_calls[0]
                .model_call_id
                .ends_with(":model:1")
        );
        assert!(
            second
                .evidence
                .model_calls
                .iter()
                .all(|call| !call.run_id.contains("first"))
        );
    }
}

#[test]
fn p01_runtime_emits_benchmark_authoritative_acoustic_eos_before_processing() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let corpus = load_pilot_corpus(&root).expect("Pilot corpus");
    let scenario = corpus
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario_id == "P01")
        .expect("P01");
    let execution = execute_episode(
        &corpus,
        scenario,
        Alternative::A,
        "acoustic-runtime",
        "test",
        InstrumentationMode::Capture,
        ControlledLatencyProfile::profile_z(),
    )
    .expect("episode");
    let acoustic = execution
        .evidence
        .events
        .iter()
        .find(|event| matches!(event.event(), CanonicalEventKind::AcousticEos))
        .expect("acoustic EOS");
    let processing = execution
        .evidence
        .events
        .iter()
        .find(|event| {
            matches!(
                event.event(),
                CanonicalEventKind::Architecture(ArchitectureEvent::ProcessingStarted)
            )
        })
        .expect("processing start");
    assert_eq!(acoustic.emitter(), EventEmitter::InteractionFixture);
    assert!(acoustic.monotonic_timestamp() <= processing.monotonic_timestamp());
}

#[test]
fn p01_all_alternatives_complete_without_runner_scoring_or_parallelism() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let corpus = load_pilot_corpus(&root).expect("Pilot corpus");
    let scenario = corpus
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario_id == "P01")
        .expect("P01");
    for alternative in Alternative::ALL {
        let execution = execute_episode(
            &corpus,
            scenario,
            alternative,
            &format!("p01-{}", alternative.id()),
            "test",
            InstrumentationMode::Capture,
            ControlledLatencyProfile::profile_z(),
        )
        .expect("episode");
        assert_eq!(execution.architecture_error, None);
        assert!(validate_episode(&execution.evidence.events).is_empty());
    }
}

#[test]
fn minimal_sink_preserves_architecture_lifecycle_and_execute_does_not_persist() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let corpus = load_pilot_corpus(&root).expect("Pilot corpus");
    let scenario = corpus
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario_id == "P01")
        .expect("P01");
    let output = std::env::temp_dir().join(format!("dp00-no-implicit-io-{}", std::process::id()));
    assert!(!output.exists());
    let execution = execute_episode(
        &corpus,
        scenario,
        Alternative::A,
        "minimal-no-io",
        "test",
        InstrumentationMode::Minimal,
        ControlledLatencyProfile::profile_z(),
    )
    .expect("episode");
    assert_eq!(execution.architecture_error, None);
    assert!(execution.evidence.events.is_empty());
    assert!(execution.evidence.model_calls.is_empty());
    assert!(execution.evidence.fixture_events.is_empty());
    assert!(
        !output.exists(),
        "timed execution must not create raw files"
    );
}
