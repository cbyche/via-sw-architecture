//! One WebSocket to one provider.
//!
//! The port of `RealtimeFrontend` (`server/src/voice/realtime-provider.mjs`),
//! split across three files by what owns what:
//!
//! | File | Upstream | Owns |
//! | --- | --- | --- |
//! | [`state`] | `handleProviderEvent`, `handleLifecycle`, the watchdogs, `send` | the socket, correlation, the response registry |
//! | [`queue`] | `outputQueue`, `enqueueAction`, `enqueueResponse` | one job at a time, in call order |
//! | this file | the constructor and the public methods | the handle, the connect handshake |
//!
//! # `dictation` mounts no model
//!
//! `docs/architecture.md` §2 makes `dictation` the one mode with no model turn:
//! the provider may be a plain streaming ASR. Everything that would create a
//! response — [`speak`](RealtimeSession::speak),
//! [`send_user_text`](RealtimeSession::send_user_text),
//! [`inject_result`](RealtimeSession::inject_result) and the rest — answers
//! `skipped / no_model_turn` in that mode without touching the socket, and
//! [`append_audio`](RealtimeSession::append_audio),
//! [`commit_audio`](RealtimeSession::commit_audio) and the transcript events
//! keep working. Nothing in the session requires a `response.*` event ever to
//! arrive.
//!
//! # Draining the event stream
//!
//! [`SessionEvents`] is a bounded channel and the session applies real
//! backpressure to the provider when it fills, which is what keeps a slow
//! consumer from growing an unbounded audio backlog. The cost is one rule: **do
//! not await a session method from the same task that drains the stream**.
//! Those methods wait on the state task, and the state task may be waiting on
//! the stream.

mod outcome;
mod pending;
mod queue;
mod state;
mod transport;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use futures::StreamExt;
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use via_i18n::Locale;
use via_protocol::SessionMode;

pub use outcome::{
    DIAGNOSTIC_RESPONSE_TIMEOUT, Diagnostic, InjectOutcome, OutcomeKind, OutcomePhase,
    ProviderEvent, ResponseContext, ResponseOrigin, ResponseOutcome, SessionEvent,
};
pub use queue::Guard;
pub use transport::{Transport, WsSink, WsStream};

use crate::capabilities::ProviderCapabilities;
use crate::error::RealtimeError;
use crate::protocol::RealtimeProtocol;
use crate::provider::{
    AgentContext, AgentContextPatch, ConnectionInfo, InputProjection, InputProjectionRequest,
    PermissionRequest, RealtimeProvider,
};
use queue::{ActionKind, CreateKind, Job, QueueContext};
use state::{Command, ResponsePredicate, State, StateSetup};

/// How long the whole connect handshake may take.
///
/// External contract — `default-value` / *RealtimeFrontend timeouts*:
/// hard-coded 25 000 ms, then the socket is terminated. The budget covers socket
/// open, `session.created`, `session.update` and `session.updated` — not just the
/// TCP connect.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(25);

/// How long to wait for `response.created`, when the provider names no other.
///
/// External contract — `realtime-provider.mjs:123-126`.
pub const DEFAULT_RESPONSE_START_TIMEOUT: Duration = Duration::from_secs(30);

/// How long a started response may produce no output before it is cancelled.
///
/// External contract — `realtime-provider.mjs:129-131`. A **sliding window**, not
/// an absolute duration limit: upstream's own comment says so, and long speech
/// must stay valid while the provider keeps streaming.
pub const DEFAULT_RESPONSE_INACTIVITY_TIMEOUT: Duration = Duration::from_secs(120);

/// The busy-retry ladder, indexed by `busy_retries - 1`.
///
/// External contract — `default-value` / *busy-response retry schedule*
/// (`realtime-provider.mjs:731`).
pub const BUSY_RETRY_DELAYS: [Duration; 3] = [
    Duration::from_millis(1200),
    Duration::from_millis(2600),
    Duration::from_millis(5000),
];

/// How many times one refused `response.create` may be replayed.
///
/// External contract — `realtime-provider.mjs:641`.
pub const MAX_BUSY_RETRIES: u32 = 3;

/// How long after cancelling an inactive response its id is still held.
///
/// External contract — `realtime-provider.mjs:703-709`. The provider is *asked*
/// to cancel; if it never confirms, the id is retired anyway so the session does
/// not stay busy forever.
pub const POST_CANCEL_RECOVERY: Duration = Duration::from_secs(1);

/// Default depth of the event stream.
pub const DEFAULT_EVENT_CAPACITY: usize = 256;

/// Default depth of the command and job channels.
pub const DEFAULT_COMMAND_CAPACITY: usize = 64;

/// Whether this session mode has a model turn at all.
///
/// `docs/architecture.md` §2: `dictation` is the one mode that mounts no model.
const fn has_model_turn(mode: SessionMode) -> bool {
    !matches!(mode, SessionMode::Dictation)
}

/// Which service, and which dialect, resolved once for this connection.
///
/// Capabilities are read **once**, at construction, exactly as upstream's
/// `this.capabilities = { ...DEFAULT_CAPABILITIES, ...provider.capabilities }`.
/// That matters: `dashscope`'s declaration is a getter whose value depends on the
/// configured model family, and a session that re-read it mid-connection could
/// change its item-correlation strategy underneath an outstanding waiter.
#[derive(Clone)]
pub(crate) struct Dialect {
    provider: Arc<dyn RealtimeProvider>,
    per_connection: Option<Arc<dyn RealtimeProtocol>>,
    capabilities: ProviderCapabilities,
}

impl Dialect {
    pub(crate) fn provider(&self) -> &dyn RealtimeProvider {
        self.provider.as_ref()
    }

    pub(crate) fn protocol(&self) -> &dyn RealtimeProtocol {
        match &self.per_connection {
            Some(protocol) => protocol.as_ref(),
            None => self.provider.protocol(),
        }
    }

    pub(crate) fn capabilities(&self) -> ProviderCapabilities {
        self.capabilities
    }
}

/// How a session is opened.
#[derive(Debug, Clone)]
pub struct SessionOptions {
    /// Which layers this session mounts. `docs/architecture.md` §2.
    pub mode: SessionMode,
    /// The locale every message this session composes is rendered in.
    pub locale: Locale,
    /// The conversation context the provider builds `session.update` from.
    pub agent_context: AgentContext,
    /// Override the response-start watchdog.
    ///
    /// Upstream's precedence, reproduced: this option, then
    /// [`RealtimeProvider::response_start_timeout`], then
    /// [`DEFAULT_RESPONSE_START_TIMEOUT`].
    pub response_start_timeout: Option<Duration>,
    /// Override the output-inactivity watchdog.
    pub response_inactivity_timeout: Option<Duration>,
    /// Override the connect budget.
    pub connect_timeout: Option<Duration>,
    /// Depth of the event stream.
    pub event_capacity: usize,
    /// Depth of the command and job channels.
    pub command_capacity: usize,
}

impl Default for SessionOptions {
    fn default() -> Self {
        Self {
            mode: SessionMode::default(),
            locale: Locale::En,
            agent_context: AgentContext::default(),
            response_start_timeout: None,
            response_inactivity_timeout: None,
            connect_timeout: None,
            event_capacity: DEFAULT_EVENT_CAPACITY,
            command_capacity: DEFAULT_COMMAND_CAPACITY,
        }
    }
}

/// Everything the session reports, in order.
#[derive(Debug)]
pub struct SessionEvents {
    receiver: mpsc::Receiver<SessionEvent>,
}

impl SessionEvents {
    /// The next event, or `None` once the session is finished.
    ///
    /// [`SessionEvent::Closed`] is always the last event before `None`.
    pub async fn recv(&mut self) -> Option<SessionEvent> {
        self.receiver.recv().await
    }

    /// Stop accepting events. Already-queued events still drain.
    pub fn close(&mut self) {
        self.receiver.close();
    }
}

/// What `send_function_output` is allowed to do afterwards.
#[derive(Debug, Clone, Default)]
pub struct FunctionOutputOptions {
    /// Whether to ask for a response once the tool result is acknowledged.
    ///
    /// `false` closes a stale tool call without giving the model a turn — which
    /// is what makes a superseded call cheap to retire.
    pub create_response: bool,
    /// The `response.create` body, when one is created.
    pub response: Option<Value>,
}

impl FunctionOutputOptions {
    /// Ask for a response, with no body. Upstream's default.
    #[must_use]
    pub fn with_response() -> Self {
        Self {
            create_response: true,
            response: None,
        }
    }

    /// Close the tool call and say nothing.
    #[must_use]
    pub fn without_response() -> Self {
        Self {
            create_response: false,
            response: None,
        }
    }
}

struct Inner {
    provider: Arc<dyn RealtimeProvider>,
    dialect: Dialect,
    connection_id: String,
    mode: SessionMode,
    generation: Arc<AtomicU64>,
    commands: mpsc::Sender<Command>,
    jobs: mpsc::Sender<Job>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        // A dropped handle must not leave the socket and its two tasks running.
        // `try_send` is enough in every ordinary case; the spawn covers a full
        // command channel, which only happens under load — and it asks for the
        // runtime rather than assuming one, because a handle dropped while the
        // runtime is shutting down would otherwise panic in a destructor. There
        // is nothing to close in that case anyway: the tasks are already gone.
        if self.commands.try_send(Command::Close).is_err()
            && let Ok(runtime) = tokio::runtime::Handle::try_current()
        {
            let commands = self.commands.clone();
            runtime.spawn(async move {
                let _ = commands.send(Command::Close).await;
            });
        }
    }
}

/// One realtime session: one socket, one provider, one dialect.
///
/// Cheap to clone — every clone is the same session.
#[derive(Clone)]
pub struct RealtimeSession {
    inner: Arc<Inner>,
}

impl core::fmt::Debug for RealtimeSession {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RealtimeSession")
            .field("provider", &self.inner.provider.key())
            .field("connection_id", &self.inner.connection_id)
            .field("mode", &self.inner.mode)
            .finish_non_exhaustive()
    }
}

impl RealtimeSession {
    /// Open a real WebSocket to this provider and configure the session.
    ///
    /// The order is upstream's and it matters:
    /// [`preflight`](RealtimeProvider::preflight), then
    /// [`is_configured`](RealtimeProvider::is_configured), then the socket. A
    /// session that is going to fail for a known reason never opens one.
    ///
    /// Resolves when the session is *usable*, not when the socket is open.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::UnsupportedModel`] or whatever else `preflight` refuses;
    /// [`RealtimeError::NotConfigured`]; [`RealtimeError::Transport`];
    /// [`RealtimeError::ConnectTimeout`]; [`RealtimeError::ProviderRefused`] when
    /// the provider answers the handshake with an `error` event.
    pub async fn connect(
        provider: Arc<dyn RealtimeProvider>,
        options: SessionOptions,
    ) -> Result<(Self, SessionEvents), RealtimeError> {
        Self::preflight(provider.as_ref(), options.locale)?;
        let url = provider.url()?;
        let headers = provider.headers();
        let budget = options.connect_timeout.unwrap_or(CONNECT_TIMEOUT);
        let started = tokio::time::Instant::now();

        let transport = match tokio::time::timeout(budget, Transport::connect(&url, &headers)).await
        {
            Ok(result) => result?,
            Err(_elapsed) => {
                return Err(RealtimeError::ConnectTimeout {
                    provider: provider.key().to_owned(),
                    message: provider.connect_timeout_message(options.locale),
                });
            }
        };

        let mut options = options;
        options.connect_timeout = Some(budget.saturating_sub(started.elapsed()));
        Self::open(provider, options, transport).await
    }

    /// Configure a session over an already-established transport.
    ///
    /// The seam `via-realtime-mock`, a local pipeline and this crate's own tests
    /// all use: everything about the session except the socket is exercised
    /// without one.
    ///
    /// # Errors
    ///
    /// As [`connect`](Self::connect), minus the transport-establishment ones.
    pub async fn open(
        provider: Arc<dyn RealtimeProvider>,
        options: SessionOptions,
        transport: Transport,
    ) -> Result<(Self, SessionEvents), RealtimeError> {
        Self::preflight(provider.as_ref(), options.locale)?;
        let budget = options.connect_timeout.unwrap_or(CONNECT_TIMEOUT);
        let locale = options.locale;
        let timeout_message = provider.connect_timeout_message(locale);
        let key = provider.key().to_owned();
        let label = provider.label().to_owned();

        let (session, events, ready) = Self::spawn(provider, options, transport);

        match tokio::time::timeout(budget, ready).await {
            Ok(Ok(Ok(()))) => Ok((session, events)),
            Ok(Ok(Err(error))) => Err(error),
            // The state task dropped the signal without answering, which only
            // happens when it stopped first.
            Ok(Err(_)) => Err(RealtimeError::ConnectionClosed { label }),
            Err(_elapsed) => {
                drop(session);
                Err(RealtimeError::ConnectTimeout {
                    provider: key,
                    message: timeout_message,
                })
            }
        }
    }

    fn preflight(provider: &dyn RealtimeProvider, locale: Locale) -> Result<(), RealtimeError> {
        provider.preflight()?;
        if provider.is_configured() {
            return Ok(());
        }
        Err(RealtimeError::NotConfigured {
            provider: provider.key().to_owned(),
            message: provider.missing_configuration_message(locale),
        })
    }

    fn spawn(
        provider: Arc<dyn RealtimeProvider>,
        options: SessionOptions,
        transport: Transport,
    ) -> (
        Self,
        SessionEvents,
        oneshot::Receiver<Result<(), RealtimeError>>,
    ) {
        let connection_id = crate::protocol::hyphenless_uuid();
        let per_connection = provider.create_protocol(&ConnectionInfo {
            connection_id: connection_id.clone(),
        });
        let dialect = Dialect {
            capabilities: provider.capabilities(),
            per_connection,
            provider: Arc::clone(&provider),
        };

        let (commands, command_receiver) = mpsc::channel(options.command_capacity.max(1));
        let (jobs, job_receiver) = mpsc::channel(options.command_capacity.max(1));
        let (events, event_receiver) = mpsc::channel(options.event_capacity.max(1));
        let (ready_signal, ready) = oneshot::channel();
        let generation = Arc::new(AtomicU64::new(0));

        let response_start_timeout = options
            .response_start_timeout
            .or_else(|| provider.response_start_timeout())
            .unwrap_or(DEFAULT_RESPONSE_START_TIMEOUT);
        let response_inactivity_timeout = options
            .response_inactivity_timeout
            .unwrap_or(DEFAULT_RESPONSE_INACTIVITY_TIMEOUT);

        let state = State::new(StateSetup {
            dialect: dialect.clone(),
            mode: options.mode,
            locale: options.locale,
            agent_context: options.agent_context,
            response_start_timeout,
            response_inactivity_timeout,
            sink: transport.sink,
            generation: Arc::clone(&generation),
            events,
            commands: commands.clone(),
            ready_signal,
        });
        tokio::spawn(state.run(command_receiver));
        tokio::spawn(queue::run(
            QueueContext {
                label: provider.label().to_owned(),
                dialect: dialect.clone(),
                commands: commands.clone(),
            },
            job_receiver,
        ));
        tokio::spawn(read_frames(transport.stream, commands.clone()));

        let session = Self {
            inner: Arc::new(Inner {
                mode: options.mode,
                connection_id,
                dialect,
                provider,
                generation,
                commands,
                jobs,
            }),
        };
        (
            session,
            SessionEvents {
                receiver: event_receiver,
            },
            ready,
        )
    }

    // ── identity ────────────────────────────────────────────────────────────

    /// The provider this session is talking to.
    #[must_use]
    pub fn provider(&self) -> &Arc<dyn RealtimeProvider> {
        &self.inner.provider
    }

    /// This connection's id, as handed to
    /// [`RealtimeProvider::create_protocol`].
    #[must_use]
    pub fn connection_id(&self) -> &str {
        &self.inner.connection_id
    }

    /// Which layers this session mounts.
    #[must_use]
    pub fn mode(&self) -> SessionMode {
        self.inner.mode
    }

    /// The capabilities snapshot taken when the session opened.
    #[must_use]
    pub fn capabilities(&self) -> ProviderCapabilities {
        self.inner.dialect.capabilities()
    }

    /// The rate the client must capture at.
    #[must_use]
    pub fn input_sample_rate(&self) -> u32 {
        self.inner.provider.input_sample_rate()
    }

    // ── audio ───────────────────────────────────────────────────────────────

    /// Append a chunk of base64 PCM to the input buffer.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::ConnectionClosed`] once the session is finished.
    pub async fn append_audio(&self, audio: &str) -> Result<(), RealtimeError> {
        let frame = self.inner.dialect.protocol().audio_append(audio);
        self.send_frame(frame).await
    }

    /// Close the current input audio buffer.
    ///
    /// Neither upstream provider needs this — both run server-side turn
    /// detection — but a `dictation` session and any push-to-talk client do.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::ConnectionClosed`] once the session is finished.
    pub async fn commit_audio(&self) -> Result<(), RealtimeError> {
        let frame = self.inner.dialect.protocol().audio_commit();
        self.send_frame(frame).await
    }

    /// Write one already-built frame.
    ///
    /// The escape hatch a provider extension needs for a dialect-specific frame
    /// this crate has no name for. It goes through the same door as everything
    /// else, so a `response.create` written this way is still correlated.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::ConnectionClosed`] once the session is finished.
    pub async fn send_frame(&self, payload: Value) -> Result<(), RealtimeError> {
        self.inner
            .commands
            .send(Command::Send(payload))
            .await
            .map_err(|_| self.closed())
    }

    // ── the session payload ─────────────────────────────────────────────────

    /// Merge a patch into the agent context and refresh the live session.
    ///
    /// The patch is applied at once; the `session.update` is queued behind
    /// whatever is speaking, because re-configuring a session mid-response is
    /// how a provider loses its place.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::ConnectionClosed`] once the session is finished.
    pub async fn update_agent_context(
        &self,
        patch: AgentContextPatch,
    ) -> Result<(), RealtimeError> {
        let (reply, ready) = oneshot::channel();
        self.inner
            .commands
            .send(Command::PatchContext { patch, reply })
            .await
            .map_err(|_| self.closed())?;
        if !ready.await.map_err(|_| self.closed())? {
            return Ok(());
        }
        let (reply, _done) = oneshot::channel();
        self.inner
            .jobs
            .send(Job::Action {
                kind: ActionKind::RefreshSession,
                generation: self.generation(),
                reply,
            })
            .await
            .map_err(|_| self.closed())
    }

    /// Resolve once everything already queued has resolved.
    ///
    /// The Rust spelling of upstream's `await frontend.outputQueue`.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::ConnectionClosed`] once the session is finished.
    pub async fn drain(&self) -> Result<(), RealtimeError> {
        self.action(ActionKind::Barrier).await.map(|_| ())
    }

    // ── conversation input ──────────────────────────────────────────────────

    /// Add text to the conversation without asking for an answer.
    ///
    /// # Errors
    ///
    /// Whatever the provider says about the item, or
    /// [`RealtimeError::ConnectionClosed`].
    pub async fn append_user_context(&self, text: &str) -> Result<bool, RealtimeError> {
        let content = text.trim();
        if content.is_empty() {
            return Ok(false);
        }
        let item = self.inner.dialect.protocol().user_text_item(content);
        self.action(ActionKind::CreateItem(item)).await
    }

    /// Add projected input to the conversation without asking for an answer.
    ///
    /// # Errors
    ///
    /// Whatever the provider says about the item, or
    /// [`RealtimeError::ConnectionClosed`].
    pub async fn append_user_input_context(
        &self,
        projection: Option<InputProjection>,
    ) -> Result<bool, RealtimeError> {
        self.action(ActionKind::ApplyInput(projection)).await
    }

    /// Ask the provider to project input parts, falling back to plain text.
    ///
    /// Upstream's `projectUserInput`, with one substitution: its default is
    /// `frontendInputProjection(parts, options)` from `shared/input-parts.mjs`,
    /// which lives in the prompt layer rather than in the transport, so the
    /// already-composed text arrives as `fallback_text`.
    #[must_use]
    pub fn project_user_input(
        &self,
        parts: &Value,
        options: &Value,
        fallback_text: Option<&str>,
    ) -> Option<InputProjection> {
        let profile = self.inner.provider.model_profile();
        if let Some(custom) = self
            .inner
            .provider
            .project_user_input(&InputProjectionRequest {
                parts,
                options,
                model_profile: profile.as_ref(),
            })
        {
            return Some(custom);
        }
        let text = fallback_text.filter(|text| !text.is_empty())?;
        Some(InputProjection::text(
            self.inner.dialect.protocol().user_text_item(text),
        ))
    }

    // ── responses ───────────────────────────────────────────────────────────

    /// Submit a typed user turn and ask for an answer.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::ConnectionClosed`] once the session is finished. A
    /// provider refusal is reported as the outcome, not as an error.
    pub async fn send_user_text(
        &self,
        text: &str,
        context: ResponseContext,
        modalities: Option<Vec<String>>,
    ) -> Result<Option<ResponseOutcome>, RealtimeError> {
        let content = text.trim();
        if content.is_empty() {
            return Ok(None);
        }
        self.response(
            ResponseOrigin::Model,
            context,
            CreateKind::UserText {
                text: content.to_owned(),
                modalities,
            },
        )
        .await
    }

    /// Submit projected input and ask for an answer.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::ConnectionClosed`] once the session is finished.
    pub async fn send_user_input(
        &self,
        projection: Option<InputProjection>,
        context: ResponseContext,
        modalities: Option<Vec<String>>,
    ) -> Result<Option<ResponseOutcome>, RealtimeError> {
        self.response(
            ResponseOrigin::Model,
            context,
            CreateKind::UserInput {
                projection,
                modalities,
            },
        )
        .await
    }

    /// Ask for a response with no new input.
    ///
    /// `guard` is evaluated when the job reaches the head of the queue, so a
    /// response that has stopped being wanted is never created.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::ConnectionClosed`] once the session is finished.
    pub async fn ensure_response(
        &self,
        context: ResponseContext,
        response: Option<Value>,
        guard: Option<Guard>,
    ) -> Result<Option<ResponseOutcome>, RealtimeError> {
        self.response(
            ResponseOrigin::Agent,
            context,
            CreateKind::Ensure { response, guard },
        )
        .await
    }

    /// Return a tool result, and optionally ask the model to continue.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::ConnectionClosed`] once the session is finished; for
    /// [`FunctionOutputOptions::without_response`], also whatever the provider
    /// says about the item.
    pub async fn send_function_output(
        &self,
        call_id: &str,
        output: Value,
        context: ResponseContext,
        options: FunctionOutputOptions,
    ) -> Result<Option<ResponseOutcome>, RealtimeError> {
        if !options.create_response {
            self.action(ActionKind::FunctionOutput {
                call_id: call_id.to_owned(),
                output,
            })
            .await?;
            return Ok(None);
        }
        self.response(
            ResponseOrigin::Agent,
            context,
            CreateKind::FunctionOutput {
                call_id: call_id.to_owned(),
                output,
                response: options.response,
            },
        )
        .await
    }

    /// Say something out of band.
    ///
    /// The response carries `conversation: 'none'`, so the utterance never
    /// becomes part of conversation history — which is what keeps a spoken
    /// progress note from being answered later as though the user had said it.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::ConnectionClosed`] once the session is finished.
    pub async fn speak(
        &self,
        text: &str,
        origin: ResponseOrigin,
        context: ResponseContext,
        guard: Option<Guard>,
    ) -> Result<Option<ResponseOutcome>, RealtimeError> {
        let content = text.trim();
        if content.is_empty() {
            return Ok(None);
        }
        self.response(
            origin,
            context,
            CreateKind::Speak {
                content: content.to_owned(),
                guard,
            },
        )
        .await
    }

    /// Deliver a finished background result.
    ///
    /// `inject_context` decides whether the result also becomes a conversation
    /// item. The returned [`InjectOutcome::context_injected`] says whether it
    /// did, which is what stops a retried announcement from injecting it twice.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::ConnectionClosed`] once the session is finished.
    pub async fn inject_result(
        &self,
        text: &str,
        origin: ResponseOrigin,
        context: ResponseContext,
        inject_context: bool,
    ) -> Result<Option<InjectOutcome>, RealtimeError> {
        let content = text.trim();
        if content.is_empty() {
            return Ok(None);
        }
        let injection = self.inner.provider.build_result_injection(content);
        let injected = Arc::new(AtomicBool::new(false));
        let outcome = self
            .response(
                origin,
                context,
                CreateKind::InjectResult {
                    item: injection.item,
                    response: injection.response,
                    inject_context,
                    injected: Arc::clone(&injected),
                },
            )
            .await?;
        Ok(Some(InjectOutcome {
            outcome,
            context_injected: injected.load(Ordering::SeqCst),
        }))
    }

    /// Ask the user about a backend authorization.
    ///
    /// The item is created **outside** the queue, so the model knows the
    /// permission's identity immediately even while the spoken question waits
    /// behind an active response — the user can already see and answer it in the
    /// TUI or Web UI.
    ///
    /// # Errors
    ///
    /// Whatever the provider says about the item, or
    /// [`RealtimeError::ConnectionClosed`].
    pub async fn inject_permission(
        &self,
        permission: &PermissionRequest,
        context: ResponseContext,
        guard: Option<Guard>,
    ) -> Result<Option<ResponseOutcome>, RealtimeError> {
        if permission.id.is_empty() || permission.summary.is_empty() {
            return Ok(None);
        }
        if !has_model_turn(self.inner.mode) {
            return Ok(Some(no_model_turn()));
        }
        let injection = self.inner.provider.build_permission_injection(permission);
        let (reply, receipt) = oneshot::channel();
        self.inner
            .commands
            .send(Command::CreateItem {
                item: injection.item,
                reply,
            })
            .await
            .map_err(|_| self.closed())?;
        receipt.await.map_err(|_| self.closed())??;

        self.response(
            ResponseOrigin::Permission,
            context,
            CreateKind::InjectPermission {
                response: injection.response,
                guard,
            },
        )
        .await
    }

    // ── cancellation ────────────────────────────────────────────────────────

    /// Cancel everything in flight and invalidate everything queued.
    ///
    /// Barge-in. The queue generation is bumped, so a job that has not started
    /// yet is dropped rather than run against a conversation that has moved on.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::ConnectionClosed`] once the session is finished.
    pub async fn cancel(&self) -> Result<(), RealtimeError> {
        self.inner
            .commands
            .send(Command::Cancel)
            .await
            .map_err(|_| self.closed())
    }

    /// Cancel only the responses a predicate matches.
    ///
    /// Answers whether an already-*started* response was cancelled, which is the
    /// only case that needed a `response.cancel` on the wire.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::ConnectionClosed`] once the session is finished.
    pub async fn cancel_responses<F>(&self, predicate: F) -> Result<bool, RealtimeError>
    where
        F: Fn(&ResponseContext, ResponseOrigin) -> bool + Send + Sync + 'static,
    {
        let predicate: ResponsePredicate = Arc::new(predicate);
        let (reply, cancelled) = oneshot::channel();
        self.inner
            .commands
            .send(Command::CancelResponses { predicate, reply })
            .await
            .map_err(|_| self.closed())?;
        cancelled.await.map_err(|_| self.closed())
    }

    /// Close the socket and settle everything.
    ///
    /// Idempotent, and implied by dropping the last handle.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::ConnectionClosed`] when it was already closed.
    pub async fn close(&self) -> Result<(), RealtimeError> {
        self.inner
            .commands
            .send(Command::Close)
            .await
            .map_err(|_| self.closed())
    }

    // ── plumbing ────────────────────────────────────────────────────────────

    fn closed(&self) -> RealtimeError {
        RealtimeError::ConnectionClosed {
            label: self.inner.provider.label().to_owned(),
        }
    }

    /// The queue generation, read **at call time**.
    ///
    /// Upstream captures `const generation = this.responseQueueGeneration`
    /// synchronously inside `enqueueResponse`, before the job is chained — so a
    /// `cancel()` that lands after the call but before the job runs still
    /// invalidates it. Only the state task writes this; every other reader is a
    /// snapshot.
    fn generation(&self) -> u64 {
        self.inner.generation.load(Ordering::SeqCst)
    }

    async fn action(&self, kind: ActionKind) -> Result<bool, RealtimeError> {
        let (reply, done) = oneshot::channel();
        self.inner
            .jobs
            .send(Job::Action {
                kind,
                generation: self.generation(),
                reply,
            })
            .await
            .map_err(|_| self.closed())?;
        done.await.map_err(|_| self.closed())?
    }

    async fn response(
        &self,
        origin: ResponseOrigin,
        context: ResponseContext,
        create: CreateKind,
    ) -> Result<Option<ResponseOutcome>, RealtimeError> {
        if !has_model_turn(self.inner.mode) {
            // `dictation` mounts no model, so there is no turn to create. Said
            // out loud as an outcome rather than as an error: the caller asked
            // for something this mode does not do, which is the same shape as a
            // guard declining.
            return Ok(Some(no_model_turn()));
        }
        let (reply, outcome) = oneshot::channel();
        self.inner
            .jobs
            .send(Job::Response {
                origin,
                context,
                generation: self.generation(),
                create,
                reply,
            })
            .await
            .map_err(|_| self.closed())?;
        outcome.await.map_err(|_| self.closed())
    }
}

fn no_model_turn() -> ResponseOutcome {
    ResponseOutcome::new(OutcomeKind::Skipped).with_phase(OutcomePhase::NoModelTurn)
}

/// Pump inbound frames into the state task.
///
/// A frame that is not JSON is dropped without comment, exactly as upstream's
/// `try { JSON.parse } catch { return }`: a provider that sends a keep-alive or a
/// stray text frame must not take the session down.
async fn read_frames(mut stream: WsStream, commands: mpsc::Sender<Command>) {
    while let Some(frame) = stream.next().await {
        let command = match frame {
            Ok(Message::Text(text)) => match serde_json::from_str::<Value>(&text) {
                Ok(value) => Command::Inbound(value),
                Err(_) => continue,
            },
            Ok(Message::Binary(bytes)) => match serde_json::from_slice::<Value>(&bytes) {
                Ok(value) => Command::Inbound(value),
                Err(_) => continue,
            },
            Ok(Message::Close(_)) => break,
            Ok(_) => continue,
            Err(detail) => {
                let _ = commands.send(Command::TransportError(detail)).await;
                break;
            }
        };
        if commands.send(command).await.is_err() {
            return;
        }
    }
    let _ = commands.send(Command::SocketClosed).await;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn only_dictation_lacks_a_model_turn() {
        assert!(!has_model_turn(SessionMode::Dictation));
        assert!(has_model_turn(SessionMode::Direct));
        assert!(has_model_turn(SessionMode::Agent));
        assert!(has_model_turn(SessionMode::Interface));
    }

    #[test]
    fn the_catalogued_timeouts_are_the_defaults() {
        assert_eq!(CONNECT_TIMEOUT, Duration::from_millis(25_000));
        assert_eq!(
            DEFAULT_RESPONSE_START_TIMEOUT,
            Duration::from_millis(30_000)
        );
        assert_eq!(
            DEFAULT_RESPONSE_INACTIVITY_TIMEOUT,
            Duration::from_millis(120_000)
        );
        assert_eq!(POST_CANCEL_RECOVERY, Duration::from_millis(1000));
        assert_eq!(MAX_BUSY_RETRIES, 3);
        assert_eq!(
            BUSY_RETRY_DELAYS.map(|delay| delay.as_millis()),
            [1200, 2600, 5000]
        );
    }

    #[test]
    fn the_default_options_mount_the_whole_stack() {
        let options = SessionOptions::default();
        assert_eq!(options.mode, SessionMode::Agent);
        assert_eq!(options.locale, Locale::En);
        assert_eq!(options.connect_timeout, None);
    }

    #[test]
    fn function_output_options_spell_both_intentions() {
        assert!(FunctionOutputOptions::with_response().create_response);
        assert!(!FunctionOutputOptions::without_response().create_response);
        // Upstream's parameter default is `createResponse = true`; `Default`
        // here is the *safe* half, so a caller that forgets is silent rather
        // than surprising the user with speech.
        assert!(!FunctionOutputOptions::default().create_response);
    }
}
