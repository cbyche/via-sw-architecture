use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Submitted,
    Running,
    AwaitingInput,
    CancelRequested,
    Cancelled,
    Completed,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskView {
    pub task_id: String,
    pub revision: u64,
    pub state: TaskState,
    pub run_id: Option<String>,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskCommand {
    pub command_id: String,
    pub task_id: String,
    pub expected_revision: u64,
    pub op: TaskOp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TaskOp {
    Create { goal: String },
    PrepareHandoff { submission_key: String, goal: String },
    ConfirmHandoff {
        submission_key: String,
        run_id: String,
        context_id: Option<String>,
    },
    AcceptExecution { run_id: String },
    ApplyObservation { observation: AgentObservation },
    Cancel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingHandoff {
    pub task_id: String,
    pub submission_key: String,
    pub goal: String,
    pub prepared_revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionLink {
    pub task_id: String,
    pub submission_key: String,
    pub run_id: String,
    pub context_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentObservation {
    pub run_id: String,
    pub source_revision: u64,
    pub kind: ObservationKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ObservationKind {
    Progress { percent: u8 },
    Question { question_id: String, text: String },
    Result { artifact: String },
    Cancelled,
    Failed { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityProfile {
    pub query: bool,
    pub streaming: bool,
    pub follow_up: bool,
    pub cancel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubmitRequest {
    pub task_id: String,
    pub submission_key: String,
    pub goal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalAccepted {
    pub run_id: String,
    pub context_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NativeReply {
    P(PReply),
    Q(QReply),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PReply {
    Accepted {
        run_id: String,
    },
    Snapshot {
        run_id: String,
        revision: u64,
        state: String,
        artifact: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QReply {
    Accepted {
        context_id: String,
        run_id: String,
    },
    Snapshot {
        context_id: String,
        run_id: String,
        revision: u64,
        state: String,
        artifact: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum WorkerRequest {
    Submit { request: SubmitRequest },
    Query { run_id: String },
    AbortHost,
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum WorkerResponse {
    Reply { reply: NativeReply },
    Pong,
    Error { message: String },
}

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("run not found: {0}")]
    RunNotFound(String),
    #[error("unsupported capability: {0}")]
    Unsupported(&'static str),
    #[error("backend error: {0}")]
    Backend(String),
}

#[async_trait]
pub trait AgentBackend: Send + Sync {
    fn capabilities(&self) -> CapabilityProfile;
    async fn submit(&self, request: SubmitRequest) -> Result<NativeReply, AgentError>;
    async fn query(&self, run_id: &str) -> Result<NativeReply, AgentError>;
}
