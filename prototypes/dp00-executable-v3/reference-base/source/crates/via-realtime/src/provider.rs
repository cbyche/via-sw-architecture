//! One realtime service, behind one trait.
//!
//! Ported from the provider half of
//! `server/src/voice/providers/provider-registry.mjs` — the ten methods in
//! `PROVIDER_METHODS`, the two sample rates, the optional response-start
//! timeout, the capability declaration, the visibility, the aliases and the
//! model profile.
//!
//! # What the type system absorbed
//!
//! Upstream spends 190 lines proving that a JavaScript object literal really is
//! a provider: that each of ten names is a function, that each of five
//! capability flags is a boolean and no sixth flag exists, that each of twelve
//! protocol methods is a function, and that each of twelve capability flags on
//! the model profile is a boolean. In Rust the trait *is* those lists and the
//! types *are* those checks, so all four disappear at compile time.
//!
//! What is left is what a well-typed provider can still get wrong, and
//! [`validate_realtime_provider`] keeps every one of them:
//!
//! | Check | Upstream | Why it survives |
//! | --- | --- | --- |
//! | non-empty key and label | `:153-155` | `&str` can be empty |
//! | key matches `^[a-z0-9][a-z0-9-]*$` | `:156-158` | it is what a client may send on `connect` |
//! | a usable input / output sample rate | `:164-169` | `0` is a number |
//! | a positive response-start timeout | `:170-180` | `Duration::ZERO` exists |
//! | no blank alias | `:208-216` | `&str` can be blank |
//! | a model profile with a real id, label and voice | `:76-128` | `Cow<str>` can be blank |
//!
//! The one strengthening is deliberate and recorded: upstream accepts
//! `inputSampleRate: 0` because `Number.isFinite(0)` is true. VIA refuses it. A
//! zero capture rate is division by zero in the resampler and a `voice.ready`
//! frame that tells every client to capture at 0 Hz.

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use via_catalog::ModelProfile;
use via_i18n::Locale;

use crate::capabilities::ProviderCapabilities;
use crate::error::RealtimeError;
use crate::event_error::ErrorClass;
use crate::protocol::RealtimeProtocol;

/// The ten provider methods upstream validates by name.
///
/// External contract — `providers/provider-registry.mjs:1-12`. One of them is
/// test-locked by name (`/缺少 model\(\)/`). They are trait methods here; the
/// list survives for `via-conformance`.
pub const PROVIDER_METHODS: [&str; 10] = [
    "model",
    "voice",
    "isConfigured",
    "url",
    "headers",
    "classifyError",
    "buildSession",
    "buildSpeakResponse",
    "buildResultInjection",
    "buildPermissionInjection",
];

/// Whether a provider appears in the client-facing provider list.
///
/// External contract — `provider-registry.mjs:55`, `VISIBILITIES`. The default
/// is [`Public`](Self::Public); a `gateway-only` provider still resolves by name
/// (so the Gateway can be pointed at it) but never shows up in a picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Visibility {
    /// Listed in client-facing provider discovery.
    #[default]
    Public,
    /// Resolvable by name, never listed.
    GatewayOnly,
}

impl Visibility {
    /// The wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::GatewayOnly => "gateway-only",
        }
    }
}

impl core::fmt::Display for Visibility {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What the session tells a provider about the conversation it is configuring.
///
/// Upstream's `agentContext` is a free-form object assembled in
/// `server/src/voice/` and read by `buildFrontendInstructions` and
/// `frontendTools`. Neither of those lives in `via-realtime` —
/// `docs/architecture.md` §9 puts the frontend prompt and the tool catalog in
/// `via-voice` — so the composed results arrive here already built, and
/// [`extra`](Self::extra) carries anything a particular provider needs on top.
///
/// The split is what keeps this crate free of a dependency on the prompt layer
/// while leaving each provider free to shape the payload: the GA dialect
/// flattens [`tools`](Self::tools) into `{ type, name, description, parameters }`
/// in `build_session`, and the flattening stays in the provider exactly as it
/// does upstream.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentContext {
    /// The composed `session.instructions` text.
    pub instructions: String,
    /// The tool catalog in the beta `{ type: 'function', function: {…} }` shape.
    ///
    /// Empty when the session declares no tools — which is every `dictation`
    /// session and any model whose profile says it cannot call functions.
    pub tools: Vec<Value>,
    /// The `<recent_conversation>` block to restore once the session is ready,
    /// or `None` when there is nothing to restore.
    pub recent_context: Option<String>,
    /// Anything else a provider reads.
    pub extra: Map<String, Value>,
}

/// A shallow patch over an [`AgentContext`].
///
/// Upstream's `updateAgentContext(patch)` is a one-level object spread —
/// `{ ...this.agentContext, ...patch }` — so a field that is absent from the
/// patch keeps its value and a field that is present replaces it wholesale.
/// `Option` is that, exactly.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentContextPatch {
    /// Replace the instructions.
    pub instructions: Option<String>,
    /// Replace the tool catalog.
    pub tools: Option<Vec<Value>>,
    /// Replace the restorable recent conversation.
    pub recent_context: Option<Option<String>>,
    /// Merge into [`AgentContext::extra`], key by key — the same one-level
    /// spread the outer patch is.
    pub extra: Map<String, Value>,
}

impl AgentContext {
    /// Apply a shallow patch.
    pub fn apply(&mut self, patch: AgentContextPatch) {
        if let Some(instructions) = patch.instructions {
            self.instructions = instructions;
        }
        if let Some(tools) = patch.tools {
            self.tools = tools;
        }
        if let Some(recent) = patch.recent_context {
            self.recent_context = recent;
        }
        for (key, value) in patch.extra {
            self.extra.insert(key, value);
        }
    }
}

/// What `build_session` is asked for.
///
/// Upstream `buildSession({ configured, agentContext })`.
#[derive(Debug, Clone, Copy)]
pub struct SessionRequest<'a> {
    /// Whether this session has already been configured once.
    ///
    /// The distinction is contract: the first `session.update` negotiates
    /// modalities, voice, audio formats and turn detection, and **every
    /// subsequent one carries only instructions and tools**. Re-sending
    /// `turn_detection` on a context refresh would reset the provider's VAD
    /// mid-conversation.
    pub configured: bool,
    /// The conversation context.
    pub agent_context: &'a AgentContext,
}

/// A conversation item plus the response that should follow it.
///
/// Upstream `buildResultInjection` / `buildPermissionInjection` both return
/// `{ item, response }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Injection {
    /// The conversation item to create first.
    pub item: Value,
    /// The `response.create` body to send once the item is acknowledged.
    pub response: Value,
}

/// A backend authorization request, as the model is told about it.
///
/// The rendered item text is catalogued prompt vocabulary —
/// `<backend_permission_request>` with `authorization_id` and `operation` — and
/// `config/frontend-agent/PROMPT.md` references the tag by name, so the
/// rendering lives in the provider and this is only its input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionRequest {
    /// The authorization id the model must quote back.
    pub id: String,
    /// A one-line description of the operation.
    pub summary: String,
}

/// How a turn's input parts become realtime frames.
///
/// Upstream `projectUserInput` returns
/// `{ beforeEvents?, conversationItem?, afterEvents? }` or `null`. All three
/// parts are optional and the order is the contract: everything in
/// [`before_events`](Self::before_events) is written, then the item is created
/// **and awaited**, then [`after_events`](Self::after_events).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InputProjection {
    /// Frames to write before the conversation item.
    pub before_events: Vec<Value>,
    /// The conversation item to create and wait for.
    pub conversation_item: Option<Value>,
    /// Frames to write after the item is acknowledged.
    pub after_events: Vec<Value>,
}

impl InputProjection {
    /// The default projection: one user text item and nothing else.
    #[must_use]
    pub fn text(item: Value) -> Self {
        Self {
            conversation_item: Some(item),
            ..Self::default()
        }
    }

    /// Whether this projection would write anything at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.before_events.is_empty()
            && self.conversation_item.is_none()
            && self.after_events.is_empty()
    }
}

/// What a provider is given when it is asked to project input parts itself.
#[derive(Debug, Clone, Copy)]
pub struct InputProjectionRequest<'a> {
    /// The normalized input parts, as the client sent them.
    pub parts: &'a Value,
    /// Projection options — upstream's `{ accompaniesVoice }` and anything a
    /// provider adds.
    pub options: &'a Value,
    /// The active model profile's capabilities, when there is a profile.
    pub model_profile: Option<&'a ModelProfile>,
}

/// Which connection a per-connection protocol adapter is being built for.
///
/// Upstream `createProtocol({ connectionId, provider })`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionInfo {
    /// A fresh uuid, one per `connect()`.
    pub connection_id: String,
}

/// One realtime service.
///
/// Everything about *which service* — endpoint, credential, model, voice,
/// session payload, error corpus. The dialect the bytes are in is
/// [`RealtimeProtocol`]'s, reached through [`protocol`](Self::protocol).
///
/// # `dictation` is why half of this is optional
///
/// `docs/architecture.md` §2: `dictation` *"mounts no model at all … which is
/// why `via-realtime`'s provider trait must not assume a model turn exists."*
/// So [`model`](Self::model) and [`model_profile`](Self::model_profile) are
/// `Option`, [`voice`](Self::voice) is `Option`, [`model_catalog`](Self::model_catalog)
/// may be empty, and nothing in the session requires a `response.*` event ever
/// to arrive. A plain streaming ASR is a valid provider.
pub trait RealtimeProvider: Send + Sync + core::fmt::Debug {
    /// The canonical key, as a client sends it on `connect` and as
    /// `/api/health` echoes it. Must match `^[a-z0-9][a-z0-9-]*$`.
    fn key(&self) -> &str;

    /// The human label, echoed in `/api/health` and interpolated into every
    /// connection error.
    fn label(&self) -> &str;

    /// Alternate spellings that resolve to [`key`](Self::key).
    ///
    /// `dashscope` keeps `qwen` and `speech-to-speech` keeps `s2s`; both are
    /// KEEP under `docs/rebrand.md` because clients and configuration files
    /// already carry them.
    fn aliases(&self) -> Vec<String> {
        Vec::new()
    }

    /// Whether this provider appears in client-facing discovery.
    fn visibility(&self) -> Visibility {
        Visibility::Public
    }

    /// The rate the client must capture at.
    ///
    /// External contract — 16 000 for both upstream providers. It reaches every
    /// client on `voice.ready`, and the wake-word detector and the resampler
    /// both key off it.
    fn input_sample_rate(&self) -> u32;

    /// The rate the provider's audio output arrives at. 24 000 for both
    /// upstream providers.
    fn output_sample_rate(&self) -> u32;

    /// How long to wait for `response.created` before giving up on a response.
    ///
    /// `None` takes the session default of 30 s. A fully local
    /// ASR → LLM → TTS pipeline can take substantially longer to produce its
    /// first response event, which is why `speech-to-speech` declares 60 s.
    fn response_start_timeout(&self) -> Option<Duration> {
        None
    }

    /// Whether the credentials and endpoint this provider needs are present.
    ///
    /// `list(configured_only)` filters on it, so an unconfigured provider is
    /// invisible in a client's picker rather than a failing choice.
    fn is_configured(&self) -> bool;

    /// The active model id, or `None` for a provider that does not choose one.
    fn model(&self) -> Option<&str>;

    /// The active voice id, or `None`.
    fn voice(&self) -> Option<&str>;

    /// The WebSocket URL to open.
    ///
    /// # Errors
    ///
    /// Whatever makes the endpoint unbuildable — an unusable configured base
    /// URL, an unresolvable workspace.
    fn url(&self) -> Result<String, RealtimeError>;

    /// Extra headers for the WebSocket upgrade.
    ///
    /// A `Vec` rather than a map because header order is observable on the wire
    /// and a provider may legitimately send two headers with the same name.
    fn headers(&self) -> Vec<(String, String)>;

    /// What this provider's Realtime implementation actually does.
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::DEFAULT
    }

    /// The profile of the active model, or `None`.
    fn model_profile(&self) -> Option<ModelProfile> {
        None
    }

    /// Every model this provider can be pointed at, in catalog order.
    ///
    /// Empty for a provider whose model is chosen upstream of VIA. The
    /// distinction is published: an empty catalog becomes `realtimeModelIds:
    /// null` in the provider descriptor, which is what upstream emits for
    /// `speech-to-speech`.
    fn model_catalog(&self) -> &[ModelProfile] {
        &[]
    }

    /// The `sha256` hash clients compare to detect that the voice configuration
    /// changed under them.
    ///
    /// Published as `/api/health.realtimeConfigurationSignature`.
    /// [`via_catalog::RealtimeIdentity`] owns the hash input and its key order;
    /// a provider builds the identity and calls `signature()` on it.
    fn configuration_signature(&self) -> String;

    /// The dialect this provider speaks.
    fn protocol(&self) -> &dyn RealtimeProtocol;

    /// A per-connection dialect adapter, when this provider needs one.
    ///
    /// Upstream's optional `createProtocol({ connectionId, provider })`. `None`
    /// means "use [`protocol`](Self::protocol)"; anything else is used for this
    /// connection only, so an adapter may close over the connection id — which
    /// is how a multiplexing proxy tags its frames.
    fn create_protocol(&self, _connection: &ConnectionInfo) -> Option<Arc<dyn RealtimeProtocol>> {
        None
    }

    /// Refuse to connect at all, before a socket is opened.
    ///
    /// The Rust seam for upstream's `modelProfile?.family === 'unknown'` gate
    /// (`realtime-provider.mjs:134-140`). VIA's model catalog has no `unknown`
    /// family — `via-catalog` answers an unrecognised id with an error rather
    /// than an all-capabilities-false profile — so the check cannot be expressed
    /// as "look at the profile the provider returned"; the provider raises it.
    ///
    /// The ordering is contract: it runs **before**
    /// [`is_configured`](Self::is_configured), and both run before the socket,
    /// so `via-realtime` never opens a connection it already knows is useless.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::UnsupportedModel`], or whatever else a provider knows in
    /// advance.
    fn preflight(&self) -> Result<(), RealtimeError> {
        Ok(())
    }

    /// Classify one of this provider's own error strings.
    ///
    /// The input is `event.error.message || event.message`, not the composed
    /// message — see [`crate::realtime_event_classification_text`].
    fn classify_error(&self, message: &str) -> ErrorClass;

    /// The `session.update` payload.
    fn build_session(&self, request: &SessionRequest<'_>) -> Value;

    /// The `response.create` body for out-of-band speech.
    ///
    /// Both upstream providers set `conversation: 'none'` so the utterance stays
    /// out of conversation history.
    fn build_speak_response(&self, content: &str) -> Value;

    /// The item and response that deliver a finished background result.
    fn build_result_injection(&self, content: &str) -> Injection;

    /// The item and response that ask the user for an authorization.
    fn build_permission_injection(&self, permission: &PermissionRequest) -> Injection;

    /// Project input parts into realtime frames, when this provider wants to.
    ///
    /// `None` means "use the default projection", which the caller supplies —
    /// upstream's default is `frontendInputProjection`, which lives in the
    /// prompt layer rather than here.
    fn project_user_input(&self, _request: &InputProjectionRequest<'_>) -> Option<InputProjection> {
        None
    }

    /// What to say when [`is_configured`](Self::is_configured) is false.
    fn missing_configuration_message(&self, locale: Locale) -> String;

    /// What to say when the connect budget runs out.
    fn connect_timeout_message(&self, locale: Locale) -> String;
}

/// Trim and lowercase, the way upstream's `cleanKey` does.
///
/// External contract — `provider-registry.mjs:57-59`: alias and key lookup is
/// `String(v).trim().toLowerCase()`, which is why `resolve('CUSTOM')` finds
/// `custom`.
#[must_use]
pub fn clean_key(value: &str) -> String {
    value.trim().to_lowercase()
}

/// Whether a string is a legal provider key.
///
/// External contract — `provider-registry.mjs:156`: `/^[a-z0-9][a-z0-9-]*$/`.
/// Written out rather than compiled because it is four conditions, and a regex
/// here would need a fallible `LazyLock` inside a validation path.
#[must_use]
pub fn is_valid_provider_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Check everything about a provider that the type system could not.
///
/// See the module docs for what upstream checked that Rust already guarantees.
///
/// # Errors
///
/// The first failure, as a [`RealtimeError`]. The order is upstream's, so a
/// provider with two faults reports the same one it would have reported there.
pub fn validate_realtime_provider(provider: &dyn RealtimeProvider) -> Result<(), RealtimeError> {
    let key = provider.key();
    if key.is_empty() || provider.label().is_empty() {
        return Err(RealtimeError::ProviderNeedsKeyAndLabel);
    }
    if !is_valid_provider_key(key) {
        return Err(RealtimeError::ProviderKeyInvalid {
            key: key.to_owned(),
        });
    }
    if provider.input_sample_rate() == 0 {
        return Err(RealtimeError::ProviderMissingInputSampleRate {
            key: key.to_owned(),
        });
    }
    if provider.output_sample_rate() == 0 {
        return Err(RealtimeError::ProviderMissingOutputSampleRate {
            key: key.to_owned(),
        });
    }
    if provider
        .response_start_timeout()
        .is_some_and(|timeout| timeout.is_zero())
    {
        return Err(RealtimeError::ProviderResponseTimeoutInvalid {
            key: key.to_owned(),
        });
    }
    if provider
        .aliases()
        .iter()
        .any(|alias| clean_key(alias).is_empty())
    {
        return Err(RealtimeError::ProviderAliasesInvalid {
            key: key.to_owned(),
        });
    }
    if let Some(profile) = provider.model_profile() {
        if profile.id.trim().is_empty() || profile.label.trim().is_empty() {
            return Err(RealtimeError::ProviderModelProfileInvalid {
                key: key.to_owned(),
            });
        }
        if profile.session_defaults.voice.trim().is_empty() {
            return Err(RealtimeError::ProviderSessionDefaultsIncomplete {
                key: key.to_owned(),
            });
        }
    }
    Ok(())
}

/// Validate a provider and hand it back.
///
/// Upstream's `defineRealtimeProvider`, and the front door of the host-extension
/// seam — see [`crate::extension`].
///
/// # Errors
///
/// Whatever [`validate_realtime_provider`] refuses.
pub fn define_realtime_provider(
    provider: Arc<dyn RealtimeProvider>,
) -> Result<Arc<dyn RealtimeProvider>, RealtimeError> {
    validate_realtime_provider(provider.as_ref())?;
    Ok(provider)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::testing::TestProvider;

    #[test]
    fn a_key_must_start_alphanumeric_and_stay_lowercase() {
        for key in [
            "a",
            "0",
            "dashscope",
            "speech-to-speech",
            "local-omni",
            "a-1",
        ] {
            assert!(is_valid_provider_key(key), "{key}");
        }
        for key in [
            "",
            "-leading",
            "Upper",
            "with_underscore",
            "with space",
            "trailing.",
            "ünicode",
        ] {
            assert!(!is_valid_provider_key(key), "{key}");
        }
    }

    #[test]
    fn clean_key_trims_and_lowercases() {
        assert_eq!(clean_key("  CUSTOM \n"), "custom");
        assert_eq!(clean_key("   "), "");
    }

    #[test]
    fn a_well_formed_provider_validates() {
        let provider = TestProvider::new("custom-provider");
        assert_eq!(validate_realtime_provider(&provider), Ok(()));
    }

    #[test]
    fn each_remaining_check_refuses_with_its_own_error() {
        assert_eq!(
            validate_realtime_provider(&TestProvider::new("")),
            Err(RealtimeError::ProviderNeedsKeyAndLabel)
        );
        assert_eq!(
            validate_realtime_provider(&TestProvider::new("ok").with_label("")),
            Err(RealtimeError::ProviderNeedsKeyAndLabel)
        );
        assert_eq!(
            validate_realtime_provider(&TestProvider::new("Upper")),
            Err(RealtimeError::ProviderKeyInvalid {
                key: "Upper".into()
            })
        );
        assert_eq!(
            validate_realtime_provider(&TestProvider::new("ok").with_input_sample_rate(0)),
            Err(RealtimeError::ProviderMissingInputSampleRate { key: "ok".into() })
        );
        assert_eq!(
            validate_realtime_provider(&TestProvider::new("ok").with_output_sample_rate(0)),
            Err(RealtimeError::ProviderMissingOutputSampleRate { key: "ok".into() })
        );
        assert_eq!(
            validate_realtime_provider(
                &TestProvider::new("ok").with_response_start_timeout(Some(Duration::ZERO))
            ),
            Err(RealtimeError::ProviderResponseTimeoutInvalid { key: "ok".into() })
        );
        assert_eq!(
            validate_realtime_provider(&TestProvider::new("ok").with_aliases(["  "])),
            Err(RealtimeError::ProviderAliasesInvalid { key: "ok".into() })
        );
    }

    #[test]
    fn a_blank_model_profile_field_is_refused() {
        let mut profile = via_catalog::local_realtime_model_profile("local-model");
        profile.id = "  ".into();
        assert_eq!(
            validate_realtime_provider(&TestProvider::new("ok").with_model_profile(Some(profile))),
            Err(RealtimeError::ProviderModelProfileInvalid { key: "ok".into() })
        );

        let mut profile = via_catalog::local_realtime_model_profile("local-model");
        profile.label = String::new().into();
        assert_eq!(
            validate_realtime_provider(&TestProvider::new("ok").with_model_profile(Some(profile))),
            Err(RealtimeError::ProviderModelProfileInvalid { key: "ok".into() })
        );

        let mut profile = via_catalog::local_realtime_model_profile("local-model");
        profile.session_defaults.voice = " ".into();
        assert_eq!(
            validate_realtime_provider(&TestProvider::new("ok").with_model_profile(Some(profile))),
            Err(RealtimeError::ProviderSessionDefaultsIncomplete { key: "ok".into() })
        );
    }

    #[test]
    fn a_provider_with_no_model_profile_is_valid() {
        // `speech-to-speech` has none at all, and `describeActiveRealtime`
        // publishes `modelProfile: null` for it.
        let provider = TestProvider::new("ok").with_model_profile(None);
        assert_eq!(validate_realtime_provider(&provider), Ok(()));
        assert!(provider.model_profile().is_none());
    }

    #[test]
    fn define_returns_the_provider_it_validated() {
        let provider: Arc<dyn RealtimeProvider> = Arc::new(TestProvider::new("defined"));
        let defined = define_realtime_provider(Arc::clone(&provider)).expect("valid");
        assert!(Arc::ptr_eq(&provider, &defined));

        let broken: Arc<dyn RealtimeProvider> = Arc::new(TestProvider::new("Broken"));
        assert_eq!(
            define_realtime_provider(broken).map(|_| ()),
            Err(RealtimeError::ProviderKeyInvalid {
                key: "Broken".into()
            })
        );
    }

    #[test]
    fn a_context_patch_is_a_one_level_spread() {
        let mut context = AgentContext {
            instructions: "old".into(),
            tools: vec![json!({ "name": "a" })],
            recent_context: Some("recent".into()),
            extra: Map::from_iter([
                ("keep".to_owned(), json!(1)),
                ("replace".to_owned(), json!(1)),
            ]),
        };
        context.apply(AgentContextPatch {
            instructions: Some("new".into()),
            extra: Map::from_iter([("replace".to_owned(), json!(2))]),
            ..Default::default()
        });

        assert_eq!(context.instructions, "new");
        // Absent fields keep their value.
        assert_eq!(context.tools, vec![json!({ "name": "a" })]);
        assert_eq!(context.recent_context.as_deref(), Some("recent"));
        assert_eq!(context.extra["keep"], json!(1));
        assert_eq!(context.extra["replace"], json!(2));
    }

    #[test]
    fn a_patch_can_clear_the_recent_context() {
        let mut context = AgentContext {
            recent_context: Some("recent".into()),
            ..Default::default()
        };
        context.apply(AgentContextPatch {
            recent_context: Some(None),
            ..Default::default()
        });
        assert_eq!(context.recent_context, None);
    }

    #[test]
    fn the_default_projection_is_one_item() {
        let projection = InputProjection::text(json!({ "type": "message" }));
        assert!(!projection.is_empty());
        assert!(projection.before_events.is_empty());
        assert!(projection.after_events.is_empty());
        assert!(InputProjection::default().is_empty());
    }

    #[test]
    fn visibility_defaults_to_public() {
        assert_eq!(Visibility::default(), Visibility::Public);
        assert_eq!(Visibility::GatewayOnly.as_str(), "gateway-only");
        assert_eq!(
            serde_json::to_value(Visibility::GatewayOnly).expect("serialize"),
            json!("gateway-only")
        );
    }
}
