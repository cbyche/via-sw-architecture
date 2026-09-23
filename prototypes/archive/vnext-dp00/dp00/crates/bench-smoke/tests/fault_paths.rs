use std::sync::{Arc, Mutex};

use alternative_a::ThinVia;
use alternative_b::ArgoCentric;
use alternative_c::HybridVia;
use alternative_d::AdaptiveVia;
use bench_core::{
    ArchitectureEvent, ArchitectureObservation, ArchitectureUnderTest, ContextEvidence,
    ExecutionId, ExecutionPort, ExecutionRequest, ExecutionResult, InitialProductState,
    InputModality, ModelPort, ModelRequest, ModelResponse, ModelStatus, MonotonicTimestamp,
    ObservationPort, ProductCorrelation, ResultId, SemanticResponsibility, TurnId, UserTurn,
};
use bench_events::{
    CanonicalEvent, CanonicalEventKind, EventEmitter, ExecutionInvocation, FailureOutcomeReason,
    InMemoryObservationCollector, LogicalModelCall, ObservableEffect, ObservableEffectType,
    ObservationContext, validate_episode,
};
use bench_fixtures::ControlledClock;
use bench_runner::{EvidenceSource, FixtureLifecycle, RawEvidence, Runner, ScenarioStimulus};

#[derive(Clone, Copy, Debug)]
enum InjectedBehavior {
    Correct,
    Malformed,
    Timeout,
    NoResponse,
    WrongCandidate,
    RejectNetworkCandidate,
}

struct Inner {
    behavior: InjectedBehavior,
    collector: InMemoryObservationCollector<ControlledClock>,
    requests: Mutex<Vec<ModelRequest>>,
    calls: Mutex<Vec<LogicalModelCall>>,
    executions: Mutex<Vec<ExecutionRequest>>,
}

#[derive(Clone)]
struct FaultPorts(Arc<Inner>);

impl FaultPorts {
    fn new(alternative: &str, behavior: InjectedBehavior) -> Self {
        Self(Arc::new(Inner {
            behavior,
            collector: InMemoryObservationCollector::with_capacity(
                ObservationContext {
                    run_id: format!("fault-{alternative}-{behavior:?}"),
                    episode_id: "episode-fault".into(),
                    scenario_id: "fault-route".into(),
                    scenario_version: "v1".into(),
                    alternative_id: alternative.into(),
                    benchmark_version: "dp00-fault-v1".into(),
                    schema_version: "canonical-event-v1".into(),
                    source_git_commit: "working-tree".into(),
                },
                ControlledClock::new(0),
                24,
            ),
            requests: Mutex::new(Vec::new()),
            calls: Mutex::new(Vec::new()),
            executions: Mutex::new(Vec::new()),
        }))
    }

    fn events(&self) -> Vec<CanonicalEvent> {
        self.0.collector.canonical_snapshot()
    }

    fn requests(&self) -> Vec<ModelRequest> {
        self.0
            .requests
            .lock()
            .expect("request trace poisoned")
            .clone()
    }

    fn calls(&self) -> Vec<LogicalModelCall> {
        self.0.calls.lock().expect("call trace poisoned").clone()
    }

    fn is_initial_interpretation(request: &ModelRequest) -> bool {
        request.decision_owner.0.ends_with("IntentRefiner")
    }
}

impl ModelPort for FaultPorts {
    type Error = String;

    fn generate(&self, request: ModelRequest) -> Result<ModelResponse, Self::Error> {
        self.0.collector.capture_benchmark_event(
            EventEmitter::ModelFixture,
            CanonicalEventKind::ModelGenerationStarted,
            ProductCorrelation::default(),
        );
        let (logical_sequence, attempt) = {
            let mut requests = self
                .0
                .requests
                .lock()
                .map_err(|_| "request trace poisoned")?;
            let sequence = requests.len() as u64 + 1;
            let attempt = requests
                .iter()
                .filter(|prior| prior.decision_owner == request.decision_owner)
                .count() as u32
                + 1;
            requests.push(request.clone());
            (sequence, attempt)
        };

        let interpretation = Self::is_initial_interpretation(&request);
        let (composite_output, model_status) = if interpretation {
            let output = if matches!(self.0.behavior, InjectedBehavior::RejectNetworkCandidate) {
                "DIAGNOSE_WIFI"
            } else {
                "GENERAL_FILE_WORK"
            };
            (output.into(), ModelStatus::Completed)
        } else {
            match self.0.behavior {
                InjectedBehavior::Correct => {
                    if request.decision_owner.0 == "B.ARGOPrimary" {
                        (
                            "ARGO_DIRECT GENERAL_FILE_WORK".into(),
                            ModelStatus::Completed,
                        )
                    } else {
                        ("ARGO".into(), ModelStatus::Completed)
                    }
                }
                InjectedBehavior::Malformed => {
                    ("not valid structured output".into(), ModelStatus::Malformed)
                }
                InjectedBehavior::Timeout => (String::new(), ModelStatus::TimedOut),
                InjectedBehavior::NoResponse => (String::new(), ModelStatus::NoResponse),
                InjectedBehavior::WrongCandidate => ("MAIL_AGENT".into(), ModelStatus::Completed),
                InjectedBehavior::RejectNetworkCandidate => {
                    let output = if attempt == 1 {
                        "NETWORK_AGENT"
                    } else {
                        "ARGO"
                    };
                    (output.into(), ModelStatus::Completed)
                }
            }
        };
        self.0.collector.capture_benchmark_event(
            EventEmitter::ModelFixture,
            CanonicalEventKind::ModelGenerationCompleted,
            ProductCorrelation::default(),
        );
        self.0
            .calls
            .lock()
            .map_err(|_| "call trace poisoned")?
            .push(LogicalModelCall {
                schema_version: "model-call-v1".into(),
                model_call_id: format!("fault-call-{logical_sequence}"),
                run_id: "fault-run".into(),
                episode_id: "fault-episode".into(),
                scenario_id: "fault".into(),
                alternative_id: "test".into(),
                logical_sequence,
                attempt,
                decision_owner: request.decision_owner,
                semantic_responsibilities: request.semantic_responsibilities,
                status: model_status,
                semantic_output_reference: None,
                route_committed_before_call: false,
                route_committed_after_call: false,
                logical_start: MonotonicTimestamp(logical_sequence * 10),
                first_output: Some(MonotonicTimestamp(logical_sequence * 10 + 1)),
                completion: MonotonicTimestamp(logical_sequence * 10 + 1),
                failure: (model_status != ModelStatus::Completed)
                    .then_some(MonotonicTimestamp(logical_sequence * 10 + 1)),
                call_class: "ORCHESTRATION".into(),
                classification_reason: "FAULT_PATH_ROUTE_RESPONSIBILITY".into(),
                qa04_primary_included: true,
                route_commit_event_id: None,
            });
        Ok(ModelResponse {
            composite_output,
            model_status,
        })
    }
}

impl ObservationPort for FaultPorts {
    fn emit(&self, observation: ArchitectureObservation) {
        self.0.collector.emit(observation);
    }
}

impl ExecutionPort for FaultPorts {
    type Error = String;

    fn accept_route(&self, route: &bench_core::ExecutionRoute) -> Result<(), Self::Error> {
        if matches!(self.0.behavior, InjectedBehavior::RejectNetworkCandidate)
            && route.initial_executor_id.0 == "NetworkAgent"
        {
            Err("NetworkAgent unavailable".into())
        } else {
            Ok(())
        }
    }

    fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult, Self::Error> {
        let executor_id = request.route.initial_executor_id.0.clone();
        self.0
            .executions
            .lock()
            .map_err(|_| "execution trace poisoned")?
            .push(request);
        self.0.collector.capture_benchmark_event(
            EventEmitter::AgentFixture,
            CanonicalEventKind::ExecutionStarted {
                invocation: ExecutionInvocation {
                    capability_id: "downloads.organize".into(),
                    executor_id,
                },
            },
            ProductCorrelation::default(),
        );
        self.0.collector.capture_benchmark_event(
            EventEmitter::OutcomeProbe,
            CanonicalEventKind::UsefulOutcomeObserved {
                effect: ObservableEffect {
                    effect_type: ObservableEffectType::DownloadsOrganized,
                    capability_id: Some("downloads.organize".into()),
                    subject_id: Some("downloads".into()),
                    target_id: None,
                    value: None,
                    before_value: None,
                    after_value: None,
                    state: Some("ORGANIZED".into()),
                    executor_id: Some("ARGO".into()),
                    authoritative_source: EventEmitter::OutcomeProbe,
                },
            },
            ProductCorrelation::default(),
        );
        Ok(ExecutionResult {
            execution_id: ExecutionId::from("execution-1"),
            result_id: ResultId::from("result-1"),
            payload: "done".into(),
        })
    }
}

impl EvidenceSource for FaultPorts {
    fn take_raw_evidence(&mut self) -> RawEvidence {
        RawEvidence {
            events: self.events(),
            model_calls: self.calls(),
            fixture_events: Vec::new(),
        }
    }
}

impl FixtureLifecycle for FaultPorts {
    type Error = String;

    fn before_episode(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn before_user_turn(&mut self, _turn_index: usize) -> Result<(), Self::Error> {
        Ok(())
    }

    fn after_episode(&mut self) -> Result<(), Self::Error> {
        self.0.collector.capture_benchmark_event(
            EventEmitter::Benchmark,
            CanonicalEventKind::EpisodeCompleted,
            ProductCorrelation::default(),
        );
        Ok(())
    }

    fn after_failure(&mut self) -> Result<(), Self::Error> {
        self.0.collector.capture_benchmark_event(
            EventEmitter::Benchmark,
            CanonicalEventKind::EpisodeFailed {
                reason: match self.0.behavior {
                    InjectedBehavior::Malformed => FailureOutcomeReason::ModelMalformed,
                    InjectedBehavior::Timeout => FailureOutcomeReason::ModelTimeout,
                    InjectedBehavior::NoResponse => FailureOutcomeReason::ModelNoResponse,
                    InjectedBehavior::RejectNetworkCandidate => {
                        FailureOutcomeReason::DispatchRejected
                    }
                    InjectedBehavior::Correct | InjectedBehavior::WrongCandidate => {
                        FailureOutcomeReason::InvalidRoute
                    }
                },
            },
            ProductCorrelation::default(),
        );
        Ok(())
    }
}

fn turn() -> UserTurn {
    UserTurn {
        turn_id: TurnId::from("turn-1"),
        modality: InputModality::Text,
        content: "다운로드 폴더를 정리해줘.".into(),
        context_evidence: vec![ContextEvidence {
            source: "controlled-interaction".into(),
            kind: "raw-context".into(),
            value: "fixture evidence only".into(),
            observed_at_product_revision: None,
        }],
    }
}

fn run_fault<A: ArchitectureUnderTest<Error = String>>(
    mut architecture: A,
    ports: &FaultPorts,
) -> String {
    let mut evidence = ports.clone();
    let mut lifecycle = ports.clone();
    match Runner.run(
        &mut architecture,
        &mut evidence,
        &mut lifecycle,
        ScenarioStimulus {
            initial_state: InitialProductState::default(),
            turns: vec![turn()],
        },
    ) {
        Err(bench_runner::RunnerError::Architecture(error)) => error,
        result => panic!("fault path must terminate through Runner: {result:?}"),
    }
}

fn execute_fault(alternative: &str, behavior: InjectedBehavior) -> (String, FaultPorts) {
    let ports = FaultPorts::new(alternative, behavior);
    let error = match alternative {
        "A" => run_fault(
            ThinVia::new(ports.clone(), ports.clone(), ports.clone()),
            &ports,
        ),
        "B" => run_fault(
            ArgoCentric::new(ports.clone(), ports.clone(), ports.clone()),
            &ports,
        ),
        "C" => run_fault(
            HybridVia::new(ports.clone(), ports.clone(), ports.clone()),
            &ports,
        ),
        "D" => run_fault(
            AdaptiveVia::new(ports.clone(), ports.clone(), ports.clone()),
            &ports,
        ),
        _ => panic!("unknown alternative"),
    };
    (error, ports)
}

fn assert_terminal_fault(alternative: &str, behavior: InjectedBehavior, expected_calls: usize) {
    let (_error, ports) = execute_fault(alternative, behavior);
    assert_eq!(ports.requests().len(), expected_calls);
    let calls = ports.calls();
    assert_eq!(calls.len(), expected_calls);
    assert_eq!(
        calls
            .iter()
            .map(|call| call.logical_sequence)
            .collect::<Vec<_>>(),
        (1..=expected_calls as u64).collect::<Vec<_>>()
    );
    assert!(
        calls
            .iter()
            .all(|call| !call.route_committed_before_call && !call.route_committed_after_call)
    );
    assert!(
        ports
            .0
            .executions
            .lock()
            .expect("execution trace poisoned")
            .is_empty()
    );
    let events = ports.events();
    assert!(
        events
            .iter()
            .any(|event| matches!(event.event(), CanonicalEventKind::EpisodeFailed { .. }))
    );
    assert!(!events.iter().any(|event| matches!(
        event.event(),
        CanonicalEventKind::Architecture(ArchitectureEvent::RouteCommitted { .. })
            | CanonicalEventKind::UsefulOutcomeObserved { .. }
    )));
    assert!(validate_episode(&events).is_empty());
}

#[test]
fn malformed_timeout_no_response_and_wrong_candidate_fail_safely_for_all_alternatives() {
    for alternative in ["A", "B", "C", "D"] {
        let expected_calls = if alternative == "B" { 1 } else { 2 };
        for behavior in [
            InjectedBehavior::Malformed,
            InjectedBehavior::Timeout,
            InjectedBehavior::NoResponse,
            InjectedBehavior::WrongCandidate,
        ] {
            assert_terminal_fault(alternative, behavior, expected_calls);
        }
    }
}

#[test]
fn b_argo_primary_retains_combined_pre_route_responsibility() {
    let ports = FaultPorts::new("B", InjectedBehavior::Correct);
    let mut architecture = ArgoCentric::new(ports.clone(), ports.clone(), ports.clone());
    architecture
        .setup(InitialProductState::default())
        .expect("setup succeeds");
    architecture.handle_user_turn(turn()).expect("B executes");

    let requests = ports.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].decision_owner.0, "B.ARGOPrimary");
    assert_eq!(
        requests[0].semantic_responsibilities,
        vec![
            SemanticResponsibility::IntentInterpretation,
            SemanticResponsibility::ReferentResolution,
            SemanticResponsibility::ExecutionRouteSelection,
            SemanticResponsibility::AgentSelection,
        ]
    );
}

#[test]
fn network_rejection_reselects_argo_before_commit_with_one_retry_generation() {
    let ports = FaultPorts::new("A", InjectedBehavior::RejectNetworkCandidate);
    let mut architecture = ThinVia::new(ports.clone(), ports.clone(), ports.clone());
    architecture
        .setup(InitialProductState::default())
        .expect("setup succeeds");
    architecture
        .handle_user_turn(turn())
        .expect("deterministic fallback succeeds");
    let mut lifecycle = ports.clone();
    lifecycle.after_episode().expect("completion event emits");

    assert_eq!(
        ports.requests().len(),
        3,
        "AgentRouter reselection is one new logical retry generation"
    );
    assert_eq!(ports.calls()[1].attempt, 1);
    assert_eq!(ports.calls()[2].attempt, 2);
    let events = ports.events();
    let commits: Vec<_> = events
        .iter()
        .filter_map(|event| match event.event() {
            CanonicalEventKind::Architecture(ArchitectureEvent::RouteCommitted {
                route, ..
            }) => Some(route),
            _ => None,
        })
        .collect();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].initial_executor_id.0, "ARGO");
    assert!(events.iter().any(|event| matches!(
        event.event(),
        CanonicalEventKind::Architecture(ArchitectureEvent::RouteCandidateRejected { route, .. })
            if route.initial_executor_id.0 == "NetworkAgent"
    )));
    assert!(validate_episode(&events).is_empty());
}
