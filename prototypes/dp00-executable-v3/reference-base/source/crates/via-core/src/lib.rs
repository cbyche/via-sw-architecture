//! VIA's Core band: configuration, paths, identity and security.
//!
//! Everything above this crate — Layer 1's realtime clients, Layer 2's Work
//! queue, Layer 3's harnesses, the binary — asks `via-core` three questions:
//! *what did the operator configure*, *where do my files live*, and *who is
//! this request*. It answers all three and nothing else.
//!
//! | Module | What | Upstream |
//! | --- | --- | --- |
//! | [`config`] | The layered resolver and one `impl Default` | `server/src/core/config.mjs` |
//! | [`paths`] | Every directory and file the product creates | `shared/runtime-environment.mjs`, `install-paths.mjs` |
//! | [`identity`] | The HMAC-signed owner cookie | `server/src/core/identity.mjs` |
//! | [`security`] | The origin allow-list and DNS-rebinding defence | `server/src/core/request-security.mjs` |
//! | [`setup`] | The Gateway setup gate | `shared/gateway-setup.mjs` |
//! | [`runtime`] | First-run scaffolding | `shared/runtime-environment.mjs` |
//! | [`memory_scopes`] | The two memory documents and their aliases | `server/src/core/memory-scopes.mjs` |
//! | [`search_path`] | `PATH` merging for spawned children | `shared/path-environment.mjs` |
//!
//! # The one design decision worth reading
//!
//! **The resolver is a pure function.** [`config::resolve`] takes an
//! [`env::EnvMap`], the contents of a configuration file, and explicit
//! overrides; it reads no environment, no filesystem and no clock. The real
//! process environment is sampled exactly once, by the binary, through
//! [`env::EnvMap::from_process`].
//!
//! That is not tidiness. `std::env::set_var` is `unsafe` in edition 2024, so
//! the alternative — a resolver that reads the process environment — would make
//! every configuration test either `unsafe` or serialized against every other
//! test in the binary. Upstream evaluates its config object once at module
//! load, which is why its own test file can only test the *helpers*
//! (`numberSetting`, `resolveBackendModels`) and never the object. VIA can test
//! the object, and does: `tests/config_snapshot.rs` is an executable inventory
//! of all 75 catalogued defaults.
//!
//! ```
//! use via_core::config::{Overrides, resolve};
//! use via_core::env::EnvMap;
//!
//! let env: EnvMap = [("DASHSCOPE_API_KEY", "sk-example")].into_iter().collect();
//! let overrides = Overrides {
//!     home_directory: "/home/via".into(),
//!     working_directory: "/srv/via".into(),
//!     ..Overrides::default()
//! };
//!
//! let config = resolve(&env, None, &overrides)?;
//! assert_eq!(config.host, "127.0.0.1");
//! assert_eq!(config.port, 3101);
//! assert_eq!(config.config_directory(), std::path::Path::new("/home/via/.config/via"));
//! # Ok::<(), via_core::CoreError>(())
//! ```
//!
//! # Fidelity
//!
//! The defaults, the environment-variable names, the file names and the two
//! security algorithms are all external contracts catalogued in
//! `docs/reference/contracts.json`. Where VIA renames an upstream name it is
//! because `docs/rebrand.md` says the string is ours — `QWEN_AUDIO_AGENT_*`,
//! `QWEN_AUDIO_*` and `QWAUDIO_*` all collapse into `VIA_*`, and
//! `~/.config/qwaudio` becomes `~/.config/via`. Vendor namespaces
//! (`DASHSCOPE_*`, `OPENCLAW_*`, `QWEN_CODE_*`, …) are untouched. Every
//! deviation is recorded in `docs/deviations/phase-1.md`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod config;
pub mod env;
pub mod envfile;
pub mod error;
pub mod identity;
pub mod memory_scopes;
pub mod paths;
pub mod runtime;
pub mod search_path;
pub mod secret;
pub mod security;
pub mod setup;
pub mod text;

pub use config::{Config, GatewayOptions, Overrides, resolve};
pub use env::EnvMap;
pub use error::CoreError;
pub use identity::{Identity, IdentityManager, IdentityMode};
pub use paths::InstallPaths;
pub use secret::Secret;
pub use security::is_allowed_origin;
pub use setup::{SetupStatus, assert_gateway_setup, gateway_setup_status};
pub use text::is_js_whitespace;
