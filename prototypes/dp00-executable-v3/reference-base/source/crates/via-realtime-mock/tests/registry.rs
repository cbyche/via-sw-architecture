//! The mock is a provider the registry cannot tell apart from a real one.
//!
//! It goes in through the same door every provider does
//! (`via_realtime::extension`), resolves by key and alias, filters on
//! `configured`, and publishes a `/api/health` descriptor and a configuration
//! signature. If any of that were special-cased, a Gateway test against the mock
//! would prove nothing about a Gateway against DashScope.

mod common;

use std::sync::Arc;

use pretty_assertions::assert_eq;
use via_catalog::{IdentityShape, RealtimeIdentity};
use via_realtime::extension::{
    ListOptions, RealtimeProviderRegistry, Visibility, define_realtime_provider,
};
use via_realtime::{RealtimeError, RealtimeProvider, describe_provider};
use via_realtime_mock::{
    MOCK_ENDPOINT, MOCK_MODEL, MOCK_PROVIDER_KEY, MockProvider, MockProviderSpec,
};

fn registry() -> RealtimeProviderRegistry {
    RealtimeProviderRegistry::with_default(MOCK_PROVIDER_KEY)
}

#[test]
fn the_mock_is_a_key_the_catalog_already_knows() {
    // The registry entry is not invented here: `via-catalog` ships the row.
    let row = via_catalog::realtime_provider_definition(MOCK_PROVIDER_KEY).expect("a catalog row");
    assert_eq!(row.key, MOCK_PROVIDER_KEY);
    assert_eq!(row.identity_shape, IdentityShape::EndpointOnly);
    assert_eq!(row.default_url, None, "there is no socket to default to");

    let spec = MockProviderSpec::default();
    assert_eq!(spec.key, row.key);
    assert_eq!(spec.label, row.label);
}

#[test]
fn it_registers_through_the_host_extension_door() {
    let provider = define_realtime_provider(Arc::new(MockProvider::new(
        MockProviderSpec::default().with_aliases(["replay"]),
    )))
    .expect("valid");

    let mut registry = registry();
    registry.register(provider).expect("registers");
    assert_eq!(
        registry.resolve(None).expect("the default").key(),
        MOCK_PROVIDER_KEY
    );
    assert_eq!(
        registry
            .resolve(Some("  REPLAY \n"))
            .expect("an alias")
            .key(),
        MOCK_PROVIDER_KEY,
        "alias lookup trims and lowercases"
    );
    assert!(matches!(
        registry.resolve(Some("nothing")),
        Err(RealtimeError::UnsupportedProvider { .. })
    ));
}

#[test]
fn a_second_provider_may_not_claim_the_same_name() {
    let mut registry = registry();
    let first: Arc<dyn RealtimeProvider> = Arc::new(MockProvider::new(MockProviderSpec::default()));
    registry.register(first).expect("registers");
    let second: Arc<dyn RealtimeProvider> =
        Arc::new(MockProvider::new(MockProviderSpec::default()));
    let error = registry.register(second).map(|_| ()).expect_err("refused");
    assert_eq!(
        error,
        RealtimeError::ProviderNameTaken {
            name: MOCK_PROVIDER_KEY.into()
        }
    );
}

#[test]
fn an_unconfigured_mock_is_invisible_in_a_picker_but_still_resolvable() {
    let mut registry = registry();
    registry
        .register(Arc::new(MockProvider::new(
            MockProviderSpec::default().with_configured(false),
        )))
        .expect("registers");

    assert!(
        registry
            .list(ListOptions {
                include_gateway_only: true,
                configured_only: true,
            })
            .is_empty(),
        "an unconfigured provider is not a choice that fails; it is not offered"
    );
    assert_eq!(
        registry
            .list(ListOptions {
                include_gateway_only: true,
                configured_only: false,
            })
            .len(),
        1
    );
    // …and the Gateway can still be pointed at it by name.
    assert_eq!(
        registry
            .resolve(Some(MOCK_PROVIDER_KEY))
            .expect("resolves")
            .key(),
        MOCK_PROVIDER_KEY
    );
}

#[test]
fn a_gateway_only_mock_resolves_by_name_and_is_never_listed() {
    let mut registry = registry();
    registry
        .register(Arc::new(MockProvider::new(
            MockProviderSpec::default().with_visibility(Visibility::GatewayOnly),
        )))
        .expect("registers");
    assert!(registry.describe_providers(false).is_empty());
    assert_eq!(registry.describe_providers(true).len(), 1);
    assert!(registry.resolve(Some(MOCK_PROVIDER_KEY)).is_ok());
}

#[test]
fn the_health_descriptor_publishes_the_one_model_a_conversing_mock_has() {
    let provider = MockProvider::new(MockProviderSpec::default());
    let descriptor = describe_provider(&provider);
    assert_eq!(descriptor.key, MOCK_PROVIDER_KEY);
    assert_eq!(descriptor.model.as_deref(), Some(MOCK_MODEL));
    assert_eq!(
        descriptor.realtime_model_ids,
        Some(vec![MOCK_MODEL.to_owned()])
    );
    assert!(descriptor.configured);
}

#[test]
fn a_dictation_mock_publishes_no_model_picker_at_all() {
    // `null` rather than `[]`, which a client renders as "no model picker"
    // rather than as "an empty one" — the same thing `speech-to-speech` does.
    let descriptor = describe_provider(&MockProvider::new(MockProviderSpec::dictation()));
    assert_eq!(descriptor.model, None);
    assert_eq!(descriptor.realtime_model_ids, None);
}

#[test]
fn the_active_realtime_block_carries_the_mocks_own_profile() {
    let mut registry = registry();
    registry
        .register(Arc::new(MockProvider::new(MockProviderSpec::default())))
        .expect("registers");
    let active = registry
        .describe_active_realtime(None)
        .expect("describes the default");

    assert_eq!(active.provider, MOCK_PROVIDER_KEY);
    assert_eq!(active.model.as_deref(), Some(MOCK_MODEL));
    assert_eq!(
        active.input_sample_rate,
        via_audio::SampleRate::HZ_16000.hz()
    );
    assert!(active.configured);

    // The profile is `via-catalog`'s `local` family, so it has real capability
    // flags rather than the all-false profile `docs/architecture.md` §7 refuses.
    let profile = active.model_profile.expect("a profile");
    assert_eq!(profile.id, MOCK_MODEL);
    assert!(profile.model_capabilities.function_calling);
    assert!(profile.transport_capabilities.audio_input);
    assert_eq!(active.model_catalog.len(), 1);
}

#[test]
fn the_configuration_signature_is_the_catalog_hash_over_this_providers_identity() {
    let provider = MockProvider::new(MockProviderSpec::default());
    assert_eq!(
        provider.configuration_signature(),
        RealtimeIdentity::new(
            IdentityShape::EndpointOnly,
            MOCK_PROVIDER_KEY,
            MOCK_ENDPOINT,
            MOCK_MODEL,
            "",
            "",
        )
        .signature()
    );
    // 64 lowercase hex characters — what `/api/health` publishes.
    let signature = provider.configuration_signature();
    assert_eq!(signature.len(), 64);
    assert!(
        signature
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    );
}

#[test]
fn two_mocks_that_differ_only_in_endpoint_publish_different_signatures() {
    let one = MockProvider::new(MockProviderSpec::default());
    let other = MockProvider::new(MockProviderSpec {
        endpoint: "mock://another".into(),
        ..MockProviderSpec::default()
    });
    assert_ne!(
        one.configuration_signature(),
        other.configuration_signature()
    );
}

#[test]
fn the_unsupported_provider_message_lists_what_is_registered() {
    let mut registry = registry();
    registry
        .register(Arc::new(MockProvider::new(MockProviderSpec::default())))
        .expect("registers");
    let error = registry
        .resolve(Some("dashscope"))
        .map(|_| ())
        .expect_err("refused");
    let RealtimeError::UnsupportedProvider {
        requested,
        available,
    } = error
    else {
        panic!("expected an unsupported-provider refusal");
    };
    assert_eq!(requested, "dashscope");
    assert_eq!(available, vec![MOCK_PROVIDER_KEY.to_owned()]);
}

#[test]
fn a_mock_can_take_a_key_of_its_own_beside_the_catalog_one() {
    // A test that needs two distinguishable mocks in one registry.
    let mut registry = RealtimeProviderRegistry::with_default("mock-a");
    for key in ["mock-a", "mock-b"] {
        registry
            .register(Arc::new(MockProvider::new(
                MockProviderSpec::default().with_key(key).with_label(key),
            )))
            .expect("registers");
    }
    assert_eq!(registry.len(), 2);
    assert_eq!(registry.provider_keys(), ["mock-a", "mock-b"]);
    // A key the catalog does not know falls back to the three-field identity
    // shape, which is the honest one for a provider with no credential.
    let provider = MockProvider::new(MockProviderSpec::default().with_key("mock-a"));
    assert_eq!(
        provider.configuration_signature(),
        RealtimeIdentity::new(
            IdentityShape::EndpointOnly,
            "mock-a",
            MOCK_ENDPOINT,
            MOCK_MODEL,
            "",
            "",
        )
        .signature()
    );
}
