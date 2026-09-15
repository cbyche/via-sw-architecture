//! Every value in this crate that `docs/reference/contracts.json` pins.
//!
//! The catalogue is **parsed, never retyped**: a test that restated the
//! expected string would assert only that this crate agrees with itself.
//! `docs/architecture.md` §14 names `reference/contracts.md` the acceptance
//! criteria; this file is `via-mcp-tools`' slice of it, and it is the slice
//! `via-conformance` files under `Crate::ViaMcpTools`.
//!
//! # Reading a contract that carries an upstream identity
//!
//! Most `exactValue`s here spell the tool names and the MCP server name the
//! way upstream did. `docs/rebrand.md` renames both, so those assertions apply
//! [`rebranded`] to the catalogued text rather than hard-coding the VIA
//! spelling — a change to *either* document then fails this test, which is the
//! point. The one identity the rebrand does **not** touch is the
//! `open-computer-use` package and bin name, which is KEEP, and the tests for
//! it assert the catalogued string unmodified.

use std::sync::OnceLock;

use via_mcp_tools::builtin::{
    COMPUTER_USE_ENV, COMPUTER_USE_SERVER_NAME, ComputerUseLaunch, DISABLED_VALUES,
    ELECTRON_RUN_AS_NODE,
};
use via_mcp_tools::context::{
    DEFAULT_SESSION_LIST_LIMIT, DelegationOutcome, DelegationRecord, DelegationStatus,
    SESSION_LIST_LIMIT_MAX, SESSION_LIST_LIMIT_MIN, SESSION_TITLE_BOUND, STATUS_RESULT_BOUND,
    STATUS_STARTED, SessionStatusResult, SessionSummary,
};
use via_mcp_tools::lifecycle::{
    APP_AGENT_DISCOVERY, APP_AGENT_MARKER, APP_AGENT_POLL, PS_ARGUMENTS, TERMINATION_GRACE,
    TerminationSignal,
};
use via_mcp_tools::protocol::{codes, method_not_allowed_body};
use via_mcp_tools::{
    DEFAULT_SESSION_INSTRUCTIONS, LOOPBACK_HOST, MCP_PATH, METHOD_NOT_ALLOWED_MESSAGE,
    SESSION_TOOL_NAMES, SESSION_TOOL_SERVER, SESSION_TOOL_SERVER_VERSION, STATUS_FAILED,
    SessionTool, error_result, internal_error_body, json_result, tool_definitions,
};

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

/// Every catalogue record filed under `kind`/`name`. Several contracts are
/// catalogued twice — once from the defining module and once from the upstream
/// test that locks them — and both records must agree with the shipped value.
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

/// Apply the identity renames `docs/rebrand.md` mandates to catalogued text.
///
/// Only the product identity moves. The longest name is substituted first so
/// `qwen_audio_agent_sessions_list` does not become `via_sessions_list`
/// twice over, and the KEEP names — the third-party npm package and its bin —
/// are deliberately absent.
fn rebranded(value: &str) -> String {
    value
        .replace("qwen_audio_agent_sessions_", "via_sessions_") // allow-brand
        .replace("qwen_audio_agent_session_", "via_session_") // allow-brand
        .replace("QWEN_AUDIO_AGENT_", "VIA_") // allow-brand
        .replace("qwen_audio_agent", SESSION_TOOL_SERVER) // allow-brand
}

// ---------------------------------------------------------------------------
// The five names, and the one array
// ---------------------------------------------------------------------------

#[test]
fn the_allow_list_is_the_catalogued_list() {
    // `tool-name / ACP Session tools served to the backend agent` is the
    // array itself, in order.
    let catalogued = rebranded(contract(
        "tool-name",
        "ACP Session tools served to the backend agent",
    ));
    let names: Vec<String> = catalogued
        .trim_matches(['[', ']'])
        .split(',')
        .map(|entry| entry.trim().trim_matches('\'').to_owned())
        .collect();
    assert_eq!(names, SESSION_TOOL_NAMES.to_vec());
}

#[test]
fn every_tool_name_is_catalogued_under_its_own_row() {
    for (tool, upstream) in SessionTool::ALL.into_iter().zip([
        "qwen_audio_agent_sessions_list",  // allow-brand: the catalogue key
        "qwen_audio_agent_session_start",  // allow-brand
        "qwen_audio_agent_session_send",   // allow-brand
        "qwen_audio_agent_session_status", // allow-brand
        "qwen_audio_agent_session_cancel", // allow-brand
    ]) {
        // Two records exist for each: the bare name, and the full tool
        // contract. Both must rebrand to the shipped name.
        for record in entries("tool-name", upstream) {
            assert!(
                rebranded(&record.exact_value).contains(tool.name()),
                "{upstream} does not rebrand to {}",
                tool.name(),
            );
        }
        assert_eq!(rebranded(upstream), tool.name());
    }
}

#[test]
fn the_mcp_server_name_and_version_are_catalogued() {
    let value = rebranded(contract(
        "tool-name",
        "via/qwen_audio_agent MCP server name",
    )); // allow-brand
    assert!(
        value.contains(&format!("'{SESSION_TOOL_SERVER}'")),
        "{value}"
    );
    assert!(
        value.contains(&format!("version '{SESSION_TOOL_SERVER_VERSION}'")),
        "{value}"
    );
}

// ---------------------------------------------------------------------------
// Titles, descriptions, annotations, schemas
// ---------------------------------------------------------------------------

/// Each tool's `tool-description` row is the description verbatim.
#[test]
fn every_description_is_the_catalogued_sentence() {
    for (tool, key) in SessionTool::ALL.into_iter().zip([
        "qwen_audio_agent_sessions_list.description", // allow-brand
        "qwen_audio_agent_session_start.description", // allow-brand
        "qwen_audio_agent_session_send.description",  // allow-brand
        "qwen_audio_agent_session_status.description", // allow-brand
        "qwen_audio_agent_session_cancel.description", // allow-brand
    ]) {
        assert_eq!(
            tool.description(),
            contract("tool-description", key),
            "{}",
            tool.name(),
        );
    }
}

/// Each tool's title appears inside its full `tool-name` contract.
#[test]
fn every_title_is_the_catalogued_title() {
    for (tool, key) in SessionTool::ALL.into_iter().zip([
        "qwen_audio_agent_sessions_list",  // allow-brand
        "qwen_audio_agent_session_start",  // allow-brand
        "qwen_audio_agent_session_send",   // allow-brand
        "qwen_audio_agent_session_status", // allow-brand
        "qwen_audio_agent_session_cancel", // allow-brand
    ]) {
        let full = entries("tool-name", key)
            .into_iter()
            .find(|record| record.exact_value.contains("title"))
            .map(|record| record.exact_value.as_str())
            .unwrap_or_default();
        assert!(
            full.contains(&format!("title '{}'", tool.title())),
            "{} title is not catalogued: {full}",
            tool.name(),
        );
    }
}

#[test]
fn the_two_read_only_tools_carry_the_catalogued_annotations() {
    let catalogued = contract("json-field", "qwen_audio_agent_sessions_list.annotations"); // allow-brand
    assert_eq!(catalogued, "{ readOnlyHint: true, openWorldHint: false }");
    for tool in SessionTool::ALL {
        let annotations = tool.annotations();
        assert_eq!(
            annotations.is_some(),
            tool.is_read_only(),
            "{}",
            tool.name(),
        );
        if let Some(annotations) = annotations {
            assert_eq!(annotations["readOnlyHint"], true);
            assert_eq!(annotations["openWorldHint"], false);
            assert_eq!(annotations.as_object().map(serde_json::Map::len), Some(2));
        }
    }
}

#[test]
fn the_input_schemas_are_the_catalogued_schemas() {
    // sessions_list: query?: string, limit?: integer 1..100, required: []
    let value = contract("json-field", "qwen_audio_agent_sessions_list.inputSchema"); // allow-brand
    assert!(value.contains("query:{type:'string'}"), "{value}");
    assert!(
        value.contains(&format!(
            "limit:{{type:'integer', minimum:{SESSION_LIST_LIMIT_MIN}, maximum:{SESSION_LIST_LIMIT_MAX}}}"
        )),
        "{value}",
    );
    assert!(value.contains("required:[]"), "{value}");
    assert_eq!(
        SessionTool::SessionsList.input_schema(),
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" },
                "limit": {
                    "type": "integer",
                    "minimum": SESSION_LIST_LIMIT_MIN,
                    "maximum": SESSION_LIST_LIMIT_MAX,
                },
            },
            "required": [],
        }),
    );

    // start: prompt required minLength 1, title optional.
    let value = contract("json-field", "qwen_audio_agent_session_start.inputSchema"); // allow-brand
    assert!(
        value.contains("prompt:{type:'string', minLength:1}"),
        "{value}"
    );
    assert!(value.contains("title:{type:'string'}"), "{value}");
    assert!(value.contains("required:['prompt']"), "{value}");
    assert_eq!(
        SessionTool::SessionStart.input_schema(),
        serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": { "type": "string", "minLength": 1 },
                "title": { "type": "string" },
            },
            "required": ["prompt"],
        }),
    );

    // send: both required, minLength 1, snake_case session_id.
    let record = entry("json-field", "qwen_audio_agent_session_send.inputSchema"); // allow-brand
    assert!(record.exact_value.contains("both required, minLength 1"));
    assert!(
        record.why.contains("snake_case field name session_id"),
        "the snake_case warning is the contract's own reason",
    );
    assert_eq!(
        SessionTool::SessionSend.input_schema(),
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string", "minLength": 1 },
                "prompt": { "type": "string", "minLength": 1 },
            },
            "required": ["session_id", "prompt"],
        }),
    );

    // status and cancel: both ids optional, nothing required.
    for (tool, key) in [
        (
            SessionTool::SessionStatus,
            "qwen_audio_agent_session_status.inputSchema", // allow-brand
        ),
        (
            SessionTool::SessionCancel,
            "qwen_audio_agent_session_cancel.inputSchema", // allow-brand
        ),
    ] {
        let value = contract("json-field", key);
        assert!(value.contains("both optional"), "{value}");
        assert_eq!(
            tool.input_schema(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "delegation_id": { "type": "string" },
                    "session_id": { "type": "string" },
                },
                "required": [],
            }),
        );
    }
}

#[test]
fn the_property_name_sets_are_the_ones_the_upstream_test_deep_equals() {
    // `json-field / ACP Session tool input schemas` is the assertion in
    // server/test/acp-session-tools.test.mjs:54-62, sorted.
    let catalogued = rebranded(contract("json-field", "ACP Session tool input schemas"));
    for tool in [
        SessionTool::SessionsList,
        SessionTool::SessionStart,
        SessionTool::SessionSend,
    ] {
        let mut properties: Vec<String> = tool.input_schema()["properties"]
            .as_object()
            .expect("an object")
            .keys()
            .cloned()
            .collect();
        properties.sort();
        let rendered = format!(
            "{}: [{}]",
            tool.name(),
            properties
                .iter()
                .map(|name| format!("'{name}'"))
                .collect::<Vec<_>>()
                .join(","),
        );
        assert!(
            catalogued.contains(&rendered),
            "{rendered} is not in the catalogued sets\n  {catalogued}",
        );
    }
}

#[test]
fn the_default_session_instructions_are_the_catalogued_prompt() {
    // Both rows carry the same sentence; upstream joins the array with single
    // spaces, and one row says so in as many words.
    for name in [
        "default sessionInstructions (backends with sessionMcp)",
        "default sessionInstructions (MCP-tool backends)",
    ] {
        let catalogued = rebranded(contract("prompt-text", name));
        assert!(
            catalogued.contains(DEFAULT_SESSION_INSTRUCTIONS),
            "{name} does not match the shipped instructions\n  catalogued: {catalogued}\n  shipped:    {DEFAULT_SESSION_INSTRUCTIONS}",
        );
    }
    // And it names two of the tools, which is why it lives beside them.
    for tool in [SessionTool::SessionStart, SessionTool::SessionSend] {
        let suffix = tool
            .name()
            .strip_prefix("via_")
            .expect("every tool name is prefixed");
        assert!(
            DEFAULT_SESSION_INSTRUCTIONS.contains(suffix),
            "{suffix} is not named in the instructions",
        );
    }
    // The delegation contract itself: asynchronous, `status=started`, stop.
    assert!(DEFAULT_SESSION_INSTRUCTIONS.contains("asynchronous"));
    assert!(DEFAULT_SESSION_INSTRUCTIONS.contains(&format!("status={STATUS_STARTED}")));
    assert!(DEFAULT_SESSION_INSTRUCTIONS.contains("Never poll it in the same turn."));
}

// ---------------------------------------------------------------------------
// Result envelopes
// ---------------------------------------------------------------------------

#[test]
fn the_success_envelope_is_the_catalogued_shape() {
    let catalogued = contract("json-field", "session tool success envelope");
    assert_eq!(
        catalogued,
        r#"{"content":[{"type":"text","text":"<JSON.stringify(value)>"}]}"#,
    );
    let envelope = json_result(&serde_json::json!({ "a": 1 }));
    assert_eq!(
        serde_json::to_string(&envelope).expect("json"),
        r#"{"content":[{"type":"text","text":"{\"a\":1}"}]}"#,
    );
}

#[test]
fn the_error_envelope_is_the_catalogued_shape() {
    for record in entries("error-code", "session tool error envelope")
        .into_iter()
        .chain(entries("json-field", "MCP tool error envelope"))
    {
        assert!(
            record.exact_value.contains(r#"\"status\":\"failed\""#),
            "{}",
            record.exact_value,
        );
        assert!(record.exact_value.contains(r#""isError":true"#));
    }
    assert_eq!(STATUS_FAILED, "failed");
    assert_eq!(
        serde_json::to_string(&error_result("boom")).expect("json"),
        r#"{"content":[{"type":"text","text":"{\"status\":\"failed\",\"error\":\"boom\"}"}],"isError":true}"#,
    );
}

#[test]
fn the_result_envelope_is_compact_json_because_the_model_parses_it() {
    let record = entry("json-field", "MCP tool error envelope");
    assert!(
        record.why.contains("compact (no-indent) JSON"),
        "the catalogue states the reason: {}",
        record.why,
    );
    let envelope = json_result(&serde_json::json!({ "a": [1, 2] }));
    let text = via_mcp_tools::envelope_text(&envelope).expect("a text block");
    assert!(!text.contains('\n') && !text.contains(": "), "{text}");
}

#[test]
fn the_started_result_is_the_catalogued_shape() {
    let record = entries("tool-name", "qwen_audio_agent_session_start") // allow-brand
        .into_iter()
        .find(|record| record.exact_value.contains("Result JSON"))
        .expect("the full tool contract");
    assert!(
        record.why.contains("'started'"),
        "the literal is the contract's own reason: {}",
        record.why,
    );
    assert_eq!(STATUS_STARTED, "started");
    let started = DelegationRecord::new("d", "s", "t", "/p").started();
    let json = serde_json::to_value(&started).expect("json");
    for field in [
        "status",
        "delegation_id",
        "session_id",
        "title",
        "directory",
    ] {
        assert!(
            record.exact_value.contains(&format!("\"{field}\"")),
            "{field} is not in {}",
            record.exact_value,
        );
        assert!(json.get(field).is_some(), "{field} is missing from {json}");
    }
    assert_eq!(json["status"], STATUS_STARTED);
}

#[test]
fn session_send_answers_in_the_same_shape_as_session_start() {
    let record = entries("tool-name", "qwen_audio_agent_session_send") // allow-brand
        .into_iter()
        .find(|record| record.exact_value.contains("Result JSON"))
        .expect("the full tool contract");
    assert!(
        record
            .exact_value
            .contains("identical shape to session_start"),
        "{}",
        record.exact_value,
    );
}

#[test]
fn the_status_result_carries_result_only_when_completed() {
    let record = entries("tool-name", "qwen_audio_agent_session_status") // allow-brand
        .into_iter()
        .find(|record| record.exact_value.contains("Result JSON"))
        .expect("the full tool contract");
    assert!(
        record.exact_value.contains(&format!(
            "first {STATUS_RESULT_BOUND} chars> only when completed"
        )),
        "{}",
        record.exact_value,
    );
    assert!(record.exact_value.contains("only when failed"));
    assert!(record.why.contains(&format!("{STATUS_RESULT_BOUND}-char")));

    let base = DelegationRecord::new("d", "s", "t", "/p");
    let completed = serde_json::to_value(SessionStatusResult::known(
        base.clone(),
        DelegationOutcome::completed("done"),
    ))
    .expect("json");
    assert_eq!(completed["result"], "done");
    assert!(completed.get("error").is_none());

    let failed = serde_json::to_value(SessionStatusResult::known(
        base,
        DelegationOutcome::failed("broke"),
    ))
    .expect("json");
    assert_eq!(failed["error"], "broke");
    assert!(failed.get("result").is_none());
}

#[test]
fn the_status_vocabulary_is_the_catalogued_one() {
    let catalogued = contract("json-field", "session_status result");
    for status in [
        DelegationStatus::Running,
        DelegationStatus::Completed,
        DelegationStatus::Failed,
        DelegationStatus::Cancelled,
    ] {
        assert!(
            catalogued.contains(&format!("'{}'", status.as_str())),
            "{} is not catalogued: {catalogued}",
            status.as_str(),
        );
    }
    assert!(catalogued.contains("'not_found'"));
    assert_eq!(SessionStatusResult::NotFound.status_str(), "not_found");
    // `cancelling` is VIA's addition; the catalogue does not have it, and
    // `docs/deviations/phase-2.md` records why via-downstream introduced it.
    assert!(!catalogued.contains("'cancelling'"), "{catalogued}");
    assert_eq!(DelegationStatus::Cancelling.as_str(), "cancelling");
}

#[test]
fn not_found_is_returned_bare() {
    let catalogued = contract("json-field", "session_status result");
    assert!(
        catalogued.contains("'not_found' is returned bare")
            || entry("json-field", "session_status result")
                .why
                .contains("returned bare"),
        "{catalogued}",
    );
    assert_eq!(
        serde_json::to_string(&SessionStatusResult::NotFound).expect("json"),
        r#"{"status":"not_found"}"#,
    );
}

#[test]
fn the_session_summary_fields_and_title_bound_are_catalogued() {
    let record = entries("tool-name", "qwen_audio_agent_sessions_list") // allow-brand
        .into_iter()
        .find(|record| record.exact_value.contains("Result JSON"))
        .expect("the full tool contract");
    for field in ["session_id", "title", "directory", "updated_at"] {
        assert!(
            record.exact_value.contains(&format!("\"{field}\"")),
            "{field} missing from {}",
            record.exact_value,
        );
    }
    assert!(
        record
            .exact_value
            .contains(&format!("title bounded to {SESSION_TITLE_BOUND} chars")),
        "{}",
        record.exact_value,
    );
    let summary = SessionSummary::new("s", &"t".repeat(500), "/p", "u");
    assert_eq!(summary.title.chars().count(), SESSION_TITLE_BOUND);
    let json = serde_json::to_value(&summary).expect("json");
    assert_eq!(
        json.as_object().expect("object").keys().collect::<Vec<_>>(),
        ["session_id", "title", "directory", "updated_at"],
    );
}

#[test]
fn the_default_page_size_and_the_hard_cap_are_catalogued() {
    let limits = contract("default-value", "timeouts and limits");
    assert!(
        limits.contains(&format!(
            "listProjectSessions default limit {DEFAULT_SESSION_LIST_LIMIT}"
        )),
        "{limits}",
    );
    assert!(
        limits.contains(&format!(
            "session/list hard page cap {SESSION_LIST_LIMIT_MAX}"
        )),
        "{limits}",
    );
    assert!(
        limits.contains(&format!(
            "statusForDelegation result cap {STATUS_RESULT_BOUND} chars"
        )),
        "{limits}",
    );
    assert!(
        limits.contains(&format!(
            "AcpSessionToolServer host '{LOOPBACK_HOST}', port 0"
        )),
        "{limits}",
    );
}

// ---------------------------------------------------------------------------
// The transport
// ---------------------------------------------------------------------------

#[test]
fn the_two_error_bodies_are_the_catalogued_bytes() {
    for key in [
        "POST /mcp (session tool server)",
        "MCP session tool endpoint",
    ] {
        let value = contract("http-route", key);
        assert!(value.contains("-32000"), "{value}");
        assert!(value.contains("-32603"), "{value}");
        assert!(value.contains(METHOD_NOT_ALLOWED_MESSAGE), "{value}");
    }
    assert_eq!(codes::METHOD_NOT_ALLOWED, -32000);
    assert_eq!(codes::INTERNAL_ERROR, -32603);
    assert_eq!(
        serde_json::to_string(&method_not_allowed_body()).expect("json"),
        r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32000,"message":"Method not allowed."}}"#,
    );
    assert_eq!(
        serde_json::to_string(&internal_error_body("<error.message>")).expect("json"),
        r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"<error.message>"}}"#,
    );
}

#[test]
fn the_trailing_period_is_stated_to_be_wire_visible() {
    let record = entry("http-route", "MCP session tool endpoint");
    assert!(
        record
            .why
            .contains("the trailing period in 'Method not allowed.'"),
        "{}",
        record.why,
    );
    assert!(METHOD_NOT_ALLOWED_MESSAGE.ends_with('.'));
}

#[test]
fn the_bind_address_and_path_are_catalogued() {
    let value = contract("http-route", "POST /mcp (session tool server)");
    assert!(
        value.contains(&format!("Bind {LOOPBACK_HOST}:0")),
        "{value}"
    );
    assert!(value.contains(&format!("Path != '{MCP_PATH}'")), "{value}");
    assert!(value.contains("404 with empty body"), "{value}");
    assert!(value.contains("stateless transport"), "{value}");
}

#[test]
fn four_oh_four_rather_than_four_oh_one_is_the_stated_intent() {
    let record = entry("http-route", "Session MCP endpoint");
    assert!(
        record
            .why
            .contains("404 (not 401) is deliberate: an unauthenticated caller learns nothing"),
        "{}",
        record.why,
    );
    assert!(
        record
            .exact_value
            .contains("an unauthenticated JSON-RPC initialize returns HTTP 404"),
        "{}",
        record.exact_value,
    );
}

#[test]
fn the_descriptor_is_the_catalogued_shape() {
    for key in [
        "session tool MCP descriptor",
        "MCP server descriptor handed to ACP",
    ] {
        let value = rebranded(contract("json-field", key));
        assert!(
            value.contains("'http'") || value.contains("\"http\""),
            "{value}"
        );
        assert!(
            value.contains(&format!("'{SESSION_TOOL_SERVER}'"))
                || value.contains(&format!("\"{SESSION_TOOL_SERVER}\"")),
            "{value}",
        );
        assert!(value.contains(MCP_PATH), "{value}");
        assert!(value.contains("Authorization"), "{value}");
        assert!(value.contains("Bearer "), "{value}");
    }
    let record = entry("json-field", "MCP server descriptor handed to ACP");
    assert!(
        record
            .exact_value
            .contains("headers is an ARRAY of {name,value} objects, not a map"),
        "{}",
        record.exact_value,
    );
}

// ---------------------------------------------------------------------------
// The builtin computer-use server
// ---------------------------------------------------------------------------

#[test]
fn the_toggle_name_and_semantics_are_catalogued() {
    let record = entry("env-var", "QWEN_AUDIO_AGENT_COMPUTER_USE"); // allow-brand: the catalogue key
    assert_eq!(rebranded(&record.name), COMPUTER_USE_ENV);
    for value in DISABLED_VALUES {
        assert!(
            record.exact_value.contains(&format!("'{value}'")),
            "{value} is not catalogued: {}",
            record.exact_value,
        );
    }
    assert!(
        record.exact_value.contains("empty or unset means ENABLED"),
        "{}",
        record.exact_value,
    );
}

#[test]
fn the_builtin_descriptor_is_the_catalogued_shape() {
    for key in [
        "builtin MCP descriptor (open-computer-use)",
        "builtin stdio MCP descriptor (open-computer-use)",
    ] {
        let record = entry("json-field", key);
        let value = record.exact_value.as_str();
        // KEEP: not rebranded, on purpose.
        assert!(value.contains(COMPUTER_USE_SERVER_NAME), "{value}");
        assert!(value.contains(ELECTRON_RUN_AS_NODE), "{value}");
        assert!(
            value.contains("'mcp'") || value.contains("\"mcp\""),
            "{value}"
        );
        // One row states the array encoding in the value, the other in the
        // reason; both are the same warning and both must be honoured.
        let stated = format!("{value}\n{}", record.why);
        assert!(
            stated.contains("array of {name,value}") || stated.contains("ARRAY of {name,value}"),
            "{stated}",
        );
    }

    // Upstream's exact descriptor, reproduced by the Node-package locator.
    let json = serde_json::to_value(
        ComputerUseLaunch::node_script("/opt/node", std::path::Path::new("/pkg/bin.js"))
            .descriptor(),
    )
    .expect("json");
    assert_eq!(json["name"], COMPUTER_USE_SERVER_NAME);
    assert_eq!(json["args"], serde_json::json!(["/pkg/bin.js", "mcp"]));
    assert_eq!(
        json["env"],
        serde_json::json!([{ "name": ELECTRON_RUN_AS_NODE, "value": "1" }]),
    );
}

#[test]
fn the_session_mcp_suppression_is_catalogued() {
    let value = contract(
        "json-field",
        "builtin stdio MCP descriptor (open-computer-use)",
    );
    assert!(
        value.contains("Suppressed entirely when profile.sessionMcp === false"),
        "{value}",
    );
}

#[test]
fn the_lifecycle_timings_are_catalogued() {
    let value = contract("default-value", "timing constants");
    assert!(
        value.contains(&format!(
            "builtin MCP APP_AGENT_DISCOVERY_MS={}",
            APP_AGENT_DISCOVERY.as_millis()
        )),
        "{value}",
    );
    assert!(
        value.contains(&format!("APP_AGENT_POLL_MS={}", APP_AGENT_POLL.as_millis())),
        "{value}",
    );
    assert!(
        value.contains(&format!(
            "{}->{} grace {}ms",
            TerminationSignal::Term,
            TerminationSignal::Kill,
            TERMINATION_GRACE.as_millis(),
        )),
        "{value}",
    );
}

// ---------------------------------------------------------------------------
// Cross-cutting
// ---------------------------------------------------------------------------

#[test]
fn the_permission_broker_carve_out_is_the_contracts_own_reason() {
    let record = entries("tool-name", "qwen_audio_agent_sessions_list") // allow-brand
        .into_iter()
        .find(|record| record.why.contains("PermissionBroker"))
        .expect("the carve-out is catalogued");
    for shape in ["exact", "endsWith '__<name>'", "startsWith '<name> ('"] {
        assert!(
            record.why.contains(shape),
            "{shape} missing: {}",
            record.why
        );
    }
    assert!(
        record
            .why
            .contains("any rename must be applied in both places atomically"),
        "{}",
        record.why,
    );

    // And the shipped matcher answers all three.
    for tool in SessionTool::ALL {
        assert!(via_mcp_tools::is_session_tool(tool.name()));
        assert!(via_mcp_tools::is_session_tool(&format!(
            "mcp__{SESSION_TOOL_SERVER}__{}",
            tool.name()
        )));
        assert!(via_mcp_tools::is_session_tool(&format!(
            "{} (arguments)",
            tool.name()
        )));
    }
}

#[test]
fn every_served_tool_appears_in_the_catalogue() {
    for definition in tool_definitions() {
        let name = definition["name"].as_str().expect("a name");
        let found = contracts().iter().any(|record| {
            rebranded(&record.name) == name || rebranded(&record.exact_value) == name
        });
        assert!(found, "{name} is served but not catalogued");
    }
}

#[test]
fn the_ps_invocation_is_the_catalogued_one() {
    // Not a `contracts.json` row of its own; the marker is, through the
    // upstream test fixture the catalogue quotes.
    assert_eq!(PS_ARGUMENTS, ["-axo", "pid=,command="]);
    assert_eq!(APP_AGENT_MARKER, "__open-computer-use-app-agent");
}

#[test]
fn the_rebrand_is_applied_not_skipped() {
    // A shipped name that still spelled the upstream identity would fail
    // here, and so would one that was renamed to something other than
    // `docs/rebrand.md`'s spelling.
    for name in SESSION_TOOL_NAMES {
        assert!(name.starts_with("via_"), "{name}");
    }
    assert_eq!(SESSION_TOOL_SERVER, "via");
    // The catalogue's own names, rebranded, are exactly the shipped array.
    let catalogued: Vec<String> = contracts()
        .iter()
        .filter(|record| record.kind == "tool-description")
        .map(|record| rebranded(record.name.trim_end_matches(".description")))
        .filter(|name| name.starts_with("via_"))
        .collect();
    let mut catalogued: Vec<String> = catalogued;
    catalogued.sort();
    catalogued.dedup();
    let mut shipped: Vec<String> = SESSION_TOOL_NAMES.iter().map(|s| (*s).to_owned()).collect();
    shipped.sort();
    assert_eq!(catalogued, shipped);
}
