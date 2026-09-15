//! The twelve named backend agents.
//!
//! This is the crate where backend names are **allowed**, and it is the reason
//! `via-acp` and `via-process` stay generic. `docs/architecture.md` §9:
//!
//! > \[upstream\] also asserts that the generic ACP and process cores must not
//! > mention a backend name. In JavaScript that needs a regex over source text;
//! > in Rust it is a crate boundary the compiler enforces.
//!
//! So `via-acp` speaks ACP to *a* child process, `via-process` supervises *a*
//! managed service, `via-downstream` declares *a* harness contract — and every
//! fact that distinguishes OpenCode from Codex from Pi is a table entry here.
//! `via-arch-test` fails the build if either core ever depends on this crate.
//!
//! | Module | What | Upstream |
//! | --- | --- | --- |
//! | [`capability`] | the twelve seven-flag declarations | `server/src/agent/backends/*.mjs` |
//! | [`driver`] | the driver table, its registry, and every launch spec | `server/src/agent/backends/registry.mjs` + the ten drivers |
//! | [`profile`] | what a driver returns beyond ACP | `backends/*.mjs` `createProfile` |
//! | [`runtime`] | the two service drivers, as `via-process` data | `server/src/process/backend-drivers/` |
//! | [`availability`] | the cached readiness a voice receipt is issued from | `server/src/agent/backend-availability.mjs` |
//! | [`detect`] | cross-platform executable discovery | `shared/backend-setup.mjs:57-91` |
//! | [`setup`] | the read-only "is this usable" report | `shared/backend-setup.mjs` |
//! | [`install`] | the pinned install specs and the executor | `shared/backend-install.mjs`, `shared/backend-catalog.mjs` |
//! | [`onboarding`] | install → configure → connect, as three separate facts | `shared/backend-onboarding.mjs`, `backend-lifecycle.mjs` |
//! | [`auth`] | read-only inspection of a backend's own sign-in | `shared/backend-auth-status.mjs` |
//! | [`openclaw`] | the one backend that is a Gateway, not a stdio child | `backends/openclaw.mjs`, `openclaw-adapter.mjs`, `openclaw-auth.mjs` |
//! | [`opencode`] | OpenCode's config isolation and resume rule | `backends/opencode.mjs`, `scripts/opencode.mjs` |
//! | [`exec`] | the real subprocess runners | `backend-install.mjs`, `backend-auth-status.mjs` |
//! | [`json5`] | just enough JSON5 to read a third-party config | *(new)* — see the module docs |
//! | [`platform`] | the three `process.platform` values these tables branch on | *(new)* |
//!
//! # Four properties this crate is responsible for
//!
//! **The credential boundary is applied per backend.** Every child is spawned
//! with [`via_acp::BackendEnv::project`] over **its own** catalogued
//! [`EnvironmentPolicy`](via_catalog::EnvironmentPolicy). The Gateway auth
//! secret, the realtime key, the memory-extractor key and every other backend's
//! credentials are *absent* from the child, not merely unused by it. The generic
//! ACP command may opt extra names in through `VIA_ACP_FORWARD_ENV` — upstream's
//! `QWEN_AUDIO_AGENT_ACP_FORWARD_ENV` — and nothing else can.
//!
//! **An inconclusive probe is `unknown`.** [`auth::AuthStatus`] has three
//! values and every branch that cannot prove a state answers the third.
//! Collapsing it to `unauthenticated` would nag users who are in fact signed in
//! through a keychain VIA cannot read.
//!
//! **A starting backend is not a broken one.** [`availability`] carries
//! `transient` end to end, because rejecting the first thing a user says after
//! launch is worse than accepting work that later reports a failure.
//!
//! **OpenClaw's two axes stay apart.** Service ownership and the ACP connection
//! are independent: with `OPENCLAW_BASE_URL` set, ownership is `external` and
//! the ACP connection is still a local `process`. VIA reads, copies and modifies
//! nothing belonging to a Gateway it does not own. See [`openclaw`].
//!
//! # Fidelity
//!
//! Every value that reproduces an upstream literal names the file it came from
//! in its own documentation, and `docs/reference/contracts.json` is the
//! acceptance spec — `tests/contracts.rs` parses it rather than restating it.
//! `assets/openclaw/openclaw.json5` is copied byte for byte with only the
//! identity renames `docs/rebrand.md` mandates, and `tests/openclaw_config.rs`
//! asserts it.
//!
//! ```
//! use via_backends::{backend_driver, backend_ids};
//!
//! // Registration is data: an id resolves to a driver, and nothing above
//! // Layer 3 ever names a concrete one.
//! assert_eq!(backend_ids().len(), 12);
//! let driver = backend_driver("  CODEX ")?;
//! assert_eq!(driver.id, "codex");
//! assert!(driver.capabilities.session_mcp);
//! # Ok::<(), via_backends::BackendsError>(())
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod auth;
pub mod availability;
pub mod capability;
pub mod detect;
pub mod driver;
pub mod error;
pub mod exec;
pub mod install;
pub mod json5;
pub mod onboarding;
pub mod openclaw;
pub mod opencode;
pub mod platform;
pub mod profile;
pub mod runtime;
pub mod setup;

pub use auth::{
    AuthInspection, AuthStatus, CredentialFiles, HostCredentialFiles, NoProbeRunner, ProbeOutcome,
    ProbeRequest, ProbeRunner, inspect_backend_authentication,
};
pub use availability::{
    Availability, AvailabilityProbe, AvailabilitySnapshot, BackendAvailability, HarnessProbe,
    ProbeFailed,
};
pub use capability::backend_capabilities;
pub use detect::{ExecutableFinder, MissingFinder, SystemFinder, find_executable};
pub use driver::{
    BackendDriver, LaunchContext, backend_driver, backend_drivers, backend_ids,
    create_backend_profile, create_downstream_agent, has_backend_driver, register_backends,
};
pub use error::BackendsError;
pub use exec::{ProcessProbeRunner, ProcessStepRunner};
pub use install::{
    ComponentReadiness, InstallError, InstallObserver, InstallRequest, InstallStepSpec,
    InstallSupport, ProgressEvent, ProgressPhase, SetupInspector, StepConfirmer, StepKind,
    StepRunner, install_backend, install_steps, install_support,
};
pub use onboarding::{
    AuthenticationSupport, ConfigurationAction, OnboardingAdapter, OnboardingState,
    backend_authentication_support, backend_configuration_action, backend_onboarding_adapter,
    resolve_backend_onboarding,
};
pub use platform::HostPlatform;
pub use profile::{BackendProfile, BackendUi, CoordinatorMeta, PrepareAction, SessionInstructions};
pub use runtime::{
    managed_launch, managed_service_port, openclaw_runtime_driver, opencode_runtime_driver,
    port_environment, runtime_registry, spawns_separate_process,
};
pub use setup::{
    BackendSetup, SetupInspection, SetupReport, format_backend_setup, inspect_backend_setups,
};
