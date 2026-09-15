//! A staging buffer with an explicit commit boundary.
//!
//! Every realtime protocol VIA speaks has the same three-verb shape on its
//! input buffer — append repeatedly, commit once at the turn boundary, clear to
//! abandon — and every provider that had to reimplement it got some corner of
//! it slightly wrong. So it lives here, once.
//!
//! Two regions, not one:
//!
//! * **staged** — appended, not yet delimited. This is the live microphone
//!   tail. It is the region a capacity bound applies to, because it is the one
//!   that grows without limit when nobody commits.
//! * **committed** — a turn's worth of audio, delimited and waiting to be
//!   taken. Unbounded, because the consumer is expected to take it at once; a
//!   ring that silently drops the *front* of a committed turn would truncate
//!   the beginning of an utterance, which is exactly the class of bug this
//!   crate exists to prevent.

use std::collections::VecDeque;
use std::time::Duration;

use crate::rate::{ChannelCount, SampleRate};
use crate::sample::{PCM16_BYTES_PER_SAMPLE, pcm16le_to_i16};

/// An append / commit / take accumulator for interleaved PCM16.
///
/// Holds `i16` rather than bytes because every consumer — resampler, VAD, WAV
/// writer — wants samples, and holding bytes would mean re-deciding endianness
/// at each of them.
#[derive(Clone, Debug)]
pub struct FrameBuffer {
    rate: SampleRate,
    channels: ChannelCount,
    capacity_frames: Option<usize>,
    staged: VecDeque<i16>,
    committed: VecDeque<i16>,
    /// A byte left over from an odd-length `append_pcm16le`, carried into the
    /// next call rather than dropped.
    partial_byte: Option<u8>,
    overrun_frames: u64,
}

impl FrameBuffer {
    /// An unbounded buffer at `rate` with `channels` interleaved channels.
    #[must_use]
    pub fn new(rate: SampleRate, channels: ChannelCount) -> Self {
        Self {
            rate,
            channels,
            capacity_frames: None,
            staged: VecDeque::new(),
            committed: VecDeque::new(),
            partial_byte: None,
            overrun_frames: 0,
        }
    }

    /// A buffer whose *staged* region holds at most `frames` frames.
    ///
    /// Appending past the bound drops the oldest staged frames — whole frames,
    /// so channel phase is preserved — and counts them in
    /// [`Self::overrun_frames`]. This is the pre-roll ring: keep the last N
    /// milliseconds of microphone audio, discard the rest, and never allocate
    /// without limit because a turn never got committed.
    #[must_use]
    pub fn with_capacity_frames(rate: SampleRate, channels: ChannelCount, frames: usize) -> Self {
        Self {
            capacity_frames: Some(frames),
            ..Self::new(rate, channels)
        }
    }

    /// The rate this buffer's samples are at.
    #[must_use]
    pub fn rate(&self) -> SampleRate {
        self.rate
    }

    /// The interleaved channel count.
    #[must_use]
    pub fn channels(&self) -> ChannelCount {
        self.channels
    }

    /// Staged-region bound in frames, if any.
    #[must_use]
    pub fn capacity_frames(&self) -> Option<usize> {
        self.capacity_frames
    }

    /// Cumulative frames dropped from the staged region because it was full.
    ///
    /// Non-zero means the host is producing audio faster than the session is
    /// committing it. Worth logging, never worth panicking over.
    #[must_use]
    pub fn overrun_frames(&self) -> u64 {
        self.overrun_frames
    }

    /// Append interleaved samples to the staged region.
    pub fn append(&mut self, samples: &[i16]) {
        self.staged.extend(samples.iter().copied());
        self.enforce_capacity();
    }

    /// Append little-endian PCM16 bytes to the staged region.
    ///
    /// Unlike [`crate::pcm16le_to_i16`], an odd trailing byte is **carried into
    /// the next call**, not dropped. A socket splits a PCM stream at arbitrary
    /// offsets, so dropping the odd byte here would not merely lose one sample:
    /// it would shift every subsequent sample in the chunk by one byte,
    /// reinterpreting the stream as noise. Upstream's decoder drops it because
    /// it decodes one self-contained base64 payload at a time; a streaming
    /// accumulator must not.
    pub fn append_pcm16le(&mut self, bytes: &[u8]) {
        match self.partial_byte.take() {
            Some(carried) if !bytes.is_empty() => {
                let joined = [carried, bytes[0]];
                self.staged.push_back(i16::from_le_bytes(joined));
                self.push_bytes(&bytes[1..]);
            }
            Some(carried) => {
                self.partial_byte = Some(carried);
            }
            None => self.push_bytes(bytes),
        }
        self.enforce_capacity();
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        self.staged.extend(pcm16le_to_i16(bytes));
        if bytes.len() % PCM16_BYTES_PER_SAMPLE == 1 {
            // `bytes` is non-empty because its length is odd.
            self.partial_byte = bytes.last().copied();
        }
    }

    /// Samples currently staged.
    #[must_use]
    pub fn staged_samples(&self) -> usize {
        self.staged.len()
    }

    /// Whole frames currently staged. A torn trailing frame is not counted.
    #[must_use]
    pub fn staged_frames(&self) -> usize {
        self.channels.frames_floor(self.staged.len())
    }

    /// Duration of the staged region.
    #[must_use]
    pub fn staged_duration(&self) -> Duration {
        self.rate.frames_to_duration(self.staged_frames() as u64)
    }

    /// Whole frames currently committed and not yet taken.
    #[must_use]
    pub fn committed_frames(&self) -> usize {
        self.channels.frames_floor(self.committed.len())
    }

    /// Duration of the committed region.
    #[must_use]
    pub fn committed_duration(&self) -> Duration {
        self.rate.frames_to_duration(self.committed_frames() as u64)
    }

    /// Whether either region holds anything.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.staged.is_empty() && self.committed.is_empty()
    }

    /// Move every whole staged frame into the committed region.
    ///
    /// Returns the number of frames committed. A torn trailing frame — possible
    /// when the host hands over a partial frame — stays staged, so the next
    /// commit picks it up complete instead of splitting it across two turns.
    pub fn commit(&mut self) -> usize {
        let frames = self.staged_frames();
        let samples = frames * self.channels.get();
        self.committed.extend(self.staged.drain(..samples));
        frames
    }

    /// Discard the staged region, keeping anything already committed.
    ///
    /// The `input_audio_buffer.clear` semantic: abandon the audio captured
    /// since the last commit. Returns the samples discarded.
    pub fn clear_staged(&mut self) -> usize {
        let discarded = self.staged.len();
        self.staged.clear();
        self.partial_byte = None;
        discarded
    }

    /// Take everything committed, leaving the staged region alone.
    ///
    /// This is the "take what is committed" operation: the provider drains a
    /// delimited turn while the microphone keeps filling the next one.
    #[must_use]
    pub fn take_committed(&mut self) -> Vec<i16> {
        self.committed.drain(..).collect()
    }

    /// Take up to `frames` frames from the front of the committed region.
    ///
    /// For a provider that must send a turn in fixed-size chunks — see
    /// [`SampleRate::capture_block_frames`].
    #[must_use]
    pub fn take_committed_frames(&mut self, frames: usize) -> Vec<i16> {
        let available = self.committed_frames();
        let take = frames.min(available) * self.channels.get();
        self.committed.drain(..take).collect()
    }

    /// Take everything committed as little-endian PCM16 bytes.
    #[must_use]
    pub fn take_committed_pcm16le(&mut self) -> Vec<u8> {
        crate::sample::i16_to_pcm16le(&self.take_committed())
    }

    /// Drop both regions and the carried partial byte, keeping the overrun
    /// counter — a session reset does not un-happen the overruns that led to
    /// it.
    pub fn reset(&mut self) {
        self.staged.clear();
        self.committed.clear();
        self.partial_byte = None;
    }

    fn enforce_capacity(&mut self) {
        let Some(capacity) = self.capacity_frames else {
            return;
        };
        let channels = self.channels.get();
        let staged = self.staged_frames();
        if staged <= capacity {
            return;
        }
        let excess = staged - capacity;
        // Drop from the front in whole frames: the torn remainder, if any, is
        // at the back, so front-dropping cannot shift channel phase.
        self.staged.drain(..excess * channels);
        self.overrun_frames = self.overrun_frames.saturating_add(excess as u64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mono() -> FrameBuffer {
        FrameBuffer::new(SampleRate::HZ_24000, ChannelCount::MONO)
    }

    #[test]
    fn append_commit_take_is_the_whole_cycle() {
        let mut buffer = mono();
        buffer.append(&[1, 2, 3]);
        assert_eq!(buffer.staged_frames(), 3);
        assert_eq!(buffer.committed_frames(), 0);

        assert_eq!(buffer.commit(), 3);
        assert_eq!(buffer.staged_frames(), 0);
        assert_eq!(buffer.committed_frames(), 3);

        assert_eq!(buffer.take_committed(), vec![1, 2, 3]);
        assert!(buffer.is_empty());
    }

    #[test]
    fn committing_does_not_disturb_audio_appended_after_it() {
        let mut buffer = mono();
        buffer.append(&[1, 2]);
        buffer.commit();
        buffer.append(&[3, 4]);
        // The next turn is already accumulating while the last one drains.
        assert_eq!(buffer.take_committed(), vec![1, 2]);
        assert_eq!(buffer.staged_frames(), 2);
        buffer.commit();
        assert_eq!(buffer.take_committed(), vec![3, 4]);
    }

    #[test]
    fn clear_staged_keeps_committed_audio() {
        let mut buffer = mono();
        buffer.append(&[1, 2]);
        buffer.commit();
        buffer.append(&[9, 9, 9]);
        assert_eq!(buffer.clear_staged(), 3);
        assert_eq!(buffer.staged_frames(), 0);
        assert_eq!(buffer.take_committed(), vec![1, 2]);
    }

    #[test]
    fn partial_frames_are_never_split_across_a_commit() {
        let mut buffer = FrameBuffer::new(SampleRate::HZ_48000, ChannelCount::STEREO);
        // Five samples is two whole stereo frames plus a torn left channel.
        buffer.append(&[1, 2, 3, 4, 5]);
        assert_eq!(buffer.commit(), 2);
        assert_eq!(buffer.take_committed(), vec![1, 2, 3, 4]);
        assert_eq!(buffer.staged_samples(), 1);
        // The right channel arrives and the frame completes intact.
        buffer.append(&[6]);
        assert_eq!(buffer.commit(), 1);
        assert_eq!(buffer.take_committed(), vec![5, 6]);
    }

    #[test]
    fn an_odd_trailing_byte_is_carried_not_dropped() {
        let mut buffer = mono();
        // 0x0201 split across two appends: naive per-chunk decoding would drop
        // the 0x01 and then read 0x02 as the low byte of the next sample,
        // shifting the entire rest of the stream by one byte.
        buffer.append_pcm16le(&[0x01]);
        assert_eq!(buffer.staged_samples(), 0);
        buffer.append_pcm16le(&[0x02, 0x03, 0x04]);
        assert_eq!(buffer.commit(), 2);
        assert_eq!(buffer.take_committed(), vec![0x0201, 0x0403]);
    }

    #[test]
    fn byte_appends_reassemble_a_stream_split_at_every_offset() {
        let samples: Vec<i16> = (-500..500).collect();
        let bytes = crate::sample::i16_to_pcm16le(&samples);
        for split in 1..7 {
            let mut buffer = mono();
            for chunk in bytes.chunks(split) {
                buffer.append_pcm16le(chunk);
            }
            buffer.commit();
            assert_eq!(
                buffer.take_committed(),
                samples,
                "chunking by {split} bytes lost the framing"
            );
        }
    }

    #[test]
    fn take_committed_frames_takes_a_prefix() {
        let mut buffer = FrameBuffer::new(SampleRate::HZ_24000, ChannelCount::STEREO);
        buffer.append(&[1, 2, 3, 4, 5, 6]);
        buffer.commit();
        assert_eq!(buffer.take_committed_frames(2), vec![1, 2, 3, 4]);
        assert_eq!(buffer.committed_frames(), 1);
        // Asking for more than is there yields what is there.
        assert_eq!(buffer.take_committed_frames(99), vec![5, 6]);
        assert_eq!(buffer.take_committed_frames(1), Vec::<i16>::new());
    }

    #[test]
    fn capacity_drops_oldest_staged_frames_and_counts_them() {
        let mut buffer =
            FrameBuffer::with_capacity_frames(SampleRate::HZ_16000, ChannelCount::STEREO, 3);
        buffer.append(&[1, 1, 2, 2, 3, 3]);
        assert_eq!(buffer.overrun_frames(), 0);
        buffer.append(&[4, 4, 5, 5]);
        assert_eq!(buffer.overrun_frames(), 2);
        buffer.commit();
        // The three most recent frames survive, still correctly paired.
        assert_eq!(buffer.take_committed(), vec![3, 3, 4, 4, 5, 5]);
    }

    #[test]
    fn capacity_never_shifts_channel_phase() {
        let mut buffer =
            FrameBuffer::with_capacity_frames(SampleRate::HZ_16000, ChannelCount::STEREO, 2);
        // Append an odd sample count so a torn frame sits at the back while the
        // capacity bound drops frames off the front.
        buffer.append(&[1, 1, 2, 2, 3, 3, 4]);
        buffer.commit();
        let taken = buffer.take_committed();
        assert_eq!(taken, vec![2, 2, 3, 3]);
        for frame in taken.chunks_exact(2) {
            assert_eq!(frame[0], frame[1], "left and right samples got out of step");
        }
    }

    #[test]
    fn durations_follow_the_rate() {
        let mut buffer = FrameBuffer::new(SampleRate::HZ_24000, ChannelCount::MONO);
        buffer.append(&vec![0_i16; 24_000]);
        assert_eq!(buffer.staged_duration(), Duration::from_secs(1));
        buffer.commit();
        assert_eq!(buffer.committed_duration(), Duration::from_secs(1));
        assert_eq!(buffer.staged_duration(), Duration::ZERO);
    }

    #[test]
    fn reset_clears_both_regions_but_keeps_the_overrun_history() {
        let mut buffer =
            FrameBuffer::with_capacity_frames(SampleRate::HZ_16000, ChannelCount::MONO, 1);
        buffer.append(&[1, 2, 3]);
        assert_eq!(buffer.overrun_frames(), 2);
        buffer.commit();
        buffer.append(&[4]);
        buffer.reset();
        assert!(buffer.is_empty());
        assert_eq!(buffer.overrun_frames(), 2);
    }
}
