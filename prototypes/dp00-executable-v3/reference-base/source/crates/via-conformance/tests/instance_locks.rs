//! The two locks that are not the Gateway lease: `cli.lock` and the shared
//! file transaction lock.
//!
//! Both are *interoperability* contracts before they are implementation
//! choices. The other side of either may be a Node Gateway, an older VIA or
//! the CLI, so the artifacts on disk — a directory rather than a file, an
//! `owner.json` with three keys in one order, a `cli.lock` with no trailing
//! newline — have to match byte for byte or the two sides exclude nothing at
//! all.
//!
//! The file transaction lock lives in `via-store` rather than `via-lock`:
//! `via-store` is the crate that reads and writes the shared profile
//! documents, and the lock exists to guard exactly those writes.

use std::fs;
use std::path::Path;
use std::time::Duration;

use pretty_assertions::assert_eq;
use serde_json::Value;

use via_conformance::expect_contract;
use via_lock::{
    CLI_LOCK_FILE_MODE, CLI_LOCK_FILE_NAME, CliAcquireOptions, CliLockError,
    MAX_CLI_ACQUIRE_ATTEMPTS, ProcessProbe, SignalOutcome, acquire_cli_instance, cli_lock_path,
};
use via_store::{
    FILE_MODE, LOCK_DIR_MODE, LOCK_DIR_SUFFIX, LOCK_OWNER_FILE_NAME, LOCK_RELEASED_PREFIX,
    LOCK_STALE_INFIX, LockError, LockOptions, LockOwner, REPLACE_BACKUP_PREFIX,
    REPLACE_BACKUP_SUFFIX, ReplaceOptions, SHARED_FILE_BUSY, acquire, lock_path, lock_stale_path,
};

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

/// Every backtick-delimited literal in a catalogued value, in order.
///
/// The catalogue quotes user-facing sentences with backticks when they contain
/// characters a quote would fight with; both CLI-lock messages are written
/// that way.
fn backticked(value: &str) -> Vec<&str> {
    value.split('`').skip(1).step_by(2).map(str::trim).collect()
}

#[test]
fn cli_instance_lock_file() {
    let contract = expect_contract("file-path", "CLI instance lock");
    let value = &contract.exact_value;

    // `<configDirectory>/cli.lock, mode 0600, content {"pid":<int>,"token":"<uuid>"}
    //  (no trailing newline)`
    assert!(value.starts_with("<configDirectory>/cli.lock"), "{value}");
    assert!(value.contains("mode 0600"), "{value}");
    assert!(value.contains("(no trailing newline)"), "{value}");
    assert_eq!(CLI_LOCK_FILE_NAME, "cli.lock");
    assert_eq!(CLI_LOCK_FILE_MODE, 0o600);
    assert_eq!(
        cli_lock_path(Path::new("/home/tester/.config/via")),
        Path::new("/home/tester/.config/via/cli.lock")
    );

    let dir = tempfile::TempDir::new().expect("tempdir");
    let handle = acquire_cli_instance(
        dir.path(),
        CliAcquireOptions::new()
            .pid(4242)
            .token("11111111-2222-3333-4444-555555555555")
            .probe(dead()),
    )
    .expect("an empty directory has no incumbent");
    assert_eq!(handle.path(), cli_lock_path(dir.path()));

    let raw = fs::read_to_string(handle.path()).expect("read");
    assert_eq!(
        raw, r#"{"pid":4242,"token":"11111111-2222-3333-4444-555555555555"}"#,
        "the document is compact, `pid` first, and has no trailing newline"
    );
    assert!(!raw.ends_with('\n'));

    // Key order, read back rather than assumed from the byte comparison alone.
    let parsed: Value = serde_json::from_str(&raw).expect("valid JSON");
    assert_eq!(
        parsed
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["pid", "token"]
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(handle.path())
            .expect("metadata")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, CLI_LOCK_FILE_MODE, "mode 0600");
    }

    assert!(handle.release(), "the lock is still ours");
    assert!(!cli_lock_path(dir.path()).exists());
}

#[test]
fn cli_lock_conflict_messages() {
    let contract = expect_contract("error-code", "CLI lock conflicts");
    let messages = backticked(&contract.exact_value);
    assert_eq!(messages.len(), 2, "{}", contract.exact_value);

    // docs/rebrand.md renames the upstream CLI binary name to `via`; nothing
    // else in either sentence moves. Deriving VIA's text from upstream's is
    // what makes a half-applied rename fail here.
    let upstream_cli_binary = "qwenaudio";
    let rebranded = |message: &str| message.replace(upstream_cli_binary, "via");
    assert!(
        messages.iter().all(|m| m.contains(upstream_cli_binary)),
        "both messages name the upstream binary: {messages:?}"
    );

    let dir = tempfile::TempDir::new().expect("tempdir");
    let _held = acquire_cli_instance(
        dir.path(),
        CliAcquireOptions::new().pid(1).token("a").probe(dead()),
    )
    .expect("first acquire");

    let refused = acquire_cli_instance(
        dir.path(),
        CliAcquireOptions::new().pid(2).token("b").probe(alive()),
    )
    .expect_err("a live holder refuses");
    assert!(matches!(refused, CliLockError::AlreadyRunning { .. }));
    assert_eq!(refused.to_string(), rebranded(messages[0]));
    assert_eq!(refused.to_string(), "另一个 via CLI 已在运行");

    // The exhausted message. Upstream reaches it only when a second process
    // recreates the lock between the unlink and the create, so the branch is
    // race-only on both sides; the string is asserted so it cannot drift while
    // the path stays unexercised.
    assert_eq!(CliLockError::Exhausted.to_string(), rebranded(messages[1]));
    assert_eq!(
        CliLockError::Exhausted.to_string(),
        "无法获取 via CLI 实例锁"
    );
    assert!(
        contract.exact_value.contains("two failed attempts"),
        "the catalogue still names the attempt bound"
    );
    assert_eq!(MAX_CLI_ACQUIRE_ATTEMPTS, 2);

    // Upstream throws a bare `Error` for both: no `code` property, so nothing
    // in this crate may invent one.
    assert!(
        !contract.exact_value.contains("code"),
        "upstream attaches no error code to either CLI-lock refusal"
    );
}

#[test]
fn shared_file_transaction_lock_artifacts() {
    let path_contract = expect_contract("file-path", "shared file transaction lock");
    let defaults = expect_contract(
        "default-value",
        "file transaction lock parameters and on-disk artifacts",
    );

    // Both records agree on the three timings.
    for value in [&path_contract.exact_value, &defaults.exact_value] {
        assert!(value.contains("2000"), "timeoutMs: {value}");
        assert!(
            value.contains("30_000") || value.contains("30000"),
            "staleMs"
        );
    }
    assert_eq!(
        LockOptions::default(),
        LockOptions {
            timeout: Duration::from_millis(2000),
            retry: Duration::from_millis(10),
            stale: Duration::from_millis(30_000),
        }
    );

    // The lock is a DIRECTORY, mode 0o700, holding `owner.json` at 0o600.
    assert!(
        defaults.exact_value.contains("is a DIRECTORY (mode 0o700)"),
        "{}",
        defaults.exact_value
    );
    assert!(defaults.exact_value.contains("owner.json"));
    assert_eq!(LOCK_DIR_SUFFIX, ".lock");
    assert_eq!(LOCK_OWNER_FILE_NAME, "owner.json");
    assert_eq!(LOCK_DIR_MODE, 0o700);
    assert_eq!(FILE_MODE, 0o600);

    let dir = tempfile::TempDir::new().expect("tempdir");
    let target = dir.path().join("frontend-notes.json");
    let guard = acquire(&target, LockOptions::default()).expect("an unheld lock");

    assert_eq!(guard.lock_path(), lock_path(&target));
    assert_eq!(
        guard.lock_path(),
        dir.path().join("frontend-notes.json.lock")
    );
    assert!(
        guard.lock_path().is_dir(),
        "the lock is a directory — the mkdir IS the test-and-set"
    );

    let owner_path = guard.lock_path().join(LOCK_OWNER_FILE_NAME);
    let raw = fs::read_to_string(&owner_path).expect("read owner.json");
    assert!(
        raw.ends_with('\n'),
        "upstream writes `${{JSON.stringify(...)}}\\n`"
    );
    let owner: Value = serde_json::from_str(raw.trim_end()).expect("valid JSON");
    assert_eq!(
        owner
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["token", "pid", "createdAt"],
        "key order is upstream's object literal order"
    );
    let parsed: LockOwner = serde_json::from_str(raw.trim_end()).expect("a LockOwner");
    assert_eq!(parsed.token, guard.token());
    assert_eq!(parsed.pid, std::process::id());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(guard.lock_path())
                .expect("metadata")
                .permissions()
                .mode()
                & 0o777,
            LOCK_DIR_MODE
        );
        assert_eq!(
            fs::metadata(&owner_path)
                .expect("metadata")
                .permissions()
                .mode()
                & 0o777,
            FILE_MODE
        );
    }

    // The two rename-aside artifacts. Reclaim uses the bare token; release
    // prefixes it, so a directory listing tells the two apart.
    assert!(
        defaults
            .exact_value
            .contains("`<lockPath>.stale.<token>` (release uses `released.<token>`)")
    );
    assert_eq!(LOCK_STALE_INFIX, ".stale.");
    assert_eq!(LOCK_RELEASED_PREFIX, "released.");
    let lock = lock_path(&target);
    assert_eq!(
        lock_stale_path(&lock, "tok"),
        dir.path().join("frontend-notes.json.lock.stale.tok")
    );
    assert_eq!(
        lock_stale_path(&lock, &format!("{LOCK_RELEASED_PREFIX}tok")),
        dir.path()
            .join("frontend-notes.json.lock.stale.released.tok")
    );

    // A second acquirer while the first still holds it: refused, with the
    // catalogued message and code.
    let busy = acquire(
        &target,
        LockOptions {
            timeout: Duration::ZERO,
            ..LockOptions::default()
        },
    )
    .expect_err("a held lock excludes a second acquirer");
    assert_eq!(busy.code(), Some(SHARED_FILE_BUSY));

    assert!(guard.release(), "the lock is still ours");
    assert!(!lock.exists(), "release removes the directory");
    // …and leaves nothing renamed aside behind it: the rename is what frees
    // the slot, the removal is what keeps the directory clean.
    let leftovers: Vec<String> = fs::read_dir(dir.path())
        .expect("read the directory")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.contains(LOCK_STALE_INFIX))
        .collect();
    assert_eq!(
        leftovers,
        Vec::<String>::new(),
        "stale artifacts left behind"
    );

    // The Windows replace ladder's on-disk artifact and its retry policy.
    assert!(
        defaults
            .exact_value
            .contains("`<target>.replace.<uuid>.bak` with 20 retries at 10 ms")
    );
    assert_eq!(REPLACE_BACKUP_PREFIX, ".replace.");
    assert_eq!(REPLACE_BACKUP_SUFFIX, ".bak");
    assert_eq!(
        ReplaceOptions::default().retries,
        20,
        "20 retries on a sharing violation"
    );
    assert_eq!(ReplaceOptions::default().retry, Duration::from_millis(10));
}

#[test]
fn file_transaction_lock_timeout_code() {
    let contract = expect_contract("error-code", "file transaction lock timeout code");
    let value = &contract.exact_value;

    // `shared_file_busy (message: `timed out waiting for shared file lock: <filePath>`)`
    assert!(value.starts_with(SHARED_FILE_BUSY), "{value}");
    let message = backticked(value);
    assert_eq!(
        message,
        ["timed out waiting for shared file lock: <filePath>"],
        "{value}"
    );

    let dir = tempfile::TempDir::new().expect("tempdir");
    let target = dir.path().join("notes.json");
    let held = acquire(&target, LockOptions::default()).expect("an unheld lock");

    let busy = acquire(
        &target,
        LockOptions {
            timeout: Duration::ZERO,
            ..LockOptions::default()
        },
    )
    .expect_err("still held");
    assert_eq!(busy.code(), Some(SHARED_FILE_BUSY));
    assert_eq!(
        busy.to_string(),
        message[0].replace("<filePath>", &target.display().to_string()),
        "the message interpolates the TARGET file, not the lock directory"
    );

    // The other variant carries no code: upstream attaches one only here.
    let io_error = LockError::Io(std::io::Error::other("boom"));
    assert_eq!(io_error.code(), None);

    drop(held);
    // Once released, the same acquisition succeeds — the refusal was the lock,
    // not a broken path.
    let _reacquired = acquire(&target, LockOptions::default()).expect("released");
}
