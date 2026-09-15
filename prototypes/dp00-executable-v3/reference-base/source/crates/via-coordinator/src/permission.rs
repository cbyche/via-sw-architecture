//! The permission broker — the thing standing between a backend's
//! `session/request_permission` and a user who is not at a keyboard.
//!
//! `server/src/agent/permission-broker.mjs` in full. Four responsibilities:
//!
//! 1. **the auto-approve carve-out** for VIA's own five session tools;
//! 2. **register, then emit** — the pending record exists before anybody is
//!    told about it;
//! 3. **owner-scoped resolution**, with a bounded ledger so a late decision is
//!    idempotent rather than an error;
//! 4. **scope cancellation**, so a permission outlives its prompt by nothing.
//!
//! # The allow-list is imported, never restated
//!
//! `docs/architecture.md` §13 names this the highest-risk rename in the port:
//! the broker matches VIA's tool names *three* ways — exact,
//! `endsWith("__<name>")`, `startsWith("<name> (")` — and a rename applied to
//! the registration but not the matcher turns every internal coordination call
//! into a permission prompt with nobody there to answer it. So the carve-out is
//! [`via_mcp_tools::is_session_tool`] and there is no second list here.
//!
//! # Register before you emit
//!
//! `docs/architecture.md` §6 states the ordering and why: *"`register(correlation_id)`
//! **before** emitting, then `wait(…)`, because that ordering is what closes the
//! race."* Here the whole of registration, emission and the resolver's
//! installation happens inside the owning task's handling of one command, so
//! there is no window at all: a decision that arrives before
//! [`PermissionBroker::request`]'s caller has even been scheduled still finds
//! the record.
//!
//! # `always` uses the backend's own session-scoped option when it has one
//!
//! The kind-preference order is
//! [`via_acp::permission::APPROVE_KIND_ORDER`] /
//! [`via_acp::permission::REJECT_KIND_ORDER`], which `via-acp` already ships
//! and which the catalogue calls a *semantic* contract: an approval prefers
//! `allow_once` and falls back to `allow_always`, a rejection prefers
//! `reject_always` and falls back to `reject_once`. The broker therefore
//! answers in [`PermissionDecision`] and lets `via-acp` pick the option — one
//! answer to *"which option expresses this decision"*, in the crate that owns
//! the wire.

use std::fmt;
use std::sync::Arc;

use indexmap::IndexMap;
use serde_json::{Map, Value};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use uuid::Uuid;
use via_acp::PermissionDecision;
use via_acp::session::clean_value;
use via_downstream::text::{DEFAULT_BOUND, bounded, clean};
use via_work::{PendingPermission, PermissionStatus};

use crate::error::CoordinatorError;
use crate::executor::{COMMAND_QUEUE_DEPTH, ExecutorStopped};

/// The prefix on every permission id.
///
/// **External contract** — `docs/reference/contracts.json` (`ws-event` /
/// *backend.permission.requested*): `auth_<32 hex, uuid with dashes removed>`.
/// The prefix reaches the user: the voice model quotes the id back in
/// `respond_agent_permission`.
pub const PERMISSION_ID_PREFIX: &str = "auth_";

/// How many resolved permissions one broker remembers.
///
/// **External contract** — `permission-broker.mjs:30` (`resolvedLimit = 200`),
/// catalogued under *timeouts and limits*. Eviction is FIFO, oldest first, so a
/// decision that arrives twice is idempotent for as long as it plausibly could
/// have been in flight.
pub const RESOLVED_LIMIT: usize = 200;

/// The bound on the tool name inside `category` and `summary`.
///
/// **External contract** — `permission-broker.mjs:57,60` (`bounded(name, 80)`).
pub const CATEGORY_BOUND: usize = 80;

/// The bound on the detail half of `summary`.
///
/// **External contract** — `permission-broker.mjs:9,61`
/// (`bounded(value, max = 300)`), which is
/// [`via_downstream::text::DEFAULT_BOUND`].
pub const SUMMARY_DETAIL_BOUND: usize = DEFAULT_BOUND;

/// The `category` a tool call with no name at all is given.
///
/// **External contract** — `permission-broker.mjs:57`
/// (`bounded(name, 80) || 'unknown'`). It is a wire-shaped enum value rather
/// than prose — the same class as `not_found` — so it is a `const` rather than
/// a `via-i18n` key.
pub const UNKNOWN_CATEGORY: &str = "unknown";

/// The separator between the tool name and its detail in `summary`.
///
/// **External contract** — `permission-broker.mjs:66` (`join('：')`), a
/// full-width colon. The catalogue names it: *"the 'auth_' id prefix and the
/// full-width colon '：' joining name and detail are literal."*
pub const SUMMARY_SEPARATOR: char = '：';

/// The `rawInput` keys the summary's detail is read from, in order.
///
/// **External contract** — `permission-broker.mjs:61-64`. It is a JavaScript
/// `||` chain, so the first **truthy** value wins.
pub const DETAIL_KEYS: [&str; 3] = ["description", "command", "path"];

/// The wire value that approves a permission.
///
/// **External contract** — `docs/reference/contracts.json` (`http-route` /
/// *POST /api/permissions/:id*): the body is
/// `{decision: 'always'|'reject'}` and any other value is a 400. The same two
/// literals are the enum in the voice frontend's permission tool schema.
pub const DECISION_ALWAYS: &str = "always";

/// The wire value that refuses a permission.
pub const DECISION_REJECT: &str = "reject";

/// Whether the Gateway asks, or approves everything.
///
/// **External contract** — `permission-broker.mjs:45`
/// (`this.permissionMode === 'full'`) and
/// `acp-backend-adapter.mjs:163` (`permissionMode === 'full' ? 'full' : 'native'`):
/// anything that is not the literal `full` is `native`, so an unrecognised
/// value fails **closed**, into asking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PermissionMode {
    /// Every permission is put to the user. The default, and the safe side of
    /// an unrecognised value.
    #[default]
    Native,
    /// The backend owns permissions; VIA approves without asking.
    Full,
}

impl PermissionMode {
    /// The wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Full => "full",
        }
    }

    /// Parse a configured value. Total, and fails closed.
    #[must_use]
    pub fn from_wire(value: &str) -> Self {
        if clean(value) == Self::Full.as_str() {
            Self::Full
        } else {
            Self::Native
        }
    }
}

/// A decision a person made.
///
/// The two literals `POST /api/permissions/:id` accepts. Distinct from
/// [`PermissionDecision`], which also has a `Cancel` arm — nobody *decides* to
/// cancel; a cancellation is what happens when the question goes away.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionResponse {
    /// Allow it.
    Always,
    /// Refuse it.
    Reject,
}

impl PermissionResponse {
    /// The wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Always => DECISION_ALWAYS,
            Self::Reject => DECISION_REJECT,
        }
    }

    /// Parse the HTTP body's `decision`, or `None` for the 400.
    ///
    /// Upstream's broker itself is looser — `const approved = decision === 'always'`
    /// treats every non-`always` value as a rejection — and the validation
    /// lives in the route (`gateway-application.mjs:366-400`). Making the
    /// broker take a *typed* decision moves the refusal to the boundary where
    /// the catalogued 400 already is, and leaves no path by which a typo
    /// silently denies.
    #[must_use]
    pub fn from_wire(value: &str) -> Option<Self> {
        match clean(value) {
            DECISION_ALWAYS => Some(Self::Always),
            DECISION_REJECT => Some(Self::Reject),
            _ => None,
        }
    }

    /// The ACP decision this expresses.
    #[must_use]
    pub const fn decision(self) -> PermissionDecision {
        match self {
            Self::Always => PermissionDecision::Approve,
            Self::Reject => PermissionDecision::Reject,
        }
    }

    /// The status a permission resolved this way reports.
    #[must_use]
    pub const fn status(self) -> PermissionStatus {
        match self {
            Self::Always => PermissionStatus::Approved,
            Self::Reject => PermissionStatus::Denied,
        }
    }
}

/// Where a permission request came from.
///
/// Upstream reads all four off the mutable ACP session object
/// (`acp-backend-adapter.mjs:1028-1033`), which is why upstream has to clear
/// `session.permissionScopeId` in a `finally` and check it is still the current
/// value first. Passing it per request removes that whole class of problem: a
/// request carries the scope it was made in and cannot be re-scoped later.
#[derive(Clone, Default)]
pub struct PermissionContext {
    /// Whose backend session asked.
    pub owner_id: String,
    /// The Work the asking turn belongs to, when there is one.
    pub work_id: Option<String>,
    /// The backend session id, for scope diagnostics.
    pub session_id: String,
    /// The prompt this permission belongs to — `prompt_<uuid>`. Cancelling the
    /// scope cancels every permission raised under it and nothing else.
    pub scope_id: String,
    /// Where `backend.permission.requested` / `.resolved` go.
    pub observer: Option<Arc<dyn PermissionObserver>>,
    /// The asking turn's cancellation scope. When it fires, the question goes
    /// away and the backend is answered `cancelled`.
    pub signal: Option<CancellationToken>,
}

impl fmt::Debug for PermissionContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PermissionContext")
            .field("owner_id", &self.owner_id)
            .field("work_id", &self.work_id)
            .field("session_id", &self.session_id)
            .field("scope_id", &self.scope_id)
            .field("observer", &self.observer.is_some())
            .finish()
    }
}

/// Where the two permission events go.
///
/// Upstream's `session.onEvent`. The payload is
/// [`via_work::PendingPermission`], which already serializes in the catalogued
/// field order and already omits `patterns` from a resolution — so a Work
/// runner can forward one straight into
/// [`via_work::RunnerEvent::PermissionRequested`].
pub trait PermissionObserver: Send + Sync + fmt::Debug {
    /// `backend.permission.requested`.
    fn requested(&self, permission: &PendingPermission);

    /// `backend.permission.resolved`.
    fn resolved(&self, permission: &PendingPermission);
}

/// The tool name a permission request is about.
///
/// **External contract** — `permission-broker.mjs:39`
/// (`clean(params?.toolCall?.name || params?.toolCall?.title)`). A JavaScript
/// `||`, so a `name` of `"   "` is truthy and wins over a real `title`,
/// yielding the empty string and therefore [`UNKNOWN_CATEGORY`].
///
/// This deliberately differs from
/// [`via_acp::PermissionRequest::tool_name`], which filters a blank `name` out
/// and falls through to the title. That one answers *"what should the UI call
/// this tool"*; this one answers *"what did the backend name it"*, and the
/// allow-list match depends on the second.
#[must_use]
pub fn tool_name(tool_call: &Value) -> String {
    let Some(object) = tool_call.as_object() else {
        return String::new();
    };
    clean_value(first_truthy(object, &["name", "title"]))
}

/// The detail half of the summary.
///
/// **External contract** — `permission-broker.mjs:61-64`.
#[must_use]
pub fn tool_detail(tool_call: &Value) -> String {
    let Some(raw_input) = tool_call.get("rawInput").and_then(Value::as_object) else {
        return String::new();
    };
    clean_value(first_truthy(raw_input, &DETAIL_KEYS))
}

/// `a || b || c` over a JSON object — the first **truthy** member.
fn first_truthy<'a>(object: &'a Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    keys.iter()
        .find_map(|key| object.get(*key).filter(|value| is_truthy(value)))
}

/// JavaScript truthiness.
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64() != Some(0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// The `category` a permission reports.
///
/// **External contract** — `permission-broker.mjs:57`.
#[must_use]
pub fn category_of(tool_call: &Value) -> String {
    let bounded_name = bounded(&tool_name(tool_call), CATEGORY_BOUND);
    if bounded_name.is_empty() {
        UNKNOWN_CATEGORY.to_owned()
    } else {
        bounded_name
    }
}

/// The `summary` a permission reports.
///
/// **External contract** — `permission-broker.mjs:58-67`: the bounded name and
/// the bounded detail, **empty parts dropped**, joined by
/// [`SUMMARY_SEPARATOR`]. A tool call with neither yields the empty string,
/// which is why the `category` has its own `unknown` fallback and the summary
/// does not.
#[must_use]
pub fn summary_of(tool_call: &Value) -> String {
    [
        bounded(&tool_name(tool_call), CATEGORY_BOUND),
        bounded(&tool_detail(tool_call), SUMMARY_DETAIL_BOUND),
    ]
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(&SUMMARY_SEPARATOR.to_string())
}

/// A fresh permission id.
///
/// **External contract** — `permission-broker.mjs:51`:
/// `` `auth_${randomUUID().replaceAll('-', '')}` ``, 32 lower-case hex digits.
#[must_use]
pub fn new_permission_id() -> String {
    format!("{PERMISSION_ID_PREFIX}{}", Uuid::new_v4().simple())
}

/// One pending record, as the owning task holds it.
struct Record {
    permission: PendingPermission,
    owner_id: String,
    scope_id: String,
    observer: Option<Arc<dyn PermissionObserver>>,
    settle: oneshot::Sender<PermissionDecision>,
}

/// What the owning task hands back for a request it did not answer at once.
enum Ticket {
    /// Answered without asking anybody: full-permission mode, or one of VIA's
    /// own session tools.
    Immediate(PermissionDecision),
    /// Registered and announced; the answer arrives on `wait`.
    Pending {
        id: String,
        wait: oneshot::Receiver<PermissionDecision>,
    },
}

enum Command {
    Request {
        tool_call: Value,
        context: PermissionContext,
        reply: oneshot::Sender<Ticket>,
    },
    Respond {
        id: String,
        response: PermissionResponse,
        owner_id: String,
        reply: oneshot::Sender<Result<PendingPermission, CoordinatorError>>,
    },
    Cancel {
        id: String,
        reply: oneshot::Sender<bool>,
    },
    CancelScope {
        scope_id: String,
        reply: oneshot::Sender<usize>,
    },
    CancelAll {
        reply: oneshot::Sender<usize>,
    },
    Pending {
        reply: oneshot::Sender<Vec<PendingPermission>>,
    },
    Resolved {
        reply: oneshot::Sender<Vec<PendingPermission>>,
    },
}

/// The broker.
///
/// An owning task, per `docs/architecture.md` §11: the pending map, the
/// resolved ledger and its FIFO eviction are all held by value in one place, so
/// "register before emit" and "the oldest of two hundred is the one evicted"
/// are properties of a single sequential handler rather than of a lock
/// discipline.
#[derive(Debug, Clone)]
pub struct PermissionBroker {
    commands: mpsc::Sender<Command>,
}

impl PermissionBroker {
    /// Start a broker on the ambient tokio runtime.
    #[must_use]
    pub fn new(mode: PermissionMode) -> Self {
        Self::spawn(mode, RESOLVED_LIMIT, None)
    }

    /// Start a broker whose task is registered on `tracker`.
    #[must_use]
    pub fn with_tracker(mode: PermissionMode, tracker: &TaskTracker) -> Self {
        Self::spawn(mode, RESOLVED_LIMIT, Some(tracker))
    }

    /// Start a broker with a non-default ledger size.
    ///
    /// Upstream's constructor takes `resolvedLimit` too
    /// (`permission-broker.mjs:30`); it exists so a test can reach the eviction
    /// boundary without building two hundred permissions.
    #[must_use]
    pub fn with_resolved_limit(mode: PermissionMode, resolved_limit: usize) -> Self {
        Self::spawn(mode, resolved_limit, None)
    }

    fn spawn(mode: PermissionMode, resolved_limit: usize, tracker: Option<&TaskTracker>) -> Self {
        let (commands, receiver) = mpsc::channel(COMMAND_QUEUE_DEPTH);
        let future = run_broker(mode, resolved_limit, receiver);
        match tracker {
            Some(tracker) => {
                tracker.spawn(future);
            }
            None => {
                tokio::spawn(future);
            }
        }
        Self { commands }
    }

    /// Answer one `session/request_permission`.
    ///
    /// Returns when a decision exists: at once for a full-permission backend or
    /// one of VIA's own session tools, and otherwise when somebody responds,
    /// the scope is cancelled, or the asking turn's signal fires.
    ///
    /// The caller turns the answer into the wire reply with
    /// [`via_acp::permission::reply`], which is where the option-kind
    /// preference order lives.
    ///
    /// # Errors
    ///
    /// [`ExecutorStopped`] when the broker's task is gone. It is deliberately
    /// *not* answered as `cancelled`: a stopped broker means the Gateway is
    /// shutting down, and a caller that wants to answer the backend anyway can
    /// map it.
    pub async fn request(
        &self,
        tool_call: &Value,
        context: PermissionContext,
    ) -> Result<PermissionDecision, ExecutorStopped> {
        let signal = context.signal.clone();
        let (reply, ticket) = oneshot::channel();
        self.commands
            .send(Command::Request {
                tool_call: tool_call.clone(),
                context,
                reply,
            })
            .await
            .map_err(|_| ExecutorStopped)?;
        match ticket.await.map_err(|_| ExecutorStopped)? {
            Ticket::Immediate(decision) => Ok(decision),
            Ticket::Pending { id, mut wait } => match signal {
                None => Ok(wait.await.unwrap_or(PermissionDecision::Cancel)),
                Some(signal) => {
                    tokio::select! {
                        settled = &mut wait => Ok(settled.unwrap_or(PermissionDecision::Cancel)),
                        () = signal.cancelled() => {
                            // Upstream's `signal.addEventListener('abort', …)`:
                            // the record is cancelled, which resolves `wait`.
                            self.cancel(&id).await;
                            Ok(wait.await.unwrap_or(PermissionDecision::Cancel))
                        }
                    }
                }
            },
        }
    }

    /// Record a person's decision.
    ///
    /// **External contract** — `permission-broker.mjs:100-134`. Idempotent for
    /// as long as the resolved ledger remembers: a second response with the
    /// same id from the same owner returns the same permission rather than
    /// failing.
    ///
    /// # Errors
    ///
    /// [`CoordinatorError::PermissionRequestUnknown`] when no pending record
    /// answers to `id`, when the ledger has forgotten it, or when it belongs to
    /// a different owner. All three share one sentence on purpose.
    pub async fn respond(
        &self,
        id: &str,
        response: PermissionResponse,
        owner_id: &str,
    ) -> Result<PendingPermission, CoordinatorError> {
        let (reply, answer) = oneshot::channel();
        self.commands
            .send(Command::Respond {
                id: id.to_owned(),
                response,
                owner_id: owner_id.to_owned(),
                reply,
            })
            .await
            .map_err(|_| ExecutorStopped)?;
        answer.await.map_err(|_| ExecutorStopped)?
    }

    /// Cancel one pending permission by id. Returns whether there was one.
    pub async fn cancel(&self, id: &str) -> bool {
        let (reply, answer) = oneshot::channel();
        if self
            .commands
            .send(Command::Cancel {
                id: id.to_owned(),
                reply,
            })
            .await
            .is_err()
        {
            return false;
        }
        answer.await.unwrap_or(false)
    }

    /// Cancel every pending permission raised under `scope_id`.
    ///
    /// **External contract** — `cancelScope`, `permission-broker.mjs:136-142`,
    /// and the test upstream names *"permission cleanup is isolated to the ACP
    /// prompt that requested it"*. A blank scope cancels **nothing**: upstream
    /// returns early on `if (!scope)`, and without that guard every permission
    /// raised outside a prompt would be cancelled by any scope teardown.
    ///
    /// Returns how many were cancelled.
    pub async fn cancel_scope(&self, scope_id: &str) -> usize {
        let (reply, answer) = oneshot::channel();
        if self
            .commands
            .send(Command::CancelScope {
                scope_id: scope_id.to_owned(),
                reply,
            })
            .await
            .is_err()
        {
            return 0;
        }
        answer.await.unwrap_or(0)
    }

    /// Cancel everything pending. Upstream's `cancelAll`, called at teardown.
    pub async fn cancel_all(&self) -> usize {
        let (reply, answer) = oneshot::channel();
        if self
            .commands
            .send(Command::CancelAll { reply })
            .await
            .is_err()
        {
            return 0;
        }
        answer.await.unwrap_or(0)
    }

    /// Every pending permission, oldest first.
    pub async fn pending(&self) -> Vec<PendingPermission> {
        let (reply, answer) = oneshot::channel();
        if self
            .commands
            .send(Command::Pending { reply })
            .await
            .is_err()
        {
            return Vec::new();
        }
        answer.await.unwrap_or_default()
    }

    /// Every remembered resolution, oldest first.
    pub async fn resolved(&self) -> Vec<PendingPermission> {
        let (reply, answer) = oneshot::channel();
        if self
            .commands
            .send(Command::Resolved { reply })
            .await
            .is_err()
        {
            return Vec::new();
        }
        answer.await.unwrap_or_default()
    }

    /// Whether the owning task is still running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        !self.commands.is_closed()
    }
}

/// The owning task.
async fn run_broker(
    mode: PermissionMode,
    resolved_limit: usize,
    mut commands: mpsc::Receiver<Command>,
) {
    let mut pending: IndexMap<String, Record> = IndexMap::new();
    let mut resolved: IndexMap<String, (String, PendingPermission)> = IndexMap::new();

    while let Some(command) = commands.recv().await {
        match command {
            Command::Request {
                tool_call,
                context,
                reply,
            } => {
                let name = tool_name(&tool_call);
                // The carve-out, and the *only* place it is applied.
                if mode == PermissionMode::Full || via_mcp_tools::is_session_tool(&name) {
                    let _ = reply.send(Ticket::Immediate(PermissionDecision::Approve));
                    continue;
                }
                let id = new_permission_id();
                let permission = PendingPermission {
                    id: id.clone(),
                    work_id: context.work_id.clone(),
                    status: PermissionStatus::Pending,
                    category: category_of(&tool_call),
                    summary: summary_of(&tool_call),
                    // `patterns: []` is present on the request and absent from
                    // the resolution; that asymmetry is the catalogued shape.
                    patterns: Some(Vec::new()),
                };
                let (settle, wait) = oneshot::channel();
                let observer = context.observer.clone();
                pending.insert(
                    id.clone(),
                    Record {
                        permission: permission.clone(),
                        owner_id: clean(&context.owner_id).to_owned(),
                        scope_id: clean(&context.scope_id).to_owned(),
                        observer: observer.clone(),
                        settle,
                    },
                );
                // Registered first, announced second. A response that races the
                // announcement finds the record.
                if let Some(observer) = observer {
                    observer.requested(&permission);
                }
                let _ = reply.send(Ticket::Pending { id, wait });
            }
            Command::Respond {
                id,
                response,
                owner_id,
                reply,
            } => {
                let answer = respond(
                    &mut pending,
                    &mut resolved,
                    resolved_limit,
                    &id,
                    response,
                    &owner_id,
                );
                let _ = reply.send(answer);
            }
            Command::Cancel { id, reply } => {
                let _ = reply.send(cancel(&mut pending, &id));
            }
            Command::CancelScope { scope_id, reply } => {
                let scope = clean(&scope_id).to_owned();
                if scope.is_empty() {
                    let _ = reply.send(0);
                    continue;
                }
                let ids: Vec<String> = pending
                    .iter()
                    .filter(|(_, record)| record.scope_id == scope)
                    .map(|(id, _)| id.clone())
                    .collect();
                let count = ids.iter().filter(|id| cancel(&mut pending, id)).count();
                let _ = reply.send(count);
            }
            Command::CancelAll { reply } => {
                let ids: Vec<String> = pending.keys().cloned().collect();
                let count = ids.iter().filter(|id| cancel(&mut pending, id)).count();
                let _ = reply.send(count);
            }
            Command::Pending { reply } => {
                let _ = reply.send(
                    pending
                        .values()
                        .map(|record| record.permission.clone())
                        .collect(),
                );
            }
            Command::Resolved { reply } => {
                let _ = reply.send(
                    resolved
                        .values()
                        .map(|(_, permission)| permission.clone())
                        .collect(),
                );
            }
        }
    }

    // Shutdown: every question that is still open goes away, and every backend
    // waiting on one is told `cancelled` rather than being left hanging.
    let ids: Vec<String> = pending.keys().cloned().collect();
    for id in &ids {
        cancel(&mut pending, id);
    }
}

/// `respond`, `permission-broker.mjs:100-134`.
fn respond(
    pending: &mut IndexMap<String, Record>,
    resolved: &mut IndexMap<String, (String, PendingPermission)>,
    resolved_limit: usize,
    id: &str,
    response: PermissionResponse,
    owner_id: &str,
) -> Result<PendingPermission, CoordinatorError> {
    let owner = clean(owner_id);
    let Some(record) = pending.shift_remove(id) else {
        // Upstream checks the ledger *before* the owner check, so a late
        // duplicate from the right owner is idempotent and one from the wrong
        // owner is indistinguishable from an unknown id.
        return match resolved.get(id) {
            Some((remembered, permission)) if remembered == owner => Ok(permission.clone()),
            _ => Err(CoordinatorError::PermissionRequestUnknown),
        };
    };
    if record.owner_id != owner {
        // Put it back: another owner's mistake must not resolve this request.
        pending.insert(id.to_owned(), record);
        return Err(CoordinatorError::PermissionRequestUnknown);
    }

    let _ = record.settle.send(response.decision());
    let permission = PendingPermission {
        id: record.permission.id.clone(),
        work_id: record.permission.work_id.clone(),
        status: response.status(),
        category: record.permission.category.clone(),
        summary: record.permission.summary.clone(),
        patterns: None,
    };
    if let Some(observer) = record.observer.as_ref() {
        observer.resolved(&permission);
    }
    resolved.insert(
        permission.id.clone(),
        (record.owner_id.clone(), permission.clone()),
    );
    while resolved.len() > resolved_limit {
        resolved.shift_remove_index(0);
    }
    Ok(permission)
}

/// `cancel(record)`, `permission-broker.mjs:84-98`.
fn cancel(pending: &mut IndexMap<String, Record>, id: &str) -> bool {
    let Some(record) = pending.shift_remove(id) else {
        return false;
    };
    let _ = record.settle.send(PermissionDecision::Cancel);
    if let Some(observer) = record.observer.as_ref() {
        observer.resolved(&PendingPermission {
            id: record.permission.id.clone(),
            work_id: record.permission.work_id.clone(),
            status: PermissionStatus::Cancelled,
            category: record.permission.category.clone(),
            summary: record.permission.summary.clone(),
            patterns: None,
        });
    }
    true
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use serde_json::json;

    #[derive(Debug, Default)]
    struct Recorder {
        events: Mutex<Vec<(&'static str, PendingPermission)>>,
    }

    impl Recorder {
        fn events(&self) -> Vec<(&'static str, PendingPermission)> {
            self.events
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
        }
    }

    impl PermissionObserver for Recorder {
        fn requested(&self, permission: &PendingPermission) {
            self.events
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(("requested", permission.clone()));
        }

        fn resolved(&self, permission: &PendingPermission) {
            self.events
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(("resolved", permission.clone()));
        }
    }

    fn context(recorder: &Arc<Recorder>) -> PermissionContext {
        PermissionContext {
            owner_id: "owner-one".to_owned(),
            work_id: Some("work-one".to_owned()),
            session_id: "coordinator-session".to_owned(),
            scope_id: "prompt-one".to_owned(),
            observer: Some(Arc::clone(recorder) as Arc<dyn PermissionObserver>),
            signal: None,
        }
    }

    fn write_call() -> Value {
        json!({ "name": "write", "rawInput": { "path": "/tmp/file" } })
    }

    /// Yield until `probe` answers true, or fail saying what never happened.
    ///
    /// Bounded on purpose: these tests run on a paused, single-threaded
    /// runtime, so an unbounded `yield_now` loop turns a broken invariant into
    /// a hang — and a hung test tells nobody anything.
    const MAX_SETTLE_POLLS: usize = 10_000;

    async fn settle_until<F, Fut>(what: &str, mut probe: F)
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        for _ in 0..MAX_SETTLE_POLLS {
            if probe().await {
                return;
            }
            tokio::task::yield_now().await;
        }
        panic!("gave up waiting for {what} after {MAX_SETTLE_POLLS} polls");
    }

    #[test]
    fn a_permission_id_is_auth_plus_thirty_two_hex() {
        let id = new_permission_id();
        let hex = id.strip_prefix(PERMISSION_ID_PREFIX).expect("the prefix");
        assert_eq!(hex.len(), 32);
        assert!(
            hex.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
        assert_ne!(id, new_permission_id());
    }

    #[test]
    fn the_summary_joins_with_a_full_width_colon_and_drops_empty_halves() {
        assert_eq!(summary_of(&write_call()), "write：/tmp/file");
        assert_eq!(summary_of(&json!({ "name": "bash" })), "bash");
        assert_eq!(
            summary_of(&json!({ "rawInput": { "command": "npm test" } })),
            "npm test",
        );
        assert_eq!(summary_of(&json!({})), "");
    }

    #[test]
    fn the_detail_prefers_description_then_command_then_path() {
        assert_eq!(
            tool_detail(&json!({
                "rawInput": { "description": "d", "command": "c", "path": "p" },
            })),
            "d",
        );
        assert_eq!(
            tool_detail(&json!({ "rawInput": { "command": "c", "path": "p" } })),
            "c",
        );
        assert_eq!(tool_detail(&json!({ "rawInput": { "path": "p" } })), "p");
        assert_eq!(tool_detail(&json!({ "rawInput": {} })), "");
        assert_eq!(tool_detail(&json!({ "rawInput": "not an object" })), "");
    }

    #[test]
    fn a_nameless_tool_call_is_categorised_unknown() {
        assert_eq!(category_of(&json!({})), UNKNOWN_CATEGORY);
        assert_eq!(category_of(&json!({ "title": "Run tests" })), "Run tests");
        assert_eq!(
            category_of(&json!({ "name": "  ", "title": "Run tests" })),
            UNKNOWN_CATEGORY,
            "a whitespace-only name is truthy in JavaScript and wins the `||`",
        );
    }

    #[test]
    fn the_category_and_the_summary_are_bounded() {
        let long = "n".repeat(CATEGORY_BOUND + 40);
        let detail = "d".repeat(SUMMARY_DETAIL_BOUND + 40);
        let call = json!({ "name": long, "rawInput": { "description": detail } });
        assert_eq!(category_of(&call).chars().count(), CATEGORY_BOUND);
        assert_eq!(
            summary_of(&call).chars().count(),
            CATEGORY_BOUND + 1 + SUMMARY_DETAIL_BOUND,
        );
    }

    #[test]
    fn permission_mode_fails_closed() {
        assert_eq!(PermissionMode::from_wire("full"), PermissionMode::Full);
        assert_eq!(PermissionMode::from_wire(" full "), PermissionMode::Full);
        for native in ["native", "", "Full", "yolo"] {
            assert_eq!(
                PermissionMode::from_wire(native),
                PermissionMode::Native,
                "{native}"
            );
        }
        assert_eq!(PermissionMode::default(), PermissionMode::Native);
    }

    #[test]
    fn only_the_two_catalogued_decisions_parse() {
        assert_eq!(
            PermissionResponse::from_wire("always"),
            Some(PermissionResponse::Always)
        );
        assert_eq!(
            PermissionResponse::from_wire(" reject "),
            Some(PermissionResponse::Reject)
        );
        for refused in ["", "approve", "deny", "once", "ALWAYS"] {
            assert_eq!(PermissionResponse::from_wire(refused), None, "{refused}");
        }
        assert_eq!(
            PermissionResponse::Always.decision(),
            PermissionDecision::Approve,
        );
        assert_eq!(
            PermissionResponse::Reject.decision(),
            PermissionDecision::Reject
        );
        assert_eq!(
            PermissionResponse::Always.status(),
            PermissionStatus::Approved
        );
        assert_eq!(
            PermissionResponse::Reject.status(),
            PermissionStatus::Denied
        );
    }

    #[tokio::test(start_paused = true)]
    async fn vias_own_session_tools_are_approved_without_asking() {
        let broker = PermissionBroker::new(PermissionMode::Native);
        let recorder = Arc::new(Recorder::default());
        for name in [
            "via_session_start",
            "mcp__via__via_session_start",
            "via_session_start (build the thing)",
        ] {
            let decision = broker
                .request(&json!({ "title": name }), context(&recorder))
                .await
                .expect("running");
            assert_eq!(decision, PermissionDecision::Approve, "{name}");
        }
        assert!(broker.pending().await.is_empty());
        assert!(
            recorder.events().is_empty(),
            "nobody was asked, so nobody was told"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_backend_tool_is_asked_about() {
        let broker = PermissionBroker::new(PermissionMode::Native);
        let recorder = Arc::new(Recorder::default());
        let asking = {
            let broker = broker.clone();
            let context = context(&recorder);
            tokio::spawn(async move { broker.request(&write_call(), context).await })
        };
        settle_until("the permission to be registered", || async {
            !broker.pending().await.is_empty()
        })
        .await;

        let pending = broker.pending().await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].status, PermissionStatus::Pending);
        assert_eq!(pending[0].category, "write");
        assert_eq!(pending[0].summary, "write：/tmp/file");
        assert_eq!(pending[0].work_id.as_deref(), Some("work-one"));
        assert_eq!(pending[0].patterns.as_deref(), Some([].as_slice()));

        let permission = broker
            .respond(&pending[0].id, PermissionResponse::Always, "owner-one")
            .await
            .expect("resolved");
        assert_eq!(permission.status, PermissionStatus::Approved);
        assert_eq!(
            permission.patterns, None,
            "patterns is absent from a resolution"
        );
        assert_eq!(
            asking.await.expect("joined").expect("running"),
            PermissionDecision::Approve,
        );

        let events = recorder.events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].0, "requested");
        assert_eq!(events[1].0, "resolved");
        assert_eq!(events[1].1.status, PermissionStatus::Approved);
    }

    #[tokio::test(start_paused = true)]
    async fn full_permission_mode_never_asks() {
        let broker = PermissionBroker::new(PermissionMode::Full);
        let recorder = Arc::new(Recorder::default());
        let decision = broker
            .request(&write_call(), context(&recorder))
            .await
            .expect("running");
        assert_eq!(decision, PermissionDecision::Approve);
        assert!(broker.pending().await.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn a_response_from_another_owner_is_reported_as_unknown() {
        let broker = PermissionBroker::new(PermissionMode::Native);
        let recorder = Arc::new(Recorder::default());
        let asking = {
            let broker = broker.clone();
            let context = context(&recorder);
            tokio::spawn(async move { broker.request(&write_call(), context).await })
        };
        settle_until("the permission to be registered", || async {
            !broker.pending().await.is_empty()
        })
        .await;
        let id = broker.pending().await[0].id.clone();

        let error = broker
            .respond(&id, PermissionResponse::Always, "owner-two")
            .await
            .expect_err("not this owner's");
        assert_eq!(error.code(), "VIA_COORDINATOR_PERMISSION_UNKNOWN");
        assert_eq!(
            broker.pending().await.len(),
            1,
            "the request is still pending for the owner it belongs to",
        );

        broker
            .respond(&id, PermissionResponse::Reject, "owner-one")
            .await
            .expect("resolved");
        assert_eq!(
            asking.await.expect("joined").expect("running"),
            PermissionDecision::Reject,
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_repeated_response_from_the_same_owner_is_idempotent() {
        let broker = PermissionBroker::new(PermissionMode::Native);
        let recorder = Arc::new(Recorder::default());
        let asking = {
            let broker = broker.clone();
            let context = context(&recorder);
            tokio::spawn(async move { broker.request(&write_call(), context).await })
        };
        settle_until("the permission to be registered", || async {
            !broker.pending().await.is_empty()
        })
        .await;
        let id = broker.pending().await[0].id.clone();

        let first = broker
            .respond(&id, PermissionResponse::Always, "owner-one")
            .await
            .expect("resolved");
        let second = broker
            .respond(&id, PermissionResponse::Reject, "owner-one")
            .await
            .expect("remembered");
        assert_eq!(
            first, second,
            "the remembered decision wins over the new one"
        );
        assert!(
            broker
                .respond(&id, PermissionResponse::Always, "owner-two")
                .await
                .is_err(),
            "the ledger is owner-scoped too",
        );
        asking.await.expect("joined").expect("running");
    }

    #[tokio::test(start_paused = true)]
    async fn an_unknown_id_is_refused() {
        let broker = PermissionBroker::new(PermissionMode::Native);
        let error = broker
            .respond("auth_missing", PermissionResponse::Always, "owner-one")
            .await
            .expect_err("unknown");
        assert!(matches!(error, CoordinatorError::PermissionRequestUnknown));
    }

    #[tokio::test(start_paused = true)]
    async fn the_resolved_ledger_evicts_oldest_first() {
        let broker = PermissionBroker::with_resolved_limit(PermissionMode::Native, 2);
        let recorder = Arc::new(Recorder::default());
        let mut ids = Vec::new();
        for _ in 0..3 {
            let asking = {
                let broker = broker.clone();
                let context = context(&recorder);
                tokio::spawn(async move { broker.request(&write_call(), context).await })
            };
            settle_until("the permission to be registered", || async {
                !broker.pending().await.is_empty()
            })
            .await;
            let id = broker.pending().await[0].id.clone();
            broker
                .respond(&id, PermissionResponse::Always, "owner-one")
                .await
                .expect("resolved");
            asking.await.expect("joined").expect("running");
            ids.push(id);
        }
        assert_eq!(broker.resolved().await.len(), 2);
        assert!(
            broker
                .respond(&ids[0], PermissionResponse::Always, "owner-one")
                .await
                .is_err(),
            "the oldest was evicted",
        );
        assert!(
            broker
                .respond(&ids[2], PermissionResponse::Always, "owner-one")
                .await
                .is_ok(),
        );
    }

    #[tokio::test(start_paused = true)]
    async fn cancelling_a_scope_leaves_every_other_scope_alone() {
        let broker = PermissionBroker::new(PermissionMode::Native);
        let recorder = Arc::new(Recorder::default());

        let coordinator = {
            let broker = broker.clone();
            let context = PermissionContext {
                scope_id: "coordinator-prompt".to_owned(),
                ..context(&recorder)
            };
            tokio::spawn(async move { broker.request(&json!({ "name": "read" }), context).await })
        };
        settle_until("the permission to be registered", || async {
            !broker.pending().await.is_empty()
        })
        .await;
        let project = {
            let broker = broker.clone();
            let context = PermissionContext {
                scope_id: "project-prompt".to_owned(),
                observer: None,
                ..context(&recorder)
            };
            tokio::spawn(async move { broker.request(&write_call(), context).await })
        };
        settle_until("the second permission to be registered", || async {
            broker.pending().await.len() >= 2
        })
        .await;

        assert_eq!(broker.cancel_scope("coordinator-prompt").await, 1);
        assert_eq!(
            coordinator.await.expect("joined").expect("running"),
            PermissionDecision::Cancel,
        );
        let last = recorder.events().pop().expect("an event");
        assert_eq!(last.0, "resolved");
        assert_eq!(last.1.status, PermissionStatus::Cancelled);

        let still_pending = broker.pending().await;
        assert_eq!(still_pending.len(), 1);
        broker
            .respond(
                &still_pending[0].id,
                PermissionResponse::Always,
                "owner-one",
            )
            .await
            .expect("resolved");
        assert_eq!(
            project.await.expect("joined").expect("running"),
            PermissionDecision::Approve,
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_blank_scope_cancels_nothing() {
        let broker = PermissionBroker::new(PermissionMode::Native);
        let recorder = Arc::new(Recorder::default());
        let asking = {
            let broker = broker.clone();
            let context = PermissionContext {
                scope_id: String::new(),
                ..context(&recorder)
            };
            tokio::spawn(async move { broker.request(&write_call(), context).await })
        };
        settle_until("the permission to be registered", || async {
            !broker.pending().await.is_empty()
        })
        .await;
        assert_eq!(broker.cancel_scope("   ").await, 0);
        assert_eq!(broker.cancel_scope("").await, 0);
        assert_eq!(broker.pending().await.len(), 1);
        assert_eq!(broker.cancel_all().await, 1);
        assert_eq!(
            asking.await.expect("joined").expect("running"),
            PermissionDecision::Cancel,
        );
    }

    #[tokio::test(start_paused = true)]
    async fn an_aborted_turn_takes_its_question_with_it() {
        let broker = PermissionBroker::new(PermissionMode::Native);
        let recorder = Arc::new(Recorder::default());
        let signal = CancellationToken::new();
        let asking = {
            let broker = broker.clone();
            let context = PermissionContext {
                signal: Some(signal.clone()),
                ..context(&recorder)
            };
            tokio::spawn(async move { broker.request(&write_call(), context).await })
        };
        settle_until("the permission to be registered", || async {
            !broker.pending().await.is_empty()
        })
        .await;
        signal.cancel();
        assert_eq!(
            asking.await.expect("joined").expect("running"),
            PermissionDecision::Cancel,
        );
        assert!(broker.pending().await.is_empty());
    }
}
