//! **The Injection Gate** — may a finished result be spoken *now*?
//!
//! `docs/architecture.md` §3 and §11 invariant 3. ARGO lists the problem under
//! *"Absent by design — do not debug these"*: **delegation results can land
//! mid-sentence**, because injection is queued the moment the result exists and
//! holding it until neither side is speaking needs playback position from the
//! client. qwen's client protocol already carries that position, as
//! `playback.started` / `playback.ended` / `playback.cancelled`, which is why
//! porting its WebSocket vocabulary whole is load-bearing rather than
//! incidental.
//!
//! # The predicate
//!
//! ```text
//! blocked = sleeping || waking || !output_enabled || window.is_blocked()
//! ```
//!
//! where `window.is_blocked()` is
//! `user_speaking || turn_pending || audio_responses.nonEmpty`
//! ([`crate::announcement::AnnouncementWindow`]).
//!
//! The three outer flags are not redundant with the window:
//!
//! - **`sleeping`** — the realtime socket is closed. There is no session to
//!   speak through, and injecting would silently drop the result.
//! - **`waking`** — the socket is mid-handshake. A response created now races
//!   the session update that has not landed.
//! - **`!output_enabled`** — this client does not own the speaker. Another
//!   frontend does, and it will claim the notification itself.
//!
//! # The drain predicate
//!
//! [`via_audio::PlaybackCursor`] is what makes "the client is still playing"
//! answerable between receipts. `playback.started` enqueues the response's
//! frames, `playback.ended` advances past them, and
//! [`PlaybackCursor::is_draining`] is true for exactly the interval in
//! between — so a receipt lost to a dropped socket still leaves the gate
//! closed for the audio's own duration rather than forever.
//!
//! # The claim
//!
//! A renewable claim prevents two live frontends presenting the same result;
//! it lives on the Work manager and is driven by
//! [`crate::announcement::NotificationClaims`].

use via_audio::{PlaybackCursor, SampleRate};

use crate::announcement::AnnouncementWindow;

/// The gateway flags that wrap the announcement window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GateFlags {
    /// The realtime socket is closed and the session is wake-word-only.
    pub sleeping: bool,
    /// The socket is coming back.
    pub waking: bool,
    /// This client owns the speaker.
    pub output_enabled: bool,
}

/// The Injection Gate.
///
/// Owns the announcement window, the gateway flags and the playback cursor, so
/// the predicate is answered in one place instead of being spelled out at each
/// of its four call sites.
#[derive(Debug)]
pub struct InjectionGate {
    window: AnnouncementWindow,
    flags: GateFlags,
    cursor: PlaybackCursor,
}

impl InjectionGate {
    /// A gate for a session whose response audio arrives at `rate`.
    #[must_use]
    pub fn new(rate: SampleRate) -> Self {
        Self {
            window: AnnouncementWindow::new(),
            flags: GateFlags::default(),
            cursor: PlaybackCursor::new(rate),
        }
    }

    /// **The predicate.**
    ///
    /// **External contract** — `realtime-gateway.mjs:266`
    /// (`isDeliveryBlocked`).
    #[must_use]
    pub fn is_blocked(&self) -> bool {
        self.flags.sleeping
            || self.flags.waking
            || !self.flags.output_enabled
            || self.window.is_blocked()
    }

    /// The announcement window, for the events that drive it.
    #[must_use]
    pub const fn window(&self) -> &AnnouncementWindow {
        &self.window
    }

    /// The announcement window, mutably.
    pub const fn window_mut(&mut self) -> &mut AnnouncementWindow {
        &mut self.window
    }

    /// The playback cursor.
    #[must_use]
    pub const fn cursor(&self) -> &PlaybackCursor {
        &self.cursor
    }

    /// The gateway flags.
    #[must_use]
    pub const fn flags(&self) -> GateFlags {
        self.flags
    }

    /// Replace the gateway flags.
    pub const fn set_flags(&mut self, flags: GateFlags) {
        self.flags = flags;
    }

    /// This client took or lost the speaker.
    pub const fn set_output_enabled(&mut self, enabled: bool) {
        self.flags.output_enabled = enabled;
    }

    /// The session entered or left sleep.
    pub const fn set_sleeping(&mut self, sleeping: bool) {
        self.flags.sleeping = sleeping;
    }

    /// The session started or finished waking.
    pub const fn set_waking(&mut self, waking: bool) {
        self.flags.waking = waking;
    }

    /// The client began playing a response's audio.
    ///
    /// `frames` is how much audio was queued for it; `0` when the gateway did
    /// not count, which leaves the cursor inert and the window authoritative.
    pub fn playback_started(&mut self, response_id: &str, frames: u64) {
        self.window.start_playback(response_id);
        if frames > 0 {
            self.cursor.enqueue(frames);
        }
    }

    /// The client finished, or dropped, a response's audio.
    ///
    /// Advancing the cursor by everything still pending is deliberate: the
    /// client is the authority on what it played, and a `playback.cancelled`
    /// means the rest will never be played at all.
    pub fn playback_finished(&mut self, response_id: &str, has_function_call: bool) {
        self.window.finish_playback(response_id, has_function_call);
        if !self.window.is_playing() {
            let pending = self.cursor.pending_frames();
            if pending > 0 {
                self.cursor.advance(pending);
            }
        }
    }

    /// Whether the client is still draining queued audio.
    ///
    /// A second opinion on top of the window: a `playback.ended` lost to a
    /// dropped socket leaves the window clean, and this keeps the gate honest
    /// for the audio's own remaining duration.
    #[must_use]
    pub fn is_draining(&self) -> bool {
        self.cursor.is_draining()
    }

    /// How much audio is still queued at the client, in milliseconds.
    #[must_use]
    pub fn pending_playback_ms(&self) -> u64 {
        self.cursor.pending_ms()
    }

    /// Drop every playback and window state.
    pub fn reset(&mut self) {
        self.window.reset();
        self.cursor.clear();
    }
}

/// Whether a client's playback receipt may be acted on.
///
/// **External contract** — `realtime-gateway.mjs:95-101`
/// (`acceptsPlaybackReceipt`). All three, and each rejects a real case:
///
/// - `output_enabled` — a client that does not own the speaker cannot report
///   what the speaker did;
/// - `active` — nor can one that lost the voice slot;
/// - `response_known` — a receipt for a response this session never created is
///   either a stale frame from a previous connection or a forged one, and
///   acting on it would confirm a notification that was never spoken.
#[must_use]
pub const fn accepts_playback_receipt(
    output_enabled: bool,
    active: bool,
    response_known: bool,
) -> bool {
    output_enabled && active && response_known
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One named way to break an otherwise-passing input.
    type Mutation = (&'static str, fn(&mut InjectionGate));
    use crate::response::ResponseOrigin;
    use pretty_assertions::assert_eq;

    fn open_gate() -> InjectionGate {
        let mut gate = InjectionGate::new(SampleRate::HZ_24000);
        gate.set_output_enabled(true);
        gate
    }

    #[test]
    fn an_idle_session_with_the_speaker_is_open() {
        assert!(!open_gate().is_blocked());
    }

    #[test]
    fn each_outer_flag_blocks_on_its_own() {
        let mutations: [Mutation; 3] = [
            ("sleeping", |gate| gate.set_sleeping(true)),
            ("waking", |gate| gate.set_waking(true)),
            ("no speaker", |gate| gate.set_output_enabled(false)),
        ];
        for (why, mutate) in mutations {
            let mut gate = open_gate();
            mutate(&mut gate);
            assert!(gate.is_blocked(), "{why} did not block");
        }
    }

    #[test]
    fn the_window_still_blocks_a_session_that_owns_the_speaker() {
        let mut gate = open_gate();
        gate.window_mut().begin_turn("voice-1");
        assert!(gate.is_blocked());
        gate.window_mut().end_speech();
        assert!(gate.is_blocked(), "the turn is still pending");
        gate.window_mut().interrupt();
        assert!(!gate.is_blocked());
    }

    #[test]
    fn a_default_gate_is_blocked_because_it_owns_no_speaker() {
        // The default must be closed: a session that has not claimed the
        // speaker must never speak into it.
        assert!(InjectionGate::new(SampleRate::HZ_24000).is_blocked());
    }

    #[test]
    fn the_cursor_reports_draining_between_the_two_receipts() {
        let mut gate = open_gate();
        gate.window_mut()
            .queue_audio("resp_1", "voice-1", ResponseOrigin::Announcement);
        assert!(!gate.is_draining(), "nothing has started playing");
        // 24 000 frames at 24 kHz is one second of audio.
        gate.playback_started("resp_1", 24_000);
        assert!(gate.is_draining());
        assert_eq!(gate.pending_playback_ms(), 1_000);
        gate.playback_finished("resp_1", false);
        assert!(!gate.is_draining());
        assert_eq!(gate.pending_playback_ms(), 0);
        assert!(
            !gate.is_blocked(),
            "the window retired the queued audio too"
        );
    }

    #[test]
    fn a_playback_start_with_no_frame_count_leaves_the_cursor_inert() {
        let mut gate = open_gate();
        gate.playback_started("resp_1", 0);
        assert!(!gate.is_draining());
        assert!(gate.window().is_playing(), "the window still knows");
    }

    #[test]
    fn a_cancelled_playback_drains_the_audio_that_will_never_be_played() {
        let mut gate = open_gate();
        gate.window_mut()
            .queue_audio("resp_1", "voice-1", ResponseOrigin::Model);
        gate.playback_started("resp_1", 48_000);
        assert_eq!(gate.pending_playback_ms(), 2_000);
        // `playback.cancelled` — the client dropped the rest.
        gate.playback_finished("resp_1", false);
        assert_eq!(gate.pending_playback_ms(), 0);
    }

    #[test]
    fn two_overlapping_responses_drain_only_when_both_finish() {
        let mut gate = open_gate();
        for id in ["resp_1", "resp_2"] {
            gate.window_mut()
                .queue_audio(id, "voice-1", ResponseOrigin::Model);
            gate.playback_started(id, 24_000);
        }
        assert_eq!(gate.pending_playback_ms(), 2_000);
        gate.playback_finished("resp_1", false);
        assert!(gate.is_draining(), "the second is still playing");
        gate.playback_finished("resp_2", false);
        assert!(!gate.is_draining());
    }

    #[test]
    fn reset_clears_the_window_and_the_cursor() {
        let mut gate = open_gate();
        gate.window_mut()
            .queue_audio("resp_1", "voice-1", ResponseOrigin::Model);
        gate.playback_started("resp_1", 24_000);
        gate.reset();
        assert!(!gate.is_draining());
        assert!(!gate.is_blocked());
        assert!(!gate.window().is_playing());
    }

    #[test]
    fn a_playback_receipt_needs_all_three_conditions() {
        assert!(accepts_playback_receipt(true, true, true));
        assert!(!accepts_playback_receipt(false, true, true));
        assert!(!accepts_playback_receipt(true, false, true));
        assert!(
            !accepts_playback_receipt(true, true, false),
            "a receipt for an unknown response must never confirm a notification",
        );
    }

    #[test]
    fn setting_the_flags_wholesale_agrees_with_the_setters() {
        let mut wholesale = InjectionGate::new(SampleRate::HZ_24000);
        wholesale.set_flags(GateFlags {
            sleeping: false,
            waking: false,
            output_enabled: true,
        });
        let mut piecemeal = InjectionGate::new(SampleRate::HZ_24000);
        piecemeal.set_output_enabled(true);
        assert_eq!(wholesale.flags(), piecemeal.flags());
        assert_eq!(wholesale.is_blocked(), piecemeal.is_blocked());
    }
}
