//! `tasks.json`.
//!
//! `server/src/task/task-store.mjs` — except that almost none of it is written
//! here, because `via-store` already is it. The atomic write, the
//! `<path>.<pid>.tmp` / `<path>.<pid>.<generation>.tmp` staging names, the
//! schema version, the quarantine-to-`<path>.corrupt-<now>`, the
//! second-order rule that a *failed* quarantine disables persistence rather
//! than clobbering the original, the generation-tagged coalesced write and the
//! health triple are all
//! [`via_store::VersionedJsonStore`]. This module supplies the three things
//! that are the task store's own: the document shape, the string table, and the
//! host timer.
//!
//! # The document
//!
//! ```json
//! { "version": 1, "tasks": [ … ] }
//! ```
//!
//! Two-space indent, trailing newline, mode `0600`. Contract —
//! `docs/reference/contracts.json` (`file-path` / *tasks.json on-disk format*).
//! A `version` that is not `1`, or a `tasks` that is not an array, quarantines.
//!
//! # The strings
//!
//! [`via_store::StoreMessages`] holds six function pointers, and this module
//! supplies one set per locale built from the `store.*` and `store.task.*`
//! catalog keys. The `zh` set renders the catalogued sentences byte for byte,
//! which is what `via-conformance` asserts; the other two are their authored
//! peers. Nothing here is a literal.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;
use via_i18n::{Locale, format as i18n_format, keys, t};
use via_store::{
    DEFAULT_DEFERRED_DELAY, NowFn, StoreHealth, StoreMessages, VersionedJsonStore, WarningFn,
};

use crate::record::PersistedWork;

/// The schema version `tasks.json` carries.
///
/// Contract — `server/src/task/task-store.mjs:15` (`const VERSION = 1`).
pub const TASK_STORE_VERSION: u64 = 1;

/// The document key the Work array lives under.
pub const TASKS_KEY: &str = "tasks";

/// The coalescing delay.
///
/// Contract — `server/src/task/task-store.mjs:22` (`deferredDelayMs = 250`).
/// The same value [`via_store::DEFAULT_DEFERRED_DELAY`] carries; named again
/// here because this is the surface it was measured for — a 500 ms progress
/// cadence on a phone (`docs/architecture.md` §4).
pub const DEFERRED_DELAY: Duration = DEFAULT_DEFERRED_DELAY;

/// The document, as it is written.
///
/// `version` is prepended by [`via_store::VersionedJsonStore`], so this carries
/// only the payload.
#[derive(Debug, Serialize)]
struct Document<'a> {
    tasks: &'a [PersistedWork],
}

/// The six task-store warnings in `locale`.
///
/// Contract — `docs/reference/contracts.json` (`error-code` / *task store
/// quarantine path + warnings*). Four of the six are the task surface's own
/// wording (`store.task.*`) and two are the generic ones (`store.*`); with the
/// label `任务状态` the `zh` column renders the catalogued sentences exactly.
///
/// One `StoreMessages` per locale, because its fields are plain function
/// pointers — which is the point: a set of `fn`s can name catalog keys, where a
/// captured locale would need a closure the store cannot hold.
#[must_use]
pub fn task_store_messages(locale: Locale) -> StoreMessages {
    match locale {
        Locale::En => StoreMessages {
            invalid_json: |label, detail| {
                i18n_format(
                    Locale::En,
                    keys::STORE_INVALID_JSON,
                    &[("label", label), ("detail", detail)],
                )
            },
            invalid_shape: |label| {
                i18n_format(
                    Locale::En,
                    keys::STORE_TASK_INVALID_SHAPE,
                    &[("label", label)],
                )
            },
            read_failed: |label, detail| {
                i18n_format(
                    Locale::En,
                    keys::STORE_READ_FAILED,
                    &[("label", label), ("detail", detail)],
                )
            },
            save_failed: |label, detail| {
                i18n_format(
                    Locale::En,
                    keys::STORE_TASK_SAVE_FAILED,
                    &[("label", label), ("detail", detail)],
                )
            },
            quarantined: |reason, path| {
                i18n_format(
                    Locale::En,
                    keys::STORE_TASK_QUARANTINED,
                    &[("reason", reason), ("path", path)],
                )
            },
            quarantine_failed: |reason, detail| {
                i18n_format(
                    Locale::En,
                    keys::STORE_TASK_QUARANTINE_FAILED,
                    &[("reason", reason), ("detail", detail)],
                )
            },
        },
        Locale::Zh => StoreMessages {
            invalid_json: |label, detail| {
                i18n_format(
                    Locale::Zh,
                    keys::STORE_INVALID_JSON,
                    &[("label", label), ("detail", detail)],
                )
            },
            invalid_shape: |label| {
                i18n_format(
                    Locale::Zh,
                    keys::STORE_TASK_INVALID_SHAPE,
                    &[("label", label)],
                )
            },
            read_failed: |label, detail| {
                i18n_format(
                    Locale::Zh,
                    keys::STORE_READ_FAILED,
                    &[("label", label), ("detail", detail)],
                )
            },
            save_failed: |label, detail| {
                i18n_format(
                    Locale::Zh,
                    keys::STORE_TASK_SAVE_FAILED,
                    &[("label", label), ("detail", detail)],
                )
            },
            quarantined: |reason, path| {
                i18n_format(
                    Locale::Zh,
                    keys::STORE_TASK_QUARANTINED,
                    &[("reason", reason), ("path", path)],
                )
            },
            quarantine_failed: |reason, detail| {
                i18n_format(
                    Locale::Zh,
                    keys::STORE_TASK_QUARANTINE_FAILED,
                    &[("reason", reason), ("detail", detail)],
                )
            },
        },
        Locale::Ko => StoreMessages {
            invalid_json: |label, detail| {
                i18n_format(
                    Locale::Ko,
                    keys::STORE_INVALID_JSON,
                    &[("label", label), ("detail", detail)],
                )
            },
            invalid_shape: |label| {
                i18n_format(
                    Locale::Ko,
                    keys::STORE_TASK_INVALID_SHAPE,
                    &[("label", label)],
                )
            },
            read_failed: |label, detail| {
                i18n_format(
                    Locale::Ko,
                    keys::STORE_READ_FAILED,
                    &[("label", label), ("detail", detail)],
                )
            },
            save_failed: |label, detail| {
                i18n_format(
                    Locale::Ko,
                    keys::STORE_TASK_SAVE_FAILED,
                    &[("label", label), ("detail", detail)],
                )
            },
            quarantined: |reason, path| {
                i18n_format(
                    Locale::Ko,
                    keys::STORE_TASK_QUARANTINED,
                    &[("reason", reason), ("path", path)],
                )
            },
            quarantine_failed: |reason, detail| {
                i18n_format(
                    Locale::Ko,
                    keys::STORE_TASK_QUARANTINE_FAILED,
                    &[("reason", reason), ("detail", detail)],
                )
            },
        },
    }
}

/// The Work subsystem's durable state.
///
/// Cheap to clone; every clone is the same store.
#[derive(Debug, Clone)]
pub struct WorkStore {
    inner: VersionedJsonStore,
}

/// Builds a [`WorkStore`].
#[derive(Default)]
pub struct WorkStoreBuilder {
    file_path: Option<PathBuf>,
    locale: Locale,
    now: Option<NowFn>,
    on_warning: Option<WarningFn>,
    deferred_delay: Option<Duration>,
    schedule: bool,
}

impl std::fmt::Debug for WorkStoreBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkStoreBuilder")
            .field("file_path", &self.file_path)
            .field("locale", &self.locale)
            .field("deferred_delay", &self.deferred_delay)
            .field("schedule", &self.schedule)
            .finish_non_exhaustive()
    }
}

impl WorkStoreBuilder {
    /// Where `tasks.json` lives. Without one the store is an in-memory no-op —
    /// upstream's `filePath = null`, which keeps a Gateway with no state
    /// directory running rather than failing.
    #[must_use]
    pub fn file_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// The locale the six warnings render in.
    #[must_use]
    pub const fn locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// The clock that stamps the quarantine filename and the warning's `at`.
    #[must_use]
    pub fn now(mut self, now: NowFn) -> Self {
        self.now = Some(now);
        self
    }

    /// Where warnings go. Upstream logs them as `task.persistence_warning` and
    /// surfaces them on `/api/health` as `taskStore.health().warning`.
    #[must_use]
    pub fn on_warning(mut self, on_warning: WarningFn) -> Self {
        self.on_warning = Some(on_warning);
        self
    }

    /// Override the coalescing delay. [`DEFERRED_DELAY`] by default.
    #[must_use]
    pub const fn deferred_delay(mut self, delay: Duration) -> Self {
        self.deferred_delay = Some(delay);
        self
    }

    /// Let coalesced writes start themselves on the ambient tokio runtime.
    ///
    /// Without this — or outside a runtime — deferred content is held until
    /// [`WorkStore::flush`], which still coalesces but never self-starts. The
    /// manager turns it on.
    #[must_use]
    pub const fn self_scheduling(mut self, schedule: bool) -> Self {
        self.schedule = schedule;
        self
    }

    /// Finish the store.
    #[must_use]
    pub fn build(self) -> WorkStore {
        let mut store = VersionedJsonStore::builder()
            .version(TASK_STORE_VERSION)
            .label(t(self.locale, keys::STORE_TASK_LABEL))
            .messages(task_store_messages(self.locale))
            .deferred_delay(self.deferred_delay.unwrap_or(DEFERRED_DELAY));
        if let Some(path) = self.file_path {
            store = store.file_path(path);
        }
        if let Some(now) = self.now {
            store = store.now(now);
        }
        if let Some(on_warning) = self.on_warning {
            store = store.on_warning(on_warning);
        }
        // The store is a leaf crate with no runtime; the timer is the host's.
        // `try_current` rather than `spawn`, so building a store outside a
        // runtime degrades to "flush-only" instead of panicking.
        if self.schedule && tokio::runtime::Handle::try_current().is_ok() {
            store = store.schedule(Arc::new(|delay, write| {
                tokio::spawn(async move {
                    tokio::time::sleep(delay).await;
                    write.run();
                });
            }));
        }
        WorkStore {
            inner: store.build(),
        }
    }
}

impl WorkStore {
    /// Start building a store.
    #[must_use]
    pub fn builder() -> WorkStoreBuilder {
        WorkStoreBuilder::default()
    }

    /// An in-memory store that never touches a disk.
    #[must_use]
    pub fn in_memory() -> Self {
        Self::builder().build()
    }

    /// Read `tasks.json`.
    ///
    /// An absent file is the normal first run and yields an empty list with no
    /// warning. Anything else that fails — unparseable, wrong version, `tasks`
    /// not an array — has already been quarantined by the time this returns,
    /// and [`Self::health`] says what happened.
    ///
    /// A record that is not a JSON object is dropped, which is upstream's
    /// `parsed.tasks.filter(task => task && typeof task === 'object')`.
    /// A record that *is* an object but carries no usable `id` is also dropped
    /// — see the crate-level deviation note.
    #[must_use]
    pub fn load(&self) -> Vec<PersistedWork> {
        let Some(document) = self
            .inner
            .load(|document| document.get(TASKS_KEY).is_some_and(Value::is_array))
        else {
            return Vec::new();
        };
        document
            .get(TASKS_KEY)
            .and_then(Value::as_array)
            .map(|tasks| {
                tasks
                    .iter()
                    .filter(|task| task.is_object())
                    .filter_map(|task| serde_json::from_value::<PersistedWork>(task.clone()).ok())
                    .filter(|task| !task.id.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Write now, superseding any pending coalesced write.
    ///
    /// Returns whether the bytes reached the disk. Every terminal transition
    /// goes through here.
    pub fn save(&self, tasks: &[PersistedWork]) -> bool {
        self.inner.save(&Document { tasks })
    }

    /// Record `tasks` as the newest state and schedule one write.
    ///
    /// A burst of activity updates costs one write of the last state. Used for
    /// `backend.activity` and for the reminder scheduler's fire, both of which
    /// can arrive many times a second.
    pub fn save_deferred(&self, tasks: &[PersistedWork]) {
        self.inner.save_deferred(&Document { tasks });
    }

    /// Write any pending coalesced content immediately.
    ///
    /// Upstream's shutdown calls `taskStore.flush()` **before** `server.close()`
    /// — that ordering is the durability guarantee for in-flight Work state
    /// (`docs/reference/contracts.json`, *shutdown sequence*).
    pub fn flush(&self) -> bool {
        self.inner.flush()
    }

    /// The health triple, as `/api/health` publishes it under `taskStore`.
    #[must_use]
    pub fn health(&self) -> StoreHealth {
        self.inner.health()
    }

    /// Whether a write would currently reach the disk.
    #[must_use]
    pub fn persistence_enabled(&self) -> bool {
        self.inner.persistence_enabled()
    }

    /// Whether coalesced content is waiting.
    #[must_use]
    pub fn has_pending_deferred(&self) -> bool {
        self.inner.has_pending_deferred()
    }

    /// The underlying document store, for callers that need its paths.
    #[must_use]
    pub const fn inner(&self) -> &VersionedJsonStore {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::{TASK_STORE_VERSION, WorkStore, task_store_messages};
    use crate::record::{PersistedWork, WorkRecord};
    use serde_json::json;
    use via_i18n::Locale;
    use via_protocol::{WorkKind, WorkStatus};

    fn persisted(id: &str) -> PersistedWork {
        serde_json::from_value(json!({
            "id": id,
            "status": "completed",
            "kind": "work",
            "objective": "o",
            "ownerId": "owner",
            "sessionId": "voice",
        }))
        .expect("parses")
    }

    #[test]
    fn the_document_is_version_first_two_space_indented_and_newline_terminated() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("tasks.json");
        let store = WorkStore::builder().file_path(&path).build();
        assert!(store.save(&[persisted("work_one")]));

        let raw = std::fs::read_to_string(&path).expect("written");
        assert!(
            raw.starts_with("{\n  \"version\": 1,\n  \"tasks\": ["),
            "{raw}"
        );
        assert!(raw.ends_with("\n"), "{raw}");
        let parsed: serde_json::Value = serde_json::from_str(&raw).expect("parses");
        assert_eq!(parsed["version"], json!(TASK_STORE_VERSION));
        assert_eq!(parsed["tasks"][0]["id"], json!("work_one"));
    }

    #[test]
    fn a_saved_record_round_trips() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = WorkStore::builder()
            .file_path(dir.path().join("tasks.json"))
            .build();
        let record = WorkRecord {
            status: WorkStatus::Completed,
            kind: WorkKind::Reminder,
            result: Some("done".to_owned()),
            ..crate::testing::blank_record("work_one", "owner")
        };
        store.save(&[record.to_persisted(0)]);
        let loaded = store.load();
        assert_eq!(loaded, vec![record.to_persisted(0)]);
    }

    #[test]
    fn a_wrong_version_quarantines_and_starts_empty() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("tasks.json");
        std::fs::write(&path, "{\"version\": 2, \"tasks\": []}\n").expect("written");
        let store = WorkStore::builder()
            .file_path(&path)
            .locale(Locale::Zh)
            .build();

        assert!(store.load().is_empty());
        let health = store.health();
        assert!(!health.ok);
        let warning = health.warning.expect("a warning");
        assert!(
            warning.message.contains("任务状态文件格式无效"),
            "{}",
            warning.message
        );
        assert!(
            warning.message.contains("原文件已隔离为"),
            "{}",
            warning.message
        );
        let quarantine = warning.quarantine_path.expect("a quarantine path");
        assert!(quarantine.contains(".corrupt-"), "{quarantine}");
        assert!(std::path::Path::new(&quarantine).exists());
        assert!(!path.exists(), "the original was moved aside");
    }

    #[test]
    fn a_non_array_tasks_field_quarantines() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("tasks.json");
        std::fs::write(&path, "{\"version\": 1, \"tasks\": {}}\n").expect("written");
        let store = WorkStore::builder().file_path(&path).build();
        assert!(store.load().is_empty());
        assert!(!store.health().ok);
    }

    #[test]
    fn unparseable_json_quarantines_with_the_task_wording() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("tasks.json");
        std::fs::write(&path, "{not json").expect("written");
        let store = WorkStore::builder()
            .file_path(&path)
            .locale(Locale::Zh)
            .build();
        assert!(store.load().is_empty());
        let warning = store.health().warning.expect("a warning");
        assert!(
            warning.message.starts_with("任务状态文件不是有效的 JSON："),
            "{}",
            warning.message
        );
        assert!(
            warning.message.contains("服务将使用空任务状态继续运行。"),
            "{}",
            warning.message
        );
    }

    #[test]
    fn an_absent_file_is_not_a_fault() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = WorkStore::builder()
            .file_path(dir.path().join("tasks.json"))
            .build();
        assert!(store.load().is_empty());
        assert!(store.health().ok);
        assert!(store.health().persistence_enabled);
    }

    #[test]
    fn a_non_object_or_idless_entry_is_dropped() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("tasks.json");
        std::fs::write(
            &path,
            serde_json::to_string(&json!({
                "version": 1,
                "tasks": [7, null, "text", {}, {"id": ""}, {"id": "work_kept"}],
            }))
            .expect("serializes"),
        )
        .expect("written");
        let store = WorkStore::builder().file_path(&path).build();
        let loaded = store.load();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "work_kept");
        assert!(store.health().ok, "dropping a bad row is not a quarantine");
    }

    #[test]
    fn an_in_memory_store_never_writes() {
        let store = WorkStore::in_memory();
        assert!(!store.save(&[persisted("work_one")]));
        assert!(store.load().is_empty());
        assert!(!store.persistence_enabled());
        assert!(store.health().ok);
    }

    #[tokio::test(start_paused = true)]
    async fn a_burst_of_deferred_saves_costs_one_write() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("tasks.json");
        let store = WorkStore::builder()
            .file_path(&path)
            .self_scheduling(true)
            .build();

        store.save_deferred(&[persisted("work_one")]);
        store.save_deferred(&[persisted("work_two")]);
        store.save_deferred(&[persisted("work_three")]);
        assert!(!path.exists(), "nothing is written before the delay");

        tokio::time::sleep(super::DEFERRED_DELAY * 4).await;
        tokio::task::yield_now().await;

        let loaded = store.load();
        assert_eq!(loaded.len(), 1);
        assert_eq!(
            loaded[0].id, "work_three",
            "only the last state reaches disk"
        );
    }

    #[test]
    fn a_synchronous_save_supersedes_a_pending_deferred_one() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("tasks.json");
        let store = WorkStore::builder().file_path(&path).build();

        store.save_deferred(&[persisted("work_stale")]);
        assert!(store.save(&[persisted("work_fresh")]));
        assert!(!store.has_pending_deferred());
        store.flush();

        let loaded = store.load();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "work_fresh");
    }

    #[test]
    fn every_locale_renders_all_six_warnings() {
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            let messages = task_store_messages(locale);
            for rendered in [
                (messages.invalid_json)("L", "D"),
                (messages.invalid_shape)("L"),
                (messages.read_failed)("L", "D"),
                (messages.save_failed)("L", "D"),
                (messages.quarantined)("R", "P"),
                (messages.quarantine_failed)("R", "D"),
            ] {
                assert!(!rendered.is_empty());
                assert!(
                    !rendered.contains("<via-i18n:"),
                    "{locale}: {rendered} — a placeholder was not filled",
                );
            }
        }
    }
}
