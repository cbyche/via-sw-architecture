//! The setup gate: may the Gateway start, and what is still missing.
//!
//! Ported from `shared/gateway-setup.mjs`. Upstream's comment says why it
//! exists, and it is worth repeating:
//!
//! > A Gateway that listens but cannot connect its voice is harder to diagnose
//! > than a refusal it can act on — the user only discovers the missing
//! > credential when their first sentence fails.
//!
//! So the Gateway refuses, with a `missing` list carrying, per entry, the
//! settings `field` a UI binds to, the environment `key` an operator sets, and
//! a `message` fit to render. The refusal happens **before the instance lease
//! is touched** (`server/src/index.mjs:50`), so a misconfigured start never
//! disturbs a running one.
//!
//! # The phase-1 milestone
//!
//! `docs/architecture.md` §15: phase 1 ends when *"`via gateway` refuses to
//! start unconfigured with the exact message in all three locales"*. The code
//! is `VIA_GATEWAY_SETUP_REQUIRED` and is never localized; the sentence beside
//! it comes from `via-i18n` and is. `tests/setup.rs` asserts all three.
//!
//! # The opt-out
//!
//! `VIA_ALLOW_UNCONFIGURED=1` skips the gate entirely. The comparison is to the
//! **exact string `1`** — `true`, `yes` and `TRUE` do not opt out
//! (`shared/gateway-setup.mjs:41`). That is deliberate: a harness that never
//! opens a voice connection sets it knowingly, and a typo fails closed.

use via_i18n::{Locale, format, keys, t};
use via_protocol::MissingSetting;

use crate::config::names;
use crate::config::realtime::resolve_realtime_frontend;
use crate::env::EnvMap;
use crate::error::CoreError;

/// The only value of `VIA_ALLOW_UNCONFIGURED` that opts out.
///
/// **External contract** — `shared/gateway-setup.mjs:41`.
pub const ALLOW_UNCONFIGURED_VALUE: &str = "1";

/// The settings field naming a missing DashScope credential.
///
/// **External contract** — `shared/gateway-setup.mjs:20`. A settings UI binds
/// to this, so it is a wire name, not prose.
pub const FIELD_DASHSCOPE_API_KEY: &str = "dashscopeApiKey";

/// The settings field naming a missing speech-to-speech endpoint.
///
/// **External contract** — `shared/gateway-setup.mjs:26`.
pub const FIELD_SPEECH_TO_SPEECH_URL: &str = "speechToSpeechRealtimeUrl";

/// The settings field naming a missing realtime API key.
///
/// VIA-owned: upstream has no `openai` provider, so it has no field for one.
/// The name follows the two upstream fields' camelCase convention.
pub const FIELD_REALTIME_API_KEY: &str = "realtimeApiKey";

/// What the setup gate found.
///
/// **External contract** — `shared/gateway-setup.mjs:30-34`, whose three fields
/// are `ready`, `provider` and `missing`, in that order.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct SetupStatus {
    /// Whether the Gateway may start.
    pub ready: bool,
    /// The active realtime provider key.
    pub provider: &'static str,
    /// Every missing setting. Empty exactly when [`Self::ready`].
    pub missing: Vec<MissingSetting>,
}

impl SetupStatus {
    /// The refusal sentence, rendered in `locale`.
    ///
    /// **External contract** — `shared/gateway-setup.mjs:44-48`:
    /// `Gateway 启动被拒绝，缺少必填配置：<KEY>（<message>）`, entries joined by
    /// `；`. The sentence, the per-entry wrapper and the separator are three
    /// `via-i18n` keys rather than one, because the wrapper's brackets and the
    /// separator are fullwidth in `zh` and ASCII in `en`/`ko`.
    #[must_use]
    pub fn refusal_message(&self, locale: Locale) -> String {
        let details = self
            .missing
            .iter()
            .map(|item| {
                format(
                    locale,
                    keys::GATEWAY_SETUP_REQUIRED_ITEM,
                    &[
                        ("key", item.key.as_str()),
                        ("message", item.message.as_str()),
                    ],
                )
            })
            .collect::<Vec<_>>()
            .join(t(locale, keys::GATEWAY_SETUP_REQUIRED_JOIN));
        format(
            locale,
            keys::GATEWAY_SETUP_REQUIRED,
            &[("details", details.as_str())],
        )
    }
}

/// Whether the environment opts out of the gate.
///
/// **External contract** — `shared/gateway-setup.mjs:41`.
#[must_use]
pub fn allows_unconfigured(env: &EnvMap) -> bool {
    env.get(names::ALLOW_UNCONFIGURED).unwrap_or_default() == ALLOW_UNCONFIGURED_VALUE
}

/// Inspect an environment without refusing.
///
/// # Errors
///
/// [`CoreError::Catalog`] when the realtime configuration cannot be resolved at
/// all — an unsupported provider, or an unknown DashScope model id. Those are
/// *malformed* configuration rather than *missing* configuration, so they are
/// errors rather than `missing` entries, exactly as upstream's
/// `normalizeRealtimeProvider` throws before `gatewaySetupStatus` can build a
/// list.
pub fn gateway_setup_status(env: &EnvMap, locale: Locale) -> Result<SetupStatus, CoreError> {
    let frontend = resolve_realtime_frontend(env)?;
    let mut missing = Vec::new();
    if !frontend.configured {
        let (field, key, message) = match frontend.provider {
            "dashscope" => (
                FIELD_DASHSCOPE_API_KEY,
                names::DASHSCOPE_API_KEY,
                t(locale, keys::GATEWAY_MISSING_DASHSCOPE_API_KEY).to_owned(),
            ),
            "speech-to-speech" => (
                FIELD_SPEECH_TO_SPEECH_URL,
                names::SPEECH_TO_SPEECH_REALTIME_URL,
                format(
                    locale,
                    keys::REALTIME_MISSING_CONFIGURATION,
                    &[("label", frontend.label)],
                ),
            ),
            // VIA extension: `openai` is the only new provider that can be
            // unconfigured — `local-omni` runs in-process and `mock` is a
            // fixture, so neither ever reaches this arm.
            _ => (
                FIELD_REALTIME_API_KEY,
                names::REALTIME_API_KEY,
                format(
                    locale,
                    keys::REALTIME_MISSING_CONFIGURATION,
                    &[("label", frontend.label)],
                ),
            ),
        };
        missing.push(MissingSetting {
            field: field.to_owned(),
            key: key.to_owned(),
            message,
        });
    }
    Ok(SetupStatus {
        ready: missing.is_empty(),
        provider: frontend.provider,
        missing,
    })
}

/// Refuse to start unless the realtime frontend is configured.
///
/// # Errors
///
/// * [`CoreError::GatewaySetupRequired`] carrying the `missing` list and the
///   localized sentence. Its [`code`](CoreError::code) is
///   `VIA_GATEWAY_SETUP_REQUIRED`, and
///   [`protocol_error`](CoreError::protocol_error) converts it to the
///   structured [`via_protocol::ProtocolError::SetupRequired`] a client sees.
/// * [`CoreError::Catalog`] — see [`gateway_setup_status`].
pub fn assert_gateway_setup(env: &EnvMap, locale: Locale) -> Result<(), CoreError> {
    if allows_unconfigured(env) {
        return Ok(());
    }
    let status = gateway_setup_status(env, locale)?;
    if status.ready {
        return Ok(());
    }
    let message = status.refusal_message(locale);
    Err(CoreError::GatewaySetupRequired {
        missing: status.missing,
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> EnvMap {
        pairs.iter().copied().collect()
    }

    #[test]
    fn an_empty_environment_is_refused() {
        let error = assert_gateway_setup(&env(&[]), Locale::En)
            .expect_err("no DASHSCOPE_API_KEY means no voice");
        assert_eq!(error.code(), via_protocol::CODE_GATEWAY_SETUP_REQUIRED);
    }

    #[test]
    fn only_the_exact_string_one_opts_out() {
        for value in ["true", "TRUE", "yes", "0", "", "01", " 1"] {
            assert!(
                assert_gateway_setup(&env(&[("VIA_ALLOW_UNCONFIGURED", value)]), Locale::En)
                    .is_err(),
                "{value:?} must not open the gate"
            );
        }
        assert!(assert_gateway_setup(&env(&[("VIA_ALLOW_UNCONFIGURED", "1")]), Locale::En).is_ok());
    }

    #[test]
    fn a_dashscope_key_is_enough() {
        let status = gateway_setup_status(&env(&[("DASHSCOPE_API_KEY", "k")]), Locale::En)
            .expect("a catalog default model resolves");
        assert!(status.ready);
        assert_eq!(status.provider, "dashscope");
        assert!(status.missing.is_empty());
    }
}
