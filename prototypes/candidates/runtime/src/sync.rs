use crate::{agent::AgentBoundary, repository::ApplyError, task::TaskAuthority};
use via_contracts::{AgentObservation, ObservationKind, TaskCommand, TaskState, TaskView};

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
                op: via_contracts::TaskOp::ApplyObservation { observation },
            })
            .await
    }

    pub async fn consume_events_since(
        &self,
        authority: &dyn TaskAuthority,
        mut current: TaskView,
        after_revision: u64,
    ) -> Result<(TaskView, u64), anyhow::Error> {
        let Some(run_id) = current.run_id.clone() else {
            return Ok((current, after_revision));
        };
        let observations = self.boundary.events_since(&run_id, after_revision).await?;
        let mut watermark = after_revision;
        for observation in observations {
            if observation.run_id != run_id {
                anyhow::bail!("Agent event stream crossed execution identity");
            }
            if observation.source_revision <= watermark {
                anyhow::bail!("Agent event stream is not strictly revision ordered");
            }
            watermark = observation.source_revision;
            current = self.ingest_event(authority, current, observation).await?;
        }
        Ok((current, watermark))
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
                reason: snapshot
                    .artifact
                    .unwrap_or_else(|| "agent reported failure".into()),
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
    use crate::{agent::EdgeNormalized, repository::Repository, task::SharedTaskService};
    use tempfile::NamedTempFile;
    use via_contracts::{AgentBackend, SubmitRequest, TaskOp};
    use via_fixture::{AgentShape, DeterministicAgent};

    #[tokio::test]
    async fn event_first_consumes_provider_specific_stream_through_boundary() {
        for shape in [AgentShape::P, AgentShape::Q] {
            let file = NamedTempFile::new().unwrap();
            let repo = Repository::open(file.path()).unwrap();
            let authority = SharedTaskService::new(repo);
            let agent = DeterministicAgent::new(shape);
            let accepted = match agent
                .submit(SubmitRequest {
                    task_id: "T1".into(),
                    submission_key: "K1".into(),
                    goal: "demo".into(),
                })
                .await
                .unwrap()
            {
                via_contracts::NativeReply::P(via_contracts::PReply::Accepted { run_id }) => run_id,
                via_contracts::NativeReply::Q(via_contracts::QReply::Accepted {
                    run_id, ..
                }) => run_id,
                _ => panic!("accepted expected"),
            };
            let created = authority
                .apply(TaskCommand {
                    command_id: "create".into(),
                    task_id: "T1".into(),
                    expected_revision: 0,
                    op: TaskOp::Create {
                        goal: "demo".into(),
                    },
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

            agent.emit_progress(&accepted, 40).unwrap();
            let sync = AgentSynchronizer::new(EdgeNormalized::new(agent));
            let (updated, watermark) = sync
                .consume_events_since(&authority, running, 1)
                .await
                .unwrap();
            assert_eq!(updated.state, TaskState::Running);
            assert_eq!(watermark, 2);
        }
    }

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
            via_contracts::NativeReply::P(via_contracts::PReply::Accepted { run_id }) => run_id,
            _ => panic!("expected P accepted"),
        };
        let created = authority
            .apply(TaskCommand {
                command_id: "create".into(),
                task_id: "T1".into(),
                expected_revision: 0,
                op: TaskOp::Create {
                    goal: "demo".into(),
                },
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
        let reconciled = sync
            .reconcile_query(&authority, progress.clone())
            .await
            .unwrap();
        assert_eq!(reconciled, progress);
    }
}
