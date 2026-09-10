#![forbid(unsafe_code)]

use std::io::{BufRead, Write};
use std::sync::Mutex;
use std::time::Instant;

use bench_core::{
    ArchitectureEvent, ArchitectureObservation, Clock, DecisionOwner, ExecutionRoute, ModelStatus,
    MonotonicTimestamp, ObservationPort, ProductCorrelation, ReferentRole, SemanticResponsibility,
    TaskRelation, TurnId,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationContext {
    pub run_id: String,
    pub episode_id: String,
    pub scenario_id: String,
    pub scenario_version: String,
    pub alternative_id: String,
    pub benchmark_version: String,
    pub schema_version: String,
    pub source_git_commit: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventEmitter {
    Benchmark,
    ArchitectureUnderTest,
    ModelFixture,
    AgentFixture,
    ToolFixture,
    OutcomeProbe,
    InteractionFixture,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalEventKind {
    AcousticEos,
    Architecture(ArchitectureEvent),
    ModelGenerationStarted,
    ModelGenerationCompleted,
    ExecutionStarted,
    UsefulOutcomeObserved { effect: ObservableEffect },
    EpisodeCompleted,
    EpisodeFailed { reason: FailureOutcomeReason },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ObservableEffectType {
    VolumeChanged,
    FileInspected,
    WifiStatusObserved,
    DocumentOpened,
    DnsCheckObserved,
    DownloadsOrganized,
    DiagnosisStarted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservableEffect {
    pub effect_type: ObservableEffectType,
    pub subject_id: Option<String>,
    pub target_id: Option<String>,
    pub value: Option<String>,
    pub state: Option<String>,
    pub executor_id: Option<String>,
    pub authoritative_source: EventEmitter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FailureOutcomeReason {
    ModelMalformed,
    ModelTimeout,
    ModelNoResponse,
    InvalidRoute,
    DispatchRejected,
    ExecutionFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalEvent {
    schema_version: String,
    event_id: String,
    run_id: String,
    episode_id: String,
    scenario_id: String,
    scenario_version: String,
    alternative_id: String,
    benchmark_version: String,
    source_git_commit: String,
    sequence_number: u64,
    monotonic_timestamp: MonotonicTimestamp,
    emitter: EventEmitter,
    product_correlation: ProductCorrelation,
    event: CanonicalEventKind,
}

impl CanonicalEvent {
    #[must_use]
    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    #[must_use]
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    #[must_use]
    pub fn monotonic_timestamp(&self) -> MonotonicTimestamp {
        self.monotonic_timestamp
    }

    #[must_use]
    pub fn emitter(&self) -> EventEmitter {
        self.emitter
    }

    #[must_use]
    pub fn event(&self) -> &CanonicalEventKind {
        &self.event
    }

    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    #[must_use]
    pub fn scenario_id(&self) -> &str {
        &self.scenario_id
    }

    #[must_use]
    pub fn alternative_id(&self) -> &str {
        &self.alternative_id
    }

    #[must_use]
    pub fn product_correlation(&self) -> &ProductCorrelation {
        &self.product_correlation
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContractViolation {
    MissingProcessingStart,
    DuplicateInitialRouteCommit,
    PrematureRouteCommit,
    CompletedWithoutRouteCommit,
    CompletedWithoutExecution,
    CompletedWithoutAuthoritativeOutcome,
    OutcomeNotProbeOwned,
    OutcomeAuthorityMismatch,
    FailedWithUsefulOutcome,
    ConflictingTerminalEvents,
    InvalidSuccessOrder,
    InvalidClarificationOrder,
}

/// Checks topology-neutral canonical event invariants. Component-internal event
/// order is intentionally outside this contract.
#[must_use]
pub fn validate_episode(events: &[CanonicalEvent]) -> Vec<ContractViolation> {
    let mut violations = Vec::new();
    let position = |predicate: fn(&CanonicalEventKind) -> bool| {
        events.iter().position(|event| predicate(event.event()))
    };
    let processing = position(|event| {
        matches!(
            event,
            CanonicalEventKind::Architecture(ArchitectureEvent::ProcessingStarted)
        )
    });
    if processing.is_none() {
        violations.push(ContractViolation::MissingProcessingStart);
    }

    let commits: Vec<_> = events
        .iter()
        .enumerate()
        .filter(|(_, event)| {
            matches!(
                event.event(),
                CanonicalEventKind::Architecture(ArchitectureEvent::RouteCommitted { .. })
            )
        })
        .collect();
    if commits.len() > 1 {
        violations.push(ContractViolation::DuplicateInitialRouteCommit);
    }
    if let Some((commit_position, _)) = commits.first() {
        let last_candidate_was_rejected =
            events[..*commit_position]
                .iter()
                .rev()
                .find_map(|event| match event.event() {
                    CanonicalEventKind::Architecture(
                        ArchitectureEvent::RouteCandidateRejected { .. },
                    ) => Some(true),
                    CanonicalEventKind::Architecture(
                        ArchitectureEvent::RouteCandidateObserved { .. },
                    ) => Some(false),
                    _ => None,
                });
        if processing.is_some_and(|start| start >= *commit_position)
            || last_candidate_was_rejected == Some(true)
        {
            violations.push(ContractViolation::PrematureRouteCommit);
        }
    }

    let completed = position(|event| matches!(event, CanonicalEventKind::EpisodeCompleted));
    let failed = position(|event| matches!(event, CanonicalEventKind::EpisodeFailed { .. }));
    let execution = position(|event| matches!(event, CanonicalEventKind::ExecutionStarted));
    let authoritative_outcome = events.iter().position(|event| {
        matches!(
            event.event(),
            CanonicalEventKind::UsefulOutcomeObserved { .. }
        ) && event.emitter() == EventEmitter::OutcomeProbe
    });
    if completed.is_some() && failed.is_some() {
        violations.push(ContractViolation::ConflictingTerminalEvents);
    }
    if completed.is_some() {
        if commits.len() != 1 {
            violations.push(ContractViolation::CompletedWithoutRouteCommit);
        }
        if execution.is_none() {
            violations.push(ContractViolation::CompletedWithoutExecution);
        }
        if authoritative_outcome.is_none() {
            violations.push(ContractViolation::CompletedWithoutAuthoritativeOutcome);
        }
        let model_positions: Vec<_> = events
            .iter()
            .enumerate()
            .filter_map(|(position, event)| {
                matches!(event.event(), CanonicalEventKind::ModelGenerationStarted)
                    .then_some(position)
            })
            .collect();
        let valid_order = processing
            .zip(commits.first().map(|(position, _)| *position))
            .zip(execution)
            .zip(authoritative_outcome)
            .zip(completed)
            .is_some_and(
                |((((processing, commit), execution), outcome), completed)| {
                    processing < commit
                        && commit < execution
                        && execution < outcome
                        && outcome < completed
                        && model_positions
                            .iter()
                            .all(|model| processing < *model && *model < commit)
                },
            );
        if !valid_order {
            violations.push(ContractViolation::InvalidSuccessOrder);
        }
    }
    if failed.is_some()
        && events.iter().any(|event| {
            matches!(
                event.event(),
                CanonicalEventKind::UsefulOutcomeObserved { .. }
            )
        })
    {
        violations.push(ContractViolation::FailedWithUsefulOutcome);
    }
    if events.iter().any(|event| {
        matches!(
            event.event(),
            CanonicalEventKind::UsefulOutcomeObserved { .. }
        ) && event.emitter() != EventEmitter::OutcomeProbe
    }) {
        violations.push(ContractViolation::OutcomeNotProbeOwned);
    }
    if events.iter().any(|event| {
        matches!(
            event.event(),
            CanonicalEventKind::UsefulOutcomeObserved { effect }
                if effect.authoritative_source != event.emitter()
        )
    }) {
        violations.push(ContractViolation::OutcomeAuthorityMismatch);
    }

    let requested = position(|event| {
        matches!(
            event,
            CanonicalEventKind::Architecture(ArchitectureEvent::ClarificationRequested { .. })
        )
    });
    let resolved = position(|event| {
        matches!(
            event,
            CanonicalEventKind::Architecture(ArchitectureEvent::ClarificationResolved { .. })
        )
    });
    if resolved.is_some_and(|resolved| requested.is_none_or(|requested| requested >= resolved)) {
        violations.push(ContractViolation::InvalidClarificationOrder);
    }
    violations
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicalModelCall {
    pub schema_version: String,
    pub model_call_id: String,
    pub run_id: String,
    pub episode_id: String,
    pub scenario_id: String,
    pub alternative_id: String,
    pub logical_sequence: u64,
    pub attempt: u32,
    pub decision_owner: DecisionOwner,
    pub semantic_responsibilities: Vec<SemanticResponsibility>,
    pub status: ModelStatus,
    pub semantic_output_reference: Option<ModelSemanticOutputReference>,
    pub route_committed_before_call: bool,
    pub route_committed_after_call: bool,
    pub logical_start: MonotonicTimestamp,
    pub first_output: Option<MonotonicTimestamp>,
    pub completion: MonotonicTimestamp,
    pub failure: Option<MonotonicTimestamp>,
    pub call_class: String,
    pub classification_reason: String,
    pub qa04_primary_included: bool,
    pub route_commit_event_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelSemanticValueKind {
    ExecutorCandidate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelSemanticOutputReference {
    pub value_kind: ModelSemanticValueKind,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActualReferentBinding {
    pub referent_role: ReferentRole,
    pub resolved_referent_id: String,
    pub turn_id: Option<TurnId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActualTaskAssociation {
    pub task_id: Option<String>,
    pub task_relation: TaskRelation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActualClarificationAction {
    pub requested: bool,
    pub resolved: bool,
    pub request_turn_id: Option<TurnId>,
    pub response_turn_id: Option<TurnId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActualResultBinding {
    pub result_id: Option<String>,
    pub task_id: Option<String>,
    pub execution_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ActualSemanticTrace {
    pub referent_bindings: Vec<ActualReferentBinding>,
    pub task_associations: Vec<ActualTaskAssociation>,
    pub committed_routes: Vec<ExecutionRoute>,
    pub clarification_actions: Vec<ActualClarificationAction>,
    pub result_bindings: Vec<ActualResultBinding>,
    pub observable_effects: Vec<ObservableEffect>,
    pub failure_outcomes: Vec<FailureOutcomeReason>,
}

/// Projects only observed raw values. It never accepts scenario or oracle data.
#[must_use]
pub fn project_actual_semantic_trace(events: &[CanonicalEvent]) -> ActualSemanticTrace {
    let mut trace = ActualSemanticTrace::default();
    let mut pending_clarifications: Vec<ActualClarificationAction> = Vec::new();
    for event in events {
        match event.event() {
            CanonicalEventKind::Architecture(ArchitectureEvent::ReferentBound {
                referent_role,
                resolved_referent_id,
            }) if event.emitter() == EventEmitter::ArchitectureUnderTest => {
                trace.referent_bindings.push(ActualReferentBinding {
                    referent_role: *referent_role,
                    resolved_referent_id: resolved_referent_id.clone(),
                    turn_id: event.product_correlation().turn_id.clone(),
                });
            }
            CanonicalEventKind::Architecture(ArchitectureEvent::TaskAssociated {
                task_relation,
            }) if event.emitter() == EventEmitter::ArchitectureUnderTest => {
                trace.task_associations.push(ActualTaskAssociation {
                    task_id: event
                        .product_correlation()
                        .task_id
                        .as_ref()
                        .map(|value| value.0.clone()),
                    task_relation: *task_relation,
                });
            }
            CanonicalEventKind::Architecture(ArchitectureEvent::RouteCommitted { route })
                if event.emitter() == EventEmitter::ArchitectureUnderTest =>
            {
                trace.committed_routes.push(route.clone());
            }
            CanonicalEventKind::Architecture(ArchitectureEvent::ClarificationRequested {
                ..
            }) if event.emitter() == EventEmitter::ArchitectureUnderTest => {
                pending_clarifications.push(ActualClarificationAction {
                    requested: true,
                    resolved: false,
                    request_turn_id: event.product_correlation().turn_id.clone(),
                    response_turn_id: None,
                });
            }
            CanonicalEventKind::Architecture(ArchitectureEvent::ClarificationResolved {
                request_turn_id,
                response_turn_id,
            }) if event.emitter() == EventEmitter::ArchitectureUnderTest => {
                if let Some(action) = pending_clarifications
                    .iter_mut()
                    .rev()
                    .find(|action| !action.resolved)
                {
                    action.resolved = true;
                    action.request_turn_id = Some(request_turn_id.clone());
                    action.response_turn_id = Some(response_turn_id.clone());
                }
            }
            CanonicalEventKind::Architecture(ArchitectureEvent::ResultBound)
                if event.emitter() == EventEmitter::ArchitectureUnderTest =>
            {
                let correlation = event.product_correlation();
                trace.result_bindings.push(ActualResultBinding {
                    result_id: correlation.result_id.as_ref().map(|value| value.0.clone()),
                    task_id: correlation.task_id.as_ref().map(|value| value.0.clone()),
                    execution_id: correlation
                        .execution_id
                        .as_ref()
                        .map(|value| value.0.clone()),
                });
            }
            CanonicalEventKind::UsefulOutcomeObserved { effect }
                if event.emitter() == EventEmitter::OutcomeProbe
                    && effect.authoritative_source == EventEmitter::OutcomeProbe =>
            {
                trace.observable_effects.push(effect.clone());
            }
            CanonicalEventKind::EpisodeFailed { reason }
                if event.emitter() == EventEmitter::Benchmark =>
            {
                trace.failure_outcomes.push(*reason);
            }
            _ => {}
        }
    }
    trace.clarification_actions = pending_clarifications;
    trace
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapturedObservation {
    sequence_number: u64,
    monotonic_timestamp: MonotonicTimestamp,
    event: CanonicalEventKind,
    product_correlation: ProductCorrelation,
    emitter: EventEmitter,
}

#[derive(Debug, Default)]
struct CaptureBuffer {
    next_sequence: u64,
    attempted_events: u64,
    append_cost_nanos: u64,
    observations: Vec<CapturedObservation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstrumentationMode {
    Capture,
    Minimal,
}

pub struct InMemoryObservationCollector<C> {
    context: ObservationContext,
    clock: C,
    mode: InstrumentationMode,
    buffer: Mutex<CaptureBuffer>,
}

impl<C: Clock> InMemoryObservationCollector<C> {
    #[must_use]
    pub fn with_capacity(context: ObservationContext, clock: C, capacity: usize) -> Self {
        Self::with_mode(context, clock, capacity, InstrumentationMode::Capture)
    }

    #[must_use]
    pub fn with_mode(
        context: ObservationContext,
        clock: C,
        capacity: usize,
        mode: InstrumentationMode,
    ) -> Self {
        Self {
            context,
            clock,
            mode,
            buffer: Mutex::new(CaptureBuffer {
                next_sequence: 0,
                attempted_events: 0,
                append_cost_nanos: 0,
                observations: Vec::with_capacity(capacity),
            }),
        }
    }

    #[must_use]
    pub fn instrumentation_mode(&self) -> InstrumentationMode {
        self.mode
    }

    #[must_use]
    pub fn capture_diagnostics(&self) -> (u64, u64) {
        let buffer = self.buffer.lock().expect("event buffer poisoned");
        (buffer.attempted_events, buffer.append_cost_nanos)
    }

    #[must_use]
    pub fn captured_snapshot(&self) -> Vec<CapturedObservation> {
        self.buffer
            .lock()
            .expect("event buffer poisoned")
            .observations
            .clone()
    }

    /// Enriches compact captures after the timed interval has closed.
    #[must_use]
    pub fn canonical_snapshot(&self) -> Vec<CanonicalEvent> {
        self.captured_snapshot()
            .into_iter()
            .map(|captured| CanonicalEvent {
                schema_version: self.context.schema_version.clone(),
                event_id: format!("{}:{}", self.context.run_id, captured.sequence_number),
                run_id: self.context.run_id.clone(),
                episode_id: self.context.episode_id.clone(),
                scenario_id: self.context.scenario_id.clone(),
                scenario_version: self.context.scenario_version.clone(),
                alternative_id: self.context.alternative_id.clone(),
                benchmark_version: self.context.benchmark_version.clone(),
                source_git_commit: self.context.source_git_commit.clone(),
                sequence_number: captured.sequence_number,
                monotonic_timestamp: captured.monotonic_timestamp,
                emitter: captured.emitter,
                product_correlation: captured.product_correlation,
                event: captured.event,
            })
            .collect()
    }

    pub fn capture_benchmark_event(
        &self,
        emitter: EventEmitter,
        event: CanonicalEventKind,
        product_correlation: ProductCorrelation,
    ) {
        self.capture(emitter, event, product_correlation);
    }

    fn capture(
        &self,
        emitter: EventEmitter,
        event: CanonicalEventKind,
        product_correlation: ProductCorrelation,
    ) {
        let timestamp = self.clock.now();
        let append_started = Instant::now();
        let mut buffer = self.buffer.lock().expect("event buffer poisoned");
        buffer.attempted_events += 1;
        if self.mode == InstrumentationMode::Minimal {
            return;
        }
        let sequence_number = buffer.next_sequence;
        buffer.next_sequence += 1;
        buffer.observations.push(CapturedObservation {
            sequence_number,
            monotonic_timestamp: timestamp,
            event,
            product_correlation,
            emitter,
        });
        buffer.append_cost_nanos = buffer
            .append_cost_nanos
            .saturating_add(u64::try_from(append_started.elapsed().as_nanos()).unwrap_or(u64::MAX));
    }
}

impl<C: Clock> ObservationPort for InMemoryObservationCollector<C> {
    fn emit(&self, observation: ArchitectureObservation) {
        self.capture(
            EventEmitter::ArchitectureUnderTest,
            CanonicalEventKind::Architecture(observation.event),
            observation.product_correlation,
        );
    }
}

pub mod serialization {
    use super::*;

    pub fn write_jsonl<W: Write>(
        events: &[CanonicalEvent],
        mut writer: W,
    ) -> serde_json::Result<()> {
        for event in events {
            serde_json::to_writer(&mut writer, event)?;
            writer.write_all(b"\n").map_err(serde_json::Error::io)?;
        }
        Ok(())
    }

    pub fn read_jsonl<T: for<'de> Deserialize<'de>, R: BufRead>(
        reader: R,
    ) -> serde_json::Result<Vec<T>> {
        reader
            .lines()
            .filter_map(|line| match line {
                Ok(line) if line.trim().is_empty() => None,
                other => Some(other),
            })
            .map(|line| {
                let line = line.map_err(serde_json::Error::io)?;
                serde_json::from_str(&line)
            })
            .collect()
    }
}
