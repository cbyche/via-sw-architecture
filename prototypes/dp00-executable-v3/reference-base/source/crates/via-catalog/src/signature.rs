//! The realtime configuration signature.
//!
//! Upstream computes `sha256(JSON.stringify(identity))` over an object literal
//! and publishes the lowercase hex digest on `/api/health` as
//! `realtimeConfigurationSignature`
//! (`shared/realtime-provider-catalog.mjs:122-137`,
//! `server/src/app/gateway-application.mjs:247`). Clients — the CLI's runtime
//! check and the desktop host's health poll — compare that digest against the one
//! they computed from the same configuration to decide whether the gateway they
//! are talking to is the one they configured.
//!
//! Two properties of the input make this fragile in a way a naive port loses.
//!
//! # 1. Key order is JS insertion order, not alphabetical
//!
//! `JSON.stringify` emits an object's own string keys in insertion order, so the
//! bytes that are hashed are
//!
//! ```text
//! {"provider":…,"endpoint":…,"model":…,"voice":…,"credential":…}
//! ```
//!
//! Sorting those keys — which is what a `HashMap` or a `BTreeMap`-backed
//! `serde_json::Map` does — produces
//! `{"credential":…,"endpoint":…,"model":…,"provider":…,"voice":…}` and a
//! completely different digest. Nothing would fail loudly: the gateway would
//! publish a signature no client could ever reproduce, and every client would
//! report the voice configuration as permanently changed.
//!
//! This crate builds the object with [`serde_json::Map`], which the workspace
//! configures with `preserve_order`, and inserts the keys in the upstream order.
//! `tests/signature.rs` asserts both the digests and the feature itself.
//!
//! # 2. The two branches have different shapes
//!
//! The dashscope branch carries five keys; the speech-to-speech branch carries
//! three — `model` and `voice` are **absent**, not `null`. A single struct with
//! `Option` fields would serialize `"model":null,"voice":null` and change the
//! hash. [`RealtimeIdentity`] is therefore an enum whose variants are the two
//! shapes, so the wrong shape is not constructible.

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::realtime_provider::IdentityShape;

/// The object the configuration signature is computed over.
///
/// External contract — `shared/realtime-provider-catalog.mjs:122-134`. Each
/// variant's field order is the key order that goes into the hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RealtimeIdentity {
    /// The dashscope-shaped branch: `provider`, `endpoint`, `model`, `voice`,
    /// `credential`, in that order.
    ///
    /// VIA's `openai` and `local-omni` providers use it too — they are
    /// model-and-voice providers, and reusing the shape keeps one code path.
    ModelAndVoice {
        /// Canonical provider key.
        provider: String,
        /// Resolved endpoint URL.
        endpoint: String,
        /// Resolved model id.
        model: String,
        /// Resolved voice id, or the empty string when no override applies.
        voice: String,
        /// The credential itself, as upstream hashes it. Never published — only
        /// the digest is.
        credential: String,
    },
    /// The speech-to-speech-shaped branch: `provider`, `endpoint`, `credential`.
    ///
    /// `model` and `voice` are absent from the hashed object, not null.
    EndpointOnly {
        /// Canonical provider key.
        provider: String,
        /// Resolved endpoint URL.
        endpoint: String,
        /// The credential itself.
        credential: String,
    },
}

impl RealtimeIdentity {
    /// Build the identity for a provider, choosing the branch from its declared
    /// [`IdentityShape`].
    ///
    /// `model` and `voice` are ignored for [`IdentityShape::EndpointOnly`], which
    /// is exactly what upstream's branch does — it simply never reads them.
    pub fn new(
        shape: IdentityShape,
        provider: impl Into<String>,
        endpoint: impl Into<String>,
        model: impl Into<String>,
        voice: impl Into<String>,
        credential: impl Into<String>,
    ) -> Self {
        match shape {
            IdentityShape::ModelAndVoice => Self::ModelAndVoice {
                provider: provider.into(),
                endpoint: endpoint.into(),
                model: model.into(),
                voice: voice.into(),
                credential: credential.into(),
            },
            IdentityShape::EndpointOnly => Self::EndpointOnly {
                provider: provider.into(),
                endpoint: endpoint.into(),
                credential: credential.into(),
            },
        }
    }

    /// The identity as a JSON object with the upstream key order.
    ///
    /// Built by explicit insertion rather than `#[derive(Serialize)]` so the
    /// order is visible at the point it matters. Requires `serde_json`'s
    /// `preserve_order` feature, which the workspace enables.
    pub fn to_json(&self) -> Value {
        let mut map = Map::new();
        match self {
            Self::ModelAndVoice {
                provider,
                endpoint,
                model,
                voice,
                credential,
            } => {
                map.insert("provider".into(), Value::String(provider.clone()));
                map.insert("endpoint".into(), Value::String(endpoint.clone()));
                map.insert("model".into(), Value::String(model.clone()));
                map.insert("voice".into(), Value::String(voice.clone()));
                map.insert("credential".into(), Value::String(credential.clone()));
            }
            Self::EndpointOnly {
                provider,
                endpoint,
                credential,
            } => {
                map.insert("provider".into(), Value::String(provider.clone()));
                map.insert("endpoint".into(), Value::String(endpoint.clone()));
                map.insert("credential".into(), Value::String(credential.clone()));
            }
        }
        Value::Object(map)
    }

    /// The exact bytes upstream's `JSON.stringify(identity)` produces.
    ///
    /// `serde_json::to_string` and `JSON.stringify` agree on everything reachable
    /// here: no whitespace, `/` left unescaped, non-ASCII emitted as raw UTF-8,
    /// and the same `\u00XX` escapes for control characters.
    pub fn canonical_json(&self) -> String {
        // `Value` serialization is infallible for a map of strings — there is no
        // non-finite float, no non-string key and no failing custom `Serialize`
        // in this value — so the empty-string arm is unreachable in practice and
        // is preferred to an `expect()`.
        serde_json::to_string(&self.to_json()).unwrap_or_default()
    }

    /// The lowercase hex SHA-256 of [`Self::canonical_json`].
    ///
    /// External contract — this is `/api/health.realtimeConfigurationSignature`.
    pub fn signature(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.canonical_json().as_bytes());
        hex::encode(hasher.finalize())
    }
}
