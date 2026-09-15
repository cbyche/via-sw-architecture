//! `local-omni:endpoint` — an OpenAI-Realtime client pointed at a **local**
//! server.
//!
//! # This is nearly free, and it is deliberately not a fork
//!
//! `docs/architecture.md` §7 lists this row as *thin*, and it is:
//! `via-realtime-openai` already implements the OpenAI-Realtime dialect, its
//! four-candidate walk, its first-frame probe, its GA/pre-GA schema repair and
//! its error corpus — all of it ported from ARGO and all of it paid for with
//! device runs. This module **configures that provider** and adds the one thing
//! it cannot know: that the endpoint is local.
//!
//! | Owned by `via-realtime-openai` | Owned here |
//! | --- | --- |
//! | the dialect, the candidate walk, the probe, the schema repair | the local-endpoint policy |
//! | the error corpus and the session payload | the `local`-family model profile |
//! | the subprotocol and the auth shapes | "no local server is running at X" |
//!
//! # The local-endpoint policy, in three rules
//!
//! **Plaintext loopback is normal.** `ws://127.0.0.1:8000` is the ordinary
//! configuration, not a downgrade. `OpenAiSettings::scheme` already keeps a
//! configured `ws://` or `http://` rather than forcing TLS, which is what makes
//! this work without a second URL builder.
//!
//! **No cloud credential is required.** `via-realtime-openai`'s own
//! `is_configured` demands an API key, because for OpenAI and Azure a missing
//! key is the whole failure. A loopback server on a single-user machine needs
//! none, so this provider's `is_configured` asks for an **endpoint** instead —
//! and refuses to guess one, because `via-catalog`'s `local-omni` row declares
//! no default URL on purpose.
//!
//! **A refusal says the server is not running.** A local server that is not
//! started answers `Connection refused`, and a local server dialled as `wss://`
//! answers a TLS error about a certificate nobody issued. Rendering either
//! verbatim sends an operator to debug the wrong thing.
//! [`diagnose_connect_failure`] turns the first into *"no local server is
//! running at X"* and the second into *"…did not complete a TLS handshake; a
//! local server usually speaks plain `ws://`"*, and leaves a failure the server
//! **answered** — an HTTP status — exactly as it was, because then the server
//! is running and the status is the diagnosis.
//!
//! # It is unverified on this machine, and it says so
//!
//! `docs/architecture.md` §7: *"CUDA-only; cannot be verified on this
//! machine."* `sgl-omni serve --enable-realtime` runs Qwen3-Omni-30B-A3B, which
//! needs a CUDA host; this repository has none. Everything below is written from
//! the OpenAI-Realtime specification and from `via-realtime-openai`'s
//! device-proven behaviour, and **no part of it has been run against a real
//! local omni server.**
//!
//! That statement is not only in this comment. It is
//! [`LocalMode::is_verified_on_this_machine`], it is
//! [`LocalHealth::verified`](crate::LocalHealth::verified) on the `/api/health`
//! note, and it is a sentence in the message catalog
//! ([`keys::REALTIME_LOCAL_ENDPOINT_UNVERIFIED`]) so an operator selecting this
//! mode reads it in their own language.
//!
//! What *is* verified here: the provider surface, the configuration gate, and
//! the refusal diagnosis — the last against a real TCP port with nothing
//! listening on it.

use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;
use via_catalog::realtime_provider::realtime_provider_definition;
use via_catalog::{ModelProfile, local_realtime_model_profile};
use via_i18n::{Locale, format as i18n_format, keys};
use via_realtime::{
    AgentContext, ErrorClass, Injection, PermissionRequest, ProviderCapabilities, RealtimeError,
    RealtimeProtocol, RealtimeProvider, RealtimeSession, SessionEvents, SessionOptions,
    SessionRequest, Transport,
};
use via_realtime_openai::{
    ConnectBudget, OpenAiDialect, OpenAiRealtimeProvider, OpenAiSettings, SUBPROTOCOL,
    SUBPROTOCOL_HEADER, connect_within, strip_scheme_and_path,
};

use crate::error::LocalError;
use crate::mode::{LocalMode, PROVIDER_KEY};
use crate::settings::{DEFAULT_LOCAL_ENDPOINT_MODEL, ENDPOINT_ENV, LocalSettings};

/// The human label used when the catalog row is somehow unavailable.
pub const FALLBACK_LABEL: &str = "Local Omni";

/// How long to wait for `response.created` from a local server.
///
/// The pipeline's budget, for the pipeline's reason — a local model's first
/// token is not a cloud model's — and with one more of its own: a server that
/// has just started has the whole 30 B model still to page in.
pub const RESPONSE_START_TIMEOUT: Duration = crate::pipeline::RESPONSE_START_TIMEOUT;

/// Transport texts that mean nothing was listening.
///
/// Lowercase substrings of a lowercased haystack, the same shape
/// `via-realtime-openai`'s corpus uses and for the same reason: a
/// `LazyLock<Regex>` whose constructor can fail inside a diagnosis path can only
/// degrade to the message this exists to replace.
///
/// The `os error` numbers are deliberately absent: they are platform-specific
/// (61 on macOS, 111 on Linux) and every platform also renders the prose.
pub const UNREACHABLE_MARKERS: [&str; 6] = [
    "connection refused",
    "connection reset",
    "no route to host",
    "network is unreachable",
    "failed to lookup address",
    "deadline has elapsed",
];

/// Transport texts that mean TLS was attempted and did not complete.
///
/// `handshake` alone is deliberately **not** here: a WebSocket upgrade is also a
/// handshake, and matching it would relabel every rejected upgrade as a TLS
/// problem.
pub const TLS_MARKERS: [&str; 5] = [
    "invalid peer certificate",
    "certificate",
    "tls handshake",
    "unknownissuer",
    "corrupt message",
];

/// Transport texts that prove the server answered.
///
/// When one of these is present the server is running, and its status is the
/// diagnosis — so the original error stands unchanged.
///
/// The walk's own aggregate wrapper (*"no realtime endpoint completed a session
/// (N attempt(s)): …"*) is deliberately **not** on this list, and that is the
/// subtle part: the wrapper carries every rejection inside it, so a walk in
/// which *any* candidate got an HTTP status still matches on the status itself.
/// Matching the wrapper would have made every walk look answered and thrown the
/// refusal diagnosis away — which is the whole thing this module exists for.
pub const ANSWERED_MARKERS: [&str; 2] = ["http ", "unexpected server response"];

/// `local-omni` in its `endpoint` mode.
///
/// One [`RealtimeProvider`], over `via-realtime-openai`'s client.
#[derive(Clone, Debug)]
pub struct LocalEndpointProvider {
    settings: LocalSettings,
    inner: OpenAiRealtimeProvider,
    label: &'static str,
    model: String,
}

impl LocalEndpointProvider {
    /// A provider reading `settings`.
    ///
    /// The mode is forced to [`LocalMode::Endpoint`]: a provider that answered
    /// as one mode while its settings said another would publish a
    /// configuration signature for a session it is not running.
    #[must_use]
    pub fn new(settings: LocalSettings) -> Self {
        let settings = settings.with_mode(LocalMode::Endpoint);
        let model = settings
            .resolved_model()
            .unwrap_or_else(|| DEFAULT_LOCAL_ENDPOINT_MODEL.to_owned());
        let definition = realtime_provider_definition(PROVIDER_KEY).ok();
        let inner = OpenAiRealtimeProvider::new(OpenAiSettings {
            base_url: settings.endpoint.clone(),
            model: model.clone(),
            voice: settings.voice.clone(),
            api_key: settings.auth_token.clone(),
            // Any host that is not `api.openai.com` needs the four-candidate
            // walk, and a local server is by definition not that host. The walk
            // is the whole reason this mode can be pointed at a server whose
            // route nobody has told VIA about.
            dialect: OpenAiDialect::Azure,
            // A local omni server produces the user's transcript itself, and
            // `whisper-1` is an OpenAI-hosted model id such a server almost
            // certainly does not have — asking for it would fail the whole
            // `session.update`. `with_input_transcription` turns it on for a
            // gateway that does host one.
            transcribe_input: false,
            locale: settings.locale,
            ..OpenAiSettings::default()
        });
        Self {
            label: definition.map_or(FALLBACK_LABEL, |row| row.label),
            settings,
            inner,
            model,
        }
    }

    /// Ask the server to transcribe the user's own audio.
    ///
    /// Off by default — see [`new`](Self::new). Turn it on when the local server
    /// is a gateway that really does host a transcription model, which is the
    /// one configuration where a `dictation` session over this mode produces
    /// anything.
    #[must_use]
    pub fn with_input_transcription(mut self, transcribe: bool) -> Self {
        let mut settings = self.inner.settings().clone();
        settings.transcribe_input = transcribe;
        self.inner = OpenAiRealtimeProvider::new(settings);
        self
    }

    /// The settings this provider was built with.
    #[must_use]
    pub fn settings(&self) -> &LocalSettings {
        &self.settings
    }

    /// The OpenAI-Realtime client underneath.
    ///
    /// Exposed because the candidate walk takes it by reference, and because a
    /// caller debugging a local server wants its
    /// [`candidate_urls`](OpenAiRealtimeProvider::candidate_urls) and its
    /// [`schema`](OpenAiRealtimeProvider::schema).
    #[must_use]
    pub fn client(&self) -> &OpenAiRealtimeProvider {
        &self.inner
    }

    /// The endpoint as an operator reads it back: host and port, no scheme.
    #[must_use]
    pub fn endpoint(&self) -> String {
        let host = strip_scheme_and_path(&self.settings.endpoint);
        if host.is_empty() {
            return self.settings.endpoint.trim().to_owned();
        }
        format!("{}://{host}", self.inner.settings().scheme())
    }

    /// The URLs the walk dials, in order.
    #[must_use]
    pub fn candidate_urls(&self) -> Vec<String> {
        self.inner.candidate_urls()
    }

    /// The `local`-family profile for the configured model id.
    #[must_use]
    pub fn profile(&self) -> ModelProfile {
        local_realtime_model_profile(&self.model)
    }
}

impl RealtimeProvider for LocalEndpointProvider {
    fn key(&self) -> &str {
        PROVIDER_KEY
    }

    fn label(&self) -> &str {
        self.label
    }

    fn input_sample_rate(&self) -> u32 {
        self.inner.input_sample_rate()
    }

    fn output_sample_rate(&self) -> u32 {
        self.inner.output_sample_rate()
    }

    fn response_start_timeout(&self) -> Option<Duration> {
        Some(RESPONSE_START_TIMEOUT)
    }

    fn is_configured(&self) -> bool {
        // **Not** the inner provider's predicate. `via-realtime-openai` demands
        // an API key because for OpenAI and Azure a missing key is the whole
        // failure; a loopback server on a single-user machine needs none. What
        // this mode cannot do without is an address, and `via-catalog`'s
        // `local-omni` row declares no default URL to fall back to.
        self.settings.endpoint_configured()
    }

    fn model(&self) -> Option<&str> {
        Some(&self.model)
    }

    fn voice(&self) -> Option<&str> {
        self.inner.voice()
    }

    fn url(&self) -> Result<String, RealtimeError> {
        self.inner.url()
    }

    fn headers(&self) -> Vec<(String, String)> {
        // Built here rather than delegated, because the inner provider sends
        // `api-key:` with an empty value when no credential is configured — and
        // an empty credential header is the kind of thing a strict local server
        // rejects with a 400 that reads like a protocol error.
        let mut headers = Vec::new();
        if !self.settings.auth_token.is_empty() {
            headers.push((
                "Authorization".to_owned(),
                format!("Bearer {}", self.settings.auth_token.expose()),
            ));
        }
        // Offered on every candidate: it is OpenAI's own subprotocol, a server
        // that does not use it is required by RFC 6455 to ignore the field, and
        // ARGO bring-up §4 records that its absence is the difference between a
        // session and silence on a relay.
        headers.push((SUBPROTOCOL_HEADER.to_owned(), SUBPROTOCOL.to_owned()));
        headers
    }

    fn capabilities(&self) -> ProviderCapabilities {
        self.inner.capabilities()
    }

    fn model_profile(&self) -> Option<ModelProfile> {
        // The `local` family, not the inner provider's `None`. A local server's
        // model id is by definition not in the DashScope table, and upstream's
        // answer to that — an all-capabilities-false profile — is the bug
        // `docs/architecture.md` §7 says not to port.
        Some(self.profile())
    }

    fn configuration_signature(&self) -> String {
        self.settings.configuration_signature()
    }

    fn protocol(&self) -> &dyn RealtimeProtocol {
        // Whichever generation the endpoint turned out to speak: the inner
        // provider remembers what its own probe accepted.
        self.inner.protocol()
    }

    fn preflight(&self) -> Result<(), RealtimeError> {
        if self.settings.endpoint_configured() {
            return Ok(());
        }
        Err(LocalError::EndpointNotConfigured {
            variable: ENDPOINT_ENV,
        }
        .into_realtime(self.settings.locale))
    }

    fn classify_error(&self, message: &str) -> ErrorClass {
        self.inner.classify_error(message)
    }

    fn build_session(&self, request: &SessionRequest<'_>) -> Value {
        self.inner.build_session(request)
    }

    fn build_speak_response(&self, content: &str) -> Value {
        self.inner.build_speak_response(content)
    }

    fn build_result_injection(&self, content: &str) -> Injection {
        self.inner.build_result_injection(content)
    }

    fn build_permission_injection(&self, permission: &PermissionRequest) -> Injection {
        self.inner.build_permission_injection(permission)
    }

    fn missing_configuration_message(&self, locale: Locale) -> String {
        i18n_format(
            locale,
            keys::REALTIME_LOCAL_ENDPOINT_MISSING,
            &[("variable", ENDPOINT_ENV)],
        )
    }

    fn connect_timeout_message(&self, locale: Locale) -> String {
        i18n_format(
            locale,
            keys::REALTIME_LOCAL_CONNECT_TIMEOUT,
            &[("endpoint", &self.endpoint())],
        )
    }
}

/// Turn a transport failure into the sentence a local operator needs.
///
/// `None` means "leave it alone": either the server answered — an HTTP status
/// is present, so it *is* running and the status is the diagnosis — or the text
/// says nothing this can improve on.
///
/// The order is the contract. `answered` is checked **first**, because a
/// candidate walk's aggregate carries every rejection it collected and one of
/// those may well mention a refused connection on a route that does not exist;
/// relabelling the whole walk as "nothing is listening" would throw away the
/// 401 that is the real answer.
#[must_use]
pub fn diagnose_connect_failure(endpoint: &str, detail: &str) -> Option<LocalError> {
    let lowered = detail.to_ascii_lowercase();
    if ANSWERED_MARKERS
        .iter()
        .any(|marker| lowered.contains(marker))
    {
        return None;
    }
    if TLS_MARKERS.iter().any(|marker| lowered.contains(marker)) {
        return Some(LocalError::TlsRefused {
            endpoint: endpoint.to_owned(),
            detail: detail.to_owned(),
        });
    }
    if UNREACHABLE_MARKERS
        .iter()
        .any(|marker| lowered.contains(marker))
    {
        return Some(LocalError::ServerUnreachable {
            endpoint: endpoint.to_owned(),
            detail: detail.to_owned(),
        });
    }
    None
}

/// Re-render a [`RealtimeError`] from a local connect attempt.
///
/// Anything that is not a transport failure passes through untouched: a
/// `NotConfigured` already carries this crate's own sentence, and a
/// `ConnectTimeout` already names the endpoint.
#[must_use]
pub fn localize_connect_failure(
    provider: &LocalEndpointProvider,
    error: RealtimeError,
    locale: Locale,
) -> RealtimeError {
    let RealtimeError::Transport { detail } = &error else {
        return error;
    };
    match diagnose_connect_failure(&provider.endpoint(), detail) {
        Some(local) => local.into_realtime(locale),
        None => error,
    }
}

/// Walk the local server's candidate routes and hand back a proved transport.
///
/// The walk is `via-realtime-openai`'s, unchanged — including its first-frame
/// probe, which is what stops a completed 101 from being mistaken for a session.
/// What this adds is the diagnosis on the way out.
///
/// # Errors
///
/// [`LocalError::ServerUnreachable`] or [`LocalError::TlsRefused`] as a
/// [`RealtimeError::Transport`] carrying the local sentence, or the walk's own
/// aggregate when the server answered.
pub async fn connect(
    provider: &LocalEndpointProvider,
    agent_context: &AgentContext,
    budget: ConnectBudget,
) -> Result<Transport, RealtimeError> {
    connect_within(provider.client(), agent_context, budget)
        .await
        .map_err(|error| localize_connect_failure(provider, error, provider.settings.locale))
}

/// Walk the candidates and configure a session on whichever one answers.
///
/// The order is `via-realtime`'s: [`preflight`](RealtimeProvider::preflight),
/// then [`is_configured`](RealtimeProvider::is_configured), then the socket — a
/// session that is going to fail for a known reason never opens one.
///
/// The session that comes back is bound to **this** provider, not to the
/// `via-realtime-openai` client underneath it, so `/api/health` reports
/// `local-omni` and the `local`-family model profile rather than `openai` and
/// none.
///
/// # Errors
///
/// [`RealtimeError::NotConfigured`] before any socket;
/// [`RealtimeError::ConnectTimeout`] when the budget runs out; the localized
/// transport failure otherwise; and whatever [`RealtimeSession::open`] refuses.
pub async fn open_session(
    provider: Arc<LocalEndpointProvider>,
    options: SessionOptions,
) -> Result<(RealtimeSession, SessionEvents), RealtimeError> {
    provider.preflight()?;
    if !provider.is_configured() {
        return Err(RealtimeError::NotConfigured {
            provider: PROVIDER_KEY.to_owned(),
            message: provider.missing_configuration_message(options.locale),
        });
    }

    let budget = options
        .connect_timeout
        .unwrap_or(via_realtime::CONNECT_TIMEOUT);
    let started = tokio::time::Instant::now();
    let walk = connect(
        provider.as_ref(),
        &options.agent_context,
        ConnectBudget::within(budget),
    );
    let transport = match tokio::time::timeout(budget, walk).await {
        Ok(result) => result?,
        Err(_elapsed) => {
            return Err(RealtimeError::ConnectTimeout {
                provider: PROVIDER_KEY.to_owned(),
                message: provider.connect_timeout_message(options.locale),
            });
        }
    };

    let mut options = options;
    options.connect_timeout = Some(budget.saturating_sub(started.elapsed()));
    let dynamic: Arc<dyn RealtimeProvider> = provider;
    RealtimeSession::open(dynamic, options, transport).await
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use via_catalog::ModelFamily;
    use via_core::Secret;
    use via_realtime::validate_realtime_provider;

    use super::*;

    fn provider() -> LocalEndpointProvider {
        LocalEndpointProvider::new(
            LocalSettings::default().with_endpoint("ws://127.0.0.1:8000/v1/realtime"),
        )
    }

    #[test]
    fn the_provider_validates_and_keeps_the_local_key() {
        let provider = provider();
        assert_eq!(validate_realtime_provider(&provider), Ok(()));
        assert_eq!(provider.key(), PROVIDER_KEY);
        assert_eq!(provider.label(), "Local Omni");
    }

    #[test]
    fn a_plaintext_loopback_endpoint_is_the_ordinary_configuration() {
        let provider = provider();
        assert!(provider.is_configured());
        assert_eq!(provider.endpoint(), "ws://127.0.0.1:8000");
        for url in provider.candidate_urls() {
            assert!(url.starts_with("ws://127.0.0.1:8000/"), "{url}");
        }
    }

    #[test]
    fn no_credential_is_required_and_none_is_sent() {
        let provider = provider();
        let headers = provider.headers();
        let names: Vec<&str> = headers.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names, [SUBPROTOCOL_HEADER]);
        assert!(provider.is_configured(), "a local server needs no key");
    }

    #[test]
    fn a_configured_token_rides_as_a_bearer_and_is_never_printable() {
        let provider = LocalEndpointProvider::new(
            LocalSettings {
                auth_token: Secret::new("local-token-secret"),
                ..LocalSettings::default()
            }
            .with_endpoint("ws://127.0.0.1:8000"),
        );
        let headers = provider.headers();
        assert_eq!(headers[0].0, "Authorization");
        assert_eq!(headers[0].1, "Bearer local-token-secret");
        assert_eq!(headers[1].0, SUBPROTOCOL_HEADER);
        assert!(!format!("{provider:?}").contains("local-token-secret"));
    }

    #[test]
    fn an_unconfigured_endpoint_is_refused_before_a_socket_and_names_the_variable() {
        let provider = LocalEndpointProvider::new(LocalSettings::default());
        assert!(!provider.is_configured());
        let error = provider.preflight().expect_err("no endpoint");
        assert!(error.to_string().contains(ENDPOINT_ENV), "{error}");
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            let message = provider.missing_configuration_message(locale);
            assert!(message.contains(ENDPOINT_ENV), "{locale}: {message}");
            assert!(!message.contains('{'), "{locale}: {message}");
        }
    }

    #[test]
    fn the_model_profile_is_the_local_family_never_an_all_false_one() {
        let profile = provider().model_profile().expect("a local profile");
        assert_eq!(profile.family, ModelFamily::Local);
        assert_eq!(profile.id, DEFAULT_LOCAL_ENDPOINT_MODEL);
        assert!(profile.transport_capabilities.audio_input);
        assert!(profile.model_capabilities.audio_output);
    }

    #[test]
    fn a_refused_connection_says_the_server_is_not_running() {
        let refusal = diagnose_connect_failure(
            "ws://127.0.0.1:8000",
            "IO error: Connection refused (os error 61)",
        )
        .expect("a refusal is diagnosable");
        assert_eq!(
            refusal.localized(Locale::En),
            "no local server is running at ws://127.0.0.1:8000"
        );
    }

    #[test]
    fn every_unreachable_marker_is_diagnosed_in_any_casing() {
        for marker in UNREACHABLE_MARKERS {
            for text in [marker.to_owned(), marker.to_ascii_uppercase()] {
                assert!(
                    matches!(
                        diagnose_connect_failure("ws://127.0.0.1:8000", &text),
                        Some(LocalError::ServerUnreachable { .. })
                    ),
                    "{text}"
                );
            }
        }
    }

    #[test]
    fn a_tls_failure_is_diagnosed_as_the_scheme_rather_than_as_a_missing_server() {
        for text in [
            "invalid peer certificate: UnknownIssuer",
            "tls handshake eof",
            "received corrupt message of type Handshake",
        ] {
            assert!(
                matches!(
                    diagnose_connect_failure("wss://127.0.0.1:8000", text),
                    Some(LocalError::TlsRefused { .. })
                ),
                "{text}"
            );
        }
    }

    #[test]
    fn a_websocket_handshake_rejection_is_not_mistaken_for_a_tls_failure() {
        // `handshake` alone is not a TLS marker: a rejected upgrade is a
        // handshake too, and the server that rejected it is running.
        assert_eq!(
            diagnose_connect_failure(
                "ws://127.0.0.1:8000",
                "WebSocket protocol error: Handshake not finished"
            ),
            None
        );
    }

    #[test]
    fn a_server_that_answered_keeps_its_own_status() {
        for text in [
            "HTTP 401: Incorrect API key provided",
            "Unexpected server response: 404",
            // The walk's own aggregate: the wrapper is not a marker, the
            // status inside it is.
            "no realtime endpoint completed a session (4 attempt(s)): HTTP 404 | HTTP 404",
        ] {
            assert_eq!(
                diagnose_connect_failure("ws://127.0.0.1:8000", text),
                None,
                "{text}"
            );
        }
    }

    #[test]
    fn a_text_this_cannot_improve_on_is_left_alone() {
        assert_eq!(
            diagnose_connect_failure("ws://127.0.0.1:8000", "space before the colon"),
            None
        );
    }

    #[test]
    fn only_a_transport_failure_is_re_rendered() {
        let provider = provider();
        let refused = localize_connect_failure(
            &provider,
            RealtimeError::Transport {
                detail: "Connection refused".to_owned(),
            },
            Locale::En,
        );
        assert!(
            refused
                .to_string()
                .contains("no local server is running at ws://127.0.0.1:8000"),
            "{refused}"
        );

        let other = RealtimeError::ConnectionClosed {
            label: "Local Omni".to_owned(),
        };
        assert_eq!(
            localize_connect_failure(&provider, other.clone(), Locale::En),
            other
        );
    }

    #[test]
    fn the_connect_timeout_sentence_names_the_endpoint_in_every_locale() {
        let provider = provider();
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            let message = provider.connect_timeout_message(locale);
            assert!(
                message.contains("ws://127.0.0.1:8000"),
                "{locale}: {message}"
            );
            assert!(!message.contains('{'), "{locale}: {message}");
        }
    }

    #[test]
    fn input_transcription_is_off_by_default_and_can_be_turned_on() {
        assert!(!provider().client().settings().transcribe_input);
        assert!(
            provider()
                .with_input_transcription(true)
                .client()
                .settings()
                .transcribe_input
        );
    }

    #[test]
    fn the_mode_is_forced_to_endpoint_whatever_the_settings_said() {
        let provider =
            LocalEndpointProvider::new(LocalSettings::default().with_mode(LocalMode::Pipeline));
        assert_eq!(provider.settings().mode, LocalMode::Endpoint);
    }
}
