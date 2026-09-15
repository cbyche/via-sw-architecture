//! Deterministic test doubles: a detector that never guesses, and a fetcher
//! that never opens a socket.
//!
//! These ship in the **default** build, not behind `cfg(test)`. Two reasons.
//! The wake-word pipeline is consumed by `via-voice` and `via-app`, and neither
//! can exercise "the user said the phrase" without a detector that can be told
//! to say so; a double behind `cfg(test)` is invisible to them. And the double
//! is where the [`WakeWordDetector`] contract — an empty slice is a no-op, a
//! match resets the stream — is pinned in a form the crate's own tests can
//! assert without an ONNX runtime.
//!
//! Nothing here is random, timed, or scheduled. A script is a list; `accept`
//! consumes one entry per non-empty chunk; the answer is whatever the entry
//! says. That is the whole model, and it is deliberate — a "fires after roughly
//! a second of audio" double reproduces the flakiness it exists to remove.

use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard};

use async_trait::async_trait;
use via_audio::SampleRate;

use crate::detect::{Detection, WakeWordDetector};
use crate::error::{Result, WakeWordError};
use crate::install::{FetchResponse, ModelFetch};

/// One entry in a [`ScriptedDetector`]'s script.
#[derive(Clone, Debug, PartialEq)]
pub enum ScriptStep {
    /// This chunk contains no wake word.
    Silence,
    /// This chunk matches, and reports this.
    Detect(Detection),
}

impl ScriptStep {
    /// A match on `keyword`, with no token detail.
    #[must_use]
    pub fn detect(keyword: impl Into<String>) -> Self {
        Self::Detect(Detection::new(keyword))
    }
}

/// A [`WakeWordDetector`] that answers from a script.
///
/// # What it counts, and why
///
/// It records chunks, samples, empty chunks and resets. Those counters are how
/// the tests around [`WakeWordStream`](crate::WakeWordStream) assert things
/// that are otherwise invisible: that a 48 kHz stream reaches the detector at
/// 16 kHz with the right number of samples, that a torn PCM frame is carried
/// rather than dropped, and that a resampler which has not yet filled a chunk
/// does not hand the engine an empty slice.
#[derive(Debug)]
pub struct ScriptedDetector {
    steps: VecDeque<ScriptStep>,
    sample_rate: SampleRate,
    chunks: usize,
    samples: usize,
    empty_chunks: usize,
    resets: usize,
    last_chunk: Vec<f32>,
}

impl ScriptedDetector {
    /// A detector that answers `Silence` forever.
    #[must_use]
    pub fn silent() -> Self {
        Self::new([])
    }

    /// A detector that follows `steps`, then answers `Silence` forever.
    #[must_use]
    pub fn new(steps: impl IntoIterator<Item = ScriptStep>) -> Self {
        Self {
            steps: steps.into_iter().collect(),
            sample_rate: SampleRate::HZ_16000,
            chunks: 0,
            samples: 0,
            empty_chunks: 0,
            resets: 0,
            last_chunk: Vec::new(),
        }
    }

    /// A detector that matches `keyword` on its first non-empty chunk.
    #[must_use]
    pub fn detecting(keyword: impl Into<String>) -> Self {
        Self::new([ScriptStep::detect(keyword)])
    }

    /// A detector that is silent for `chunks` chunks and then matches.
    #[must_use]
    pub fn after(chunks: usize, detection: Detection) -> Self {
        let mut steps: Vec<ScriptStep> = vec![ScriptStep::Silence; chunks];
        steps.push(ScriptStep::Detect(detection));
        Self::new(steps)
    }

    /// Declare a different [`WakeWordDetector::sample_rate`].
    ///
    /// Only useful for proving that
    /// [`WakeWordStream`](crate::WakeWordStream) builds its resampler from the
    /// detector's rate rather than assuming 16 kHz.
    #[must_use]
    pub fn at_rate(mut self, rate: SampleRate) -> Self {
        self.sample_rate = rate;
        self
    }

    /// Non-empty chunks accepted.
    #[must_use]
    pub fn chunks(&self) -> usize {
        self.chunks
    }

    /// Samples accepted, across all non-empty chunks.
    #[must_use]
    pub fn samples(&self) -> usize {
        self.samples
    }

    /// Empty chunks offered. A caller that respects the contract sends none.
    #[must_use]
    pub fn empty_chunks(&self) -> usize {
        self.empty_chunks
    }

    /// Resets, counting the self-reset a match performs.
    #[must_use]
    pub fn resets(&self) -> usize {
        self.resets
    }

    /// Script entries not yet consumed.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.steps.len()
    }

    /// The most recent non-empty chunk, exactly as the detector received it.
    ///
    /// Bounded to one chunk on purpose: a double that kept every sample of a
    /// live microphone would grow without limit, and one chunk is what the
    /// interesting assertions need — the `f32` values a PCM16 payload decoded
    /// to, and the length a resampler produced.
    #[must_use]
    pub fn last_chunk(&self) -> &[f32] {
        &self.last_chunk
    }
}

impl WakeWordDetector for ScriptedDetector {
    fn accept(&mut self, samples: &[f32]) -> Option<Detection> {
        if samples.is_empty() {
            // The contract: an empty chunk consumes no script entry and cannot
            // fire. Counted, so a caller that sends one can be caught.
            self.empty_chunks += 1;
            return None;
        }
        self.chunks += 1;
        self.samples += samples.len();
        self.last_chunk.clear();
        self.last_chunk.extend_from_slice(samples);
        match self.steps.pop_front() {
            Some(ScriptStep::Detect(detection)) => {
                // The contract: a match resets the stream, so one utterance
                // cannot fire twice.
                self.resets += 1;
                Some(detection)
            }
            Some(ScriptStep::Silence) | None => None,
        }
    }

    fn reset(&mut self) {
        self.resets += 1;
    }

    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
}

/// One entry in a [`ScriptedFetch`]'s script.
#[derive(Clone, Debug, PartialEq, Eq)]
enum FetchStep {
    /// Answer with this response.
    Answer(FetchResponse),
    /// Fail the request itself, with this detail.
    Fail(String),
}

/// A [`ModelFetch`] that answers from a script and records what it was asked.
#[derive(Debug, Default)]
pub struct ScriptedFetch {
    state: Mutex<FetchState>,
}

#[derive(Debug, Default)]
struct FetchState {
    steps: VecDeque<FetchStep>,
    requested: Vec<String>,
}

impl ScriptedFetch {
    /// A fetcher with an empty script. Every call is refused.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A fetcher that serves `body` once and is then exhausted.
    #[must_use]
    pub fn serving(body: Vec<u8>) -> Self {
        let fetcher = Self::new();
        fetcher.push_ok(body);
        fetcher
    }

    /// Queue a 200 carrying `body`.
    pub fn push_ok(&self, body: Vec<u8>) {
        self.push(FetchStep::Answer(FetchResponse::ok(body)));
    }

    /// Queue a bodiless answer with this status.
    ///
    /// Use `200` for the "2xx with no body" branch upstream checks separately
    /// (`model-manager.mjs:51`).
    pub fn push_status(&self, status: u16) {
        self.push(FetchStep::Answer(FetchResponse::status(status)));
    }

    /// Queue a whole [`FetchResponse`].
    pub fn push_response(&self, response: FetchResponse) {
        self.push(FetchStep::Answer(response));
    }

    /// Queue a transport failure — no answer at all.
    pub fn push_failure(&self, detail: impl Into<String>) {
        self.push(FetchStep::Fail(detail.into()));
    }

    /// Every URL asked for, in order.
    #[must_use]
    pub fn requested(&self) -> Vec<String> {
        lock(&self.state).requested.clone()
    }

    /// How many requests were made.
    #[must_use]
    pub fn requests(&self) -> usize {
        lock(&self.state).requested.len()
    }

    /// Script entries not yet consumed.
    #[must_use]
    pub fn remaining(&self) -> usize {
        lock(&self.state).steps.len()
    }

    fn push(&self, step: FetchStep) {
        lock(&self.state).steps.push_back(step);
    }
}

#[async_trait]
impl ModelFetch for ScriptedFetch {
    async fn fetch(&self, url: &str) -> Result<FetchResponse> {
        let step = {
            let mut state = lock(&self.state);
            state.requested.push(url.to_owned());
            state.steps.pop_front()
        };
        match step {
            Some(FetchStep::Answer(response)) => Ok(response),
            Some(FetchStep::Fail(detail)) => Err(WakeWordError::Fetch {
                url: url.to_owned(),
                detail,
            }),
            // An exhausted script is a test asking for a request it did not
            // arrange, which is a defect in the test rather than a plausible
            // network answer. It is reported as a transport failure so the
            // message names the URL that was not expected.
            None => Err(WakeWordError::Fetch {
                url: url.to_owned(),
                detail: "the scripted fetch has no response left".to_owned(),
            }),
        }
    }
}

/// Lock, treating a poisoned mutex as a live one: the state is a queue and a
/// log, and an unrelated panic must not make every later call fail.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
