//! The WebSocket frame codec — the whole client↔server vocabulary.
//!
//! [`via_protocol`] owns the 52 names; this module owns their **payloads**:
//! which fields each frame carries, how a missing or wrongly-typed field
//! degrades, and the order the keys go out in.
//!
//! Ported from `server/src/voice/realtime-gateway.mjs`.
//!
//! # Two decoding rules, both upstream's
//!
//! **An unparseable frame is silently ignored.** `ws.on('message', …)` wraps
//! `JSON.parse` in a bare `try { … } catch { return }`
//! (`realtime-gateway.mjs`), so a malformed frame produces no `error` event and
//! no close. [`ClientFrame::decode`] answers `None` for the same three cases:
//! not JSON, not an object, and a `type` that is not one of the sixteen.
//!
//! **Every scalar is read the way JavaScript reads it.** `String(event.x || '')`
//! turns `null`, `undefined`, `0` and `false` into `""`; `event.x === true`
//! turns everything that is not the boolean `true` into `false`. Both are
//! reproduced literally — see [`as_string`] and [`is_true`] — because a client
//! that sends `"takeover": "yes"` must *not* take over.

use serde::Serialize;
use serde_json::{Map, Value, json};
use via_protocol::{ClientInputCapabilities, ClientType, GatewayClientEvent, SessionMode};

/// `String(value || '')` — JavaScript's own coercion.
///
/// A JSON string becomes itself; every other value, `null` included, becomes
/// `""`. Upstream never sees a number here because every call site is a name,
/// an id or a reason.
#[must_use]
pub fn as_string(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_default()
}

/// `value === true` — the strict comparison, not truthiness.
#[must_use]
pub fn is_true(value: Option<&Value>) -> bool {
    value == Some(&Value::Bool(true))
}

/// `typeof value === 'boolean' ? value : undefined`.
///
/// The `connect` frame distinguishes *"the client said false"* from *"the
/// client said nothing"*, and
/// [`client_voice_capabilities`](via_voice::client_voice_capabilities) branches
/// on the difference.
#[must_use]
pub fn as_optional_bool(value: Option<&Value>) -> Option<bool> {
    value.and_then(Value::as_bool)
}

/// Who is on the other end of the socket.
///
/// **External contract** — `realtime-gateway.mjs:3399`'s `clientDescriptor`:
/// an unrecognised `clientType` falls back to `web`, the label is trimmed and
/// cut to 40 characters and **omitted entirely** when empty, and the instance
/// id is trimmed, cut to 80 and `null` when empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClientDescriptor {
    /// `desktop` | `cli` | `web`.
    #[serde(rename = "type")]
    pub client_type: ClientType,
    /// The client's own name, when it gave one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// A stable per-window id, or `null`.
    #[serde(rename = "instanceId")]
    pub instance_id: Option<String>,
}

impl Default for ClientDescriptor {
    /// The descriptor a connection has before its `connect` frame arrives:
    /// `web`, unlabelled, no instance id. Upstream's `clientDescriptor({})`
    /// answers the same thing, which is why a frame sent before `connect` is
    /// attributed to a `web` client rather than to none.
    fn default() -> Self {
        Self {
            client_type: via_protocol::DEFAULT_CLIENT_TYPE,
            label: None,
            instance_id: None,
        }
    }
}

/// How long a client label may be.
///
/// **External contract** — `realtime-gateway.mjs:3399`: `.slice(0, 40)`.
pub const CLIENT_LABEL_BOUND: usize = 40;

/// How long a client instance id may be.
///
/// **External contract** — `.slice(0, 80)`.
pub const CLIENT_INSTANCE_ID_BOUND: usize = 80;

impl ClientDescriptor {
    /// Build a descriptor from a `connect` frame.
    #[must_use]
    pub fn from_connect(fields: &Map<String, Value>) -> Self {
        let label = bounded(&as_string(fields.get("clientLabel")), CLIENT_LABEL_BOUND);
        let instance_id = bounded(
            &as_string(fields.get("clientInstanceId")),
            CLIENT_INSTANCE_ID_BOUND,
        );
        Self {
            client_type: fields
                .get("clientType")
                .and_then(Value::as_str)
                .and_then(ClientType::from_wire)
                .unwrap_or(via_protocol::DEFAULT_CLIENT_TYPE),
            label: (!label.is_empty()).then_some(label),
            instance_id: (!instance_id.is_empty()).then_some(instance_id),
        }
    }
}

/// `String(x).trim().slice(0, n)`, counting Unicode scalar values.
fn bounded(value: &str, limit: usize) -> String {
    value.trim().chars().take(limit).collect()
}

/// The `connect` frame, decoded.
///
/// Every field is optional on the wire; the defaults here are upstream's.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Connect {
    /// Who is connecting.
    pub descriptor: ClientDescriptor,
    /// The realtime provider this session wants, when it named one.
    pub provider: Option<String>,
    /// **VIA's own**: the session mode, fixed for the connection's lifetime
    /// (`docs/architecture.md` §2). Absent means [`SessionMode::Agent`].
    pub mode: SessionMode,
    /// The legacy single audio flag.
    pub voice_enabled: bool,
    /// Explicit capture capability.
    pub input_enabled: Option<bool>,
    /// Explicit playback capability.
    pub output_enabled: Option<bool>,
    /// A client that will never play audio.
    pub text_only: bool,
    /// The client wants wake-word listening only, with no realtime session
    /// until the phrase fires.
    ///
    /// **External contract** — `docs/reference/contracts.json`, `json-field` /
    /// *connect event fields (client -> server)*: `wakeWordOnly`. Upstream's
    /// `requestExplicitSleep()` (`realtime-gateway.mjs:1690-1704`) is what it
    /// means: skip `ensureFrontend()` entirely and go straight to preparing a
    /// detector, so a desktop client that only wants to be woken never pays
    /// for a realtime session it has not asked for yet.
    pub wake_word_only: bool,
    /// Whether this client is claiming the slot from a live holder.
    pub takeover: bool,
    /// The client's IANA zone.
    pub time_zone: String,
    /// The client's BCP-47 tag.
    pub locale: String,
    /// The client's launch directory.
    pub working_directory: String,
    /// The states the client declares it manages itself.
    ///
    /// **External contract** — only a `desktop` client's `sleeping` is honoured
    /// (`realtime-gateway.mjs`), because a desktop that advertises the sleeping
    /// state owns its own inactivity policy and the Gateway's legacy timer must
    /// not fight it.
    pub client_states: Vec<String>,
    /// What kinds of input the client can send, or `None`.
    pub input_capabilities: Option<ClientInputCapabilities>,
}

/// One decoded client frame.
///
/// Sixteen variants, one per [`GatewayClientEvent`]. Matching is exhaustive on
/// purpose: `via-protocol`'s vocabularies are not `#[non_exhaustive]`, so a new
/// event is a compile error here rather than a frame that is quietly dropped.
#[derive(Debug, Clone, PartialEq)]
pub enum ClientFrame {
    /// `connect`.
    Connect(Box<Connect>),
    /// `unmute` — `{takeover}`.
    Unmute {
        /// Claim the slot from a live holder.
        takeover: bool,
    },
    /// `mute`.
    Mute,
    /// `input.unmute` — `{takeover}`.
    InputUnmute {
        /// Claim the slot from a live holder.
        takeover: bool,
    },
    /// `input.mute`.
    InputMute,
    /// `audio.append` — `{audio}`, base64 PCM.
    AudioAppend {
        /// The chunk, still base64: it is forwarded, not decoded, so a
        /// provider that wants a different container decodes once.
        audio: String,
    },
    /// `text.message` — `{text}`.
    TextMessage {
        /// The typed turn.
        text: String,
        /// Any parts the client attached inline.
        parts: Vec<Value>,
    },
    /// `input.message` — `{text, parts}`. Handled identically to
    /// [`TextMessage`](Self::TextMessage).
    InputMessage {
        /// The typed turn.
        text: String,
        /// Previously staged or inline parts.
        parts: Vec<Value>,
    },
    /// `input.parts` — `{parts}`.
    InputParts {
        /// The parts to stage for the next message.
        parts: Vec<Value>,
    },
    /// `interrupt`.
    Interrupt,
    /// `sleep`.
    Sleep,
    /// `wake`.
    Wake,
    /// `playback.started` — `{responseId}`.
    PlaybackStarted {
        /// Which response began playing.
        response_id: String,
    },
    /// `playback.ended` — `{responseId}`.
    PlaybackEnded {
        /// Which response finished playing.
        response_id: String,
    },
    /// `playback.cancelled` — `{responseId, reason}`.
    PlaybackCancelled {
        /// Which response was dropped.
        response_id: String,
        /// Why, when the client said.
        reason: String,
    },
    /// `input.suspend.ack` — `{owner}`.
    InputSuspendAck {
        /// Which suspension is being acknowledged.
        owner: String,
    },
}

impl ClientFrame {
    /// Decode one text frame.
    ///
    /// `None` for anything that is not a JSON object whose `type` is one of the
    /// sixteen client events. Upstream ignores all three cases identically, and
    /// so does this: a client that speaks nonsense gets no answer at all.
    #[must_use]
    pub fn decode(raw: &str) -> Option<Self> {
        let value: Value = serde_json::from_str(raw).ok()?;
        let fields = value.as_object()?;
        let event = GatewayClientEvent::from_wire(fields.get("type")?.as_str()?)?;
        Some(match event {
            GatewayClientEvent::Connect => Self::Connect(Box::new(Connect {
                descriptor: ClientDescriptor::from_connect(fields),
                provider: {
                    let provider = as_string(fields.get("provider"));
                    (!provider.is_empty()).then_some(provider)
                },
                mode: fields
                    .get("mode")
                    .and_then(Value::as_str)
                    .and_then(SessionMode::from_wire)
                    .unwrap_or_default(),
                voice_enabled: is_true(fields.get("voiceEnabled")),
                input_enabled: as_optional_bool(fields.get("inputEnabled")),
                output_enabled: as_optional_bool(fields.get("outputEnabled")),
                text_only: is_true(fields.get("textOnly")),
                wake_word_only: is_true(fields.get("wakeWordOnly")),
                takeover: is_true(fields.get("takeover")),
                time_zone: as_string(fields.get("timeZone")),
                locale: as_string(fields.get("locale")),
                working_directory: as_string(fields.get("workingDirectory")),
                client_states: fields
                    .get("clientStates")
                    .and_then(Value::as_array)
                    .map(|states| {
                        states
                            .iter()
                            .filter_map(Value::as_str)
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default(),
                input_capabilities: fields.get("inputCapabilities").and_then(|capabilities| {
                    let capabilities = capabilities.as_object()?;
                    Some(ClientInputCapabilities {
                        text: is_true(capabilities.get("text")),
                        audio: is_true(capabilities.get("audio")),
                        image: is_true(capabilities.get("image")),
                        resource: is_true(capabilities.get("resource")),
                    })
                }),
            })),
            GatewayClientEvent::Unmute => Self::Unmute {
                takeover: is_true(fields.get("takeover")),
            },
            GatewayClientEvent::Mute => Self::Mute,
            GatewayClientEvent::InputUnmute => Self::InputUnmute {
                takeover: is_true(fields.get("takeover")),
            },
            GatewayClientEvent::InputMute => Self::InputMute,
            GatewayClientEvent::AudioAppend => Self::AudioAppend {
                audio: as_string(fields.get("audio")),
            },
            GatewayClientEvent::TextMessage => Self::TextMessage {
                text: as_string(fields.get("text")),
                parts: parts(fields),
            },
            GatewayClientEvent::InputMessage => Self::InputMessage {
                text: as_string(fields.get("text")),
                parts: parts(fields),
            },
            GatewayClientEvent::InputParts => Self::InputParts {
                parts: parts(fields),
            },
            GatewayClientEvent::Interrupt => Self::Interrupt,
            GatewayClientEvent::Sleep => Self::Sleep,
            GatewayClientEvent::Wake => Self::Wake,
            GatewayClientEvent::PlaybackStarted => Self::PlaybackStarted {
                response_id: as_string(fields.get("responseId")),
            },
            GatewayClientEvent::PlaybackEnded => Self::PlaybackEnded {
                response_id: as_string(fields.get("responseId")),
            },
            GatewayClientEvent::PlaybackCancelled => Self::PlaybackCancelled {
                response_id: as_string(fields.get("responseId")),
                reason: as_string(fields.get("reason")),
            },
            GatewayClientEvent::InputSuspendAck => Self::InputSuspendAck {
                owner: as_string(fields.get("owner")),
            },
        })
    }

    /// Which vocabulary member this frame is.
    #[must_use]
    pub const fn event(&self) -> GatewayClientEvent {
        match self {
            Self::Connect(_) => GatewayClientEvent::Connect,
            Self::Unmute { .. } => GatewayClientEvent::Unmute,
            Self::Mute => GatewayClientEvent::Mute,
            Self::InputUnmute { .. } => GatewayClientEvent::InputUnmute,
            Self::InputMute => GatewayClientEvent::InputMute,
            Self::AudioAppend { .. } => GatewayClientEvent::AudioAppend,
            Self::TextMessage { .. } => GatewayClientEvent::TextMessage,
            Self::InputMessage { .. } => GatewayClientEvent::InputMessage,
            Self::InputParts { .. } => GatewayClientEvent::InputParts,
            Self::Interrupt => GatewayClientEvent::Interrupt,
            Self::Sleep => GatewayClientEvent::Sleep,
            Self::Wake => GatewayClientEvent::Wake,
            Self::PlaybackStarted { .. } => GatewayClientEvent::PlaybackStarted,
            Self::PlaybackEnded { .. } => GatewayClientEvent::PlaybackEnded,
            Self::PlaybackCancelled { .. } => GatewayClientEvent::PlaybackCancelled,
            Self::InputSuspendAck { .. } => GatewayClientEvent::InputSuspendAck,
        }
    }
}

/// `Array.isArray(event.parts) ? event.parts : []`.
fn parts(fields: &Map<String, Value>) -> Vec<Value> {
    fields
        .get("parts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// One outbound frame.
///
/// `type` is always the first key, because upstream's object literals put it
/// there and a client that logs raw frames reads it first.
#[derive(Debug, Clone, PartialEq)]
pub struct ServerFrame(Map<String, Value>);

impl ServerFrame {
    /// A frame carrying only its type.
    #[must_use]
    pub fn new(event: impl AsRef<str>) -> Self {
        let mut fields = Map::new();
        fields.insert("type".to_owned(), Value::String(event.as_ref().to_owned()));
        Self(fields)
    }

    /// Add a field. Insertion order is preserved.
    #[must_use]
    pub fn with(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.0.insert(key.to_owned(), value.into());
        self
    }

    /// Add a field only when there is one, reproducing `...(x ? {k: x} : {})`.
    #[must_use]
    pub fn with_optional(self, key: &str, value: Option<impl Into<Value>>) -> Self {
        match value {
            Some(value) => self.with(key, value),
            None => self,
        }
    }

    /// The frame as JSON.
    #[must_use]
    pub fn into_value(self) -> Value {
        Value::Object(self.0)
    }

    /// The frame as the text a socket carries.
    #[must_use]
    pub fn encode(self) -> String {
        self.into_value().to_string()
    }
}

/// `playback.clear` with a reason.
///
/// **External contract** — `realtime-gateway.mjs`: `input_suspended` when the
/// host takes the microphone, `user_interruption` on a barge-in, and no reason
/// at all when a client loses the voice slot.
#[must_use]
pub fn playback_clear(reason: Option<&str>) -> ServerFrame {
    ServerFrame::new(via_protocol::GatewayServerEvent::PlaybackClear.as_str())
        .with_optional("reason", reason)
}

/// `input.suspend` — `{owner, reason, expiresAt}`.
#[must_use]
pub fn input_suspend(status: &via_voice::SuspensionStatus) -> ServerFrame {
    ServerFrame::new(via_protocol::GatewayServerEvent::InputSuspend.as_str())
        .with(
            "owner",
            status.owner.clone().map_or(Value::Null, Value::from),
        )
        .with("reason", status.reason.clone())
        .with(
            "expiresAt",
            status.expires_at.map_or(Value::Null, Value::from),
        )
}

/// `input.resume` — no payload.
#[must_use]
pub fn input_resume() -> ServerFrame {
    ServerFrame::new(via_protocol::GatewayServerEvent::InputResume.as_str())
}

/// `voice.ownership` — `{state, holder}`.
///
/// **External contract** — `realtime-gateway.mjs`'s `broadcastVoiceOwnership`:
/// `active` for the holder, `busy` for everyone else while someone holds, and
/// `available` when nobody does.
#[must_use]
pub fn voice_ownership(state: &str, holder: Option<&ClientDescriptor>) -> ServerFrame {
    ServerFrame::new(via_protocol::GatewayServerEvent::VoiceOwnership.as_str())
        .with("state", state)
        .with(
            "holder",
            holder.map_or(Value::Null, |descriptor| {
                serde_json::to_value(descriptor).unwrap_or(Value::Null)
            }),
        )
}

/// `voice.ready` — `{inputSampleRate, provider, providerLabel}`.
#[must_use]
pub fn voice_ready(input_sample_rate: u32, provider: &str, provider_label: &str) -> ServerFrame {
    ServerFrame::new(via_protocol::GatewayServerEvent::VoiceReady.as_str())
        .with("inputSampleRate", input_sample_rate)
        .with("provider", provider)
        .with("providerLabel", provider_label)
}

/// `error` — `{message}`.
#[must_use]
pub fn error(message: &str) -> ServerFrame {
    ServerFrame::new(via_protocol::GatewayServerEvent::Error.as_str()).with("message", message)
}

/// `voice.connection` — `{state, provider, message?}`.
///
/// **External contract** — `docs/reference/contracts.json`, `json-field` /
/// *voice.connection*: `{ type, state, provider, message? }`, `message`
/// carried only for `unavailable`. `state`/`provider`/the blocking text are
/// [`via_realtime::realtime_connection_status`]'s own frozen
/// `{provider, state, error?}` — this only renames `error` to the wire's own
/// key, `message`, which upstream's five literal `send(ws, {...})` call sites
/// spell directly (`realtime-gateway.mjs:1389-1394,1444-1448,1504-1509,
/// 1539-1543,1582-1588`) rather than through the shared helper.
#[must_use]
pub fn voice_connection(status: &via_realtime::RealtimeConnectionStatus) -> ServerFrame {
    ServerFrame::new(via_protocol::GatewayServerEvent::VoiceConnection.as_str())
        .with("state", status.state.as_str())
        .with("provider", status.provider.clone())
        .with_optional("message", status.error.clone())
}

/// `client.state` — `{state}`.
///
/// **External contract** — `tool-call-handler.mjs:65,664`
/// (`requestClientState`): an `enter_sleep` call echoed back to a client that
/// declared it manages the state itself.
#[must_use]
pub fn client_state(state: &str) -> ServerFrame {
    ServerFrame::new(via_protocol::GatewayServerEvent::ClientState.as_str()).with("state", state)
}

/// `voice.sleep` — the detector's own lifecycle, distinct from the three
/// client-requested states (`sleeping`/`waking`/`awake`) built ad hoc at their
/// own call sites in [`crate::realtime::connection`].
///
/// **External contract** — `docs/reference/contracts.json`, `json-field` /
/// *voice.sleep*: `{ type, state, wakeWord?, timeoutMs?, message? }`. Each of
/// the four functions below reproduces one state's own literal key order —
/// they are not interchangeable, because upstream's four `send(ws, {...})`
/// call sites do not share one shape:
///
/// | State | Keys, in order | Upstream |
/// | --- | --- | --- |
/// | `preparing` | `state, wakeWord` | `realtime-gateway.mjs:1632-1636` |
/// | `enabled` | `state, timeoutMs, wakeWord` | `:1653-1657` |
/// | `disabled` | `state, message` | `:1664-1669,1673-1677` |
/// | `detected` | `state, wakeWord` | `:1752-1756` |
pub mod voice_sleep {
    use super::ServerFrame;

    fn frame(state: &str) -> ServerFrame {
        ServerFrame::new(via_protocol::GatewayServerEvent::VoiceSleep.as_str()).with("state", state)
    }

    /// `{type, state: 'preparing', wakeWord}` — a detector is being built.
    #[must_use]
    pub fn preparing(wake_word: &str) -> ServerFrame {
        frame("preparing").with("wakeWord", wake_word)
    }

    /// `{type, state: 'enabled', timeoutMs, wakeWord}` — listening.
    #[must_use]
    pub fn enabled(timeout_ms: u64, wake_word: &str) -> ServerFrame {
        frame("enabled")
            .with("timeoutMs", timeout_ms)
            .with("wakeWord", wake_word)
    }

    /// `{type, state: 'disabled', message}` — nothing is listening, and why.
    #[must_use]
    pub fn disabled(message: &str) -> ServerFrame {
        frame("disabled").with("message", message)
    }

    /// `{type, state: 'detected', wakeWord}` — the phrase just fired.
    #[must_use]
    pub fn detected(wake_word: &str) -> ServerFrame {
        frame("detected").with("wakeWord", wake_word)
    }
}

/// The seven-key `publicResponseContext(context)` spread.
///
/// **External contract** — `realtime-gateway.mjs:582-590`, spread into
/// `response.started`, `response.interrupted`, `transcript.delta` and
/// `transcript.final` (`:601-625,701,774`) — never nested, always spliced
/// straight into the frame's own keys, key order included. `taskId` is always
/// present, `null` when there is none, because upstream's context object
/// always carries that key; `taskIds` / `turnIds` / `deliverySequence` are
/// **omitted** rather than written `null` or `[]`, because upstream's context
/// never carries those keys at all until something sets them, and a spread of
/// a genuinely absent property serializes to nothing.
#[must_use]
pub fn with_public_response_context(
    frame: ServerFrame,
    context: &via_voice::ResponseContext,
) -> ServerFrame {
    frame
        .with("turnId", context.turn_id.clone())
        .with(
            "taskId",
            context.task_id.clone().map_or(Value::Null, Value::from),
        )
        .with_optional(
            "taskIds",
            (!context.task_ids.is_empty()).then(|| context.task_ids.clone()),
        )
        .with_optional(
            "turnIds",
            (!context.turn_ids.is_empty()).then(|| context.turn_ids.clone()),
        )
        .with("origin", context.origin.as_str())
        .with("turnGeneration", context.turn_generation)
        .with_optional("deliverySequence", context.delivery_sequence)
}

/// `timeline.inline` — `{type, item:{id, taskId, turnId, ...extra}}`.
///
/// **External contract** — `docs/reference/contracts.json`, `json-field` /
/// *timeline.inline frame*: `{type:'timeline.inline', item:{id:
/// `inline_${task.id}`, taskId, turnId|null, title, format, content}}`.
/// `extra` carries whatever the caller spreads after `turnId` — the typed
/// `{title, format, content}` triple for a terminal result
/// ([`inline_block_extra`]), or the opaque pass-through object for a
/// delegation announcement ([`delegated_inline_extra`]).
#[must_use]
pub fn timeline_inline(
    id: String,
    task_id: &str,
    turn_id: Option<&str>,
    extra: Map<String, Value>,
) -> ServerFrame {
    let mut item = Map::new();
    item.insert("id".to_owned(), Value::String(id));
    item.insert("taskId".to_owned(), Value::String(task_id.to_owned()));
    item.insert(
        "turnId".to_owned(),
        turn_id.map_or(Value::Null, |turn_id| Value::String(turn_id.to_owned())),
    );
    for (key, value) in extra {
        item.insert(key, value);
    }
    ServerFrame::new(via_protocol::GatewayServerEvent::TimelineInline.as_str())
        .with("item", Value::Object(item))
}

/// The `{title, format, content}` triple a terminal result's inline block
/// spreads into a [`timeline_inline`] item.
///
/// **External contract** — `realtime-gateway.mjs:952-963`:
/// `...task.resultMetadata.presentation.inline`. `via_work::presentation::project`
/// already guarantees the content is non-blank, so there is no truthiness
/// check to repeat here.
#[must_use]
pub fn inline_block_extra(inline: &via_work::InlineBlock) -> Map<String, Value> {
    let mut extra = Map::new();
    extra.insert("title".to_owned(), Value::String(inline.title.clone()));
    extra.insert(
        "format".to_owned(),
        Value::String(inline.format.as_str().to_owned()),
    );
    extra.insert("content".to_owned(), Value::String(inline.content.clone()));
    extra
}

/// The delegated variant's inline block: opaque JSON, present only when
/// `content` is a non-blank string.
///
/// **External contract** — `realtime-gateway.mjs:919-931`:
/// `presentation?.inline?.content` gates the frame, and the object is spread
/// **verbatim** — unlike a terminal result's inline block, this one is never
/// reshaped through [`via_work::presentation::project`], because
/// [`via_work::DelegationPresentation::inline`] is passed through unprojected.
#[must_use]
pub fn delegated_inline_extra(inline: &Value) -> Option<Map<String, Value>> {
    let object = inline.as_object()?;
    let content = object.get("content")?.as_str()?;
    if content.trim().is_empty() {
        return None;
    }
    Some(object.clone())
}

/// One Work-plane frame.
///
/// **External contract** — `{type, task}` for every event, plus `permission`
/// on `task.permission.resolved` and `message`/`delegated` on
/// `task.progress.check`. The last is declared and never forwarded; see
/// [`crate::realtime::forwards_to_client`].
#[must_use]
pub fn task_frame(event: &via_work::WorkEvent) -> ServerFrame {
    let mut frame = ServerFrame::new(event.kind.as_str()).with("task", json!(event.task.clone()));
    match &event.details {
        via_work::WorkEventDetails::None => {}
        via_work::WorkEventDetails::ProgressCheck { message, delegated } => {
            frame = frame
                .with("message", message.clone())
                .with("delegated", *delegated);
        }
        via_work::WorkEventDetails::PermissionResolved { permission } => {
            frame = frame.with("permission", json!(permission.clone()));
        }
    }
    frame
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn an_unparseable_frame_is_ignored_rather_than_answered() {
        assert_eq!(ClientFrame::decode("not json"), None);
        assert_eq!(ClientFrame::decode("[1,2,3]"), None);
        assert_eq!(ClientFrame::decode("{}"), None);
        assert_eq!(ClientFrame::decode("{\"type\":42}"), None);
        assert_eq!(ClientFrame::decode("{\"type\":\"audio.delta\"}"), None);
        assert_eq!(ClientFrame::decode("{\"type\":\"nope\"}"), None);
    }

    #[test]
    fn every_client_event_decodes_from_its_own_name() {
        for event in GatewayClientEvent::ALL {
            let frame = ClientFrame::decode(&json!({ "type": event.as_str() }).to_string())
                .unwrap_or_else(|| panic!("{} must decode", event.as_str()));
            assert_eq!(frame.event(), *event);
        }
    }

    #[test]
    fn takeover_is_the_boolean_true_and_nothing_else() {
        let takeover = |value: Value| match ClientFrame::decode(
            &json!({ "type": "unmute", "takeover": value }).to_string(),
        ) {
            Some(ClientFrame::Unmute { takeover }) => takeover,
            other => panic!("expected unmute, got {other:?}"),
        };
        assert!(takeover(json!(true)));
        assert!(!takeover(json!("true")));
        assert!(!takeover(json!(1)));
        assert!(!takeover(json!(null)));
    }

    #[test]
    fn connect_distinguishes_said_false_from_said_nothing() {
        let decoded =
            ClientFrame::decode(&json!({ "type": "connect", "outputEnabled": false }).to_string());
        let Some(ClientFrame::Connect(connect)) = decoded else {
            panic!("expected connect")
        };
        assert_eq!(connect.output_enabled, Some(false));
        assert_eq!(connect.input_enabled, None);
    }

    #[test]
    fn an_unknown_client_type_falls_back_to_web_and_labels_are_bounded() {
        let decoded = ClientFrame::decode(
            &json!({
                "type": "connect",
                "clientType": "toaster",
                "clientLabel": format!("  {}  ", "x".repeat(60)),
                "clientInstanceId": "   ",
            })
            .to_string(),
        );
        let Some(ClientFrame::Connect(connect)) = decoded else {
            panic!("expected connect")
        };
        assert_eq!(connect.descriptor.client_type, ClientType::Web);
        assert_eq!(
            connect.descriptor.label.as_deref().map(str::len),
            Some(CLIENT_LABEL_BOUND),
        );
        assert_eq!(
            connect.descriptor.instance_id, None,
            "a blank instance id is null, not an empty string"
        );
    }

    #[test]
    fn an_empty_label_is_omitted_from_the_descriptor_entirely() {
        let descriptor = ClientDescriptor::from_connect(
            json!({ "clientType": "cli" }).as_object().expect("object"),
        );
        let encoded = serde_json::to_value(&descriptor).expect("serializes");
        assert_eq!(encoded, json!({ "type": "cli", "instanceId": null }));
    }

    #[test]
    fn the_session_mode_is_chosen_at_connect_and_defaults_to_agent() {
        let mode = |frame: Value| match ClientFrame::decode(&frame.to_string()) {
            Some(ClientFrame::Connect(connect)) => connect.mode,
            other => panic!("expected connect, got {other:?}"),
        };
        assert_eq!(mode(json!({ "type": "connect" })), SessionMode::Agent);
        assert_eq!(
            mode(json!({ "type": "connect", "mode": "dictation" })),
            SessionMode::Dictation,
        );
        assert_eq!(
            mode(json!({ "type": "connect", "mode": "nonsense" })),
            SessionMode::Agent,
            "an unknown mode is the default, not a refusal",
        );
    }

    #[test]
    fn type_is_the_first_key_of_every_outbound_frame() {
        let frame = ServerFrame::new("voice.ready")
            .with("inputSampleRate", 16_000)
            .with("provider", "mock");
        let encoded = frame.encode();
        assert!(
            encoded.starts_with("{\"type\":\"voice.ready\""),
            "got {encoded}"
        );
    }

    #[test]
    fn the_suspension_frames_are_the_catalogued_shapes() {
        let status = via_voice::SuspensionStatus {
            suspended: true,
            holders: Vec::new(),
            owner: Some("host-app".to_owned()),
            reason: "dictation".to_owned(),
            expires_at: Some(1_700_000_000_000),
        };
        assert_eq!(
            input_suspend(&status).into_value(),
            json!({
                "type": "input.suspend",
                "owner": "host-app",
                "reason": "dictation",
                "expiresAt": 1_700_000_000_000_i64,
            }),
        );
        assert_eq!(
            input_resume().into_value(),
            json!({ "type": "input.resume" })
        );
        assert_eq!(
            playback_clear(Some("input_suspended")).into_value(),
            json!({ "type": "playback.clear", "reason": "input_suspended" }),
        );
        assert_eq!(
            playback_clear(None).into_value(),
            json!({ "type": "playback.clear" }),
            "a slot loss clears playback with no reason at all",
        );
    }

    #[test]
    fn voice_connection_renames_error_to_the_wires_message_key() {
        let connected =
            via_realtime::realtime_connection_status(&via_realtime::RealtimeConnectionInputs {
                provider: "dashscope".to_owned(),
                ready: true,
                ..via_realtime::RealtimeConnectionInputs::default()
            });
        assert_eq!(
            voice_connection(&connected).into_value(),
            json!({ "type": "voice.connection", "state": "connected", "provider": "dashscope" }),
            "a healthy status carries no message key at all",
        );

        let unavailable =
            via_realtime::realtime_connection_status(&via_realtime::RealtimeConnectionInputs {
                provider: "dashscope".to_owned(),
                blocked_error: "凭据缺失".to_owned(),
                ..via_realtime::RealtimeConnectionInputs::default()
            });
        assert_eq!(
            voice_connection(&unavailable).into_value(),
            json!({
                "type": "voice.connection",
                "state": "unavailable",
                "provider": "dashscope",
                "message": "凭据缺失",
            }),
        );
    }

    #[test]
    fn client_state_carries_exactly_the_declared_state() {
        assert_eq!(
            client_state("sleeping").into_value(),
            json!({ "type": "client.state", "state": "sleeping" }),
        );
    }

    #[test]
    fn wake_word_only_is_the_boolean_true_and_nothing_else() {
        let wake_word_only = |value: Value| match ClientFrame::decode(
            &json!({ "type": "connect", "wakeWordOnly": value }).to_string(),
        ) {
            Some(ClientFrame::Connect(connect)) => connect.wake_word_only,
            other => panic!("expected connect, got {other:?}"),
        };
        assert!(wake_word_only(json!(true)));
        assert!(!wake_word_only(json!("true")));
        assert!(!wake_word_only(json!(1)));
        assert!(!wake_word_only(Value::Null));
        assert!(
            !ClientFrame::decode(&json!({ "type": "connect" }).to_string())
                .and_then(|frame| match frame {
                    ClientFrame::Connect(connect) => Some(connect.wake_word_only),
                    _ => None,
                })
                .unwrap_or_default()
        );
    }

    /// A frame's keys, in the order they were inserted — `assert_eq!` on two
    /// `Value::Object`s compares as maps and would not catch a reordering,
    /// which is exactly the thing each state's own literal key order matters
    /// for here.
    fn keys(frame: ServerFrame) -> Vec<String> {
        frame
            .into_value()
            .as_object()
            .expect("a frame is always an object")
            .keys()
            .cloned()
            .collect()
    }

    #[test]
    fn each_voice_sleep_state_has_its_own_catalogued_key_order() {
        assert_eq!(
            voice_sleep::preparing("hey via").into_value(),
            json!({ "type": "voice.sleep", "state": "preparing", "wakeWord": "hey via" }),
        );
        assert_eq!(
            keys(voice_sleep::preparing("hey via")),
            ["type", "state", "wakeWord"],
        );

        assert_eq!(
            voice_sleep::enabled(5_000, "hey via").into_value(),
            json!({
                "type": "voice.sleep",
                "state": "enabled",
                "timeoutMs": 5_000,
                "wakeWord": "hey via",
            }),
        );
        assert_eq!(
            keys(voice_sleep::enabled(5_000, "hey via")),
            ["type", "state", "timeoutMs", "wakeWord"],
            "enabled carries timeoutMs before wakeWord",
        );

        assert_eq!(
            voice_sleep::disabled("no wake phrase is configured").into_value(),
            json!({
                "type": "voice.sleep",
                "state": "disabled",
                "message": "no wake phrase is configured",
            }),
            "disabled carries no wakeWord at all",
        );
        assert_eq!(
            keys(voice_sleep::disabled("no wake phrase is configured")),
            ["type", "state", "message"],
        );

        assert_eq!(
            voice_sleep::detected("hey via").into_value(),
            json!({ "type": "voice.sleep", "state": "detected", "wakeWord": "hey via" }),
        );
        assert_eq!(
            keys(voice_sleep::detected("hey via")),
            ["type", "state", "wakeWord"],
        );
    }

    #[test]
    fn the_public_response_context_spread_omits_what_was_never_set() {
        let context = via_voice::ResponseContext {
            turn_id: "voice-1".to_owned(),
            origin: via_voice::ResponseOrigin::Model,
            turn_generation: -1,
            ..via_voice::ResponseContext::default()
        };
        let frame = with_public_response_context(ServerFrame::new("response.started"), &context)
            .into_value();
        assert_eq!(
            frame,
            json!({
                "type": "response.started",
                "turnId": "voice-1",
                "taskId": null,
                "origin": "model",
                "turnGeneration": -1,
            }),
            "taskIds, turnIds and deliverySequence are absent, not null or empty",
        );
    }

    #[test]
    fn timeline_inline_carries_the_terminal_result_shape_in_order() {
        let inline = via_work::InlineBlock {
            title: "Diff".to_owned(),
            format: via_work::InlineFormat::Code,
            content: "+ line".to_owned(),
        };
        let frame = timeline_inline(
            "inline_work_1".to_owned(),
            "work_1",
            Some("voice-1"),
            inline_block_extra(&inline),
        )
        .into_value();
        assert_eq!(
            frame,
            json!({
                "type": "timeline.inline",
                "item": {
                    "id": "inline_work_1",
                    "taskId": "work_1",
                    "turnId": "voice-1",
                    "title": "Diff",
                    "format": "code",
                    "content": "+ line",
                },
            }),
        );
        let keys: Vec<&String> = frame["item"].as_object().expect("object").keys().collect();
        assert_eq!(
            keys,
            ["id", "taskId", "turnId", "title", "format", "content"]
        );
    }

    #[test]
    fn timeline_inline_writes_a_null_turn_id_rather_than_omitting_it() {
        let inline = via_work::InlineBlock {
            title: String::new(),
            format: via_work::InlineFormat::Markdown,
            content: "body".to_owned(),
        };
        let frame = timeline_inline(
            "inline_work_1".to_owned(),
            "work_1",
            None,
            inline_block_extra(&inline),
        )
        .into_value();
        assert_eq!(frame["item"]["turnId"], Value::Null);
    }

    #[test]
    fn the_delegated_inline_extra_spreads_the_opaque_object_verbatim() {
        let extra = delegated_inline_extra(&json!({
            "content": "spawned worker",
            "kind": "spawn",
            "anything": {"nested": true},
        }))
        .expect("content is a non-blank string");
        let frame = timeline_inline("inline_work_1_delegated".to_owned(), "work_1", None, extra)
            .into_value();
        assert_eq!(
            frame,
            json!({
                "type": "timeline.inline",
                "item": {
                    "id": "inline_work_1_delegated",
                    "taskId": "work_1",
                    "turnId": null,
                    "content": "spawned worker",
                    "kind": "spawn",
                    "anything": {"nested": true},
                },
            }),
        );
    }

    #[test]
    fn the_delegated_inline_extra_refuses_a_blank_or_missing_content() {
        assert_eq!(delegated_inline_extra(&json!({"content": "   "})), None);
        assert_eq!(
            delegated_inline_extra(&json!({"title": "no content"})),
            None
        );
        assert_eq!(delegated_inline_extra(&json!("not an object")), None);
        assert_eq!(delegated_inline_extra(&json!({"content": 42})), None);
    }

    #[test]
    fn the_public_response_context_spread_carries_every_key_when_set() {
        let context = via_voice::ResponseContext {
            turn_id: "voice-1".to_owned(),
            task_id: Some("work_1".to_owned()),
            task_ids: vec!["work_1".to_owned(), "work_2".to_owned()],
            turn_ids: vec!["voice-1".to_owned()],
            origin: via_voice::ResponseOrigin::Announcement,
            turn_generation: 3,
            delivery_sequence: Some(7),
            ..via_voice::ResponseContext::default()
        };
        let frame =
            with_public_response_context(ServerFrame::new("response.interrupted"), &context)
                .into_value();
        assert_eq!(
            frame,
            json!({
                "type": "response.interrupted",
                "turnId": "voice-1",
                "taskId": "work_1",
                "taskIds": ["work_1", "work_2"],
                "turnIds": ["voice-1"],
                "origin": "announcement",
                "turnGeneration": 3,
                "deliverySequence": 7,
            }),
        );
    }
}
