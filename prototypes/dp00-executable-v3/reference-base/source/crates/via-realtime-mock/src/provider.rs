//! The `mock` realtime provider.
//!
//! One [`via_realtime::RealtimeProvider`], declared as data so a JSON fixture
//! can configure it. It is a real provider in every sense the registry cares
//! about — it validates, it resolves by key, it publishes a configuration
//! signature and a `/api/health` descriptor — and it differs from the two
//! shipped providers in exactly one way: [`url`](MockProvider::url) names no
//! socket, because the service it fronts is a task in this process.
//!
//! # What it does not restate
//!
//! - [`via_catalog`] owns the `mock` row: the key, the label and the
//!   configuration-identity shape all come out of
//!   [`via_catalog::realtime_providers`], and the model profile out of
//!   [`via_catalog::local_realtime_model_profile`].
//! - [`via_realtime`] owns both dialects. `MockDialect` chooses one; it does not
//!   implement one.
//! - [`via_realtime::testing::default_test_classification`] owns the union of
//!   the two shipped providers' error corpora, so a scripted refusal is
//!   classified exactly the way a real one is.
//! - [`via_i18n`] owns the two sentences a person reads.
//!
//! # Declaring capabilities honestly
//!
//! [`ProviderCapabilities`] is five flags and every one of them exists to be
//! exercised, so [`MockProviderSpec::capabilities`] is free and the script
//! server *behaves* accordingly rather than merely declaring: it withholds
//! `session.updated` when it says it does not acknowledge, refuses a concurrent
//! `response.create` when it says it has one slot, echoes correlation metadata
//! only when it says it correlates, and replaces item ids when it says it does
//! not echo them. See [`crate::server`].

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use via_catalog::{
    IdentityShape, ModelProfile, ProviderDefinition, RealtimeIdentity, local_realtime_model_profile,
};
use via_i18n::{Locale, format, keys};
use via_realtime::testing::default_test_classification;
use via_realtime::{
    ErrorClass, Injection, PermissionRequest, ProviderCapabilities, RealtimeError,
    RealtimeProtocol, RealtimeProvider, SessionRequest, Visibility, ga_realtime_protocol,
    openai_compatible_protocol,
};

/// The catalog key this provider answers to.
///
/// External contract — the `mock` row of `via_catalog::realtime_providers`, and
/// what a client sends as `provider` on the `connect` event.
pub const MOCK_PROVIDER_KEY: &str = "mock";

/// The label used when the catalog row cannot be found.
///
/// The catalog is the source; this is the unreachable fallback that keeps
/// [`Default`] total, and
/// `tests::the_defaults_come_from_the_catalog_row_not_from_here` proves the arm
/// is dead.
const FALLBACK_LABEL: &str = "Mock Realtime";

/// The model id a conversing mock declares.
///
/// VIA's own — the mock is not a vendor service, so this names nothing external.
/// It resolves through [`via_catalog::local_realtime_model_profile`], which is
/// what gives the mock real capability flags rather than the
/// all-capabilities-false profile `docs/architecture.md` §7 refuses to port.
pub const MOCK_MODEL: &str = "mock-realtime";

/// The endpoint a mock provider publishes.
///
/// It is not reachable and is not meant to be: the "service" is an owning task
/// in this process. It exists because the endpoint is one of the three fields
/// the `EndpointOnly` configuration signature hashes, and because a descriptor
/// with a blank endpoint reads as a misconfiguration rather than as a mock.
pub const MOCK_ENDPOINT: &str = "mock://in-process";

/// The model-visible envelope a permission question is asked in.
///
/// External contract — `prompt-text` / *backend permission injection item text*
/// (`providers/dashscope.mjs:113-132`, `providers/s2s.mjs:118-137`).
/// `config/frontend-agent/PROMPT.md` references the tag by name, so a mock that
/// spelled it differently would make every permission test agree with itself and
/// with nothing else.
pub const PERMISSION_REQUEST_TAG: &str = "backend_permission_request";

/// Which wire dialect a mock provider speaks.
///
/// Both are [`via_realtime`]'s; this only chooses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MockDialect {
    /// The beta OpenAI Realtime envelope, which DashScope speaks.
    ///
    /// No portable response-metadata contract, so a `beta` mock cannot correlate
    /// by metadata however it declares itself — the same constraint the real
    /// dialect has.
    #[default]
    Beta,
    /// The GA (2025+) envelope, which huggingface/speech-to-speech speaks.
    ///
    /// `response.create` carries `output_modalities` and a metadata correlation
    /// id, and text deltas arrive as `response.output_text.*`.
    Ga,
}

impl MockDialect {
    /// The shipped adapter for this dialect.
    #[must_use]
    fn protocol(self) -> Box<dyn RealtimeProtocol> {
        match self {
            Self::Beta => Box::new(openai_compatible_protocol()),
            Self::Ga => Box::new(ga_realtime_protocol()),
        }
    }

    /// Whether this dialect carries a correlation id at all.
    ///
    /// [`via_realtime::OpenAiCompatibleProtocol`]'s `correlate_response_create`
    /// is the identity,
    /// so a beta provider that declared `response_metadata_correlation` would
    /// simply never correlate.
    #[must_use]
    pub const fn correlates(self) -> bool {
        matches!(self, Self::Ga)
    }
}

/// One classification rule a script can add.
///
/// Checked in declaration order, before
/// [`default_test_classification`]. Matching is a case-insensitive substring
/// test, which is what the two shipped corpora' regexes reduce to for a fixed
/// phrase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationRule {
    /// The phrase to look for.
    pub contains: String,
    /// What to classify a message containing it as.
    pub class: ErrorClass,
}

impl ClassificationRule {
    /// Classify any message containing `contains` as `class`.
    #[must_use]
    pub fn new(contains: impl Into<String>, class: ErrorClass) -> Self {
        Self {
            contains: contains.into(),
            class,
        }
    }
}

/// Everything a mock provider declares about itself.
///
/// Every field is data so a JSON fixture can set it, and the whole struct takes
/// `#[serde(default)]` so a fixture names only what it changes. The defaults are
/// the catalog's `mock` row plus a conversing local model.
///
/// Nothing is `skip_serializing_if`, deliberately. `model: null` and
/// `model: <absent>` mean *opposite* things here — the first is a provider that
/// mounts no model, the second is one that takes the default — so skipping an
/// absent `Option` would turn a `dictation` spec into a conversing one on the
/// way back in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MockProviderSpec {
    /// The canonical key. Must match `^[a-z0-9][a-z0-9-]*$`.
    pub key: String,
    /// The human label, echoed in `/api/health` and in every connection error.
    pub label: String,
    /// Alternate spellings that resolve to [`key`](Self::key).
    pub aliases: Vec<String>,
    /// Whether this provider appears in client-facing discovery.
    pub visibility: Visibility,
    /// Which wire dialect it speaks.
    pub dialect: MockDialect,
    /// The five behavioural flags.
    pub capabilities: ProviderCapabilities,
    /// The rate a client must capture at.
    pub input_sample_rate: u32,
    /// The rate this provider's audio output arrives at.
    pub output_sample_rate: u32,
    /// Override the response-start watchdog, in milliseconds.
    pub response_start_timeout_ms: Option<u64>,
    /// Whether the provider reports itself as configured.
    ///
    /// `false` makes `open` refuse with `RealtimeError::NotConfigured` before
    /// anything is written, which is the path a picker's `configured_only`
    /// filter exists for.
    pub configured: bool,
    /// The active model id, or `None` for a provider that mounts no model.
    pub model: Option<String>,
    /// The active voice id, or `None`.
    pub voice: Option<String>,
    /// The modalities every response body declares.
    ///
    /// `None` derives them from the model profile, the way
    /// `providers/dashscope.mjs:35-41` does.
    pub modalities: Option<Vec<String>>,
    /// The endpoint the configuration signature hashes.
    pub endpoint: String,
    /// The credential the configuration signature hashes. Empty for a mock.
    pub credential: String,
    /// Extra error classification, checked before the shared corpus.
    pub classification: Vec<ClassificationRule>,
    /// Refuse to connect at all, before a socket is opened.
    ///
    /// The `preflight()` gate: `true` answers
    /// [`RealtimeError::UnsupportedModel`], which is the Rust seam for
    /// upstream's `modelProfile?.family === 'unknown'` check.
    pub unsupported_model: bool,
}

/// The catalog's `mock` row, when it is there.
fn catalog_row() -> Option<&'static ProviderDefinition> {
    via_catalog::realtime_providers()
        .iter()
        .find(|provider| provider.key == MOCK_PROVIDER_KEY)
}

impl Default for MockProviderSpec {
    fn default() -> Self {
        let row = catalog_row();
        let model = MOCK_MODEL.to_owned();
        let voice = local_realtime_model_profile(&model)
            .session_defaults
            .voice
            .to_string();
        Self {
            key: row.map_or(MOCK_PROVIDER_KEY, |row| row.key).to_owned(),
            label: row.map_or(FALLBACK_LABEL, |row| row.label).to_owned(),
            aliases: row
                .map(|row| row.aliases)
                .unwrap_or_default()
                .iter()
                .map(|alias| (*alias).to_owned())
                .collect(),
            visibility: Visibility::Public,
            dialect: MockDialect::Beta,
            capabilities: ProviderCapabilities::DEFAULT,
            input_sample_rate: via_audio::SampleRate::HZ_16000.hz(),
            output_sample_rate: via_audio::SampleRate::HZ_24000.hz(),
            response_start_timeout_ms: None,
            configured: true,
            model: Some(model),
            voice: Some(voice),
            modalities: None,
            endpoint: MOCK_ENDPOINT.to_owned(),
            credential: String::new(),
            classification: Vec::new(),
            unsupported_model: false,
        }
    }
}

impl MockProviderSpec {
    /// A provider that mounts no model — `docs/architecture.md` §2's
    /// `dictation`.
    ///
    /// No model, no voice, and therefore no model profile and an empty model
    /// catalog, which `/api/health` publishes as `realtimeModelIds: null`
    /// exactly as it does for `speech-to-speech`.
    #[must_use]
    pub fn dictation() -> Self {
        Self {
            model: None,
            voice: None,
            ..Self::default()
        }
    }

    /// The capability set huggingface/speech-to-speech declares, on the GA
    /// dialect.
    ///
    /// The other side of every flag that has one: no `session.updated`, one
    /// response slot, metadata correlation, per-response instructions.
    #[must_use]
    pub fn speech_to_speech_shaped() -> Self {
        Self {
            dialect: MockDialect::Ga,
            capabilities: ProviderCapabilities {
                acknowledges_session_update: false,
                single_response_slot: true,
                response_metadata_correlation: true,
                per_response_instructions: true,
                ..ProviderCapabilities::DEFAULT
            },
            ..Self::default()
        }
    }

    /// Override the key.
    #[must_use]
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }

    /// Override the label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Override the aliases.
    #[must_use]
    pub fn with_aliases<I, S>(mut self, aliases: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.aliases = aliases.into_iter().map(Into::into).collect();
        self
    }

    /// Override the visibility.
    #[must_use]
    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Override the dialect.
    #[must_use]
    pub fn with_dialect(mut self, dialect: MockDialect) -> Self {
        self.dialect = dialect;
        self
    }

    /// Override the five flags.
    #[must_use]
    pub fn with_capabilities(mut self, capabilities: ProviderCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Override whether the provider reports itself configured.
    #[must_use]
    pub fn with_configured(mut self, configured: bool) -> Self {
        self.configured = configured;
        self
    }

    /// Override the model id.
    #[must_use]
    pub fn with_model(mut self, model: Option<&str>) -> Self {
        self.model = model.map(str::to_owned);
        self
    }

    /// Override the voice id.
    #[must_use]
    pub fn with_voice(mut self, voice: Option<&str>) -> Self {
        self.voice = voice.map(str::to_owned);
        self
    }

    /// Override the declared modalities.
    #[must_use]
    pub fn with_modalities<I, S>(mut self, modalities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.modalities = Some(modalities.into_iter().map(Into::into).collect());
        self
    }

    /// Override the capture rate.
    #[must_use]
    pub fn with_input_sample_rate(mut self, rate: u32) -> Self {
        self.input_sample_rate = rate;
        self
    }

    /// Override the playback rate.
    #[must_use]
    pub fn with_output_sample_rate(mut self, rate: u32) -> Self {
        self.output_sample_rate = rate;
        self
    }

    /// Override the response-start budget.
    #[must_use]
    pub fn with_response_start_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.response_start_timeout_ms =
            timeout.map(|timeout| u64::try_from(timeout.as_millis()).unwrap_or(u64::MAX));
        self
    }

    /// Add a classification rule, checked before the shared corpus.
    #[must_use]
    pub fn classifying(mut self, contains: impl Into<String>, class: ErrorClass) -> Self {
        self.classification
            .push(ClassificationRule::new(contains, class));
        self
    }

    /// Refuse to connect at all — the `preflight()` gate.
    #[must_use]
    pub fn unsupported(mut self) -> Self {
        self.unsupported_model = true;
        self
    }

    /// The profile of the declared model, if there is one.
    #[must_use]
    pub fn model_profile(&self) -> Option<ModelProfile> {
        self.model.as_deref().map(local_realtime_model_profile)
    }

    /// The configuration-identity shape, from the catalog row for this key.
    ///
    /// A key the catalog does not know takes
    /// [`IdentityShape::EndpointOnly`] — the mock has no vendor credential to
    /// name, so the three-key shape is the honest one.
    #[must_use]
    pub fn identity_shape(&self) -> IdentityShape {
        via_catalog::realtime_provider_definition(&self.key)
            .map_or(IdentityShape::EndpointOnly, |row| row.identity_shape)
    }

    /// This provider, validated and ready for a registry or a session.
    ///
    /// # Errors
    ///
    /// Whatever [`via_realtime::validate_realtime_provider`] refuses: a blank
    /// key or label, a key a client could never send, a zero sample rate, a zero
    /// response-start timeout, a blank alias.
    pub fn build(self) -> Result<Arc<dyn RealtimeProvider>, RealtimeError> {
        via_realtime::define_realtime_provider(Arc::new(MockProvider::new(self)))
    }
}

/// A deterministic realtime provider.
///
/// Build it from a [`MockProviderSpec`]; drive it with a
/// [`Script`](crate::Script) through [`MockRealtime`](crate::MockRealtime).
#[derive(Debug)]
pub struct MockProvider {
    spec: MockProviderSpec,
    protocol: Box<dyn RealtimeProtocol>,
    profile: Option<ModelProfile>,
    catalog: Vec<ModelProfile>,
    identity_shape: IdentityShape,
}

impl MockProvider {
    /// Build a provider from its declaration, without validating it.
    ///
    /// [`MockProviderSpec::build`] is the validating door and is what every
    /// caller should use; this one exists for a test that wants to observe an
    /// *invalid* provider.
    #[must_use]
    pub fn new(spec: MockProviderSpec) -> Self {
        let profile = spec.model_profile();
        Self {
            protocol: spec.dialect.protocol(),
            catalog: profile.iter().cloned().collect(),
            identity_shape: spec.identity_shape(),
            profile,
            spec,
        }
    }

    /// What this provider declares about itself.
    #[must_use]
    pub fn spec(&self) -> &MockProviderSpec {
        &self.spec
    }

    /// The modalities every response body this provider builds declares.
    ///
    /// Derived from the model profile the way
    /// `providers/dashscope.mjs:35-41` derives its own, unless the spec named
    /// them. A provider with no profile declares `audio`, which is what
    /// `providers/s2s.mjs` hard-codes for the same reason: there is no profile
    /// to ask.
    fn modalities(&self) -> Value {
        if let Some(declared) = &self.spec.modalities {
            return Value::Array(declared.iter().cloned().map(Value::String).collect());
        }
        let Some(profile) = &self.profile else {
            return Value::Array(vec![Value::String("audio".to_owned())]);
        };
        let mut modalities = Vec::new();
        if profile.model_capabilities.text_output {
            modalities.push(Value::String("text".to_owned()));
        }
        if profile.model_capabilities.audio_output {
            modalities.push(Value::String("audio".to_owned()));
        }
        Value::Array(modalities)
    }

    /// The `{ modalities, tool_choice: 'none' }` half every injection response
    /// shares.
    ///
    /// It says `modalities` even on the GA dialect, deliberately:
    /// [`GaRealtimeProtocol::response_create`](via_realtime::GaRealtimeProtocol)
    /// rewrites the key to `output_modalities` on the way out, and upstream is
    /// explicit that the rewrite lives there precisely so shared frontend code
    /// keeps passing the beta field name (`ga-protocol.mjs:7-9`).
    fn injection_response(&self) -> Map<String, Value> {
        let mut response = Map::new();
        response.insert("modalities".to_owned(), self.modalities());
        response.insert("tool_choice".to_owned(), Value::String("none".to_owned()));
        response
    }
}

impl RealtimeProvider for MockProvider {
    fn key(&self) -> &str {
        &self.spec.key
    }

    fn label(&self) -> &str {
        &self.spec.label
    }

    fn aliases(&self) -> Vec<String> {
        self.spec.aliases.clone()
    }

    fn visibility(&self) -> Visibility {
        self.spec.visibility
    }

    fn input_sample_rate(&self) -> u32 {
        self.spec.input_sample_rate
    }

    fn output_sample_rate(&self) -> u32 {
        self.spec.output_sample_rate
    }

    fn response_start_timeout(&self) -> Option<Duration> {
        self.spec
            .response_start_timeout_ms
            .map(Duration::from_millis)
    }

    fn is_configured(&self) -> bool {
        self.spec.configured
    }

    fn model(&self) -> Option<&str> {
        self.spec.model.as_deref()
    }

    fn voice(&self) -> Option<&str> {
        self.spec.voice.as_deref()
    }

    /// The endpoint, which names no socket.
    ///
    /// A mock session is opened with [`MockRealtime::open`](crate::MockRealtime::open),
    /// which builds an in-process transport and calls
    /// [`RealtimeSession::open`](via_realtime::RealtimeSession::open).
    /// `RealtimeSession::connect` would try to dial this URL and fail, which is
    /// the correct answer to "connect to the mock over TCP".
    fn url(&self) -> Result<String, RealtimeError> {
        Ok(self.spec.endpoint.clone())
    }

    fn headers(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn capabilities(&self) -> ProviderCapabilities {
        self.spec.capabilities
    }

    fn model_profile(&self) -> Option<ModelProfile> {
        self.profile.clone()
    }

    fn model_catalog(&self) -> &[ModelProfile] {
        &self.catalog
    }

    fn configuration_signature(&self) -> String {
        RealtimeIdentity::new(
            self.identity_shape,
            self.spec.key.clone(),
            self.spec.endpoint.clone(),
            self.spec.model.clone().unwrap_or_default(),
            self.spec.voice.clone().unwrap_or_default(),
            self.spec.credential.clone(),
        )
        .signature()
    }

    fn protocol(&self) -> &dyn RealtimeProtocol {
        self.protocol.as_ref()
    }

    fn preflight(&self) -> Result<(), RealtimeError> {
        if !self.spec.unsupported_model {
            return Ok(());
        }
        Err(RealtimeError::UnsupportedModel {
            id: self.spec.model.clone().unwrap_or_default(),
            label: self.spec.label.clone(),
        })
    }

    fn classify_error(&self, message: &str) -> ErrorClass {
        let lowered = message.to_lowercase();
        for rule in &self.spec.classification {
            if lowered.contains(&rule.contains.to_lowercase()) {
                return rule.class;
            }
        }
        default_test_classification(message)
    }

    /// The `session.update` payload.
    ///
    /// The shape follows `providers/dashscope.mjs:70-92` in the one way that is
    /// contract rather than cosmetic: **the first update negotiates and every
    /// later one carries only instructions and tools**, because re-sending
    /// `turn_detection` on a context refresh resets the provider's VAD
    /// mid-conversation. The GA dialect spells the modality key
    /// `output_modalities`, as `providers/s2s.mjs:67-95` does.
    fn build_session(&self, request: &SessionRequest<'_>) -> Value {
        let mut session = Map::new();
        session.insert(
            "instructions".to_owned(),
            Value::String(request.agent_context.instructions.clone()),
        );
        let calls_functions = self
            .profile
            .as_ref()
            .is_some_and(|profile| profile.model_capabilities.function_calling);
        if calls_functions && !request.agent_context.tools.is_empty() {
            session.insert(
                "tools".to_owned(),
                Value::Array(request.agent_context.tools.clone()),
            );
        }
        if !request.configured {
            let modality_key = if self.spec.dialect.correlates() {
                "output_modalities"
            } else {
                "modalities"
            };
            session.insert(modality_key.to_owned(), self.modalities());
            if let Some(profile) = &self.profile {
                if profile.model_capabilities.audio_output
                    && let Some(voice) = &self.spec.voice
                {
                    session.insert("voice".to_owned(), Value::String(voice.clone()));
                    session.insert(
                        "output_audio_format".to_owned(),
                        Value::String("pcm".to_owned()),
                    );
                }
                if profile.transport_capabilities.audio_input {
                    session.insert(
                        "input_audio_format".to_owned(),
                        Value::String("pcm".to_owned()),
                    );
                    session.insert(
                        "turn_detection".to_owned(),
                        serde_json::to_value(profile.session_defaults.turn_detection)
                            .unwrap_or(Value::Null),
                    );
                }
            }
        }
        for (key, value) in &request.agent_context.extra {
            session.insert(key.clone(), value.clone());
        }
        Value::Object(session)
    }

    /// The `response.create` body for out-of-band speech.
    ///
    /// External contract — `json-field` / *buildSpeakResponse payload*:
    /// `conversation: 'none'` is what keeps a spoken progress note out of
    /// conversation history, so it is never answered later as though the user
    /// had said it.
    fn build_speak_response(&self, content: &str) -> Value {
        let mut response = Map::new();
        response.insert("conversation".to_owned(), Value::String("none".to_owned()));
        response.insert("modalities".to_owned(), self.modalities());
        response.insert("instructions".to_owned(), Value::String(content.to_owned()));
        Value::Object(response)
    }

    /// External contract — `json-field` / *buildResultInjection payload*:
    /// a synthetic user message plus a tool-free response.
    fn build_result_injection(&self, content: &str) -> Injection {
        let mut response = self.injection_response();
        response.insert("instructions".to_owned(), Value::String(content.to_owned()));
        Injection {
            item: self.protocol.user_text_item(content),
            response: Value::Object(response),
        }
    }

    /// External contract — `prompt-text` / *backend permission injection item
    /// text*.
    fn build_permission_injection(&self, permission: &PermissionRequest) -> Injection {
        let text = [
            format!("<{PERMISSION_REQUEST_TAG}>"),
            format!("authorization_id={}", permission.id),
            format!("operation={}", permission.summary),
            format!("</{PERMISSION_REQUEST_TAG}>"),
        ]
        .join("\n");
        Injection {
            item: self.protocol.user_text_item(&text),
            response: Value::Object(self.injection_response()),
        }
    }

    fn missing_configuration_message(&self, locale: Locale) -> String {
        format(
            locale,
            keys::REALTIME_MISSING_CONFIGURATION,
            &[("label", &self.spec.label)],
        )
    }

    fn connect_timeout_message(&self, locale: Locale) -> String {
        format(
            locale,
            keys::REALTIME_CONNECT_TIMEOUT,
            &[("label", &self.spec.label)],
        )
    }
}

/// Both dialects, in the order a picker should offer them.
///
/// Exists so a caller — or `via-conformance` — can walk the set rather than
/// name each variant, which is what keeps a third dialect from being added
/// without anyone noticing.
#[must_use]
pub const fn dialects() -> [MockDialect; 2] {
    [MockDialect::Beta, MockDialect::Ga]
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use via_realtime::{AgentContext, validate_realtime_provider};

    use super::*;

    fn built(spec: MockProviderSpec) -> MockProvider {
        MockProvider::new(spec)
    }

    #[test]
    fn the_defaults_come_from_the_catalog_row_not_from_here() {
        let row = catalog_row().expect("the catalog has a `mock` row");
        let spec = MockProviderSpec::default();
        assert_eq!(spec.key, row.key);
        assert_eq!(spec.label, row.label);
        assert_eq!(spec.aliases, row.aliases.to_vec());
        assert_eq!(spec.identity_shape(), row.identity_shape);
        // The fallback label is only ever the unreachable arm.
        assert_eq!(row.label, FALLBACK_LABEL);
    }

    #[test]
    fn the_default_provider_validates() {
        assert_eq!(
            validate_realtime_provider(&built(MockProviderSpec::default())),
            Ok(())
        );
        assert_eq!(
            validate_realtime_provider(&built(MockProviderSpec::dictation())),
            Ok(())
        );
        assert_eq!(
            validate_realtime_provider(&built(MockProviderSpec::speech_to_speech_shaped())),
            Ok(())
        );
    }

    #[test]
    fn build_refuses_a_provider_the_registry_would_refuse() {
        let error = MockProviderSpec::default()
            .with_key("Mock")
            .build()
            .map(|_| ())
            .expect_err("refused");
        assert_eq!(
            error,
            RealtimeError::ProviderKeyInvalid { key: "Mock".into() }
        );
    }

    #[test]
    fn the_sample_rates_are_the_catalogued_realtime_pair() {
        let spec = MockProviderSpec::default();
        assert_eq!(spec.input_sample_rate, via_audio::SampleRate::HZ_16000.hz());
        assert_eq!(
            spec.output_sample_rate,
            via_audio::SampleRate::HZ_24000.hz()
        );
    }

    #[test]
    fn a_dictation_provider_mounts_no_model_and_publishes_no_catalog() {
        let provider = built(MockProviderSpec::dictation());
        assert_eq!(provider.model(), None);
        assert_eq!(provider.voice(), None);
        assert_eq!(provider.model_profile(), None);
        assert!(provider.model_catalog().is_empty());
        // `describe_provider` turns an empty catalog into `realtimeModelIds: null`.
        assert_eq!(
            via_realtime::describe_provider(&provider).realtime_model_ids,
            None
        );
    }

    #[test]
    fn a_conversing_provider_publishes_its_one_model() {
        let provider = built(MockProviderSpec::default());
        assert_eq!(provider.model(), Some(MOCK_MODEL));
        assert_eq!(
            via_realtime::describe_provider(&provider).realtime_model_ids,
            Some(vec![MOCK_MODEL.to_owned()])
        );
        // The voice is the local family's own, not a literal restated here.
        assert_eq!(
            provider.voice(),
            Some(
                local_realtime_model_profile(MOCK_MODEL)
                    .session_defaults
                    .voice
                    .as_ref()
            )
        );
    }

    #[test]
    fn the_signature_is_the_catalog_hash_over_the_endpoint_only_shape() {
        let provider = built(MockProviderSpec::default());
        let expected = RealtimeIdentity::new(
            IdentityShape::EndpointOnly,
            MOCK_PROVIDER_KEY,
            MOCK_ENDPOINT,
            MOCK_MODEL,
            "",
            "",
        )
        .signature();
        assert_eq!(provider.configuration_signature(), expected);
        // `EndpointOnly` drops model and voice, so changing the model cannot
        // change the digest — which is the shape's whole point.
        let other = built(MockProviderSpec::default().with_model(Some("something-else")));
        assert_eq!(other.configuration_signature(), expected);
        // The endpoint does change it.
        let moved = built(MockProviderSpec {
            endpoint: "mock://elsewhere".into(),
            ..MockProviderSpec::default()
        });
        assert_ne!(moved.configuration_signature(), expected);
    }

    #[test]
    fn the_first_session_update_negotiates_and_later_ones_do_not() {
        let provider = built(MockProviderSpec::default());
        let context = AgentContext {
            instructions: "be brief".into(),
            tools: vec![json!({ "type": "function" })],
            ..AgentContext::default()
        };
        let first = provider.build_session(&SessionRequest {
            configured: false,
            agent_context: &context,
        });
        assert_eq!(first["modalities"], json!(["text", "audio"]));
        assert_eq!(first["input_audio_format"], json!("pcm"));
        assert_eq!(first["output_audio_format"], json!("pcm"));
        assert_eq!(first["turn_detection"]["type"], json!("server_vad"));

        let later = provider.build_session(&SessionRequest {
            configured: true,
            agent_context: &context,
        });
        let keys: Vec<&str> = later
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["instructions", "tools"]);
    }

    #[test]
    fn the_ga_dialect_spells_the_modality_key_its_own_way() {
        let provider = built(MockProviderSpec::speech_to_speech_shaped());
        let session = provider.build_session(&SessionRequest {
            configured: false,
            agent_context: &AgentContext::default(),
        });
        assert!(session.get("modalities").is_none());
        assert_eq!(session["output_modalities"], json!(["text", "audio"]));
    }

    #[test]
    fn a_provider_with_no_model_declares_no_tools_and_no_negotiation_extras() {
        let provider = built(MockProviderSpec::dictation());
        let context = AgentContext {
            tools: vec![json!({ "type": "function" })],
            ..AgentContext::default()
        };
        let session = provider.build_session(&SessionRequest {
            configured: false,
            agent_context: &context,
        });
        assert!(
            session.get("tools").is_none(),
            "no profile, no function calling"
        );
        assert!(session.get("turn_detection").is_none());
        assert_eq!(session["modalities"], json!(["audio"]));
    }

    #[test]
    fn agent_context_extra_lands_on_the_session_payload() {
        let provider = built(MockProviderSpec::default());
        let mut context = AgentContext::default();
        context
            .extra
            .insert("custom".to_owned(), json!({ "nested": true }));
        let session = provider.build_session(&SessionRequest {
            configured: true,
            agent_context: &context,
        });
        assert_eq!(session["custom"], json!({ "nested": true }));
    }

    #[test]
    fn out_of_band_speech_stays_out_of_conversation_history() {
        let response = built(MockProviderSpec::default()).build_speak_response("nearly done");
        assert_eq!(response["conversation"], json!("none"));
        assert_eq!(response["instructions"], json!("nearly done"));
    }

    #[test]
    fn a_result_injection_is_a_user_message_plus_a_tool_free_response() {
        let injection = built(MockProviderSpec::default()).build_result_injection("it is done");
        assert_eq!(injection.item["type"], json!("message"));
        assert_eq!(injection.item["role"], json!("user"));
        assert_eq!(
            injection.item["content"][0],
            json!({ "type": "input_text", "text": "it is done" })
        );
        assert_eq!(injection.response["tool_choice"], json!("none"));
    }

    #[test]
    fn a_permission_injection_carries_the_catalogued_tag() {
        let injection =
            built(MockProviderSpec::default()).build_permission_injection(&PermissionRequest {
                id: "auth-1".into(),
                summary: "write to disk".into(),
            });
        let text = injection.item["content"][0]["text"]
            .as_str()
            .expect("text")
            .to_owned();
        assert_eq!(
            text,
            "<backend_permission_request>\nauthorization_id=auth-1\noperation=write to disk\n</backend_permission_request>"
        );
        assert_eq!(injection.response["tool_choice"], json!("none"));
    }

    #[test]
    fn classification_falls_through_to_the_shared_corpus() {
        let provider = built(MockProviderSpec::default());
        assert_eq!(
            provider.classify_error(crate::script::messages::RESPONSE_SLOT_BUSY),
            ErrorClass::ResponseSlotBusy
        );
        assert_eq!(
            provider.classify_error(crate::script::messages::INPUT_BUSY),
            ErrorClass::InputBusy
        );
        assert_eq!(provider.classify_error("who knows"), ErrorClass::Other);
    }

    #[test]
    fn a_script_rule_is_checked_before_the_shared_corpus() {
        let provider = built(
            MockProviderSpec::default()
                .classifying("who knows", ErrorClass::Fatal)
                // Case-insensitive, and it wins over the shared corpus.
                .classifying("USER IS SPEAKING", ErrorClass::CapacityBusy),
        );
        assert_eq!(provider.classify_error("Who Knows?"), ErrorClass::Fatal);
        assert_eq!(
            provider.classify_error(crate::script::messages::INPUT_BUSY),
            ErrorClass::CapacityBusy
        );
    }

    #[test]
    fn preflight_refuses_before_anything_else_when_the_model_is_unsupported() {
        let provider = built(MockProviderSpec::default().unsupported());
        assert_eq!(
            provider.preflight(),
            Err(RealtimeError::UnsupportedModel {
                id: MOCK_MODEL.into(),
                label: MockProviderSpec::default().label,
            })
        );
        assert_eq!(built(MockProviderSpec::default()).preflight(), Ok(()));
    }

    #[test]
    fn the_two_person_facing_sentences_come_from_the_catalog() {
        let provider = built(MockProviderSpec::default());
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            let missing = provider.missing_configuration_message(locale);
            let timeout = provider.connect_timeout_message(locale);
            assert!(missing.contains(provider.label()), "{locale:?} {missing}");
            assert!(timeout.contains(provider.label()), "{locale:?} {timeout}");
            assert_ne!(missing, timeout);
            assert!(!missing.contains('{'), "{locale:?} left a placeholder");
            assert!(!timeout.contains('{'), "{locale:?} left a placeholder");
        }
    }

    #[test]
    fn the_dialect_chooses_a_shipped_adapter_rather_than_implementing_one() {
        let beta = built(MockProviderSpec::default().with_dialect(MockDialect::Beta));
        // The beta dialect has no portable correlation contract, whatever a
        // provider declares.
        assert_eq!(
            beta.protocol()
                .correlate_response_create(json!({ "type": "response.create" }), "request-1"),
            json!({ "type": "response.create" })
        );
        assert!(!MockDialect::Beta.correlates());

        let ga = built(MockProviderSpec::default().with_dialect(MockDialect::Ga));
        let correlated = ga
            .protocol()
            .correlate_response_create(json!({ "type": "response.create" }), "request-1");
        assert_eq!(
            ga.protocol().response_correlation_id(&correlated),
            "request-1"
        );
        assert!(MockDialect::Ga.correlates());
        assert_eq!(dialects(), [MockDialect::Beta, MockDialect::Ga]);
    }

    #[test]
    fn the_spec_round_trips_through_json() {
        for spec in [
            MockProviderSpec::default(),
            MockProviderSpec::dictation(),
            MockProviderSpec::speech_to_speech_shaped()
                .with_visibility(Visibility::GatewayOnly)
                .with_aliases(["m"])
                .with_response_start_timeout(Some(Duration::from_secs(60)))
                .classifying("boom", ErrorClass::Fatal)
                .unsupported(),
        ] {
            let text = serde_json::to_string(&spec).expect("serialize");
            assert_eq!(
                serde_json::from_str::<MockProviderSpec>(&text).expect("parse"),
                spec
            );
        }
    }

    #[test]
    fn a_spec_fixture_may_name_only_what_it_changes() {
        let spec: MockProviderSpec = serde_json::from_value(json!({
            "key": "mock",
            "label": "Mock Realtime",
            "inputSampleRate": 16_000,
            "outputSampleRate": 24_000,
            "capabilities": {
                "acknowledgesSessionUpdate": false,
                "singleResponseSlot": true,
                "responseMetadataCorrelation": true,
                "perResponseInstructions": true,
                "conversationItemIdEcho": false,
            },
        }))
        .expect("parse");
        assert!(spec.configured, "`configured` defaults to true");
        assert_eq!(spec.dialect, MockDialect::Beta);
        assert_eq!(
            spec.capabilities.as_array(),
            [false, true, true, true, false]
        );
    }

    #[test]
    fn a_response_start_timeout_of_zero_is_refused_by_validation() {
        let error = MockProviderSpec::default()
            .with_response_start_timeout(Some(Duration::ZERO))
            .build()
            .map(|_| ())
            .expect_err("refused");
        assert_eq!(
            error,
            RealtimeError::ProviderResponseTimeoutInvalid {
                key: MOCK_PROVIDER_KEY.into()
            }
        );
    }
}
