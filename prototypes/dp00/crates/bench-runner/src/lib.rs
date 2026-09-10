#![forbid(unsafe_code)]

use bench_core::{ArchitectureUnderTest, InitialProductState, UserTurn};
use bench_events::{CanonicalEvent, LogicalModelCall};

pub const ASYNC_RUNTIME: &str = "tokio-1.53.1";
pub const RUNTIME_WORKER_POLICY_VERSION: &str = "tokio-current-thread-v0";

pub fn build_qualification_runtime() -> std::io::Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioStimulus {
    pub initial_state: InitialProductState,
    pub turns: Vec<UserTurn>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RawEvidence {
    pub events: Vec<CanonicalEvent>,
    pub model_calls: Vec<LogicalModelCall>,
}

pub trait EvidenceSource {
    fn take_raw_evidence(&mut self) -> RawEvidence;
}

pub trait FixtureLifecycle {
    type Error;

    fn before_episode(&mut self) -> Result<(), Self::Error>;
    fn before_user_turn(&mut self, turn_index: usize) -> Result<(), Self::Error>;
    fn after_episode(&mut self) -> Result<(), Self::Error>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum RunnerError<ArchitectureError, FixtureError> {
    Architecture(ArchitectureError),
    Fixture(FixtureError),
}

#[derive(Debug, Default)]
pub struct Runner;

impl Runner {
    pub fn run<A, E, F>(
        &self,
        architecture: &mut A,
        evidence: &mut E,
        fixtures: &mut F,
        stimulus: ScenarioStimulus,
    ) -> Result<RawEvidence, RunnerError<A::Error, F::Error>>
    where
        A: ArchitectureUnderTest,
        E: EvidenceSource,
        F: FixtureLifecycle,
    {
        fixtures.before_episode().map_err(RunnerError::Fixture)?;
        architecture
            .setup(stimulus.initial_state)
            .map_err(RunnerError::Architecture)?;

        for (turn_index, turn) in stimulus.turns.into_iter().enumerate() {
            fixtures
                .before_user_turn(turn_index)
                .map_err(RunnerError::Fixture)?;
            architecture
                .handle_user_turn(turn)
                .map_err(RunnerError::Architecture)?;
        }

        architecture.teardown().map_err(RunnerError::Architecture)?;
        fixtures.after_episode().map_err(RunnerError::Fixture)?;
        Ok(evidence.take_raw_evidence())
    }
}
