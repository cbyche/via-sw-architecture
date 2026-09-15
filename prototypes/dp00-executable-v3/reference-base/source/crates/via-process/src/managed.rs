//! Resolving *which* backend the Gateway manages, and *how* it owns it.
//!
//! Ported from `server/src/process/managed-backend.mjs:1-73,110-129`.
//!
//! # Ownership is two axes, not one
//!
//! `owned` means VIA starts the process and stops it when VIA exits.
//! `external` means VIA connects to an address someone else published, and
//! never starts, moves or stops it — no port probe, no reallocation, no
//! signal. Upstream's own test says so out loud: an external Gateway
//! *"must not probe or move the port"* and *"must not spawn"*
//! (`server/test/managed-backend.test.mjs:189-198`).
//!
//! That is ownership of the **service**. How VIA reaches the backend over ACP
//! is a *separate* field —
//! [`AcpConnection`](crate::runtime_state::AcpConnection) — and an external
//! service can still be reached through a locally spawned `process` adapter.
//! Collapsing the two into one enum would make that combination
//! unrepresentable, and it is the normal shape for a bridged backend.
//!
//! # Two copies of one validation
//!
//! `env-var/QWEN_AUDIO_AGENT_BACKEND_PERMISSION_MODE` records that upstream
//! validates the permission mode in *both* `config.mjs` and
//! `managed-backend.mjs`, with identical messages, and that *"both must be
//! reproduced"*. `via-core`'s resolver has the first; [`permission_mode`] is
//! the second. They differ in one observable way, which is upstream's doing
//! and not a slip: `config.mjs` only validates when a backend is configured,
//! while this one validates unconditionally.

use via_catalog::{Ownership, effective_backend_permission_mode};
use via_core::EnvMap;
use via_core::config::names;

use crate::driver::{
    BackendRuntimeDriver, BackendRuntimeRegistry, ResolveRequest,
    normalize_backend_runtime_protocol,
};
use crate::endpoint::parse_service_endpoint;
use crate::error::ProcessError;

/// The default, and the only mode that leaves the backend's own approval
/// prompts in place.
///
/// **External contract** — `server/src/process/managed-backend.mjs:16`,
/// catalogued as *"Security-relevant default — must stay 'native'"*.
pub const PERMISSION_MODE_NATIVE: &str = "native";

/// Full permission: the Gateway turns the backend's own approvals off.
///
/// **External contract** — `server/src/process/managed-backend.mjs:19`.
pub const PERMISSION_MODE_FULL: &str = "full";

/// The two accepted permission modes, in the order the refusal lists them.
///
/// **External contract** — `不支持的后台权限模式：{mode}（可选 native、full）`.
pub const BACKEND_PERMISSION_MODES: &[&str] = &[PERMISSION_MODE_NATIVE, PERMISSION_MODE_FULL];

/// The backend the Gateway is managing, as `resolveManagedBackend` answers it.
///
/// **External contract** — `server/src/process/backend-drivers/shared.mjs:21-53`.
/// Serialized field order is upstream's object-literal order, which
/// `server/test/managed-backend.test.mjs` deep-equals against.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedBackend {
    /// The backend protocol id.
    pub protocol: String,
    /// Who owns the backend service.
    pub ownership: Ownership,
    /// The permission mode actually in effect, after
    /// [`effective_backend_permission_mode`] has had its say.
    pub permission_mode: String,
    /// The service address, or `None` for a backend with no service of its
    /// own. `None` serializes as JSON `null`, as upstream's `baseUrl: null`.
    pub base_url: Option<String>,
}

/// The configured permission mode, validated.
///
/// **External contract** — `server/src/process/managed-backend.mjs:14-22`.
/// Upstream writes `String(env.X || 'native').toLowerCase()` — lowercased but
/// **not trimmed**, so `" full "` is a refusal rather than `full`. Reproduced
/// exactly; `via-core`'s copy of this check does the same.
///
/// # Errors
///
/// [`ProcessError::UnsupportedPermissionMode`] for anything but `native` or
/// `full`.
pub fn permission_mode(env: &EnvMap) -> Result<String, ProcessError> {
    let mode = env
        .get_truthy(names::BACKEND_PERMISSION_MODE)
        .unwrap_or(PERMISSION_MODE_NATIVE)
        .to_lowercase();
    if !BACKEND_PERMISSION_MODES.contains(&mode.as_str()) {
        return Err(ProcessError::UnsupportedPermissionMode { requested: mode });
    }
    Ok(mode)
}

/// Decide who owns the backend service.
///
/// **External contract** — `server/src/process/managed-backend.mjs:24-40` and
/// `env-var/QWEN_AUDIO_AGENT_BACKEND_OWNERSHIP`. An explicit request wins if
/// the driver can honour it; otherwise a driver that both supports an external
/// service *and* has its base-URL variable set to a non-empty value defaults to
/// `external`, and everything else to `owned`.
///
/// The auto-detection is why *setting* an address flips ownership: an operator
/// who published their own service has, by publishing it, said they run it.
///
/// # Errors
///
/// - [`ProcessError::UnsupportedOwnership`] — neither empty, `owned`, nor
///   `external`.
/// - [`ProcessError::ExternalServiceUnsupported`] — `external` asked of a
///   driver the Gateway must launch itself. Upstream interpolates the driver
///   **id** at this call site, not the catalog label; see the variant docs.
pub fn backend_ownership(
    driver: &BackendRuntimeDriver,
    env: &EnvMap,
) -> Result<Ownership, ProcessError> {
    let requested = env.get_trimmed(names::BACKEND_OWNERSHIP).to_lowercase();
    if !requested.is_empty() {
        let ownership = match requested.as_str() {
            "owned" => Ownership::Owned,
            "external" => Ownership::External,
            _ => return Err(ProcessError::UnsupportedOwnership { requested }),
        };
        if ownership == Ownership::External && !driver.supports_external_service {
            return Err(ProcessError::ExternalServiceUnsupported {
                label: driver.id.clone(),
            });
        }
        return Ok(ownership);
    }
    let base_url_configured = driver
        .service
        .as_ref()
        .is_some_and(|service| !env.get_trimmed(&service.base_url_environment).is_empty());
    Ok(if driver.supports_external_service && base_url_configured {
        Ownership::External
    } else {
        Ownership::Owned
    })
}

/// Which backend the Gateway manages, if any.
///
/// **External contract** — `server/src/process/managed-backend.mjs:42-59`.
/// `None` is frontend-only operation: `AGENT_PROTOCOL` unset, empty, or the
/// `none` sentinel. That is not an error — `docs/architecture.md` §2 records
/// that `agent` mode degrades to `direct` when no harness is configured, which
/// is what makes a fresh install useful before any backend is installed.
///
/// # Errors
///
/// - [`ProcessError::UnsupportedBackend`] — an `AGENT_PROTOCOL` with no
///   catalog entry.
/// - [`ProcessError::UnsupportedPermissionMode`] / [`ProcessError::UnsupportedOwnership`]
///   / [`ProcessError::ExternalServiceUnsupported`] — from the two resolvers
///   above.
/// - [`ProcessError::FullPermissionRequiresOwned`] — `full` on a service VIA
///   does not own. The first of the two places upstream raises this.
/// - Whatever [`BackendRuntimeDriver::resolve`] refuses with.
pub fn resolve_managed_backend(
    registry: &BackendRuntimeRegistry,
    env: &EnvMap,
) -> Result<Option<ManagedBackend>, ProcessError> {
    let protocol =
        normalize_backend_runtime_protocol(env.get(names::AGENT_PROTOCOL).unwrap_or_default());
    if protocol.is_empty() {
        return Ok(None);
    }
    let driver = registry.driver(&protocol)?;
    let resolved_permission_mode =
        effective_backend_permission_mode(&protocol, &permission_mode(env)?);
    let ownership = backend_ownership(&driver, env)?;
    if resolved_permission_mode == PERMISSION_MODE_FULL && ownership != Ownership::Owned {
        return Err(ProcessError::FullPermissionRequiresOwned);
    }
    driver
        .resolve(&ResolveRequest {
            env,
            ownership,
            permission_mode: &resolved_permission_mode,
        })
        .map(Some)
}

/// Let a driver write full-permission mode into whatever its backend reads.
///
/// **External contract** — `server/src/process/managed-backend.mjs:61-69`. A
/// no-op unless the mode is `full`, and the second of the two places
/// [`ProcessError::FullPermissionRequiresOwned`] is raised — the redundancy is
/// upstream's, and it is the right kind: this function mutates the environment
/// a child will inherit, so it re-checks rather than trusting its caller.
///
/// # Errors
///
/// - [`ProcessError::FullPermissionRequiresOwned`] — `full` on a service VIA
///   does not own.
/// - Whatever the driver's hook refuses with.
pub fn apply_backend_permission_mode(
    driver: &BackendRuntimeDriver,
    env: &mut EnvMap,
    backend: &ManagedBackend,
) -> Result<(), ProcessError> {
    if backend.permission_mode != PERMISSION_MODE_FULL {
        return Ok(());
    }
    if backend.ownership != Ownership::Owned {
        return Err(ProcessError::FullPermissionRequiresOwned);
    }
    match &driver.hooks {
        Some(hooks) => hooks.apply_permission_mode(env, backend),
        None => Ok(()),
    }
}

/// Publish the resolved address into the child's environment.
///
/// **External contract** — `server/src/process/backend-drivers/shared.mjs:57-65`
/// (`applyLocalAddress`), reached through `managed-backend.mjs:110-112`. Two
/// writes: the base URL, and the port as a separate variable because the
/// backends read them separately.
///
/// The port fallback is upstream's exactly: an explicit port wins, else `443`
/// for `https:` and `80` for **everything else including `wss:`**. That last
/// part is a quirk — `wss://host` would publish port `80` — and it is
/// reproduced rather than corrected, because the only path that reaches it is
/// an owned backend whose address has just been reallocated and therefore
/// always carries an explicit port.
///
/// A no-op for a driver with no service address.
///
/// # Errors
///
/// [`ProcessError::InvalidServiceUrl`] if the resolved address does not parse,
/// which cannot happen for an address this crate produced.
pub fn apply_backend_address(
    driver: &BackendRuntimeDriver,
    env: &mut EnvMap,
    backend: &ManagedBackend,
) -> Result<(), ProcessError> {
    let (Some(service), Some(base_url)) = (&driver.service, &backend.base_url) else {
        return Ok(());
    };
    let target = parse_service_endpoint(base_url)?;
    env.set(service.base_url_environment.as_str(), base_url.as_str());
    let port = match target.port() {
        Some(port) => port.to_string(),
        None if target.scheme() == "https" => "443".to_owned(),
        None => "80".to_owned(),
    };
    env.set(service.port_environment.as_str(), port);
    Ok(())
}
