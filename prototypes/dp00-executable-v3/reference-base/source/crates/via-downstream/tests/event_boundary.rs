//! The progress surface is a boundary. This is the file that proves it.
//!
//! `docs/architecture.md` §6 and upstream's own architecture document agree:
//! *"Session IDs, subagent IDs, raw permission payloads, and raw reasoning are
//! not shown."* A hostile backend is driven at the projection here from every
//! angle it has — an update kind that should be silent, a `rawInput` stuffed
//! with routing internals, unbounded strings, and a merge that tries to
//! smuggle a field in from an earlier update.

mod common;

use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;
use via_downstream::{
    ActivityStatus, ActivityTracker, DETAIL_BOUND, PlanEntry, PlanEntryStatus, RawSessionUpdate,
    SessionEvent, TOOL_LABEL_BOUND, TOOL_NAME_BOUND, ToolCategory, UPDATE_AGENT_MESSAGE_CHUNK,
    UPDATE_PLAN, UPDATE_TOOL_CALL, UPDATE_TOOL_CALL_UPDATE,
};

/// The routing internals a hostile backend would most like to see forwarded.
const SECRETS: [&str; 6] = [
    "sess_01HZZZZZZZZZZZZZZZZZZZZZZZ",
    "subagent_7f3a",
    "auth_deadbeefdeadbeefdeadbeefdeadbeef",
    "the user's private reasoning trace",
    "Bearer sk-live-0000",
    "coordinator:ana:backend",
];

/// A tool call whose `rawInput` carries every internal a backend could try to
/// smuggle out, under the exact key names the adapter's own records use.
fn hostile_update() -> RawSessionUpdate {
    RawSessionUpdate {
        session_update: UPDATE_TOOL_CALL.to_owned(),
        tool_call_id: Some("call-1".to_owned()),
        name: Some("run".to_owned()),
        title: Some("Run".to_owned()),
        status: Some("in_progress".to_owned()),
        raw_input: Some(json!({
            "sessionId": SECRETS[0],
            "session_id": SECRETS[0],
            "childSessionKey": SECRETS[5],
            "subagentId": SECRETS[1],
            "permission": { "id": SECRETS[2], "patterns": ["**"] },
            "reasoning": SECRETS[3],
            "authorization": SECRETS[4],
            "content": SECRETS[3],
        })),
        entries: Vec::new(),
    }
}

/// Nothing a hostile `rawInput` carries reaches the serialized event.
///
/// The projection reads exactly four keys out of `rawInput`
/// (`description`, `query`, `path`, `command`); every other key, including the
/// ones the adapter's own session records use, has no path to the surface.
#[test]
fn no_routing_internal_survives_the_projection() {
    let mut tracker = ActivityTracker::new();
    let event = tracker
        .project(&hostile_update())
        .expect("a tool call projects");
    let rendered = serde_json::to_string(&event).expect("serializable");

    for secret in SECRETS {
        assert!(
            !rendered.contains(secret),
            "`{secret}` reached the progress surface: {rendered}",
        );
    }
    for key in [
        "sessionId",
        "session_id",
        "childSessionKey",
        "subagentId",
        "permission",
        "reasoning",
        "authorization",
        "content",
    ] {
        assert!(
            !rendered.contains(key),
            "the key `{key}` reached the progress surface: {rendered}",
        );
    }
    assert_eq!(
        rendered,
        r#"{"id":"call-1","kind":"tool","tool":"run","label":"Run","status":"in_progress","category":"run","detail":""}"#,
    );
}

/// Reasoning does not reach the surface even as a *shape*: an
/// `agent_thought_chunk` projects to nothing at all, where an
/// `agent_message_chunk` projects to the contentless
/// [`SessionEvent::Text`].
#[test]
fn reasoning_projects_to_nothing_and_text_projects_to_nothing_but_its_shape() {
    let mut tracker = ActivityTracker::new();

    let thought = RawSessionUpdate {
        session_update: "agent_thought_chunk".to_owned(),
        title: Some(SECRETS[3].to_owned()),
        raw_input: Some(json!({ "description": SECRETS[3] })),
        ..RawSessionUpdate::default()
    };
    assert_eq!(tracker.project(&thought), None);

    let message = RawSessionUpdate {
        session_update: UPDATE_AGENT_MESSAGE_CHUNK.to_owned(),
        title: Some(SECRETS[3].to_owned()),
        raw_input: Some(json!({ "description": SECRETS[3] })),
        ..RawSessionUpdate::default()
    };
    let event = tracker
        .project(&message)
        .expect("an agent message projects");
    assert_eq!(event, SessionEvent::Text);
    let rendered = serde_json::to_string(&event).expect("serializable");
    assert!(!rendered.contains(SECRETS[3]), "{rendered}");
}

/// Every update kind that is not a plan, a tool call or an agent message
/// projects to nothing — including the ones a future ACP revision will add.
#[rstest]
#[case("")]
#[case("agent_thought_chunk")]
#[case("user_message_chunk")]
#[case("current_mode_update")]
#[case("available_commands_update")]
#[case("session_config_update")]
#[case("permission_request")]
#[case("tool_call ")]
#[case("TOOL_CALL")]
#[case("some_update_acp_has_not_invented_yet")]
fn silence_is_the_default(#[case] kind: &str) {
    let mut tracker = ActivityTracker::new();
    let update = RawSessionUpdate {
        session_update: kind.to_owned(),
        tool_call_id: Some("call-1".to_owned()),
        name: Some(SECRETS[0].to_owned()),
        title: Some(SECRETS[0].to_owned()),
        status: Some("in_progress".to_owned()),
        raw_input: Some(json!({ "description": SECRETS[0] })),
        entries: vec![PlanEntry {
            content: SECRETS[0].to_owned(),
            status: PlanEntryStatus::InProgress,
        }],
    };
    assert_eq!(tracker.project(&update), None, "`{kind}` should be silent");
}

/// Every user-visible string is bounded, whatever a backend sends.
#[test]
fn every_string_on_the_surface_is_bounded() {
    let long = "x".repeat(10_000);
    let mut tracker = ActivityTracker::new();
    let event = tracker
        .project(&RawSessionUpdate {
            session_update: UPDATE_TOOL_CALL.to_owned(),
            tool_call_id: Some("call-1".to_owned()),
            name: Some(long.clone()),
            title: Some(long.clone()),
            status: Some("pending".to_owned()),
            raw_input: Some(json!({ "description": long })),
            entries: Vec::new(),
        })
        .expect("projects");
    let SessionEvent::Tool(activity) = &event else {
        panic!("expected a tool activity");
    };
    assert_eq!(activity.tool().chars().count(), TOOL_NAME_BOUND);
    assert_eq!(activity.label().chars().count(), TOOL_LABEL_BOUND);
    assert_eq!(activity.detail().chars().count(), DETAIL_BOUND);

    let plan = tracker
        .project(&RawSessionUpdate::plan(vec![PlanEntry {
            content: long,
            status: PlanEntryStatus::InProgress,
        }]))
        .expect("projects");
    let SessionEvent::Plan(activity) = &plan else {
        panic!("expected a plan activity");
    };
    assert_eq!(activity.detail().chars().count(), DETAIL_BOUND);
}

/// A tool call id is not bounded upstream and is not bounded here — it is the
/// dedupe key, so truncating it would merge two distinct calls. It is
/// nonetheless trimmed, and an empty one becomes `null` rather than `""`.
#[rstest]
#[case(Some("call-1"), Some("call-1"))]
#[case(Some("  call-1  "), Some("call-1"))]
#[case(Some(""), None)]
#[case(Some("   "), None)]
#[case(None, None)]
fn the_tool_call_id_is_trimmed_and_optional(
    #[case] sent: Option<&str>,
    #[case] expected: Option<&str>,
) {
    let mut tracker = ActivityTracker::new();
    let event = tracker
        .project(&RawSessionUpdate {
            session_update: UPDATE_TOOL_CALL.to_owned(),
            tool_call_id: sent.map(str::to_owned),
            ..RawSessionUpdate::default()
        })
        .expect("projects");
    assert_eq!(event.id(), expected);
}

/// The `{...known, ...update}` merge: a later update inherits what it omits
/// and overrides what it carries.
#[test]
fn a_tool_call_update_inherits_the_announcement() {
    let mut tracker = ActivityTracker::new();
    tracker
        .project(&RawSessionUpdate {
            session_update: UPDATE_TOOL_CALL.to_owned(),
            tool_call_id: Some("call-1".to_owned()),
            name: Some("bash".to_owned()),
            title: Some("Run the build".to_owned()),
            status: Some("pending".to_owned()),
            raw_input: Some(json!({ "command": "cargo build" })),
            entries: Vec::new(),
        })
        .expect("projects");
    assert_eq!(tracker.len(), 1);

    let update = tracker
        .project(&RawSessionUpdate {
            session_update: UPDATE_TOOL_CALL_UPDATE.to_owned(),
            tool_call_id: Some("call-1".to_owned()),
            status: Some("in_progress".to_owned()),
            ..RawSessionUpdate::default()
        })
        .expect("projects");
    let SessionEvent::Tool(activity) = &update else {
        panic!("expected a tool activity");
    };
    assert_eq!(activity.tool(), "bash");
    assert_eq!(activity.label(), "Run the build");
    assert_eq!(activity.detail(), "cargo build");
    assert_eq!(activity.status(), ActivityStatus::InProgress);
}

/// The merge cannot reach across tool call ids: one call's `rawInput` never
/// becomes another's.
#[test]
fn the_merge_is_keyed_and_cannot_cross_tool_calls() {
    let mut tracker = ActivityTracker::new();
    tracker
        .project(&RawSessionUpdate {
            session_update: UPDATE_TOOL_CALL.to_owned(),
            tool_call_id: Some("call-1".to_owned()),
            raw_input: Some(json!({ "command": SECRETS[0] })),
            ..RawSessionUpdate::default()
        })
        .expect("projects");
    let other = tracker
        .project(&RawSessionUpdate {
            session_update: UPDATE_TOOL_CALL_UPDATE.to_owned(),
            tool_call_id: Some("call-2".to_owned()),
            status: Some("in_progress".to_owned()),
            ..RawSessionUpdate::default()
        })
        .expect("projects");
    let rendered = serde_json::to_string(&other).expect("serializable");
    assert!(!rendered.contains(SECRETS[0]), "{rendered}");
}

/// A terminal status makes the tracker forget the call, so a later update with
/// the same id starts clean. `['completed','failed'].includes(status)`.
#[rstest]
#[case("completed", true)]
#[case("failed", true)]
#[case("pending", false)]
#[case("in_progress", false)]
#[case("running", false)]
#[case("who_knows", false)]
fn a_terminal_status_clears_the_tracker(#[case] status: &str, #[case] forgotten: bool) {
    let mut tracker = ActivityTracker::new();
    tracker
        .project(&RawSessionUpdate {
            session_update: UPDATE_TOOL_CALL.to_owned(),
            tool_call_id: Some("call-1".to_owned()),
            name: Some("bash".to_owned()),
            status: Some(status.to_owned()),
            ..RawSessionUpdate::default()
        })
        .expect("projects");
    assert_eq!(tracker.is_empty(), forgotten, "status `{status}`");
}

/// An unrecognised status becomes `running` rather than being forwarded — the
/// one deviation on this surface, and the reason for it: the status is
/// model-visible, so an attacker-chosen string must not reach it.
#[test]
fn an_unrecognised_status_becomes_running() {
    let mut tracker = ActivityTracker::new();
    let event = tracker
        .project(&RawSessionUpdate {
            session_update: UPDATE_TOOL_CALL.to_owned(),
            tool_call_id: Some("call-1".to_owned()),
            status: Some(format!("completed\"}}, \"leak\": \"{}\"", SECRETS[0])),
            ..RawSessionUpdate::default()
        })
        .expect("projects");
    assert_eq!(event.status(), ActivityStatus::Running);
    let rendered = serde_json::to_string(&event).expect("serializable");
    assert!(!rendered.contains(SECRETS[0]), "{rendered}");
    assert!(rendered.contains(r#""status":"running""#), "{rendered}");
}

/// `categoryForTool`'s five groups, in upstream's precedence order, over
/// upstream's own needles.
#[rstest]
#[case("generate_image", "", ToolCategory::Image)]
#[case("", "\u{7ed8}\u{56fe}", ToolCategory::Image)]
#[case("web_search", "", ToolCategory::Search)]
#[case("fetch", "", ToolCategory::Search)]
#[case("read_file", "", ToolCategory::Read)]
#[case("glob", "", ToolCategory::Read)]
#[case("write_file", "", ToolCategory::Write)]
#[case("apply_patch", "", ToolCategory::Write)]
#[case("bash", "", ToolCategory::Run)]
#[case("", "", ToolCategory::Run)]
// Precedence: an "image" needle beats a "read" needle in the same hint.
#[case("read_image", "", ToolCategory::Image)]
// Chinese needles are matched too.
#[case("", "\u{8bfb}\u{53d6}\u{6587}\u{4ef6}", ToolCategory::Read)]
fn tool_categories_follow_upstreams_precedence(
    #[case] name: &str,
    #[case] title: &str,
    #[case] expected: ToolCategory,
) {
    assert_eq!(ToolCategory::classify(name, title, None), expected);
}

/// The hint includes `JSON.stringify(rawInput)`, so a tool named opaquely is
/// still classified from its arguments.
#[test]
fn the_category_hint_includes_the_raw_input() {
    assert_eq!(
        ToolCategory::classify("tool", "", Some(&json!({ "command": "grep -r via" }))),
        ToolCategory::Read,
    );
    assert_eq!(
        ToolCategory::classify("tool", "", Some(&json!({ "path": "/etc" }))),
        ToolCategory::Run,
    );
    // No `rawInput` at all is upstream's `JSON.stringify({})`, not a crash.
    assert_eq!(ToolCategory::classify("tool", "", None), ToolCategory::Run);
}

/// The `detail` chain falls through *falsy* values the way JavaScript's `||`
/// does, so `{"description": ""}` does not shadow a real `command`.
#[rstest]
#[case(json!({ "description": "d", "query": "q", "path": "p", "command": "c" }), "d")]
#[case(json!({ "description": "", "query": "q" }), "q")]
#[case(json!({ "description": null, "path": "p" }), "p")]
#[case(json!({ "description": false, "command": "c" }), "c")]
#[case(json!({ "query": 0, "command": "c" }), "c")]
#[case(json!({ "query": 7 }), "7")]
#[case(json!({}), "")]
fn the_detail_chain_skips_falsy_candidates(
    #[case] raw_input: serde_json::Value,
    #[case] expected: &str,
) {
    let mut tracker = ActivityTracker::new();
    let event = tracker
        .project(&RawSessionUpdate {
            session_update: UPDATE_TOOL_CALL.to_owned(),
            raw_input: Some(raw_input),
            ..RawSessionUpdate::default()
        })
        .expect("projects");
    let SessionEvent::Tool(activity) = &event else {
        panic!("expected a tool activity");
    };
    assert_eq!(activity.detail(), expected);
}

/// `tool` falls back to the literal `'tool'` and `label` to the empty string,
/// exactly as upstream does.
#[test]
fn the_tool_name_has_a_fallback_and_the_label_does_not() {
    let mut tracker = ActivityTracker::new();
    let event = tracker
        .project(&RawSessionUpdate {
            session_update: UPDATE_TOOL_CALL.to_owned(),
            ..RawSessionUpdate::default()
        })
        .expect("projects");
    let SessionEvent::Tool(activity) = &event else {
        panic!("expected a tool activity");
    };
    assert_eq!(activity.tool(), via_downstream::DEFAULT_TOOL_NAME);
    assert_eq!(activity.label(), "");
    assert_eq!(activity.detail(), "");
    assert_eq!(activity.status(), ActivityStatus::Running);
}

/// A plan reports progress, never its entries' identities.
#[test]
fn a_plan_reports_counts_and_one_bounded_line() {
    let mut tracker = ActivityTracker::new();
    let event = tracker
        .project(&RawSessionUpdate {
            session_update: UPDATE_PLAN.to_owned(),
            entries: vec![
                PlanEntry {
                    content: "one".to_owned(),
                    status: PlanEntryStatus::Completed,
                },
                PlanEntry {
                    content: "two".to_owned(),
                    status: PlanEntryStatus::Completed,
                },
                PlanEntry {
                    content: SECRETS[0].to_owned(),
                    status: PlanEntryStatus::Pending,
                },
                PlanEntry {
                    content: "four".to_owned(),
                    status: PlanEntryStatus::InProgress,
                },
            ],
            ..RawSessionUpdate::default()
        })
        .expect("projects");
    let SessionEvent::Plan(activity) = &event else {
        panic!("expected a plan activity");
    };
    // `in_progress` wins over `pending`, so the pending entry's text — which is
    // where the secret was put — never reaches the surface.
    assert_eq!(activity.detail(), "four");
    assert_eq!(activity.completed(), 2);
    assert_eq!(activity.total(), 4);
    assert_eq!(activity.status(), ActivityStatus::Running);
}

/// The tracker's lifetime is the caller's to end.
#[test]
fn clearing_the_tracker_forgets_everything() {
    let mut tracker = ActivityTracker::new();
    for index in 0..5 {
        tracker
            .project(&RawSessionUpdate::tool_call(&format!("call-{index}")))
            .expect("projects");
    }
    assert_eq!(tracker.len(), 5);
    tracker.clear();
    assert!(tracker.is_empty());
}
