//! Contract tests for the `mkdir`-based transaction lock.
//!
//! Literals from `shared/file-transaction-lock.mjs` (v1.11.0) and the
//! catalogued *"file transaction lock parameters and on-disk artifacts"* and
//! *"shared file transaction lock"* entries.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tempfile::TempDir;
use via_store::{LockError, LockOptions, LockOwner, acquire, with_file_transaction};

fn lock_path_for(file: &Path) -> PathBuf {
    PathBuf::from(format!("{}.lock", file.display()))
}

fn stale_artifacts(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .expect("read_dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.to_string_lossy().contains(".stale."))
        .collect()
}

#[test]
fn defaults_match_upstream() {
    // shared/file-transaction-lock.mjs:46-49.
    assert_eq!(
        LockOptions::default(),
        LockOptions {
            timeout: Duration::from_millis(2000),
            retry: Duration::from_millis(10),
            stale: Duration::from_millis(30_000),
        }
    );
}

#[test]
fn the_lock_is_a_directory_holding_a_compact_owner_record() {
    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("frontend-notes.json");
    let guard = acquire(&file, LockOptions::default()).expect("acquire");

    let lock = lock_path_for(&file);
    assert_eq!(guard.lock_path(), lock);
    assert!(
        lock.is_dir(),
        "mkdir is the lock primitive, not a lock file"
    );

    let raw = fs::read_to_string(lock.join("owner.json")).expect("owner.json");
    // `${JSON.stringify({ token, pid, createdAt })}\n` — compact, key order
    // token / pid / createdAt, one trailing newline.
    assert!(raw.starts_with(r#"{"token":"#), "got {raw:?}");
    assert!(raw.contains(r#","pid":"#), "got {raw:?}");
    assert!(raw.contains(r#","createdAt":"#), "got {raw:?}");
    assert!(raw.ends_with("}\n"), "got {raw:?}");
    assert_eq!(
        raw.matches('\n').count(),
        1,
        "compact JSON plus exactly one trailing newline, not indented: {raw:?}"
    );

    let owner: LockOwner = serde_json::from_str(raw.trim_end()).expect("parse");
    assert_eq!(owner.token, guard.token());
    assert_eq!(owner.pid, std::process::id());
    assert!(owner.created_at > 0);
}

#[cfg(unix)]
#[test]
fn the_lock_directory_is_0700_and_the_owner_record_0600() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("frontend-notes.json");
    let guard = acquire(&file, LockOptions::default()).expect("acquire");

    let mode = |path: &Path| fs::metadata(path).expect("metadata").permissions().mode() & 0o777;
    assert_eq!(mode(guard.lock_path()), 0o700);
    assert_eq!(mode(&guard.lock_path().join("owner.json")), 0o600);
}

#[test]
fn a_held_lock_times_out_with_shared_file_busy() {
    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("frontend-notes.json");
    let _held = acquire(&file, LockOptions::default()).expect("acquire");

    let error = acquire(
        &file,
        LockOptions {
            timeout: Duration::from_millis(60),
            ..LockOptions::default()
        },
    )
    .expect_err("must not double-acquire");

    // shared/file-transaction-lock.mjs:87-89.
    assert_eq!(
        error.to_string(),
        format!("timed out waiting for shared file lock: {}", file.display())
    );
    assert_eq!(error.code(), Some("shared_file_busy"));
    assert!(matches!(error, LockError::Busy { .. }));
}

#[test]
fn releasing_removes_the_lock_and_leaves_no_artifact() {
    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("frontend-notes.json");

    let guard = acquire(&file, LockOptions::default()).expect("acquire");
    assert!(guard.release(), "the owner may release");

    assert!(!lock_path_for(&file).exists());
    assert!(
        stale_artifacts(dir.path()).is_empty(),
        "release renames to <lockPath>.stale.released.<token> and then removes it"
    );
    // And the slot is immediately reusable.
    drop(acquire(&file, LockOptions::default()).expect("re-acquire"));
}

#[test]
fn dropping_the_guard_releases_it() {
    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("frontend-notes.json");

    {
        let _guard = acquire(&file, LockOptions::default()).expect("acquire");
        assert!(lock_path_for(&file).is_dir());
    }
    assert!(!lock_path_for(&file).exists());
}

#[test]
fn a_lock_that_changed_hands_is_not_released_by_the_previous_owner() {
    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("frontend-notes.json");
    let guard = acquire(&file, LockOptions::default()).expect("acquire");

    // Somebody reclaimed it as stale and took it over.
    let usurper = LockOwner {
        token: "someone-else".into(),
        pid: 4242,
        created_at: 1,
    };
    fs::write(
        guard.lock_path().join("owner.json"),
        format!("{}\n", serde_json::to_string(&usurper).expect("encode")),
    )
    .expect("overwrite");

    let lock = guard.lock_path().to_path_buf();
    assert!(!guard.release(), "a token mismatch must not release");
    assert!(lock.is_dir(), "the new owner keeps its lock");
}

#[test]
fn an_abandoned_lock_is_reclaimed_by_age() {
    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("frontend-notes.json");

    // A crashed process leaves the directory behind — here without even an
    // owner record, which upstream's `malformedLockIsStale` also tolerates.
    fs::create_dir_all(lock_path_for(&file)).expect("orphan");

    let guard = acquire(
        &file,
        LockOptions {
            stale: Duration::ZERO,
            ..LockOptions::default()
        },
    )
    .expect("reclaim");

    // Reclaim is by age, never by probing the recorded pid: pid visibility
    // differs across hosts and containers, and a recycled pid can make two live
    // processes both believe they own the transaction.
    assert!(
        fs::read_to_string(guard.lock_path().join("owner.json"))
            .expect("owner.json")
            .contains(guard.token()),
        "the reclaiming process is now the recorded owner"
    );
    assert!(stale_artifacts(dir.path()).is_empty());
}

#[test]
fn a_legacy_file_shaped_lock_stays_recoverable() {
    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("frontend-notes.json");

    // Before the representation changed to a directory the lock was a file.
    // `readOwner` still reads it, and a stale one still gets reclaimed.
    fs::write(
        lock_path_for(&file),
        "{\"token\":\"old\",\"pid\":1,\"createdAt\":1}\n",
    )
    .expect("legacy lock");

    let guard = acquire(
        &file,
        LockOptions {
            stale: Duration::ZERO,
            ..LockOptions::default()
        },
    )
    .expect("reclaim");
    assert!(guard.lock_path().is_dir());
    assert!(stale_artifacts(dir.path()).is_empty());
}

#[test]
fn a_fresh_lock_is_not_reclaimed_early() {
    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("frontend-notes.json");
    let held = acquire(&file, LockOptions::default()).expect("acquire");
    let token = held.token().to_owned();

    let error = acquire(
        &file,
        LockOptions {
            timeout: Duration::from_millis(40),
            stale: Duration::from_secs(3600),
            ..LockOptions::default()
        },
    )
    .expect_err("must not steal a live lock");
    assert_eq!(error.code(), Some("shared_file_busy"));
    assert!(
        fs::read_to_string(held.lock_path().join("owner.json"))
            .expect("owner.json")
            .contains(&token)
    );
}

#[test]
fn a_transaction_returns_the_actions_value_and_releases() {
    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("frontend-notes.json");

    let value = with_file_transaction(Some(file.as_path()), LockOptions::default(), || {
        assert!(lock_path_for(&file).is_dir(), "held for the whole action");
        41 + 1
    })
    .expect("transaction");

    assert_eq!(value, 42);
    assert!(!lock_path_for(&file).exists());
}

#[test]
fn a_transaction_without_a_path_runs_unlocked() {
    // `if (!filePath) return action()` — an unconfigured surface still works.
    let ran = with_file_transaction(None, LockOptions::default(), || "ran").expect("transaction");
    assert_eq!(ran, "ran");
}

#[test]
fn a_panicking_action_still_releases_the_lock() {
    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("frontend-notes.json");

    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let outcome = std::panic::catch_unwind(|| {
        with_file_transaction(Some(file.as_path()), LockOptions::default(), || {
            panic!("the guarded section blew up");
        })
    });
    std::panic::set_hook(previous);

    assert!(outcome.is_err());
    // Upstream's `finally`; here it is `Drop`, which also covers `?`.
    assert!(!lock_path_for(&file).exists());
    drop(acquire(&file, LockOptions::default()).expect("re-acquire"));
}

#[test]
fn threads_in_one_process_serialize_through_the_lock() {
    let dir = TempDir::new().expect("tempdir");
    let counter = dir.path().join("counter.json");
    fs::write(&counter, "0").expect("seed");

    std::thread::scope(|scope| {
        for _ in 0..6 {
            let counter = counter.clone();
            scope.spawn(move || {
                for _ in 0..15 {
                    with_file_transaction(
                        Some(counter.as_path()),
                        LockOptions {
                            timeout: Duration::from_secs(30),
                            ..LockOptions::default()
                        },
                        || {
                            let current: u32 = fs::read_to_string(&counter)
                                .expect("read")
                                .trim()
                                .parse()
                                .expect("parse");
                            fs::write(&counter, (current + 1).to_string()).expect("write");
                        },
                    )
                    .expect("transaction");
                }
            });
        }
    });

    assert_eq!(fs::read_to_string(&counter).expect("read"), "90");
}
