use async_trait::async_trait;
use gate2_contracts::{
    AgentBackend, AgentError, CanonicalAccepted, NativeReply, PReply, QReply, SubmitRequest,
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
        normalize_accept(self.backend.submit(request).await?)
    }

    async fn query(&self, run_id: &str) -> Result<CanonicalSnapshot, AgentError> {
        normalize_snapshot(self.backend.query(run_id).await?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedAccepted {
    P { run_id: String },
    Q { context_id: String, run_id: String },
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
}

fn normalize_accept(reply: NativeReply) -> Result<CanonicalAccepted, AgentError> {
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

fn normalize_snapshot(reply: NativeReply) -> Result<CanonicalSnapshot, AgentError> {
    match reply {
        NativeReply::P(PReply::Snapshot {
            run_id,
            revision,
            state,
            artifact,
        }) => Ok(CanonicalSnapshot {
            run_id,
            source_revision: revision,
            state,
            artifact,
        }),
        NativeReply::Q(QReply::Snapshot {
            context_id: _,
            run_id,
            revision,
            state,
            artifact,
        }) => Ok(CanonicalSnapshot {
            run_id,
            source_revision: revision,
            state,
            artifact,
        }),
        _ => Err(AgentError::Backend(
            "query returned non-snapshot native reply".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gate2_fixture::{AgentShape, DeterministicAgent};

    #[tokio::test]
    async fn edge_and_typed_preserve_same_canonical_semantics() {
        let req = SubmitRequest {
            task_id: "T1".into(),
            submission_key: "K1".into(),
            goal: "demo".into(),
        };
        let edge = EdgeNormalized::new(DeterministicAgent::new(AgentShape::Q));
        let typed = CoreVisibleTyped::new(DeterministicAgent::new(AgentShape::Q));
        let a = edge.submit(req.clone()).await.unwrap();
        let b = typed.submit(req).await.unwrap();
        assert_eq!(a.context_id.as_deref(), Some("ctx-T1"));
        assert_eq!(b.context_id.as_deref(), Some("ctx-T1"));
        assert_eq!(
            edge.query(&a.run_id).await.unwrap().state,
            typed.query(&b.run_id).await.unwrap().state
        );
    }
}
