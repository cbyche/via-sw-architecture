//! Result delivery, ported from `server/test/result-delivery.test.mjs`,
//! `server/test/offline-notifications.test.mjs` and
//! `server/test/task-result-projector.test.mjs`.
//!
//! The property every one of these protects is **exactly once**. The Work
//! manager raises `task.completed` *and* `task.notification.pending` for the
//! same result, and a client subscribes to both; the delivery lease is what
//! stops the user hearing it twice.

mod common;

use std::sync::Arc;

use common::{Events, advance, clock, settle};
use pretty_assertions::assert_eq;
use serde_json::json;
use via_protocol::WorkStatus;
use via_work::projector::project;
use via_work::testing::{GatedRunner, ImmediateRunner};
use via_work::{
    NewWork, NotificationClaim, NotificationStatus, PublicWork, WorkEventKind, WorkManager,
};

const OWNER: &str = "owner-1";
const SESSION: &str = "session-1";
const CLAIMANT: &str = "test";

/// The delivery-relevant half of `attachRealtimeGateway`'s subscriber: both
/// events lead to a claim, and the claim is what makes it once.
struct Harness {
    manager: WorkManager,
    spoken: Vec<String>,
    inline: Vec<serde_json::Value>,
    projected: Vec<String>,
}

impl Harness {
    fn new(manager: WorkManager) -> Self {
        Self {
            manager,
            spoken: Vec::new(),
            inline: Vec::new(),
            projected: Vec::new(),
        }
    }

    /// Drain the recorder and act on every event, exactly as the gateway's
    /// subscriber does.
    async fn drive(&mut self, events: &mut Events) {
        for event in events.take() {
            match event.kind {
                WorkEventKind::Completed | WorkEventKind::Failed => {
                    self.record(&event.task);
                    if let Some(inline) = event
                        .task
                        .result_metadata
                        .as_ref()
                        .and_then(|metadata| metadata.presentation.inline.as_ref())
                    {
                        self.inline.push(json!({
                            "id": format!("inline_{}", event.task.id),
                            "taskId": event.task.id,
                            "title": inline.title,
                            "format": inline.format,
                            "content": inline.content,
                        }));
                    }
                    self.claim(std::slice::from_ref(&event.task.id)).await;
                }
                WorkEventKind::NotificationPending => {
                    self.claim(std::slice::from_ref(&event.task.id)).await;
                }
                _ => {}
            }
        }
    }

    async fn claim(&mut self, ids: &[String]) {
        let claimed = self
            .manager
            .claim_notifications(
                NotificationClaim::new(OWNER, SESSION, CLAIMANT).only(ids.to_vec()),
            )
            .await;
        for work in claimed {
            self.record(&work);
            let speech = work
                .result_metadata
                .as_ref()
                .map(|metadata| metadata.presentation.speech.clone())
                .filter(|speech| !speech.is_empty())
                .or_else(|| work.result.clone())
                .or_else(|| work.error.clone())
                .unwrap_or_default();
            self.spoken.push(format!("[COMPLETE] {speech}"));
            self.manager
                .mark_notifications_delivered(std::slice::from_ref(&work.id), Some(CLAIMANT))
                .await;
        }
    }

    /// `recordTaskResult` — idempotent by construction, so recording twice
    /// yields one message.
    fn record(&mut self, work: &PublicWork) {
        if let Some(projection) = project(OWNER, SESSION, work)
            && !self.projected.contains(&projection.id)
        {
            self.projected.push(projection.id);
        }
    }
}

fn manager_with(metadata: Option<serde_json::Value>) -> WorkManager {
    let runner = metadata.map_or_else(
        || Arc::new(ImmediateRunner::completing("快速排序已实现")) as Arc<dyn via_work::WorkRunner>,
        |metadata| Arc::new(ImmediateRunner::completing_with("快速排序已实现", metadata)),
    );
    WorkManager::builder().now(clock()).runner(runner).build()
}

/// Upstream: *programming task: inline code goes to timeline.inline, speech
/// goes to injectResult, each exactly once*.
#[tokio::test(start_paused = true)]
async fn a_programming_result_yields_one_inline_block_and_one_spoken_line() {
    let manager = manager_with(Some(json!({
        "presentation": {
            "speech": "快速排序已实现，使用了经典的分治策略。",
            "inline": {
                "title": "快速排序实现",
                "format": "code",
                "content": "function quicksort(arr) { return arr }",
            },
        },
    })));
    let mut events = Events::new(manager.subscribe());
    let mut harness = Harness::new(manager.clone());

    let accepted = manager
        .create(NewWork::new("编写一个快速排序", OWNER).session(SESSION))
        .await
        .expect("accepted");
    manager.wait(&accepted.work.id).await;
    settle().await;
    harness.drive(&mut events).await;

    assert_eq!(harness.inline.len(), 1, "one inline block");
    assert_eq!(harness.inline[0]["title"], json!("快速排序实现"));
    assert_eq!(harness.inline[0]["format"], json!("code"));
    assert_eq!(harness.spoken.len(), 1, "spoken exactly once");
    assert!(harness.spoken[0].contains("[COMPLETE]"));
    assert!(harness.spoken[0].contains("快速排序已实现"));
    assert_eq!(
        harness.projected,
        vec![format!("agent:{}", accepted.work.id)]
    );
}

/// Upstream: *no duplication: task.completed and task.notification.pending both
/// fire but injectResult called once*.
#[tokio::test(start_paused = true)]
async fn both_events_fire_and_the_result_is_still_delivered_once() {
    let manager = manager_with(Some(json!({
        "presentation": {"speech": "快速排序已实现。", "inline": null},
    })));
    let mut events = Events::new(manager.subscribe());
    let mut harness = Harness::new(manager.clone());

    let accepted = manager
        .create(NewWork::new("编写一个快速排序", OWNER).session(SESSION))
        .await
        .expect("accepted");
    manager.wait(&accepted.work.id).await;
    settle().await;

    let kinds = events.kinds_for(&accepted.work.id);
    assert!(kinds.contains(&WorkEventKind::Completed));
    assert!(kinds.contains(&WorkEventKind::NotificationPending));
    harness.drive(&mut events).await;

    assert_eq!(harness.spoken.len(), 1, "two events, one delivery");
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .notification_status,
        NotificationStatus::Delivered,
    );
}

/// Upstream: *task with inline=null: no timeline.inline sent, injectResult
/// still called once*.
#[tokio::test(start_paused = true)]
async fn a_speech_only_result_shows_nothing_inline() {
    let manager = manager_with(Some(json!({
        "presentation": {"speech": "今天晴，25度。", "inline": null},
    })));
    let mut events = Events::new(manager.subscribe());
    let mut harness = Harness::new(manager.clone());

    let accepted = manager
        .create(NewWork::new("查询天气", OWNER).session(SESSION))
        .await
        .expect("accepted");
    manager.wait(&accepted.work.id).await;
    settle().await;
    harness.drive(&mut events).await;

    assert!(harness.inline.is_empty());
    assert_eq!(harness.spoken.len(), 1);
    assert!(harness.spoken[0].contains("今天晴"));
}

/// Upstream: *failed task: no inline shown, error delivered via injectResult
/// once*.
#[tokio::test(start_paused = true)]
async fn a_failed_result_is_announced_once_with_no_inline_block() {
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(ImmediateRunner::failing(
            "syntax error in generated code",
        )))
        .build();
    let mut events = Events::new(manager.subscribe());
    let mut harness = Harness::new(manager.clone());

    let accepted = manager
        .create(NewWork::new("编写一个快速排序", OWNER).session(SESSION))
        .await
        .expect("accepted");
    manager.wait(&accepted.work.id).await;
    settle().await;
    harness.drive(&mut events).await;

    assert!(harness.inline.is_empty());
    assert_eq!(harness.spoken.len(), 1);
    assert!(harness.spoken[0].contains("syntax error"));
    assert_eq!(harness.projected.len(), 1);
}

/// Upstream: *multiple programming tasks: each gets one inline and one
/// injectResult*.
#[tokio::test(start_paused = true)]
async fn two_results_are_delivered_once_each() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let mut harness = Harness::new(manager.clone());

    for (objective, title, body) in [
        ("实现冒泡排序", "冒泡排序", "function bubble(arr) {}"),
        ("实现归并排序", "归并排序", "function merge(arr) {}"),
    ] {
        let runner = ImmediateRunner::completing_with(
            objective,
            json!({
                "presentation": {
                    "speech": format!("{objective}已实现。"),
                    "inline": {"title": title, "format": "code", "content": body},
                },
            }),
        );
        let accepted = manager
            .create(
                NewWork::new(objective, OWNER)
                    .session(SESSION)
                    .runner(Arc::new(runner)),
            )
            .await
            .expect("accepted");
        manager.wait(&accepted.work.id).await;
        settle().await;
        harness.drive(&mut events).await;
    }

    assert_eq!(harness.inline.len(), 2);
    assert_eq!(harness.inline[0]["title"], json!("冒泡排序"));
    assert_eq!(harness.inline[1]["title"], json!("归并排序"));
    assert_eq!(harness.spoken.len(), 2);
    assert!(harness.spoken[0].contains("冒泡排序"));
    assert!(harness.spoken[1].contains("归并排序"));
    assert_eq!(harness.projected.len(), 2);
}

/// A cancelled Work is announced to nobody: `notificationStatus` is `none`, so
/// there is nothing to claim and nothing to project.
#[tokio::test(start_paused = true)]
async fn a_cancelled_work_is_delivered_to_nobody() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let mut harness = Harness::new(manager.clone());
    let runner = GatedRunner::new();

    let accepted = manager
        .create(
            NewWork::new("取消我", OWNER)
                .session(SESSION)
                .runner(Arc::new(runner.clone())),
        )
        .await
        .expect("accepted");
    settle().await;
    manager.cancel(&accepted.work.id, Some(OWNER)).await;
    settle().await;
    harness.drive(&mut events).await;

    assert!(harness.spoken.is_empty());
    assert!(
        harness.projected.is_empty(),
        "cancellation is not a transcript entry"
    );
    assert_eq!(
        manager
            .claim_notifications(NotificationClaim::new(OWNER, SESSION, CLAIMANT))
            .await
            .len(),
        0,
    );
}

// ── the offline hand-off ───────────────────────────────────────────────────

/// Upstream: *drops delayed progress after the task has completed*.
///
/// The delay is the host's (`VIA_OFFLINE_NOTIFICATION_DELAY_MS`); what this
/// asserts is the *predicate* the host re-checks after it — that the Work is
/// still active.
#[tokio::test(start_paused = true)]
async fn a_delayed_progress_notice_is_dropped_once_the_work_has_finished() {
    let manager = WorkManager::builder()
        .now(clock())
        .progress_check_ms(30_000)
        .build();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new();
    let accepted = manager
        .create(
            NewWork::new("build", OWNER)
                .session(SESSION)
                .runner(Arc::new(runner.clone())),
        )
        .await
        .expect("accepted");
    settle().await;
    advance(30_000).await;

    let announcement = events
        .first(WorkEventKind::ProgressCheck)
        .expect("an announcement");
    assert_eq!(
        announcement.task.work_state,
        via_protocol::WorkState::Active,
        "it was active when the announcement was raised",
    );

    runner.complete("done");
    manager.wait(&accepted.work.id).await;

    // The host's delayed re-read: no longer active, so nothing is shown.
    let current = manager.get(&accepted.work.id, None).await.expect("exists");
    assert_ne!(current.work_state, via_protocol::WorkState::Active);
    assert_eq!(current.status, WorkStatus::Completed);
}

/// Upstream: *delivers terminal notification only while its claim is pending*.
#[tokio::test(start_paused = true)]
async fn a_terminal_notice_is_shown_only_while_its_claim_is_pending() {
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(ImmediateRunner::completing("done")))
        .build();
    let accepted = manager
        .create(NewWork::new("build", OWNER).session(SESSION))
        .await
        .expect("accepted");
    manager.wait(&accepted.work.id).await;

    // First pass: pending, so the host shows it.
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .notification_status,
        NotificationStatus::Pending,
    );

    let claimed = manager
        .claim_notifications(NotificationClaim::new(OWNER, SESSION, CLAIMANT))
        .await;
    manager
        .mark_notifications_delivered(&[claimed[0].id.clone()], Some(CLAIMANT))
        .await;

    // Second pass: delivered, so the host shows nothing.
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .notification_status,
        NotificationStatus::Delivered,
    );
}

// ── the projection ─────────────────────────────────────────────────────────

/// Upstream: *records terminal work results identically* and *deduplicates a
/// terminal result recorded again during reconnect claim*.
#[tokio::test(start_paused = true)]
async fn two_results_project_to_two_stable_ids() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut projected = Vec::new();
    for (objective, content) in [("first", "first result"), ("second", "second result")] {
        let accepted = manager
            .create(
                NewWork::new(objective, "personal")
                    .session("main")
                    .turn(&format!("turn-{objective}"))
                    .runner(Arc::new(ImmediateRunner::completing(content))),
            )
            .await
            .expect("accepted");
        let work = manager.wait(&accepted.work.id).await.expect("terminal");
        // Twice, as upstream records it from two subscribers.
        let once = project("personal", "main", &work).expect("projects");
        let twice = project("personal", "main", &work).expect("projects");
        assert_eq!(once, twice, "re-projection is a no-op");
        projected.push(once);
    }

    assert_eq!(projected.len(), 2);
    assert_ne!(projected[0].id, projected[1].id);
    for projection in &projected {
        assert!(projection.id.starts_with("agent:"));
        assert_eq!(projection.source, "agent-result");
        assert_eq!(projection.role, "assistant");
        assert_eq!(projection.task_id, projection.id["agent:".len()..]);
    }
    assert_eq!(projected[0].content.as_deref(), Some("first result"));
    assert_eq!(projected[1].content.as_deref(), Some("second result"));
}
