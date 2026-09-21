use gate2_contracts::{ObservationKind, TaskCommand, TaskOp, TaskState, TaskView};
use rusqlite::{params, Connection, OptionalExtension};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplyError {
    #[error("task not found: {0}")]
    NotFound(String),
    #[error("task already exists: {0}")]
    AlreadyExists(String),
    #[error("revision conflict for {task_id}: expected {expected}, actual {actual}")]
    Conflict {
        task_id: String,
        expected: u64,
        actual: u64,
    },
    #[error("storage error: {0}")]
    Storage(String),
}

#[derive(Clone)]
pub struct Repository {
    connection: Arc<Mutex<Connection>>,
}

impl Repository {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ApplyError> {
        let connection = Connection::open(path).map_err(storage)?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(storage)?;
        connection
            .pragma_update(None, "synchronous", "FULL")
            .map_err(storage)?;
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS tasks (
                    task_id TEXT PRIMARY KEY,
                    revision INTEGER NOT NULL,
                    state_json TEXT NOT NULL,
                    run_id TEXT,
                    result TEXT
                 );
                 CREATE TABLE IF NOT EXISTS commands (
                    command_id TEXT PRIMARY KEY,
                    task_id TEXT NOT NULL,
                    result_json TEXT NOT NULL
                 );",
            )
            .map_err(storage)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub async fn apply(&self, command: TaskCommand) -> Result<TaskView, ApplyError> {
        let connection = self.connection.clone();
        tokio::task::spawn_blocking(move || apply_blocking(connection, command))
            .await
            .map_err(|e| ApplyError::Storage(format!("join error: {e}")))?
    }

    pub async fn get(&self, task_id: &str) -> Result<TaskView, ApplyError> {
        let connection = self.connection.clone();
        let task_id = task_id.to_owned();
        tokio::task::spawn_blocking(move || {
            let conn = connection
                .lock()
                .map_err(|_| ApplyError::Storage("poisoned DB lock".into()))?;
            load_task(&conn, &task_id)?.ok_or_else(|| ApplyError::NotFound(task_id))
        })
        .await
        .map_err(|e| ApplyError::Storage(format!("join error: {e}")))?
    }
}

fn apply_blocking(
    connection: Arc<Mutex<Connection>>,
    command: TaskCommand,
) -> Result<TaskView, ApplyError> {
    let mut conn = connection
        .lock()
        .map_err(|_| ApplyError::Storage("poisoned DB lock".into()))?;
    let tx = conn.transaction().map_err(storage)?;

    if let Some(json) = tx
        .query_row(
            "SELECT result_json FROM commands WHERE command_id=?1",
            params![command.command_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(storage)?
    {
        return serde_json::from_str(&json).map_err(|e| ApplyError::Storage(e.to_string()));
    }

    let next = match command.op {
        TaskOp::Create { .. } => {
            if load_task(&tx, &command.task_id)?.is_some() {
                return Err(ApplyError::AlreadyExists(command.task_id));
            }
            TaskView {
                task_id: command.task_id.clone(),
                revision: 1,
                state: TaskState::Submitted,
                run_id: None,
                result: None,
            }
        }
        op => {
            let current = load_task(&tx, &command.task_id)?
                .ok_or_else(|| ApplyError::NotFound(command.task_id.clone()))?;
            if current.revision != command.expected_revision {
                return Err(ApplyError::Conflict {
                    task_id: command.task_id.clone(),
                    expected: command.expected_revision,
                    actual: current.revision,
                });
            }
            transition(current, op)
        }
    };

    let state_json =
        serde_json::to_string(&next.state).map_err(|e| ApplyError::Storage(e.to_string()))?;
    tx.execute(
        "INSERT INTO tasks(task_id, revision, state_json, run_id, result)
         VALUES(?1,?2,?3,?4,?5)
         ON CONFLICT(task_id) DO UPDATE SET
           revision=excluded.revision,
           state_json=excluded.state_json,
           run_id=excluded.run_id,
           result=excluded.result",
        params![
            next.task_id,
            next.revision,
            state_json,
            next.run_id,
            next.result
        ],
    )
    .map_err(storage)?;

    let result_json =
        serde_json::to_string(&next).map_err(|e| ApplyError::Storage(e.to_string()))?;
    tx.execute(
        "INSERT INTO commands(command_id, task_id, result_json) VALUES(?1,?2,?3)",
        params![command.command_id, command.task_id, result_json],
    )
    .map_err(storage)?;
    tx.commit().map_err(storage)?;
    Ok(next)
}

fn transition(mut current: TaskView, op: TaskOp) -> TaskView {
    current.revision += 1;
    match op {
        TaskOp::AcceptExecution { run_id } => {
            current.run_id = Some(run_id);
            current.state = TaskState::Running;
        }
        TaskOp::ApplyObservation { observation } => match observation.kind {
            ObservationKind::Progress { .. } => current.state = TaskState::Running,
            ObservationKind::Question { .. } => current.state = TaskState::AwaitingInput,
            ObservationKind::Result { artifact } => {
                current.state = TaskState::Completed;
                current.result = Some(artifact);
            }
            ObservationKind::Cancelled => current.state = TaskState::Cancelled,
            ObservationKind::Failed { reason } => {
                current.state = TaskState::Failed;
                current.result = Some(reason);
            }
        },
        TaskOp::Cancel => current.state = TaskState::CancelRequested,
        TaskOp::Create { .. } => unreachable!("create handled before transition"),
    }
    current
}

fn load_task(conn: &Connection, task_id: &str) -> Result<Option<TaskView>, ApplyError> {
    conn.query_row(
        "SELECT revision,state_json,run_id,result FROM tasks WHERE task_id=?1",
        params![task_id],
        |row| {
            let revision: u64 = row.get(0)?;
            let state_json: String = row.get(1)?;
            let state: TaskState = serde_json::from_str(&state_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            Ok(TaskView {
                task_id: task_id.to_owned(),
                revision,
                state,
                run_id: row.get(2)?,
                result: row.get(3)?,
            })
        },
    )
    .optional()
    .map_err(storage)
}

fn storage(error: rusqlite::Error) -> ApplyError {
    ApplyError::Storage(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gate2_contracts::{TaskCommand, TaskOp};
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn command_dedup_returns_same_revision() {
        let file = NamedTempFile::new().unwrap();
        let repo = Repository::open(file.path()).unwrap();
        let cmd = TaskCommand {
            command_id: "cmd-1".into(),
            task_id: "T1".into(),
            expected_revision: 0,
            op: TaskOp::Create {
                goal: "demo".into(),
            },
        };
        let a = repo.apply(cmd.clone()).await.unwrap();
        let b = repo.apply(cmd).await.unwrap();
        assert_eq!(a, b);
        assert_eq!(a.revision, 1);
    }
}
