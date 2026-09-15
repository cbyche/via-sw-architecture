//! Four `via-realtime`-owned rows this crate asserts directly.
//!
//! `via-realtime`'s own `tests/contracts.rs` already asserts most of what it
//! owns; the four rows here are the ones nothing shipped compares yet, because
//! the code they describe lives one layer up in `via-voice` (the frontend tool
//! catalog, the input-part `_meta` namespace) or nowhere yet at all (the
//! `describeActiveRealtime` capability flag sets). Each test below is the
//! comparison the row was waiting for — the catalogued value, parsed rather
//! than retyped, against the value VIA actually ships.
//!
//! The other five rows this agent closed in this pair of crates — two more
//! `via-realtime` rows, and all three `via-realtime-openai` rows — needed no
//! new test at all: a real, already-shipped comparison exists in
//! `via-realtime`'s or `via-realtime-openai`'s own tree, and the registry rows
//! for those point at it directly as [`via_conformance::Coverage::Behavioural`]
//! rather than duplicating it here.

use serde_json::Value;
use via_conformance::expect_contract;
use via_conformance::value;
use via_i18n::Locale;

// ── model / transport capability flag names ─────────────────────────────────

/// `default-value` / *model / transport capability flag names*.
///
/// > `MODEL_CAPABILITY_FLAGS = […]; TRANSPORT_CAPABILITY_FLAGS = […]`. Every
/// > listed flag must be a boolean and no extra non-boolean value is allowed.
///
/// `via-catalog` owns `ModelCapabilities` and `TransportCapabilities`
/// themselves (asserted, as a divergence, by `via-catalog`'s own
/// `tests/realtime_catalog.rs::capability_flag_names_and_order`); what this row
/// owns is `via-realtime`'s obligation to publish both sets over
/// `describeActiveRealtime` unchanged. The struct is the closed list and the
/// type is the boolean check, so both upstream failure modes are compile
/// errors here — what is worth a runtime test is that the *names* published on
/// `/api/health` are still the catalogued ones, through the real registry path
/// rather than by constructing the structs directly.
#[test]
fn model_and_transport_capability_flags_are_the_catalogued_closed_boolean_sets() {
    let contract = expect_contract("default-value", "model / transport capability flag names");
    let model_flags = value::js_string_array(value::before(
        &contract.exact_value,
        "; TRANSPORT_CAPABILITY_FLAGS",
    ))
    .expect("a bracketed MODEL_CAPABILITY_FLAGS array");
    let transport_flags = value::js_string_array(value::after(
        &contract.exact_value,
        "TRANSPORT_CAPABILITY_FLAGS = ",
    ))
    .expect("a bracketed TRANSPORT_CAPABILITY_FLAGS array");
    assert_eq!(model_flags.len(), 7, "{model_flags:?}");
    assert_eq!(transport_flags.len(), 5, "{transport_flags:?}");
    assert!(
        contract.exact_value.contains("no extra non-boolean value"),
        "the catalogue no longer documents the closed-boolean-set requirement"
    );

    let profile = via_catalog::local_realtime_model_profile("conformance-test-model");
    let mut registry = via_realtime::RealtimeProviderRegistry::with_default("conformance");
    registry
        .register(std::sync::Arc::new(
            via_realtime::testing::TestProvider::new("conformance")
                .with_model_profile(Some(profile)),
        ))
        .map(|_| ())
        .expect("a valid provider registers");
    let active = registry
        .describe_active_realtime(None)
        .expect("a registered default provider always describes");

    let model_json = serde_json::to_value(
        active
            .model_capabilities
            .expect("the profile carries model capabilities"),
    )
    .expect("ModelCapabilities serializes");
    let model_object = model_json.as_object().expect("an object");
    assert_eq!(
        model_object.keys().map(String::as_str).collect::<Vec<_>>(),
        model_flags,
        "the published modelCapabilities keys are not the catalogued flag set"
    );
    assert!(
        model_object.values().all(Value::is_boolean),
        "every modelCapabilities flag must be a boolean: {model_json}"
    );

    let transport_json = serde_json::to_value(
        active
            .transport_capabilities
            .expect("the profile carries transport capabilities"),
    )
    .expect("TransportCapabilities serializes");
    let transport_object = transport_json.as_object().expect("an object");
    assert_eq!(
        transport_object
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        transport_flags,
        "the published transportCapabilities keys are not the catalogued flag set"
    );
    assert!(
        transport_object.values().all(Value::is_boolean),
        "every transportCapabilities flag must be a boolean: {transport_json}"
    );
}

// ── the input-part `_meta` reference key ────────────────────────────────────

/// `json-field` / *input part `_meta` reference key*.
///
/// > `'qwen-audio-agent/inputRef'` with values like `'input_1'`; a
/// > client-supplied `_meta` is STRIPPED during `normalizeInputParts`.
///
/// Both halves live in `via-voice::input`, not in `via-realtime` — the key
/// constant and the reference round trip are `via-voice`'s own, and neither
/// one has a test that exercises the *stripping* half of this exact contract.
/// This constructs a part the way a client's JSON would carry a forged
/// reference into someone else's conversation, and proves
/// `normalize_file_part` throws it away rather than trusting it.
#[test]
fn a_client_supplied_meta_is_stripped_and_the_key_is_the_catalogued_rebrand() {
    let contract = expect_contract("json-field", "input part _meta reference key");
    let quoted = value::quoted_literals(&contract.exact_value);
    assert_eq!(quoted.len(), 2, "{quoted:?}");
    let upstream_key = quoted[0];
    let example_reference = quoted[1];
    assert!(
        contract.exact_value.contains("STRIPPED"),
        "the catalogue no longer documents the stripping behaviour"
    );

    assert_eq!(
        via_voice::input::INPUT_REF_META_KEY,
        upstream_key.replace("qwen-audio-agent/", "via/"),
        "docs/rebrand.md renames the _meta namespace itself, not just its value"
    );

    // A legitimate, server-assigned reference round-trips.
    let referenced =
        via_voice::input::InputPart::file("image/png", "https://example.invalid/one.png")
            .with_reference(example_reference);
    assert_eq!(referenced.reference(), example_reference);

    // A client that sends the same key on the wire, forging a reference into
    // another conversation's asset, has it dropped: `normalize_file_part`
    // rebuilds the part with a fresh `_meta` unconditionally.
    let mut forged_meta = serde_json::Map::new();
    forged_meta.insert(
        via_voice::input::INPUT_REF_META_KEY.to_owned(),
        serde_json::json!("someone-elses-asset"),
    );
    let forged: via_voice::input::InputPart = serde_json::from_value(serde_json::json!({
        "type": "file",
        "mime": "image/png",
        "url": "https://example.invalid/one.png",
        "_meta": forged_meta,
    }))
    .expect("a well-formed file part deserializes");
    assert_eq!(
        forged.reference(),
        "someone-elses-asset",
        "the forged part carries the reference going in"
    );

    let normalized = via_voice::input::normalize_file_part(&forged, Locale::Zh)
        .expect("a well-formed https url normalizes");
    assert_eq!(
        normalized.reference(),
        "",
        "a client-supplied _meta must not survive normalizeInputParts"
    );
}

// ── the `spawn_thinking` schema ──────────────────────────────────────────────

/// `tool-description` / *spawn_thinking parameter schema*.
///
/// > `required: ['objective']; properties.input_refs = {type:'array',
/// > maxItems:8}`.
///
/// Owned end to end by `via-voice::tools::catalog::spawn_thinking_tool`; the
/// crate's own tests check `maxItems` and the `required` list separately but
/// neither compares the catalogued literal, so this reads both straight out of
/// `docs/reference/contracts.json` and checks the shipped schema against them
/// together.
#[test]
fn the_spawn_thinking_schema_matches_the_catalogued_shape() {
    let contract = expect_contract("tool-description", "spawn_thinking parameter schema");
    let quoted = value::quoted_literals(&contract.exact_value);
    assert_eq!(quoted, ["objective", "array"], "{quoted:?}");
    let max_items: u64 = value::after(&contract.exact_value, "maxItems:")
        .trim_end_matches('}')
        .trim()
        .parse()
        .expect("maxItems is followed by an integer");

    let tool = via_voice::tools::catalog::spawn_thinking_tool(Locale::Zh);
    assert_eq!(
        tool["function"]["name"],
        serde_json::json!(via_voice::tools::catalog::SPAWN_THINKING)
    );
    let parameters = &tool["function"]["parameters"];
    assert_eq!(parameters["required"], serde_json::json!([quoted[0]]));
    assert_eq!(
        parameters["properties"]["input_refs"]["type"],
        serde_json::json!(quoted[1])
    );
    assert_eq!(
        parameters["properties"]["input_refs"]["maxItems"],
        serde_json::json!(max_items)
    );
    assert_eq!(
        u64::try_from(via_voice::tools::catalog::MAX_INPUT_REFS).expect("fits"),
        max_items
    );
}

// ── the frontend tool set ────────────────────────────────────────────────────

/// `tool-name` / *frontend tool set exposed to the realtime model*.
///
/// > `['spawn_thinking', …, 'respond_agent_permission']` plus `'enter_sleep'`
/// > registered only for clients advertising the `'sleeping'` state.
///
/// Owned end to end by `via-voice::tools::catalog`. `via-voice`'s own
/// `every_tool_name_this_crate_declares_is_catalogued` only checks that each
/// declared name is *somewhere* in the catalogue, not that this exact row's
/// eight names are the shipped `ALWAYS_DECLARED` set in this order — so this
/// reads the list out of the catalogue and drives the real conditional
/// wiring, [`via_voice::tools::catalog::frontend_tools`], the way a client
/// that has and has not declared `sleeping` would.
#[test]
fn the_frontend_tool_set_is_the_catalogued_one_and_enter_sleep_is_gated_on_the_sleeping_state() {
    let contract = expect_contract(
        "tool-name",
        "frontend tool set exposed to the realtime model",
    );
    let always = value::js_string_array(value::before(&contract.exact_value, "plus"))
        .expect("a bracketed array of always-declared tool names");
    assert_eq!(always.len(), 8, "{always:?}");
    assert_eq!(always, via_voice::tools::catalog::ALWAYS_DECLARED.to_vec());
    assert!(contract.exact_value.contains("'enter_sleep'"));
    assert!(contract.exact_value.contains("'sleeping'"));
    assert_eq!(via_voice::tools::catalog::ENTER_SLEEP, "enter_sleep");
    assert_eq!(via_voice::tools::catalog::SLEEPING_CLIENT_STATE, "sleeping");

    let plan = via_voice::mode::ModePlan::new(via_protocol::SessionMode::Agent, true);

    let without_sleep = via_voice::tools::catalog::frontend_tools(Locale::Zh, &[], plan);
    assert_eq!(declared_names(&without_sleep), always);

    let with_sleep =
        via_voice::tools::catalog::frontend_tools(Locale::Zh, &["sleeping".to_owned()], plan);
    let mut expected_with_sleep = always;
    expected_with_sleep.push("enter_sleep");
    assert_eq!(declared_names(&with_sleep), expected_with_sleep);

    // An unrelated declared state does not unlock it.
    let other = via_voice::tools::catalog::frontend_tools(Locale::Zh, &["hidden".to_owned()], plan);
    assert_eq!(other.len(), 8);
}

fn declared_names(tools: &[Value]) -> Vec<&str> {
    tools
        .iter()
        .map(|tool| tool["function"]["name"].as_str().unwrap_or_default())
        .collect()
}
