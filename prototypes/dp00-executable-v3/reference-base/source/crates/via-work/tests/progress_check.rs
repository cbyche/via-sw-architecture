//! The long-running-work announcement, ported from
//! `server/test/progress-check.test.mjs`.

mod common;

use std::sync::Arc;

use common::{Events, advance, clock, settle};
use pretty_assertions::assert_eq;
use via_downstream::{ActivityTracker, RawSessionUpdate, SessionEvent};
use via_i18n::Locale;
use via_protocol::WorkStatus;
use via_work::testing::{GatedRunner, ScriptedCoordinatorQuery, delegation};
use via_work::{NewScheduledWork, NewWork, RunnerEvent, Schedule, WorkEventKind, WorkManager};

const OWNER: &str = "owner";
const CADENCE: i64 = 30_000;

fn bash_activity() -> SessionEvent {
    let mut tracker = ActivityTracker::new();
    tracker
        .project(&RawSessionUpdate {
            name: Some("bash".to_owned()),
            status: Some("running".to_owned()),
            raw_input: Some(serde_json::json!({"command": "npm test"})),
            ..RawSessionUpdate::tool_call("call_1")
        })
        .expect("projects")
}

fn manager() -> WorkManager {
    WorkManager::builder()
        .now(clock())
        .locale(Locale::Zh)
        .progress_check_ms(CADENCE)
        .build()
}

/// Upstream: *progress check timer emits task.progress.check with
/// activity-based message*.
#[tokio::test(start_paused = true)]
async fn the_announcement_is_composed_from_the_newest_activity() {
    let manager = manager();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new().emitting(vec![RunnerEvent::Activity(bash_activity())]);

    let accepted = manager
        .create(NewWork::new("测试进度任务", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Running,
    );
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .activity
            .len(),
        1,
    );

    advance(u64::try_from(CADENCE).expect("fits")).await;
    let event = events
        .first(WorkEventKind::ProgressCheck)
        .expect("an announcement");
    let message = event.message().expect("a message");
    assert!(message.contains("npm test"), "{message}");
    assert!(message.contains("执行"), "{message}");
    assert!(message.contains("测试进度任务"), "{message}");
    assert!(!event.is_delegated_message());

    runner.complete("done");
    manager.wait(&accepted.work.id).await;
}

/// Upstream: *progress check handles no activity gracefully*.
#[tokio::test(start_paused = true)]
async fn a_work_with_no_activity_still_announces() {
    let manager = manager();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new();

    let accepted = manager
        .create(NewWork::new("无活动任务", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;
    advance(u64::try_from(CADENCE).expect("fits")).await;

    let message = events
        .first(WorkEventKind::ProgressCheck)
        .expect("an announcement")
        .message()
        .expect("a message")
        .to_owned();
    assert!(message.contains("正在处理中"), "{message}");

    runner.complete("done");
    manager.wait(&accepted.work.id).await;
}

/// Upstream: *delegated task progress check queries coordinator*.
#[tokio::test(start_paused = true)]
async fn a_delegated_work_asks_the_coordinator_instead_of_its_stale_ring() {
    let manager = manager();
    let mut events = Events::new(manager.subscribe());
    let query = ScriptedCoordinatorQuery::answering("第三层正在运行测试");
    manager
        .configure_coordinator_query(Arc::new(query.clone()))
        .await;

    let runner = GatedRunner::new().emitting(vec![RunnerEvent::Delegated(delegation(
        "delegated-session",
        "session-2",
    ))]);
    let accepted = manager
        .create(NewWork::new("委托任务", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Delegated,
    );

    advance(u64::try_from(CADENCE).expect("fits")).await;
    let asked = query.asked();
    assert_eq!(asked.len(), 1, "the coordinator was asked exactly once");
    assert_eq!(asked[0].0, accepted.work.id);
    assert!(
        asked[0].1.contains("委托任务"),
        "the composed message is the question: {}",
        asked[0].1,
    );

    let event = events
        .first(WorkEventKind::ProgressCheck)
        .expect("an announcement");
    assert!(event.is_delegated_message());
    assert_eq!(event.message(), Some("第三层正在运行测试"));

    runner.complete("done");
    manager.wait(&accepted.work.id).await;
}

/// Upstream's `.catch` arm: a coordinator that refuses falls back to the
/// composed message, and says the answer did **not** come from the coordinator.
#[tokio::test(start_paused = true)]
async fn a_refusing_coordinator_falls_back_to_the_composed_message() {
    let manager = manager();
    let mut events = Events::new(manager.subscribe());
    manager
        .configure_coordinator_query(Arc::new(ScriptedCoordinatorQuery::refusing("没有找到")))
        .await;

    let runner = GatedRunner::new().emitting(vec![RunnerEvent::Delegated(delegation(
        "run-one",
        "target-one",
    ))]);
    let accepted = manager
        .create(NewWork::new("委托任务", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;
    advance(u64::try_from(CADENCE).expect("fits")).await;

    let event = events
        .first(WorkEventKind::ProgressCheck)
        .expect("an announcement");
    assert!(!event.is_delegated_message());
    let message = event.message().expect("a message");
    assert!(message.contains("委托任务"), "{message}");

    runner.complete("done");
    manager.wait(&accepted.work.id).await;
}

/// An empty coordinator answer falls back to the composed message but is still
/// a coordinator answer — upstream's `result?.content || message`.
#[tokio::test(start_paused = true)]
async fn an_empty_coordinator_answer_falls_back_without_lying_about_its_source() {
    let manager = manager();
    let mut events = Events::new(manager.subscribe());
    manager
        .configure_coordinator_query(Arc::new(ScriptedCoordinatorQuery::answering("   ")))
        .await;

    let runner = GatedRunner::new().emitting(vec![RunnerEvent::Delegated(delegation(
        "run-one",
        "target-one",
    ))]);
    let accepted = manager
        .create(NewWork::new("委托任务", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;
    advance(u64::try_from(CADENCE).expect("fits")).await;

    let event = events
        .first(WorkEventKind::ProgressCheck)
        .expect("an announcement");
    assert!(event.is_delegated_message());
    assert!(
        event
            .message()
            .is_some_and(|message| message.contains("委托任务")),
        "{:?}",
        event.message(),
    );

    runner.complete("done");
    manager.wait(&accepted.work.id).await;
}

/// Upstream: *progress check timer stops after task completes*.
#[tokio::test(start_paused = true)]
async fn the_announcement_stops_the_moment_the_work_settles() {
    let manager = manager();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new();
    let accepted = manager
        .create(NewWork::new("完成后停止进度检查", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;
    runner.complete("完成");
    manager.wait(&accepted.work.id).await;
    settle().await;

    let before = events.count(WorkEventKind::ProgressCheck);
    advance(u64::try_from(CADENCE).expect("fits") * 4).await;
    assert_eq!(events.count(WorkEventKind::ProgressCheck), before);
    assert_eq!(
        events.count(WorkEventKind::Progress),
        events.count(WorkEventKind::Progress),
    );
}

/// Upstream: *scheduled tasks stay quiet while they run*.
#[tokio::test(start_paused = true)]
async fn a_scheduled_task_never_announces_progress() {
    let manager = manager();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new().emitting(vec![RunnerEvent::Activity(bash_activity())]);
    let accepted = manager
        .create_scheduled(
            NewScheduledWork::task(
                "安静执行的定时任务",
                OWNER,
                Schedule::at(common::BASE_MS - 1_000),
            )
            .runner(Arc::new(runner.clone())),
        )
        .await
        .expect("accepted");
    manager
        .fire_scheduled(std::slice::from_ref(&accepted.work.id))
        .await;
    settle().await;

    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Running,
    );
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .progress_check_ms,
        None,
    );

    advance(u64::try_from(CADENCE).expect("fits") * 3).await;
    assert_eq!(events.count(WorkEventKind::ProgressCheck), 0);

    runner.complete("done");
    manager.wait(&accepted.work.id).await;
}

/// A `scheduled_task` restored from a `tasks.json` that *does* carry a
/// `progressCheckMs` still says nothing.
///
/// Upstream's own fixture (`task-manager-scheduled.test.mjs:224`) persists
/// `progressCheckMs: 300_000` on a `scheduled_task`, so the field alone cannot
/// be the gate: `start()` checks the **kind**
/// (`server/src/task/task-manager.mjs:583`).
#[tokio::test(start_paused = true)]
async fn a_restored_scheduled_task_with_a_persisted_cadence_is_still_quiet() {
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("tasks.json");
    let document = serde_json::json!({
        "version": 1,
        "tasks": [{
            "id": "work_restored_task",
            "status": "scheduled",
            "kind": "scheduled_task",
            "objective": "恢复的定时任务",
            "ownerId": OWNER,
            "sessionId": "voice",
            "createdAt": common::BASE_MS,
            "schedule": {"type": "at", "at": common::BASE_MS - 1, "recurrence": "once"},
            "timeoutMs": 1_800_000,
            "progressCheckMs": 300_000,
        }],
    });
    std::fs::write(
        &path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&document).expect("serializes")
        ),
    )
    .expect("seeded");

    let manager = WorkManager::builder()
        .now(clock())
        .locale(Locale::Zh)
        .progress_check_ms(CADENCE)
        // Ten minutes of one-second liveness ticks; every one must be seen so
        // the absence of an announcement among them means something.
        .event_capacity(4_096)
        .store(
            via_work::WorkStore::builder()
                .file_path(&path)
                .self_scheduling(true)
                .build(),
        )
        .build();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new().emitting(vec![RunnerEvent::Activity(bash_activity())]);
    manager
        .configure_scheduled_task_runner(Arc::new(runner.clone()))
        .await;

    assert_eq!(
        manager
            .get("work_restored_task", None)
            .await
            .expect("restored")
            .progress_check_ms,
        Some(300_000),
        "the persisted cadence survived, exactly as upstream's fixture has it",
    );
    manager
        .fire_scheduled(std::slice::from_ref(&"work_restored_task".to_owned()))
        .await;
    settle().await;
    assert_eq!(
        manager
            .get("work_restored_task", None)
            .await
            .expect("restored")
            .status,
        WorkStatus::Running,
    );

    advance(u64::try_from(300_000).expect("fits") * 2).await;
    assert_eq!(
        events.count(WorkEventKind::ProgressCheck),
        0,
        "a scheduled task stays quiet however its cadence was persisted",
    );

    runner.complete("done");
    manager.wait("work_restored_task").await;
}

/// A reminder is equally quiet.
#[tokio::test(start_paused = true)]
async fn a_reminder_never_announces_progress() {
    let manager = manager();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new();
    let accepted = manager
        .create_scheduled(
            NewScheduledWork::reminder("提醒", OWNER, Schedule::at(common::BASE_MS - 1))
                .runner(Arc::new(runner.clone())),
        )
        .await
        .expect("accepted");
    manager
        .fire_scheduled(std::slice::from_ref(&accepted.work.id))
        .await;
    settle().await;

    advance(u64::try_from(CADENCE).expect("fits") * 3).await;
    assert_eq!(events.count(WorkEventKind::ProgressCheck), 0);
    runner.complete("done");
    manager.wait(&accepted.work.id).await;
}

/// A zero cadence turns the announcement off entirely —
/// `Math.max(0, Number(progressCheckMs) || 0)`.
#[tokio::test(start_paused = true)]
async fn a_zero_cadence_disables_the_announcement() {
    let manager = WorkManager::builder()
        .now(clock())
        // Ten minutes of one-second liveness ticks is 600 events; the recorder
        // must see every one of them to prove none of them is an announcement.
        .event_capacity(4_096)
        .now(clock())
        .progress_check_ms(0)
        .build();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new();
    let accepted = manager
        .create(NewWork::new("安静", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .progress_check_ms,
        None,
    );

    advance(600_000).await;
    assert_eq!(events.count(WorkEventKind::ProgressCheck), 0);
    runner.complete("done");
    manager.wait(&accepted.work.id).await;
}

/// The announcement repeats on its cadence for as long as the Work runs, and
/// the elapsed minutes climb with it.
#[tokio::test(start_paused = true)]
async fn the_announcement_repeats_and_counts_the_minutes() {
    let manager = WorkManager::builder()
        .now(clock())
        .locale(Locale::Zh)
        .progress_check_ms(60_000)
        .build();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new();
    let accepted = manager
        .create(NewWork::new("长跑", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;

    advance(180_000).await;
    let announcements: Vec<String> = events
        .all()
        .iter()
        .filter(|event| event.kind == WorkEventKind::ProgressCheck)
        .filter_map(|event| event.message().map(str::to_owned))
        .collect();
    assert_eq!(announcements.len(), 3);
    assert!(
        announcements[0].contains("已运行 1 分钟"),
        "{}",
        announcements[0]
    );
    assert!(
        announcements[1].contains("已运行 2 分钟"),
        "{}",
        announcements[1]
    );
    assert!(
        announcements[2].contains("已运行 3 分钟"),
        "{}",
        announcements[2]
    );

    runner.complete("done");
    manager.wait(&accepted.work.id).await;
}

/// The one-second liveness tick runs beside the announcement and does not
/// persist — `task.progress` is emitted with `persist: false`.
#[tokio::test(start_paused = true)]
async fn the_liveness_tick_runs_every_second_while_active() {
    let manager = WorkManager::builder()
        .now(clock())
        .progress_check_ms(0)
        .build();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new();
    let accepted = manager
        .create(NewWork::new("tick", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;

    advance(5_000).await;
    let ticks = events.count(WorkEventKind::Progress);
    assert!(
        (4..=6).contains(&ticks),
        "five seconds, five ticks: {ticks}"
    );
    assert!(!WorkEventKind::Progress.persists());

    runner.complete("done");
    manager.wait(&accepted.work.id).await;
    settle().await;
    let after = events.count(WorkEventKind::Progress);
    advance(5_000).await;
    assert_eq!(
        events.count(WorkEventKind::Progress),
        after,
        "the tick stops with the Work",
    );
}
