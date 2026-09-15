//! The adapter-normalised event vocabulary — VIA's public progress surface.
//!
//! Upstream's `activityFromUpdate`
//! (`server/src/agent/acp-backend-session-utils.mjs:91-140`) is the whole of
//! it: standard `session/update` notifications in, *generic activity* out.
//! Upstream's own architecture document says what that means
//! (`docs/architecture.md` §6 there, §6 here):
//!
//! > tool name, bounded user-safe detail, and running/completed state;
//! > text/reasoning activity represented only as "organizing result".
//! > […] Session IDs, subagent IDs, raw permission payloads, and raw reasoning
//! > are not shown.
//!
//! `docs/reference/contracts.json` (`json-field` / *backend.activity event*)
//! catalogues the three payload shapes exactly, and adds: *"This IS the public
//! progress surface."*
//!
//! # This module is a boundary, not a mapper
//!
//! Three things make the guarantee hold rather than merely be documented:
//!
//! 1. **The input type has nowhere to put a secret.** [`RawSessionUpdate`] has
//!    no session id, no sub-agent id, no permission payload and no message or
//!    reasoning text. An adapter that wanted to forward one would have to
//!    change this type, which is a reviewable diff.
//! 2. **The output types cannot be built by hand.** [`PlanActivity`] and
//!    [`ToolActivity`] have private fields and bounding constructors, so there
//!    is no path to an unbounded string on this surface — not from a driver,
//!    not from a test double.
//! 3. **Everything else projects to nothing.** `agent_thought_chunk`,
//!    permission traffic, mode changes and every future update kind return
//!    [`None`]. The default is silence, so a new upstream update kind cannot
//!    start leaking by simply existing.
//!
//! `tests/event_boundary.rs` drives a hostile update through all three.

use std::fmt;

use indexmap::IndexMap;
use serde::Serialize;
use serde::ser::SerializeMap;
use serde_json::Value;

use crate::text::{DEFAULT_BOUND, bounded, clean};

/// `sessionUpdate: 'plan'`.
pub const UPDATE_PLAN: &str = "plan";
/// `sessionUpdate: 'tool_call'` — a tool call announced.
pub const UPDATE_TOOL_CALL: &str = "tool_call";
/// `sessionUpdate: 'tool_call_update'` — a tool call progressing.
pub const UPDATE_TOOL_CALL_UPDATE: &str = "tool_call_update";
/// `sessionUpdate: 'agent_message_chunk'` — the only textual update that
/// projects to anything, and it projects to [`SessionEvent::Text`] with the
/// text discarded.
pub const UPDATE_AGENT_MESSAGE_CHUNK: &str = "agent_message_chunk";

/// The fixed `id` every plan activity carries.
///
/// `server/src/agent/acp-backend-session-utils.mjs:96`. It is a constant, not a
/// session id: TaskManager dedupes activity by `id`, so one plan id per session
/// is what keeps a replanning agent from filling the ring.
pub const PLAN_ACTIVITY_ID: &str = "acp-plan";

/// The `tool` field's fallback when a tool call carries neither name nor title.
///
/// `server/src/agent/acp-backend-session-utils.mjs:126`.
pub const DEFAULT_TOOL_NAME: &str = "tool";

/// Bound on the `tool` field. `acp-backend-session-utils.mjs:126`.
pub const TOOL_NAME_BOUND: usize = 100;
/// Bound on the `label` field. `acp-backend-session-utils.mjs:127-132`.
pub const TOOL_LABEL_BOUND: usize = 160;
/// Bound on every `detail` field, and [`crate::text::DEFAULT_BOUND`].
pub const DETAIL_BOUND: usize = DEFAULT_BOUND;

/// The running/completed state an activity reports.
///
/// Upstream passes `merged.status || 'running'` straight through, so the
/// vocabulary is ACP's `ToolCallStatus` plus the `'running'` default.
///
/// # The one deviation
///
/// An **unrecognised** status is treated as [`Self::Running`] here, where
/// upstream forwards the string verbatim. The status is model-visible (it is
/// interpolated into the progress prompt and into `session_status`), so an
/// unbounded, attacker-chosen string on a typed state field is precisely what
/// this boundary exists to stop. Nothing downstream changes: an unrecognised
/// status is not `completed` or `failed` upstream either, so the tracker keeps
/// the entry exactly as it does there. Recorded in
/// `docs/deviations/phase-2.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActivityStatus {
    /// The tool call is announced but not started.
    Pending,
    /// The tool call is executing.
    InProgress,
    /// The tool call finished successfully.
    Completed,
    /// The tool call failed.
    Failed,
    /// Upstream's default when the update carried no usable status.
    #[default]
    Running,
}

impl ActivityStatus {
    /// The wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Running => "running",
        }
    }

    /// Parse a wire spelling, or `None` for anything else.
    #[must_use]
    pub fn from_wire(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "in_progress" => Some(Self::InProgress),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "running" => Some(Self::Running),
            _ => None,
        }
    }

    /// Whether an activity in this state is finished.
    ///
    /// `['completed', 'failed'].includes(merged.status)`
    /// (`acp-backend-session-utils.mjs:136`) — the test that decides whether
    /// the tracker forgets a tool call.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }
}

impl fmt::Display for ActivityStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for ActivityStatus {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// What kind of work a tool call is doing, for the UI's stable phrasing.
///
/// `categoryForTool`, `server/src/agent/acp-backend-session-utils.mjs:80-89`.
/// The five values map to "searching", "reading", "generating an image" and so
/// on; the point of collapsing a hundred tool names into five is that the
/// phrase shown to the user does not depend on what a backend called its tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToolCategory {
    /// Image generation or drawing.
    Image,
    /// Search, fetch or browsing.
    Search,
    /// Reading or locating files.
    Read,
    /// Writing, editing or patching.
    Write,
    /// Anything else — upstream's fallback.
    #[default]
    Run,
}

impl ToolCategory {
    /// The wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Search => "search",
            Self::Read => "read",
            Self::Write => "write",
            Self::Run => "run",
        }
    }

    /// Classify a tool call, exactly as `categoryForTool` does.
    ///
    /// The hint is `[name, title, JSON.stringify(rawInput || {})].join(' ')`
    /// lower-cased, and the first matching group wins in the order
    /// image → search → read → write → run. Both the English and the Chinese
    /// needles are upstream's; they are matched against a backend's own tool
    /// names, which are not VIA's to rename.
    #[must_use]
    pub fn classify(name: &str, title: &str, raw_input: Option<&Value>) -> Self {
        const IMAGE: [&str; 6] = ["image", "draw", "canvas", "图片", "图像", "绘图"];
        const SEARCH: [&str; 6] = ["search", "web", "fetch", "browser", "搜索", "查询"];
        const READ: [&str; 6] = ["read", "glob", "grep", "list", "读取", "查找"];
        const WRITE: [&str; 5] = ["write", "edit", "patch", "写入", "修改"];

        let serialized = raw_input.map_or_else(|| "{}".to_owned(), json_stringify);
        let hint = format!("{name} {title} {serialized}").to_lowercase();
        for (needles, category) in [
            (IMAGE.as_slice(), Self::Image),
            (SEARCH.as_slice(), Self::Search),
            (READ.as_slice(), Self::Read),
            (WRITE.as_slice(), Self::Write),
        ] {
            if needles.iter().any(|needle| hint.contains(needle)) {
                return category;
            }
        }
        Self::Run
    }
}

impl fmt::Display for ToolCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for ToolCategory {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// `JSON.stringify(value)`.
fn json_stringify(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "{}".to_owned())
}

/// One entry of an agent's plan.
///
/// ACP's `PlanEntry`, as upstream reads it
/// (`acp-backend-session-utils.mjs:92-95`): only `status` and `content` are
/// consulted, and only `content` reaches the user — bounded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanEntry {
    /// The entry's text.
    pub content: String,
    /// Where the entry is.
    pub status: PlanEntryStatus,
}

/// The three states a plan entry can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlanEntryStatus {
    /// Not started.
    Pending,
    /// In flight — at most one entry is, and it is the one the user hears
    /// about.
    InProgress,
    /// Done; counted into `completed`.
    Completed,
}

/// A `session/update` notification, reduced to the fields the projection reads.
///
/// **This type is the boundary's first line.** It carries no session id, no
/// sub-agent id, no permission payload and no message or reasoning text,
/// because the projection must never have one to forward. Adapters build it
/// from their own transport types; `via-acp` builds it from ACP's
/// `SessionNotification`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RawSessionUpdate {
    /// The `sessionUpdate` discriminant — see [`UPDATE_PLAN`] and friends.
    /// Anything not named there projects to nothing.
    pub session_update: String,
    /// `toolCallId`, the key the tracker merges on and the id the UI dedupes
    /// on. A *tool call* id: it is the agent's own, is scoped to one session,
    /// and names nothing outside it.
    pub tool_call_id: Option<String>,
    /// The tool's name, when the backend sends one.
    pub name: Option<String>,
    /// The tool call's human title.
    pub title: Option<String>,
    /// The raw status string, as received. Parsed by
    /// [`ActivityStatus::from_wire`].
    pub status: Option<String>,
    /// The tool's arguments, exactly as the backend sent them. Only
    /// `description`, `query`, `path` and `command` are ever read out of it,
    /// and each is bounded on the way.
    pub raw_input: Option<Value>,
    /// The plan, for `sessionUpdate: 'plan'`.
    pub entries: Vec<PlanEntry>,
}

impl RawSessionUpdate {
    /// A tool-call update, the shape adapters build most often.
    #[must_use]
    pub fn tool_call(id: &str) -> Self {
        Self {
            session_update: UPDATE_TOOL_CALL.to_owned(),
            tool_call_id: Some(id.to_owned()),
            ..Self::default()
        }
    }

    /// A plan update.
    #[must_use]
    pub fn plan(entries: Vec<PlanEntry>) -> Self {
        Self {
            session_update: UPDATE_PLAN.to_owned(),
            entries,
            ..Self::default()
        }
    }

    /// The `{...known, ...update}` merge upstream performs before projecting a
    /// tool call (`acp-backend-session-utils.mjs:117-120`): a field the update
    /// carries wins, a field it omits is inherited.
    fn merged_over(&self, previous: &Self) -> Self {
        Self {
            session_update: self.session_update.clone(),
            tool_call_id: self
                .tool_call_id
                .clone()
                .or_else(|| previous.tool_call_id.clone()),
            name: self.name.clone().or_else(|| previous.name.clone()),
            title: self.title.clone().or_else(|| previous.title.clone()),
            status: self.status.clone().or_else(|| previous.status.clone()),
            raw_input: self
                .raw_input
                .clone()
                .or_else(|| previous.raw_input.clone()),
            entries: if self.entries.is_empty() {
                previous.entries.clone()
            } else {
                self.entries.clone()
            },
        }
    }
}

/// A plan activity: how far through its plan the agent is.
///
/// Fields are private; [`PlanActivity::from_entries`] is the only way to build
/// one, and it bounds the only string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanActivity {
    status: ActivityStatus,
    detail: String,
    completed: usize,
    total: usize,
}

impl PlanActivity {
    /// Project a plan, exactly as `acp-backend-session-utils.mjs:92-103` does.
    ///
    /// `status` is [`ActivityStatus::Running`] while any entry is in progress
    /// or pending and [`ActivityStatus::Completed`] otherwise — which makes an
    /// *empty* plan `completed`, as upstream's does.
    #[must_use]
    pub fn from_entries(entries: &[PlanEntry]) -> Self {
        let completed = entries
            .iter()
            .filter(|entry| entry.status == PlanEntryStatus::Completed)
            .count();
        let current = entries
            .iter()
            .find(|entry| entry.status == PlanEntryStatus::InProgress)
            .or_else(|| {
                entries
                    .iter()
                    .find(|entry| entry.status == PlanEntryStatus::Pending)
            });
        Self {
            status: current.map_or(ActivityStatus::Completed, |_| ActivityStatus::Running),
            detail: bounded(
                current.map_or("", |entry| entry.content.as_str()),
                DETAIL_BOUND,
            ),
            completed,
            total: entries.len(),
        }
    }

    /// Whether the plan still has work in it.
    #[must_use]
    pub const fn status(&self) -> ActivityStatus {
        self.status
    }

    /// The current entry's text, bounded to [`DETAIL_BOUND`].
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// How many entries are done.
    #[must_use]
    pub const fn completed(&self) -> usize {
        self.completed
    }

    /// How many entries there are.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.total
    }
}

/// A tool activity: what the agent is doing right now, in generic terms.
///
/// Fields are private; [`ToolActivity::project`] is the only way to build one,
/// and every string it produces is bounded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolActivity {
    id: Option<String>,
    tool: String,
    label: String,
    status: ActivityStatus,
    category: ToolCategory,
    detail: String,
}

impl ToolActivity {
    /// Project a merged tool-call update
    /// (`acp-backend-session-utils.mjs:121-135`).
    ///
    /// - `tool` is `name || title`, bounded to [`TOOL_NAME_BOUND`], falling
    ///   back to [`DEFAULT_TOOL_NAME`] when the result is empty;
    /// - `label` is `rawInput.description || title`, bounded to
    ///   [`TOOL_LABEL_BOUND`];
    /// - `detail` is the first of `rawInput.description`, `.query`, `.path`,
    ///   `.command`, bounded to [`DETAIL_BOUND`];
    /// - `category` is derived, never supplied.
    #[must_use]
    fn project(update: &RawSessionUpdate) -> Self {
        let id = clean(update.tool_call_id.as_deref().unwrap_or_default());
        let name = update.name.as_deref().unwrap_or_default();
        let title = update.title.as_deref().unwrap_or_default();
        let raw_input = update.raw_input.as_ref();

        let tool = bounded(js_or(&[name, title]), TOOL_NAME_BOUND);
        let description = raw_input.and_then(|value| js_field(value, "description"));
        let label = bounded(
            js_or(&[description.as_deref().unwrap_or_default(), title]),
            TOOL_LABEL_BOUND,
        );
        let detail = bounded(
            js_or(&[
                description.as_deref().unwrap_or_default(),
                raw_input
                    .and_then(|value| js_field(value, "query"))
                    .as_deref()
                    .unwrap_or_default(),
                raw_input
                    .and_then(|value| js_field(value, "path"))
                    .as_deref()
                    .unwrap_or_default(),
                raw_input
                    .and_then(|value| js_field(value, "command"))
                    .as_deref()
                    .unwrap_or_default(),
            ]),
            DETAIL_BOUND,
        );

        Self {
            id: (!id.is_empty()).then(|| id.to_owned()),
            tool: if tool.is_empty() {
                DEFAULT_TOOL_NAME.to_owned()
            } else {
                tool
            },
            label,
            status: update
                .status
                .as_deref()
                .and_then(ActivityStatus::from_wire)
                .unwrap_or_default(),
            category: ToolCategory::classify(name, title, raw_input),
            detail,
        }
    }

    /// The tool call id, when the update carried one.
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// The tool's name, bounded to [`TOOL_NAME_BOUND`], never empty.
    #[must_use]
    pub fn tool(&self) -> &str {
        &self.tool
    }

    /// The tool call's human label, bounded to [`TOOL_LABEL_BOUND`].
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Where the tool call is.
    #[must_use]
    pub const fn status(&self) -> ActivityStatus {
        self.status
    }

    /// Which of the five generic categories this is.
    #[must_use]
    pub const fn category(&self) -> ToolCategory {
        self.category
    }

    /// The user-safe detail, bounded to [`DETAIL_BOUND`].
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// One normalised activity event.
///
/// Serializes to exactly the `activity` payload
/// `docs/reference/contracts.json` catalogues under *backend.activity event*,
/// field for field and in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    /// The agent's plan moved.
    Plan(PlanActivity),
    /// A tool call was announced or progressed.
    Tool(ToolActivity),
    /// The agent produced text.
    ///
    /// **The text is not here, and there is nowhere to put it.** Upstream
    /// collapses every textual update to `{id: null, kind: 'text', status:
    /// 'running'}` and the UI renders that as "organizing result". Reasoning
    /// (`agent_thought_chunk`) does not even reach this variant — it projects
    /// to [`None`].
    Text,
}

impl SessionEvent {
    /// The `kind` this event serializes as.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Plan(_) => "plan",
            Self::Tool(_) => "tool",
            Self::Text => "text",
        }
    }

    /// The `id` the UI dedupes on: `acp-plan`, the tool call id, or none.
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::Plan(_) => Some(PLAN_ACTIVITY_ID),
            Self::Tool(activity) => activity.id(),
            Self::Text => None,
        }
    }

    /// The state this event reports.
    #[must_use]
    pub const fn status(&self) -> ActivityStatus {
        match self {
            Self::Plan(activity) => activity.status,
            Self::Tool(activity) => activity.status,
            Self::Text => ActivityStatus::Running,
        }
    }
}

impl Serialize for SessionEvent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Plan(activity) => {
                let mut map = serializer.serialize_map(Some(6))?;
                map.serialize_entry("id", PLAN_ACTIVITY_ID)?;
                map.serialize_entry("kind", "plan")?;
                map.serialize_entry("status", &activity.status)?;
                map.serialize_entry("detail", &activity.detail)?;
                map.serialize_entry("completed", &activity.completed)?;
                map.serialize_entry("total", &activity.total)?;
                map.end()
            }
            Self::Tool(activity) => {
                let mut map = serializer.serialize_map(Some(7))?;
                map.serialize_entry("id", &activity.id)?;
                map.serialize_entry("kind", "tool")?;
                map.serialize_entry("tool", &activity.tool)?;
                map.serialize_entry("label", &activity.label)?;
                map.serialize_entry("status", &activity.status)?;
                map.serialize_entry("category", &activity.category)?;
                map.serialize_entry("detail", &activity.detail)?;
                map.end()
            }
            Self::Text => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("id", &None::<&str>)?;
                map.serialize_entry("kind", "text")?;
                map.serialize_entry("status", &ActivityStatus::Running)?;
                map.end()
            }
        }
    }
}

/// The per-session memory `activityFromUpdate`'s `known` map provides.
///
/// A `tool_call_update` usually carries only what changed, so the projection
/// needs the announcement it is updating. The tracker holds the merged update
/// per tool call id and forgets it the moment the call reaches a terminal
/// status.
///
/// # Lifetime and size
///
/// Upstream's map is unbounded and so is this, because the two must agree on
/// which updates project to what. A tool call that never terminates therefore
/// occupies an entry for as long as the session lives — the session's owner
/// (`via-acp`) is where that lifetime is decided, and [`Self::clear`] is how it
/// is ended.
#[derive(Debug, Clone, Default)]
pub struct ActivityTracker {
    seen: IndexMap<String, RawSessionUpdate>,
}

impl ActivityTracker {
    /// An empty tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Project one `session/update` notification into an activity event.
    ///
    /// `activityFromUpdate(update, known)`,
    /// `server/src/agent/acp-backend-session-utils.mjs:91-140`. Returns `None`
    /// for every update kind that is not a plan, a tool call or an agent
    /// message — which is upstream's `return null`, and is what keeps
    /// reasoning, permission traffic and mode changes off this surface.
    pub fn project(&mut self, update: &RawSessionUpdate) -> Option<SessionEvent> {
        if update.session_update == UPDATE_PLAN {
            return Some(SessionEvent::Plan(PlanActivity::from_entries(
                &update.entries,
            )));
        }
        if update.session_update != UPDATE_TOOL_CALL
            && update.session_update != UPDATE_TOOL_CALL_UPDATE
        {
            return (update.session_update == UPDATE_AGENT_MESSAGE_CHUNK)
                .then_some(SessionEvent::Text);
        }

        let id = clean(update.tool_call_id.as_deref().unwrap_or_default()).to_owned();
        let merged = self
            .seen
            .get(&id)
            .map_or_else(|| update.clone(), |previous| update.merged_over(previous));
        let activity = ToolActivity::project(&merged);
        if !id.is_empty() {
            if activity.status.is_terminal() {
                self.seen.shift_remove(&id);
            } else {
                self.seen.insert(id, merged);
            }
        }
        Some(SessionEvent::Tool(activity))
    }

    /// How many in-flight tool calls the tracker is remembering.
    #[must_use]
    pub fn len(&self) -> usize {
        self.seen.len()
    }

    /// Whether the tracker is remembering nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }

    /// Forget every in-flight tool call. Call it when the session ends.
    pub fn clear(&mut self) {
        self.seen.clear();
    }
}

/// JavaScript `a || b || …` over strings: the first non-empty one, or `""`.
fn js_or<'a>(candidates: &[&'a str]) -> &'a str {
    candidates
        .iter()
        .copied()
        .find(|candidate| !candidate.is_empty())
        .unwrap_or_default()
}

/// `value?.<field>` coerced the way `bounded()`'s `String(value || '')` would.
///
/// Returns `None` when the field is absent or *falsy* — JavaScript's `||`
/// chain skips `null`, `false`, `0`, `NaN` and `""` alike, so a
/// `{"path": ""}` falls through to the next candidate exactly as it does
/// upstream. A truthy non-string is coerced: `5` renders as `"5"`, `["a","b"]`
/// as `"a,b"`, and an object as `"[object Object]"`, which is what
/// `String(...)` produces and what upstream would put on the wire.
fn js_field(value: &Value, field: &str) -> Option<String> {
    let field = value.get(field)?;
    if !js_truthy(field) {
        return None;
    }
    Some(js_display(field))
}

/// JavaScript truthiness for a JSON value.
fn js_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().is_some_and(|value| value != 0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// `String(value)` for a JSON value.
fn js_display(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.as_f64().map_or_else(
            || number.to_string(),
            |value| {
                if value.fract() == 0.0 && value.abs() < 1e21 {
                    format!("{value:.0}")
                } else {
                    value.to_string()
                }
            },
        ),
        Value::String(text) => text.clone(),
        // `String([a, b])` is `Array.prototype.join(',')`, applied recursively,
        // with null and undefined rendering as empty.
        Value::Array(items) => items
            .iter()
            .map(|item| {
                if item.is_null() {
                    String::new()
                } else {
                    js_display(item)
                }
            })
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".to_owned(),
    }
}
