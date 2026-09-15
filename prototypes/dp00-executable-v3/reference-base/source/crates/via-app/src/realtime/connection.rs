//! One WebSocket connection.
//!
//! Ported from the per-connection half of
//! `server/src/voice/realtime-gateway.mjs`. Everything here is *decided* by
//! `via-voice` and *plumbed* here; where the two might drift, the `via-voice`
//! item is called rather than re-derived —
//! [`via_voice::upgrade_decision`], [`via_voice::client_voice_capabilities`],
//! [`via_voice::accepts_playback_receipt`] and [`via_voice::ModePlan`].
//!
//! # Three fan-ins, one task
//!
//! A connection selects over exactly three sources, and owning all three in one
//! task is what makes the ordering between them defined:
//!
//! | Source | Carries |
//! | --- | --- |
//! | the socket | the sixteen client events |
//! | [`InputArbitration::subscribe`](via_voice::InputArbitration::subscribe) | the host taking and releasing the microphone |
//! | [`WorkManager::subscribe`](via_work::WorkManager::subscribe) | the Work plane |
//!
//! Outbound frames go the other way through one bounded `mpsc`, so a broadcast
//! raised by an HTTP handler or by another connection never writes to this
//! socket directly.

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use via_audio::{ChannelCount, SampleRate};
use via_i18n::{format, keys};
use via_protocol::{GatewayServerEvent, GatewayTaskEvent, WorkKind};
use via_voice::{
    InjectionGate, ModePlan, RealtimeStateInputs, SleepingWakeWord, WakeWordLifecycle,
    WakeWordPrepareInputs, WakeWordPrepareOutcome, accepts_playback_receipt,
    client_voice_capabilities, realtime_connection_status,
};

use crate::realtime::clients::{Connection as Registered, OUTBOUND_QUEUE_DEPTH, disconnected};
use crate::realtime::engine::{EngineContext, EngineFactory, SharedGate, VoiceEngine};
use crate::realtime::frames::{
    ClientDescriptor, ClientFrame, ServerFrame, delegated_inline_extra, error as error_frame,
    inline_block_extra, input_resume, input_suspend, playback_clear, task_frame, timeline_inline,
    voice_connection, voice_ready, voice_sleep,
};
use crate::state::AppState;

/// The default `sessionId` query parameter.
///
/// **External contract** — `realtime-gateway.mjs:158-177`: `sessionId` defaults
/// to `main`.
pub const DEFAULT_SESSION_ID: &str = "main";

/// The largest frame the socket accepts.
///
/// **External contract** — `docs/reference/contracts.json`, `ws-route` /
/// *WS /api/realtime*: `maxPayload 20*1024*1024 bytes`. Staged image parts are
/// what needs the headroom.
pub const MAX_PAYLOAD_BYTES: usize = 20 * 1024 * 1024;

/// Whether a Work event reaches a client on the socket.
///
/// **External contract** — `shared/realtime-events.mjs:52-67` plus the two
/// interceptions [`GatewayTaskEvent`] documents:
///
/// - an event outside the fourteen-name vocabulary is manager-only
///   (`task.accepted`, `task.notification.pending`,
///   `task.notification.delivered`);
/// - `task.progress.check` is **declared and not forwarded** — it drives a
///   spoken progress update instead (`realtime-gateway.mjs:831-859`);
/// - Work whose kind is [`WorkKind::Control`] is filtered out entirely, so no
///   task event for it ever reaches a client.
#[must_use]
pub fn forwards_to_client(event: &via_work::WorkEvent) -> bool {
    if event.task.kind == WorkKind::Control {
        return false;
    }
    match GatewayTaskEvent::from_wire(event.kind.as_str()) {
        Some(GatewayTaskEvent::ProgressCheck) | None => false,
        Some(_) => true,
    }
}

/// What one connection is told when the host takes or releases the microphone.
///
/// **External contract** — `realtime-gateway.mjs`'s `applyInputSuspension`: on
/// a suspension, `playback.clear{reason:'input_suspended'}` **then**
/// `input.suspend{owner,reason,expiresAt}`; on a release, `input.resume`. The
/// order backs the `input.suspend-clears-playback` capability — playback stops
/// *with* capture, so a host recording cannot pick up this Gateway's own
/// speech.
///
/// It is **idempotent**: a status that does not change `previous` produces no
/// frames at all. The arbitration already notifies only on edges, so this is
/// the second guard rather than the first — and it is the one that matters when
/// a subscriber lagged and the connection re-reads the current status instead
/// of replaying the edges it missed.
#[must_use]
pub fn suspension_frames(previous: bool, status: &via_voice::SuspensionStatus) -> Vec<ServerFrame> {
    if previous == status.suspended {
        return Vec::new();
    }
    if status.suspended {
        vec![
            playback_clear(Some("input_suspended")),
            input_suspend(status),
        ]
    } else {
        vec![input_resume()]
    }
}

/// One connection's mutable state.
struct Session {
    state: AppState,
    id: String,
    owner_id: String,
    session_id: String,
    outbound: mpsc::Sender<ServerFrame>,
    descriptor: ClientDescriptor,
    mode: ModePlan,
    engine: Option<Arc<dyn VoiceEngine>>,
    engines: Arc<dyn EngineFactory>,
    provider: String,
    /// Upstream's `nonVoiceClient` — a client that will never play audio.
    text_only: bool,
    input_enabled: bool,
    output_enabled: bool,
    /// Upstream's `inputSuspended`; defence in depth on `audio.append`.
    input_suspended: bool,
    sleeping: bool,
    waking: bool,
    /// The Injection Gate, shared with the engine — see [`SharedGate`].
    gate: SharedGate,
    /// The parts staged by `input.parts` for the next message.
    pending_parts: Vec<via_voice::InputPart>,
    /// The states the client declared it manages itself, from the `connect`
    /// frame's `clientStates` — carried into [`EngineContext::client_states`]
    /// so an engine opened after this connect still sees it.
    client_states: Vec<String>,
    /// The wake-word half of this connection: settings plus the engine that
    /// opens a detector — [`crate::Services::wake_word`], cloned once at
    /// accept time.
    wake_word: WakeWordLifecycle,
    /// The live detector, once [`Session::prepare_wake_word`] has built one.
    /// `None` before that, and again after a failed detection chain disables
    /// it for the rest of this connection.
    wake_word_stream: Option<SleepingWakeWord>,
    /// Audio that arrived before the engine was open — upstream's
    /// `pendingAudio` (`realtime-gateway.mjs:186`).
    ///
    /// A client that connects with `voiceEnabled: true` and starts streaming
    /// immediately never sends `unmute`, so without this its first words are
    /// simply lost. Bounded at [`via_voice::MAX_PENDING_AUDIO_CHUNKS`], keeping
    /// the **newest** chunks: replaying a minute of stale speech on connect is
    /// worse than replaying the last second.
    pending_audio: Vec<String>,
    /// Whether a *lazy* engine open has already failed on this connection.
    ///
    /// Upstream guards its lazy open with `!connectPromise &&
    /// !scheduledRealtimeReconnect` (`realtime-gateway.mjs:2037`). VIA needs no
    /// `connectPromise` — [`Session::ensure_engine`] takes `&mut self`, so it
    /// cannot be entered twice concurrently — but it does need the other half
    /// of that guard's job: without it a failed open would be retried on every
    /// 20 ms chunk, so one unreachable provider becomes fifty connect attempts
    /// a second.
    ///
    /// Only the lazy path reads it. An explicit `unmute` or `wake` is a user
    /// asking again, and clears it.
    lazy_open_failed: bool,
}

impl Session {
    /// Apply one mutation to the shared gate.
    ///
    /// A poisoned lock answers `None` rather than panicking: a gate that
    /// cannot be read is a session that cannot speak a result, which is the
    /// safe failure. Every caller here is a fire-and-forget mutation.
    fn with_gate<R>(&self, apply: impl FnOnce(&mut InjectionGate) -> R) -> Option<R> {
        self.gate.lock().ok().map(|mut gate| apply(&mut gate))
    }

    fn send(&self, frame: ServerFrame) {
        // A full queue means a client that is not reading. Upstream's `ws.send`
        // buffers instead; either way the broadcaster is never stalled.
        let _ = self.outbound.try_send(frame);
    }

    fn is_active(&self) -> bool {
        self.state.clients().is_active(&self.owner_id, &self.id)
    }

    /// The two side effects of a Work event beyond the `{type, task}` frame
    /// [`task_frame`] already carries: a `timeline.inline` push, and — for a
    /// terminal result — the conversation record of it.
    ///
    /// **External contract** — `realtime-gateway.mjs:919-965`, both
    /// unconditional sends inside the same `taskManager.subscribe` callback
    /// that forwards the frame itself: neither depends on this connection
    /// currently holding the voice slot, so both run for every connection of
    /// the owner, not only the one presenting the result aloud.
    async fn on_work_event(&mut self, event: &via_work::WorkEvent) {
        match event.kind {
            via_work::WorkEventKind::Delegated => {
                if let Some(extra) = event
                    .task
                    .delegation
                    .as_ref()
                    .and_then(|delegation| delegation.presentation.as_ref())
                    .and_then(|presentation| presentation.inline.as_ref())
                    .and_then(delegated_inline_extra)
                {
                    self.send(timeline_inline(
                        format!("inline_{}_delegated", event.task.id),
                        &event.task.id,
                        event.task.turn_id.as_deref(),
                        extra,
                    ));
                }
            }
            via_work::WorkEventKind::Completed | via_work::WorkEventKind::Failed => {
                self.record_result(&event.task).await;
                if let Some(inline) = event
                    .task
                    .result_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.presentation.inline.as_ref())
                {
                    self.send(timeline_inline(
                        format!("inline_{}", event.task.id),
                        &event.task.id,
                        event.task.turn_id.as_deref(),
                        inline_block_extra(inline),
                    ));
                }
            }
            _ => {}
        }
    }

    /// `recordTaskResult` — `task-result-projector.mjs`, wired at the voice
    /// path's own points. [`via_work::projector::project`] names the ids and
    /// source; this only hands the projection to the conversation store.
    async fn record_result(&mut self, task: &via_work::PublicWork) {
        let Some(projection) = via_work::projector::project(&self.owner_id, &self.session_id, task)
        else {
            return;
        };
        let mut input = via_conversation::RecordInput::new(
            projection.owner_id,
            projection.session_id,
            projection.id,
            via_conversation::MessageRole::Assistant,
            projection.content.unwrap_or_default(),
            via_conversation::MessageSource::AgentResult,
        )
        .task(projection.task_id);
        if let Some(turn_id) = projection.turn_id {
            input = input.turn(turn_id);
        }
        let _ = self.state.services().conversation_sync.record(input).await;
    }

    fn broadcast_ownership(&self) {
        for (outbound, frame) in self.state.clients().ownership_broadcast(&self.owner_id) {
            let _ = outbound.try_send(frame);
        }
    }

    /// Upstream's `activateVoiceClient({takeover, enableInput, enableOutput})`.
    fn activate(&mut self, takeover: bool, enable_input: bool, enable_output: bool) -> bool {
        let granted = self.state.clients().activate(&self.id, takeover).granted;
        self.input_enabled = granted && enable_input;
        self.output_enabled = granted && enable_output;
        self.with_gate(|gate| gate.set_output_enabled(self.output_enabled));
        self.broadcast_ownership();
        granted
    }

    fn release(&mut self) {
        self.state.clients().release(&self.id);
        self.input_enabled = false;
        self.output_enabled = false;
        self.with_gate(|gate| gate.set_output_enabled(false));
        self.broadcast_ownership();
    }

    /// The `voice.connection` state for this session right now.
    ///
    /// **External contract** — the precedence
    /// [`via_realtime::realtime_connection_status`] already decides; this
    /// only supplies the inputs for the two frame-emitting call sites that
    /// are not [`Self::publish_status`]'s `/api/health` aggregation
    /// (`ensure_engine`'s `connecting` / `connected`).
    fn connection_status(
        &self,
        connecting: bool,
        blocked_error: &str,
    ) -> via_realtime::RealtimeConnectionStatus {
        via_realtime::realtime_connection_status(&via_realtime::RealtimeConnectionInputs {
            provider: self.provider.clone(),
            blocked_error: blocked_error.to_owned(),
            sleeping: self.sleeping,
            waking: self.waking,
            ready: self.engine.as_ref().is_some_and(|engine| engine.ready()),
            connecting,
        })
    }

    fn publish_status(&self) {
        let realtime = realtime_connection_status(&RealtimeStateInputs {
            provider: self.provider.clone(),
            blocked_error: String::new(),
            sleeping: self.sleeping,
            waking: self.waking,
            ready: self.engine.as_ref().is_some_and(|engine| engine.ready()),
            connecting: false,
        });
        self.state.clients().update(&self.id, self.mode, realtime);
    }

    /// Hold one chunk until an engine exists — upstream's `pendingAudio.push`
    /// plus its cap (`realtime-gateway.mjs:2030-2033`).
    ///
    /// Upstream's `splice(0, len - MAX)` drops from the **front**, so the
    /// buffer keeps the newest [`via_voice::MAX_PENDING_AUDIO_CHUNKS`]. Dropping
    /// the tail instead would replay the beginning of a sentence and discard
    /// its end, which is the wrong half to keep.
    fn buffer_audio(&mut self, audio: String) {
        self.pending_audio.push(audio);
        if self.pending_audio.len() > via_voice::MAX_PENDING_AUDIO_CHUNKS {
            let excess = self.pending_audio.len() - via_voice::MAX_PENDING_AUDIO_CHUNKS;
            self.pending_audio.drain(..excess);
        }
    }

    async fn ensure_engine(&mut self) {
        if self.engine.is_some() {
            return;
        }
        // `connectFrontendNow` — `realtime-gateway.mjs:1444-1448`.
        self.send(voice_connection(&self.connection_status(true, "")));
        self.engine = self
            .engines
            .open_for(EngineContext {
                provider: self.provider.clone(),
                owner_id: self.owner_id.clone(),
                session_id: self.session_id.clone(),
                outbound: self.outbound.clone(),
                gate: Arc::clone(&self.gate),
                client_states: self.client_states.clone(),
            })
            .await;
        if let Some(engine) = &self.engine {
            // `:1539-1562` — the successful half of `connectFrontendNow`,
            // `voice.connection` before `voice.ready`. The unavailable half
            // (`:1582-1588`, a failed connect) is the factory's to send: only
            // it has the refusal text, and only it knows which provider
            // actually failed to open
            // ([`crate::realtime::engine::EngineFactory::open_for`]).
            self.send(voice_connection(&self.connection_status(false, "")));
            self.send(voice_ready(
                engine.input_sample_rate(),
                engine.provider(),
                engine.provider_label(),
            ));
        }
        // `:1545-1546` — `pendingAudio.forEach(audio =>
        // createdFrontend.appendAudio(audio)); pendingAudio = []`. Flushing
        // here rather than at the call site covers every path that opens an
        // engine, so audio buffered during a close window replays on the
        // reconnect exactly as upstream's does.
        if let Some(engine) = self.engine.clone() {
            self.lazy_open_failed = false;
            for audio in std::mem::take(&mut self.pending_audio) {
                engine.append_audio(&audio).await;
            }
        }
        self.publish_status();
    }

    /// Upstream's `applyInputSuspension(status)`.
    ///
    /// The frames are [`suspension_frames`]'s; what is here is the one side
    /// effect that is not a frame — cancelling the model's response so a host
    /// recording does not capture this Gateway mid-sentence.
    async fn apply_suspension(&mut self, status: &via_voice::SuspensionStatus) {
        let frames = suspension_frames(self.input_suspended, status);
        if frames.is_empty() {
            return;
        }
        self.input_suspended = status.suspended;
        if status.suspended
            && let Some(engine) = &self.engine
        {
            engine.cancel().await;
        }
        for frame in frames {
            self.send(frame);
        }
    }

    /// The phrase every `voice.sleep` frame echoes.
    fn wake_word_text(&self) -> String {
        self.state.services().config.wake_word.clone()
    }

    /// `enterSleep` — `realtime-gateway.mjs:1608-1631`, the client-requested
    /// half: `sleep` and, below, a wake-word detection.
    async fn enter_sleep(&mut self) {
        self.sleeping = true;
        // `:1614` — `pendingAudio = []`. Audio buffered before sleeping was
        // meant for the model; once asleep it belongs to the wake-word
        // detector, and replaying it on wake would speak into the wrong turn.
        self.pending_audio.clear();
        self.with_gate(|gate| gate.set_sleeping(true));
        self.send(
            ServerFrame::new(GatewayServerEvent::VoiceSleep.as_str())
                .with("state", "sleeping")
                .with("wakeWord", self.wake_word_text()),
        );
        self.publish_status();
    }

    /// `wakeFromSleep` — `realtime-gateway.mjs:1740-1751`, minus the retry
    /// ladder `attemptWakeConnect` adds around `ensureFrontend()`: a failed
    /// engine open here is [`Self::ensure_engine`]'s own, already-handled,
    /// refusal path, not a second one.
    async fn wake_up(&mut self) {
        if !self.sleeping {
            return;
        }
        self.waking = true;
        self.with_gate(|gate| gate.set_waking(true));
        self.send(
            ServerFrame::new(GatewayServerEvent::VoiceSleep.as_str())
                .with("state", "waking")
                .with("wakeWord", self.wake_word_text()),
        );
        self.ensure_engine().await;
        self.sleeping = false;
        self.waking = false;
        self.with_gate(|gate| {
            gate.set_sleeping(false);
            gate.set_waking(false);
        });
        self.send(
            ServerFrame::new(GatewayServerEvent::VoiceSleep.as_str())
                .with("state", "awake")
                .with("wakeWord", self.wake_word_text()),
        );
        self.publish_status();
    }

    /// The rate `audio.append` arrives at — the selected provider's own
    /// `inputSampleRate`, exactly as `acceptSleepingAudio` reads it fresh on
    /// every call (`realtime-gateway.mjs:1971`) rather than caching it.
    fn capture_sample_rate(&self) -> SampleRate {
        self.state
            .services()
            .realtime_registry
            .resolve(Some(&self.provider))
            .ok()
            .and_then(|provider| SampleRate::try_from(provider.input_sample_rate()).ok())
            .unwrap_or(SampleRate::HZ_16000)
    }

    /// `prepareSleepMode` — `realtime-gateway.mjs:1632-1657`. A no-op once a
    /// stream already exists: VIA prepares at most once per connection, so
    /// there is no "already preparing" state to guard against the way
    /// upstream's `wakeDetectorPromise` does.
    async fn prepare_wake_word(&mut self) {
        if self.wake_word_stream.is_some() || !self.wake_word.is_enabled() {
            return;
        }
        // `preparing` is sent the instant the attempt starts, not after —
        // upstream sends it *before* `createSherpaWakeWordDetector`'s promise
        // resolves, and [`via_voice::WakeWordLifecycle::prepare`] cannot send
        // it itself without knowing how a caller builds a frame.
        self.send(voice_sleep::preparing(&self.wake_word_text()));
        let outcome = self
            .wake_word
            .prepare(
                self.state.locale(),
                WakeWordPrepareInputs {
                    non_voice_client: self.text_only,
                    input_suspended: self.input_suspended,
                },
                self.capture_sample_rate(),
                ChannelCount::MONO,
            )
            .await;
        match outcome {
            // The guard inside `prepare` refused after all — a race between
            // this check and the async attempt (a suspension landing mid-way,
            // say). The `preparing` frame already sent is upstream's own
            // shape too: it sends it unconditionally, before the guard.
            WakeWordPrepareOutcome::NotApplicable => {}
            WakeWordPrepareOutcome::Disabled { message } => {
                self.send(voice_sleep::disabled(&message));
            }
            WakeWordPrepareOutcome::Enabled(enabled) => {
                self.send(voice_sleep::enabled(
                    enabled.timeout_ms,
                    enabled.keyword.text(),
                ));
                self.wake_word_stream = Some(enabled.stream);
            }
        }
    }

    /// `requestExplicitSleep()` — `realtime-gateway.mjs:1690-1704`. The
    /// `connect{wakeWordOnly: true}` path: build a detector and, only once
    /// one exists, enter sleep without ever calling
    /// [`Self::ensure_engine`] — a wake-word-only client pays for no realtime
    /// session until the phrase actually fires.
    async fn request_explicit_sleep(&mut self) {
        if !self.wake_word.is_enabled() || self.text_only {
            return;
        }
        self.prepare_wake_word().await;
        if self.wake_word_stream.is_some() {
            self.enter_sleep().await;
        }
    }

    /// `acceptSleepingAudio` — `realtime-gateway.mjs:1963-1975`. A match
    /// wakes the session; a detector chain failure disables wake-word
    /// detection for the rest of this connection rather than leaving a
    /// session asleep with no way back in.
    async fn accept_sleeping_audio(&mut self, audio: &str) {
        let Some(stream) = self.wake_word_stream.as_mut() else {
            return;
        };
        match stream.accept_base64_pcm16le(audio) {
            Ok(Some(_detection)) => {
                self.send(voice_sleep::detected(&self.wake_word_text()));
                self.wake_up().await;
            }
            Ok(None) => {}
            Err(error) => {
                self.wake_word_stream = None;
                self.send(voice_sleep::disabled(&error.localized(self.state.locale())));
            }
        }
    }

    async fn handle(&mut self, frame: ClientFrame) {
        match frame {
            ClientFrame::Connect(connect) => self.on_connect(*connect).await,
            ClientFrame::Unmute { takeover } => {
                // An explicit unmute is the user asking again, so a previous
                // lazy-open failure stops suppressing the retry.
                self.lazy_open_failed = false;
                if self.text_only {
                    self.input_enabled = false;
                    self.output_enabled = true;
                    self.with_gate(|gate| gate.set_output_enabled(true));
                    self.broadcast_ownership();
                } else {
                    self.activate(takeover, true, true);
                }
                self.ensure_engine().await;
            }
            ClientFrame::InputUnmute { takeover } => {
                self.lazy_open_failed = false;
                if self.text_only {
                    return;
                }
                if self.is_active() {
                    self.input_enabled = true;
                    self.output_enabled = true;
                    self.with_gate(|gate| gate.set_output_enabled(true));
                    self.broadcast_ownership();
                } else {
                    self.activate(takeover, true, true);
                }
                if self.sleeping {
                    return;
                }
                self.ensure_engine().await;
            }
            ClientFrame::Mute => {
                self.release();
                self.sleeping = false;
                self.waking = false;
                self.with_gate(|gate| {
                    gate.set_sleeping(false);
                    gate.set_waking(false);
                    gate.window_mut().reset();
                });
                if let Some(engine) = self.engine.take() {
                    engine.close().await;
                }
                self.publish_status();
            }
            ClientFrame::InputMute => {
                self.input_enabled = false;
            }
            ClientFrame::AudioAppend { audio } => {
                // `realtime-gateway.mjs:1963-1975`: sleeping audio never
                // reaches the model at all — it goes to the wake-word
                // detector instead, when this connection has one.
                if self.sleeping {
                    self.accept_sleeping_audio(&audio).await;
                    return;
                }
                // Defence in depth, upstream's own words: a client that has not
                // yet acted on a suspension must not be able to feed audio
                // through it.
                if !self.input_enabled || self.input_suspended || !self.is_active() {
                    return;
                }
                // `realtime-gateway.mjs:2028-2040`. A ready engine takes the
                // chunk; anything else buffers it and opens one.
                //
                // The lazy open is what makes `voiceEnabled: true` mean what a
                // client thinks it means. `on_connect` activates the voice slot
                // but does not open an engine, and only `unmute` /
                // `input.unmute` / a text message / `wake` do — so a client
                // that connects and immediately streams (which is upstream's
                // own client behaviour) had every chunk dropped, and never saw
                // `voice.state: listening`, `turn.started`, or a transcript for
                // that speech.
                if let Some(engine) = &self.engine {
                    engine.append_audio(&audio).await;
                    return;
                }
                self.buffer_audio(audio);
                if !self.lazy_open_failed {
                    self.ensure_engine().await;
                    // `ensure_engine` leaves the engine `None` when the open
                    // failed, and has already told the client why.
                    self.lazy_open_failed = self.engine.is_none();
                }
            }
            ClientFrame::TextMessage { text, .. } | ClientFrame::InputMessage { text, .. } => {
                self.on_text(&text).await;
            }
            ClientFrame::InputParts { parts } => {
                // **External contract** — `realtime-gateway.mjs`:
                // `pendingInputParts = Array.isArray(parts) && parts.length
                //   ? inputFileParts(normalizeInputParts(parts)) : []`,
                // and a refusal is answered with an `error` frame. Note the
                // empty-array arm **clears** the staged set rather than being
                // ignored, which is how a client cancels an attachment.
                if parts.is_empty() {
                    self.pending_parts.clear();
                    return;
                }
                let mut decoded = Vec::with_capacity(parts.len());
                for part in parts {
                    // `input-parts.mjs`'s asymmetry, reproduced: a `null` entry
                    // is skipped, an unknown `type` is refused by name.
                    if part.is_null() {
                        continue;
                    }
                    let kind = part
                        .get("type")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .to_owned();
                    match serde_json::from_value::<via_voice::InputPart>(part) {
                        Ok(part) => decoded.push(part),
                        Err(_) => {
                            self.send(error_frame(&format(
                                self.state.locale(),
                                keys::INPUT_UNSUPPORTED_PART_TYPE,
                                &[("kind", &kind)],
                            )));
                            return;
                        }
                    }
                }
                // `via-voice` owns normalization and every refusal message.
                match via_voice::normalize_input_parts(&decoded, "", self.state.locale()) {
                    Ok(normalized) => {
                        self.pending_parts = via_voice::input_file_parts(&normalized);
                    }
                    Err(refusal) => self.send(error_frame(&refusal.to_string())),
                }
            }
            ClientFrame::Interrupt => {
                self.with_gate(|gate| gate.window_mut().interrupt());
                if let Some(engine) = &self.engine {
                    engine.cancel().await;
                }
            }
            ClientFrame::Sleep => self.enter_sleep().await,
            ClientFrame::Wake => {
                self.lazy_open_failed = false;
                self.wake_up().await;
            }
            ClientFrame::PlaybackStarted { response_id } => {
                if self.accepts_receipt() {
                    self.with_gate(|gate| gate.playback_started(&response_id, 0));
                    if let Some(engine) = &self.engine {
                        engine.playback_started(&response_id).await;
                    }
                }
            }
            ClientFrame::PlaybackEnded { response_id } => {
                if self.accepts_receipt() {
                    self.with_gate(|gate| gate.playback_finished(&response_id, false));
                    if let Some(engine) = &self.engine {
                        engine.playback_ended(&response_id).await;
                    }
                }
            }
            ClientFrame::PlaybackCancelled {
                response_id,
                reason,
            } => {
                if self.accepts_receipt() {
                    self.with_gate(|gate| gate.playback_finished(&response_id, false));
                    if let Some(engine) = &self.engine {
                        engine.playback_cancelled(&response_id, &reason).await;
                    }
                }
            }
            ClientFrame::InputSuspendAck { owner } => {
                // A host must not wait for this: pressing a key to record is
                // latency sensitive, so the acknowledgement only feeds status
                // display and timeout healing.
                self.state.services().logger.debug(
                    "input.suspend_acknowledged",
                    via_log::fields([
                        (
                            "clientType",
                            serde_json::to_value(self.descriptor.client_type)
                                .unwrap_or(serde_json::Value::Null),
                        ),
                        (
                            "owner",
                            if owner.is_empty() {
                                serde_json::Value::Null
                            } else {
                                serde_json::Value::String(owner)
                            },
                        ),
                    ]),
                    "",
                );
            }
        }
    }

    /// `acceptsPlaybackReceipt({outputEnabled, active, responseKnown})`.
    ///
    /// `responseKnown` is `true` here because the response-context table is the
    /// engine's, not the socket's; the two gates that are the socket's —
    /// output enabled and holding the slot — are checked.
    fn accepts_receipt(&self) -> bool {
        accepts_playback_receipt(self.output_enabled, self.is_active(), true)
    }

    async fn on_connect(&mut self, connect: crate::realtime::frames::Connect) {
        self.descriptor = connect.descriptor.clone();
        self.state
            .clients()
            .describe(&self.id, connect.descriptor.clone());
        self.text_only = connect.text_only;
        self.client_states.clone_from(&connect.client_states);

        // The mode is fixed for the connection's lifetime and chosen here
        // (`docs/architecture.md` §2). The degradation is reported, never
        // silent: it reaches `/api/health` through the client registry.
        self.mode = ModePlan::new(connect.mode, self.state.services().backend.enabled());

        // A client may pick a realtime front end per session. An unknown name
        // is reported instead of silently falling back, so a typo does not look
        // like a working session on the wrong provider.
        if let Some(requested) = &connect.provider
            && requested != &self.provider
        {
            match self
                .state
                .services()
                .realtime_registry
                .resolve(Some(requested))
            {
                Ok(provider) => {
                    self.provider = provider.key().to_owned();
                    if let Some(engine) = self.engine.take() {
                        engine.close().await;
                    }
                }
                Err(refusal) => {
                    self.send(error_frame(&refusal.to_string()));
                    return;
                }
            }
        }

        let capabilities = client_voice_capabilities(via_voice::DeclaredCapabilities {
            voice_enabled: connect.voice_enabled,
            input_enabled: connect.input_enabled,
            output_enabled: connect.output_enabled,
            text_only: self.text_only,
        });
        if capabilities.participates_in_voice_arbitration {
            self.activate(
                connect.takeover,
                capabilities.input_enabled,
                capabilities.output_enabled,
            );
        } else {
            self.state.clients().release(&self.id);
            self.input_enabled = capabilities.input_enabled;
            self.output_enabled = capabilities.output_enabled;
            self.with_gate(|gate| gate.set_output_enabled(self.output_enabled));
            self.broadcast_ownership();
        }
        self.publish_status();

        // A client that connects mid-suspension is told before it opens a
        // microphone (`server/test/input-suspend-protocol.test.mjs`).
        let status = self.state.services().input_arbitration.status().await;
        if status.suspended {
            self.input_suspended = true;
            self.send(playback_clear(Some("input_suspended")));
            self.send(input_suspend(&status));
        }

        // `realtime-gateway.mjs:1970-1974`: a wake-word-only client skips
        // `ensureFrontend()` entirely and goes straight to sleep, once a
        // detector actually exists.
        if connect.wake_word_only {
            self.request_explicit_sleep().await;
        }
    }

    async fn on_text(&mut self, text: &str) {
        if self.sleeping || self.waking {
            self.send(error_frame(&format(
                self.state.locale(),
                keys::REALTIME_ASLEEP,
                &[("wake_word", &self.state.services().config.wake_word)],
            )));
            return;
        }
        let turn_id = via_voice::new_text_turn_id();
        self.with_gate(|gate| {
            gate.window_mut().begin_turn(&turn_id);
            gate.window_mut().end_speech();
        });
        self.send(playback_clear(Some("user_interruption")));
        self.send(
            ServerFrame::new(GatewayServerEvent::TurnStarted.as_str())
                .with("turnId", turn_id.clone()),
        );
        self.send(
            ServerFrame::new(GatewayServerEvent::VoiceState.as_str())
                .with("state", "processing")
                .with("turnId", turn_id.clone())
                .with("origin", "model"),
        );
        self.pending_parts.clear();
        self.ensure_engine().await;
        if let Some(engine) = &self.engine {
            engine.submit_text(&turn_id, text).await;
        }
    }
}

/// Run one accepted WebSocket connection to completion.
pub async fn run(socket: WebSocket, state: AppState, owner_id: String, session_id: String) {
    let (mut sink, mut stream) = socket.split();
    let (outbound, mut outbox) = mpsc::channel::<ServerFrame>(OUTBOUND_QUEUE_DEPTH);
    let id = crate::realtime::clients::new_connection_id();
    let provider = state
        .services()
        .realtime_provider
        .clone()
        .unwrap_or_default();

    state.clients().insert(Registered {
        id: id.clone(),
        owner_id: owner_id.clone(),
        descriptor: ClientDescriptor::default(),
        mode: ModePlan::new(
            via_protocol::SessionMode::default(),
            state.services().backend.enabled(),
        ),
        realtime: disconnected(&provider),
        outbound: outbound.clone(),
    });

    let mut suspensions = state.services().input_arbitration.subscribe();
    let mut work_events = state.services().work.subscribe();

    let mut session = Session {
        engines: state.engines(),
        state: state.clone(),
        id: id.clone(),
        owner_id: owner_id.clone(),
        session_id,
        outbound,
        descriptor: ClientDescriptor::default(),
        mode: ModePlan::new(
            via_protocol::SessionMode::default(),
            state.services().backend.enabled(),
        ),
        engine: None,
        provider,
        text_only: false,
        input_enabled: false,
        output_enabled: false,
        input_suspended: false,
        sleeping: false,
        waking: false,
        gate: Arc::new(std::sync::Mutex::new(InjectionGate::new(
            SampleRate::HZ_24000,
        ))),
        pending_parts: Vec::new(),
        client_states: Vec::new(),
        wake_word: state.services().wake_word.clone(),
        wake_word_stream: None,
        pending_audio: Vec::new(),
        lazy_open_failed: false,
    };

    state.services().logger.info(
        "voice_client.connected",
        via_log::fields([
            ("ownerId", owner_id.clone().into()),
            ("sessionId", session.session_id.clone().into()),
        ]),
        "",
    );

    loop {
        tokio::select! {
            // Outbound first: a frame already queued goes out before the next
            // inbound frame is handled, which is what keeps `playback.clear`
            // ahead of `input.suspend`.
            biased;
            frame = outbox.recv() => {
                let Some(frame) = frame else { break };
                if sink.send(Message::Text(frame.encode().into())).await.is_err() {
                    break;
                }
            }
            change = suspensions.recv() => {
                match change {
                    Ok(change) => session.apply_suspension(&change.status).await,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {}
                    // A lagged subscriber has missed edges, and the *current*
                    // status is the only thing that matters — so re-read it
                    // rather than replaying.
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        let status = session
                            .state
                            .services()
                            .input_arbitration
                            .status()
                            .await;
                        session.apply_suspension(&status).await;
                    }
                }
            }
            event = work_events.recv() => {
                if let Ok(event) = event
                    && event.owner_id == owner_id
                    && forwards_to_client(&event)
                {
                    session.send(task_frame(&event));
                    session.on_work_event(&event).await;
                }
            }
            message = stream.next() => {
                let Some(Ok(message)) = message else { break };
                match message {
                    Message::Text(text) => {
                        if let Some(frame) = ClientFrame::decode(&text) {
                            session.handle(frame).await;
                        }
                    }
                    // Binary frames are not part of this protocol: upstream
                    // parses every frame as JSON text and ignores what does not
                    // parse.
                    Message::Binary(_) | Message::Ping(_) | Message::Pong(_) => {}
                    Message::Close(_) => break,
                }
            }
        }
    }

    if let Some(engine) = session.engine.take() {
        engine.close().await;
    }
    let released = state.clients().remove(&id);
    if released {
        for (target, frame) in state.clients().ownership_broadcast(&owner_id) {
            let _ = target.try_send(frame);
        }
    }
    state.services().logger.info(
        "voice_client.disconnected",
        via_log::fields([("ownerId", owner_id.into())]),
        "",
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use via_work::{DelegationRef, WorkEvent, WorkEventKind};

    fn event(kind: WorkEventKind, work_kind: WorkKind) -> WorkEvent {
        let mut task = via_work::testing::blank_record("work_1", "user_1").to_public(0);
        task.kind = work_kind;
        WorkEvent::new(kind, task)
    }

    /// A `Session` with no live socket underneath it — enough to exercise
    /// [`Session::on_work_event`] without an accepted WebSocket, which is how
    /// the delegated-announcement half is tested: a real `task.delegated`
    /// can only otherwise be produced by driving a full coordinator.
    /// `realtime-gateway.mjs:2030-2033` — the cap, and *which* end it drops.
    #[tokio::test]
    async fn the_audio_buffer_keeps_the_newest_chunks_and_drops_the_oldest() {
        let (outbound, _rx) = mpsc::channel(OUTBOUND_QUEUE_DEPTH);
        let mut session = test_session(outbound);

        let overflow = via_voice::MAX_PENDING_AUDIO_CHUNKS + 5;
        for index in 0..overflow {
            session.buffer_audio(format!("chunk-{index}"));
        }

        assert_eq!(
            session.pending_audio.len(),
            via_voice::MAX_PENDING_AUDIO_CHUNKS,
            "the buffer is bounded",
        );
        assert_eq!(
            session.pending_audio.first().map(String::as_str),
            Some("chunk-5"),
            "the OLDEST five were dropped; keeping the tail instead would \
             replay the start of a sentence and discard its end",
        );
        assert_eq!(
            session.pending_audio.last().map(String::as_str),
            Some(format!("chunk-{}", overflow - 1)).as_deref(),
        );
    }

    /// `:1614` — `pendingAudio = []` inside `enterSleep`.
    #[tokio::test]
    async fn sleeping_drops_audio_that_was_meant_for_the_model() {
        let (outbound, _rx) = mpsc::channel(OUTBOUND_QUEUE_DEPTH);
        let mut session = test_session(outbound);
        session.buffer_audio("spoken-before-sleep".to_owned());

        session.enter_sleep().await;

        assert!(
            session.pending_audio.is_empty(),
            "once asleep the audio belongs to the wake-word detector, and \
             replaying it on wake would speak into the wrong turn",
        );
    }

    fn test_session(outbound: mpsc::Sender<ServerFrame>) -> Session {
        let state = AppState::new(
            crate::testing::test_services(),
            crate::state::InstanceIdentity::default(),
        );
        let wake_word = state.services().wake_word.clone();
        Session {
            engines: state.engines(),
            state,
            id: "conn_1".to_owned(),
            owner_id: "user_1".to_owned(),
            session_id: "main".to_owned(),
            outbound,
            descriptor: ClientDescriptor::default(),
            mode: ModePlan::new(via_protocol::SessionMode::default(), false),
            engine: None,
            provider: "mock".to_owned(),
            text_only: false,
            input_enabled: false,
            output_enabled: false,
            input_suspended: false,
            sleeping: false,
            waking: false,
            gate: Arc::new(std::sync::Mutex::new(InjectionGate::new(
                SampleRate::HZ_24000,
            ))),
            pending_parts: Vec::new(),
            client_states: Vec::new(),
            wake_word,
            wake_word_stream: None,
            pending_audio: Vec::new(),
            lazy_open_failed: false,
        }
    }

    #[tokio::test]
    async fn a_delegated_works_inline_announcement_carries_the_delegated_suffix() {
        let (outbound, mut outbox) = mpsc::channel::<ServerFrame>(8);
        let mut session = test_session(outbound);

        let mut record = via_work::testing::blank_record("work_1", "user_1");
        record.turn_id = Some("voice-1".to_owned());
        record.delegation = Some(
            DelegationRef::new("d1", "session-1").with_presentation(json!({
                "speech": "starting",
                "inline": {"content": "spawned", "kind": "spawn"},
            })),
        );
        let event = WorkEvent::new(WorkEventKind::Delegated, record.to_public(0));

        session.on_work_event(&event).await;

        let frame = outbox
            .try_recv()
            .expect("a timeline.inline frame")
            .into_value();
        assert_eq!(
            frame,
            json!({
                "type": "timeline.inline",
                "item": {
                    "id": "inline_work_1_delegated",
                    "taskId": "work_1",
                    "turnId": "voice-1",
                    "content": "spawned",
                    "kind": "spawn",
                },
            }),
        );
        assert!(
            outbox.try_recv().is_err(),
            "exactly one frame for the delegated announcement"
        );
    }

    #[tokio::test]
    async fn a_delegated_work_with_no_inline_content_sends_nothing() {
        let (outbound, mut outbox) = mpsc::channel::<ServerFrame>(8);
        let mut session = test_session(outbound);

        let mut record = via_work::testing::blank_record("work_1", "user_1");
        record.delegation = Some(
            DelegationRef::new("d1", "session-1")
                .with_presentation(json!({ "speech": "starting" })),
        );
        let event = WorkEvent::new(WorkEventKind::Delegated, record.to_public(0));

        session.on_work_event(&event).await;

        assert!(outbox.try_recv().is_err(), "no inline block, no frame");
    }

    #[test]
    fn control_work_never_reaches_a_client() {
        assert!(!forwards_to_client(&event(
            WorkEventKind::Completed,
            WorkKind::Control
        )));
        assert!(forwards_to_client(&event(
            WorkEventKind::Completed,
            WorkKind::Work
        )));
    }

    #[test]
    fn the_three_manager_only_events_are_not_on_the_socket() {
        for kind in [
            WorkEventKind::Accepted,
            WorkEventKind::NotificationPending,
            WorkEventKind::NotificationDelivered,
        ] {
            assert!(
                !forwards_to_client(&event(kind, WorkKind::Work)),
                "{} is manager-only",
                kind.as_str()
            );
        }
    }

    #[test]
    fn progress_check_is_declared_and_intercepted() {
        assert!(
            GatewayTaskEvent::from_wire("task.progress.check").is_some(),
            "it is in the vocabulary",
        );
        assert!(
            !forwards_to_client(&event(WorkEventKind::ProgressCheck, WorkKind::Work)),
            "and it is still not forwarded",
        );
    }

    fn suspended(owner: &str) -> via_voice::SuspensionStatus {
        via_voice::SuspensionStatus {
            suspended: true,
            holders: Vec::new(),
            owner: Some(owner.to_owned()),
            reason: "dictation".to_owned(),
            expires_at: Some(1_700_000_000_000),
        }
    }

    #[test]
    fn a_suspension_clears_playback_before_it_announces_itself() {
        let kinds: Vec<String> = suspension_frames(false, &suspended("host-app"))
            .into_iter()
            .map(|frame| {
                frame.into_value()["type"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned()
            })
            .collect();
        assert_eq!(kinds, ["playback.clear", "input.suspend"]);
    }

    #[test]
    fn a_release_answers_one_resume() {
        let kinds: Vec<String> = suspension_frames(true, &via_voice::SuspensionStatus::default())
            .into_iter()
            .map(|frame| {
                frame.into_value()["type"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned()
            })
            .collect();
        assert_eq!(kinds, ["input.resume"]);
    }

    #[test]
    fn a_status_that_changes_nothing_produces_no_frames() {
        assert!(
            suspension_frames(true, &suspended("host-app")).is_empty(),
            "a re-read after a lagged subscription must not re-announce",
        );
        assert!(
            suspension_frames(true, &suspended("another-host")).is_empty(),
            "a different holder is still a suspension, not a new edge",
        );
        assert!(suspension_frames(false, &via_voice::SuspensionStatus::default()).is_empty(),);
    }

    #[test]
    fn the_payload_cap_is_twenty_mebibytes() {
        assert_eq!(MAX_PAYLOAD_BYTES, 20_971_520);
    }
}
