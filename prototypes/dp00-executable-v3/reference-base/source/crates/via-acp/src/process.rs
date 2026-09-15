//! The one place in this crate that starts a child process.
//!
//! Ported from `server/src/agent/acp-process-client.mjs:147-167` (spawn) and
//! `:571-643` (teardown).
//!
//! # Two invariants, both enforced by this module's shape
//!
//! **`.env_clear()` comes first.** `docs/architecture.md` §17 asks it as a
//! review question, because Rust's [`Command`](tokio::process::Command)
//! inherits the parent environment by default and Node's `spawn` replaces it.
//! [`spawn`] takes a [`BackendEnv`], which only `via-catalog`'s environment
//! policy can produce, and clears before applying it. There is no other spawn
//! site in the crate for a raw map to reach.
//!
//! **The child leads its own process group.** Backends are commonly launched
//! through a package-runner wrapper, so killing the immediate child orphans the
//! real agent: it re-parents to pid 1 and does not reliably exit on stdin EOF.
//! `process-wrap`'s `ProcessGroup::leader()` on unix and `JobObject` on Windows
//! is what makes teardown actually tear down.
//!
//! # Why not the SDK's own spawn
//!
//! `agent_client_protocol::AcpAgent` will spawn a child for you, and it does
//! set a process group. It is not usable here for two independent reasons:
//!
//! 1. it calls `Command::envs(&config.env)` **without** clearing, so every
//!    Gateway secret in the parent environment would reach the agent — the
//!    exact failure the credential boundary exists to prevent;
//! 2. its teardown is a straight `SIGKILL` to the group after a fixed one-second
//!    grace, with no `SIGTERM` first, so an agent that wants to flush state on
//!    shutdown never gets the chance.
//!
//! So VIA spawns, and hands the SDK the byte streams. That also sidesteps the
//! SDK's `async_process`/`async_io` dependency, which would otherwise start a
//! smol reactor beside tokio for the lifetime of the connection.

use std::process::Stdio;
use std::time::Instant;

use process_wrap::tokio::{ChildWrapper, CommandWrap};
use tokio::process::{ChildStderr, ChildStdin, ChildStdout};

use crate::env::BackendEnv;
use crate::error::{AcpError, Result, SpawnErrno};
use crate::limits::{PROCESS_TREE_GRACE, PROCESS_TREE_POLL};

/// How a child is to be started.
///
/// Backend-specific behaviour arrives here as **data**: this crate names no
/// backend, so a command, its arguments, its working directory and its
/// projected environment are the whole of what distinguishes one agent from
/// another at the process layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnSpec {
    /// The executable, resolved by the caller.
    pub command: String,
    /// Arguments, in order.
    pub args: Vec<String>,
    /// The working directory, or `None` to inherit the Gateway's.
    pub cwd: Option<std::path::PathBuf>,
    /// The child's **entire** environment.
    pub env: BackendEnv,
}

impl SpawnSpec {
    /// A spec for `command` with no arguments and an empty environment.
    #[must_use]
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            cwd: None,
            env: BackendEnv::default(),
        }
    }

    /// Set the arguments.
    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    /// Set the working directory.
    #[must_use]
    pub fn cwd(mut self, cwd: impl Into<std::path::PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Set the projected environment.
    #[must_use]
    pub fn env(mut self, env: BackendEnv) -> Self {
        self.env = env;
        self
    }
}

/// A spawned agent process, with its stdio taken.
#[derive(Debug)]
pub struct AcpChild {
    child: Box<dyn ChildWrapper>,
    stdin: ChildStdin,
    stdout: ChildStdout,
    stderr: ChildStderr,
}

impl AcpChild {
    /// The child's process id, if the operating system still has one.
    #[must_use]
    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }

    /// Take the three streams, leaving the child handle behind.
    ///
    /// The handle is what tears the process tree down, so it deliberately
    /// outlives the streams: a connection that finishes with stdio still open
    /// must still be able to escalate.
    #[must_use]
    pub fn into_parts(self) -> (ChildStdin, ChildStdout, ChildStderr, AcpChildHandle) {
        (
            self.stdin,
            self.stdout,
            self.stderr,
            AcpChildHandle {
                child: self.child,
                stopped: false,
            },
        )
    }
}

/// The teardown half of a spawned child.
#[derive(Debug)]
pub struct AcpChildHandle {
    child: Box<dyn ChildWrapper>,
    stopped: bool,
}

impl AcpChildHandle {
    /// The child's process id, if the operating system still has one.
    #[must_use]
    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }

    /// Whether the child has already exited, without waiting for it.
    ///
    /// `None` means "still running"; the exit status is discarded because the
    /// only caller cares whether the process is gone.
    pub fn has_exited(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(Some(_)) | Err(_))
    }

    /// Tear the process tree down: `SIGTERM`, wait, then `SIGKILL`.
    ///
    /// **Contract** — `stopProcessTree`, `acp-process-client.mjs:619-643`:
    /// signal the **group**, poll liveness every
    /// [`PROCESS_TREE_POLL`] for up to [`PROCESS_TREE_GRACE`], and escalate only
    /// if it is still alive. The 750 ms grace is chosen to fit inside the
    /// Gateway's 2 s hard shutdown deadline with room for adapter and logger
    /// cleanup after it.
    ///
    /// Idempotent: upstream memoizes the cleanup per child so a process that
    /// exits while a shutdown is already running is not signalled twice, and a
    /// second call here is a no-op for the same reason.
    ///
    /// On Windows the group is a job object, which `process-wrap` tears down
    /// through the same `start_kill`, so the ladder collapses to one step —
    /// there is no Windows signal that means "please exit".
    pub async fn stop(&mut self) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        if self.has_exited() {
            return;
        }

        #[cfg(unix)]
        {
            let sigterm = nix::sys::signal::Signal::SIGTERM as i32;
            if self.child.signal(sigterm).is_err() {
                // The group is already gone, or we may not signal it. Either
                // way there is nothing to escalate to.
                let _ = self.child.start_kill();
                let _ = self.child.wait().await;
                return;
            }

            let deadline = Instant::now() + PROCESS_TREE_GRACE;
            while Instant::now() < deadline {
                if self.has_exited() {
                    return;
                }
                tokio::time::sleep(PROCESS_TREE_POLL.min(PROCESS_TREE_GRACE)).await;
            }
        }

        if self.has_exited() {
            return;
        }
        let _ = self.child.start_kill();
        let _ = self.child.wait().await;
    }

    /// Wait for the child to exit and report how.
    ///
    /// The string is upstream's `signal || code || 'unknown'`
    /// (`acp-process-client.mjs:203`), which is what
    /// [`AcpError::ProcessExited`] interpolates.
    pub async fn wait_for_exit(&mut self) -> String {
        match self.child.wait().await {
            Ok(status) => exit_description(&status),
            Err(_) => UNKNOWN_EXIT.to_owned(),
        }
    }
}

/// What `signal || code || 'unknown'` renders as when neither is available.
///
/// **External contract** — `acp-process-client.mjs:203`.
pub const UNKNOWN_EXIT: &str = "unknown";

/// `signal || code || 'unknown'`.
///
/// A signalled process reports the **signal**, not the code, because that is
/// the actionable half: "killed by SIGKILL" and "exited 137" are the same event
/// and only one of them says why.
#[must_use]
pub fn exit_description(status: &std::process::ExitStatus) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt as _;
        if let Some(signal) = status.signal() {
            return signal_name(signal);
        }
    }
    status
        .code()
        .map_or_else(|| UNKNOWN_EXIT.to_owned(), |code| code.to_string())
}

#[cfg(unix)]
fn signal_name(signal: i32) -> String {
    nix::sys::signal::Signal::try_from(signal)
        .map_or_else(|_| signal.to_string(), |signal| signal.as_str().to_owned())
}

/// Start an ACP agent process.
///
/// The child gets piped stdin, stdout and stderr, leads its own process group,
/// and receives **exactly** the variables in `spec.env` — nothing is inherited.
///
/// # Errors
///
/// [`AcpError::ProcessSpawnFailed`], with the errno classified so a missing
/// executable stays distinguishable from a permission problem. Upstream copies
/// `error.code` and `error.cause` off the Node spawn error for the same reason,
/// and `acp-process-client.test.mjs:59-72` asserts it.
pub fn spawn(label: &str, spec: &SpawnSpec) -> Result<AcpChild> {
    let mut command = tokio::process::Command::new(&spec.command);
    command.args(&spec.args);
    if let Some(cwd) = &spec.cwd {
        command.current_dir(cwd);
    }

    // The credential boundary. Clearing first is the whole point: everything
    // the child sees is decided by `BackendEnv`, which only a catalogued
    // environment policy can build.
    command.env_clear();
    for (name, value) in spec.env.iter() {
        command.env(name, value);
    }

    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut wrapped = CommandWrap::from(command);
    #[cfg(unix)]
    wrapped.wrap(process_wrap::tokio::ProcessGroup::leader());
    #[cfg(windows)]
    wrapped.wrap(process_wrap::tokio::JobObject);

    let mut child = wrapped.spawn().map_err(|error| {
        tracing::error!(
            event = "acp.process_start_failed",
            backend = label,
            command = spec.command,
            error = %error,
        );
        AcpError::ProcessSpawnFailed {
            label: label.to_owned(),
            detail: error.to_string(),
            errno: SpawnErrno::from_io(&error),
        }
    })?;

    let missing = |stream: &'static str| AcpError::ProcessSpawnFailed {
        label: label.to_owned(),
        detail: std::format!("failed to open {stream}"),
        errno: SpawnErrno::Other,
    };
    let stdin = child.stdin().take().ok_or_else(|| missing("stdin"))?;
    let stdout = child.stdout().take().ok_or_else(|| missing("stdout"))?;
    let stderr = child.stderr().take().ok_or_else(|| missing("stderr"))?;

    tracing::info!(
        event = "acp.process_started",
        backend = label,
        pid = child.id(),
        command = spec.command,
    );

    Ok(AcpChild {
        child,
        stdin,
        stdout,
        stderr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_executable_is_classified_as_enoent() {
        // Upstream: 'preserves ENOENT when a local ACP executable cannot be
        // spawned' (acp-process-client.test.mjs:59-72).
        let spec = SpawnSpec::new("__via_acp_missing_agent_binary__");
        let error = spawn("Example", &spec).expect_err("no such executable");
        assert_eq!(error.spawn_errno(), Some(SpawnErrno::NoEnt));
        assert_eq!(
            error.spawn_errno().and_then(SpawnErrno::as_str),
            Some("ENOENT"),
            "the POSIX name is what drives the `CLI not installed` UX"
        );
        assert_eq!(error.code(), "VIA_ACP_PROCESS_SPAWN_FAILED");
        assert_eq!(error.status(), 0);
    }

    #[test]
    #[cfg(unix)]
    fn an_exit_description_prefers_the_signal_over_the_code() {
        use std::os::unix::process::ExitStatusExt as _;

        // 137 == 128 + 9, the shell's rendering of "killed by SIGKILL". The raw
        // wait status for a signal death is the signal number in the low byte.
        let signalled = std::process::ExitStatus::from_raw(9);
        assert_eq!(exit_description(&signalled), "SIGKILL");

        // A raw status of `code << 8` is a normal exit.
        let normal = std::process::ExitStatus::from_raw(3 << 8);
        assert_eq!(exit_description(&normal), "3");
    }

    #[test]
    fn a_spec_carries_the_whole_child_environment_and_nothing_more() {
        let spec = SpawnSpec::new("agent").args(["--acp"]).cwd("/workspace");
        assert_eq!(spec.args, ["--acp"]);
        assert_eq!(
            spec.cwd.as_deref(),
            Some(std::path::Path::new("/workspace"))
        );
        assert!(
            spec.env.is_empty(),
            "a spec that was never given a projected environment gives the child nothing"
        );
    }
}
