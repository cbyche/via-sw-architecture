//! VIA's Layer-3 extension point.
//!
//! Everything that answers a delegated prompt sits behind two traits declared
//! here — [`DownstreamAgent`] and [`HarnessSession`] — and nothing above Layer 3
//! names a concrete harness. `docs/architecture.md` §6:
//!
//! > **VIA ships no agent harness of its own.** […] Layer 3 is a single
//! > extension point — `trait DownstreamAgent` — and every harness, ARGO's
//! > included, is a plugin behind it.
//!
//! | Module | What | Upstream |
//! | --- | --- | --- |
//! | [`agent`] | the two traits | *(new)* — the contract below, lifted into one seam |
//! | [`descriptor`] | id, label, seven flags, and the validation | `server/src/agent/backends/registry.mjs:22-45` |
//! | [`capability`] | the seven-flag contract | `server/src/agent/backends/registry.mjs:12-20` |
//! | [`registry`] | configured id → harness | `server/src/agent/backends/registry.mjs:47-70` |
//! | [`session_key`] | the coordinator and project key formats | `server/src/agent/acp-backend-session-utils.mjs:9-15` |
//! | [`event`] | the adapter-normalised progress vocabulary | `server/src/agent/acp-backend-session-utils.mjs:80-140` |
//! | [`cancel`] | confirmed cancellation | `server/src/agent/acp-backend-adapter.mjs:907-928,1373-1422` |
//! | [`prompt`] | one turn in, one turn out | `server/src/agent/acp-process-client.mjs:436-511` |
//! | [`health`] | can this harness take a turn | `server/src/agent/backend-runtime-state.mjs` |
//! | [`error`] | every refusal, coded | `server/src/agent/backend-adapter.mjs`, `backends/registry.mjs` |
//! | [`text`] | `clean` and `bounded` | `server/src/agent/acp-backend-session-utils.mjs:1-7` |
//! | [`testing`] | a deterministic double | *(new)* |
//!
//! # This crate is new; the contract in it is not
//!
//! Upstream has no single trait over its backends. It has a *validated plugin
//! contract* spread across four files, and this crate is that contract lifted
//! rather than a new design:
//!
//! - `backends/registry.mjs` — the seven capability flags, the four
//!   registration refusals, and the three lookups. Becomes
//!   [`BackendCapabilities`], [`HarnessDescriptor::declare`] and
//!   [`HarnessRegistry`].
//! - `backends/shared.mjs` — `clean`, and the connection description a driver
//!   returns. Becomes [`text::clean`]; the connection itself is `via-acp`'s,
//!   because it is ACP's.
//! - `backend-adapter.mjs` — `AgentError { status, body, protocol }`. Becomes
//!   [`HarnessError::Agent`], the one arm a driver may put an unforeseen
//!   failure in.
//! - `acp-backend-profile.mjs` — the entry point. Becomes
//!   [`DownstreamAgent::open`].
//!
//! # Three properties, enforced by types
//!
//! **A descriptor is validated or it does not exist.**
//! [`HarnessDescriptor::declare`] is the only constructor and it runs every
//! rule, so `docs/architecture.md` §6's *"rejected at startup, not discovered
//! mid-turn"* is not a convention a driver can forget.
//!
//! **A cancelled turn cannot be reported as a completed one.**
//! [`PromptOutcome::new`] refuses [`StopReason::Cancelled`], and
//! [`CancelOutcome::Requested`] projects to
//! [`WorkStatus::Cancelling`](via_protocol::WorkStatus::Cancelling) with no
//! path to `Cancelled` that does not go through
//! [`CancelOutcome::confirm`].
//!
//! **The progress surface has nowhere to put a secret.**
//! [`RawSessionUpdate`] carries no session id, no sub-agent id, no permission
//! payload and no reasoning; [`PlanActivity`] and [`ToolActivity`] have private
//! fields and bounding constructors; every update kind not named in
//! [`event`] projects to `None`.
//!
//! # Fidelity
//!
//! Every value that reproduces an upstream literal names the file it came from
//! in its own documentation, and `docs/reference/contracts.json` is the
//! acceptance spec — `tests/contracts.rs` parses it rather than retyping it.
//! Every deviation from an upstream behaviour is documented on the item that
//! carries it, and belongs in `docs/deviations/phase-2.md` alongside the rest
//! of the phase.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod agent;
pub mod cancel;
pub mod capability;
pub mod descriptor;
pub mod error;
pub mod event;
pub mod health;
pub mod prompt;
pub mod registry;
pub mod session_key;
pub mod text;

#[cfg(feature = "testing")]
pub mod testing;

pub use agent::{DownstreamAgent, HarnessSession};
pub use cancel::{
    CancelConfirmationError, CancelOutcome, CancelRoute, CancelScope, CancelTarget,
    STATUS_NOT_FOUND, TerminalState,
};
pub use capability::{BackendCapabilities, CAPABILITY_FLAGS, CapabilityFault};
pub use descriptor::HarnessDescriptor;
pub use error::{HarnessError, SESSION_BUSY_STATUS};
pub use event::{
    ActivityStatus, ActivityTracker, DEFAULT_TOOL_NAME, DETAIL_BOUND, PLAN_ACTIVITY_ID,
    PlanActivity, PlanEntry, PlanEntryStatus, RawSessionUpdate, SessionEvent, TOOL_LABEL_BOUND,
    TOOL_NAME_BOUND, ToolActivity, ToolCategory, UPDATE_AGENT_MESSAGE_CHUNK, UPDATE_PLAN,
    UPDATE_TOOL_CALL, UPDATE_TOOL_CALL_UPDATE,
};
pub use health::{HarnessHealth, HarnessStatus, HarnessStatusCode};
pub use prompt::{PromptAttachment, PromptOutcome, PromptRequest, StopReason};
pub use registry::HarnessRegistry;
pub use session_key::{
    COORDINATOR_KEY_SUFFIX, DEFAULT_OWNER_ID, KEY_SEPARATOR, SessionKey, SessionKeyScope,
};
