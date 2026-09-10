#![forbid(unsafe_code)]

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use bench_events::InstrumentationMode;
use bench_fixtures::pilot_assets::load_pilot_corpus;
use bench_runner::pilot_runtime::execute_episode;
use bench_runner::{
    ASYNC_RUNTIME, Alternative, ControlledLatencyProfile, PilotRunnerConfig,
    RUNTIME_WORKER_POLICY_VERSION, RunMode, RunProvenance, build_episode_plan,
    build_qualification_runtime, command_output, file_identity, guard_run_mode,
    inspect_source_state, persist_raw_run,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("dp00-pilot-runner: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let config = parse_config(env::args().skip(1), &repository_root)?;
    let source = inspect_source_state(&repository_root).map_err(|error| error.to_string())?;
    guard_run_mode(config.run_mode, &source).map_err(str::to_owned)?;
    let runtime = build_qualification_runtime().map_err(|error| error.to_string())?;
    runtime.block_on(async { tokio::task::yield_now().await });
    let corpus = load_pilot_corpus(&repository_root).map_err(|error| error.to_string())?;
    let invocation_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let rustc =
        command_output(&repository_root, "rustc", &["-Vv"]).map_err(|error| error.to_string())?;
    let cargo =
        command_output(&repository_root, "cargo", &["-V"]).map_err(|error| error.to_string())?;
    let target = rustc
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .unwrap_or("UNKNOWN")
        .to_owned();
    let toolchain =
        std::fs::read_to_string(repository_root.join("prototypes/dp00/rust-toolchain.toml"))
            .map_err(|error| error.to_string())?;
    let cargo_lock_identity = file_identity(&repository_root.join("prototypes/dp00/Cargo.lock"))
        .map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&config.raw_output_root).map_err(|error| error.to_string())?;
    let plan_items = build_episode_plan(&config);
    let mut persisted = 0_u64;

    for (ordinal, item) in plan_items.into_iter().enumerate() {
        let scenario = corpus
            .scenarios
            .iter()
            .find(|scenario| scenario.scenario_id == item.scenario_id)
            .ok_or_else(|| format!("unknown scenario {}", item.scenario_id))?;
        let behavior_plan = corpus
            .behavior_plans
            .plans
            .iter()
            .find(|plan| plan.scenario_id == item.scenario_id)
            .ok_or_else(|| format!("missing behavior plan for {}", item.scenario_id))?;
        let mode_label = if config.run_mode == RunMode::Official {
            "official"
        } else {
            "dev"
        };
        let population_label = if item.measurement_population {
            "measured"
        } else {
            "warmup"
        };
        let run_id = format!(
            "{mode_label}-{population_label}-{invocation_id}-{}-{}-r{}-p{}",
            item.scenario_id,
            item.alternative.id(),
            item.repetition_index,
            item.sequence_position
        );
        let execution = execute_episode(
            &corpus,
            scenario,
            item.alternative,
            &run_id,
            &source.source_git_commit,
            config.instrumentation_mode,
            config.latency_profile.clone(),
        )?;
        if !item.measurement_population {
            continue;
        }
        let provenance = RunProvenance {
            provenance_schema_version: "dp00-pilot-provenance-v1".into(),
            run_id,
            official: config.run_mode == RunMode::Official,
            source_git_commit: source.source_git_commit.clone(),
            working_tree_clean: source.working_tree_clean,
            pilot_corpus_id: corpus.index.pilot_corpus_id.clone(),
            pilot_corpus_version: corpus.index.pilot_corpus_version.clone(),
            alternative: item.alternative,
            scenario_id: scenario.scenario_id.clone(),
            scenario_version: scenario.scenario_version.clone(),
            semantic_behavior_plan_id: behavior_plan.behavior_plan_id.clone(),
            semantic_behavior_plan_version: behavior_plan.behavior_plan_version.clone(),
            latency_profile: config.latency_profile.clone(),
            warmup_count: config.warmup_count,
            measured_repetition_count: config.measured_repetition_count,
            measurement_population: item.measurement_population,
            order_policy: config.order_policy.clone(),
            order_cycle: item.order_cycle,
            sequence_position: item.sequence_position,
            repetition_index: item.repetition_index,
            instrumentation_mode: config.instrumentation_mode,
            episode_elapsed_nanos: execution.episode_elapsed_nanos,
            event_count: execution.attempted_event_count,
            capture_append_cost_nanos: execution.capture_append_cost_nanos,
            model_profile: "dp00-base@v0".into(),
            prompt_profile: behavior_plan.payload_registry_version.clone(),
            cache_policy: "DISABLED".into(),
            rust_toolchain: toolchain.trim().into(),
            rustc_version: rustc.clone(),
            cargo_version: cargo.clone(),
            target: target.clone(),
            build_profile: if cfg!(debug_assertions) {
                "development"
            } else {
                "qualification-or-release"
            }
            .into(),
            tokio_resolved_version: ASYNC_RUNTIME.trim_start_matches("tokio-").into(),
            runtime_worker_policy: RUNTIME_WORKER_POLICY_VERSION.into(),
            cargo_lock_identity: cargo_lock_identity.clone(),
            os: env::consts::OS.into(),
            machine_architecture: env::consts::ARCH.into(),
            canonical_event_schema_version: scenario
                .provenance
                .canonical_event_schema_version
                .clone(),
            model_call_schema_version: "model-call-v0".into(),
        };
        let directory = persist_raw_run(&config.raw_output_root, &provenance, &execution.evidence)
            .map_err(|error| format!("raw persistence failed: {error:?}"))?;
        persisted += 1;
        println!(
            "persisted {} ({}/{}, architecture_error={})",
            directory.display(),
            ordinal + 1,
            persisted,
            execution.architecture_error.as_deref().unwrap_or("NONE")
        );
    }
    println!("development/official run complete: {persisted} measured raw directories");
    Ok(())
}

fn parse_config(
    arguments: impl Iterator<Item = String>,
    repository_root: &Path,
) -> Result<PilotRunnerConfig, String> {
    let mut corpus = "pilot-v0".to_owned();
    let mut alternatives = Alternative::ALL.to_vec();
    let mut scenario_ids: Vec<String> = (1..=10).map(|number| format!("P{number:02}")).collect();
    let mut latency_profile = ControlledLatencyProfile::profile_z();
    let mut model_delay_override = None;
    let mut agent_delay_override = None;
    let mut tool_delay_override = None;
    let mut warmup_count = 1;
    let mut repetitions = 1;
    let mut instrumentation_mode = InstrumentationMode::Capture;
    let mut raw_output_root = repository_root.join("results/raw/pilot-v0");
    let mut run_mode = RunMode::Development;
    let arguments = arguments.collect::<Vec<_>>();
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        let value = |index: &mut usize| -> Result<&str, String> {
            *index += 1;
            arguments
                .get(*index)
                .map(String::as_str)
                .ok_or_else(|| format!("{argument} requires a value"))
        };
        match argument.as_str() {
            "--corpus" => corpus = value(&mut index)?.into(),
            "--alternatives" => alternatives = parse_alternatives(value(&mut index)?)?,
            "--scenarios" => {
                scenario_ids = value(&mut index)?.split(',').map(str::to_owned).collect()
            }
            "--latency-profile" => {
                latency_profile = match value(&mut index)? {
                    "Z" | "z" => ControlledLatencyProfile::profile_z(),
                    "C" | "c" => ControlledLatencyProfile::profile_c(),
                    other => return Err(format!("unknown latency profile {other}")),
                }
            }
            "--model-delay-micros" => {
                model_delay_override = Some(parse_u64(value(&mut index)?, argument)?)
            }
            "--agent-delay-micros" => {
                agent_delay_override = Some(parse_u64(value(&mut index)?, argument)?)
            }
            "--tool-delay-micros" => {
                tool_delay_override = Some(parse_u64(value(&mut index)?, argument)?)
            }
            "--warmup" => warmup_count = parse_u32(value(&mut index)?, argument)?,
            "--repetitions" => repetitions = parse_u32(value(&mut index)?, argument)?,
            "--instrumentation" => {
                instrumentation_mode = match value(&mut index)? {
                    "capture" => InstrumentationMode::Capture,
                    "minimal" | "noop" => InstrumentationMode::Minimal,
                    other => return Err(format!("unknown instrumentation mode {other}")),
                }
            }
            "--output-root" => raw_output_root = PathBuf::from(value(&mut index)?),
            "--official" => run_mode = RunMode::Official,
            "--development" => run_mode = RunMode::Development,
            "--help" | "-h" => {
                return Err("usage: dp00-pilot-runner [--corpus pilot-v0] [--alternatives A,B,C,D] [--scenarios P01,...] [--latency-profile Z|C] [--model-delay-micros N] [--agent-delay-micros N] [--tool-delay-micros N] [--warmup N] [--repetitions N] [--instrumentation capture|minimal] [--output-root PATH] [--official|--development]".into());
            }
            other => return Err(format!("unknown option {other}")),
        }
        index += 1;
    }
    if corpus != "pilot-v0" {
        return Err("only the strict pilot-v0 corpus is supported".into());
    }
    if alternatives.is_empty() || scenario_ids.is_empty() || repetitions == 0 {
        return Err("alternatives, scenarios, and repetitions must be non-empty".into());
    }
    if let Some(delay) = model_delay_override {
        latency_profile.model_delay_micros = delay;
    }
    if let Some(delay) = agent_delay_override {
        latency_profile.agent_delay_micros = delay;
    }
    if let Some(delay) = tool_delay_override {
        latency_profile.tool_delay_micros = delay;
    }
    Ok(PilotRunnerConfig {
        pilot_corpus: corpus,
        alternatives,
        scenario_ids,
        latency_profile,
        warmup_count,
        measured_repetition_count: repetitions,
        order_policy: "DETERMINISTIC_CYCLIC_V1".into(),
        instrumentation_mode,
        raw_output_root,
        run_mode,
    })
}

fn parse_alternatives(value: &str) -> Result<Vec<Alternative>, String> {
    value
        .split(',')
        .map(|value| match value {
            "A" | "a" => Ok(Alternative::A),
            "B" | "b" => Ok(Alternative::B),
            "C" | "c" => Ok(Alternative::C),
            "D" | "d" => Ok(Alternative::D),
            other => Err(format!("unknown alternative {other}")),
        })
        .collect()
}

fn parse_u32(value: &str, option: &str) -> Result<u32, String> {
    value
        .parse()
        .map_err(|_| format!("{option} requires an unsigned integer"))
}

fn parse_u64(value: &str, option: &str) -> Result<u64, String> {
    value
        .parse()
        .map_err(|_| format!("{option} requires an unsigned integer"))
}
