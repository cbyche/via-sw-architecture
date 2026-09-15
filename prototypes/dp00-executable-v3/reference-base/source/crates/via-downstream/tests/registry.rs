//! One dispatcher: registration validation, lookup, and what `open` refuses.

mod common;

use std::sync::Arc;

use pretty_assertions::assert_eq;
use rstest::rstest;
use via_downstream::testing::{ScriptedHarness, ScriptedTurn};
use via_downstream::{
    BackendCapabilities, DownstreamAgent, HarnessError, HarnessRegistry, SessionKey,
};
use via_i18n::Locale;

/// `Result::expect_err` needs the `Ok` type to be `Debug`, and neither trait
/// object is — deliberately, so that a session holding a credential cannot be
/// printed by accident. These two do the same job with a real message.
fn resolve_error(registry: &HarnessRegistry, configured: &str) -> HarnessError {
    match registry.resolve(configured) {
        Ok(agent) => panic!(
            "`{configured}` unexpectedly resolved to `{}`",
            agent.descriptor().id(),
        ),
        Err(error) => error,
    }
}

async fn open_error(registry: &HarnessRegistry, key: &SessionKey) -> HarnessError {
    match registry.open(key).await {
        Ok(session) => panic!(
            "`{}` unexpectedly opened session `{}`",
            key.as_str(),
            session.session_id(),
        ),
        Err(error) => error,
    }
}

fn harness(id: &str) -> Arc<dyn DownstreamAgent> {
    Arc::new(
        ScriptedHarness::builder(id)
            .turn(ScriptedTurn::completed("ok"))
            .build()
            .unwrap_or_else(|error| panic!("{id}: {error}")),
    )
}

/// Every catalogued backend can be registered, and each resolves to itself.
///
/// The mirror of upstream's *"every advertised backend has Agent and Runtime
/// drivers"* (`server/test/backend-driver-registry.test.mjs:16-29`) for the
/// half this crate owns.
#[test]
fn every_catalogued_backend_can_be_registered_and_resolved() {
    let mut registry = HarnessRegistry::new();
    for id in via_catalog::backend_names() {
        registry
            .register(harness(id))
            .unwrap_or_else(|error| panic!("{id}: {error}"));
    }
    assert_eq!(registry.len(), 12);
    assert_eq!(registry.ids(), via_catalog::backend_names());

    for id in via_catalog::backend_names() {
        let agent = registry
            .resolve(id)
            .unwrap_or_else(|error| panic!("{id}: {error}"));
        assert_eq!(agent.descriptor().id(), id);
        assert!(registry.contains(id));
    }
}

/// Lookup trims and lower-cases, as `backendDriver()` does.
#[rstest]
#[case("codex")]
#[case("CODEX")]
#[case("  Codex  ")]
#[case("\tcodex\n")]
fn lookup_normalises_the_configured_id(#[case] configured: &str) {
    let mut registry = HarnessRegistry::new();
    registry.register(harness("codex")).expect("registers");
    assert!(registry.contains(configured));
    assert_eq!(
        registry
            .resolve(configured)
            .expect("resolves")
            .descriptor()
            .id(),
        "codex",
    );
}

/// An id nobody registered is upstream's `不支持的后台 Agent：<id>`, with the
/// normalised id interpolated.
#[rstest]
#[case("ghost", "ghost")]
#[case("  GHOST  ", "ghost")]
#[case("none", "none")]
fn an_unregistered_id_is_refused(#[case] configured: &str, #[case] reported: &str) {
    let registry = HarnessRegistry::new();
    let error = resolve_error(&registry, configured);
    match &error {
        HarnessError::UnsupportedBackend { id } => assert_eq!(id, reported),
        other => panic!("wrong variant: {other:?}"),
    }
    assert_eq!(
        error.message(Locale::Zh),
        format!("不支持的后台 Agent：{reported}"),
    );
}

/// An empty id is *not configured*, which is a supported mode rather than a
/// mistake — the deviation from upstream's single message, recorded in
/// `docs/deviations/phase-2.md`.
#[rstest]
#[case("")]
#[case("   ")]
#[case("\t\n")]
fn an_empty_id_reports_that_nothing_is_configured(#[case] configured: &str) {
    let registry = HarnessRegistry::new();
    let error = resolve_error(&registry, configured);
    assert!(matches!(error, HarnessError::NotConfigured));
    assert_eq!(error.message(Locale::Zh), "当前未配置后台 Agent");
    assert!(registry.is_empty());
    assert!(!registry.contains(configured));
}

/// Registration re-validates the descriptor it is handed, so a harness that
/// hands over a descriptor for a backend it is not cannot slip through.
#[test]
fn registration_revalidates_the_descriptor() {
    let mut registry = HarnessRegistry::new();
    let bad = ScriptedHarness::builder("codex")
        .capabilities(BackendCapabilities {
            delegation: true,
            permissions: true,
            backend_ui: true,
            native_session_history: true,
            external_mcp: true,
            native_delegation: false,
            session_mcp: true,
        })
        .build();
    let error = bad.expect_err("codex has no web UI");
    assert!(matches!(error, HarnessError::IncompleteCapabilities { .. }));
    assert_eq!(
        error.message(Locale::Zh),
        "后台 Driver 能力声明不完整：codex",
    );
    assert!(registry.register(harness("codex")).is_ok());
}

/// Two harnesses claiming one id is last-wins, as upstream's `Map` is, and the
/// displaced one is handed back rather than lost.
#[test]
fn a_duplicate_registration_returns_the_harness_it_displaced() {
    let mut registry = HarnessRegistry::new();
    assert!(
        registry
            .register(harness("codex"))
            .expect("registers")
            .is_none()
    );

    let second = harness("codex");
    let displaced = registry
        .register(Arc::clone(&second))
        .expect("registers")
        .expect("something was displaced");
    assert_eq!(displaced.descriptor().id(), "codex");
    assert_eq!(registry.len(), 1);
    assert!(Arc::ptr_eq(
        registry.resolve("codex").expect("resolves"),
        &second,
    ));
}

/// Registration order is reporting order, so a diagnostic reads the same on
/// every run.
#[test]
fn registration_order_is_stable() {
    let mut registry = HarnessRegistry::new();
    for id in ["pi", "codex", "acp", "opencode"] {
        registry.register(harness(id)).expect("registers");
    }
    assert_eq!(registry.ids(), vec!["pi", "codex", "acp", "opencode"]);
}

/// `open` routes on the key's own protocol, so a key can only ever be opened
/// on the harness it names.
#[tokio::test]
async fn open_routes_on_the_keys_protocol() {
    let mut registry = HarnessRegistry::new();
    registry.register(harness("codex")).expect("registers");
    registry.register(harness("pi")).expect("registers");

    let session = registry
        .open(&SessionKey::coordinator("codex", "ana"))
        .await
        .expect("opens");
    assert_eq!(session.session_id(), "codex-session-1");

    let session = registry
        .open(&SessionKey::project("pi", "p-1"))
        .await
        .expect("opens");
    assert_eq!(session.session_id(), "pi-session-1");

    let error = open_error(&registry, &SessionKey::coordinator("ghost", "ana")).await;
    assert!(matches!(error, HarnessError::UnsupportedBackend { .. }));
}

/// A session with a blank id is upstream's invalid Profile: well-typed and
/// unusable, and refused before it reaches a caller.
#[tokio::test]
async fn an_empty_session_id_is_an_invalid_profile() {
    assert_invalid_profile("").await;
}

/// Whitespace is not an id either — the check trims first.
#[tokio::test]
async fn a_whitespace_session_id_is_an_invalid_profile() {
    assert_invalid_profile("   ").await;
}

async fn assert_invalid_profile(session_id: &str) {
    let mut registry = HarnessRegistry::new();
    registry
        .register(Arc::new(
            ScriptedHarness::builder("codex")
                .session_id(session_id)
                .build()
                .expect("declares"),
        ))
        .expect("registers");

    let error = open_error(&registry, &SessionKey::coordinator("codex", "ana")).await;
    match &error {
        HarnessError::InvalidSession { id } => assert_eq!(id, "codex"),
        other => panic!("wrong variant: {other:?}"),
    }
    assert_eq!(
        error.message(Locale::Zh),
        "后台 Driver 返回了无效 Profile：codex",
    );
}

/// The registry reports the descriptor it holds, so callers do not have to
/// resolve and unwrap to read a capability flag.
#[test]
fn the_registry_reports_descriptors() {
    let mut registry = HarnessRegistry::new();
    registry.register(harness("openclaw")).expect("registers");
    let descriptor = registry.descriptor("OpenClaw").expect("registered");
    assert_eq!(descriptor.id(), "openclaw");
    assert_eq!(descriptor.label(), "OpenClaw");
    assert!(descriptor.capabilities().backend_ui);
    assert_eq!(registry.descriptor("ghost"), None);
}

/// A registry with nothing in it is the frontend-only Gateway, not a broken
/// one.
#[test]
fn an_empty_registry_is_a_valid_state() {
    let registry = HarnessRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    assert!(registry.ids().is_empty());
    assert!(format!("{registry:?}").contains("HarnessRegistry"));
}
