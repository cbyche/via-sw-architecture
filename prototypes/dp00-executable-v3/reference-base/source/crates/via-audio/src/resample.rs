//! Sample-rate conversion between the three rates VIA moves audio at.
//!
//! # Which rates, and why
//!
//! | Rate | Who wants it |
//! | --- | --- |
//! | 16 kHz | realtime **input** (`dashscope`, `s2s`), streaming ASR, the wake-word feature extractor |
//! | 24 kHz | realtime **output** — model speech, fixed by the OpenAI Realtime API and matched by every provider VIA ships |
//! | 48 kHz | what host capture hardware commonly opens at |
//!
//! Every pair among them is a small rational ratio: 48→24 is 1:2, 48→16 is
//! 1:3, 24→16 is 2:3, 16→24 is 3:2.
//!
//! # What was chosen, and what was rejected
//!
//! **Rejected: drop-sample decimation for the exact integer ratios.** Taking
//! every second sample of a 48 kHz stream to get 24 kHz is two lines of code
//! and it is wrong. Everything above the *new* Nyquist frequency does not
//! disappear; it folds back into the passband as a mirror image. Going 48→16,
//! any energy between 8 and 24 kHz — room tone, a fan, sibilance, switching
//! noise from a laptop's own power supply — lands back on top of the speech the
//! ASR is trying to read. It does not sound like a glitch. It sounds like the
//! model mishearing things, which is the single hardest class of bug to trace
//! back to an audio conversion, and the reason this crate exists.
//!
//! **Chosen: one engine for every ratio** — `rubato`'s [`FftFixedIn`], which
//! band-limits properly and is exact for rational rate pairs. Keeping a single
//! path means the integer ratios are exercised by the same code the awkward
//! ones use, so 44 100 → 24 000 is not a second, less-tested implementation.
//! The one shortcut taken is that equal rates skip the engine entirely and copy
//! (see [`Resampler::is_pass_through`]).
//!
//! # Layout: planar in, interleaved out
//!
//! `rubato` works in **planar** `Vec<Vec<f32>>` — one contiguous buffer per
//! channel. Realtime sockets carry **interleaved** PCM16 — `L R L R …` as
//! little-endian `i16`. Neither side is going to change, so the adapters live
//! here: [`Resampler::process_interleaved_pcm16`] takes what comes off the
//! socket and returns what goes back onto it, and
//! [`Resampler::process_planar`] is available for a caller that is already
//! planar.
//!
//! # Delay
//!
//! An FFT resampler has a group delay of half its output transform, so its
//! output lags its input. This crate **compensates**: it discards exactly
//! [`Resampler::output_delay_frames`] leading output frames, so output frame
//! *n* corresponds to input frame *n · out/in*. [`Resampler::finish`] then
//! flushes the tail and trims to the exact expected length, which makes a
//! one-shot conversion length-exact — 4800 frames at 48 kHz become exactly 2400
//! at 24 kHz, not 2400 ± a chunk.

use rubato::{FftFixedIn, Resampler as _};

use crate::error::Result;
use crate::rate::{ChannelCount, SampleRate};
use crate::sample;

/// Number of sub-chunks `rubato` is asked to split a chunk into. One keeps the
/// internal transform aligned with our chunk size instead of subdividing it.
const SUB_CHUNKS: usize = 1;

/// A streaming, band-limited sample-rate converter.
///
/// Feed it whatever sizes the host produces; it accumulates internally and
/// emits whole processed chunks. Call [`Resampler::finish`] at end of stream to
/// get the tail.
pub struct Resampler {
    input_rate: SampleRate,
    output_rate: SampleRate,
    channels: ChannelCount,
    engine: Option<Engine>,
    input_frames: u64,
    output_frames: u64,
}

struct Engine {
    inner: FftFixedIn<f32>,
    chunk_frames: usize,
    /// Planar staging: one buffer per channel, drained a chunk at a time.
    staged: Vec<Vec<f32>>,
    /// Reusable planar output, sized to the engine's maximum.
    scratch: Vec<Vec<f32>>,
    /// Leading output frames still to be discarded for delay compensation.
    delay_remaining: usize,
    /// Total delay, kept for reporting after compensation has run out.
    delay_frames: usize,
}

impl Resampler {
    /// Build a converter from `input_rate` to `output_rate` over `channels`
    /// interleaved channels.
    ///
    /// # Errors
    ///
    /// [`crate::AudioError::ResamplerConstruction`] if `rubato` rejects the
    /// rate pair.
    pub fn new(
        input_rate: SampleRate,
        output_rate: SampleRate,
        channels: ChannelCount,
    ) -> Result<Self> {
        let engine = if input_rate == output_rate {
            None
        } else {
            Some(Engine::new(input_rate, output_rate, channels)?)
        };
        Ok(Self {
            input_rate,
            output_rate,
            channels,
            engine,
            input_frames: 0,
            output_frames: 0,
        })
    }

    /// The rate this converter consumes.
    #[must_use]
    pub fn input_rate(&self) -> SampleRate {
        self.input_rate
    }

    /// The rate this converter produces.
    #[must_use]
    pub fn output_rate(&self) -> SampleRate {
        self.output_rate
    }

    /// The interleaved channel count.
    #[must_use]
    pub fn channels(&self) -> ChannelCount {
        self.channels
    }

    /// Whether the rates match, so audio is copied rather than converted.
    #[must_use]
    pub fn is_pass_through(&self) -> bool {
        self.engine.is_none()
    }

    /// Group delay of the underlying engine, in output frames.
    ///
    /// Zero for a pass-through. Reported for diagnostics only — the delay is
    /// already compensated for in the output, so a caller does not need to
    /// subtract it.
    #[must_use]
    pub fn output_delay_frames(&self) -> usize {
        self.engine.as_ref().map_or(0, |engine| engine.delay_frames)
    }

    /// Input frames the engine consumes per internal chunk.
    ///
    /// One for a pass-through, since it has no chunking.
    #[must_use]
    pub fn chunk_frames(&self) -> usize {
        self.engine.as_ref().map_or(1, |engine| engine.chunk_frames)
    }

    /// Output frames that *should* have been produced for everything fed in so
    /// far, at the exact rate ratio.
    ///
    /// [`Self::finish`] pads or trims to reach this. Exposed so a caller can
    /// assert its own accounting against the converter's.
    #[must_use]
    pub fn expected_output_frames(&self) -> u64 {
        let input = u128::from(self.input_frames);
        let out_hz = u128::from(self.output_rate.hz());
        let in_hz = u128::from(self.input_rate.hz());
        u64::try_from(input * out_hz / in_hz).unwrap_or(u64::MAX)
    }

    /// Convert planar `f32` frames, returning whatever whole chunks are ready.
    ///
    /// An empty return is normal: the engine buffers until it has a full chunk.
    ///
    /// # Errors
    ///
    /// [`crate::AudioError::ChannelCountMismatch`] if `input` does not have
    /// this converter's channel count, [`crate::AudioError::ChannelLengthMismatch`]
    /// if its channels are ragged, or a `rubato` failure.
    pub fn process_planar<C: AsRef<[f32]>>(&mut self, input: &[C]) -> Result<Vec<Vec<f32>>> {
        if input.len() != self.channels.get() {
            return Err(crate::AudioError::ChannelCountMismatch {
                expected: self.channels.get(),
                actual: input.len(),
            });
        }
        let frames = sample::planar_frames(input)?;
        self.input_frames = self.input_frames.saturating_add(frames as u64);

        let Some(engine) = self.engine.as_mut() else {
            let copied: Vec<Vec<f32>> = input.iter().map(|c| c.as_ref().to_vec()).collect();
            self.output_frames = self.output_frames.saturating_add(frames as u64);
            return Ok(copied);
        };

        let produced = engine.push(input)?;
        let produced_frames = produced.first().map_or(0, Vec::len);
        self.output_frames = self.output_frames.saturating_add(produced_frames as u64);
        Ok(produced)
    }

    /// Convert interleaved `f32` frames.
    ///
    /// # Errors
    ///
    /// [`crate::AudioError::NotFrameAligned`] if the sample count is not a
    /// multiple of the channel count, or a `rubato` failure.
    pub fn process_interleaved(&mut self, input: &[f32]) -> Result<Vec<f32>> {
        let planar = sample::deinterleave(input, self.channels)?;
        let produced = self.process_planar(&planar)?;
        flatten(&produced)
    }

    /// Convert interleaved PCM16 frames — the socket-to-socket path.
    ///
    /// # Errors
    ///
    /// As [`Self::process_interleaved`].
    pub fn process_interleaved_pcm16(&mut self, input: &[i16]) -> Result<Vec<i16>> {
        let planar = sample::deinterleave_pcm16(input, self.channels)?;
        let produced = self.process_planar(&planar)?;
        flatten_pcm16(&produced)
    }

    /// Flush the engine and return the remaining planar output, trimmed to the
    /// exact expected length.
    ///
    /// After this call the converter has emitted exactly
    /// [`Self::expected_output_frames`] frames in total.
    ///
    /// # Errors
    ///
    /// A `rubato` failure.
    pub fn finish_planar(&mut self) -> Result<Vec<Vec<f32>>> {
        let expected = self.expected_output_frames();
        let channels = self.channels.get();

        let Some(engine) = self.engine.as_mut() else {
            return Ok(vec![Vec::new(); channels]);
        };

        let mut tail: Vec<Vec<f32>> = vec![Vec::new(); channels];
        // Bound the flush: enough zero chunks to push the staged frames and the
        // compensated delay all the way through, plus slack. A bound rather
        // than `while` so a pathological rate pair cannot spin forever.
        let max_chunks = engine.flush_chunk_bound();
        for _ in 0..max_chunks {
            if self.output_frames + tail_frames(&tail) as u64 >= expected {
                break;
            }
            let produced = engine.push_silence()?;
            append_planar(&mut tail, &produced);
        }

        self.output_frames = self.output_frames.saturating_add(tail_frames(&tail) as u64);

        // Trim an overshoot, or pad a shortfall with silence, so the total is
        // exactly the rate ratio applied to the input. Either correction is at
        // most a fraction of a chunk.
        let overshoot = self.output_frames.saturating_sub(expected) as usize;
        if overshoot > 0 {
            for channel in &mut tail {
                let keep = channel.len().saturating_sub(overshoot);
                channel.truncate(keep);
            }
            self.output_frames = expected;
        } else {
            let shortfall = (expected - self.output_frames) as usize;
            if shortfall > 0 {
                for channel in &mut tail {
                    channel.resize(channel.len() + shortfall, 0.0);
                }
                self.output_frames = expected;
            }
        }
        Ok(tail)
    }

    /// Flush and return the remaining output, interleaved.
    ///
    /// # Errors
    ///
    /// As [`Self::finish_planar`].
    pub fn finish(&mut self) -> Result<Vec<f32>> {
        let tail = self.finish_planar()?;
        flatten(&tail)
    }

    /// Flush and return the remaining output as interleaved PCM16.
    ///
    /// # Errors
    ///
    /// As [`Self::finish_planar`].
    pub fn finish_pcm16(&mut self) -> Result<Vec<i16>> {
        let tail = self.finish_planar()?;
        flatten_pcm16(&tail)
    }
}

impl core::fmt::Debug for Resampler {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Resampler")
            .field("input_rate", &self.input_rate)
            .field("output_rate", &self.output_rate)
            .field("channels", &self.channels)
            .field("pass_through", &self.is_pass_through())
            .field("chunk_frames", &self.chunk_frames())
            .field("output_delay_frames", &self.output_delay_frames())
            .finish()
    }
}

impl Engine {
    fn new(
        input_rate: SampleRate,
        output_rate: SampleRate,
        channels: ChannelCount,
    ) -> Result<Self> {
        // One capture block of input per chunk — the same 20 ms cadence the
        // host already produces (`SampleRate::capture_block_frames`), so the
        // converter's natural chunk is the one it will actually be fed.
        let chunk_frames = input_rate.capture_block_frames();
        let inner = FftFixedIn::<f32>::new(
            input_rate.hz() as usize,
            output_rate.hz() as usize,
            chunk_frames,
            SUB_CHUNKS,
            channels.get(),
        )?;
        let delay_frames = inner.output_delay();
        let max_out = inner.output_frames_max();
        Ok(Self {
            inner,
            chunk_frames,
            staged: vec![Vec::new(); channels.get()],
            scratch: vec![vec![0.0; max_out]; channels.get()],
            delay_remaining: delay_frames,
            delay_frames,
        })
    }

    /// Frames of input the flush loop may still have to push through: the
    /// staged remainder plus the uncompensated delay, in chunks, plus slack for
    /// the trailing partial chunk.
    fn flush_chunk_bound(&self) -> usize {
        let staged = self.staged.first().map_or(0, Vec::len);
        let pending = staged + self.delay_remaining + self.chunk_frames;
        pending.div_ceil(self.chunk_frames) + 2
    }

    fn push<C: AsRef<[f32]>>(&mut self, input: &[C]) -> Result<Vec<Vec<f32>>> {
        for (channel, incoming) in self.staged.iter_mut().zip(input) {
            channel.extend_from_slice(incoming.as_ref());
        }
        self.drain()
    }

    fn push_silence(&mut self) -> Result<Vec<Vec<f32>>> {
        for channel in &mut self.staged {
            channel.resize(channel.len() + self.chunk_frames, 0.0);
        }
        self.drain()
    }

    fn drain(&mut self) -> Result<Vec<Vec<f32>>> {
        let mut out: Vec<Vec<f32>> = vec![Vec::new(); self.staged.len()];
        while self.staged.first().map_or(0, Vec::len) >= self.chunk_frames {
            let (_, produced) =
                self.inner
                    .process_into_buffer(&self.staged, &mut self.scratch, None)?;
            for channel in &mut self.staged {
                channel.drain(..self.chunk_frames);
            }
            // Discard the group-delay preamble so output frame n lines up with
            // input frame n * out/in.
            let skip = self.delay_remaining.min(produced);
            self.delay_remaining -= skip;
            for (sink, source) in out.iter_mut().zip(&self.scratch) {
                sink.extend_from_slice(&source[skip..produced]);
            }
        }
        Ok(out)
    }
}

fn tail_frames(planar: &[Vec<f32>]) -> usize {
    planar.first().map_or(0, Vec::len)
}

fn append_planar(sink: &mut [Vec<f32>], source: &[Vec<f32>]) {
    for (channel, produced) in sink.iter_mut().zip(source) {
        channel.extend_from_slice(produced);
    }
}

fn flatten(planar: &[Vec<f32>]) -> Result<Vec<f32>> {
    if planar.iter().all(Vec::is_empty) {
        return Ok(Vec::new());
    }
    sample::interleave(planar)
}

fn flatten_pcm16(planar: &[Vec<f32>]) -> Result<Vec<i16>> {
    if planar.iter().all(Vec::is_empty) {
        return Ok(Vec::new());
    }
    sample::interleave_pcm16(planar)
}

/// One-shot conversion of an interleaved PCM16 buffer.
///
/// Constructs a [`Resampler`], feeds the whole buffer, and flushes — so the
/// result is length-exact and time-aligned with the input.
///
/// # Errors
///
/// As [`Resampler::process_interleaved_pcm16`].
pub fn resample_interleaved_pcm16(
    input: &[i16],
    channels: ChannelCount,
    input_rate: SampleRate,
    output_rate: SampleRate,
) -> Result<Vec<i16>> {
    let mut resampler = Resampler::new(input_rate, output_rate, channels)?;
    let mut out = resampler.process_interleaved_pcm16(input)?;
    out.extend(resampler.finish_pcm16()?);
    Ok(out)
}

/// One-shot conversion of a mono PCM16 buffer — the common case.
///
/// # Errors
///
/// As [`resample_interleaved_pcm16`].
pub fn resample_mono_pcm16(
    input: &[i16],
    input_rate: SampleRate,
    output_rate: SampleRate,
) -> Result<Vec<i16>> {
    resample_interleaved_pcm16(input, ChannelCount::MONO, input_rate, output_rate)
}

/// One-shot conversion of an interleaved `f32` buffer.
///
/// # Errors
///
/// As [`Resampler::process_interleaved`].
pub fn resample_interleaved(
    input: &[f32],
    channels: ChannelCount,
    input_rate: SampleRate,
    output_rate: SampleRate,
) -> Result<Vec<f32>> {
    let mut resampler = Resampler::new(input_rate, output_rate, channels)?;
    let mut out = resampler.process_interleaved(input)?;
    out.extend(resampler.finish()?);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;

    fn tone(frames: usize, hz: f32, rate: SampleRate) -> Vec<f32> {
        let step = TAU * hz / rate.hz() as f32;
        (0..frames).map(|n| (step * n as f32).sin()).collect()
    }

    fn rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum: f32 = samples.iter().map(|s| s * s).sum();
        (sum / samples.len() as f32).sqrt()
    }

    /// Naive drop-sample decimation, present only so the tests can show what it
    /// does to a tone above the new Nyquist frequency.
    fn decimate(samples: &[f32], factor: usize) -> Vec<f32> {
        samples.iter().step_by(factor).copied().collect()
    }

    fn one_shot(input: &[f32], from: SampleRate, to: SampleRate) -> Vec<f32> {
        resample_interleaved(input, ChannelCount::MONO, from, to).expect("mono resample")
    }

    #[test]
    fn equal_rates_pass_through_untouched() {
        let mut resampler = Resampler::new(
            SampleRate::HZ_24000,
            SampleRate::HZ_24000,
            ChannelCount::MONO,
        )
        .expect("same rate");
        assert!(resampler.is_pass_through());
        assert_eq!(resampler.output_delay_frames(), 0);
        let input: Vec<i16> = (0..1_000).collect();
        let out = resampler
            .process_interleaved_pcm16(&input)
            .expect("pass through");
        assert_eq!(out, input);
        assert!(resampler.finish_pcm16().expect("flush").is_empty());
    }

    #[test]
    fn output_length_is_exactly_the_rate_ratio() {
        let cases = [
            (SampleRate::HZ_48000, SampleRate::HZ_24000, 4_800, 2_400),
            (SampleRate::HZ_48000, SampleRate::HZ_16000, 4_800, 1_600),
            (SampleRate::HZ_24000, SampleRate::HZ_16000, 4_800, 3_200),
            (SampleRate::HZ_16000, SampleRate::HZ_24000, 4_800, 7_200),
            (SampleRate::HZ_16000, SampleRate::HZ_48000, 4_800, 14_400),
            (SampleRate::HZ_24000, SampleRate::HZ_48000, 4_800, 9_600),
        ];
        for (from, to, frames, expected) in cases {
            let input = tone(frames, 440.0, from);
            let out = one_shot(&input, from, to);
            assert_eq!(
                out.len(),
                expected,
                "{from} -> {to} produced {} frames, wanted {expected}",
                out.len()
            );
        }
    }

    #[test]
    fn a_speech_band_tone_survives_every_conversion() {
        // 440 Hz is well inside every passband, so amplitude must be preserved.
        for (from, to) in [
            (SampleRate::HZ_48000, SampleRate::HZ_24000),
            (SampleRate::HZ_48000, SampleRate::HZ_16000),
            (SampleRate::HZ_24000, SampleRate::HZ_16000),
            (SampleRate::HZ_16000, SampleRate::HZ_24000),
            (SampleRate::HZ_24000, SampleRate::HZ_48000),
        ] {
            let input = tone(24_000, 440.0, from);
            let out = one_shot(&input, from, to);
            // Ignore the first and last 10 ms: transform edges.
            let edge = to.millis_to_frames(10) as usize;
            let interior = &out[edge..out.len() - edge];
            let level = rms(interior);
            assert!(
                (level - rms(&input)).abs() < 0.02,
                "{from} -> {to} changed a 440 Hz tone's level from {} to {level}",
                rms(&input)
            );
        }
    }

    #[test]
    fn a_tone_above_the_new_nyquist_is_rejected_not_folded() {
        // 9 kHz at 48 kHz, downsampled to 16 kHz whose Nyquist is 8 kHz. Naive
        // decimation folds it to 16000 - 9000 = 7 kHz at full amplitude, right
        // in the middle of the speech band. A band-limited resampler must
        // throw it away instead.
        let from = SampleRate::HZ_48000;
        let to = SampleRate::HZ_16000;
        let input = tone(24_000, 9_000.0, from);
        assert!((rms(&input) - 0.707).abs() < 0.01, "test tone is not unity");

        let resampled = one_shot(&input, from, to);
        let edge = to.millis_to_frames(20) as usize;
        let level = rms(&resampled[edge..resampled.len() - edge]);
        assert!(
            level < 0.05,
            "9 kHz leaked through a 48k -> 16k conversion at rms {level}"
        );

        // And the comparison that makes the point: drop-sample decimation keeps
        // essentially all of it, as an alias at 7 kHz.
        let aliased = decimate(&input, 3);
        assert!(
            rms(&aliased) > 0.5,
            "decimation was expected to alias the tone through at full level"
        );
    }

    #[test]
    fn streaming_in_odd_chunks_matches_a_single_shot() {
        let from = SampleRate::HZ_48000;
        let to = SampleRate::HZ_24000;
        let input = tone(9_600, 300.0, from);
        let whole = one_shot(&input, from, to);

        let mut resampler = Resampler::new(from, to, ChannelCount::MONO).expect("rates");
        let mut streamed = Vec::new();
        // 337 is deliberately coprime with the internal chunk size, so chunk
        // boundaries never line up with call boundaries.
        for chunk in input.chunks(337) {
            streamed.extend(resampler.process_interleaved(chunk).expect("chunk"));
        }
        streamed.extend(resampler.finish().expect("flush"));

        assert_eq!(streamed.len(), whole.len());
        for (index, (a, b)) in streamed.iter().zip(&whole).enumerate() {
            assert!(
                (a - b).abs() < 1e-6,
                "frame {index} differs: streamed {a}, one-shot {b}"
            );
        }
    }

    #[test]
    fn stereo_channels_stay_independent() {
        let from = SampleRate::HZ_48000;
        let to = SampleRate::HZ_24000;
        let left = tone(4_800, 400.0, from);
        // Right channel is silent; if interleaving is off by one, energy from
        // the left channel bleeds into it.
        let mut interleaved = Vec::with_capacity(left.len() * 2);
        for sample in &left {
            interleaved.push(*sample);
            interleaved.push(0.0);
        }
        let out = resample_interleaved(&interleaved, ChannelCount::STEREO, from, to)
            .expect("stereo resample");
        assert_eq!(out.len(), 4_800);

        let planar = sample::deinterleave(&out, ChannelCount::STEREO).expect("aligned");
        assert!(rms(&planar[0]) > 0.5, "left channel lost its tone");
        assert!(
            rms(&planar[1]) < 1e-6,
            "silent right channel picked up {} of energy",
            rms(&planar[1])
        );
    }

    #[test]
    fn a_ragged_or_misshapen_input_is_refused() {
        let mut resampler = Resampler::new(
            SampleRate::HZ_48000,
            SampleRate::HZ_24000,
            ChannelCount::STEREO,
        )
        .expect("rates");
        assert!(matches!(
            resampler.process_interleaved(&[0.0, 0.0, 0.0]),
            Err(crate::AudioError::NotFrameAligned {
                samples: 3,
                channels: 2
            })
        ));
        let mono_planar = vec![vec![0.0_f32; 10]];
        assert!(matches!(
            resampler.process_planar(&mono_planar),
            Err(crate::AudioError::ChannelCountMismatch {
                expected: 2,
                actual: 1
            })
        ));
    }

    #[test]
    fn a_non_integer_ratio_uses_the_same_path() {
        let from = SampleRate::new(44_100).expect("non-zero");
        let to = SampleRate::HZ_24000;
        let input = tone(44_100, 1_000.0, from);
        let out = one_shot(&input, from, to);
        // 44100 frames * 24000 / 44100 = 24000 exactly.
        assert_eq!(out.len(), 24_000);
        let edge = to.millis_to_frames(20) as usize;
        let level = rms(&out[edge..out.len() - edge]);
        assert!(
            (level - 0.707).abs() < 0.02,
            "1 kHz tone came out at rms {level}"
        );
    }

    #[test]
    fn silence_in_is_silence_out() {
        let out = resample_mono_pcm16(
            &vec![0_i16; 9_600],
            SampleRate::HZ_48000,
            SampleRate::HZ_16000,
        )
        .expect("resample");
        assert_eq!(out.len(), 3_200);
        assert!(out.iter().all(|s| *s == 0));
    }

    #[test]
    fn an_empty_input_produces_an_empty_output() {
        let out =
            resample_mono_pcm16(&[], SampleRate::HZ_48000, SampleRate::HZ_24000).expect("resample");
        assert!(out.is_empty());
    }

    #[test]
    fn delay_is_reported_and_already_compensated() {
        let resampler = Resampler::new(
            SampleRate::HZ_48000,
            SampleRate::HZ_24000,
            ChannelCount::MONO,
        )
        .expect("rates");
        assert!(resampler.output_delay_frames() > 0);
        // 20 ms of input per chunk, which at 48 kHz is one capture block.
        assert_eq!(resampler.chunk_frames(), 960);

        // Compensation is what makes an impulse land where it went in. Put a
        // click 100 ms into a 48 kHz stream and find it 100 ms into the 24 kHz
        // one.
        let mut input = vec![0.0_f32; 48_000];
        input[4_800] = 1.0;
        let out = one_shot(&input, SampleRate::HZ_48000, SampleRate::HZ_24000);
        let peak = out
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
            .map(|(index, _)| index)
            .expect("non-empty");
        assert!(
            peak.abs_diff(2_400) <= 2,
            "impulse landed at frame {peak}, expected 2400"
        );
    }
}
