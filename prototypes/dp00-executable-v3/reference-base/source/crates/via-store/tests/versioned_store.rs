//! Contract tests for [`VersionedJsonStore`].
//!
//! Every literal asserted here was read out of `qwen-audio-agent` v1.11.0:
//! `server/src/core/versioned-json-store.mjs`, `server/src/task/task-store.mjs`
//! and `server/test/task-store.test.mjs`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};
use tempfile::TempDir;
use via_store::{
    DEFAULT_DEFERRED_DELAY, DEFAULT_LABEL, DEFAULT_VERSION, DeferredWrite, ScheduleFn,
    StoreMessages, StoreWarning, VersionedJsonStore,
};

/// A fixed clock, so the quarantine filename is a literal the test can name.
const FIXED_NOW: i64 = 1_700_000_000_000;

fn fixed_clock() -> via_store::NowFn {
    Arc::new(|| FIXED_NOW)
}

/// Collects warnings the way `/api/health` and the log sink do.
#[derive(Clone, Default)]
struct Warnings(Arc<Mutex<Vec<StoreWarning>>>);

impl Warnings {
    fn sink(&self) -> via_store::WarningFn {
        let seen = Arc::clone(&self.0);
        Arc::new(move |warning: &StoreWarning| {
            seen.lock().expect("warning sink").push(warning.clone());
        })
    }

    fn take(&self) -> Vec<StoreWarning> {
        std::mem::take(&mut *self.0.lock().expect("warning sink"))
    }
}

/// A hand-cranked timer: holds the scheduled writes so a test can decide when —
/// and whether — each one fires.
#[derive(Clone, Default)]
struct ManualTimer(Arc<Mutex<Vec<(Duration, DeferredWrite)>>>);

impl ManualTimer {
    fn schedule_fn(&self) -> ScheduleFn {
        let pending = Arc::clone(&self.0);
        Arc::new(move |delay: Duration, write: DeferredWrite| {
            pending.lock().expect("timer").push((delay, write));
        })
    }

    fn len(&self) -> usize {
        self.0.lock().expect("timer").len()
    }

    fn delays(&self) -> Vec<Duration> {
        self.0
            .lock()
            .expect("timer")
            .iter()
            .map(|(delay, _)| *delay)
            .collect()
    }

    /// Fire the scheduled write at `index`, leaving the rest pending.
    fn fire(&self, index: usize) {
        let write = self.0.lock().expect("timer").remove(index).1;
        write.run();
    }
}

fn store_at(path: &Path) -> VersionedJsonStore {
    VersionedJsonStore::builder()
        .file_path(path)
        .now(fixed_clock())
        .build()
}

fn quarantine_path_for(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.corrupt-{FIXED_NOW}", path.display()))
}

fn siblings_with_suffix(dir: &Path, suffix: &str) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .expect("read_dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.to_string_lossy().ends_with(suffix))
        .collect()
}

// ── 1. atomic write, and the on-disk format ────────────────────────────────

#[test]
fn on_disk_format_is_two_space_indented_with_a_trailing_newline() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    let store = store_at(&path);

    assert!(store.save(&json!({ "tasks": [{ "id": "work-one" }] })));

    // Contract: `${JSON.stringify({ version: 1, tasks }, null, 2)}\n`
    // — task-store.mjs:98, catalogued as "tasks.json on-disk format".
    assert_eq!(
        fs::read_to_string(&path).expect("read"),
        "{\n  \"version\": 1,\n  \"tasks\": [\n    {\n      \"id\": \"work-one\"\n    }\n  ]\n}\n"
    );
}

#[test]
fn version_is_the_documents_first_key() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("acp-sessions.json");
    let store = store_at(&path);

    // The registry payload order is `coordinators` then `projects`
    // (acp-session-registry.mjs:93-96); `version` is spread in front of both.
    store.save(&json!({ "coordinators": {}, "projects": {} }));

    let raw = fs::read_to_string(&path).expect("read");
    let keys: Vec<String> = serde_json::from_str::<serde_json::Map<String, Value>>(&raw)
        .expect("parse")
        .keys()
        .cloned()
        .collect();
    assert_eq!(keys, ["version", "coordinators", "projects"]);
}

#[test]
fn a_payload_key_named_version_wins_but_keeps_the_leading_position() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("state.json");
    store_at(&path).save(&json!({ "version": 9, "value": true }));

    // `{ version: 1, ...{ version: 9 } }` is `{ version: 9 }` in JS, still in
    // the first slot. Reproduced rather than "fixed" so a document written by
    // either implementation reads back the same in the other.
    assert_eq!(
        fs::read_to_string(&path).expect("read"),
        "{\n  \"version\": 9,\n  \"value\": true\n}\n"
    );
}

#[cfg(unix)]
#[test]
fn the_document_is_written_at_mode_0600() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    store_at(&path).save(&json!({ "tasks": [] }));

    let mode = fs::metadata(&path).expect("metadata").permissions().mode();
    assert_eq!(mode & 0o777, 0o600, "versioned-json-store.mjs:80");
}

#[test]
fn temp_paths_carry_the_pid_and_the_generation() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    let store = store_at(&path);
    let pid = std::process::id();

    // Contract — "tasks.json on-disk format": `${filePath}.${process.pid}.tmp`
    // for a synchronous save, `${filePath}.${process.pid}.${generation}.tmp`
    // for a deferred one.
    assert_eq!(
        store.sync_temp_path(),
        Some(PathBuf::from(format!("{}.{pid}.tmp", path.display())))
    );
    assert_eq!(
        store.deferred_temp_path(7),
        Some(PathBuf::from(format!("{}.{pid}.7.tmp", path.display())))
    );
    // The staging file must be a sibling, or the rename would cross devices
    // and stop being atomic.
    assert_eq!(
        store.sync_temp_path().as_deref().and_then(Path::parent),
        path.parent()
    );
}

#[test]
fn a_completed_save_leaves_no_temp_file_behind() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    store_at(&path).save(&json!({ "tasks": [] }));

    assert!(siblings_with_suffix(dir.path(), ".tmp").is_empty());
}

#[test]
fn a_missing_parent_directory_is_created() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("state").join("acp-sessions.json");

    assert!(store_at(&path).save(&json!({ "coordinators": {} })));
    assert!(path.exists());
}

// ── 2. schema versioning ───────────────────────────────────────────────────

#[test]
fn defaults_match_upstream() {
    assert_eq!(DEFAULT_VERSION, 1, "versioned-json-store.mjs:12");
    assert_eq!(DEFAULT_LABEL, "状态", "versioned-json-store.mjs:13");
    assert_eq!(
        DEFAULT_DEFERRED_DELAY,
        Duration::from_millis(250),
        "task-store.mjs:22"
    );

    let store = VersionedJsonStore::builder().build();
    assert_eq!(store.version(), DEFAULT_VERSION);
    assert_eq!(store.label(), DEFAULT_LABEL);
}

#[test]
fn a_round_trip_returns_the_document_including_its_version() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    let store = store_at(&path);
    store.save(&json!({ "tasks": [{ "id": "work-one" }] }));

    let document = store.load(|_| true).expect("document");
    assert_eq!(document.get("version"), Some(&json!(1)));
    assert_eq!(document.get("tasks"), Some(&json!([{ "id": "work-one" }])));
}

#[test]
fn a_migration_hook_upgrades_an_older_document_instead_of_quarantining() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    fs::write(&path, r#"{"version":0,"work":[{"id":"work-one"}]}"#).expect("seed");

    let seen_versions = Arc::new(Mutex::new(Vec::new()));
    let recorded = Arc::clone(&seen_versions);
    let store = VersionedJsonStore::builder()
        .file_path(&path)
        .now(fixed_clock())
        .migrate(Arc::new(move |on_disk, mut document| {
            recorded.lock().expect("versions").push(on_disk);
            let work = document.remove("work")?;
            document.insert("tasks".into(), work);
            Some(document)
        }))
        .build();

    let document = store
        .load(|d| d.get("tasks").is_some_and(Value::is_array))
        .expect("migrated");
    assert_eq!(*seen_versions.lock().expect("versions"), [Some(0)]);
    assert_eq!(document.get("version"), Some(&json!(1)));
    assert_eq!(document.get("tasks"), Some(&json!([{ "id": "work-one" }])));
    // Nothing was moved aside: a migration is not a corruption.
    assert!(!quarantine_path_for(&path).exists());
    assert!(store.health().ok);
}

#[test]
fn a_migration_hook_that_declines_falls_through_to_quarantine() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    fs::write(&path, r#"{"version":999,"tasks":[]}"#).expect("seed");

    let store = VersionedJsonStore::builder()
        .file_path(&path)
        .now(fixed_clock())
        .migrate(Arc::new(|_, _| None))
        .build();

    assert!(store.load(|_| true).is_none());
    assert!(quarantine_path_for(&path).exists());
}

#[test]
fn without_a_hook_an_unsupported_version_is_quarantined() {
    // server/test/acp-session-registry.test.mjs:86-102.
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("acp-sessions.json");
    fs::write(&path, r#"{"version":999,"coordinators":{},"projects":{}}"#).expect("seed");

    let warnings = Warnings::default();
    let store = VersionedJsonStore::builder()
        .file_path(&path)
        .now(fixed_clock())
        .on_warning(warnings.sink())
        .build();

    assert!(store.load(|_| true).is_none());
    assert!(!store.health().ok);
    assert!(!path.exists());
    assert_eq!(
        warnings.take()[0].message,
        format!(
            "状态文件格式或版本无效；原文件已隔离为 {}。",
            quarantine_path_for(&path).display()
        )
    );
}

// ── 3. quarantine on corrupt ───────────────────────────────────────────────

#[test]
fn all_six_default_warning_strings_are_upstreams() {
    // server/src/core/versioned-json-store.mjs:36-86, with label '状态'.
    let m = StoreMessages::DEFAULT;
    assert_eq!(
        (m.invalid_json)("状态", "Unexpected token"),
        "状态文件不是有效的 JSON：Unexpected token"
    );
    assert_eq!((m.invalid_shape)("状态"), "状态文件格式或版本无效");
    assert_eq!(
        (m.read_failed)("状态", "EACCES"),
        "无法读取状态文件：EACCES"
    );
    assert_eq!(
        (m.save_failed)("状态", "ENOSPC"),
        "无法保存状态文件：ENOSPC"
    );
    assert_eq!(
        (m.quarantined)("原因", "/tmp/a.json.corrupt-1"),
        "原因；原文件已隔离为 /tmp/a.json.corrupt-1。"
    );
    assert_eq!(
        (m.quarantine_failed)("原因", "EPERM"),
        "原因；隔离失败（EPERM），已禁用持久化以保护原文件。"
    );
}

#[test]
fn all_six_task_store_warning_strings_are_upstreams() {
    // server/src/task/task-store.mjs:45-104, catalogued as
    // "task store quarantine path + warnings", with label '任务状态'.
    let m = StoreMessages::TASK_STORE;
    assert_eq!(
        (m.invalid_json)("任务状态", "Unexpected token"),
        "任务状态文件不是有效的 JSON：Unexpected token"
    );
    assert_eq!((m.invalid_shape)("任务状态"), "任务状态文件格式无效");
    assert_eq!(
        (m.read_failed)("任务状态", "EACCES"),
        "无法读取任务状态文件：EACCES"
    );
    assert_eq!(
        (m.save_failed)("任务状态", "ENOSPC"),
        "无法保存任务状态：ENOSPC"
    );
    assert_eq!(
        (m.quarantined)("原因", "/tmp/tasks.json.corrupt-1"),
        "原因；原文件已隔离为 /tmp/tasks.json.corrupt-1，服务将使用空任务状态继续运行。"
    );
    assert_eq!(
        (m.quarantine_failed)("原因", "EPERM"),
        "原因；隔离失败（EPERM），已禁用任务持久化以保护原文件。"
    );
}

#[test]
fn an_absent_file_loads_as_a_fallback_without_a_warning() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    let warnings = Warnings::default();
    let store = VersionedJsonStore::builder()
        .file_path(&path)
        .on_warning(warnings.sink())
        .build();

    assert!(store.load(|_| true).is_none());
    assert!(warnings.take().is_empty(), "a first run is not a fault");
    assert!(store.health().ok);
    assert!(store.persistence_enabled());
}

#[test]
fn corrupt_json_is_quarantined_and_the_original_is_retained() {
    // server/test/acp-session-registry.test.mjs:55-84.
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("acp-sessions.json");
    fs::write(&path, "{not-json").expect("seed");

    let warnings = Warnings::default();
    let store = VersionedJsonStore::builder()
        .file_path(&path)
        .now(fixed_clock())
        .on_warning(warnings.sink())
        .build();

    assert!(store.load(|_| true).is_none());

    let quarantined = quarantine_path_for(&path);
    assert_eq!(
        fs::read_to_string(&quarantined).expect("quarantined"),
        "{not-json",
        "the original bytes are retained, not deleted"
    );
    assert!(!path.exists());

    let seen = warnings.take();
    assert_eq!(seen.len(), 1);
    assert!(seen[0].message.starts_with("状态文件不是有效的 JSON："));
    assert!(
        seen[0]
            .message
            .ends_with(&format!("；原文件已隔离为 {}。", quarantined.display()))
    );
    assert_eq!(
        seen[0].quarantine_path,
        Some(quarantined.display().to_string())
    );
    assert_eq!(seen[0].at, FIXED_NOW);

    // Quarantine is recovery, not failure: the store keeps persisting.
    assert!(!store.health().ok);
    assert!(store.persistence_enabled());
    assert!(store.save(&json!({ "coordinators": { "qoder:owner:backend": {} } })));
    assert!(store.load(|_| true).is_some());
}

#[test]
fn a_document_rejected_by_validate_is_quarantined() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    fs::write(&path, r#"{"version":1,"tasks":"not-an-array"}"#).expect("seed");

    let warnings = Warnings::default();
    let store = VersionedJsonStore::builder()
        .file_path(&path)
        .now(fixed_clock())
        .on_warning(warnings.sink())
        .build();

    // task-store.mjs:79 — `!Array.isArray(parsed.tasks)` is a quarantine, and
    // it reports the same reason as a version mismatch.
    assert!(
        store
            .load(|d| d.get("tasks").is_some_and(Value::is_array))
            .is_none()
    );
    assert!(
        warnings.take()[0]
            .message
            .starts_with("状态文件格式或版本无效")
    );
    assert!(quarantine_path_for(&path).exists());
}

#[test]
fn a_document_that_is_not_an_object_is_quarantined() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    fs::write(&path, "[1,2,3]").expect("seed");

    let store = store_at(&path);
    // `parsed?.version !== this.version` — an array has no `version`.
    assert!(store.load(|_| true).is_none());
    assert!(quarantine_path_for(&path).exists());
}

#[test]
fn a_failed_quarantine_disables_persistence_rather_than_clobbering_the_file() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    fs::write(&path, "{not-json").expect("seed");

    // Block the rename by occupying the exact quarantine path with a
    // directory: POSIX rename() fails with EISDIR when the destination is one.
    let blocked = quarantine_path_for(&path);
    fs::create_dir(&blocked).expect("block");
    fs::write(blocked.join("keep"), "x").expect("block");

    let warnings = Warnings::default();
    let store = VersionedJsonStore::builder()
        .file_path(&path)
        .now(fixed_clock())
        .on_warning(warnings.sink())
        .build();

    assert!(store.load(|_| true).is_none());

    // The whole point: the bytes we could not move aside are the only copy of
    // the user's state, so persistence goes off and the file stays put.
    assert!(!store.persistence_enabled());
    assert!(!store.health().persistence_enabled);
    assert!(!store.health().ok);

    let seen = warnings.take();
    assert_eq!(seen.len(), 1);
    assert!(seen[0].message.starts_with("状态文件不是有效的 JSON："));
    assert!(
        seen[0].message.contains("；隔离失败（"),
        "got {:?}",
        seen[0].message
    );
    assert!(seen[0].message.ends_with("），已禁用持久化以保护原文件。"));
    assert_eq!(seen[0].quarantine_path, None);

    // And every later write is refused, in both flavours.
    assert_eq!(fs::read_to_string(&path).expect("read"), "{not-json");
    assert!(!store.save(&json!({ "tasks": [] })));
    store.save_deferred(&json!({ "tasks": ["ignored"] }));
    assert!(!store.has_pending_deferred());
    assert!(!store.flush());
    assert_eq!(
        fs::read_to_string(&path).expect("read"),
        "{not-json",
        "the original must survive untouched"
    );
    assert!(store.load(|_| true).is_none());
    assert!(
        warnings.take().is_empty(),
        "one warning, not one per attempt"
    );
}

#[test]
fn an_unreadable_document_disables_persistence_without_quarantining_it() {
    let dir = TempDir::new().expect("tempdir");
    // A directory where the document should be: readable metadata, unreadable
    // contents, and emphatically not a parse failure.
    let path = dir.path().join("tasks.json");
    fs::create_dir(&path).expect("seed");

    let warnings = Warnings::default();
    let store = VersionedJsonStore::builder()
        .file_path(&path)
        .now(fixed_clock())
        .on_warning(warnings.sink())
        .build();

    assert!(store.load(|_| true).is_none());
    assert!(!store.persistence_enabled());
    assert!(
        !quarantine_path_for(&path).exists(),
        "an I/O failure says nothing about the bytes, so they are left alone"
    );

    let seen = warnings.take();
    assert!(seen[0].message.starts_with("无法读取状态文件："));
    assert_eq!(seen[0].quarantine_path, None);
    assert!(path.is_dir());
}

#[test]
fn a_panicking_warning_sink_does_not_prevent_startup() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    fs::write(&path, "{not-json").expect("seed");

    let store = VersionedJsonStore::builder()
        .file_path(&path)
        .now(fixed_clock())
        .on_warning(Arc::new(|_| panic!("the log sink is down")))
        .build();

    // versioned-json-store.mjs:31-34 — "Diagnostics must never prevent
    // startup."
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    assert!(store.load(|_| true).is_none());
    std::panic::set_hook(previous);

    assert!(quarantine_path_for(&path).exists());
    assert!(store.warning().is_some());
}

// ── 4. coalesced saves ─────────────────────────────────────────────────────

#[test]
fn a_burst_of_mutations_writes_only_the_newest_state() {
    // server/test/task-store.test.mjs:97-111, with the 60 s delay replaced by
    // a timer the test drives by hand.
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    let timer = ManualTimer::default();
    let store = VersionedJsonStore::builder()
        .file_path(&path)
        .now(fixed_clock())
        .deferred_delay(Duration::from_millis(60_000))
        .schedule(timer.schedule_fn())
        .build();

    store.save_deferred(&json!({ "tasks": [{ "id": "work-one", "activity": ["first"] }] }));
    store.save_deferred(&json!({ "tasks": [{ "id": "work-one", "activity": ["latest"] }] }));

    assert_eq!(timer.len(), 2);
    assert_eq!(timer.delays(), [Duration::from_millis(60_000); 2]);
    assert!(
        !path.exists(),
        "nothing is written before the delay elapses"
    );

    // The first ticket is stale — the second mutation superseded it — so it
    // must not write the intermediate state.
    timer.fire(0);
    assert!(!path.exists());
    assert!(store.has_pending_deferred());

    timer.fire(0);
    let saved: Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
    assert_eq!(saved["tasks"][0]["activity"], json!(["latest"]));
    assert!(!store.has_pending_deferred());
    assert!(siblings_with_suffix(dir.path(), ".tmp").is_empty());
}

#[test]
fn flush_writes_pending_content_without_a_timer() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    let store = VersionedJsonStore::builder()
        .file_path(&path)
        .now(fixed_clock())
        .build();

    store.save_deferred(&json!({ "tasks": [{ "id": "work-one", "activity": ["first"] }] }));
    store.save_deferred(&json!({ "tasks": [{ "id": "work-one", "activity": ["latest"] }] }));
    assert!(!path.exists());

    assert!(store.flush());
    let saved: Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
    assert_eq!(saved["tasks"][0]["activity"], json!(["latest"]));
    assert!(!store.flush(), "a second flush has nothing left to write");
}

#[test]
fn a_synchronous_terminal_save_supersedes_an_older_deferred_write() {
    // server/test/task-store.test.mjs:113-125.
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    let timer = ManualTimer::default();
    let store = VersionedJsonStore::builder()
        .file_path(&path)
        .now(fixed_clock())
        .deferred_delay(Duration::from_millis(60_000))
        .schedule(timer.schedule_fn())
        .build();

    store.save_deferred(&json!({ "tasks": [{ "id": "work-one", "status": "running" }] }));
    assert!(store.save(&json!({ "tasks": [{ "id": "work-one", "status": "completed" }] })));

    // The scheduled ticket outlives the sync save and must be inert.
    timer.fire(0);
    assert!(!store.flush(), "the ticket was superseded, not merely late");

    let saved: Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
    assert_eq!(saved["tasks"][0]["status"], json!("completed"));
    assert!(siblings_with_suffix(dir.path(), ".tmp").is_empty());
}

#[test]
fn a_dropped_ticket_leaves_the_content_pending_for_the_next_one() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    let timer = ManualTimer::default();
    let store = VersionedJsonStore::builder()
        .file_path(&path)
        .now(fixed_clock())
        .schedule(timer.schedule_fn())
        .build();

    store.save_deferred(&json!({ "tasks": ["a"] }));
    drop(timer);
    assert!(store.has_pending_deferred());
    assert!(store.flush());
    assert!(path.exists());
}

#[test]
fn concurrent_writers_never_leave_a_partial_document_or_a_stray_temp() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    let store = VersionedJsonStore::builder()
        .file_path(&path)
        .now(fixed_clock())
        .build();

    std::thread::scope(|scope| {
        for worker in 0..8u32 {
            let store = store.clone();
            scope.spawn(move || {
                for round in 0..40u32 {
                    let payload = json!({ "tasks": [{ "id": worker, "round": round }] });
                    if round % 2 == 0 {
                        store.save(&payload);
                    } else {
                        store.save_deferred(&payload);
                        store.flush();
                    }
                }
            });
        }
    });
    store.flush();

    let saved: Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
    assert_eq!(saved["version"], json!(1));
    assert!(saved["tasks"][0]["round"].is_number());
    assert!(
        siblings_with_suffix(dir.path(), ".tmp").is_empty(),
        "a superseded write must clean up its own staging file"
    );
    assert!(store.persistence_enabled());
}

// ── health ─────────────────────────────────────────────────────────────────

#[test]
fn health_serializes_in_the_api_health_field_order() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("tasks.json");
    let store = store_at(&path);

    // Contract — "/api/health.notes and /api/health.taskStore":
    // {ok, persistenceEnabled, warning}, warning {message, quarantinePath, at}.
    assert_eq!(
        serde_json::to_string(&store.health()).expect("serialize"),
        r#"{"ok":true,"persistenceEnabled":true,"warning":null}"#
    );

    fs::write(&path, "{not-json").expect("seed");
    store.load(|_| true);
    let encoded = serde_json::to_string(&store.health()).expect("serialize");
    assert!(encoded.starts_with(r#"{"ok":false,"persistenceEnabled":true,"warning":{"message":"#));
    assert!(encoded.ends_with(&format!(r#""at":{FIXED_NOW}}}}}"#)));
    assert!(encoded.contains(r#""quarantinePath":"#));
}

#[test]
fn a_store_without_a_path_is_a_working_no_op() {
    // versioned-json-store.mjs:11 — `filePath = null` is the default, and a
    // Gateway with no state directory still has to run.
    let store = VersionedJsonStore::builder().build();

    assert!(store.file_path().is_none());
    assert!(store.load(|_| true).is_none());
    assert!(!store.save(&json!({ "tasks": [] })));
    store.save_deferred(&json!({ "tasks": [] }));
    assert!(!store.flush());
    assert_eq!(
        store.health(),
        via_store::StoreHealth {
            ok: true,
            persistence_enabled: false,
            warning: None,
        }
    );
}
