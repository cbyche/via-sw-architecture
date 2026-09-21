use anyhow::{Context, anyhow};
use async_trait::async_trait;
use gate2_contracts::{
    AgentBackend, NativeEvent, NativeReply, SubmitRequest, WorkerRequest, WorkerResponse,
};
use std::{path::Path, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::Mutex,
};

#[async_trait]
pub trait IntegrationBridge: Send + Sync {
    async fn submit(&self, request: SubmitRequest) -> anyhow::Result<NativeReply>;
    async fn query(&self, run_id: &str) -> anyhow::Result<NativeReply>;
    async fn follow_up(&self, run_id: &str, text: String) -> anyhow::Result<NativeReply>;
    async fn cancel(&self, run_id: &str) -> anyhow::Result<NativeReply>;
    async fn events_since(
        &self,
        run_id: &str,
        after_revision: u64,
    ) -> anyhow::Result<Vec<NativeEvent>>;
}

pub struct LocalBridge {
    backend: Arc<dyn AgentBackend>,
}

impl LocalBridge {
    pub fn new(backend: Arc<dyn AgentBackend>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl IntegrationBridge for LocalBridge {
    async fn submit(&self, request: SubmitRequest) -> anyhow::Result<NativeReply> {
        self.backend.submit(request).await.map_err(Into::into)
    }

    async fn query(&self, run_id: &str) -> anyhow::Result<NativeReply> {
        self.backend.query(run_id).await.map_err(Into::into)
    }

    async fn follow_up(&self, run_id: &str, text: String) -> anyhow::Result<NativeReply> {
        self.backend.follow_up(run_id, text).await.map_err(Into::into)
    }

    async fn cancel(&self, run_id: &str) -> anyhow::Result<NativeReply> {
        self.backend.cancel(run_id).await.map_err(Into::into)
    }

    async fn events_since(
        &self,
        run_id: &str,
        after_revision: u64,
    ) -> anyhow::Result<Vec<NativeEvent>> {
        self.backend
            .events_since(run_id, after_revision)
            .await
            .map_err(Into::into)
    }
}

struct ProcessSession {
    child: Child,
    stdin: ChildStdin,
    stdout: Lines<BufReader<ChildStdout>>,
}

pub struct ProcessBridge {
    session: Mutex<ProcessSession>,
}

impl ProcessBridge {
    pub async fn spawn(worker: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut child = Command::new(worker.as_ref())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .with_context(|| format!("spawn integration worker {}", worker.as_ref().display()))?;
        let stdin = child.stdin.take().context("worker stdin")?;
        let stdout = child.stdout.take().context("worker stdout")?;
        Ok(Self {
            session: Mutex::new(ProcessSession {
                child,
                stdin,
                stdout: BufReader::new(stdout).lines(),
            }),
        })
    }

    async fn request(&self, request: WorkerRequest) -> anyhow::Result<WorkerResponse> {
        let mut session = self.session.lock().await;
        let line = serde_json::to_string(&request)?;
        session.stdin.write_all(line.as_bytes()).await?;
        session.stdin.write_all(b"\n").await?;
        session.stdin.flush().await?;
        let line = session
            .stdout
            .next_line()
            .await?
            .ok_or_else(|| anyhow!("integration worker closed stdout"))?;
        Ok(serde_json::from_str(&line)?)
    }

    pub async fn abort_host(&self) -> anyhow::Result<()> {
        let _ = self.request(WorkerRequest::AbortHost).await;
        let mut session = self.session.lock().await;
        let _ = session.child.wait().await;
        Ok(())
    }
}

#[async_trait]
impl IntegrationBridge for ProcessBridge {
    async fn submit(&self, request: SubmitRequest) -> anyhow::Result<NativeReply> {
        match self.request(WorkerRequest::Submit { request }).await? {
            WorkerResponse::Reply { reply } => Ok(reply),
            WorkerResponse::Error { message } => Err(anyhow!(message)),
            other => Err(anyhow!("unexpected worker response: {other:?}")),
        }
    }

    async fn query(&self, run_id: &str) -> anyhow::Result<NativeReply> {
        match self
            .request(WorkerRequest::Query {
                run_id: run_id.to_owned(),
            })
            .await?
        {
            WorkerResponse::Reply { reply } => Ok(reply),
            WorkerResponse::Error { message } => Err(anyhow!(message)),
            other => Err(anyhow!("unexpected worker response: {other:?}")),
        }
    }

    async fn follow_up(&self, run_id: &str, text: String) -> anyhow::Result<NativeReply> {
        match self
            .request(WorkerRequest::FollowUp {
                run_id: run_id.to_owned(),
                text,
            })
            .await?
        {
            WorkerResponse::Reply { reply } => Ok(reply),
            WorkerResponse::Error { message } => Err(anyhow!(message)),
            other => Err(anyhow!("unexpected worker response: {other:?}")),
        }
    }

    async fn cancel(&self, run_id: &str) -> anyhow::Result<NativeReply> {
        match self
            .request(WorkerRequest::Cancel {
                run_id: run_id.to_owned(),
            })
            .await?
        {
            WorkerResponse::Reply { reply } => Ok(reply),
            WorkerResponse::Error { message } => Err(anyhow!(message)),
            other => Err(anyhow!("unexpected worker response: {other:?}")),
        }
    }

    async fn events_since(
        &self,
        run_id: &str,
        after_revision: u64,
    ) -> anyhow::Result<Vec<NativeEvent>> {
        match self
            .request(WorkerRequest::EventsSince {
                run_id: run_id.to_owned(),
                after_revision,
            })
            .await?
        {
            WorkerResponse::Events { events } => Ok(events),
            WorkerResponse::Error { message } => Err(anyhow!(message)),
            other => Err(anyhow!("unexpected worker response: {other:?}")),
        }
    }
}
