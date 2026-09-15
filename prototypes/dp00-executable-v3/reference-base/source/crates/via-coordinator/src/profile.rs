//! What the coordinator needs to know about the backend, as data.
//!
//! Upstream reads these off `AcpBackendAdapter`'s constructor options and its
//! `profile` object (`server/src/agent/acp-backend-adapter.mjs:133-232`). Here
//! they are one struct, supplied by whoever resolved the backend — which is
//! `via-backends`, the one crate allowed to name one.
//!
//! # Why the instruction sentences arrive as data
//!
//! Three shipped profiles override the default session-instructions paragraph
//! and one overrides both control sentences, and every one of those overrides
//! names a backend's own tools. `docs/architecture.md` §9 forbids a generic
//! crate from binding to the named-backend registry, so this crate takes the
//! *rendered sentence* and never asks which backend it came from.
//! [`CoordinatorProfile::default_for`] renders the defaults for a caller that
//! has no override.

use std::time::Duration;

use via_core::Config;
use via_i18n::{Locale, format, keys};

use crate::permission::PermissionMode;

/// The request timeout a coordinator turn gets when the profile names none.
///
/// **External contract** — `acp-backend-adapter.mjs:139`
/// (`timeoutMs = 300_000`), catalogued under *timeouts and limits*. It is the
/// same number as [`via_core::Config::agent_timeout_ms`]'s default, which is
/// where a configured Gateway reads it from.
pub const DEFAULT_TIMEOUT_MS: u64 = 300_000;

/// Everything about one backend the coordination layer reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoordinatorProfile {
    /// The backend id. The first segment of every
    /// [`SessionKey`](via_downstream::SessionKey), and the `protocol` on every
    /// result envelope.
    pub protocol: String,
    /// The human label. It reaches the operator inside a dozen refusals.
    pub label: String,
    /// The coordinator session's working directory.
    pub directory: String,
    /// Whether VIA asks about permissions or the backend owns them.
    pub permission_mode: PermissionMode,
    /// The paragraph appended to
    /// [`BACKEND_AGENT_INSTRUCTIONS`](crate::instructions::BACKEND_AGENT_INSTRUCTIONS).
    pub session_instructions: String,
    /// The sentence a cancel control turn carries, or `None` for the default.
    pub cancel_instruction: Option<String>,
    /// The sentence a status control turn carries, or `None` for the default.
    pub status_instruction: Option<String>,
    /// The title a delegated task gets when the backend named none.
    pub delegation_title: String,
    /// Whether this backend delegates through its own session tools rather than
    /// VIA's MCP ones. Gates [`crate::native`] entirely: upstream's
    /// `if (!this.profile.nativeDelegation) return`
    /// (`acp-backend-adapter.mjs:655`).
    pub native_delegation: bool,
    /// The per-turn deadline. `None` lets the caller own it, which is what a
    /// delegated project prompt gets (`timeoutMs: 0`).
    pub timeout: Option<Duration>,
}

impl CoordinatorProfile {
    /// A profile with VIA's default instructions for a backend that reaches
    /// Layer 3 through the five MCP session tools.
    ///
    /// The session paragraph is
    /// [`via_mcp_tools::DEFAULT_SESSION_INSTRUCTIONS`] and the delegation title
    /// is [`via_i18n::keys::ACP_PROJECT_TASK_LABEL`] — upstream's
    /// `` `${this.label} 项目任务` ``.
    #[must_use]
    pub fn default_for(protocol: &str, label: &str, locale: Locale) -> Self {
        Self {
            protocol: protocol.to_owned(),
            label: label.to_owned(),
            directory: String::new(),
            permission_mode: PermissionMode::Native,
            session_instructions: via_mcp_tools::DEFAULT_SESSION_INSTRUCTIONS.to_owned(),
            cancel_instruction: None,
            status_instruction: None,
            delegation_title: format(locale, keys::ACP_PROJECT_TASK_LABEL, &[("label", label)]),
            native_delegation: false,
            timeout: Some(Duration::from_millis(DEFAULT_TIMEOUT_MS)),
        }
    }

    /// Apply the operator's configuration: the permission mode and the request
    /// timeout.
    ///
    /// The other fields are the backend's, not the operator's, and are set by
    /// whoever resolved the backend.
    #[must_use]
    pub fn configured(mut self, config: &Config) -> Self {
        self.permission_mode = PermissionMode::from_wire(&config.backend_permission_mode);
        self.timeout = u64::try_from(config.agent_timeout_ms)
            .ok()
            .filter(|millis| *millis > 0)
            .map(Duration::from_millis);
        self
    }

    /// Set the coordinator session's working directory.
    #[must_use]
    pub fn directory(mut self, directory: &str) -> Self {
        self.directory = directory.to_owned();
        self
    }

    /// Replace the session-instructions paragraph.
    #[must_use]
    pub fn session_instructions(mut self, instructions: &str) -> Self {
        self.session_instructions = instructions.to_owned();
        self
    }

    /// Replace both control sentences.
    #[must_use]
    pub fn control_instructions(mut self, cancel: &str, status: &str) -> Self {
        self.cancel_instruction = Some(cancel.to_owned());
        self.status_instruction = Some(status.to_owned());
        self
    }

    /// Declare that this backend delegates natively.
    #[must_use]
    pub const fn native_delegation(mut self, native: bool) -> Self {
        self.native_delegation = native;
        self
    }

    /// The per-turn timeout in milliseconds, or `None`.
    #[must_use]
    pub fn timeout_ms(&self) -> Option<u64> {
        self.timeout
            .map(|timeout| u64::try_from(timeout.as_millis()).unwrap_or(DEFAULT_TIMEOUT_MS))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_profile_carries_vias_own_session_instructions() {
        let profile = CoordinatorProfile::default_for("opencode", "OpenCode", Locale::Zh);
        assert_eq!(
            profile.session_instructions,
            via_mcp_tools::DEFAULT_SESSION_INSTRUCTIONS,
        );
        assert_eq!(profile.delegation_title, "OpenCode 项目任务");
        assert_eq!(profile.timeout_ms(), Some(DEFAULT_TIMEOUT_MS));
        assert!(!profile.native_delegation);
        assert_eq!(profile.permission_mode, PermissionMode::Native);
        assert_eq!(profile.cancel_instruction, None);
    }

    #[test]
    fn configuration_supplies_the_permission_mode_and_the_timeout() {
        let config = Config {
            backend_permission_mode: "full".to_owned(),
            agent_timeout_ms: 42_000,
            ..Config::default()
        };
        let profile = CoordinatorProfile::default_for("pi", "Pi", Locale::En).configured(&config);
        assert_eq!(profile.permission_mode, PermissionMode::Full);
        assert_eq!(profile.timeout_ms(), Some(42_000));
    }

    #[test]
    fn a_non_positive_timeout_means_the_caller_owns_the_deadline() {
        for millis in [0, -1] {
            let config = Config {
                agent_timeout_ms: millis,
                ..Config::default()
            };
            let profile =
                CoordinatorProfile::default_for("pi", "Pi", Locale::En).configured(&config);
            assert_eq!(profile.timeout_ms(), None, "{millis} ms");
        }
    }

    #[test]
    fn overrides_replace_the_defaults_without_naming_a_backend() {
        let profile = CoordinatorProfile::default_for("openclaw", "OpenClaw", Locale::En)
            .session_instructions("use your own tools")
            .control_instructions("stop it", "how is it")
            .native_delegation(true)
            .directory("/coordinator");
        assert_eq!(profile.session_instructions, "use your own tools");
        assert_eq!(profile.cancel_instruction.as_deref(), Some("stop it"));
        assert_eq!(profile.status_instruction.as_deref(), Some("how is it"));
        assert!(profile.native_delegation);
        assert_eq!(profile.directory, "/coordinator");
    }
}
