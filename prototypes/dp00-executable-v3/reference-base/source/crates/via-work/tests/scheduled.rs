//! Scheduled Work, ported from `server/test/task-manager-scheduled.test.mjs`
//! plus the wall-clock watchdog `server/src/task/task-manager.mjs:549-580`
//! arms and nothing upstream covers.

mod common;

use std::sync::Arc;

use common::{Events, advance, clock, settle};
use pretty_assertions::assert_eq;
use via_i18n::{Locale, keys, t};
use via_protocol::{WorkKind, WorkStatus};
use via_work::limits::{DEFAULT_SCHEDULED_TASK_TIMEOUT_MS, SCHEDULED_TASK_CLEANUP_MS};
use via_work::testing::{GatedRunner, ImmediateRunner};
use via_work::{
    NewScheduledWork, NewWork, NotificationStatus, Schedule, WorkEventKind, WorkManager, WorkQuery,
};

const OWNER: &str = "owner";
const SESSION: &str = "voice";
const FUTURE: i64 = common::BASE_MS + 60_000;

// ── creation ───────────────────────────────────────────────────────────────

/// Upstream: *createScheduled creates a task with status scheduled and correct
/// kind*, and *createScheduled emits task.scheduled event*.
#[tokio::test(start_paused = true)]
async fn a_reminder_is_scheduled_with_no_timeout_and_no_progress_check() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());

    let accepted = manager
        .create_scheduled(
            NewScheduledWork::reminder("提醒我开会", OWNER, Schedule::at(FUTURE))
                .session(SESSION)
                .turn("turn-1"),
        )
        .await
        .expect("accepted");

    assert_eq!(accepted.work.status, WorkStatus::Scheduled);
    assert_eq!(accepted.work.kind, WorkKind::Reminder);
    assert_eq!(accepted.work.work_state, via_protocol::WorkState::Scheduled);
    assert_eq!(
        accepted.work.schedule,
        Some(Schedule::at(FUTURE)),
        "the catalogued schedule object",
    );
    assert_eq!(accepted.work.timeout_ms, None);
    assert_eq!(accepted.work.progress_check_ms, None);
    assert!(!accepted.reused);
    assert!(events.saw(WorkEventKind::Scheduled));
    assert!(
        !events.saw(WorkEventKind::Running),
        "createScheduled does not drain",
    );
}

/// Upstream: *createScheduled with type=task sets timeout without progress
/// checks*.
#[tokio::test(start_paused = true)]
async fn a_scheduled_task_gets_a_timeout_and_still_no_progress_check() {
    let manager = WorkManager::builder().now(clock()).build();
    let accepted = manager
        .create_scheduled(NewScheduledWork::task(
            "查构建状态",
            OWNER,
            Schedule::at(FUTURE),
        ))
        .await
        .expect("accepted");
    assert_eq!(accepted.work.kind, WorkKind::ScheduledTask);
    assert_eq!(
        accepted.work.timeout_ms,
        Some(DEFAULT_SCHEDULED_TASK_TIMEOUT_MS),
    );
    assert_eq!(accepted.work.progress_check_ms, None);

    let overridden = manager
        .create_scheduled(
            NewScheduledWork::task("另一个", OWNER, Schedule::at(FUTURE)).timeout_ms(90_000),
        )
        .await
        .expect("accepted");
    assert_eq!(overridden.work.timeout_ms, Some(90_000));
}

/// Upstream: *createScheduled does not call drain (scheduled tasks wait)*, and
/// *scheduled task appears in list output*.
#[tokio::test(start_paused = true)]
async fn a_scheduled_work_waits_and_is_listed() {
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(ImmediateRunner::completing("done")))
        .build();
    manager
        .create_scheduled(NewScheduledWork::reminder(
            "不立即执行",
            OWNER,
            Schedule::at(FUTURE),
        ))
        .await
        .expect("accepted");
    advance(5_000).await;

    let listed = manager.list(WorkQuery::owner(OWNER)).await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].status, WorkStatus::Scheduled);
    assert_eq!(listed[0].kind, WorkKind::Reminder);
    assert!(
        !manager
            .list(WorkQuery::owner(OWNER).active())
            .await
            .iter()
            .any(|_| true),
        "`scheduled` is not one of the five active statuses",
    );
}

// ── cancellation ───────────────────────────────────────────────────────────

/// Upstream: *scheduled task is cancellable (CANCELLABLE includes scheduled)*.
#[tokio::test(start_paused = true)]
async fn a_scheduled_work_cancels_without_passing_through_cancelling() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let accepted = manager
        .create_scheduled(NewScheduledWork::reminder(
            "取消我",
            OWNER,
            Schedule::at(FUTURE),
        ))
        .await
        .expect("accepted");

    let cancelled = manager
        .cancel(&accepted.work.id, Some(OWNER))
        .await
        .expect("cancellable");
    assert_eq!(cancelled.status, WorkStatus::Cancelled);
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Cancelled,
    );
    assert!(
        !events.saw(WorkEventKind::Cancelling),
        "nothing was running, so there was nothing to confirm",
    );
    assert!(events.saw(WorkEventKind::Cancelled));

    // Firing it afterwards does nothing.
    assert_eq!(
        manager
            .fire_scheduled(std::slice::from_ref(&accepted.work.id))
            .await,
        0
    );
}

// ── firing ─────────────────────────────────────────────────────────────────

/// A fired reminder speaks its stored text back, with a presentation.
#[tokio::test(start_paused = true)]
async fn a_fired_reminder_speaks_its_own_text() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let accepted = manager
        .create_scheduled(NewScheduledWork::reminder(
            "该开会了",
            OWNER,
            Schedule::at(FUTURE),
        ))
        .await
        .expect("accepted");

    assert_eq!(
        manager
            .fire_scheduled(std::slice::from_ref(&accepted.work.id))
            .await,
        1
    );
    let fired = manager.wait(&accepted.work.id).await.expect("terminal");
    settle().await;

    assert_eq!(fired.status, WorkStatus::Completed);
    assert_eq!(fired.result.as_deref(), Some("该开会了"));
    assert_eq!(
        fired
            .result_metadata
            .expect("a presentation")
            .presentation
            .speech,
        "该开会了",
    );
    assert_eq!(fired.notification_status, NotificationStatus::Pending);
    assert_eq!(
        events.kinds_for(&accepted.work.id),
        vec![
            WorkEventKind::Scheduled,
            WorkEventKind::ScheduledFired,
            WorkEventKind::Running,
            WorkEventKind::Completed,
            WorkEventKind::NotificationPending,
        ],
    );
}

/// Firing something that is not `scheduled` is a no-op, so a timer that races a
/// cancellation cannot resurrect it.
#[tokio::test(start_paused = true)]
async fn firing_a_work_that_is_not_scheduled_changes_nothing() {
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(ImmediateRunner::completing("done")))
        .build();
    let ordinary = manager
        .create(NewWork::new("ordinary", OWNER))
        .await
        .expect("accepted");
    manager.wait(&ordinary.work.id).await;

    assert_eq!(
        manager
            .fire_scheduled(std::slice::from_ref(&ordinary.work.id))
            .await,
        0
    );
    assert_eq!(
        manager
            .fire_scheduled(&["work_nonexistent".to_owned()])
            .await,
        0
    );
    assert_eq!(
        manager
            .get(&ordinary.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Completed,
    );
}

// ── the wall-clock watchdog ────────────────────────────────────────────────

/// The abort comes first, and the runner gets the catalogued reason.
#[tokio::test(start_paused = true)]
async fn the_watchdog_aborts_before_it_force_fails() {
    let manager = WorkManager::builder()
        .now(clock())
        .locale(Locale::Zh)
        .build();
    let runner = GatedRunner::stubborn();
    let accepted = manager
        .create_scheduled(
            NewScheduledWork::task("长时间任务", OWNER, Schedule::at(FUTURE))
                .timeout_ms(60_000)
                .runner(Arc::new(runner.clone())),
        )
        .await
        .expect("accepted");
    manager
        .fire_scheduled(std::slice::from_ref(&accepted.work.id))
        .await;
    settle().await;
    assert!(runner.is_running());

    advance(60_000).await;
    assert!(runner.observed_abort(), "the abort is asked for first");
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Running,
        "the cleanup window has not closed yet",
    );

    // A runner that honours the abort inside the window reports its own
    // failure, not the watchdog's.
    advance(u64::try_from(SCHEDULED_TASK_CLEANUP_MS).expect("fits")).await;
    let failed = manager.get(&accepted.work.id, None).await.expect("exists");
    assert_eq!(failed.status, WorkStatus::Failed);
    assert_eq!(
        failed.error.as_deref(),
        Some(
            via_i18n::format(
                Locale::Zh,
                keys::WORK_SCHEDULED_TIMEOUT_ERROR,
                &[("minutes", "1")],
            )
            .as_str()
        ),
    );
    assert_eq!(failed.notification_status, NotificationStatus::Pending);
    assert_eq!(
        manager
            .wait(&accepted.work.id)
            .await
            .expect("terminal")
            .status,
        WorkStatus::Failed,
        "the waiter resolves rather than hanging",
    );
}

/// A runner that honours the abort inside the cleanup window keeps its own
/// failure: the watchdog does not overwrite it.
#[tokio::test(start_paused = true)]
async fn a_runner_that_stops_inside_the_window_reports_its_own_failure() {
    let manager = WorkManager::builder()
        .now(clock())
        .locale(Locale::Zh)
        .build();
    // `GatedRunner::new` returns as soon as it is aborted.
    let runner = GatedRunner::new();
    let accepted = manager
        .create_scheduled(
            NewScheduledWork::task("守规矩的任务", OWNER, Schedule::at(FUTURE))
                .timeout_ms(60_000)
                .runner(Arc::new(runner.clone())),
        )
        .await
        .expect("accepted");
    manager
        .fire_scheduled(std::slice::from_ref(&accepted.work.id))
        .await;
    settle().await;

    advance(60_000).await;
    let failed = manager.get(&accepted.work.id, None).await.expect("exists");
    assert_eq!(failed.status, WorkStatus::Failed);
    assert_eq!(
        failed.error.as_deref(),
        Some(t(Locale::Zh, keys::WORK_SCHEDULED_TIMEOUT_ABORT)),
        "the runner reported the abort reason it was given",
    );

    advance(u64::try_from(SCHEDULED_TASK_CLEANUP_MS).expect("fits") + 1_000).await;
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .error
            .as_deref(),
        Some(t(Locale::Zh, keys::WORK_SCHEDULED_TIMEOUT_ABORT)),
        "the closing cleanup window did not overwrite it",
    );
}

/// The watchdog is `scheduled_task`-only: ordinary Work runs as long as it
/// needs to.
#[tokio::test(start_paused = true)]
async fn ordinary_work_has_no_wall_clock_budget() {
    let manager = WorkManager::builder()
        .now(clock())
        .progress_check_ms(0)
        .build();
    let runner = GatedRunner::stubborn();
    let accepted = manager
        .create(NewWork::new("长跑", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;

    advance(4 * DEFAULT_SCHEDULED_TASK_TIMEOUT_MS.unsigned_abs()).await;
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Running,
    );
    assert!(!runner.observed_abort());

    runner.complete("finally");
    assert_eq!(
        manager
            .wait(&accepted.work.id)
            .await
            .expect("terminal")
            .result
            .as_deref(),
        Some("finally"),
    );
}

/// A reminder gets no watchdog either, however long its runner takes.
#[tokio::test(start_paused = true)]
async fn a_reminder_has_no_wall_clock_budget() {
    let manager = WorkManager::builder().now(clock()).build();
    let runner = GatedRunner::stubborn();
    let accepted = manager
        .create_scheduled(
            NewScheduledWork::reminder("慢提醒", OWNER, Schedule::at(FUTURE))
                .runner(Arc::new(runner.clone())),
        )
        .await
        .expect("accepted");
    assert_eq!(accepted.work.timeout_ms, None);

    manager
        .fire_scheduled(std::slice::from_ref(&accepted.work.id))
        .await;
    settle().await;
    advance(4 * DEFAULT_SCHEDULED_TASK_TIMEOUT_MS.unsigned_abs()).await;
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Running,
    );
    assert!(!runner.observed_abort());
    runner.complete("done");
    manager.wait(&accepted.work.id).await;
}
