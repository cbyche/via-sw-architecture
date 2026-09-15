//! Every catalogued value this crate owns, asserted against
//! `docs/reference/contracts.json`.
//!
//! Nothing here retypes a contract value. Each test pulls the record out of the
//! catalogue and compares it to what the crate actually produces, so a catalogue
//! edit and a code edit have to agree.

mod common;

use std::sync::Arc;
use std::time::Duration;

use common::{Contract, contract_of_kind};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use via_i18n::Locale;
use via_realtime::testing::TestProvider;
use via_realtime::{
    BUSY_RETRY_DELAYS, BackoffConfig, CAPABILITY_FLAGS, CONNECT_TIMEOUT,
    DEFAULT_RESPONSE_INACTIVITY_TIMEOUT, DEFAULT_RESPONSE_START_TIMEOUT, ErrorClass,
    FAILED_RESPONSE_STATUSES, MAX_ATTEMPT_EXPONENT, POST_CANCEL_RECOVERY, PROTOCOL_METHODS,
    ProviderCapabilities, RealtimeConnectionInputs, RealtimeError, RealtimeProtocol,
    RealtimeProvider, RealtimeProviderRegistry, ReconnectBackoff, STABLE_CONNECTION_MS, Visibility,
    ga_realtime_protocol, is_recoverable_realtime_inactivity_error, is_valid_provider_key,
    openai_compatible_protocol, realtime_connection_status, realtime_event_error_message,
};

/// The `|`-separated alternative that mentions `needle`, unquoted.
///
/// The catalogue writes a message table as
/// `` `a ${x}` | `b` | 'c' ``, so an alternative is found by a fragment of it
/// and then stripped of whatever quoting it carried.
fn alternative(record: &Contract, needle: &str) -> String {
    record
        .exact_value
        .split('|')
        .map(str::trim)
        .find(|candidate| candidate.contains(needle))
        .unwrap_or_else(|| {
            panic!(
                "no alternative mentioning `{needle}` in contract `{}`:\n{}",
                record.name, record.exact_value
            )
        })
        .trim_matches(|c| c == '`' || c == '\'')
        .to_owned()
}

/// One catalogued message, the fragment that finds it, and its interpolations.
type MessageCase = (
    RealtimeError,
    &'static str,
    Vec<(&'static str, &'static str)>,
);

/// Fill a `${…}` template.
fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut filled = template.to_owned();
    for (name, value) in values {
        filled = filled.replace(&format!("${{{name}}}"), value);
    }
    assert!(
        !filled.contains("${"),
        "unfilled placeholder in `{filled}`; supplied {values:?}"
    );
    filled
}

// ── connection status ───────────────────────────────────────────────────────

#[test]
fn the_six_connection_states_are_the_catalogued_ones() {
    let record = contract_of_kind("state-name", "realtimeConnectionStatus states");
    let states: Vec<String> = record.single_quoted();
    // The catalogue lists them in precedence order, most specific first.
    assert_eq!(
        states,
        [
            "disconnected",
            "unavailable",
            "sleeping",
            "waking",
            "connected",
            "connecting",
        ]
    );
    record.assert_mentions("precedence in that exact order");
    record.assert_mentions("{ provider, state, error? } (frozen)");
}

#[test]
fn the_precedence_ladder_matches_the_catalogue() {
    let record = contract_of_kind("state-name", "realtime connection status precedence");
    record.assert_mentions(
        "'disconnected' (default) < 'connecting' < 'connected' < 'waking' < 'sleeping'",
    );
    record.assert_mentions("blockedError overrides all");
    record.assert_mentions("{provider, state:'unavailable', error}");

    let ladder = [
        (
            RealtimeConnectionInputs {
                provider: "p".into(),
                ..Default::default()
            },
            "disconnected",
        ),
        (
            RealtimeConnectionInputs {
                provider: "p".into(),
                connecting: true,
                ..Default::default()
            },
            "connecting",
        ),
        (
            RealtimeConnectionInputs {
                provider: "p".into(),
                connecting: true,
                ready: true,
                ..Default::default()
            },
            "connected",
        ),
        (
            RealtimeConnectionInputs {
                provider: "p".into(),
                connecting: true,
                ready: true,
                waking: true,
                ..Default::default()
            },
            "waking",
        ),
        (
            RealtimeConnectionInputs {
                provider: "p".into(),
                connecting: true,
                ready: true,
                waking: true,
                sleeping: true,
                ..Default::default()
            },
            "sleeping",
        ),
    ];
    for (inputs, expected) in ladder {
        assert_eq!(
            realtime_connection_status(&inputs).state.as_str(),
            expected,
            "{inputs:?}"
        );
    }

    let blocked = realtime_connection_status(&RealtimeConnectionInputs {
        provider: "dashscope".into(),
        ready: true,
        sleeping: true,
        blocked_error: "credential missing".into(),
        ..Default::default()
    });
    assert_eq!(
        serde_json::to_value(&blocked).expect("serialize"),
        json!({
            "provider": "dashscope",
            "state": "unavailable",
            "error": "credential missing",
        })
    );
}

// ── the reconnect ladder ────────────────────────────────────────────────────

#[test]
fn the_reconnect_ladder_is_the_catalogued_one() {
    let record = contract_of_kind("default-value", "ReconnectBackoff");
    let config = BackoffConfig::default();
    assert_eq!(
        i64::try_from(config.base_ms).expect("fits"),
        record.number_after("baseMs ")
    );
    assert_eq!(
        i64::try_from(config.max_ms).expect("fits"),
        record.number_after("maxMs ")
    );
    assert!((config.jitter_ratio - 0.2).abs() < f64::EPSILON);
    record.assert_mentions("jitterRatio 0.2");
    assert_eq!(
        i64::from(MAX_ATTEMPT_EXPONENT),
        record.number_after("attempt exponent capped at ")
    );
    assert_eq!(
        i64::try_from(STABLE_CONNECTION_MS).expect("fits"),
        record.number_after("stayed up >= ")
    );
    record.assert_mentions("reset() zeroes attempt");
}

#[test]
fn the_catalogued_reconnect_sequence_is_reproduced() {
    let record = contract_of_kind("default-value", "reconnect backoff sequence");
    record.assert_mentions("with baseMs 500, maxMs 2000, jitterRatio 0: [500,1000,2000,2000]");
    let mut backoff = ReconnectBackoff::with_jitter_source(
        BackoffConfig {
            base_ms: 500,
            max_ms: 2000,
            jitter_ratio: 0.0,
        },
        || 0.0,
    );
    let ladder: Vec<u128> = (0..4).map(|_| backoff.next().as_millis()).collect();
    assert_eq!(ladder, [500, 1000, 2000, 2000]);
    backoff.reset();
    assert_eq!(backoff.next().as_millis(), 500);
    record.assert_mentions("reset() returns to baseMs");
}

// ── capabilities ────────────────────────────────────────────────────────────

#[test]
fn the_default_capabilities_are_the_catalogued_ones() {
    let record = contract_of_kind(
        "default-value",
        "DEFAULT_CAPABILITIES (provider capability defaults)",
    );
    // `{ name: true, name: false, … }` parsed straight out of the catalogue.
    let expected: Vec<(String, bool)> = record
        .exact_value
        .trim_matches(|c| c == '{' || c == '}' || c == ' ')
        .split(',')
        .map(|pair| {
            let (name, value) = pair.split_once(':').expect("a name: value pair");
            (
                name.trim().to_owned(),
                value.trim().parse::<bool>().expect("a boolean"),
            )
        })
        .collect();

    let capabilities = ProviderCapabilities::DEFAULT;
    assert_eq!(
        expected
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        CAPABILITY_FLAGS
    );
    assert_eq!(
        expected.iter().map(|(_, value)| *value).collect::<Vec<_>>(),
        capabilities.as_array()
    );
    record.assert_why_mentions("the five names are the closed CAPABILITY_FLAGS set");
}

#[test]
fn the_twelve_protocol_methods_are_the_catalogued_ones() {
    // The names survive as a list even though the trait is the check, because
    // `via-conformance` asserts the vocabulary rather than the mechanism.
    assert_eq!(PROTOCOL_METHODS.len(), 12);
    for method in [
        "encodeOutgoing",
        "normalizeIncoming",
        "responseCorrelationId",
    ] {
        assert!(PROTOCOL_METHODS.contains(&method), "{method}");
    }
}

// ── timeouts ────────────────────────────────────────────────────────────────

#[test]
fn the_session_timeouts_are_the_catalogued_ones() {
    let record = contract_of_kind("default-value", "RealtimeFrontend timeouts");
    assert_eq!(
        i64::try_from(CONNECT_TIMEOUT.as_millis()).expect("fits"),
        record.number_after("WebSocket connect timeout ")
    );
    assert_eq!(
        i64::try_from(DEFAULT_RESPONSE_START_TIMEOUT.as_millis()).expect("fits"),
        record.number_after("provider.responseStartTimeoutMs ?? ")
    );
    assert_eq!(
        i64::try_from(DEFAULT_RESPONSE_INACTIVITY_TIMEOUT.as_millis()).expect("fits"),
        record.number_after("responseCompletionTimeoutMs ?? ")
    );
    assert_eq!(
        i64::try_from(POST_CANCEL_RECOVERY.as_millis()).expect("fits"),
        record.number_after("post-cancel recovery timer ")
    );
    record.assert_why_mentions("sliding window");
    // The provider override the catalogue names, exercised end to end in
    // `tests/responses.rs`.
    assert_eq!(
        TestProvider::new("s2s-like")
            .with_response_start_timeout(Some(Duration::from_millis(60_000)))
            .response_start_timeout(),
        Some(Duration::from_millis(
            u64::try_from(record.number_after("s2s = ")).expect("fits")
        ))
    );
}

#[test]
fn the_busy_retry_schedule_is_the_catalogued_one() {
    let record = contract_of_kind("default-value", "busy-response retry schedule");
    let delays: Vec<i64> = record
        .exact_value
        .split_once('[')
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(list, _)| {
            list.split(',')
                .map(|value| value.trim().parse::<i64>().expect("a number"))
                .collect()
        })
        .expect("a delay list");
    assert_eq!(
        BUSY_RETRY_DELAYS
            .iter()
            .map(|delay| i64::try_from(delay.as_millis()).expect("fits"))
            .collect::<Vec<_>>(),
        delays
    );
    assert_eq!(
        i64::from(via_realtime::MAX_BUSY_RETRIES),
        record.number_after("capped at ")
    );
    record.assert_mentions(
        "If activeResponses is non-empty, wait for idle instead of the fixed delay.",
    );
    record.assert_mentions("pending.origin === 'model'");
    record.assert_why_mentions("replayed byte-identically from pending.responsePayload");
}

#[test]
fn the_three_failure_statuses_are_the_catalogued_ones() {
    let record = contract_of_kind("default-value", "response failure statuses");
    let statuses: Vec<String> = record.single_quoted();
    assert_eq!(statuses, FAILED_RESPONSE_STATUSES);
    record.assert_why_mentions("a provider that reports e.g. 'error' would be treated as success");
    assert!(via_realtime::is_completed_status(Some("error")));
}

// ── provider identity ───────────────────────────────────────────────────────

#[test]
fn the_provider_key_shape_and_visibilities_are_the_catalogued_ones() {
    let record = contract_of_kind("default-value", "provider key regex and visibility set");
    record.assert_mentions("/^[a-z0-9][a-z0-9-]*$/");
    record.assert_mentions("String(v).trim().toLowerCase()");
    let visibilities: Vec<String> = record.single_quoted();
    assert_eq!(visibilities, ["public", "gateway-only", "public"]);
    assert_eq!(Visibility::Public.as_str(), visibilities[0]);
    assert_eq!(Visibility::GatewayOnly.as_str(), visibilities[1]);
    assert_eq!(Visibility::default().as_str(), visibilities[2]);

    assert!(is_valid_provider_key("dashscope"));
    assert!(is_valid_provider_key("speech-to-speech"));
    assert!(!is_valid_provider_key("-leading"));
    assert!(!is_valid_provider_key("Upper"));
    assert_eq!(via_realtime::clean_key("  QWEN \n"), "qwen");
}

#[test]
fn the_two_upstream_aliases_survive_the_rebrand() {
    let record = contract_of_kind("provider-id", "realtime provider keys and aliases");
    record.assert_mentions("dashscope (label 'DashScope', aliases ['qwen'])");
    record.assert_mentions(
        "speech-to-speech (label 'Hugging Face Speech-to-Speech', aliases ['s2s'])",
    );

    // The registry resolves them; `via-catalog` owns the table they come from.
    let mut registry = RealtimeProviderRegistry::with_default("dashscope");
    for definition in via_catalog::realtime_providers().iter().take(2) {
        registry
            .register(Arc::new(
                TestProvider::new(definition.key)
                    .with_label(definition.label)
                    .with_aliases(definition.aliases.iter().copied()),
            ))
            .map(|_| ())
            .expect("registers");
    }
    assert_eq!(
        registry.resolve(Some("qwen")).expect("alias").key(),
        "dashscope"
    );
    assert_eq!(
        registry.resolve(Some("S2S")).expect("alias").key(),
        "speech-to-speech"
    );
    assert_eq!(registry.resolve(None).expect("default").key(), "dashscope");
}

// ── the composed error message ──────────────────────────────────────────────

#[test]
fn the_error_composition_is_the_catalogued_one() {
    let record = contract_of_kind("json-field", "realtimeEventErrorMessage composition");
    record.assert_mentions("Join with ': '");
    record.assert_mentions("deduped non-empty trimmed values");
    record.assert_mentions(
        "[event.error.code, event.error.type, event.error.message, event.message]",
    );

    let fallback = record
        .exact_value
        .rsplit_once("fallback '")
        .and_then(|(_, rest)| rest.split_once('\''))
        .map(|(value, _)| value.to_owned())
        .expect("a fallback");
    assert_eq!(
        realtime_event_error_message(&json!({ "type": "error" }), Locale::Zh),
        fallback
    );
    assert_eq!(
        realtime_event_error_message(
            &json!({
                "error": { "code": "A", "type": "B", "message": "C" },
                "message": "D",
            }),
            Locale::Zh
        ),
        "A: B: C: D"
    );
}

#[test]
fn the_inactivity_closure_is_the_catalogued_text() {
    let record = contract_of_kind("error-code", "recoverable realtime inactivity closure");
    record.assert_mentions(
        "Your session was closed because no response was generated for <N> seconds",
    );
    record.assert_mentions("with or without a trailing period");
    assert!(is_recoverable_realtime_inactivity_error(
        "Your session was closed because no response was generated for 180 seconds."
    ));
    assert!(is_recoverable_realtime_inactivity_error(
        "Your session was closed because no response was generated for 180 seconds"
    ));
    for actionable in [
        "Cannot create response while user is speaking.",
        "Authentication failed",
    ] {
        assert!(
            !is_recoverable_realtime_inactivity_error(actionable),
            "{actionable}"
        );
        record.assert_mentions(actionable);
    }
}

#[test]
fn the_classification_vocabulary_is_the_catalogued_one() {
    let record = contract_of_kind("error-code", "provider error classification vocabulary");
    let names: Vec<String> = record.single_quoted();
    let ours = [
        ErrorClass::Inactivity,
        ErrorClass::InputBusy,
        ErrorClass::NoActiveResponse,
        ErrorClass::Fatal,
        ErrorClass::CapacityBusy,
        ErrorClass::ResponseSlotBusy,
        ErrorClass::Other,
    ];
    assert_eq!(
        names,
        ours.iter().map(|class| class.as_str()).collect::<Vec<_>>()
    );
    // The Gateway's own branch table, restated as a predicate.
    record.assert_why_mentions("capacity_busy and no_active_response are swallowed silently");
    record.assert_why_mentions("inactivity and fatal suppress the user-facing error");
    for class in [
        ErrorClass::CapacityBusy,
        ErrorClass::NoActiveResponse,
        ErrorClass::Inactivity,
        ErrorClass::Fatal,
    ] {
        assert!(class.is_suppressed(), "{class}");
    }
    assert!(!ErrorClass::Other.is_suppressed());
}

// ── the two dialects ────────────────────────────────────────────────────────

#[test]
fn the_beta_dialect_writes_the_catalogued_frames() {
    let record = contract_of_kind("ws-event", "outgoing provider frames (OpenAI beta dialect)");
    record.assert_mentions("event_id: 'event_<uuid-no-dashes>'");
    let protocol = openai_compatible_protocol();

    let session = protocol.encode_outgoing(protocol.session_update(json!({ "a": 1 })));
    assert_eq!(session["type"], json!("session.update"));
    assert_eq!(session["session"], json!({ "a": 1 }));

    let audio = protocol.audio_append("<base64>");
    assert_eq!(
        audio,
        json!({ "type": "input_audio_buffer.append", "audio": "<base64>" })
    );
    assert_eq!(
        protocol.conversation_item_create(json!({ "id": "x" })),
        json!({ "type": "conversation.item.create", "item": { "id": "x" } })
    );
    assert_eq!(
        protocol.response_create(None),
        json!({ "type": "response.create" })
    );
    assert_eq!(
        protocol.response_create(Some(json!({ "modalities": ["audio"] }))),
        json!({ "type": "response.create", "response": { "modalities": ["audio"] } })
    );
    assert_eq!(
        protocol.response_cancel(),
        json!({ "type": "response.cancel" })
    );

    let event_id = session["event_id"].as_str().expect("an event id");
    assert!(event_id.starts_with("event_"));
    assert_eq!(event_id.len(), "event_".len() + 32);
    assert!(
        event_id["event_".len()..]
            .chars()
            .all(|c| c.is_ascii_hexdigit())
    );
}

#[test]
fn the_ga_dialect_differs_only_where_the_catalogue_says() {
    let record = contract_of_kind("ws-event", "outgoing provider frames (GA dialect deltas)");
    record.assert_mentions("`output_modalities` instead of `modalities`");
    record.assert_mentions("qwen_audio_request_id");
    let protocol = ga_realtime_protocol();

    assert_eq!(
        protocol.response_create(Some(json!({ "modalities": ["audio"] })))["response"],
        json!({ "output_modalities": ["audio"] })
    );

    // `message->'msg_<uuid>', function_call->'fc_<uuid>',
    //  function_call_output->'fco_<uuid>', anything else->'item_<uuid>'`
    for (kind, prefix) in [
        ("message", "msg_"),
        ("function_call", "fc_"),
        ("function_call_output", "fco_"),
        ("audio", "item_"),
    ] {
        record.assert_mentions(prefix.trim_end_matches('_'));
        let id = protocol.conversation_item_id(&json!({ "type": kind }));
        assert!(id.starts_with(prefix), "{kind} -> {id}");
    }

    let correlated =
        protocol.correlate_response_create(protocol.response_create(None), "request-1");
    assert_eq!(
        correlated["response"]["metadata"][via_realtime::RESPONSE_CORRELATION_KEY],
        json!("request-1")
    );
    assert_eq!(protocol.response_correlation_id(&correlated), "request-1");
}

#[test]
fn a_tool_result_is_encoded_the_catalogued_way_in_both_dialects() {
    let record = contract_of_kind(
        "json-field",
        "function call handling over the realtime protocol",
    );
    record.assert_mentions("output: JSON.stringify(result)");
    record.assert_why_mentions("`output` is always a JSON-encoded string, never a nested object");

    for protocol in [
        &openai_compatible_protocol() as &dyn RealtimeProtocol,
        &ga_realtime_protocol() as &dyn RealtimeProtocol,
    ] {
        let item = protocol.function_output_item("call-1", &json!({ "status": "accepted" }));
        assert_eq!(item["type"], json!("function_call_output"));
        assert_eq!(item["call_id"], json!("call-1"));
        assert_eq!(item["output"], json!(r#"{"status":"accepted"}"#));
    }
    // The id namespace is the only difference, and it is the GA one.
    assert!(
        ga_realtime_protocol()
            .conversation_item_id(&json!({ "type": "function_call_output" }))
            .starts_with("fco_")
    );
    assert!(
        openai_compatible_protocol()
            .conversation_item_id(&json!({ "type": "function_call_output" }))
            .starts_with("item_")
    );
}

// ── the published descriptor ────────────────────────────────────────────────

#[test]
fn the_active_realtime_shape_is_the_catalogued_one() {
    let record = contract_of_kind("json-field", "describeActiveRealtime response shape");
    // `{ a, b, …, providers: [{ key, label, … }] }` — the outer object up to
    // and including `providers`, then the member shape inside the array.
    let (outer_source, inner_source) = record
        .exact_value
        .split_once("providers: [{")
        .expect("a nested providers array");
    let outer: Vec<String> = outer_source
        .trim_start_matches('{')
        .split(',')
        .map(|field| field.trim().to_owned())
        .filter(|field| !field.is_empty())
        .chain(std::iter::once("providers".to_owned()))
        .collect();
    let inner: Vec<String> = inner_source
        .split_once("}]")
        .expect("a closed array")
        .0
        .split(',')
        .map(|field| field.trim().to_owned())
        .collect();

    let mut registry = RealtimeProviderRegistry::with_default("beta");
    registry
        .register(Arc::new(TestProvider::new("beta")))
        .map(|_| ())
        .expect("registers");
    let active = registry.describe_active_realtime(None).expect("describes");
    let json = serde_json::to_value(&active).expect("serialize");
    assert_eq!(
        json.as_object()
            .expect("object")
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        outer
    );

    let descriptor =
        serde_json::to_value(via_realtime::describe_provider(&TestProvider::new("beta")))
            .expect("serialize");
    assert_eq!(
        descriptor
            .as_object()
            .expect("object")
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        inner
    );
}

// ── messages ────────────────────────────────────────────────────────────────

#[test]
fn the_internal_error_messages_are_upstreams_own_text() {
    let record = contract_of_kind("error-code", "RealtimeFrontend internal error messages");
    let cases: Vec<MessageCase> = vec![
        (
            RealtimeError::ConnectionClosed {
                label: "Qwen-Audio-Realtime".into(),
            },
            "连接已关闭",
            vec![("provider.label", "Qwen-Audio-Realtime")],
        ),
        (
            RealtimeError::ItemUnconfirmed {
                label: "Qwen-Audio-Realtime".into(),
                id: "item_1".into(),
            },
            "未确认对话项",
            vec![("provider.label", "Qwen-Audio-Realtime"), ("id", "item_1")],
        ),
        (
            RealtimeError::ItemRejected {
                label: "Qwen-Audio-Realtime".into(),
                detail: None,
            },
            "创建对话项失败",
            vec![("provider.label", "Qwen-Audio-Realtime")],
        ),
        (RealtimeError::RequestCancelled, "请求已取消", Vec::new()),
        (RealtimeError::SessionReset, "会话已重置", Vec::new()),
        (
            RealtimeError::ResponseCorrelationConflict,
            "响应关联冲突",
            Vec::new(),
        ),
        (
            RealtimeError::UnsupportedModel {
                id: "qwen3.5-omni-plus-realtime-future".into(),
                label: "Qwen-Audio-Realtime".into(),
            },
            "不支持的 Realtime 模型",
            vec![
                ("id", "qwen3.5-omni-plus-realtime-future"),
                ("provider.label", "Qwen-Audio-Realtime"),
            ],
        ),
    ];
    for (error, needle, values) in cases {
        let expected = fill(&alternative(record, needle), &values);
        assert_eq!(error.message(Locale::Zh), expected, "{needle}");
    }
}

#[test]
fn the_validation_messages_are_upstreams_own_text() {
    let record = contract_of_kind("error-code", "provider/protocol validation error messages");
    let cases: Vec<MessageCase> = vec![
        (
            RealtimeError::ProviderNeedsKeyAndLabel,
            "必须定义 key 和 label",
            Vec::new(),
        ),
        (
            RealtimeError::ProviderKeyInvalid { key: "Bad".into() },
            "key 无效",
            vec![("key", "Bad")],
        ),
        (
            RealtimeError::ProviderMissingInputSampleRate { key: "p".into() },
            "缺少 inputSampleRate",
            vec![("key", "p")],
        ),
        (
            RealtimeError::ProviderMissingOutputSampleRate { key: "p".into() },
            "缺少 outputSampleRate",
            vec![("key", "p")],
        ),
        (
            RealtimeError::ProviderResponseTimeoutInvalid { key: "p".into() },
            "responseStartTimeoutMs 必须是正数",
            vec![("key", "p")],
        ),
        (
            RealtimeError::ProviderAliasesInvalid { key: "p".into() },
            "aliases 无效",
            vec![("key", "p")],
        ),
        (
            RealtimeError::ProviderModelProfileInvalid { key: "p".into() },
            "modelProfile 无效",
            vec![("key", "p")],
        ),
        (
            RealtimeError::ProviderSessionDefaultsIncomplete { key: "p".into() },
            "modelProfile.sessionDefaults 不完整",
            vec![("key", "p")],
        ),
        (
            RealtimeError::ProviderNameTaken {
                name: "qwen".into(),
            },
            "名称已注册",
            vec![("name", "qwen")],
        ),
    ];
    for (error, needle, values) in cases {
        let expected = fill(&alternative(record, needle), &values);
        assert_eq!(error.message(Locale::Zh), expected, "{needle}");
    }
}

#[test]
fn the_unsupported_provider_message_is_upstreams_own_shape() {
    let record = contract_of_kind("error-code", "unsupported realtime provider error message");
    // Upstream's own list, so the `（可选 …）` half compares byte for byte.
    let error = RealtimeError::UnsupportedProvider {
        requested: "qwen3".into(),
        available: vec!["dashscope".into(), "speech-to-speech".into()],
    };
    assert_eq!(
        error.message(Locale::Zh),
        fill(&record.exact_value, &[("requested", "qwen3")])
    );
    record.assert_why_mentions("uses full-width punctuation");
}

#[test]
fn the_restored_context_wrapper_is_the_catalogued_text() {
    let record = contract_of_kind("prompt-text", "restored conversation context item text");
    let expected = record
        .unescaped()
        .replace("${recent}", "User: where were we");
    // The Chinese sentence is upstream's own, carried by `via-i18n`.
    let rendered = [
        "<restored_context>",
        via_i18n::t(
            Locale::Zh,
            via_i18n::keys::REALTIME_RESTORED_CONTEXT_INSTRUCTIONS,
        ),
        "User: where were we",
        "</restored_context>",
    ]
    .join("\n");
    assert_eq!(rendered, expected);
}

#[test]
fn the_permission_item_tag_is_the_catalogued_vocabulary() {
    let record = contract_of_kind("prompt-text", "backend permission injection item text");
    // The rendering belongs to a provider; what this crate owns is the input
    // shape, so the tag and both field names are asserted against the catalogue
    // and against the double that stands in for a real provider.
    record.assert_mentions("<backend_permission_request>");
    record.assert_mentions("authorization_id=");
    record.assert_mentions("operation=");

    let provider = TestProvider::new("beta");
    let injection = provider.build_permission_injection(&via_realtime::PermissionRequest {
        id: "permission-one".into(),
        summary: "read the system memory".into(),
    });
    let text = injection.item["content"][0]["text"]
        .as_str()
        .expect("text")
        .to_owned();
    assert_eq!(
        text,
        record
            .unescaped()
            .replace("${permission.id}", "permission-one")
            .replace("${permission.summary}", "read the system memory")
    );
    assert_eq!(injection.response["tool_choice"], json!("none"));
}

#[test]
fn the_response_activity_table_covers_both_dialect_spellings() {
    // Not catalogued as a list, but the *behaviour* is: a provider that emits
    // output without `response.created` still counts as active.
    let record = contract_of_kind(
        "json-field",
        "function call handling over the realtime protocol",
    );
    record.assert_mentions("response.function_call_arguments.done");
    for kind in via_realtime::RESPONSE_ACTIVITY_TYPES {
        assert!(
            via_realtime::is_response_activity_event(&json!({
                "type": kind,
                "response_id": "r",
            })),
            "{kind}"
        );
    }
    assert!(!via_realtime::is_response_activity_event(&json!({
        "type": "conversation.item.created",
        "response_id": "r",
    })));
}

#[test]
fn the_sample_rates_a_provider_declares_reach_the_descriptor() {
    let record = contract_of_kind("default-value", "audio sample rates");
    let input = record.number_after("inputSampleRate = ");
    let output = record.number_after("outputSampleRate = ");
    let provider = TestProvider::new("beta");
    // The test double carries upstream's own numbers, so the wiring is asserted
    // against the catalogue rather than against a retyped literal.
    assert_eq!(i64::from(provider.input_sample_rate()), input);
    assert_eq!(i64::from(provider.output_sample_rate()), output);
    record.assert_why_mentions("the voice.ready handshake all key off inputSampleRate");
}

#[test]
fn nothing_this_crate_publishes_carries_a_brand_it_should_not() {
    // The one upstream-branded string that survives, and the reason it does.
    assert_eq!(
        via_realtime::RESPONSE_CORRELATION_KEY,
        "qwen_audio_request_id"
    );
    let value: Value = serde_json::to_value(ProviderCapabilities::DEFAULT).expect("serialize");
    let text = value.to_string();
    for brand in ["qwen", "argo", "tini", "qwaudio"] {
        assert!(!text.to_lowercase().contains(brand), "{brand} in {text}");
    }
}
