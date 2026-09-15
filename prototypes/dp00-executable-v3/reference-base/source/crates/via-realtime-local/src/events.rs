//! The event vocabulary the pipeline speaks.
//!
//! **This is the whole test.** `docs/architecture.md` §7 asks the local pipeline
//! to be *"presented to the Gateway as one realtime session"*, and the
//! measurable form of that is: every frame it emits is one a cloud provider
//! emits, spelled the way the shipped tables spell it, so **nothing above this
//! crate can tell the difference**.
//!
//! So nothing here is invented. Every name below is one the shipped Gateway
//! already reads, and each is annotated with the shipped table that reads it:
//!
//! | Frame | Read by |
//! | --- | --- |
//! | `session.created` / `session.updated` | `via-realtime`'s handshake (`session/state.rs`) |
//! | `error` | `realtime_event_classification_text`, then this crate's `classify_error` |
//! | `input_audio_buffer.speech_started` / `.speech_stopped` | `via-voice`'s `is_sleep_activity`, and the Injection Gate's `userSpeaking` |
//! | `input_audio_buffer.committed` | the input-turn item id |
//! | `conversation.item.created` | `via-realtime`'s item waiters (`resolve_item_waiter`) |
//! | `conversation.item.input_audio_transcription.delta` / `.completed` | `via-voice`'s `streaming_input_transcript` |
//! | `response.created` / `response.done` | `via-realtime`'s correlation and `RESPONSE_ACTIVITY_TYPES` |
//! | `response.audio.delta` / `.done` | `RESPONSE_ACTIVITY_TYPES`, and the client's playback |
//! | `response.audio_transcript.delta` / `.done` | `RESPONSE_ACTIVITY_TYPES` |
//! | `response.output_item.added`, `response.function_call_arguments.done` | the tool-call path |
//!
//! # Two fields that are not decoration
//!
//! **`text` and `stash` on a transcription delta.** `via-voice` renders the
//! running transcript as `` `${text}${stash}`.trim() ``
//! (`input-transcript.mjs:6-9`). A recognizer's uncommitted tail belongs in
//! `stash`; putting it in `text` makes the transcript jump backwards when the
//! recognizer revises it.
//!
//! **`response.metadata`.** The GA dialect's [`RESPONSE_CORRELATION_KEY`] is
//! what lets the session tell a response *it* asked for from an automatic
//! turn-detection response on the same session. The pipeline is both ends of its
//! own wire, so it can echo the key exactly — which is why
//! [`LocalPipelineProvider`](crate::LocalPipelineProvider) declares
//! `response_metadata_correlation: true` where a cloud provider that only
//! *might* echo it has to leave the flag at `false`.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde_json::{Map, Value};
use via_realtime::{FAILED_RESPONSE_STATUSES, RESPONSE_CORRELATION_KEY};

// ── statuses ────────────────────────────────────────────────────────────────

/// The `response.status` of a response that ran to completion.
///
/// The one status that is *not* in [`FAILED_RESPONSE_STATUSES`], and it is
/// spelled out because a status upstream does not recognise is treated as
/// success — so a typo here would be invisible.
pub const RESPONSE_COMPLETED: &str = "completed";

/// The `response.status` of a response a barge-in or a `response.cancel` cut.
///
/// Read out of `via-realtime`'s own table rather than retyped: it is the value
/// `is_completed_status` refuses, and two copies of it is a response that
/// reports success after being cancelled.
pub const RESPONSE_CANCELLED: &str = FAILED_RESPONSE_STATUSES[1];

/// The `response.status` of a response a stage failed during.
pub const RESPONSE_FAILED: &str = FAILED_RESPONSE_STATUSES[0];

/// The `response.status` a response carries while it is running.
pub const RESPONSE_IN_PROGRESS: &str = "in_progress";

/// The `error.type` every error this pipeline emits carries.
///
/// OpenAI Realtime's own value for a client-caused refusal, which is what both
/// of this pipeline's errors are.
pub const ERROR_TYPE: &str = "invalid_request_error";

// ── event names ─────────────────────────────────────────────────────────────

/// Every event type the pipeline can emit, in the order a turn produces them.
///
/// Exists so a test can assert that the pipeline's vocabulary is a **subset** of
/// what the shipped Gateway already reads, rather than checking one name at a
/// time and missing the one that was invented.
pub const EMITTED_EVENT_TYPES: [&str; 18] = [
    "session.created",
    "session.updated",
    "error",
    "input_audio_buffer.speech_started",
    "input_audio_buffer.speech_stopped",
    "input_audio_buffer.committed",
    "conversation.item.created",
    "conversation.item.truncated",
    "conversation.item.input_audio_transcription.delta",
    "conversation.item.input_audio_transcription.completed",
    "response.created",
    "response.output_item.added",
    "response.audio_transcript.delta",
    "response.audio.delta",
    "response.audio.done",
    "response.audio_transcript.done",
    "response.function_call_arguments.done",
    "response.done",
];

/// The frames a client writes that the pipeline acts on.
///
/// Anything else is ignored rather than refused: a dialect adapter above may
/// legitimately write a frame this stage has no opinion about, and answering it
/// with an `error` would put a protocol complaint on screen for a frame that did
/// no harm.
pub const HANDLED_CLIENT_FRAMES: [&str; 7] = [
    "session.update",
    "input_audio_buffer.append",
    "input_audio_buffer.commit",
    "input_audio_buffer.clear",
    "conversation.item.create",
    "conversation.item.truncate",
    "response.create",
];

// ── audio ───────────────────────────────────────────────────────────────────

/// PCM16 samples as the base64 a realtime audio delta carries.
#[must_use]
pub fn encode_audio(samples: &[i16]) -> String {
    BASE64.encode(via_audio::i16_to_pcm16le(samples))
}

/// The PCM16 samples inside a base64 audio append.
///
/// A frame that is not base64, or whose bytes are not a whole number of
/// samples, answers `None` rather than a truncated buffer: half a sample is
/// channel phase lost, and every sample after it is wrong.
#[must_use]
pub fn decode_audio(encoded: &str) -> Option<Vec<i16>> {
    let bytes = BASE64.decode(encoded.trim()).ok()?;
    if bytes.len() % via_audio::PCM16_BYTES_PER_SAMPLE != 0 {
        return None;
    }
    Some(via_audio::pcm16le_to_i16(&bytes))
}

// ── builders ────────────────────────────────────────────────────────────────

fn frame(kind: &str, fields: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    let mut map = Map::new();
    map.insert("type".to_owned(), Value::String(kind.to_owned()));
    for (key, value) in fields {
        map.insert(key.to_owned(), value);
    }
    Value::Object(map)
}

fn text(value: &str) -> Value {
    Value::String(value.to_owned())
}

/// `session.created`, the first frame of every session.
#[must_use]
pub fn session_created(session: Value) -> Value {
    frame("session.created", [("session", session)])
}

/// `session.updated`, the acknowledgement that makes the session ready.
#[must_use]
pub fn session_updated(session: Value) -> Value {
    frame("session.updated", [("session", session)])
}

/// An `error` frame in the shape both nestings of
/// [`is_active_response_conflict`](via_realtime_openai::is_active_response_conflict)
/// accept.
#[must_use]
pub fn error(code: &str, message: &str) -> Value {
    let mut body = Map::new();
    body.insert("type".to_owned(), text(ERROR_TYPE));
    body.insert("code".to_owned(), text(code));
    body.insert("message".to_owned(), text(message));
    frame("error", [("error", Value::Object(body))])
}

/// `input_audio_buffer.speech_started` — the Injection Gate's `userSpeaking`
/// edge, and the barge-in the Gateway sees.
#[must_use]
pub fn speech_started(item_id: &str, audio_start_ms: u64) -> Value {
    frame(
        "input_audio_buffer.speech_started",
        [
            ("audio_start_ms", Value::from(audio_start_ms)),
            ("item_id", text(item_id)),
        ],
    )
}

/// `input_audio_buffer.speech_stopped`.
#[must_use]
pub fn speech_stopped(item_id: &str, audio_end_ms: u64) -> Value {
    frame(
        "input_audio_buffer.speech_stopped",
        [
            ("audio_end_ms", Value::from(audio_end_ms)),
            ("item_id", text(item_id)),
        ],
    )
}

/// `input_audio_buffer.committed`.
#[must_use]
pub fn input_committed(item_id: &str, previous_item_id: Option<&str>) -> Value {
    frame(
        "input_audio_buffer.committed",
        [
            (
                "previous_item_id",
                previous_item_id.map_or(Value::Null, text),
            ),
            ("item_id", text(item_id)),
        ],
    )
}

/// `conversation.item.created`, echoing the item the client wrote.
///
/// The item is echoed **verbatim, id included**, which is what
/// `conversation_item_id_echo: true` declares and what lets
/// `resolve_item_waiter` match on the id the client minted rather than on
/// "there is exactly one outstanding".
#[must_use]
pub fn item_created(item: Value) -> Value {
    frame("conversation.item.created", [("item", item)])
}

/// `conversation.item.truncated`.
#[must_use]
pub fn item_truncated(item_id: &str, audio_end_ms: u64) -> Value {
    frame(
        "conversation.item.truncated",
        [
            ("item_id", text(item_id)),
            ("content_index", Value::from(0)),
            ("audio_end_ms", Value::from(audio_end_ms)),
        ],
    )
}

/// A streaming input-transcript delta.
///
/// `text` is the committed prefix and `stash` the recognizer's uncommitted
/// tail; `via-voice` concatenates them. See the module docs.
#[must_use]
pub fn transcription_delta(item_id: &str, committed: &str, stash: &str) -> Value {
    frame(
        "conversation.item.input_audio_transcription.delta",
        [
            ("item_id", text(item_id)),
            ("content_index", Value::from(0)),
            ("text", text(committed)),
            ("stash", text(stash)),
        ],
    )
}

/// The final input transcript for an utterance.
#[must_use]
pub fn transcription_completed(item_id: &str, transcript: &str) -> Value {
    frame(
        "conversation.item.input_audio_transcription.completed",
        [
            ("item_id", text(item_id)),
            ("content_index", Value::from(0)),
            ("transcript", text(transcript)),
        ],
    )
}

/// The `response` object both `response.created` and `response.done` carry.
fn response_object(id: &str, status: &str, correlation: Option<&str>, output: Vec<Value>) -> Value {
    let mut response = Map::new();
    response.insert("id".to_owned(), text(id));
    response.insert("object".to_owned(), text("realtime.response"));
    response.insert("status".to_owned(), text(status));
    if let Some(correlation) = correlation {
        let mut metadata = Map::new();
        metadata.insert(RESPONSE_CORRELATION_KEY.to_owned(), text(correlation));
        response.insert("metadata".to_owned(), Value::Object(metadata));
    }
    response.insert("output".to_owned(), Value::Array(output));
    Value::Object(response)
}

/// `response.created`.
#[must_use]
pub fn response_created(id: &str, correlation: Option<&str>) -> Value {
    frame(
        "response.created",
        [(
            "response",
            response_object(id, RESPONSE_IN_PROGRESS, correlation, Vec::new()),
        )],
    )
}

/// `response.done`.
#[must_use]
pub fn response_done(
    id: &str,
    status: &str,
    correlation: Option<&str>,
    output: Vec<Value>,
) -> Value {
    frame(
        "response.done",
        [("response", response_object(id, status, correlation, output))],
    )
}

/// `response.output_item.added`.
#[must_use]
pub fn output_item_added(response_id: &str, output_index: usize, item: Value) -> Value {
    frame(
        "response.output_item.added",
        [
            ("response_id", text(response_id)),
            ("output_index", Value::from(output_index)),
            ("item", item),
        ],
    )
}

/// The assistant message item a spoken response is delivered as.
#[must_use]
pub fn assistant_message_item(item_id: &str) -> Value {
    let mut item = Map::new();
    item.insert("id".to_owned(), text(item_id));
    item.insert("object".to_owned(), text("realtime.item"));
    item.insert("type".to_owned(), text("message"));
    item.insert("status".to_owned(), text(RESPONSE_IN_PROGRESS));
    item.insert("role".to_owned(), text("assistant"));
    item.insert("content".to_owned(), Value::Array(Vec::new()));
    Value::Object(item)
}

/// The function-call item a tool call is delivered as.
#[must_use]
pub fn function_call_item(item_id: &str, call_id: &str, name: &str, arguments: &str) -> Value {
    let mut item = Map::new();
    item.insert("id".to_owned(), text(item_id));
    item.insert("object".to_owned(), text("realtime.item"));
    item.insert("type".to_owned(), text("function_call"));
    item.insert("status".to_owned(), text(RESPONSE_COMPLETED));
    item.insert("name".to_owned(), text(name));
    item.insert("call_id".to_owned(), text(call_id));
    item.insert("arguments".to_owned(), text(arguments));
    Value::Object(item)
}

/// `response.audio_transcript.delta` — what the model is saying, as text.
#[must_use]
pub fn audio_transcript_delta(response_id: &str, item_id: &str, delta: &str) -> Value {
    frame(
        "response.audio_transcript.delta",
        [
            ("response_id", text(response_id)),
            ("item_id", text(item_id)),
            ("output_index", Value::from(0)),
            ("content_index", Value::from(0)),
            ("delta", text(delta)),
        ],
    )
}

/// `response.audio.delta` — base64 PCM16 at the speaker's rate.
#[must_use]
pub fn audio_delta(response_id: &str, item_id: &str, samples: &[i16]) -> Value {
    frame(
        "response.audio.delta",
        [
            ("response_id", text(response_id)),
            ("item_id", text(item_id)),
            ("output_index", Value::from(0)),
            ("content_index", Value::from(0)),
            ("delta", text(&encode_audio(samples))),
        ],
    )
}

/// `response.audio.done`.
#[must_use]
pub fn audio_done(response_id: &str, item_id: &str) -> Value {
    frame(
        "response.audio.done",
        [
            ("response_id", text(response_id)),
            ("item_id", text(item_id)),
            ("output_index", Value::from(0)),
            ("content_index", Value::from(0)),
        ],
    )
}

/// `response.audio_transcript.done`.
#[must_use]
pub fn audio_transcript_done(response_id: &str, item_id: &str, transcript: &str) -> Value {
    frame(
        "response.audio_transcript.done",
        [
            ("response_id", text(response_id)),
            ("item_id", text(item_id)),
            ("output_index", Value::from(0)),
            ("content_index", Value::from(0)),
            ("transcript", text(transcript)),
        ],
    )
}

/// `response.function_call_arguments.done`.
#[must_use]
pub fn function_call_arguments_done(
    response_id: &str,
    item_id: &str,
    call_id: &str,
    name: &str,
    arguments: &str,
) -> Value {
    frame(
        "response.function_call_arguments.done",
        [
            ("response_id", text(response_id)),
            ("item_id", text(item_id)),
            ("output_index", Value::from(0)),
            ("call_id", text(call_id)),
            ("name", text(name)),
            ("arguments", text(arguments)),
        ],
    )
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use via_realtime::{RESPONSE_ACTIVITY_TYPES, is_completed_status, realtime_response_id};

    use super::*;

    #[test]
    fn every_response_frame_carries_an_id_the_shipped_lookup_can_find() {
        // `response-lifecycle.mjs:24-29`: `response_id || response.id ||
        // item.response_id`. A frame the session cannot attribute proves
        // nothing, so every one of ours must be attributable.
        let frames = [
            response_created("resp_1", None),
            response_done("resp_1", RESPONSE_COMPLETED, None, Vec::new()),
            audio_transcript_delta("resp_1", "item_1", "hi"),
            audio_delta("resp_1", "item_1", &[0, 1]),
            audio_done("resp_1", "item_1"),
            audio_transcript_done("resp_1", "item_1", "hi"),
            function_call_arguments_done("resp_1", "item_1", "call_1", "f", "{}"),
            output_item_added("resp_1", 0, assistant_message_item("item_1")),
        ];
        for frame in frames {
            assert_eq!(realtime_response_id(&frame), "resp_1", "{frame}");
        }
    }

    #[test]
    fn every_response_frame_is_in_the_shipped_activity_table() {
        for frame in [
            response_created("r", None),
            response_done("r", RESPONSE_COMPLETED, None, Vec::new()),
            audio_transcript_delta("r", "i", ""),
            audio_delta("r", "i", &[]),
            audio_done("r", "i"),
            audio_transcript_done("r", "i", ""),
            function_call_arguments_done("r", "i", "c", "f", "{}"),
            output_item_added("r", 0, assistant_message_item("i")),
        ] {
            let kind = frame["type"].as_str().unwrap_or_default();
            assert!(
                RESPONSE_ACTIVITY_TYPES.contains(&kind),
                "{kind} is not response activity the Gateway recognises"
            );
        }
    }

    #[test]
    fn the_three_statuses_agree_with_the_shipped_failure_table() {
        assert!(is_completed_status(Some(RESPONSE_COMPLETED)));
        assert!(!is_completed_status(Some(RESPONSE_CANCELLED)));
        assert!(!is_completed_status(Some(RESPONSE_FAILED)));
        assert_eq!(RESPONSE_CANCELLED, "cancelled");
        assert_eq!(RESPONSE_FAILED, "failed");
    }

    #[test]
    fn the_correlation_key_is_echoed_only_when_the_client_sent_one() {
        let created = response_created("resp_1", Some("req-9"));
        assert_eq!(
            created["response"]["metadata"][RESPONSE_CORRELATION_KEY],
            json!("req-9")
        );
        let done = response_done("resp_1", RESPONSE_COMPLETED, Some("req-9"), Vec::new());
        assert_eq!(
            done["response"]["metadata"][RESPONSE_CORRELATION_KEY],
            json!("req-9")
        );
        // The pipeline echoes exactly what the GA dialect stamped, so the
        // dialect's own reader finds it.
        let protocol = via_realtime::ga_realtime_protocol();
        assert_eq!(
            via_realtime::RealtimeProtocol::response_correlation_id(&protocol, &created),
            "req-9"
        );

        let uncorrelated = response_created("resp_1", None);
        assert!(uncorrelated["response"].get("metadata").is_none());
        assert_eq!(
            via_realtime::RealtimeProtocol::response_correlation_id(&protocol, &uncorrelated),
            ""
        );
    }

    #[test]
    fn an_error_frame_is_readable_through_both_shipped_paths() {
        let frame = error("conversation_already_has_active_response", "busy");
        // `realtime-provider.mjs:46-59` reads `error.message`.
        assert_eq!(
            via_realtime::realtime_event_classification_text(&frame),
            "busy"
        );
        // ARGO's conflict matcher reads `error.code`.
        assert!(via_realtime_openai::is_active_response_conflict(&frame));
    }

    #[test]
    fn a_transcription_delta_renders_the_way_via_voice_will() {
        let frame = transcription_delta("item_1", "turn on the ", "ligh");
        assert_eq!(frame["text"], json!("turn on the "));
        assert_eq!(frame["stash"], json!("ligh"));
        assert_eq!(frame["item_id"], json!("item_1"));
    }

    #[test]
    fn audio_round_trips_through_base64_pcm16() {
        let samples: Vec<i16> = vec![0, 1, -1, i16::MAX, i16::MIN, 4242];
        let encoded = encode_audio(&samples);
        assert_eq!(decode_audio(&encoded), Some(samples.clone()));

        let frame = audio_delta("r", "i", &samples);
        let delta = frame["delta"].as_str().unwrap_or_default();
        assert_eq!(decode_audio(delta), Some(samples));
    }

    #[test]
    fn an_unusable_audio_payload_is_none_rather_than_a_truncated_buffer() {
        assert_eq!(decode_audio("not base64!!"), None);
        // One byte is half a sample: channel phase is lost, and every sample
        // after it would be wrong.
        assert_eq!(decode_audio(&BASE64.encode([0u8; 3])), None);
        assert_eq!(decode_audio(""), Some(Vec::new()));
    }

    #[test]
    fn whitespace_around_an_audio_payload_is_tolerated() {
        let encoded = encode_audio(&[7, 8]);
        assert_eq!(decode_audio(&format!("  {encoded}\n")), Some(vec![7, 8]));
    }

    #[test]
    fn the_emitted_vocabulary_has_no_duplicates_and_leads_with_type() {
        let mut sorted = EMITTED_EVENT_TYPES.to_vec();
        sorted.sort_unstable();
        let mut unique = sorted.clone();
        unique.dedup();
        assert_eq!(sorted, unique);

        let frame = speech_started("item_1", 40);
        let keys: Vec<&str> = frame
            .as_object()
            .map(|object| object.keys().map(String::as_str).collect())
            .unwrap_or_default();
        assert_eq!(keys, ["type", "audio_start_ms", "item_id"]);
    }

    #[test]
    fn a_committed_frame_carries_a_null_previous_item_when_there_is_none() {
        assert_eq!(
            input_committed("item_2", None)["previous_item_id"],
            json!(null)
        );
        assert_eq!(
            input_committed("item_2", Some("item_1"))["previous_item_id"],
            json!("item_1")
        );
    }

    #[test]
    fn an_item_is_echoed_verbatim_so_the_client_id_matches() {
        let item = json!({ "id": "item_client", "type": "message", "role": "user" });
        assert_eq!(item_created(item.clone())["item"], item);
    }

    #[test]
    fn a_function_call_item_carries_the_three_fields_the_gateway_reads() {
        let item = function_call_item("item_1", "call_1", "request_delegation", "{\"a\":1}");
        assert_eq!(item["type"], json!("function_call"));
        assert_eq!(item["call_id"], json!("call_1"));
        assert_eq!(item["name"], json!("request_delegation"));
        assert_eq!(item["arguments"], json!("{\"a\":1}"));
    }
}
