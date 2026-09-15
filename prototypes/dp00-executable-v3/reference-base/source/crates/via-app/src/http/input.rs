//! The host's microphone control plane.
//!
//! Ported from `server/src/app/gateway-application.mjs:275-296`.
//!
//! An external host — an input method, a platform app, a screen recorder —
//! announces that it is taking the microphone, and the Gateway *commands* its
//! clients to stop capturing. Upstream's comment is the design in one sentence:
//! both calls are idempotent per owner, and a suspension expires on its own so
//! **a host that crashes cannot silence the Gateway for good**.
//!
//! `via-voice` owns the state machine ([`via_voice::InputArbitration`]) and the
//! numbers; this module is the three routes in front of it. The relay onto the
//! socket — `input.suspend` / `input.resume` and the `playback.clear` that
//! accompanies a suspension — is [`crate::realtime`]'s.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use via_voice::SuspensionStatus;

use crate::state::AppState;

/// One field of a request body, coerced the way upstream reads it.
///
/// **External contract** — `docs/reference/contracts.json`, `http-route` /
/// *POST /api/input/suspend*: `{owner:string(required), reason?:string,
/// ttlMs?:number}`. The bounds — owner to 80 characters, reason to 200, ttl
/// defaulting to 15 000 ms and capped at 300 000 — are
/// [`via_voice::arbitration`]'s, not this module's; the coercions
/// (`String(owner || '')`, `Number(ttlMs)`) are
/// [`crate::http::coerce`]'s.
fn field<'a>(body: Option<&'a Json<Value>>, name: &str) -> Option<&'a Value> {
    body.and_then(|Json(body)| body.as_object())
        .and_then(|body| body.get(name))
}

/// `POST /api/input/suspend`.
///
/// **External contract** — 200 with the arbitration status; 400 with
/// `{"error":…,"code":"VIA_INPUT_OWNER_REQUIRED"}` when `owner` is blank.
/// Upstream's code is `QWAUDIO_INPUT_OWNER_REQUIRED`; `docs/rebrand.md` renames
/// the `QWAUDIO_` namespace, and [`via_protocol::CODE_INPUT_OWNER_REQUIRED`]
/// is the renamed constant.
pub async fn suspend(State(state): State<AppState>, body: Option<Json<Value>>) -> Response {
    let owner = crate::http::coerce::js_string(field(body.as_ref(), "owner"));
    let reason = crate::http::coerce::js_string(field(body.as_ref(), "reason"));
    // `ttlFor(ttlMs)` — a non-finite or non-positive value falls back to the
    // default (`input-arbitration.mjs:61-65`). An absent one is spelled `None`
    // so the two paths cannot diverge here.
    let ttl_ms = crate::http::coerce::js_number(field(body.as_ref(), "ttlMs"))
        .filter(|value| value.is_finite() && *value > 0.0)
        .map(|value| value as u64);

    match state
        .services()
        .input_arbitration
        .suspend(&owner, &reason, ttl_ms)
        .await
    {
        Ok(status) => Json(status).into_response(),
        Err(refusal) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": refusal.message,
                "code": refusal.code(),
            })),
        )
            .into_response(),
    }
}

/// `POST /api/input/resume`.
///
/// **External contract** — *"Resuming an unknown owner is a no-op that still
/// returns 200 with current status."*
pub async fn resume(
    State(state): State<AppState>,
    body: Option<Json<Value>>,
) -> Json<SuspensionStatus> {
    let owner = crate::http::coerce::js_string(field(body.as_ref(), "owner"));
    Json(state.services().input_arbitration.resume(&owner).await)
}

/// `GET /api/input`.
pub async fn status(State(state): State<AppState>) -> Json<SuspensionStatus> {
    Json(state.services().input_arbitration.status().await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn ttl(body: &str) -> Option<u64> {
        let parsed: Value = serde_json::from_str(body).expect("a suspend body");
        let wrapped = Json(parsed);
        crate::http::coerce::js_number(field(Some(&wrapped), "ttlMs"))
            .filter(|value| value.is_finite() && *value > 0.0)
            .map(|value| value as u64)
    }

    #[test]
    fn an_absent_or_unusable_ttl_falls_back_rather_than_failing() {
        assert_eq!(ttl("{\"owner\":\"host\"}"), None);
        assert_eq!(ttl("{\"owner\":\"host\",\"ttlMs\":0}"), None);
        assert_eq!(ttl("{\"owner\":\"host\",\"ttlMs\":-1}"), None);
        assert_eq!(ttl("{\"owner\":\"host\",\"ttlMs\":null}"), None);
        assert_eq!(ttl("{\"owner\":\"host\",\"ttlMs\":\"nope\"}"), None);
        assert_eq!(ttl("{\"owner\":\"host\",\"ttlMs\":60000}"), Some(60_000));
        assert_eq!(
            ttl("{\"owner\":\"host\",\"ttlMs\":\"60000\"}"),
            Some(60_000),
            "Number('60000') is 60000",
        );
    }

    #[test]
    fn the_owner_required_code_is_the_rebranded_one() {
        assert_eq!(
            via_protocol::CODE_INPUT_OWNER_REQUIRED,
            "VIA_INPUT_OWNER_REQUIRED",
        );
    }
}
