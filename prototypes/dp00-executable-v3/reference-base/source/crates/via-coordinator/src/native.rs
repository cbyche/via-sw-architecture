//! Noticing that a backend delegated on its own.
//!
//! Most backends reach Layer 3 through VIA's five MCP tools, so the Gateway
//! *issues* the delegation and knows about it by construction. One shipped
//! backend does not: it has its own session tools, and the only evidence that a
//! Layer-3 Session was spawned is a `session/update` notification going past.
//! This module is the predicate that spots it.
//!
//! # Both halves are catalogued
//!
//! `docs/reference/contracts.json` (`state-name` / *native delegation detection
//! predicate*), `server/src/agent/acp-backend-adapter.mjs:656-694`. It fires
//! only when **all** of:
//!
//! 1. `sessionUpdate` is `tool_call` or `tool_call_update`;
//! 2. the **merged** status is `completed` — a tool call that announced itself
//!    and has not finished is not a delegation yet;
//! 3. `run.delegation` is still null — one coordinator turn delegates once;
//! 4. `/sessions_(spawn|send)/` matches the lower-cased, trimmed
//!    `name || title`;
//! 5. [`via_acp::native_tool_output`] yields a non-empty session id;
//! 6. …and a non-empty `runId`. Missing either id creates nothing.
//!
//! and the traversal in (5) is catalogued separately (`state-name` /
//! *nativeToolOutput extraction order*) and is `via-acp`'s, called through
//! rather than copied.
//!
//! # The regex is somebody else's contract
//!
//! The catalogue says so in as many words: *"This regex is matched against a
//! THIRD-PARTY tool name … it is an external contract with OpenClaw, not a VIA
//! identifier."* It is therefore **not** renamed, and it is not derived from
//! [`via_mcp_tools::SESSION_TOOL_NAMES`] either — those are the tools VIA
//! serves, and these are the tools somebody else serves.
//!
//! # Two key orders that are nearly the same, and must not be merged
//!
//! [`SESSION_ID_KEYS`] here is `childSessionKey, sessionKey, session_id,
//! sessionId` (`acp-backend-adapter.mjs:671-675`). `via-acp`'s traversal tests
//! `childSessionKey, sessionKey, sessionId, session_id`
//! (`acp-backend-session-utils.mjs:159-164`) — the last two swapped. Upstream
//! has both orders, they disagree only for output carrying *both* spellings
//! with different values, and reproducing one of them twice would silently pick
//! the other winner.

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{Map, Value};
use via_acp::session::{clean_value, native_tool_output};
use via_downstream::event::{UPDATE_TOOL_CALL, UPDATE_TOOL_CALL_UPDATE};
use via_downstream::text::{bounded, clean};

/// The pattern a delegating tool's name must match.
///
/// **External contract, not renamed** — `acp-backend-adapter.mjs:668`,
/// `/sessions_(spawn|send)/`. Unanchored: a namespaced
/// `mcp__openclaw__sessions_spawn` matches, which is the point.
pub const NATIVE_DELEGATION_TOOL_PATTERN: &str = "sessions_(spawn|send)";

/// The `rawOutput` keys a session id is read from, **in this order**.
///
/// **External contract** — `acp-backend-adapter.mjs:671-675`. See the module
/// docs for why this is not the same order `via-acp` traverses with.
pub const SESSION_ID_KEYS: [&str; 4] = ["childSessionKey", "sessionKey", "session_id", "sessionId"];

/// The `rawOutput` key the delegation id is read from.
///
/// **External contract** — `acp-backend-adapter.mjs:677`. On this path the
/// delegation id is the backend's **own** `runId`, taken verbatim — structurally
/// unlike the `<protocol>_run_<uuid>` the MCP path mints, and both must
/// round-trip (`docs/reference/contracts.json`, *delegation id format*).
pub const RUN_ID_KEY: &str = "runId";

/// The `rawInput` keys a working directory is read from, in order.
///
/// **External contract** — `acp-backend-adapter.mjs:683-686`.
pub const DIRECTORY_KEYS: [&str; 2] = ["cwd", "directory"];

/// The `rawInput` keys a title is read from, in order.
///
/// **External contract** — `acp-backend-adapter.mjs:688-691`.
pub const TITLE_KEYS: [&str; 3] = ["label", "task", "message"];

/// The bound on a detected delegation's title.
///
/// **External contract** — `acp-backend-adapter.mjs:691` (`bounded(..., 160)`).
/// The same bound [`via_work::delegation::DELEGATION_TITLE_BOUND`] publishes at,
/// applied here on the way in.
pub const TITLE_BOUND: usize = 160;

/// The status a tool call must have reached.
///
/// **External contract** — `acp-backend-adapter.mjs:666`. Spelled through
/// [`via_downstream::ActivityStatus`] so the two vocabularies cannot drift.
#[must_use]
pub fn completed_status() -> &'static str {
    via_downstream::ActivityStatus::Completed.as_str()
}

/// `Option` rather than an `unwrap`: the pattern is a literal with no runtime
/// input, so `None` is unreachable, and a crate that must not panic in a voice
/// turn should not make an unreachable case the one that ends it. `None` means
/// no native delegation is ever detected — strictly less behaviour, never wrong
/// behaviour. The test below is what actually proves it compiles. The idiom is
/// `via-acp`'s, kept identical on purpose.
static DELEGATION_TOOL: Lazy<Option<Regex>> =
    Lazy::new(|| Regex::new(NATIVE_DELEGATION_TOOL_PATTERN).ok());

/// One `session/update` notification, reduced to what detection reads.
///
/// **This is deliberately not [`via_downstream::RawSessionUpdate`].** That type
/// is the *public progress surface*'s input and has nowhere to put a
/// `rawOutput` — which is exactly the field a session id hides in. Detection
/// reads it and publishes none of it: what escapes this module is a
/// [`NativeDelegation`], whose four fields are the same four
/// [`via_work::DelegationRef`] already persists.
///
/// # Absent and `null` are the same thing here
///
/// A JavaScript spread lets an explicit `null` overwrite an inherited value,
/// where [`Option::or`] treats both as "inherit". The five fields merged here
/// are read as `name || title`, `status === 'completed'`,
/// `nativeToolOutput(rawOutput)` and `rawInput?.x` — every one of which treats
/// `null` and absent identically, so the difference is unobservable. The choice
/// matches [`via_downstream::RawSessionUpdate`]'s and is recorded in
/// `docs/deviations/phase-3.md`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NativeToolUpdate {
    /// The `sessionUpdate` discriminant.
    pub session_update: String,
    /// `toolCallId`, the key the merge is done on.
    pub tool_call_id: String,
    /// The tool's name.
    pub name: Option<String>,
    /// The tool call's title.
    pub title: Option<String>,
    /// The raw status string.
    pub status: Option<String>,
    /// The tool's arguments.
    pub raw_input: Option<Value>,
    /// The tool's result. **Never published.**
    pub raw_output: Option<Value>,
}

impl NativeToolUpdate {
    /// A `tool_call` update for `tool_call_id`.
    #[must_use]
    pub fn tool_call(tool_call_id: &str) -> Self {
        Self {
            session_update: UPDATE_TOOL_CALL.to_owned(),
            tool_call_id: tool_call_id.to_owned(),
            ..Self::default()
        }
    }

    /// A `tool_call_update` for `tool_call_id`.
    #[must_use]
    pub fn tool_call_update(tool_call_id: &str) -> Self {
        Self {
            session_update: UPDATE_TOOL_CALL_UPDATE.to_owned(),
            tool_call_id: tool_call_id.to_owned(),
            ..Self::default()
        }
    }

    /// Set the tool name.
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Set the tool call's title.
    #[must_use]
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_owned());
        self
    }

    /// Set the status.
    #[must_use]
    pub fn status(mut self, status: &str) -> Self {
        self.status = Some(status.to_owned());
        self
    }

    /// Set the arguments.
    #[must_use]
    pub fn raw_input(mut self, raw_input: Value) -> Self {
        self.raw_input = Some(raw_input);
        self
    }

    /// Set the result.
    #[must_use]
    pub fn raw_output(mut self, raw_output: Value) -> Self {
        self.raw_output = Some(raw_output);
        self
    }

    /// Whether this update is one of the two kinds detection looks at.
    #[must_use]
    pub fn is_tool_call(&self) -> bool {
        matches!(
            self.session_update.as_str(),
            UPDATE_TOOL_CALL | UPDATE_TOOL_CALL_UPDATE
        )
    }

    /// `{...known, ...update}` — a field this update carries wins, one it omits
    /// is inherited.
    fn merged_over(&self, previous: &Self) -> Self {
        Self {
            session_update: self.session_update.clone(),
            tool_call_id: if self.tool_call_id.is_empty() {
                previous.tool_call_id.clone()
            } else {
                self.tool_call_id.clone()
            },
            name: self.name.clone().or_else(|| previous.name.clone()),
            title: self.title.clone().or_else(|| previous.title.clone()),
            status: self.status.clone().or_else(|| previous.status.clone()),
            raw_input: self
                .raw_input
                .clone()
                .or_else(|| previous.raw_input.clone()),
            raw_output: self
                .raw_output
                .clone()
                .or_else(|| previous.raw_output.clone()),
        }
    }

    /// `String(merged.name || merged.title).trim().toLowerCase()`.
    ///
    /// JavaScript's `||` picks the first **truthy** value, and a non-empty
    /// string is truthy however much of it is whitespace — so a `name` of
    /// `"   "` wins over a real `title` and the result is empty. Reproduced,
    /// and tested: the difference decides whether detection fires.
    #[must_use]
    fn tool_name(&self) -> String {
        let chosen = self
            .name
            .as_deref()
            .filter(|name| !name.is_empty())
            .or(self.title.as_deref())
            .unwrap_or_default();
        clean(chosen).to_lowercase()
    }

    /// The first **truthy** value of `keys` in `rawInput`, if any.
    fn raw_input_field(&self, keys: &[&str]) -> Option<&Value> {
        let object = self.raw_input.as_ref().and_then(Value::as_object)?;
        first_truthy(object, keys)
    }
}

/// `a || b || c` over a JSON object — the first **truthy** member, not the
/// first non-empty one.
///
/// The distinction is upstream's and is observable: `clean(a || b)` with a
/// whitespace-only `a` yields the empty string, where a first-non-empty walk
/// would have moved on to `b`. Every `||` chain this module reproduces goes
/// through here.
fn first_truthy<'a>(object: &'a Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    keys.iter()
        .find_map(|key| object.get(*key).filter(|value| is_truthy(value)))
}

/// JavaScript truthiness.
///
/// The same predicate `via-acp`'s traversal applies, restated here only because
/// it is private there; both are three lines and both are tested against the
/// falsy values that matter — `""`, `0`, `false`, `null`.
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64() != Some(0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// Whether `name` names a delegating tool.
///
/// **External contract** — the predicate's fourth clause. Exposed because it is
/// the one part of detection a caller may want to ask about on its own, and
/// because a test that asserts the regex should not have to build an update to
/// do it.
#[must_use]
pub fn is_delegation_tool(name: &str) -> bool {
    DELEGATION_TOOL
        .as_ref()
        .is_some_and(|pattern| pattern.is_match(&clean(name).to_lowercase()))
}

/// What a caller supplies for the two fields the tool call may not carry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NativeDelegationDefaults {
    /// The coordinator's own directory, used when the tool named none.
    pub directory: String,
    /// The profile's delegation title, used when the tool named none.
    ///
    /// Upstream's `profile.defaultDelegationTitle || `${label} 项目任务``; the
    /// second half is [`via_i18n::keys::ACP_PROJECT_TASK_LABEL`], rendered by
    /// whoever knows the backend's label.
    pub title: String,
}

/// A Layer-3 Session a backend opened by itself.
///
/// The four fields [`via_work::DelegationRef`] persists, which is what makes
/// this path and the MCP path one lifecycle rather than two.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeDelegation {
    /// The backend's own `runId`, verbatim.
    pub delegation_id: String,
    /// The Layer-3 Session's id.
    pub session_id: String,
    /// Its project directory.
    pub directory: String,
    /// Its title, bounded to [`TITLE_BOUND`].
    pub title: String,
}

/// The per-turn state detection needs: the merge map, and whether this turn has
/// already delegated.
///
/// Upstream's `run.nativeToolCalls` and `run.delegation`
/// (`acp-backend-adapter.mjs:1000-1005`). One per coordinator turn — the map is
/// **not** pruned when a tool call completes, unlike
/// [`via_downstream::ActivityTracker`]'s, because a completed call is exactly
/// the one detection cares about and forgetting it would let a repeated
/// `tool_call_update` re-detect the same delegation.
#[derive(Debug, Default)]
pub struct NativeDelegationDetector {
    calls: IndexMap<String, NativeToolUpdate>,
    delegated: bool,
}

impl NativeDelegationDetector {
    /// A detector for one coordinator turn.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether this turn has already produced a delegation.
    #[must_use]
    pub const fn has_delegated(&self) -> bool {
        self.delegated
    }

    /// How many distinct tool calls this turn has seen.
    ///
    /// One of the four inputs to the empty-response recovery predicate
    /// (`docs/reference/contracts.json`, *empty coordinator response*), which
    /// is why it is exposed rather than internal.
    #[must_use]
    pub fn tool_calls(&self) -> usize {
        self.calls.len()
    }

    /// Mark the turn as having delegated through some other path.
    ///
    /// The MCP path creates a delegation without any `session/update` going
    /// past, and upstream's third clause — *`run.delegation` is still null* —
    /// covers both paths with one field. This is how that field is set from
    /// outside.
    pub fn mark_delegated(&mut self) {
        self.delegated = true;
    }

    /// Fold one update in, and report a delegation if this is the one.
    ///
    /// **External contract** — the whole predicate. Returns `None` for every
    /// update that does not satisfy all six clauses, including every update
    /// after the first delegation.
    pub fn observe(
        &mut self,
        update: &NativeToolUpdate,
        defaults: &NativeDelegationDefaults,
    ) -> Option<NativeDelegation> {
        if !update.is_tool_call() {
            return None;
        }
        let id = clean(&update.tool_call_id).to_owned();
        let merged = match self.calls.get(&id) {
            Some(previous) => update.merged_over(previous),
            None => update.clone(),
        };
        self.calls.insert(id, merged.clone());

        // `merged.status !== 'completed'` — a strict comparison with no trim,
        // exactly as upstream writes it.
        if merged.status.as_deref() != Some(completed_status()) {
            return None;
        }
        if self.delegated {
            return None;
        }
        if !is_delegation_tool(&merged.tool_name()) {
            return None;
        }
        let output = native_tool_output(merged.raw_output.as_ref().unwrap_or(&Value::Null));
        let session_id = clean_value(first_truthy(&output, &SESSION_ID_KEYS));
        if session_id.is_empty() {
            return None;
        }
        let delegation_id = clean_value(output.get(RUN_ID_KEY));
        if delegation_id.is_empty() {
            return None;
        }

        // `clean(rawInput.cwd || rawInput.directory || this.directory)`: the
        // coordinator's own directory is the **third arm of the same `||`**, so
        // a tool that named a whitespace-only cwd gets the empty string rather
        // than falling back to it.
        let directory = merged.raw_input_field(&DIRECTORY_KEYS).map_or_else(
            || clean(&defaults.directory).to_owned(),
            |value| clean_value(Some(value)),
        );
        // `bounded(rawInput.label || rawInput.task || rawInput.message, 160)
        //  || defaultTitle`: here the fallback is **outside** the `||`, so a
        // whitespace-only label does reach the profile's title.
        let title = bounded(
            &merged
                .raw_input_field(&TITLE_KEYS)
                .map_or_else(String::new, |value| clean_value(Some(value))),
            TITLE_BOUND,
        );
        let title = if title.is_empty() {
            defaults.title.clone()
        } else {
            title
        };

        self.delegated = true;
        Some(NativeDelegation {
            delegation_id,
            session_id,
            directory,
            title,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    fn defaults() -> NativeDelegationDefaults {
        NativeDelegationDefaults {
            directory: "/coordinator".to_owned(),
            title: "OpenClaw 项目任务".to_owned(),
        }
    }

    fn spawn_output() -> Value {
        json!({
            "content": [{
                "type": "text",
                "text": json!({
                    "status": "accepted",
                    "runId": "run-one",
                    "childSessionKey": "agent:child:one",
                })
                .to_string(),
            }],
        })
    }

    fn completed_spawn() -> NativeToolUpdate {
        NativeToolUpdate::tool_call("spawn-one")
            .name("sessions_spawn")
            .title("Spawn project Session")
            .status("completed")
            .raw_input(json!({ "task": "build project", "cwd": "/project" }))
            .raw_output(spawn_output())
    }

    #[test]
    fn the_pattern_compiles() {
        assert!(DELEGATION_TOOL.is_some());
    }

    #[test]
    fn the_regex_matches_the_two_third_party_tools_and_their_namespaced_forms() {
        for name in [
            "sessions_spawn",
            "sessions_send",
            "mcp__openclaw__sessions_spawn",
            "  SESSIONS_SPAWN  ",
            "sessions_spawn (build the thing)",
        ] {
            assert!(is_delegation_tool(name), "{name}");
        }
        for name in [
            "",
            "sessions_list",
            "sessions_history",
            "session_spawn",
            "spawn_sessions",
            "via_session_start",
        ] {
            assert!(!is_delegation_tool(name), "{name}");
        }
    }

    #[test]
    fn the_via_tools_are_not_native_delegation_tools() {
        for name in via_mcp_tools::SESSION_TOOL_NAMES {
            assert!(
                !is_delegation_tool(name),
                "{name} is VIA's own; detection must not fire on it",
            );
        }
    }

    #[test]
    fn a_completed_spawn_with_both_ids_is_a_delegation() {
        let mut detector = NativeDelegationDetector::new();
        let delegation = detector
            .observe(&completed_spawn(), &defaults())
            .expect("a delegation");
        assert_eq!(
            delegation,
            NativeDelegation {
                delegation_id: "run-one".to_owned(),
                session_id: "agent:child:one".to_owned(),
                directory: "/project".to_owned(),
                title: "build project".to_owned(),
            },
        );
        assert!(detector.has_delegated());
        assert_eq!(detector.tool_calls(), 1);
    }

    #[test]
    fn one_turn_delegates_once() {
        let mut detector = NativeDelegationDetector::new();
        assert!(detector.observe(&completed_spawn(), &defaults()).is_some());
        let second = NativeToolUpdate::tool_call("spawn-two")
            .name("sessions_spawn")
            .status("completed")
            .raw_output(json!({ "runId": "run-two", "sessionKey": "agent:child:two" }));
        assert_eq!(
            detector.observe(&second, &defaults()),
            None,
            "run.delegation is no longer null",
        );
        assert_eq!(detector.tool_calls(), 2, "the call is still merged in");
    }

    #[test]
    fn the_status_is_read_from_the_merge_not_from_one_update() {
        let mut detector = NativeDelegationDetector::new();
        let announced = NativeToolUpdate::tool_call("spawn-one")
            .name("sessions_spawn")
            .status("in_progress")
            .raw_input(json!({ "task": "build project" }))
            .raw_output(spawn_output());
        assert_eq!(detector.observe(&announced, &defaults()), None);

        // The completion carries only the status; everything else is inherited.
        let finished = NativeToolUpdate::tool_call_update("spawn-one").status("completed");
        let delegation = detector
            .observe(&finished, &defaults())
            .expect("the merge supplies the name and the output");
        assert_eq!(delegation.delegation_id, "run-one");
        assert_eq!(delegation.session_id, "agent:child:one");
        assert_eq!(
            delegation.directory, "/coordinator",
            "no cwd was named, so the coordinator's own directory is used",
        );
    }

    #[test]
    fn missing_either_id_creates_nothing() {
        let mut detector = NativeDelegationDetector::new();
        let no_run = NativeToolUpdate::tool_call("a")
            .name("sessions_spawn")
            .status("completed")
            .raw_output(json!({ "childSessionKey": "agent:child:one" }));
        assert_eq!(detector.observe(&no_run, &defaults()), None);

        let no_session = NativeToolUpdate::tool_call("b")
            .name("sessions_send")
            .status("completed")
            .raw_output(json!({ "runId": "run-one" }));
        assert_eq!(detector.observe(&no_session, &defaults()), None);
        assert!(!detector.has_delegated());
    }

    #[test]
    fn a_non_tool_update_is_ignored_entirely() {
        let mut detector = NativeDelegationDetector::new();
        let mut update = completed_spawn();
        update.session_update = "agent_message_chunk".to_owned();
        assert_eq!(detector.observe(&update, &defaults()), None);
        assert_eq!(
            detector.tool_calls(),
            0,
            "it is not even merged in — the first clause returns before the map is touched",
        );
    }

    #[test]
    fn the_session_id_key_order_is_the_adapters_not_the_traversals() {
        // Both spellings present, different values. `session_id` wins here;
        // `via-acp`'s own traversal order would have picked `sessionId`.
        let mut detector = NativeDelegationDetector::new();
        let update = NativeToolUpdate::tool_call("a")
            .name("sessions_spawn")
            .status("completed")
            .raw_output(json!({
                "runId": "run-one",
                "sessionId": "camel",
                "session_id": "snake",
            }));
        let delegation = detector
            .observe(&update, &defaults())
            .expect("a delegation");
        assert_eq!(delegation.session_id, "snake");
    }

    #[test]
    fn the_title_falls_back_through_three_keys_then_to_the_profile() {
        let mut detector = NativeDelegationDetector::new();
        let labelled = NativeToolUpdate::tool_call("a")
            .name("sessions_spawn")
            .status("completed")
            .raw_input(json!({ "label": "Labelled", "task": "Task", "message": "Message" }))
            .raw_output(json!({ "runId": "r", "sessionKey": "s" }));
        assert_eq!(
            detector.observe(&labelled, &defaults()).map(|d| d.title),
            Some("Labelled".to_owned()),
        );

        let mut detector = NativeDelegationDetector::new();
        let messaged = NativeToolUpdate::tool_call("a")
            .name("sessions_send")
            .status("completed")
            .raw_input(json!({ "message": "  Continue   the   work  " }))
            .raw_output(json!({ "runId": "r", "sessionKey": "s" }));
        assert_eq!(
            detector.observe(&messaged, &defaults()).map(|d| d.title),
            Some("Continue the work".to_owned()),
            "the title is `bounded`, which collapses whitespace runs",
        );

        let mut detector = NativeDelegationDetector::new();
        let bare = NativeToolUpdate::tool_call("a")
            .name("sessions_spawn")
            .status("completed")
            .raw_output(json!({ "runId": "r", "sessionKey": "s" }));
        assert_eq!(
            detector.observe(&bare, &defaults()).map(|d| d.title),
            Some("OpenClaw 项目任务".to_owned()),
        );
    }

    #[test]
    fn a_long_title_is_bounded_to_a_hundred_and_sixty() {
        let mut detector = NativeDelegationDetector::new();
        let update = NativeToolUpdate::tool_call("a")
            .name("sessions_spawn")
            .status("completed")
            .raw_input(json!({ "task": "t".repeat(TITLE_BOUND + 40) }))
            .raw_output(json!({ "runId": "r", "sessionKey": "s" }));
        let delegation = detector
            .observe(&update, &defaults())
            .expect("a delegation");
        assert_eq!(delegation.title.chars().count(), TITLE_BOUND);
    }

    #[test]
    fn the_name_wins_over_the_title_and_the_title_is_the_fallback() {
        let mut detector = NativeDelegationDetector::new();
        let title_only = NativeToolUpdate::tool_call("a")
            .title("sessions_spawn (project)")
            .status("completed")
            .raw_output(json!({ "runId": "r", "sessionKey": "s" }));
        assert!(detector.observe(&title_only, &defaults()).is_some());

        let mut detector = NativeDelegationDetector::new();
        let mismatched = NativeToolUpdate::tool_call("a")
            .name("bash")
            .title("sessions_spawn")
            .status("completed")
            .raw_output(json!({ "runId": "r", "sessionKey": "s" }));
        assert_eq!(
            detector.observe(&mismatched, &defaults()),
            None,
            "`name || title` means a present name is the whole answer",
        );
    }

    #[test]
    fn a_marked_delegation_stops_detection() {
        let mut detector = NativeDelegationDetector::new();
        detector.mark_delegated();
        assert_eq!(detector.observe(&completed_spawn(), &defaults()), None);
        assert!(detector.has_delegated());
    }

    #[test]
    fn hostile_output_shapes_yield_nothing_rather_than_a_delegation() {
        for output in [
            json!(null),
            json!("not json"),
            json!([]),
            json!({ "runId": "", "sessionKey": "s" }),
            json!({ "runId": "  ", "sessionKey": "s" }),
            json!({ "runId": 0, "sessionKey": "s" }),
            json!({ "runId": "r", "sessionKey": "   " }),
        ] {
            let mut detector = NativeDelegationDetector::new();
            let update = NativeToolUpdate::tool_call("a")
                .name("sessions_spawn")
                .status("completed")
                .raw_output(output.clone());
            assert_eq!(detector.observe(&update, &defaults()), None, "{output}");
        }
    }
}
