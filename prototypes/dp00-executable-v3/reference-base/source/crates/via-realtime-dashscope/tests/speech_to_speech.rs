//! The huggingface/speech-to-speech provider, against
//! `server/test/realtime-provider.test.mjs:1779-1823` and the GA-dialect
//! sections that reach it.

use std::sync::Arc;
use std::time::Duration;

use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{Value, json};
use via_catalog::DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL;
use via_core::Secret;
use via_i18n::Locale;
use via_realtime::testing::test_transport;
use via_realtime::{
    AgentContext, ErrorClass, PermissionRequest, ProviderCapabilities, RealtimeError,
    RealtimeProvider, RealtimeSession, SessionOptions, SessionRequest, validate_realtime_provider,
};
use via_realtime_dashscope::{SpeechToSpeechProvider, SpeechToSpeechSettings};

fn provider() -> SpeechToSpeechProvider {
    SpeechToSpeechProvider::new(SpeechToSpeechSettings {
        configured: true,
        ..SpeechToSpeechSettings::default()
    })
}

fn context() -> AgentContext {
    AgentContext {
        instructions: "you are VIA".to_owned(),
        tools: vec![
            json!({
                "type": "function",
                "function": {
                    "name": "spawn_thinking",
                    "description": "delegate",
                    "parameters": { "type": "object", "required": ["objective"] },
                },
            }),
            json!({
                "type": "function",
                "function": { "name": "get_current_time", "description": "now", "parameters": {} },
            }),
        ],
        ..AgentContext::default()
    }
}

fn session(provider: &SpeechToSpeechProvider, context: &AgentContext) -> Value {
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
    // `realtime-provider.test.mjs:1779-1800`.
    let provider = provider();
    assert_eq!(validate_realtime_provider(&provider), Ok(()));
    assert_eq!(provider.key(), "speech-to-speech");
    assert_eq!(provider.label(), "Hugging Face Speech-to-Speech");
    assert_eq!(provider.aliases(), ["s2s"]);
    assert_eq!(provider.input_sample_rate(), 16_000);
    assert_eq!(provider.output_sample_rate(), 24_000);
    assert_eq!(
        provider.response_start_timeout(),
        Some(Duration::from_secs(60))
    );
}

#[test]
fn the_far_end_owns_the_model_the_voice_and_the_catalog() {
    let provider = provider();
    assert_eq!(provider.model(), None);
    assert_eq!(provider.voice(), None);
    assert_eq!(provider.model_profile(), None);
    assert!(provider.model_catalog().is_empty());
    // An empty catalog is published as `realtimeModelIds: null`, which a client
    // renders as *no* model picker rather than as an empty one.
    assert_eq!(
        via_realtime::describe_provider(&provider).realtime_model_ids,
        None
    );
}

#[test]
fn the_four_declared_capabilities_are_the_ones_upstream_declares() {
    assert_eq!(
        provider().capabilities(),
        ProviderCapabilities {
            acknowledges_session_update: false,
            single_response_slot: true,
            response_metadata_correlation: true,
            per_response_instructions: true,
            // Undeclared upstream, so it keeps the shared baseline.
            conversation_item_id_echo: true,
        }
    );
}

// ── the GA session payload ──────────────────────────────────────────────────

#[test]
fn the_session_negotiates_only_the_output_rate() {
    // `realtime-provider.test.mjs:1779-1800`.
    let session = session(&provider(), &context());
    assert_eq!(session["type"], json!("realtime"));
    assert_eq!(session["output_modalities"], json!(["audio"]));
    assert_eq!(
        session["audio"]["input"]["turn_detection"],
        json!({ "type": "server_vad", "interrupt_response": true })
    );
    assert_eq!(
        session["audio"]["output"]["format"],
        json!({ "type": "audio/pcm", "rate": 24_000 })
    );
    // No voice: the far end owns it.
    assert_eq!(session["audio"]["output"].get("voice"), None);
}

#[test]
fn the_absent_input_format_is_the_contract() {
    // The GA `AudioPCM` schema only accepts 24 kHz when the format is stated,
    // and speech-to-speech treats an omitted format as its native 16 kHz. So
    // declaring the capture rate here would invalidate the *whole*
    // `session.update`, not just one field. Upstream test-locks the absence.
    let session = session(&provider(), &context());
    assert_eq!(session["audio"]["input"].get("format"), None);
    assert_eq!(keys(&session["audio"]["input"]), ["turn_detection"]);
}

#[test]
fn the_session_key_order_is_upstreams_object_literal_order() {
    let session = session(&provider(), &context());
    assert_eq!(
        keys(&session),
        [
            "type",
            "instructions",
            "tools",
            "output_modalities",
            "audio"
        ]
    );
    assert_eq!(keys(&session["audio"]), ["input", "output"]);
}

#[test]
fn tools_are_flattened_out_of_the_beta_shape() {
    // The catalogued `tool schema shape per dialect`: two incompatible
    // encodings of one tool set, and the flattening lives in the provider.
    let session = session(&provider(), &context());
    assert_eq!(
        session["tools"],
        json!([
            {
                "type": "function",
                "name": "spawn_thinking",
                "description": "delegate",
                "parameters": { "type": "object", "required": ["objective"] },
            },
            {
                "type": "function",
                "name": "get_current_time",
                "description": "now",
                "parameters": {},
            },
        ])
    );
    // The nested `function` key is gone, not merely shadowed.
    assert_eq!(session["tools"][0].get("function"), None);
    assert_eq!(
        keys(&session["tools"][0]),
        ["type", "name", "description", "parameters"]
    );
}

#[test]
fn a_tool_that_is_not_in_the_beta_shape_survives_rather_than_becoming_empty() {
    // Dropping it would remove a capability with no diagnostic at all; passing
    // it through makes the service report the schema error.
    let context = AgentContext {
        tools: vec![json!({ "type": "function", "name": "already_flat" })],
        ..AgentContext::default()
    };
    let session = session(&provider(), &context);
    assert_eq!(
        session["tools"],
        json!([{ "type": "function", "name": "already_flat" }])
    );
}

#[test]
fn a_tool_missing_a_field_declares_it_as_null_rather_than_omitting_it() {
    // JavaScript's `{ name: tool.function.name, … }` writes `undefined`, which
    // `JSON.stringify` drops — but a *present* key with a null value is what
    // makes the service's own validation name the field, and a silently absent
    // `parameters` is the harder bug to find.
    let context = AgentContext {
        tools: vec![json!({ "type": "function", "function": { "name": "bare" } })],
        ..AgentContext::default()
    };
    let session = session(&provider(), &context);
    assert_eq!(
        session["tools"][0],
        json!({
            "type": "function",
            "name": "bare",
            "description": null,
            "parameters": null,
        })
    );
}

#[test]
fn every_update_carries_the_whole_payload() {
    // Unlike DashScope, this provider does not branch on `configured`: it
    // applies `session.update` silently, so re-sending the whole thing is how a
    // context refresh is acknowledged at all.
    let context = context();
    let first = session(&provider(), &context);
    let later = provider().build_session(&SessionRequest {
        configured: true,
        agent_context: &context,
    });
    assert_eq!(first, later);
}

// ── the response bodies ─────────────────────────────────────────────────────

#[test]
fn out_of_band_speech_pins_tool_choice_as_well_as_the_conversation() {
    // `realtime-provider.test.mjs:1815-1822`. Four keys, one more than
    // DashScope's three.
    let response = provider().build_speak_response("任务完成");
    assert_eq!(
        keys(&response),
        ["conversation", "modalities", "instructions", "tool_choice"]
    );
    assert_eq!(response["conversation"], json!("none"));
    assert_eq!(response["modalities"], json!(["audio"]));
    assert_eq!(response["tool_choice"], json!("none"));
    let instructions = response["instructions"].as_str().unwrap_or_default();
    assert!(instructions.contains("任务完成"), "{instructions}");
}

#[test]
fn the_beta_modalities_key_is_what_the_dialect_rewrites() {
    // Every response body here writes `modalities`; the GA dialect moves it to
    // `output_modalities` in `response_create`. Writing `output_modalities`
    // directly would survive one rewrite and be dropped by the next.
    let provider = provider();
    let frame = provider
        .protocol()
        .response_create(Some(provider.build_speak_response("x")));
    assert_eq!(frame["response"]["output_modalities"], json!(["audio"]));
    assert_eq!(frame["response"].get("modalities"), None);
}

#[test]
fn both_injections_are_audio_only_and_tool_free() {
    let provider = provider();
    let result = provider.build_result_injection("构建完成");
    let permission = provider.build_permission_injection(&PermissionRequest {
        id: "perm_2".to_owned(),
        summary: "run tests".to_owned(),
    });

    for response in [&result.response, &permission.response] {
        assert_eq!(
            keys(response),
            ["modalities", "tool_choice", "instructions"]
        );
        assert_eq!(response["modalities"], json!(["audio"]));
        assert_eq!(response["tool_choice"], json!("none"));
    }
    assert_eq!(
        result.item,
        json!({
            "type": "message",
            "role": "user",
            "content": [{ "type": "input_text", "text": "构建完成" }],
        })
    );
    assert_eq!(
        permission.item["content"][0]["text"],
        json!(
            "<backend_permission_request>\n\
             authorization_id=perm_2\n\
             operation=run tests\n\
             </backend_permission_request>"
        )
    );
}

#[test]
fn the_permission_item_is_byte_identical_to_the_dashscope_one() {
    // The catalogued contract says "identical at s2s.mjs:125-128", and
    // `config/frontend-agent/PROMPT.md` refers to the tag by name — so the two
    // providers must not drift apart.
    let permission = PermissionRequest {
        id: "perm_3".to_owned(),
        summary: "删除目录".to_owned(),
    };
    let dashscope = via_realtime_dashscope::DashScopeProvider::default()
        .build_permission_injection(&permission);
    let s2s = provider().build_permission_injection(&permission);
    assert_eq!(dashscope.item, s2s.item);
}

// ── endpoint and credential ─────────────────────────────────────────────────

#[test]
fn the_endpoint_is_dialled_verbatim() {
    // No query parameter: there is no model to name.
    assert_eq!(
        provider().url(),
        Ok(DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL.to_owned())
    );
}

#[test]
fn the_authorization_header_appears_only_when_a_token_is_configured() {
    assert_eq!(provider().headers(), Vec::new());
    let with_token = SpeechToSpeechProvider::new(SpeechToSpeechSettings {
        auth_token: Secret::new("hf-token"),
        configured: true,
        ..SpeechToSpeechSettings::default()
    });
    assert_eq!(
        with_token.headers(),
        [("Authorization".to_owned(), "Bearer hf-token".to_owned())]
    );
    assert!(!format!("{with_token:?}").contains("hf-token"));
}

#[test]
fn a_default_endpoint_alone_never_advertises_the_provider() {
    // Upstream: "do not advertise a local service merely because a default
    // endpoint exists."
    let provider = SpeechToSpeechProvider::default();
    assert_eq!(
        provider.url(),
        Ok(DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL.to_owned())
    );
    assert!(!provider.is_configured());
}

#[test]
fn the_signature_omits_the_model_and_voice_keys_entirely() {
    // `EndpointOnly`: `model` and `voice` are absent from the hashed object,
    // not null — a struct with `Option` fields would hash a different string.
    let provider = provider();
    let expected = via_catalog::RealtimeIdentity::EndpointOnly {
        provider: "speech-to-speech".to_owned(),
        endpoint: DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL.to_owned(),
        credential: String::new(),
    };
    assert_eq!(provider.configuration_signature(), expected.signature());
    assert_eq!(
        expected.canonical_json(),
        format!(
            r#"{{"provider":"speech-to-speech","endpoint":"{DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL}","credential":""}}"#
        )
    );
}

#[test]
fn the_signature_is_the_one_via_core_resolved_for_the_same_environment() {
    let env: via_core::EnvMap = [
        ("VIA_REALTIME_PROVIDER", "s2s"),
        (
            "SPEECH_TO_SPEECH_REALTIME_URL",
            "ws://127.0.0.1:9000/v1/realtime",
        ),
        ("SPEECH_TO_SPEECH_AUTH_TOKEN", "hf-token"),
    ]
    .into_iter()
    .collect();
    let frontend = via_core::config::resolve_realtime_frontend(&env).expect("resolves");
    let provider =
        SpeechToSpeechProvider::new(SpeechToSpeechSettings::from_frontend(&frontend, Locale::En));

    assert!(provider.is_configured());
    assert_eq!(provider.configuration_signature(), frontend.signature);
}

// ── refusals ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn an_unconfigured_endpoint_is_refused_before_a_socket_is_opened() {
    let provider: Arc<dyn RealtimeProvider> = Arc::new(SpeechToSpeechProvider::default());
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
            provider: "speech-to-speech".to_owned(),
            message: "请先配置 SPEECH_TO_SPEECH_REALTIME_URL".to_owned(),
        })
    );
    assert!(peer.drain_frames().is_empty());
}

#[test]
fn there_is_no_preflight_refusal_at_all() {
    // Nothing about this provider is knowable in advance: it has no model to
    // recognise. Inventing a refusal here would block a working endpoint.
    assert_eq!(SpeechToSpeechProvider::default().preflight(), Ok(()));
}

#[rstest]
#[case(Locale::En)]
#[case(Locale::Zh)]
#[case(Locale::Ko)]
fn the_connect_timeout_sentence_names_the_endpoint_it_tried(#[case] locale: Locale) {
    let provider = SpeechToSpeechProvider::new(SpeechToSpeechSettings {
        url: "ws://127.0.0.1:9999/v1/realtime".to_owned(),
        locale,
        ..SpeechToSpeechSettings::default()
    });
    let message = provider.connect_timeout_message(locale);
    assert!(
        message.contains("ws://127.0.0.1:9999/v1/realtime"),
        "{message}"
    );
    assert!(!message.contains('{'), "{message}");
}

#[test]
fn the_zh_connect_timeout_sentence_is_upstreams_own() {
    let provider = provider();
    assert_eq!(
        provider.connect_timeout_message(Locale::Zh),
        format!(
            "连接 Hugging Face speech-to-speech 服务超时（{DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL}），\
             请确认 speech-to-speech 服务已启动"
        )
    );
}

// ── error classification through the trait ──────────────────────────────────

#[rstest]
#[case(
    "All 1 session slots are in use. Disconnect an existing client first.",
    ErrorClass::CapacityBusy
)]
#[case("session_limit_reached", ErrorClass::CapacityBusy)]
#[case("Another response is in progress", ErrorClass::ResponseSlotBusy)]
#[case("No active response", ErrorClass::NoActiveResponse)]
#[case("Invalid API-key provided.", ErrorClass::Other)]
fn the_provider_classifies_through_its_own_corpus(
    #[case] message: &str,
    #[case] expected: ErrorClass,
) {
    assert_eq!(provider().classify_error(message), expected, "{message}");
}

#[test]
fn a_busy_response_slot_is_retryable_only_because_the_flag_says_so() {
    // The pairing is the point: `response_slot_busy` is retried *because* the
    // provider declares one response slot, and the session reads the flag
    // rather than the provider key.
    let provider = provider();
    assert!(provider.capabilities().single_response_slot);
    assert_eq!(
        provider.classify_error("Another response is in progress"),
        ErrorClass::ResponseSlotBusy
    );
    assert!(!ErrorClass::ResponseSlotBusy.is_suppressed());
}
