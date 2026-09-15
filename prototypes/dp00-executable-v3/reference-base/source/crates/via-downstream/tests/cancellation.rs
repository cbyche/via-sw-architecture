//! Cancellation as a state machine: every legal edge, and every illegal one.
//!
//! The property under test is the one `docs/architecture.md` §4 states as
//! *"cancelling is a state, not an action"*: there must be no way to get from
//! a cancel that was merely **sent** to a report that work **stopped**, other
//! than through an explicit confirmation.

use pretty_assertions::assert_eq;
use rstest::rstest;
use via_downstream::{CancelOutcome, CancelRoute, CancelScope, CancelTarget, TerminalState};
use via_protocol::WorkStatus;

fn requested() -> CancelOutcome {
    CancelOutcome::requested(CancelRoute::Adapter, CancelTarget::delegation("d-1"))
}

fn confirmed() -> CancelOutcome {
    requested()
        .confirm("2026-08-22T09:00:00.000Z")
        .expect("the one legal edge")
}

fn already(state: TerminalState) -> CancelOutcome {
    CancelOutcome::AlreadyFinished {
        target: CancelTarget::session("s-1"),
        state,
    }
}

/// Exactly one starting state can be confirmed.
#[test]
fn requested_is_the_only_confirmable_state() {
    assert!(requested().confirm("t").is_ok());

    for outcome in [
        CancelOutcome::NotFound,
        already(TerminalState::Completed),
        already(TerminalState::Failed),
        already(TerminalState::Cancelled),
        confirmed(),
    ] {
        let before = outcome.status_str();
        let error = outcome
            .confirm("t")
            .expect_err("only a delivered cancel can be confirmed");
        assert_eq!(error.state, before);
        assert_eq!(error.code(), "VIA_HARNESS_CANCEL_NOT_AWAITED");
    }
}

/// Confirming twice is refused: two instants both claiming to be *the*
/// confirmation is exactly the ambiguity `confirmed_at` exists to remove.
#[test]
fn a_confirmation_cannot_be_overwritten() {
    let once = confirmed();
    assert_eq!(once.confirmed_at(), Some("2026-08-22T09:00:00.000Z"));
    let error = once
        .clone()
        .confirm("2026-08-22T10:00:00.000Z")
        .expect_err("already confirmed");
    assert_eq!(error.state, "cancelled");
    assert_eq!(once.confirmed_at(), Some("2026-08-22T09:00:00.000Z"));
}

/// The whole state → status projection, in one table.
#[rstest]
#[case::delivered(requested(), Some(WorkStatus::Cancelling), false)]
#[case::confirmed(confirmed(), Some(WorkStatus::Cancelled), true)]
#[case::nothing_to_cancel(CancelOutcome::NotFound, None, false)]
#[case::already_completed(already(TerminalState::Completed), Some(WorkStatus::Completed), false)]
#[case::already_failed(already(TerminalState::Failed), Some(WorkStatus::Failed), false)]
#[case::already_cancelled(already(TerminalState::Cancelled), Some(WorkStatus::Cancelled), true)]
fn every_outcome_projects_to_one_work_status(
    #[case] outcome: CancelOutcome,
    #[case] status: Option<WorkStatus>,
    #[case] confirmed: bool,
) {
    assert_eq!(outcome.work_status(), status);
    assert_eq!(outcome.is_confirmed(), confirmed);
}

/// No outcome that is not confirmed may report a terminal Work status other
/// than one it *found* — and none of them may report `cancelled` unless the
/// cancellation was actually observed.
#[test]
fn only_an_observed_cancellation_reports_cancelled() {
    let reports_cancelled = [
        requested(),
        confirmed(),
        CancelOutcome::NotFound,
        already(TerminalState::Completed),
        already(TerminalState::Failed),
        already(TerminalState::Cancelled),
    ]
    .into_iter()
    .filter(|outcome| outcome.work_status() == Some(WorkStatus::Cancelled))
    .count();
    // Only the confirmed one, and the one that found an earlier confirmed one.
    assert_eq!(reports_cancelled, 2);
}

/// `confirmed_at` exists only on a confirmed outcome, and nowhere else.
#[test]
fn only_a_confirmed_outcome_carries_an_instant() {
    assert_eq!(requested().confirmed_at(), None);
    assert_eq!(CancelOutcome::NotFound.confirmed_at(), None);
    assert_eq!(already(TerminalState::Cancelled).confirmed_at(), None);
    assert_eq!(confirmed().confirmed_at(), Some("2026-08-22T09:00:00.000Z"));
}

/// A route exists only where a cancel was actually delivered.
#[test]
fn only_a_delivered_cancel_has_a_route() {
    assert_eq!(requested().route(), Some(CancelRoute::Adapter));
    assert_eq!(confirmed().route(), Some(CancelRoute::Adapter));
    assert_eq!(CancelOutcome::NotFound.route(), None);
    assert_eq!(already(TerminalState::Completed).route(), None);
    assert_eq!(CancelRoute::Coordinator.as_str(), "coordinator");
    assert_eq!(CancelRoute::Adapter.as_str(), "adapter");
}

/// The confirmation carries the route it was confirmed over, so a coordinator
/// confirmation and a transport one are distinguishable after the fact.
#[test]
fn confirmation_preserves_the_route() {
    let outcome =
        CancelOutcome::requested(CancelRoute::Coordinator, CancelTarget::delegation("d-1"))
            .confirm("t")
            .expect("legal");
    assert_eq!(outcome.route(), Some(CancelRoute::Coordinator));
    assert_eq!(
        outcome
            .target()
            .and_then(|target| target.delegation_id.as_deref()),
        Some("d-1"),
    );
}

/// `NotFound` names nothing, and serializes bare.
#[test]
fn nothing_to_cancel_names_nothing() {
    assert_eq!(CancelOutcome::NotFound.target(), None);
    assert_eq!(
        serde_json::to_string(&CancelOutcome::NotFound).expect("serializable"),
        r#"{"status":"not_found"}"#,
    );
}

/// Scope names, and the delegation id a scope carries.
#[test]
fn scopes_are_named_and_carry_their_target() {
    assert_eq!(CancelScope::Turn.as_str(), "turn");
    assert_eq!(CancelScope::Session.as_str(), "session");
    let delegation = CancelScope::Delegation {
        delegation_id: "d-1".to_owned(),
    };
    assert_eq!(delegation.as_str(), "delegation");
    assert_eq!(delegation.delegation_id(), Some("d-1"));
    assert_eq!(CancelScope::Turn.delegation_id(), None);
    assert_eq!(CancelScope::Session.delegation_id(), None);
}
