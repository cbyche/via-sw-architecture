//! What the five tools are wired to, and the shapes that cross the wire.
//!
//! Upstream's tool handlers are five closures on a plain object built by
//! `AcpBackendAdapter.toolContext(run)`
//! (`server/src/agent/acp-backend-adapter.mjs:932-938`); the payloads they
//! return are built at `:696-928`. Here the object is a trait and the payloads
//! are types, for one reason: every one of these values is **parsed by the
//! model**, so the field names, the field order and — for two of them — the
//! *conditional presence* of a field are contracts. A `serde_json::Value`
//! returned from a handler would put all of that beyond the reach of a
//! compiler.
//!
//! # Arguments are validated the way `zod` validates them
//!
//! Upstream declares each tool's arguments as a `zod` object and the MCP SDK
//! rejects anything that does not parse before the handler runs. The three
//! behaviours that matters for are reproduced in [`SessionsListInput`] and its
//! siblings:
//!
//! * a **missing** optional field is fine, an explicit `null` is not — `zod`'s
//!   `.optional()` admits `undefined`, never `null`;
//! * **unknown properties are dropped**, not rejected — a bare `z.object` is
//!   not `.strict()`;
//! * a **required** field that is absent, of the wrong type, or empty where
//!   `.min(1)` was declared, fails the call before the Gateway is touched.

use core::fmt;

use async_trait::async_trait;
use serde::Serialize;
use serde::ser::SerializeMap;
use serde_json::{Map, Value};
use via_downstream::text::{bounded, clean, is_js_whitespace};
use via_downstream::{CancelOutcome, HarnessError};
use via_protocol::WorkStatus;

use crate::tools::SessionTool;

/// The `status` literal `via_session_start` and `via_session_send` answer with.
///
/// **External contract.** `docs/reference/contracts.json`
/// (`tool-name / session_start`): *"'started' (not 'ok'/'accepted') is the
/// exact literal the coordinator model is instructed to key off before
/// returning state=delegated."*
pub const STATUS_STARTED: &str = "started";

/// The bound a listed Session's title is rendered at.
///
/// **External contract.** Upstream `sessionSummary`,
/// `server/src/agent/acp-backend-session-utils.mjs:69-76`.
pub const SESSION_TITLE_BOUND: usize = 160;

/// The bound a completed delegation's result is clipped to before the model
/// sees it.
///
/// **External contract.** Upstream `statusForDelegation`,
/// `server/src/agent/acp-backend-adapter.mjs:899`:
/// `clean(record.result?.content).slice(0, 4000)`. Note this is a *clip*, not
/// a [`bounded`] — interior whitespace is preserved, because the model is
/// reading prose.
pub const STATUS_RESULT_BOUND: usize = 4000;

/// The smallest `limit` `via_sessions_list` accepts.
///
/// **External contract** — the `z.number().int().min(1).max(100)` bound in
/// upstream `acp-session-tools.mjs:40`.
pub const SESSION_LIST_LIMIT_MIN: i64 = 1;

/// The largest `limit` `via_sessions_list` accepts.
///
/// **External contract** — as [`SESSION_LIST_LIMIT_MIN`]. It is also
/// upstream's `MAX_SESSION_RESULTS` and the `session/list` hard page cap.
pub const SESSION_LIST_LIMIT_MAX: i64 = 100;

/// The `limit` used when the model sends none.
///
/// **External contract** — upstream `listProjectSessions`,
/// `acp-backend-adapter.mjs:698`: `limit: limit || 20`. It lives beside the
/// schema that bounds it so an implementer has nothing to restate.
pub const DEFAULT_SESSION_LIST_LIMIT: i64 = 20;

/// `clean(value).slice(0, max)`, counting UTF-16 code units.
///
/// The plain clip upstream applies to a delegation result. It is deliberately
/// *not* [`bounded`]: that collapses interior whitespace, which would reflow
/// the prose the model is about to read.
///
/// # The one deviation
///
/// `String.prototype.slice` counts UTF-16 code units and will cut an astral
/// character in half. A Rust `String` cannot hold a lone surrogate, so this
/// stops one character early instead — identical to upstream for every string
/// inside the Basic Multilingual Plane, and the same trade
/// [`bounded`] already records.
#[must_use]
pub fn clip(value: &str, max: usize) -> String {
    let trimmed = clean(value);
    let mut out = String::new();
    let mut units = 0usize;
    for c in trimmed.chars() {
        let width = c.len_utf16();
        if units + width > max {
            break;
        }
        units += width;
        out.push(c);
    }
    out
}

// ---------------------------------------------------------------------------
// Argument validation
// ---------------------------------------------------------------------------

/// One reason a tool call's arguments did not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaViolation {
    /// The property it is about.
    pub property: &'static str,
    /// What was wrong with it.
    pub problem: String,
}

impl fmt::Display for SchemaViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.property, self.problem)
    }
}

/// A tool call whose arguments do not satisfy the tool's JSON Schema.
///
/// The message shape is the MCP SDK's — `Invalid arguments for tool <name>: `
/// followed by the parser's detail — and it reaches the model as a JSON-RPC
/// `-32602`, never as the tool's own error envelope. Upstream's detail is the
/// `zod` issue list; VIA's is the deterministic rendering of
/// [`Self::violations`], because the SDK's formatting is not part of the port.
/// Recorded in `docs/deviations/phase-3.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidArguments {
    /// The tool that was called.
    pub tool: SessionTool,
    /// Every violation, in schema-declaration order.
    pub violations: Vec<SchemaViolation>,
}

impl fmt::Display for InvalidArguments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid arguments for tool {}: ", self.tool.name())?;
        for (index, violation) in self.violations.iter().enumerate() {
            if index > 0 {
                f.write_str("; ")?;
            }
            write!(f, "{violation}")?;
        }
        Ok(())
    }
}

impl std::error::Error for InvalidArguments {}

/// The JSON type name `zod` reports for a value, for its `Expected x, received
/// y` messages.
fn type_name(value: Option<&Value>) -> &'static str {
    match value {
        None => "undefined",
        Some(Value::Null) => "null",
        Some(Value::Bool(_)) => "boolean",
        Some(Value::Number(_)) => "number",
        Some(Value::String(_)) => "string",
        Some(Value::Array(_)) => "array",
        Some(Value::Object(_)) => "object",
    }
}

/// A reader over one tool call's `arguments` object.
///
/// Accumulates violations rather than stopping at the first, so a model that
/// got two fields wrong is told about both.
struct Arguments<'a> {
    object: Option<&'a Map<String, Value>>,
    violations: Vec<SchemaViolation>,
}

impl<'a> Arguments<'a> {
    /// `arguments` may be absent entirely — the SDK passes `{}` — but if it is
    /// present it must be an object.
    fn new(raw: Option<&'a Value>) -> Self {
        let (object, violations) = match raw {
            None | Some(Value::Null) => (None, Vec::new()),
            Some(Value::Object(object)) => (Some(object), Vec::new()),
            Some(other) => (
                None,
                vec![SchemaViolation {
                    property: "arguments",
                    problem: format!("Expected object, received {}", type_name(Some(other))),
                }],
            ),
        };
        Self { object, violations }
    }

    fn get(&self, property: &str) -> Option<&'a Value> {
        self.object.and_then(|object| object.get(property))
    }

    fn fail(&mut self, property: &'static str, problem: String) {
        self.violations.push(SchemaViolation { property, problem });
    }

    /// `z.string()` — required.
    fn required_string(&mut self, property: &'static str, min_length: usize) -> String {
        match self.get(property) {
            Some(Value::String(text)) => {
                if text.chars().count() < min_length {
                    self.fail(
                        property,
                        format!("String must contain at least {min_length} character(s)"),
                    );
                }
                text.clone()
            }
            other => {
                self.fail(
                    property,
                    if other.is_none() {
                        "Required".to_owned()
                    } else {
                        format!("Expected string, received {}", type_name(other))
                    },
                );
                String::new()
            }
        }
    }

    /// `z.string().optional()` — absent is fine, `null` is not.
    fn optional_string(&mut self, property: &'static str) -> Option<String> {
        match self.get(property) {
            None => None,
            Some(Value::String(text)) => Some(text.clone()),
            other => {
                self.fail(
                    property,
                    format!("Expected string, received {}", type_name(other)),
                );
                None
            }
        }
    }

    /// `z.number().int().min(min).max(max).optional()`.
    fn optional_bounded_integer(
        &mut self,
        property: &'static str,
        min: i64,
        max: i64,
    ) -> Option<i64> {
        let value = self.get(property)?;
        let Some(number) = value.as_f64() else {
            self.fail(
                property,
                format!("Expected number, received {}", type_name(Some(value))),
            );
            return None;
        };
        let Some(integer) = value.as_i64().filter(|_| number.fract() == 0.0) else {
            self.fail(property, "Expected integer, received float".to_owned());
            return None;
        };
        if integer < min {
            self.fail(
                property,
                format!("Number must be greater than or equal to {min}"),
            );
            return None;
        }
        if integer > max {
            self.fail(
                property,
                format!("Number must be less than or equal to {max}"),
            );
            return None;
        }
        Some(integer)
    }

    fn finish<T>(self, tool: SessionTool, parsed: T) -> Result<T, InvalidArguments> {
        if self.violations.is_empty() {
            Ok(parsed)
        } else {
            Err(InvalidArguments {
                tool,
                violations: self.violations,
            })
        }
    }
}

/// Arguments for `via_sessions_list`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionsListInput {
    /// A case-insensitive substring filter over `title + ' ' + directory`.
    pub query: Option<String>,
    /// How many Sessions to page, bounded 1..=100.
    pub limit: Option<i64>,
}

impl SessionsListInput {
    /// Parse a tool call's `arguments`.
    ///
    /// # Errors
    ///
    /// [`InvalidArguments`] when the object does not satisfy
    /// [`SessionTool::SessionsList`]'s schema.
    pub fn parse(raw: Option<&Value>) -> Result<Self, InvalidArguments> {
        let mut arguments = Arguments::new(raw);
        let query = arguments.optional_string("query");
        let limit = arguments.optional_bounded_integer(
            "limit",
            SESSION_LIST_LIMIT_MIN,
            SESSION_LIST_LIMIT_MAX,
        );
        arguments.finish(SessionTool::SessionsList, Self { query, limit })
    }

    /// The page size to ask the harness for.
    ///
    /// Upstream's `limit || 20`, which — being JavaScript — also treats an
    /// explicit `0` as absent. The schema's minimum is 1, so `0` cannot reach
    /// here and the two agree.
    #[must_use]
    pub fn effective_limit(&self) -> i64 {
        self.limit.unwrap_or(DEFAULT_SESSION_LIST_LIMIT)
    }

    /// The lower-cased needle the harness filters with, or `None` when the
    /// model sent no filter.
    ///
    /// Upstream `clean(query).toLowerCase()`, then `!needle || …` — an
    /// all-whitespace query is no filter at all.
    #[must_use]
    pub fn needle(&self) -> Option<String> {
        let needle = clean(self.query.as_deref().unwrap_or_default()).to_lowercase();
        (!needle.is_empty()).then_some(needle)
    }
}

/// Arguments for `via_session_start`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionStartInput {
    /// The natural task text. Required, non-empty.
    pub prompt: String,
    /// An optional title for the Session.
    pub title: Option<String>,
}

impl SessionStartInput {
    /// Parse a tool call's `arguments`.
    ///
    /// # Errors
    ///
    /// [`InvalidArguments`] when the object does not satisfy
    /// [`SessionTool::SessionStart`]'s schema.
    pub fn parse(raw: Option<&Value>) -> Result<Self, InvalidArguments> {
        let mut arguments = Arguments::new(raw);
        let prompt = arguments.required_string("prompt", 1);
        let title = arguments.optional_string("title");
        arguments.finish(SessionTool::SessionStart, Self { prompt, title })
    }
}

/// Arguments for `via_session_send`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSendInput {
    /// The exact `session_id` the Session list returned.
    pub session_id: String,
    /// The natural task text. Required, non-empty.
    pub prompt: String,
}

impl SessionSendInput {
    /// Parse a tool call's `arguments`.
    ///
    /// # Errors
    ///
    /// [`InvalidArguments`] when the object does not satisfy
    /// [`SessionTool::SessionSend`]'s schema.
    pub fn parse(raw: Option<&Value>) -> Result<Self, InvalidArguments> {
        let mut arguments = Arguments::new(raw);
        let session_id = arguments.required_string("session_id", 1);
        let prompt = arguments.required_string("prompt", 1);
        arguments.finish(SessionTool::SessionSend, Self { session_id, prompt })
    }
}

/// Arguments for `via_session_status` and `via_session_cancel`.
///
/// One type for both because upstream declares one schema for both and looks
/// both up through the same `findDelegation`
/// (`acp-backend-adapter.mjs:727-735`), which matches on *either* id.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DelegationLookupInput {
    /// The delegation id VIA issued.
    pub delegation_id: Option<String>,
    /// The backend's own session id.
    pub session_id: Option<String>,
}

impl DelegationLookupInput {
    /// Parse a tool call's `arguments`.
    ///
    /// # Errors
    ///
    /// [`InvalidArguments`] when the object does not satisfy `tool`'s schema.
    /// `tool` is carried only so the message names the tool that was called.
    pub fn parse(tool: SessionTool, raw: Option<&Value>) -> Result<Self, InvalidArguments> {
        let mut arguments = Arguments::new(raw);
        let delegation_id = arguments.optional_string("delegation_id");
        let session_id = arguments.optional_string("session_id");
        arguments.finish(
            tool,
            Self {
                delegation_id,
                session_id,
            },
        )
    }

    /// Whether `record` is the delegation this lookup names.
    ///
    /// Upstream `findDelegation` (`acp-backend-adapter.mjs:727-735`): a
    /// *cleaned, non-empty* `delegation_id` matching the record's id, **or** a
    /// cleaned, non-empty `session_id` matching the record's session id. A
    /// lookup that names neither matches nothing, which is why an empty call
    /// answers `not_found` rather than picking the first delegation it finds.
    #[must_use]
    pub fn matches(&self, record: &DelegationRecord) -> bool {
        fn named(value: &Option<String>) -> Option<&str> {
            let value = clean(value.as_deref().unwrap_or_default());
            (!value.is_empty()).then_some(value)
        }
        named(&self.delegation_id).is_some_and(|id| id == record.delegation_id)
            || named(&self.session_id).is_some_and(|id| id == record.session_id)
    }
}

// ---------------------------------------------------------------------------
// Results
// ---------------------------------------------------------------------------

/// One row of `via_sessions_list`'s answer.
///
/// **External contract** — upstream `sessionSummary`,
/// `server/src/agent/acp-backend-session-utils.mjs:69-76`. The field names are
/// `snake_case` and deliberately differ from the `camelCase` ACP fields they
/// are derived from; the order is the catalogued order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SessionSummary {
    /// The backend's session id.
    pub session_id: String,
    /// The Session's title, [`bounded`] to [`SESSION_TITLE_BOUND`].
    pub title: String,
    /// The Session's project directory — ACP's `cwd`.
    pub directory: String,
    /// When it last changed, as the backend reported it.
    pub updated_at: String,
}

impl SessionSummary {
    /// Build a summary, applying upstream's `clean`/`bounded` treatment.
    #[must_use]
    pub fn new(session_id: &str, title: &str, directory: &str, updated_at: &str) -> Self {
        Self {
            session_id: clean(session_id).to_owned(),
            title: bounded(title, SESSION_TITLE_BOUND),
            directory: clean(directory).to_owned(),
            updated_at: clean(updated_at).to_owned(),
        }
    }

    /// The haystack upstream's substring filter runs over:
    /// `[title, directory].join(' ').toLowerCase()`.
    #[must_use]
    pub fn filter_haystack(&self) -> String {
        format!("{} {}", self.title, self.directory).to_lowercase()
    }
}

/// `via_sessions_list`'s answer.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SessionsListResult {
    /// The matching Sessions, coordinator Sessions already filtered out.
    pub sessions: Vec<SessionSummary>,
}

impl SessionsListResult {
    /// An answer carrying these Sessions.
    #[must_use]
    pub fn new(sessions: Vec<SessionSummary>) -> Self {
        Self { sessions }
    }
}

/// What `via_session_start` and `via_session_send` answer with.
///
/// **External contract** — upstream `startProjectSession` /
/// `continueProjectSession` (`acp-backend-adapter.mjs:804-887`). Both return
/// the identical shape, `status` first and always [`STATUS_STARTED`].
///
/// The `delegation_id` is **opaque**. It is the correlation handle the Gateway
/// answers [`SessionTool::SessionStatus`] and
/// [`SessionTool::SessionCancel`] against, and the *only* thing the backend
/// agent is expected to do with it is hand it back in the delegated response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegationStarted {
    /// The opaque delegation id.
    pub delegation_id: String,
    /// The project Session the work went to.
    pub session_id: String,
    /// The Session's title.
    pub title: String,
    /// The Session's project directory.
    pub directory: String,
}

impl Serialize for DelegationStarted {
    /// `{status, delegation_id, session_id, title, directory}`, in the
    /// catalogued order with the constant `status` first.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(5))?;
        map.serialize_entry("status", STATUS_STARTED)?;
        map.serialize_entry("delegation_id", &self.delegation_id)?;
        map.serialize_entry("session_id", &self.session_id)?;
        map.serialize_entry("title", &self.title)?;
        map.serialize_entry("directory", &self.directory)?;
        map.end()
    }
}

/// The status vocabulary a delegation is reported in.
///
/// **External contract** — upstream `record.status`
/// (`acp-backend-adapter.mjs:750,777,781`) is one of `running`, `completed`,
/// `failed`, `cancelled`. The spellings are not restated here: each projects
/// to a [`WorkStatus`] and takes its wire form from
/// [`WorkStatus::as_str`], so the delegation vocabulary and the Work
/// vocabulary cannot drift apart.
///
/// # `cancelling` is VIA's
///
/// `via-downstream` already reports a delivered-but-unconfirmed cancel as
/// `cancelling` rather than upstream's optimistic `cancelled`
/// (`docs/deviations/phase-2.md`, `docs/architecture.md` §4: *"cancelling is a
/// state, not an action"*). A status query about such a delegation has to be
/// able to say so, so the variant exists here too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DelegationStatus {
    /// The delegated Session is working.
    Running,
    /// It finished and produced a result.
    Completed,
    /// It ended in a failure.
    Failed,
    /// A cancel was delivered; the harness has not confirmed it yet.
    Cancelling,
    /// It was cancelled, confirmed.
    Cancelled,
}

impl DelegationStatus {
    /// Every status, in the order upstream lists them plus VIA's own.
    pub const ALL: [DelegationStatus; 5] = [
        Self::Running,
        Self::Completed,
        Self::Failed,
        Self::Cancelling,
        Self::Cancelled,
    ];

    /// The Work status this projects to.
    #[must_use]
    pub const fn work_status(self) -> WorkStatus {
        match self {
            Self::Running => WorkStatus::Running,
            Self::Completed => WorkStatus::Completed,
            Self::Failed => WorkStatus::Failed,
            Self::Cancelling => WorkStatus::Cancelling,
            Self::Cancelled => WorkStatus::Cancelled,
        }
    }

    /// The wire spelling, taken from [`WorkStatus`].
    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.work_status().as_str()
    }

    /// Whether the delegation has stopped for good.
    ///
    /// Upstream's short-circuit list,
    /// `['completed', 'failed', 'cancelled'].includes(record.status)`
    /// (`acp-backend-adapter.mjs:910`). `cancelling` is deliberately **not**
    /// terminal: that is the whole point of having it.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

impl fmt::Display for DelegationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How a delegation is doing, with the one extra field its state allows.
///
/// The pairing is the contract: `result` appears **only** on a completed
/// delegation and `error` **only** on a failed one
/// (`docs/reference/contracts.json`, `tool-name / session_status`). Carrying
/// the detail inside the state rather than beside it is what makes the other
/// six combinations unrepresentable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegationOutcome {
    /// Working.
    Running,
    /// A cancel was delivered and is not confirmed.
    Cancelling,
    /// Cancelled, confirmed.
    Cancelled,
    /// Finished, with the result the model is shown.
    Completed {
        /// The delegated Session's answer, clipped to
        /// [`STATUS_RESULT_BOUND`].
        result: String,
    },
    /// Failed, with the message the model is shown.
    Failed {
        /// The failure, cleaned.
        error: String,
    },
}

impl DelegationOutcome {
    /// A completed delegation, applying upstream's `clean(...).slice(0, 4000)`.
    #[must_use]
    pub fn completed(result: &str) -> Self {
        Self::Completed {
            result: clip(result, STATUS_RESULT_BOUND),
        }
    }

    /// A failed delegation, applying upstream's `clean(...)`.
    ///
    /// Upstream does not bound this one
    /// (`acp-backend-adapter.mjs:903`), and neither does VIA: it is the
    /// harness's own already-localized sentence, not backend-authored prose.
    #[must_use]
    pub fn failed(error: &str) -> Self {
        Self::Failed {
            error: clean(error).to_owned(),
        }
    }

    /// The status this outcome reports.
    #[must_use]
    pub const fn status(&self) -> DelegationStatus {
        match self {
            Self::Running => DelegationStatus::Running,
            Self::Cancelling => DelegationStatus::Cancelling,
            Self::Cancelled => DelegationStatus::Cancelled,
            Self::Completed { .. } => DelegationStatus::Completed,
            Self::Failed { .. } => DelegationStatus::Failed,
        }
    }
}

/// The four identity fields every delegation answer carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegationRecord {
    /// The opaque delegation id.
    pub delegation_id: String,
    /// The project Session it runs in.
    pub session_id: String,
    /// The Session's title.
    pub title: String,
    /// The Session's project directory.
    pub directory: String,
}

impl DelegationRecord {
    /// Build a record, applying upstream's `clean`/`bounded` treatment.
    #[must_use]
    pub fn new(delegation_id: &str, session_id: &str, title: &str, directory: &str) -> Self {
        Self {
            delegation_id: clean(delegation_id).to_owned(),
            session_id: clean(session_id).to_owned(),
            title: bounded(title, SESSION_TITLE_BOUND),
            directory: clean(directory).to_owned(),
        }
    }

    /// The `started` answer for this record.
    #[must_use]
    pub fn started(&self) -> DelegationStarted {
        DelegationStarted {
            delegation_id: self.delegation_id.clone(),
            session_id: self.session_id.clone(),
            title: self.title.clone(),
            directory: self.directory.clone(),
        }
    }
}

/// What `via_session_status` answers with.
///
/// **External contract** — upstream `statusForDelegation`
/// (`acp-backend-adapter.mjs:889-905`). [`Self::NotFound`] serializes **bare**:
/// `{"status":"not_found"}` and nothing else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionStatusResult {
    /// No delegation answers to either id.
    NotFound,
    /// The delegation, and how it is doing.
    Known {
        /// Its identity.
        record: DelegationRecord,
        /// Its state, with the one field that state allows.
        outcome: DelegationOutcome,
    },
}

impl SessionStatusResult {
    /// A known delegation.
    #[must_use]
    pub fn known(record: DelegationRecord, outcome: DelegationOutcome) -> Self {
        Self::Known { record, outcome }
    }

    /// The `status` string this answer reports.
    #[must_use]
    pub fn status_str(&self) -> &'static str {
        match self {
            Self::NotFound => via_downstream::STATUS_NOT_FOUND,
            Self::Known { outcome, .. } => outcome.status().as_str(),
        }
    }
}

impl Serialize for SessionStatusResult {
    /// The catalogued field order, with `result`/`error` present only where
    /// the outcome allows.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::NotFound => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("status", self.status_str())?;
                map.end()
            }
            Self::Known { record, outcome } => {
                let extra = usize::from(!matches!(
                    outcome,
                    DelegationOutcome::Running
                        | DelegationOutcome::Cancelling
                        | DelegationOutcome::Cancelled
                ));
                let mut map = serializer.serialize_map(Some(5 + extra))?;
                map.serialize_entry("status", self.status_str())?;
                map.serialize_entry("delegation_id", &record.delegation_id)?;
                map.serialize_entry("session_id", &record.session_id)?;
                map.serialize_entry("title", &record.title)?;
                map.serialize_entry("directory", &record.directory)?;
                match outcome {
                    DelegationOutcome::Completed { result } => {
                        map.serialize_entry("result", result)?;
                    }
                    DelegationOutcome::Failed { error } => {
                        map.serialize_entry("error", error)?;
                    }
                    DelegationOutcome::Running
                    | DelegationOutcome::Cancelling
                    | DelegationOutcome::Cancelled => {}
                }
                map.end()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The seam
// ---------------------------------------------------------------------------

/// What the five tools are wired to.
///
/// Upstream's `toolContext(run)` — five closures over one coordination run
/// (`server/src/agent/acp-backend-adapter.mjs:932-938`). A registration owns
/// one of these and may swap it for another
/// ([`SessionToolRegistration::update`](crate::SessionToolRegistration::update))
/// without disturbing the descriptor the backend already connected to, which
/// is why the trait is behind an `Arc` rather than owned by the server.
///
/// # Errors are the tool's, not the transport's
///
/// Every method returns [`HarnessError`], and every failure becomes the
/// **error envelope** — `{"status":"failed","error":"…"}` with `isError:true`
/// — not a JSON-RPC error. Upstream wraps each handler in `try/catch` for
/// exactly this reason: a coordination tool that fails should tell the model
/// what happened in the same channel it would have answered in.
#[async_trait]
pub trait SessionToolContext: Send + Sync + fmt::Debug {
    /// `via_sessions_list` — the project Sessions the model may continue.
    ///
    /// # Errors
    ///
    /// Whatever the harness reports while listing.
    async fn list_sessions(
        &self,
        input: SessionsListInput,
    ) -> Result<SessionsListResult, HarnessError>;

    /// `via_session_start` — open a new project Session and delegate to it.
    ///
    /// # Errors
    ///
    /// Whatever the harness reports while opening or submitting.
    async fn start_session(
        &self,
        input: SessionStartInput,
    ) -> Result<DelegationStarted, HarnessError>;

    /// `via_session_send` — continue an existing project Session.
    ///
    /// # Errors
    ///
    /// Whatever the harness reports, including the catalogued refusal when the
    /// Session's project directory cannot be recovered.
    async fn send_session(
        &self,
        input: SessionSendInput,
    ) -> Result<DelegationStarted, HarnessError>;

    /// `via_session_status` — observe a delegation. **Never** acts on it.
    ///
    /// # Errors
    ///
    /// Whatever the harness reports. A delegation that does not exist is
    /// [`SessionStatusResult::NotFound`], which is an answer rather than an
    /// error.
    async fn session_status(
        &self,
        input: DelegationLookupInput,
    ) -> Result<SessionStatusResult, HarnessError>;

    /// `via_session_cancel` — cancel a delegated project Session.
    ///
    /// Answers with `via-downstream`'s [`CancelOutcome`], which already
    /// serializes to the catalogued `{status, delegation_id, session_id}` and
    /// already refuses to report an unconfirmed cancel as done.
    ///
    /// # Errors
    ///
    /// Whatever the harness reports. Nothing to cancel is
    /// [`CancelOutcome::NotFound`], not an error.
    async fn cancel_session(
        &self,
        input: DelegationLookupInput,
    ) -> Result<CancelOutcome, HarnessError>;
}

/// Whether `haystack` contains `needle`, case-insensitively, the way
/// upstream's `String.prototype.includes` over two lower-cased strings does.
///
/// Exposed because the filter is part of `via_sessions_list`'s catalogued
/// behaviour and an implementer should not have to rediscover that the
/// haystack is `title + ' ' + directory`.
#[must_use]
pub fn matches_query(summary: &SessionSummary, needle: Option<&str>) -> bool {
    needle.is_none_or(|needle| summary.filter_haystack().contains(needle))
}

/// Whether `value` holds any character ECMAScript's `\s` matches.
///
/// A thin re-export of `via-downstream`'s predicate, kept here so this
/// module's own callers do not each reach across the crate boundary for it.
#[must_use]
pub fn contains_js_whitespace(value: &str) -> bool {
    value.chars().any(is_js_whitespace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn an_absent_optional_is_fine_and_an_explicit_null_is_not() {
        assert_eq!(
            SessionsListInput::parse(Some(&json!({}))),
            Ok(SessionsListInput::default()),
        );
        let error = SessionsListInput::parse(Some(&json!({ "query": null })))
            .expect_err("null is not undefined");
        assert_eq!(error.violations.len(), 1);
        assert_eq!(error.violations[0].property, "query");
        assert_eq!(
            error.violations[0].problem,
            "Expected string, received null"
        );
    }

    #[test]
    fn unknown_properties_are_dropped_not_rejected() {
        let parsed = SessionsListInput::parse(Some(&json!({ "nope": 1, "query": "x" })))
            .expect("a bare z.object is not strict");
        assert_eq!(parsed.query.as_deref(), Some("x"));
    }

    #[test]
    fn missing_arguments_are_the_same_as_an_empty_object() {
        assert_eq!(
            DelegationLookupInput::parse(SessionTool::SessionStatus, None),
            Ok(DelegationLookupInput::default()),
        );
    }

    #[test]
    fn a_required_string_must_be_present_typed_and_non_empty() {
        let missing = SessionStartInput::parse(Some(&json!({}))).expect_err("prompt is required");
        assert_eq!(missing.violations[0].problem, "Required");

        let typed = SessionStartInput::parse(Some(&json!({ "prompt": 7 })))
            .expect_err("prompt is a string");
        assert_eq!(
            typed.violations[0].problem,
            "Expected string, received number",
        );

        let empty =
            SessionStartInput::parse(Some(&json!({ "prompt": "" }))).expect_err("prompt is min(1)");
        assert_eq!(
            empty.violations[0].problem,
            "String must contain at least 1 character(s)",
        );
    }

    #[test]
    fn a_whitespace_only_prompt_still_satisfies_min_one() {
        // `z.string().min(1)` counts characters, it does not trim. Upstream
        // cleans the prompt afterwards, in the adapter.
        assert!(SessionStartInput::parse(Some(&json!({ "prompt": " " }))).is_ok());
    }

    #[test]
    fn every_violation_is_reported_not_just_the_first() {
        let error = SessionSendInput::parse(Some(&json!({}))).expect_err("both required");
        assert_eq!(
            error
                .violations
                .iter()
                .map(|v| v.property)
                .collect::<Vec<_>>(),
            ["session_id", "prompt"],
        );
        assert_eq!(
            error.to_string(),
            "Invalid arguments for tool via_session_send: session_id: Required; prompt: Required",
        );
    }

    #[test]
    fn limit_is_a_bounded_integer() {
        let ok = SessionsListInput::parse(Some(&json!({ "limit": 100 }))).expect("in range");
        assert_eq!(ok.limit, Some(100));

        for (value, problem) in [
            (json!(0), "Number must be greater than or equal to 1"),
            (json!(101), "Number must be less than or equal to 100"),
            (json!(2.5), "Expected integer, received float"),
            (json!("20"), "Expected number, received string"),
        ] {
            let error = SessionsListInput::parse(Some(&json!({ "limit": value })))
                .expect_err("out of schema");
            assert_eq!(error.violations[0].problem, problem, "for {value}");
        }
    }

    #[test]
    fn arguments_that_are_not_an_object_are_rejected() {
        let error = SessionsListInput::parse(Some(&json!([1, 2]))).expect_err("not an object");
        assert_eq!(error.violations[0].property, "arguments");
        assert_eq!(
            error.violations[0].problem,
            "Expected object, received array",
        );
    }

    #[test]
    fn the_default_limit_is_the_catalogued_twenty() {
        assert_eq!(SessionsListInput::default().effective_limit(), 20);
        assert_eq!(
            SessionsListInput {
                limit: Some(5),
                ..SessionsListInput::default()
            }
            .effective_limit(),
            5,
        );
    }

    #[test]
    fn an_all_whitespace_query_is_no_filter() {
        let input = SessionsListInput {
            query: Some("   ".to_owned()),
            limit: None,
        };
        assert_eq!(input.needle(), None);
        let input = SessionsListInput {
            query: Some("  Project  ".to_owned()),
            limit: None,
        };
        assert_eq!(input.needle().as_deref(), Some("project"));
    }

    #[test]
    fn a_lookup_naming_nothing_matches_nothing() {
        let record = DelegationRecord::new("d-1", "s-1", "t", "/tmp");
        assert!(!DelegationLookupInput::default().matches(&record));
        assert!(
            !DelegationLookupInput {
                delegation_id: Some("   ".to_owned()),
                session_id: Some(String::new()),
            }
            .matches(&record)
        );
        assert!(
            DelegationLookupInput {
                delegation_id: Some(" d-1 ".to_owned()),
                session_id: None,
            }
            .matches(&record)
        );
        assert!(
            DelegationLookupInput {
                delegation_id: None,
                session_id: Some("s-1".to_owned()),
            }
            .matches(&record)
        );
    }

    #[test]
    fn not_found_serializes_bare() {
        assert_eq!(
            serde_json::to_string(&SessionStatusResult::NotFound).expect("json"),
            r#"{"status":"not_found"}"#,
        );
    }

    #[test]
    fn result_appears_only_when_completed_and_error_only_when_failed() {
        let record = DelegationRecord::new("d-1", "s-1", "Title", "/srv/project");
        let running = SessionStatusResult::known(record.clone(), DelegationOutcome::Running);
        let json = serde_json::to_value(&running).expect("json");
        assert!(json.get("result").is_none());
        assert!(json.get("error").is_none());
        assert_eq!(
            json.as_object().expect("object").keys().collect::<Vec<_>>(),
            [
                "status",
                "delegation_id",
                "session_id",
                "title",
                "directory"
            ],
        );

        let completed =
            SessionStatusResult::known(record.clone(), DelegationOutcome::completed("  done  "));
        let json = serde_json::to_value(&completed).expect("json");
        assert_eq!(json["result"], "done");
        assert!(json.get("error").is_none());

        let failed = SessionStatusResult::known(record, DelegationOutcome::failed(" it broke "));
        let json = serde_json::to_value(&failed).expect("json");
        assert_eq!(json["error"], "it broke");
        assert!(json.get("result").is_none());
    }

    #[test]
    fn a_completed_result_is_clipped_to_four_thousand() {
        let long = "x".repeat(5_000);
        let DelegationOutcome::Completed { result } = DelegationOutcome::completed(&long) else {
            unreachable!("completed")
        };
        assert_eq!(result.chars().count(), STATUS_RESULT_BOUND);
    }

    #[test]
    fn clipping_preserves_interior_whitespace() {
        // The difference from `bounded`, which would collapse the run.
        assert_eq!(clip("  a   b  ", 100), "a   b");
    }

    #[test]
    fn clipping_never_splits_an_astral_character() {
        let text = "\u{1f600}\u{1f600}";
        assert_eq!(clip(text, 3), "\u{1f600}");
        assert_eq!(clip(text, 4), text);
    }

    #[test]
    fn the_started_answer_puts_status_first() {
        let started = DelegationRecord::new("d-1", "s-1", "T", "/p").started();
        assert_eq!(
            serde_json::to_string(&started).expect("json"),
            r#"{"status":"started","delegation_id":"d-1","session_id":"s-1","title":"T","directory":"/p"}"#,
        );
    }

    #[test]
    fn a_session_summary_bounds_its_title_and_cleans_the_rest() {
        let summary = SessionSummary::new(
            " s-1 ",
            &format!("  a\n\nb  {}", "z".repeat(200)),
            " /srv ",
            " 2026-08-22 ",
        );
        assert_eq!(summary.session_id, "s-1");
        assert_eq!(summary.directory, "/srv");
        assert_eq!(summary.updated_at, "2026-08-22");
        assert_eq!(summary.title.chars().count(), SESSION_TITLE_BOUND);
        assert!(summary.title.starts_with("a b z"));
    }

    #[test]
    fn the_filter_haystack_is_title_space_directory() {
        let summary = SessionSummary::new("s", "Build IT", "/Srv/Project", "");
        assert_eq!(summary.filter_haystack(), "build it /srv/project");
        assert!(matches_query(&summary, Some("srv/pro")));
        assert!(matches_query(&summary, Some("it /")));
        assert!(!matches_query(&summary, Some("nope")));
        assert!(matches_query(&summary, None));
    }

    #[test]
    fn delegation_status_spells_itself_through_work_status() {
        for status in DelegationStatus::ALL {
            assert_eq!(status.as_str(), status.work_status().as_str());
        }
        assert_eq!(DelegationStatus::Running.as_str(), "running");
        assert_eq!(DelegationStatus::Cancelling.as_str(), "cancelling");
        assert!(!DelegationStatus::Cancelling.is_terminal());
        assert!(DelegationStatus::Cancelled.is_terminal());
    }

    #[test]
    fn whitespace_detection_is_ecmascripts() {
        assert!(contains_js_whitespace("a\u{feff}b"));
        assert!(!contains_js_whitespace("ab"));
    }
}
