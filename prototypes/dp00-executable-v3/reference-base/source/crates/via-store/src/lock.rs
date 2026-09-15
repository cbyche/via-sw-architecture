//! The cross-process file transaction lock.
//!
//! Ported from `shared/file-transaction-lock.mjs`. Shared profile files
//! (`frontend-notes.json`, the memory Markdown documents, `config.env`) are
//! deliberately writable by more than one Gateway, so every read-modify-write
//! runs inside one transaction and independent runtimes cannot silently
//! overwrite each other.
//!
//! **The primitive is `mkdir`, and that is a compatibility surface, not an
//! implementation detail.** The other side of this lock may be a Node Gateway,
//! an older VIA, or the CLI, and `std::fs::File::lock` (advisory `flock`) would
//! not exclude any of them: two processes using different primitives on the
//! same file exclude nothing. Creating a directory is atomic on POSIX and on
//! Windows, needs no open handle to leak, and — unlike a lock *file* — never
//! exposes a partially written owner record, because the record is written
//! after the directory already exists. Every observable artifact below is
//! catalogued as *"file transaction lock parameters and on-disk artifacts"*.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::atomic::{create_dir_all_mode, with_suffix};

/// Suffix that turns a target file path into its lock directory.
///
/// Contract — `shared/file-transaction-lock.mjs:12`: the lock for
/// `<filePath>` is the **directory** `<filePath>.lock`.
pub const LOCK_DIR_SUFFIX: &str = ".lock";

/// Name of the owner record inside the lock directory.
///
/// Contract — `shared/file-transaction-lock.mjs:13,57`.
pub const LOCK_OWNER_FILE_NAME: &str = "owner.json";

/// Infix of the path a reclaimed lock is renamed to before deletion:
/// `<lockPath>.stale.<token>`.
///
/// Contract — `shared/file-transaction-lock.mjs:37`. The rename is what makes
/// reclaiming atomic, and the renamed directory is an artifact a crash can
/// leave behind, so the name is external.
pub const LOCK_STALE_INFIX: &str = ".stale.";

/// Prefix applied to the token when a lock is released rather than reclaimed,
/// producing `<lockPath>.stale.released.<token>`.
///
/// Contract — `shared/file-transaction-lock.mjs:100`. The two spellings let a
/// human reading a directory listing tell an orderly release apart from a
/// stale-lock reclaim.
pub const LOCK_RELEASED_PREFIX: &str = "released.";

/// The lock directory for `file_path`: `<file_path>.lock`.
///
/// Contract — `shared/file-transaction-lock.mjs:12`. Public because the
/// directory is observable and another process may be inspecting it.
#[must_use]
pub fn lock_path(file_path: &Path) -> PathBuf {
    with_suffix(file_path, LOCK_DIR_SUFFIX)
}

/// Where a lock directory is renamed before it is removed:
/// `<lock_path>.stale.<token>`.
///
/// Contract — `shared/file-transaction-lock.mjs:37,100`. Release passes
/// `released.<token>` as the token, which is why the two artifacts differ.
#[must_use]
pub fn lock_stale_path(lock_path: &Path, token: &str) -> PathBuf {
    with_suffix(lock_path, &format!("{LOCK_STALE_INFIX}{token}"))
}

/// The owner record inside the lock directory.
///
/// Contract — `shared/file-transaction-lock.mjs:57,63`: `owner.json` holds
/// `` `${JSON.stringify({ token, pid, createdAt })}\n` `` at mode `0o600`. Key
/// order is `token`, `pid`, `createdAt`, and the JSON is compact, not indented.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockOwner {
    /// The acquisition's unique token. Only its holder may release the lock.
    pub token: String,
    /// The owning process id. **Diagnostic only** — see [`LockOptions::stale`].
    pub pid: u32,
    /// Epoch milliseconds at acquisition.
    pub created_at: i64,
}

/// The error code a caller branches on when a transaction times out.
///
/// Contract — `shared/file-transaction-lock.mjs:88`
/// (`error.code = 'shared_file_busy'`). Brand-free, so `docs/rebrand.md` keeps
/// it verbatim: it is thrown across the Desktop/CLI boundary and a Node
/// Gateway on the other side matches this exact string.
pub const SHARED_FILE_BUSY: &str = "shared_file_busy";

/// Acquisition parameters.
///
/// Contract — `shared/file-transaction-lock.mjs:46-49`: `timeoutMs = 2000`,
/// `retryMs = 10`, `staleMs = 30_000`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LockOptions {
    /// How long to wait for a busy lock before giving up.
    pub timeout: Duration,
    /// How long to sleep between attempts.
    pub retry: Duration,
    /// How old a lock must be before it is reclaimed.
    pub stale: Duration,
}

impl Default for LockOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(2000),
            retry: Duration::from_millis(10),
            stale: Duration::from_millis(30_000),
        }
    }
}

/// Why a transaction could not be opened.
#[derive(Debug, thiserror::Error)]
pub enum LockError {
    /// Another process held the lock for the whole timeout.
    ///
    /// Contract — `shared/file-transaction-lock.mjs:87-89`: the message is
    /// `` `timed out waiting for shared file lock: ${filePath}` `` and the
    /// error carries `code = 'shared_file_busy'`, which callers branch on
    /// across the Desktop/CLI boundary.
    #[error("timed out waiting for shared file lock: {path}")]
    Busy {
        /// The **target file**, not the lock directory — upstream interpolates
        /// `filePath`.
        path: String,
    },
    /// The lock directory could not be created, written or read.
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl LockError {
    /// The stable error code, where one exists.
    ///
    /// Contract — catalogued *"file transaction lock timeout code"*.
    #[must_use]
    pub fn code(&self) -> Option<&'static str> {
        match self {
            Self::Busy { .. } => Some(SHARED_FILE_BUSY),
            Self::Io(_) => None,
        }
    }
}

/// A held transaction. Releasing on drop is what makes `?` in the guarded
/// section safe.
#[derive(Debug)]
pub struct LockGuard {
    lock_path: PathBuf,
    token: String,
    released: bool,
}

impl LockGuard {
    /// The lock directory, `<filePath>.lock`.
    #[must_use]
    pub fn lock_path(&self) -> &Path {
        &self.lock_path
    }

    /// This acquisition's token.
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Release early.
    ///
    /// Returns `false` when the lock was already reclaimed by someone else —
    /// upstream returns the same boolean, because releasing a lock another
    /// process now owns would be worse than reporting it.
    pub fn release(mut self) -> bool {
        self.release_in_place()
    }

    fn release_in_place(&mut self) -> bool {
        if self.released {
            return false;
        }
        self.released = true;
        match read_owner(&self.lock_path) {
            Some(owner) if owner.token == self.token => reclaim(
                &self.lock_path,
                &format!("{LOCK_RELEASED_PREFIX}{}", self.token),
            ),
            _ => false,
        }
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        self.release_in_place();
    }
}

/// Read the owner record, tolerating a lock written by an older release.
///
/// Contract — `shared/file-transaction-lock.mjs:16-25`: try
/// `<lockPath>/owner.json` first, then `<lockPath>` itself, so a stale *file*
/// lock from before the representation changed to a directory stays
/// recoverable.
fn read_owner(lock_path: &Path) -> Option<LockOwner> {
    let candidates = [
        lock_path.join(LOCK_OWNER_FILE_NAME),
        lock_path.to_path_buf(),
    ];
    for candidate in candidates {
        if let Ok(raw) = fs::read_to_string(&candidate)
            && let Ok(owner) = serde_json::from_str::<LockOwner>(&raw)
        {
            return Some(owner);
        }
    }
    None
}

/// Rename the lock aside and delete it. Renaming first means the slot is free
/// the instant the rename returns, so the winner of a race never waits on
/// someone else's `rm -rf`.
fn reclaim(lock_path: &Path, token: &str) -> bool {
    let stale_path = lock_stale_path(lock_path, token);
    if fs::rename(lock_path, &stale_path).is_err() {
        return false;
    }
    // `rmSync(..., { recursive: true, force: true })`: the legacy file-lock
    // shape is not a directory, so both removals are attempted and neither
    // failure matters.
    if fs::remove_dir_all(&stale_path).is_err() {
        let _ = fs::remove_file(&stale_path);
    }
    true
}

/// Whether an existing lock is old enough to take.
///
/// Contract — `shared/file-transaction-lock.mjs:27-34`. Reclaim is **by age,
/// not by PID probing**: PID visibility differs across hosts and containers,
/// and a PID that has been recycled can make two live processes both believe
/// they own the same transaction. Transactions here are synchronous and short,
/// so an abandoned lock is recovered after the bounded stale interval. An
/// unstattable lock counts as stale.
fn is_stale(lock_path: &Path, now_ms: i64, stale: Duration) -> bool {
    let modified = fs::metadata(lock_path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .and_then(|elapsed| i64::try_from(elapsed.as_millis()).ok());

    match modified {
        Some(mtime_ms) => {
            now_ms.saturating_sub(mtime_ms) >= i64::try_from(stale.as_millis()).unwrap_or(i64::MAX)
        }
        None => true,
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// Acquire the transaction lock for `file_path`.
///
/// The lock lives at `<file_path>.lock` — a directory at mode `0o700`
/// containing `owner.json` at mode `0o600`.
///
/// # Errors
///
/// [`LockError::Busy`] when the timeout elapses with the lock still held;
/// [`LockError::Io`] when the lock directory cannot be created or written.
pub fn acquire(file_path: &Path, options: LockOptions) -> Result<LockGuard, LockError> {
    let lock_path = lock_path(file_path);
    let token = uuid::Uuid::new_v4().to_string();
    let deadline = now_ms().saturating_add(i64::try_from(options.timeout.as_millis()).unwrap_or(0));

    if let Some(parent) = file_path.parent()
        && !parent.as_os_str().is_empty()
    {
        create_dir_all_mode(parent)?;
    }

    loop {
        match create_lock_dir(&lock_path) {
            Ok(()) => {
                let owner = LockOwner {
                    token: token.clone(),
                    pid: std::process::id(),
                    created_at: now_ms(),
                };
                if let Err(error) = write_owner(&lock_path, &owner) {
                    // Never leave a lock nobody can identify: an owner record
                    // that failed to write would deadlock every later acquirer
                    // until the stale interval.
                    let _ = fs::remove_dir_all(&lock_path);
                    return Err(LockError::Io(error));
                }
                return Ok(LockGuard {
                    lock_path,
                    token,
                    released: false,
                });
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(LockError::Io(error)),
        }

        let now = now_ms();
        if is_stale(&lock_path, now, options.stale) && reclaim(&lock_path, &token) {
            continue;
        }
        if now >= deadline {
            return Err(LockError::Busy {
                path: file_path.display().to_string(),
            });
        }
        let remaining = u64::try_from(deadline.saturating_sub(now))
            .unwrap_or(1)
            .max(1);
        thread::sleep(Duration::from_millis(
            u64::try_from(options.retry.as_millis())
                .unwrap_or(u64::MAX)
                .min(remaining),
        ));
    }
}

/// Non-recursive `mkdir`, which is the atomic test-and-set.
fn create_lock_dir(lock_path: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(crate::atomic::LOCK_DIR_MODE);
    }
    builder.create(lock_path)
}

fn write_owner(lock_path: &Path, owner: &LockOwner) -> io::Result<()> {
    use std::io::Write;

    let body = serde_json::to_string(owner)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(crate::atomic::FILE_MODE);
    }
    let mut file = options.open(lock_path.join(LOCK_OWNER_FILE_NAME))?;
    writeln!(file, "{body}")?;
    file.sync_all()
}

/// Run `action` inside one cross-process transaction on `file_path`.
///
/// A `None` path runs `action` unlocked — upstream's
/// `if (!filePath) return action()`, which keeps an unconfigured surface
/// working instead of failing.
///
/// # Errors
///
/// [`LockError::Busy`] or [`LockError::Io`] from [`acquire`]. `action` is not
/// run if the lock cannot be taken.
pub fn with_file_transaction<T, F>(
    file_path: Option<&Path>,
    options: LockOptions,
    action: F,
) -> Result<T, LockError>
where
    F: FnOnce() -> T,
{
    let Some(file_path) = file_path else {
        return Ok(action());
    };
    let guard = acquire(file_path, options)?;
    let outcome = action();
    drop(guard);
    Ok(outcome)
}
