//! `start_managed_backend` — the whole sequence, in upstream's order.
//!
//! Ported from `server/src/process/managed-backend.mjs:168-231`.
//!
//! ```text
//!   resolve            no backend?          -> own nothing
//!   write ownership back into env
//!   apply permission mode
//!   no separate process? external service?  -> own nothing
//!   driver prepares the environment
//!   refuse a non-loopback address
//!   probe the address; if busy, allocate another and log the move
//!   publish the address into the child's env
//!   build the spawn spec and spawn
//! ```
//!
//! Three of those steps are refusals rather than work, and the order they come
//! in is the contract: the ownership write-back happens *before* the child is
//! spawned so the child sees the resolved value
//! (`env-var/QWEN_AUDIO_AGENT_BACKEND_OWNERSHIP`), and the loopback refusal
//! happens *before* the port probe so the Gateway never opens a socket to a
//! remote host it was never going to launch on.

use std::path::PathBuf;

use serde_json::{Map, Value, json};
use via_core::EnvMap;
use via_core::config::names;
use via_core::search_path::Platform;
use via_log::Logger;

use crate::address::AddressAllocator;
use crate::driver::BackendRuntimeRegistry;
use crate::endpoint::is_local_backend;
use crate::error::ProcessError;
use crate::managed::{
    ManagedBackend, apply_backend_address, apply_backend_permission_mode, resolve_managed_backend,
};
use crate::spawn::{BackendSpawner, CommandResolver, spawn_spec};
use crate::supervisor::ManagedBackendRuntime;

/// Logged when an occupied address forces the backend to move.
///
/// **External contract** — `server/src/process/managed-backend.mjs:210`.
pub const EVENT_ADDRESS_REALLOCATED: &str = "backend.address_reallocated";

/// Logged once the child is running.
///
/// **External contract** — `server/src/process/managed-backend.mjs:219`.
pub const EVENT_PROCESS_STARTED: &str = "backend.process_started";

/// Logged when the spawn itself fails.
///
/// **External contract** — `server/src/process/managed-backend.mjs:226`.
pub const EVENT_PROCESS_START_FAILED: &str = "backend.process_start_failed";

/// The collaborators [`start_managed_backend`] needs.
///
/// Every one of them is a trait object for the reason upstream passes
/// `spawnImpl` / `isAddressInUse` / `findFreeAddress`: the behaviours worth
/// asserting are mostly *absences* — no probe, no move, no spawn — and an
/// absence can only be asserted against a collaborator that would have
/// noticed.
pub struct StartOptions<'a> {
    /// The installation root; becomes the child's working directory.
    pub root: PathBuf,
    /// The drivers this Gateway knows about.
    pub registry: &'a BackendRuntimeRegistry,
    /// Which platform's `PATH` conventions the child environment follows.
    pub platform: Platform,
    /// Probes and allocates loopback addresses.
    pub allocator: &'a dyn AddressAllocator,
    /// Turns a spec into a process.
    pub spawner: &'a dyn BackendSpawner,
    /// Finds the launch command on the child's `PATH`.
    pub resolver: &'a dyn CommandResolver,
    /// Where the three lifecycle events go.
    pub logger: Option<&'a Logger>,
}

impl std::fmt::Debug for StartOptions<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StartOptions")
            .field("root", &self.root)
            .field("platform", &self.platform)
            .field("registered_drivers", &self.registry.registered())
            .field("logger", &self.logger.is_some())
            .finish_non_exhaustive()
    }
}

/// What a start produced.
///
/// # Deviation
///
/// Upstream returns the runtime alone and leaves the resolved backend
/// observable only through the mutated environment. VIA returns both, because
/// the caller needs [`ManagedBackend::base_url`] *after* a reallocation may
/// have changed it, and recovering that by re-reading the environment would
/// mean re-deriving which variable to read — which is driver knowledge this
/// crate has and the caller should not need. Recorded in
/// `docs/deviations/phase-2.md`.
#[derive(Debug)]
pub struct ManagedBackendStart {
    /// The backend that was resolved, or `None` for frontend-only operation.
    pub backend: Option<ManagedBackend>,
    /// The process handle. Owns nothing unless VIA actually spawned.
    pub runtime: ManagedBackendRuntime,
}

/// Resolve, prepare and — where VIA owns it — launch the backend.
///
/// **External contract** — `server/src/process/managed-backend.mjs:168-231`.
/// `env` is mutated in place, exactly as upstream mutates `process.env`: the
/// resolved ownership, the driver's prepared variables and the published
/// address all have to be visible to the child, and the child inherits a
/// projection of this map.
///
/// Four separate paths end with a runtime that owns no process, and they are
/// four different situations rather than one: no backend configured, a backend
/// the Gateway hosts in-process, an external service, and — reachable only
/// through a hand-built driver — an ownership the driver cannot honour.
///
/// # Errors
///
/// Anything [`resolve_managed_backend`], [`apply_backend_permission_mode`] or
/// [`spawn_spec`] refuses with, plus [`ProcessError::LocalOnly`] for a
/// non-loopback address and [`ProcessError::Io`] for a failed spawn.
pub async fn start_managed_backend(
    env: &mut EnvMap,
    options: &StartOptions<'_>,
) -> Result<ManagedBackendStart, ProcessError> {
    let Some(mut backend) = resolve_managed_backend(options.registry, env)? else {
        return Ok(ManagedBackendStart {
            backend: None,
            runtime: ManagedBackendRuntime::detached(),
        });
    };
    let driver = options.registry.driver(&backend.protocol)?;

    // Before anything else: the child must see the ownership VIA decided on,
    // not the (possibly empty) value the operator configured.
    env.set(names::BACKEND_OWNERSHIP, backend.ownership.as_str());
    apply_backend_permission_mode(&driver, env, &backend)?;

    if !driver.separate_managed_process || backend.ownership != via_catalog::Ownership::Owned {
        // An in-Gateway backend has no service to start; an external one is
        // someone else's to start, move and stop. Neither probes a port and
        // neither spawns.
        return Ok(ManagedBackendStart {
            backend: Some(backend),
            runtime: ManagedBackendRuntime::detached(),
        });
    }

    if let Some(hooks) = &driver.hooks {
        hooks.prepare_environment(env, &backend)?;
    }

    let base_url =
        backend
            .base_url
            .clone()
            .ok_or_else(|| ProcessError::DriverMissingManagedLaunch {
                id: driver.id.clone(),
            })?;
    if !is_local_backend(&base_url)? {
        return Err(ProcessError::LocalOnly { url: base_url });
    }

    if options.allocator.address_in_use(&base_url).await? {
        log(
            options.logger,
            LogLevel::Info,
            EVENT_ADDRESS_REALLOCATED,
            [
                ("backend".to_owned(), json!(backend.protocol)),
                ("requestedBaseUrl".to_owned(), json!(base_url)),
            ]
            .into_iter()
            .collect(),
        );
        backend.base_url = Some(options.allocator.allocate(&base_url).await?);
    }
    apply_backend_address(&driver, env, &backend)?;

    let spec = spawn_spec(
        &driver,
        &options.root,
        env,
        options.platform,
        options.resolver,
    )?;
    let child = match options.spawner.spawn(&spec).await {
        Ok(child) => child,
        Err(error) => {
            log(
                options.logger,
                LogLevel::Error,
                EVENT_PROCESS_START_FAILED,
                [
                    ("backend".to_owned(), json!(backend.protocol)),
                    ("error".to_owned(), json!(error.to_string())),
                ]
                .into_iter()
                .collect(),
            );
            return Err(error);
        }
    };
    log(
        options.logger,
        LogLevel::Info,
        EVENT_PROCESS_STARTED,
        [
            ("backend".to_owned(), json!(backend.protocol)),
            // Upstream writes `pid: child.pid` here, and loses it: `pid` is one
            // of the seven reserved names its logger stamps *after* the caller's
            // fields (`shared/logger.mjs:227,302`), so the record carries the
            // Gateway's own pid and the child's never reaches the log.
            // `via-log` reserves the same seven for the same reason, so the
            // value is published under a name that survives. Recorded in
            // `docs/deviations/phase-2.md`.
            ("childPid".to_owned(), json!(child.id())),
            ("baseUrl".to_owned(), json!(backend.base_url)),
            ("ownership".to_owned(), json!(backend.ownership.as_str())),
        ]
        .into_iter()
        .collect(),
    );
    Ok(ManagedBackendStart {
        backend: Some(backend),
        runtime: ManagedBackendRuntime::owning(child),
    })
}

enum LogLevel {
    Info,
    Error,
}

/// Emit one lifecycle event, if a logger was supplied.
///
/// Upstream's `logger?.info(...)`. The message is empty because these are
/// machine events, not sentences: `via-log`'s record carries `event` and the
/// fields, and nothing here is read by a person in prose form.
fn log(logger: Option<&Logger>, level: LogLevel, event: &str, fields: Map<String, Value>) {
    let Some(logger) = logger else { return };
    match level {
        LogLevel::Info => logger.info(event, fields, ""),
        LogLevel::Error => logger.error(event, fields, ""),
    }
}
