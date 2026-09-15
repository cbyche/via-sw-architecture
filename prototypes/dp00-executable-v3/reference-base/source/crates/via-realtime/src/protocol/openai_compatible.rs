//! The beta OpenAI Realtime dialect.
//!
//! Ported from `server/src/voice/providers/openai-compatible-protocol.mjs`.
//! Upstream's own framing of what this file is for:
//!
//! > Wire adapter for Realtime providers that use the OpenAI-compatible event
//! > envelope. Gateway code only consumes the normalized events returned by
//! > `normalizeIncoming()`, so a provider with a different protocol can
//! > implement the same adapter surface without leaking its wire format into the
//! > Gateway.
//!
//! Two of its answers are the interesting ones, and both are deliberate
//! *absences*:
//!
//! - **`normalize_incoming` is the identity.** The beta names already are the
//!   normalized names, so there is nothing to rewrite.
//! - **`correlate_response_create` is the identity too.** The beta dialect has
//!   no portable response-metadata contract, so the session keeps its FIFO
//!   correlation — which is why `dashscope` leaves
//!   [`response_metadata_correlation`](crate::ProviderCapabilities::response_metadata_correlation)
//!   at the baseline `false`.

use serde_json::Value;

use super::{
    RealtimeProtocol, audio_append_frame, function_output_item_value, hyphenless_uuid,
    item_create_frame, response_cancel_frame, session_update_frame, user_text_item_value,
    with_event_id,
};

/// The beta dialect adapter.
///
/// Stateless: [`openai_compatible_protocol`] hands back the same value every
/// time, and a provider can hold it by value.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct OpenAiCompatibleProtocol;

/// The beta dialect adapter.
#[must_use]
pub const fn openai_compatible_protocol() -> OpenAiCompatibleProtocol {
    OpenAiCompatibleProtocol
}

impl RealtimeProtocol for OpenAiCompatibleProtocol {
    fn encode_outgoing(&self, payload: Value) -> Value {
        with_event_id(payload)
    }

    fn normalize_incoming(&self, event: Value) -> Vec<Value> {
        // Upstream `normalizeIncoming: event => event`, filtered for falsy by
        // the caller — so a `null` frame is dropped rather than forwarded.
        if event.is_null() {
            return Vec::new();
        }
        vec![event]
    }

    fn session_update(&self, session: Value) -> Value {
        session_update_frame(session)
    }

    fn audio_append(&self, audio: &str) -> Value {
        audio_append_frame(audio)
    }

    fn conversation_item_id(&self, _item: &Value) -> String {
        // The beta dialect accepts one opaque namespace for every item type.
        format!("item_{}", hyphenless_uuid())
    }

    fn conversation_item_create(&self, item: Value) -> Value {
        item_create_frame(item)
    }

    fn response_create(&self, response: Option<Value>) -> Value {
        let mut frame = serde_json::Map::new();
        frame.insert(
            "type".to_owned(),
            Value::String("response.create".to_owned()),
        );
        if let Some(body) = response {
            frame.insert("response".to_owned(), body);
        }
        Value::Object(frame)
    }

    fn correlate_response_create(&self, payload: Value, _request_id: &str) -> Value {
        payload
    }

    fn response_correlation_id(&self, _event: &Value) -> String {
        String::new()
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

    fn protocol() -> OpenAiCompatibleProtocol {
        openai_compatible_protocol()
    }

    #[test]
    fn the_five_outgoing_frames_are_the_catalogued_ones() {
        let protocol = protocol();
        assert_eq!(
            protocol.session_update(json!({ "instructions": "hi" })),
            json!({ "type": "session.update", "session": { "instructions": "hi" } })
        );
        assert_eq!(
            protocol.audio_append("pcm"),
            json!({ "type": "input_audio_buffer.append", "audio": "pcm" })
        );
        assert_eq!(
            protocol.conversation_item_create(json!({ "id": "item_1" })),
            json!({ "type": "conversation.item.create", "item": { "id": "item_1" } })
        );
        assert_eq!(
            protocol.response_create(Some(json!({ "modalities": ["text"] }))),
            json!({ "type": "response.create", "response": { "modalities": ["text"] } })
        );
        assert_eq!(
            protocol.response_cancel(),
            json!({ "type": "response.cancel" })
        );
    }

    #[test]
    fn a_response_create_with_no_body_omits_the_key_entirely() {
        let frame = protocol().response_create(None);
        assert_eq!(frame, json!({ "type": "response.create" }));
        assert!(!frame.as_object().expect("object").contains_key("response"));
    }

    #[test]
    fn an_empty_body_is_not_the_same_as_no_body() {
        assert_eq!(
            protocol().response_create(Some(json!({}))),
            json!({ "type": "response.create", "response": {} })
        );
    }

    #[test]
    fn every_item_type_shares_one_id_namespace() {
        let protocol = protocol();
        for item in [
            json!({ "type": "message" }),
            json!({ "type": "function_call_output" }),
            json!({ "type": "anything" }),
            json!(null),
        ] {
            let id = protocol.conversation_item_id(&item);
            let hex = id.strip_prefix("item_").unwrap_or_else(|| panic!("{id}"));
            assert_eq!(hex.len(), 32, "{id}");
            assert!(hex.chars().all(|c| c.is_ascii_hexdigit()), "{id}");
        }
    }

    #[test]
    fn incoming_events_pass_through_unchanged() {
        let event = json!({
            "type": "input_audio_buffer.speech_started",
            "event_id": "event-input-1",
            "item_id": "item-input-1",
        });
        assert_eq!(protocol().normalize_incoming(event.clone()), vec![event]);
    }

    #[test]
    fn a_null_frame_is_dropped() {
        assert!(protocol().normalize_incoming(json!(null)).is_empty());
    }

    #[test]
    fn correlation_is_a_no_op_in_both_directions() {
        let protocol = protocol();
        let payload = json!({ "type": "response.create", "response": { "modalities": ["audio"] } });
        assert_eq!(
            protocol.correlate_response_create(payload.clone(), "request-1"),
            payload
        );
        assert_eq!(
            protocol.response_correlation_id(&json!({
                "response": { "metadata": { "qwen_audio_request_id": "request-1" } }
            })),
            ""
        );
    }

    #[test]
    fn the_envelope_stamps_an_event_id() {
        let frame = protocol().encode_outgoing(protocol().audio_append("pcm"));
        assert!(
            frame["event_id"]
                .as_str()
                .is_some_and(|id| id.starts_with("event_"))
        );
        assert_eq!(frame["type"], json!("input_audio_buffer.append"));
        assert_eq!(frame["audio"], json!("pcm"));
    }

    #[test]
    fn the_commit_frame_is_the_shared_default() {
        assert_eq!(
            protocol().audio_commit(),
            json!({ "type": "input_audio_buffer.commit" })
        );
    }

    #[test]
    fn there_are_no_connection_messages_by_default() {
        assert!(protocol().connection_messages().is_empty());
    }
}
