use gate2_contracts::{TaskCommand, TaskOp, TaskState};
use gate2_fixture::{AgentShape, DeterministicAgent};
use gate2_runtime::{
    agent::{AgentBoundary, EdgeNormalized},
    handoff::HandoffCoordinator,
    repository::Repository,
    sync::AgentSynchronizer,
    task::{PerTaskSupervisors, SharedTaskService, TaskAuthority},
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tempfile::tempdir;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout, Command},
    time::{Instant, sleep},
};

fn authority(candidate: &str, repository: Repository) -> anyhow::Result<Arc<dyn TaskAuthority>> {
    Ok(match candidate {
        "shared" => Arc::new(SharedTaskService::new(repository)),
        "per_task" => Arc::new(PerTaskSupervisors::new(repository)),
        other => anyhow::bail!("unknown TASK candidate: {other}"),
    })
}

async fn host_prepare(
    candidate: &str,
    repository: &Repository,
    agent: &DeterministicAgent,
    active_tasks: usize,
    completed: bool,
) -> anyhow::Result<serde_json::Value> {
    let task_authority = authority(candidate, repository.clone())?;
    let coordinator = HandoffCoordinator::new(
        EdgeNormalized::new(agent.clone()),
        repository.clone(),
    );
    let mut prepared = Vec::with_capacity(active_tasks);
    for index in 0..active_tasks {
        let task_id = format!("W09-PROC-T{index}");
        let created = task_authority
            .apply(TaskCommand {
                command_id: format!("w09-proc-create-{index}"),
                task_id: task_id.clone(),
                expected_revision: 0,
                op: TaskOp::Create {
                    goal: format!("W-09 process recovery Task {index}"),
                },
            })
            .await?;
        let result = coordinator
            .handoff(
                task_authority.as_ref(),
                created,
                format!("W-09 process recovery Task {index}"),
                format!("w09-proc-submit-{index}"),
            )
            .await?;
        let mut final_task = result.task;
        if completed {
            let artifact = format!("W09-PROC-ART-{index}");
            agent.complete(&result.accepted.run_id, artifact.clone())?;
            let sync = AgentSynchronizer::new(EdgeNormalized::new(agent.clone()));
            let (updated, _) = sync
                .consume_events_since(task_authority.as_ref(), final_task, 1)
                .await?;
            anyhow::ensure!(updated.state == TaskState::Completed);
            anyhow::ensure!(updated.result.as_deref() == Some(artifact.as_str()));
            final_task = updated;
        }
        prepared.push(serde_json::json!({
            "task_id":task_id,
            "run_id":result.accepted.run_id,
            "state":final_task.state
        }));
    }
    Ok(serde_json::json!({"status":"PASS","prepared":prepared}))
}

async fn host_probe(
    candidate: &str,
    repository: &Repository,
    agent: &DeterministicAgent,
    active_tasks: usize,
    completed: bool,
) -> anyhow::Result<serde_json::Value> {
    let task_authority = authority(candidate, repository.clone())?;
    let boundary = EdgeNormalized::new(agent.clone());
    let mut recovered = Vec::with_capacity(active_tasks);

    for index in 0..active_tasks {
        let task_id = format!("W09-PROC-T{index}");
        let current = repository.get(&task_id).await?;
        let link = repository
            .execution_link(&task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing ExecutionLink for {task_id}"))?;
        anyhow::ensure!(current.run_id.as_deref() == Some(link.run_id.as_str()));
        anyhow::ensure!(link.context_id.is_some());

        let snapshot = boundary.query(&link.run_id).await?;
        anyhow::ensure!(snapshot.run_id == link.run_id);
        if completed {
            anyhow::ensure!(current.state == TaskState::Completed);
            anyhow::ensure!(snapshot.state == "completed");
            anyhow::ensure!(current.result == snapshot.artifact);
        } else {
            anyhow::ensure!(current.state == TaskState::Running);
            anyhow::ensure!(snapshot.state == "running");
            let local = task_authority
                .apply(TaskCommand {
                    command_id: format!("w09-proc-recovery-cancel-{index}"),
                    task_id: task_id.clone(),
                    expected_revision: current.revision,
                    op: TaskOp::Cancel,
                })
                .await?;
            anyhow::ensure!(local.state == TaskState::CancelRequested);
            let external = boundary.cancel(&link.run_id).await?;
            anyhow::ensure!(external.requested);
        }
        recovered.push(serde_json::json!({
            "task_id":task_id,
            "run_id":link.run_id,
            "context_id":link.context_id,
            "task_state_before_probe":current.state,
            "agent_state":snapshot.state,
            "control_verified":!completed
        }));
    }
    Ok(serde_json::json!({"status":"PASS","recovered":recovered}))
}

async fn write_line(
    stdout: &mut tokio::io::Stdout,
    value: &serde_json::Value,
) -> anyhow::Result<()> {
    stdout
        .write_all(serde_json::to_string(value)?.as_bytes())
        .await?;
    stdout.write_all(b"\n").await?;
    stdout.flush().await?;
    Ok(())
}

/// Hidden child-process role used by the external controller below. The process owns
/// volatile VIA Task runtime objects; only SQLite and the persistent downstream Agent
/// fixture state live outside its fault domain.
pub async fn runtime_host(
    candidate: String,
    db: PathBuf,
    agent_state: PathBuf,
    active_tasks: usize,
    completed: bool,
) -> anyhow::Result<()> {
    let repository = Repository::open(db)?;
    let agent = DeterministicAgent::persistent(AgentShape::Q, agent_state)?;
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = lines.next_line().await? {
        let request: serde_json::Value = serde_json::from_str(&line)?;
        match request.get("op").and_then(|value| value.as_str()) {
            Some("prepare") => {
                let value =
                    host_prepare(&candidate, &repository, &agent, active_tasks, completed).await?;
                write_line(&mut stdout, &value).await?;
            }
            Some("probe") => {
                let value =
                    host_probe(&candidate, &repository, &agent, active_tasks, completed).await?;
                write_line(&mut stdout, &value).await?;
            }
            Some("abort") => std::process::abort(),
            other => anyhow::bail!("unsupported W-09 runtime-host command: {other:?}"),
        }
    }
    Ok(())
}

struct RuntimeSession {
    child: Child,
    stdin: ChildStdin,
    stdout: Lines<BufReader<ChildStdout>>,
}

impl RuntimeSession {
    async fn spawn(
        executable: &Path,
        candidate: &str,
        db: &Path,
        agent_state: &Path,
        active_tasks: usize,
        completed: bool,
    ) -> anyhow::Result<Self> {
        let mut command = Command::new(executable);
        command
            .arg("w09-runtime-host")
            .arg("--task-candidate")
            .arg(candidate)
            .arg("--db")
            .arg(db)
            .arg("--agent-state")
            .arg(agent_state)
            .arg("--active-tasks")
            .arg(active_tasks.to_string());
        if completed {
            command.arg("--completed");
        }
        let mut child = command
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("W-09 runtime host stdin missing"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("W-09 runtime host stdout missing"))?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout).lines(),
        })
    }

    async fn request(&mut self, op: &str) -> anyhow::Result<Option<serde_json::Value>> {
        self.stdin
            .write_all(serde_json::to_string(&serde_json::json!({"op":op}))?.as_bytes())
            .await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        match self.stdout.next_line().await? {
            Some(line) => Ok(Some(serde_json::from_str(&line)?)),
            None => Ok(None),
        }
    }
}

async fn one_stratum(
    executable: &Path,
    candidate: &str,
    active_tasks: usize,
    completed: bool,
    restart_delay: Duration,
) -> anyhow::Result<serde_json::Value> {
    let dir = tempdir()?;
    let db = dir.path().join("via-task.db");
    let agent_state = dir.path().join("external-agent-state.json");

    let mut first = RuntimeSession::spawn(
        executable,
        candidate,
        &db,
        &agent_state,
        active_tasks,
        completed,
    )
    .await?;
    let prepared = first
        .request("prepare")
        .await?
        .ok_or_else(|| anyhow::anyhow!("W-09 runtime host ended during prepare"))?;
    anyhow::ensure!(prepared["status"] == "PASS");

    let fault_start = Instant::now();
    let abort_reply = first.request("abort").await?;
    anyhow::ensure!(abort_reply.is_none(), "fatal process fault returned normally");
    let _ = first.child.wait().await?;
    sleep(restart_delay).await;

    let mut second = RuntimeSession::spawn(
        executable,
        candidate,
        &db,
        &agent_state,
        active_tasks,
        completed,
    )
    .await?;
    let probe = second
        .request("probe")
        .await?
        .ok_or_else(|| anyhow::anyhow!("W-09 restarted host ended during probe"))?;
    anyhow::ensure!(probe["status"] == "PASS");

    Ok(serde_json::json!({
        "task_candidate":candidate,
        "state":if completed {"completed_queryable"} else {"running_queryable"},
        "active_tasks":active_tasks,
        "process_abort_observed":true,
        "restart_delay_ms":restart_delay.as_millis(),
        "recovery_elapsed_ms":fault_start.elapsed().as_millis(),
        "recovered":probe["recovered"]
    }))
}

pub async fn whole_process_smoke() -> anyhow::Result<serde_json::Value> {
    let executable = std::env::current_exe()?;
    let mut strata = Vec::new();
    for candidate in ["shared", "per_task"] {
        for completed in [false, true] {
            for active_tasks in [1usize, 4usize] {
                strata.push(
                    one_stratum(
                        &executable,
                        candidate,
                        active_tasks,
                        completed,
                        Duration::from_millis(2),
                    )
                    .await?,
                );
            }
        }
    }
    Ok(serde_json::json!({
        "status":"PASS",
        "scope":"W-09 whole-VIA process-abort four-strata correctness smoke",
        "strata":strata,
        "external_agent_fixture":"persistent state outside VIA runtime process",
        "durable_via_state":"SQLite reopened after process loss",
        "w09_representative_metric":"NOT_RUN",
        "w09_metric_eligible":false,
        "note":"Final W-09 requires the frozen 500ms controller delay and 100 scored trials per stratum."
    }))
}
