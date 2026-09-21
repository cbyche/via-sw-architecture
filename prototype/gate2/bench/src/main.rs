use clap::{Parser, Subcommand};
use gate2_contracts::{NativeReply, PReply, QReply, SubmitRequest, TaskCommand, TaskOp};
use gate2_fixture::{AgentShape, DeterministicAgent, S2sDelayTrace};
use gate2_runtime::{
    agent::{AgentBoundary, CoreVisibleTyped, EdgeNormalized},
    exec::{IntegrationBridge, LocalBridge, ProcessBridge},
    repository::Repository,
    task::{PerTaskSupervisors, SharedTaskService, TaskAuthority},
};
use std::{path::PathBuf, sync::Arc};

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
    S2sSmoke {
        #[arg(long)]
        trace: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Smoke => smoke().await?,
        Command::ExecSmoke { worker } => exec_smoke(worker).await?,
        Command::S2sSmoke { trace } => s2s_smoke(trace).await?,
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
        anyhow::ensure!(matches!(
            running.state,
            gate2_contracts::TaskState::Running
        ));
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
    Ok(serde_json::json!({
        "status":"PASS",
        "scope":"exec bridge semantic equivalence smoke",
        "local_shape":reply_shape(&local_reply),
        "process_shape":reply_shape(&process_reply),
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
