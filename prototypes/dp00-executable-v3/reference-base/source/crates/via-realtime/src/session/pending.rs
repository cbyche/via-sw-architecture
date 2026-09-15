//! One response the session asked for and has not yet accounted for.
//!
//! Upstream's `pending` object (`realtime-provider.mjs:492-499`), which is
//! shared by reference between three places at once: the correlation queue, the
//! response-id map, and the closure that resolves the caller's promise. Rust
//! spells "shared by reference" as `Arc`, and the two owning tasks both need to
//! mutate it, so the mutable half sits behind a `std::sync::Mutex` that is never
//! held across an `await`.
//!
//! Identity is `Arc::ptr_eq`, which is upstream's `indexOf(pending)` and
//! `responseWaiters.get(id) === pending`. It is not the request id: the request
//! id is a *correlation* token that only the GA dialect echoes, and two pendings
//! can legitimately exist with none of it observable.

use std::sync::{Arc, Mutex, MutexGuard};

use serde_json::Value;
use tokio::sync::oneshot;
use tokio::task::AbortHandle;
use tokio::time::Instant;

use super::outcome::{ResponseContext, ResponseOrigin, ResponseOutcome};

/// The mutable half.
#[derive(Debug, Default)]
struct PendingInner {
    settled: bool,
    reply: Option<oneshot::Sender<ResponseOutcome>>,
    timer: Option<AbortHandle>,
    response_payload: Option<Value>,
    busy_retries: u32,
    started_at: Option<Instant>,
    last_activity_at: Option<Instant>,
}

/// A response start that is still in flight.
#[derive(Debug)]
pub(crate) struct PendingResponse {
    origin: ResponseOrigin,
    context: ResponseContext,
    request_id: String,
    inner: Mutex<PendingInner>,
}

impl PendingResponse {
    pub(crate) fn new(
        origin: ResponseOrigin,
        context: ResponseContext,
        request_id: String,
        reply: oneshot::Sender<ResponseOutcome>,
    ) -> Arc<Self> {
        Arc::new(Self {
            origin,
            context,
            request_id,
            inner: Mutex::new(PendingInner {
                reply: Some(reply),
                ..PendingInner::default()
            }),
        })
    }

    /// A poison-tolerant lock.
    ///
    /// A panic while this guard is held would poison the mutex and turn every
    /// later `lock()` into an error; recovering the guard keeps a single
    /// unrelated panic from wedging the whole session, and there is no invariant
    /// inside `PendingInner` that a partial write could corrupt — every field is
    /// independent.
    fn inner(&self) -> MutexGuard<'_, PendingInner> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    pub(crate) fn origin(&self) -> ResponseOrigin {
        self.origin
    }

    pub(crate) fn context(&self) -> &ResponseContext {
        &self.context
    }

    pub(crate) fn request_id(&self) -> &str {
        &self.request_id
    }

    pub(crate) fn is_settled(&self) -> bool {
        self.inner().settled
    }

    /// Resolve the caller's promise, once.
    ///
    /// Upstream `settlePending`: *"if (!pending || pending.settled) return"* —
    /// so every later attempt is a no-op, which is what lets the cancel path,
    /// the timeout path and the `response.done` path all race safely.
    pub(crate) fn settle(&self, outcome: ResponseOutcome) {
        let mut inner = self.inner();
        if inner.settled {
            return;
        }
        inner.settled = true;
        if let Some(timer) = inner.timer.take() {
            timer.abort();
        }
        if let Some(reply) = inner.reply.take() {
            // The receiver is gone only if the caller stopped awaiting, which is
            // not this side's problem.
            let _ = reply.send(outcome);
        }
    }

    /// Replace the timer, cancelling whatever was armed before.
    ///
    /// Upstream `clearTimeout(pending.timer); pending.timer = setTimeout(...)`.
    pub(crate) fn arm_timer(&self, timer: AbortHandle) {
        let mut inner = self.inner();
        if let Some(previous) = inner.timer.replace(timer) {
            previous.abort();
        }
    }

    /// Cancel the timer without arming another. Upstream `clearTimeout`.
    pub(crate) fn cancel_timer(&self) {
        if let Some(timer) = self.inner().timer.take() {
            timer.abort();
        }
    }

    /// Remember the exact frame that was written for this response.
    ///
    /// Upstream's comment is the reason it is stored rather than rebuilt:
    /// *"Remember the exact payload so transient response-slot and Smart Turn
    /// input collisions can replay it without rebuilding conversation state."*
    /// A rebuilt payload would also lose the GA correlation metadata.
    pub(crate) fn set_response_payload(&self, payload: Value) {
        self.inner().response_payload = Some(payload);
    }

    pub(crate) fn response_payload(&self) -> Option<Value> {
        self.inner().response_payload.clone()
    }

    pub(crate) fn busy_retries(&self) -> u32 {
        self.inner().busy_retries
    }

    /// Count one busy retry and return the new total.
    pub(crate) fn record_busy_retry(&self) -> u32 {
        let mut inner = self.inner();
        inner.busy_retries = inner.busy_retries.saturating_add(1);
        inner.busy_retries
    }

    /// Note that the provider has started this response.
    pub(crate) fn mark_started(&self, at: Instant) {
        self.inner().started_at = Some(at);
    }

    pub(crate) fn started_at(&self) -> Option<Instant> {
        self.inner().started_at
    }

    /// Note output activity, which re-opens the inactivity window.
    pub(crate) fn mark_activity(&self, at: Instant) {
        self.inner().last_activity_at = Some(at);
    }

    pub(crate) fn last_activity_at(&self) -> Option<Instant> {
        self.inner().last_activity_at
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::super::outcome::{OutcomeKind, OutcomePhase};
    use super::*;

    fn pending() -> (Arc<PendingResponse>, oneshot::Receiver<ResponseOutcome>) {
        let (tx, rx) = oneshot::channel();
        (
            PendingResponse::new(
                ResponseOrigin::Agent,
                ResponseContext::new().with("turnId", "t"),
                "request-1".to_owned(),
                tx,
            ),
            rx,
        )
    }

    #[tokio::test]
    async fn settling_resolves_the_caller_exactly_once() {
        let (pending, rx) = pending();
        assert!(!pending.is_settled());
        pending.settle(ResponseOutcome::new(OutcomeKind::Completed).with_response_id("r"));
        assert!(pending.is_settled());
        // A later settle is a no-op, not a second value.
        pending.settle(ResponseOutcome::new(OutcomeKind::Cancelled));
        let outcome = rx.await.expect("outcome");
        assert_eq!(outcome.kind, OutcomeKind::Completed);
        assert_eq!(outcome.response_id.as_deref(), Some("r"));
    }

    #[tokio::test]
    async fn settling_cancels_the_armed_timer() {
        let (pending, _rx) = pending();
        let handle = tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        });
        let abort = handle.abort_handle();
        pending.arm_timer(abort);
        pending.settle(ResponseOutcome::new(OutcomeKind::TimedOut).with_phase(OutcomePhase::Start));
        assert!(
            handle.await.is_err(),
            "the timer task must have been aborted"
        );
    }

    #[tokio::test]
    async fn arming_a_second_timer_cancels_the_first() {
        let (pending, _rx) = pending();
        let first = tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        });
        let second = tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        });
        pending.arm_timer(first.abort_handle());
        pending.arm_timer(second.abort_handle());
        assert!(first.await.is_err());
        pending.cancel_timer();
        assert!(second.await.is_err());
    }

    #[test]
    fn identity_is_the_allocation_not_the_request_id() {
        let (first, _a) = pending();
        let (second, _b) = pending();
        assert_eq!(first.request_id(), second.request_id());
        assert!(!Arc::ptr_eq(&first, &second));
        assert!(Arc::ptr_eq(&first, &Arc::clone(&first)));
    }

    #[test]
    fn busy_retries_count_up_and_saturate() {
        let (pending, _rx) = pending();
        assert_eq!(pending.busy_retries(), 0);
        assert_eq!(pending.record_busy_retry(), 1);
        assert_eq!(pending.record_busy_retry(), 2);
        assert_eq!(pending.busy_retries(), 2);
    }

    #[test]
    fn the_context_and_origin_are_carried_verbatim() {
        let (pending, _rx) = pending();
        assert_eq!(pending.origin(), ResponseOrigin::Agent);
        assert_eq!(pending.context().turn_id(), Some("t"));
    }
}
