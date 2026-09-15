//! The duplex illusion.
//!
//! A realtime session is full duplex and interruptible. A cascade is three
//! request/response calls. This module is the whole of the distance between
//! them, and it is where the design effort of this crate went.
//!
//! ```text
//!            ┌──────────────── one owning task ────────────────┐
//!  frames in │  VAD ─► ASR ─► [turn] ─► sentences ─► TTS ─► PCM │ frames out
//!  ─────────►│   │                │                      │      ├──────────►
//!            │   └── barge-in ────┴──────────────────────┘      │
//!            └─────────────────────────────────────────────────┘
//! ```
//!
//! # What makes it duplex rather than a queue
//!
//! Three things happen at once and the loop polls all three, `biased`, in this
//! order:
//!
//! 1. **A frame from the session.** First, always. Audio is how barge-in
//!    arrives, and a loop that drained a generation before reading its input
//!    would hear the interruption after the turn it was meant to interrupt.
//! 2. **A delta from the reasoning turn**, while it is generating.
//! 3. **A chunk from the speaker**, while it is speaking.
//!
//! 2 and 3 overlap on purpose: [`crate::sentence`] cuts the generation into
//! utterances and the speaker starts on the first one while the reasoning stage
//! is still producing the second. That overlap is the difference between a voice
//! agent and a form submission — on a laptop, the whole generation would
//! otherwise be added to the time-to-first-audio.
//!
//! # Barge-in
//!
//! `docs/architecture.md` §7 and §11. When the VAD reports a speech **edge**
//! while a response is live, three things happen in this order, and the order is
//! what a cloud provider with server VAD and `interrupt_response` does:
//!
//! 1. `input_audio_buffer.speech_started` — the Gateway's `userSpeaking` term,
//!    and the Injection Gate's first blocking input;
//! 2. the reasoning turn and the speech stream are **dropped**, which is the
//!    cancellation ([`crate::stages`]), and the half-generated sentence is
//!    discarded so it cannot surface on the next turn;
//! 3. `response.done` with status `cancelled`, so `via-realtime` settles the
//!    caller's outcome as [`OutcomeKind::Cancelled`] rather than leaving it to
//!    the response-inactivity watchdog 120 seconds later.
//!
//! [`PlaybackCursor`] is what makes step 2 reportable: the response's speech
//! clock says how much audio reached the wire before the cut, which is the
//! number `conversation.item.truncate` carries and the number the `[local]` log
//! line prints. A second cursor runs on the input, so `audio_start_ms` and
//! `audio_end_ms` are positions in the session's own audio rather than
//! wall-clock guesses.
//!
//! # One response slot, a two-deep queue
//!
//! The pipeline runs one reasoning turn at a time, so it declares
//! [`single_response_slot`]. A create that cannot start yet is **queued**
//! rather than refused: there is no remote service to say no, and starting an
//! announcement the moment the user's turn lands is what the Injection Gate
//! would have asked for anyway.
//!
//! [`MAX_QUEUED_RESPONSES`] is two, which is exactly the depth the one real
//! interleaving needs — a `response.create` that arrived while the user was
//! speaking, plus the turn that user's speech is about to produce. A third is a
//! genuine conflict, and it is refused with the **catalogued** vocabulary:
//! ARGO's [`ACTIVE_RESPONSE_CONFLICT_CODE`] and its invariant phrase, so
//! `via-realtime`'s bounded busy-retry ladder replays the refused payload byte
//! for byte instead of the Gateway inventing a retry.
//!
//! # The wire strings are not localized
//!
//! The two `error` messages this module emits are **protocol vocabulary**:
//! `classify_error` matches on them, and `via-realtime-openai`'s corpus is what
//! matches. Translating them would silently stop the busy-retry ladder in two
//! locales out of three. Everything a *person* reads still comes from
//! `via-i18n`, through [`crate::LocalError`].
//!
//! [`OutcomeKind::Cancelled`]: via_realtime::OutcomeKind::Cancelled
//! [`single_response_slot`]: via_realtime::ProviderCapabilities::single_response_slot
//! [`ACTIVE_RESPONSE_CONFLICT_CODE`]: via_realtime_openai::ACTIVE_RESPONSE_CONFLICT_CODE

use std::collections::VecDeque;

use futures::StreamExt as _;
use futures::channel::mpsc as futures_mpsc;
use serde_json::{Map, Value};
use tokio_tungstenite::tungstenite::Message;
use via_audio::{ChannelCount, PlaybackCursor, Resampler, SampleRate};
use via_protocol::SessionMode;
use via_realtime::RESPONSE_CORRELATION_KEY;
use via_realtime_openai::{ACTIVE_RESPONSE_CONFLICT_CODE, CANCEL_NOT_ACTIVE_CODE};

use crate::events;
use crate::sentence::SentenceSplitter;
use crate::stages::{
    ResponseDelta, ResponseStream, SpeechEvent, SpeechStream, StageError, Stages, ToolCall,
    TranscriptUpdate, Turn, TurnMessage,
};

/// The tracing target the pipeline's one-glance diagnostics go to.
///
/// Beside `via-realtime-openai`'s `[live]` markers, and for the same reason: one
/// `grep` shows the whole of a session's turn structure.
pub const LOG_TARGET: &str = "via::realtime::local";

/// Depth of the frame channel from the session to the machine.
///
/// **Bounded**, unlike the machine's own outbound channel, and the asymmetry is
/// deliberate. The session is the only writer here and it rate-limits itself to
/// what a client sends, so backpressure is real and safe. The machine's outbound
/// channel is unbounded because the machine must never block while writing: a
/// blocked machine stops reading this channel, the session's writer blocks, the
/// session's state task stops draining events, and the two halves deadlock.
pub const INBOUND_CAPACITY: usize = 64;

/// How many creates may wait for the one response slot.
///
/// Two, because two is the depth the one real interleaving needs — see the
/// module docs. A third is refused rather than dropped, so a Gateway that
/// genuinely over-produces sees the conflict instead of losing an announcement
/// silently.
pub const MAX_QUEUED_RESPONSES: usize = 2;

/// The refusal a `response.create` beyond [`MAX_QUEUED_RESPONSES`] gets.
///
/// **Wire vocabulary, not prose.** The sentence carries ARGO's invariant phrase
/// so `classify_openai_error` answers `ResponseSlotBusy` and the bounded ladder
/// replays the payload. See the module docs.
#[must_use]
pub fn active_response_conflict_message(response_id: &str) -> String {
    format!("Conversation already has an active response in progress: {response_id}.")
}

/// The refusal a `response.cancel` with nothing to cancel gets.
///
/// Wire vocabulary again: it carries both halves `is_benign_cancel_race` needs,
/// so `via-realtime` classifies it `NoActiveResponse` and suppresses it. Without
/// it, a host that wires barge-in to speech-start puts an error on screen for
/// every first utterance — ARGO bring-up §2.
pub const CANCEL_NOT_ACTIVE_MESSAGE: &str = "Cancellation failed: no active response found";

/// The error code a stage failure that has no response to attach to carries.
pub const STAGE_UNAVAILABLE_CODE: &str = "local_stage_unavailable";

/// What the machine needs to run.
pub struct MachineOptions {
    /// The four engines.
    pub stages: Stages,
    /// Which layers this session mounts — `dictation` mounts no model turn.
    pub mode: SessionMode,
    /// The voice the speaker is asked for.
    pub voice: String,
    /// The rate the client captures at, as the provider declared it.
    pub client_rate: SampleRate,
}

/// A user turn in progress.
#[derive(Debug)]
struct Listening {
    item_id: String,
    started_ms: u64,
}

/// Who asked for a response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CreateSource {
    /// The client wrote `response.create`; a waiter is attached to it.
    Client,
    /// Turn detection completed a user utterance; nothing is waiting.
    TurnDetection,
}

/// A response that has been asked for but not started.
#[derive(Debug)]
struct PendingCreate {
    source: CreateSource,
    correlation: Option<String>,
    instructions: Option<String>,
    transcript: String,
}

/// The response that is live.
struct ActiveResponse {
    id: String,
    item_id: String,
    correlation: Option<String>,
    splitter: SentenceSplitter,
    transcript: String,
    utterances: VecDeque<String>,
    /// The response's speech clock: frames produced, then frames written.
    speech: PlaybackCursor,
    reasoning_done: bool,
    audio_open: bool,
    output: Vec<Value>,
}

/// The state the loop owns by value.
struct Machine {
    stages: Stages,
    mode: SessionMode,
    voice: String,
    outbound: futures_mpsc::UnboundedSender<Result<Message, String>>,
    closed: bool,

    resampler: Option<Resampler>,
    block_frames: usize,
    staged: Vec<f32>,
    input_clock: PlaybackCursor,

    session: Value,
    instructions: String,
    tools: Vec<Value>,
    history: Vec<TurnMessage>,

    listening: Option<Listening>,
    active: Option<ActiveResponse>,
    queue: VecDeque<PendingCreate>,
    pending_user_text: Option<String>,
    previous_item: Option<String>,

    sequence: u64,
}

fn frames_of(samples: usize) -> u64 {
    u64::try_from(samples).unwrap_or(u64::MAX)
}

/// Run the pipeline until the session drops its end of the transport.
///
/// Every failure is reported to the session as a frame rather than returned: a
/// pipeline that returned an error would have nobody to return it to. The task
/// ends when the inbound channel closes, which is what dropping the
/// [`RealtimeSession`](via_realtime::RealtimeSession) does.
pub(crate) async fn run(
    options: MachineOptions,
    inbound: futures_mpsc::Receiver<Message>,
    outbound: futures_mpsc::UnboundedSender<Result<Message, String>>,
) {
    let engine_rate = options.stages.voice_activity.sample_rate();
    let resampler = build_resampler(options.client_rate, engine_rate);

    let mut machine = Machine {
        block_frames: engine_rate.capture_block_frames(),
        resampler,
        staged: Vec::new(),
        input_clock: PlaybackCursor::new(engine_rate),
        stages: options.stages,
        mode: options.mode,
        voice: options.voice,
        outbound,
        closed: false,
        session: Value::Object(Map::new()),
        instructions: String::new(),
        tools: Vec::new(),
        history: Vec::new(),
        listening: None,
        active: None,
        queue: VecDeque::new(),
        pending_user_text: None,
        previous_item: None,
        sequence: 0,
    };

    let opening = machine.session.clone();
    machine.emit(events::session_created(opening));

    let mut inbound = inbound;
    let mut reasoning: Option<ResponseStream> = None;
    let mut speech: Option<SpeechStream> = None;

    while !machine.closed {
        tokio::select! {
            biased;

            frame = inbound.next() => {
                let Some(frame) = frame else { break };
                machine.on_frame(&frame, &mut reasoning, &mut speech).await;
            }

            delta = next_delta(&mut reasoning) => {
                machine.on_reasoning(delta, &mut reasoning, &mut speech).await;
            }

            chunk = next_chunk(&mut speech) => {
                machine.on_speech(chunk, &mut reasoning, &mut speech).await;
            }
        }
        // Draining here rather than at each site that frees the slot is what
        // keeps `start` out of its own call graph: an `async fn` that reached
        // itself through the queue would be an infinitely sized future.
        machine.drain_queue(&mut reasoning, &mut speech).await;
    }

    tracing::debug!(target: LOG_TARGET, "[local] pipeline closed");
}

/// The converter between the rate the client captures at and the rate the
/// engines consume.
///
/// `None` when the two agree, which is the shipped configuration — the provider
/// declares [`PIPELINE_INPUT_RATE`](crate::stages::PIPELINE_INPUT_RATE) as its
/// `input_sample_rate`, so a client that obeys `voice.ready` needs no
/// conversion at all. It is built anyway, because a stage is free to declare a
/// different rate and a pipeline that silently fed a 16 kHz VAD 24 kHz audio
/// would not crash — it would just stop hearing the user.
fn build_resampler(client_rate: SampleRate, engine_rate: SampleRate) -> Option<Resampler> {
    if client_rate == engine_rate {
        return None;
    }
    match Resampler::new(client_rate, engine_rate, ChannelCount::MONO) {
        Ok(resampler) => {
            tracing::info!(
                target: LOG_TARGET,
                "[local] converting capture {} Hz to engine {} Hz",
                client_rate.hz(),
                engine_rate.hz(),
            );
            Some(resampler)
        }
        Err(error) => {
            // A rate pair `rubato` refuses. Running without a converter would
            // feed the engines audio at the wrong speed, so the block loop
            // drops audio instead and says why — the session still answers
            // frames and still closes cleanly.
            tracing::error!(
                target: LOG_TARGET,
                "[local] no converter from {} Hz to {} Hz: {error}",
                client_rate.hz(),
                engine_rate.hz(),
            );
            None
        }
    }
}

/// The next reasoning delta, or a future that never completes when there is no
/// turn in flight.
async fn next_delta(
    stream: &mut Option<ResponseStream>,
) -> Option<Result<ResponseDelta, StageError>> {
    match stream {
        Some(inner) => inner.next().await,
        None => std::future::pending().await,
    }
}

/// The next speech chunk, or a future that never completes when nothing is
/// speaking.
async fn next_chunk(stream: &mut Option<SpeechStream>) -> Option<Result<Vec<i16>, StageError>> {
    match stream {
        Some(inner) => inner.next().await,
        None => std::future::pending().await,
    }
}

impl Machine {
    // ── plumbing ────────────────────────────────────────────────────────────

    fn emit(&mut self, event: Value) {
        if self.closed {
            return;
        }
        let text = event.to_string();
        if self
            .outbound
            .unbounded_send(Ok(Message::Text(text.into())))
            .is_err()
        {
            self.closed = true;
        }
    }

    fn next_id(&mut self, prefix: &str) -> String {
        self.sequence += 1;
        format!("{prefix}_local_{}", self.sequence)
    }

    /// `docs/architecture.md` §2: `dictation` is the one mode that mounts no
    /// model at all, and on the local pipeline it is the cheapest path by a wide
    /// margin — VAD plus ASR, no LLM, no TTS.
    fn has_model_turn(&self) -> bool {
        !matches!(self.mode, SessionMode::Dictation)
    }

    // ── inbound frames ──────────────────────────────────────────────────────

    async fn on_frame(
        &mut self,
        frame: &Message,
        reasoning: &mut Option<ResponseStream>,
        speech: &mut Option<SpeechStream>,
    ) {
        let Message::Text(text) = frame else {
            // Ping, pong, binary, close: nothing the pipeline speaks. A local
            // transport never sends them, and a future one that does must not
            // be answered with a protocol complaint.
            return;
        };
        let Ok(event) = serde_json::from_str::<Value>(text.as_str()) else {
            tracing::warn!(target: LOG_TARGET, "[local] dropped an unparseable frame");
            return;
        };
        let kind = event
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        tracing::trace!(target: LOG_TARGET, "[local] inbound type={kind}");

        match kind.as_str() {
            "session.update" => {
                let session = event.get("session").cloned().unwrap_or(Value::Null);
                self.apply_session(session);
                let updated = self.session.clone();
                self.emit(events::session_updated(updated));
            }
            "input_audio_buffer.append" => {
                let audio = event
                    .get("audio")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                self.on_audio(&audio, reasoning, speech).await;
            }
            "input_audio_buffer.commit" => self.close_utterance(reasoning, speech),
            "input_audio_buffer.clear" => {
                self.stages.transcriber.reset();
                self.stages.voice_activity.reset();
                self.listening = None;
                self.staged.clear();
            }
            "conversation.item.create" => {
                self.on_item(event.get("item").cloned().unwrap_or(Value::Null));
            }
            "conversation.item.truncate" => self.on_truncate(&event),
            "response.create" => self.on_create(event.get("response")),
            "response.cancel" => self.on_cancel(reasoning, speech),
            _ => {}
        }
    }

    fn apply_session(&mut self, session: Value) {
        let Value::Object(fields) = session else {
            return;
        };
        if let Some(instructions) = fields.get("instructions").and_then(Value::as_str) {
            self.instructions = instructions.to_owned();
        }
        if let Some(tools) = fields.get("tools").and_then(Value::as_array) {
            self.tools = tools.clone();
        }
        // The payload is merged rather than replaced, exactly as a realtime
        // service treats it: `session.update` is a patch, and every update after
        // the first carries only instructions and tools.
        let mut merged = match self.session.take() {
            Value::Object(existing) => existing,
            _ => Map::new(),
        };
        for (key, value) in fields {
            merged.insert(key, value);
        }
        self.session = Value::Object(merged);
    }

    /// Record the newest user input, moving whatever it replaces into history.
    ///
    /// Only one input can be *this* turn's; a previous one that never got a
    /// response is conversation history all the same, and dropping it would
    /// lose a turn the model was told about.
    fn remember_user_text(&mut self, text: String) {
        if let Some(previous) = self.pending_user_text.replace(text) {
            self.history.push(TurnMessage::user(previous));
        }
    }

    fn on_item(&mut self, item: Value) {
        if item.is_null() {
            return;
        }
        if let Some(text) = user_item_text(&item) {
            self.remember_user_text(text);
        }
        if let Some(id) = item.get("id").and_then(Value::as_str) {
            self.previous_item = Some(id.to_owned());
        }
        self.emit(events::item_created(item));
    }

    fn on_truncate(&mut self, event: &Value) {
        let item_id = event
            .get("item_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        // The client's own number when it sent one, because it is the only side
        // that knows how much of the audio actually reached the speaker. The
        // response's speech clock is the fallback.
        let audio_end_ms = event
            .get("audio_end_ms")
            .and_then(Value::as_u64)
            .unwrap_or_else(|| {
                self.active
                    .as_ref()
                    .map_or(0, |active| active.speech.audio_end_ms())
            });
        self.emit(events::item_truncated(&item_id, audio_end_ms));
    }

    // ── audio in ────────────────────────────────────────────────────────────

    async fn on_audio(
        &mut self,
        encoded: &str,
        reasoning: &mut Option<ResponseStream>,
        speech: &mut Option<SpeechStream>,
    ) {
        let Some(samples) = events::decode_audio(encoded) else {
            tracing::warn!(target: LOG_TARGET, "[local] dropped an unusable audio append");
            return;
        };
        if samples.is_empty() {
            return;
        }
        let converted = self.convert(&samples);
        self.staged.extend_from_slice(&converted);

        // Whole blocks only. A stage that was handed a partial block would see
        // the utterance's timing shift by however much the client happened to
        // put in one frame.
        while self.staged.len() >= self.block_frames && !self.closed {
            let block: Vec<f32> = self.staged.drain(..self.block_frames).collect();
            self.on_block(&block, reasoning, speech);
        }
    }

    fn convert(&mut self, samples: &[i16]) -> Vec<f32> {
        let mono = via_audio::pcm16_to_f32_slice(samples);
        let Some(resampler) = self.resampler.as_mut() else {
            return mono;
        };
        match resampler.process_interleaved(&mono) {
            Ok(converted) => converted,
            Err(error) => {
                tracing::warn!(target: LOG_TARGET, "[local] resample failed: {error}");
                Vec::new()
            }
        }
    }

    fn on_block(
        &mut self,
        block: &[f32],
        reasoning: &mut Option<ResponseStream>,
        speech: &mut Option<SpeechStream>,
    ) {
        // The input clock, so `audio_start_ms` and `audio_end_ms` are positions
        // in the session's own audio rather than wall-clock guesses.
        let frames = frames_of(block.len());
        self.input_clock.enqueue(frames);
        self.input_clock.advance(frames);

        let activity = match self.stages.voice_activity.accept(block) {
            Ok(activity) => activity,
            Err(error) => {
                self.report_stage_failure(&error, reasoning, speech);
                return;
            }
        };

        if activity == SpeechEvent::Started {
            self.open_utterance(reasoning, speech);
        }

        if activity.is_voiced() {
            match self.stages.transcriber.accept(block) {
                Ok(update) => self.publish_transcript(&update),
                Err(error) => {
                    self.report_stage_failure(&error, reasoning, speech);
                    return;
                }
            }
        }

        if activity == SpeechEvent::Ended {
            self.close_utterance(reasoning, speech);
        }
    }

    fn open_utterance(
        &mut self,
        reasoning: &mut Option<ResponseStream>,
        speech: &mut Option<SpeechStream>,
    ) {
        let item_id = self.next_id("item");
        let started_ms = self.input_clock.audio_end_ms();
        self.emit(events::speech_started(&item_id, started_ms));
        self.listening = Some(Listening {
            item_id,
            started_ms,
        });

        // Barge-in. The speech-start edge is published *first* — it is the
        // Gateway's `userSpeaking`, and the Injection Gate has to see it before
        // the cancellation it caused.
        if self.active.is_some() {
            let cut_ms = self
                .active
                .as_ref()
                .map_or(0, |active| active.speech.audio_end_ms());
            tracing::info!(
                target: LOG_TARGET,
                "[local] barge-in after {cut_ms} ms of speech"
            );
            self.cancel_active(reasoning, speech);
        }
    }

    fn publish_transcript(&mut self, update: &TranscriptUpdate) {
        if update.is_empty() {
            return;
        }
        let Some(listening) = self.listening.as_ref() else {
            return;
        };
        let item_id = listening.item_id.clone();
        self.emit(events::transcription_delta(
            &item_id,
            &update.text,
            &update.stash,
        ));
    }

    fn close_utterance(
        &mut self,
        reasoning: &mut Option<ResponseStream>,
        speech: &mut Option<SpeechStream>,
    ) {
        let Some(listening) = self.listening.take() else {
            // An explicit commit with nothing buffered. A push-to-talk client
            // that releases the key twice does this, and it is not an error.
            return;
        };
        let ended_ms = self.input_clock.audio_end_ms();
        let transcript = match self.stages.transcriber.finish() {
            Ok(transcript) => transcript,
            Err(error) => {
                self.emit(events::speech_stopped(&listening.item_id, ended_ms));
                self.report_stage_failure(&error, reasoning, speech);
                return;
            }
        };
        self.stages.voice_activity.reset();

        self.emit(events::speech_stopped(&listening.item_id, ended_ms));
        let previous = self.previous_item.clone();
        self.emit(events::input_committed(
            &listening.item_id,
            previous.as_deref(),
        ));
        self.previous_item = Some(listening.item_id.clone());
        self.emit(events::transcription_completed(
            &listening.item_id,
            &transcript,
        ));
        tracing::debug!(
            target: LOG_TARGET,
            "[local] utterance {} spanned {}-{} ms",
            listening.item_id,
            listening.started_ms,
            ended_ms,
        );

        if transcript.trim().is_empty() {
            // Turn detection fired on something that transcribed to nothing.
            // Publishing the empty transcript is right; answering it is not.
            return;
        }
        if !self.has_model_turn() {
            return;
        }
        // An item nobody asked a response for is still conversation history;
        // only the newest input is *this* turn's.
        if let Some(previous) = self.pending_user_text.take() {
            self.history.push(TurnMessage::user(previous));
        }
        self.enqueue_response(PendingCreate {
            source: CreateSource::TurnDetection,
            correlation: None,
            instructions: None,
            transcript,
        });
    }

    // ── responses ───────────────────────────────────────────────────────────

    fn on_create(&mut self, body: Option<&Value>) {
        let correlation = body
            .and_then(|body| body.get("metadata"))
            .and_then(|metadata| metadata.get(RESPONSE_CORRELATION_KEY))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let instructions = body
            .and_then(|body| body.get("instructions"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let transcript = self.pending_user_text.take().unwrap_or_default();
        self.enqueue_response(PendingCreate {
            source: CreateSource::Client,
            correlation,
            instructions,
            transcript,
        });
    }

    /// Take a create into the queue, or refuse it.
    ///
    /// Nothing starts here: the run loop drains the queue after every branch,
    /// which is what keeps [`Self::start`] out of its own call graph.
    fn enqueue_response(&mut self, create: PendingCreate) {
        if self.queue.len() < MAX_QUEUED_RESPONSES {
            self.queue.push_back(create);
            return;
        }
        match create.source {
            CreateSource::Client => {
                let active = self
                    .active
                    .as_ref()
                    .map_or_else(String::new, |active| active.id.clone());
                self.emit(events::error(
                    ACTIVE_RESPONSE_CONFLICT_CODE,
                    &active_response_conflict_message(&active),
                ));
            }
            CreateSource::TurnDetection => {
                // Nothing is waiting on an automatic turn, so there is nothing
                // to refuse to. Dropping it is visible in the log rather than on
                // the wire.
                tracing::warn!(
                    target: LOG_TARGET,
                    "[local] dropped an automatic turn: the response queue is full"
                );
            }
        }
    }

    async fn drain_queue(
        &mut self,
        reasoning: &mut Option<ResponseStream>,
        speech: &mut Option<SpeechStream>,
    ) {
        // Terminates: every iteration pops one entry, and `start` never pushes.
        while !self.closed && self.active.is_none() && self.listening.is_none() {
            let Some(next) = self.queue.pop_front() else {
                return;
            };
            self.start(next, reasoning, speech).await;
        }
    }

    async fn start(
        &mut self,
        create: PendingCreate,
        reasoning: &mut Option<ResponseStream>,
        speech: &mut Option<SpeechStream>,
    ) {
        // `history` is everything *before* this turn and `transcript` is the
        // turn's own input, so a stage that renders both — every one of them
        // does — never shows the model its own user turn twice. The push
        // happens after the turn is built, which is what makes that true.
        let turn = Turn {
            instructions: self.instructions.clone(),
            response_instructions: create.instructions,
            transcript: create.transcript.clone(),
            history: self.history.clone(),
            tools: self.tools.clone(),
        };
        if !create.transcript.trim().is_empty() {
            self.history.push(TurnMessage::user(create.transcript));
        }
        let stream = match self.stages.responder.respond(turn).await {
            Ok(stream) => stream,
            Err(error) => {
                // Refused before the response exists, which is what a provider
                // answers a `response.create` it cannot serve with: an `error`
                // carrying no response id, which `via-realtime` binds to the
                // start that has not been correlated yet.
                tracing::warn!(
                    target: LOG_TARGET,
                    "[local] the reasoning turn refused to start: {error}"
                );
                self.emit(events::error(error.stage.as_str(), &error.to_string()));
                return;
            }
        };

        let id = self.next_id("resp");
        let item_id = self.next_id("msg");
        self.emit(events::response_created(&id, create.correlation.as_deref()));
        self.emit(events::output_item_added(
            &id,
            0,
            events::assistant_message_item(&item_id),
        ));
        tracing::debug!(target: LOG_TARGET, "[local] response {id} started");

        self.active = Some(ActiveResponse {
            id,
            item_id,
            correlation: create.correlation,
            splitter: SentenceSplitter::new(),
            transcript: String::new(),
            utterances: VecDeque::new(),
            speech: PlaybackCursor::new(self.stages.speaker.sample_rate()),
            reasoning_done: false,
            audio_open: false,
            output: Vec::new(),
        });
        *reasoning = Some(stream);
        *speech = None;
    }

    async fn on_reasoning(
        &mut self,
        delta: Option<Result<ResponseDelta, StageError>>,
        reasoning: &mut Option<ResponseStream>,
        speech: &mut Option<SpeechStream>,
    ) {
        match delta {
            Some(Ok(ResponseDelta::Text(text))) => {
                let Some(active) = self.active.as_mut() else {
                    return;
                };
                active.transcript.push_str(&text);
                let sentences = active.splitter.push(&text);
                active.utterances.extend(sentences);
                let (id, item_id) = (active.id.clone(), active.item_id.clone());
                self.emit(events::audio_transcript_delta(&id, &item_id, &text));
                self.speak_next(reasoning, speech).await;
            }
            Some(Ok(ResponseDelta::Tool(call))) => self.publish_tool_call(&call),
            Some(Err(error)) => {
                tracing::warn!(target: LOG_TARGET, "[local] the reasoning turn failed: {error}");
                self.report_stage_failure(&error, reasoning, speech);
            }
            None => {
                *reasoning = None;
                let Some(active) = self.active.as_mut() else {
                    return;
                };
                active.reasoning_done = true;
                let tail = active.splitter.flush();
                active.utterances.extend(tail);
                self.speak_next(reasoning, speech).await;
                self.finish_if_done(speech);
            }
        }
    }

    fn publish_tool_call(&mut self, call: &ToolCall) {
        let Some(active) = self.active.as_ref() else {
            return;
        };
        // Output index 0 is the assistant message the response opened with, so
        // tool items start at 1.
        let (id, index) = (active.id.clone(), active.output.len() + 1);
        let item_id = self.next_id("fc");
        let item = events::function_call_item(&item_id, &call.call_id, &call.name, &call.arguments);
        if let Some(active) = self.active.as_mut() {
            active.output.push(item.clone());
        }
        self.emit(events::output_item_added(&id, index, item));
        self.emit(events::function_call_arguments_done(
            &id,
            &item_id,
            &call.call_id,
            &call.name,
            &call.arguments,
        ));
    }

    async fn speak_next(
        &mut self,
        reasoning: &mut Option<ResponseStream>,
        speech: &mut Option<SpeechStream>,
    ) {
        if speech.is_some() {
            return;
        }
        let Some(utterance) = self
            .active
            .as_mut()
            .and_then(|active| active.utterances.pop_front())
        else {
            return;
        };
        let voice = self.voice.clone();
        match self.stages.speaker.speak(&utterance, &voice).await {
            Ok(stream) => *speech = Some(stream),
            Err(error) => {
                tracing::warn!(target: LOG_TARGET, "[local] synthesis refused: {error}");
                self.report_stage_failure(&error, reasoning, speech);
            }
        }
    }

    async fn on_speech(
        &mut self,
        chunk: Option<Result<Vec<i16>, StageError>>,
        reasoning: &mut Option<ResponseStream>,
        speech: &mut Option<SpeechStream>,
    ) {
        match chunk {
            Some(Ok(samples)) => {
                let Some(active) = self.active.as_mut() else {
                    return;
                };
                let frames = frames_of(samples.len());
                // Produced, then written: the gap between the two is what
                // `PlaybackCursor::is_draining` is, and `audio_end_ms` after
                // both is the response's speech position — the number a
                // barge-in reports and `conversation.item.truncate` carries.
                active.speech.enqueue(frames);
                active.audio_open = true;
                let (id, item_id) = (active.id.clone(), active.item_id.clone());
                self.emit(events::audio_delta(&id, &item_id, &samples));
                if let Some(active) = self.active.as_mut() {
                    active.speech.advance(frames);
                }
            }
            Some(Err(error)) => {
                tracing::warn!(target: LOG_TARGET, "[local] synthesis failed: {error}");
                self.report_stage_failure(&error, reasoning, speech);
            }
            None => {
                *speech = None;
                self.speak_next(reasoning, speech).await;
                self.finish_if_done(speech);
            }
        }
    }

    fn finish_if_done(&mut self, speech: &Option<SpeechStream>) {
        let done = self.active.as_ref().is_some_and(|active| {
            active.reasoning_done && active.utterances.is_empty() && speech.is_none()
        });
        if !done {
            return;
        }
        let Some(active) = self.active.take() else {
            return;
        };
        if active.audio_open {
            self.emit(events::audio_done(&active.id, &active.item_id));
        }
        self.emit(events::audio_transcript_done(
            &active.id,
            &active.item_id,
            active.transcript.trim(),
        ));
        if !active.transcript.trim().is_empty() {
            self.history
                .push(TurnMessage::assistant(active.transcript.trim().to_owned()));
        }
        tracing::debug!(
            target: LOG_TARGET,
            "[local] response {} completed with {} ms of speech",
            active.id,
            active.speech.audio_end_ms(),
        );
        self.emit(events::response_done(
            &active.id,
            events::RESPONSE_COMPLETED,
            active.correlation.as_deref(),
            active.output.clone(),
        ));
    }

    // ── cancellation ────────────────────────────────────────────────────────

    fn on_cancel(
        &mut self,
        reasoning: &mut Option<ResponseStream>,
        speech: &mut Option<SpeechStream>,
    ) {
        if self.active.is_some() {
            self.cancel_active(reasoning, speech);
            return;
        }
        if self.queue.pop_front().is_some() {
            // A queued create that never started has no response id, so there
            // is nothing to close out on the wire; the caller's waiter settles
            // on the session's own response-start watchdog.
            return;
        }
        self.emit(events::error(
            CANCEL_NOT_ACTIVE_CODE,
            CANCEL_NOT_ACTIVE_MESSAGE,
        ));
    }

    /// Cut the live response: drop both streams, discard the half-generated
    /// sentence, and close it out as `cancelled`.
    fn cancel_active(
        &mut self,
        reasoning: &mut Option<ResponseStream>,
        speech: &mut Option<SpeechStream>,
    ) {
        // Dropping the streams *is* the cancellation — `crate::stages`. Both are
        // dropped before anything is emitted, so a stage that is still producing
        // cannot land a delta on a response the Gateway has already been told is
        // over.
        *reasoning = None;
        *speech = None;
        let Some(mut active) = self.active.take() else {
            return;
        };
        active.splitter.clear();
        active.utterances.clear();
        let discarded = active.speech.clear();
        if active.audio_open {
            self.emit(events::audio_done(&active.id, &active.item_id));
        }
        self.emit(events::audio_transcript_done(
            &active.id,
            &active.item_id,
            active.transcript.trim(),
        ));
        tracing::debug!(
            target: LOG_TARGET,
            "[local] response {} cancelled, {discarded} frame(s) discarded unwritten",
            active.id,
        );
        self.emit(events::response_done(
            &active.id,
            events::RESPONSE_CANCELLED,
            active.correlation.as_deref(),
            active.output.clone(),
        ));
    }

    /// Report a stage failure and end whatever it was serving.
    fn report_stage_failure(
        &mut self,
        error: &StageError,
        reasoning: &mut Option<ResponseStream>,
        speech: &mut Option<SpeechStream>,
    ) {
        *reasoning = None;
        *speech = None;
        let Some(active) = self.active.take() else {
            // A failure with no live response — the VAD or the recognizer.
            // There is no waiter to settle, so it is reported as a bare error,
            // which `via-realtime` forwards and the Gateway shows.
            self.emit(events::error(STAGE_UNAVAILABLE_CODE, &error.to_string()));
            return;
        };
        let mut frame = events::error(error.stage.as_str(), &error.to_string());
        if let Some(fields) = frame.as_object_mut() {
            // Bound to the response, so the caller's own outcome is what fails
            // rather than the session at large.
            fields.insert("response_id".to_owned(), Value::String(active.id.clone()));
        }
        self.emit(frame);
        if active.audio_open {
            self.emit(events::audio_done(&active.id, &active.item_id));
        }
        self.emit(events::response_done(
            &active.id,
            events::RESPONSE_FAILED,
            active.correlation.as_deref(),
            active.output.clone(),
        ));
    }
}

/// The text of a `message` item whose role is `user`, if it has one.
///
/// The shape is `via-realtime`'s own `user_text_item`:
/// `{ type: 'message', role: 'user', content: [{ type: 'input_text', text }] }`.
fn user_item_text(item: &Value) -> Option<String> {
    if item.get("type").and_then(Value::as_str)? != "message" {
        return None;
    }
    if item.get("role").and_then(Value::as_str)? != "user" {
        return None;
    }
    let parts = item.get("content")?.as_array()?;
    let mut text = String::new();
    for part in parts {
        if let Some(body) = part.get("text").and_then(Value::as_str) {
            text.push_str(body);
        }
    }
    (!text.trim().is_empty()).then_some(text)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use via_realtime::ErrorClass;
    use via_realtime_openai::classify_openai_error;

    use super::*;

    #[test]
    fn the_conflict_refusal_classifies_as_a_busy_slot_so_the_ladder_runs() {
        let message = active_response_conflict_message("resp_local_1");
        assert_eq!(
            classify_openai_error(&message),
            ErrorClass::ResponseSlotBusy,
            "a refusal the shipped corpus does not recognise stops the retry ladder"
        );
        let frame = events::error(ACTIVE_RESPONSE_CONFLICT_CODE, &message);
        assert!(via_realtime_openai::is_active_response_conflict(&frame));
    }

    #[test]
    fn the_cancel_refusal_classifies_as_suppressed_rather_than_shown() {
        assert_eq!(
            classify_openai_error(CANCEL_NOT_ACTIVE_MESSAGE),
            ErrorClass::NoActiveResponse
        );
        assert!(ErrorClass::NoActiveResponse.is_suppressed());
    }

    #[test]
    fn a_stage_failure_message_is_never_mistaken_for_a_busy_slot_or_a_fatal() {
        for stage in crate::stages::Stage::ALL {
            let error = StageError::new(stage, "the engine refused the graph");
            assert_eq!(
                classify_openai_error(&error.to_string()),
                ErrorClass::Other,
                "{stage}: a stage failure must not arm the busy ladder or block the connection"
            );
        }
    }

    #[test]
    fn a_user_text_item_is_recognised_and_nothing_else_is() {
        let protocol = via_realtime::ga_realtime_protocol();
        let item = via_realtime::RealtimeProtocol::user_text_item(&protocol, "hello there");
        assert_eq!(user_item_text(&item).as_deref(), Some("hello there"));

        for other in [
            json!({ "type": "message", "role": "assistant", "content": [] }),
            json!({ "type": "function_call_output", "call_id": "c", "output": "{}" }),
            json!({ "type": "message", "role": "user", "content": [] }),
            json!({ "type": "message", "role": "user", "content": [{ "type": "input_text", "text": "  " }] }),
            json!({ "type": "message", "role": "user" }),
            json!({}),
            json!("not an object"),
        ] {
            assert_eq!(user_item_text(&other), None, "{other}");
        }
    }

    #[test]
    fn a_multi_part_user_item_is_concatenated() {
        let item = json!({
            "type": "message",
            "role": "user",
            "content": [
                { "type": "input_text", "text": "turn on " },
                { "type": "input_text", "text": "the lights" },
            ],
        });
        assert_eq!(user_item_text(&item).as_deref(), Some("turn on the lights"));
    }

    #[test]
    fn a_matching_rate_pair_needs_no_converter_and_a_differing_one_does() {
        assert!(build_resampler(SampleRate::HZ_16000, SampleRate::HZ_16000).is_none());
        let converter = build_resampler(SampleRate::HZ_24000, SampleRate::HZ_16000)
            .expect("24 kHz to 16 kHz is a rate pair rubato accepts");
        assert!(!converter.is_pass_through());
    }

    #[test]
    fn the_frame_count_saturates_rather_than_wrapping() {
        assert_eq!(frames_of(0), 0);
        assert_eq!(frames_of(480), 480);
        assert_eq!(
            frames_of(usize::MAX),
            u64::try_from(usize::MAX).unwrap_or(u64::MAX)
        );
    }
}
