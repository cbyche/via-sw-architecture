//! The huggingface/speech-to-speech provider.
//!
//! A whole port of `server/src/voice/providers/s2s.mjs` (138 lines). Upstream's
//! own summary of what it is:
//!
//! > Thin profile for a user-managed huggingface/speech-to-speech endpoint. The
//! > upstream process owns all model, STT, TTS and voice choices; this profile
//! > only describes its OpenAI Realtime wire contract.
//!
//! That sentence is the whole design. [`model`](RealtimeProvider::model),
//! [`voice`](RealtimeProvider::voice) and
//! [`model_profile`](RealtimeProvider::model_profile) are all `None`, the model
//! catalog is empty — which `/api/health` publishes as
//! `realtimeModelIds: null`, so a client renders *no* model picker rather than
//! an empty one — and nothing here has an opinion about what the far end runs.
//!
//! # Why it ships beside DashScope
//!
//! `docs/architecture.md` §15 lists `via-realtime-s2s` as a phase-6 crate.
//! Shipping it here instead is deliberate and recorded in
//! `docs/deviations/phase-5-via-realtime-dashscope.md`: it is 138 lines, it has
//! no dependency DashScope does not already have, and the two are what
//! `server/src/voice/providers/registry.mjs` registers **together** as the
//! built-in pair. Splitting a 138-line file into its own crate to satisfy a
//! table would give the port a seam upstream does not have.
//!
//! # Four differences from DashScope that are each load-bearing
//!
//! | Difference | Why |
//! | --- | --- |
//! | the **GA** dialect | `output_modalities`, `response.output_text.*`, per-type item id namespaces |
//! | four non-default capability flags | no `session.updated`, one response slot, echoed metadata, per-response instructions |
//! | a **60 s** response-start budget | a fully local ASR → LLM → TTS pipeline is slower to first token than a cloud model |
//! | `Authorization` only when a token is set | the service is usually unauthenticated on loopback |
//!
//! # The absent `audio.input.format` is not an omission
//!
//! `s2s.mjs:80-85` explains it, and the contract is test-locked upstream
//! (`session.audio.input.format === undefined`): speech-to-speech treats an
//! omitted input format as its native 16 kHz pipeline rate, while OpenAI's GA
//! `AudioPCM` schema only accepts 24 kHz when the format is stated. Declaring
//! 16 kHz would make the **entire** `session.update` invalid. So the input
//! format is left out and only the output format is negotiated.

use serde_json::{Map, Value};
use via_audio::SampleRate;
use via_catalog::realtime_provider::realtime_provider_definition;
use via_catalog::{IdentityShape, RealtimeIdentity};
use via_i18n::{Locale, format as i18n_format, keys, t};
use via_realtime::{
    ErrorClass, GaRealtimeProtocol, Injection, PermissionRequest, ProviderCapabilities,
    RealtimeError, RealtimeProtocol, RealtimeProvider, SessionRequest, ga_realtime_protocol,
};

use crate::classify::classify_speech_to_speech_error;
use crate::prompt::{
    permission_request_text, permission_response_instructions, result_response_instructions,
    speak_response_instructions,
};
use crate::settings::SpeechToSpeechSettings;

use std::time::Duration;

/// The canonical provider key.
///
/// External contract — `s2s.mjs:31`, catalogued as
/// `built-in realtime providers`.
pub const KEY: &str = "speech-to-speech";

/// The human label.
///
/// External contract — `s2s.mjs:32`. It names the third-party
/// huggingface/speech-to-speech project, so it is **KEEP**.
pub const LABEL: &str = "Hugging Face Speech-to-Speech";

/// The alternate spelling that resolves to [`KEY`].
///
/// External contract — `s2s.mjs:33`.
pub const ALIAS: &str = "s2s";

/// The rate a client must capture at.
///
/// External contract — `s2s.mjs:11`, catalogued as `audio sample rates`.
pub const INPUT_SAMPLE_RATE: SampleRate = SampleRate::HZ_16000;

/// The rate this provider's audio output arrives at.
///
/// External contract — `s2s.mjs:12`. Also the `rate` declared in
/// `session.audio.output.format`: the upstream service resamples its internal
/// 16 kHz pipeline output to the common client playback rate.
pub const OUTPUT_SAMPLE_RATE: SampleRate = SampleRate::HZ_24000;

/// How long to wait for `response.created`.
///
/// External contract — `s2s.mjs:38`, catalogued under `RealtimeFrontend
/// timeouts`. Twice the 30 s session default, because a fully local
/// ASR → LLM → TTS pipeline can take substantially longer than a cloud model
/// before producing its first response event.
pub const RESPONSE_START_TIMEOUT: Duration = Duration::from_secs(60);

/// The GA session type discriminator.
///
/// External contract — `s2s.mjs:69`. The GA schema requires it; the beta dialect
/// has no such key.
pub const SESSION_TYPE: &str = "realtime";

/// The turn-detection mode negotiated on the input side.
///
/// External contract — `s2s.mjs:85`. Plain server-side VAD, with
/// `interrupt_response` on so the user can barge in.
pub const TURN_DETECTION_TYPE: &str = "server_vad";

/// The GA audio format discriminator.
///
/// External contract — `s2s.mjs:91`. A MIME-shaped name, unlike the beta
/// dialect's bare `'pcm'`.
pub const OUTPUT_AUDIO_FORMAT: &str = "audio/pcm";

/// The single modality every response body declares.
///
/// External contract — `s2s.mjs:78,100,110,133`. Fixed rather than derived: the
/// service has no model profile to derive it from.
pub const AUDIO_MODALITY: &str = "audio";

/// A user-run huggingface/speech-to-speech endpoint, behind [`RealtimeProvider`].
#[derive(Clone, Debug)]
pub struct SpeechToSpeechProvider {
    settings: SpeechToSpeechSettings,
    identity_shape: IdentityShape,
    protocol: GaRealtimeProtocol,
}

impl SpeechToSpeechProvider {
    /// A provider reading `settings`.
    #[must_use]
    pub fn new(settings: SpeechToSpeechSettings) -> Self {
        Self {
            settings,
            // `EndpointOnly` — the `model` and `voice` keys are **absent** from
            // the hashed object, not null. See `via_catalog::signature`.
            identity_shape: realtime_provider_definition(KEY)
                .map_or(IdentityShape::EndpointOnly, |row| row.identity_shape),
            protocol: ga_realtime_protocol(),
        }
    }

    /// The settings this provider was built with.
    #[must_use]
    pub fn settings(&self) -> &SpeechToSpeechSettings {
        &self.settings
    }

    /// `['audio']`.
    fn modalities() -> Value {
        Value::Array(vec![Value::String(AUDIO_MODALITY.to_owned())])
    }

    /// The GA tool shape.
    ///
    /// External contract — `s2s.mjs:72-77`, catalogued as
    /// `tool schema shape per dialect`: the beta dialect nests a tool under
    /// `function`, GA flattens it. The flattening lives in the **provider**, not
    /// in the dialect adapter, exactly as upstream — a tool catalog is composed
    /// once, in the beta shape, and each provider shapes it for its own wire.
    ///
    /// A tool that is not in the beta shape is passed through unchanged rather
    /// than dropped: an empty `{}` on the wire is a schema error the service
    /// reports, and silently discarding a tool is a capability that disappears
    /// with no diagnostic at all.
    fn ga_tools(tools: &[Value]) -> Value {
        Value::Array(tools.iter().map(Self::ga_tool).collect())
    }

    fn ga_tool(tool: &Value) -> Value {
        let Some(function) = tool.get("function").and_then(Value::as_object) else {
            return tool.clone();
        };
        let mut flattened = Map::new();
        flattened.insert("type".to_owned(), Value::String("function".to_owned()));
        for field in ["name", "description", "parameters"] {
            flattened.insert(
                field.to_owned(),
                function.get(field).cloned().unwrap_or(Value::Null),
            );
        }
        Value::Object(flattened)
    }

    /// The `response.create` body shared by both injections.
    ///
    /// External contract — `s2s.mjs:111-115,132-136`.
    fn injection_response(instructions: String) -> Value {
        let mut response = Map::new();
        response.insert("modalities".to_owned(), Self::modalities());
        response.insert("tool_choice".to_owned(), Value::String("none".to_owned()));
        response.insert("instructions".to_owned(), Value::String(instructions));
        Value::Object(response)
    }
}

impl RealtimeProvider for SpeechToSpeechProvider {
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

    fn response_start_timeout(&self) -> Option<Duration> {
        Some(RESPONSE_START_TIMEOUT)
    }

    fn is_configured(&self) -> bool {
        // External contract — `s2s.mjs:57`. The predicate itself lives in
        // `via_core::config::resolve_realtime_frontend`, because *"do not
        // advertise a local service merely because a default endpoint
        // exists"* is a statement about the environment, not about the socket.
        self.settings.configured
    }

    fn model(&self) -> Option<&str> {
        // `s2s.mjs:55` — the far end owns the model, and saying otherwise would
        // put a model id in `/api/health` that nothing chose.
        None
    }

    fn voice(&self) -> Option<&str> {
        // `s2s.mjs:56` — likewise for the voice.
        None
    }

    fn url(&self) -> Result<String, RealtimeError> {
        // External contract — `s2s.mjs:61`. No query parameter: there is no
        // model to name.
        Ok(self.settings.url.clone())
    }

    fn headers(&self) -> Vec<(String, String)> {
        // External contract — `s2s.mjs:62-64`, catalogued as
        // `provider Authorization headers`: a bearer token **only** when one is
        // configured, because the service is usually unauthenticated on
        // loopback and an empty `Bearer ` is a header some proxies reject.
        if self.settings.auth_token.is_empty() {
            return Vec::new();
        }
        vec![(
            "Authorization".to_owned(),
            format!("Bearer {}", self.settings.auth_token.expose()),
        )]
    }

    fn capabilities(&self) -> ProviderCapabilities {
        // External contract — `s2s.mjs:41-53`. Each flag is one upstream
        // comment, kept verbatim in meaning.
        ProviderCapabilities {
            // Applies `session.update` without sending `session.updated`, so
            // the session becomes ready the moment the update is written.
            acknowledges_session_update: false,
            // One response slot per session: a gateway `response.create` can
            // race a server-VAD turn and gets refused instead of queued.
            single_response_slot: true,
            // Echoes response metadata, so a gateway-created response is told
            // apart from an automatic one without relying on arrival order.
            response_metadata_correlation: true,
            // Supports transient instructions on an individual `response.create`.
            per_response_instructions: true,
            ..ProviderCapabilities::DEFAULT
        }
    }

    fn configuration_signature(&self) -> String {
        RealtimeIdentity::new(
            self.identity_shape,
            KEY,
            self.settings.url.as_str(),
            // Ignored by `EndpointOnly`, which is exactly what upstream's
            // branch does — it never reads them.
            "",
            "",
            self.settings.auth_token.expose(),
        )
        .signature()
    }

    fn protocol(&self) -> &dyn RealtimeProtocol {
        &self.protocol
    }

    fn classify_error(&self, message: &str) -> ErrorClass {
        classify_speech_to_speech_error(message)
    }

    fn build_session(&self, request: &SessionRequest<'_>) -> Value {
        // External contract — `s2s.mjs:67-96`, catalogued as
        // `s2s session.update payload (GA dialect)`. Key insertion order is
        // upstream's object-literal order.
        //
        // Note what this does **not** branch on: `configured`. Unlike
        // DashScope, every update carries the whole payload, because the
        // service applies it silently and re-sending it is how a context
        // refresh is acknowledged at all.
        let mut session = Map::new();
        session.insert("type".to_owned(), Value::String(SESSION_TYPE.to_owned()));
        session.insert(
            "instructions".to_owned(),
            Value::String(request.agent_context.instructions.clone()),
        );
        session.insert(
            "tools".to_owned(),
            Self::ga_tools(&request.agent_context.tools),
        );
        session.insert("output_modalities".to_owned(), Self::modalities());

        let mut turn_detection = Map::new();
        turn_detection.insert(
            "type".to_owned(),
            Value::String(TURN_DETECTION_TYPE.to_owned()),
        );
        turn_detection.insert("interrupt_response".to_owned(), Value::Bool(true));
        let mut input = Map::new();
        // No `format` key. See the module docs — this absence is the contract.
        input.insert("turn_detection".to_owned(), Value::Object(turn_detection));

        let mut format = Map::new();
        format.insert(
            "type".to_owned(),
            Value::String(OUTPUT_AUDIO_FORMAT.to_owned()),
        );
        format.insert(
            "rate".to_owned(),
            Value::Number(OUTPUT_SAMPLE_RATE.hz().into()),
        );
        let mut output = Map::new();
        output.insert("format".to_owned(), Value::Object(format));

        let mut audio = Map::new();
        audio.insert("input".to_owned(), Value::Object(input));
        audio.insert("output".to_owned(), Value::Object(output));
        session.insert("audio".to_owned(), Value::Object(audio));

        Value::Object(session)
    }

    fn build_speak_response(&self, content: &str) -> Value {
        // External contract — `s2s.mjs:98-103`. Four keys — one more than
        // DashScope's three: this provider also pins `tool_choice: 'none'`,
        // because its single response slot makes a stray tool call expensive.
        let mut response = Map::new();
        response.insert("conversation".to_owned(), Value::String("none".to_owned()));
        response.insert("modalities".to_owned(), Self::modalities());
        response.insert(
            "instructions".to_owned(),
            Value::String(speak_response_instructions(content, self.settings.locale)),
        );
        response.insert("tool_choice".to_owned(), Value::String("none".to_owned()));
        Value::Object(response)
    }

    fn build_result_injection(&self, content: &str) -> Injection {
        // External contract — `s2s.mjs:105-116`.
        Injection {
            item: self.protocol.user_text_item(content),
            response: Self::injection_response(result_response_instructions(self.settings.locale)),
        }
    }

    fn build_permission_injection(&self, permission: &PermissionRequest) -> Injection {
        // External contract — `s2s.mjs:118-137`.
        Injection {
            item: self
                .protocol
                .user_text_item(&permission_request_text(permission)),
            response: Self::injection_response(permission_response_instructions(
                self.settings.locale,
            )),
        }
    }

    fn missing_configuration_message(&self, locale: Locale) -> String {
        // External contract — `s2s.mjs:58`,
        // `请先配置 SPEECH_TO_SPEECH_REALTIME_URL`. The variable name is **KEEP**:
        // it names the third-party service, not VIA.
        t(locale, keys::GATEWAY_MISSING_SPEECH_TO_SPEECH_URL).to_owned()
    }

    fn connect_timeout_message(&self, locale: Locale) -> String {
        // External contract — `s2s.mjs:59`. The endpoint is interpolated, which
        // is the whole point of the sentence: a user who pointed the Gateway at
        // the wrong port learns which port it tried.
        i18n_format(
            locale,
            keys::REALTIME_CONNECT_TIMEOUT_SPEECH_TO_SPEECH,
            &[("url", &self.settings.url)],
        )
    }
}

impl Default for SpeechToSpeechProvider {
    fn default() -> Self {
        Self::new(SpeechToSpeechSettings::default())
    }
}

/// Build a provider straight from a resolved configuration.
impl From<&via_core::Config> for SpeechToSpeechProvider {
    fn from(config: &via_core::Config) -> Self {
        Self::new(SpeechToSpeechSettings::from_config(config))
    }
}
