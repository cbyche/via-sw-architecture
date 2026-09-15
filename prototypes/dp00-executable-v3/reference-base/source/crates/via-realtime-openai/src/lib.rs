//! VIA's OpenAI / Azure / litellm realtime provider.
//!
//! **This is the one crate whose upstream is ARGO rather than qwen-audio-agent.**
//! Ported from `tinicore/src/llm/providers/openai_live.rs` (2,704 lines,
//! 57 tests) with `tinicore-traits/src/live.rs` for its event vocabulary and
//! `docs/plans/VOICE_AGENT_BRINGUP.md` as the acceptance spec. `docs/architecture.md`
//! §3 and §14 record why: ARGO's client encodes four litellm-gateway properties
//! that each cost a device run to discover, and writing VIA's realtime client
//! from the qwen JavaScript would rediscover every one of them.
//!
//! ARGO is credited in [`NOTICE`](https://github.com/sanghn-kim/VIA/blob/main/NOTICE)
//! as VIA's Layer 1 upstream, and every ported construct below names its source
//! line.
//!
//! # It is one provider, not a second interface
//!
//! [`OpenAiRealtimeProvider`] is a plain [`via_realtime::RealtimeProvider`].
//! `via-realtime` still owns the session machine, the registry, the connection
//! status, the response watchdogs, the busy-retry ladder and the reconnect
//! backoff. What this crate owns is **the dialect and the connect walk**.
//!
//! ```
//! use via_realtime::RealtimeProvider;
//! use via_realtime_openai::{OpenAiRealtimeProvider, OpenAiSettings};
//! use via_core::Secret;
//!
//! let provider = OpenAiRealtimeProvider::new(OpenAiSettings {
//!     api_key: Secret::new("sk-test"),
//!     ..OpenAiSettings::default()
//! });
//!
//! assert_eq!(provider.key(), "openai");
//! assert!(provider.is_configured());
//! assert_eq!(
//!     provider.url().expect("a default endpoint"),
//!     "wss://api.openai.com/v1/realtime?model=gpt-realtime-2.1-mini",
//! );
//! ```
//!
//! # The five things it reproduces
//!
//! Each cost a device run to discover; ARGO bring-up §4 and §8b are the record.
//!
//! | # | What | Where |
//! | --- | --- | --- |
//! | 1 | **Binary frames, not Text.** The gateway frames the whole session as Binary; a reader that accepts only Text sees "connected but silent" for days. | [`connect::protocol_text`] |
//! | 2 | **`Sec-WebSocket-Protocol: realtime` is mandatory.** The upgrade succeeds without it but is not routed. | [`SUBPROTOCOL`] |
//! | 3 | **GA vs pre-GA schema**, decided by the endpoint rather than the route, repaired **once** in-session, and memoised **per host per process**. | [`schema`], [`OpenAiRealtimeProvider::build_session_for`], [`connect::InboundFilter`] |
//! | 4 | **The candidate walk.** Four URLs ordered by what the *host* is; a completed 101 is not a session; pings do not count; the deadline is computed once, outside the loop. | [`dialect::candidate_urls`], [`connect::connect`] |
//! | 5 | **The duplicate-`response.create` filter.** The gateway issues its own duplicate on each VAD commit and relays the refusal to us. | [`conflict`], [`connect::InboundFilter`] |
//!
//! # The two log markers
//!
//! ARGO's one-glance diagnostics are kept verbatim, on the
//! [`connect::LOG_TARGET`] tracing target:
//!
//! ```text
//! [live] dialect=azure host=sr-aic-llm-proxy.example
//! [live] schema=Ga for wss://sr-aic-llm-proxy.example/v1/realtime?model=gpt-realtime-2.1-mini
//! ```
//!
//! plus `[live] attempt`, `[live] candidate rejected`, `[live] probe accepted`,
//! `[live] connected via`, `[live] inbound type=`, `[live] outbound type=`,
//! `[live] endpoint rejected the GA discriminator` and `[live] dropped an
//! active-response conflict the client did not cause`.
//!
//! # What this crate does not own
//!
//! - **the provider key, label, default endpoint and identity shape** —
//!   [`via_catalog`]'s `openai` row;
//! - **the configuration-signature hash and its key order** —
//!   [`via_catalog::RealtimeIdentity`];
//! - **the environment reading** — [`via_core::config::resolve_realtime_frontend`];
//! - **the wire envelope, the id namespaces and the correlation key** —
//!   [`via_realtime::ga_realtime_protocol`] and
//!   [`via_realtime::openai_compatible_protocol`];
//! - **the socket lifecycle, the watchdogs and the busy-retry ladder** —
//!   [`via_realtime::RealtimeSession`];
//! - **the event decoding.** ARGO's `decode_server_event` maps into ARGO's own
//!   `LiveInboundEvent` vocabulary; VIA's Layer 1 consumes provider events
//!   directly and already accepts both the pre-GA and the GA event spellings
//!   (`via-realtime`'s `RESPONSE_ACTIVITY_TYPES`, `via-voice`'s response
//!   tables). Porting the decoder would have been a second vocabulary;
//!   `docs/deviations/phase-6-via-realtime-openai.md` records it;
//! - **every sentence a person or the model reads** — [`via_i18n`], through
//!   [`prompt`].
//!
//! [`via_core::config::resolve_realtime_frontend`]: via_core::config::resolve_realtime_frontend

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod classify;
pub mod conflict;
pub mod connect;
pub mod dialect;
pub mod prompt;
pub mod provider;
pub mod schema;
pub mod settings;

use std::sync::Arc;

use via_realtime::{RealtimeError, RealtimeProvider, RealtimeProviderRegistry};

pub use classify::{
    AUTHORIZATION_PATTERNS, CREDENTIAL_PATTERNS, MODEL_ACCESS_PATTERNS, QUOTA_PATTERNS,
    classify_openai_error, fatal_patterns,
};
pub use conflict::{
    ACTIVE_RESPONSE_CONFLICT_CODE, ACTIVE_RESPONSE_CONFLICT_PHRASE, CANCEL_NOT_ACTIVE_CODE,
    CANCEL_RACE_PHRASES, CreateAccounting, is_active_response_conflict, is_benign_cancel_race,
};
pub use connect::{
    AZURE_ERROR_CODE_HEADER, CONNECT_ERROR_BODY_LIMIT, CONNECT_TOTAL_BUDGET, ConnectBudget,
    FIRST_FRAME_TIMEOUT, InboundFilter, InboundVerdict, LOG_TARGET, MAX_CANDIDATES,
    PROBE_EXCERPT_CHARS, ProbeVerdict, SESSION_UPDATE_SIZE_WARN, classify_probe_frame, connect,
    connect_failure, connect_within, describe_ws_error, frame_kind, open_session, protocol_text,
    session_update_frame, wire,
};
pub use dialect::{
    AZURE_REALTIME_API_VERSION, AZURE_REALTIME_LEGACY_API_VERSION, INSECURE_SCHEME,
    OPENAI_REALTIME_HOST, OpenAiDialect, SECURE_SCHEME, SUBPROTOCOL, SUBPROTOCOL_HEADER,
    candidate_urls, encode_query_value, host_is_azure_resource, strip_scheme_and_path, url_scheme,
};
pub use prompt::{
    PERMISSION_AUTHORIZATION_ID_FIELD, PERMISSION_OPERATION_FIELD, PERMISSION_REQUEST_CLOSE_TAG,
    PERMISSION_REQUEST_OPEN_TAG, permission_request_text, permission_response_instructions,
    result_response_instructions, speak_response_instructions,
};
pub use provider::{
    GA_OUTPUT_MODALITIES, INPUT_SAMPLE_RATE, INPUT_TRANSCRIPTION_MODEL, KEY, OUTPUT_SAMPLE_RATE,
    OpenAiRealtimeProvider, PREVIEW_MODALITIES, SESSION_TYPE,
};
pub use schema::{
    RealtimeSchema, endpoint_key, protocol_for_url, protocol_from_route, rejects_ga_discriminator,
    remember_preview_endpoint,
};
pub use settings::{DEFAULT_OPENAI_REALTIME_MODEL, DEFAULT_OPENAI_REALTIME_VOICE, OpenAiSettings};

/// The provider, built from a resolved configuration.
#[must_use]
pub fn openai_provider(config: &via_core::Config) -> Arc<dyn RealtimeProvider> {
    Arc::new(OpenAiRealtimeProvider::from(config))
}

/// Register the OpenAI provider into an existing registry.
///
/// Registration validates the provider and refuses a key another already
/// claimed, so this is also how a host extension finds out that it picked a
/// colliding name.
///
/// # Errors
///
/// [`RealtimeError::ProviderNameTaken`] when `registry` already answers to
/// `openai`, or whatever [`via_realtime::validate_realtime_provider`] refuses.
pub fn register_openai_provider(
    registry: &mut RealtimeProviderRegistry,
    config: &via_core::Config,
) -> Result<(), RealtimeError> {
    registry.register(openai_provider(config)).map(|_| ())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use via_core::Config;
    use via_realtime::RealtimeProviderRegistry;

    use super::*;

    #[test]
    fn the_provider_registers_under_its_catalog_key_and_no_alias() {
        let config = Config::default();
        let mut registry = RealtimeProviderRegistry::with_default("openai");
        register_openai_provider(&mut registry, &config).expect("registers");
        assert_eq!(registry.provider_keys(), ["openai"]);
        assert_eq!(registry.resolvable_names(), ["openai"]);
        assert_eq!(registry.resolve(None).expect("default").key(), "openai");
        assert_eq!(
            registry.resolve(Some("  OpenAI ")).expect("resolves").key(),
            "openai"
        );
    }

    #[test]
    fn registering_twice_is_refused_rather_than_shadowing() {
        let config = Config::default();
        let mut registry = RealtimeProviderRegistry::with_default("openai");
        register_openai_provider(&mut registry, &config).expect("registers");
        assert_eq!(
            register_openai_provider(&mut registry, &config),
            Err(RealtimeError::ProviderNameTaken {
                name: "openai".into()
            })
        );
    }

    #[test]
    fn an_unconfigured_provider_is_invisible_in_a_picker_but_still_resolves() {
        // A fresh install: no credential. The provider must not show up as a
        // choice that would fail, and must still be reachable by name so the
        // Gateway can report why.
        let config = Config::default();
        let mut registry = RealtimeProviderRegistry::with_default("openai");
        register_openai_provider(&mut registry, &config).expect("registers");
        assert!(registry.describe_providers(true).is_empty());
        assert!(registry.resolve(Some("openai")).is_ok());
    }
}
