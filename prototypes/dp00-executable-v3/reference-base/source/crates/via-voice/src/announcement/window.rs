//! The announcement window — may a finished result be spoken *now*?
//!
//! Ported from `server/src/voice/announcement/announcement-window.mjs`.
//!
//! This is the heart of the Injection Gate (`docs/architecture.md` §3). ARGO
//! lists the problem it solves under "Absent by design — do not debug these":
//! *delegation results can land mid-sentence*. The window is the state that
//! makes "not mid-sentence" decidable, and its three inputs arrive as the
//! `playback.started` / `playback.ended` / `playback.cancelled` client events —
//! which is why porting qwen's WebSocket vocabulary whole is load-bearing.
//!
//! # The predicate
//!
//! ```text
//! blocked = userSpeaking || turnPending || audioResponses.nonEmpty
//! ```
//!
//! Three different "someone is mid-sentence" conditions, and each closes a
//! distinct hole:
//!
//! - **`user_speaking`** — the user has the floor. Speaking over them is the
//!   most obvious failure and the only one VAD alone would catch.
//! - **`turn_pending`** — the user finished speaking but the model has not
//!   answered yet. Without this, a result that completes in the gap between
//!   speech-stopped and response-created is spoken *instead of* the answer to
//!   the question just asked.
//! - **`audio_responses` non-empty** — a response's audio has been queued to
//!   the client but has not finished playing. Generation finishing is not
//!   playback finishing, and the client is the only party that knows the
//!   difference.
//!
//! The gateway wraps it as
//! `sleeping || waking || !output_enabled || window.is_blocked()`; see
//! [`crate::gate::InjectionGate`].
//!
//! # Why `turn_pending` clears the way it does
//!
//! [`AnnouncementWindow::response_done`] clears it only for a response that
//! belongs to the **active** turn, carries no audio, and either called no tool
//! or had that call suppressed or failed. A response with audio is not done
//! until playback ends ([`AnnouncementWindow::finish_playback`]); a response
//! that called a tool is not the answer, it is the *request* for one, and the
//! turn is still pending until the follow-up lands.

use indexmap::{IndexMap, IndexSet};

use crate::response::ResponseOrigin;

/// One queued audio response, as the window tracks it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct QueuedAudio {
    turn_id: String,
    origin: ResponseOrigin,
}

/// What a `response.done` looked like, for the window's purposes.
///
/// **External contract** — `announcement-window.mjs:20-27`, the destructured
/// `responseDone({...})` argument.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResponseDone {
    /// The turn this response answered.
    pub turn_id: String,
    /// Where the response came from.
    pub origin: ResponseOrigin,
    /// Whether it emitted audio.
    pub has_audio: bool,
    /// Whether it authored a tool call.
    pub has_function_call: bool,
    /// Whether it was cancelled or superseded.
    pub suppressed: bool,
    /// Whether `response.done` reported a failure status.
    pub failed: bool,
}

/// The blocking state of one voice session.
#[derive(Clone, Debug, Default)]
pub struct AnnouncementWindow {
    user_speaking: bool,
    active_turn_id: String,
    turn_pending: bool,
    audio_responses: IndexMap<String, QueuedAudio>,
    playing_responses: IndexSet<String>,
}

impl AnnouncementWindow {
    /// A window with nothing in flight.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The user started a turn.
    ///
    /// **External contract** — `announcement-window.mjs:10-14`. Called from
    /// `input_audio_buffer.speech_started` and, deliberately, from a typed
    /// `input.message` too: a text submission is a first-class user turn and
    /// must block a result exactly as speech does.
    pub fn begin_turn(&mut self, turn_id: &str) {
        self.user_speaking = true;
        self.active_turn_id = turn_id.to_owned();
        self.turn_pending = true;
    }

    /// The user stopped speaking. The turn stays pending.
    pub fn end_speech(&mut self) {
        self.user_speaking = false;
    }

    /// A response finished generating.
    ///
    /// **External contract** — `announcement-window.mjs:20-35`. Four reasons to
    /// leave the turn pending, in upstream's order:
    ///
    /// 1. an announcement's own response never satisfies a user turn;
    /// 2. a response for some other turn says nothing about this one;
    /// 3. a response with audio is not finished until it has been *played*;
    /// 4. a live tool call means the answer is still coming — unless the call
    ///    was suppressed or the response failed, in which case nothing more is
    ///    coming and holding the window open would deadlock it.
    pub fn response_done(&mut self, done: &ResponseDone) {
        if done.origin == ResponseOrigin::Announcement
            || done.turn_id != self.active_turn_id
            || done.has_audio
            || (done.has_function_call && !done.suppressed && !done.failed)
        {
            return;
        }
        self.turn_pending = false;
    }

    /// Record that `response_id` has audio queued at the client.
    ///
    /// **External contract** — `announcement-window.mjs:37-44`. An empty id is
    /// ignored: a provider that omits the response id has given the gateway
    /// nothing it could later retire, and a permanently un-retirable entry
    /// would block every future announcement.
    pub fn queue_audio(&mut self, response_id: &str, turn_id: &str, origin: ResponseOrigin) {
        if response_id.is_empty() {
            return;
        }
        self.audio_responses.insert(
            response_id.to_owned(),
            QueuedAudio {
                turn_id: turn_id.to_owned(),
                origin,
            },
        );
    }

    /// The client reported `playback.started` for `response_id`.
    pub fn start_playback(&mut self, response_id: &str) {
        if !response_id.is_empty() {
            self.playing_responses.insert(response_id.to_owned());
        }
    }

    /// The client reported `playback.ended` or `playback.cancelled`.
    ///
    /// **External contract** — `announcement-window.mjs:51-64`. Retiring the
    /// queued entry is unconditional — the audio is gone either way — but
    /// clearing `turn_pending` needs the same four conditions
    /// [`Self::response_done`] applies, minus the audio one, which playback
    /// ending has just satisfied.
    pub fn finish_playback(&mut self, response_id: &str, has_function_call: bool) {
        let context = self.audio_responses.shift_remove(response_id);
        self.playing_responses.shift_remove(response_id);
        if let Some(context) = context
            && context.origin != ResponseOrigin::Announcement
            && !context.turn_id.is_empty()
            && context.turn_id == self.active_turn_id
            && !has_function_call
        {
            self.turn_pending = false;
        }
    }

    /// Barge-in: the user cut the turn short.
    ///
    /// Clears only `turn_pending`. Queued audio is retired by the
    /// `playback.cancelled` receipts the client sends for it — the gateway
    /// does not get to assume the client dropped what it was told to drop.
    pub fn interrupt(&mut self) {
        self.turn_pending = false;
    }

    /// Drop everything. Sleep, deactivation, mute and socket close all use it.
    pub fn reset(&mut self) {
        self.user_speaking = false;
        self.active_turn_id.clear();
        self.turn_pending = false;
        self.audio_responses.clear();
        self.playing_responses.clear();
    }

    /// **The predicate.** `userSpeaking || turnPending || audioResponses > 0`.
    ///
    /// **External contract** — `announcement-window.mjs:78-84`, and
    /// `docs/architecture.md` §11 invariant 3.
    #[must_use]
    pub fn is_blocked(&self) -> bool {
        self.user_speaking || self.turn_pending || !self.audio_responses.is_empty()
    }

    /// Whether any response is actually playing right now.
    ///
    /// Distinct from [`Self::is_blocked`]: audio can be *queued* without having
    /// started, and that still blocks.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        !self.playing_responses.is_empty()
    }

    /// Whether the user has the floor.
    #[must_use]
    pub const fn user_speaking(&self) -> bool {
        self.user_speaking
    }

    /// Whether a user turn is still waiting for its answer.
    #[must_use]
    pub const fn turn_pending(&self) -> bool {
        self.turn_pending
    }

    /// The turn the window is tracking.
    #[must_use]
    pub fn active_turn_id(&self) -> &str {
        &self.active_turn_id
    }

    /// How many responses have audio queued at the client.
    #[must_use]
    pub fn queued_audio_len(&self) -> usize {
        self.audio_responses.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model_done(turn_id: &str) -> ResponseDone {
        ResponseDone {
            turn_id: turn_id.to_owned(),
            origin: ResponseOrigin::Model,
            ..ResponseDone::default()
        }
    }

    #[test]
    fn a_fresh_window_is_open() {
        assert!(!AnnouncementWindow::new().is_blocked());
    }

    #[test]
    fn each_of_the_three_conditions_blocks_on_its_own() {
        let mut speaking = AnnouncementWindow::new();
        speaking.begin_turn("voice-1");
        speaking.turn_pending = false;
        assert!(speaking.is_blocked(), "user speaking blocks");

        let mut pending = AnnouncementWindow::new();
        pending.begin_turn("voice-1");
        pending.end_speech();
        assert!(pending.is_blocked(), "a pending turn blocks");

        let mut queued = AnnouncementWindow::new();
        queued.queue_audio("resp_1", "voice-1", ResponseOrigin::Model);
        assert!(queued.is_blocked(), "queued audio blocks");
    }

    #[test]
    fn a_silent_answer_to_the_active_turn_opens_the_window() {
        let mut window = AnnouncementWindow::new();
        window.begin_turn("voice-1");
        window.end_speech();
        window.response_done(&model_done("voice-1"));
        assert!(!window.is_blocked());
    }

    #[test]
    fn an_announcements_own_response_never_clears_the_turn() {
        let mut window = AnnouncementWindow::new();
        window.begin_turn("voice-1");
        window.end_speech();
        window.response_done(&ResponseDone {
            origin: ResponseOrigin::Announcement,
            ..model_done("voice-1")
        });
        assert!(window.is_blocked());
    }

    #[test]
    fn a_response_for_another_turn_says_nothing_about_this_one() {
        let mut window = AnnouncementWindow::new();
        window.begin_turn("voice-2");
        window.end_speech();
        window.response_done(&model_done("voice-1"));
        assert!(window.is_blocked());
    }

    #[test]
    fn a_response_with_audio_stays_pending_until_playback_ends() {
        let mut window = AnnouncementWindow::new();
        window.begin_turn("voice-1");
        window.end_speech();
        window.queue_audio("resp_1", "voice-1", ResponseOrigin::Model);
        window.response_done(&ResponseDone {
            has_audio: true,
            ..model_done("voice-1")
        });
        assert!(
            window.turn_pending(),
            "generation finishing is not playback finishing"
        );
        window.start_playback("resp_1");
        assert!(window.is_playing());
        window.finish_playback("resp_1", false);
        assert!(!window.is_blocked());
        assert!(!window.is_playing());
    }

    #[test]
    fn a_live_tool_call_keeps_the_turn_pending_but_a_dead_one_does_not() {
        let mut live = AnnouncementWindow::new();
        live.begin_turn("voice-1");
        live.end_speech();
        live.response_done(&ResponseDone {
            has_function_call: true,
            ..model_done("voice-1")
        });
        assert!(live.is_blocked(), "the answer is still coming");

        for release in [
            ResponseDone {
                has_function_call: true,
                suppressed: true,
                ..model_done("voice-1")
            },
            ResponseDone {
                has_function_call: true,
                failed: true,
                ..model_done("voice-1")
            },
        ] {
            let mut window = AnnouncementWindow::new();
            window.begin_turn("voice-1");
            window.end_speech();
            window.response_done(&release);
            assert!(!window.is_blocked(), "nothing more is coming: {release:?}");
        }
    }

    #[test]
    fn finishing_playback_of_a_tool_call_response_keeps_the_turn_pending() {
        let mut window = AnnouncementWindow::new();
        window.begin_turn("voice-1");
        window.end_speech();
        window.queue_audio("resp_1", "voice-1", ResponseOrigin::Model);
        window.finish_playback("resp_1", true);
        assert!(window.turn_pending());
        assert_eq!(window.queued_audio_len(), 0, "the audio is still retired");
    }

    #[test]
    fn finishing_an_announcements_playback_never_clears_the_users_turn() {
        let mut window = AnnouncementWindow::new();
        window.begin_turn("voice-1");
        window.end_speech();
        window.queue_audio("resp_1", "voice-1", ResponseOrigin::Announcement);
        window.finish_playback("resp_1", false);
        assert!(window.turn_pending());
    }

    #[test]
    fn an_empty_response_id_is_never_queued() {
        let mut window = AnnouncementWindow::new();
        window.queue_audio("", "voice-1", ResponseOrigin::Model);
        assert!(
            !window.is_blocked(),
            "an unretirable entry would deadlock the gate"
        );
        window.start_playback("");
        assert!(!window.is_playing());
    }

    #[test]
    fn finishing_an_unknown_response_is_a_no_op() {
        let mut window = AnnouncementWindow::new();
        window.begin_turn("voice-1");
        window.end_speech();
        window.finish_playback("never_seen", false);
        assert!(window.turn_pending(), "an unknown id proves nothing");
    }

    #[test]
    fn interrupt_clears_the_turn_but_leaves_queued_audio_to_its_receipts() {
        let mut window = AnnouncementWindow::new();
        window.begin_turn("voice-1");
        window.end_speech();
        window.queue_audio("resp_1", "voice-1", ResponseOrigin::Model);
        window.interrupt();
        assert!(!window.turn_pending());
        assert!(
            window.is_blocked(),
            "the client has not confirmed the drop yet"
        );
        window.finish_playback("resp_1", false);
        assert!(!window.is_blocked());
    }

    #[test]
    fn reset_clears_every_field() {
        let mut window = AnnouncementWindow::new();
        window.begin_turn("voice-1");
        window.queue_audio("resp_1", "voice-1", ResponseOrigin::Model);
        window.start_playback("resp_1");
        window.reset();
        assert!(!window.is_blocked());
        assert!(!window.is_playing());
        assert!(window.active_turn_id().is_empty());
        assert!(!window.user_speaking());
    }

    #[test]
    fn requeuing_the_same_response_id_replaces_rather_than_duplicates() {
        let mut window = AnnouncementWindow::new();
        window.queue_audio("resp_1", "voice-1", ResponseOrigin::Model);
        window.queue_audio("resp_1", "voice-2", ResponseOrigin::Announcement);
        assert_eq!(window.queued_audio_len(), 1);
        window.finish_playback("resp_1", false);
        assert_eq!(window.queued_audio_len(), 0);
    }

    #[test]
    fn a_queued_response_with_no_turn_id_never_clears_the_turn() {
        // `context.turnId &&` in upstream. A response the gateway could not
        // attribute must not be able to satisfy an arbitrary pending turn.
        let mut window = AnnouncementWindow::new();
        window.begin_turn("");
        window.end_speech();
        window.queue_audio("resp_1", "", ResponseOrigin::Model);
        window.finish_playback("resp_1", false);
        assert!(window.turn_pending());
    }
}
