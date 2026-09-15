//! What a harness says it is, checked before anyone believes it.
//!
//! Upstream's `validateBackendDriver`
//! (`server/src/agent/backends/registry.mjs:22-45`) runs at module load, over a
//! hard-coded list, and throws before the process is up. `docs/architecture.md`
//! §6 is explicit about why that matters: *"a descriptor that is incomplete or
//! internally inconsistent is rejected at startup, not discovered mid-turn.
//! This is qwen's backend-driver contract, and it is worth porting exactly."*
//!
//! VIA's registration is data rather than a hard-coded list, so the check moves
//! from module load to [`HarnessDescriptor::declare`] — which is the **only**
//! constructor. A `HarnessDescriptor` that exists has passed every rule, so a
//! harness cannot hand the dispatcher an unvalidated one however it is written.
//!
//! # The rules, in upstream's order
//!
//! | Upstream | Here |
//! | --- | --- |
//! | `backendDefinition(driver.id)` is null | [`HarnessError::DriverNotRegistered`] |
//! | `validateBackendSkillsSpec(definition)` | compile-time: [`via_catalog::SkillsSpec`] has no "unset" |
//! | `driver.label !== definition.label` | [`HarnessError::DriverLabelMismatch`] |
//! | `typeof driver.createProfile !== 'function'` | compile-time: [`DownstreamAgent`](crate::DownstreamAgent) requires `open` |
//! | any of the seven flags is not a boolean | compile-time: seven `bool` fields |
//! | a stray non-boolean capability | compile-time: `deny_unknown_fields` |
//! | — *(VIA)* | [`HarnessError::IncompleteCapabilities`], see [`CapabilityFault`] |

use serde::Serialize;
use serde::ser::SerializeMap;
use via_catalog::{BackendDefinition, backend_definition, normalize_backend_protocol};

use crate::capability::{BackendCapabilities, CapabilityFault};
use crate::error::HarnessError;

/// A validated harness identity: catalog entry plus capability declaration.
///
/// `Copy`, because everything in it is: the definition is a `&'static`
/// reference into `via-catalog`'s table and the capabilities are seven bools.
/// That is upstream's `Object.freeze` (`registry.mjs:83`) expressed as a type —
/// nothing can mutate a descriptor after it has been validated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HarnessDescriptor {
    definition: &'static BackendDefinition,
    capabilities: BackendCapabilities,
}

impl HarnessDescriptor {
    /// Declare a harness, and validate the declaration.
    ///
    /// `id` is normalised the way the backend catalog normalises a protocol —
    /// trimmed and lower-cased, with `none` meaning *no backend*
    /// (`shared/backend-catalog.mjs:480-483`) — so `" Codex "` resolves to the
    /// `codex` definition and `none` resolves to nothing.
    ///
    /// # Errors
    ///
    /// - [`HarnessError::DriverNotRegistered`] — `id` is not one of the twelve
    ///   catalogued backends.
    /// - [`HarnessError::DriverLabelMismatch`] — `label` is not the catalog's
    ///   label for that backend.
    /// - [`HarnessError::IncompleteCapabilities`] — the declaration
    ///   contradicts itself or the catalog; the error lists every rule that
    ///   failed, not just the first.
    pub fn declare(
        id: &str,
        label: &str,
        capabilities: BackendCapabilities,
    ) -> Result<Self, HarnessError> {
        let Some(definition) = backend_definition(id) else {
            return Err(HarnessError::DriverNotRegistered {
                id: normalize_backend_protocol(id),
            });
        };
        if label != definition.label {
            return Err(HarnessError::DriverLabelMismatch {
                id: definition.id.to_owned(),
                declared: label.to_owned(),
                expected: definition.label,
            });
        }
        let faults = capability_faults(definition, capabilities);
        if !faults.is_empty() {
            return Err(HarnessError::IncompleteCapabilities {
                id: definition.id.to_owned(),
                faults,
            });
        }
        Ok(Self {
            definition,
            capabilities,
        })
    }

    /// The catalogued backend id — the key a harness is registered and
    /// configured under.
    #[must_use]
    pub const fn id(&self) -> &'static str {
        self.definition.id
    }

    /// The catalogued label, interpolated into every operator-facing message
    /// about this backend.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        self.definition.label
    }

    /// The seven flags.
    #[must_use]
    pub const fn capabilities(&self) -> BackendCapabilities {
        self.capabilities
    }

    /// The whole catalog entry behind this harness.
    #[must_use]
    pub const fn definition(&self) -> &'static BackendDefinition {
        self.definition
    }
}

/// Every capability rule that fails for this declaration.
///
/// The three self-consistency rules come from [`BackendCapabilities::faults`];
/// the two below need the catalog and so live here.
fn capability_faults(
    definition: &BackendDefinition,
    capabilities: BackendCapabilities,
) -> Vec<CapabilityFault> {
    let mut faults = capabilities.faults();
    if capabilities.permissions == definition.always_full_permission {
        faults.push(CapabilityFault::PermissionsContradictCatalog);
    }
    if capabilities.backend_ui != definition.default_base_url.is_some() {
        faults.push(CapabilityFault::BackendUiContradictsCatalog);
    }
    faults
}

impl Serialize for HarnessDescriptor {
    /// `{ id, label, capabilities }`, in that order.
    ///
    /// The subset of upstream's `describe()`
    /// (`server/src/agent/acp-backend-adapter.mjs:254-272`) that belongs to the
    /// seam rather than to a transport: `baseUrl`, `transport`, `acpConnection`
    /// and the rest are `via-acp`'s to add.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("id", self.definition.id)?;
        map.serialize_entry("label", self.definition.label)?;
        map.serialize_entry("capabilities", &self.capabilities)?;
        map.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The declaration every MCP-session backend shares, e.g.
    /// `server/src/agent/backends/codex.mjs:9-17`.
    const SESSION_MCP: BackendCapabilities = BackendCapabilities {
        delegation: true,
        permissions: true,
        backend_ui: false,
        native_session_history: true,
        external_mcp: true,
        native_delegation: false,
        session_mcp: true,
    };

    #[test]
    fn a_catalogued_backend_declares_successfully() {
        let descriptor = HarnessDescriptor::declare("codex", "Codex", SESSION_MCP)
            .expect("codex is in the catalog");
        assert_eq!(descriptor.id(), "codex");
        assert_eq!(descriptor.label(), "Codex");
        assert_eq!(descriptor.capabilities(), SESSION_MCP);
    }

    #[test]
    fn the_id_is_normalised_the_way_the_catalog_normalises_it() {
        let descriptor =
            HarnessDescriptor::declare("  CODEX  ", "Codex", SESSION_MCP).expect("normalised");
        assert_eq!(descriptor.id(), "codex");
    }

    #[test]
    fn none_is_not_a_backend() {
        let error = HarnessDescriptor::declare("none", "Codex", SESSION_MCP)
            .expect_err("`none` means no backend");
        assert!(matches!(error, HarnessError::DriverNotRegistered { .. }));
    }

    #[test]
    fn a_wrong_label_is_refused() {
        let error = HarnessDescriptor::declare("codex", "codex", SESSION_MCP)
            .expect_err("the catalog spells it `Codex`");
        match error {
            HarnessError::DriverLabelMismatch {
                declared, expected, ..
            } => {
                assert_eq!(declared, "codex");
                assert_eq!(expected, "Codex");
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn a_web_ui_claim_without_a_base_url_is_refused() {
        let error = HarnessDescriptor::declare(
            "codex",
            "Codex",
            BackendCapabilities {
                backend_ui: true,
                ..SESSION_MCP
            },
        )
        .expect_err("codex has no base URL to redirect to");
        match error {
            HarnessError::IncompleteCapabilities { faults, .. } => {
                assert_eq!(faults, vec![CapabilityFault::BackendUiContradictsCatalog]);
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }
}
