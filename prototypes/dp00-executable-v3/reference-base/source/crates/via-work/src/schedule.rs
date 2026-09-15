//! When a `reminder` or a `scheduled_task` is due.
//!
//! `server/src/task/task-manager.mjs:425`, catalogued as *schedule object*:
//!
//! ```json
//! { "type": "at", "at": 1755820800000, "recurrence": "once" }
//! ```
//!
//! Three fields, no more. The reminder scheduler reads `at` directly; nothing
//! reads `type`, which exists so a future `cron` variant can be told apart from
//! a one-shot without a schema migration.

use serde::{Deserialize, Serialize};

/// The `type` a one-shot schedule carries.
///
/// Contract — `server/src/task/task-manager.mjs:425`. It is the only value
/// upstream ever writes.
pub const SCHEDULE_TYPE_AT: &str = "at";

/// The `recurrence` a schedule defaults to.
///
/// Contract — `server/src/task/task-manager.mjs:409`
/// (`{ at, recurrence = 'once' } = {}`).
pub const RECURRENCE_ONCE: &str = "once";

/// A due time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schedule {
    /// Always [`SCHEDULE_TYPE_AT`] today.
    #[serde(rename = "type")]
    pub kind: String,
    /// Epoch milliseconds. The reminder scheduler's only input.
    pub at: i64,
    /// `once`, or whatever a caller passed. **Upstream does not act on any
    /// other value** — `reminder-scheduler.mjs:104` marks recurrence handling
    /// as a later phase — so a recurring schedule fires exactly once and is
    /// then terminal.
    pub recurrence: String,
}

impl Schedule {
    /// A one-shot schedule at `at`.
    #[must_use]
    pub fn at(at: i64) -> Self {
        Self {
            kind: SCHEDULE_TYPE_AT.to_owned(),
            at,
            recurrence: RECURRENCE_ONCE.to_owned(),
        }
    }

    /// A schedule at `at` with an explicit recurrence.
    ///
    /// An empty recurrence becomes [`RECURRENCE_ONCE`], reproducing JavaScript's
    /// `recurrence = 'once'` default parameter, which fires for `undefined`
    /// only — but upstream's caller passes `args.recurrence || 'once'`
    /// (`tool-call-handler.mjs:302`), so an empty string never reaches it.
    #[must_use]
    pub fn at_with_recurrence(at: i64, recurrence: &str) -> Self {
        Self {
            kind: SCHEDULE_TYPE_AT.to_owned(),
            at,
            recurrence: if recurrence.is_empty() {
                RECURRENCE_ONCE.to_owned()
            } else {
                recurrence.to_owned()
            },
        }
    }

    /// Whether this schedule is due at `now`.
    ///
    /// `t.schedule?.at <= now` (`reminder-scheduler.mjs:53,102`) — inclusive,
    /// so a schedule for exactly now fires now.
    #[must_use]
    pub const fn is_due_at(&self, now: i64) -> bool {
        self.at <= now
    }
}

#[cfg(test)]
mod tests {
    use super::{RECURRENCE_ONCE, SCHEDULE_TYPE_AT, Schedule};
    use serde_json::json;

    #[test]
    fn the_schedule_serializes_in_the_catalogued_shape() {
        let rendered = serde_json::to_value(Schedule::at(1_755_820_800_000)).expect("serializes");
        assert_eq!(
            rendered,
            json!({"type": "at", "at": 1_755_820_800_000_i64, "recurrence": "once"})
        );
        assert_eq!(
            rendered.as_object().expect("object").keys().next(),
            Some(&"type".to_owned())
        );
    }

    #[test]
    fn due_is_inclusive() {
        let schedule = Schedule::at(1_000);
        assert!(!schedule.is_due_at(999));
        assert!(schedule.is_due_at(1_000));
        assert!(schedule.is_due_at(1_001));
    }

    #[test]
    fn an_empty_recurrence_becomes_once() {
        assert_eq!(
            Schedule::at_with_recurrence(1, "").recurrence,
            RECURRENCE_ONCE
        );
        assert_eq!(Schedule::at_with_recurrence(1, "daily").recurrence, "daily");
        assert_eq!(Schedule::at(1).kind, SCHEDULE_TYPE_AT);
    }
}
