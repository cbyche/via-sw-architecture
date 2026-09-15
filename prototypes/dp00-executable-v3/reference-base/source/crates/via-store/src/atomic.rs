//! Crash-atomic file replacement.
//!
//! The sequence is: write a temp file *in the target's own directory*, `fsync`
//! it, rename it over the target, then `fsync` the directory so the rename
//! itself is durable. Upstream (`writeFileSync` + `renameSync`) gets the
//! rename-for-atomicity part but skips both barriers; on a phone that loses
//! power mid-save the difference is a truncated `tasks.json`.
//!
//! [`replace_file`] ports `replaceFileSync` from
//! `shared/file-transaction-lock.mjs` lines 111-169.

use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

/// Mode for every file this crate creates: owner read/write only.
///
/// Contract — `server/src/core/versioned-json-store.mjs:80`,
/// `server/src/task/task-store.mjs:99` and
/// `shared/file-transaction-lock.mjs:63` all pass `{ mode: 0o600 }`.
///
/// The value is a contract on every platform; only its *enforcement* is
/// Unix-only, because Windows has no mode bits and the file inherits the
/// parent ACL instead.
pub const FILE_MODE: u32 = 0o600;

/// Mode for the lock directory and its parents.
///
/// Contract — `shared/file-transaction-lock.mjs:53,60` (`mode: 0o700`).
/// Enforced on Unix only, as [`FILE_MODE`].
pub const LOCK_DIR_MODE: u32 = 0o700;

/// Infix of the Windows replace backup: `<target>.replace.<uuid>.bak`.
///
/// Contract — `shared/file-transaction-lock.mjs:141`. An artifact a crash
/// during a Windows replace can leave on disk, so a cleanup tool needs the
/// exact shape. See [`replace_file_with`].
pub const REPLACE_BACKUP_PREFIX: &str = ".replace.";

/// Suffix of the Windows replace backup: `<target>.replace.<uuid>.bak`.
///
/// Contract — `shared/file-transaction-lock.mjs:141`.
pub const REPLACE_BACKUP_SUFFIX: &str = ".bak";

/// Append a suffix to a path without going through `str`, so a non-UTF-8 path
/// survives intact.
pub(crate) fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut raw = path.to_path_buf().into_os_string();
    raw.push(suffix);
    PathBuf::from(raw)
}

/// Create `path` and every missing parent, with mode `0o700` on Unix.
pub(crate) fn create_dir_all_mode(path: &Path) -> io::Result<()> {
    let mut builder = DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(LOCK_DIR_MODE);
    }
    builder.create(path)
}

/// Write `contents` to `temp`, creating the parent directory if needed, and
/// `fsync` the file before returning.
///
/// The caller owns the temp path because its *name* is an external contract —
/// see [`VersionedJsonStore::sync_temp_path`] and
/// [`VersionedJsonStore::deferred_temp_path`].
///
/// [`VersionedJsonStore::sync_temp_path`]: crate::VersionedJsonStore::sync_temp_path
/// [`VersionedJsonStore::deferred_temp_path`]: crate::VersionedJsonStore::deferred_temp_path
pub fn write_temp_file(temp: &Path, contents: &str) -> io::Result<()> {
    if let Some(parent) = temp.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(FILE_MODE);
    }

    let mut file = options.open(temp)?;
    file.write_all(contents.as_bytes())?;
    // Barrier 1: the bytes are on the device before anything points at them.
    file.sync_all()?;
    Ok(())
}

/// Move a written temp file over `target` and make the rename durable.
///
/// The directory `fsync` is best-effort and skipped on Windows, which has no
/// directory handle to sync; some Unix filesystems also reject `fsync` on a
/// directory with `EINVAL`. A failure here means "the rename may not survive a
/// power cut", not "the write failed" — the data is already visible to every
/// reader — so it must not be escalated into disabling persistence.
pub fn commit_temp_file(temp: &Path, target: &Path) -> io::Result<()> {
    replace_file(temp, target)?;
    #[cfg(not(windows))]
    if let Some(parent) = target.parent()
        && let Ok(dir) = fs::File::open(parent)
    {
        // Barrier 2: the directory entry itself.
        let _ = dir.sync_all();
    }
    Ok(())
}

/// Write `contents` to `target` atomically, staging through `temp`.
pub fn write_atomic(temp: &Path, target: &Path, contents: &str) -> io::Result<()> {
    write_temp_file(temp, contents)?;
    commit_temp_file(temp, target)
}

/// Retry policy for [`replace_file_with`].
///
/// Contract — `shared/file-transaction-lock.mjs:126-129`: `retries = 20`,
/// `retryMs = 10`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplaceOptions {
    /// How many times a sharing-violation rename is retried.
    pub retries: u32,
    /// How long to wait between retries.
    pub retry: Duration,
    /// Whether to apply the Windows replace ladder. Defaults to
    /// `cfg!(windows)`; settable so the ladder is reachable in tests on any
    /// host.
    pub windows_semantics: bool,
}

impl Default for ReplaceOptions {
    fn default() -> Self {
        Self {
            retries: 20,
            retry: Duration::from_millis(10),
            windows_semantics: cfg!(windows),
        }
    }
}

/// The error classes Windows raises when a reader or an antivirus scanner is
/// holding the destination.
///
/// Contract — `shared/file-transaction-lock.mjs:109`
/// (`['EACCES', 'EBUSY', 'EPERM']`).
fn is_sharing_violation(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        ErrorKind::PermissionDenied | ErrorKind::ResourceBusy
    )
}

fn rename_with_retries(source: &Path, target: &Path, options: &ReplaceOptions) -> io::Result<()> {
    let mut attempt = 0;
    loop {
        match fs::rename(source, target) {
            Ok(()) => return Ok(()),
            Err(error) if is_sharing_violation(&error) && attempt < options.retries => {
                attempt += 1;
                thread::sleep(options.retry);
            }
            Err(error) => return Err(error),
        }
    }
}

/// Replace `target` with `temp` using the default retry policy.
pub fn replace_file(temp: &Path, target: &Path) -> io::Result<()> {
    replace_file_with(temp, target, &ReplaceOptions::default())
}

/// Replace `target` with `temp`.
///
/// On Unix this is a bare `rename(2)`, which is atomic and replaces the
/// destination. Windows rename cannot consistently replace an existing
/// destination and can be delayed by a reader or antivirus, so the previous
/// file is first moved aside to `<target>.replace.<uuid>.bak` — a recoverable
/// backup — and rolled back if the second rename fails. Ported from
/// `replaceFileSync`, `shared/file-transaction-lock.mjs:126-169`.
pub fn replace_file_with(temp: &Path, target: &Path, options: &ReplaceOptions) -> io::Result<()> {
    if !options.windows_semantics {
        return fs::rename(temp, target);
    }

    match fs::rename(temp, target) {
        Ok(()) => return Ok(()),
        Err(error) if is_sharing_violation(&error) => {}
        Err(error) => return Err(error),
    }

    let backup = with_suffix(
        target,
        &format!(
            "{REPLACE_BACKUP_PREFIX}{}{REPLACE_BACKUP_SUFFIX}",
            uuid::Uuid::new_v4()
        ),
    );
    if let Err(error) = rename_with_retries(target, &backup, options) {
        if error.kind() != ErrorKind::NotFound {
            return Err(error);
        }
        // Nothing to preserve: the destination vanished between the two
        // attempts, so the plain move is all that is left to do.
        return rename_with_retries(temp, target, options);
    }

    match rename_with_retries(temp, target, options) {
        Ok(()) => {
            let _ = fs::remove_file(&backup);
            Ok(())
        }
        Err(error) => {
            // Keep the backup on disk when rollback itself is blocked.
            let _ = rename_with_retries(&backup, target, options);
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_atomic_leaves_no_temp_behind() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let target = dir.path().join("nested").join("doc.json");
        let temp = with_suffix(&target, ".9.tmp");

        write_atomic(&temp, &target, "{}\n").expect("write");

        assert_eq!(fs::read_to_string(&target).expect("read"), "{}\n");
        assert!(!temp.exists());
    }

    #[cfg(unix)]
    #[test]
    fn temp_file_is_created_mode_0600() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::TempDir::new().expect("tempdir");
        let temp = dir.path().join("doc.json.1.tmp");
        write_temp_file(&temp, "x").expect("write");

        let mode = fs::metadata(&temp).expect("metadata").permissions().mode();
        assert_eq!(mode & 0o777, FILE_MODE);
    }

    #[test]
    fn windows_ladder_replaces_an_existing_destination() {
        // rename(2) never raises a sharing violation on Unix, so the ladder
        // short-circuits on its first attempt; this asserts that the forced
        // path is still correct rather than exercising the backup dance.
        let dir = tempfile::TempDir::new().expect("tempdir");
        let target = dir.path().join("doc.json");
        let temp = dir.path().join("doc.json.tmp");
        fs::write(&target, "old").expect("seed");
        fs::write(&temp, "new").expect("seed");

        let options = ReplaceOptions {
            windows_semantics: true,
            ..ReplaceOptions::default()
        };
        replace_file_with(&temp, &target, &options).expect("replace");

        assert_eq!(fs::read_to_string(&target).expect("read"), "new");
        assert!(!temp.exists());
    }

    #[test]
    fn rename_with_retries_does_not_retry_an_unrelated_error() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let missing = dir.path().join("absent");
        let target = dir.path().join("target");

        let error = rename_with_retries(&missing, &target, &ReplaceOptions::default())
            .expect_err("must fail");
        assert_eq!(error.kind(), ErrorKind::NotFound);
    }
}
