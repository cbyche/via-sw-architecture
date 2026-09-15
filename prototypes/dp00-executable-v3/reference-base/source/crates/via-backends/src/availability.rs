//! Cached backend availability for receipt-based tool acceptance.
//!
//! Ported from `server/src/agent/backend-availability.mjs`, with the probe
//! itself from `server/src/app/gateway-application.mjs:450-467`. Upstream's
//! opening comment is the whole design:
//!
//! > The voice frontend must hand out `spawn_thinking` receipts in
//! > milliseconds, so it can never block on a live health probe. This cache
//! > answers synchronously from the last known state and refreshes itself in the
//! > background; a backend that looks healthy here but fails at dispatch
//! > surfaces through the failed-task announcement path instead of the tool
//! > receipt.
//!
//! # Three states, not two
//!
//! [`AvailabilitySnapshot::known`] is what separates *"the backend is down"*
//! from *"nobody has looked yet"* and from *"it is still coming up"*. Both of
//! the unknown cases answer optimistically, because rejecting the first thing a
//! user says after launch is worse than accepting work that later reports a
//! failure. `docs/reference/contracts.json` records the same thing about
//! [`Availability::transient`]: *"the `transient` flag is what keeps
//! receipt-based `spawn_thinking` from being rejected during a cold start."*
//!
//! # Timing
//!
//! `snapshot()` is synchronous and may only *start* work, never wait for it.
//! It spawns the background refresh on the ambient tokio runtime; with no
//! runtime running there is nothing to spawn onto and the snapshot simply stays
//! stale, which is the same answer upstream gives when its event loop is busy.

use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use async_trait::async_trait;
use futures::FutureExt as _;
use futures::future::{BoxFuture, Shared};
use via_downstream::{DownstreamAgent, HarnessStatus, HarnessStatusCode};

/// How long a completed probe is trusted.
///
/// **External contract** — `backend-availability.mjs:9` (`ttlMs = 15_000`).
pub const DEFAULT_TTL: Duration = Duration::from_millis(15_000);

/// How soon a transient result is re-probed.
///
/// **External contract** — `backend-availability.mjs:10` (`retryMs = 500`).
pub const DEFAULT_RETRY: Duration = Duration::from_millis(500);

/// One probe's answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Availability {
    /// Whether a backend is configured at all.
    pub configured: bool,
    /// Whether it can take work right now.
    pub ok: bool,
    /// Whether `!ok` is a cold start rather than a fault.
    pub transient: bool,
}

impl Availability {
    /// `{ configured: false, ok: false }` — no backend configured.
    ///
    /// `gateway-application.mjs:452`. `transient` is absent upstream, which
    /// `result?.transient === true` reads as `false`.
    #[must_use]
    pub const fn not_configured() -> Self {
        Self {
            configured: false,
            ok: false,
            transient: false,
        }
    }
}

/// The synchronous view a receipt is issued from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AvailabilitySnapshot {
    /// Whether a backend is configured.
    pub configured: bool,
    /// Whether it can take work.
    pub ok: bool,
    /// Whether this answer rests on a completed, settled probe.
    ///
    /// `false` means *accept optimistically and let dispatch report failures* —
    /// either because no probe has finished yet
    /// (`backend-availability.mjs:36-39`) or because the last one said the
    /// backend was still starting.
    pub known: bool,
}

/// A probe failure.
///
/// Upstream has no type for this: `refresh()` wraps the call in `.catch()` and
/// treats a thrown probe as evidence the backend is unreachable
/// (`backend-availability.mjs:69-77`). A probe that cannot fail simply never
/// constructs one.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct ProbeFailed(pub String);

/// Something that can answer "is the backend available?".
#[async_trait]
pub trait AvailabilityProbe: Send + Sync + fmt::Debug {
    /// Run one probe.
    ///
    /// # Errors
    ///
    /// [`ProbeFailed`] — which the cache reads as `ok: false`, never as an
    /// error to propagate.
    async fn probe(&self) -> Result<Availability, ProbeFailed>;
}

/// The probe the Gateway installs: ask the configured harness how it is.
///
/// **External contract** — `server/src/app/gateway-application.mjs:450-467`,
/// reproduced condition for condition:
///
/// ```text
/// if (!agent.enabled) return { configured: false, ok: false }
/// const health = await agent.health()
/// return {
///   configured: true,
///   ok: health.ok === true,
///   transient: health.status === 'starting'
///     || ['NOT_STARTED', 'STARTING', 'BACKEND_STARTING'].includes(health.code),
/// }
/// ```
///
/// `agent.enabled` is `None` here: a Gateway in frontend-only mode has no
/// harness to hold, and `docs/architecture.md` §2 makes that a supported mode
/// rather than a fault.
#[derive(Clone)]
pub struct HarnessProbe {
    agent: Option<Arc<dyn DownstreamAgent>>,
}

impl HarnessProbe {
    /// A probe over the configured harness, or over nothing.
    #[must_use]
    pub fn new(agent: Option<Arc<dyn DownstreamAgent>>) -> Self {
        Self { agent }
    }
}

impl fmt::Debug for HarnessProbe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HarnessProbe")
            .field(
                "backend",
                &self.agent.as_ref().map(|agent| agent.descriptor().id()),
            )
            .finish()
    }
}

#[async_trait]
impl AvailabilityProbe for HarnessProbe {
    async fn probe(&self) -> Result<Availability, ProbeFailed> {
        let Some(agent) = &self.agent else {
            return Ok(Availability::not_configured());
        };
        let health = agent.health().await;
        Ok(Availability {
            configured: true,
            ok: health.is_ok(),
            transient: health.status() == HarnessStatus::Starting
                || HarnessStatusCode::TRANSIENT.contains(&health.code()),
        })
    }
}

/// A monotonic millisecond clock.
///
/// Upstream injects `now = () => Date.now()` so its TTL test can move time by
/// assignment; this is the same seam.
pub type Clock = Arc<dyn Fn() -> u64 + Send + Sync>;

/// The system clock, in milliseconds since the Unix epoch.
#[must_use]
pub fn system_clock() -> Clock {
    Arc::new(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| {
                u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
            })
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cached {
    configured: bool,
    ok: bool,
}

struct State {
    last: Option<Cached>,
    transient: bool,
    checked_at: u64,
    refreshing: Option<Shared<BoxFuture<'static, ()>>>,
    retry: Option<tokio::task::JoinHandle<()>>,
    closed: bool,
}

struct Inner {
    probe: Arc<dyn AvailabilityProbe>,
    ttl_ms: u64,
    retry_ms: u64,
    now: Clock,
    state: Mutex<State>,
}

impl Inner {
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The cache itself.
///
/// Cheap to clone; every clone shares one probe, one in-flight refresh and one
/// retry timer.
#[derive(Clone)]
pub struct BackendAvailability {
    inner: Arc<Inner>,
}

impl fmt::Debug for BackendAvailability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.inner.lock();
        f.debug_struct("BackendAvailability")
            .field("probe", &self.inner.probe)
            .field("ttl_ms", &self.inner.ttl_ms)
            .field("retry_ms", &self.inner.retry_ms)
            .field("last", &state.last)
            .field("transient", &state.transient)
            .field("closed", &state.closed)
            .finish()
    }
}

impl BackendAvailability {
    /// A cache with the catalogued defaults and the system clock.
    ///
    /// **External contract** — `backend-availability.mjs:7-25`. Constructing
    /// does **not** probe, exactly as upstream's constructor does not; the
    /// Gateway's call site follows it with one eager `refresh()`, which is what
    /// [`Self::start`] does in one step.
    #[must_use]
    pub fn new(probe: Arc<dyn AvailabilityProbe>) -> Self {
        Self::with_options(probe, DEFAULT_TTL, DEFAULT_RETRY, system_clock())
    }

    /// A cache with every knob supplied.
    #[must_use]
    pub fn with_options(
        probe: Arc<dyn AvailabilityProbe>,
        ttl: Duration,
        retry: Duration,
        now: Clock,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                probe,
                ttl_ms: u64::try_from(ttl.as_millis()).unwrap_or(u64::MAX),
                retry_ms: u64::try_from(retry.as_millis()).unwrap_or(u64::MAX),
                now,
                state: Mutex::new(State {
                    last: None,
                    transient: false,
                    checked_at: 0,
                    refreshing: None,
                    retry: None,
                    closed: false,
                }),
            }),
        }
    }

    /// [`Self::new`] plus the one eager refresh the Gateway performs.
    ///
    /// **External contract** — `gateway-application.mjs:450,468`
    /// (`new BackendAvailability({ probe }); backendAvailability.refresh()`).
    /// The refresh runs in the background: the Gateway does not await it, and
    /// neither does this.
    #[must_use]
    pub fn start(probe: Arc<dyn AvailabilityProbe>) -> Self {
        let availability = Self::new(probe);
        availability.spawn_refresh();
        availability
    }

    /// The synchronous view.
    ///
    /// **External contract** — `backend-availability.mjs:35-40`. A snapshot past
    /// the TTL starts a background refresh and still answers from the old value:
    /// the caller is issuing a receipt and cannot wait.
    #[must_use]
    pub fn snapshot(&self) -> AvailabilitySnapshot {
        let (expired, cached, transient) = {
            let state = self.inner.lock();
            let expired = (self.inner.now)().saturating_sub(state.checked_at) >= self.inner.ttl_ms;
            (expired, state.last, state.transient)
        };
        if expired {
            self.spawn_refresh();
        }
        match cached {
            None => AvailabilitySnapshot {
                configured: true,
                ok: true,
                known: false,
            },
            Some(cached) => AvailabilitySnapshot {
                configured: cached.configured,
                ok: cached.ok,
                known: !transient,
            },
        }
    }

    /// Run — or join — one background refresh.
    ///
    /// **External contract** — `backend-availability.mjs:52-79`. Concurrent
    /// callers share one probe, and the returned future never fails: a probe
    /// that failed *is* the answer.
    pub fn refresh(&self) -> impl std::future::Future<Output = ()> + Send + use<> {
        let mut state = self.inner.lock();
        if let Some(existing) = &state.refreshing {
            return existing.clone();
        }
        let inner = Arc::clone(&self.inner);
        let shared = async move {
            let result = inner.probe.probe().await;
            {
                let mut state = inner.lock();
                match result {
                    Ok(result) => {
                        state.last = Some(Cached {
                            configured: result.configured,
                            ok: result.ok,
                        });
                        state.transient = result.transient;
                    }
                    Err(_) => {
                        // A failing probe is itself evidence the backend is
                        // unreachable, and it clears `transient`: an error is
                        // not a cold start.
                        state.last = Some(Cached {
                            configured: state.last.is_none_or(|last| last.configured),
                            ok: false,
                        });
                        state.transient = false;
                    }
                }
                state.checked_at = (inner.now)();
                state.refreshing = None;
            }
            schedule_retry(&inner);
        }
        .boxed()
        .shared();
        state.refreshing = Some(shared.clone());
        drop(state);
        shared
    }

    /// Stop the retry timer.
    ///
    /// **External contract** — `backend-availability.mjs:81-85`. An in-flight
    /// probe is left to finish; only the *next* one is cancelled, which is what
    /// `clearTimeout` does.
    pub fn close(&self) {
        let mut state = self.inner.lock();
        state.closed = true;
        if let Some(handle) = state.retry.take() {
            handle.abort();
        }
    }

    /// Start a refresh in the background and drop the handle.
    fn spawn_refresh(&self) {
        let future = self.refresh();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(future);
        } else {
            tracing::debug!(
                event = "backend.availability_refresh_skipped",
                reason = "no tokio runtime",
            );
        }
    }
}

/// `scheduleRetry()` — `backend-availability.mjs:42-50`.
///
/// Only a transient result schedules one, only one is ever outstanding, and a
/// closed cache schedules none.
fn schedule_retry(inner: &Arc<Inner>) {
    let mut state = inner.lock();
    if state.closed || state.retry.is_some() || !state.transient {
        return;
    }
    let Ok(runtime) = tokio::runtime::Handle::try_current() else {
        return;
    };
    let retry_ms = inner.retry_ms;
    let inner_for_task = Arc::clone(inner);
    state.retry = Some(runtime.spawn(async move {
        tokio::time::sleep(Duration::from_millis(retry_ms)).await;
        let refresh = {
            let mut state = inner_for_task.lock();
            state.retry = None;
            if state.closed {
                return;
            }
            drop(state);
            BackendAvailability {
                inner: Arc::clone(&inner_for_task),
            }
            .refresh()
        };
        refresh.await;
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// A probe scripted by a closure over a call counter, so a test can say
    /// "the second probe answers differently" the way upstream's do.
    struct ScriptedProbe {
        calls: Arc<AtomicU64>,
        answer: Box<dyn Fn(u64) -> Result<Availability, ProbeFailed> + Send + Sync>,
    }

    impl fmt::Debug for ScriptedProbe {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("ScriptedProbe").finish_non_exhaustive()
        }
    }

    #[async_trait]
    impl AvailabilityProbe for ScriptedProbe {
        async fn probe(&self) -> Result<Availability, ProbeFailed> {
            let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            (self.answer)(call)
        }
    }

    fn scripted(
        answer: impl Fn(u64) -> Result<Availability, ProbeFailed> + Send + Sync + 'static,
    ) -> (Arc<dyn AvailabilityProbe>, Arc<AtomicU64>) {
        let calls = Arc::new(AtomicU64::new(0));
        let probe = ScriptedProbe {
            calls: Arc::clone(&calls),
            answer: Box::new(answer),
        };
        (Arc::new(probe), calls)
    }

    fn available(ok: bool) -> Result<Availability, ProbeFailed> {
        Ok(Availability {
            configured: true,
            ok,
            transient: false,
        })
    }

    #[tokio::test]
    async fn answers_optimistically_before_the_first_probe_completes() {
        let (probe, calls) = scripted(|_| available(false));
        let availability = BackendAvailability::new(probe);

        assert_eq!(
            availability.snapshot(),
            AvailabilitySnapshot {
                configured: true,
                ok: true,
                known: false,
            }
        );
        // The snapshot kicked exactly one background probe.
        availability.refresh().await;
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn reports_the_cached_result_synchronously_until_the_ttl_expires() {
        let clock = Arc::new(AtomicU64::new(0));
        let reader = Arc::clone(&clock);
        let (probe, calls) = scripted(|call| available(call == 1));
        let availability = BackendAvailability::with_options(
            probe,
            Duration::from_millis(1000),
            DEFAULT_RETRY,
            Arc::new(move || reader.load(Ordering::SeqCst)),
        );

        availability.refresh().await;
        assert_eq!(
            availability.snapshot(),
            AvailabilitySnapshot {
                configured: true,
                ok: true,
                known: true,
            }
        );

        // Within the TTL the snapshot never re-probes.
        clock.store(999, Ordering::SeqCst);
        let _ = availability.snapshot();
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        clock.store(2000, Ordering::SeqCst);
        let _ = availability.snapshot();
        availability.refresh().await;
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert!(!availability.snapshot().ok);
    }

    #[tokio::test]
    async fn treats_a_failing_probe_as_unreachable_without_propagating() {
        let (probe, _) = scripted(|_| Err(ProbeFailed("probe exploded".to_owned())));
        let availability = BackendAvailability::new(probe);

        availability.refresh().await;
        assert_eq!(
            availability.snapshot(),
            AvailabilitySnapshot {
                configured: true,
                ok: false,
                known: true,
            }
        );
    }

    #[tokio::test]
    async fn a_failing_probe_keeps_a_remembered_not_configured() {
        // `configured: this.last?.configured !== false` — a probe that fails
        // after the backend was known to be unconfigured must not silently
        // promote it to configured.
        let (probe, _) = scripted(|call| {
            if call == 1 {
                Ok(Availability::not_configured())
            } else {
                Err(ProbeFailed("gone".to_owned()))
            }
        });
        let availability = BackendAvailability::new(probe);
        availability.refresh().await;
        availability.refresh().await;
        assert_eq!(
            availability.snapshot(),
            AvailabilitySnapshot {
                configured: false,
                ok: false,
                known: true,
            }
        );
    }

    #[tokio::test]
    async fn shares_one_in_flight_probe_between_concurrent_refreshes() {
        let (probe, calls) = scripted(|_| available(true));
        let availability = BackendAvailability::new(probe);

        futures::future::join3(
            availability.refresh(),
            availability.refresh(),
            availability.refresh(),
        )
        .await;
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn remembers_that_the_backend_is_not_configured() {
        let (probe, _) = scripted(|_| Ok(Availability::not_configured()));
        let availability = BackendAvailability::new(probe);
        availability.refresh().await;
        assert_eq!(
            availability.snapshot(),
            AvailabilitySnapshot {
                configured: false,
                ok: false,
                known: true,
            }
        );
    }

    #[tokio::test]
    async fn treats_startup_as_unknown_and_retries_until_ready() {
        let (probe, calls) = scripted(|call| {
            if call == 1 {
                Ok(Availability {
                    configured: true,
                    ok: false,
                    transient: true,
                })
            } else {
                available(true)
            }
        });
        let availability = BackendAvailability::with_options(
            probe,
            DEFAULT_TTL,
            Duration::from_millis(1),
            system_clock(),
        );

        availability.refresh().await;
        assert_eq!(
            availability.snapshot(),
            AvailabilitySnapshot {
                configured: true,
                ok: false,
                known: false,
            },
            "a starting backend is not a broken one"
        );

        while calls.load(Ordering::SeqCst) < 2 {
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        availability.refresh().await;
        assert_eq!(
            availability.snapshot(),
            AvailabilitySnapshot {
                configured: true,
                ok: true,
                known: true,
            }
        );
        availability.close();
    }

    #[tokio::test]
    async fn a_closed_cache_stops_retrying() {
        let (probe, calls) = scripted(|_| {
            Ok(Availability {
                configured: true,
                ok: false,
                transient: true,
            })
        });
        let availability = BackendAvailability::with_options(
            probe,
            DEFAULT_TTL,
            Duration::from_millis(1),
            system_clock(),
        );
        availability.refresh().await;
        availability.close();
        let after_close = calls.load(Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(calls.load(Ordering::SeqCst), after_close);
    }

    #[tokio::test]
    async fn an_unconfigured_gateway_probes_as_not_configured() {
        let probe = HarnessProbe::new(None);
        assert_eq!(
            probe.probe().await.expect("never fails"),
            Availability::not_configured()
        );
    }
}
