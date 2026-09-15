//! The `zh` column against `docs/reference/contracts.json`.
//!
//! `zh` is not authored — it is upstream qwen-audio-agent's own text, and the
//! catalogue is the acceptance criterion for that claim. Nothing here retypes
//! an expected value: every expectation is parsed out of `contracts.json` at
//! test time, so a contract that is corrected upstream fails the build instead
//! of quietly disagreeing with a constant.
//!
//! Two shapes of assertion:
//!
//! - **exact** — the contract's `exactValue` *is* the message, so the `zh`
//!   template must equal it byte for byte once `{name}` is written back as
//!   upstream's `${name}`.
//! - **contained** — the `exactValue` is a JSON blob, a wrapped prompt block
//!   or a `|`-joined family of messages, so the message must appear inside it
//!   verbatim.
//!
//! A third table covers the values `docs/rebrand.md` renames: those are
//! compared after the documented rename is applied to the *upstream* side, so
//! the test still fails if the Chinese prose around the renamed token drifts.

use std::fs;
use std::sync::OnceLock;

use serde_json::Value;
use via_i18n::{Key, Locale, keys, t};

const CONTRACTS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/reference/contracts.json"
);

/// `(name, exactValue)` for every contract, JS escapes resolved. Read once.
fn catalogue() -> &'static [(String, String)] {
    static CATALOGUE: OnceLock<Vec<(String, String)>> = OnceLock::new();
    CATALOGUE.get_or_init(|| {
        let text =
            fs::read_to_string(CONTRACTS).expect("docs/reference/contracts.json must be readable");
        let all: Value = serde_json::from_str(&text).expect("contracts.json must be valid JSON");
        let entries = all
            .as_array()
            .expect("contracts.json must be an array")
            .clone();
        entries
            .iter()
            .filter_map(|entry| {
                let name = entry.get("name")?.as_str()?;
                let value = entry.get("exactValue")?.as_str()?;
                Some((name.to_owned(), unescape(value)))
            })
            .collect()
    })
}

/// Every `exactValue` catalogued under `name`, JS escapes resolved.
fn contract_values(name: &str) -> Vec<&'static str> {
    let found: Vec<&'static str> = catalogue()
        .iter()
        .filter(|(candidate, _)| candidate == name)
        .map(|(_, value)| value.as_str())
        .collect();
    assert!(
        !found.is_empty(),
        "no contract in contracts.json is named `{name}`"
    );
    found
}

/// The catalogue writes newlines and tabs as the two-character escapes a
/// JavaScript source line would show. Resolve them so a comparison against a
/// real Rust string is comparing the same bytes.
fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// VIA's `{name}` written back as upstream's `${name}`, so a template can be
/// compared against the JavaScript the catalogue quotes.
fn as_upstream_template(template: &str) -> String {
    let mut out = String::with_capacity(template.len() + 8);
    let bytes = template.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'{' if bytes.get(index + 1) == Some(&b'{') => {
                out.push('{');
                index += 2;
            }
            b'}' if bytes.get(index + 1) == Some(&b'}') => {
                out.push('}');
                index += 2;
            }
            b'{' => {
                let rest = &template[index + 1..];
                let len = rest.find('}').expect("terminated placeholder");
                out.push_str("${");
                out.push_str(&rest[..len]);
                out.push('}');
                index += 1 + len + 1;
            }
            _ => {
                let start = index;
                while index < bytes.len() && bytes[index] != b'{' && bytes[index] != b'}' {
                    index += 1;
                }
                out.push_str(&template[start..index]);
            }
        }
    }
    out
}

/// The identity renames `docs/rebrand.md` applies inside otherwise-verbatim
/// Chinese sentences. Applied to the *upstream* value before comparing, which
/// is the same direction `via-conformance` takes.
fn rebranded(upstream: &str) -> String {
    // allow-brand: these are the upstream literals, quoted so the rename can
    // allow-brand: be applied to them and asserted rather than assumed.
    const RENAMES: &[(&str, &str)] = &[
        ("qwen-audio-agent", "VIA"),                     // allow-brand
        ("qwen_audio_agent_sessions_", "via_sessions_"), // allow-brand
        ("qwen_audio_agent_session_", "via_session_"),   // allow-brand
        ("qwen_audio_agent_", "via_"),                   // allow-brand
        ("QWEN_AUDIO_AGENT_", "VIA_"),                   // allow-brand
        ("QWEN_AUDIO_", "VIA_"),                         // allow-brand
        ("qwenaudio", "via"),                            // allow-brand
    ];
    let mut out = upstream.to_owned();
    for (from, to) in RENAMES {
        out = out.replace(from, to);
    }
    out
}

// ── exact ──────────────────────────────────────────────────────────────────

/// Keys whose `zh` value is the whole of a catalogued `exactValue`.
const EXACT: &[(&str, Key)] = &[
    ("memory truncation marker", keys::MEMORY_TRUNCATION_MARKER),
    (
        "coordinator expected-JSON example line",
        keys::COORDINATOR_EXPECTED_JSON_EXAMPLE,
    ),
    (
        "speakResponseInstructions(content)",
        keys::VOICE_INSTRUCTIONS_SPEAK_RESPONSE,
    ),
    (
        "non-dashscope missingConfigurationMessage template",
        keys::REALTIME_MISSING_CONFIGURATION,
    ),
    (
        "resultResponseInstructions",
        keys::VOICE_INSTRUCTIONS_RESULT_RESPONSE,
    ),
    (
        "permissionResponseInstructions",
        keys::VOICE_INSTRUCTIONS_PERMISSION_RESPONSE,
    ),
    (
        "result truncation suffix",
        keys::REALTIME_RESULT_TRUNCATED_SUFFIX,
    ),
    (
        "permission-resolved silent context note",
        keys::REALTIME_PERMISSION_RESOLVED_NOTE,
    ),
    (
        "spawn_thinking.description",
        keys::VOICE_TOOL_SPAWN_THINKING_DESCRIPTION,
    ),
    ("notes.description", keys::VOICE_TOOL_NOTES_DESCRIPTION),
    (
        "respond_agent_permission.description",
        keys::VOICE_TOOL_RESPOND_AGENT_PERMISSION_DESCRIPTION,
    ),
    (
        "enter_sleep.description",
        keys::VOICE_TOOL_ENTER_SLEEP_DESCRIPTION,
    ),
    (
        "schedule_reminder.description",
        keys::VOICE_TOOL_SCHEDULE_REMINDER_DESCRIPTION,
    ),
    (
        "get_current_time.description",
        keys::VOICE_TOOL_GET_CURRENT_TIME_DESCRIPTION,
    ),
    ("memory.description", keys::VOICE_TOOL_MEMORY_DESCRIPTION),
    (
        "cancel_agent_task.description",
        keys::VOICE_TOOL_CANCEL_AGENT_TASK_DESCRIPTION,
    ),
    (
        "get_agent_task_status.description",
        keys::VOICE_TOOL_GET_AGENT_TASK_STATUS_DESCRIPTION,
    ),
];

#[test]
fn every_exact_contract_value_is_reproduced_byte_for_byte() {
    for (contract, key) in EXACT {
        let ours = as_upstream_template(t(Locale::Zh, *key));
        let candidates = contract_values(contract);
        assert!(
            candidates.iter().any(|value| **value == *ours),
            "{key} does not reproduce contract `{contract}`.\n  ours:     {ours:?}\n  \
             catalogue: {candidates:?}"
        );
    }
}

// ── renamed, then exact ────────────────────────────────────────────────────

/// Keys whose `zh` value is a catalogued `exactValue` with one identity token
/// renamed per `docs/rebrand.md`.
const REBRANDED: &[(&str, Key)] = &[
    (
        "restart force-fail error (interactive work)",
        keys::WORK_RESTART_INTERACTIVE_INCOMPLETE,
    ),
    (
        "restart force-fail error (unrecoverable delegated work)",
        keys::WORK_RESTART_DELEGATED_LOST,
    ),
    (
        "dashscope missingConfigurationMessage",
        keys::GATEWAY_MISSING_DASHSCOPE_API_KEY,
    ),
];

#[test]
fn every_rebranded_contract_value_matches_after_the_documented_rename() {
    for (contract, key) in REBRANDED {
        let ours = as_upstream_template(t(Locale::Zh, *key));
        let candidates = contract_values(contract);
        assert!(
            candidates.iter().any(|value| rebranded(value) == ours),
            "{key} does not reproduce contract `{contract}` after the rebrand.\n  \
             ours:      {ours:?}\n  catalogue: {candidates:?}"
        );
        // The rename must be the *only* difference: without it, the comparison
        // fails. A key that landed in this table by accident is caught here.
        assert!(
            !candidates.iter().any(|value| **value == *ours),
            "{key} needs no rebrand — move it to EXACT"
        );
    }
}

// ── contained ──────────────────────────────────────────────────────────────

/// Keys whose `zh` value appears verbatim inside a larger catalogued value —
/// a JSON payload, a wrapped prompt block, or a `|`-joined family.
const CONTAINED: &[(&str, Key)] = &[
    (
        "QWAUDIO_GATEWAY_SETUP_REQUIRED",
        keys::GATEWAY_SETUP_REQUIRED,
    ), // allow-brand
    (
        "QWAUDIO_GATEWAY_ALREADY_RUNNING", // allow-brand
        keys::LOCK_GATEWAY_ALREADY_RUNNING,
    ),
    (
        "QWAUDIO_INPUT_OWNER_REQUIRED", // allow-brand
        keys::GATEWAY_INPUT_SUSPEND_REQUIRES_OWNER,
    ),
    ("GET /api/backend/ui", keys::GATEWAY_BACKEND_HAS_NO_WEB_UI),
    (
        "POST /api/permissions/:id",
        keys::GATEWAY_PERMISSION_REQUEST_UNKNOWN,
    ),
    (
        "POST /api/permissions/:id",
        keys::GATEWAY_PERMISSION_DECISION_INVALID,
    ),
    (
        "not-configured agent responses",
        keys::GATEWAY_FRONTEND_ONLY_LABEL,
    ),
    ("invalid_time", keys::VOICE_ERROR_INVALID_TIME),
    ("stale call output", keys::VOICE_RESULT_SUPERSEDED),
    (
        "spawn_thinking duplicate output",
        keys::VOICE_RESULT_DUPLICATE,
    ),
    (
        "permission_decision_required",
        keys::VOICE_ERROR_PERMISSION_DECISION_REQUIRED,
    ),
    (
        "backend_unavailable (not configured)",
        keys::VOICE_ERROR_BACKEND_NOT_CONFIGURED,
    ),
    (
        "backend_unavailable (disconnected)",
        keys::VOICE_ERROR_BACKEND_DISCONNECTED,
    ),
    (
        "cancel_agent_task outputs",
        keys::VOICE_RESULT_CANCEL_NOT_FOUND,
    ),
    (
        "cancel_agent_task outputs",
        keys::VOICE_RESULT_CANCEL_NOT_ACTIVE,
    ),
    ("cancel_agent_task outputs", keys::VOICE_RESULT_CANCELLED),
    (
        "get_agent_task_status delegated-query output",
        keys::VOICE_RESULT_QUERYING,
    ),
    (
        "get_agent_task_status delegated-query output",
        keys::VOICE_RESULT_QUERY_ALREADY_IN_FLIGHT,
    ),
    ("notes tool failure codes", keys::NOTES_ERROR_UNAVAILABLE),
    ("notes tool failure codes", keys::NOTES_ERROR_INVALID_ACTION),
    ("notes tool failure codes", keys::NOTES_ERROR_MISSING_TARGET),
    ("notes tool failure codes", keys::NOTES_ERROR_MISSING_ITEMS),
    ("notes tool failure codes", keys::NOTES_ERROR_SENSITIVE),
    ("notes tool failure codes", keys::NOTES_ERROR_WRITE_FAILED),
    ("memory tool failure codes", keys::MEMORY_ERROR_UNAVAILABLE),
    (
        "memory tool failure codes",
        keys::MEMORY_ERROR_INVALID_ACTION,
    ),
    (
        "memory tool failure codes",
        keys::MEMORY_ERROR_INVALID_DOCUMENT_READ,
    ),
    (
        "memory tool failure codes",
        keys::MEMORY_ERROR_INVALID_DOCUMENT_WRITE,
    ),
    (
        "memory tool failure codes",
        keys::MEMORY_ERROR_APPEND_NEEDS_CONTENT,
    ),
    (
        "memory tool failure codes",
        keys::MEMORY_ERROR_REPLACE_NEEDS_TEXTS,
    ),
    ("memory tool failure codes", keys::MEMORY_ERROR_SENSITIVE),
    (
        "memory tool failure codes",
        keys::MEMORY_ERROR_STALE_DOCUMENT,
    ),
    ("memory tool failure codes", keys::MEMORY_ERROR_WRITE_FAILED),
    (
        "MarkdownContextStore edit error codes",
        keys::MEMORY_STALE_DOCUMENT_CODE,
    ),
    (
        "MarkdownContextStore edit error codes",
        keys::MEMORY_INVALID_EDIT_CODE,
    ),
    (
        "MarkdownContextStore edit error codes",
        keys::MEMORY_AMBIGUOUS_EDIT_CODE,
    ),
    (
        "MarkdownContextStore edit error codes",
        keys::MEMORY_EDIT_NOT_FOUND_CODE,
    ),
    ("memory audit disable warning", keys::MEMORY_AUDIT_DISABLED),
    ("MEMORY_SCOPES", keys::MEMORY_SCOPE_USER_LABEL),
    ("MEMORY_SCOPES", keys::MEMORY_SCOPE_MEMORY_LABEL),
    ("cancellation strings", keys::WORK_CANCELLED_BY_USER),
    ("cancellation strings", keys::WORK_CANCEL_FAILED),
    ("cancellation strings", keys::WORK_NO_RUNNER_CONFIGURED),
    (
        "scheduled-task timeout errors",
        keys::WORK_SCHEDULED_TIMEOUT_ABORT,
    ),
    (
        "scheduled-task timeout errors",
        keys::WORK_SCHEDULED_TIMEOUT_ERROR,
    ),
    (
        "task store quarantine path + warnings",
        keys::STORE_TASK_QUARANTINED,
    ),
    (
        "task store quarantine path + warnings",
        keys::STORE_TASK_QUARANTINE_FAILED,
    ),
    (
        "notes quarantine + persistence warnings",
        keys::STORE_NOTES_INVALID_JSON,
    ),
    (
        "notes quarantine + persistence warnings",
        keys::STORE_NOTES_INVALID_SHAPE,
    ),
    (
        "notes quarantine + persistence warnings",
        keys::STORE_NOTES_READ_FAILED,
    ),
    (
        "notes quarantine + persistence warnings",
        keys::STORE_NOTES_SAVE_FAILED,
    ),
    (
        "notes quarantine + persistence warnings",
        keys::STORE_NOTES_QUARANTINED,
    ),
    (
        "notes quarantine + persistence warnings",
        keys::STORE_NOTES_QUARANTINE_FAILED,
    ),
    (
        "notes quarantine + persistence warnings",
        keys::STORE_NOTES_PERSISTENCE_DISABLED,
    ),
    (
        "notes quarantine + persistence warnings",
        keys::STORE_NOTES_UNAVAILABLE,
    ),
    (
        "input normalization rejection messages",
        keys::INPUT_MISSING_MIME,
    ),
    (
        "input normalization rejection messages",
        keys::INPUT_MISSING_URL,
    ),
    (
        "input normalization rejection messages",
        keys::INPUT_INVALID_URL,
    ),
    (
        "input normalization rejection messages",
        keys::INPUT_DATA_URL_NEEDS_BASE64,
    ),
    ("input normalization rejection messages", keys::INPUT_EMPTY),
    (
        "frontendInputProjection model-visible envelope",
        keys::INPUT_FALLBACK_WITH_VOICE,
    ),
    (
        "frontendInputProjection model-visible envelope",
        keys::INPUT_FALLBACK_WITHOUT_TEXT,
    ),
    (
        "restored conversation context item text",
        keys::REALTIME_RESTORED_CONTEXT_INSTRUCTIONS,
    ),
    (
        "progress injection text",
        keys::REALTIME_PROGRESS_INSTRUCTIONS,
    ),
    (
        "work results wrapper (announcement payload)",
        keys::REALTIME_WORK_RESULTS_HEADER,
    ),
    (
        "delegated result injection block",
        keys::ACP_DELEGATION_RESULT_INSTRUCTIONS,
    ),
    ("cancel control turn", keys::ACP_CANCEL_CONTROL_TAIL),
    ("status control turn", keys::ACP_STATUS_CONTROL_TAIL),
    ("status control turn", keys::ACP_STATUS_CONTROL_NO_QUESTION),
    (
        "coordinator protocol retry block",
        keys::COORDINATOR_PROTOCOL_RETRY_INSTRUCTIONS,
    ),
    (
        "generic-ACP configuration errors",
        keys::BACKEND_GENERIC_ACP_REQUIRES_COMMAND,
    ),
    (
        "generic-ACP configuration errors",
        keys::BACKEND_GENERIC_ACP_FULL_PERMISSION_UNSAFE,
    ),
    (
        "driver validation errors",
        keys::BACKEND_DRIVER_NOT_REGISTERED,
    ),
    (
        "driver validation errors",
        keys::BACKEND_DRIVER_LABEL_MISMATCH,
    ),
    (
        "driver validation errors",
        keys::BACKEND_DRIVER_MISSING_CREATE_PROFILE,
    ),
    (
        "driver validation errors",
        keys::BACKEND_DRIVER_INCOMPLETE_CAPABILITIES,
    ),
    (
        "driver validation errors",
        keys::BACKEND_DRIVER_INVALID_PROFILE,
    ),
    (
        "driver validation errors",
        keys::BACKEND_RUNTIME_DRIVER_ID_MISMATCH,
    ),
    (
        "driver validation errors",
        keys::BACKEND_RUNTIME_DRIVER_MISSING_RESOLVE,
    ),
    (
        "driver validation errors",
        keys::BACKEND_RUNTIME_DRIVER_MISSING_OWNERSHIP,
    ),
    (
        "driver validation errors",
        keys::BACKEND_RUNTIME_DRIVER_EXTERNAL_MISMATCH,
    ),
    (
        "driver validation errors",
        keys::BACKEND_RUNTIME_DRIVER_MISSING_MANAGED_SCRIPT,
    ),
    (
        "runtime/ownership errors",
        keys::BACKEND_FULL_PERMISSION_REQUIRES_OWNED,
    ),
    (
        "runtime/ownership errors",
        keys::BACKEND_SERVICE_URL_HAS_CREDENTIALS,
    ),
    (
        "runtime/ownership errors",
        keys::BACKEND_OPENCLAW_FULL_PERMISSION_UNSAFE,
    ),
    (
        "AgentClient unsupported-capability errors",
        keys::AGENT_PERMISSION_UNSUPPORTED,
    ),
    (
        "AgentClient unsupported-capability errors",
        keys::AGENT_CANCEL_UNSUPPORTED,
    ),
    (
        "AgentClient unsupported-capability errors",
        keys::AGENT_QUERY_UNSUPPORTED,
    ),
    (
        "AgentClient unsupported-capability errors",
        keys::AGENT_RESUME_UNSUPPORTED,
    ),
    (
        "AgentClient unsupported-capability errors",
        keys::AGENT_NOT_CONFIGURED,
    ),
    ("OpenClaw gateway errors", keys::OPENCLAW_GATEWAY_CLOSED),
    (
        "OpenClaw gateway errors",
        keys::OPENCLAW_GATEWAY_REQUEST_CANCELLED,
    ),
    ("OpenClaw gateway errors", keys::OPENCLAW_NO_RUN_ID),
    ("OpenClaw gateway errors", keys::OPENCLAW_SUBTASK_NO_TEXT),
    (
        "OpenClaw gateway errors",
        keys::OPENCLAW_MODEL_OVERRIDE_INCOMPLETE,
    ),
    (
        "openclaw bridge diagnostics",
        keys::OPENCLAW_MISSING_DEVICE_SCOPE,
    ),
    (
        "delegation lookup / lifecycle errors",
        keys::ACP_DELEGATION_ALREADY_STARTED,
    ),
    ("process lifecycle errors", keys::ACP_INITIALIZE_FAILED),
    ("session/prompt", keys::ACP_SESSION_CANCELLED),
    ("session/load", keys::ACP_RESUME_UNSUPPORTED),
    ("concurrent prompt on one session", keys::ACP_SESSION_BUSY),
    (
        "adapter session-config errors",
        keys::ACP_UNSUPPORTED_CONNECTION,
    ),
    (
        "normalizeCoordinatorContent legacy inline upgrade",
        keys::ACP_AGENT_RESULT_TITLE,
    ),
    (
        "setup human report header/footer",
        keys::SETUP_REPORT_HEADER,
    ),
    ("setup human report header/footer", keys::SETUP_SELECTED),
    (
        "setup human report header/footer",
        keys::SETUP_FOOTER_MODELS,
    ),
    (
        "setup human report header/footer",
        keys::SETUP_FOOTER_AUTO_DOWNLOAD,
    ),
    (
        "setup human report header/footer",
        keys::SETUP_FOOTER_OTHER_BACKENDS,
    ),
    (
        "service address restriction",
        keys::CLI_SERVICE_LOCAL_HTTP_ONLY,
    ),
    (
        "foreground/service collision",
        keys::CLI_FOREGROUND_SERVICE_COLLISION,
    ),
    (
        "service action with config flags",
        keys::CLI_SERVICE_READS_CONFIG_FILE,
    ),
    ("unknown flag / missing value", keys::CLI_UNKNOWN_ARGUMENT),
    (
        "unknown flag / missing value",
        keys::CLI_OPTION_MISSING_VALUE,
    ),
    ("--realtime-model", keys::CLI_REALTIME_MODEL_CONFIG_SET_ONLY),
    (
        "--realtime-model",
        keys::CLI_CONFIG_SET_NEEDS_REALTIME_MODEL,
    ),
    ("--session", keys::CLI_SESSION_CANNOT_BE_EMPTY),
    ("--no-open", keys::CLI_NO_OPEN_WEBUI_ONLY),
    ("--takeover", keys::CLI_TAKEOVER_TUI_WEBUI_ONLY),
    ("--json", keys::CLI_JSON_SETUP_ONLY),
    ("--yes / -y", keys::CLI_YES_INSTALL_ONLY),
    ("--skill / --list", keys::CLI_LIST_AND_SKILL_CONFLICT),
    ("--audio-mode", keys::CLI_AUDIO_MODE_TUI_ONLY),
    ("--audio-mode", keys::CLI_UNSUPPORTED_AUDIO_MODE),
    ("config actions", keys::CLI_UNKNOWN_CONFIG_COMMAND),
    ("SKILL_ACTIONS", keys::CLI_SKILL_NEEDS_SUBCOMMAND),
    ("gateway status output", keys::CLI_SERVICE_STATE_RUNNING),
    ("gateway status output", keys::CLI_SERVICE_STATE_STOPPED),
    (
        "gateway status output",
        keys::CLI_SERVICE_STATE_NOT_INSTALLED,
    ),
    ("service lifecycle output", keys::CLI_SERVICE_STOPPED),
    ("service lifecycle output", keys::CLI_SERVICE_REMOVED),
    ("gatewaySummary", keys::GATEWAY_FRONTEND_ONLY_MODE),
    ("install flow output", keys::INSTALL_CONFIRM_QUESTION),
    ("install flow output", keys::INSTALL_STEP_SKIPPED),
    ("install target validation", keys::CLI_INSTALL_GENERIC_ACP),
    (
        "gateway reuse compatibility errors",
        keys::CLI_REUSE_BACKEND_REPORT_INCOMPLETE,
    ),
    (
        "gateway reuse compatibility errors",
        keys::CLI_REUSE_REALTIME_REPORT_INCOMPLETE,
    ),
    ("startup timeout messages", keys::GATEWAY_START_TIMEOUT),
    ("browser launchers", keys::CLI_BROWSER_OPEN_FAILED),
    (
        "CodeBuddy template missing default model",
        keys::BACKEND_CODEBUDDY_TEMPLATE_MISSING_MODEL,
    ),
    (
        "connect timeout messages",
        keys::REALTIME_CONNECT_TIMEOUT_DASHSCOPE,
    ),
    (
        "connect timeout messages",
        keys::REALTIME_CONNECT_TIMEOUT_SPEECH_TO_SPEECH,
    ),
    (
        "missing configuration messages",
        keys::GATEWAY_MISSING_DASHSCOPE_API_KEY_SHORT,
    ),
    (
        "missing configuration messages",
        keys::GATEWAY_MISSING_SPEECH_TO_SPEECH_URL,
    ),
    (
        "provider/protocol validation error messages",
        keys::REALTIME_PROVIDER_NEEDS_KEY_AND_LABEL,
    ),
    (
        "provider/protocol validation error messages",
        keys::REALTIME_PROVIDER_KEY_INVALID,
    ),
    (
        "provider/protocol validation error messages",
        keys::REALTIME_PROVIDER_NAME_TAKEN,
    ),
    (
        "provider/protocol validation error messages",
        keys::REALTIME_PROVIDER_UNKNOWN_CAPABILITY,
    ),
    (
        "provider/protocol validation error messages",
        keys::REALTIME_PROVIDER_VISIBILITY_INVALID,
    ),
    (
        "provider/protocol validation error messages",
        keys::REALTIME_PROVIDER_ALIASES_INVALID,
    ),
    (
        "provider/protocol validation error messages",
        keys::REALTIME_PROVIDER_SESSION_DEFAULTS_INCOMPLETE,
    ),
    (
        "unsupported realtime provider error message",
        keys::REALTIME_UNSUPPORTED_FRONTEND,
    ),
    (
        "user-facing gateway error messages (Chinese, sent as { type:'error', message })",
        keys::REALTIME_MODEL_NEVER_REPLIED,
    ),
    (
        "user-facing gateway error messages (Chinese, sent as { type:'error', message })",
        keys::REALTIME_PERMISSION_ASK_FAILED,
    ),
    (
        "user-facing gateway error messages (Chinese, sent as { type:'error', message })",
        keys::REALTIME_ANNOUNCEMENT_RETRYING,
    ),
    (
        "RealtimeFrontend internal error messages",
        keys::REALTIME_REQUEST_CANCELLED,
    ),
    (
        "RealtimeFrontend internal error messages",
        keys::REALTIME_SESSION_RESET,
    ),
    (
        "RealtimeFrontend internal error messages",
        keys::REALTIME_RESPONSE_CORRELATION_CONFLICT,
    ),
    (
        "RealtimeFrontend internal error messages",
        keys::REALTIME_CONNECTION_CLOSED,
    ),
    (
        "realtimeEventErrorMessage composition",
        keys::REALTIME_SERVICE_ERROR,
    ),
    (
        "background work progress-check message",
        keys::WORK_ACTIVITY_VERB_RUN,
    ),
    (
        "background work progress-check message",
        keys::WORK_ACTIVITY_VERB_IMAGE,
    ),
    (
        "<recent_conversation> line format",
        keys::REALTIME_ROLE_USER,
    ),
    (
        "<recent_conversation> line format",
        keys::REALTIME_ROLE_ASSISTANT,
    ),
    ("seed file templates", keys::RUNTIME_USER_MD_HEADING_ADDRESS),
    (
        "seed file templates",
        keys::RUNTIME_USER_MD_HEADING_INTERACTION,
    ),
    ("seed file templates", keys::MEMORY_MIGRATED_USER_HEADING),
    ("seed file templates", keys::MEMORY_MIGRATED_MEMORY_HEADING),
    (
        "memory extractor user message layout",
        keys::MEMORY_EXTRACTOR_SECTION_USER,
    ),
    (
        "memory extractor user message layout",
        keys::MEMORY_EXTRACTOR_SECTION_MEMORY,
    ),
    (
        "memory extractor user message layout",
        keys::MEMORY_EXTRACTOR_SECTION_TRANSCRIPT,
    ),
];

/// The literal runs of a template — everything outside `{placeholder}`.
///
/// The catalogue quotes JavaScript, so a template's interpolations appear
/// there in whatever spelling the surveyor used (`<KEY>`, `${key}`, an inlined
/// expansion). The *literal* text between them is the part that is a contract,
/// and it is what this compares.
fn literal_segments(template: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let bytes = template.as_bytes();
    let mut index = 0;
    let mut start = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'{' if bytes.get(index + 1) == Some(&b'{') => index += 2,
            b'}' if bytes.get(index + 1) == Some(&b'}') => index += 2,
            b'{' => {
                if start < index {
                    out.push(&template[start..index]);
                }
                let rest = &template[index + 1..];
                let len = rest.find('}').expect("terminated placeholder");
                index += 1 + len + 1;
                start = index;
            }
            _ => index += 1,
        }
    }
    if start < template.len() {
        out.push(&template[start..]);
    }
    // A run that is only whitespace is layout, not contract text: the
    // catalogue quotes a trailing `\n` on some CLI lines and omits it on
    // others, and neither spelling is a claim about the message.
    out.retain(|segment| !segment.trim().is_empty());
    out
}

/// True when every segment appears in `haystack`, in order and without
/// overlapping.
fn contains_in_order(haystack: &str, segments: &[&str]) -> bool {
    let mut cursor = 0;
    for segment in segments {
        match haystack[cursor..].find(segment) {
            Some(offset) => cursor += offset + segment.len(),
            None => return false,
        }
    }
    true
}

#[test]
fn every_contained_contract_value_carries_our_message_verbatim() {
    for (contract, key) in CONTAINED {
        let ours = t(Locale::Zh, *key);
        let segments = literal_segments(ours);
        assert!(
            !segments.is_empty(),
            "{key} has no literal text to compare against a contract"
        );
        let candidates = contract_values(contract);
        assert!(
            candidates
                .iter()
                .any(|value| contains_in_order(value, &segments)),
            "contract `{contract}` does not carry {key}.\n  ours:      {ours:?}\n  \
             segments:  {segments:?}\n  catalogue: {candidates:?}"
        );
    }
}

/// The memory-extractor system prompt is catalogued truncated — the surveyor
/// cut it at 600 characters and said so. Assert the part that *is* catalogued
/// is a prefix of ours, which is the strongest claim the catalogue supports.
#[test]
fn the_memory_extractor_prompt_matches_the_catalogued_prefix() {
    const MARKER: &str = " \u{2026}[remaining";
    let catalogued = contract_values("MEMORY EXTRACTOR system prompt (EXTRACTOR_SYSTEM_PROMPT)");
    let full = catalogued.first().expect("one entry");
    let cut = full.find(MARKER).unwrap_or_else(|| {
        panic!("the catalogued extractor prompt no longer carries its truncation marker")
    });
    let prefix = &full[..cut];
    let ours = as_upstream_template(t(Locale::Zh, keys::MEMORY_EXTRACTOR_SYSTEM_PROMPT));
    assert!(
        ours.starts_with(prefix),
        "the catalogued prefix is not a prefix of ours.\n  catalogued: {prefix:?}\n  \
         ours:       {ours:?}"
    );
    // The truncated tail is named in the catalogue by its opening words; each
    // has to be somewhere in the rest of our value.
    for tail in ["不提取", "绝不提取", "已有内容覆盖", "没有值得修改"] {
        assert!(
            ours.contains(tail),
            "our extractor prompt is missing {tail:?}"
        );
    }
}

// ── coverage ───────────────────────────────────────────────────────────────

#[test]
fn the_contract_tables_are_free_of_duplicates_and_cover_a_meaningful_share() {
    let mut named: Vec<&str> = EXACT
        .iter()
        .chain(REBRANDED)
        .chain(CONTAINED)
        .map(|(_, key)| key.as_str())
        .collect();
    let total = named.len();
    named.sort_unstable();
    named.dedup();
    assert_eq!(
        total,
        named.len(),
        "a key is asserted against two contracts"
    );
    assert!(
        named.len() >= 150,
        "only {} keys are pinned to a contract",
        named.len()
    );
}

/// A key in more than one table would let a weak assertion stand in for a
/// strong one.
#[test]
fn exact_and_contained_do_not_overlap() {
    for (_, exact) in EXACT.iter().chain(REBRANDED) {
        assert!(
            !CONTAINED.iter().any(|(_, key)| key == exact),
            "{exact} is asserted both exactly and by containment"
        );
    }
}
