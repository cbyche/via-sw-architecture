//! The owning task's command vocabulary.
//!
//! `docs/architecture.md` §11:
//!
//! > Each gets an owning task, not a mutex: `tokio::sync::Mutex` gives mutual
//! > exclusion but not FIFO order, and three of the four need order. State held
//! > by value in one task, `enum Command { … reply: oneshot::Sender<R> }` over a
//! > bounded `mpsc`, shutdown by dropping senders.
//!
//! This is that enum. Every mutation of every Work goes through it, including
//! the ones a runner raises and the ones a timer raises, so "the queue admitted
//! A before B" and "A's activity was recorded before B started" are the same
//! statement.
//!
//! It is crate-private: the public surface is [`WorkManager`](crate::WorkManager),
//! whose methods are one command each.

use std::sync::Arc;

use tokio::sync::oneshot;
use via_downstream::CancelOutcome;
use via_store::StoreHealth;

use crate::limits::RetentionPolicy;
use crate::manager::{
    NewScheduledWork, NewWork, NotificationClaim, ScheduledWork, WorkAcceptance, WorkQuery,
};
use crate::reconcile::CancellationFact;
use crate::record::PublicWork;
use crate::runner::{
    CoordinatorQuery, DelegatedWorkRecovery, RunFailure, RunnerEvent, WorkOutcome, WorkRunner,
};

/// One instruction for the Work manager's owning task.
pub(crate) enum Command {
    // ── the public surface ──────────────────────────────────────────────
    Create {
        request: Box<NewWork>,
        reply: oneshot::Sender<WorkAcceptance>,
    },
    CreateScheduled {
        request: Box<NewScheduledWork>,
        reply: oneshot::Sender<WorkAcceptance>,
    },
    Get {
        work_id: String,
        owner_id: Option<String>,
        reply: oneshot::Sender<Option<PublicWork>>,
    },
    List {
        query: WorkQuery,
        reply: oneshot::Sender<Vec<PublicWork>>,
    },
    Cancel {
        work_id: String,
        owner_id: Option<String>,
        reply: oneshot::Sender<Option<PublicWork>>,
    },
    ConfirmCancellation {
        work_id: String,
        confirmed_at: String,
        reply: oneshot::Sender<Option<PublicWork>>,
    },
    Wait {
        work_id: String,
        reply: oneshot::Sender<Option<PublicWork>>,
    },

    // ── notifications ───────────────────────────────────────────────────
    ClaimNotifications {
        request: Box<NotificationClaim>,
        reply: oneshot::Sender<Vec<PublicWork>>,
    },
    MarkDelivered {
        work_ids: Vec<String>,
        claimant_id: Option<String>,
        reply: oneshot::Sender<usize>,
    },
    RenewClaims {
        work_ids: Vec<String>,
        claimant_id: Option<String>,
        reply: oneshot::Sender<usize>,
    },
    ReleaseClaims {
        work_ids: Vec<String>,
        claimant_id: Option<String>,
        reply: oneshot::Sender<usize>,
    },
    ReclaimExpiredClaims {
        reply: oneshot::Sender<usize>,
    },

    // ── configuration ───────────────────────────────────────────────────
    ConfigureRetention {
        policy: RetentionPolicy,
        reply: oneshot::Sender<()>,
    },
    ConfigureScheduledTaskRunner {
        runner: Arc<dyn WorkRunner>,
        reply: oneshot::Sender<()>,
    },
    ConfigureCoordinatorQuery {
        query: Arc<dyn CoordinatorQuery>,
        reply: oneshot::Sender<()>,
    },
    RecoverDelegated {
        recovery: Arc<dyn DelegatedWorkRecovery>,
        reply: oneshot::Sender<usize>,
    },

    // ── maintenance ─────────────────────────────────────────────────────
    Prune {
        reply: oneshot::Sender<usize>,
    },
    Persist {
        reply: oneshot::Sender<bool>,
    },
    Flush {
        reply: oneshot::Sender<bool>,
    },
    StoreHealth {
        reply: oneshot::Sender<StoreHealth>,
    },

    // ── the reminder scheduler ──────────────────────────────────────────
    ScheduledSnapshot {
        reply: oneshot::Sender<Vec<ScheduledWork>>,
    },
    FireScheduled {
        work_ids: Vec<String>,
        reply: oneshot::Sender<usize>,
    },

    // ── the reconciliation ledger ───────────────────────────────────────
    RecordCancellationFact {
        owner_id: String,
        fact: Box<CancellationFact>,
        reply: oneshot::Sender<()>,
    },
    PendingFacts {
        owner_id: String,
        reply: oneshot::Sender<Vec<CancellationFact>>,
    },
    ClearFacts {
        owner_id: String,
        reply: oneshot::Sender<usize>,
    },

    // ── raised by runners, cancelers and timers ─────────────────────────
    RunnerEvent {
        work_id: String,
        event: RunnerEvent,
    },
    RunnerSettled {
        work_id: String,
        result: Box<Result<WorkOutcome, RunFailure>>,
    },
    CancelerSettled {
        work_id: String,
        result: Box<Result<CancelOutcome, RunFailure>>,
    },
    /// The one-second liveness tick.
    ProgressTick {
        work_id: String,
    },
    /// The long-running announcement is due.
    ProgressCheckTick {
        work_id: String,
    },
    /// The coordinator answered — or refused to answer — a delegated Work's
    /// progress query.
    ProgressCheckAnswer {
        work_id: String,
        message: String,
        delegated: bool,
    },
    /// A `scheduled_task` exhausted its wall-clock budget: abort it.
    TimeoutFired {
        work_id: String,
    },
    /// The cleanup window after that abort closed: force-fail it.
    TimeoutCleanup {
        work_id: String,
    },
}
