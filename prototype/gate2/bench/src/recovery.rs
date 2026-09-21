use gate2_contracts::{TaskCommand, TaskOp, TaskState};
use gate2_fixture::{AgentShape, DeterministicAgent};
use gate2_runtime::{
    agent::{AgentBoundary, EdgeNormalized},
    handoff::HandoffCoordinator,
    repository::Repository,
    sync::AgentSynchronizer,
    task::{PerTaskSupervisors, SharedTaskService, TaskAuthority},
};
use std::{sync::Arc, time::Duration};
use tempfile::NamedTempFile;
use tokio::time::{Instant, sleep};

fn authority(candidate: &str, repository: Repository) -> Arc<dyn TaskAuthority> {
    match candidate {
        "shared" => Arc::new(SharedTaskService::new(repository)),
        "per_task" => Arc::new(PerTaskSupervisors::new(repository)),
        other => panic!("unknown recovery candidate: {other}"),
    }
}

async fn prepare(
    candidate: &str,
    path: &std::path::Path,
    agent: &DeterministicAgent,
    active_tasks: usize,
    completed: bool,
) -> anyhow::Result<()> {
    let repository = Repository::open(path)?;
    let task_authority = authority(candidate, repository.clone());
    let coordinator = HandoffCoordinator::new(
        EdgeNormalized::new(agent.clone()),
        repository.clone(),
    );

    for index in 0..active_tasks {
        let task_id = format!("W09-T{index}");
        let created = task_authority
            .apply(TaskCommand {
                command_id: format!("w09-create-{index}"),
                task_id: task_id.clone(),
                expected_revision: 0,
                op: TaskOp::Create {
                    goal: format!("W-09 recovery Task {index}"),
                },
            })
            .await?;
        let result = coordinator
            .handoff(
                task_authority.as_ref(),
                created,
                format!("W-09 recovery Task {index}"),
                format!("w09-submit-{index}"),
            )
            .await?;

        if completed {
            let artifact = format!("W09-ART-{index}");
            agent.complete(&result.accepted.run_id, artifact.clone())?;
            let synchronizer = AgentSynchronizer::new(EdgeNormalized::new(agent.clone()));
            let (updated, _) = synchronizer
                .consume_events_since(task_authority.as_ref(), result.task, 1)
                .await?;
            anyhow::ensure!(updated.state == TaskState::Completed);
            anyhow::ensure!(updated.result.as_deref() == Some(artifact.as_str()));
        }
    }
    Ok(())
}

async fn recover(
    candidate: &str,
    path: &std::path::Path,
    agent: &DeterministicAgent,
    active_tasks: usize,
    completed: bool,
    restart_delay: Duration,
) -> anyhow::Result<serde_json::Value> {
    let fault_start = Instant::now();

    // The pre-fault Repository/TaskAuthority values are gone before this delay.
    // Only the SQLite file and the external Agent fixture state survive.
    sleep(restart_delay).await;

    let repository = Repository::open(path)?;
    let task_authority = authority(candidate, repository.clone());
    let boundary = EdgeNormalized::new(agent.clone());

    let mut recovered = Vec::new();
    for index in 0..active_tasks {
        let task_id = format!("W09-T{index}");
        let current = repository.get(&task_id).await?;
        let link = repository
            .execution_link(&task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing durable ExecutionLink for {task_id}"))?;
        anyhow::ensure!(current.run_id.as_deref() == Some(link.run_id.as_str()));
        anyhow::ensure!(
            link.context_id.is_some(),
            "Q-shaped fixture must preserve context identity"
        );

        let snapshot = boundary.query(&link.run_id).await?;
        anyhow::ensure!(snapshot.run_id == link.run_id);

        if completed {
            anyhow::ensure!(current.state == TaskState::Completed);
            anyhow::ensure!(snapshot.state == "completed");
            anyhow::ensure!(current.result == snapshot.artifact);
        } else {
            anyhow::ensure!(current.state == TaskState::Running);
            anyhow::ensure!(snapshot.state == "running");

            // A running Task is not considered recovered merely because it can be read.
            // Exercise one permitted user control through the recreated Task authority.
            let cancel_local = task_authority
                .apply(TaskCommand {
                    command_id: format!("w09-recovery-cancel-{index}"),
                    task_id: task_id.clone(),
                    expected_revision: current.revision,
                    op: TaskOp::Cancel,
                })
                .await?;
            anyhow::ensure!(cancel_local.state == TaskState::CancelRequested);
            let cancel_external = boundary.cancel(&link.run_id).await?;
            anyhow::ensure!(cancel_external.requested);
        }

        recovered.push(serde_json::json!({
            "task_id":task_id,
            "run_id":link.run_id,
            "context_id":link.context_id,
            "pre_control_state":current.state,
            "agent_state":snapshot.state
        }));
    }

    Ok(serde_json::json!({
        "candidate":candidate,
        "state":if completed {"completed_queryable"} else {"running_queryable"},
        "active_tasks":active_tasks,
        "restart_delay_ms":restart_delay.as_millis(),
        "recovery_elapsed_ms":fault_start.elapsed().as_millis(),
        "recovered":recovered
    }))
}

async fn one_stratum(
    candidate: &str,
    active_tasks: usize,
    completed: bool,
) -> anyhow::Result<serde_json::Value> {
    let file = NamedTempFile::new()?;
    let path = file.path().to_path_buf();
    let agent = DeterministicAgent::new(AgentShape::Q);

    prepare(candidate, &path, &agent, active_tasks, completed).await?;

    // Dropping all VIA-side values happens when prepare returns. The Agent clone held
    // here owns the external fixture state independently of the recreated VIA runtime.
    recover(
        candidate,
        &path,
        &agent,
        active_tasks,
        completed,
        Duration::from_millis(2),
    )
    .await
}

pub async fn whole_restart_smoke() -> anyhow::Result<serde_json::Value> {
    let mut strata = Vec::new();
    for candidate in ["shared", "per_task"] {
        for completed in [false, true] {
            for active_tasks in [1usize, 4usize] {
                strata.push(one_stratum(candidate, active_tasks, completed).await?);
            }
        }
    }

    Ok(serde_json::json!({
        "status":"PASS",
        "scope":"W-09 whole-restart four-strata correctness smoke for both TASK candidates",
        "strata":strata,
        "fault_model":"VIA-side runtime objects discarded; SQLite and external Agent fixture survive",
        "os_process_kill":false,
        "w09_representative_metric":"NOT_RUN",
        "w09_metric_eligible":false,
        "note":"Final W-09 requires real process restart, 500ms controller delay, 100 trials/stratum, and the two integration-fatal strata."
    }))
}
