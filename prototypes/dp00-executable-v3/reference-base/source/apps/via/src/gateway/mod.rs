//! The Gateway runtime — the process around `via-app`.
//!
//! `via-app` is the composition root and the server. This module is everything
//! that can only happen in a *process*: the single-instance lease and its
//! heartbeat, the two signals, the banner, and the exit code. It is the Rust
//! shape of `server/src/index.mjs` plus the `command === 'gateway'` arm of
//! `cli/src/launcher.mjs:381-421`.
//!
//! # The order, and why each step is where it is
//!
//! ```text
//! plan / apply arguments        // cli/src/launcher.mjs:55-86
//! resolve configuration
//! assertGatewaySetup            // BEFORE the lease is touched
//! acquireGatewayLease           // one Gateway per configuration directory
//! compose services              // gateway-application.mjs:41-54
//! bind
//! lease.update({state:'ready', origin})
//! setInterval(heartbeat, 15_000)
//! …serve…
//! SIGINT / SIGTERM → close, in upstream's order
//! ```
//!
//! **The gate comes before the lease** and upstream says why on the line above
//! it: *"A Gateway that listens but cannot connect its voice is harder to
//! diagnose than a refusal the user can act on."* A misconfigured start must
//! not disturb a running one.
//!
//! **The lease is held for the process's life, not taken and released.** Phase 1
//! took it only to prove it was available; a running Gateway holds it, refreshes
//! it every [`via_lock::GATEWAY_HEARTBEAT_INTERVAL`], publishes its origin on it
//! so `via chat` can find it, and gives it back on the way out.
//!
//! # Why this module owns the lease rather than [`via_app::Startup::begin`]
//!
//! [`via_app::heartbeat`] takes a [`via_lock::GatewayLeaseHandle`] **by value**,
//! and `Startup::begin` keeps the one it acquired inside its own `Runtime`. So
//! an embedder that wants both a lease and a heartbeat has to own the lease. The
//! gate-then-lease ordering `Startup::begin` exists to guarantee is reproduced
//! here literally, and `tests/gateway_lease.rs` asserts it the same way
//! `via-app`'s own test does — by checking that `gateway.lock` does not exist
//! after a refused start.

pub mod compose;
pub mod delegation;
pub mod engine;
pub mod frontend;
pub mod opener;

use std::io::Write;
use std::sync::Arc;

use tokio_util::sync::CancellationToken;
use via_app::{GatewayApplication, InstanceIdentity, Serving};
use via_core::{Config, EnvMap};
use via_i18n::{Locale, format, keys, t};
use via_lock::{
    AcquireOptions, GatewayLeaseHandle, LEASE_STATE_READY, LeaseUpdate, acquire_gateway_lease,
};

pub use compose::{Composed, Composition};
pub use engine::{EngineServices, RealtimeEngine, RealtimeEngineFactory};
pub use opener::{ConnectOpener, SessionOpener};

use crate::error::CliError;

/// The `owner` this binary stamps into the lease.
///
/// **External contract** — `server/src/index.mjs:51-53` passes `"cli"` or
/// `"desktop"`; VIA has no desktop host, so the Gateway is always CLI-owned.
pub const LEASE_OWNER: &str = crate::commands::gateway::LEASE_OWNER;

/// A Gateway that has taken its lease and bound its socket.
///
/// Split out from [`serve`] so a test can drive the *same* boot to the point of
/// listening, take the address, and shut it down deliberately rather than by
/// signal.
pub struct Booted {
    /// The bound server, ready to run.
    pub serving: Serving,
    /// Everything the composition produced.
    pub composed: Composed,
    /// The lease, held for the process's life.
    pub lease: GatewayLeaseHandle,
    /// Cancels the heartbeat.
    pub heartbeat: CancellationToken,
}

impl std::fmt::Debug for Booted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Booted")
            .field("origin", &self.serving.origin())
            .field("instance_id", &self.lease.instance_id())
            .finish_non_exhaustive()
    }
}

impl Booted {
    /// `http://<host>:<boundPort>` — the origin published on the lease.
    #[must_use]
    pub fn origin(&self) -> String {
        self.serving.origin()
    }
}

/// Run the gate, take the lease, compose, and bind.
///
/// # Errors
///
/// * [`CliError::Core`] when the setup gate refuses — **before** the lease is
///   touched.
/// * [`CliError::Refused`] with `VIA_GATEWAY_ALREADY_RUNNING` when another
///   Gateway holds this configuration directory, and for a socket that cannot
///   be bound.
pub async fn boot(
    config: &Config,
    environment: &EnvMap,
    composition: Composition,
) -> Result<Booted, CliError> {
    // 1. The gate, before anything on disk is touched.
    via_core::assert_gateway_setup(environment, config.locale)?;

    // 2. The lease.
    let mut lease = acquire_gateway_lease(
        config.config_directory(),
        AcquireOptions::new().owner(LEASE_OWNER),
    )
    .map_err(|error| crate::commands::gateway::lease_error(error, config.locale))?;
    let started_at = via_lock::iso8601(chrono::Utc::now());
    let identity = InstanceIdentity {
        instance_id: Some(lease.instance_id().to_owned()),
        started_at: Some(started_at),
    };

    // 3. Everything below the socket.
    let composed = composition.compose(config, environment).await?;
    composed.services.logger.info(
        "gateway.lease_acquired",
        via_log::fields([
            ("instanceId", lease.instance_id().into()),
            ("owner", LEASE_OWNER.into()),
        ]),
        "",
    );

    // 4. The socket.
    let application = GatewayApplication::build_with(
        composed.services.clone(),
        identity,
        Some(composed.engines.clone()),
    );
    let serving = application.bind_configured().await.map_err(|error| {
        // The lease is released by the `Drop` of the handle we are about to
        // discard, so a failed bind does not leave a lease naming a process
        // that never listened.
        CliError::Refused {
            code: via_protocol::CODE_GATEWAY_SETUP_REQUIRED,
            message: error.to_string(),
        }
    })?;

    // 5. Publish the origin so `via chat` can find this Gateway by its lease
    //    rather than by guessing the port (`server/src/index.mjs:136-140`).
    let origin = serving.origin();
    let _ = lease.update(
        LeaseUpdate::new()
            .state(LEASE_STATE_READY)
            .origin(origin.clone()),
    );

    Ok(Booted {
        serving,
        composed,
        lease,
        heartbeat: CancellationToken::new(),
    })
}

/// Serve until a signal arrives, then close in upstream's order.
///
/// The banner is written to `out` before the first request can arrive, because
/// a user who typed `via gateway && open …` needs the address on the line
/// *before* the Gateway starts logging.
///
/// # Errors
///
/// Whatever [`boot`] refuses with. A clean shutdown is `Ok(())`.
pub async fn serve(
    config: &Config,
    environment: &EnvMap,
    composition: Composition,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    let booted = boot(config, environment, composition).await?;
    // The signal handlers are installed **before** the banner, because the
    // banner is what tells a supervisor the Gateway is up — and a SIGTERM that
    // arrives between the two would otherwise take its default disposition and
    // kill the process with the lease still on disk.
    let stop = shutdown_signal();
    let origin = booted.origin();
    let summary = summary(booted.serving.state(), config.locale).await;
    writeln!(
        out,
        "{}{summary}",
        format(
            config.locale,
            keys::CLI_GATEWAY_BANNER_STARTED,
            &[("url", &origin)],
        )
    )
    .map_err(|error| CliError::io("write", "<stdout>", error))?;
    out.flush()
        .map_err(|error| CliError::io("flush", "<stdout>", error))?;

    run(booted, stop).await;
    Ok(())
}

/// Serve `booted` until `stop` resolves, then close everything.
///
/// The close sequence is `via_app::runtime::shutdown`'s, with the two things
/// this module owns bracketed around it: the heartbeat is cancelled **first**
/// so a shutdown that takes seconds does not keep refreshing a lease it is
/// about to release, and the lease is released **last**.
pub async fn run(booted: Booted, stop: impl Future<Output = ()> + Send) {
    let Booted {
        serving,
        composed,
        lease,
        heartbeat,
    } = booted;

    let shutdown = serving.shutdown_token();
    let beating = tokio::spawn(via_app::heartbeat(lease, heartbeat.clone()));
    let served = tokio::spawn(serving.run());

    stop.await;
    // 1. Stop refreshing a lease that is about to go back.
    heartbeat.cancel();
    // 2. Stop listening; `Serving::run` then runs `via-app`'s close sequence.
    shutdown.cancel();
    let _ = served.await;

    // 3. Layer 3, and the tasks every engine spawned.
    // `Arc::into_inner` is the "nobody else holds this" check: the runners hold
    // clones for as long as a delegation is in flight, so a coordinator that is
    // still referenced is one still working and must not be closed under it.
    if let Some(coordinator) = composed.coordinator
        && let Some(coordinator) = Arc::into_inner(coordinator)
    {
        coordinator.close().await;
    }
    composed.tracker.close();
    // 4. `.finally(() => logger.flush())` — the log line explaining a failed
    //    stop is the one thing a user has afterwards.
    composed.services.logger.flush();

    // 5. `process.once('exit', …)`'s `gatewayLease?.release()`
    //    (`index.mjs:127-132`), and it is **last**. The heartbeat task owns the
    //    lease while it runs and hands it back when it stops, which is the only
    //    way to release a handle the heartbeat was given — see
    //    [`via_app::heartbeat`].
    if let Ok(lease) = beating.await {
        let _ = lease.release();
    }
}

/// The banner's second line.
///
/// **External contract** — `gatewaySummary(health)`,
/// `cli/src/launcher.mjs:88-102`: the Realtime label, then either
/// *voice-chat-only mode* or `<backend label> <connected|not connected>`,
/// joined with ` · `, with empty parts dropped. VIA reads the same
/// `/api/health` payload the CLI would have fetched, from the state that
/// serves it.
async fn summary(state: &via_app::AppState, locale: Locale) -> String {
    let Ok(health) = state.health().await else {
        return String::new();
    };
    let model = health
        .realtime_model_profile
        .as_ref()
        .map(|profile| profile.label.to_string())
        .filter(|label| !label.is_empty())
        .or_else(|| Some(health.realtime_label.clone()).filter(|label| !label.is_empty()))
        .or_else(|| health.realtime_model.clone())
        .unwrap_or_default();
    let realtime = if model.is_empty() {
        String::new()
    } else {
        format(
            locale,
            keys::CLI_GATEWAY_SUMMARY_REALTIME,
            &[("model", &model)],
        )
    };

    let enabled = health
        .backend
        .get("enabled")
        .and_then(serde_json::Value::as_bool)
        != Some(false);
    let backend = if enabled {
        let label = ["label", "kind", "protocol"]
            .into_iter()
            .find_map(|key| {
                health
                    .backend
                    .get(key)
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.is_empty())
            })
            .map_or_else(
                || t(locale, keys::GATEWAY_BACKEND_AGENT_LABEL).to_owned(),
                str::to_owned,
            );
        let ok = health
            .backend
            .get("ok")
            .and_then(serde_json::Value::as_bool)
            == Some(true);
        let state = if ok {
            t(locale, keys::GATEWAY_CONNECTED)
        } else {
            t(locale, keys::GATEWAY_DISCONNECTED)
        };
        std::format!("{label} {state}")
    } else {
        t(locale, keys::GATEWAY_FRONTEND_ONLY_MODE).to_owned()
    };

    [realtime, backend]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(SUMMARY_SEPARATOR)
}

/// What the banner's two parts are joined with.
///
/// **External contract** — `cli/src/launcher.mjs:94,101`: `.join(' · ')`.
pub const SUMMARY_SEPARATOR: &str = " · ";

/// Install the two signal handlers, and answer a future that resolves when
/// either fires.
///
/// **External contract** — `cli/src/launcher.mjs:387-394` handles both, and
/// treats SIGHUP as SIGTERM because *"a terminal or SSH session closing sends
/// SIGHUP rather than SIGINT"*. VIA's Gateway is this process rather than a
/// child of it, so SIGHUP's default disposition already terminates it; the two
/// signals upstream names explicitly are the two handled here.
///
/// **Registration is eager, and that is the point.** `tokio::signal::ctrl_c()`
/// registers on its first poll, so a future that is only awaited after the
/// banner leaves a window in which SIGTERM takes its default disposition —
/// killing the process with the lease still on disk. Both streams are opened
/// here, before the caller has printed anything.
pub fn shutdown_signal() -> impl Future<Output = ()> + Send {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let interrupt = signal(SignalKind::interrupt());
        let terminate = signal(SignalKind::terminate());
        async move {
            match (interrupt, terminate) {
                (Ok(mut interrupt), Ok(mut terminate)) => {
                    tokio::select! {
                        _ = interrupt.recv() => {}
                        _ = terminate.recv() => {}
                    }
                }
                // A reactor that cannot register a handler still stops on the
                // default disposition; losing the graceful close is better than
                // refusing to start.
                _ => std::future::pending().await,
            }
        }
    }
    #[cfg(not(unix))]
    {
        async {
            let _ = tokio::signal::ctrl_c().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn the_summary_separator_is_upstreams() {
        assert_eq!(SUMMARY_SEPARATOR, " · ");
    }

    #[test]
    fn the_lease_owner_is_the_cli() {
        assert_eq!(LEASE_OWNER, "cli");
        assert_ne!(LEASE_OWNER, via_lock::DEFAULT_LEASE_OWNER);
    }
}
