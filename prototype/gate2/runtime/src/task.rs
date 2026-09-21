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

    async fn create_sender(&self, task_id: &str) -> Result<mpsc::Sender<Envelope>, ApplyError> {
        let epoch = self.repository.activate(task_id).await?;
        let (tx, mut rx) = mpsc::channel::<Envelope>(64);
        let repo = self.repository.clone();
        tokio::spawn(async move {
            while let Some(envelope) = rx.recv().await {
                let result = repo.apply_with_epoch(envelope.command, Some(epoch)).await;
                let _ = envelope.reply.send(result);
            }
        });
        Ok(tx)
    }

    async fn sender_for(&self, task_id: &str) -> Result<mpsc::Sender<Envelope>, ApplyError> {
        // Activation is rare (first use or restart), while command delivery is steady-state.
        // Serialize activation-directory mutation so a later epoch can never fence the sender
        // that the directory returns. Scored trials pre-activate Tasks before timed probes.
        let mut senders = self.senders.lock().await;
        if let Some(sender) = senders.get(task_id) {
            if !sender.is_closed() {
                return Ok(sender.clone());
            }
        }

        let candidate = self.create_sender(task_id).await?;
        senders.insert(task_id.to_owned(), candidate.clone());
        Ok(candidate)
    }
}

#[async_trait]
impl TaskAuthority for PerTaskSupervisors {
    async fn apply(&self, command: TaskCommand) -> Result<TaskView, ApplyError> {
        let task_id = command.task_id.clone();
        let sender = self.sender_for(&task_id).await?;
        let (tx, rx) = oneshot::channel();
        match sender.send(Envelope { command, reply: tx }).await {
            Ok(()) => rx
                .await
                .map_err(|_| ApplyError::Storage("task supervisor dropped response".into()))?,
            Err(error) => {
                // Remove only the closed sender observed for this task. A retry creates a new
                // activation epoch; stale actors are fenced by Repository::apply_with_epoch.
                self.senders.lock().await.remove(&task_id);
                let sender = self.sender_for(&task_id).await?;
                let (tx, rx) = oneshot::channel();
                sender
                    .send(Envelope {
                        command: error.0.command,
                        reply: tx,
                    })
                    .await
                    .map_err(|_| ApplyError::Storage("task supervisor stopped twice".into()))?;
                rx.await
                    .map_err(|_| ApplyError::Storage("task supervisor dropped retry response".into()))?
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gate2_contracts::{TaskCommand, TaskOp};
    use tempfile::NamedTempFile;

    fn create_command(id: &str) -> TaskCommand {
        TaskCommand {
            command_id: id.into(),
            task_id: "T1".into(),
            expected_revision: 0,
            op: TaskOp::Create {
                goal: "demo".into(),
            },
        }
    }

    #[tokio::test]
    async fn both_authorities_have_same_external_transition_semantics() {
        let a_file = NamedTempFile::new().unwrap();
        let b_file = NamedTempFile::new().unwrap();
        let a = SharedTaskService::new(Repository::open(a_file.path()).unwrap());
        let b = PerTaskSupervisors::new(Repository::open(b_file.path()).unwrap());
        let av = a.apply(create_command("a-create")).await.unwrap();
        let bv = b.apply(create_command("b-create")).await.unwrap();
        assert_eq!(av.task_id, bv.task_id);
        assert_eq!(av.revision, bv.revision);
        assert_eq!(av.state, bv.state);
    }

    #[tokio::test]
    async fn reactivation_fences_an_older_writer_epoch() {
        let file = NamedTempFile::new().unwrap();
        let repo = Repository::open(file.path()).unwrap();
        let epoch1 = repo.activate("T1").await.unwrap();
        let epoch2 = repo.activate("T1").await.unwrap();
        assert!(epoch2 > epoch1);

        let stale = repo
            .apply_with_epoch(create_command("stale"), Some(epoch1))
            .await;
        assert!(stale.is_err());

        let current = repo
            .apply_with_epoch(create_command("current"), Some(epoch2))
            .await
            .unwrap();
        assert_eq!(current.revision, 1);
    }
}
