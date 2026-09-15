//! VIA's on-device realtime providers.
//!
//! Phase 8. `docs/architecture.md` §7 opens with three verified facts, and all
//! three shape this crate rather than decorating it:
//!
//! 1. **Qwen3.5-Omni has no open weights** — it is DashScope-API-only. So
//!    "on-device omni" means Qwen3-Omni-30B-A3B, which is what
//!    [`local-omni:endpoint`](endpoint) points at.
//! 2. **No Rust runtime anywhere runs the Qwen3-Omni Talker** — checked against
//!    candle, mistral.rs, llama.cpp (whose `mtmd_gen_audio_type` is
//!    `{NONE, QWEN3TTS, POCKETTTS}`) and mlx-rs. So the on-device path that
//!    actually runs on a laptop is **componentized**, not omni.
//! 3. **The official `sherpa-onnx` crate covers VAD, streaming ASR and TTS
//!    behind one interface** — which is what makes that cascade one dependency
//!    instead of three.
//!
//! # Two providers, one key
//!
//! | Mode | What it is | Verified here |
//! | --- | --- | --- |
//! | [`local-omni:pipeline`](pipeline) | VAD → streaming ASR → the reasoning turn → TTS, presented to the Gateway as **one realtime session** | the machine, through [`scripted`] |
//! | [`local-omni:endpoint`](endpoint) | an OpenAI-Realtime client pointed at a local server (`sgl-omni serve --enable-realtime`) | the surface and the refusal diagnosis only — **CUDA-only, unverified on this machine** |
//!
//! Both register under the one key `via-catalog` declares — `local-omni` —
//! because a `:` is not a legal provider key and because a client that stored
//! `local-omni` must keep resolving whichever mode the operator later chose.
//! [`LocalMode`] is that choice; [`local_provider`] makes it.
//!
//! # The build constraint that shapes the crate
//!
//! `docs/adr/0001-placement.md` records that VIA's native dependencies —
//! `sherpa-onnx`, `llama-cpp-2`, `cpal` — are precisely what keeps VIA out of
//! ARGO's workspace, because ARGO requires every crate to cross-compile clean to
//! four targets. A C toolchain in every VIA build lane, for a feature most
//! builds never exercise, is that same decision made badly at a smaller scale.
//! `via-wake-word` draws the line at a trait; so does this crate.
//!
//! | Half | Feature | What is in it |
//! | --- | --- | --- |
//! | **the session machine** | *default* | [`machine`] · [`stages`] · [`sentence`] · [`events`] · [`LocalPipelineProvider`] · [`LocalEndpointProvider`] · [`LocalSettings`] · [`WeightsSet`] · [`scripted`] |
//! | the sherpa engines | `sherpa` | [`SherpaVoiceActivity`] · [`SherpaTranscriber`] · [`SherpaSpeaker`] |
//! | the reasoning engine | `llama` | [`LlamaResponder`] |
//!
//! `cargo build -p via-realtime-local` needs no C toolchain, no weights and no
//! network. `cargo test -p via-realtime-local` drives **the whole pipeline** —
//! the handshake, an utterance, a turn, incremental synthesis, barge-in,
//! cancellation, tool calls, `dictation`, and every stage failure — against
//! [`scripted`], because that is the only part CI can ever run.
//!
//! # The hard part is the duplex illusion
//!
//! A realtime session is full duplex and interruptible; a cascade is three
//! request/response calls. [`machine`] is the whole of that distance: one
//! owning task, three sources polled `biased` with inbound frames first,
//! sentence-level synthesis overlapped with generation, and barge-in that cuts
//! speech mid-utterance and discards the in-flight reasoning turn by **dropping
//! its stream**.
//!
//! The measure of success is negative: the event vocabulary in [`events`] is
//! the one a cloud provider emits, spelled the way the shipped tables spell it,
//! so **nothing above this crate can tell the difference**.
//!
//! # One bug not to port
//!
//! `docs/architecture.md` §7: upstream answers an unknown realtime model id with
//! an all-capabilities-false profile, and `dashscope.mjs:84-87` gates
//! `session.turn_detection` on `transportCapabilities.audioInput` — so an
//! unknown id opens a session that **connects and never hears the user**. A
//! local model id is by definition not in the DashScope table, so on this path
//! that fallback would be reached on the happy path.
//!
//! Both halves of the fix live here. The model profile is `via-catalog`'s
//! [`local_realtime_model_profile`](via_catalog::local_realtime_model_profile),
//! whose flags are real; and a missing weights file is a clean configuration
//! error **naming the path**, raised from `preflight` before a socket, a task or
//! a session exists — never a session that connects and hears nothing.
//!
//! # What it builds on rather than restating
//!
//! | Owned by | What |
//! | --- | --- |
//! | `via-realtime` | the provider and protocol traits, [`RealtimeSession`](via_realtime::RealtimeSession), `Transport`, the watchdogs, the busy-retry ladder, the GA dialect and its correlation key |
//! | `via-realtime-openai` | **the entire endpoint client** — the dialect, the candidate walk, the first-frame probe, the schema repair, the error corpus, the three prompt sentences and the conflict vocabulary |
//! | `via-catalog` | the `local-omni` row, the `local`-family profile, the configuration-signature hash |
//! | `via-core` | `Config`, `InstallPaths`, `Secret` |
//! | `via-audio` | `SampleRate`, `PlaybackCursor`, the resampler, every PCM16 ↔ `f32` conversion |
//! | `via-i18n` | every sentence a person reads |
//! | `via-protocol` | `SessionMode` — `dictation` mounts no model turn |
//!
//! Deviations are recorded in
//! `docs/deviations/phase-8-via-realtime-local.md`.
//!
//! # Example
//!
//! The whole pipeline, with no weights and no network:
//!
//! ```
//! use std::sync::Arc;
//! use via_realtime_local::{
//!     LocalPipeline, LocalPipelineProvider, LocalSettings, ScriptedResponder, ScriptedSpeaker,
//!     ScriptedTranscriber, ScriptedVoiceActivity, Stages,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # tokio::runtime::Runtime::new()?.block_on(async {
//! let (session, _events) = LocalPipeline::new(
//!     LocalPipelineProvider::with_supplied_stages(LocalSettings::default()),
//!     Stages {
//!         voice_activity: Box::new(ScriptedVoiceActivity::utterance(2)),
//!         transcriber: Box::new(ScriptedTranscriber::hearing("turn on the lights")),
//!         responder: Arc::new(ScriptedResponder::saying("On it. ")),
//!         speaker: Arc::new(ScriptedSpeaker::new()),
//!     },
//! )
//! .open()
//! .await?;
//!
//! assert_eq!(session.provider().key(), "local-omni");
//! assert_eq!(session.input_sample_rate(), 16_000);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # })
//! # }
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod endpoint;
pub mod error;
pub mod events;
pub mod health;
pub mod machine;
pub mod mode;
pub mod pipeline;
pub mod scripted;
pub mod sentence;
pub mod settings;
pub mod stages;
pub mod weights;

#[cfg(feature = "llama")]
pub mod llama;

#[cfg(feature = "sherpa")]
pub mod sherpa;

use std::sync::Arc;

use via_realtime::{RealtimeError, RealtimeProvider, RealtimeProviderRegistry};

pub use endpoint::{
    ANSWERED_MARKERS, LocalEndpointProvider, TLS_MARKERS, UNREACHABLE_MARKERS,
    diagnose_connect_failure, localize_connect_failure,
};
pub use error::LocalError;
pub use events::{EMITTED_EVENT_TYPES, HANDLED_CLIENT_FRAMES, decode_audio, encode_audio};
pub use health::LocalHealth;
pub use machine::{
    CANCEL_NOT_ACTIVE_MESSAGE, LOG_TARGET, MAX_QUEUED_RESPONSES, active_response_conflict_message,
};
pub use mode::{LocalMode, MODE_SEPARATOR, PROVIDER_KEY};
pub use pipeline::{
    LocalPipeline, LocalPipelineProvider, RESPONSE_START_TIMEOUT, StageOrigin, validate_stages,
};
pub use scripted::{
    CancelCount, ScriptedResponder, ScriptedSpeaker, ScriptedTranscriber, ScriptedTurn,
    ScriptedVoiceActivity, SpokenUtterance,
};
pub use sentence::{HARD_TERMINATORS, MAX_SENTENCE_CHARS, SOFT_TERMINATORS, SentenceSplitter};
pub use settings::{DEFAULT_LOCAL_ENDPOINT_MODEL, ENDPOINT_ENV, LocalSettings};
pub use stages::{
    PIPELINE_INPUT_RATE, PIPELINE_OUTPUT_RATE, Responder, ResponseDelta, ResponseStream, Speaker,
    SpeechEvent, SpeechStream, Stage, StageError, Stages, ToolCall, Transcriber, TranscriptUpdate,
    Turn, TurnMessage, TurnRole, VoiceActivity,
};
pub use weights::{AsrWeights, LOCAL_MODEL_DIRECTORY, RequiredPath, TtsWeights, WeightsSet};

#[cfg(feature = "llama")]
pub use llama::{LlamaOptions, LlamaResponder};

#[cfg(feature = "sherpa")]
pub use sherpa::{SherpaSpeaker, SherpaTranscriber, SherpaVoiceActivity, sherpa_stages};

/// The provider both modes register under, read from `via-catalog`.
///
/// [`PROVIDER_KEY`] is the same string as a `const`; this is the lookup, and
/// the crate's own tests assert the two agree. Retyping a catalog key is how a
/// rename silently stops resolving.
#[must_use]
pub fn provider_key() -> &'static str {
    via_catalog::realtime_provider::realtime_provider_definition(PROVIDER_KEY)
        .map_or(PROVIDER_KEY, |row| row.key)
}

/// The on-device provider a configuration selects.
///
/// The mode is derived by [`LocalSettings::from_config`] — an endpoint
/// configured is [`LocalMode::Endpoint`], nothing configured is
/// [`LocalMode::Pipeline`] — because `via-catalog`'s `local-omni` row declares
/// no default URL on purpose: *"`pipeline` needs none and `endpoint` must be
/// told where to look."*
///
/// The provider it hands back is a description, not a session. Opening one is
/// [`LocalPipeline::open`] (which needs [`Stages`]) or
/// [`endpoint::open_session`].
#[must_use]
pub fn local_provider(config: &via_core::Config) -> Arc<dyn RealtimeProvider> {
    let settings = LocalSettings::from_config(config);
    match settings.mode {
        LocalMode::Pipeline => Arc::new(LocalPipelineProvider::new(settings)),
        LocalMode::Endpoint => Arc::new(LocalEndpointProvider::new(settings)),
    }
}

/// What `/api/health` says about the on-device path.
#[must_use]
pub fn local_health(config: &via_core::Config) -> LocalHealth {
    LocalHealth::describe(&LocalSettings::from_config(config), config.locale)
}

/// Register the on-device provider into an existing registry.
///
/// Registration validates the provider and refuses a key another already
/// claimed, so this is also how a host extension finds out that it picked a
/// colliding name.
///
/// # Errors
///
/// [`RealtimeError::ProviderNameTaken`] when `registry` already answers to
/// `local-omni`, or whatever
/// [`validate_realtime_provider`](via_realtime::validate_realtime_provider)
/// refuses.
pub fn register_local_provider(
    registry: &mut RealtimeProviderRegistry,
    config: &via_core::Config,
) -> Result<(), RealtimeError> {
    registry.register(local_provider(config)).map(|_| ())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use via_core::Config;

    use super::*;

    #[test]
    fn the_constant_and_the_catalog_agree_on_the_key() {
        assert_eq!(provider_key(), PROVIDER_KEY);
        assert_eq!(PROVIDER_KEY, "local-omni");
    }

    #[test]
    fn an_empty_configuration_selects_the_pipeline() {
        // `docs/architecture.md` §16: the componentized path first, because it
        // runs on the dev box and can be exercised.
        let provider = local_provider(&Config::default());
        assert_eq!(provider.key(), PROVIDER_KEY);
        assert_eq!(provider.input_sample_rate(), PIPELINE_INPUT_RATE.hz());
        assert_eq!(provider.output_sample_rate(), PIPELINE_OUTPUT_RATE.hz());
    }

    #[test]
    fn the_provider_registers_under_its_catalog_key_and_no_alias() {
        let config = Config::default();
        let mut registry = RealtimeProviderRegistry::with_default(PROVIDER_KEY);
        register_local_provider(&mut registry, &config).expect("registers");
        assert_eq!(registry.provider_keys(), [PROVIDER_KEY]);
        assert_eq!(registry.resolvable_names(), [PROVIDER_KEY]);
        assert_eq!(
            registry
                .resolve(Some("  LOCAL-OMNI "))
                .expect("resolves")
                .key(),
            PROVIDER_KEY
        );
    }

    #[test]
    fn registering_twice_is_refused_rather_than_shadowing() {
        let config = Config::default();
        let mut registry = RealtimeProviderRegistry::with_default(PROVIDER_KEY);
        register_local_provider(&mut registry, &config).expect("registers");
        assert_eq!(
            register_local_provider(&mut registry, &config),
            Err(RealtimeError::ProviderNameTaken {
                name: PROVIDER_KEY.to_owned()
            })
        );
    }

    #[test]
    fn a_fresh_install_is_invisible_in_a_picker_but_still_resolves() {
        // No weights on disk: the provider must not show up as a choice that
        // would fail, and must still be reachable by name so the Gateway can
        // report why.
        let config = Config::default();
        let mut registry = RealtimeProviderRegistry::with_default(PROVIDER_KEY);
        register_local_provider(&mut registry, &config).expect("registers");
        assert!(registry.describe_providers(true).is_empty());
        assert!(registry.resolve(Some(PROVIDER_KEY)).is_ok());
    }

    #[test]
    fn the_health_note_describes_the_mode_the_configuration_selected() {
        let health = local_health(&Config::default());
        assert_eq!(health.mode, LocalMode::Pipeline.qualified());
        assert!(health.verified);
    }
}
