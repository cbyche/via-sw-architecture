use async_trait::async_trait;
use gate2_contracts::{
    AgentBackend, AgentError, CapabilityProfile, NativeReply, PReply, QReply, SubmitRequest,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, sync::Mutex, time::{Duration, Instant}};

#[derive(Debug, Clone, Copy)]
pub enum AgentShape { P, Q }

#[derive(Debug, Clone)]
struct Run {
    context_id: Option<String>,
    revision: u64,
    state: String,
    artifact: Option<String>,
}

#[derive(Default)]
struct FixtureState {
    runs: HashMap<String, Run>,
    submissions: HashMap<String, (SubmitRequest, String)>,
}

/// Test-only external behavior. Durability across *fixture* process termination is not claimed.
pub struct DeterministicAgent {
    shape: AgentShape,
    state: Mutex<FixtureState>,
}

impl DeterministicAgent {
    pub fn new(shape: AgentShape) -> Self {
        Self { shape, state: Mutex::new(FixtureState::default()) }
    }

    fn accepted(&self, run_id: String, context_id: Option<String>) -> NativeReply {
        match self.shape {
            AgentShape::P => NativeReply::P(PReply::Accepted { run_id }),
            AgentShape::Q => NativeReply::Q(QReply::Accepted {
                context_id: context_id.expect("Q fixture always has context"), run_id,
            }),
        }
    }

    pub fn lookup_submission(&self, key: &str) -> Result<NativeReply, AgentError> {
        let state = self.state.lock().map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?;
        let (_, run_id) = state.submissions.get(key).ok_or_else(|| AgentError::RunNotFound(key.into()))?;
        let run = state.runs.get(run_id).ok_or_else(|| AgentError::RunNotFound(run_id.clone()))?;
        Ok(self.accepted(run_id.clone(), run.context_id.clone()))
    }

    pub fn run_count(&self) -> Result<usize, AgentError> {
        Ok(self.state.lock().map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?.runs.len())
    }

    pub fn complete(&self, run_id: &str, artifact: impl Into<String>) -> Result<(), AgentError> {
        let artifact = artifact.into();
        if artifact.is_empty() { return Err(AgentError::Backend("empty artifact".into())); }
        let mut state = self.state.lock().map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?;
        let run = state.runs.get_mut(run_id).ok_or_else(|| AgentError::RunNotFound(run_id.into()))?;
        if run.state == "completed" {
            if run.artifact.as_ref() == Some(&artifact) { return Ok(()); }
            return Err(AgentError::Backend("conflicting terminal artifact".into()));
        }
        run.revision += 1;
        run.state = "completed".into();
        run.artifact = Some(artifact);
        Ok(())
    }
}

#[async_trait]
impl AgentBackend for DeterministicAgent {
    fn capabilities(&self) -> CapabilityProfile {
        // Do not advertise operations absent from AgentBackend and this fixture.
        CapabilityProfile { query: true, streaming: false, follow_up: false, cancel: false }
    }

    async fn submit(&self, request: SubmitRequest) -> Result<NativeReply, AgentError> {
        if request.submission_key.is_empty() || request.task_id.is_empty() || request.goal.trim().is_empty() {
            return Err(AgentError::Backend("empty submit identity/goal".into()));
        }
        let mut state = self.state.lock().map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?;
        if let Some((original, run_id)) = state.submissions.get(&request.submission_key) {
            if original != &request { return Err(AgentError::Backend("submission key reused with changed task/goal".into())); }
            let run = state.runs.get(run_id).ok_or_else(|| AgentError::RunNotFound(run_id.clone()))?;
            return Ok(self.accepted(run_id.clone(), run.context_id.clone()));
        }
        let run_id = format!("run-{:04}", state.runs.len() + 1);
        let context_id = match self.shape { AgentShape::P => None, AgentShape::Q => Some(format!("ctx-{}", request.task_id)) };
        state.runs.insert(run_id.clone(), Run { context_id: context_id.clone(), revision: 1, state: "running".into(), artifact: None });
        state.submissions.insert(request.submission_key.clone(), (request, run_id.clone()));
        Ok(self.accepted(run_id, context_id))
    }

    async fn query(&self, run_id: &str) -> Result<NativeReply, AgentError> {
        let run = self.state.lock().map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?
            .runs.get(run_id).cloned().ok_or_else(|| AgentError::RunNotFound(run_id.into()))?;
        Ok(match self.shape {
            AgentShape::P => NativeReply::P(PReply::Snapshot { run_id: run_id.into(), revision: run.revision, state: run.state, artifact: run.artifact }),
            AgentShape::Q => NativeReply::Q(QReply::Snapshot { context_id: run.context_id.expect("Q context"), run_id: run_id.into(), revision: run.revision, state: run.state, artifact: run.artifact }),
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
        anyhow::ensure!(!trace.source.trim().is_empty(), "S2S trace requires provenance");
        anyhow::ensure!(trace.samples_ms.iter().all(|v| *v <= 60_000), "smoke delay exceeds 60s safety bound");
        Ok(trace)
    }

    pub fn delay_for(&self, trial: usize) -> Duration {
        assert!(!self.samples_ms.is_empty(), "construct traces through from_path validation");
        Duration::from_millis(self.samples_ms[trial % self.samples_ms.len()])
    }

    pub async fn replay(&self, trial: usize) -> Duration {
        let delay = self.delay_for(trial);
        tokio::time::sleep(delay).await;
        delay
    }

    /// Returns scheduled and actual elapsed durations separately; neither is live S2S performance.
    pub async fn replay_observed(&self, trial: usize) -> (Duration, Duration) {
        let scheduled = self.delay_for(trial);
        let start = Instant::now();
        tokio::time::sleep(scheduled).await;
        (scheduled, start.elapsed())
    }

    pub fn is_evaluation_eligible(&self) -> bool {
        // A provenance label alone cannot approve a trace or produce a real-system score.
        // Even measured-source replay remains SIMULATED_E2E in the endpoint ledger.
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> SubmitRequest { SubmitRequest { task_id: "T1".into(), submission_key: "K1".into(), goal: "demo".into() } }

    #[tokio::test]
    async fn p_and_q_keep_native_identity_shapes() {
        assert!(matches!(DeterministicAgent::new(AgentShape::P).submit(request()).await.unwrap(), NativeReply::P(PReply::Accepted { .. })));
        assert!(matches!(DeterministicAgent::new(AgentShape::Q).submit(request()).await.unwrap(), NativeReply::Q(QReply::Accepted { .. })));
    }

    #[tokio::test]
    async fn exact_submission_retry_returns_same_execution() {
        for shape in [AgentShape::P, AgentShape::Q] {
            let agent = DeterministicAgent::new(shape);
            let first = agent.submit(request()).await.unwrap();
            assert_eq!(first, agent.submit(request()).await.unwrap());
            assert_eq!(first, agent.lookup_submission("K1").unwrap());
            assert_eq!(agent.run_count().unwrap(), 1);
        }
    }

    #[tokio::test]
    async fn same_key_different_goal_is_rejected() {
        let agent = DeterministicAgent::new(AgentShape::Q);
        agent.submit(request()).await.unwrap();
        let mut changed = request(); changed.goal = "different".into();
        assert!(agent.submit(changed).await.is_err());
        assert_eq!(agent.run_count().unwrap(), 1);
    }

    #[tokio::test]
    async fn independent_key_creates_independent_execution() {
        let agent = DeterministicAgent::new(AgentShape::P);
        let first = agent.submit(request()).await.unwrap();
        let mut next = request(); next.submission_key = "K2".into();
        assert_ne!(first, agent.submit(next).await.unwrap());
        assert_eq!(agent.run_count().unwrap(), 2);
    }

    #[tokio::test]
    async fn complete_is_idempotent_but_not_overwritable() {
        let agent = DeterministicAgent::new(AgentShape::P);
        let reply = agent.submit(request()).await.unwrap();
        let NativeReply::P(PReply::Accepted { run_id }) = reply else { panic!("P accepted expected") };
        agent.complete(&run_id, "ART1").unwrap();
        let before = agent.query(&run_id).await.unwrap();
        agent.complete(&run_id, "ART1").unwrap();
        assert_eq!(before, agent.query(&run_id).await.unwrap());
        assert!(agent.complete(&run_id, "ART2").is_err());
    }

    #[test]
    fn unsupported_operations_are_not_advertised() {
        let c = DeterministicAgent::new(AgentShape::P).capabilities();
        assert!(c.query); assert!(!c.streaming); assert!(!c.follow_up); assert!(!c.cancel);
    }

    #[test]
    fn synthetic_s2s_trace_is_not_evaluation_evidence() {
        let trace = S2sDelayTrace { evidence_level: "TEST_ONLY".into(), source: "smoke".into(), samples_ms: vec![1,2,3] };
        assert!(!trace.is_evaluation_eligible());
        assert_eq!(trace.delay_for(4), Duration::from_millis(2));
    }

    #[test]
    fn_measured_label_does_not_self_approve_replay() {
        let trace = S2sDelayTrace { evidence_level: "MEASURED_S2S".into(), source: "unverified".into(), samples_ms: vec![1] };
        assert!(!trace.is_evaluation_eligible());
    }

    #[tokio::test]
    async fn observed_replay_keeps_scheduled_and_elapsed_distinct() {
        let trace = S2sDelayTrace { evidence_level: "TEST_ONLY".into(), source: "smoke".into(), samples_ms: vec![1] };
        let (scheduled, elapsed) = trace.replay_observed(0).await;
        assert_eq!(scheduled, Duration::from_millis(1));
        assert!(elapsed >= scheduled);
    }
}
