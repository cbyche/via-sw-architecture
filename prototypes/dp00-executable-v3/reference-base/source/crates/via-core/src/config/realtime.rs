//! Resolving the realtime frontend from the environment.
//!
//! Ported from `resolveRealtimeFrontendConfiguration`
//! (`shared/realtime-provider-catalog.mjs:84-155`). The *tables* — provider
//! keys, aliases, labels, default endpoints, model profiles, and the
//! configuration-signature machinery — are `via-catalog`'s and are deliberately
//! pure. This module is the environment reading on top of them, which is the
//! half `via-catalog` refuses to do.
//!
//! # Two contracts worth naming
//!
//! **The voice override is family-scoped.** `VIA_REALTIME_VOICE` applies only to
//! the `audio` family and `VIA_OMNI_REALTIME_VOICE` only to `omni`, so switching
//! models never clobbers the other family's preference. With neither set the
//! resolved voice is the **empty string**, not the profile default — the profile
//! default is applied later, by the provider, in `session.update`.
//!
//! **A default endpoint is not a configuration.** `speech_to_speech_configured`
//! is true only when the user set an endpoint *explicitly* or selected the
//! provider. Upstream's comment says why: "Do not advertise a local service
//! merely because a default endpoint exists."
//!
//! # Where VIA extends upstream
//!
//! Upstream has two providers; VIA has five (`docs/architecture.md` §7). The
//! three new ones need a `configured` predicate and an identity shape, neither
//! of which upstream has an opinion about. Each extension is marked below.

use via_catalog::realtime_model::{ModelFamily, local_realtime_model_profile};
use via_catalog::realtime_provider::{
    dashscope_workspace_realtime_url, normalize_dashscope_base_url,
    normalize_speech_to_speech_base_url,
};
use via_catalog::{
    DEFAULT_DASHSCOPE_REALTIME_MODEL, DEFAULT_DASHSCOPE_REALTIME_URL, DEFAULT_REALTIME_PROVIDER,
    DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL, IdentityShape, RealtimeIdentity,
    realtime_provider_definition, resolve_dashscope_realtime_model_profile,
};

use crate::config::names;
use crate::env::EnvMap;
use crate::error::CoreError;
use crate::secret::Secret;

/// The realtime frontend, resolved.
///
/// Every field is computed regardless of which provider is active, because
/// `/api/health` publishes the DashScope and speech-to-speech values side by
/// side so a settings UI can offer both. Only [`Self::configured`] and
/// [`Self::signature`] branch on the provider.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RealtimeFrontend {
    /// Canonical provider key.
    pub provider: &'static str,
    /// Human label for the provider.
    pub label: &'static str,
    /// Whether the active provider has everything it needs to connect.
    pub configured: bool,
    /// Lowercase hex SHA-256 of the configuration identity.
    pub signature: String,
    /// `VIA_REALTIME_API_KEY || DASHSCOPE_API_KEY`.
    pub dashscope_api_key: Secret,
    /// Resolved DashScope WebSocket endpoint, trailing `?` stripped.
    pub dashscope_realtime_url: String,
    /// Resolved DashScope model id.
    pub dashscope_model: String,
    /// Family-scoped voice override, or empty.
    pub dashscope_voice: String,
    /// Resolved speech-to-speech endpoint, trailing `/` stripped.
    pub speech_to_speech_realtime_url: String,
    /// Optional bearer token for a proxied speech-to-speech service.
    pub speech_to_speech_auth_token: Secret,
    /// Whether speech-to-speech was configured explicitly.
    pub speech_to_speech_configured: bool,
}

impl Default for RealtimeFrontend {
    /// The frontend an empty environment resolves to, minus the signature.
    ///
    /// Every value comes from `via-catalog`, which is where the realtime
    /// defaults live; nothing is retyped here. [`Self::signature`] is left
    /// empty because it is a digest of the other fields rather than a default —
    /// [`resolve_realtime_frontend`] computes it.
    fn default() -> Self {
        Self {
            provider: DEFAULT_REALTIME_PROVIDER,
            label: realtime_provider_definition(DEFAULT_REALTIME_PROVIDER)
                .map(|definition| definition.label)
                .unwrap_or_default(),
            configured: false,
            signature: String::new(),
            dashscope_api_key: Secret::default(),
            dashscope_realtime_url: DEFAULT_DASHSCOPE_REALTIME_URL.to_owned(),
            dashscope_model: DEFAULT_DASHSCOPE_REALTIME_MODEL.to_owned(),
            dashscope_voice: String::new(),
            speech_to_speech_realtime_url: DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL.to_owned(),
            speech_to_speech_auth_token: Secret::default(),
            speech_to_speech_configured: false,
        }
    }
}

/// The environment variable whose value overrides the voice for `family`.
///
/// **External contract** — `shared/realtime-provider-catalog.mjs:51-59`, with
/// the mapping itself owned by [`ModelFamily::voice_override_env`].
#[must_use]
pub fn voice_override(env: &EnvMap, family: ModelFamily) -> String {
    env.get_trimmed(family.voice_override_env()).to_owned()
}

/// Read the realtime frontend out of an environment.
///
/// # Errors
///
/// * [`CoreError::Catalog`] wrapping `UnsupportedRealtimeProvider` when
///   `VIA_REALTIME_PROVIDER` names nothing.
/// * [`CoreError::Catalog`] wrapping `UnknownRealtimeModel` when the DashScope
///   provider is selected with a model id that is not in the catalog. Upstream
///   answers that with an all-capabilities-false profile, which opens a session
///   that hears nothing; VIA refuses instead (`docs/architecture.md` §7).
pub fn resolve_realtime_frontend(env: &EnvMap) -> Result<RealtimeFrontend, CoreError> {
    let definition = realtime_provider_definition(env.get(names::REALTIME_PROVIDER).unwrap_or(""))?;

    let credential = env
        .first_truthy(&[names::REALTIME_API_KEY, names::DASHSCOPE_API_KEY])
        .unwrap_or_default()
        .trim()
        .to_owned();

    let workspace_url = env
        .get_truthy(names::DASHSCOPE_WORKSPACE_ID)
        .map(dashscope_workspace_realtime_url);
    let dashscope_realtime_url = normalize_dashscope_base_url(
        env.first_truthy(&[names::REALTIME_BASE_URL, names::REALTIME_URL])
            .unwrap_or_else(|| {
                workspace_url
                    .as_deref()
                    .unwrap_or(DEFAULT_DASHSCOPE_REALTIME_URL)
            }),
    )
    .to_owned();

    let dashscope_model = {
        let configured = env.get_trimmed(names::REALTIME_MODEL);
        if configured.is_empty() {
            DEFAULT_DASHSCOPE_REALTIME_MODEL.to_owned()
        } else {
            configured.to_owned()
        }
    };

    let speech_to_speech_realtime_url = normalize_speech_to_speech_base_url(
        env.first_truthy(&[
            names::SPEECH_TO_SPEECH_REALTIME_URL,
            names::S2S_REALTIME_URL,
        ])
        .unwrap_or(DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL),
    )
    .to_owned();

    let speech_to_speech_auth_token = env
        .first_truthy(&[names::SPEECH_TO_SPEECH_AUTH_TOKEN, names::S2S_API_KEY])
        .unwrap_or_default()
        .trim()
        .to_owned();

    // A default endpoint alone never advertises the provider.
    let speech_to_speech_configured = !env
        .get_trimmed(names::SPEECH_TO_SPEECH_REALTIME_URL)
        .is_empty()
        || !env.get_trimmed(names::S2S_REALTIME_URL).is_empty()
        || definition.key == "speech-to-speech";

    // The DashScope voice override needs the model's family, and an id that is
    // not in the catalog has no family. Only the DashScope provider consults
    // the catalog; the others carry ids VIA's own providers define.
    let dashscope_voice = if definition.key == "dashscope" {
        let profile = resolve_dashscope_realtime_model_profile(&dashscope_model)?;
        voice_override(env, profile.family)
    } else {
        // VIA extension: `openai` and `local-omni` carry user-chosen model
        // coordinates, so their voice lives under the `local` family's variable
        // rather than either DashScope-family one. `via-realtime-openai` may
        // narrow this; nothing upstream covers it.
        voice_override(env, local_realtime_model_profile(&dashscope_model).family)
    };

    let (endpoint, identity_credential) = match definition.key {
        "dashscope" => (dashscope_realtime_url.clone(), credential.clone()),
        "speech-to-speech" => (
            speech_to_speech_realtime_url.clone(),
            speech_to_speech_auth_token.clone(),
        ),
        // VIA extension: the remaining three take the same explicit base-URL
        // chain, falling back to whatever default the catalog declares — none,
        // for `local-omni:pipeline` and `mock`.
        _ => (
            env.first_truthy(&[names::REALTIME_BASE_URL, names::REALTIME_URL])
                .map(|value| normalize_dashscope_base_url(value).to_owned())
                .or_else(|| definition.default_url.map(str::to_owned))
                .unwrap_or_default(),
            credential.clone(),
        ),
    };

    let configured = match definition.key {
        "dashscope" => !credential.is_empty(),
        "speech-to-speech" => speech_to_speech_configured,
        // VIA extension: `openai` needs a key; `local-omni` and `mock` need
        // nothing, because the pipeline runs in-process and the mock is a
        // fixture.
        "openai" => !credential.is_empty(),
        _ => true,
    };

    let signature = RealtimeIdentity::new(
        definition.identity_shape,
        definition.key,
        endpoint,
        dashscope_model.as_str(),
        dashscope_voice.as_str(),
        identity_credential,
    )
    .signature();

    Ok(RealtimeFrontend {
        provider: definition.key,
        label: definition.label,
        configured,
        signature,
        dashscope_api_key: Secret::new(credential),
        dashscope_realtime_url,
        dashscope_model,
        dashscope_voice,
        speech_to_speech_realtime_url,
        speech_to_speech_auth_token: Secret::new(speech_to_speech_auth_token),
        speech_to_speech_configured,
    })
}

/// The identity shape the active provider hashes under.
///
/// Exposed so `via-conformance` can assert the two upstream providers keep
/// their upstream shapes.
#[must_use]
pub fn identity_shape(provider: &str) -> Option<IdentityShape> {
    realtime_provider_definition(provider)
        .ok()
        .map(|definition| definition.identity_shape)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> EnvMap {
        pairs.iter().copied().collect()
    }

    #[test]
    fn a_default_endpoint_alone_does_not_configure_speech_to_speech() {
        let frontend = resolve_realtime_frontend(&env(&[])).expect("defaults resolve");
        assert_eq!(
            frontend.speech_to_speech_realtime_url,
            "ws://127.0.0.1:8765/v1/realtime"
        );
        assert!(!frontend.speech_to_speech_configured);
    }

    #[test]
    fn the_voice_override_is_family_scoped() {
        let audio = resolve_realtime_frontend(&env(&[
            ("DASHSCOPE_API_KEY", "k"),
            ("VIA_REALTIME_MODEL", "qwen-audio-3.0-realtime-plus"),
            ("VIA_REALTIME_VOICE", "Cherry"),
            ("VIA_OMNI_REALTIME_VOICE", "Ethan-custom"),
        ]))
        .expect("a catalog model resolves");
        let omni = resolve_realtime_frontend(&env(&[
            ("DASHSCOPE_API_KEY", "k"),
            ("VIA_REALTIME_MODEL", "qwen3.5-omni-plus-realtime"),
            ("VIA_REALTIME_VOICE", "Cherry"),
            ("VIA_OMNI_REALTIME_VOICE", "Ethan-custom"),
        ]))
        .expect("a catalog model resolves");
        assert_eq!(audio.dashscope_voice, "Cherry");
        assert_eq!(omni.dashscope_voice, "Ethan-custom");
    }

    #[test]
    fn changing_only_the_model_changes_the_signature() {
        let shared: &[(&str, &str)] = &[
            ("DASHSCOPE_API_KEY", "same-key"),
            ("VIA_REALTIME_BASE_URL", "wss://gateway.example/realtime"),
            ("VIA_REALTIME_VOICE", "same-voice"),
        ];
        let mut first: Vec<(&str, &str)> = shared.to_vec();
        first.push(("VIA_REALTIME_MODEL", "qwen-audio-3.0-realtime-plus"));
        let mut second: Vec<(&str, &str)> = shared.to_vec();
        second.push(("VIA_REALTIME_MODEL", "qwen3.5-omni-plus-realtime"));

        let a = resolve_realtime_frontend(&first.into_iter().collect()).expect("resolves");
        let b = resolve_realtime_frontend(&second.into_iter().collect()).expect("resolves");
        assert_ne!(a.signature, b.signature);
    }
}
