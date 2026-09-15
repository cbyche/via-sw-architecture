//! The keyed serial executor — the second of the two guards.
//!
//! `server/src/agent/keyed-serial-executor.mjs` in full, eighteen lines:
//!
//! ```js
//! run(key, operation) {
//!   const previous = this.queues.get(key) || Promise.resolve()
//!   const current = previous.catch(() => {}).then(operation)
//!   this.queues.set(key, current)
//!   return current.finally(() => {
//!     if (this.queues.get(key) === current) this.queues.delete(key)
//!   })
//! }
//! ```
//!
//! Three properties are hiding in there, and all three are the contract:
//!
//! 1. **Order.** A promise chain runs its links in the order they were
//!    appended. This is not mutual exclusion — it is FIFO — and
//!    `docs/architecture.md` §11 is explicit that *"`tokio::sync::Mutex` gives
//!    mutual exclusion but not FIFO order, and the ordering IS the contract."*
//!    So this is an **owning task** over a bounded `mpsc`, and the waiter queue
//!    is a `VecDeque` in arrival order.
//! 2. **A failed operation does not wedge the lane.** `previous.catch(() => {})`
//!    swallows the previous link's rejection before chaining. Here the
//!    equivalent is that the lane is released when the [`LanePermit`] is
//!    **dropped**, which happens whether the operation returned, panicked, or
//!    was cancelled mid-await.
//! 3. **The lane is forgotten when it empties.** `queues.delete(key)` — so
//!    [`KeyedSerialExecutor::size`] measures live lanes rather than every key
//!    ever used, and a Gateway that has served ten thousand delegations is not
//!    carrying ten thousand map entries.
//!
//! # Why there are two guards
//!
//! `docs/architecture.md` §11, invariant 2: *"Both the Gateway queue **and** the
//! ACP adapter serialize session writes. The double guard is deliberate;
//! porting one of the two looks correct until it is under load."* The Gateway
//! queue is [`via_work`]'s per-owner coordinator lane
//! ([`via_work::coordinator_lane`], width 1); this is the adapter's. They
//! protect different things: the lane bounds how much Work is *admitted*, this
//! bounds concurrent *writes to one backend session* — including the writes
//! that never went through the Work queue at all, which is every hidden control
//! turn in [`crate::prompts`].
//!
//! # Two channels, and why one of them is unbounded
//!
//! Commands arrive on a bounded channel, as `docs/architecture.md` §11 asks.
//! Releases arrive on their own **unbounded** channel because a release is sent
//! from [`Drop`], which cannot await: a bounded `try_send` that hit a full
//! channel would drop the release and wedge the lane forever. The release
//! channel is bounded in practice by the number of outstanding permits, which
//! is bounded by the number of live lanes.

use std::collections::VecDeque;
use std::future::Future;

use indexmap::IndexMap;
use tokio::sync::{mpsc, oneshot};
use tokio_util::task::TaskTracker;

/// How many commands may be in flight before a caller waits.
///
/// VIA's own — upstream has no queue at all, only a promise chain. The number
/// is a back-pressure knob rather than a contract: it bounds how many tasks can
/// be *between* calling [`KeyedSerialExecutor::acquire`] and being enqueued,
/// and any positive value preserves FIFO order because arrival order is send
/// order.
pub const COMMAND_QUEUE_DEPTH: usize = 64;

/// The executor's owning task has shut down.
///
/// Developer-facing: it means the [`KeyedSerialExecutor`] was closed while
/// something still held a handle to it, which is a shutdown-ordering bug rather
/// than anything an operator can act on. The same class as
/// [`via_work::ManagerStopped`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the coordinator's serial executor has shut down")]
pub struct ExecutorStopped;

impl ExecutorStopped {
    /// A stable machine-readable code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        "VIA_COORDINATOR_EXECUTOR_STOPPED"
    }
}

/// One message to the owning task.
enum Command {
    /// Join `key`'s queue; `granted` fires when the lane is this caller's.
    Acquire {
        key: String,
        granted: oneshot::Sender<()>,
    },
    /// How many lanes are live.
    Size { reply: oneshot::Sender<usize> },
    /// How many callers are queued on `key`, the holder included.
    Depth {
        key: String,
        reply: oneshot::Sender<usize>,
    },
}

/// One key's queue.
#[derive(Debug, Default)]
struct Lane {
    held: bool,
    waiters: VecDeque<oneshot::Sender<()>>,
}

/// The right to run on one key, until dropped.
///
/// Dropping it releases the lane, whether the operation finished, failed, or
/// was cancelled. There is deliberately no `release()` method: an explicit
/// release is a second way to end the permit's life and therefore a second way
/// to forget to.
#[derive(Debug)]
pub struct LanePermit {
    key: String,
    releases: mpsc::UnboundedSender<String>,
}

impl LanePermit {
    /// The key this permit holds.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }
}

impl Drop for LanePermit {
    fn drop(&mut self) {
        // A closed executor means the lane no longer exists; there is nothing
        // to release and nothing to report to.
        let _ = self.releases.send(std::mem::take(&mut self.key));
    }
}

/// Serializes operations per key, in the order they asked.
///
/// Cheap to clone; every clone addresses the same owning task.
#[derive(Debug, Clone)]
pub struct KeyedSerialExecutor {
    commands: mpsc::Sender<Command>,
    releases: mpsc::UnboundedSender<String>,
}

impl Default for KeyedSerialExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyedSerialExecutor {
    /// Start an executor on the ambient tokio runtime.
    #[must_use]
    pub fn new() -> Self {
        Self::spawn(None)
    }

    /// Start an executor whose task is registered on `tracker`, so a Gateway
    /// shutdown joins it.
    #[must_use]
    pub fn with_tracker(tracker: &TaskTracker) -> Self {
        Self::spawn(Some(tracker))
    }

    fn spawn(tracker: Option<&TaskTracker>) -> Self {
        let (commands, command_rx) = mpsc::channel(COMMAND_QUEUE_DEPTH);
        let (releases, release_rx) = mpsc::unbounded_channel();
        let future = run_executor(command_rx, release_rx);
        match tracker {
            Some(tracker) => {
                tracker.spawn(future);
            }
            None => {
                tokio::spawn(future);
            }
        }
        Self { commands, releases }
    }

    /// Take `key`'s lane, waiting behind everybody who asked first.
    ///
    /// # Errors
    ///
    /// [`ExecutorStopped`] when the owning task is gone.
    pub async fn acquire(&self, key: &str) -> Result<LanePermit, ExecutorStopped> {
        let (granted, wait) = oneshot::channel();
        self.commands
            .send(Command::Acquire {
                key: key.to_owned(),
                granted,
            })
            .await
            .map_err(|_| ExecutorStopped)?;
        wait.await.map_err(|_| ExecutorStopped)?;
        Ok(LanePermit {
            key: key.to_owned(),
            releases: self.releases.clone(),
        })
    }

    /// Run `operation` with `key`'s lane held.
    ///
    /// The direct translation of upstream's `run(key, operation)`. The permit
    /// is dropped when the operation ends *or* when this future is dropped, so
    /// a cancelled caller never leaves the lane held.
    ///
    /// # Deadlock
    ///
    /// An operation that acquires the **same** key waits for itself, exactly as
    /// a promise chain that awaits its own link would. Nested acquisition of a
    /// *different* key is fine and is what the coordinator does: a coordinator
    /// turn holds `coordinator:<key>` and a delegated prompt holds
    /// `target:<session id>`.
    ///
    /// # Errors
    ///
    /// [`ExecutorStopped`] when the owning task is gone. `operation` is not
    /// started in that case.
    pub async fn run<F>(&self, key: &str, operation: F) -> Result<F::Output, ExecutorStopped>
    where
        F: Future,
    {
        let permit = self.acquire(key).await?;
        let output = operation.await;
        drop(permit);
        Ok(output)
    }

    /// How many lanes are live.
    ///
    /// Upstream's `get size()`. Returns `0` once the executor has stopped,
    /// which is true rather than merely convenient: a stopped executor holds
    /// nothing.
    pub async fn size(&self) -> usize {
        let (reply, answer) = oneshot::channel();
        if self.commands.send(Command::Size { reply }).await.is_err() {
            return 0;
        }
        answer.await.unwrap_or(0)
    }

    /// How many callers `key` has, the holder included.
    ///
    /// VIA's own, and the reason it exists is that FIFO order is otherwise
    /// unobservable: a test can prove exclusion by timing, but proving *order*
    /// needs to see the queue.
    pub async fn depth(&self, key: &str) -> usize {
        let (reply, answer) = oneshot::channel();
        if self
            .commands
            .send(Command::Depth {
                key: key.to_owned(),
                reply,
            })
            .await
            .is_err()
        {
            return 0;
        }
        answer.await.unwrap_or(0)
    }

    /// Whether the owning task is still running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        !self.commands.is_closed()
    }
}

/// The owning task: all the state, held by value, in one place.
async fn run_executor(
    mut commands: mpsc::Receiver<Command>,
    mut releases: mpsc::UnboundedReceiver<String>,
) {
    let mut lanes: IndexMap<String, Lane> = IndexMap::new();
    loop {
        tokio::select! {
            // Biased, releases first: a lane that has just been given back is
            // free before the next acquisition is considered, so a caller that
            // was already queued is granted rather than a lane being reported
            // busy to somebody who arrives in the same tick.
            biased;
            released = releases.recv() => match released {
                Some(key) => release(&mut lanes, &key),
                None => break,
            },
            command = commands.recv() => match command {
                Some(Command::Acquire { key, granted }) => acquire(&mut lanes, key, granted),
                Some(Command::Size { reply }) => {
                    let _ = reply.send(lanes.len());
                }
                Some(Command::Depth { key, reply }) => {
                    let depth = lanes.get(&key).map_or(0, |lane| {
                        usize::from(lane.held) + lane.waiters.len()
                    });
                    let _ = reply.send(depth);
                }
                None => break,
            },
        }
    }
}

/// Grant `key` now, or queue behind whoever holds it.
fn acquire(lanes: &mut IndexMap<String, Lane>, key: String, granted: oneshot::Sender<()>) {
    let lane = lanes.entry(key).or_default();
    if lane.held {
        lane.waiters.push_back(granted);
        return;
    }
    // A receiver that has already gone away is a caller whose future was
    // dropped between sending and being granted. Nothing took the lane, so it
    // stays free rather than being held by nobody.
    if granted.send(()).is_ok() {
        lane.held = true;
    }
}

/// Hand `key` to the next waiter, or forget the lane.
fn release(lanes: &mut IndexMap<String, Lane>, key: &str) {
    let Some(lane) = lanes.get_mut(key) else {
        return;
    };
    lane.held = false;
    while let Some(waiter) = lane.waiters.pop_front() {
        if waiter.send(()).is_ok() {
            lane.held = true;
            return;
        }
    }
    // `shift_remove`, not `swap_remove`: `size` is observable and lanes are
    // reported in creation order.
    lanes.shift_remove(key);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;

    use super::*;

    fn record(log: &Arc<Mutex<Vec<&'static str>>>, entry: &'static str) {
        log.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(entry);
    }

    fn entries(log: &Arc<Mutex<Vec<&'static str>>>) -> Vec<&'static str> {
        log.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Yield until `probe` answers true, or fail saying what never happened.
    ///
    /// Bounded on purpose: these tests run on a paused, single-threaded
    /// runtime, so an unbounded `yield_now` loop turns a broken invariant into
    /// a hang — and a hung test tells nobody anything.
    const MAX_SETTLE_POLLS: usize = 10_000;

    async fn settle_until<F, Fut>(what: &str, mut probe: F)
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        for _ in 0..MAX_SETTLE_POLLS {
            if probe().await {
                return;
            }
            tokio::task::yield_now().await;
        }
        panic!("gave up waiting for {what} after {MAX_SETTLE_POLLS} polls");
    }

    #[tokio::test(start_paused = true)]
    async fn operations_on_one_key_run_in_the_order_they_asked() {
        let executor = KeyedSerialExecutor::new();
        let log = Arc::new(Mutex::new(Vec::new()));

        // Each is enqueued before the next asks, so arrival order is
        // deterministic and the assertion is about order rather than timing.
        let first = executor.acquire("k").await.expect("running");
        assert_eq!(executor.depth("k").await, 1);

        let second = {
            let executor = executor.clone();
            let log = Arc::clone(&log);
            tokio::spawn(async move {
                let permit = executor.acquire("k").await.expect("running");
                record(&log, "second");
                drop(permit);
            })
        };
        settle_until("the second caller to queue", || async {
            executor.depth("k").await >= 2
        })
        .await;

        let third = {
            let executor = executor.clone();
            let log = Arc::clone(&log);
            tokio::spawn(async move {
                let permit = executor.acquire("k").await.expect("running");
                record(&log, "third");
                drop(permit);
            })
        };
        settle_until("the third caller to queue", || async {
            executor.depth("k").await >= 3
        })
        .await;

        record(&log, "first");
        drop(first);
        second.await.expect("joined");
        third.await.expect("joined");
        assert_eq!(entries(&log), ["first", "second", "third"]);
    }

    #[tokio::test(start_paused = true)]
    async fn a_different_key_is_a_different_lane() {
        let executor = KeyedSerialExecutor::new();
        let held = executor.acquire("one").await.expect("running");
        let other = executor.acquire("two").await.expect("running");
        assert_eq!(executor.size().await, 2);
        drop(held);
        drop(other);
        settle_until("the lane to be forgotten", || async {
            executor.size().await == 0
        })
        .await;
    }

    #[tokio::test(start_paused = true)]
    async fn an_empty_lane_is_forgotten() {
        let executor = KeyedSerialExecutor::new();
        assert_eq!(executor.size().await, 0);
        let permit = executor.acquire("k").await.expect("running");
        assert_eq!(executor.size().await, 1);
        drop(permit);
        settle_until("the lane to be forgotten", || async {
            executor.size().await == 0
        })
        .await;
        assert_eq!(executor.depth("k").await, 0);
    }

    #[tokio::test(start_paused = true)]
    async fn a_panicking_operation_does_not_wedge_its_lane() {
        let executor = KeyedSerialExecutor::new();
        let panicked = {
            let executor = executor.clone();
            tokio::spawn(async move {
                let _permit = executor.acquire("k").await.expect("running");
                panic!("the operation failed");
            })
        };
        assert!(panicked.await.is_err(), "the task panicked");
        // The permit was dropped by the unwind, so the lane is free again.
        let next = executor.acquire("k").await.expect("running");
        assert_eq!(next.key(), "k");
    }

    #[tokio::test(start_paused = true)]
    async fn a_cancelled_waiter_does_not_hold_the_lane_it_never_took() {
        let executor = KeyedSerialExecutor::new();
        let held = executor.acquire("k").await.expect("running");

        let cancelled = {
            let executor = executor.clone();
            tokio::spawn(async move {
                let _permit = executor.acquire("k").await;
            })
        };
        settle_until("the second caller to queue", || async {
            executor.depth("k").await >= 2
        })
        .await;
        cancelled.abort();
        let _ = cancelled.await;

        drop(held);
        // The abandoned waiter is skipped; the lane is grantable at once.
        let next = tokio::time::timeout(std::time::Duration::from_secs(1), executor.acquire("k"))
            .await
            .expect("the lane was not left held by a task that went away")
            .expect("running");
        drop(next);
    }

    #[tokio::test(start_paused = true)]
    async fn run_releases_on_the_way_out() {
        let executor = KeyedSerialExecutor::new();
        let answer = executor.run("k", async { 7_u8 }).await.expect("running");
        assert_eq!(answer, 7);
        settle_until("the lane to be forgotten", || async {
            executor.size().await == 0
        })
        .await;
    }

    #[tokio::test(start_paused = true)]
    async fn a_clone_addresses_the_same_owning_task() {
        let executor = KeyedSerialExecutor::new();
        let clone = executor.clone();
        let held = executor.acquire("k").await.expect("running");
        assert_eq!(
            clone.depth("k").await,
            1,
            "a clone sees the lane the original holds",
        );

        drop(executor);
        assert!(
            clone.is_running(),
            "the owning task outlives any one handle",
        );

        drop(held);
        settle_until("the lane to be forgotten", || async {
            clone.size().await == 0
        })
        .await;
        assert_eq!(clone.acquire("k").await.expect("running").key(), "k");
    }

    #[test]
    fn the_stopped_refusal_is_coded_and_developer_facing() {
        // It is a shutdown-ordering bug, not something an operator can act on,
        // so it carries a code and English rather than a `via-i18n` key.
        assert_eq!(ExecutorStopped.code(), "VIA_COORDINATOR_EXECUTOR_STOPPED");
        assert_eq!(
            ExecutorStopped.to_string(),
            "the coordinator's serial executor has shut down",
        );
    }
}
