//! The two providers upstream ships.
//!
//! Ported from `qwen-audio-agent` v1.11.0
//! `server/src/voice/providers/dashscope.mjs`, `providers/s2s.mjs` and the
//! registration half of `providers/registry.mjs`, against the provider half of
//! `server/test/realtime-provider.test.mjs`.
//!
//! | Provider | Key (alias) | Dialect | What it is |
//! | --- | --- | --- | --- |
//! | [`DashScopeProvider`] | `dashscope` (`qwen`) | beta OpenAI Realtime | Cloud Qwen Audio 3.0 / Qwen3.5 Omni |
//! | [`SpeechToSpeechProvider`] | `speech-to-speech` (`s2s`) | GA OpenAI Realtime | A user-run huggingface/speech-to-speech endpoint |
//!
//! ```
//! use via_realtime::RealtimeProvider;
//! use via_realtime_dashscope::{DashScopeProvider, DashScopeSettings};
//! use via_core::Secret;
//!
//! let provider = DashScopeProvider::new(DashScopeSettings {
//!     api_key: Secret::new("sk-test"),
//!     ..DashScopeSettings::default()
//! });
//!
//! assert_eq!(provider.key(), "dashscope");
//! assert_eq!(provider.aliases(), ["qwen"]);
//! assert!(provider.is_configured());
//! assert_eq!(
//!     provider.url().expect("a default endpoint"),
//!     "wss://dashscope.aliyuncs.com/api-ws/v1/realtime?model=qwen-audio-3.0-realtime-plus",
//! );
//! ```
//!
//! # ⚠ The bug this crate exists partly to *not* have
//!
//! `docs/architecture.md` §7, *"One bug not to port"*. Upstream gates
//! `session.turn_detection` on `transportCapabilities.audioInput`, and its
//! model catalog answers an unrecognised id with an all-capabilities-false
//! profile. Put together, that pair builds **a session that hears nothing** —
//! no input format, no turn detection, no error, no log line. Upstream's guard
//! against it is a separate `family === 'unknown'` check in a different file,
//! and a local model id is by definition not in the DashScope table.
//!
//! [`via_catalog`] removed the fallback by making an unrecognised id an `Err`.
//! This crate re-expresses the guard where the knowledge lives:
//! [`DashScopeProvider::preflight`] turns that `Err` into
//! [`RealtimeError::UnsupportedModel`] naming the id, and `via-realtime` runs
//! `preflight()` **before** `is_configured()` and both before any socket is
//! opened.
//!
//! ```
//! use via_realtime::{RealtimeError, RealtimeProvider};
//! use via_realtime_dashscope::{DashScopeProvider, DashScopeSettings};
//!
//! let provider = DashScopeProvider::new(DashScopeSettings {
//!     model: "qwen3.5-omni-plus-realtime-future".into(),
//!     ..DashScopeSettings::default()
//! });
//!
//! assert_eq!(
//!     provider.preflight(),
//!     Err(RealtimeError::UnsupportedModel {
//!         id: "qwen3.5-omni-plus-realtime-future".into(),
//!         label: "Qwen-Audio-Realtime".into(),
//!     }),
//! );
//! ```
//!
//! # Why both providers live in one crate
//!
//! `docs/architecture.md` §15 lists `via-realtime-s2s` as a separate phase-6
//! crate. Shipping the two together is a recorded deviation
//! (`docs/deviations/phase-5-via-realtime-dashscope.md`): `s2s.mjs` is 138
//! lines, it needs no dependency `dashscope.mjs` does not already have, and
//! `providers/registry.mjs` registers them as one built-in pair —
//! [`builtin_provider_registry`] is that registration. A separate crate for 138
//! lines would add a seam the implementation VIA follows does not have.
//!
//! They are not, however, the same provider wearing two hats. The dialects
//! differ (beta vs GA), the capability declarations differ in four of five
//! flags, the response-start budgets differ by 2×, and the error corpora share
//! exactly one arm. Every one of those differences is a separate contract, and
//! `tests/` asserts each.
//!
//! # What this crate does not own
//!
//! - **the model table** — the four profiles, their ids, labels, voices and
//!   turn-detection modes are [`via_catalog`]'s, and so is the family-scoped
//!   voice-override rule;
//! - **the provider keys, aliases and endpoint defaults** — [`via_catalog`];
//! - **the configuration-signature hash and its key order** —
//!   [`via_catalog::RealtimeIdentity`];
//! - **the environment reading** — [`via_core::config::resolve_realtime_frontend`];
//! - **the wire envelope, the id namespaces and the correlation key** —
//!   [`via_realtime::openai_compatible_protocol`] and
//!   [`via_realtime::ga_realtime_protocol`];
//! - **the socket, the watchdogs, the busy-retry ladder** —
//!   [`via_realtime::RealtimeSession`];
//! - **every sentence a person or the model reads** — [`via_i18n`].
//!
//! [`RealtimeError::UnsupportedModel`]: via_realtime::RealtimeError::UnsupportedModel
//! [`via_realtime::RealtimeSession`]: via_realtime::RealtimeSession

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod classify;
pub mod dashscope;
pub mod prompt;
pub mod settings;
pub mod speech_to_speech;

use std::sync::Arc;

use via_realtime::{RealtimeError, RealtimeProvider, RealtimeProviderRegistry};

pub use classify::{
    DASHSCOPE_FATAL_PATTERNS, DASHSCOPE_INPUT_BUSY, NO_ACTIVE_RESPONSE,
    SPEECH_TO_SPEECH_CAPACITY_PATTERN, SPEECH_TO_SPEECH_RESPONSE_SLOT_BUSY,
    classify_dashscope_error, classify_speech_to_speech_error, patterns_compile,
};
pub use dashscope::DashScopeProvider;
pub use prompt::{
    PERMISSION_AUTHORIZATION_ID_FIELD, PERMISSION_OPERATION_FIELD, PERMISSION_REQUEST_CLOSE_TAG,
    PERMISSION_REQUEST_OPEN_TAG, permission_request_text, permission_response_instructions,
    result_response_instructions, speak_response_instructions,
};
pub use settings::{DashScopeSettings, SpeechToSpeechSettings};
pub use speech_to_speech::SpeechToSpeechProvider;

/// The two built-in providers, in upstream's registration order.
///
/// External contract — `providers/registry.mjs:17-19`:
/// `createRealtimeProviderRegistry({ providers: [dashscopeProvider, s2sProvider] })`.
/// The order is observable: `RealtimeProviderRegistry::list` returns it, and the
/// unsupported-provider message lists it.
#[must_use]
pub fn builtin_providers(config: &via_core::Config) -> Vec<Arc<dyn RealtimeProvider>> {
    vec![
        Arc::new(DashScopeProvider::from(config)),
        Arc::new(SpeechToSpeechProvider::from(config)),
    ]
}

/// Register the two built-in providers into an existing registry.
///
/// Registration validates each provider and refuses a key or alias another
/// already claimed, so this is also how a host extension finds out that it
/// picked a colliding name.
///
/// # Errors
///
/// [`RealtimeError::ProviderNameTaken`] when `registry` already answers to
/// `dashscope`, `qwen`, `speech-to-speech` or `s2s`, or whatever
/// [`via_realtime::validate_realtime_provider`] refuses.
pub fn register_builtin_providers(
    registry: &mut RealtimeProviderRegistry,
    config: &via_core::Config,
) -> Result<(), RealtimeError> {
    for provider in builtin_providers(config) {
        registry.register(provider)?;
    }
    Ok(())
}

/// A registry holding the two built-in providers, defaulting to the configured
/// one.
///
/// Upstream `defaultRealtimeProviderRegistry` plus
/// `resolveRealtimeProvider(requested)`'s `|| config.audioProvider` fallback,
/// which is what makes `registry.resolve(None)` answer with the operator's
/// choice rather than with a hard-coded key.
///
/// # Errors
///
/// As [`register_builtin_providers`].
pub fn builtin_provider_registry(
    config: &via_core::Config,
) -> Result<RealtimeProviderRegistry, RealtimeError> {
    let mut registry = RealtimeProviderRegistry::with_default(config.realtime.provider);
    register_builtin_providers(&mut registry, config)?;
    Ok(registry)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use via_core::Config;

    use super::*;

    #[test]
    fn the_registry_holds_both_providers_in_upstream_order() {
        let registry = builtin_provider_registry(&Config::default()).expect("registers");
        assert_eq!(registry.provider_keys(), ["dashscope", "speech-to-speech"]);
        assert_eq!(
            registry.resolvable_names(),
            ["dashscope", "qwen", "s2s", "speech-to-speech"]
        );
    }

    #[test]
    fn both_aliases_resolve_case_insensitively() {
        let registry = builtin_provider_registry(&Config::default()).expect("registers");
        for (requested, expected) in [
            ("qwen", "dashscope"),
            ("QWEN", "dashscope"),
            ("  Qwen  ", "dashscope"),
            ("s2s", "speech-to-speech"),
            ("S2S", "speech-to-speech"),
        ] {
            assert_eq!(
                registry.resolve(Some(requested)).expect("resolves").key(),
                expected,
                "{requested}"
            );
        }
    }

    #[test]
    fn the_default_provider_is_the_configured_one() {
        let registry = builtin_provider_registry(&Config::default()).expect("registers");
        assert_eq!(registry.default_provider(), "dashscope");
        assert_eq!(registry.resolve(None).expect("default").key(), "dashscope");
    }

    #[test]
    fn registering_twice_is_refused_rather_than_shadowing() {
        let config = Config::default();
        let mut registry = builtin_provider_registry(&config).expect("registers");
        assert_eq!(
            register_builtin_providers(&mut registry, &config),
            Err(RealtimeError::ProviderNameTaken {
                name: "dashscope".into()
            })
        );
    }

    #[test]
    fn an_unconfigured_pair_is_invisible_in_a_picker_but_still_resolves() {
        // A fresh install: no DashScope key, no speech-to-speech endpoint.
        let registry = builtin_provider_registry(&Config::default()).expect("registers");
        assert!(registry.describe_providers(false).is_empty());
        assert!(registry.resolve(Some("dashscope")).is_ok());
        assert!(registry.resolve(Some("s2s")).is_ok());
    }
}
