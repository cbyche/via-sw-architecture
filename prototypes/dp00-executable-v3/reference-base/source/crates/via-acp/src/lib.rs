//! VIA's generic Agent Client Protocol client.
//!
//! One client, twelve backends. This crate speaks ACP to a child process and
//! knows nothing about which agent is on the other end: everything
//! backend-specific — the command, its arguments, its credential namespace, its
//! capability flags, the two hooks that rewrite its output and its errors —
//! arrives as **data** on an [`AcpBackendProfile`].
//!
//! That is a rule, not a style. `docs/architecture.md` §9 requires that
//! `via-acp` not depend on `via-backends`, `via-arch-test` enforces it, and
//! upstream asserts the same thing with a regex over source text
//! (`server/test/dependency-boundaries.test.mjs`, *"generic ACP and process
//! cores do not bind to named backends"*). No backend name appears anywhere in
//! this crate, including its test fixtures.
//!
//! # What is ported, and from where
//!
//! | Module | Upstream |
//! | --- | --- |
//! | [`client`] | `server/src/agent/acp-process-client.mjs` |
//! | [`content`] | `server/src/agent/acp-content.mjs` |
//! | [`session`] | `server/src/agent/acp-backend-session-utils.mjs` |
//! | [`registry`] | `server/src/agent/acp-session-registry.mjs` |
//! | [`factory`] | `server/src/agent/acp-client-factory.mjs` |
//! | [`permission`] | `acp-process-client.mjs` + `permission-broker.mjs` |
//! | [`error`] | `server/src/agent/backend-adapter.mjs` (`AgentError`) |
//! | [`env`] | `shared/backend-environment.mjs` |
//! | [`downstream`] | `acp-backend-profile.mjs`, `agent-client.mjs` |
//!
//! The ten catalogued [`methods`] and the catalogued [`limits`] are asserted
//! against `docs/reference/contracts.json` by this crate's own tests.
//!
//! # Three decisions worth reading before the code
//!
//! **The SDK owns JSON-RPC; VIA owns the process.** `agent-client-protocol`
//! frames NDJSON, correlates requests, dispatches notifications and cancels a
//! dropped request. It is not asked to spawn, because its spawn inherits the
//! parent environment, and it is not asked to tear down, because its teardown
//! is a bare `SIGKILL` after a fixed grace. See [`process`].
//!
//! **The credential boundary is a type.** Node's `spawn` replaces the child's
//! environment; Rust's `Command` inherits it. So [`env::BackendEnv`] has no
//! constructor but a projection through `via-catalog`'s environment policy, and
//! the single spawn site calls `.env_clear()` before applying it. A raw map
//! cannot reach a child process from here.
//!
//! **Cancellation carries a reason.** A prompt that the user stopped and a
//! prompt that ran out of time both end; which one it was decides what the user
//! hears next. [`cancel::CancelSignal`] is `AbortSignal` with its `reason`
//! intact, and `stopReason: "cancelled"` is an error rather than a result —
//! *"a port that returns `Ok` on `cancelled` would silently complete cancelled
//! Work."*
//!
//! # What this crate does **not** own
//!
//! `clean`, `bounded`, the two session-key formats and the `backend.activity`
//! projection all live in `via-downstream`, because they are Layer-3 vocabulary
//! shared by every harness rather than ACP mechanics. This crate reaches for
//! them rather than restating them.
//!
//! ```no_run
//! use via_acp::{AcpProcessClient, BackendEnv, SpawnSpec};
//! use via_catalog::EnvironmentPolicy;
//! use via_core::EnvMap;
//!
//! # async fn example() -> Result<(), via_acp::AcpError> {
//! // The child receives the portable OS variables plus this policy's
//! // namespace, and nothing else.
//! let policy = EnvironmentPolicy {
//!     names: &["EXAMPLE_API_KEY"],
//!     prefixes: &[],
//!     explicit_list_environment: None,
//! };
//! let env = BackendEnv::project(&policy, &EnvMap::from_process(), &[]);
//!
//! let client = AcpProcessClient::builder()
//!     .label("Example Agent")
//!     .spawn_spec(SpawnSpec::new("example-agent").args(["--acp"]).env(env))
//!     .build();
//!
//! let initialize = client.start().await?;
//! assert_eq!(initialize.protocol_version, via_acp::PROTOCOL_VERSION);
//! client.close().await;
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod cancel;
pub mod client;
pub mod content;
pub mod downstream;
pub mod env;
pub mod error;
pub mod factory;
pub mod limits;
pub mod methods;
pub mod permission;
pub mod process;
pub mod registry;
pub mod session;

pub use cancel::{CancelReason, CancelSignal, PausableTimeout};
pub use client::{
    AcpProcessClient, AcpProcessClientBuilder, CLIENT_NAME, CLIENT_TITLE, CLIENT_VERSION,
    FormatRequestError, PROTOCOL_VERSION, PermissionHandler, PromptOptions, PromptTurn,
    RequestErrorContext, SanitizeOutput, SessionDetails, SessionRecord, SessionRole, SharedSession,
    UpdateObserver, client_capabilities, declares_no_capability,
};
pub use content::{
    ATTACHMENT_URI_PREFIX, DataUrl, Prompt, append_prompt_blocks, assert_prompt_capabilities,
    input_parts_to_acp_blocks, is_text_mime, non_text_prompt_blocks, normalize_acp_prompt,
    normalize_acp_prompt_value, parse_data_url, prompt_with_input_parts, transform_prompt_text,
};
pub use downstream::{AcpBackendProfile, AcpDownstreamAgent, AcpHarnessSession};
pub use env::{BackendEnv, INTERNAL_NAMES, SYSTEM_NAMES, SYSTEM_PREFIXES};
pub use error::{AcpError, PROTOCOL, Result, STATUS_CONFLICT, SpawnErrno};
pub use factory::{
    ACP_CONNECTION_PROCESS, AcpConnection, create_acp_client, create_acp_client_by_kind,
};
pub use permission::{PermissionDecision, PermissionRequest};
pub use process::{AcpChild, AcpChildHandle, SpawnSpec};
pub use registry::{AcpSessionRegistry, CoordinatorRecord, ProjectRecord};
pub use session::{
    Presentation, SessionSummary, coordinator_presentation, native_tool_output,
    normalize_coordinator_content, parse_coordinator_payload, session_summary, text_from_update,
};
