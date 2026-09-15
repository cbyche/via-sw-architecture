//! The model turn: [`via_realtime::RealtimeSession`] behind
//! [`via_app::VoiceEngine`].
//!
//! `docs/deviations/phase-5-via-app.md` left exactly one thing owed —
//! *"**Binding `via_realtime::RealtimeSession` to it is the remaining phase-5
//! step**"* — and this module is it. It is the second half of upstream's
//! `server/src/voice/realtime-gateway.mjs`: `via-app` owns the socket plumbing,
//! and what a provider event *means* is decided here, by calling `via-voice`.
//!
//! # What one engine owns
//!
//! One engine is one live realtime session, and it owns four things:
//!
//! | | What | Whose rules |
//! | --- | --- | --- |
//! | the session | one socket, correlation, the output queue | [`via_realtime`] |
//! | the tool-call handler | the nine frontend tools | [`via_voice::ToolCallHandler`] |
//! | the announcement manager | batching, retry, the claim | [`via_voice::AnnouncementManager`] |
//! | two pumps | provider events, and the Work plane | here |
//!
//! Everything else is borrowed: the Injection Gate is the connection's
//! ([`via_app::SharedGate`]), the Work queue is the Gateway's, and every
//! sentence is [`via_i18n`]'s.
//!
//! # Why the pumps are tasks and `submit_text` is not awaited
//!
//! `via_app::VoiceEngine::submit_text` is called from inside the connection's
//! single `select!` loop. Awaiting a model turn there would stop that loop for
//! the length of the response — so the socket would stop reading
//! `playback.started`, which is the receipt the Injection Gate is built on, and
//! an announcement would be blocked by the very turn it is waiting behind.
//! Upstream does not await it either: `submitInputMessage(event)` is called
//! without `await` and its promise is caught. The turn is therefore spawned,
//! and the ordering that matters is kept where it already is — in the session's
//! own serial output queue (`docs/architecture.md` §11, invariant 2).

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::Mutex;
use tokio_util::task::TaskTracker;
use via_app::realtime::frames::ServerFrame;
use via_app::{EngineContext, EngineFactory, SharedGate, VoiceEngine};
use via_i18n::{Locale, keys, t};
use via_protocol::{GatewayServerEvent, WorkStatus};
use via_realtime::{SessionEvent, SessionEvents, SessionOptions};
use via_voice::tools::{
    BackendAvailability, CommittedTurn, DelegationRunners, PermissionResponder,
};
use via_voice::{
    Announcement, AnnouncementManager, AnnouncementManagerConfig, CorrelatedContext, ModePlan,
    NotificationClaims, ProviderView, ResponseContext, ResponseContexts, ResponseOrigin,
    ResponseRequestContext, ServerEvent, ToolCall, ToolCallHandler, ToolCallHandlerConfig,
    VoiceFrontend, ensure_response_context, merge_response_context,
    response_activity_context_patch,
};
use via_work::{NotificationClaim, PublicWork, WorkEventKind, WorkManager};

use crate::gateway::frontend::{ProviderSnapshot, SessionFrontend};
use crate::gateway::opener::SessionOpener;

/// Everything an engine needs that is not per-connection.
///
/// Built once at composition and cloned into every connection, exactly as
/// upstream's `attachRealtimeGateway({...})` closes over one set of services
/// for every socket it accepts.
#[derive(Clone)]
pub struct EngineServices {
    /// Every Gateway service, by injection.
    pub services: via_app::Services,
    /// Where a realtime session comes from.
    pub opener: Arc<dyn SessionOpener>,
    /// The runners a `spawn_thinking` submits with.
    pub runners: Arc<dyn DelegationRunners>,
    /// Whether the backend can take work.
    pub availability: Arc<dyn BackendAvailability>,
    /// The permission relay, when a harness is configured.
    pub permissions: Option<Arc<dyn PermissionResponder>>,
    /// Where the engines' tasks are registered, so shutdown can wait on them.
    pub tracker: TaskTracker,
}

impl std::fmt::Debug for EngineServices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineServices")
            .field("opener", &self.opener)
            .field("availability", &self.availability)
            .field("has_permission_relay", &self.permissions.is_some())
            .finish_non_exhaustive()
    }
}

/// Opens a real realtime session per connection.
#[derive(Clone, Debug)]
pub struct RealtimeEngineFactory {
    services: EngineServices,
}

impl RealtimeEngineFactory {
    /// A factory over `services`.
    #[must_use]
    pub const fn new(services: EngineServices) -> Self {
        Self { services }
    }
}

#[async_trait]
impl EngineFactory for RealtimeEngineFactory {
    /// Never reached: [`open_for`](EngineFactory::open_for) is overridden, and
    /// an engine with nowhere to write its frames would be a session whose
    /// answers nobody hears.
    async fn open(&self, _provider: &str) -> Option<Arc<dyn VoiceEngine>> {
        None
    }

    async fn open_for(&self, context: EngineContext) -> Option<Arc<dyn VoiceEngine>> {
        // The socket is kept before the engine is attempted, because a refusal
        // is something the *client* has to be told: upstream answers a failed
        // `ensureFrontend()` with `send(ws, {type: 'error', message})`
        // (`realtime-gateway.mjs:508`). Without it a user who typed a message
        // into `via chat` sees `turn.started` and then silence, which is the
        // one failure shape a text client cannot diagnose.
        let outbound = context.outbound.clone();
        let provider = context.provider.clone();
        match RealtimeEngine::open(self.services.clone(), context).await {
            Ok(engine) => Some(engine),
            Err(refusal) => {
                let _ = outbound.try_send(via_app::realtime::frames::error(&refusal));
                // `connectFrontendNow`'s connect-failure half —
                // `realtime-gateway.mjs:1582-1588`.
                let _ = outbound.try_send(via_app::realtime::frames::voice_connection(
                    &via_realtime::realtime_connection_status(
                        &via_realtime::RealtimeConnectionInputs {
                            provider,
                            blocked_error: refusal.clone(),
                            ..via_realtime::RealtimeConnectionInputs::default()
                        },
                    ),
                ));
                self.services.services.logger.error(
                    "realtime.open_failed",
                    via_log::fields([("error", refusal.into())]),
                    "",
                );
                None
            }
        }
    }
}

/// The turn the gateway has committed to, and its generation.
///
/// **External contract** — `realtime-gateway.mjs`'s `commitTurn` /
/// `currentTurn()`. The generation is what makes a tool call from a superseded
/// turn detectable ([`ToolCallHandler::is_stale`]) rather than merely late.
///
/// [`via_voice::TurnTracker`] is upstream's `turnId`/`turnGeneration` **and**
/// `committedTurnId`/`committedTurnGeneration` in one: a text turn commits the
/// instant it is minted ([`RealtimeEngine::submit_text`]), and a voice turn
/// commits when its transcript settles (`on_provider`'s
/// `conversation.item.input_audio_transcription.completed` arm) — both paths
/// share the one tracker so [`RealtimeEngine::fallback_turn`] and
/// [`RealtimeEngine::committed`] never disagree about which turn is current.
type SharedTurn = Arc<Mutex<via_voice::TurnTracker>>;

/// How many playback receipts may be in flight to the pump task at once.
///
/// One per response the client is currently rendering; a session has few of
/// those, so this is generous headroom rather than a tuned bound.
const PLAYBACK_CONTROL_QUEUE_DEPTH: usize = 32;

/// A client's `playback.started` / `playback.ended` / `playback.cancelled`
/// receipt, carried into the pump task that owns the response contexts they
/// act on.
///
/// **External contract** — `realtime-gateway.mjs`'s `startPlayback` /
/// `finishPlayback` / `cancelQueuedPlayback`, all three closures over the same
/// `responseContexts` map [`RealtimeEngine::pump_provider`] holds locally; a
/// connection cannot reach that state directly; see the [`VoiceEngine`]
/// implementation below.
#[derive(Debug, Clone)]
enum PlaybackControl {
    /// `startPlayback(id)`.
    Started(String),
    /// `finishPlayback(id)`.
    Ended(String),
    /// `cancelQueuedPlayback(id, {reason})`.
    Cancelled {
        /// The response whose audio was dropped.
        response_id: String,
        /// `event.reason`, e.g. `user_interruption`.
        reason: String,
    },
}

/// The pump task's own local variables, upstream's closure scope made
/// explicit: the response contexts, the item-id-to-turn correlation
/// (`inputTurns`), and whether the user currently has the floor
/// (`userSpeaking`).
struct PumpState {
    contexts: ResponseContexts,
    correlation: via_voice::TurnCorrelation,
    user_speaking: bool,
}

impl PumpState {
    fn new() -> Self {
        Self {
            contexts: ResponseContexts::new(),
            correlation: via_voice::TurnCorrelation::default(),
            user_speaking: false,
        }
    }
}

/// One live realtime session, as the socket consumes it.
pub struct RealtimeEngine {
    frontend: Arc<SessionFrontend>,
    snapshot: ProviderSnapshot,
    tools: Arc<ToolCallHandler>,
    announcements: AnnouncementManager,
    turn: SharedTurn,
    /// Recorded so a delegated tool call has the words the user just said,
    /// even when they arrived as speech rather than text.
    transcripts: via_voice::TurnTranscripts,
    tracker: TaskTracker,
    outbound: tokio::sync::mpsc::Sender<ServerFrame>,
    /// Where a client's playback receipt reaches the pump — see
    /// [`PlaybackControl`].
    playback: tokio::sync::mpsc::Sender<PlaybackControl>,
    locale: Locale,
}

impl std::fmt::Debug for RealtimeEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealtimeEngine")
            .field("provider", &self.snapshot.key())
            .finish_non_exhaustive()
    }
}

impl RealtimeEngine {
    /// Open a session and start its two pumps.
    ///
    /// # Errors
    ///
    /// The already-localized refusal the provider or the transport gave. It is
    /// logged and the connection continues with no engine — a Gateway whose
    /// realtime credential is wrong still serves `/api/health`, still relays
    /// input suspensions and still forwards the Work plane
    /// (`via_app::realtime::engine`).
    pub async fn open(
        services: EngineServices,
        context: EngineContext,
    ) -> Result<Arc<Self>, String> {
        let EngineServices {
            services: gateway,
            opener,
            runners,
            availability,
            permissions,
            tracker,
        } = services;
        let locale = gateway.locale;

        let provider = gateway
            .realtime_registry
            .resolve(Some(&context.provider))
            .map_err(|refusal| refusal.to_string())?;

        let client = via_voice::ToolClientContext {
            states: context.client_states.clone(),
            ..via_voice::ToolClientContext::default()
        };
        // The prompt layer's client context and the tool layer's are different
        // types on purpose: one is what the model is *told*, the other what the
        // handler *reads*. `timeZone` / `locale` / `workingDirectory` are not
        // threaded through yet — only `states` is, which is what
        // `frontend_tools` and `enter_sleep`'s own `declares` check need.
        let prompt_client = via_conversation::normalize_client_context(
            &via_conversation::RawClientContext::default(),
        );
        let mode = ModePlan::new(
            via_protocol::SessionMode::default(),
            gateway.backend.enabled(),
        );
        let options = SessionOptions {
            locale,
            agent_context: via_realtime::AgentContext {
                instructions: via_voice::build_packaged_frontend_instructions(
                    locale,
                    &prompt_client,
                    &[],
                ),
                tools: via_voice::frontend_tools(locale, &client.states, mode),
                ..via_realtime::AgentContext::default()
            },
            ..SessionOptions::default()
        };

        let (session, events) = opener
            .open(&provider, options)
            .await
            .map_err(|error| error.to_string())?;
        let frontend = Arc::new(SessionFrontend::new(session));
        let snapshot = frontend.snapshot().clone();

        let claimant_id = via_voice::new_claimant_id();
        let transcripts = via_voice::TurnTranscripts::default();
        let client_state_outbound = context.outbound.clone();
        let tools = Arc::new(ToolCallHandler::new(ToolCallHandlerConfig {
            locale,
            owner_id: context.owner_id.clone(),
            session_id: context.session_id.clone(),
            mode,
            work: Arc::clone(&gateway.work),
            transcripts: transcripts.clone(),
            assets: (*gateway.input_assets).clone(),
            memory: via_conversation::MemoryTool::new(Some((*gateway.memory).clone()), locale),
            notes: via_conversation::NotesTool::new(Some((*gateway.notes).clone()), locale),
            availability,
            runners,
            permission_responder: permissions,
            policy: Arc::clone(&gateway.permission_policy),
            // `tool-call-handler.mjs:65,664` — `enter_sleep`'s
            // `requestClientState('sleeping')`. Entering sleep itself is the
            // wake-word lifecycle's, owned by a later stage; this is only the
            // echo.
            request_client_state: Some(Arc::new(move |state: &str| {
                let _ =
                    client_state_outbound.try_send(via_app::realtime::frames::client_state(state));
            })),
        }));
        // The handler starts every connect with `ClientContext::default()`
        // (`tool-call-handler.mjs`'s own initial state) and is told the real
        // one exactly once here, from the same snapshot `client` above —
        // `via_voice::ToolClientContext` and the config field's type are the
        // same type under two names; see `ToolCallHandlerConfig::
        // request_client_state`'s note on what is and is not threaded
        // through yet.
        tools.set_client_context(client.clone());

        // The announcement manager's blocking predicate *is* the connection's
        // Injection Gate — `docs/architecture.md` §11, invariant 3. Sharing it
        // rather than deriving a second one is what makes a `playback.started`
        // the socket received actually hold a result back.
        let gate: SharedGate = Arc::clone(&context.gate);
        let blocked_gate = Arc::clone(&gate);
        let announcement_frontend = Arc::clone(&frontend);
        let announcements = AnnouncementManager::with_tracker(
            AnnouncementManagerConfig {
                locale,
                announce_into_context: gateway.config.announce_into_context,
                result_context_max_chars: usize::try_from(gateway.config.result_context_max_chars)
                    .unwrap_or(via_voice::announcement::DEFAULT_RESULT_CONTEXT_MAX_CHARS),
                batch_window_ms: u64::try_from(gateway.config.announcement_batch_ms).unwrap_or(0),
                ..AnnouncementManagerConfig::default()
            },
            Arc::new(move || Some(Arc::clone(&announcement_frontend) as Arc<dyn VoiceFrontend>)),
            Arc::new(move || blocked_gate.lock().map_or(true, |gate| gate.is_blocked())),
            Arc::new(WorkClaims {
                work: Arc::clone(&gateway.work),
                claimant_id: claimant_id.clone(),
            }),
            Some(tracker.clone()),
        );

        let (playback_tx, playback_rx) = tokio::sync::mpsc::channel(PLAYBACK_CONTROL_QUEUE_DEPTH);
        let engine = Arc::new(Self {
            frontend,
            snapshot,
            tools,
            announcements,
            turn: Arc::new(Mutex::new(via_voice::TurnTracker::new())),
            transcripts,
            tracker: tracker.clone(),
            outbound: context.outbound.clone(),
            playback: playback_tx,
            locale,
        });

        let pump = Arc::clone(&engine);
        tracker.spawn(async move { pump.pump_provider(events, playback_rx, gate).await });

        let plane = Arc::clone(&engine);
        let work = Arc::clone(&gateway.work);
        let owner_id = context.owner_id.clone();
        let session_id = context.session_id.clone();
        tracker.spawn(async move {
            plane
                .pump_work(work, owner_id, session_id, claimant_id)
                .await;
        });

        Ok(engine)
    }

    fn send(&self, frame: ServerFrame) {
        let _ = self.outbound.try_send(frame);
    }

    async fn committed(&self) -> CommittedTurn {
        let turn = self.turn.lock().await;
        let committed = turn.committed();
        CommittedTurn {
            turn_id: committed.turn_id.clone(),
            turn_generation: committed.turn_generation,
        }
    }

    /// `fallbackResponseContext()` — `realtime-gateway.mjs:592-599`: the
    /// committed turn when there is one, otherwise the turn currently being
    /// captured. A response that starts mid-speech, before its transcript has
    /// settled, still needs *a* turn to correlate against.
    async fn fallback_turn(&self) -> via_voice::TurnRef {
        self.turn.lock().await.fallback_turn()
    }

    /// Everything the provider says, translated into frames and tool calls.
    ///
    /// Ported from the event switch of `realtime-gateway.mjs:1042-1300`,
    /// including the input-side transcription branches
    /// `docs/deviations/phase-5-apps-via.md` left owed to whichever provider
    /// first emitted one.
    ///
    /// The two sources are fair-selected rather than `biased`: a client's
    /// playback receipt for response *A* and the provider's next event for
    /// response *B* have no ordering upstream promises between them, unlike
    /// the socket's own outbound-before-inbound bias
    /// ([`via_app::realtime::connection`]).
    async fn pump_provider(
        self: Arc<Self>,
        mut events: SessionEvents,
        mut playback: tokio::sync::mpsc::Receiver<PlaybackControl>,
        gate: SharedGate,
    ) {
        let mut state = PumpState::new();
        loop {
            tokio::select! {
                event = events.recv() => {
                    let Some(event) = event else { break };
                    match event {
                        SessionEvent::Provider(provider) => {
                            self.on_provider(&mut state, &gate, &provider).await;
                        }
                        // A provider failure is the one thing a user can act on, so
                        // it is the one thing that becomes an `error` frame.
                        SessionEvent::Error(error) => {
                            self.send(via_app::realtime::frames::error(&error.to_string()));
                        }
                        SessionEvent::Diagnostic(_) => {}
                        SessionEvent::Closed => break,
                    }
                }
                control = playback.recv() => {
                    let Some(control) = control else { continue };
                    self.on_playback(&mut state, control).await;
                }
            }
        }
    }

    async fn on_provider(
        &self,
        state: &mut PumpState,
        gate: &SharedGate,
        provider: &via_realtime::ProviderEvent,
    ) {
        let event = server_event(provider);
        let id = event.realtime_response_id().to_owned();
        let fallback_turn = self.fallback_turn().await;
        let fallback = ResponseContext {
            turn_id: fallback_turn.turn_id,
            origin: crate::gateway::frontend::into_voice_origin(provider.origin),
            turn_generation: fallback_turn.turn_generation,
            ..ResponseContext::default()
        };
        let contexts = &mut state.contexts;

        // Upstream's lifecycle setup runs *before* the switch, so a provider
        // that emits output before (or instead of) `response.created` still
        // gets a context (`realtime-gateway.mjs:1112-1114`).
        //
        // **Merged only when the merge has something to say.** Upstream's
        // `mergeResponseContext` is a JavaScript spread —
        // `{...existing, ...authoritative, …}` — so a patch with no keys
        // changes nothing. `via_voice::merge_response_context` assigns instead,
        // preserving a named list of progress flags and dropping the rest,
        // which its own `a_later_delta_without_metadata_keeps_the_correlated
        // _context` asserts. So the *caller* is where "an uncorrelated delta
        // must not erase the turn it belongs to" lives: merge on the first
        // sighting, which takes the whole fallback, and afterwards only when
        // the provider actually echoed a correlation.
        if event.is_response_activity() {
            let first_sighting = !contexts.contains_key(&id);
            if first_sighting || event.voice_context.is_some() {
                let patch = response_activity_context_patch(contexts.get(&id), &event, &fallback);
                merge_response_context(contexts, &id, patch);
            }
        }

        match event.event_type.as_str() {
            "response.created" => {
                let context = ensure_response_context(contexts, &id, &fallback);
                if !context.response_started {
                    context.response_started = true;
                    let frame = ServerFrame::new(GatewayServerEvent::ResponseStarted.as_str())
                        .with("responseId", id.clone());
                    self.send(via_app::realtime::frames::with_public_response_context(
                        frame, context,
                    ));
                }
            }
            "response.function_call_arguments.done" => {
                let context = ensure_response_context(contexts, &id, &fallback);
                context.has_function_call = true;
                let call = ToolCall {
                    call_id: string_field(&provider.event, "call_id"),
                    name: string_field(&provider.event, "name"),
                    arguments: string_field(&provider.event, "arguments"),
                    response_id: id.clone(),
                    turn_id: Some(context.turn_id.clone()),
                    turn_generation: Some(context.turn_generation),
                };
                self.dispatch_tool(call).await;
            }
            "response.audio.delta" | "response.output_audio.delta" => {
                let context = ensure_response_context(contexts, &id, &fallback);
                if context.suppressed {
                    return;
                }
                context.has_audio = true;
                let turn_id = context.turn_id.clone();
                let origin = context.origin;
                if !id.is_empty() {
                    let _ = gate.lock().map(|mut gate| {
                        gate.window_mut().queue_audio(&id, &turn_id, origin);
                    });
                }
                self.send(
                    ServerFrame::new(GatewayServerEvent::AudioDelta.as_str())
                        .with("audio", string_field(&provider.event, "delta"))
                        .with(
                            "sampleRate",
                            number_field(&provider.event, "sampleRate")
                                .unwrap_or_else(|| i64::from(self.snapshot.output_sample_rate())),
                        )
                        .with("responseId", id.clone())
                        .with("turnId", turn_id),
                );
            }
            "response.audio_transcript.delta" | "response.output_audio_transcript.delta" => {
                self.assistant_transcript(contexts, &id, &fallback, &event.delta, false);
            }
            "response.audio_transcript.done" | "response.output_audio_transcript.done" => {
                let content = event.transcript.clone();
                {
                    let context = ensure_response_context(contexts, &id, &fallback);
                    context.transcript_done = true;
                    context
                        .assistant_transcript
                        .clone_from(&content.clone().unwrap_or_default());
                }
                self.assistant_transcript(contexts, &id, &fallback, &content, true);
            }
            "response.text.delta" | "response.output_text.delta" => {
                self.assistant_transcript(contexts, &id, &fallback, &event.delta, false);
            }
            "response.text.done" | "response.output_text.done" => {
                let content = string_optional(&provider.event, "text");
                {
                    let context = ensure_response_context(contexts, &id, &fallback);
                    context.transcript_done = true;
                    context
                        .assistant_transcript
                        .clone_from(&content.clone().unwrap_or_default());
                }
                self.assistant_transcript(contexts, &id, &fallback, &content, true);
            }
            "response.done" => {
                self.on_response_done(contexts, gate, &id, &event, &fallback)
                    .await;
            }
            "input_audio_buffer.speech_started" => {
                self.on_speech_started(state, gate, &event).await;
            }
            "input_audio_buffer.speech_stopped" => {
                self.on_speech_stopped(state, gate, &event).await;
            }
            "input_audio_buffer.committed" => {
                self.on_input_committed(state, gate, &event).await;
            }
            _ if via_voice::response::STREAMING_INPUT_TRANSCRIPT_EVENTS
                .contains(&event.event_type.as_str()) =>
            {
                self.on_input_transcript_streaming(state, &event).await;
            }
            "conversation.item.input_audio_transcription.completed" => {
                self.on_input_transcript_completed(state, &event).await;
            }
            "conversation.item.input_audio_transcription.failed" => {
                self.on_input_transcript_failed(state, &event).await;
            }
            _ => {}
        }
    }

    /// `input_audio_buffer.speech_started` — `realtime-gateway.mjs:970-1011`.
    ///
    /// Mints a fresh voice turn unless the provider already correlated this
    /// utterance to a known one via `event.item_id`
    /// ([`via_voice::TurnCorrelation::lookup`]). Two things upstream does here
    /// are **not** reproduced, both out of this stage's scope: attaching
    /// staged `input.parts` to the turn (`frontend?.appendUserInputContext`)
    /// and the response-start watchdog (`clearResponseCandidate`).
    async fn on_speech_started(
        &self,
        state: &mut PumpState,
        gate: &SharedGate,
        event: &ServerEvent,
    ) {
        state.user_speaking = true;
        let item_id = event.item_id.clone().unwrap_or_default();
        let known = if item_id.is_empty() {
            None
        } else {
            state.correlation.lookup(&item_id).cloned()
        };
        let current = {
            let mut turn = self.turn.lock().await;
            match known {
                Some(known) => {
                    turn.adopt(known.clone());
                    known
                }
                None => {
                    let minted = turn.begin_voice_turn(now_ms());
                    state.correlation.remember(&item_id, minted.clone());
                    minted
                }
            }
        };
        let _ = gate
            .lock()
            .map(|mut gate| gate.window_mut().begin_turn(&current.turn_id));
        self.announcements.dismiss_active().await;
        self.send(via_app::realtime::frames::playback_clear(Some(
            "user_interruption",
        )));
        self.send(
            ServerFrame::new(GatewayServerEvent::TurnStarted.as_str())
                .with("turnId", current.turn_id.clone()),
        );
        // Unlike every other `voice.state`, `listening` carries no `origin` —
        // `realtime-gateway.mjs:1011` sends exactly `{type, state, turnId}`.
        self.send(
            ServerFrame::new(GatewayServerEvent::VoiceState.as_str())
                .with("state", "listening")
                .with("turnId", current.turn_id),
        );
        self.frontend.cancel().await;
    }

    /// `input_audio_buffer.speech_stopped` — `realtime-gateway.mjs:1017-1035`.
    async fn on_speech_stopped(
        &self,
        state: &mut PumpState,
        gate: &SharedGate,
        event: &ServerEvent,
    ) {
        let item_id = event.item_id.clone().unwrap_or_default();
        let current = self.turn.lock().await.current().clone();
        let stopped_turn = state.correlation.resolve(&item_id, current);
        state.user_speaking = false;
        let _ = gate.lock().map(|mut gate| gate.window_mut().end_speech());
        if event.reason.as_deref() == Some("turn_invalid") {
            state.correlation.invalidate(&item_id);
            self.send(
                ServerFrame::new(GatewayServerEvent::TranscriptDiscard.as_str())
                    .with("role", "user")
                    .with("turnId", stopped_turn.turn_id.clone())
                    .with("reason", "turn_invalid"),
            );
            self.send(
                ServerFrame::new(GatewayServerEvent::VoiceState.as_str())
                    .with("state", "idle")
                    .with("turnId", stopped_turn.turn_id)
                    .with("origin", "model"),
            );
        } else {
            // `expectResponseFor` (the response-start watchdog) is not
            // reproduced — see the module documentation.
            self.send(
                ServerFrame::new(GatewayServerEvent::VoiceState.as_str())
                    .with("state", "processing")
                    .with("turnId", stopped_turn.turn_id)
                    .with("origin", "model"),
            );
        }
    }

    /// `input_audio_buffer.committed` — `realtime-gateway.mjs:1039-1050`.
    async fn on_input_committed(
        &self,
        state: &mut PumpState,
        gate: &SharedGate,
        event: &ServerEvent,
    ) {
        let item_id = event.item_id.clone().unwrap_or_default();
        let current = self.turn.lock().await.current().clone();
        let committed_turn = state.correlation.resolve(&item_id, current);
        state.user_speaking = false;
        let _ = gate.lock().map(|mut gate| gate.window_mut().end_speech());
        if !state.correlation.is_invalid(&item_id) {
            self.send(
                ServerFrame::new(GatewayServerEvent::VoiceState.as_str())
                    .with("state", "processing")
                    .with("turnId", committed_turn.turn_id)
                    .with("origin", "model"),
            );
        }
    }

    /// `conversation.item.input_audio_transcription.{delta,text}` —
    /// `realtime-gateway.mjs:1054-1070`. Streaming ASR: `replace: true` means
    /// the content is the running transcript, not an increment.
    async fn on_input_transcript_streaming(&self, state: &mut PumpState, event: &ServerEvent) {
        let item_id = event.item_id.clone().unwrap_or_default();
        if state.correlation.is_invalid(&item_id) {
            return;
        }
        let current = self.turn.lock().await.current().clone();
        let turn = state.correlation.resolve(&item_id, current);
        let transcript = event.streaming_input_transcript();
        if turn.turn_id.is_empty() || transcript.is_empty() {
            return;
        }
        self.send(
            ServerFrame::new(GatewayServerEvent::TranscriptDelta.as_str())
                .with("role", "user")
                .with("content", transcript)
                .with("turnId", turn.turn_id)
                .with("replace", true),
        );
    }

    /// `conversation.item.input_audio_transcription.completed` —
    /// `realtime-gateway.mjs:1073-1106`. An empty transcript is discarded
    /// rather than spoken as silence; a non-empty one commits the turn, which
    /// is what makes it the fallback for a response with no correlation of
    /// its own.
    async fn on_input_transcript_completed(&self, state: &mut PumpState, event: &ServerEvent) {
        let item_id = event.item_id.clone().unwrap_or_default();
        let current = self.turn.lock().await.current().clone();
        let completed = state.correlation.complete(&item_id, current);
        if completed.invalid {
            return;
        }
        let transcript = event.transcript.clone().unwrap_or_default();
        let transcript = transcript.trim();
        if transcript.is_empty() {
            self.send(
                ServerFrame::new(GatewayServerEvent::TranscriptDiscard.as_str())
                    .with("role", "user")
                    .with("turnId", completed.context.turn_id),
            );
            return;
        }
        {
            let mut turn = self.turn.lock().await;
            turn.commit(&completed.context);
        }
        self.transcripts
            .record(&completed.context.turn_id, transcript)
            .await;
        self.send(
            ServerFrame::new(GatewayServerEvent::TranscriptFinal.as_str())
                .with("role", "user")
                .with("content", transcript)
                .with("turnId", completed.context.turn_id),
        );
    }

    /// `conversation.item.input_audio_transcription.failed` —
    /// `realtime-gateway.mjs:1109-1113`.
    async fn on_input_transcript_failed(&self, state: &mut PumpState, event: &ServerEvent) {
        let item_id = event.item_id.clone().unwrap_or_default();
        let current = self.turn.lock().await.current().clone();
        let completed = state.correlation.complete(&item_id, current);
        self.send(
            ServerFrame::new(GatewayServerEvent::TranscriptDiscard.as_str())
                .with("role", "user")
                .with("turnId", completed.context.turn_id),
        );
    }

    /// Dispatch one playback receipt against the response-context state only
    /// the pump task owns.
    ///
    /// **External contract** — `startPlayback` / `finishPlayback` /
    /// `cancelQueuedPlayback`, `realtime-gateway.mjs:662-711,780-825`. The
    /// window half of each (`announcementWindow.startPlayback` /
    /// `finishPlayback`) already ran in
    /// [`via_app::realtime::connection`], against the shared
    /// [`via_voice::InjectionGate`], before the receipt ever reaches here —
    /// what is left is the half that needs the response contexts.
    async fn on_playback(&self, state: &mut PumpState, control: PlaybackControl) {
        match control {
            PlaybackControl::Started(id) => self.start_playback(state, &id).await,
            PlaybackControl::Ended(id) => self.finish_playback(state, &id).await,
            PlaybackControl::Cancelled {
                response_id,
                reason,
            } => {
                self.cancel_queued_playback(state, &response_id, &reason)
                    .await;
            }
        }
    }

    /// The current (uncommitted) turn's id — upstream's ambient `turnId`,
    /// used as the playback fallback when a response carries none of its own.
    async fn current_turn_id(&self) -> String {
        self.turn.lock().await.current().turn_id.clone()
    }

    /// `startPlayback` — `realtime-gateway.mjs:662-680`.
    ///
    /// Not reproduced: confirming an announcement's notification and flushing
    /// transcript fragments held for this playback
    /// (`confirmsTaskNotificationOnPlaybackStart`, `flushPendingTranscripts`)
    /// — a separate, not-yet-wired deferred-transcript mechanism this stage
    /// does not touch.
    async fn start_playback(&self, state: &mut PumpState, id: &str) {
        if state
            .contexts
            .get(id)
            .is_some_and(|context| context.suppressed)
        {
            return;
        }
        let turn_id = match state
            .contexts
            .get(id)
            .map(|context| context.turn_id.clone())
            .filter(|turn_id| !turn_id.is_empty())
        {
            Some(turn_id) => turn_id,
            None => self.current_turn_id().await,
        };
        let origin = state
            .contexts
            .get(id)
            .map_or(ResponseOrigin::Model, |context| context.origin);
        self.send(
            ServerFrame::new(GatewayServerEvent::VoiceState.as_str())
                .with("state", "speaking")
                .with("turnId", turn_id)
                .with("origin", origin.as_str()),
        );
        if let Some(context) = state.contexts.get_mut(id) {
            context.playback_started = true;
        }
    }

    /// `finishPlayback` — `realtime-gateway.mjs:780-803`.
    async fn finish_playback(&self, state: &mut PumpState, id: &str) {
        if state
            .contexts
            .get(id)
            .is_some_and(|context| context.suppressed)
        {
            return;
        }
        let context_turn_id = state
            .contexts
            .get(id)
            .map(|context| context.turn_id.clone())
            .filter(|turn_id| !turn_id.is_empty());
        let origin = state
            .contexts
            .get(id)
            .map_or(ResponseOrigin::Model, |context| context.origin);

        if let Some(context) = state.contexts.get_mut(id) {
            context.playback_ended = true;
        }
        if state
            .contexts
            .get(id)
            .is_some_and(ResponseContext::is_complete)
        {
            state.contexts.shift_remove(id);
        }

        let current_turn_id = self.current_turn_id().await;
        let (voice_state, turn_id) = if state.user_speaking {
            ("listening", current_turn_id)
        } else {
            ("idle", context_turn_id.unwrap_or(current_turn_id))
        };
        self.send(
            ServerFrame::new(GatewayServerEvent::VoiceState.as_str())
                .with("state", voice_state)
                .with("turnId", turn_id)
                .with("origin", origin.as_str()),
        );
    }

    /// `cancelQueuedPlayback` — `realtime-gateway.mjs:683-716`.
    async fn cancel_queued_playback(&self, state: &mut PumpState, id: &str, reason: &str) {
        let origin = state.contexts.get(id).map(|context| context.origin);
        if origin == Some(ResponseOrigin::Announcement) {
            let task_ids = state
                .contexts
                .get(id)
                .map(ResponseContext::task_ids)
                .unwrap_or_default();
            if reason == "user_interruption" {
                self.announcements.confirm_many(task_ids).await;
            } else {
                self.announcements.retry_many(task_ids).await;
            }
        }

        if reason == "user_interruption"
            && state
                .contexts
                .get(id)
                .is_some_and(|context| context.playback_started)
            && let Some(context) = state.contexts.get(id)
        {
            let frame = ServerFrame::new(GatewayServerEvent::ResponseInterrupted.as_str())
                .with("responseId", id.to_owned());
            self.send(via_app::realtime::frames::with_public_response_context(
                frame, context,
            ));
        }

        let context_turn_id = state
            .contexts
            .get(id)
            .map(|context| context.turn_id.clone())
            .filter(|turn_id| !turn_id.is_empty());

        // The tombstone (`suppressed`) stays in the map rather than being
        // evicted on a timer as upstream does
        // (`scheduleResponseContextCleanup`): late provider audio for this id
        // must keep finding a suppressed context, not a fresh one.
        if let Some(context) = state.contexts.get_mut(id) {
            context.suppressed = true;
            context.playback_ended = true;
            context.pending_transcripts.clear();
        }

        let current_turn_id = self.current_turn_id().await;
        let (voice_state, turn_id) = if state.user_speaking {
            ("listening", current_turn_id)
        } else {
            ("idle", context_turn_id.unwrap_or(current_turn_id))
        };
        self.send(
            ServerFrame::new(GatewayServerEvent::VoiceState.as_str())
                .with("state", voice_state)
                .with("turnId", turn_id)
                .with("origin", origin.unwrap_or(ResponseOrigin::Model).as_str()),
        );
    }

    /// `emitAssistantTranscript` — `realtime-gateway.mjs:1157-1206`.
    ///
    /// A text client receives its answer here rather than as audio, which is
    /// the whole of what `via chat` renders.
    fn assistant_transcript(
        &self,
        contexts: &mut ResponseContexts,
        id: &str,
        fallback: &ResponseContext,
        content: &Option<String>,
        settled: bool,
    ) {
        let context = ensure_response_context(contexts, id, fallback);
        if context.suppressed {
            return;
        }
        let content = content.clone().unwrap_or_default();
        let event = if settled {
            GatewayServerEvent::TranscriptFinal
        } else {
            GatewayServerEvent::TranscriptDelta
        };
        if !settled && content.is_empty() {
            return;
        }
        // `emitAssistantTranscript` — `realtime-gateway.mjs:601-625`: `role`,
        // `content`, `responseId`, then the seven-key `publicResponseContext`
        // spread (which is where `turnId` comes from here, not a standalone
        // field).
        let frame = ServerFrame::new(event.as_str())
            .with("role", "assistant")
            .with("content", content)
            .with("responseId", id.to_owned());
        self.send(via_app::realtime::frames::with_public_response_context(
            frame, context,
        ));
    }

    /// `response.done` — `realtime-gateway.mjs:1233-1300`.
    async fn on_response_done(
        &self,
        contexts: &mut ResponseContexts,
        gate: &SharedGate,
        id: &str,
        event: &ServerEvent,
        fallback: &ResponseContext,
    ) {
        let failed = event.response_failed();
        let (turn_id, has_audio, suppressed, transcript, origin, task_ids) = {
            let context = ensure_response_context(contexts, id, fallback);
            context.response_done = true;
            (
                context.turn_id.clone(),
                context.has_audio,
                context.suppressed,
                context.assistant_transcript.clone(),
                context.origin,
                context.task_ids.clone(),
            )
        };

        // A tool response that produced neither audio nor words is the one the
        // handler must close itself, or the model waits on an output that
        // never comes.
        self.tools
            .finish_tool_response(
                id,
                failed || suppressed || has_audio || !transcript.trim().is_empty(),
                self.frontend.as_ref(),
            )
            .await;

        if !suppressed {
            self.send(
                ServerFrame::new(GatewayServerEvent::AudioDone.as_str())
                    .with("responseId", id.to_owned())
                    .with("turnId", turn_id.clone()),
            );
            if !has_audio {
                self.send(
                    ServerFrame::new(GatewayServerEvent::VoiceState.as_str())
                        .with("state", "idle")
                        .with("turnId", turn_id.clone())
                        .with("origin", origin.as_str()),
                );
            }
        }

        let _ = gate.lock().map(|mut gate| {
            gate.window_mut().response_done(&via_voice::ResponseDone {
                turn_id: turn_id.clone(),
                origin,
                has_audio,
                failed,
                ..via_voice::ResponseDone::default()
            });
        });

        // A **non-voice** client never sends a playback receipt, so
        // `response.done` is the only proof an announcement was delivered.
        // Upstream's `completedNonVoiceAnnouncement`, exactly.
        if origin == ResponseOrigin::Announcement && !failed && !task_ids.is_empty() {
            self.announcements.confirm_many(task_ids).await;
        }
        if failed || has_audio {
            return;
        }
        contexts.shift_remove(id);
    }

    /// Answer one tool call, off the pump.
    ///
    /// Spawned because a `spawn_thinking` writes to the Work manager and then
    /// writes a function output back through the session's own queue; running
    /// that inline would stop the pump for the length of it, and the pump is
    /// what feeds the client its transcript.
    async fn dispatch_tool(&self, call: ToolCall) {
        // A call with no `call_id` can never be answered — there is nothing to
        // write the output against — so it is reported rather than dropped.
        // **External contract** — `voice.error.missing_call_id`.
        if call.call_id.is_empty() {
            self.send(via_app::realtime::frames::error(t(
                self.locale,
                keys::VOICE_ERROR_MISSING_CALL_ID,
            )));
            return;
        }
        let tools = Arc::clone(&self.tools);
        let frontend = Arc::clone(&self.frontend);
        let committed = self.committed().await;
        self.tracker.spawn(async move {
            tools.handle(&call, &committed, frontend.as_ref()).await;
        });
    }

    /// The Work plane: claims, announcements and the spoken progress check.
    ///
    /// Ported from `realtime-gateway.mjs:790-870`. The socket already forwards
    /// every catalogued task event to the client
    /// ([`via_app::realtime::forwards_to_client`]); what happens *here* is the
    /// half a client cannot do — claiming a notification so exactly one
    /// frontend speaks it, and turning `task.progress.check` into speech
    /// instead of into a frame.
    async fn pump_work(
        self: Arc<Self>,
        work: Arc<WorkManager>,
        owner_id: String,
        session_id: String,
        claimant_id: String,
    ) {
        let mut events = work.subscribe();
        // A frontend that connects after a result landed still speaks it: the
        // claim is what a *reconnecting* client uses to pick up work raised
        // while it was away.
        self.claim(&work, &owner_id, &session_id, &claimant_id)
            .await;
        while let Ok(event) = events.recv().await {
            if event.owner_id != owner_id {
                continue;
            }
            match event.kind {
                WorkEventKind::NotificationPending
                | WorkEventKind::Completed
                | WorkEventKind::Failed => {
                    self.claim(&work, &owner_id, &session_id, &claimant_id)
                        .await;
                }
                // A cancelled Work has nothing to say, but an active batch
                // containing it must still be able to finish.
                WorkEventKind::Cancelled => {
                    self.announcements.remove(&event.task.id).await;
                }
                WorkEventKind::ProgressCheck => {
                    if let via_work::WorkEventDetails::ProgressCheck { message, .. } =
                        &event.details
                    {
                        self.speak_progress(&event.task, message).await;
                    }
                }
                _ => {}
            }
        }
    }

    /// Claim what is deliverable and queue it.
    async fn claim(&self, work: &WorkManager, owner_id: &str, session_id: &str, claimant_id: &str) {
        let claimed = work
            .claim_notifications(
                NotificationClaim::new(owner_id, session_id, claimant_id).across_sessions(),
            )
            .await;
        for item in claimed {
            if let Some(announcement) = announcement(&item) {
                self.announcements.queue(announcement).await;
            } else {
                // Nothing to say, so the claim is given straight back rather
                // than held until its lease expires.
                work.release_notification_claims(&[item.id], Some(claimant_id))
                    .await;
            }
        }
    }

    /// `task.progress.check` — spoken, never forwarded.
    ///
    /// **External contract** — `realtime-gateway.mjs:831-859`, and
    /// [`via_app::realtime::forwards_to_client`] is the other half of the same
    /// rule: the event is *declared* in the client vocabulary and *intercepted*
    /// here.
    async fn speak_progress(&self, task: &PublicWork, message: &str) {
        let context = ResponseRequestContext {
            turn_id: task.turn_id.clone(),
            task_id: Some(task.id.clone()),
            ..ResponseRequestContext::default()
        };
        self.frontend
            .speak(message, ResponseOrigin::Agent, context)
            .await;
    }
}

#[async_trait]
impl VoiceEngine for RealtimeEngine {
    async fn append_audio(&self, audio: &str) {
        self.frontend.append_audio(audio).await;
    }

    async fn submit_text(&self, turn_id: &str, text: &str) {
        {
            // `realtime-gateway.mjs:1780-1783`: a typed turn is current and
            // committed in the same breath — there is no ASR step to wait for.
            let mut turn = self.turn.lock().await;
            let current = turn.begin_typed_turn(turn_id);
            turn.commit(&current);
        }
        let committed = self.committed().await;
        let frontend = Arc::clone(&self.frontend);
        let parts = vec![via_voice::InputPart::text(text)];
        let context = ResponseRequestContext {
            turn_id: Some(committed.turn_id),
            turn_generation: Some(committed.turn_generation),
            ..ResponseRequestContext::default()
        };
        // Not awaited — see the module documentation.
        self.tracker.spawn(async move {
            frontend.send_user_input(&parts, context).await;
        });
    }

    async fn cancel(&self) {
        self.frontend.cancel().await;
        // A user who cut an announcement off has heard it; re-speaking a result
        // they deliberately interrupted is worse than dropping it
        // (`AnnouncementManager::dismiss_active`).
        self.announcements.dismiss_active().await;
    }

    async fn close(&self) {
        self.announcements.close().await;
        self.frontend.close().await;
    }

    // `try_send`, not an awaited `send` — the same "a lagging reader never
    // stalls the caller" rule every outbound queue in this tree already
    // follows (`Self::send`, `Session::send` in `via_app::realtime::connection`).
    async fn playback_started(&self, response_id: &str) {
        let _ = self
            .playback
            .try_send(PlaybackControl::Started(response_id.to_owned()));
    }

    async fn playback_ended(&self, response_id: &str) {
        let _ = self
            .playback
            .try_send(PlaybackControl::Ended(response_id.to_owned()));
    }

    async fn playback_cancelled(&self, response_id: &str, reason: &str) {
        let _ = self.playback.try_send(PlaybackControl::Cancelled {
            response_id: response_id.to_owned(),
            reason: reason.to_owned(),
        });
    }

    fn ready(&self) -> bool {
        self.frontend.ready()
    }

    fn provider(&self) -> &str {
        self.snapshot.key()
    }

    fn provider_label(&self) -> &str {
        self.snapshot.label()
    }

    fn input_sample_rate(&self) -> u32 {
        self.snapshot.input_sample_rate()
    }
}

/// The notification claims the announcement manager holds, backed by the Work
/// manager's lease.
#[derive(Debug)]
struct WorkClaims {
    work: Arc<WorkManager>,
    claimant_id: String,
}

#[async_trait]
impl NotificationClaims for WorkClaims {
    async fn delivered(&self, work_ids: &[String]) {
        self.work
            .mark_notifications_delivered(work_ids, Some(&self.claimant_id))
            .await;
    }

    async fn renew(&self, work_ids: &[String]) {
        self.work
            .renew_notification_claims(work_ids, Some(&self.claimant_id))
            .await;
    }

    async fn release(&self, work_ids: &[String]) {
        self.work
            .release_notification_claims(work_ids, Some(&self.claimant_id))
            .await;
    }
}

/// What a claimed Work is announced as, or `None` when there is nothing to say.
///
/// **External contract** — `realtime-gateway.mjs`'s `queueAnnouncement`: only
/// the two terminal outcomes are spoken. A `cancelled` Work is silent, because
/// the user is the one who cancelled it.
#[must_use]
pub fn announcement(task: &PublicWork) -> Option<Announcement> {
    let announcement = match task.status {
        WorkStatus::Completed => Announcement::completed(
            &task.id,
            &task.objective,
            task.result.as_deref().unwrap_or_default(),
        ),
        WorkStatus::Failed => Announcement::failed(
            &task.id,
            &task.objective,
            task.error.as_deref().unwrap_or_default(),
        ),
        _ => return None,
    };
    Some(
        announcement
            .turn(task.turn_id.clone())
            .at(task.completed_at),
    )
}

/// A [`via_realtime::ProviderEvent`] as the voice layer's [`ServerEvent`].
///
/// The two crates split one JavaScript object: `via-realtime` normalizes the
/// dialect and attaches the correlation, and `via-voice` names the fields the
/// gateway branches on. This is the join, and it is deliberately total — every
/// field `ServerEvent` declares is read here, so a provider that starts sending
/// one does not need a second change somewhere else.
#[must_use]
pub fn server_event(provider: &via_realtime::ProviderEvent) -> ServerEvent {
    let event = &provider.event;
    ServerEvent {
        event_type: string_field(event, "type"),
        response_id: string_optional(event, "response_id"),
        response_object_id: event
            .get("response")
            .and_then(|response| response.get("id"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        item_response_id: event
            .get("item")
            .and_then(|item| item.get("response_id"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        item_id: string_optional(event, "item_id"),
        response_status: event
            .get("response")
            .and_then(|response| response.get("status"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        delta: string_optional(event, "delta"),
        text: string_optional(event, "text"),
        stash: string_optional(event, "stash"),
        transcript: string_optional(event, "transcript"),
        reason: string_optional(event, "reason"),
        voice_origin: Some(crate::gateway::frontend::into_voice_origin(provider.origin)),
        voice_context: correlated(&provider.context),
        voice_retried: provider.retried,
    }
}

/// The correlation the session echoed back, as the voice layer's shape.
///
/// `None` when nothing was attached: the distinction between *"the provider
/// echoed nothing"* and *"the provider echoed a default"* is what
/// [`response_activity_context_patch`] branches on.
fn correlated(context: &via_realtime::ResponseContext) -> Option<CorrelatedContext> {
    if context.is_empty() {
        return None;
    }
    let map = context.as_map();
    Some(CorrelatedContext {
        turn_id: map.get("turnId").and_then(Value::as_str).map(str::to_owned),
        task_id: map
            .get("taskId")
            .map(|value| value.as_str().map(str::to_owned)),
        task_ids: map.get("taskIds").and_then(strings),
        turn_ids: map.get("turnIds").and_then(strings),
        authorization_id: map
            .get("authorizationId")
            .map(|value| value.as_str().map(str::to_owned)),
        turn_generation: map.get("turnGeneration").and_then(Value::as_i64),
        delivery_sequence: map.get("deliverySequence").map(Value::as_u64),
        consumes_task_notification: map.get("consumesTaskNotification").and_then(Value::as_bool),
    })
}

fn strings(value: &Value) -> Option<Vec<String>> {
    Some(
        value
            .as_array()?
            .iter()
            .filter_map(|item| item.as_str().map(str::to_owned))
            .collect(),
    )
}

/// `String(event.x || '')` — the coercion every scalar read goes through.
fn string_field(event: &Value, key: &str) -> String {
    string_optional(event, key).unwrap_or_default()
}

fn string_optional(event: &Value, key: &str) -> Option<String> {
    event.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn number_field(event: &Value, key: &str) -> Option<i64> {
    event
        .get(key)
        .and_then(Value::as_i64)
        .filter(|rate| *rate > 0)
}

/// `Date.now()` — milliseconds since the epoch, for
/// [`via_voice::TurnTracker::begin_voice_turn`]'s id.
fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    fn provider_event(event: Value) -> via_realtime::ProviderEvent {
        via_realtime::ProviderEvent {
            event,
            origin: via_realtime::ResponseOrigin::Model,
            context: via_realtime::ResponseContext::new(),
            retried: false,
        }
    }

    #[test]
    fn the_response_id_is_resolved_in_upstreams_order() {
        let event = server_event(&provider_event(json!({
            "type": "response.output_item.done",
            "response": { "id": "resp_outer" },
            "item": { "response_id": "resp_inner" },
        })));
        assert_eq!(event.realtime_response_id(), "resp_outer");

        let event = server_event(&provider_event(json!({
            "type": "response.output_item.done",
            "response_id": "resp_direct",
            "response": { "id": "resp_outer" },
        })));
        assert_eq!(event.realtime_response_id(), "resp_direct");

        let event = server_event(&provider_event(json!({
            "type": "response.output_item.done",
            "item": { "response_id": "resp_inner" },
        })));
        assert_eq!(event.realtime_response_id(), "resp_inner");
    }

    #[test]
    fn a_failing_status_is_read_off_the_nested_response_object() {
        for status in ["failed", "cancelled", "incomplete"] {
            let event = server_event(&provider_event(json!({
                "type": "response.done",
                "response": { "id": "resp_1", "status": status },
            })));
            assert!(event.response_failed(), "{status}");
        }
        let event = server_event(&provider_event(json!({
            "type": "response.done",
            "response": { "id": "resp_1", "status": "completed" },
        })));
        assert!(!event.response_failed());
    }

    #[test]
    fn an_uncorrelated_event_carries_no_context_at_all() {
        let event = server_event(&provider_event(json!({ "type": "response.created" })));
        assert_eq!(event.voice_context, None);
    }

    #[test]
    fn the_correlation_round_trips_through_both_vocabularies() {
        let request = ResponseRequestContext {
            turn_id: Some("voice-1".to_owned()),
            task_id: Some("work_1".to_owned()),
            task_ids: vec!["work_1".to_owned()],
            turn_ids: vec!["voice-1".to_owned()],
            authorization_id: Some("auth_1".to_owned()),
            turn_generation: Some(2),
            delivery_sequence: Some(9),
            consumes_task_notification: true,
        };
        let mut provider = provider_event(json!({ "type": "response.done" }));
        provider.context = crate::gateway::frontend::context(&request);
        let correlated = server_event(&provider)
            .voice_context
            .expect("a non-empty context correlates");
        assert_eq!(correlated.turn_id.as_deref(), Some("voice-1"));
        assert_eq!(correlated.task_id, Some(Some("work_1".to_owned())));
        assert_eq!(correlated.task_ids, Some(vec!["work_1".to_owned()]));
        assert_eq!(correlated.turn_ids, Some(vec!["voice-1".to_owned()]));
        assert_eq!(correlated.authorization_id, Some(Some("auth_1".to_owned())));
        assert_eq!(correlated.turn_generation, Some(2));
        assert_eq!(correlated.delivery_sequence, Some(Some(9)));
        assert_eq!(correlated.consumes_task_notification, Some(true));
    }

    fn work(status: WorkStatus) -> PublicWork {
        let mut task = via_work::testing::blank_record("work_1", "user_1").to_public(0);
        task.status = status;
        task.objective = "summarise the diff".to_owned();
        task.result = Some("Done.".to_owned());
        task.error = Some("It broke.".to_owned());
        task
    }

    #[test]
    fn only_the_two_terminal_outcomes_are_announced() {
        let completed = announcement(&work(WorkStatus::Completed)).expect("completed speaks");
        assert_eq!(completed.result, "Done.");
        assert_eq!(completed.error, "");

        let failed = announcement(&work(WorkStatus::Failed)).expect("failed speaks");
        assert_eq!(failed.error, "It broke.");
        assert_eq!(failed.result, "");

        for silent in [
            WorkStatus::Cancelled,
            WorkStatus::Queued,
            WorkStatus::Running,
            WorkStatus::Delegated,
            WorkStatus::Finalizing,
            WorkStatus::Cancelling,
            WorkStatus::Scheduled,
        ] {
            assert!(
                announcement(&work(silent)).is_none(),
                "{silent:?} has nothing to say",
            );
        }
    }

    #[test]
    fn a_zero_sample_rate_falls_back_rather_than_being_carried() {
        assert_eq!(
            number_field(&json!({ "sampleRate": 0 }), "sampleRate"),
            None
        );
        assert_eq!(
            number_field(&json!({ "sampleRate": 24000 }), "sampleRate"),
            Some(24_000)
        );
        assert_eq!(number_field(&json!({}), "sampleRate"), None);
    }

    #[test]
    fn a_missing_scalar_reads_as_the_empty_string() {
        assert_eq!(string_field(&json!({}), "delta"), "");
        assert_eq!(string_field(&json!({ "delta": 7 }), "delta"), "");
        assert_eq!(string_field(&json!({ "delta": "x" }), "delta"), "x");
    }
}
