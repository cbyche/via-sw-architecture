use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use bench_core::{
    ArchitectureEvent, ArchitectureObservation, Clock, DecisionOwner, ModelStatus,
    MonotonicTimestamp, ObservationPort, ProductCorrelation, SemanticResponsibility,
};
use bench_events::{
    CanonicalEventKind, EventEmitter, InMemoryObservationCollector, InstrumentationMode,
    LogicalModelCall, ObservationContext, project_actual_semantic_trace, validate_episode,
};
use bench_fixtures::pilot_assets::load_pilot_corpus;
use bench_runner::pilot_runtime::execute_episode;
use bench_runner::{
    Alternative, CampaignConfiguration, CampaignProvenance, ControlledLatencyProfile,
    PilotRunnerConfig, RawEvidence, RunMode, RunProvenance, SourceState, begin_campaign,
    begin_campaign_profile, build_episode_plan, counterbalanced_order, file_identity,
    guard_run_mode, persist_raw_run, reload_events, reload_model_calls,
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
        calibration_identity: None,
        raw_output_root: PathBuf::from("results/raw/pilot-v0"),
        run_mode: RunMode::Development,
    }
}

fn provenance(run_id: &str) -> RunProvenance {
    RunProvenance {
        provenance_schema_version: "dp00-pilot-provenance-v3".into(),
        run_id: run_id.into(),
        campaign_id: None,
        campaign_profile_sequence_index: None,
        official: false,
        source_git_commit: "test-sha".into(),
        source_sha: "test-sha".into(),
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
        calibration_id: None,
        cycle_id: None,
        pair_id: None,
        order_slot: None,
        mode_order_slot: None,
        repetition_id: None,
        calibration_protocol_version: None,
        calibration_execution_ordinal: None,
        episode_elapsed_nanos: 100,
        event_count: 1,
        attempted_event_count: 1,
        measurement_spine_event_count: 1,
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
        canonical_event_schema_version: "canonical-event-v1".into(),
        model_call_schema_version: "model-call-v1".into(),
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
            schema_version: "canonical-event-v1".into(),
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
            schema_version: "model-call-v1".into(),
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
            semantic_output_reference: None,
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

    let expected = [
        ("R1", 50_000, 50_000, 50_000),
        ("R2", 100_000, 20_000, 20_000),
        ("R3", 20_000, 100_000, 20_000),
        ("R4", 20_000, 20_000, 100_000),
    ];
    for (profile, expected) in ControlledLatencyProfile::frozen_realistic_profiles()
        .iter()
        .zip(expected)
    {
        assert_eq!(
            (
                profile.profile_id.as_str(),
                profile.model_delay_micros,
                profile.agent_delay_micros,
                profile.tool_delay_micros,
            ),
            expected
        );
        assert_eq!(profile.version, "dp00-realistic-sensitivity-v1");
        assert!(
            profile
                .calibration_status
                .contains("NOT_PRODUCTION_MEASUREMENT")
        );
    }
}

#[test]
fn dependency_delay_budget_is_additive_and_architecture_neutral() {
    let profile = ControlledLatencyProfile::profile_r2();
    assert_eq!(
        profile.configured_delay_budget_micros(2, 3, 4),
        2 * 100_000 + 3 * 20_000 + 4 * 20_000
    );
    for _alternative in Alternative::ALL {
        assert_eq!(
            profile.configured_delay_budget_micros(2, 3, 4),
            340_000,
            "cost depends only on semantic event counts"
        );
    }
}

#[test]
fn machine_readable_profile_contract_matches_runner_constants() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let contract: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("benchmark/contracts/dp00-realistic-execution-profiles-v1.json"))
            .expect("profile contract"),
    )
    .expect("valid profile contract JSON");
    assert_eq!(contract["contract_version"], "v1");
    assert_eq!(contract["status"], "PROSPECTIVELY_FROZEN");
    let configured = contract["profiles"].as_array().expect("profile array");
    for (record, profile) in configured
        .iter()
        .zip(ControlledLatencyProfile::frozen_realistic_profiles())
    {
        assert_eq!(record["profile_id"], profile.profile_id);
        assert_eq!(record["version"], profile.version);
        assert_eq!(record["model_delay_micros"], profile.model_delay_micros);
        assert_eq!(record["agent_delay_micros"], profile.agent_delay_micros);
        assert_eq!(record["tool_delay_micros"], profile.tool_delay_micros);
    }
}

#[test]
fn realistic_delay_changes_timing_but_not_routing_or_correctness_semantics() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let corpus = load_pilot_corpus(&root).expect("Pilot corpus");
    for scenario_id in ["P01", "P12"] {
        let scenario = corpus
            .scenarios
            .iter()
            .find(|scenario| scenario.scenario_id == scenario_id)
            .expect("scenario");
        for alternative in Alternative::ALL {
            let run_id = format!("profile-semantics-{scenario_id}-{}", alternative.id());
            let zero = execute_episode(
                &corpus,
                scenario,
                alternative,
                &run_id,
                "test",
                InstrumentationMode::Capture,
                ControlledLatencyProfile::profile_z(),
            )
            .expect("Profile Z episode");
            let realistic = execute_episode(
                &corpus,
                scenario,
                alternative,
                &run_id,
                "test",
                InstrumentationMode::Capture,
                ControlledLatencyProfile::profile_r1(),
            )
            .expect("Profile R1 episode");

            assert_eq!(zero.architecture_error, realistic.architecture_error);
            assert_eq!(
                project_actual_semantic_trace(&zero.evidence.events),
                project_actual_semantic_trace(&realistic.evidence.events),
                "{} {scenario_id} semantic trace",
                alternative.id()
            );
            assert_eq!(
                zero.evidence.model_calls.len(),
                realistic.evidence.model_calls.len()
            );

            let agent_starts = realistic
                .evidence
                .fixture_events
                .iter()
                .filter(|event| event.fixture_kind == "AGENT" && event.action == "START_ACCEPT")
                .count() as u64;
            let tool_starts = realistic
                .evidence
                .fixture_events
                .iter()
                .filter(|event| event.fixture_kind == "TOOL" && event.action == "START_ACCEPT")
                .count() as u64;
            let expected_budget = ControlledLatencyProfile::profile_r1()
                .configured_delay_budget_micros(
                    realistic.evidence.model_calls.len() as u64,
                    agent_starts,
                    tool_starts,
                );
            assert!(
                realistic.episode_elapsed_nanos >= expected_budget * 1_000,
                "{} {scenario_id} did not incur every configured semantic charge",
                alternative.id()
            );

            if scenario_id == "P12" {
                let commits = realistic
                    .evidence
                    .events
                    .iter()
                    .enumerate()
                    .filter_map(|(position, event)| match event.event() {
                        CanonicalEventKind::Architecture(ArchitectureEvent::RouteCommitted {
                            subgoal_id: Some(subgoal_id),
                            ..
                        }) => Some((position, subgoal_id.0.as_str())),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                let first_execution = realistic
                    .evidence
                    .events
                    .iter()
                    .position(|event| {
                        matches!(event.event(), CanonicalEventKind::ExecutionStarted { .. })
                    })
                    .expect("P12 execution");
                assert_eq!(
                    commits.iter().map(|item| item.1).collect::<Vec<_>>(),
                    ["S1", "S2"]
                );
                assert!(first_execution > commits[1].0);
                assert_eq!(agent_starts + tool_starts, 2);
            }
        }
    }
}

#[test]
fn capture_and_minimal_use_the_same_lifecycle_with_different_sinks() {
    for mode in [InstrumentationMode::Capture, InstrumentationMode::Minimal] {
        let collector = InMemoryObservationCollector::with_mode(
            ObservationContext {
                run_id: "mode".into(),
                episode_id: "episode".into(),
                scenario_id: "P01".into(),
                scenario_version: "v0.1".into(),
                alternative_id: "A".into(),
                benchmark_version: "pilot-v0".into(),
                schema_version: "canonical-event-v1".into(),
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
        assert_eq!(collector.capture_diagnostics().1, 1);
        assert_eq!(collector.canonical_snapshot().len(), 1);
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
fn official_campaign_attests_once_and_persists_ordered_z_then_c_without_source_mutation() {
    let root = std::env::temp_dir().join(format!("dp00-campaign-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale campaign directory");
    }
    let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let architecture_sources = [
        "prototypes/dp00/crates/alternative-a/src/architecture.rs",
        "prototypes/dp00/crates/alternative-b/src/architecture.rs",
        "prototypes/dp00/crates/alternative-c/src/architecture.rs",
        "prototypes/dp00/crates/alternative-d/src/architecture.rs",
    ];
    let before = architecture_sources
        .iter()
        .map(|path| file_identity(&repository_root.join(path)).expect("source identity"))
        .collect::<Vec<_>>();
    let profiles = vec![
        ControlledLatencyProfile::profile_z(),
        ControlledLatencyProfile::profile_c(),
    ];
    let campaign = CampaignProvenance {
        provenance_schema_version: "dp00-pilot-campaign-provenance-v1".into(),
        campaign_id: "campaign-z-c".into(),
        source_git_commit: "same-test-sha".into(),
        initial_working_tree_clean: true,
        pilot_corpus_id: "dp00-pilot-v0".into(),
        pilot_corpus_version: "v0.1".into(),
        profile_sequence: profiles.clone(),
        campaign_configuration: CampaignConfiguration {
            alternatives: Alternative::ALL.to_vec(),
            scenario_ids: vec!["P04".into(), "P05".into(), "P09".into()],
            warmup_count: 0,
            measured_repetition_count: 1,
            order_policy: "DETERMINISTIC_CYCLIC_V1".into(),
            instrumentation_mode: InstrumentationMode::Capture,
        },
        runner_version: "test".into(),
    };
    guard_run_mode(
        RunMode::Official,
        &SourceState {
            source_git_commit: campaign.source_git_commit.clone(),
            working_tree_clean: campaign.initial_working_tree_clean,
        },
    )
    .expect("clean source accepted once");
    let campaign_directory = begin_campaign(&root, &campaign).expect("campaign begins");
    assert!(
        begin_campaign(&root, &campaign).is_err(),
        "campaign overwrite rejected"
    );

    for (index, profile) in profiles.iter().enumerate() {
        let profile_directory =
            begin_campaign_profile(&campaign_directory, profile).expect("profile begins");
        let mut run = provenance(&format!("{}-run", profile.profile_id));
        run.official = true;
        run.working_tree_clean = true;
        run.source_git_commit = campaign.source_git_commit.clone();
        run.pilot_corpus_id = campaign.pilot_corpus_id.clone();
        run.pilot_corpus_version = campaign.pilot_corpus_version.clone();
        run.campaign_id = Some(campaign.campaign_id.clone());
        run.campaign_profile_sequence_index = Some(index as u32);
        run.latency_profile = profile.clone();
        persist_raw_run(&profile_directory, &run, &evidence()).expect("profile raw persists");
    }

    let stored: CampaignProvenance = serde_json::from_slice(
        &fs::read(campaign_directory.join("campaign-provenance.json"))
            .expect("campaign provenance"),
    )
    .expect("campaign provenance parses");
    assert_eq!(stored.profile_sequence, profiles);
    assert_eq!(stored.profile_sequence[0].profile_id, "Z");
    assert_eq!(stored.profile_sequence[1].profile_id, "C");
    for profile_id in ["z", "c"] {
        let run: RunProvenance = serde_json::from_slice(
            &fs::read(
                campaign_directory
                    .join(format!("profile-{profile_id}"))
                    .join(format!("{}-run", profile_id.to_ascii_uppercase()))
                    .join("provenance.json"),
            )
            .expect("run provenance"),
        )
        .expect("run provenance parses");
        assert_eq!(run.source_git_commit, "same-test-sha");
        assert_eq!(run.pilot_corpus_version, "v0.1");
        assert_eq!(
            run.latency_profile.profile_id.to_ascii_lowercase(),
            profile_id
        );
    }
    let after = architecture_sources
        .iter()
        .map(|path| file_identity(&repository_root.join(path)).expect("source identity"))
        .collect::<Vec<_>>();
    assert_eq!(
        before, after,
        "campaign support does not mutate A/B/C/D sources"
    );
    fs::remove_dir_all(&root).expect("remove campaign directory");
}

#[test]
fn provenance_serialization_contains_required_reconstruction_fields() {
    let mut original = provenance("provenance-fields");
    original.provenance_schema_version = "dp00-pilot-provenance-v4".into();
    original.calibration_id = Some("calibration-1".into());
    original.cycle_id = Some("cycle-1".into());
    original.pair_id = Some("calibration-1:cycle-1:rotation-1:P01:A".into());
    original.order_slot = Some(0);
    original.mode_order_slot = Some(1);
    original.repetition_id = Some("rotation-1".into());
    original.calibration_protocol_version = Some("dp00-calibration-protocol-v1".into());
    original.calibration_execution_ordinal = Some(1);
    let value = serde_json::to_value(&original).expect("serialize");
    for field in [
        "source_git_commit",
        "source_sha",
        "working_tree_clean",
        "pilot_corpus_id",
        "semantic_behavior_plan_version",
        "latency_profile",
        "sequence_position",
        "instrumentation_mode",
        "calibration_id",
        "cycle_id",
        "pair_id",
        "order_slot",
        "mode_order_slot",
        "repetition_id",
        "calibration_protocol_version",
        "calibration_execution_ordinal",
        "attempted_event_count",
        "measurement_spine_event_count",
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
    let round_trip: RunProvenance = serde_json::from_value(value).expect("round trip");
    assert_eq!(round_trip.calibration_id, original.calibration_id);
    assert_eq!(round_trip.cycle_id, original.cycle_id);
    assert_eq!(round_trip.pair_id, original.pair_id);
    assert_eq!(round_trip.order_slot, original.order_slot);
    assert_eq!(round_trip.mode_order_slot, original.mode_order_slot);
    assert_eq!(round_trip.repetition_id, original.repetition_id);
    assert_eq!(
        round_trip.calibration_protocol_version,
        original.calibration_protocol_version
    );
    assert_eq!(
        round_trip.calibration_execution_ordinal,
        original.calibration_execution_ordinal
    );
    assert_eq!(round_trip.source_sha, original.source_git_commit);
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
fn minimal_sink_persists_measurement_spine_in_memory_without_implicit_io() {
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
    assert!(
        execution
            .evidence
            .events
            .iter()
            .any(|event| matches!(event.event(), CanonicalEventKind::AcousticEos))
    );
    assert!(execution.evidence.events.iter().any(|event| matches!(
        event.event(),
        CanonicalEventKind::UsefulOutcomeObserved { .. }
    )));
    assert!(execution.evidence.events.iter().any(|event| matches!(
        event.event(),
        CanonicalEventKind::Architecture(ArchitectureEvent::RouteCommitted { .. })
    )));
    assert!(!execution.evidence.model_calls.is_empty());
    assert!(execution.evidence.fixture_events.is_empty());
    assert_eq!(
        execution.measurement_spine_event_count,
        execution.evidence.events.len() as u64
    );
    assert!(execution.attempted_event_count > execution.measurement_spine_event_count);
    assert!(
        !output.exists(),
        "timed execution must not create raw files"
    );
}

#[test]
fn capture_and_minimal_spines_preserve_ftol_and_compound_semantics() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let corpus = load_pilot_corpus(&root).expect("Pilot corpus");
    for scenario_id in ["P01", "P12"] {
        let scenario = corpus
            .scenarios
            .iter()
            .find(|scenario| scenario.scenario_id == scenario_id)
            .expect("scenario");
        let executions = [InstrumentationMode::Capture, InstrumentationMode::Minimal].map(|mode| {
            execute_episode(
                &corpus,
                scenario,
                Alternative::A,
                &format!("paired-{scenario_id}"),
                "test",
                mode,
                ControlledLatencyProfile::profile_z(),
            )
            .expect("paired episode")
        });
        assert_eq!(
            project_actual_semantic_trace(&executions[0].evidence.events),
            project_actual_semantic_trace(&executions[1].evidence.events)
        );
        assert_eq!(
            executions[0].evidence.model_calls.len(),
            executions[1].evidence.model_calls.len()
        );
        if scenario_id == "P01" {
            for execution in &executions {
                let eos = execution
                    .evidence
                    .events
                    .iter()
                    .find(|event| matches!(event.event(), CanonicalEventKind::AcousticEos))
                    .expect("authoritative EOS")
                    .monotonic_timestamp()
                    .0;
                let outcome = execution
                    .evidence
                    .events
                    .iter()
                    .find(|event| {
                        matches!(
                            event.event(),
                            CanonicalEventKind::UsefulOutcomeObserved { .. }
                        )
                    })
                    .expect("authoritative outcome")
                    .monotonic_timestamp()
                    .0;
                assert!(
                    outcome >= eos,
                    "FTOL uses authoritative boundary timestamps"
                );
            }
        } else {
            let correlations = executions.each_ref().map(|execution| {
                execution
                    .evidence
                    .events
                    .iter()
                    .filter_map(|event| match event.event() {
                        CanonicalEventKind::Architecture(ArchitectureEvent::RouteCommitted {
                            subgoal_id: Some(subgoal_id),
                            ..
                        }) => Some((
                            event.product_correlation().parent_task_id.clone(),
                            event.product_correlation().child_task_id.clone(),
                            subgoal_id.clone(),
                        )),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            });
            assert_eq!(correlations[0], correlations[1]);
        }
    }
}
