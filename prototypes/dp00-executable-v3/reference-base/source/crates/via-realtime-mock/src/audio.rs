//! Canned PCM, so the mock's speech has real sample counts.
//!
//! The Injection Gate's blocking predicate is
//! `userSpeaking || turnPending || audioResponses.nonEmpty`
//! (`docs/architecture.md` §3, §11), and its third term is
//! [`via_audio::PlaybackCursor::is_draining`] — which counts *frames*. A mock
//! that streamed the string `"audio"` in a `response.audio.delta` would let
//! every gate test pass with no audio in it at all, so the mock streams real
//! base64 PCM16 whose frame count is arithmetic a test can predict.
//!
//! Nothing here duplicates [`via_audio`]: [`CannedAudio`] is a [`WavAudio`] plus
//! the two things a realtime socket adds — base64, and chunking at the block
//! cadence.
//!
//! # Determinism
//!
//! [`CannedAudio::tone`] is integer arithmetic end to end. A sine would be one
//! line shorter and would make the fixture depend on the platform's libm in its
//! last unit in the last place, which is exactly the kind of "passes here, fails
//! in CI" this crate exists to remove.
//!
//! ```
//! use std::time::Duration;
//!
//! use via_audio::{PlaybackCursor, SampleRate};
//! use via_realtime_mock::CannedAudio;
//!
//! let speech = CannedAudio::tone(SampleRate::HZ_24000, Duration::from_millis(200));
//! assert_eq!(speech.frames(), 4_800);
//! assert_eq!(speech.duration(), Duration::from_millis(200));
//!
//! // Ten 20 ms `response.audio.delta` frames, and a cursor that drains them.
//! let deltas = speech.audio_deltas();
//! assert_eq!(deltas.len(), 10);
//!
//! let mut cursor = PlaybackCursor::new(SampleRate::HZ_24000);
//! for chunk in speech.chunks() {
//!     cursor.enqueue(chunk.frames as u64);
//! }
//! assert!(cursor.is_draining());
//! cursor.advance(speech.frames() as u64);
//! assert!(!cursor.is_draining());
//! assert_eq!(cursor.audio_end_ms(), 200);
//! ```

use std::path::Path;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::Value;
use via_audio::{BLOCK_MILLIS, ChannelCount, SampleRate, WavAudio, read_wav, read_wav_file};

use crate::error::MockError;
use crate::script::events;

/// Peak amplitude of [`CannedAudio::tone`], in PCM16 units.
///
/// About −8.7 dBFS: loud enough that a fixture written to disk is audible, far
/// enough from the rails that no boundary case in [`via_audio::sample`] is
/// silently exercised by accident.
pub const TONE_AMPLITUDE: i16 = 12_000;

/// Frequency of [`CannedAudio::tone`], in hertz.
///
/// 200 Hz divides all three of `via-audio`'s rates exactly (80, 120 and 240
/// frames per cycle), so the waveform has no rounding tail at any of them.
pub const TONE_HZ: u32 = 200;

/// The base64 codec a realtime socket uses in both directions.
///
/// Standard alphabet with padding — `Buffer.from(bytes).toString('base64')` on
/// the upstream side, and what every OpenAI-Realtime-shaped provider sends back
/// in `response.audio.delta`.
fn codec() -> &'static base64::engine::general_purpose::GeneralPurpose {
    &STANDARD
}

/// Encode PCM16 bytes the way a realtime socket carries them.
#[must_use]
pub fn encode_audio(pcm16le: &[u8]) -> String {
    codec().encode(pcm16le)
}

/// Decode a realtime audio frame back to PCM16 bytes.
///
/// # Errors
///
/// [`MockError::NotBase64`] when the frame is not base64.
pub fn decode_audio(base64: &str) -> Result<Vec<u8>, MockError> {
    codec()
        .decode(base64)
        .map_err(|error| MockError::NotBase64 {
            detail: error.to_string(),
        })
}

/// One `response.audio.delta` worth of PCM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioChunk {
    /// The base64 PCM16 payload, exactly as it goes in `delta`.
    pub base64: String,
    /// How many frames it carries.
    ///
    /// What a [`via_audio::PlaybackCursor`] is told to enqueue.
    pub frames: usize,
}

impl AudioChunk {
    /// How long this chunk plays for at `rate`.
    #[must_use]
    pub fn duration(&self, rate: SampleRate) -> Duration {
        rate.frames_to_duration(self.frames as u64)
    }
}

/// Deterministic PCM16 the mock can stream as model speech.
///
/// The same three fields [`WavAudio`] carries — rate, channel count,
/// interleaved samples — plus the two things a realtime socket adds. It holds
/// them itself rather than wrapping a `WavAudio` for one reason: `WavAudio::new`
/// is fallible, and [`silence`](Self::silence) and [`tone`](Self::tone) are
/// not. Converting in either direction is [`from_wav_audio`](Self::from_wav_audio)
/// and [`to_wav_audio`](Self::to_wav_audio).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CannedAudio {
    rate: SampleRate,
    channels: ChannelCount,
    samples: Vec<i16>,
}

impl CannedAudio {
    /// The rate a realtime provider's output arrives at.
    ///
    /// External contract — [`via_audio::SampleRate::HZ_24000`], which is
    /// `providers/dashscope.mjs:48` and `providers/s2s.mjs:12`.
    pub const DEFAULT_RATE: SampleRate = SampleRate::HZ_24000;

    /// Wrap already-decoded interleaved PCM16.
    ///
    /// # Errors
    ///
    /// [`MockError::Audio`] when `samples` is not a whole number of frames — a
    /// torn interleaved buffer means channel phase was lost, and every frame
    /// after it has its channels swapped.
    pub fn from_samples(
        rate: SampleRate,
        channels: ChannelCount,
        samples: Vec<i16>,
    ) -> Result<Self, MockError> {
        channels.frames(samples.len())?;
        Ok(Self {
            rate,
            channels,
            samples,
        })
    }

    /// Adopt audio `via-audio` produced.
    #[must_use]
    pub fn from_wav_audio(audio: WavAudio) -> Self {
        Self {
            rate: audio.rate(),
            channels: audio.channels(),
            samples: audio.into_samples(),
        }
    }

    /// `duration` of digital silence, mono.
    ///
    /// Useful precisely because it is *not* audible: a gate test that has to
    /// show the speaker draining wants frames, not sound.
    #[must_use]
    pub fn silence(rate: SampleRate, duration: Duration) -> Self {
        Self::mono(
            rate,
            vec![0; usize_frames(rate.duration_to_frames(duration))],
        )
    }

    /// `duration` of a [`TONE_HZ`] triangle wave, mono.
    ///
    /// Integer arithmetic only — see the module docs.
    #[must_use]
    pub fn tone(rate: SampleRate, duration: Duration) -> Self {
        let frames = usize_frames(rate.duration_to_frames(duration));
        // `TONE_HZ` is never zero and no rate `via-audio` names is below it, so
        // `period` is at least 2 and `half` at least 1. The clamps state that
        // rather than relying on it, and keep the division below total for a
        // rate a caller invents.
        let period = i64::from(rate.hz() / TONE_HZ).max(2);
        let half = (period / 2).max(1);
        let amplitude = i64::from(TONE_AMPLITUDE);
        let mut samples = Vec::with_capacity(frames);
        for frame in 0..frames {
            let phase = (frame as i64) % period;
            // `<` and `<=` are the same function here — at `phase == half`,
            // `period - phase` is `half` too — so a mutation of this comparison
            // survives by being equivalent rather than by being untested.
            let rise = if phase < half { phase } else { period - phase };
            // `rise` runs 0..=half, so the value runs -amplitude..=amplitude,
            // which always fits an `i16`. The clamp is belt and braces for a
            // caller-invented rate whose period is odd.
            let value = (rise * 2 * amplitude) / half - amplitude;
            samples.push(value.clamp(i64::from(i16::MIN), i64::from(i16::MAX)) as i16);
        }
        Self::mono(rate, samples)
    }

    /// Mono audio, which cannot be torn: one sample is one frame.
    fn mono(rate: SampleRate, samples: Vec<i16>) -> Self {
        Self {
            rate,
            channels: ChannelCount::MONO,
            samples,
        }
    }

    /// Read a WAV fixture from memory.
    ///
    /// Accepts every bit depth [`via_audio::read_wav`] does, normalised to
    /// PCM16.
    ///
    /// # Errors
    ///
    /// [`MockError::Audio`] on a malformed or undecodable file.
    pub fn from_wav_bytes(bytes: &[u8]) -> Result<Self, MockError> {
        Ok(Self::from_wav_audio(read_wav(std::io::Cursor::new(bytes))?))
    }

    /// Read a WAV fixture from disk.
    ///
    /// # Errors
    ///
    /// [`MockError::Audio`] when the file cannot be opened or decoded.
    pub fn from_wav_file<P: AsRef<Path>>(path: P) -> Result<Self, MockError> {
        Ok(Self::from_wav_audio(read_wav_file(path)?))
    }

    /// The same audio as a [`WavAudio`], ready for [`via_audio::write_wav`].
    ///
    /// # Errors
    ///
    /// [`MockError::Audio`] never, in practice: every constructor above already
    /// refused a torn buffer. The signature keeps the guarantee `via-audio`'s
    /// own rather than restating it here.
    pub fn to_wav_audio(&self) -> Result<WavAudio, MockError> {
        Ok(WavAudio::new(
            self.rate,
            self.channels,
            self.samples.clone(),
        )?)
    }

    /// The rate these samples play at.
    #[must_use]
    pub fn rate(&self) -> SampleRate {
        self.rate
    }

    /// The interleaved channel count.
    #[must_use]
    pub fn channels(&self) -> ChannelCount {
        self.channels
    }

    /// The interleaved samples.
    #[must_use]
    pub fn samples(&self) -> &[i16] {
        &self.samples
    }

    /// Frame count — what a [`via_audio::PlaybackCursor`] counts.
    #[must_use]
    pub fn frames(&self) -> usize {
        self.channels.frames_floor(self.samples.len())
    }

    /// Playing time, exact at this rate.
    #[must_use]
    pub fn duration(&self) -> Duration {
        self.rate.frames_to_duration(self.frames() as u64)
    }

    /// Whether there is any audio at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// The samples as little-endian PCM16 bytes.
    #[must_use]
    pub fn to_pcm16le(&self) -> Vec<u8> {
        via_audio::i16_to_pcm16le(&self.samples)
    }

    /// The whole clip as one base64 PCM16 payload.
    #[must_use]
    pub fn to_base64(&self) -> String {
        encode_audio(&self.to_pcm16le())
    }

    /// Split into [`BLOCK_MILLIS`] chunks — the cadence a host plays at.
    ///
    /// The last chunk is short rather than padded, so the frame counts sum to
    /// [`frames`](Self::frames) exactly and a `PlaybackCursor` fed from them
    /// reports the clip's real duration.
    #[must_use]
    pub fn chunks(&self) -> Vec<AudioChunk> {
        self.chunks_of(Duration::from_millis(BLOCK_MILLIS))
    }

    /// Split into chunks of `block`.
    ///
    /// A `block` shorter than one frame yields one chunk per frame rather than
    /// an unbounded number of empty ones.
    #[must_use]
    pub fn chunks_of(&self, block: Duration) -> Vec<AudioChunk> {
        let channels = self.channels.get();
        let frames_per_chunk = usize_frames(self.rate.duration_to_frames(block)).max(1);
        self.samples
            .chunks(frames_per_chunk.saturating_mul(channels))
            .map(|samples| AudioChunk {
                base64: encode_audio(&via_audio::i16_to_pcm16le(samples)),
                frames: samples.len() / channels,
            })
            .collect()
    }

    /// One `response.audio.delta` per [`BLOCK_MILLIS`] chunk.
    #[must_use]
    pub fn audio_deltas(&self) -> Vec<Value> {
        self.audio_deltas_of(Duration::from_millis(BLOCK_MILLIS))
    }

    /// One `response.audio.delta` per `block`.
    #[must_use]
    pub fn audio_deltas_of(&self, block: Duration) -> Vec<Value> {
        self.chunks_of(block)
            .into_iter()
            .map(|chunk| events::audio_delta(&chunk.base64))
            .collect()
    }
}

/// `u64` frames as `usize`, saturating rather than wrapping on a 32-bit host.
fn usize_frames(frames: u64) -> usize {
    usize::try_from(frames).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn a_tone_is_the_exact_frame_count_for_its_duration() {
        for (rate, millis, frames) in [
            (SampleRate::HZ_16000, 100_u64, 1_600_usize),
            (SampleRate::HZ_24000, 100, 2_400),
            (SampleRate::HZ_48000, 100, 4_800),
            (SampleRate::HZ_24000, 0, 0),
        ] {
            let audio = CannedAudio::tone(rate, Duration::from_millis(millis));
            assert_eq!(audio.frames(), frames, "{rate:?} {millis}ms");
            assert_eq!(audio.duration(), Duration::from_millis(millis));
        }
    }

    #[test]
    fn a_tone_is_byte_identical_every_time_and_on_every_host() {
        let first = CannedAudio::tone(SampleRate::HZ_24000, Duration::from_millis(40));
        let second = CannedAudio::tone(SampleRate::HZ_24000, Duration::from_millis(40));
        assert_eq!(first, second);
        // Integer arithmetic, so the value is nameable rather than approximate:
        // 24 000 / 200 = 120 frames per cycle, half = 60, so frame 60 is the
        // positive peak and frame 0 the negative one.
        assert_eq!(first.samples()[0], -TONE_AMPLITUDE);
        assert_eq!(first.samples()[60], TONE_AMPLITUDE);
        assert_eq!(first.samples()[120], -TONE_AMPLITUDE);
    }

    #[test]
    fn a_tone_never_leaves_the_pcm16_range() {
        for rate in [
            SampleRate::HZ_16000,
            SampleRate::HZ_24000,
            SampleRate::HZ_48000,
        ] {
            let audio = CannedAudio::tone(rate, Duration::from_millis(50));
            let peak = audio.samples().iter().copied().map(i16::abs).max();
            assert_eq!(peak, Some(TONE_AMPLITUDE), "{rate:?}");
        }
    }

    #[test]
    fn silence_is_frames_without_sound() {
        let audio = CannedAudio::silence(SampleRate::HZ_24000, Duration::from_millis(60));
        assert_eq!(audio.frames(), 1_440);
        assert!(audio.samples().iter().all(|sample| *sample == 0));
        assert!(!audio.is_empty());
        assert!(CannedAudio::silence(SampleRate::HZ_24000, Duration::ZERO).is_empty());
    }

    #[test]
    fn chunks_sum_to_the_whole_clip_even_when_the_last_one_is_short() {
        // 50 ms at 24 kHz is 1 200 frames: two full 20 ms blocks and a 10 ms
        // remainder.
        let audio = CannedAudio::tone(SampleRate::HZ_24000, Duration::from_millis(50));
        let chunks = audio.chunks();
        assert_eq!(chunks.len(), 3);
        assert_eq!(
            chunks.iter().map(|chunk| chunk.frames).collect::<Vec<_>>(),
            [480, 480, 240]
        );
        assert_eq!(
            chunks.iter().map(|chunk| chunk.frames).sum::<usize>(),
            audio.frames()
        );
        assert_eq!(chunks[0].duration(audio.rate()), Duration::from_millis(20));
        assert_eq!(chunks[2].duration(audio.rate()), Duration::from_millis(10));
    }

    #[test]
    fn a_chunk_shorter_than_a_frame_still_terminates() {
        let audio = CannedAudio::tone(SampleRate::HZ_24000, Duration::from_millis(1));
        let chunks = audio.chunks_of(Duration::ZERO);
        assert_eq!(chunks.len(), audio.frames());
        assert!(chunks.iter().all(|chunk| chunk.frames == 1));
    }

    #[test]
    fn an_empty_clip_has_no_chunks_and_no_deltas() {
        let audio = CannedAudio::silence(SampleRate::HZ_24000, Duration::ZERO);
        assert!(audio.chunks().is_empty());
        assert!(audio.audio_deltas().is_empty());
        assert_eq!(audio.to_base64(), "");
    }

    #[test]
    fn base64_round_trips_to_the_same_pcm() {
        let audio = CannedAudio::tone(SampleRate::HZ_24000, Duration::from_millis(20));
        let decoded = decode_audio(&audio.to_base64()).expect("base64");
        assert_eq!(decoded, audio.to_pcm16le());
        assert_eq!(decoded.len(), audio.frames() * 2);
    }

    #[test]
    fn text_that_is_not_base64_is_refused_rather_than_guessed() {
        let error = decode_audio("not base64!!").expect_err("refused");
        assert_eq!(error.code(), "VIA_MOCK_NOT_BASE64");
    }

    #[test]
    fn a_delta_carries_the_chunk_and_nothing_else() {
        let audio = CannedAudio::tone(SampleRate::HZ_24000, Duration::from_millis(20));
        let deltas = audio.audio_deltas();
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0]["type"], "response.audio.delta");
        assert_eq!(deltas[0]["delta"], audio.chunks()[0].base64.as_str());
    }

    #[test]
    fn a_stereo_clip_keeps_its_frame_count_through_chunking() {
        let audio = CannedAudio::from_samples(
            SampleRate::HZ_48000,
            ChannelCount::STEREO,
            (0..960).map(|n| n as i16).collect(),
        )
        .expect("stereo");
        assert_eq!(audio.frames(), 480);
        assert_eq!(
            audio.chunks().iter().map(|c| c.frames).sum::<usize>(),
            audio.frames()
        );
    }

    #[test]
    fn a_torn_stereo_buffer_is_refused() {
        let error =
            CannedAudio::from_samples(SampleRate::HZ_48000, ChannelCount::STEREO, vec![1, 2, 3])
                .expect_err("torn");
        assert_eq!(error.code(), "VIA_MOCK_AUDIO");
    }

    #[test]
    fn a_wav_fixture_round_trips_through_memory() {
        let audio = CannedAudio::tone(SampleRate::HZ_24000, Duration::from_millis(40));
        let mut bytes = std::io::Cursor::new(Vec::new());
        via_audio::write_wav(&mut bytes, &audio.to_wav_audio().expect("wav")).expect("write");
        let read = CannedAudio::from_wav_bytes(&bytes.into_inner()).expect("read");
        assert_eq!(read, audio);
    }

    #[test]
    fn a_wav_file_that_is_not_a_wav_file_says_which_one() {
        let error = CannedAudio::from_wav_bytes(b"not a riff header").expect_err("refused");
        assert_eq!(error.code(), "VIA_MOCK_AUDIO");
        let MockError::Audio { detail } = error else {
            panic!("expected an audio failure");
        };
        // The source chain is flattened in, so the message says more than
        // `AudioError::Wav`'s own four words.
        assert!(detail.len() > "wav i/o failed".len(), "{detail}");
    }

    #[test]
    fn a_missing_wav_file_is_an_audio_failure_not_a_panic() {
        let error =
            CannedAudio::from_wav_file("/nonexistent/via-mock-fixture.wav").expect_err("refused");
        assert_eq!(error.code(), "VIA_MOCK_AUDIO");
    }
}
