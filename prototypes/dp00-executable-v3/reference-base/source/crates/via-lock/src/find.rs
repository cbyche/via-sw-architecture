//! Finding a Gateway that is already running.
//!
//! Ported from upstream `findRunningGateway`
//! (`shared/gateway-instance-lock.mjs:168-188`).

use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::lease::{GatewayLease, read_gateway_lease};

/// The `/api/health` field that carries the Gateway's instance identity.
///
/// **External contract.** Upstream `server/src/app/gateway-application.mjs:224-268`
/// serves it, and `shared/gateway-instance-lock.mjs:179` compares it against
/// the lease's `instanceId`.
pub const HEALTH_INSTANCE_ID_FIELD: &str = "gatewayInstanceId";

/// Reads `/api/health` from a Gateway origin.
///
/// Injected rather than implemented here: this is a leaf crate in phase 0 and
/// the workspace has no HTTP client yet, and keeping the probe a seam is also
/// what lets the discovery race be tested without a socket. A bare closure
/// `|origin: &str| -> Option<Value>` implements it.
///
/// The document is returned whole — [`RunningGateway`] hands it back to the
/// caller, which reads far more of it than this crate does. `None` means the
/// origin did not answer, or did not answer with JSON.
pub trait HealthProbe {
    /// Fetch `<origin>/api/health`.
    fn read_health(&self, origin: &str) -> Option<Value>;
}

impl<F> HealthProbe for F
where
    F: Fn(&str) -> Option<Value>,
{
    fn read_health(&self, origin: &str) -> Option<Value> {
        self(origin)
    }
}

/// Read `gatewayInstanceId` out of a `/api/health` document.
///
/// A missing field, or one that is not a string, answers `None` — which never
/// equals a lease's `instanceId`, so an endpoint that is not a VIA Gateway can
/// never be mistaken for one.
#[must_use]
pub fn health_instance_id(health: &Value) -> Option<&str> {
    health.get(HEALTH_INSTANCE_ID_FIELD)?.as_str()
}

/// How long [`find_running_gateway`] waits for a starting Gateway.
#[derive(Debug, Clone, Copy)]
pub struct FindOptions {
    /// Total time to keep polling. `Duration::ZERO` means a single pass.
    ///
    /// **External contract.** Upstream default 3000 ms
    /// (`shared/gateway-instance-lock.mjs:170`).
    pub timeout: Duration,
    /// Delay between passes.
    ///
    /// **External contract.** Upstream default 100 ms
    /// (`shared/gateway-instance-lock.mjs:171`).
    pub interval: Duration,
}

impl Default for FindOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(3000),
            interval: Duration::from_millis(100),
        }
    }
}

impl FindOptions {
    /// Upstream's defaults: 3000 ms with a 100 ms poll.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the total wait.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Override the poll interval.
    #[must_use]
    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }
}

/// A Gateway that is running, reachable, and provably the one the lease names.
#[derive(Debug, Clone)]
pub struct RunningGateway {
    /// The lease as read from disk.
    pub lease: GatewayLease,
    /// `lease.origin`, lifted out because every caller wants it.
    pub origin: String,
    /// The `/api/health` document the origin answered with.
    pub health: Value,
}

/// Find the Gateway serving this configuration directory, if one is up.
///
/// **External contract.** Upstream `findRunningGateway`
/// (`shared/gateway-instance-lock.mjs:168-188`), including three behaviours
/// that look incidental and are not:
///
/// * **No lease means no Gateway, immediately.** Missing file, unparseable
///   file, wrong schema — the function returns `None` without waiting out the
///   timeout. There is nothing to wait *for*.
/// * **An empty `origin` keeps polling.** The lease exists but the Gateway has
///   not bound its listener yet, which is exactly the race a second launch is
///   trying to lose gracefully. The health probe is not called until there is
///   an origin to call it on.
/// * **The identity must match.** The origin only counts when
///   `health.gatewayInstanceId` equals the lease's `instanceId`. A port that a
///   stranger reused — a dead Gateway's port picked up by some other server,
///   or a stale lease pointing at a live-but-unrelated service — therefore
///   reads as "not running" rather than being handed to the caller as a
///   Gateway.
///
/// This port is synchronous where upstream is `async`: phase 0 has no runtime,
/// and the only await was the poll delay. The health probe is a seam either
/// way.
pub fn find_running_gateway(
    config_directory: &Path,
    probe: &dyn HealthProbe,
    options: FindOptions,
) -> Option<RunningGateway> {
    let deadline = Instant::now() + options.timeout;
    loop {
        let lease = read_gateway_lease(config_directory)?;
        if !lease.origin.is_empty()
            && let Some(health) = probe.read_health(&lease.origin)
            && health_instance_id(&health) == Some(lease.instance_id.as_str())
        {
            return Some(RunningGateway {
                origin: lease.origin.clone(),
                lease,
                health,
            });
        }
        if Instant::now() >= deadline {
            return None;
        }
        thread::sleep(options.interval);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_instance_id_must_be_a_string() {
        assert_eq!(
            health_instance_id(&json!({ "gatewayInstanceId": "abc" })),
            Some("abc")
        );
        assert_eq!(health_instance_id(&json!({ "gatewayInstanceId": 7 })), None);
        assert_eq!(health_instance_id(&json!({ "ok": true })), None);
        assert_eq!(health_instance_id(&json!("not an object")), None);
    }

    #[test]
    fn the_polling_defaults_match_upstream() {
        let defaults = FindOptions::new();
        assert_eq!(defaults.timeout, Duration::from_millis(3000));
        assert_eq!(defaults.interval, Duration::from_millis(100));
    }
}
