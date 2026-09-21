use crate::{
    agent::AgentBoundary,
    repository::ApplyError,
    task::TaskAuthority,
};
use gate2_contracts::{AgentObservation, ObservationKind, TaskCommand, TaskState, TaskView};

pub struct AgentSynchronizer<B> {
    boundary: B,
}

impl<B> AgentSynchronizer<B> {
    pub fn new(boundary: B) -> Self {
        Self { boundary }
    }
}

impl<B: AgentBoundary> AgentSynchronizer<B> {
    pub async fn ingest_event(
        &self,
        authority: &dyn TaskAuthority,
        current: TaskView,
        observation: AgentObservation,
    ) -> Result<TaskView, ApplyError> {
        if current.run_id.as_deref() != Some(observation.run_id.as_str()) {
            return Err(ApplyError::Invalid(
                "Agent event belongs to a different execution".into(),
            ));
        }
        let command_id = format!(
            "agent-event:{}:{}",
            observation.run_id, observation.source_revision
        );
        authority
            .apply(TaskCommand {
                command_id,
                task_id: current.task_id,
                expected_revision: current.revision,
                op: gate2_contracts::TaskOp::ApplyObservation { observation },
            })
            .await
    }

    pub async fn reconcile_query(
        &self,
        authority: &dyn TaskAuthority,
        current: TaskView,
    ) -> Result<TaskView, anyhow::Error> {
        let Some(run_id) = current.run_id.clone() else {
            return Ok(current);
        };
        let snapshot = self.boundary.query(&run_id).await?;
        if snapshot.source_revision == 0 {
            anyhow::bail!("Agent snapshot must expose a positive source revision");
        }

        let kind = match snapshot.state.as_str() {
            "completed" => {
                let artifact = snapshot
                    .artifact
                    .ok_or_else(|| anyhow::anyhow!("completed snapshot missing artifact"))?;
                ObservationKind::Result { artifact }
            }
            "cancelled" => ObservationKind::Cancelled,
            "failed" => ObservationKind::Failed {
                reason: snapshot.artifact.unwrap_or_else(|| "agent reported failure".into()),
            },
            "running" => {
                // A query can confirm freshness without inventing progress. Preserve state if
                // there is no more informative source observation.
                return Ok(current);
            }
            other => anyhow::bail!("unsupported snapshot state: {other}"),
        };

        let observation = AgentObservation {
            run_id: snapshot.run_id,
            source_revision: snapshot.source_revision,
            kind,
        };
        self.ingest_event(authority, current, observation)
            .await
            .map_err(Into::into)
    }

    pub fn needs_reconciliation_after_disconnect(current: &TaskView) -> bool {
        !matches!(
            current.state,
            TaskState::Completed | TaskState::Cancelled | TaskState::Failed
        ) && current.run_id.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        agent::EdgeNormalized,
        repository::Repository,
        task::SharedTaskService,
    };
    use gate2_contracts::{SubmitRequest, TaskOp};
    use gate2_fixture::{AgentShape, DeterministicAgent};
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn event_then_query_reconciliation_does_not_create_a_second_truth() {
        let file = NamedTempFile::new().unwrap();
        let repo = Repository::open(file.path()).unwrap();
        let authority = SharedTaskService::new(repo);
        let agent = DeterministicAgent::new(AgentShape::P);
        let accepted = match agent
            .submit(SubmitRequest {
                task_id: "T1".into(),
                submission_key: "K1".into(),
                goal: "demo".into(),
            })
            .await
            .unwrap()
        {
            gate2_contracts::NativeReply::P(gate2_contracts::PReply::Accepted { run_id }) => run_id,
            _ => panic!("expected P accepted"),
        };
        let created = authority
            .apply(TaskCommand {
                command_id: "create".into(),
                task_id: "T1".into(),
                expected_revision: 0,
                op: TaskOp::Create { goal: "demo".into() },
            })
            .await
            .unwrap();
        let running = authority
            .apply(TaskCommand {
                command_id: "accept".into(),
                task_id: "T1".into(),
                expected_revision: created.revision,
                op: TaskOp::AcceptExecution {
                    run_id: accepted.clone(),
                },
            })
            .await
            .unwrap();

        let sync = AgentSynchronizer::new(EdgeNormalized::new(agent));
        let progress = sync
            .ingest_event(
                &authority,
                running,
                AgentObservation {
                    run_id: accepted,
                    source_revision: 1,
                    kind: ObservationKind::Progress { percent: 25 },
                },
            )
            .await
            .unwrap();
        let reconciled = sync.reconcile_query(&authority, progress.clone()).await.unwrap();
        assert_eq!(reconciled, progress);
    }
}
