use crate::{
    agent::AgentBoundary,
    repository::Repository,
    task::TaskAuthority,
};
use gate2_contracts::{
    CanonicalAccepted, ExecutionLink, SubmitRequest, TaskCommand, TaskOp, TaskView,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffResult {
    pub accepted: CanonicalAccepted,
    pub task: TaskView,
    pub link: ExecutionLink,
}

pub struct HandoffCoordinator<B> {
    boundary: B,
    repository: Repository,
}

impl<B> HandoffCoordinator<B> {
    pub fn new(boundary: B, repository: Repository) -> Self {
        Self { boundary, repository }
    }
}

impl<B: AgentBoundary> HandoffCoordinator<B> {
    pub async fn handoff(
        &self,
        authority: &dyn TaskAuthority,
        current: TaskView,
        goal: String,
        submission_key: String,
    ) -> anyhow::Result<HandoffResult> {
        let prepared = authority
            .apply(TaskCommand {
                command_id: format!("prepare-handoff:{submission_key}"),
                task_id: current.task_id.clone(),
                expected_revision: current.revision,
                op: TaskOp::PrepareHandoff {
                    submission_key: submission_key.clone(),
                    goal: goal.clone(),
                },
            })
            .await?;

        let accepted = self
            .boundary
            .submit(SubmitRequest {
                task_id: current.task_id.clone(),
                submission_key: submission_key.clone(),
                goal,
            })
            .await?;

        self.confirm(authority, prepared, submission_key, accepted)
            .await
    }

    pub async fn recover_pending(
        &self,
        authority: &dyn TaskAuthority,
    ) -> anyhow::Result<Vec<HandoffResult>> {
        let mut recovered = Vec::new();
        for pending in self.repository.pending_handoffs().await? {
            let current = self.repository.get(&pending.task_id).await?;
            let accepted = self
                .boundary
                .submit(SubmitRequest {
                    task_id: pending.task_id.clone(),
                    submission_key: pending.submission_key.clone(),
                    goal: pending.goal,
                })
                .await?;
            recovered.push(
                self.confirm(
                    authority,
                    current,
                    pending.submission_key,
                    accepted,
                )
                .await?,
            );
        }
        Ok(recovered)
    }

    async fn confirm(
        &self,
        authority: &dyn TaskAuthority,
        current: TaskView,
        submission_key: String,
        accepted: CanonicalAccepted,
    ) -> anyhow::Result<HandoffResult> {
        let task = authority
            .apply(TaskCommand {
                command_id: format!("confirm-handoff:{submission_key}"),
                task_id: current.task_id.clone(),
                expected_revision: current.revision,
                op: TaskOp::ConfirmHandoff {
                    submission_key: submission_key.clone(),
                    run_id: accepted.run_id.clone(),
                    context_id: accepted.context_id.clone(),
                },
            })
            .await?;
        let link = self
            .repository
            .execution_link(&task.task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("handoff completed without durable ExecutionLink"))?;
        Ok(HandoffResult {
            accepted,
            task,
            link,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        agent::EdgeNormalized,
        task::{PerTaskSupervisors, SharedTaskService},
    };
    use gate2_contracts::TaskOp;
    use gate2_fixture::{AgentShape, DeterministicAgent};
    use tempfile::NamedTempFile;

    async fn create(authority: &dyn TaskAuthority) -> TaskView {
        authority
            .apply(TaskCommand {
                command_id: "create".into(),
                task_id: "T1".into(),
                expected_revision: 0,
                op: TaskOp::Create {
                    goal: "demo".into(),
                },
            })
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn shared_service_handoff_ends_with_durable_link() {
        let file = NamedTempFile::new().unwrap();
        let repository = Repository::open(file.path()).unwrap();
        let authority = SharedTaskService::new(repository.clone());
        let current = create(&authority).await;
        let coordinator = HandoffCoordinator::new(
            EdgeNormalized::new(DeterministicAgent::new(AgentShape::Q)),
            repository.clone(),
        );

        let result = coordinator
            .handoff(&authority, current, "demo".into(), "K1".into())
            .await
            .unwrap();

        assert_eq!(result.task.run_id.as_deref(), Some(result.accepted.run_id.as_str()));
        assert_eq!(result.link.submission_key, "K1");
        assert!(repository.pending_handoffs().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn per_task_supervisor_uses_same_durable_handoff_contract() {
        let file = NamedTempFile::new().unwrap();
        let repository = Repository::open(file.path()).unwrap();
        let authority = PerTaskSupervisors::new(repository.clone());
        let current = create(&authority).await;
        let coordinator = HandoffCoordinator::new(
            EdgeNormalized::new(DeterministicAgent::new(AgentShape::P)),
            repository.clone(),
        );

        let result = coordinator
            .handoff(&authority, current, "demo".into(), "K1".into())
            .await
            .unwrap();

        assert_eq!(result.link.run_id, result.accepted.run_id);
        assert!(repository.pending_handoffs().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn accepted_before_local_commit_recovers_without_duplicate_execution() {
        let file = NamedTempFile::new().unwrap();
        let repository = Repository::open(file.path()).unwrap();
        let authority = SharedTaskService::new(repository.clone());
        let current = create(&authority).await;
        let agent = DeterministicAgent::new(AgentShape::P);
        let coordinator = HandoffCoordinator::new(EdgeNormalized::new(agent), repository.clone());

        let prepared = authority
            .apply(TaskCommand {
                command_id: "prepare-handoff:K1".into(),
                task_id: current.task_id.clone(),
                expected_revision: current.revision,
                op: TaskOp::PrepareHandoff {
                    submission_key: "K1".into(),
                    goal: "demo".into(),
                },
            })
            .await
            .unwrap();

        let first = coordinator
            .boundary
            .submit(SubmitRequest {
                task_id: "T1".into(),
                submission_key: "K1".into(),
                goal: "demo".into(),
            })
            .await
            .unwrap();
        let first_run = first.run_id.clone();

        // Simulate VIA memory loss after Agent acceptance but before local link commit.
        drop(authority);
        let reopened = Repository::open(file.path()).unwrap();
        let restarted_authority = SharedTaskService::new(reopened.clone());
        assert_eq!(reopened.get("T1").await.unwrap(), prepared);

        let recovered = coordinator
            .recover_pending(&restarted_authority)
            .await
            .unwrap();
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].accepted.run_id, first_run);
        assert_eq!(recovered[0].link.run_id, first_run);
        assert!(reopened.pending_handoffs().await.unwrap().is_empty());

        // Agent fixture is idempotent by submission key: retry must return the same run.
        let retried = coordinator
            .boundary
            .submit(SubmitRequest {
                task_id: "T1".into(),
                submission_key: "K1".into(),
                goal: "demo".into(),
            })
            .await
            .unwrap();
        assert_eq!(retried.run_id, first_run);
    }
}
