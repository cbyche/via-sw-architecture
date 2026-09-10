#![forbid(unsafe_code)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use bench_core::{
    ArchitectureObservation, ExecutionId, ExecutionPort, ExecutionRequest, ExecutionResult,
    ModelPort, ModelRequest, ModelResponse, ModelStatus, MonotonicTimestamp, ObservationPort,
    ProductCorrelation, ResultId, SemanticResponsibility,
};
use bench_events::{
    CanonicalEvent, CanonicalEventKind, EventEmitter, FailureOutcomeReason,
    InMemoryObservationCollector, LogicalModelCall, ObservableEffect, ObservableEffectType,
    ObservationContext,
};
use bench_fixtures::ControlledClock;
use bench_replay::{
    ReplayAdapter, ReplayAttempt, ReplayContext, ReplayOperation, ResponsibilityMapping,
};
use bench_runner::{EvidenceSource, FixtureLifecycle, RawEvidence};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scenario {
    S1Local,
    S2General,
    S3Specialized,
    S4FollowUp,
    S5Clarification,
}

impl Scenario {
    pub fn id(self) -> &'static str {
        match self {
            Self::S1Local => "S1",
            Self::S2General => "S2",
            Self::S3Specialized => "S3",
            Self::S4FollowUp => "S4",
            Self::S5Clarification => "S5",
        }
    }
}

#[derive(Debug, Default)]
struct WorldState {
    executions: Vec<ExecutionRequest>,
    outcomes: Vec<String>,
}

struct Inner {
    scenario: Scenario,
    collector: InMemoryObservationCollector<ControlledClock>,
    replay: ReplayAdapter,
    calls: Mutex<Vec<LogicalModelCall>>,
    requests: Mutex<Vec<ModelRequest>>,
    world: Mutex<WorldState>,
    next_id: AtomicU64,
}

#[derive(Clone)]
pub struct SmokePorts {
    inner: Arc<Inner>,
}

impl SmokePorts {
    pub fn new(alternative: &str, scenario: Scenario) -> Self {
        let collector = InMemoryObservationCollector::with_capacity(
            ObservationContext {
                run_id: format!("smoke-{alternative}-{}", scenario.id()),
                episode_id: format!("episode-{}", scenario.id()),
                scenario_id: scenario.id().into(),
                scenario_version: "s1-s5-v0".into(),
                alternative_id: alternative.into(),
                benchmark_version: "dp00-smoke-v0".into(),
                schema_version: "canonical-event-v1".into(),
                source_git_commit: "working-tree".into(),
            },
            ControlledClock::new(0),
            32,
        );
        Self {
            inner: Arc::new(Inner {
                scenario,
                collector,
                replay: replay(alternative, scenario),
                calls: Mutex::new(Vec::new()),
                requests: Mutex::new(Vec::new()),
                world: Mutex::new(WorldState::default()),
                next_id: AtomicU64::new(1),
            }),
        }
    }

    pub fn events(&self) -> Vec<CanonicalEvent> {
        self.inner.collector.canonical_snapshot()
    }

    pub fn model_requests(&self) -> Vec<ModelRequest> {
        self.inner
            .requests
            .lock()
            .expect("request trace poisoned")
            .clone()
    }

    pub fn executions(&self) -> Vec<ExecutionRequest> {
        self.inner
            .world
            .lock()
            .expect("world state poisoned")
            .executions
            .clone()
    }

    pub fn outcomes(&self) -> Vec<String> {
        self.inner
            .world
            .lock()
            .expect("world state poisoned")
            .outcomes
            .clone()
    }

    fn capture(&self, emitter: EventEmitter, event: CanonicalEventKind) {
        self.inner
            .collector
            .capture_benchmark_event(emitter, event, ProductCorrelation::default());
    }
}

impl ModelPort for SmokePorts {
    type Error = String;

    fn generate(&self, request: ModelRequest) -> Result<ModelResponse, Self::Error> {
        let call_number = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        self.capture(
            EventEmitter::ModelFixture,
            CanonicalEventKind::ModelGenerationStarted,
        );
        let (logical_sequence, attempt) = {
            let mut requests = self
                .inner
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
        let response = self
            .inner
            .replay
            .generate(request.clone())
            .map_err(|error| format!("{error:?}"))?;
        self.inner
            .calls
            .lock()
            .map_err(|_| "model-call trace poisoned")?
            .push(LogicalModelCall {
                schema_version: "model-call-v1".into(),
                model_call_id: format!("model-call-{call_number}"),
                run_id: format!(
                    "smoke-{}-{}",
                    self.inner.replay.context().alternative_id,
                    self.inner.scenario.id()
                ),
                episode_id: format!("episode-{}", self.inner.scenario.id()),
                scenario_id: self.inner.scenario.id().into(),
                alternative_id: self.inner.replay.context().alternative_id.clone(),
                logical_sequence,
                attempt,
                decision_owner: request.decision_owner,
                semantic_responsibilities: request.semantic_responsibilities,
                status: response.model_status,
                semantic_output_reference: None,
                route_committed_before_call: false,
                route_committed_after_call: false,
                logical_start: MonotonicTimestamp(call_number * 10),
                first_output: Some(MonotonicTimestamp(call_number * 10 + 1)),
                completion: MonotonicTimestamp(call_number * 10 + 1),
                failure: None,
                call_class: "ORCHESTRATION".into(),
                classification_reason: "BASE_ARCHITECTURE_ROUTE_RESPONSIBILITY".into(),
                qa04_primary_included: true,
                route_commit_event_id: None,
            });
        self.capture(
            EventEmitter::ModelFixture,
            CanonicalEventKind::ModelGenerationCompleted,
        );
        Ok(response)
    }
}

impl ObservationPort for SmokePorts {
    fn emit(&self, observation: ArchitectureObservation) {
        self.inner.collector.emit(observation);
    }
}

impl ExecutionPort for SmokePorts {
    type Error = String;

    fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult, Self::Error> {
        let emitter = if request.route.route_kind == bench_core::ExecutionRouteKind::LocalDirect {
            EventEmitter::ToolFixture
        } else {
            EventEmitter::AgentFixture
        };
        self.capture(emitter, CanonicalEventKind::ExecutionStarted);
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        let outcome = observable_outcome(&request.semantic_action);
        let result = ExecutionResult {
            execution_id: ExecutionId(format!("execution-{id}")),
            result_id: ResultId(format!("result-{id}")),
            payload: outcome.clone(),
        };
        let mut world = self
            .inner
            .world
            .lock()
            .map_err(|_| "world state poisoned")?;
        world.executions.push(request);
        world.outcomes.push(outcome);
        drop(world);
        self.capture(
            EventEmitter::OutcomeProbe,
            CanonicalEventKind::UsefulOutcomeObserved {
                effect: smoke_effect(&result.payload),
            },
        );
        Ok(result)
    }
}

impl EvidenceSource for SmokePorts {
    fn take_raw_evidence(&mut self) -> RawEvidence {
        RawEvidence {
            events: self.events(),
            model_calls: self
                .inner
                .calls
                .lock()
                .expect("model-call trace poisoned")
                .clone(),
            fixture_events: Vec::new(),
        }
    }
}

impl FixtureLifecycle for SmokePorts {
    type Error = String;

    fn before_episode(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn before_user_turn(&mut self, turn_index: usize) -> Result<(), Self::Error> {
        if self.inner.scenario == Scenario::S5Clarification && turn_index == 1 {
            let clarification_requested = self.events().iter().any(|event| {
                matches!(
                    event.event(),
                    CanonicalEventKind::Architecture(
                        bench_core::ArchitectureEvent::ClarificationRequested { .. }
                    )
                )
            });
            if !clarification_requested {
                return Err(
                    "scripted clarification reply requested before AUT clarification".into(),
                );
            }
        }
        Ok(())
    }

    fn after_episode(&mut self) -> Result<(), Self::Error> {
        self.capture(
            EventEmitter::Benchmark,
            CanonicalEventKind::EpisodeCompleted,
        );
        Ok(())
    }

    fn after_failure(&mut self) -> Result<(), Self::Error> {
        self.capture(
            EventEmitter::Benchmark,
            CanonicalEventKind::EpisodeFailed {
                reason: FailureOutcomeReason::ExecutionFailure,
            },
        );
        Ok(())
    }
}

fn smoke_effect(outcome: &str) -> ObservableEffect {
    ObservableEffect {
        effect_type: if outcome == "right_document_opened" {
            ObservableEffectType::DocumentOpened
        } else if outcome == "volume_reduced" {
            ObservableEffectType::VolumeChanged
        } else if outcome == "downloads_organized" {
            ObservableEffectType::DownloadsOrganized
        } else {
            ObservableEffectType::WifiStatusObserved
        },
        subject_id: Some(outcome.into()),
        target_id: None,
        value: None,
        state: Some("OBSERVED".into()),
        executor_id: None,
        authoritative_source: EventEmitter::OutcomeProbe,
    }
}

fn observable_outcome(action: &str) -> String {
    if action.contains("Volume") || action.contains("LOCAL_VOLUME") {
        "volume_reduced".into()
    } else if action.contains("Wifi")
        || action.contains("WIFI")
        || action.contains("Continue")
        || action.contains("CONTINUE")
    {
        "wifi_diagnosis_updated".into()
    } else if action.contains("Document") || action.contains("DOCUMENT") {
        "right_document_opened".into()
    } else {
        "downloads_organized".into()
    }
}

fn replay(alternative: &str, scenario: Scenario) -> ReplayAdapter {
    ReplayAdapter::new(
        ReplayContext {
            run_id: format!("smoke-{alternative}-{}", scenario.id()),
            episode_id: format!("episode-{}", scenario.id()),
            scenario_id: scenario.id().into(),
            scenario_version: "s1-s5-v0".into(),
            alternative_id: alternative.into(),
            current_turn_fixture: "turn-script-v0".into(),
            semantic_behavior_plan_id: format!("{}-behavior-v0", scenario.id()),
            semantic_behavior_plan_version: "v0".into(),
            owner_responsibility_mapping_version: "dp00-base-v0".into(),
            replay_payload_registry_version: "smoke-payload-v0".into(),
        },
        ResponsibilityMapping::dp00_base(),
        operations(alternative, scenario),
    )
}

fn operations(alternative: &str, scenario: Scenario) -> Vec<ReplayOperation> {
    let completed = |output: &str| ReplayAttempt {
        output: output.into(),
        status: ModelStatus::Completed,
    };
    let intent = match scenario {
        Scenario::S1Local => vec![completed("LOCAL_VOLUME")],
        Scenario::S2General => vec![completed("GENERAL_FILE_WORK")],
        Scenario::S3Specialized => vec![completed("DIAGNOSE_WIFI")],
        Scenario::S4FollowUp => vec![completed("CONTINUE_T1")],
        Scenario::S5Clarification => vec![
            completed("AMBIGUOUS_DOCUMENT"),
            completed("OPEN_RIGHT_DOCUMENT"),
        ],
    };
    let referent = intent.clone();
    let route_output = match (alternative, scenario) {
        ("B", Scenario::S1Local) => "ARGO_DIRECT LOCAL_VOLUME",
        ("B", Scenario::S2General) => "ARGO_DIRECT GENERAL_FILE_WORK",
        ("B", Scenario::S3Specialized) => "DELEGATE NETWORK_AGENT DIAGNOSE_WIFI",
        ("B", Scenario::S4FollowUp) => "CONTINUE_T1",
        ("B", Scenario::S5Clarification) => "ARGO_DIRECT OPEN_RIGHT_DOCUMENT",
        ("D", Scenario::S1Local) => "LOCAL_VOLUME",
        ("D", Scenario::S2General) => "ARGO",
        ("D", Scenario::S3Specialized) => "NETWORK_AGENT",
        ("D", Scenario::S4FollowUp) => "CONTINUE_T1",
        ("D", Scenario::S5Clarification) => "LOCAL_DOCUMENT",
        (_, _) => "UNUSED_ROUTE",
    };
    let route = if scenario == Scenario::S5Clarification && alternative == "B" {
        vec![completed("UNRESOLVED"), completed(route_output)]
    } else {
        vec![completed(route_output)]
    };
    let agent_output = match scenario {
        Scenario::S3Specialized => "NETWORK_AGENT",
        _ => "ARGO",
    };
    let agent = if scenario == Scenario::S5Clarification && alternative == "B" {
        vec![completed("UNRESOLVED"), completed("ARGO")]
    } else {
        vec![completed(agent_output)]
    };
    vec![
        ReplayOperation::new(
            "turn.intent",
            SemanticResponsibility::IntentInterpretation,
            intent,
        ),
        ReplayOperation::new(
            "turn.referent",
            SemanticResponsibility::ReferentResolution,
            referent,
        ),
        ReplayOperation::new(
            "turn.route",
            SemanticResponsibility::ExecutionRouteSelection,
            route,
        ),
        ReplayOperation::new("turn.agent", SemanticResponsibility::AgentSelection, agent),
    ]
}
