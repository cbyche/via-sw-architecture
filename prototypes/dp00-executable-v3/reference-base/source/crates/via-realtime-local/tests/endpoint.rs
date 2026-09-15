//! `local-omni:endpoint`, as far as this machine can take it.
//!
//! `docs/architecture.md` §7: *"CUDA-only; cannot be verified on this
//! machine."* There is no local omni server here, so what is asserted is the
//! part that does not need one — and the one behaviour this crate adds on top of
//! `via-realtime-openai` is exactly that part.
//!
//! **The refusal is tested against a real socket.** A `TcpListener` is bound on
//! port 0 and then dropped, which leaves a port on `127.0.0.1` with nothing
//! behind it and no chance of a stray process answering. That is the closest
//! thing to "the operator has not started the server yet" that a test can build,
//! and it exercises the whole path: the candidate walk, the aggregate it
//! composes, the diagnosis, and the localized sentence.

mod common;

use std::sync::Arc;
use std::time::Duration;

use common::contract_of_kind;
use pretty_assertions::assert_eq;
use tokio::net::TcpListener;
use via_catalog::ModelFamily;
use via_i18n::Locale;
use via_realtime::{
    AgentContext, RealtimeError, RealtimeProvider, SessionOptions, validate_realtime_provider,
};
use via_realtime_local::{
    ENDPOINT_ENV, LocalEndpointProvider, LocalHealth, LocalMode, LocalSettings,
    diagnose_connect_failure, endpoint,
};
use via_realtime_openai::ConnectBudget;

/// A budget short enough that a refused connect finishes instantly and a hung
/// one still fails rather than hanging the suite.
const FAST: ConnectBudget = ConnectBudget {
    total: Duration::from_secs(2),
    first_frame: Duration::from_millis(200),
};

/// The whole test's bound.
const DEADLINE: Duration = Duration::from_secs(10);

/// A loopback address with nothing listening on it.
///
/// Bound and then released, so the port is one the OS just confirmed was free —
/// which is as close to "the server is not running" as a test can get without
/// guessing a port number and hoping.
async fn closed_port() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a loopback port");
    let address = listener.local_addr().expect("a bound address");
    drop(listener);
    format!("ws://{address}")
}

fn provider(endpoint: &str) -> LocalEndpointProvider {
    LocalEndpointProvider::new(LocalSettings::default().with_endpoint(endpoint))
}

// ── the surface ─────────────────────────────────────────────────────────────

#[test]
fn the_provider_validates_and_publishes_the_local_family() {
    let provider = provider("ws://127.0.0.1:8000/v1/realtime");
    assert_eq!(validate_realtime_provider(&provider), Ok(()));
    assert_eq!(provider.key(), "local-omni");

    let profile = provider.model_profile().expect("a local profile");
    assert_eq!(profile.family, ModelFamily::Local);
    assert!(
        profile.transport_capabilities.audio_input,
        "the flag upstream's all-false profile leaves false, which is the whole \
         of `connects and hears nothing`"
    );
    assert!(profile.model_capabilities.function_calling);
}

#[test]
fn the_session_payload_is_the_openai_clients_own() {
    // The point of the row being *thin*: the dialect, the schema and the payload
    // are `via-realtime-openai`'s, unchanged.
    let provider = provider("ws://127.0.0.1:8000");
    let context = AgentContext {
        instructions: "You are VIA.".to_owned(),
        ..AgentContext::default()
    };
    let session = provider.build_session(&via_realtime::SessionRequest {
        configured: false,
        agent_context: &context,
    });
    assert_eq!(session["instructions"], serde_json::json!("You are VIA."));
    assert_eq!(
        provider.client().settings().base_url,
        "ws://127.0.0.1:8000",
        "the endpoint reached the client verbatim"
    );
}

#[test]
fn the_walk_dials_the_local_host_in_plaintext_on_every_candidate() {
    let provider = provider("ws://127.0.0.1:8000");
    let candidates = provider.candidate_urls();
    assert!(
        candidates.len() > 1,
        "a local server's route is not known in advance, so the walk is what \
         finds it: {candidates:?}"
    );
    for url in &candidates {
        assert!(url.starts_with("ws://127.0.0.1:8000/"), "{url}");
    }
}

#[test]
fn an_https_endpoint_keeps_tls_and_a_plaintext_one_does_not_gain_it() {
    let secure = provider("https://gpu-box.internal:8000");
    assert!(
        secure
            .candidate_urls()
            .iter()
            .all(|url| url.starts_with("wss://")),
        "{:?}",
        secure.candidate_urls()
    );
    let plain = provider("http://127.0.0.1:8000");
    assert!(
        plain
            .candidate_urls()
            .iter()
            .all(|url| url.starts_with("ws://")),
        "{:?}",
        plain.candidate_urls()
    );
}

// ── the configuration gate ──────────────────────────────────────────────────

#[tokio::test]
async fn an_unconfigured_endpoint_never_opens_a_socket() {
    let provider = Arc::new(LocalEndpointProvider::new(LocalSettings::default()));
    let opened = tokio::time::timeout(
        DEADLINE,
        endpoint::open_session(Arc::clone(&provider), SessionOptions::default()),
    )
    .await
    .expect("the refusal is immediate — no socket is opened at all");

    let error = opened.expect_err("refused");
    assert_eq!(error.code(), "VIA_REALTIME_NOT_CONFIGURED");
    assert!(error.to_string().contains(ENDPOINT_ENV), "{error}");
}

#[test]
fn the_variable_the_refusal_names_is_the_one_via_core_reads() {
    // A refusal that names a variable nobody reads is worse than one that names
    // none. The first half is the shipped constant; the second is the
    // catalogued upstream row this is the `docs/rebrand.md` rename of, parsed
    // out of the catalogue rather than retyped.
    assert_eq!(ENDPOINT_ENV, via_core::config::names::REALTIME_BASE_URL);
    contract_of_kind("env-var", "realtime frontend environment variables")
        .assert_mentions("REALTIME_BASE_URL");
}

// ── the refusal, against a real socket ──────────────────────────────────────

#[tokio::test]
async fn a_port_with_nothing_behind_it_says_no_local_server_is_running() {
    let address = closed_port().await;
    let provider = provider(&address);
    let context = AgentContext::default();

    let error = tokio::time::timeout(DEADLINE, endpoint::connect(&provider, &context, FAST))
        .await
        .expect("the walk finished inside its bound")
        .expect_err("nothing is listening");

    let rendered = error.to_string();
    assert!(
        rendered.contains(&format!("no local server is running at {address}")),
        "the refusal must name the endpoint rather than the transport: {rendered}"
    );
    // The operator's half survives beside the user's.
    assert!(
        rendered.to_ascii_lowercase().contains("connection refused")
            || rendered.to_ascii_lowercase().contains("os error"),
        "the diagnostic detail was thrown away: {rendered}"
    );
    assert_eq!(error.code(), "VIA_REALTIME_TRANSPORT");
}

#[tokio::test]
async fn the_refusal_is_rendered_in_the_configured_locale() {
    let address = closed_port().await;
    for locale in [Locale::Zh, Locale::Ko] {
        let provider = LocalEndpointProvider::new(
            LocalSettings::default()
                .with_endpoint(&address)
                .with_locale(locale),
        );
        let error = tokio::time::timeout(
            DEADLINE,
            endpoint::connect(&provider, &AgentContext::default(), FAST),
        )
        .await
        .expect("bounded")
        .expect_err("nothing is listening");
        let rendered = error.to_string();
        // The sentence a person reads is the catalogue's, in their locale. It is
        // read out of `via-i18n` rather than retyped, so a reworded translation
        // moves the test with the catalogue rather than against it.
        let expected = via_i18n::format(
            locale,
            via_i18n::keys::REALTIME_LOCAL_SERVER_UNREACHABLE,
            &[("endpoint", address.as_str())],
        );
        assert!(rendered.contains(&expected), "{locale}: {rendered}");
        assert!(!rendered.contains('{'), "{locale}: {rendered}");
        // The English diagnostic rides *after* it, in parentheses — two
        // audiences, one string, and the operator's half is never the one the
        // user reads first.
        let user_half = rendered
            .find(&expected)
            .expect("the localized sentence is present");
        let operator_half = rendered
            .find("no local server is running")
            .expect("the diagnostic is kept");
        assert!(
            user_half < operator_half,
            "{locale}: the diagnostic must not lead: {rendered}"
        );
    }
}

#[tokio::test]
async fn a_session_opened_against_a_dead_port_refuses_rather_than_hanging() {
    let address = closed_port().await;
    let provider = Arc::new(provider(&address));
    let opened = tokio::time::timeout(
        DEADLINE,
        endpoint::open_session(
            Arc::clone(&provider),
            SessionOptions {
                connect_timeout: Some(Duration::from_secs(2)),
                ..SessionOptions::default()
            },
        ),
    )
    .await
    .expect("bounded");
    let error = opened.expect_err("nothing is listening");
    assert!(
        error.to_string().contains(&address),
        "the endpoint must be in the message: {error}"
    );
}

// ── the diagnosis, on the texts these transports actually produce ───────────

#[test]
fn a_walk_whose_every_candidate_was_refused_is_still_a_missing_server() {
    // The subtle case: the walk wraps its rejections in an aggregate. The
    // wrapper must not read as "the server answered", or the diagnosis this
    // module exists for is thrown away on the one path that needs it.
    let aggregate = "no realtime endpoint completed a session (4 attempt(s)): \
                     IO error: Connection refused (os error 61) | \
                     IO error: Connection refused (os error 61)";
    let refusal = diagnose_connect_failure("ws://127.0.0.1:8000", aggregate)
        .expect("a walk of refusals is a missing server");
    assert_eq!(
        refusal.localized(Locale::En),
        "no local server is running at ws://127.0.0.1:8000"
    );
}

#[test]
fn a_walk_in_which_one_candidate_answered_keeps_its_status() {
    let aggregate = "no realtime endpoint completed a session (4 attempt(s)): \
                     IO error: Connection refused | HTTP 404: not found";
    assert_eq!(
        diagnose_connect_failure("ws://127.0.0.1:8000", aggregate),
        None,
        "a server that answered is running, and its status is the diagnosis"
    );
}

// ── the health note ─────────────────────────────────────────────────────────

#[test]
fn the_health_note_says_the_mode_is_unverified_on_this_machine() {
    // `docs/architecture.md` §16: it "ships unverified-on-hardware, saying so".
    // Saying so in a source comment is not saying so; this is the surface an
    // operator reads.
    let health = LocalHealth::describe(
        &LocalSettings::default().with_endpoint("ws://127.0.0.1:8000"),
        Locale::En,
    );
    assert_eq!(health.mode, LocalMode::Endpoint.qualified());
    assert!(!health.verified);
    let note = health.note.expect("a note");
    assert!(note.contains("CUDA"), "{note}");
    assert!(health.configured);

    // And the pipeline, which does run here, claims no such caveat.
    let pipeline = LocalHealth::describe(&LocalSettings::default(), Locale::En);
    assert!(pipeline.verified);
    assert_eq!(pipeline.note, None);
}

#[test]
fn the_two_modes_do_not_share_a_configuration_signature() {
    // A client caches the realtime configuration against this hash. Switching
    // between an in-process pipeline and a remote server changes the sample
    // rates, the voice and whether a server is needed at all — a signature that
    // did not move would leave every cached client silently wrong.
    let pipeline = LocalSettings::default();
    let remote = LocalSettings::default().with_endpoint("ws://127.0.0.1:8000");
    assert_ne!(
        pipeline.configuration_signature(),
        remote.configuration_signature()
    );
}

// ── what it does not borrow from the OpenAI provider ────────────────────────

#[test]
fn the_credential_gate_is_the_local_one_not_the_cloud_one() {
    // `via-realtime-openai`'s own `is_configured` demands an API key. A loopback
    // server on a single-user machine has none, and requiring one would make the
    // ordinary local configuration report itself unusable.
    let provider = provider("ws://127.0.0.1:8000");
    assert!(provider.is_configured());
    assert!(
        !provider.client().is_configured(),
        "the client underneath would have refused this configuration, which is \
         exactly the policy this crate replaces"
    );
}

#[test]
fn the_response_start_budget_is_the_local_one() {
    let provider = provider("ws://127.0.0.1:8000");
    assert_eq!(
        provider.response_start_timeout(),
        Some(via_realtime_local::RESPONSE_START_TIMEOUT)
    );
    assert!(
        via_realtime_local::RESPONSE_START_TIMEOUT > via_realtime::DEFAULT_RESPONSE_START_TIMEOUT,
        "a 30 B model paging in is not a cloud model's first token"
    );
}

#[tokio::test]
async fn connecting_to_a_provider_with_no_endpoint_is_refused_before_the_walk() {
    let provider = LocalEndpointProvider::new(LocalSettings::default());
    assert!(matches!(
        provider.preflight(),
        Err(RealtimeError::NotConfigured { .. })
    ));
}
