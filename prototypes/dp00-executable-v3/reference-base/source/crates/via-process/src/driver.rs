//! Backend runtime drivers: the data a caller hands in so this crate can
//! supervise a backend it is not allowed to know the name of.
//!
//! Ported from `server/src/process/backend-drivers/registry.mjs:1-64` and
//! `server/src/process/backend-drivers/shared.mjs:21-65`.
//!
//! # The boundary this module exists to keep
//!
//! Upstream asserts with a regex over source text that the generic process
//! core never mentions `openclaw|opencode|qoder|qwen|kimi|…`
//! (`server/test/dependency-boundaries.test.mjs:60-73`). VIA asserts it as a
//! crate edge instead: `via-process` does not depend on `via-backends`, and
//! `via-arch-test`'s `via_acp_and_via_process_do_not_depend_on_via_backends`
//! fails the build if it ever does.
//!
//! So a driver is *data*. Upstream's `openCodeRuntimeDriver` and
//! `openClawRuntimeDriver` become [`BackendRuntimeDriver`] values built in
//! `via-backends`; upstream's `managedProcessDriver(definition)` factory —
//! which manufactures a driver for the ten backends with no dedicated one —
//! becomes [`BackendRuntimeDriver::in_gateway`]. The three behaviours a
//! dedicated driver adds beyond data (a bespoke refusal, an environment it
//! prepares, a permission mode it writes into a config blob) arrive as a
//! [`BackendRuntimeHooks`] implementation.
//!
//! # Validation happens at registration
//!
//! `docs/architecture.md` §6: *"A descriptor that is incomplete or internally
//! inconsistent is rejected at startup, not discovered mid-turn."*
//! [`BackendRuntimeRegistry::register`] runs [`validate_runtime_driver`], which
//! is upstream's `validateRuntimeDriver` minus the two duck-typing checks Rust
//! makes unrepresentable.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use via_catalog::{BackendDefinition, EnvironmentPolicy, Ownership, backend_definition};
use via_core::EnvMap;

use crate::endpoint::DEFAULT_SERVICE_PROTOCOLS;
use crate::error::ProcessError;
use crate::managed::ManagedBackend;

/// Normalise a configured `AGENT_PROTOCOL` value.
///
/// **External contract** — `server/src/process/backend-drivers/registry.mjs:51-54`
/// and `env-var/AGENT_PROTOCOL`. Trim, lowercase, and map the `none` sentinel
/// to the empty string, which means "no backend, frontend-only".
#[must_use]
pub fn normalize_backend_runtime_protocol(protocol: &str) -> String {
    let id = protocol.trim().to_lowercase();
    if id == via_catalog::BACKEND_NONE_SENTINEL {
        String::new()
    } else {
        id
    }
}

/// Where a driver publishes its service, when it publishes one.
///
/// Ported from the four fields upstream's two service drivers share
/// (`backend-drivers/opencode.mjs:33-37`, `openclaw.mjs:8-12,28-33`) plus the
/// two `applyLocalAddress` takes (`shared.mjs:57-65`). A driver with no
/// `ServiceAddress` is upstream's `managedOnlyBackend`: it has no HTTP or
/// WebSocket surface at all and its `baseUrl` is `null`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceAddress {
    /// The variable an operator overrides the address with.
    pub base_url_environment: String,
    /// The address used when nothing overrides it.
    pub default_base_url: String,
    /// The variable the resolved port is written back into.
    pub port_environment: String,
    /// The URL schemes this service admits, each with its trailing colon.
    pub protocols: Vec<String>,
}

impl ServiceAddress {
    /// A service reachable over plain HTTP only.
    #[must_use]
    pub fn http(
        base_url_environment: impl Into<String>,
        default_base_url: impl Into<String>,
        port_environment: impl Into<String>,
    ) -> Self {
        Self {
            base_url_environment: base_url_environment.into(),
            default_base_url: default_base_url.into(),
            port_environment: port_environment.into(),
            protocols: vec!["http:".to_owned(), "https:".to_owned()],
        }
    }

    /// A service reachable over HTTP or WebSocket.
    ///
    /// `env-var/OPENCLAW_BASE_URL` notes that exactly one shipped driver
    /// admits `ws:`/`wss:`; the shape stays general because it is the driver
    /// that decides, not this crate.
    #[must_use]
    pub fn websocket(
        base_url_environment: impl Into<String>,
        default_base_url: impl Into<String>,
        port_environment: impl Into<String>,
    ) -> Self {
        Self {
            base_url_environment: base_url_environment.into(),
            default_base_url: default_base_url.into(),
            port_environment: port_environment.into(),
            protocols: DEFAULT_SERVICE_PROTOCOLS
                .iter()
                .map(|value| (*value).to_owned())
                .collect(),
        }
    }

    fn protocol_refs(&self) -> Vec<&str> {
        self.protocols.iter().map(String::as_str).collect()
    }
}

/// How a driver's separate managed process is launched.
///
/// # Deviation
///
/// Upstream's field is `managedScript` — a `scripts/*.mjs` file name resolved
/// against the repository root and spawned with `process.execPath`
/// (`state-name/managed backend spawn spec`). `docs/architecture.md` §10 says
/// those shims do not survive the port: *"the shims become `via-backends`
/// launch specs"*. So the driver carries a command and its arguments, and the
/// command is resolved on the child's composed `PATH`. The refusal for a
/// driver that declares a separate process and supplies no launch is
/// upstream's `后台 Runtime Driver 缺少 managedScript：<id>`, unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedLaunch {
    /// The executable, as a bare name to resolve on `PATH` or an absolute
    /// path to use as-is.
    pub command: String,
    /// Arguments, passed through verbatim — never through a shell.
    pub arguments: Vec<String>,
    /// Extra environment entries stamped after the policy projection.
    ///
    /// Upstream's `additions`, which `spawnSpec` uses for
    /// `ELECTRON_RUN_AS_NODE=1` (`managed-backend.mjs:114-119`). That
    /// particular entry is a Node-runtime concern and belongs to whoever
    /// builds the driver, not to this crate.
    pub environment_additions: Vec<(String, String)>,
}

impl ManagedLaunch {
    /// A launch with no extra environment.
    #[must_use]
    pub fn new(command: impl Into<String>, arguments: Vec<String>) -> Self {
        Self {
            command: command.into(),
            arguments,
            environment_additions: Vec::new(),
        }
    }
}

/// What a driver needs to know to answer a resolve.
///
/// Upstream passes `{ env, ownership, permissionMode }` to `driver.resolve`
/// (`managed-backend.mjs:54-58`).
#[derive(Debug, Clone, Copy)]
pub struct ResolveRequest<'a> {
    /// The Gateway's environment.
    pub env: &'a EnvMap,
    /// Service ownership, already decided.
    pub ownership: Ownership,
    /// The permission mode, already normalised through
    /// [`via_catalog::effective_backend_permission_mode`].
    pub permission_mode: &'a str,
}

/// The three behaviours a dedicated driver adds beyond data.
///
/// Every method defaults to doing nothing, matching upstream's optional-call
/// syntax (`driver.applyPermissionMode?.(…)`).
pub trait BackendRuntimeHooks: Send + Sync + fmt::Debug {
    /// A refusal only this driver can express, checked before anything else.
    ///
    /// Upstream's one instance is `backend-drivers/openclaw.mjs:17-20`, whose
    /// message names three configuration sections of a third-party product.
    /// Returning [`ProcessError::DriverRefused`] keeps the sentence in
    /// `via-i18n` while keeping the *decision* in the driver.
    ///
    /// # Errors
    ///
    /// Whatever the driver refuses with.
    fn guard(&self, request: &ResolveRequest<'_>) -> Result<(), ProcessError> {
        let _ = request;
        Ok(())
    }

    /// Prepare the environment just before the child is spawned.
    ///
    /// Upstream's `prepareEnvironment` (`backend-drivers/openclaw.mjs:41-51`),
    /// which mints a transport credential when the Gateway is managing the
    /// backend and none is configured.
    ///
    /// # Errors
    ///
    /// Whatever the driver refuses with.
    fn prepare_environment(
        &self,
        env: &mut EnvMap,
        backend: &ManagedBackend,
    ) -> Result<(), ProcessError> {
        let _ = (env, backend);
        Ok(())
    }

    /// Write full-permission mode into whatever the backend reads.
    ///
    /// Upstream's `applyPermissionMode` (`backend-drivers/opencode.mjs:53-77`),
    /// which rewrites an inline JSON configuration blob.
    ///
    /// # Errors
    ///
    /// Whatever the driver refuses with.
    fn apply_permission_mode(
        &self,
        env: &mut EnvMap,
        backend: &ManagedBackend,
    ) -> Result<(), ProcessError> {
        let _ = (env, backend);
        Ok(())
    }
}

/// One backend's runtime driver.
///
/// Ported from the driver object literals in
/// `server/src/process/backend-drivers/`. Everything here is declarative
/// except [`Self::hooks`].
#[derive(Clone)]
pub struct BackendRuntimeDriver {
    /// The backend protocol id. Must match a catalog entry.
    pub id: String,
    /// The catalog label, interpolated into refusals.
    pub label: String,
    /// Whether the Gateway spawns a separate service process for this backend.
    ///
    /// **External contract** — `state-name/managed backend spawn spec`: only
    /// two backends do; *"the rest are in-process ACP stdio clients started
    /// elsewhere"*.
    pub separate_managed_process: bool,
    /// Whether the backend may be an already-running external service.
    pub supports_external_service: bool,
    /// Whether the Gateway's single switch may turn on full permission.
    pub supports_full_permission: bool,
    /// The service address, when the driver publishes one.
    pub service: Option<ServiceAddress>,
    /// How the separate process is launched, when there is one.
    pub launch: Option<ManagedLaunch>,
    /// The credential-namespace boundary for this backend's children.
    pub environment: EnvironmentPolicy,
    /// The driver's own behaviour, when it has any.
    pub hooks: Option<Arc<dyn BackendRuntimeHooks>>,
}

impl fmt::Debug for BackendRuntimeDriver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BackendRuntimeDriver")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("separate_managed_process", &self.separate_managed_process)
            .field("supports_external_service", &self.supports_external_service)
            .field("supports_full_permission", &self.supports_full_permission)
            .field("service", &self.service)
            .field("launch", &self.launch)
            .field("hooks", &self.hooks.is_some())
            .finish()
    }
}

impl BackendRuntimeDriver {
    /// A driver for a backend the Gateway hosts in-process — upstream's
    /// `managedProcessDriver(definition)`.
    ///
    /// **External contract** — `server/src/process/backend-drivers/registry.mjs:33-49`.
    /// No separate process, no service address, and
    /// [`Self::supports_full_permission`] taken straight from the catalog, so
    /// the refusal a `full` request meets is the catalog's generic
    /// `{label} 无法安全地统一开启最高权限模式`.
    #[must_use]
    pub fn in_gateway(definition: &BackendDefinition) -> Self {
        Self {
            id: definition.id.to_owned(),
            label: definition.label.to_owned(),
            separate_managed_process: false,
            supports_external_service: definition.supports_external_service,
            supports_full_permission: definition.supports_full_permission,
            service: None,
            launch: None,
            environment: definition.environment,
            hooks: None,
        }
    }

    /// Resolve this driver against the environment.
    ///
    /// **External contract** — `backend-drivers/shared.mjs:21-53`, whose two
    /// shapes are selected by [`Self::service`]:
    ///
    /// * `Some` — upstream's `localManagedBackend`: the configured address (or
    ///   the default) reduced to its origin and checked against the driver's
    ///   scheme list.
    /// * `None` — upstream's `managedOnlyBackend`: no address, and a hard
    ///   refusal if the caller asked for external ownership.
    ///
    /// The two guards that run first are upstream's driver-level checks: the
    /// driver's own [`BackendRuntimeHooks::guard`], then the catalog's generic
    /// full-permission refusal. Order matters — a driver with a bespoke
    /// message must produce it rather than the generic one.
    ///
    /// # Errors
    ///
    /// [`ProcessError::DriverRefused`], [`ProcessError::FullPermissionUnsafe`],
    /// [`ProcessError::MustBeGatewayStarted`], or any error
    /// [`crate::endpoint::normalize_service_endpoint`] raises.
    pub fn resolve(&self, request: &ResolveRequest<'_>) -> Result<ManagedBackend, ProcessError> {
        if let Some(hooks) = &self.hooks {
            hooks.guard(request)?;
        }
        if request.permission_mode == crate::managed::PERMISSION_MODE_FULL
            && !self.supports_full_permission
        {
            return Err(ProcessError::FullPermissionUnsafe {
                label: self.label.clone(),
            });
        }
        match &self.service {
            Some(service) => {
                let configured = request.env.get_trimmed(&service.base_url_environment);
                let value = if configured.is_empty() {
                    service.default_base_url.as_str()
                } else {
                    configured
                };
                let base_url =
                    crate::endpoint::normalize_service_endpoint(value, &service.protocol_refs())?;
                Ok(ManagedBackend {
                    protocol: self.id.clone(),
                    ownership: request.ownership,
                    permission_mode: request.permission_mode.to_owned(),
                    base_url: Some(base_url),
                })
            }
            None => {
                if request.ownership != Ownership::Owned {
                    return Err(ProcessError::MustBeGatewayStarted {
                        label: self.label.clone(),
                    });
                }
                Ok(ManagedBackend {
                    protocol: self.id.clone(),
                    ownership: request.ownership,
                    permission_mode: request.permission_mode.to_owned(),
                    base_url: None,
                })
            }
        }
    }
}

/// Check a driver against the catalog entry it claims to implement.
///
/// **External contract** — `server/src/process/backend-drivers/registry.mjs:11-31`.
/// Upstream makes five checks; two of them —
/// `后台 Runtime Driver 缺少 resolve` and
/// `后台 Runtime Driver 缺少进程归属声明` — test that a JavaScript object has a
/// function-typed property and a boolean-typed property. In Rust those are
/// struct fields with types, so neither state is representable and neither
/// check has anything to do. The remaining three are here, with upstream's
/// messages. Recorded in `docs/deviations/phase-2.md`.
///
/// Upstream's registry test also asserts that a driver's
/// `baseUrlEnvironment` and `defaultBaseUrl` agree with the catalog
/// (`server/test/backend-driver-registry.test.mjs:19-27`); that assertion is
/// stronger than `validateRuntimeDriver` itself and is reproduced here, since
/// `default-value/backend default base URLs` records the three declarations
/// *"that a startup test asserts must agree"*.
///
/// Promoting that assertion into the validator is a **strengthening**, and the
/// case it catches is real: upstream's `managedProcessDriver` manufactures a
/// driver with no address at all, and upstream never reaches it for a backend
/// that has one only because its driver map is a module literal. VIA's registry
/// is built by the caller, so an unregistered service backend *is* reachable,
/// and manufacturing an addressless driver for it would leave the Gateway
/// connecting nowhere. Recorded in `docs/deviations/phase-2.md`.
///
/// `definition` is a parameter rather than a catalog lookup, exactly as
/// upstream's `validateRuntimeDriver(driver, definition)` takes one. That is
/// what lets this crate's own tests exercise every refusal against a synthetic
/// definition without writing a real backend's name into a file that is not
/// allowed to hold one.
///
/// # Errors
///
/// - [`ProcessError::DriverIdMismatch`] — the id or the label disagrees with
///   the catalog, or the service address does.
/// - [`ProcessError::DriverExternalServiceMismatch`] — the external-service
///   capability disagrees.
/// - [`ProcessError::DriverMissingManagedLaunch`] — a separate managed process
///   was declared with no launch command or no service address.
pub fn validate_runtime_driver(
    driver: &BackendRuntimeDriver,
    definition: &BackendDefinition,
) -> Result<(), ProcessError> {
    if definition.id != driver.id || definition.label != driver.label {
        return Err(ProcessError::DriverIdMismatch {
            id: driver.id.clone(),
        });
    }
    if driver.supports_external_service != definition.supports_external_service {
        return Err(ProcessError::DriverExternalServiceMismatch {
            id: driver.id.clone(),
        });
    }
    let declared_base_url = driver
        .service
        .as_ref()
        .map(|service| service.base_url_environment.as_str());
    if declared_base_url != definition.base_url_environment {
        return Err(ProcessError::DriverIdMismatch {
            id: driver.id.clone(),
        });
    }
    let declared_default = driver
        .service
        .as_ref()
        .map(|service| service.default_base_url.as_str());
    if declared_default != definition.default_base_url {
        return Err(ProcessError::DriverIdMismatch {
            id: driver.id.clone(),
        });
    }
    // Upstream tests only for a missing `managedScript`. VIA also refuses a
    // separate managed process with no service address: the whole point of
    // spawning one is that it publishes an address the Gateway then connects
    // to, and a driver that declares the spawn without the address would reach
    // the loopback check with `baseUrl: null` — where upstream's
    // `new URL(null)` throws an untyped `TypeError`. Same incompleteness, same
    // message, caught at registration instead of at launch.
    if driver.separate_managed_process && (driver.launch.is_none() || driver.service.is_none()) {
        return Err(ProcessError::DriverMissingManagedLaunch {
            id: driver.id.clone(),
        });
    }
    Ok(())
}

/// The catalog entry for an id, or the refusal upstream's registry raises.
///
/// **External contract** — `server/src/process/backend-drivers/registry.mjs:56-62`.
/// The id is normalised first, so `none` and an empty value both reach the
/// refusal rather than resolving to a backend.
///
/// # Errors
///
/// [`ProcessError::UnsupportedBackend`].
fn catalogued(protocol: &str) -> Result<BackendDefinition, ProcessError> {
    let id = normalize_backend_runtime_protocol(protocol);
    backend_definition(&id)
        .copied()
        .ok_or(ProcessError::UnsupportedBackend { protocol: id })
}

/// The drivers a Gateway knows about.
///
/// Ported from `server/src/process/backend-drivers/registry.mjs:6-9,56-63`.
/// Upstream's map is a module-level literal naming two drivers, with a factory
/// for everything else; VIA's is built by the caller, because this crate may
/// not name a backend. What stays here is the lookup and the validation.
#[derive(Debug, Clone, Default)]
pub struct BackendRuntimeRegistry {
    drivers: BTreeMap<String, BackendRuntimeDriver>,
}

impl BackendRuntimeRegistry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a driver, validating it first.
    ///
    /// # Errors
    ///
    /// Whatever [`validate_runtime_driver`] refuses with. Registration is the
    /// moment an inconsistent descriptor must be caught — see the module docs.
    pub fn register(&mut self, driver: BackendRuntimeDriver) -> Result<(), ProcessError> {
        validate_runtime_driver(&driver, &catalogued(&driver.id)?)?;
        self.drivers.insert(driver.id.clone(), driver);
        Ok(())
    }

    /// Look up a driver by protocol id, normalising the id first.
    ///
    /// **External contract** — `server/src/process/backend-drivers/registry.mjs:56-63`.
    /// A registered driver wins; otherwise a catalogued backend gets the
    /// in-Gateway driver upstream's factory would have manufactured; otherwise
    /// the id is refused.
    ///
    /// The returned driver is validated on every lookup, as upstream's is.
    ///
    /// # Errors
    ///
    /// - [`ProcessError::UnsupportedBackend`] — no such backend.
    /// - Whatever [`validate_runtime_driver`] refuses with.
    pub fn driver(&self, protocol: &str) -> Result<BackendRuntimeDriver, ProcessError> {
        let id = normalize_backend_runtime_protocol(protocol);
        let definition = catalogued(&id)?;
        if let Some(driver) = self.drivers.get(&id) {
            validate_runtime_driver(driver, &definition)?;
            return Ok(driver.clone());
        }
        let driver = BackendRuntimeDriver::in_gateway(&definition);
        validate_runtime_driver(&driver, &definition)?;
        Ok(driver)
    }

    /// The ids of every explicitly registered driver.
    #[must_use]
    pub fn registered(&self) -> Vec<&str> {
        self.drivers.keys().map(String::as_str).collect()
    }
}
