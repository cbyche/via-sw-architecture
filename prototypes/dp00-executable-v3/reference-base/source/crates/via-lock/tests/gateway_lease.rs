//! The Gateway lease, end to end.
//!
//! The four `upstream:` tests are ports of
//! `test/gateway-instance-lock.test.mjs` (qwen-audio-agent v1.11.0); the rest
//! pin values and behaviours that upstream's suite leaves implicit but that
//! other processes depend on.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{Value, json};
use tempfile::TempDir;

use via_lock::{
    AcquireOptions, Clock, DEFAULT_LEASE_OWNER, FindOptions, GATEWAY_HEARTBEAT_INTERVAL,
    GATEWAY_LOCK_FILE_NAME, GATEWAY_LOCK_SCHEMA, GatewayLease, GatewayLeaseHandle, LEASE_FILE_MODE,
    LEASE_STATE_READY, LEASE_STATE_STARTING, LeaseError, LeaseUpdate, MAX_ACQUIRE_ATTEMPTS,
    ProcessProbe, SignalOutcome, VIA_GATEWAY_ALREADY_RUNNING, acquire_gateway_lease,
    find_running_gateway, gateway_lock_path, lease_stale_path, lease_temp_path, read_gateway_lease,
};

// ── test doubles ────────────────────────────────────────────────────────────

/// A clock frozen at one instant, counting the calls acquisition makes.
#[derive(Debug)]
struct SteppingClock {
    start: DateTime<Utc>,
    step: chrono::TimeDelta,
    calls: AtomicUsize,
}

impl SteppingClock {
    fn new() -> Self {
        Self {
            start: Utc
                .with_ymd_and_hms(2026, 8, 22, 10, 36, 0)
                .single()
                .expect("a real, unambiguous UTC instant"),
            step: chrono::TimeDelta::try_seconds(1).expect("1s is in range"),
            calls: AtomicUsize::new(0),
        }
    }
}

impl Clock for SteppingClock {
    fn now(&self) -> DateTime<Utc> {
        let nth = self.calls.fetch_add(1, Ordering::SeqCst);
        self.start + self.step * i32::try_from(nth).expect("test clock call count fits in i32")
    }
}

/// A clock that recreates the lease file on every tick — a stand-in for a
/// process that keeps winning the create race and then dying before it can be
/// found alive. Acquisition ticks the clock exactly once per attempt (upstream
/// stamps the timestamp at the top of the loop body), so the shared counter
/// *is* the attempt count.
#[derive(Debug)]
struct RaceLosingClock {
    path: PathBuf,
    calls: Arc<AtomicUsize>,
}

impl Clock for RaceLosingClock {
    fn now(&self) -> DateTime<Utc> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        write_foreign_lease(&self.path, "squatter", 99);
        Utc.with_ymd_and_hms(2026, 8, 22, 10, 36, 0)
            .single()
            .expect("a real, unambiguous UTC instant")
    }
}

/// Answers a fixed outcome for a fixed set of pids, `ESRCH` for the rest, and
/// counts how often it was consulted.
#[derive(Debug)]
struct ScriptedProbe {
    running: Vec<i64>,
    outcome: SignalOutcome,
    calls: AtomicUsize,
}

impl ScriptedProbe {
    fn new(running: impl IntoIterator<Item = i64>, outcome: SignalOutcome) -> Self {
        Self {
            running: running.into_iter().collect(),
            outcome,
            calls: AtomicUsize::new(0),
        }
    }

    fn nothing_running() -> Self {
        Self::new([], SignalOutcome::NoSuchProcess)
    }
}

impl ProcessProbe for ScriptedProbe {
    fn signal_zero(&self, pid: i64) -> SignalOutcome {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.running.contains(&pid) {
            self.outcome
        } else {
            SignalOutcome::NoSuchProcess
        }
    }
}

/// A probe that fails the test if anything asks it a question.
#[derive(Debug)]
struct NeverProbed;

impl ProcessProbe for NeverProbed {
    fn signal_zero(&self, pid: i64) -> SignalOutcome {
        panic!("liveness must not be probed for pid {pid}");
    }
}

// ── helpers ─────────────────────────────────────────────────────────────────

fn options(pid: i64, instance_id: &str, probe: impl ProcessProbe + 'static) -> AcquireOptions {
    AcquireOptions::new()
        .pid(pid)
        .instance_id(instance_id)
        .clock(Box::new(SteppingClock::new()))
        .probe(Box::new(probe))
}

/// Write a lease this process does not own, the way upstream's tests do: a
/// partial document with only `schema`, `instanceId` and `pid`.
fn write_foreign_lease(path: &Path, instance_id: &str, pid: i64) {
    fs::write(
        path,
        json!({
            "schema": GATEWAY_LOCK_SCHEMA,
            "instanceId": instance_id,
            "pid": pid,
        })
        .to_string(),
    )
    .expect("the temp directory is writable");
}

fn read_raw(path: &Path) -> String {
    fs::read_to_string(path).expect("the lease file exists")
}

fn acquired(directory: &Path, options: AcquireOptions) -> GatewayLeaseHandle {
    acquire_gateway_lease(directory, options).expect("the lease is free")
}

// ── contract values ─────────────────────────────────────────────────────────

#[test]
fn contract_constants_match_the_rebranded_upstream_values() {
    assert_eq!(GATEWAY_LOCK_SCHEMA, "via.gateway-lock/v1");
    assert_eq!(GATEWAY_LOCK_FILE_NAME, "gateway.lock");
    assert_eq!(VIA_GATEWAY_ALREADY_RUNNING, "VIA_GATEWAY_ALREADY_RUNNING");
    assert_eq!(DEFAULT_LEASE_OWNER, "gateway");
    assert_eq!(LEASE_STATE_STARTING, "starting");
    assert_eq!(LEASE_STATE_READY, "ready");
    assert_eq!(LEASE_FILE_MODE, 0o600);
    assert_eq!(MAX_ACQUIRE_ATTEMPTS, 4);
    assert_eq!(GATEWAY_HEARTBEAT_INTERVAL, Duration::from_millis(15_000));

    let directory = Path::new("/home/example/.config/via");
    assert_eq!(
        gateway_lock_path(directory),
        Path::new("/home/example/.config/via/gateway.lock")
    );
}

#[test]
fn the_lease_is_one_json_line_with_the_contract_field_order() {
    let directory = TempDir::new().expect("a temp directory");
    let mut lease = acquired(
        directory.path(),
        AcquireOptions::new()
            .pid(303)
            .instance_id("current")
            .owner("desktop")
            .clock(Box::new(SteppingClock::new()))
            .probe(Box::new(ScriptedProbe::nothing_running())),
    );
    let path = gateway_lock_path(directory.path());

    assert_eq!(
        read_raw(&path),
        concat!(
            r#"{"schema":"via.gateway-lock/v1","instanceId":"current","pid":303,"#,
            r#""owner":"desktop","state":"starting","origin":"","#,
            r#""startedAt":"2026-08-22T10:36:00.000Z","#,
            r#""heartbeatAt":"2026-08-22T10:36:00.000Z"}"#,
            "\n",
        ),
    );

    assert!(
        lease
            .update(
                LeaseUpdate::new()
                    .state(LEASE_STATE_READY)
                    .origin("http://127.0.0.1:3101"),
            )
            .expect("the lease is still ours"),
    );

    // `startedAt` is stamped once; only `heartbeatAt` advances.
    assert_eq!(
        read_raw(&path),
        concat!(
            r#"{"schema":"via.gateway-lock/v1","instanceId":"current","pid":303,"#,
            r#""owner":"desktop","state":"ready","origin":"http://127.0.0.1:3101","#,
            r#""startedAt":"2026-08-22T10:36:00.000Z","#,
            r#""heartbeatAt":"2026-08-22T10:36:01.000Z"}"#,
            "\n",
        ),
    );
}

#[cfg(unix)]
#[test]
fn the_lease_is_owner_only_and_so_is_the_directory_it_creates() {
    use std::os::unix::fs::PermissionsExt;

    let parent = TempDir::new().expect("a temp directory");
    let directory = parent.path().join("nested").join("via");
    let handle = acquired(
        &directory,
        options(11, "modes", ScriptedProbe::nothing_running()),
    );

    let file_mode = fs::metadata(handle.path())
        .expect("the lease exists")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(file_mode, 0o600, "the lease names a pid and an origin");

    let dir_mode = fs::metadata(&directory)
        .expect("the directory was created")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(dir_mode, 0o700);
}

// ── acquisition ─────────────────────────────────────────────────────────────

#[test]
fn upstream_allows_only_one_live_gateway_per_configuration_directory() {
    let directory = TempDir::new().expect("a temp directory");
    let first = acquired(
        directory.path(),
        options(
            101,
            "first",
            ScriptedProbe::new([101], SignalOutcome::Delivered),
        ),
    );

    let conflict = acquire_gateway_lease(
        directory.path(),
        options(
            202,
            "second",
            ScriptedProbe::new([101], SignalOutcome::Delivered),
        ),
    )
    .expect_err("the first Gateway is still alive");

    assert_eq!(conflict.code(), Some(VIA_GATEWAY_ALREADY_RUNNING));
    assert_eq!(
        conflict.lease().map(|lease| lease.instance_id.as_str()),
        Some("first"),
    );
    assert!(matches!(conflict, LeaseError::AlreadyRunning { .. }));

    assert!(first.release().expect("removable"));
}

/// The bug this crate exists to not have. `kill(pid, 0)` answering `EPERM`
/// means the process is running under a uid we cannot signal — a `root`-owned
/// Gateway, or one started by another user. Reading that as "dead" hands the
/// lease to a second Gateway while the first is still serving.
#[rstest]
#[case::ok_is_alive(SignalOutcome::Delivered)]
#[case::eperm_is_alive(SignalOutcome::PermissionDenied)]
fn a_live_incumbent_is_a_conflict_however_the_kernel_says_so(#[case] outcome: SignalOutcome) {
    let directory = TempDir::new().expect("a temp directory");
    write_foreign_lease(&gateway_lock_path(directory.path()), "incumbent", 101);

    let conflict = acquire_gateway_lease(
        directory.path(),
        options(202, "challenger", ScriptedProbe::new([101], outcome)),
    )
    .expect_err("the incumbent process exists");

    assert_eq!(conflict.code(), Some(VIA_GATEWAY_ALREADY_RUNNING));
    assert_eq!(
        read_gateway_lease(directory.path())
            .expect("the incumbent lease survived")
            .instance_id,
        "incumbent",
        "a conflicting acquisition must not touch the file",
    );
}

#[test]
fn upstream_recovers_a_stale_gateway_lease_atomically() {
    let directory = TempDir::new().expect("a temp directory");
    let path = gateway_lock_path(directory.path());
    write_foreign_lease(&path, "stale", 99);

    let handle = acquire_gateway_lease(
        directory.path(),
        options(505, "fresh", ScriptedProbe::nothing_running()),
    )
    .expect("pid 99 answers ESRCH");

    assert_eq!(
        read_gateway_lease(directory.path())
            .expect("the fresh lease")
            .instance_id,
        "fresh",
    );
    assert!(
        !lease_stale_path(&path, "fresh").exists(),
        "the reclaimed lease is renamed aside and then deleted",
    );
    assert!(handle.release().expect("removable"));
}

#[test]
fn a_lease_naming_a_non_positive_pid_is_reclaimed_without_probing() {
    let directory = TempDir::new().expect("a temp directory");
    // pid 0 addresses the caller's whole process group in kill(2); it must
    // never reach the probe, and it can never name a live Gateway.
    write_foreign_lease(&gateway_lock_path(directory.path()), "corrupt", 0);

    let handle = acquired(directory.path(), options(700, "fresh", NeverProbed));
    assert_eq!(handle.instance_id(), "fresh");
}

#[test]
fn an_unparseable_lease_is_reclaimed_rather_than_obeyed() {
    let directory = TempDir::new().expect("a temp directory");
    let path = gateway_lock_path(directory.path());
    fs::write(&path, "{not json at all").expect("writable");

    let handle = acquired(
        directory.path(),
        options(700, "fresh", ScriptedProbe::nothing_running()),
    );
    assert_eq!(handle.instance_id(), "fresh");
    assert_eq!(
        read_gateway_lease(directory.path())
            .expect("a readable lease now")
            .pid,
        700,
    );
}

#[test]
fn a_lease_with_a_foreign_schema_is_not_a_lease() {
    let directory = TempDir::new().expect("a temp directory");
    fs::write(
        gateway_lock_path(directory.path()),
        json!({ "schema": "other.gateway-lock/v9", "instanceId": "alien", "pid": 1 }).to_string(),
    )
    .expect("writable");

    assert!(read_gateway_lease(directory.path()).is_none());
    // …so it is reclaimed, and pid 1 — which is alive — is never consulted.
    let handle = acquired(directory.path(), options(700, "fresh", NeverProbed));
    assert_eq!(handle.instance_id(), "fresh");
}

#[test]
fn acquisition_gives_up_after_exactly_four_attempts() {
    let directory = TempDir::new().expect("a temp directory");
    let attempts = Arc::new(AtomicUsize::new(0));

    let error = acquire_gateway_lease(
        directory.path(),
        AcquireOptions::new()
            .pid(808)
            .instance_id("loser")
            .clock(Box::new(RaceLosingClock {
                path: gateway_lock_path(directory.path()),
                calls: Arc::clone(&attempts),
            }))
            .probe(Box::new(ScriptedProbe::nothing_running())),
    )
    .expect_err("every attempt loses the create race");

    assert!(matches!(error, LeaseError::Exhausted));
    assert_eq!(error.code(), None);
    assert_eq!(error.lease(), None);
    assert_eq!(error.to_string(), "无法获取 Gateway 实例租约");
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        MAX_ACQUIRE_ATTEMPTS,
        "four attempts, no more — an unbounded loop would spin here forever",
    );
    assert!(
        read_gateway_lease(directory.path()).is_none(),
        "the last attempt reclaimed the squatter's lease before giving up",
    );
}

#[test]
fn reclaiming_a_stale_lease_costs_exactly_one_extra_attempt() {
    let directory = TempDir::new().expect("a temp directory");
    write_foreign_lease(&gateway_lock_path(directory.path()), "stale", 99);

    // The stepping clock advances one second per tick, so the timestamps show
    // how many attempts ran: the second attempt is the one that succeeds.
    let handle = acquired(
        directory.path(),
        options(505, "fresh", ScriptedProbe::nothing_running()),
    );
    assert_eq!(handle.lease().started_at, "2026-08-22T10:36:01.000Z");
}

// ── holding and giving up ───────────────────────────────────────────────────

#[test]
fn upstream_publishes_readiness_and_releases_only_its_own_gateway_lease() {
    let directory = TempDir::new().expect("a temp directory");
    let mut handle = acquired(
        directory.path(),
        AcquireOptions::new()
            .pid(303)
            .instance_id("current")
            .owner("desktop")
            .clock(Box::new(SteppingClock::new()))
            .probe(Box::new(ScriptedProbe::new(
                [303],
                SignalOutcome::Delivered,
            ))),
    );

    assert!(
        handle
            .update(
                LeaseUpdate::new()
                    .state(LEASE_STATE_READY)
                    .origin("http://127.0.0.1:3101"),
            )
            .expect("still ours"),
    );

    let published = read_gateway_lease(directory.path()).expect("a readable lease");
    assert_eq!(
        published,
        GatewayLease {
            schema: GATEWAY_LOCK_SCHEMA.to_owned(),
            instance_id: "current".to_owned(),
            pid: 303,
            owner: "desktop".to_owned(),
            state: "ready".to_owned(),
            origin: "http://127.0.0.1:3101".to_owned(),
            started_at: published.started_at.clone(),
            heartbeat_at: published.heartbeat_at.clone(),
        },
    );

    // Another instance takes the file over.
    write_foreign_lease(&gateway_lock_path(directory.path()), "replacement", 404);

    assert!(
        !handle.release().expect("no i/o failure"),
        "release must refuse a lease it no longer owns",
    );
    assert_eq!(
        read_gateway_lease(directory.path())
            .expect("the replacement survived")
            .instance_id,
        "replacement",
    );
}

#[test]
fn update_refuses_once_the_lease_is_gone_or_taken() {
    let directory = TempDir::new().expect("a temp directory");
    let mut handle = acquired(
        directory.path(),
        options(303, "current", ScriptedProbe::nothing_running()),
    );

    write_foreign_lease(&gateway_lock_path(directory.path()), "replacement", 404);
    assert!(
        !handle
            .update(LeaseUpdate::new().state("ready"))
            .expect("ok")
    );
    assert_eq!(
        read_gateway_lease(directory.path())
            .expect("still the replacement")
            .instance_id,
        "replacement",
        "a refused update must not rewrite the file",
    );

    fs::remove_file(gateway_lock_path(directory.path())).expect("removable");
    assert!(!handle.heartbeat().expect("ok"));
    assert!(!handle.release().expect("ok"));
}

#[test]
fn a_heartbeat_restamps_only_heartbeat_at_and_leaves_no_temp_file() {
    let directory = TempDir::new().expect("a temp directory");
    let mut handle = acquired(
        directory.path(),
        options(303, "current", ScriptedProbe::nothing_running()),
    );
    let started_at = handle.lease().started_at.clone();

    assert!(handle.heartbeat().expect("still ours"));

    let beaten = read_gateway_lease(directory.path()).expect("readable");
    assert_eq!(beaten.started_at, started_at);
    assert_eq!(beaten.heartbeat_at, "2026-08-22T10:36:01.000Z");
    assert_eq!(beaten.state, LEASE_STATE_STARTING);
    assert!(
        !lease_temp_path(&gateway_lock_path(directory.path()), "current").exists(),
        "the temp file is consumed by the rename",
    );
}

#[test]
fn a_stranded_temp_file_does_not_wedge_the_next_update() {
    let directory = TempDir::new().expect("a temp directory");
    let mut handle = acquired(
        directory.path(),
        options(303, "current", ScriptedProbe::nothing_running()),
    );
    let temporary = lease_temp_path(&gateway_lock_path(directory.path()), "current");
    fs::write(&temporary, "left over from a crash").expect("writable");

    // The exclusive create fails, the failure path unlinks the leftover, and
    // the update after it succeeds — upstream's self-healing, reproduced.
    assert!(handle.heartbeat().is_err());
    assert!(!temporary.exists());
    assert!(handle.heartbeat().expect("the path is clear now"));
}

#[test]
fn dropping_a_handle_does_not_release_the_lease() {
    let directory = TempDir::new().expect("a temp directory");
    let handle = acquired(
        directory.path(),
        options(
            909,
            "dropped",
            ScriptedProbe::new([909], SignalOutcome::Delivered),
        ),
    );
    drop(handle);

    let survivor = read_gateway_lease(directory.path()).expect("the lease outlives the handle");
    assert_eq!(survivor.instance_id, "dropped");

    // And it still excludes a second Gateway, which is the point: a lease that
    // evaporated on an early return would let two Gateways serve at once.
    let conflict = acquire_gateway_lease(
        directory.path(),
        options(
            910,
            "next",
            ScriptedProbe::new([909], SignalOutcome::Delivered),
        ),
    )
    .expect_err("pid 909 still answers");
    assert_eq!(conflict.code(), Some(VIA_GATEWAY_ALREADY_RUNNING));
}

#[test]
fn the_conflict_message_names_the_incumbents_origin() {
    let directory = TempDir::new().expect("a temp directory");
    let mut incumbent = acquired(
        directory.path(),
        options(
            101,
            "first",
            ScriptedProbe::new([101], SignalOutcome::Delivered),
        ),
    );

    let silent = acquire_gateway_lease(
        directory.path(),
        options(
            202,
            "second",
            ScriptedProbe::new([101], SignalOutcome::Delivered),
        ),
    )
    .expect_err("alive");
    assert_eq!(silent.to_string(), "已有 Gateway 正在运行");

    incumbent
        .update(LeaseUpdate::new().origin("http://127.0.0.1:3101"))
        .expect("still ours");

    let published = acquire_gateway_lease(
        directory.path(),
        options(
            202,
            "second",
            ScriptedProbe::new([101], SignalOutcome::Delivered),
        ),
    )
    .expect_err("alive");
    assert_eq!(
        published.to_string(),
        "已有 Gateway 正在运行：http://127.0.0.1:3101",
    );
}

// ── discovery ───────────────────────────────────────────────────────────────

#[test]
fn upstream_discovers_only_the_gateway_whose_health_identity_matches_its_lease() {
    let directory = TempDir::new().expect("a temp directory");
    let mut handle = acquired(
        directory.path(),
        options(
            606,
            "discoverable",
            ScriptedProbe::new([606], SignalOutcome::Delivered),
        ),
    );
    handle
        .update(
            LeaseUpdate::new()
                .state(LEASE_STATE_READY)
                .origin("http://127.0.0.1:3210"),
        )
        .expect("still ours");

    let stranger =
        |_origin: &str| -> Option<Value> { Some(json!({ "gatewayInstanceId": "different" })) };
    assert!(
        find_running_gateway(
            directory.path(),
            &stranger,
            FindOptions::new().timeout(Duration::ZERO),
        )
        .is_none(),
        "a port a stranger reused is not a running Gateway",
    );

    let ours = |origin: &str| -> Option<Value> {
        Some(json!({ "gatewayInstanceId": "discoverable", "origin": origin }))
    };
    let active = find_running_gateway(
        directory.path(),
        &ours,
        FindOptions::new().timeout(Duration::ZERO),
    )
    .expect("the identities match");

    assert_eq!(active.origin, "http://127.0.0.1:3210");
    assert_eq!(active.lease.instance_id, "discoverable");
    assert_eq!(
        via_lock::health_instance_id(&active.health),
        Some("discoverable"),
    );
    assert!(read_raw(handle.path()).contains("discoverable"));
}

#[test]
fn discovery_gives_up_at_once_when_there_is_no_lease() {
    let directory = TempDir::new().expect("a temp directory");
    let probed = AtomicUsize::new(0);
    let probe = |_origin: &str| -> Option<Value> {
        probed.fetch_add(1, Ordering::SeqCst);
        None
    };

    // A full default timeout, and it must still return immediately: waiting
    // for a lease that does not exist has nothing to wait for.
    assert!(find_running_gateway(directory.path(), &probe, FindOptions::new()).is_none());
    assert_eq!(probed.load(Ordering::SeqCst), 0);
}

#[test]
fn discovery_does_not_probe_a_gateway_that_has_not_published_an_origin() {
    let directory = TempDir::new().expect("a temp directory");
    let _handle = acquired(
        directory.path(),
        options(606, "starting", ScriptedProbe::nothing_running()),
    );

    let probed = AtomicUsize::new(0);
    let probe = |_origin: &str| -> Option<Value> {
        probed.fetch_add(1, Ordering::SeqCst);
        None
    };
    assert!(
        find_running_gateway(
            directory.path(),
            &probe,
            FindOptions::new().timeout(Duration::ZERO),
        )
        .is_none(),
    );
    assert_eq!(
        probed.load(Ordering::SeqCst),
        0,
        "state 'starting' carries an empty origin, so there is nothing to call",
    );
}

#[test]
fn discovery_polls_until_the_starting_gateway_answers() {
    let directory = TempDir::new().expect("a temp directory");
    let mut handle = acquired(
        directory.path(),
        options(606, "slow", ScriptedProbe::nothing_running()),
    );
    handle
        .update(
            LeaseUpdate::new()
                .state(LEASE_STATE_READY)
                .origin("http://127.0.0.1:3210"),
        )
        .expect("still ours");

    // The health endpoint is not up for the first two polls.
    let attempts = AtomicUsize::new(0);
    let probe = |_origin: &str| -> Option<Value> {
        if attempts.fetch_add(1, Ordering::SeqCst) < 2 {
            None
        } else {
            Some(json!({ "gatewayInstanceId": "slow" }))
        }
    };

    let active = find_running_gateway(
        directory.path(),
        &probe,
        FindOptions::new()
            .timeout(Duration::from_secs(5))
            .interval(Duration::from_millis(1)),
    )
    .expect("the Gateway finishes starting inside the window");

    assert_eq!(active.origin, "http://127.0.0.1:3210");
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

#[test]
fn discovery_returns_the_whole_health_document() {
    let directory = TempDir::new().expect("a temp directory");
    let mut handle = acquired(
        directory.path(),
        options(606, "whole", ScriptedProbe::nothing_running()),
    );
    handle
        .update(LeaseUpdate::new().origin("http://127.0.0.1:3210"))
        .expect("still ours");

    let probe = |_origin: &str| -> Option<Value> {
        Some(json!({
            "ok": true,
            "status": "ready",
            "gatewayInstanceId": "whole",
            "capabilities": ["gateway.instance-lease"],
        }))
    };
    let active = find_running_gateway(
        directory.path(),
        &probe,
        FindOptions::new().timeout(Duration::ZERO),
    )
    .expect("identities match");

    assert_eq!(active.health["status"], json!("ready"));
    assert_eq!(
        active.health["capabilities"][0],
        json!("gateway.instance-lease")
    );
}

// ── reading ─────────────────────────────────────────────────────────────────

#[test]
fn a_partial_lease_reads_back_with_defaults() {
    let directory = TempDir::new().expect("a temp directory");
    write_foreign_lease(&gateway_lock_path(directory.path()), "partial", 42);

    let lease = read_gateway_lease(directory.path()).expect("schema and instanceId are enough");
    assert_eq!(lease.instance_id, "partial");
    assert_eq!(lease.pid, 42);
    assert_eq!(lease.owner, "");
    assert_eq!(lease.origin, "");
    assert_eq!(lease.started_at, "");
}

#[test]
fn every_unreadable_lease_is_the_same_absence() {
    let directory = TempDir::new().expect("a temp directory");
    let path = gateway_lock_path(directory.path());

    assert!(read_gateway_lease(directory.path()).is_none(), "no file");

    for body in [
        "",
        "not json",
        "[]",
        "5",
        r#"{"schema":"via.gateway-lock/v1"}"#,
        r#"{"instanceId":"x","pid":1}"#,
        r#"{"schema":"via.gateway-lock/v2","instanceId":"x"}"#,
    ] {
        fs::write(&path, body).expect("writable");
        assert!(
            read_gateway_lease(directory.path()).is_none(),
            "{body:?} must not read as a lease",
        );
    }
}

#[test]
fn unknown_fields_do_not_stop_a_lease_from_reading() {
    let directory = TempDir::new().expect("a temp directory");
    fs::write(
        gateway_lock_path(directory.path()),
        json!({
            "schema": GATEWAY_LOCK_SCHEMA,
            "instanceId": "future",
            "pid": 1,
            "somethingNewer": { "added": "by a later build" },
        })
        .to_string(),
    )
    .expect("writable");

    assert_eq!(
        read_gateway_lease(directory.path())
            .expect("readable")
            .instance_id,
        "future",
    );
}
