//! The two shapes every tool result comes back in.
//!
//! Ported from upstream `server/src/agent/acp-session-tools.mjs:18-30`:
//!
//! ```js
//! function jsonResult(value, isError = false) {
//!   return {
//!     content: [{ type: 'text', text: JSON.stringify(value) }],
//!     ...(isError ? { isError: true } : {}),
//!   }
//! }
//!
//! function errorResult(error) {
//!   return jsonResult({ status: 'failed', error: error?.message || String(error) }, true)
//! }
//! ```
//!
//! # Why this is not just serialization
//!
//! The payload is **a JSON string inside a text block**, not a JSON object in
//! the result. The model receives one string and parses it, so
//! `JSON.stringify`'s compactness — no indentation, no spaces after separators
//! — and its key order are both observable. `serde_json::to_string` is compact
//! by the same definition, and the workspace's `preserve_order` keeps the key
//! order the payload type declared.
//!
//! `isError` is **absent** on success rather than `false`. A backend that
//! branches on `'isError' in result` and one that branches on
//! `result.isError === true` must both see what upstream shows them.

use serde::Serialize;
use serde_json::{Value, json};

/// The `status` literal a failed tool call reports.
///
/// **External contract** — `acp-session-tools.mjs:27`, catalogued under
/// `error-code / session tool error envelope`.
pub const STATUS_FAILED: &str = "failed";

/// The content-block type every result uses.
///
/// **External contract** — `acp-session-tools.mjs:20`. One block, always
/// `text`.
pub const CONTENT_TYPE_TEXT: &str = "text";

/// Wrap an already-serialized payload string as a successful tool result.
///
/// Kept separate from [`json_result`] so a caller that already has the exact
/// bytes — a passthrough, a test fixture — does not have to round-trip them
/// through a `Value` and risk reordering.
#[must_use]
pub fn text_result(text: String) -> Value {
    json!({ "content": [{ "type": CONTENT_TYPE_TEXT, "text": text }] })
}

/// `jsonResult(value)` — the success envelope.
///
/// The payload is serialized compactly and placed in a single text block.
/// Serialization cannot fail for the result types this crate defines; if a
/// caller's own type does fail, the envelope degrades to
/// [`error_result`] rather than losing the call, because the model is owed an
/// answer either way.
#[must_use]
pub fn json_result<T: Serialize>(value: &T) -> Value {
    match serde_json::to_string(value) {
        Ok(text) => text_result(text),
        Err(error) => error_result(&error.to_string()),
    }
}

/// `errorResult(error)` — the failure envelope.
///
/// `{"content":[{"type":"text","text":"{\"status\":\"failed\",\"error\":\"…\"}"}],"isError":true}`.
/// The message is the caller's already-localized sentence: upstream's
/// `error?.message || String(error)`, which for VIA is
/// [`HarnessError::message`](via_downstream::HarnessError::message).
#[must_use]
pub fn error_result(message: &str) -> Value {
    let payload = json!({ "status": STATUS_FAILED, "error": message });
    let text = serde_json::to_string(&payload).unwrap_or_else(|_| {
        // Unreachable: `payload` is two string fields. Falling back to the
        // literal keeps the model's parse working rather than returning a
        // shape it has never been shown.
        format!(r#"{{"status":"{STATUS_FAILED}","error":""}}"#)
    });
    let mut envelope = text_result(text);
    if let Some(object) = envelope.as_object_mut() {
        object.insert("isError".to_owned(), Value::Bool(true));
    }
    envelope
}

/// The text a result envelope carries, if it is shaped like one.
///
/// The inverse of [`text_result`], for callers — the stdio transport's tests,
/// a coordinator asserting what the model was shown — that need to read a
/// result back without restating its shape.
#[must_use]
pub fn envelope_text(envelope: &Value) -> Option<&str> {
    envelope.get("content")?.get(0)?.get("text")?.as_str()
}

/// Whether an envelope is the failure shape.
#[must_use]
pub fn is_error_envelope(envelope: &Value) -> bool {
    envelope.get("isError") == Some(&Value::Bool(true))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn success_carries_one_compact_text_block_and_no_is_error() {
        let envelope = json_result(&json!({ "sessions": [{ "session_id": "one" }] }));
        assert_eq!(
            serde_json::to_string(&envelope).expect("json"),
            r#"{"content":[{"type":"text","text":"{\"sessions\":[{\"session_id\":\"one\"}]}"}]}"#,
        );
        assert!(envelope.get("isError").is_none());
        assert!(!is_error_envelope(&envelope));
    }

    #[test]
    fn failure_is_the_status_failed_payload_with_is_error_true() {
        let envelope = error_result("the backend refused");
        assert_eq!(
            serde_json::to_string(&envelope).expect("json"),
            r#"{"content":[{"type":"text","text":"{\"status\":\"failed\",\"error\":\"the backend refused\"}"}],"isError":true}"#,
        );
        assert!(is_error_envelope(&envelope));
    }

    #[test]
    fn a_hostile_message_cannot_break_out_of_the_payload() {
        let envelope = error_result(r#"" , "isError": false, "x": ""#);
        let text = envelope_text(&envelope).expect("a text block");
        let parsed: Value = serde_json::from_str(text).expect("still one object");
        assert_eq!(parsed["status"], STATUS_FAILED);
        assert_eq!(parsed["error"], r#"" , "isError": false, "x": ""#);
        assert_eq!(parsed.as_object().expect("object").len(), 2);
        assert!(is_error_envelope(&envelope));
    }

    #[test]
    fn the_payload_key_order_is_status_then_error() {
        let envelope = error_result("x");
        let text = envelope_text(&envelope).expect("a text block");
        assert!(text.starts_with(r#"{"status":"#), "{text}");
    }

    #[test]
    fn envelope_text_declines_a_foreign_shape() {
        assert_eq!(envelope_text(&json!({})), None);
        assert_eq!(envelope_text(&json!({ "content": [] })), None);
        assert_eq!(
            envelope_text(&json!({ "content": [{ "type": "image" }] })),
            None
        );
    }
}
