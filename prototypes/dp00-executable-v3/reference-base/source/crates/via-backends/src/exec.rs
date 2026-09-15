//! The real subprocess runners the install and auth probes use.
//!
//! Kept apart from the logic they serve so that `install.rs` and `auth.rs` stay
//! pure and testable: upstream injects `spawnImpl` / `run` into both for exactly
//! the same reason, and every test in this crate exercises the branching through
//! a scripted runner rather than through a real process.
//!
//! # Why `process-wrap`
//!
//! `npm install -g` forks: `npm` spawns `node`, which spawns package lifecycle
//! scripts. Killing only the immediate child on a timeout or a cancel leaves the
//! rest running and re-parented. `ProcessGroup::leader()` on unix and
//! `JobObject` on Windows are what upstream approximates with
//! `detached: true` + `process.kill(-pid)` and `taskkill /pid <pid> /t /f`
//! (`shared/backend-install.mjs:263-288`).
//!
//! # Both runners clear the environment first
//!
//! `docs/architecture.md` §17 asks it as a review question, and it applies to an
//! installer and a status probe exactly as it applies to an agent: Rust's
//! `Command` inherits by default, so an installer spawned without
//! `.env_clear()` hands `npm` — and every lifecycle script it runs — the
//! Gateway's auth secret and realtime key.

use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use process_wrap::tokio::CommandWrap;
use tokio::io::{AsyncBufReadExt as _, AsyncReadExt as _, BufReader};

use crate::auth::{ProbeOutcome, ProbeRequest, ProbeRunner};
use crate::install::{
    InstallCancel, OutputStream, StepOutcome, StepRequest, StepRunner, append_output,
};

/// How often a running step is checked for cancellation.
///
/// Cancellation is a cooperative flag rather than a signal, so it is observed on
/// a poll. 50 ms is below human perception and far below the ten-minute step
/// budget.
const CANCEL_POLL: Duration = Duration::from_millis(50);

/// How long a killed child is given to actually go away.
const KILL_GRACE: Duration = Duration::from_millis(2_000);

/// Runs installation steps as real child processes.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessStepRunner;

#[async_trait]
impl StepRunner for ProcessStepRunner {
    async fn run(
        &self,
        request: StepRequest,
        cancel: &InstallCancel,
        on_output: &(dyn for<'a> Fn(OutputStream, &'a str) + Send + Sync),
    ) -> StepOutcome {
        if cancel.is_cancelled() {
            return StepOutcome {
                code: -1,
                aborted: true,
                ..StepOutcome::default()
            };
        }
        let mut command = tokio::process::Command::new(&request.command);
        command.args(&request.arguments);
        command.env_clear();
        for (name, value) in request.environment.iter() {
            command.env(name, value);
        }
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut wrapped = CommandWrap::from(command);
        #[cfg(unix)]
        wrapped.wrap(process_wrap::tokio::ProcessGroup::leader());
        #[cfg(windows)]
        wrapped.wrap(process_wrap::tokio::JobObject);

        let mut child = match wrapped.spawn() {
            Ok(child) => child,
            Err(error) => {
                return StepOutcome {
                    code: -1,
                    error: Some(error.to_string()),
                    ..StepOutcome::default()
                };
            }
        };

        let (sender, mut receiver) =
            tokio::sync::mpsc::unbounded_channel::<(OutputStream, String)>();
        if let Some(stream) = child.stdout().take() {
            spawn_reader(stream, OutputStream::Stdout, sender.clone());
        }
        if let Some(stream) = child.stderr().take() {
            spawn_reader(stream, OutputStream::Stderr, sender.clone());
        }
        drop(sender);

        let deadline = tokio::time::Instant::now() + request.timeout;
        let mut output = String::new();
        let mut timeout = false;
        let mut aborted = false;
        loop {
            tokio::select! {
                biased;
                received = receiver.recv() => match received {
                    Some((stream, line)) => {
                        output = append_output(&output, stream, &line);
                        on_output(stream, &line);
                    }
                    // Both readers are finished, so the child has closed its
                    // streams; only the exit status is left.
                    None => break,
                },
                () = tokio::time::sleep_until(deadline) => {
                    timeout = true;
                    break;
                }
                () = tokio::time::sleep(CANCEL_POLL) => {
                    if cancel.is_cancelled() {
                        aborted = true;
                        break;
                    }
                }
            }
        }

        if timeout || aborted {
            let _ = child.start_kill();
            let _ = tokio::time::timeout(KILL_GRACE, child.wait()).await;
            return StepOutcome {
                code: -1,
                output,
                timeout,
                aborted,
                error: None,
            };
        }

        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        match tokio::time::timeout(remaining, child.wait()).await {
            Ok(Ok(status)) => StepOutcome {
                code: status.code().unwrap_or(-1),
                output,
                ..StepOutcome::default()
            },
            Ok(Err(error)) => StepOutcome {
                code: -1,
                output,
                error: Some(error.to_string()),
                ..StepOutcome::default()
            },
            Err(_) => {
                let _ = child.start_kill();
                StepOutcome {
                    code: -1,
                    output,
                    timeout: true,
                    ..StepOutcome::default()
                }
            }
        }
    }
}

fn spawn_reader<R>(
    stream: R,
    which: OutputStream,
    sender: tokio::sync::mpsc::UnboundedSender<(OutputStream, String)>,
) where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(stream).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if sender.send((which, std::format!("{line}\n"))).is_err() {
                break;
            }
        }
    });
}

/// Runs a backend's own read-only status command.
///
/// **External contract** — `runStatus`, `shared/backend-auth-status.mjs:183-224`.
/// stdout and stderr are merged (a CLI may print "not logged in" to either), the
/// child's stdin is closed so a CLI that would prompt gets EOF instead, and the
/// whole thing is bounded by a deadline. A probe that times out is **not** a
/// logged-out user: it produces no output, and no output parses to
/// [`AuthStatus::Unknown`](crate::auth::AuthStatus::Unknown).
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessProbeRunner;

#[async_trait]
impl ProbeRunner for ProcessProbeRunner {
    async fn run(&self, request: ProbeRequest) -> ProbeOutcome {
        let mut command = tokio::process::Command::new(&request.command);
        command.args(&request.arguments);
        command.env_clear();
        for (name, value) in request.environment.iter() {
            command.env(name, value);
        }
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut wrapped = CommandWrap::from(command);
        #[cfg(unix)]
        wrapped.wrap(process_wrap::tokio::ProcessGroup::leader());
        #[cfg(windows)]
        wrapped.wrap(process_wrap::tokio::JobObject);

        let Ok(mut child) = wrapped.spawn() else {
            return ProbeOutcome::default();
        };
        let mut stdout = child.stdout().take();
        let mut stderr = child.stderr().take();
        let gathered = tokio::time::timeout(request.timeout, async {
            let mut buffer = Vec::new();
            if let Some(stream) = &mut stdout {
                let _ = stream.read_to_end(&mut buffer).await;
            }
            if let Some(stream) = &mut stderr {
                let _ = stream.read_to_end(&mut buffer).await;
            }
            let status = child.wait().await;
            (buffer, status)
        })
        .await;
        match gathered {
            Ok((buffer, status)) => ProbeOutcome {
                ok: status.is_ok_and(|status| status.success()),
                output: String::from_utf8_lossy(&buffer).into_owned(),
            },
            Err(_) => ProbeOutcome::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use via_core::EnvMap;

    #[tokio::test]
    async fn a_missing_executable_is_a_failed_step_rather_than_a_panic() {
        let outcome = ProcessStepRunner
            .run(
                StepRequest {
                    command: "__via_backends_missing_installer__".to_owned(),
                    arguments: Vec::new(),
                    environment: EnvMap::new(),
                    timeout: Duration::from_secs(1),
                },
                &InstallCancel::new(),
                &|_, _| {},
            )
            .await;
        assert_eq!(outcome.code, -1);
        assert!(outcome.error.is_some());
        assert!(!outcome.timeout);
        assert!(!outcome.aborted);
    }

    #[tokio::test]
    async fn a_cancel_raised_before_the_step_never_spawns_anything() {
        let cancel = InstallCancel::new();
        cancel.cancel();
        let outcome = ProcessStepRunner
            .run(
                StepRequest {
                    command: "/bin/sh".to_owned(),
                    arguments: vec!["-c".to_owned(), "exit 0".to_owned()],
                    environment: EnvMap::new(),
                    timeout: Duration::from_secs(1),
                },
                &cancel,
                &|_, _| {},
            )
            .await;
        assert!(outcome.aborted);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn a_probe_reads_both_streams_and_reports_the_exit_status() {
        let outcome = ProcessProbeRunner
            .run(ProbeRequest {
                command: "/bin/sh".to_owned(),
                arguments: vec![
                    "-c".to_owned(),
                    "echo out; echo err 1>&2; exit 3".to_owned(),
                ],
                environment: EnvMap::new(),
                timeout: Duration::from_secs(5),
            })
            .await;
        assert!(!outcome.ok, "exit 3 is not success");
        assert!(outcome.output.contains("out"));
        assert!(outcome.output.contains("err"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn a_step_that_outruns_its_deadline_is_a_timeout() {
        let outcome = ProcessStepRunner
            .run(
                StepRequest {
                    command: "/bin/sh".to_owned(),
                    arguments: vec!["-c".to_owned(), "sleep 30".to_owned()],
                    environment: EnvMap::new(),
                    timeout: Duration::from_millis(100),
                },
                &InstallCancel::new(),
                &|_, _| {},
            )
            .await;
        assert!(outcome.timeout);
        assert_eq!(outcome.code, -1);
    }
}
