//! The output queue.
//!
//! Upstream's `outputQueue` is one promise chain:
//! `this.outputQueue = this.outputQueue.then(run, run)`. Three properties fall
//! out of that one line and all three are load-bearing:
//!
//! 1. **jobs run one at a time, in call order** — `.then` on the tail;
//! 2. **a job waits for the previous job's *outcome*, not just its writes** —
//!    `run` returns the outcome promise, so the chain awaits the response's
//!    `response.done`, not the `response.create`;
//! 3. **a rejected job does not break the chain** — the same handler is passed
//!    for both arms.
//!
//! Here that is a task draining a bounded `mpsc` of [`Job`]s and awaiting each
//! one to completion. It is a second task rather than more work inside
//! [`super::state`] because a job blocks on things only the state task can
//! deliver — a conversation-item receipt, `response.created`, `response.done` —
//! so running it there would deadlock on the first `await`.
//!
//! The busy-retry ladder is the one thing that deliberately does **not** come
//! through here; see [`super::state::State::retry_refused`].

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::{Map, Value};
use tokio::sync::{mpsc, oneshot};

use super::Dialect;
use super::outcome::{OutcomeKind, ResponseContext, ResponseOrigin, ResponseOutcome};
use super::pending::PendingResponse;
use super::state::{BeginOutcome, Command, CreateResult};
use crate::error::RealtimeError;
use crate::provider::InputProjection;

/// A late guard: upstream's `shouldSpeak` / `shouldCreate`.
///
/// Evaluated at the moment the job reaches the head of the queue, which is the
/// whole point — the condition that made a spoken result worth saying may have
/// stopped being true while it waited behind a two-minute answer.
pub type Guard = Arc<dyn Fn() -> bool + Send + Sync>;

/// A queued step that creates no response.
///
/// Upstream `enqueueAction`.
pub(crate) enum ActionKind {
    /// Re-send `session.update` from the current agent context.
    RefreshSession,
    /// Create one conversation item and wait for its receipt.
    CreateItem(Value),
    /// Write a projected input without asking for an answer.
    ApplyInput(Option<InputProjection>),
    /// Close a tool call without asking for an answer.
    FunctionOutput { call_id: String, output: Value },
    /// Do nothing; resolve when everything queued before it has resolved.
    ///
    /// **Not an upstream method.** It is the Rust spelling of upstream's
    /// `await frontend.outputQueue`, which reaches into the object because
    /// JavaScript lets it.
    Barrier,
}

/// A queued step that creates a response.
///
/// Upstream passes an anonymous `create` closure to `enqueueResponse`; the seven
/// call sites are enumerated instead, because a boxed async closure would hide
/// exactly the part of each one that differs.
pub(crate) enum CreateKind {
    /// `speak`.
    Speak {
        content: String,
        guard: Option<Guard>,
    },
    /// `ensureResponse`.
    Ensure {
        response: Option<Value>,
        guard: Option<Guard>,
    },
    /// `sendUserText`.
    UserText {
        text: String,
        modalities: Option<Vec<String>>,
    },
    /// `sendUserInput`.
    UserInput {
        projection: Option<InputProjection>,
        modalities: Option<Vec<String>>,
    },
    /// `sendFunctionOutput` with `createResponse: true`.
    FunctionOutput {
        call_id: String,
        output: Value,
        response: Option<Value>,
    },
    /// `injectResult`.
    InjectResult {
        item: Value,
        response: Value,
        inject_context: bool,
        injected: Arc<AtomicBool>,
    },
    /// `injectPermission`. The item was already created outside the queue.
    InjectPermission {
        response: Value,
        guard: Option<Guard>,
    },
}

/// One entry in the output queue.
pub(crate) enum Job {
    Action {
        kind: ActionKind,
        generation: u64,
        reply: oneshot::Sender<Result<bool, RealtimeError>>,
    },
    Response {
        origin: ResponseOrigin,
        context: ResponseContext,
        generation: u64,
        create: CreateKind,
        reply: oneshot::Sender<Option<ResponseOutcome>>,
    },
}

/// What the queue task needs to build frames and talk to the state task.
#[derive(Clone)]
pub(crate) struct QueueContext {
    pub(crate) dialect: Dialect,
    pub(crate) commands: mpsc::Sender<Command>,
    pub(crate) label: String,
}

impl QueueContext {
    fn closed(&self) -> RealtimeError {
        RealtimeError::ConnectionClosed {
            label: self.label.clone(),
        }
    }

    async fn send(&self, payload: Value) -> Result<(), RealtimeError> {
        self.commands
            .send(Command::Send(payload))
            .await
            .map_err(|_| self.closed())
    }

    /// Upstream `createConversationItem`.
    async fn create_item(&self, item: Value) -> Result<Value, RealtimeError> {
        let (reply, receipt) = oneshot::channel();
        self.commands
            .send(Command::CreateItem { item, reply })
            .await
            .map_err(|_| self.closed())?;
        receipt.await.map_err(|_| self.closed())?
    }

    /// Upstream `applyUserInput`.
    async fn apply_projection(
        &self,
        projection: &Option<InputProjection>,
    ) -> Result<bool, RealtimeError> {
        let Some(projection) = projection else {
            return Ok(false);
        };
        for event in &projection.before_events {
            self.send(event.clone()).await?;
        }
        if let Some(item) = &projection.conversation_item {
            self.create_item(item.clone()).await?;
        }
        for event in &projection.after_events {
            self.send(event.clone()).await?;
        }
        Ok(true)
    }

    fn response_create(&self, response: Option<Value>) -> Value {
        self.dialect.protocol().response_create(response)
    }
}

/// `modalities ? { modalities } : undefined`.
///
/// An empty list is still a list — it is truthy in JavaScript — so it produces
/// `{ modalities: [] }` rather than no body at all.
fn modalities_body(modalities: &Option<Vec<String>>) -> Option<Value> {
    modalities.as_ref().map(|modalities| {
        let mut body = Map::new();
        body.insert(
            "modalities".to_owned(),
            Value::Array(modalities.iter().cloned().map(Value::String).collect()),
        );
        Value::Object(body)
    })
}

/// Drain the queue, one job at a time, each to completion.
pub(crate) async fn run(context: QueueContext, mut jobs: mpsc::Receiver<Job>) {
    while let Some(job) = jobs.recv().await {
        match job {
            Job::Action {
                kind,
                generation,
                reply,
            } => {
                let outcome = run_action(&context, kind, generation).await;
                let _ = reply.send(outcome);
            }
            Job::Response {
                origin,
                context: response_context,
                generation,
                create,
                reply,
            } => {
                let outcome =
                    run_response(&context, origin, response_context, generation, create).await;
                let _ = reply.send(outcome);
            }
        }
    }
}

async fn run_action(
    context: &QueueContext,
    kind: ActionKind,
    generation: u64,
) -> Result<bool, RealtimeError> {
    // `updateAgentContext`'s refresh checks `ready` but deliberately not the
    // queue generation: a barge-in cancels responses, it does not invalidate the
    // instructions the model is about to be given.
    let check_generation = !matches!(kind, ActionKind::RefreshSession);
    let (reply, proceed) = oneshot::channel();
    context
        .commands
        .send(Command::AwaitProceed {
            generation,
            check_generation,
            reply,
        })
        .await
        .map_err(|_| context.closed())?;
    if !proceed.await.map_err(|_| context.closed())? {
        return Ok(false);
    }

    match kind {
        ActionKind::RefreshSession => {
            context
                .commands
                .send(Command::UpdateSession)
                .await
                .map_err(|_| context.closed())?;
            Ok(false)
        }
        ActionKind::CreateItem(item) => {
            context.create_item(item).await?;
            Ok(true)
        }
        ActionKind::ApplyInput(projection) => context.apply_projection(&projection).await,
        ActionKind::FunctionOutput { call_id, output } => {
            let item = context
                .dialect
                .protocol()
                .function_output_item(&call_id, &output);
            context.create_item(item).await?;
            Ok(true)
        }
        ActionKind::Barrier => Ok(true),
    }
}

async fn run_response(
    context: &QueueContext,
    origin: ResponseOrigin,
    response_context: ResponseContext,
    generation: u64,
    create: CreateKind,
) -> Option<ResponseOutcome> {
    let (reply, begun) = oneshot::channel();
    if context
        .commands
        .send(Command::BeginResponse {
            origin,
            context: response_context,
            generation,
            reply,
        })
        .await
        .is_err()
    {
        return None;
    }
    let (pending, outcome) = match begun.await.ok()? {
        BeginOutcome::Stale => return None,
        BeginOutcome::Settled { outcome } => return outcome.await.ok(),
        BeginOutcome::Started { pending, outcome } => (pending, outcome),
    };

    let result = match execute_create(context, &create, &pending).await {
        Ok(true) => CreateResult::Done,
        Ok(false) => CreateResult::Skipped,
        Err(error) => CreateResult::Failed(error),
    };
    if context
        .commands
        .send(Command::FinishCreate {
            pending: Arc::clone(&pending),
            result,
        })
        .await
        .is_err()
    {
        // The state task is gone, so nothing else will ever settle this
        // pending — and an unsettled pending is a caller that waits forever.
        pending.settle(ResponseOutcome::new(OutcomeKind::Cancelled));
    }
    outcome.await.ok()
}

/// Upstream's `create` callbacks, one arm each.
///
/// `Ok(false)` is upstream's `return false`: a guard declined, so nothing was
/// written and the outcome is `skipped / deduplicated`.
async fn execute_create(
    context: &QueueContext,
    create: &CreateKind,
    pending: &Arc<PendingResponse>,
) -> Result<bool, RealtimeError> {
    match create {
        CreateKind::Speak { content, guard } => {
            if declined(guard) {
                return Ok(false);
            }
            let body = context.dialect.provider().build_speak_response(content);
            context.send(context.response_create(Some(body))).await?;
            Ok(true)
        }
        CreateKind::Ensure { response, guard } => {
            if declined(guard) {
                return Ok(false);
            }
            context
                .send(context.response_create(response.clone()))
                .await?;
            Ok(true)
        }
        CreateKind::UserText { text, modalities } => {
            let item = context.dialect.protocol().user_text_item(text);
            context.create_item(item).await?;
            context
                .send(context.response_create(modalities_body(modalities)))
                .await?;
            Ok(true)
        }
        CreateKind::UserInput {
            projection,
            modalities,
        } => {
            if !context.apply_projection(projection).await? {
                return Ok(false);
            }
            context
                .send(context.response_create(modalities_body(modalities)))
                .await?;
            Ok(true)
        }
        CreateKind::FunctionOutput {
            call_id,
            output,
            response,
        } => {
            let item = context
                .dialect
                .protocol()
                .function_output_item(call_id, output);
            context.create_item(item).await?;
            context
                .send(context.response_create(response.clone()))
                .await?;
            Ok(true)
        }
        CreateKind::InjectResult {
            item,
            response,
            inject_context,
            injected,
        } => {
            if *inject_context {
                context.create_item(item.clone()).await?;
                injected.store(true, Ordering::SeqCst);
            }
            context
                .send(context.response_create(Some(response.clone())))
                .await?;
            Ok(true)
        }
        CreateKind::InjectPermission { response, guard } => {
            // The permission may already have been answered while its item was
            // being written, in which case asking about it aloud is worse than
            // saying nothing.
            if pending.is_settled() || declined(guard) {
                return Ok(false);
            }
            context
                .send(context.response_create(Some(response.clone())))
                .await?;
            Ok(true)
        }
    }
}

fn declined(guard: &Option<Guard>) -> bool {
    guard.as_ref().is_some_and(|guard| !guard())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn no_modalities_means_no_response_body_at_all() {
        assert_eq!(modalities_body(&None), None);
    }

    #[test]
    fn an_empty_modality_list_is_still_a_body() {
        assert_eq!(
            modalities_body(&Some(Vec::new())),
            Some(json!({ "modalities": [] }))
        );
    }

    #[test]
    fn modalities_are_carried_verbatim() {
        assert_eq!(
            modalities_body(&Some(vec!["text".to_owned(), "audio".to_owned()])),
            Some(json!({ "modalities": ["text", "audio"] }))
        );
    }

    #[test]
    fn a_missing_guard_never_declines() {
        assert!(!declined(&None));
        assert!(!declined(&Some(Arc::new(|| true))));
        assert!(declined(&Some(Arc::new(|| false))));
    }
}
