use async_trait::async_trait;
use gate2_contracts::{
    AgentBackend, AgentError, AgentObservation, CancelOutcome, CanonicalAccepted, NativeEvent,
    NativeReply, ObservationKind, PEventKind, PReply, QEventKind, QReply, SubmitRequest,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSnapshot {
    pub run_id: String,
    pub source_revision: u64,
    pub state: String,
    pub artifact: Option<String>,
}

#[async_trait]
pub trait AgentBoundary: Send + Sync {
    async fn submit(&self, request: SubmitRequest) -> Result<CanonicalAccepted, AgentError>;
    async fn query(&self, run_id: &str) -> Result<CanonicalSnapshot, AgentError>;
    async fn follow_up(&self, run_id: &str, text: String)
        -> Result<CanonicalAccepted, AgentError>;
    async fn cancel(&self, run_id: &str) -> Result<CancelOutcome, AgentError>;
    async fn events_since(
        &self,
        run_id: &str,
        after_revision: u64,
    ) -> Result<Vec<AgentObservation>, AgentError>;
}

pub struct EdgeNormalized<B> {
    backend: B,
}

impl<B> EdgeNormalized<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl<B: AgentBackend> AgentBoundary for EdgeNormalized<B> {
    async fn submit(&self, request: SubmitRequest) -> Result<CanonicalAccepted, AgentError> {
        normalize_submit(self.backend.submit(request).await?)
    }

    async fn query(&self, run_id: &str) -> Result<CanonicalSnapshot, AgentError> {
        normalize_snapshot(self.backend.query(run_id).await?)
    }

    async fn follow_up(
        &self,
        run_id: &str,
        text: String,
    ) -> Result<CanonicalAccepted, AgentError> {
        normalize_follow_up(self.backend.follow_up(run_id, text).await?)
    }

    async fn cancel(&self, run_id: &str) -> Result<CancelOutcome, AgentError> {
        normalize_cancel(self.backend.cancel(run_id).await?)
    }

    async fn events_since(
        &self,
        run_id: &str,
        after_revision: u64,
    ) -> Result<Vec<AgentObservation>, AgentError> {
        self.backend
            .events_since(run_id, after_revision)
            .await?
            .into_iter()
            .map(normalize_event)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedAccepted {
    P { run_id: String },
    Q { context_id: String, run_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedFollowUp {
    P {
        run_id: String,
        revision: u64,
    },
    Q {
        context_id: String,
        previous_run_id: String,
        run_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedCancel {
    PConfirmed { run_id: String, revision: u64 },
    QRequested {
        context_id: String,
        run_id: String,
        revision: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedSnapshot {
    P {
        run_id: String,
        revision: u64,
        state: String,
        artifact: Option<String>,
    },
    Q {
        context_id: String,
        run_id: String,
        revision: u64,
        state: String,
        artifact: Option<String>,
    },
}

/// Candidate B: provider-neutral typed lifecycle variants are intentionally visible
/// to a Core handler before becoming the same canonical Task/feedback semantics.
pub struct CoreVisibleTyped<B> {
    backend: B,
}

impl<B> CoreVisibleTyped<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
}

impl<B: AgentBackend> CoreVisibleTyped<B> {
    async fn typed_submit(&self, request: SubmitRequest) -> Result<TypedAccepted, AgentError> {
        match self.backend.submit(request).await? {
            NativeReply::P(PReply::Accepted { run_id }) => Ok(TypedAccepted::P { run_id }),
            NativeReply::Q(QReply::Accepted { context_id, run_id }) => {
                Ok(TypedAccepted::Q { context_id, run_id })
            }
            _ => Err(AgentError::Backend(
                "submit returned non-accepted native reply".into(),
            )),
        }
    }

    async fn typed_query(&self, run_id: &str) -> Result<TypedSnapshot, AgentError> {
        match self.backend.query(run_id).await? {
            NativeReply::P(PReply::Snapshot {
                run_id,
                revision,
                state,
                artifact,
            }) => Ok(TypedSnapshot::P {
                run_id,
                revision,
                state,
                artifact,
            }),
            NativeReply::Q(QReply::Snapshot {
                context_id,
                run_id,
                revision,
                state,
                artifact,
            }) => Ok(TypedSnapshot::Q {
                context_id,
                run_id,
                revision,
                state,
                artifact,
            }),
            _ => Err(AgentError::Backend(
                "query returned non-snapshot native reply".into(),
            )),
        }
    }

    async fn typed_follow_up(
        &self,
        run_id: &str,
        text: String,
    ) -> Result<TypedFollowUp, AgentError> {
        match self.backend.follow_up(run_id, text).await? {
            NativeReply::P(PReply::FollowUpAccepted { run_id, revision }) => {
                Ok(TypedFollowUp::P { run_id, revision })
            }
            NativeReply::Q(QReply::ContinuationAccepted {
                context_id,
                previous_run_id,
                run_id,
            }) => Ok(TypedFollowUp::Q {
                context_id,
                previous_run_id,
                run_id,
            }),
            _ => Err(AgentError::Backend(
                "follow-up returned unexpected native reply".into(),
            )),
        }
    }

    async fn typed_cancel(&self, run_id: &str) -> Result<TypedCancel, AgentError> {
        match self.backend.cancel(run_id).await? {
            NativeReply::P(PReply::CancelConfirmed { run_id, revision }) => {
                Ok(TypedCancel::PConfirmed { run_id, revision })
            }
            NativeReply::Q(QReply::CancelRequested {
                context_id,
                run_id,
                revision,
            }) => Ok(TypedCancel::QRequested {
                context_id,
                run_id,
                revision,
            }),
            _ => Err(AgentError::Backend(
                "cancel returned unexpected native reply".into(),
            )),
        }
    }
}

#[async_trait]
impl<B: AgentBackend> AgentBoundary for CoreVisibleTyped<B> {
    async fn submit(&self, request: SubmitRequest) -> Result<CanonicalAccepted, AgentError> {
        Ok(match self.typed_submit(request).await? {
            TypedAccepted::P { run_id } => CanonicalAccepted {
                run_id,
                context_id: None,
            },
            TypedAccepted::Q { context_id, run_id } => CanonicalAccepted {
                run_id,
                context_id: Some(context_id),
            },
        })
    }

    async fn query(&self, run_id: &str) -> Result<CanonicalSnapshot, AgentError> {
        Ok(match self.typed_query(run_id).await? {
            TypedSnapshot::P {
                run_id,
                revision,
                state,
                artifact,
            } => CanonicalSnapshot {
                run_id,
                source_revision: revision,
                state,
                artifact,
            },
            TypedSnapshot::Q {
                context_id: _,
                run_id,
                revision,
                state,
                artifact,
            } => CanonicalSnapshot {
                run_id,
                source_revision: revision,
                state,
                artifact,
            },
        })
    }

    async fn follow_up(
        &self,
        run_id: &str,
        text: String,
    ) -> Result<CanonicalAccepted, AgentError> {
        Ok(match self.typed_follow_up(run_id, text).await? {
            TypedFollowUp::P {
                run_id,
                revision: _,
            } => CanonicalAccepted {
                run_id,
                context_id: None,
            },
            TypedFollowUp::Q {
                context_id,
                previous_run_id: _,
                run_id,
            } => CanonicalAccepted {
                run_id,
                context_id: Some(context_id),
            },
        })
    }

    async fn cancel(&self, run_id: &str) -> Result<CancelOutcome, AgentError> {
        Ok(match self.typed_cancel(run_id).await? {
            TypedCancel::PConfirmed {
                run_id,
                revision: _,
            } => CancelOutcome {
                run_id,
                requested: true,
                confirmed: true,
            },
            TypedCancel::QRequested {
                context_id: _,
                run_id,
                revision: _,
            } => CancelOutcome {
                run_id,
                requested: true,
                confirmed: false,
            },
        })
    }

    async fn events_since(
        &self,
        run_id: &str,
        after_revision: u64,
    ) -> Result<Vec<AgentObservation>, AgentError> {
        self.backend
            .events_since(run_id, after_revision)
            .await?
            .into_iter()
            .map(normalize_event)
            .collect()
    }
}

fn normalize_submit(reply: NativeReply) -> Result<CanonicalAccepted, AgentError> {
    match reply {
        NativeReply::P(PReply::Accepted { run_id }) => Ok(CanonicalAccepted {
            run_id,
            context_id: None,
        }),
        NativeReply::Q(QReply::Accepted { context_id, run_id }) => Ok(CanonicalAccepted {
            run_id,
            context_id: Some(context_id),
        }),
        _ => Err(AgentError::Backend(
            "submit returned non-accepted native reply".into(),
        )),
    }
}

fn normalize_follow_up(reply: NativeReply) -> Result<CanonicalAccepted, AgentError> {
    match reply {
        NativeReply::P(PReply::FollowUpAccepted {
            run_id,
            revision: _,
        }) => Ok(CanonicalAccepted {
            run_id,
            context_id: None,
        }),
        NativeReply::Q(QReply::ContinuationAccepted {
            context_id,
            previous_run_id: _,
            run_id,
        }) => Ok(CanonicalAccepted {
            run_id,
            context_id: Some(context_id),
        }),
        _ => Err(AgentError::Backend(
            "follow-up returned unexpected native reply".into(),
        )),
    }
}

fn normalize_cancel(reply: NativeReply) -> Result<CancelOutcome, AgentError> {
    match reply {
        NativeReply::P(PReply::CancelConfirmed {
            run_id,
            revision: _,
        }) => Ok(CancelOutcome {
            run_id,
            requested: true,
            confirmed: true,
        }),
        NativeReply::Q(QReply::CancelRequested {
            context_id: _,
            run_id,
            revision: _,
        }) => Ok(CancelOutcome {
            run_id,
            requested: true,
            confirmed: false,
        }),
        _ => Err(AgentError::Backend(
            "cancel returned unexpected native reply".into(),
        )),
    }
}

fn normalize_event(event: NativeEvent) -> Result<AgentObservation, AgentError> {
    Ok(match event {
        NativeEvent::P(event) => AgentObservation {
            run_id: event.run_id,
            source_revision: event.revision,
            kind: match event.kind {
                PEventKind::Progress { percent } => ObservationKind::Progress { percent },
                PEventKind::Question { question_id, text } => {
                    ObservationKind::Question { question_id, text }
                }
                PEventKind::Result { artifact } => ObservationKind::Result { artifact },
                PEventKind::Cancelled => ObservationKind::Cancelled,
                PEventKind::Failed { reason } => ObservationKind::Failed { reason },
            },
        },
        NativeEvent::Q(event) => AgentObservation {
            run_id: event.run_id,
            source_revision: event.revision,
            kind: match event.kind {
                QEventKind::Running { progress_percent } => ObservationKind::Progress {
                    percent: progress_percent.unwrap_or(0),
                },
                QEventKind::InputRequired { request_id, prompt } => ObservationKind::Question {
                    question_id: request_id,
                    text: prompt,
                },
                QEventKind::ArtifactReady { artifact_id } => {
                    ObservationKind::Result {
                        artifact: artifact_id,
                    }
                }
                QEventKind::Cancelled => ObservationKind::Cancelled,
                QEventKind::Failure { message } => {
                    ObservationKind::Failed { reason: message }
                }
            },
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gate2_fixture::{AgentShape, DeterministicAgent};

    fn request() -> SubmitRequest {
        SubmitRequest {
            task_id: "T1".into(),
            submission_key: "K1".into(),
            goal: "demo".into(),
        }
    }

    #[tokio::test]
    async fn edge_and_typed_preserve_same_submit_query_semantics() {
        let edge = EdgeNormalized::new(DeterministicAgent::new(AgentShape::Q));
        let typed = CoreVisibleTyped::new(DeterministicAgent::new(AgentShape::Q));
        let a = edge.submit(request()).await.unwrap();
        let b = typed.submit(request()).await.unwrap();
        assert_eq!(a.context_id.as_deref(), Some("ctx-T1"));
        assert_eq!(b.context_id.as_deref(), Some("ctx-T1"));
        assert_eq!(
            edge.query(&a.run_id).await.unwrap().state,
            typed.query(&b.run_id).await.unwrap().state
        );
    }

    #[tokio::test]
    async fn edge_and_typed_preserve_cancel_confirmation_semantics() {
        for shape in [AgentShape::P, AgentShape::Q] {
            let edge = EdgeNormalized::new(DeterministicAgent::new(shape));
            let typed = CoreVisibleTyped::new(DeterministicAgent::new(shape));
            let a = edge.submit(request()).await.unwrap();
            let b = typed.submit(request()).await.unwrap();
            assert_eq!(
                edge.cancel(&a.run_id).await.unwrap().confirmed,
                typed.cancel(&b.run_id).await.unwrap().confirmed
            );
        }
    }

    #[tokio::test]
    async fn edge_and_typed_normalize_different_event_envelopes_equally() {
        let p_edge_backend = DeterministicAgent::new(AgentShape::P);
        let p_typed_backend = DeterministicAgent::new(AgentShape::P);
        let p1 = p_edge_backend.submit(request()).await.unwrap();
        let p2 = p_typed_backend.submit(request()).await.unwrap();
        let p1_run = match p1 {
            NativeReply::P(PReply::Accepted { run_id }) => run_id,
            _ => unreachable!(),
        };
        let p2_run = match p2 {
            NativeReply::P(PReply::Accepted { run_id }) => run_id,
            _ => unreachable!(),
        };
        p_edge_backend.emit_progress(&p1_run, 40).unwrap();
        p_typed_backend.emit_progress(&p2_run, 40).unwrap();
        let edge = EdgeNormalized::new(p_edge_backend);
        let typed = CoreVisibleTyped::new(p_typed_backend);
        assert_eq!(
            edge.events_since(&p1_run, 1).await.unwrap()[0].kind,
            typed.events_since(&p2_run, 1).await.unwrap()[0].kind
        );

        let q_edge_backend = DeterministicAgent::new(AgentShape::Q);
        let q_typed_backend = DeterministicAgent::new(AgentShape::Q);
        let q1 = q_edge_backend.submit(request()).await.unwrap();
        let q2 = q_typed_backend.submit(request()).await.unwrap();
        let q1_run = match q1 {
            NativeReply::Q(QReply::Accepted { run_id, .. }) => run_id,
            _ => unreachable!(),
        };
        let q2_run = match q2 {
            NativeReply::Q(QReply::Accepted { run_id, .. }) => run_id,
            _ => unreachable!(),
        };
        q_edge_backend.emit_progress(&q1_run, 40).unwrap();
        q_typed_backend.emit_progress(&q2_run, 40).unwrap();
        let edge = EdgeNormalized::new(q_edge_backend);
        let typed = CoreVisibleTyped::new(q_typed_backend);
        assert_eq!(
            edge.events_since(&q1_run, 1).await.unwrap()[0].kind,
            typed.events_since(&q2_run, 1).await.unwrap()[0].kind
        );
    }
}
