//! `GET /api/backend/ui` — a 302 to the harness's own web address.
//!
//! Ported from `server/src/app/gateway-application.mjs:298-313`.
//!
//! **External contract** — 302 when the harness declares `backendUi` **and**
//! resolves a URL; otherwise 404 with the same body in **both** branches, which
//! the catalogue calls out explicitly (*"identical body in both 404
//! branches"*). A harness that declares the capability but hands back nothing
//! is not distinguishable from one that never had it, and upstream keeps it
//! that way rather than leaking which of the two happened.
//!
//! The message is user-visible Chinese prose upstream
//! (`'当前后台 Agent 没有独立的 Web 地址'`), so `via-i18n` owns it.

use axum::Json;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::json;
use via_i18n::{keys, t};

use crate::backend::PermissionRelayError;
use crate::http::middleware::RequestIdentity;
use crate::state::AppState;

/// `GET /api/backend/ui`.
pub async fn ui(State(state): State<AppState>, identity: RequestIdentity) -> Response {
    let services = state.services();
    let no_web_address = || {
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": t(state.locale(), keys::GATEWAY_BACKEND_HAS_NO_WEB_UI),
            })),
        )
            .into_response()
    };

    if !services.backend.describe().backend_ui() {
        return no_web_address();
    }
    match services.backend.ui_url(&identity.owner_id).await {
        Ok(Some(url)) => found(&url),
        Ok(None) => no_web_address(),
        // Upstream's `next(error)` arm.
        Err(error) => {
            let detail = match error {
                PermissionRelayError::NotFound(message) | PermissionRelayError::Failed(message) => {
                    message
                }
            };
            services.logger.error(
                "http.unhandled_error",
                via_log::fields([
                    ("method", "GET".into()),
                    ("path", "/api/backend/ui".into()),
                    ("error", detail.into()),
                ]),
                "",
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// The status a successful `GET /api/backend/ui` answers with.
///
/// **External contract** — `gateway-application.mjs:309`: `res.redirect(302,
/// url)`. Express's two-argument `redirect` sends **302 Found**, not 307, so
/// [`axum::response::Redirect::temporary`] (which is 307) would be the wrong
/// verb. This
/// constant is what the router asserts against.
pub const BACKEND_UI_REDIRECT_STATUS: u16 = 302;

/// A 302 to `url`, spelled without `Redirect`'s 307 default.
#[must_use]
pub fn found(url: &str) -> Response {
    match axum::http::HeaderValue::from_str(url) {
        Ok(value) => (StatusCode::FOUND, [(header::LOCATION, value)]).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn the_redirect_is_302_not_307() {
        let response = found("http://127.0.0.1:4096/ui");
        assert_eq!(response.status().as_u16(), BACKEND_UI_REDIRECT_STATUS);
        assert_eq!(
            response
                .headers()
                .get(header::LOCATION)
                .and_then(|value| value.to_str().ok()),
            Some("http://127.0.0.1:4096/ui"),
        );
        assert_ne!(
            axum::response::Redirect::temporary("http://127.0.0.1:4096/ui")
                .into_response()
                .status()
                .as_u16(),
            BACKEND_UI_REDIRECT_STATUS,
            "axum's Redirect::temporary is 307; upstream sends 302",
        );
    }

    #[test]
    fn a_url_that_cannot_be_a_header_does_not_panic() {
        assert_eq!(
            found("http://example\u{0}/ui").status(),
            StatusCode::INTERNAL_SERVER_ERROR,
        );
    }
}
