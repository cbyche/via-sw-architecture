//! **The lease, across processes** — and the one probe that must not be
//! backwards.
//!
//! `apps/via` asserts the single-instance refusal twice already: once against a
//! lease this test process holds through `via-lock`'s own API
//! (`tests/setup_gate.rs`) and once against a genuinely serving Gateway
//! (`tests/chat_client.rs`). Neither can reach the case that matters most,
//! because both processes die *cleanly*.
//!
//! # The case that matters
//!
//! A Gateway that is SIGKILLed leaves `gateway.lock` on disk naming a pid that
//! no longer exists. Whether the next start reclaims it or obeys it comes down
//! to one call — `kill(pid, 0)` — and to reading its three outcomes correctly:
//!
//! | | Means | Alive? |
//! | --- | --- | --- |
//! | `Ok` | the process exists and we may signal it | yes |
//! | `EPERM` | **the process exists** under another uid | **yes** |
//! | `ESRCH` | no such process | no |
//!
//! Collapsing that to `is_ok()` reads `EPERM` as *dead* and steals the lease
//! from a Gateway running perfectly well under another account. Inverting it
//! reads `ESRCH` as *alive* and wedges the install forever. Both arms are
//! exercised here against the real kernel:
//!
//! * [`a_lease_left_by_an_unclean_death_is_reclaimed`] — `ESRCH`, from a pid
//!   this test genuinely killed;
//! * [`a_lease_naming_a_process_we_may_not_signal_is_obeyed`] — `EPERM`, from
//!   pid 1, which is always running and is not ours to signal.

mod support;

use pretty_assertions::assert_eq;
use support::{FILE_BUDGET, Machine, wait_until};

/// A key so the setup gate is not the thing under test.
const CONFIGURED: [(&str, &str); 1] = [("DASHSCOPE_API_KEY", "sk-e2e-lease")];

/// Write a lease naming `pid` into `path`.
///
/// Built through `via-lock`'s own type rather than as a hand-written JSON
/// literal, so the document presented to the binary is the one `via-lock`
/// writes — field names, field order and schema string included.
fn write_lease(path: &std::path::Path, pid: i64, origin: &str) {
    let lease = via_lock::GatewayLease {
        schema: via_lock::GATEWAY_LOCK_SCHEMA.to_owned(),
        instance_id: "e2e-0000-0000-0000-000000000000".to_owned(),
        pid,
        owner: "cli".to_owned(),
        state: via_lock::LEASE_STATE_READY.to_owned(),
        origin: origin.to_owned(),
        started_at: "2026-08-22T00:00:00.000Z".to_owned(),
        heartbeat_at: "2026-08-22T00:00:00.000Z".to_owned(),
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("config dir");
    }
    let body = serde_json::to_string(&lease).expect("a lease serializes");
    std::fs::write(path, format!("{body}\n")).expect("write the lease");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_lease_left_by_an_unclean_death_is_reclaimed() {
    let machine = Machine::new();

    // ── 1. an incumbent ─────────────────────────────────────────────────────
    let first = machine.start_via_gateway(&CONFIGURED);
    first.require_serving();
    let incumbent_pid = first.pid();
    let incumbent_origin = first.origin();
    let held = machine.lease().expect("a serving Gateway holds the lease");
    assert_eq!(held.pid, incumbent_pid);
    assert_eq!(held.origin, incumbent_origin);
    assert_eq!(held.state, via_lock::LEASE_STATE_READY);

    // ── 2. a second process is refused, and told where the first one is ─────
    let second = machine.run_via(&CONFIGURED, &["gateway", "--url", support::EPHEMERAL_URL]);
    assert_eq!(second.code, Some(1));
    assert_eq!(
        second.error_code(),
        Some(via_lock::VIA_GATEWAY_ALREADY_RUNNING),
        "records: {:?}",
        second.records,
    );
    assert!(
        second.stderr.contains(&incumbent_origin),
        "the refusal names where the incumbent is listening, so a client can go there \
         instead of guessing: {}",
        second.stderr,
    );
    assert_eq!(
        machine.lease().map(|lease| lease.pid),
        Some(incumbent_pid),
        "a refused start must not have disturbed the incumbent's lease",
    );

    // ── 3. the incumbent dies **uncleanly** ─────────────────────────────────
    // SIGKILL cannot be intercepted, so none of the close sequence runs: no
    // flush, no `lease.release()`. The lease is left behind on purpose.
    let killed = first.kill();
    assert_eq!(
        killed.code, None,
        "a signalled process reports no exit code, which is how we know nothing ran",
    );
    assert!(
        machine.lease_file().exists(),
        "SIGKILL leaves the lease on disk — that is the whole premise",
    );
    assert_eq!(machine.lease().map(|lease| lease.pid), Some(incumbent_pid));

    // The pid must genuinely be gone before the reclaim is meaningful; a
    // reclaim of a lease whose process is still exiting proves nothing.
    let reaped = wait_until(FILE_BUDGET, || {
        !via_lock::process_is_alive(incumbent_pid, &via_lock::SystemProcessProbe)
    })
    .await;
    assert!(reaped, "pid {incumbent_pid} never became ESRCH");

    // ── 4. the next start reclaims it ───────────────────────────────────────
    let third = machine.start_via_gateway(&CONFIGURED);
    third.require_serving();
    let reclaimed = machine.lease().expect("the third Gateway holds the lease");
    assert_eq!(
        reclaimed.pid,
        third.pid(),
        "the stale lease was reclaimed, not obeyed",
    );
    assert_ne!(reclaimed.instance_id, held.instance_id);
    assert_eq!(reclaimed.origin, third.origin());
    assert_ne!(
        reclaimed.origin, incumbent_origin,
        "the published origin is the new listener's, not the dead one's",
    );

    let run = third.stop();
    assert_eq!(run.code, Some(0), "stderr: {}", run.stderr);
    assert!(!machine.lease_file().exists());
}

#[cfg(unix)]
#[test]
fn a_lease_naming_a_process_we_may_not_signal_is_obeyed() {
    // pid 1 is init/launchd: always running, and owned by root. Under an
    // unprivileged uid `kill(1, 0)` answers **EPERM**, which means *the process
    // exists* — and a probe that reads EPERM as "dead" would steal the lease of
    // a Gateway running under another account. Under root it answers `Ok`,
    // which is also alive, so this case asserts the same outcome either way and
    // is not uid-dependent.
    let machine = Machine::new();
    assert!(
        via_lock::process_is_alive(1, &via_lock::SystemProcessProbe),
        "pid 1 must read as alive for this case to be testing anything",
    );

    write_lease(&machine.lease_file(), 1, "http://127.0.0.1:3101");
    let run = machine.run_via(&CONFIGURED, &["gateway", "--url", support::EPHEMERAL_URL]);

    assert_eq!(run.code, Some(1), "stdout: {}", run.stdout);
    assert_eq!(
        run.error_code(),
        Some(via_lock::VIA_GATEWAY_ALREADY_RUNNING),
        "a lease whose process exists must be obeyed, whoever owns that process. \
         records: {:?}",
        run.records,
    );
    assert_eq!(
        machine.lease().map(|lease| lease.pid),
        Some(1),
        "the refused start left the incumbent's lease exactly as it found it",
    );
}

#[cfg(unix)]
#[test]
fn a_lease_naming_a_pid_that_cannot_exist_is_reclaimed() {
    // The other arm, with no process to kill: an id above every platform's
    // `pid_max` and still inside `i32`, so it reaches `kill(2)` and comes back
    // ESRCH. Together with the EPERM case above, this pins both directions of
    // the one call the whole single-instance contract rests on.
    const IMPOSSIBLE: i64 = 0x7fff_fff0;
    assert!(!via_lock::process_is_alive(
        IMPOSSIBLE,
        &via_lock::SystemProcessProbe
    ));

    let machine = Machine::new();
    write_lease(&machine.lease_file(), IMPOSSIBLE, "http://127.0.0.1:3101");
    let gateway = machine.start_via_gateway(&CONFIGURED);
    gateway.require_serving();
    assert_eq!(
        machine.lease().map(|lease| lease.pid),
        Some(gateway.pid()),
        "a lease naming nobody is reclaimed",
    );
    let _ = gateway.stop();
}
