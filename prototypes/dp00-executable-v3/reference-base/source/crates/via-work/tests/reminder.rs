//! The reminder scheduler, ported from
//! `server/test/reminder-scheduler.test.mjs`.
//!
//! Every ladder here runs under `#[tokio::test(start_paused = true)]`, so a
//! twenty-four-hour sleep costs microseconds and is exact: a test that asserts
//! "not one millisecond early" means it.

mod common;

use std::sync::Arc;

use common::{Events, advance, clock, seed_tasks, settle};
use pretty_assertions::assert_eq;
use serde_json::json;
use via_i18n::Locale;
use via_protocol::WorkStatus;
use via_work::limits::DEFAULT_REMINDER_STAGGER_MS;
use via_work::testing::ImmediateRunner;
use via_work::{
    NewScheduledWork, ReminderScheduler, Schedule, WorkEventKind, WorkManager, WorkQuery, WorkStore,
};

const OWNER: &str = "owner";

fn manager() -> WorkManager {
    WorkManager::builder()
        .now(clock())
        .runner(Arc::new(ImmediateRunner::completing("done")))
        .build()
}

/// A reminder fires at its due time, and **not one millisecond earlier**.
#[tokio::test(start_paused = true)]
async fn a_reminder_fires_at_its_due_time_and_not_before() {
    let manager = manager();
    let mut events = Events::new(manager.subscribe());
    let scheduler = ReminderScheduler::start(&manager, clock(), 0);

    let accepted = manager
        .create_scheduled(NewScheduledWork::reminder(
            "开会",
            OWNER,
            Schedule::at(common::BASE_MS + 60_000),
        ))
        .await
        .expect("accepted");
    settle().await;

    advance(59_990).await;
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Scheduled,
    );
    assert!(!events.saw(WorkEventKind::ScheduledFired));

    advance(20).await;
    assert!(events.saw(WorkEventKind::ScheduledFired));
    assert_eq!(
        manager
            .wait(&accepted.work.id)
            .await
            .expect("terminal")
            .status,
        WorkStatus::Completed,
    );
    scheduler.close();
}

/// The timer is armed to the **nearest** due time, and re-armed when a nearer
/// one arrives.
#[tokio::test(start_paused = true)]
async fn the_timer_follows_the_nearest_due_time() {
    let manager = manager();
    let scheduler = ReminderScheduler::start(&manager, clock(), 0);

    let far = manager
        .create_scheduled(NewScheduledWork::reminder(
            "远",
            OWNER,
            Schedule::at(common::BASE_MS + 3_600_000),
        ))
        .await
        .expect("accepted");
    settle().await;

    // A nearer one, created after the timer was already armed for the far one.
    let near = manager
        .create_scheduled(NewScheduledWork::reminder(
            "近",
            OWNER,
            Schedule::at(common::BASE_MS + 10_000),
        ))
        .await
        .expect("accepted");
    settle().await;

    advance(10_000).await;
    assert_eq!(
        manager.wait(&near.work.id).await.expect("terminal").status,
        WorkStatus::Completed,
    );
    assert_eq!(
        manager
            .get(&far.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Scheduled,
        "the far one is still waiting",
    );

    advance(3_600_000).await;
    assert_eq!(
        manager.wait(&far.work.id).await.expect("terminal").status,
        WorkStatus::Completed,
    );
    scheduler.close();
}

/// Upstream: *reschedule re-arms when a scheduled task is cancelled*.
#[tokio::test(start_paused = true)]
async fn cancelling_the_only_reminder_leaves_nothing_armed() {
    let manager = manager();
    let mut events = Events::new(manager.subscribe());
    let scheduler = ReminderScheduler::start(&manager, clock(), 0);

    let accepted = manager
        .create_scheduled(NewScheduledWork::reminder(
            "取消我",
            OWNER,
            Schedule::at(common::BASE_MS + 60_000),
        ))
        .await
        .expect("accepted");
    settle().await;
    manager.cancel(&accepted.work.id, Some(OWNER)).await;
    settle().await;

    advance(120_000).await;
    assert!(
        !events.saw(WorkEventKind::ScheduledFired),
        "a cancelled reminder never fires",
    );
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Cancelled,
    );
    scheduler.close();
}

/// Upstream: *reschedule does nothing when no future scheduled tasks exist* —
/// and the stronger property that follows from one `Sleep` rather than a poll:
/// a day passes with nothing scheduled and nothing happens at all.
#[tokio::test(start_paused = true)]
async fn an_empty_schedule_never_wakes_up() {
    let manager = manager();
    let mut events = Events::new(manager.subscribe());
    let scheduler = ReminderScheduler::start(&manager, clock(), 0);
    settle().await;

    advance(24 * 60 * 60 * 1_000).await;
    assert!(
        events.all().is_empty(),
        "nothing was scheduled, so nothing happened"
    );
    assert_eq!(
        scheduler.rearms(),
        1,
        "an empty schedule is computed once and then parked forever",
    );
    assert!(!scheduler.is_closed());
    scheduler.close();
    assert!(scheduler.is_closed());
}

/// One `Sleep`, not a poll: a reminder a day away is not touched by the
/// twenty-three hours before it.
#[tokio::test(start_paused = true)]
async fn a_distant_reminder_is_slept_through_in_one_go() {
    let manager = manager();
    let mut events = Events::new(manager.subscribe());
    let scheduler = ReminderScheduler::start(&manager, clock(), 0);

    let day = 24 * 60 * 60 * 1_000;
    let accepted = manager
        .create_scheduled(NewScheduledWork::reminder(
            "明天",
            OWNER,
            Schedule::at(common::BASE_MS + day),
        ))
        .await
        .expect("accepted");
    settle().await;

    advance(u64::try_from(day).expect("fits") - 1_000).await;
    assert_eq!(events.count(WorkEventKind::ScheduledFired), 0);
    assert_eq!(
        events.count(WorkEventKind::Scheduled),
        1,
        "the only event in twenty-three hours is the one that created it",
    );

    advance(1_000).await;
    assert_eq!(
        manager
            .wait(&accepted.work.id)
            .await
            .expect("terminal")
            .status,
        WorkStatus::Completed,
    );

    // **Never polls.** The timer was recomputed a handful of times — once at
    // start, once when the reminder was created, once when it fired, once when
    // the fire changed the state — and not once per interval. A scheduler that
    // woke every 50 ms would be at 1.7 million by now.
    assert!(
        scheduler.rearms() <= 8,
        "one Sleep, re-armed on mutation: {} recomputations across a day",
        scheduler.rearms(),
    );
    scheduler.close();
}

/// Upstream: *restoreOverdue staggers overdue tasks with increasing delays*.
#[tokio::test(start_paused = true)]
async fn an_overdue_backlog_is_staggered_in_due_order() {
    let directory = tempfile::tempdir().expect("tempdir");
    let record = |id: &str, at: i64| {
        json!({
            "id": id,
            "status": "scheduled",
            "kind": "reminder",
            "objective": id,
            "ownerId": OWNER,
            "sessionId": "voice",
            "createdAt": at,
            "schedule": {"type": "at", "at": at, "recurrence": "once"},
        })
    };
    // Written out of due order on purpose: the stagger follows `schedule.at`,
    // not the order they happen to sit in the file.
    let path = seed_tasks(
        directory.path(),
        json!([
            record("work-third", common::BASE_MS - 500),
            record("work-first", common::BASE_MS - 3_000),
            record("work-second", common::BASE_MS - 1_500),
        ]),
    );

    let manager = WorkManager::builder()
        .now(clock())
        .locale(Locale::Zh)
        .store(
            WorkStore::builder()
                .file_path(&path)
                .locale(Locale::Zh)
                .self_scheduling(true)
                .build(),
        )
        .build();
    let stagger = 30_000;
    let scheduler = ReminderScheduler::start(&manager, clock(), stagger);
    settle().await;

    let fired = async |manager: &WorkManager, id: &str| -> bool {
        manager.get(id, None).await.expect("exists").status != WorkStatus::Scheduled
    };

    assert!(fired(&manager, "work-first").await, "index 0 fires at once");
    assert!(!fired(&manager, "work-second").await);
    assert!(!fired(&manager, "work-third").await);

    advance(u64::try_from(stagger).expect("fits")).await;
    assert!(
        fired(&manager, "work-second").await,
        "index 1 fires one stagger later"
    );
    assert!(!fired(&manager, "work-third").await);

    advance(u64::try_from(stagger).expect("fits")).await;
    assert!(
        fired(&manager, "work-third").await,
        "index 2 fires two staggers later"
    );

    for id in ["work-first", "work-second", "work-third"] {
        assert_eq!(
            manager.wait(id).await.expect("terminal").status,
            WorkStatus::Completed,
            "{id}",
        );
    }
    scheduler.close();
}

/// A zero stagger replays the whole backlog at once, which is what
/// `VIA_REMINDER_STAGGER_MS=0` asks for.
#[tokio::test(start_paused = true)]
async fn a_zero_stagger_replays_the_backlog_together() {
    let directory = tempfile::tempdir().expect("tempdir");
    let path = seed_tasks(
        directory.path(),
        json!(
            (0..4)
                .map(|index| json!({
                    "id": format!("work-{index}"),
                    "status": "scheduled",
                    "kind": "reminder",
                    "objective": format!("overdue {index}"),
                    "ownerId": OWNER,
                    "sessionId": "voice",
                    "createdAt": common::BASE_MS - 1_000,
                    "schedule": {
                        "type": "at",
                        "at": common::BASE_MS - 1_000 - i64::from(index),
                        "recurrence": "once",
                    },
                }))
                .collect::<Vec<_>>()
        ),
    );
    let manager = WorkManager::builder()
        .now(clock())
        .store(
            WorkStore::builder()
                .file_path(&path)
                .self_scheduling(true)
                .build(),
        )
        .build();
    let scheduler = ReminderScheduler::start(&manager, clock(), 0);
    settle().await;

    for index in 0..4 {
        assert_eq!(
            manager
                .wait(&format!("work-{index}"))
                .await
                .expect("terminal")
                .status,
            WorkStatus::Completed,
        );
    }
    scheduler.close();
}

/// The stagger is clamped to the catalogued range, so a hostile configuration
/// cannot park the backlog for a week.
#[tokio::test(start_paused = true)]
async fn the_stagger_is_clamped_to_its_catalogued_ceiling() {
    let directory = tempfile::tempdir().expect("tempdir");
    let path = seed_tasks(
        directory.path(),
        json!([
            {
                "id": "work-a",
                "status": "scheduled",
                "kind": "reminder",
                "objective": "a",
                "ownerId": OWNER,
                "sessionId": "voice",
                "createdAt": common::BASE_MS - 2_000,
                "schedule": {"type": "at", "at": common::BASE_MS - 2_000, "recurrence": "once"},
            },
            {
                "id": "work-b",
                "status": "scheduled",
                "kind": "reminder",
                "objective": "b",
                "ownerId": OWNER,
                "sessionId": "voice",
                "createdAt": common::BASE_MS - 1_000,
                "schedule": {"type": "at", "at": common::BASE_MS - 1_000, "recurrence": "once"},
            },
        ]),
    );
    let manager = WorkManager::builder()
        .now(clock())
        .store(
            WorkStore::builder()
                .file_path(&path)
                .self_scheduling(true)
                .build(),
        )
        .build();
    // Far above the catalogued ceiling of 300 000 ms.
    let scheduler = ReminderScheduler::start(&manager, clock(), 7 * 24 * 60 * 60 * 1_000);
    settle().await;

    advance(300_000).await;
    assert_eq!(
        manager.get("work-b", None).await.expect("exists").status,
        WorkStatus::Completed,
        "the clamped ceiling is five minutes, not a week",
    );
    scheduler.close();
}

/// The shipped stagger default is the catalogued one.
#[test]
fn the_default_stagger_is_thirty_seconds() {
    assert_eq!(via_work::reminder::DEFAULT_STAGGER_MS, 30_000);
    assert_eq!(DEFAULT_REMINDER_STAGGER_MS, 30_000);
}

/// A closed scheduler stops firing.
#[tokio::test(start_paused = true)]
async fn a_closed_scheduler_stops_firing() {
    let manager = manager();
    let scheduler = ReminderScheduler::start(&manager, clock(), 0);
    let accepted = manager
        .create_scheduled(NewScheduledWork::reminder(
            "later",
            OWNER,
            Schedule::at(common::BASE_MS + 60_000),
        ))
        .await
        .expect("accepted");
    settle().await;
    scheduler.close();
    settle().await;

    advance(120_000).await;
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Scheduled,
    );
    assert_eq!(manager.list(WorkQuery::owner(OWNER)).await.len(), 1);
}
