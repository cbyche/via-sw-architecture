//! Realtime provider registry.
//!
//! Ported from `shared/realtime-provider-catalog.mjs`. Upstream ships two
//! providers; VIA ships five (`docs/architecture.md` §7), so this is an
//! *extension* of the upstream table rather than a copy of it. The two upstream
//! entries — their keys, labels, alias lists and default URLs — are reproduced
//! exactly, and they stay first so that the ordered name list keeps upstream's
//! prefix.
//!
//! The `qwen` alias is KEEP per `docs/rebrand.md`: it names the frontend voice
//! engine's qwen runtime, clients already send `provider: "qwen"` on the connect
//! event, and users already have it in their config.

use serde::Serialize;

use crate::error::CatalogError;

/// Default realtime provider key.
///
/// External contract — `shared/realtime-provider-catalog.mjs:18`.
pub const DEFAULT_REALTIME_PROVIDER: &str = "dashscope";

/// Default DashScope realtime endpoint.
///
/// External contract — `shared/realtime-provider-catalog.mjs:19`. A vendor
/// endpoint; KEEP.
pub const DEFAULT_DASHSCOPE_REALTIME_URL: &str = "wss://dashscope.aliyuncs.com/api-ws/v1/realtime";

/// Default endpoint for the user-run `speech-to-speech` service.
///
/// External contract — `shared/realtime-provider-catalog.mjs:20`. Also quoted
/// verbatim in the generated `config.env` template.
pub const DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL: &str = "ws://127.0.0.1:8765/v1/realtime";

/// Default OpenAI realtime endpoint.
///
/// **Not from the qwen upstream** — VIA's `openai` provider is ported from ARGO
/// (`tinicore/src/llm/providers/openai_live.rs:9`), whose candidate walk starts
/// at `wss://api.openai.com/v1/realtime?model=...`. Azure and litellm gateways
/// override it.
pub const DEFAULT_OPENAI_REALTIME_URL: &str = "wss://api.openai.com/v1/realtime";

/// Host-and-path suffix of the DashScope per-workspace endpoint.
///
/// External contract — `shared/realtime-provider-catalog.mjs:94`. Used verbatim
/// when `DASHSCOPE_WORKSPACE_ID` is set and no explicit base URL is given. See
/// [`dashscope_workspace_realtime_url`].
pub const DASHSCOPE_WORKSPACE_REALTIME_URL_SUFFIX: &str =
    ".cn-beijing.maas.aliyuncs.com/api-ws/v1/realtime";

/// Which shape a provider's configuration-identity object takes.
///
/// Upstream branches on `provider === 'dashscope'` inline
/// (`shared/realtime-provider-catalog.mjs:122-134`). Making it a declared
/// property of the provider is what lets VIA add providers without touching the
/// signature code — and, more importantly, without accidentally emitting `null`
/// model/voice keys into the hash. See [`crate::signature`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum IdentityShape {
    /// `{ provider, endpoint, model, voice, credential }` — the dashscope branch.
    ModelAndVoice,
    /// `{ provider, endpoint, credential }` — the speech-to-speech branch. The
    /// `model` and `voice` keys are **absent**, not null.
    EndpointOnly,
}

/// One entry in the realtime provider registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDefinition {
    /// The canonical provider key, as accepted by config and by the client
    /// connect event and echoed in `/api/health.realtimeProvider`.
    pub key: &'static str,
    /// The human label, echoed in `/api/health.realtimeLabel`.
    pub label: &'static str,
    /// Alternate spellings that resolve to [`Self::key`].
    pub aliases: &'static [&'static str],
    /// Endpoint used when nothing overrides it, where one exists.
    pub default_url: Option<&'static str>,
    /// Shape of this provider's configuration-identity object.
    pub identity_shape: IdentityShape,
}

impl ProviderDefinition {
    /// Whether `value` is this provider's key or one of its aliases.
    ///
    /// `value` is compared as given; callers normally pass a trimmed, lowercased
    /// string — [`normalize_realtime_provider`] does that for them.
    pub fn matches(&self, value: &str) -> bool {
        self.key == value || self.aliases.contains(&value)
    }
}

/// The provider registry, in catalog order.
///
/// The first two entries and their exact keys, labels and aliases are external
/// contract — `shared/realtime-provider-catalog.mjs:22-33`. The last three are
/// VIA's, per `docs/architecture.md` §7.
static REALTIME_PROVIDERS: [ProviderDefinition; 5] = [
    ProviderDefinition {
        key: "dashscope",
        label: "DashScope",
        // KEEP: `qwen` names the vendor's realtime runtime, not VIA.
        aliases: &["qwen"],
        default_url: Some(DEFAULT_DASHSCOPE_REALTIME_URL),
        identity_shape: IdentityShape::ModelAndVoice,
    },
    ProviderDefinition {
        key: "speech-to-speech",
        label: "Hugging Face Speech-to-Speech",
        aliases: &["s2s"],
        default_url: Some(DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL),
        identity_shape: IdentityShape::EndpointOnly,
    },
    ProviderDefinition {
        // VIA's primary realtime client, ported from ARGO's `openai_live.rs`
        // rather than from `providers/dashscope.mjs`. Covers OpenAI, Azure and
        // litellm gateways. `docs/architecture.md` §3.
        key: "openai",
        label: "OpenAI Realtime",
        aliases: &[],
        default_url: Some(DEFAULT_OPENAI_REALTIME_URL),
        identity_shape: IdentityShape::ModelAndVoice,
    },
    ProviderDefinition {
        // The on-device path. Two modes, `local-omni:pipeline` (100% Rust:
        // sherpa-onnx + llama-cpp-2) and `local-omni:endpoint` (an OpenAI-
        // compatible local server); the mode is a session setting, not a
        // separate provider key. There is deliberately no default endpoint —
        // `pipeline` needs none and `endpoint` must be told where to look.
        key: "local-omni",
        label: "Local Omni",
        aliases: &[],
        default_url: None,
        identity_shape: IdentityShape::ModelAndVoice,
    },
    ProviderDefinition {
        // Deterministic replay. Makes the whole gateway testable with no
        // weights, no GPU and no network — see `docs/fidelity.md` ("Added,
        // which is not a port").
        key: "mock",
        label: "Mock Realtime",
        aliases: &[],
        default_url: None,
        identity_shape: IdentityShape::EndpointOnly,
    },
];

/// The provider registry, in catalog order.
pub fn realtime_providers() -> &'static [ProviderDefinition] {
    &REALTIME_PROVIDERS
}

/// Provider keys in catalog order.
///
/// Upstream `realtimeProviderNames()`
/// (`shared/realtime-provider-catalog.mjs:61-63`) interpolates this list into its
/// unsupported-provider message. VIA's list is longer, so that message's text
/// necessarily differs; the ordering rule (catalog order, keys only, no aliases)
/// is preserved.
pub fn realtime_provider_names() -> Vec<&'static str> {
    REALTIME_PROVIDERS.iter().map(|p| p.key).collect()
}

/// Resolve a provider key or alias to its canonical key, falling back to
/// [`DEFAULT_REALTIME_PROVIDER`] when `value` is empty.
///
/// # Errors
///
/// [`CatalogError::UnsupportedRealtimeProvider`] when nothing matches.
pub fn normalize_realtime_provider(value: &str) -> Result<&'static str, CatalogError> {
    normalize_realtime_provider_with_fallback(value, DEFAULT_REALTIME_PROVIDER)
}

/// Resolve a provider key or alias to its canonical key, with an explicit
/// fallback for an empty `value`.
///
/// Matches upstream's `normalizeRealtimeProvider`
/// (`shared/realtime-provider-catalog.mjs:65-77`): the fallback substitutes for a
/// falsy value *before* trimming and lowercasing, so a whitespace-only value is a
/// hard error rather than silently becoming the default.
///
/// # Errors
///
/// [`CatalogError::UnsupportedRealtimeProvider`] when nothing matches.
pub fn normalize_realtime_provider_with_fallback(
    value: &str,
    fallback: &str,
) -> Result<&'static str, CatalogError> {
    let raw = if value.is_empty() { fallback } else { value };
    let requested = raw.trim().to_lowercase();
    REALTIME_PROVIDERS
        .iter()
        .find(|provider| provider.matches(&requested))
        .map(|provider| provider.key)
        .ok_or(CatalogError::UnsupportedRealtimeProvider { requested })
}

/// Resolve a provider key or alias to its full definition.
///
/// # Errors
///
/// [`CatalogError::UnsupportedRealtimeProvider`] when nothing matches.
pub fn realtime_provider_definition(
    value: &str,
) -> Result<&'static ProviderDefinition, CatalogError> {
    let key = normalize_realtime_provider(value)?;
    REALTIME_PROVIDERS
        .iter()
        .find(|provider| provider.key == key)
        .ok_or(CatalogError::UnsupportedRealtimeProvider {
            requested: key.to_owned(),
        })
}

/// The DashScope endpoint for a specific Model Studio workspace.
///
/// External contract — `shared/realtime-provider-catalog.mjs:94`. Used only when
/// `DASHSCOPE_WORKSPACE_ID` is set and no explicit base URL is configured.
pub fn dashscope_workspace_realtime_url(workspace_id: &str) -> String {
    format!("wss://{workspace_id}{DASHSCOPE_WORKSPACE_REALTIME_URL_SUFFIX}")
}

/// Trim, then strip any trailing `?` characters.
///
/// External contract — `shared/realtime-provider-catalog.mjs:89-98`
/// (`withoutTrailing(value, /\?+$/)`), applied to the DashScope base URL because
/// [`realtime_url`] appends its own query separator.
pub fn normalize_dashscope_base_url(value: &str) -> &str {
    value.trim().trim_end_matches('?')
}

/// Trim, then strip any trailing `/` characters.
///
/// External contract — `shared/realtime-provider-catalog.mjs:105-110`
/// (`withoutTrailing(value, /\/+$/)`), applied to the speech-to-speech base URL.
pub fn normalize_speech_to_speech_base_url(value: &str) -> &str {
    value.trim().trim_end_matches('/')
}

/// Append the model to a realtime base URL as a query parameter.
///
/// External contract — `server/src/core/config.mjs:509-512`:
/// `` `${base}${base.includes('?') ? '&' : '?'}model=${encodeURIComponent(model)}` ``.
pub fn realtime_url(base: &str, model: &str) -> String {
    let separator = if base.contains('?') { '&' } else { '?' };
    format!("{base}{separator}model={}", encode_uri_component(model))
}

/// `encodeURIComponent` semantics: percent-encode every byte outside
/// `A-Z a-z 0-9 - _ . ! ~ * ' ( )`, using uppercase hex over the UTF-8 encoding.
///
/// Reproduced rather than delegated because the exact unreserved set is what
/// makes [`realtime_url`] byte-identical to upstream's output.
fn encode_uri_component(value: &str) -> String {
    const UNRESERVED: &[u8] = b"-_.!~*'()";
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        if byte.is_ascii_alphanumeric() || UNRESERVED.contains(byte) {
            out.push(*byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}
