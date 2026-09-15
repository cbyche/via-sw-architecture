//! The lifecycle graph, as the manager walks it.
//!
//! `via_protocol::WorkStatus::can_transition_to` is the graph and is not
//! restated here; what is asserted is that the manager **only** ever walks it,
//! and that each of the three missing edges the graph documents is missing in
//! practice as well as on paper.

mod common;

use std::sync::Arc;

use common::{Events, clock, settle};
use pretty_assertions::assert_eq;
use via_protocol::WorkStatus;
use via_work::testing::{DeafRunner, GatedRunner, ImmediateRunner, ScriptedCanceler, delegation};
use via_work::{NewScheduledWork, NewWork, RunnerEvent, Schedule, WorkEventKind, WorkManager};

const OWNER: &str = "owner";

/// Every event name the manager can raise, mapped to the status the record is
/// in when it is raised. Walking a Work through its whole life and comparing
/// the observed pairs against the graph is the strongest statement available:
/// nothing the manager did was an edge the graph does not have.
async fn observed_pairs(events: &mut Events, work_id: &str) -> Vec<(WorkStatus, WorkStatus)> {
    let mut pairs = Vec::new();
    let mut previous: Option<WorkStatus> = None;
    for event in events.all() {
        if event.task.id != work_id {
            continue;
        }
        // Only the events that *carry* a status change are edges; the liveness
        // tick and the notification events repeat the status they found.
        let status = event.task.status;
        if let Some(from) = previous
            && from != status
        {
            pairs.push((from, status));
        }
        previous = Some(status);
    }
    pairs
}

/// The ordinary path: `queued -> running -> completed`, every step a legal
/// edge.
#[tokio::test(start_paused = true)]
async fn the_ordinary_path_walks_only_legal_edges() {
    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(ImmediateRunner::completing("done")))
        .build();
    let mut events = Events::new(manager.subscribe());
    let accepted = manager
        .create(NewWork::new("objective", OWNER))
        .await
        .expect("accepted");
    manager.wait(&accepted.work.id).await;
    settle().await;

    let pairs = observed_pairs(&mut events, &accepted.work.id).await;
    assert_eq!(
        pairs,
        vec![
            (WorkStatus::Queued, WorkStatus::Running),
            (WorkStatus::Running, WorkStatus::Completed),
        ],
    );
    for (from, to) in pairs {
        assert!(from.can_transition_to(to), "{from} -> {to}");
    }
}

/// The delegated path: `queued -> running -> delegated -> finalizing ->
/// completed`.
#[tokio::test(start_paused = true)]
async fn the_delegated_path_walks_only_legal_edges() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new();
    let accepted = manager
        .create(NewWork::new("委托", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;
    runner
        .emit(RunnerEvent::Delegated(delegation("run-one", "target-one")))
        .await;
    settle().await;
    runner
        .emit(RunnerEvent::DelegationCompleted(delegation(
            "run-one",
            "target-one",
        )))
        .await;
    settle().await;
    runner.complete("done");
    manager.wait(&accepted.work.id).await;
    settle().await;

    let pairs = observed_pairs(&mut events, &accepted.work.id).await;
    assert_eq!(
        pairs,
        vec![
            (WorkStatus::Queued, WorkStatus::Running),
            (WorkStatus::Running, WorkStatus::Delegated),
            (WorkStatus::Delegated, WorkStatus::Finalizing),
            (WorkStatus::Finalizing, WorkStatus::Completed),
        ],
    );
    for (from, to) in pairs {
        assert!(from.can_transition_to(to), "{from} -> {to}");
    }
}

/// The cancellation path: `running -> cancelling -> cancelled`, and never
/// `running -> cancelled` directly.
#[tokio::test(start_paused = true)]
async fn the_cancellation_path_always_passes_through_cancelling() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::stubborn();
    let accepted = manager
        .create(
            NewWork::new("cancel me", OWNER)
                .runner(Arc::new(runner.clone()))
                .canceler(Arc::new(ScriptedCanceler::requesting())),
        )
        .await
        .expect("accepted");
    settle().await;

    let cancel = manager.cancel(&accepted.work.id, Some(OWNER));
    tokio::pin!(cancel);
    tokio::select! {
        _ = &mut cancel => panic!("confirmed too early"),
        () = settle() => {}
    }
    runner.fail("aborted");
    cancel.await;
    settle().await;

    let pairs = observed_pairs(&mut events, &accepted.work.id).await;
    assert_eq!(
        pairs,
        vec![
            (WorkStatus::Queued, WorkStatus::Running),
            (WorkStatus::Running, WorkStatus::Cancelling),
            (WorkStatus::Cancelling, WorkStatus::Cancelled),
        ],
    );
    assert!(
        !WorkStatus::Running.can_transition_to(WorkStatus::Cancelled),
        "the graph has no shortcut, and neither does the manager",
    );
}

/// The scheduled path: `scheduled -> queued -> running -> completed`, and the
/// short-circuit `scheduled -> cancelled`.
#[tokio::test(start_paused = true)]
async fn the_scheduled_paths_walk_only_legal_edges() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let fired = manager
        .create_scheduled(NewScheduledWork::reminder(
            "fire",
            OWNER,
            Schedule::at(common::BASE_MS - 1),
        ))
        .await
        .expect("accepted");
    let cancelled = manager
        .create_scheduled(NewScheduledWork::reminder(
            "cancel",
            OWNER,
            Schedule::at(common::BASE_MS + 60_000),
        ))
        .await
        .expect("accepted");
    manager
        .fire_scheduled(std::slice::from_ref(&fired.work.id))
        .await;
    manager.wait(&fired.work.id).await;
    manager.cancel(&cancelled.work.id, Some(OWNER)).await;
    settle().await;

    assert_eq!(
        observed_pairs(&mut events, &fired.work.id).await,
        vec![
            (WorkStatus::Scheduled, WorkStatus::Queued),
            (WorkStatus::Queued, WorkStatus::Running),
            (WorkStatus::Running, WorkStatus::Completed),
        ],
    );
    assert_eq!(
        observed_pairs(&mut events, &cancelled.work.id).await,
        vec![(WorkStatus::Scheduled, WorkStatus::Cancelled)],
    );
}

/// The three edges the graph deliberately omits, refused in practice.
#[tokio::test(start_paused = true)]
async fn the_three_missing_edges_are_missing_in_practice_too() {
    // 1. `running -> finalizing` — finalizing is only reachable through
    //    `delegated`, which `delegation.rs` proves by dropping the event.
    assert!(!WorkStatus::Running.can_transition_to(WorkStatus::Finalizing));

    // 2. `cancelling -> completed` — a result after a stop is dropped.
    assert!(!WorkStatus::Cancelling.can_transition_to(WorkStatus::Completed));
    let manager = WorkManager::builder().now(clock()).build();
    let runner = GatedRunner::stubborn();
    let accepted = manager
        .create(
            NewWork::new("cancel me", OWNER)
                .runner(Arc::new(runner.clone()))
                .canceler(Arc::new(ScriptedCanceler::requesting())),
        )
        .await
        .expect("accepted");
    settle().await;
    let cancel = manager.cancel(&accepted.work.id, Some(OWNER));
    tokio::pin!(cancel);
    tokio::select! {
        _ = &mut cancel => panic!("confirmed too early"),
        () = settle() => {}
    }
    runner.complete("a result nobody asked for any more");
    let cancelled = cancel.await.expect("cancellable");
    assert_eq!(cancelled.status, WorkStatus::Cancelled);
    assert_eq!(cancelled.result, None);

    // 3. `queued -> cancelling` — nothing has started, so nothing needs
    //    confirming.
    assert!(!WorkStatus::Queued.can_transition_to(WorkStatus::Cancelling));
    assert!(!WorkStatus::Scheduled.can_transition_to(WorkStatus::Cancelling));
}

/// A terminal Work is terminal: nothing the manager can be told moves it.
#[tokio::test(start_paused = true)]
async fn nothing_moves_a_terminal_work() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new();
    let accepted = manager
        .create(NewWork::new("done", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;
    runner.complete("done");
    manager.wait(&accepted.work.id).await;
    settle().await;
    let before = events.all().len();

    for event in [
        RunnerEvent::Delegated(delegation("run-one", "target-one")),
        RunnerEvent::DelegationCompleted(delegation("run-one", "target-one")),
        RunnerEvent::PermissionRequested(via_work::PendingPermission::pending("a", "b", "c")),
    ] {
        runner.emit(event).await;
        settle().await;
    }
    assert_eq!(events.all().len(), before, "not one further event");

    let work = manager.get(&accepted.work.id, None).await.expect("exists");
    assert_eq!(work.status, WorkStatus::Completed);
    assert!(work.delegation.is_none());
    assert!(work.authorization.is_none());
    assert!(
        manager
            .cancel(&accepted.work.id, Some(OWNER))
            .await
            .is_none(),
        "and it cannot be cancelled",
    );
    for status in WorkStatus::TERMINAL {
        assert!(status.is_terminal());
        for next in WorkStatus::ALL {
            assert!(
                !status.can_transition_to(*next),
                "{status} is terminal but claims an edge to {next}",
            );
        }
    }
}

/// A Work that is never confirmed stopped stays exactly where it is: the graph
/// has an edge out of `cancelling`, and the manager refuses to take it without
/// evidence.
#[tokio::test(start_paused = true)]
async fn cancelling_is_a_state_that_can_be_stayed_in() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let accepted = manager
        .create(NewWork::new("deaf", OWNER).runner(Arc::new(DeafRunner)))
        .await
        .expect("accepted");
    settle().await;
    let cancel = manager.cancel(&accepted.work.id, Some(OWNER));
    tokio::pin!(cancel);
    tokio::select! {
        _ = &mut cancel => panic!("nothing confirmed anything"),
        () = settle() => {}
    }

    assert_eq!(
        observed_pairs(&mut events, &accepted.work.id).await,
        vec![
            (WorkStatus::Queued, WorkStatus::Running),
            (WorkStatus::Running, WorkStatus::Cancelling),
        ],
    );
    assert!(events.saw(WorkEventKind::Cancelling));
    assert!(!events.saw(WorkEventKind::Cancelled));
    assert!(WorkStatus::Cancelling.can_transition_to(WorkStatus::Cancelled));
    assert!(WorkStatus::Cancelling.can_transition_to(WorkStatus::Failed));
}
