//! The OpenAI / Azure / litellm realtime provider.
//!
//! One [`via_realtime::RealtimeProvider`], behind qwen's interface. Ported from
//! ARGO `tinicore/src/llm/providers/openai_live.rs` — the dialect, the two
//! session schemas, the auth shapes and the endpoint memory. It is **not** a
//! replacement for that interface: `via-realtime` already owns the session
//! machine, the registry, the connection status and the reconnect ladder, and
//! this file owns the dialect. The connect *walk* is [`crate::connect()`].
//!
//! # What is deliberately not declared
//!
//! **No audio format.** Both generations default to 24 kHz mono little-endian
//! PCM16, which is exactly what the host captures and plays, and GA accepts no
//! other PCM rate anyway — declaring it would add a field that can only ever
//! restate the default or be wrong. (ARGO `openai_live.rs:805-808`.)
//!
//! **No `turn_detection`.** The endpoint's own default is server VAD with
//! `interrupt_response`, which is what a barge-in session needs; ARGO never sends
//! the field, and re-sending it on a context refresh is how a provider's VAD gets
//! reset mid-conversation.
//!
//! **No `OpenAI-Beta` header.** That header selected the beta interface, which
//! OpenAI shut down on 2026-05-12. Sending it now is the inverse of the mismatch
//! it used to prevent — it asks for a gone interface while the payload and model
//! are GA — and Azure never accepted it at all.

use std::sync::{Arc, Mutex, PoisonError};

use serde_json::{Map, Value};
use via_audio::SampleRate;
use via_catalog::realtime_provider::realtime_provider_definition;
use via_catalog::{IdentityShape, RealtimeIdentity};
use via_i18n::{Locale, format, keys};
use via_realtime::{
    ErrorClass, GaRealtimeProtocol, Injection, OpenAiCompatibleProtocol, PermissionRequest,
    ProviderCapabilities, RealtimeError, RealtimeProtocol, RealtimeProvider, SessionRequest,
    ga_realtime_protocol, openai_compatible_protocol,
};

use crate::classify::classify_openai_error;
use crate::dialect::{OpenAiDialect, candidate_urls, host_is_azure_resource};
use crate::prompt::{
    permission_request_text, permission_response_instructions, result_response_instructions,
    speak_response_instructions,
};
use crate::schema::{RealtimeSchema, protocol_for_url};
use crate::settings::OpenAiSettings;

/// The canonical provider key.
///
/// External contract — `via-catalog`'s `openai` row, which `docs/architecture.md`
/// §7 adds to upstream's two. A client sends it on the `connect` frame and
/// `/api/health` echoes it.
pub const KEY: &str = "openai";

/// The human label, used when the catalog row is somehow unavailable.
pub const FALLBACK_LABEL: &str = "OpenAI Realtime";

/// The rate a client must capture at, in hertz.
///
/// The OpenAI Realtime input buffer is 24 kHz mono PCM16 in both generations,
/// and GA accepts no other PCM rate. It differs from DashScope's 16 000, which
/// is exactly why the value reaches every client on `voice.ready` instead of
/// being assumed: the resampler and the wake-word extractor both key off it.
pub const INPUT_SAMPLE_RATE: SampleRate = SampleRate::HZ_24000;

/// The rate this provider's audio output arrives at, in hertz.
pub const OUTPUT_SAMPLE_RATE: SampleRate = SampleRate::HZ_24000;

/// The GA session discriminator.
///
/// External contract — ARGO `openai_live.rs:730-736`. Its absence is a hard
/// rejection (`Missing required parameter: 'session.type'`), and the other value
/// (`transcription`) opens a transcription-only session, so it is never
/// inferable from the rest of the payload.
pub const SESSION_TYPE: &str = "realtime";

/// The transcription model for the user's own audio.
///
/// External contract — ARGO `openai_live.rs:196-200`. `whisper-1` because it
/// predates the GA split and **both generations accept it**; the newer
/// `gpt-4o-transcribe` family is GA-only, and the endpoint this provider was
/// written for speaks the pre-GA schema.
pub const INPUT_TRANSCRIPTION_MODEL: &str = "whisper-1";

/// The single GA output modality.
///
/// GA documents exactly two values, `["audio"]` and `["text"]`, and an audio
/// response already carries its transcript — so the pre-GA idiom of asking for
/// both is at best redundant and is not documented as accepted. ARGO collapses
/// rather than forwarding it (`openai_live.rs:748-760`).
pub const GA_OUTPUT_MODALITIES: [&str; 1] = ["audio"];

/// The pre-GA modality list.
///
/// The order is upstream qwen's own (`text` before `audio`), which is what both
/// beta-dialect providers in this workspace send.
pub const PREVIEW_MODALITIES: [&str; 2] = ["text", "audio"];

/// OpenAI Realtime, Azure Realtime and everything that gateways to them.
#[derive(Clone, Debug)]
pub struct OpenAiRealtimeProvider {
    settings: OpenAiSettings,
    label: &'static str,
    identity_shape: IdentityShape,
    ga: GaRealtimeProtocol,
    beta: OpenAiCompatibleProtocol,
    /// The candidate the walk committed to, once it has.
    ///
    /// Interior mutability because [`RealtimeProvider`] is `&self` throughout
    /// and the schema a session is configured in is a property of the endpoint
    /// that answered, not of the configuration. Holds a URL, never a credential.
    connected_url: Arc<Mutex<Option<String>>>,
}

impl OpenAiRealtimeProvider {
    /// A provider reading `settings`.
    #[must_use]
    pub fn new(settings: OpenAiSettings) -> Self {
        let definition = realtime_provider_definition(KEY).ok();
        Self {
            settings,
            label: definition.map_or(FALLBACK_LABEL, |row| row.label),
            // The shape is the catalog's, so adding a provider never has to
            // touch the signature code. `KEY` is one of the catalog's own rows,
            // so the lookup cannot miss; naming the shape it would have found
            // keeps this total without an `expect()`.
            identity_shape: definition
                .map_or(IdentityShape::ModelAndVoice, |row| row.identity_shape),
            ga: ga_realtime_protocol(),
            beta: openai_compatible_protocol(),
            connected_url: Arc::new(Mutex::new(None)),
        }
    }

    /// The settings this provider was built with.
    #[must_use]
    pub fn settings(&self) -> &OpenAiSettings {
        &self.settings
    }

    /// The host every candidate is built against.
    #[must_use]
    pub fn host(&self) -> String {
        self.settings.host()
    }

    /// Whether this provider's host is a first-party Azure resource.
    ///
    /// Decides both the candidate order and whether a bearer rides alongside
    /// `api-key`.
    #[must_use]
    pub fn host_is_azure_resource(&self) -> bool {
        host_is_azure_resource(&self.host())
    }

    /// The URLs the walk dials, in order.
    #[must_use]
    pub fn candidate_urls(&self) -> Vec<String> {
        candidate_urls(
            self.settings.dialect,
            self.settings.scheme(),
            &self.host(),
            self.settings.resolved_model(),
            self.settings.api_version(),
        )
    }

    /// The URL this session is configured against.
    ///
    /// The candidate the walk committed to, or — before a walk has run, and for
    /// a caller that used [`RealtimeSession::connect`] rather than
    /// [`crate::connect::connect`] — the first candidate, which is the one that
    /// would have been dialled first anyway.
    ///
    /// [`RealtimeSession::connect`]: via_realtime::RealtimeSession::connect
    #[must_use]
    pub fn session_url(&self) -> String {
        if let Some(url) = self
            .connected_url
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
        {
            return url;
        }
        self.candidate_urls()
            .into_iter()
            .next()
            .unwrap_or_else(|| format!("{}://{}/v1/realtime", self.settings.scheme(), self.host()))
    }

    /// Record the candidate a walk committed to.
    ///
    /// Called by [`crate::connect::connect`] the moment the first-frame probe
    /// accepts an endpoint, so [`schema`](Self::schema) and
    /// [`protocol`](RealtimeProvider::protocol) answer for the endpoint that is
    /// actually on the other end rather than for the one that was tried first.
    pub fn note_connected(&self, url: &str) {
        *self
            .connected_url
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(url.to_owned());
    }

    /// Forget the committed candidate.
    ///
    /// The walk calls this before it starts, so a reconnect after a gateway
    /// moved does not configure the new socket for the old endpoint.
    pub fn forget_connected(&self) {
        *self
            .connected_url
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = None;
    }

    /// The session schema this connection speaks.
    ///
    /// [`protocol_for_url`] answers with the endpoint's own verdict when it has
    /// given one, and with the route's shape until then.
    #[must_use]
    pub fn schema(&self) -> RealtimeSchema {
        protocol_for_url(&self.session_url())
    }

    /// The modalities a `response.create` body declares.
    ///
    /// The beta spelling (`modalities`) in both cases: the GA dialect adapter
    /// rewrites the key to `output_modalities` in
    /// [`response_create`](RealtimeProtocol::response_create), which is exactly
    /// where upstream puts that rename so no call site has to know.
    #[must_use]
    pub fn response_modalities(&self) -> Value {
        Self::modalities_for(self.schema())
    }

    fn modalities_for(schema: RealtimeSchema) -> Value {
        let names: &[&str] = match schema {
            RealtimeSchema::Ga => &GA_OUTPUT_MODALITIES,
            RealtimeSchema::Preview => &PREVIEW_MODALITIES,
        };
        Value::Array(
            names
                .iter()
                .map(|name| Value::String((*name).to_owned()))
                .collect(),
        )
    }

    /// The tool catalog, flattened.
    ///
    /// External contract — ARGO `openai_live.rs:812-828`: `tools` is *the one
    /// part of the payload that did not move at GA*, and both generations take
    /// the flat `{ type, name, description, parameters }` shape. VIA's
    /// [`AgentContext`](via_realtime::AgentContext) carries the beta-nested
    /// shape, because that is the one form a tool catalog is composed in, so the
    /// flattening lives in the provider — exactly where `s2s.mjs` puts it.
    ///
    /// A tool that is not in the nested shape is passed through unchanged rather
    /// than dropped: an empty `{}` on the wire is a schema error the service
    /// reports, and silently discarding a tool is a capability that disappears
    /// with no diagnostic at all.
    #[must_use]
    pub fn flat_tools(tools: &[Value]) -> Value {
        Value::Array(tools.iter().map(Self::flat_tool).collect())
    }

    fn flat_tool(tool: &Value) -> Value {
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

    /// The `session.update` body in one named schema.
    ///
    /// External contract — ARGO `openai_live.rs:705-829` (`build_session_update`),
    /// minus the frame wrapper, which is [`RealtimeProtocol::session_update`]'s.
    /// Key insertion order is ARGO's and goes on the wire.
    ///
    /// The explicit schema is what makes the candidate walk a walk: the walk
    /// varies the **route**, and the route is what decides the schema. Holding a
    /// single payload across candidates would dial every preview route with a GA
    /// body it must reject, and the rejection would read as *"wrong route"*
    /// rather than *"wrong schema"*.
    #[must_use]
    pub fn build_session_for(&self, schema: RealtimeSchema, request: &SessionRequest<'_>) -> Value {
        let mut session = Map::new();

        if schema == RealtimeSchema::Ga {
            // Required on **every** update, not only the first: it is the
            // discriminator that says which kind of session this is.
            session.insert("type".to_owned(), Value::String(SESSION_TYPE.to_owned()));
        }

        // Everything below the instructions is negotiated **once**. Upstream
        // qwen's catalogued rule (`DashScope session.update payload
        // (subsequent, configured)`) is what this follows, and for this endpoint
        // there is a second reason: OpenAI refuses a voice change once audio has
        // been generated, so re-sending the audio block on a context refresh is
        // the one thing that turns a harmless refresh into a session error.
        if !request.configured && schema == RealtimeSchema::Preview {
            session.insert("modalities".to_owned(), Self::modalities_for(schema));
        }
        if !request.configured && schema == RealtimeSchema::Ga {
            session.insert("output_modalities".to_owned(), Self::modalities_for(schema));
        }

        session.insert(
            "instructions".to_owned(),
            Value::String(request.agent_context.instructions.clone()),
        );

        if !request.configured {
            // Voice and input transcription both live under `session.audio` at
            // GA and flat on `session` before it, so they are assembled together
            // — inserting `audio` twice would drop whichever went first.
            match schema {
                RealtimeSchema::Ga => {
                    let mut audio = Map::new();
                    audio.insert(
                        "output".to_owned(),
                        Value::Object(Map::from_iter([(
                            "voice".to_owned(),
                            Value::String(self.settings.resolved_voice().to_owned()),
                        )])),
                    );
                    if self.settings.transcribe_input {
                        audio.insert("input".to_owned(), Self::ga_input_transcription());
                    }
                    session.insert("audio".to_owned(), Value::Object(audio));
                }
                RealtimeSchema::Preview => {
                    session.insert(
                        "voice".to_owned(),
                        Value::String(self.settings.resolved_voice().to_owned()),
                    );
                    if self.settings.transcribe_input {
                        session.insert(
                            "input_audio_transcription".to_owned(),
                            Self::transcription_model(),
                        );
                    }
                }
            }
        }

        if !request.agent_context.tools.is_empty() {
            session.insert(
                "tools".to_owned(),
                Self::flat_tools(&request.agent_context.tools),
            );
        }

        Value::Object(session)
    }

    fn transcription_model() -> Value {
        Value::Object(Map::from_iter([(
            "model".to_owned(),
            Value::String(INPUT_TRANSCRIPTION_MODEL.to_owned()),
        )]))
    }

    fn ga_input_transcription() -> Value {
        Value::Object(Map::from_iter([(
            "transcription".to_owned(),
            Self::transcription_model(),
        )]))
    }

    /// The `response.create` body shared by both injections.
    ///
    /// External contract — `dashscope.mjs:106-110`, `s2s.mjs:111-115`: three
    /// keys, in this order. `tool_choice: 'none'` is what stops the model
    /// answering a result or a permission question by calling another tool.
    fn injection_response(&self, instructions: String) -> Value {
        let mut response = Map::new();
        response.insert("modalities".to_owned(), self.response_modalities());
        response.insert("tool_choice".to_owned(), Value::String("none".to_owned()));
        response.insert("instructions".to_owned(), Value::String(instructions));
        Value::Object(response)
    }

    /// The `Authorization` / `api-key` pair this dialect authenticates with.
    ///
    /// External contract — ARGO `openai_live.rs:505-556`.
    ///
    /// Realtime cannot take the credential as a query parameter, so it rides the
    /// upgrade request's headers. On Azure a bearer goes out **as well** when the
    /// host is not a first-party resource, because then it is a gateway and
    /// gateways usually want one. Sending both is safe in a way that guessing is
    /// not: a real Azure resource ignores the header it does not use, and the
    /// alternative is a failed handshake whose cause is indistinguishable from a
    /// wrong route.
    #[must_use]
    pub fn auth_headers(&self) -> Vec<(String, String)> {
        let key = self.settings.api_key.expose();
        match self.settings.dialect {
            OpenAiDialect::OpenAi => {
                vec![("Authorization".to_owned(), format!("Bearer {key}"))]
            }
            OpenAiDialect::Azure => {
                let mut headers = vec![("api-key".to_owned(), key.to_owned())];
                if !self.host_is_azure_resource() {
                    headers.push(("Authorization".to_owned(), format!("Bearer {key}")));
                }
                headers
            }
        }
    }
}

impl RealtimeProvider for OpenAiRealtimeProvider {
    fn key(&self) -> &str {
        KEY
    }

    fn label(&self) -> &str {
        self.label
    }

    fn input_sample_rate(&self) -> u32 {
        INPUT_SAMPLE_RATE.hz()
    }

    fn output_sample_rate(&self) -> u32 {
        OUTPUT_SAMPLE_RATE.hz()
    }

    fn is_configured(&self) -> bool {
        if self.settings.api_key.is_empty() {
            return false;
        }
        // ARGO bring-up §5: *"A provider that is not OpenAI and has no endpoint
        // is now refused outright."* Before that guard it fell back to
        // `api.openai.com`, which answers `Incorrect API key provided` — so a
        // missing endpoint arrived as an accusation about the credential, from a
        // host nobody configured, after the whole candidate walk had been spent.
        self.settings.dialect == OpenAiDialect::OpenAi || self.settings.endpoint_configured()
    }

    fn model(&self) -> Option<&str> {
        Some(self.settings.resolved_model())
    }

    fn voice(&self) -> Option<&str> {
        Some(self.settings.resolved_voice())
    }

    fn url(&self) -> Result<String, RealtimeError> {
        // The **first** candidate. The walk is `crate::connect::connect`, which
        // is what a caller should use; this answers for the one code path that
        // cannot — `RealtimeSession::connect`, which opens a single socket — and
        // for that path the first candidate is the one it would have dialled.
        Ok(self.session_url())
    }

    fn headers(&self) -> Vec<(String, String)> {
        let mut headers = self.auth_headers();
        // Offered on **every** candidate. It is OpenAI's own subprotocol, a
        // server that does not use it is required by RFC 6455 to ignore the
        // field, and `api.openai.com` answers without it — so the downside is a
        // header, and the upside is the difference between a session and
        // silence (upstream ARGO bring-up §4).
        headers.push((
            crate::dialect::SUBPROTOCOL_HEADER.to_owned(),
            crate::dialect::SUBPROTOCOL.to_owned(),
        ));
        headers
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            // "Conversation already has an active response in progress" is the
            // provider naming this constraint outright. Declaring it is what
            // arms `via-realtime`'s bounded busy-retry ladder, which replays the
            // refused `response.create` byte-identically.
            single_response_slot: true,
            // Both generations accept `response.instructions`, which is what
            // lets `build_speak_response` set `conversation: 'none'` and still
            // have the model say the right thing.
            per_response_instructions: true,
            // `response_metadata_correlation` stays at the baseline `false` on
            // purpose: GA echoes response metadata but pre-GA does not, and the
            // capability snapshot is taken once, at construction, **before** the
            // endpoint has said which generation it speaks. A wrong `true`
            // mis-correlates silently; FIFO correlation is merely less precise.
            // `docs/deviations/phase-6-via-realtime-openai.md`.
            ..ProviderCapabilities::DEFAULT
        }
    }

    fn configuration_signature(&self) -> String {
        RealtimeIdentity::new(
            self.identity_shape,
            KEY,
            // The endpoint as configured, before any candidate is built from it.
            self.settings.base_url.as_str(),
            self.settings.model.as_str(),
            // The raw override, empty included: the default is not part of the
            // operator's configuration, so applying it would make the signature
            // change when this crate changed rather than when the config did.
            self.settings.voice.as_str(),
            self.settings.api_key.expose(),
        )
        .signature()
    }

    fn protocol(&self) -> &dyn RealtimeProtocol {
        match self.schema() {
            RealtimeSchema::Ga => &self.ga,
            RealtimeSchema::Preview => &self.beta,
        }
    }

    fn classify_error(&self, message: &str) -> ErrorClass {
        classify_openai_error(message)
    }

    fn build_session(&self, request: &SessionRequest<'_>) -> Value {
        self.build_session_for(self.schema(), request)
    }

    fn build_speak_response(&self, content: &str) -> Value {
        // External contract — `dashscope.mjs:94-98`, `s2s.mjs:98-102`.
        // `conversation: 'none'` keeps the utterance out of conversation
        // history, which is why the content has to be inside the instructions
        // rather than in an item.
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
        Injection {
            item: self.protocol().user_text_item(content),
            response: self.injection_response(result_response_instructions(self.settings.locale)),
        }
    }

    fn build_permission_injection(&self, permission: &PermissionRequest) -> Injection {
        Injection {
            item: self
                .protocol()
                .user_text_item(&permission_request_text(permission)),
            response: self
                .injection_response(permission_response_instructions(self.settings.locale)),
        }
    }

    fn missing_configuration_message(&self, locale: Locale) -> String {
        // "check its service address and configuration" covers both halves of
        // `is_configured`: a missing credential and — on the Azure dialect — a
        // missing endpoint.
        format(
            locale,
            keys::REALTIME_MISSING_CONFIGURATION,
            &[("label", self.label)],
        )
    }

    fn connect_timeout_message(&self, locale: Locale) -> String {
        format(
            locale,
            keys::REALTIME_CONNECT_TIMEOUT,
            &[("label", self.label)],
        )
    }
}

impl Default for OpenAiRealtimeProvider {
    fn default() -> Self {
        Self::new(OpenAiSettings::default())
    }
}

/// Build a provider straight from a resolved configuration.
impl From<&via_core::Config> for OpenAiRealtimeProvider {
    fn from(config: &via_core::Config) -> Self {
        Self::new(OpenAiSettings::from_config(config))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use via_core::Secret;
    use via_realtime::{AgentContext, validate_realtime_provider};

    use super::*;
    use crate::dialect::OPENAI_REALTIME_HOST;

    fn build(settings: OpenAiSettings) -> OpenAiRealtimeProvider {
        OpenAiRealtimeProvider::new(settings)
    }

    fn configured() -> OpenAiRealtimeProvider {
        build(OpenAiSettings {
            api_key: Secret::new("sk-test"),
            ..OpenAiSettings::default()
        })
    }

    fn context() -> AgentContext {
        AgentContext {
            instructions: "be brief".to_owned(),
            tools: vec![json!({
                "type": "function",
                "function": {
                    "name": "web_search",
                    "description": "search",
                    "parameters": { "type": "object" }
                }
            })],
            ..AgentContext::default()
        }
    }

    fn session(
        provider: &OpenAiRealtimeProvider,
        schema: RealtimeSchema,
        configured: bool,
    ) -> Value {
        let context = context();
        provider.build_session_for(
            schema,
            &SessionRequest {
                configured,
                agent_context: &context,
            },
        )
    }

    #[test]
    fn the_provider_validates() {
        assert_eq!(validate_realtime_provider(&configured()), Ok(()));
        assert_eq!(configured().key(), "openai");
        assert_eq!(configured().label(), "OpenAI Realtime");
        assert!(configured().aliases().is_empty());
    }

    #[test]
    fn the_label_comes_from_the_catalog_row() {
        let row = realtime_provider_definition(KEY).expect("the openai row exists");
        assert_eq!(configured().label(), row.label);
        assert_eq!(row.label, FALLBACK_LABEL);
    }

    #[test]
    fn both_sample_rates_are_the_realtime_pcm_rate() {
        assert_eq!(configured().input_sample_rate(), 24_000);
        assert_eq!(configured().output_sample_rate(), 24_000);
    }

    #[test]
    fn a_key_alone_configures_the_openai_dialect() {
        assert!(configured().is_configured());
        assert!(!OpenAiRealtimeProvider::default().is_configured());
    }

    #[test]
    fn the_azure_dialect_with_no_endpoint_is_unconfigured_rather_than_dialling_openai() {
        // ARGO bring-up §5: the fallback to `api.openai.com` cost two rounds,
        // because it arrived as an accusation about the credential from a host
        // nobody configured.
        let provider = build(OpenAiSettings {
            api_key: Secret::new("sk-test"),
            dialect: OpenAiDialect::Azure,
            base_url: String::new(),
            ..OpenAiSettings::default()
        });
        assert!(!provider.is_configured());
        assert!(
            !provider
                .missing_configuration_message(Locale::En)
                .contains('{')
        );

        let with_endpoint = build(OpenAiSettings {
            api_key: Secret::new("sk-test"),
            dialect: OpenAiDialect::Azure,
            base_url: "https://r.openai.azure.com".to_owned(),
            ..OpenAiSettings::default()
        });
        assert!(with_endpoint.is_configured());
    }

    #[test]
    fn the_upgrade_offers_the_realtime_subprotocol_and_never_the_beta_header() {
        // The subprotocol pins the header the gateway probe showed to matter:
        // without it the handshake got a bare 101, with it the server echoed
        // `Sec-WebSocket-Protocol: realtime`. A regression here presents as a
        // socket that upgrades and then never speaks.
        let headers = configured().headers();
        assert!(
            headers
                .iter()
                .any(|(name, value)| name == "Sec-WebSocket-Protocol" && value == "realtime"),
            "{headers:?}"
        );
        for (name, _) in &headers {
            assert!(
                !name.eq_ignore_ascii_case("OpenAI-Beta"),
                "the beta interface shut down 2026-05-12: {headers:?}"
            );
        }
        // A credential must never ride in a subprotocol name — that field is
        // logged by every proxy on the path.
        for (name, value) in &headers {
            if name == "Sec-WebSocket-Protocol" {
                assert_eq!(value, "realtime");
            }
        }
    }

    #[test]
    fn the_openai_dialect_authenticates_with_a_bearer_only() {
        let headers = configured().auth_headers();
        assert_eq!(
            headers,
            vec![("Authorization".to_owned(), "Bearer sk-test".to_owned())]
        );
    }

    #[test]
    fn a_first_party_azure_resource_gets_api_key_and_no_bearer() {
        let provider = build(OpenAiSettings {
            api_key: Secret::new("azure-key"),
            dialect: OpenAiDialect::Azure,
            base_url: "https://r.openai.azure.com/openai/v1".to_owned(),
            ..OpenAiSettings::default()
        });
        assert_eq!(
            provider.auth_headers(),
            vec![("api-key".to_owned(), "azure-key".to_owned())]
        );
    }

    #[test]
    fn a_gateway_in_front_of_azure_gets_both_credentials() {
        // Sending both is safe in a way that guessing is not: a real resource
        // ignores the header it does not use.
        let provider = build(OpenAiSettings {
            api_key: Secret::new("gw-key"),
            dialect: OpenAiDialect::Azure,
            base_url: "https://proxy.example.internal".to_owned(),
            ..OpenAiSettings::default()
        });
        assert_eq!(
            provider.auth_headers(),
            vec![
                ("api-key".to_owned(), "gw-key".to_owned()),
                ("Authorization".to_owned(), "Bearer gw-key".to_owned()),
            ]
        );
    }

    #[test]
    fn the_url_is_the_first_candidate_until_a_walk_commits() {
        let provider = build(OpenAiSettings {
            api_key: Secret::new("k"),
            dialect: OpenAiDialect::Azure,
            base_url: "https://gw.example".to_owned(),
            ..OpenAiSettings::default()
        });
        assert_eq!(
            provider.url().expect("a url"),
            "wss://gw.example/v1/realtime?model=gpt-realtime-2.1-mini"
        );

        provider.note_connected("wss://gw.example/openai/realtime?api-version=x&deployment=d");
        assert_eq!(
            provider.url().expect("a url"),
            "wss://gw.example/openai/realtime?api-version=x&deployment=d"
        );
        provider.forget_connected();
        assert!(
            provider
                .url()
                .expect("a url")
                .ends_with("/v1/realtime?model=gpt-realtime-2.1-mini")
        );
    }

    #[test]
    fn the_schema_follows_the_candidate_that_answered() {
        let provider = build(OpenAiSettings {
            api_key: Secret::new("k"),
            dialect: OpenAiDialect::Azure,
            base_url: "https://schema-walk.example".to_owned(),
            ..OpenAiSettings::default()
        });
        assert_eq!(provider.schema(), RealtimeSchema::Ga);
        provider
            .note_connected("wss://schema-walk.example/openai/realtime?api-version=x&deployment=d");
        assert_eq!(provider.schema(), RealtimeSchema::Preview);
    }

    #[test]
    fn the_dialect_selects_the_wire_adapter() {
        let provider = build(OpenAiSettings {
            api_key: Secret::new("k"),
            dialect: OpenAiDialect::Azure,
            base_url: "https://adapter-pick.example".to_owned(),
            ..OpenAiSettings::default()
        });
        // GA namespaces conversation item ids; the beta dialect does not.
        let item = json!({ "type": "function_call_output" });
        assert!(
            provider
                .protocol()
                .conversation_item_id(&item)
                .starts_with("fco_"),
            "the GA route must use the GA dialect"
        );
        provider.note_connected(
            "wss://adapter-pick.example/openai/realtime?api-version=x&deployment=d",
        );
        assert!(
            !provider
                .protocol()
                .conversation_item_id(&item)
                .starts_with("fco_"),
            "a preview route must use the beta dialect"
        );
    }

    #[test]
    fn the_ga_session_payload_is_argos_shape_and_key_order() {
        let payload = session(&configured(), RealtimeSchema::Ga, false);
        let keys: Vec<&str> = payload
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            [
                "type",
                "output_modalities",
                "instructions",
                "audio",
                "tools"
            ]
        );
        assert_eq!(payload["type"], json!("realtime"));
        assert_eq!(payload["output_modalities"], json!(["audio"]));
        assert_eq!(payload["instructions"], json!("be brief"));
        assert_eq!(payload["audio"]["output"]["voice"], json!("alloy"));
        assert_eq!(payload["tools"][0]["type"], json!("function"));
        assert_eq!(payload["tools"][0]["name"], json!("web_search"));
        // Every relocated field must be ABSENT, not merely duplicated — GA
        // rejects unknown parameters rather than ignoring them.
        for gone in ["modalities", "voice", "input_audio_transcription"] {
            assert!(
                payload.get(gone).is_none(),
                "{gone} must not survive into GA"
            );
        }
    }

    #[test]
    fn the_preview_session_payload_is_flat_and_carries_no_discriminator() {
        let payload = session(&configured(), RealtimeSchema::Preview, false);
        let keys: Vec<&str> = payload
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            [
                "modalities",
                "instructions",
                "voice",
                "input_audio_transcription",
                "tools"
            ]
        );
        assert_eq!(payload["modalities"], json!(["text", "audio"]));
        assert_eq!(payload["voice"], json!("alloy"));
        // The discriminator is GA-only; sending it to the preview interface is
        // itself a rejection (`Unknown parameter: 'session.type'`).
        assert!(payload.get("type").is_none());
        assert!(payload.get("audio").is_none());
        assert!(payload.get("output_modalities").is_none());
    }

    #[test]
    fn a_subsequent_update_renegotiates_nothing_but_keeps_the_ga_discriminator() {
        let ga = session(&configured(), RealtimeSchema::Ga, true);
        assert_eq!(
            ga.as_object().expect("object").keys().collect::<Vec<_>>(),
            ["type", "instructions", "tools"]
        );
        assert_eq!(ga["type"], json!("realtime"));
        for gone in ["output_modalities", "audio"] {
            assert!(ga.get(gone).is_none(), "{gone} must not be renegotiated");
        }

        let preview = session(&configured(), RealtimeSchema::Preview, true);
        assert_eq!(
            preview
                .as_object()
                .expect("object")
                .keys()
                .collect::<Vec<_>>(),
            ["instructions", "tools"]
        );
        for gone in ["type", "modalities", "voice", "input_audio_transcription"] {
            assert!(preview.get(gone).is_none(), "{gone}");
        }
    }

    #[test]
    fn input_transcription_lands_in_each_schemas_own_place() {
        let ga = session(&configured(), RealtimeSchema::Ga, false);
        assert_eq!(
            ga["audio"]["input"]["transcription"]["model"],
            json!("whisper-1")
        );
        // Both live under `session.audio` at GA. Inserting that key twice would
        // drop whichever was written first — the regression this pins.
        assert_eq!(ga["audio"]["output"]["voice"], json!("alloy"));

        let preview = session(&configured(), RealtimeSchema::Preview, false);
        assert_eq!(
            preview["input_audio_transcription"]["model"],
            json!("whisper-1")
        );
    }

    #[test]
    fn transcription_can_be_turned_off_and_then_neither_schema_asks_for_it() {
        let provider = build(OpenAiSettings {
            api_key: Secret::new("k"),
            transcribe_input: false,
            ..OpenAiSettings::default()
        });
        let ga = session(&provider, RealtimeSchema::Ga, false);
        assert!(ga["audio"].get("input").is_none());
        assert_eq!(ga["audio"]["output"]["voice"], json!("alloy"));

        let preview = session(&provider, RealtimeSchema::Preview, false);
        assert!(preview.get("input_audio_transcription").is_none());
        assert_eq!(preview["voice"], json!("alloy"));
    }

    #[test]
    fn a_session_with_no_tools_declares_no_tools_key() {
        let provider = configured();
        let empty = AgentContext {
            instructions: "hi".to_owned(),
            ..AgentContext::default()
        };
        for schema in [RealtimeSchema::Ga, RealtimeSchema::Preview] {
            let payload = provider.build_session_for(
                schema,
                &SessionRequest {
                    configured: false,
                    agent_context: &empty,
                },
            );
            assert!(payload.get("tools").is_none(), "{schema}");
        }
    }

    #[test]
    fn a_tool_that_is_not_in_the_nested_shape_is_passed_through_rather_than_dropped() {
        let already_flat = json!({ "type": "function", "name": "n", "parameters": {} });
        assert_eq!(
            OpenAiRealtimeProvider::flat_tools(std::slice::from_ref(&already_flat)),
            json!([already_flat])
        );
    }

    #[test]
    fn a_nested_tool_missing_a_field_flattens_to_null_rather_than_vanishing() {
        let partial = json!({ "type": "function", "function": { "name": "n" } });
        assert_eq!(
            OpenAiRealtimeProvider::flat_tools(std::slice::from_ref(&partial)),
            json!([{ "type": "function", "name": "n", "description": null, "parameters": null }])
        );
    }

    #[test]
    fn the_capability_declaration_is_the_two_flags_this_endpoint_needs() {
        let capabilities = configured().capabilities();
        assert!(capabilities.single_response_slot);
        assert!(capabilities.per_response_instructions);
        assert!(capabilities.acknowledges_session_update);
        assert!(capabilities.conversation_item_id_echo);
        assert!(
            !capabilities.response_metadata_correlation,
            "the generation is unknown at construction; FIFO correlation is the safe answer"
        );
    }

    #[test]
    fn the_speak_response_keeps_the_utterance_out_of_history() {
        let response = configured().build_speak_response("三点有个会");
        assert_eq!(response["conversation"], json!("none"));
        assert_eq!(response["modalities"], json!(["audio"]));
        assert!(
            response["instructions"]
                .as_str()
                .unwrap_or_default()
                .contains("三点有个会")
        );
    }

    #[test]
    fn both_injections_are_an_item_plus_a_tool_free_response() {
        let provider = configured();
        let result = provider.build_result_injection("done");
        assert_eq!(result.item["role"], json!("user"));
        assert_eq!(result.item["content"][0]["text"], json!("done"));
        assert_eq!(result.response["tool_choice"], json!("none"));

        let permission = provider.build_permission_injection(&PermissionRequest {
            id: "perm_1".to_owned(),
            summary: "delete build".to_owned(),
        });
        assert!(
            permission.item["content"][0]["text"]
                .as_str()
                .unwrap_or_default()
                .starts_with("<backend_permission_request>")
        );
        assert_eq!(permission.response["tool_choice"], json!("none"));
    }

    #[test]
    fn the_configuration_signature_moves_with_the_configuration_and_not_with_the_defaults() {
        let base = OpenAiSettings {
            api_key: Secret::new("k"),
            ..OpenAiSettings::default()
        };
        let first = build(base.clone()).configuration_signature();
        assert_eq!(first.len(), 64, "lowercase hex sha-256");
        assert_eq!(first, build(base.clone()).configuration_signature());

        for changed in [
            OpenAiSettings {
                model: "gpt-realtime-2.1".to_owned(),
                ..base.clone()
            },
            OpenAiSettings {
                voice: "marin".to_owned(),
                ..base.clone()
            },
            OpenAiSettings {
                base_url: "https://gw.example".to_owned(),
                ..base.clone()
            },
            OpenAiSettings {
                api_key: Secret::new("k2"),
                ..base.clone()
            },
        ] {
            assert_ne!(build(changed).configuration_signature(), first);
        }

        // The locale and the transcription knob are not part of the identity —
        // neither changes what a client would have to reconnect for.
        assert_eq!(
            build(OpenAiSettings {
                locale: Locale::Ko,
                transcribe_input: false,
                ..base
            })
            .configuration_signature(),
            first
        );
    }

    #[test]
    fn the_provider_never_prints_its_credential() {
        let rendered = format!("{:?}", configured());
        assert!(!rendered.contains("sk-test"), "{rendered}");
    }

    #[test]
    fn an_unconfigured_provider_still_names_a_dialable_host() {
        let provider = OpenAiRealtimeProvider::default();
        assert_eq!(provider.host(), OPENAI_REALTIME_HOST);
        assert_eq!(provider.candidate_urls().len(), 1);
    }
}
