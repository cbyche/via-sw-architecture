//! The provider seam a host plugs into.
//!
//! Ported from `server/test/realtime-provider-registry.test.mjs` plus the
//! `createProtocol` half of `server/test/realtime-provider.test.mjs`, and
//! extended with the adversarial registry input upstream's tests do not cover.
//!
//! Everything here goes through `via_realtime::extension`, which is the module a
//! host actually imports — so if a name a provider needs stops being re-exported,
//! this file stops compiling.

mod common;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::harness::{Harness, RoutedProtocol, collect_events, expect_any_frame};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use via_i18n::Locale;
use via_protocol::SessionMode;
use via_realtime::extension::{
    ConnectionInfo, Injection, ListOptions, PermissionRequest, ProviderCapabilities, RealtimeError,
    RealtimeProtocol, RealtimeProvider, RealtimeProviderRegistry, RealtimeSession, SessionOptions,
    SessionRequest, Visibility, define_realtime_provider, ga_realtime_protocol,
    openai_compatible_protocol, validate_realtime_provider,
};
use via_realtime::testing::{TestProvider, test_transport};
use via_realtime::{ErrorClass, ResponseContext, ResponseOrigin};

// ── validation at the door ──────────────────────────────────────────────────

#[test]
fn a_provider_is_defined_only_after_its_contract_is_validated() {
    let provider: Arc<dyn RealtimeProvider> = Arc::new(
        TestProvider::new("house-provider")
            .with_aliases(["house"])
            .with_protocol(Box::new(openai_compatible_protocol())),
    );
    let defined = define_realtime_provider(Arc::clone(&provider)).expect("valid");
    assert!(
        Arc::ptr_eq(&provider, &defined),
        "the same value comes back"
    );

    let broken: Arc<dyn RealtimeProvider> = Arc::new(TestProvider::new("Broken"));
    assert_eq!(
        define_realtime_provider(broken).map(|_| ()),
        Err(RealtimeError::ProviderKeyInvalid {
            key: "Broken".into()
        })
    );
}

#[test]
fn the_registry_validates_again_on_the_way_in() {
    // Belt and braces: a host may build a provider without calling
    // `define_realtime_provider`, and the table must still be clean.
    let mut registry = RealtimeProviderRegistry::new();
    assert!(
        registry
            .register(Arc::new(TestProvider::new("with space")))
            .is_err()
    );
    assert!(registry.is_empty());
    assert_eq!(registry.provider_keys(), Vec::<&str>::new());
}

#[test]
fn every_adversarial_key_is_refused_or_normalised() {
    let mut registry = RealtimeProviderRegistry::with_default("ok");
    registry
        .register(Arc::new(TestProvider::new("ok")))
        .map(|_| ())
        .expect("registers");

    for key in ["", " ", "OK2", "_ok", "-ok", "ok.2", "ök", "ok/2", "ok\n"] {
        assert!(
            registry.register(Arc::new(TestProvider::new(key))).is_err(),
            "{key:?} must not be a provider key"
        );
    }
    // …and every one of them still resolves to nothing rather than to `ok`.
    for key in ["OK2", "_ok", "ok.2"] {
        assert!(registry.resolve(Some(key)).is_err(), "{key}");
    }
    // Lookup is trim + lowercase, so these all reach the same provider.
    for spelling in ["ok", "OK", "  Ok  ", "\tok\n"] {
        assert_eq!(
            registry.resolve(Some(spelling)).expect("resolves").key(),
            "ok",
            "{spelling:?}"
        );
    }
}

#[test]
fn a_blank_alias_is_refused_rather_than_silently_dropped() {
    let mut registry = RealtimeProviderRegistry::new();
    assert_eq!(
        registry
            .register(Arc::new(
                TestProvider::new("ok").with_aliases(["fine", "   "])
            ))
            .map(|_| ()),
        Err(RealtimeError::ProviderAliasesInvalid { key: "ok".into() })
    );
    assert!(registry.is_empty());
}

#[test]
fn an_alias_is_normalised_before_it_is_claimed() {
    let mut registry = RealtimeProviderRegistry::new();
    registry
        .register(Arc::new(TestProvider::new("a").with_aliases(["  QWEN "])))
        .map(|_| ())
        .expect("registers");
    assert_eq!(registry.resolve(Some("qwen")).expect("alias").key(), "a");
    // The normalised form is what is claimed, so a differently-spelled
    // duplicate is still a duplicate.
    assert_eq!(
        registry
            .register(Arc::new(TestProvider::new("b").with_aliases(["Qwen"])))
            .map(|_| ()),
        Err(RealtimeError::ProviderNameTaken {
            name: "qwen".into()
        })
    );
}

#[test]
fn a_gateway_only_provider_is_reachable_but_never_advertised() {
    let mut registry = RealtimeProviderRegistry::with_default("public-provider");
    for provider in [
        TestProvider::new("public-provider"),
        TestProvider::new("private-provider").with_visibility(Visibility::GatewayOnly),
    ] {
        registry
            .register(Arc::new(provider))
            .map(|_| ())
            .expect("registers");
    }

    assert_eq!(
        registry
            .describe_providers(false)
            .iter()
            .map(|descriptor| descriptor.key.clone())
            .collect::<Vec<_>>(),
        ["public-provider"]
    );
    assert_eq!(
        registry
            .describe_providers(true)
            .iter()
            .map(|descriptor| descriptor.key.clone())
            .collect::<Vec<_>>(),
        ["public-provider", "private-provider"]
    );
    assert_eq!(
        registry
            .resolve(Some("private-provider"))
            .expect("resolves")
            .key(),
        "private-provider"
    );
    assert_eq!(
        registry
            .list(ListOptions {
                include_gateway_only: true,
                configured_only: true,
            })
            .len(),
        2
    );
}

#[test]
fn two_dialects_live_in_one_registry() {
    let mut registry = RealtimeProviderRegistry::with_default("beta");
    registry
        .register(Arc::new(TestProvider::new("beta")))
        .map(|_| ())
        .expect("registers");
    registry
        .register(Arc::new(TestProvider::ga("ga")))
        .map(|_| ())
        .expect("registers");

    let beta = registry.resolve(Some("beta")).expect("beta");
    let ga = registry.resolve(Some("ga")).expect("ga");
    assert!(
        beta.protocol()
            .conversation_item_id(&json!({ "type": "message" }))
            .starts_with("item_")
    );
    assert!(
        ga.protocol()
            .conversation_item_id(&json!({ "type": "message" }))
            .starts_with("msg_")
    );
    assert_ne!(beta.capabilities(), ga.capabilities());
}

// ── the per-connection dialect ──────────────────────────────────────────────

/// A provider whose adapter is built per connection and closes over its id.
#[derive(Debug)]
struct RoutingProvider {
    inner: TestProvider,
    seen: Mutex<Vec<String>>,
}

impl RoutingProvider {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: TestProvider::new("routing"),
            seen: Mutex::new(Vec::new()),
        })
    }

    fn connection_ids(&self) -> Vec<String> {
        self.seen
            .lock()
            .map(|seen| seen.clone())
            .unwrap_or_default()
    }
}

impl RealtimeProvider for RoutingProvider {
    fn key(&self) -> &str {
        self.inner.key()
    }
    fn label(&self) -> &str {
        self.inner.label()
    }
    fn input_sample_rate(&self) -> u32 {
        self.inner.input_sample_rate()
    }
    fn output_sample_rate(&self) -> u32 {
        self.inner.output_sample_rate()
    }
    fn is_configured(&self) -> bool {
        true
    }
    fn model(&self) -> Option<&str> {
        self.inner.model()
    }
    fn voice(&self) -> Option<&str> {
        None
    }
    fn url(&self) -> Result<String, RealtimeError> {
        self.inner.url()
    }
    fn headers(&self) -> Vec<(String, String)> {
        Vec::new()
    }
    fn configuration_signature(&self) -> String {
        self.inner.configuration_signature()
    }
    fn protocol(&self) -> &dyn RealtimeProtocol {
        self.inner.protocol()
    }
    fn create_protocol(&self, connection: &ConnectionInfo) -> Option<Arc<dyn RealtimeProtocol>> {
        if let Ok(mut seen) = self.seen.lock() {
            seen.push(connection.connection_id.clone());
        }
        Some(Arc::new(RoutedProtocol::new(&connection.connection_id)))
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
        self.inner.missing_configuration_message(locale)
    }
    fn connect_timeout_message(&self, locale: Locale) -> String {
        self.inner.connect_timeout_message(locale)
    }
}

#[tokio::test]
async fn a_per_connection_dialect_tags_and_filters_every_frame() {
    let provider = RoutingProvider::new();
    let (transport, mut peer) = test_transport();
    let handshake = tokio::spawn(async move {
        let start = expect_any_frame(&mut peer).await;
        let route = start["route"].as_str().expect("a route").to_owned();
        peer.send(json!({ "type": "session.created", "route": route }));
        let update = expect_any_frame(&mut peer).await;
        peer.send(json!({ "type": "session.updated", "route": route }));
        (start, update, route, peer)
    });
    let (session, events) = RealtimeSession::open(
        Arc::clone(&provider) as Arc<dyn RealtimeProvider>,
        SessionOptions::default(),
        transport,
    )
    .await
    .expect("opens");
    let (start, update, route, peer) = handshake.await.expect("handshake");
    let log = collect_events(events);

    // The adapter was built once, for this connection, and knows its id.
    assert_eq!(provider.connection_ids(), vec![session.connection_id()]);
    assert_eq!(route, session.connection_id());
    assert_eq!(start["type"], json!("start"));
    assert_eq!(update["route"], json!(route.clone()));

    // A frame for somebody else's connection is dropped rather than forwarded.
    peer.send(json!({ "type": "response.created", "response": { "id": "x" }, "route": "other" }));
    peer.send(json!({ "type": "response.created", "response": { "id": "mine" }, "route": route }));
    let created = log.wait_for_kind("response.created").await;
    assert_eq!(created.event["response"]["id"], json!("mine"));
    // …and the route is stripped before the caller sees it.
    assert!(created.event.get("route").is_none());
    assert_eq!(
        log.provider_kinds(),
        ["session.created", "session.updated", "response.created"]
    );
}

#[tokio::test]
async fn two_sessions_get_two_adapters() {
    let provider = RoutingProvider::new();
    for _ in 0..2 {
        let (transport, mut peer) = test_transport();
        let handshake = tokio::spawn(async move {
            let start = expect_any_frame(&mut peer).await;
            let route = start["route"].as_str().expect("a route").to_owned();
            peer.send(json!({ "type": "session.created", "route": route }));
            expect_any_frame(&mut peer).await;
            peer.send(json!({ "type": "session.updated", "route": route }));
            peer
        });
        let _session = RealtimeSession::open(
            Arc::clone(&provider) as Arc<dyn RealtimeProvider>,
            SessionOptions::default(),
            transport,
        )
        .await
        .expect("opens");
        let _peer = handshake.await.expect("handshake");
    }
    let ids = provider.connection_ids();
    assert_eq!(ids.len(), 2);
    assert_ne!(ids[0], ids[1], "each connection gets its own id");
}

// ── a provider with no model at all ─────────────────────────────────────────

#[test]
fn a_streaming_asr_is_a_complete_provider() {
    // `docs/architecture.md` §2: in `dictation` the realtime provider may be a
    // plain streaming ASR rather than a speech-to-speech model. Nothing about
    // the trait may require a model.
    let asr = TestProvider::new("local-asr")
        .with_model(None)
        .with_voice(None)
        .with_model_profile(None)
        .with_model_catalog([]);
    assert_eq!(validate_realtime_provider(&asr), Ok(()));
    assert_eq!(asr.model(), None);
    assert_eq!(asr.voice(), None);
    assert_eq!(asr.model_profile(), None);
    assert!(asr.model_catalog().is_empty());

    let descriptor = via_realtime::describe_provider(&asr);
    assert_eq!(descriptor.model, None);
    assert_eq!(descriptor.realtime_model_ids, None);
}

#[tokio::test]
async fn a_dictation_session_transcribes_and_nothing_else() {
    let asr = TestProvider::new("local-asr")
        .with_model(None)
        .with_voice(None)
        .with_model_profile(None)
        .shared();
    let Harness {
        session,
        mut peer,
        events,
        ..
    } = Harness::open(
        asr,
        SessionOptions {
            mode: SessionMode::Dictation,
            ..SessionOptions::default()
        },
    )
    .await;

    session.append_audio("cGNt").await.expect("append");
    session.commit_audio().await.expect("commit");
    assert_eq!(
        expect_any_frame(&mut peer).await["type"],
        json!("input_audio_buffer.append")
    );
    assert_eq!(
        expect_any_frame(&mut peer).await["type"],
        json!("input_audio_buffer.commit")
    );

    // Transcripts flow with no model turn behind them.
    peer.send(json!({
        "type": "conversation.item.input_audio_transcription.completed",
        "item_id": "item-1",
        "transcript": "hello there",
    }));
    let transcript = events
        .wait_for_kind("conversation.item.input_audio_transcription.completed")
        .await;
    assert_eq!(transcript.event["transcript"], json!("hello there"));
    assert_eq!(transcript.origin, ResponseOrigin::Model);

    // And nothing ever asked the provider for a response.
    let outcome = session
        .speak(
            "say it",
            ResponseOrigin::Agent,
            ResponseContext::new(),
            None,
        )
        .await
        .expect("call")
        .expect("an outcome");
    assert!(outcome.is_skipped());
    assert!(
        !peer
            .drain_frames()
            .iter()
            .any(|frame| frame["type"] == json!("response.create"))
    );
}

// ── the re-export surface ───────────────────────────────────────────────────

#[test]
fn the_seam_exports_everything_a_host_needs_to_write_a_provider() {
    // A compile-time assertion: each name below is what a host reaches for, and
    // dropping one from `extension` breaks this file rather than a downstream
    // crate.
    fn assert_object_safe(_: &dyn RealtimeProvider, _: &dyn RealtimeProtocol) {}
    let provider = TestProvider::new("host");
    assert_object_safe(&provider, &openai_compatible_protocol());
    assert_object_safe(&provider, &ga_realtime_protocol());

    let capabilities = ProviderCapabilities {
        per_response_instructions: true,
        ..ProviderCapabilities::DEFAULT
    };
    assert!(capabilities.per_response_instructions);
    assert_eq!(Visibility::default(), Visibility::Public);
    assert_eq!(
        ConnectionInfo {
            connection_id: "c".into()
        }
        .connection_id,
        "c"
    );
    assert_eq!(
        SessionOptions::default().connect_timeout,
        Option::<Duration>::None
    );
}

#[test]
fn a_host_can_build_a_registry_from_nothing() {
    let mut registry = RealtimeProviderRegistry::with_default("house");
    let provider = define_realtime_provider(Arc::new(
        TestProvider::new("house-provider").with_aliases(["house"]),
    ))
    .expect("valid");
    registry.register(provider).map(|_| ()).expect("registers");

    assert_eq!(registry.len(), 1);
    assert_eq!(registry.default_provider(), "house");
    assert_eq!(registry.resolvable_names(), ["house", "house-provider"]);
    assert_eq!(
        registry.resolve(None).expect("default").key(),
        "house-provider"
    );

    let active = registry.describe_active_realtime(None).expect("describes");
    assert_eq!(active.provider, "house-provider");
    assert_eq!(active.configuration_signature, "test-signature");
    assert_eq!(active.providers.len(), 1);
}
