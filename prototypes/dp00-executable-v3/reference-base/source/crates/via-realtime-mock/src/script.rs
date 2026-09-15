//! What the mock provider does, written down.
//!
//! A realtime service is a function from the frames a client writes to the
//! events it sends back. A [`Script`] is that function, spelled as a list of
//! [`ScriptStep`]s: *when this frame goes out, send these events back, with
//! these delays.*
//!
//! # The four rules
//!
//! 1. **Steps are scanned in declaration order.** The first step that matches
//!    the frame and has not been exhausted fires, and only that one.
//! 2. **A step is [`Repeat::Once`] unless it says otherwise.** So a script of
//!    three [`Trigger::ResponseCreate`] steps is three consecutive model turns,
//!    which is how a tool round trip is written.
//! 3. **When no step matches, the mock behaves like a provider.** It
//!    acknowledges `session.update`, echoes `conversation.item.create`, and
//!    answers `response.create` with `response.created` + `response.done`. An
//!    empty script is therefore a working provider, not a mute one.
//! 4. **Ids are stamped, not written.** An emitted event that is response
//!    activity and carries no response id gets the id of the response its
//!    trigger belongs to; a `conversation.item.created` with no item id gets
//!    the one being acknowledged. [`Emission::verbatim`] opts out, which is how
//!    the adversarial cases — a `response.created` with no id at all — are
//!    scripted.
//!
//! # Determinism
//!
//! Every id the mock mints is a counter: `resp_1`, `item_mock_1`, `event_mock_1`.
//! There is no clock and no RNG anywhere in this crate, so the same script
//! produces the same bytes on every run and on every host — with one named
//! exception, the client-minted item id an echoing provider hands back. See the
//! crate docs.
//!
//! Delays are [`tokio::time::sleep`], so under `#[tokio::test(start_paused = true)]`
//! they cost no wall-clock time and cooperate with `tokio::time::advance`.
//!
//! # Example
//!
//! ```
//! use via_realtime_mock::{Script, script::turns};
//!
//! // Turn one calls a tool; turn two speaks the answer.
//! let script = Script::conversation()
//!     .turn(turns::call_tool("call_1", "spawn_thinking", &serde_json::json!({
//!         "objective": "summarise the log",
//!     })))
//!     .turn(turns::say("I have started on that."));
//! assert_eq!(script.steps().len(), 2);
//! ```

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use via_protocol::SessionMode;

use crate::error::MockError;
use crate::provider::MockProviderSpec;

/// The `response.done` status a turn that worked reports.
///
/// External contract, by exclusion — `via_realtime::FAILED_RESPONSE_STATUSES`
/// is the closed list of the three statuses that are *not* this.
pub const COMPLETED: &str = "completed";

/// The `response.done` status a cancelled turn reports.
///
/// External contract — one of `via_realtime::FAILED_RESPONSE_STATUSES`.
pub const CANCELLED: &str = "cancelled";

/// Which frame a [`ScriptStep`] answers.
///
/// The seven named variants are the frames `via-realtime` writes;
/// [`Frame`](Self::Frame) is the escape hatch for a dialect frame this crate has
/// no name for — including one a provider extension invented.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Trigger {
    /// `session.update`.
    SessionUpdate,
    /// `conversation.item.create`.
    ItemCreate,
    /// `response.create`.
    ResponseCreate,
    /// `response.cancel`.
    ResponseCancel,
    /// `input_audio_buffer.append`.
    AudioAppend,
    /// `input_audio_buffer.commit`.
    AudioCommit,
    /// Any frame with this exact `type`.
    Frame(String),
}

impl Trigger {
    /// The frame type this trigger answers.
    #[must_use]
    pub fn frame_type(&self) -> &str {
        match self {
            Self::SessionUpdate => "session.update",
            Self::ItemCreate => "conversation.item.create",
            Self::ResponseCreate => "response.create",
            Self::ResponseCancel => "response.cancel",
            Self::AudioAppend => "input_audio_buffer.append",
            Self::AudioCommit => "input_audio_buffer.commit",
            Self::Frame(kind) => kind,
        }
    }

    /// Whether a frame of type `kind` fires this trigger.
    #[must_use]
    pub fn matches(&self, kind: &str) -> bool {
        self.frame_type() == kind
    }
}

/// Whether a step fires once or every time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Repeat {
    /// Fire once, then fall through to the next matching step — and, when there
    /// is none, to the mock's default behaviour.
    ///
    /// The default, because a script is normally a sequence of distinct turns.
    #[default]
    Once,
    /// Fire every time. A provider that always refuses, a transcript rule that
    /// answers every commit.
    Always,
}

/// Whether the mock fills in the ids an emitted event left out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Stamp {
    /// Fill in `event_id`, the response id and the item id when they are absent.
    #[default]
    Auto,
    /// Send exactly what the script wrote.
    ///
    /// The only way to script a `response.created` with no `response.id`, which
    /// is the case `via-realtime` had to correct upstream on — see
    /// `docs/deviations/phase-5-via-realtime.md`.
    Verbatim,
}

/// One event the mock sends back, and how long it waits first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Emission {
    /// How long to wait before sending it.
    ///
    /// Relative to the previous emission in the queue, so a list of
    /// `after: 20ms` emissions is a 50 Hz stream rather than a burst. Virtual
    /// time: under `start_paused` it costs nothing.
    #[serde(rename = "afterMs", default, with = "millis")]
    pub after: Duration,
    /// The event, as the provider would send it.
    pub event: Value,
    /// Whether the mock fills in absent ids.
    #[serde(default)]
    pub stamp: Stamp,
}

impl Emission {
    /// Send `event` as soon as the trigger fires.
    #[must_use]
    pub fn now(event: Value) -> Self {
        Self {
            after: Duration::ZERO,
            event,
            stamp: Stamp::Auto,
        }
    }

    /// Send `event` after a virtual `delay`.
    #[must_use]
    pub fn after(delay: Duration, event: Value) -> Self {
        Self {
            after: delay,
            event,
            stamp: Stamp::Auto,
        }
    }

    /// Send `event` exactly as written, filling nothing in.
    #[must_use]
    pub fn verbatim(event: Value) -> Self {
        Self {
            after: Duration::ZERO,
            event,
            stamp: Stamp::Verbatim,
        }
    }

    /// Delay this emission, builder style.
    #[must_use]
    pub fn delayed(mut self, delay: Duration) -> Self {
        self.after = delay;
        self
    }

    /// Stop the mock filling in ids for this emission, builder style.
    ///
    /// [`Stamp::Verbatim`] by another name, for an emission that already had a
    /// delay attached.
    #[must_use]
    pub fn unstamped(mut self) -> Self {
        self.stamp = Stamp::Verbatim;
        self
    }

    /// The event's `type`, or `""`.
    #[must_use]
    pub fn kind(&self) -> &str {
        self.event
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default()
    }
}

impl From<Value> for Emission {
    fn from(event: Value) -> Self {
        Self::now(event)
    }
}

/// `Duration` as whole milliseconds, which is how a fixture spells a delay.
mod millis {
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serializer};

    pub(super) fn serialize<S: Serializer>(
        value: &Duration,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_u64(u64::try_from(value.as_millis()).unwrap_or(u64::MAX))
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Duration, D::Error> {
        Ok(Duration::from_millis(u64::deserialize(deserializer)?))
    }
}

/// One rule: when this frame goes out, send these events back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptStep {
    /// The frame that fires it.
    pub on: Trigger,
    /// Whether it fires once or every time.
    #[serde(default)]
    pub repeat: Repeat,
    /// What to send back. Empty is meaningful: it silences the mock's default
    /// answer for that frame, which is how a response-start timeout is
    /// scripted.
    #[serde(default)]
    pub emit: Vec<Emission>,
}

impl ScriptStep {
    /// A step that fires once.
    #[must_use]
    pub fn once<I: IntoIterator<Item = Emission>>(on: Trigger, emit: I) -> Self {
        Self {
            on,
            repeat: Repeat::Once,
            emit: emit.into_iter().collect(),
        }
    }

    /// A step that fires every time.
    #[must_use]
    pub fn always<I: IntoIterator<Item = Emission>>(on: Trigger, emit: I) -> Self {
        Self {
            on,
            repeat: Repeat::Always,
            emit: emit.into_iter().collect(),
        }
    }
}

/// Which shape of session this script describes.
///
/// `docs/architecture.md` §2 makes `dictation` the one mode that mounts no model
/// at all, so it is a script *kind* rather than a flag: it changes what the
/// provider declares (no model, no voice, an empty model catalog) as well as
/// which session mode the script opens in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScriptKind {
    /// A model that converses: transcripts, speech, tool calls, responses.
    #[default]
    Conversation,
    /// A plain streaming ASR: transcripts and nothing else.
    ///
    /// Every response-creating call answers `skipped / no_model_turn` without
    /// touching the socket, so a `dictation` script never needs a
    /// [`Trigger::ResponseCreate`] step — and a step that has one will never
    /// fire.
    Dictation,
}

impl ScriptKind {
    /// The session mode this kind opens in.
    #[must_use]
    pub const fn session_mode(self) -> SessionMode {
        match self {
            // `agent` is `SessionMode`'s own default: the whole stack.
            Self::Conversation => SessionMode::Agent,
            Self::Dictation => SessionMode::Dictation,
        }
    }

    /// The provider this kind declares when a fixture names none.
    ///
    /// This is why the kind is not merely a [`SessionMode`]: a `dictation`
    /// script's provider *mounts no model*, and a fixture that says
    /// `"kind": "dictation"` and nothing else has to get one.
    #[must_use]
    pub fn default_provider(self) -> MockProviderSpec {
        match self {
            Self::Conversation => MockProviderSpec::default(),
            Self::Dictation => MockProviderSpec::dictation(),
        }
    }
}

/// A deterministic realtime provider, written down.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "ScriptWire", into = "ScriptWire")]
pub struct Script {
    /// Conversation or dictation.
    pub kind: ScriptKind,
    /// The session mode to open in, when it is not the one
    /// [`ScriptKind::session_mode`] implies.
    pub mode: Option<SessionMode>,
    /// What the provider declares about itself.
    pub provider: MockProviderSpec,
    /// What the mock sends the moment the transport opens.
    ///
    /// Defaults to one `session.created`, which is what starts the handshake.
    /// **An empty list is a provider that never speaks**, which is how a
    /// connect timeout is scripted.
    pub on_connect: Vec<Emission>,
    /// The rules, in the order they are scanned.
    pub steps: Vec<ScriptStep>,
}

/// The JSON form of a [`Script`].
///
/// It exists for one field. `provider` is `Option` on the wire and never in the
/// struct, because *absent* and *present* mean different things: a fixture that
/// says `"kind": "dictation"` and nothing else must get a provider that mounts
/// no model, and `#[serde(default)]` on a plain field would hand it a conversing
/// one. Every other field is a straight copy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScriptWire {
    #[serde(default)]
    kind: ScriptKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mode: Option<SessionMode>,
    #[serde(default)]
    provider: Option<MockProviderSpec>,
    #[serde(default = "Script::default_on_connect")]
    on_connect: Vec<Emission>,
    #[serde(default)]
    steps: Vec<ScriptStep>,
}

impl From<ScriptWire> for Script {
    fn from(wire: ScriptWire) -> Self {
        Self {
            provider: wire
                .provider
                .unwrap_or_else(|| wire.kind.default_provider()),
            kind: wire.kind,
            mode: wire.mode,
            on_connect: wire.on_connect,
            steps: wire.steps,
        }
    }
}

impl From<Script> for ScriptWire {
    fn from(script: Script) -> Self {
        Self {
            kind: script.kind,
            mode: script.mode,
            provider: Some(script.provider),
            on_connect: script.on_connect,
            steps: script.steps,
        }
    }
}

impl Default for Script {
    fn default() -> Self {
        Self {
            kind: ScriptKind::default(),
            mode: None,
            provider: MockProviderSpec::default(),
            on_connect: Self::default_on_connect(),
            steps: Vec::new(),
        }
    }
}

impl Script {
    /// The handshake opener every realtime service sends.
    fn default_on_connect() -> Vec<Emission> {
        vec![Emission::now(events::session_created())]
    }

    /// A conversing provider with no rules — every frame gets the default
    /// answer.
    #[must_use]
    pub fn conversation() -> Self {
        Self::default()
    }

    /// A streaming ASR with no rules.
    ///
    /// The provider declares no model and no voice, and the session opens in
    /// [`SessionMode::Dictation`].
    #[must_use]
    pub fn dictation() -> Self {
        Self {
            kind: ScriptKind::Dictation,
            provider: MockProviderSpec::dictation(),
            ..Self::default()
        }
    }

    /// The session mode this script opens in.
    #[must_use]
    pub fn session_mode(&self) -> SessionMode {
        self.mode.unwrap_or_else(|| self.kind.session_mode())
    }

    /// The rules.
    #[must_use]
    pub fn steps(&self) -> &[ScriptStep] {
        &self.steps
    }

    /// Replace the provider declaration.
    #[must_use]
    pub fn with_provider(mut self, provider: MockProviderSpec) -> Self {
        self.provider = provider;
        self
    }

    /// Edit the provider declaration in place, builder style.
    #[must_use]
    pub fn map_provider<F>(mut self, edit: F) -> Self
    where
        F: FnOnce(MockProviderSpec) -> MockProviderSpec,
    {
        self.provider = edit(self.provider);
        self
    }

    /// Open in a specific session mode rather than the kind's own.
    #[must_use]
    pub fn with_mode(mut self, mode: SessionMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Replace what the mock sends when the transport opens.
    #[must_use]
    pub fn with_on_connect<I: IntoIterator<Item = Emission>>(mut self, emit: I) -> Self {
        self.on_connect = emit.into_iter().collect();
        self
    }

    /// Send nothing at all when the transport opens.
    ///
    /// The session then never becomes usable and `open` reports
    /// `RealtimeError::ConnectTimeout` when its budget runs out.
    #[must_use]
    pub fn silent_on_connect(self) -> Self {
        self.with_on_connect(Vec::new())
    }

    /// Add a step that fires once.
    #[must_use]
    pub fn on<I: IntoIterator<Item = Emission>>(mut self, trigger: Trigger, emit: I) -> Self {
        self.steps.push(ScriptStep::once(trigger, emit));
        self
    }

    /// Add a step that fires every time.
    #[must_use]
    pub fn on_always<I: IntoIterator<Item = Emission>>(
        mut self,
        trigger: Trigger,
        emit: I,
    ) -> Self {
        self.steps.push(ScriptStep::always(trigger, emit));
        self
    }

    /// Add one model turn: the answer to the next `response.create`.
    #[must_use]
    pub fn turn<I: IntoIterator<Item = Emission>>(self, emit: I) -> Self {
        self.on(Trigger::ResponseCreate, emit)
    }

    /// Add several model turns, in order.
    #[must_use]
    pub fn turns<I, T>(mut self, turns: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoIterator<Item = Emission>,
    {
        for emit in turns {
            self = self.turn(emit);
        }
        self
    }

    /// Read a script from JSON text.
    ///
    /// # Errors
    ///
    /// [`MockError::ScriptInvalid`], carrying the deserializer's own message,
    /// which names the offending field and its line.
    pub fn from_json_str(json: &str) -> Result<Self, MockError> {
        serde_json::from_str(json).map_err(|error| MockError::ScriptInvalid {
            detail: error.to_string(),
        })
    }

    /// Read a script from a JSON fixture file.
    ///
    /// The point of the file form: a test in another crate keeps its script
    /// beside itself rather than inside a Rust literal.
    ///
    /// # Errors
    ///
    /// [`MockError::FixtureUnreadable`] when the file cannot be read,
    /// [`MockError::FixtureInvalid`] when it is not a script. Both name the
    /// path, because a fixture failure is otherwise indistinguishable from a
    /// test failure.
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, MockError> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|error| MockError::FixtureUnreadable {
            path: path.display().to_string(),
            detail: error.to_string(),
        })?;
        serde_json::from_str(&text).map_err(|error| MockError::FixtureInvalid {
            path: path.display().to_string(),
            detail: error.to_string(),
        })
    }

    /// The script as JSON text, ready to be written back out as a fixture.
    ///
    /// # Errors
    ///
    /// [`MockError::ScriptInvalid`] when the script holds a `Value` that cannot
    /// be serialized, which no `Value` can.
    pub fn to_json_string(&self) -> Result<String, MockError> {
        serde_json::to_string_pretty(self).map_err(|error| MockError::ScriptInvalid {
            detail: error.to_string(),
        })
    }
}

/// The wire sentences a provider sends, so a script can drive each
/// [`via_realtime::ErrorClass`] branch.
///
/// **Simulated third-party text, not VIA prose.** These are the phrases the two
/// shipped providers' `classifyError` corpora match on
/// (`providers/dashscope.mjs:16-29`, `providers/s2s.mjs:14-23`), reproduced so a
/// mock refusal is classified the same way a real one is. They deliberately do
/// **not** go through `via-i18n`: translating them would stop them classifying,
/// which is the same reason `RealtimeError::Transport` carries the transport's
/// own text (`docs/deviations/phase-5-via-realtime.md`).
pub mod messages {
    /// Classified `response_slot_busy` — `s2s.mjs:20`.
    pub const RESPONSE_SLOT_BUSY: &str =
        "Cannot create response while another response is in progress.";

    /// Classified `input_busy` — `dashscope.mjs:18`.
    pub const INPUT_BUSY: &str = "Cannot create response while user is speaking.";

    /// Classified `no_active_response` — both providers.
    pub const NO_ACTIVE_RESPONSE: &str = "Cannot cancel response, no active response found.";

    /// Classified `capacity_busy` — `s2s.mjs:15`.
    pub const CAPACITY_BUSY: &str =
        "All 1 session slots are in use. Disconnect an existing client first.";

    /// Classified `inactivity` — `realtime-errors.mjs:1-6`, the one closure that
    /// must never be shown to anybody.
    pub const INACTIVITY: &str =
        "Your session was closed because no response was generated for 180 seconds.";

    /// Classified `fatal` — `dashscope.mjs:21`.
    pub const FATAL: &str = "Invalid API-key provided.";
}

/// The events a provider sends, as values.
///
/// Every name here is one `via-realtime` or the Gateway already branches on;
/// nothing invents a wire name. Where a value is optional on the wire it is
/// omitted rather than sent as `null`, because the mock's job is to look like a
/// provider.
pub mod events {
    use serde_json::{Map, Value, json};

    /// `session.created` — the handshake opener.
    #[must_use]
    pub fn session_created() -> Value {
        json!({ "type": "session.created", "session": { "id": "sess_mock" } })
    }

    /// `session.updated` — the acknowledgement of a `session.update`.
    #[must_use]
    pub fn session_updated() -> Value {
        json!({ "type": "session.updated" })
    }

    /// `conversation.item.created` — an item receipt.
    #[must_use]
    pub fn item_created(item: Value) -> Value {
        json!({ "type": "conversation.item.created", "item": item })
    }

    /// `response.created` — a response has started.
    #[must_use]
    pub fn response_created() -> Value {
        json!({ "type": "response.created", "response": {} })
    }

    /// `response.done` with a status.
    ///
    /// [`COMPLETED`](super::COMPLETED) is success; the three failures are
    /// `via_realtime::FAILED_RESPONSE_STATUSES`.
    #[must_use]
    pub fn response_done(status: &str) -> Value {
        json!({ "type": "response.done", "response": { "status": status } })
    }

    /// `response.text.delta` — a chunk of the model's text.
    #[must_use]
    pub fn text_delta(delta: &str) -> Value {
        json!({ "type": "response.text.delta", "delta": delta })
    }

    /// `response.text.done` — the whole of the model's text.
    #[must_use]
    pub fn text_done(text: &str) -> Value {
        json!({ "type": "response.text.done", "text": text })
    }

    /// `response.audio_transcript.delta` — a chunk of what the model is saying.
    #[must_use]
    pub fn audio_transcript_delta(delta: &str) -> Value {
        json!({ "type": "response.audio_transcript.delta", "delta": delta })
    }

    /// `response.audio_transcript.done` — the whole of what the model said.
    #[must_use]
    pub fn audio_transcript_done(transcript: &str) -> Value {
        json!({ "type": "response.audio_transcript.done", "transcript": transcript })
    }

    /// `response.audio.delta` — a chunk of base64 PCM16 speech.
    ///
    /// [`CannedAudio::audio_deltas`](crate::CannedAudio::audio_deltas) builds a
    /// whole clip's worth.
    #[must_use]
    pub fn audio_delta(base64: &str) -> Value {
        json!({ "type": "response.audio.delta", "delta": base64 })
    }

    /// `response.audio.done` — the model finished speaking.
    #[must_use]
    pub fn audio_done() -> Value {
        json!({ "type": "response.audio.done" })
    }

    /// `response.function_call_arguments.done` — the model called a tool.
    ///
    /// `arguments` goes on the wire as a JSON-encoded **string**, never as a
    /// nested object — catalogued at `json-field` / *function call handling over
    /// the realtime protocol*, and the same rule
    /// `RealtimeProtocol::function_output_item` obeys in the other direction.
    #[must_use]
    pub fn function_call(call_id: &str, name: &str, arguments: &Value) -> Value {
        json!({
            "type": "response.function_call_arguments.done",
            "call_id": call_id,
            "name": name,
            // A `Value` always serializes; the empty-string arm is unreachable
            // and is preferred to an `expect()`.
            "arguments": serde_json::to_string(arguments).unwrap_or_default(),
        })
    }

    /// `error` carrying only a message.
    #[must_use]
    pub fn error(message: &str) -> Value {
        json!({ "type": "error", "error": { "message": message } })
    }

    /// `error` with the full three-field body a provider sends.
    ///
    /// `via_realtime::realtime_event_error_message` joins the non-empty deduped
    /// values of `code`, `type` and `message` with `": "`, so all three are
    /// observable in what the session reports.
    #[must_use]
    pub fn error_detailed(code: &str, kind: &str, message: &str) -> Value {
        let mut error = Map::new();
        error.insert("code".to_owned(), Value::String(code.to_owned()));
        error.insert("type".to_owned(), Value::String(kind.to_owned()));
        error.insert("message".to_owned(), Value::String(message.to_owned()));
        json!({ "type": "error", "error": Value::Object(error) })
    }

    /// `input_audio_buffer.speech_started` — the user started talking.
    #[must_use]
    pub fn speech_started() -> Value {
        json!({ "type": "input_audio_buffer.speech_started" })
    }

    /// `input_audio_buffer.speech_stopped` — the user stopped talking.
    #[must_use]
    pub fn speech_stopped() -> Value {
        json!({ "type": "input_audio_buffer.speech_stopped" })
    }

    /// `input_audio_buffer.committed` — the input buffer closed into an item.
    #[must_use]
    pub fn input_committed() -> Value {
        json!({ "type": "input_audio_buffer.committed" })
    }

    /// `conversation.item.input_audio_transcription.delta` — streaming ASR.
    #[must_use]
    pub fn input_transcript_delta(delta: &str) -> Value {
        json!({
            "type": "conversation.item.input_audio_transcription.delta",
            "delta": delta,
        })
    }

    /// `conversation.item.input_audio_transcription.completed` — the final
    /// transcript of one user turn.
    #[must_use]
    pub fn input_transcript_completed(transcript: &str) -> Value {
        json!({
            "type": "conversation.item.input_audio_transcription.completed",
            "transcript": transcript,
        })
    }

    /// `conversation.item.input_audio_transcription.failed` — nothing was heard.
    #[must_use]
    pub fn input_transcript_failed() -> Value {
        json!({ "type": "conversation.item.input_audio_transcription.failed" })
    }
}

/// Whole turns, so a script reads as behaviour rather than as event soup.
///
/// A scripted step **replaces** the mock's default answer, so a turn has to
/// carry its own `response.created` and `response.done`. Each helper here is
/// exactly that, correctly bracketed.
pub mod turns {
    use serde_json::Value;

    use super::{COMPLETED, Emission, events};
    use crate::audio::CannedAudio;

    /// One turn that answers in text and completes.
    #[must_use]
    pub fn say(text: &str) -> Vec<Emission> {
        vec![
            Emission::now(events::response_created()),
            Emission::now(events::text_delta(text)),
            Emission::now(events::text_done(text)),
            Emission::now(events::response_done(COMPLETED)),
        ]
    }

    /// One turn that speaks: transcript, audio and completion.
    ///
    /// The audio is real PCM16, so a `PlaybackCursor` fed from these deltas
    /// reports the clip's real duration and the Injection Gate's drain
    /// predicate has something to drain.
    #[must_use]
    pub fn speak(transcript: &str, audio: &CannedAudio) -> Vec<Emission> {
        let mut emissions = vec![
            Emission::now(events::response_created()),
            Emission::now(events::audio_transcript_delta(transcript)),
        ];
        emissions.extend(audio.audio_deltas().into_iter().map(Emission::now));
        emissions.push(Emission::now(events::audio_done()));
        emissions.push(Emission::now(events::audio_transcript_done(transcript)));
        emissions.push(Emission::now(events::response_done(COMPLETED)));
        emissions
    }

    /// One turn that calls a tool and completes.
    ///
    /// The caller answers with
    /// `RealtimeSession::send_function_output(call_id, …)`, whose
    /// `response.create` the *next* turn in the script answers.
    #[must_use]
    pub fn call_tool(call_id: &str, name: &str, arguments: &Value) -> Vec<Emission> {
        vec![
            Emission::now(events::response_created()),
            Emission::now(events::function_call(call_id, name, arguments)),
            Emission::now(events::response_done(COMPLETED)),
        ]
    }

    /// One turn that starts and then ends in a failing status.
    #[must_use]
    pub fn fail(status: &str) -> Vec<Emission> {
        vec![
            Emission::now(events::response_created()),
            Emission::now(events::response_done(status)),
        ]
    }

    /// One turn the provider refuses outright, before it ever starts.
    #[must_use]
    pub fn refuse(message: &str) -> Vec<Emission> {
        vec![Emission::now(events::error(message))]
    }

    /// One turn that starts and then goes silent.
    ///
    /// Nothing else arrives, so the session's output-inactivity watchdog is what
    /// ends it.
    #[must_use]
    pub fn stall() -> Vec<Emission> {
        vec![Emission::now(events::response_created())]
    }

    /// A turn that never starts at all.
    ///
    /// The empty list silences the mock's default answer, so the session's
    /// response-*start* watchdog is what ends it.
    #[must_use]
    pub fn never_start() -> Vec<Emission> {
        Vec::new()
    }
}

/// `{ key: value, ...object }` — the JavaScript spread, which puts the inserted
/// key first and leaves an existing one where it was.
pub(crate) fn with_leading(object: Value, key: &str, value: Value) -> Value {
    let mut fields = Map::new();
    fields.insert(key.to_owned(), value);
    if let Value::Object(existing) = object {
        for (name, held) in existing {
            fields.insert(name, held);
        }
    }
    Value::Object(fields)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use via_realtime::{FAILED_RESPONSE_STATUSES, RESPONSE_ACTIVITY_TYPES, is_completed_status};

    #[test]
    fn every_named_trigger_is_a_frame_via_realtime_actually_writes() {
        for trigger in [
            Trigger::SessionUpdate,
            Trigger::ItemCreate,
            Trigger::ResponseCreate,
            Trigger::ResponseCancel,
            Trigger::AudioAppend,
            Trigger::AudioCommit,
        ] {
            assert!(trigger.matches(trigger.frame_type()));
            assert!(!trigger.matches("something.else"));
        }
        assert_eq!(
            Trigger::Frame("x.y".into()).frame_type(),
            "x.y",
            "the escape hatch is the string itself"
        );
    }

    #[test]
    fn a_trigger_serializes_as_a_name_and_the_escape_hatch_as_an_object() {
        assert_eq!(
            serde_json::to_value(Trigger::ResponseCreate).expect("serialize"),
            json!("responseCreate")
        );
        assert_eq!(
            serde_json::to_value(Trigger::Frame("a.b".into())).expect("serialize"),
            json!({ "frame": "a.b" })
        );
        assert_eq!(
            serde_json::from_value::<Trigger>(json!("audioCommit")).expect("parse"),
            Trigger::AudioCommit
        );
    }

    #[test]
    fn the_defaults_are_once_and_auto() {
        assert_eq!(Repeat::default(), Repeat::Once);
        assert_eq!(Stamp::default(), Stamp::Auto);
        assert_eq!(
            ScriptStep::once(Trigger::ItemCreate, []).repeat,
            Repeat::Once
        );
        assert_eq!(
            ScriptStep::always(Trigger::ItemCreate, []).repeat,
            Repeat::Always
        );
    }

    #[test]
    fn an_emission_carries_its_delay_and_its_stamping_choice() {
        let event = json!({ "type": "response.created" });
        assert_eq!(Emission::now(event.clone()).after, Duration::ZERO);
        assert_eq!(
            Emission::after(Duration::from_millis(40), event.clone()).after,
            Duration::from_millis(40)
        );
        assert_eq!(Emission::verbatim(event.clone()).stamp, Stamp::Verbatim);
        assert_eq!(
            Emission::now(event.clone())
                .delayed(Duration::from_millis(7))
                .after,
            Duration::from_millis(7)
        );
        assert_eq!(
            Emission::now(event.clone()).unstamped().stamp,
            Stamp::Verbatim
        );
        assert_eq!(Emission::from(event).kind(), "response.created");
        assert_eq!(Emission::now(json!({})).kind(), "");
    }

    #[test]
    fn a_delay_round_trips_as_whole_milliseconds() {
        let emission = Emission::after(
            Duration::from_millis(1_234),
            json!({ "type": "response.done" }),
        );
        let json = serde_json::to_value(&emission).expect("serialize");
        assert_eq!(json["afterMs"], json!(1_234));
        assert_eq!(
            serde_json::from_value::<Emission>(json).expect("parse"),
            emission
        );
    }

    #[test]
    fn an_emission_with_no_delay_and_no_stamp_key_still_parses() {
        let emission: Emission =
            serde_json::from_value(json!({ "event": { "type": "error" } })).expect("parse");
        assert_eq!(emission.after, Duration::ZERO);
        assert_eq!(emission.stamp, Stamp::Auto);
    }

    #[test]
    fn a_default_script_opens_the_handshake_and_has_no_rules() {
        let script = Script::default();
        assert_eq!(script.kind, ScriptKind::Conversation);
        assert_eq!(script.session_mode(), SessionMode::Agent);
        assert!(script.steps().is_empty());
        assert_eq!(script.on_connect.len(), 1);
        assert_eq!(script.on_connect[0].kind(), "session.created");
    }

    #[test]
    fn a_dictation_script_opens_in_dictation_mode_with_no_model() {
        let script = Script::dictation();
        assert_eq!(script.kind, ScriptKind::Dictation);
        assert_eq!(script.session_mode(), SessionMode::Dictation);
        assert_eq!(script.provider.model, None);
        assert_eq!(script.provider.voice, None);
    }

    #[test]
    fn an_explicit_mode_overrides_the_kinds_own() {
        let script = Script::conversation().with_mode(SessionMode::Direct);
        assert_eq!(script.session_mode(), SessionMode::Direct);
        // …and the kind is untouched, so the provider still declares a model.
        assert_eq!(script.kind, ScriptKind::Conversation);
    }

    #[test]
    fn turns_are_response_create_steps_in_order() {
        let script = Script::conversation().turns([turns::say("one"), turns::say("two")]);
        assert_eq!(script.steps().len(), 2);
        for step in script.steps() {
            assert_eq!(step.on, Trigger::ResponseCreate);
            assert_eq!(step.repeat, Repeat::Once);
        }
        assert_eq!(script.steps()[0].emit[1].event["delta"], json!("one"));
        assert_eq!(script.steps()[1].emit[1].event["delta"], json!("two"));
    }

    #[test]
    fn silent_on_connect_is_a_provider_that_never_speaks() {
        assert!(
            Script::conversation()
                .silent_on_connect()
                .on_connect
                .is_empty()
        );
    }

    #[test]
    fn a_script_round_trips_through_json_unchanged() {
        let script = Script::conversation()
            .turn(turns::say("hello"))
            .on_always(
                Trigger::AudioCommit,
                [Emission::now(events::input_transcript_completed("hi"))],
            )
            .on(
                Trigger::Frame("custom.frame".into()),
                [Emission::after(
                    Duration::from_millis(15),
                    events::error("boom"),
                )],
            );
        let text = script.to_json_string().expect("serialize");
        assert_eq!(Script::from_json_str(&text).expect("parse"), script);
    }

    #[test]
    fn json_that_is_not_a_script_names_the_problem() {
        let error = Script::from_json_str("{ not json").expect_err("refused");
        assert_eq!(error.code(), "VIA_MOCK_SCRIPT_INVALID");
        let error = Script::from_json_str(r#"{ "kind": "singing" }"#).expect_err("refused");
        assert_eq!(error.code(), "VIA_MOCK_SCRIPT_INVALID");
    }

    #[test]
    fn a_minimal_fixture_takes_every_default() {
        let script = Script::from_json_str("{}").expect("parse");
        assert_eq!(script, Script::default());
    }

    #[test]
    fn an_explicitly_empty_on_connect_is_kept_rather_than_defaulted() {
        let script = Script::from_json_str(r#"{ "onConnect": [] }"#).expect("parse");
        assert!(script.on_connect.is_empty());
    }

    #[test]
    fn every_turn_helper_brackets_its_own_response() {
        for emissions in [
            turns::say("x"),
            turns::call_tool("call_1", "tool", &json!({})),
            turns::fail("failed"),
        ] {
            assert_eq!(
                emissions.first().map(Emission::kind),
                Some("response.created")
            );
            assert_eq!(emissions.last().map(Emission::kind), Some("response.done"));
        }
        assert_eq!(turns::stall().len(), 1);
        assert!(turns::never_start().is_empty());
        assert_eq!(turns::refuse("no")[0].kind(), "error");
    }

    #[test]
    fn the_two_statuses_this_module_names_are_the_catalogued_ones() {
        assert!(is_completed_status(Some(COMPLETED)));
        assert!(FAILED_RESPONSE_STATUSES.contains(&CANCELLED));
    }

    #[test]
    fn every_response_event_this_module_builds_is_in_the_activity_table() {
        for event in [
            events::response_created(),
            events::response_done(COMPLETED),
            events::text_delta("a"),
            events::text_done("a"),
            events::audio_transcript_delta("a"),
            events::audio_transcript_done("a"),
            events::audio_delta("a"),
            events::audio_done(),
            events::function_call("c", "n", &json!({})),
        ] {
            let kind = event["type"].as_str().unwrap_or_default();
            assert!(
                RESPONSE_ACTIVITY_TYPES.contains(&kind),
                "{kind} is not response activity"
            );
        }
    }

    #[test]
    fn a_tool_call_encodes_its_arguments_as_a_string() {
        let event = events::function_call("call_1", "spawn_thinking", &json!({ "n": 1 }));
        assert_eq!(event["arguments"], json!(r#"{"n":1}"#));
        assert!(event["arguments"].is_string());
        assert_eq!(event["call_id"], json!("call_1"));
        assert_eq!(event["name"], json!("spawn_thinking"));
    }

    #[test]
    fn a_detailed_error_keeps_the_three_fields_in_composition_order() {
        let event = events::error_detailed("Quota", "insufficient_quota", "exhausted");
        let keys: Vec<&str> = event["error"]
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["code", "type", "message"]);
    }

    #[test]
    fn the_spread_puts_the_new_key_first_and_leaves_an_existing_one_alone() {
        let spread = with_leading(json!({ "type": "message" }), "id", json!("item_1"));
        let keys: Vec<&str> = spread
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["id", "type"]);

        let kept = with_leading(
            json!({ "id": "mine", "type": "message" }),
            "id",
            json!("new"),
        );
        assert_eq!(kept["id"], json!("mine"));
        assert_eq!(
            with_leading(json!(null), "id", json!("x")),
            json!({ "id": "x" })
        );
    }
}
