//! The reminder scheduler: **one** timer, re-armed, never a poll.
//!
//! `server/src/task/reminder-scheduler.mjs`, whose own header states the rule:
//!
//! > Does not poll. Uses a single `setTimeout` for the next due task, re-arming
//! > after each fire. On restart, overdue tasks are staggered (not replayed
//! > simultaneously) to avoid swamping the backend agent.
//!
//! Here that is one [`tokio::time::Sleep`] inside one task, recomputed on every
//! mutation the manager announces — a Work scheduled, a Work cancelled, a Work
//! fired. Between those, the task is parked; there is no interval anywhere in
//! this file.
//!
//! # The one structural change
//!
//! Upstream arms **N extra timers** for the overdue backlog
//! (`reminder-scheduler.mjs:62-72`) and its single timer deliberately ignores
//! anything already due (`:86`, `schedule.at > now`), so the two mechanisms do
//! not collide. Here the stagger is expressed as a *deadline override* — an
//! overdue Work's effective due time becomes `now + index × stagger` — and the
//! one sleep serves both. Same behaviour, same order, same spacing; one timer
//! instead of N+1. Recorded in `docs/deviations/phase-3.md`.
//!
//! # Why staggering exists at all
//!
//! A Gateway that was down for an hour comes back with every reminder in that
//! hour overdue. Firing them together would put a dozen coordinator turns into
//! the queue at once, and the per-owner cap is two — so eleven of them would sit
//! `queued` while the user hears nothing. Thirty seconds apart is upstream's
//! answer.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::sync::broadcast::error::RecvError;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::clock::NowFn;
use crate::event::WorkEventKind;
use crate::limits::{
    DEFAULT_REMINDER_STAGGER_MS, MAX_REMINDER_STAGGER_MS, MIN_REMINDER_STAGGER_MS, clamp,
};
use crate::manager::WorkManager;

/// A running reminder scheduler.
///
/// Dropping it does **not** stop the timer — call [`ReminderScheduler::close`],
/// or drop the [`WorkManager`], which closes the event stream the loop reads.
#[derive(Debug)]
pub struct ReminderScheduler {
    shutdown: CancellationToken,
    rearms: Arc<AtomicUsize>,
}

impl ReminderScheduler {
    /// Start the scheduler.
    ///
    /// The overdue backlog is staggered immediately; everything else waits for
    /// its own due time. `stagger_ms` is clamped to
    /// `[0, 300_000]`, which is `VIA_REMINDER_STAGGER_MS`'s range.
    ///
    /// # Panics
    ///
    /// Needs a tokio runtime.
    #[must_use]
    pub fn start(manager: &WorkManager, now: NowFn, stagger_ms: i64) -> Self {
        Self::start_on(manager, now, stagger_ms, manager.tracker().clone())
    }

    /// Start the scheduler on a specific tracker.
    ///
    /// # Panics
    ///
    /// Needs a tokio runtime.
    #[must_use]
    pub fn start_on(
        manager: &WorkManager,
        now: NowFn,
        stagger_ms: i64,
        tracker: TaskTracker,
    ) -> Self {
        let shutdown = CancellationToken::new();
        let stagger = clamp(stagger_ms, MIN_REMINDER_STAGGER_MS, MAX_REMINDER_STAGGER_MS);
        let loop_shutdown = shutdown.clone();
        let rearms = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&rearms);
        let manager = manager.clone();
        tracker.spawn(async move { run(manager, now, stagger, loop_shutdown, counter).await });
        Self { shutdown, rearms }
    }

    /// Stop the scheduler. Idempotent.
    pub fn close(&self) {
        self.shutdown.cancel();
    }

    /// Whether [`Self::close`] has been called.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.shutdown.is_cancelled()
    }

    /// How many times the timer has been recomputed since it started.
    ///
    /// The observable form of *"never polls"*: this rises only when the
    /// manager announces something that could have changed what is due, or
    /// when the timer itself fires. A scheduler that woke on a fixed interval
    /// would climb with the clock instead, which is exactly what
    /// `tests/reminder.rs` asserts it does not do.
    #[must_use]
    pub fn rearms(&self) -> usize {
        self.rearms.load(Ordering::Relaxed)
    }
}

/// The events that change what is due, and therefore re-arm the timer.
///
/// `reminder-scheduler.mjs:26-30` subscribes to `task.scheduled` and
/// `task.cancelled`; `task.scheduled.fired` is added here because it is the
/// event the fire itself raises, and the timer must not re-arm to a Work that
/// has just left `scheduled`.
const REARMING_EVENTS: &[WorkEventKind] = &[
    WorkEventKind::Scheduled,
    WorkEventKind::ScheduledFired,
    WorkEventKind::Cancelled,
];

async fn run(
    manager: WorkManager,
    now: NowFn,
    stagger_ms: i64,
    shutdown: CancellationToken,
    rearms: Arc<AtomicUsize>,
) {
    let mut events = manager.subscribe();
    // The overdue backlog's deadline overrides: `work_id -> effective due time`.
    // Computed once, at start, exactly as `restoreOverdue` does.
    let mut overrides: Vec<(String, i64)> = {
        let start = now();
        let mut overdue: Vec<_> = manager
            .scheduled()
            .await
            .into_iter()
            .filter(|work| work.at <= start)
            .collect();
        overdue.sort_by_key(|work| work.at);
        overdue
            .into_iter()
            .enumerate()
            .map(|(index, work)| {
                let offset = i64::try_from(index).unwrap_or(i64::MAX);
                (
                    work.work_id,
                    start.saturating_add(offset.saturating_mul(stagger_ms)),
                )
            })
            .collect()
    };

    // `deadline` is the one timer. It is recomputed only when something the
    // manager announced could have changed what is due — never on a tick, and
    // never on a schedule of its own.
    let mut deadline: Option<tokio::time::Instant> = None;
    let mut stale = true;
    loop {
        if shutdown.is_cancelled() {
            return;
        }
        if stale {
            rearms.fetch_add(1, Ordering::Relaxed);
            let scheduled = manager.scheduled().await;
            // An override only survives while its Work is still scheduled.
            overrides.retain(|(work_id, _)| scheduled.iter().any(|work| work.work_id == *work_id));

            let moment = now();
            let mut due = Vec::new();
            let mut next: Option<i64> = None;
            for work in &scheduled {
                let at = overrides
                    .iter()
                    .find(|(work_id, _)| *work_id == work.work_id)
                    .map_or(work.at, |(_, at)| *at);
                if at <= moment {
                    due.push(work.work_id.clone());
                } else {
                    next = Some(next.map_or(at, |current: i64| current.min(at)));
                }
            }

            if !due.is_empty() {
                manager.fire_scheduled(&due).await;
                overrides.retain(|(work_id, _)| !due.contains(work_id));
                // Straight back round: firing may have made the next one due
                // too, and the state has just changed under us.
                continue;
            }
            deadline = next.map(|at| {
                tokio::time::Instant::now() + Duration::from_millis((at - moment).unsigned_abs())
            });
        }

        // Every arm answers whether the next pass has to recompute, so
        // `stale` is assigned here rather than cleared above: a new arm that
        // forgot to answer would not compile.
        stale = tokio::select! {
            () = shutdown.cancelled() => return,
            () = maybe_sleep_until(deadline) => true,
            received = events.recv() => match received {
                Ok(event) => REARMING_EVENTS.contains(&event.kind),
                // A lagged subscriber has missed a mutation, so the safe answer
                // is to recompute rather than to keep sleeping on a deadline
                // that may be stale.
                Err(RecvError::Lagged(_)) => true,
                Err(RecvError::Closed) => return,
            },
        };
    }
}

/// Sleep until `deadline`, or wait forever when nothing is due.
async fn maybe_sleep_until(deadline: Option<tokio::time::Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline).await,
        None => std::future::pending().await,
    }
}

/// The shipped stagger, re-exported so a caller can spell the default.
pub const DEFAULT_STAGGER_MS: i64 = DEFAULT_REMINDER_STAGGER_MS;
