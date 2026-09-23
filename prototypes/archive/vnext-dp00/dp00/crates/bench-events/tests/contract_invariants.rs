use bench_core::{
    ArchitectureEvent, ArchitectureObservation, ExecutionRoute, ExecutionRouteKind, ExecutorId,
    ObservationPort, ProductCorrelation, SubgoalId, TaskId,
};
use bench_events::{
    CanonicalEventKind, ContractViolation, EventEmitter, ExecutionInvocation, FailureOutcomeReason,
    InMemoryObservationCollector, ObservableEffect, ObservableEffectType, ObservationContext,
    validate_episode,
};
use bench_fixtures::ControlledClock;

fn collector() -> InMemoryObservationCollector<ControlledClock> {
    InMemoryObservationCollector::with_capacity(
        ObservationContext {
            run_id: "run-owned-by-benchmark".into(),
            episode_id: "episode-1".into(),
            scenario_id: "fault-1".into(),
            scenario_version: "v1".into(),
            alternative_id: "A".into(),
            benchmark_version: "v1".into(),
            schema_version: "v1".into(),
            source_git_commit: "test".into(),
        },
        ControlledClock::new(0),
        16,
    )
}

fn route(executor: &str) -> ExecutionRoute {
    let executor = ExecutorId::from(executor);
    ExecutionRoute {
        route_kind: ExecutionRouteKind::ExecutorDirect,
        initial_executor_id: executor.clone(),
        final_executor_id_if_known: Some(executor.clone()),
        delegation_chain: vec![executor],
    }
}

fn architecture(
    collector: &InMemoryObservationCollector<ControlledClock>,
    event: ArchitectureEvent,
) {
    collector.emit(ArchitectureObservation {
        event,
        product_correlation: ProductCorrelation::default(),
    });
}

fn compound_commit(
    collector: &InMemoryObservationCollector<ControlledClock>,
    subgoal: &str,
    child: &str,
) {
    collector.emit(ArchitectureObservation {
        event: ArchitectureEvent::RouteCommitted {
            route: route("ARGO"),
            subgoal_id: Some(SubgoalId::from(subgoal)),
        },
        product_correlation: ProductCorrelation {
            task_id: Some(TaskId::from(child)),
            parent_task_id: Some(TaskId::from("parent")),
            child_task_id: Some(TaskId::from(child)),
            subgoal_id: Some(SubgoalId::from(subgoal)),
            ..ProductCorrelation::default()
        },
    });
}

fn benchmark(
    collector: &InMemoryObservationCollector<ControlledClock>,
    emitter: EventEmitter,
    event: CanonicalEventKind,
) {
    collector.capture_benchmark_event(emitter, event, ProductCorrelation::default());
}

fn outcome() -> CanonicalEventKind {
    CanonicalEventKind::UsefulOutcomeObserved {
        effect: ObservableEffect {
            effect_type: ObservableEffectType::WifiStatusObserved,
            capability_id: Some("network.status".into()),
            subject_id: Some("wifi_connection_1".into()),
            target_id: None,
            value: None,
            before_value: None,
            after_value: None,
            state: Some("OBSERVED".into()),
            executor_id: Some("NetworkAgent".into()),
            authoritative_source: EventEmitter::OutcomeProbe,
        },
    }
}

fn execution_started(executor_id: &str) -> CanonicalEventKind {
    CanonicalEventKind::ExecutionStarted {
        invocation: ExecutionInvocation {
            capability_id: "network.status".into(),
            executor_id: executor_id.into(),
        },
    }
}

#[test]
fn successful_episode_satisfies_canonical_partial_order() {
    let collector = collector();
    architecture(&collector, ArchitectureEvent::ProcessingStarted);
    benchmark(
        &collector,
        EventEmitter::ModelFixture,
        CanonicalEventKind::ModelGenerationStarted,
    );
    benchmark(
        &collector,
        EventEmitter::ModelFixture,
        CanonicalEventKind::ModelGenerationCompleted,
    );
    architecture(
        &collector,
        ArchitectureEvent::RouteCommitted {
            route: route("ARGO"),
            subgoal_id: None,
        },
    );
    benchmark(
        &collector,
        EventEmitter::AgentFixture,
        execution_started("ARGO"),
    );
    benchmark(&collector, EventEmitter::OutcomeProbe, outcome());
    benchmark(
        &collector,
        EventEmitter::Benchmark,
        CanonicalEventKind::EpisodeCompleted,
    );

    assert!(validate_episode(&collector.canonical_snapshot()).is_empty());
}

#[test]
fn zero_model_call_success_satisfies_canonical_partial_order() {
    let collector = collector();
    architecture(&collector, ArchitectureEvent::ProcessingStarted);
    architecture(
        &collector,
        ArchitectureEvent::RouteCommitted {
            route: route("VIA_LOCAL_VOLUME"),
            subgoal_id: None,
        },
    );
    benchmark(
        &collector,
        EventEmitter::ToolFixture,
        execution_started("VIA_LOCAL_VOLUME"),
    );
    benchmark(&collector, EventEmitter::OutcomeProbe, outcome());
    benchmark(
        &collector,
        EventEmitter::Benchmark,
        CanonicalEventKind::EpisodeCompleted,
    );

    assert!(validate_episode(&collector.canonical_snapshot()).is_empty());
}

#[test]
fn acoustic_eos_is_owned_by_the_interaction_fixture() {
    let collector = collector();
    benchmark(
        &collector,
        EventEmitter::InteractionFixture,
        CanonicalEventKind::AcousticEos,
    );
    let events = collector.canonical_snapshot();
    assert_eq!(events[0].event(), &CanonicalEventKind::AcousticEos);
    assert_eq!(events[0].emitter(), EventEmitter::InteractionFixture);
}

#[test]
fn duplicate_initial_commit_is_rejected() {
    let collector = collector();
    architecture(&collector, ArchitectureEvent::ProcessingStarted);
    architecture(
        &collector,
        ArchitectureEvent::RouteCommitted {
            route: route("ARGO"),
            subgoal_id: None,
        },
    );
    architecture(
        &collector,
        ArchitectureEvent::RouteCommitted {
            route: route("NetworkAgent"),
            subgoal_id: None,
        },
    );

    assert!(
        validate_episode(&collector.canonical_snapshot())
            .contains(&ContractViolation::DuplicateInitialRouteCommit)
    );
}

#[test]
fn distinct_subgoal_commits_form_a_valid_initial_route_plan_barrier() {
    let collector = collector();
    architecture(&collector, ArchitectureEvent::ProcessingStarted);
    compound_commit(&collector, "S1", "child-1");
    compound_commit(&collector, "S2", "child-2");
    benchmark(
        &collector,
        EventEmitter::AgentFixture,
        execution_started("ARGO"),
    );
    benchmark(&collector, EventEmitter::OutcomeProbe, outcome());
    benchmark(
        &collector,
        EventEmitter::Benchmark,
        CanonicalEventKind::EpisodeCompleted,
    );

    assert!(validate_episode(&collector.canonical_snapshot()).is_empty());
}

#[test]
fn duplicate_subgoal_commit_and_execution_before_barrier_are_rejected() {
    let duplicate = collector();
    architecture(&duplicate, ArchitectureEvent::ProcessingStarted);
    compound_commit(&duplicate, "S1", "child-1");
    compound_commit(&duplicate, "S1", "child-1");
    assert!(
        validate_episode(&duplicate.canonical_snapshot())
            .contains(&ContractViolation::DuplicateInitialRouteCommit)
    );

    let premature = collector();
    architecture(&premature, ArchitectureEvent::ProcessingStarted);
    compound_commit(&premature, "S1", "child-1");
    benchmark(
        &premature,
        EventEmitter::AgentFixture,
        execution_started("ARGO"),
    );
    compound_commit(&premature, "S2", "child-2");
    assert!(
        validate_episode(&premature.canonical_snapshot())
            .contains(&ContractViolation::ExecutionBeforeInitialRoutePlanBarrier)
    );
}

#[test]
fn compound_route_commit_requires_matching_parent_child_subgoal_correlation() {
    let collector = collector();
    architecture(&collector, ArchitectureEvent::ProcessingStarted);
    architecture(
        &collector,
        ArchitectureEvent::RouteCommitted {
            route: route("ARGO"),
            subgoal_id: Some(SubgoalId::from("S1")),
        },
    );

    assert!(
        validate_episode(&collector.canonical_snapshot())
            .contains(&ContractViolation::InvalidSemanticPayload)
    );
}

#[test]
fn rejected_candidate_can_be_reselected_without_becoming_a_commit() {
    let collector = collector();
    architecture(&collector, ArchitectureEvent::ProcessingStarted);
    benchmark(
        &collector,
        EventEmitter::ModelFixture,
        CanonicalEventKind::ModelGenerationStarted,
    );
    benchmark(
        &collector,
        EventEmitter::ModelFixture,
        CanonicalEventKind::ModelGenerationCompleted,
    );
    architecture(
        &collector,
        ArchitectureEvent::RouteCandidateObserved {
            route: route("NetworkAgent"),
        },
    );
    architecture(
        &collector,
        ArchitectureEvent::RouteCandidateRejected {
            route: route("NetworkAgent"),
            reason: "synchronous dispatch reject".into(),
        },
    );
    architecture(
        &collector,
        ArchitectureEvent::RouteCandidateObserved {
            route: route("ARGO"),
        },
    );
    architecture(
        &collector,
        ArchitectureEvent::RouteCommitted {
            route: route("ARGO"),
            subgoal_id: None,
        },
    );
    benchmark(
        &collector,
        EventEmitter::AgentFixture,
        execution_started("ARGO"),
    );
    benchmark(&collector, EventEmitter::OutcomeProbe, outcome());
    benchmark(
        &collector,
        EventEmitter::Benchmark,
        CanonicalEventKind::EpisodeCompleted,
    );

    let events = collector.canonical_snapshot();
    assert!(validate_episode(&events).is_empty());
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(
                event.event(),
                CanonicalEventKind::Architecture(ArchitectureEvent::RouteCommitted { .. })
            ))
            .count(),
        1
    );
}

#[test]
fn synchronously_rejected_candidate_cannot_be_committed() {
    let collector = collector();
    architecture(&collector, ArchitectureEvent::ProcessingStarted);
    architecture(
        &collector,
        ArchitectureEvent::RouteCandidateObserved {
            route: route("NetworkAgent"),
        },
    );
    architecture(
        &collector,
        ArchitectureEvent::RouteCandidateRejected {
            route: route("NetworkAgent"),
            reason: "synchronous dispatch reject".into(),
        },
    );
    architecture(
        &collector,
        ArchitectureEvent::RouteCommitted {
            route: route("NetworkAgent"),
            subgoal_id: None,
        },
    );

    assert!(
        validate_episode(&collector.canonical_snapshot())
            .contains(&ContractViolation::PrematureRouteCommit)
    );
}

#[test]
fn aut_self_report_cannot_be_authoritative_useful_outcome() {
    let collector = collector();
    architecture(&collector, ArchitectureEvent::ProcessingStarted);
    architecture(
        &collector,
        ArchitectureEvent::RouteCommitted {
            route: route("ARGO"),
            subgoal_id: None,
        },
    );
    benchmark(
        &collector,
        EventEmitter::AgentFixture,
        execution_started("ARGO"),
    );
    benchmark(&collector, EventEmitter::ArchitectureUnderTest, outcome());
    benchmark(
        &collector,
        EventEmitter::Benchmark,
        CanonicalEventKind::EpisodeCompleted,
    );

    let violations = validate_episode(&collector.canonical_snapshot());
    assert!(violations.contains(&ContractViolation::OutcomeNotProbeOwned));
    assert!(violations.contains(&ContractViolation::CompletedWithoutAuthoritativeOutcome));
}

#[test]
fn failed_episode_has_no_route_or_fabricated_outcome() {
    let collector = collector();
    architecture(&collector, ArchitectureEvent::ProcessingStarted);
    benchmark(
        &collector,
        EventEmitter::ModelFixture,
        CanonicalEventKind::ModelGenerationStarted,
    );
    benchmark(
        &collector,
        EventEmitter::ModelFixture,
        CanonicalEventKind::ModelGenerationCompleted,
    );
    benchmark(
        &collector,
        EventEmitter::Benchmark,
        CanonicalEventKind::EpisodeFailed {
            reason: FailureOutcomeReason::ModelTimeout,
        },
    );

    assert!(validate_episode(&collector.canonical_snapshot()).is_empty());
}

#[test]
fn adapter_owns_provenance_sequence_clock_and_emitter() {
    let collector = collector();
    architecture(&collector, ArchitectureEvent::ProcessingStarted);
    let events = collector.canonical_snapshot();
    let event = &events[0];

    assert_eq!(event.run_id(), "run-owned-by-benchmark");
    assert_eq!(event.scenario_id(), "fault-1");
    assert_eq!(event.alternative_id(), "A");
    assert_eq!(event.sequence_number(), 0);
    assert_eq!(event.emitter(), EventEmitter::ArchitectureUnderTest);
}
