//! `POST /api/permissions/{id}` — the approval UI's one route.
//!
//! Ported from `server/src/app/gateway-application.mjs:365-404`.
//!
//! **The ordering is the contract**, and the catalogue says so in as many
//! words: *"the local permission policy is mutated BEFORE the backend call and
//! rolled back on error."* Three steps, in this order:
//!
//! 1. find the active Work whose `authorization.id` matches, and read the
//!    session's current permission mode;
//! 2. apply the decision to the **local** policy, so a second request in the
//!    same session sees it even while the relay is still in flight;
//! 3. relay to the harness — and on failure put the previous mode back.
//!
//! Skipping step 3's rollback would leave a session permanently auto-approving
//! after a relay that never landed, which is the failure this ordering exists
//! to prevent.
//!
//! The decision enum is exactly `['always','reject']` — no `once`, no `allow`.
//! It is the same pair as the voice frontend tool's schema
//! (`server/src/voice/frontend-tools.mjs:172`), which is why it lives in
//! `via-voice` and is imported rather than restated.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use via_i18n::{keys, t};
use via_voice::{PermissionDecision, PermissionMode};
use via_work::WorkQuery;

use crate::backend::PermissionRelayError;
use crate::http::middleware::RequestIdentity;
use crate::state::AppState;

/// `POST /api/permissions/{id}`.
pub async fn respond(
    State(state): State<AppState>,
    identity: RequestIdentity,
    Path(id): Path<String>,
    body: Option<Json<Value>>,
) -> Response {
    // `String(req.body?.decision || '')` — a wrongly-typed field must reach the
    // catalogued 400, not a deserialization failure. See [`crate::http::coerce`].
    let raw = crate::http::coerce::js_string(
        body.as_ref()
            .and_then(|Json(body)| body.as_object())
            .and_then(|body| body.get("decision")),
    );
    // `String(req.body?.decision || '')` then a membership test — an absent
    // body, a null decision and a typo are all the same 400.
    let Some(decision) = PermissionDecision::from_wire(&raw) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": t(state.locale(), keys::GATEWAY_PERMISSION_DECISION_INVALID),
            })),
        )
            .into_response();
    };

    let services = state.services();
    let permission_work = services
        .work
        .list(WorkQuery::owner(&identity.owner_id).active())
        .await
        .into_iter()
        .find(|task| {
            task.authorization
                .as_ref()
                .is_some_and(|authorization| authorization.id == id)
        });

    let previous_mode: Option<(String, PermissionMode)> = match &permission_work {
        Some(task) => {
            let mut policy = services.permission_policy.lock().await;
            let mode = policy.mode(&identity.owner_id, &task.session_id);
            policy.apply_decision(&identity.owner_id, &task.session_id, decision);
            Some((task.session_id.clone(), mode))
        }
        None => None,
    };

    match services
        .backend
        .respond_permission(&id, decision, &identity.owner_id)
        .await
    {
        Ok(permission) => Json(permission).into_response(),
        Err(error) => {
            if let Some((session_id, mode)) = previous_mode {
                services.permission_policy.lock().await.set_mode(
                    &identity.owner_id,
                    &session_id,
                    mode,
                );
            }
            match error {
                PermissionRelayError::NotFound(message) => {
                    (StatusCode::NOT_FOUND, Json(json!({ "error": message }))).into_response()
                }
                // Upstream's `next(error)` — the terminal handler logs and
                // Express answers 500 with no JSON envelope. VIA logs the same
                // event and answers a bare 500 for the same reason: a body
                // here would be a new contract nobody asked for.
                PermissionRelayError::Failed(message) => {
                    services.logger.error(
                        "http.unhandled_error",
                        via_log::fields([
                            ("method", "POST".into()),
                            ("path", "/api/permissions/:id".into()),
                            ("error", message.into()),
                        ]),
                        "",
                    );
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn the_decision_enum_is_exactly_always_and_reject() {
        assert_eq!(
            PermissionDecision::from_wire("always"),
            Some(PermissionDecision::Always),
        );
        assert_eq!(
            PermissionDecision::from_wire("reject"),
            Some(PermissionDecision::Reject),
        );
        for refused in ["once", "allow", "deny", "", "ALWAYS"] {
            assert_eq!(
                PermissionDecision::from_wire(refused),
                None,
                "`{refused}` is not one of the two literals",
            );
        }
    }

    #[test]
    fn an_absent_body_and_a_wrongly_typed_one_take_the_same_branch() {
        let read = |body: &str| {
            let parsed: Value = serde_json::from_str(body).expect("a body");
            crate::http::coerce::js_string(parsed.as_object().and_then(|body| body.get("decision")))
        };
        for body in [
            "{}",
            "{\"decision\":null}",
            "{\"decision\":true}",
            "{\"decision\":5}",
        ] {
            assert_eq!(
                PermissionDecision::from_wire(&read(body)),
                None,
                "body {body} must reach the catalogued 400",
            );
        }
        assert_eq!(
            PermissionDecision::from_wire(&read("{\"decision\":\"always\"}")),
            Some(PermissionDecision::Always),
        );
    }
}
