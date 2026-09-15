//! Reaping the app-agents the baseline computer-use server leaves behind.
//!
//! A port of upstream `createBuiltinMcpLifecycle`
//! (`server/src/agent/builtin-mcp.mjs:78-169`).
//!
//! # What it is actually for
//!
//! The computer-use server spawns a macOS *app agent* — a separate helper
//! process, recognised by [`APP_AGENT_MARKER`] on its command line — that does
//! not exit when the MCP server it was started for does. Left alone they
//! accumulate. So the Gateway snapshots which app-agents already existed when
//! it started, and on shutdown terminates only the ones that appeared since.
//!
//! Every one of the rules below is a rule about **not killing the wrong
//! thing**:
//!
//! * **Baseline.** Only pids absent from the start-up snapshot are candidates.
//!   Another application's helper is never touched.
//! * **Used.** If no backend session ever connected, nothing is killed —
//!   whatever is running was not started by us.
//! * **Discovery window.** An app-agent can appear *after* the MCP server has
//!   gone, so candidates are collected for [`APP_AGENT_DISCOVERY`] rather than
//!   sampled once.
//! * **Active-MCP veto.** If any computer-use MCP server is still running —
//!   another Gateway, or the user's own — nothing is killed at all. A shared
//!   helper is not ours to end.
//! * **Ladder.** `SIGTERM`, wait [`TERMINATION_GRACE`], then `SIGKILL`, and
//!   each stage re-reads the process table so a pid that has already exited is
//!   not signalled into whatever now holds its number.
//!
//! # The two seams
//!
//! Upstream injects `platform`, `listProcesses`, `killImpl`, `delay`, `now`
//! and `discoveryMs`. Here `listProcesses`/`killImpl` are one
//! [`ProcessTable`], `delay`/`now` are `tokio::time` — so
//! `#[tokio::test(start_paused = true)]` runs the whole ladder in
//! microseconds with no wall-clock flakiness — and `platform` collapses to
//! [`LifecycleOptions::macos`], the single bit upstream's `=== 'darwin'`
//! actually tests.

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::OnceCell;
use tokio::time::{Instant, sleep};
use via_downstream::text::is_js_whitespace;

use crate::builtin::{BUILTIN_MCP_LIFECYCLE, BuiltinMcpKind, BuiltinMcpServer};

/// The command-line marker that identifies a computer-use app-agent.
///
/// **External contract** — upstream `builtin-mcp.mjs:18`. **KEEP**: it is the
/// third-party helper's own argument, not a VIA string.
pub const APP_AGENT_MARKER: &str = "__open-computer-use-app-agent";

/// How long new app-agents are collected for before the ladder starts.
///
/// **External contract** — upstream `APP_AGENT_DISCOVERY_MS = 500`.
pub const APP_AGENT_DISCOVERY: Duration = Duration::from_millis(500);

/// How often the process table is re-read during the discovery window.
///
/// **External contract** — upstream `APP_AGENT_POLL_MS = 50`. The effective
/// interval is `min(poll, discovery)`, so a caller shortening the window
/// shortens the poll with it.
pub const APP_AGENT_POLL: Duration = Duration::from_millis(50);

/// The wait between `SIGTERM` and `SIGKILL`.
///
/// **External contract** — upstream `builtin-mcp.mjs:155`, catalogued as
/// *"builtin MCP … SIGTERM->SIGKILL grace 100ms"*.
pub const TERMINATION_GRACE: Duration = Duration::from_millis(100);

/// The `ps` executable and the format upstream reads.
///
/// **External contract** — upstream `builtin-mcp.mjs:81-83`:
/// `execFileSync('ps', ['-axo', 'pid=,command='])`. The `=` suffixes suppress
/// the header line.
pub const PS_ARGUMENTS: [&str; 2] = ["-axo", "pid=,command="];

/// One row of the process table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessEntry {
    /// The process id.
    pub pid: u32,
    /// The full command line.
    pub command: String,
}

impl ProcessEntry {
    /// Build an entry.
    #[must_use]
    pub fn new(pid: u32, command: impl Into<String>) -> Self {
        Self {
            pid,
            command: command.into(),
        }
    }

    /// `isAppAgent` — upstream `builtin-mcp.mjs:92-94`, a plain substring
    /// test.
    #[must_use]
    pub fn is_app_agent(&self) -> bool {
        self.command.contains(APP_AGENT_MARKER)
    }

    /// `isActiveMcp` — upstream `builtin-mcp.mjs:96-102`.
    ///
    /// `/OpenComputerUse(?:\s|$)/` **and** `/\smcp(?:\s|$)/` **and not** an
    /// app-agent. Both patterns are token tests rather than substring tests:
    /// `OpenComputerUseHelper` is not a match and neither is `--mcp-port`.
    #[must_use]
    pub fn is_active_mcp(&self) -> bool {
        token_at_end(&self.command, "OpenComputerUse", false)
            && token_at_end(&self.command, "mcp", true)
            && !self.is_app_agent()
    }
}

/// Whether `needle` occurs in `haystack` followed by ECMAScript whitespace or
/// end of string, optionally requiring whitespace before it too.
///
/// Hand-written rather than compiled, for the same reason
/// [`bearer_token`](crate::server::bearer_token) is: ECMAScript's `\s` and the
/// `regex` crate's disagree in both directions, and these two predicates
/// decide which processes receive a signal.
fn token_at_end(haystack: &str, needle: &str, require_leading_whitespace: bool) -> bool {
    if needle.is_empty() {
        return false;
    }
    let mut from = 0usize;
    while let Some(offset) = haystack.get(from..).and_then(|rest| rest.find(needle)) {
        let start = from + offset;
        let end = start + needle.len();
        let leading_ok = !require_leading_whitespace
            || haystack[..start]
                .chars()
                .next_back()
                .is_some_and(is_js_whitespace);
        let trailing_ok = haystack[end..].chars().next().is_none_or(is_js_whitespace);
        if leading_ok && trailing_ok {
            return true;
        }
        // Past this occurrence's first character, so the scan always
        // progresses and always lands on a `char` boundary.
        from = start + haystack[start..].chars().next().map_or(1, char::len_utf8);
    }
    false
}

/// Parse one line of `ps -axo pid=,command=` output.
///
/// **External contract** — upstream `builtin-mcp.mjs:83-88`:
/// `line.trim().match(/^(\d+)\s+(.+)$/)`. A line that does not match is
/// dropped. A pid too large for a `u32` is likewise dropped, where upstream
/// would carry a `Number` that can never equal a real pid — the same
/// unreachable outcome, reached earlier.
#[must_use]
pub fn parse_process_line(line: &str) -> Option<ProcessEntry> {
    let trimmed = line.trim_matches(is_js_whitespace);
    let digits: String = trimmed.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    let rest = &trimmed[digits.len()..];
    let command = rest.trim_start_matches(is_js_whitespace);
    if command.len() == rest.len() || command.is_empty() {
        return None;
    }
    Some(ProcessEntry::new(digits.parse().ok()?, command))
}

/// Parse a whole `ps` listing.
#[must_use]
pub fn parse_process_table(output: &str) -> Vec<ProcessEntry> {
    output.split('\n').filter_map(parse_process_line).collect()
}

/// The two signals the ladder sends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminationSignal {
    /// Please stop.
    Term,
    /// Stop.
    Kill,
}

impl TerminationSignal {
    /// The name upstream passes to `process.kill`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Term => "SIGTERM",
            Self::Kill => "SIGKILL",
        }
    }
}

impl fmt::Display for TerminationSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Reading and signalling processes this program did not spawn.
///
/// Not `via-process`'s job: that crate supervises children it started, through
/// `process-wrap`'s group ladder. These are strangers found by name in the
/// system process table.
#[async_trait]
pub trait ProcessTable: fmt::Debug + Send + Sync {
    /// Every process, or an empty listing when the table cannot be read.
    ///
    /// Upstream's `catch { return [] }`: an unreadable table means no
    /// candidates, never a failure. Killing nothing is always the safe answer.
    async fn list(&self) -> Vec<ProcessEntry>;

    /// Signal one pid, ignoring failure.
    ///
    /// Upstream wraps each `process.kill` in `try {} catch {}` with the
    /// comment *"the app-agent may exit between process discovery and
    /// signaling"* — a race, not an error.
    async fn signal(&self, pid: u32, signal: TerminationSignal);
}

/// The real table: `ps` for reading, `kill(2)` for signalling.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemProcessTable;

#[async_trait]
impl ProcessTable for SystemProcessTable {
    async fn list(&self) -> Vec<ProcessEntry> {
        match tokio::process::Command::new("ps")
            .args(PS_ARGUMENTS)
            .output()
            .await
        {
            Ok(output) => parse_process_table(&String::from_utf8_lossy(&output.stdout)),
            Err(_) => Vec::new(),
        }
    }

    #[cfg(unix)]
    async fn signal(&self, pid: u32, signal: TerminationSignal) {
        let Ok(raw) = i32::try_from(pid) else {
            return;
        };
        let signal = match signal {
            TerminationSignal::Term => nix::sys::signal::Signal::SIGTERM,
            TerminationSignal::Kill => nix::sys::signal::Signal::SIGKILL,
        };
        let _ = nix::sys::signal::kill(nix::unistd::Pid::from_raw(raw), signal);
    }

    #[cfg(not(unix))]
    async fn signal(&self, _pid: u32, _signal: TerminationSignal) {
        // Upstream's lifecycle is macOS-only; there is nothing to signal here.
    }
}

/// How a lifecycle is configured.
#[derive(Debug, Clone)]
pub struct LifecycleOptions {
    /// Whether this host is macOS.
    ///
    /// Upstream's `platform === 'darwin'`, reduced to the one bit it tests.
    /// Defaults to the build target; a test sets it explicitly, which is how
    /// the off-macOS no-op is provable on a macOS developer machine.
    pub macos: bool,
    /// How processes are read and signalled.
    pub processes: Arc<dyn ProcessTable>,
    /// How long to collect new app-agents for.
    pub discovery: Duration,
}

impl Default for LifecycleOptions {
    fn default() -> Self {
        Self {
            macos: cfg!(target_os = "macos"),
            processes: Arc::new(SystemProcessTable),
            discovery: APP_AGENT_DISCOVERY,
        }
    }
}

/// What a managing lifecycle remembers.
#[derive(Debug)]
struct Managed {
    baseline: Vec<u32>,
    options: LifecycleOptions,
}

/// The cleanup that runs when the Gateway stops.
///
/// Built by [`Self::configure`], told a backend actually used the builtin
/// server by [`Self::mark_used`], and run once by [`Self::close`].
#[derive(Debug)]
pub struct BuiltinMcpLifecycle {
    /// `None` when there is nothing to manage: no computer-use descriptor, or
    /// not macOS. In that state the process table is **never read** —
    /// upstream's own test asserts exactly that.
    managed: Option<Managed>,
    used: AtomicBool,
    closed: OnceCell<()>,
}

impl BuiltinMcpLifecycle {
    /// Configure a lifecycle for these builtin servers.
    ///
    /// Snapshots the existing app-agents immediately, as upstream does in its
    /// constructor: anything already running belongs to somebody else.
    pub async fn configure(servers: &[BuiltinMcpServer], options: LifecycleOptions) -> Self {
        let manages = servers
            .iter()
            .any(|server| server.kind == BuiltinMcpKind::OpenComputerUse);
        if !manages || !options.macos {
            return Self {
                managed: None,
                used: AtomicBool::new(false),
                closed: OnceCell::new(),
            };
        }
        let baseline = options
            .processes
            .list()
            .await
            .into_iter()
            .filter(ProcessEntry::is_app_agent)
            .map(|entry| entry.pid)
            .collect();
        Self {
            managed: Some(Managed { baseline, options }),
            used: AtomicBool::new(false),
            closed: OnceCell::new(),
        }
    }

    /// Configure against the real process table and this build's platform.
    pub async fn system(servers: &[BuiltinMcpServer]) -> Self {
        Self::configure(servers, LifecycleOptions::default()).await
    }

    /// Whether this lifecycle manages anything at all.
    #[must_use]
    pub const fn manages(&self) -> bool {
        self.managed.is_some()
    }

    /// Record that a backend session was given the builtin server.
    ///
    /// Until this is called, [`Self::close`] does nothing: an app-agent that
    /// appeared while VIA was running but was not started through VIA is not
    /// VIA's to kill.
    pub fn mark_used(&self) {
        self.used.store(true, Ordering::Release);
    }

    /// Whether [`Self::mark_used`] was called.
    #[must_use]
    pub fn is_used(&self) -> bool {
        self.used.load(Ordering::Acquire)
    }

    /// Run the cleanup, at most once.
    ///
    /// Upstream memoizes `closePromise`, so concurrent callers share one run;
    /// a [`OnceCell`] is that memo. The `!used` early return happens *before*
    /// the memo, exactly as upstream's does — a lifecycle closed before it was
    /// used can still be closed properly afterwards.
    pub async fn close(&self) {
        if !self.is_used() {
            return;
        }
        let Some(managed) = self.managed.as_ref() else {
            return;
        };
        self.closed.get_or_init(|| reap(managed)).await;
    }
}

/// The discovery window, the veto and the two-stage ladder.
async fn reap(managed: &Managed) {
    let processes = managed.options.processes.as_ref();

    // The window. `deadline` is fixed before the first read, so a slow `ps`
    // shortens the window rather than extending it — upstream's behaviour,
    // because `now()` is sampled once.
    let deadline = Instant::now() + managed.options.discovery;
    let interval = APP_AGENT_POLL.min(managed.options.discovery);
    let mut discovered: Vec<u32> = Vec::new();
    loop {
        for entry in processes.list().await {
            if entry.is_app_agent()
                && !managed.baseline.contains(&entry.pid)
                && !discovered.contains(&entry.pid)
            {
                discovered.push(entry.pid);
            }
        }
        if Instant::now() >= deadline {
            break;
        }
        sleep(interval).await;
    }

    // The veto. Somebody else's computer-use MCP server is still up, so the
    // helpers may be theirs.
    let current = processes.list().await;
    if current.iter().any(ProcessEntry::is_active_mcp) {
        tracing::debug!(
            event = "mcp.builtin_lifecycle_vetoed",
            marker = BUILTIN_MCP_LIFECYCLE,
            discovered = discovered.len(),
        );
        return;
    }

    let live: Vec<u32> = current.iter().map(|entry| entry.pid).collect();
    for pid in discovered.iter().copied().filter(|pid| live.contains(pid)) {
        processes.signal(pid, TerminationSignal::Term).await;
    }

    sleep(TERMINATION_GRACE).await;

    let remaining: Vec<u32> = processes
        .list()
        .await
        .into_iter()
        .map(|entry| entry.pid)
        .collect();
    for pid in discovered
        .iter()
        .copied()
        .filter(|pid| remaining.contains(pid))
    {
        processes.signal(pid, TerminationSignal::Kill).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ps_line_is_a_pid_then_a_command() {
        assert_eq!(
            parse_process_line("  1234   /pkg/OpenComputerUse mcp  "),
            Some(ProcessEntry::new(1234, "/pkg/OpenComputerUse mcp")),
        );
        assert_eq!(parse_process_line(""), None);
        assert_eq!(parse_process_line("1234"), None);
        assert_eq!(parse_process_line("1234 "), None);
        assert_eq!(parse_process_line("abc /bin/sh"), None);
        assert_eq!(parse_process_line("  "), None);
    }

    #[test]
    fn a_table_drops_the_lines_that_do_not_parse() {
        let table = parse_process_table("1 /a\nnonsense\n\n22   /b c\n");
        assert_eq!(
            table,
            vec![ProcessEntry::new(1, "/a"), ProcessEntry::new(22, "/b c")],
        );
    }

    #[test]
    fn the_app_agent_marker_is_a_substring_test() {
        assert!(ProcessEntry::new(1, format!("/x {APP_AGENT_MARKER} /tmp/s.sock")).is_app_agent());
        assert!(!ProcessEntry::new(1, "/x --open-computer-use").is_app_agent());
    }

    #[test]
    fn active_mcp_needs_both_tokens_and_no_marker() {
        assert!(ProcessEntry::new(1, "/pkg/OpenComputerUse mcp").is_active_mcp());
        assert!(ProcessEntry::new(1, "/pkg/OpenComputerUse mcp --port 1").is_active_mcp());
        // `OpenComputerUse` must be a whole token.
        assert!(!ProcessEntry::new(1, "/pkg/OpenComputerUseHelper mcp").is_active_mcp());
        // `mcp` must be a whole token, with whitespace before it.
        assert!(!ProcessEntry::new(1, "/pkg/OpenComputerUse mcpx").is_active_mcp());
        assert!(!ProcessEntry::new(1, "/pkg/OpenComputerUse --mcp").is_active_mcp());
        // An app-agent is never an active MCP server.
        assert!(
            !ProcessEntry::new(1, format!("/pkg/OpenComputerUse mcp {APP_AGENT_MARKER}"),)
                .is_active_mcp()
        );
    }

    #[test]
    fn the_two_signal_names_are_upstreams() {
        assert_eq!(TerminationSignal::Term.as_str(), "SIGTERM");
        assert_eq!(TerminationSignal::Kill.as_str(), "SIGKILL");
    }
}
