//! The task that owns the socket and the correlation.
//!
//! `docs/architecture.md` §11: *"Each gets an owning task, not a mutex …  State
//! held by value in one task, `enum Command { … reply: oneshot::Sender<R> }` over
//! a bounded `mpsc`."* Two of the four invariants meet here — the announcement
//! window's inputs (`active_responses`) and delegation correlation — and both are
//! order-sensitive, so both live in one task rather than behind a lock.
//!
//! This file is the port of `handleProviderEvent`, `handleLifecycle`,
//! `armResponseInactivityTimeout`, `retryRefusedResponse`, `cancel`,
//! `cancelResponses`, `resetResponses` and `send`
//! (`server/src/voice/realtime-provider.mjs:211-240`, `:548-761`, `:773-815`).
//! The *output queue* — the other half of upstream's object — is
//! [`super::queue`], because a job that awaits a conversation-item receipt
//! cannot run inside the task that has to deliver it.
//!
//! # The one place the port corrects rather than copies
//!
//! `response.created` with no usable response id: upstream removes the pending
//! response from the correlation queue and clears its start timer *before*
//! checking the id, so a pending that reaches that branch is never registered
//! anywhere and never settles — the caller's promise hangs, and with it the
//! whole output queue. VIA settles it `failed / correlation` instead. Neither
//! shipped dialect can reach the branch: both always carry `response.id`.

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use indexmap::IndexMap;
use serde_json::{Map, Value};
use tokio::sync::{mpsc, oneshot};
use tokio::task::AbortHandle;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message;
use via_i18n::{Locale, keys, t};
use via_protocol::SessionMode;

use super::outcome::{
    DIAGNOSTIC_RESPONSE_TIMEOUT, Diagnostic, OutcomeKind, OutcomePhase, ProviderEvent,
    ResponseContext, ResponseOrigin, ResponseOutcome, SessionEvent,
};
use super::pending::PendingResponse;
use super::transport::WsSink;
use super::{BUSY_RETRY_DELAYS, Dialect, MAX_BUSY_RETRIES, POST_CANCEL_RECOVERY, has_model_turn};
use crate::error::RealtimeError;
use crate::event_error::{
    ErrorClass, realtime_event_classification_text, realtime_event_error_message,
};
use crate::lifecycle::{is_completed_status, is_response_activity_event, realtime_response_id};
use crate::provider::{AgentContext, AgentContextPatch, SessionRequest};

/// A caller-supplied `cancelResponses` filter.
pub(crate) type ResponsePredicate =
    Arc<dyn Fn(&ResponseContext, ResponseOrigin) -> bool + Send + Sync>;

/// How the queue task's `create` step ended.
#[derive(Debug)]
pub(crate) enum CreateResult {
    /// Everything was written; arm the response-start watchdog.
    Done,
    /// A guard declined it. Upstream `create` returning `false`.
    Skipped,
    /// It threw. Upstream `create` rejecting.
    Failed(RealtimeError),
}

/// The answer to "may I start a response now?".
#[derive(Debug)]
pub(crate) enum BeginOutcome {
    /// Registered. The queue task now owns the `create` step.
    Started {
        pending: Arc<PendingResponse>,
        outcome: oneshot::Receiver<ResponseOutcome>,
    },
    /// Refused and already settled — a correlation conflict.
    Settled {
        outcome: oneshot::Receiver<ResponseOutcome>,
    },
    /// The session is not ready, or the queue generation moved on. Upstream's
    /// `run` returning `undefined`.
    Stale,
}

/// Everything the session's owning task is asked to do.
pub(crate) enum Command {
    /// A frame arrived.
    Inbound(Value),
    /// The transport failed.
    TransportError(String),
    /// The transport ended.
    SocketClosed,
    /// Write a frame.
    Send(Value),
    /// Create a conversation item and wait for its receipt.
    CreateItem {
        item: Value,
        reply: oneshot::Sender<Result<Value, RealtimeError>>,
    },
    /// Wait for idle, then register a response start.
    BeginResponse {
        origin: ResponseOrigin,
        context: ResponseContext,
        generation: u64,
        reply: oneshot::Sender<BeginOutcome>,
    },
    /// The queue task finished the `create` step.
    FinishCreate {
        pending: Arc<PendingResponse>,
        result: CreateResult,
    },
    /// Wait for idle, then report whether the caller may proceed.
    AwaitProceed {
        generation: u64,
        check_generation: bool,
        reply: oneshot::Sender<bool>,
    },
    /// Cancel everything in flight.
    Cancel,
    /// Cancel the responses a predicate matches.
    CancelResponses {
        predicate: ResponsePredicate,
        reply: oneshot::Sender<bool>,
    },
    /// Merge a context patch; answer with whether the session is ready.
    PatchContext {
        patch: AgentContextPatch,
        reply: oneshot::Sender<bool>,
    },
    /// Re-send `session.update` from the current context.
    UpdateSession,
    /// A conversation item was never acknowledged.
    ItemTimeout { id: String },
    /// A response never started.
    StartTimeout { pending: Arc<PendingResponse> },
    /// A response may have stopped producing output.
    InactivityTimeout {
        response_id: String,
        pending: Arc<PendingResponse>,
    },
    /// The grace period after cancelling an inactive response elapsed.
    InactivityRecovery {
        response_id: String,
        pending: Arc<PendingResponse>,
    },
    /// Replay a `response.create` the provider refused as busy.
    ReplayRefused {
        pending: Arc<PendingResponse>,
        generation: u64,
    },
    /// Close the session.
    Close,
}

/// A command parked until the session goes idle.
///
/// Upstream's `idleWaiters`, except that what is parked is the *whole* step
/// rather than only the wake-up: `await whenIdle()` followed by a check-and-
/// register is one atomic block in JavaScript, and splitting it into two
/// round-trips would open a window where an inbound `response.created` lands
/// between the two.
enum Deferred {
    Begin {
        origin: ResponseOrigin,
        context: ResponseContext,
        generation: u64,
        reply: oneshot::Sender<BeginOutcome>,
    },
    Proceed {
        generation: u64,
        check_generation: bool,
        reply: oneshot::Sender<bool>,
    },
    Replay {
        pending: Arc<PendingResponse>,
        generation: u64,
    },
}

/// A conversation item the session is waiting to be acknowledged.
struct ItemWaiter {
    reply: oneshot::Sender<Result<Value, RealtimeError>>,
    timer: Option<AbortHandle>,
}

impl ItemWaiter {
    fn cancel_timer(&mut self) {
        if let Some(timer) = self.timer.take() {
            timer.abort();
        }
    }
}

/// Everything the session knows.
pub(crate) struct State {
    dialect: Dialect,
    label: String,
    mode: SessionMode,
    locale: Locale,
    agent_context: AgentContext,
    response_start_timeout: Duration,
    response_inactivity_timeout: Duration,

    sink: Option<WsSink>,
    ready: bool,
    session_configured: bool,
    recent_context_injected: bool,
    generation: Arc<AtomicU64>,

    active_responses: HashSet<String>,
    pending_responses: VecDeque<Arc<PendingResponse>>,
    response_waiters: IndexMap<String, Arc<PendingResponse>>,
    item_waiters: IndexMap<String, ItemWaiter>,
    deferred: VecDeque<Deferred>,

    events: mpsc::Sender<SessionEvent>,
    commands: mpsc::Sender<Command>,
    ready_signal: Option<oneshot::Sender<Result<(), RealtimeError>>>,
}

/// Everything [`State::new`] needs.
pub(crate) struct StateSetup {
    pub(crate) dialect: Dialect,
    pub(crate) mode: SessionMode,
    pub(crate) locale: Locale,
    pub(crate) agent_context: AgentContext,
    pub(crate) response_start_timeout: Duration,
    pub(crate) response_inactivity_timeout: Duration,
    pub(crate) sink: WsSink,
    pub(crate) generation: Arc<AtomicU64>,
    pub(crate) events: mpsc::Sender<SessionEvent>,
    pub(crate) commands: mpsc::Sender<Command>,
    pub(crate) ready_signal: oneshot::Sender<Result<(), RealtimeError>>,
}

impl State {
    pub(crate) fn new(setup: StateSetup) -> Self {
        Self {
            label: setup.dialect.provider().label().to_owned(),
            dialect: setup.dialect,
            mode: setup.mode,
            locale: setup.locale,
            agent_context: setup.agent_context,
            response_start_timeout: setup.response_start_timeout,
            response_inactivity_timeout: setup.response_inactivity_timeout,
            sink: Some(setup.sink),
            ready: false,
            session_configured: false,
            recent_context_injected: false,
            generation: setup.generation,
            active_responses: HashSet::new(),
            pending_responses: VecDeque::new(),
            response_waiters: IndexMap::new(),
            item_waiters: IndexMap::new(),
            deferred: VecDeque::new(),
            events: setup.events,
            commands: setup.commands,
            ready_signal: Some(setup.ready_signal),
        }
    }

    /// Run until the socket is gone.
    pub(crate) async fn run(mut self, mut commands: mpsc::Receiver<Command>) {
        // Upstream writes `connectionMessages` from the socket's `open` handler,
        // before anything else can be queued behind them.
        for message in self.dialect.protocol().connection_messages() {
            self.send(message).await;
        }
        while let Some(command) = commands.recv().await {
            if self.handle(command).await {
                break;
            }
        }
        self.shutdown().await;
    }

    /// Handle one command; `true` means the session is finished.
    async fn handle(&mut self, command: Command) -> bool {
        match command {
            Command::Inbound(event) => return self.handle_inbound(event).await,
            Command::TransportError(detail) => {
                // Upstream `ws.on('error')`: report it, and let the socket's
                // own close finish the connect promise.
                self.emit(SessionEvent::Error(RealtimeError::Transport {
                    detail: detail.clone(),
                }))
                .await;
                self.fail_connect(RealtimeError::Transport { detail });
            }
            Command::SocketClosed => return true,
            Command::Send(payload) => self.send(payload).await,
            Command::CreateItem { item, reply } => self.create_item(item, reply).await,
            Command::BeginResponse {
                origin,
                context,
                generation,
                reply,
            } => {
                if self.active_responses.is_empty() {
                    self.begin_response(origin, context, generation, reply)
                        .await;
                } else {
                    self.deferred.push_back(Deferred::Begin {
                        origin,
                        context,
                        generation,
                        reply,
                    });
                }
            }
            Command::FinishCreate { pending, result } => {
                self.finish_create(&pending, result).await;
            }
            Command::AwaitProceed {
                generation,
                check_generation,
                reply,
            } => {
                if self.active_responses.is_empty() {
                    let _ = reply.send(self.may_proceed(generation, check_generation));
                } else {
                    self.deferred.push_back(Deferred::Proceed {
                        generation,
                        check_generation,
                        reply,
                    });
                }
            }
            Command::Cancel => self.cancel().await,
            Command::CancelResponses { predicate, reply } => {
                let cancelled = self.cancel_responses(&predicate).await;
                let _ = reply.send(cancelled);
            }
            Command::PatchContext { patch, reply } => {
                self.agent_context.apply(patch);
                let _ = reply.send(self.ready);
            }
            Command::UpdateSession => {
                if self.ready {
                    self.update_session().await;
                }
            }
            Command::ItemTimeout { id } => self.item_timeout(&id),
            Command::StartTimeout { pending } => self.start_timeout(&pending),
            Command::InactivityTimeout {
                response_id,
                pending,
            } => self.inactivity_timeout(&response_id, &pending).await,
            Command::InactivityRecovery {
                response_id,
                pending,
            } => self.inactivity_recovery(&response_id, &pending).await,
            Command::ReplayRefused {
                pending,
                generation,
            } => {
                if self.active_responses.is_empty() {
                    self.replay_refused(&pending, generation).await;
                } else {
                    self.deferred.push_back(Deferred::Replay {
                        pending,
                        generation,
                    });
                }
            }
            Command::Close => {
                self.close_socket().await;
                return true;
            }
        }
        false
    }

    /// Settle everything and say goodbye.
    ///
    /// Upstream's `ws.on('close')`: `ready = false`, `sessionConfigured = false`,
    /// `recentContextInjected = false`, `resetResponses()`, reject the connect
    /// promise, `onClose()`.
    async fn shutdown(&mut self) {
        self.ready = false;
        self.session_configured = false;
        self.recent_context_injected = false;
        self.reset_responses().await;
        self.fail_connect(RealtimeError::ConnectionClosed {
            label: self.label.clone(),
        });
        self.sink = None;
        self.emit(SessionEvent::Closed).await;
    }

    // ── inbound ─────────────────────────────────────────────────────────────

    /// Upstream `handleProviderEvent`.
    async fn handle_inbound(&mut self, raw: Value) -> bool {
        let events = self.dialect.protocol().normalize_incoming(raw);
        for event in events {
            if event.is_null() {
                continue;
            }
            let kind = event
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();

            // An `error` before the session is usable is what rejects
            // `connect()`. Making it fast is the point: a speech-to-speech
            // session slot that is still occupied answers immediately, and the
            // caller's backoff can retry rather than waiting out the connect
            // budget.
            if kind == "error" && !self.ready {
                let error = RealtimeError::ProviderRefused {
                    message: realtime_event_error_message(&event, self.locale),
                };
                self.emit(SessionEvent::Error(error.clone())).await;
                self.fail_connect(error);
                self.close_socket().await;
                return true;
            }

            if kind == "session.created" {
                self.update_session().await;
                if !self.dialect.capabilities().acknowledges_session_update {
                    self.mark_ready().await;
                }
            }
            if kind == "session.updated" {
                self.mark_ready().await;
            }

            let mut annotated = ProviderEvent {
                event,
                origin: ResponseOrigin::Model,
                context: ResponseContext::new(),
                retried: false,
            };
            self.handle_lifecycle(&mut annotated).await;
            self.emit(SessionEvent::Provider(Box::new(annotated))).await;
        }
        false
    }

    /// Upstream `handleLifecycle`.
    async fn handle_lifecycle(&mut self, annotated: &mut ProviderEvent) {
        let kind = annotated.kind().to_owned();
        let event = &annotated.event;

        if kind == "conversation.item.created" {
            self.resolve_item_waiter(event);
        }

        if kind == "error" && !self.item_waiters.is_empty() {
            // Note the early return: an error while an item receipt is
            // outstanding belongs to that item, and must not also settle a
            // response. It is how `sendUserText` reports
            // `failed / input` rather than `failed / <status>`.
            self.reject_first_item_waiter(event);
            return;
        }

        if is_response_activity_event(event) {
            let id = realtime_response_id(event).to_owned();
            self.active_responses.insert(id.clone());
            if kind != "response.done"
                && let Some(pending) = self.response_waiters.get(&id).cloned()
            {
                self.arm_inactivity_timeout(&id, &pending);
            }
        }

        if kind == "response.created" {
            self.handle_response_created(annotated).await;
        }

        if kind == "response.done" || kind == "error" {
            self.handle_response_end(&kind, annotated).await;
        }
    }

    async fn handle_response_created(&mut self, annotated: &mut ProviderEvent) {
        let id = realtime_response_id(&annotated.event).to_owned();
        let pending = if self.dialect.capabilities().response_metadata_correlation {
            let request_id = self
                .dialect
                .protocol()
                .response_correlation_id(&annotated.event);
            if request_id.is_empty() {
                None
            } else {
                self.pending_responses
                    .iter()
                    .position(|pending| pending.request_id() == request_id)
                    .and_then(|index| self.pending_responses.remove(index))
            }
        } else {
            self.pending_responses.pop_front()
        };

        if let Some(pending) = &pending {
            pending.cancel_timer();
        }
        annotated.origin = pending
            .as_ref()
            .map_or(ResponseOrigin::Model, |pending| pending.origin());
        annotated.context = pending
            .as_ref()
            .map_or_else(ResponseContext::new, |pending| pending.context().clone());

        let Some(pending) = pending else {
            if !id.is_empty() {
                self.active_responses.insert(id);
            }
            return;
        };
        if id.is_empty() {
            // See the module docs: upstream leaves this pending unreachable.
            pending.settle(
                ResponseOutcome::new(OutcomeKind::Failed).with_phase(OutcomePhase::Correlation),
            );
            return;
        }
        self.active_responses.insert(id.clone());
        self.response_waiters
            .insert(id.clone(), Arc::clone(&pending));
        pending.mark_started(Instant::now());
        self.arm_inactivity_timeout(&id, &pending);
    }

    async fn handle_response_end(&mut self, kind: &str, annotated: &mut ProviderEvent) {
        let is_error = kind == "error";
        let mut id = realtime_response_id(&annotated.event).to_owned();
        let mut pending = self.response_waiters.get(&id).cloned();

        // An error with no response id at all belongs to the start that has not
        // been correlated yet.
        if is_error && pending.is_none() && id.is_empty() && !self.pending_responses.is_empty() {
            pending = self.pending_responses.pop_front();
        }
        // An error with exactly one response in flight belongs to it, whatever
        // id it did or did not carry.
        if is_error
            && pending.is_none()
            && self.response_waiters.len() == 1
            && let Some((first_id, first)) = self
                .response_waiters
                .iter()
                .next()
                .map(|(key, value)| (key.clone(), Arc::clone(value)))
        {
            id = first_id;
            pending = Some(first);
        }

        // A `response.create` can race either another response or the tail of a
        // Smart Turn input. Both are transient: retry the exact refused payload
        // instead of surfacing a protocol timing error to the user.
        let refusal = if is_error {
            self.dialect
                .provider()
                .classify_error(&realtime_event_classification_text(&annotated.event))
        } else {
            ErrorClass::Other
        };
        let slot_busy = self.dialect.capabilities().single_response_slot
            && refusal == ErrorClass::ResponseSlotBusy;
        let input_busy = pending
            .as_ref()
            .is_some_and(|pending| pending.origin() == ResponseOrigin::Model)
            && refusal == ErrorClass::InputBusy;
        if (slot_busy || input_busy)
            && let Some(pending) = pending.clone()
            && pending.response_payload().is_some()
            && pending.busy_retries() < MAX_BUSY_RETRIES
        {
            pending.record_busy_retry();
            annotated.retried = true;
            pending.cancel_timer();
            // The correlated id belongs to the server's own response — the
            // refusal proves ours never started — so unbind the mismatched
            // mapping. The real response still retires the id through
            // `active_responses`.
            if !id.is_empty()
                && self
                    .response_waiters
                    .get(&id)
                    .is_some_and(|waiter| Arc::ptr_eq(waiter, &pending))
            {
                self.response_waiters.shift_remove(&id);
            }
            self.retry_refused(pending);
            return;
        }

        if let Some(pending) = &pending {
            annotated.origin = pending.origin();
            annotated.context = pending.context().clone();
        }
        if !id.is_empty() {
            self.active_responses.remove(&id);
            self.response_waiters.shift_remove(&id);
        }
        let status = annotated
            .event
            .get("response")
            .and_then(|response| response.get("status"))
            .and_then(Value::as_str);
        let completed = !is_error && is_completed_status(status);
        if let Some(pending) = pending {
            let outcome = if completed {
                ResponseOutcome::new(OutcomeKind::Completed).with_response_id(&id)
            } else {
                ResponseOutcome::new(OutcomeKind::Failed)
                    .with_response_id(&id)
                    .with_status(status)
            };
            pending.settle(outcome);
        }
        self.resolve_idle().await;
    }

    fn resolve_item_waiter(&mut self, event: &Value) {
        let id = event
            .get("item")
            .and_then(|item| item.get("id"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        let key = if self.item_waiters.contains_key(id) {
            Some(id.to_owned())
        } else if !self.dialect.capabilities().conversation_item_id_echo
            && self.item_waiters.len() == 1
        {
            // A provider that replaces the id can still be matched, but only
            // when there is exactly one thing it could be.
            self.item_waiters.keys().next().cloned()
        } else {
            None
        };
        let Some(key) = key else { return };
        let Some(mut waiter) = self.item_waiters.shift_remove(&key) else {
            return;
        };
        waiter.cancel_timer();
        let item = event.get("item").cloned().unwrap_or(Value::Null);
        let _ = waiter.reply.send(Ok(item));
    }

    fn reject_first_item_waiter(&mut self, event: &Value) {
        let Some(key) = self.item_waiters.keys().next().cloned() else {
            return;
        };
        let Some(mut waiter) = self.item_waiters.shift_remove(&key) else {
            return;
        };
        waiter.cancel_timer();
        let detail = event
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .filter(|message| !message.is_empty());
        let _ = waiter.reply.send(Err(RealtimeError::ItemRejected {
            label: self.label.clone(),
            detail,
        }));
    }

    // ── session lifecycle ───────────────────────────────────────────────────

    async fn mark_ready(&mut self) {
        self.ready = true;
        self.session_configured = true;
        self.restore_recent_conversation().await;
        if let Some(signal) = self.ready_signal.take() {
            let _ = signal.send(Ok(()));
        }
    }

    fn fail_connect(&mut self, error: RealtimeError) {
        if let Some(signal) = self.ready_signal.take() {
            let _ = signal.send(Err(error));
        }
    }

    /// Upstream `updateSession`.
    async fn update_session(&mut self) {
        let session = self.dialect.provider().build_session(&SessionRequest {
            configured: self.session_configured,
            agent_context: &self.agent_context,
        });
        let frame = self.dialect.protocol().session_update(session);
        self.send(frame).await;
    }

    /// Upstream `restoreRecentConversation`.
    ///
    /// Written once per connection, fire and forget: the receipt is not awaited
    /// because nothing downstream depends on it, and blocking session readiness
    /// on it would delay the first user turn.
    async fn restore_recent_conversation(&mut self) {
        if self.recent_context_injected {
            return;
        }
        self.recent_context_injected = true;
        if !has_model_turn(self.mode) {
            // `dictation` mounts no model, so there is no conversation to
            // restore context into. `docs/architecture.md` §2.
            return;
        }
        let Some(recent) = self
            .agent_context
            .recent_context
            .as_deref()
            .filter(|recent| !recent.is_empty())
        else {
            return;
        };
        let text = [
            "<restored_context>",
            t(self.locale, keys::REALTIME_RESTORED_CONTEXT_INSTRUCTIONS),
            recent,
            "</restored_context>",
        ]
        .join("\n");
        let item = self.dialect.protocol().user_text_item(&text);
        let id = self.item_id(&item);
        let frame = self
            .dialect
            .protocol()
            .conversation_item_create(with_id(item, &id));
        self.send(frame).await;
    }

    // ── the output side ─────────────────────────────────────────────────────

    /// Upstream `send`.
    ///
    /// The correlation stamp happens here rather than at the call site because
    /// `retryRefusedResponse` replays a stored frame through the same door, and
    /// the stamp has to be idempotent for the replay to keep its request id.
    async fn send(&mut self, payload: Value) {
        if self.sink.is_none() {
            return;
        }
        let body = self.encode_frame(payload);
        let text = serde_json::to_string(&body).unwrap_or_default();
        let Some(sink) = self.sink.as_mut() else {
            return;
        };
        if let Err(detail) = futures::SinkExt::send(sink, Message::Text(text.into())).await {
            self.sink = None;
            self.emit(SessionEvent::Error(RealtimeError::Transport { detail }))
                .await;
        }
    }

    fn encode_frame(&self, payload: Value) -> Value {
        let is_response_create = payload
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(|kind| kind == "response.create");
        let outgoing = match (is_response_create, self.pending_responses.back()) {
            (true, Some(pending)) => {
                let correlated = self
                    .dialect
                    .protocol()
                    .correlate_response_create(payload, pending.request_id());
                pending.set_response_payload(correlated.clone());
                correlated
            }
            _ => payload,
        };
        self.dialect.protocol().encode_outgoing(outgoing)
    }

    /// Upstream `createConversationItem`.
    async fn create_item(
        &mut self,
        item: Value,
        reply: oneshot::Sender<Result<Value, RealtimeError>>,
    ) {
        let id = self.item_id(&item);
        let timer = self.spawn_timer(
            self.response_start_timeout,
            Command::ItemTimeout { id: id.clone() },
        );
        self.item_waiters.insert(
            id.clone(),
            ItemWaiter {
                reply,
                timer: Some(timer),
            },
        );
        let frame = self
            .dialect
            .protocol()
            .conversation_item_create(with_id(item, &id));
        self.send(frame).await;
    }

    fn item_timeout(&mut self, id: &str) {
        let Some(mut waiter) = self.item_waiters.shift_remove(id) else {
            return;
        };
        waiter.cancel_timer();
        let _ = waiter.reply.send(Err(RealtimeError::ItemUnconfirmed {
            label: self.label.clone(),
            id: id.to_owned(),
        }));
    }

    /// Id namespaces are dialect-specific — the GA dialect derives them from the
    /// item type — so the protocol adapter mints the id unless the caller
    /// already chose one.
    fn item_id(&self, item: &Value) -> String {
        item.get("id")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .map_or_else(
                || self.dialect.protocol().conversation_item_id(item),
                str::to_owned,
            )
    }

    fn may_proceed(&self, generation: u64, check_generation: bool) -> bool {
        self.ready && (!check_generation || generation == self.generation.load(Ordering::SeqCst))
    }

    /// Upstream `enqueueResponse`'s registration half.
    async fn begin_response(
        &mut self,
        origin: ResponseOrigin,
        context: ResponseContext,
        generation: u64,
        reply: oneshot::Sender<BeginOutcome>,
    ) {
        if !self.may_proceed(generation, true) {
            let _ = reply.send(BeginOutcome::Stale);
            return;
        }
        let (tx, rx) = oneshot::channel();
        let pending = PendingResponse::new(origin, context, crate::protocol::hyphenless_uuid(), tx);
        if !self.pending_responses.is_empty() {
            // Fail closed: with two unanswered starts there is no way to know
            // which `response.created` belongs to which.
            let error = RealtimeError::ResponseCorrelationConflict;
            pending.settle(
                ResponseOutcome::new(OutcomeKind::Failed)
                    .with_phase(OutcomePhase::Correlation)
                    .with_error(error.message(self.locale)),
            );
            self.emit(SessionEvent::Error(error)).await;
            let _ = reply.send(BeginOutcome::Settled { outcome: rx });
            return;
        }
        self.pending_responses.push_back(Arc::clone(&pending));
        let _ = reply.send(BeginOutcome::Started {
            pending,
            outcome: rx,
        });
    }

    async fn finish_create(&mut self, pending: &Arc<PendingResponse>, result: CreateResult) {
        match result {
            CreateResult::Done => {
                if !pending.is_settled() {
                    let timer = self.spawn_timer(
                        self.response_start_timeout,
                        Command::StartTimeout {
                            pending: Arc::clone(pending),
                        },
                    );
                    pending.arm_timer(timer);
                }
            }
            CreateResult::Skipped => {
                self.drop_pending(pending);
                pending.settle(
                    ResponseOutcome::new(OutcomeKind::Skipped)
                        .with_phase(OutcomePhase::Deduplicated),
                );
            }
            CreateResult::Failed(error) => {
                self.drop_pending(pending);
                pending.settle(
                    ResponseOutcome::new(OutcomeKind::Failed)
                        .with_phase(OutcomePhase::Input)
                        .with_error(error.message(self.locale)),
                );
                // A failure that came out of a provider event is already
                // reported as the outcome; reporting it again as a session
                // error would surface one refusal twice.
                if !error.is_provider_event() {
                    self.emit(SessionEvent::Error(error)).await;
                }
            }
        }
    }

    fn drop_pending(&mut self, pending: &Arc<PendingResponse>) {
        if let Some(index) = self
            .pending_responses
            .iter()
            .position(|candidate| Arc::ptr_eq(candidate, pending))
        {
            self.pending_responses.remove(index);
        }
    }

    fn start_timeout(&mut self, pending: &Arc<PendingResponse>) {
        self.drop_pending(pending);
        pending.settle(ResponseOutcome::new(OutcomeKind::TimedOut).with_phase(OutcomePhase::Start));
    }

    // ── the two watchdogs ───────────────────────────────────────────────────

    /// Upstream `armResponseInactivityTimeout`.
    ///
    /// A **sliding window**, not an absolute duration limit: every response
    /// activity event re-opens it, so long speech stays valid as long as the
    /// provider keeps streaming.
    fn arm_inactivity_timeout(&mut self, response_id: &str, pending: &Arc<PendingResponse>) {
        pending.cancel_timer();
        pending.mark_activity(Instant::now());
        let timer = self.spawn_timer(
            self.response_inactivity_timeout,
            Command::InactivityTimeout {
                response_id: response_id.to_owned(),
                pending: Arc::clone(pending),
            },
        );
        pending.arm_timer(timer);
    }

    async fn inactivity_timeout(&mut self, response_id: &str, pending: &Arc<PendingResponse>) {
        if !self
            .response_waiters
            .get(response_id)
            .is_some_and(|waiter| Arc::ptr_eq(waiter, pending))
        {
            return;
        }
        let now = Instant::now();
        let inactivity = pending
            .last_activity_at()
            .map_or(Duration::ZERO, |at| now.saturating_duration_since(at));
        if let Some(remaining) = self.response_inactivity_timeout.checked_sub(inactivity)
            && !remaining.is_zero()
        {
            // Activity landed after the timer was armed; wait out the rest of
            // the window rather than cancelling a response that is still
            // speaking.
            let timer = self.spawn_timer(
                remaining,
                Command::InactivityTimeout {
                    response_id: response_id.to_owned(),
                    pending: Arc::clone(pending),
                },
            );
            pending.arm_timer(timer);
            return;
        }

        let elapsed = pending
            .started_at()
            .map_or(Duration::ZERO, |at| now.saturating_duration_since(at));
        self.emit(SessionEvent::Diagnostic(Diagnostic {
            event: DIAGNOSTIC_RESPONSE_TIMEOUT,
            provider: self.dialect.provider().key().to_owned(),
            response_id: response_id.to_owned(),
            phase: "inactivity",
            inactivity_ms: duration_ms(inactivity),
            elapsed_ms: duration_ms(elapsed),
        }))
        .await;

        let cancel = self.dialect.protocol().response_cancel();
        self.send(cancel).await;
        pending.settle(
            ResponseOutcome::new(OutcomeKind::TimedOut)
                .with_phase(OutcomePhase::Inactivity)
                .with_response_id(response_id),
        );
        // The provider is *asked* to cancel; if it never confirms, the id is
        // retired anyway so the session does not stay busy forever. Detached on
        // purpose — upstream calls `unref()` on it.
        let commands = self.commands.clone();
        let response_id = response_id.to_owned();
        let pending = Arc::clone(pending);
        tokio::spawn(async move {
            tokio::time::sleep(POST_CANCEL_RECOVERY).await;
            let _ = commands
                .send(Command::InactivityRecovery {
                    response_id,
                    pending,
                })
                .await;
        });
    }

    async fn inactivity_recovery(&mut self, response_id: &str, pending: &Arc<PendingResponse>) {
        if !self
            .response_waiters
            .get(response_id)
            .is_some_and(|waiter| Arc::ptr_eq(waiter, pending))
        {
            return;
        }
        self.response_waiters.shift_remove(response_id);
        self.active_responses.remove(response_id);
        self.resolve_idle().await;
    }

    // ── the busy-retry ladder ───────────────────────────────────────────────

    /// Upstream `retryRefusedResponse`.
    ///
    /// Two constraints shape it, and both are upstream's own words:
    ///
    /// 1. it must **not** be scheduled through the output queue — *"the refused
    ///    response's outcome promise is what the queue tail awaits, so queueing
    ///    the retry behind it deadlocks the whole pipeline"*;
    /// 2. *"a known active response provides the real release signal. A bounded
    ///    delay is used only when the busy error arrives before
    ///    `response.created`, so the server-side response is not visible yet."*
    fn retry_refused(&mut self, pending: Arc<PendingResponse>) {
        let generation = self.generation.load(Ordering::SeqCst);
        let index = usize::try_from(pending.busy_retries().saturating_sub(1)).unwrap_or(0);
        let delay = BUSY_RETRY_DELAYS[index.min(BUSY_RETRY_DELAYS.len() - 1)];
        let wait = if self.active_responses.is_empty() {
            Some(delay)
        } else {
            None
        };
        let commands = self.commands.clone();
        tokio::spawn(async move {
            if let Some(delay) = wait {
                tokio::time::sleep(delay).await;
            }
            let _ = commands
                .send(Command::ReplayRefused {
                    pending,
                    generation,
                })
                .await;
        });
    }

    async fn replay_refused(&mut self, pending: &Arc<PendingResponse>, generation: u64) {
        if !self.may_proceed(generation, true) {
            pending.settle(
                ResponseOutcome::new(OutcomeKind::Cancelled).with_phase(OutcomePhase::Start),
            );
            return;
        }
        if !self.pending_responses.is_empty() {
            pending.settle(
                ResponseOutcome::new(OutcomeKind::Failed).with_phase(OutcomePhase::Correlation),
            );
            return;
        }
        let Some(payload) = pending.response_payload() else {
            pending
                .settle(ResponseOutcome::new(OutcomeKind::Failed).with_phase(OutcomePhase::Start));
            return;
        };
        self.pending_responses.push_back(Arc::clone(pending));
        self.send(payload).await;
        if !pending.is_settled() {
            let timer = self.spawn_timer(
                self.response_start_timeout,
                Command::StartTimeout {
                    pending: Arc::clone(pending),
                },
            );
            pending.arm_timer(timer);
        }
    }

    // ── cancellation ────────────────────────────────────────────────────────

    /// Upstream `cancel`.
    async fn cancel(&mut self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        let has_response = !self.active_responses.is_empty() || !self.pending_responses.is_empty();
        for pending in std::mem::take(&mut self.pending_responses) {
            pending.settle(
                ResponseOutcome::new(OutcomeKind::Cancelled).with_phase(OutcomePhase::Start),
            );
        }
        self.reject_item_waiters(&RealtimeError::RequestCancelled);
        if has_response {
            let frame = self.dialect.protocol().response_cancel();
            self.send(frame).await;
        }
    }

    /// Upstream `cancelResponses`.
    async fn cancel_responses(&mut self, predicate: &ResponsePredicate) -> bool {
        let mut retained = VecDeque::with_capacity(self.pending_responses.len());
        for pending in std::mem::take(&mut self.pending_responses) {
            if predicate(pending.context(), pending.origin()) {
                pending.settle(
                    ResponseOutcome::new(OutcomeKind::Cancelled).with_phase(OutcomePhase::Start),
                );
            } else {
                retained.push_back(pending);
            }
        }
        self.pending_responses = retained;

        let mut cancelled_active = false;
        // The waiters are deliberately **not** removed: the response is still
        // running on the provider, and its `response.done` still has to retire
        // the id.
        for pending in self.response_waiters.values().cloned().collect::<Vec<_>>() {
            if !predicate(pending.context(), pending.origin()) {
                continue;
            }
            cancelled_active = true;
            pending.settle(
                ResponseOutcome::new(OutcomeKind::Cancelled).with_phase(OutcomePhase::Completion),
            );
        }
        if cancelled_active {
            let frame = self.dialect.protocol().response_cancel();
            self.send(frame).await;
        }
        cancelled_active
    }

    /// Upstream `resetResponses`.
    async fn reset_responses(&mut self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.active_responses.clear();
        self.reject_item_waiters(&RealtimeError::SessionReset);
        for pending in std::mem::take(&mut self.pending_responses) {
            pending.settle(ResponseOutcome::new(OutcomeKind::Cancelled));
        }
        for pending in std::mem::take(&mut self.response_waiters).into_values() {
            pending.settle(ResponseOutcome::new(OutcomeKind::Cancelled));
        }
        self.resolve_idle().await;
    }

    fn reject_item_waiters(&mut self, error: &RealtimeError) {
        for (_, mut waiter) in std::mem::take(&mut self.item_waiters) {
            waiter.cancel_timer();
            let _ = waiter.reply.send(Err(error.clone()));
        }
    }

    async fn close_socket(&mut self) {
        if let Some(sink) = self.sink.as_mut() {
            let _ = futures::SinkExt::send(sink, Message::Close(None)).await;
            let _ = futures::SinkExt::close(sink).await;
        }
    }

    // ── idle ────────────────────────────────────────────────────────────────

    /// Upstream `resolveIdle`.
    async fn resolve_idle(&mut self) {
        if !self.active_responses.is_empty() {
            return;
        }
        while let Some(deferred) = self.deferred.pop_front() {
            match deferred {
                Deferred::Begin {
                    origin,
                    context,
                    generation,
                    reply,
                } => {
                    self.begin_response(origin, context, generation, reply)
                        .await
                }
                Deferred::Proceed {
                    generation,
                    check_generation,
                    reply,
                } => {
                    let _ = reply.send(self.may_proceed(generation, check_generation));
                }
                Deferred::Replay {
                    pending,
                    generation,
                } => self.replay_refused(&pending, generation).await,
            }
        }
    }

    // ── plumbing ────────────────────────────────────────────────────────────

    /// Takes `&mut self` rather than `&self` on purpose: the task's future must
    /// be `Send`, a shared borrow of `State` is only `Send` if `State` is `Sync`,
    /// and `State` owns a `dyn Sink` that is `Send` but not `Sync`. Every caller
    /// already holds the exclusive borrow.
    async fn emit(&mut self, event: SessionEvent) {
        let _ = self.events.send(event).await;
    }

    fn spawn_timer(&self, delay: Duration, command: Command) -> AbortHandle {
        let commands = self.commands.clone();
        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            let _ = commands.send(command).await;
        })
        .abort_handle()
    }
}

/// `{ id, ...item }` — upstream's spread, so an item that already carries an id
/// keeps it and the id leads the serialized object.
fn with_id(item: Value, id: &str) -> Value {
    let mut fields = Map::new();
    fields.insert("id".to_owned(), Value::String(id.to_owned()));
    if let Value::Object(existing) = item {
        for (key, value) in existing {
            fields.insert(key, value);
        }
    }
    Value::Object(fields)
}

fn duration_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn an_item_keeps_an_id_it_already_had_and_the_id_leads() {
        let with_existing = with_id(json!({ "id": "mine", "type": "message" }), "generated");
        assert_eq!(with_existing["id"], json!("mine"));
        let keys: Vec<&str> = with_existing
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["id", "type"]);

        let minted = with_id(json!({ "type": "message" }), "generated");
        assert_eq!(minted["id"], json!("generated"));
    }

    #[test]
    fn a_non_object_item_still_gets_an_id_wrapper() {
        assert_eq!(with_id(json!(null), "x"), json!({ "id": "x" }));
    }

    #[test]
    fn durations_saturate_rather_than_wrapping() {
        assert_eq!(duration_ms(Duration::from_millis(1500)), 1500);
        assert_eq!(duration_ms(Duration::MAX), u64::MAX);
    }
}
