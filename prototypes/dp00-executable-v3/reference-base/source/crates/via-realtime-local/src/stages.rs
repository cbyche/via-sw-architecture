//! The four stages a cascade is made of, as traits.
//!
//! `docs/architecture.md` §7: *"the shipping local path is componentized"* —
//! sherpa-onnx's Silero VAD, its streaming ASR, `llama-cpp-2` on a Qwen3 GGUF,
//! and Kokoro/Piper TTS. Every one of those is a native C++ tree, and
//! `docs/adr/0001-placement.md` records that VIA's native dependencies are
//! exactly what keeps it out of ARGO's workspace. So the stage is a **trait**,
//! the trait is in the default build, and the engines are behind
//! `--features sherpa` and `--features llama`.
//!
//! The whole point is that [`crate::machine`] — the part that is hard, the part
//! that turns three request/response calls into one full-duplex interruptible
//! session — never names an engine. It drives these four traits, and CI drives
//! it through [`crate::scripted`].
//!
//! # The two rates
//!
//! | Direction | Rate | Why |
//! | --- | --- | --- |
//! | in | [`PIPELINE_INPUT_RATE`] | Silero VAD and every sherpa streaming ASR model are 16 kHz feature extractors, the same rate `via-wake-word` runs at |
//! | out | [`PIPELINE_OUTPUT_RATE`] | Kokoro-82M synthesizes at 24 kHz, which is also what every realtime client already plays |
//!
//! Both are [`via_audio::SampleRate`] constants rather than integers here, so a
//! stage that disagrees with the resampler in front of it is a type the caller
//! can compare rather than a number nobody checks. A rate mismatch does not
//! crash anything — it just quietly stops hearing the user.
//!
//! # Cancellation is a `drop`
//!
//! Barge-in has to cut speech mid-word and throw away a reasoning turn that is
//! still generating. Both of those are streams, and **dropping the stream is the
//! cancellation** — there is no `cancel()` on either trait, because a second way
//! to stop a turn is a second thing that can be forgotten.
//!
//! Two obligations follow, and they are contract:
//!
//! - **A dropped stream must not block.** A real engine generates on a blocking
//!   thread; it must observe the drop through a flag or a closed channel and
//!   unwind on its own time, never inside `Drop`.
//! - **A dropped stream must be inert.** Nothing may reach the machine after the
//!   drop, because the machine has already moved the turn on.

use std::fmt;

use futures::stream::BoxStream;
use serde_json::Value;
use via_audio::SampleRate;

/// The rate the VAD and the ASR consume, in hertz.
///
/// Not an upstream contract — upstream has no local pipeline. It is
/// [`SampleRate::HZ_16000`], read from `via-audio` rather than retyped for the
/// same reason `via-wake-word`'s feature extractor reads it: two copies of a
/// feature-extractor rate is a bug with no symptom.
pub const PIPELINE_INPUT_RATE: SampleRate = SampleRate::HZ_16000;

/// The rate the speaker produces, in hertz.
///
/// [`SampleRate::HZ_24000`] — Kokoro-82M's synthesis rate, and the rate every
/// OpenAI-Realtime client already plays, so the Gateway's `PlaybackCursor`
/// counts the same frames the device does.
pub const PIPELINE_OUTPUT_RATE: SampleRate = SampleRate::HZ_24000;

/// Which stage a failure came from.
///
/// The four names are the ones a person reads in
/// [`crate::LocalError::Stage`]'s sentence and the ones the `[local]` log
/// markers carry, so they are short and stable rather than prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    /// Voice activity detection.
    Vad,
    /// Streaming speech recognition.
    Asr,
    /// The reasoning turn.
    Reasoning,
    /// Speech synthesis.
    Tts,
}

impl Stage {
    /// Every stage, in pipeline order.
    pub const ALL: [Self; 4] = [Self::Vad, Self::Asr, Self::Reasoning, Self::Tts];

    /// The short name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Vad => "vad",
            Self::Asr => "asr",
            Self::Reasoning => "reasoning",
            Self::Tts => "tts",
        }
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One stage refused.
///
/// Deliberately a struct rather than an enum: a stage's failure modes belong to
/// the engine, and typing them here would make this crate's error surface a
/// union of four third-party libraries'. What the machine needs is *which* stage
/// and *what it said*, and that is exactly these two fields.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("the on-device {stage} stage failed: {detail}")]
pub struct StageError {
    /// Which stage.
    pub stage: Stage,
    /// What the engine reported.
    pub detail: String,
}

impl StageError {
    /// A failure in `stage`.
    #[must_use]
    pub fn new(stage: Stage, detail: impl Into<String>) -> Self {
        Self {
            stage,
            detail: detail.into(),
        }
    }
}

/// What the voice-activity detector says about one block of audio.
///
/// [`Started`](Self::Started) and [`Ended`](Self::Ended) are **edges**, not
/// levels: an implementation reports each exactly once per utterance, and
/// [`Speaking`](Self::Speaking) for every block in between. The machine keys
/// barge-in on the edge, so a detector that reported `Started` on every speech
/// block would cancel the same response over and over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SpeechEvent {
    /// No speech in this block, and none before it.
    #[default]
    Silence,
    /// Speech begins here.
    Started,
    /// Speech continues.
    Speaking,
    /// Speech ends here.
    Ended,
}

impl SpeechEvent {
    /// Whether this block carried speech.
    ///
    /// [`Ended`](Self::Ended) does *not* — it is the first block of the
    /// trailing silence, which is what makes the utterance's audio boundary the
    /// block before it.
    #[must_use]
    pub const fn is_voiced(self) -> bool {
        matches!(self, Self::Started | Self::Speaking)
    }
}

/// A streaming voice-activity detector.
///
/// # The contract implementations must keep
///
/// - **An empty slice is a no-op** and answers [`SpeechEvent::Silence`]. A
///   resampler that has not yet filled a block hands out empty slices routinely.
/// - **The edges are edges.** One [`Started`](SpeechEvent::Started) and one
///   [`Ended`](SpeechEvent::Ended) per utterance.
/// - **[`reset`](Self::reset) forgets the utterance**, so the next block can
///   open a new one.
///
/// All three are asserted against [`ScriptedVoiceActivity`](crate::ScriptedVoiceActivity)
/// as well as against the engine, because a double that gets them wrong tests
/// nothing.
pub trait VoiceActivity: Send + fmt::Debug {
    /// Feed one block of mono samples at [`sample_rate`](Self::sample_rate).
    ///
    /// # Errors
    ///
    /// [`StageError`] with [`Stage::Vad`] when the engine refuses the block.
    fn accept(&mut self, samples: &[f32]) -> Result<SpeechEvent, StageError>;

    /// Forget the utterance in progress.
    fn reset(&mut self);

    /// The rate `accept` expects.
    fn sample_rate(&self) -> SampleRate {
        PIPELINE_INPUT_RATE
    }
}

/// The running transcript of the utterance in progress.
///
/// The two fields are the shipped wire contract, not a choice:
/// `via-voice`'s `streaming_input_transcript` is `` `${text}${stash}`.trim() ``
/// (`input-transcript.mjs:6-9`), where `stash` is the recognizer's uncommitted
/// tail. Dropping the tail makes the running transcript lag a word behind what
/// the user just said, which is visible on screen.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TranscriptUpdate {
    /// The committed prefix.
    pub text: String,
    /// The uncommitted tail.
    pub stash: String,
}

impl TranscriptUpdate {
    /// A committed prefix with no tail.
    #[must_use]
    pub fn committed(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            stash: String::new(),
        }
    }

    /// A committed prefix and the recognizer's uncommitted tail.
    #[must_use]
    pub fn with_stash(text: impl Into<String>, stash: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            stash: stash.into(),
        }
    }

    /// Whether there is nothing to publish.
    ///
    /// The machine skips an empty update rather than emitting a transcription
    /// delta that says nothing, because every one of those is a frame the
    /// Gateway relays to every client.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty() && self.stash.is_empty()
    }

    /// What `via-voice` will render: `text` + `stash`, trimmed.
    #[must_use]
    pub fn rendered(&self) -> String {
        format!("{}{}", self.text, self.stash).trim().to_owned()
    }
}

/// A streaming speech recognizer.
///
/// # The contract implementations must keep
///
/// - **An empty slice is a no-op** and answers an empty [`TranscriptUpdate`].
/// - **[`accept`](Self::accept) is cumulative.** `text` is the whole committed
///   transcript of the utterance so far, not the delta since the last call —
///   that is what the wire contract carries.
/// - **[`finish`](Self::finish) may be called with nothing fed**, and answers
///   the empty string. A VAD edge with no audio behind it is an ordinary
///   occurrence.
/// - **[`finish`](Self::finish) leaves the recognizer ready** for the next
///   utterance, exactly as [`reset`](Self::reset) does.
pub trait Transcriber: Send + fmt::Debug {
    /// Feed one block of mono samples at [`sample_rate`](Self::sample_rate).
    ///
    /// # Errors
    ///
    /// [`StageError`] with [`Stage::Asr`].
    fn accept(&mut self, samples: &[f32]) -> Result<TranscriptUpdate, StageError>;

    /// Close the utterance and answer its final transcript.
    ///
    /// # Errors
    ///
    /// [`StageError`] with [`Stage::Asr`].
    fn finish(&mut self) -> Result<String, StageError>;

    /// Throw the utterance away without transcribing it.
    fn reset(&mut self);

    /// The rate `accept` expects.
    fn sample_rate(&self) -> SampleRate {
        PIPELINE_INPUT_RATE
    }
}

/// A tool call the reasoning stage authored.
///
/// `via-catalog`'s `local` family declares `function_calling: true`, and it is
/// true: `docs/architecture.md` §3 gives Layer 1 eight tools, and a local model
/// that could not call them would degrade the Gateway to `dictation` without
/// saying so. The three fields are what
/// `response.function_call_arguments.done` carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    /// The id the tool result must quote back.
    pub call_id: String,
    /// The tool name.
    pub name: String,
    /// The arguments, as a JSON-encoded **string** — the shape the wire carries
    /// and the shape `via-voice` parses.
    pub arguments: String,
}

impl ToolCall {
    /// A call with JSON-encoded `arguments`.
    #[must_use]
    pub fn new(
        call_id: impl Into<String>,
        name: impl Into<String>,
        arguments: impl Into<String>,
    ) -> Self {
        Self {
            call_id: call_id.into(),
            name: name.into(),
            arguments: arguments.into(),
        }
    }
}

/// One piece of a reasoning turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseDelta {
    /// More of the answer's text.
    Text(String),
    /// A tool call. The machine publishes it and ends the turn's text there.
    Tool(ToolCall),
}

/// A reasoning turn in progress.
///
/// Dropping it cancels the turn — see the module docs.
pub type ResponseStream = BoxStream<'static, Result<ResponseDelta, StageError>>;

/// One side of the conversation, as the reasoning stage is shown it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnRole {
    /// The user.
    User,
    /// The assistant.
    Assistant,
}

/// One prior message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnMessage {
    /// Who said it.
    pub role: TurnRole,
    /// What was said.
    pub text: String,
}

impl TurnMessage {
    /// A user message.
    #[must_use]
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: TurnRole::User,
            text: text.into(),
        }
    }

    /// An assistant message.
    #[must_use]
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: TurnRole::Assistant,
            text: text.into(),
        }
    }
}

/// What the reasoning stage is asked for.
///
/// [`instructions`](Self::instructions) is the whole reason this is a struct and
/// not a string. A realtime `response.create` may carry its own `instructions`
/// — that is how the Gateway makes the model read a finished background result
/// aloud without the result ever entering the conversation
/// (`conversation: 'none'`) — and a responder that ignored them would answer the
/// previous user turn again instead.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Turn {
    /// The session-level persona and rules.
    pub instructions: String,
    /// Per-response instructions from `response.create`, when there were any.
    pub response_instructions: Option<String>,
    /// The user's words for this turn, empty when the turn has no new input.
    pub transcript: String,
    /// The conversation so far, oldest first.
    pub history: Vec<TurnMessage>,
    /// The tool catalog, in the shape `session.update` carried it.
    pub tools: Vec<Value>,
}

/// The reasoning stage.
///
/// `&self` rather than `&mut self`: one loaded model serves every turn, and the
/// per-turn state is the [`ResponseStream`] it hands back. That is what makes
/// "cancel the in-flight turn" a drop of the stream rather than a mutation of a
/// shared object the machine would have to lock.
#[async_trait::async_trait]
pub trait Responder: Send + Sync + fmt::Debug {
    /// Begin a turn.
    ///
    /// # Errors
    ///
    /// [`StageError`] with [`Stage::Reasoning`] when the turn cannot be started
    /// at all. A failure *during* generation arrives on the stream instead.
    async fn respond(&self, turn: Turn) -> Result<ResponseStream, StageError>;
}

/// An utterance being synthesized, in PCM16 blocks at
/// [`Speaker::sample_rate`].
///
/// Dropping it cuts the utterance — see the module docs.
pub type SpeechStream = BoxStream<'static, Result<Vec<i16>, StageError>>;

/// The speech-synthesis stage.
#[async_trait::async_trait]
pub trait Speaker: Send + Sync + fmt::Debug {
    /// Synthesize `text` in `voice`.
    ///
    /// # Errors
    ///
    /// [`StageError`] with [`Stage::Tts`] when synthesis cannot be started.
    async fn speak(&self, text: &str, voice: &str) -> Result<SpeechStream, StageError>;

    /// The rate the chunks come at.
    fn sample_rate(&self) -> SampleRate {
        PIPELINE_OUTPUT_RATE
    }
}

/// The four stages, boxed.
///
/// A struct rather than four arguments, because the machine, the provider and
/// every test take the same four and an argument list of four boxed traits is a
/// swap waiting to happen.
pub struct Stages {
    /// Voice activity detection.
    pub voice_activity: Box<dyn VoiceActivity>,
    /// Streaming recognition.
    pub transcriber: Box<dyn Transcriber>,
    /// The reasoning turn.
    pub responder: std::sync::Arc<dyn Responder>,
    /// Speech synthesis.
    pub speaker: std::sync::Arc<dyn Speaker>,
}

impl fmt::Debug for Stages {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stages")
            .field("voice_activity", &self.voice_activity)
            .field("transcriber", &self.transcriber)
            .field("responder", &self.responder)
            .field("speaker", &self.speaker)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn the_two_rates_are_via_audios_constants_rather_than_numbers() {
        assert_eq!(PIPELINE_INPUT_RATE, SampleRate::HZ_16000);
        assert_eq!(PIPELINE_OUTPUT_RATE, SampleRate::HZ_24000);
    }

    #[test]
    fn only_the_two_speaking_events_are_voiced() {
        assert!(SpeechEvent::Started.is_voiced());
        assert!(SpeechEvent::Speaking.is_voiced());
        assert!(!SpeechEvent::Ended.is_voiced());
        assert!(!SpeechEvent::Silence.is_voiced());
        assert_eq!(SpeechEvent::default(), SpeechEvent::Silence);
    }

    #[test]
    fn the_stage_names_are_short_stable_and_unique() {
        let names: Vec<&str> = Stage::ALL.iter().map(|stage| stage.as_str()).collect();
        assert_eq!(names, ["vad", "asr", "reasoning", "tts"]);
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
    }

    #[test]
    fn a_stage_failure_reads_as_a_sentence_naming_the_stage() {
        let error = StageError::new(Stage::Tts, "voice pack `af_heart` is not installed");
        assert_eq!(
            error.to_string(),
            "the on-device tts stage failed: voice pack `af_heart` is not installed"
        );
    }

    #[test]
    fn a_transcript_update_renders_the_way_via_voice_will() {
        // `${text}${stash}`.trim() — the tail is concatenated, not dropped.
        let update = TranscriptUpdate::with_stash("turn on the ", "ligh");
        assert_eq!(update.rendered(), "turn on the ligh");
        assert!(!update.is_empty());

        assert!(TranscriptUpdate::default().is_empty());
        assert_eq!(TranscriptUpdate::default().rendered(), "");
        // A stash on its own is still something to publish.
        assert!(!TranscriptUpdate::with_stash("", "h").is_empty());
    }

    #[test]
    fn a_committed_update_has_no_tail() {
        let update = TranscriptUpdate::committed("done");
        assert_eq!(update.stash, "");
        assert_eq!(update.rendered(), "done");
    }
}
