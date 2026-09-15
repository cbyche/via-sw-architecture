//! Every catalogued value this crate owns, asserted against
//! `docs/reference/contracts.json`.
//!
//! Nothing here retypes a contract value. Each test pulls the record out of the
//! catalogue and compares it to what the crate actually produces, so a
//! catalogue edit and a code edit have to agree.
//!
//! The twelve records `via-conformance` assigns to this crate:
//!
//! | Kind | Name |
//! | --- | --- |
//! | `provider-descriptor` | built-in realtime providers |
//! | `default-value` | audio sample rates |
//! | `error-code` | DashScope classifyError regexes |
//! | `error-code` | s2s classifyError regexes |
//! | `error-code` | provider error classification vocabulary |
//! | `error-code` | connect timeout messages |
//! | `error-code` | missing configuration messages |
//! | `http-route` | provider Authorization headers |
//! | `json-field` | DashScope session.update payload (first, unconfigured) |
//! | `json-field` | DashScope session.update payload (subsequent, configured) |
//! | `json-field` | s2s session.update payload (GA dialect) |
//! | `json-field` | buildSpeakResponse payloads |
//! | `json-field` | buildResultInjection payload |
//! | `prompt-text` | backend permission injection item text |
//! | `prompt-text` | `<backend_permission_request>` injection |
//!
//! plus the endpoint, model-table and timeout records it reads through
//! `via-catalog`.

mod common;

use common::contract_of_kind;
use pretty_assertions::assert_eq;
use regex::Regex;
use serde_json::{Value, json};
use via_core::Secret;
use via_i18n::Locale;
use via_realtime::{AgentContext, ErrorClass, PermissionRequest, RealtimeProvider, SessionRequest};
use via_realtime_dashscope::{
    DashScopeProvider, DashScopeSettings, SpeechToSpeechProvider, SpeechToSpeechSettings,
    classify_dashscope_error, classify_speech_to_speech_error, patterns_compile,
    permission_request_text,
};

fn dashscope(model: &str) -> DashScopeProvider {
    DashScopeProvider::new(DashScopeSettings {
        model: model.to_owned(),
        api_key: Secret::new("sk-test"),
        ..DashScopeSettings::default()
    })
}

fn speech_to_speech() -> SpeechToSpeechProvider {
    SpeechToSpeechProvider::new(SpeechToSpeechSettings {
        configured: true,
        ..SpeechToSpeechSettings::default()
    })
}

fn context() -> AgentContext {
    AgentContext {
        instructions: "i".to_owned(),
        tools: vec![json!({
            "type": "function",
            "function": { "name": "t", "description": "d", "parameters": {} },
        })],
        ..AgentContext::default()
    }
}

fn key_list(value: &Value) -> Vec<String> {
    value
        .as_object()
        .map(|object| object.keys().cloned().collect())
        .unwrap_or_default()
}

// ── the provider descriptors ────────────────────────────────────────────────

#[test]
fn both_descriptors_are_the_catalogued_ones() {
    let record = contract_of_kind("provider-descriptor", "built-in realtime providers");
    let quoted = record.single_quoted();
    assert_eq!(
        quoted,
        [
            dashscope(via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL)
                .key()
                .to_owned(),
            dashscope(via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL)
                .label()
                .to_owned(),
            dashscope(via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL).aliases()[0].clone(),
            speech_to_speech().key().to_owned(),
            speech_to_speech().label().to_owned(),
            speech_to_speech().aliases()[0].clone(),
        ],
        "the catalogue lists key, label and alias for each provider, in order"
    );

    let expected_input = record.number_after("inputSampleRate ");
    let expected_output = record.number_after("outputSampleRate ");
    for provider in [
        &dashscope(via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL) as &dyn RealtimeProvider,
        &speech_to_speech(),
    ] {
        assert_eq!(u64::from(provider.input_sample_rate()), expected_input);
        assert_eq!(u64::from(provider.output_sample_rate()), expected_output);
    }

    assert_eq!(
        speech_to_speech()
            .response_start_timeout()
            .map(|timeout| timeout.as_millis()),
        Some(u128::from(record.number_after("responseStartTimeoutMs ")))
    );
    // DashScope declares none, so the session default applies.
    assert_eq!(
        dashscope(via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL).response_start_timeout(),
        None
    );
}

#[test]
fn the_sample_rates_are_the_catalogued_ones() {
    let record = contract_of_kind("default-value", "audio sample rates");
    let provider = dashscope(via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL);
    assert_eq!(
        u64::from(provider.input_sample_rate()),
        record.number_after("inputSampleRate = ")
    );
    assert_eq!(
        u64::from(provider.output_sample_rate()),
        record.number_after("outputSampleRate = ")
    );
    // The catalogue says the two providers agree; asserting that is what stops
    // one of them drifting on its own.
    record.assert_mentions("(dashscope and s2s)");
    assert_eq!(
        provider.input_sample_rate(),
        speech_to_speech().input_sample_rate()
    );
    assert_eq!(
        provider.output_sample_rate(),
        speech_to_speech().output_sample_rate()
    );
}

#[test]
fn the_aliases_and_the_configured_predicates_are_the_catalogued_ones() {
    let record = contract_of_kind("default-value", "provider aliases and configured gating");
    record.assert_mentions("dashscope isConfigured = Boolean(dashscopeApiKey)");
    record.assert_mentions("a default endpoint alone does NOT count as configured");

    assert!(!DashScopeProvider::default().is_configured());
    assert!(
        DashScopeProvider::new(DashScopeSettings {
            api_key: Secret::new("k"),
            ..DashScopeSettings::default()
        })
        .is_configured()
    );
    assert!(!SpeechToSpeechProvider::default().is_configured());
}

// ── endpoints and credentials ───────────────────────────────────────────────

#[test]
fn the_endpoint_defaults_and_url_construction_are_the_catalogued_ones() {
    let record = contract_of_kind("default-value", "endpoint defaults and URL construction");

    let dashscope_url = record.quoted_after("DEFAULT_DASHSCOPE_REALTIME_URL = ");
    let s2s_url = record.quoted_after("DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL = ");
    assert_eq!(
        DashScopeProvider::default().url(),
        Ok(format!(
            "{dashscope_url}?model={}",
            via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL
        ))
    );
    assert_eq!(SpeechToSpeechProvider::default().url(), Ok(s2s_url));

    // The workspace form is the catalogue's template with the id substituted.
    let workspace = record
        .template_after("workspace form = ")
        .replace("${DASHSCOPE_WORKSPACE_ID}", "ws-42");
    let provider = DashScopeProvider::new(DashScopeSettings {
        base_url: via_catalog::realtime_provider::dashscope_workspace_realtime_url("ws-42"),
        model: "m".to_owned(),
        ..DashScopeSettings::default()
    });
    assert_eq!(provider.url(), Ok(format!("{workspace}?model=m")));
}

#[test]
fn the_authorization_headers_are_the_catalogued_ones() {
    let record = contract_of_kind("http-route", "provider Authorization headers");
    record.assert_mentions("(always)");
    record.assert_mentions("only when");

    let dashscope_header = record
        .section("dashscope: ", Some("; s2s:"))
        .template_after("Authorization: ")
        .replace("${config.dashscopeApiKey}", "sk-live");
    assert_eq!(
        DashScopeProvider::new(DashScopeSettings {
            api_key: Secret::new("sk-live"),
            ..DashScopeSettings::default()
        })
        .headers(),
        [("Authorization".to_owned(), dashscope_header)]
    );

    let s2s_header = record
        .section("s2s: ", None)
        .template_after("Authorization: ")
        .replace("${token}", "hf-token");
    assert_eq!(
        SpeechToSpeechProvider::new(SpeechToSpeechSettings {
            auth_token: Secret::new("hf-token"),
            ..SpeechToSpeechSettings::default()
        })
        .headers(),
        [("Authorization".to_owned(), s2s_header)]
    );
    // `else {}` — no header at all, not an empty bearer.
    assert_eq!(SpeechToSpeechProvider::default().headers(), Vec::new());
}

// ── the two provider-supplied sentences ─────────────────────────────────────

#[test]
fn the_missing_configuration_messages_are_the_catalogued_ones() {
    let record = contract_of_kind("error-code", "missing configuration messages");
    assert_eq!(
        DashScopeProvider::default().missing_configuration_message(Locale::Zh),
        record.quoted_after("dashscope: ")
    );
    assert_eq!(
        SpeechToSpeechProvider::default().missing_configuration_message(Locale::Zh),
        record.quoted_after("s2s: ")
    );
    // The setup-gate variant is a different sentence and belongs to `via-core`;
    // asserting it is *not* what a provider says keeps the two from merging.
    let setup_gate = record.quoted_after("setup-gate variant: ");
    assert_ne!(
        DashScopeProvider::default().missing_configuration_message(Locale::Zh),
        setup_gate
    );
}

#[test]
fn the_connect_timeout_messages_are_the_catalogued_ones() {
    let record = contract_of_kind("error-code", "connect timeout messages");
    assert_eq!(
        DashScopeProvider::default().connect_timeout_message(Locale::Zh),
        record.quoted_after("dashscope: ")
    );

    let url = "ws://127.0.0.1:9000/v1/realtime";
    let expected = record
        .template_after("s2s: ")
        .replace("${config.speechToSpeechRealtimeUrl}", url);
    let provider = SpeechToSpeechProvider::new(SpeechToSpeechSettings {
        url: url.to_owned(),
        ..SpeechToSpeechSettings::default()
    });
    assert_eq!(provider.connect_timeout_message(Locale::Zh), expected);
}

#[test]
fn both_sentences_render_in_every_locale() {
    // The catalogue's `zh` is upstream's own text; `en` and `ko` are authored
    // peers and must not be a key name or an unfilled template.
    for locale in [Locale::En, Locale::Zh, Locale::Ko] {
        for message in [
            DashScopeProvider::default().missing_configuration_message(locale),
            DashScopeProvider::default().connect_timeout_message(locale),
            SpeechToSpeechProvider::default().missing_configuration_message(locale),
            SpeechToSpeechProvider::default().connect_timeout_message(locale),
        ] {
            assert!(!message.is_empty(), "{locale}");
            assert!(!message.contains('{'), "{locale}: {message}");
            assert!(!message.contains("<via-i18n:"), "{locale}: {message}");
        }
    }
}

// ── error classification ────────────────────────────────────────────────────

/// Apply a catalogued corpus in the catalogue's own order.
fn classify_by_catalogue(corpus: &[(String, Vec<Regex>)], message: &str) -> String {
    corpus
        .iter()
        .find(|(_, patterns)| patterns.iter().any(|pattern| pattern.is_match(message)))
        .map_or_else(|| "other".to_owned(), |(class, _)| class.clone())
}

/// Messages that exercise every arm of both corpora, plus the near misses.
///
/// Deliberately all-ASCII: the catalogued patterns are compiled with Rust's
/// Unicode-aware `(?i)` and this crate folds ASCII only, which is JavaScript's
/// own behaviour. The two agree on every ASCII input, and the one input where
/// they do not is asserted separately in `src/classify.rs`.
const CORPUS: [&str; 30] = [
    "",
    "Your session was closed because no response was generated for 180 seconds.",
    "Your session was closed because no response was generated for 60 seconds",
    "session was closed because no response was generated for seconds",
    "Cannot create response while user is speaking.",
    "USER IS SPEAKING",
    "No active response",
    "no active response to cancel",
    "InvalidApiKey: Invalid API-key provided.",
    "Invalid API-key provided.",
    "invalid_api_key",
    "invalid-api-key",
    "incorrect api key",
    "Authentication failed",
    "Unauthorized",
    "Unexpected server response: 401",
    "Unexpected server response: 403",
    "Unexpected server response: 500",
    "Arrearage: Access denied, please make sure your account is in good standing.",
    "no arrearages recorded",
    "AllocationQuota.FreeTierOnly: The free tier of the model has been exhausted.",
    "Free allocated quota exceeded.",
    "Free tier for this model is exhausted",
    "Model.AccessDenied",
    "model_not_found",
    "the found model was not the requested one",
    "You exceeded your current quota, please check your plan.",
    "All 1 session slots are in use. Disconnect an existing client first.",
    "session_limit_reached",
    "Another response is in progress.",
];

#[test]
fn the_dashscope_corpus_agrees_with_the_catalogued_regexes() {
    assert!(patterns_compile(), "the transcribed patterns must compile");
    let record = contract_of_kind("error-code", "DashScope classifyError regexes");
    let corpus = record.js_regexes();
    assert_eq!(
        corpus
            .iter()
            .map(|(class, _)| class.as_str())
            .collect::<Vec<_>>(),
        ["inactivity", "input_busy", "no_active_response", "fatal"],
        "the catalogue's order is the evaluation order"
    );
    // The four `fatal` alternatives are one class, checked together.
    assert_eq!(corpus[3].1.len(), 4);

    for message in CORPUS {
        assert_eq!(
            classify_dashscope_error(message).as_str(),
            classify_by_catalogue(&corpus, message),
            "{message:?}"
        );
    }
}

#[test]
fn the_speech_to_speech_corpus_agrees_with_the_catalogued_regexes() {
    let record = contract_of_kind("error-code", "s2s classifyError regexes");
    let corpus = record.js_regexes();
    assert_eq!(
        corpus
            .iter()
            .map(|(class, _)| class.as_str())
            .collect::<Vec<_>>(),
        ["capacity_busy", "response_slot_busy", "no_active_response"]
    );
    // The catalogue names the literal service sentence the corpus is locked to.
    record.assert_why_mentions("All 1 session slots are in use.");

    for message in CORPUS {
        assert_eq!(
            classify_speech_to_speech_error(message).as_str(),
            classify_by_catalogue(&corpus, message),
            "{message:?}"
        );
    }
}

#[test]
fn every_class_either_provider_produces_is_in_the_catalogued_vocabulary() {
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

    let dashscope = dashscope(via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL);
    let s2s = speech_to_speech();
    for message in CORPUS {
        for class in [
            dashscope.classify_error(message),
            s2s.classify_error(message),
        ] {
            assert!(
                vocabulary.iter().any(|name| name == class.as_str()),
                "{class} is not in the catalogued vocabulary"
            );
        }
    }
    // Between them the two providers reach every member but one — `fatal` is
    // DashScope-only and `capacity_busy` / `response_slot_busy` are
    // speech-to-speech-only, which is itself the contract.
    assert_eq!(
        dashscope.classify_error("session_limit_reached"),
        ErrorClass::Other
    );
    assert_eq!(
        s2s.classify_error("Invalid API-key provided."),
        ErrorClass::Other
    );
}

// ── the session payloads ────────────────────────────────────────────────────

#[test]
fn the_first_dashscope_session_update_matches_the_catalogued_payload() {
    let record = contract_of_kind(
        "json-field",
        "DashScope session.update payload (first, unconfigured)",
    );
    let context = context();
    let session =
        dashscope(via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL).build_session(&SessionRequest {
            configured: false,
            agent_context: &context,
        });
    record.assert_key_order(&key_list(&session));

    // The two literals the catalogue spells out.
    assert_eq!(session["modalities"], json!(["text", "audio"]));
    assert_eq!(
        session["output_audio_format"],
        json!(record.quoted_after("output_audio_format: "))
    );
    assert_eq!(
        session["input_audio_format"],
        json!(record.quoted_after("input_audio_format: "))
    );
    // The catalogue records both conditionals; assert each still holds.
    record.assert_why_mentions("only when modelCapabilities.audioOutput");
    record.assert_why_mentions("only when transportCapabilities.audioInput");
}

#[test]
fn every_later_dashscope_session_update_matches_the_catalogued_payload() {
    let record = contract_of_kind(
        "json-field",
        "DashScope session.update payload (subsequent, configured)",
    );
    record.assert_mentions("are omitted on every update after the first");
    let context = context();
    let session =
        dashscope(via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL).build_session(&SessionRequest {
            configured: true,
            agent_context: &context,
        });
    assert_eq!(key_list(&session), ["instructions", "tools"]);
    for omitted in [
        "modalities",
        "voice",
        "output_audio_format",
        "turn_detection",
    ] {
        assert_eq!(session.get(omitted), None, "{omitted}");
    }
}

#[test]
fn each_family_negotiates_the_catalogued_voice_and_turn_detection() {
    let record = contract_of_kind("default-value", "DashScope model catalog ids and profiles");
    let context = context();
    for profile in via_catalog::dashscope_realtime_model_profiles() {
        let row = record.section(&format!("'{}'", profile.id), Some(")"));
        let voice = row.quoted_after("voice ");
        let turn_detection = if row.exact_value.contains("semantic_vad") {
            "semantic_vad"
        } else {
            "smart_turn"
        };

        let session = dashscope(&profile.id).build_session(&SessionRequest {
            configured: false,
            agent_context: &context,
        });
        assert_eq!(session["voice"], json!(voice), "{}", profile.id);
        assert_eq!(
            session["turn_detection"],
            json!({ "type": turn_detection }),
            "{}",
            profile.id
        );
        assert_eq!(row.quoted_after("label "), profile.label.as_ref());
    }
}

#[test]
fn an_unknown_model_is_refused_at_connect_as_the_catalogue_says_it_must_be() {
    // The catalogue's own rationale for why an unknown id matters: it is what
    // makes `connect()` reject up front. VIA has no `unknown` family, so the
    // rejection moved to `preflight()`; the behaviour it names has not.
    let record = contract_of_kind("default-value", "DashScope model catalog ids and profiles");
    record.assert_mentions("Unknown id -> family 'unknown', all capabilities false");
    record.assert_why_mentions("reject up front");

    let provider = dashscope("qwen-audio-3.0-realtime-turbo");
    assert!(provider.preflight().is_err());
    assert!(
        provider
            .preflight()
            .expect_err("refused")
            .message(Locale::Zh)
            .contains("qwen-audio-3.0-realtime-turbo"),
        "the refusal must name the id the operator typed"
    );
}

#[test]
fn the_s2s_session_update_matches_the_catalogued_ga_payload() {
    let record = contract_of_kind("json-field", "s2s session.update payload (GA dialect)");
    let context = context();
    let session = speech_to_speech().build_session(&SessionRequest {
        configured: false,
        agent_context: &context,
    });
    record.assert_key_order(&key_list(&session));

    assert_eq!(session["type"], json!(record.quoted_after("type: ")));
    assert_eq!(session["output_modalities"], json!(["audio"]));
    assert_eq!(
        session["audio"]["input"]["turn_detection"]["type"],
        json!(record.quoted_after("turn_detection: { type: "))
    );
    assert_eq!(
        session["audio"]["output"]["format"],
        json!({
            "type": record.quoted_after("format: { type: "),
            "rate": record.number_after("rate: "),
        })
    );
    // The absence is the contract, and the catalogue says so in `why`.
    record.assert_why_mentions("deliberate ABSENCE of audio.input.format");
    assert_eq!(session["audio"]["input"].get("format"), None);
}

#[test]
fn the_ga_tool_shape_is_the_catalogued_one() {
    let record = contract_of_kind("json-field", "tool schema shape per dialect");
    record.assert_mentions("flattened by s2s.buildSession");
    let context = context();

    let beta =
        dashscope(via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL).build_session(&SessionRequest {
            configured: false,
            agent_context: &context,
        });
    let ga = speech_to_speech().build_session(&SessionRequest {
        configured: false,
        agent_context: &context,
    });

    // beta: `{ type: 'function', function: { name, description, parameters } }`
    assert_eq!(key_list(&beta["tools"][0]), ["type", "function"]);
    assert_eq!(
        key_list(&beta["tools"][0]["function"]),
        ["name", "description", "parameters"]
    );
    // GA: `{ type: 'function', name, description, parameters }`
    assert_eq!(
        key_list(&ga["tools"][0]),
        ["type", "name", "description", "parameters"]
    );
}

// ── the response bodies ─────────────────────────────────────────────────────

#[test]
fn both_speak_responses_match_the_catalogued_payloads() {
    let record = contract_of_kind("json-field", "buildSpeakResponse payloads");
    record.assert_why_mentions("conversation:'none' keeps it out of the conversation history");

    let dashscope_body =
        dashscope(via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL).build_speak_response("x");
    record
        .section("dashscope: ", Some("; s2s:"))
        .assert_key_order(&key_list(&dashscope_body));
    assert_eq!(dashscope_body["conversation"], json!("none"));
    assert_eq!(dashscope_body.get("tool_choice"), None);

    let s2s_body = speech_to_speech().build_speak_response("x");
    record
        .section("s2s: ", None)
        .assert_key_order(&key_list(&s2s_body));
    assert_eq!(s2s_body["modalities"], json!(["audio"]));
    assert_eq!(s2s_body["tool_choice"], json!("none"));
}

#[test]
fn both_result_injections_match_the_catalogued_payload() {
    let record = contract_of_kind("json-field", "buildResultInjection payload");
    let expected_item = json!({
        "type": record.quoted_after("item: { type: "),
        "role": record.quoted_after("role: "),
        "content": [{ "type": record.quoted_after("content: [{ type: "), "text": "结果" }],
    });

    for injection in [
        dashscope(via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL).build_result_injection("结果"),
        speech_to_speech().build_result_injection("结果"),
    ] {
        assert_eq!(injection.item, expected_item);
        record
            .section("response: ", None)
            .assert_key_order(&key_list(&injection.response));
        assert_eq!(
            injection.response["tool_choice"],
            json!(record.quoted_after("tool_choice: "))
        );
    }
}

#[test]
fn both_permission_items_match_the_catalogued_prompt_text() {
    let permission = PermissionRequest {
        id: "perm_1".to_owned(),
        summary: "run tests".to_owned(),
    };
    let rendered = permission_request_text(&permission);

    // The catalogue carries this one twice, in two spellings of the same
    // template. Both must produce the same text.
    let interpolated = contract_of_kind("prompt-text", "backend permission injection item text")
        .unescaped()
        .replace("${permission.id}", &permission.id)
        .replace("${permission.summary}", &permission.summary);
    assert_eq!(rendered, interpolated);

    let bracketed = contract_of_kind("prompt-text", "<backend_permission_request> injection")
        .unescaped()
        .replace("[permission.id]", &permission.id)
        .replace("[permission.summary]", &permission.summary);
    assert_eq!(rendered, bracketed);

    // …and both providers must inject exactly it.
    for injection in [
        dashscope(via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL)
            .build_permission_injection(&permission),
        speech_to_speech().build_permission_injection(&permission),
    ] {
        assert_eq!(injection.item["content"][0]["text"], json!(rendered));
        assert_eq!(injection.response["tool_choice"], json!("none"));
    }
}

#[test]
fn the_permission_field_names_are_the_ones_the_prompt_refers_to() {
    let record = contract_of_kind("prompt-text", "<backend_permission_request> injection");
    // Renaming either half alone wedges `respond_agent_permission`; the
    // catalogue's rationale says so, and this is the assertion that it holds.
    record.assert_why_mentions("authorization_id");
    record.assert_why_mentions("PROMPT.md");
    assert!(
        record
            .exact_value
            .contains(via_realtime_dashscope::PERMISSION_REQUEST_OPEN_TAG)
    );
    assert!(
        record
            .exact_value
            .contains(via_realtime_dashscope::PERMISSION_AUTHORIZATION_ID_FIELD)
    );
    assert!(
        record
            .exact_value
            .contains(via_realtime_dashscope::PERMISSION_OPERATION_FIELD)
    );
}

// ── timeouts ────────────────────────────────────────────────────────────────

#[test]
fn the_speech_to_speech_response_start_budget_is_the_catalogued_one() {
    let record = contract_of_kind("default-value", "RealtimeFrontend timeouts");
    assert_eq!(
        speech_to_speech()
            .response_start_timeout()
            .map(|timeout| timeout.as_millis()),
        Some(u128::from(record.number_after("s2s = ")))
    );
    // The catalogue's default is what a provider that declares nothing gets;
    // DashScope is that provider.
    record.assert_mentions("provider.responseStartTimeoutMs ?? 30000");
    assert_eq!(
        dashscope(via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL).response_start_timeout(),
        None
    );
}
