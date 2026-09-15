//! What each provider reads out of the configuration.
//!
//! Upstream's providers close over the `config` module singleton and read
//! `config.audioModel`, `config.audioVoice`, `config.audioRealtimeBaseUrl`,
//! `config.dashscopeApiKey`, `config.speechToSpeechRealtimeUrl`,
//! `config.speechToSpeechAuthToken` and `config.speechToSpeechConfigured`
//! whenever they are asked a question. VIA's configuration is a value rather
//! than a singleton, so a provider is *given* the fields it reads — the same
//! fields, resolved by [`via_core`], and nothing else.
//!
//! # Why a struct and not `&Config`
//!
//! Three reasons, in order of weight:
//!
//! 1. **The credential must not be printable.** [`RealtimeProvider`] requires
//!    `Debug`, and a provider that holds a `String` API key prints it the first
//!    time an operator formats a session with `{:?}`. [`Secret`] is the shipped
//!    answer and it is what these structs hold.
//! 2. **A test should not have to build a `Config`.** Every upstream provider
//!    test is "set two fields, ask one question"; a settings literal is that.
//! 3. **The read set is the documentation.** Six fields, listed once, is a
//!    checkable statement about what a provider depends on.
//!
//! [`RealtimeProvider`]: via_realtime::RealtimeProvider

use via_catalog::{DEFAULT_DASHSCOPE_REALTIME_MODEL, DEFAULT_DASHSCOPE_REALTIME_URL};
use via_core::config::RealtimeFrontend;
use via_core::{Config, Secret};
use via_i18n::Locale;

/// What [`DashScopeProvider`](crate::DashScopeProvider) reads.
///
/// The field names are VIA's; each one names the upstream `config` property it
/// stands for in its own documentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashScopeSettings {
    /// Upstream `config.audioRealtimeBaseUrl` — the endpoint before
    /// [`realtime_url`](via_catalog::realtime_provider::realtime_url) appends
    /// `?model=`. Already trimmed of a trailing `?` by
    /// [`via_core::config::resolve_realtime_frontend`].
    pub base_url: String,

    /// Upstream `config.audioModel` — the id as configured, **not** resolved
    /// against the catalog. An id that is not in the catalog survives to
    /// [`preflight`](via_realtime::RealtimeProvider::preflight), which is where
    /// it is refused.
    pub model: String,

    /// Upstream `config.audioVoice` — the *family-scoped override*, empty when
    /// the user set neither `VIA_REALTIME_VOICE` nor `VIA_OMNI_REALTIME_VOICE`.
    ///
    /// Empty is not "no voice": the profile default is applied on top of it in
    /// [`voice`](via_realtime::RealtimeProvider::voice), exactly as upstream's
    /// `config.audioVoice || profile.sessionDefaults.voice` does. The raw
    /// override — empty string included — is what the configuration signature
    /// hashes.
    pub voice: String,

    /// Upstream `config.dashscopeApiKey`.
    pub api_key: Secret,

    /// The locale the two provider-supplied sentences are rendered in.
    ///
    /// Upstream has one locale and interpolates the sentence directly;
    /// `docs/architecture.md` §16 gives VIA three, and
    /// `RealtimeError::{NotConfigured, ConnectTimeout}` carry the provider's
    /// already-localized string rather than a key.
    pub locale: Locale,
}

impl Default for DashScopeSettings {
    /// The settings an empty environment resolves to.
    ///
    /// Every literal comes from [`via_catalog`]; nothing is retyped.
    fn default() -> Self {
        Self {
            base_url: DEFAULT_DASHSCOPE_REALTIME_URL.to_owned(),
            model: DEFAULT_DASHSCOPE_REALTIME_MODEL.to_owned(),
            voice: String::new(),
            api_key: Secret::default(),
            locale: Locale::En,
        }
    }
}

impl DashScopeSettings {
    /// Read the DashScope half of a resolved realtime frontend.
    ///
    /// Every field is present on [`RealtimeFrontend`] regardless of which
    /// provider is active — upstream publishes both providers' values side by
    /// side so a settings UI can offer both — so this never depends on
    /// `dashscope` being the selected provider.
    #[must_use]
    pub fn from_frontend(frontend: &RealtimeFrontend, locale: Locale) -> Self {
        Self {
            base_url: frontend.dashscope_realtime_url.clone(),
            model: frontend.dashscope_model.clone(),
            voice: frontend.dashscope_voice.clone(),
            api_key: frontend.dashscope_api_key.clone(),
            locale,
        }
    }

    /// Read a whole configuration.
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        Self::from_frontend(&config.realtime, config.locale)
    }
}

/// What [`SpeechToSpeechProvider`](crate::SpeechToSpeechProvider) reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeechToSpeechSettings {
    /// Upstream `config.speechToSpeechRealtimeUrl`. Already trimmed of trailing
    /// `/` by [`via_core::config::resolve_realtime_frontend`], and interpolated
    /// into the connect-timeout sentence as well as dialled.
    pub url: String,

    /// Upstream `config.speechToSpeechAuthToken`. Optional: an `Authorization`
    /// header is sent **only** when this is set.
    pub auth_token: Secret,

    /// Upstream `config.speechToSpeechConfigured`.
    ///
    /// A default endpoint alone is deliberately not a configuration — upstream's
    /// own comment is *"do not advertise a local service merely because a
    /// default endpoint exists"* — which is why this is a resolved flag rather
    /// than `!url.is_empty()`.
    pub configured: bool,

    /// The locale the two provider-supplied sentences are rendered in.
    pub locale: Locale,
}

impl Default for SpeechToSpeechSettings {
    fn default() -> Self {
        Self {
            url: via_catalog::DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL.to_owned(),
            auth_token: Secret::default(),
            configured: false,
            locale: Locale::En,
        }
    }
}

impl SpeechToSpeechSettings {
    /// Read the speech-to-speech half of a resolved realtime frontend.
    #[must_use]
    pub fn from_frontend(frontend: &RealtimeFrontend, locale: Locale) -> Self {
        Self {
            url: frontend.speech_to_speech_realtime_url.clone(),
            auth_token: frontend.speech_to_speech_auth_token.clone(),
            configured: frontend.speech_to_speech_configured,
            locale,
        }
    }

    /// Read a whole configuration.
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        Self::from_frontend(&config.realtime, config.locale)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn the_defaults_are_the_catalogs_defaults() {
        let dashscope = DashScopeSettings::default();
        assert_eq!(dashscope.base_url, DEFAULT_DASHSCOPE_REALTIME_URL);
        assert_eq!(dashscope.model, DEFAULT_DASHSCOPE_REALTIME_MODEL);
        assert!(dashscope.voice.is_empty());
        assert!(dashscope.api_key.is_empty());

        let s2s = SpeechToSpeechSettings::default();
        assert_eq!(s2s.url, via_catalog::DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL);
        assert!(!s2s.configured, "a default endpoint is not a configuration");
    }

    #[test]
    fn a_default_frontend_and_default_settings_agree() {
        let frontend = RealtimeFrontend::default();
        assert_eq!(
            DashScopeSettings::from_frontend(&frontend, Locale::En),
            DashScopeSettings::default()
        );
        assert_eq!(
            SpeechToSpeechSettings::from_frontend(&frontend, Locale::En),
            SpeechToSpeechSettings::default()
        );
    }

    #[test]
    fn neither_settings_struct_can_print_its_credential() {
        let dashscope = DashScopeSettings {
            api_key: Secret::new("sk-live-secret"),
            voice: "longanqian".to_owned(),
            ..DashScopeSettings::default()
        };
        let rendered = format!("{dashscope:?}");
        assert!(!rendered.contains("sk-live-secret"), "{rendered}");
        assert!(rendered.contains(via_core::secret::REDACTED), "{rendered}");

        let s2s = SpeechToSpeechSettings {
            auth_token: Secret::new("hf-token"),
            ..SpeechToSpeechSettings::default()
        };
        assert!(!format!("{s2s:?}").contains("hf-token"));
    }
}
