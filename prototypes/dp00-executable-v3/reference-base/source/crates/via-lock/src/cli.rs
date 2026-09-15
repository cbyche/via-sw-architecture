//! The CLI instance lock — `<configDir>/cli.lock`.
//!
//! Ported from upstream `cli/src/instance-lock.mjs:11-66`.
//!
//! A second, *smaller* lock than the Gateway lease, and deliberately not the
//! same file. The Gateway lease says "one Gateway per configuration
//! directory"; this one says "one interactive CLI per configuration
//! directory", and the two are independent — a CLI attaches to a Gateway it
//! did not start, and a Gateway runs with no CLI attached at all.
//!
//! It is much simpler than [`crate::acquire_gateway_lease`] and the
//! differences are all upstream's, not simplifications:
//!
//! | | Gateway lease | CLI lock |
//! | --- | --- | --- |
//! | file | `gateway.lock` | `cli.lock` |
//! | document | eight fields, schema-tagged | `{"pid","token"}` |
//! | trailing newline | yes | **no** |
//! | attempts | 4 | 2 |
//! | stale reclaim | rename aside, then delete | delete |
//! | conflict carries a code | yes | no |
//!
//! The absent trailing newline is not a detail to tidy up: upstream writes
//! `JSON.stringify({ pid, token })` with no `\n`, and a byte-comparing test on
//! the other side of a migration would notice one appearing.

use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::probe::{ProcessProbe, SystemProcessProbe, current_pid, process_is_alive};

/// The lock file's name inside the configuration directory.
///
/// **External contract.** Upstream `cli/src/instance-lock.mjs:29` —
/// `resolve(configDirectory, 'cli.lock')`. Brand-free, so `docs/rebrand.md`
/// keeps it verbatim and the `~/.config` move stays a pure directory rename.
pub const CLI_LOCK_FILE_NAME: &str = "cli.lock";

/// Unix mode of the lock file: owner read/write only.
///
/// **External contract.** Upstream `openSync(path, 'wx', 0o600)`
/// (`cli/src/instance-lock.mjs:31`). As with `open(2)`, the effective mode is
/// `0o600 & !umask`. Not applied on Windows, which has no mode bits.
pub const CLI_LOCK_FILE_MODE: u32 = 0o600;

/// How many times acquisition will find and clear a dead holder before giving
/// up.
///
/// **External contract.** Upstream `cli/src/instance-lock.mjs:30` —
/// `for (let attempt = 0; attempt < 2; attempt += 1)`. Two, not four: unlike
/// the Gateway lease there is no rename-aside step, so a losing racer has
/// nothing to retry against and a third pass would only mask a livelock.
pub const MAX_CLI_ACQUIRE_ATTEMPTS: usize = 2;

/// `<configDirectory>/cli.lock`.
///
/// **External contract.** Upstream `cli/src/instance-lock.mjs:29`. As with
/// [`gateway_lock_path`](crate::gateway_lock_path) this joins rather than
/// absolutizing; callers pass the directory `VIA_CONFIG_DIR` resolution
/// produced.
#[must_use]
pub fn cli_lock_path(config_directory: &Path) -> PathBuf {
    config_directory.join(CLI_LOCK_FILE_NAME)
}

/// The document written to `cli.lock`.
///
/// **External contract — field order included.** Upstream writes
/// `JSON.stringify({ pid, token })` (`cli/src/instance-lock.mjs:32`), so the
/// on-disk bytes are `{"pid":<int>,"token":"<uuid>"}` with **no** trailing
/// newline. `serde` serializes in declaration order; do not reorder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliLockDocument {
    /// The holding process id, probed with `kill(pid, 0)`.
    ///
    /// `i64` for the same reason as
    /// [`GatewayLease::pid`](crate::GatewayLease::pid): a foreign or corrupt
    /// document must not make the whole read panic. Upstream coerces with
    /// `Number(existing?.pid)`, so a *string* pid naming a live process would
    /// block it and reads as dead here; a document that shape is already
    /// corrupt, and both sides then take the lock.
    #[serde(default)]
    pub pid: i64,
    /// This acquisition's unique token. Only its holder may release the lock.
    #[serde(default)]
    pub token: String,
}

/// Why the CLI instance lock could not be taken.
///
/// Upstream throws a plain `Error` for both refusals with **no `code`
/// property** (`cli/src/instance-lock.mjs:55,64`), which is why — unlike
/// [`LeaseError`](crate::LeaseError) — there is no `code()` accessor to branch
/// on here. Upstream's own callers match on the message.
#[derive(Debug, Error)]
pub enum CliLockError {
    /// Another CLI holds the lock and its process is alive.
    ///
    /// **External contract.** Upstream `cli/src/instance-lock.mjs:55`.
    //
    // The literal below is upstream's `zh` string with the CLI binary renamed
    // per `docs/rebrand.md`, and it stays a literal: `docs/architecture.md` §9
    // gives `shared -> nothing`, so this Leaf-band crate may not depend on
    // `via-i18n` (`via-arch-test::leaf_violations`). `lock.cli_already_running`
    // is its catalog twin, and `via-conformance`'s
    // `tests/localized_leaf_messages.rs` asserts the two are the same bytes.
    #[error("另一个 via CLI 已在运行")]
    AlreadyRunning {
        /// The incumbent document, as read from disk. `None` when the file was
        /// unreadable — upstream reaches the throw only with a live pid, so
        /// this is `Some` on every path upstream can take; it is an `Option`
        /// so the type cannot promise more than the file does.
        holder: Option<Box<CliLockDocument>>,
    },

    /// Both attempts found a lock and neither could be cleared.
    ///
    /// **External contract.** Upstream `cli/src/instance-lock.mjs:64`, thrown
    /// after [`MAX_CLI_ACQUIRE_ATTEMPTS`] passes.
    //
    // A literal for the reason above; `lock.cli_lease_exhausted` is its catalog
    // twin and `via-conformance` pins the two together.
    #[error("无法获取 via CLI 实例锁")]
    Exhausted,

    /// The filesystem refused an operation the lock depends on.
    ///
    /// Upstream rethrows the raw error for anything that is not `EEXIST` on
    /// create or `ENOENT` on unlink; this names the path as well.
    #[error("cli instance lock i/o failed at {path}")]
    Io {
        /// The path being created, written or removed.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: io::Error,
    },
}

impl CliLockError {
    fn io(path: &Path, source: io::Error) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            source,
        }
    }
}

/// Knobs for [`acquire_cli_instance`], defaulted as upstream defaults them
/// (`cli/src/instance-lock.mjs:22-27`).
#[derive(Debug)]
pub struct CliAcquireOptions {
    pid: i64,
    token: String,
    probe: Box<dyn ProcessProbe>,
}

impl Default for CliAcquireOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl CliAcquireOptions {
    /// This process's pid, a fresh UUID v4 token, and the platform liveness
    /// probe.
    ///
    /// Generates a random token, so two calls are not interchangeable.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pid: current_pid(),
            token: Uuid::new_v4().to_string(),
            probe: Box::new(SystemProcessProbe),
        }
    }

    /// The pid written into the lock and probed by any later challenger.
    #[must_use]
    pub fn pid(mut self, pid: i64) -> Self {
        self.pid = pid;
        self
    }

    /// Override the generated token.
    #[must_use]
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = token.into();
        self
    }

    /// Replace the liveness probe, which decides whether an incumbent lock is
    /// a conflict or a corpse.
    #[must_use]
    pub fn probe(mut self, probe: Box<dyn ProcessProbe>) -> Self {
        self.probe = probe;
        self
    }
}

/// A CLI instance lock this process currently holds.
///
/// As with [`GatewayLeaseHandle`](crate::GatewayLeaseHandle) there is
/// deliberately **no `Drop` impl** and [`release`](Self::release) consumes the
/// handle — see that type's documentation for why. The destructuring in
/// `release` is what makes the absence compiler-enforced.
#[derive(Debug)]
pub struct CliInstanceHandle {
    path: PathBuf,
    document: CliLockDocument,
}

impl CliInstanceHandle {
    /// The lock file this handle owns.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The document this process wrote.
    #[must_use]
    pub fn document(&self) -> &CliLockDocument {
        &self.document
    }

    /// Delete the lock file, if it is still ours.
    ///
    /// **External contract.** Upstream `release`
    /// (`cli/src/instance-lock.mjs:35-44`): the file is removed only when
    /// **both** `pid` and `token` still match. A missing, unparseable or
    /// replaced lock is not an error — it simply no longer belongs to this
    /// process — so this returns `false` rather than failing, exactly as
    /// upstream's bare `catch {}` does.
    #[must_use]
    pub fn release(self) -> bool {
        // Destructured field by field so that adding `impl Drop` later fails
        // to compile here (E0509) instead of silently deleting another CLI's
        // lock at the end of some unrelated scope.
        let Self { path, document } = self;

        let Some(current) = read_document(&path) else {
            return false;
        };
        if current.pid != document.pid || current.token != document.token {
            return false;
        }
        fs::remove_file(&path).is_ok()
    }
}

/// Take the CLI instance lock for `config_directory`.
///
/// **External contract.** Upstream `acquireCliInstance`
/// (`cli/src/instance-lock.mjs:21-65`):
///
/// 1. Create `cli.lock` with `O_EXCL` at mode [`CLI_LOCK_FILE_MODE`] — the
///    create *is* the lock — and write `{"pid","token"}` with no trailing
///    newline.
/// 2. On `EEXIST`, read the incumbent. If its pid is alive, fail with
///    [`CliLockError::AlreadyRunning`]. Otherwise delete the file and retry.
/// 3. At most [`MAX_CLI_ACQUIRE_ATTEMPTS`] passes, then
///    [`CliLockError::Exhausted`].
///
/// An **unreadable** incumbent counts as dead and is cleared, matching
/// upstream's `existing = null` fallback: a lock nobody can parse names
/// nobody.
///
/// Unlike [`acquire_gateway_lease`](crate::acquire_gateway_lease) this does not
/// create the configuration directory. Upstream does not either — the CLI
/// resolves an existing directory before it gets here — so a missing directory
/// surfaces as [`CliLockError::Io`] rather than being created behind the
/// caller's back.
///
/// # Errors
///
/// [`CliLockError::AlreadyRunning`] when a live CLI holds the lock,
/// [`CliLockError::Exhausted`] when every attempt found one, and
/// [`CliLockError::Io`] for any filesystem failure other than the expected
/// `EEXIST` on create and `ENOENT` on unlink.
pub fn acquire_cli_instance(
    config_directory: &Path,
    options: CliAcquireOptions,
) -> Result<CliInstanceHandle, CliLockError> {
    let path = cli_lock_path(config_directory);
    let CliAcquireOptions { pid, token, probe } = options;

    for _attempt in 0..MAX_CLI_ACQUIRE_ATTEMPTS {
        let document = CliLockDocument {
            pid,
            token: token.clone(),
        };
        match write_new_document(&path, &document) {
            Ok(()) => return Ok(CliInstanceHandle { path, document }),
            Err(source) if source.kind() == ErrorKind::AlreadyExists => {
                let existing = read_document(&path);
                let holder_pid = existing.as_ref().map_or(0, |document| document.pid);
                if process_is_alive(holder_pid, probe.as_ref()) {
                    return Err(CliLockError::AlreadyRunning {
                        holder: existing.map(Box::new),
                    });
                }
                match fs::remove_file(&path) {
                    Ok(()) => {}
                    Err(source) if source.kind() == ErrorKind::NotFound => {}
                    Err(source) => return Err(CliLockError::io(&path, source)),
                }
            }
            Err(source) => return Err(CliLockError::io(&path, source)),
        }
    }

    Err(CliLockError::Exhausted)
}

/// Read and parse `cli.lock`, or `None` for any failure.
///
/// **External contract.** Upstream wraps the read *and* the parse in one
/// `try` and falls back to `null` (`cli/src/instance-lock.mjs:48-52`), so a
/// missing file and a truncated one are the same answer.
fn read_document(path: &Path) -> Option<CliLockDocument> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Create `path` exclusively and write the document with **no** trailing
/// newline.
///
/// **External contract.** Upstream `openSync(path, 'wx', 0o600)` +
/// `writeFileSync(fd, JSON.stringify({ pid, token }), 'utf8')`
/// (`cli/src/instance-lock.mjs:31-33`). `create_new(true)` is the same
/// `O_WRONLY|O_CREAT|O_EXCL` syscall, and it is the mutual-exclusion primitive
/// the whole module rests on.
fn write_new_document(path: &Path, document: &CliLockDocument) -> io::Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(CLI_LOCK_FILE_MODE);
    }
    let mut file = options.open(path)?;
    let body = serde_json::to_string(document).map_err(io::Error::other)?;
    file.write_all(body.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::SignalOutcome;

    #[derive(Debug)]
    struct Fixed(SignalOutcome);

    impl ProcessProbe for Fixed {
        fn signal_zero(&self, _pid: i64) -> SignalOutcome {
            self.0
        }
    }

    fn dead() -> Box<dyn ProcessProbe> {
        Box::new(Fixed(SignalOutcome::NoSuchProcess))
    }

    fn alive() -> Box<dyn ProcessProbe> {
        Box::new(Fixed(SignalOutcome::Delivered))
    }

    #[test]
    fn the_document_has_no_trailing_newline() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let handle = acquire_cli_instance(
            dir.path(),
            CliAcquireOptions::new()
                .pid(4242)
                .token("tok-1")
                .probe(dead()),
        )
        .expect("acquire");

        let raw = fs::read_to_string(handle.path()).expect("read");
        assert_eq!(raw, r#"{"pid":4242,"token":"tok-1"}"#);
        assert!(!raw.ends_with('\n'), "upstream writes no trailing newline");
    }

    #[test]
    fn a_live_holder_refuses_and_a_dead_one_is_cleared() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let first = acquire_cli_instance(
            dir.path(),
            CliAcquireOptions::new().pid(1).token("a").probe(dead()),
        )
        .expect("first acquire");

        let refused = acquire_cli_instance(
            dir.path(),
            CliAcquireOptions::new().pid(2).token("b").probe(alive()),
        )
        .expect_err("a live holder must refuse");
        assert!(matches!(refused, CliLockError::AlreadyRunning { .. }));
        assert_eq!(refused.to_string(), "另一个 via CLI 已在运行");
        let CliLockError::AlreadyRunning { holder } = &refused else {
            panic!("expected a conflict");
        };
        assert_eq!(
            holder.as_deref(),
            Some(&CliLockDocument {
                pid: 1,
                token: "a".to_owned()
            })
        );

        // The same file, with the holder now dead, is cleared and retaken.
        let second = acquire_cli_instance(
            dir.path(),
            CliAcquireOptions::new().pid(2).token("b").probe(dead()),
        )
        .expect("a dead holder must be cleared");
        assert_eq!(second.document().pid, 2);

        // The first handle no longer owns the file, so its release is a no-op
        // and must not delete the second holder's lock.
        assert!(!first.release());
        assert!(second.path().exists());
        assert!(second.release());
    }

    #[test]
    fn an_unparseable_lock_counts_as_dead() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = cli_lock_path(dir.path());
        fs::write(&path, "{not json").expect("seed");

        let handle = acquire_cli_instance(
            dir.path(),
            CliAcquireOptions::new()
                .pid(9)
                .token("t")
                // A probe that must never be consulted would be stronger, but
                // upstream *does* call it with NaN; `alive` proves the pid
                // guard rejects the absent id before the probe can matter.
                .probe(alive()),
        )
        .expect("an unparseable lock names nobody");
        assert_eq!(handle.document().pid, 9);
    }

    #[test]
    fn release_requires_both_pid_and_token() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = cli_lock_path(dir.path());

        for replacement in [
            r#"{"pid":9,"token":"other"}"#,
            r#"{"pid":10,"token":"t"}"#,
            r#"{"pid":10,"token":"other"}"#,
        ] {
            let handle = acquire_cli_instance(
                dir.path(),
                CliAcquireOptions::new().pid(9).token("t").probe(dead()),
            )
            .expect("acquire");
            fs::write(&path, replacement).expect("overwrite");
            assert!(
                !handle.release(),
                "a lock replaced by {replacement} is not ours to delete"
            );
            assert!(path.exists(), "{replacement} must survive our release");
            fs::remove_file(&path).expect("clean up");
        }
    }

    #[test]
    fn a_missing_configuration_directory_is_an_io_error() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let missing = dir.path().join("absent");
        let error = acquire_cli_instance(&missing, CliAcquireOptions::new().probe(dead()))
            .expect_err("upstream does not create the directory either");
        assert!(matches!(error, CliLockError::Io { .. }));
    }

    #[test]
    fn the_attempt_bound_and_the_exhausted_message_are_upstream_s() {
        assert_eq!(MAX_CLI_ACQUIRE_ATTEMPTS, 2);
        // Reaching Exhausted needs a second process to recreate the lock
        // between our unlink and our create — upstream's branch is race-only
        // too. The message is asserted here so it cannot drift while the path
        // stays unexercised.
        assert_eq!(
            CliLockError::Exhausted.to_string(),
            "无法获取 via CLI 实例锁"
        );
    }

    #[cfg(unix)]
    #[test]
    fn the_lock_file_is_mode_600() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::TempDir::new().expect("tempdir");
        let handle =
            acquire_cli_instance(dir.path(), CliAcquireOptions::new().probe(dead())).expect("lock");
        let mode = fs::metadata(handle.path())
            .expect("metadata")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, CLI_LOCK_FILE_MODE);
    }
}
