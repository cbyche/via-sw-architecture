#![forbid(unsafe_code)]

use std::io::Write;
use std::sync::Mutex;

use bench_core::{
    ArchitectureEvent, ArchitectureObservation, Clock, DecisionOwner, MonotonicTimestamp,
    ObservationPort, ProductCorrelation, SemanticResponsibility,
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
    Architecture(ArchitectureEvent),
    ModelGenerationStarted,
    ModelGenerationCompleted,
    ExecutionStarted,
    UsefulOutcomeObserved,
    EpisodeCompleted,
    EpisodeFailed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicalModelCall {
    pub model_call_id: String,
    pub decision_owner: DecisionOwner,
    pub semantic_responsibilities: Vec<SemanticResponsibility>,
    pub logical_start: MonotonicTimestamp,
    pub completion: MonotonicTimestamp,
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
    observations: Vec<CapturedObservation>,
}

pub struct InMemoryObservationCollector<C> {
    context: ObservationContext,
    clock: C,
    buffer: Mutex<CaptureBuffer>,
}

impl<C: Clock> InMemoryObservationCollector<C> {
    #[must_use]
    pub fn with_capacity(context: ObservationContext, clock: C, capacity: usize) -> Self {
        Self {
            context,
            clock,
            buffer: Mutex::new(CaptureBuffer {
                next_sequence: 0,
                observations: Vec::with_capacity(capacity),
            }),
        }
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
        let mut buffer = self.buffer.lock().expect("event buffer poisoned");
        let sequence_number = buffer.next_sequence;
        buffer.next_sequence += 1;
        buffer.observations.push(CapturedObservation {
            sequence_number,
            monotonic_timestamp: timestamp,
            event,
            product_correlation,
            emitter,
        });
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
}
