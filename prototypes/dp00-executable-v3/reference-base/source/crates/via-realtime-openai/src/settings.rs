//! What the provider reads out of the configuration.
//!
//! ARGO's provider is constructed from a `ProviderConfig` — `api_key`,
//! `base_url`, `azure_api_version` — plus one builder knob
//! (`with_input_transcription`). VIA's configuration is a value rather than a
//! singleton, so the provider is *given* the fields it reads: the same fields,
//! resolved by [`via_core`], and nothing else.
//!
//! # Why a struct and not `&Config`
//!
//! The same three reasons `via-realtime-dashscope` gives, and the first one is
//! the one that matters: [`RealtimeProvider`] requires `Debug`, and a provider
//! holding a `String` API key prints it the first time an operator formats a
//! session with `{:?}`. [`Secret`] is the shipped answer.
//!
//! [`RealtimeProvider`]: via_realtime::RealtimeProvider

use via_catalog::{DEFAULT_DASHSCOPE_REALTIME_MODEL, DEFAULT_DASHSCOPE_REALTIME_URL};
use via_core::config::RealtimeFrontend;
use via_core::{Config, Secret};
use via_i18n::Locale;

use crate::dialect::{
    AZURE_REALTIME_API_VERSION, OpenAiDialect, strip_scheme_and_path, url_scheme,
};

/// The realtime deployment dialled when nothing chose one.
///
/// From ARGO's `DEFAULT_REALTIME_MODEL` (`argo-mobile/app/voice.tsx`), and the
/// bring-up doc says why it is the *mini* variant: *"a voice session streams
/// continuously, so the cheap variant is the right thing to iterate against."*
///
/// It must be a `gpt-realtime*` id — the `gpt-4o-realtime-preview*` family was
/// retired 2026-05-07 and the interface that served it a week later. That is
/// documented rather than enforced: on Azure the deployment name is arbitrary,
/// so a name check would refuse the primary deployment shape this provider
/// exists to reach.
pub const DEFAULT_OPENAI_REALTIME_MODEL: &str = "gpt-realtime-2.1-mini";

/// The voice used when nothing overrides it.
///
/// `alloy` predates the GA split and both generations accept it, which is the
/// same reason [`INPUT_TRANSCRIPTION_MODEL`](crate::INPUT_TRANSCRIPTION_MODEL)
/// is `whisper-1`: a default that only works on one side of the split is a
/// default that fails on the endpoint this provider was written for.
pub const DEFAULT_OPENAI_REALTIME_VOICE: &str = "alloy";

/// What [`OpenAiRealtimeProvider`](crate::OpenAiRealtimeProvider) reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenAiSettings {
    /// The endpoint, in any spelling a config surface writes.
    ///
    /// Only the **host** survives — see
    /// [`crate::strip_scheme_and_path`] — so
    /// `https://res.openai.azure.com/openai/v1` and `res.openai.azure.com` are
    /// the same endpoint, and a stored `/openai/v1` cannot double up into
    /// `/openai/v1/openai/v1/realtime`.
    ///
    /// **Empty means "nothing was configured"**, which is a different thing from
    /// `api.openai.com`: with the [`Azure`](OpenAiDialect::Azure) dialect an
    /// empty endpoint makes the provider *unconfigured* rather than dialling
    /// OpenAI. ARGO bring-up §5 records why — before that guard a missing
    /// endpoint fell back to `api.openai.com`, which answers `Incorrect API key
    /// provided`, so it arrived as an accusation about the credential from a
    /// host nobody configured, after the whole candidate walk had been spent.
    /// Two bring-up rounds were lost to it.
    pub base_url: String,

    /// The model id, or on Azure's classic routes the **deployment name**.
    ///
    /// Azure addresses a deployment, not a model, so a resource whose realtime
    /// deployment is named something other than the model id must configure that
    /// name here.
    pub model: String,

    /// The voice override, empty when the user set none.
    ///
    /// Empty is not "no voice": [`DEFAULT_OPENAI_REALTIME_VOICE`] is applied on
    /// top of it, exactly as the DashScope provider applies its profile default.
    /// The raw override — empty string included — is what the configuration
    /// signature hashes, because the default is not part of the operator's
    /// configuration.
    pub voice: String,

    /// The credential.
    ///
    /// Sent as `Authorization: Bearer` in the OpenAI dialect and as `api-key` in
    /// the Azure one — plus a bearer when the host is not a first-party Azure
    /// resource, because then it is a gateway and gateways usually want one.
    pub api_key: Secret,

    /// Which dialect to dial in.
    ///
    /// Defaulted from the host by [`OpenAiDialect::for_host`]; set explicitly to
    /// hold a non-default host to one candidate.
    pub dialect: OpenAiDialect,

    /// The `api-version` for Azure's **classic** routes only.
    ///
    /// Empty falls back to [`AZURE_REALTIME_API_VERSION`]. It is deliberately
    /// not applied to the v1 surface: Microsoft lists the parameter's presence
    /// there as the cause of a `401`, which reads exactly like a bad credential.
    pub azure_api_version: String,

    /// Ask the provider to transcribe the **user's** audio.
    ///
    /// ARGO defaults this **off** — it is an extra billed model on the provider
    /// side, and a session that never renders the user's words must not pay for
    /// them. VIA defaults it **on**, and `docs/deviations/phase-6-via-realtime-openai.md`
    /// records why: `docs/architecture.md` §2 makes `dictation` a session mode
    /// whose entire output is the user's transcript, and Layer 1 publishes
    /// `conversation.item.input_audio_transcription.*` to every client. With the
    /// flag off, a `dictation` session over this provider produces nothing at
    /// all — which presents as a broken microphone rather than as a
    /// configuration.
    pub transcribe_input: bool,

    /// The locale the two provider-supplied sentences are rendered in.
    pub locale: Locale,
}

impl Default for OpenAiSettings {
    /// The settings an empty environment resolves to: OpenAI's own host, the
    /// default deployment, no credential.
    fn default() -> Self {
        Self {
            base_url: String::new(),
            model: DEFAULT_OPENAI_REALTIME_MODEL.to_owned(),
            voice: String::new(),
            api_key: Secret::default(),
            dialect: OpenAiDialect::OpenAi,
            azure_api_version: AZURE_REALTIME_API_VERSION.to_owned(),
            transcribe_input: true,
            locale: Locale::En,
        }
    }
}

impl OpenAiSettings {
    /// Read the resolved realtime frontend.
    ///
    /// # The one mapping that is not one-to-one
    ///
    /// [`RealtimeFrontend`] is provider-agnostic but its field *names* are
    /// DashScope's: `VIA_REALTIME_BASE_URL` / `VIA_REALTIME_URL` land in
    /// `dashscope_realtime_url` and `VIA_REALTIME_MODEL` in `dashscope_model`,
    /// whichever provider is active — `via_core::config::resolve_realtime_frontend`
    /// reads one environment chain for all five providers. What differs is only
    /// the **default** each falls back to, and `RealtimeFrontend` records
    /// DashScope's.
    ///
    /// So a value still equal to the DashScope default means *"the operator set
    /// nothing"*, and this substitutes OpenAI's default for it. An operator who
    /// deliberately points this provider at `dashscope.aliyuncs.com` is the one
    /// case that mis-reads, and it is not a configuration anyone has.
    /// `docs/deviations/phase-6-via-realtime-openai.md` records the alternative
    /// (`openai_*` fields on `RealtimeFrontend`) and why it was not taken:
    /// `via-core` is shipped, and this crate must not edit it.
    #[must_use]
    pub fn from_frontend(frontend: &RealtimeFrontend, locale: Locale) -> Self {
        let base_url = if frontend.dashscope_realtime_url == DEFAULT_DASHSCOPE_REALTIME_URL {
            String::new()
        } else {
            frontend.dashscope_realtime_url.clone()
        };
        let model = if frontend.dashscope_model == DEFAULT_DASHSCOPE_REALTIME_MODEL {
            DEFAULT_OPENAI_REALTIME_MODEL.to_owned()
        } else {
            frontend.dashscope_model.clone()
        };
        let dialect = OpenAiDialect::for_host(&strip_scheme_and_path(&base_url));
        Self {
            base_url,
            model,
            // `VIA_LOCAL_REALTIME_VOICE`: `resolve_realtime_frontend` scopes the
            // voice override by model family, and `openai` carries user-chosen
            // model coordinates, so it reads the `local` family's variable.
            voice: frontend.dashscope_voice.clone(),
            api_key: frontend.dashscope_api_key.clone(),
            dialect,
            azure_api_version: AZURE_REALTIME_API_VERSION.to_owned(),
            transcribe_input: true,
            locale,
        }
    }

    /// Read a whole configuration.
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        Self::from_frontend(&config.realtime, config.locale)
    }

    /// The host every candidate URL is built against.
    ///
    /// An unconfigured endpoint resolves to [`OPENAI_REALTIME_HOST`], which is
    /// correct for the [`OpenAi`](OpenAiDialect::OpenAi) dialect and refused by
    /// [`is_configured`](via_realtime::RealtimeProvider::is_configured) for the
    /// Azure one.
    ///
    /// [`OPENAI_REALTIME_HOST`]: crate::OPENAI_REALTIME_HOST
    #[must_use]
    pub fn host(&self) -> String {
        let host = strip_scheme_and_path(&self.base_url);
        if host.is_empty() {
            return crate::dialect::OPENAI_REALTIME_HOST.to_owned();
        }
        host
    }

    /// The URL scheme every candidate is built with.
    ///
    /// `wss` unless the endpoint was configured as `ws://` or `http://` — see
    /// [`INSECURE_SCHEME`](crate::dialect::INSECURE_SCHEME) for why a plaintext
    /// loopback gateway has to stay reachable.
    #[must_use]
    pub fn scheme(&self) -> &'static str {
        url_scheme(&self.base_url)
    }

    /// Whether an endpoint was configured at all.
    #[must_use]
    pub fn endpoint_configured(&self) -> bool {
        !strip_scheme_and_path(&self.base_url).is_empty()
    }

    /// The `api-version` for Azure's classic routes, with the default beneath
    /// it.
    ///
    /// A config surface that writes an empty string must not produce
    /// `api-version=`, which Azure rejects as malformed.
    #[must_use]
    pub fn api_version(&self) -> &str {
        let configured = self.azure_api_version.trim();
        if configured.is_empty() {
            AZURE_REALTIME_API_VERSION
        } else {
            configured
        }
    }

    /// The model id as it goes on the wire, with the default beneath it.
    #[must_use]
    pub fn resolved_model(&self) -> &str {
        let configured = self.model.trim();
        if configured.is_empty() {
            DEFAULT_OPENAI_REALTIME_MODEL
        } else {
            configured
        }
    }

    /// The voice override, or the default beneath it.
    #[must_use]
    pub fn resolved_voice(&self) -> &str {
        let configured = self.voice.trim();
        if configured.is_empty() {
            DEFAULT_OPENAI_REALTIME_VOICE
        } else {
            configured
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::dialect::OPENAI_REALTIME_HOST;

    #[test]
    fn an_empty_environment_resolves_to_openais_own_host_and_deployment() {
        let settings = OpenAiSettings::from_frontend(&RealtimeFrontend::default(), Locale::En);
        assert_eq!(settings.base_url, "");
        assert_eq!(settings.host(), OPENAI_REALTIME_HOST);
        assert!(!settings.endpoint_configured());
        assert_eq!(settings.resolved_model(), DEFAULT_OPENAI_REALTIME_MODEL);
        assert_eq!(settings.resolved_voice(), DEFAULT_OPENAI_REALTIME_VOICE);
        assert_eq!(settings.dialect, OpenAiDialect::OpenAi);
        assert!(settings.api_key.is_empty());
    }

    #[test]
    fn a_configured_gateway_endpoint_selects_the_azure_dialect_and_its_own_host() {
        let frontend = RealtimeFrontend {
            dashscope_realtime_url: "https://sr-aic-llm-proxy.example/openai/v1".to_owned(),
            dashscope_model: "gpt-realtime-2.1".to_owned(),
            ..RealtimeFrontend::default()
        };
        let settings = OpenAiSettings::from_frontend(&frontend, Locale::Ko);
        assert_eq!(settings.host(), "sr-aic-llm-proxy.example");
        assert!(settings.endpoint_configured());
        assert_eq!(settings.resolved_model(), "gpt-realtime-2.1");
        assert_eq!(
            settings.dialect,
            OpenAiDialect::Azure,
            "any host that is not api.openai.com needs the four-candidate walk"
        );
        assert_eq!(settings.locale, Locale::Ko);
    }

    #[test]
    fn pointing_the_endpoint_at_openais_own_host_keeps_one_candidate() {
        let frontend = RealtimeFrontend {
            dashscope_realtime_url: "wss://api.openai.com/v1/realtime".to_owned(),
            ..RealtimeFrontend::default()
        };
        let settings = OpenAiSettings::from_frontend(&frontend, Locale::En);
        assert_eq!(settings.dialect, OpenAiDialect::OpenAi);
        assert_eq!(settings.host(), OPENAI_REALTIME_HOST);
        assert!(settings.endpoint_configured());
    }

    #[test]
    fn the_dashscope_shaped_defaults_are_read_as_unset() {
        // `resolve_realtime_frontend` records DashScope's defaults whichever
        // provider is active; substituting OpenAI's is what makes `openai`
        // usable with nothing but a key set.
        let frontend = RealtimeFrontend::default();
        assert_eq!(
            frontend.dashscope_realtime_url,
            DEFAULT_DASHSCOPE_REALTIME_URL
        );
        assert_eq!(frontend.dashscope_model, DEFAULT_DASHSCOPE_REALTIME_MODEL);

        let settings = OpenAiSettings::from_frontend(&frontend, Locale::En);
        assert_eq!(settings.base_url, "");
        assert_eq!(settings.model, DEFAULT_OPENAI_REALTIME_MODEL);
    }

    #[test]
    fn a_blank_api_version_falls_back_rather_than_producing_an_empty_parameter() {
        for configured in ["", "   ", "\n"] {
            let settings = OpenAiSettings {
                azure_api_version: configured.to_owned(),
                ..OpenAiSettings::default()
            };
            assert_eq!(settings.api_version(), AZURE_REALTIME_API_VERSION);
        }
        let settings = OpenAiSettings {
            azure_api_version: "  2024-12-17 ".to_owned(),
            ..OpenAiSettings::default()
        };
        assert_eq!(settings.api_version(), "2024-12-17");
    }

    #[test]
    fn a_blank_model_or_voice_falls_back_to_the_default() {
        let settings = OpenAiSettings {
            model: "   ".to_owned(),
            voice: "  ".to_owned(),
            ..OpenAiSettings::default()
        };
        assert_eq!(settings.resolved_model(), DEFAULT_OPENAI_REALTIME_MODEL);
        assert_eq!(settings.resolved_voice(), DEFAULT_OPENAI_REALTIME_VOICE);
    }

    #[test]
    fn a_loopback_gateway_stays_reachable_without_tls() {
        let frontend = RealtimeFrontend {
            dashscope_realtime_url: "ws://127.0.0.1:4000".to_owned(),
            ..RealtimeFrontend::default()
        };
        let settings = OpenAiSettings::from_frontend(&frontend, Locale::En);
        assert_eq!(settings.scheme(), "ws");
        assert_eq!(settings.host(), "127.0.0.1:4000");
        assert_eq!(settings.dialect, OpenAiDialect::Azure);
    }

    #[test]
    fn an_unconfigured_or_https_endpoint_keeps_tls() {
        assert_eq!(OpenAiSettings::default().scheme(), "wss");
        assert_eq!(
            OpenAiSettings {
                base_url: "https://r.openai.azure.com".to_owned(),
                ..OpenAiSettings::default()
            }
            .scheme(),
            "wss"
        );
    }

    #[test]
    fn the_credential_is_never_printable() {
        let settings = OpenAiSettings {
            api_key: Secret::new("sk-super-secret"),
            ..OpenAiSettings::default()
        };
        let rendered = format!("{settings:?}");
        assert!(!rendered.contains("sk-super-secret"), "{rendered}");
    }

    #[test]
    fn user_transcription_is_on_by_default_because_dictation_is_a_mode() {
        assert!(OpenAiSettings::default().transcribe_input);
        assert!(
            OpenAiSettings::from_frontend(&RealtimeFrontend::default(), Locale::En)
                .transcribe_input
        );
    }
}
