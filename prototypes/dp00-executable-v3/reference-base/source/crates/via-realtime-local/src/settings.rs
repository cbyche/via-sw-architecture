//! What the two on-device providers read.
//!
//! The same shape `via-realtime-dashscope` and `via-realtime-openai` use: a
//! value, not a `&Config`. The first of their three reasons is the one that
//! matters here too — [`RealtimeProvider`] requires `Debug`, and a provider
//! holding a `String` bearer token prints it the first time an operator formats
//! a session with `{:?}`. [`Secret`] is the shipped answer.
//!
//! # Where the mode comes from
//!
//! `via-core` owns environment reading, ships no local-mode variable, and this
//! crate does not edit it. So the mode is **derived from the configuration that
//! already exists**, using the rule `via-catalog`'s `local-omni` row states:
//!
//! > There is deliberately no default endpoint — `pipeline` needs none and
//! > `endpoint` must be told where to look.
//!
//! An endpoint configured is therefore [`LocalMode::Endpoint`]; nothing
//! configured is [`LocalMode::Pipeline`], which is also `docs/architecture.md`
//! §16's ordering (*"the 100% Rust componentized path first, because it runs on
//! the dev box and can be exercised"*). [`LocalSettings::with_mode`] overrides
//! it, and [`LocalMode::parse`] is there for whoever wires a config surface
//! later. `docs/deviations/phase-8-via-realtime-local.md` records the choice and
//! the alternative that was not taken.
//!
//! [`RealtimeProvider`]: via_realtime::RealtimeProvider

use std::path::PathBuf;

use via_catalog::realtime_model::DEFAULT_LOCAL_REALTIME_VOICE;
use via_catalog::realtime_provider::realtime_provider_definition;
use via_catalog::{
    DEFAULT_DASHSCOPE_REALTIME_MODEL, DEFAULT_DASHSCOPE_REALTIME_URL, IdentityShape,
    RealtimeIdentity,
};
use via_core::config::RealtimeFrontend;
use via_core::{Config, Secret};
use via_i18n::Locale;

use crate::mode::{LocalMode, PROVIDER_KEY};
use crate::weights::{LOCAL_MODEL_DIRECTORY, WeightsSet};

/// The model id `local-omni:endpoint` publishes when nothing named one.
///
/// **Not an upstream contract** — upstream has no local family. It is the id
/// `docs/architecture.md` §7 pins the on-device omni path to, and the reason it
/// is that id and not `qwen3.5-omni-*` is one of the three verified facts §7
/// opens with: *"Qwen3.5-Omni has no open weights (DashScope-API-only) … So
/// 'on-device omni' means Qwen3-Omni-30B-A3B."* It names the vendor's model, so
/// it is **KEEP** under `docs/rebrand.md`.
pub const DEFAULT_LOCAL_ENDPOINT_MODEL: &str = "Qwen3-Omni-30B-A3B";

/// The environment variable an operator sets to point at a local server.
///
/// Not this crate's own variable: `via_core::config::resolve_realtime_frontend`
/// reads one endpoint chain for all five providers, and this is the first link
/// of it. Named here so [`LocalError::EndpointNotConfigured`] can tell an
/// operator what to set rather than that something is missing.
///
/// [`LocalError::EndpointNotConfigured`]: crate::LocalError::EndpointNotConfigured
pub const ENDPOINT_ENV: &str = "VIA_REALTIME_BASE_URL";

/// What [`LocalPipelineProvider`](crate::LocalPipelineProvider) and
/// [`LocalEndpointProvider`](crate::LocalEndpointProvider) read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalSettings {
    /// Which on-device path is mounted.
    pub mode: LocalMode,

    /// The local server's address, in any spelling a config surface writes.
    ///
    /// Empty means "nothing was configured", which for
    /// [`LocalMode::Endpoint`] is the whole of *not configured*: there is no
    /// default to fall back to, by design.
    pub endpoint: String,

    /// A bearer for a local server that wants one.
    ///
    /// Usually empty. A loopback server on a single-user machine needs no
    /// credential, and requiring one would make the common case fail — which is
    /// exactly the local-endpoint policy this crate adds on top of
    /// `via-realtime-openai`, whose own `is_configured` demands a key.
    pub auth_token: Secret,

    /// The model id override, empty when the operator set none.
    ///
    /// The raw override — empty included — is what the configuration signature
    /// hashes, because a default this crate applies is not part of the
    /// operator's configuration.
    pub model: String,

    /// The voice override, empty when the operator set none.
    pub voice: String,

    /// Where the pipeline's model files are.
    pub weights: WeightsSet,

    /// The locale every sentence these providers compose is rendered in.
    pub locale: Locale,
}

impl Default for LocalSettings {
    /// The settings an empty environment resolves to: the pipeline, no
    /// endpoint, no credential, the conventional model root under the current
    /// directory's `.config`-relative default.
    fn default() -> Self {
        Self {
            mode: LocalMode::default(),
            endpoint: String::new(),
            auth_token: Secret::default(),
            model: String::new(),
            voice: String::new(),
            weights: WeightsSet::resolve(PathBuf::from(LOCAL_MODEL_DIRECTORY)),
            locale: Locale::En,
        }
    }
}

impl LocalSettings {
    /// Read a resolved realtime frontend and a model root.
    ///
    /// # The one mapping that is not one-to-one
    ///
    /// [`RealtimeFrontend`]'s field *names* are DashScope's —
    /// `via_core::config::resolve_realtime_frontend` reads one environment chain
    /// for all five providers and only the **default** each falls back to
    /// differs. So `dashscope_realtime_url` carries whatever
    /// [`ENDPOINT_ENV`] was set to (and, for `local-omni`, nothing when it was
    /// not, because the catalog row declares no default URL), `dashscope_model`
    /// carries `VIA_REALTIME_MODEL`, and `dashscope_voice` already carries the
    /// **`local` family's** variable — `via-core` routes every non-DashScope
    /// provider through `local_realtime_model_profile(...).family`, so no
    /// re-reading is needed here.
    ///
    /// A `dashscope_model` still equal to the DashScope default means the
    /// operator set nothing, exactly as `via-realtime-openai` reads it.
    #[must_use]
    pub fn from_frontend(
        frontend: &RealtimeFrontend,
        model_root: impl Into<PathBuf>,
        locale: Locale,
    ) -> Self {
        // A `dashscope_realtime_url` still equal to the DashScope default means
        // the operator set nothing — `resolve_realtime_frontend` records
        // DashScope's default whichever provider is active — and for this
        // provider "nothing" is the whole of *no endpoint*: `via-catalog`'s
        // `local-omni` row declares no default URL on purpose. Reading it as an
        // address would point the endpoint mode at a cloud host nobody chose.
        // `via-realtime-openai` reads the same field the same way.
        let endpoint = if frontend.dashscope_realtime_url == DEFAULT_DASHSCOPE_REALTIME_URL {
            String::new()
        } else {
            frontend.dashscope_realtime_url.trim().to_owned()
        };
        let model = if frontend.dashscope_model == DEFAULT_DASHSCOPE_REALTIME_MODEL {
            String::new()
        } else {
            frontend.dashscope_model.clone()
        };
        let mode = if endpoint.is_empty() {
            LocalMode::Pipeline
        } else {
            LocalMode::Endpoint
        };
        Self {
            mode,
            endpoint,
            auth_token: frontend.dashscope_api_key.clone(),
            model,
            voice: frontend.dashscope_voice.clone(),
            weights: WeightsSet::discover(model_root),
            locale,
        }
    }

    /// Read a whole configuration.
    ///
    /// The model root is `<config>/models/local-omni`, beside the wake word's
    /// `<config>/models/wake-word` — `via_core::InstallPaths` is the one place
    /// that decides where a config directory is, and this does not second-guess
    /// it.
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        Self::from_frontend(
            &config.realtime,
            config.paths.config_directory().join(LOCAL_MODEL_DIRECTORY),
            config.locale,
        )
    }

    /// Override the derived mode.
    #[must_use]
    pub fn with_mode(mut self, mode: LocalMode) -> Self {
        self.mode = mode;
        self
    }

    /// Point the pipeline at a different model root.
    #[must_use]
    pub fn with_model_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.weights = WeightsSet::discover(root);
        self
    }

    /// Replace the resolved weights wholesale.
    ///
    /// For a deployment whose files are not laid out conventionally: one field
    /// moves, the rest are inherited.
    #[must_use]
    pub fn with_weights(mut self, weights: WeightsSet) -> Self {
        self.weights = weights;
        self
    }

    /// Point the endpoint provider somewhere, and select that mode.
    #[must_use]
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into().trim().to_owned();
        self.mode = LocalMode::Endpoint;
        self
    }

    /// The locale every composed sentence is rendered in.
    #[must_use]
    pub fn with_locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// Whether an endpoint was configured at all.
    #[must_use]
    pub fn endpoint_configured(&self) -> bool {
        !self.endpoint.trim().is_empty()
    }

    /// The model id this provider publishes.
    ///
    /// `None` only for a pipeline whose reasoning path has no usable stem —
    /// which is why [`RealtimeProvider::model`] is `Option` and why the model
    /// profile is `None` in the same state rather than an invented id with real
    /// capability flags attached to nothing.
    ///
    /// [`RealtimeProvider::model`]: via_realtime::RealtimeProvider::model
    #[must_use]
    pub fn resolved_model(&self) -> Option<String> {
        let configured = self.model.trim();
        if !configured.is_empty() {
            return Some(configured.to_owned());
        }
        match self.mode {
            LocalMode::Pipeline => self.weights.model_id(),
            LocalMode::Endpoint => Some(DEFAULT_LOCAL_ENDPOINT_MODEL.to_owned()),
        }
    }

    /// The voice, with `via-catalog`'s `local`-family default beneath it.
    #[must_use]
    pub fn resolved_voice(&self) -> &str {
        let configured = self.voice.trim();
        if configured.is_empty() {
            DEFAULT_LOCAL_REALTIME_VOICE
        } else {
            configured
        }
    }

    /// The identity shape `local-omni` hashes under.
    ///
    /// The catalog's, so adding a provider never has to touch the signature
    /// code. `local-omni` is one of the catalog's own rows, so the lookup
    /// cannot miss; naming the shape it would have found keeps this total
    /// without an `expect()`.
    #[must_use]
    pub fn identity_shape(&self) -> IdentityShape {
        realtime_provider_definition(PROVIDER_KEY)
            .map_or(IdentityShape::ModelAndVoice, |row| row.identity_shape)
    }

    /// The `sha256` clients compare to detect that the voice configuration
    /// changed under them.
    ///
    /// The **mode** is part of the endpoint half of the identity, because
    /// switching between the pipeline and a local server changes everything a
    /// client can observe — the sample rates it must capture at, the voice it
    /// will hear, whether there is a server at all. A signature that did not
    /// move would leave a client's cached configuration silently wrong, which
    /// is the exact failure the signature exists to prevent.
    #[must_use]
    pub fn configuration_signature(&self) -> String {
        let endpoint = match self.mode {
            LocalMode::Pipeline => self.mode.qualified(),
            LocalMode::Endpoint => format!("{}|{}", self.mode.qualified(), self.endpoint),
        };
        RealtimeIdentity::new(
            self.identity_shape(),
            PROVIDER_KEY,
            endpoint,
            // The raw overrides, empty included: a default this crate applies is
            // not part of the operator's configuration, so applying it would
            // make the signature change when this crate changed rather than
            // when the configuration did.
            self.model.as_str(),
            self.voice.as_str(),
            self.auth_token.expose(),
        )
        .signature()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn frontend(url: &str, model: &str) -> RealtimeFrontend {
        RealtimeFrontend {
            dashscope_realtime_url: url.to_owned(),
            dashscope_model: model.to_owned(),
            ..RealtimeFrontend::default()
        }
    }

    #[test]
    fn an_empty_environment_is_the_pipeline_with_no_endpoint() {
        let settings = LocalSettings::from_frontend(
            &frontend("", DEFAULT_DASHSCOPE_REALTIME_MODEL),
            "/models/local-omni",
            Locale::En,
        );
        assert_eq!(settings.mode, LocalMode::Pipeline);
        assert!(!settings.endpoint_configured());
        assert_eq!(settings.model, "");
        assert!(settings.auth_token.is_empty());
    }

    #[test]
    fn a_configured_endpoint_selects_endpoint_mode() {
        let settings = LocalSettings::from_frontend(
            &frontend("ws://127.0.0.1:8000/v1/realtime", "sgl-omni"),
            "/models/local-omni",
            Locale::Ko,
        );
        assert_eq!(settings.mode, LocalMode::Endpoint);
        assert!(settings.endpoint_configured());
        assert_eq!(settings.resolved_model().as_deref(), Some("sgl-omni"));
        assert_eq!(settings.locale, Locale::Ko);
    }

    #[test]
    fn the_dashscope_shaped_default_model_is_read_as_unset() {
        // `resolve_realtime_frontend` records DashScope's default whichever
        // provider is active; treating it as "unset" is what lets this
        // provider's own default apply.
        let settings = LocalSettings::from_frontend(
            &frontend("ws://127.0.0.1:8000", DEFAULT_DASHSCOPE_REALTIME_MODEL),
            "/models/local-omni",
            Locale::En,
        );
        assert_eq!(settings.model, "");
        assert_eq!(
            settings.resolved_model().as_deref(),
            Some(DEFAULT_LOCAL_ENDPOINT_MODEL)
        );
    }

    #[test]
    fn a_pipeline_publishes_the_reasoning_files_stem_as_its_model_id() {
        let settings = LocalSettings::default().with_weights(WeightsSet {
            reasoning: PathBuf::from("/m/reasoning/Qwen3-8B-Q4_K_M.gguf"),
            ..WeightsSet::resolve("/m")
        });
        assert_eq!(settings.mode, LocalMode::Pipeline);
        assert_eq!(
            settings.resolved_model().as_deref(),
            Some("Qwen3-8B-Q4_K_M")
        );
    }

    #[test]
    fn an_explicit_model_override_wins_in_either_mode() {
        for mode in LocalMode::ALL {
            let settings = LocalSettings {
                model: "  my-model  ".to_owned(),
                ..LocalSettings::default()
            }
            .with_mode(mode);
            assert_eq!(
                settings.resolved_model().as_deref(),
                Some("my-model"),
                "{mode}"
            );
        }
    }

    #[test]
    fn a_pipeline_with_no_usable_reasoning_stem_has_no_model_id() {
        let settings = LocalSettings::default().with_weights(WeightsSet {
            reasoning: PathBuf::from("/"),
            ..WeightsSet::resolve("/m")
        });
        assert_eq!(settings.resolved_model(), None);
    }

    #[test]
    fn the_voice_falls_back_to_the_catalogs_local_family_default() {
        assert_eq!(
            LocalSettings::default().resolved_voice(),
            DEFAULT_LOCAL_REALTIME_VOICE
        );
        let settings = LocalSettings {
            voice: "  af_bella ".to_owned(),
            ..LocalSettings::default()
        };
        assert_eq!(settings.resolved_voice(), "af_bella");
    }

    #[test]
    fn switching_mode_moves_the_signature() {
        let pipeline = LocalSettings::default();
        let endpoint = pipeline.clone().with_endpoint("ws://127.0.0.1:8000");
        assert_ne!(
            pipeline.configuration_signature(),
            endpoint.configuration_signature(),
            "a client's cached configuration must not survive a mode switch"
        );
    }

    #[test]
    fn moving_the_endpoint_moves_the_signature_and_a_repeat_does_not() {
        let first = LocalSettings::default().with_endpoint("ws://127.0.0.1:8000");
        let same = LocalSettings::default().with_endpoint("ws://127.0.0.1:8000");
        let other = LocalSettings::default().with_endpoint("ws://127.0.0.1:9000");
        assert_eq!(
            first.configuration_signature(),
            same.configuration_signature()
        );
        assert_ne!(
            first.configuration_signature(),
            other.configuration_signature()
        );
    }

    #[test]
    fn the_signature_hashes_the_raw_override_not_the_applied_default() {
        // Setting the voice to exactly the default must still change the
        // signature: the operator configured something, and a client's cache
        // has to notice.
        let unset = LocalSettings::default();
        let set = LocalSettings {
            voice: DEFAULT_LOCAL_REALTIME_VOICE.to_owned(),
            ..LocalSettings::default()
        };
        assert_eq!(unset.resolved_voice(), set.resolved_voice());
        assert_ne!(
            unset.configuration_signature(),
            set.configuration_signature()
        );
    }

    #[test]
    fn the_credential_is_never_printable() {
        let settings = LocalSettings {
            auth_token: Secret::new("local-token-secret"),
            ..LocalSettings::default()
        };
        let rendered = format!("{settings:?}");
        assert!(!rendered.contains("local-token-secret"), "{rendered}");
    }

    #[test]
    fn with_endpoint_trims_and_selects_the_mode() {
        let settings = LocalSettings::default().with_endpoint("  ws://127.0.0.1:8000  ");
        assert_eq!(settings.endpoint, "ws://127.0.0.1:8000");
        assert_eq!(settings.mode, LocalMode::Endpoint);
    }

    #[test]
    fn the_identity_shape_is_the_catalogs() {
        assert_eq!(
            LocalSettings::default().identity_shape(),
            realtime_provider_definition(PROVIDER_KEY)
                .expect("local-omni is a catalog row")
                .identity_shape
        );
    }
}
