use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use via_contracts::{
    AgentBackend, AgentError, CapabilityProfile, NativeEvent, NativeReply, PEvent, PEventKind,
    PReply, QEvent, QEventKind, QReply, SubmitRequest,
};

#[derive(Debug, Clone, Copy)]
pub enum AgentShape {
    P,
    Q,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Run {
    context_id: Option<String>,
    revision: u64,
    state: String,
    artifact: Option<String>,
    events: Vec<NativeEvent>,
}

#[derive(Default, Serialize, Deserialize)]
struct FixtureState {
    runs: HashMap<String, Run>,
    submissions: HashMap<String, (SubmitRequest, String)>,
}

/// Deterministic full-capability Agent fixture. P and Q provide the same user-visible
/// capability set but deliberately expose different native lifecycle shapes.
#[derive(Clone)]
pub struct DeterministicAgent {
    shape: AgentShape,
    state: Arc<Mutex<FixtureState>>,
    state_path: Option<Arc<PathBuf>>,
}

impl DeterministicAgent {
    pub fn new(shape: AgentShape) -> Self {
        Self {
            shape,
            state: Arc::new(Mutex::new(FixtureState::default())),
            state_path: None,
        }
    }

    /// Persistent fixture mode keeps downstream Agent execution state outside a
    /// VIA integration worker's volatile memory. It is test infrastructure, not a
    /// production Agent persistence design.
    pub fn persistent(shape: AgentShape, path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let state = if path.exists() {
            serde_json::from_slice(&std::fs::read(&path)?)?
        } else {
            FixtureState::default()
        };
        let agent = Self {
            shape,
            state: Arc::new(Mutex::new(state)),
            state_path: Some(Arc::new(path)),
        };
        {
            let state = agent
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("poisoned fixture lock"))?;
            agent
                .persist_locked(&state)
                .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        }
        Ok(agent)
    }

    fn persist_locked(&self, state: &FixtureState) -> Result<(), AgentError> {
        let Some(path) = &self.state_path else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| AgentError::Backend(error.to_string()))?;
        }
        let bytes =
            serde_json::to_vec(state).map_err(|error| AgentError::Backend(error.to_string()))?;
        let mut file = std::fs::File::create(path.as_ref())
            .map_err(|error| AgentError::Backend(error.to_string()))?;
        file.write_all(&bytes)
            .map_err(|error| AgentError::Backend(error.to_string()))?;
        file.sync_all()
            .map_err(|error| AgentError::Backend(error.to_string()))
    }

    fn accepted(&self, run_id: String, context_id: Option<String>) -> NativeReply {
        match self.shape {
            AgentShape::P => NativeReply::P(PReply::Accepted { run_id }),
            AgentShape::Q => NativeReply::Q(QReply::Accepted {
                context_id: context_id.expect("Q fixture always has context"),
                run_id,
            }),
        }
    }

    fn running_event(&self, run_id: &str, run: &Run, percent: Option<u8>) -> NativeEvent {
        match self.shape {
            AgentShape::P => NativeEvent::P(PEvent {
                run_id: run_id.into(),
                revision: run.revision,
                kind: PEventKind::Progress {
                    percent: percent.unwrap_or(0),
                },
            }),
            AgentShape::Q => NativeEvent::Q(QEvent {
                context_id: run.context_id.clone().expect("Q context"),
                run_id: run_id.into(),
                revision: run.revision,
                kind: QEventKind::Running {
                    progress_percent: percent,
                },
            }),
        }
    }

    pub fn lookup_submission(&self, key: &str) -> Result<NativeReply, AgentError> {
        let state = self
            .state
            .lock()
            .map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?;
        let (_, run_id) = state
            .submissions
            .get(key)
            .ok_or_else(|| AgentError::RunNotFound(key.into()))?;
        let run = state
            .runs
            .get(run_id)
            .ok_or_else(|| AgentError::RunNotFound(run_id.clone()))?;
        Ok(self.accepted(run_id.clone(), run.context_id.clone()))
    }

    pub fn run_count(&self) -> Result<usize, AgentError> {
        Ok(self
            .state
            .lock()
            .map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?
            .runs
            .len())
    }

    pub fn emit_progress(&self, run_id: &str, percent: u8) -> Result<(), AgentError> {
        if percent > 100 {
            return Err(AgentError::Backend("progress outside 0..100".into()));
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?;
        let run = state
            .runs
            .get_mut(run_id)
            .ok_or_else(|| AgentError::RunNotFound(run_id.into()))?;
        if run.state != "running" {
            return Err(AgentError::Backend(
                "progress requires running execution".into(),
            ));
        }
        run.revision += 1;
        let event = match self.shape {
            AgentShape::P => NativeEvent::P(PEvent {
                run_id: run_id.into(),
                revision: run.revision,
                kind: PEventKind::Progress { percent },
            }),
            AgentShape::Q => NativeEvent::Q(QEvent {
                context_id: run.context_id.clone().expect("Q context"),
                run_id: run_id.into(),
                revision: run.revision,
                kind: QEventKind::Running {
                    progress_percent: Some(percent),
                },
            }),
        };
        run.events.push(event);
        self.persist_locked(&state)?;
        Ok(())
    }

    pub fn ask(
        &self,
        run_id: &str,
        question_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<(), AgentError> {
        let question_id = question_id.into();
        let text = text.into();
        if question_id.is_empty() || text.is_empty() {
            return Err(AgentError::Backend("question identity/text missing".into()));
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?;
        let run = state
            .runs
            .get_mut(run_id)
            .ok_or_else(|| AgentError::RunNotFound(run_id.into()))?;
        run.revision += 1;
        let event = match self.shape {
            AgentShape::P => NativeEvent::P(PEvent {
                run_id: run_id.into(),
                revision: run.revision,
                kind: PEventKind::Question { question_id, text },
            }),
            AgentShape::Q => NativeEvent::Q(QEvent {
                context_id: run.context_id.clone().expect("Q context"),
                run_id: run_id.into(),
                revision: run.revision,
                kind: QEventKind::InputRequired {
                    request_id: question_id,
                    prompt: text,
                },
            }),
        };
        run.events.push(event);
        self.persist_locked(&state)?;
        Ok(())
    }

    pub fn complete(&self, run_id: &str, artifact: impl Into<String>) -> Result<(), AgentError> {
        let artifact = artifact.into();
        if artifact.is_empty() {
            return Err(AgentError::Backend("empty artifact".into()));
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?;
        let run = state
            .runs
            .get_mut(run_id)
            .ok_or_else(|| AgentError::RunNotFound(run_id.into()))?;
        if run.state == "completed" {
            if run.artifact.as_ref() == Some(&artifact) {
                return Ok(());
            }
            return Err(AgentError::Backend("conflicting terminal artifact".into()));
        }
        run.revision += 1;
        run.state = "completed".into();
        run.artifact = Some(artifact.clone());
        let event = match self.shape {
            AgentShape::P => NativeEvent::P(PEvent {
                run_id: run_id.into(),
                revision: run.revision,
                kind: PEventKind::Result { artifact },
            }),
            AgentShape::Q => NativeEvent::Q(QEvent {
                context_id: run.context_id.clone().expect("Q context"),
                run_id: run_id.into(),
                revision: run.revision,
                kind: QEventKind::ArtifactReady {
                    artifact_id: artifact,
                },
            }),
        };
        run.events.push(event);
        self.persist_locked(&state)?;
        Ok(())
    }

    pub fn confirm_cancel(&self, run_id: &str) -> Result<(), AgentError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?;
        let run = state
            .runs
            .get_mut(run_id)
            .ok_or_else(|| AgentError::RunNotFound(run_id.into()))?;
        if run.state == "cancelled" {
            return Ok(());
        }
        run.revision += 1;
        run.state = "cancelled".into();
        let event = match self.shape {
            AgentShape::P => NativeEvent::P(PEvent {
                run_id: run_id.into(),
                revision: run.revision,
                kind: PEventKind::Cancelled,
            }),
            AgentShape::Q => NativeEvent::Q(QEvent {
                context_id: run.context_id.clone().expect("Q context"),
                run_id: run_id.into(),
                revision: run.revision,
                kind: QEventKind::Cancelled,
            }),
        };
        run.events.push(event);
        self.persist_locked(&state)?;
        Ok(())
    }

    pub fn fail(&self, run_id: &str, reason: impl Into<String>) -> Result<(), AgentError> {
        let reason = reason.into();
        if reason.is_empty() {
            return Err(AgentError::Backend("empty failure reason".into()));
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?;
        let run = state
            .runs
            .get_mut(run_id)
            .ok_or_else(|| AgentError::RunNotFound(run_id.into()))?;
        run.revision += 1;
        run.state = "failed".into();
        run.artifact = Some(reason.clone());
        let event = match self.shape {
            AgentShape::P => NativeEvent::P(PEvent {
                run_id: run_id.into(),
                revision: run.revision,
                kind: PEventKind::Failed { reason },
            }),
            AgentShape::Q => NativeEvent::Q(QEvent {
                context_id: run.context_id.clone().expect("Q context"),
                run_id: run_id.into(),
                revision: run.revision,
                kind: QEventKind::Failure { message: reason },
            }),
        };
        run.events.push(event);
        self.persist_locked(&state)?;
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
        if request.submission_key.is_empty()
            || request.task_id.is_empty()
            || request.goal.trim().is_empty()
        {
            return Err(AgentError::Backend("empty submit identity/goal".into()));
        }

        let mut state = self
            .state
            .lock()
            .map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?;
        if let Some((original, run_id)) = state.submissions.get(&request.submission_key) {
            if original != &request {
                return Err(AgentError::Backend(
                    "submission key reused with changed task/goal".into(),
                ));
            }
            let run = state
                .runs
                .get(run_id)
                .ok_or_else(|| AgentError::RunNotFound(run_id.clone()))?;
            return Ok(self.accepted(run_id.clone(), run.context_id.clone()));
        }

        let run_id = format!("run-{:04}", state.runs.len() + 1);
        let context_id = match self.shape {
            AgentShape::P => None,
            AgentShape::Q => Some(format!("ctx-{}", request.task_id)),
        };
        let mut run = Run {
            context_id: context_id.clone(),
            revision: 1,
            state: "running".into(),
            artifact: None,
            events: Vec::new(),
        };
        run.events.push(self.running_event(&run_id, &run, Some(0)));
        state.runs.insert(run_id.clone(), run);
        state
            .submissions
            .insert(request.submission_key.clone(), (request, run_id.clone()));
        self.persist_locked(&state)?;
        Ok(self.accepted(run_id, context_id))
    }

    async fn query(&self, run_id: &str) -> Result<NativeReply, AgentError> {
        let run = self
            .state
            .lock()
            .map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?
            .runs
            .get(run_id)
            .cloned()
            .ok_or_else(|| AgentError::RunNotFound(run_id.into()))?;

        Ok(match self.shape {
            AgentShape::P => NativeReply::P(PReply::Snapshot {
                run_id: run_id.into(),
                revision: run.revision,
                state: run.state,
                artifact: run.artifact,
            }),
            AgentShape::Q => NativeReply::Q(QReply::Snapshot {
                context_id: run.context_id.expect("Q context"),
                run_id: run_id.into(),
                revision: run.revision,
                state: run.state,
                artifact: run.artifact,
            }),
        })
    }

    async fn follow_up(&self, run_id: &str, text: String) -> Result<NativeReply, AgentError> {
        if text.trim().is_empty() {
            return Err(AgentError::Backend("empty follow-up".into()));
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?;

        match self.shape {
            AgentShape::P => {
                let run = state
                    .runs
                    .get_mut(run_id)
                    .ok_or_else(|| AgentError::RunNotFound(run_id.into()))?;
                if run.state != "running" {
                    return Err(AgentError::Backend(
                        "P follow-up requires running execution".into(),
                    ));
                }
                run.revision += 1;
                let revision = run.revision;
                let event = NativeEvent::P(PEvent {
                    run_id: run_id.into(),
                    revision,
                    kind: PEventKind::Progress { percent: 0 },
                });
                run.events.push(event);
                self.persist_locked(&state)?;
                Ok(NativeReply::P(PReply::FollowUpAccepted {
                    run_id: run_id.into(),
                    revision,
                }))
            }
            AgentShape::Q => {
                let previous = state
                    .runs
                    .get(run_id)
                    .cloned()
                    .ok_or_else(|| AgentError::RunNotFound(run_id.into()))?;
                let context_id = previous.context_id.expect("Q context");
                let next_id = format!("run-{:04}", state.runs.len() + 1);
                let mut next = Run {
                    context_id: Some(context_id.clone()),
                    revision: 1,
                    state: "running".into(),
                    artifact: None,
                    events: Vec::new(),
                };
                next.events.push(NativeEvent::Q(QEvent {
                    context_id: context_id.clone(),
                    run_id: next_id.clone(),
                    revision: 1,
                    kind: QEventKind::Running {
                        progress_percent: Some(0),
                    },
                }));
                state.runs.insert(next_id.clone(), next);
                self.persist_locked(&state)?;
                Ok(NativeReply::Q(QReply::ContinuationAccepted {
                    context_id,
                    previous_run_id: run_id.into(),
                    run_id: next_id,
                }))
            }
        }
    }

    async fn cancel(&self, run_id: &str) -> Result<NativeReply, AgentError> {
        match self.shape {
            AgentShape::P => {
                self.confirm_cancel(run_id)?;
                let state = self
                    .state
                    .lock()
                    .map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?;
                let run = state
                    .runs
                    .get(run_id)
                    .ok_or_else(|| AgentError::RunNotFound(run_id.into()))?;
                Ok(NativeReply::P(PReply::CancelConfirmed {
                    run_id: run_id.into(),
                    revision: run.revision,
                }))
            }
            AgentShape::Q => {
                let mut state = self
                    .state
                    .lock()
                    .map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?;
                let run = state
                    .runs
                    .get_mut(run_id)
                    .ok_or_else(|| AgentError::RunNotFound(run_id.into()))?;
                if run.state != "running" {
                    return Err(AgentError::Backend(
                        "Q cancel request requires running execution".into(),
                    ));
                }
                run.revision += 1;
                let context_id = run.context_id.clone().expect("Q context");
                let revision = run.revision;
                self.persist_locked(&state)?;
                Ok(NativeReply::Q(QReply::CancelRequested {
                    context_id,
                    run_id: run_id.into(),
                    revision,
                }))
            }
        }
    }

    async fn events_since(
        &self,
        run_id: &str,
        after_revision: u64,
    ) -> Result<Vec<NativeEvent>, AgentError> {
        let state = self
            .state
            .lock()
            .map_err(|_| AgentError::Backend("poisoned fixture lock".into()))?;
        let run = state
            .runs
            .get(run_id)
            .ok_or_else(|| AgentError::RunNotFound(run_id.into()))?;
        Ok(run
            .events
            .iter()
            .filter(|event| match event {
                NativeEvent::P(event) => event.revision > after_revision,
                NativeEvent::Q(event) => event.revision > after_revision,
            })
            .cloned()
            .collect())
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
        let trace: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        anyhow::ensure!(
            !trace.samples_ms.is_empty(),
            "S2S delay trace must contain samples"
        );
        anyhow::ensure!(
            !trace.source.trim().is_empty(),
            "S2S trace requires provenance"
        );
        anyhow::ensure!(
            trace.samples_ms.iter().all(|v| *v <= 60_000),
            "smoke delay exceeds 60s safety bound"
        );
        Ok(trace)
    }

    pub fn delay_for(&self, trial: usize) -> Duration {
        assert!(
            !self.samples_ms.is_empty(),
            "construct traces through from_path validation"
        );
        Duration::from_millis(self.samples_ms[trial % self.samples_ms.len()])
    }

    pub async fn replay(&self, trial: usize) -> Duration {
        let delay = self.delay_for(trial);
        tokio::time::sleep(delay).await;
        delay
    }

    pub async fn replay_observed(&self, trial: usize) -> (Duration, Duration) {
        let scheduled = self.delay_for(trial);
        let start = Instant::now();
        tokio::time::sleep(scheduled).await;
        (scheduled, start.elapsed())
    }

    pub fn is_evaluation_eligible(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> SubmitRequest {
        SubmitRequest {
            task_id: "T1".into(),
            submission_key: "K1".into(),
            goal: "demo".into(),
        }
    }

    fn accepted_run(reply: NativeReply) -> String {
        match reply {
            NativeReply::P(PReply::Accepted { run_id }) => run_id,
            NativeReply::Q(QReply::Accepted { run_id, .. }) => run_id,
            other => panic!("unexpected accepted reply: {other:?}"),
        }
    }

    #[tokio::test]
    async fn p_and_q_keep_native_identity_shapes() {
        assert!(matches!(
            DeterministicAgent::new(AgentShape::P)
                .submit(request())
                .await
                .unwrap(),
            NativeReply::P(PReply::Accepted { .. })
        ));
        assert!(matches!(
            DeterministicAgent::new(AgentShape::Q)
                .submit(request())
                .await
                .unwrap(),
            NativeReply::Q(QReply::Accepted { .. })
        ));
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
        let mut changed = request();
        changed.goal = "different".into();
        assert!(agent.submit(changed).await.is_err());
        assert_eq!(agent.run_count().unwrap(), 1);
    }

    #[tokio::test]
    async fn independent_key_creates_independent_execution() {
        let agent = DeterministicAgent::new(AgentShape::P);
        let first = agent.submit(request()).await.unwrap();
        let mut next = request();
        next.submission_key = "K2".into();
        assert_ne!(first, agent.submit(next).await.unwrap());
        assert_eq!(agent.run_count().unwrap(), 2);
    }

    #[tokio::test]
    async fn full_capability_profile_is_actually_implemented() {
        for shape in [AgentShape::P, AgentShape::Q] {
            let c = DeterministicAgent::new(shape).capabilities();
            assert!(c.query && c.streaming && c.follow_up && c.cancel);
        }
    }

    #[tokio::test]
    async fn p_followup_stays_in_run_while_q_creates_continuation_run() {
        let p = DeterministicAgent::new(AgentShape::P);
        let q = DeterministicAgent::new(AgentShape::Q);
        let p_run = accepted_run(p.submit(request()).await.unwrap());
        let q_run = accepted_run(q.submit(request()).await.unwrap());

        let NativeReply::P(PReply::FollowUpAccepted { run_id, .. }) =
            p.follow_up(&p_run, "continue".into()).await.unwrap()
        else {
            panic!("P follow-up shape");
        };
        assert_eq!(run_id, p_run);

        let NativeReply::Q(QReply::ContinuationAccepted {
            previous_run_id,
            run_id,
            ..
        }) = q.follow_up(&q_run, "continue".into()).await.unwrap()
        else {
            panic!("Q continuation shape");
        };
        assert_eq!(previous_run_id, q_run);
        assert_ne!(run_id, q_run);
    }

    #[tokio::test]
    async fn p_cancel_confirms_immediately_q_requires_later_confirmation() {
        let p = DeterministicAgent::new(AgentShape::P);
        let q = DeterministicAgent::new(AgentShape::Q);
        let p_run = accepted_run(p.submit(request()).await.unwrap());
        let q_run = accepted_run(q.submit(request()).await.unwrap());

        assert!(matches!(
            p.cancel(&p_run).await.unwrap(),
            NativeReply::P(PReply::CancelConfirmed { .. })
        ));
        assert!(matches!(
            q.cancel(&q_run).await.unwrap(),
            NativeReply::Q(QReply::CancelRequested { .. })
        ));
        let NativeReply::Q(QReply::Snapshot { state, .. }) = q.query(&q_run).await.unwrap() else {
            panic!("Q snapshot");
        };
        assert_eq!(state, "running");
        q.confirm_cancel(&q_run).unwrap();
        let NativeReply::Q(QReply::Snapshot { state, .. }) = q.query(&q_run).await.unwrap() else {
            panic!("Q snapshot");
        };
        assert_eq!(state, "cancelled");
    }

    #[tokio::test]
    async fn p_and_q_emit_different_native_event_shapes() {
        let p = DeterministicAgent::new(AgentShape::P);
        let q = DeterministicAgent::new(AgentShape::Q);
        let p_run = accepted_run(p.submit(request()).await.unwrap());
        let q_run = accepted_run(q.submit(request()).await.unwrap());
        p.emit_progress(&p_run, 40).unwrap();
        q.emit_progress(&q_run, 40).unwrap();

        let p_events = p.events_since(&p_run, 1).await.unwrap();
        let q_events = q.events_since(&q_run, 1).await.unwrap();
        assert!(matches!(
            p_events.as_slice(),
            [NativeEvent::P(PEvent {
                kind: PEventKind::Progress { percent: 40 },
                ..
            })]
        ));
        assert!(matches!(
            q_events.as_slice(),
            [NativeEvent::Q(QEvent {
                kind: QEventKind::Running {
                    progress_percent: Some(40)
                },
                ..
            })]
        ));
    }

    #[tokio::test]
    async fn complete_is_idempotent_but_not_overwritable() {
        let agent = DeterministicAgent::new(AgentShape::P);
        let run_id = accepted_run(agent.submit(request()).await.unwrap());
        agent.complete(&run_id, "ART1").unwrap();
        let before = agent.query(&run_id).await.unwrap();
        agent.complete(&run_id, "ART1").unwrap();
        assert_eq!(before, agent.query(&run_id).await.unwrap());
        assert!(agent.complete(&run_id, "ART2").is_err());
    }

    #[tokio::test]
    async fn persistent_fixture_reopens_after_integration_memory_loss() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("agent-state.json");
        let first = DeterministicAgent::persistent(AgentShape::Q, &path).unwrap();
        let run_id = accepted_run(first.submit(request()).await.unwrap());
        first.emit_progress(&run_id, 40).unwrap();
        drop(first);

        let reopened = DeterministicAgent::persistent(AgentShape::Q, &path).unwrap();
        let NativeReply::Q(QReply::Snapshot {
            state, revision, ..
        }) = reopened.query(&run_id).await.unwrap()
        else {
            panic!("Q snapshot expected");
        };
        assert_eq!(state, "running");
        assert_eq!(revision, 2);
        assert_eq!(reopened.run_count().unwrap(), 1);
    }

    #[tokio::test]
    async fn cloned_fixture_preserves_external_execution_state() {
        let agent = DeterministicAgent::new(AgentShape::P);
        let peer = agent.clone();
        let run_id = accepted_run(agent.submit(request()).await.unwrap());
        peer.complete(&run_id, "ART1").unwrap();
        let NativeReply::P(PReply::Snapshot {
            state, artifact, ..
        }) = agent.query(&run_id).await.unwrap()
        else {
            panic!("P snapshot expected");
        };
        assert_eq!(state, "completed");
        assert_eq!(artifact.as_deref(), Some("ART1"));
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

    #[test]
    fn measured_label_does_not_self_approve_replay() {
        let trace = S2sDelayTrace {
            evidence_level: "MEASURED_S2S".into(),
            source: "unverified".into(),
            samples_ms: vec![1],
        };
        assert!(!trace.is_evaluation_eligible());
    }

    #[tokio::test]
    async fn observed_replay_keeps_scheduled_and_elapsed_distinct() {
        let trace = S2sDelayTrace {
            evidence_level: "TEST_ONLY".into(),
            source: "smoke".into(),
            samples_ms: vec![1],
        };
        let (scheduled, elapsed) = trace.replay_observed(0).await;
        assert_eq!(scheduled, Duration::from_millis(1));
        assert!(elapsed >= scheduled);
    }
}
