use gate2_contracts::{
    NativeReply, QReply, SubmitRequest, TaskCommand, TaskOp, TaskState, WorkerRequest,
    WorkerResponse,
};
use gate2_runtime::{
    repository::Repository,
    task::{SharedTaskService, TaskAuthority},
};
use std::{path::{Path, PathBuf}, time::Duration};
use tempfile::tempdir;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout, Command},
    time::{Instant, sleep},
};

struct CandidateHost {
    child: Child,
    stdin: ChildStdin,
    stdout: Lines<BufReader<ChildStdout>>,
}

impl CandidateHost {
    async fn spawn(
        host: &Path,
        mode: &str,
        worker: Option<&Path>,
        agent_state_file: &Path,
    ) -> anyhow::Result<Self> {
        let mut command = Command::new(host);
        command
            .arg("--mode")
            .arg(mode)
            .arg("--agent-state-file")
            .arg(agent_state_file)
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

    async fn ping(&mut self) -> anyhow::Result<()> {
        anyhow::ensure!(
            matches!(
                self.request(WorkerRequest::Ping).await?,
                Some(WorkerResponse::Pong)
            ),
            "candidate host did not answer ping"
        );
        Ok(())
    }
}

fn accepted_q(response: Option<WorkerResponse>) -> anyhow::Result<(String, String)> {
    match response {
        Some(WorkerResponse::Reply {
            reply: NativeReply::Q(QReply::Accepted { context_id, run_id }),
        }) => Ok((context_id, run_id)),
        other => anyhow::bail!("expected Q accepted reply, got {other:?}"),
    }
}

fn running_q(response: Option<WorkerResponse>) -> anyhow::Result<()> {
    match response {
        Some(WorkerResponse::Reply {
            reply:
                NativeReply::Q(QReply::Snapshot {
                    state,
                    artifact,
                    ..
                }),
        }) => {
            anyhow::ensure!(state == "running", "external Agent state is not running");
            anyhow::ensure!(artifact.is_none(), "running Agent unexpectedly has terminal artifact");
            Ok(())
        }
        other => anyhow::bail!("expected Q running snapshot, got {other:?}"),
    }
}

fn cancel_requested_q(response: Option<WorkerResponse>) -> anyhow::Result<()> {
    match response {
        Some(WorkerResponse::Reply {
            reply: NativeReply::Q(QReply::CancelRequested { .. }),
        }) => Ok(()),
        other => anyhow::bail!("expected Q cancel-request acknowledgement, got {other:?}"),
    }
}

async fn prepare_links(
    host: &mut CandidateHost,
    repository: &Repository,
    active_tasks: usize,
) -> anyhow::Result<Vec<(String, String)>> {
    // TASK-DP01 is deliberately fixed to SharedTaskService for this one-factor EXEC
    // comparison. The EXEC candidate must be the only architecture dimension changed.
    let authority = SharedTaskService::new(repository.clone());
    let mut links = Vec::with_capacity(active_tasks);

    for index in 0..active_tasks {
        let task_id = format!("W09-EXEC-T{index}");
        let submission_key = format!("w09-exec-submit-{index}");
        let goal = format!("W-09 integration-fatal Task {index}");
        let created = authority
            .apply(TaskCommand {
                command_id: format!("w09-exec-create-{index}"),
                task_id: task_id.clone(),
                expected_revision: 0,
                op: TaskOp::Create { goal: goal.clone() },
            })
            .await?;
        let prepared = authority
            .apply(TaskCommand {
                command_id: format!("prepare-handoff:{submission_key}"),
                task_id: task_id.clone(),
                expected_revision: created.revision,
                op: TaskOp::PrepareHandoff {
                    submission_key: submission_key.clone(),
                    goal: goal.clone(),
                },
            })
            .await?;

        let (context_id, run_id) = accepted_q(
            host.request(WorkerRequest::Submit {
                request: SubmitRequest {
                    task_id: task_id.clone(),
                    submission_key: submission_key.clone(),
                    goal,
                },
            })
            .await?,
        )?;

        let running = authority
            .apply(TaskCommand {
                command_id: format!("confirm-handoff:{submission_key}"),
                task_id: task_id.clone(),
                expected_revision: prepared.revision,
                op: TaskOp::ConfirmHandoff {
                    submission_key,
                    run_id: run_id.clone(),
                    context_id: Some(context_id.clone()),
                },
            })
            .await?;
        anyhow::ensure!(running.state == TaskState::Running);
        anyhow::ensure!(running.run_id.as_deref() == Some(run_id.as_str()));

        let link = repository
            .execution_link(&task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing durable ExecutionLink for {task_id}"))?;
        anyhow::ensure!(link.run_id == run_id);
        anyhow::ensure!(link.context_id.as_deref() == Some(context_id.as_str()));
        links.push((task_id, run_id));
    }
    Ok(links)
}

async fn one_stratum(
    host_binary: &Path,
    worker_binary: &Path,
    mode: &str,
    active_tasks: usize,
) -> anyhow::Result<serde_json::Value> {
    let dir = tempdir()?;
    let db_path = dir.path().join("via-task.db");
    let agent_state = dir.path().join("external-agent-state.json");
    let repository = Repository::open(&db_path)?;

    let worker = (mode == "isolated").then_some(worker_binary);
    let mut host = CandidateHost::spawn(host_binary, mode, worker, &agent_state).await?;
    host.ping().await?;
    let links = prepare_links(&mut host, &repository, active_tasks).await?;

    let fault_start = Instant::now();
    let abort_reply = host.request(WorkerRequest::AbortHost).await?;
    let shared_host_restarted = if mode == "shared" {
        anyhow::ensure!(
            abort_reply.is_none(),
            "shared-process fatal fault unexpectedly returned normally"
        );
        let _ = host.child.wait().await?;
        sleep(Duration::from_millis(2)).await;
        host = CandidateHost::spawn(host_binary, mode, None, &agent_state).await?;
        host.ping().await?;
        true
    } else {
        anyhow::ensure!(
            matches!(abort_reply, Some(WorkerResponse::Pong)),
            "isolated Core host did not survive worker fatal/restart"
        );
        anyhow::ensure!(
            host.child.try_wait()?.is_none(),
            "isolated Core host exited after worker fatal fault"
        );
        false
    };

    let authority = SharedTaskService::new(repository.clone());
    let mut recovered = Vec::with_capacity(active_tasks);
    for (index, (task_id, run_id)) in links.into_iter().enumerate() {
        let task = repository.get(&task_id).await?;
        let link = repository
            .execution_link(&task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("ExecutionLink disappeared for {task_id}"))?;
        anyhow::ensure!(task.state == TaskState::Running);
        anyhow::ensure!(task.run_id.as_deref() == Some(run_id.as_str()));
        anyhow::ensure!(link.run_id == run_id);

        running_q(
            host.request(WorkerRequest::Query {
                run_id: run_id.clone(),
            })
            .await?,
        )?;

        // Recovery completion requires usable control, not just a queryable process.
        let cancel_local = authority
            .apply(TaskCommand {
                command_id: format!("w09-exec-recovery-cancel-{index}"),
                task_id: task_id.clone(),
                expected_revision: task.revision,
                op: TaskOp::Cancel,
            })
            .await?;
        anyhow::ensure!(cancel_local.state == TaskState::CancelRequested);
        cancel_requested_q(
            host.request(WorkerRequest::Cancel {
                run_id: run_id.clone(),
            })
            .await?,
        )?;

        recovered.push(serde_json::json!({
            "task_id":task_id,
            "run_id":run_id,
            "durable_link_preserved":true,
            "agent_queryable_after_fault":true,
            "user_control_available_after_fault":true
        }));
    }

    Ok(serde_json::json!({
        "exec_candidate":mode,
        "active_tasks":active_tasks,
        "external_agent_state_file":true,
        "shared_host_restarted":shared_host_restarted,
        "isolated_core_survived":mode == "isolated",
        "recovery_elapsed_ms":fault_start.elapsed().as_millis(),
        "recovered":recovered
    }))
}

pub async fn integration_fatal_smoke(
    host_binary: PathBuf,
    worker_binary: PathBuf,
) -> anyhow::Result<serde_json::Value> {
    let mut strata = Vec::new();
    for mode in ["shared", "isolated"] {
        for active_tasks in [1usize, 4usize] {
            strata.push(
                one_stratum(&host_binary, &worker_binary, mode, active_tasks).await?,
            );
        }
    }
    Ok(serde_json::json!({
        "status":"PASS",
        "scope":"W-09 integration-host fatal running/queryable × 1/4 Task correctness smoke",
        "task_architecture_fixed":"SharedTaskService for both EXEC candidates",
        "external_agent_fixture":"persistent state outside integration worker volatile memory",
        "strata":strata,
        "w09_representative_metric":"NOT_RUN",
        "w09_metric_eligible":false,
        "note":"Final W-09 still requires frozen 500ms restart controller timing and 100 scored trials per stratum."
    }))
}
