//! The one thing that can go wrong with the manager itself.
//!
//! Everything a *Work* can fail at is a [`RunFailure`](crate::RunFailure) — an
//! already-localized sentence that becomes `task.error`. This is the other
//! case: the manager is not there to ask.
//!
//! It is developer-facing and deliberately not localized, for the reason
//! `docs/architecture.md` §16 gives — *"error codes are not localized, only the
//! message beside them"* — and because no operator can act on it: a stopped
//! manager means the Gateway is shutting down.

/// The Work manager could not be reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the Work manager has stopped; no new Work can be accepted")]
pub struct ManagerStopped;

impl ManagerStopped {
    /// A stable machine-readable code.
    ///
    /// The `VIA_WORK_` prefix rather than a bare `VIA_` one, so it is never
    /// mistaken for an inherited upstream contract — the same convention
    /// `via-protocol` used for its two invented codes.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        "VIA_WORK_MANAGER_STOPPED"
    }
}

#[cfg(test)]
mod tests {
    use super::ManagerStopped;

    #[test]
    fn the_code_is_namespaced_and_the_message_is_developer_facing() {
        assert_eq!(ManagerStopped.code(), "VIA_WORK_MANAGER_STOPPED");
        assert!(ManagerStopped.to_string().contains("Work manager"));
    }
}
