mod containment;
mod exec_recovery;
mod recovery;
mod whole_process_recovery;

use clap::{Parser, Subcommand};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout, Command as TokioCommand},
};
use via_contracts::{
    NativeReply, PReply, QReply, SubmitRequest, TaskCommand, TaskOp, WorkerRequest, WorkerResponse,
};
use via_fixture::{AgentShape, DeterministicAgent, S2sDelayTrace};
use via_runtime::{
    agent::{AgentBoundary, CoreVisibleTyped, EdgeNormalized},
    exec::{IntegrationBridge, LocalBridge, ProcessBridge},
    repository::Repository,
    task::{PerTaskSupervisors, SharedTaskService, TaskAuthority},
    workload::{BackgroundLoadConfig, run_background_load},
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Smoke,
    ExecSmoke {
        #[arg(long)]
        worker: PathBuf,
    },
    ExecAbortSmoke {
        #[arg(long)]
        worker: PathBuf,
    },
    ExecBlastSmoke {
        #[arg(long)]
        host: PathBuf,
        #[arg(long)]
        worker: PathBuf,
    },
    Dp11NormalDiagnostic {
        #[arg(long)]
        worker: PathBuf,
        #[arg(long, default_value_t = 200)]
        trials: usize,
        #[arg(long, default_value_t = 20)]
        warmup: usize,
    },
    S2sSmoke {
        #[arg(long)]
        trace: PathBuf,
    },
    BackgroundLoadSmoke,
    Qa09WholeRestartSmoke,
    Qa09IntegrationFatalSmoke {
        #[arg(long)]
        host: PathBuf,
        #[arg(long)]
        worker: PathBuf,
    },
    Qa09WholeProcessSmoke,
    Qa09IntegrationFatal {
        #[arg(long)]
        host: PathBuf,
        #[arg(long)]
        worker: PathBuf,
        #[arg(long, default_value = "smoke")]
        profile: String,
        #[arg(long)]
        freeze_fingerprint: Option<String>,
    },
    Qa09WholeProcess {
        #[arg(long, default_value = "smoke")]
        profile: String,
        #[arg(long)]
        freeze_fingerprint: Option<String>,
    },
    ContainmentDiagnostic {
        #[arg(long)]
        spec: PathBuf,
        #[arg(long)]
        host: PathBuf,
        #[arg(long)]
        worker: PathBuf,
        #[arg(long, default_value = "smoke")]
        profile: String,
        #[arg(long)]
        freeze_fingerprint: Option<String>,
    },
    Qa09RuntimeHost {
        #[arg(long)]
        task_candidate: String,
        #[arg(long)]
        db: PathBuf,
        #[arg(long)]
        agent_state: PathBuf,
        #[arg(long)]
        active_tasks: usize,
        #[arg(long)]
        completed: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Smoke => smoke().await?,
        Command::ExecSmoke { worker } => exec_smoke(worker).await?,
        Command::ExecAbortSmoke { worker } => exec_abort_smoke(worker).await?,
        Command::ExecBlastSmoke { host, worker } => exec_blast_smoke(host, worker).await?,
        Command::Dp11NormalDiagnostic {
            worker,
            trials,
            warmup,
        } => dp11_normal_diagnostic(worker, trials, warmup).await?,
        Command::S2sSmoke { trace } => s2s_smoke(trace).await?,
        Command::BackgroundLoadSmoke => background_load_smoke().await?,
        Command::Qa09WholeRestartSmoke => recovery::whole_restart_smoke().await?,
        Command::Qa09IntegrationFatalSmoke { host, worker } => {
            exec_recovery::integration_fatal_smoke(host, worker).await?
        }
        Command::Qa09WholeProcessSmoke => whole_process_recovery::whole_process_smoke().await?,
        Command::Qa09IntegrationFatal {
            host,
            worker,
            profile,
            freeze_fingerprint,
        } => {
            exec_recovery::integration_fatal(host, worker, &profile, freeze_fingerprint.as_deref())
                .await?
        }
        Command::Qa09WholeProcess {
            profile,
            freeze_fingerprint,
        } => whole_process_recovery::whole_process(&profile, freeze_fingerprint.as_deref()).await?,
        Command::ContainmentDiagnostic {
            spec,
            host,
            worker,
            profile,
            freeze_fingerprint,
        } => containment::run(spec, host, worker, profile, freeze_fingerprint).await?,
        Command::Qa09RuntimeHost {
            task_candidate,
            db,
            agent_state,
            active_tasks,
            completed,
        } => {
            whole_process_recovery::runtime_host(
                task_candidate,
                db,
                agent_state,
                active_tasks,
                completed,
            )
            .await?;
            return Ok(());
        }
    };
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

async fn smoke() -> anyhow::Result<serde_json::Value> {
    task_smoke().await?;
    agent_smoke().await?;
    Ok(serde_json::json!({
        "status":"PASS",
        "scope":"task+agent smoke",
        "benchmark":"NOT_RUN"
    }))
}

async fn task_smoke() -> anyhow::Result<()> {
    let a_file = tempfile::NamedTempFile::new()?;
    let b_file = tempfile::NamedTempFile::new()?;
    let a = SharedTaskService::new(Repository::open(a_file.path())?);
    let b = PerTaskSupervisors::new(Repository::open(b_file.path())?);

    for authority in [&a as &dyn TaskAuthority, &b as &dyn TaskAuthority] {
        let created = authority
            .apply(TaskCommand {
                command_id: "create".into(),
                task_id: "T1".into(),
                expected_revision: 0,
                op: TaskOp::Create {
                    goal: "demo".into(),
                },
            })
            .await?;
        let running = authority
            .apply(TaskCommand {
                command_id: "accept".into(),
                task_id: "T1".into(),
                expected_revision: created.revision,
                op: TaskOp::AcceptExecution {
                    run_id: "run-1".into(),
                },
            })
            .await?;
        anyhow::ensure!(matches!(running.state, via_contracts::TaskState::Running));
    }
    Ok(())
}

async fn agent_smoke() -> anyhow::Result<()> {
    let request = SubmitRequest {
        task_id: "T1".into(),
        submission_key: "K1".into(),
        goal: "demo".into(),
    };
    let edge = EdgeNormalized::new(DeterministicAgent::new(AgentShape::Q));
    let typed = CoreVisibleTyped::new(DeterministicAgent::new(AgentShape::Q));
    let a = edge.submit(request.clone()).await?;
    let b = typed.submit(request).await?;
    anyhow::ensure!(a.context_id.as_deref() == Some("ctx-T1"));
    anyhow::ensure!(b.context_id.as_deref() == Some("ctx-T1"));
    Ok(())
}

async fn exec_smoke(worker: PathBuf) -> anyhow::Result<serde_json::Value> {
    let local = LocalBridge::new(Arc::new(DeterministicAgent::new(AgentShape::Q)));
    let process = ProcessBridge::spawn(&worker).await?;
    let request = SubmitRequest {
        task_id: "T1".into(),
        submission_key: "K1".into(),
        goal: "demo".into(),
    };
    let local_reply = local.submit(request.clone()).await?;
    let process_reply = process.submit(request).await?;
    let local_run = run_id(&local_reply)?;
    let process_run = run_id(&process_reply)?;
    let _ = local.query(&local_run).await?;
    let _ = process.query(&process_run).await?;
    let local_events = local.events_since(&local_run, 0).await?;
    let process_events = process.events_since(&process_run, 0).await?;
    anyhow::ensure!(!local_events.is_empty() && !process_events.is_empty());

    let local_follow = local.follow_up(&local_run, "continue".into()).await?;
    let process_follow = process.follow_up(&process_run, "continue".into()).await?;
    anyhow::ensure!(reply_shape(&local_follow) == reply_shape(&process_follow));

    let local_cancel = local.cancel(&local_run).await?;
    let process_cancel = process.cancel(&process_run).await?;
    anyhow::ensure!(reply_shape(&local_cancel) == reply_shape(&process_cancel));

    Ok(serde_json::json!({
        "status":"PASS",
        "scope":"exec bridge full-lifecycle semantic transport smoke",
        "local_shape":reply_shape(&local_reply),
        "process_shape":reply_shape(&process_reply),
        "events_transported":true,
        "follow_up_transported":true,
        "cancel_transported":true,
        "benchmark":"NOT_RUN"
    }))
}

async fn exec_abort_smoke(worker: PathBuf) -> anyhow::Result<serde_json::Value> {
    let first = ProcessBridge::spawn(&worker).await?;
    let request = SubmitRequest {
        task_id: "T-abort".into(),
        submission_key: "K-abort-1".into(),
        goal: "before abort".into(),
    };
    let accepted = first.submit(request).await?;
    let run = run_id(&accepted)?;
    let _ = first.query(&run).await?;
    first.abort_host().await?;

    // Reaching this line proves the Core/controller process was not the worker host.
    // Spawn a fresh worker and prove the integration boundary is usable again.
    let restarted = ProcessBridge::spawn(&worker).await?;
    let after = restarted
        .submit(SubmitRequest {
            task_id: "T-after".into(),
            submission_key: "K-after-1".into(),
            goal: "after worker restart".into(),
        })
        .await?;
    Ok(serde_json::json!({
        "status":"PASS",
        "scope":"process-isolated worker fatal + restart correctness smoke",
        "first_run":run,
        "restart_reply_shape":reply_shape(&after),
        "parent_process_survived":true,
        "benchmark":"NOT_RUN"
    }))
}

fn elapsed_ns(start: std::time::Instant) -> u64 {
    u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

async fn dp11_bridge_trial(
    bridge: &dyn IntegrationBridge,
    candidate: &str,
    trial: usize,
    warmup: bool,
) -> anyhow::Result<serde_json::Value> {
    let request = SubmitRequest {
        task_id: format!("DP11-NORMAL-{candidate}-{trial}"),
        submission_key: format!("dp11-normal-{candidate}-{trial}"),
        goal: "summarize the synthetic project update".into(),
    };

    let lifecycle_start = std::time::Instant::now();
    let submit_start = std::time::Instant::now();
    let accepted = bridge.submit(request).await?;
    let submit_ns = elapsed_ns(submit_start);
    let run = run_id(&accepted)?;

    let query_start = std::time::Instant::now();
    let _ = bridge.query(&run).await?;
    let query_ns = elapsed_ns(query_start);

    let events_start = std::time::Instant::now();
    let events = bridge.events_since(&run, 0).await?;
    let events_ns = elapsed_ns(events_start);
    anyhow::ensure!(
        !events.is_empty(),
        "normal diagnostic lost initial Agent event"
    );

    let follow_up_start = std::time::Instant::now();
    let _ = bridge
        .follow_up(&run, "use the concise format".into())
        .await?;
    let follow_up_ns = elapsed_ns(follow_up_start);

    let cancel_start = std::time::Instant::now();
    let _ = bridge.cancel(&run).await?;
    let cancel_ns = elapsed_ns(cancel_start);

    Ok(serde_json::json!({
        "candidate":candidate,
        "trial":trial,
        "warmup":warmup,
        "submit_ns":submit_ns,
        "query_ns":query_ns,
        "events_since_ns":events_ns,
        "follow_up_ns":follow_up_ns,
        "cancel_ns":cancel_ns,
        "full_lifecycle_ns":elapsed_ns(lifecycle_start),
        "correctness_pass":true
    }))
}

/// Measures only the VIA-owned Agent-client bridge subspan. It is deliberately
/// diagnostic evidence, not QA-01/03/05 user-endpoint evidence.
async fn dp11_normal_diagnostic(
    worker: PathBuf,
    trials: usize,
    warmup: usize,
) -> anyhow::Result<serde_json::Value> {
    anyhow::ensure!(trials > 0, "DP-11 normal diagnostic requires scored trials");

    let local = LocalBridge::new(Arc::new(DeterministicAgent::new(AgentShape::Q)));
    let process = ProcessBridge::spawn(&worker).await?;
    let mut runs = Vec::with_capacity((trials + warmup) * 2);

    for candidate in ["same_process", "isolated_worker"] {
        let bridge: &dyn IntegrationBridge = if candidate == "same_process" {
            &local
        } else {
            &process
        };
        for index in 0..(warmup + trials) {
            runs.push(dp11_bridge_trial(bridge, candidate, index + 1, index < warmup).await?);
        }
    }

    Ok(serde_json::json!({
        "status":"PASS",
        "scope":"VIA-DP-11 normal Agent-client bridge subspan",
        "evidence_label":"MEASURED_REFERENCE_HARNESS",
        "metric_eligible":false,
        "reason":"This excludes the frozen user/acoustic endpoints required by QA-01, QA-03, and QA-05.",
        "trials_per_candidate":trials,
        "warmup_per_candidate":warmup,
        "runs":runs
    }))
}

struct HostSession {
    child: Child,
    stdin: ChildStdin,
    stdout: Lines<BufReader<ChildStdout>>,
}

impl HostSession {
    async fn spawn(host: &PathBuf, mode: &str, worker: Option<&PathBuf>) -> anyhow::Result<Self> {
        let mut command = TokioCommand::new(host);
        command
            .arg("--mode")
            .arg(mode)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .kill_on_drop(true);
        if let Some(worker) = worker {
            command.arg("--worker").arg(worker);
        }
        let mut child = command.spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("candidate host stdin missing"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("candidate host stdout missing"))?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout).lines(),
        })
    }

    async fn request(&mut self, request: WorkerRequest) -> anyhow::Result<Option<WorkerResponse>> {
        let value = self.request_value(serde_json::to_value(request)?).await?;
        value
            .map(serde_json::from_value)
            .transpose()
            .map_err(Into::into)
    }

    async fn request_value(
        &mut self,
        request: serde_json::Value,
    ) -> anyhow::Result<Option<serde_json::Value>> {
        self.stdin
            .write_all(serde_json::to_string(&request)?.as_bytes())
            .await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        match self.stdout.next_line().await? {
            Some(line) => Ok(Some(serde_json::from_str(&line)?)),
            None => Ok(None),
        }
    }
}

async fn probe_independent_core_units(
    session: &mut HostSession,
) -> anyhow::Result<Vec<serde_json::Value>> {
    let capabilities = [
        ("direct_voice_interaction", "CAP-S2S-DIRECT"),
        ("text_interaction", "CAP-TEXT-INTERACTION"),
        ("unrelated_task_query_control", "CAP-LOCAL-TASK-CARD-READ"),
        ("context_source_read", "CAP-BOUNDED-DOC-READ"),
    ];
    let mut observations = Vec::with_capacity(capabilities.len());
    for (unit, capability) in capabilities {
        let response = session
            .request_value(serde_json::json!({
                "op":"core_probe",
                "capability":capability
            }))
            .await?;
        let available = matches!(response, Some(ref value) if value["status"] == "core_probe");
        observations.push(serde_json::json!({
            "unit":unit,
            "capability":capability,
            "available":available,
            "response":response
        }));
    }
    Ok(observations)
}

async fn exec_blast_smoke(host: PathBuf, worker: PathBuf) -> anyhow::Result<serde_json::Value> {
    let mut shared = HostSession::spawn(&host, "shared", None).await?;
    anyhow::ensure!(
        matches!(
            shared.request(WorkerRequest::Ping).await?,
            Some(WorkerResponse::Pong)
        ),
        "shared host did not start"
    );
    let shared_abort_reply = shared.request(WorkerRequest::AbortHost).await?;
    anyhow::ensure!(
        shared_abort_reply.is_none(),
        "shared-process fatal fault unexpectedly returned a normal response"
    );
    let shared_status = shared.child.wait().await?;

    let mut isolated = HostSession::spawn(&host, "isolated", Some(&worker)).await?;
    anyhow::ensure!(
        matches!(
            isolated.request(WorkerRequest::Ping).await?,
            Some(WorkerResponse::Pong)
        ),
        "isolated host did not start"
    );
    anyhow::ensure!(
        matches!(
            isolated.request(WorkerRequest::AbortHost).await?,
            Some(WorkerResponse::Pong)
        ),
        "isolated host did not survive worker abort/restart"
    );
    anyhow::ensure!(
        matches!(
            isolated.request(WorkerRequest::Ping).await?,
            Some(WorkerResponse::Pong)
        ),
        "isolated host not usable after worker restart"
    );
    anyhow::ensure!(
        isolated.child.try_wait()?.is_none(),
        "isolated Core host exited after integration worker fatal fault"
    );
    let isolated_independent_unit_probes = probe_independent_core_units(&mut isolated).await?;
    anyhow::ensure!(
        isolated_independent_unit_probes
            .iter()
            .all(|probe| probe["available"] == true),
        "isolated Core lost an independent user-visible unit"
    );

    Ok(serde_json::json!({
        "status":"PASS",
        "scope":"symmetric EXEC fatal-fault blast-radius smoke",
        "shared_host_exited":true,
        "shared_exit_success":shared_status.success(),
        "isolated_core_host_survived":true,
        "isolated_worker_restarted":true,
        "shared_independent_unit_probes":"UNAVAILABLE_BECAUSE_CORE_PROCESS_EXITED",
        "isolated_independent_unit_probes":isolated_independent_unit_probes,
        "benchmark":"NOT_RUN"
    }))
}

async fn background_load_smoke() -> anyhow::Result<serde_json::Value> {
    let mut runs = Vec::new();
    for candidate in ["shared", "per_task"] {
        for active_tasks in [1usize, 4usize] {
            let file = tempfile::NamedTempFile::new()?;
            let repository = Repository::open(file.path())?;
            let authority: Arc<dyn TaskAuthority> = if candidate == "shared" {
                Arc::new(SharedTaskService::new(repository))
            } else {
                Arc::new(PerTaskSupervisors::new(repository))
            };
            let report = run_background_load(
                authority,
                BackgroundLoadConfig {
                    active_tasks,
                    update_period: Duration::from_millis(2),
                    rounds: 3,
                },
            )
            .await?;
            anyhow::ensure!(
                report.updates_applied == active_tasks * 3,
                "background workload smoke dropped background updates"
            );
            runs.push(serde_json::json!({
                "candidate": candidate,
                "active_tasks": active_tasks,
                "updates_applied": report.updates_applied,
                "elapsed_ms": report.elapsed_ms,
                "representative_metric_eligible": report.representative_metric_eligible
            }));
        }
    }

    Ok(serde_json::json!({
        "status":"PASS",
        "scope":"background Task load-shape smoke retained as a non-scoring workload diagnostic",
        "fixed_shape":"1 update per Task per cadence; production cadence remains 1000ms up to 30s",
        "runs":runs,
        "representative_metric":"NOT_RUN",
        "note":"Active QA workload level and scored foreground probes are not yet frozen.",
        "benchmark":"NOT_RUN"
    }))
}

async fn s2s_smoke(trace: PathBuf) -> anyhow::Result<serde_json::Value> {
    let trace = S2sDelayTrace::from_path(trace)?;
    let delay = trace.replay(0).await;
    Ok(serde_json::json!({
        "status":"PASS",
        "evidence_level":trace.evidence_level,
        "source":trace.source,
        "sample_ms":delay.as_millis(),
        "evaluation_eligible":trace.is_evaluation_eligible(),
        "benchmark":"NOT_RUN"
    }))
}

fn run_id(reply: &NativeReply) -> anyhow::Result<String> {
    match reply {
        NativeReply::P(PReply::Accepted { run_id }) => Ok(run_id.clone()),
        NativeReply::Q(QReply::Accepted { run_id, .. }) => Ok(run_id.clone()),
        _ => anyhow::bail!("expected accepted reply"),
    }
}

fn reply_shape(reply: &NativeReply) -> &'static str {
    match reply {
        NativeReply::P(_) => "P",
        NativeReply::Q(_) => "Q",
    }
}
