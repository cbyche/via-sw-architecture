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
use via_contracts::{TaskCommand, TaskOp, TaskState};
use via_fixture::{AgentShape, DeterministicAgent};
use via_runtime::{
    agent::{AgentBoundary, EdgeNormalized},
    handoff::HandoffCoordinator,
    repository::Repository,
    sync::AgentSynchronizer,
    task::{PerTaskSupervisors, SharedTaskService, TaskAuthority},
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
    let coordinator =
        HandoffCoordinator::new(EdgeNormalized::new(agent.clone()), repository.clone());
    let mut prepared = Vec::with_capacity(active_tasks);
    for index in 0..active_tasks {
        let task_id = format!("QA09-PROC-T{index}");
        let created = task_authority
            .apply(TaskCommand {
                command_id: format!("qa09-proc-create-{index}"),
                task_id: task_id.clone(),
                expected_revision: 0,
                op: TaskOp::Create {
                    goal: format!("QA-09 process recovery Task {index}"),
                },
            })
            .await?;
        let result = coordinator
            .handoff(
                task_authority.as_ref(),
                created,
                format!("QA-09 process recovery Task {index}"),
                format!("qa09-proc-submit-{index}"),
            )
            .await?;
        let mut final_task = result.task;
        if completed {
            let artifact = format!("QA09-PROC-ART-{index}");
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
        let task_id = format!("QA09-PROC-T{index}");
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
                    command_id: format!("qa09-proc-recovery-cancel-{index}"),
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
            other => anyhow::bail!("unsupported QA-09 runtime-host command: {other:?}"),
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
            .arg("qa09-runtime-host")
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
            .ok_or_else(|| anyhow::anyhow!("QA-09 runtime host stdin missing"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("QA-09 runtime host stdout missing"))?;
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
        .ok_or_else(|| anyhow::anyhow!("QA-09 runtime host ended during prepare"))?;
    anyhow::ensure!(prepared["status"] == "PASS");

    let fault_start = Instant::now();
    let abort_reply = first.request("abort").await?;
    anyhow::ensure!(
        abort_reply.is_none(),
        "fatal process fault returned normally"
    );
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
        .ok_or_else(|| anyhow::anyhow!("QA-09 restarted host ended during probe"))?;
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
    whole_process("smoke", None).await
}

fn nearest_rank_p95(values: &[u64]) -> anyhow::Result<u64> {
    anyhow::ensure!(!values.is_empty(), "p95 requires at least one QA-09 trial");
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let rank = (95 * sorted.len()).div_ceil(100);
    Ok(sorted[rank - 1])
}

pub async fn whole_process(
    profile: &str,
    freeze_fingerprint: Option<&str>,
) -> anyhow::Result<serde_json::Value> {
    let (trials_per_stratum, restart_delay, metric_eligible) = match profile {
        "smoke" => (1usize, Duration::from_millis(2), false),
        "frozen" => (100usize, Duration::from_millis(500), true),
        other => anyhow::bail!("unknown QA-09 profile: {other}; use smoke or frozen"),
    };
    if metric_eligible {
        let fingerprint = freeze_fingerprint
            .ok_or_else(|| anyhow::anyhow!("frozen QA-09 requires --freeze-fingerprint"))?;
        anyhow::ensure!(
            fingerprint.len() == 64 && fingerprint.chars().all(|value| value.is_ascii_hexdigit()),
            "invalid QA-09 freeze fingerprint"
        );
    }
    let executable = std::env::current_exe()?;
    let mut candidates = Vec::new();
    for candidate in ["shared", "per_task"] {
        let mut strata = Vec::new();
        for completed in [false, true] {
            for active_tasks in [1usize, 4usize] {
                let mut trials = Vec::with_capacity(trials_per_stratum);
                for trial in 0..trials_per_stratum {
                    let mut value = one_stratum(
                        &executable,
                        candidate,
                        active_tasks,
                        completed,
                        restart_delay,
                    )
                    .await?;
                    value["trial"] = serde_json::json!(trial + 1);
                    trials.push(value);
                }
                let elapsed = trials
                    .iter()
                    .map(|value| value["recovery_elapsed_ms"].as_u64().unwrap())
                    .collect::<Vec<_>>();
                strata.push(serde_json::json!({
                    "state":if completed {"completed_queryable"} else {"running_queryable"},
                    "active_tasks":active_tasks,
                    "trials":trials,
                    "p95_recovery_ms":nearest_rank_p95(&elapsed)?
                }));
            }
        }
        let p95_values = strata
            .iter()
            .map(|value| value["p95_recovery_ms"].as_f64().unwrap())
            .collect::<Vec<_>>();
        candidates.push(serde_json::json!({
            "task_candidate":candidate,
            "strata":strata,
            "four_strata_mean_p95_ms":p95_values.iter().sum::<f64>() / p95_values.len() as f64
        }));
    }
    Ok(serde_json::json!({
        "status":"PASS",
        "scope":"QA-09 whole-VIA process-abort four-strata trials",
        "profile":profile,
        "freeze_fingerprint":freeze_fingerprint,
        "trials_per_stratum":trials_per_stratum,
        "restart_delay_ms":restart_delay.as_millis(),
        "candidates":candidates,
        "external_agent_fixture":"persistent state outside VIA runtime process",
        "durable_via_state":"SQLite reopened after process loss",
        "legacy_representative_fragment":"REQUIRES_TWO_INTEGRATION_FATAL_STRATA",
        "legacy_profile_metric_eligible":metric_eligible,
        "active_qa_metric_eligible":false,
        "note":"Legacy recovery diagnostic only; it is not the active draft QA-09 representative metric."
    }))
}
