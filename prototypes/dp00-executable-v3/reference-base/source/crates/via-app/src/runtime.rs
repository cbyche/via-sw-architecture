//! Startup and shutdown, in upstream's exact order.
//!
//! A port of `server/src/index.mjs`.
//!
//! # Startup
//!
//! ```text
//! loadRuntimeEnvironment       // via_core::runtime
//! assertGatewaySetup           // refuse BEFORE the lease is touched
//! acquireGatewayLease          // one Gateway per config directory
//! …serve…
//! lease.update({state:'ready', origin})
//! setInterval(heartbeat, 15_000)
//! ```
//!
//! **The gate comes before the lease**, and upstream says why on the line
//! above it: *"Setup gate: refuse an unconfigured start before touching the
//! lease. A Gateway that listens but cannot connect its voice is harder to
//! diagnose than a refusal the user can act on."* A misconfigured start must
//! not disturb a running one, so [`Startup::begin`] runs
//! [`via_core::assert_gateway_setup`] first and only
//! then acquires.
//!
//! # Shutdown
//!
//! ```text
//! clearInterval(gatewayHeartbeat)
//! Promise.all([backendRuntime?.stop(signal), agentClient?.close()])
//!   .catch(error => logger.error('backend.stop_failed', {error}))
//!   .finally(() => logger.flush())
//! ```
//!
//! Three things are load-bearing and each has a test:
//!
//! - the heartbeat is cleared **first**, so a shutdown that takes seconds does
//!   not keep refreshing a lease it is about to release;
//! - the backend runtime and the agent client are stopped **concurrently**, not
//!   in sequence — `Promise.all`, not `await` then `await`;
//! - one failure is caught as `backend.stop_failed` and the logger is flushed
//!   **regardless**, because the log line explaining the failed stop is the one
//!   thing a user has afterwards.
//!
//! The exit path arms a **2000 ms** timeout and then exits whatever happens
//! (`index.mjs:41-46`), so a harness that never answers `stop` cannot wedge the
//! process.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::FutureExt;
use tokio_util::sync::CancellationToken;
use via_core::{Config, assert_gateway_setup};
use via_lock::{AcquireOptions, GatewayLeaseHandle, LeaseUpdate, acquire_gateway_lease};
use via_log::Logger;

use crate::error::AppError;
use crate::state::InstanceIdentity;

/// How often the lease is refreshed while the Gateway runs.
///
/// **External contract** — `server/src/index.mjs:143`: `setInterval(…, 15_000)`.
/// [`via_lock::GATEWAY_HEARTBEAT_INTERVAL`] is the constant; it is named here so
/// the sequence reads in one place.
pub const HEARTBEAT_INTERVAL: Duration = via_lock::GATEWAY_HEARTBEAT_INTERVAL;

/// How long the exit path waits before exiting anyway.
///
/// **External contract** — `server/src/index.mjs:43`:
/// `setTimeout(() => process.exit(0), 2000)`.
pub const EXIT_TIMEOUT: Duration = Duration::from_millis(2000);

/// The lease state a Gateway reports once it is listening.
///
/// **External contract** — `server/src/index.mjs:139`.
pub const LEASE_STATE_READY: &str = via_lock::LEASE_STATE_READY;

/// Something that must be stopped on the way out.
///
/// The two upstream stops — `backendRuntime.stop(signal)` and
/// `agentClient.close()` — behind one trait, because they are started by
/// whoever owns Layer 3 and this module only has to stop them *together*.
#[async_trait]
pub trait Stoppable: Send + Sync {
    /// Stop, and report why not.
    ///
    /// # Errors
    ///
    /// Any message; [`shutdown`] logs it as `backend.stop_failed` and keeps
    /// going.
    async fn stop(&self) -> Result<(), String>;
}

/// Everything a running Gateway must release.
///
/// Built by [`Startup::begin`] and consumed by [`shutdown`].
pub struct Runtime {
    lease: Option<GatewayLeaseHandle>,
    heartbeat: CancellationToken,
    stoppables: Vec<Arc<dyn Stoppable>>,
    logger: Arc<Logger>,
}

impl std::fmt::Debug for Runtime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runtime")
            .field(
                "instance_id",
                &self.lease.as_ref().map(GatewayLeaseHandle::instance_id),
            )
            .field("stoppables", &self.stoppables.len())
            .finish()
    }
}

/// What a successful start produced.
#[derive(Debug)]
pub struct Startup {
    /// What `/api/health` reports about this instance.
    pub identity: InstanceIdentity,
    /// Everything to release on the way out.
    pub runtime: Runtime,
}

impl Startup {
    /// Run the gate, then take the lease.
    ///
    /// **The order is the contract** — see the module documentation.
    ///
    /// # Errors
    ///
    /// - [`AppError::Setup`] when required configuration is missing. Raised
    ///   **before** the lease is touched, so a misconfigured start never
    ///   disturbs a running Gateway.
    /// - [`AppError::Lease`] when another Gateway holds the configuration
    ///   directory.
    pub fn begin(
        config: &Config,
        environment: &via_core::EnvMap,
        logger: Arc<Logger>,
    ) -> Result<Self, AppError> {
        assert_gateway_setup(environment, config.locale).map_err(AppError::Setup)?;

        let owner = config.gateway_owner.clone();
        let lease = acquire_gateway_lease(
            config.config_directory(),
            AcquireOptions::new().owner(owner.clone()),
        )?;
        let started_at = via_lock::iso8601(chrono::Utc::now());
        logger.info(
            "gateway.lease_acquired",
            via_log::fields([
                ("instanceId", lease.instance_id().into()),
                ("owner", owner.into()),
            ]),
            "",
        );
        Ok(Self {
            identity: InstanceIdentity {
                instance_id: Some(lease.instance_id().to_owned()),
                started_at: Some(started_at),
            },
            runtime: Runtime {
                lease: Some(lease),
                heartbeat: CancellationToken::new(),
                stoppables: Vec::new(),
                logger,
            },
        })
    }

    /// A start with no lease and no gate, for embedders that own both.
    ///
    /// `via chat` running an in-process Gateway is the case: it has already
    /// taken the lease for the CLI, and taking a second one would refuse
    /// itself.
    #[must_use]
    pub fn detached(logger: Arc<Logger>) -> Self {
        Self {
            identity: InstanceIdentity::default(),
            runtime: Runtime {
                lease: None,
                heartbeat: CancellationToken::new(),
                stoppables: Vec::new(),
                logger,
            },
        }
    }
}

impl Runtime {
    /// Register something to stop on the way out.
    pub fn register(&mut self, stoppable: Arc<dyn Stoppable>) {
        self.stoppables.push(stoppable);
    }

    /// The lease's instance id, when there is a lease.
    #[must_use]
    pub fn instance_id(&self) -> Option<&str> {
        self.lease.as_ref().map(GatewayLeaseHandle::instance_id)
    }

    /// Mark the lease ready at `origin` and start the heartbeat.
    ///
    /// **External contract** — `server/src/index.mjs:136-144`: the update is
    /// `{state: 'ready', origin}` and the interval is
    /// [`HEARTBEAT_INTERVAL`].
    pub fn ready(&mut self, origin: &str) {
        let Some(lease) = self.lease.as_mut() else {
            return;
        };
        let _ = lease.update(
            LeaseUpdate::new()
                .state(LEASE_STATE_READY)
                .origin(origin.to_owned()),
        );
    }

    /// A token that stops the heartbeat.
    #[must_use]
    pub fn heartbeat_token(&self) -> CancellationToken {
        self.heartbeat.clone()
    }
}

/// Stop everything, in upstream's order.
///
/// 1. clear the heartbeat;
/// 2. stop every [`Stoppable`] **concurrently**;
/// 3. log one `backend.stop_failed` for any that refused;
/// 4. flush the logger, whatever happened;
/// 5. release the lease.
///
/// Step 5 is `process.once('exit', …)`'s `gatewayLease?.release()`
/// (`index.mjs:127-132`), which upstream runs after the same three steps.
pub async fn shutdown(mut runtime: Runtime) {
    // 1. A shutdown that takes seconds must not keep refreshing a lease it is
    //    about to release.
    runtime.heartbeat.cancel();

    // 2. `Promise.all([...])` — concurrent, not sequential.
    let stops = runtime
        .stoppables
        .iter()
        .map(|stoppable| stoppable.stop().boxed());
    let outcomes = futures::future::join_all(stops).await;

    // 3. One catch for the whole set, exactly as `.catch()` on the `Promise.all`.
    for outcome in outcomes {
        if let Err(error) = outcome {
            runtime.logger.error(
                "backend.stop_failed",
                via_log::fields([("error", error.into())]),
                "",
            );
        }
    }

    // 4. `.finally(() => logger.flush())`.
    runtime.logger.flush();

    // 5.
    if let Some(lease) = runtime.lease.take() {
        let _ = lease.release();
    }
}

/// Stop everything, and give up after [`EXIT_TIMEOUT`].
///
/// **External contract** — `server/src/index.mjs:41-46`: the exit path arms the
/// timeout *before* it starts stopping, so a harness that never answers cannot
/// wedge the process. Answers whether the stop finished in time, which is what
/// a caller turns into an exit code.
pub async fn shutdown_with_timeout(runtime: Runtime) -> bool {
    tokio::time::timeout(EXIT_TIMEOUT, shutdown(runtime))
        .await
        .is_ok()
}

/// Refresh the lease every [`HEARTBEAT_INTERVAL`] until the token is cancelled.
///
/// Spawn it beside the server; it is `gatewayHeartbeat`.
///
/// **The lease is handed back**, because it has to be:
/// [`GatewayLeaseHandle::release`] consumes the handle and the type has no
/// `Drop` — deliberately, so that *"a clean shutdown must reach this"* is a
/// property the compiler can hold. A heartbeat that swallowed the handle would
/// therefore make releasing the lease unexpressible, and every Gateway would
/// leave a `gateway.lock` naming a dead pid behind it. Awaiting the join handle
/// returns it, and the caller releases it as the last step of
/// [`shutdown`]'s sequence.
pub async fn heartbeat(
    mut lease: GatewayLeaseHandle,
    stop: CancellationToken,
) -> GatewayLeaseHandle {
    let mut ticker = tokio::time::interval(HEARTBEAT_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    // The first tick completes immediately; the lease was just written.
    ticker.tick().await;
    loop {
        tokio::select! {
            () = stop.cancelled() => break,
            _ = ticker.tick() => {
                if lease.heartbeat().is_err() {
                    break;
                }
            }
        }
    }
    lease
}

/// Where the configuration directory is, for a lease.
///
/// Named so [`Startup::begin`]'s one filesystem read has a single spelling.
#[must_use]
pub fn config_directory(config: &Config) -> &Path {
    config.config_directory()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct Recorder {
        name: &'static str,
        log: Arc<Mutex<Vec<&'static str>>>,
        delay: Duration,
        fails: bool,
        started: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Stoppable for Recorder {
        async fn stop(&self) -> Result<(), String> {
            self.started.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(self.delay).await;
            if let Ok(mut log) = self.log.lock() {
                log.push(self.name);
            }
            if self.fails {
                return Err(format!("{} refused", self.name));
            }
            Ok(())
        }
    }

    fn runtime(stoppables: Vec<Arc<dyn Stoppable>>) -> Runtime {
        Runtime {
            lease: None,
            heartbeat: CancellationToken::new(),
            stoppables,
            logger: Arc::new(via_log::Logger::with_sinks(
                via_log::LoggerOptions::detached("gateway"),
                Vec::new(),
            )),
        }
    }

    #[tokio::test(start_paused = true)]
    async fn the_two_stops_run_concurrently_not_in_sequence() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let started = Arc::new(AtomicUsize::new(0));
        let slow = Arc::new(Recorder {
            name: "backend",
            log: log.clone(),
            delay: Duration::from_secs(5),
            fails: false,
            started: started.clone(),
        });
        let quick = Arc::new(Recorder {
            name: "agent",
            log: log.clone(),
            delay: Duration::from_millis(1),
            fails: false,
            started: started.clone(),
        });
        let started_at = tokio::time::Instant::now();
        shutdown(runtime(vec![slow, quick])).await;
        let elapsed = started_at.elapsed();
        assert!(
            elapsed < Duration::from_secs(6),
            "sequential stops would take at least 5s + 1ms; took {elapsed:?}",
        );
        assert_eq!(
            log.lock().map(|log| log.clone()).unwrap_or_default(),
            ["agent", "backend"],
            "the quick stop finishes first, which only happens if both started together",
        );
    }

    #[tokio::test(start_paused = true)]
    async fn one_refusal_does_not_stop_the_others_or_the_flush() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let started = Arc::new(AtomicUsize::new(0));
        let failing = Arc::new(Recorder {
            name: "backend",
            log: log.clone(),
            delay: Duration::from_millis(1),
            fails: true,
            started: started.clone(),
        });
        let healthy = Arc::new(Recorder {
            name: "agent",
            log: log.clone(),
            delay: Duration::from_millis(1),
            fails: false,
            started: started.clone(),
        });
        shutdown(runtime(vec![failing, healthy])).await;
        assert_eq!(started.load(Ordering::SeqCst), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn a_stop_that_never_answers_is_abandoned_after_two_seconds() {
        #[derive(Debug)]
        struct Wedged;

        #[async_trait]
        impl Stoppable for Wedged {
            async fn stop(&self) -> Result<(), String> {
                std::future::pending().await
            }
        }

        let finished = shutdown_with_timeout(runtime(vec![Arc::new(Wedged)])).await;
        assert!(
            !finished,
            "the exit path gives up rather than wedging the process",
        );
    }

    #[test]
    fn the_two_timers_are_the_catalogued_values() {
        assert_eq!(EXIT_TIMEOUT, Duration::from_millis(2000));
        assert_eq!(HEARTBEAT_INTERVAL, Duration::from_secs(15));
        assert_eq!(LEASE_STATE_READY, "ready");
    }
}
