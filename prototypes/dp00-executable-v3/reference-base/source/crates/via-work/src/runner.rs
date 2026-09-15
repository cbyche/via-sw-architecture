//! The seams: what runs a Work, what stops one, and what the runner may say
//! while it does.
//!
//! Upstream passes four closures into `TaskManager` — `runner`, `canceler`,
//! `scheduledTaskRunner` and `coordinatorQueryDelegatedWork` — and every one of
//! them is supplied by the coordinator (`gateway-application.mjs:73-140`,
//! `tool-call-handler.mjs:221-263`). They are traits here for the same reason
//! `via-downstream` has traits: the Work subsystem must be able to run without
//! a coordinator at all, which is what every test in this crate does.
//!
//! # What a runner may say
//!
//! [`RunnerEvent`] is upstream's `onEvent` vocabulary
//! (`server/src/task/task-manager.mjs:491-537`), and it is closed. Five
//! variants, two of which carry a [`DelegationRef`] and two a
//! [`PendingPermission`]; the fifth carries a `via_downstream::SessionEvent`,
//! which has already been through Layer 3's projection and so cannot carry a
//! session id, a sub-agent id, a permission payload or any reasoning.
//! `docs/architecture.md` §17, question 4: *"Are tool events used only for
//! generic progress?"* — by construction.
//!
//! # Abort carries a reason
//!
//! [`AbortSignal`] is a [`CancellationToken`] plus the reason the abort was
//! raised with, because upstream's reason is not diagnostic: it reaches the ACP
//! adapter and is what a backend is told. `用户已取消这项工作` and
//! `定时任务执行超时，正在终止` are both catalogued strings, and both arrive
//! this way.
//!
//! # Work items own their own cancellation scope
//!
//! `docs/architecture.md` §11 records the ARGO defect this design avoids: a
//! cloned `CancellationToken` shares one `Arc<AtomicBool>`, so a detached
//! sub-agent that captured the parent turn's token dies when one barge-in
//! cancels that turn. Every Work here is given a token of its **own**, minted
//! when it starts and never derived from a caller's.

use std::future::Future;
use std::sync::{Arc, Mutex, MutexGuard};

use async_trait::async_trait;
use serde_json::Value;
use tokio_util::sync::CancellationToken;
use via_downstream::{CancelOutcome, SessionEvent};
use via_protocol::{WorkKind, WorkStatus};

use crate::command::Command;
use crate::delegation::DelegationRef;
use crate::permission::PendingPermission;
use crate::record::WorkSnapshot;
use crate::schedule::Schedule;

/// Lock, treating poisoning as "the value is still there".
///
/// The only thing behind this lock is an abort reason string. Refusing to
/// report it because some other task panicked would turn one panic into a Work
/// that can never be cancelled.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// A Work's own cancellation scope.
///
/// `AbortController`'s `signal`, with `reason` — which upstream relies on
/// (`signal.reason` is read by the ACP adapter's abort listener) and which
/// [`CancellationToken`] alone does not carry.
#[derive(Debug, Clone)]
pub struct AbortSignal {
    token: CancellationToken,
    reason: Arc<Mutex<Option<String>>>,
}

impl Default for AbortSignal {
    fn default() -> Self {
        Self::new()
    }
}

impl AbortSignal {
    /// A fresh, un-aborted signal.
    #[must_use]
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            reason: Arc::new(Mutex::new(None)),
        }
    }

    /// Abort with `reason`.
    ///
    /// The first reason wins: `AbortController.abort` is a no-op once the
    /// signal is already aborted, so a timeout that fires while a user
    /// cancellation is in flight does not overwrite what the backend is told.
    pub fn abort(&self, reason: &str) {
        if self.token.is_cancelled() {
            return;
        }
        *lock(&self.reason) = Some(reason.to_owned());
        self.token.cancel();
    }

    /// Whether the Work has been asked to stop.
    #[must_use]
    pub fn is_aborted(&self) -> bool {
        self.token.is_cancelled()
    }

    /// Why it was asked to stop, once it has been.
    #[must_use]
    pub fn reason(&self) -> Option<String> {
        lock(&self.reason).clone()
    }

    /// Resolves when the Work is asked to stop.
    ///
    /// Already-aborted resolves immediately.
    pub async fn aborted(&self) {
        self.token.cancelled().await;
    }

    /// The underlying token, for a runner that must pass one further down.
    ///
    /// Cloning it is safe *here*: every clone belongs to this Work and to no
    /// other, which is the property `docs/architecture.md` §11 asks for.
    #[must_use]
    pub const fn token(&self) -> &CancellationToken {
        &self.token
    }
}

/// Everything a runner is told about the Work it is running.
///
/// `taskExecutionContext` — `server/src/task/task-manager.mjs:18-29`, frozen
/// there and immutable here. Note what is *not* on it: no backend session, no
/// harness, no permission decision. `docs/architecture.md` §6 — *"a harness
/// answers prompts and emits events; it never owns a queue, a `work_id`, or a
/// permission decision"* — and the same is true in the other direction.
#[derive(Debug, Clone)]
pub struct WorkContext {
    /// The Work id. Upstream's `taskId`, and the coordinator's
    /// `coordinationRunId`.
    pub work_id: String,
    /// Whose Work it is.
    pub owner_id: String,
    /// Which session raised it; `main` when none was named.
    pub session_id: String,
    /// Which turn raised it.
    pub turn_id: Option<String>,
    /// Which of the four kinds.
    pub kind: WorkKind,
    /// The due time, for a `reminder` or `scheduled_task`.
    pub schedule: Option<Schedule>,
    /// Where progress goes.
    pub events: WorkEventSink,
    /// This Work's own cancellation scope.
    pub signal: AbortSignal,
}

/// Where a runner's progress goes.
///
/// Every event is delivered to the manager's owning task over its bounded
/// command channel, so a runner's events are ordered against every other
/// mutation rather than racing them.
#[derive(Debug, Clone)]
pub struct WorkEventSink {
    work_id: String,
    commands: tokio::sync::mpsc::Sender<Command>,
}

impl WorkEventSink {
    pub(crate) const fn new(work_id: String, commands: tokio::sync::mpsc::Sender<Command>) -> Self {
        Self { work_id, commands }
    }

    /// Report progress.
    ///
    /// Returns `false` when the manager has shut down, which is the only way
    /// this can fail. Upstream's `onEvent` returns nothing and swallows
    /// everything; a runner that wants to stop early when nobody is listening
    /// can, and one that ignores the answer behaves exactly as upstream does.
    pub async fn emit(&self, event: RunnerEvent) -> bool {
        self.commands
            .send(Command::RunnerEvent {
                work_id: self.work_id.clone(),
                event,
            })
            .await
            .is_ok()
    }

    /// The Work these events belong to.
    #[must_use]
    pub fn work_id(&self) -> &str {
        &self.work_id
    }
}

/// What a runner may report while it runs.
///
/// Closed, and each variant is one of upstream's `backend.*` event types.
#[derive(Debug, Clone, PartialEq)]
pub enum RunnerEvent {
    /// `backend.activity` — generic tool progress, already projected by
    /// Layer 3.
    Activity(SessionEvent),
    /// `backend.delegated` — the coordinator handed the objective to a project
    /// session. The Work becomes `delegated` and **releases its scheduler
    /// lane**, so the next Work in the lane starts while this one waits.
    Delegated(DelegationRef),
    /// `backend.delegation.completed` — the delegated session finished.
    ///
    /// Accepted only when it correlates with the delegation the Work is
    /// actually waiting on; see [`DelegationRef::correlates_with`].
    DelegationCompleted(DelegationRef),
    /// `backend.permission.requested` — a backend wants authorization.
    PermissionRequested(PendingPermission),
    /// `backend.permission.resolved` — the decision landed.
    PermissionResolved(PendingPermission),
}

/// What a finished runner produced.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorkOutcome {
    /// The result text. Trimmed by the manager, exactly as
    /// `String(outcome?.content ?? outcome ?? '').trim()` does.
    pub content: String,
    /// The runner's own metadata, raw.
    ///
    /// Only its `presentation` (or legacy `decision.presentation`) is ever
    /// published or persisted — see [`crate::presentation`].
    pub metadata: Option<Value>,
}

impl WorkOutcome {
    /// An outcome with content and no metadata.
    #[must_use]
    pub fn content(content: &str) -> Self {
        Self {
            content: content.to_owned(),
            metadata: None,
        }
    }

    /// Attach metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Why a runner or canceler stopped.
///
/// Carries an **already-localized** sentence, because the runner is the one
/// that knows what failed. It becomes `task.error` verbatim, is persisted, and
/// is spoken to the user.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct RunFailure {
    /// The sentence.
    pub message: String,
}

impl RunFailure {
    /// A failure carrying `message`.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<via_downstream::HarnessError> for RunFailure {
    /// A harness refusal is a Work failure whose message is the harness's.
    fn from(error: via_downstream::HarnessError) -> Self {
        Self::new(error.to_string())
    }
}

/// Something that can carry out a Work.
#[async_trait]
pub trait WorkRunner: Send + Sync + 'static {
    /// Run `objective` to completion.
    ///
    /// # Errors
    ///
    /// [`RunFailure`] with an already-localized message, which becomes the
    /// Work's `error`.
    async fn run(&self, objective: String, context: WorkContext)
    -> Result<WorkOutcome, RunFailure>;
}

#[async_trait]
impl<F, Fut> WorkRunner for F
where
    F: Fn(String, WorkContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<WorkOutcome, RunFailure>> + Send,
{
    async fn run(
        &self,
        objective: String,
        context: WorkContext,
    ) -> Result<WorkOutcome, RunFailure> {
        self(objective, context).await
    }
}

/// What a canceler is asked.
///
/// `server/src/task/task-manager.mjs:727-733`:
/// `{ task, previousStatus, abort }`. `task` is the *snapshot*, with the full
/// delegation, because a `delegated` cancel has to name the target.
#[derive(Debug, Clone)]
pub struct CancelRequest {
    /// The Work, with its delegation.
    pub work: WorkSnapshot,
    /// What the Work was doing when the stop was asked for. The canceler
    /// branches on this: `delegated` goes to the coordinator, everything else
    /// aborts locally.
    pub previous_status: WorkStatus,
    signal: AbortSignal,
    default_reason: String,
}

impl CancelRequest {
    pub(crate) const fn new(
        work: WorkSnapshot,
        previous_status: WorkStatus,
        signal: AbortSignal,
        default_reason: String,
    ) -> Self {
        Self {
            work,
            previous_status,
            signal,
            default_reason,
        }
    }

    /// Abort the running request with the default reason.
    ///
    /// `abort: reason => task.abortController?.abort(reason || new Error('用户已取消这项工作'))`
    /// — the catalogued *cancellation strings* entry.
    pub fn abort(&self) {
        self.signal.abort(&self.default_reason);
    }

    /// Abort with a reason of the canceler's own.
    pub fn abort_with(&self, reason: &str) {
        self.signal.abort(reason);
    }

    /// The Work's cancellation scope, for a canceler that needs to observe it.
    #[must_use]
    pub const fn signal(&self) -> &AbortSignal {
        &self.signal
    }

    /// The default abort reason, already localized.
    #[must_use]
    pub fn default_reason(&self) -> &str {
        &self.default_reason
    }
}

/// Something that can stop a Work.
///
/// The return value is `via_downstream::CancelOutcome`, so "the coordinator
/// confirmed it" and "the transport was told" are different answers and the
/// manager can tell them apart. A canceler that returns
/// [`CancelOutcome::Requested`] leaves the Work `cancelling` until the abort it
/// raised actually settles the runner.
#[async_trait]
pub trait WorkCanceler: Send + Sync + 'static {
    /// Stop the Work `request` names.
    ///
    /// # Errors
    ///
    /// [`RunFailure`]. The Work then **fails** with
    /// `取消失败：<message>` rather than being reported as cancelled —
    /// `server/src/task/task-manager.mjs:752-753`.
    async fn cancel(&self, request: CancelRequest) -> Result<CancelOutcome, RunFailure>;
}

#[async_trait]
impl<F, Fut> WorkCanceler for F
where
    F: Fn(CancelRequest) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<CancelOutcome, RunFailure>> + Send,
{
    async fn cancel(&self, request: CancelRequest) -> Result<CancelOutcome, RunFailure> {
        self(request).await
    }
}

/// Reattaching to a delegated project session that outlived a Gateway restart.
///
/// `taskManager.recoverDelegated({ canRecover, runner, canceler })` —
/// `server/src/task/task-manager.mjs:249-277`, wired to the ACP adapter at
/// `gateway-application.mjs:73-83`.
#[async_trait]
pub trait DelegatedWorkRecovery: Send + Sync + 'static {
    /// Whether this Work's delegation can still be reattached to.
    ///
    /// A `false` here is what makes the Work fail with the *unrecoverable
    /// delegated work* restart message rather than the interactive one.
    fn can_recover(&self, work: &WorkSnapshot) -> bool;

    /// Reattach and wait for the delegated session's result.
    ///
    /// # Errors
    ///
    /// [`RunFailure`] if the reattach or the presentation turn fails.
    async fn run(
        &self,
        work: WorkSnapshot,
        context: WorkContext,
    ) -> Result<WorkOutcome, RunFailure>;

    /// Stop a reattached delegation.
    ///
    /// # Errors
    ///
    /// [`RunFailure`], which fails the Work with `取消失败：…`.
    async fn cancel(
        &self,
        work: WorkSnapshot,
        request: CancelRequest,
    ) -> Result<CancelOutcome, RunFailure>;
}

/// Asking the coordinator what a delegated Work is doing.
///
/// `taskManager.configureCoordinatorQuery(...)` —
/// `server/src/task/task-manager.mjs:143-145`, wired to
/// `coordinator.queryDelegatedWork` at `gateway-application.mjs:84-86`. Used by
/// the progress check, and only by it: a Work that is `delegated` when its
/// announcement falls due asks the coordinator rather than reading a ring that
/// stopped moving when the lane was released.
#[async_trait]
pub trait CoordinatorQuery: Send + Sync + 'static {
    /// Ask about `work_id`.
    ///
    /// # Errors
    ///
    /// [`RunFailure`]. The announcement then falls back to the composed
    /// activity message with `delegated: false`, which is upstream's `.catch`.
    async fn query(
        &self,
        work_id: &str,
        question: &str,
        owner_id: &str,
    ) -> Result<WorkOutcome, RunFailure>;
}

#[cfg(test)]
mod tests {
    use super::{AbortSignal, RunFailure, WorkOutcome};
    use serde_json::json;

    #[test]
    fn the_first_abort_reason_wins() {
        let signal = AbortSignal::new();
        assert!(!signal.is_aborted());
        assert_eq!(signal.reason(), None);
        signal.abort("first");
        signal.abort("second");
        assert!(signal.is_aborted());
        assert_eq!(signal.reason().as_deref(), Some("first"));
    }

    #[tokio::test]
    async fn aborted_resolves_and_stays_resolved() {
        let signal = AbortSignal::new();
        let waiter = signal.clone();
        let handle = tokio::spawn(async move { waiter.aborted().await });
        signal.abort("stop");
        handle.await.expect("the waiter resolves");
        signal.aborted().await;
    }

    #[test]
    fn a_clone_is_the_same_scope_and_a_new_signal_is_not() {
        let signal = AbortSignal::new();
        let clone = signal.clone();
        let sibling = AbortSignal::new();
        signal.abort("stop");
        assert!(clone.is_aborted(), "a clone belongs to the same Work");
        assert!(
            !sibling.is_aborted(),
            "an independent Work is not cancelled by somebody else's barge-in",
        );
    }

    #[test]
    fn an_outcome_carries_content_and_optional_metadata() {
        let outcome = WorkOutcome::content("done");
        assert_eq!(outcome.content, "done");
        assert!(outcome.metadata.is_none());
        let with = outcome.with_metadata(json!({"presentation": {"speech": "s"}}));
        assert_eq!(
            with.metadata,
            Some(json!({"presentation": {"speech": "s"}}))
        );
    }

    #[test]
    fn a_failure_displays_its_message() {
        let failure = RunFailure::new("后台 Agent 未返回内容");
        assert_eq!(failure.to_string(), "后台 Agent 未返回内容");
        assert_eq!(failure.message, "后台 Agent 未返回内容");
    }
}
