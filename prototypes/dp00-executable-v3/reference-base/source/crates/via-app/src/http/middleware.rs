//! The middleware stack, in upstream's order.
//!
//! **The order is load-bearing** — `docs/reference/contracts.json`,
//! `http-behaviour` / *middleware order and limits*:
//!
//! ```text
//! app.disable('x-powered-by')                 // nothing to disable in axum
//! app.use(enforceSameOrigin)                  // origin_layer
//! app.use(identity + X-Request-Id + logctx)   // identity_layer
//! app.use(access log on res 'finish')         // access_log_layer
//! app.use(express.json({limit:'1mb'}))        // DefaultBodyLimit
//! ```
//!
//! and the catalogue says why it matters: *"an oversized body is rejected by
//! the JSON parser only after the origin check and identity issuance have
//! already run."*
//!
//! One consequence worth stating because it looks like a bug: an **origin
//! rejection carries no `X-Request-Id`**. Upstream's `enforceSameOrigin`
//! answers 403 and returns without calling `next()`
//! (`server/src/core/request-security.mjs:74-80`), so the header the *next*
//! middleware would have set is never set. The catalogue's *"observable
//! response header on all routes, including 403/404"* is about the routes'
//! own refusals, which do carry it.
//!
//! # `via-core` owns the rules
//!
//! The origin allow-list, the DNS-rebinding defence and the signed cookie are
//! [`via_core::security`] and [`via_core::identity`]. Nothing here restates a
//! rule; this module is the wiring that puts them in front of every route,
//! including `/livez`, `/readyz` and the WebSocket upgrade.

use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderName, HeaderValue, Request, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use uuid::Uuid;
use via_core::identity::HttpIdentity;
use via_core::security::{ORIGIN_NOT_ALLOWED, is_allowed_origin};
use via_log::{Logger, fields};

use crate::state::AppState;

/// The response header every request that gets past the origin check carries.
///
/// **External contract** — `docs/reference/contracts.json`, `http-header` /
/// *X-Request-Id*: `res.setHeader('X-Request-Id', randomUUID())`, and the same
/// value goes into the log context beside `ownerId`.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// The request extension the identity layer leaves behind.
///
/// Upstream writes `req.identity`; a typed extension is the same thing with a
/// compiler behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestIdentity {
    /// The resolved owner.
    pub owner_id: String,
    /// This request's id, as sent on `X-Request-Id`.
    pub request_id: String,
}

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for RequestIdentity {
    type Rejection = Response;

    /// Reads what [`identity_layer`] left behind.
    ///
    /// The rejection is unreachable through the assembled router — the layer is
    /// applied to every route including the fallback — so it answers 500 rather
    /// than inventing a client-facing status for a wiring mistake.
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Self>()
            .cloned()
            .ok_or_else(|| StatusCode::INTERNAL_SERVER_ERROR.into_response())
    }
}

/// Reject a request whose `Origin` and `Host` do not agree.
///
/// **External contract** — `request-security.mjs:74-80`: status 403, body
/// `{"error":"origin not allowed"}`, applied as the **first** middleware so it
/// covers `/livez`, `/readyz` and the WebSocket upgrade alike.
pub async fn origin_layer(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let headers = request.headers();
    let host = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok());
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok());
    if !is_allowed_origin(host, origin, &state.services().config.allowed_origins) {
        return (
            StatusCode::FORBIDDEN,
            axum::Json(json!({ "error": ORIGIN_NOT_ALLOWED })),
        )
            .into_response();
    }
    next.run(request).await
}

/// Resolve the owner, mint `X-Request-Id`, and set the identity cookie.
///
/// **External contract** — `gateway-application.mjs:186-194`. The cookie is
/// `via-core`'s (`identity.mjs:66-79`, rebranded per `docs/rebrand.md`); the
/// header and the log context are this layer's.
///
/// # Deviation: the log context is per-call, not ambient
///
/// Upstream wraps `next()` in `AsyncLocalStorage`, which follows the value
/// across every `await` in the request. `via_log::run_with_log_context` is a
/// thread-local and does not (recorded in `via-log`'s own deviations), so this
/// layer attaches `requestId` and `ownerId` to the request as an extension and
/// the handlers that log read them from there. Nothing is silently lost: a
/// handler that forgets is missing two fields, not correlating the wrong ones.
pub async fn identity_layer(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let secure = is_secure(&request);
    let cookie = request
        .headers()
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok());
    let resolved = state.services().identity.resolve_http(cookie, secure);
    let request_id = Uuid::new_v4().to_string();
    request.extensions_mut().insert(RequestIdentity {
        owner_id: resolved.identity().owner_id.clone(),
        request_id: request_id.clone(),
    });

    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static(REQUEST_ID_HEADER), value);
    }
    if let HttpIdentity::Issued { set_cookie, .. } = &resolved
        && let Ok(value) = HeaderValue::from_str(set_cookie)
    {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
    response
}

/// Log one line per completed request.
///
/// **External contract** — `gateway-application.mjs:195-211`:
/// `warn 'http.request_failed'` at status ≥ 500, otherwise
/// `debug 'http.request_completed'`, with `{method, path, status, durationMs}`
/// in that field order.
pub async fn access_log_layer(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_owned();
    let started_at = std::time::Instant::now();
    let response = next.run(request).await;
    let status = response.status().as_u16();
    let duration_ms = i64::try_from(started_at.elapsed().as_millis()).unwrap_or(i64::MAX);
    log_request(
        &state.services().logger,
        &method,
        &path,
        status,
        duration_ms,
    );
    response
}

fn log_request(logger: &Logger, method: &str, path: &str, status: u16, duration_ms: i64) {
    let entries = fields([
        ("method", method.into()),
        ("path", path.into()),
        ("status", status.into()),
        ("durationMs", duration_ms.into()),
    ]);
    if status >= 500 {
        logger.warn("http.request_failed", entries, "");
    } else {
        logger.debug("http.request_completed", entries, "");
    }
}

/// Whether the cookie must carry `Secure`.
///
/// **External contract** — `identity.mjs:66-79`:
/// `req.socket.encrypted || x-forwarded-proto === 'https'`. VIA terminates TLS
/// nowhere, so the first arm is always false and the proxy header is the whole
/// test.
fn is_secure(request: &Request<Body>) -> bool {
    request
        .headers()
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("https"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn request(headers: &[(&str, &str)]) -> Request<Body> {
        let mut builder = Request::builder().uri("/api/health");
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        builder.body(Body::empty()).expect("a request")
    }

    #[test]
    fn only_a_forwarded_https_proto_marks_the_cookie_secure() {
        assert!(!is_secure(&request(&[])));
        assert!(!is_secure(&request(&[("x-forwarded-proto", "http")])));
        assert!(is_secure(&request(&[("x-forwarded-proto", "https")])));
        assert!(
            is_secure(&request(&[("x-forwarded-proto", "HTTPS")])),
            "header values are compared case-insensitively"
        );
    }

    #[test]
    fn the_origin_rejection_body_is_the_catalogued_literal() {
        assert_eq!(
            serde_json::to_string(&json!({ "error": ORIGIN_NOT_ALLOWED })).expect("serializes"),
            "{\"error\":\"origin not allowed\"}",
        );
        assert_eq!(
            via_core::security::ORIGIN_REJECTION_BODY,
            "{\"error\":\"origin not allowed\"}",
        );
    }
}
