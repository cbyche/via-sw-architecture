//! Timed, architecture-neutral Pilot episode integration.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use alternative_a::ThinVia;
use alternative_b::ArgoCentric;
use alternative_c::HybridVia;
use alternative_d::AdaptiveVia;
use bench_core::{
    ArchitectureObservation, ArchitectureUnderTest, Clock, ExecutionId, ExecutionPort,
    ExecutionRequest, ExecutionResult, ModelPort, ModelRequest, ModelResponse, MonotonicTimestamp,
    ObservationPort, ProductCorrelation, ResultId,
};
use bench_events::{
    CanonicalEvent, CanonicalEventKind, EventEmitter, FailureOutcomeReason,
    InMemoryObservationCollector, InstrumentationMode, LogicalModelCall,
    ModelSemanticOutputReference, ModelSemanticValueKind, ObservableEffect, ObservableEffectType,
    ObservationContext,
};
use bench_fixtures::SystemMonotonicClock;
use bench_fixtures::pilot_assets::{BehaviorPlan, RuntimeScenario};
use bench_fixtures::pilot_materialization::{MaterializedScenario, materialize_scenario};
use bench_replay::{ReplayAdapter, ReplayContext, ResponsibilityMapping};

use crate::{
    Alternative, ControlledLatencyProfile, EvidenceSource, FixtureEvent, FixtureLifecycle,
    RawEvidence, Runner, ScenarioStimulus,
};

#[derive(Clone)]
struct SharedClock(Arc<SystemMonotonicClock>);

impl Clock for SharedClock {
    fn now(&self) -> MonotonicTimestamp {
        self.0.now()
    }
}

#[derive(Debug, Default)]
struct FixtureState {
    events: Vec<CapturedFixtureEvent>,
    execution_count: u64,
}

#[derive(Clone, Debug)]
enum FixtureEventKind {
    AgentStartAccepted,
    ToolStartAccepted,
    OutcomeDetected(ObservableEffect),
    AcousticEos {
        turn_index: usize,
        offset_micros: u64,
    },
}

#[derive(Clone, Debug)]
struct CapturedFixtureEvent {
    sequence_number: u64,
    monotonic_timestamp: MonotonicTimestamp,
    kind: FixtureEventKind,
    subject_id: String,
}

struct Inner {
    run_id: String,
    episode_id: String,
    scenario_id: String,
    collector: InMemoryObservationCollector<SharedClock>,
    clock: SharedClock,
    replay: ReplayAdapter,
    latency: ControlledLatencyProfile,
    acoustic_eos_offsets_micros: Vec<Option<u64>>,
    clarification_triggered_turns: Vec<bool>,
    turn_ids: Vec<String>,
    calls: Mutex<Vec<LogicalModelCall>>,
    fixtures: Mutex<FixtureState>,
    next_model_call_id: AtomicU64,
}

#[derive(Clone)]
pub struct PilotPorts {
    inner: Arc<Inner>,
}

pub struct PilotPortsConfig<'a> {
    pub run_id: &'a str,
    pub episode_id: &'a str,
    pub alternative: Alternative,
    pub source_git_commit: &'a str,
    pub instrumentation_mode: InstrumentationMode,
    pub latency_profile: ControlledLatencyProfile,
}

impl PilotPorts {
    pub fn new(
        config: PilotPortsConfig<'_>,
        scenario: &RuntimeScenario,
        plan: &BehaviorPlan,
        materialized: MaterializedScenario,
    ) -> Self {
        let clock = SharedClock(Arc::new(SystemMonotonicClock::default()));
        let turn_ids = materialized
            .turns
            .iter()
            .map(|turn| turn.turn_id.0.clone())
            .collect();
        let collector = InMemoryObservationCollector::with_mode(
            ObservationContext {
                run_id: config.run_id.into(),
                episode_id: config.episode_id.into(),
                scenario_id: scenario.scenario_id.clone(),
                scenario_version: scenario.scenario_version.clone(),
                alternative_id: config.alternative.id().into(),
                benchmark_version: scenario.provenance.benchmark_version.clone(),
                schema_version: scenario.provenance.canonical_event_schema_version.clone(),
                source_git_commit: config.source_git_commit.into(),
            },
            clock.clone(),
            64,
            config.instrumentation_mode,
        );
        Self {
            inner: Arc::new(Inner {
                run_id: config.run_id.into(),
                episode_id: config.episode_id.into(),
                scenario_id: scenario.scenario_id.clone(),
                collector,
                clock,
                replay: ReplayAdapter::new(
                    ReplayContext {
                        run_id: config.run_id.into(),
                        episode_id: config.episode_id.into(),
                        scenario_id: scenario.scenario_id.clone(),
                        scenario_version: scenario.scenario_version.clone(),
                        alternative_id: config.alternative.id().into(),
                        current_turn_fixture: scenario
                            .stimulus
                            .interaction
                            .first()
                            .map_or_else(|| "NONE".into(), |turn| turn.fixture_version.clone()),
                        semantic_behavior_plan_id: plan.behavior_plan_id.clone(),
                        semantic_behavior_plan_version: plan.behavior_plan_version.clone(),
                        owner_responsibility_mapping_version: plan
                            .allowed_owner_mapping_version
                            .clone(),
                        replay_payload_registry_version: plan.payload_registry_version.clone(),
                    },
                    ResponsibilityMapping::dp00_base(),
                    materialized.replay_operations,
                ),
                latency: config.latency_profile,
                acoustic_eos_offsets_micros: materialized.acoustic_eos_offsets_micros,
                clarification_triggered_turns: materialized.clarification_triggered_turns,
                turn_ids,
                calls: Mutex::new(Vec::new()),
                fixtures: Mutex::new(FixtureState::default()),
                next_model_call_id: AtomicU64::new(1),
            }),
        }
    }

    #[must_use]
    pub fn events(&self) -> Vec<CanonicalEvent> {
        self.inner.collector.canonical_snapshot()
    }

    #[must_use]
    pub fn capture_diagnostics(&self) -> (u64, u64) {
        self.inner.collector.capture_diagnostics()
    }

    fn capture(&self, emitter: EventEmitter, event: CanonicalEventKind) {
        self.capture_correlated(emitter, event, ProductCorrelation::default());
    }

    fn capture_correlated(
        &self,
        emitter: EventEmitter,
        event: CanonicalEventKind,
        product_correlation: ProductCorrelation,
    ) {
        self.inner
            .collector
            .capture_benchmark_event(emitter, event, product_correlation);
    }

    fn fixture_event(&self, kind: FixtureEventKind, subject_id: String) {
        let timestamp = self.inner.clock.now();
        if self.inner.collector.instrumentation_mode() == InstrumentationMode::Minimal {
            return;
        }
        let mut fixtures = self.inner.fixtures.lock().expect("fixture trace poisoned");
        let sequence_number = u64::try_from(fixtures.events.len()).unwrap_or(u64::MAX);
        fixtures.events.push(CapturedFixtureEvent {
            sequence_number,
            monotonic_timestamp: timestamp,
            kind,
            subject_id,
        });
    }
}

impl ModelPort for PilotPorts {
    type Error = String;

    fn generate(&self, request: ModelRequest) -> Result<ModelResponse, Self::Error> {
        let model_call_number = self
            .inner
            .next_model_call_id
            .fetch_add(1, Ordering::Relaxed);
        let start = self.inner.clock.now();
        self.capture(
            EventEmitter::ModelFixture,
            CanonicalEventKind::ModelGenerationStarted,
        );
        thread::sleep(Duration::from_micros(self.inner.latency.model_delay_micros));
        let response = self
            .inner
            .replay
            .generate(request.clone())
            .map_err(|error| format!("{error:?}"));
        let completion = self.inner.clock.now();
        self.capture(
            EventEmitter::ModelFixture,
            CanonicalEventKind::ModelGenerationCompleted,
        );
        let status = response
            .as_ref()
            .map_or(bench_core::ModelStatus::Failed, |response| {
                response.model_status
            });
        if self.inner.collector.instrumentation_mode() == InstrumentationMode::Capture {
            let mut calls = self
                .inner
                .calls
                .lock()
                .map_err(|_| "model call trace poisoned")?;
            let logical_sequence = u64::try_from(calls.len()).unwrap_or(u64::MAX) + 1;
            let attempt = u32::try_from(
                calls
                    .iter()
                    .filter(|call| call.decision_owner == request.decision_owner)
                    .count(),
            )
            .unwrap_or(u32::MAX)
            .saturating_add(1);
            calls.push(LogicalModelCall {
                schema_version: "model-call-v1".into(),
                model_call_id: format!("{}:model:{model_call_number}", self.inner.run_id),
                run_id: self.inner.run_id.clone(),
                episode_id: self.inner.episode_id.clone(),
                scenario_id: self.inner.scenario_id.clone(),
                alternative_id: self.inner.replay.context().alternative_id.clone(),
                logical_sequence,
                attempt,
                decision_owner: request.decision_owner.clone(),
                semantic_responsibilities: request.semantic_responsibilities.clone(),
                status,
                semantic_output_reference: response
                    .as_ref()
                    .ok()
                    .and_then(model_semantic_output_reference),
                route_committed_before_call: false,
                route_committed_after_call: false,
                logical_start: start,
                first_output: (status == bench_core::ModelStatus::Completed).then_some(completion),
                completion,
                failure: (status != bench_core::ModelStatus::Completed).then_some(completion),
                call_class: if request.decision_owner.0 == "B.ARGOPrimary" {
                    "MIXED".into()
                } else {
                    "ORCHESTRATION".into()
                },
                classification_reason: "BASE_ARCHITECTURE_PRE_ROUTE_SEMANTIC_RESPONSIBILITY".into(),
                qa04_primary_included: false,
                route_commit_event_id: None,
            });
        }
        response
    }
}

impl ObservationPort for PilotPorts {
    fn emit(&self, observation: ArchitectureObservation) {
        self.inner.collector.emit(observation);
    }
}

impl ExecutionPort for PilotPorts {
    type Error = String;

    fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult, Self::Error> {
        let is_tool = request.route.route_kind == bench_core::ExecutionRouteKind::LocalDirect;
        let (emitter, fixture_kind, delay) = if is_tool {
            (
                EventEmitter::ToolFixture,
                FixtureEventKind::ToolStartAccepted,
                self.inner.latency.tool_delay_micros,
            )
        } else {
            (
                EventEmitter::AgentFixture,
                FixtureEventKind::AgentStartAccepted,
                self.inner.latency.agent_delay_micros,
            )
        };
        let mut fixtures = self.inner.fixtures.lock().expect("fixture state poisoned");
        fixtures.execution_count += 1;
        let number = fixtures.execution_count;
        drop(fixtures);
        let execution_id = ExecutionId(format!("{}:execution:{number}", self.inner.run_id));
        let result_id = ResultId(format!("{}:result:{number}", self.inner.run_id));
        let correlation = ProductCorrelation {
            task_id: Some(request.task_id.clone()),
            execution_id: Some(execution_id.clone()),
            result_id: Some(result_id.clone()),
            ..ProductCorrelation::default()
        };
        self.capture_correlated(
            emitter,
            CanonicalEventKind::ExecutionStarted,
            correlation.clone(),
        );
        self.fixture_event(fixture_kind, request.route.initial_executor_id.0.clone());
        thread::sleep(Duration::from_micros(delay));
        let effect = effect_from_execution_request(&request);
        self.fixture_event(
            FixtureEventKind::OutcomeDetected(effect.clone()),
            effect
                .subject_id
                .clone()
                .unwrap_or_else(|| request.task_id.0.clone()),
        );
        self.capture_correlated(
            EventEmitter::OutcomeProbe,
            CanonicalEventKind::UsefulOutcomeObserved { effect },
            correlation,
        );
        Ok(ExecutionResult {
            execution_id,
            result_id,
            payload: format!("{}:fixture-outcome", self.inner.scenario_id),
        })
    }
}

impl EvidenceSource for PilotPorts {
    fn take_raw_evidence(&mut self) -> RawEvidence {
        let events = self.events();
        let route_commit = events.iter().find(|event| {
            matches!(
                event.event(),
                CanonicalEventKind::Architecture(
                    bench_core::ArchitectureEvent::RouteCommitted { .. }
                )
            )
        });
        let mut calls = self
            .inner
            .calls
            .lock()
            .expect("model trace poisoned")
            .clone();
        if let Some(commit) = route_commit {
            for call in &mut calls {
                call.route_committed_before_call =
                    commit.monotonic_timestamp() < call.logical_start;
                call.route_committed_after_call = commit.monotonic_timestamp() >= call.completion;
                call.qa04_primary_included = call.completion <= commit.monotonic_timestamp()
                    && matches!(call.call_class.as_str(), "ORCHESTRATION" | "MIXED");
                if call.qa04_primary_included {
                    call.route_commit_event_id = Some(commit.event_id().into());
                }
            }
        }
        let fixture_events = self
            .inner
            .fixtures
            .lock()
            .expect("fixture trace poisoned")
            .events
            .iter()
            .map(|event| enrich_fixture_event(&self.inner, event))
            .collect();
        RawEvidence {
            events,
            model_calls: calls,
            fixture_events,
        }
    }
}

impl FixtureLifecycle for PilotPorts {
    type Error = String;

    fn before_episode(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn before_user_turn(&mut self, turn_index: usize) -> Result<(), Self::Error> {
        let turn_id = self
            .inner
            .turn_ids
            .get(turn_index)
            .ok_or_else(|| "turn fixture correlation missing".to_owned())?;
        self.inner
            .replay
            .set_current_turn(turn_id)
            .map_err(|error| format!("{error:?}"))?;
        if self
            .inner
            .clarification_triggered_turns
            .get(turn_index)
            .copied()
            .unwrap_or(false)
            && !self.events().iter().any(|event| {
                matches!(
                    event.event(),
                    CanonicalEventKind::Architecture(
                        bench_core::ArchitectureEvent::ClarificationRequested { .. }
                    )
                )
            })
        {
            return Err("scripted reply requires an actual clarification request".into());
        }
        if let Some(Some(offset)) = self.inner.acoustic_eos_offsets_micros.get(turn_index) {
            self.fixture_event(
                FixtureEventKind::AcousticEos {
                    turn_index,
                    offset_micros: *offset,
                },
                String::new(),
            );
            self.capture(
                EventEmitter::InteractionFixture,
                CanonicalEventKind::AcousticEos,
            );
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
        Ok(())
    }
}

fn enrich_fixture_event(inner: &Inner, event: &CapturedFixtureEvent) -> FixtureEvent {
    let (fixture_kind, action, outcome, subject_id) = match &event.kind {
        FixtureEventKind::AgentStartAccepted => (
            "AGENT",
            "START_ACCEPT",
            "ACCEPTED".into(),
            event.subject_id.clone(),
        ),
        FixtureEventKind::ToolStartAccepted => (
            "TOOL",
            "START_ACCEPT",
            "ACCEPTED".into(),
            event.subject_id.clone(),
        ),
        FixtureEventKind::OutcomeDetected(effect) => (
            "OUTCOME_PROBE",
            "DETECT",
            format!("{:?}", effect.effect_type),
            event.subject_id.clone(),
        ),
        FixtureEventKind::AcousticEos {
            turn_index,
            offset_micros,
        } => (
            "INTERACTION_FIXTURE",
            "GROUND_TRUTH_ACOUSTIC_EOS",
            format!("fixture_offset_micros={offset_micros}"),
            format!("turn-{turn_index}"),
        ),
    };
    FixtureEvent {
        run_id: inner.run_id.clone(),
        episode_id: inner.episode_id.clone(),
        sequence_number: event.sequence_number,
        monotonic_timestamp_nanos: event.monotonic_timestamp.0,
        fixture_kind: fixture_kind.into(),
        action: action.into(),
        subject_id,
        outcome,
        observable_effect: match &event.kind {
            FixtureEventKind::OutcomeDetected(effect) => Some(effect.clone()),
            _ => None,
        },
    }
}

#[derive(Debug)]
pub struct EpisodeExecution {
    pub evidence: RawEvidence,
    pub architecture_error: Option<String>,
    pub episode_elapsed_nanos: u64,
    pub attempted_event_count: u64,
    pub capture_append_cost_nanos: u64,
}

pub fn execute_episode(
    corpus: &bench_fixtures::pilot_assets::PilotCorpus,
    scenario: &RuntimeScenario,
    alternative: Alternative,
    run_id: &str,
    source_git_commit: &str,
    instrumentation_mode: InstrumentationMode,
    latency_profile: ControlledLatencyProfile,
) -> Result<EpisodeExecution, String> {
    let materialized = materialize_scenario(corpus, scenario).map_err(|error| error.to_string())?;
    let stimulus = ScenarioStimulus {
        initial_state: materialized.initial_state.clone(),
        turns: materialized.turns.clone(),
    };
    let plan = corpus
        .behavior_plans
        .plans
        .iter()
        .find(|plan| plan.scenario_id == scenario.scenario_id)
        .ok_or_else(|| "behavior plan missing".to_owned())?;
    let ports = PilotPorts::new(
        PilotPortsConfig {
            run_id,
            episode_id: &format!("{run_id}:episode"),
            alternative,
            source_git_commit,
            instrumentation_mode,
            latency_profile,
        },
        scenario,
        plan,
        materialized,
    );
    match alternative {
        Alternative::A => run_architecture(
            ThinVia::new(ports.clone(), ports.clone(), ports.clone()),
            ports,
            stimulus,
        ),
        Alternative::B => run_architecture(
            ArgoCentric::new(ports.clone(), ports.clone(), ports.clone()),
            ports,
            stimulus,
        ),
        Alternative::C => run_architecture(
            HybridVia::new(ports.clone(), ports.clone(), ports.clone()),
            ports,
            stimulus,
        ),
        Alternative::D => run_architecture(
            AdaptiveVia::new(ports.clone(), ports.clone(), ports.clone()),
            ports,
            stimulus,
        ),
    }
}

fn run_architecture<A>(
    mut architecture: A,
    ports: PilotPorts,
    stimulus: ScenarioStimulus,
) -> Result<EpisodeExecution, String>
where
    A: ArchitectureUnderTest<Error = String>,
{
    let started = Instant::now();
    let mut evidence_source = ports.clone();
    let mut lifecycle = ports.clone();
    let result = Runner.run(
        &mut architecture,
        &mut evidence_source,
        &mut lifecycle,
        stimulus,
    );
    let elapsed = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let evidence = match result {
        Ok(evidence) => evidence,
        Err(error) => {
            ports.record_terminal_failure(classify_failure(&format!("{error:?}")));
            let evidence = evidence_source.take_raw_evidence();
            let (attempted_event_count, capture_append_cost_nanos) = ports.capture_diagnostics();
            return Ok(EpisodeExecution {
                evidence,
                architecture_error: Some(format!("{error:?}")),
                episode_elapsed_nanos: elapsed,
                attempted_event_count,
                capture_append_cost_nanos,
            });
        }
    };
    let (attempted_event_count, capture_append_cost_nanos) = ports.capture_diagnostics();
    Ok(EpisodeExecution {
        evidence,
        architecture_error: None,
        episode_elapsed_nanos: elapsed,
        attempted_event_count,
        capture_append_cost_nanos,
    })
}

impl PilotPorts {
    fn record_terminal_failure(&self, reason: FailureOutcomeReason) {
        self.capture(
            EventEmitter::Benchmark,
            CanonicalEventKind::EpisodeFailed { reason },
        );
    }
}

fn model_semantic_output_reference(
    response: &ModelResponse,
) -> Option<ModelSemanticOutputReference> {
    let output = response.completed_output().ok()?;
    let value = if output.contains("MAIL_AGENT") {
        "MailAgent"
    } else if output.contains("NETWORK_AGENT") {
        "NetworkAgent"
    } else if output.contains("FILE_AGENT") {
        "FileAgent"
    } else if output.contains("ARGO") {
        "ARGO"
    } else {
        return None;
    };
    Some(ModelSemanticOutputReference {
        value_kind: ModelSemanticValueKind::ExecutorCandidate,
        value: value.into(),
    })
}

fn effect_from_execution_request(request: &ExecutionRequest) -> ObservableEffect {
    let action = request.semantic_action.as_str();
    let (effect_type, subject_id, value, state) = if action.contains("LocalVolume")
        || action.contains("LOCAL_VOLUME")
    {
        (
            ObservableEffectType::VolumeChanged,
            Some("system-volume".into()),
            Some("35".into()),
            Some("CHANGED".into()),
        )
    } else if action.contains("OpenRightDocument") || action.contains("OPEN_RIGHT_DOCUMENT") {
        (
            ObservableEffectType::DocumentOpened,
            Some("doc-right".into()),
            None,
            Some("OPEN_USABLE".into()),
        )
    } else if action.contains("ContinueTask") || action.contains("CONTINUE_T1") {
        (
            ObservableEffectType::DnsCheckObserved,
            Some(request.task_id.0.clone()),
            None,
            Some("OBSERVED".into()),
        )
    } else if action.contains("DiagnoseWifi") || action.contains("DIAGNOSE_WIFI") {
        (
            ObservableEffectType::WifiStatusObserved,
            Some("wifi_connection_1".into()),
            None,
            Some("OBSERVED".into()),
        )
    } else if action.contains("LatestDownloadLookup") || action.contains("LOOKUP_LATEST_DOWNLOAD") {
        (
            ObservableEffectType::FileInspected,
            Some("downloads".into()),
            None,
            Some("LATEST_FILE_NAME_EMITTED".into()),
        )
    } else if action.contains("OrganizeDownloads") || action.contains("ORGANIZE_DOWNLOADS") {
        (
            ObservableEffectType::DownloadsOrganized,
            Some("downloads".into()),
            None,
            Some("ORGANIZED".into()),
        )
    } else if action.contains("GeneralFileWork") || action.contains("GENERAL_FILE_WORK") {
        (
            ObservableEffectType::FileInspected,
            Some("general-file-work".into()),
            None,
            Some("RESULT_EMITTED".into()),
        )
    } else {
        (
            ObservableEffectType::DiagnosisStarted,
            Some(request.task_id.0.clone()),
            None,
            Some("STARTED".into()),
        )
    };
    ObservableEffect {
        effect_type,
        subject_id,
        target_id: None,
        value,
        state,
        executor_id: request
            .route
            .final_executor_id_if_known
            .as_ref()
            .or(Some(&request.route.initial_executor_id))
            .map(|value| value.0.clone()),
        authoritative_source: EventEmitter::OutcomeProbe,
    }
}

fn classify_failure(error: &str) -> FailureOutcomeReason {
    if error.contains("Malformed") {
        FailureOutcomeReason::ModelMalformed
    } else if error.contains("TimedOut") || error.to_ascii_lowercase().contains("timeout") {
        FailureOutcomeReason::ModelTimeout
    } else if error.contains("NoResponse") {
        FailureOutcomeReason::ModelNoResponse
    } else if error.contains("rejected route") || error.contains("dispatch reject") {
        FailureOutcomeReason::DispatchRejected
    } else if error.contains("executor error") {
        FailureOutcomeReason::ExecutionFailure
    } else {
        FailureOutcomeReason::InvalidRoute
    }
}
