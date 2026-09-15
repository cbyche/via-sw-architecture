//! The Layer-3 session identity.
//!
//! Two shapes, both from `server/src/agent/acp-backend-session-utils.mjs:9-15`:
//!
//! ```js
//! export function coordinatorKey(ownerId, protocol) {
//!   return `${protocol}:${encodeURIComponent(clean(ownerId) || 'personal')}:backend`
//! }
//! export function projectSessionKey(protocol, sessionId) {
//!   return `${protocol}:${clean(sessionId)}`
//! }
//! ```
//!
//! `docs/reference/contracts.json` (`state-name` / *coordinator session key
//! format*) calls the coordinator variant **THE fixed identity that survives
//! voice sessions, Work IDs, and Gateway restarts**: it is the in-memory map
//! key, the serial-executor lock key prefix, and the key persisted in
//! `state/acp-sessions.json`. Changing how it renders orphans every stored
//! session.
//!
//! Four details in that one template are load-bearing, and each has its own
//! test in `tests/session_key.rs`:
//!
//! 1. the owner id is **trimmed** (`clean`), so `" ana "` and `"ana"` are one
//!    identity;
//! 2. an empty owner id becomes the literal [`DEFAULT_OWNER_ID`], not an empty
//!    segment;
//! 3. the owner id is **percent-encoded**, so a `:` in an owner id cannot
//!    forge a segment boundary;
//! 4. it is **not** lower-cased here. `Owner` and `owner` are different
//!    coordinator sessions. (Upstream lower-cases in exactly one other place —
//!    OpenClaw's own `agent:<coordinatorAgent>:via:<owner>:backend` wire key,
//!    `server/src/agent/backends/openclaw.mjs:108-118` — which belongs to that
//!    driver, not to this seam.)

use std::fmt;

use crate::text::{clean, encode_uri_component};

/// The owner id a coordinator key falls back to when none was supplied.
///
/// `server/src/agent/acp-backend-session-utils.mjs:10` — the literal
/// `'personal'`, reached whenever `clean(ownerId)` is empty.
pub const DEFAULT_OWNER_ID: &str = "personal";

/// The trailing segment of every coordinator key.
///
/// `server/src/agent/acp-backend-session-utils.mjs:10`.
pub const COORDINATOR_KEY_SUFFIX: &str = "backend";

/// The separator between key segments.
pub const KEY_SEPARATOR: char = ':';

/// Which of the two session shapes a [`SessionKey`] names.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SessionKeyScope {
    /// The one long-lived session per owner per backend: the coordinator.
    ///
    /// Renders as `<protocol>:<encoded owner>:backend`.
    Coordinator {
        /// The owner id **after** `clean` and the [`DEFAULT_OWNER_ID`]
        /// fallback, before percent-encoding.
        owner_id: String,
    },
    /// One delegated project session, addressed by the id the backend gave it.
    ///
    /// Renders as `<protocol>:<session id>`.
    Project {
        /// The backend's session id, after `clean`.
        session_id: String,
    },
}

/// A Layer-3 session identity, rendered once at construction.
///
/// The rendered string is cached rather than recomputed because it is used as a
/// map key and a lock key on hot paths, and because caching removes any chance
/// of two call sites rendering the same key two ways.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionKey {
    protocol: String,
    scope: SessionKeyScope,
    rendered: String,
}

impl SessionKey {
    /// `coordinatorKey(ownerId, protocol)`.
    ///
    /// Total: every input produces a key. Upstream applies no validation here
    /// either — the protocol has already been through the backend catalog by
    /// the time a coordinator key is built, and
    /// [`HarnessRegistry`](crate::HarnessRegistry) is what enforces that.
    #[must_use]
    pub fn coordinator(protocol: &str, owner_id: &str) -> Self {
        let owner = clean(owner_id);
        let owner = if owner.is_empty() {
            DEFAULT_OWNER_ID
        } else {
            owner
        };
        let rendered = format!(
            "{protocol}{KEY_SEPARATOR}{encoded}{KEY_SEPARATOR}{COORDINATOR_KEY_SUFFIX}",
            encoded = encode_uri_component(owner),
        );
        Self {
            protocol: protocol.to_owned(),
            scope: SessionKeyScope::Coordinator {
                owner_id: owner.to_owned(),
            },
            rendered,
        }
    }

    /// `projectSessionKey(protocol, sessionId)`.
    ///
    /// Note the asymmetry with [`Self::coordinator`], which is upstream's and
    /// is deliberate: a project session id is the backend's own opaque handle
    /// and is **not** percent-encoded, because the key is only ever used to
    /// look the session back up locally, never to reconstruct the id.
    #[must_use]
    pub fn project(protocol: &str, session_id: &str) -> Self {
        let session_id = clean(session_id);
        let rendered = format!("{protocol}{KEY_SEPARATOR}{session_id}");
        Self {
            protocol: protocol.to_owned(),
            scope: SessionKeyScope::Project {
                session_id: session_id.to_owned(),
            },
            rendered,
        }
    }

    /// The rendered key — the string that goes into maps, locks and storage.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.rendered
    }

    /// The backend protocol id this key is scoped to.
    #[must_use]
    pub fn protocol(&self) -> &str {
        &self.protocol
    }

    /// Which shape this is.
    #[must_use]
    pub fn scope(&self) -> &SessionKeyScope {
        &self.scope
    }

    /// Whether this is the per-owner coordinator session.
    #[must_use]
    pub fn is_coordinator(&self) -> bool {
        matches!(self.scope, SessionKeyScope::Coordinator { .. })
    }

    /// The owner id, for a coordinator key — already trimmed and defaulted.
    #[must_use]
    pub fn owner_id(&self) -> Option<&str> {
        match &self.scope {
            SessionKeyScope::Coordinator { owner_id } => Some(owner_id),
            SessionKeyScope::Project { .. } => None,
        }
    }

    /// The backend's session id, for a project key.
    #[must_use]
    pub fn session_id(&self) -> Option<&str> {
        match &self.scope {
            SessionKeyScope::Project { session_id } => Some(session_id),
            SessionKeyScope::Coordinator { .. } => None,
        }
    }
}

impl fmt::Display for SessionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.rendered)
    }
}

impl serde::Serialize for SessionKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.rendered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinator_key_renders_the_documented_examples() {
        assert_eq!(
            SessionKey::coordinator("openclaw", "owner one").as_str(),
            "openclaw:owner%20one:backend",
        );
        assert_eq!(
            SessionKey::coordinator("opencode", "").as_str(),
            "opencode:personal:backend",
        );
    }

    #[test]
    fn a_colon_in_an_owner_id_cannot_forge_a_segment() {
        let key = SessionKey::coordinator("acp", "a:backend:b");
        assert_eq!(key.as_str(), "acp:a%3Abackend%3Ab:backend");
        assert_eq!(key.as_str().split(KEY_SEPARATOR).count(), 3);
    }

    #[test]
    fn project_key_is_not_encoded() {
        assert_eq!(
            SessionKey::project("codex", "  sess-1  ").as_str(),
            "codex:sess-1",
        );
    }
}
