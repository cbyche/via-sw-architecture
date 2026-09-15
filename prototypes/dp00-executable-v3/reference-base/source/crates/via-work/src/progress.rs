//! The long-running-work announcement.
//!
//! `server/src/task/task-manager.mjs:583-641`. Every
//! `progressCheckMs` — 300 seconds by default — a `work`-kind Work that is
//! still active composes one sentence from its newest activity and emits it as
//! `task.progress.check`. The voice layer speaks it; the host shows it as a
//! desktop notification when nobody is listening.
//!
//! Contract — `docs/reference/contracts.json` (`prompt-text` / *background work
//! progress-check message*), including the note that the curly quotes are
//! literal. They are, and they live in the `zh` catalog entry, where the rest
//! of the sentence does.
//!
//! # Only `work`
//!
//! `create()` gives a `progressCheckMs` to `kind: 'work'` and to nothing else
//! (`task-manager.mjs:391-393`), and `start()` arms the timer only for `work`
//! (`:583`). Reminders and scheduled tasks stay quiet until they finish, fail,
//! or need a permission — which is what `docs/reference/contracts.json`'s *Work
//! kind values* means by *"`work` is the only kind that gets a
//! `progressCheckMs`"*.

use via_i18n::{Key, Locale, format as i18n_format, keys};

use crate::activity::Activity;
use crate::text::slice_units;

/// The default announcement cadence.
///
/// Contract — `VIA_BACKGROUND_TASK_PROGRESS_CHECK_MS`
/// (upstream `QWEN_AUDIO_AGENT_BACKGROUND_TASK_PROGRESS_CHECK_MS`, with
/// `…_SCHEDULED_TASK_PROGRESS_CHECK_MS` as its legacy fallback), default
/// `300_000`, minimum `30_000` — `server/src/core/config.mjs:463-491`.
pub const DEFAULT_PROGRESS_CHECK_MS: i64 = 300_000;

/// The floor the configuration clamps the cadence to.
///
/// Contract — the same entry: `min 30_000`.
pub const MIN_PROGRESS_CHECK_MS: i64 = 30_000;

/// How much of the objective the announcement quotes.
///
/// Contract — `server/src/task/task-manager.mjs:605,609`
/// (`task.objective.slice(0, 80)`).
pub const OBJECTIVE_QUOTE_BOUND: usize = 80;

/// The five categories that have a verb of their own, in upstream's declaration
/// order, with the catalog key each maps to.
///
/// Contract — `server/src/task/task-manager.mjs:596-602`. The sixth entry is
/// the `|| '处理'` fallback and has no category: an activity whose category is
/// absent, empty or unrecognised uses it.
pub const ACTIVITY_VERBS: &[(&str, Key)] = &[
    ("run", keys::WORK_ACTIVITY_VERB_RUN),
    ("read", keys::WORK_ACTIVITY_VERB_READ),
    ("write", keys::WORK_ACTIVITY_VERB_WRITE),
    ("search", keys::WORK_ACTIVITY_VERB_SEARCH),
    ("image", keys::WORK_ACTIVITY_VERB_IMAGE),
];

/// The verb for `category`, or the fallback.
///
/// `{run:…, read:…, write:…, search:…, image:…}[category] || '处理'`.
#[must_use]
pub fn activity_verb(locale: Locale, category: Option<&str>) -> &'static str {
    let key = category
        .and_then(|category| {
            ACTIVITY_VERBS
                .iter()
                .find(|(name, _)| *name == category)
                .map(|(_, key)| *key)
        })
        .unwrap_or(keys::WORK_ACTIVITY_VERB_OTHER);
    via_i18n::t(locale, key)
}

/// `Math.round((now - (startedAt || now)) / 60000)`.
///
/// `server/src/task/task-manager.mjs:591-593`. Note the JavaScript `||`: a
/// Work with no `startedAt` — or one of exactly `0` — reports zero minutes
/// rather than fifty-six years.
#[must_use]
pub fn elapsed_minutes(started_at: Option<i64>, now: i64) -> i64 {
    let started_at = match started_at {
        Some(started_at) if started_at != 0 => started_at,
        _ => now,
    };
    #[expect(
        clippy::cast_precision_loss,
        reason = "epoch milliseconds are exact in f64 for the next 285,000 years"
    )]
    let minutes = (now - started_at) as f64 / 60_000.0;
    // JavaScript `Math.round` and Rust `f64::round` agree for every positive
    // value, including the .5 case both round away from zero.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the quotient of two epoch-millisecond readings fits i64"
    )]
    let rounded = minutes.round() as i64;
    rounded
}

/// Compose the announcement.
///
/// The two templates are `work.progress_with_activity` and
/// `work.progress_without_activity`; which one is used depends only on whether
/// the activity ring has a newest entry.
///
/// `detail` carries its own leading space, because upstream writes
/// `` detail = lastActivity.detail ? ` ${lastActivity.detail}` : '' `` and
/// interpolates `正在${verb}${detail}` with no separator of its own.
#[must_use]
pub fn progress_message(
    locale: Locale,
    objective: &str,
    started_at: Option<i64>,
    now: i64,
    newest: Option<&Activity>,
) -> String {
    let quoted = slice_units(objective, OBJECTIVE_QUOTE_BOUND);
    let minutes = elapsed_minutes(started_at, now).to_string();
    match newest {
        None => i18n_format(
            locale,
            keys::WORK_PROGRESS_WITHOUT_ACTIVITY,
            &[("objective", &quoted), ("minutes", &minutes)],
        ),
        Some(activity) => {
            let verb = activity_verb(locale, activity.category.as_deref());
            let detail = match activity.detail.as_deref() {
                Some(detail) if !detail.is_empty() => format!(" {detail}"),
                _ => String::new(),
            };
            i18n_format(
                locale,
                keys::WORK_PROGRESS_WITH_ACTIVITY,
                &[
                    ("objective", &quoted),
                    ("minutes", &minutes),
                    ("verb", verb),
                    ("detail", &detail),
                    ("status", &activity.status),
                ],
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ACTIVITY_VERBS, DEFAULT_PROGRESS_CHECK_MS, MIN_PROGRESS_CHECK_MS, OBJECTIVE_QUOTE_BOUND,
        activity_verb, elapsed_minutes, progress_message,
    };
    use crate::activity::Activity;
    use via_i18n::Locale;

    fn activity(category: Option<&str>, detail: Option<&str>) -> Activity {
        Activity {
            id: Some("call".to_owned()),
            kind: "tool".to_owned(),
            tool: Some("bash".to_owned()),
            label: Some("bash".to_owned()),
            status: "running".to_owned(),
            category: category.map(str::to_owned),
            detail: detail.map(str::to_owned),
            completed: None,
            total: None,
        }
    }

    #[test]
    fn the_cadence_defaults_are_the_catalogued_ones() {
        assert_eq!(DEFAULT_PROGRESS_CHECK_MS, 300_000);
        assert_eq!(MIN_PROGRESS_CHECK_MS, 30_000);
    }

    #[test]
    fn the_chinese_message_with_activity_is_the_catalogued_sentence() {
        let message = progress_message(
            Locale::Zh,
            "测试进度任务",
            Some(1_000),
            1_000 + 3 * 60_000,
            Some(&activity(Some("run"), Some("npm test"))),
        );
        assert_eq!(
            message,
            "任务“测试进度任务”已运行 3 分钟，正在执行 npm test（running）"
        );
    }

    #[test]
    fn the_chinese_message_without_activity_is_the_catalogued_sentence() {
        let message = progress_message(Locale::Zh, "无活动任务", Some(1_000), 1_000 + 60_000, None);
        assert_eq!(message, "任务“无活动任务”已运行 1 分钟，正在处理中");
    }

    #[test]
    fn the_verb_table_is_upstreams_five_plus_a_fallback() {
        let names: Vec<&str> = ACTIVITY_VERBS.iter().map(|(name, _)| *name).collect();
        assert_eq!(names, ["run", "read", "write", "search", "image"]);
        for (category, verb) in [
            ("run", "执行"),
            ("read", "读取"),
            ("write", "修改"),
            ("search", "搜索"),
            ("image", "生成图片"),
        ] {
            assert_eq!(activity_verb(Locale::Zh, Some(category)), verb);
        }
        for unknown in [None, Some(""), Some("unknown"), Some("RUN")] {
            assert_eq!(activity_verb(Locale::Zh, unknown), "处理", "{unknown:?}");
        }
    }

    #[test]
    fn a_missing_detail_leaves_no_stray_space() {
        let message = progress_message(
            Locale::Zh,
            "任务",
            Some(1_000),
            1_000,
            Some(&activity(Some("read"), None)),
        );
        assert_eq!(message, "任务“任务”已运行 0 分钟，正在读取（running）");
        let empty = progress_message(
            Locale::Zh,
            "任务",
            Some(1_000),
            1_000,
            Some(&activity(Some("read"), Some(""))),
        );
        assert_eq!(empty, message);
    }

    #[test]
    fn the_activity_status_is_interpolated_verbatim() {
        let mut entry = activity(Some("run"), None);
        entry.status = "completed".to_owned();
        let message = progress_message(Locale::Zh, "t", Some(1_000), 1_000, Some(&entry));
        assert!(message.ends_with("（completed）"), "{message}");
    }

    #[test]
    fn the_objective_is_quoted_to_eighty_units() {
        let long = "x".repeat(OBJECTIVE_QUOTE_BOUND + 40);
        let message = progress_message(Locale::Zh, &long, Some(1_000), 1_000, None);
        assert!(message.contains(&"x".repeat(OBJECTIVE_QUOTE_BOUND)));
        assert!(!message.contains(&"x".repeat(OBJECTIVE_QUOTE_BOUND + 1)));
    }

    #[test]
    fn elapsed_minutes_rounds_and_survives_a_missing_start() {
        const START: i64 = 1_000;
        assert_eq!(elapsed_minutes(Some(START), START), 0);
        assert_eq!(elapsed_minutes(Some(START), START + 29_999), 0);
        assert_eq!(
            elapsed_minutes(Some(START), START + 30_000),
            1,
            "half rounds away from zero, exactly as Math.round does",
        );
        assert_eq!(elapsed_minutes(Some(START), START + 89_999), 1);
        assert_eq!(elapsed_minutes(Some(START), START + 90_000), 2);
        assert_eq!(
            elapsed_minutes(None, 1_700_000_000_000),
            0,
            "a Work with no start reports zero, not fifty-six years",
        );
        assert_eq!(
            elapsed_minutes(Some(0), 1_700_000_000_000),
            0,
            "`task.startedAt || Date.now()` — a startedAt of 0 is falsy upstream too",
        );
        assert_eq!(
            elapsed_minutes(Some(START), START + 1_699_999_999_000),
            28_333_333
        );
    }

    #[test]
    fn every_locale_renders_both_templates_completely() {
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            for newest in [None, Some(&activity(Some("write"), Some("src/lib.rs")))] {
                let message = progress_message(locale, "objective", Some(1), 120_001, newest);
                assert!(!message.contains("<via-i18n:"), "{locale}: {message}");
                assert!(!message.contains('{'), "{locale}: {message}");
            }
        }
    }
}
