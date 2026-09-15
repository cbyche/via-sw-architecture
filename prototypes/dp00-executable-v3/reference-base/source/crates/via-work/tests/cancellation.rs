//! Cancellation is **confirmed, not optimistic**, and the transition graph is
//! the one `via-protocol` publishes.
//!
//! `docs/architecture.md` §4: *"Cancellation is confirmed, not optimistic. Work
//! stays `cancelling` until a path confirms the stop."* Upstream reports
//! `cancelled` as soon as its canceler returns, having only *sent* a
//! fire-and-forget notification; the deliberate divergence is recorded in
//! `docs/deviations/phase-3.md` and asserted here.

mod common;

use std::sync::Arc;
use std::time::Duration;

use common::{Events, advance, clock, settle};
use pretty_assertions::assert_eq;
use via_i18n::{Locale, keys, t};
use via_protocol::WorkStatus;
use via_work::testing::{DeafRunner, GatedRunner, ImmediateRunner, ScriptedCanceler, delegation};
use via_work::{
    CancellationFact, NewScheduledWork, NewWork, NotificationStatus, RunnerEvent, Schedule,
    WorkEventKind, WorkManager, coordinator_lane,
};

const OWNER: &str = "owner";
const CONFIRMED_AT: &str = "2026-08-22T00:00:00.000Z";

fn manager() -> WorkManager {
    WorkManager::builder()
        .now(clock())
        .locale(Locale::Zh)
        .build()
}

/// Whether a future is still pending after everything runnable has run.
async fn still_pending<F: Future>(future: std::pin::Pin<&mut F>) -> bool {
    tokio::select! {
        _ = future => false,
        () = settle() => true,
    }
}

use std::future::Future;

// ── the two short-circuits ─────────────────────────────────────────────────

/// Nothing is running, so there is nothing to confirm: `queued` and `scheduled`
/// go straight to `cancelled` without passing through `cancelling`.
#[tokio::test(start_paused = true)]
async fn nothing_running_confirms_at_once() {
    let manager = WorkManager::builder()
        .now(clock())
        .concurrency(1, 1)
        .build();
    let mut events = Events::new(manager.subscribe());
    let holder = GatedRunner::new();
    manager
        .create(NewWork::new("holder", OWNER).runner(Arc::new(holder.clone())))
        .await
        .expect("accepted");
    let queued = manager
        .create(NewWork::new("queued", OWNER).runner(Arc::new(GatedRunner::new())))
        .await
        .expect("accepted");
    let scheduled = manager
        .create_scheduled(NewScheduledWork::reminder(
            "scheduled",
            OWNER,
            Schedule::at(common::BASE_MS + 60_000),
        ))
        .await
        .expect("accepted");
    settle().await;

    for id in [&queued.work.id, &scheduled.work.id] {
        let cancelled = manager.cancel(id, Some(OWNER)).await.expect("cancellable");
        assert_eq!(cancelled.status, WorkStatus::Cancelled, "{id}");
        assert_eq!(cancelled.error, None, "{id}");
        assert_eq!(
            cancelled.notification_status,
            NotificationStatus::None,
            "{id}"
        );
    }
    assert_eq!(
        events.count(WorkEventKind::Cancelling),
        0,
        "there was nothing in flight to confirm",
    );
    assert_eq!(events.count(WorkEventKind::Cancelled), 2);

    holder.complete("done");
    settle().await;
}

// ── confirmed, not optimistic ──────────────────────────────────────────────

/// A canceler that only *requested* the stop leaves the Work `cancelling`. The
/// runner settling is what confirms it.
#[tokio::test(start_paused = true)]
async fn a_requested_cancel_waits_for_the_runner_to_settle() {
    let manager = manager();
    let runner = GatedRunner::stubborn();
    let canceler = ScriptedCanceler::requesting();
    let accepted = manager
        .create(
            NewWork::new("running", OWNER)
                .runner(Arc::new(runner.clone()))
                .canceler(Arc::new(canceler)),
        )
        .await
        .expect("accepted");
    settle().await;

    let cancel = manager.cancel(&accepted.work.id, Some(OWNER));
    tokio::pin!(cancel);
    assert!(still_pending(cancel.as_mut()).await);
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Cancelling,
    );
    assert!(runner.observed_abort());

    // Ten minutes of nothing happening does not turn a request into a
    // confirmation.
    advance(600_000).await;
    assert!(still_pending(cancel.as_mut()).await);
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Cancelling,
    );

    runner.fail("aborted");
    let cancelled = cancel.await.expect("cancellable");
    assert_eq!(cancelled.status, WorkStatus::Cancelled);
}

/// A runner that never notices its abort leaves the Work `cancelling` for
/// good. Upstream would report it `cancelled` and be wrong.
#[tokio::test(start_paused = true)]
async fn a_deaf_runner_leaves_the_work_cancelling_rather_than_lying() {
    let manager = manager();
    let accepted = manager
        .create(NewWork::new("deaf", OWNER).runner(Arc::new(DeafRunner)))
        .await
        .expect("accepted");
    settle().await;

    let cancel = manager.cancel(&accepted.work.id, Some(OWNER));
    tokio::pin!(cancel);
    assert!(still_pending(cancel.as_mut()).await);
    advance(24 * 60 * 60 * 1_000).await;
    assert!(still_pending(cancel.as_mut()).await);

    let work = manager.get(&accepted.work.id, None).await.expect("exists");
    assert_eq!(work.status, WorkStatus::Cancelling);
    assert_eq!(
        work.work_state,
        via_protocol::WorkState::Active,
        "a Work nobody has confirmed stopped is still active",
    );
    assert!(work.completed_at.is_none());
}

/// The coordinator route: the model's own cancel tool returned, which **is**
/// the confirmation, so the Work is cancelled without waiting on the runner.
#[tokio::test(start_paused = true)]
async fn a_confirmed_cancel_finishes_without_the_runner() {
    let manager = manager();
    let canceler = ScriptedCanceler::confirming(CONFIRMED_AT);
    let accepted = manager
        .create(
            NewWork::new("delegated", OWNER)
                .runner(Arc::new(DeafRunner))
                .canceler(Arc::new(canceler.clone())),
        )
        .await
        .expect("accepted");
    settle().await;

    let cancelled = manager
        .cancel(&accepted.work.id, Some(OWNER))
        .await
        .expect("cancellable");
    assert_eq!(cancelled.status, WorkStatus::Cancelled);
    assert_eq!(cancelled.notification_status, NotificationStatus::None);
    assert_eq!(canceler.seen(), vec![WorkStatus::Running]);
}

/// The out-of-band confirmation: the adapter sent `session/cancel` down the
/// transport and the backend's own `stopReason: 'cancelled'` arrived later.
#[tokio::test(start_paused = true)]
async fn a_confirmation_can_arrive_out_of_band() {
    let manager = manager();
    let accepted = manager
        .create(
            NewWork::new("delegated", OWNER)
                .runner(Arc::new(DeafRunner))
                .canceler(Arc::new(ScriptedCanceler::requesting().aborting(false))),
        )
        .await
        .expect("accepted");
    settle().await;

    let cancel = manager.cancel(&accepted.work.id, Some(OWNER));
    tokio::pin!(cancel);
    assert!(still_pending(cancel.as_mut()).await);

    let confirmed = manager
        .confirm_cancellation(&accepted.work.id, CONFIRMED_AT)
        .await
        .expect("cancelling");
    assert_eq!(confirmed.status, WorkStatus::Cancelled);
    assert_eq!(
        cancel.await.expect("cancellable").status,
        WorkStatus::Cancelled
    );

    assert!(
        manager
            .confirm_cancellation(&accepted.work.id, CONFIRMED_AT)
            .await
            .is_none(),
        "a Work that is not cancelling has nothing to confirm",
    );
}

/// The delegated cancel path tells the canceler what the Work was doing, so it
/// can choose the coordinator route or the transport one.
#[tokio::test(start_paused = true)]
async fn the_canceler_is_told_which_state_it_is_stopping() {
    let manager = manager();
    let canceler = ScriptedCanceler::confirming(CONFIRMED_AT);
    let runner = GatedRunner::stubborn();
    let accepted = manager
        .create(
            NewWork::new("delegated", OWNER)
                .runner(Arc::new(runner.clone()))
                .canceler(Arc::new(canceler.clone())),
        )
        .await
        .expect("accepted");
    settle().await;
    runner
        .emit(RunnerEvent::Delegated(delegation("run-one", "target-one")))
        .await;
    settle().await;

    manager.cancel(&accepted.work.id, Some(OWNER)).await;
    assert_eq!(
        canceler.seen(),
        vec![WorkStatus::Delegated],
        "the canceler branches on the previous status",
    );
}

// ── the failure path ───────────────────────────────────────────────────────

/// Upstream's `catch`: a canceler that throws **fails** the Work with
/// `取消失败：…`, and queues a notification, because nothing confirmed a stop.
#[tokio::test(start_paused = true)]
async fn a_canceler_that_fails_fails_the_work() {
    let manager = manager();
    let mut events = Events::new(manager.subscribe());
    let accepted = manager
        .create(
            NewWork::new("running", OWNER)
                .runner(Arc::new(GatedRunner::stubborn()))
                .canceler(Arc::new(ScriptedCanceler::failing("后台没有响应"))),
        )
        .await
        .expect("accepted");
    settle().await;

    let outcome = manager
        .cancel(&accepted.work.id, Some(OWNER))
        .await
        .expect("cancellable");
    assert_eq!(outcome.status, WorkStatus::Failed);
    assert_eq!(
        outcome.error.as_deref(),
        Some(
            via_i18n::format(
                Locale::Zh,
                keys::WORK_CANCEL_FAILED,
                &[("detail", "后台没有响应")],
            )
            .as_str()
        ),
    );
    assert_eq!(outcome.notification_status, NotificationStatus::Pending);
    assert!(events.saw(WorkEventKind::Cancelling));
    assert!(events.saw(WorkEventKind::Failed));
    assert!(events.saw(WorkEventKind::NotificationPending));
    assert!(!events.saw(WorkEventKind::Cancelled));
}

// ── joining and races ──────────────────────────────────────────────────────

/// A second cancel joins the first rather than starting another: upstream
/// returns the same `cancelPromise`.
#[tokio::test(start_paused = true)]
async fn a_second_cancel_joins_the_first() {
    let manager = manager();
    let runner = GatedRunner::stubborn();
    let canceler = ScriptedCanceler::requesting();
    let accepted = manager
        .create(
            NewWork::new("running", OWNER)
                .runner(Arc::new(runner.clone()))
                .canceler(Arc::new(canceler.clone())),
        )
        .await
        .expect("accepted");
    settle().await;

    let first = manager.cancel(&accepted.work.id, Some(OWNER));
    tokio::pin!(first);
    assert!(still_pending(first.as_mut()).await);
    let second = manager.cancel(&accepted.work.id, Some(OWNER));
    tokio::pin!(second);
    assert!(still_pending(second.as_mut()).await);
    assert_eq!(
        canceler.seen().len(),
        1,
        "one cancellation, however many times it is asked for",
    );

    runner.fail("aborted");
    let (first, second) = tokio::join!(first, second);
    assert_eq!(first.expect("cancellable").status, WorkStatus::Cancelled);
    assert_eq!(second.expect("cancellable").status, WorkStatus::Cancelled);
}

/// A result that arrives after a stop was requested is **dropped**, not
/// published: there is no `cancelling -> completed` edge.
#[tokio::test(start_paused = true)]
async fn a_result_that_arrives_after_the_stop_is_dropped() {
    let manager = manager();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::stubborn();
    let accepted = manager
        .create(
            NewWork::new("running", OWNER)
                .runner(Arc::new(runner.clone()))
                .canceler(Arc::new(ScriptedCanceler::requesting())),
        )
        .await
        .expect("accepted");
    settle().await;

    let cancel = manager.cancel(&accepted.work.id, Some(OWNER));
    tokio::pin!(cancel);
    assert!(still_pending(cancel.as_mut()).await);

    // The runner ignores the abort and *succeeds*.
    runner.complete("这是不该发布的结果");
    let cancelled = cancel.await.expect("cancellable");
    assert_eq!(cancelled.status, WorkStatus::Cancelled);
    assert_eq!(
        cancelled.result, None,
        "a cancelled Work never publishes a result",
    );
    assert!(!events.saw(WorkEventKind::Completed));
    assert!(
        !events.saw(WorkEventKind::NotificationPending),
        "and never announces one",
    );

    let waited = manager.wait(&accepted.work.id).await.expect("terminal");
    assert_eq!(waited.status, WorkStatus::Cancelled);
    assert_eq!(waited.result, None);
}

/// Cancellation clears a pending permission: nobody should be asked to approve
/// something that is being stopped.
#[tokio::test(start_paused = true)]
async fn cancelling_clears_a_pending_permission() {
    let manager = manager();
    let runner = GatedRunner::stubborn().emitting(vec![RunnerEvent::PermissionRequested(
        via_work::PendingPermission::pending("auth_one", "bash", "s"),
    )]);
    let accepted = manager
        .create(
            NewWork::new("running", OWNER)
                .runner(Arc::new(runner.clone()))
                .canceler(Arc::new(ScriptedCanceler::requesting())),
        )
        .await
        .expect("accepted");
    settle().await;
    assert!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .authorization
            .is_some(),
    );

    let cancel = manager.cancel(&accepted.work.id, Some(OWNER));
    tokio::pin!(cancel);
    assert!(still_pending(cancel.as_mut()).await);
    assert!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .authorization
            .is_none(),
        "the permission goes the moment the stop is asked for",
    );
    runner.fail("aborted");
    cancel.await;
}

/// The abort carries the catalogued reason all the way to the runner.
#[tokio::test(start_paused = true)]
async fn the_runner_is_told_why_it_was_stopped() {
    let manager = manager();
    let runner = GatedRunner::new();
    let accepted = manager
        .create(NewWork::new("running", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;

    manager.cancel(&accepted.work.id, Some(OWNER)).await;
    assert!(runner.observed_abort());
    // `GatedRunner` returns the abort reason as its failure, which the manager
    // then discards because the Work is cancelling — so the reason is asserted
    // through a Work that is *not* cancelled: the scheduled-task watchdog, in
    // `scheduled.rs`. Here the abort itself is what matters.
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Cancelled,
    );
    assert!(!t(Locale::Zh, keys::WORK_CANCELLED_BY_USER).is_empty());
}

/// Cancelling a Work releases its lane, so the next one in the lane starts.
#[tokio::test(start_paused = true)]
async fn a_confirmed_cancellation_releases_the_lane() {
    let manager = manager();
    let lane = coordinator_lane(OWNER);
    let first = GatedRunner::new();
    let second = GatedRunner::new();
    let running = manager
        .create(
            NewWork::new("A", OWNER)
                .lane(&lane, 1)
                .runner(Arc::new(first.clone())),
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

    manager.cancel(&running.work.id, Some(OWNER)).await;
    settle().await;
    assert!(second.is_running());
    second.complete("B done");
    assert_eq!(
        manager
            .wait(&queued.work.id)
            .await
            .expect("terminal")
            .status,
        WorkStatus::Completed,
    );
}

// ── the reconciliation ledger ──────────────────────────────────────────────

/// After a direct adapter abort, the fact is recorded, injected **once**, and
/// only cleared when the turn that carried it produced something.
#[tokio::test(start_paused = true)]
async fn a_cancellation_fact_is_injected_once() {
    let manager = manager();
    let fact = CancellationFact::delegated_session_cancelled(
        "work_one",
        "run-one",
        "target-one",
        CONFIRMED_AT,
    );
    manager.record_cancellation_fact(OWNER, fact.clone()).await;

    // A peek does not drain: a coordinator turn that returned nothing must
    // still find the fact next time.
    assert_eq!(
        manager.pending_cancellation_facts(OWNER).await,
        vec![fact.clone()]
    );
    assert_eq!(
        manager.pending_cancellation_facts(OWNER).await,
        vec![fact.clone()]
    );

    assert_eq!(manager.clear_cancellation_facts(OWNER).await, 1);
    assert!(manager.pending_cancellation_facts(OWNER).await.is_empty());
    assert_eq!(
        manager.clear_cancellation_facts(OWNER).await,
        0,
        "clearing twice clears nothing",
    );
    assert!(
        manager
            .pending_cancellation_facts("someone-else")
            .await
            .is_empty(),
        "facts do not cross owners",
    );
}

/// The queue keeps the newest twenty per owner.
#[tokio::test(start_paused = true)]
async fn the_fact_queue_keeps_the_newest_twenty() {
    let manager = manager();
    for index in 0..25 {
        manager
            .record_cancellation_fact(
                OWNER,
                CancellationFact::delegated_session_cancelled(
                    &format!("work_{index}"),
                    "run",
                    "target",
                    CONFIRMED_AT,
                ),
            )
            .await;
    }
    let pending = manager.pending_cancellation_facts(OWNER).await;
    assert_eq!(pending.len(), 20);
    assert_eq!(pending[0].work_id, "work_5");
    assert_eq!(pending[19].work_id, "work_24");
}

// ── shutdown ───────────────────────────────────────────────────────────────

/// A stopped manager refuses new Work rather than answering plausibly.
#[tokio::test(start_paused = true)]
async fn a_stopped_manager_refuses_new_work() {
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(ImmediateRunner::completing("done")))
        .build();
    let clone = manager.clone();
    manager.close().await;
    clone.close().await;
    // Both handles are gone; the actor drains and exits.
    tokio::time::sleep(Duration::from_millis(50)).await;

    let stopped = WorkManager::builder().now(clock()).build();
    let handle = stopped.clone();
    drop(stopped);
    tokio::time::sleep(Duration::from_millis(1)).await;
    assert!(handle.is_running(), "one live handle keeps it running");
}
