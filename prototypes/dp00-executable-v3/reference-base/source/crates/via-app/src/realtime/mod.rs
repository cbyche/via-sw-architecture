//! `WS /api/realtime` — the one voice endpoint.
//!
//! Ported from `server/src/voice/realtime-gateway.mjs`'s transport half; the
//! decisions it makes are `via-voice`'s and are called rather than re-derived.
//!
//! # The upgrade contract, in order
//!
//! [`via_voice::upgrade_decision`] is the rule and the **order is the
//! contract** — *"an unknown route must not leak whether an origin would have
//! been allowed"*:
//!
//! | | Answer |
//! | --- | --- |
//! | path is not `/api/realtime` | **the socket is destroyed** — no HTTP response at all |
//! | origin disallowed | `HTTP/1.1 403 Forbidden` · `text/plain` · `origin not allowed` |
//! | no resolvable identity | `HTTP/1.1 401 Unauthorized` · `text/plain` · `identity required` |
//! | otherwise | 101, `maxPayload` [`connection::MAX_PAYLOAD_BYTES`] |
//!
//! The first row is why `via-app` owns its accept loop instead of calling
//! `axum::serve`: a handler can only *return a response*, and the contract is
//! that there is none. [`crate::serve`] holds that half.
//!
//! The other two are answered here rather than by
//! [`crate::http::middleware::origin_layer`], because the upgrade's refusals
//! are **plain text**, not the JSON body an HTTP route gets. The route is
//! therefore merged into the router without the HTTP middleware stack, exactly
//! as upstream attaches it to `server.on('upgrade')` rather than to Express.
//!
//! # Identity is resolved, never issued
//!
//! `identityManager.resolveUpgrade(request)` reads the cookie and answers
//! `null` when there is not a valid one; it does **not** mint. A browser that
//! has never spoken HTTP to this Gateway therefore gets 401 and goes to fetch a
//! cookie first, which is what stops an unauthenticated socket from silently
//! becoming a new owner.

pub mod clients;
pub mod connection;
pub mod engine;
pub mod frames;

use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use via_core::security::is_allowed_origin;
use via_voice::{UpgradeDecision, upgrade_decision};

pub use clients::{Connection, SlotClaim, VoiceClientRegistry, deactivation_frames};
pub use connection::{DEFAULT_SESSION_ID, MAX_PAYLOAD_BYTES, forwards_to_client};
pub use engine::{
    EngineContext, EngineFactory, NoEngineFactory, NoModelEngine, SharedGate, VoiceEngine,
};
pub use frames::{ClientDescriptor, ClientFrame, ServerFrame};

use crate::state::AppState;

/// `/api/realtime`'s query string.
#[derive(Debug, Clone, Deserialize)]
pub struct RealtimeQuery {
    /// Which conversation this socket belongs to. Defaults to `main`.
    #[serde(rename = "sessionId", default = "default_session_id")]
    pub session_id: String,
}

fn default_session_id() -> String {
    DEFAULT_SESSION_ID.to_owned()
}

impl Default for RealtimeQuery {
    fn default() -> Self {
        Self {
            session_id: default_session_id(),
        }
    }
}

/// `GET /api/realtime`.
pub async fn upgrade(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<RealtimeQuery>,
    ws: WebSocketUpgrade,
) -> Response {
    let host = header_str(&headers, header::HOST);
    let origin = header_str(&headers, header::ORIGIN);
    let origin_allowed = is_allowed_origin(host, origin, &state.services().config.allowed_origins);
    let identity = state
        .services()
        .identity
        .resolve_upgrade(header_str(&headers, header::COOKIE));

    // The route matched, or this handler would not be running.
    match upgrade_decision(true, origin_allowed, identity.is_some()) {
        UpgradeDecision::Accept => {}
        UpgradeDecision::Reject { status, message } => return reject(status, message),
        // Unreachable through the router: an unmatched path never gets here,
        // and `crate::serve` destroys its socket before hyper builds a request.
        UpgradeDecision::Destroy => return StatusCode::NOT_FOUND.into_response(),
    }

    let Some(identity) = identity else {
        return reject("401 Unauthorized", "identity required");
    };
    let owner_id = identity.owner_id;
    let session_id = query.session_id;

    ws.max_message_size(MAX_PAYLOAD_BYTES)
        .on_upgrade(move |socket| connection::run(socket, state, owner_id, session_id))
}

/// The plain-text refusal an upgrade gets.
///
/// **External contract** — `realtime-gateway.mjs`'s `rejectUpgrade`:
/// `HTTP/1.1 <status>\r\nConnection: close\r\nContent-Type: text/plain\r\n\r\n<message>`.
///
/// # Deviation: hyper writes the status line
///
/// Upstream writes those bytes onto the raw socket itself, so the response has
/// exactly three header lines. Here the response is built normally and hyper
/// adds `content-length` and `date`. The status, the `connection: close`, the
/// content type and the body — everything a client parses — are identical;
/// the two extra headers are what any conforming HTTP/1.1 response carries.
fn reject(status: &str, message: &str) -> Response {
    let code = status
        .split_whitespace()
        .next()
        .and_then(|code| code.parse::<u16>().ok())
        .and_then(|code| StatusCode::from_u16(code).ok())
        .unwrap_or(StatusCode::FORBIDDEN);
    (
        code,
        [
            (header::CONNECTION, "close"),
            (header::CONTENT_TYPE, "text/plain"),
        ],
        message.to_owned(),
    )
        .into_response()
}

fn header_str(headers: &HeaderMap, name: header::HeaderName) -> Option<&str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn the_two_refusals_are_the_catalogued_status_lines() {
        let forbidden = reject("403 Forbidden", "origin not allowed");
        assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
        assert_eq!(
            forbidden
                .headers()
                .get(header::CONNECTION)
                .and_then(|value| value.to_str().ok()),
            Some("close"),
        );
        assert_eq!(
            forbidden
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("text/plain"),
        );
        assert_eq!(
            reject("401 Unauthorized", "identity required").status(),
            StatusCode::UNAUTHORIZED,
        );
    }

    #[test]
    fn an_unparseable_status_line_still_refuses() {
        assert_eq!(reject("nonsense", "x").status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn the_session_id_defaults_to_main() {
        assert_eq!(RealtimeQuery::default().session_id, DEFAULT_SESSION_ID);
        assert_eq!(DEFAULT_SESSION_ID, "main");
        let named: RealtimeQuery =
            serde_json::from_str("{\"sessionId\":\"scratch\"}").expect("a query");
        assert_eq!(named.session_id, "scratch");
    }

    #[test]
    fn the_upgrade_order_is_route_then_origin_then_identity() {
        assert_eq!(
            upgrade_decision(false, false, false),
            UpgradeDecision::Destroy
        );
        assert!(matches!(
            upgrade_decision(true, false, true),
            UpgradeDecision::Reject { message, .. } if message == "origin not allowed"
        ));
        assert!(matches!(
            upgrade_decision(true, true, false),
            UpgradeDecision::Reject { message, .. } if message == "identity required"
        ));
        assert_eq!(upgrade_decision(true, true, true), UpgradeDecision::Accept);
    }
}
