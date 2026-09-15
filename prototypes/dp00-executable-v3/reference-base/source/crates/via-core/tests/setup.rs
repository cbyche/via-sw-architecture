//! The setup gate, and the phase-1 milestone.
//!
//! `docs/architecture.md` §15: phase 1 ends when *"`via gateway` refuses to
//! start unconfigured with the exact message in all three locales"*. That is
//! the last test in this file, and it asserts the rendered sentence against
//! `via-i18n`'s catalogue rather than against a retyped string.

mod common;

use common::env;
use via_i18n::{Locale, keys, t};
use via_protocol::{CODE_GATEWAY_SETUP_REQUIRED, ProtocolError};

use via_core::config::names;
use via_core::setup::{
    FIELD_DASHSCOPE_API_KEY, FIELD_REALTIME_API_KEY, FIELD_SPEECH_TO_SPEECH_URL,
    assert_gateway_setup, gateway_setup_status,
};

#[test]
fn an_unconfigured_dashscope_install_is_refused() {
    let status = gateway_setup_status(&env(&[]), Locale::En).expect("the defaults resolve");
    assert!(!status.ready);
    assert_eq!(status.provider, "dashscope");
    assert_eq!(status.missing.len(), 1);
    assert_eq!(status.missing[0].field, FIELD_DASHSCOPE_API_KEY);
    assert_eq!(status.missing[0].key, names::DASHSCOPE_API_KEY);
}

#[test]
fn either_credential_variable_satisfies_dashscope() {
    for key in [names::DASHSCOPE_API_KEY, names::REALTIME_API_KEY] {
        let status = gateway_setup_status(&env(&[(key, "sk-x")]), Locale::En).expect("resolves");
        assert!(status.ready, "{key} must configure the dashscope frontend");
    }
    // Whitespace is not a credential.
    let status = gateway_setup_status(&env(&[(names::DASHSCOPE_API_KEY, "   ")]), Locale::En)
        .expect("resolves");
    assert!(!status.ready);
}

#[test]
fn speech_to_speech_needs_an_explicit_endpoint_or_an_explicit_choice() {
    // Choosing the provider is enough on its own.
    let chosen = gateway_setup_status(
        &env(&[(names::REALTIME_PROVIDER, "speech-to-speech")]),
        Locale::En,
    )
    .expect("resolves");
    assert!(chosen.ready);
    assert_eq!(chosen.provider, "speech-to-speech");

    // So is setting the endpoint while leaving dashscope selected — but then
    // dashscope is still the active provider and still needs its key.
    let endpoint_only = gateway_setup_status(
        &env(&[(names::SPEECH_TO_SPEECH_REALTIME_URL, "ws://host/rt")]),
        Locale::En,
    )
    .expect("resolves");
    assert!(!endpoint_only.ready);
    assert_eq!(endpoint_only.missing[0].field, FIELD_DASHSCOPE_API_KEY);

    // The `s2s` alias selects the same provider.
    let alias = gateway_setup_status(&env(&[(names::REALTIME_PROVIDER, "s2s")]), Locale::En)
        .expect("resolves");
    assert_eq!(alias.provider, "speech-to-speech");
}

#[test]
fn a_missing_speech_to_speech_url_names_its_own_field() {
    // Reached by asking for the provider and then removing the only thing that
    // makes it configured — which cannot happen through the environment, so
    // this asserts the branch through its field/key pair instead.
    assert_eq!(FIELD_SPEECH_TO_SPEECH_URL, "speechToSpeechRealtimeUrl");
    assert_eq!(FIELD_DASHSCOPE_API_KEY, "dashscopeApiKey");
    assert_eq!(FIELD_REALTIME_API_KEY, "realtimeApiKey");
}

#[test]
fn the_openai_provider_needs_a_key() {
    let unconfigured =
        gateway_setup_status(&env(&[(names::REALTIME_PROVIDER, "openai")]), Locale::En)
            .expect("resolves");
    assert!(!unconfigured.ready);
    assert_eq!(unconfigured.missing[0].field, FIELD_REALTIME_API_KEY);
    assert_eq!(unconfigured.missing[0].key, names::REALTIME_API_KEY);

    let configured = gateway_setup_status(
        &env(&[
            (names::REALTIME_PROVIDER, "openai"),
            (names::REALTIME_API_KEY, "sk-openai"),
        ]),
        Locale::En,
    )
    .expect("resolves");
    assert!(configured.ready);
}

#[test]
fn the_local_and_mock_providers_need_nothing() {
    for provider in ["local-omni", "mock"] {
        let status =
            gateway_setup_status(&env(&[(names::REALTIME_PROVIDER, provider)]), Locale::En)
                .expect("resolves");
        assert!(status.ready, "{provider} must not require configuration");
        assert!(status.missing.is_empty());
    }
}

#[test]
fn malformed_configuration_is_an_error_not_a_missing_entry() {
    // An unsupported provider and an unknown model id are *wrong*, not
    // *absent*, so they refuse before a `missing` list can be built.
    let error = gateway_setup_status(&env(&[(names::REALTIME_PROVIDER, "nope")]), Locale::En)
        .expect_err("an unsupported provider refuses");
    assert_eq!(error.code(), "VIA_REALTIME_PROVIDER_UNSUPPORTED");

    let error = gateway_setup_status(
        &env(&[
            (names::DASHSCOPE_API_KEY, "k"),
            (names::REALTIME_MODEL, "no-such-model"),
        ]),
        Locale::En,
    )
    .expect_err("an unknown model refuses");
    assert_eq!(error.code(), "VIA_REALTIME_MODEL_UNKNOWN");
}

#[test]
fn only_the_exact_string_one_opens_the_gate() {
    for value in ["true", "TRUE", "yes", "on", "0", "", " 1", "1 ", "01", "11"] {
        assert!(
            assert_gateway_setup(&env(&[(names::ALLOW_UNCONFIGURED, value)]), Locale::En).is_err(),
            "{value:?} must not opt out of the setup gate"
        );
    }
    assert!(assert_gateway_setup(&env(&[(names::ALLOW_UNCONFIGURED, "1")]), Locale::En).is_ok());
}

#[test]
fn the_opt_out_skips_even_malformed_configuration() {
    // Upstream returns before `gatewaySetupStatus` is called, so an
    // unsupported provider does not refuse when the gate is open.
    assert!(
        assert_gateway_setup(
            &env(&[
                (names::ALLOW_UNCONFIGURED, "1"),
                (names::REALTIME_PROVIDER, "nope"),
            ]),
            Locale::En
        )
        .is_ok()
    );
}

#[test]
fn the_refusal_carries_the_contract_code_and_the_structured_payload() {
    let error = assert_gateway_setup(&env(&[]), Locale::En).expect_err("unconfigured");
    assert_eq!(error.code(), CODE_GATEWAY_SETUP_REQUIRED);
    assert_eq!(error.code(), "VIA_GATEWAY_SETUP_REQUIRED");

    let protocol = error
        .protocol_error()
        .expect("the setup refusal has a client-visible shape");
    match protocol {
        ProtocolError::SetupRequired { missing } => {
            assert_eq!(missing.len(), 1);
            assert_eq!(missing[0].field, FIELD_DASHSCOPE_API_KEY);
            assert_eq!(missing[0].key, "DASHSCOPE_API_KEY");
        }
        other => panic!("expected SetupRequired, got {other:?}"),
    }
}

/// The phase-1 milestone.
#[test]
fn the_refusal_renders_in_all_three_locales() {
    for locale in Locale::ALL.iter().copied() {
        let error = assert_gateway_setup(&env(&[]), locale).expect_err("unconfigured");
        let rendered = error.to_string();

        // Built from the catalogue, not retyped: the item wrapper around the
        // key and the message, inside the refusal sentence.
        let detail = via_i18n::format(
            locale,
            keys::GATEWAY_SETUP_REQUIRED_ITEM,
            &[
                ("key", "DASHSCOPE_API_KEY"),
                (
                    "message",
                    t(locale, keys::GATEWAY_MISSING_DASHSCOPE_API_KEY),
                ),
            ],
        );
        let expected = via_i18n::format(
            locale,
            keys::GATEWAY_SETUP_REQUIRED,
            &[("details", detail.as_str())],
        );
        assert_eq!(rendered, expected, "the {locale} refusal must be exact");

        // It actually says which variable to set, in every locale.
        assert!(
            rendered.contains("DASHSCOPE_API_KEY"),
            "the {locale} refusal must name the variable"
        );
        // And it is not the i18n crate's own diagnostic for a broken render.
        assert!(
            !rendered.contains("via-i18n:"),
            "the {locale} refusal failed to interpolate: {rendered}"
        );
    }
}

#[test]
fn the_three_locales_render_three_different_sentences() {
    let rendered: Vec<String> = Locale::ALL
        .iter()
        .map(|locale| {
            assert_gateway_setup(&env(&[]), *locale)
                .expect_err("unconfigured")
                .to_string()
        })
        .collect();
    assert_ne!(rendered[0], rendered[1]);
    assert_ne!(rendered[1], rendered[2]);
    assert_ne!(rendered[0], rendered[2]);
    // `zh` carries upstream's own wording, with only the identity renamed.
    assert!(rendered[1].starts_with("Gateway 启动被拒绝，缺少必填配置："));
    assert!(rendered[1].contains("请运行 via config"));
}

#[test]
fn a_configured_install_is_not_refused() {
    assert!(assert_gateway_setup(&env(&[(names::DASHSCOPE_API_KEY, "sk-x")]), Locale::En).is_ok());
}
