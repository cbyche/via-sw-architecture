//! Which generation of the Realtime **session schema** an endpoint speaks.
//!
//! Ported from ARGO `tinicore/src/llm/providers/openai_live.rs:585-700`
//! (`RealtimeProtocol`, `protocol_for_url`, `protocol_from_route`,
//! `PREVIEW_ENDPOINTS`, `remember_preview_endpoint`,
//! `rejects_ga_discriminator`).
//!
//! # Why the type is called `RealtimeSchema` here
//!
//! ARGO names it `RealtimeProtocol`. In VIA that name is already taken by
//! [`via_realtime::RealtimeProtocol`], the *wire dialect* trait — and the two
//! are genuinely different questions. The dialect decides the envelope, the
//! event names and the id namespaces; the schema decides the shape of the one
//! object inside `session.update`. Renamed, and the `Debug` spelling of each
//! variant is unchanged so ARGO's `[live] schema=Ga` log marker still reads the
//! same.
//!
//! # The lesson this file is
//!
//! Classifying by route is a guess, and on a gateway it is a wrong one: the
//! litellm gateway ARGO was brought up against serves GA's own path
//! (`/v1/realtime`) while speaking the pre-GA schema, and answers the GA
//! discriminator with `Unknown parameter: 'session.type'.` The route cannot
//! know that. The endpoint can, and it says so on the first `session.update`.
//!
//! So the answer is remembered **per host, for the process lifetime**. Without
//! the memo the repair happens once per session, which lands a reconfiguration
//! in the middle of the user's first utterance — because that is exactly when a
//! voice user starts talking.

use std::collections::HashSet;
use std::sync::{LazyLock, Mutex, PoisonError};

use serde_json::Value;

/// Which generation of the `session.update` schema a route speaks.
///
/// This is not a preference — the two are different wire schemas selected by
/// the endpoint, and a session configured in the wrong one is rejected field by
/// field. Every knob except `instructions` and `tools` moved at GA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RealtimeSchema {
    /// The GA schema. `session.type` is a required discriminator, modalities
    /// are `output_modalities`, and the audio knobs live under `session.audio`.
    Ga,
    /// The pre-GA schema — flat `modalities` / `voice` on `session`, no
    /// discriminator.
    ///
    /// OpenAI **shut this interface down on 2026-05-12**, so it is reachable
    /// only through a gateway or an Azure resource still serving the preview
    /// route. Kept because those routes are still in the candidate walk; not
    /// something new code should target.
    Preview,
}

impl RealtimeSchema {
    /// The word ARGO's `[live] schema=` marker prints.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ga => "Ga",
            Self::Preview => "Preview",
        }
    }
}

impl core::fmt::Display for RealtimeSchema {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Classify a candidate URL by the schema its endpoint speaks.
///
/// **What this endpoint answered last time beats what its address suggests.**
/// [`protocol_from_route`] is the guess; [`remember_preview_endpoint`] is the
/// answer, and the answer wins.
#[must_use]
pub fn protocol_for_url(url: &str) -> RealtimeSchema {
    if remembered_protocol(url) == Some(RealtimeSchema::Preview) {
        return RealtimeSchema::Preview;
    }
    protocol_from_route(url)
}

/// Route-shape classification, used until an endpoint has told us otherwise.
///
/// The path is the whole signal: Azure's preview surface is `/openai/realtime`
/// (with `deployment=` + a dated `api-version`), and everything else this crate
/// dials — OpenAI's `/v1/realtime` and Azure's OpenAI-compatible
/// `/openai/v1/realtime` — is GA. Checked in that order because
/// `/openai/v1/realtime` contains both prefixes.
#[must_use]
pub fn protocol_from_route(url: &str) -> RealtimeSchema {
    let path = url.split_once("://").map_or(url, |(_, rest)| rest);
    let path = path.split_once('/').map_or("", |(_, rest)| rest);
    if path.starts_with("openai/realtime") {
        RealtimeSchema::Preview
    } else {
        RealtimeSchema::Ga
    }
}

/// Endpoints observed to speak the pre-GA schema, keyed by host.
///
/// Host rather than full URL: the schema is a property of the deployment, not
/// of which model or `api-version` a particular session asked for.
/// Process-lifetime only — an endpoint that is upgraded under a running Gateway
/// is a restart away from being re-learned, which is the right trade for a cache
/// whose only alternative is a wrong guess every time.
static PREVIEW_ENDPOINTS: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

/// The host (with port, when one is given) a URL addresses, lowercased.
///
/// This is the memo's key. Keying on the whole URL would relearn the schema for
/// every model and every `api-version` the same deployment is asked for.
#[must_use]
pub fn endpoint_key(url: &str) -> String {
    let no_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    no_scheme
        .split(['/', '?'])
        .next()
        .unwrap_or(no_scheme)
        .to_ascii_lowercase()
}

fn remembered_protocol(url: &str) -> Option<RealtimeSchema> {
    PREVIEW_ENDPOINTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .contains(&endpoint_key(url))
        .then_some(RealtimeSchema::Preview)
}

/// Record that an endpoint refused the GA discriminator.
///
/// Called from the repair path, so the knowledge is a *consequence* of the
/// repair rather than a second thing to keep in sync with it.
pub fn remember_preview_endpoint(url: &str) {
    PREVIEW_ENDPOINTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(endpoint_key(url));
}

/// Whether a server `error` frame is the endpoint saying it does not know the
/// GA session discriminator — i.e. it speaks the pre-GA schema.
///
/// The route cannot answer this once a relay is in the path. The gateway ARGO
/// was brought up against serves `/v1/realtime`, GA's own path, and forwards to
/// something that rejects `session.type` with `Unknown parameter:
/// 'session.type'.` — so a URL that looks GA reaches a backend that is not.
/// Asking the endpoint is the only reliable way, and it answers on the first
/// `session.update`.
///
/// Matched on the parameter name rather than the sentence around it: the
/// wording differs across OpenAI, Azure and the relays in between, but
/// `session.type` in an error is only ever this. Deliberately one-directional —
/// the reverse (a GA endpoint refusing the pre-GA shape) has not been observed
/// and would need its own evidence.
///
/// Both nestings are accepted: OpenAI wraps the detail in `error`, some relays
/// flatten it onto the frame.
#[must_use]
pub fn rejects_ga_discriminator(event: &Value) -> bool {
    if event.get("type").and_then(Value::as_str) != Some("error") {
        return false;
    }
    let message = event
        .get("error")
        .and_then(|error| error.get("message"))
        .or_else(|| event.get("message"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    message.contains("session.type")
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn the_schema_prints_the_word_argos_marker_prints() {
        assert_eq!(RealtimeSchema::Ga.to_string(), "Ga");
        assert_eq!(RealtimeSchema::Preview.to_string(), "Preview");
        assert_eq!(format!("{:?}", RealtimeSchema::Ga), "Ga");
        assert_eq!(format!("{:?}", RealtimeSchema::Preview), "Preview");
    }

    #[test]
    fn the_schema_follows_the_route_not_the_dialect() {
        // `/openai/v1/realtime` contains `/v1/realtime` AND starts with
        // `openai/`, so classification order is load-bearing.
        for ga in [
            "wss://api.openai.com/v1/realtime?model=gpt-realtime-2.1",
            "wss://res.openai.azure.com/openai/v1/realtime?model=d",
            "wss://gateway.internal/v1/realtime?model=m",
        ] {
            assert_eq!(protocol_from_route(ga), RealtimeSchema::Ga, "{ga}");
        }
        for preview in [
            "wss://res.openai.azure.com/openai/realtime?api-version=2025-08-28&deployment=d",
            "wss://res.openai.azure.com/openai/realtime?api-version=2024-10-01-preview&deployment=d",
        ] {
            assert_eq!(
                protocol_from_route(preview),
                RealtimeSchema::Preview,
                "{preview}"
            );
        }
    }

    #[test]
    fn a_url_with_no_path_at_all_reads_as_ga() {
        // `split_once('/')` finds nothing, so the path is empty. GA is the right
        // answer: a bare host is not Azure's preview surface.
        assert_eq!(
            protocol_from_route("wss://host.example"),
            RealtimeSchema::Ga
        );
        assert_eq!(protocol_from_route("host.example"), RealtimeSchema::Ga);
    }

    #[test]
    fn the_memo_key_is_the_host_alone() {
        for url in [
            "wss://gw.example/v1/realtime?model=a",
            "wss://GW.EXAMPLE/openai/realtime?api-version=x&deployment=b",
            "https://gw.example",
            "gw.example/v1/realtime",
        ] {
            assert_eq!(endpoint_key(url), "gw.example", "{url}");
        }
    }

    #[test]
    fn the_memo_key_keeps_the_port_so_two_local_endpoints_do_not_share_a_verdict() {
        assert_eq!(
            endpoint_key("ws://127.0.0.1:8765/v1/realtime"),
            "127.0.0.1:8765"
        );
        assert_ne!(
            endpoint_key("ws://127.0.0.1:8765/v1/realtime"),
            endpoint_key("ws://127.0.0.1:8766/v1/realtime")
        );
    }

    #[test]
    fn a_remembered_endpoint_overrides_the_route_guess() {
        // The route says GA; the endpoint said otherwise once. What it said
        // wins, which is the whole point — the guess is what put a
        // reconfiguration in the middle of a user's first sentence.
        let url = "wss://schema-memo-unit.invalid/v1/realtime?model=m";
        assert_eq!(protocol_from_route(url), RealtimeSchema::Ga);
        assert_eq!(protocol_for_url(url), RealtimeSchema::Ga);

        remember_preview_endpoint(url);
        assert_eq!(
            protocol_for_url(url),
            RealtimeSchema::Preview,
            "the observation must beat the guess"
        );
        // And for any other route on the same host, including a different model
        // and a different api-version.
        assert_eq!(
            protocol_for_url("wss://schema-memo-unit.invalid/openai/v1/realtime?model=z"),
            RealtimeSchema::Preview
        );
        // The route guess itself is untouched — it is the guess, not the answer.
        assert_eq!(protocol_from_route(url), RealtimeSchema::Ga);
    }

    #[test]
    fn an_unknown_endpoint_still_follows_its_route() {
        assert_eq!(
            protocol_for_url("wss://schema-unseen-a.invalid/openai/realtime?api-version=x"),
            RealtimeSchema::Preview
        );
        assert_eq!(
            protocol_for_url("wss://schema-unseen-b.invalid/v1/realtime?model=m"),
            RealtimeSchema::Ga
        );
    }

    #[test]
    fn the_memo_does_not_leak_between_hosts() {
        remember_preview_endpoint("wss://schema-leak-one.invalid/v1/realtime");
        assert_eq!(
            protocol_for_url("wss://schema-leak-two.invalid/v1/realtime"),
            RealtimeSchema::Ga
        );
    }

    #[test]
    fn the_ga_discriminator_rejection_is_recognised_in_both_nestings() {
        // The exact frame observed from the gateway. Recognising it is what
        // turns a dead session into a resend in the schema it does speak.
        assert!(rejects_ga_discriminator(&json!({
            "type": "error",
            "error": { "message": "Unknown parameter: 'session.type'." }
        })));
        // Some relays flatten the detail onto the frame.
        assert!(rejects_ga_discriminator(&json!({
            "type": "error",
            "message": "Unknown parameter: 'session.type'."
        })));
    }

    #[test]
    fn everything_else_falls_through_the_discriminator_check() {
        // Or a real error gets swallowed and resent-over instead of reaching
        // the user.
        for other in [
            json!({ "type": "error", "error": { "message": "Incorrect API key provided" } }),
            json!({ "type": "error", "error": { "message": "Unknown parameter: 'session.voice'." } }),
            json!({ "type": "session.created" }),
            json!({ "type": "response.output_audio.delta", "delta": "QUJD" }),
            json!({ "type": "error" }),
            json!({}),
            json!("session.type"),
            Value::Null,
        ] {
            assert!(!rejects_ga_discriminator(&other), "{other}");
        }
    }

    #[test]
    fn a_frame_that_merely_mentions_session_type_outside_an_error_is_not_a_rejection() {
        // Only `error` frames qualify, whatever the body says — otherwise a
        // `session.created` echoing the payload would trigger a repair.
        assert!(!rejects_ga_discriminator(&json!({
            "type": "session.created",
            "session": { "type": "realtime" },
            "message": "session.type"
        })));
    }
}
