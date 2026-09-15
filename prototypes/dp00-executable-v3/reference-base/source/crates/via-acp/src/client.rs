//! The ACP process client.
//!
//! Ported from `server/src/agent/acp-process-client.mjs` in full: the process
//! lifecycle, the ten catalogued JSON-RPC methods, the session table whose
//! entries keep their identity across a resume, the pausable prompt deadline,
//! and the error wrapping that reaches API clients.
//!
//! # What the SDK owns, and what VIA owns
//!
//! `agent-client-protocol` frames NDJSON over the child's stdio, correlates
//! requests with responses, dispatches inbound notifications and requests, and
//! auto-cancels a request whose future is dropped. None of that is re-written
//! here — [`crate::methods`] asserts that VIA and the SDK agree on every method
//! name, and everything below goes through `ConnectionTo<Agent>`.
//!
//! Three things the SDK does not do, which are this module's:
//!
//! 1. **Spawning.** [`crate::process`] spawns, because the SDK's own spawn
//!    inherits the parent environment and would carry Gateway secrets across
//!    the credential boundary.
//! 2. **Teardown.** The SDK goes straight to `SIGKILL` after a fixed one-second
//!    grace; VIA runs `SIGTERM` → poll → `SIGKILL` through `process-wrap`.
//! 3. **A long-lived client object.** The SDK's connection is scope-shaped —
//!    `connect_with(transport, |cx| async { … })` ends the connection when the
//!    closure returns. Upstream's client is an object with `start`, `prompt`
//!    and `close` callable from anywhere, so the closure here parks on a
//!    shutdown channel and publishes its `ConnectionTo<Agent>` to the outside.
//!    That is `docs/architecture.md` §11's owning-task shape: the connection
//!    lives in one task, and everyone else holds a handle.
//!
//! # The client declares no client capabilities
//!
//! [`CLIENT_CAPABILITIES_ARE_EMPTY`] — the Gateway advertises no `fs/*` and no
//! `terminal/*`, so an agent must not attempt them. That is catalogued under
//! *"initialize"* and it is a deliberate narrowing: file and terminal access
//! belong to the backend agent's own sandbox, not to a bridge that would
//! perform them with the Gateway's privileges.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use agent_client_protocol::schema::v1::{
    AgentCapabilities, ClientCapabilities, CloseSessionRequest, ContentBlock, Implementation,
    InitializeRequest, InitializeResponse, ListSessionsRequest, LoadSessionRequest, McpServer,
    Meta, NewSessionRequest, PromptRequest as AcpPromptRequest, PromptResponse,
    ResumeSessionRequest, SessionId, SetSessionConfigOptionRequest,
};
use agent_client_protocol::schema::{ProtocolVersion, v1::CancelNotification};
use agent_client_protocol::{
    Agent, Client, ConnectionTo, JsonRpcNotification, JsonRpcRequest, Lines, SentRequest,
    is_incoming_transport_closed, on_receive_notification, on_receive_request,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt as _, AsyncReadExt as _, AsyncWriteExt as _};
use tokio::sync::{Mutex as AsyncMutex, oneshot};
use via_i18n::Locale;

use crate::cancel::{CancelSignal, PausableTimeout};
use crate::content::{Prompt, assert_prompt_capabilities, normalize_acp_prompt};
use crate::error::{AcpError, Result};
use crate::limits;
use crate::methods;
use crate::permission::{PermissionDecision, PermissionRequest, reply as permission_reply};
use crate::process::{AcpChildHandle, SpawnSpec};
use crate::session::text_from_update;

/// The `clientInfo.name` every backend agent sees.
///
/// **External contract** — `acp-process-client.mjs:173,219`, renamed per
/// `docs/rebrand.md` ("qwen-audio-agent (ACP client identity)" → `via`). Some
/// backends log it, and some display it.
pub const CLIENT_NAME: &str = "via";

/// The `clientInfo.title` every backend agent sees.
///
/// **External contract** — `acp-process-client.mjs:220`, renamed per
/// `docs/rebrand.md` ("qwen-audio-agent Gateway" → `VIA Gateway`).
pub const CLIENT_TITLE: &str = "VIA Gateway";

/// The `clientInfo.version` sent on `initialize`.
pub const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The client capabilities VIA declares — deliberately none.
///
/// **External contract** — *"initialize"*: `clientCapabilities` is empty on
/// purpose, so ACP agents must not attempt `fs/*` or `terminal/*`. File and
/// terminal access belong to the backend agent's own sandbox, not to a bridge
/// that would perform them with the Gateway's privileges.
///
/// VIA sends the SDK's `ClientCapabilities::default()`, in which every flag is
/// false and the `session` extension is absent — semantically identical to
/// upstream's `{}`, and byte-different only in that the SDK spells the two
/// false `fs` flags out. Recorded in `docs/deviations/phase-2.md`.
#[must_use]
pub fn client_capabilities() -> ClientCapabilities {
    ClientCapabilities::default()
}

/// Whether a capability declaration promises the agent nothing.
///
/// The predicate the contract is actually about, so that a future SDK release
/// that flipped a default fails here rather than silently inviting an agent to
/// call `fs/read_text_file`.
#[must_use]
pub fn declares_no_capability(capabilities: &ClientCapabilities) -> bool {
    !capabilities.fs.read_text_file
        && !capabilities.fs.write_text_file
        && !capabilities.terminal
        && capabilities.session.is_none()
}

/// The protocol version VIA speaks.
///
/// Stable **v1** (`docs/architecture.md` §12). Protocol v2 is a draft; the SDK
/// keeps it behind a feature which this crate does not enable, so `V2` is not
/// even constructible here.
pub const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::V1;

/// `session/update`, read as the raw notification it is.
///
/// The SDK's typed `SessionNotification` would drop the undeclared fields real
/// backends send — `name` on a tool call above all — so the client dispatches
/// on this and hands observers the raw `update`. See [`crate::session`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonRpcNotification)]
#[notification(method = "session/update")]
#[serde(rename_all = "camelCase")]
pub struct RawSessionNotification {
    /// Which session the update belongs to.
    #[serde(default)]
    pub session_id: Option<String>,
    /// The update itself, verbatim.
    #[serde(default)]
    pub update: Option<Value>,
}

/// `session/request_permission`, read as the raw request it is.
#[derive(Debug, Clone, Serialize, Deserialize, JsonRpcRequest)]
#[request(method = "session/request_permission", response = serde_json::Value)]
#[serde(rename_all = "camelCase")]
pub struct RawPermissionRequest {
    /// Which session is asking.
    #[serde(default)]
    pub session_id: Option<String>,
    /// The tool call, verbatim.
    #[serde(default)]
    pub tool_call: Option<Value>,
    /// The options offered.
    #[serde(default)]
    pub options: Vec<agent_client_protocol::schema::v1::PermissionOption>,
}

/// `session/set_model`, the legacy model path.
///
/// **External contract** — *"session/set_model"*: the method name is a
/// hard-coded literal upstream because the SDK has no such method, and the
/// params are `{sessionId, modelId}` with both coerced to strings.
#[derive(Debug, Clone, Serialize, Deserialize, JsonRpcRequest)]
#[request(method = "session/set_model", response = serde_json::Value)]
#[serde(rename_all = "camelCase")]
pub struct SetModelRequest {
    /// The session to retarget.
    pub session_id: String,
    /// The model id to select.
    pub model_id: String,
}

/// What a session is for.
///
/// **Contract** — `acp-process-client.mjs:362,384` (`role = 'project'`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionRole {
    /// One of the many delegated project sessions. The default.
    #[default]
    Project,
    /// The single long-lived session per owner.
    Coordinator,
}

/// What the client remembers about one ACP session.
///
/// **Contract** — `rememberSession`, `acp-process-client.mjs:345-355`. The
/// record is **merged in place** and its identity is stable: the adapter keeps
/// runtime routing fields on the same object, and replacing it on every resume
/// would hand a permission request a stale closure and route the answer to a
/// finished task. [`SharedSession`] is that identity, made explicit.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SessionRecord {
    /// The backend's own session id.
    pub session_id: String,
    /// The working directory the session was opened against.
    pub cwd: Option<std::path::PathBuf>,
    /// The `_meta` sent with `session/new` or `session/resume`.
    pub meta: Option<Meta>,
    /// Whose session this is.
    pub owner_id: Option<String>,
    /// Coordinator or project.
    pub role: SessionRole,
    /// The MCP servers declared for it.
    pub mcp_servers: Vec<McpServer>,
    /// The last `session/new` / `session/resume` / `session/load` response.
    pub response: Option<Value>,
    /// Fields the layer above keeps on this record — upstream's `onEvent`,
    /// `ownerId`, `coordinationRunId`. Opaque here on purpose: this crate does
    /// not know what a Work id is.
    pub routing: serde_json::Map<String, Value>,
}

/// A session record whose identity survives re-registration.
pub type SharedSession = Arc<Mutex<SessionRecord>>;

/// The fields a caller supplies when remembering a session. `None` leaves the
/// existing value in place — this is the merge, not a replacement.
#[derive(Debug, Clone, Default)]
pub struct SessionDetails {
    /// The working directory.
    pub cwd: Option<std::path::PathBuf>,
    /// The `_meta` object.
    pub meta: Option<Meta>,
    /// The owner.
    pub owner_id: Option<String>,
    /// Coordinator or project.
    pub role: Option<SessionRole>,
    /// The MCP servers.
    pub mcp_servers: Option<Vec<McpServer>>,
    /// The response that produced this record.
    pub response: Option<Value>,
}

/// What a completed prompt turn produced.
#[derive(Debug, Clone, PartialEq)]
pub struct PromptTurn {
    /// The agent's text for the turn: every `agent_message_chunk` text block,
    /// joined and trimmed once at the end.
    pub content: String,
    /// The `session/prompt` response. Never `stopReason: "cancelled"` — that is
    /// [`AcpError::Cancelled`] instead.
    pub response: PromptResponse,
}

/// Per-turn options.
#[derive(Default)]
pub struct PromptOptions {
    /// The caller's cancellation signal, composed with the client's deadline.
    pub signal: Option<CancelSignal>,
    /// Override the client's default deadline. `Some(None)` is upstream's
    /// `timeoutMs: 0` — no deadline at all, the caller owns it.
    pub timeout: Option<Option<Duration>>,
    /// A per-turn update observer, called before the client-wide one.
    pub on_update: Option<UpdateObserver>,
}

impl std::fmt::Debug for PromptOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PromptOptions")
            .field("signal", &self.signal)
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

/// What a `session/request_permission` handler is.
///
/// Returning a future rather than answering inline is deliberate: the SDK's
/// dispatch loop is blocked while a handler runs, and a permission prompt with
/// nobody at the keyboard can take minutes.
pub type PermissionHandler = Arc<
    dyn Fn(PermissionRequest) -> futures::future::BoxFuture<'static, PermissionDecision>
        + Send
        + Sync,
>;

/// A `session/update` observer: the session id and the raw update.
pub type UpdateObserver = Arc<dyn Fn(&str, &Value) + Send + Sync>;

/// A hook that rewrites the child's stdout/stderr before it is stored or shown.
///
/// Backend-specific behaviour delivered as data — this crate names no backend,
/// so a profile supplies the function.
pub type SanitizeOutput = Arc<dyn Fn(&str) -> String + Send + Sync>;

/// A hook that rewrites a JSON-RPC failure into a sentence a person can act on.
///
/// **Contract** — `requestErrorMessage(error, formatRequestError)`,
/// `acp-process-client.mjs:38-46`. Returning `None` (upstream: an empty string)
/// falls through to the default composition.
pub type FormatRequestError = Arc<dyn Fn(&RequestErrorContext) -> Option<String> + Send + Sync>;

/// What a [`FormatRequestError`] hook is given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestErrorContext {
    /// The JSON-RPC error code.
    pub code: i32,
    /// The error's message, trimmed.
    pub message: String,
    /// `data.details`, else `data` when it is a string; trimmed, else empty.
    pub details: String,
}

/// `requestErrorDetails(error)` — `acp-process-client.mjs:29-36`.
#[must_use]
fn request_error_details(error: &agent_client_protocol::Error) -> String {
    if let Some(details) = error
        .data
        .as_ref()
        .and_then(|data| data.get("details"))
        .and_then(Value::as_str)
        .filter(|details| !details.trim().is_empty())
    {
        return details.trim().to_owned();
    }
    error
        .data
        .as_ref()
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|data| !data.is_empty())
        .unwrap_or_default()
        .to_owned()
}

/// `requestErrorMessage(error, formatRequestError)` —
/// `acp-process-client.mjs:38-46`.
///
/// The fullwidth parentheses in the fallback are upstream's and are catalogued;
/// they only appear when the details are not already inside the message, which
/// is what keeps a backend that already explains itself from saying it twice.
fn request_error_message(
    error: &agent_client_protocol::Error,
    format_request_error: Option<&FormatRequestError>,
) -> String {
    let message = error.message.trim().to_owned();
    let details = request_error_details(error);
    let context = RequestErrorContext {
        code: error.code.into(),
        message: message.clone(),
        details: details.clone(),
    };
    if let Some(formatted) = format_request_error
        .and_then(|hook| hook(&context))
        .map(|formatted| formatted.trim().to_owned())
        .filter(|formatted| !formatted.is_empty())
    {
        return formatted;
    }
    if !details.is_empty() && !message.contains(&details) {
        return std::format!("{message}（{details}）");
    }
    message
}

/// Node's `util.stripVTControlCharacters`.
///
/// **Contract** — `cleanProcessOutput`, `acp-process-client.mjs:24-27`. A
/// backend that paints its startup banner must not have the paint end up inside
/// an error message a person reads, or inside a log line.
#[must_use]
pub fn strip_vt_control_characters(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\u{1b}' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            // CSI: parameters and intermediates, then a final byte in @..~.
            Some('[') => {
                for next in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&next) {
                        break;
                    }
                }
            }
            // OSC: runs to BEL or to ST (ESC \).
            Some(']') => {
                while let Some(next) = chars.next() {
                    if next == '\u{7}' {
                        break;
                    }
                    if next == '\u{1b}' && chars.peek() == Some(&'\\') {
                        chars.next();
                        break;
                    }
                }
            }
            // Any other two-character escape is dropped whole.
            Some(_) | None => {}
        }
    }
    out
}

/// Builder for [`AcpProcessClient`].
#[derive(Default)]
pub struct AcpProcessClientBuilder {
    label: String,
    spec: Option<SpawnSpec>,
    timeout: Duration,
    locale: Locale,
    on_permission: Option<PermissionHandler>,
    on_update: Option<UpdateObserver>,
    sanitize_process_output: Option<SanitizeOutput>,
    format_request_error: Option<FormatRequestError>,
}

impl std::fmt::Debug for AcpProcessClientBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcpProcessClientBuilder")
            .field("label", &self.label)
            .field("spec", &self.spec)
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

impl AcpProcessClientBuilder {
    /// The backend's display label, interpolated into every message.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// How the child is started.
    #[must_use]
    pub fn spawn_spec(mut self, spec: SpawnSpec) -> Self {
        self.spec = Some(spec);
        self
    }

    /// The default per-request deadline. Defaults to [`limits::DEFAULT_TIMEOUT`].
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// The locale error sentences are rendered in.
    #[must_use]
    pub fn locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// Install the permission handler. Without one every request is answered
    /// `cancelled`.
    #[must_use]
    pub fn on_permission(mut self, handler: PermissionHandler) -> Self {
        self.on_permission = Some(handler);
        self
    }

    /// Install the client-wide update observer.
    #[must_use]
    pub fn on_update(mut self, observer: UpdateObserver) -> Self {
        self.on_update = Some(observer);
        self
    }

    /// Install the output sanitizer.
    #[must_use]
    pub fn sanitize_process_output(mut self, sanitize: SanitizeOutput) -> Self {
        self.sanitize_process_output = Some(sanitize);
        self
    }

    /// Install the request-error formatter.
    #[must_use]
    pub fn format_request_error(mut self, format: FormatRequestError) -> Self {
        self.format_request_error = Some(format);
        self
    }

    /// Finish the client.
    ///
    /// Nothing is spawned yet: upstream constructs the client eagerly and
    /// starts the process on the first request, so that a Gateway whose backend
    /// is not installed still boots and still reports why.
    #[must_use]
    pub fn build(self) -> AcpProcessClient {
        let timeout = if self.timeout.is_zero() {
            limits::DEFAULT_TIMEOUT
        } else {
            self.timeout
        };
        AcpProcessClient {
            inner: Arc::new(Inner {
                label: self.label,
                spec: self.spec.unwrap_or_else(|| SpawnSpec::new(String::new())),
                timeout,
                locale: self.locale,
                on_permission: self.on_permission,
                on_update: self.on_update,
                sanitize_process_output: self.sanitize_process_output,
                format_request_error: self.format_request_error,
                stderr: Mutex::new(String::new()),
                sessions: Mutex::new(HashMap::new()),
                active_prompts: Mutex::new(HashMap::new()),
                connection: AsyncMutex::new(None),
            }),
        }
    }
}

struct ActivePrompt {
    text: Mutex<Vec<String>>,
    timer: PausableTimeout,
    on_update: Option<UpdateObserver>,
}

struct Connection {
    cx: ConnectionTo<Agent>,
    initialize: InitializeResponse,
    child: Arc<AsyncMutex<AcpChildHandle>>,
    shutdown: Option<oneshot::Sender<()>>,
    task: tokio::task::JoinHandle<()>,
}

struct Inner {
    label: String,
    spec: SpawnSpec,
    timeout: Duration,
    locale: Locale,
    on_permission: Option<PermissionHandler>,
    on_update: Option<UpdateObserver>,
    sanitize_process_output: Option<SanitizeOutput>,
    format_request_error: Option<FormatRequestError>,
    stderr: Mutex<String>,
    sessions: Mutex<HashMap<String, SharedSession>>,
    active_prompts: Mutex<HashMap<String, Arc<ActivePrompt>>>,
    connection: AsyncMutex<Option<Connection>>,
}

/// One ACP agent process, and the session table on top of it.
///
/// Cheap to clone; every clone is the same client and the same child process.
#[derive(Clone)]
pub struct AcpProcessClient {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for AcpProcessClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcpProcessClient")
            .field("label", &self.inner.label)
            .field("command", &self.inner.spec.command)
            .field("timeout", &self.inner.timeout)
            .finish_non_exhaustive()
    }
}

impl AcpProcessClient {
    /// Start building a client.
    #[must_use]
    pub fn builder() -> AcpProcessClientBuilder {
        AcpProcessClientBuilder::default()
    }

    /// The backend's display label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.inner.label
    }

    /// The default per-request deadline.
    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.inner.timeout
    }

    /// How the child is started.
    #[must_use]
    pub fn spawn_spec(&self) -> &SpawnSpec {
        &self.inner.spec
    }

    /// The bounded tail of the child's stderr.
    #[must_use]
    pub fn stderr(&self) -> String {
        self.inner.stderr()
    }

    /// Whether the process is up and initialized.
    ///
    /// **Contract** — `get ready()`, `acp-process-client.mjs:115-122`: an
    /// initialize result, a live connection, and a child that has neither
    /// exited nor been signalled.
    pub async fn is_ready(&self) -> bool {
        let guard = self.inner.connection.lock().await;
        guard.as_ref().is_some_and(|connection| {
            !connection.task.is_finished()
                && connection
                    .child
                    .try_lock()
                    .is_ok_and(|mut child| !child.has_exited())
        })
    }

    /// Start the process and complete `initialize`, or return the cached result.
    ///
    /// **Contract** — `start()`, `acp-process-client.mjs:131-145`. Concurrent
    /// callers share one initialization: the second caller waits on the same
    /// lock and then sees the cached result rather than spawning a second
    /// agent.
    ///
    /// # Errors
    ///
    /// [`AcpError::ProcessSpawnFailed`], [`AcpError::ProcessExited`],
    /// [`AcpError::InitializeFailed`] or
    /// [`AcpError::ProtocolVersionMismatch`].
    pub async fn start(&self) -> Result<InitializeResponse> {
        let mut guard = self.inner.connection.lock().await;
        if let Some(connection) = guard.as_ref()
            && !connection.task.is_finished()
        {
            return Ok(connection.initialize.clone());
        }
        if let Some(mut stale) = guard.take() {
            stale.shutdown.take();
            stale.child.lock().await.stop().await;
        }
        let connection = self.start_process().await?;
        let initialize = connection.initialize.clone();
        *guard = Some(connection);
        Ok(initialize)
    }

    /// The agent's declared capabilities, or the default when not started.
    ///
    /// **Contract** — `get capabilities()`, `acp-process-client.mjs:107-109`
    /// (`|| {}`): asking before `initialize` answers "declares nothing" rather
    /// than failing, so a capability gate reads the same either way.
    pub async fn capabilities(&self) -> AgentCapabilities {
        let guard = self.inner.connection.lock().await;
        guard
            .as_ref()
            .map(|connection| connection.initialize.agent_capabilities.clone())
            .unwrap_or_default()
    }

    /// The agent's self-description, once it has given one.
    pub async fn agent_info(&self) -> Option<Implementation> {
        let guard = self.inner.connection.lock().await;
        guard
            .as_ref()
            .and_then(|connection| connection.initialize.agent_info.clone())
    }

    /// Remember a session, **merging** into any existing record.
    ///
    /// **Contract** — `rememberSession`, `acp-process-client.mjs:345-355`.
    /// A blank id records nothing and returns `None`. The returned handle is
    /// the same `Arc` for the same id across every call, which is the identity
    /// upstream's comment is about.
    pub fn remember_session(
        &self,
        session_id: &str,
        details: SessionDetails,
    ) -> Option<SharedSession> {
        let id = session_id.trim();
        if id.is_empty() {
            return None;
        }
        let mut sessions = self.inner.lock_sessions();
        let record = sessions
            .entry(id.to_owned())
            .or_insert_with(|| Arc::new(Mutex::new(SessionRecord::default())));
        {
            let mut session = record
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(cwd) = details.cwd {
                session.cwd = Some(cwd);
            }
            if let Some(meta) = details.meta {
                session.meta = Some(meta);
            }
            if let Some(owner_id) = details.owner_id {
                session.owner_id = Some(owner_id);
            }
            if let Some(role) = details.role {
                session.role = role;
            }
            if let Some(mcp_servers) = details.mcp_servers {
                session.mcp_servers = mcp_servers;
            }
            if let Some(response) = details.response {
                session.response = Some(response);
            }
            // Assigned last and unconditionally, exactly as upstream's
            // `Object.assign(session, details, { sessionId: id })` does.
            session.session_id = id.to_owned();
        }
        Some(Arc::clone(record))
    }

    /// The record for `session_id`, if the client has one.
    #[must_use]
    pub fn session(&self, session_id: &str) -> Option<SharedSession> {
        self.inner.lock_sessions().get(session_id).map(Arc::clone)
    }

    /// How many sessions the client is tracking.
    #[must_use]
    pub fn session_count(&self) -> usize {
        self.inner.lock_sessions().len()
    }

    /// `session/new`.
    ///
    /// **Contract** — params `{cwd, mcpServers, _meta?}`, with `_meta` omitted
    /// entirely when absent.
    ///
    /// # Errors
    ///
    /// Anything [`Self::start`] raises, or [`AcpError::RequestFailed`].
    pub async fn new_session(
        &self,
        cwd: impl Into<std::path::PathBuf>,
        mcp_servers: Vec<McpServer>,
        meta: Option<Meta>,
        details: SessionDetails,
    ) -> Result<SharedSession> {
        let cwd = cwd.into();
        let mut request = NewSessionRequest::new(cwd.clone()).mcp_servers(mcp_servers.clone());
        if let Some(meta) = meta.clone() {
            request = request.meta(meta);
        }
        let response = self
            .request(
                methods::SESSION_NEW,
                request,
                None,
                Some(self.inner.timeout),
            )
            .await?;
        let session_id = response.session_id.0.to_string();
        self.remember_session(
            &session_id,
            SessionDetails {
                cwd: Some(cwd),
                meta,
                mcp_servers: Some(mcp_servers),
                response: serde_json::to_value(&response).ok(),
                ..details
            },
        )
        .ok_or_else(|| self.inner.process_gone())
    }

    /// `session/resume`, falling back to `session/load`.
    ///
    /// **Contract** — the capability gate order matters: `resume` wins over
    /// `load`, and an agent that declares neither raises
    /// [`AcpError::ResumeUnsupported`] rather than silently starting a new
    /// conversation. Both methods take identical params.
    ///
    /// # Errors
    ///
    /// [`AcpError::ResumeUnsupported`] when the agent supports neither, plus
    /// anything [`Self::start`] or the request itself raises.
    pub async fn resume_session(
        &self,
        session_id: &str,
        cwd: impl Into<std::path::PathBuf>,
        mcp_servers: Vec<McpServer>,
        meta: Option<Meta>,
        details: SessionDetails,
    ) -> Result<SharedSession> {
        self.start().await?;
        let cwd = cwd.into();
        let capabilities = self.capabilities().await;
        let id = SessionId::new(session_id.to_owned());

        let response: Value = if capabilities.session_capabilities.resume.is_some() {
            let mut request =
                ResumeSessionRequest::new(id, cwd.clone()).mcp_servers(mcp_servers.clone());
            if let Some(meta) = meta.clone() {
                request = request.meta(meta);
            }
            let response = self
                .request(
                    methods::SESSION_RESUME,
                    request,
                    None,
                    Some(self.inner.timeout),
                )
                .await?;
            serde_json::to_value(response).unwrap_or(Value::Null)
        } else if capabilities.load_session {
            let mut request =
                LoadSessionRequest::new(id, cwd.clone()).mcp_servers(mcp_servers.clone());
            if let Some(meta) = meta.clone() {
                request = request.meta(meta);
            }
            let response = self
                .request(
                    methods::SESSION_LOAD,
                    request,
                    None,
                    Some(self.inner.timeout),
                )
                .await?;
            serde_json::to_value(response).unwrap_or(Value::Null)
        } else {
            return Err(AcpError::ResumeUnsupported {
                label: self.inner.label.clone(),
            });
        };

        self.remember_session(
            session_id,
            SessionDetails {
                cwd: Some(cwd),
                meta,
                mcp_servers: Some(mcp_servers),
                response: Some(response),
                ..details
            },
        )
        .ok_or_else(|| self.inner.process_gone())
    }

    /// `session/list`, paginated.
    ///
    /// **Contract** — the page size lives inside `_meta.limit` and is
    /// `min(100, max(1, limit - collected))`; the loop continues while a
    /// `nextCursor` is returned and fewer than `limit` sessions have been
    /// collected, then truncates. An agent that does not declare
    /// `sessionCapabilities.list` yields an empty list **without a request**.
    ///
    /// # Errors
    ///
    /// Anything [`Self::start`] or the request raises.
    pub async fn list_sessions(
        &self,
        cwd: Option<std::path::PathBuf>,
        limit: usize,
        signal: Option<CancelSignal>,
    ) -> Result<Vec<Value>> {
        self.start().await?;
        if self
            .capabilities()
            .await
            .session_capabilities
            .list
            .is_none()
        {
            return Ok(Vec::new());
        }
        let mut sessions: Vec<Value> = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let page =
                limits::SESSION_LIST_PAGE_CAP.min(limit.saturating_sub(sessions.len()).max(1));
            let mut meta = Meta::new();
            meta.insert("limit".to_owned(), Value::from(page));
            let mut request = ListSessionsRequest::new().meta(meta);
            if let Some(cwd) = cwd.clone() {
                request = request.cwd(cwd);
            }
            if let Some(cursor) = cursor.clone() {
                request = request.cursor(cursor);
            }
            let response = self
                .request(
                    methods::SESSION_LIST,
                    request,
                    signal.clone(),
                    Some(limits::SESSION_LIST_TIMEOUT),
                )
                .await?;
            sessions.extend(
                response
                    .sessions
                    .iter()
                    .filter_map(|session| serde_json::to_value(session).ok()),
            );
            cursor = response.next_cursor.filter(|cursor| !cursor.is_empty());
            if cursor.is_none() || sessions.len() >= limit {
                break;
            }
        }
        sessions.truncate(limit);
        Ok(sessions)
    }

    /// `session/prompt` — the only method that carries user content.
    ///
    /// **Contract**, in the order the steps happen because the order is
    /// observable:
    ///
    /// 1. a second prompt for a session that already has one is
    ///    [`AcpError::SessionBusy`] (HTTP 409) **before** anything is started;
    /// 2. the prompt is normalized and checked against the agent's declared
    ///    prompt capabilities;
    /// 3. the turn is registered, so an inbound permission request can find it
    ///    and stop its clock;
    /// 4. an already-aborted signal still sends `session/cancel`;
    /// 5. the request goes out with **no transport deadline** — the pausable
    ///    timer owns it;
    /// 6. `stopReason: "cancelled"` is [`AcpError::Cancelled`], never `Ok`.
    ///
    /// # Errors
    ///
    /// [`AcpError::SessionBusy`], the capability refusals from
    /// [`assert_prompt_capabilities`], [`AcpError::Cancelled`], or anything the
    /// transport reports.
    pub async fn prompt(
        &self,
        session_id: &str,
        prompt: Prompt,
        options: PromptOptions,
    ) -> Result<PromptTurn> {
        let id = session_id.to_owned();
        if self.inner.lock_prompts().contains_key(&id) {
            return Err(AcpError::SessionBusy {
                label: self.inner.label.clone(),
                id,
            });
        }
        self.start().await?;
        let blocks = normalize_acp_prompt(&prompt)?;
        assert_prompt_capabilities(&blocks, &self.capabilities().await.prompt_capabilities)?;

        let timeout = options.timeout.unwrap_or(Some(self.inner.timeout));
        let combined = CancelSignal::new();
        if let Some(caller) = options.signal.clone() {
            if let Some(reason) = caller.reason() {
                combined.abort(reason);
            } else {
                let combined = combined.clone();
                tokio::spawn(async move {
                    caller.cancelled().await;
                    if let Some(reason) = caller.reason() {
                        combined.abort(reason);
                    }
                });
            }
        }
        let active = Arc::new(ActivePrompt {
            text: Mutex::new(Vec::new()),
            timer: PausableTimeout::arm(combined.clone(), timeout),
            on_update: options.on_update,
        });
        self.inner
            .lock_prompts()
            .insert(id.clone(), Arc::clone(&active));

        // The abort listener, and the already-aborted case. Both are
        // fire-and-forget: `docs/reference/contracts.json` — "cancellation must
        // never block".
        let cancel_task = {
            let client = self.clone();
            let combined = combined.clone();
            let id = id.clone();
            tokio::spawn(async move {
                combined.cancelled().await;
                let _ = client.cancel_session(&id).await;
            })
        };

        let outcome = self
            .send_prompt(&id, blocks, &combined, Arc::clone(&active))
            .await;

        cancel_task.abort();
        {
            let mut prompts = self.inner.lock_prompts();
            if prompts
                .get(&id)
                .is_some_and(|current| Arc::ptr_eq(current, &active))
            {
                prompts.remove(&id);
            }
        }
        outcome
    }

    async fn send_prompt(
        &self,
        id: &str,
        blocks: Vec<ContentBlock>,
        combined: &CancelSignal,
        active: Arc<ActivePrompt>,
    ) -> Result<PromptTurn> {
        let request = AcpPromptRequest::new(SessionId::new(id.to_owned()), blocks);
        // `timeoutMs: 0` on the transport: the pausable timer is the deadline.
        let response = self
            .request(
                methods::SESSION_PROMPT,
                request,
                Some(combined.clone()),
                None,
            )
            .await?;
        let content = {
            let text = active
                .text
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            text.concat().trim().to_owned()
        };
        if matches!(
            response.stop_reason,
            agent_client_protocol::schema::v1::StopReason::Cancelled
        ) {
            return Err(AcpError::Cancelled {
                reason: combined
                    .reason()
                    .and_then(|reason| reason.detail().map(str::to_owned)),
            });
        }
        Ok(PromptTurn { content, response })
    }

    /// `session/cancel` — a notification, fire and forget.
    ///
    /// **Contract** — no id, no response, errors swallowed. The `Result` here
    /// reports only that the process could not be started at all; a failure to
    /// deliver the notification is not surfaced, because a cancel that reports
    /// an error would give a caller something to wait on, and cancellation must
    /// never block.
    ///
    /// # Errors
    ///
    /// Anything [`Self::start`] raises.
    pub async fn cancel_session(&self, session_id: &str) -> Result<()> {
        let cx = self.connection().await?;
        let notification = CancelNotification::new(SessionId::new(session_id.to_owned()));
        let _ = cx.send_notification(notification);
        Ok(())
    }

    /// `session/set_config_option`.
    ///
    /// **Contract** — all three params are coerced to strings, and the
    /// deadline is 15 s rather than the client default.
    ///
    /// # Errors
    ///
    /// Anything [`Self::start`] or the request raises.
    pub async fn set_session_config_option(
        &self,
        session_id: &str,
        config_id: &str,
        value: &str,
    ) -> Result<Value> {
        let request = SetSessionConfigOptionRequest::new(
            SessionId::new(session_id.to_owned()),
            agent_client_protocol::schema::v1::SessionConfigId::new(config_id.to_owned()),
            // `value_id` serializes untagged as `{"value": "..."}` and the
            // request flattens it, which is upstream's `value: String(...)`.
            agent_client_protocol::schema::v1::SessionConfigOptionValue::value_id(value.to_owned()),
        );
        let response = self
            .request(
                methods::SESSION_SET_CONFIG_OPTION,
                request,
                None,
                Some(limits::SET_CONFIG_OPTION_TIMEOUT),
            )
            .await?;
        Ok(serde_json::to_value(response).unwrap_or(Value::Null))
    }

    /// `session/set_model` — the legacy path.
    ///
    /// # Errors
    ///
    /// Anything [`Self::start`] or the request raises.
    pub async fn set_legacy_session_model(
        &self,
        session_id: &str,
        model_id: &str,
    ) -> Result<Value> {
        self.request(
            methods::SESSION_SET_MODEL,
            SetModelRequest {
                session_id: session_id.to_owned(),
                model_id: model_id.to_owned(),
            },
            None,
            Some(limits::SET_MODEL_TIMEOUT),
        )
        .await
    }

    /// `session/close`, or a `session/cancel` notification when the agent does
    /// not declare it.
    ///
    /// **Contract** — the divergent bookkeeping is intentional and catalogued:
    /// on the close path the local record is dropped, on the cancel fallback it
    /// is **kept**, because the session still exists on the agent's side.
    ///
    /// # Errors
    ///
    /// Anything [`Self::start`] or the request raises.
    pub async fn close_session(&self, session_id: &str) -> Result<()> {
        self.start().await?;
        if self
            .capabilities()
            .await
            .session_capabilities
            .close
            .is_none()
        {
            self.cancel_session(session_id).await?;
            return Ok(());
        }
        self.request(
            methods::SESSION_CLOSE,
            CloseSessionRequest::new(SessionId::new(session_id.to_owned())),
            None,
            Some(limits::CLOSE_SESSION_TIMEOUT),
        )
        .await?;
        self.inner.lock_sessions().remove(session_id);
        Ok(())
    }

    /// Cancel every in-flight prompt, close the connection and tear the process
    /// tree down.
    ///
    /// **Contract** — `close()`, `acp-process-client.mjs:557-569`: every active
    /// prompt is cancelled first, fire and forget, and only then is the
    /// connection dropped.
    pub async fn close(&self) {
        let active: Vec<String> = self.inner.lock_prompts().keys().cloned().collect();
        for session_id in active {
            let _ = self.cancel_session(&session_id).await;
        }
        let mut guard = self.inner.connection.lock().await;
        if let Some(mut connection) = guard.take() {
            // Dropping the shutdown sender ends the connection closure, which
            // closes the transport and gives the child stdin EOF.
            connection.shutdown.take();
            let _ = tokio::time::timeout(limits::CLOSE_SESSION_TIMEOUT, &mut connection.task).await;
            connection.task.abort();
            connection.child.lock().await.stop().await;
        }
    }

    async fn connection(&self) -> Result<ConnectionTo<Agent>> {
        self.start().await?;
        let guard = self.inner.connection.lock().await;
        guard
            .as_ref()
            .map(|connection| connection.cx.clone())
            .ok_or_else(|| self.inner.process_gone())
    }

    /// Send one JSON-RPC request, wrapping every failure the way upstream does.
    ///
    /// **Contract** — *"ACP request failure wrapper"*. Three rules:
    ///
    /// * if the caller's own signal aborted, its reason is raised instead of a
    ///   transport error, so a cancelled turn does not report itself as a
    ///   backend failure;
    /// * stderr is attached for a transport failure and **omitted** for a
    ///   JSON-RPC error response, because a backend that answered with an error
    ///   has already said what it wants to say;
    /// * `body` is the stderr tail if there is one, else the error's own
    ///   `data.details`.
    async fn request<R>(
        &self,
        method: &'static str,
        request: R,
        signal: Option<CancelSignal>,
        timeout: Option<Duration>,
    ) -> Result<R::Response>
    where
        R: JsonRpcRequest,
        R::Response: Send,
    {
        let cx = self.connection().await?;
        let signal = signal.unwrap_or_default();
        let _timer = PausableTimeout::arm(signal.clone(), timeout);
        let sent: SentRequest<R::Response> = cx.send_request(request);

        let outcome = {
            let cancelled = signal.cancelled();
            futures::pin_mut!(cancelled);
            let response = sent.block_task();
            futures::pin_mut!(response);
            match futures::future::select(response, cancelled).await {
                futures::future::Either::Left((result, _)) => Some(result),
                // Dropping the request future auto-cancels it over JSON-RPC.
                futures::future::Either::Right(((), _)) => None,
            }
        };

        match outcome {
            Some(Ok(response)) => Ok(response),
            Some(Err(error)) => Err(self.inner.wrap_request_error(method, &error, &signal)),
            None => Err(self.inner.cancellation_error(&signal)),
        }
    }

    async fn start_process(&self) -> Result<Connection> {
        self.inner.reset_stderr();
        let child = crate::process::spawn(&self.inner.label, &self.inner.spec)?;
        let (stdin, stdout, stderr, handle) = child.into_parts();
        let handle = Arc::new(AsyncMutex::new(handle));

        // Bound the stderr tail. The SDK bounds its own copy too, but VIA's is
        // the one that reaches an error message, so it is kept here.
        {
            let inner = Arc::clone(&self.inner);
            tokio::spawn(async move {
                let mut stderr = stderr;
                let mut buffer = [0_u8; 8192];
                loop {
                    match stderr.read(&mut buffer).await {
                        Ok(0) | Err(_) => return,
                        Ok(read) => {
                            inner.append_stderr(&String::from_utf8_lossy(&buffer[..read]));
                        }
                    }
                }
            });
        }

        let incoming =
            futures::stream::unfold(tokio::io::BufReader::new(stdout), |mut reader| async move {
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) => None,
                    Ok(_) => {
                        while line.ends_with('\n') || line.ends_with('\r') {
                            line.pop();
                        }
                        Some((Ok(line), reader))
                    }
                    Err(error) => Some((Err(error), reader)),
                }
            });
        let outgoing = futures::sink::unfold(stdin, |mut stdin, line: String| async move {
            stdin.write_all(line.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await?;
            Ok::<_, std::io::Error>(stdin)
        });
        let transport = Lines::new(Box::pin(outgoing), Box::pin(incoming));

        let (ready_tx, ready_rx) = oneshot::channel::<ConnectionTo<Agent>>();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let task = {
            let inner = Arc::clone(&self.inner);
            let permission_inner = Arc::clone(&self.inner);
            tokio::spawn(async move {
                let result = Client
                    .builder()
                    .name(CLIENT_NAME)
                    .on_receive_notification(
                        async move |notification: RawSessionNotification, _cx| {
                            inner.handle_update(&notification);
                            Ok(())
                        },
                        on_receive_notification!(),
                    )
                    .on_receive_request(
                        async move |request: RawPermissionRequest, responder, _cx| {
                            let inner = Arc::clone(&permission_inner);
                            // Answering off the dispatch loop: a permission
                            // prompt can take minutes, and the loop is what
                            // carries the agent's progress updates.
                            tokio::spawn(async move {
                                let response = inner.handle_permission(request).await;
                                let _ = responder.respond(response);
                            });
                            Ok(agent_client_protocol::Handled::Yes)
                        },
                        on_receive_request!(),
                    )
                    .connect_with(transport, async move |cx: ConnectionTo<Agent>| {
                        if ready_tx.send(cx).is_err() {
                            return Ok(());
                        }
                        let _ = shutdown_rx.await;
                        Ok(())
                    })
                    .await;
                if let Err(error) = result {
                    tracing::debug!(event = "acp.connection_closed", error = %error);
                }
            })
        };

        let cx = match ready_rx.await {
            Ok(cx) => cx,
            Err(_) => {
                let mut child = handle.lock().await;
                return Err(self.inner.start_failure(&mut child, None).await);
            }
        };

        let initialize = InitializeRequest::new(PROTOCOL_VERSION)
            .client_capabilities(client_capabilities())
            .client_info(
                Implementation::new(CLIENT_NAME.to_owned(), CLIENT_VERSION.to_owned())
                    .title(CLIENT_TITLE.to_owned()),
            );
        let signal = CancelSignal::new();
        let _timer = PausableTimeout::arm(signal.clone(), Some(limits::INITIALIZE_TIMEOUT));
        let sent = cx.send_request(initialize);
        let response = {
            let cancelled = signal.cancelled();
            futures::pin_mut!(cancelled);
            let response = sent.block_task();
            futures::pin_mut!(response);
            match futures::future::select(response, cancelled).await {
                futures::future::Either::Left((result, _)) => result,
                futures::future::Either::Right(((), _)) => {
                    Err(agent_client_protocol::Error::request_cancelled())
                }
            }
        };

        let initialize = match response {
            Ok(initialize) => initialize,
            Err(error) => {
                task.abort();
                let mut child = handle.lock().await;
                return Err(self.inner.start_failure(&mut child, Some(&error)).await);
            }
        };

        tracing::info!(
            event = "acp.initialized",
            backend = self.inner.label,
            protocol_version = initialize.protocol_version.as_u16(),
        );

        if initialize.protocol_version != PROTOCOL_VERSION {
            task.abort();
            handle.lock().await.stop().await;
            return Err(AcpError::ProtocolVersionMismatch {
                label: self.inner.label.clone(),
                agent: initialize.protocol_version.as_u16().to_string(),
                client: PROTOCOL_VERSION.as_u16().to_string(),
            });
        }

        Ok(Connection {
            cx,
            initialize,
            child: handle,
            shutdown: Some(shutdown_tx),
            task,
        })
    }
}

impl Inner {
    fn lock_sessions(&self) -> std::sync::MutexGuard<'_, HashMap<String, SharedSession>> {
        self.sessions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn lock_prompts(&self) -> std::sync::MutexGuard<'_, HashMap<String, Arc<ActivePrompt>>> {
        self.active_prompts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn process_gone(&self) -> AcpError {
        AcpError::ProcessGone {
            label: self.label.clone(),
        }
    }

    fn reset_stderr(&self) {
        self.stderr
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    fn stderr(&self) -> String {
        self.stderr
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// `appendStderr` — `acp-process-client.mjs:124-129`.
    ///
    /// Strip terminal control sequences, run the profile's sanitizer, trim,
    /// then keep the **last** [`limits::MAX_STDERR_CHARS`] characters. Keeping
    /// the tail rather than the head is the point: the last thing a failing
    /// backend printed is the actionable part.
    fn append_stderr(&self, chunk: &str) {
        let mut stderr = self
            .stderr
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let combined = std::format!("{stderr}{chunk}");
        let cleaned = self.clean_process_output(&combined);
        let kept: String = if cleaned.chars().count() > limits::MAX_STDERR_CHARS {
            cleaned
                .chars()
                .skip(cleaned.chars().count() - limits::MAX_STDERR_CHARS)
                .collect()
        } else {
            cleaned
        };
        *stderr = kept;
    }

    fn clean_process_output(&self, value: &str) -> String {
        let stripped = strip_vt_control_characters(value).trim().to_owned();
        let sanitized = self
            .sanitize_process_output
            .as_ref()
            .map_or(stripped.clone(), |sanitize| sanitize(&stripped));
        sanitized.trim().to_owned()
    }

    fn handle_update(&self, notification: &RawSessionNotification) {
        // "Notifications with a missing sessionId or missing update are
        // silently dropped."
        let (Some(session_id), Some(update)) = (
            notification
                .session_id
                .as_deref()
                .map(str::trim)
                .filter(|id| !id.is_empty()),
            notification.update.as_ref(),
        ) else {
            return;
        };
        let active = self.lock_prompts().get(session_id).map(Arc::clone);
        if let Some(active) = active.as_ref() {
            let text = text_from_update(update);
            if !text.is_empty() {
                active
                    .text
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(text.to_owned());
            }
        }
        // "Every exception thrown by downstream projection is swallowed" — UI
        // projection must never interrupt the ACP connection. A panicking
        // observer is caught here for the same reason.
        if let Some(on_update) = active.as_ref().and_then(|active| active.on_update.as_ref()) {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                on_update(session_id, update);
            }));
        }
        if let Some(on_update) = self.on_update.as_ref() {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                on_update(session_id, update);
            }));
        }
    }

    async fn handle_permission(&self, request: RawPermissionRequest) -> Value {
        let session_id = request.session_id.unwrap_or_default();
        let Some(handler) = self.on_permission.as_ref() else {
            // No handler: `cancelled`, never a rejection.
            return serde_json::to_value(permission_reply(
                &request.options,
                PermissionDecision::Cancel,
            ))
            .unwrap_or(Value::Null);
        };
        let active = self.lock_prompts().get(&session_id).map(Arc::clone);
        if let Some(active) = active.as_ref() {
            active.timer.pause();
        }
        let decision = handler(PermissionRequest {
            session_id,
            tool_call: request.tool_call.unwrap_or(Value::Null),
            options: request.options.clone(),
        })
        .await;
        if let Some(active) = active.as_ref() {
            active.timer.resume();
        }
        serde_json::to_value(permission_reply(&request.options, decision)).unwrap_or(Value::Null)
    }

    fn cancellation_error(&self, signal: &CancelSignal) -> AcpError {
        AcpError::Cancelled {
            reason: signal
                .reason()
                .and_then(|reason| reason.detail().map(str::to_owned)),
        }
    }

    fn wrap_request_error(
        &self,
        method: &str,
        error: &agent_client_protocol::Error,
        signal: &CancelSignal,
    ) -> AcpError {
        if signal.is_aborted() {
            return self.cancellation_error(signal);
        }
        let transport_failure = is_incoming_transport_closed(error);
        let stderr = if transport_failure {
            self.clean_process_output(&self.stderr())
        } else {
            String::new()
        };
        let details = request_error_details(error);
        AcpError::RequestFailed {
            label: self.label.clone(),
            method: method.to_owned(),
            detail: request_error_message(error, self.format_request_error.as_ref()),
            stderr: stderr.clone(),
            body: if stderr.is_empty() { details } else { stderr },
        }
    }

    /// Classify a start-up failure.
    ///
    /// Upstream races the `initialize` request against the child's `exit`
    /// event, so which of `进程意外退出` and `初始化失败` is reported depends on
    /// which fired first. Here the question is asked rather than raced: if the
    /// child is already gone, report the exit; otherwise, if it wrote anything
    /// to stderr, report that; otherwise report the transport failure. The set
    /// of messages is upstream's and the choice is deterministic.
    async fn start_failure(
        &self,
        child: &mut AcpChildHandle,
        error: Option<&agent_client_protocol::Error>,
    ) -> AcpError {
        let exited = child.has_exited();
        let code = if exited {
            child.wait_for_exit().await
        } else {
            String::new()
        };
        child.stop().await;
        let stderr = self.clean_process_output(&self.stderr());
        tracing::error!(
            event = "acp.initialization_failed",
            backend = self.label,
            exited,
            stderr = stderr,
        );
        if exited {
            return AcpError::ProcessExited {
                label: self.label.clone(),
                code,
                stderr,
            };
        }
        if !stderr.is_empty() {
            return AcpError::InitializeFailed {
                label: self.label.clone(),
                stderr,
            };
        }
        match error {
            Some(error) => AcpError::RequestFailed {
                label: self.label.clone(),
                method: methods::INITIALIZE.to_owned(),
                detail: request_error_message(error, self.format_request_error.as_ref()),
                stderr: String::new(),
                body: request_error_details(error),
            },
            None => self.process_gone(),
        }
    }
}

/// The locale this client renders its messages in.
impl AcpProcessClient {
    /// The locale error sentences are rendered in.
    #[must_use]
    pub fn locale(&self) -> Locale {
        self.inner.locale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_client_identity_is_vias_own() {
        assert_eq!(CLIENT_NAME, "via");
        assert_eq!(CLIENT_TITLE, "VIA Gateway");
        assert!(!CLIENT_VERSION.is_empty());
    }

    #[test]
    fn the_client_declares_no_capability_at_all() {
        // The contract is "declares nothing", which is what an agent tests
        // before attempting `fs/*` or `terminal/*`.
        assert!(declares_no_capability(&client_capabilities()));
        let declared = serde_json::to_value(client_capabilities()).expect("serializes");
        assert_eq!(declared["terminal"], serde_json::json!(false));
        assert_eq!(declared["fs"]["readTextFile"], serde_json::json!(false));
        assert_eq!(declared["fs"]["writeTextFile"], serde_json::json!(false));
        assert!(
            declared.get("session").is_none(),
            "no client session extension is advertised either"
        );

        // And the predicate is not vacuous.
        let mut promising = ClientCapabilities::default();
        promising.terminal = true;
        assert!(!declares_no_capability(&promising));
        let mut reading = ClientCapabilities::default();
        reading.fs.read_text_file = true;
        assert!(!declares_no_capability(&reading));
    }

    #[test]
    fn vt_control_sequences_never_reach_a_message() {
        assert_eq!(
            strip_vt_control_characters("\u{1b}[1;36m banner \u{1b}[0m tail"),
            " banner  tail"
        );
        assert_eq!(
            strip_vt_control_characters("\u{1b}]0;window title\u{7}kept"),
            "kept"
        );
        assert_eq!(
            strip_vt_control_characters("\u{1b}]8;;https://example.test\u{1b}\\link"),
            "link"
        );
        assert_eq!(strip_vt_control_characters("plain"), "plain");
        assert_eq!(strip_vt_control_characters("\u{1b}"), "");
    }

    #[test]
    fn request_details_prefer_the_structured_field() {
        let structured = agent_client_protocol::Error::new(-32603, "Internal error")
            .data(serde_json::json!({ "details": "  missing scope  " }));
        assert_eq!(request_error_details(&structured), "missing scope");

        let string_data = agent_client_protocol::Error::new(-32603, "Internal error")
            .data(serde_json::json!(" raw "));
        assert_eq!(request_error_details(&string_data), "raw");

        let none = agent_client_protocol::Error::new(-32603, "Internal error");
        assert_eq!(request_error_details(&none), "");
    }

    #[test]
    fn the_default_composition_appends_details_in_fullwidth_parentheses() {
        let error = agent_client_protocol::Error::new(-32603, "Internal error")
            .data(serde_json::json!({ "details": "missing scope" }));
        assert_eq!(
            request_error_message(&error, None),
            "Internal error（missing scope）"
        );

        let already_said =
            agent_client_protocol::Error::new(-32603, "Internal error: missing scope")
                .data(serde_json::json!({ "details": "missing scope" }));
        assert_eq!(
            request_error_message(&already_said, None),
            "Internal error: missing scope",
            "details already inside the message are not repeated"
        );
    }

    #[test]
    fn a_profile_hook_replaces_the_composed_message() {
        // Backend-specific behaviour arrives as data on a profile: this crate
        // names no backend, so the hook is a function value.
        let hook: FormatRequestError = Arc::new(|context: &RequestErrorContext| {
            (!context.details.is_empty()).then(|| std::format!("approve {}", context.details))
        });
        let error = agent_client_protocol::Error::new(-32603, "Internal error")
            .data(serde_json::json!({ "details": "operator.write" }));
        assert_eq!(
            request_error_message(&error, Some(&hook)),
            "approve operator.write"
        );

        let blank: FormatRequestError = Arc::new(|_| Some("   ".to_owned()));
        assert_eq!(
            request_error_message(&error, Some(&blank)),
            "Internal error（operator.write）",
            "a hook that answers with whitespace falls through to the default"
        );
    }

    #[test]
    fn a_session_record_keeps_its_identity_across_re_registration() {
        // Upstream: 'keeps session object identity stable across
        // re-registration' (acp-process-client.test.mjs:8-28).
        let client = AcpProcessClient::builder().label("Test Agent").build();
        let first = client
            .remember_session(
                "sess-1",
                SessionDetails {
                    role: Some(SessionRole::Coordinator),
                    ..SessionDetails::default()
                },
            )
            .expect("a non-blank id is remembered");
        first
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .routing
            .insert("coordinationRunId".into(), Value::from("work_current"));

        let second = client
            .remember_session(
                "sess-1",
                SessionDetails {
                    cwd: Some("/next".into()),
                    ..SessionDetails::default()
                },
            )
            .expect("remembered again");

        assert!(
            Arc::ptr_eq(&first, &second),
            "a resume must not replace the record, or a permission request \
             answers into a finished task"
        );
        let record = second
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert_eq!(record.session_id, "sess-1");
        assert_eq!(record.cwd.as_deref(), Some(std::path::Path::new("/next")));
        assert_eq!(
            record.role,
            SessionRole::Coordinator,
            "a field the second call did not mention is not blanked"
        );
        assert_eq!(
            record.routing.get("coordinationRunId"),
            Some(&Value::from("work_current")),
            "runtime routing the layer above keeps on the record survives"
        );
    }

    #[test]
    fn a_blank_session_id_is_remembered_as_nothing() {
        let client = AcpProcessClient::builder().label("Test Agent").build();
        assert!(
            client
                .remember_session("", SessionDetails::default())
                .is_none()
        );
        assert!(
            client
                .remember_session("   ", SessionDetails::default())
                .is_none()
        );
        assert_eq!(client.session_count(), 0);
    }

    #[test]
    fn the_default_deadline_is_the_catalogued_one() {
        let client = AcpProcessClient::builder().label("x").build();
        assert_eq!(client.timeout(), limits::DEFAULT_TIMEOUT);
        let overridden = AcpProcessClient::builder()
            .label("x")
            .timeout(Duration::from_secs(9))
            .build();
        assert_eq!(overridden.timeout(), Duration::from_secs(9));
    }

    #[tokio::test]
    async fn stderr_keeps_the_tail_and_drops_the_paint() {
        let client = AcpProcessClient::builder().label("x").build();
        client
            .inner
            .append_stderr("\u{1b}[1;36m starting \u{1b}[0m\n");
        assert_eq!(client.stderr(), "starting");

        client
            .inner
            .append_stderr(&"x".repeat(limits::MAX_STDERR_CHARS + 500));
        let stderr = client.stderr();
        assert_eq!(stderr.chars().count(), limits::MAX_STDERR_CHARS);
        assert!(
            stderr.ends_with('x'),
            "the tail is kept: the last thing a failing backend printed"
        );
        assert!(
            !stderr.starts_with("starting"),
            "and the head is what gets dropped"
        );
    }

    #[tokio::test]
    async fn a_client_that_never_started_declares_nothing_and_is_not_ready() {
        let client = AcpProcessClient::builder().label("x").build();
        assert!(!client.is_ready().await);
        let capabilities = client.capabilities().await;
        assert!(!capabilities.prompt_capabilities.image);
        assert!(!capabilities.load_session);
        assert!(capabilities.session_capabilities.resume.is_none());
        assert!(client.agent_info().await.is_none());
    }
}
