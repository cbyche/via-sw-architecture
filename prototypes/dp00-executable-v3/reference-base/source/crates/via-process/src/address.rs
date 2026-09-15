//! Loopback address allocation: is this port taken, and if so, which one is
//! free?
//!
//! Ported from `server/src/process/managed-backend.mjs:75-108`.
//!
//! `state-name/managed backend readiness / restart / shutdown` describes the
//! whole of it: *"Pre-spawn it probes `backendAddressInUse` (TCP connect,
//! 300 ms timeout, connect → true, timeout/error → false) and, if busy, calls
//! `allocateBackendAddress` (listen on port 0 at the same host, read the
//! assigned port, close, return origin) then writes the new URL+PORT into
//! env."*
//!
//! # Why a connect and not a bind
//!
//! Probing by *binding* would answer "can I have this port", which is a
//! different question: a `SO_REUSEADDR` socket in `TIME_WAIT`, or a listener
//! on `0.0.0.0` when the probe binds `127.0.0.1`, both give the wrong answer.
//! Connecting asks "is something answering here", which is the question that
//! decides whether starting a second backend on this port would collide.
//!
//! # Why every failure means "free"
//!
//! A refused connection, an unreachable host and a timeout all resolve to
//! `false`. That is deliberate: the probe's only job is to decide whether to
//! move, and moving unnecessarily costs nothing while failing to move
//! collides. The 300 ms bound is what keeps a hung loopback from stalling
//! startup.

use std::time::Duration;

use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream};

use crate::endpoint::{hostname_of, origin_of, parse_service_endpoint, port_of};
use crate::error::ProcessError;

/// How long the in-use probe waits for a connection.
///
/// **External contract** — `server/src/process/managed-backend.mjs:75`,
/// catalogued under `default-value/timing constants` as
/// *"backendAddressInUse probe 300ms"*.
pub const ADDRESS_PROBE_TIMEOUT: Duration = Duration::from_millis(300);

/// The host upstream substitutes for `localhost` before binding.
///
/// **External contract** — `server/src/process/managed-backend.mjs:94`. A
/// bind to the name `localhost` may land on `::1` or `127.0.0.1` depending on
/// the resolver, and the allocated port must be the one the child is told
/// about, so the name is pinned to an address first.
pub const LOCALHOST_BIND_HOST: &str = "127.0.0.1";

/// The name that gets substituted.
pub const LOCALHOST_NAME: &str = "localhost";

/// Probing and allocating a loopback address.
///
/// A trait so a test can assert the *decisions* — that an external backend
/// never probes, that a busy address moves — without binding a real socket,
/// exactly as upstream's `isAddressInUse` / `findFreeAddress` injection
/// points do (`server/test/managed-backend.test.mjs:118-119,193-195`).
#[async_trait]
pub trait AddressAllocator: Send + Sync {
    /// Whether something is already answering at this address.
    ///
    /// # Errors
    ///
    /// Only [`ProcessError::InvalidServiceUrl`]. A socket failure is not an
    /// error here — it is the answer `false`.
    async fn address_in_use(&self, base_url: &str) -> Result<bool, ProcessError>;

    /// A free address on the same host, as an origin.
    ///
    /// # Errors
    ///
    /// [`ProcessError::InvalidServiceUrl`] or [`ProcessError::Io`] when no
    /// ephemeral port can be bound.
    async fn allocate(&self, base_url: &str) -> Result<String, ProcessError>;
}

/// The real allocator, over tokio's sockets.
#[derive(Debug, Clone, Copy, Default)]
pub struct TokioAddressAllocator {
    /// The probe timeout. [`ADDRESS_PROBE_TIMEOUT`] unless overridden.
    timeout: Option<Duration>,
}

impl TokioAddressAllocator {
    /// An allocator with upstream's 300 ms probe timeout.
    #[must_use]
    pub const fn new() -> Self {
        Self { timeout: None }
    }

    /// An allocator with a different probe timeout.
    ///
    /// Upstream's `timeoutMs` parameter, which exists for the same reason:
    /// a test should not have to wait 300 ms to prove the timeout arm.
    #[must_use]
    pub const fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout: Some(timeout),
        }
    }

    fn probe_timeout(self) -> Duration {
        self.timeout.unwrap_or(ADDRESS_PROBE_TIMEOUT)
    }
}

#[async_trait]
impl AddressAllocator for TokioAddressAllocator {
    async fn address_in_use(&self, base_url: &str) -> Result<bool, ProcessError> {
        let target = parse_service_endpoint(base_url)?;
        let authority = format!(
            "{host}:{port}",
            host = hostname_of(&target),
            port = port_of(&target)
        );
        // connect -> true; timeout, refusal, DNS failure -> false.
        Ok(matches!(
            tokio::time::timeout(self.probe_timeout(), TcpStream::connect(authority)).await,
            Ok(Ok(_))
        ))
    }

    async fn allocate(&self, base_url: &str) -> Result<String, ProcessError> {
        let mut target = parse_service_endpoint(base_url)?;
        let host = match hostname_of(&target).as_str() {
            LOCALHOST_NAME => LOCALHOST_BIND_HOST.to_owned(),
            other => other.to_owned(),
        };
        let listener = TcpListener::bind(format!("{host}:0"))
            .await
            .map_err(|error| ProcessError::io("bind an ephemeral backend port", error))?;
        let port = listener
            .local_addr()
            .map_err(|error| ProcessError::io("read the allocated backend port", error))?
            .port();
        // Upstream closes the server before returning the origin, so the child
        // can bind the port it was just handed. Dropping is that close.
        drop(listener);
        if target.set_host(Some(&host)).is_err() || target.set_port(Some(port)).is_err() {
            return Err(ProcessError::InvalidServiceUrl {
                value: base_url.to_owned(),
            });
        }
        Ok(origin_of(&target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_listening_socket_reads_as_in_use() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let allocator = TokioAddressAllocator::new();
        assert!(
            allocator
                .address_in_use(&format!("http://127.0.0.1:{port}"))
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn a_closed_port_reads_as_free() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let allocator = TokioAddressAllocator::new();
        assert!(
            !allocator
                .address_in_use(&format!("http://127.0.0.1:{port}"))
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn allocation_pins_localhost_to_an_address_and_keeps_the_scheme() {
        let allocated = TokioAddressAllocator::new()
            .allocate("http://localhost:18789")
            .await
            .unwrap();
        assert!(
            allocated.starts_with("http://127.0.0.1:"),
            "expected a pinned loopback origin, got {allocated}"
        );
        let port: u16 = allocated.rsplit(':').next().unwrap().parse().unwrap();
        assert_ne!(
            port, 0,
            "an ephemeral port must be read back, not left at 0"
        );
    }

    #[tokio::test]
    async fn allocation_answers_an_origin_with_no_path() {
        let allocated = TokioAddressAllocator::new()
            .allocate("http://127.0.0.1:4096/path?query#fragment")
            .await
            .unwrap();
        assert!(!allocated.contains("/path"), "{allocated}");
        assert!(!allocated.contains('?'), "{allocated}");
    }

    #[tokio::test]
    async fn an_unroutable_address_reads_as_free_within_the_probe_budget() {
        // TEST-NET-1 (RFC 5737) never answers; the probe must give up rather
        // than inherit the OS connect timeout.
        let allocator = TokioAddressAllocator::with_timeout(Duration::from_millis(50));
        let started = std::time::Instant::now();
        assert!(
            !allocator
                .address_in_use("http://192.0.2.1:9")
                .await
                .unwrap()
        );
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "the probe did not honour its timeout: {:?}",
            started.elapsed()
        );
    }
}
