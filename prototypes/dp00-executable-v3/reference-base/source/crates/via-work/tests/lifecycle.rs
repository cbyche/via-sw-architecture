//! The Work lifecycle, ported from `server/test/task-manager.test.mjs`.
//!
//! Every test here has an upstream counterpart, named in its doc comment. Where
//! a case has no upstream counterpart it says so and says why it was added.

mod common;

use std::sync::Arc;

use common::{Events, advance, clock, settle};
use pretty_assertions::assert_eq;
use serde_json::json;
use via_downstream::{RawSessionUpdate, SessionEvent};
use via_protocol::{WorkKind, WorkState, WorkStatus};
use via_work::testing::{GatedRunner, ImmediateRunner, ScriptedCanceler, delegation};
use via_work::{
    NewWork, NotificationClaim, NotificationStatus, PendingPermission, RetentionPolicy,
    RunnerEvent, WorkEventKind, WorkManager, WorkQuery, coordinator_lane,
};

const OWNER: &str = "owner";
const SESSION: &str = "voice";

fn tool_activity(id: &str, tool: &str) -> SessionEvent {
    let mut tracker = via_downstream::ActivityTracker::new();
    tracker
        .project(&RawSessionUpdate {
            name: Some(tool.to_owned()),
            status: Some("running".to_owned()),
            ..RawSessionUpdate::tool_call(id)
        })
        .expect("a tool call projects")
}

// ── admission and ordering ─────────────────────────────────────────────────

/// Upstream: *serializes work in the same coordinator lane while accepting
/// immediately*.
#[tokio::test(start_paused = true)]
async fn one_coordinator_lane_runs_one_work_at_a_time() {
    let manager = WorkManager::builder().now(clock()).build();
    let first_runner = GatedRunner::new().emitting(vec![RunnerEvent::Activity(tool_activity(
        "call-one", "read",
    ))]);
    let second_runner = GatedRunner::new();
    let lane = coordinator_lane(OWNER);

    let first = manager
        .create(
            NewWork::new("A", OWNER)
                .session(SESSION)
                .lane(&lane, 1)
                .runner(Arc::new(first_runner.clone())),
        )
        .await
        .expect("accepted");
    let second = manager
        .create(
            NewWork::new("B", OWNER)
                .session(SESSION)
                .lane(&lane, 1)
                .runner(Arc::new(second_runner.clone())),
        )
        .await
        .expect("accepted");

    // The receipt is always `queued`, even for Work the scheduler starts at
    // once: upstream defers `drain()` to a microtask.
    assert_eq!(first.work.status, WorkStatus::Queued);
    assert_eq!(second.work.status, WorkStatus::Queued);

    settle().await;
    assert_eq!(
        manager
            .get(&first.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Running,
    );
    assert_eq!(
        manager
            .get(&second.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Queued,
    );
    assert!(first_runner.is_running());
    assert!(!second_runner.is_running(), "the lane admits one at a time");

    let activity = manager
        .get(&first.work.id, None)
        .await
        .expect("exists")
        .activity;
    assert_eq!(activity.len(), 1);
    assert_eq!(activity[0].tool.as_deref(), Some("read"));

    first_runner.complete("A done");
    settle().await;
    assert!(second_runner.is_running(), "B starts only after A ends");
    second_runner.complete("B done");

    let first_done = manager.wait(&first.work.id).await.expect("terminal");
    let second_done = manager.wait(&second.work.id).await.expect("terminal");
    assert_eq!(first_done.result.as_deref(), Some("A done"));
    assert_eq!(second_done.result.as_deref(), Some("B done"));
}

/// Upstream: *runs a hidden control query before queued ordinary work*.
#[tokio::test(start_paused = true)]
async fn a_control_query_overtakes_queued_work_and_is_hidden_from_listings() {
    let manager = WorkManager::builder().now(clock()).build();
    let lane = coordinator_lane(OWNER);
    let running = GatedRunner::new();
    let ordinary = GatedRunner::new();
    let query = GatedRunner::new();

    let first = manager
        .create(
            NewWork::new("正在执行的普通任务", OWNER)
                .session(SESSION)
                .lane(&lane, 1)
                .runner(Arc::new(running.clone())),
        )
        .await
        .expect("accepted");
    let queued = manager
        .create(
            NewWork::new("排队的普通任务", OWNER)
                .session(SESSION)
                .lane(&lane, 1)
                .runner(Arc::new(ordinary.clone())),
        )
        .await
        .expect("accepted");
    settle().await;

    let control = manager
        .create(
            NewWork::new("高优先级状态查询", OWNER)
                .session(SESSION)
                .lane(&lane, 1)
                .kind(WorkKind::Control)
                .parent("delegated-work")
                .priority(100)
                .runner(Arc::new(query.clone())),
        )
        .await
        .expect("accepted");

    let visible = manager.list(WorkQuery::owner(OWNER)).await;
    assert!(
        !visible.iter().any(|work| work.id == control.work.id),
        "control Work is hidden",
    );
    let everything = manager.list(WorkQuery::owner(OWNER).with_control()).await;
    assert!(everything.iter().any(|work| work.id == control.work.id));

    running.complete("first done");
    settle().await;
    assert!(query.is_running(), "priority 100 overtakes the queued Work");
    assert!(!ordinary.is_running());

    query.complete("query done");
    settle().await;
    assert!(ordinary.is_running());
    ordinary.complete("ordinary done");

    for id in [&first.work.id, &control.work.id, &queued.work.id] {
        assert_eq!(
            manager.wait(id).await.expect("terminal").status,
            WorkStatus::Completed,
        );
    }
}

/// No upstream counterpart. Two Work items created in the same millisecond must
/// keep creation order, which upstream gets from V8's stable sort; here it comes
/// from a stable sort over an insertion-ordered map.
#[tokio::test(start_paused = true)]
async fn equal_priority_work_created_in_one_millisecond_keeps_its_order() {
    let manager = WorkManager::builder()
        .now(clock())
        .concurrency(1, 1)
        .build();
    let runners: Vec<GatedRunner> = (0..4).map(|_| GatedRunner::new()).collect();
    let mut ids = Vec::new();
    for runner in &runners {
        ids.push(
            manager
                .create(NewWork::new("objective", OWNER).runner(Arc::new(runner.clone())))
                .await
                .expect("accepted")
                .work
                .id,
        );
    }
    settle().await;

    for (index, runner) in runners.iter().enumerate() {
        assert!(
            runner.is_running(),
            "runner {index} should be running by now"
        );
        assert!(
            runners[index + 1..].iter().all(|later| !later.is_running()),
            "nothing after {index} may have started",
        );
        runner.complete("done");
        settle().await;
    }
    for id in ids {
        assert_eq!(
            manager.wait(&id).await.expect("terminal").status,
            WorkStatus::Completed
        );
    }
}

// ── permissions ────────────────────────────────────────────────────────────

/// Upstream: *publishes a bounded pending permission on the active work*.
#[tokio::test(start_paused = true)]
async fn a_pending_permission_is_published_and_cleared_by_its_own_resolution() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let permission =
        PendingPermission::pending("auth_one", "bash", "bash：npm test").with_work_id("work_one");
    let runner =
        GatedRunner::new().emitting(vec![RunnerEvent::PermissionRequested(permission.clone())]);

    let accepted = manager
        .create(
            NewWork::new("运行检查", OWNER)
                .session(SESSION)
                .runner(Arc::new(runner.clone())),
        )
        .await
        .expect("accepted");
    settle().await;

    let work = manager.get(&accepted.work.id, None).await.expect("exists");
    assert_eq!(
        work.authorization.as_ref().map(|auth| auth.id.as_str()),
        Some("auth_one"),
    );
    assert!(events.saw(WorkEventKind::PermissionRequested));

    // A resolution for a *different* permission announces itself but leaves the
    // pending one in place.
    let other = PendingPermission::pending("auth_two", "bash", "s")
        .with_status(via_work::PermissionStatus::Approved);
    runner.emit(RunnerEvent::PermissionResolved(other)).await;
    settle().await;
    assert!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .authorization
            .is_some(),
    );

    let resolved = permission.with_status(via_work::PermissionStatus::Approved);
    runner.emit(RunnerEvent::PermissionResolved(resolved)).await;
    settle().await;
    assert!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .authorization
            .is_none(),
    );
    let event = events
        .first(WorkEventKind::PermissionResolved)
        .expect("resolved");
    assert!(event.permission().is_some(), "the decision travels with it");

    runner.complete("完成");
    manager.wait(&accepted.work.id).await;
}

/// No upstream counterpart: upstream would put a pending permission back onto a
/// *completed* record, because its `onEvent` closure outlives the runner.
#[tokio::test(start_paused = true)]
async fn a_late_permission_never_lands_on_settled_work() {
    let manager = WorkManager::builder().now(clock()).build();
    let runner = GatedRunner::new();
    let accepted = manager
        .create(NewWork::new("objective", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;
    runner.complete("done");
    manager.wait(&accepted.work.id).await;

    runner
        .emit(RunnerEvent::PermissionRequested(
            PendingPermission::pending("auth_late", "bash", "s"),
        ))
        .await;
    settle().await;
    let work = manager.get(&accepted.work.id, None).await.expect("exists");
    assert_eq!(work.status, WorkStatus::Completed);
    assert!(work.authorization.is_none());
}

// ── delegation ─────────────────────────────────────────────────────────────

/// Upstream: *keeps delegated work active while releasing its coordinator
/// lane*.
#[tokio::test(start_paused = true)]
async fn delegated_work_stays_active_and_frees_its_lane() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let lane = coordinator_lane(OWNER);
    let delegated = delegation("run-one", "ses-target").with_presentation(json!({
        "speech": "项目已经接着做了。",
        "inline": null,
    }));
    let first = GatedRunner::new().emitting(vec![RunnerEvent::Delegated(delegated)]);
    let second = GatedRunner::new();

    let work = manager
        .create(
            NewWork::new("继续已有项目", OWNER)
                .session(SESSION)
                .lane(&lane, 1)
                .runner(Arc::new(first.clone())),
        )
        .await
        .expect("accepted");
    let query = manager
        .create(
            NewWork::new("查询任务状态", OWNER)
                .session(SESSION)
                .lane(&lane, 1)
                .runner(Arc::new(second.clone())),
        )
        .await
        .expect("accepted");
    settle().await;

    let public = manager.get(&work.work.id, None).await.expect("exists");
    assert_eq!(public.status, WorkStatus::Delegated);
    assert_eq!(public.work_state, WorkState::Active);
    let published = public.delegation.expect("a delegation");
    assert_eq!(published.status, "running");
    assert_eq!(published.title, "project");
    assert_eq!(
        published.presentation.expect("a presentation").speech,
        "项目已经接着做了。",
    );
    assert!(events.saw(WorkEventKind::Delegated));
    assert!(second.is_running(), "the lane was released on delegation");

    second.complete("仍在执行");
    first.complete("目标结果");
    let finished = manager.wait(&work.work.id).await.expect("terminal");
    assert_eq!(finished.status, WorkStatus::Completed);
    assert_eq!(finished.result.as_deref(), Some("目标结果"));
    assert_eq!(
        manager.wait(&query.work.id).await.expect("terminal").status,
        WorkStatus::Completed,
    );
}

// ── cancellation ───────────────────────────────────────────────────────────

/// Upstream: *cancels queued work without starting it*.
#[tokio::test(start_paused = true)]
async fn queued_work_cancels_without_ever_starting() {
    let manager = WorkManager::builder()
        .now(clock())
        .concurrency(1, 1)
        .build();
    let first = GatedRunner::new();
    let second = GatedRunner::new();

    let running = manager
        .create(NewWork::new("A", OWNER).runner(Arc::new(first.clone())))
        .await
        .expect("accepted");
    let queued = manager
        .create(NewWork::new("B", OWNER).runner(Arc::new(second.clone())))
        .await
        .expect("accepted");
    settle().await;
    assert_eq!(
        manager
            .get(&queued.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Queued,
    );

    let cancelled = manager
        .cancel(&queued.work.id, Some(OWNER))
        .await
        .expect("cancellable");
    assert_eq!(cancelled.status, WorkStatus::Cancelled);

    first.complete("A done");
    manager.wait(&running.work.id).await;
    settle().await;
    assert!(!second.is_running(), "a cancelled Work never runs");
    assert_eq!(
        manager
            .get(&queued.work.id, None)
            .await
            .expect("exists")
            .notification_status,
        NotificationStatus::None,
        "a cancelled Work announces nothing",
    );
}

/// Upstream: *aborts running work and only then releases its coordinator lane*.
#[tokio::test(start_paused = true)]
async fn running_work_holds_its_lane_until_the_cancellation_is_confirmed() {
    let manager = WorkManager::builder().now(clock()).build();
    let lane = coordinator_lane(OWNER);
    // A runner that notices the abort and keeps going: until it settles,
    // nothing has confirmed that the request stopped.
    let first = GatedRunner::stubborn();
    let second = GatedRunner::new();
    let canceler = ScriptedCanceler::requesting();

    let running = manager
        .create(
            NewWork::new("A", OWNER)
                .lane(&lane, 1)
                .runner(Arc::new(first.clone()))
                .canceler(Arc::new(canceler.clone())),
        )
        .await
        .expect("accepted");
    let queued = manager
        .create(
            NewWork::new("B", OWNER)
                .lane(&lane, 1)
                .runner(Arc::new(second.clone())),
        )
        .await
        .expect("accepted");
    settle().await;

    let cancellation = manager.cancel(&running.work.id, Some(OWNER));
    tokio::pin!(cancellation);
    // The Work is `cancelling` the moment the request lands, before anything
    // has confirmed anything.
    tokio::select! {
        _ = &mut cancellation => panic!("cancellation resolved before it was confirmed"),
        () = settle() => {}
    }
    assert_eq!(
        manager
            .get(&running.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Cancelling,
    );
    assert!(first.observed_abort(), "the canceler aborted the runner");
    assert!(
        !second.is_running(),
        "the lane is still held while cancelling"
    );

    // The runner settles: *that* is the confirmation.
    first.fail("aborted");
    let cancelled = cancellation.await.expect("cancellable");
    assert_eq!(cancelled.status, WorkStatus::Cancelled);
    assert_eq!(cancelled.error, None, "a cancelled Work carries no error");
    assert_eq!(canceler.seen(), vec![WorkStatus::Running]);

    settle().await;
    assert!(second.is_running(), "the lane is released once confirmed");
    second.complete("B done");
    manager.wait(&queued.work.id).await;
}

// ── notifications ──────────────────────────────────────────────────────────

/// Upstream: *claims a completed result once and releases an unplayed claim*.
#[tokio::test(start_paused = true)]
async fn a_result_is_claimed_once_and_can_be_given_back() {
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(ImmediateRunner::completing("结果")))
        .build();
    let accepted = manager
        .create(NewWork::new("完成工作", OWNER).session(SESSION))
        .await
        .expect("accepted");
    manager.wait(&accepted.work.id).await;

    let claim = NotificationClaim::new(OWNER, SESSION, "voice-one");
    let claimed = manager.claim_notifications(claim.clone()).await;
    assert_eq!(claimed.len(), 1);
    assert_eq!(
        manager.claim_notifications(claim).await.len(),
        0,
        "a claimed notification is invisible to the next claimant",
    );

    manager
        .release_notification_claims(std::slice::from_ref(&accepted.work.id), Some("voice-one"))
        .await;
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .notification_status,
        NotificationStatus::Pending,
    );
}

/// Upstream: *a new voice session can deliver unfinished results for the same
/// owner*, and *prefers the originating session unless cross-session recovery
/// is explicit*.
#[tokio::test(start_paused = true)]
async fn cross_session_recovery_is_explicit() {
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(ImmediateRunner::completing("结果")))
        .build();
    let accepted = manager
        .create(NewWork::new("跨会话工作", OWNER).session("old-voice-session"))
        .await
        .expect("accepted");
    manager.wait(&accepted.work.id).await;

    assert_eq!(
        manager
            .claim_notifications(NotificationClaim::new(OWNER, "new-voice-session", "other"))
            .await
            .len(),
        0,
        "another session does not take it by default",
    );
    let claimed = manager
        .claim_notifications(
            NotificationClaim::new(OWNER, "new-voice-session", "new-client").across_sessions(),
        )
        .await;
    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].id, accepted.work.id);
}

/// Upstream: *reclaims an expired notification delivery lease*.
#[tokio::test(start_paused = true)]
async fn an_expired_delivery_lease_is_reclaimed() {
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(ImmediateRunner::completing("结果")))
        .retention(RetentionPolicy {
            // Below the catalogued floor on purpose: the clamp raises it to
            // 5 000 ms, which is what the ladder below is timed against.
            notification_claim_ttl_ms: 1,
            ..RetentionPolicy::default()
        })
        .build();
    let accepted = manager
        .create(NewWork::new("租约恢复", OWNER).session(SESSION))
        .await
        .expect("accepted");
    manager.wait(&accepted.work.id).await;

    manager
        .claim_notifications(NotificationClaim::new(OWNER, SESSION, "stale-client"))
        .await;
    advance(4_000).await;
    assert_eq!(
        manager
            .claim_notifications(NotificationClaim::new(OWNER, SESSION, "eager-client"))
            .await
            .len(),
        0,
        "the lease is still inside the clamped 5 s TTL",
    );

    advance(2_000).await;
    let reclaimed = manager
        .claim_notifications(NotificationClaim::new(OWNER, SESSION, "new-client"))
        .await;
    assert_eq!(reclaimed.len(), 1);
    assert_eq!(reclaimed[0].id, accepted.work.id);
}

/// Only the lease holder may mark a notification delivered or renew it.
#[tokio::test(start_paused = true)]
async fn only_the_lease_holder_may_deliver_or_renew() {
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(ImmediateRunner::completing("结果")))
        .build();
    let accepted = manager
        .create(NewWork::new("交付", OWNER).session(SESSION))
        .await
        .expect("accepted");
    manager.wait(&accepted.work.id).await;
    manager
        .claim_notifications(NotificationClaim::new(OWNER, SESSION, "holder"))
        .await;

    let ids = vec![accepted.work.id.clone()];
    assert_eq!(
        manager
            .mark_notifications_delivered(&ids, Some("impostor"))
            .await,
        0,
    );
    assert_eq!(
        manager
            .renew_notification_claims(&ids, Some("impostor"))
            .await,
        0
    );
    assert_eq!(
        manager
            .release_notification_claims(&ids, Some("impostor"))
            .await,
        0
    );

    assert_eq!(
        manager
            .renew_notification_claims(&ids, Some("holder"))
            .await,
        1
    );
    assert_eq!(
        manager
            .mark_notifications_delivered(&ids, Some("holder"))
            .await,
        1
    );

    let work = manager.get(&accepted.work.id, None).await.expect("exists");
    assert_eq!(work.notification_status, NotificationStatus::Delivered);
    assert!(work.notification_delivered_at.is_some());
    assert_eq!(
        manager
            .mark_notifications_delivered(&ids, Some("holder"))
            .await,
        0,
        "delivery is final",
    );
}

// ── retention ──────────────────────────────────────────────────────────────

/// Upstream: *does not evict pending notifications to satisfy delivered history
/// limit*.
#[tokio::test(start_paused = true)]
async fn a_pending_notification_is_never_evicted_for_the_history_cap() {
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(ImmediateRunner::completing("结果")))
        .retention(RetentionPolicy {
            // Clamped up to the catalogued floor of 10; the point stands with
            // eleven Work items.
            max_terminal_tasks_per_owner: 1,
            ..RetentionPolicy::default()
        })
        .build();

    let mut ids = Vec::new();
    for index in 0..11 {
        let accepted = manager
            .create(NewWork::new(&format!("work {index}"), OWNER).session(SESSION))
            .await
            .expect("accepted");
        manager.wait(&accepted.work.id).await;
        ids.push(accepted.work.id);
    }
    manager.prune().await;
    assert_eq!(
        manager.list(WorkQuery::owner(OWNER)).await.len(),
        11,
        "every one of them still owes the user a notification",
    );

    // Deliver them all, and the cap applies.
    manager
        .claim_notifications(NotificationClaim::new(OWNER, SESSION, "client"))
        .await;
    manager
        .mark_notifications_delivered(&ids, Some("client"))
        .await;
    manager.prune().await;
    assert_eq!(manager.list(WorkQuery::owner(OWNER)).await.len(), 10);
}

/// No upstream counterpart as a unit test: the age rule, with the two different
/// TTLs, exercised over a 7-day ladder in microseconds.
#[tokio::test(start_paused = true)]
async fn the_two_retention_ttls_apply_to_the_two_kinds_of_terminal_work() {
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(ImmediateRunner::completing("结果")))
        .build();
    let delivered = manager
        .create(NewWork::new("delivered", OWNER).session(SESSION))
        .await
        .expect("accepted");
    let owed = manager
        .create(NewWork::new("owed", OWNER).session(SESSION))
        .await
        .expect("accepted");
    manager.wait(&delivered.work.id).await;
    manager.wait(&owed.work.id).await;

    manager
        .claim_notifications(
            NotificationClaim::new(OWNER, SESSION, "client").only(vec![delivered.work.id.clone()]),
        )
        .await;
    manager
        .mark_notifications_delivered(std::slice::from_ref(&delivered.work.id), Some("client"))
        .await;

    // A day and a millisecond: the delivered one is past its TTL, the owed one
    // has six more days.
    advance(24 * 60 * 60 * 1_000 + 1).await;
    manager.prune().await;
    assert!(manager.get(&delivered.work.id, None).await.is_none());
    assert!(manager.get(&owed.work.id, None).await.is_some());

    advance(7 * 24 * 60 * 60 * 1_000).await;
    manager.prune().await;
    assert!(
        manager.get(&owed.work.id, None).await.is_none(),
        "even an undelivered result is dropped after seven days",
    );
}

// ── duplicate submission ───────────────────────────────────────────────────

/// Upstream: *reuses a persisted submission key instead of running duplicate
/// work* — the second half, across a restart, is in `restart.rs`.
#[tokio::test(start_paused = true)]
async fn a_repeated_submission_key_returns_the_first_work_and_runs_nothing() {
    let runner = ImmediateRunner::completing("完成");
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(runner.clone()))
        .build();

    let first = manager
        .create(
            NewWork::new("只执行一次", OWNER)
                .session(SESSION)
                .submission_key("delegation:voice:turn-one"),
        )
        .await
        .expect("accepted");
    manager.wait(&first.work.id).await;

    let duplicate = manager
        .create(
            NewWork::new("不要再次执行", OWNER)
                .session(SESSION)
                .submission_key("delegation:voice:turn-one"),
        )
        .await
        .expect("accepted");
    settle().await;

    assert_eq!(duplicate.work.id, first.work.id);
    assert!(duplicate.reused);
    assert_eq!(runner.runs(), 1);
    assert_eq!(
        duplicate.work.objective, "只执行一次",
        "the *first* Work is returned, not the second request",
    );
}

/// The key is per owner, is trimmed, and an empty one is no key at all.
#[tokio::test(start_paused = true)]
async fn the_submission_key_is_scoped_trimmed_and_optional() {
    let runner = ImmediateRunner::completing("done");
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(runner.clone()))
        .build();

    let first = manager
        .create(NewWork::new("A", OWNER).submission_key("  turn-one  "))
        .await
        .expect("accepted");
    let trimmed = manager
        .create(NewWork::new("B", OWNER).submission_key("turn-one"))
        .await
        .expect("accepted");
    assert!(trimmed.reused, "the key is trimmed on both sides");
    assert_eq!(trimmed.work.id, first.work.id);

    let other_owner = manager
        .create(NewWork::new("C", "someone-else").submission_key("turn-one"))
        .await
        .expect("accepted");
    assert!(!other_owner.reused, "keys do not cross owners");

    let trimmed_objective = manager
        .create(NewWork::new("  \t 有空白的目标 \n ", OWNER))
        .await
        .expect("accepted");
    assert_eq!(
        trimmed_objective.work.objective, "有空白的目标",
        "`String(objective || '').trim()`",
    );
    assert_eq!(
        manager
            .create(NewWork::new("   ", OWNER))
            .await
            .expect("accepted")
            .work
            .objective,
        "",
        "an all-whitespace objective trims to nothing rather than to spaces",
    );

    let blank_one = manager
        .create(NewWork::new("D", OWNER).submission_key("   "))
        .await
        .expect("accepted");
    let blank_two = manager
        .create(NewWork::new("E", OWNER).submission_key(""))
        .await
        .expect("accepted");
    assert!(!blank_one.reused);
    assert!(!blank_two.reused, "an empty key never matches another");
    assert_ne!(blank_one.work.id, blank_two.work.id);
}

// ── result metadata ────────────────────────────────────────────────────────

/// Upstream: *publishes only presentation metadata from a completed result*.
#[tokio::test(start_paused = true)]
async fn only_the_presentation_escapes_a_completed_result() {
    let metadata = json!({
        "presentation": {
            "speech": "报告已经生成。",
            "inline": {"title": "报告", "format": "markdown", "content": "# 完成"},
        },
        "backendRef": {"sessionId": "backend-session", "directory": "/private/project"},
        "delegation": {"id": "delegation-id", "sessionId": "target-session"},
    });
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(ImmediateRunner::completing_with(
            "报告已经生成",
            metadata,
        )))
        .build();
    let mut events = Events::new(manager.subscribe());

    let accepted = manager
        .create(NewWork::new("生成报告", OWNER).session(SESSION))
        .await
        .expect("accepted");
    let completed = manager.wait(&accepted.work.id).await.expect("terminal");
    settle().await;

    let published = serde_json::to_value(completed.result_metadata.clone()).expect("serializes");
    assert_eq!(
        published,
        json!({
            "presentation": {
                "speech": "报告已经生成。",
                "inline": {"title": "报告", "format": "markdown", "content": "# 完成"},
            }
        }),
    );

    let event = events.first(WorkEventKind::Completed).expect("completed");
    assert_eq!(event.task.result_metadata, completed.result_metadata);
}

// ── ownership ──────────────────────────────────────────────────────────────

/// No upstream counterpart as a unit test: `get`, `cancel` and `list` all take
/// an owner filter, and upstream's route handlers rely on it for
/// authorization.
#[tokio::test(start_paused = true)]
async fn another_owner_can_neither_see_nor_cancel_the_work() {
    let manager = WorkManager::builder().now(clock()).build();
    let runner = GatedRunner::new();
    let accepted = manager
        .create(NewWork::new("private", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;

    assert!(
        manager
            .get(&accepted.work.id, Some("intruder"))
            .await
            .is_none()
    );
    assert!(
        manager
            .cancel(&accepted.work.id, Some("intruder"))
            .await
            .is_none()
    );
    assert!(
        manager.list(WorkQuery::owner("intruder")).await.is_empty(),
        "and it is not in their listing either",
    );
    assert!(manager.get(&accepted.work.id, Some(OWNER)).await.is_some());
    assert!(runner.is_running(), "the intruder changed nothing");

    runner.complete("done");
    manager.wait(&accepted.work.id).await;
}

/// Cancelling something that is already terminal, or that never existed,
/// answers `None` rather than inventing a Work.
#[tokio::test(start_paused = true)]
async fn cancelling_finished_or_unknown_work_answers_nothing() {
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(ImmediateRunner::completing("done")))
        .build();
    assert!(manager.cancel("work_nonexistent", None).await.is_none());
    assert!(manager.wait("work_nonexistent").await.is_none());
    assert!(manager.get("work_nonexistent", None).await.is_none());

    let accepted = manager
        .create(NewWork::new("objective", OWNER))
        .await
        .expect("accepted");
    manager.wait(&accepted.work.id).await;
    assert!(
        manager
            .cancel(&accepted.work.id, Some(OWNER))
            .await
            .is_none(),
        "a completed Work is not cancellable",
    );
}

// ── failure ────────────────────────────────────────────────────────────────

/// A runner with no runner configured fails with the catalogued sentence.
#[tokio::test(start_paused = true)]
async fn work_with_no_runner_fails_with_the_catalogued_sentence() {
    let manager = WorkManager::builder()
        .now(clock())
        .locale(via_i18n::Locale::Zh)
        .build();
    let accepted = manager
        .create(NewWork::new("objective", OWNER))
        .await
        .expect("accepted");
    let failed = manager.wait(&accepted.work.id).await.expect("terminal");
    assert_eq!(failed.status, WorkStatus::Failed);
    assert_eq!(failed.error.as_deref(), Some("未配置后台 Agent 执行器"));
    assert_eq!(failed.notification_status, NotificationStatus::Pending);
}

/// A failing runner's message becomes `task.error` verbatim.
#[tokio::test(start_paused = true)]
async fn a_failing_runner_publishes_its_own_sentence() {
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(ImmediateRunner::failing(
            "syntax error in generated code",
        )))
        .build();
    let mut events = Events::new(manager.subscribe());
    let accepted = manager
        .create(NewWork::new("编写一个快速排序", OWNER))
        .await
        .expect("accepted");
    let failed = manager.wait(&accepted.work.id).await.expect("terminal");
    settle().await;

    assert_eq!(failed.status, WorkStatus::Failed);
    assert_eq!(
        failed.error.as_deref(),
        Some("syntax error in generated code")
    );
    assert!(
        failed.result_metadata.is_none(),
        "a failure carries no inline block"
    );
    assert_eq!(
        events.kinds_for(&accepted.work.id),
        vec![
            WorkEventKind::Accepted,
            WorkEventKind::Running,
            WorkEventKind::Failed,
            WorkEventKind::NotificationPending,
        ],
    );
}
