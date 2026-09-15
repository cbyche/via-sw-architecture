//! Every catalogued value this crate depends on, **parsed** out of
//! `docs/reference/contracts.json`.
//!
//! A retyped literal proves the code matches the test. A parsed one proves the
//! code matches the specification, and it fails loudly when a contract is
//! renamed or removed.
//!
//! # This crate is mostly *new*, and the tests say which parts are
//!
//! `docs/architecture.md` §9 marks `via-realtime-local` with an asterisk: no
//! upstream counterpart. So there is no catalogued row for the pipeline's own
//! machinery, and inventing one would be worse than having none.
//!
//! What *is* catalogued is everything the pipeline has to agree with in order to
//! be indistinguishable from a cloud provider — the sample rates, the response
//! failure statuses, the response-id lookup order, the tool-call round trip, the
//! error classification vocabulary, and the response-start budget. Those are the
//! rows below, and each is asserted against the shipped constant rather than
//! against a copy of it.

mod common;

use common::contract_of_kind;
use pretty_assertions::assert_eq;
use via_audio::SampleRate;
use via_catalog::TurnDetectionKind;
use via_realtime::{ErrorClass, RESPONSE_ACTIVITY_TYPES, realtime_response_id};
use via_realtime_local::{
    EMITTED_EVENT_TYPES, LocalPipelineProvider, LocalSettings, PIPELINE_INPUT_RATE,
    PIPELINE_OUTPUT_RATE, RESPONSE_START_TIMEOUT, events,
};

#[test]
fn the_two_pipeline_rates_are_the_catalogued_realtime_rates() {
    // `default-value` / *audio sample rates*: 16 000 in, 24 000 out. The local
    // pipeline is not exempt — a client is told what to capture at on
    // `voice.ready`, and a pipeline that asked for something else would be the
    // one provider a host had to special-case.
    let contract = contract_of_kind("default-value", "audio sample rates");
    assert_eq!(
        u64::from(PIPELINE_INPUT_RATE.hz()),
        contract.number_after("inputSampleRate = ")
    );
    assert_eq!(
        u64::from(PIPELINE_OUTPUT_RATE.hz()),
        contract.number_after("outputSampleRate = ")
    );
    // And they are `via-audio`'s constants rather than integers of our own, so
    // the resampler and the stages cannot drift apart.
    assert_eq!(PIPELINE_INPUT_RATE, SampleRate::HZ_16000);
    assert_eq!(PIPELINE_OUTPUT_RATE, SampleRate::HZ_24000);
}

#[test]
fn the_block_and_chunk_sizes_are_the_catalogued_ones() {
    // Same row: "Capture chunk = capture_rate/50 (20 ms, min 160 frames);
    // playback block = playback_rate/50 (min 240 frames)". `via-audio` owns the
    // arithmetic; this asserts the pipeline uses it rather than a round number.
    contract_of_kind("default-value", "audio sample rates").assert_mentions("capture_rate/50");
    assert_eq!(PIPELINE_INPUT_RATE.capture_block_frames(), 320);
    assert_eq!(PIPELINE_OUTPUT_RATE.playback_block_frames(), 480);
    assert_eq!(
        via_realtime_local::scripted::SPEECH_CHUNK_SAMPLES,
        PIPELINE_OUTPUT_RATE.playback_block_frames()
    );
}

#[test]
fn the_three_response_statuses_are_the_catalogued_failure_set_plus_completed() {
    // `default-value` / *response failure statuses*. The catalogue's own
    // warning is why this matters: a provider that reports a status upstream
    // does not know is treated as **success**, so a barge-in that answered
    // anything but `cancelled` would look like a completed turn.
    let contract = contract_of_kind("default-value", "response failure statuses");
    for status in [events::RESPONSE_FAILED, events::RESPONSE_CANCELLED] {
        contract.assert_mentions(status);
    }
    assert!(
        !contract.exact_value.contains(events::RESPONSE_COMPLETED),
        "`completed` must not be a failure status"
    );
    assert!(via_realtime::is_completed_status(Some(
        events::RESPONSE_COMPLETED
    )));
}

#[test]
fn every_response_frame_carries_an_id_the_catalogued_lookup_order_finds() {
    // `json-field` / *realtimeResponseId resolution order*:
    // `event.response_id || event.response.id || event.item.response_id || ''`.
    // A frame the Gateway cannot attribute proves nothing, and the pipeline
    // emits eight kinds of response frame.
    contract_of_kind("json-field", "realtimeResponseId resolution order")
        .assert_mentions("event.response_id");

    let frames = [
        events::response_created("resp_local_1", None),
        events::response_done("resp_local_1", events::RESPONSE_COMPLETED, None, Vec::new()),
        events::output_item_added("resp_local_1", 0, events::assistant_message_item("msg_1")),
        events::audio_transcript_delta("resp_local_1", "msg_1", "hi"),
        events::audio_transcript_done("resp_local_1", "msg_1", "hi"),
        events::audio_delta("resp_local_1", "msg_1", &[1, 2]),
        events::audio_done("resp_local_1", "msg_1"),
        events::function_call_arguments_done("resp_local_1", "fc_1", "call_1", "f", "{}"),
    ];
    for frame in frames {
        assert_eq!(realtime_response_id(&frame), "resp_local_1", "{frame}");
    }
}

#[test]
fn the_tool_call_round_trip_is_the_catalogued_one() {
    // `json-field` / *function call handling over the realtime protocol*. The
    // trigger event and the three fields the Gateway reads off it are the
    // contract; the pipeline emits exactly that event with exactly those
    // fields, or a local model's tool call reaches nobody.
    let contract = contract_of_kind(
        "json-field",
        "function call handling over the realtime protocol",
    );
    contract.assert_mentions("response.function_call_arguments.done");
    for field in ["call_id", "name", "arguments"] {
        contract.assert_mentions(field);
    }

    let frame = events::function_call_arguments_done(
        "resp_local_1",
        "fc_local_2",
        "call_1",
        "request_delegation",
        r#"{"objective":"x"}"#,
    );
    assert_eq!(frame["type"], "response.function_call_arguments.done");
    for field in ["call_id", "name", "arguments"] {
        assert!(
            frame.get(field).is_some(),
            "{field} is missing from {frame}"
        );
    }
    assert!(
        frame["arguments"].is_string(),
        "`arguments` is a JSON string on the wire, not an object"
    );
}

#[test]
fn the_local_family_answers_the_turn_detection_row_that_upstream_leaves_null() {
    // `default-value` / *turn detection by model family*: upstream's table ends
    // `unknown family -> null`, and that null is `docs/architecture.md` §7's
    // "one bug not to port" — the session connects and never hears the user.
    // A local model id is by definition unknown to that table.
    let contract = contract_of_kind("default-value", "turn detection by model family");
    contract.assert_mentions("unknown family -> null");
    contract.assert_mentions("No threshold / silence_duration_ms fields are ever sent");

    let provider = LocalPipelineProvider::with_supplied_stages(
        LocalSettings::default().with_weights(via_realtime_local::WeightsSet {
            reasoning: std::path::PathBuf::from("/m/reasoning/Qwen3-8B-Q4_K_M.gguf"),
            ..via_realtime_local::WeightsSet::resolve("/m")
        }),
    );
    let profile = via_realtime::RealtimeProvider::model_profile(&provider)
        .expect("a local model id resolves to a profile, never to null");
    assert_eq!(
        profile.session_defaults.turn_detection.kind,
        TurnDetectionKind::ServerVad
    );
    assert!(profile.transport_capabilities.audio_input);

    // And the payload that reaches the wire carries it, with no threshold and
    // no silence duration — the half of the row that is about what is *absent*.
    let session = via_realtime::RealtimeProvider::build_session(
        &provider,
        &via_realtime::SessionRequest {
            configured: false,
            agent_context: &via_realtime::AgentContext::default(),
        },
    );
    let turn_detection = &session["audio"]["input"]["turn_detection"];
    assert_eq!(turn_detection["type"], "server_vad");
    assert!(
        turn_detection.get("threshold").is_none(),
        "{turn_detection}"
    );
    assert!(
        turn_detection.get("silence_duration_ms").is_none(),
        "{turn_detection}"
    );
}

#[test]
fn the_response_start_budget_is_the_catalogued_local_one() {
    // `default-value` / *RealtimeFrontend timeouts*: the session default is
    // 30 000 ms and the componentized service declares 60 000. The local
    // pipeline is that service, in-process.
    let contract = contract_of_kind("default-value", "RealtimeFrontend timeouts");
    let millis =
        |duration: std::time::Duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX);
    assert_eq!(
        millis(RESPONSE_START_TIMEOUT),
        contract.number_after("s2s = ")
    );
    assert_eq!(
        millis(via_realtime::DEFAULT_RESPONSE_START_TIMEOUT),
        contract.number_after("provider.responseStartTimeoutMs ?? "),
        "the session default the local budget doubles"
    );
    assert_eq!(
        millis(via_realtime::CONNECT_TIMEOUT),
        contract.number_after("WebSocket connect timeout ")
    );
}

#[test]
fn the_classification_vocabulary_this_crate_relies_on_is_the_catalogued_one() {
    // `error-code` / *provider error classification vocabulary*. The pipeline's
    // two wire refusals only work because the shipped corpus recognises them,
    // and the corpus only helps because the Gateway branches on these seven
    // names.
    let contract = contract_of_kind("error-code", "provider error classification vocabulary");
    for class in [
        ErrorClass::Inactivity,
        ErrorClass::InputBusy,
        ErrorClass::NoActiveResponse,
        ErrorClass::Fatal,
        ErrorClass::CapacityBusy,
        ErrorClass::ResponseSlotBusy,
        ErrorClass::Other,
    ] {
        contract.assert_mentions(class.as_str());
    }

    // The two the pipeline itself produces.
    assert_eq!(
        via_realtime_openai::classify_openai_error(
            &via_realtime_local::active_response_conflict_message("resp_local_1")
        ),
        ErrorClass::ResponseSlotBusy
    );
    assert_eq!(
        via_realtime_openai::classify_openai_error(via_realtime_local::CANCEL_NOT_ACTIVE_MESSAGE),
        ErrorClass::NoActiveResponse
    );
}

#[test]
fn every_response_frame_the_pipeline_emits_is_shipped_response_activity() {
    // Not a catalogued row but a shipped table, and the same discipline: the
    // pipeline's vocabulary is asserted against `via-realtime`'s own list rather
    // than against a copy of it. A name that is not on that list is a frame the
    // Gateway does not count as response activity, which means the watchdog
    // cancels the response while it is still speaking.
    for kind in EMITTED_EVENT_TYPES
        .iter()
        .filter(|kind| kind.starts_with("response."))
    {
        assert!(
            RESPONSE_ACTIVITY_TYPES.contains(kind),
            "{kind} is not response activity to the Gateway"
        );
    }
}

#[test]
fn the_provider_key_is_the_catalogs_and_the_mode_names_are_the_architectures() {
    assert_eq!(via_realtime_local::provider_key(), "local-omni");
    assert_eq!(
        via_realtime_local::LocalMode::Pipeline.qualified(),
        "local-omni:pipeline"
    );
    assert_eq!(
        via_realtime_local::LocalMode::Endpoint.qualified(),
        "local-omni:endpoint"
    );
    // The row exists in `via-catalog`, with no default endpoint — which is what
    // makes "the endpoint must be told where to look" a refusal rather than a
    // guess.
    let row = via_catalog::realtime_provider::realtime_provider_definition("local-omni")
        .expect("a catalog row");
    assert_eq!(row.default_url, None);
    assert!(row.aliases.is_empty());
}
