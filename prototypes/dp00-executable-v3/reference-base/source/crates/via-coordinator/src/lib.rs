//! `via-coordinator` — the delegation envelope, and everything that reads the
//! reply.
//!
//! Layer 2's other half. [`via_work`] owns *what* work exists and when it runs;
//! this crate owns the one conversation VIA has with a backend agent about it:
//! what the agent is told, in what envelope, over which session, with what
//! serialization in front of it, and how the answer is read back.
//!
//! Ported from `qwen-audio-agent` v1.11.0:
//! `server/src/agent/{coordinator,backend-agent-instructions,permission-broker,keyed-serial-executor}.mjs`
//! and the coordination half of `server/src/agent/acp-backend-adapter.mjs` —
//! the transport half is `via-acp`'s.
//!
//! | Module | What | Upstream |
//! | --- | --- | --- |
//! | [`instructions`] | the fifteen lines, verbatim, and their wrapper | `backend-agent-instructions.mjs`, `acp-backend-adapter.mjs:940-955` |
//! | [`envelope`] | `via.coordination.v1` and the prompt around it | `coordinator.mjs:143-237` |
//! | [`decision`] | the two response shapes, and reading one back | `coordinator.mjs:7-118` |
//! | [`prompts`] | the retry, delegation-result, reconciliation and control blocks | `coordinator.mjs:270-276`, `acp-backend-adapter.mjs:1007-1017,1139-1155,1388-1443` |
//! | [`executor`] | the keyed serial executor, as an owning task | `keyed-serial-executor.mjs` |
//! | [`permission`] | pending, resolved, scoped, and the auto-approve carve-out | `permission-broker.mjs` |
//! | [`native`] | spotting a backend that delegated on its own | `acp-backend-adapter.mjs:651-694` |
//! | [`delegation`] | the delegations one coordinator owns | `acp-backend-adapter.mjs:696-938` |
//! | [`profile`] | what the coordinator needs to know about the backend | `acp-backend-adapter.mjs:133-232` |
//! | [`runtime`] | the fixed session, the turn ladder, the lifecycle | `coordinator.mjs:239-318`, `acp-backend-adapter.mjs:989-1271` |
//! | [`intent`] | `docs/architecture.md` §4's IntentSet fan-out | *(new)* |
//!
//! # The five things this crate is about
//!
//! **One fixed session per owner and backend.**
//! [`SessionKey::coordinator`](via_downstream::SessionKey::coordinator) is
//! *"THE fixed identity that survives voice sessions, Work IDs, and Gateway
//! restarts"*. A new voice conversation continues in the same backend context;
//! nothing about a turn can change which session it lands in.
//!
//! **Two guards, not one.** `docs/architecture.md` §11: the Gateway queue
//! ([`via_work::coordinator_lane`]) bounds admission and
//! [`executor::KeyedSerialExecutor`] bounds session writes. The double guard is
//! deliberate — the hidden control turns in [`prompts`] never go through the
//! Work queue at all, and they write to the same session.
//!
//! **The delegated reply is not a completion.**
//! `docs/reference/contracts.json` is blunt about it: *"It is NEVER a
//! user-visible completion; the adapter treats it as a lock-release signal."*
//! [`Coordinator::run_coordinator`] lets the turn finish, announces the
//! delegation, **drops the lane**, waits outside it, and only then takes one
//! more turn to compose what the user actually hears.
//!
//! **Reading a reply is an exact algorithm.**
//! [`via_acp::parse_coordinator_payload`](via_acp::session::parse_coordinator_payload)
//! — three iterations, a `json` fence, a double-encoded string, a
//! first-`{`-to-last-`}` narrowing and a no-progress guard. It is `via-acp`'s
//! and is called, never copied: *"an off-by-one in either the count or the
//! no-progress guard changes which outputs are accepted."*
//!
//! **The permission carve-out is imported.** [`via_mcp_tools::is_session_tool`]
//! and its three matcher shapes. `docs/architecture.md` §13 calls a partial
//! rename here one of the two renames in the port that wedge the coordinator,
//! so [`permission::PermissionBroker`] has no list of its own.
//!
//! # What VIA does not tell the backend
//!
//! The envelope carries the user's request, recent voice context, the user's
//! preferences and memory, the Work in flight, and a final response shape. It
//! carries **no instruction about how to use the backend's own capabilities** —
//! which tools to reach for, how to plan, whether to sub-agent. The backend
//! owns its execution strategy; VIA owns routing, permissions, cancellation and
//! delivery.
//!
//! # Example
//!
//! ```no_run
//! use std::sync::Arc;
//! use chrono::Utc;
//! use via_coordinator::{Coordinator, CoordinationRequest, CoordinatorProfile, TurnOptions};
//! use via_downstream::testing::{ScriptedHarness, ScriptedTurn};
//! use via_i18n::Locale;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let harness = ScriptedHarness::builder("opencode")
//!     .turn(ScriptedTurn::completed(
//!         r#"{"work_id":"work_1","state":"completed","mode":"respond",
//!             "presentation":{"speech":"Done.","inline":null}}"#,
//!     ))
//!     .build()?;
//!
//! let coordinator = Coordinator::builder(
//!     Arc::new(harness),
//!     CoordinatorProfile::default_for("opencode", "OpenCode", Locale::En),
//! )
//! .locale(Locale::En)
//! .build();
//!
//! let outcome = coordinator
//!     .run(
//!         &CoordinationRequest {
//!             original_request: "summarise the diff",
//!             objective: "summarise the diff",
//!             coordination_run_id: "work_1",
//!             ..CoordinationRequest::default()
//!         },
//!         Utc::now(),
//!         &TurnOptions::new("ana", "work_1"),
//!     )
//!     .await?;
//! assert_eq!(outcome.content(), "Done.");
//! # Ok(())
//! # }
//! ```
//!
//! # Fidelity
//!
//! Every value that reproduces an upstream literal names the file it came from
//! in its own documentation, and `docs/reference/contracts.json` is the
//! acceptance spec — `tests/contracts.rs` parses it rather than retyping it.
//! Deviations are recorded in `docs/deviations/phase-3.md`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod decision;
pub mod delegation;
pub mod envelope;
pub mod error;
pub mod executor;
pub mod instructions;
pub mod intent;
pub mod native;
pub mod permission;
pub mod profile;
pub mod prompts;
pub mod runtime;

#[cfg(feature = "testing")]
pub mod testing;

pub use decision::{
    CoordinatorDecision, DecisionMode, DecisionState, coordinator_decision_schema,
    coordinator_response_state, is_deliverable_state, normalize_inline, normalize_presentation,
    parse_coordinator_decision,
};
pub use delegation::{
    DelegationRecord, DelegationRegistry, coordinator_session_lane, new_delegation_id,
    new_permission_scope_id, target_lane,
};
pub use envelope::{
    COORDINATION_PROTOCOL, CoordinationRequest, Delivery, EnvelopeAttachment, TrustedBackendEvent,
    build_coordinator_prompt, coordination_envelope, expected_json_example,
};
pub use error::CoordinatorError;
pub use executor::{ExecutorStopped, KeyedSerialExecutor, LanePermit};
pub use instructions::{
    BACKEND_AGENT_INSTRUCTIONS, BACKEND_INSTRUCTIONS_CLOSE_TAG, BACKEND_INSTRUCTIONS_OPEN_TAG,
    coordinator_instructions,
};
pub use intent::{Intent, IntentSet, IntentSetError, IntentSubmission};
pub use native::{
    NativeDelegation, NativeDelegationDefaults, NativeDelegationDetector, NativeToolUpdate,
    is_delegation_tool,
};
pub use permission::{
    PermissionBroker, PermissionContext, PermissionMode, PermissionObserver, PermissionResponse,
};
pub use profile::CoordinatorProfile;
pub use prompts::{
    DelegationResult, MAX_DELEGATION_RESULT_CHARS, cancel_control_prompt, delegation_result_prompt,
    protocol_retry_prompt, reconciliation_prompt, status_control_prompt,
};
pub use runtime::{
    BackendRef, CoordinationObserver, CoordinationOutcome, Coordinator, CoordinatorBuilder,
    EmptySessionDirectory, NativeDelegationAdapter, ProjectSessionDirectory, ResultEnvelope,
    TurnOptions,
};
