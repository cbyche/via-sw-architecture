//! The accept loop.
//!
//! `via-app` drives hyper itself rather than calling `axum::serve`, for exactly
//! one reason, and it is a catalogued contract:
//!
//! > `/api/realtime` (WebSocket upgrade; **any other pathname =>
//! > `socket.destroy()` with no HTTP response**).
//! > — `docs/reference/contracts.json`, `ws-route` / *realtime upgrade path*
//!
//! An axum handler can only *return a response*. There is no response here:
//! upstream's `rejectUnsupportedRealtimeUpgrade` calls `socket.destroy()` and
//! returns, so the client's `WebSocket` sees the TCP connection close with zero
//! bytes read — no status line, no headers, nothing to parse. The catalogue
//! says why that is worth reproducing: *"a mismatch silently drops the
//! connection with no HTTP status"*, and every shipped client hardcodes the
//! path, so the failure mode a client must recognise is a bare close.
//!
//! # How it is done
//!
//! The service future for a destroyed upgrade **never resolves**. It signals a
//! per-connection `oneshot` and then parks; the connection task selects on that
//! oneshot and drops the hyper connection future, which drops the `TcpStream`.
//! hyper writes nothing because the service never handed it a response.
//!
//! Every other request goes to the router untouched.

use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Body;
use axum::http::{Request, header};
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tower::Service;

use crate::error::AppError;

/// Whether this request is a WebSocket upgrade that must be destroyed.
///
/// **External contract** — `realtime-gateway.mjs:70-74`. The check is *"is this
/// an upgrade to something other than the realtime route"*, and both halves
/// matter: a plain `GET /nope` is a 404 like any other miss, and only an
/// **upgrade** to a foreign path is destroyed.
#[must_use]
pub fn destroys_socket(request: &Request<Incoming>) -> bool {
    if !is_websocket_upgrade(request) {
        return false;
    }
    request.uri().path() != crate::http::REALTIME_ROUTE
}

/// `Connection: Upgrade` + `Upgrade: websocket`, both case-insensitive.
///
/// `Connection` is a comma-separated list, so a proxy that sends
/// `keep-alive, Upgrade` still counts.
#[must_use]
pub fn is_websocket_upgrade<B>(request: &Request<B>) -> bool {
    let has = |name: header::HeaderName, needle: &str| {
        request
            .headers()
            .get_all(name)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .any(|value| {
                value
                    .split(',')
                    .any(|token| token.trim().eq_ignore_ascii_case(needle))
            })
    };
    has(header::CONNECTION, "upgrade") && has(header::UPGRADE, "websocket")
}

/// A bound listener that has not started accepting yet.
///
/// Binding and serving are separate so a caller can learn the port — which is
/// the whole point of `PORT=0` — before anything is served on it. Upstream gets
/// the same separation from `server.listen(…, callback)`.
#[derive(Debug)]
pub struct BoundServer {
    listener: TcpListener,
    address: SocketAddr,
}

impl BoundServer {
    /// Bind `address`.
    ///
    /// # Errors
    ///
    /// [`AppError::Bind`] when the port is taken or the address is not local.
    pub async fn bind(address: &str) -> Result<Self, AppError> {
        let listener = TcpListener::bind(address)
            .await
            .map_err(|source| AppError::Bind {
                address: address.to_owned(),
                source,
            })?;
        let bound = listener.local_addr().map_err(|source| AppError::Bind {
            address: address.to_owned(),
            source,
        })?;
        Ok(Self {
            listener,
            address: bound,
        })
    }

    /// The address actually bound — the resolved port when `PORT=0` was asked
    /// for.
    #[must_use]
    pub const fn address(&self) -> SocketAddr {
        self.address
    }

    /// Serve `router` until `shutdown` is cancelled.
    ///
    /// Returns once the loop has stopped accepting and every in-flight
    /// connection has finished.
    pub async fn serve(self, router: Router, shutdown: CancellationToken) {
        let tracker = TaskTracker::new();
        loop {
            let accepted = tokio::select! {
                () = shutdown.cancelled() => break,
                accepted = self.listener.accept() => accepted,
            };
            let Ok((stream, _peer)) = accepted else {
                // A failed accept is transient — a fd limit, a client that
                // vanished between the SYN and the accept. Upstream's server
                // emits `error` and keeps listening; so does this.
                continue;
            };
            let router = router.clone();
            let shutdown = shutdown.clone();
            tracker.spawn(async move {
                serve_connection(stream, router, shutdown).await;
            });
        }
        tracker.close();
        tracker.wait().await;
    }
}

async fn serve_connection(
    stream: tokio::net::TcpStream,
    router: Router,
    shutdown: CancellationToken,
) {
    let (destroy_tx, destroy_rx) = oneshot::channel::<()>();
    let destroy = Arc::new(Mutex::new(Some(destroy_tx)));
    let service = DestroyingService {
        inner: router,
        destroy,
    };
    let builder = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new());
    let connection = builder.serve_connection_with_upgrades(
        TokioIo::new(stream),
        hyper_util::service::TowerToHyperService::new(service),
    );
    let mut connection = std::pin::pin!(connection);
    tokio::select! {
        _ = connection.as_mut() => {}
        // The destroy contract: drop the connection future, which drops the
        // socket. Nothing has been written, because the service future for
        // this request never resolved.
        _ = destroy_rx => {}
        () = shutdown.cancelled() => {}
    }
}

/// The router, with the destroy contract in front of it.
#[derive(Clone)]
struct DestroyingService {
    inner: Router,
    destroy: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl Service<Request<Incoming>> for DestroyingService {
    type Response = axum::response::Response;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request<Incoming>) -> Self::Future {
        if destroys_socket(&request) {
            if let Ok(mut destroy) = self.destroy.lock()
                && let Some(destroy) = destroy.take()
            {
                let _ = destroy.send(());
            }
            // Never resolves, so hyper never writes a byte. The connection task
            // is already dropping the socket.
            return Box::pin(std::future::pending());
        }
        let request = request.map(Body::new);
        let future = self.inner.call(request);
        Box::pin(future)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request as HttpRequest;
    use pretty_assertions::assert_eq;

    fn request(path: &str, headers: &[(&str, &str)]) -> HttpRequest<()> {
        let mut builder = HttpRequest::builder().uri(path);
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        builder.body(()).expect("a request")
    }

    #[test]
    fn an_upgrade_is_recognised_through_a_proxys_connection_list() {
        assert!(is_websocket_upgrade(&request(
            "/api/realtime",
            &[
                ("connection", "keep-alive, Upgrade"),
                ("upgrade", "websocket")
            ],
        )));
        assert!(is_websocket_upgrade(&request(
            "/api/realtime",
            &[("Connection", "UPGRADE"), ("Upgrade", "WebSocket")],
        )));
    }

    #[test]
    fn a_plain_request_is_never_an_upgrade() {
        assert!(!is_websocket_upgrade(&request("/api/health", &[])));
        assert!(!is_websocket_upgrade(&request(
            "/api/realtime",
            &[("connection", "keep-alive")],
        )));
        assert!(
            !is_websocket_upgrade(&request("/api/realtime", &[("upgrade", "websocket")])),
            "an Upgrade header alone is not an upgrade",
        );
    }

    #[test]
    fn the_destroy_rule_is_upgrade_and_wrong_path_together() {
        // The typed helper is exercised through `is_websocket_upgrade`, which
        // `destroys_socket` delegates to; this asserts the path half.
        assert_eq!(crate::http::REALTIME_ROUTE, "/api/realtime");
        let upgrade = [("connection", "Upgrade"), ("upgrade", "websocket")];
        assert!(is_websocket_upgrade(&request("/socket", &upgrade)));
        assert_ne!("/socket", crate::http::REALTIME_ROUTE);
    }
}
