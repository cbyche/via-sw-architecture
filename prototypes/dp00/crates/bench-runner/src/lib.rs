#![forbid(unsafe_code)]

pub mod pilot_runtime;

use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use bench_core::{ArchitectureUnderTest, InitialProductState, UserTurn};
use bench_events::{
    CanonicalEvent, InstrumentationMode, LogicalModelCall, ObservableEffect, serialization,
};
use serde::{Deserialize, Serialize};

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
    pub fixture_events: Vec<FixtureEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureEvent {
    pub run_id: String,
    pub episode_id: String,
    pub sequence_number: u64,
    pub monotonic_timestamp_nanos: u64,
    pub fixture_kind: String,
    pub action: String,
    pub subject_id: String,
    pub outcome: String,
    pub observable_effect: Option<ObservableEffect>,
}

pub trait EvidenceSource {
    fn take_raw_evidence(&mut self) -> RawEvidence;
}

pub trait FixtureLifecycle {
    type Error;

    fn before_episode(&mut self) -> Result<(), Self::Error>;
    fn before_user_turn(&mut self, turn_index: usize) -> Result<(), Self::Error>;
    fn after_episode(&mut self) -> Result<(), Self::Error>;

    fn after_failure(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
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
        if let Err(error) = architecture.setup(stimulus.initial_state) {
            fixtures.after_failure().map_err(RunnerError::Fixture)?;
            return Err(RunnerError::Architecture(error));
        }

        for (turn_index, turn) in stimulus.turns.into_iter().enumerate() {
            fixtures
                .before_user_turn(turn_index)
                .map_err(RunnerError::Fixture)?;
            if let Err(error) = architecture.handle_user_turn(turn) {
                fixtures.after_failure().map_err(RunnerError::Fixture)?;
                return Err(RunnerError::Architecture(error));
            }
        }

        if let Err(error) = architecture.teardown() {
            fixtures.after_failure().map_err(RunnerError::Fixture)?;
            return Err(RunnerError::Architecture(error));
        }
        fixtures.after_episode().map_err(RunnerError::Fixture)?;
        Ok(evidence.take_raw_evidence())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Alternative {
    A,
    B,
    C,
    D,
}

impl Alternative {
    pub const ALL: [Self; 4] = [Self::A, Self::B, Self::C, Self::D];

    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RunMode {
    Development,
    Official,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlledLatencyProfile {
    pub profile_id: String,
    pub version: String,
    pub model_delay_micros: u64,
    pub agent_delay_micros: u64,
    pub tool_delay_micros: u64,
    pub calibration_status: String,
}

impl ControlledLatencyProfile {
    #[must_use]
    pub fn profile_z() -> Self {
        Self {
            profile_id: "Z".into(),
            version: "pilot-z-v0".into(),
            model_delay_micros: 0,
            agent_delay_micros: 0,
            tool_delay_micros: 0,
            calibration_status: "MINIMAL_DETERMINISTIC_BASELINE".into(),
        }
    }

    #[must_use]
    pub fn profile_c() -> Self {
        Self {
            profile_id: "C".into(),
            version: "pilot-c-provisional-v0".into(),
            model_delay_micros: 1_000,
            agent_delay_micros: 1_000,
            tool_delay_micros: 1_000,
            calibration_status: "PILOT_CALIBRATION_PARAMETER_NOT_PRODUCTION_OR_SCORING".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PilotRunnerConfig {
    pub pilot_corpus: String,
    pub alternatives: Vec<Alternative>,
    pub scenario_ids: Vec<String>,
    pub latency_profile: ControlledLatencyProfile,
    pub warmup_count: u32,
    pub measured_repetition_count: u32,
    pub order_policy: String,
    pub instrumentation_mode: InstrumentationMode,
    pub raw_output_root: PathBuf,
    pub run_mode: RunMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PilotCampaignConfig {
    pub runner: PilotRunnerConfig,
    pub profiles: Vec<ControlledLatencyProfile>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignConfiguration {
    pub alternatives: Vec<Alternative>,
    pub scenario_ids: Vec<String>,
    pub warmup_count: u32,
    pub measured_repetition_count: u32,
    pub order_policy: String,
    pub instrumentation_mode: InstrumentationMode,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignProvenance {
    pub provenance_schema_version: String,
    pub campaign_id: String,
    pub source_git_commit: String,
    pub initial_working_tree_clean: bool,
    pub pilot_corpus_id: String,
    pub pilot_corpus_version: String,
    pub profile_sequence: Vec<ControlledLatencyProfile>,
    pub campaign_configuration: CampaignConfiguration,
    pub runner_version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodePlan {
    pub alternative: Alternative,
    pub scenario_id: String,
    pub order_cycle: u32,
    pub sequence_position: u32,
    pub repetition_index: u32,
    pub measurement_population: bool,
}

#[must_use]
pub fn counterbalanced_order(alternatives: &[Alternative], cycle: u32) -> Vec<Alternative> {
    if alternatives.is_empty() {
        return Vec::new();
    }
    let offset = usize::try_from(cycle).unwrap_or(usize::MAX) % alternatives.len();
    alternatives
        .iter()
        .cycle()
        .skip(offset)
        .take(alternatives.len())
        .copied()
        .collect()
}

#[must_use]
pub fn build_episode_plan(config: &PilotRunnerConfig) -> Vec<EpisodePlan> {
    let mut plan = Vec::new();
    for (measurement_population, repetitions) in [
        (false, config.warmup_count),
        (true, config.measured_repetition_count),
    ] {
        for repetition_index in 0..repetitions {
            let order = counterbalanced_order(&config.alternatives, repetition_index);
            for scenario_id in &config.scenario_ids {
                for (sequence_position, alternative) in order.iter().enumerate() {
                    plan.push(EpisodePlan {
                        alternative: *alternative,
                        scenario_id: scenario_id.clone(),
                        order_cycle: repetition_index,
                        sequence_position: u32::try_from(sequence_position).unwrap_or(u32::MAX),
                        repetition_index,
                        measurement_population,
                    });
                }
            }
        }
    }
    plan
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunProvenance {
    pub provenance_schema_version: String,
    pub run_id: String,
    pub campaign_id: Option<String>,
    pub campaign_profile_sequence_index: Option<u32>,
    pub official: bool,
    pub source_git_commit: String,
    pub working_tree_clean: bool,
    pub pilot_corpus_id: String,
    pub pilot_corpus_version: String,
    pub alternative: Alternative,
    pub scenario_id: String,
    pub scenario_version: String,
    pub semantic_behavior_plan_id: String,
    pub semantic_behavior_plan_version: String,
    pub latency_profile: ControlledLatencyProfile,
    pub warmup_count: u32,
    pub measured_repetition_count: u32,
    pub measurement_population: bool,
    pub order_policy: String,
    pub order_cycle: u32,
    pub sequence_position: u32,
    pub repetition_index: u32,
    pub instrumentation_mode: InstrumentationMode,
    pub episode_elapsed_nanos: u64,
    pub event_count: u64,
    pub capture_append_cost_nanos: u64,
    pub model_profile: String,
    pub prompt_profile: String,
    pub cache_policy: String,
    pub rust_toolchain: String,
    pub rustc_version: String,
    pub cargo_version: String,
    pub target: String,
    pub build_profile: String,
    pub tokio_resolved_version: String,
    pub runtime_worker_policy: String,
    pub cargo_lock_identity: String,
    pub os: String,
    pub machine_architecture: String,
    pub canonical_event_schema_version: String,
    pub model_call_schema_version: String,
}

#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    Json(serde_json::Error),
    ExistingRun(PathBuf),
    ExistingCampaign(PathBuf),
}

pub fn begin_campaign(
    output_root: &Path,
    provenance: &CampaignProvenance,
) -> Result<PathBuf, PersistenceError> {
    let campaign_directory = output_root.join(&provenance.campaign_id);
    match fs::create_dir(&campaign_directory) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(PersistenceError::ExistingCampaign(campaign_directory));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir_all(output_root)?;
            fs::create_dir(&campaign_directory)?;
        }
        Err(error) => return Err(PersistenceError::Io(error)),
    }
    write_json_create_new(
        campaign_directory.join("campaign-provenance.json"),
        provenance,
    )?;
    Ok(campaign_directory)
}

pub fn begin_campaign_profile(
    campaign_directory: &Path,
    profile: &ControlledLatencyProfile,
) -> Result<PathBuf, PersistenceError> {
    let profile_directory = campaign_directory.join(format!(
        "profile-{}",
        profile.profile_id.to_ascii_lowercase()
    ));
    fs::create_dir(&profile_directory).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            PersistenceError::ExistingRun(profile_directory.clone())
        } else {
            PersistenceError::Io(error)
        }
    })?;
    Ok(profile_directory)
}

impl From<std::io::Error> for PersistenceError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

pub fn persist_raw_run(
    output_root: &Path,
    provenance: &RunProvenance,
    evidence: &RawEvidence,
) -> Result<PathBuf, PersistenceError> {
    let run_directory = output_root.join(&provenance.run_id);
    match fs::create_dir(&run_directory) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(PersistenceError::ExistingRun(run_directory));
        }
        Err(error) => return Err(PersistenceError::Io(error)),
    }
    write_json_create_new(run_directory.join("provenance.json"), provenance)?;
    write_jsonl_create_new(
        run_directory.join("canonical-events.jsonl"),
        &evidence.events,
    )?;
    write_jsonl_create_new(
        run_directory.join("model-calls.jsonl"),
        &evidence.model_calls,
    )?;
    if !evidence.fixture_events.is_empty() {
        write_jsonl_create_new(
            run_directory.join("fixture-events.jsonl"),
            &evidence.fixture_events,
        )?;
    }
    Ok(run_directory)
}

fn create_new(path: &Path) -> std::io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

fn write_json_create_new<T: Serialize>(path: PathBuf, value: &T) -> Result<(), PersistenceError> {
    let mut writer = BufWriter::new(create_new(&path)?);
    serde_json::to_writer_pretty(&mut writer, value)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

fn write_jsonl_create_new<T: Serialize>(
    path: PathBuf,
    values: &[T],
) -> Result<(), PersistenceError> {
    let mut writer = BufWriter::new(create_new(&path)?);
    for value in values {
        serde_json::to_writer(&mut writer, value)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;
    Ok(())
}

pub fn reload_events(path: &Path) -> Result<Vec<CanonicalEvent>, PersistenceError> {
    Ok(serialization::read_jsonl(BufReader::new(File::open(
        path,
    )?))?)
}

pub fn reload_model_calls(path: &Path) -> Result<Vec<LogicalModelCall>, PersistenceError> {
    Ok(serialization::read_jsonl(BufReader::new(File::open(
        path,
    )?))?)
}

pub fn reload_fixture_events(path: &Path) -> Result<Vec<FixtureEvent>, PersistenceError> {
    Ok(serialization::read_jsonl(BufReader::new(File::open(
        path,
    )?))?)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceState {
    pub source_git_commit: String,
    pub working_tree_clean: bool,
}

pub fn inspect_source_state(repository_root: &Path) -> std::io::Result<SourceState> {
    let head = command_output(repository_root, "git", &["rev-parse", "HEAD"])?;
    let status = command_output(repository_root, "git", &["status", "--porcelain"])?;
    Ok(SourceState {
        source_git_commit: head,
        working_tree_clean: status.is_empty(),
    })
}

pub fn guard_run_mode(mode: RunMode, state: &SourceState) -> Result<(), &'static str> {
    if mode == RunMode::Official && !state.working_tree_clean {
        Err("official Pilot run requires a clean working tree")
    } else {
        Ok(())
    }
}

pub fn command_output(root: &Path, program: &str, arguments: &[&str]) -> std::io::Result<String> {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "{program} exited with {}",
            output.status
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

pub fn file_identity(path: &Path) -> std::io::Result<String> {
    let bytes = fs::read(path)?;
    let hash = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    });
    Ok(format!("fnv1a64-v1:{hash:016x}"))
}
