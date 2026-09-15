//! Cancellation, modelled as **confirmed rather than optimistic**.
//!
//! Upstream's `cancelDelegation`
//! (`server/src/agent/acp-backend-adapter.mjs:907-928`) sets
//! `record.status = 'cancelled'` *before* it aborts anything, and reports that
//! status back through `qwen_audio_agent_session_cancel`. The transport call it
//! then makes is `session/cancel`, an ACP **notification** — fire and forget,
//! with the agent's own `stopReason: 'cancelled'` arriving later, if at all.
//! So the model is told the work is cancelled while it may still be running.
//!
//! `docs/architecture.md` calls this out twice: §4 lists `cancelling` as a Work
//! status of its own — *"cancelling is a state, not an action"* — and §6 names
//! ARGO's `argo-a2a-delegation` as *"the one place cancellation-as-state is done
//! right"*. This module is where VIA takes that side.
//!
//! # The shape of the guarantee
//!
//! [`CancelOutcome`] has no method that reports a cancel as done unless the
//! harness confirmed it:
//!
//! - [`CancelOutcome::Requested`] projects to
//!   [`WorkStatus::Cancelling`](via_protocol::WorkStatus::Cancelling), never to
//!   `Cancelled`;
//! - [`CancelOutcome::is_confirmed`] is false for it;
//! - a `Requested` outcome becomes `Confirmed` only through
//!   [`CancelOutcome::confirm`], which needs a confirmation timestamp and
//!   refuses every other starting state.
//!
//! There is deliberately no `From<CancelScope> for CancelOutcome` and no
//! `Default`: an outcome exists because something reported one.

use serde::Serialize;
use serde::ser::SerializeMap;
use via_protocol::WorkStatus;

use crate::capability::BackendCapabilities;

/// The `status` a cancel reports when there was nothing to cancel.
///
/// `docs/reference/contracts.json` (`json-field` / *session_cancel result*):
/// `{ status: 'not_found' }` is returned **bare**, with no other fields.
pub const STATUS_NOT_FOUND: &str = "not_found";

/// What a cancel is aimed at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelScope {
    /// The prompt currently in flight on this session, and nothing else.
    ///
    /// Upstream's `session/cancel` on the coordinator session
    /// (`server/src/agent/acp-process-client.mjs:482-487`).
    Turn,
    /// The session as a whole: the in-flight prompt and anything it owns.
    ///
    /// Upstream's teardown path, `acp-backend-adapter.mjs:1464`.
    Session,
    /// One delegated project session, by the delegation id VIA gave it.
    ///
    /// Upstream `cancelDelegation({ delegation_id })`,
    /// `acp-backend-adapter.mjs:907`.
    Delegation {
        /// The delegation id.
        delegation_id: String,
    },
}

impl CancelScope {
    /// A stable machine-readable name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Turn => "turn",
            Self::Session => "session",
            Self::Delegation { .. } => "delegation",
        }
    }

    /// Whether a harness with these capabilities can serve this scope.
    ///
    /// Cancelling a *delegation* presupposes the harness can delegate at all;
    /// upstream refuses with `当前后台 Agent 不支持取消第三层 Session`
    /// ([`HarnessError::CancelUnsupported`](crate::HarnessError::CancelUnsupported)).
    /// Cancelling a turn or a session is transport-level and every harness owes
    /// it.
    #[must_use]
    pub const fn is_supported_by(&self, capabilities: BackendCapabilities) -> bool {
        match self {
            Self::Turn | Self::Session => true,
            Self::Delegation { .. } => capabilities.delegation,
        }
    }

    /// The delegation id, when this scope names one.
    #[must_use]
    pub fn delegation_id(&self) -> Option<&str> {
        match self {
            Self::Delegation { delegation_id } => Some(delegation_id),
            Self::Turn | Self::Session => None,
        }
    }
}

/// How the cancel reached the harness.
///
/// Upstream's `route` field, `acp-backend-adapter.mjs:1396,1419`. It matters
/// because the two routes carry different evidence: the coordinator route
/// returns only after the coordinator's own tool call came back, so it *is* the
/// confirmation; the adapter route is a transport notification with nothing
/// coming back, which is exactly the case this module refuses to call done.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CancelRoute {
    /// Asked the coordinator session to cancel, and it answered.
    Coordinator,
    /// Sent straight down the transport, bypassing the coordinator.
    ///
    /// Upstream falls back to this when the coordinator is mid-turn or its
    /// control turn threw — *"cancellation is urgent"*
    /// (`acp-backend-adapter.mjs:1404`).
    Adapter,
}

impl CancelRoute {
    /// A stable machine-readable name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Coordinator => "coordinator",
            Self::Adapter => "adapter",
        }
    }
}

/// What the cancel was aimed at, as reported back.
///
/// Upstream returns `{ delegationId, sessionId }` from `cancelDelegatedWork`
/// and `{ delegation_id, session_id }` from the MCP tool.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CancelTarget {
    /// The delegation id, when the scope named one.
    pub delegation_id: Option<String>,
    /// The backend's session id, when it is known.
    pub session_id: Option<String>,
}

impl CancelTarget {
    /// A target that names only a delegation.
    #[must_use]
    pub fn delegation(delegation_id: &str) -> Self {
        Self {
            delegation_id: Some(delegation_id.to_owned()),
            session_id: None,
        }
    }

    /// A target that names only a session.
    #[must_use]
    pub fn session(session_id: &str) -> Self {
        Self {
            delegation_id: None,
            session_id: Some(session_id.to_owned()),
        }
    }
}

/// The three states work can already be in when a cancel arrives.
///
/// Upstream's short-circuit,
/// `['completed', 'failed', 'cancelled'].includes(record.status)`
/// (`acp-backend-adapter.mjs:910`): terminal work is left alone and its
/// existing status is reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminalState {
    /// It had already finished successfully.
    Completed,
    /// It had already failed.
    Failed,
    /// It had already been cancelled — by an earlier, confirmed cancel.
    Cancelled,
}

impl TerminalState {
    /// The Work status this projects to.
    #[must_use]
    pub const fn work_status(self) -> WorkStatus {
        match self {
            Self::Completed => WorkStatus::Completed,
            Self::Failed => WorkStatus::Failed,
            Self::Cancelled => WorkStatus::Cancelled,
        }
    }
}

/// What came of a cancel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelOutcome {
    /// The cancel was delivered. **The harness has not confirmed it.**
    ///
    /// This is the honest answer for a fire-and-forget transport notification.
    /// It projects to [`WorkStatus::Cancelling`] and
    /// [`Self::is_confirmed`] is false, so nothing above this seam can report
    /// the work as cancelled on the strength of it.
    Requested {
        /// How the cancel was delivered.
        route: CancelRoute,
        /// What it was aimed at.
        target: CancelTarget,
    },
    /// The harness confirmed the work stopped.
    ///
    /// Reached when the coordinator's own cancel tool returned, or when the
    /// transport reported `stopReason: 'cancelled'` for the turn. The timestamp
    /// is upstream's `confirmed_at` — the field the reconciliation fact carries
    /// so the model is told what the Gateway *verified*, not what it attempted
    /// (`acp-backend-adapter.mjs:1407-1416`).
    Confirmed {
        /// How the cancel was delivered.
        route: CancelRoute,
        /// What it was aimed at.
        target: CancelTarget,
        /// ISO-8601 instant the confirmation was observed. Supplied by the
        /// caller: this crate reads no clock.
        confirmed_at: String,
    },
    /// There was nothing under that id to cancel.
    ///
    /// Serializes bare, as [`STATUS_NOT_FOUND`] and nothing else.
    NotFound,
    /// The work had already reached a terminal state before the cancel
    /// arrived.
    AlreadyFinished {
        /// What it was aimed at.
        target: CancelTarget,
        /// The state it was already in.
        state: TerminalState,
    },
}

impl CancelOutcome {
    /// The cancel was delivered but is not confirmed.
    #[must_use]
    pub fn requested(route: CancelRoute, target: CancelTarget) -> Self {
        Self::Requested { route, target }
    }

    /// Whether the harness confirmed the work has stopped.
    ///
    /// False for [`Self::Requested`] — that is the whole point — and false for
    /// [`Self::NotFound`], which confirms nothing about anything. True for
    /// [`Self::AlreadyFinished`] only when the state it found was
    /// [`TerminalState::Cancelled`]: work that had already *completed* was
    /// never cancelled, and saying so would be the same lie in a different
    /// place.
    #[must_use]
    pub const fn is_confirmed(&self) -> bool {
        match self {
            Self::Confirmed { .. } => true,
            Self::AlreadyFinished { state, .. } => matches!(state, TerminalState::Cancelled),
            Self::Requested { .. } | Self::NotFound => false,
        }
    }

    /// Promote a delivered cancel to a confirmed one.
    ///
    /// The only legal edge in this state machine. `confirmed_at` is an ISO-8601
    /// instant supplied by the caller.
    ///
    /// # Errors
    ///
    /// [`CancelConfirmationError`] from any state but [`Self::Requested`]:
    /// confirming a `NotFound` invents work that never existed, confirming an
    /// `AlreadyFinished` rewrites how work ended, and confirming a `Confirmed`
    /// twice means two different instants both claim to be *the* confirmation.
    pub fn confirm(self, confirmed_at: &str) -> Result<Self, CancelConfirmationError> {
        match self {
            Self::Requested { route, target } => Ok(Self::Confirmed {
                route,
                target,
                confirmed_at: confirmed_at.to_owned(),
            }),
            other => Err(CancelConfirmationError {
                state: other.status_str(),
            }),
        }
    }

    /// The Work status this outcome projects to, or `None` when it names no
    /// Work.
    ///
    /// The mapping is the contract: `Requested` is
    /// [`WorkStatus::Cancelling`] and there is no path from it to
    /// [`WorkStatus::Cancelled`] that does not go through [`Self::confirm`].
    #[must_use]
    pub const fn work_status(&self) -> Option<WorkStatus> {
        match self {
            Self::Requested { .. } => Some(WorkStatus::Cancelling),
            Self::Confirmed { .. } => Some(WorkStatus::Cancelled),
            Self::AlreadyFinished { state, .. } => Some(state.work_status()),
            Self::NotFound => None,
        }
    }

    /// The `status` string this outcome reports.
    ///
    /// [`STATUS_NOT_FOUND`] for [`Self::NotFound`]; otherwise the wire spelling
    /// of [`Self::work_status`].
    ///
    /// # The deliberate divergence
    ///
    /// Upstream answers `cancelled` here for a cancel it has only *sent*,
    /// because it wrote `record.status = 'cancelled'` before awaiting anything.
    /// VIA answers `cancelling` until the harness confirms. Recorded in
    /// `docs/deviations/phase-2.md`.
    #[must_use]
    pub fn status_str(&self) -> &'static str {
        self.work_status()
            .map_or(STATUS_NOT_FOUND, WorkStatus::as_str)
    }

    /// What the cancel was aimed at, when it named anything.
    #[must_use]
    pub const fn target(&self) -> Option<&CancelTarget> {
        match self {
            Self::Requested { target, .. }
            | Self::Confirmed { target, .. }
            | Self::AlreadyFinished { target, .. } => Some(target),
            Self::NotFound => None,
        }
    }

    /// How the cancel was delivered, when it was delivered at all.
    #[must_use]
    pub const fn route(&self) -> Option<CancelRoute> {
        match self {
            Self::Requested { route, .. } | Self::Confirmed { route, .. } => Some(*route),
            Self::NotFound | Self::AlreadyFinished { .. } => None,
        }
    }

    /// The instant the harness confirmed, for a confirmed outcome.
    #[must_use]
    pub fn confirmed_at(&self) -> Option<&str> {
        match self {
            Self::Confirmed { confirmed_at, .. } => Some(confirmed_at),
            _ => None,
        }
    }
}

impl Serialize for CancelOutcome {
    /// `{ status, delegation_id, session_id }`, or a bare `{ status:
    /// 'not_found' }`.
    ///
    /// `docs/reference/contracts.json` (`json-field` / *session_cancel
    /// result*). Field order is the catalogued order.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.target() {
            None => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("status", self.status_str())?;
                map.end()
            }
            Some(target) => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("status", self.status_str())?;
                map.serialize_entry("delegation_id", &target.delegation_id)?;
                map.serialize_entry("session_id", &target.session_id)?;
                map.end()
            }
        }
    }
}

/// A cancellation was confirmed from a state that cannot be confirmed.
///
/// Developer-facing, never localized, and deliberately not a
/// [`HarnessError`](crate::HarnessError): no operator can act on it and no
/// client should see it. It is the same class as
/// [`via_protocol::ProtocolError::IllegalTransition`] — a wiring bug caught by
/// a type rather than by a support ticket.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("a cancellation reported as `{state}` cannot be confirmed; only `cancelling` can")]
pub struct CancelConfirmationError {
    /// The status the outcome was in when confirmation was attempted.
    pub state: &'static str,
}

impl CancelConfirmationError {
    /// A stable machine-readable code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        "VIA_HARNESS_CANCEL_NOT_AWAITED"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn requested() -> CancelOutcome {
        CancelOutcome::requested(CancelRoute::Adapter, CancelTarget::delegation("d-1"))
    }

    #[test]
    fn a_delivered_cancel_is_not_a_cancelled_one() {
        let outcome = requested();
        assert!(!outcome.is_confirmed());
        assert_eq!(outcome.work_status(), Some(WorkStatus::Cancelling));
        assert_eq!(outcome.status_str(), "cancelling");
        assert_eq!(outcome.confirmed_at(), None);
    }

    #[test]
    fn confirming_a_delivered_cancel_is_the_only_legal_edge() {
        let outcome = requested()
            .confirm("2026-08-22T00:00:00.000Z")
            .expect("legal");
        assert!(outcome.is_confirmed());
        assert_eq!(outcome.work_status(), Some(WorkStatus::Cancelled));
        assert_eq!(outcome.confirmed_at(), Some("2026-08-22T00:00:00.000Z"));
    }

    #[test]
    fn nothing_else_can_be_confirmed() {
        for (outcome, state) in [
            (CancelOutcome::NotFound, "not_found"),
            (
                CancelOutcome::AlreadyFinished {
                    target: CancelTarget::delegation("d-1"),
                    state: TerminalState::Completed,
                },
                "completed",
            ),
            (requested().confirm("t").expect("legal"), "cancelled"),
        ] {
            let error = outcome.confirm("t").expect_err("illegal");
            assert_eq!(error.state, state);
            assert_eq!(error.code(), "VIA_HARNESS_CANCEL_NOT_AWAITED");
        }
    }

    #[test]
    fn already_completed_work_was_never_cancelled() {
        let outcome = CancelOutcome::AlreadyFinished {
            target: CancelTarget::session("s-1"),
            state: TerminalState::Completed,
        };
        assert!(!outcome.is_confirmed());
        let already = CancelOutcome::AlreadyFinished {
            target: CancelTarget::session("s-1"),
            state: TerminalState::Cancelled,
        };
        assert!(already.is_confirmed());
    }

    #[test]
    fn delegation_scope_needs_the_delegation_capability() {
        let caps = BackendCapabilities {
            delegation: false,
            permissions: true,
            backend_ui: false,
            native_session_history: true,
            external_mcp: false,
            native_delegation: false,
            session_mcp: false,
        };
        assert!(CancelScope::Turn.is_supported_by(caps));
        assert!(CancelScope::Session.is_supported_by(caps));
        assert!(
            !CancelScope::Delegation {
                delegation_id: "d-1".to_owned(),
            }
            .is_supported_by(caps)
        );
    }
}
