//! Which server events prove a response exists, and which id it belongs to.
//!
//! Ported from `server/src/voice/response-lifecycle.mjs`. It sits in
//! `via-realtime` rather than `via-voice` because the session's own correlation
//! depends on it: `handleLifecycle` cannot decide whether a response is still
//! producing output without this table, and a crate that owns the session must
//! own the predicate the session branches on.
//!
//! The twenty names are deliberately a *union* of the beta and GA dialects. A
//! compliant provider opens with `response.created`, but — upstream's own
//! comment — *"compatible implementations may emit output activity (or even
//! `response.done`) without that opening event"*, and the GA dialect spells four
//! of them `response.output_*`. Recognising both spellings here is what lets one
//! session state machine serve both dialects.

/// Every server event that proves a Realtime response exists.
///
/// External contract — `response-lifecycle.mjs:1-22`. Sorted the way upstream
/// declares it: lifecycle, then output items, then content parts, then function
/// call arguments, then audio, then transcripts, then text — each in
/// beta-spelling / GA-spelling pairs.
pub const RESPONSE_ACTIVITY_TYPES: [&str; 20] = [
    "response.created",
    "response.done",
    "response.output_item.added",
    "response.output_item.done",
    "response.content_part.added",
    "response.content_part.done",
    "response.function_call_arguments.delta",
    "response.function_call_arguments.done",
    "response.audio.delta",
    "response.audio.done",
    "response.output_audio.delta",
    "response.output_audio.done",
    "response.audio_transcript.delta",
    "response.audio_transcript.done",
    "response.output_audio_transcript.delta",
    "response.output_audio_transcript.done",
    "response.text.delta",
    "response.text.done",
    "response.output_text.delta",
    "response.output_text.done",
];

/// The response id an event belongs to, in upstream's lookup order.
///
/// External contract — `response-lifecycle.mjs:24-29`:
/// `event.response_id || event.response.id || event.item.response_id || ''`.
/// The empty string means "no id", which the session treats as "do not bind a
/// waiter to it".
#[must_use]
pub fn realtime_response_id(event: &serde_json::Value) -> &str {
    for path in [
        &["response_id"][..],
        &["response", "id"][..],
        &["item", "response_id"][..],
    ] {
        let mut cursor = event;
        let mut found = true;
        for segment in path {
            match cursor.get(segment) {
                Some(next) => cursor = next,
                None => {
                    found = false;
                    break;
                }
            }
        }
        if found
            && let Some(id) = cursor.as_str()
            && !id.is_empty()
        {
            return id;
        }
    }
    ""
}

/// True for a server event that proves a Realtime response exists.
///
/// External contract — `response-lifecycle.mjs:36-41`. Both halves are required:
/// an event with no response id proves nothing, and an event outside the table
/// (a `conversation.item.created`, a transcription delta for *input* audio) is
/// not response activity even when it carries one.
#[must_use]
pub fn is_response_activity_event(event: &serde_json::Value) -> bool {
    if realtime_response_id(event).is_empty() {
        return false;
    }
    event
        .get("type")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|kind| RESPONSE_ACTIVITY_TYPES.contains(&kind))
}

/// The three response statuses `response.done` reports as a failure.
///
/// External contract — `default-value` / *response failure statuses*
/// (`realtime-provider.mjs:663-668`). The catalogue's warning is the reason this
/// is a constant rather than an inline literal: *"a provider that reports e.g.
/// `'error'` would be treated as success."*
pub const FAILED_RESPONSE_STATUSES: [&str; 3] = ["failed", "cancelled", "incomplete"];

/// Whether a `response.done` with this status counts as a completed response.
#[must_use]
pub fn is_completed_status(status: Option<&str>) -> bool {
    !status.is_some_and(|status| FAILED_RESPONSE_STATUSES.contains(&status))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn the_id_lookup_order_is_response_id_then_response_then_item() {
        assert_eq!(
            realtime_response_id(&json!({ "response_id": "a", "response": { "id": "b" } })),
            "a"
        );
        assert_eq!(
            realtime_response_id(
                &json!({ "response": { "id": "b" }, "item": { "response_id": "c" } })
            ),
            "b"
        );
        assert_eq!(
            realtime_response_id(&json!({ "item": { "response_id": "c" } })),
            "c"
        );
        assert_eq!(realtime_response_id(&json!({})), "");
    }

    #[test]
    fn an_empty_id_falls_through_to_the_next_candidate() {
        assert_eq!(
            realtime_response_id(&json!({ "response_id": "", "response": { "id": "b" } })),
            "b"
        );
    }

    #[test]
    fn a_non_string_id_is_not_an_id() {
        assert_eq!(realtime_response_id(&json!({ "response_id": 7 })), "");
    }

    #[test]
    fn activity_needs_both_an_id_and_a_listed_type() {
        assert!(is_response_activity_event(
            &json!({ "type": "response.audio.delta", "response_id": "r" })
        ));
        assert!(!is_response_activity_event(
            &json!({ "type": "response.audio.delta" })
        ));
        assert!(!is_response_activity_event(
            &json!({ "type": "conversation.item.created", "response_id": "r" })
        ));
        assert!(!is_response_activity_event(&json!({ "response_id": "r" })));
    }

    #[test]
    fn both_dialect_spellings_are_activity() {
        for kind in [
            "response.audio_transcript.delta",
            "response.output_audio_transcript.delta",
            "response.text.done",
            "response.output_text.done",
        ] {
            assert!(
                is_response_activity_event(&json!({ "type": kind, "response_id": "r" })),
                "{kind}"
            );
        }
    }

    #[test]
    fn the_activity_table_has_no_duplicates() {
        let mut sorted = RESPONSE_ACTIVITY_TYPES.to_vec();
        sorted.sort_unstable();
        let mut unique = sorted.clone();
        unique.dedup();
        assert_eq!(sorted, unique);
    }

    #[test]
    fn only_the_three_catalogued_statuses_are_failures() {
        assert!(is_completed_status(Some("completed")));
        assert!(is_completed_status(None));
        // A status upstream does not know is treated as success, deliberately.
        assert!(is_completed_status(Some("error")));
        for status in FAILED_RESPONSE_STATUSES {
            assert!(!is_completed_status(Some(status)), "{status}");
        }
    }
}
