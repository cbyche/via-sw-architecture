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
pub struct CancelOutcome {
    pub run_id: String,
    pub requested: bool,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NativeEvent {
    P(PEvent),
    Q(QEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PEvent {
    pub run_id: String,
    pub revision: u64,
    pub kind: PEventKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PEventKind {
    Progress { percent: u8 },
    Question { question_id: String, text: String },
    Result { artifact: String },
    Cancelled,
    Failed { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QEvent {
    pub context_id: String,
    pub run_id: String,
    pub revision: u64,
    pub kind: QEventKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QEventKind {
    Running { progress_percent: Option<u8> },
    InputRequired { request_id: String, prompt: String },
    ArtifactReady { artifact_id: String },
    Cancelled,
    Failure { message: String },
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
    FollowUpAccepted {
        run_id: String,
        revision: u64,
    },
    CancelConfirmed {
        run_id: String,
        revision: u64,
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
    ContinuationAccepted {
        context_id: String,
        previous_run_id: String,
        run_id: String,
    },
    CancelRequested {
        context_id: String,
        run_id: String,
        revision: u64,
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
    FollowUp { run_id: String, text: String },
    Cancel { run_id: String },
    EventsSince { run_id: String, after_revision: u64 },
    AbortHost,
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum WorkerResponse {
    Reply { reply: NativeReply },
    Events { events: Vec<NativeEvent> },
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
    async fn follow_up(&self, run_id: &str, text: String) -> Result<NativeReply, AgentError>;
    async fn cancel(&self, run_id: &str) -> Result<NativeReply, AgentError>;
    async fn events_since(
        &self,
        run_id: &str,
        after_revision: u64,
    ) -> Result<Vec<NativeEvent>, AgentError>;
}
