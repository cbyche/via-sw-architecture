//! `ManagedBackendRuntime` — the handle that owns a backend child process, and
//! the SIGTERM → wait → SIGKILL ladder that ends it.
//!
//! Ported from `server/src/process/managed-backend.mjs:131-166`.
//!
//! # Signal the group, not the child
//!
//! Backends launch through package-runner wrappers — `npx`, `uvx`, a shell
//! shim — so the process VIA spawns is rarely the process VIA cares about.
//! Signalling only the immediate child leaves the real agent running with its
//! parent gone, holding the port VIA is about to reallocate. Upstream's answer
//! is `detached: true` plus `process.kill(-pid, signal)` on non-Windows and
//! nothing at all on Windows; VIA's is `process-wrap`, which gives the same
//! process group on unix *and* a Job Object on Windows, so grandchild
//! containment is real on both.
//!
//! # What this type deliberately does not do
//!
//! There is no readiness probe and no restart. `docs/reference/contracts.json`
//! (`state-name/managed backend readiness / restart / shutdown`) is explicit:
//! *"There is NO readiness probe and NO restart in managed-backend.mjs […]
//! restart is the embedding host's job via GatewayProcess, not the
//! Gateway's."* Backend exit is fatal to the Gateway process; supervision here
//! ends at teardown.

use std::fmt;
use std::time::Duration;

use async_trait::async_trait;

/// How long a graceful stop is given before the ladder escalates.
///
/// **External contract** — `server/src/process/managed-backend.mjs:153`,
/// catalogued under `default-value/timing constants` as
/// *"ManagedBackendRuntime stop grace 1500ms"*.
pub const STOP_GRACE: Duration = Duration::from_millis(1500);

/// The two rungs of the shutdown ladder.
///
/// **External contract** — `server/src/process/managed-backend.mjs:139,157`.
/// The numbers are POSIX-fixed and identical on every unix VIA targets; on
/// Windows neither exists and the child wrapper terminates the Job Object
/// instead, which is what Node's `child.kill(signal)` does there too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StopSignal {
    /// Ask the process group to exit.
    Term,
    /// Make it.
    Kill,
}

impl StopSignal {
    /// The POSIX signal number.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Term => 15,
            Self::Kill => 9,
        }
    }

    /// The name upstream logs and tests assert.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Term => "SIGTERM",
            Self::Kill => "SIGKILL",
        }
    }
}

/// How a supervised child ended.
///
/// `std::process::ExitStatus` cannot be constructed portably, so the two
/// fields upstream reads — `exitCode` and `signalCode` — are carried directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChildExit {
    /// The exit code, when the process exited normally.
    pub code: Option<i32>,
    /// The signal that ended it, when one did. Unix only.
    pub signal: Option<i32>,
}

/// A child process this crate can supervise.
///
/// A trait rather than a concrete type for the same reason upstream's tests
/// inject `killImpl`: the ladder's contract is *which signals are sent, in
/// which order, to which target*, and that is worth asserting without a real
/// process and a real 1.5 s wait.
#[async_trait]
pub trait SupervisedChild: Send + Sync + fmt::Debug {
    /// The process id, when the process is still known.
    fn id(&self) -> Option<u32>;

    /// The exit status if the process has already ended, without blocking.
    ///
    /// Upstream's `child.exitCode != null || child.signalCode != null` guard.
    ///
    /// # Errors
    ///
    /// Whatever the platform's non-blocking wait reports.
    fn poll_exit(&mut self) -> Result<Option<ChildExit>, std::io::Error>;

    /// Take the child's piped stdout, when one was requested.
    ///
    /// `None` unless the spec asked for [`ChildStdio::Piped`](crate::ChildStdio),
    /// and `None` on every call after the first — the pipe is moved out, not
    /// borrowed. Upstream never pipes a managed backend (its `stdio` is
    /// `'inherit'`), so this has no counterpart; it exists because
    /// [`BackendRuntimeState`](crate::BackendRuntimeState) appends a captured
    /// stderr tail to its failure message, and something has to capture it.
    fn take_stdout(&mut self) -> Option<tokio::process::ChildStdout> {
        None
    }

    /// Take the child's piped stderr, when one was requested.
    ///
    /// See [`Self::take_stdout`].
    fn take_stderr(&mut self) -> Option<tokio::process::ChildStderr> {
        None
    }

    /// Send `signal` to the whole process group.
    ///
    /// # Errors
    ///
    /// Whatever the platform reports. Callers treat a failure as "the group
    /// already exited", which is what upstream's bare `catch` does.
    fn signal(&mut self, signal: StopSignal) -> Result<(), std::io::Error>;

    /// Wait for the process to end.
    ///
    /// # Errors
    ///
    /// Whatever the platform's wait reports.
    async fn wait(&mut self) -> Result<ChildExit, std::io::Error>;
}

/// The handle `start_managed_backend` returns.
///
/// Ported from `server/src/process/managed-backend.mjs:131-166`. Holds a child
/// only when VIA actually spawned one: a frontend-only Gateway, an external
/// service, and a backend the Gateway hosts in-process all produce a runtime
/// with `None` here and [`Self::owns_process`] answering `false`.
#[derive(Debug, Default)]
pub struct ManagedBackendRuntime {
    child: Option<Box<dyn SupervisedChild>>,
}

impl ManagedBackendRuntime {
    /// A runtime that owns nothing.
    #[must_use]
    pub fn detached() -> Self {
        Self { child: None }
    }

    /// A runtime that owns `child`.
    #[must_use]
    pub fn owning(child: Box<dyn SupervisedChild>) -> Self {
        Self { child: Some(child) }
    }

    /// Whether VIA started the process it is holding.
    ///
    /// **External contract** — `server/src/process/managed-backend.mjs:137-139`.
    /// Upstream's tests read exactly this to prove that external ownership and
    /// frontend-only operation spawn nothing.
    #[must_use]
    pub fn owns_process(&self) -> bool {
        self.child.is_some()
    }

    /// The supervised process id, when there is one.
    #[must_use]
    pub fn pid(&self) -> Option<u32> {
        self.child.as_ref().and_then(|child| child.id())
    }

    /// Signal the process group once, without waiting.
    ///
    /// **External contract** — `server/src/process/managed-backend.mjs:141-152`.
    /// A no-op when there is no child or it has already exited. A signal that
    /// fails is swallowed, because the only way it can fail is that the group
    /// exited between the status check and the signal — upstream's comment says
    /// so, and racing that is not an error.
    pub fn close(&mut self, signal: StopSignal) {
        let Some(child) = self.child.as_mut() else {
            return;
        };
        if matches!(child.poll_exit(), Ok(Some(_)) | Err(_)) {
            return;
        }
        let _ = child.signal(signal);
    }

    /// Ask the process group to stop, then make it.
    ///
    /// **External contract** — `server/src/process/managed-backend.mjs:154-166`.
    /// Send `signal`, wait up to `grace`, and on timeout send `SIGKILL`. There
    /// is deliberately no second wait after the escalation: upstream returns as
    /// soon as it has escalated, and adding a wait would change how long
    /// Gateway shutdown can block.
    pub async fn stop(&mut self, signal: StopSignal, grace: Duration) {
        let Some(child) = self.child.as_mut() else {
            return;
        };
        if matches!(child.poll_exit(), Ok(Some(_)) | Err(_)) {
            return;
        }
        let _ = child.signal(signal);
        if tokio::time::timeout(grace, child.wait()).await.is_err() {
            let _ = child.signal(StopSignal::Kill);
        }
    }

    /// [`Self::stop`] with upstream's defaults: `SIGTERM` and
    /// [`STOP_GRACE`].
    pub async fn shutdown(&mut self) {
        self.stop(StopSignal::Term, STOP_GRACE).await;
    }

    /// Wait for the supervised process to end, if there is one.
    ///
    /// Upstream has no such method — `index.mjs` listens for `exit` and calls
    /// `process.exit(1)`. The wait is exposed instead of a callback because
    /// that decision, *"backend exit is fatal"*, belongs to the embedding host
    /// and not to this crate.
    ///
    /// # Errors
    ///
    /// Whatever the platform's wait reports.
    pub async fn wait(&mut self) -> Result<Option<ChildExit>, std::io::Error> {
        match self.child.as_mut() {
            Some(child) => child.wait().await.map(Some),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    /// A child that records the signals it is sent and never exits on its own.
    ///
    /// Upstream's fixture is the same shape: an `EventEmitter` with a `kill`
    /// that records rather than kills
    /// (`server/test/managed-backend.test.mjs:26-36`).
    #[derive(Debug)]
    struct RecordingChild {
        pid: u32,
        signals: Arc<Mutex<Vec<StopSignal>>>,
        exits_on_term: bool,
        exit: Option<ChildExit>,
    }

    impl RecordingChild {
        fn new(signals: Arc<Mutex<Vec<StopSignal>>>, exits_on_term: bool) -> Self {
            Self {
                pid: 4242,
                signals,
                exits_on_term,
                exit: None,
            }
        }
    }

    #[async_trait]
    impl SupervisedChild for RecordingChild {
        fn id(&self) -> Option<u32> {
            Some(self.pid)
        }

        fn poll_exit(&mut self) -> Result<Option<ChildExit>, std::io::Error> {
            Ok(self.exit)
        }

        fn signal(&mut self, signal: StopSignal) -> Result<(), std::io::Error> {
            self.signals.lock().expect("signal log").push(signal);
            if signal == StopSignal::Term && self.exits_on_term {
                self.exit = Some(ChildExit {
                    code: None,
                    signal: Some(signal.as_i32()),
                });
            }
            if signal == StopSignal::Kill {
                self.exit = Some(ChildExit {
                    code: None,
                    signal: Some(signal.as_i32()),
                });
            }
            Ok(())
        }

        async fn wait(&mut self) -> Result<ChildExit, std::io::Error> {
            loop {
                if let Some(exit) = self.exit {
                    return Ok(exit);
                }
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }
    }

    fn runtime(exits_on_term: bool) -> (ManagedBackendRuntime, Arc<Mutex<Vec<StopSignal>>>) {
        let signals = Arc::new(Mutex::new(Vec::new()));
        let child = RecordingChild::new(Arc::clone(&signals), exits_on_term);
        (ManagedBackendRuntime::owning(Box::new(child)), signals)
    }

    #[tokio::test]
    async fn close_signals_once() {
        let (mut runtime, signals) = runtime(false);
        runtime.close(StopSignal::Term);
        assert_eq!(*signals.lock().unwrap(), vec![StopSignal::Term]);
    }

    #[tokio::test]
    async fn a_graceful_child_is_never_killed() {
        let (mut runtime, signals) = runtime(true);
        runtime.stop(StopSignal::Term, Duration::from_secs(5)).await;
        assert_eq!(*signals.lock().unwrap(), vec![StopSignal::Term]);
    }

    #[tokio::test]
    async fn a_child_that_ignores_sigterm_is_killed() {
        let (mut runtime, signals) = runtime(false);
        runtime
            .stop(StopSignal::Term, Duration::from_millis(1))
            .await;
        assert_eq!(
            *signals.lock().unwrap(),
            vec![StopSignal::Term, StopSignal::Kill]
        );
    }

    #[tokio::test]
    async fn an_already_exited_child_is_not_signalled() {
        let (mut runtime, signals) = runtime(true);
        runtime.close(StopSignal::Term);
        runtime.close(StopSignal::Term);
        runtime
            .stop(StopSignal::Term, Duration::from_millis(1))
            .await;
        assert_eq!(*signals.lock().unwrap(), vec![StopSignal::Term]);
    }

    #[tokio::test]
    async fn a_detached_runtime_owns_nothing_and_stops_silently() {
        let mut runtime = ManagedBackendRuntime::detached();
        assert!(!runtime.owns_process());
        assert_eq!(runtime.pid(), None);
        runtime.close(StopSignal::Term);
        runtime.shutdown().await;
        assert_eq!(runtime.wait().await.unwrap(), None);
    }
}
