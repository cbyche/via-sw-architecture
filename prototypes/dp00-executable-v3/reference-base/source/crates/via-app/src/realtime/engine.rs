//! The model turn, behind one trait.
//!
//! `docs/deviations/phase-5-via-voice.md` splits upstream's
//! `realtime-gateway.mjs` in two: *"the gateway's socket plumbing is
//! `via-app`'s"*, and the decisions that plumbing makes are `via-voice`'s. This
//! trait is where the split lands at runtime — everything a connection does
//! that needs a **live realtime session** goes through it, and everything else
//! ([`crate::realtime::Connection`]) is implemented directly.
//!
//! # Why a trait and not `via_realtime::RealtimeSession`
//!
//! Three reasons, and the third is the one that matters:
//!
//! 1. `dictation` mounts no model at all (`docs/architecture.md` §2), so a
//!    connection must be able to run with no engine rather than with a broken
//!    one.
//! 2. `agent` with no harness degrades to `direct` — a [`via_voice::ModePlan`]
//!    decision the connection makes *before* it asks for an engine.
//! 3. It makes the socket testable. A `Connection` driven by
//!    [`RecordingEngine`] exercises every frame in the vocabulary with no
//!    provider, no credential and no network, which is what phase 5's *"`via
//!    chat` works end to end — no audio hardware, no model weights"* means.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tokio::sync::mpsc;

/// What a connection asks of a live realtime session.
///
/// Each method corresponds to one arm of upstream's `ws.on('message')`
/// (`realtime-gateway.mjs`) that reaches `frontend`.
#[async_trait]
pub trait VoiceEngine: Send + Sync + std::fmt::Debug {
    /// `frontend.appendAudio(event.audio)` — a base64 PCM chunk.
    ///
    /// The chunk is **not** decoded on the way through: the provider decides
    /// its own container, and decoding here would cost one copy per chunk for
    /// nothing.
    async fn append_audio(&self, _audio: &str) {}

    /// `submitInputMessage(event)` — a typed user turn.
    ///
    /// `turn_id` is minted by the connection
    /// ([`via_voice::new_text_turn_id`]) before this is called, because
    /// `turn.started` goes out first whether or not an engine exists.
    async fn submit_text(&self, _turn_id: &str, _text: &str) {}

    /// `frontend.cancel()` — barge-in, `mute`, and a host suspension.
    async fn cancel(&self) {}

    /// `startPlayback(id)` — the client began playing `response_id`'s audio.
    ///
    /// Called after [`via_voice::InjectionGate::playback_started`]
    /// (`crate::realtime::connection`'s job — it owns the shared gate); this
    /// is the half only the engine can do, because
    /// upstream's `responseContexts` map is the pump task's alone. Drives
    /// `voice.state: speaking`.
    async fn playback_started(&self, _response_id: &str) {}

    /// `finishPlayback(id)` — the client finished `response_id`'s audio.
    ///
    /// Drives `voice.state: listening` (the user has since started a new
    /// turn) or `voice.state: idle`.
    async fn playback_ended(&self, _response_id: &str) {}

    /// `cancelQueuedPlayback(id, {reason})` — the client dropped
    /// `response_id`'s queued audio without playing all of it.
    ///
    /// `reason` is `"user_interruption"` for a barge-in; a playback that had
    /// already started is what turns into `response.interrupted`.
    async fn playback_cancelled(&self, _response_id: &str, _reason: &str) {}

    /// `frontend.close()` — the socket is going away.
    async fn close(&self) {}

    /// Whether a realtime session is usable right now.
    ///
    /// Feeds [`via_voice::realtime_connection_status`], which is what
    /// `/api/health`'s `voiceClients.realtime` counts.
    fn ready(&self) -> bool {
        false
    }

    /// The provider key this engine is pointed at.
    fn provider(&self) -> &str;

    /// The provider's label, for `voice.ready`.
    fn provider_label(&self) -> &str {
        self.provider()
    }

    /// The rate the client must capture at.
    fn input_sample_rate(&self) -> u32;
}

/// The engine a session with no model runs.
///
/// This is `dictation`'s engine and the default everywhere else until a
/// provider is registered. It is not an error state: a Gateway with no realtime
/// credential still serves `/api/health`, still relays input suspensions, still
/// forwards the Work plane, and still lets a client hold the voice slot. It
/// simply never produces a model turn.
#[derive(Debug, Clone)]
pub struct NoModelEngine {
    provider: String,
    input_sample_rate: u32,
}

impl NoModelEngine {
    /// An engine for `provider` that captures at `input_sample_rate`.
    #[must_use]
    pub fn new(provider: impl Into<String>, input_sample_rate: u32) -> Self {
        Self {
            provider: provider.into(),
            input_sample_rate,
        }
    }
}

#[async_trait]
impl VoiceEngine for NoModelEngine {
    fn provider(&self) -> &str {
        &self.provider
    }

    fn input_sample_rate(&self) -> u32 {
        self.input_sample_rate
    }
}

/// The Injection Gate, shared between the socket and the engine.
///
/// The socket feeds it — barge-in, a new user turn, the playback receipts, the
/// output-enabled flag — and the engine reads it, because
/// [`via_voice::AnnouncementManager`]'s blocking predicate *is* this gate
/// (`docs/architecture.md` §11, invariant 3). Two gates would drift the moment
/// one of them missed a receipt, and the symptom would be a result spoken over
/// the user.
///
/// A `std::sync::Mutex` rather than an owning task: every operation is a
/// handful of field writes with no ordering requirement *between callers* —
/// the ordering that matters is inside [`via_voice::AnnouncementWindow`], and
/// it already owns it.
pub type SharedGate = Arc<Mutex<via_voice::InjectionGate>>;

/// What a connection tells the factory about itself.
///
/// Upstream's `ensureFrontend()` is a closure over the whole connection scope,
/// so it can reach the socket, the owner, the session and the announcement
/// window without being handed any of them. A trait cannot close over a scope,
/// so the four things the closure actually uses are passed instead.
pub struct EngineContext {
    /// Which realtime front end this connection selected.
    pub provider: String,
    /// Whose session this is.
    pub owner_id: String,
    /// Which conversation, from `?sessionId=`.
    pub session_id: String,
    /// Where a frame this engine produces goes.
    ///
    /// The same bounded queue the socket's own frames go through, so a
    /// `transcript.delta` the model produced cannot overtake the
    /// `turn.started` the socket sent.
    pub outbound: mpsc::Sender<crate::realtime::frames::ServerFrame>,
    /// The Injection Gate this connection feeds.
    pub gate: SharedGate,
    /// The states the client's `connect` frame declared it manages itself,
    /// e.g. `sleeping` — `via_voice::tools::ClientContext::states`, and the
    /// gate `enter_sleep` reads before answering or calling
    /// `request_client_state`.
    ///
    /// Only this one field of the connect-time client context is threaded
    /// through today; `timeZone` / `locale` / `workingDirectory` still reach
    /// every tool as their defaults, which is a pre-existing gap this does
    /// not close.
    pub client_states: Vec<String>,
}

impl std::fmt::Debug for EngineContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineContext")
            .field("provider", &self.provider)
            .field("owner_id", &self.owner_id)
            .field("session_id", &self.session_id)
            .finish_non_exhaustive()
    }
}

/// Opens an engine for one connection.
///
/// Upstream's `ensureFrontend()` closure, lifted so a test can supply one that
/// never touches a socket.
#[async_trait]
pub trait EngineFactory: Send + Sync + std::fmt::Debug {
    /// Open an engine for `provider`, or answer `None` when this session runs
    /// no model at all.
    async fn open(&self, provider: &str) -> Option<Arc<dyn VoiceEngine>>;

    /// Open an engine that can also *write* to this connection.
    ///
    /// The default forwards to [`open`](Self::open) and drops the rest of the
    /// context, which is right for every engine that only consumes — the
    /// [`NoEngineFactory`] and [`RecordingEngineFactory`] both do.
    /// A real realtime binding overrides this, because a model turn is only
    /// observable as frames on the socket.
    async fn open_for(&self, context: EngineContext) -> Option<Arc<dyn VoiceEngine>> {
        self.open(&context.provider).await
    }
}

/// The factory a Gateway with no realtime binding uses.
///
/// Answers `None`, so every connection runs with no engine.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoEngineFactory;

#[async_trait]
impl EngineFactory for NoEngineFactory {
    async fn open(&self, _provider: &str) -> Option<Arc<dyn VoiceEngine>> {
        None
    }
}

/// An engine that records what it was asked to do.
///
/// The socket's test double: it lets a test assert *"this frame reached the
/// model layer"* without a provider.
#[derive(Debug, Default)]
pub struct RecordingEngine {
    provider: String,
    input_sample_rate: u32,
    calls: Mutex<Vec<EngineCall>>,
}

/// One call [`RecordingEngine`] saw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineCall {
    /// [`VoiceEngine::append_audio`].
    AppendAudio(String),
    /// [`VoiceEngine::submit_text`].
    SubmitText {
        /// The turn the connection minted.
        turn_id: String,
        /// What the user typed.
        text: String,
    },
    /// [`VoiceEngine::cancel`].
    Cancel,
    /// [`VoiceEngine::close`].
    Close,
    /// [`VoiceEngine::playback_started`].
    PlaybackStarted(String),
    /// [`VoiceEngine::playback_ended`].
    PlaybackEnded(String),
    /// [`VoiceEngine::playback_cancelled`].
    PlaybackCancelled {
        /// The response whose audio was dropped.
        response_id: String,
        /// Why, when the client said.
        reason: String,
    },
}

impl RecordingEngine {
    /// A recording engine for `provider`.
    #[must_use]
    pub fn new(provider: impl Into<String>, input_sample_rate: u32) -> Self {
        Self {
            provider: provider.into(),
            input_sample_rate,
            calls: Mutex::new(Vec::new()),
        }
    }

    /// What it has been asked to do, in order.
    #[must_use]
    pub fn calls(&self) -> Vec<EngineCall> {
        self.calls
            .lock()
            .map(|calls| calls.clone())
            .unwrap_or_default()
    }

    fn record(&self, call: EngineCall) {
        if let Ok(mut calls) = self.calls.lock() {
            calls.push(call);
        }
    }
}

#[async_trait]
impl VoiceEngine for RecordingEngine {
    async fn append_audio(&self, audio: &str) {
        self.record(EngineCall::AppendAudio(audio.to_owned()));
    }

    async fn submit_text(&self, turn_id: &str, text: &str) {
        self.record(EngineCall::SubmitText {
            turn_id: turn_id.to_owned(),
            text: text.to_owned(),
        });
    }

    async fn cancel(&self) {
        self.record(EngineCall::Cancel);
    }

    async fn close(&self) {
        self.record(EngineCall::Close);
    }

    async fn playback_started(&self, response_id: &str) {
        self.record(EngineCall::PlaybackStarted(response_id.to_owned()));
    }

    async fn playback_ended(&self, response_id: &str) {
        self.record(EngineCall::PlaybackEnded(response_id.to_owned()));
    }

    async fn playback_cancelled(&self, response_id: &str, reason: &str) {
        self.record(EngineCall::PlaybackCancelled {
            response_id: response_id.to_owned(),
            reason: reason.to_owned(),
        });
    }

    fn ready(&self) -> bool {
        true
    }

    fn provider(&self) -> &str {
        &self.provider
    }

    fn input_sample_rate(&self) -> u32 {
        self.input_sample_rate
    }
}

/// A factory that hands out one shared [`RecordingEngine`].
#[derive(Debug)]
pub struct RecordingEngineFactory(Arc<RecordingEngine>);

impl RecordingEngineFactory {
    /// A factory over `engine`.
    #[must_use]
    pub const fn new(engine: Arc<RecordingEngine>) -> Self {
        Self(engine)
    }
}

#[async_trait]
impl EngineFactory for RecordingEngineFactory {
    async fn open(&self, _provider: &str) -> Option<Arc<dyn VoiceEngine>> {
        Some(self.0.clone())
    }
}
