//! The Work record's public shape and its lifecycle graph.
//!
//! Literals from upstream `server/src/task/task-manager.mjs:7-16,58,361,414`
//! and `docs/reference/contracts.json` ("Work status values", "workState
//! values", "Work kind values"). The transition graph is
//! `docs/architecture.md` §4, reconciled against the transitions upstream
//! actually performs — see `WorkStatus::can_transition_to`.

use std::collections::BTreeSet;
use std::str::FromStr;

use pretty_assertions::assert_eq;
use rstest::rstest;
use via_protocol::{ProtocolError, WorkKind, WorkState, WorkStatus};

use WorkStatus::{
    Cancelled, Cancelling, Completed, Delegated, Failed, Finalizing, Queued, Running, Scheduled,
};

/// `task-manager.mjs:7-16` — the nine statuses, in declaration order.
const STATUSES: &[(WorkStatus, &str)] = &[
    (Scheduled, "scheduled"),
    (Queued, "queued"),
    (Running, "running"),
    (Delegated, "delegated"),
    (Finalizing, "finalizing"),
    (Cancelling, "cancelling"),
    (Completed, "completed"),
    (Failed, "failed"),
    (Cancelled, "cancelled"),
];

/// `task-manager.mjs:58` — the five values `workState` can take.
const STATES: &[(WorkState, &str)] = &[
    (WorkState::Active, "active"),
    (WorkState::Scheduled, "scheduled"),
    (WorkState::Completed, "completed"),
    (WorkState::Failed, "failed"),
    (WorkState::Cancelled, "cancelled"),
];

/// `task-manager.mjs:361,414`; `tool-call-handler.mjs:886`.
const KINDS: &[(WorkKind, &str)] = &[
    (WorkKind::Work, "work"),
    (WorkKind::Reminder, "reminder"),
    (WorkKind::ScheduledTask, "scheduled_task"),
    (WorkKind::Control, "control"),
];

/// Every edge of the lifecycle graph, with the upstream line that performs it.
const LEGAL_EDGES: &[(WorkStatus, WorkStatus)] = &[
    (Scheduled, Queued),     // reminder-scheduler.mjs:66,103
    (Scheduled, Cancelled),  // task-manager.mjs:718 — short-circuit
    (Queued, Running),       // task-manager.mjs:479
    (Queued, Cancelled),     // task-manager.mjs:718 — short-circuit
    (Running, Delegated),    // task-manager.mjs:508
    (Running, Completed),    // task-manager.mjs:657
    (Running, Failed),       // task-manager.mjs:666
    (Running, Cancelling),   // task-manager.mjs:721
    (Delegated, Finalizing), // task-manager.mjs:522
    (Delegated, Completed),
    (Delegated, Failed),
    (Delegated, Cancelling),
    (Finalizing, Completed),
    (Finalizing, Failed),
    (Finalizing, Cancelling),
    (Cancelling, Cancelled), // task-manager.mjs:775
    (Cancelling, Failed),    // task-manager.mjs:752 — the canceler itself threw
];

#[test]
fn there_are_exactly_nine_statuses_in_upstream_order() {
    assert_eq!(WorkStatus::ALL.len(), 9);
    assert_eq!(
        WorkStatus::ALL
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
        vec![
            "scheduled",
            "queued",
            "running",
            "delegated",
            "finalizing",
            "cancelling",
            "completed",
            "failed",
            "cancelled",
        ],
    );
}

#[test]
fn statuses_states_and_kinds_round_trip_through_serde() {
    for (status, wire) in STATUSES {
        assert_eq!(status.as_str(), *wire);
        assert_eq!(status.to_string(), *wire);
        assert_eq!(WorkStatus::from_wire(wire), Some(*status));
        assert_eq!(WorkStatus::from_str(wire), Ok(*status));
        assert_eq!(
            serde_json::to_string(status).expect("serialize"),
            format!("\"{wire}\""),
        );
    }
    for (state, wire) in STATES {
        assert_eq!(state.as_str(), *wire);
        assert_eq!(WorkState::from_wire(wire), Some(*state));
        assert_eq!(
            serde_json::to_string(state).expect("serialize"),
            format!("\"{wire}\""),
        );
    }
    for (kind, wire) in KINDS {
        assert_eq!(kind.as_str(), *wire);
        assert_eq!(WorkKind::from_wire(wire), Some(*kind));
        assert_eq!(
            serde_json::to_string(kind).expect("serialize"),
            format!("\"{wire}\""),
        );
    }
}

/// `scheduled_task` keeps its underscore — it is a stored value, not a display
/// label, and a dash would silently orphan every persisted record.
#[test]
fn scheduled_task_is_snake_case_on_the_wire() {
    assert_eq!(WorkKind::ScheduledTask.as_str(), "scheduled_task");
    assert_eq!(WorkKind::from_wire("scheduled-task"), None);
    assert_eq!(WorkKind::from_wire("scheduledTask"), None);
}

#[test]
fn the_default_kind_is_work() {
    assert_eq!(WorkKind::default(), WorkKind::Work);
    assert_eq!(WorkKind::default().as_str(), "work");
}

#[test]
fn control_work_is_the_only_kind_hidden_from_clients() {
    for (kind, _) in KINDS {
        assert_eq!(
            kind.is_client_visible(),
            *kind != WorkKind::Control,
            "{kind} client visibility",
        );
    }
}

/// `workState = ACTIVE.has(status) ? 'active' : status` — the projection
/// upstream publishes on every record.
#[rstest]
#[case(Scheduled, WorkState::Scheduled)]
#[case(Queued, WorkState::Active)]
#[case(Running, WorkState::Active)]
#[case(Delegated, WorkState::Active)]
#[case(Finalizing, WorkState::Active)]
#[case(Cancelling, WorkState::Active)]
#[case(Completed, WorkState::Completed)]
#[case(Failed, WorkState::Failed)]
#[case(Cancelled, WorkState::Cancelled)]
fn status_projects_onto_the_published_work_state(
    #[case] status: WorkStatus,
    #[case] state: WorkState,
) {
    assert_eq!(status.state(), state);
}

/// The non-`active` projections are the status string itself, which is what
/// lets a client compare `workState` and `status` directly.
#[test]
fn a_non_active_state_is_spelled_like_its_status() {
    for (status, wire) in STATUSES {
        if status.is_active() {
            assert_eq!(status.state().as_str(), "active");
        } else {
            assert_eq!(status.state().as_str(), *wire);
        }
    }
}

/// `scheduled` reaches the wire as a `workState`: it is not one of the five
/// statuses `ACTIVE` collapses, so `publicTask` publishes it verbatim.
#[test]
fn scheduled_is_a_published_work_state_not_an_active_one() {
    assert_eq!(Scheduled.state(), WorkState::Scheduled);
    assert!(!Scheduled.is_active());
    assert_eq!(WorkState::ALL.len(), 5);
}

#[test]
fn the_upstream_status_sets_are_reproduced() {
    assert_eq!(
        WorkStatus::ACTIVE,
        &[Queued, Running, Delegated, Finalizing, Cancelling],
    );
    assert_eq!(
        WorkStatus::CANCELLABLE,
        &[Scheduled, Queued, Running, Delegated, Finalizing],
    );
    assert_eq!(WorkStatus::TERMINAL, &[Completed, Failed, Cancelled]);
    assert_eq!(WorkStatus::REPLAYABLE_REMINDER, &[Queued, Running]);

    for status in WorkStatus::ALL {
        assert_eq!(status.is_active(), WorkStatus::ACTIVE.contains(status));
        assert_eq!(
            status.is_cancellable(),
            WorkStatus::CANCELLABLE.contains(status),
        );
        assert_eq!(status.is_terminal(), WorkStatus::TERMINAL.contains(status));
        assert_eq!(
            status.is_replayable_reminder(),
            WorkStatus::REPLAYABLE_REMINDER.contains(status),
        );
    }
}

/// `cancelling` is not in `CANCELLABLE`: a second cancel joins the in-flight
/// one rather than starting a new one.
#[test]
fn cancelling_is_not_itself_cancellable() {
    assert!(!Cancelling.is_cancellable());
    assert!(Cancelling.is_active());
    assert!(!Cancelling.is_terminal());
}

/// The whole 9x9 matrix: the 17 edges above are legal and the other 64 pairs
/// are not. Asserting the complement is the point — an over-permissive graph is
/// the failure mode a hand-written list of positive cases cannot catch.
#[test]
fn the_transition_graph_is_exactly_seventeen_edges() {
    let legal: BTreeSet<(WorkStatus, WorkStatus)> = LEGAL_EDGES.iter().copied().collect();
    assert_eq!(legal.len(), 17, "the edge table has a duplicate");

    let mut permitted = 0usize;
    for from in WorkStatus::ALL {
        for to in WorkStatus::ALL {
            let expected = legal.contains(&(*from, *to));
            assert_eq!(
                from.can_transition_to(*to),
                expected,
                "{from} -> {to} should be {}",
                if expected { "legal" } else { "illegal" },
            );
            if expected {
                permitted += 1;
            }
        }
    }
    assert_eq!(permitted, 17);
    assert_eq!(WorkStatus::ALL.len() * WorkStatus::ALL.len(), 81);
}

/// Terminal means terminal: nothing leaves `completed`, `failed` or
/// `cancelled`, including to itself.
#[rstest]
#[case(Completed)]
#[case(Failed)]
#[case(Cancelled)]
fn terminal_statuses_have_no_outgoing_edges(#[case] terminal: WorkStatus) {
    assert!(terminal.is_terminal());
    for to in WorkStatus::ALL {
        assert!(
            !terminal.can_transition_to(*to),
            "{terminal} -> {to} must be illegal",
        );
    }
}

#[test]
fn no_status_transitions_to_itself() {
    for status in WorkStatus::ALL {
        assert!(!status.can_transition_to(*status), "{status} -> {status}");
    }
}

/// The illegal edges worth naming, each of which a plausible implementation
/// would wrongly allow.
#[rstest]
// Nothing has started, so upstream short-circuits straight to `cancelled`
// (task-manager.mjs:718) — these two never pass through `cancelling`.
#[case(Scheduled, Cancelling)]
#[case(Queued, Cancelling)]
// `scheduled` is dispatched by the reminder scheduler into the queue; it never
// starts directly (reminder-scheduler.mjs:66,103).
#[case(Scheduled, Running)]
#[case(Scheduled, Delegated)]
#[case(Scheduled, Completed)]
#[case(Scheduled, Failed)]
// Admission is the scheduler's job; a queued Work cannot be delegated or
// completed without running.
#[case(Queued, Delegated)]
#[case(Queued, Finalizing)]
#[case(Queued, Completed)]
#[case(Queued, Failed)]
// `finalizing` means "a *delegated* session reported completion", so it is
// reachable only through `delegated` (task-manager.mjs:522).
#[case(Running, Finalizing)]
// Cancellation is confirmed, not optimistic: a result arriving after a stop was
// requested is dropped, never published.
#[case(Cancelling, Completed)]
#[case(Cancelling, Delegated)]
#[case(Cancelling, Running)]
// The lifecycle does not run backwards. `restore()` rewriting a persisted
// `delegated` record to `queued` is a store-level recovery of a record from a
// dead process, not a transition of a live one.
#[case(Delegated, Running)]
#[case(Delegated, Queued)]
#[case(Finalizing, Delegated)]
#[case(Running, Queued)]
#[case(Queued, Scheduled)]
fn named_illegal_edges_are_refused(#[case] from: WorkStatus, #[case] to: WorkStatus) {
    assert!(
        !from.can_transition_to(to),
        "{from} -> {to} must be illegal"
    );
    assert_eq!(
        from.transition_to(to),
        Err(ProtocolError::IllegalTransition { from, to }),
    );
}

#[test]
fn a_legal_transition_yields_the_next_status() {
    for (from, to) in LEGAL_EDGES {
        assert_eq!(from.transition_to(*to), Ok(*to), "{from} -> {to}");
    }
}

/// Every non-terminal status can reach a terminal one in one step, so no Work
/// can be wedged: there is always a way out.
#[test]
fn every_non_terminal_status_can_reach_a_terminal_one() {
    for from in WorkStatus::ALL {
        if from.is_terminal() {
            continue;
        }
        assert!(
            WorkStatus::TERMINAL
                .iter()
                .any(|terminal| from.can_transition_to(*terminal)),
            "{from} has no single-step exit to a terminal status",
        );
    }
}

/// Every non-terminal status is reachable from the two entry points upstream
/// creates records in: `queued` (`create`) and `scheduled` (`createScheduled`).
#[test]
fn every_status_is_reachable_from_an_entry_point() {
    let mut reached: BTreeSet<WorkStatus> = [Queued, Scheduled].into_iter().collect();
    loop {
        let next: BTreeSet<WorkStatus> = reached
            .iter()
            .flat_map(|from| {
                WorkStatus::ALL
                    .iter()
                    .filter(move |to| from.can_transition_to(**to))
                    .copied()
            })
            .collect();
        let grown: BTreeSet<WorkStatus> = reached.union(&next).copied().collect();
        if grown == reached {
            break;
        }
        reached = grown;
    }
    assert_eq!(reached.len(), 9, "unreachable statuses: {reached:?}");
}

#[test]
fn unknown_work_strings_are_rejected() {
    assert_eq!(WorkStatus::from_wire("pending"), None);
    assert_eq!(WorkStatus::from_wire("done"), None);
    assert_eq!(WorkStatus::from_wire("active"), None);
    assert_eq!(WorkState::from_wire("queued"), None);
    assert_eq!(WorkKind::from_wire("task"), None);
    assert_eq!(
        WorkStatus::from_str("Running"),
        Err(ProtocolError::UnknownWireValue {
            vocabulary: "WorkStatus",
            value: "Running".to_owned(),
        }),
    );
}
