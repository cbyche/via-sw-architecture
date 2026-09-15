//! Deterministic doubles for the Work seams.
//!
//! `via-coordinator`, `via-mcp-tools` and `via-app` all need a runner that does
//! what a test tells it to, and three private copies of one would drift. They
//! live beside the seams they double, exactly as
//! [`via_downstream::testing::ScriptedHarness`] does.
//!
//! Nothing here reads a clock or spawns a task of its own: a
//! [`GatedRunner`] parks until it is released, and everything else answers
//! immediately.
//!
//! Behind the `testing` feature, which is **on by default** so a sibling crate
//! reaches it without restating the feature.

use std::sync::{Arc, Mutex, MutexGuard};

use async_trait::async_trait;
use tokio::sync::Notify;
use via_downstream::{CancelOutcome, CancelRoute, CancelTarget};
use via_protocol::{WorkKind, WorkStatus};

use crate::delegation::DelegationRef;
use crate::record::{NotificationStatus, WorkRecord};
use crate::runner::{
    CancelRequest, CoordinatorQuery, RunFailure, RunnerEvent, WorkCanceler, WorkContext,
    WorkEventSink, WorkOutcome, WorkRunner,
};

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// A [`WorkRecord`] with every field at its zero value.
///
/// For tests that need a record without going through the manager. `status` is
/// [`WorkStatus::Queued`] and `kind` is [`WorkKind::Work`], which is what
/// `create()` mints.
#[must_use]
pub fn blank_record(work_id: &str, owner_id: &str) -> WorkRecord {
    WorkRecord {
        id: work_id.to_owned(),
        status: WorkStatus::Queued,
        kind: WorkKind::Work,
        parent_work_id: None,
        priority: 0,
        objective: String::new(),
        owner_id: owner_id.to_owned(),
        session_id: crate::record::DEFAULT_SESSION_ID.to_owned(),
        turn_id: None,
        submission_key: None,
        lane_key: None,
        lane_limit: crate::scheduler::COORDINATOR_LANE_LIMIT,
        created_at: 0,
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
        progress_check_ms: None,
    }
}

/// A runner that finishes at once with a fixed result.
#[derive(Debug, Clone)]
pub struct ImmediateRunner {
    outcome: Result<WorkOutcome, RunFailure>,
    started: Arc<Mutex<Vec<String>>>,
}

impl ImmediateRunner {
    /// Completes with `content`.
    #[must_use]
    pub fn completing(content: &str) -> Self {
        Self {
            outcome: Ok(WorkOutcome::content(content)),
            started: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Completes with `content` and `metadata`.
    #[must_use]
    pub fn completing_with(content: &str, metadata: serde_json::Value) -> Self {
        Self {
            outcome: Ok(WorkOutcome::content(content).with_metadata(metadata)),
            started: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Fails with `message`.
    #[must_use]
    pub fn failing(message: &str) -> Self {
        Self {
            outcome: Err(RunFailure::new(message)),
            started: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Every objective this runner was given, in order.
    #[must_use]
    pub fn started(&self) -> Vec<String> {
        lock(&self.started).clone()
    }

    /// How many times it ran.
    #[must_use]
    pub fn runs(&self) -> usize {
        lock(&self.started).len()
    }
}

#[async_trait]
impl WorkRunner for ImmediateRunner {
    async fn run(
        &self,
        objective: String,
        _context: WorkContext,
    ) -> Result<WorkOutcome, RunFailure> {
        lock(&self.started).push(objective);
        self.outcome.clone()
    }
}

/// A runner that parks until it is released, so a test can observe a Work
/// mid-flight.
///
/// Every gate is independent: releasing one does not release another.
#[derive(Debug, Clone, Default)]
pub struct GatedRunner {
    state: Arc<GateState>,
}

#[derive(Debug, Default)]
struct GateState {
    notify: Notify,
    inner: Mutex<GateInner>,
}

#[derive(Debug, Default)]
struct GateInner {
    released: bool,
    outcome: Option<Result<WorkOutcome, RunFailure>>,
    started: Vec<String>,
    emit_on_start: Vec<RunnerEvent>,
    observed_abort: bool,
    honours_abort: bool,
    sink: Option<WorkEventSink>,
}

impl GatedRunner {
    /// A runner that parks, and returns as soon as it is aborted.
    #[must_use]
    pub fn new() -> Self {
        let runner = Self::default();
        lock(&runner.state.inner).honours_abort = true;
        runner
    }

    /// A runner that *notices* its abort and keeps running anyway.
    ///
    /// The adversarial case for confirmed cancellation: the abort was
    /// delivered, and nothing has confirmed that anything stopped. A Work with
    /// this runner stays `cancelling` until [`Self::settle`] releases it.
    #[must_use]
    pub fn stubborn() -> Self {
        Self::default()
    }

    /// Emit these events before parking, in order.
    #[must_use]
    pub fn emitting(self, events: Vec<RunnerEvent>) -> Self {
        lock(&self.state.inner).emit_on_start = events;
        self
    }

    /// Release with a completed result.
    pub fn complete(&self, content: &str) {
        self.settle(Ok(WorkOutcome::content(content)));
    }

    /// Release with a failure.
    pub fn fail(&self, message: &str) {
        self.settle(Err(RunFailure::new(message)));
    }

    /// Release with an explicit outcome.
    pub fn settle(&self, outcome: Result<WorkOutcome, RunFailure>) {
        {
            let mut inner = lock(&self.state.inner);
            inner.released = true;
            inner.outcome = Some(outcome);
        }
        self.state.notify.notify_waiters();
    }

    /// Whether the runner has started at least once.
    #[must_use]
    pub fn is_running(&self) -> bool {
        !lock(&self.state.inner).started.is_empty()
    }

    /// The event sink the Work handed this runner, once it has started.
    ///
    /// The way a test emits a [`RunnerEvent`] at a moment of its own choosing —
    /// including *after* the Work has settled, which is the case upstream gets
    /// wrong.
    #[must_use]
    pub fn sink(&self) -> Option<WorkEventSink> {
        lock(&self.state.inner).sink.clone()
    }

    /// Emit one event through the sink this runner was given.
    ///
    /// # Panics
    ///
    /// If the runner has not started yet, so a test cannot silently assert
    /// nothing.
    pub async fn emit(&self, event: RunnerEvent) {
        let sink = self
            .sink()
            .expect("the runner must have started before it can emit");
        sink.emit(event).await;
    }

    /// Whether the runner saw its Work's abort signal fire.
    #[must_use]
    pub fn observed_abort(&self) -> bool {
        lock(&self.state.inner).observed_abort
    }

    /// How many times it started.
    #[must_use]
    pub fn runs(&self) -> usize {
        lock(&self.state.inner).started.len()
    }
}

#[async_trait]
impl WorkRunner for GatedRunner {
    async fn run(
        &self,
        objective: String,
        context: WorkContext,
    ) -> Result<WorkOutcome, RunFailure> {
        let events = {
            let mut inner = lock(&self.state.inner);
            inner.started.push(objective);
            inner.sink = Some(context.events.clone());
            std::mem::take(&mut inner.emit_on_start)
        };
        for event in events {
            context.events.emit(event).await;
        }
        loop {
            let waiter = self.state.notify.notified();
            if let Some(outcome) = lock(&self.state.inner).outcome.clone() {
                return outcome;
            }
            tokio::select! {
                () = waiter => {}
                () = context.signal.aborted() => {
                    let honours = {
                        let mut inner = lock(&self.state.inner);
                        inner.observed_abort = true;
                        inner.honours_abort
                    };
                    if honours {
                        // A cancelled turn is an error, never a completed one:
                        // `via_downstream::PromptOutcome::new` refuses the same
                        // thing at the Layer-3 seam.
                        return Err(RunFailure::new(
                            context.signal.reason().unwrap_or_default(),
                        ));
                    }
                    // Wait to be released, so a test can observe a Work that is
                    // `cancelling` and has confirmed nothing.
                    let released = self.state.notify.notified();
                    if lock(&self.state.inner).outcome.is_none() {
                        released.await;
                    }
                }
            }
        }
    }
}

/// A runner that never returns and never observes its abort signal.
///
/// The adversarial case for confirmed cancellation: the Work stays
/// `cancelling` because nothing ever confirms that it stopped.
#[derive(Debug, Clone, Default)]
pub struct DeafRunner;

#[async_trait]
impl WorkRunner for DeafRunner {
    async fn run(
        &self,
        _objective: String,
        _context: WorkContext,
    ) -> Result<WorkOutcome, RunFailure> {
        std::future::pending().await
    }
}

/// A canceler that answers with a fixed outcome, recording what it was asked.
#[derive(Debug, Clone)]
pub struct ScriptedCanceler {
    outcome: Result<CancelOutcome, RunFailure>,
    abort: bool,
    seen: Arc<Mutex<Vec<WorkStatus>>>,
}

impl ScriptedCanceler {
    /// Aborts the runner and reports a *requested* — unconfirmed — cancel.
    ///
    /// The honest answer for a fire-and-forget transport, and the one that
    /// leaves the Work `cancelling` until the abort settles the runner.
    #[must_use]
    pub fn requesting() -> Self {
        Self {
            outcome: Ok(CancelOutcome::requested(
                CancelRoute::Adapter,
                CancelTarget::default(),
            )),
            abort: true,
            seen: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Reports a confirmed cancel without aborting anything.
    ///
    /// The coordinator route: the model's own cancel tool returned, which is
    /// itself the confirmation.
    #[must_use]
    pub fn confirming(confirmed_at: &str) -> Self {
        let outcome = CancelOutcome::requested(CancelRoute::Coordinator, CancelTarget::default())
            .confirm(confirmed_at)
            .unwrap_or(CancelOutcome::NotFound);
        Self {
            outcome: Ok(outcome),
            abort: false,
            seen: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Fails, which makes the Work fail with `取消失败：…`.
    #[must_use]
    pub fn failing(message: &str) -> Self {
        Self {
            outcome: Err(RunFailure::new(message)),
            abort: false,
            seen: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Also abort the runner. `requesting` already does.
    #[must_use]
    pub const fn aborting(mut self, abort: bool) -> Self {
        self.abort = abort;
        self
    }

    /// The `previous_status` of every cancel this canceler was asked for.
    #[must_use]
    pub fn seen(&self) -> Vec<WorkStatus> {
        lock(&self.seen).clone()
    }
}

#[async_trait]
impl WorkCanceler for ScriptedCanceler {
    async fn cancel(&self, request: CancelRequest) -> Result<CancelOutcome, RunFailure> {
        lock(&self.seen).push(request.previous_status);
        if self.abort {
            request.abort();
        }
        self.outcome.clone()
    }
}

/// A coordinator status query that answers with a fixed sentence.
#[derive(Debug, Clone)]
pub struct ScriptedCoordinatorQuery {
    answer: Result<WorkOutcome, RunFailure>,
    asked: Arc<Mutex<Vec<(String, String)>>>,
}

impl ScriptedCoordinatorQuery {
    /// Answers `content`.
    #[must_use]
    pub fn answering(content: &str) -> Self {
        Self {
            answer: Ok(WorkOutcome::content(content)),
            asked: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Refuses.
    #[must_use]
    pub fn refusing(message: &str) -> Self {
        Self {
            answer: Err(RunFailure::new(message)),
            asked: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Every `(work_id, question)` this query was asked, in order.
    #[must_use]
    pub fn asked(&self) -> Vec<(String, String)> {
        lock(&self.asked).clone()
    }
}

#[async_trait]
impl CoordinatorQuery for ScriptedCoordinatorQuery {
    async fn query(
        &self,
        work_id: &str,
        question: &str,
        _owner_id: &str,
    ) -> Result<WorkOutcome, RunFailure> {
        lock(&self.asked).push((work_id.to_owned(), question.to_owned()));
        self.answer.clone()
    }
}

/// A delegation an adapter would report.
#[must_use]
pub fn delegation(id: &str, session_id: &str) -> DelegationRef {
    DelegationRef::new(id, session_id).with_title("project")
}
