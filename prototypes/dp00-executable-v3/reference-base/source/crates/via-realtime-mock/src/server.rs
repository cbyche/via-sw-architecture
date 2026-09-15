//! The script server: one owning task standing in for a realtime service.
//!
//! `docs/architecture.md` §11 — *"Each gets an owning task, not a mutex …  State
//! held by value in one task, `enum Command { … reply: oneshot::Sender<R> }` over
//! a bounded `mpsc`."* Everything the mock knows lives here by value: the script
//! cursor, the id counters, the emission queue and the transcript. A
//! [`MockHandle`](crate::MockHandle) asks; it never reaches in.
//!
//! That is not ceremony. The transcript is read *while* the session under test
//! is still writing to it, and a mutex would give a test a snapshot torn across
//! a frame boundary — which is the exact class of flake a deterministic mock
//! exists to remove.
//!
//! # The loop
//!
//! Three things can happen, and they are polled `biased` in this order so the
//! outcome is a function of the script rather than of the scheduler:
//!
//! 1. **A queued emission is due** — send it.
//! 2. **A handle asked something** — answer it.
//! 3. **The session wrote a frame** — record it, match it against the script,
//!    and queue what it answers with.
//!
//! Emissions are one FIFO with absolute deadlines, so a delayed event never
//! overtakes an earlier one and a frame that arrives mid-turn is answered
//! *after* the turn it interrupted, exactly as a real service would.
//!
//! # Capabilities are behaviour here, not just a declaration
//!
//! | Flag | What this server does |
//! | --- | --- |
//! | `acknowledges_session_update` | withholds `session.updated` when false |
//! | `single_response_slot` | refuses a `response.create` that races an active response |
//! | `response_metadata_correlation` | echoes the correlation id on `response.created` / `response.done` |
//! | `conversation_item_id_echo` | replaces the client's item id when false |
//! | `per_response_instructions` | declared only — no session behaviour branches on it |

use std::collections::VecDeque;
use std::time::Duration;

use futures::StreamExt as _;
use futures::channel::mpsc as futures_mpsc;
use serde_json::{Map, Value};
use tokio::sync::{mpsc, oneshot};
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message;
use via_realtime::{
    ProviderCapabilities, RESPONSE_ACTIVITY_TYPES, RESPONSE_CORRELATION_KEY, Transport,
    realtime_response_id,
};

use crate::audio::decode_audio;
use crate::error::MockError;
use crate::script::{
    CANCELLED, COMPLETED, Emission, Repeat, Script, Stamp, events, messages, with_leading,
};

/// The id prefix every response the mock mints carries.
pub const RESPONSE_ID_PREFIX: &str = "resp_";

/// The id prefix every conversation item the mock mints carries.
///
/// Deliberately unlike the dialects' own `item_` / `msg_` / `fco_` namespaces:
/// when [`ProviderCapabilities::conversation_item_id_echo`] is false the mock
/// *replaces* the client's id, and a replacement that is visibly the mock's own
/// is what makes the substitution legible in a failing test.
pub const ITEM_ID_PREFIX: &str = "item_mock_";

/// The id prefix every event the mock sends carries.
pub const EVENT_ID_PREFIX: &str = "event_mock_";

/// Depth of the command channel between a handle and the server.
pub const COMMAND_CAPACITY: usize = 32;

/// The events that name the *input* item they belong to.
///
/// Sourced from what the Gateway reads `item_id` off
/// (`server/src/voice/realtime-gateway.mjs:1041-1112`,
/// `server/src/voice/input-transcript.mjs:1-3`). An emitted event of one of
/// these types with no `item_id` gets the input turn's.
pub const ITEM_SCOPED_EVENTS: [&str; 8] = [
    "input_audio_buffer.committed",
    "input_audio_buffer.speech_started",
    "input_audio_buffer.speech_stopped",
    "conversation.item.input_audio_transcription.delta",
    "conversation.item.input_audio_transcription.text",
    "conversation.item.input_audio_transcription.completed",
    "conversation.item.input_audio_transcription.failed",
    "conversation.item.ambient_audio_transcription.completed",
];

/// One tool result the session returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionOutput {
    /// The call it answers.
    pub call_id: String,
    /// The result, decoded from the JSON string the wire carries it as.
    ///
    /// `output` is always a JSON-encoded **string** on the wire — catalogued at
    /// `json-field` / *function call handling over the realtime protocol* — so a
    /// test that compared it as an object would be asserting against a shape
    /// that never travels. Decoding it here is what lets a test compare values
    /// without restating the encoding.
    pub output: Value,
}

/// Everything that crossed the transport, in order.
///
/// A snapshot: it is a copy taken inside the server task, so it can never be
/// torn across a frame.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Transcript {
    outbound: Vec<Value>,
    inbound: Vec<Value>,
}

impl Transcript {
    /// Every frame the session wrote, in order, as it went on the wire —
    /// envelope, `event_id` and all.
    #[must_use]
    pub fn outbound(&self) -> &[Value] {
        &self.outbound
    }

    /// Every event the mock sent, in order.
    #[must_use]
    pub fn inbound(&self) -> &[Value] {
        &self.inbound
    }

    /// The `type` of every frame the session wrote.
    #[must_use]
    pub fn outbound_kinds(&self) -> Vec<&str> {
        self.outbound.iter().map(kind_of).collect()
    }

    /// The `type` of every event the mock sent.
    #[must_use]
    pub fn inbound_kinds(&self) -> Vec<&str> {
        self.inbound.iter().map(kind_of).collect()
    }

    /// Every frame the session wrote of one type.
    #[must_use]
    pub fn frames_of(&self, kind: &str) -> Vec<&Value> {
        self.outbound
            .iter()
            .filter(|frame| kind_of(frame) == kind)
            .collect()
    }

    /// The first frame the session wrote of one type.
    #[must_use]
    pub fn first_of(&self, kind: &str) -> Option<&Value> {
        self.outbound.iter().find(|frame| kind_of(frame) == kind)
    }

    /// How many frames of one type the session wrote.
    #[must_use]
    pub fn count_of(&self, kind: &str) -> usize {
        self.frames_of(kind).len()
    }

    /// The `session` payload of every `session.update`.
    #[must_use]
    pub fn session_updates(&self) -> Vec<&Value> {
        self.frames_of("session.update")
            .into_iter()
            .filter_map(|frame| frame.get("session"))
            .collect()
    }

    /// The `item` of every `conversation.item.create`.
    #[must_use]
    pub fn items(&self) -> Vec<&Value> {
        self.frames_of("conversation.item.create")
            .into_iter()
            .filter_map(|frame| frame.get("item"))
            .collect()
    }

    /// The `response` body of every `response.create`, `None` where there was
    /// none.
    ///
    /// The distinction is contract:
    /// [`RealtimeProtocol::response_create`](via_realtime::RealtimeProtocol::response_create)
    /// omits the key entirely rather than sending an empty object.
    #[must_use]
    pub fn response_creates(&self) -> Vec<Option<&Value>> {
        self.frames_of("response.create")
            .into_iter()
            .map(|frame| frame.get("response"))
            .collect()
    }

    /// Every tool result the session returned, decoded.
    #[must_use]
    pub fn function_outputs(&self) -> Vec<FunctionOutput> {
        self.items()
            .into_iter()
            .filter(|item| item.get("type").and_then(Value::as_str) == Some("function_call_output"))
            .map(|item| FunctionOutput {
                call_id: item
                    .get("call_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                output: item
                    .get("output")
                    .and_then(Value::as_str)
                    .and_then(|output| serde_json::from_str(output).ok())
                    .unwrap_or(Value::Null),
            })
            .collect()
    }

    /// The base64 payload of every `input_audio_buffer.append`.
    #[must_use]
    pub fn input_audio_base64(&self) -> Vec<&str> {
        self.frames_of("input_audio_buffer.append")
            .into_iter()
            .filter_map(|frame| frame.get("audio").and_then(Value::as_str))
            .collect()
    }

    /// Every appended audio frame, decoded and concatenated.
    ///
    /// # Errors
    ///
    /// [`MockError::NotBase64`] when the session wrote a frame that is not
    /// base64 — which is a bug in the caller under test, not in the fixture.
    pub fn input_audio_pcm16(&self) -> Result<Vec<i16>, MockError> {
        let mut bytes = Vec::new();
        for chunk in self.input_audio_base64() {
            bytes.extend(decode_audio(chunk)?);
        }
        Ok(via_audio::pcm16le_to_i16(&bytes))
    }
}

/// The `type` of a frame, or `""`.
fn kind_of(frame: &Value) -> &str {
    frame
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
}

/// What a handle asks the server to do.
pub(crate) enum MockCommand {
    /// Copy out everything that has crossed the transport.
    Snapshot(oneshot::Sender<Transcript>),
    /// Send these events, as though a step had matched.
    Emit {
        emissions: Vec<Emission>,
        reply: oneshot::Sender<()>,
    },
}

/// One event, waiting for its deadline.
struct Queued {
    at: Instant,
    event: Value,
}

/// What one trigger knows about the frame that fired it.
#[derive(Debug, Default)]
struct Context {
    /// The response an emitted event should bind to.
    response_id: Option<String>,
    /// The conversation item an emitted event should name.
    item_id: Option<String>,
    /// The correlation id the client stamped on this `response.create`.
    correlation: Option<String>,
}

/// The mock service.
pub(crate) struct ScriptServer {
    script: Script,
    capabilities: ProviderCapabilities,
    correlates: bool,
    to_session: futures_mpsc::UnboundedSender<Result<Message, String>>,
    transcript: Transcript,
    /// Per-step exhaustion, parallel to `script.steps`.
    used: Vec<bool>,
    responses: u64,
    items: u64,
    events: u64,
    /// The response the service considers in flight, tracked at *emit* time —
    /// a response exists once the provider has said so, not once it decided to.
    active_response: Option<String>,
    /// The input turn transcripts and commits belong to.
    input_item: Option<String>,
    /// The deadline of the last queued emission, so a delay is relative to the
    /// emission before it rather than to an arbitrary "now".
    last_deadline: Option<Instant>,
}

impl ScriptServer {
    fn new(
        script: Script,
        correlates: bool,
        to_session: futures_mpsc::UnboundedSender<Result<Message, String>>,
    ) -> Self {
        Self {
            capabilities: script.provider.capabilities,
            used: vec![false; script.steps.len()],
            script,
            correlates,
            to_session,
            transcript: Transcript::default(),
            responses: 0,
            items: 0,
            events: 0,
            active_response: None,
            input_item: None,
            last_deadline: None,
        }
    }

    // ── inbound: what the session wrote ─────────────────────────────────────

    /// Record a frame and queue whatever it is answered with.
    fn on_frame(&mut self, frame: Value) -> Vec<Queued> {
        let kind = kind_of(&frame).to_owned();
        self.transcript.outbound.push(frame.clone());

        // A provider with one response slot refuses a racing `response.create`
        // *before* the script is consulted: it is the service's own state, not
        // script content, and leaving the step unconsumed is what lets the
        // session's busy-retry ladder replay into it.
        if kind == "response.create"
            && self.capabilities.single_response_slot
            && self.active_response.is_some()
        {
            let refusal = self.stamp(
                &Emission::now(events::error(messages::RESPONSE_SLOT_BUSY)),
                &Context::default(),
            );
            return self.enqueue(vec![(Duration::ZERO, refusal)]);
        }

        let context = self.context_for(&kind, &frame);
        let emissions = match self.match_step(&kind) {
            Some(emissions) => emissions,
            None => self.default_emissions(&kind, &frame, &context),
        };
        let planned: Vec<(Duration, Value)> = emissions
            .iter()
            .map(|emission| (emission.after, self.stamp(emission, &context)))
            .collect();
        if kind == "input_audio_buffer.commit" {
            // The turn is closed; the next append opens a new one.
            self.input_item = None;
        }
        self.enqueue(planned)
    }

    /// The first unexhausted step that matches, marking it used when it fires
    /// once.
    fn match_step(&mut self, kind: &str) -> Option<Vec<Emission>> {
        let index = self
            .script
            .steps
            .iter()
            .enumerate()
            .find(|(index, step)| !self.used[*index] && step.on.matches(kind))
            .map(|(index, _)| index)?;
        if self.script.steps[index].repeat == Repeat::Once {
            self.used[index] = true;
        }
        Some(self.script.steps[index].emit.clone())
    }

    /// What a provider does when the script says nothing.
    fn default_emissions(&self, kind: &str, frame: &Value, context: &Context) -> Vec<Emission> {
        match kind {
            "session.update" if self.capabilities.acknowledges_session_update => {
                vec![Emission::now(events::session_updated())]
            }
            "conversation.item.create" => {
                vec![Emission::now(events::item_created(
                    self.echoed_item(frame, context),
                ))]
            }
            "response.create" => vec![
                Emission::now(events::response_created()),
                Emission::now(events::response_done(COMPLETED)),
            ],
            // A provider answers a cancel it can act on and says nothing about
            // one it cannot; the "no active response" refusal is scriptable
            // rather than default, because the session only writes
            // `response.cancel` when it believes something is running.
            "response.cancel" if self.active_response.is_some() => {
                vec![Emission::now(events::response_done(CANCELLED))]
            }
            "input_audio_buffer.commit" => vec![Emission::now(events::input_committed())],
            _ => Vec::new(),
        }
    }

    /// The item a `conversation.item.created` echoes back.
    ///
    /// The whole item as the client sent it, with the id the service assigned —
    /// which is the client's own when
    /// [`ProviderCapabilities::conversation_item_id_echo`] is true, and the
    /// mock's when it is not.
    fn echoed_item(&self, frame: &Value, context: &Context) -> Value {
        let item = frame.get("item").cloned().unwrap_or(Value::Null);
        let stripped = match item {
            Value::Object(mut fields) => {
                fields.shift_remove("id");
                Value::Object(fields)
            }
            other => other,
        };
        let with_status = match stripped {
            Value::Object(mut fields) => {
                fields
                    .entry("status".to_owned())
                    .or_insert_with(|| Value::String(COMPLETED.to_owned()));
                Value::Object(fields)
            }
            other => other,
        };
        match &context.item_id {
            Some(id) => with_leading(with_status, "id", Value::String(id.clone())),
            None => with_status,
        }
    }

    /// What this trigger knows, resolved once so every emission it queues shares
    /// it.
    fn context_for(&mut self, kind: &str, frame: &Value) -> Context {
        match kind {
            "response.create" => {
                self.responses += 1;
                Context {
                    response_id: Some(format!("{RESPONSE_ID_PREFIX}{}", self.responses)),
                    item_id: None,
                    correlation: frame
                        .get("response")
                        .and_then(|response| response.get("metadata"))
                        .and_then(|metadata| metadata.get(RESPONSE_CORRELATION_KEY))
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                }
            }
            "conversation.item.create" => {
                let sent = frame
                    .get("item")
                    .and_then(|item| item.get("id"))
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let id = if self.capabilities.conversation_item_id_echo && !sent.is_empty() {
                    sent.to_owned()
                } else {
                    self.next_item_id()
                };
                Context {
                    response_id: self.active_response.clone(),
                    item_id: Some(id),
                    correlation: None,
                }
            }
            "input_audio_buffer.append" | "input_audio_buffer.commit" => Context {
                response_id: self.active_response.clone(),
                item_id: Some(self.ensure_input_item()),
                correlation: None,
            },
            _ => Context {
                response_id: self.active_response.clone(),
                item_id: self.input_item.clone(),
                correlation: None,
            },
        }
    }

    fn next_item_id(&mut self) -> String {
        self.items += 1;
        format!("{ITEM_ID_PREFIX}{}", self.items)
    }

    fn ensure_input_item(&mut self) -> String {
        match &self.input_item {
            Some(id) => id.clone(),
            None => {
                let id = self.next_item_id();
                self.input_item = Some(id.clone());
                id
            }
        }
    }

    // ── stamping ────────────────────────────────────────────────────────────

    /// Fill in the ids the script left out.
    ///
    /// See the crate docs' rule 4. [`Stamp::Verbatim`] skips every step of it,
    /// which is the only way to script an event with a *deliberately* missing
    /// id.
    fn stamp(&mut self, emission: &Emission, context: &Context) -> Value {
        let mut event = emission.event.clone();
        if emission.stamp == Stamp::Verbatim {
            return event;
        }
        let kind = kind_of(&event).to_owned();

        if !has_field(&event, "event_id") {
            self.events += 1;
            let id = format!("{EVENT_ID_PREFIX}{}", self.events);
            event = with_leading(event, "event_id", Value::String(id));
        }

        if let Some(item_id) = &context.item_id {
            if kind == "conversation.item.created" {
                set_nested_if_absent(&mut event, "item", "id", item_id);
            } else if ITEM_SCOPED_EVENTS.contains(&kind.as_str()) {
                set_if_absent(&mut event, "item_id", item_id);
            }
        }

        if RESPONSE_ACTIVITY_TYPES.contains(&kind.as_str()) {
            let lifecycle = kind == "response.created" || kind == "response.done";
            if let Some(response_id) = &context.response_id
                && realtime_response_id(&event).is_empty()
            {
                if lifecycle {
                    set_nested_if_absent(&mut event, "response", "id", response_id);
                } else {
                    set_if_absent(&mut event, "response_id", response_id);
                }
            }
            // Only a provider that declares metadata correlation echoes it, and
            // only a dialect that carries one has anything to echo.
            if lifecycle
                && self.correlates
                && self.capabilities.response_metadata_correlation
                && let Some(correlation) = &context.correlation
            {
                set_correlation(&mut event, correlation);
            }
        }
        event
    }

    // ── outbound: what the mock sends ───────────────────────────────────────

    /// Turn planned emissions into deadlines, in one FIFO.
    fn enqueue(&mut self, planned: Vec<(Duration, Value)>) -> Vec<Queued> {
        let mut queued = Vec::with_capacity(planned.len());
        for (after, event) in planned {
            let now = Instant::now();
            let base = match self.last_deadline {
                Some(previous) if previous > now => previous,
                _ => now,
            };
            let at = base + after;
            self.last_deadline = Some(at);
            queued.push(Queued { at, event });
        }
        queued
    }

    /// Send one event to the session.
    ///
    /// Answers whether the transport is still there.
    fn emit(&mut self, event: Value) -> bool {
        let response_id = realtime_response_id(&event).to_owned();
        match kind_of(&event) {
            "response.created" if !response_id.is_empty() => {
                self.active_response = Some(response_id);
            }
            "response.done" => {
                if response_id.is_empty()
                    || self.active_response.as_deref() == Some(response_id.as_str())
                {
                    self.active_response = None;
                }
            }
            _ => {}
        }
        self.transcript.inbound.push(event.clone());
        // A `Value` always serializes; the empty-string arm is unreachable and
        // is preferred to an `expect()`.
        let text = serde_json::to_string(&event).unwrap_or_default();
        self.to_session
            .unbounded_send(Ok(Message::Text(text.into())))
            .is_ok()
    }

    /// Answer one handle command.
    fn handle(&mut self, command: MockCommand) -> Vec<Queued> {
        match command {
            MockCommand::Snapshot(reply) => {
                let _ = reply.send(self.transcript.clone());
                Vec::new()
            }
            MockCommand::Emit { emissions, reply } => {
                let context = Context {
                    response_id: self.active_response.clone(),
                    item_id: self.input_item.clone(),
                    correlation: None,
                };
                let planned: Vec<(Duration, Value)> = emissions
                    .iter()
                    .map(|emission| (emission.after, self.stamp(emission, &context)))
                    .collect();
                let queued = self.enqueue(planned);
                let _ = reply.send(());
                queued
            }
        }
    }
}

/// Whether a top-level field is present.
fn has_field(event: &Value, key: &str) -> bool {
    event.get(key).is_some()
}

/// Set a top-level string field when it is absent.
fn set_if_absent(event: &mut Value, key: &str, value: &str) {
    if let Some(fields) = event.as_object_mut() {
        fields
            .entry(key.to_owned())
            .or_insert_with(|| Value::String(value.to_owned()));
    }
}

/// Set `event[outer][key]` when it is absent, creating `outer` if it has to.
fn set_nested_if_absent(event: &mut Value, outer: &str, key: &str, value: &str) {
    let Some(fields) = event.as_object_mut() else {
        return;
    };
    let nested = fields
        .entry(outer.to_owned())
        .or_insert_with(|| Value::Object(Map::new()));
    if !nested.is_object() {
        *nested = Value::Object(Map::new());
    }
    if let Some(nested) = nested.as_object_mut() {
        nested
            .entry(key.to_owned())
            .or_insert_with(|| Value::String(value.to_owned()));
    }
}

/// Echo the correlation id back where the GA dialect reads it.
fn set_correlation(event: &mut Value, correlation: &str) {
    let Some(fields) = event.as_object_mut() else {
        return;
    };
    let response = fields
        .entry("response".to_owned())
        .or_insert_with(|| Value::Object(Map::new()));
    if !response.is_object() {
        *response = Value::Object(Map::new());
    }
    if let Some(response) = response.as_object_mut() {
        let metadata = response
            .entry("metadata".to_owned())
            .or_insert_with(|| Value::Object(Map::new()));
        if !metadata.is_object() {
            *metadata = Value::Object(Map::new());
        }
        if let Some(metadata) = metadata.as_object_mut() {
            metadata
                .entry(RESPONSE_CORRELATION_KEY.to_owned())
                .or_insert_with(|| Value::String(correlation.to_owned()));
        }
    }
}

/// Build the in-process transport and start the script server on it.
///
/// Returns the [`Transport`] to hand to
/// [`RealtimeSession::open`](via_realtime::RealtimeSession::open) and the
/// command channel a [`MockHandle`](crate::MockHandle) talks over.
pub(crate) fn spawn(script: Script, correlates: bool) -> (Transport, mpsc::Sender<MockCommand>) {
    let (to_session, session_reads) = futures_mpsc::unbounded::<Result<Message, String>>();
    let (session_writes, outbound) = futures_mpsc::unbounded::<Message>();
    let (commands, command_receiver) = mpsc::channel(COMMAND_CAPACITY);

    let server = ScriptServer::new(script, correlates, to_session);
    tokio::spawn(run(server, outbound, command_receiver));

    (Transport::new(session_writes, session_reads), commands)
}

/// The owning task.
async fn run(
    mut server: ScriptServer,
    mut outbound: futures_mpsc::UnboundedReceiver<Message>,
    mut commands: mpsc::Receiver<MockCommand>,
) {
    let mut pending: VecDeque<Queued> = VecDeque::new();
    let mut transport_open = true;
    let mut commands_open = true;

    // Upstream's `ws.on('open')`: whatever the service says first, it says
    // before it has been asked anything. There is no trigger and therefore no
    // context, so an opening emission that wants to name a response has to spell
    // the id out and take `Stamp::Verbatim`.
    let opening = server.script.on_connect.clone();
    let planned: Vec<(Duration, Value)> = opening
        .iter()
        .map(|emission| (emission.after, server.stamp(emission, &Context::default())))
        .collect();
    pending.extend(server.enqueue(planned));

    while transport_open || commands_open {
        let due = pending.front().map(|queued| queued.at);
        tokio::select! {
            biased;

            () = sleep_until(due), if due.is_some() => {
                if let Some(queued) = pending.pop_front()
                    && !server.emit(queued.event)
                {
                    transport_open = false;
                }
            }

            command = commands.recv(), if commands_open => {
                match command {
                    Some(command) => pending.extend(server.handle(command)),
                    None => commands_open = false,
                }
            }

            frame = outbound.next(), if transport_open => {
                match frame.as_ref().map(decode_frame) {
                    Some(Frame::Json(value)) => pending.extend(server.on_frame(value)),
                    Some(Frame::Ignored) => {}
                    Some(Frame::Closed) | None => transport_open = false,
                }
            }
        }
    }
}

/// Wait until `due`, or forever when there is nothing queued.
///
/// The `forever` arm is never polled — the caller guards the branch with
/// `if due.is_some()` — and exists so the function is total.
async fn sleep_until(due: Option<Instant>) {
    match due {
        Some(at) => tokio::time::sleep_until(at).await,
        None => std::future::pending().await,
    }
}

/// What one WebSocket message means to the server.
enum Frame {
    Json(Value),
    Ignored,
    Closed,
}

/// Decode a frame the way [`via_realtime`]'s own reader does: a frame that is
/// not JSON is dropped without comment rather than taken as a failure.
fn decode_frame(message: &Message) -> Frame {
    match message {
        Message::Text(text) => serde_json::from_str(text).map_or(Frame::Ignored, Frame::Json),
        Message::Binary(bytes) => serde_json::from_slice(bytes).map_or(Frame::Ignored, Frame::Json),
        Message::Close(_) => Frame::Closed,
        Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => Frame::Ignored,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    fn server(script: Script) -> ScriptServer {
        let correlates = script.provider.dialect.correlates();
        let (to_session, _reads) = futures_mpsc::unbounded();
        // The receiver is dropped: these tests exercise planning and stamping,
        // which happen before anything is sent.
        ScriptServer::new(script, correlates, to_session)
    }

    fn stamped(server: &mut ScriptServer, event: Value, context: &Context) -> Value {
        server.stamp(&Emission::now(event), context)
    }

    #[test]
    fn every_stamped_event_gets_a_deterministic_event_id() {
        let mut server = server(Script::conversation());
        let first = stamped(&mut server, json!({ "type": "error" }), &Context::default());
        let second = stamped(&mut server, json!({ "type": "error" }), &Context::default());
        assert_eq!(first["event_id"], json!("event_mock_1"));
        assert_eq!(second["event_id"], json!("event_mock_2"));
        // The generated id leads, as it does in both dialects' own envelopes.
        let keys: Vec<&str> = first
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["event_id", "type"]);
    }

    #[test]
    fn an_event_id_the_script_wrote_is_left_alone() {
        let mut server = server(Script::conversation());
        let event = stamped(
            &mut server,
            json!({ "type": "error", "event_id": "mine" }),
            &Context::default(),
        );
        assert_eq!(event["event_id"], json!("mine"));
    }

    #[test]
    fn a_verbatim_emission_is_sent_exactly_as_written() {
        let mut server = server(Script::conversation());
        let context = Context {
            response_id: Some("resp_1".into()),
            ..Context::default()
        };
        let event = server.stamp(
            &Emission::verbatim(json!({ "type": "response.created", "response": {} })),
            &context,
        );
        assert_eq!(event, json!({ "type": "response.created", "response": {} }));
        assert!(realtime_response_id(&event).is_empty());
    }

    #[test]
    fn response_lifecycle_events_get_a_nested_id_and_activity_gets_a_flat_one() {
        let mut server = server(Script::conversation());
        let context = Context {
            response_id: Some("resp_7".into()),
            ..Context::default()
        };
        let created = stamped(&mut server, events::response_created(), &context);
        assert_eq!(created["response"]["id"], json!("resp_7"));
        let delta = stamped(&mut server, events::text_delta("hi"), &context);
        assert_eq!(delta["response_id"], json!("resp_7"));
        // Both are found by the shared lookup.
        assert_eq!(realtime_response_id(&created), "resp_7");
        assert_eq!(realtime_response_id(&delta), "resp_7");
    }

    #[test]
    fn a_response_id_the_script_wrote_wins_over_the_stamp() {
        let mut server = server(Script::conversation());
        let context = Context {
            response_id: Some("resp_1".into()),
            ..Context::default()
        };
        let event = stamped(
            &mut server,
            json!({ "type": "response.done", "response": { "id": "resp_vad", "status": "completed" } }),
            &context,
        );
        assert_eq!(event["response"]["id"], json!("resp_vad"));
    }

    #[test]
    fn an_event_outside_the_activity_table_is_never_given_a_response_id() {
        let mut server = server(Script::conversation());
        let context = Context {
            response_id: Some("resp_1".into()),
            item_id: None,
            correlation: None,
        };
        let event = stamped(&mut server, json!({ "type": "session.updated" }), &context);
        assert!(event.get("response_id").is_none());
    }

    #[test]
    fn item_scoped_events_are_stamped_with_the_input_turn() {
        let mut server = server(Script::dictation());
        let context = Context {
            item_id: Some("item_mock_1".into()),
            ..Context::default()
        };
        for event in [
            events::input_committed(),
            events::speech_started(),
            events::input_transcript_delta("he"),
            events::input_transcript_completed("hello"),
            events::input_transcript_failed(),
        ] {
            let kind = kind_of(&event).to_owned();
            let stamped = stamped(&mut server, event, &context);
            assert_eq!(stamped["item_id"], json!("item_mock_1"), "{kind}");
        }
        let created = stamped(&mut server, events::item_created(json!({})), &context);
        assert_eq!(created["item"]["id"], json!("item_mock_1"));
    }

    #[test]
    fn correlation_is_echoed_only_by_a_provider_that_declares_and_speaks_it() {
        let context = Context {
            response_id: Some("resp_1".into()),
            correlation: Some("request-1".into()),
            item_id: None,
        };

        let mut ga = server(
            Script::conversation()
                .with_provider(crate::MockProviderSpec::speech_to_speech_shaped()),
        );
        let created = stamped(&mut ga, events::response_created(), &context);
        assert_eq!(
            created["response"]["metadata"][RESPONSE_CORRELATION_KEY],
            json!("request-1")
        );

        // The beta dialect carries no correlation id, so declaring the flag
        // changes nothing — the same constraint the real dialect has.
        let mut beta = server(Script::conversation().map_provider(|spec| {
            spec.with_capabilities(ProviderCapabilities {
                response_metadata_correlation: true,
                ..ProviderCapabilities::DEFAULT
            })
        }));
        let created = stamped(&mut beta, events::response_created(), &context);
        assert!(created["response"].get("metadata").is_none());

        // And a GA provider that does *not* declare it stays silent too: the
        // dialect can carry the id, but this service says it does not echo one,
        // so the session must fall back to FIFO correlation rather than being
        // handed metadata a real provider would not have sent.
        let mut quiet_ga = server(Script::conversation().map_provider(|spec| {
            spec.with_dialect(crate::MockDialect::Ga)
                .with_capabilities(ProviderCapabilities {
                    response_metadata_correlation: false,
                    ..ProviderCapabilities::DEFAULT
                })
        }));
        let created = stamped(&mut quiet_ga, events::response_created(), &context);
        assert!(created["response"].get("metadata").is_none(), "{created}");
    }

    #[test]
    fn an_event_that_already_names_its_response_is_never_given_a_second_id() {
        // `realtime_response_id` looks at `response_id`, then `response.id`,
        // then `item.response_id`. An event that carries only the third already
        // names its response, so stamping a top-level `response_id` from the
        // trigger's context would bind it to a *different* response and make the
        // event correlate two ways at once.
        let mut server = server(Script::conversation());
        let context = Context {
            response_id: Some("resp_1".into()),
            ..Context::default()
        };
        let event = stamped(
            &mut server,
            json!({
                "type": "response.audio.delta",
                "item": { "response_id": "resp_other" },
                "delta": "audio",
            }),
            &context,
        );
        assert!(event.get("response_id").is_none(), "{event}");
        assert_eq!(realtime_response_id(&event), "resp_other");
    }

    #[test]
    fn steps_fire_in_order_and_once_by_default() {
        let mut server = server(
            Script::conversation()
                .turn(crate::script::turns::say("first"))
                .turn(crate::script::turns::say("second")),
        );
        let first = server.match_step("response.create").expect("first step");
        assert_eq!(first[1].event["delta"], json!("first"));
        let second = server.match_step("response.create").expect("second step");
        assert_eq!(second[1].event["delta"], json!("second"));
        assert!(
            server.match_step("response.create").is_none(),
            "both steps are exhausted, so the default answers"
        );
    }

    #[test]
    fn an_always_step_never_exhausts() {
        let mut server = server(Script::conversation().on_always(
            crate::script::Trigger::AudioCommit,
            [Emission::now(events::input_transcript_completed("hi"))],
        ));
        for _ in 0..5 {
            assert!(server.match_step("input_audio_buffer.commit").is_some());
        }
    }

    #[test]
    fn a_step_for_another_frame_is_not_consumed() {
        let mut server = server(
            Script::conversation()
                .on(crate::script::Trigger::ItemCreate, [])
                .turn(crate::script::turns::say("only turn")),
        );
        assert!(server.match_step("response.create").is_some());
        assert!(
            server.match_step("conversation.item.create").is_some(),
            "the item step is still there"
        );
    }

    #[test]
    fn the_default_answers_are_what_a_provider_would_say() {
        let server = server(Script::conversation());
        let context = Context::default();
        assert_eq!(
            server
                .default_emissions("session.update", &json!({}), &context)
                .iter()
                .map(Emission::kind)
                .collect::<Vec<_>>(),
            ["session.updated"]
        );
        assert_eq!(
            server
                .default_emissions("response.create", &json!({}), &context)
                .iter()
                .map(Emission::kind)
                .collect::<Vec<_>>(),
            ["response.created", "response.done"]
        );
        assert_eq!(
            server
                .default_emissions("input_audio_buffer.commit", &json!({}), &context)
                .iter()
                .map(Emission::kind)
                .collect::<Vec<_>>(),
            ["input_audio_buffer.committed"]
        );
        assert!(
            server
                .default_emissions("input_audio_buffer.append", &json!({}), &context)
                .is_empty()
        );
        assert!(
            server
                .default_emissions("response.cancel", &json!({}), &context)
                .is_empty(),
            "nothing is running, so there is nothing to cancel"
        );
    }

    #[test]
    fn a_provider_that_does_not_acknowledge_says_nothing_about_session_update() {
        let server = server(
            Script::conversation()
                .with_provider(crate::MockProviderSpec::speech_to_speech_shaped()),
        );
        assert!(
            server
                .default_emissions("session.update", &json!({}), &Context::default())
                .is_empty()
        );
    }

    #[test]
    fn an_echoing_provider_keeps_the_clients_item_id_and_a_non_echoing_one_replaces_it() {
        let frame = json!({
            "type": "conversation.item.create",
            "item": { "id": "item_client", "type": "message" },
        });

        let mut echoing = server(Script::conversation());
        let context = echoing.context_for("conversation.item.create", &frame);
        assert_eq!(context.item_id.as_deref(), Some("item_client"));
        let item = echoing.echoed_item(&frame, &context);
        assert_eq!(item["id"], json!("item_client"));
        assert_eq!(item["type"], json!("message"));
        assert_eq!(item["status"], json!("completed"));

        let mut replacing = server(Script::conversation().map_provider(|spec| {
            spec.with_capabilities(ProviderCapabilities {
                conversation_item_id_echo: false,
                ..ProviderCapabilities::DEFAULT
            })
        }));
        let context = replacing.context_for("conversation.item.create", &frame);
        assert_eq!(context.item_id.as_deref(), Some("item_mock_1"));
        assert_eq!(
            replacing.echoed_item(&frame, &context)["id"],
            json!("item_mock_1")
        );
    }

    #[test]
    fn an_item_with_no_id_at_all_still_gets_one() {
        let mut server = server(Script::conversation());
        let frame = json!({ "type": "conversation.item.create", "item": { "type": "message" } });
        let context = server.context_for("conversation.item.create", &frame);
        assert_eq!(context.item_id.as_deref(), Some("item_mock_1"));
    }

    #[test]
    fn response_ids_count_up_and_the_correlation_id_is_read_off_the_frame() {
        let mut server = server(Script::conversation());
        let plain = server.context_for("response.create", &json!({ "type": "response.create" }));
        assert_eq!(plain.response_id.as_deref(), Some("resp_1"));
        assert_eq!(plain.correlation, None);

        let correlated = server.context_for(
            "response.create",
            &json!({
                "type": "response.create",
                "response": { "metadata": { RESPONSE_CORRELATION_KEY: "request-9" } },
            }),
        );
        assert_eq!(correlated.response_id.as_deref(), Some("resp_2"));
        assert_eq!(correlated.correlation.as_deref(), Some("request-9"));
    }

    #[test]
    fn an_input_turn_keeps_one_item_id_until_it_is_committed() {
        let mut server = server(Script::dictation());
        let append = json!({ "type": "input_audio_buffer.append", "audio": "" });
        let first = server.context_for("input_audio_buffer.append", &append);
        let second = server.context_for("input_audio_buffer.append", &append);
        assert_eq!(first.item_id, second.item_id);

        server.on_frame(json!({ "type": "input_audio_buffer.commit" }));
        let next_turn = server.context_for("input_audio_buffer.append", &append);
        assert_ne!(next_turn.item_id, first.item_id);
    }

    #[test]
    fn emitting_response_created_and_done_opens_and_closes_the_slot() {
        let mut server = server(Script::conversation());
        assert_eq!(server.active_response, None);
        server.emit(json!({ "type": "response.created", "response": { "id": "resp_1" } }));
        assert_eq!(server.active_response.as_deref(), Some("resp_1"));
        // A `response.done` for something else leaves the slot alone.
        server.emit(json!({ "type": "response.done", "response": { "id": "other" } }));
        assert_eq!(server.active_response.as_deref(), Some("resp_1"));
        server.emit(json!({ "type": "response.done", "response": { "id": "resp_1" } }));
        assert_eq!(server.active_response, None);
    }

    #[test]
    fn one_response_slot_refuses_a_racing_create_without_consuming_a_step() {
        let mut server = server(
            Script::conversation()
                .with_provider(crate::MockProviderSpec::speech_to_speech_shaped())
                .turn(crate::script::turns::say("the answer")),
        );
        server.emit(json!({ "type": "response.created", "response": { "id": "resp_vad" } }));

        let queued = server.on_frame(json!({ "type": "response.create" }));
        assert_eq!(queued.len(), 1);
        assert_eq!(kind_of(&queued[0].event), "error");
        assert_eq!(
            queued[0].event["error"]["message"],
            json!(messages::RESPONSE_SLOT_BUSY)
        );
        assert!(!server.used[0], "the scripted turn is still waiting");

        // Once the racing response ends, the retry gets the scripted turn.
        server.emit(json!({ "type": "response.done", "response": { "id": "resp_vad" } }));
        let queued = server.on_frame(json!({ "type": "response.create" }));
        assert_eq!(
            queued.iter().map(|q| kind_of(&q.event)).collect::<Vec<_>>(),
            [
                "response.created",
                "response.text.delta",
                "response.text.done",
                "response.done"
            ]
        );
    }

    #[test]
    fn a_provider_with_a_queueing_slot_never_refuses() {
        let mut server = server(Script::conversation());
        server.emit(json!({ "type": "response.created", "response": { "id": "resp_vad" } }));
        let queued = server.on_frame(json!({ "type": "response.create" }));
        assert_eq!(kind_of(&queued[0].event), "response.created");
    }

    #[test]
    fn delays_accumulate_along_the_queue_rather_than_from_one_instant() {
        let mut server = server(Script::conversation());
        let queued = server.enqueue(vec![
            (Duration::from_millis(10), json!({ "type": "a" })),
            (Duration::from_millis(10), json!({ "type": "b" })),
            (Duration::ZERO, json!({ "type": "c" })),
        ]);
        assert_eq!(queued[1].at - queued[0].at, Duration::from_millis(10));
        assert_eq!(queued[2].at, queued[1].at);
        assert!(queued[0].at <= queued[1].at && queued[1].at <= queued[2].at);
    }

    #[test]
    fn a_frame_that_is_not_json_is_dropped_rather_than_taken_as_a_failure() {
        assert!(matches!(
            decode_frame(&Message::Text("not json".into())),
            Frame::Ignored
        ));
        assert!(matches!(
            decode_frame(&Message::Ping(Vec::new().into())),
            Frame::Ignored
        ));
        assert!(matches!(decode_frame(&Message::Close(None)), Frame::Closed));
        assert!(matches!(
            decode_frame(&Message::Text(r#"{"type":"x"}"#.into())),
            Frame::Json(_)
        ));
    }

    #[test]
    fn the_transcript_reads_back_what_crossed_the_transport() {
        let mut transcript = Transcript::default();
        transcript.outbound.push(json!({
            "type": "conversation.item.create",
            "item": {
                "type": "function_call_output",
                "call_id": "call_1",
                "output": r#"{"status":"ok"}"#,
            },
        }));
        transcript
            .outbound
            .push(json!({ "type": "response.create", "response": { "modalities": ["text"] } }));
        transcript
            .outbound
            .push(json!({ "type": "response.create" }));
        transcript
            .outbound
            .push(json!({ "type": "session.update", "session": { "instructions": "be brief" } }));
        transcript
            .inbound
            .push(json!({ "type": "session.created" }));

        assert_eq!(
            transcript.function_outputs(),
            vec![FunctionOutput {
                call_id: "call_1".into(),
                output: json!({ "status": "ok" }),
            }]
        );
        assert_eq!(
            transcript.response_creates(),
            vec![Some(&json!({ "modalities": ["text"] })), None]
        );
        assert_eq!(
            transcript.session_updates(),
            vec![&json!({ "instructions": "be brief" })]
        );
        assert_eq!(transcript.count_of("response.create"), 2);
        assert_eq!(transcript.inbound_kinds(), ["session.created"]);
        assert_eq!(
            transcript.first_of("session.update").map(kind_of),
            Some("session.update")
        );
        assert_eq!(transcript.first_of("nothing.here"), None);
    }

    #[test]
    fn appended_audio_decodes_back_to_the_samples_that_were_sent() {
        let audio =
            crate::CannedAudio::tone(via_audio::SampleRate::HZ_16000, Duration::from_millis(40));
        let mut transcript = Transcript::default();
        for chunk in audio.chunks() {
            transcript.outbound.push(json!({
                "type": "input_audio_buffer.append",
                "audio": chunk.base64,
            }));
        }
        assert_eq!(transcript.input_audio_base64().len(), 2);
        assert_eq!(
            transcript.input_audio_pcm16().expect("decode"),
            audio.samples()
        );
    }

    #[test]
    fn audio_that_is_not_base64_is_reported_rather_than_swallowed() {
        let mut transcript = Transcript::default();
        transcript.outbound.push(json!({
            "type": "input_audio_buffer.append",
            "audio": "not base64!!",
        }));
        // The failure is the *caller under test* having put something that is
        // not base64 PCM16 on the wire, so it is reported rather than decoded
        // into silence.
        let error = transcript
            .input_audio_pcm16()
            .map(|_| ())
            .expect_err("refused");
        assert_eq!(error.code(), "VIA_MOCK_NOT_BASE64");
        assert!(matches!(error, MockError::NotBase64 { .. }), "{error:?}");
    }

    #[test]
    fn a_nested_field_is_created_when_the_event_has_the_key_as_a_non_object() {
        let mut event = json!({ "type": "response.created", "response": 7 });
        set_nested_if_absent(&mut event, "response", "id", "resp_1");
        assert_eq!(event["response"], json!({ "id": "resp_1" }));

        let mut event = json!({ "type": "response.created", "response": { "metadata": 7 } });
        set_correlation(&mut event, "request-1");
        assert_eq!(
            event["response"]["metadata"][RESPONSE_CORRELATION_KEY],
            json!("request-1")
        );
    }

    #[test]
    fn stamping_a_non_object_event_changes_nothing_and_does_not_panic() {
        let mut server = server(Script::conversation());
        let context = Context {
            response_id: Some("resp_1".into()),
            item_id: Some("item_mock_1".into()),
            correlation: None,
        };
        // `with_leading` wraps a non-object into `{ event_id }`, which is the
        // JavaScript spread's own answer to `{ ...5 }`.
        let event = stamped(&mut server, json!("not an object"), &context);
        assert_eq!(event["event_id"], json!("event_mock_1"));
    }
}
