//! The DashScope provider, against the provider half of
//! `server/test/realtime-provider.test.mjs`.
//!
//! Upstream mutates a module singleton (`config.audioModel = …`) and restores it
//! in `t.after`; here each case builds its own settings, so the whole file is
//! order-independent and runs in parallel.

use std::sync::Arc;

use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{Value, json};
use via_catalog::{DEFAULT_DASHSCOPE_REALTIME_MODEL, DEFAULT_DASHSCOPE_REALTIME_URL};
use via_core::Secret;
use via_i18n::Locale;
use via_realtime::testing::test_transport;
use via_realtime::{
    AgentContext, PermissionRequest, ProviderCapabilities, RealtimeError, RealtimeProvider,
    RealtimeSession, SessionOptions, SessionRequest, validate_realtime_provider,
};
use via_realtime_dashscope::{DashScopeProvider, DashScopeSettings};

const OMNI_FLASH: &str = "qwen3.5-omni-flash-realtime";
const OMNI_PLUS: &str = "qwen3.5-omni-plus-realtime";
const AUDIO_PLUS: &str = DEFAULT_DASHSCOPE_REALTIME_MODEL;
const AUDIO_FLASH: &str = "qwen-audio-3.0-realtime-flash";

fn provider(model: &str) -> DashScopeProvider {
    DashScopeProvider::new(DashScopeSettings {
        model: model.to_owned(),
        api_key: Secret::new("sk-test"),
        ..DashScopeSettings::default()
    })
}

fn context() -> AgentContext {
    AgentContext {
        instructions: "you are VIA".to_owned(),
        tools: vec![json!({
            "type": "function",
            "function": { "name": "spawn_thinking", "description": "d", "parameters": {} },
        })],
        ..AgentContext::default()
    }
}

fn first_session(provider: &DashScopeProvider, context: &AgentContext) -> Value {
    provider.build_session(&SessionRequest {
        configured: false,
        agent_context: context,
    })
}

fn keys(value: &Value) -> Vec<String> {
    value
        .as_object()
        .map(|object| object.keys().cloned().collect())
        .unwrap_or_default()
}

// ── the descriptor ──────────────────────────────────────────────────────────

#[test]
fn the_provider_validates_and_carries_its_catalogued_descriptor() {
    let provider = provider(AUDIO_PLUS);
    assert_eq!(validate_realtime_provider(&provider), Ok(()));
    assert_eq!(provider.key(), "dashscope");
    assert_eq!(provider.label(), "Qwen-Audio-Realtime");
    assert_eq!(provider.aliases(), ["qwen"]);
    assert_eq!(provider.input_sample_rate(), 16_000);
    assert_eq!(provider.output_sample_rate(), 24_000);
    // Upstream declares no `responseStartTimeoutMs`, so the session default of
    // 30 s applies. Saying `Some(30s)` here would be a different contract.
    assert_eq!(provider.response_start_timeout(), None);
}

#[test]
fn the_model_catalog_is_the_whole_dashscope_table_in_order() {
    let provider = provider(AUDIO_PLUS);
    let ids: Vec<&str> = provider
        .model_catalog()
        .iter()
        .map(|profile| profile.id.as_ref())
        .collect();
    assert_eq!(ids, [OMNI_FLASH, OMNI_PLUS, AUDIO_PLUS, AUDIO_FLASH]);
    assert_eq!(
        provider.model_catalog(),
        via_catalog::dashscope_realtime_model_profiles(),
        "the catalog is the table's, not a copy of it"
    );
}

// ── the session payload, per family ─────────────────────────────────────────

#[test]
fn the_default_model_configures_smart_turn_only() {
    // `realtime-provider.test.mjs:353-380`.
    let session = first_session(&provider(AUDIO_PLUS), &context());
    assert_eq!(session["turn_detection"], json!({ "type": "smart_turn" }));
    // Exactly one key: no threshold, no silence_duration_ms, ever.
    assert_eq!(keys(&session["turn_detection"]), ["type"]);
}

#[rstest]
#[case(OMNI_FLASH, "Ethan", "semantic_vad", false)]
#[case(OMNI_PLUS, "Ethan", "semantic_vad", false)]
#[case(AUDIO_PLUS, "longanqian", "smart_turn", true)]
#[case(AUDIO_FLASH, "longanqian", "smart_turn", true)]
fn each_family_negotiates_its_own_voice_and_turn_detection(
    #[case] model: &str,
    #[case] voice: &str,
    #[case] turn_detection: &str,
    #[case] item_id_echo: bool,
) {
    // `realtime-provider.test.mjs:382-435`.
    let provider = provider(model);
    let session = first_session(&provider, &context());

    assert_eq!(session["voice"], json!(voice));
    assert_eq!(provider.voice(), Some(voice));
    assert_eq!(session["turn_detection"], json!({ "type": turn_detection }));
    assert_eq!(session["modalities"], json!(["text", "audio"]));
    assert_eq!(
        provider.build_speak_response("完成")["modalities"],
        json!(["text", "audio"])
    );
    // The one capability that is model-dependent: the Omni models replace a
    // client-assigned conversation item id, so it must not be waited on.
    assert_eq!(
        provider.capabilities().conversation_item_id_echo,
        item_id_echo
    );
}

#[test]
fn the_remaining_capability_flags_are_the_shared_baseline() {
    // `realtime-provider.test.mjs:1613-1623`.
    assert_eq!(
        provider(AUDIO_PLUS).capabilities(),
        ProviderCapabilities {
            acknowledges_session_update: true,
            single_response_slot: false,
            response_metadata_correlation: false,
            per_response_instructions: true,
            conversation_item_id_echo: true,
        }
    );
}

#[test]
fn a_family_voice_override_beats_the_profile_default() {
    // `realtime-provider.test.mjs:437-451`.
    let provider = DashScopeProvider::new(DashScopeSettings {
        model: OMNI_PLUS.to_owned(),
        voice: "custom-omni".to_owned(),
        ..DashScopeSettings::default()
    });
    assert_eq!(
        first_session(&provider, &context())["voice"],
        json!("custom-omni")
    );
    assert_eq!(provider.voice(), Some("custom-omni"));
}

#[test]
fn the_first_session_update_key_order_is_upstreams_assignment_order() {
    let session = first_session(&provider(AUDIO_PLUS), &context());
    assert_eq!(
        keys(&session),
        [
            "instructions",
            "tools",
            "modalities",
            "voice",
            "output_audio_format",
            "input_audio_format",
            "turn_detection",
        ]
    );
    assert_eq!(session["instructions"], json!("you are VIA"));
    assert_eq!(session["output_audio_format"], json!("pcm"));
    assert_eq!(session["input_audio_format"], json!("pcm"));
}

#[test]
fn every_later_update_carries_only_instructions_and_tools() {
    // The catalogued `(subsequent, configured)` payload. Re-sending
    // `turn_detection` on a context refresh would reset the provider's VAD
    // mid-conversation, and re-sending the formats would renegotiate audio.
    let context = context();
    let session = provider(AUDIO_PLUS).build_session(&SessionRequest {
        configured: true,
        agent_context: &context,
    });
    assert_eq!(keys(&session), ["instructions", "tools"]);
}

#[test]
fn the_tool_catalog_is_carried_through_in_the_beta_shape() {
    // The beta dialect nests a tool under `function`; only the GA provider
    // flattens it. A DashScope session must pass it through untouched.
    let context = context();
    let session = first_session(&provider(AUDIO_PLUS), &context);
    assert_eq!(session["tools"], json!(context.tools));
    assert_eq!(
        session["tools"][0]["function"]["name"],
        json!("spawn_thinking")
    );
}

#[test]
fn a_session_with_no_tools_still_declares_the_empty_catalog() {
    // Upstream gates `tools` on the model's `functionCalling` flag, not on the
    // catalog being non-empty, so a `dictation` context clears the tools rather
    // than leaving whatever the previous update declared.
    let context = AgentContext {
        instructions: "transcribe".to_owned(),
        ..AgentContext::default()
    };
    let session = first_session(&provider(AUDIO_PLUS), &context);
    assert_eq!(session["tools"], json!([]));
}

// ── the response bodies ─────────────────────────────────────────────────────

#[test]
fn out_of_band_speech_stays_out_of_conversation_history() {
    let response = provider(AUDIO_PLUS).build_speak_response("三点有个会");
    assert_eq!(
        keys(&response),
        ["conversation", "modalities", "instructions"]
    );
    assert_eq!(response["conversation"], json!("none"));
    // The content rides *inside* the instructions, because `conversation: none`
    // means there is no item to read it from.
    let instructions = response["instructions"].as_str().unwrap_or_default();
    assert!(instructions.contains("三点有个会"), "{instructions}");
}

#[test]
fn a_result_injection_is_a_user_item_plus_a_tool_free_response() {
    let injection = provider(AUDIO_PLUS).build_result_injection("构建完成");
    assert_eq!(
        injection.item,
        json!({
            "type": "message",
            "role": "user",
            "content": [{ "type": "input_text", "text": "构建完成" }],
        })
    );
    assert_eq!(
        keys(&injection.response),
        ["modalities", "tool_choice", "instructions"]
    );
    assert_eq!(injection.response["tool_choice"], json!("none"));
    assert_eq!(injection.response["modalities"], json!(["text", "audio"]));
}

#[test]
fn a_permission_injection_carries_the_prompt_tag_verbatim() {
    let injection = provider(AUDIO_PLUS).build_permission_injection(&PermissionRequest {
        id: "perm_7".to_owned(),
        summary: "删除构建目录".to_owned(),
    });
    assert_eq!(
        injection.item["content"][0]["text"],
        json!(
            "<backend_permission_request>\n\
             authorization_id=perm_7\n\
             operation=删除构建目录\n\
             </backend_permission_request>"
        )
    );
    assert_eq!(injection.response["tool_choice"], json!("none"));
}

#[rstest]
#[case(Locale::En)]
#[case(Locale::Zh)]
#[case(Locale::Ko)]
fn every_composed_instruction_renders_in_every_locale(#[case] locale: Locale) {
    let provider = DashScopeProvider::new(DashScopeSettings {
        locale,
        ..DashScopeSettings::default()
    });
    let permission = PermissionRequest {
        id: "p".to_owned(),
        summary: "s".to_owned(),
    };
    for body in [
        provider.build_speak_response("x"),
        provider.build_result_injection("x").response,
        provider.build_permission_injection(&permission).response,
    ] {
        let instructions = body["instructions"].as_str().unwrap_or_default();
        assert!(!instructions.is_empty(), "{locale}");
        assert!(
            !instructions.contains("<via-i18n:"),
            "{locale}: {instructions}"
        );
        assert!(!instructions.contains('{'), "{locale}: {instructions}");
    }
}

// ── endpoint, credential, signature ─────────────────────────────────────────

#[test]
fn the_endpoint_carries_the_model_as_a_query_parameter() {
    assert_eq!(
        provider(AUDIO_PLUS).url(),
        Ok(format!(
            "{DEFAULT_DASHSCOPE_REALTIME_URL}?model={AUDIO_PLUS}"
        ))
    );
}

#[rstest]
// A base URL that already has a query gets `&`, not a second `?`.
#[case(
    "wss://gateway.example/realtime?tenant=a",
    "wss://gateway.example/realtime?tenant=a&model=m"
)]
#[case(
    "wss://gateway.example/realtime",
    "wss://gateway.example/realtime?model=m"
)]
fn the_query_separator_follows_the_base_url(#[case] base: &str, #[case] expected: &str) {
    let provider = DashScopeProvider::new(DashScopeSettings {
        base_url: base.to_owned(),
        model: "m".to_owned(),
        ..DashScopeSettings::default()
    });
    assert_eq!(provider.url(), Ok(expected.to_owned()));
}

#[test]
fn a_model_id_is_percent_encoded_into_the_endpoint() {
    let provider = DashScopeProvider::new(DashScopeSettings {
        base_url: "wss://h/r".to_owned(),
        model: "a b/c".to_owned(),
        ..DashScopeSettings::default()
    });
    assert_eq!(provider.url(), Ok("wss://h/r?model=a%20b%2Fc".to_owned()));
}

#[test]
fn the_authorization_header_is_sent_even_without_a_key() {
    // Upstream sends it unconditionally. A missing credential then fails as a
    // 401 the `fatal` classifier recognises, rather than as an unauthenticated
    // upgrade the service answers in some other way.
    let unconfigured = DashScopeProvider::default();
    assert!(!unconfigured.is_configured());
    assert_eq!(
        unconfigured.headers(),
        [("Authorization".to_owned(), "Bearer ".to_owned())]
    );
    assert_eq!(
        provider(AUDIO_PLUS).headers(),
        [("Authorization".to_owned(), "Bearer sk-test".to_owned())]
    );
}

#[test]
fn a_provider_never_prints_its_credential() {
    let rendered = format!("{:?}", provider(AUDIO_PLUS));
    assert!(!rendered.contains("sk-test"), "{rendered}");
}

#[test]
fn the_signature_is_the_one_via_core_resolved_for_the_same_environment() {
    // The provider computes its own identity rather than reading a cached one,
    // so this is the assertion that the two agree — and therefore that a client
    // comparing `/api/health.realtimeConfigurationSignature` against its own
    // configuration still matches.
    let env: via_core::EnvMap = [
        ("DASHSCOPE_API_KEY", "sk-live"),
        ("VIA_REALTIME_BASE_URL", "wss://gateway.example/realtime"),
        ("VIA_REALTIME_MODEL", OMNI_PLUS),
        ("VIA_OMNI_REALTIME_VOICE", "Ethan-custom"),
    ]
    .into_iter()
    .collect();
    let frontend = via_core::config::resolve_realtime_frontend(&env).expect("resolves");
    let provider = DashScopeProvider::new(DashScopeSettings::from_frontend(&frontend, Locale::En));

    assert_eq!(provider.configuration_signature(), frontend.signature);
    assert!(!frontend.signature.is_empty());
}

#[test]
fn changing_only_the_credential_changes_the_signature() {
    let base = DashScopeSettings::default();
    let first = DashScopeProvider::new(DashScopeSettings {
        api_key: Secret::new("one"),
        ..base.clone()
    });
    let second = DashScopeProvider::new(DashScopeSettings {
        api_key: Secret::new("two"),
        ..base
    });
    assert_ne!(
        first.configuration_signature(),
        second.configuration_signature()
    );
}

#[test]
fn the_signature_hashes_the_override_not_the_resolved_voice() {
    // With no override the hashed voice is the empty string, *not* the profile
    // default: the profile default is the catalog's, and a signature that moved
    // when the catalog moved would report drift the operator did not cause.
    let without = DashScopeProvider::new(DashScopeSettings::default());
    let with_default_spelled_out = DashScopeProvider::new(DashScopeSettings {
        voice: "longanqian".to_owned(),
        ..DashScopeSettings::default()
    });
    assert_eq!(without.voice(), Some("longanqian"));
    assert_eq!(with_default_spelled_out.voice(), Some("longanqian"));
    assert_ne!(
        without.configuration_signature(),
        with_default_spelled_out.configuration_signature(),
        "the hash input is the override, so spelling out the default changes it"
    );
}

// ── ⚠ the bug that is not ported ────────────────────────────────────────────

#[test]
fn an_unknown_model_is_refused_and_the_refusal_names_it() {
    let provider = provider("qwen3.5-omni-flash-realtime-future");
    assert_eq!(
        provider.preflight(),
        Err(RealtimeError::UnsupportedModel {
            id: "qwen3.5-omni-flash-realtime-future".to_owned(),
            label: "Qwen-Audio-Realtime".to_owned(),
        })
    );
    // Upstream's own message, in upstream's own locale.
    assert_eq!(
        provider
            .preflight()
            .expect_err("refused")
            .message(Locale::Zh),
        "不支持的 Realtime 模型：qwen3.5-omni-flash-realtime-future（Qwen-Audio-Realtime）"
    );
}

#[test]
fn capabilities_are_never_inferred_from_the_shape_of_an_id() {
    // `realtime-provider.test.mjs:473-491`, "fails closed … without inferring
    // Omni behavior". The id *looks* like an Omni model; it is still refused.
    let provider = provider("qwen3.5-omni-plus-realtime-future");
    assert!(provider.preflight().is_err());
    assert_eq!(provider.model_profile(), None);
    assert_eq!(
        provider.build_speak_response("完成")["modalities"],
        json!([])
    );
}

#[test]
fn the_refusal_names_the_resolved_id_not_the_padded_one() {
    // Upstream interpolates `modelProfile.id`, which is the *trimmed* value.
    assert_eq!(
        provider("  not-a-model \n").preflight(),
        Err(RealtimeError::UnsupportedModel {
            id: "not-a-model".to_owned(),
            label: "Qwen-Audio-Realtime".to_owned(),
        })
    );
}

#[test]
fn an_empty_model_id_resolves_to_the_catalog_default_rather_than_failing() {
    let provider = DashScopeProvider::new(DashScopeSettings {
        model: "   ".to_owned(),
        ..DashScopeSettings::default()
    });
    assert_eq!(provider.preflight(), Ok(()));
    assert_eq!(
        provider
            .model_profile()
            .map(|profile| profile.id.to_string()),
        Some(AUDIO_PLUS.to_owned())
    );
}

#[tokio::test]
async fn an_unknown_model_is_refused_before_a_socket_is_opened() {
    // `realtime-provider.test.mjs:493-506` asserts `frontend.ws === null`
    // afterwards; the equivalent here is that the transport saw no frame at all.
    let provider: Arc<dyn RealtimeProvider> = Arc::new(provider("qwen-audio-9.9-realtime"));
    let (transport, mut peer) = test_transport();

    let outcome = RealtimeSession::open(provider, SessionOptions::default(), transport).await;

    assert_eq!(
        outcome.err(),
        Some(RealtimeError::UnsupportedModel {
            id: "qwen-audio-9.9-realtime".to_owned(),
            label: "Qwen-Audio-Realtime".to_owned(),
        })
    );
    assert!(
        peer.drain_frames().is_empty(),
        "nothing may be written before the model is known"
    );
}

#[tokio::test]
async fn an_unknown_model_outranks_a_missing_credential() {
    // The ordering is contract: `preflight()` runs before `is_configured()`, so
    // an operator with two faults is told about the one that would have
    // produced a session that hears nothing.
    let provider: Arc<dyn RealtimeProvider> = Arc::new(DashScopeProvider::new(DashScopeSettings {
        model: "nope".to_owned(),
        ..DashScopeSettings::default()
    }));
    let (transport, mut peer) = test_transport();

    let outcome = RealtimeSession::open(provider, SessionOptions::default(), transport).await;

    assert!(
        matches!(outcome, Err(RealtimeError::UnsupportedModel { .. })),
        "{outcome:?}"
    );
    assert!(peer.drain_frames().is_empty());
}

#[tokio::test]
async fn a_missing_credential_is_refused_with_the_providers_own_sentence() {
    let provider: Arc<dyn RealtimeProvider> = Arc::new(DashScopeProvider::default());
    let (transport, mut peer) = test_transport();

    let outcome = RealtimeSession::open(
        provider,
        SessionOptions {
            locale: Locale::Zh,
            ..SessionOptions::default()
        },
        transport,
    )
    .await;

    assert_eq!(
        outcome.err(),
        Some(RealtimeError::NotConfigured {
            provider: "dashscope".to_owned(),
            message: "请先配置 DASHSCOPE_API_KEY".to_owned(),
        })
    );
    assert!(peer.drain_frames().is_empty());
}

// ── error classification through the trait ──────────────────────────────────

#[test]
fn the_provider_classifies_through_its_own_corpus() {
    let provider = provider(AUDIO_PLUS);
    assert_eq!(
        provider.classify_error("InvalidApiKey: Invalid API-key provided."),
        via_realtime::ErrorClass::Fatal
    );
    assert_eq!(
        provider.classify_error("All 1 session slots are in use."),
        via_realtime::ErrorClass::Other,
        "the speech-to-speech corpus must not leak into this one"
    );
}
