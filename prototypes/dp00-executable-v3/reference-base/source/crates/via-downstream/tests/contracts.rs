//! Every contract value this crate owns, asserted against
//! `docs/reference/contracts.json`.
//!
//! Nothing here retypes a catalogue value: the key names, the field orders, the
//! bounds, the status vocabularies and the two worked session-key examples are
//! all scanned out of the catalogue's own prose and compared against what the
//! shipped types produce.

mod common;

use common::{
    between, boolean_pairs, contract_value, contract_why, double_quoted_tokens, json_keys,
    quoted_tokens, upstream_declarations,
};
use pretty_assertions::assert_eq;
use via_downstream::{
    ActivityStatus, ActivityTracker, BackendCapabilities, CAPABILITY_FLAGS, CancelOutcome,
    CancelRoute, CancelTarget, DEFAULT_TOOL_NAME, DETAIL_BOUND, HarnessError, HarnessHealth,
    HarnessStatus, HarnessStatusCode, PLAN_ACTIVITY_ID, PlanEntry, PlanEntryStatus, PromptOutcome,
    RawSessionUpdate, STATUS_NOT_FOUND, SessionEvent, SessionKey, StopReason, TOOL_LABEL_BOUND,
    TOOL_NAME_BOUND, TerminalState, ToolCategory, UPDATE_AGENT_MESSAGE_CHUNK, UPDATE_PLAN,
    UPDATE_TOOL_CALL,
};
use via_i18n::Locale;
use via_protocol::WorkStatus;

// ── coordinator session key ────────────────────────────────────────────────

/// `state-name` / *coordinator session key format*.
///
/// The catalogue states the template and two worked examples. Both examples
/// are parsed out of it and rebuilt through the shipped constructor.
#[test]
fn coordinator_session_key_matches_the_catalogued_examples() {
    let stated = contract_value("state-name", "coordinator session key format");

    // The examples are the only double-quoted tokens shaped `a:b:backend`.
    let examples: Vec<String> = double_quoted_tokens(&stated)
        .into_iter()
        .filter(|token| token.ends_with(":backend") && token.matches(':').count() == 2)
        .collect();
    assert_eq!(
        examples,
        vec!["openclaw:owner%20one:backend", "opencode:personal:backend"],
        "the catalogue's worked examples moved; the test is reading the wrong entry",
    );

    assert_eq!(
        SessionKey::coordinator("openclaw", "owner one").as_str(),
        examples[0],
    );
    assert_eq!(
        SessionKey::coordinator("opencode", "").as_str(),
        examples[1]
    );
}

/// The four properties the catalogue calls out in the same entry, each read
/// back out of its prose so the test fails if the catalogue stops claiming it.
#[test]
fn coordinator_session_key_keeps_all_four_stated_properties() {
    let stated = contract_value("state-name", "coordinator session key format");
    assert!(stated.contains("trimmed"), "{stated}");
    assert!(stated.contains("'personal'"), "{stated}");
    assert!(stated.contains("percent-encoded"), "{stated}");
    assert!(stated.contains("NOT lowercased"), "{stated}");

    // Trimmed, and defaulted only when nothing is left.
    assert_eq!(
        SessionKey::coordinator("acp", "   ana   ").as_str(),
        SessionKey::coordinator("acp", "ana").as_str(),
    );
    assert_eq!(
        SessionKey::coordinator("acp", " \t ").as_str(),
        "acp:personal:backend",
    );
    // Percent-encoded, and not lower-cased.
    assert_eq!(
        SessionKey::coordinator("acp", "Owner One").as_str(),
        "acp:Owner%20One:backend",
    );
    assert_ne!(
        SessionKey::coordinator("acp", "Owner").as_str(),
        SessionKey::coordinator("acp", "owner").as_str(),
    );
}

/// The catalogue's `why` names the three places the key is used. It is quoted
/// here so a reviewer sees why the format is not free to drift.
#[test]
fn the_catalogue_still_says_the_key_is_persisted() {
    let why = contract_why("state-name", "coordinator session key format");
    assert!(why.contains("persisted registry key"), "{why}");
    assert!(why.contains("lock key prefix"), "{why}");
}

// ── capability contract ────────────────────────────────────────────────────

/// `json-field` / *backend agent driver capability contract*.
///
/// The seven flag names, in the catalogue's order, against
/// [`CAPABILITY_FLAGS`].
#[test]
fn the_seven_capability_flags_are_the_catalogued_seven() {
    let stated = contract_value("json-field", "backend agent driver capability contract");
    let names: Vec<&str> = between(&stated, "seven required booleans: ", ";")
        .split(", ")
        .map(str::trim)
        .collect();
    assert_eq!(names, CAPABILITY_FLAGS.to_vec());
}

/// The same entry records DeepSeek's exact answers. They are parsed out and
/// compared field by field against the declaration this crate's tests carry.
#[test]
fn deepseeks_declaration_matches_the_catalogue() {
    let stated = contract_value("json-field", "backend agent driver capability contract");
    let pairs = boolean_pairs(between(&stated, "deepseek profile is {", "}"));
    assert_eq!(pairs.len(), CAPABILITY_FLAGS.len());

    let deepseek = upstream_declarations()
        .into_iter()
        .find(|(id, _)| *id == "deepseek")
        .map(|(_, capabilities)| capabilities)
        .expect("deepseek is one of the twelve");

    for (name, expected) in pairs {
        assert_eq!(
            deepseek.flag(&name),
            Some(expected),
            "deepseek.{name} disagrees with the catalogue",
        );
    }
}

/// The catalogue says the profile's capabilities are **frozen**. In Rust that
/// is `Copy` with no interior mutability: reading a descriptor's capabilities
/// and mutating the copy cannot reach the descriptor.
#[test]
fn the_capability_declaration_is_frozen() {
    let stated = contract_value("json-field", "backend agent driver capability contract");
    assert!(stated.contains("frozen"), "{stated}");

    let descriptor = via_downstream::HarnessDescriptor::declare(
        "deepseek",
        "DeepSeek",
        upstream_declarations()
            .into_iter()
            .find(|(id, _)| *id == "deepseek")
            .map(|(_, capabilities)| capabilities)
            .expect("deepseek is one of the twelve"),
    )
    .expect("deepseek declares consistently");

    let mut copy = descriptor.capabilities();
    copy.delegation = true;
    assert!(copy.delegation, "the copy is the caller's to change");
    assert!(
        !descriptor.capabilities().delegation,
        "the descriptor is not",
    );
}

// ── backend.activity ───────────────────────────────────────────────────────

/// `json-field` / *backend.activity event*, plan variant: keys, order, and the
/// two literals.
#[test]
fn the_plan_activity_payload_matches_the_catalogue() {
    let stated = contract_value("json-field", "backend.activity event");
    let variant = between(&stated, "Plan variant: ", ". Tool variant:");
    let keys = json_keys(variant);
    assert_eq!(
        keys,
        vec!["id", "kind", "status", "detail", "completed", "total"],
    );
    assert!(
        variant.contains(&format!("\"{PLAN_ACTIVITY_ID}\"")),
        "{variant}"
    );
    assert!(variant.contains("bounded 300"), "{variant}");
    assert_eq!(DETAIL_BOUND, 300);

    let mut tracker = ActivityTracker::new();
    let event = tracker
        .project(&RawSessionUpdate::plan(vec![
            PlanEntry {
                content: "  done  ".to_owned(),
                status: PlanEntryStatus::Completed,
            },
            PlanEntry {
                content: "now\tdoing".to_owned(),
                status: PlanEntryStatus::InProgress,
            },
        ]))
        .expect("a plan projects");
    let rendered = serde_json::to_string(&event).expect("serializable");
    assert_eq!(
        rendered,
        r#"{"id":"acp-plan","kind":"plan","status":"running","detail":"now doing","completed":1,"total":2}"#,
    );
    assert_eq!(json_keys(&rendered), keys);
}

/// The tool variant: keys, order, the three bounds, the default tool name and
/// the five categories.
#[test]
fn the_tool_activity_payload_matches_the_catalogue() {
    let stated = contract_value("json-field", "backend.activity event");
    let variant = between(&stated, "Tool variant: ", ". Text variant:");
    let keys = json_keys(variant);
    assert_eq!(
        keys,
        vec![
            "id", "kind", "tool", "label", "status", "category", "detail"
        ],
    );

    assert!(variant.contains("bounded 100"), "{variant}");
    assert!(variant.contains("bounded 160"), "{variant}");
    assert!(variant.contains("bounded 300"), "{variant}");
    assert_eq!(
        (TOOL_NAME_BOUND, TOOL_LABEL_BOUND, DETAIL_BOUND),
        (100, 160, 300),
    );
    assert!(
        variant.contains(&format!("default '{DEFAULT_TOOL_NAME}'")),
        "{variant}",
    );

    let categories: Vec<String> = double_quoted_tokens(between(variant, "\"category\":", ","))
        .into_iter()
        .collect();
    assert_eq!(categories, vec!["image", "search", "read", "write", "run"]);
    assert_eq!(
        categories,
        [
            ToolCategory::Image,
            ToolCategory::Search,
            ToolCategory::Read,
            ToolCategory::Write,
            ToolCategory::Run,
        ]
        .map(|category| category.as_str().to_owned())
        .to_vec(),
    );

    let mut tracker = ActivityTracker::new();
    let event = tracker
        .project(&RawSessionUpdate {
            session_update: UPDATE_TOOL_CALL.to_owned(),
            tool_call_id: Some("call-1".to_owned()),
            name: Some("grep".to_owned()),
            title: Some("Find files".to_owned()),
            status: Some("in_progress".to_owned()),
            raw_input: Some(serde_json::json!({ "path": "/srv/via" })),
            entries: Vec::new(),
        })
        .expect("a tool call projects");
    let rendered = serde_json::to_string(&event).expect("serializable");
    assert_eq!(
        rendered,
        r#"{"id":"call-1","kind":"tool","tool":"grep","label":"Find files","status":"in_progress","category":"read","detail":"/srv/via"}"#,
    );
    assert_eq!(json_keys(&rendered), keys);
}

/// The text variant, and the rule that everything else projects to nothing.
#[test]
fn the_text_activity_payload_matches_the_catalogue() {
    let stated = contract_value("json-field", "backend.activity event");
    let variant = between(&stated, "Text variant: ", ". Anything else");
    assert_eq!(json_keys(variant), vec!["id", "kind", "status"]);
    assert!(
        stated.contains("Anything else => null (no event)"),
        "{stated}"
    );

    let mut tracker = ActivityTracker::new();
    let event = tracker
        .project(&RawSessionUpdate {
            session_update: UPDATE_AGENT_MESSAGE_CHUNK.to_owned(),
            ..RawSessionUpdate::default()
        })
        .expect("an agent message projects");
    assert_eq!(event, SessionEvent::Text);
    assert_eq!(
        serde_json::to_string(&event).expect("serializable"),
        r#"{"id":null,"kind":"text","status":"running"}"#,
    );
}

/// The catalogue's `why` is the boundary statement this crate is built around.
#[test]
fn the_catalogue_still_calls_the_activity_surface_public_and_scrubbed() {
    let why = contract_why("json-field", "backend.activity event");
    assert!(why.contains("This IS the public progress surface"), "{why}");
    for absent in [
        "Session IDs",
        "subagent IDs",
        "raw permission payloads",
        "raw reasoning",
    ] {
        assert!(
            why.contains(absent),
            "the catalogue no longer names {absent}"
        );
    }
}

// ── prompt / cancellation ──────────────────────────────────────────────────

/// `json-rpc-method` / *session/prompt*: `cancelled` is a hard error.
#[test]
fn a_cancelled_stop_reason_cannot_become_a_successful_outcome() {
    let why = contract_why("json-rpc-method", "session/prompt");
    assert!(
        why.contains(
            "a Rust port that returns Ok on 'cancelled' would silently complete cancelled Work"
        ),
        "{why}",
    );
    let stated = contract_value("json-rpc-method", "session/prompt");
    assert!(stated.contains(r#"stopReason === "cancelled""#), "{stated}");

    let error = PromptOutcome::new("partial", StopReason::from_wire("cancelled"))
        .expect_err("cancelled must not return Ok");
    assert!(error.is_cancelled());
}

/// `json-field` / *session_cancel result*: the field set, the order, and the
/// bare `not_found`.
#[test]
fn the_cancel_result_payload_matches_the_catalogue() {
    let stated = contract_value("json-field", "session_cancel result");
    assert!(
        stated.contains(&format!("'{STATUS_NOT_FOUND}'")),
        "{stated}"
    );

    assert_eq!(
        serde_json::to_string(&CancelOutcome::NotFound).expect("serializable"),
        r#"{"status":"not_found"}"#,
    );

    let outcome = CancelOutcome::requested(CancelRoute::Adapter, CancelTarget::delegation("d-1"));
    let rendered = serde_json::to_string(&outcome).expect("serializable");
    assert_eq!(
        rendered,
        r#"{"status":"cancelling","delegation_id":"d-1","session_id":null}"#,
    );
    // The catalogue names the three fields in this order.
    let stated_fields: Vec<&str> = between(&stated, "{ ", " }")
        .split(", ")
        .map(|field| field.split(':').next().unwrap_or_default().trim())
        .collect();
    assert_eq!(stated_fields, vec!["status", "delegation_id", "session_id"]);
}

/// The deliberate divergence, stated against the catalogue rather than only in
/// prose: upstream's `record.status` is already `cancelled` when this payload
/// is built, VIA's is `cancelling` until the harness confirms.
#[test]
fn an_unconfirmed_cancel_reports_cancelling_where_upstream_reports_cancelled() {
    let stated = contract_value("json-field", "session_cancel result");
    assert!(
        stated.contains("record.status after cancellation"),
        "{stated}"
    );

    let requested = CancelOutcome::requested(CancelRoute::Adapter, CancelTarget::delegation("d-1"));
    assert_eq!(requested.work_status(), Some(WorkStatus::Cancelling));
    assert_eq!(requested.status_str(), WorkStatus::Cancelling.as_str());
    assert!(!requested.is_confirmed());

    let confirmed = requested
        .confirm("2026-08-22T09:00:00.000Z")
        .expect("the one legal edge");
    assert_eq!(confirmed.work_status(), Some(WorkStatus::Cancelled));
    assert_eq!(confirmed.status_str(), WorkStatus::Cancelled.as_str());
    assert!(confirmed.is_confirmed());
}

/// The three terminal states upstream short-circuits on project to the three
/// terminal Work statuses.
#[test]
fn terminal_states_project_to_work_statuses() {
    assert_eq!(
        TerminalState::Completed.work_status(),
        WorkStatus::Completed,
    );
    assert_eq!(TerminalState::Failed.work_status(), WorkStatus::Failed);
    assert_eq!(
        TerminalState::Cancelled.work_status(),
        WorkStatus::Cancelled
    );
}

// ── health ─────────────────────────────────────────────────────────────────

/// `error-code` / *backend status codes*: all thirteen codes and all five
/// statuses, parsed out of the catalogue.
#[test]
fn the_health_vocabulary_is_the_catalogued_vocabulary() {
    let stated = contract_value("error-code", "backend status codes");
    let (codes_part, statuses_part) = stated
        .split_once("Status strings:")
        .expect("the catalogue states both halves");

    let codes: Vec<String> = quoted_tokens(codes_part);
    assert_eq!(
        codes,
        HarnessStatusCode::ALL
            .map(|code| code.as_str().to_owned())
            .to_vec(),
    );

    let statuses: Vec<String> = quoted_tokens(statuses_part);
    assert_eq!(
        statuses,
        [
            HarnessStatus::Stopped,
            HarnessStatus::Starting,
            HarnessStatus::Ready,
            HarnessStatus::Failed,
            HarnessStatus::NotConfigured,
        ]
        .map(|status| status.as_str().to_owned())
        .to_vec(),
    );
}

/// `state-name` / *BackendAvailability probe*: the three codes that count as a
/// cold start, and the `status === 'starting'` arm.
#[test]
fn the_transient_predicate_is_the_catalogued_predicate() {
    let stated = contract_value("state-name", "BackendAvailability probe");
    let listed = between(&stated, "health.status === 'starting' || [", "]");
    let codes: Vec<String> = quoted_tokens(listed);
    assert_eq!(
        codes,
        HarnessStatusCode::TRANSIENT
            .map(|code| code.as_str().to_owned())
            .to_vec(),
    );

    for code in HarnessStatusCode::ALL {
        let health = HarnessHealth::failed(code, "boom");
        let expected = HarnessStatusCode::TRANSIENT.contains(&code);
        assert_eq!(
            health.is_transient(),
            expected,
            "{} classified wrongly on a failed health",
            code.as_str(),
        );
    }
    // The other arm: a `starting` status is transient whatever its code.
    assert!(HarnessHealth::starting().is_transient());
    assert!(HarnessHealth::backend_starting().is_transient());
}

// ── driver validation errors ───────────────────────────────────────────────

/// `error-code` / *driver validation errors*.
///
/// Every refusal this crate is responsible for is rendered through the real
/// [`HarnessError::message`] path and matched against the catalogue's segment,
/// byte for byte. The catalogue's `<id>` placeholder is fed in as the id, so
/// the interpolation point is asserted too.
#[test]
fn every_agent_driver_refusal_renders_the_catalogued_sentence() {
    let stated = contract_value("error-code", "driver validation errors");
    let segments: Vec<&str> = stated.split(" | ").map(str::trim).collect();

    let owned = [
        HarnessError::DriverNotRegistered {
            id: "<id>".to_owned(),
        },
        HarnessError::DriverLabelMismatch {
            id: "<id>".to_owned(),
            declared: "wrong".to_owned(),
            expected: "Codex",
        },
        HarnessError::IncompleteCapabilities {
            id: "<id>".to_owned(),
            faults: Vec::new(),
        },
        HarnessError::InvalidSession {
            id: "<id>".to_owned(),
        },
        HarnessError::UnsupportedBackend {
            id: "<id>".to_owned(),
        },
    ];

    for error in owned {
        let rendered = error.message(Locale::Zh);
        assert!(
            segments.contains(&rendered.as_str()),
            "`{rendered}` is not one of the catalogued driver validation errors",
        );
    }
}

/// The refusals this crate deliberately does **not** reproduce are still in
/// the catalogue, and are still somebody's.
///
/// `缺少 createProfile` and the two `skills` refusals are unrepresentable in
/// Rust; the five `Runtime Driver` ones belong to `via-process`. Asserting they
/// are present keeps the deviation honest — if the catalogue drops one, this
/// test says so instead of the deviation quietly becoming a lie.
#[test]
fn the_refusals_this_crate_does_not_own_are_still_catalogued() {
    let stated = contract_value("error-code", "driver validation errors");
    for segment in [
        "后台 Driver 缺少 createProfile：<id>",
        "后台 Runtime Driver 标识不一致：<id>",
        "后台 Runtime Driver 缺少 resolve：<id>",
        "后台 Runtime Driver 缺少进程归属声明：<id>",
        "后台 Runtime Driver 外部服务能力不一致：<id>",
        "后台 Runtime Driver 缺少 managedScript：<id>",
    ] {
        assert!(
            stated.contains(segment),
            "the catalogue no longer has `{segment}`"
        );
    }
}

/// `error-code` / *service endpoint rejections* names the `external ownership`
/// refusal that `via-catalog` already owns, and this crate re-exports through
/// [`HarnessError::Configuration`]. The check here is that the seam does not
/// grow a second copy of it.
#[test]
fn the_seam_does_not_restate_catalog_refusals() {
    let stated = contract_value("error-code", "service endpoint rejections");
    assert!(stated.contains("不支持连接外部后台服务"), "{stated}");

    let error = HarnessError::Configuration(via_core::CoreError::Catalog(
        via_catalog::CatalogError::ExternalServiceUnsupported { label: "OpenCode" },
    ));
    assert_eq!(error.code(), "VIA_BACKEND_EXTERNAL_SERVICE_UNSUPPORTED");
}

// ── status defaults ────────────────────────────────────────────────────────

/// Upstream's `merged.status || 'running'` default, and the terminal test that
/// decides whether the tracker forgets a tool call.
#[test]
fn the_activity_status_default_and_terminal_test_are_upstreams() {
    let stated = contract_value("json-field", "backend.activity event");
    assert!(stated.contains("update.status||'running'"), "{stated}");
    assert_eq!(ActivityStatus::default(), ActivityStatus::Running);

    assert!(ActivityStatus::Completed.is_terminal());
    assert!(ActivityStatus::Failed.is_terminal());
    for status in [
        ActivityStatus::Pending,
        ActivityStatus::InProgress,
        ActivityStatus::Running,
    ] {
        assert!(!status.is_terminal(), "{status} is not terminal upstream");
    }
}

/// The plan variant's `id` is a constant, not a session identifier — the
/// catalogue states the literal and the `why` states the reason.
#[test]
fn the_plan_activity_id_is_a_constant() {
    let why = contract_why("json-field", "backend.activity event");
    assert!(why.contains("dedupes by activity.id"), "{why}");

    let mut tracker = ActivityTracker::new();
    let first = tracker
        .project(&RawSessionUpdate::plan(Vec::new()))
        .expect("projects");
    let second = tracker
        .project(&RawSessionUpdate {
            session_update: UPDATE_PLAN.to_owned(),
            entries: vec![PlanEntry {
                content: "step".to_owned(),
                status: PlanEntryStatus::Pending,
            }],
            ..RawSessionUpdate::default()
        })
        .expect("projects");
    assert_eq!(first.id(), second.id());
    assert_eq!(first.id(), Some(PLAN_ACTIVITY_ID));
    // An empty plan is `completed`, as upstream's is.
    assert_eq!(first.status(), ActivityStatus::Completed);
    assert_eq!(second.status(), ActivityStatus::Running);
}

/// A sanity check on the fixture the other tests lean on: every one of the
/// twelve ids in [`upstream_declarations`] is a catalogued backend, and the
/// list is the whole catalog.
#[test]
fn the_upstream_declaration_table_covers_the_whole_catalog() {
    let ids: Vec<&str> = upstream_declarations()
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    assert_eq!(ids.len(), 12);
    let mut catalogued = via_catalog::backend_names();
    let mut declared = ids.clone();
    catalogued.sort_unstable();
    declared.sort_unstable();
    assert_eq!(declared, catalogued);
}

/// `BackendCapabilities` serializes with upstream's key spellings, in
/// upstream's order.
#[test]
fn capabilities_serialize_with_the_catalogued_key_order() {
    let capabilities = BackendCapabilities {
        delegation: true,
        permissions: true,
        backend_ui: false,
        native_session_history: true,
        external_mcp: true,
        native_delegation: false,
        session_mcp: true,
    };
    let rendered = serde_json::to_string(&capabilities).expect("serializable");
    assert_eq!(json_keys(&rendered), CAPABILITY_FLAGS.to_vec());
}
