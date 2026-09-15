//! Real processes: the shutdown ladder, process-group containment, and the
//! environment boundary as the child actually observes it.
//!
//! Everything else in this crate's tests uses a recording child, which proves
//! *which signals are sent in which order*. These prove the harder half — that
//! sending them actually ends the process tree, and that the environment the
//! child sees is the one this crate computed and nothing more.
//!
//! Upstream has no equivalent: its supervision tests are all against an
//! `EventEmitter` fixture. The behaviours these cover are exactly the ones
//! `docs/architecture.md` §12 says `process-wrap` was chosen for
//! (*"a real SIGTERM → wait → SIGKILL ladder, which also fixes Windows
//! grandchild containment"*) and §17 item 9 asks about (`.env_clear()`).

#![cfg(unix)]

mod common;

use std::path::PathBuf;
use std::time::Duration;

use common::env;
use via_core::EnvMap;
use via_process::{
    BackendSpawner, ChildStdio, ManagedBackendRuntime, ProcessSpawner, SpawnSpec, StopSignal,
};

fn spec(script: &str, environment: EnvMap, stdio: ChildStdio) -> SpawnSpec {
    SpawnSpec {
        command: PathBuf::from("/bin/sh"),
        arguments: vec!["-c".to_owned(), script.to_owned()],
        working_directory: std::env::temp_dir(),
        environment,
        stdio,
    }
}

#[tokio::test]
async fn a_child_that_ignores_sigterm_is_really_killed() {
    // `trap '' TERM` makes the shell ignore SIGTERM outright, which is the
    // exact case the ladder exists for: an agent wrapper that swallows the
    // polite request.
    let child = ProcessSpawner
        .spawn(&spec(
            "trap '' TERM; while :; do sleep 0.05; done",
            env(&[("PATH", "/usr/bin:/bin")]),
            ChildStdio::Inherit,
        ))
        .await
        .expect("the shell spawns");
    let pid = child.id().expect("a running child has a pid");
    let mut runtime = ManagedBackendRuntime::owning(child);

    runtime
        .stop(StopSignal::Term, Duration::from_millis(200))
        .await;

    // The escalation is fire-and-forget upstream, so give the kernel a moment
    // and then confirm the process is genuinely gone rather than merely
    // signalled.
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline && process_is_alive(pid) {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(!process_is_alive(pid), "pid {pid} survived the ladder");
}

#[tokio::test]
async fn a_graceful_child_never_reaches_sigkill() {
    let child = ProcessSpawner
        .spawn(&spec(
            "while :; do sleep 0.05; done",
            env(&[("PATH", "/usr/bin:/bin")]),
            ChildStdio::Inherit,
        ))
        .await
        .expect("the shell spawns");
    let mut runtime = ManagedBackendRuntime::owning(child);

    let started = std::time::Instant::now();
    runtime.shutdown().await;
    assert!(
        started.elapsed() < via_process::STOP_GRACE,
        "a child that honours SIGTERM must not wait out the whole grace period",
    );
}

#[tokio::test]
async fn killing_the_group_kills_a_grandchild_too() {
    // The reason `process-wrap` is a dependency. The shell here stands in for
    // an `npx` wrapper: signalling only the immediate child would leave the
    // grandchild — the process that actually matters — running.
    let marker =
        std::env::temp_dir().join(format!("via-process-grandchild-{}", std::process::id()));
    let _ = std::fs::remove_file(&marker);
    let script = format!(
        "( while :; do echo alive > '{marker}'; sleep 0.05; done ) & wait",
        marker = marker.display(),
    );
    let child = ProcessSpawner
        .spawn(&spec(
            &script,
            env(&[("PATH", "/usr/bin:/bin")]),
            ChildStdio::Inherit,
        ))
        .await
        .expect("the shell spawns");
    let mut runtime = ManagedBackendRuntime::owning(child);

    // Wait for the grandchild to prove it is running.
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline && !marker.exists() {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(marker.exists(), "the grandchild never started");

    runtime
        .stop(StopSignal::Term, Duration::from_millis(200))
        .await;
    tokio::time::sleep(Duration::from_millis(300)).await;

    // If the grandchild is dead the marker stops being refreshed.
    let before = std::fs::metadata(&marker)
        .and_then(|meta| meta.modified())
        .ok();
    tokio::time::sleep(Duration::from_millis(400)).await;
    let after = std::fs::metadata(&marker)
        .and_then(|meta| meta.modified())
        .ok();
    let _ = std::fs::remove_file(&marker);
    assert_eq!(
        before, after,
        "the grandchild is still writing, so the process group survived",
    );
}

#[tokio::test]
async fn the_child_sees_only_the_environment_this_crate_computed() {
    // The `.env_clear()` assertion. `sh` adds `PWD`, `SHLVL` and `_` of its
    // own, so the test is that nothing *this process* holds leaks, not that
    // the child's environment is byte-identical.
    let mut child = ProcessSpawner
        .spawn(&spec(
            "env",
            env(&[
                ("PATH", "/usr/bin:/bin"),
                ("VIA_ENV_LOADED", "1"),
                ("FIXTURE_TOKEN", "crosses"),
            ]),
            ChildStdio::Piped,
        ))
        .await
        .expect("the shell spawns");

    let mut buffer = Vec::new();
    {
        use tokio::io::AsyncReadExt;
        let mut stdout = child.take_stdout().expect("piped stdout");
        stdout.read_to_end(&mut buffer).await.expect("read stdout");
        assert!(
            child.take_stdout().is_none(),
            "the pipe is moved out, not borrowed",
        );
    }
    let _ = child.wait().await;
    let observed = String::from_utf8_lossy(&buffer);

    assert!(observed.contains("FIXTURE_TOKEN=crosses"), "{observed}");
    assert!(observed.contains("VIA_ENV_LOADED=1"), "{observed}");
    // The marker this test process sets is not on any allow-list and was never
    // put into the spec, so it must not be visible to the child.
    assert!(
        !observed.contains("CARGO_PKG_NAME"),
        "the child inherited the test process's environment: {observed}",
    );
}

/// Whether `pid` still exists. `kill(pid, 0)` is the portable liveness probe.
fn process_is_alive(pid: u32) -> bool {
    // Safe wrapper: `std::process::Command` is the only signal-free way to ask
    // without a libc dependency, and `kill -0` is exactly `kill(pid, 0)`.
    std::process::Command::new("/bin/kill")
        .args(["-0", &pid.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
