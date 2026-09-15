//! `AbortSignal`, with its reason.
//!
//! Ported from `server/src/agent/backend-adapter.mjs:11-15` (`requestSignal`)
//! and the prompt's pausable timer in
//! `server/src/agent/acp-process-client.mjs:451-511`.
//!
//! # Why the reason travels with the signal
//!
//! Upstream does not merely abort a prompt; it rethrows `signal.reason`
//! (`acp-process-client.mjs:321,502`). A prompt that was cancelled by the user
//! and a prompt that ran out of time both stop, and the difference between them
//! decides what the user hears. A bare `CancellationToken` would collapse the
//! two.
//!
//! # The timer is pausable, and re-arms from full
//!
//! While a `session/request_permission` is outstanding the prompt's clock
//! stops: with nobody at a keyboard an approval can take minutes, and a turn
//! that timed out waiting for its own permission prompt would be a bug the user
//! experiences as randomness. When the permission resolves the timer is
//! **re-armed from the full duration**, not resumed from where it paused —
//! upstream's `armTimeout()` calls `setTimeout(…, timeoutMs)` again, and the
//! reason is that the deadline is meant to bound how long the *agent* works
//! uninterrupted, not the wall-clock life of the turn.

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use tokio::sync::Notify;

/// Why a request stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelReason {
    /// The deadline elapsed.
    ///
    /// Upstream's `DOMException('The operation was aborted due to timeout',
    /// 'TimeoutError')`.
    Timeout,
    /// The caller asked, with its own explanation.
    Caller(String),
}

impl CancelReason {
    /// The caller's explanation, if it gave one.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        match self {
            Self::Timeout => None,
            Self::Caller(detail) => Some(detail),
        }
    }

    /// Whether this is a deadline rather than a decision.
    #[must_use]
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout)
    }
}

#[derive(Debug, Default)]
struct SignalState {
    reason: Option<CancelReason>,
}

/// A cancellation signal, cheap to clone and shared by every holder.
///
/// The equivalent of one `AbortSignal`: it fires at most once, keeps the reason
/// it fired with, and every clone observes the same firing.
#[derive(Debug, Clone, Default)]
pub struct CancelSignal {
    state: Arc<Mutex<SignalState>>,
    notify: Arc<Notify>,
}

impl CancelSignal {
    /// A signal that has not fired.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A signal that has already fired.
    ///
    /// Upstream's "the combined signal is already aborted at prompt start" case
    /// (`acp-process-client.mjs:490`), which still sends `session/cancel`.
    #[must_use]
    pub fn aborted(reason: CancelReason) -> Self {
        let signal = Self::new();
        signal.abort(reason);
        signal
    }

    /// Fire the signal. The **first** reason wins; later ones are ignored.
    ///
    /// Returns whether this call is the one that fired it, which is what lets a
    /// timeout and a user cancellation race without either overwriting the
    /// other's explanation.
    pub fn abort(&self, reason: CancelReason) -> bool {
        let fired = {
            let mut state = self.lock();
            if state.reason.is_some() {
                false
            } else {
                state.reason = Some(reason);
                true
            }
        };
        if fired {
            self.notify.notify_waiters();
        }
        fired
    }

    /// Whether the signal has fired.
    #[must_use]
    pub fn is_aborted(&self) -> bool {
        self.lock().reason.is_some()
    }

    /// The reason it fired, if it has.
    #[must_use]
    pub fn reason(&self) -> Option<CancelReason> {
        self.lock().reason.clone()
    }

    /// Resolve when the signal fires.
    ///
    /// Already-fired signals resolve immediately, and the `Notified` future is
    /// registered **before** the state is re-checked, so a signal that fires
    /// between the two cannot be missed.
    pub async fn cancelled(&self) {
        loop {
            let notified = self.notify.notified();
            if self.is_aborted() {
                return;
            }
            notified.await;
            if self.is_aborted() {
                return;
            }
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, SignalState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// A deadline that can be paused and re-armed.
///
/// **Contract** — `acp-process-client.mjs:451-478`. Pausing is reference
/// counted, because nothing stops an agent from having two permission prompts
/// outstanding at once; the clock restarts only when the last of them resolves.
#[derive(Debug)]
pub struct PausableTimeout {
    signal: CancelSignal,
    timeout: Option<Duration>,
    state: Arc<Mutex<TimerState>>,
    wake: Arc<Notify>,
    task: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Debug, Default)]
struct TimerState {
    depth: usize,
    generation: u64,
    stopped: bool,
}

impl PausableTimeout {
    /// Arm a deadline against `signal`.
    ///
    /// A `timeout` of `None` — upstream's `timeoutMs: 0` — arms nothing: the
    /// caller owns the deadline. That is what delegated work uses, because the
    /// client's own pausable timer is running it instead.
    #[must_use]
    pub fn arm(signal: CancelSignal, timeout: Option<Duration>) -> Self {
        let mut timer = Self {
            signal,
            timeout,
            state: Arc::new(Mutex::new(TimerState::default())),
            wake: Arc::new(Notify::new()),
            task: None,
        };
        if timeout.is_some() {
            timer.task = Some(timer.spawn_loop());
        }
        timer
    }

    fn spawn_loop(&self) -> tokio::task::JoinHandle<()> {
        let signal = self.signal.clone();
        let state = Arc::clone(&self.state);
        let wake = Arc::clone(&self.wake);
        let Some(timeout) = self.timeout else {
            // Unreachable: `arm` only spawns when a timeout is set.
            return tokio::spawn(async {});
        };
        tokio::spawn(async move {
            loop {
                let (paused, generation, stopped) = {
                    let state = state
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    (state.depth > 0, state.generation, state.stopped)
                };
                if stopped || signal.is_aborted() {
                    return;
                }
                if paused {
                    wake.notified().await;
                    continue;
                }
                let woken = tokio::select! {
                    () = tokio::time::sleep(timeout) => false,
                    () = wake.notified() => true,
                };
                if woken {
                    // Pause, resume or stop: re-read the state and, if the
                    // clock is running again, start the full duration over.
                    continue;
                }
                let unchanged = {
                    let state = state
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    !state.stopped && state.depth == 0 && state.generation == generation
                };
                if unchanged {
                    signal.abort(CancelReason::Timeout);
                    return;
                }
            }
        })
    }

    /// Stop the clock — a permission prompt is outstanding.
    pub fn pause(&self) {
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.depth += 1;
            state.generation = state.generation.wrapping_add(1);
        }
        self.wake.notify_waiters();
    }

    /// Start the clock again, from the **full** duration.
    ///
    /// A no-op while another permission prompt is still outstanding.
    pub fn resume(&self) {
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.depth = state.depth.saturating_sub(1);
            state.generation = state.generation.wrapping_add(1);
        }
        self.wake.notify_waiters();
    }

    /// Whether the clock is currently stopped.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .depth
            > 0
    }
}

impl Drop for PausableTimeout {
    /// Upstream's `clearTimeout(timeoutTimer)` in the prompt's `finally`.
    fn drop(&mut self) {
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.stopped = true;
            state.generation = state.generation.wrapping_add(1);
        }
        self.wake.notify_waiters();
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_reason_wins() {
        let signal = CancelSignal::new();
        assert!(!signal.is_aborted());
        assert!(signal.abort(CancelReason::Caller("user".into())));
        assert!(
            !signal.abort(CancelReason::Timeout),
            "a later abort must not overwrite the reason a caller will rethrow"
        );
        assert_eq!(signal.reason(), Some(CancelReason::Caller("user".into())));
        assert_eq!(
            signal.reason().and_then(|r| r.detail().map(str::to_owned)),
            Some("user".into())
        );
    }

    #[tokio::test]
    async fn an_already_aborted_signal_resolves_immediately() {
        let signal = CancelSignal::aborted(CancelReason::Timeout);
        tokio::time::timeout(Duration::from_millis(50), signal.cancelled())
            .await
            .expect("an already-fired signal must not wait");
        assert!(signal.reason().is_some_and(|reason| reason.is_timeout()));
    }

    #[tokio::test]
    async fn a_signal_that_fires_while_waiting_wakes_every_waiter() {
        let signal = CancelSignal::new();
        let first = {
            let signal = signal.clone();
            tokio::spawn(async move { signal.cancelled().await })
        };
        let second = {
            let signal = signal.clone();
            tokio::spawn(async move { signal.cancelled().await })
        };
        tokio::time::sleep(Duration::from_millis(10)).await;
        signal.abort(CancelReason::Timeout);
        tokio::time::timeout(Duration::from_millis(200), async {
            first.await.expect("joined");
            second.await.expect("joined");
        })
        .await
        .expect("both waiters woke");
    }

    #[tokio::test]
    async fn a_timeout_fires_on_its_own() {
        let signal = CancelSignal::new();
        let _timer = PausableTimeout::arm(signal.clone(), Some(Duration::from_millis(20)));
        tokio::time::timeout(Duration::from_millis(500), signal.cancelled())
            .await
            .expect("the deadline fired");
        assert_eq!(signal.reason(), Some(CancelReason::Timeout));
    }

    #[tokio::test]
    async fn no_timeout_means_the_caller_owns_the_deadline() {
        let signal = CancelSignal::new();
        let _timer = PausableTimeout::arm(signal.clone(), None);
        tokio::time::sleep(Duration::from_millis(60)).await;
        assert!(!signal.is_aborted(), "timeoutMs: 0 arms nothing at all");
    }

    #[tokio::test]
    async fn a_paused_timer_never_fires() {
        // Upstream: 'pauses the prompt timeout while waiting for user
        // permission' (acp-process-client.test.mjs:222-267).
        let signal = CancelSignal::new();
        let timer = PausableTimeout::arm(signal.clone(), Some(Duration::from_millis(30)));
        tokio::time::sleep(Duration::from_millis(10)).await;
        timer.pause();
        assert!(timer.is_paused());
        tokio::time::sleep(Duration::from_millis(120)).await;
        assert!(
            !signal.is_aborted(),
            "a turn parked on a permission prompt must not time out"
        );

        timer.resume();
        assert!(!timer.is_paused());
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(
            !signal.is_aborted(),
            "the clock restarts from the full duration, not from where it paused"
        );
        tokio::time::timeout(Duration::from_millis(500), signal.cancelled())
            .await
            .expect("and then it does fire");
    }

    #[tokio::test]
    async fn nested_pauses_are_reference_counted() {
        let signal = CancelSignal::new();
        let timer = PausableTimeout::arm(signal.clone(), Some(Duration::from_millis(25)));
        timer.pause();
        timer.pause();
        timer.resume();
        assert!(
            timer.is_paused(),
            "two outstanding permission prompts, one resolved: still paused"
        );
        tokio::time::sleep(Duration::from_millis(80)).await;
        assert!(!signal.is_aborted());
        timer.resume();
        assert!(!timer.is_paused());
        tokio::time::timeout(Duration::from_millis(500), signal.cancelled())
            .await
            .expect("the last resume restarts the clock");
    }

    #[tokio::test]
    async fn resuming_more_often_than_pausing_does_not_underflow() {
        let signal = CancelSignal::new();
        let timer = PausableTimeout::arm(signal.clone(), Some(Duration::from_millis(20)));
        timer.resume();
        timer.resume();
        assert!(!timer.is_paused());
        tokio::time::timeout(Duration::from_millis(500), signal.cancelled())
            .await
            .expect("the clock is still running");
    }

    #[tokio::test]
    async fn dropping_the_timer_disarms_it() {
        let signal = CancelSignal::new();
        {
            let _timer = PausableTimeout::arm(signal.clone(), Some(Duration::from_millis(20)));
        }
        tokio::time::sleep(Duration::from_millis(80)).await;
        assert!(
            !signal.is_aborted(),
            "a prompt that returned must not abort its own signal afterwards"
        );
    }
}
