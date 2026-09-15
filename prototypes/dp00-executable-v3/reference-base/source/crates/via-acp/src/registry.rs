//! The durable map from VIA's stable coordinator key to a backend's native
//! session id.
//!
//! Ported from `server/src/agent/acp-session-registry.mjs`. The two key
//! formats it is indexed by — `<protocol>:<encoded owner>:backend` and
//! `<protocol>:<session id>` — are [`via_downstream::SessionKey`], not this
//! crate's: they are the Layer-3 identity every harness shares, and a second
//! spelling of them would orphan stored sessions the first time the two
//! disagreed.
//!
//! # Why this file exists at all
//!
//! An ACP session id is minted by the backend and means nothing to VIA. A Work
//! item, a voice session and a Gateway restart all outlive it. The registry is
//! the join: a key VIA computes from *(protocol, owner)* on one side, the
//! backend's own id on the other, and enough context (`cwd`) to resume rather
//! than start over. Without it every restart is a new conversation, which the
//! user experiences as the assistant forgetting what it was doing.
//!
//! # The file is an upgrade path, not an implementation detail
//!
//! `state/acp-sessions.json` is catalogued as *"backend session state file"*
//! precisely so a VIA install can be pointed at an existing Node install's
//! configuration directory and keep its coordinator sessions. Every observable
//! property of that file — the two-space indent, the trailing newline, mode
//! `0600`, the `<path>.<pid>.tmp` staging name, the
//! `<path>.corrupt-<epoch ms>` quarantine name — belongs to
//! [`via_store::VersionedJsonStore`], which is why this module holds no file
//! I/O of its own.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use via_downstream::SessionKey;
use via_i18n::{Locale, keys};
use via_store::{StoreHealth, StoreMessages, VersionedJsonStore, WarningFn};

/// The schema version written into the document.
///
/// **External contract** — `acp-session-registry.mjs:4` (`const VERSION = 1`).
/// A document with any other version is quarantined, not migrated: the format
/// has had exactly one shape, and guessing at a future one would risk
/// clobbering a file a newer build wrote.
pub const VERSION: u64 = 1;

/// One coordinator session as it is stored.
///
/// **External contract** — field names and order are the on-disk format:
/// `{sessionId, cwd, updatedAt}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoordinatorRecord {
    /// The backend's own session id.
    pub session_id: String,
    /// The working directory the session was opened against.
    pub cwd: String,
    /// Epoch milliseconds of the last write.
    pub updated_at: i64,
}

/// One project session as it is stored.
///
/// **External contract** — `{sessionId, cwd, title, updatedAt}`. `title` has no
/// counterpart on [`CoordinatorRecord`]: a coordinator session has no title
/// because it is never listed to a user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRecord {
    /// The backend's own session id.
    pub session_id: String,
    /// The project directory. Recovering this from a bare session id is the
    /// reason `projects` exists.
    pub cwd: String,
    /// The backend-supplied title, or empty.
    pub title: String,
    /// Epoch milliseconds of the last write.
    pub updated_at: i64,
}

/// Epoch-millisecond clock, injectable because `updatedAt` is on disk.
pub type NowFn = std::sync::Arc<dyn Fn() -> i64 + Send + Sync>;

/// The persisted ACP session index.
///
/// Load is lazy and happens exactly once, as upstream's `loaded` flag is:
/// a quarantine must not run again on the second read.
pub struct AcpSessionRegistry {
    store: VersionedJsonStore,
    now: NowFn,
    state: std::sync::Mutex<State>,
}

#[derive(Debug, Default)]
struct State {
    loaded: bool,
    coordinators: BTreeMap<String, CoordinatorRecord>,
    projects: BTreeMap<String, ProjectRecord>,
}

/// Builder for [`AcpSessionRegistry`].
#[derive(Default)]
pub struct AcpSessionRegistryBuilder {
    file_path: Option<std::path::PathBuf>,
    locale: Locale,
    on_warning: Option<WarningFn>,
    now: Option<NowFn>,
}

impl std::fmt::Debug for AcpSessionRegistryBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcpSessionRegistryBuilder")
            .field("file_path", &self.file_path)
            .field("locale", &self.locale)
            .finish_non_exhaustive()
    }
}

impl AcpSessionRegistryBuilder {
    /// Where the index lives.
    ///
    /// Omitting it gives a working in-memory registry, which is upstream's
    /// `filePath = null` default: a Gateway with no configuration directory
    /// still runs, it just forgets its sessions on restart.
    #[must_use]
    pub fn file_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// The locale the store's label is rendered in.
    #[must_use]
    pub fn locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// Where persistence warnings go.
    ///
    /// Upstream logs `acp.session_index_persistence_warning`; the sink is a
    /// parameter here so a caller can also surface it on `/api/health`, which
    /// is where a person actually notices a quarantined index.
    #[must_use]
    pub fn on_warning(mut self, on_warning: WarningFn) -> Self {
        self.on_warning = Some(on_warning);
        self
    }

    /// Override the clock that stamps `updatedAt`.
    #[must_use]
    pub fn now(mut self, now: NowFn) -> Self {
        self.now = Some(now);
        self
    }

    /// Finish the registry.
    #[must_use]
    pub fn build(self) -> AcpSessionRegistry {
        let mut store = VersionedJsonStore::builder()
            .version(VERSION)
            .label(via_i18n::t(
                self.locale,
                keys::STORE_LABEL_ACP_SESSION_INDEX,
            ))
            .messages(StoreMessages::DEFAULT);
        if let Some(path) = self.file_path {
            store = store.file_path(path);
        }
        if let Some(on_warning) = self.on_warning {
            store = store.on_warning(on_warning);
        }
        AcpSessionRegistry {
            store: store.build(),
            now: self.now.unwrap_or_else(|| std::sync::Arc::new(now_ms)),
            state: std::sync::Mutex::new(State::default()),
        }
    }
}

impl std::fmt::Debug for AcpSessionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcpSessionRegistry")
            .field("store", &self.store)
            .finish_non_exhaustive()
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

impl AcpSessionRegistry {
    /// Start building a registry.
    #[must_use]
    pub fn builder() -> AcpSessionRegistryBuilder {
        AcpSessionRegistryBuilder::default()
    }

    /// The coordinator session for `key`, if one is recorded.
    ///
    /// A project key never matches: the two live in separate maps precisely so
    /// that a project session id colliding with an owner id cannot resume the
    /// wrong conversation.
    #[must_use]
    pub fn get(&self, key: &SessionKey) -> Option<CoordinatorRecord> {
        let mut state = self.lock();
        self.load(&mut state);
        state.coordinators.get(key.as_str()).cloned()
    }

    /// Record a coordinator session and persist immediately.
    ///
    /// **Contract** — `acp-session-registry.mjs:52-60`: the record is
    /// *replaced*, not merged, and `updatedAt` is taken now.
    pub fn set(&self, key: &SessionKey, session_id: &str, cwd: &str) {
        let mut state = self.lock();
        self.load(&mut state);
        state.coordinators.insert(
            key.as_str().to_owned(),
            CoordinatorRecord {
                session_id: session_id.to_owned(),
                cwd: cwd.to_owned(),
                updated_at: (self.now)(),
            },
        );
        self.save(&state);
    }

    /// Forget a coordinator session and persist immediately.
    ///
    /// Upstream saves even when the key was absent; so does this, because the
    /// write is also how a caller flushes a registry it believes is stale.
    pub fn delete(&self, key: &SessionKey) {
        let mut state = self.lock();
        self.load(&mut state);
        state.coordinators.remove(key.as_str());
        self.save(&state);
    }

    /// The project session for `key`, if one is recorded.
    #[must_use]
    pub fn get_project(&self, key: &SessionKey) -> Option<ProjectRecord> {
        let mut state = self.lock();
        self.load(&mut state);
        state.projects.get(key.as_str()).cloned()
    }

    /// Record one project session.
    pub fn set_project(&self, key: &SessionKey, session_id: &str, cwd: &str, title: &str) {
        self.set_projects(&[(key, session_id, cwd, title)]);
    }

    /// Record several project sessions in one write.
    ///
    /// **Contract** — `acp-session-registry.mjs:78-94`. Two behaviours that are
    /// easy to lose and both matter:
    ///
    /// * an entry with a blank session id **or** a blank `cwd` is skipped, not
    ///   stored blank — a project record whose `cwd` is unknown cannot be
    ///   resumed, so storing it would just produce a later failure;
    /// * if every entry is skipped, **nothing is written at all**, which is what
    ///   keeps a `session/list` response full of unusable entries from
    ///   rewriting the file on every poll.
    pub fn set_projects(&self, entries: &[(&SessionKey, &str, &str, &str)]) {
        let mut state = self.lock();
        self.load(&mut state);
        let mut changed = false;
        for (key, session_id, cwd, title) in entries {
            if session_id.is_empty() || cwd.is_empty() {
                continue;
            }
            state.projects.insert(
                key.as_str().to_owned(),
                ProjectRecord {
                    session_id: (*session_id).to_owned(),
                    cwd: (*cwd).to_owned(),
                    title: (*title).to_owned(),
                    updated_at: (self.now)(),
                },
            );
            changed = true;
        }
        if changed {
            self.save(&state);
        }
    }

    /// The store's health triple, as it appears on `/api/health`.
    #[must_use]
    pub fn health(&self) -> StoreHealth {
        self.store.health()
    }

    /// The path the index is persisted to, if any.
    #[must_use]
    pub fn file_path(&self) -> Option<&std::path::Path> {
        self.store.file_path()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        // A poisoned lock means a panic happened while the map was borrowed.
        // The map is plain data with no invariant a panic could have broken
        // halfway, so recovering it is strictly better than propagating a
        // panic into a voice turn.
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Upstream's `load()`: at most once, and quarantine is part of it.
    fn load(&self, state: &mut State) {
        if state.loaded {
            return;
        }
        state.loaded = true;
        let Some(document) = self.store.load(validate) else {
            return;
        };
        state.coordinators = read_map(&document, "coordinators");
        state.projects = read_map(&document, "projects");
    }

    fn save(&self, state: &State) {
        self.store.save(&Document {
            coordinators: &state.coordinators,
            projects: &state.projects,
        });
    }
}

/// The document as it is written.
///
/// **External contract** — key order is `coordinators` then `projects`, after
/// the store's own `version`.
#[derive(Serialize)]
struct Document<'a> {
    coordinators: &'a BTreeMap<String, CoordinatorRecord>,
    projects: &'a BTreeMap<String, ProjectRecord>,
}

/// `validate` — `acp-session-registry.mjs:31-40`.
///
/// `coordinators` must be present and a non-array object; `projects` may be
/// absent (a pre-`projects` file from an older install is valid and must load,
/// not quarantine) but if present must also be a non-array object.
fn validate(document: &Map<String, Value>) -> bool {
    let coordinators_ok = document
        .get("coordinators")
        .is_some_and(|value| value.is_object());
    let projects_ok = match document.get("projects") {
        None | Some(Value::Null) => true,
        Some(value) => value.is_object(),
    };
    coordinators_ok && projects_ok
}

/// Read one map, dropping members that do not deserialize.
///
/// A single malformed record must not cost the user every other session in the
/// file, and it must not quarantine either: the document as a whole is valid,
/// so upstream's `validate` accepts it and then reads members loosely
/// (`value && typeof value === 'object' ? {...value} : null`).
fn read_map<T: for<'de> Deserialize<'de>>(
    document: &Map<String, Value>,
    field: &str,
) -> BTreeMap<String, T> {
    document
        .get(field)
        .and_then(Value::as_object)
        .map(|map| {
            map.iter()
                .filter_map(|(key, value)| {
                    serde_json::from_value::<T>(value.clone())
                        .ok()
                        .map(|record| (key.clone(), record))
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coordinator() -> SessionKey {
        SessionKey::coordinator("example", "owner")
    }

    #[test]
    fn an_in_memory_registry_still_works() {
        let registry = AcpSessionRegistry::builder().build();
        registry.set(&coordinator(), "s", "/w");
        assert_eq!(
            registry.get(&coordinator()).map(|record| record.session_id),
            Some("s".to_owned())
        );
        assert!(registry.file_path().is_none());
        assert!(registry.health().ok);
        assert!(
            !registry.health().persistence_enabled,
            "there is no file to persist to"
        );
    }

    #[test]
    fn coordinator_and_project_maps_never_collide() {
        let registry = AcpSessionRegistry::builder().build();
        let project = SessionKey::project("example", "owner");
        registry.set(&coordinator(), "coord", "/coord");
        registry.set_project(&project, "proj", "/proj", "Project");
        assert_eq!(registry.get(&project), None);
        assert_eq!(registry.get_project(&coordinator()), None);
        assert_eq!(
            registry
                .get_project(&project)
                .map(|record| record.session_id),
            Some("proj".to_owned())
        );
    }

    #[test]
    fn a_project_entry_without_a_directory_is_skipped_entirely() {
        let registry = AcpSessionRegistry::builder().build();
        let key = SessionKey::project("example", "p");
        registry.set_project(&key, "s", "", "title");
        assert_eq!(registry.get_project(&key), None);
        registry.set_project(&key, "", "/w", "title");
        assert_eq!(registry.get_project(&key), None);
    }

    #[test]
    fn deleting_an_absent_coordinator_is_not_an_error() {
        let registry = AcpSessionRegistry::builder().build();
        registry.delete(&coordinator());
        assert_eq!(registry.get(&coordinator()), None);
    }

    #[test]
    fn validate_accepts_a_file_written_before_projects_existed() {
        let document: Map<String, Value> = serde_json::from_value(serde_json::json!({
            "version": 1,
            "coordinators": {},
        }))
        .expect("object");
        assert!(validate(&document));
    }

    #[test]
    fn validate_rejects_arrays_and_absent_coordinators() {
        for invalid in [
            serde_json::json!({ "version": 1 }),
            serde_json::json!({ "version": 1, "coordinators": [] }),
            serde_json::json!({ "version": 1, "coordinators": {}, "projects": [] }),
            serde_json::json!({ "version": 1, "coordinators": "no" }),
        ] {
            let document: Map<String, Value> =
                serde_json::from_value(invalid.clone()).expect("object");
            assert!(!validate(&document), "{invalid}");
        }
    }

    #[test]
    fn a_malformed_record_costs_only_itself() {
        let map: BTreeMap<String, CoordinatorRecord> = read_map(
            &serde_json::from_value(serde_json::json!({
                "coordinators": {
                    "good": { "sessionId": "s", "cwd": "/w", "updatedAt": 1 },
                    "bad": { "sessionId": "s" },
                    "worse": 7,
                },
            }))
            .expect("object"),
            "coordinators",
        );
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("good"));
    }
}
