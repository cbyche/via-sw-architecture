#![forbid(unsafe_code)]

pub mod pilot_assets;

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use bench_core::{
    Clock, ExecutionId, ExecutionRoute, InitialProductState, MonotonicTimestamp, ResultId, TaskId,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionRequest {
    pub committed_route: ExecutionRoute,
    pub task_id: Option<TaskId>,
    pub input: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionOutcome {
    pub execution_id: ExecutionId,
    pub result_id: ResultId,
    pub payload: String,
}

pub trait AgentFixture: Send + Sync {
    type Error;

    /// Executes only after the architecture has selected a route.
    fn execute(&self, request: &ExecutionRequest) -> Result<ExecutionOutcome, Self::Error>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolInvocation {
    pub tool_id: String,
    pub arguments: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolOutcome {
    pub payload: String,
}

pub trait ToolFixture: Send + Sync {
    type Error;

    fn invoke(&self, invocation: &ToolInvocation) -> Result<ToolOutcome, Self::Error>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProbeObservation {
    pub probe_id: String,
    pub useful_outcome_observed: bool,
    pub observed_at: MonotonicTimestamp,
}

pub trait OutcomeProbe: Send + Sync {
    type Error;

    fn observe(&self) -> Result<ProbeObservation, Self::Error>;
}

pub trait StateSeeder<A> {
    type Error;

    /// Materializes logical facts before the timed episode begins.
    fn seed(&self, architecture: &mut A, state: &InitialProductState) -> Result<(), Self::Error>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatencyProfile {
    pub profile_id: String,
    pub version: String,
    pub model_delay_micros: u64,
    pub agent_delay_micros: u64,
    pub tool_delay_micros: u64,
}

pub trait LatencyController: Send + Sync {
    fn delay(&self, duration: Duration);
}

#[derive(Debug)]
pub struct TokioLatencyController;

impl TokioLatencyController {
    pub async fn delay_async(duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}

#[derive(Debug)]
pub struct SystemMonotonicClock {
    epoch: Instant,
}

impl Default for SystemMonotonicClock {
    fn default() -> Self {
        Self {
            epoch: Instant::now(),
        }
    }
}

impl Clock for SystemMonotonicClock {
    fn now(&self) -> MonotonicTimestamp {
        let nanos = self.epoch.elapsed().as_nanos();
        MonotonicTimestamp(u64::try_from(nanos).unwrap_or(u64::MAX))
    }
}

#[derive(Debug, Default)]
pub struct ControlledClock {
    nanos: AtomicU64,
}

impl ControlledClock {
    pub fn new(start_nanos: u64) -> Self {
        Self {
            nanos: AtomicU64::new(start_nanos),
        }
    }

    pub fn advance(&self, duration: Duration) {
        let delta = u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX);
        self.nanos.fetch_add(delta, Ordering::Relaxed);
    }
}

impl Clock for ControlledClock {
    fn now(&self) -> MonotonicTimestamp {
        MonotonicTimestamp(self.nanos.load(Ordering::Relaxed))
    }
}
