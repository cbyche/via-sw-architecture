use gate2_contracts::{ObservationKind, TaskCommand, TaskOp, TaskState, TaskView};
use rusqlite::{Connection, OptionalExtension, params};
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
    Conflict { task_id: String, expected: u64, actual: u64 },
    #[error("invalid command or state transition: {0}")]
    Invalid(String),
    #[error("storage error: {0}")]
    Storage(String),
}

#[derive(Clone)]
pub struct Repository { connection: Arc<Mutex<Connection>> }

impl Repository {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ApplyError> {
        let connection = Connection::open(path).map_err(storage)?;
        connection.pragma_update(None, "journal_mode", "WAL").map_err(storage)?;
        connection.pragma_update(None, "synchronous", "FULL").map_err(storage)?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks (
                task_id TEXT PRIMARY KEY, revision INTEGER NOT NULL,
                state_json TEXT NOT NULL, run_id TEXT, result TEXT
             );
             CREATE TABLE IF NOT EXISTS commands (
                command_id TEXT PRIMARY KEY, task_id TEXT NOT NULL, result_json TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS command_payloads (
                command_id TEXT PRIMARY KEY, payload TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS task_evidence (
                task_id TEXT PRIMARY KEY, source_revision INTEGER NOT NULL,
                observation_json TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS task_goals (
                task_id TEXT PRIMARY KEY, goal TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS task_owners (
                task_id TEXT PRIMARY KEY, epoch INTEGER NOT NULL
             );"
        ).map_err(storage)?;
        Ok(Self { connection: Arc::new(Mutex::new(connection)) })
    }

    pub async fn apply(&self, command: TaskCommand) -> Result<TaskView, ApplyError> {
        self.apply_with_epoch(command, None).await
    }

    pub async fn apply_with_epoch(&self, command: TaskCommand, epoch: Option<u64>) -> Result<TaskView, ApplyError> {
        let connection = self.connection.clone();
        tokio::task::spawn_blocking(move || apply_blocking(connection, command, epoch))
            .await.map_err(|e| ApplyError::Storage(format!("join error: {e}")))?
    }

    pub async fn activate(&self, task_id: &str) -> Result<u64, ApplyError> {
        let connection = self.connection.clone();
        let task_id = task_id.to_owned();
        tokio::task::spawn_blocking(move || {
            let mut conn = connection.lock().map_err(|_| ApplyError::Storage("poisoned DB lock".into()))?;
            let tx = conn.transaction().map_err(storage)?;
            tx.execute("INSERT INTO task_owners VALUES(?1,1) ON CONFLICT(task_id) DO UPDATE SET epoch=epoch+1", [&task_id]).map_err(storage)?;
            let epoch: i64 = tx.query_row("SELECT epoch FROM task_owners WHERE task_id=?1", [&task_id], |r| r.get(0)).map_err(storage)?;
            tx.commit().map_err(storage)?;
            u64::try_from(epoch).map_err(|_| ApplyError::Storage("invalid owner epoch".into()))
        }).await.map_err(|e| ApplyError::Storage(e.to_string()))?
    }

    pub async fn get(&self, task_id: &str) -> Result<TaskView, ApplyError> {
        let connection = self.connection.clone();
        let task_id = task_id.to_owned();
        tokio::task::spawn_blocking(move || {
            let conn = connection.lock().map_err(|_| ApplyError::Storage("poisoned DB lock".into()))?;
            load_task(&conn, &task_id)?.ok_or(ApplyError::NotFound(task_id))
        }).await.map_err(|e| ApplyError::Storage(format!("join error: {e}")))?
    }
}

fn invalid(message: &str) -> ApplyError { ApplyError::Invalid(message.into()) }
fn terminal(state: &TaskState) -> bool {
    matches!(state, TaskState::Completed | TaskState::Cancelled | TaskState::Failed)
}

fn apply_blocking(connection: Arc<Mutex<Connection>>, command: TaskCommand, epoch: Option<u64>) -> Result<TaskView, ApplyError> {
    if command.command_id.is_empty() || command.task_id.is_empty() { return Err(invalid("empty command/task ID")); }
    let payload = serde_json::to_string(&command).map_err(|e| ApplyError::Storage(e.to_string()))?;
    let mut conn = connection.lock().map_err(|_| ApplyError::Storage("poisoned DB lock".into()))?;
    let tx = conn.transaction().map_err(storage)?;
    if let Some(epoch) = epoch {
        let actual: Option<i64> = tx.query_row("SELECT epoch FROM task_owners WHERE task_id=?1", [&command.task_id], |r| r.get(0)).optional().map_err(storage)?;
        if actual != i64::try_from(epoch).ok() { return Err(invalid("stale Task activation writer")); }
    }
    let prior: Option<(String, Option<String>)> = tx.query_row(
        "SELECT c.result_json,p.payload FROM commands c LEFT JOIN command_payloads p USING(command_id) WHERE c.command_id=?1",
        [&command.command_id], |r| Ok((r.get(0)?, r.get(1)?))
    ).optional().map_err(storage)?;
    if let Some((json, original)) = prior {
        if original.as_deref() != Some(payload.as_str()) {
            return Err(invalid("command ID reused with different payload, or legacy dedup record lacks proof"));
        }
        return serde_json::from_str(&json).map_err(|e| ApplyError::Storage(e.to_string()));
    }
    let current = load_task(&tx, &command.task_id)?;
    let actual = current.as_ref().map_or(0, |t| t.revision);
    if actual != command.expected_revision {
        return Err(ApplyError::Conflict { task_id: command.task_id.clone(), expected: command.expected_revision, actual });
    }
    let mut new_evidence: Option<(u64, String)> = None;
    let next = match (&command.op, current) {
        (TaskOp::Create { goal }, None) => {
            if goal.trim().is_empty() { return Err(invalid("empty goal")); }
            tx.execute("INSERT INTO task_goals VALUES(?1,?2)", params![command.task_id,goal]).map_err(storage)?;
            TaskView { task_id: command.task_id.clone(), revision: 1, state: TaskState::Submitted, run_id: None, result: None }
        }
        (TaskOp::Create { .. }, Some(_)) => return Err(ApplyError::AlreadyExists(command.task_id)),
        (_, None) => return Err(ApplyError::NotFound(command.task_id)),
        (op, Some(current)) => {
            let mut next = current.clone();
            let mut mutate = true;
            match op {
                TaskOp::AcceptExecution { run_id } => {
                    if run_id.is_empty() { return Err(invalid("empty execution ID")); }
                    if let Some(existing) = &current.run_id {
                        if existing != run_id { return Err(invalid("execution rebind needs a new explicit lifecycle command")); }
                        mutate = false;
                    } else {
                        if terminal(&current.state) { return Err(invalid("cannot bind execution to terminal Task")); }
                        next.run_id = Some(run_id.clone());
                        if current.state != TaskState::CancelRequested { next.state = TaskState::Running; }
                    }
                }
                TaskOp::Cancel => {
                    if terminal(&current.state) || current.state == TaskState::CancelRequested { mutate = false; }
                    else { next.state = TaskState::CancelRequested; }
                }
                TaskOp::ApplyObservation { observation } => {
                    if current.run_id.as_deref() != Some(observation.run_id.as_str()) { return Err(invalid("event belongs to a different execution")); }
                    if matches!(observation.kind, ObservationKind::Progress { percent } if percent > 100) { return Err(invalid("progress outside 0..100")); }
                    if matches!(&observation.kind, ObservationKind::Question { question_id, text } if question_id.is_empty() || text.is_empty()) { return Err(invalid("question identity/text missing")); }
                    let serialized = serde_json::to_string(observation).map_err(|e| ApplyError::Storage(e.to_string()))?;
                    let prior: Option<(i64,String)> = tx.query_row("SELECT source_revision,observation_json FROM task_evidence WHERE task_id=?1", [&command.task_id], |r| Ok((r.get(0)?,r.get(1)?))).optional().map_err(storage)?;
                    if let Some((revision, original)) = prior {
                        let revision = u64::try_from(revision).map_err(|_| invalid("invalid stored source revision"))?;
                        if observation.source_revision < revision { mutate = false; }
                        if observation.source_revision == revision {
                            if original != serialized { return Err(invalid("same source revision with conflicting observation")); }
                            mutate = false;
                        }
                    }
                    if mutate {
                        match &observation.kind {
                            ObservationKind::Progress { .. } => {
                                if terminal(&current.state) { return Err(invalid("terminal state cannot return to running")); }
                                if !matches!(current.state, TaskState::CancelRequested | TaskState::AwaitingInput) { next.state = TaskState::Running; }
                            }
                            ObservationKind::Question { .. } => {
                                if terminal(&current.state) { return Err(invalid("terminal Task cannot request input")); }
                                if current.state != TaskState::CancelRequested { next.state = TaskState::AwaitingInput; }
                            }
                            ObservationKind::Result { artifact } => {
                                if artifact.is_empty() { return Err(invalid("empty result artifact")); }
                                next.state = TaskState::Completed; next.result = Some(artifact.clone());
                            }
                            ObservationKind::Cancelled => next.state = TaskState::Cancelled,
                            ObservationKind::Failed { reason } => { next.state = TaskState::Failed; next.result = Some(reason.clone()); }
                        }
                        if terminal(&current.state) && (next.state != current.state || next.result != current.result) { return Err(invalid("conflicting terminal outcome")); }
                        new_evidence = Some((observation.source_revision,serialized));
                    }
                }
                TaskOp::Create { .. } => unreachable!("handled above"),
            }
            if mutate { next.revision = current.revision.checked_add(1).ok_or_else(|| invalid("revision overflow"))?; }
            next
        }
    };
    let state_json = serde_json::to_string(&next.state).map_err(|e| ApplyError::Storage(e.to_string()))?;
    let revision = i64::try_from(next.revision).map_err(|_| invalid("revision overflow"))?;
    tx.execute("INSERT INTO tasks VALUES(?1,?2,?3,?4,?5) ON CONFLICT(task_id) DO UPDATE SET revision=excluded.revision,state_json=excluded.state_json,run_id=excluded.run_id,result=excluded.result",
        params![next.task_id,revision,state_json,next.run_id,next.result]).map_err(storage)?;
    if let Some((revision,json)) = new_evidence {
        let revision = i64::try_from(revision).map_err(|_| invalid("source revision overflow"))?;
        tx.execute("INSERT INTO task_evidence VALUES(?1,?2,?3) ON CONFLICT(task_id) DO UPDATE SET source_revision=excluded.source_revision,observation_json=excluded.observation_json", params![command.task_id,revision,json]).map_err(storage)?;
    }
    let result = serde_json::to_string(&next).map_err(|e| ApplyError::Storage(e.to_string()))?;
    tx.execute("INSERT INTO commands VALUES(?1,?2,?3)",params![command.command_id,command.task_id,result]).map_err(storage)?;
    tx.execute("INSERT INTO command_payloads VALUES(?1,?2)",params![command.command_id,payload]).map_err(storage)?;
    tx.commit().map_err(storage)?;
    Ok(next)
}

fn load_task(conn: &Connection, task_id: &str) -> Result<Option<TaskView>, ApplyError> {
    conn.query_row("SELECT revision,state_json,run_id,result FROM tasks WHERE task_id=?1", [task_id], |row| {
        let raw: i64 = row.get(0)?;
        let revision = u64::try_from(raw).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0,rusqlite::types::Type::Integer,Box::new(e)))?;
        let json: String = row.get(1)?;
        let state = serde_json::from_str(&json).map_err(|e| rusqlite::Error::FromSqlConversionFailure(1,rusqlite::types::Type::Text,Box::new(e)))?;
        Ok(TaskView { task_id:task_id.into(),revision,state,run_id:row.get(2)?,result:row.get(3)? })
    }).optional().map_err(storage)
}
fn storage(error: rusqlite::Error) -> ApplyError { ApplyError::Storage(error.to_string()) }
