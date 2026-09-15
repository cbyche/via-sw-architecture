//! A deterministic double for every stage.
//!
//! These ship in the **default** build, not behind `cfg(test)`, for the same two
//! reasons `via-wake-word`'s [`ScriptedDetector`] does. The Gateway above this
//! crate cannot exercise "the user spoke and the model answered" without stages
//! that can be told to do so, and a double behind `cfg(test)` is invisible to
//! it. And this is where the four stage contracts — an empty block is a no-op,
//! the VAD edges are edges, a dropped stream is the cancellation — are pinned in
//! a form the crate's own tests can assert with no ONNX runtime and no GGUF.
//!
//! **They are also the only part of this crate CI can ever run.** `--features
//! sherpa` needs a C toolchain and `--features llama` needs weights;
//! `cargo test -p via-realtime-local` needs neither, and it still drives the
//! whole duplex machine — barge-in, interruption, tool calls, cancellation and
//! every error path — because the machine only ever sees these four traits.
//!
//! # Nothing here is random, timed, or scheduled
//!
//! A script is a list. `accept` consumes one entry per **non-empty** block; a
//! stream yields its entries in order and then ends. [`ScriptedSpeaker`]'s PCM is
//! integer arithmetic, so the same script produces the same bytes on every run
//! and on every host. A "fires after roughly a second of audio" double
//! reproduces exactly the flakiness a double exists to remove.
//!
//! # Proving a cancellation happened
//!
//! [`stages`](crate::stages) makes dropping a stream *be* the cancellation,
//! which leaves a test with nothing to assert — a turn that was discarded and a
//! turn that never started look alike from outside. So both scripted streams
//! count their own drops into a shared [`CancelCount`], and
//! [`ScriptedResponder::stalling`] and [`ScriptedSpeaker::stalling`] produce a
//! stream that never ends on its own. A test that wants to prove barge-in
//! discarded an in-flight reasoning turn starts a stalling turn, barges in, and
//! asserts the count moved.
//!
//! [`ScriptedDetector`]: via_wake_word::ScriptedDetector

use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll};

use futures::Stream;
use via_audio::SampleRate;

use crate::stages::{
    PIPELINE_INPUT_RATE, PIPELINE_OUTPUT_RATE, Responder, ResponseDelta, ResponseStream, Speaker,
    SpeechEvent, SpeechStream, Stage, StageError, ToolCall, Transcriber, TranscriptUpdate, Turn,
    VoiceActivity,
};

/// How many scripted streams have been dropped.
///
/// Cheap to clone; every clone counts into the same total.
#[derive(Debug, Clone, Default)]
pub struct CancelCount(Arc<AtomicUsize>);

impl CancelCount {
    /// A counter at zero.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// How many streams have been dropped so far.
    #[must_use]
    pub fn get(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }

    fn record(&self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// A stream that yields a fixed list and then either ends or stalls forever.
struct ScriptStream<T> {
    items: VecDeque<Result<T, StageError>>,
    stall_at_end: bool,
    dropped: CancelCount,
}

impl<T: Unpin> Stream for ScriptStream<T> {
    type Item = Result<T, StageError>;

    fn poll_next(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match this.items.pop_front() {
            Some(item) => Poll::Ready(Some(item)),
            // Deliberately parked with no waker: a stalling stream models an
            // engine that is still thinking, and the only thing that ends it is
            // the machine dropping it. The task still wakes on its other
            // branches, which is what barge-in arrives on.
            None if this.stall_at_end => Poll::Pending,
            None => Poll::Ready(None),
        }
    }
}

impl<T> Drop for ScriptStream<T> {
    fn drop(&mut self) {
        self.dropped.record();
    }
}

fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    // A poisoned lock here means a test panicked while holding it; the recorded
    // calls are still readable and are exactly what that test needs to see.
    value
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

// ── voice activity ──────────────────────────────────────────────────────────

/// A [`VoiceActivity`] that answers from a script.
///
/// One entry per **non-empty** block; an empty block is a no-op and answers
/// [`SpeechEvent::Silence`] without consuming one. Once the script runs out it
/// answers `Silence` forever, so a test only writes the part it cares about.
#[derive(Debug)]
pub struct ScriptedVoiceActivity {
    steps: VecDeque<SpeechEvent>,
    sample_rate: SampleRate,
    failure: Option<String>,
    blocks: usize,
    samples: usize,
    empty_blocks: usize,
    resets: usize,
}

impl ScriptedVoiceActivity {
    /// A detector that hears nothing, ever.
    #[must_use]
    pub fn silent() -> Self {
        Self::new([])
    }

    /// A detector that follows `steps`, then answers `Silence` forever.
    #[must_use]
    pub fn new(steps: impl IntoIterator<Item = SpeechEvent>) -> Self {
        Self {
            steps: steps.into_iter().collect(),
            sample_rate: PIPELINE_INPUT_RATE,
            failure: None,
            blocks: 0,
            samples: 0,
            empty_blocks: 0,
            resets: 0,
        }
    }

    /// One utterance `blocks` blocks long: `Started`, then `Speaking`, then
    /// `Ended`.
    ///
    /// `blocks` counts the **voiced** blocks, so `utterance(1)` is a single
    /// `Started` followed by `Ended` — the shortest utterance a VAD can report,
    /// and the one that catches an off-by-one in the machine's audio
    /// accounting.
    #[must_use]
    pub fn utterance(blocks: usize) -> Self {
        let mut steps = vec![SpeechEvent::Started];
        steps.extend(std::iter::repeat_n(
            SpeechEvent::Speaking,
            blocks.saturating_sub(1),
        ));
        steps.push(SpeechEvent::Ended);
        Self::new(steps)
    }

    /// `count` utterances of `blocks` voiced blocks each, separated by one
    /// silent block.
    #[must_use]
    pub fn utterances(count: usize, blocks: usize) -> Self {
        let mut steps = Vec::new();
        for index in 0..count {
            if index > 0 {
                steps.push(SpeechEvent::Silence);
            }
            steps.push(SpeechEvent::Started);
            steps.extend(std::iter::repeat_n(
                SpeechEvent::Speaking,
                blocks.saturating_sub(1),
            ));
            steps.push(SpeechEvent::Ended);
        }
        Self::new(steps)
    }

    /// A detector whose engine refuses every non-empty block.
    #[must_use]
    pub fn failing(detail: impl Into<String>) -> Self {
        Self {
            failure: Some(detail.into()),
            ..Self::silent()
        }
    }

    /// Declare a different [`VoiceActivity::sample_rate`].
    #[must_use]
    pub fn at_rate(mut self, rate: SampleRate) -> Self {
        self.sample_rate = rate;
        self
    }

    /// Non-empty blocks accepted.
    #[must_use]
    pub fn blocks(&self) -> usize {
        self.blocks
    }

    /// Samples accepted, across every non-empty block.
    #[must_use]
    pub fn samples(&self) -> usize {
        self.samples
    }

    /// Empty blocks accepted, none of which consumed a step.
    #[must_use]
    pub fn empty_blocks(&self) -> usize {
        self.empty_blocks
    }

    /// How many times the detector was reset.
    #[must_use]
    pub fn resets(&self) -> usize {
        self.resets
    }

    /// Steps still to be answered.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.steps.len()
    }
}

impl VoiceActivity for ScriptedVoiceActivity {
    fn accept(&mut self, samples: &[f32]) -> Result<SpeechEvent, StageError> {
        if samples.is_empty() {
            self.empty_blocks += 1;
            return Ok(SpeechEvent::Silence);
        }
        if let Some(detail) = &self.failure {
            return Err(StageError::new(Stage::Vad, detail.clone()));
        }
        self.blocks += 1;
        self.samples += samples.len();
        Ok(self.steps.pop_front().unwrap_or(SpeechEvent::Silence))
    }

    fn reset(&mut self) {
        self.resets += 1;
    }

    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
}

// ── transcription ───────────────────────────────────────────────────────────

/// A [`Transcriber`] that answers from a script.
#[derive(Debug)]
pub struct ScriptedTranscriber {
    updates: VecDeque<TranscriptUpdate>,
    /// Whether the word queue has been loaded for the utterance in progress.
    ///
    /// Without it, a recognizer whose words have run out would re-load the same
    /// phrase on the next block and start the utterance over — which reads in a
    /// failing test as "the double repeats itself" rather than as the bug it is.
    armed: bool,
    transcripts: VecDeque<String>,
    accept_failure: Option<String>,
    finish_failure: Option<String>,
    blocks: usize,
    empty_blocks: usize,
    finishes: usize,
    resets: usize,
}

impl ScriptedTranscriber {
    /// A recognizer that hears nothing.
    #[must_use]
    pub fn deaf() -> Self {
        Self {
            updates: VecDeque::new(),
            armed: false,
            transcripts: VecDeque::new(),
            accept_failure: None,
            finish_failure: None,
            blocks: 0,
            empty_blocks: 0,
            finishes: 0,
            resets: 0,
        }
    }

    /// A recognizer that reveals `phrase` one word per block and answers
    /// `finish` with the whole thing.
    ///
    /// The word being revealed rides in
    /// [`stash`](TranscriptUpdate::stash) until the next block commits it, which
    /// is what a streaming recognizer actually does and what makes the
    /// `text` + `stash` contract exercised rather than merely declared.
    #[must_use]
    pub fn hearing(phrase: &str) -> Self {
        Self::hearing_each([phrase])
    }

    /// One utterance per phrase, in order.
    #[must_use]
    pub fn hearing_each<'a>(phrases: impl IntoIterator<Item = &'a str>) -> Self {
        let mut recognizer = Self::deaf();
        for phrase in phrases {
            recognizer.transcripts.push_back(phrase.trim().to_owned());
        }
        recognizer
    }

    /// A recognizer whose engine refuses every non-empty block.
    #[must_use]
    pub fn failing(detail: impl Into<String>) -> Self {
        Self {
            accept_failure: Some(detail.into()),
            ..Self::deaf()
        }
    }

    /// A recognizer that accepts audio and then refuses to close the
    /// utterance.
    #[must_use]
    pub fn failing_on_finish(detail: impl Into<String>) -> Self {
        Self {
            finish_failure: Some(detail.into()),
            ..Self::deaf()
        }
    }

    /// Non-empty blocks accepted.
    #[must_use]
    pub fn blocks(&self) -> usize {
        self.blocks
    }

    /// Empty blocks accepted, none of which produced an update.
    #[must_use]
    pub fn empty_blocks(&self) -> usize {
        self.empty_blocks
    }

    /// Utterances closed.
    #[must_use]
    pub fn finishes(&self) -> usize {
        self.finishes
    }

    /// Utterances thrown away.
    #[must_use]
    pub fn resets(&self) -> usize {
        self.resets
    }

    /// Load the word queue for the next utterance.
    fn arm(&mut self) {
        if self.armed {
            return;
        }
        self.armed = true;
        let Some(phrase) = self.transcripts.front() else {
            return;
        };
        let words: Vec<&str> = phrase.split_whitespace().collect();
        let mut committed = String::new();
        for (index, word) in words.iter().enumerate() {
            self.updates
                .push_back(TranscriptUpdate::with_stash(committed.clone(), *word));
            if index + 1 < words.len() {
                committed.push_str(word);
                committed.push(' ');
            }
        }
    }
}

impl Transcriber for ScriptedTranscriber {
    fn accept(&mut self, samples: &[f32]) -> Result<TranscriptUpdate, StageError> {
        if samples.is_empty() {
            self.empty_blocks += 1;
            return Ok(TranscriptUpdate::default());
        }
        if let Some(detail) = &self.accept_failure {
            return Err(StageError::new(Stage::Asr, detail.clone()));
        }
        self.blocks += 1;
        self.arm();
        if let Some(update) = self.updates.pop_front() {
            return Ok(update);
        }
        Ok(TranscriptUpdate::default())
    }

    fn finish(&mut self) -> Result<String, StageError> {
        self.finishes += 1;
        if let Some(detail) = &self.finish_failure {
            return Err(StageError::new(Stage::Asr, detail.clone()));
        }
        self.updates.clear();
        self.armed = false;
        Ok(self.transcripts.pop_front().unwrap_or_default())
    }

    fn reset(&mut self) {
        self.resets += 1;
        self.updates.clear();
        self.armed = false;
    }
}

// ── reasoning ───────────────────────────────────────────────────────────────

/// One scripted reasoning turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptedTurn {
    deltas: Vec<Result<ResponseDelta, StageError>>,
    stall_at_end: bool,
}

impl ScriptedTurn {
    /// A turn that says `text`, one delta per word, and ends.
    #[must_use]
    pub fn saying(text: &str) -> Self {
        let mut deltas = Vec::new();
        let words: Vec<&str> = text.split_inclusive(' ').collect();
        for word in words {
            deltas.push(Ok(ResponseDelta::Text(word.to_owned())));
        }
        Self {
            deltas,
            stall_at_end: false,
        }
    }

    /// A turn that calls a tool and ends.
    #[must_use]
    pub fn calling(call: ToolCall) -> Self {
        Self {
            deltas: vec![Ok(ResponseDelta::Tool(call))],
            stall_at_end: false,
        }
    }

    /// A turn built out of explicit deltas.
    #[must_use]
    pub fn deltas(deltas: impl IntoIterator<Item = Result<ResponseDelta, StageError>>) -> Self {
        Self {
            deltas: deltas.into_iter().collect(),
            stall_at_end: false,
        }
    }

    /// A turn that fails partway through generation.
    #[must_use]
    pub fn failing_midway(text: &str, detail: impl Into<String>) -> Self {
        let mut turn = Self::saying(text);
        turn.deltas
            .push(Err(StageError::new(Stage::Reasoning, detail)));
        turn
    }

    /// A turn that never finishes on its own.
    ///
    /// The only thing that ends it is the machine dropping the stream, which is
    /// what makes an in-flight cancellation observable.
    #[must_use]
    pub fn stalling() -> Self {
        Self {
            deltas: Vec::new(),
            stall_at_end: true,
        }
    }

    /// A turn that says `text` and then never finishes.
    #[must_use]
    pub fn saying_then_stalling(text: &str) -> Self {
        Self {
            stall_at_end: true,
            ..Self::saying(text)
        }
    }
}

/// A [`Responder`] that answers from a script.
#[derive(Debug)]
pub struct ScriptedResponder {
    turns: Mutex<VecDeque<ScriptedTurn>>,
    asked: Mutex<Vec<Turn>>,
    open_failure: Option<String>,
    cancelled: CancelCount,
}

impl ScriptedResponder {
    /// A responder that answers every turn with `text`.
    #[must_use]
    pub fn saying(text: &str) -> Self {
        Self::turns([ScriptedTurn::saying(text)])
    }

    /// A responder that answers each turn from the script, in order, and then
    /// answers with nothing at all.
    #[must_use]
    pub fn turns(turns: impl IntoIterator<Item = ScriptedTurn>) -> Self {
        Self {
            turns: Mutex::new(turns.into_iter().collect()),
            asked: Mutex::new(Vec::new()),
            open_failure: None,
            cancelled: CancelCount::new(),
        }
    }

    /// A responder whose every turn stalls until it is cancelled.
    #[must_use]
    pub fn stalling() -> Self {
        Self::turns([ScriptedTurn::stalling()])
    }

    /// A responder that cannot start a turn at all.
    #[must_use]
    pub fn failing(detail: impl Into<String>) -> Self {
        Self {
            open_failure: Some(detail.into()),
            ..Self::turns([])
        }
    }

    /// Every turn it was asked for, in order.
    #[must_use]
    pub fn asked(&self) -> Vec<Turn> {
        lock(&self.asked).clone()
    }

    /// How many turns it was asked for.
    #[must_use]
    pub fn turn_count(&self) -> usize {
        lock(&self.asked).len()
    }

    /// How many of its streams were dropped.
    #[must_use]
    pub fn cancellations(&self) -> CancelCount {
        self.cancelled.clone()
    }
}

#[async_trait::async_trait]
impl Responder for ScriptedResponder {
    async fn respond(&self, turn: Turn) -> Result<ResponseStream, StageError> {
        lock(&self.asked).push(turn);
        if let Some(detail) = &self.open_failure {
            return Err(StageError::new(Stage::Reasoning, detail.clone()));
        }
        // The last scripted turn repeats, so a test that scripts one answer and
        // drives three turns gets three answers rather than two silences.
        let script = {
            let mut turns = lock(&self.turns);
            match turns.len() {
                0 => ScriptedTurn::deltas([]),
                1 => turns
                    .front()
                    .cloned()
                    .unwrap_or_else(|| ScriptedTurn::deltas([])),
                _ => turns
                    .pop_front()
                    .unwrap_or_else(|| ScriptedTurn::deltas([])),
            }
        };
        Ok(Box::pin(ScriptStream {
            items: script.deltas.into_iter().collect(),
            stall_at_end: script.stall_at_end,
            dropped: self.cancelled.clone(),
        }))
    }
}

// ── speech ──────────────────────────────────────────────────────────────────

/// Samples in one scripted speech chunk: 20 ms at
/// [`PIPELINE_OUTPUT_RATE`].
///
/// Read from `via-audio` rather than written as `480`, so a chunk is a block of
/// audio rather than a number.
pub const SPEECH_CHUNK_SAMPLES: usize = PIPELINE_OUTPUT_RATE.playback_block_frames();

/// Characters of text per synthesized chunk.
///
/// Arbitrary but fixed: it is what makes "a longer sentence produces more audio"
/// true in the doubles, which is what the barge-in tests need in order to cut
/// something in the middle.
pub const SPEECH_CHARS_PER_CHUNK: usize = 8;

/// One synthesis the speaker was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpokenUtterance {
    /// The text.
    pub text: String,
    /// The voice it was asked for in.
    pub voice: String,
}

/// A [`Speaker`] that synthesizes a deterministic ramp.
///
/// No RNG, no wall clock, no float arithmetic: sample *n* of chunk *c* is
/// `(c * SPEECH_CHUNK_SAMPLES + n) % 1000` as an `i16`, so the same text
/// produces the same bytes on every host and a test can assert the audio it
/// received rather than only its length.
#[derive(Debug)]
pub struct ScriptedSpeaker {
    spoken: Mutex<Vec<SpokenUtterance>>,
    open_failure: Option<String>,
    stream_failure: Option<String>,
    stall_at_end: bool,
    sample_rate: SampleRate,
    cancelled: CancelCount,
}

impl Default for ScriptedSpeaker {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptedSpeaker {
    /// A speaker that synthesizes every utterance it is given.
    #[must_use]
    pub fn new() -> Self {
        Self {
            spoken: Mutex::new(Vec::new()),
            open_failure: None,
            stream_failure: None,
            stall_at_end: false,
            sample_rate: PIPELINE_OUTPUT_RATE,
            cancelled: CancelCount::new(),
        }
    }

    /// A speaker that cannot start synthesis.
    #[must_use]
    pub fn failing(detail: impl Into<String>) -> Self {
        Self {
            open_failure: Some(detail.into()),
            ..Self::new()
        }
    }

    /// A speaker that starts and then fails partway through the utterance.
    #[must_use]
    pub fn failing_midway(detail: impl Into<String>) -> Self {
        Self {
            stream_failure: Some(detail.into()),
            ..Self::new()
        }
    }

    /// A speaker whose utterances never end on their own.
    ///
    /// What barge-in has to cut: an utterance still streaming when the user
    /// starts talking over it.
    #[must_use]
    pub fn stalling() -> Self {
        Self {
            stall_at_end: true,
            ..Self::new()
        }
    }

    /// Declare a different [`Speaker::sample_rate`].
    #[must_use]
    pub fn at_rate(mut self, rate: SampleRate) -> Self {
        self.sample_rate = rate;
        self
    }

    /// Every utterance it was asked for, in order.
    #[must_use]
    pub fn spoken(&self) -> Vec<SpokenUtterance> {
        lock(&self.spoken).clone()
    }

    /// The text of every utterance it was asked for.
    #[must_use]
    pub fn spoken_text(&self) -> Vec<String> {
        lock(&self.spoken)
            .iter()
            .map(|utterance| utterance.text.clone())
            .collect()
    }

    /// How many of its streams were dropped.
    #[must_use]
    pub fn cancellations(&self) -> CancelCount {
        self.cancelled.clone()
    }

    /// The chunks `text` synthesizes to.
    ///
    /// Public so a test can assert the exact audio a turn produced without
    /// restating the ramp.
    #[must_use]
    pub fn chunks_for(text: &str) -> Vec<Vec<i16>> {
        let count = text.chars().count().div_ceil(SPEECH_CHARS_PER_CHUNK).max(1);
        (0..count)
            .map(|chunk| {
                (0..SPEECH_CHUNK_SAMPLES)
                    .map(|sample| {
                        let index = chunk * SPEECH_CHUNK_SAMPLES + sample;
                        // `% 1000` keeps every value inside `i16` with no cast
                        // that could wrap, and `try_from` on a bounded value
                        // cannot fail — the fallback keeps this total.
                        i16::try_from(index % 1_000).unwrap_or(0)
                    })
                    .collect()
            })
            .collect()
    }

    /// How many samples `text` synthesizes to.
    #[must_use]
    pub fn samples_for(text: &str) -> usize {
        Self::chunks_for(text).iter().map(Vec::len).sum()
    }
}

#[async_trait::async_trait]
impl Speaker for ScriptedSpeaker {
    async fn speak(&self, text: &str, voice: &str) -> Result<SpeechStream, StageError> {
        lock(&self.spoken).push(SpokenUtterance {
            text: text.to_owned(),
            voice: voice.to_owned(),
        });
        if let Some(detail) = &self.open_failure {
            return Err(StageError::new(Stage::Tts, detail.clone()));
        }
        let mut items: VecDeque<Result<Vec<i16>, StageError>> =
            Self::chunks_for(text).into_iter().map(Ok).collect();
        if let Some(detail) = &self.stream_failure {
            // One chunk, then the failure: a synthesis that dies mid-utterance
            // has already put audio on the wire, and the machine has to close
            // it out rather than pretend it never spoke.
            items.truncate(1);
            items.push_back(Err(StageError::new(Stage::Tts, detail.clone())));
        }
        Ok(Box::pin(ScriptStream {
            items,
            stall_at_end: self.stall_at_end,
            dropped: self.cancelled.clone(),
        }))
    }

    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt as _;
    use pretty_assertions::assert_eq;

    use super::*;

    const BLOCK: [f32; 4] = [0.0, 0.1, -0.1, 0.0];

    #[test]
    fn an_empty_block_is_a_no_op_for_both_streaming_stages() {
        let mut vad = ScriptedVoiceActivity::utterance(2);
        assert_eq!(vad.accept(&[]), Ok(SpeechEvent::Silence));
        assert_eq!(vad.empty_blocks(), 1);
        assert_eq!(vad.blocks(), 0);
        assert_eq!(vad.remaining(), 3, "an empty block consumed a step");

        let mut asr = ScriptedTranscriber::hearing("hello there");
        assert_eq!(asr.accept(&[]), Ok(TranscriptUpdate::default()));
        assert_eq!(asr.empty_blocks(), 1);
        assert_eq!(asr.blocks(), 0);
    }

    #[test]
    fn a_scripted_utterance_reports_one_start_and_one_end() {
        let mut vad = ScriptedVoiceActivity::utterance(3);
        let mut seen = Vec::new();
        for _ in 0..6 {
            seen.push(vad.accept(&BLOCK).expect("scripted"));
        }
        assert_eq!(
            seen,
            [
                SpeechEvent::Started,
                SpeechEvent::Speaking,
                SpeechEvent::Speaking,
                SpeechEvent::Ended,
                SpeechEvent::Silence,
                SpeechEvent::Silence,
            ]
        );
        assert_eq!(vad.blocks(), 6);
        assert_eq!(vad.samples(), 6 * BLOCK.len());
    }

    #[test]
    fn the_shortest_utterance_is_one_voiced_block() {
        let mut vad = ScriptedVoiceActivity::utterance(1);
        assert_eq!(vad.accept(&BLOCK), Ok(SpeechEvent::Started));
        assert_eq!(vad.accept(&BLOCK), Ok(SpeechEvent::Ended));
        assert_eq!(vad.accept(&BLOCK), Ok(SpeechEvent::Silence));
    }

    #[test]
    fn two_utterances_are_separated_by_silence() {
        let mut vad = ScriptedVoiceActivity::utterances(2, 1);
        let seen: Vec<SpeechEvent> = (0..5)
            .map(|_| vad.accept(&BLOCK).expect("scripted"))
            .collect();
        assert_eq!(
            seen,
            [
                SpeechEvent::Started,
                SpeechEvent::Ended,
                SpeechEvent::Silence,
                SpeechEvent::Started,
                SpeechEvent::Ended,
            ]
        );
    }

    #[test]
    fn a_failing_detector_names_its_stage() {
        let mut vad = ScriptedVoiceActivity::failing("onnxruntime refused the graph");
        assert_eq!(
            vad.accept(&BLOCK),
            Err(StageError::new(Stage::Vad, "onnxruntime refused the graph"))
        );
        // An empty block is a no-op even for a failing detector.
        assert_eq!(vad.accept(&[]), Ok(SpeechEvent::Silence));
    }

    #[test]
    fn a_recognizer_reveals_one_word_per_block_with_the_tail_in_stash() {
        let mut asr = ScriptedTranscriber::hearing("turn on the lights");
        assert_eq!(
            asr.accept(&BLOCK),
            Ok(TranscriptUpdate::with_stash("", "turn"))
        );
        assert_eq!(
            asr.accept(&BLOCK),
            Ok(TranscriptUpdate::with_stash("turn ", "on"))
        );
        assert_eq!(
            asr.accept(&BLOCK),
            Ok(TranscriptUpdate::with_stash("turn on ", "the"))
        );
        assert_eq!(
            asr.accept(&BLOCK),
            Ok(TranscriptUpdate::with_stash("turn on the ", "lights"))
        );
        // Out of words: nothing more to publish.
        assert_eq!(asr.accept(&BLOCK), Ok(TranscriptUpdate::default()));
        assert_eq!(asr.finish(), Ok("turn on the lights".to_owned()));
        assert_eq!(asr.finishes(), 1);
    }

    #[test]
    fn each_scripted_phrase_is_one_utterance() {
        let mut asr = ScriptedTranscriber::hearing_each(["first", "second"]);
        assert_eq!(
            asr.accept(&BLOCK),
            Ok(TranscriptUpdate::with_stash("", "first"))
        );
        assert_eq!(asr.finish(), Ok("first".to_owned()));
        assert_eq!(
            asr.accept(&BLOCK),
            Ok(TranscriptUpdate::with_stash("", "second"))
        );
        assert_eq!(asr.finish(), Ok("second".to_owned()));
        assert_eq!(asr.finish(), Ok(String::new()), "the script ran out");
    }

    #[test]
    fn finishing_with_nothing_fed_is_an_ordinary_empty_transcript() {
        let mut asr = ScriptedTranscriber::deaf();
        assert_eq!(asr.finish(), Ok(String::new()));
    }

    #[test]
    fn a_recognizer_can_fail_on_a_block_or_on_the_close() {
        let mut on_block = ScriptedTranscriber::failing("decoder overflow");
        assert_eq!(
            on_block.accept(&BLOCK),
            Err(StageError::new(Stage::Asr, "decoder overflow"))
        );

        let mut on_close = ScriptedTranscriber::failing_on_finish("endpoint not reached");
        assert_eq!(on_close.accept(&BLOCK), Ok(TranscriptUpdate::default()));
        assert_eq!(
            on_close.finish(),
            Err(StageError::new(Stage::Asr, "endpoint not reached"))
        );
    }

    #[tokio::test]
    async fn a_scripted_turn_yields_its_deltas_and_ends() {
        let responder = ScriptedResponder::saying("On it. ");
        let mut stream = responder
            .respond(Turn::default())
            .await
            .expect("a scripted turn opens");
        let mut seen = Vec::new();
        while let Some(delta) = stream.next().await {
            seen.push(delta.expect("scripted"));
        }
        assert_eq!(
            seen,
            [
                ResponseDelta::Text("On ".to_owned()),
                ResponseDelta::Text("it. ".to_owned()),
            ]
        );
        assert_eq!(responder.turn_count(), 1);
    }

    #[tokio::test]
    async fn the_last_scripted_turn_repeats() {
        let responder = ScriptedResponder::saying("Yes.");
        for _ in 0..3 {
            let mut stream = responder.respond(Turn::default()).await.expect("opens");
            assert!(stream.next().await.is_some());
        }
        assert_eq!(responder.turn_count(), 3);
    }

    #[tokio::test]
    async fn dropping_a_stream_is_counted_as_a_cancellation() {
        let responder = ScriptedResponder::stalling();
        let cancelled = responder.cancellations();
        assert_eq!(cancelled.get(), 0);
        {
            let _stream = responder.respond(Turn::default()).await.expect("opens");
        }
        assert_eq!(cancelled.get(), 1);
    }

    #[tokio::test]
    async fn a_stalling_turn_never_yields_on_its_own() {
        // The bound is the point: the property is "this does not terminate",
        // so it is asserted with a deadline rather than by waiting.
        let responder = ScriptedResponder::stalling();
        let mut stream = responder.respond(Turn::default()).await.expect("opens");
        let polled =
            tokio::time::timeout(std::time::Duration::from_millis(50), stream.next()).await;
        assert!(polled.is_err(), "a stalling turn must not end by itself");
    }

    #[tokio::test]
    async fn a_responder_can_refuse_to_start() {
        let responder = ScriptedResponder::failing("no GGUF loaded");
        assert_eq!(
            responder.respond(Turn::default()).await.err(),
            Some(StageError::new(Stage::Reasoning, "no GGUF loaded"))
        );
        assert_eq!(responder.turn_count(), 1, "the ask is still recorded");
    }

    #[tokio::test]
    async fn the_speaker_records_the_voice_and_synthesizes_a_deterministic_ramp() {
        let speaker = ScriptedSpeaker::new();
        let mut stream = speaker.speak("hello", "af_heart").await.expect("opens");
        let mut chunks = Vec::new();
        while let Some(chunk) = stream.next().await {
            chunks.push(chunk.expect("scripted"));
        }
        assert_eq!(chunks, ScriptedSpeaker::chunks_for("hello"));
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), SPEECH_CHUNK_SAMPLES);
        assert_eq!(
            speaker.spoken(),
            [SpokenUtterance {
                text: "hello".to_owned(),
                voice: "af_heart".to_owned(),
            }]
        );
    }

    #[test]
    fn a_longer_utterance_synthesizes_to_more_chunks_and_is_stable() {
        assert_eq!(ScriptedSpeaker::chunks_for("").len(), 1);
        assert_eq!(ScriptedSpeaker::chunks_for("12345678").len(), 1);
        assert_eq!(ScriptedSpeaker::chunks_for("123456789").len(), 2);
        assert_eq!(
            ScriptedSpeaker::chunks_for("a longer sentence"),
            ScriptedSpeaker::chunks_for("a longer sentence"),
            "the ramp is a function of the text and nothing else"
        );
        assert_eq!(
            ScriptedSpeaker::samples_for("123456789"),
            2 * SPEECH_CHUNK_SAMPLES
        );
    }

    #[tokio::test]
    async fn a_speaker_can_fail_to_start_or_fail_after_it_has_spoken() {
        let speaker = ScriptedSpeaker::failing("voice pack missing");
        assert_eq!(
            speaker.speak("hi", "af_heart").await.err(),
            Some(StageError::new(Stage::Tts, "voice pack missing"))
        );

        let speaker = ScriptedSpeaker::failing_midway("phonemizer died");
        let mut stream = speaker
            .speak("a longer one", "af_heart")
            .await
            .expect("opens");
        assert!(stream.next().await.is_some_and(|chunk| chunk.is_ok()));
        assert_eq!(
            stream.next().await,
            Some(Err(StageError::new(Stage::Tts, "phonemizer died")))
        );
        assert_eq!(stream.next().await, None);
    }

    #[test]
    fn the_chunk_size_is_via_audios_playback_block() {
        assert_eq!(
            SPEECH_CHUNK_SAMPLES,
            PIPELINE_OUTPUT_RATE.playback_block_frames()
        );
    }
}
