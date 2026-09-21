use crate::repository::{ApplyError, Repository};
use async_trait::async_trait;
use gate2_contracts::{TaskCommand, TaskView};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, oneshot, Mutex};

#[async_trait]
pub trait TaskAuthority: Send + Sync {
    async fn apply(&self, command: TaskCommand) -> Result<TaskView, ApplyError>;
}

#[derive(Clone)]
pub struct SharedTaskService {
    repository: Repository,
}

impl SharedTaskService {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl TaskAuthority for SharedTaskService {
    async fn apply(&self, command: TaskCommand) -> Result<TaskView, ApplyError> {
        self.repository.apply(command).await
    }
}

struct Envelope {
    command: TaskCommand,
    reply: oneshot::Sender<Result<TaskView, ApplyError>>,
}

#[derive(Clone)]
pub struct PerTaskSupervisors {
    repository: Repository,
    senders: Arc<Mutex<HashMap<String, mpsc::Sender<Envelope>>>>,
}

impl PerTaskSupervisors {
    pub fn new(repository: Repository) -> Self {
        Self {
            repository,
            senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn sender_for(&self, task_id: &str) -> mpsc::Sender<Envelope> {
        let mut senders = self.senders.lock().await;
        if let Some(sender) = senders.get(task_id) {
            return sender.clone();
        }

        let (tx, mut rx) = mpsc::channel::<Envelope>(64);
        let repo = self.repository.clone();
        tokio::spawn(async move {
            while let Some(envelope) = rx.recv().await {
                let result = repo.apply(envelope.command).await;
                let _ = envelope.reply.send(result);
            }
        });
        senders.insert(task_id.to_owned(), tx.clone());
        tx
    }
}

#[async_trait]
impl TaskAuthority for PerTaskSupervisors {
    async fn apply(&self, command: TaskCommand) -> Result<TaskView, ApplyError> {
        let sender = self.sender_for(&command.task_id).await;
        let (tx, rx) = oneshot::channel();
        sender
            .send(Envelope { command, reply: tx })
            .await
            .map_err(|_| ApplyError::Storage("task supervisor stopped".into()))?;
        rx.await
            .map_err(|_| ApplyError::Storage("task supervisor dropped response".into()))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gate2_contracts::{TaskCommand, TaskOp};
    use tempfile::NamedTempFile;

    async fn create(authority: &dyn TaskAuthority) -> TaskView {
        authority
            .apply(TaskCommand {
                command_id: "c1".into(),
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
    async fn both_authorities_have_same_external_transition_semantics() {
        let a_file = NamedTempFile::new().unwrap();
        let b_file = NamedTempFile::new().unwrap();
        let a = SharedTaskService::new(Repository::open(a_file.path()).unwrap());
        let b = PerTaskSupervisors::new(Repository::open(b_file.path()).unwrap());
        assert_eq!(create(&a).await, create(&b).await);
    }
}
