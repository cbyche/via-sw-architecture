mod exec_recovery;
mod recovery;
mod whole_process_recovery;

use clap::{Parser, Subcommand};
use gate2_contracts::{
    NativeReply, PReply, QReply, SubmitRequest, TaskCommand, TaskOp, WorkerRequest, WorkerResponse,
};
use gate2_fixture::{AgentShape, DeterministicAgent, S2sDelayTrace};
use gate2_runtime::{
    agent::{AgentBoundary, CoreVisibleTyped, EdgeNormalized},
    exec::{IntegrationBridge, LocalBridge, ProcessBridge},
    repository::Repository,
    task::{PerTaskSupervisors, SharedTaskService, TaskAuthority},
    workload::{BackgroundLoadConfig, run_background_load},
};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout, Command as TokioCommand},
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
    S2sSmoke {
        #[arg(long)]
        trace: PathBuf,
    },
    W04LoadSmoke,
    W09WholeRestartSmoke,
    W09IntegrationFatalSmoke {
        #[arg(long)]
        host: PathBuf,
        #[arg(long)]
        worker: PathBuf,
    },
    W09WholeProcessSmoke,
    W09RuntimeHost {
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
        Command::S2sSmoke { trace } => s2s_smoke(trace).await?,
        Command::W04LoadSmoke => w04_load_smoke().await?,
        Command::W09WholeRestartSmoke => recovery::whole_restart_smoke().await?,
        Command::W09IntegrationFatalSmoke { host, worker } => {
            exec_recovery::integration_fatal_smoke(host, worker).await?
        }
        Command::W09WholeProcessSmoke => whole_process_recovery::whole_process_smoke().await?,
        Command::W09RuntimeHost {
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
        anyhow::ensure!(matches!(running.state, gate2_contracts::TaskState::Running));
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


struct HostSession {
    child: Child,
    stdin: ChildStdin,
    stdout: Lines<BufReader<ChildStdout>>,
}

impl HostSession {
    async fn spawn(
        host: &PathBuf,
        mode: &str,
        worker: Option<&PathBuf>,
    ) -> anyhow::Result<Self> {
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

    async fn request(
        &mut self,
        request: WorkerRequest,
    ) -> anyhow::Result<Option<WorkerResponse>> {
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

async fn exec_blast_smoke(
    host: PathBuf,
    worker: PathBuf,
) -> anyhow::Result<serde_json::Value> {
    let mut shared = HostSession::spawn(&host, "shared", None).await?;
    anyhow::ensure!(
        matches!(shared.request(WorkerRequest::Ping).await?, Some(WorkerResponse::Pong)),
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
        matches!(isolated.request(WorkerRequest::Ping).await?, Some(WorkerResponse::Pong)),
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
        matches!(isolated.request(WorkerRequest::Ping).await?, Some(WorkerResponse::Pong)),
        "isolated host not usable after worker restart"
    );
    anyhow::ensure!(
        isolated.child.try_wait()?.is_none(),
        "isolated Core host exited after integration worker fatal fault"
    );

    Ok(serde_json::json!({
        "status":"PASS",
        "scope":"symmetric EXEC fatal-fault blast-radius smoke",
        "shared_host_exited":true,
        "shared_exit_success":shared_status.success(),
        "isolated_core_host_survived":true,
        "isolated_worker_restarted":true,
        "benchmark":"NOT_RUN"
    }))
}

async fn w04_load_smoke() -> anyhow::Result<serde_json::Value> {
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
                "W-04 workload smoke dropped background updates"
            );
            runs.push(serde_json::json!({
                "candidate": candidate,
                "active_tasks": active_tasks,
                "updates_applied": report.updates_applied,
                "elapsed_ms": report.elapsed_ms,
                "w04_metric_eligible": report.w04_metric_eligible
            }));
        }
    }

    Ok(serde_json::json!({
        "status":"PASS",
        "scope":"W-04 fixed 1-vs-4 active background Task load-shape smoke",
        "fixed_shape":"1 update per Task per cadence; production cadence remains 1000ms up to 30s",
        "runs":runs,
        "w04_representative_metric":"NOT_RUN",
        "note":"Final W-04 requires the six W-01 foreground probes while this load is active.",
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
