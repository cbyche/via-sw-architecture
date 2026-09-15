//! The model-visible envelope a batch of finished Work is presented in.
//!
//! Ported from `server/src/voice/announcement/announcement-manager.mjs:374-403`.
//!
//! Every byte here is a catalogued `prompt-text` contract, and the shape is
//! doing real work: the `[COMPLETE]` prefix is what `PROMPT.md` refers to by
//! name, the XML-ish wrapper is what separates *these are results* from *this
//! is what the user just said*, and `--- event N ---` is what makes the "you
//! must cover every event" rule in
//! [`crate::tools::instructions::result_response_instructions`] checkable by
//! the model.

use chrono::{TimeZone, Utc};
use via_i18n::{Locale, format as i18n_format, keys, t};

use crate::text::{bounded_code_points, code_point_len, trim};

/// The marker that opens a result presentation.
///
/// **External contract** — `announcement-manager.mjs:386`. `PROMPT.md` names
/// it: *"先前工作的最终结果会以 `[COMPLETE]` 上下文到达"*.
pub const COMPLETE_MARKER: &str = "[COMPLETE]";

/// The opening tag of the results block.
///
/// **External contract** — `announcement-manager.mjs:387`, rebranded per
/// `docs/rebrand.md` (`qwen_audio_agent_work_results` → `via_work_results`).
pub const WORK_RESULTS_OPEN_TAG: &str = "<via_work_results>";

/// The closing tag of the results block.
pub const WORK_RESULTS_CLOSE_TAG: &str = "</via_work_results>";

/// The marker that opens a progress update.
///
/// **External contract** — `realtime-gateway.mjs:841`.
pub const PROGRESS_MARKER: &str = "[PROGRESS]";

/// The opening tag of a progress block.
///
/// **External contract** — `realtime-gateway.mjs:842`, rebranded per
/// `docs/rebrand.md` (`qwen_audio_agent_progress` → `via_progress`).
pub const PROGRESS_OPEN_TAG: &str = "<via_progress>";

/// The closing tag of a progress block.
pub const PROGRESS_CLOSE_TAG: &str = "</via_progress>";

/// Which terminal event an announcement carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnnouncementEvent {
    /// `task.completed` — the Work produced a result.
    Completed,
    /// `task.failed` — the Work produced an error.
    Failed,
}

impl AnnouncementEvent {
    /// The `type:` line's value.
    ///
    /// **External contract** — `announcement-manager.mjs:62,73`: the
    /// [`via_protocol::GatewayTaskEvent`] name, verbatim.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => via_protocol::GatewayTaskEvent::Completed.as_str(),
            Self::Failed => via_protocol::GatewayTaskEvent::Failed.as_str(),
        }
    }
}

/// One finished Work, queued for presentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Announcement {
    /// The Work id.
    pub work_id: String,
    /// Which terminal event this is.
    pub event: AnnouncementEvent,
    /// The user's original request.
    pub objective: String,
    /// The result text, for a completion.
    pub result: String,
    /// The error text, for a failure.
    pub error: String,
    /// The turn that submitted the Work.
    pub turn_id: Option<String>,
    /// When it finished, in epoch milliseconds.
    pub completed_at: Option<i64>,
    /// Insertion order, assigned by the manager.
    pub sequence: u64,
}

impl Announcement {
    /// A completed Work.
    #[must_use]
    pub fn completed(work_id: &str, objective: &str, result: &str) -> Self {
        Self {
            work_id: work_id.to_owned(),
            event: AnnouncementEvent::Completed,
            objective: objective.to_owned(),
            result: result.to_owned(),
            error: String::new(),
            turn_id: None,
            completed_at: None,
            sequence: 0,
        }
    }

    /// A failed Work.
    #[must_use]
    pub fn failed(work_id: &str, objective: &str, error: &str) -> Self {
        Self {
            work_id: work_id.to_owned(),
            event: AnnouncementEvent::Failed,
            objective: objective.to_owned(),
            result: String::new(),
            error: error.to_owned(),
            turn_id: None,
            completed_at: None,
            sequence: 0,
        }
    }

    /// Set the submitting turn.
    #[must_use]
    pub fn turn(mut self, turn_id: Option<String>) -> Self {
        self.turn_id = turn_id;
        self
    }

    /// Set the completion instant.
    #[must_use]
    pub const fn at(mut self, completed_at: Option<i64>) -> Self {
        self.completed_at = completed_at;
        self
    }

    /// This announcement's `--- event N ---` block.
    ///
    /// **External contract** — `announcement-manager.mjs:374-384`. Field order
    /// is fixed; `original_request` and `completed_at` are **omitted entirely**
    /// when empty rather than rendered blank, because `.filter(Boolean)` drops
    /// them. The `result:` / `error:` line always renders, even empty, because
    /// its own value is not what is filtered.
    #[must_use]
    pub fn block(&self, index: usize) -> String {
        let mut lines = vec![
            format!("--- event {} ---", index + 1),
            format!("type: {}", self.event.as_str()),
            format!("work_id: {}", self.work_id),
        ];
        if !self.objective.is_empty() {
            lines.push(format!("original_request: {}", self.objective));
        }
        if let Some(completed_at) = self.completed_at
            && let Some(rendered) = iso_instant(completed_at)
        {
            lines.push(format!("completed_at: {rendered}"));
        }
        lines.push(match self.event {
            AnnouncementEvent::Completed => format!("result:\n{}", trim(&self.result)),
            AnnouncementEvent::Failed => format!("error:\n{}", trim(&self.error)),
        });
        lines.join("\n")
    }
}

/// `new Date(ms).toISOString()`.
///
/// `None` for an instant outside the representable range, which upstream would
/// render as `Invalid Date` and then throw on. Dropping the line is the
/// conservative answer: the announcement still carries the result.
fn iso_instant(epoch_ms: i64) -> Option<String> {
    Utc.timestamp_millis_opt(epoch_ms)
        .single()
        .map(|instant| instant.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
}

/// The whole model-visible envelope for a batch.
///
/// **External contract** — `announcement-manager.mjs:374-392`.
#[must_use]
pub fn format_work_results(announcements: &[Announcement], locale: Locale) -> String {
    let mut lines = vec![
        COMPLETE_MARKER.to_owned(),
        WORK_RESULTS_OPEN_TAG.to_owned(),
        t(locale, keys::REALTIME_WORK_RESULTS_HEADER).to_owned(),
    ];
    for (index, announcement) in announcements.iter().enumerate() {
        lines.push(announcement.block(index));
    }
    lines.push(WORK_RESULTS_CLOSE_TAG.to_owned());
    lines.join("\n")
}

/// The model-visible envelope for one progress update.
///
/// **External contract** — `realtime-gateway.mjs:841-848`. Unlike a result,
/// this is explicitly labelled *not the final result*, because a model that
/// treats a progress line as a completion tells the user the work is done.
#[must_use]
pub fn format_progress(message: &str, locale: Locale) -> String {
    [
        PROGRESS_MARKER,
        PROGRESS_OPEN_TAG,
        t(locale, keys::REALTIME_PROGRESS_INSTRUCTIONS),
        message,
        PROGRESS_CLOSE_TAG,
    ]
    .join("\n")
}

/// Clip `value` to `max_chars` code points, appending the truncation notice.
///
/// **External contract** — `announcement-manager.mjs:394-403`. Three
/// behaviours worth naming:
///
/// - the bound counts **code points** (`Array.from(text)`), not UTF-16 units,
///   so a CJK result is not silently halved;
/// - `max_chars` is floored at 1, so a misconfigured `0` truncates to one
///   character rather than producing an empty announcement;
/// - when the limit is no larger than the suffix itself, the suffix is dropped
///   and the text is hard-clipped, because a notice longer than the budget
///   would push the real content out entirely.
#[must_use]
pub fn truncate_result(value: &str, max_chars: usize, locale: Locale) -> String {
    let text = trim(value);
    let limit = max_chars.max(1);
    if code_point_len(text) <= limit {
        return text.to_owned();
    }
    let suffix = t(locale, keys::REALTIME_RESULT_TRUNCATED_SUFFIX);
    let suffix_len = code_point_len(suffix);
    if limit <= suffix_len {
        return bounded_code_points(text, limit);
    }
    format!("{}{suffix}", bounded_code_points(text, limit - suffix_len))
}

/// The `<restored_context>` block replayed after a reconnect.
///
/// **External contract** — `realtime-provider.mjs:250-265`. The gateway's own
/// wrapper around [`via_conversation::build_recent_conversation_context`]; the
/// explicit *"not a new request"* line is what stops the model answering a
/// question it already answered before the socket dropped.
#[must_use]
pub fn format_restored_context(recent: &str, locale: Locale) -> String {
    [
        "<restored_context>",
        t(locale, keys::REALTIME_RESTORED_CONTEXT_INSTRUCTIONS),
        recent,
        "</restored_context>",
    ]
    .join("\n")
}

/// The silent note injected when a permission was resolved elsewhere.
///
/// **External contract** — `realtime-gateway.mjs:893-897`. Without it a model
/// that already asked the question aloud keeps asking, or reports the user's
/// later spoken confirmation as a request that has expired.
#[must_use]
pub fn permission_resolved_note(locale: Locale) -> String {
    t(locale, keys::REALTIME_PERMISSION_RESOLVED_NOTE).to_owned()
}

/// The `already asleep` refusal, carrying the configured wake phrase.
///
/// **External contract** — `realtime-gateway.mjs:2046-2049`. VIA's wake phrase
/// is configuration, never a literal (`docs/architecture.md` §16), so the key
/// takes a `{wake_word}` placeholder where upstream hard-coded its own.
#[must_use]
pub fn asleep_message(wake_word: &str, locale: Locale) -> String {
    i18n_format(locale, keys::REALTIME_ASLEEP, &[("wake_word", wake_word)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn a_completed_block_renders_every_field_in_order() {
        let announcement = Announcement::completed("work_1", "查一下天气", "晴，26 度")
            .at(Some(1_700_000_000_000));
        assert_eq!(
            announcement.block(0),
            concat!(
                "--- event 1 ---\n",
                "type: task.completed\n",
                "work_id: work_1\n",
                "original_request: 查一下天气\n",
                "completed_at: 2023-11-14T22:13:20.000Z\n",
                "result:\n晴，26 度",
            ),
        );
    }

    #[test]
    fn a_failed_block_carries_the_error_line_instead() {
        let announcement = Announcement::failed("work_2", "跑测试", "端口被占用");
        assert_eq!(
            announcement.block(1),
            "--- event 2 ---\ntype: task.failed\nwork_id: work_2\noriginal_request: 跑测试\nerror:\n端口被占用",
        );
    }

    #[test]
    fn empty_optional_lines_are_omitted_rather_than_blank() {
        let announcement = Announcement::completed("work_3", "", "done");
        let block = announcement.block(0);
        assert!(!block.contains("original_request"));
        assert!(!block.contains("completed_at"));
        assert_eq!(
            block,
            "--- event 1 ---\ntype: task.completed\nwork_id: work_3\nresult:\ndone",
        );
    }

    #[test]
    fn the_envelope_wraps_every_block() {
        let text = format_work_results(
            &[
                Announcement::completed("work_1", "a", "one"),
                Announcement::failed("work_2", "b", "two"),
            ],
            Locale::Zh,
        );
        assert!(text.starts_with("[COMPLETE]\n<via_work_results>\n"));
        assert!(text.ends_with("\n</via_work_results>"));
        assert!(text.contains("--- event 1 ---"));
        assert!(text.contains("--- event 2 ---"));
        assert!(text.contains(t(Locale::Zh, keys::REALTIME_WORK_RESULTS_HEADER)));
        assert!(!text.contains("qwen"), "the wrapper is rebranded");
    }

    #[test]
    fn an_empty_batch_still_produces_a_well_formed_envelope() {
        let text = format_work_results(&[], Locale::Zh);
        assert_eq!(
            text,
            format!(
                "[COMPLETE]\n<via_work_results>\n{}\n</via_work_results>",
                t(Locale::Zh, keys::REALTIME_WORK_RESULTS_HEADER),
            ),
        );
    }

    #[test]
    fn truncation_counts_code_points_and_appends_the_notice() {
        let long = "阿".repeat(200);
        let clipped = truncate_result(&long, 50, Locale::Zh);
        assert_eq!(code_point_len(&clipped), 50);
        assert!(clipped.ends_with(t(Locale::Zh, keys::REALTIME_RESULT_TRUNCATED_SUFFIX)));
    }

    #[test]
    fn text_within_the_budget_is_returned_trimmed_and_whole() {
        assert_eq!(truncate_result("  hello  ", 100, Locale::Zh), "hello");
        let exact = "阿".repeat(10);
        assert_eq!(truncate_result(&exact, 10, Locale::Zh), exact);
    }

    #[test]
    fn a_budget_no_larger_than_the_notice_hard_clips_instead() {
        let suffix_len = code_point_len(t(Locale::Zh, keys::REALTIME_RESULT_TRUNCATED_SUFFIX));
        let long = "阿".repeat(200);
        let clipped = truncate_result(&long, suffix_len, Locale::Zh);
        assert_eq!(code_point_len(&clipped), suffix_len);
        assert!(!clipped.contains('…'));
    }

    #[test]
    fn a_zero_budget_is_floored_at_one_character() {
        let clipped = truncate_result("abcdef", 0, Locale::Zh);
        assert_eq!(clipped, "a");
    }

    #[test]
    fn an_unrepresentable_completion_instant_drops_only_its_line() {
        let announcement = Announcement::completed("work_1", "x", "y").at(Some(i64::MAX));
        let block = announcement.block(0);
        assert!(!block.contains("completed_at"));
        assert!(block.contains("result:\ny"));
    }

    #[test]
    fn the_progress_envelope_says_it_is_not_a_result() {
        let text = format_progress("已经跑完一半", Locale::Zh);
        assert!(text.starts_with("[PROGRESS]\n<via_progress>\n"));
        assert!(text.ends_with("已经跑完一半\n</via_progress>"));
        assert!(!text.contains("qwen"));
    }

    #[test]
    fn the_restored_context_block_is_labelled_as_not_a_new_request() {
        let text =
            format_restored_context("<recent_conversation>x</recent_conversation>", Locale::Zh);
        assert!(text.starts_with("<restored_context>\n"));
        assert!(text.ends_with("\n</restored_context>"));
        assert!(text.contains(t(Locale::Zh, keys::REALTIME_RESTORED_CONTEXT_INSTRUCTIONS)));
    }

    #[test]
    fn the_asleep_message_carries_the_configured_wake_phrase() {
        let message = asleep_message("你好 VIA", Locale::Zh);
        assert!(message.contains("你好 VIA"));
        // Spelled by code point so this assertion does not itself trip
        // `scripts/brand_leak.py`, whose whole job is to fail on that literal.
        let upstream_identity: String = ['\u{5343}', '\u{95ee}'].iter().collect();
        assert!(
            !message.contains(&upstream_identity),
            "the wake phrase is configuration, never a literal",
        );
    }

    #[test]
    fn the_event_names_come_from_via_protocol() {
        assert_eq!(AnnouncementEvent::Completed.as_str(), "task.completed");
        assert_eq!(AnnouncementEvent::Failed.as_str(), "task.failed");
    }
}
