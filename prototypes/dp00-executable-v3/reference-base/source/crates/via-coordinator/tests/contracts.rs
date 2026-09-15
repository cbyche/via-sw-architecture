//! Every value in this crate that `docs/reference/contracts.json` pins.
//!
//! The catalogue is **parsed, never retyped**: a test that restated the
//! expected string would assert only that this crate agrees with itself.
//! `docs/architecture.md` §14 names `reference/contracts.md` the acceptance
//! criteria; this file is `via-coordinator`'s slice of it.
//!
//! # Reading a contract that carries an upstream identity
//!
//! Most model-visible values here are XML-ish tags and one prompt paragraph
//! that spell the product name the way upstream did. `docs/rebrand.md` renames
//! them, so those assertions apply [`rebranded`] to the catalogued text rather
//! than hard-coding the VIA spelling — a change to *either* document then fails
//! this test, which is the point.

use std::sync::OnceLock;

use via_coordinator::decision::INLINE_TITLE_BOUND;
use via_coordinator::delegation::{
    DELEGATION_ID_INFIX, PERMISSION_SCOPE_PREFIX, new_delegation_id, new_permission_scope_id,
};
use via_coordinator::envelope::{
    COMPLETION_AUTOMATIC, DELIVERY_STATUS_MEANINGFUL_ONLY, DELIVERY_STATUS_SILENT,
    MAX_CONTEXT_MESSAGES, MAX_MEMORY_RECORDS, MAX_WORK_LINES, OWNER_SCOPE,
    TRUSTED_BACKEND_EVENT_CONTENT_BOUND, TRUSTED_BACKEND_EVENT_KIND, WORKING_DIRECTORY_SCOPE,
};
use via_coordinator::instructions::{
    BACKEND_AGENT_INSTRUCTION_LINE_COUNT, backend_agent_instruction_lines,
};
use via_coordinator::native::{
    NATIVE_DELEGATION_TOOL_PATTERN, RUN_ID_KEY, SESSION_ID_KEYS, TITLE_BOUND, completed_status,
    is_delegation_tool,
};
use via_coordinator::permission::{
    CATEGORY_BOUND, DECISION_ALWAYS, DECISION_REJECT, PERMISSION_ID_PREFIX, RESOLVED_LIMIT,
    SUMMARY_DETAIL_BOUND, SUMMARY_SEPARATOR, UNKNOWN_CATEGORY, category_of, summary_of,
};
use via_coordinator::profile::DEFAULT_TIMEOUT_MS;
use via_coordinator::prompts::{
    CONTROL_CANCEL_OPEN_TAG, CONTROL_CLOSE_TAG, CONTROL_STATUS_OPEN_TAG,
    DELEGATION_RESULT_CLOSE_TAG, DELEGATION_RESULT_OPEN_TAG, DelegationResult,
    MAX_DELEGATION_RESULT_CHARS, PROTOCOL_RETRY_CLOSE_TAG, PROTOCOL_RETRY_OPEN_TAG,
    RECONCILIATION_CLOSE_TAG, RECONCILIATION_OPEN_TAG, cancel_control_prompt,
    delegation_result_prompt, protocol_retry_prompt, reconciliation_prompt, status_control_prompt,
};
use via_coordinator::runtime::{BACKEND_REF_ROLE, PROTOCOL_RETRY_ATTEMPTS};
use via_coordinator::{
    BACKEND_AGENT_INSTRUCTIONS, BACKEND_INSTRUCTIONS_CLOSE_TAG, BACKEND_INSTRUCTIONS_OPEN_TAG,
    COORDINATION_PROTOCOL, CoordinationRequest, DecisionMode, DecisionState, PermissionResponse,
    coordination_envelope, coordinator_decision_schema, coordinator_instructions,
    expected_json_example, parse_coordinator_decision,
};
use via_i18n::Locale;

/// One catalogued contract.
#[derive(Debug, serde::Deserialize)]
struct Contract {
    kind: String,
    name: String,
    #[serde(rename = "exactValue")]
    exact_value: String,
    /// Why the value is a contract. Several of these carry the *reason* the
    /// boundary exists, which is worth asserting: the reason is what a future
    /// change would have to argue with.
    why: String,
}

fn contracts() -> &'static [Contract] {
    static CONTRACTS: OnceLock<Vec<Contract>> = OnceLock::new();
    CONTRACTS.get_or_init(|| {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/reference/contracts.json");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        serde_json::from_str(&text).expect("contracts.json is a JSON array of contracts")
    })
}

/// Every catalogue record filed under `kind`/`name`.
fn entries(kind: &str, name: &str) -> Vec<&'static Contract> {
    let found: Vec<&'static Contract> = contracts()
        .iter()
        .filter(|entry| entry.kind == kind && entry.name == name)
        .collect();
    assert!(!found.is_empty(), "no catalogued contract {kind}/{name}");
    found
}

/// The single record filed under `kind`/`name`.
fn entry(kind: &str, name: &str) -> &'static Contract {
    let found = entries(kind, name);
    assert_eq!(found.len(), 1, "{kind}/{name} is catalogued more than once");
    found[0]
}

/// The `exactValue` of one contract.
fn contract(kind: &str, name: &str) -> &'static str {
    entry(kind, name).exact_value.as_str()
}

/// The record under `kind`/`name` whose value starts with `prefix`.
///
/// Several contracts are catalogued twice — once from the defining module and
/// once from the upstream test or the consuming module that locks them — and
/// the two records are not the same *shape*: one is the literal, one is prose
/// about it. This picks the literal.
fn literal(kind: &str, name: &str, prefix: &str) -> &'static str {
    entries(kind, name)
        .into_iter()
        .find(|record| record.exact_value.starts_with(prefix))
        .map(|record| record.exact_value.as_str())
        .unwrap_or_else(|| panic!("no {kind}/{name} record starts with {prefix:?}"))
}

/// The catalogue escapes newlines as the two characters `\` and `n`.
fn unescaped(value: &str) -> String {
    value.replace("\\n", "\n")
}

/// Apply the identity renames `docs/rebrand.md` mandates to catalogued text.
///
/// Only the product identity moves, longest name first. Every tag this crate
/// emits and the envelope's `protocol` field are RENAME rows; nothing here is
/// KEEP.
fn rebranded(value: &str) -> String {
    value
        .replace("qwen_audio_agent_", "via_") // allow-brand: the catalogue key
        .replace("qwen-audio-agent.coordination.v1", COORDINATION_PROTOCOL) // allow-brand
        .replace("qwen-audio-agent", "VIA") // allow-brand
}

// ---------------------------------------------------------------------------
// The instructions and their wrapper
// ---------------------------------------------------------------------------

#[test]
fn the_backend_instructions_are_the_catalogued_text_rebranded() {
    let catalogued = unescaped(literal(
        "prompt-text",
        "BACKEND_AGENT_INSTRUCTIONS",
        "You are the backend Agent",
    ));
    assert_eq!(rebranded(&catalogued), BACKEND_AGENT_INSTRUCTIONS);
    assert_eq!(
        catalogued.split('\n').count(),
        BACKEND_AGENT_INSTRUCTION_LINE_COUNT,
        "the catalogued value is the authority on the line count",
    );
    assert_eq!(
        backend_agent_instruction_lines().len(),
        BACKEND_AGENT_INSTRUCTION_LINE_COUNT,
    );
}

#[test]
fn the_catalogues_prose_about_the_instructions_still_describes_them() {
    // The second record is prose. It names the first and last lines and two of
    // the three rebranded ones; asserting it keeps the *description* honest as
    // well as the value.
    let prose = entries("prompt-text", "BACKEND_AGENT_INSTRUCTIONS")
        .into_iter()
        .find(|record| !record.exact_value.starts_with("You are the backend Agent"))
        .expect("a prose record");
    let lines = backend_agent_instruction_lines();
    for quoted in [
        "You are the backend Agent for qwen-audio-agent.", // allow-brand
        "Treat the qwen-audio-agent request envelope as the current user request.", // allow-brand
        "Do not modify qwen-audio-agent itself unless explicitly requested.", // allow-brand
        "Do not expose backend routing, protocol fields, Agent IDs, or Session IDs.",
    ] {
        assert!(prose.exact_value.contains(quoted), "{quoted}");
        assert!(
            lines.contains(&rebranded(quoted).as_str()),
            "{quoted} is described but not shipped",
        );
    }
}

#[test]
fn the_wrapper_is_the_catalogued_framing() {
    // `<tag>\n<instructions>\n<session instructions>\n</tag>\n\n<content>`.
    let shape = unescaped(contract("prompt-text", "coordinator instruction wrapper"));
    let rendered = coordinator_instructions("SESSION", "TURN");
    let mut expected = rebranded(&shape);
    expected = expected
        .replace("<BACKEND_AGENT_INSTRUCTIONS>", BACKEND_AGENT_INSTRUCTIONS)
        .replace("<profile.sessionInstructions or default>", "SESSION")
        .replace("<original prompt text>", "TURN");
    assert_eq!(rendered, expected);
    assert!(rendered.starts_with(BACKEND_INSTRUCTIONS_OPEN_TAG));
    assert!(rendered.contains(BACKEND_INSTRUCTIONS_CLOSE_TAG));
}

#[test]
fn the_wrapper_is_applied_to_the_text_block_only() {
    // The catalogued reason: *"applied via transformPromptText so only the text
    // block of a multi-part prompt is wrapped"*.
    let why = &entry("prompt-text", "coordinator instruction wrapper").why;
    assert!(why.contains("transformPromptText"), "{why}");
    assert!(
        why.contains("multi-part"),
        "the reason the wrapper is not applied to an attachment: {why}",
    );
}

// ---------------------------------------------------------------------------
// The envelope
// ---------------------------------------------------------------------------

#[test]
fn every_envelope_literal_is_the_catalogued_one() {
    let shape = rebranded(contract("json-field", "coordination request envelope"));
    for literal in [
        COORDINATION_PROTOCOL,
        OWNER_SCOPE,
        WORKING_DIRECTORY_SCOPE,
        COMPLETION_AUTOMATIC,
        DELIVERY_STATUS_MEANINGFUL_ONLY,
        DELIVERY_STATUS_SILENT,
        TRUSTED_BACKEND_EVENT_KIND,
    ] {
        assert!(shape.contains(literal), "{literal} is not catalogued");
    }
    assert!(shape.contains(&TRUSTED_BACKEND_EVENT_CONTENT_BOUND.to_string()));
}

#[test]
fn the_envelope_renders_the_catalogued_field_order() {
    let shape = contract("json-field", "coordination request envelope");
    let envelope = coordination_envelope(
        &CoordinationRequest::default(),
        chrono::DateTime::UNIX_EPOCH,
    );
    let rendered: Vec<&str> = envelope
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect();
    // The catalogued shape lists the keys in order; walking it forwards must
    // find each rendered key after the previous one. (The catalogue writes some
    // of them as JavaScript shorthand — `request_id,` rather than
    // `request_id: …` — so the needle is the bare name.)
    let mut cursor = 0usize;
    for key in &rendered {
        let found = shape[cursor..]
            .find(key)
            .unwrap_or_else(|| panic!("{key} is not catalogued after position {cursor}"));
        cursor += found + key.len();
    }
    assert_eq!(rendered.len(), 10);
}

#[test]
fn the_expected_json_example_is_the_catalogued_line() {
    let catalogued = contract("prompt-text", "coordinator expected-JSON example line");
    // The `zh` locale is upstream's own text, so its rendering is the
    // byte-for-byte comparison; `en` and `ko` are its authored peers.
    assert_eq!(expected_json_example(Locale::Zh), catalogued);
}

#[test]
fn the_envelope_slices_are_the_catalogued_numbers() {
    // Catalogued in the source lines the envelope contract cites rather than as
    // numbers of their own; asserted here so a change has to argue with a test.
    assert_eq!(MAX_CONTEXT_MESSAGES, 10);
    assert_eq!(MAX_MEMORY_RECORDS, 20);
    assert_eq!(MAX_WORK_LINES, 10);
}

// ---------------------------------------------------------------------------
// The two response shapes
// ---------------------------------------------------------------------------

#[test]
fn the_decision_schema_is_the_catalogued_schema() {
    let catalogued = contract("json-field", "COORDINATOR_DECISION_SCHEMA");
    let schema = coordinator_decision_schema();
    let rendered = schema.to_string();

    // Every property name, every enum value and both `required` lists.
    for fragment in [
        "work_id",
        "state",
        "mode",
        "presentation",
        "delegation_id",
        "target_session_id",
        "speech",
        "inline",
        "title",
        "format",
        "content",
        "markdown",
        "code",
        "link",
        "completed",
        "delegated",
        "respond",
        "delegate",
        "additionalProperties",
        "oneOf",
        "anyOf",
    ] {
        assert!(
            catalogued.contains(fragment),
            "{fragment} is not catalogued"
        );
        assert!(rendered.contains(fragment), "{fragment} is not rendered");
    }
    assert_eq!(
        schema["oneOf"].as_array().map(Vec::len),
        Some(2),
        "one shape per catalogued branch",
    );
    assert_eq!(schema["oneOf"][0]["additionalProperties"], false);
    assert_eq!(schema["oneOf"][1]["additionalProperties"], false);
}

#[test]
fn the_completed_shape_is_what_the_model_is_asked_for() {
    let catalogued = contract(
        "json-field",
        "coordinator 'completed' decision (model output)",
    );
    assert!(catalogued.contains(&format!(
        r#""state":"{}""#,
        DecisionState::Completed.as_str()
    )));
    assert!(catalogued.contains(&format!(r#""mode":"{}""#, DecisionMode::Respond.as_str())));
    assert!(catalogued.contains("additionalProperties:false"));
}

#[test]
fn the_delegated_shape_is_never_a_completion() {
    let record = entry(
        "json-field",
        "coordinator 'delegated' decision (model output)",
    );
    assert!(record.exact_value.contains(&format!(
        r#""state":"{}""#,
        DecisionState::Delegated.as_str()
    )));
    assert!(
        record
            .exact_value
            .contains(&format!(r#""mode":"{}""#, DecisionMode::Delegate.as_str()))
    );
    assert!(
        record.why.contains("NEVER a user-visible completion"),
        "{}",
        record.why,
    );
    assert!(record.why.contains("lock-release signal"), "{}", record.why);
    // The type carries it: a parsed decision has no way to say `delegated`.
    let parsed = parse_coordinator_decision(&record.exact_value, "work-one");
    assert_eq!(parsed.state(), DecisionState::Completed);
    assert_eq!(parsed.mode(), DecisionMode::Respond);
}

#[test]
fn a_double_encoded_decision_is_tolerated_because_the_catalogue_says_so() {
    let record = entry("json-field", "ACP coordinator decision JSON");
    assert!(
        record.exact_value.contains("double-JSON-encoded"),
        "{}",
        record.exact_value,
    );
    let inner = r#"{"work_id":"w","state":"completed","mode":"respond","presentation":{"speech":"done","inline":null}}"#;
    let encoded = serde_json::Value::String(inner.to_owned()).to_string();
    assert_eq!(
        parse_coordinator_decision(&encoded, "w")
            .presentation
            .speech,
        "done",
    );
}

#[test]
fn the_inline_title_bound_is_the_catalogued_one() {
    // Catalogued under `task resultMetadata projection` as 120, and applied a
    // second time by `via-work`. The two must be the same number.
    assert_eq!(
        INLINE_TITLE_BOUND,
        via_work::presentation::INLINE_TITLE_BOUND
    );
    assert_eq!(INLINE_TITLE_BOUND, 120);
}

// ---------------------------------------------------------------------------
// The four blocks
// ---------------------------------------------------------------------------

#[test]
fn the_retry_block_is_the_catalogued_block() {
    let shape = rebranded(&unescaped(contract(
        "prompt-text",
        "coordinator protocol retry block",
    )));
    let rendered = protocol_retry_prompt("work-one", "active", Locale::Zh);
    let expected = shape
        .replace("<coordinationRunId>", "work-one")
        .replace("<state>", "active");
    assert_eq!(rendered, expected);
    assert!(rendered.starts_with(PROTOCOL_RETRY_OPEN_TAG));
    assert!(rendered.ends_with(PROTOCOL_RETRY_CLOSE_TAG));
}

#[test]
fn the_retry_is_sent_at_most_twice() {
    let why = &entry("prompt-text", "coordinator protocol retry block").why;
    let catalogued: usize = why
        .split_whitespace()
        .find_map(|word| word.parse().ok())
        .expect("the catalogued retry count");
    assert_eq!(PROTOCOL_RETRY_ATTEMPTS, catalogued, "{why}");
}

#[test]
fn the_delegation_result_block_is_the_catalogued_block() {
    let record = entry("prompt-text", "delegated result injection block");
    assert!(
        record
            .exact_value
            .contains(&MAX_DELEGATION_RESULT_CHARS.to_string()),
        "{}",
        record.exact_value,
    );
    assert!(record.why.contains("2-space indented"), "{}", record.why,);
    assert!(
        record
            .why
            .contains("request_id, delegation_id, target_session_id, directory, result"),
        "{}",
        record.why,
    );

    let rendered = delegation_result_prompt(
        &DelegationResult {
            delegation_id: "opencode_run_1".to_owned(),
            target_session_id: "project-1".to_owned(),
            directory: "/project".to_owned(),
            content: "built it".to_owned(),
        },
        "work-one",
        Locale::Zh,
    );
    assert!(rendered.starts_with(DELEGATION_RESULT_OPEN_TAG));
    assert!(rendered.contains(DELEGATION_RESULT_CLOSE_TAG));
    let body: Vec<&str> = rendered.split('\n').collect();
    assert_eq!(body[1], "{");
    assert!(body[2].starts_with("  \"request_id\""), "{}", body[2]);

    let tail = unescaped(contract("prompt-text", "delegation result block"));
    for sentence in tail.split('\n').filter(|line| {
        line.starts_with('这')
            || line.starts_with('请')
            || line.starts_with('返')
            || line.starts_with('不')
    }) {
        assert!(rendered.contains(sentence), "{sentence}");
    }
}

#[test]
fn the_reconciliation_block_is_the_catalogued_block() {
    let shape = rebranded(&unescaped(literal(
        "prompt-text",
        "reconciliation block",
        "<qwen_audio_agent_reconciliation>", // allow-brand: the catalogue key
    )));
    let fact = via_work::CancellationFact::delegated_session_cancelled(
        "work-one",
        "opencode_run_1",
        "project-1",
        "2026-08-22T00:00:00.000Z",
    );
    let rendered =
        reconciliation_prompt(std::slice::from_ref(&fact), "the next request", Locale::Zh);
    assert!(rendered.starts_with(RECONCILIATION_OPEN_TAG));
    assert!(rendered.contains(RECONCILIATION_CLOSE_TAG));
    assert!(rendered.contains(&serde_json::to_string(&fact).expect("json")));
    // The catalogued instruction sentence, verbatim.
    let instruction = shape
        .split('\n')
        .find(|line| line.starts_with("以上是"))
        .expect("the catalogued instruction");
    assert!(rendered.contains(instruction), "{rendered}");
    assert!(rendered.ends_with("\n\nthe next request"));

    // The cap is `via-work`'s, and the catalogue says twenty.
    assert!(
        entries("prompt-text", "reconciliation block")
            .iter()
            .any(|entry| entry
                .why
                .contains(&via_work::reconcile::MAX_FACTS_PER_OWNER.to_string())),
    );
}

#[test]
fn the_two_control_turns_are_the_catalogued_turns() {
    let cancel = rebranded(&unescaped(contract("prompt-text", "cancel control turn")));
    let rendered = cancel_control_prompt("opencode_run_1", None, Locale::Zh);
    assert!(rendered.starts_with(CONTROL_CANCEL_OPEN_TAG));
    assert!(rendered.ends_with(CONTROL_CLOSE_TAG));
    assert!(cancel.contains(CONTROL_CANCEL_OPEN_TAG), "{cancel}");
    let tail = cancel
        .split('\n')
        .find(|line| line.starts_with("工具返回后"))
        .expect("the catalogued tail");
    assert!(rendered.contains(tail));

    let status = rebranded(&unescaped(contract("prompt-text", "status control turn")));
    let asked = status_control_prompt("opencode_run_1", "做到哪了", None, Locale::Zh);
    assert!(asked.starts_with(CONTROL_STATUS_OPEN_TAG));
    assert!(asked.ends_with(CONTROL_CLOSE_TAG));
    let tail = status
        .split('\n')
        .find(|line| line.starts_with("只根据工具结果"))
        .expect("the catalogued tail");
    assert!(asked.contains(tail));
    assert!(
        asked.contains("via_session_status"),
        "the default instruction names one of VIA's own tools: {asked}",
    );
}

#[test]
fn the_control_block_tags_carry_their_kind() {
    let both = rebranded(contract("prompt-text", "coordinator control blocks"));
    assert!(both.contains(CONTROL_CANCEL_OPEN_TAG), "{both}");
    assert!(both.contains(CONTROL_STATUS_OPEN_TAG), "{both}");
    assert!(both.contains(CONTROL_CLOSE_TAG), "{both}");
}

// ---------------------------------------------------------------------------
// Permissions
// ---------------------------------------------------------------------------

#[test]
fn the_permission_payload_is_the_catalogued_payload() {
    let shape = contract(
        "ws-event",
        "backend.permission.requested / backend.permission.resolved",
    );
    assert!(shape.contains(PERMISSION_ID_PREFIX), "{shape}");
    assert!(shape.contains(&CATEGORY_BOUND.to_string()));
    assert!(shape.contains(&SUMMARY_DETAIL_BOUND.to_string()));
    assert!(shape.contains(UNKNOWN_CATEGORY));
    assert!(shape.contains(SUMMARY_SEPARATOR), "{shape}");

    let call = serde_json::json!({
        "name": "write",
        "rawInput": { "path": "/tmp/file" },
    });
    assert_eq!(category_of(&call), "write");
    assert_eq!(
        summary_of(&call),
        format!("write{SUMMARY_SEPARATOR}/tmp/file"),
    );
    assert_eq!(category_of(&serde_json::json!({})), UNKNOWN_CATEGORY);

    let id = via_coordinator::permission::new_permission_id();
    assert!(id.starts_with(PERMISSION_ID_PREFIX));
    assert_eq!(id.len(), PERMISSION_ID_PREFIX.len() + 32);
}

#[test]
fn patterns_is_on_the_request_and_absent_from_the_resolution() {
    let record = entry(
        "json-field",
        "backend.permission.requested / .resolved events",
    );
    assert!(
        record
            .exact_value
            .contains("`patterns` is absent from the resolved shape"),
        "{}",
        record.exact_value,
    );
    assert!(
        record.why.contains(&RESOLVED_LIMIT.to_string()),
        "the retention cap is catalogued: {}",
        record.why,
    );
}

#[test]
fn the_two_decision_literals_are_the_catalogued_ones() {
    assert!(
        entries("http-route", "POST /api/permissions/:id")
            .iter()
            .any(|record| record.exact_value.contains(DECISION_ALWAYS)
                && record.exact_value.contains(DECISION_REJECT)),
        "neither catalogued record carries both decision literals",
    );
    assert_eq!(
        PermissionResponse::from_wire(DECISION_ALWAYS),
        Some(PermissionResponse::Always),
    );
    assert_eq!(
        PermissionResponse::from_wire(DECISION_REJECT),
        Some(PermissionResponse::Reject),
    );
    assert_eq!(PermissionResponse::from_wire("approve"), None);
}

#[test]
fn always_maps_to_the_catalogued_option_kind_order() {
    // The mapping itself is `via-acp`'s; what this asserts is that the
    // coordinator's `always` reaches it as an *approval*, which is the half
    // that decides the user's persisted permission state.
    let record = entry("json-field", "ACP requestPermission reply mapping");
    assert!(
        record.exact_value.contains("allow_once"),
        "{}",
        record.exact_value,
    );
    assert!(
        record.why.contains("allow_once BEFORE allow_always"),
        "{}",
        record.why
    );
    assert_ne!(
        via_acp::permission::APPROVE_KIND_ORDER[0],
        via_acp::permission::APPROVE_KIND_ORDER[1],
    );
    assert_eq!(
        PermissionResponse::Always.decision(),
        via_acp::PermissionDecision::Approve,
    );
    assert_eq!(
        PermissionResponse::Reject.decision(),
        via_acp::PermissionDecision::Reject,
    );
}

#[test]
fn the_permission_scope_id_is_the_catalogued_format() {
    let record = entry("default-value", "permission scope id format");
    assert!(
        record.exact_value.contains(PERMISSION_SCOPE_PREFIX),
        "{}",
        record.exact_value,
    );
    assert!(
        record
            .why
            .contains("Bounds permission cancellation to the prompt"),
        "{}",
        record.why,
    );
    let scope = new_permission_scope_id();
    assert!(scope.starts_with(PERMISSION_SCOPE_PREFIX));
    assert_eq!(scope.len(), PERMISSION_SCOPE_PREFIX.len() + 36);
}

// ---------------------------------------------------------------------------
// Delegation
// ---------------------------------------------------------------------------

#[test]
fn the_delegation_id_is_the_catalogued_format() {
    let record = entry("default-value", "delegation id format (MCP-tool path)");
    assert!(
        record.exact_value.contains(DELEGATION_ID_INFIX),
        "{}",
        record.exact_value,
    );
    assert!(
        record.exact_value.contains("WITH dashes"),
        "{}",
        record.exact_value,
    );
    assert!(
        record.exact_value.contains("runId"),
        "the native path's structurally different id: {}",
        record.exact_value,
    );
    let id = new_delegation_id("opencode");
    assert!(id.starts_with("opencode_run_"));
    assert_eq!(id.matches('-').count(), 4);
}

#[test]
fn the_native_detection_predicate_is_the_catalogued_predicate() {
    let record = entry("state-name", "native delegation detection predicate");
    let value = &record.exact_value;
    assert!(value.contains(NATIVE_DELEGATION_TOOL_PATTERN), "{value}");
    assert!(value.contains(completed_status()), "{value}");
    assert!(value.contains(RUN_ID_KEY), "{value}");
    for key in SESSION_ID_KEYS {
        assert!(value.contains(key), "{key} is not catalogued");
    }
    // The catalogued key order, verbatim, including the swap against
    // `via-acp`'s own traversal order.
    let joined = SESSION_ID_KEYS.join(" || ");
    assert!(value.contains(&joined), "expected `{joined}` in {value}");
    assert!(
        record.why.contains("THIRD-PARTY"),
        "the regex is somebody else's contract: {}",
        record.why,
    );
    assert!(is_delegation_tool("sessions_spawn"));
    assert!(is_delegation_tool("sessions_send"));
    assert_eq!(TITLE_BOUND, 160);
}

#[test]
fn the_unwrapping_algorithm_is_catalogued_with_its_iteration_cap() {
    let record = entry("state-name", "parseCoordinatorPayload unwrapping algorithm");
    assert!(
        record.exact_value.contains("at most 3 times"),
        "{}",
        record.exact_value,
    );
    assert_eq!(via_acp::session::PAYLOAD_UNWRAP_DEPTH, 3);
    assert!(
        record.why.contains("Off-by-one"),
        "the reason the cap is a contract: {}",
        record.why,
    );

    // Three unwraps are allowed; a fourth is one too many.
    let mut nested = r#"{"state":"completed"}"#.to_owned();
    for _ in 0..2 {
        nested = serde_json::Value::String(nested).to_string();
    }
    assert!(via_acp::session::parse_coordinator_payload(&nested).is_some());
    let too_deep = serde_json::Value::String(nested).to_string();
    assert!(via_acp::session::parse_coordinator_payload(&too_deep).is_none());
}

#[test]
fn the_two_delegation_events_carry_what_the_catalogue_says() {
    let delegated = entry("json-field", "backend.delegated event");
    assert!(
        delegated.exact_value.contains("presentation"),
        "{}",
        delegated.exact_value,
    );
    assert!(
        delegated.why.contains("RELEASE the scheduler lane"),
        "{}",
        delegated.why,
    );

    let completed = entry("json-field", "backend.delegation.completed event");
    assert!(
        completed.exact_value.contains("no presentation field"),
        "{}",
        completed.exact_value,
    );
    assert!(completed.why.contains("finalizing"), "{}", completed.why,);
}

#[test]
fn the_result_envelope_role_is_the_catalogued_literal() {
    let record = entry("json-field", "resultEnvelope (adapter -> Coordinator)");
    assert!(
        record
            .exact_value
            .contains(&format!(r#""{BACKEND_REF_ROLE}""#)),
        "{}",
        record.exact_value,
    );
    assert!(record.why.contains("fixed literal"), "{}", record.why,);
}

#[test]
fn the_empty_response_recovery_predicate_is_the_catalogued_one() {
    let record = entry("error-code", "empty coordinator response");
    for clause in [
        "!run.receivedUpdate",
        "!run.delegation",
        "run.nativeToolCalls.size === 0",
        "run.toolCalls.size === 0",
    ] {
        assert!(record.exact_value.contains(clause), "{clause}");
    }
    assert!(record.exact_value.contains("502"), "{}", record.exact_value,);
    assert_eq!(via_coordinator::error::EMPTY_RESPONSE_STATUS, 502);
    assert!(
        record.why.contains("retried exactly once"),
        "{}",
        record.why,
    );
}

#[test]
fn owner_scoping_is_reported_as_not_found() {
    let record = entry("error-code", "delegation lookup / lifecycle errors");
    assert!(
        record
            .why
            .contains("reported as not-found, never as forbidden"),
        "{}",
        record.why,
    );
    // Every one of the five sentences, in the locale upstream wrote them in.
    for (error, upstream) in [
        (
            via_coordinator::CoordinatorError::DelegationAlreadyStarted,
            "当前协调轮次已经启动了一个第三层任务",
        ),
        (
            via_coordinator::CoordinatorError::NotCancellable {
                label: "OpenCode".to_owned(),
            },
            "没有找到可取消的 OpenCode 项目任务",
        ),
        (
            via_coordinator::CoordinatorError::DelegationNotFound {
                label: "OpenCode".to_owned(),
            },
            "没有找到对应的 OpenCode 项目任务",
        ),
        (
            via_coordinator::CoordinatorError::NotRecoverable {
                label: "OpenCode".to_owned(),
            },
            "OpenCode 无法恢复这项第三层任务",
        ),
        (
            via_coordinator::CoordinatorError::SessionDirectoryUnknown {
                label: "OpenCode".to_owned(),
            },
            "OpenCode Session 的项目目录未知，请先查询 Session 列表后再继续",
        ),
    ] {
        assert_eq!(error.message(Locale::Zh), upstream);
        let templated = upstream.replace("OpenCode", "${label}");
        assert!(
            record.exact_value.contains(&templated) || record.exact_value.contains(upstream),
            "{templated} is not catalogued",
        );
    }
}

// ---------------------------------------------------------------------------
// The numbers
// ---------------------------------------------------------------------------

#[test]
fn every_limit_is_the_catalogued_number() {
    let limits = contract("default-value", "timeouts and limits");
    for (name, value) in [
        ("MAX_DELEGATION_RESULT_CHARS", MAX_DELEGATION_RESULT_CHARS),
        ("PermissionBroker resolvedLimit", RESOLVED_LIMIT),
        (
            "AcpBackendAdapter.timeoutMs",
            usize::try_from(DEFAULT_TIMEOUT_MS).expect("fits"),
        ),
    ] {
        // The catalogue writes large numbers with an underscore separator, as
        // JavaScript does.
        let plain = value.to_string();
        let grouped = format!(
            "{}_{}",
            &plain[..plain.len() - 3],
            &plain[plain.len() - 3..]
        );
        assert!(
            limits.contains(&plain) || limits.contains(&grouped),
            "{name} = {value} is not catalogued",
        );
    }
    assert_eq!(
        TRUSTED_BACKEND_EVENT_CONTENT_BOUND,
        MAX_DELEGATION_RESULT_CHARS
    );
}

#[test]
fn the_coordinator_session_key_is_the_catalogued_format() {
    let record = entry("state-name", "coordinator session key format");
    assert!(
        record.exact_value.contains("backend"),
        "{}",
        record.exact_value,
    );
    assert!(record.why.contains("THE fixed identity"), "{}", record.why,);
    // Built through `via-downstream`, never rendered here.
    assert_eq!(
        via_downstream::SessionKey::coordinator("opencode", "owner one").as_str(),
        "opencode:owner%20one:backend",
    );
}
