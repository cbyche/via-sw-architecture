//! The per-session backend-permission policy.
//!
//! Ported from `server/src/voice/session-permission-policy.mjs`.
//!
//! When the user says "yes, and stop asking", `always` is recorded here and
//! every later permission request in **that session** is auto-approved without
//! reaching the model or the user. The scope is deliberately narrow — one
//! owner, one session — so a decision made in one voice conversation does not
//! silently pre-approve a different one.
//!
//! Both bounds exist for the same reason: this is an *authorization relaxation*
//! held in memory, so it must decay. Six hours outlives any plausible session
//! and no laptop lid.

use std::collections::HashMap;

/// How many `(owner, session)` pairs are remembered.
///
/// **External contract** — `session-permission-policy.mjs:6`
/// (`maxSessions = 500`).
pub const DEFAULT_MAX_SESSIONS: usize = 500;

/// How long a recorded mode survives without being touched.
///
/// **External contract** — `session-permission-policy.mjs:7`
/// (`ttlMs = 6 * 60 * 60 * 1000`).
pub const DEFAULT_TTL_MS: i64 = 6 * 60 * 60 * 1_000;

/// The session id used when a client did not name one.
///
/// **External contract** — `session-permission-policy.mjs:2`
/// (`sessionId || 'main'`).
pub const DEFAULT_SESSION_ID: &str = "main";

/// What this session does with a backend permission request.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PermissionMode {
    /// Relay the request to the model, which asks the user.
    #[default]
    Ask,
    /// Approve without asking.
    AutoAllow,
}

impl PermissionMode {
    /// The stored spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::AutoAllow => "auto_allow",
        }
    }

    /// Parse a stored spelling.
    ///
    /// **External contract** — `session-permission-policy.mjs:36`: anything
    /// that is not exactly `auto_allow` normalizes to `ask`. Fail-closed, so a
    /// corrupt or unknown value never grants an approval.
    #[must_use]
    pub fn from_wire_or_ask(value: &str) -> Self {
        if value == "auto_allow" {
            Self::AutoAllow
        } else {
            Self::Ask
        }
    }
}

/// The decision the model relayed from the user.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PermissionDecision {
    /// Allow this operation, and auto-allow the rest of this session.
    Always,
    /// Refuse this operation; keep asking.
    Reject,
}

impl PermissionDecision {
    /// The wire spelling, as `respond_agent_permission.decision` sends it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Reject => "reject",
        }
    }

    /// Parse a wire spelling. Anything else is not a decision at all.
    #[must_use]
    pub fn from_wire(value: &str) -> Option<Self> {
        match value {
            "always" => Some(Self::Always),
            "reject" => Some(Self::Reject),
            _ => None,
        }
    }

    /// The mode this decision installs.
    ///
    /// **External contract** — `session-permission-policy.mjs:45-51`: only
    /// `always` relaxes; `reject` restores asking.
    #[must_use]
    pub const fn mode(self) -> PermissionMode {
        match self {
            Self::Always => PermissionMode::AutoAllow,
            Self::Reject => PermissionMode::Ask,
        }
    }
}

/// A monotonic millisecond clock.
pub type Clock = std::sync::Arc<dyn Fn() -> i64 + Send + Sync>;

#[derive(Clone, Copy, Debug)]
struct SessionState {
    mode: PermissionMode,
    updated_at: i64,
}

/// Per-session auto-allow state.
#[derive(Clone)]
pub struct SessionPermissionPolicy {
    max_sessions: usize,
    ttl_ms: i64,
    clock: Clock,
    sessions: HashMap<String, SessionState>,
}

impl std::fmt::Debug for SessionPermissionPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionPermissionPolicy")
            .field("max_sessions", &self.max_sessions)
            .field("ttl_ms", &self.ttl_ms)
            .field("sessions", &self.sessions.len())
            .finish()
    }
}

/// `${ownerId}\u0000${sessionId || 'main'}`.
///
/// **External contract** — `session-permission-policy.mjs:1-3`. NUL is the
/// separator because neither an owner id (a UUID or `user_personal`) nor a
/// session id (a URL query parameter) can contain one in practice. It is
/// **not** an injective encoding for arbitrary input, and upstream's is not
/// either — a caller that could supply a NUL inside an id would be able to
/// forge a key, which is why ids reach this function from
/// [`via_core::Identity`] and the socket's query string rather than from a
/// model.
fn key(owner_id: &str, session_id: &str) -> String {
    let session = if session_id.is_empty() {
        DEFAULT_SESSION_ID
    } else {
        session_id
    };
    format!("{owner_id}\u{0}{session}")
}

impl SessionPermissionPolicy {
    /// A policy with the contract bounds and the system clock.
    #[must_use]
    pub fn new() -> Self {
        Self::with_clock(std::sync::Arc::new(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| {
                    i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
                })
        }))
    }

    /// A policy driven by `clock`.
    #[must_use]
    pub fn with_clock(clock: Clock) -> Self {
        Self {
            max_sessions: DEFAULT_MAX_SESSIONS,
            ttl_ms: DEFAULT_TTL_MS,
            clock,
            sessions: HashMap::new(),
        }
    }

    /// Override the session cap.
    #[must_use]
    pub const fn max_sessions(mut self, max_sessions: usize) -> Self {
        self.max_sessions = max_sessions;
        self
    }

    /// Override the TTL.
    #[must_use]
    pub const fn ttl_ms(mut self, ttl_ms: i64) -> Self {
        self.ttl_ms = ttl_ms;
        self
    }

    /// Drop expired sessions, then the oldest until the cap holds.
    ///
    /// **External contract** — `session-permission-policy.mjs:17-28`. The TTL
    /// comparison is `>=`, so a session exactly at the boundary is dropped.
    pub fn prune(&mut self) {
        let now = (self.clock)();
        self.sessions
            .retain(|_, state| now.saturating_sub(state.updated_at) < self.ttl_ms);
        while self.sessions.len() > self.max_sessions {
            let oldest = self
                .sessions
                .iter()
                .min_by_key(|(id, state)| (state.updated_at, (*id).clone()))
                .map(|(id, _)| id.clone());
            let Some(oldest) = oldest else { break };
            self.sessions.remove(&oldest);
        }
    }

    /// This session's mode, defaulting to [`PermissionMode::Ask`].
    pub fn mode(&mut self, owner_id: &str, session_id: &str) -> PermissionMode {
        self.prune();
        self.sessions
            .get(&key(owner_id, session_id))
            .map_or(PermissionMode::Ask, |state| state.mode)
    }

    /// Record a mode for this session.
    pub fn set_mode(
        &mut self,
        owner_id: &str,
        session_id: &str,
        mode: PermissionMode,
    ) -> PermissionMode {
        self.sessions.insert(
            key(owner_id, session_id),
            SessionState {
                mode,
                updated_at: (self.clock)(),
            },
        );
        self.prune();
        mode
    }

    /// Apply the user's decision.
    pub fn apply_decision(
        &mut self,
        owner_id: &str,
        session_id: &str,
        decision: PermissionDecision,
    ) -> PermissionMode {
        self.set_mode(owner_id, session_id, decision.mode())
    }

    /// Whether this session auto-approves.
    pub fn should_auto_allow(&mut self, owner_id: &str, session_id: &str) -> bool {
        self.mode(owner_id, session_id) == PermissionMode::AutoAllow
    }

    /// How many sessions are remembered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    /// Whether nothing is remembered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

impl Default for SessionPermissionPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicI64, Ordering};

    fn clock() -> (Clock, Arc<AtomicI64>) {
        let now = Arc::new(AtomicI64::new(0));
        let handle = Arc::clone(&now);
        (Arc::new(move || handle.load(Ordering::SeqCst)), now)
    }

    #[test]
    fn an_unknown_session_asks() {
        let (clock, _) = clock();
        let mut policy = SessionPermissionPolicy::with_clock(clock);
        assert_eq!(policy.mode("alice", "main"), PermissionMode::Ask);
        assert!(!policy.should_auto_allow("alice", "main"));
    }

    #[test]
    fn always_relaxes_and_reject_restores() {
        let (clock, _) = clock();
        let mut policy = SessionPermissionPolicy::with_clock(clock);
        policy.apply_decision("alice", "main", PermissionDecision::Always);
        assert!(policy.should_auto_allow("alice", "main"));
        policy.apply_decision("alice", "main", PermissionDecision::Reject);
        assert!(!policy.should_auto_allow("alice", "main"));
    }

    #[test]
    fn the_relaxation_is_scoped_to_one_owner_and_one_session() {
        let (clock, _) = clock();
        let mut policy = SessionPermissionPolicy::with_clock(clock);
        policy.apply_decision("alice", "main", PermissionDecision::Always);
        assert!(!policy.should_auto_allow("alice", "other"));
        assert!(!policy.should_auto_allow("bob", "main"));
    }

    #[test]
    fn an_empty_session_id_is_main() {
        let (clock, _) = clock();
        let mut policy = SessionPermissionPolicy::with_clock(clock);
        policy.apply_decision("alice", "", PermissionDecision::Always);
        assert!(policy.should_auto_allow("alice", "main"));
        assert_eq!(policy.len(), 1);
    }

    #[test]
    fn ordinary_owner_and_session_ids_never_collide() {
        let (clock, _) = clock();
        let mut policy = SessionPermissionPolicy::with_clock(clock);
        policy.apply_decision("user_personal", "main", PermissionDecision::Always);
        for (owner, session) in [
            ("user_personal", "main2"),
            ("user_personal2", "main"),
            ("user", "personalmain"),
            ("user_personalmain", ""),
        ] {
            assert!(
                !policy.should_auto_allow(owner, session),
                "({owner}, {session}) must not inherit the relaxation",
            );
        }
    }

    #[test]
    fn a_relaxation_expires_at_the_ttl_boundary() {
        let (clock, now) = clock();
        let mut policy = SessionPermissionPolicy::with_clock(clock);
        policy.apply_decision("alice", "main", PermissionDecision::Always);
        now.store(DEFAULT_TTL_MS - 1, Ordering::SeqCst);
        assert!(policy.should_auto_allow("alice", "main"), "just inside");
        now.store(DEFAULT_TTL_MS, Ordering::SeqCst);
        assert!(
            !policy.should_auto_allow("alice", "main"),
            "exactly at the bound"
        );
        assert!(policy.is_empty(), "and pruned");
    }

    #[test]
    fn setting_a_mode_refreshes_the_ttl() {
        let (clock, now) = clock();
        let mut policy = SessionPermissionPolicy::with_clock(clock);
        policy.apply_decision("alice", "main", PermissionDecision::Always);
        now.store(DEFAULT_TTL_MS - 1, Ordering::SeqCst);
        policy.set_mode("alice", "main", PermissionMode::AutoAllow);
        now.store(DEFAULT_TTL_MS + 1, Ordering::SeqCst);
        assert!(
            policy.should_auto_allow("alice", "main"),
            "the refresh moved the deadline"
        );
    }

    #[test]
    fn the_session_cap_evicts_the_oldest() {
        let (clock, now) = clock();
        let mut policy = SessionPermissionPolicy::with_clock(clock).max_sessions(2);
        for (index, session) in ["a", "b", "c"].into_iter().enumerate() {
            now.store(i64::try_from(index).unwrap_or_default(), Ordering::SeqCst);
            policy.apply_decision("alice", session, PermissionDecision::Always);
        }
        assert_eq!(policy.len(), 2);
        assert!(!policy.should_auto_allow("alice", "a"), "the oldest went");
        assert!(policy.should_auto_allow("alice", "b"));
        assert!(policy.should_auto_allow("alice", "c"));
    }

    #[test]
    fn an_unknown_stored_mode_fails_closed() {
        assert_eq!(
            PermissionMode::from_wire_or_ask("auto_allow"),
            PermissionMode::AutoAllow
        );
        for value in ["ask", "AUTO_ALLOW", "allow", "", "true"] {
            assert_eq!(
                PermissionMode::from_wire_or_ask(value),
                PermissionMode::Ask,
                "{value} must not grant",
            );
        }
    }

    #[test]
    fn only_the_two_decisions_parse() {
        assert_eq!(
            PermissionDecision::from_wire("always"),
            Some(PermissionDecision::Always),
        );
        assert_eq!(
            PermissionDecision::from_wire("reject"),
            Some(PermissionDecision::Reject),
        );
        for value in ["allow", "deny", "yes", "", "Always"] {
            assert_eq!(PermissionDecision::from_wire(value), None, "{value}");
        }
    }
}
