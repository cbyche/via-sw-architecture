//! Playback position accounting.
//!
//! Two callers in VIA need the same number and neither can approximate it:
//!
//! * The **Injection Gate** (`docs/architecture.md` §3) must answer "is the
//!   speaker still draining?" before it lets a finished result be spoken. Its
//!   blocking predicate is `userSpeaking || turnPending ||
//!   audioResponses.nonEmpty`, and the third term is precisely
//!   [`PlaybackCursor::is_draining`].
//! * `conversation.item.truncate` must be told `audio_end_ms` — how much of the
//!   model's speech the user actually heard — when a barge-in cuts a response
//!   short. ARGO's `interrupt()` sends `response.cancel` and never truncates,
//!   so the model's transcript keeps audio the user never heard; fixing that
//!   needs this number to be right.
//!
//! Counting frames rather than sampling a wall clock is deliberate. Frames are
//! what the host's audio callback actually consumed; a wall clock drifts
//! against the device's crystal and cannot distinguish "the audio played" from
//! "the audio was queued and the device underran".

use std::time::Duration;

use crate::rate::SampleRate;

/// Frames enqueued for playback versus frames actually played.
///
/// Cheap, `Copy`, and rate-aware. Reset it on `playback.clear` — a barge-in
/// discards the queue, and a cursor that kept counting would report a speaker
/// draining audio that was thrown away.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PlaybackCursor {
    rate: SampleRate,
    enqueued: u64,
    played: u64,
}

impl PlaybackCursor {
    /// A cursor at zero for audio at `rate`.
    #[must_use]
    pub fn new(rate: SampleRate) -> Self {
        Self {
            rate,
            enqueued: 0,
            played: 0,
        }
    }

    /// The rate this cursor counts at.
    #[must_use]
    pub fn rate(&self) -> SampleRate {
        self.rate
    }

    /// Record `frames` handed to the output device.
    pub fn enqueue(&mut self, frames: u64) {
        self.enqueued = self.enqueued.saturating_add(frames);
    }

    /// Record `frames` reported as played by the output device.
    ///
    /// Saturates at the enqueued count: a host that over-reports (some
    /// callbacks count the frames they were *asked* for, including a
    /// zero-filled underrun tail) must not be able to push `played` past
    /// `enqueued` and make [`Self::pending_frames`] wrap.
    pub fn advance(&mut self, frames: u64) {
        self.played = self.played.saturating_add(frames).min(self.enqueued);
    }

    /// Frames enqueued so far.
    #[must_use]
    pub fn enqueued_frames(&self) -> u64 {
        self.enqueued
    }

    /// Frames played so far.
    #[must_use]
    pub fn played_frames(&self) -> u64 {
        self.played
    }

    /// Frames queued but not yet played.
    #[must_use]
    pub fn pending_frames(&self) -> u64 {
        self.enqueued - self.played
    }

    /// How much audio the user has already heard.
    #[must_use]
    pub fn played_duration(&self) -> Duration {
        self.rate.frames_to_duration(self.played)
    }

    /// How much audio is still queued to be heard.
    #[must_use]
    pub fn pending_duration(&self) -> Duration {
        self.rate.frames_to_duration(self.pending_frames())
    }

    /// The value for `conversation.item.truncate`'s `audio_end_ms`.
    ///
    /// Floored, so it can only ever claim the user heard *less* than they did.
    /// Truncating early re-states a word the user already heard; truncating
    /// late leaves the model believing it said something the user never got —
    /// and that is the failure this number exists to prevent.
    #[must_use]
    pub fn audio_end_ms(&self) -> u64 {
        self.rate.frames_to_millis(self.played)
    }

    /// Milliseconds until the queue drains, rounded up.
    ///
    /// Rounded up so the Injection Gate never declares silence a fraction of a
    /// millisecond early and clips the tail of a word.
    #[must_use]
    pub fn pending_ms(&self) -> u64 {
        self.rate.frames_to_millis_ceil(self.pending_frames())
    }

    /// Whether the speaker still has audio to emit.
    ///
    /// The Injection Gate's `audioResponses.nonEmpty` term.
    #[must_use]
    pub fn is_draining(&self) -> bool {
        self.played < self.enqueued
    }

    /// Drop both counters — a `playback.clear` / barge-in.
    ///
    /// Returns the frames that were discarded unplayed, which is what a caller
    /// reports as "we cut off N ms of speech".
    pub fn clear(&mut self) -> u64 {
        let discarded = self.pending_frames();
        self.enqueued = 0;
        self.played = 0;
        discarded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_cursor_is_not_draining() {
        let cursor = PlaybackCursor::new(SampleRate::HZ_24000);
        assert!(!cursor.is_draining());
        assert_eq!(cursor.pending_frames(), 0);
        assert_eq!(cursor.audio_end_ms(), 0);
    }

    #[test]
    fn audio_end_ms_is_exact_at_the_realtime_rate() {
        let mut cursor = PlaybackCursor::new(SampleRate::HZ_24000);
        // One second of model speech queued, 250 ms of it played.
        cursor.enqueue(24_000);
        cursor.advance(6_000);
        assert_eq!(cursor.audio_end_ms(), 250);
        assert_eq!(cursor.played_duration(), Duration::from_millis(250));
        assert_eq!(cursor.pending_ms(), 750);
        assert!(cursor.is_draining());
    }

    #[test]
    fn played_millis_floor_and_pending_millis_ceil() {
        let mut cursor = PlaybackCursor::new(SampleRate::HZ_24000);
        cursor.enqueue(24_000);
        // 25 frames is 1.0416… ms.
        cursor.advance(25);
        assert_eq!(cursor.audio_end_ms(), 1);
        // 23 975 frames remain: 998.958… ms, so the gate waits 999 ms.
        assert_eq!(cursor.pending_ms(), 999);
    }

    #[test]
    fn over_reported_playback_saturates_instead_of_wrapping() {
        let mut cursor = PlaybackCursor::new(SampleRate::HZ_24000);
        cursor.enqueue(480);
        cursor.advance(1_000);
        assert_eq!(cursor.played_frames(), 480);
        assert_eq!(cursor.pending_frames(), 0);
        assert!(!cursor.is_draining());
    }

    #[test]
    fn clear_reports_what_was_cut_off() {
        let mut cursor = PlaybackCursor::new(SampleRate::HZ_24000);
        cursor.enqueue(24_000);
        cursor.advance(2_400);
        // Barge-in: the user heard 100 ms and the other 900 ms is thrown away.
        assert_eq!(cursor.audio_end_ms(), 100);
        assert_eq!(cursor.clear(), 21_600);
        assert!(!cursor.is_draining());
        assert_eq!(cursor.audio_end_ms(), 0);
    }

    #[test]
    fn a_full_drain_is_reported_as_silent() {
        let mut cursor = PlaybackCursor::new(SampleRate::HZ_24000);
        for _ in 0..10 {
            cursor.enqueue(480);
        }
        assert!(cursor.is_draining());
        cursor.advance(4_800);
        assert!(!cursor.is_draining());
        assert_eq!(cursor.pending_duration(), Duration::ZERO);
        assert_eq!(cursor.audio_end_ms(), 200);
    }
}
