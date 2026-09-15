//! The seven-flag backend capability contract.
//!
//! Upstream declares it once, in `server/src/agent/backends/registry.mjs:12-20`:
//!
//! ```js
//! const CAPABILITY_FLAGS = [
//!   'delegation', 'permissions', 'backendUi', 'nativeSessionHistory',
//!   'externalMcp', 'nativeDelegation', 'sessionMcp',
//! ]
//! ```
//!
//! and every one of the twelve drivers in `server/src/agent/backends/*.mjs`
//! answers all seven. `docs/reference/contracts.json` (`json-field` / *backend
//! agent driver capability contract*) records the list and DeepSeek's exact
//! answers, and says what they are for: they *drive which Session tools and
//! prompts are offered per backend*.
//!
//! # What "incomplete" becomes in Rust
//!
//! Upstream's check is `typeof driver.capabilities[flag] !== 'boolean'` over the
//! seven, plus `Object.values(...).some(v => typeof v !== 'boolean')` to catch a
//! *stray* key. Both faults are unrepresentable here: [`BackendCapabilities`] is
//! a struct of seven `bool`s, so a missing flag will not compile and a stray one
//! has nowhere to go. That is the *stronger* guarantee, not a dropped check.
//!
//! What survives as a runtime check is the other half of the sentence in
//! `docs/architecture.md` §6 — *"a descriptor that is incomplete or internally
//! **inconsistent** is rejected at startup, not discovered mid-turn"*. The
//! invariants below are read off the twelve upstream declarations, and
//! `tests/capability.rs` asserts every one of the twelve satisfies all of them.
//!
//! # Immutability
//!
//! `createBackendProfile` publishes `Object.freeze({ ...driver.capabilities })`
//! (`registry.mjs:83`) so a profile cannot be talked out of its own contract
//! mid-run. [`BackendCapabilities`] is `Copy` with no interior mutability and
//! [`HarnessDescriptor::capabilities`](crate::HarnessDescriptor::capabilities)
//! hands back a copy, which is the same guarantee without the runtime call.

use serde::{Deserialize, Serialize};

/// The seven flag names, in upstream's declaration order.
///
/// `server/src/agent/backends/registry.mjs:12-20`. The order is not cosmetic:
/// it is the order [`BackendCapabilities`] serializes in, and the catalogued
/// contract states the list in it.
pub const CAPABILITY_FLAGS: [&str; 7] = [
    "delegation",
    "permissions",
    "backendUi",
    "nativeSessionHistory",
    "externalMcp",
    "nativeDelegation",
    "sessionMcp",
];

/// What a harness says it can do.
///
/// Serializes to exactly the object upstream freezes onto every profile —
/// same key spellings, same order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackendCapabilities {
    /// The harness can carry out delegated project work at all — by either
    /// transport. See [`Self::session_mcp`] and [`Self::native_delegation`].
    pub delegation: bool,
    /// The harness asks before privileged operations, so VIA's permission
    /// relay has something to relay. `false` means it never asks, which is a
    /// safety disclosure rather than a setting — see
    /// [`via_catalog::BackendDefinition::always_full_permission`].
    pub permissions: bool,
    /// The harness serves a web interface of its own, which
    /// `GET /api/backend/ui` redirects to
    /// (`server/src/app/gateway-application.mjs:298-312`).
    pub backend_ui: bool,
    /// The harness keeps its own session history, so VIA does not have to
    /// replay a transcript to continue a session.
    pub native_session_history: bool,
    /// The harness accepts external MCP servers. VIA's own session tools are
    /// delivered as one, so this is what [`Self::session_mcp`] rests on.
    pub external_mcp: bool,
    /// The harness has its own sub-session mechanism and VIA drives that
    /// instead of its MCP session tools. OpenClaw is the only upstream driver
    /// that does (`server/src/agent/backends/openclaw.mjs:39-47`).
    pub native_delegation: bool,
    /// VIA's session tools are offered to this harness over MCP.
    pub session_mcp: bool,
}

impl BackendCapabilities {
    /// The seven flags as `(name, value)` pairs, in [`CAPABILITY_FLAGS`] order.
    #[must_use]
    pub const fn flags(&self) -> [(&'static str, bool); 7] {
        [
            ("delegation", self.delegation),
            ("permissions", self.permissions),
            ("backendUi", self.backend_ui),
            ("nativeSessionHistory", self.native_session_history),
            ("externalMcp", self.external_mcp),
            ("nativeDelegation", self.native_delegation),
            ("sessionMcp", self.session_mcp),
        ]
    }

    /// One flag by its upstream name, or `None` if there is no such flag.
    ///
    /// The lookup upstream writes as `driver.capabilities[flag]`. Names are
    /// matched exactly — `backendui` is not `backendUi`.
    #[must_use]
    pub fn flag(&self, name: &str) -> Option<bool> {
        self.flags()
            .into_iter()
            .find(|(flag, _)| *flag == name)
            .map(|(_, value)| value)
    }

    /// Check the declaration against itself.
    ///
    /// Returns every fault found, in a stable order, so a driver author fixes
    /// one declaration once rather than one fault per restart. An empty slice
    /// means the declaration is internally consistent.
    #[must_use]
    pub fn faults(&self) -> Vec<CapabilityFault> {
        let mut faults = Vec::new();
        if self.delegation != (self.session_mcp || self.native_delegation) {
            faults.push(CapabilityFault::DelegationTransportMismatch);
        }
        if self.session_mcp && self.native_delegation {
            faults.push(CapabilityFault::DelegationTransportConflict);
        }
        if self.session_mcp && !self.external_mcp {
            faults.push(CapabilityFault::SessionMcpWithoutExternalMcp);
        }
        faults
    }

    /// Whether [`Self::faults`] is empty.
    #[must_use]
    pub fn is_consistent(&self) -> bool {
        self.faults().is_empty()
    }
}

/// One way a capability declaration contradicts itself or the backend catalog.
///
/// Every variant renders as upstream's single capability refusal,
/// `后台 Driver 能力声明不完整：<id>` (`via_i18n::keys::BACKEND_DRIVER_INCOMPLETE_CAPABILITIES`),
/// because upstream has exactly one message for the whole capability block. The
/// discriminant is VIA's addition: it says *which* rule failed without changing
/// what the operator reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityFault {
    /// `delegation` is not the union of its two transports.
    ///
    /// All twelve upstream drivers satisfy
    /// `delegation === sessionMcp || nativeDelegation`: a harness that can
    /// delegate must say *how*, and one that declares a transport must admit it
    /// delegates. Declaring `delegation` alone would advertise Layer-3 project
    /// work with no way to start any.
    DelegationTransportMismatch,
    /// Both delegation transports are declared at once.
    ///
    /// They are alternatives, not layers: `sessionMcp` means VIA's session
    /// tools drive delegation, `nativeDelegation` means the harness's own
    /// mechanism does. Declaring both leaves the dispatcher to guess, and no
    /// upstream driver declares both.
    DelegationTransportConflict,
    /// `sessionMcp` without `externalMcp`.
    ///
    /// VIA's session tools *are* an external MCP server. A harness that will
    /// not accept one cannot be offered them.
    SessionMcpWithoutExternalMcp,
    /// `permissions` disagrees with the catalog's
    /// [`always_full_permission`](via_catalog::BackendDefinition::always_full_permission).
    ///
    /// A harness with no approval gate is permanently at full permission, and
    /// the catalog is where that is disclosed to the user
    /// (`shared/backend-catalog.mjs:470-478`). Pi is the one upstream backend
    /// on either side of this, and it is on both.
    PermissionsContradictCatalog,
    /// `backendUi` disagrees with the catalog's
    /// [`default_base_url`](via_catalog::BackendDefinition::default_base_url).
    ///
    /// The web interface `GET /api/backend/ui` redirects to is served at that
    /// base URL; a harness with no base URL has nowhere to redirect to, and a
    /// `backendUi` claim would turn a 404 into a broken link.
    BackendUiContradictsCatalog,
}

impl CapabilityFault {
    /// A stable machine-readable name for this fault.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DelegationTransportMismatch => "delegation_transport_mismatch",
            Self::DelegationTransportConflict => "delegation_transport_conflict",
            Self::SessionMcpWithoutExternalMcp => "session_mcp_without_external_mcp",
            Self::PermissionsContradictCatalog => "permissions_contradict_catalog",
            Self::BackendUiContradictsCatalog => "backend_ui_contradicts_catalog",
        }
    }
}

impl std::fmt::Display for CapabilityFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// OpenClaw's declaration, `server/src/agent/backends/openclaw.mjs:39-47`.
    const OPENCLAW: BackendCapabilities = BackendCapabilities {
        delegation: true,
        permissions: true,
        backend_ui: true,
        native_session_history: true,
        external_mcp: false,
        native_delegation: true,
        session_mcp: false,
    };

    #[test]
    fn a_real_declaration_is_consistent() {
        assert!(OPENCLAW.is_consistent(), "{:?}", OPENCLAW.faults());
    }

    #[test]
    fn flag_lookup_is_exact() {
        assert_eq!(OPENCLAW.flag("nativeDelegation"), Some(true));
        assert_eq!(OPENCLAW.flag("externalMcp"), Some(false));
        assert_eq!(OPENCLAW.flag("backendui"), None);
        assert_eq!(OPENCLAW.flag(""), None);
    }

    #[test]
    fn declaring_delegation_with_no_transport_is_a_fault() {
        let broken = BackendCapabilities {
            native_delegation: false,
            ..OPENCLAW
        };
        assert_eq!(
            broken.faults(),
            vec![CapabilityFault::DelegationTransportMismatch],
        );
    }

    #[test]
    fn declaring_both_transports_is_a_fault() {
        let broken = BackendCapabilities {
            external_mcp: true,
            session_mcp: true,
            ..OPENCLAW
        };
        assert_eq!(
            broken.faults(),
            vec![CapabilityFault::DelegationTransportConflict],
        );
    }

    #[test]
    fn session_tools_without_external_mcp_is_a_fault() {
        let broken = BackendCapabilities {
            native_delegation: false,
            session_mcp: true,
            external_mcp: false,
            ..OPENCLAW
        };
        assert_eq!(
            broken.faults(),
            vec![CapabilityFault::SessionMcpWithoutExternalMcp],
        );
    }

    #[test]
    fn every_fault_is_reported_not_just_the_first() {
        let broken = BackendCapabilities {
            delegation: false,
            permissions: true,
            backend_ui: false,
            native_session_history: false,
            external_mcp: false,
            native_delegation: true,
            session_mcp: true,
        };
        assert_eq!(
            broken.faults(),
            vec![
                CapabilityFault::DelegationTransportMismatch,
                CapabilityFault::DelegationTransportConflict,
                CapabilityFault::SessionMcpWithoutExternalMcp,
            ],
        );
    }
}
