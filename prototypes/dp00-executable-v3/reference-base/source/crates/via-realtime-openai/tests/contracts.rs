//! Every catalogued value this crate touches, **parsed out of
//! `docs/reference/contracts.json`** rather than retyped.
//!
//! A retyped literal proves the code matches the test; a parsed one proves the
//! code matches the specification, and it fails loudly when a contract is
//! renamed or removed.
//!
//! Three of the assertions below are deliberate *divergences*, and each one is
//! asserted against the contract it diverges from rather than being left
//! unstated: the input sample rate, the tool shape in the pre-GA generation, and
//! the quota sentence upstream classifies as `other`.

mod common;

use std::time::Duration;

use common::contract_of_kind;
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use via_core::Secret;
use via_realtime::{
    AgentContext, CONNECT_TIMEOUT, ErrorClass, ProviderCapabilities, RealtimeProvider,
    SessionRequest,
};
use via_realtime_openai::{
    CONNECT_TOTAL_BUDGET, FIRST_FRAME_TIMEOUT, INPUT_SAMPLE_RATE, KEY, OUTPUT_SAMPLE_RATE,
    OpenAiDialect, OpenAiRealtimeProvider, OpenAiSettings, RealtimeSchema, classify_openai_error,
    session_update_frame,
};

fn provider() -> OpenAiRealtimeProvider {
    OpenAiRealtimeProvider::new(OpenAiSettings {
        api_key: Secret::new("sk-test"),
        ..OpenAiSettings::default()
    })
}

fn context() -> AgentContext {
    AgentContext {
        instructions: "be brief".to_owned(),
        tools: vec![json!({
            "type": "function",
            "function": { "name": "web_search", "description": "search", "parameters": {} }
        })],
        ..AgentContext::default()
    }
}

// ── the classification vocabulary ────────────────────────────────────────────

#[test]
fn every_class_this_corpus_returns_is_in_the_catalogued_vocabulary() {
    // `provider error classification vocabulary`: seven strings the Gateway
    // branches on. A class this crate invented would be a class nothing handles.
    let contract = contract_of_kind("error-code", "provider error classification vocabulary");
    let vocabulary = contract.single_quoted();
    assert_eq!(vocabulary.len(), 7, "{vocabulary:?}");

    for message in [
        "session was closed because no response was generated for 30 seconds",
        "Cancellation failed: no active response found",
        "Conversation already has an active response in progress: resp_x",
        "Incorrect API key provided",
        "something nobody has a pattern for",
    ] {
        let class = classify_openai_error(message);
        assert!(
            vocabulary.iter().any(|name| name == class.as_str()),
            "`{message}` classified as `{class}`, which is not in {vocabulary:?}"
        );
    }
}

#[test]
fn the_five_upstream_fatal_sentences_are_fatal_here_too() {
    // `realtime provider fatal error classification` is test-locked upstream. A
    // gateway relays its upstream's prose verbatim, so a session that reaches
    // DashScope through litellm is configured as *this* provider and receives
    // these sentences.
    let contract = contract_of_kind("error-code", "realtime provider fatal error classification");
    let quoted = contract.single_quoted();
    // 'fatal', then the five sentences, then 'other', then the divergent one.
    assert_eq!(quoted[0], "fatal");
    for sentence in &quoted[1..6] {
        assert_eq!(
            classify_openai_error(sentence),
            ErrorClass::Fatal,
            "{sentence}"
        );
    }
    assert_eq!(quoted[6], "other");
}

#[test]
fn the_quota_sentence_upstream_calls_other_is_fatal_here_and_that_is_deliberate() {
    // The divergence, asserted rather than left unstated.
    //
    // Upstream classifies `You exceeded your current quota…` as `other` — and it
    // is right to, because that is **OpenAI's** wording and DashScope never
    // sends it, so for DashScope's corpus it is an unrecognised message. For
    // this provider it is the endpoint's own billing message, and no amount of
    // reconnecting will fix it.
    let contract = contract_of_kind("error-code", "realtime provider fatal error classification");
    let quoted = contract.single_quoted();
    let divergent = &quoted[7];
    assert!(
        divergent.contains("exceeded your current quota"),
        "{divergent}"
    );
    assert_eq!(classify_openai_error(divergent), ErrorClass::Fatal);
    assert!(
        ErrorClass::Fatal.is_suppressed(),
        "fatal suppresses the user-facing error and blocks the connection"
    );
}

// ── the capability declaration ───────────────────────────────────────────────

#[test]
fn the_capability_declaration_differs_from_the_catalogued_baseline_in_exactly_two_flags() {
    // `DEFAULT_CAPABILITIES (provider capability defaults)`: the five names and
    // their default values. Anything this provider does not declare must take
    // the catalogued value.
    let contract = contract_of_kind(
        "default-value",
        "DEFAULT_CAPABILITIES (provider capability defaults)",
    );
    let baseline: Vec<(String, bool)> = contract
        .exact_value
        .trim_matches(|c| c == '{' || c == '}' || c == ' ')
        .split(',')
        .filter_map(|pair| {
            let (name, value) = pair.split_once(':')?;
            Some((name.trim().to_owned(), value.trim() == "true"))
        })
        .collect();
    assert_eq!(baseline.len(), 5, "{baseline:?}");

    let declared = provider().capabilities();
    let mine: Vec<(&str, bool)> = via_realtime::CAPABILITY_FLAGS
        .iter()
        .copied()
        .zip(declared.as_array())
        .collect();

    let differences: Vec<&str> = baseline
        .iter()
        .zip(mine.iter())
        .filter_map(|((catalogued, default), (name, value))| {
            assert_eq!(catalogued, name, "the flag order is the catalogue's");
            (default != value).then_some(*name)
        })
        .collect();
    assert_eq!(
        differences,
        ["singleResponseSlot", "perResponseInstructions"],
        "declared: {declared:?}"
    );
    assert_eq!(
        declared,
        ProviderCapabilities {
            single_response_slot: true,
            per_response_instructions: true,
            ..ProviderCapabilities::DEFAULT
        }
    );
}

// ── the timeouts ─────────────────────────────────────────────────────────────

#[test]
fn the_connect_walk_fits_inside_the_catalogued_connect_timeout() {
    // `RealtimeFrontend timeouts`: 25 000 ms, hard-coded, then the socket is
    // terminated. The walk has to leave room for `session.created` /
    // `session.update` / `session.updated` on top of it.
    let contract = contract_of_kind("default-value", "RealtimeFrontend timeouts");
    let catalogued = Duration::from_millis(contract.number_after("WebSocket connect timeout"));
    assert_eq!(catalogued, CONNECT_TIMEOUT);
    assert!(
        CONNECT_TOTAL_BUDGET < catalogued,
        "{CONNECT_TOTAL_BUDGET:?} must fit inside {catalogued:?}"
    );
    // Four candidates × a three-second probe is the budget exactly: the ceiling
    // exists so the walk cannot cost more than that arithmetic.
    assert_eq!(FIRST_FRAME_TIMEOUT * 4, CONNECT_TOTAL_BUDGET);
}

// ── the sample rates ─────────────────────────────────────────────────────────

#[test]
fn the_output_rate_is_the_catalogued_one_and_the_input_rate_deliberately_is_not() {
    // `audio sample rates`: 16 000 in, 24 000 out, "dashscope and s2s" — this
    // provider is neither. The OpenAI Realtime input buffer is 24 kHz PCM16 in
    // both generations and GA accepts no other PCM rate, so declaring 16 000
    // here would tell every client to capture at a rate the endpoint refuses.
    let contract = contract_of_kind("default-value", "audio sample rates");
    let catalogued_input = contract.number_after("inputSampleRate");
    let catalogued_output = contract.number_after("outputSampleRate");
    assert_eq!(catalogued_input, 16_000);
    assert_eq!(catalogued_output, 24_000);

    assert_eq!(u64::from(OUTPUT_SAMPLE_RATE.hz()), catalogued_output);
    assert_ne!(u64::from(INPUT_SAMPLE_RATE.hz()), catalogued_input);
    assert_eq!(INPUT_SAMPLE_RATE.hz(), OUTPUT_SAMPLE_RATE.hz());
    // The contract itself says why the value has to be published rather than
    // assumed, which is exactly why the difference is safe.
    contract.assert_mentions("inputSampleRate");
    assert!(
        contract.why.contains("voice.ready"),
        "the rate reaches every client: {}",
        contract.why
    );
}

// ── the wire frames ──────────────────────────────────────────────────────────

#[test]
fn the_pre_ga_session_update_is_the_catalogued_beta_envelope() {
    // `outgoing provider frames (OpenAI beta dialect)`: the envelope and its key
    // order. `event_id` is prepended by `encodeOutgoing`, so a payload key of
    // the same name would be overwritten — which is why it must lead.
    let contract = contract_of_kind("ws-event", "outgoing provider frames (OpenAI beta dialect)");
    contract.assert_key_order(&["event_id", "type", "session"]);

    let context = context();
    let frame = session_update_frame(&provider(), RealtimeSchema::Preview, &context)
        .expect("the payload serializes");
    let value: Value = serde_json::from_str(&frame).expect("valid json");
    let keys: Vec<&str> = value
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, ["event_id", "type", "session"]);
    assert_eq!(value["type"], json!("session.update"));
    let event_id = value["event_id"].as_str().expect("an event id");
    let hex = event_id
        .strip_prefix("event_")
        .expect("the catalogued prefix");
    assert_eq!(hex.len(), 32, "a uuid with no dashes");
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn a_subsequent_update_omits_everything_the_catalogue_says_it_must() {
    // `DashScope session.update payload (subsequent, configured)`: modalities,
    // voice, formats and turn_detection are omitted on every update after the
    // first. This provider follows the same rule for a second reason — OpenAI
    // refuses a voice change once audio has been generated.
    let contract = contract_of_kind(
        "json-field",
        "DashScope session.update payload (subsequent, configured)",
    );
    contract.assert_mentions("omitted on every update after the first");
    for omitted in ["modalities", "voice", "turn_detection"] {
        contract.assert_mentions(omitted);
    }

    let provider = provider();
    let context = context();
    for schema in [RealtimeSchema::Ga, RealtimeSchema::Preview] {
        let payload = provider.build_session_for(
            schema,
            &SessionRequest {
                configured: true,
                agent_context: &context,
            },
        );
        for omitted in [
            "modalities",
            "output_modalities",
            "voice",
            "audio",
            "turn_detection",
            "input_audio_transcription",
            "input_audio_format",
            "output_audio_format",
        ] {
            assert!(payload.get(omitted).is_none(), "{schema}: {omitted}");
        }
        assert_eq!(payload["instructions"], json!("be brief"));
        assert!(payload.get("tools").is_some());
    }
}

#[test]
fn the_tool_shape_is_the_catalogued_ga_one_in_both_generations() {
    // `tool schema shape per dialect`: beta nests a tool under `function`, GA
    // flattens it. **This provider flattens in both**, which is the divergence —
    // ARGO's payload builder calls `tools` *"the one part of the payload that did
    // not move at GA"* and emits the flat shape to both generations, because the
    // pre-GA OpenAI Realtime API takes it flat. The nested form is DashScope's,
    // not the beta dialect's.
    let contract = contract_of_kind("json-field", "tool schema shape per dialect");
    contract.assert_mentions("type: 'function', name, description, parameters");
    assert!(
        contract.why.contains("flattening lives in the provider"),
        "the catalogue still puts the flattening in the provider: {}",
        contract.why
    );

    let provider = provider();
    let context = context();
    for schema in [RealtimeSchema::Ga, RealtimeSchema::Preview] {
        let payload = provider.build_session_for(
            schema,
            &SessionRequest {
                configured: false,
                agent_context: &context,
            },
        );
        let tool = &payload["tools"][0];
        let keys: Vec<&str> = tool
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            ["type", "name", "description", "parameters"],
            "{schema}"
        );
        assert!(tool.get("function").is_none(), "{schema}");
    }
}

#[test]
fn a_tool_result_uses_the_catalogued_ga_id_namespace() {
    // `function call handling over the realtime protocol`: "GA identical shape
    // but id namespace 'fco_'". The GA server rejects an id from the wrong
    // namespace outright, so which dialect the provider hands back is
    // load-bearing.
    let contract = contract_of_kind(
        "json-field",
        "function call handling over the realtime protocol",
    );
    contract.assert_mentions("fco_");
    contract.assert_mentions("function_call_output");

    let provider = provider();
    assert_eq!(provider.schema(), RealtimeSchema::Ga);
    let item = provider
        .protocol()
        .function_output_item("call_7", &json!({ "ok": true }));
    assert_eq!(item["type"], json!("function_call_output"));
    // `output` is always a JSON-encoded string, never a nested object.
    assert_eq!(item["output"], json!(r#"{"ok":true}"#));
    assert!(
        provider
            .protocol()
            .conversation_item_id(&item)
            .starts_with("fco_")
    );
}

// ── identity ─────────────────────────────────────────────────────────────────

#[test]
fn this_provider_is_a_via_addition_and_not_one_of_the_two_upstream_keys() {
    // `realtime provider keys and aliases`: upstream ships two. `openai` is
    // VIA's, per `docs/architecture.md` §7 — which is why it carries no alias:
    // no client and no config file has ever named it.
    let contract = contract_of_kind("provider-id", "realtime provider keys and aliases");
    contract.assert_mentions("dashscope");
    contract.assert_mentions("speech-to-speech");
    assert!(
        !contract.exact_value.contains("openai"),
        "the catalogue must not claim `openai` is upstream's: {}",
        contract.exact_value
    );

    let provider = provider();
    assert_eq!(provider.key(), KEY);
    assert!(provider.aliases().is_empty());
    let row = via_catalog::realtime_provider_definition(KEY).expect("the openai row");
    assert_eq!(provider.label(), row.label);
    assert!(row.aliases.is_empty());
}

#[test]
fn the_credential_always_rides_the_upgrade_in_the_dialects_own_header() {
    // `provider Authorization headers` catalogues the two upstream providers.
    // DashScope's rule is the one this follows: the header is sent **always**,
    // even when the key is empty, so a missing credential fails as a 401 the
    // `fatal` classifier recognises rather than as an unauthenticated upgrade
    // the service answers some other way.
    let contract = contract_of_kind("http-route", "provider Authorization headers");
    contract.assert_mentions("Bearer");
    contract.assert_mentions("(always)");

    let openai = OpenAiRealtimeProvider::new(OpenAiSettings::default());
    assert_eq!(
        openai.auth_headers(),
        vec![("Authorization".to_owned(), "Bearer ".to_owned())],
        "empty credential, header still sent"
    );

    let azure = OpenAiRealtimeProvider::new(OpenAiSettings {
        dialect: OpenAiDialect::Azure,
        base_url: "https://r.openai.azure.com".to_owned(),
        api_key: Secret::new("k"),
        ..OpenAiSettings::default()
    });
    assert_eq!(
        azure.auth_headers(),
        vec![("api-key".to_owned(), "k".to_owned())]
    );
}
