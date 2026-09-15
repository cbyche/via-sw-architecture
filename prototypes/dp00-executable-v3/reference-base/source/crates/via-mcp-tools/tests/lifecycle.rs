//! The builtin-MCP app-agent reaper.
//!
//! A port of the four cases in `server/test/builtin-mcp.test.mjs:50-127`, plus
//! the ones that test does not reach: the unused lifecycle, the concurrent
//! close, the pid that exits mid-ladder, and a baseline app-agent that must
//! survive.
//!
//! Every test here runs under `#[tokio::test(start_paused = true)]`, so the
//! 500 ms discovery window, its 50 ms poll and the 100 ms `SIGTERM` grace are
//! virtual: the whole ladder completes in microseconds and always in the same
//! order, with no wall-clock flakiness to retry away.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use async_trait::async_trait;
use via_mcp_tools::builtin::{BuiltinMcpKind, BuiltinMcpServer, ComputerUseLaunch};
use via_mcp_tools::lifecycle::{
    APP_AGENT_MARKER, BuiltinMcpLifecycle, LifecycleOptions, ProcessEntry, ProcessTable,
    TerminationSignal,
};

/// A scripted process table. Reading it is recorded; signalling it is
/// recorded and, unless the pid was scripted as stubborn, removes the entry —
/// so the `SIGTERM` → `SIGKILL` ladder is exercised against a table that
/// actually changes underneath it.
#[derive(Debug, Default)]
struct ScriptedProcesses {
    state: Mutex<ScriptedState>,
}

#[derive(Debug, Default)]
struct ScriptedState {
    entries: Vec<ProcessEntry>,
    signals: Vec<(u32, TerminationSignal)>,
    reads: usize,
    /// Pids that ignore `SIGTERM`, forcing the second stage.
    stubborn: Vec<u32>,
    /// Entries appended after the Nth read, so a process can appear *during*
    /// the discovery window.
    appear_after_read: Vec<(usize, ProcessEntry)>,
}

impl ScriptedProcesses {
    fn new(entries: Vec<ProcessEntry>) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(ScriptedState {
                entries,
                ..ScriptedState::default()
            }),
        })
    }

    fn locked(&self) -> std::sync::MutexGuard<'_, ScriptedState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn add(&self, entry: ProcessEntry) {
        self.locked().entries.push(entry);
    }

    fn add_after_read(&self, reads: usize, entry: ProcessEntry) {
        self.locked().appear_after_read.push((reads, entry));
    }

    fn stubborn(&self, pid: u32) {
        self.locked().stubborn.push(pid);
    }

    fn signals(&self) -> Vec<(u32, TerminationSignal)> {
        self.locked().signals.clone()
    }

    fn reads(&self) -> usize {
        self.locked().reads
    }
}

#[async_trait]
impl ProcessTable for ScriptedProcesses {
    async fn list(&self) -> Vec<ProcessEntry> {
        let mut state = self.locked();
        state.reads += 1;
        let reads = state.reads;
        let due: Vec<ProcessEntry> = state
            .appear_after_read
            .iter()
            .filter(|(after, _)| *after == reads)
            .map(|(_, entry)| entry.clone())
            .collect();
        state.entries.extend(due);
        state.entries.clone()
    }

    async fn signal(&self, pid: u32, signal: TerminationSignal) {
        let mut state = self.locked();
        state.signals.push((pid, signal));
        if signal == TerminationSignal::Kill || !state.stubborn.contains(&pid) {
            state.entries.retain(|entry| entry.pid != pid);
        }
    }
}

fn app_agent(pid: u32, socket: &str) -> ProcessEntry {
    ProcessEntry::new(
        pid,
        format!("/pkg/OpenComputerUse {APP_AGENT_MARKER} {socket}"),
    )
}

fn managed() -> Vec<BuiltinMcpServer> {
    vec![BuiltinMcpServer {
        descriptor: ComputerUseLaunch::executable("/usr/local/bin/open-computer-use").descriptor(),
        kind: BuiltinMcpKind::OpenComputerUse,
    }]
}

fn options(processes: Arc<ScriptedProcesses>, macos: bool) -> LifecycleOptions {
    LifecycleOptions {
        macos,
        processes,
        discovery: Duration::from_millis(100),
    }
}

#[tokio::test(start_paused = true)]
async fn cleans_only_app_agents_created_after_the_lifecycle_starts() {
    let processes = ScriptedProcesses::new(vec![app_agent(10, "/tmp/old.sock")]);
    let lifecycle =
        BuiltinMcpLifecycle::configure(&managed(), options(processes.clone(), true)).await;

    // Appears after the baseline was taken — this is the one that is ours.
    processes.add(app_agent(20, "/tmp/new.sock"));
    // Upstream's double is a static list that never changes, so the pid is
    // still there for the second stage. `stubborn` reproduces that; the
    // ordinary case is the next test.
    processes.stubborn(20);

    lifecycle.mark_used();
    lifecycle.close().await;

    assert_eq!(
        processes.signals(),
        vec![(20, TerminationSignal::Term), (20, TerminationSignal::Kill),],
    );
}

#[tokio::test(start_paused = true)]
async fn a_pid_that_stops_on_sigterm_is_never_sigkilled() {
    let processes = ScriptedProcesses::new(Vec::new());
    let lifecycle =
        BuiltinMcpLifecycle::configure(&managed(), options(processes.clone(), true)).await;
    processes.add(app_agent(20, "/tmp/new.sock"));

    lifecycle.mark_used();
    lifecycle.close().await;

    // The scripted table removes a pid that does not ignore SIGTERM, so the
    // second stage finds nothing to escalate against.
    assert_eq!(processes.signals(), vec![(20, TerminationSignal::Term)]);
}

#[tokio::test(start_paused = true)]
async fn a_stubborn_pid_is_escalated() {
    let processes = ScriptedProcesses::new(Vec::new());
    let lifecycle =
        BuiltinMcpLifecycle::configure(&managed(), options(processes.clone(), true)).await;
    processes.add(app_agent(20, "/tmp/new.sock"));
    processes.stubborn(20);

    lifecycle.mark_used();
    lifecycle.close().await;

    assert_eq!(
        processes.signals(),
        vec![(20, TerminationSignal::Term), (20, TerminationSignal::Kill),],
    );
}

#[tokio::test(start_paused = true)]
async fn preserves_a_new_app_agent_while_another_mcp_process_is_active() {
    let processes = ScriptedProcesses::new(Vec::new());
    let lifecycle =
        BuiltinMcpLifecycle::configure(&managed(), options(processes.clone(), true)).await;
    processes.add(app_agent(20, "/tmp/new.sock"));
    processes.add(ProcessEntry::new(30, "/pkg/OpenComputerUse mcp"));

    lifecycle.mark_used();
    lifecycle.close().await;

    assert_eq!(processes.signals(), Vec::new());
}

#[tokio::test(start_paused = true)]
async fn the_lifecycle_is_a_no_op_off_macos_or_without_the_managed_server() {
    let processes = ScriptedProcesses::new(vec![app_agent(20, "/tmp/new.sock")]);

    let off_macos =
        BuiltinMcpLifecycle::configure(&managed(), options(processes.clone(), false)).await;
    off_macos.mark_used();
    off_macos.close().await;

    let unmanaged = BuiltinMcpLifecycle::configure(&[], options(processes.clone(), true)).await;
    unmanaged.mark_used();
    unmanaged.close().await;

    // Upstream's assertion is `assert.equal(listed, false)`: the process table
    // is never even read.
    assert_eq!(processes.reads(), 0);
    assert_eq!(processes.signals(), Vec::new());
    assert!(!off_macos.manages());
    assert!(!unmanaged.manages());
}

#[tokio::test(start_paused = true)]
async fn an_unused_lifecycle_kills_nothing() {
    let processes = ScriptedProcesses::new(Vec::new());
    let lifecycle =
        BuiltinMcpLifecycle::configure(&managed(), options(processes.clone(), true)).await;
    processes.add(app_agent(20, "/tmp/new.sock"));

    // No `mark_used`: whatever is running was not started through VIA.
    lifecycle.close().await;
    assert_eq!(processes.signals(), Vec::new());
    // One read, for the baseline, and nothing since.
    assert_eq!(processes.reads(), 1);

    // Marking it used afterwards still allows a proper close — upstream's
    // `if (!used) return` sits *before* the memoised promise.
    lifecycle.mark_used();
    lifecycle.close().await;
    assert_eq!(processes.signals(), vec![(20, TerminationSignal::Term)]);
}

#[tokio::test(start_paused = true)]
async fn close_runs_once_however_many_callers_ask() {
    let processes = ScriptedProcesses::new(Vec::new());
    let lifecycle = Arc::new(
        BuiltinMcpLifecycle::configure(&managed(), options(processes.clone(), true)).await,
    );
    processes.add(app_agent(20, "/tmp/new.sock"));
    processes.stubborn(20);
    lifecycle.mark_used();

    let mut handles = Vec::new();
    for _ in 0..8 {
        let lifecycle = lifecycle.clone();
        handles.push(tokio::spawn(async move { lifecycle.close().await }));
    }
    for handle in handles {
        handle.await.expect("no panic");
    }

    // Two signals, not sixteen.
    assert_eq!(
        processes.signals(),
        vec![(20, TerminationSignal::Term), (20, TerminationSignal::Kill),],
    );
}

#[tokio::test(start_paused = true)]
async fn an_app_agent_that_appears_during_the_window_is_still_discovered() {
    let processes = ScriptedProcesses::new(Vec::new());
    let lifecycle =
        BuiltinMcpLifecycle::configure(&managed(), options(processes.clone(), true)).await;
    // The baseline read is #1; this one lands on the third read, well inside
    // the 100 ms window at a 50 ms poll.
    processes.add_after_read(3, app_agent(21, "/tmp/late.sock"));
    processes.stubborn(21);

    lifecycle.mark_used();
    lifecycle.close().await;

    assert_eq!(
        processes.signals(),
        vec![(21, TerminationSignal::Term), (21, TerminationSignal::Kill),],
    );
}

#[tokio::test(start_paused = true)]
async fn a_discovered_pid_that_has_already_exited_is_not_signalled() {
    let processes = ScriptedProcesses::new(Vec::new());
    let lifecycle =
        BuiltinMcpLifecycle::configure(&managed(), options(processes.clone(), true)).await;
    processes.add(app_agent(20, "/tmp/new.sock"));

    lifecycle.mark_used();
    // It exits on its own before the window closes.
    tokio::spawn({
        let processes = processes.clone();
        async move {
            tokio::time::sleep(Duration::from_millis(60)).await;
            processes.locked().entries.retain(|entry| entry.pid != 20);
        }
    });
    lifecycle.close().await;

    // Discovered, but not live when the ladder ran: signalling it would risk
    // hitting whatever now holds that pid.
    assert_eq!(processes.signals(), Vec::new());
}

#[tokio::test(start_paused = true)]
async fn a_baseline_app_agent_is_never_touched_even_when_ours_is_killed() {
    let processes = ScriptedProcesses::new(vec![
        app_agent(10, "/tmp/theirs.sock"),
        ProcessEntry::new(11, "/usr/bin/unrelated"),
    ]);
    let lifecycle =
        BuiltinMcpLifecycle::configure(&managed(), options(processes.clone(), true)).await;
    processes.add(app_agent(20, "/tmp/ours.sock"));
    processes.stubborn(20);

    lifecycle.mark_used();
    lifecycle.close().await;

    for (pid, _) in processes.signals() {
        assert_eq!(pid, 20, "only the pid we started may be signalled");
    }
    assert_eq!(processes.signals().len(), 2);
}

#[tokio::test(start_paused = true)]
async fn a_process_that_is_not_an_app_agent_is_never_a_candidate() {
    let processes = ScriptedProcesses::new(Vec::new());
    let lifecycle =
        BuiltinMcpLifecycle::configure(&managed(), options(processes.clone(), true)).await;
    // Looks adjacent, is not marked.
    processes.add(ProcessEntry::new(40, "/pkg/OpenComputerUse --app-agent"));
    processes.add(ProcessEntry::new(41, "/pkg/OpenComputerUseHelper"));

    lifecycle.mark_used();
    lifecycle.close().await;

    assert_eq!(processes.signals(), Vec::new());
}
