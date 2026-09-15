//! The HTTP route table.
//!
//! Ported from `server/src/app/gateway-application.mjs:216-436`, route for
//! route and status for status. Every entry here has a catalogued
//! `exactValue` in `docs/reference/contracts.json`.
//!
//! | Route | Status |
//! | --- | --- |
//! | `GET /livez` | 200 `{"ok":true,"status":"live"}` |
//! | `GET /readyz` | 200 `{"ok":true,"status":"ready"}` |
//! | `GET /api/health` | 200, twenty-six keys in a fixed order |
//! | `POST /api/input/suspend` | 200 status · 400 `VIA_INPUT_OWNER_REQUIRED` |
//! | `POST /api/input/resume` | 200 status |
//! | `GET /api/input` | 200 status |
//! | `GET /api/backend/ui` | 302 · 404 |
//! | `GET /api/tasks` | 200 `{tasks:[…]}` |
//! | `GET /api/timeline` | 200 `{items:[…]}` |
//! | `GET /api/tasks/{id}` | 200 · 404 |
//! | `DELETE /api/tasks/{id}` | 200 · 404 · 409 |
//! | `GET /api/tasks/{id}/events` | 200 `text/event-stream` · 404 |
//! | `POST /api/permissions/{id}` | 200 · 400 · 404 |
//! | `GET /api/realtime` | 101 · 403 · 401 · *destroyed* |
//!
//! # What is deliberately absent
//!
//! Upstream ends its table with three static-hosting routes
//! (`gateway-application.mjs:423-436`): `/skins/*`, `express.static(web/dist)`
//! and a `GET *` fallback that serves `index.html`. **VIA ships no web UI**, so
//! all three are gone and the two capabilities that advertise them —
//! `web.same-origin-ui` and `web.skin-assets` — are in
//! [`via_protocol::DROPPED_UPSTREAM_CAPABILITIES`] rather than in the
//! advertised list.
//!
//! That changes one observable behaviour, and [`FALLBACK_IS_JSON_404`] is where
//! it is recorded: upstream's catch-all *swallows every unmatched GET including
//! unknown `/api` paths, which therefore return `index.html` rather than a JSON
//! 404*. With nothing to serve, an unmatched path is a JSON 404 here. Pretending
//! otherwise would mean serving a 200 with no body, which is worse than an
//! honest miss.

pub mod backend;
pub mod coerce;
pub mod input;
pub mod middleware;
pub mod permissions;
pub mod probes;
pub mod tasks;

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;
use via_i18n::{keys, t};

use crate::state::AppState;

/// The JSON body limit.
///
/// **External contract** — `gateway-application.mjs:212`:
/// `express.json({limit: '1mb'})`. It is applied **innermost**, so an oversized
/// body is refused only after the origin check and identity issuance have run.
pub const JSON_BODY_LIMIT: usize = 1024 * 1024;

/// Whether an unmatched path answers with a JSON 404.
///
/// `true` in VIA, `false` upstream — see the module documentation. Named as a
/// constant so `via-conformance` can point a Partial row at it rather than at
/// prose.
pub const FALLBACK_IS_JSON_404: bool = true;

/// The `GET /api/realtime` path.
///
/// **External contract** — `server/src/voice/realtime-gateway.mjs:71`, and
/// `via_voice::REALTIME_ROUTE` is the constant. Restated here only so the route
/// table reads in one place.
pub const REALTIME_ROUTE: &str = via_voice::REALTIME_ROUTE;

/// Assemble the router.
///
/// Two routers, merged. The HTTP half carries the middleware stack in
/// upstream's declaration order, outermost first, which is what
/// [`tower::ServiceBuilder`] gives: origin → identity → access log → body
/// limit (see [`middleware`] for why that order is load-bearing).
///
/// **The realtime route carries none of it**, and that is upstream's shape
/// rather than an omission: `attachRealtimeGateway` hangs off
/// `server.on('upgrade')`, not off Express, so no Express middleware ever runs
/// for an upgrade. Reproducing that matters because the two refusals differ —
/// an HTTP route answers a disallowed origin with
/// `{"error":"origin not allowed"}` in JSON, and an upgrade answers with the
/// same sentence in **plain text** after a raw status line
/// (`realtime-gateway.mjs`'s `rejectUpgrade`). The realtime handler therefore
/// runs the origin check and the identity resolution itself; see
/// [`crate::realtime`].
///
/// The identity difference is the sharper one: the HTTP layer **issues** a
/// cookie when a request has none, while the upgrade path only ever
/// **resolves** one and answers 401 otherwise. Sharing a layer would silently
/// mint an owner for every unauthenticated socket.
pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/livez", get(probes::livez))
        .route("/readyz", get(probes::readyz))
        .route("/api/health", get(probes::health))
        .route("/api/input/suspend", post(input::suspend))
        .route("/api/input/resume", post(input::resume))
        .route("/api/input", get(input::status))
        .route("/api/backend/ui", get(backend::ui))
        .route("/api/tasks", get(tasks::list))
        .route("/api/timeline", get(tasks::timeline))
        .route("/api/tasks/{id}", get(tasks::get).delete(tasks::cancel))
        .route("/api/tasks/{id}/events", get(tasks::events))
        .route("/api/permissions/{id}", post(permissions::respond))
        .fallback(fallback)
        .layer(
            tower::ServiceBuilder::new()
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    middleware::origin_layer,
                ))
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    middleware::identity_layer,
                ))
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    middleware::access_log_layer,
                ))
                .layer(DefaultBodyLimit::max(JSON_BODY_LIMIT)),
        );

    Router::new()
        .route(REALTIME_ROUTE, get(crate::realtime::upgrade))
        .merge(api)
        .with_state(state)
}

/// The body every unmatched path answers with.
///
/// **External contract, reused rather than invented** —
/// `gateway-application.mjs:434` gives `{"error":"not found"}` for a
/// `/skins/*` miss. That is the only JSON 404 upstream's static tail produces,
/// so VIA's fallback answers with the same body rather than minting a new one.
/// See [`FALLBACK_IS_JSON_404`] for why there is a fallback at all.
pub const NOT_FOUND_BODY: &str = "not found";

/// Every unmatched path.
async fn fallback() -> (axum::http::StatusCode, Json<serde_json::Value>) {
    (
        axum::http::StatusCode::NOT_FOUND,
        Json(json!({ "error": NOT_FOUND_BODY })),
    )
}

/// The Chinese-prose 404 body a missing backend web address answers with.
///
/// Re-exported so `via-conformance` can assert the catalogued string without
/// reaching into a private module.
#[must_use]
pub fn backend_ui_missing_message(locale: via_i18n::Locale) -> &'static str {
    t(locale, keys::GATEWAY_BACKEND_HAS_NO_WEB_UI)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn the_body_limit_is_one_mebibyte() {
        assert_eq!(JSON_BODY_LIMIT, 1_048_576);
    }

    #[test]
    fn the_realtime_route_is_the_one_via_voice_publishes() {
        assert_eq!(REALTIME_ROUTE, "/api/realtime");
    }

    #[test]
    fn the_backend_ui_message_is_upstreams_chinese_prose() {
        assert_eq!(
            backend_ui_missing_message(via_i18n::Locale::Zh),
            "当前后台 Agent 没有独立的 Web 地址",
        );
    }
}
