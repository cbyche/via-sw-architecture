//! The owning task.
//!
//! One `tokio` task holds the whole Work table by value and is the only thing
//! that mutates it. Every runner, canceler and timer talks to it over the
//! bounded command channel, so "A was admitted before B" and "A's activity was
//! recorded before its lane was released" are properties of a queue rather than
//! of a lock.
//!
//! `handle` is deliberately **synchronous**: nothing in the state machine
//! awaits, so no ordering can be lost between a decision and the mutation that
//! follows from it. Everything that must await — a runner, a canceler, a
//! coordinator query, a timer — is spawned and reports back as another command.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use indexmap::IndexMap;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use via_downstream::text::clean;
use via_downstream::{CancelOutcome, CancelRoute, CancelTarget};
use via_i18n::{Locale, format as i18n_format, keys, t};
use via_log::{LogLevel, Logger};
use via_protocol::{WorkKind, WorkStatus};
use via_store::StoreHealth;

use crate::activity::{self, Activity};
use crate::clock::NowFn;
use crate::command::Command;
use crate::delegation::DelegationRef;
use crate::event::{WorkEvent, WorkEventKind};
use crate::limits::{PROGRESS_HEARTBEAT_MS, RetentionPolicy, SCHEDULED_TASK_CLEANUP_MS};
use crate::progress::progress_message;
use crate::reconcile::ReconciliationLedger;
use crate::record::{
    DEFAULT_SESSION_ID, NotificationStatus, PersistedWork, PublicWork, WorkRecord, WorkSnapshot,
    new_work_id,
};
use crate::runner::{
    AbortSignal, CancelRequest, CoordinatorQuery, DelegatedWorkRecovery, RunFailure, RunnerEvent,
    WorkCanceler, WorkContext, WorkEventSink, WorkOutcome, WorkRunner,
};
use crate::scheduler::{Admission, AdmissionScheduler};
use crate::store::WorkStore;

use super::{
    NewScheduledWork, NewWork, NotificationClaim, ScheduledKind, ScheduledWork, WorkAcceptance,
    WorkManager, WorkManagerBuilder, WorkQuery,
};

/// The runner a reminder gets when nobody supplies one.
///
/// `server/src/task/task-manager.mjs:163-166,444-447`:
///
/// ```js
/// async (obj) => ({ content: obj, metadata: { presentation: { speech: obj } } })
/// ```
///
/// It speaks the stored text back, which is why a reminder that had already
/// fired when the Gateway stopped can be safely replayed as overdue catch-up:
/// running it twice says the same sentence twice, and never does anything.
#[must_use]
pub fn reminder_runner() -> Arc<dyn WorkRunner> {
    Arc::new(|objective: String, _context: WorkContext| async move {
        Ok(WorkOutcome {
            metadata: Some(serde_json::json!({
                "presentation": { "speech": objective.clone() },
            })),
            content: objective,
        })
    })
}

/// A recovered delegation's runner: upstream's
/// `(_objective, context) => runner(snapshot, context)`.
struct RecoveryRunner {
    recovery: Arc<dyn DelegatedWorkRecovery>,
    snapshot: WorkSnapshot,
}

#[async_trait]
impl WorkRunner for RecoveryRunner {
    async fn run(
        &self,
        _objective: String,
        context: WorkContext,
    ) -> Result<WorkOutcome, RunFailure> {
        self.recovery.run(self.snapshot.clone(), context).await
    }
}

/// A recovered delegation's canceler: upstream's
/// `context => canceler(snapshot, context)`.
struct RecoveryCanceler {
    recovery: Arc<dyn DelegatedWorkRecovery>,
    snapshot: WorkSnapshot,
}

#[async_trait]
impl WorkCanceler for RecoveryCanceler {
    async fn cancel(&self, request: CancelRequest) -> Result<CancelOutcome, RunFailure> {
        self.recovery.cancel(self.snapshot.clone(), request).await
    }
}

/// One Work, plus everything about it that does not survive a restart.
struct Entry {
    record: WorkRecord,
    runner: Option<Arc<dyn WorkRunner>>,
    canceler: Option<Arc<dyn WorkCanceler>>,
    signal: AbortSignal,
    timers: CancellationToken,
    terminal_handled: bool,
    scheduler_held: bool,
    /// What the canceler reported, once it has reported anything.
    cancellation: Option<CancelOutcome>,
    /// Whether a cancellation is in flight — upstream's `cancelPromise`.
    cancel_requested: bool,
    /// Whether the abort a cancellation raised actually settled the runner.
    /// **This is the second of the two confirmations.**
    cancel_runner_settled: bool,
    waiters: Vec<oneshot::Sender<Option<PublicWork>>>,
    cancel_waiters: Vec<oneshot::Sender<Option<PublicWork>>>,
}

impl Entry {
    fn new(record: WorkRecord) -> Self {
        Self {
            record,
            runner: None,
            canceler: None,
            signal: AbortSignal::new(),
            timers: CancellationToken::new(),
            terminal_handled: false,
            scheduler_held: false,
            cancellation: None,
            cancel_requested: false,
            cancel_runner_settled: false,
            waiters: Vec::new(),
            cancel_waiters: Vec::new(),
        }
    }

    fn admission(&self) -> Admission {
        Admission {
            work_id: self.record.id.clone(),
            owner_id: self.record.owner_id.clone(),
            lane_key: self.record.lane_key.clone(),
            lane_limit: self.record.lane_limit,
        }
    }
}

pub(super) struct Actor {
    works: IndexMap<String, Entry>,
    scheduler: AdmissionScheduler,
    store: WorkStore,
    retention: RetentionPolicy,
    progress_check_ms: i64,
    scheduled_task_timeout_ms: i64,
    default_runner: Option<Arc<dyn WorkRunner>>,
    scheduled_task_runner: Option<Arc<dyn WorkRunner>>,
    coordinator_query: Option<Arc<dyn CoordinatorQuery>>,
    ledger: ReconciliationLedger,
    recovery_candidates: Vec<String>,
    locale: Locale,
    now: NowFn,
    logger: Option<Logger>,
    events: broadcast::Sender<WorkEvent>,
    commands: mpsc::WeakSender<Command>,
    tracker: TaskTracker,
}

/// Build the manager, restore `tasks.json`, and start the owning task.
pub(super) fn spawn(builder: WorkManagerBuilder) -> WorkManager {
    let parts = builder.parts();

    let (commands, receiver) = mpsc::channel(parts.command_capacity.max(1));
    let (events, _) = broadcast::channel(parts.event_capacity.max(1));
    let tracker = parts.tracker;

    let mut actor = Actor {
        works: IndexMap::new(),
        scheduler: AdmissionScheduler::new(parts.max_concurrent, parts.max_concurrent_per_owner),
        store: parts.store.unwrap_or_else(WorkStore::in_memory),
        retention: parts.retention,
        progress_check_ms: parts.progress_check_ms,
        scheduled_task_timeout_ms: parts.scheduled_task_timeout_ms,
        default_runner: parts.runner,
        scheduled_task_runner: None,
        coordinator_query: None,
        ledger: ReconciliationLedger::new(),
        recovery_candidates: Vec::new(),
        locale: parts.locale,
        now: parts.now,
        logger: parts.logger,
        events: events.clone(),
        commands: commands.downgrade(),
        tracker: tracker.clone(),
    };
    actor.restore();

    tracker.spawn(actor.run(receiver));

    WorkManager {
        commands,
        events,
        tracker,
    }
}

impl Actor {
    async fn run(mut self, mut receiver: mpsc::Receiver<Command>) {
        while let Some(command) = receiver.recv().await {
            self.handle(command);
        }
        // Shutdown is dropping the senders. Everything queued has been
        // processed; the coalesced write is the only state that has not
        // reached the disk.
        self.store.flush();
    }

    // ── restart recovery ────────────────────────────────────────────────

    /// `restore()` — `server/src/task/task-manager.mjs:147-231`.
    ///
    /// Active Work cannot be safely resumed after a Gateway restart, because
    /// nothing survived that knows what the backend was doing. It is
    /// force-failed with an explicit reason and its notification is queued, so
    /// the user still hears that it was interrupted rather than watching it
    /// vanish.
    ///
    /// Two exceptions, both because *something* did survive:
    ///
    /// - a `reminder` that was `queued` or `running` is restored as
    ///   `scheduled` and re-fired as overdue catch-up: its runner only speaks
    ///   the stored text, so replaying it is safe;
    /// - a `delegated` or `finalizing` Work whose delegation carries **both**
    ///   ids is restored as `queued` and offered to
    ///   [`WorkManager::recover_delegated`], because the backend session it
    ///   named is still there.
    fn restore(&mut self) {
        let now = (self.now)();
        for saved in self.store.load() {
            let replayable_reminder =
                saved.kind == WorkKind::Reminder && saved.status.is_replayable_reminder();
            if saved.status == WorkStatus::Scheduled || replayable_reminder {
                let mut record = restore_record(saved);
                record.status = WorkStatus::Scheduled;
                let kind = record.kind;
                let mut entry = Entry::new(record);
                // A `scheduled_task`'s runner is attached from
                // `scheduled_task_runner` when it starts; a reminder's is
                // rebuilt here, because it is a pure function of the objective.
                entry.runner = (kind == WorkKind::Reminder).then(reminder_runner);
                self.works.insert(entry.record.id.clone(), entry);
                continue;
            }

            let was_active = saved.status.is_active();
            let can_recover =
                matches!(saved.status, WorkStatus::Delegated | WorkStatus::Finalizing)
                    && saved
                        .delegation
                        .as_ref()
                        .is_some_and(DelegationRef::is_addressable);

            let saved_status = saved.status;
            let saved_notification = saved.notification_status;
            let saved_delegation = saved.delegation.clone();
            let saved_authorization = saved.authorization.clone();
            let mut record = restore_record(saved);

            record.status = if can_recover {
                WorkStatus::Queued
            } else if was_active {
                WorkStatus::Failed
            } else {
                saved_status
            };
            if was_active && !can_recover {
                record.error =
                    Some(t(self.locale, keys::WORK_RESTART_INTERACTIVE_INCOMPLETE).to_owned());
                record.completed_at = Some(now);
            }
            record.delegation = if can_recover || !was_active {
                saved_delegation
            } else {
                None
            };
            record.authorization = if was_active || saved_status.is_terminal() {
                None
            } else {
                saved_authorization
            };
            record.notification_status = if (was_active && !can_recover)
                || saved_notification == NotificationStatus::Delivering
            {
                NotificationStatus::Pending
            } else if can_recover {
                NotificationStatus::None
            } else {
                saved_notification
            };
            record.notification_claimant_id = None;
            record.notification_claimed_at = None;

            let id = record.id.clone();
            let mut entry = Entry::new(record);
            entry.terminal_handled = entry.record.status.is_terminal();
            self.works.insert(id.clone(), entry);
            if can_recover {
                self.recovery_candidates.push(id);
            }
        }
        self.prune();
    }

    // ── the command dispatch ────────────────────────────────────────────

    #[expect(
        clippy::too_many_lines,
        reason = "one arm per command; splitting it would hide the dispatch"
    )]
    fn handle(&mut self, command: Command) {
        match command {
            Command::Create { request, reply } => {
                let acceptance = self.create(*request);
                let _ = reply.send(acceptance);
            }
            Command::CreateScheduled { request, reply } => {
                let acceptance = self.create_scheduled(*request);
                let _ = reply.send(acceptance);
            }
            Command::Get {
                work_id,
                owner_id,
                reply,
            } => {
                let now = (self.now)();
                let found = self
                    .works
                    .get(&work_id)
                    .filter(|entry| owns(entry, owner_id.as_deref()))
                    .map(|entry| entry.record.to_public(now));
                let _ = reply.send(found);
            }
            Command::List { query, reply } => {
                let _ = reply.send(self.list(&query));
            }
            Command::Cancel {
                work_id,
                owner_id,
                reply,
            } => self.cancel(&work_id, owner_id.as_deref(), reply),
            Command::ConfirmCancellation {
                work_id,
                confirmed_at,
                reply,
            } => self.confirm_cancellation(&work_id, &confirmed_at, reply),
            Command::Wait { work_id, reply } => {
                let now = (self.now)();
                match self.works.get_mut(&work_id) {
                    None => {
                        let _ = reply.send(None);
                    }
                    Some(entry) if entry.record.status.is_terminal() => {
                        let _ = reply.send(Some(entry.record.to_public(now)));
                    }
                    Some(entry) => entry.waiters.push(reply),
                }
            }
            Command::ClaimNotifications { request, reply } => {
                let claimed = self.claim_notifications(&request);
                let _ = reply.send(claimed);
            }
            Command::MarkDelivered {
                work_ids,
                claimant_id,
                reply,
            } => {
                let count = self.mark_delivered(&work_ids, claimant_id.as_deref());
                let _ = reply.send(count);
            }
            Command::RenewClaims {
                work_ids,
                claimant_id,
                reply,
            } => {
                let count = self.renew_claims(&work_ids, claimant_id.as_deref());
                let _ = reply.send(count);
            }
            Command::ReleaseClaims {
                work_ids,
                claimant_id,
                reply,
            } => {
                let count = self.release_claims(&work_ids, claimant_id.as_deref());
                let _ = reply.send(count);
            }
            Command::ReclaimExpiredClaims { reply } => {
                let count = self.reclaim_expired(true);
                let _ = reply.send(count);
            }
            Command::ConfigureRetention { policy, reply } => {
                self.retention = policy;
                self.prune();
                let _ = reply.send(());
            }
            Command::ConfigureScheduledTaskRunner { runner, reply } => {
                self.scheduled_task_runner = Some(runner);
                let _ = reply.send(());
            }
            Command::ConfigureCoordinatorQuery { query, reply } => {
                self.coordinator_query = Some(query);
                let _ = reply.send(());
            }
            Command::RecoverDelegated { recovery, reply } => {
                let count = self.recover_delegated(&recovery);
                let _ = reply.send(count);
            }
            Command::Prune { reply } => {
                let count = self.prune();
                let _ = reply.send(count);
            }
            Command::Persist { reply } => {
                let _ = reply.send(self.persist());
            }
            Command::Flush { reply } => {
                let _ = reply.send(self.store.flush());
            }
            Command::StoreHealth { reply } => {
                let _ = reply.send(self.store.health());
            }
            Command::ScheduledSnapshot { reply } => {
                let scheduled = self
                    .works
                    .values()
                    .filter(|entry| entry.record.status == WorkStatus::Scheduled)
                    .filter_map(|entry| {
                        entry
                            .record
                            .schedule
                            .as_ref()
                            .map(|schedule| ScheduledWork {
                                work_id: entry.record.id.clone(),
                                owner_id: entry.record.owner_id.clone(),
                                at: schedule.at,
                            })
                    })
                    .collect();
                let _ = reply.send(scheduled);
            }
            Command::FireScheduled { work_ids, reply } => {
                let count = self.fire_scheduled(&work_ids);
                let _ = reply.send(count);
            }
            Command::RecordCancellationFact {
                owner_id,
                fact,
                reply,
            } => {
                self.ledger.record(&owner_id, *fact);
                let _ = reply.send(());
            }
            Command::PendingFacts { owner_id, reply } => {
                let _ = reply.send(self.ledger.pending(&owner_id).to_vec());
            }
            Command::ClearFacts { owner_id, reply } => {
                let _ = reply.send(self.ledger.clear(&owner_id));
            }
            Command::RunnerEvent { work_id, event } => self.runner_event(&work_id, event),
            Command::RunnerSettled { work_id, result } => self.runner_settled(&work_id, *result),
            Command::CancelerSettled { work_id, result } => {
                self.canceler_settled(&work_id, *result);
            }
            Command::ProgressTick { work_id } => {
                if self
                    .works
                    .get(&work_id)
                    .is_some_and(|entry| entry.record.status.is_active())
                {
                    self.emit(WorkEventKind::Progress, &work_id);
                }
            }
            Command::ProgressCheckTick { work_id } => self.progress_check(&work_id),
            Command::ProgressCheckAnswer {
                work_id,
                message,
                delegated,
            } => {
                let now = (self.now)();
                if let Some(work) = self
                    .works
                    .get(&work_id)
                    .filter(|entry| entry.record.status.is_active())
                    .map(|entry| entry.record.to_public(now))
                {
                    self.publish(WorkEvent::progress_check(work, message, delegated));
                }
            }
            Command::TimeoutFired { work_id } => {
                let reason = t(self.locale, keys::WORK_SCHEDULED_TIMEOUT_ABORT);
                if let Some(entry) = self
                    .works
                    .get(&work_id)
                    .filter(|entry| entry.record.status.is_active())
                {
                    entry.signal.abort(reason);
                }
            }
            Command::TimeoutCleanup { work_id } => self.timeout_cleanup(&work_id),
        }
    }

    // ── create ──────────────────────────────────────────────────────────

    fn create(&mut self, request: NewWork) -> WorkAcceptance {
        let now = (self.now)();
        let owner_id = request.owner_id;
        let submission_key =
            clean(request.submission_key.as_deref().unwrap_or_default()).to_owned();

        // Duplicate suppression. A realtime model that re-calls the delegation
        // tool because it did not hear its own acknowledgement is a routine
        // failure mode, and the key survives a restart, so the second call is
        // answered with the first Work rather than running it twice.
        if !submission_key.is_empty()
            && let Some(existing) = self.works.values().find(|entry| {
                entry.record.owner_id == owner_id
                    && entry.record.submission_key.as_deref() == Some(submission_key.as_str())
            })
        {
            return WorkAcceptance {
                work: existing.record.to_public(now),
                reused: true,
            };
        }

        let kind = request.kind;
        let record = WorkRecord {
            id: new_work_id(),
            status: WorkStatus::Queued,
            kind,
            parent_work_id: request.parent_work_id.filter(|id| !id.is_empty()),
            priority: request.priority,
            objective: clean(&request.objective).to_owned(),
            owner_id,
            session_id: session_or_default(request.session_id),
            turn_id: request.turn_id.filter(|id| !id.is_empty()),
            submission_key: (!submission_key.is_empty()).then_some(submission_key),
            lane_key: request.lane_key.filter(|key| !key.is_empty()),
            lane_limit: request.lane_limit,
            created_at: now,
            started_at: None,
            completed_at: None,
            elapsed_ms: 0,
            result: None,
            error: None,
            result_metadata: None,
            activity: Vec::new(),
            delegation: None,
            authorization: None,
            notification_status: NotificationStatus::None,
            notification_claimant_id: None,
            notification_claimed_at: None,
            notification_delivered_at: None,
            schedule: None,
            timeout_ms: None,
            // Only `work` announces long-running progress.
            progress_check_ms: (kind == WorkKind::Work && self.progress_check_ms > 0)
                .then_some(self.progress_check_ms),
        };

        let id = record.id.clone();
        // The receipt is taken **before** draining: upstream defers `drain()`
        // to a microtask, so `create()` always answers `queued` even when the
        // scheduler starts the Work immediately.
        let receipt = record.to_public(now);
        let mut entry = Entry::new(record);
        entry.runner = request.runner.or_else(|| self.default_runner.clone());
        entry.canceler = request.canceler;
        self.works.insert(id.clone(), entry);

        self.emit(WorkEventKind::Accepted, &id);
        self.drain();
        WorkAcceptance {
            work: receipt,
            reused: false,
        }
    }

    fn create_scheduled(&mut self, request: NewScheduledWork) -> WorkAcceptance {
        let now = (self.now)();
        let kind = request.kind;
        let record = WorkRecord {
            id: new_work_id(),
            status: WorkStatus::Scheduled,
            kind: kind.work_kind(),
            parent_work_id: None,
            priority: 0,
            objective: clean(&request.objective).to_owned(),
            owner_id: request.owner_id,
            session_id: session_or_default(request.session_id),
            turn_id: request.turn_id.filter(|id| !id.is_empty()),
            submission_key: None,
            lane_key: None,
            lane_limit: crate::scheduler::COORDINATOR_LANE_LIMIT,
            created_at: now,
            started_at: None,
            completed_at: None,
            elapsed_ms: 0,
            result: None,
            error: None,
            result_metadata: None,
            activity: Vec::new(),
            delegation: None,
            authorization: None,
            notification_status: NotificationStatus::None,
            notification_claimant_id: None,
            notification_claimed_at: None,
            notification_delivered_at: None,
            schedule: Some(request.schedule),
            // Only `scheduled_task` gets a wall-clock watchdog.
            timeout_ms: (kind == ScheduledKind::Task).then(|| {
                request
                    .timeout_ms
                    .filter(|timeout| *timeout > 0)
                    .unwrap_or(self.scheduled_task_timeout_ms)
            }),
            progress_check_ms: None,
        };

        let id = record.id.clone();
        let receipt = record.to_public(now);
        let mut entry = Entry::new(record);
        entry.runner = request
            .runner
            .or_else(|| (kind == ScheduledKind::Reminder).then(reminder_runner));
        self.works.insert(id.clone(), entry);

        self.emit(WorkEventKind::Scheduled, &id);
        // Deliberately no drain: a scheduled Work waits for its timer.
        WorkAcceptance {
            work: receipt,
            reused: false,
        }
    }

    // ── admission ───────────────────────────────────────────────────────

    /// `drain()` — `server/src/task/task-manager.mjs:465-476`.
    ///
    /// Queued Work in priority order, then creation order. Rust's `sort_by` is
    /// stable and `works` iterates in insertion order, so two Work items
    /// created in the same millisecond keep the order they were created in —
    /// which is what V8's stable sort gives upstream.
    fn drain(&mut self) {
        let mut candidates: Vec<(i64, i64, String)> = self
            .works
            .values()
            .filter(|entry| entry.record.status == WorkStatus::Queued)
            .map(|entry| {
                (
                    entry.record.priority,
                    entry.record.created_at,
                    entry.record.id.clone(),
                )
            })
            .collect();
        candidates.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
        for (_, _, work_id) in candidates {
            let Some(entry) = self.works.get(&work_id) else {
                continue;
            };
            if !self.scheduler.can_start(&entry.admission()) {
                continue;
            }
            self.start(&work_id);
        }
    }

    /// `start(task)` — `server/src/task/task-manager.mjs:478-705`.
    fn start(&mut self, work_id: &str) {
        let now = (self.now)();
        let Some(entry) = self.works.get_mut(work_id) else {
            return;
        };
        if !apply_status(&mut entry.record, WorkStatus::Running, self.logger.as_ref()) {
            return;
        }
        entry.record.started_at = Some(now);
        entry.signal = AbortSignal::new();
        entry.timers = CancellationToken::new();
        entry.terminal_handled = false;
        // A `scheduled_task` restored from disk lost its runner in
        // serialization; the configured one is attached here.
        if entry.runner.is_none() && entry.record.kind == WorkKind::ScheduledTask {
            entry.runner.clone_from(&self.scheduled_task_runner);
        }

        let admission = entry.admission();
        let runner = entry.runner.clone();
        let signal = entry.signal.clone();
        let timers = entry.timers.clone();
        let objective = entry.record.objective.clone();
        let kind = entry.record.kind;
        let timeout_ms = entry.record.timeout_ms;
        let progress_check_ms = entry.record.progress_check_ms;
        let context_seed = (
            entry.record.id.clone(),
            entry.record.owner_id.clone(),
            entry.record.session_id.clone(),
            entry.record.turn_id.clone(),
            entry.record.schedule.clone(),
        );
        entry.scheduler_held = true;
        self.scheduler.acquire(admission);

        self.emit(WorkEventKind::Running, work_id);

        self.arm_heartbeat(work_id, &timers);
        if kind == WorkKind::ScheduledTask
            && let Some(timeout_ms) = timeout_ms.filter(|timeout| *timeout > 0)
        {
            self.arm_timeout(work_id, timeout_ms, &timers);
        }
        if kind == WorkKind::Work
            && let Some(cadence) = progress_check_ms.filter(|cadence| *cadence > 0)
        {
            self.arm_progress_check(work_id, cadence, &timers);
        }

        let Some(commands) = self.commands.upgrade() else {
            return;
        };
        let context = WorkContext {
            work_id: context_seed.0,
            owner_id: context_seed.1,
            session_id: context_seed.2,
            turn_id: context_seed.3,
            kind,
            schedule: context_seed.4,
            events: WorkEventSink::new(work_id.to_owned(), commands.clone()),
            signal,
        };
        let missing_runner = t(self.locale, keys::WORK_NO_RUNNER_CONFIGURED).to_owned();
        let id = work_id.to_owned();
        self.tracker.spawn(async move {
            let result = match runner {
                Some(runner) => runner.run(objective, context).await,
                None => Err(RunFailure::new(missing_runner)),
            };
            let _ = commands
                .send(Command::RunnerSettled {
                    work_id: id,
                    result: Box::new(result),
                })
                .await;
        });
    }

    // ── timers ──────────────────────────────────────────────────────────

    fn arm_heartbeat(&self, work_id: &str, timers: &CancellationToken) {
        self.arm_repeating(
            work_id,
            Duration::from_millis(PROGRESS_HEARTBEAT_MS.unsigned_abs()),
            timers,
            |work_id| Command::ProgressTick { work_id },
        );
    }

    fn arm_progress_check(&self, work_id: &str, cadence_ms: i64, timers: &CancellationToken) {
        self.arm_repeating(
            work_id,
            Duration::from_millis(cadence_ms.unsigned_abs()),
            timers,
            |work_id| Command::ProgressCheckTick { work_id },
        );
    }

    fn arm_repeating(
        &self,
        work_id: &str,
        every: Duration,
        timers: &CancellationToken,
        build: fn(String) -> Command,
    ) {
        let Some(commands) = self.commands.upgrade() else {
            return;
        };
        let token = timers.clone();
        let work_id = work_id.to_owned();
        self.tracker.spawn(async move {
            loop {
                tokio::select! {
                    () = token.cancelled() => break,
                    () = tokio::time::sleep(every) => {
                        if commands.send(build(work_id.clone())).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });
    }

    /// The hard wall-clock budget for a `scheduled_task`
    /// (`server/src/task/task-manager.mjs:549-580`): abort first, then a
    /// five-second cleanup window, then force-fail. The window exists so a
    /// runner that honours the abort reports its own failure rather than being
    /// overwritten by the watchdog's.
    fn arm_timeout(&self, work_id: &str, timeout_ms: i64, timers: &CancellationToken) {
        let Some(commands) = self.commands.upgrade() else {
            return;
        };
        let token = timers.clone();
        let work_id = work_id.to_owned();
        let budget = Duration::from_millis(timeout_ms.unsigned_abs());
        let cleanup = Duration::from_millis(SCHEDULED_TASK_CLEANUP_MS.unsigned_abs());
        self.tracker.spawn(async move {
            tokio::select! {
                () = token.cancelled() => return,
                () = tokio::time::sleep(budget) => {}
            }
            if commands
                .send(Command::TimeoutFired {
                    work_id: work_id.clone(),
                })
                .await
                .is_err()
            {
                return;
            }
            tokio::select! {
                () = token.cancelled() => return,
                () = tokio::time::sleep(cleanup) => {}
            }
            let _ = commands.send(Command::TimeoutCleanup { work_id }).await;
        });
    }

    fn timeout_cleanup(&mut self, work_id: &str) {
        let now = (self.now)();
        let Some(entry) = self.works.get_mut(work_id) else {
            return;
        };
        if !entry.record.status.is_active() {
            return;
        }
        let minutes = entry.record.timeout_ms.map_or(0, round_minutes).to_string();
        if !apply_status(&mut entry.record, WorkStatus::Failed, self.logger.as_ref()) {
            return;
        }
        entry.terminal_handled = true;
        entry.record.error = Some(i18n_format(
            self.locale,
            keys::WORK_SCHEDULED_TIMEOUT_ERROR,
            &[("minutes", &minutes)],
        ));
        entry.record.completed_at = Some(now);
        entry.record.elapsed_ms = entry.record.started_at.map_or(0, |started| now - started);
        entry.record.notification_status = NotificationStatus::Pending;
        entry.timers.cancel();
        self.release_lane(work_id);
        self.emit(WorkEventKind::Failed, work_id);
        self.emit(WorkEventKind::NotificationPending, work_id);
        self.persist_deferred();
        // Upstream leaves `task.resolve` uncalled on this path, so every
        // `wait()` on a timed-out scheduled task hangs forever. Resolved here;
        // recorded in `docs/deviations/phase-3.md`.
        self.resolve_waiters(work_id);
        self.drain();
    }

    // ── runner reports ──────────────────────────────────────────────────

    /// The `onEvent` closure — `server/src/task/task-manager.mjs:491-537`.
    fn runner_event(&mut self, work_id: &str, event: RunnerEvent) {
        let Some(entry) = self.works.get_mut(work_id) else {
            return;
        };
        // A settled Work has no progress to report. Upstream has no such
        // guard, so a late `backend.permission.requested` can put a pending
        // permission back onto a completed record.
        if entry.record.status.is_terminal() {
            return;
        }

        match event {
            RunnerEvent::PermissionRequested(permission) => {
                entry.record.authorization = Some(permission);
                self.emit(WorkEventKind::PermissionRequested, work_id);
                return;
            }
            RunnerEvent::PermissionResolved(permission) => {
                if entry
                    .record
                    .authorization
                    .as_ref()
                    .is_some_and(|pending| pending.is_same_request(&permission))
                {
                    entry.record.authorization = None;
                }
                let now = (self.now)();
                let work = entry.record.to_public(now);
                self.publish(WorkEvent::permission_resolved(work, permission));
                return;
            }
            _ => {}
        }

        // A cancellation in flight stops progress from moving the record.
        if matches!(
            entry.record.status,
            WorkStatus::Cancelling | WorkStatus::Cancelled
        ) {
            return;
        }

        match event {
            RunnerEvent::Delegated(delegation) => {
                if !entry.record.status.can_transition_to(WorkStatus::Delegated) {
                    return;
                }
                // Delegation correlation: a Work waits on exactly one target.
                if entry
                    .record
                    .delegation
                    .as_ref()
                    .is_some_and(|existing| !existing.correlates_with(&delegation))
                {
                    return;
                }
                entry.record.status = WorkStatus::Delegated;
                entry.record.delegation = Some(delegation);
                // The lane is released here: the coordinator session is free
                // while the delegated session works.
                self.release_lane(work_id);
                self.emit(WorkEventKind::Delegated, work_id);
                self.drain();
            }
            RunnerEvent::DelegationCompleted(delegation) => {
                // Only the completion correlated to *this* delegation may move
                // the Work — `docs/architecture.md` §11, invariant 4. A stale
                // result, a sibling delegation's result, and a completion that
                // arrives while the Work is not delegated at all are dropped.
                if entry.record.status != WorkStatus::Delegated {
                    return;
                }
                let correlated = entry
                    .record
                    .delegation
                    .as_ref()
                    .is_some_and(|existing| existing.correlates_with(&delegation));
                if !correlated {
                    return;
                }
                entry.record.status = WorkStatus::Finalizing;
                entry.record.delegation = Some(delegation.completed());
                self.emit(WorkEventKind::Finalizing, work_id);
            }
            RunnerEvent::Activity(session_event) => {
                activity::merge(&mut entry.record.activity, Activity::from(&session_event));
                self.emit(WorkEventKind::Progress, work_id);
                self.persist_deferred();
            }
            RunnerEvent::PermissionRequested(_) | RunnerEvent::PermissionResolved(_) => {}
        }
    }

    /// The runner settled — `server/src/task/task-manager.mjs:652-704`.
    fn runner_settled(&mut self, work_id: &str, result: Result<WorkOutcome, RunFailure>) {
        let now = (self.now)();
        let Some(entry) = self.works.get_mut(work_id) else {
            return;
        };
        let cancelling = matches!(
            entry.record.status,
            WorkStatus::Cancelling | WorkStatus::Cancelled
        );
        if !entry.terminal_handled && !cancelling {
            let outcome = match result {
                Ok(outcome) => {
                    if !apply_status(
                        &mut entry.record,
                        WorkStatus::Completed,
                        self.logger.as_ref(),
                    ) {
                        return;
                    }
                    entry.record.result = Some(clean(&outcome.content).to_owned());
                    entry.record.result_metadata = outcome.metadata;
                    true
                }
                Err(failure) => {
                    if !apply_status(&mut entry.record, WorkStatus::Failed, self.logger.as_ref()) {
                        return;
                    }
                    entry.record.error = Some(failure.message);
                    false
                }
            };
            debug_assert_eq!(
                outcome,
                entry.record.status == WorkStatus::Completed,
                "the branch above set the status it reports",
            );
        }

        if entry.terminal_handled {
            return;
        }
        entry.timers.cancel();
        entry.record.authorization = None;

        match entry.record.status {
            // The abort a cancellation raised has now settled the runner.
            // **That is a confirmation**: the request really stopped.
            WorkStatus::Cancelling => {
                entry.cancel_runner_settled = true;
                self.try_finish_cancellation(work_id);
                return;
            }
            WorkStatus::Cancelled => {
                self.release_lane(work_id);
                self.drain();
                return;
            }
            _ => {}
        }

        entry.record.completed_at = Some(now);
        entry.record.elapsed_ms = entry.record.started_at.map_or(0, |started| now - started);
        entry.record.notification_status = NotificationStatus::Pending;
        entry.terminal_handled = true;
        let completed = entry.record.status == WorkStatus::Completed;
        self.release_lane(work_id);
        self.emit(
            if completed {
                WorkEventKind::Completed
            } else {
                WorkEventKind::Failed
            },
            work_id,
        );
        self.emit(WorkEventKind::NotificationPending, work_id);
        self.resolve_waiters(work_id);
        self.prune();
        self.drain();
    }

    // ── cancellation ────────────────────────────────────────────────────

    /// `cancel(id, {ownerId})` — `server/src/task/task-manager.mjs:707-772`.
    ///
    /// **Cancellation is a state, not an action.** `queued` and `scheduled`
    /// have nothing running, so they short-circuit straight to `cancelled`;
    /// everything else becomes `cancelling` and stays there until a
    /// confirmation arrives.
    fn cancel(
        &mut self,
        work_id: &str,
        owner_id: Option<&str>,
        reply: oneshot::Sender<Option<PublicWork>>,
    ) {
        let Some(entry) = self.works.get_mut(work_id) else {
            let _ = reply.send(None);
            return;
        };
        if !owns(entry, owner_id) {
            let _ = reply.send(None);
            return;
        }
        let previous = entry.record.status;
        if !previous.is_cancellable() && previous != WorkStatus::Cancelling {
            let _ = reply.send(None);
            return;
        }
        if entry.cancel_requested {
            // Join the cancellation already in flight — upstream returns the
            // same `cancelPromise` rather than starting a second one.
            entry.cancel_waiters.push(reply);
            return;
        }
        entry.cancel_requested = true;

        if matches!(previous, WorkStatus::Queued | WorkStatus::Scheduled) {
            entry.cancel_waiters.push(reply);
            self.finish_cancellation(work_id);
            return;
        }

        if !apply_status(
            &mut entry.record,
            WorkStatus::Cancelling,
            self.logger.as_ref(),
        ) {
            let _ = reply.send(None);
            return;
        }
        entry.record.authorization = None;
        entry.cancel_waiters.push(reply);
        let canceler = entry.canceler.clone();
        let signal = entry.signal.clone();
        let now = (self.now)();
        let snapshot = entry.record.to_snapshot(now);
        let delegation = entry.record.delegation.clone();

        self.emit(WorkEventKind::Cancelling, work_id);

        let reason = t(self.locale, keys::WORK_CANCELLED_BY_USER).to_owned();
        let Some(canceler) = canceler else {
            // No canceler: abort directly, and let the runner settling be the
            // confirmation.
            signal.abort(&reason);
            if let Some(entry) = self.works.get_mut(work_id) {
                entry.cancellation = Some(CancelOutcome::requested(
                    CancelRoute::Adapter,
                    cancel_target(delegation.as_ref()),
                ));
            }
            self.try_finish_cancellation(work_id);
            return;
        };

        let Some(commands) = self.commands.upgrade() else {
            return;
        };
        let request = CancelRequest::new(snapshot, previous, signal, reason);
        let id = work_id.to_owned();
        self.tracker.spawn(async move {
            let result = canceler.cancel(request).await;
            let _ = commands
                .send(Command::CancelerSettled {
                    work_id: id,
                    result: Box::new(result),
                })
                .await;
        });
    }

    fn canceler_settled(&mut self, work_id: &str, result: Result<CancelOutcome, RunFailure>) {
        match result {
            Ok(outcome) => {
                if let Some(entry) = self.works.get_mut(work_id) {
                    entry.cancellation = Some(outcome);
                }
                self.try_finish_cancellation(work_id);
            }
            Err(failure) => self.cancellation_failed(work_id, &failure.message),
        }
    }

    /// A confirmation arrived out of band.
    fn confirm_cancellation(
        &mut self,
        work_id: &str,
        confirmed_at: &str,
        reply: oneshot::Sender<Option<PublicWork>>,
    ) {
        let Some(entry) = self.works.get_mut(work_id) else {
            let _ = reply.send(None);
            return;
        };
        if entry.record.status != WorkStatus::Cancelling {
            let _ = reply.send(None);
            return;
        }
        let target = cancel_target(entry.record.delegation.as_ref());
        let confirmed = entry
            .cancellation
            .take()
            .unwrap_or_else(|| CancelOutcome::requested(CancelRoute::Adapter, target))
            .confirm(confirmed_at);
        entry.cancellation = Some(match confirmed {
            Ok(outcome) => outcome,
            // Already confirmed, or not confirmable — either way the Work is
            // finished below, which is what the caller asked for.
            Err(_) => CancelOutcome::Requested {
                route: CancelRoute::Adapter,
                target: cancel_target(entry.record.delegation.as_ref()),
            },
        });
        entry.cancel_runner_settled = true;
        entry.cancel_waiters.push(reply);
        self.finish_cancellation(work_id);
    }

    /// Whether both halves of the confirmation are in.
    ///
    /// A canceler that *confirmed* is enough on its own — the coordinator's own
    /// cancel tool returned, which is evidence the backend stopped. A canceler
    /// that only *requested* needs the abort it raised to have settled the
    /// runner. Until one of those holds, the Work stays `cancelling`, which is
    /// the whole of `docs/architecture.md` §4's *"cancellation is confirmed,
    /// not optimistic"*.
    fn try_finish_cancellation(&mut self, work_id: &str) {
        let ready = self.works.get(work_id).is_some_and(|entry| {
            entry.record.status == WorkStatus::Cancelling
                && entry
                    .cancellation
                    .as_ref()
                    .is_some_and(|outcome| outcome.is_confirmed() || entry.cancel_runner_settled)
        });
        if ready {
            self.finish_cancellation(work_id);
        }
    }

    /// `finishCancellation(task)` — `server/src/task/task-manager.mjs:774-798`.
    fn finish_cancellation(&mut self, work_id: &str) {
        let now = (self.now)();
        let Some(entry) = self.works.get_mut(work_id) else {
            return;
        };
        if !apply_status(
            &mut entry.record,
            WorkStatus::Cancelled,
            self.logger.as_ref(),
        ) {
            return;
        }
        entry.record.authorization = None;
        entry.record.completed_at = Some(now);
        entry.record.elapsed_ms = entry.record.started_at.map_or(0, |started| now - started);
        entry.record.error = None;
        // A cancelled Work announces nothing: the user asked for it to stop.
        entry.record.notification_status = NotificationStatus::None;
        entry.terminal_handled = true;
        entry.timers.cancel();
        self.release_lane(work_id);
        self.emit(WorkEventKind::Cancelled, work_id);
        self.resolve_waiters(work_id);
        self.prune();
        self.drain();
    }

    /// The canceler itself threw — `server/src/task/task-manager.mjs:742-770`.
    ///
    /// The Work **fails**; it is not reported as cancelled, because nothing
    /// confirmed that it stopped.
    fn cancellation_failed(&mut self, work_id: &str, detail: &str) {
        let now = (self.now)();
        let reason = t(self.locale, keys::WORK_CANCELLED_BY_USER).to_owned();
        let Some(entry) = self.works.get_mut(work_id) else {
            return;
        };
        entry.signal.abort(&reason);
        entry.timers.cancel();
        entry.record.authorization = None;
        if !apply_status(&mut entry.record, WorkStatus::Failed, self.logger.as_ref()) {
            return;
        }
        entry.record.error = Some(i18n_format(
            self.locale,
            keys::WORK_CANCEL_FAILED,
            &[("detail", detail)],
        ));
        entry.record.completed_at = Some(now);
        entry.record.elapsed_ms = entry.record.started_at.map_or(0, |started| now - started);
        entry.record.notification_status = NotificationStatus::Pending;
        entry.terminal_handled = true;
        self.release_lane(work_id);
        self.emit(WorkEventKind::Failed, work_id);
        self.emit(WorkEventKind::NotificationPending, work_id);
        self.resolve_waiters(work_id);
        self.prune();
        self.drain();
    }

    // ── recovery ────────────────────────────────────────────────────────

    fn recover_delegated(&mut self, recovery: &Arc<dyn DelegatedWorkRecovery>) -> usize {
        let now = (self.now)();
        let candidates = std::mem::take(&mut self.recovery_candidates);
        let mut recovered = 0;
        for work_id in candidates {
            let Some(entry) = self.works.get_mut(&work_id) else {
                continue;
            };
            let snapshot = entry.record.to_snapshot(now);
            if recovery.can_recover(&snapshot) {
                entry.runner = Some(Arc::new(RecoveryRunner {
                    recovery: Arc::clone(recovery),
                    snapshot: snapshot.clone(),
                }));
                entry.canceler = Some(Arc::new(RecoveryCanceler {
                    recovery: Arc::clone(recovery),
                    snapshot,
                }));
                // Upstream calls `start` directly rather than `drain`: a
                // reattached delegation is resumed regardless of admission,
                // because refusing it would strand a backend session nothing
                // is listening to.
                self.start(&work_id);
                recovered += 1;
                continue;
            }
            // Crash recovery, not a lifecycle transition: this record is from a
            // dead process until something accepts it, which is why
            // `via_protocol`'s graph has no `queued -> failed` edge.
            entry.record.status = WorkStatus::Failed;
            entry.record.error = Some(t(self.locale, keys::WORK_RESTART_DELEGATED_LOST).to_owned());
            entry.record.completed_at = Some(now);
            entry.record.notification_status = NotificationStatus::Pending;
            entry.terminal_handled = true;
            self.emit(WorkEventKind::Failed, &work_id);
            self.emit(WorkEventKind::NotificationPending, &work_id);
            self.resolve_waiters(&work_id);
        }
        recovered
    }

    // ── the reminder scheduler's two calls ──────────────────────────────

    fn fire_scheduled(&mut self, work_ids: &[String]) -> usize {
        let mut fired = Vec::new();
        for work_id in work_ids {
            let Some(entry) = self.works.get_mut(work_id) else {
                continue;
            };
            if entry.record.status != WorkStatus::Scheduled {
                continue;
            }
            if apply_status(&mut entry.record, WorkStatus::Queued, self.logger.as_ref()) {
                fired.push(work_id.clone());
            }
        }
        for work_id in &fired {
            self.emit(WorkEventKind::ScheduledFired, work_id);
        }
        if !fired.is_empty() {
            self.persist_deferred();
            self.drain();
        }
        fired.len()
    }

    // ── progress check ──────────────────────────────────────────────────

    fn progress_check(&mut self, work_id: &str) {
        let now = (self.now)();
        let Some(entry) = self.works.get(work_id) else {
            return;
        };
        if !entry.record.status.is_active() {
            return;
        }
        let message = progress_message(
            self.locale,
            &entry.record.objective,
            entry.record.started_at,
            now,
            entry.record.activity.last(),
        );

        let delegated =
            entry.record.status == WorkStatus::Delegated && entry.record.delegation.is_some();
        let Some(query) = self.coordinator_query.clone().filter(|_| delegated) else {
            let work = entry.record.to_public(now);
            self.publish(WorkEvent::progress_check(work, message, false));
            return;
        };

        let Some(commands) = self.commands.upgrade() else {
            return;
        };
        let owner_id = entry.record.owner_id.clone();
        let id = work_id.to_owned();
        self.tracker.spawn(async move {
            let answer = query.query(&id, &message, &owner_id).await;
            let command = match answer {
                Ok(outcome) if !outcome.content.trim().is_empty() => Command::ProgressCheckAnswer {
                    work_id: id,
                    message: outcome.content,
                    delegated: true,
                },
                // An empty answer falls back to the composed message but is
                // still *delegated*: the coordinator did reply.
                Ok(_) => Command::ProgressCheckAnswer {
                    work_id: id,
                    message,
                    delegated: true,
                },
                Err(_) => Command::ProgressCheckAnswer {
                    work_id: id,
                    message,
                    delegated: false,
                },
            };
            let _ = commands.send(command).await;
        });
    }

    // ── notifications ───────────────────────────────────────────────────

    fn claim_notifications(&mut self, request: &NotificationClaim) -> Vec<PublicWork> {
        self.reclaim_expired(true);
        let now = (self.now)();
        let requested: Option<Vec<&str>> = request
            .work_ids
            .as_ref()
            .filter(|ids| !ids.is_empty())
            .map(|ids| ids.iter().map(String::as_str).collect());
        let mut claimed = Vec::new();
        for entry in self.works.values_mut() {
            if entry.record.owner_id != request.owner_id
                || entry.record.notification_status != NotificationStatus::Pending
            {
                continue;
            }
            if let Some(session_id) = request.session_id.as_deref()
                && !request.include_other_sessions
                && entry.record.session_id != session_id
            {
                continue;
            }
            if requested
                .as_ref()
                .is_some_and(|ids| !ids.contains(&entry.record.id.as_str()))
            {
                continue;
            }
            entry.record.notification_status = NotificationStatus::Delivering;
            entry.record.notification_claimant_id = Some(request.claimant_id.clone());
            entry.record.notification_claimed_at = Some(now);
            claimed.push(entry.record.to_public(now));
        }
        if !claimed.is_empty() {
            self.persist();
        }
        claimed.sort_by_key(|work| work.created_at);
        claimed
    }

    fn mark_delivered(&mut self, work_ids: &[String], claimant_id: Option<&str>) -> usize {
        let now = (self.now)();
        let mut delivered = Vec::new();
        for work_id in work_ids {
            let Some(entry) = self.works.get_mut(work_id) else {
                continue;
            };
            if !holds_lease(entry, claimant_id) {
                continue;
            }
            entry.record.notification_status = NotificationStatus::Delivered;
            entry.record.notification_claimant_id = None;
            entry.record.notification_claimed_at = None;
            entry.record.notification_delivered_at = Some(now);
            delivered.push(work_id.clone());
        }
        for work_id in &delivered {
            self.emit(WorkEventKind::NotificationDelivered, work_id);
        }
        if !delivered.is_empty() {
            self.persist();
        }
        delivered.len()
    }

    fn renew_claims(&mut self, work_ids: &[String], claimant_id: Option<&str>) -> usize {
        let now = (self.now)();
        let mut renewed = 0;
        for work_id in work_ids {
            let Some(entry) = self.works.get_mut(work_id) else {
                continue;
            };
            if !holds_lease(entry, claimant_id) {
                continue;
            }
            entry.record.notification_claimed_at = Some(now);
            renewed += 1;
        }
        // Deliberately no persist: the lease is not persisted state.
        renewed
    }

    fn release_claims(&mut self, work_ids: &[String], claimant_id: Option<&str>) -> usize {
        let mut released = 0;
        for work_id in work_ids {
            let Some(entry) = self.works.get_mut(work_id) else {
                continue;
            };
            if !holds_lease(entry, claimant_id) {
                continue;
            }
            entry.record.notification_status = NotificationStatus::Pending;
            entry.record.notification_claimant_id = None;
            entry.record.notification_claimed_at = None;
            released += 1;
        }
        if released > 0 {
            self.persist();
        }
        released
    }

    fn reclaim_expired(&mut self, persist: bool) -> usize {
        let now = (self.now)();
        let ttl = self.retention.notification_claim_ttl_ms;
        let mut reclaimed = 0;
        for entry in self.works.values_mut() {
            if entry.record.notification_status != NotificationStatus::Delivering {
                continue;
            }
            let Some(claimed_at) = entry.record.notification_claimed_at else {
                continue;
            };
            if now - claimed_at < ttl {
                continue;
            }
            entry.record.notification_status = NotificationStatus::Pending;
            entry.record.notification_claimant_id = None;
            entry.record.notification_claimed_at = None;
            reclaimed += 1;
        }
        if reclaimed > 0 && persist {
            self.persist();
        }
        reclaimed
    }

    // ── retention ───────────────────────────────────────────────────────

    /// `prune()` — `server/src/task/task-manager.mjs:941-974`.
    ///
    /// Two rules, in this order:
    ///
    /// 1. **Age.** A terminal Work older than its TTL is dropped. Which TTL
    ///    depends on whether a notification is still owed, and the owed one is
    ///    seven days against the ordinary twenty-four hours.
    /// 2. **Count.** Beyond that, an owner keeps at most
    ///    `max_terminal_tasks_per_owner` terminal Work items, newest first —
    ///    but **only counting ones with nothing owed**. A pending notification
    ///    is never evicted to make room for history.
    fn prune(&mut self) -> usize {
        let now = (self.now)();
        let mut changed = self.reclaim_expired(false) > 0;
        let mut dropped: Vec<String> = Vec::new();
        let mut terminal_by_owner: IndexMap<String, Vec<(i64, String)>> = IndexMap::new();

        for entry in self.works.values() {
            if !entry.record.status.is_terminal() {
                continue;
            }
            let age = now - entry.record.completed_at.unwrap_or(entry.record.created_at);
            let awaiting = entry.record.notification_status.is_awaiting_delivery();
            if age > self.retention.ttl_for(awaiting) {
                dropped.push(entry.record.id.clone());
                continue;
            }
            if awaiting {
                continue;
            }
            terminal_by_owner
                .entry(entry.record.owner_id.clone())
                .or_default()
                .push((entry.record.created_at, entry.record.id.clone()));
        }

        let cap =
            usize::try_from(self.retention.max_terminal_tasks_per_owner).unwrap_or(usize::MAX);
        for mut items in terminal_by_owner.into_values() {
            if items.len() <= cap {
                continue;
            }
            items.sort_by(|left, right| right.0.cmp(&left.0));
            for (_, work_id) in items.into_iter().skip(cap) {
                dropped.push(work_id);
            }
        }

        for work_id in &dropped {
            if let Some(entry) = self.works.shift_remove(work_id) {
                entry.timers.cancel();
                changed = true;
            }
        }
        if changed {
            self.persist();
        }
        dropped.len()
    }

    fn list(&mut self, query: &WorkQuery) -> Vec<PublicWork> {
        self.prune();
        let now = (self.now)();
        let mut found: Vec<PublicWork> = self
            .works
            .values()
            .filter(|entry| {
                query
                    .owner_id
                    .as_deref()
                    .is_none_or(|owner| entry.record.owner_id == owner)
                    && query
                        .session_id
                        .as_deref()
                        .is_none_or(|session| entry.record.session_id == session)
                    && (!query.active_only || entry.record.status.is_active())
                    && (query.include_control || entry.record.kind != WorkKind::Control)
            })
            .map(|entry| entry.record.to_public(now))
            .collect();
        found.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        found
    }

    // ── plumbing ────────────────────────────────────────────────────────

    fn release_lane(&mut self, work_id: &str) {
        if let Some(entry) = self.works.get_mut(work_id)
            && entry.scheduler_held
        {
            entry.scheduler_held = false;
            self.scheduler.release(work_id);
        }
    }

    fn resolve_waiters(&mut self, work_id: &str) {
        let now = (self.now)();
        let Some(entry) = self.works.get_mut(work_id) else {
            return;
        };
        let work = entry.record.to_public(now);
        for waiter in entry.waiters.drain(..) {
            let _ = waiter.send(Some(work.clone()));
        }
        for waiter in entry.cancel_waiters.drain(..) {
            let _ = waiter.send(Some(work.clone()));
        }
    }

    fn emit(&mut self, kind: WorkEventKind, work_id: &str) {
        let now = (self.now)();
        let Some(work) = self
            .works
            .get(work_id)
            .map(|entry| entry.record.to_public(now))
        else {
            return;
        };
        self.publish(WorkEvent::new(kind, work));
    }

    fn publish(&mut self, event: WorkEvent) {
        if let Some(logger) = &self.logger {
            logger.emit(
                event.kind.log_level(),
                event.kind.as_str(),
                via_log::fields([
                    ("taskId", event.task.id.clone().into()),
                    ("ownerId", event.task.owner_id.clone().into()),
                    ("sessionId", event.task.session_id.clone().into()),
                    ("turnId", event.task.turn_id.clone().into()),
                    ("kind", event.task.kind.as_str().into()),
                    ("status", event.task.status.as_str().into()),
                    ("elapsedMs", event.task.elapsed_ms.into()),
                    ("hasError", event.task.error.is_some().into()),
                ]),
                "",
            );
        }
        let persists = event.kind.persists();
        // One observer must not break the work queue: a broadcast with no
        // receivers is not an error.
        let _ = self.events.send(event);
        if persists {
            self.persist();
        }
    }

    fn persisted(&self) -> Vec<PersistedWork> {
        let now = (self.now)();
        self.works
            .values()
            .map(|entry| entry.record.to_persisted(now))
            .collect()
    }

    fn persist(&self) -> bool {
        self.store.save(&self.persisted())
    }

    fn persist_deferred(&self) {
        self.store.save_deferred(&self.persisted());
    }
}

/// Whether `owner_id` may see this Work. `None` is "no filter".
fn owns(entry: &Entry, owner_id: Option<&str>) -> bool {
    owner_id.is_none_or(|owner| entry.record.owner_id == owner)
}

/// Whether `claimant_id` currently holds this Work's delivery lease.
fn holds_lease(entry: &Entry, claimant_id: Option<&str>) -> bool {
    entry.record.notification_status == NotificationStatus::Delivering
        && claimant_id.is_none_or(|claimant| {
            entry.record.notification_claimant_id.as_deref() == Some(claimant)
        })
}

/// `String(sessionId || 'main')`.
fn session_or_default(session_id: Option<String>) -> String {
    session_id
        .filter(|session| !session.is_empty())
        .unwrap_or_else(|| DEFAULT_SESSION_ID.to_owned())
}

/// `Math.round(timeoutMs / 60000)`.
fn round_minutes(timeout_ms: i64) -> i64 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "a millisecond budget is exact in f64"
    )]
    let minutes = timeout_ms as f64 / 60_000.0;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a millisecond budget divided by 60000 fits i64"
    )]
    let rounded = minutes.round() as i64;
    rounded
}

/// What a cancel is aimed at, for the outcome the manager records.
fn cancel_target(delegation: Option<&DelegationRef>) -> CancelTarget {
    delegation.map_or_else(CancelTarget::default, |delegation| CancelTarget {
        delegation_id: Some(delegation.id.clone()),
        session_id: Some(delegation.session_id.clone()),
    })
}

/// Move a Work to `next`, refusing an edge the lifecycle graph does not have.
///
/// The graph is `via_protocol::WorkStatus::can_transition_to` and is not
/// restated here. A refusal is a wiring bug — it means something tried to
/// publish a state the record cannot be in — so it is logged and dropped rather
/// than applied.
fn apply_status(record: &mut WorkRecord, next: WorkStatus, logger: Option<&Logger>) -> bool {
    match record.status.transition_to(next) {
        Ok(next) => {
            record.status = next;
            true
        }
        Err(error) => {
            if let Some(logger) = logger {
                logger.emit(
                    LogLevel::Warn,
                    "task.illegal_transition",
                    via_log::fields([
                        ("taskId", record.id.clone().into()),
                        ("from", record.status.as_str().into()),
                        ("to", next.as_str().into()),
                    ]),
                    &error.to_string(),
                );
            }
            false
        }
    }
}

/// The persisted record, back in memory, before restart recovery rewrites it.
fn restore_record(saved: PersistedWork) -> WorkRecord {
    WorkRecord {
        id: saved.id,
        status: saved.status,
        kind: saved.kind,
        parent_work_id: saved.parent_work_id,
        // Priority and lane do not survive a restart, which is upstream's
        // stated consequence: lane serialization is re-established by whatever
        // resubmits, not by the store.
        priority: 0,
        objective: saved.objective,
        owner_id: saved.owner_id,
        session_id: saved.session_id,
        turn_id: saved.turn_id,
        submission_key: saved.submission_key,
        lane_key: None,
        lane_limit: crate::scheduler::COORDINATOR_LANE_LIMIT,
        created_at: saved.created_at,
        started_at: saved.started_at,
        completed_at: saved.completed_at,
        elapsed_ms: saved.elapsed_ms,
        result: saved.result,
        error: saved.error,
        result_metadata: saved.result_metadata,
        activity: saved.activity,
        delegation: saved.delegation,
        authorization: saved.authorization,
        notification_status: saved.notification_status,
        notification_claimant_id: None,
        notification_claimed_at: None,
        notification_delivered_at: saved.notification_delivered_at,
        schedule: saved.schedule,
        timeout_ms: saved.timeout_ms,
        progress_check_ms: saved.progress_check_ms,
    }
}

/// The store health a stopped manager reports.
pub(super) const fn unavailable_health() -> StoreHealth {
    StoreHealth {
        ok: false,
        persistence_enabled: false,
        warning: None,
    }
}
