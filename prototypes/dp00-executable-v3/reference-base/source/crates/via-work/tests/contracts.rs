//! Every contract value this crate owns, asserted against
//! `docs/reference/contracts.json`.
//!
//! Nothing here retypes a catalogue value: the field orders, the vocabularies,
//! the defaults, the retention numbers and the two restart sentences are all
//! scanned out of the catalogue's own prose and compared against what the
//! shipped types produce.

mod common;

use common::{
    bar_list, clause_with, contract_value, contract_why, json_keys, numbers_in, quoted_tokens,
    settle,
};
use pretty_assertions::assert_eq;
use serde_json::json;
use std::sync::Arc;
use via_i18n::{Locale, keys, t};
use via_protocol::WorkKind;
use via_work::activity::ACTIVITY_RING;
use via_work::delegation::{DELEGATION_SPEECH_BOUND, DELEGATION_TITLE_BOUND, DelegationRef};
use via_work::limits::{
    DEFAULT_MAX_TERMINAL_TASKS_PER_OWNER, DEFAULT_NOTIFICATION_CLAIM_TTL_MS,
    DEFAULT_PENDING_NOTIFICATION_TTL_MS, DEFAULT_REMINDER_MAX_PER_OWNER,
    DEFAULT_REMINDER_STAGGER_MS, DEFAULT_SCHEDULED_TASK_TIMEOUT_MS, DEFAULT_TERMINAL_TTL_MS,
    MAX_REMINDER_MAX_PER_OWNER, MAX_REMINDER_STAGGER_MS, MIN_MAX_TERMINAL_TASKS_PER_OWNER,
    MIN_NOTIFICATION_CLAIM_TTL_MS, MIN_PENDING_NOTIFICATION_TTL_MS, MIN_REMINDER_MAX_PER_OWNER,
    MIN_REMINDER_STAGGER_MS, MIN_SCHEDULED_TASK_TIMEOUT_MS, MIN_TERMINAL_TTL_MS,
    PROGRESS_HEARTBEAT_MS, SCHEDULED_TASK_CLEANUP_MS,
};
use via_work::presentation::INLINE_TITLE_BOUND;
use via_work::progress::{DEFAULT_PROGRESS_CHECK_MS, MIN_PROGRESS_CHECK_MS};
use via_work::record::WORK_ID_PREFIX;
use via_work::scheduler::{
    COORDINATOR_LANE_LIMIT, DEFAULT_MAX_CONCURRENT, DEFAULT_MAX_CONCURRENT_PER_OWNER,
};
use via_work::store::DEFERRED_DELAY;
use via_work::testing::{ImmediateRunner, blank_record};
use via_work::{
    LOG_INFO_EVENT_NAMES, NewWork, NotificationStatus, PendingPermission, WorkEventKind,
    WorkManager, WorkRecord, new_work_id,
};

// ── the record's two shapes ────────────────────────────────────────────────

/// `json-field` / *publicTask / Work record*.
///
/// The catalogue states the field list; it is scanned out and compared against
/// a real `serde_json` rendering, so a reordered or renamed field fails here
/// rather than in a client.
#[test]
fn public_work_publishes_the_catalogued_fields_in_order() {
    let stated = contract_value(
        "json-field",
        "publicTask / Work record (HTTP GET /api/tasks, /api/tasks/:id, SSE /api/tasks/:id/events, WS task events)",
    );
    let expected: Vec<String> = stated
        .trim_matches(|character| character == '{' || character == '}' || character == ' ')
        .split(',')
        .map(|field| field.trim().to_owned())
        .filter(|field| !field.is_empty())
        .collect();

    let mut record = blank_record("work_one", "owner");
    // `notificationDeliveredAt` is `undefined` until a delivery happens, and
    // `JSON.stringify` omits it; set it so the full list is comparable.
    record.notification_delivered_at = Some(7);
    let rendered = serde_json::to_value(record.to_public(0)).expect("serializes");
    let shipped: Vec<String> = rendered
        .as_object()
        .expect("an object")
        .keys()
        .cloned()
        .collect();

    assert_eq!(expected, shipped, "publicTask field order");
    assert_eq!(shipped.len(), 25);

    // `notificationDeliveredAt` is `undefined` until a delivery happens, and
    // `JSON.stringify` omits an undefined key. The field is therefore *absent*
    // from a fresh record's payload rather than `null` — 24 keys, not 25.
    let fresh =
        serde_json::to_value(blank_record("work_one", "owner").to_public(0)).expect("serializes");
    let fresh_keys: Vec<String> = fresh
        .as_object()
        .expect("an object")
        .keys()
        .cloned()
        .collect();
    assert_eq!(fresh_keys.len(), 24);
    assert!(
        !fresh_keys.contains(&"notificationDeliveredAt".to_owned()),
        "{fresh}"
    );
    assert_eq!(
        expected
            .iter()
            .filter(|field| *field != "notificationDeliveredAt")
            .cloned()
            .collect::<Vec<String>>(),
        fresh_keys,
    );
}

/// The same entry's `why` states three properties; each is asserted.
#[test]
fn public_work_keeps_the_three_stated_properties() {
    let why = contract_why(
        "json-field",
        "publicTask / Work record (HTTP GET /api/tasks, /api/tasks/:id, SSE /api/tasks/:id/events, WS task events)",
    );
    assert!(why.contains("workId is a duplicate of id"), "{why}");
    assert!(why.contains("'active' for any ACTIVE status"), "{why}");
    assert!(why.contains("elapsedMs is computed live"), "{why}");

    let mut record = blank_record("work_one", "owner");
    record.status = via_protocol::WorkStatus::Running;
    record.started_at = Some(1_000);
    record.elapsed_ms = 42;
    let public = record.to_public(4_000);
    assert_eq!(public.work_id, public.id);
    assert_eq!(public.work_state, via_protocol::WorkState::Active);
    assert_eq!(public.elapsed_ms, 3_000);

    record.status = via_protocol::WorkStatus::Completed;
    let public = record.to_public(4_000);
    assert_eq!(public.work_state, via_protocol::WorkState::Completed);
    assert_eq!(public.elapsed_ms, 42, "frozen once terminal");
}

/// `json-field` / *persisted task record (persistedTask)*.
#[test]
fn persisted_work_is_public_minus_two_plus_submission_key() {
    let stated = contract_value("json-field", "persisted task record (persistedTask)");
    assert!(stated.contains("MINUS workId and workState"), "{stated}");
    assert!(stated.contains("PLUS submissionKey"), "{stated}");
    assert!(stated.contains("delegation deep-copied"), "{stated}");

    let mut record = blank_record("work_one", "owner");
    record.notification_delivered_at = Some(7);
    record.delegation = Some(DelegationRef::new("run-one", "target-one"));
    let public = serde_json::to_value(record.to_public(0)).expect("serializes");
    let persisted = serde_json::to_value(record.to_persisted(0)).expect("serializes");

    let public_keys: Vec<String> = public
        .as_object()
        .expect("object")
        .keys()
        .cloned()
        .collect();
    let persisted_keys: Vec<String> = persisted
        .as_object()
        .expect("object")
        .keys()
        .cloned()
        .collect();
    let expected: Vec<String> = public_keys
        .into_iter()
        .filter(|key| key != "workId" && key != "workState")
        .chain(std::iter::once("submissionKey".to_owned()))
        .collect();
    assert_eq!(persisted_keys, expected);

    // The deep-copied delegation keeps what the public one drops.
    assert_eq!(persisted["delegation"]["id"], json!("run-one"));
    assert_eq!(persisted["delegation"]["sessionId"], json!("target-one"));
}

/// The same entry names nine fields that must **not** survive a restart.
#[test]
fn the_nine_unpersisted_fields_are_absent_from_disk() {
    let stated = contract_value("json-field", "persisted task record (persistedTask)");
    let not_persisted = common::segment_after(&stated, "NOT persisted:", '.');
    let names: Vec<&str> = not_persisted
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect();
    assert_eq!(names.len(), 9, "{not_persisted}");

    let mut record = blank_record("work_one", "owner");
    record.priority = 100;
    record.lane_key = Some("coordinator:owner".to_owned());
    record.notification_claimant_id = Some("client".to_owned());
    record.notification_claimed_at = Some(9);
    let persisted = serde_json::to_string(&record.to_persisted(0)).expect("serializes");
    for name in names {
        assert!(
            !persisted.contains(&format!("\"{name}\"")),
            "`{name}` must not reach tasks.json: {persisted}",
        );
    }
}

/// `json-field` / *Work id format*.
#[test]
fn work_ids_are_the_catalogued_template() {
    let stated = contract_value("json-field", "Work id format");
    assert!(stated.contains("work_<uuidv4>"), "{stated}");
    assert_eq!(WORK_ID_PREFIX, "work_");
    let id = new_work_id();
    assert!(id.starts_with(WORK_ID_PREFIX));
    let uuid = id.trim_start_matches(WORK_ID_PREFIX);
    assert_eq!(uuid.len(), 36, "a dashed UUID");
    assert_eq!(uuid, uuid.to_lowercase());
}

/// `json-field` / *Work kind values*, and the three properties its `why`
/// states.
#[test]
fn the_four_kinds_and_their_three_stated_properties() {
    let stated = contract_value("json-field", "Work kind values");
    let expected = bar_list(&stated);
    let shipped: Vec<String> = WorkKind::ALL
        .iter()
        .map(|kind| kind.as_str().to_owned())
        .collect();
    assert_eq!(expected, shipped);

    let why = contract_why("json-field", "Work kind values");
    assert!(why.contains("'control' is hidden from list()"), "{why}");
    assert!(
        why.contains("only kind that gets a progressCheckMs"),
        "{why}"
    );
    assert!(
        why.contains("only kind that gets a timeout watchdog"),
        "{why}"
    );
    assert!(!WorkKind::Control.is_client_visible());
}

/// `json-field` / *resultMetadata (publicResultMetadata)*.
#[test]
fn result_metadata_publishes_only_the_presentation() {
    let stated = contract_value("json-field", "resultMetadata (publicResultMetadata)");
    assert!(stated.contains("sliced to 120"), "{stated}");
    assert_eq!(INLINE_TITLE_BOUND, 120);
    let formats = quoted_tokens(&stated);
    assert!(formats.contains(&"markdown".to_owned()), "{stated}");
    assert!(formats.contains(&"code".to_owned()), "{stated}");
    assert!(formats.contains(&"link".to_owned()), "{stated}");
    assert!(
        stated.contains("metadata.decision.presentation"),
        "the legacy fallback must survive: {stated}",
    );

    let mut record = blank_record("work_one", "owner");
    record.result_metadata = Some(json!({
        "decision": {"presentation": {"speech": "s", "inline": null}},
        "backendRef": {"directory": "/private/project"},
    }));
    let rendered = serde_json::to_value(record.to_public(0)).expect("serializes");
    assert_eq!(
        rendered["resultMetadata"],
        json!({"presentation": {"speech": "s", "inline": null}}),
    );
}

/// `json-field` / *task resultMetadata projection* — the information-disclosure
/// boundary.
#[test]
fn a_backend_reference_reaches_neither_the_wire_nor_the_disk() {
    let why = contract_why("json-field", "task resultMetadata projection");
    assert!(why.contains("information-disclosure boundary"), "{why}");
    let stated = contract_value("json-field", "task resultMetadata projection");
    assert!(
        stated.contains("backendRef and delegation are stripped"),
        "{stated}"
    );

    let mut record = blank_record("work_one", "owner");
    record.result_metadata = Some(json!({
        "presentation": {"speech": "done", "inline": null},
        "backendRef": {"sessionId": "s", "directory": "/private/project"},
        "delegation": {"id": "run", "sessionId": "target"},
    }));
    for rendered in [
        serde_json::to_string(&record.to_public(0)).expect("serializes"),
        serde_json::to_string(&record.to_persisted(0)).expect("serializes"),
    ] {
        assert!(!rendered.contains("/private/project"), "{rendered}");
        assert!(!rendered.contains("backendRef"), "{rendered}");
    }
}

/// `json-field` / *delegation (public form)* — the two slicing lengths.
#[test]
fn the_public_delegation_slices_at_the_catalogued_lengths() {
    let stated = contract_value("json-field", "delegation (public form)");
    let bounds = numbers_in(&stated);
    assert!(bounds.contains(&160), "{stated}");
    assert!(bounds.contains(&1200), "{stated}");
    assert_eq!(DELEGATION_TITLE_BOUND, 160);
    assert_eq!(DELEGATION_SPEECH_BOUND, 1200);
    assert!(stated.contains("default 'running'"), "{stated}");

    let public = DelegationRef::new("run", "target")
        .with_title(&"t".repeat(400))
        .with_presentation(json!({"speech": "s".repeat(2000), "inline": null}))
        .to_public();
    assert_eq!(public.status, "running");
    assert_eq!(public.title.chars().count(), DELEGATION_TITLE_BOUND);
    assert_eq!(
        public.presentation.expect("present").speech.chars().count(),
        DELEGATION_SPEECH_BOUND,
    );
}

/// `json-field` / *schedule object*.
#[test]
fn the_schedule_object_is_the_catalogued_shape() {
    let stated = contract_value("json-field", "schedule object");
    let keys = json_keys(&format!("{{{}}}", stated.replace(':', "\":")));
    let _ = keys;
    assert!(stated.contains("type: 'at'"), "{stated}");
    assert!(stated.contains("recurrence"), "{stated}");
    assert!(stated.contains("default 'once'"), "{stated}");

    let rendered = serde_json::to_value(via_work::Schedule::at(7)).expect("serializes");
    assert_eq!(
        rendered,
        json!({"type": "at", "at": 7, "recurrence": "once"})
    );
}

// ── the event vocabulary ───────────────────────────────────────────────────

/// `ws-event` / *TaskManager event type names*.
#[test]
fn the_sixteen_event_names_are_the_catalogued_ones_in_order() {
    let stated = contract_value("ws-event", "TaskManager event type names");
    let expected: Vec<String> = stated
        .split(';')
        .next()
        .unwrap_or_default()
        .split('|')
        .map(|name| name.trim().to_owned())
        .filter(|name| name.starts_with("task."))
        .collect();
    let shipped: Vec<String> = WorkEventKind::ALL
        .iter()
        .map(|kind| kind.as_str().to_owned())
        .collect();
    assert_eq!(expected, shipped);
    assert_eq!(shipped.len(), 16);
}

/// The same entry's `why` pins the info/debug split and the never-emitted name.
#[test]
fn the_log_split_is_the_catalogued_list() {
    let why = contract_why("ws-event", "TaskManager event type names");
    let info_clause = common::segment_after(&why, "The info-vs-debug log split is:", ';');
    let expected: Vec<&str> = info_clause
        .split(',')
        .map(|token| token.trim().trim_end_matches(" log at info"))
        .filter(|token| token.starts_with("task."))
        .collect();
    assert_eq!(expected, LOG_INFO_EVENT_NAMES);
    assert!(
        why.contains("'task.created' appears in that log list but is never emitted"),
        "{why}",
    );
    assert!(LOG_INFO_EVENT_NAMES.contains(&"task.created"));
    assert!(WorkEventKind::from_wire("task.created").is_none());
}

/// `json-field` / *TaskManager event envelope*.
#[tokio::test(start_paused = true)]
async fn the_event_envelope_is_the_catalogued_shape() {
    let stated = contract_value("json-field", "TaskManager event envelope");
    assert!(stated.contains("{ type, ownerId, task:"), "{stated}");
    assert!(stated.contains("{ message, delegated }"), "{stated}");
    assert!(stated.contains("{ permission }"), "{stated}");

    let manager = WorkManager::builder()
        .now(common::clock())
        .runner(Arc::new(ImmediateRunner::completing("done")))
        .build();
    let mut events = common::Events::new(manager.subscribe());
    let accepted = manager
        .create(NewWork::new("objective", "owner"))
        .await
        .expect("accepted");
    manager.wait(&accepted.work.id).await;
    settle().await;

    let event = events.first(WorkEventKind::Completed).expect("completed");
    let rendered = serde_json::to_value(&event).expect("serializes");
    let keys: Vec<&String> = rendered.as_object().expect("object").keys().collect();
    assert_eq!(keys, ["type", "ownerId", "task"]);
    assert_eq!(rendered["type"], json!("task.completed"));
    assert_eq!(rendered["ownerId"], json!("owner"));

    // The two shapes that add fields.
    let permission = PendingPermission::pending("auth_one", "bash", "bash：npm test");
    let resolved = via_work::WorkEvent::permission_resolved(event.task.clone(), permission);
    let rendered = serde_json::to_value(&resolved).expect("serializes");
    assert!(rendered.get("permission").is_some(), "{rendered}");

    let checked = via_work::WorkEvent::progress_check(event.task, "message".to_owned(), true);
    let rendered = serde_json::to_value(&checked).expect("serializes");
    assert_eq!(rendered["message"], json!("message"));
    assert_eq!(rendered["delegated"], json!(true));
}

// ── the two restart sentences ──────────────────────────────────────────────

/// `prompt-text` / *restart force-fail error (interactive work)*.
///
/// The catalogued value carries the upstream product name; `docs/rebrand.md`
/// renames it, so the assertion is on the Chinese prose around it, which is
/// byte for byte upstream's.
#[test]
fn the_interactive_restart_sentence_is_upstreams_prose() {
    let stated = contract_value("prompt-text", "restart force-fail error (interactive work)");
    let shipped = t(Locale::Zh, keys::WORK_RESTART_INTERACTIVE_INCOMPLETE);
    assert_eq!(
        stated.replace("qwen-audio-agent", "VIA"),
        shipped,
        "only the product name is renamed",
    );
    assert!(shipped.starts_with("VIA "), "{shipped}");
    assert!(!shipped.contains("qwen"), "{shipped}");
}

/// `prompt-text` / *restart force-fail error (unrecoverable delegated work)*.
#[test]
fn the_delegated_restart_sentence_is_upstreams_prose_and_differs() {
    let stated = contract_value(
        "prompt-text",
        "restart force-fail error (unrecoverable delegated work)",
    );
    let shipped = t(Locale::Zh, keys::WORK_RESTART_DELEGATED_LOST);
    assert_eq!(stated.replace("qwen-audio-agent", "VIA"), shipped);

    let interactive = t(Locale::Zh, keys::WORK_RESTART_INTERACTIVE_INCOMPLETE);
    assert_ne!(
        shipped, interactive,
        "the two restart reasons must stay distinguishable",
    );
    assert!(
        shipped.contains("项目任务") && shipped.contains("失去连接"),
        "the delegated one names the lost connection: {shipped}",
    );
    assert!(
        interactive.contains("尚未完成"),
        "the interactive one names unfinished work: {interactive}",
    );

    for locale in [Locale::En, Locale::Ko] {
        assert_ne!(
            t(locale, keys::WORK_RESTART_DELEGATED_LOST),
            t(locale, keys::WORK_RESTART_INTERACTIVE_INCOMPLETE),
            "{locale}: the two must differ in every locale",
        );
    }
}

/// `prompt-text` / *scheduled-task timeout errors*.
#[test]
fn the_scheduled_timeout_strings_are_the_catalogued_ones() {
    let stated = contract_value("prompt-text", "scheduled-task timeout errors");
    let abort = stated
        .split("abort reason:")
        .nth(1)
        .and_then(|rest| rest.split('|').next())
        .map(str::trim)
        .expect("an abort reason");
    assert_eq!(abort, t(Locale::Zh, keys::WORK_SCHEDULED_TIMEOUT_ABORT));
    assert!(stated.contains("Math.round"), "{stated}");

    let rendered = via_i18n::format(
        Locale::Zh,
        keys::WORK_SCHEDULED_TIMEOUT_ERROR,
        &[("minutes", "30")],
    );
    assert_eq!(rendered, "定时任务执行超时（30 分钟）");
}

/// `prompt-text` / *cancellation strings*.
#[test]
fn the_cancellation_strings_are_the_catalogued_ones() {
    let stated = contract_value("prompt-text", "cancellation strings");
    let clauses: Vec<&str> = stated.split('|').map(str::trim).collect();
    let abort = clauses[0]
        .trim_start_matches("abort reason:")
        .trim()
        .to_owned();
    assert_eq!(abort, t(Locale::Zh, keys::WORK_CANCELLED_BY_USER));

    let missing = clauses
        .iter()
        .find(|clause| clause.starts_with("missing runner:"))
        .expect("a missing-runner clause")
        .trim_start_matches("missing runner:")
        .trim();
    assert_eq!(missing, t(Locale::Zh, keys::WORK_NO_RUNNER_CONFIGURED));

    assert!(stated.contains("取消失败："), "{stated}");
    assert_eq!(
        via_i18n::format(Locale::Zh, keys::WORK_CANCEL_FAILED, &[("detail", "boom")]),
        "取消失败：boom",
    );
}

/// `prompt-text` / *background work progress-check message*, including the
/// verb table and the literal curly quotes.
#[test]
fn the_progress_check_message_is_the_catalogued_template() {
    let stated = contract_value("prompt-text", "background work progress-check message");
    let why = contract_why("prompt-text", "background work progress-check message");
    assert!(why.contains("curly quotes"), "{why}");

    for (category, verb) in [
        ("run", "执行"),
        ("read", "读取"),
        ("write", "修改"),
        ("search", "搜索"),
        ("image", "生成图片"),
    ] {
        assert!(
            stated.contains(&format!("{category}:'{verb}'")),
            "{category} -> {verb} is not in {stated}",
        );
        assert_eq!(
            via_work::progress::activity_verb(Locale::Zh, Some(category)),
            verb,
        );
    }
    assert!(stated.contains("|| '处理'"), "{stated}");
    assert_eq!(via_work::progress::activity_verb(Locale::Zh, None), "处理");

    let message = via_work::progress_message(Locale::Zh, "objective", Some(1), 1, None);
    assert!(message.contains('“') && message.contains('”'), "{message}");
    assert!(
        stated.contains("objective.slice(0,80)"),
        "the quote bound: {stated}",
    );
}

// ── defaults ───────────────────────────────────────────────────────────────

/// `default-value` / *TaskManager / TaskScheduler / TaskStore defaults*.
#[test]
fn every_shipped_default_is_the_catalogued_one() {
    let stated = contract_value(
        "default-value",
        "TaskManager / TaskScheduler / TaskStore defaults",
    );
    let named = |needle: &str| -> i64 {
        let clause = stated
            .split(',')
            .find(|clause| clause.contains(needle))
            .unwrap_or_else(|| panic!("no clause mentions `{needle}` in {stated}"));
        *numbers_in(clause)
            .last()
            .unwrap_or_else(|| panic!("no number in `{clause}`"))
    };

    assert_eq!(named("maxConcurrent="), DEFAULT_MAX_CONCURRENT as i64);
    assert_eq!(
        named("maxConcurrentPerOwner="),
        DEFAULT_MAX_CONCURRENT_PER_OWNER as i64
    );
    assert_eq!(named("terminalTtlMs="), DEFAULT_TERMINAL_TTL_MS);
    assert_eq!(
        named("pendingNotificationTtlMs="),
        DEFAULT_PENDING_NOTIFICATION_TTL_MS
    );
    assert_eq!(
        named("notificationClaimTtlMs="),
        DEFAULT_NOTIFICATION_CLAIM_TTL_MS
    );
    assert_eq!(
        named("maxTerminalTasksPerOwner="),
        DEFAULT_MAX_TERMINAL_TASKS_PER_OWNER
    );
    assert_eq!(
        named("deferredDelayMs="),
        i64::try_from(DEFERRED_DELAY.as_millis()).expect("fits"),
    );
    assert_eq!(named("laneLimit fallback="), COORDINATOR_LANE_LIMIT as i64);
    assert_eq!(named("progress heartbeat interval="), PROGRESS_HEARTBEAT_MS);
    assert_eq!(
        named("cleanup window after abort="),
        SCHEDULED_TASK_CLEANUP_MS
    );
    assert_eq!(named("activity ring="), ACTIVITY_RING as i64);
    assert!(
        stated.contains("progressCheckMs=config.backgroundTaskProgressCheckMs (300_000)"),
        "{stated}",
    );
    assert_eq!(DEFAULT_PROGRESS_CHECK_MS, 300_000);
}

/// `env-var` / *Work-subsystem environment variables* — the defaults and the
/// clamps, read out of the catalogue's own prose.
#[test]
fn every_environment_default_and_clamp_matches() {
    let stated = contract_value("env-var", "Work-subsystem environment variables");
    let numbers = |needle: &str| -> Vec<i64> { numbers_in(&clause_with(&stated, needle)) };

    assert_eq!(
        numbers("TASK_TERMINAL_TTL_MS"),
        [DEFAULT_TERMINAL_TTL_MS, MIN_TERMINAL_TTL_MS]
    );
    assert_eq!(
        numbers("TASK_NOTIFICATION_TTL_MS"),
        [
            DEFAULT_PENDING_NOTIFICATION_TTL_MS,
            MIN_PENDING_NOTIFICATION_TTL_MS
        ]
    );
    assert_eq!(
        numbers("TASK_NOTIFICATION_CLAIM_TTL_MS"),
        [
            DEFAULT_NOTIFICATION_CLAIM_TTL_MS,
            MIN_NOTIFICATION_CLAIM_TTL_MS
        ]
    );
    assert_eq!(
        numbers("MAX_TERMINAL_TASKS_PER_OWNER"),
        [
            DEFAULT_MAX_TERMINAL_TASKS_PER_OWNER,
            MIN_MAX_TERMINAL_TASKS_PER_OWNER
        ]
    );
    assert_eq!(
        numbers("REMINDER_MAX_PER_OWNER"),
        [
            DEFAULT_REMINDER_MAX_PER_OWNER,
            MIN_REMINDER_MAX_PER_OWNER,
            MAX_REMINDER_MAX_PER_OWNER
        ]
    );
    assert_eq!(
        numbers("REMINDER_STAGGER_MS"),
        [
            DEFAULT_REMINDER_STAGGER_MS,
            MIN_REMINDER_STAGGER_MS,
            MAX_REMINDER_STAGGER_MS
        ]
    );
    assert_eq!(
        numbers("SCHEDULED_TASK_TIMEOUT_MS"),
        [
            DEFAULT_SCHEDULED_TASK_TIMEOUT_MS,
            MIN_SCHEDULED_TASK_TIMEOUT_MS
        ]
    );
    assert_eq!(
        numbers("BACKGROUND_TASK_PROGRESS_CHECK_MS"),
        [DEFAULT_PROGRESS_CHECK_MS, MIN_PROGRESS_CHECK_MS]
    );

    // The two admission caps carry a ceiling as well as a floor.
    let concurrent = numbers("TASK_MAX_CONCURRENT ");
    assert_eq!(concurrent[0], DEFAULT_MAX_CONCURRENT as i64);
    assert_eq!(concurrent[1..], [1, 64]);
    let per_owner = numbers("TASK_MAX_CONCURRENT_PER_OWNER");
    assert_eq!(per_owner[0], DEFAULT_MAX_CONCURRENT_PER_OWNER as i64);
    assert_eq!(per_owner[1..], [1, 16]);
}

// ── the store ──────────────────────────────────────────────────────────────

/// `file-path` / *tasks.json on-disk format*.
#[test]
fn tasks_json_is_the_catalogued_document() {
    let stated = contract_value("file-path", "tasks.json on-disk format");
    assert!(stated.contains("version: 1"), "{stated}");
    assert!(stated.contains("2-space indent"), "{stated}");
    assert!(stated.contains("trailing newline"), "{stated}");
    assert!(stated.contains("0o600"), "{stated}");
    assert!(
        stated.contains("${filePath}.${process.pid}.tmp"),
        "{stated}"
    );
    assert!(
        stated.contains("${filePath}.${process.pid}.${generation}.tmp"),
        "{stated}",
    );

    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("tasks.json");
    let store = via_work::WorkStore::builder().file_path(&path).build();
    let record = WorkRecord {
        result: Some("done".to_owned()),
        ..blank_record("work_one", "owner")
    };
    assert!(store.save(&[record.to_persisted(0)]));

    let raw = std::fs::read_to_string(&path).expect("written");
    assert!(raw.starts_with("{\n  \"version\": 1,\n"), "{raw}");
    assert!(raw.ends_with('\n'), "{raw}");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path)
            .expect("metadata")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "{mode:o}");
    }

    let inner = store.inner();
    let sync_temp = inner.sync_temp_path().expect("a temp path");
    assert!(
        sync_temp
            .to_string_lossy()
            .ends_with(&format!(".{}.tmp", std::process::id())),
        "{}",
        sync_temp.display(),
    );
    let deferred_temp = inner.deferred_temp_path(7).expect("a temp path");
    assert!(
        deferred_temp
            .to_string_lossy()
            .ends_with(&format!(".{}.7.tmp", std::process::id())),
        "{}",
        deferred_temp.display(),
    );
}

/// `error-code` / *task store quarantine path + warnings* — all six sentences,
/// rendered in `zh`, plus the quarantine filename.
#[test]
fn the_six_store_warnings_render_the_catalogued_sentences() {
    let stated = contract_value("error-code", "task store quarantine path + warnings");
    assert!(
        stated.contains("`${this.filePath}.corrupt-${this.now()}`"),
        "{stated}",
    );
    let messages = via_work::store::task_store_messages(Locale::Zh);
    let label = t(Locale::Zh, keys::STORE_TASK_LABEL);
    assert_eq!(label, "任务状态");

    let json_reason = (messages.invalid_json)(label, "Unexpected token");
    assert!(stated.contains("任务状态文件不是有效的 JSON："), "{stated}",);
    assert_eq!(json_reason, "任务状态文件不是有效的 JSON：Unexpected token");

    assert!(
        stated.contains("shape reason: `任务状态文件格式无效`"),
        "{stated}"
    );
    assert_eq!((messages.invalid_shape)(label), "任务状态文件格式无效");

    assert!(
        stated.contains("read failure: `无法读取任务状态文件："),
        "{stated}"
    );
    assert_eq!(
        (messages.read_failed)(label, "EACCES"),
        "无法读取任务状态文件：EACCES",
    );

    assert!(
        stated.contains("save failure: `无法保存任务状态："),
        "{stated}"
    );
    assert_eq!(
        (messages.save_failed)(label, "ENOSPC"),
        "无法保存任务状态：ENOSPC"
    );

    assert_eq!(
        (messages.quarantined)("R", "/p"),
        "R；原文件已隔离为 /p，服务将使用空任务状态继续运行。",
    );
    assert!(
        stated.contains("服务将使用空任务状态继续运行。"),
        "{stated}",
    );

    assert_eq!(
        (messages.quarantine_failed)("R", "EPERM"),
        "R；隔离失败（EPERM），已禁用任务持久化以保护原文件。",
    );
    assert!(
        stated.contains("已禁用任务持久化以保护原文件。"),
        "{stated}",
    );
}

/// `json-field` / */api/health.notes and /api/health.taskStore*.
#[tokio::test(start_paused = true)]
async fn the_store_health_triple_is_the_catalogued_shape() {
    let stated = contract_value("json-field", "/api/health.notes and /api/health.taskStore");
    assert!(
        stated.contains("taskStore: {ok, persistenceEnabled, warning}"),
        "{stated}"
    );

    let manager = WorkManager::in_memory();
    let health = manager.store_health().await;
    let rendered = serde_json::to_value(health).expect("serializes");
    let keys: Vec<&String> = rendered.as_object().expect("object").keys().collect();
    assert_eq!(keys, ["ok", "persistenceEnabled", "warning"]);
}

// ── restart recovery ───────────────────────────────────────────────────────

/// `json-field` / *restart recovery of interrupted tasks*.
#[test]
fn the_restart_recovery_rule_is_the_catalogued_one() {
    let stated = contract_value("json-field", "restart recovery of interrupted tasks");
    assert!(stated.contains("'queued' or 'running'"), "{stated}");
    assert!(stated.contains("status 'failed'"), "{stated}");
    assert!(stated.contains("/重启/"), "{stated}");
    assert!(stated.contains("notificationStatus 'pending'"), "{stated}");

    // The regex the catalogue names is `/重启/`, and both sentences match it.
    for key in [
        keys::WORK_RESTART_INTERACTIVE_INCOMPLETE,
        keys::WORK_RESTART_DELEGATED_LOST,
    ] {
        assert!(
            t(Locale::Zh, key).contains("重启"),
            "{}",
            t(Locale::Zh, key)
        );
    }
    assert_eq!(NotificationStatus::Pending.as_str(), "pending");
}

// ── the conversation projection ────────────────────────────────────────────

/// `json-field` / *conversation message id conventions* and *conversation
/// source values*.
#[test]
fn the_projection_uses_the_catalogued_id_and_source() {
    let ids = contract_value("json-field", "conversation message id conventions");
    assert!(ids.contains("`agent:${taskId}`"), "{ids}");
    let sources = contract_value("json-field", "conversation source values");
    assert!(
        bar_list(&sources).contains(&"agent-result".to_owned()),
        "{sources}"
    );

    let mut record = blank_record("work_one", "owner");
    record.status = via_protocol::WorkStatus::Completed;
    record.result = Some("done".to_owned());
    let projected =
        via_work::projector::project("owner", "main", &record.to_public(0)).expect("projects");
    assert_eq!(projected.id, "agent:work_one");
    assert_eq!(projected.source, "agent-result");
}

// ── the reconciliation ledger ──────────────────────────────────────────────

/// `prompt-text` / *reconciliation block* — the fact shape and the cap.
#[test]
fn the_cancellation_fact_is_the_catalogued_shape_and_cap() {
    let stated = common::contract_values("prompt-text", "reconciliation block").join("\n");
    assert_eq!(
        common::contract_values("prompt-text", "reconciliation block").len(),
        2,
        "the catalogue records the block from both of its call sites",
    );
    assert!(stated.contains("delegated_session_cancelled"), "{stated}");
    assert!(stated.contains("work_id"), "{stated}");
    assert!(stated.contains("delegation_id"), "{stated}");
    assert!(stated.contains("target_session_id"), "{stated}");
    assert!(stated.contains("confirmed_at"), "{stated}");
    assert!(stated.contains("last 20"), "{stated}");

    let fact = via_work::CancellationFact::delegated_session_cancelled(
        "work_one",
        "run-one",
        "target-one",
        "2026-08-22T00:00:00.000Z",
    );
    let rendered = serde_json::to_value(fact).expect("serializes");
    let fields: Vec<&String> = rendered.as_object().expect("object").keys().collect();
    assert_eq!(
        fields,
        [
            "kind",
            "work_id",
            "delegation_id",
            "target_session_id",
            "confirmed_at"
        ],
    );
    assert_eq!(via_work::reconcile::MAX_FACTS_PER_OWNER, 20);
}
