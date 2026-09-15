//! Secret redaction — the security boundary of this crate.
//!
//! A log line that leaks `DASHSCOPE_API_KEY` or a gateway `Bearer` token is a
//! real incident, so every value that reaches a sink passes through here first.
//!
//! **External contract.** The four patterns, the four sentinel strings and the
//! three caps are reproduced from upstream `shared/logger.mjs:26-33,59-110`:
//!
//! ```text
//! REDACTED          = '[REDACTED]'
//! SENSITIVE_KEY     = /(?:api[_-]?key|authorization|cookie|credential|password|secret|token$)/i
//! AUTH_VALUE        = /\b(?:Bearer|Basic)\s+[A-Za-z0-9._~+/=-]+/gi
//! API_KEY_VALUE     = /\bsk-[A-Za-z0-9_-]{8,}\b/g
//! URL_SECRET_VALUE  = /([?&](?:api[_-]?key|token|access_token|secret|password)=)[^&\s]+/gi
//! MAX_STRING_CHARS  = 32_000   suffix '…[truncated]'
//! MAX_COLLECTION_ITEMS = 100   marker '[Truncated]'
//! MAX_DEPTH         = 8        marker '[MaxDepth]'
//! circular marker   = '[Circular]'
//! ```
//!
//! Two properties are worth stating because they are easy to break:
//!
//! * `SENSITIVE_KEY` is a *substring* match, so `accessToken` and `apiKey`
//!   redact, but only the `token` alternative is anchored (`token$`) — which is
//!   what keeps `inputTokens` readable.
//! * The key test runs *before* the value test, so a sensitive key redacts the
//!   whole subtree, not just its strings.

use regex::{Captures, Regex};
use serde_json::{Map, Value};
use std::sync::LazyLock;

/// The replacement written in place of a secret.
///
/// **External contract** — upstream `shared/logger.mjs:29`.
pub const REDACTED: &str = "[REDACTED]";

/// Marker for a value already visited on this path.
///
/// **External contract** — upstream `shared/logger.mjs:95`.
///
/// A [`serde_json::Value`] tree is acyclic by construction, so this crate
/// never *produces* the marker; it is reproduced because it is part of the
/// documented marker vocabulary and appears in log files written by upstream.
pub const CIRCULAR: &str = "[Circular]";

/// Marker written where nesting exceeds [`MAX_DEPTH`].
///
/// **External contract** — upstream `shared/logger.mjs:94`.
pub const MAX_DEPTH_MARKER: &str = "[MaxDepth]";

/// Marker appended to an array clipped at [`MAX_COLLECTION_ITEMS`].
///
/// **External contract** — upstream `shared/logger.mjs:101`.
pub const TRUNCATED_MARKER: &str = "[Truncated]";

/// Suffix appended to a string clipped at [`MAX_STRING_CHARS`].
///
/// **External contract** — upstream `shared/logger.mjs:65`.
pub const TRUNCATED_SUFFIX: &str = "…[truncated]";

/// Longest string kept intact.
///
/// **External contract** — upstream `shared/logger.mjs:26`.
pub const MAX_STRING_CHARS: usize = 32_000;

/// Longest array or object kept intact.
///
/// **External contract** — upstream `shared/logger.mjs:27`.
pub const MAX_COLLECTION_ITEMS: usize = 100;

/// Deepest nesting level walked before [`MAX_DEPTH_MARKER`] is written.
///
/// **External contract** — upstream `shared/logger.mjs:28`.
pub const MAX_DEPTH: usize = 8;

/// Keys whose value is redacted wholesale.
///
/// **External contract** — upstream `SENSITIVE_KEY`
/// (`shared/logger.mjs:30`). `(?-u:\b)` is not present upstream; it is the
/// Rust spelling of JavaScript's ASCII-only word boundary and appears only in
/// the value patterns below.
pub const SENSITIVE_KEY_PATTERN: &str =
    r"(?i)(?:api[_-]?key|authorization|cookie|credential|password|secret|token$)";

/// `Bearer` / `Basic` credentials anywhere in free text.
///
/// **External contract** — upstream `AUTH_VALUE` (`shared/logger.mjs:31`).
pub const AUTH_VALUE_PATTERN: &str = r"(?i)(?-u:\b)(?:Bearer|Basic)\s+[A-Za-z0-9._~+/=-]+";

/// OpenAI-style `sk-…` keys anywhere in free text. Case sensitive upstream.
///
/// **External contract** — upstream `API_KEY_VALUE` (`shared/logger.mjs:32`).
pub const API_KEY_VALUE_PATTERN: &str = r"(?-u:\b)sk-[A-Za-z0-9_-]{8,}(?-u:\b)";

/// Secrets carried in a URL query string.
///
/// **External contract** — upstream `URL_SECRET_VALUE`
/// (`shared/logger.mjs:33`).
pub const URL_SECRET_VALUE_PATTERN: &str =
    r"(?i)([?&](?:api[_-]?key|token|access_token|secret|password)=)[^&\s]+";

// Every pattern below is a crate-private compile-time literal, and
// `patterns_compile` asserts each one builds. A panic here is unreachable for
// any build that passes the test suite.
static SENSITIVE_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(SENSITIVE_KEY_PATTERN).expect("SENSITIVE_KEY_PATTERN is a valid regex")
});
static AUTH_VALUE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(AUTH_VALUE_PATTERN).expect("AUTH_VALUE_PATTERN is a valid regex"));
static API_KEY_VALUE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(API_KEY_VALUE_PATTERN).expect("API_KEY_VALUE_PATTERN is a valid regex")
});
static URL_SECRET_VALUE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(URL_SECRET_VALUE_PATTERN).expect("URL_SECRET_VALUE_PATTERN is a valid regex")
});

/// Whether a field name alone is enough to redact its value.
///
/// **External contract** — upstream `SENSITIVE_KEY.test(key)`
/// (`shared/logger.mjs:89`).
#[must_use]
pub fn is_sensitive_key(key: &str) -> bool {
    SENSITIVE_KEY.is_match(key)
}

/// Scrub secrets out of a free-text string and clip it to
/// [`MAX_STRING_CHARS`].
///
/// The three substitutions run in upstream's order — auth header shape, then
/// `sk-` keys, then URL query secrets — because the first rewrite can create
/// or destroy a match for the later ones.
///
/// **External contract** — upstream `scrubString`
/// (`shared/logger.mjs:59-67`).
#[must_use]
pub fn scrub_string(value: &str) -> String {
    let scrubbed = AUTH_VALUE.replace_all(value, |caps: &Captures<'_>| {
        let matched = caps.get(0).map_or("", |m| m.as_str());
        // Upstream: `${match.split(/\s/, 1)[0]} ${REDACTED}` — keep the scheme
        // word with its original casing, drop everything after it.
        let scheme = matched.split(char::is_whitespace).next().unwrap_or(matched);
        format!("{scheme} {REDACTED}")
    });
    let scrubbed = API_KEY_VALUE.replace_all(&scrubbed, regex::NoExpand(REDACTED));
    let scrubbed = URL_SECRET_VALUE.replace_all(&scrubbed, |caps: &Captures<'_>| {
        let prefix = caps.get(1).map_or("", |m| m.as_str());
        format!("{prefix}{REDACTED}")
    });
    truncate_string(&scrubbed)
}

fn truncate_string(value: &str) -> String {
    // Upstream measures in UTF-16 code units (`String.prototype.length`);
    // counting Unicode scalar values is the closest total function in Rust and
    // is identical for the ASCII payloads this cap exists to bound. See the
    // crate docs for the recorded deviation.
    let mut boundary = None;
    for (index, (offset, _)) in value.char_indices().enumerate() {
        if index == MAX_STRING_CHARS {
            boundary = Some(offset);
            break;
        }
    }
    match boundary {
        Some(offset) => format!("{}{TRUNCATED_SUFFIX}", &value[..offset]),
        None => value.to_owned(),
    }
}

/// Redact one value, given the field name it was found under.
///
/// **External contract** — upstream `redactLogValue`
/// (`shared/logger.mjs:83-110`).
#[must_use]
pub fn redact_value_for_key(value: &Value, key: &str) -> Value {
    redact_inner(value, key, 0)
}

/// Redact one value that has no field name of its own (an array element, or a
/// top-level payload).
#[must_use]
pub fn redact_value(value: &Value) -> Value {
    redact_inner(value, "", 0)
}

/// Redact a whole field map, applying the key test to each entry.
///
/// The map is clipped to [`MAX_COLLECTION_ITEMS`] entries, matching upstream's
/// object branch, which clips but — unlike the array branch — appends no
/// marker.
///
/// Insertion order is preserved: it is observable in every log line.
#[must_use]
pub fn redact_map(map: &Map<String, Value>) -> Map<String, Value> {
    redact_object(map, 0)
}

fn redact_object(map: &Map<String, Value>, depth: usize) -> Map<String, Value> {
    map.iter()
        .take(MAX_COLLECTION_ITEMS)
        .map(|(key, value)| (key.clone(), redact_inner(value, key, depth + 1)))
        .collect()
}

fn redact_inner(value: &Value, key: &str, depth: usize) -> Value {
    if is_sensitive_key(key) {
        return Value::String(REDACTED.to_owned());
    }
    match value {
        // Upstream returns null/undefined untouched, and returns non-object
        // primitives untouched, *before* the depth guard applies.
        Value::Null | Value::Bool(_) | Value::Number(_) => value.clone(),
        Value::String(text) => Value::String(scrub_string(text)),
        Value::Array(items) => {
            if depth >= MAX_DEPTH {
                return Value::String(MAX_DEPTH_MARKER.to_owned());
            }
            let mut redacted: Vec<Value> = items
                .iter()
                .take(MAX_COLLECTION_ITEMS)
                .map(|item| redact_inner(item, "", depth + 1))
                .collect();
            if items.len() > MAX_COLLECTION_ITEMS {
                redacted.push(Value::String(TRUNCATED_MARKER.to_owned()));
            }
            Value::Array(redacted)
        }
        Value::Object(entries) => {
            if depth >= MAX_DEPTH {
                return Value::String(MAX_DEPTH_MARKER.to_owned());
            }
            Value::Object(redact_object(entries, depth))
        }
    }
}

/// A serialized error, in upstream's field order.
///
/// **External contract** — upstream `serializeError`
/// (`shared/logger.mjs:69-81`): `name`, `message`, then `code` and `stack`
/// only when present, then any remaining properties whose names do not
/// collide with those four.
///
/// Rust errors carry no `name` and no captured stack, so [`Self::from_error`]
/// uses upstream's own default of `"Error"` and leaves `stack` unset rather
/// than inventing a value.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ErrorRecord {
    /// `error.name`, defaulting to `"Error"`.
    pub name: String,
    /// `error.message`.
    pub message: String,
    /// `error.code`, when the error carries one.
    pub code: Option<String>,
    /// `error.stack`, when one was captured.
    pub stack: Option<String>,
    /// Remaining error properties, in insertion order.
    pub extra: Map<String, Value>,
}

impl ErrorRecord {
    /// Build a record from any [`std::error::Error`].
    #[must_use]
    pub fn from_error(error: &(dyn std::error::Error + 'static)) -> Self {
        Self {
            name: "Error".to_owned(),
            message: error.to_string(),
            code: None,
            stack: None,
            extra: Map::new(),
        }
    }

    /// Build a record from a plain message.
    #[must_use]
    pub fn from_message(message: impl Into<String>) -> Self {
        Self {
            name: "Error".to_owned(),
            message: message.into(),
            code: None,
            stack: None,
            extra: Map::new(),
        }
    }

    /// Render the record as a redacted JSON object in upstream's field order.
    #[must_use]
    pub fn to_value(&self) -> Value {
        let mut result = Map::new();
        let name = if self.name.is_empty() {
            "Error"
        } else {
            self.name.as_str()
        };
        result.insert("name".to_owned(), Value::String(scrub_string(name)));
        result.insert(
            "message".to_owned(),
            Value::String(scrub_string(&self.message)),
        );
        if let Some(code) = &self.code {
            result.insert("code".to_owned(), Value::String(scrub_string(code)));
        }
        if let Some(stack) = &self.stack {
            result.insert("stack".to_owned(), Value::String(scrub_string(stack)));
        }
        for (key, value) in self.extra.iter().take(MAX_COLLECTION_ITEMS) {
            // Upstream: `if (key in result) continue`.
            if result.contains_key(key) {
                continue;
            }
            result.insert(key.clone(), redact_inner(value, key, 1));
        }
        Value::Object(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn patterns_compile() {
        for pattern in [
            SENSITIVE_KEY_PATTERN,
            AUTH_VALUE_PATTERN,
            API_KEY_VALUE_PATTERN,
            URL_SECRET_VALUE_PATTERN,
        ] {
            assert!(Regex::new(pattern).is_ok(), "pattern failed: {pattern}");
        }
    }

    #[test]
    fn sentinels_are_the_upstream_literals() {
        assert_eq!(REDACTED, "[REDACTED]");
        assert_eq!(CIRCULAR, "[Circular]");
        assert_eq!(MAX_DEPTH_MARKER, "[MaxDepth]");
        assert_eq!(TRUNCATED_MARKER, "[Truncated]");
        assert_eq!(TRUNCATED_SUFFIX, "…[truncated]");
        assert_eq!(MAX_STRING_CHARS, 32_000);
        assert_eq!(MAX_COLLECTION_ITEMS, 100);
        assert_eq!(MAX_DEPTH, 8);
    }

    #[test]
    fn error_record_keeps_upstream_field_order() {
        let record = ErrorRecord {
            name: "TypeError".to_owned(),
            message: "boom".to_owned(),
            code: Some("ENOENT".to_owned()),
            stack: Some("at <anonymous>".to_owned()),
            extra: {
                let mut extra = Map::new();
                extra.insert("authorization".to_owned(), json!("Bearer leak"));
                extra.insert("attempt".to_owned(), json!(2));
                extra
            },
        };
        let value = record.to_value();
        let Value::Object(map) = &value else {
            panic!("expected an object");
        };
        let keys: Vec<&str> = map.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            [
                "name",
                "message",
                "code",
                "stack",
                "authorization",
                "attempt"
            ]
        );
        assert_eq!(map["authorization"], json!(REDACTED));
        assert_eq!(map["attempt"], json!(2));
    }
}
