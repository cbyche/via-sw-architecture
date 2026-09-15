//! The `via.log/v1` record spine.
//!
//! **External contract.** One JSON object per line:
//!
//! ```text
//! {...base, ...context, ...fields, "schema", "time", "level",
//!  "component", "event", "pid", "message"?}
//! ```
//!
//! The spine is written *last*, so a caller-supplied field named `schema`,
//! `level`, `component`, `event` or `pid` is overwritten rather than merged —
//! upstream `shared/logger.mjs:293-304`, and the reason
//! `docs/reference/contracts.json` records the entry as "Caller-supplied
//! fields named schema/level/component/event/pid MUST be ignored, not merged."
//!
//! `message` is deliberately **not** in that list: upstream spreads it only
//! when the caller passed a non-empty message, so a `message` field survives
//! an emit that supplies none.
//!
//! Map insertion order is the wire order. `serde_json` is configured with
//! `preserve_order`, so writing the spine over an existing key overwrites the
//! value *in place* and reproduces JavaScript's object-spread positioning
//! exactly.

use crate::level::LogLevel;
use crate::redact::{redact_map, scrub_string};
use chrono::{SecondsFormat, Utc};
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};

/// The schema string stamped onto every record.
///
/// **External contract** — upstream `LOG_SCHEMA = 'qwaudio.log/v1'`
/// (`shared/logger.mjs:13`), renamed per `docs/rebrand.md`
/// (`qwaudio.log/v1` → `via.log/v1`).
pub const LOG_SCHEMA: &str = "via.log/v1";

/// The seven spine keys, in wire order.
///
/// **External contract** — upstream's `reserved` set for the console line
/// (`shared/logger.mjs:226-228`) and the spine literal
/// (`shared/logger.mjs:293-304`).
pub const RESERVED_FIELDS: [&str; 7] = [
    "schema",
    "time",
    "level",
    "component",
    "event",
    "pid",
    "message",
];

/// Event name used when a call site supplies none.
///
/// **External contract** — upstream `safeEventName`
/// (`shared/logger.mjs:112-115`).
pub const DEFAULT_EVENT: &str = "log";

/// Component name used when a logger is built without one.
///
/// **External contract** — upstream `String(component || 'app')`
/// (`shared/logger.mjs:300`).
pub const DEFAULT_COMPONENT: &str = "app";

/// The Gateway logger's component name.
///
/// **External contract** — upstream `server/src/core/logger.mjs:12`.
pub const GATEWAY_COMPONENT: &str = "gateway";

/// The Gateway logger's file name.
///
/// **External contract** — upstream `server/src/core/logger.mjs:13`.
pub const GATEWAY_LOG_FILE_NAME: &str = "gateway.log";

/// Apply upstream's event-name fallback.
///
/// **External contract** — upstream `safeEventName`
/// (`shared/logger.mjs:112-115`).
#[must_use]
pub fn safe_event_name(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        DEFAULT_EVENT.to_owned()
    } else {
        trimmed.to_owned()
    }
}

/// One `via.log/v1` record.
///
/// Field declaration order below is the wire order of the spine;
/// [`LogRecord::to_map`] is the authority and reproduces it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogRecord {
    /// Merged `base` + context + caller fields, in insertion order. Written
    /// *before* the spine.
    pub extra: Map<String, Value>,
    /// `schema` — always [`LOG_SCHEMA`] for records this crate produces.
    pub schema: String,
    /// `time` — ISO-8601 with millisecond precision and a `Z` suffix, matching
    /// JavaScript's `Date#toISOString`.
    pub time: String,
    /// `level` — serialized as its lowercase name.
    pub level: LogLevel,
    /// `component` — the logger's name.
    pub component: String,
    /// `event` — the dotted event name.
    pub event: String,
    /// `pid` — the emitting process id.
    pub pid: u32,
    /// `message` — omitted when the call site passed none.
    pub message: Option<String>,
}

impl LogRecord {
    /// Assemble a record from already-redacted parts.
    ///
    /// `extra` is taken as-is; use [`LogRecord::redacted`] (or go through
    /// [`crate::Logger`]) if it has not been through [`redact_map`] yet.
    ///
    /// Applies the three normalisations upstream performs at the same point:
    /// [`safe_event_name`], the [`DEFAULT_COMPONENT`] fallback, and dropping an
    /// empty message.
    #[must_use]
    pub fn new(
        level: LogLevel,
        component: &str,
        event: &str,
        extra: Map<String, Value>,
        message: &str,
    ) -> Self {
        Self {
            extra,
            schema: LOG_SCHEMA.to_owned(),
            time: now_iso8601(),
            level,
            component: if component.is_empty() {
                DEFAULT_COMPONENT.to_owned()
            } else {
                component.to_owned()
            },
            event: safe_event_name(event),
            pid: std::process::id(),
            message: if message.is_empty() {
                None
            } else {
                Some(message.to_owned())
            },
        }
    }

    /// Return a copy with `extra` and `message` run through redaction.
    ///
    /// Redaction is idempotent, so calling this on a record whose parts were
    /// already redacted is a no-op beyond the allocation.
    #[must_use]
    pub fn redacted(&self) -> Self {
        Self {
            extra: redact_map(&self.extra),
            schema: self.schema.clone(),
            time: self.time.clone(),
            level: self.level,
            component: self.component.clone(),
            event: self.event.clone(),
            pid: self.pid,
            message: self.message.as_deref().map(scrub_string),
        }
    }

    /// Render the record as an ordered JSON object.
    ///
    /// **External contract** — the spine is inserted last, so a colliding
    /// caller field keeps its position but loses its value, exactly as
    /// JavaScript object spread does.
    #[must_use]
    pub fn to_map(&self) -> Map<String, Value> {
        let mut map = self.extra.clone();
        map.insert("schema".to_owned(), Value::String(self.schema.clone()));
        map.insert("time".to_owned(), Value::String(self.time.clone()));
        map.insert(
            "level".to_owned(),
            Value::String(self.level.as_str().to_owned()),
        );
        map.insert(
            "component".to_owned(),
            Value::String(self.component.clone()),
        );
        map.insert("event".to_owned(), Value::String(self.event.clone()));
        map.insert("pid".to_owned(), Value::Number(self.pid.into()));
        // Upstream spreads `message` only when the caller supplied one, which
        // is what lets a `message` field in `fields` survive.
        if let Some(message) = &self.message {
            map.insert("message".to_owned(), Value::String(message.clone()));
        }
        map
    }

    /// The JSON-lines representation, newline included.
    #[must_use]
    pub fn to_line(&self) -> String {
        let map = self.to_map();
        // A `Map<String, Value>` always serializes: every key is a string and
        // `Value::Number` cannot hold NaN or infinity, so serde_json's only
        // documented failure modes are unreachable. Degrading to an empty line
        // rather than panicking keeps the rule that logging never interrupts
        // the application.
        let mut line = serde_json::to_string(&map).unwrap_or_default();
        line.push('\n');
        line
    }

    /// The human-readable console representation, newline included.
    ///
    /// **External contract** — upstream `consoleLine`
    /// (`shared/logger.mjs:225-237`):
    /// `` `${time} ${LEVEL} ${component} ${event}${message ? ': ' + message : ''}${details}` ``
    /// where `details` is the record minus the seven [`RESERVED_FIELDS`],
    /// serialized as JSON and prefixed with a space.
    #[must_use]
    pub fn to_console_line(&self) -> String {
        let map = self.to_map();
        let details: Map<String, Value> = map
            .iter()
            .filter(|(key, _)| !RESERVED_FIELDS.contains(&key.as_str()))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        let suffix = if details.is_empty() {
            String::new()
        } else {
            // Same unreachable-failure reasoning as `to_line`.
            format!(" {}", serde_json::to_string(&details).unwrap_or_default())
        };
        // Upstream reads `record.message`, which is the merged value — a
        // `message` field survives when the call site passed none.
        let message = map
            .get("message")
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
            .map(|text| format!(": {text}"))
            .unwrap_or_default();
        format!(
            "{} {} {} {}{}{}\n",
            self.time,
            self.level.as_str().to_uppercase(),
            self.component,
            self.event,
            message,
            suffix
        )
    }
}

impl Serialize for LogRecord {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_map().serialize(serializer)
    }
}

/// The current time in the exact shape JavaScript's `Date#toISOString`
/// produces: UTC, millisecond precision, `Z` suffix.
#[must_use]
pub fn now_iso8601() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn map(pairs: &[(&str, Value)]) -> Map<String, Value> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), value.clone()))
            .collect()
    }

    #[test]
    fn schema_is_the_rebranded_contract_value() {
        assert_eq!(LOG_SCHEMA, "via.log/v1");
    }

    #[test]
    fn spine_is_written_in_contract_order() {
        let record = LogRecord::new(
            LogLevel::Info,
            "gateway",
            "realtime.connected",
            map(&[("provider", json!("dashscope"))]),
            "Realtime ready",
        );
        let keys: Vec<String> = record.to_map().keys().cloned().collect();
        assert_eq!(
            keys,
            [
                "provider",
                "schema",
                "time",
                "level",
                "component",
                "event",
                "pid",
                "message"
            ]
        );
    }

    #[test]
    fn empty_event_falls_back_to_log() {
        assert_eq!(safe_event_name("  "), "log");
        assert_eq!(
            safe_event_name(" realtime.connected "),
            "realtime.connected"
        );
    }

    #[test]
    fn time_matches_javascript_to_iso_string_shape() {
        let time = now_iso8601();
        assert!(time.ends_with('Z'), "{time}");
        assert_eq!(time.len(), 24, "{time}");
        assert_eq!(&time[4..5], "-");
        assert_eq!(&time[10..11], "T");
    }
}
