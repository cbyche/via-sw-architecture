//! `/livez`, `/readyz` and `/api/health`.
//!
//! Ported from `server/src/app/gateway-application.mjs:216-269`.
//!
//! All three sit **behind** the origin check, which the catalogue makes a point
//! of: *"Still passes through enforceSameOrigin + identity middleware, so a
//! disallowed Origin gets 403 instead of 200."* A liveness probe from a browser
//! on the wrong origin is refused like anything else.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::health::{HealthPayload, LivenessProbe, ReadinessProbe};
use crate::state::AppState;

/// `GET /livez`.
pub async fn livez() -> Json<LivenessProbe> {
    Json(LivenessProbe::default())
}

/// `GET /readyz`.
///
/// Reports ready as soon as the socket is listening. It deliberately does
/// **not** reflect backend readiness — that is `/api/health`'s `backend` block.
pub async fn readyz() -> Json<ReadinessProbe> {
    Json(ReadinessProbe::default())
}

/// `GET /api/health`.
///
/// The one failure mode is a configured default realtime provider that is not
/// registered; see [`AppState::health`]. Upstream throws there too, and the
/// throw reaches the terminal error handler.
pub async fn health(State(state): State<AppState>) -> Response {
    match state.health().await {
        Ok(payload) => Json::<HealthPayload>(payload).into_response(),
        Err(error) => {
            state.services().logger.error(
                "http.unhandled_error",
                via_log::fields([
                    ("method", "GET".into()),
                    ("path", "/api/health".into()),
                    ("error", error.to_string().into()),
                ]),
                "",
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
