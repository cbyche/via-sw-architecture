//! Sample rates, channel counts, and exact `frames ↔ Duration` accounting.
//!
//! Everything here is integer arithmetic. Nothing rounds through `f64`, because
//! the two consumers of this module — the Injection Gate's "is the speaker
//! still draining" predicate and `conversation.item.truncate(audio_end_ms)` —
//! both compare the result against a wire value, and a half-millisecond of
//! float drift there is a bug that presents as a truncated sentence.

use core::num::{NonZeroU32, NonZeroUsize};
use core::time::Duration;

use crate::error::{AudioError, Result};

/// Nanoseconds in one second, as the widening type used for exact rate math.
const NANOS_PER_SEC: u128 = 1_000_000_000;

/// Milliseconds in one second.
const MILLIS_PER_SEC: u128 = 1_000;

/// Divisor upstream applies to a sample rate to get its capture / playback
/// block size: one block is a fiftieth of a second, i.e. 20 ms.
///
/// **External contract** — `tui/native/portaudio-voice-io.py:133,146`
/// (`blocksize=max(160, capture_rate // 50)` and
/// `blocksize=max(240, playback_rate // 50)`), catalogued as contract
/// `audio sample rates`.
const BLOCKS_PER_SEC: u32 = 50;

/// Floor on the capture block, in frames.
///
/// **External contract** — `tui/native/portaudio-voice-io.py:133`.
const MIN_CAPTURE_BLOCK_FRAMES: usize = 160;

/// Floor on the playback block, in frames.
///
/// **External contract** — `tui/native/portaudio-voice-io.py:146`.
const MIN_PLAYBACK_BLOCK_FRAMES: usize = 240;

/// Nominal duration of one capture or playback block.
///
/// **External contract** — `tui/native/portaudio-voice-io.py:24-25,133,146`,
/// catalogued as contract `audio sample rates` ("Capture chunk =
/// capture_rate/50 (20 ms, …)").
pub const BLOCK_MILLIS: u64 = 20;

/// A non-zero sample rate in hertz.
///
/// A newtype rather than a bare `u32` for two reasons: it is non-zero, so the
/// divisions below need no runtime guard; and it makes
/// `Resampler::new(input, output, …)` impossible to call with the rates
/// transposed without noticing.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SampleRate(NonZeroU32);

impl SampleRate {
    /// 16 kHz — the realtime *input* rate.
    ///
    /// **External contract** — `server/src/voice/providers/dashscope.mjs:47`
    /// and `server/src/voice/providers/s2s.mjs:11`, catalogued as contract
    /// `audio sample rates` and as `built-in realtime providers`. It is also
    /// the rate the wake-word feature extractor is configured at
    /// (`server/src/voice/wake-word/sherpa-detector.mjs:26`,
    /// `featConfig.samplingRate`).
    pub const HZ_16000: Self = Self::konst(16_000);

    /// 24 kHz — the realtime *output* rate, and the rate the OpenAI Realtime
    /// API fixes for model speech.
    ///
    /// **External contract** — `server/src/voice/providers/dashscope.mjs:48`
    /// and `server/src/voice/providers/s2s.mjs:12`, catalogued as contract
    /// `audio sample rates`. The Layer-1 upstream pins the same value as
    /// `REALTIME_SAMPLE_RATE_HZ` (upstream `tiniffi/src/live_ffi.rs:60`).
    pub const HZ_24000: Self = Self::konst(24_000);

    /// 48 kHz — not a wire rate, but what host capture devices commonly open
    /// at, which is why both downward conversions exist at all.
    pub const HZ_48000: Self = Self::konst(48_000);

    /// Construct at compile time.
    ///
    /// Private, and called only from the `const` items above with non-zero
    /// literals, so the fallback arm is dead. It is a fallback rather than a
    /// `panic!` or an `unwrap` because both are production-code smells the
    /// repo's audit gates reject on sight; the `const` block below proves at
    /// compile time that no constant actually took it.
    const fn konst(hz: u32) -> Self {
        match NonZeroU32::new(hz) {
            Some(hz) => Self(hz),
            None => Self(NonZeroU32::MIN),
        }
    }

    /// Construct from hertz.
    ///
    /// # Errors
    ///
    /// [`AudioError::ZeroSampleRate`] if `hz` is zero.
    pub const fn new(hz: u32) -> Result<Self> {
        match NonZeroU32::new(hz) {
            Some(hz) => Ok(Self(hz)),
            None => Err(AudioError::ZeroSampleRate),
        }
    }

    /// The rate in hertz.
    #[must_use]
    pub const fn hz(self) -> u32 {
        self.0.get()
    }

    /// The rate as its non-zero representation.
    #[must_use]
    pub const fn get(self) -> NonZeroU32 {
        self.0
    }

    /// Frames in one millisecond, when the rate divides evenly by 1000.
    ///
    /// `Some(16)`, `Some(24)`, `Some(48)` for the three rates this crate cares
    /// about — which is exactly why `audio_end_ms` is lossless on all of them.
    /// `None` for a rate like 44 100 Hz, where a millisecond is 44.1 frames and
    /// millisecond arithmetic necessarily rounds.
    #[must_use]
    pub const fn frames_per_millisecond(self) -> Option<u32> {
        let hz = self.hz();
        if hz.is_multiple_of(1_000) {
            Some(hz / 1_000)
        } else {
            None
        }
    }

    /// Exact duration of `frames` frames, floored to the nanosecond.
    ///
    /// For 16/24/48 kHz the nanosecond floor is only reachable by frame counts
    /// that are not whole milliseconds, and it never accumulates: the result is
    /// computed from the absolute frame count, not by summing per-chunk
    /// durations.
    #[must_use]
    pub fn frames_to_duration(self, frames: u64) -> Duration {
        let hz = u128::from(self.hz());
        let nanos = u128::from(frames) * NANOS_PER_SEC / hz;
        // `nanos / NANOS_PER_SEC` fits u64 for any u64 frame count at any
        // non-zero rate, and the remainder is < 1e9 so it fits u32.
        let secs = (nanos / NANOS_PER_SEC) as u64;
        let sub = (nanos % NANOS_PER_SEC) as u32;
        Duration::new(secs, sub)
    }

    /// Frames spanned by `duration`, floored.
    ///
    /// Floors rather than rounds so that a caller converting a wire duration
    /// into a buffer index can never index past the audio it was given.
    #[must_use]
    pub fn duration_to_frames(self, duration: Duration) -> u64 {
        let hz = u128::from(self.hz());
        let frames = duration.as_nanos() * hz / NANOS_PER_SEC;
        u64::try_from(frames).unwrap_or(u64::MAX)
    }

    /// Whole milliseconds spanned by `frames`, floored.
    ///
    /// This is the value `conversation.item.truncate` wants for
    /// `audio_end_ms`. Flooring is the conservative direction: it can only
    /// claim the user heard *less* than they did, never more, so the model's
    /// transcript is never left holding audio that was cut off.
    #[must_use]
    pub const fn frames_to_millis(self, frames: u64) -> u64 {
        let hz = self.hz() as u128;
        ((frames as u128) * MILLIS_PER_SEC / hz) as u64
    }

    /// Whole milliseconds spanned by `frames`, rounded up.
    ///
    /// The complement of [`Self::frames_to_millis`], for the Injection Gate:
    /// when asking "how long until the speaker has drained", rounding up means
    /// never declaring silence a fraction of a millisecond early.
    #[must_use]
    pub const fn frames_to_millis_ceil(self, frames: u64) -> u64 {
        let hz = self.hz() as u128;
        let numerator = (frames as u128) * MILLIS_PER_SEC;
        numerator.div_ceil(hz) as u64
    }

    /// Frames spanned by `millis` whole milliseconds, floored.
    ///
    /// Exact (no flooring actually occurs) for every rate where
    /// [`Self::frames_per_millisecond`] is `Some`.
    #[must_use]
    pub const fn millis_to_frames(self, millis: u64) -> u64 {
        let hz = self.hz() as u128;
        ((millis as u128) * hz / MILLIS_PER_SEC) as u64
    }

    /// Frames in one capture block: `rate / 50`, floored at 160.
    ///
    /// **External contract** — `tui/native/portaudio-voice-io.py:133`
    /// (`blocksize=max(160, capture_rate // 50)`), catalogued as contract
    /// `audio sample rates`. 320 frames at 16 kHz, 960 at 48 kHz; the 160 floor
    /// only binds below 8 kHz.
    #[must_use]
    pub const fn capture_block_frames(self) -> usize {
        let blocks = (self.hz() / BLOCKS_PER_SEC) as usize;
        if blocks < MIN_CAPTURE_BLOCK_FRAMES {
            MIN_CAPTURE_BLOCK_FRAMES
        } else {
            blocks
        }
    }

    /// Frames in one playback block: `rate / 50`, floored at 240.
    ///
    /// **External contract** — `tui/native/portaudio-voice-io.py:146`
    /// (`blocksize=max(240, playback_rate // 50)`), catalogued as contract
    /// `audio sample rates`. 480 frames at 24 kHz; the 240 floor only binds
    /// below 12 kHz.
    #[must_use]
    pub const fn playback_block_frames(self) -> usize {
        let blocks = (self.hz() / BLOCKS_PER_SEC) as usize;
        if blocks < MIN_PLAYBACK_BLOCK_FRAMES {
            MIN_PLAYBACK_BLOCK_FRAMES
        } else {
            blocks
        }
    }
}

impl core::fmt::Display for SampleRate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} Hz", self.hz())
    }
}

impl TryFrom<u32> for SampleRate {
    type Error = AudioError;

    fn try_from(hz: u32) -> Result<Self> {
        Self::new(hz)
    }
}

impl From<SampleRate> for u32 {
    fn from(rate: SampleRate) -> Self {
        rate.hz()
    }
}

/// A non-zero channel count.
///
/// One frame is `channels` consecutive interleaved samples. Carrying the count
/// in a type rather than a `usize` parameter is what keeps
/// [`crate::interleave`] and [`crate::deinterleave`] from being called with a
/// sample count where a frame count belongs.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ChannelCount(NonZeroUsize);

impl ChannelCount {
    /// One channel — every realtime wire format in VIA.
    pub const MONO: Self = Self::konst(1);

    /// Two interleaved channels — what host capture hardware often hands over.
    pub const STEREO: Self = Self::konst(2);

    /// Construct at compile time. As [`SampleRate::konst`]: private, called
    /// only with non-zero literals, and the fallback arm is proven dead by the
    /// `const` block at the end of this module.
    const fn konst(channels: usize) -> Self {
        match NonZeroUsize::new(channels) {
            Some(channels) => Self(channels),
            None => Self(NonZeroUsize::MIN),
        }
    }

    /// Construct from a count.
    ///
    /// # Errors
    ///
    /// [`AudioError::ZeroChannels`] if `channels` is zero.
    pub const fn new(channels: usize) -> Result<Self> {
        match NonZeroUsize::new(channels) {
            Some(channels) => Ok(Self(channels)),
            None => Err(AudioError::ZeroChannels),
        }
    }

    /// The channel count.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0.get()
    }

    /// Whether this is a single channel.
    #[must_use]
    pub const fn is_mono(self) -> bool {
        self.0.get() == 1
    }

    /// Frames represented by `samples` interleaved samples.
    ///
    /// # Errors
    ///
    /// [`AudioError::NotFrameAligned`] when `samples` does not divide evenly,
    /// i.e. the buffer ends mid-frame.
    pub const fn frames(self, samples: usize) -> Result<usize> {
        if samples.is_multiple_of(self.0.get()) {
            Ok(samples / self.0.get())
        } else {
            Err(AudioError::NotFrameAligned {
                samples,
                channels: self.0.get(),
            })
        }
    }

    /// Frames represented by `samples` interleaved samples, discarding a torn
    /// trailing frame.
    ///
    /// Use this only where the stream genuinely has no framing guarantee. On a
    /// VIA wire path prefer [`Self::frames`], which reports the tear.
    #[must_use]
    pub const fn frames_floor(self, samples: usize) -> usize {
        samples / self.0.get()
    }
}

impl core::fmt::Display for ChannelCount {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0.get())
    }
}

impl TryFrom<usize> for ChannelCount {
    type Error = AudioError;

    fn try_from(channels: usize) -> Result<Self> {
        Self::new(channels)
    }
}

impl From<ChannelCount> for usize {
    fn from(channels: ChannelCount) -> Self {
        channels.get()
    }
}

/// Compile-time proof that every constant above carries the value it names,
/// and therefore that neither `konst` fallback arm was taken.
///
/// This runs during compilation: if a constant is ever edited to zero, the
/// build fails here rather than the value silently becoming 1.
const _: () = {
    assert!(SampleRate::HZ_16000.hz() == 16_000);
    assert!(SampleRate::HZ_24000.hz() == 24_000);
    assert!(SampleRate::HZ_48000.hz() == 48_000);
    assert!(ChannelCount::MONO.get() == 1);
    assert!(ChannelCount::STEREO.get() == 2);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_rejects_zero() {
        assert!(matches!(
            SampleRate::new(0),
            Err(AudioError::ZeroSampleRate)
        ));
        assert!(matches!(
            ChannelCount::new(0),
            Err(AudioError::ZeroChannels)
        ));
    }

    #[test]
    fn frames_per_millisecond_is_exact_for_wire_rates() {
        assert_eq!(SampleRate::HZ_16000.frames_per_millisecond(), Some(16));
        assert_eq!(SampleRate::HZ_24000.frames_per_millisecond(), Some(24));
        assert_eq!(SampleRate::HZ_48000.frames_per_millisecond(), Some(48));
        let cd = SampleRate::new(44_100).expect("44100 is non-zero");
        assert_eq!(cd.frames_per_millisecond(), None);
    }

    #[test]
    fn duration_round_trips_on_whole_milliseconds() {
        for rate in [
            SampleRate::HZ_16000,
            SampleRate::HZ_24000,
            SampleRate::HZ_48000,
        ] {
            for millis in [0_u64, 1, 20, 999, 60_000] {
                let frames = rate.millis_to_frames(millis);
                assert_eq!(rate.frames_to_millis(frames), millis);
                assert_eq!(rate.frames_to_millis_ceil(frames), millis);
                assert_eq!(
                    rate.frames_to_duration(frames),
                    Duration::from_millis(millis)
                );
                assert_eq!(
                    rate.duration_to_frames(Duration::from_millis(millis)),
                    frames
                );
            }
        }
    }

    #[test]
    fn sub_millisecond_frames_floor_and_ceil_apart() {
        let rate = SampleRate::HZ_24000;
        // 25 frames is 1.041666… ms.
        assert_eq!(rate.frames_to_millis(25), 1);
        assert_eq!(rate.frames_to_millis_ceil(25), 2);
        // Exactly one millisecond agrees in both directions.
        assert_eq!(rate.frames_to_millis(24), 1);
        assert_eq!(rate.frames_to_millis_ceil(24), 1);
        // And nanosecond resolution is kept, not rounded to the millisecond.
        assert_eq!(
            rate.frames_to_duration(1),
            Duration::new(0, 41_666) // 1/24000 s = 41666.66… ns, floored
        );
    }

    #[test]
    fn one_hour_of_audio_stays_exact() {
        let rate = SampleRate::HZ_48000;
        let frames = 48_000_u64 * 3_600;
        assert_eq!(rate.frames_to_duration(frames), Duration::from_secs(3_600));
        assert_eq!(rate.frames_to_millis(frames), 3_600_000);
    }

    #[test]
    fn channel_frames_reports_a_torn_frame() {
        assert_eq!(ChannelCount::STEREO.frames(8).expect("8 is aligned"), 4);
        assert!(matches!(
            ChannelCount::STEREO.frames(7),
            Err(AudioError::NotFrameAligned {
                samples: 7,
                channels: 2
            })
        ));
        assert_eq!(ChannelCount::STEREO.frames_floor(7), 3);
    }
}
