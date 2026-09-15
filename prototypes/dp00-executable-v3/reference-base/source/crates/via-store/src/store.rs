//! [`VersionedJsonStore`] — the durable JSON document.
//!
//! Ported from `server/src/core/versioned-json-store.mjs` (load / save /
//! quarantine / health) and `server/src/task/task-store.mjs` (the
//! generation-tagged coalesced write). Upstream keeps those in two files
//! because JavaScript has no way to share the state machine without sharing the
//! warning strings; here the strings are a parameter ([`StoreMessages`]) and
//! the machine is written once.

use std::fmt;
use std::fs;
use std::io;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::atomic::{commit_temp_file, with_suffix, write_temp_file};
use crate::messages::StoreMessages;

/// Epoch-millisecond clock, injectable because it decides the observable
/// quarantine filename (`<path>.corrupt-<now>`).
pub type NowFn = Arc<dyn Fn() -> i64 + Send + Sync>;

/// Diagnostics sink. Upstream defaults to a no-op in
/// `versioned-json-store.mjs:15` and to `console.warn` in `task-store.mjs:21`.
pub type WarningFn = Arc<dyn Fn(&StoreWarning) + Send + Sync>;

/// Schema migration hook.
///
/// Called with the `version` found on disk and the parsed document when that
/// version is not the store's own. Returning `Some(document)` accepts the
/// migrated document (its `version` is then forced to the store's version and
/// the caller's `validate` still runs); returning `None` falls through to
/// quarantine, which is upstream's only behaviour — upstream has no hook, so a
/// store built without one is byte-for-byte upstream.
pub type MigrateFn =
    Arc<dyn Fn(Option<u64>, Map<String, Value>) -> Option<Map<String, Value>> + Send + Sync>;

/// "Call this back after `delay`."
///
/// The coalescing scheme needs a timer but this is a leaf crate, so the timer
/// is the host's. A tokio host writes
/// `Arc::new(|delay, write| { tokio::spawn(async move { sleep(delay).await; write.run(); }); })`.
pub type ScheduleFn = Arc<dyn Fn(Duration, DeferredWrite) + Send + Sync>;

/// A pending coalesced write handed to the host's timer.
///
/// Running it after the delay performs the write **if it is still the newest**
/// scheduled write; a stale one is a no-op, which is how upstream's
/// `clearTimeout` is reproduced without the store owning a cancellable timer.
/// Dropping it without running it simply cancels it — the content stays pending
/// until the next schedule or a [`flush`](VersionedJsonStore::flush).
pub struct DeferredWrite {
    run: Box<dyn FnOnce() + Send + 'static>,
}

impl DeferredWrite {
    /// Perform the write if this is still the newest scheduled one.
    pub fn run(self) {
        (self.run)()
    }
}

impl fmt::Debug for DeferredWrite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DeferredWrite")
    }
}

/// A persistence warning, as it appears on `/api/health`.
///
/// Contract — `server/src/core/versioned-json-store.mjs:29` and the catalogued
/// */api/health.notes and /api/health.taskStore*: `warning` is
/// `{message, quarantinePath, at}` or `null`. Field order is the serialization
/// order below.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreWarning {
    /// The human-readable sentence, from [`StoreMessages`].
    pub message: String,
    /// Where the original was moved to, or `null` when nothing was quarantined.
    pub quarantine_path: Option<String>,
    /// Epoch milliseconds, from the store's clock.
    pub at: i64,
}

/// The store's health triple.
///
/// Contract — `server/src/core/versioned-json-store.mjs:92-98`:
/// `{ok, persistenceEnabled, warning}`, in that order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreHealth {
    /// `true` while no warning has ever been raised.
    pub ok: bool,
    /// `true` when a path is configured and persistence has not been disabled.
    pub persistence_enabled: bool,
    /// The most recent warning, or `null`.
    pub warning: Option<StoreWarning>,
}

#[derive(Debug, Default)]
struct State {
    warning: Option<StoreWarning>,
    persistence_disabled: bool,
    deferred_content: Option<String>,
    write_generation: u64,
    schedule_seq: u64,
}

struct Inner {
    file_path: Option<PathBuf>,
    version: u64,
    label: String,
    messages: StoreMessages,
    now: NowFn,
    on_warning: WarningFn,
    migrate: Option<MigrateFn>,
    deferred_delay: Duration,
    schedule: Option<ScheduleFn>,
    state: Mutex<State>,
    /// Serializes the actual file operations. Upstream chains them onto one
    /// promise (`task-store.mjs:149`); this is that chain.
    writes: Mutex<()>,
    self_ref: Weak<Inner>,
}

/// A JSON document on disk that survives corruption, partial writes and a
/// 500 ms mutation cadence.
///
/// Cheap to clone — every clone is the same store.
#[derive(Clone)]
pub struct VersionedJsonStore {
    inner: Arc<Inner>,
}

impl fmt::Debug for VersionedJsonStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VersionedJsonStore")
            .field("file_path", &self.inner.file_path)
            .field("version", &self.inner.version)
            .field("label", &self.inner.label)
            .field("health", &self.health())
            .finish()
    }
}

/// Builder for [`VersionedJsonStore`].
pub struct VersionedJsonStoreBuilder {
    file_path: Option<PathBuf>,
    version: u64,
    label: String,
    messages: StoreMessages,
    now: Option<NowFn>,
    on_warning: Option<WarningFn>,
    migrate: Option<MigrateFn>,
    deferred_delay: Duration,
    schedule: Option<ScheduleFn>,
}

impl fmt::Debug for VersionedJsonStoreBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VersionedJsonStoreBuilder")
            .field("file_path", &self.file_path)
            .field("version", &self.version)
            .field("label", &self.label)
            .field("deferred_delay", &self.deferred_delay)
            .finish_non_exhaustive()
    }
}

/// The default document version.
///
/// Contract — `versioned-json-store.mjs:12`, `task-store.mjs:15` and
/// `acp-session-registry.mjs:4` all use `1`.
pub const DEFAULT_VERSION: u64 = 1;

/// The default surface label, interpolated into four of the six warnings.
///
/// Contract — `server/src/core/versioned-json-store.mjs:13` (`label = '状态'`).
pub const DEFAULT_LABEL: &str = "状态";

/// The default coalescing delay.
///
/// Contract — `server/src/task/task-store.mjs:22` (`deferredDelayMs = 250`),
/// catalogued as *"TaskManager / TaskScheduler / TaskStore defaults"*.
pub const DEFAULT_DEFERRED_DELAY: Duration = Duration::from_millis(250);

impl Default for VersionedJsonStoreBuilder {
    fn default() -> Self {
        Self {
            file_path: None,
            version: DEFAULT_VERSION,
            label: DEFAULT_LABEL.to_owned(),
            messages: StoreMessages::DEFAULT,
            now: None,
            on_warning: None,
            migrate: None,
            deferred_delay: DEFAULT_DEFERRED_DELAY,
            schedule: None,
        }
    }
}

impl VersionedJsonStoreBuilder {
    /// Where the document lives. Upstream's default is `null`, which makes the
    /// store a working in-memory no-op rather than an error — a Gateway with no
    /// state directory still runs.
    #[must_use]
    pub fn file_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// The schema version written as the document's first key.
    #[must_use]
    pub fn version(mut self, version: u64) -> Self {
        self.version = version;
        self
    }

    /// The surface name interpolated into the warning strings.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// The warning string table. Defaults to [`StoreMessages::DEFAULT`].
    #[must_use]
    pub fn messages(mut self, messages: StoreMessages) -> Self {
        self.messages = messages;
        self
    }

    /// Override the clock. The quarantine filename embeds it, so a fixed clock
    /// makes that path deterministic.
    #[must_use]
    pub fn now(mut self, now: NowFn) -> Self {
        self.now = Some(now);
        self
    }

    /// Where warnings go. Never called more than once per warning.
    #[must_use]
    pub fn on_warning(mut self, on_warning: WarningFn) -> Self {
        self.on_warning = Some(on_warning);
        self
    }

    /// Install a [`MigrateFn`]. Without one, a version mismatch quarantines.
    #[must_use]
    pub fn migrate(mut self, migrate: MigrateFn) -> Self {
        self.migrate = Some(migrate);
        self
    }

    /// How long [`save_deferred`](VersionedJsonStore::save_deferred) waits.
    #[must_use]
    pub fn deferred_delay(mut self, delay: Duration) -> Self {
        self.deferred_delay = delay;
        self
    }

    /// Install the host's timer. Without one, deferred content is held until
    /// [`flush`](VersionedJsonStore::flush) — still coalescing, just never
    /// self-starting.
    #[must_use]
    pub fn schedule(mut self, schedule: ScheduleFn) -> Self {
        self.schedule = Some(schedule);
        self
    }

    /// Finish the store.
    #[must_use]
    pub fn build(self) -> VersionedJsonStore {
        let Self {
            file_path,
            version,
            label,
            messages,
            now,
            on_warning,
            migrate,
            deferred_delay,
            schedule,
        } = self;
        let now = now.unwrap_or_else(|| Arc::new(system_now_ms));
        let on_warning = on_warning.unwrap_or_else(|| Arc::new(|_: &StoreWarning| {}));
        let inner = Arc::new_cyclic(|self_ref| Inner {
            file_path,
            version,
            label,
            messages,
            now,
            on_warning,
            migrate,
            deferred_delay,
            schedule,
            state: Mutex::new(State::default()),
            writes: Mutex::new(()),
            self_ref: self_ref.clone(),
        });
        VersionedJsonStore { inner }
    }
}

fn system_now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

impl VersionedJsonStore {
    /// Start building a store.
    #[must_use]
    pub fn builder() -> VersionedJsonStoreBuilder {
        VersionedJsonStoreBuilder::default()
    }

    /// The configured path, or `None` for an in-memory store.
    #[must_use]
    pub fn file_path(&self) -> Option<&Path> {
        self.inner.file_path.as_deref()
    }

    /// The schema version this store reads and writes.
    #[must_use]
    pub fn version(&self) -> u64 {
        self.inner.version
    }

    /// The surface label used in warnings.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.inner.label
    }

    /// The temp path a synchronous [`save`](Self::save) stages through.
    ///
    /// Contract — `` `${filePath}.${process.pid}.tmp` ``
    /// (`server/src/core/versioned-json-store.mjs:79`,
    /// `server/src/task/task-store.mjs:95`; catalogued as *"tasks.json on-disk
    /// format"*). Public because `via-conformance` asserts the shape.
    #[must_use]
    pub fn sync_temp_path(&self) -> Option<PathBuf> {
        self.inner
            .file_path
            .as_deref()
            .map(|path| with_suffix(path, &format!(".{}.tmp", std::process::id())))
    }

    /// The temp path the coalesced write of `generation` stages through.
    ///
    /// Contract — `` `${filePath}.${process.pid}.${generation}.tmp` ``
    /// (`server/src/task/task-store.mjs:129`). The generation is in the name so
    /// two overlapping deferred writes cannot share a staging file.
    #[must_use]
    pub fn deferred_temp_path(&self, generation: u64) -> Option<PathBuf> {
        self.inner
            .file_path
            .as_deref()
            .map(|path| with_suffix(path, &format!(".{}.{generation}.tmp", std::process::id())))
    }

    /// The health triple surfaced on `/api/health`.
    #[must_use]
    pub fn health(&self) -> StoreHealth {
        let state = self.state();
        StoreHealth {
            ok: state.warning.is_none(),
            persistence_enabled: self.inner.file_path.is_some() && !state.persistence_disabled,
            warning: state.warning.clone(),
        }
    }

    /// The most recent warning, if any.
    #[must_use]
    pub fn warning(&self) -> Option<StoreWarning> {
        self.state().warning.clone()
    }

    /// Whether a write would currently reach the disk.
    #[must_use]
    pub fn persistence_enabled(&self) -> bool {
        let state = self.state();
        self.inner.file_path.is_some() && !state.persistence_disabled
    }

    /// Whether coalesced content is waiting to be written.
    #[must_use]
    pub fn has_pending_deferred(&self) -> bool {
        self.state().deferred_content.is_some()
    }

    /// Read the document.
    ///
    /// Returns `None` when the caller should fall back to an empty state —
    /// upstream's `fallback()`. That is the answer for an absent file (no
    /// warning), an unreadable file, a corrupt file, and a document whose
    /// `version` or shape `validate` rejects. In every case but the first the
    /// original is moved aside first; see [`Self::health`] for what happened.
    ///
    /// The returned map is the whole document, `version` key included.
    pub fn load<V>(&self, validate: V) -> Option<Map<String, Value>>
    where
        V: Fn(&Map<String, Value>) -> bool,
    {
        let path = self.inner.file_path.as_deref()?;
        if self.state().persistence_disabled {
            return None;
        }

        let raw = match fs::read_to_string(path) {
            Ok(raw) => raw,
            // An absent file is the normal first run, not a fault.
            Err(error) if error.kind() == io::ErrorKind::NotFound => return None,
            Err(error) => {
                // Not a parse failure: the bytes may be perfectly good and
                // simply unreachable. Quarantining here would destroy them.
                self.state().persistence_disabled = true;
                let message =
                    (self.inner.messages.read_failed)(&self.inner.label, &error.to_string());
                self.warn(message, None);
                return None;
            }
        };

        let parsed = match serde_json::from_str::<Value>(&raw) {
            Ok(parsed) => parsed,
            Err(error) => {
                let reason =
                    (self.inner.messages.invalid_json)(&self.inner.label, &error.to_string());
                self.quarantine(&reason);
                return None;
            }
        };

        // `parsed?.version !== this.version` in JS: a non-object has no
        // `version`, so it lands in the same branch as a version mismatch.
        let document = match parsed {
            Value::Object(document) => document,
            _ => {
                let reason = (self.inner.messages.invalid_shape)(&self.inner.label);
                self.quarantine(&reason);
                return None;
            }
        };

        let on_disk = document.get("version").and_then(Value::as_u64);
        let document = if on_disk == Some(self.inner.version) {
            document
        } else {
            match &self.inner.migrate {
                Some(migrate) => match migrate(on_disk, document) {
                    Some(mut migrated) => {
                        migrated.insert("version".into(), Value::from(self.inner.version));
                        migrated
                    }
                    None => {
                        let reason = (self.inner.messages.invalid_shape)(&self.inner.label);
                        self.quarantine(&reason);
                        return None;
                    }
                },
                None => {
                    let reason = (self.inner.messages.invalid_shape)(&self.inner.label);
                    self.quarantine(&reason);
                    return None;
                }
            }
        };

        if !validate(&document) {
            let reason = (self.inner.messages.invalid_shape)(&self.inner.label);
            self.quarantine(&reason);
            return None;
        }

        Some(document)
    }

    /// Write the document now, and cancel any pending coalesced write.
    ///
    /// Returns whether the bytes reached the disk. A terminal state change
    /// (work completed, notification delivered) uses this; a progress tick uses
    /// [`save_deferred`](Self::save_deferred).
    pub fn save<T: Serialize>(&self, value: &T) -> bool {
        if !self.persistence_enabled() {
            return false;
        }
        let Some(temp) = self.sync_temp_path() else {
            return false;
        };
        let Some(target) = self.inner.file_path.clone() else {
            return false;
        };

        let Some(content) = self.document(value) else {
            return false;
        };
        {
            let mut state = self.state();
            // Supersede any deferred write, in flight or merely scheduled:
            // this content is newer than anything they hold.
            state.deferred_content = None;
            state.schedule_seq = state.schedule_seq.wrapping_add(1);
            state.write_generation = state.write_generation.wrapping_add(1);
        }

        let _writes = self.writes();
        match write_temp_file(&temp, &content).and_then(|()| commit_temp_file(&temp, &target)) {
            Ok(()) => true,
            Err(error) => {
                let _ = fs::remove_file(&temp);
                self.state().persistence_disabled = true;
                let message =
                    (self.inner.messages.save_failed)(&self.inner.label, &error.to_string());
                self.warn(message, None);
                false
            }
        }
    }

    /// Record the document as the newest state and schedule one write.
    ///
    /// Each call replaces whatever was pending, so a burst of mutations costs
    /// exactly one write of the *last* state — the intermediate ones are never
    /// serialized to disk at all. VIA runs on phones with a 500 ms progress
    /// cadence; uncoalesced writes there are a battery and a flash-wear cost.
    pub fn save_deferred<T: Serialize>(&self, value: &T) {
        if !self.persistence_enabled() {
            return;
        }
        let Some(content) = self.document(value) else {
            return;
        };

        let seq = {
            let mut state = self.state();
            state.deferred_content = Some(content);
            state.schedule_seq = state.schedule_seq.wrapping_add(1);
            state.schedule_seq
        };

        if let Some(schedule) = self.inner.schedule.clone() {
            let weak = self.inner.self_ref.clone();
            schedule(
                self.inner.deferred_delay,
                DeferredWrite {
                    run: Box::new(move || {
                        if let Some(inner) = weak.upgrade() {
                            VersionedJsonStore { inner }.run_scheduled(seq);
                        }
                    }),
                },
            );
        }
    }

    /// Write any pending coalesced content immediately, on this thread.
    ///
    /// Returns whether anything was written. Call it before shutting down.
    pub fn flush(&self) -> bool {
        {
            // Upstream's `clearTimeout`: a timer that has already been handed
            // to the host must find itself stale when it fires.
            let mut state = self.state();
            state.schedule_seq = state.schedule_seq.wrapping_add(1);
        }
        self.start_deferred_write()
    }

    /// A scheduled write firing. Stale tickets are dropped.
    fn run_scheduled(&self, seq: u64) {
        if self.state().schedule_seq != seq {
            return;
        }
        self.start_deferred_write();
    }

    fn start_deferred_write(&self) -> bool {
        let Some(target) = self.inner.file_path.clone() else {
            return false;
        };

        let (content, generation) = {
            let mut state = self.state();
            if state.persistence_disabled {
                return false;
            }
            let Some(content) = state.deferred_content.take() else {
                return false;
            };
            state.write_generation = state.write_generation.wrapping_add(1);
            (content, state.write_generation)
        };

        let Some(temp) = self.deferred_temp_path(generation) else {
            return false;
        };

        let _writes = self.writes();
        match write_temp_file(&temp, &content) {
            Ok(()) => {
                // The generation gate. Between taking the content and holding
                // the write lock a newer mutation may have landed; if so this
                // state is already history and must never reach the target.
                if self.state().write_generation != generation {
                    let _ = fs::remove_file(&temp);
                    return false;
                }
                match commit_temp_file(&temp, &target) {
                    Ok(()) => true,
                    Err(error) => {
                        let _ = fs::remove_file(&temp);
                        self.fail_deferred(generation, &error);
                        false
                    }
                }
            }
            Err(error) => {
                let _ = fs::remove_file(&temp);
                self.fail_deferred(generation, &error);
                false
            }
        }
    }

    /// A superseded write that failed is not evidence the disk is broken — the
    /// newer write is the one whose outcome matters.
    fn fail_deferred(&self, generation: u64, error: &io::Error) {
        {
            let mut state = self.state();
            if state.write_generation != generation {
                return;
            }
            state.persistence_disabled = true;
        }
        let message = (self.inner.messages.save_failed)(&self.inner.label, &error.to_string());
        self.warn(message, None);
    }

    /// `` `${JSON.stringify({ version, ...value }, null, 2)}\n` ``.
    ///
    /// Contract — the catalogued *"VersionedJsonStore on-disk format"*: two
    /// space indent, trailing newline, and `version` **first**, before the
    /// spread payload. `serde_json`'s `preserve_order` feature is what makes
    /// the key order reproducible; a `BTreeMap`-backed map would sort `version`
    /// into the middle.
    ///
    /// `None` means the payload could not be serialized at all, in which case
    /// nothing is written: a placeholder document would silently replace good
    /// state with an empty one.
    fn document<T: Serialize>(&self, value: &T) -> Option<String> {
        let mut document = Map::new();
        document.insert("version".into(), Value::from(self.inner.version));
        // A payload that is not a JSON object contributes no keys, which is
        // what spreading a primitive does in JavaScript.
        if let Ok(Value::Object(payload)) = serde_json::to_value(value) {
            // Re-inserting an existing key keeps its original position and
            // replaces the value, exactly as a JS object literal spread does.
            for (key, entry) in payload {
                document.insert(key, entry);
            }
        }
        let body = serde_json::to_string_pretty(&document).ok()?;
        Some(format!("{body}\n"))
    }

    fn quarantine(&self, reason: &str) {
        let Some(path) = self.inner.file_path.clone() else {
            return;
        };
        let quarantine_path = with_suffix(&path, &format!(".corrupt-{}", (self.inner.now)()));
        let shown = quarantine_path.display().to_string();

        match fs::rename(&path, &quarantine_path) {
            Ok(()) => {
                let message = (self.inner.messages.quarantined)(reason, &shown);
                self.warn(message, Some(shown));
            }
            Err(error) => {
                // The second-order rule: we could not move the original aside,
                // so we must not write over it either. Persistence off, file
                // intact, warning loud.
                self.state().persistence_disabled = true;
                let message = (self.inner.messages.quarantine_failed)(reason, &error.to_string());
                self.warn(message, None);
            }
        }
    }

    fn warn(&self, message: String, quarantine_path: Option<String>) {
        let warning = StoreWarning {
            message,
            quarantine_path,
            at: (self.inner.now)(),
        };
        self.state().warning = Some(warning.clone());

        let on_warning = self.inner.on_warning.clone();
        // Diagnostics must never prevent startup — upstream wraps the callback
        // in a bare `try {} catch {}` for exactly this reason
        // (`versioned-json-store.mjs:31-34`).
        let _ = std::panic::catch_unwind(AssertUnwindSafe(move || on_warning(&warning)));
    }

    /// A poisoned lock means some other thread panicked while holding it. The
    /// state it guards is a warning record and three counters; refusing to
    /// serve health because of it would turn one panic into a dead Gateway.
    fn state(&self) -> MutexGuard<'_, State> {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn writes(&self) -> MutexGuard<'_, ()> {
        self.inner
            .writes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
