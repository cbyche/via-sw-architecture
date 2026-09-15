//! The DashScope realtime provider.
//!
//! A whole port of `server/src/voice/providers/dashscope.mjs` (133 lines), which
//! is the provider qwen's realtime gateway was written against — cloud Qwen
//! Audio 3.0 and Qwen3.5 Omni, over the beta OpenAI Realtime envelope.
//!
//! # ⚠ The bug that is not ported
//!
//! Upstream writes (`dashscope.mjs:84-89`):
//!
//! ```js
//! if (profile.transportCapabilities.audioInput) {
//!   session.input_audio_format = 'pcm'
//! }
//! session.turn_detection = profile.transportCapabilities.audioInput
//!   ? profile.sessionDefaults.turnDetection
//!   : null
//! ```
//!
//! With any of the four catalogued profiles that gate *is* the contract — all
//! four declare `audioInput: true`. What makes it dangerous is what sits under
//! it: `resolveDashScopeRealtimeModelProfile()` answers an unrecognised id with
//! an **all-capabilities-false** profile, and this code turns that profile into
//! a session with no input format and no turn detection — one that opens
//! successfully and then never hears the user. No error, no log line, no failed
//! connection; just a microphone that goes nowhere.
//!
//! Upstream's only protection against that is a *separate* check, in a
//! different file — `realtime-provider.mjs:134-140` refuses to connect when
//! `modelProfile.family === 'unknown'`. It works, and it is one forgotten line
//! away from not working, because the fallback that makes it necessary lives
//! nowhere near it. `docs/architecture.md` §7 records why that matters for VIA
//! specifically: **a local model id is by definition not in the DashScope
//! table**, so on the on-device path the all-false fallback is reached on the
//! happy path.
//!
//! VIA keeps the gate and removes the fallback:
//!
//! 1. [`via_catalog::resolve_dashscope_realtime_model_profile`] answers an
//!    unrecognised id with `Err`. There is no `unknown` family in VIA at all,
//!    which is also why upstream's check cannot be transcribed as written.
//! 2. [`DashScopeProvider::preflight`] is that check, re-expressed where the
//!    knowledge lives: it turns the `Err` into
//!    [`RealtimeError::UnsupportedModel`] **naming the id**, and `via-realtime`
//!    runs `preflight()` before `is_configured()` and both before the socket.
//!
//! The two together are what make the deaf payload unreachable rather than
//! merely unlikely: [`profile`](DashScopeProvider::profile) returns `None` for
//! exactly the ids `preflight` refuses, so no session that ever opens can be
//! built from one.
//! `tests/dashscope.rs::an_unknown_model_is_refused_before_a_socket_is_opened`
//! asserts the transport saw no frame at all.
//!
//! # What this file does not decide
//!
//! - the four model profiles, their voices and their turn-detection modes —
//!   [`via_catalog`];
//! - the provider key, its `qwen` alias and the endpoint defaults —
//!   [`via_catalog`];
//! - the configuration-signature hash input and its key order —
//!   [`via_catalog::RealtimeIdentity`];
//! - the wire envelope — [`via_realtime::openai_compatible_protocol`];
//! - any sentence a person or the model reads — [`via_i18n`], through
//!   [`crate::prompt`].

use serde_json::{Map, Value};
use via_audio::SampleRate;
use via_catalog::realtime_provider::{realtime_provider_definition, realtime_url};
use via_catalog::{
    IdentityShape, ModelFamily, ModelProfile, RealtimeIdentity, dashscope_realtime_model_profiles,
    resolve_dashscope_realtime_model_profile,
};
use via_i18n::{Locale, keys, t};
use via_realtime::{
    ErrorClass, Injection, OpenAiCompatibleProtocol, PermissionRequest, ProviderCapabilities,
    RealtimeError, RealtimeProtocol, RealtimeProvider, SessionRequest, openai_compatible_protocol,
};

use crate::classify::classify_dashscope_error;
use crate::prompt::{
    permission_request_text, permission_response_instructions, result_response_instructions,
    speak_response_instructions,
};
use crate::settings::DashScopeSettings;

/// The canonical provider key.
///
/// External contract — `dashscope.mjs:44`, catalogued as
/// `built-in realtime providers`. A client sends it on the `connect` frame and
/// `/api/health` echoes it. **KEEP** under `docs/rebrand.md`: it names Alibaba's
/// service, not VIA.
pub const KEY: &str = "dashscope";

/// The human label.
///
/// External contract — `dashscope.mjs:45`. **KEEP**: it is the display name of
/// the Qwen realtime service this provider dials, and `docs/rebrand.md` carves
/// out the frontend voice engine's qwen runtime by name.
///
/// It deliberately differs from
/// [`via_catalog`]'s row label (`"DashScope"`): upstream carries two labels for
/// this provider — the *configuration* label in
/// `shared/realtime-provider-catalog.mjs:25` and the *voice provider* label
/// here — and `describeActiveRealtime` publishes this one.
pub const LABEL: &str = "Qwen-Audio-Realtime";

/// The alternate spelling that resolves to [`KEY`].
///
/// External contract — `dashscope.mjs:46`. **KEEP**, and the rebrand survey says
/// why twice: it names the vendor's realtime runtime rather than VIA, and users
/// already have `provider=qwen` in `config.env`.
pub const ALIAS: &str = "qwen";

/// The rate a client must capture at, in hertz.
///
/// External contract — `dashscope.mjs:47`, catalogued as `audio sample rates`.
/// Reaches every client on `voice.ready`; the wake-word extractor and the
/// resampler both key off it. The value is [`SampleRate::HZ_16000`]'s.
pub const INPUT_SAMPLE_RATE: SampleRate = SampleRate::HZ_16000;

/// The rate this provider's audio output arrives at, in hertz.
///
/// External contract — `dashscope.mjs:48`. The value is
/// [`SampleRate::HZ_24000`]'s.
pub const OUTPUT_SAMPLE_RATE: SampleRate = SampleRate::HZ_24000;

/// The audio format negotiated in both directions.
///
/// External contract — `dashscope.mjs:82,85`, catalogued as
/// `DashScope session.update payload (first, unconfigured)`. Bare `'pcm'`, not
/// `'pcm16'` and not an object: the beta dialect names the format this way.
pub const AUDIO_FORMAT: &str = "pcm";

/// Cloud Qwen realtime, behind [`RealtimeProvider`].
#[derive(Clone, Debug)]
pub struct DashScopeProvider {
    settings: DashScopeSettings,
    identity_shape: IdentityShape,
    protocol: OpenAiCompatibleProtocol,
}

impl DashScopeProvider {
    /// A provider reading `settings`.
    #[must_use]
    pub fn new(settings: DashScopeSettings) -> Self {
        Self {
            settings,
            // The shape is the catalog's, so adding a provider never has to
            // touch the signature code. `KEY` is one of the catalog's own rows,
            // so the lookup cannot miss; naming the shape it would have found
            // keeps this total without an `expect()`, the same way
            // `via_catalog::default_dashscope_realtime_model_profile` does.
            identity_shape: realtime_provider_definition(KEY)
                .map_or(IdentityShape::ModelAndVoice, |row| row.identity_shape),
            protocol: openai_compatible_protocol(),
        }
    }

    /// The settings this provider was built with.
    #[must_use]
    pub fn settings(&self) -> &DashScopeSettings {
        &self.settings
    }

    /// The profile of the configured model, or `None` when the id is not in the
    /// catalog.
    ///
    /// Upstream's `activeModelProfile()` never returns nothing — that is the
    /// bug. Here `None` is reachable only for an id
    /// [`preflight`](Self::preflight) already refuses, so nothing downstream of
    /// a successful connect ever sees it.
    #[must_use]
    pub fn profile(&self) -> Option<&'static ModelProfile> {
        resolve_dashscope_realtime_model_profile(&self.settings.model).ok()
    }

    /// The modalities every response body declares.
    ///
    /// External contract — `dashscope.mjs:35-41`:
    /// `[textOutput && 'text', audioOutput && 'audio'].filter(Boolean)`, so the
    /// order is fixed and an unresolvable profile yields `[]` exactly as
    /// upstream's all-false one did.
    #[must_use]
    pub fn response_modalities(&self) -> Value {
        let mut modalities = Vec::with_capacity(2);
        if let Some(profile) = self.profile() {
            if profile.model_capabilities.text_output {
                modalities.push(Value::String("text".to_owned()));
            }
            if profile.model_capabilities.audio_output {
                modalities.push(Value::String("audio".to_owned()));
            }
        }
        Value::Array(modalities)
    }

    /// The configured voice override, or the family default beneath it.
    ///
    /// External contract — `dashscope.mjs:61`:
    /// `config.audioVoice || profile.sessionDefaults.voice`. The override is
    /// family-scoped and already trimmed by
    /// [`via_core::config::resolve_realtime_frontend`], so an empty string means
    /// *"nothing was set"* rather than *"the empty voice"*.
    fn resolved_voice(&self) -> Option<&str> {
        if !self.settings.voice.is_empty() {
            return Some(&self.settings.voice);
        }
        self.profile()
            .map(|profile| profile.session_defaults.voice.as_ref())
    }

    /// The `response.create` body shared by both injections.
    ///
    /// External contract — `dashscope.mjs:106-110,127-131`: three keys, in this
    /// order. `tool_choice: 'none'` is what stops the model answering a result
    /// or a permission question by calling another tool.
    fn injection_response(&self, instructions: String) -> Value {
        let mut response = Map::new();
        response.insert("modalities".to_owned(), self.response_modalities());
        response.insert("tool_choice".to_owned(), Value::String("none".to_owned()));
        response.insert("instructions".to_owned(), Value::String(instructions));
        Value::Object(response)
    }

    /// `{ type: 'message', role: 'user', content: [{ type: 'input_text', text }] }`.
    ///
    /// External contract — `dashscope.mjs:101-105,115-125`. The same shape the
    /// dialect's [`user_text_item`](RealtimeProtocol::user_text_item) builds,
    /// reached through the dialect so there is one definition of it.
    fn user_item(&self, text: &str) -> Value {
        self.protocol.user_text_item(text)
    }
}

impl RealtimeProvider for DashScopeProvider {
    fn key(&self) -> &str {
        KEY
    }

    fn label(&self) -> &str {
        LABEL
    }

    fn aliases(&self) -> Vec<String> {
        vec![ALIAS.to_owned()]
    }

    fn input_sample_rate(&self) -> u32 {
        INPUT_SAMPLE_RATE.hz()
    }

    fn output_sample_rate(&self) -> u32 {
        OUTPUT_SAMPLE_RATE.hz()
    }

    fn is_configured(&self) -> bool {
        // External contract — `dashscope.mjs:62`, `Boolean(config.dashscopeApiKey)`.
        !self.settings.api_key.is_empty()
    }

    fn model(&self) -> Option<&str> {
        // `config.audioModel` — as configured, not as resolved. An id the
        // catalog does not know still has to reach `preflight` to be named in
        // its refusal, and `/api/health` reports what the operator actually set.
        Some(&self.settings.model)
    }

    fn voice(&self) -> Option<&str> {
        self.resolved_voice()
    }

    fn url(&self) -> Result<String, RealtimeError> {
        // External contract — `config.mjs:509-512`, through
        // `via_catalog::realtime_provider::realtime_url`: the model rides as a
        // query parameter, `&`-joined when the base already has one.
        Ok(realtime_url(&self.settings.base_url, &self.settings.model))
    }

    fn headers(&self) -> Vec<(String, String)> {
        // External contract — `dashscope.mjs:67`, catalogued as
        // `provider Authorization headers`: **always** sent, even when the key
        // is empty, so a missing credential fails as a 401 the `fatal`
        // classifier recognises rather than as an unauthenticated upgrade the
        // service answers some other way.
        vec![(
            "Authorization".to_owned(),
            format!("Bearer {}", self.settings.api_key.expose()),
        )]
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            // `dashscope.mjs:53`.
            per_response_instructions: true,
            // `dashscope.mjs:54` — `activeModelProfile().family !== 'omni'`.
            // The Omni models acknowledge a conversation item under an id of
            // their own choosing, so a client-assigned id must not be waited
            // on. An id the catalog does not recognise keeps the baseline,
            // which is what upstream's `'unknown' !== 'omni'` also produced.
            conversation_item_id_echo: self
                .profile()
                .is_none_or(|profile| profile.family != ModelFamily::Omni),
            ..ProviderCapabilities::DEFAULT
        }
    }

    fn model_profile(&self) -> Option<ModelProfile> {
        self.profile().cloned()
    }

    fn model_catalog(&self) -> &[ModelProfile] {
        dashscope_realtime_model_profiles()
    }

    fn configuration_signature(&self) -> String {
        RealtimeIdentity::new(
            self.identity_shape,
            KEY,
            // The base endpoint, before `?model=` — which is what
            // `via_core::config::resolve_realtime_frontend` hashes, so an
            // active DashScope frontend and this provider agree.
            self.settings.base_url.as_str(),
            self.settings.model.as_str(),
            // The raw family-scoped override, empty included. Not the resolved
            // voice: the profile default is not part of the operator's
            // configuration, so applying it would make the signature change
            // when the catalog changed rather than when the config did.
            self.settings.voice.as_str(),
            self.settings.api_key.expose(),
        )
        .signature()
    }

    fn protocol(&self) -> &dyn RealtimeProtocol {
        &self.protocol
    }

    fn preflight(&self) -> Result<(), RealtimeError> {
        // **The bug fix.** See the module docs.
        resolve_dashscope_realtime_model_profile(&self.settings.model)
            .map(|_| ())
            .map_err(|error| match error {
                via_catalog::CatalogError::UnknownRealtimeModel { model } => {
                    RealtimeError::UnsupportedModel {
                        // The *resolved* id — trimmed, and with the catalog
                        // default substituted for an empty value — because that
                        // is the id the connection would have used, and it is
                        // what upstream's message interpolated
                        // (`realtime-provider.mjs:135-138`,
                        // `this.modelProfile.id`).
                        id: model,
                        label: LABEL.to_owned(),
                    }
                }
                // `resolve_dashscope_realtime_model_profile` has exactly one
                // failure. Anything else is still an unsupported model from
                // this provider's side, and reporting it as such beats
                // inventing a class for a case the catalog cannot produce.
                _ => RealtimeError::UnsupportedModel {
                    id: self.settings.model.trim().to_owned(),
                    label: LABEL.to_owned(),
                },
            })
    }

    fn classify_error(&self, message: &str) -> ErrorClass {
        classify_dashscope_error(message)
    }

    fn build_session(&self, request: &SessionRequest<'_>) -> Value {
        // External contract — `dashscope.mjs:70-92`, catalogued twice: once for
        // the first (unconfigured) update and once for every later one. Key
        // insertion order is upstream's assignment order and goes on the wire.
        let profile = self.profile();
        let mut session = Map::new();
        session.insert(
            "instructions".to_owned(),
            Value::String(request.agent_context.instructions.clone()),
        );
        if profile.is_some_and(|profile| profile.model_capabilities.function_calling) {
            session.insert(
                "tools".to_owned(),
                Value::Array(request.agent_context.tools.clone()),
            );
        }
        if !request.configured {
            // Everything below is negotiated **once**. Re-sending it on a
            // context refresh would renegotiate the audio format mid-call and
            // reset the provider's turn detection.
            session.insert("modalities".to_owned(), self.response_modalities());
            if profile.is_some_and(|profile| profile.model_capabilities.audio_output) {
                session.insert(
                    "voice".to_owned(),
                    Value::String(self.resolved_voice().unwrap_or_default().to_owned()),
                );
                session.insert(
                    "output_audio_format".to_owned(),
                    Value::String(AUDIO_FORMAT.to_owned()),
                );
            }
            let audio_input =
                profile.is_some_and(|profile| profile.transport_capabilities.audio_input);
            if audio_input {
                session.insert(
                    "input_audio_format".to_owned(),
                    Value::String(AUDIO_FORMAT.to_owned()),
                );
            }
            // The gate upstream has, kept — and unreachable in its broken
            // direction, because `preflight()` refuses every id that would
            // reach it. See the module docs.
            let turn_detection = match profile {
                Some(profile) if audio_input => {
                    // A three-field struct of unit enums; serialization has no
                    // failing branch, and `Null` is the value upstream would
                    // have produced anyway.
                    serde_json::to_value(profile.session_defaults.turn_detection)
                        .unwrap_or(Value::Null)
                }
                _ => Value::Null,
            };
            session.insert("turn_detection".to_owned(), turn_detection);
        }
        Value::Object(session)
    }

    fn build_speak_response(&self, content: &str) -> Value {
        // External contract — `dashscope.mjs:94-98`. `conversation: 'none'`
        // keeps the utterance out of conversation history, which is why the
        // content has to be inside the instructions rather than in an item.
        let mut response = Map::new();
        response.insert("conversation".to_owned(), Value::String("none".to_owned()));
        response.insert("modalities".to_owned(), self.response_modalities());
        response.insert(
            "instructions".to_owned(),
            Value::String(speak_response_instructions(content, self.settings.locale)),
        );
        Value::Object(response)
    }

    fn build_result_injection(&self, content: &str) -> Injection {
        // External contract — `dashscope.mjs:100-111`.
        Injection {
            item: self.user_item(content),
            response: self.injection_response(result_response_instructions(self.settings.locale)),
        }
    }

    fn build_permission_injection(&self, permission: &PermissionRequest) -> Injection {
        // External contract — `dashscope.mjs:113-132`.
        Injection {
            item: self.user_item(&permission_request_text(permission)),
            response: self
                .injection_response(permission_response_instructions(self.settings.locale)),
        }
    }

    fn missing_configuration_message(&self, locale: Locale) -> String {
        // External contract — `dashscope.mjs:63`, `请先配置 DASHSCOPE_API_KEY`.
        // `DASHSCOPE_API_KEY` is **KEEP**: it is Alibaba's own variable name and
        // users already have it set from the vendor's documentation.
        t(locale, keys::GATEWAY_MISSING_DASHSCOPE_API_KEY_SHORT).to_owned()
    }

    fn connect_timeout_message(&self, locale: Locale) -> String {
        // External contract — `dashscope.mjs:64`, `连接 Qwen Audio Realtime 超时`.
        t(locale, keys::REALTIME_CONNECT_TIMEOUT_DASHSCOPE).to_owned()
    }
}

impl Default for DashScopeProvider {
    fn default() -> Self {
        Self::new(DashScopeSettings::default())
    }
}

/// Build a provider straight from a resolved configuration.
impl From<&via_core::Config> for DashScopeProvider {
    fn from(config: &via_core::Config) -> Self {
        Self::new(DashScopeSettings::from_config(config))
    }
}
