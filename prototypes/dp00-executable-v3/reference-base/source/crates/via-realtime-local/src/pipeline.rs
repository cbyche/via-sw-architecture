//! `local-omni:pipeline` — the componentized path, as one realtime provider.
//!
//! `docs/architecture.md` §7 and §16: *"the 100% Rust componentized path first,
//! because it runs on the dev box and can be exercised."* This is that path's
//! front door — a plain [`RealtimeProvider`] over the machine in
//! [`crate::machine`], reached through the [`Transport`] seam `via-realtime` put
//! there for exactly this.
//!
//! The session above it is the **real** [`RealtimeSession`]: the same two owning
//! tasks, the same correlation, the same two watchdogs, the same busy-retry
//! ladder. Nothing about Layer 1 is re-implemented here.
//!
//! # The one thing this provider must not get wrong
//!
//! `docs/architecture.md` §7's *"one bug not to port"*. Upstream answers an
//! unknown model id with an all-capabilities-false profile and then gates
//! `session.turn_detection` on `transportCapabilities.audioInput`, so the
//! session connects and never hears the user. A local model id is by definition
//! unknown to that table.
//!
//! Both halves of the fix are load-bearing here:
//!
//! - [`model_profile`](RealtimeProvider::model_profile) is
//!   [`local_realtime_model_profile`], whose flags are **real** —
//!   `audio_input: true` above all — and whose `turn_detection` is the `local`
//!   family's own, read out of the catalog rather than written down again;
//! - [`preflight`](RealtimeProvider::preflight) refuses a missing weights file
//!   as a configuration error **naming the path**, before a task, a channel or a
//!   session exists.
//!
//! # Where the stages come from
//!
//! [`StageOrigin`] is the difference between the shipping path and the one CI
//! runs, and it is explicit rather than inferred:
//!
//! | Origin | Built by | `is_configured` |
//! | --- | --- | --- |
//! | [`Weights`](StageOrigin::Weights) | `--features sherpa` / `--features llama`, from [`WeightsSet`](crate::WeightsSet) | the files are on disk |
//! | [`Supplied`](StageOrigin::Supplied) | the caller — [`crate::scripted`], or an embedder that already owns an ASR | always |
//!
//! [`RealtimeSession`]: via_realtime::RealtimeSession
//! [`Transport`]: via_realtime::Transport
//! [`local_realtime_model_profile`]: via_catalog::local_realtime_model_profile

use std::sync::Arc;
use std::time::Duration;

use serde_json::{Map, Value};
use via_catalog::realtime_provider::realtime_provider_definition;
use via_catalog::{ModelProfile, local_realtime_model_profile};
use via_i18n::{Locale, format as i18n_format, keys};
use via_realtime::{
    ErrorClass, GaRealtimeProtocol, Injection, PermissionRequest, ProviderCapabilities,
    RealtimeError, RealtimeProtocol, RealtimeProvider, RealtimeSession, SessionEvents,
    SessionOptions, SessionRequest, Transport, ga_realtime_protocol,
};
use via_realtime_openai::{
    GA_OUTPUT_MODALITIES, OpenAiRealtimeProvider, SESSION_TYPE, classify_openai_error,
    permission_request_text, permission_response_instructions, result_response_instructions,
    speak_response_instructions,
};

use crate::error::LocalError;
use crate::machine::{INBOUND_CAPACITY, MachineOptions};
use crate::mode::{LocalMode, PROVIDER_KEY};
use crate::settings::LocalSettings;
use crate::stages::{PIPELINE_INPUT_RATE, PIPELINE_OUTPUT_RATE, Stage, StageError, Stages};

/// The human label used when the catalog row is somehow unavailable.
pub const FALLBACK_LABEL: &str = "Local Omni";

/// How long to wait for `response.created` on the local pipeline.
///
/// The same 60 s `via-realtime-dashscope`'s speech-to-speech provider declares,
/// and for the reason that crate records: *"a fully local ASR → LLM → TTS
/// pipeline can take substantially longer than a cloud model before producing
/// its first response event."* Twice the 30 s session default. On a laptop
/// loading a Qwen3 GGUF's first token, 30 s is not a generous budget.
pub const RESPONSE_START_TIMEOUT: Duration = Duration::from_secs(60);

/// The GA audio-format discriminator.
///
/// The MIME-shaped name the GA schema uses, unlike the beta dialect's bare
/// `'pcm'`.
pub const AUDIO_FORMAT: &str = "audio/pcm";

/// Whether this provider's stages load model files.
///
/// See the module docs. It is a declaration rather than a guess because the
/// consequence differs: a provider whose engines need files must refuse to open
/// when they are absent, and a provider handed working stages must not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StageOrigin {
    /// Engines built from [`WeightsSet`](crate::WeightsSet), behind
    /// `--features sherpa` / `--features llama`. The shipping path.
    #[default]
    Weights,
    /// Stages the caller supplies in-process: the deterministic doubles, or an
    /// embedder that already owns a recognizer.
    Supplied,
}

/// `local-omni` in its `pipeline` mode.
#[derive(Clone, Debug)]
pub struct LocalPipelineProvider {
    settings: LocalSettings,
    origin: StageOrigin,
    label: &'static str,
    model: Option<String>,
    voice: String,
    protocol: GaRealtimeProtocol,
}

impl LocalPipelineProvider {
    /// A provider whose stages load [`WeightsSet`](crate::WeightsSet) from disk.
    #[must_use]
    pub fn new(settings: LocalSettings) -> Self {
        Self::with_origin(settings, StageOrigin::Weights)
    }

    /// A provider whose stages the caller supplies.
    ///
    /// The seam CI runs on, and the seam an embedder that already owns a
    /// recognizer uses. It does not check the model root, because there is
    /// nothing there to check.
    #[must_use]
    pub fn with_supplied_stages(settings: LocalSettings) -> Self {
        Self::with_origin(settings, StageOrigin::Supplied)
    }

    fn with_origin(settings: LocalSettings, origin: StageOrigin) -> Self {
        let settings = settings.with_mode(LocalMode::Pipeline);
        let definition = realtime_provider_definition(PROVIDER_KEY).ok();
        Self {
            label: definition.map_or(FALLBACK_LABEL, |row| row.label),
            model: settings.resolved_model(),
            voice: settings.resolved_voice().to_owned(),
            protocol: ga_realtime_protocol(),
            settings,
            origin,
        }
    }

    /// The settings this provider was built with.
    #[must_use]
    pub fn settings(&self) -> &LocalSettings {
        &self.settings
    }

    /// Where its stages come from.
    #[must_use]
    pub fn origin(&self) -> StageOrigin {
        self.origin
    }

    /// The `local`-family profile for the resolved model id.
    ///
    /// `None` only when there is no model id at all — see
    /// [`LocalSettings::resolved_model`]. Never an all-capabilities-false
    /// profile: `via-catalog` has no such thing, by design.
    #[must_use]
    pub fn profile(&self) -> Option<ModelProfile> {
        self.model.as_deref().map(local_realtime_model_profile)
    }

    /// The `response.create` body shared by both injections.
    ///
    /// Three keys in this order, exactly as `dashscope.mjs:106-110` and
    /// `s2s.mjs:111-115` write them. `tool_choice: 'none'` is what stops the
    /// model answering a result or a permission question by calling another
    /// tool.
    fn injection_response(&self, instructions: String) -> Value {
        let mut response = Map::new();
        response.insert("modalities".to_owned(), modalities());
        response.insert("tool_choice".to_owned(), Value::String("none".to_owned()));
        response.insert("instructions".to_owned(), Value::String(instructions));
        Value::Object(response)
    }
}

/// The single output modality, in the beta spelling the GA adapter rewrites.
///
/// `modalities` rather than `output_modalities`, because
/// [`GaRealtimeProtocol::response_create`] moves the key itself — which is
/// exactly where upstream puts that rename, so no call site has to know.
fn modalities() -> Value {
    Value::Array(
        GA_OUTPUT_MODALITIES
            .iter()
            .map(|name| Value::String((*name).to_owned()))
            .collect(),
    )
}

fn audio_format(rate: u32) -> Value {
    let mut format = Map::new();
    format.insert("type".to_owned(), Value::String(AUDIO_FORMAT.to_owned()));
    format.insert("rate".to_owned(), Value::Number(rate.into()));
    Value::Object(format)
}

impl RealtimeProvider for LocalPipelineProvider {
    fn key(&self) -> &str {
        PROVIDER_KEY
    }

    fn label(&self) -> &str {
        self.label
    }

    fn input_sample_rate(&self) -> u32 {
        PIPELINE_INPUT_RATE.hz()
    }

    fn output_sample_rate(&self) -> u32 {
        PIPELINE_OUTPUT_RATE.hz()
    }

    fn response_start_timeout(&self) -> Option<Duration> {
        Some(RESPONSE_START_TIMEOUT)
    }

    fn is_configured(&self) -> bool {
        match self.origin {
            StageOrigin::Weights => self.settings.weights.is_installed(),
            StageOrigin::Supplied => true,
        }
    }

    fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    fn voice(&self) -> Option<&str> {
        Some(&self.voice)
    }

    fn url(&self) -> Result<String, RealtimeError> {
        // The pipeline is in-process; there is no socket to dial, and answering
        // with a plausible-looking URL would send `RealtimeSession::connect`
        // somewhere it cannot go. `LocalPipeline::open` is the door.
        Err(RealtimeError::Transport {
            detail: format!(
                "{} runs in-process and has no endpoint; open it with LocalPipeline::open",
                LocalMode::Pipeline.qualified()
            ),
        })
    }

    fn headers(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            // The machine answers every `session.update` with `session.updated`.
            acknowledges_session_update: true,
            // One reasoning turn at a time. Declaring it is what arms
            // `via-realtime`'s bounded busy-retry ladder, which replays a
            // refused `response.create` byte-identically.
            single_response_slot: true,
            // The pipeline is both ends of its own wire, so it echoes the GA
            // dialect's correlation key exactly. A cloud provider that only
            // *might* echo it has to leave this `false`; this one knows.
            response_metadata_correlation: true,
            // `response.create` carries `instructions`, which is how a finished
            // background result is spoken without entering the conversation.
            per_response_instructions: true,
            // The client's item id is echoed verbatim.
            conversation_item_id_echo: true,
        }
    }

    fn model_profile(&self) -> Option<ModelProfile> {
        self.profile()
    }

    fn configuration_signature(&self) -> String {
        self.settings.configuration_signature()
    }

    fn protocol(&self) -> &dyn RealtimeProtocol {
        &self.protocol
    }

    fn preflight(&self) -> Result<(), RealtimeError> {
        if self.origin == StageOrigin::Supplied {
            return Ok(());
        }
        // The gate `docs/architecture.md` §7 asks for: a missing weights file is
        // a clean configuration error **naming the path**, raised before a
        // socket, a task or a session exists — never a session that connects and
        // hears nothing.
        self.settings
            .weights
            .verify()
            .map_err(|error| error.into_realtime(self.settings.locale))
    }

    fn classify_error(&self, message: &str) -> ErrorClass {
        // The corpus is `via-realtime-openai`'s, and deliberately: the two
        // refusals the machine emits are ARGO's own wire vocabulary, so the
        // shipped classifier is the one that has to recognise them. See
        // `crate::machine`.
        classify_openai_error(message)
    }

    fn build_session(&self, request: &SessionRequest<'_>) -> Value {
        // The GA shape, laid out as `via-realtime-dashscope`'s speech-to-speech
        // provider lays it out — and, like that provider, it does **not** branch
        // on `configured`: the machine applies every update whole, and a context
        // refresh that dropped the audio block would leave the pipeline with no
        // turn detection.
        let mut session = Map::new();
        session.insert("type".to_owned(), Value::String(SESSION_TYPE.to_owned()));
        session.insert(
            "instructions".to_owned(),
            Value::String(request.agent_context.instructions.clone()),
        );
        session.insert(
            "tools".to_owned(),
            // The flattening is `via-realtime-openai`'s: `AgentContext` carries
            // the beta-nested shape because that is the one form a tool catalog
            // is composed in, and both GA endpoints want it flat.
            OpenAiRealtimeProvider::flat_tools(&request.agent_context.tools),
        );
        session.insert("output_modalities".to_owned(), modalities());

        let mut turn_detection = Map::new();
        // Read out of the catalog's `local` family rather than written here:
        // this is the field upstream's all-false profile leaves null, and the
        // whole of "the session connects and hears nothing".
        let kind = self.profile().map_or_else(
            || Value::String("server_vad".to_owned()),
            |profile| {
                serde_json::to_value(profile.session_defaults.turn_detection.kind)
                    .unwrap_or_else(|_| Value::String("server_vad".to_owned()))
            },
        );
        turn_detection.insert("type".to_owned(), kind);
        turn_detection.insert("interrupt_response".to_owned(), Value::Bool(true));

        let mut input = Map::new();
        input.insert("format".to_owned(), audio_format(PIPELINE_INPUT_RATE.hz()));
        input.insert("turn_detection".to_owned(), Value::Object(turn_detection));

        let mut output = Map::new();
        output.insert("voice".to_owned(), Value::String(self.voice.clone()));
        output.insert("format".to_owned(), audio_format(PIPELINE_OUTPUT_RATE.hz()));

        let mut audio = Map::new();
        audio.insert("input".to_owned(), Value::Object(input));
        audio.insert("output".to_owned(), Value::Object(output));
        session.insert("audio".to_owned(), Value::Object(audio));

        Value::Object(session)
    }

    fn build_speak_response(&self, content: &str) -> Value {
        let mut response = Map::new();
        response.insert("conversation".to_owned(), Value::String("none".to_owned()));
        response.insert("modalities".to_owned(), modalities());
        response.insert(
            "instructions".to_owned(),
            Value::String(speak_response_instructions(content, self.settings.locale)),
        );
        Value::Object(response)
    }

    fn build_result_injection(&self, content: &str) -> Injection {
        Injection {
            item: self.protocol.user_text_item(content),
            response: self.injection_response(result_response_instructions(self.settings.locale)),
        }
    }

    fn build_permission_injection(&self, permission: &PermissionRequest) -> Injection {
        Injection {
            item: self
                .protocol
                .user_text_item(&permission_request_text(permission)),
            response: self
                .injection_response(permission_response_instructions(self.settings.locale)),
        }
    }

    fn missing_configuration_message(&self, locale: Locale) -> String {
        self.settings.weights.verify().err().map_or_else(
            || {
                i18n_format(
                    locale,
                    keys::REALTIME_MISSING_CONFIGURATION,
                    &[("label", self.label)],
                )
            },
            |error| error.localized(locale),
        )
    }

    fn connect_timeout_message(&self, locale: Locale) -> String {
        i18n_format(
            locale,
            keys::REALTIME_CONNECT_TIMEOUT,
            &[("label", self.label)],
        )
    }
}

/// The stage invariant the machine cannot recover from.
///
/// The VAD and the recognizer are fed the **same** block, so a recognizer that
/// declares a different rate would be handed audio at the wrong speed — and,
/// like every rate mismatch, that does not crash: it just quietly stops
/// transcribing. One comparison at open turns it into a configuration error.
///
/// # Errors
///
/// [`LocalError::Stage`] naming both rates.
pub fn validate_stages(stages: &Stages) -> Result<(), LocalError> {
    let engine = stages.voice_activity.sample_rate();
    let recognizer = stages.transcriber.sample_rate();
    if engine == recognizer {
        return Ok(());
    }
    Err(LocalError::Stage(StageError::new(
        Stage::Asr,
        format!(
            "the recognizer runs at {} Hz but the detector in front of it runs at {} Hz",
            recognizer.hz(),
            engine.hz()
        ),
    )))
}

/// A configured provider and the four stages behind it.
///
/// ```no_run
/// use std::sync::Arc;
/// use via_realtime_local::{
///     LocalPipeline, LocalPipelineProvider, LocalSettings, ScriptedResponder,
///     ScriptedSpeaker, ScriptedTranscriber, ScriptedVoiceActivity, Stages,
/// };
///
/// # async fn open() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = LocalPipelineProvider::with_supplied_stages(LocalSettings::default());
/// let (session, events) = LocalPipeline::new(
///     provider,
///     Stages {
///         voice_activity: Box::new(ScriptedVoiceActivity::utterance(2)),
///         transcriber: Box::new(ScriptedTranscriber::hearing("turn on the lights")),
///         responder: Arc::new(ScriptedResponder::saying("On it. ")),
///         speaker: Arc::new(ScriptedSpeaker::new()),
///     },
/// )
/// .open()
/// .await?;
/// # let _ = (session, events);
/// # Ok(())
/// # }
/// ```
pub struct LocalPipeline {
    provider: Arc<LocalPipelineProvider>,
    stages: Stages,
    options: SessionOptions,
}

impl core::fmt::Debug for LocalPipeline {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LocalPipeline")
            .field("provider", &self.provider)
            .field("stages", &self.stages)
            .field("mode", &self.options.mode)
            .finish_non_exhaustive()
    }
}

impl LocalPipeline {
    /// A pipeline over `provider` and `stages`.
    #[must_use]
    pub fn new(provider: LocalPipelineProvider, stages: Stages) -> Self {
        Self::over(Arc::new(provider), stages)
    }

    /// A pipeline over a provider that is already shared.
    #[must_use]
    pub fn over(provider: Arc<LocalPipelineProvider>, stages: Stages) -> Self {
        Self {
            provider,
            stages,
            options: SessionOptions::default(),
        }
    }

    /// Replace the session options wholesale.
    #[must_use]
    pub fn with_options(mut self, options: SessionOptions) -> Self {
        self.options = options;
        self
    }

    /// Open in a particular session mode.
    ///
    /// `dictation` is the cheap path: VAD plus ASR, no reasoning turn and no
    /// synthesis, and the machine never creates a response.
    #[must_use]
    pub fn with_mode(mut self, mode: via_protocol::SessionMode) -> Self {
        self.options.mode = mode;
        self
    }

    /// The provider this pipeline presents.
    #[must_use]
    pub fn provider(&self) -> &Arc<LocalPipelineProvider> {
        &self.provider
    }

    /// Start the machine and configure a session over it.
    ///
    /// # Errors
    ///
    /// Whatever [`preflight`](RealtimeProvider::preflight) refuses — a missing
    /// weights file, naming the path — then [`validate_stages`], then whatever
    /// [`RealtimeSession::open`] refuses.
    pub async fn open(self) -> Result<(RealtimeSession, SessionEvents), RealtimeError> {
        // The order is `via-realtime`'s and it matters: preflight, then
        // configuration, then anything that costs a task.
        self.provider.preflight()?;
        if !self.provider.is_configured() {
            return Err(RealtimeError::NotConfigured {
                provider: PROVIDER_KEY.to_owned(),
                message: self
                    .provider
                    .missing_configuration_message(self.options.locale),
            });
        }
        validate_stages(&self.stages).map_err(|error| error.into_realtime(self.options.locale))?;

        let (to_machine, inbound) = futures::channel::mpsc::channel(INBOUND_CAPACITY);
        let (outbound, from_machine) = futures::channel::mpsc::unbounded();
        let transport = Transport::new(to_machine, from_machine);

        let voice = self.provider.voice.clone();
        let client_rate = PIPELINE_INPUT_RATE;
        tokio::spawn(crate::machine::run(
            MachineOptions {
                stages: self.stages,
                mode: self.options.mode,
                voice,
                client_rate,
            },
            inbound,
            outbound,
        ));

        let provider: Arc<dyn RealtimeProvider> = self.provider;
        RealtimeSession::open(provider, self.options, transport).await
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use via_catalog::{ModelFamily, TurnDetectionKind};
    use via_realtime::{AgentContext, validate_realtime_provider};

    use super::*;
    use crate::scripted::{
        ScriptedResponder, ScriptedSpeaker, ScriptedTranscriber, ScriptedVoiceActivity,
    };
    use crate::weights::WeightsSet;

    fn provider() -> LocalPipelineProvider {
        LocalPipelineProvider::with_supplied_stages(LocalSettings::default().with_weights(
            WeightsSet {
                reasoning: std::path::PathBuf::from("/m/reasoning/Qwen3-8B-Q4_K_M.gguf"),
                ..WeightsSet::resolve("/m")
            },
        ))
    }

    fn context() -> AgentContext {
        AgentContext {
            instructions: "You are VIA.".to_owned(),
            tools: vec![json!({
                "type": "function",
                "function": {
                    "name": "request_delegation",
                    "description": "Delegate work.",
                    "parameters": { "type": "object" },
                },
            })],
            ..AgentContext::default()
        }
    }

    fn stages() -> Stages {
        Stages {
            voice_activity: Box::new(ScriptedVoiceActivity::silent()),
            transcriber: Box::new(ScriptedTranscriber::deaf()),
            responder: Arc::new(ScriptedResponder::saying("ok")),
            speaker: Arc::new(ScriptedSpeaker::new()),
        }
    }

    #[test]
    fn the_provider_validates_and_registers_under_the_catalog_key() {
        let provider = provider();
        assert_eq!(validate_realtime_provider(&provider), Ok(()));
        assert_eq!(provider.key(), PROVIDER_KEY);
        assert_eq!(provider.label(), "Local Omni");
        assert!(provider.aliases().is_empty());
    }

    #[test]
    fn the_model_profile_is_the_local_family_with_real_capability_flags() {
        // The bug not to port: an unknown id must not produce an all-false
        // profile, because `audio_input: false` is a session that hears nothing.
        let profile = provider().model_profile().expect("a resolved model id");
        assert_eq!(profile.family, ModelFamily::Local);
        assert_eq!(profile.id, "Qwen3-8B-Q4_K_M");
        assert!(profile.transport_capabilities.audio_input);
        assert!(profile.model_capabilities.audio_input);
        assert!(profile.model_capabilities.audio_output);
        assert!(profile.model_capabilities.function_calling);
        assert_eq!(
            profile.session_defaults.turn_detection.kind,
            TurnDetectionKind::ServerVad
        );
    }

    #[test]
    fn the_session_payload_carries_turn_detection_from_the_catalog() {
        let session = provider().build_session(&SessionRequest {
            configured: false,
            agent_context: &context(),
        });
        assert_eq!(session["type"], json!(SESSION_TYPE));
        assert_eq!(session["instructions"], json!("You are VIA."));
        assert_eq!(session["output_modalities"], json!(["audio"]));
        assert_eq!(
            session["audio"]["input"]["turn_detection"],
            json!({ "type": "server_vad", "interrupt_response": true })
        );
        assert_eq!(
            session["audio"]["input"]["format"],
            json!({ "type": "audio/pcm", "rate": 16_000 })
        );
        assert_eq!(
            session["audio"]["output"]["format"],
            json!({ "type": "audio/pcm", "rate": 24_000 })
        );
        assert_eq!(
            session["audio"]["output"]["voice"],
            json!(via_catalog::realtime_model::DEFAULT_LOCAL_REALTIME_VOICE)
        );
    }

    #[test]
    fn the_tool_catalog_is_flattened_the_way_both_ga_endpoints_want_it() {
        let session = provider().build_session(&SessionRequest {
            configured: false,
            agent_context: &context(),
        });
        assert_eq!(
            session["tools"],
            json!([{
                "type": "function",
                "name": "request_delegation",
                "description": "Delegate work.",
                "parameters": { "type": "object" },
            }])
        );
    }

    #[test]
    fn every_update_carries_the_whole_payload() {
        // Unlike DashScope, and for the same reason speech-to-speech does:
        // the machine applies each update whole, and a refresh that dropped the
        // audio block would leave the pipeline with no turn detection.
        let provider = provider();
        let first = provider.build_session(&SessionRequest {
            configured: false,
            agent_context: &context(),
        });
        let refresh = provider.build_session(&SessionRequest {
            configured: true,
            agent_context: &context(),
        });
        assert_eq!(first, refresh);
        assert!(refresh["audio"]["input"].get("turn_detection").is_some());
    }

    #[test]
    fn the_capabilities_are_the_five_a_pipeline_can_actually_keep() {
        let capabilities = provider().capabilities();
        assert!(capabilities.acknowledges_session_update);
        assert!(capabilities.single_response_slot);
        assert!(capabilities.response_metadata_correlation);
        assert!(capabilities.per_response_instructions);
        assert!(capabilities.conversation_item_id_echo);
    }

    #[test]
    fn the_pipeline_has_no_endpoint_and_says_so_rather_than_inventing_one() {
        let error = provider().url().expect_err("in-process");
        assert_eq!(error.code(), "VIA_REALTIME_TRANSPORT");
        assert!(error.to_string().contains("local-omni:pipeline"));
        assert!(provider().headers().is_empty());
    }

    #[test]
    fn supplied_stages_are_configured_and_weights_are_not_until_they_are_there() {
        assert!(provider().is_configured());
        assert_eq!(provider().preflight(), Ok(()));

        let on_disk = LocalPipelineProvider::new(
            LocalSettings::default().with_model_root("/nonexistent-model-root"),
        );
        assert!(!on_disk.is_configured());
        let error = on_disk.preflight().expect_err("no weights");
        assert!(
            error.to_string().contains("/nonexistent-model-root"),
            "the refusal must name the path: {error}"
        );
    }

    #[test]
    fn a_broken_install_names_the_absent_file_rather_than_the_root() {
        let root = tempfile::tempdir().expect("tempdir");
        let set = WeightsSet::resolve(root.path());
        for required in set.required() {
            if required.is_directory {
                std::fs::create_dir_all(&required.path).expect("directory");
            } else {
                if let Some(parent) = required.path.parent() {
                    std::fs::create_dir_all(parent).expect("parent");
                }
                std::fs::write(&required.path, b"x").expect("file");
            }
        }
        std::fs::remove_file(&set.speaker.model).expect("remove one");

        let provider =
            LocalPipelineProvider::new(LocalSettings::default().with_model_root(root.path()));
        assert!(!provider.is_configured());
        let error = provider.preflight().expect_err("one file missing");
        assert!(
            error
                .to_string()
                .contains(&set.speaker.model.display().to_string()),
            "{error}"
        );
    }

    #[test]
    fn matching_stage_rates_validate_and_a_mismatch_is_named() {
        assert_eq!(validate_stages(&stages()), Ok(()));

        let mismatched = Stages {
            voice_activity: Box::new(
                ScriptedVoiceActivity::silent().at_rate(via_audio::SampleRate::HZ_24000),
            ),
            ..stages()
        };
        let error = validate_stages(&mismatched).expect_err("rates disagree");
        let rendered = error.to_string();
        assert!(rendered.contains("16000"), "{rendered}");
        assert!(rendered.contains("24000"), "{rendered}");
    }

    #[test]
    fn the_two_injections_carry_the_catalogued_shapes() {
        let provider = provider();
        let result = provider.build_result_injection("the build finished");
        assert_eq!(result.item["type"], json!("message"));
        assert_eq!(result.item["role"], json!("user"));
        assert_eq!(result.response["tool_choice"], json!("none"));
        let keys: Vec<&str> = result
            .response
            .as_object()
            .map(|object| object.keys().map(String::as_str).collect())
            .unwrap_or_default();
        assert_eq!(keys, ["modalities", "tool_choice", "instructions"]);

        let permission = provider.build_permission_injection(&PermissionRequest {
            id: "perm_1".to_owned(),
            summary: "run `rm -rf build`".to_owned(),
        });
        let text = permission.item["content"][0]["text"]
            .as_str()
            .unwrap_or_default();
        assert!(text.starts_with("<backend_permission_request>"), "{text}");
        assert!(text.contains("authorization_id=perm_1"), "{text}");
    }

    #[test]
    fn a_speak_response_stays_out_of_conversation_history() {
        let response = provider().build_speak_response("three o'clock");
        assert_eq!(response["conversation"], json!("none"));
        assert!(
            response["instructions"]
                .as_str()
                .unwrap_or_default()
                .contains("three o'clock")
        );
    }

    #[test]
    fn the_response_start_budget_is_the_local_one_not_the_session_default() {
        assert_eq!(
            provider().response_start_timeout(),
            Some(RESPONSE_START_TIMEOUT)
        );
        assert!(RESPONSE_START_TIMEOUT > via_realtime::DEFAULT_RESPONSE_START_TIMEOUT);
    }

    #[test]
    fn the_two_provider_sentences_render_in_every_locale() {
        let provider = provider();
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            for rendered in [
                provider.missing_configuration_message(locale),
                provider.connect_timeout_message(locale),
            ] {
                assert!(!rendered.is_empty(), "{locale}");
                assert!(!rendered.contains('{'), "{locale}: {rendered}");
            }
        }
    }

    #[test]
    fn a_provider_with_no_model_id_publishes_no_profile_rather_than_a_false_one() {
        let provider = LocalPipelineProvider::with_supplied_stages(
            LocalSettings::default().with_weights(WeightsSet {
                reasoning: std::path::PathBuf::from("/"),
                ..WeightsSet::resolve("/m")
            }),
        );
        assert_eq!(provider.model(), None);
        assert_eq!(provider.model_profile(), None);
        // And it still validates: `dictation` is a mode with no model at all.
        assert_eq!(validate_realtime_provider(&provider), Ok(()));
    }

    #[test]
    fn the_mode_is_forced_to_pipeline_whatever_the_settings_said() {
        let provider = LocalPipelineProvider::with_supplied_stages(
            LocalSettings::default().with_endpoint("ws://127.0.0.1:8000"),
        );
        assert_eq!(provider.settings().mode, LocalMode::Pipeline);
    }
}
