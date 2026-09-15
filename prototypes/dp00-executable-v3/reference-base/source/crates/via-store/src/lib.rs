//! Durable JSON persistence for VIA.
//!
//! A leaf crate: no runtime, no logging backend, no VIA types. Everything that
//! needs a clock or a timer takes one as a parameter.
//!
//! Four behaviours, all ported rather than invented, because each one is a file
//! on disk that some other process may already be reading:
//!
//! 1. **Atomic write.** Temp file beside the target, `fsync`, rename, `fsync`
//!    the directory. The temp filename is itself a contract:
//!    `<path>.<pid>.tmp` for a synchronous save and
//!    `<path>.<pid>.<generation>.tmp` for a coalesced one.
//! 2. **Schema versioning.** `version` is the document's first key, and a
//!    mismatch either runs the [`MigrateFn`] or quarantines.
//! 3. **Quarantine on corrupt.** A document that will not parse is moved to
//!    `<path>.corrupt-<epoch ms>` and the store starts fresh with a warning.
//!    **If the quarantine itself fails, persistence is disabled rather than
//!    the original clobbered** — the bytes we could not move aside are the only
//!    copy of the user's state.
//! 4. **Coalesced saves.** A burst of mutations costs one write of the last
//!    state. VIA runs on phones with a 500 ms progress cadence
//!    (`docs/architecture.md` §4), where an uncoalesced write per tick is a
//!    battery and flash-wear cost.
//!
//! Plus [`with_file_transaction`], the `mkdir`-based cross-process lock that
//! lets a Node Gateway and VIA share the same profile files during migration.
//!
//! ```
//! use serde_json::json;
//! use via_store::{StoreMessages, VersionedJsonStore};
//!
//! let dir = tempfile::TempDir::new()?;
//! let store = VersionedJsonStore::builder()
//!     .file_path(dir.path().join("tasks.json"))
//!     .label("任务状态")
//!     .messages(StoreMessages::TASK_STORE)
//!     .build();
//!
//! store.save(&json!({ "tasks": [] }));
//! let document = store.load(|d| d.get("tasks").is_some_and(|t| t.is_array()));
//! assert_eq!(document.and_then(|d| d.get("version").cloned()), Some(json!(1)));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Ported from `qwen-audio-agent` v1.11.0:
//! `server/src/core/versioned-json-store.mjs`,
//! `server/src/task/task-store.mjs` and `shared/file-transaction-lock.mjs`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod atomic;
mod lock;
mod messages;
mod store;

pub use atomic::{
    FILE_MODE, LOCK_DIR_MODE, REPLACE_BACKUP_PREFIX, REPLACE_BACKUP_SUFFIX, ReplaceOptions,
    commit_temp_file, replace_file, replace_file_with, write_atomic, write_temp_file,
};
pub use lock::{
    LOCK_DIR_SUFFIX, LOCK_OWNER_FILE_NAME, LOCK_RELEASED_PREFIX, LOCK_STALE_INFIX, LockError,
    LockGuard, LockOptions, LockOwner, SHARED_FILE_BUSY, acquire, lock_path, lock_stale_path,
    with_file_transaction,
};
pub use messages::StoreMessages;
pub use store::{
    DEFAULT_DEFERRED_DELAY, DEFAULT_LABEL, DEFAULT_VERSION, DeferredWrite, MigrateFn, NowFn,
    ScheduleFn, StoreHealth, StoreWarning, VersionedJsonStore, VersionedJsonStoreBuilder,
    WarningFn,
};
