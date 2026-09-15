//! PCM16 ↔ `f32` conversion, byte codecs, and interleave / deinterleave.
//!
//! # The convention, stated once
//!
//! `i16` is asymmetric: it spans `-32768 ..= 32767`. There is no scale factor
//! that maps that range onto `-1.0 ..= 1.0` bijectively, so a choice has to be
//! made, and the two halves of the round trip have to agree on it.
//!
//! **This crate divides by 32768 and clamps on the way back.**
//!
//! ```text
//!   i16 → f32:   x = v / 32768                    (exact for every v)
//!   f32 → i16:   v = clamp(round(x * 32768), -32768, 32767)
//! ```
//!
//! Consequences, all of them tested:
//!
//! * `-32768 → -1.0` exactly, `0 → 0.0` exactly, `32767 → 32767/32768`.
//! * `i16 → f32 → i16` is **exact for all 65 536 inputs**. `v / 32768` is
//!   representable in `f32` without loss (a power-of-two divisor and a 16-bit
//!   numerator), so multiplying back recovers `v` before rounding even
//!   matters.
//! * `f32 → i16 → f32` is within [`PCM16_ROUND_TRIP_EPSILON`] for inputs in
//!   `-1.0 ..= 32767/32768`. Above `32767/32768` the value saturates — `+1.0`
//!   comes back as `32767/32768`, an error of `1/32768` — because `+1.0`
//!   simply has no `i16` image. That asymmetry is the price of `-1.0` being
//!   exact, and `-1.0` being exact is what the upstream contract requires.
//! * `NaN` maps to `0` (silence) rather than to a random sample.
//!
//! The alternative — dividing by 32767 — makes `+1.0` exact and `-1.0`
//! overflow instead, and would shift every wake-word detection threshold in the
//! product by one part in 32768. Upstream picked 32768 and the contract
//! catalogue locks it.
//!
//! # External contract
//!
//! `server/src/voice/wake-word/sherpa-detector.mjs:9-17` and its test
//! `server/test/wake-word-detector.test.mjs:5-17`, catalogued as
//! `pcm16 -> f32 conversion` and `PCM16 to float conversion`:
//!
//! > divisor is 32768 (not 32767): -32768→-1, -16384→-0.5, 0→0,
//! > 32767→32767/32768; an odd trailing byte is dropped

use crate::error::Result;
use crate::rate::ChannelCount;

/// The scale factor between `i16` PCM and normalised `f32`.
///
/// **External contract** — 32768, not 32767, from
/// `server/src/voice/wake-word/sherpa-detector.mjs:14`
/// (`bytes.readInt16LE(index * 2) / 32768`). Changing it shifts every
/// wake-word detection threshold.
pub const PCM16_SCALE: f32 = 32_768.0;

/// Bytes per PCM16 sample on the wire.
pub const PCM16_BYTES_PER_SAMPLE: usize = 2;

/// Worst-case error of an `f32 → i16 → f32` round trip.
///
/// Half a quantisation step: `0.5 / 32768 == 1 / 65536`. Guaranteed for inputs
/// in `-1.0 ..= 32767/32768`; see the module docs for what happens above that.
pub const PCM16_ROUND_TRIP_EPSILON: f32 = 1.0 / 65_536.0;

/// Convert one PCM16 sample to normalised `f32`.
///
/// Exact for every input: `-32768 → -1.0`, `-16384 → -0.5`, `0 → 0.0`,
/// `32767 → 0.999969…`.
#[inline]
#[must_use]
pub fn pcm16_to_f32(sample: i16) -> f32 {
    f32::from(sample) / PCM16_SCALE
}

/// Convert one normalised `f32` sample to PCM16.
///
/// Rounds to nearest (ties away from zero) and clamps to `i16`'s asymmetric
/// range. `NaN` becomes `0`.
#[inline]
#[must_use]
pub fn f32_to_pcm16(sample: f32) -> i16 {
    // `round` is what makes the i16 → f32 → i16 round trip exact; truncation
    // would bias every negative sample one step toward -32768.
    let scaled = (sample * PCM16_SCALE).round();
    // `f32::clamp` propagates NaN rather than panicking on it, and the
    // subsequent `as` cast maps NaN to 0 — silence, which is the only sane
    // image for a sample that is not a number.
    let clamped = scaled.clamp(f32::from(i16::MIN), f32::from(i16::MAX));
    // Saturating float→int cast; the clamp above has already put the value in
    // range, so this is a plain truncation of an integral value.
    clamped as i16
}

/// Convert a slice of PCM16 samples to normalised `f32`.
#[must_use]
pub fn pcm16_to_f32_slice(samples: &[i16]) -> Vec<f32> {
    samples.iter().copied().map(pcm16_to_f32).collect()
}

/// Convert a slice of normalised `f32` samples to PCM16.
#[must_use]
pub fn f32_to_pcm16_slice(samples: &[f32]) -> Vec<i16> {
    samples.iter().copied().map(f32_to_pcm16).collect()
}

/// Decode little-endian PCM16 bytes into `i16` samples.
///
/// **External contract** — the sample count is `floor(bytes.len() / 2)`; a lone
/// trailing byte is dropped, matching
/// `server/src/voice/wake-word/sherpa-detector.mjs:11` and the upstream test
/// `server/test/wake-word-detector.test.mjs:15-18`. A realtime socket splits a
/// PCM stream at arbitrary byte offsets, so a half sample at the end of a chunk
/// is normal traffic, not an error — but see [`crate::FrameBuffer`], which
/// carries the odd byte forward instead of losing it.
#[must_use]
pub fn pcm16le_to_i16(bytes: &[u8]) -> Vec<i16> {
    bytes
        .chunks_exact(PCM16_BYTES_PER_SAMPLE)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
        .collect()
}

/// Decode little-endian PCM16 bytes straight to normalised `f32`.
///
/// The composition of [`pcm16le_to_i16`] and [`pcm16_to_f32`], which is the
/// exact operation upstream's `pcm16Base64ToFloat32` performs after its base64
/// decode. Base64 itself is deliberately *not* this crate's business — it is a
/// transport concern that belongs with the wire codec.
#[must_use]
pub fn pcm16le_to_f32(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(PCM16_BYTES_PER_SAMPLE)
        .map(|pair| pcm16_to_f32(i16::from_le_bytes([pair[0], pair[1]])))
        .collect()
}

/// Encode `i16` samples as little-endian PCM16 bytes.
#[must_use]
pub fn i16_to_pcm16le(samples: &[i16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * PCM16_BYTES_PER_SAMPLE);
    for sample in samples {
        out.extend_from_slice(&sample.to_le_bytes());
    }
    out
}

/// Encode normalised `f32` samples as little-endian PCM16 bytes.
#[must_use]
pub fn f32_to_pcm16le(samples: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * PCM16_BYTES_PER_SAMPLE);
    for sample in samples {
        out.extend_from_slice(&f32_to_pcm16(*sample).to_le_bytes());
    }
    out
}

/// Split an interleaved buffer into one `Vec` per channel.
///
/// The layout adapter for [`crate::Resampler`]: sockets carry interleaved
/// samples, `rubato` wants planar `Vec<Vec<_>>`.
///
/// # Errors
///
/// [`crate::AudioError::NotFrameAligned`] if `interleaved.len()` is not a
/// multiple of `channels`.
pub fn deinterleave<T: Copy>(interleaved: &[T], channels: ChannelCount) -> Result<Vec<Vec<T>>> {
    let frames = channels.frames(interleaved.len())?;
    let n = channels.get();
    let mut planar: Vec<Vec<T>> = (0..n).map(|_| Vec::with_capacity(frames)).collect();
    for frame in interleaved.chunks_exact(n) {
        for (channel, sample) in planar.iter_mut().zip(frame) {
            channel.push(*sample);
        }
    }
    Ok(planar)
}

/// Split an interleaved PCM16 buffer into planar normalised `f32`.
///
/// # Errors
///
/// [`crate::AudioError::NotFrameAligned`] if the sample count is not a multiple
/// of `channels`.
pub fn deinterleave_pcm16(interleaved: &[i16], channels: ChannelCount) -> Result<Vec<Vec<f32>>> {
    let frames = channels.frames(interleaved.len())?;
    let n = channels.get();
    let mut planar: Vec<Vec<f32>> = (0..n).map(|_| Vec::with_capacity(frames)).collect();
    for frame in interleaved.chunks_exact(n) {
        for (channel, sample) in planar.iter_mut().zip(frame) {
            channel.push(pcm16_to_f32(*sample));
        }
    }
    Ok(planar)
}

/// Merge per-channel buffers into one interleaved buffer.
///
/// Generic over the inner container so it accepts `&[Vec<f32>]` (what `rubato`
/// returns) and `&[&[f32]]` alike.
///
/// # Errors
///
/// [`crate::AudioError::ZeroChannels`] if `planar` is empty, or
/// [`crate::AudioError::ChannelLengthMismatch`] if the channels are ragged.
pub fn interleave<T: Copy, C: AsRef<[T]>>(planar: &[C]) -> Result<Vec<T>> {
    let frames = planar_frames(planar)?;
    let channels = planar.len();
    let mut out = Vec::with_capacity(frames * channels);
    for frame in 0..frames {
        for channel in planar {
            out.push(channel.as_ref()[frame]);
        }
    }
    Ok(out)
}

/// Merge planar normalised `f32` channels into one interleaved PCM16 buffer.
///
/// # Errors
///
/// As [`interleave`].
pub fn interleave_pcm16<C: AsRef<[f32]>>(planar: &[C]) -> Result<Vec<i16>> {
    let frames = planar_frames(planar)?;
    let channels = planar.len();
    let mut out = Vec::with_capacity(frames * channels);
    for frame in 0..frames {
        for channel in planar {
            out.push(f32_to_pcm16(channel.as_ref()[frame]));
        }
    }
    Ok(out)
}

/// Frame count of a planar buffer, validating that it is a rectangle.
///
/// # Errors
///
/// [`crate::AudioError::ZeroChannels`] if `planar` is empty, or
/// [`crate::AudioError::ChannelLengthMismatch`] on the first ragged channel.
pub fn planar_frames<T, C: AsRef<[T]>>(planar: &[C]) -> Result<usize> {
    let Some(first) = planar.first() else {
        return Err(crate::AudioError::ZeroChannels);
    };
    let frames = first.as_ref().len();
    for (index, channel) in planar.iter().enumerate().skip(1) {
        let actual = channel.as_ref().len();
        if actual != frames {
            return Err(crate::AudioError::ChannelLengthMismatch {
                first: frames,
                channel: index,
                actual,
            });
        }
    }
    Ok(frames)
}

/// Average an interleaved multi-channel `f32` buffer down to mono.
///
/// Averaging is done in `f32`, so no intermediate can overflow. Two channels
/// that are in phase keep their amplitude; two that are in antiphase cancel,
/// which is the correct behaviour and not a bug — it is why a mono downmix of a
/// stereo mic array can be quieter than either channel.
///
/// # Errors
///
/// [`crate::AudioError::NotFrameAligned`] if the sample count is not a multiple
/// of `channels`.
pub fn downmix_to_mono(interleaved: &[f32], channels: ChannelCount) -> Result<Vec<f32>> {
    let frames = channels.frames(interleaved.len())?;
    if channels.is_mono() {
        return Ok(interleaved.to_vec());
    }
    let n = channels.get();
    // `n` is non-zero by construction, so the reciprocal is finite.
    let scale = 1.0 / n as f32;
    let mut out = Vec::with_capacity(frames);
    for frame in interleaved.chunks_exact(n) {
        out.push(frame.iter().sum::<f32>() * scale);
    }
    Ok(out)
}

/// Average an interleaved multi-channel PCM16 buffer down to mono PCM16.
///
/// The sum is accumulated in `i32` and divided before conversion, so a frame of
/// `[-32768, -32768]` averages to `-32768` rather than wrapping.
///
/// # Errors
///
/// [`crate::AudioError::NotFrameAligned`] if the sample count is not a multiple
/// of `channels`.
pub fn downmix_to_mono_pcm16(interleaved: &[i16], channels: ChannelCount) -> Result<Vec<i16>> {
    let frames = channels.frames(interleaved.len())?;
    if channels.is_mono() {
        return Ok(interleaved.to_vec());
    }
    let n = channels.get();
    let divisor = n as i32;
    let mut out = Vec::with_capacity(frames);
    for frame in interleaved.chunks_exact(n) {
        let sum: i32 = frame.iter().map(|s| i32::from(*s)).sum();
        // |sum| <= 32768 * n, so sum / n is within i16 by construction.
        let mean = sum / divisor;
        out.push(mean.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16);
    }
    Ok(out)
}

/// Duplicate a mono buffer across `channels` interleaved channels.
#[must_use]
pub fn upmix_from_mono<T: Copy>(mono: &[T], channels: ChannelCount) -> Vec<T> {
    let n = channels.get();
    if n == 1 {
        return mono.to_vec();
    }
    let mut out = Vec::with_capacity(mono.len() * n);
    for sample in mono {
        for _ in 0..n {
            out.push(*sample);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deterministic xorshift, so the property tests are reproducible without
    /// pulling a property-testing crate into the workspace table.
    struct Rng(u64);

    impl Rng {
        fn next_u32(&mut self) -> u32 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            (x >> 32) as u32
        }

        /// Uniform in `-1.0 ..= 32767/32768`, the range with a round-trip
        /// guarantee.
        fn next_sample(&mut self) -> f32 {
            let unit = f32::from_bits((self.next_u32() >> 9) | 0x3f80_0000) - 1.0; // [0, 1)
            unit * (1.0 + 32_767.0 / 32_768.0) - 1.0
        }
    }

    #[test]
    fn boundary_samples_are_the_documented_values() {
        assert_eq!(pcm16_to_f32(i16::MIN), -1.0);
        assert_eq!(pcm16_to_f32(-16_384), -0.5);
        assert_eq!(pcm16_to_f32(0), 0.0);
        assert_eq!(pcm16_to_f32(i16::MAX), 32_767.0 / 32_768.0);

        assert_eq!(f32_to_pcm16(-1.0), i16::MIN);
        assert_eq!(f32_to_pcm16(-0.5), -16_384);
        assert_eq!(f32_to_pcm16(0.0), 0);
        assert_eq!(f32_to_pcm16(32_767.0 / 32_768.0), i16::MAX);
    }

    #[test]
    fn out_of_range_input_saturates_rather_than_wrapping() {
        // The whole point of clamping: +1.0 has no i16 image, and the naive
        // `(x * 32768.0) as i16` would be fine here but `(x * 32767.0)` style
        // scaling or a wrapping cast would not.
        assert_eq!(f32_to_pcm16(1.0), i16::MAX);
        assert_eq!(f32_to_pcm16(2.0), i16::MAX);
        assert_eq!(f32_to_pcm16(1e30), i16::MAX);
        assert_eq!(f32_to_pcm16(f32::INFINITY), i16::MAX);
        assert_eq!(f32_to_pcm16(-1.0000001), i16::MIN);
        assert_eq!(f32_to_pcm16(-2.0), i16::MIN);
        assert_eq!(f32_to_pcm16(f32::NEG_INFINITY), i16::MIN);
        assert_eq!(f32_to_pcm16(f32::NAN), 0);
    }

    #[test]
    fn pcm16_round_trip_is_exact_for_every_sample() {
        for raw in i16::MIN..=i16::MAX {
            let round_tripped = f32_to_pcm16(pcm16_to_f32(raw));
            assert_eq!(
                round_tripped, raw,
                "i16 {raw} did not survive the round trip"
            );
        }
    }

    #[test]
    fn f32_round_trip_stays_within_the_documented_epsilon() {
        let mut rng = Rng(0x5eed_1234_abcd_0001);
        for _ in 0..200_000 {
            let sample = rng.next_sample();
            let round_tripped = pcm16_to_f32(f32_to_pcm16(sample));
            let error = (round_tripped - sample).abs();
            assert!(
                error <= PCM16_ROUND_TRIP_EPSILON,
                "{sample} -> {round_tripped} drifted by {error}, over {PCM16_ROUND_TRIP_EPSILON}"
            );
        }
    }

    #[test]
    fn f32_round_trip_epsilon_is_tight_at_a_tie() {
        // Exactly half a step above zero: rounds away from zero, so the error
        // is exactly the epsilon and the bound above must be inclusive.
        let half_step = 0.5 / PCM16_SCALE;
        assert_eq!(f32_to_pcm16(half_step), 1);
        let error = (pcm16_to_f32(1) - half_step).abs();
        assert_eq!(error, PCM16_ROUND_TRIP_EPSILON);
    }

    #[test]
    fn saturation_above_the_last_representable_sample_is_bounded() {
        // Documented exception to the epsilon: +1.0 saturates, and the error is
        // one full step rather than half a step.
        let error = (pcm16_to_f32(f32_to_pcm16(1.0)) - 1.0).abs();
        assert_eq!(error, 1.0 / PCM16_SCALE);
    }

    #[test]
    fn byte_codec_round_trips() {
        let samples: Vec<i16> = vec![i16::MIN, -16_384, -1, 0, 1, 16_384, i16::MAX];
        let bytes = i16_to_pcm16le(&samples);
        assert_eq!(bytes.len(), samples.len() * 2);
        assert_eq!(pcm16le_to_i16(&bytes), samples);
        // Little-endian, explicitly: -32768 is 0x8000 -> [0x00, 0x80].
        assert_eq!(&bytes[0..2], &[0x00, 0x80]);
        // 32767 is 0x7fff -> [0xff, 0x7f].
        assert_eq!(&bytes[12..14], &[0xff, 0x7f]);
    }

    #[test]
    fn interleave_round_trips_stereo() {
        let planar = vec![vec![1.0_f32, 3.0, 5.0], vec![2.0, 4.0, 6.0]];
        let flat = interleave(&planar).expect("rectangular");
        assert_eq!(flat, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let back = deinterleave(&flat, ChannelCount::STEREO).expect("aligned");
        assert_eq!(back, planar);
    }

    #[test]
    fn interleave_rejects_a_ragged_rectangle() {
        let planar = vec![vec![1.0_f32, 2.0], vec![3.0]];
        assert!(matches!(
            interleave(&planar),
            Err(crate::AudioError::ChannelLengthMismatch {
                first: 2,
                channel: 1,
                actual: 1
            })
        ));
    }

    #[test]
    fn deinterleave_rejects_a_torn_frame() {
        let flat = [1.0_f32, 2.0, 3.0];
        assert!(matches!(
            deinterleave(&flat, ChannelCount::STEREO),
            Err(crate::AudioError::NotFrameAligned {
                samples: 3,
                channels: 2
            })
        ));
    }

    #[test]
    fn downmix_averages_and_does_not_overflow() {
        let loud = [i16::MIN, i16::MIN, i16::MAX, i16::MAX];
        assert_eq!(
            downmix_to_mono_pcm16(&loud, ChannelCount::STEREO).expect("aligned"),
            vec![i16::MIN, i16::MAX]
        );
        let antiphase = [1.0_f32, -1.0, 0.5, -0.5];
        assert_eq!(
            downmix_to_mono(&antiphase, ChannelCount::STEREO).expect("aligned"),
            vec![0.0, 0.0]
        );
    }

    #[test]
    fn upmix_duplicates() {
        assert_eq!(
            upmix_from_mono(&[1_i16, 2], ChannelCount::STEREO),
            vec![1, 1, 2, 2]
        );
        assert_eq!(upmix_from_mono(&[1_i16, 2], ChannelCount::MONO), vec![1, 2]);
    }
}
