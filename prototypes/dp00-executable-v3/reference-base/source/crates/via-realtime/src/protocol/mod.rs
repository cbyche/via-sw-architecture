//! The wire dialect.
//!
//! Upstream splits a realtime frontend in two: `provider` answers *which
//! service* — the endpoint, the credential, the model, the session payload —
//! and `protocol` answers *which dialect* the bytes are in. The split is what
//! lets `providers/s2s.mjs` and a future local model share one adapter, and it
//! is why the Gateway never sees a wire format at all: it consumes only what
//! [`RealtimeProtocol::normalize_incoming`] returns.
//!
//! Two dialects ship:
//!
//! | Module | Dialect | Who speaks it |
//! | --- | --- | --- |
//! | [`openai_compatible`] | the beta OpenAI Realtime envelope | DashScope |
//! | [`ga`] | the GA (2025+) OpenAI Realtime envelope | huggingface/speech-to-speech |
//!
//! Upstream validates a protocol object against a closed list of twelve method
//! names at registration time (`providers/provider-registry.mjs:14-27`,
//! `PROTOCOL_METHODS`), because a JavaScript object literal can silently be
//! missing one. In Rust the trait is that list, so the check is the compiler's;
//! [`PROTOCOL_METHODS`] keeps the names for `via-conformance`.

pub mod ga;
pub mod openai_compatible;

use serde_json::{Map, Value};

pub use ga::{GaRealtimeProtocol, RESPONSE_CORRELATION_KEY, ga_realtime_protocol};
pub use openai_compatible::{OpenAiCompatibleProtocol, openai_compatible_protocol};

/// The twelve method names upstream validates a protocol object against.
///
/// External contract — `providers/provider-registry.mjs:14-27`. Two of them are
/// test-locked by name (`/encodeOutgoing/`), and `connectionMessages` is
/// deliberately **not** in the list: it is optional, and upstream checks only
/// that it is a function when present.
pub const PROTOCOL_METHODS: [&str; 12] = [
    "encodeOutgoing",
    "normalizeIncoming",
    "sessionUpdate",
    "audioAppend",
    "conversationItemId",
    "conversationItemCreate",
    "responseCreate",
    "correlateResponseCreate",
    "responseCorrelationId",
    "responseCancel",
    "userTextItem",
    "functionOutputItem",
];

/// One realtime wire dialect.
///
/// Every method takes and returns owned [`Value`]s because every one of them is
/// a frame in flight: the session builds a payload, hands it to the dialect, and
/// writes whatever comes back. Nothing here holds state that outlives a frame —
/// [`RealtimeProvider::create_protocol`](crate::RealtimeProvider::create_protocol)
/// exists for a dialect that needs per-connection state, and it hands back a
/// fresh adapter rather than mutating a shared one.
pub trait RealtimeProtocol: Send + Sync + core::fmt::Debug {
    /// Wrap an outgoing payload in this dialect's envelope.
    ///
    /// Both shipped dialects prepend `event_id: "event_<32 hex>"`. The insertion
    /// order matters: upstream writes `{ event_id: eventId(), ...payload }`, so
    /// a payload that carries its own `event_id` **overwrites** the generated
    /// one while keeping the leading position.
    fn encode_outgoing(&self, payload: Value) -> Value;

    /// Turn one provider frame into zero, one or many normalized events.
    ///
    /// Upstream returns `event | event[] | null` and the caller filters falsy
    /// values (`realtime-provider.mjs:46-49`), so the vector is the whole of
    /// that contract: an empty vector is "drop this frame".
    fn normalize_incoming(&self, event: Value) -> Vec<Value>;

    /// The `session.update` frame for a session payload.
    fn session_update(&self, session: Value) -> Value;

    /// The frame that appends a chunk of base64 PCM to the input buffer.
    fn audio_append(&self, audio: &str) -> Value;

    /// The frame that closes the current input audio buffer.
    ///
    /// **Not an upstream method.** Upstream never commits explicitly: both of
    /// its providers run server-side turn detection, so the service commits for
    /// it. `dictation` sessions (`docs/architecture.md` §2) and any client that
    /// drives push-to-talk need the explicit form, and both dialects spell it
    /// the same way, so the default implementation is the answer for every
    /// dialect that follows the OpenAI Realtime envelope.
    fn audio_commit(&self) -> Value {
        Value::Object(Map::from_iter([(
            "type".to_owned(),
            Value::String("input_audio_buffer.commit".to_owned()),
        )]))
    }

    /// Mint a client-assigned conversation item id for this item.
    ///
    /// Id namespaces are dialect-specific — the GA dialect derives them from the
    /// item's `type` and rejects an id from the wrong namespace outright — which
    /// is exactly why the session asks the adapter instead of minting one
    /// itself.
    fn conversation_item_id(&self, item: &Value) -> String;

    /// The `conversation.item.create` frame for an item.
    fn conversation_item_create(&self, item: Value) -> Value;

    /// The `response.create` frame, with an optional response body.
    ///
    /// `None` means "no `response` key at all", which is not the same as an
    /// empty object: an empty object is a body the provider will validate.
    fn response_create(&self, response: Option<Value>) -> Value;

    /// Stamp a correlation id onto a `response.create` payload.
    ///
    /// A dialect with no portable metadata contract returns the payload
    /// unchanged, and the session falls back to FIFO correlation.
    fn correlate_response_create(&self, payload: Value, request_id: &str) -> Value;

    /// Read back the correlation id a provider echoed on `response.created` /
    /// `response.done`, or `""` when this dialect has none.
    fn response_correlation_id(&self, event: &Value) -> String;

    /// The `response.cancel` frame.
    fn response_cancel(&self) -> Value;

    /// A user message item carrying one block of text.
    fn user_text_item(&self, text: &str) -> Value;

    /// A tool result item.
    ///
    /// `output` is always a **JSON-encoded string**, never a nested object —
    /// catalogued at `json-field` / *function call handling over the realtime
    /// protocol*.
    fn function_output_item(&self, call_id: &str, output: &Value) -> Value;

    /// Frames to write the moment the socket opens, before anything else.
    ///
    /// Upstream's optional `connectionMessages`. Empty by default; a dialect
    /// that needs a route handshake (a multiplexing proxy, a local pipeline
    /// selecting a voice pack) fills it in.
    fn connection_messages(&self) -> Vec<Value> {
        Vec::new()
    }
}

/// `event_${randomUUID().replaceAll('-', '')}`.
///
/// External contract — `openai-compatible-protocol.mjs:3-5` and
/// `ga-protocol.mjs:3-5`, test-locked as `/^event_[a-f0-9]+$/`.
#[must_use]
pub fn event_id() -> String {
    format!("event_{}", hyphenless_uuid())
}

/// A v4 UUID with the hyphens removed: 32 lowercase hex characters.
#[must_use]
pub fn hyphenless_uuid() -> String {
    let mut buffer = uuid::Uuid::encode_buffer();
    uuid::Uuid::new_v4()
        .simple()
        .encode_lower(&mut buffer)
        .to_owned()
}

/// `{ event_id: <generated>, ...payload }` — the envelope both dialects share.
///
/// Kept here rather than duplicated because the *ordering* rule is subtle: the
/// generated id goes in first so it leads the serialized object, and a payload
/// key of the same name replaces its value without moving it, which is exactly
/// what the JavaScript spread does.
fn with_event_id(payload: Value) -> Value {
    let mut envelope = Map::new();
    envelope.insert("event_id".to_owned(), Value::String(event_id()));
    match payload {
        Value::Object(fields) => {
            for (key, value) in fields {
                envelope.insert(key, value);
            }
        }
        // A non-object payload cannot be spread in JavaScript either — `{...5}`
        // is `{}` — so it contributes nothing but its own absence.
        other => {
            if !other.is_null() {
                envelope.insert("payload".to_owned(), other);
            }
        }
    }
    Value::Object(envelope)
}

/// `{ type, item }` — shared by both dialects.
fn item_create_frame(item: Value) -> Value {
    Value::Object(Map::from_iter([
        (
            "type".to_owned(),
            Value::String("conversation.item.create".to_owned()),
        ),
        ("item".to_owned(), item),
    ]))
}

/// `{ type: 'message', role: 'user', content: [{ type: 'input_text', text }] }`.
///
/// Identical in both dialects, and the shape is contract three times over: it is
/// what `sendUserText` writes, what a restored-context injection is, and what a
/// result injection's item is.
fn user_text_item_value(text: &str) -> Value {
    Value::Object(Map::from_iter([
        ("type".to_owned(), Value::String("message".to_owned())),
        ("role".to_owned(), Value::String("user".to_owned())),
        (
            "content".to_owned(),
            Value::Array(vec![Value::Object(Map::from_iter([
                ("type".to_owned(), Value::String("input_text".to_owned())),
                ("text".to_owned(), Value::String(text.to_owned())),
            ]))]),
        ),
    ]))
}

/// `{ type: 'function_call_output', call_id, output: JSON.stringify(output) }`.
fn function_output_item_value(call_id: &str, output: &Value) -> Value {
    Value::Object(Map::from_iter([
        (
            "type".to_owned(),
            Value::String("function_call_output".to_owned()),
        ),
        ("call_id".to_owned(), Value::String(call_id.to_owned())),
        (
            "output".to_owned(),
            // `serde_json::to_string` and `JSON.stringify` agree on everything a
            // tool result can hold. The empty-string arm is unreachable for a
            // `Value` — there is no non-finite float and no failing custom
            // `Serialize` in one — and is preferred to an `expect()`.
            Value::String(serde_json::to_string(output).unwrap_or_default()),
        ),
    ]))
}

/// `{ type: 'response.cancel' }`, shared by both dialects.
fn response_cancel_frame() -> Value {
    Value::Object(Map::from_iter([(
        "type".to_owned(),
        Value::String("response.cancel".to_owned()),
    )]))
}

/// `{ type: 'input_audio_buffer.append', audio }`, shared by both dialects.
fn audio_append_frame(audio: &str) -> Value {
    Value::Object(Map::from_iter([
        (
            "type".to_owned(),
            Value::String("input_audio_buffer.append".to_owned()),
        ),
        ("audio".to_owned(), Value::String(audio.to_owned())),
    ]))
}

/// `{ type: 'session.update', session }`, shared by both dialects.
fn session_update_frame(session: Value) -> Value {
    Value::Object(Map::from_iter([
        (
            "type".to_owned(),
            Value::String("session.update".to_owned()),
        ),
        ("session".to_owned(), session),
    ]))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn an_event_id_is_the_catalogued_shape() {
        let id = event_id();
        let hex = id.strip_prefix("event_").expect("event_ prefix");
        assert_eq!(hex.len(), 32);
        assert!(
            hex.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
    }

    #[test]
    fn event_ids_do_not_repeat() {
        let first = event_id();
        assert_ne!(first, event_id());
    }

    #[test]
    fn the_envelope_puts_event_id_first() {
        let frame = with_event_id(json!({ "type": "session.update", "session": {} }));
        let keys: Vec<&str> = frame
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["event_id", "type", "session"]);
    }

    #[test]
    fn a_payload_event_id_overwrites_the_generated_one_in_place() {
        let frame = with_event_id(json!({ "event_id": "mine", "type": "x" }));
        assert_eq!(frame["event_id"], json!("mine"));
        let keys: Vec<&str> = frame
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["event_id", "type"]);
    }

    #[test]
    fn a_tool_result_is_encoded_as_a_string_never_as_an_object() {
        let item = function_output_item_value("call-1", &json!({ "status": "ok", "n": 1 }));
        assert_eq!(item["output"], json!(r#"{"status":"ok","n":1}"#));
        assert!(item["output"].is_string());
    }

    #[test]
    fn the_protocol_method_list_has_no_duplicates() {
        let mut sorted = PROTOCOL_METHODS.to_vec();
        sorted.sort_unstable();
        let mut unique = sorted.clone();
        unique.dedup();
        assert_eq!(sorted, unique);
    }
}
