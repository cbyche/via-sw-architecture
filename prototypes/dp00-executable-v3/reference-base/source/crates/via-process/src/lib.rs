//! `via-process` — supervision of Gateway-owned backend child processes.
//!
//! Layer 3 (`docs/architecture.md` §9), and the half of it that has nothing to
//! do with ACP: deciding *whether* VIA starts a backend at all, on *what
//! address*, with *what environment*, and how it is torn down.
//!
//! | Module | What | Upstream |
//! | --- | --- | --- |
//! | [`driver`] | The driver descriptor, its registry and its validation | `server/src/process/backend-drivers/registry.mjs`, `shared.mjs` |
//! | [`managed`] | Ownership, permission mode, the resolved backend | `server/src/process/managed-backend.mjs:14-73` |
//! | [`endpoint`] | Service addresses, default ports, the loopback test | `server/src/process/backend-drivers/shared.mjs:1-19` |
//! | [`address`] | The in-use probe and ephemeral-port allocation | `server/src/process/managed-backend.mjs:75-108` |
//! | [`environment`] | The child environment trust boundary and its `PATH` | `shared/backend-environment.mjs`, `shared/backend-install.mjs:206-231` |
//! | [`spawn`] | The spawn spec and `process-wrap` | `server/src/process/managed-backend.mjs:114-129` |
//! | [`supervisor`] | `SIGTERM → wait → SIGKILL` | `server/src/process/managed-backend.mjs:131-166` |
//! | [`start`] | The whole sequence | `server/src/process/managed-backend.mjs:168-231` |
//! | [`runtime_state`] | The health surface a backend is read through | `server/src/agent/backend-runtime-state.mjs` |
//!
//! # This crate may not name a backend
//!
//! Upstream enforces that with a regex over source text
//! (`server/test/dependency-boundaries.test.mjs:60-73`); VIA enforces it as a
//! crate edge — `via-process` does not depend on `via-backends`, and
//! `via-arch-test` fails the build if it ever does
//! (`docs/architecture.md` §17 item 8). So every backend-specific fact arrives
//! as data on a [`driver::BackendRuntimeDriver`], and the three behaviours a
//! driver adds beyond data arrive as a [`driver::BackendRuntimeHooks`]
//! implementation. There is no `openclaw.rs` here and there never can be.
//!
//! ```
//! use via_catalog::Ownership;
//! use via_core::EnvMap;
//! use via_process::{
//!     BackendRuntimeDriver, BackendRuntimeRegistry, resolve_managed_backend,
//! };
//!
//! // A backend picked by a *property* rather than by name — this crate is not
//! // allowed to write one down. This is one the Gateway hosts in-process:
//! // no service address of its own.
//! let definition = via_catalog::backend_definitions()
//!     .iter()
//!     .find(|entry| entry.base_url_environment.is_none())
//!     .expect("the catalog has an in-Gateway backend");
//!
//! // The driver `via-backends` would build: data, straight from the catalog.
//! let mut registry = BackendRuntimeRegistry::new();
//! registry.register(BackendRuntimeDriver::in_gateway(definition))?;
//!
//! let env: EnvMap = [("AGENT_PROTOCOL", definition.id)].into_iter().collect();
//! let backend = resolve_managed_backend(&registry, &env)?.expect("a backend");
//! assert_eq!(backend.ownership, Ownership::Owned);
//! assert_eq!(backend.base_url, None); // hosted in-process: nothing to spawn
//!
//! // Frontend-only operation is `None`, not an error.
//! let bare: EnvMap = [("AGENT_PROTOCOL", "none")].into_iter().collect();
//! assert!(resolve_managed_backend(&registry, &bare)?.is_none());
//! # Ok::<(), via_process::ProcessError>(())
//! ```
//!
//! # Two axes, not one
//!
//! Service **ownership** — `owned` or `external` — says who starts and stops
//! the backend service. The **ACP connection** —
//! [`runtime_state::AcpConnection`] — says how VIA talks to it. They are
//! independent: a service someone else published can still be reached through
//! a locally spawned adapter process. Collapsing them would make that shape
//! unrepresentable, and it is the normal shape for a bridged backend.
//!
//! # There is no readiness probe and no restart
//!
//! `docs/reference/contracts.json`, `state-name/managed backend readiness /
//! restart / shutdown`: *"There is NO readiness probe and NO restart in
//! managed-backend.mjs […] restart is the embedding host's job via
//! GatewayProcess, not the Gateway's."* This crate reproduces that. Backend
//! exit is fatal to the Gateway process, and deciding what to do about it is
//! the embedder's; [`supervisor::ManagedBackendRuntime::wait`] is how they
//! find out.
//!
//! # Fidelity
//!
//! Every literal here is an external contract: the 300 ms probe budget, the
//! 1500 ms stop grace, the 30 s failure backoff, the twelve status codes, the
//! environment allow-lists, and eleven refusal messages. Each is
//! doc-commented with the upstream file it came from; the acceptance spec is
//! `docs/reference/contracts.json` and the deviations are recorded in
//! `docs/deviations/phase-2.md`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod address;
pub mod driver;
pub mod endpoint;
pub mod environment;
pub mod error;
pub mod managed;
pub mod runtime_state;
pub mod spawn;
pub mod start;
pub mod supervisor;

pub use address::{
    ADDRESS_PROBE_TIMEOUT, AddressAllocator, LOCALHOST_BIND_HOST, LOCALHOST_NAME,
    TokioAddressAllocator,
};
pub use driver::{
    BackendRuntimeDriver, BackendRuntimeHooks, BackendRuntimeRegistry, ManagedLaunch,
    ResolveRequest, ServiceAddress, normalize_backend_runtime_protocol, validate_runtime_driver,
};
pub use endpoint::{
    DEFAULT_SERVICE_PROTOCOLS, LOOPBACK_HOSTS, is_local_backend, normalize_service_endpoint,
    service_endpoint_port,
};
pub use environment::{
    ENV_LOADED_NAME, ENV_LOADED_VALUE, EXECUTABLE_PATH_SUFFIXES, INTERNAL_NAMES, SYSTEM_NAMES,
    SYSTEM_PREFIXES, backend_environment, compose_child_search_path, is_forwarded,
};
pub use error::ProcessError;
pub use managed::{
    BACKEND_PERMISSION_MODES, ManagedBackend, PERMISSION_MODE_FULL, PERMISSION_MODE_NATIVE,
    apply_backend_address, apply_backend_permission_mode, backend_ownership, permission_mode,
    resolve_managed_backend,
};
pub use runtime_state::{
    AcpConnection, BackendFailure, BackendRuntimeState, BackendRuntimeStateOptions,
    BackendStatusCode, BackendStatusKind, DEFAULT_BACKOFF_MS, InitializedAgent, TRANSPORT_ACP,
    backend_failure_code,
};
pub use spawn::{
    BackendSpawner, ChildStdio, CommandResolver, ProcessSpawner, SpawnSpec, WhichResolver,
    spawn_spec,
};
pub use start::{
    EVENT_ADDRESS_REALLOCATED, EVENT_PROCESS_START_FAILED, EVENT_PROCESS_STARTED,
    ManagedBackendStart, StartOptions, start_managed_backend,
};
pub use supervisor::{ChildExit, ManagedBackendRuntime, STOP_GRACE, StopSignal, SupervisedChild};
