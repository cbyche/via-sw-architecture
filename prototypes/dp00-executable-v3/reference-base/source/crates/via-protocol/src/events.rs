//! The complete Gateway WebSocket wire vocabulary.
//!
//! Ported from `shared/realtime-events.mjs`, which upstream publishes as the
//! package entry point `qwen-audio-agent/realtime-events`. Every string here is
//! an external contract: a client that spells one by hand is on its own, and
//! renaming or reordering breaks every client.
//!
//! Three vocabularies, 52 names, no overlap:
//!
//! | Type | Direction | Names |
//! | --- | --- | --- |
//! | [`GatewayClientEvent`] | client → server | 16 |
//! | [`GatewayServerEvent`] | server → client | 22 |
//! | [`GatewayTaskEvent`] | server → client (task plane) | 14 |
//!
//! Upstream gates direction with two `Set`s — `GATEWAY_CLIENT_EVENT_TYPES` and
//! `GATEWAY_SERVER_EVENT_TYPES` (the union of the last two vocabularies). In
//! Rust the direction is a *type*: a `GatewayServerEvent` cannot be sent by a
//! client, because the client-frame decoder only ever produces
//! `GatewayClientEvent`. The runtime gate is still available — every vocabulary
//! has `as_str` and `from_wire`, and [`GatewayOutboundEvent`] reproduces the
//! union set upstream actually validates against.
//!
//! None of these strings carries the upstream brand, so all 52 survive the
//! rebrand byte for byte (`docs/rebrand.md`, KEEP: "All GatewayClientEvent /
//! GatewayServerEvent / GatewayTaskEvent names").

use crate::macros::wire_enum;

wire_enum! {
    /// The client → server vocabulary — all 16 `type` strings a WebSocket
    /// client may send.
    ///
    /// External contract, from `shared/realtime-events.mjs:1-21`. Upstream's
    /// `GATEWAY_CLIENT_EVENT_TYPES` is the inbound validity gate; an unknown
    /// string is rejected, which here is `from_wire` returning `None`.
    pub enum GatewayClientEvent {
        /// `connect` — opens the session and negotiates it: client type and
        /// label, provider selection, `textOnly` / `voiceEnabled` /
        /// `inputEnabled` / `outputEnabled`, takeover, wake-word-only, time
        /// zone, locale, working directory and input capabilities.
        ///
        /// In VIA this frame also carries the [`SessionMode`](crate::SessionMode).
        Connect = "connect",
        /// `unmute` — claim the voice session for this client (optionally with
        /// `takeover: true`) and enable output.
        Unmute = "unmute",
        /// `mute` — release the voice client, leave sleep, and close the
        /// realtime connection.
        Mute = "mute",
        /// `input.unmute` — enable capture on the already-active voice client
        /// without re-establishing the session.
        InputUnmute = "input.unmute",
        /// `input.mute` — client-declared microphone mute; drops buffered
        /// audio. Weaker than the server's
        /// [`InputSuspend`](GatewayServerEvent::InputSuspend), which also stops
        /// wake-word detection.
        InputMute = "input.mute",
        /// `audio.append` — a chunk of captured PCM. Ignored while sleeping
        /// (except to feed the wake detector), while input is muted, and — as
        /// defence in depth — while an input suspension is in force.
        AudioAppend = "audio.append",
        /// `text.message` — a typed user turn.
        TextMessage = "text.message",
        /// `input.message` — a typed user turn carrying previously staged
        /// input parts. Handled identically to
        /// [`TextMessage`](Self::TextMessage).
        InputMessage = "input.message",
        /// `input.parts` — stages attachments (images, resources) for the next
        /// message; a malformed part is answered with
        /// [`Error`](GatewayServerEvent::Error).
        InputParts = "input.parts",
        /// `interrupt` — barge-in. Bumps the turn generation, interrupts the
        /// announcement window, dismisses the active announcement and cancels
        /// the model's response.
        Interrupt = "interrupt",
        /// `sleep` — explicitly enter wake-word-only listening.
        Sleep = "sleep",
        /// `wake` — leave sleep and restore the foreground connection, reusing
        /// the same reconnect and backoff path as wake-word detection.
        Wake = "wake",
        /// `playback.started` — the host began playing a response's audio.
        /// Carries `responseId`.
        ///
        /// One of the three receipts the Injection Gate needs and that ARGO's
        /// realtime stack has no way to report (`docs/architecture.md` §3).
        PlaybackStarted = "playback.started",
        /// `playback.ended` — the host finished playing a response's audio.
        /// Carries `responseId`. A delivery is marked delivered only after
        /// this arrives.
        PlaybackEnded = "playback.ended",
        /// `playback.cancelled` — queued audio for a response was dropped
        /// without being played. Carries `responseId` and an optional
        /// `reason`.
        PlaybackCancelled = "playback.cancelled",
        /// `input.suspend.ack` — confirms that a host-requested suspension took
        /// effect on this client.
        ///
        /// A host must not wait for it: pressing a key to record is latency
        /// sensitive, so the acknowledgement only feeds status display and
        /// timeout healing. Backs the `input.suspend-ack` capability.
        InputSuspendAck = "input.suspend.ack",
    }
}

wire_enum! {
    /// The server → client vocabulary — all 22 `type` strings the Gateway
    /// emits on the session plane.
    ///
    /// External contract, from `shared/realtime-events.mjs:23-50`.
    ///
    /// Note that [`GatewayConnected`](Self::GatewayConnected) and
    /// [`GatewayDisconnected`](Self::GatewayDisconnected) are declared here but
    /// never emitted by the server — they are synthesised client-side from the
    /// socket's own open/close. The constants stay exported without being
    /// emitted, exactly as upstream.
    pub enum GatewayServerEvent {
        /// `gateway.connected` — declared, never emitted by the server; a
        /// client synthesises it when its socket opens.
        GatewayConnected = "gateway.connected",
        /// `gateway.disconnected` — declared, never emitted by the server; a
        /// client synthesises it when its socket closes.
        GatewayDisconnected = "gateway.disconnected",
        /// `voice.connection` — realtime transport state (`connected`,
        /// `sleeping`, …) plus the provider key.
        VoiceConnection = "voice.connection",
        /// `voice.ready` — the realtime session is usable. Carries
        /// `inputSampleRate`, `provider` and `providerLabel`, which is how a
        /// client learns what rate to capture at.
        VoiceReady = "voice.ready",
        /// `voice.state` — the conversational state machine: `idle`,
        /// `listening`, `thinking`, `speaking`. Carries `turnId` where one
        /// exists.
        VoiceState = "voice.state",
        /// `voice.ownership` — which client currently owns the microphone and
        /// the speaker, broadcast on every claim, takeover and release.
        VoiceOwnership = "voice.ownership",
        /// `voice.deactivated` — this client lost the voice session, normally
        /// to another client's takeover.
        VoiceDeactivated = "voice.deactivated",
        /// `voice.sleep` — sleep-mode progress: `sleeping`, `waking`, `awake`.
        /// Carries the configured wake word so a client can prompt with it.
        VoiceSleep = "voice.sleep",
        /// `turn.started` — a user turn was accepted; carries its `turnId`.
        TurnStarted = "turn.started",
        /// `playback.clear` — drop queued audio now. Carries a `reason`
        /// (`input_suspended`, barge-in, …).
        PlaybackClear = "playback.clear",
        /// `input.suspend` — stop capturing outright so an external controller
        /// can take the microphone: no capture, no wake-word detection.
        ///
        /// The input-side counterpart of
        /// [`PlaybackClear`](Self::PlaybackClear) and strictly stronger than
        /// the client-declared [`InputMute`](GatewayClientEvent::InputMute).
        /// Carries `owner`, `reason` and `expiresAt`.
        InputSuspend = "input.suspend",
        /// `input.resume` — capture may resume.
        InputResume = "input.resume",
        /// `audio.delta` — a chunk of response audio, tagged with its
        /// `responseId`.
        AudioDelta = "audio.delta",
        /// `audio.done` — no further audio for this `responseId`.
        AudioDone = "audio.done",
        /// `response.started` — the model began a response.
        ResponseStarted = "response.started",
        /// `response.interrupted` — the response was cut short, normally by
        /// barge-in.
        ResponseInterrupted = "response.interrupted",
        /// `transcript.delta` — an incremental transcript line. On a `user`
        /// delta `replace: true` means the content is the full running
        /// transcript, not an increment.
        TranscriptDelta = "transcript.delta",
        /// `transcript.final` — the settled transcript for a turn.
        TranscriptFinal = "transcript.final",
        /// `transcript.discard` — a user turn's transcript is void; carries an
        /// optional `reason` such as `turn_invalid`.
        TranscriptDiscard = "transcript.discard",
        /// `timeline.inline` — an inline presentation block (markdown, code or
        /// link) to render beside the conversation.
        TimelineInline = "timeline.inline",
        /// `client.state` — a client-declared state the Gateway is echoing
        /// back, e.g. `sleeping`.
        ClientState = "client.state",
        /// `error` — a human-readable failure for this session. The message is
        /// composed from the provider's `code`, `type` and `message` fields.
        Error = "error",
    }
}

wire_enum! {
    /// The server → client task plane — all 14 Work lifecycle `type` strings.
    ///
    /// External contract, from `shared/realtime-events.mjs:52-67`. These names
    /// appear in three places: on the WebSocket, as SSE frame types on
    /// `GET /api/tasks/:id/events`, and as the `type` of the Work manager's
    /// subscription events.
    ///
    /// They share the outbound namespace with [`GatewayServerEvent`] and must
    /// not collide with it — see [`GatewayOutboundEvent`].
    ///
    /// The vocabulary is neither a superset nor a subset of what the Work
    /// manager emits internally, and both mismatches are contract:
    ///
    /// - `task.accepted`, `task.notification.pending` and
    ///   `task.notification.delivered` are emitted internally but never
    ///   declared here, so they never reach a client;
    /// - [`ProgressCheck`](Self::ProgressCheck) is declared but intercepted at
    ///   the socket — it drives a spoken progress update instead of being
    ///   forwarded (`server/src/voice/realtime-gateway.mjs:831-859`);
    /// - [`NotificationOffline`](Self::NotificationOffline) is declared and
    ///   never emitted, like
    ///   [`GatewayServerEvent::GatewayConnected`]. The constant stays exported.
    ///
    /// Work whose [`WorkKind`](crate::WorkKind) is
    /// [`Control`](crate::WorkKind::Control) is filtered out entirely: no task
    /// event for it ever reaches a client.
    pub enum GatewayTaskEvent {
        /// `task.scheduled` — a reminder or scheduled task was accepted and is
        /// waiting for its timer. Its Work status is
        /// [`WorkStatus::Scheduled`](crate::WorkStatus::Scheduled).
        Scheduled = "task.scheduled",
        /// `task.scheduled.fired` — the timer fired; the Work moved from
        /// `scheduled` to `queued`.
        ScheduledFired = "task.scheduled.fired",
        /// `task.running` — the Work was admitted by the scheduler and started.
        Running = "task.running",
        /// `task.delegated` — the coordinator handed the Work to a backend
        /// session; the scheduler lane is released at this point.
        Delegated = "task.delegated",
        /// `task.finalizing` — the delegated session reported completion and
        /// the coordinator is composing the result.
        Finalizing = "task.finalizing",
        /// `task.cancelling` — cancellation is in flight. Cancellation is a
        /// *state*, not an action: the Work stays here until a path confirms
        /// the stop.
        Cancelling = "task.cancelling",
        /// `task.progress` — periodic liveness plus the bounded activity list.
        /// Emitted about once a second while the Work is active and not
        /// persisted.
        Progress = "task.progress",
        /// `task.progress.check` — the long-running-work announcement. Handled
        /// internally and **not** forwarded to clients.
        ProgressCheck = "task.progress.check",
        /// `task.completed` — terminal success; `result` is populated.
        Completed = "task.completed",
        /// `task.failed` — terminal failure; `error` is populated.
        Failed = "task.failed",
        /// `task.cancelled` — terminal cancellation, confirmed.
        Cancelled = "task.cancelled",
        /// `task.permission.requested` — a backend asked for authorization;
        /// the Work's `authorization` field carries the request.
        PermissionRequested = "task.permission.requested",
        /// `task.permission.resolved` — the authorization was approved, denied
        /// or cancelled. The frame additionally carries `permission`.
        PermissionResolved = "task.permission.resolved",
        /// `task.notification.offline` — a terminal result could not be spoken
        /// because no client was listening, so it is offered to the host as an
        /// OS notification instead.
        ///
        /// Declared upstream but never emitted; the offline hand-off travels
        /// as a host IPC message (`via:offline-notification`) rather than as a
        /// socket frame. The constant is exported all the same.
        NotificationOffline = "task.notification.offline",
    }
}

/// The complete outbound vocabulary: [`GatewayServerEvent`] ∪
/// [`GatewayTaskEvent`], 36 names on one socket.
///
/// This is upstream's `GATEWAY_SERVER_EVENT_TYPES`
/// (`shared/realtime-events.mjs:73-76`), the set the Gateway validates outbound
/// frames against. Upstream locks the two vocabularies against collision by
/// asserting that the merged `Set`'s size equals the sum of the two objects'
/// key counts (`test/realtime-events.test.mjs:11-21`); here the same property
/// is asserted directly.
///
/// Serialises as the bare wire string, so a frame's `type` round-trips through
/// this type unchanged.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(untagged)]
pub enum GatewayOutboundEvent {
    /// A session-plane event.
    Session(GatewayServerEvent),
    /// A task-plane event.
    Task(GatewayTaskEvent),
}

impl GatewayOutboundEvent {
    /// The vocabulary's name, as it appears in a
    /// [`ProtocolError::UnknownWireValue`](crate::ProtocolError::UnknownWireValue).
    pub const VOCABULARY: &'static str = "GatewayOutboundEvent";

    /// The exact wire string for this event.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Session(event) => event.as_str(),
            Self::Task(event) => event.as_str(),
        }
    }

    /// Parses any outbound wire string, returning `None` for a name that is in
    /// neither vocabulary.
    ///
    /// The runtime outbound gate. A [`GatewayClientEvent`] name never parses
    /// here, which is the direction check upstream's two `Set`s perform.
    pub fn from_wire(wire: &str) -> Option<Self> {
        if let Some(event) = GatewayServerEvent::from_wire(wire) {
            return Some(Self::Session(event));
        }
        GatewayTaskEvent::from_wire(wire).map(Self::Task)
    }
}

impl core::fmt::Display for GatewayOutboundEvent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl core::str::FromStr for GatewayOutboundEvent {
    type Err = crate::error::ProtocolError;

    fn from_str(wire: &str) -> Result<Self, Self::Err> {
        Self::from_wire(wire).ok_or_else(|| crate::error::ProtocolError::UnknownWireValue {
            vocabulary: Self::VOCABULARY,
            value: wire.to_owned(),
        })
    }
}

impl AsRef<str> for GatewayOutboundEvent {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<GatewayServerEvent> for GatewayOutboundEvent {
    fn from(event: GatewayServerEvent) -> Self {
        Self::Session(event)
    }
}

impl From<GatewayTaskEvent> for GatewayOutboundEvent {
    fn from(event: GatewayTaskEvent) -> Self {
        Self::Task(event)
    }
}
