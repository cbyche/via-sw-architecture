//! Realtime model profiles.
//!
//! Ported from `shared/realtime-model-catalog.mjs`. Every model id, voice id,
//! label and turn-detection type string in this module is an **external
//! contract**: the ids and voices go on the wire to DashScope, and the whole
//! profile is echoed to clients as `/api/health.realtimeModelCatalog`. All of
//! them are KEEP under `docs/rebrand.md` — they name the vendor's models, not
//! VIA.
//!
//! # The one bug that is not ported
//!
//! Upstream's `resolveDashScopeRealtimeModelProfile()` answers an unrecognised id
//! with a profile whose capability flags are all `false`, and
//! `server/src/voice/providers/dashscope.mjs:84-87` writes
//!
//! ```js
//! session.turn_detection = profile.transportCapabilities.audioInput
//!   ? profile.sessionDefaults.turnDetection
//!   : null
//! ```
//!
//! so an unknown id produces a session with no `input_audio_format` and no turn
//! detection — it connects successfully and then never hears the user. A local
//! model id is by definition not in the DashScope table, so on the on-device path
//! that fallback is reached on the happy path.
//!
//! VIA makes two changes, per `docs/architecture.md` §7 and `docs/fidelity.md`
//! ("Corrected, not copied"):
//!
//! 1. [`resolve_dashscope_realtime_model_profile`] returns
//!    [`CatalogError::UnknownRealtimeModel`] instead of an all-false profile, so
//!    there is no such thing as an `unknown` family here.
//! 2. [`ModelFamily::Local`] exists and carries **real** capability flags — most
//!    importantly `audio_input: true` — so the on-device providers get turn
//!    detection instead of silence.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::error::CatalogError;

/// Default DashScope realtime model.
///
/// External contract — `shared/realtime-model-catalog.mjs:1`. Sent to DashScope
/// as the session model when no override is configured and echoed in
/// `/api/health` as `realtimeModel`.
pub const DEFAULT_DASHSCOPE_REALTIME_MODEL: &str = "qwen-audio-3.0-realtime-plus";

/// Default voice for the `audio` model family.
///
/// External contract — `shared/realtime-model-catalog.mjs:2`. A DashScope-owned
/// voice id; the service rejects unknown values.
pub const DEFAULT_DASHSCOPE_REALTIME_VOICE: &str = "longanqian";

/// Default voice for the `omni` model family.
///
/// External contract — `shared/realtime-model-catalog.mjs:33`. A DashScope-owned
/// voice id.
pub const DASHSCOPE_OMNI_REALTIME_VOICE: &str = "Ethan";

/// The Audio 3.0 Flash model id.
///
/// External contract — `shared/realtime-model-catalog.mjs:4`.
pub const DASHSCOPE_AUDIO_FLASH_REALTIME_MODEL: &str = "qwen-audio-3.0-realtime-flash";

/// The Omni Flash model id.
///
/// External contract — `shared/realtime-model-catalog.mjs:5`.
pub const DASHSCOPE_OMNI_FLASH_REALTIME_MODEL: &str = "qwen3.5-omni-flash-realtime";

/// The Omni Plus model id.
///
/// External contract — `shared/realtime-model-catalog.mjs:6`.
pub const DASHSCOPE_OMNI_PLUS_REALTIME_MODEL: &str = "qwen3.5-omni-plus-realtime";

/// Default voice for the `local` family.
///
/// **Not an upstream contract.** Upstream has no local family. This is Kokoro-82M's
/// stock voice id, used as the catalog default so a local profile has a concrete
/// value; `via-realtime-local` resolves the voice pack that is actually installed
/// and overrides it. Configurable through [`LOCAL_FAMILY_VOICE_ENV`].
pub const DEFAULT_LOCAL_REALTIME_VOICE: &str = "af_heart";

/// Environment variable carrying the voice override for the `audio` family.
///
/// Upstream `QWEN_AUDIO_REALTIME_VOICE`, renamed per `docs/rebrand.md`.
pub const AUDIO_FAMILY_VOICE_ENV: &str = "VIA_REALTIME_VOICE";

/// Environment variable carrying the voice override for the `omni` family.
///
/// Upstream `QWEN_OMNI_REALTIME_VOICE`, renamed per `docs/rebrand.md`. The
/// rebrand table calls out that the Audio/Omni family distinction must be
/// preserved through the rename.
pub const OMNI_FAMILY_VOICE_ENV: &str = "VIA_OMNI_REALTIME_VOICE";

/// Environment variable carrying the voice override for the `local` family.
///
/// **New.** Upstream has no local family, so this variable has no upstream peer;
/// it follows the same family-scoped naming so switching families never clobbers
/// another family's stored preference.
pub const LOCAL_FAMILY_VOICE_ENV: &str = "VIA_LOCAL_REALTIME_VOICE";

/// Which family of realtime model a profile belongs to.
///
/// The wire form is the lowercase variant name; it is published in
/// `/api/health.realtimeModelProfile.family` and read back by clients.
///
/// Upstream has a fourth value, `"unknown"`, produced by its unknown-id fallback.
/// VIA has no such variant by design — see the module docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelFamily {
    /// Qwen3.5 Omni realtime models (`Ethan` / `semantic_vad`).
    Omni,
    /// Qwen Audio 3.0 realtime models (`longanqian` / `smart_turn`).
    Audio,
    /// VIA's on-device providers. Not an upstream family.
    Local,
}

impl ModelFamily {
    /// The wire string for this family.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Omni => "omni",
            Self::Audio => "audio",
            Self::Local => "local",
        }
    }

    /// The environment variable that overrides this family's voice.
    ///
    /// Upstream's `resolveDashScopeRealtimeVoiceOverride()`
    /// (`shared/realtime-provider-catalog.mjs:51-59`) hard-codes this mapping and
    /// returns `''` for any other family. Reading the environment is deliberately
    /// *not* done here — `via-catalog` is a pure table — but the family-scoping
    /// rule lives here so no caller has to restate it.
    pub const fn voice_override_env(self) -> &'static str {
        match self {
            Self::Omni => OMNI_FAMILY_VOICE_ENV,
            Self::Audio => AUDIO_FAMILY_VOICE_ENV,
            Self::Local => LOCAL_FAMILY_VOICE_ENV,
        }
    }
}

/// The turn-detection mode a family asks the provider for.
///
/// The wire form is the snake_case variant name and is sent verbatim as
/// `session.turn_detection.type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnDetectionKind {
    /// DashScope's omni-family mode. External contract —
    /// `shared/realtime-model-catalog.mjs:34`.
    SemanticVad,
    /// DashScope's audio-family mode. External contract —
    /// `shared/realtime-model-catalog.mjs:38`.
    SmartTurn,
    /// The OpenAI-Realtime dialect's server-side VAD.
    ///
    /// **Not an upstream contract.** VIA's `local` family uses it because the
    /// on-device pipeline runs its own Silero VAD and `local-omni:endpoint`
    /// speaks the OpenAI-Realtime dialect, where `server_vad` is the existing
    /// name for exactly that arrangement.
    ServerVad,
}

/// The `turn_detection` object.
///
/// Exactly one field. External contract, test-locked upstream at
/// `server/test/realtime-provider.test.mjs:353-380`:
/// `assert.equal(session.turn_detection.threshold, undefined)` — no `threshold`
/// and no `silence_duration_ms` is ever sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TurnDetection {
    /// The mode. Serialized as `type`.
    #[serde(rename = "type")]
    pub kind: TurnDetectionKind,
}

impl TurnDetection {
    /// Construct a turn-detection object.
    pub const fn new(kind: TurnDetectionKind) -> Self {
        Self { kind }
    }
}

/// What a family asks for in `session.update` before any user override.
///
/// External contract — `shared/realtime-model-catalog.mjs:32-39`. Key order
/// (`voice`, then `turnDetection`) matches the upstream object literal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDefaults {
    /// The voice id. DashScope-owned for the `omni` and `audio` families.
    pub voice: Cow<'static, str>,
    /// The turn-detection object.
    ///
    /// Upstream models the unknown family as `turnDetection: null`; VIA has no
    /// unknown family, so this is never absent.
    pub turn_detection: TurnDetection,
}

/// What the *model* can consume and produce.
///
/// External contract — `shared/realtime-model-catalog.mjs:8-11`. The seven field
/// names and their order are published as
/// `/api/health.realtimeModelProfile.modelCapabilities` and validated flag by
/// flag by `server/src/voice/providers/provider-registry.mjs:36-52`, which throws
/// if any listed flag is not a boolean.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCapabilities {
    /// Accepts text input.
    pub text_input: bool,
    /// Accepts audio input.
    pub audio_input: bool,
    /// Accepts image input.
    pub image_input: bool,
    /// Accepts video input.
    pub video_input: bool,
    /// Produces text output.
    pub text_output: bool,
    /// Produces audio output.
    pub audio_output: bool,
    /// Authors tool calls.
    pub function_calling: bool,
}

/// What the *transport* to that model can carry.
///
/// External contract — `shared/realtime-model-catalog.mjs:12-15`. Five fields, in
/// this order.
///
/// `audio_input` is the load-bearing one: it is what upstream gates
/// `session.turn_detection` on, which is why an all-false profile is a silent
/// deafness bug rather than a cosmetic one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportCapabilities {
    /// The transport can carry text input.
    pub text_input: bool,
    /// The transport can carry audio input.
    pub audio_input: bool,
    /// The transport can carry image input.
    pub image_input: bool,
    /// The transport can carry host observations.
    pub observation_input: bool,
    /// The transport can carry native video input.
    pub native_video_input: bool,
}

/// One entry in the realtime model catalog.
///
/// Field order is upstream's object-literal order
/// (`shared/realtime-model-catalog.mjs:46-53`) and is observable: the whole
/// profile is embedded in `/api/health` as `realtimeModelProfile` and
/// `realtimeModelCatalog`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelProfile {
    /// The vendor model id, sent on the wire.
    pub id: Cow<'static, str>,
    /// The human label shown in a model picker.
    pub label: Cow<'static, str>,
    /// Which family this model belongs to.
    pub family: ModelFamily,
    /// Voice and turn detection asked for at session open.
    pub session_defaults: SessionDefaults,
    /// What the model can consume and produce.
    pub model_capabilities: ModelCapabilities,
    /// What the transport to it can carry.
    pub transport_capabilities: TransportCapabilities,
}

/// Model capabilities shared by both Omni profiles.
///
/// External contract — `shared/realtime-model-catalog.mjs:8-11`.
const OMNI_MODEL_CAPABILITIES: ModelCapabilities = ModelCapabilities {
    text_input: true,
    audio_input: true,
    image_input: true,
    video_input: false,
    text_output: true,
    audio_output: true,
    function_calling: true,
};

/// Transport capabilities shared by both Omni profiles.
///
/// External contract — `shared/realtime-model-catalog.mjs:12-15`.
const OMNI_TRANSPORT_CAPABILITIES: TransportCapabilities = TransportCapabilities {
    text_input: true,
    audio_input: true,
    image_input: false,
    observation_input: false,
    native_video_input: false,
};

/// Model capabilities shared by both Audio 3.0 profiles.
///
/// External contract — `shared/realtime-model-catalog.mjs:16-19`
/// (`LEGACY_MODEL_CAPABILITIES`). Identical to the Omni set except
/// `image_input`.
const AUDIO_MODEL_CAPABILITIES: ModelCapabilities = ModelCapabilities {
    text_input: true,
    audio_input: true,
    image_input: false,
    video_input: false,
    text_output: true,
    audio_output: true,
    function_calling: true,
};

/// Transport capabilities shared by both Audio 3.0 profiles.
///
/// External contract — `shared/realtime-model-catalog.mjs:20-23`
/// (`LEGACY_TRANSPORT_CAPABILITIES`).
const AUDIO_TRANSPORT_CAPABILITIES: TransportCapabilities = TransportCapabilities {
    text_input: true,
    audio_input: true,
    image_input: false,
    observation_input: false,
    native_video_input: false,
};

/// Model capabilities for VIA's on-device providers.
///
/// **Not an upstream contract** — this is the replacement for upstream's
/// all-false unknown-id profile. The shipping local path
/// (`local-omni:pipeline`: sherpa-onnx VAD → streaming ASR → Kokoro/Piper TTS,
/// with `llama-cpp-2` on a Qwen3 GGUF for the reasoning turn) really does take
/// text and audio in, really does emit text and audio, and really does author
/// tool calls. Images are out of scope for the componentized pipeline.
const LOCAL_MODEL_CAPABILITIES: ModelCapabilities = ModelCapabilities {
    text_input: true,
    audio_input: true,
    image_input: false,
    video_input: false,
    text_output: true,
    audio_output: true,
    function_calling: true,
};

/// Transport capabilities for VIA's on-device providers.
///
/// **Not an upstream contract.** `audio_input: true` is the whole point: it is
/// the flag upstream gates `session.turn_detection` on.
const LOCAL_TRANSPORT_CAPABILITIES: TransportCapabilities = TransportCapabilities {
    text_input: true,
    audio_input: true,
    image_input: false,
    observation_input: false,
    native_video_input: false,
};

/// Omni-family session defaults.
///
/// External contract — `shared/realtime-model-catalog.mjs:32-35`.
const OMNI_SESSION_DEFAULTS: SessionDefaults = SessionDefaults {
    voice: Cow::Borrowed(DASHSCOPE_OMNI_REALTIME_VOICE),
    turn_detection: TurnDetection::new(TurnDetectionKind::SemanticVad),
};

/// Audio-family session defaults.
///
/// External contract — `shared/realtime-model-catalog.mjs:36-39`.
const AUDIO_SESSION_DEFAULTS: SessionDefaults = SessionDefaults {
    voice: Cow::Borrowed(DEFAULT_DASHSCOPE_REALTIME_VOICE),
    turn_detection: TurnDetection::new(TurnDetectionKind::SmartTurn),
};

/// Local-family session defaults. Not an upstream contract.
const LOCAL_SESSION_DEFAULTS: SessionDefaults = SessionDefaults {
    voice: Cow::Borrowed(DEFAULT_LOCAL_REALTIME_VOICE),
    turn_detection: TurnDetection::new(TurnDetectionKind::ServerVad),
};

/// The four DashScope profiles, in catalog order.
///
/// External contract — `shared/realtime-model-catalog.mjs:45-78`. The *order* is
/// itself contract: it is deep-equality asserted by
/// `test/realtime-provider-catalog.test.mjs:44-107`, echoed in
/// `/api/health.realtimeModelCatalog`, and printed by `via config` as the list of
/// available models.
static DASHSCOPE_REALTIME_MODEL_PROFILES: [ModelProfile; 4] = [
    ModelProfile {
        id: Cow::Borrowed(DASHSCOPE_OMNI_FLASH_REALTIME_MODEL),
        label: Cow::Borrowed("Qwen3.5 Omni Flash Realtime"),
        family: ModelFamily::Omni,
        session_defaults: OMNI_SESSION_DEFAULTS,
        model_capabilities: OMNI_MODEL_CAPABILITIES,
        transport_capabilities: OMNI_TRANSPORT_CAPABILITIES,
    },
    ModelProfile {
        id: Cow::Borrowed(DASHSCOPE_OMNI_PLUS_REALTIME_MODEL),
        label: Cow::Borrowed("Qwen3.5 Omni Plus Realtime"),
        family: ModelFamily::Omni,
        session_defaults: OMNI_SESSION_DEFAULTS,
        model_capabilities: OMNI_MODEL_CAPABILITIES,
        transport_capabilities: OMNI_TRANSPORT_CAPABILITIES,
    },
    ModelProfile {
        id: Cow::Borrowed(DEFAULT_DASHSCOPE_REALTIME_MODEL),
        label: Cow::Borrowed("Qwen Audio 3.0 Realtime Plus"),
        family: ModelFamily::Audio,
        session_defaults: AUDIO_SESSION_DEFAULTS,
        model_capabilities: AUDIO_MODEL_CAPABILITIES,
        transport_capabilities: AUDIO_TRANSPORT_CAPABILITIES,
    },
    ModelProfile {
        id: Cow::Borrowed(DASHSCOPE_AUDIO_FLASH_REALTIME_MODEL),
        label: Cow::Borrowed("Qwen Audio 3.0 Realtime Flash"),
        family: ModelFamily::Audio,
        session_defaults: AUDIO_SESSION_DEFAULTS,
        model_capabilities: AUDIO_MODEL_CAPABILITIES,
        transport_capabilities: AUDIO_TRANSPORT_CAPABILITIES,
    },
];

/// The DashScope realtime model catalog, in catalog order.
///
/// Upstream `listDashScopeRealtimeModelProfiles()`
/// (`shared/realtime-model-catalog.mjs:84-86`).
pub fn dashscope_realtime_model_profiles() -> &'static [ModelProfile] {
    &DASHSCOPE_REALTIME_MODEL_PROFILES
}

/// Resolve a DashScope realtime model id to its profile.
///
/// An empty or whitespace-only id resolves to
/// [`DEFAULT_DASHSCOPE_REALTIME_MODEL`], matching upstream
/// (`shared/realtime-model-catalog.mjs:88-99`).
///
/// # Errors
///
/// Returns [`CatalogError::UnknownRealtimeModel`] for any id not in the table.
/// **This is the deliberate divergence from upstream**, which returns an
/// all-capabilities-false profile — see the module docs and
/// `docs/architecture.md` §7. Capabilities are never inferred from the shape of
/// the id: `qwen3.5-omni-plus-realtime-future` is an error, not an omni profile.
pub fn resolve_dashscope_realtime_model_profile(
    model: &str,
) -> Result<&'static ModelProfile, CatalogError> {
    let trimmed = model.trim();
    let id = if trimmed.is_empty() {
        DEFAULT_DASHSCOPE_REALTIME_MODEL
    } else {
        trimmed
    };
    DASHSCOPE_REALTIME_MODEL_PROFILES
        .iter()
        .find(|profile| profile.id == id)
        .ok_or_else(|| CatalogError::UnknownRealtimeModel {
            model: id.to_owned(),
        })
    // No `unwrap_or_else(all_false_profile)`. That fallback is the bug.
}

/// The default DashScope profile.
pub fn default_dashscope_realtime_model_profile() -> &'static ModelProfile {
    // The default id is one of the four literals in the table above, so the
    // lookup cannot fail; falling back to index 2 rather than panicking keeps
    // this function total without an `expect()`.
    resolve_dashscope_realtime_model_profile(DEFAULT_DASHSCOPE_REALTIME_MODEL)
        .unwrap_or(&DASHSCOPE_REALTIME_MODEL_PROFILES[2])
}

/// Build the `local`-family profile for an on-device model id.
///
/// **Not a port.** Local model ids are user-chosen GGUF/ONNX coordinates, so
/// there is no fixed table to look them up in — but they must still get real
/// capability flags rather than upstream's all-false fallback. The label mirrors
/// the id, which is what upstream did for ids it did not recognise.
pub fn local_realtime_model_profile(model: &str) -> ModelProfile {
    let id = model.trim().to_owned();
    ModelProfile {
        label: Cow::Owned(id.clone()),
        id: Cow::Owned(id),
        family: ModelFamily::Local,
        session_defaults: LOCAL_SESSION_DEFAULTS,
        model_capabilities: LOCAL_MODEL_CAPABILITIES,
        transport_capabilities: LOCAL_TRANSPORT_CAPABILITIES,
    }
}
