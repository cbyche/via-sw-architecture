use crate::{repository::ApplyError, task::TaskAuthority};
use gate2_contracts::{AgentObservation, ObservationKind, TaskCommand, TaskOp, TaskView};
use serde::Serialize;
use std::{sync::Arc, time::Duration};
use tokio::time::{Instant, sleep_until};

#[derive(Debug, Clone, Copy)]
pub struct BackgroundLoadConfig {
    pub active_tasks: usize,
    pub update_period: Duration,
    pub rounds: usize,
}

impl BackgroundLoadConfig {
    pub fn validate(self) -> Result<Self, ApplyError> {
        if !matches!(self.active_tasks, 1 | 4) {
            return Err(ApplyError::Invalid(
                "W-04 background load requires exactly 1 or 4 active Tasks".into(),
            ));
        }
        if self.rounds == 0 {
            return Err(ApplyError::Invalid(
                "W-04 background load requires at least one update round".into(),
            ));
        }
        if self.update_period.is_zero() {
            return Err(ApplyError::Invalid(
                "W-04 background update period must be non-zero".into(),
            ));
        }
        Ok(self)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BackgroundLoadReport {
    pub active_tasks: usize,
    pub rounds: usize,
    pub update_period_ms: u128,
    pub updates_applied: usize,
    pub elapsed_ms: u128,
    pub w04_metric_eligible: bool,
}

async fn prepare_task(
    authority: &dyn TaskAuthority,
    index: usize,
) -> Result<TaskView, ApplyError> {
    let task_id = format!("W04-BG-{index}");
    let run_id = format!("w04-run-{index}");
    let created = authority
        .apply(TaskCommand {
            command_id: format!("w04-create-{index}"),
            task_id: task_id.clone(),
            expected_revision: 0,
            op: TaskOp::Create {
                goal: format!("deterministic W-04 background Task {index}"),
            },
        })
        .await?;
    authority
        .apply(TaskCommand {
            command_id: format!("w04-accept-{index}"),
            task_id,
            expected_revision: created.revision,
            op: TaskOp::AcceptExecution { run_id },
        })
        .await
}

/// Drive the fixed W-04 background workload shape.
///
/// This intentionally does not compute W-04. The final W-04 representative metric is
/// W-01 Macro-p95 under this load at 4 active Tasks divided by the same candidate's
/// W-01 Macro-p95 at 1 active Task. A caller must run the six W-01 foreground probes
/// while this workload is active and retain those endpoint measurements separately.
pub async fn run_background_load(
    authority: Arc<dyn TaskAuthority>,
    config: BackgroundLoadConfig,
) -> Result<BackgroundLoadReport, ApplyError> {
    let config = config.validate()?;
    let mut tasks = Vec::with_capacity(config.active_tasks);
    for index in 0..config.active_tasks {
        tasks.push(prepare_task(authority.as_ref(), index).await?);
    }

    // All background Tasks share one logical cadence origin. This avoids accidentally
    // giving later-created Tasks a lower event rate.
    let start = Instant::now();
    let mut workers = Vec::with_capacity(tasks.len());
    for mut current in tasks {
        let authority = authority.clone();
        let update_period = config.update_period;
        let rounds = config.rounds;
        workers.push(tokio::spawn(async move {
            let mut applied = 0usize;
            for round in 1..=rounds {
                let multiplier = u32::try_from(round)
                    .map_err(|_| ApplyError::Invalid("too many W-04 rounds".into()))?;
                sleep_until(start + update_period.saturating_mul(multiplier)).await;
                let percent = u8::try_from((round * 100) / (rounds + 1))
                    .map_err(|_| ApplyError::Invalid("W-04 progress conversion failed".into()))?;
                let run_id = current
                    .run_id
                    .clone()
                    .ok_or_else(|| ApplyError::Invalid("W-04 background Task lost run identity".into()))?;
                let source_revision = u64::try_from(round + 1)
                    .map_err(|_| ApplyError::Invalid("W-04 source revision overflow".into()))?;
                current = authority
                    .apply(TaskCommand {
                        command_id: format!("w04-event-{}-{source_revision}", current.task_id),
                        task_id: current.task_id.clone(),
                        expected_revision: current.revision,
                        op: TaskOp::ApplyObservation {
                            observation: AgentObservation {
                                run_id,
                                source_revision,
                                kind: ObservationKind::Progress { percent },
                            },
                        },
                    })
                    .await?;
                applied += 1;
            }
            Ok::<usize, ApplyError>(applied)
        }));
    }

    let mut updates_applied = 0usize;
    for worker in workers {
        updates_applied += worker
            .await
            .map_err(|error| ApplyError::Storage(format!("W-04 workload task join failed: {error}")))??;
    }

    Ok(BackgroundLoadReport {
        active_tasks: config.active_tasks,
        rounds: config.rounds,
        update_period_ms: config.update_period.as_millis(),
        updates_applied,
        elapsed_ms: start.elapsed().as_millis(),
        // Background workload execution alone is not the W-04 representative metric.
        w04_metric_eligible: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{repository::Repository, task::{PerTaskSupervisors, SharedTaskService}};
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn rejects_noncanonical_background_task_counts() {
        let file = NamedTempFile::new().unwrap();
        let authority: Arc<dyn TaskAuthority> =
            Arc::new(SharedTaskService::new(Repository::open(file.path()).unwrap()));
        let result = run_background_load(
            authority,
            BackgroundLoadConfig {
                active_tasks: 2,
                update_period: Duration::from_millis(1),
                rounds: 1,
            },
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn drives_same_fixed_load_for_both_task_candidates() {
        for per_task in [false, true] {
            let file = NamedTempFile::new().unwrap();
            let repository = Repository::open(file.path()).unwrap();
            let authority: Arc<dyn TaskAuthority> = if per_task {
                Arc::new(PerTaskSupervisors::new(repository.clone()))
            } else {
                Arc::new(SharedTaskService::new(repository.clone()))
            };
            let report = run_background_load(
                authority,
                BackgroundLoadConfig {
                    active_tasks: 4,
                    update_period: Duration::from_millis(2),
                    rounds: 3,
                },
            )
            .await
            .unwrap();
            assert_eq!(report.updates_applied, 12);
            assert!(!report.w04_metric_eligible);
            for index in 0..4 {
                let task = repository.get(&format!("W04-BG-{index}")).await.unwrap();
                assert_eq!(task.revision, 5);
            }
        }
    }
}
