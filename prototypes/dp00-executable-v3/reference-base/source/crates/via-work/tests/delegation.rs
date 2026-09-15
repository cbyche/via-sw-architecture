//! Delegation correlation — `docs/architecture.md` §11, invariant 4.
//!
//! > Only the completion correlated to that delegation id may complete the
//! > Work. The request timeout applies to the coordinator turn and the
//! > presentation turn — **not** while waiting on the delegated session.
//!
//! Four things must not complete a Work: a busy target, an empty result, an
//! unrelated session update, and a stale result. Each has a test here.

mod common;

use std::sync::Arc;

use common::{Events, advance, clock, settle};
use pretty_assertions::assert_eq;
use via_downstream::testing::{ScriptedHarness, ScriptedTurn};
use via_downstream::{
    ActivityTracker, HarnessRegistry, PromptRequest, RawSessionUpdate, SessionKey,
};
use via_protocol::{WorkState, WorkStatus};
use via_work::testing::{GatedRunner, delegation};
use via_work::{
    DelegationRef, NewWork, RunFailure, RunnerEvent, WorkEventKind, WorkManager, WorkOutcome,
};

const OWNER: &str = "owner";

async fn delegated_work(
    manager: &WorkManager,
    runner: &GatedRunner,
    id: &str,
    session: &str,
) -> String {
    let accepted = manager
        .create(NewWork::new("委托", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;
    runner
        .emit(RunnerEvent::Delegated(delegation(id, session)))
        .await;
    settle().await;
    assert_eq!(
        manager
            .get(&accepted.work.id, None)
            .await
            .expect("exists")
            .status,
        WorkStatus::Delegated,
    );
    accepted.work.id
}

/// The happy path: the correlated completion moves the Work to `finalizing`.
#[tokio::test(start_paused = true)]
async fn the_correlated_completion_is_the_one_that_finalizes() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new();
    let work_id = delegated_work(&manager, &runner, "run-one", "target-one").await;

    runner
        .emit(RunnerEvent::DelegationCompleted(delegation(
            "run-one",
            "target-one",
        )))
        .await;
    settle().await;

    let work = manager.get(&work_id, None).await.expect("exists");
    assert_eq!(work.status, WorkStatus::Finalizing);
    assert_eq!(work.work_state, WorkState::Active);
    assert_eq!(
        work.delegation.expect("a delegation").status,
        "completed",
        "the delegation is stamped completed",
    );
    assert!(events.saw(WorkEventKind::Finalizing));

    runner.complete("最终结果");
    let finished = manager.wait(&work_id).await.expect("terminal");
    assert_eq!(finished.status, WorkStatus::Completed);
    assert_eq!(finished.result.as_deref(), Some("最终结果"));
}

/// **A stale result must not complete the Work.** A completion carrying a
/// different delegation id is dropped.
#[tokio::test(start_paused = true)]
async fn a_completion_for_a_different_delegation_is_dropped() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new();
    let work_id = delegated_work(&manager, &runner, "run-one", "target-one").await;

    for stale in [
        delegation("run-two", "target-one"),
        delegation("run-one", "target-two"),
        delegation("", "target-one"),
    ] {
        runner.emit(RunnerEvent::DelegationCompleted(stale)).await;
        settle().await;
        assert_eq!(
            manager.get(&work_id, None).await.expect("exists").status,
            WorkStatus::Delegated,
            "an uncorrelated completion changes nothing",
        );
    }
    assert!(!events.saw(WorkEventKind::Finalizing));

    runner.complete("done");
    manager.wait(&work_id).await;
}

/// **An unrelated session update must not complete the Work.** Activity moves
/// the ring and nothing else.
#[tokio::test(start_paused = true)]
async fn an_unrelated_session_update_only_moves_the_activity_ring() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new();
    let work_id = delegated_work(&manager, &runner, "run-one", "target-one").await;

    let mut tracker = ActivityTracker::new();
    for name in ["bash", "read", "write"] {
        let event = tracker
            .project(&RawSessionUpdate {
                name: Some(name.to_owned()),
                ..RawSessionUpdate::tool_call(name)
            })
            .expect("projects");
        runner.emit(RunnerEvent::Activity(event)).await;
    }
    settle().await;

    let work = manager.get(&work_id, None).await.expect("exists");
    assert_eq!(
        work.status,
        WorkStatus::Delegated,
        "activity is not a result"
    );
    assert_eq!(work.activity.len(), 3);
    assert!(work.result.is_none());
    assert!(!events.saw(WorkEventKind::Finalizing));
    assert!(!events.saw(WorkEventKind::Completed));

    runner.complete("done");
    manager.wait(&work_id).await;
}

/// **A busy target must not complete the Work.** A session with a turn already
/// in flight answers `HarnessError::SessionBusy`, and a refusal is not a
/// result.
#[tokio::test(start_paused = true)]
async fn a_busy_target_fails_the_work_rather_than_completing_it() {
    let harness = Arc::new(
        ScriptedHarness::builder("codex")
            .turn(ScriptedTurn::completed("never reached"))
            .build()
            .expect("a valid descriptor"),
    );
    let mut registry = HarnessRegistry::new();
    registry
        .register(Arc::clone(&harness) as Arc<dyn via_downstream::DownstreamAgent>)
        .expect("registered");
    let session = Arc::new(
        registry
            .open(&SessionKey::coordinator("codex", OWNER))
            .await
            .expect("opened"),
    );
    // Pin the session as mid-turn: this is the race a second prompt loses.
    let scripted = harness.sessions().into_iter().next().expect("one session");
    scripted.hold();
    assert!(scripted.is_busy());

    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(
            move |_objective: String, _context: via_work::WorkContext| {
                let session = Arc::clone(&session);
                async move {
                    session
                        .prompt(PromptRequest::text(OWNER, "two"))
                        .await
                        .map(|outcome| WorkOutcome::content(outcome.content()))
                        .map_err(RunFailure::from)
                }
            },
        ))
        .build();

    let accepted = manager
        .create(NewWork::new("委托", OWNER))
        .await
        .expect("accepted");
    let finished = manager.wait(&accepted.work.id).await.expect("terminal");
    assert_eq!(
        finished.status,
        WorkStatus::Failed,
        "a busy target is a failure, never a result",
    );
    assert!(finished.result.is_none());
    assert!(finished.error.is_some_and(|error| !error.is_empty()));
    assert_eq!(
        harness.remaining_turns(),
        1,
        "the scripted turn was never consumed",
    );
}

/// **An empty result must not complete the Work with nothing.** The harness
/// double refuses when its script runs out, exactly as the seam does, and the
/// manager records the refusal rather than a blank success.
#[tokio::test(start_paused = true)]
async fn an_exhausted_target_fails_rather_than_completing_empty() {
    let harness = ScriptedHarness::builder("codex")
        .build()
        .expect("a valid descriptor");
    let mut registry = HarnessRegistry::new();
    registry.register(Arc::new(harness)).expect("registered");
    let session = Arc::new(
        registry
            .open(&SessionKey::coordinator("codex", OWNER))
            .await
            .expect("opened"),
    );

    let manager = WorkManager::builder()
        .now(clock())
        .runner(Arc::new(
            move |_objective: String, _context: via_work::WorkContext| {
                let session = Arc::clone(&session);
                async move {
                    session
                        .prompt(PromptRequest::text(OWNER, "anything"))
                        .await
                        .map(|outcome| WorkOutcome::content(outcome.content()))
                        .map_err(RunFailure::from)
                }
            },
        ))
        .build();

    let accepted = manager
        .create(NewWork::new("委托", OWNER))
        .await
        .expect("accepted");
    let finished = manager.wait(&accepted.work.id).await.expect("terminal");
    assert_eq!(finished.status, WorkStatus::Failed);
    assert!(
        finished
            .error
            .as_deref()
            .is_some_and(|error| !error.is_empty()),
        "the refusal carries a sentence",
    );
}

/// A completion that arrives while the Work is **not** delegated is dropped:
/// `finalizing` means a *delegated* session reported completion, so it is only
/// reachable through `delegated`.
#[tokio::test(start_paused = true)]
async fn a_completion_before_the_delegation_is_dropped() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new();
    let accepted = manager
        .create(NewWork::new("委托", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;

    runner
        .emit(RunnerEvent::DelegationCompleted(delegation(
            "run-one",
            "target-one",
        )))
        .await;
    settle().await;

    let work = manager.get(&accepted.work.id, None).await.expect("exists");
    assert_eq!(work.status, WorkStatus::Running);
    assert!(work.delegation.is_none());
    assert!(!events.saw(WorkEventKind::Finalizing));

    runner.complete("done");
    manager.wait(&accepted.work.id).await;
}

/// A second completion for the same delegation, after `finalizing`, is
/// dropped: there is no `finalizing -> finalizing` edge.
#[tokio::test(start_paused = true)]
async fn a_repeated_completion_does_not_re_finalize() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new();
    let work_id = delegated_work(&manager, &runner, "run-one", "target-one").await;

    for _ in 0..3 {
        runner
            .emit(RunnerEvent::DelegationCompleted(delegation(
                "run-one",
                "target-one",
            )))
            .await;
        settle().await;
    }
    assert_eq!(
        events.count(WorkEventKind::Finalizing),
        1,
        "one delegation, one finalizing",
    );

    runner.complete("done");
    manager.wait(&work_id).await;
}

/// A second `backend.delegated` naming a *different* target is refused: a Work
/// waits on exactly one delegation.
#[tokio::test(start_paused = true)]
async fn a_work_is_never_re_delegated_to_a_second_target() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new();
    let work_id = delegated_work(&manager, &runner, "run-one", "target-one").await;

    runner
        .emit(RunnerEvent::Delegated(delegation("run-two", "target-two")))
        .await;
    settle().await;
    let work = manager.get(&work_id, None).await.expect("exists");
    let published = work.delegation.expect("a delegation");
    assert_eq!(published.title, "project");
    assert_eq!(
        events.count(WorkEventKind::Delegated),
        1,
        "the second delegation was refused",
    );

    // But the *same* delegation, re-announced, is accepted — which is what a
    // recovered run does when it reattaches.
    runner
        .emit(RunnerEvent::DelegationCompleted(delegation(
            "run-one",
            "target-one",
        )))
        .await;
    settle().await;
    assert_eq!(
        manager.get(&work_id, None).await.expect("exists").status,
        WorkStatus::Finalizing,
    );

    runner.complete("done");
    manager.wait(&work_id).await;
}

/// **The request timeout does not run while the Work waits on the delegated
/// session.** Nothing in this crate can fail a `delegated` Work of kind `work`
/// on a wall clock: the only watchdog belongs to `scheduled_task`.
#[tokio::test(start_paused = true)]
async fn a_delegated_work_is_never_timed_out_by_the_work_subsystem() {
    let manager = WorkManager::builder()
        .now(clock())
        .progress_check_ms(60_000)
        .event_capacity(65_536)
        .build();
    let runner = GatedRunner::stubborn();
    let work_id = delegated_work(&manager, &runner, "run-one", "target-one").await;

    // An hour parked on the delegated session.
    advance(60 * 60 * 1_000).await;
    let work = manager.get(&work_id, None).await.expect("exists");
    assert_eq!(work.status, WorkStatus::Delegated);
    assert_eq!(work.work_state, WorkState::Active);
    assert!(work.error.is_none());
    assert!(!runner.observed_abort(), "nothing aborted it");
    assert_eq!(work.timeout_ms, None, "ordinary work has no budget");

    runner.settle(Ok(WorkOutcome::content("终于完成")));
    let finished = manager.wait(&work_id).await.expect("terminal");
    assert_eq!(finished.status, WorkStatus::Completed);
    assert_eq!(finished.result.as_deref(), Some("终于完成"));
}

/// The delegation's ids never reach a client, however the Work ends.
#[tokio::test(start_paused = true)]
async fn a_delegations_ids_never_reach_a_client() {
    let manager = WorkManager::builder().now(clock()).build();
    let mut events = Events::new(manager.subscribe());
    let runner = GatedRunner::new();
    let secret = DelegationRef::new("run-secret", "agent:child:secret")
        .with_directory("/private/customer")
        .with_title("项目");

    let accepted = manager
        .create(NewWork::new("委托", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;
    runner.emit(RunnerEvent::Delegated(secret)).await;
    settle().await;
    runner.complete("done");
    manager.wait(&accepted.work.id).await;
    settle().await;

    for event in events.all() {
        let rendered = serde_json::to_string(&event).expect("serializes");
        assert!(!rendered.contains("run-secret"), "{rendered}");
        assert!(!rendered.contains("agent:child:secret"), "{rendered}");
        assert!(!rendered.contains("/private/customer"), "{rendered}");
    }
}
