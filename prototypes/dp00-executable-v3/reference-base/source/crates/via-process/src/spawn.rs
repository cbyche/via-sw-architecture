//! Building and executing the spawn spec.
//!
//! Ported from `server/src/process/managed-backend.mjs:114-129`
//! (`spawnSpec`), whose exact shape is catalogued as `state-name/managed
//! backend spawn spec`.
//!
//! # The three things that make this spawn safe
//!
//! **`env_clear()` before anything.** Rust's `Command` inherits the parent
//! environment by default, so the trust boundary
//! [`crate::environment::backend_environment`] computes is only real if
//! nothing else is inherited. `docs/architecture.md` §17 item 9 asks the
//! reviewer this directly.
//!
//! **No shell.** The command and its arguments are passed as a vector, never
//! joined into a line. A backend id and a workspace path both come from
//! configuration, and a shell would make either of them executable.
//!
//! **A process group, always.** See [`crate::supervisor`].

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use via_core::EnvMap;
use via_core::search_path::Platform;

use crate::driver::BackendRuntimeDriver;
use crate::environment::{backend_environment, compose_child_search_path};
use crate::error::ProcessError;
use crate::supervisor::{ChildExit, StopSignal, SupervisedChild};

/// What the child's standard streams are wired to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChildStdio {
    /// The Gateway's own streams — upstream's `stdio: 'inherit'`.
    ///
    /// **External contract** — `state-name/managed backend spawn spec`. A
    /// managed backend service logs to the Gateway's console, which is where an
    /// operator running `via gateway` in the foreground looks for it.
    #[default]
    Inherit,
    /// Pipes, for a caller that captures the child's output.
    ///
    /// VIA-owned. `BackendRuntimeState::failed` appends a captured stderr tail
    /// to its message, and a caller that wants that has to ask for the pipe.
    Piped,
}

/// Everything needed to launch one managed backend.
///
/// **External contract** — `state-name/managed backend spawn spec`:
/// `{command, args, options: {cwd, env, detached, stdio}}`. `detached` has no
/// field here because it is not optional any more — every managed child is a
/// process-group leader, which is strictly stronger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnSpec {
    /// The resolved executable.
    pub command: PathBuf,
    /// Arguments, verbatim.
    pub arguments: Vec<String>,
    /// The working directory — upstream's `cwd: root`.
    pub working_directory: PathBuf,
    /// The projected child environment. Nothing outside this reaches the child.
    pub environment: EnvMap,
    /// Where the child's streams go.
    pub stdio: ChildStdio,
}

/// Finding a launch command on the child's `PATH`.
///
/// A trait because `which` can only search the *host's* filesystem, and a
/// projected Windows child environment has to be testable from a developer's
/// Mac — the same reason [`via_core::search_path::Platform`] is a parameter
/// rather than a `cfg`.
pub trait CommandResolver: Send + Sync + std::fmt::Debug {
    /// Resolve `command` against `search_path`.
    ///
    /// # Errors
    ///
    /// [`ProcessError::CommandNotFound`] when there is no such executable.
    fn resolve(
        &self,
        command: &str,
        search_path: &str,
        working_directory: &Path,
    ) -> Result<PathBuf, ProcessError>;
}

/// The real resolver.
///
/// A command containing a path separator is taken as written — that is what a
/// shell does, and it is what makes an absolute path in configuration work.
/// Anything else is looked up on the supplied `PATH`, with `which` handling
/// `PATHEXT` on Windows.
#[derive(Debug, Clone, Copy, Default)]
pub struct WhichResolver;

impl CommandResolver for WhichResolver {
    fn resolve(
        &self,
        command: &str,
        search_path: &str,
        working_directory: &Path,
    ) -> Result<PathBuf, ProcessError> {
        if command.contains('/') || command.contains('\\') {
            return Ok(PathBuf::from(command));
        }
        which::which_in(command, Some(search_path), working_directory).map_err(|_| {
            ProcessError::CommandNotFound {
                command: command.to_owned(),
            }
        })
    }
}

/// Build the spawn spec for a driver.
///
/// **External contract** — `server/src/process/managed-backend.mjs:114-129`,
/// with the deviation recorded on [`crate::driver::ManagedLaunch`]: upstream
/// spawns `process.execPath` with a `scripts/*.mjs` path, VIA spawns the
/// driver's own command.
///
/// The order of operations matters. The environment is projected *first*, so
/// the `PATH` that the command is resolved against is the child's `PATH` and
/// not the Gateway's — a backend whose policy forwards a version manager's
/// variables must be found on the same `PATH` it will later run with.
///
/// # Errors
///
/// - [`ProcessError::DriverMissingManagedLaunch`] — the driver declares a
///   separate managed process and supplies no launch command.
/// - [`ProcessError::CommandNotFound`] — the command is not on the child's
///   `PATH`.
pub fn spawn_spec(
    driver: &BackendRuntimeDriver,
    root: &Path,
    env: &EnvMap,
    platform: Platform,
    resolver: &dyn CommandResolver,
) -> Result<SpawnSpec, ProcessError> {
    let launch =
        driver
            .launch
            .as_ref()
            .ok_or_else(|| ProcessError::DriverMissingManagedLaunch {
                id: driver.id.clone(),
            })?;
    let mut environment =
        backend_environment(&driver.environment, env, &launch.environment_additions);
    let command = resolver.resolve(
        &launch.command,
        environment.get("PATH").unwrap_or_default(),
        root,
    )?;
    environment.set(
        "PATH",
        compose_child_search_path(
            environment.get("PATH").unwrap_or_default(),
            &command.to_string_lossy(),
            platform,
        ),
    );
    let arguments = launch
        .arguments
        .iter()
        .map(|argument| resolve_argument(argument, &environment))
        .collect();
    Ok(SpawnSpec {
        command,
        arguments,
        working_directory: root.to_path_buf(),
        environment,
        stdio: ChildStdio::Inherit,
    })
}

/// Expand `${NAME}` and `${NAME:-fallback}` in one argument, from the child's
/// own resolved environment.
///
/// # Why argv is resolved here and not when the driver is built
///
/// Upstream spawns `node scripts/<backend>.mjs`, and that shim reads
/// `process.env.<X>_PORT` **at start-up** — after
/// [`apply_backend_address`](crate::apply_backend_address) has published a
/// possibly *reallocated* port. VIA spawns the binary directly, so the port is
/// on the argv instead; baking it when the driver was built meant a
/// reallocation reached the child's environment and never its command line, and
/// the child listened on the port VIA had already given up on.
///
/// Resolving here is the equivalent moment: [`spawn_spec`] is the last thing
/// before the process exists, and it already holds the environment
/// `apply_backend_address` wrote into. The substitution is deliberately
/// generic — this crate may not name a backend, and it does not need to.
///
/// An unmatched `${…}` is left **verbatim** rather than blanked: an argument
/// that silently becomes `--port ` is worse than one that visibly did not
/// expand, because the first looks like a configuration problem and the second
/// looks like the bug it is.
fn resolve_argument(argument: &str, environment: &EnvMap) -> String {
    let mut out = String::with_capacity(argument.len());
    let mut rest = argument;
    while let Some(start) = rest.find("${") {
        let (before, from_marker) = rest.split_at(start);
        out.push_str(before);
        let Some(end) = from_marker.find('}') else {
            // No closing brace: the remainder is literal.
            out.push_str(from_marker);
            return out;
        };
        let body = &from_marker[2..end];
        let (name, fallback) = match body.split_once(":-") {
            Some((name, fallback)) => (name, Some(fallback)),
            None => (body, None),
        };
        match environment.get_trimmed(name) {
            value if !value.is_empty() => out.push_str(value),
            _ => match fallback {
                Some(fallback) => out.push_str(fallback),
                // Unresolvable and no fallback — keep the placeholder.
                None => out.push_str(&from_marker[..=end]),
            },
        }
        rest = &from_marker[end + 1..];
    }
    out.push_str(rest);
    out
}

/// Turning a [`SpawnSpec`] into a running process.
///
/// Injected for the same reason upstream injects `spawnImpl`: the tests that
/// matter most here are the ones asserting that nothing is spawned at all.
#[async_trait]
pub trait BackendSpawner: Send + Sync + std::fmt::Debug {
    /// Launch the child.
    ///
    /// # Errors
    ///
    /// [`ProcessError::Io`] when the platform refuses to spawn.
    async fn spawn(&self, spec: &SpawnSpec) -> Result<Box<dyn SupervisedChild>, ProcessError>;
}

/// The real spawner: `process-wrap` over `tokio::process`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessSpawner;

#[async_trait]
impl BackendSpawner for ProcessSpawner {
    async fn spawn(&self, spec: &SpawnSpec) -> Result<Box<dyn SupervisedChild>, ProcessError> {
        use process_wrap::tokio::CommandWrap;

        let mut command = tokio::process::Command::new(&spec.command);
        command.args(&spec.arguments);
        command.current_dir(&spec.working_directory);
        // The trust boundary. Without this the child inherits every Gateway
        // secret in the parent environment — see `crate::environment`.
        command.env_clear();
        for (name, value) in spec.environment.iter() {
            command.env(name, value);
        }
        match spec.stdio {
            ChildStdio::Inherit => {
                command
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit());
            }
            ChildStdio::Piped => {
                command
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped());
            }
        }

        let mut wrapped = CommandWrap::from(command);
        #[cfg(unix)]
        wrapped.wrap(process_wrap::tokio::ProcessGroup::leader());
        #[cfg(windows)]
        wrapped.wrap(process_wrap::tokio::JobObject);

        let child = wrapped
            .spawn()
            .map_err(|error| ProcessError::io("spawn the managed backend", error))?;
        Ok(Box::new(WrappedChild { child, exit: None }))
    }
}

/// A `process-wrap` child behind [`SupervisedChild`].
struct WrappedChild {
    child: Box<dyn process_wrap::tokio::ChildWrapper>,
    exit: Option<ChildExit>,
}

impl std::fmt::Debug for WrappedChild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WrappedChild")
            .field("id", &self.child.id())
            .field("exit", &self.exit)
            .finish()
    }
}

/// `ExitStatus` to the two fields upstream reads.
fn child_exit(status: &std::process::ExitStatus) -> ChildExit {
    #[cfg(unix)]
    let signal = {
        use std::os::unix::process::ExitStatusExt;
        status.signal()
    };
    #[cfg(not(unix))]
    let signal = None;
    ChildExit {
        code: status.code(),
        signal,
    }
}

#[async_trait]
impl SupervisedChild for WrappedChild {
    fn id(&self) -> Option<u32> {
        self.child.id()
    }

    fn poll_exit(&mut self) -> Result<Option<ChildExit>, std::io::Error> {
        if let Some(exit) = self.exit {
            return Ok(Some(exit));
        }
        let observed = self.child.try_wait()?.map(|status| child_exit(&status));
        self.exit = observed;
        Ok(observed)
    }

    fn take_stdout(&mut self) -> Option<tokio::process::ChildStdout> {
        self.child.stdout().take()
    }

    fn take_stderr(&mut self) -> Option<tokio::process::ChildStderr> {
        self.child.stderr().take()
    }

    fn signal(&mut self, signal: StopSignal) -> Result<(), std::io::Error> {
        #[cfg(unix)]
        {
            // `ProcessGroupChild::signal` sends to the whole group, which is
            // the point — see the module docs on `crate::supervisor`.
            self.child.signal(signal.as_i32())
        }
        #[cfg(not(unix))]
        {
            // Windows has no signals; both rungs of the ladder terminate the
            // Job Object, which is what Node's `child.kill(signal)` does too.
            let _ = signal;
            self.child.start_kill()
        }
    }

    async fn wait(&mut self) -> Result<ChildExit, std::io::Error> {
        if let Some(exit) = self.exit {
            return Ok(exit);
        }
        let status = self.child.wait().await?;
        let exit = child_exit(&status);
        self.exit = Some(exit);
        Ok(exit)
    }
}
