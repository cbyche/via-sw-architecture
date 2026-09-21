use async_trait::async_trait;
use gate2_contracts::{
    AgentBackend, AgentError, CapabilityProfile, NativeReply, PReply, QReply, SubmitRequest,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    time::Duration,
};

#[derive(Debug, Clone, Copy)]
pub enum AgentShape {
    P,
    Q,
}

#[derive(Debug, Clone)]
struct Run {
    context_id: Option<String>,
    revision: u64,
    state: String,
    artifact: Option<String>,
}

pub struct DeterministicAgent {
    shape: AgentShape,
    sequence: AtomicU64,
    runs: Mutex<HashMap<String, Run>>,
}

impl DeterministicAgent {
    pub fn new(shape: AgentShape) -> Self {
        Self {
            shape,
            sequence: AtomicU64::new(1),
            runs: Mutex::new(HashMap::new()),
        }
    }

    pub fn complete(&self, run_id: &str, artifact: impl Into<String>) -> Result<(), AgentError> {
        let mut runs = self
            .runs
            .lock()
            .map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?;
        let run = runs
            .get_mut(run_id)
            .ok_or_else(|| AgentError::RunNotFound(run_id.to_owned()))?;
        run.revision += 1;
        run.state = "completed".into();
        run.artifact = Some(artifact.into());
        Ok(())
    }
}

#[async_trait]
impl AgentBackend for DeterministicAgent {
    fn capabilities(&self) -> CapabilityProfile {
        CapabilityProfile {
            query: true,
            streaming: true,
            follow_up: true,
            cancel: true,
        }
    }

    async fn submit(&self, request: SubmitRequest) -> Result<NativeReply, AgentError> {
        let n = self.sequence.fetch_add(1, Ordering::SeqCst);
        let run_id = format!("run-{n:04}");
        let context_id = match self.shape {
            AgentShape::P => None,
            AgentShape::Q => Some(format!("ctx-{}", request.task_id)),
        };
        self.runs
            .lock()
            .map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?
            .insert(
                run_id.clone(),
                Run {
                    context_id: context_id.clone(),
                    revision: 1,
                    state: "running".into(),
                    artifact: None,
                },
            );
        Ok(match self.shape {
            AgentShape::P => NativeReply::P(PReply::Accepted { run_id }),
            AgentShape::Q => NativeReply::Q(QReply::Accepted {
                context_id: context_id.expect("Q context"),
                run_id,
            }),
        })
    }

    async fn query(&self, run_id: &str) -> Result<NativeReply, AgentError> {
        let run = self
            .runs
            .lock()
            .map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?
            .get(run_id)
            .cloned()
            .ok_or_else(|| AgentError::RunNotFound(run_id.to_owned()))?;
        Ok(match self.shape {
            AgentShape::P => NativeReply::P(PReply::Snapshot {
                run_id: run_id.to_owned(),
                revision: run.revision,
                state: run.state,
                artifact: run.artifact,
            }),
            AgentShape::Q => NativeReply::Q(QReply::Snapshot {
                context_id: run.context_id.expect("Q context"),
                run_id: run_id.to_owned(),
                revision: run.revision,
                state: run.state,
                artifact: run.artifact,
            }),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S2sDelayTrace {
    pub evidence_level: String,
    pub source: String,
    pub samples_ms: Vec<u64>,
}

impl S2sDelayTrace {
    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let bytes = std::fs::read(path)?;
        let trace: Self = serde_json::from_slice(&bytes)?;
        anyhow::ensure!(!trace.samples_ms.is_empty(), "S2S delay trace must contain samples");
        Ok(trace)
    }

    pub fn delay_for(&self, trial: usize) -> Duration {
        Duration::from_millis(self.samples_ms[trial % self.samples_ms.len()])
    }

    pub async fn replay(&self, trial: usize) -> Duration {
        let delay = self.delay_for(trial);
        tokio::time::sleep(delay).await;
        delay
    }

    pub fn is_evaluation_eligible(&self) -> bool {
        matches!(
            self.evidence_level.as_str(),
            "MEASURED_S2S" | "FROZEN_REPLAY_FROM_MEASURED_S2S"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn p_and_q_keep_native_identity_shapes() {
        let p = DeterministicAgent::new(AgentShape::P);
        let q = DeterministicAgent::new(AgentShape::Q);
        let request = SubmitRequest {
            task_id: "T1".into(),
            submission_key: "K1".into(),
            goal: "demo".into(),
        };
        assert!(matches!(
            p.submit(request.clone()).await.unwrap(),
            NativeReply::P(PReply::Accepted { .. })
        ));
        assert!(matches!(
            q.submit(request).await.unwrap(),
            NativeReply::Q(QReply::Accepted { .. })
        ));
    }

    #[test]
    fn synthetic_s2s_trace_is_not_evaluation_evidence() {
        let trace = S2sDelayTrace {
            evidence_level: "TEST_ONLY".into(),
            source: "smoke".into(),
            samples_ms: vec![1, 2, 3],
        };
        assert!(!trace.is_evaluation_eligible());
        assert_eq!(trace.delay_for(4), Duration::from_millis(2));
    }
}
