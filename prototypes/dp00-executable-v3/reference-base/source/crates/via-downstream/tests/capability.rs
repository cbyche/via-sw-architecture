//! The capability contract, against the twelve declarations upstream ships.
//!
//! The rules in [`BackendCapabilities::faults`] and the two catalog
//! cross-checks in [`HarnessDescriptor::declare`] were read off those twelve.
//! This file is what keeps that claim true: every upstream declaration must
//! pass, and every rule must reject something.

mod common;

use common::upstream_declarations;
use pretty_assertions::assert_eq;
use rstest::rstest;
use via_downstream::{BackendCapabilities, CapabilityFault, HarnessDescriptor, HarnessError};

/// The declaration shape ten of the twelve share.
const MCP_SESSION: BackendCapabilities = BackendCapabilities {
    delegation: true,
    permissions: true,
    backend_ui: false,
    native_session_history: true,
    external_mcp: true,
    native_delegation: false,
    session_mcp: true,
};

/// Every backend upstream ships declares its capabilities, and every one of
/// those declarations passes every rule VIA validates.
///
/// This is the test that makes the rules a *port* rather than an invention: a
/// rule no upstream driver satisfies would fail here.
#[test]
fn every_upstream_declaration_is_accepted() {
    for (id, capabilities) in upstream_declarations() {
        let definition = via_catalog::backend_definition(id)
            .unwrap_or_else(|| panic!("{id} is not in the backend catalog"));
        let descriptor = HarnessDescriptor::declare(id, definition.label, capabilities)
            .unwrap_or_else(|error| panic!("{id} was refused: {error}"));
        assert_eq!(descriptor.capabilities(), capabilities);
        assert!(capabilities.is_consistent(), "{id}");
    }
}

/// `delegation` is the union of its two transports, in every upstream
/// declaration. Flipping either side of the equation must break it.
#[rstest]
#[case::claims_delegation_with_no_transport(
    BackendCapabilities { session_mcp: false, external_mcp: false, ..MCP_SESSION },
    CapabilityFault::DelegationTransportMismatch,
)]
#[case::hides_delegation_while_declaring_a_transport(
    BackendCapabilities { delegation: false, ..MCP_SESSION },
    CapabilityFault::DelegationTransportMismatch,
)]
#[case::declares_both_transports(
    BackendCapabilities { native_delegation: true, ..MCP_SESSION },
    CapabilityFault::DelegationTransportConflict,
)]
#[case::session_tools_without_an_mcp_connection(
    BackendCapabilities { external_mcp: false, ..MCP_SESSION },
    CapabilityFault::SessionMcpWithoutExternalMcp,
)]
fn each_self_consistency_rule_rejects_something(
    #[case] capabilities: BackendCapabilities,
    #[case] expected: CapabilityFault,
) {
    assert!(
        capabilities.faults().contains(&expected),
        "{capabilities:?} did not raise {expected}",
    );
    let error = HarnessDescriptor::declare("codex", "Codex", capabilities)
        .expect_err("an inconsistent declaration is refused");
    match error {
        HarnessError::IncompleteCapabilities { id, faults } => {
            assert_eq!(id, "codex");
            assert!(faults.contains(&expected), "{faults:?}");
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

/// `permissions` is the negation of the catalog's `alwaysFullPermission`, in
/// both directions.
///
/// Pi is the one backend upstream marks as having no approval gate, and the one
/// driver that declares `permissions: false`. Swapping either half is a fault.
#[test]
fn permissions_must_agree_with_the_catalog() {
    // Pi, correctly declaring that it never asks.
    let pi = BackendCapabilities {
        delegation: false,
        permissions: false,
        backend_ui: false,
        native_session_history: true,
        external_mcp: false,
        native_delegation: false,
        session_mcp: false,
    };
    assert!(HarnessDescriptor::declare("pi", "Pi", pi).is_ok());

    // Pi, claiming a permission gate the catalog says it does not have.
    let lying = BackendCapabilities {
        permissions: true,
        ..pi
    };
    assert_faults(
        HarnessDescriptor::declare("pi", "Pi", lying),
        &[CapabilityFault::PermissionsContradictCatalog],
    );

    // Codex, disclaiming a permission gate the catalog says it has.
    let disclaiming = BackendCapabilities {
        permissions: false,
        ..MCP_SESSION
    };
    assert_faults(
        HarnessDescriptor::declare("codex", "Codex", disclaiming),
        &[CapabilityFault::PermissionsContradictCatalog],
    );
}

/// `backendUi` is exactly "the catalog gives this backend a base URL".
///
/// OpenCode and OpenClaw are the two with one, and the two that declare it.
#[test]
fn a_web_ui_claim_must_agree_with_the_catalog() {
    let with_ui = BackendCapabilities {
        backend_ui: true,
        ..MCP_SESSION
    };
    assert!(HarnessDescriptor::declare("opencode", "OpenCode", with_ui).is_ok());

    assert_faults(
        HarnessDescriptor::declare("opencode", "OpenCode", MCP_SESSION),
        &[CapabilityFault::BackendUiContradictsCatalog],
    );
    assert_faults(
        HarnessDescriptor::declare("claude", "Claude Code", with_ui),
        &[CapabilityFault::BackendUiContradictsCatalog],
    );
}

/// A declaration wrong in several ways reports all of them, so a driver author
/// fixes one declaration once.
#[test]
fn every_fault_is_reported_at_once() {
    let broken = BackendCapabilities {
        delegation: false,
        permissions: false,
        backend_ui: true,
        native_session_history: false,
        external_mcp: false,
        native_delegation: true,
        session_mcp: true,
    };
    assert_faults(
        HarnessDescriptor::declare("codex", "Codex", broken),
        &[
            CapabilityFault::DelegationTransportMismatch,
            CapabilityFault::DelegationTransportConflict,
            CapabilityFault::SessionMcpWithoutExternalMcp,
            CapabilityFault::PermissionsContradictCatalog,
            CapabilityFault::BackendUiContradictsCatalog,
        ],
    );
}

/// The flag lookup is exact, and covers all seven.
#[test]
fn every_flag_is_reachable_by_its_upstream_name() {
    for (name, value) in MCP_SESSION.flags() {
        assert_eq!(MCP_SESSION.flag(name), Some(value));
    }
    for absent in ["", "delegation ", "Delegation", "sessionMCP", "unknown"] {
        assert_eq!(MCP_SESSION.flag(absent), None, "{absent} matched");
    }
}

/// Each fault has a distinct machine-readable name — a discriminant that
/// collided would make the `code`-plus-reason pair useless in a log.
#[test]
fn fault_names_are_distinct() {
    let names = [
        CapabilityFault::DelegationTransportMismatch,
        CapabilityFault::DelegationTransportConflict,
        CapabilityFault::SessionMcpWithoutExternalMcp,
        CapabilityFault::PermissionsContradictCatalog,
        CapabilityFault::BackendUiContradictsCatalog,
    ]
    .map(CapabilityFault::as_str);
    let mut unique = names.to_vec();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), names.len());
}

fn assert_faults(result: Result<HarnessDescriptor, HarnessError>, expected: &[CapabilityFault]) {
    match result.expect_err("the declaration is inconsistent") {
        HarnessError::IncompleteCapabilities { faults, .. } => {
            assert_eq!(faults, expected.to_vec());
        }
        other => panic!("wrong variant: {other:?}"),
    }
}
