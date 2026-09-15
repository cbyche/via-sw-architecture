//! Every catalogued value this crate reproduces, asserted against
//! `docs/reference/contracts.json`.
//!
//! Nothing here retypes a contract value. Each test pulls the record out of the
//! catalogue and compares it to what the crate actually produces, so a catalogue
//! edit and a code edit have to agree.
//!
//! The mock is a *test double*, which makes this file more load-bearing than it
//! looks: a double that speaks a slightly different dialect than the real
//! provider makes every test above it agree with itself and with nothing else.

mod common;

use std::time::Duration;

use common::contract_of_kind;
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use via_realtime::{
    BUSY_RETRY_DELAYS, CONNECT_TIMEOUT, DEFAULT_RESPONSE_INACTIVITY_TIMEOUT,
    DEFAULT_RESPONSE_START_TIMEOUT, ErrorClass, FAILED_RESPONSE_STATUSES, MAX_BUSY_RETRIES,
    POST_CANCEL_RECOVERY, PermissionRequest, RealtimeProvider,
};
use via_realtime_mock::{
    CannedAudio, MockProvider, MockProviderSpec, PERMISSION_REQUEST_TAG, events, messages,
    script::COMPLETED,
};

fn provider() -> MockProvider {
    MockProvider::new(MockProviderSpec::default())
}

// ── audio ───────────────────────────────────────────────────────────────────

#[test]
fn the_two_sample_rates_are_the_catalogued_pair() {
    let record = contract_of_kind("default-value", "audio sample rates");
    let input = u32::try_from(record.number_after("inputSampleRate")).expect("a rate");
    let output = u32::try_from(record.number_after("outputSampleRate")).expect("a rate");
    assert_eq!(input, 16_000);
    assert_eq!(output, 24_000);

    let provider = provider();
    assert_eq!(provider.input_sample_rate(), input);
    assert_eq!(provider.output_sample_rate(), output);
    assert_eq!(CannedAudio::DEFAULT_RATE.hz(), output);
}

#[test]
fn canned_speech_is_chunked_at_the_catalogued_block() {
    let record = contract_of_kind("default-value", "audio sample rates");
    record.assert_mentions("20 ms");
    // `via-audio` owns the block; the mock only uses it.
    assert_eq!(via_audio::BLOCK_MILLIS, 20);
    let clip = CannedAudio::tone(CannedAudio::DEFAULT_RATE, Duration::from_millis(100));
    let chunks = clip.chunks();
    assert_eq!(chunks.len(), 5);
    assert!(
        chunks
            .iter()
            .all(|chunk| chunk.duration(clip.rate()) == Duration::from_millis(20))
    );
}

// ── responses ───────────────────────────────────────────────────────────────

#[test]
fn the_three_failure_statuses_are_the_catalogued_ones_and_the_mock_can_script_each() {
    let record = contract_of_kind("default-value", "response failure statuses");
    let statuses = record.single_quoted();
    assert_eq!(statuses, ["failed", "cancelled", "incomplete"]);
    assert_eq!(FAILED_RESPONSE_STATUSES.to_vec(), statuses);
    // The mock's own name for the one that is *not* a failure.
    assert!(!statuses.contains(&COMPLETED.to_owned()));
    assert_eq!(
        via_realtime_mock::CANCELLED,
        "cancelled",
        "the cancel default reports a catalogued failure status"
    );

    for status in &statuses {
        let done = events::response_done(status);
        assert_eq!(done["response"]["status"], json!(status));
    }
}

#[test]
fn the_busy_retry_schedule_is_the_catalogued_ladder() {
    let record = contract_of_kind("default-value", "busy-response retry schedule");
    record.assert_mentions("[1200, 2600, 5000]");
    record.assert_mentions("capped at 3 retries");
    assert_eq!(
        BUSY_RETRY_DELAYS.map(|delay| delay.as_millis()),
        [1200, 2600, 5000]
    );
    assert_eq!(MAX_BUSY_RETRIES, 3);
    // The two phrases a mock has to send to reach either half of the ladder.
    record.assert_mentions("response_slot_busy");
    record.assert_mentions("input_busy");
    assert_eq!(
        provider().classify_error(messages::RESPONSE_SLOT_BUSY),
        ErrorClass::ResponseSlotBusy
    );
    assert_eq!(
        provider().classify_error(messages::INPUT_BUSY),
        ErrorClass::InputBusy
    );
}

#[test]
fn the_timeouts_a_script_shortens_are_the_catalogued_ones() {
    let record = contract_of_kind("default-value", "RealtimeFrontend timeouts");
    assert_eq!(
        CONNECT_TIMEOUT.as_millis() as i64,
        record.number_after("WebSocket connect timeout")
    );
    assert_eq!(
        DEFAULT_RESPONSE_START_TIMEOUT.as_millis() as i64,
        record.number_after("provider.responseStartTimeoutMs ?? ")
    );
    assert_eq!(
        DEFAULT_RESPONSE_INACTIVITY_TIMEOUT.as_millis() as i64,
        record.number_after("responseCompletionTimeoutMs ?? ")
    );
    assert_eq!(
        POST_CANCEL_RECOVERY.as_millis() as i64,
        record.number_after("post-cancel recovery timer")
    );

    // A provider may name its own response-start budget; `s2s` names 60 s, and
    // a mock can too.
    let slow = MockProvider::new(
        MockProviderSpec::default().with_response_start_timeout(Some(Duration::from_secs(60))),
    );
    assert_eq!(
        slow.response_start_timeout(),
        Some(Duration::from_millis(
            u64::try_from(record.number_after("s2s = ")).expect("a timeout")
        ))
    );
    assert_eq!(provider().response_start_timeout(), None);
}

// ── error classification ────────────────────────────────────────────────────

#[test]
fn the_classification_vocabulary_is_closed_and_every_member_is_reachable() {
    let record = contract_of_kind("error-code", "provider error classification vocabulary");
    let vocabulary = record.single_quoted();
    assert_eq!(
        vocabulary,
        [
            "inactivity",
            "input_busy",
            "no_active_response",
            "fatal",
            "capacity_busy",
            "response_slot_busy",
            "other",
        ]
    );

    let provider = provider();
    let reached: Vec<String> = [
        messages::INACTIVITY,
        messages::INPUT_BUSY,
        messages::NO_ACTIVE_RESPONSE,
        messages::FATAL,
        messages::CAPACITY_BUSY,
        messages::RESPONSE_SLOT_BUSY,
        "nothing anybody has a pattern for",
    ]
    .iter()
    .map(|message| provider.classify_error(message).as_str().to_owned())
    .collect();
    assert_eq!(
        reached, vocabulary,
        "every class is one scripted message away"
    );
}

// ── the payloads the provider builds ────────────────────────────────────────

#[test]
fn the_speak_response_is_the_catalogued_out_of_band_shape() {
    let record = contract_of_kind("json-field", "buildSpeakResponse payloads");
    record.assert_mentions("conversation: 'none'");
    record.assert_why_mentions("keeps it out of the conversation history");

    let response = provider().build_speak_response("nearly finished");
    assert_eq!(response["conversation"], json!("none"));
    assert!(response.get("modalities").is_some());
    assert_eq!(response["instructions"], json!("nearly finished"));
}

#[test]
fn the_result_injection_is_the_catalogued_item_and_response_pair() {
    let record = contract_of_kind("json-field", "buildResultInjection payload");
    record.assert_mentions("type: 'message'");
    record.assert_mentions("role: 'user'");
    record.assert_mentions("type: 'input_text'");
    record.assert_mentions("tool_choice: 'none'");

    let injection = provider().build_result_injection("the build finished");
    assert_eq!(injection.item["type"], json!("message"));
    assert_eq!(injection.item["role"], json!("user"));
    assert_eq!(
        injection.item["content"][0],
        json!({ "type": "input_text", "text": "the build finished" })
    );
    assert_eq!(injection.response["tool_choice"], json!("none"));
}

#[test]
fn the_permission_item_is_the_catalogued_prompt_vocabulary() {
    let record = contract_of_kind("prompt-text", "backend permission injection item text");
    let template = record
        .unescaped()
        .replace("${permission.id}", "auth-1")
        .replace("${permission.summary}", "write to /tmp");

    let injection = provider().build_permission_injection(&PermissionRequest {
        id: "auth-1".into(),
        summary: "write to /tmp".into(),
    });
    assert_eq!(injection.item["content"][0]["text"], json!(template));
    // The tag name itself is referenced by `config/frontend-agent/PROMPT.md`, so
    // it is published as a constant rather than buried in a `format!`.
    assert!(
        template.contains(&format!("<{PERMISSION_REQUEST_TAG}>")),
        "{template}"
    );
}

// ── tool calls ──────────────────────────────────────────────────────────────

#[test]
fn a_scripted_tool_call_carries_the_catalogued_fields() {
    let record = contract_of_kind(
        "json-field",
        "function call handling over the realtime protocol",
    );
    record.assert_mentions("Trigger event: response.function_call_arguments.done");
    record.assert_mentions("arguments (JSON string");
    record.assert_mentions("output: JSON.stringify(result)");

    let event = events::function_call("call_1", "spawn_thinking", &json!({ "objective": "x" }));
    assert_eq!(
        event["type"],
        json!("response.function_call_arguments.done")
    );
    assert_eq!(event["call_id"], json!("call_1"));
    assert_eq!(event["name"], json!("spawn_thinking"));
    assert!(
        event["arguments"].is_string(),
        "arguments is a JSON string, never a nested object"
    );
    assert_eq!(
        serde_json::from_str::<Value>(event["arguments"].as_str().expect("a string"))
            .expect("valid json"),
        json!({ "objective": "x" })
    );
}

// ── identity ────────────────────────────────────────────────────────────────

#[test]
fn nothing_this_crate_publishes_carries_a_brand_it_should_not() {
    // The one upstream-branded string reachable from here is
    // `RESPONSE_CORRELATION_KEY`, and it is `via-realtime`'s, KEEP under
    // `docs/rebrand.md` because a third-party endpoint echoes it back. This
    // crate must add none of its own.
    let provider = provider();
    let surfaces = [
        json!(provider.key()),
        json!(provider.label()),
        json!(provider.model()),
        json!(provider.voice()),
        json!(provider.url().unwrap_or_default()),
        provider.build_speak_response("x"),
        provider.build_result_injection("x").item,
        provider.build_result_injection("x").response,
        json!(messages::RESPONSE_SLOT_BUSY),
        json!(messages::INPUT_BUSY),
        json!(messages::NO_ACTIVE_RESPONSE),
        json!(messages::CAPACITY_BUSY),
        json!(messages::INACTIVITY),
        json!(messages::FATAL),
    ];
    for surface in surfaces {
        let text = surface.to_string().to_lowercase();
        for brand in ["qwen", "qwaudio", "argo", "tini"] {
            assert!(!text.contains(brand), "`{brand}` leaked into {surface}");
        }
    }
}

#[test]
fn the_mock_row_is_the_catalogs_and_not_this_crates() {
    // The upstream provider table has two entries and VIA extends it; `mock` is
    // one of VIA's, and it lives in `via-catalog` so `/api/health` and the
    // config parser agree with this crate about the key.
    let record = contract_of_kind("provider-descriptor", "built-in realtime providers");
    record.assert_mentions("key 'dashscope'");
    record.assert_mentions("key 'speech-to-speech'");
    assert!(
        !record.exact_value.contains("'mock'"),
        "the catalogue describes upstream's two; `mock` is VIA's own extension"
    );

    let row = via_catalog::realtime_provider_definition(via_realtime_mock::MOCK_PROVIDER_KEY)
        .expect("the catalog row");
    assert_eq!(MockProviderSpec::default().key, row.key);
    assert_eq!(MockProviderSpec::default().label, row.label);
}
