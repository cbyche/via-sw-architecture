//! The five coordination tools, and the one array every consumer reads them
//! from.
//!
//! Ported from upstream `server/src/agent/acp-session-tools.mjs:9-131`, with
//! the identity substitution `docs/rebrand.md` mandates applied and nothing
//! else changed. Every string in this module is **model-visible**: the names
//! the backend agent calls, the titles and descriptions it reads to decide
//! *whether* to call, and the JSON Schemas it fills in.
//!
//! # Why one array
//!
//! `docs/architecture.md` §13 flags this as the highest-risk rename in the
//! port:
//!
//! > the five MCP tool names (the permission broker auto-approves them via
//! > *three* name-shape matches — exact, `endsWith("__<name>")`,
//! > `startsWith("<name> (")` — so a partial rename wedges the coordinator)
//!
//! Upstream has the same hazard and answers it the same way: one exported
//! `const` array (`ACP_SESSION_TOOL_NAMES`) that both the registration and
//! `permission-broker.mjs:40-43` read. [`SESSION_TOOL_NAMES`] is that array.
//! [`SessionTool::name`] indexes into it, [`tool_definitions`] is built from
//! [`SessionTool::ALL`], and [`match_session_tool`] iterates it — so there is
//! no second list to forget. A rename that edits one place and not the other
//! does not compile, because there is only one place.
//!
//! # The delegation contract
//!
//! [`SessionTool::SessionStart`] and [`SessionTool::SessionSend`] return an
//! **opaque delegation id** and nothing else of substance. Once either has
//! answered `status=started`, the backend agent's turn is over: the Gateway
//! owns waiting, cancellation, permission routing and result correlation, and
//! the agent must neither poll, nor redo the work, nor answer from its own
//! context. [`DEFAULT_SESSION_INSTRUCTIONS`] is the sentence that tells it so,
//! and it lives beside the names it interpolates for the same reason the
//! allow-list does.
//!
//! [`SessionTool::SessionStatus`] is **observational only**. When the query
//! fails, the backend reports the failure; it does not go and look at the
//! target directory with its own file tools. That rule is stated in the
//! coordinator prompt (`docs/reference/contracts.json`,
//! *tool-description / session_status*) and is the reason the tool is declared
//! `readOnlyHint`.

use serde_json::{Value, json};

/// The MCP server name backends see, and the namespace they prefix tool names
/// with.
///
/// **External contract.** Upstream `ACP_SESSION_TOOL_SERVER`
/// (`server/src/agent/acp-session-tools.mjs:9`), renamed by
/// `docs/rebrand.md`. It is used three times: as the `name` on the descriptor
/// handed to ACP, as the MCP implementation name in the `initialize` reply,
/// and — because backends namespace tool names with it — as the reason
/// [`SessionToolMatch::NamespacePrefixed`] exists.
pub const SESSION_TOOL_SERVER: &str = "via";

/// The MCP implementation version reported by `initialize`.
///
/// **External contract.** Upstream `acp-session-tools.mjs:231-233`:
/// `new McpServer({ name: ACP_SESSION_TOOL_SERVER, version: '1.0.0' })`. It is
/// the *server's* version, not VIA's, and it does not move with the crate.
pub const SESSION_TOOL_SERVER_VERSION: &str = "1.0.0";

/// The five tool names, in upstream's declaration order.
///
/// **External contract, and the single source of truth.** Upstream
/// `ACP_SESSION_TOOL_NAMES` (`server/src/agent/acp-session-tools.mjs:10-16`),
/// renamed per `docs/rebrand.md` rows 115-120.
///
/// Read by:
///
/// * [`SessionTool::name`], hence by every tool definition served over MCP;
/// * [`match_session_tool`], hence by the permission broker's auto-approve
///   carve-out;
/// * this crate's `tests/contracts.rs`, against the catalogue.
///
/// Nothing else may spell one of these out.
pub const SESSION_TOOL_NAMES: [&str; SessionTool::COUNT] = [
    "via_sessions_list",
    "via_session_start",
    "via_session_send",
    "via_session_status",
    "via_session_cancel",
];

/// The default session instructions for a harness that reaches Layer 3 through
/// these tools.
///
/// **External contract** (`prompt-text` / *default sessionInstructions*),
/// upstream `server/src/agent/acp-backend-adapter.mjs:941-947`, joined with
/// single spaces and rebranded. It is a `const` here rather than in the
/// adapter because it names two of [`SESSION_TOOL_NAMES`]' suffixes: keeping
/// the sentence beside the array is what stops a rename from leaving the model
/// instructed to call a tool that no longer exists.
///
/// Three profiles override it with their own text; those live in
/// `via-backends` because they name a backend.
pub const DEFAULT_SESSION_INSTRUCTIONS: &str = "The via MCP tools are the only interface for opening, continuing, querying, and cancelling third-layer project Sessions. session_start and session_send are asynchronous. After either returns status=started, return the delegated response required by the request envelope and stop this turn. Never poll it in the same turn.";

/// One of the five coordination tools.
///
/// The discriminants are upstream's declaration order, which is also
/// [`SESSION_TOOL_NAMES`]' order and the order `tools/list` reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SessionTool {
    /// `via_sessions_list` — find an existing project Session to continue.
    SessionsList,
    /// `via_session_start` — open a new project Session, asynchronously.
    SessionStart,
    /// `via_session_send` — continue an existing project Session,
    /// asynchronously.
    SessionSend,
    /// `via_session_status` — read a delegation's status. Observational only.
    SessionStatus,
    /// `via_session_cancel` — cancel a delegated project Session.
    SessionCancel,
}

impl SessionTool {
    /// How many tools there are. Upstream serves exactly five.
    pub const COUNT: usize = 5;

    /// Every tool, in upstream's declaration order.
    pub const ALL: [SessionTool; Self::COUNT] = [
        Self::SessionsList,
        Self::SessionStart,
        Self::SessionSend,
        Self::SessionStatus,
        Self::SessionCancel,
    ];

    /// This tool's position in [`SESSION_TOOL_NAMES`].
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// The model-visible tool name.
    ///
    /// Indexes [`SESSION_TOOL_NAMES`]; it is not a second copy of the literal.
    #[must_use]
    pub const fn name(self) -> &'static str {
        SESSION_TOOL_NAMES[self.index()]
    }

    /// The model-visible title.
    ///
    /// **External contract** — `acp-session-tools.mjs:36,58,76,94,116`. The
    /// titles carry no product identity and are reproduced unchanged.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::SessionsList => "List Agent Sessions",
            Self::SessionStart => "Start Agent Session",
            Self::SessionSend => "Continue Agent Session",
            Self::SessionStatus => "Query Agent Session",
            Self::SessionCancel => "Cancel Agent Session",
        }
    }

    /// The model-visible description.
    ///
    /// **External contract** — `acp-session-tools.mjs:37,59,77,95,117`. These
    /// sentences are what steer the coordinator: the first decides whether a
    /// prior Session is found instead of silently replaced, and the second
    /// encodes two hard product invariants — the Gateway owns directory
    /// resolution, and request envelopes never reach project history.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::SessionsList => {
                "List existing project Sessions. Use this to find the exact Session when the user asks to continue previous work."
            }
            Self::SessionStart => {
                "Start a new Session asynchronously in the same project as the coordinator Session. Call it directly without creating or choosing a directory. Send only the natural task text."
            }
            Self::SessionSend => {
                "Continue an existing project Session asynchronously. Use the exact session_id returned by the Session list; the Gateway restores its original project directory."
            }
            Self::SessionStatus => {
                "Read the current status and latest known result of a delegated project Session."
            }
            Self::SessionCancel => "Cancel a delegated project Session.",
        }
    }

    /// Whether this tool only observes.
    ///
    /// The two `readOnlyHint: true` tools. Some backends use the hint to skip
    /// a permission prompt, so it is a behavioural flag rather than a label.
    #[must_use]
    pub const fn is_read_only(self) -> bool {
        matches!(self, Self::SessionsList | Self::SessionStatus)
    }

    /// The model-visible MCP annotations, or `None` where upstream declares no
    /// annotations block at all.
    ///
    /// **External contract** — `acp-session-tools.mjs:42-45,100-103`. Only the
    /// two read-only tools carry one, and both carry the same pair:
    /// `{ readOnlyHint: true, openWorldHint: false }`. `openWorldHint: false`
    /// is the truthful claim that these tools reach VIA's own Gateway and
    /// nothing beyond it.
    #[must_use]
    pub fn annotations(self) -> Option<Value> {
        self.is_read_only()
            .then(|| json!({ "readOnlyHint": true, "openWorldHint": false }))
    }

    /// The model-visible JSON Schema for this tool's arguments.
    ///
    /// **External contract** — `acp-session-tools.mjs:38-41,60-63,78-81,
    /// 96-99,118-121`, catalogued under `json-field / <tool>.inputSchema`.
    /// Property order is the order upstream declared its zod fields in and is
    /// preserved by the workspace's `serde_json/preserve_order`; `required` is
    /// present even when empty, as the catalogue spells it.
    ///
    /// Note the whole surface is `snake_case` while ACP's own surface is
    /// `camelCase`. That is upstream's choice and it is load-bearing: the
    /// model is told to pass back the `session_id` it read out of a result.
    #[must_use]
    pub fn input_schema(self) -> Value {
        match self {
            Self::SessionsList => json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "limit": { "type": "integer", "minimum": 1, "maximum": 100 },
                },
                "required": [],
            }),
            Self::SessionStart => json!({
                "type": "object",
                "properties": {
                    "prompt": { "type": "string", "minLength": 1 },
                    "title": { "type": "string" },
                },
                "required": ["prompt"],
            }),
            Self::SessionSend => json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "minLength": 1 },
                    "prompt": { "type": "string", "minLength": 1 },
                },
                "required": ["session_id", "prompt"],
            }),
            Self::SessionStatus | Self::SessionCancel => json!({
                "type": "object",
                "properties": {
                    "delegation_id": { "type": "string" },
                    "session_id": { "type": "string" },
                },
                "required": [],
            }),
        }
    }

    /// This tool as one entry of an MCP `tools/list` reply.
    ///
    /// Field order is `name`, `title`, `description`, `inputSchema`, and
    /// `annotations` last and only where upstream declares one — an absent
    /// annotations block and an empty one are different things to a backend
    /// that reads `readOnlyHint`.
    #[must_use]
    pub fn definition(self) -> Value {
        let mut entry = json!({
            "name": self.name(),
            "title": self.title(),
            "description": self.description(),
            "inputSchema": self.input_schema(),
        });
        if let (Some(annotations), Some(object)) = (self.annotations(), entry.as_object_mut()) {
            object.insert("annotations".to_owned(), annotations);
        }
        entry
    }

    /// The tool with this exact name.
    ///
    /// Exact match only. The two lenient shapes a *backend* may present a name
    /// in belong to [`match_session_tool`], which is the permission broker's
    /// question, not the dispatcher's.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tool| tool.name() == name)
    }
}

impl core::fmt::Display for SessionTool {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}

/// Every tool definition, in `tools/list` order.
#[must_use]
pub fn tool_definitions() -> Vec<Value> {
    SessionTool::ALL
        .into_iter()
        .map(SessionTool::definition)
        .collect()
}

/// The full `tools/list` result payload.
#[must_use]
pub fn tools_list_result() -> Value {
    json!({ "tools": tool_definitions() })
}

/// Which of the three name shapes matched.
///
/// **External contract** — upstream `PermissionBroker.request`,
/// `server/src/agent/permission-broker.mjs:40-43`:
///
/// ```js
/// const internal = ACP_SESSION_TOOL_NAMES.some(toolName => (
///   name === toolName
///   || name.endsWith(`__${toolName}`)
///   || name.startsWith(`${toolName} (`)
/// ))
/// ```
///
/// All three exist because a backend does not necessarily present the tool
/// name it was given. Some prefix it with the MCP server namespace
/// (`mcp__via__via_session_start`); some render a human title with the
/// arguments appended (`via_session_start (build the thing)`). A rename that
/// updates the registration but not this matcher turns every internal
/// coordination call into a permission prompt nobody is there to answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SessionToolMatch {
    /// `name === toolName`.
    Exact,
    /// `name.endsWith("__" + toolName)` — the backend namespaced it.
    NamespacePrefixed,
    /// `name.startsWith(toolName + " (")` — the backend rendered a title.
    TitlePrefixed,
}

impl SessionToolMatch {
    /// Every shape, in the order upstream tests them.
    pub const ALL: [SessionToolMatch; 3] =
        [Self::Exact, Self::NamespacePrefixed, Self::TitlePrefixed];

    /// A stable machine-readable name, for logs and tests.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::NamespacePrefixed => "namespace_prefixed",
            Self::TitlePrefixed => "title_prefixed",
        }
    }

    /// Whether `name` presents `tool` in this shape.
    ///
    /// Reproduced without narrowing. `endsWith("__" + tool)` is true for a
    /// name that is *exactly* `__<tool>`, with nothing in front of the
    /// separator, and this says so too. Adding a "there must be a real
    /// namespace" guard would be strictly stricter than upstream, and being
    /// stricter here is the failure mode `docs/architecture.md` §13 names:
    /// a coordination call that stops being auto-approved becomes a permission
    /// prompt nobody answers. The looseness costs nothing — a backend that
    /// wanted the carve-out could simply present the exact name.
    #[must_use]
    pub fn matches(self, name: &str, tool: SessionTool) -> bool {
        let tool = tool.name();
        match self {
            Self::Exact => name == tool,
            Self::NamespacePrefixed => name.ends_with(&format!("__{tool}")),
            Self::TitlePrefixed => name.starts_with(&format!("{tool} (")),
        }
    }
}

/// Which coordination tool a backend-presented tool name refers to, and how it
/// was spelled — or `None` when it is not one of ours.
///
/// This is the permission broker's auto-approve carve-out, exported from the
/// crate that owns the names so that there is exactly one source of truth. The
/// broker itself lives with whoever owns `session/request_permission`
/// handling; it must call this rather than restate the shapes.
///
/// The name is [`clean`](via_downstream::text::clean)ed first, matching
/// upstream, which trims once at
/// `permission-broker.mjs:38` before testing anything. Trimming is idempotent,
/// so a caller that already cleaned loses nothing — and a caller that forgot
/// does not silently fall through to a prompt.
#[must_use]
pub fn match_session_tool(name: &str) -> Option<(SessionTool, SessionToolMatch)> {
    let name = via_downstream::text::clean(name);
    if name.is_empty() {
        return None;
    }
    // Upstream's iteration order: every tool, and for each the three shapes.
    // The order is not observable — at most one tool can match, because no
    // name is a `__`- or ` (`-suffixed extension of another — but reproducing
    // it keeps the two implementations comparable.
    SessionTool::ALL.into_iter().find_map(|tool| {
        SessionToolMatch::ALL
            .into_iter()
            .find(|shape| shape.matches(name, tool))
            .map(|shape| (tool, shape))
    })
}

/// Whether a backend-presented tool name is one of VIA's own coordination
/// tools, in any of the three shapes.
///
/// The predicate form of [`match_session_tool`]. This is what a permission
/// broker branches on: `true` means auto-approve without prompting anybody.
#[must_use]
pub fn is_session_tool(name: &str) -> bool {
    match_session_tool(name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_array_and_the_enum_cannot_disagree() {
        let from_enum: Vec<&str> = SessionTool::ALL
            .into_iter()
            .map(SessionTool::name)
            .collect();
        assert_eq!(from_enum, SESSION_TOOL_NAMES.to_vec());
        assert_eq!(SessionTool::ALL.len(), SESSION_TOOL_NAMES.len());
    }

    #[test]
    fn every_registered_tool_is_on_the_allow_list() {
        for definition in tool_definitions() {
            let name = definition["name"].as_str().expect("a name");
            assert!(
                is_session_tool(name),
                "{name} is served but not auto-approved"
            );
        }
    }

    #[test]
    fn every_allow_listed_name_is_registered() {
        let served: Vec<String> = tool_definitions()
            .iter()
            .map(|definition| definition["name"].as_str().unwrap_or_default().to_owned())
            .collect();
        for name in SESSION_TOOL_NAMES {
            assert!(served.iter().any(|entry| entry == name), "{name} unserved");
        }
    }

    #[test]
    fn the_three_shapes_all_match() {
        for tool in SessionTool::ALL {
            let name = tool.name();
            assert_eq!(
                match_session_tool(name),
                Some((tool, SessionToolMatch::Exact)),
            );
            assert_eq!(
                match_session_tool(&format!("mcp__{SESSION_TOOL_SERVER}__{name}")),
                Some((tool, SessionToolMatch::NamespacePrefixed)),
            );
            assert_eq!(
                match_session_tool(&format!("{name} (build the thing)")),
                Some((tool, SessionToolMatch::TitlePrefixed)),
            );
        }
    }

    #[test]
    fn a_foreign_tool_is_not_auto_approved() {
        for name in [
            "bash",
            "read_file",
            "via_session",
            "via_session_start_extra",
            "via_session_startx",
            "prefix_via_session_start",
            "_via_session_start",
            "__via_session_start_",
            "(via_session_start)",
            "via_session_start(",
            "",
            "   ",
        ] {
            assert!(!is_session_tool(name), "{name:?} matched");
        }
    }

    #[test]
    fn the_namespace_shape_is_upstreams_bare_ends_with() {
        // Upstream's `endsWith("__" + tool)` is true with nothing in front of
        // the separator, and this reproduces that rather than narrowing it.
        assert_eq!(
            match_session_tool("__via_session_start"),
            Some((
                SessionTool::SessionStart,
                SessionToolMatch::NamespacePrefixed
            )),
        );
        assert!(is_session_tool("x__via_session_start"));
        assert!(is_session_tool(
            "mcp__someone_elses_server__via_session_send"
        ));
    }

    #[test]
    fn the_name_is_cleaned_before_matching() {
        assert!(is_session_tool("  via_session_start  "));
        assert!(is_session_tool("\u{feff}via_session_start"));
    }

    #[test]
    fn read_only_tools_are_exactly_the_annotated_ones() {
        for tool in SessionTool::ALL {
            assert_eq!(tool.is_read_only(), tool.annotations().is_some());
        }
        assert_eq!(
            SessionTool::ALL
                .into_iter()
                .filter(|tool| tool.is_read_only())
                .collect::<Vec<_>>(),
            vec![SessionTool::SessionsList, SessionTool::SessionStatus],
        );
    }

    #[test]
    fn from_name_is_exact() {
        assert_eq!(
            SessionTool::from_name("via_session_start"),
            Some(SessionTool::SessionStart),
        );
        assert_eq!(SessionTool::from_name("mcp__via__via_session_start"), None);
        assert_eq!(SessionTool::from_name(" via_session_start"), None);
    }

    #[test]
    fn schema_property_order_is_the_declaration_order() {
        let order = |tool: SessionTool| -> Vec<String> {
            tool.input_schema()["properties"]
                .as_object()
                .expect("an object")
                .keys()
                .cloned()
                .collect()
        };
        assert_eq!(order(SessionTool::SessionsList), ["query", "limit"]);
        assert_eq!(order(SessionTool::SessionStart), ["prompt", "title"]);
        assert_eq!(order(SessionTool::SessionSend), ["session_id", "prompt"]);
        assert_eq!(
            order(SessionTool::SessionStatus),
            ["delegation_id", "session_id"],
        );
        assert_eq!(
            order(SessionTool::SessionCancel),
            ["delegation_id", "session_id"],
        );
    }

    #[test]
    fn a_definition_orders_its_keys_for_the_model() {
        let keys: Vec<String> = SessionTool::SessionsList
            .definition()
            .as_object()
            .expect("an object")
            .keys()
            .cloned()
            .collect();
        assert_eq!(
            keys,
            ["name", "title", "description", "inputSchema", "annotations"],
        );
        let bare: Vec<String> = SessionTool::SessionStart
            .definition()
            .as_object()
            .expect("an object")
            .keys()
            .cloned()
            .collect();
        assert_eq!(bare, ["name", "title", "description", "inputSchema"]);
    }
}
