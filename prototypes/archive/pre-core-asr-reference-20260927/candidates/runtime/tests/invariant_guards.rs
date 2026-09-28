//! Correctness guards only. These tests deliberately do not measure performance.
use tempfile::NamedTempFile;
use via_contracts::{AgentObservation, ObservationKind, TaskCommand, TaskOp, TaskState};
use via_runtime::repository::Repository;

fn command(task: &str, id: &str, revision: u64, op: TaskOp) -> TaskCommand {
    TaskCommand {
        command_id: id.into(),
        task_id: task.into(),
        expected_revision: revision,
        op,
    }
}

async fn running(repo: &Repository) {
    repo.apply(command(
        "T1",
        "create",
        0,
        TaskOp::Create {
            goal: "demo".into(),
        },
    ))
    .await
    .unwrap();
    repo.apply(command(
        "T1",
        "accept",
        1,
        TaskOp::AcceptExecution {
            run_id: "R1".into(),
        },
    ))
    .await
    .unwrap();
}

fn observed(id: &str, revision: u64, source: u64, run: &str, kind: ObservationKind) -> TaskCommand {
    command(
        "T1",
        id,
        revision,
        TaskOp::ApplyObservation {
            observation: AgentObservation {
                run_id: run.into(),
                source_revision: source,
                kind,
            },
        },
    )
}

#[tokio::test]
async fn reused_command_id_cannot_cross_task_boundary() {
    let file = NamedTempFile::new().unwrap();
    let repo = Repository::open(file.path()).unwrap();
    repo.apply(command(
        "T1",
        "same-id",
        0,
        TaskOp::Create {
            goal: "first".into(),
        },
    ))
    .await
    .unwrap();
    let result = repo
        .apply(command(
            "T2",
            "same-id",
            0,
            TaskOp::Create {
                goal: "other".into(),
            },
        ))
        .await;
    assert!(
        result.is_err(),
        "dedup must bind identity to payload and Task"
    );
    assert!(repo.get("T2").await.is_err());
}

#[tokio::test]
async fn reused_command_id_cannot_change_goal() {
    let file = NamedTempFile::new().unwrap();
    let repo = Repository::open(file.path()).unwrap();
    repo.apply(command(
        "T1",
        "same-id",
        0,
        TaskOp::Create {
            goal: "first".into(),
        },
    ))
    .await
    .unwrap();
    assert!(
        repo.apply(command(
            "T1",
            "same-id",
            0,
            TaskOp::Create {
                goal: "other".into(),
            },
        ))
        .await
        .is_err()
    );
}

#[tokio::test]
async fn wrong_execution_event_is_rejected_without_mutation() {
    let file = NamedTempFile::new().unwrap();
    let repo = Repository::open(file.path()).unwrap();
    running(&repo).await;
    let before = repo.get("T1").await.unwrap();
    assert!(
        repo.apply(observed(
            "bad",
            2,
            8,
            "R2",
            ObservationKind::Result {
                artifact: "wrong".into(),
            },
        ))
        .await
        .is_err()
    );
    assert_eq!(repo.get("T1").await.unwrap(), before);
}

#[tokio::test]
async fn older_source_event_cannot_overwrite_newer_snapshot() {
    let file = NamedTempFile::new().unwrap();
    let repo = Repository::open(file.path()).unwrap();
    running(&repo).await;
    let newer = repo
        .apply(observed(
            "new",
            2,
            5,
            "R1",
            ObservationKind::Question {
                question_id: "Q1".into(),
                text: "confirm?".into(),
            },
        ))
        .await
        .unwrap();
    let result = repo
        .apply(observed(
            "old",
            newer.revision,
            4,
            "R1",
            ObservationKind::Progress { percent: 10 },
        ))
        .await;
    assert!(result.is_ok() || result.is_err());
    assert_eq!(
        repo.get("T1").await.unwrap(),
        newer,
        "stale source data must not change state or local revision"
    );
}

#[tokio::test]
async fn duplicated_source_revision_does_not_create_new_transition() {
    let file = NamedTempFile::new().unwrap();
    let repo = Repository::open(file.path()).unwrap();
    running(&repo).await;
    let first = repo
        .apply(observed(
            "evt-a",
            2,
            3,
            "R1",
            ObservationKind::Progress { percent: 40 },
        ))
        .await
        .unwrap();
    let second = repo
        .apply(observed(
            "evt-b",
            first.revision,
            3,
            "R1",
            ObservationKind::Progress { percent: 40 },
        ))
        .await
        .unwrap();
    assert_eq!(first, second);
}

#[tokio::test]
async fn cancel_request_survives_running_progress() {
    let file = NamedTempFile::new().unwrap();
    let repo = Repository::open(file.path()).unwrap();
    running(&repo).await;
    let cancel = repo
        .apply(command("T1", "cancel", 2, TaskOp::Cancel))
        .await
        .unwrap();
    let next = repo
        .apply(observed(
            "progress",
            cancel.revision,
            3,
            "R1",
            ObservationKind::Progress { percent: 50 },
        ))
        .await
        .unwrap();
    assert_eq!(next.state, TaskState::CancelRequested);
}

#[tokio::test]
async fn canceled_requested_is_not_canceled_confirmed() {
    let file = NamedTempFile::new().unwrap();
    let repo = Repository::open(file.path()).unwrap();
    running(&repo).await;
    let value = repo
        .apply(command("T1", "cancel", 2, TaskOp::Cancel))
        .await
        .unwrap();
    assert_eq!(value.state, TaskState::CancelRequested);
}

#[tokio::test]
async fn late_cancel_cannot_reopen_completed_task() {
    let file = NamedTempFile::new().unwrap();
    let repo = Repository::open(file.path()).unwrap();
    running(&repo).await;
    let done = repo
        .apply(observed(
            "done",
            2,
            5,
            "R1",
            ObservationKind::Result {
                artifact: "ART1".into(),
            },
        ))
        .await
        .unwrap();
    let _ = repo
        .apply(command("T1", "late-cancel", done.revision, TaskOp::Cancel))
        .await;
    assert_eq!(repo.get("T1").await.unwrap(), done);
}

#[tokio::test]
async fn terminal_state_cannot_roll_back_to_running() {
    let file = NamedTempFile::new().unwrap();
    let repo = Repository::open(file.path()).unwrap();
    running(&repo).await;
    let done = repo
        .apply(observed(
            "done",
            2,
            5,
            "R1",
            ObservationKind::Result {
                artifact: "ART1".into(),
            },
        ))
        .await
        .unwrap();
    let _ = repo
        .apply(observed(
            "bad-progress",
            done.revision,
            6,
            "R1",
            ObservationKind::Progress { percent: 100 },
        ))
        .await;
    assert_eq!(repo.get("T1").await.unwrap(), done);
}

#[tokio::test]
async fn execution_cannot_be_rebound_by_plain_accept() {
    let file = NamedTempFile::new().unwrap();
    let repo = Repository::open(file.path()).unwrap();
    running(&repo).await;
    assert!(
        repo.apply(command(
            "T1",
            "wrong-accept",
            2,
            TaskOp::AcceptExecution {
                run_id: "R2".into(),
            },
        ))
        .await
        .is_err()
    );
}

#[tokio::test]
async fn invalid_progress_is_not_accepted() {
    let file = NamedTempFile::new().unwrap();
    let repo = Repository::open(file.path()).unwrap();
    running(&repo).await;
    assert!(
        repo.apply(observed(
            "invalid",
            2,
            4,
            "R1",
            ObservationKind::Progress { percent: 101 },
        ))
        .await
        .is_err()
    );
}

#[tokio::test]
async fn stale_revision_and_reopen_preserve_state() {
    let file = NamedTempFile::new().unwrap();
    let repo = Repository::open(file.path()).unwrap();
    running(&repo).await;
    assert!(
        repo.apply(command("T1", "stale-cancel", 1, TaskOp::Cancel))
            .await
            .is_err()
    );
    let before = repo.get("T1").await.unwrap();
    drop(repo);
    let reopened = Repository::open(file.path()).unwrap();
    assert_eq!(reopened.get("T1").await.unwrap(), before);
}
