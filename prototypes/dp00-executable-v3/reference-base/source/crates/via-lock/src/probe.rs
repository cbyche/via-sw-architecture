//! Liveness and time, both injectable.
//!
//! Ported from upstream `shared/gateway-instance-lock.mjs:19-27`
//! (`processIsAlive`) and its `now` / `killImpl` seams.

use std::fmt;

use chrono::{DateTime, SecondsFormat, Utc};

/// The result of sending signal 0 to a process.
///
/// Signal 0 delivers nothing; it only runs the kernel's existence and
/// permission checks. The three outcomes are not interchangeable, and this
/// enum exists because collapsing them is the bug this crate most needs to
/// avoid:
///
/// | `kill(pid, 0)` | Means | Alive? |
/// | --- | --- | --- |
/// | `Ok` | the process exists and we may signal it | yes |
/// | `EPERM` | **the process exists** but belongs to another user | **yes** |
/// | `ESRCH` | no such process | no |
///
/// Treating the call as a boolean `is_ok()` reads `EPERM` as "dead" and steals
/// a lease from a Gateway that is running perfectly well under another
/// account — a `root`-owned or another user's instance. Upstream gets this
/// right by returning `error?.code === 'EPERM'` from its `catch`
/// (`shared/gateway-instance-lock.mjs:26`), and so does [`Self::is_alive`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalOutcome {
    /// The signal was accepted: the process exists and is signalable by us.
    Delivered,
    /// `EPERM` — the process exists and we are not allowed to signal it.
    PermissionDenied,
    /// `ESRCH` — no process with that id.
    NoSuchProcess,
    /// Any other `errno`. Upstream treats every non-`EPERM` error as dead, and
    /// so does [`Self::is_alive`].
    Failed(i32),
    /// This platform has no liveness probe. See [`SystemProcessProbe`].
    Unsupported,
}

impl SignalOutcome {
    /// Whether this outcome means the process is still running.
    ///
    /// **External contract.** Upstream `processIsAlive`
    /// (`shared/gateway-instance-lock.mjs:19-27`): `Ok` and `EPERM` are alive,
    /// every other error is dead.
    ///
    /// [`Self::Unsupported`] counts as alive. That is the safe direction: on a
    /// platform that cannot answer the question, refusing to start is
    /// recoverable by hand, whereas stealing a live lease corrupts a running
    /// session.
    #[must_use]
    pub fn is_alive(self) -> bool {
        matches!(
            self,
            Self::Delivered | Self::PermissionDenied | Self::Unsupported
        )
    }
}

/// Sends signal 0 to a process id.
///
/// Injectable so the acquisition tests can drive both arms of the EPERM/ESRCH
/// distinction without spawning real processes; upstream injects the same seam
/// as `killImpl`.
pub trait ProcessProbe: fmt::Debug + Send + Sync {
    /// Probe `pid`. Implementations are never called with `pid <= 0` — see
    /// [`process_is_alive`].
    fn signal_zero(&self, pid: i64) -> SignalOutcome;
}

/// Wall-clock time, injectable so timestamps are assertable.
///
/// Upstream injects the same seam as `now`
/// (`shared/gateway-instance-lock.mjs:92`). The *format* is not part of the
/// seam: it is a contract, applied by [`iso8601`].
pub trait Clock: fmt::Debug + Send + Sync {
    /// The current instant.
    fn now(&self) -> DateTime<Utc>;
}

/// [`Clock`] backed by the system clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Format an instant the way the lease document spells timestamps.
///
/// **External contract.** Upstream stamps `now().toISOString()`
/// (`shared/gateway-instance-lock.mjs:102`), which is UTC with exactly three
/// fractional digits and a `Z` suffix: `2026-08-22T10:36:00.000Z`. Anything
/// that reads `startedAt` / `heartbeatAt` — including a Node Gateway running
/// alongside during migration — parses that shape.
#[must_use]
pub fn iso8601(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// This process's id, as the lease spells it.
#[must_use]
pub fn current_pid() -> i64 {
    i64::from(std::process::id())
}

/// Whether the process that wrote a lease is still running.
///
/// **External contract.** Upstream `processIsAlive`
/// (`shared/gateway-instance-lock.mjs:19-27`).
///
/// The `pid <= 0` guard is load-bearing in two ways. It reproduces upstream's
/// `Number.isInteger(pid) && pid > 0` rejection, and it keeps this crate from
/// ever handing a non-positive id to `kill(2)`, where `0` means *every process
/// in the caller's process group* and `-1` means *every process the caller may
/// signal*. A corrupt lease file must not turn a liveness check into a
/// broadcast signal. Ids past `i32::MAX` cannot name a process on any
/// supported platform and read as dead for the same reason upstream's
/// non-integer ids do.
#[must_use]
pub fn process_is_alive(pid: i64, probe: &dyn ProcessProbe) -> bool {
    if pid <= 0 {
        return false;
    }
    probe.signal_zero(pid).is_alive()
}

/// [`ProcessProbe`] backed by the platform.
///
/// # Unix
///
/// `kill(pid, 0)` via `nix`, mapping `EPERM` and `ESRCH` to their own
/// [`SignalOutcome`] variants.
///
/// # Windows — a known, deliberate gap
///
/// Not implemented: every call answers [`SignalOutcome::Unsupported`], which
/// [`SignalOutcome::is_alive`] reads as *alive*, so a Windows build never
/// reclaims a lease it cannot prove is stale. The consequence is real and
/// stated rather than stubbed over: after an unclean shutdown on Windows,
/// `gateway.lock` must be deleted by hand before a Gateway will start.
/// [`Self::proves_liveness`] reports this at runtime, so a host can say so in
/// its own words instead of leaving the user with an unexplained refusal.
///
/// **Why it is not simply implemented.** The correct check is `OpenProcess` +
/// `GetExitCodeProcess`, *plus* a creation-time comparison — a Windows pid
/// alone is ambiguous because the kernel recycles ids, and a probe that omits
/// the creation-time check will eventually call a recycled pid "alive" and
/// refuse to start forever. Every route to those calls is a raw FFI binding:
///
/// * `windows-sys` / `winapi` — `unsafe extern "system"` calls. This
///   workspace sets `#![forbid(unsafe_code)]` on this crate and bans `unsafe`
///   outright, so this is not a dependency decision, it is a rule.
/// * a safe wrapper such as `sysinfo` — a process-table crate pulled in for
///   one boolean, on a platform this port has no machine to test on.
///
/// A liveness probe that is wrong in the *alive* direction wedges a Windows
/// install; wrong in the *dead* direction it steals a running Gateway's lease
/// and corrupts a live session. Shipping an untested implementation of the
/// second failure mode is worse than the documented refusal above, so the
/// refusal stands until someone can test it on Windows.
///
/// **The seam is already there.** [`ProcessProbe`] is injected
/// ([`AcquireOptions::probe`](crate::AcquireOptions::probe),
/// [`CliAcquireOptions::probe`](crate::CliAcquireOptions::probe)), so a
/// Windows host that has a tested implementation supplies it without this
/// crate changing at all.
///
/// Upstream has the check for free because Node implements
/// `process.kill(pid, 0)` on Windows itself.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemProcessProbe;

impl SystemProcessProbe {
    /// Whether this platform's probe can *prove* a process dead.
    ///
    /// `true` on Unix, where `ESRCH` is conclusive. `false` everywhere else,
    /// where every answer is [`SignalOutcome::Unsupported`] and therefore
    /// "alive" — see the type documentation.
    ///
    /// This exists so the refusal is legible: a host that finds
    /// [`LeaseError::AlreadyRunning`](crate::LeaseError::AlreadyRunning) on a
    /// platform where this is `false` can tell the user that the incumbent may
    /// simply be a stale `gateway.lock`, rather than reporting a running
    /// Gateway that is not there.
    #[must_use]
    pub const fn proves_liveness() -> bool {
        cfg!(unix)
    }
}

#[cfg(unix)]
impl ProcessProbe for SystemProcessProbe {
    fn signal_zero(&self, pid: i64) -> SignalOutcome {
        use nix::errno::Errno;
        use nix::unistd::Pid;

        let Ok(raw) = i32::try_from(pid) else {
            return SignalOutcome::NoSuchProcess;
        };
        match nix::sys::signal::kill(Pid::from_raw(raw), None) {
            Ok(()) => SignalOutcome::Delivered,
            Err(Errno::EPERM) => SignalOutcome::PermissionDenied,
            Err(Errno::ESRCH) => SignalOutcome::NoSuchProcess,
            Err(other) => SignalOutcome::Failed(other as i32),
        }
    }
}

#[cfg(not(unix))]
impl ProcessProbe for SystemProcessProbe {
    fn signal_zero(&self, _pid: i64) -> SignalOutcome {
        SignalOutcome::Unsupported
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[derive(Debug)]
    struct NeverCalled;

    impl ProcessProbe for NeverCalled {
        fn signal_zero(&self, pid: i64) -> SignalOutcome {
            panic!("the probe must not be consulted for pid {pid}");
        }
    }

    #[test]
    fn eperm_is_alive_and_esrch_is_dead() {
        assert!(SignalOutcome::Delivered.is_alive());
        assert!(
            SignalOutcome::PermissionDenied.is_alive(),
            "EPERM means the process exists under another uid"
        );
        assert!(!SignalOutcome::NoSuchProcess.is_alive());
        assert!(!SignalOutcome::Failed(22).is_alive());
        assert!(SignalOutcome::Unsupported.is_alive());
    }

    #[test]
    fn non_positive_pids_are_dead_and_never_reach_the_probe() {
        assert!(!process_is_alive(0, &NeverCalled));
        assert!(!process_is_alive(-1, &NeverCalled));
        assert!(!process_is_alive(i64::MIN, &NeverCalled));
    }

    #[test]
    fn iso8601_matches_the_javascript_shape() {
        let at = Utc
            .with_ymd_and_hms(2026, 8, 22, 10, 36, 0)
            .single()
            .expect("2026-08-22T10:36:00Z is a real, unambiguous UTC instant");
        assert_eq!(iso8601(at), "2026-08-22T10:36:00.000Z");
    }

    #[test]
    fn liveness_is_provable_exactly_where_the_probe_is_implemented() {
        assert_eq!(SystemProcessProbe::proves_liveness(), cfg!(unix));
        // Wherever it is not provable, the probe must answer Unsupported, and
        // Unsupported must be the *safe* direction: alive.
        if !SystemProcessProbe::proves_liveness() {
            assert_eq!(
                SystemProcessProbe.signal_zero(1),
                SignalOutcome::Unsupported
            );
            assert!(process_is_alive(1, &SystemProcessProbe));
        }
    }

    #[cfg(unix)]
    #[test]
    fn the_system_probe_sees_this_process() {
        assert_eq!(
            SystemProcessProbe.signal_zero(current_pid()),
            SignalOutcome::Delivered
        );
        assert!(process_is_alive(current_pid(), &SystemProcessProbe));
    }

    #[cfg(unix)]
    #[test]
    fn the_system_probe_reports_esrch_for_an_impossible_pid() {
        // Above every platform's pid_max, so it cannot collide with a real
        // process, and inside i32 so it still reaches kill(2).
        assert_eq!(
            SystemProcessProbe.signal_zero(0x7fff_fff0),
            SignalOutcome::NoSuchProcess
        );
        assert!(!process_is_alive(0x7fff_fff0, &SystemProcessProbe));
        assert!(!process_is_alive(
            i64::from(i32::MAX) + 1,
            &SystemProcessProbe
        ));
    }

    #[cfg(unix)]
    #[test]
    fn pid_1_exists_and_is_not_ours_to_signal() {
        // init/launchd is always running and is owned by root. Under an
        // unprivileged uid this is the EPERM arm against the real kernel; as
        // root it is the Ok arm. Both are alive, which is the whole point.
        assert!(process_is_alive(1, &SystemProcessProbe));
    }
}
