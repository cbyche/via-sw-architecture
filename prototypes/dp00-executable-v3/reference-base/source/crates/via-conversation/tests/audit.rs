//! [`MemoryAudit`] — ported from `server/test/memory-audit.test.mjs`.

use std::sync::{Arc, Mutex};

use pretty_assertions::assert_eq;
use via_conversation::audit::{AuditEvent, AuditWarning, MemoryAudit, SkipReason, iso_timestamp};
use via_i18n::{Locale, keys};

#[test]
fn one_json_line_per_event_at_owner_only_permissions() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("state").join("memory-audit.jsonl");
    let audit = MemoryAudit::builder()
        .file_path(&path)
        .now(Arc::new(|| 1000))
        .build();

    assert!(audit.record(&AuditEvent::error("owner", "boom")));
    assert!(audit.record(&AuditEvent::skip("owner", SkipReason::NoChange)));

    let lines: Vec<serde_json::Value> = std::fs::read_to_string(&path)
        .expect("readable")
        .lines()
        .map(|line| serde_json::from_str(line).expect("one JSON line"))
        .collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["op"], serde_json::json!("error"));
    assert_eq!(lines[0]["at"], serde_json::json!(iso_timestamp(1000)));
    assert_eq!(lines[1]["reason"], serde_json::json!("no_change"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path)
            .expect("metadata")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    let health = audit.health();
    assert_eq!(
        serde_json::to_value(&health).expect("serializable"),
        serde_json::json!({
            "ok": true,
            "configured": true,
            "enabled": true,
            "warning": null,
        })
    );
}

#[test]
fn an_audit_with_no_path_stays_silent() {
    let audit = MemoryAudit::default();
    assert!(!audit.record(&AuditEvent::error("owner", "boom")));
    let health = audit.health();
    assert!(!health.configured);
    assert!(!health.enabled);
    assert!(health.ok, "unconfigured is not unhealthy");
}

#[test]
fn a_write_failure_disables_the_audit_and_warns_exactly_once() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    // A regular file used as a directory segment: the parent cannot be made.
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, "not a directory").expect("write");

    let warnings: Arc<Mutex<Vec<AuditWarning>>> = Arc::default();
    let sink = warnings.clone();
    let audit = MemoryAudit::builder()
        .file_path(blocker.join("nested").join("memory-audit.jsonl"))
        .locale(Locale::Zh)
        .now(Arc::new(|| 55))
        .on_warning(Arc::new(move |warning| {
            sink.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(warning.clone());
        }))
        .build();

    assert!(!audit.record(&AuditEvent::error("owner", "first")));
    assert!(!audit.record(&AuditEvent::error("owner", "second")));

    let raised = warnings
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    assert_eq!(raised.len(), 1, "one warning, not one per attempt");
    assert_eq!(raised[0].at, 55);
    let template = via_i18n::t(Locale::Zh, keys::MEMORY_AUDIT_DISABLED);
    let (head, _) = template
        .split_once("{detail}")
        .expect("a templated message");
    assert!(raised[0].message.starts_with(head));
    assert!(raised[0].message.ends_with("记忆功能不受影响。"));

    let health = audit.health();
    assert!(!health.ok);
    assert!(!health.enabled);
    assert!(health.configured);
}

#[test]
fn a_panicking_warning_sink_does_not_break_a_record() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, "not a directory").expect("write");
    let audit = MemoryAudit::builder()
        .file_path(blocker.join("nested").join("memory-audit.jsonl"))
        .on_warning(Arc::new(|_| panic!("a misbehaving diagnostics sink")))
        .build();

    // "Diagnostics must not prevent memory operations."
    assert!(!audit.record(&AuditEvent::error("owner", "boom")));
    assert!(!audit.health().enabled);
}

#[test]
fn a_patch_line_records_the_shape_of_the_change_and_not_its_text() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("memory-audit.jsonl");
    let audit = MemoryAudit::builder()
        .file_path(&path)
        .now(Arc::new(|| 7))
        .build();

    let mut before = serde_json::Map::new();
    before.insert("user".to_owned(), serde_json::Value::Null);
    before.insert("memory".to_owned(), serde_json::json!("aaaaaaaaaaaaaaaa"));
    let mut after = serde_json::Map::new();
    after.insert("user".to_owned(), serde_json::json!("bbbbbbbbbbbbbbbb"));
    after.insert("memory".to_owned(), serde_json::json!("cccccccccccccccc"));

    assert!(audit.record(&AuditEvent::Patch {
        op: "patch",
        owner_id: "owner".to_owned(),
        documents: vec!["user".to_owned(), "memory".to_owned()],
        changed: 3,
        before_revisions: before,
        after_revisions: after,
        edits: 2,
        appended: true,
    }));

    let raw = std::fs::read_to_string(&path).expect("readable");
    let line: serde_json::Value = serde_json::from_str(raw.trim()).expect("one JSON line");
    assert_eq!(
        line.as_object()
            .expect("an object")
            .keys()
            .collect::<Vec<_>>(),
        vec![
            "at",
            "op",
            "ownerId",
            "documents",
            "changed",
            "beforeRevisions",
            "afterRevisions",
            "edits",
            "appended"
        ]
    );
    assert_eq!(line["beforeRevisions"]["user"], serde_json::Value::Null);
    assert_eq!(line["changed"], serde_json::json!(3));
    assert_eq!(line["appended"], serde_json::json!(true));
}

#[test]
fn every_op_carries_its_owner() {
    for event in [
        AuditEvent::skip("owner-a", SkipReason::DocumentBoundary),
        AuditEvent::error("owner-a", "boom"),
    ] {
        let value = serde_json::to_value(&event).expect("serializable");
        assert_eq!(value["ownerId"], serde_json::json!("owner-a"));
        assert_eq!(value["op"], serde_json::json!(event.op()));
    }
}

#[test]
fn the_audit_appends_rather_than_replacing() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("memory-audit.jsonl");
    std::fs::write(&path, "{\"at\":\"earlier\"}\n").expect("seeded");
    let audit = MemoryAudit::builder().file_path(&path).build();
    assert!(audit.record(&AuditEvent::skip("owner", SkipReason::Sensitive)));

    let raw = std::fs::read_to_string(&path).expect("readable");
    assert_eq!(raw.lines().count(), 2);
    assert!(raw.starts_with("{\"at\":\"earlier\"}"));
}
