//! The five coordination tools VIA serves to backend agents.
//!
//! A backend agent answers VIA's requests. This crate is how it asks VIA back:
//! *what project Sessions exist*, *start one*, *continue one*, *how is it
//! doing*, *stop it*. Five tools, served over MCP, and nothing else.
//!
//! | Module | What | Upstream |
//! | --- | --- | --- |
//! | [`tools`] | the five names, titles, descriptions and schemas — and the one array | `server/src/agent/acp-session-tools.mjs:9-131` |
//! | [`context`] | what the tools are wired to, and every shape that crosses | `acp-session-tools.mjs` + `acp-backend-adapter.mjs:696-938` |
//! | [`envelope`] | `jsonResult` / `errorResult` | `acp-session-tools.mjs:18-30` |
//! | [`protocol`] | the MCP JSON-RPC dispatch | the SDK upstream delegates to |
//! | [`registry`] | token → context, as an owning task | `acp-session-tools.mjs:143,180-209` |
//! | [`server`] | loopback HTTP: bind, authenticate, 404 | `acp-session-tools.mjs:133-262` |
//! | [`stdio`] | `via mcp-serve` | *(new)* — `docs/architecture.md` §10, §12 |
//! | [`builtin`] | the baseline computer-use MCP descriptor | `server/src/agent/builtin-mcp.mjs:1-76` |
//! | [`lifecycle`] | reaping its app-agents | `builtin-mcp.mjs:78-169` |
//! | [`testing`] | a recording double | *(new)* |
//!
//! # The rename that wedges the coordinator
//!
//! `docs/architecture.md` §13 calls the five tool names one of the two renames
//! in the whole port that carry real risk, because a **permission broker**
//! auto-approves them by three different name shapes — exact,
//! `endsWith("__<name>")`, `startsWith("<name> (")`. Rename the registration
//! and not the matcher and every internal coordination call becomes a
//! permission prompt with nobody there to answer it.
//!
//! So there is exactly one array, [`SESSION_TOOL_NAMES`], and everything reads
//! it: the tool definitions served over MCP, and [`match_session_tool`] —
//! **which is where the broker's carve-out lives**. The broker itself belongs
//! to whoever owns `session/request_permission`; it must call
//! [`is_session_tool`] rather than restate the shapes, and this crate is the
//! single source of truth it calls into.
//!
//! ```
//! use via_mcp_tools::{SESSION_TOOL_NAMES, SessionToolMatch, is_session_tool, match_session_tool};
//!
//! // The registration and the allow-list are the same five strings.
//! assert_eq!(SESSION_TOOL_NAMES[1], "via_session_start");
//!
//! // All three shapes a backend may present a name in are auto-approved.
//! assert!(is_session_tool("via_session_start"));
//! assert!(is_session_tool("mcp__via__via_session_start"));
//! assert!(is_session_tool("via_session_start (build the thing)"));
//! assert!(!is_session_tool("bash"));
//!
//! assert_eq!(
//!     match_session_tool("mcp__via__via_session_start").map(|(_, shape)| shape),
//!     Some(SessionToolMatch::NamespacePrefixed),
//! );
//! ```
//!
//! # The delegation contract
//!
//! `via_session_start` and `via_session_send` are **asynchronous** and answer
//! with an opaque delegation id and `status: "started"`. After that the
//! backend agent's turn is over. It does not poll, does not repeat the work
//! and does not answer from its own context: the adapter owns waiting,
//! cancellation, permission routing and result correlation.
//! [`DEFAULT_SESSION_INSTRUCTIONS`] is the sentence that says so to the model,
//! and it lives beside the tool names it interpolates.
//!
//! `via_session_status` is **observational only**. When it fails the backend
//! reports the failure; it does not substitute its own file tools against the
//! target directory.
//!
//! # The transport
//!
//! Loopback HTTP, exactly as upstream: `127.0.0.1:0`, `POST /mcp`,
//! `Authorization: Bearer <uuid>`, and **404 with an empty body** for any
//! other path or an unknown token. The token travels in a header on the ACP
//! descriptor and never in the URL. See [`server`] for why each of those is
//! the way it is.
//!
//! ```no_run
//! use std::sync::Arc;
//! use via_mcp_tools::{SessionToolServer, testing::RecordingContext};
//!
//! # async fn example() -> Result<(), via_mcp_tools::McpToolsError> {
//! let server = SessionToolServer::new();
//! let registration = server.register(Arc::new(RecordingContext::new())).await?;
//!
//! // Straight into `session/new`'s `mcpServers`.
//! let descriptor = registration.descriptor().clone();
//!
//! registration.release().await;
//! server.close().await;
//! # Ok(())
//! # }
//! ```
//!
//! # Fidelity
//!
//! Every model-visible string here is an external contract catalogued in
//! `docs/reference/contracts.json`, and `tests/contracts.rs` asserts each one
//! against the catalogue rather than against a retyped copy. The identity
//! substitutions are `docs/rebrand.md`'s and only those: the tool names and
//! the MCP server name are ours and are renamed;
//! [`COMPUTER_USE_PACKAGE`](builtin::COMPUTER_USE_PACKAGE) and
//! [`COMPUTER_USE_SERVER_NAME`](builtin::COMPUTER_USE_SERVER_NAME) name
//! somebody else's npm package and are KEEP.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod builtin;
pub mod context;
pub mod envelope;
pub mod error;
pub mod lifecycle;
pub mod protocol;
pub mod registry;
pub mod server;
pub mod stdio;
pub mod tools;

#[cfg(feature = "testing")]
pub mod testing;

pub use builtin::{
    BUILTIN_MCP_LIFECYCLE, BuiltinMcpKind, BuiltinMcpServer, COMPUTER_USE_ENV,
    COMPUTER_USE_MCP_ARGUMENT, COMPUTER_USE_PACKAGE, COMPUTER_USE_SERVER_NAME, ComputerUseLaunch,
    ComputerUseLocator, DISABLED_VALUES, ELECTRON_RUN_AS_NODE, FixedLocator, NodePackageLocator,
    PathLocator, builtin_mcp_servers, builtin_mcp_servers_for, computer_use_enabled,
    computer_use_mcp_server, descriptors, externally_readable, resolve_package_bin,
    setting_enabled,
};
pub use context::{
    DEFAULT_SESSION_LIST_LIMIT, DelegationLookupInput, DelegationOutcome, DelegationRecord,
    DelegationStarted, DelegationStatus, InvalidArguments, SESSION_LIST_LIMIT_MAX,
    SESSION_LIST_LIMIT_MIN, SESSION_TITLE_BOUND, STATUS_RESULT_BOUND, STATUS_STARTED,
    SchemaViolation, SessionSendInput, SessionStartInput, SessionStatusResult, SessionSummary,
    SessionToolContext, SessionsListInput, SessionsListResult, clip, matches_query,
};
pub use envelope::{
    CONTENT_TYPE_TEXT, STATUS_FAILED, envelope_text, error_result, is_error_envelope, json_result,
    text_result,
};
pub use error::McpToolsError;
pub use lifecycle::{
    APP_AGENT_DISCOVERY, APP_AGENT_MARKER, APP_AGENT_POLL, BuiltinMcpLifecycle, LifecycleOptions,
    PS_ARGUMENTS, ProcessEntry, ProcessTable, SystemProcessTable, TERMINATION_GRACE,
    TerminationSignal, parse_process_line, parse_process_table,
};
pub use protocol::{
    JSONRPC_VERSION, LATEST_PROTOCOL_VERSION, METHOD_NOT_ALLOWED_MESSAGE,
    SUPPORTED_PROTOCOL_VERSIONS, dispatch, error_response, initialize_result, internal_error_body,
    method_not_allowed_body, negotiate_protocol_version, response,
};
pub use registry::{REGISTRY_QUEUE_DEPTH, RegistryHandle, SharedContext};
pub use server::{
    AUTHORIZATION_HEADER, LOOPBACK_HOST, MAX_MESSAGE_BYTES, MCP_PATH, SessionToolRegistration,
    SessionToolServer, bearer_token,
};
pub use stdio::serve_stdio;
pub use tools::{
    DEFAULT_SESSION_INSTRUCTIONS, SESSION_TOOL_NAMES, SESSION_TOOL_SERVER,
    SESSION_TOOL_SERVER_VERSION, SessionTool, SessionToolMatch, is_session_tool,
    match_session_tool, tool_definitions, tools_list_result,
};
