//! Reading what a backend agent said.
//!
//! Ported from `server/src/agent/acp-backend-session-utils.mjs`: the payload
//! unwrapping algorithm, the presentation normalizer, the `sessions_list` row,
//! and the traversal that finds a session key inside arbitrary third-party tool
//! output.
//!
//! # What is *not* here
//!
//! The same upstream file also holds `clean`, `bounded`, the two session key
//! formats and `activityFromUpdate`. All four are Layer-3 vocabulary rather
//! than ACP mechanics, and `via-downstream` owns them:
//! [`via_downstream::text`], [`via_downstream::SessionKey`] and
//! [`via_downstream::ActivityTracker`]. This module reaches for those rather
//! than restating them — a second `bounded` would be a second answer to "how
//! long may a hostile backend's string be on the public progress surface".
//!
//! # Why this works on JSON rather than on the SDK's typed updates
//!
//! `session/update` carries fields the ACP v1 schema does not declare — most
//! importantly `name` on a tool call, which upstream prefers over `title` when
//! naming an activity, and which real backends send. Deserializing into the
//! SDK's `SessionUpdate` would drop it silently and rename every tool in the
//! UI. Upstream reads the raw notification; so does this, and
//! [`crate::client`] hands the raw value to observers.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use via_downstream::text::{bounded, clean};
use via_i18n::{Locale, keys};

/// The bound on a session title in a `sessions_list` row.
///
/// **External contract** — `acp-backend-session-utils.mjs:72`
/// (`bounded(session?.title, 160)`).
pub const SESSION_TITLE_BOUND: usize = 160;

/// How many times [`parse_coordinator_payload`] will unwrap.
///
/// **External contract** — `acp-backend-session-utils.mjs:19` (`depth < 3`).
/// The iteration count decides which malformed model output is accepted, so it
/// is part of the contract rather than a tuning knob.
pub const PAYLOAD_UNWRAP_DEPTH: usize = 3;

/// The `format` a legacy string inline is upgraded to.
///
/// **External contract** — `acp-backend-session-utils.mjs:50`. A wire value,
/// not a localized one.
pub const LEGACY_INLINE_FORMAT: &str = "markdown";

/// `/```(?:json)?\s*([\s\S]*?)```/i` — non-greedy, so the **first** fence wins.
///
/// `Option` rather than an `unwrap`: the pattern is a literal with no runtime
/// input, so `None` is unreachable, and a crate that forbids panicking in a
/// voice turn should not make an unreachable case the one that ends it. If it
/// ever were `None` the fence step is skipped and the JSON parse still runs —
/// a strictly smaller behaviour rather than a wrong one. The test below is
/// what actually proves it compiles.
static FENCED_BLOCK: Lazy<Option<Regex>> =
    Lazy::new(|| Regex::new(r"(?is)```(?:json)?\s*(.*?)```").ok());

/// `String(value || '').trim()` over a JSON value.
///
/// [`via_downstream::text::clean`] is the string half; this is the coercion in
/// front of it, because these values arrive from a model and may be `null`, a
/// number, or missing entirely. The falsy cases — `null`, `false`, `0`, absent
/// — all become the empty string, exactly as `value || ''` does.
///
/// One divergence, recorded because it is real but unreachable in practice: an
/// **array or object** renders as JSON here and as `"1,2"` /
/// `"[object Object]"` in JavaScript. Every call site reads a field the
/// protocol declares as a string; neither rendering is useful, and JSON is at
/// least diagnosable.
#[must_use]
pub fn clean_value(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => clean(text).to_owned(),
        Some(Value::Null) | Some(Value::Bool(false)) | None => String::new(),
        Some(Value::Number(number)) if number.as_f64() == Some(0.0) => String::new(),
        Some(other) => clean(&other.to_string()).to_owned(),
    }
}

/// Dig a JSON object out of whatever a model actually returned.
///
/// **Contract** — *"parseCoordinatorPayload unwrapping algorithm"*,
/// `acp-backend-session-utils.mjs:17-39`. This is how **every** backend
/// model's final answer is parsed, so the exact shape of the loop is the
/// contract:
///
/// 1. take the first fenced code block, if there is one;
/// 2. parse — a JSON **string** result becomes the next candidate and the loop
///    continues (a model that double-encoded its answer is recovered); a
///    non-null object is returned; anything else — a number, `true`, `null`,
///    an array — returns `None`;
/// 3. on a parse failure, narrow to the span between the first `{` and the last
///    `}`; if that is not a strict narrowing, stop, because looping on it would
///    not terminate.
///
/// At most [`PAYLOAD_UNWRAP_DEPTH`] iterations. An off-by-one in either the
/// count or the no-progress guard changes which outputs are accepted.
#[must_use]
pub fn parse_coordinator_payload(content: &str) -> Option<Map<String, Value>> {
    let mut candidate = clean(content).to_owned();
    for _ in 0..PAYLOAD_UNWRAP_DEPTH {
        if candidate.is_empty() {
            return None;
        }
        if let Some(pattern) = FENCED_BLOCK.as_ref()
            && let Some(captures) = pattern.captures(&candidate)
            && let Some(inner) = captures.get(1)
        {
            candidate = clean(inner.as_str()).to_owned();
        }
        match serde_json::from_str::<Value>(&candidate) {
            Ok(Value::String(text)) => candidate = clean(&text).to_owned(),
            Ok(Value::Object(object)) => return Some(object),
            Ok(_) => return None,
            Err(_) => {
                let start = candidate.find('{')?;
                let end = candidate.rfind('}')?;
                if end <= start {
                    return None;
                }
                let narrowed = candidate[start..=end].to_owned();
                if narrowed == candidate {
                    return None;
                }
                candidate = narrowed;
            }
        }
    }
    None
}

/// Upgrade a legacy string `presentation.inline` into the object form.
///
/// **Contract** — *"normalizeCoordinatorContent legacy inline upgrade"*,
/// `acp-backend-session-utils.mjs:41-56`. Two details are catalogued and both
/// are easy to get subtly wrong:
///
/// * the blank test runs on the **trimmed** value, but the `content` that is
///   stored is the **original, untrimmed** string;
/// * a blank inline becomes `null`, not an object with empty content, because
///   `null` is what the presentation layer treats as "nothing to show".
///
/// A payload that does not parse is returned as the trimmed text it was.
/// A payload that does parse is re-serialized **compactly**, which is what
/// makes this function's output a stable input to the coordinator.
///
/// The upgraded title reaches the user's screen, so it comes from `via-i18n`
/// (`acp.agent_result_title`, upstream's `Agent 结果`) rather than a literal.
#[must_use]
pub fn normalize_coordinator_content(content: &str, locale: Locale) -> String {
    let text = clean(content);
    let Some(mut parsed) = parse_coordinator_payload(text) else {
        return text.to_owned();
    };
    let inline = parsed
        .get("presentation")
        .and_then(Value::as_object)
        .and_then(|presentation| presentation.get("inline"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    if let Some(inline) = inline {
        let replacement = if clean(&inline).is_empty() {
            Value::Null
        } else {
            let mut upgraded = Map::new();
            upgraded.insert(
                "title".to_owned(),
                Value::String(via_i18n::t(locale, keys::ACP_AGENT_RESULT_TITLE).to_owned()),
            );
            upgraded.insert(
                "format".to_owned(),
                Value::String(LEGACY_INLINE_FORMAT.to_owned()),
            );
            upgraded.insert("content".to_owned(), Value::String(inline));
            Value::Object(upgraded)
        };
        if let Some(Value::Object(presentation)) = parsed.get_mut("presentation") {
            presentation.insert("inline".to_owned(), replacement);
        }
    }
    Value::Object(parsed).to_string()
}

/// The speech and inline halves of a coordinator presentation.
///
/// **Contract** — `coordinatorPresentation`,
/// `acp-backend-session-utils.mjs:58-67`. `inline` survives only if it is
/// already an object: a string that was never normalized is dropped rather than
/// shown raw.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Presentation {
    /// What is spoken, trimmed; empty when absent.
    pub speech: String,
    /// What is shown, or `None`.
    pub inline: Option<Map<String, Value>>,
}

/// Read the presentation out of a coordinator answer.
///
/// Returns `None` when the answer does not parse, or parses to something with
/// no object-shaped `presentation`.
#[must_use]
pub fn coordinator_presentation(content: &str) -> Option<Presentation> {
    let presentation = parse_coordinator_payload(content)?
        .get("presentation")
        .and_then(Value::as_object)
        .cloned()?;
    Some(Presentation {
        speech: clean_value(presentation.get("speech")),
        inline: presentation
            .get("inline")
            .and_then(Value::as_object)
            .cloned(),
    })
}

/// One row of a `sessions_list` answer.
///
/// **External contract** — *"sessions_list result entry"*,
/// `acp-backend-session-utils.mjs:69-76`. Field names are snake_case because
/// they are model-visible: this is what an MCP tool hands back to a backend
/// agent, not an internal struct.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSummary {
    /// The backend's own session id.
    pub session_id: String,
    /// The title, bounded to [`SESSION_TITLE_BOUND`].
    pub title: String,
    /// The session's working directory.
    pub directory: String,
    /// The backend's own `updatedAt`, as a string.
    pub updated_at: String,
}

/// Project one `session/list` entry into its model-visible summary.
///
/// Reads the raw entry rather than the SDK's `SessionInfo` for the same reason
/// the module does elsewhere: `updatedAt` is whatever the backend put there,
/// and coercing it to a typed timestamp would either lose it or fail.
#[must_use]
pub fn session_summary(session: &Value) -> SessionSummary {
    SessionSummary {
        session_id: clean_value(session.get("sessionId")),
        title: bounded(&clean_value(session.get("title")), SESSION_TITLE_BOUND),
        directory: clean_value(session.get("cwd")),
        updated_at: clean_value(session.get("updatedAt")),
    }
}

/// Find the object carrying a session key inside arbitrary tool output.
///
/// **Contract** — *"nativeToolOutput extraction order"*,
/// `acp-backend-session-utils.mjs:142-172`. This reads output shapes owned by
/// third parties, so the traversal order is the contract: it decides which key
/// wins when several are present.
///
/// 1. a string is parsed and recursed into; unparseable gives `{}`;
/// 2. an array yields the **first** member whose recursion is non-empty;
/// 3. an object with a **truthy** `childSessionKey`, `sessionKey`, `sessionId`
///    or `session_id` is returned as-is — an empty-string session id does not
///    stop the search, which is what lets a wrapper that declares the field but
///    has not filled it in yet be traversed through;
/// 4. otherwise recurse into `details`;
/// 5. otherwise, for each member of `content`, recurse into `block.text`, else
///    `block.content`, else the block itself;
/// 6. otherwise `{}`.
#[must_use]
pub fn native_tool_output(value: &Value) -> Map<String, Value> {
    const SESSION_KEYS: [&str; 4] = ["childSessionKey", "sessionKey", "sessionId", "session_id"];

    match value {
        Value::String(text) if text.is_empty() => Map::new(),
        Value::String(text) => serde_json::from_str::<Value>(text)
            .map(|parsed| native_tool_output(&parsed))
            .unwrap_or_default(),
        Value::Array(items) => items
            .iter()
            .map(native_tool_output)
            .find(|parsed| !parsed.is_empty())
            .unwrap_or_default(),
        Value::Object(object) => {
            if SESSION_KEYS
                .iter()
                .any(|key| object.get(*key).is_some_and(is_truthy))
            {
                return object.clone();
            }
            if let Some(details) = object.get("details") {
                let parsed = native_tool_output(details);
                if !parsed.is_empty() {
                    return parsed;
                }
            }
            for block in object
                .get("content")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default()
            {
                let inner = block
                    .get("text")
                    .filter(|value| is_truthy(value))
                    .or_else(|| block.get("content").filter(|value| is_truthy(value)))
                    .unwrap_or(block);
                let parsed = native_tool_output(inner);
                if !parsed.is_empty() {
                    return parsed;
                }
            }
            Map::new()
        }
        // `!value` and `typeof value !== 'object'` both land here.
        _ => Map::new(),
    }
}

/// JavaScript truthiness, for the `a || b || c` chains this module reproduces.
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64() != Some(0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// The text one `session/update` contributed to the assistant's answer.
///
/// **Contract** — `textFromUpdate`, `acp-process-client.mjs:48-54`, and the
/// catalogued *"session/update"* note: the returned content is built **only**
/// from `agent_message_chunk` updates whose content block is text. Thought
/// chunks, user-message echoes and non-text content contribute nothing, which
/// is what keeps a backend that reasons out loud from having its reasoning
/// spoken.
///
/// The text is taken **verbatim**, not cleaned: chunks are concatenated and the
/// join is trimmed once at the end, so trimming here would delete the spaces
/// between words.
#[must_use]
pub fn text_from_update(update: &Value) -> &str {
    if update.get("sessionUpdate").and_then(Value::as_str)
        != Some(via_downstream::event::UPDATE_AGENT_MESSAGE_CHUNK)
    {
        return "";
    }
    let Some(content) = update.get("content") else {
        return "";
    };
    if content.get("type").and_then(Value::as_str) != Some("text") {
        return "";
    }
    content
        .get("text")
        .and_then(Value::as_str)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn only_agent_message_text_contributes_to_the_answer() {
        assert_eq!(
            text_from_update(&json!({
                "sessionUpdate": "agent_message_chunk",
                "content": { "type": "text", "text": " answer " },
            })),
            " answer ",
            "chunks are joined before trimming, so a chunk's own spaces survive"
        );
        for silent in [
            json!({ "sessionUpdate": "agent_thought_chunk", "content": { "type": "text", "text": "reasoning" } }),
            json!({ "sessionUpdate": "user_message_chunk", "content": { "type": "text", "text": "echo" } }),
            json!({ "sessionUpdate": "agent_message_chunk", "content": { "type": "image", "data": "x" } }),
            json!({ "sessionUpdate": "agent_message_chunk" }),
            json!({}),
        ] {
            assert_eq!(text_from_update(&silent), "", "{silent}");
        }
    }

    #[test]
    fn the_fence_pattern_compiles() {
        // The `Option` above is unreachable, and this is what keeps it so.
        assert!(FENCED_BLOCK.is_some());
    }

    #[test]
    fn payload_unwrapping_handles_every_shape_a_model_produces() {
        let object = json!({ "status": "completed" });
        assert_eq!(
            parse_coordinator_payload(r#"{"status":"completed"}"#),
            object.as_object().cloned()
        );
        assert_eq!(
            parse_coordinator_payload("```json\n{\"status\":\"completed\"}\n```"),
            object.as_object().cloned()
        );
        assert_eq!(
            parse_coordinator_payload("```\n{\"status\":\"completed\"}\n```"),
            object.as_object().cloned(),
            "the language tag is optional"
        );
        assert_eq!(
            parse_coordinator_payload("```JSON\n{\"status\":\"completed\"}\n```"),
            object.as_object().cloned(),
            "the fence match is case-insensitive"
        );
        assert_eq!(
            parse_coordinator_payload(r#""{\"status\":\"completed\"}""#),
            object.as_object().cloned(),
            "a double-encoded answer is recovered"
        );
        assert_eq!(
            parse_coordinator_payload("Sure! {\"status\":\"completed\"} hope that helps"),
            object.as_object().cloned(),
            "prose around an object is narrowed away"
        );
    }

    #[test]
    fn payload_unwrapping_refuses_what_upstream_refuses() {
        for refused in [
            "",
            "   ",
            "[1,2,3]",
            "42",
            "null",
            "true",
            "no braces here",
            "{not json}",
        ] {
            assert_eq!(
                parse_coordinator_payload(refused),
                None,
                "{refused:?} must not parse"
            );
        }
    }

    #[test]
    fn payload_unwrapping_gives_up_after_three_iterations() {
        let mut nested = json!({ "ok": true }).to_string();
        for _ in 0..3 {
            nested = Value::String(nested).to_string();
        }
        assert_eq!(
            parse_coordinator_payload(&nested),
            None,
            "a fourth unwrap is one too many"
        );

        let mut just_enough = json!({ "ok": true }).to_string();
        for _ in 0..2 {
            just_enough = Value::String(just_enough).to_string();
        }
        assert!(
            parse_coordinator_payload(&just_enough).is_some(),
            "three are allowed"
        );
    }

    #[test]
    fn the_legacy_inline_upgrade_keeps_the_untrimmed_content() {
        let normalized = normalize_coordinator_content(
            r#"{"presentation":{"speech":"done","inline":"  # Report  "}}"#,
            Locale::Zh,
        );
        let parsed: Value = serde_json::from_str(&normalized).expect("compact JSON");
        let inline = &parsed["presentation"]["inline"];
        assert_eq!(inline["title"], "Agent 结果");
        assert_eq!(inline["format"], "markdown");
        assert_eq!(
            inline["content"], "  # Report  ",
            "the blank check trims but the stored content does not"
        );
        assert!(
            !normalized.contains(": "),
            "the payload is re-serialized compactly: {normalized}"
        );
        assert_eq!(
            inline
                .as_object()
                .expect("an object")
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["title", "format", "content"],
            "key order is the upstream object literal's"
        );
    }

    #[test]
    fn a_blank_legacy_inline_becomes_null() {
        let normalized =
            normalize_coordinator_content(r#"{"presentation":{"inline":"   "}}"#, Locale::En);
        let parsed: Value = serde_json::from_str(&normalized).expect("compact JSON");
        assert_eq!(parsed["presentation"]["inline"], Value::Null);
    }

    #[test]
    fn an_unparseable_answer_is_returned_as_trimmed_text() {
        assert_eq!(
            normalize_coordinator_content("  just talking  ", Locale::En),
            "just talking"
        );
    }

    #[test]
    fn an_object_inline_is_left_alone() {
        let source =
            r#"{"presentation":{"inline":{"title":"Kept","format":"markdown","content":"x"}}}"#;
        let normalized = normalize_coordinator_content(source, Locale::En);
        let parsed: Value = serde_json::from_str(&normalized).expect("compact JSON");
        assert_eq!(parsed["presentation"]["inline"]["title"], "Kept");
    }

    #[test]
    fn presentation_drops_an_inline_that_is_still_a_string() {
        let presentation =
            coordinator_presentation(r#"{"presentation":{"speech":"  hi  ","inline":"raw"}}"#)
                .expect("a presentation is present");
        assert_eq!(presentation.speech, "hi");
        assert_eq!(presentation.inline, None);

        assert_eq!(coordinator_presentation("not json"), None);
        assert_eq!(coordinator_presentation(r#"{"presentation":"x"}"#), None);
        assert_eq!(coordinator_presentation(r#"{"other":1}"#), None);
    }

    #[test]
    fn session_summaries_bound_the_title_only() {
        let long = "t".repeat(400);
        let summary = session_summary(&json!({
            "sessionId": " s-1 ",
            "title": long,
            "cwd": " /work ",
            "updatedAt": " 2026-08-22 ",
        }));
        assert_eq!(summary.session_id, "s-1");
        assert_eq!(summary.title.chars().count(), SESSION_TITLE_BOUND);
        assert_eq!(summary.directory, "/work");
        assert_eq!(summary.updated_at, "2026-08-22");

        let empty = session_summary(&json!({}));
        assert_eq!(empty.session_id, "");
        assert_eq!(empty.title, "");
        assert_eq!(empty.directory, "");
        assert_eq!(empty.updated_at, "");
    }

    #[test]
    fn clean_value_collapses_every_falsy_json_value() {
        assert_eq!(clean_value(None), "");
        assert_eq!(clean_value(Some(&json!(null))), "");
        assert_eq!(clean_value(Some(&json!(false))), "");
        assert_eq!(clean_value(Some(&json!(0))), "");
        assert_eq!(clean_value(Some(&json!("  x  "))), "x");
        assert_eq!(clean_value(Some(&json!(7))), "7");
        assert_eq!(clean_value(Some(&json!(true))), "true");
    }

    #[test]
    fn native_tool_output_returns_the_first_object_carrying_a_session_key() {
        assert_eq!(
            native_tool_output(&json!({ "sessionId": "s-1", "extra": 1 }))["sessionId"],
            "s-1"
        );
        assert_eq!(
            native_tool_output(&json!("{\"sessionKey\":\"k\"}"))["sessionKey"],
            "k"
        );
        assert_eq!(
            native_tool_output(&json!([{ "nothing": 1 }, { "session_id": "s-2" }]))["session_id"],
            "s-2"
        );
        let both = native_tool_output(&json!({
            "details": { "sessionId": "from-details" },
            "content": [{ "text": "{\"sessionId\":\"from-content\"}" }],
        }));
        assert_eq!(
            both["sessionId"], "from-details",
            "`details` is searched before `content`"
        );
        assert_eq!(
            native_tool_output(&json!({
                "content": [{ "text": "{\"childSessionKey\":\"c\"}" }],
            }))["childSessionKey"],
            "c"
        );
        assert_eq!(
            native_tool_output(&json!({ "content": [{ "sessionId": "bare-block" }] }))["sessionId"],
            "bare-block"
        );
        assert_eq!(
            native_tool_output(&json!({
                "childSessionKey": "child",
                "sessionId": "own",
            }))["childSessionKey"],
            "child",
            "the object is returned whole, so the caller picks by its own order"
        );
    }

    #[test]
    fn native_tool_output_gives_up_quietly() {
        for empty in [
            json!(null),
            json!(""),
            json!("not json"),
            json!(42),
            json!(true),
            json!([]),
            json!([{ "a": 1 }]),
            json!({ "a": 1 }),
            json!({ "content": "not an array" }),
            json!({ "sessionId": "" }),
            json!({ "details": null }),
        ] {
            assert!(
                native_tool_output(&empty).is_empty(),
                "{empty} must yield no session"
            );
        }
    }
}
