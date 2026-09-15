//! Persistence and restart recovery, ported from
//! `server/test/task-store.test.mjs` and the restore half of
//! `server/test/task-manager-scheduled.test.mjs`.
//!
//! The property under test is the one a user notices: **work that a crash
//! interrupted is never silently lost.** It either comes back, or it fails with
//! a sentence saying why.

mod common;

use std::sync::Arc;

use async_trait::async_trait;
use common::{Events, clock, read_tasks, seed_tasks, settle};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use via_i18n::{Locale, keys, t};
use via_protocol::WorkStatus;
use via_work::testing::{GatedRunner, ImmediateRunner};
use via_work::{
    CancelRequest, DelegatedWorkRecovery, NewWork, NotificationClaim, NotificationStatus,
    RunFailure, RunnerEvent, WorkContext, WorkEventKind, WorkManager, WorkOutcome, WorkQuery,
    WorkSnapshot, WorkStore,
};

const OWNER: &str = "owner";
const SESSION: &str = "voice";

fn store(path: &std::path::Path) -> WorkStore {
    WorkStore::builder()
        .file_path(path)
        .locale(Locale::Zh)
        .self_scheduling(true)
        .build()
}

fn persisted(overrides: Value) -> Value {
    let mut record = json!({
        "id": "work-one",
        "status": "running",
        "kind": "work",
        "parentWorkId": null,
        "objective": "未完成",
        "ownerId": OWNER,
        "sessionId": SESSION,
        "turnId": "turn-1",
        "createdAt": 1_700_000_000_000_i64,
        "startedAt": 1_700_000_000_000_i64,
        "completedAt": null,
        "elapsedMs": 0,
        "result": null,
        "error": null,
        "resultMetadata": null,
        "activity": [],
        "delegation": null,
        "authorization": null,
        "notificationStatus": "none",
        "schedule": null,
        "timeoutMs": null,
        "progressCheckMs": null,
        "submissionKey": null,
    });
    if let (Some(base), Some(extra)) = (record.as_object_mut(), overrides.as_object()) {
        for (key, value) in extra {
            base.insert(key.clone(), value.clone());
        }
    }
    record
}

// ── what survives ──────────────────────────────────────────────────────────

/// Upstream: *persists final work and notification delivery state*.
#[tokio::test(start_paused = true)]
async fn a_completed_result_and_its_delivery_state_survive_a_restart() {
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("tasks.json");

    let first = WorkManager::builder()
        .now(clock())
        .store(store(&path))
        .runner(Arc::new(ImmediateRunner::completing("完成")))
        .build();
    let accepted = first
        .create(NewWork::new("保存结果", OWNER).session(SESSION))
        .await
        .expect("accepted");
    first.wait(&accepted.work.id).await;
    let claimed = first
        .claim_notifications(NotificationClaim::new(OWNER, SESSION, "client"))
        .await;
    first
        .mark_notifications_delivered(&[claimed[0].id.clone()], Some("client"))
        .await;
    first.close().await;

    let restored = WorkManager::builder()
        .now(clock())
        .store(store(&path))
        .build();
    let work = restored
        .get(&accepted.work.id, None)
        .await
        .expect("restored");
    assert_eq!(work.result.as_deref(), Some("完成"));
    assert_eq!(work.notification_status, NotificationStatus::Delivered);
    assert!(work.notification_delivered_at.is_some());
}

// ── what is force-failed ───────────────────────────────────────────────────

/// Upstream: *marks interrupted queued or running work as failed after
/// restart*.
#[tokio::test(start_paused = true)]
async fn interrupted_work_fails_with_the_interactive_restart_sentence() {
    for status in ["queued", "running", "cancelling"] {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = seed_tasks(
            directory.path(),
            json!([persisted(json!({ "status": status }))]),
        );
        let manager = WorkManager::builder()
            .now(clock())
            .locale(Locale::Zh)
            .store(store(&path))
            .build();

        let work = manager.get("work-one", None).await.expect("restored");
        assert_eq!(work.status, WorkStatus::Failed, "{status}");
        assert_eq!(
            work.error.as_deref(),
            Some(t(Locale::Zh, keys::WORK_RESTART_INTERACTIVE_INCOMPLETE)),
            "{status}",
        );
        assert!(
            work.error
                .as_deref()
                .is_some_and(|error| error.contains("重启"))
        );
        assert_eq!(
            work.notification_status,
            NotificationStatus::Pending,
            "{status}"
        );
        assert!(work.completed_at.is_some(), "{status}");
    }
}

/// A `delivering` notification is put back to `pending`: the client that held
/// the lease is gone, so nobody is going to speak it.
#[tokio::test(start_paused = true)]
async fn an_interrupted_delivery_lease_becomes_pending_again() {
    let directory = tempfile::tempdir().expect("tempdir");
    let path = seed_tasks(
        directory.path(),
        json!([persisted(json!({
            "status": "completed",
            "notificationStatus": "delivering",
            "completedAt": 1_700_000_000_000_i64,
            "result": "结果",
        }))]),
    );
    let manager = WorkManager::builder()
        .now(clock())
        .store(store(&path))
        .build();
    let work = manager.get("work-one", None).await.expect("restored");
    assert_eq!(work.status, WorkStatus::Completed);
    assert_eq!(work.notification_status, NotificationStatus::Pending);
}

/// Upstream: *drops stale permissions restored on terminal work*.
#[tokio::test(start_paused = true)]
async fn a_stale_permission_never_survives_a_restart() {
    let directory = tempfile::tempdir().expect("tempdir");
    let authorization = json!({
        "id": "auth-stale",
        "status": "pending",
        "category": "bash",
        "summary": "List directory",
    });
    let path = seed_tasks(
        directory.path(),
        json!([
            persisted(json!({
                "id": "work-complete",
                "status": "completed",
                "completedAt": 1_700_000_000_000_i64,
                "authorization": authorization.clone(),
            })),
            persisted(json!({
                "id": "work-active",
                "status": "running",
                "authorization": authorization,
            })),
        ]),
    );
    let manager = WorkManager::builder()
        .now(clock())
        .store(store(&path))
        .build();
    for id in ["work-complete", "work-active"] {
        assert!(
            manager
                .get(id, None)
                .await
                .expect("restored")
                .authorization
                .is_none(),
            "{id}",
        );
    }
}

// ── reminders ──────────────────────────────────────────────────────────────

/// Upstream: *restore re-schedules queued and running reminders for catch-up*.
#[tokio::test(start_paused = true)]
async fn a_reminder_that_had_already_fired_is_replayed_as_scheduled() {
    for status in ["queued", "running"] {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = seed_tasks(
            directory.path(),
            json!([persisted(json!({
                "id": "work-reminder",
                "kind": "reminder",
                "status": status,
                "objective": "已到点的提醒",
                "schedule": {"type": "at", "at": 1_699_999_940_000_i64, "recurrence": "once"},
            }))]),
        );
        let manager = WorkManager::builder()
            .now(clock())
            .store(store(&path))
            .build();

        let work = manager.get("work-reminder", None).await.expect("restored");
        assert_eq!(work.status, WorkStatus::Scheduled, "{status}");
        assert_eq!(work.error, None, "{status}");

        // Its runner was rebuilt, so firing it speaks the stored text back.
        manager.fire_scheduled(&["work-reminder".to_owned()]).await;
        let fired = manager.wait("work-reminder").await.expect("terminal");
        assert_eq!(fired.status, WorkStatus::Completed, "{status}");
        assert_eq!(fired.result.as_deref(), Some("已到点的提醒"), "{status}");
        assert_eq!(
            fired
                .result_metadata
                .expect("a presentation")
                .presentation
                .speech,
            "已到点的提醒",
            "{status}",
        );
    }
}

/// Upstream: *restore never re-schedules a reminder whose cancellation had
/// started*.
#[tokio::test(start_paused = true)]
async fn a_cancelling_reminder_is_not_replayed() {
    let directory = tempfile::tempdir().expect("tempdir");
    let path = seed_tasks(
        directory.path(),
        json!([persisted(json!({
            "id": "work-reminder",
            "kind": "reminder",
            "status": "cancelling",
        }))]),
    );
    let manager = WorkManager::builder()
        .now(clock())
        .locale(Locale::Zh)
        .store(store(&path))
        .build();
    let work = manager.get("work-reminder", None).await.expect("restored");
    assert_ne!(work.status, WorkStatus::Scheduled);
    assert_eq!(work.status, WorkStatus::Failed);
}

/// Upstream: *restore recovers scheduled tasks with reminder runner rebuilt*
/// and *restored scheduled_task receives its complete persisted execution
/// context*.
#[tokio::test(start_paused = true)]
async fn a_restored_scheduled_task_gets_the_configured_runner_and_its_context() {
    let directory = tempfile::tempdir().expect("tempdir");
    let schedule = json!({"type": "at", "at": 1_700_000_060_000_i64, "recurrence": "once"});
    let path = seed_tasks(
        directory.path(),
        json!([persisted(json!({
            "id": "work_restored_task",
            "kind": "scheduled_task",
            "status": "scheduled",
            "objective": "恢复的定时任务",
            "startedAt": null,
            "schedule": schedule.clone(),
            "timeoutMs": 1_800_000,
        }))]),
    );
    let manager = WorkManager::builder()
        .now(clock())
        .store(store(&path))
        .build();

    let seen: Arc<std::sync::Mutex<Option<WorkContext>>> = Arc::new(std::sync::Mutex::new(None));
    let captured = Arc::clone(&seen);
    manager
        .configure_scheduled_task_runner(Arc::new(
            move |objective: String, context: WorkContext| {
                let captured = Arc::clone(&captured);
                async move {
                    *captured.lock().unwrap_or_else(|poison| poison.into_inner()) = Some(context);
                    Ok(WorkOutcome::content(&objective))
                }
            },
        ))
        .await;

    let work = manager
        .get("work_restored_task", None)
        .await
        .expect("restored");
    assert_eq!(work.status, WorkStatus::Scheduled);
    assert_eq!(work.timeout_ms, Some(1_800_000));
    assert_eq!(
        work.progress_check_ms, None,
        "a scheduled task stays quiet while it runs",
    );

    manager
        .fire_scheduled(&["work_restored_task".to_owned()])
        .await;
    let completed = manager.wait("work_restored_task").await.expect("terminal");
    assert_eq!(completed.status, WorkStatus::Completed);

    let context = seen
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .clone()
        .expect("the runner ran");
    assert_eq!(context.work_id, "work_restored_task");
    assert_eq!(context.owner_id, OWNER);
    assert_eq!(context.session_id, SESSION);
    assert_eq!(context.turn_id.as_deref(), Some("turn-1"));
    assert_eq!(context.kind, via_protocol::WorkKind::ScheduledTask);
    assert_eq!(
        serde_json::to_value(context.schedule).expect("serializes"),
        schedule,
    );
    assert!(!context.signal.is_aborted());
}

// ── delegated recovery ─────────────────────────────────────────────────────

struct Recovery {
    accept: bool,
    result: String,
}

#[async_trait]
impl DelegatedWorkRecovery for Recovery {
    fn can_recover(&self, work: &WorkSnapshot) -> bool {
        self.accept && work.is_recoverable()
    }

    async fn run(
        &self,
        work: WorkSnapshot,
        context: WorkContext,
    ) -> Result<WorkOutcome, RunFailure> {
        let delegation = work.delegation.clone().expect("a delegation");
        context
            .events
            .emit(RunnerEvent::Delegated(delegation.clone()))
            .await;
        context
            .events
            .emit(RunnerEvent::DelegationCompleted(delegation))
            .await;
        Ok(WorkOutcome::content(&self.result))
    }

    async fn cancel(
        &self,
        _work: WorkSnapshot,
        request: CancelRequest,
    ) -> Result<via_downstream::CancelOutcome, RunFailure> {
        request.abort();
        Ok(via_downstream::CancelOutcome::requested(
            via_downstream::CancelRoute::Adapter,
            via_downstream::CancelTarget::default(),
        ))
    }
}

/// Upstream: *reattaches a persisted delegated run when its adapter supports
/// recovery*.
#[tokio::test(start_paused = true)]
async fn a_persisted_delegation_is_reattached_and_its_ids_survive() {
    let directory = tempfile::tempdir().expect("tempdir");
    let delegation = json!({
        "id": "run-one",
        "sessionId": "agent:child:one",
        "directory": "/project",
        "title": "项目任务",
    });
    let path = seed_tasks(
        directory.path(),
        json!([persisted(json!({
            "id": "work-delegated",
            "status": "delegated",
            "objective": "继续项目",
            "delegation": delegation,
        }))]),
    );
    let manager = WorkManager::builder()
        .now(clock())
        .store(store(&path))
        .build();
    let mut events = Events::new(manager.subscribe());

    // Before recovery runs, the Work is queued and holds its delegation.
    let queued = manager.get("work-delegated", None).await.expect("restored");
    assert_eq!(queued.status, WorkStatus::Queued);
    assert_eq!(queued.notification_status, NotificationStatus::None);

    let recovered = manager
        .recover_delegated(Arc::new(Recovery {
            accept: true,
            result: "恢复后的结果".to_owned(),
        }))
        .await;
    assert_eq!(recovered, 1);

    let finished = manager.wait("work-delegated").await.expect("terminal");
    assert_eq!(finished.status, WorkStatus::Completed);
    assert_eq!(finished.result.as_deref(), Some("恢复后的结果"));
    assert_eq!(
        events.kinds_for("work-delegated"),
        vec![
            WorkEventKind::Running,
            WorkEventKind::Delegated,
            WorkEventKind::Finalizing,
            WorkEventKind::Completed,
            WorkEventKind::NotificationPending,
        ],
    );

    manager.persist().await;
    let saved = read_tasks(&path);
    assert_eq!(saved[0]["delegation"]["id"], json!("run-one"));
    assert_eq!(
        saved[0]["delegation"]["sessionId"],
        json!("agent:child:one")
    );
    assert_eq!(saved[0]["delegation"]["status"], json!("completed"));
}

/// Upstream: the `canRecover() === false` branch — the *second* restart
/// sentence, which is the one that names a lost connection.
#[tokio::test(start_paused = true)]
async fn an_unrecoverable_delegation_fails_with_the_other_restart_sentence() {
    let directory = tempfile::tempdir().expect("tempdir");
    let path = seed_tasks(
        directory.path(),
        json!([persisted(json!({
            "id": "work-delegated",
            "status": "finalizing",
            "delegation": {"id": "run-one", "sessionId": "agent:child:one"},
        }))]),
    );
    let manager = WorkManager::builder()
        .now(clock())
        .locale(Locale::Zh)
        .store(store(&path))
        .build();
    let mut events = Events::new(manager.subscribe());

    let recovered = manager
        .recover_delegated(Arc::new(Recovery {
            accept: false,
            result: String::new(),
        }))
        .await;
    assert_eq!(recovered, 0);

    let work = manager.get("work-delegated", None).await.expect("restored");
    assert_eq!(work.status, WorkStatus::Failed);
    assert_eq!(
        work.error.as_deref(),
        Some(t(Locale::Zh, keys::WORK_RESTART_DELEGATED_LOST)),
    );
    assert_ne!(
        work.error.as_deref(),
        Some(t(Locale::Zh, keys::WORK_RESTART_INTERACTIVE_INCOMPLETE)),
        "the two restart reasons are not interchangeable",
    );
    assert_eq!(work.notification_status, NotificationStatus::Pending);
    assert!(events.saw(WorkEventKind::Failed));
    assert!(events.saw(WorkEventKind::NotificationPending));

    // The waiter resolves rather than hanging.
    assert_eq!(
        manager
            .wait("work-delegated")
            .await
            .expect("terminal")
            .status,
        WorkStatus::Failed,
    );
}

/// A delegation missing either id is not addressable, so it never becomes a
/// recovery candidate — it is force-failed at restore with the *interactive*
/// sentence.
#[tokio::test(start_paused = true)]
async fn a_delegation_missing_an_id_is_not_a_recovery_candidate() {
    for delegation in [
        json!({"id": "run-one", "sessionId": ""}),
        json!({"id": "", "sessionId": "agent:child:one"}),
    ] {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = seed_tasks(
            directory.path(),
            json!([persisted(json!({
                "id": "work-delegated",
                "status": "delegated",
                "delegation": delegation.clone(),
            }))]),
        );
        let manager = WorkManager::builder()
            .now(clock())
            .locale(Locale::Zh)
            .store(store(&path))
            .build();

        let work = manager.get("work-delegated", None).await.expect("restored");
        assert_eq!(work.status, WorkStatus::Failed, "{delegation}");
        assert_eq!(
            work.error.as_deref(),
            Some(t(Locale::Zh, keys::WORK_RESTART_INTERACTIVE_INCOMPLETE)),
            "{delegation}",
        );
        assert!(
            work.delegation.is_none(),
            "an unrecoverable delegation is dropped from the record",
        );
        assert_eq!(
            manager
                .recover_delegated(Arc::new(Recovery {
                    accept: true,
                    result: String::new(),
                }))
                .await,
            0,
        );
    }
}

// ── the legacy metadata shape ──────────────────────────────────────────────

/// Upstream: *projects legacy decision presentation when restoring a task*.
#[tokio::test(start_paused = true)]
async fn the_legacy_metadata_shape_is_projected_forward_and_re_persisted() {
    let directory = tempfile::tempdir().expect("tempdir");
    let path = seed_tasks(
        directory.path(),
        json!([persisted(json!({
            "id": "legacy-work",
            "status": "completed",
            "objective": "旧任务",
            "completedAt": 1_700_000_000_000_i64,
            "result": "旧结果",
            "resultMetadata": {
                "decision": {
                    "presentation": {
                        "speech": "旧任务已经完成。",
                        "inline": {
                            "title": "旧结果",
                            "format": "code",
                            "content": "const done = true",
                        },
                    },
                },
                "backendRef": {"sessionId": "legacy-session", "directory": "/private/legacy"},
            },
        }))]),
    );
    let manager = WorkManager::builder()
        .now(clock())
        .store(store(&path))
        .build();

    let restored = manager.get("legacy-work", None).await.expect("restored");
    let published = serde_json::to_value(restored.result_metadata).expect("serializes");
    assert_eq!(
        published,
        json!({
            "presentation": {
                "speech": "旧任务已经完成。",
                "inline": {"title": "旧结果", "format": "code", "content": "const done = true"},
            }
        }),
    );

    manager.persist().await;
    let saved = read_tasks(&path);
    assert_eq!(saved[0]["resultMetadata"], published);
    assert!(
        !saved[0].to_string().contains("/private/legacy"),
        "the backend reference is gone from disk too",
    );
}

// ── what does not survive ──────────────────────────────────────────────────

/// The catalogue's stated consequence: *"Restored work has no lane and priority
/// 0, so lane serialization does not survive a restart."*
#[tokio::test(start_paused = true)]
async fn lane_serialization_does_not_survive_a_restart() {
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("tasks.json");
    let lane = via_work::coordinator_lane(OWNER);

    let first = WorkManager::builder()
        .now(clock())
        .store(store(&path))
        .build();
    let runners: Vec<GatedRunner> = (0..2).map(|_| GatedRunner::new()).collect();
    for runner in &runners {
        first
            .create(
                NewWork::new("laned", OWNER)
                    .lane(&lane, 1)
                    .runner(Arc::new(runner.clone())),
            )
            .await
            .expect("accepted");
    }
    settle().await;
    assert!(runners[0].is_running() && !runners[1].is_running());
    first.persist().await;

    let saved = read_tasks(&path);
    for record in &saved {
        assert!(record.get("laneKey").is_none(), "{record}");
        assert!(record.get("priority").is_none(), "{record}");
    }

    // Both were `running`, so a restart force-fails both rather than
    // re-queueing them into a lane that no longer exists.
    first.close().await;
    let restored = WorkManager::builder()
        .now(clock())
        .locale(Locale::Zh)
        .store(store(&path))
        .build();
    let works = restored.list(WorkQuery::owner(OWNER)).await;
    assert_eq!(works.len(), 2);
    assert!(works.iter().all(|work| work.status == WorkStatus::Failed));
}

/// A corrupt `tasks.json` starts the manager empty and says so on the health
/// surface, rather than refusing to start.
#[tokio::test(start_paused = true)]
async fn a_corrupt_store_starts_empty_with_a_warning() {
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("tasks.json");
    std::fs::write(&path, "{\"version\": 1, \"tasks\": \"not an array\"}\n").expect("seeded");

    let manager = WorkManager::builder()
        .now(clock())
        .locale(Locale::Zh)
        .store(store(&path))
        .build();
    assert!(manager.list(WorkQuery::default()).await.is_empty());

    let health = manager.store_health().await;
    assert!(!health.ok);
    assert!(
        health.persistence_enabled,
        "persistence continues after a quarantine"
    );
    let warning = health.warning.expect("a warning");
    assert!(
        warning.message.contains("任务状态文件格式无效"),
        "{}",
        warning.message
    );
    assert!(warning.quarantine_path.is_some());

    // And the manager still works.
    let accepted = manager
        .create(NewWork::new("after", OWNER).runner(Arc::new(ImmediateRunner::completing("ok"))))
        .await
        .expect("accepted");
    assert_eq!(
        manager
            .wait(&accepted.work.id)
            .await
            .expect("terminal")
            .status,
        WorkStatus::Completed,
    );
}

/// Upstream: the restart half of *reuses a persisted submission key instead of
/// running duplicate work*.
#[tokio::test(start_paused = true)]
async fn a_submission_key_suppresses_a_duplicate_across_a_restart() {
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("tasks.json");
    let runner = ImmediateRunner::completing("完成");

    let first = WorkManager::builder()
        .now(clock())
        .store(store(&path))
        .runner(Arc::new(runner.clone()))
        .build();
    let accepted = first
        .create(
            NewWork::new("只执行一次", OWNER)
                .session(SESSION)
                .submission_key("delegation:voice:turn-one"),
        )
        .await
        .expect("accepted");
    first.wait(&accepted.work.id).await;
    first.close().await;
    assert_eq!(runner.runs(), 1);

    let restored = WorkManager::builder()
        .now(clock())
        .store(store(&path))
        .runner(Arc::new(runner.clone()))
        .build();
    let duplicate = restored
        .create(
            NewWork::new("不要再次执行", OWNER)
                .session(SESSION)
                .submission_key("delegation:voice:turn-one"),
        )
        .await
        .expect("accepted");
    settle().await;

    assert_eq!(duplicate.work.id, accepted.work.id);
    assert!(duplicate.reused);
    assert_eq!(runner.runs(), 1, "the model's second call ran nothing");
}

// ── coalescing ─────────────────────────────────────────────────────────────

/// Upstream: *coalesces high-frequency task activity into a deferred atomic
/// write* — at the manager level, where the activity actually comes from.
#[tokio::test(start_paused = true)]
async fn a_burst_of_activity_costs_one_write_of_the_last_state() {
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("tasks.json");
    let manager = WorkManager::builder()
        .now(clock())
        .store(store(&path))
        .build();
    let runner = GatedRunner::new();
    let accepted = manager
        .create(NewWork::new("busy", OWNER).runner(Arc::new(runner.clone())))
        .await
        .expect("accepted");
    settle().await;

    let mut tracker = via_downstream::ActivityTracker::new();
    for index in 0..8 {
        let event = tracker
            .project(&via_downstream::RawSessionUpdate {
                name: Some(format!("tool-{index}")),
                ..via_downstream::RawSessionUpdate::tool_call(&format!("call-{index}"))
            })
            .expect("projects");
        runner.emit(RunnerEvent::Activity(event)).await;
    }
    settle().await;

    // The deferred write has not landed yet, so the file still holds the
    // state the last *synchronous* save wrote — `task.running`, with no
    // activity.
    let before = read_tasks(&path);
    assert_eq!(before[0]["activity"], json!([]));

    manager.flush().await;
    let after = read_tasks(&path);
    assert_eq!(
        after[0]["activity"].as_array().map(Vec::len),
        Some(8),
        "one write, carrying every entry",
    );

    runner.complete("done");
    manager.wait(&accepted.work.id).await;
}
