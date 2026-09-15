//! The real two-writer test.
//!
//! The point of a `mkdir` lock rather than an in-process mutex is that the
//! other side may be a **different process** — the Desktop Gateway, the CLI, or
//! a Node Gateway mid-migration. Threads cannot demonstrate that, so this test
//! re-executes the test binary and has both processes run the same
//! read-modify-write against one counter file. Without mutual exclusion the
//! interleaved read/write loses updates and the final count comes up short.

use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use tempfile::TempDir;
use via_store::{LockOptions, with_file_transaction};

/// Set by the parent to point the re-executed binary at the shared directory.
const SHARED_ENV: &str = "VIA_STORE_TWO_WRITER_DIR";
const ROUNDS: u32 = 40;
const RENDEZVOUS_TIMEOUT: Duration = Duration::from_secs(30);

fn options() -> LockOptions {
    LockOptions {
        // Generous: the peer holds the lock across a deliberate sleep, and CI
        // machines are slow. Still bounded, so a genuine deadlock fails the
        // test rather than hanging the suite.
        timeout: Duration::from_secs(60),
        ..LockOptions::default()
    }
}

fn wait_for(marker: &Path) -> bool {
    let deadline = Instant::now() + RENDEZVOUS_TIMEOUT;
    while Instant::now() < deadline {
        if marker.exists() {
            return true;
        }
        thread::sleep(Duration::from_millis(5));
    }
    false
}

/// One process's share of the work: read, pause, write back one more, and note
/// in the log which process did it.
///
/// The pause is what makes the test meaningful — it widens the
/// read-modify-write window far past the point where two unsynchronized
/// processes would collide.
fn bump(shared: &Path, rounds: u32) {
    let counter = shared.join("counter.json");
    let log = shared.join("writers.log");
    let pid = std::process::id();

    for _ in 0..rounds {
        with_file_transaction(Some(counter.as_path()), options(), || {
            let current: u32 = fs::read_to_string(&counter)
                .expect("read counter")
                .trim()
                .parse()
                .expect("parse counter");
            thread::sleep(Duration::from_millis(1));
            fs::write(&counter, (current + 1).to_string()).expect("write counter");

            let mut entries = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log)
                .expect("open log");
            writeln!(entries, "{pid}").expect("write log");
        })
        .expect("transaction");

        // Yield between transactions. The retry loop is a spin, not a queue —
        // upstream's is too — so a writer that re-enters with zero gap starves
        // the other one and the test would prove nothing.
        thread::sleep(Duration::from_millis(3));
    }
}

/// The child half. A no-op unless this process *is* the child, which is how one
/// test binary plays both roles.
#[test]
fn two_writer_child() {
    let Ok(shared) = env::var(SHARED_ENV) else {
        return;
    };
    let shared = PathBuf::from(shared);

    // Rendezvous, so both processes are inside the loop at the same time. A
    // child that only starts after the parent has finished would make this
    // test pass without ever contending.
    fs::write(shared.join("child-ready"), "1").expect("ready");
    assert!(
        wait_for(&shared.join("go")),
        "parent never released the start"
    );

    bump(&shared, ROUNDS);
}

#[test]
fn two_processes_never_lose_an_update() {
    let dir = TempDir::new().expect("tempdir");
    let shared = dir.path();
    fs::write(shared.join("counter.json"), "0").expect("seed");

    let executable = env::current_exe().expect("current_exe");
    let mut child = Command::new(executable)
        .args([
            "two_writer_child",
            "--exact",
            "--nocapture",
            "--test-threads",
            "1",
        ])
        .env(SHARED_ENV, shared)
        .spawn()
        .expect("spawn the second writer");

    assert!(
        wait_for(&shared.join("child-ready")),
        "the second writer never started"
    );
    fs::write(shared.join("go"), "1").expect("go");

    bump(shared, ROUNDS);

    let status = child.wait().expect("wait for the second writer");
    assert!(status.success(), "the second writer failed: {status:?}");

    assert_eq!(
        fs::read_to_string(shared.join("counter.json")).expect("read counter"),
        (ROUNDS * 2).to_string(),
        "an update was lost, so the two processes were not mutually excluded"
    );

    let log = fs::read_to_string(shared.join("writers.log")).expect("read log");
    let writers: Vec<&str> = log.lines().collect();
    assert_eq!(
        writers.len(),
        usize::try_from(ROUNDS * 2).expect("round count"),
    );

    // Both processes really were in the loop together: if they had run one
    // after the other the log would show a single handover.
    let handovers = writers.windows(2).filter(|pair| pair[0] != pair[1]).count();
    assert!(
        handovers >= 2,
        "the two writers never actually contended (handovers: {handovers})"
    );

    assert!(
        !shared.join("counter.json.lock").exists(),
        "both processes released"
    );
}
