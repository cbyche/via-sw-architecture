//! The twelve capability declarations, reproduced from their drivers.
//!
//! Upstream declares the seven flags once per driver object
//! (`server/src/agent/backends/*.mjs`), freezes the result onto every profile
//! (`registry.mjs:83`) and asserts them at startup. `via-downstream` owns the
//! *contract* — the flag names, the consistency rules and the refusal; this
//! module owns the *values*, because they name backends and only this crate may.
//!
//! # Why they are `const` and not derived
//!
//! Four of the seven flags could be inferred from the catalog
//! (`backendUi` from `defaultBaseUrl`, `permissions` from
//! `alwaysFullPermission`), and `via-downstream` already cross-checks those two.
//! The other five cannot: `externalMcp: false` for DeepSeek and Pi records that
//! *their adapters* silently drop `mcpServers`, and no catalog field says so.
//! Inferring some and declaring others would hide which is which, so all seven
//! are declared, exactly as upstream declares them, and validation catches a
//! declaration that contradicts the catalog.
//!
//! `docs/reference/contracts.json` (`json-field` / *backend agent driver
//! capability contract*) pins DeepSeek's set specifically, and
//! `server/test/backend-driver-registry.test.mjs:50-66` asserts it; so does
//! `tests/capabilities.rs`.

use via_downstream::BackendCapabilities;

/// The declaration shared by every backend VIA reaches through its own ACP
/// session tools.
///
/// `server/src/agent/backends/local-acp.mjs:11-19`, and repeated verbatim by
/// `codebuddy.mjs:6-14`, `codex.mjs:11-19`, `claude.mjs:8-16` and
/// `generic-acp.mjs:9-17`.
pub const SESSION_MCP: BackendCapabilities = BackendCapabilities {
    delegation: true,
    permissions: true,
    backend_ui: false,
    native_session_history: true,
    external_mcp: true,
    native_delegation: false,
    session_mcp: true,
};

/// OpenCode — [`SESSION_MCP`] plus its own web interface.
///
/// `server/src/agent/backends/opencode.mjs:31-39`.
pub const OPENCODE: BackendCapabilities = BackendCapabilities {
    backend_ui: true,
    ..SESSION_MCP
};

/// OpenClaw — the one backend VIA delegates through the harness's *own*
/// session mechanism rather than through MCP.
///
/// `server/src/agent/backends/openclaw.mjs:39-47`.
pub const OPENCLAW: BackendCapabilities = BackendCapabilities {
    delegation: true,
    permissions: true,
    backend_ui: true,
    native_session_history: true,
    external_mcp: false,
    native_delegation: true,
    session_mcp: false,
};

/// DeepSeek Harness — the frozen set
/// `server/test/backend-driver-registry.test.mjs:50-66` asserts by value.
///
/// `server/src/agent/backends/deepseek-harness.mjs:11-19`. Its ACP adapter
/// rejects a non-empty `mcpServers`, so VIA's session tools cannot be offered
/// and there is no delegation transport at all.
pub const DEEPSEEK: BackendCapabilities = BackendCapabilities {
    delegation: false,
    permissions: true,
    backend_ui: false,
    native_session_history: false,
    external_mcp: false,
    native_delegation: false,
    session_mcp: false,
};

/// Pi — the only backend that declares `permissions: false`.
///
/// `server/src/agent/backends/pi.mjs:11-19`. Pi has no approval gate in any
/// configuration, which is why the catalog also marks it
/// [`always_full_permission`](via_catalog::BackendDefinition::always_full_permission):
/// the declaration is a safety disclosure, not a setting.
pub const PI: BackendCapabilities = BackendCapabilities {
    delegation: false,
    permissions: false,
    backend_ui: false,
    native_session_history: true,
    external_mcp: false,
    native_delegation: false,
    session_mcp: false,
};

/// The catalogued backend id → its declaration.
///
/// Returns `None` for anything that is not one of the twelve. The ids are
/// matched exactly; normalisation is the caller's, through
/// [`via_catalog::normalize_backend_protocol`].
#[must_use]
pub fn backend_capabilities(id: &str) -> Option<BackendCapabilities> {
    Some(match id {
        "opencode" => OPENCODE,
        "openclaw" => OPENCLAW,
        // qoder / qwen / kimi / hermes are `localAcpBackend(...)` instances and
        // share one declaration (`local-acp.mjs:11-19`).
        "qoder" | "qwen" | "kimi" | "hermes" | "codebuddy" | "codex" | "claude" | "acp" => {
            SESSION_MCP
        }
        "deepseek" => DEEPSEEK,
        "pi" => PI,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catalogued_backend_declares_a_consistent_set() {
        for definition in via_catalog::backend_definitions() {
            let capabilities =
                backend_capabilities(definition.id).expect("every catalogued id declares");
            assert!(
                capabilities.is_consistent(),
                "{}: {:?}",
                definition.id,
                capabilities.faults()
            );
        }
    }

    #[test]
    fn nothing_outside_the_catalog_declares() {
        assert!(backend_capabilities("none").is_none());
        assert!(backend_capabilities("").is_none());
        assert!(backend_capabilities("OPENCODE").is_none());
    }

    #[test]
    fn deepseek_is_the_frozen_set_the_registry_test_asserts() {
        assert_eq!(
            backend_capabilities("deepseek"),
            Some(BackendCapabilities {
                delegation: false,
                permissions: true,
                backend_ui: false,
                native_session_history: false,
                external_mcp: false,
                native_delegation: false,
                session_mcp: false,
            })
        );
    }
}
