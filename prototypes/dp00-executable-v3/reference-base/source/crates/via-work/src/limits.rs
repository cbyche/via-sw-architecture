//! Every retention, cadence and cap the Work subsystem observes, with the
//! clamps the configuration applies.
//!
//! Contract — `docs/reference/contracts.json`, three entries:
//!
//! - `env-var` / *Work-subsystem environment variables*
//! - `env-var` / *task and session retention*
//! - `default-value` / *TaskManager / TaskScheduler / TaskStore defaults*
//!
//! all from `server/src/core/config.mjs:386-421,463-491` and
//! `server/src/task/task-manager.mjs:105-112`. The values are duplicated in
//! `via_core::Config`, which is where they are *read from the environment*;
//! they are named again here because this is the crate that acts on them, and
//! because a manager built without a `Config` — every test in this crate — must
//! still start on the shipped numbers.
//!
//! [`RetentionPolicy::clamped`] applies the same floors and ceilings
//! `numberSetting` does, so a caller that hands the manager an unclamped value
//! gets the same behaviour a caller that went through `Config` would.

use serde::{Deserialize, Serialize};

/// How long a delivered or unremarkable terminal Work is kept: 24 hours.
pub const DEFAULT_TERMINAL_TTL_MS: i64 = 86_400_000;
/// The floor `VIA_TASK_TERMINAL_TTL_MS` is clamped to: 60 seconds.
pub const MIN_TERMINAL_TTL_MS: i64 = 60_000;

/// How long a terminal Work whose notification is still owed is kept: 7 days.
///
/// Deliberately far longer than [`DEFAULT_TERMINAL_TTL_MS`]: a result nobody
/// has heard yet is the one thing retention must not throw away.
pub const DEFAULT_PENDING_NOTIFICATION_TTL_MS: i64 = 604_800_000;
/// The floor `VIA_TASK_NOTIFICATION_TTL_MS` is clamped to: 60 seconds.
pub const MIN_PENDING_NOTIFICATION_TTL_MS: i64 = 60_000;

/// How long one client may hold a delivery lease before it is reclaimed:
/// 60 seconds.
pub const DEFAULT_NOTIFICATION_CLAIM_TTL_MS: i64 = 60_000;
/// The floor `VIA_TASK_NOTIFICATION_CLAIM_TTL_MS` is clamped to: 5 seconds.
pub const MIN_NOTIFICATION_CLAIM_TTL_MS: i64 = 5_000;

/// How many *delivered* terminal Work items one owner keeps: 100.
pub const DEFAULT_MAX_TERMINAL_TASKS_PER_OWNER: i64 = 100;
/// The floor `VIA_MAX_TERMINAL_TASKS_PER_OWNER` is clamped to: 10.
pub const MIN_MAX_TERMINAL_TASKS_PER_OWNER: i64 = 10;

/// The liveness tick while a Work is active: one second.
///
/// Contract — `server/src/task/task-manager.mjs:489`
/// (*progress heartbeat interval=1000 ms*). Emits `task.progress` and does
/// **not** persist.
pub const PROGRESS_HEARTBEAT_MS: i64 = 1_000;

/// How long a `scheduled_task` gets after its abort before it is force-failed:
/// 5 seconds.
///
/// Contract — `server/src/task/task-manager.mjs:576`
/// (*scheduled-task cleanup window after abort=5000 ms*). The abort is asked
/// for first; this is how long a well-behaved runner has to honour it before
/// the manager stops waiting.
pub const SCHEDULED_TASK_CLEANUP_MS: i64 = 5_000;

/// The wall-clock budget a `scheduled_task` runs under: 30 minutes.
pub const DEFAULT_SCHEDULED_TASK_TIMEOUT_MS: i64 = 1_800_000;
/// The floor `VIA_SCHEDULED_TASK_TIMEOUT_MS` is clamped to: 60 seconds.
pub const MIN_SCHEDULED_TASK_TIMEOUT_MS: i64 = 60_000;

/// The spread applied to overdue reminders on restart: 30 seconds apart.
pub const DEFAULT_REMINDER_STAGGER_MS: i64 = 30_000;
/// The floor `VIA_REMINDER_STAGGER_MS` is clamped to: 0.
pub const MIN_REMINDER_STAGGER_MS: i64 = 0;
/// The ceiling `VIA_REMINDER_STAGGER_MS` is clamped to: 5 minutes.
pub const MAX_REMINDER_STAGGER_MS: i64 = 300_000;

/// How many reminders one owner may hold: 50.
pub const DEFAULT_REMINDER_MAX_PER_OWNER: i64 = 50;
/// The floor `VIA_REMINDER_MAX_PER_OWNER` is clamped to: 1.
pub const MIN_REMINDER_MAX_PER_OWNER: i64 = 1;
/// The ceiling `VIA_REMINDER_MAX_PER_OWNER` is clamped to: 500.
pub const MAX_REMINDER_MAX_PER_OWNER: i64 = 500;

/// `numberSetting`'s clamp: a value below `min` becomes `min`, above `max`
/// becomes `max`.
#[must_use]
pub const fn clamp(value: i64, min: i64, max: i64) -> i64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// How long Work is kept, and how many of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicy {
    /// TTL for a terminal Work with nothing owed.
    pub terminal_ttl_ms: i64,
    /// TTL for a terminal Work whose notification is still `pending` or
    /// `delivering`.
    pub pending_notification_ttl_ms: i64,
    /// How long a delivery lease survives without a renewal.
    pub notification_claim_ttl_ms: i64,
    /// The per-owner cap on *delivered* terminal Work.
    pub max_terminal_tasks_per_owner: i64,
}

impl Default for RetentionPolicy {
    /// The four shipped defaults.
    fn default() -> Self {
        Self {
            terminal_ttl_ms: DEFAULT_TERMINAL_TTL_MS,
            pending_notification_ttl_ms: DEFAULT_PENDING_NOTIFICATION_TTL_MS,
            notification_claim_ttl_ms: DEFAULT_NOTIFICATION_CLAIM_TTL_MS,
            max_terminal_tasks_per_owner: DEFAULT_MAX_TERMINAL_TASKS_PER_OWNER,
        }
    }
}

impl RetentionPolicy {
    /// This policy with every floor applied.
    ///
    /// The same clamps `numberSetting` applies when the values come from the
    /// environment, so a hand-built policy and a configured one cannot diverge.
    /// None of the four has a ceiling upstream.
    #[must_use]
    pub const fn clamped(self) -> Self {
        Self {
            terminal_ttl_ms: clamp(self.terminal_ttl_ms, MIN_TERMINAL_TTL_MS, i64::MAX),
            pending_notification_ttl_ms: clamp(
                self.pending_notification_ttl_ms,
                MIN_PENDING_NOTIFICATION_TTL_MS,
                i64::MAX,
            ),
            notification_claim_ttl_ms: clamp(
                self.notification_claim_ttl_ms,
                MIN_NOTIFICATION_CLAIM_TTL_MS,
                i64::MAX,
            ),
            max_terminal_tasks_per_owner: clamp(
                self.max_terminal_tasks_per_owner,
                MIN_MAX_TERMINAL_TASKS_PER_OWNER,
                i64::MAX,
            ),
        }
    }

    /// Which TTL a terminal Work is retained under.
    ///
    /// `awaitingDelivery ? pendingNotificationTtlMs : terminalTtlMs`
    /// (`server/src/task/task-manager.mjs:948-953`).
    #[must_use]
    pub const fn ttl_for(&self, awaiting_delivery: bool) -> i64 {
        if awaiting_delivery {
            self.pending_notification_ttl_ms
        } else {
            self.terminal_ttl_ms
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_MAX_TERMINAL_TASKS_PER_OWNER, DEFAULT_PENDING_NOTIFICATION_TTL_MS,
        DEFAULT_TERMINAL_TTL_MS, MIN_MAX_TERMINAL_TASKS_PER_OWNER, MIN_NOTIFICATION_CLAIM_TTL_MS,
        MIN_PENDING_NOTIFICATION_TTL_MS, MIN_TERMINAL_TTL_MS, RetentionPolicy, clamp,
    };

    #[test]
    fn the_defaults_are_twenty_four_hours_and_seven_days() {
        let policy = RetentionPolicy::default();
        assert_eq!(policy.terminal_ttl_ms, 24 * 60 * 60 * 1_000);
        assert_eq!(policy.pending_notification_ttl_ms, 7 * 24 * 60 * 60 * 1_000);
        assert_eq!(policy.notification_claim_ttl_ms, 60_000);
        assert_eq!(policy.max_terminal_tasks_per_owner, 100);
        assert_eq!(
            policy,
            policy.clamped(),
            "the defaults are already inside the clamps"
        );
    }

    #[test]
    fn an_undeliverable_result_is_kept_far_longer_than_a_delivered_one() {
        let policy = RetentionPolicy::default();
        assert!(policy.ttl_for(true) > policy.ttl_for(false));
        assert_eq!(policy.ttl_for(false), DEFAULT_TERMINAL_TTL_MS);
        assert_eq!(policy.ttl_for(true), DEFAULT_PENDING_NOTIFICATION_TTL_MS);
    }

    #[test]
    fn every_floor_is_applied() {
        let clamped = RetentionPolicy {
            terminal_ttl_ms: 1,
            pending_notification_ttl_ms: 1,
            notification_claim_ttl_ms: 1,
            max_terminal_tasks_per_owner: 1,
        }
        .clamped();
        assert_eq!(clamped.terminal_ttl_ms, MIN_TERMINAL_TTL_MS);
        assert_eq!(
            clamped.pending_notification_ttl_ms,
            MIN_PENDING_NOTIFICATION_TTL_MS
        );
        assert_eq!(
            clamped.notification_claim_ttl_ms,
            MIN_NOTIFICATION_CLAIM_TTL_MS
        );
        assert_eq!(
            clamped.max_terminal_tasks_per_owner,
            MIN_MAX_TERMINAL_TASKS_PER_OWNER
        );
    }

    #[test]
    fn a_value_above_a_floor_is_untouched() {
        let policy = RetentionPolicy {
            terminal_ttl_ms: DEFAULT_TERMINAL_TTL_MS * 2,
            max_terminal_tasks_per_owner: DEFAULT_MAX_TERMINAL_TASKS_PER_OWNER * 2,
            ..RetentionPolicy::default()
        };
        assert_eq!(policy.clamped(), policy);
    }

    #[test]
    fn clamp_is_inclusive_at_both_ends() {
        assert_eq!(clamp(5, 5, 10), 5);
        assert_eq!(clamp(10, 5, 10), 10);
        assert_eq!(clamp(4, 5, 10), 5);
        assert_eq!(clamp(11, 5, 10), 10);
    }
}
