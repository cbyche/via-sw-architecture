//! The GA (2025+) OpenAI Realtime dialect.
//!
//! Ported from `server/src/voice/providers/ga-protocol.mjs`. Upstream's own
//! summary of what differs from the beta dialect, and the whole of what this
//! file exists for:
//!
//! > - response payloads use `output_modalities` instead of `modalities`;
//! > - text deltas arrive as `response.output_text.*` instead of
//! >   `response.text.*`;
//! > - conversation item ids are namespaced per item type.
//!
//! Each of the three is a hard failure if it is missed rather than a cosmetic
//! difference:
//!
//! - a `modalities` key on a GA `response.create` is an unknown field;
//! - un-rewritten `response.output_text.*` events never reach the transcript
//!   path, so a GA provider looks mute;
//! - the GA server rejects an id from the wrong namespace outright — the
//!   catalogue quotes the error, *"ID must start with 'fco_'"*.
//!
//! It also carries the one thing the beta dialect cannot do: response metadata
//! that is echoed back on `response.created` and `response.done`, which is what
//! lets the session tell a response it asked for from an automatic server-VAD
//! turn on the same session.

use serde_json::{Map, Value};

use super::{
    RealtimeProtocol, audio_append_frame, function_output_item_value, hyphenless_uuid,
    item_create_frame, response_cancel_frame, session_update_frame, user_text_item_value,
    with_event_id,
};

/// The metadata key a GA response carries the correlation id in.
///
/// External contract — `ga-protocol.mjs:25`, echoed by the service on
/// `response.created` and `response.done`. It names the upstream product and is
/// **KEEP** under `docs/rebrand.md`: renaming it would silently stop correlating
/// against a service that echoes the old key, and the key travels to a
/// third-party endpoint VIA does not control.
pub const RESPONSE_CORRELATION_KEY: &str = "qwen_audio_request_id";

/// The id prefix each item type is minted under.
///
/// External contract — `ga-protocol.mjs:19-23`. An item type that is not listed
/// keeps the generic `item` namespace.
pub const ID_PREFIXES: [(&str, &str); 3] = [
    ("message", "msg"),
    ("function_call", "fc"),
    ("function_call_output", "fco"),
];

/// The namespace for an unlisted item type.
pub const DEFAULT_ID_PREFIX: &str = "item";

/// The GA dialect adapter.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GaRealtimeProtocol;

/// The GA dialect adapter.
#[must_use]
pub const fn ga_realtime_protocol() -> GaRealtimeProtocol {
    GaRealtimeProtocol
}

/// Rewrite a response body's `modalities` into GA's `output_modalities`.
///
/// The rewrite happens here, once, rather than in every call site: shared
/// frontend code — `buildSpeakResponse`, `buildResultInjection`, the `modalities`
/// option on `sendUserText` — still passes the beta field name, and upstream is
/// explicit that this is the reason (`ga-protocol.mjs:7-9`).
///
/// The key is **moved to the end**, matching the JavaScript
/// `const { modalities, ...rest } = response; return { ...rest, output_modalities }`.
fn ga_response(response: Value) -> Value {
    let Value::Object(mut fields) = response else {
        return response;
    };
    if let Some(modalities) = fields.shift_remove("modalities") {
        fields.insert("output_modalities".to_owned(), modalities);
    }
    Value::Object(fields)
}

impl RealtimeProtocol for GaRealtimeProtocol {
    fn encode_outgoing(&self, payload: Value) -> Value {
        with_event_id(payload)
    }

    fn normalize_incoming(&self, event: Value) -> Vec<Value> {
        if event.is_null() {
            return Vec::new();
        }
        let renamed = match event.get("type").and_then(Value::as_str) {
            Some("response.output_text.delta") => Some("response.text.delta"),
            Some("response.output_text.done") => Some("response.text.done"),
            _ => None,
        };
        let Some(renamed) = renamed else {
            return vec![event];
        };
        let mut event = event;
        if let Value::Object(fields) = &mut event {
            // `{ ...event, type }` replaces the value in place, so the key order
            // of every other field is untouched.
            fields.insert("type".to_owned(), Value::String(renamed.to_owned()));
        }
        vec![event]
    }

    fn session_update(&self, session: Value) -> Value {
        session_update_frame(session)
    }

    fn audio_append(&self, audio: &str) -> Value {
        audio_append_frame(audio)
    }

    fn conversation_item_id(&self, item: &Value) -> String {
        let kind = item.get("type").and_then(Value::as_str).unwrap_or_default();
        let prefix = ID_PREFIXES
            .iter()
            .find(|(name, _)| *name == kind)
            .map_or(DEFAULT_ID_PREFIX, |(_, prefix)| *prefix);
        format!("{prefix}_{}", hyphenless_uuid())
    }

    fn conversation_item_create(&self, item: Value) -> Value {
        item_create_frame(item)
    }

    fn response_create(&self, response: Option<Value>) -> Value {
        let mut frame = Map::new();
        frame.insert(
            "type".to_owned(),
            Value::String("response.create".to_owned()),
        );
        if let Some(body) = response {
            frame.insert("response".to_owned(), ga_response(body));
        }
        Value::Object(frame)
    }

    fn correlate_response_create(&self, mut payload: Value, request_id: &str) -> Value {
        // Written as in-place mutation rather than rebuild-and-replace because
        // the JavaScript spread `{ ...payload, response: { ... } }` keeps every
        // key where it was; removing and re-inserting `response` would move it
        // to the end, and the frame is compared byte for byte in the retry path.
        if let Some(fields) = payload.as_object_mut() {
            let response = fields
                .entry("response")
                .or_insert_with(|| Value::Object(Map::new()));
            if !response.is_object() {
                *response = Value::Object(Map::new());
            }
            if let Some(response) = response.as_object_mut() {
                let metadata = response
                    .entry("metadata")
                    .or_insert_with(|| Value::Object(Map::new()));
                if !metadata.is_object() {
                    *metadata = Value::Object(Map::new());
                }
                if let Some(metadata) = metadata.as_object_mut() {
                    metadata.insert(
                        RESPONSE_CORRELATION_KEY.to_owned(),
                        Value::String(request_id.to_owned()),
                    );
                }
            }
        }
        payload
    }

    fn response_correlation_id(&self, event: &Value) -> String {
        event
            .get("response")
            .and_then(|response| response.get("metadata"))
            .and_then(|metadata| metadata.get(RESPONSE_CORRELATION_KEY))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    }

    fn response_cancel(&self) -> Value {
        response_cancel_frame()
    }

    fn user_text_item(&self, text: &str) -> Value {
        user_text_item_value(text)
    }

    fn function_output_item(&self, call_id: &str, output: &Value) -> Value {
        function_output_item_value(call_id, output)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    fn protocol() -> GaRealtimeProtocol {
        ga_realtime_protocol()
    }

    #[test]
    fn modalities_becomes_output_modalities() {
        let frame = protocol().response_create(Some(json!({
            "modalities": ["text", "audio"],
            "tool_choice": "none",
        })));
        assert_eq!(
            frame["response"],
            json!({ "tool_choice": "none", "output_modalities": ["text", "audio"] })
        );
        assert!(
            !frame["response"]
                .as_object()
                .expect("object")
                .contains_key("modalities")
        );
    }

    #[test]
    fn a_body_without_modalities_is_untouched() {
        assert_eq!(
            protocol().response_create(Some(json!({ "instructions": "say it" }))),
            json!({ "type": "response.create", "response": { "instructions": "say it" } })
        );
    }

    #[test]
    fn a_response_create_with_no_body_omits_the_key() {
        assert_eq!(
            protocol().response_create(None),
            json!({ "type": "response.create" })
        );
    }

    #[test]
    fn item_ids_are_namespaced_by_item_type() {
        let protocol = protocol();
        for (kind, prefix) in [
            ("message", "msg_"),
            ("function_call", "fc_"),
            ("function_call_output", "fco_"),
            ("audio", "item_"),
            ("", "item_"),
        ] {
            let id = protocol.conversation_item_id(&json!({ "type": kind }));
            let hex = id
                .strip_prefix(prefix)
                .unwrap_or_else(|| panic!("{kind} -> {id}"));
            assert_eq!(hex.len(), 32, "{id}");
            assert!(hex.chars().all(|c| c.is_ascii_hexdigit()), "{id}");
        }
    }

    #[test]
    fn an_item_with_no_type_gets_the_generic_namespace() {
        assert!(
            protocol()
                .conversation_item_id(&json!({}))
                .starts_with("item_")
        );
    }

    #[test]
    fn ga_text_events_are_renamed_to_the_shared_names() {
        let protocol = protocol();
        let events = protocol.normalize_incoming(json!({
            "type": "response.output_text.delta",
            "delta": "你好",
        }));
        assert_eq!(
            events,
            vec![json!({ "type": "response.text.delta", "delta": "你好" })]
        );

        let done = protocol.normalize_incoming(json!({
            "type": "response.output_text.done",
            "text": "你好",
        }));
        assert_eq!(
            done,
            vec![json!({ "type": "response.text.done", "text": "你好" })]
        );
    }

    #[test]
    fn standard_input_events_pass_through_unchanged() {
        let event = json!({
            "type": "input_audio_buffer.speech_started",
            "event_id": "event-input-1",
            "item_id": "item-input-1",
        });
        assert_eq!(protocol().normalize_incoming(event.clone()), vec![event]);
    }

    #[test]
    fn the_audio_deltas_are_not_renamed() {
        // Only the two *text* names are rewritten; `response.output_audio.*` is
        // in the activity table under both spellings instead.
        let event = json!({ "type": "response.output_audio.delta", "response_id": "r" });
        assert_eq!(protocol().normalize_incoming(event.clone()), vec![event]);
    }

    #[test]
    fn correlation_metadata_is_added_without_losing_the_body() {
        let protocol = protocol();
        let payload = protocol.response_create(Some(json!({
            "modalities": ["audio"],
            "tool_choice": "none",
        })));
        let correlated = protocol.correlate_response_create(payload, "request-1");
        assert_eq!(correlated["response"]["tool_choice"], json!("none"));
        assert_eq!(
            correlated["response"]["output_modalities"],
            json!(["audio"])
        );
        assert_eq!(
            correlated["response"]["metadata"][RESPONSE_CORRELATION_KEY],
            json!("request-1")
        );
        assert_eq!(protocol.response_correlation_id(&correlated), "request-1");
        // The frame keeps its key order: `type` still leads.
        let keys: Vec<&str> = correlated
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["type", "response"]);
    }

    #[test]
    fn correlation_preserves_metadata_the_caller_already_set() {
        let correlated = protocol().correlate_response_create(
            json!({
                "type": "response.create",
                "response": { "metadata": { "trace": "abc" } },
            }),
            "request-2",
        );
        assert_eq!(correlated["response"]["metadata"]["trace"], json!("abc"));
        assert_eq!(
            correlated["response"]["metadata"][RESPONSE_CORRELATION_KEY],
            json!("request-2")
        );
    }

    #[test]
    fn correlation_creates_the_response_body_when_there_is_none() {
        let correlated =
            protocol().correlate_response_create(json!({ "type": "response.create" }), "request-3");
        assert_eq!(
            correlated,
            json!({
                "type": "response.create",
                "response": { "metadata": { RESPONSE_CORRELATION_KEY: "request-3" } },
            })
        );
    }

    #[test]
    fn a_correlation_id_that_is_absent_or_not_a_string_reads_as_empty() {
        let protocol = protocol();
        assert_eq!(protocol.response_correlation_id(&json!({})), "");
        assert_eq!(
            protocol.response_correlation_id(&json!({ "response": { "id": "r" } })),
            ""
        );
        assert_eq!(
            protocol.response_correlation_id(&json!({
                "response": { "metadata": { RESPONSE_CORRELATION_KEY: 7 } }
            })),
            ""
        );
    }

    #[test]
    fn the_correlation_key_is_the_catalogued_one() {
        assert_eq!(RESPONSE_CORRELATION_KEY, "qwen_audio_request_id");
    }
}
