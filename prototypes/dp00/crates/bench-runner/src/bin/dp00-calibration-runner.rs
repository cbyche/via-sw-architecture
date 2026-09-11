#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use bench_fixtures::pilot_assets::load_pilot_corpus;
use bench_runner::pilot_runtime::execute_episode;
use bench_runner::{
    ALTERNATIVE_ROTATION_POLICY_VERSION, ASYNC_RUNTIME, Alternative, CALIBRATION_MANIFEST_VERSION,
    CALIBRATION_PROTOCOL_VERSION, CALIBRATION_RUN_PROVENANCE_VERSION, CalibrationManifest,
    CalibrationPopulation, ControlledLatencyProfile, FIXED_REPETITION_POLICY_VERSION,
    FULL_PREWARM_CYCLES, MEASURED_CALIBRATION_CYCLES, MODE_ORDER_POLICY_VERSION,
    PrewarmCycleCompletion, PrewarmPathIdentity, RUNTIME_WORKER_POLICY_VERSION, RunProvenance,
    build_counterbalanced_calibration_schedule, build_qualification_runtime, command_output,
    file_identity, inspect_source_state, persist_calibration_manifest, persist_raw_run,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("dp00-calibration-runner: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let (calibration_id, output_root) = parse_args(env::args().skip(1))?;
    let source = inspect_source_state(&repository_root).map_err(|error| error.to_string())?;
    if !source.working_tree_clean {
        return Err("counterbalanced calibration requires a clean frozen source".into());
    }
    if output_root.exists() {
        return Err(format!(
            "calibration output already exists: {}",
            output_root.display()
        ));
    }
    std::fs::create_dir_all(&output_root).map_err(|error| error.to_string())?;

    let runtime = build_qualification_runtime().map_err(|error| error.to_string())?;
    runtime.block_on(async { tokio::task::yield_now().await });
    let corpus = load_pilot_corpus(&repository_root).map_err(|error| error.to_string())?;
    let scenario_ids = (1..=12)
        .map(|number| format!("P{number:02}"))
        .collect::<Vec<_>>();
    let alternatives = Alternative::ALL;
    let invocation_warmup_count = 1;
    let schedule = build_counterbalanced_calibration_schedule(
        &calibration_id,
        &scenario_ids,
        &alternatives,
        invocation_warmup_count,
    );
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
    let latency_profile = ControlledLatencyProfile::profile_z();
    let mut prewarm_completed: BTreeMap<String, Vec<PrewarmPathIdentity>> = BTreeMap::new();
    let mut measured_run_ids = Vec::new();

    for item in &schedule {
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
        let population = match item.population {
            CalibrationPopulation::FullPrewarm => "full-prewarm",
            CalibrationPopulation::InvocationWarmup => "invocation-warmup",
            CalibrationPopulation::Measured => "measured",
        };
        let mode = match item.instrumentation_mode {
            bench_events::InstrumentationMode::Capture => "capture",
            bench_events::InstrumentationMode::Minimal => "minimal",
        };
        let ordinal = item
            .measured_execution_ordinal
            .map_or_else(|| "discard".into(), |value| value.to_string());
        let run_id = format!(
            "calibration-{invocation_id}-{population}-{}-{}-{}-{mode}-{ordinal}",
            item.cycle_id,
            item.scenario_id,
            item.alternative.id(),
        );
        let execution = execute_episode(
            &corpus,
            scenario,
            item.alternative,
            &run_id,
            &source.source_git_commit,
            item.instrumentation_mode,
            latency_profile.clone(),
        )?;

        match item.population {
            CalibrationPopulation::FullPrewarm => {
                prewarm_completed
                    .entry(item.cycle_id.clone())
                    .or_default()
                    .push(PrewarmPathIdentity {
                        scenario_id: item.scenario_id.clone(),
                        alternative: item.alternative.id().into(),
                        instrumentation_mode: mode.to_ascii_uppercase(),
                    });
            }
            CalibrationPopulation::InvocationWarmup => {}
            CalibrationPopulation::Measured => {
                let provenance = RunProvenance {
                    provenance_schema_version: CALIBRATION_RUN_PROVENANCE_VERSION.into(),
                    run_id: run_id.clone(),
                    campaign_id: None,
                    campaign_profile_sequence_index: None,
                    official: false,
                    source_git_commit: source.source_git_commit.clone(),
                    source_sha: source.source_git_commit.clone(),
                    working_tree_clean: source.working_tree_clean,
                    pilot_corpus_id: corpus.index.pilot_corpus_id.clone(),
                    pilot_corpus_version: corpus.index.pilot_corpus_version.clone(),
                    alternative: item.alternative,
                    scenario_id: scenario.scenario_id.clone(),
                    scenario_version: scenario.scenario_version.clone(),
                    semantic_behavior_plan_id: behavior_plan.behavior_plan_id.clone(),
                    semantic_behavior_plan_version: behavior_plan.behavior_plan_version.clone(),
                    latency_profile: latency_profile.clone(),
                    warmup_count: invocation_warmup_count,
                    measured_repetition_count: MEASURED_CALIBRATION_CYCLES,
                    measurement_population: true,
                    order_policy: ALTERNATIVE_ROTATION_POLICY_VERSION.into(),
                    order_cycle: item.cycle_index,
                    sequence_position: item.order_slot,
                    repetition_index: item.cycle_index,
                    instrumentation_mode: item.instrumentation_mode,
                    calibration_id: Some(calibration_id.clone()),
                    cycle_id: Some(item.cycle_id.clone()),
                    pair_id: Some(item.pair_id.clone()),
                    order_slot: Some(item.order_slot),
                    mode_order_slot: Some(item.mode_order_slot),
                    repetition_id: Some(item.repetition_id.clone()),
                    calibration_protocol_version: Some(CALIBRATION_PROTOCOL_VERSION.into()),
                    calibration_execution_ordinal: item.measured_execution_ordinal,
                    episode_elapsed_nanos: execution.episode_elapsed_nanos,
                    event_count: u64::try_from(execution.evidence.events.len()).unwrap_or(u64::MAX),
                    attempted_event_count: execution.attempted_event_count,
                    measurement_spine_event_count: execution.measurement_spine_event_count,
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
                    model_call_schema_version: "model-call-v1".into(),
                };
                persist_raw_run(&output_root, &provenance, &execution.evidence)
                    .map_err(|error| format!("raw persistence failed: {error:?}"))?;
                measured_run_ids.push(run_id);
            }
        }
    }

    let expected_paths_per_prewarm =
        u32::try_from(scenario_ids.len() * alternatives.len() * 2).unwrap_or(u32::MAX);
    let prewarm_cycles = (0..FULL_PREWARM_CYCLES)
        .map(|cycle| {
            let cycle_id = format!("W{cycle}");
            let covered_paths = prewarm_completed.remove(&cycle_id).unwrap_or_default();
            let completed_path_count = u32::try_from(covered_paths.len()).unwrap_or(u32::MAX);
            PrewarmCycleCompletion {
                cycle_id,
                completed: completed_path_count == expected_paths_per_prewarm,
                expected_path_count: expected_paths_per_prewarm,
                completed_path_count,
                covered_paths,
            }
        })
        .collect::<Vec<_>>();
    let completed_pairs = u32::try_from(measured_run_ids.len() / 2).unwrap_or(u32::MAX);
    let manifest = CalibrationManifest {
        provenance_schema_version: CALIBRATION_MANIFEST_VERSION.into(),
        calibration_protocol_version: CALIBRATION_PROTOCOL_VERSION.into(),
        calibration_id,
        source_sha: source.source_git_commit,
        full_prewarm_cycle_count: FULL_PREWARM_CYCLES,
        completed_full_prewarm_cycle_count: u32::try_from(
            prewarm_cycles
                .iter()
                .filter(|cycle| cycle.completed)
                .count(),
        )
        .unwrap_or(u32::MAX),
        prewarm_cycles,
        invocation_warmup_count,
        measured_cycle_count: MEASURED_CALIBRATION_CYCLES,
        mode_order_policy: MODE_ORDER_POLICY_VERSION.into(),
        alternative_rotation_policy: ALTERNATIVE_ROTATION_POLICY_VERSION.into(),
        repetition_policy: FIXED_REPETITION_POLICY_VERSION.into(),
        adaptive_stopping: false,
        expected_measured_execution_count: 1_536,
        completed_measured_execution_count: u32::try_from(measured_run_ids.len())
            .unwrap_or(u32::MAX),
        expected_pair_count: 768,
        completed_pair_count: completed_pairs,
        expected_qa01_pair_count: 192,
        measured_run_ids,
    };
    persist_calibration_manifest(&output_root, &manifest)
        .map_err(|error| format!("manifest persistence failed: {error:?}"))?;
    println!(
        "counterbalanced calibration complete: {}",
        output_root.display()
    );
    Ok(())
}

fn parse_args(arguments: impl Iterator<Item = String>) -> Result<(String, PathBuf), String> {
    let arguments = arguments.collect::<Vec<_>>();
    let mut calibration_id = None;
    let mut output_root = None;
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        index += 1;
        if matches!(argument.as_str(), "--help" | "-h") {
            return Err(
                "usage: dp00-calibration-runner --calibration-id ID --output-root PATH".into(),
            );
        }
        let value = arguments
            .get(index)
            .ok_or_else(|| format!("{argument} requires a value"))?;
        match argument.as_str() {
            "--calibration-id" => calibration_id = Some(value.clone()),
            "--output-root" => output_root = Some(PathBuf::from(value)),
            other => return Err(format!("unknown option {other}")),
        }
        index += 1;
    }
    let calibration_id = calibration_id.ok_or("--calibration-id is required")?;
    if calibration_id.is_empty() {
        return Err("--calibration-id must not be empty".into());
    }
    let output_root = output_root.ok_or("--output-root is required")?;
    Ok((calibration_id, output_root))
}
