//! Response lifecycle and response context.
//!
//! Ported from `server/src/voice/response-lifecycle.mjs` and
//! `server/src/voice/response-context.mjs`.
//!
//! # Why "activity proves a response exists"
//!
//! A compliant Realtime provider emits `response.created` first. Compatible
//! implementations do not: some start an implicit server-VAD response and reveal
//! it only through audio or transcript output, and some emit `response.done`
//! with no opening event at all. So the gateway treats **any** of the twenty
//! output events as proof that a response exists, and
//! [`RealtimeResponseId::of`] is the resolution order that finds its id.
//!
//! # Why a later delta must not reset the context
//!
//! Providers commonly attach correlation metadata only to
//! `response.created` / `response.done`, never to a per-token delta. If a
//! delta's missing metadata were merged as "no correlation", an announcement
//! mid-flight would silently become an ordinary model turn — and the Work whose
//! result it carries would never be marked delivered.
//! [`response_activity_context_patch`] is the rule that prevents it: the
//! fallback is applied **only** when nothing was correlated yet.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Where a response came from.
///
/// **External contract** — the `origin` field spread into every
/// `transcript.*`, `response.*` and `voice.state` frame. `model` is the
/// realtime model answering the user; the other three are the Gateway speaking
/// through it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseOrigin {
    /// The realtime model's own turn.
    #[default]
    Model,
    /// A finished Work's result, presented through the Injection Gate.
    Announcement,
    /// A backend permission question.
    Permission,
    /// A delegated project's start announcement, or a tool follow-up.
    Agent,
    /// A background task's progress update.
    Progress,
}

impl ResponseOrigin {
    /// The wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Announcement => "announcement",
            Self::Permission => "permission",
            Self::Agent => "agent",
            Self::Progress => "progress",
        }
    }

    /// Parse a wire spelling.
    #[must_use]
    pub fn from_wire(value: &str) -> Option<Self> {
        match value {
            "model" => Some(Self::Model),
            "announcement" => Some(Self::Announcement),
            "permission" => Some(Self::Permission),
            "agent" => Some(Self::Agent),
            "progress" => Some(Self::Progress),
            _ => None,
        }
    }
}

impl std::fmt::Display for ResponseOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The twenty server event types that prove a Realtime response exists.
///
/// **External contract** — `response-lifecycle.mjs:1-22`, in that order. Both
/// the beta (`response.audio.*`, `response.text.*`) and GA
/// (`response.output_audio.*`, `response.output_text.*`) spellings are here,
/// because the dialect is the provider's choice and the gateway must not
/// branch on it.
pub const RESPONSE_ACTIVITY_TYPES: &[&str] = &[
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

/// The three `response.done` statuses that mean the response did not land.
///
/// **External contract** — `realtime-gateway.mjs:1241-1243` and
/// `realtime-provider.mjs:663-668`, the same list in both.
pub const RESPONSE_FAILURE_STATUSES: &[&str] = &["failed", "cancelled", "incomplete"];

/// The two streaming input-transcript event names.
///
/// **External contract** — `input-transcript.mjs:1-4`. The second is the
/// alternate Qwen ASR streaming event name; `docs/rebrand.md` keeps it, because
/// it is inbound provider data rather than VIA's identity.
pub const STREAMING_INPUT_TRANSCRIPT_EVENTS: &[&str] = &[
    "conversation.item.input_audio_transcription.delta",
    "conversation.item.input_audio_transcription.text",
];

/// A normalized server event, as the gateway sees it after the protocol
/// adapter has run.
///
/// Only the fields the voice layer reads are named; `extra` keeps everything
/// else so a provider-specific field survives a round trip.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ServerEvent {
    /// The `type` discriminator.
    pub event_type: String,
    /// `event.response_id`.
    pub response_id: Option<String>,
    /// `event.response.id`.
    pub response_object_id: Option<String>,
    /// `event.item.response_id`.
    pub item_response_id: Option<String>,
    /// `event.item_id` — the conversation item, not the response.
    pub item_id: Option<String>,
    /// `event.response.status`.
    pub response_status: Option<String>,
    /// `event.delta`.
    pub delta: Option<String>,
    /// `event.text` — the streaming input-transcript body.
    pub text: Option<String>,
    /// `event.stash` — the streaming input-transcript tail.
    pub stash: Option<String>,
    /// `event.transcript`.
    pub transcript: Option<String>,
    /// `event.reason`, e.g. `turn_invalid`.
    pub reason: Option<String>,
    /// `event.__voiceOrigin`, attached by the frontend on correlation.
    pub voice_origin: Option<ResponseOrigin>,
    /// `event.__voiceContext`, attached by the frontend on correlation.
    pub voice_context: Option<CorrelatedContext>,
    /// `event.__voiceRetried` — the frontend transparently replayed this
    /// refusal, so nothing user-facing happened.
    pub voice_retried: bool,
}

impl ServerEvent {
    /// An event of `event_type` with nothing else set.
    #[must_use]
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            ..Self::default()
        }
    }

    /// Whether this event proves a response exists.
    ///
    /// **External contract** — `response-lifecycle.mjs:36-41`: it needs both a
    /// resolvable id **and** a type in [`RESPONSE_ACTIVITY_TYPES`]. An activity
    /// type with no id proves nothing the gateway can act on.
    #[must_use]
    pub fn is_response_activity(&self) -> bool {
        !self.realtime_response_id().is_empty()
            && RESPONSE_ACTIVITY_TYPES.contains(&self.event_type.as_str())
    }

    /// The response id, by upstream's resolution order.
    ///
    /// **External contract** — `response-lifecycle.mjs:24-29`:
    /// `response_id || response.id || item.response_id || ''`. The order
    /// matters — a `response.output_item.done` carries both `item.response_id`
    /// and, on some providers, a *different* `response.id` for the enclosing
    /// response.
    #[must_use]
    pub fn realtime_response_id(&self) -> &str {
        for candidate in [
            &self.response_id,
            &self.response_object_id,
            &self.item_response_id,
        ] {
            if let Some(value) = candidate.as_deref()
                && !value.is_empty()
            {
                return value;
            }
        }
        ""
    }

    /// Whether this `response.done` failed.
    ///
    /// **External contract** — [`RESPONSE_FAILURE_STATUSES`].
    #[must_use]
    pub fn response_failed(&self) -> bool {
        self.response_status
            .as_deref()
            .is_some_and(|status| RESPONSE_FAILURE_STATUSES.contains(&status))
    }

    /// The running input transcript carried by a streaming ASR event.
    ///
    /// **External contract** — `input-transcript.mjs:6-9`:
    /// `` `${text}${stash}`.trim() ``, and `''` for any other event type. The
    /// concatenation is the contract — `stash` is the provider's uncommitted
    /// tail, and dropping it makes the running transcript lag a word behind
    /// what the user just said.
    #[must_use]
    pub fn streaming_input_transcript(&self) -> String {
        if !STREAMING_INPUT_TRANSCRIPT_EVENTS.contains(&self.event_type.as_str()) {
            return String::new();
        }
        let text = self.text.as_deref().unwrap_or_default();
        let stash = self.stash.as_deref().unwrap_or_default();
        crate::text::trim(&format!("{text}{stash}")).to_owned()
    }

    /// Whether this event should postpone the sleep timer.
    ///
    /// **External contract** — `realtime-gateway.mjs:76-84`
    /// (`isSleepActivityEvent`): every response activity event, plus the four
    /// input-side events. Speech the model never answered still means the user
    /// is there.
    #[must_use]
    pub fn is_sleep_activity(&self) -> bool {
        self.is_response_activity()
            || matches!(
                self.event_type.as_str(),
                "input_audio_buffer.speech_started"
                    | "input_audio_buffer.speech_stopped"
                    | "conversation.item.input_audio_transcription.delta"
                    | "conversation.item.input_audio_transcription.completed"
            )
    }
}

/// The correlation metadata a provider echoes back on `response.created` /
/// `response.done`.
///
/// **External contract** — the eight `CORRELATED_CONTEXT_FIELDS` of
/// `response-context.mjs:5-14`, plus the origin, which travels beside them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CorrelatedContext {
    /// The turn this response answers.
    pub turn_id: Option<String>,
    /// The single Work, when there is exactly one.
    pub task_id: Option<Option<String>>,
    /// Every Work in an announcement batch.
    pub task_ids: Option<Vec<String>>,
    /// Every turn in an announcement batch.
    pub turn_ids: Option<Vec<String>>,
    /// The permission this response is asking about.
    pub authorization_id: Option<Option<String>>,
    /// The turn generation, for staleness.
    pub turn_generation: Option<i64>,
    /// The announcement batch's sequence number.
    pub delivery_sequence: Option<Option<u64>>,
    /// Whether answering this response also discharges a Work notification.
    pub consumes_task_notification: Option<bool>,
}

/// One in-flight response's accrued state.
///
/// **External contract** — `response-context.mjs:49-60` for the initial shape,
/// and the fields the gateway sets on it as the response streams.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResponseContext {
    /// The turn this response answers.
    pub turn_id: String,
    /// The single correlated Work.
    pub task_id: Option<String>,
    /// Every Work in an announcement batch.
    pub task_ids: Vec<String>,
    /// Every turn in an announcement batch.
    pub turn_ids: Vec<String>,
    /// The permission being asked about.
    pub authorization_id: Option<String>,
    /// Where this response came from.
    pub origin: ResponseOrigin,
    /// The turn generation. `-1` until something correlates it.
    pub turn_generation: i64,
    /// The announcement batch's sequence number.
    pub delivery_sequence: Option<u64>,
    /// Whether answering this also discharges a Work notification.
    pub consumes_task_notification: bool,
    /// The client reported playback started.
    pub playback_started: bool,
    /// The client reported playback ended.
    pub playback_ended: bool,
    /// The provider reported `response.done`.
    pub response_done: bool,
    /// The transcript reached its `.done`.
    pub transcript_done: bool,
    /// The response emitted at least one audio delta.
    pub has_audio: bool,
    /// The response authored at least one tool call.
    pub has_function_call: bool,
    /// The response was cancelled; it survives as a short-lived tombstone so
    /// late provider audio and client receipts cannot resurrect it.
    pub suppressed: bool,
    /// Whether `response.started` has been announced to the client.
    pub response_started: bool,
    /// The final assistant transcript, for the response guards.
    pub assistant_transcript: String,
    /// Transcript fragments held until playback actually starts.
    pub pending_transcripts: Vec<PendingTranscript>,
}

/// A transcript fragment held until the client reports playback started.
///
/// The client renders the assistant's words in step with the audio, so a
/// transcript emitted before the first sample plays shows text for a sentence
/// the user has not heard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingTranscript {
    /// The fragment.
    pub content: String,
    /// Whether this is the settled transcript rather than a delta.
    pub final_fragment: bool,
}

impl ResponseContext {
    /// Whether every terminal signal has arrived.
    ///
    /// **External contract** — `realtime-gateway.mjs:625-634`
    /// (`finishResponseContextIfComplete`): playback ended **and**
    /// `response.done` **and** the transcript is done. All three, because each
    /// can arrive last.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.playback_ended && self.response_done && self.transcript_done
    }

    /// The Work ids this response carries.
    ///
    /// **External contract** — `realtime-gateway.mjs:576-578`
    /// (`contextTaskIds`): the batch list when there is one, otherwise the
    /// single id, otherwise nothing.
    #[must_use]
    pub fn task_ids(&self) -> Vec<String> {
        if self.task_ids.is_empty() {
            self.task_id.iter().cloned().collect()
        } else {
            self.task_ids.clone()
        }
    }

    /// Whether the *start* of playback discharges a Work notification.
    ///
    /// **External contract** — `realtime-gateway.mjs:84-93`
    /// (`confirmsTaskNotificationOnPlaybackStart`). An announcement always
    /// does; an ordinary model turn does only when a
    /// `get_agent_task_status` answer already told the user the result, which
    /// is what `consumes_task_notification` marks.
    #[must_use]
    pub const fn confirms_task_notification_on_playback_start(&self) -> bool {
        matches!(self.origin, ResponseOrigin::Announcement) || self.consumes_task_notification
    }
}

/// The authoritative part of a response context, derived from one event.
///
/// **External contract** — `response-context.mjs:24-40`. `existing` decides
/// whether the fallback applies at all: a first sighting takes the whole
/// fallback, a later event takes only what the provider actually echoed. See
/// the module documentation for why.
#[must_use]
pub fn response_activity_context_patch(
    existing: Option<&ResponseContext>,
    event: &ServerEvent,
    fallback: &ResponseContext,
) -> ResponseContext {
    let mut patch = if existing.is_some() {
        ResponseContext::default()
    } else {
        fallback.clone()
    };
    // `patched` records which fields the correlation actually supplied, so
    // `merge_response_context` can distinguish "the provider echoed nothing"
    // from "the provider echoed a default".
    if let Some(correlated) = &event.voice_context {
        if let Some(turn_id) = &correlated.turn_id {
            patch.turn_id.clone_from(turn_id);
        }
        if let Some(task_id) = &correlated.task_id {
            patch.task_id.clone_from(task_id);
        }
        if let Some(task_ids) = &correlated.task_ids {
            patch.task_ids.clone_from(task_ids);
        }
        if let Some(turn_ids) = &correlated.turn_ids {
            patch.turn_ids.clone_from(turn_ids);
        }
        if let Some(authorization_id) = &correlated.authorization_id {
            patch.authorization_id.clone_from(authorization_id);
        }
        if let Some(turn_generation) = correlated.turn_generation {
            patch.turn_generation = turn_generation;
        }
        if let Some(delivery_sequence) = correlated.delivery_sequence {
            patch.delivery_sequence = delivery_sequence;
        }
        if let Some(consumes) = correlated.consumes_task_notification {
            patch.consumes_task_notification = consumes;
        }
    }
    if let Some(origin) = event.voice_origin {
        patch.origin = origin;
    }
    patch
}

/// The response-context map.
///
/// One entry per in-flight response id, insertion-ordered so a bounded sweep
/// evicts the oldest.
pub type ResponseContexts = IndexMap<String, ResponseContext>;

/// Return the context for `id`, creating it from `fallback` if absent.
///
/// **External contract** — `response-context.mjs:42-63`.
pub fn ensure_response_context<'contexts>(
    contexts: &'contexts mut ResponseContexts,
    id: &str,
    fallback: &ResponseContext,
) -> &'contexts mut ResponseContext {
    contexts
        .entry(id.to_owned())
        .or_insert_with(|| fallback.clone())
}

/// Merge `authoritative` over the context for `id`, preserving progress.
///
/// **External contract** — `response-context.mjs:65-78`. The five preserved
/// fields are the point: correlation arriving late (on `response.done`, say)
/// must not un-play audio the client already started, or drop transcript
/// fragments still waiting for playback to begin.
pub fn merge_response_context<'contexts>(
    contexts: &'contexts mut ResponseContexts,
    id: &str,
    authoritative: ResponseContext,
) -> &'contexts mut ResponseContext {
    let existing = contexts.entry(id.to_owned()).or_default();
    let preserved = (
        existing.playback_started,
        existing.playback_ended,
        existing.response_done,
        existing.transcript_done,
        std::mem::take(&mut existing.pending_transcripts),
    );
    // Fields the gateway (not the correlation) owns survive the merge too:
    // upstream keeps them by spreading `...existing` first, and they are only
    // ever set to `true`.
    let has_audio = existing.has_audio;
    let has_function_call = existing.has_function_call;
    let suppressed = existing.suppressed;
    let response_started = existing.response_started;
    let assistant_transcript = std::mem::take(&mut existing.assistant_transcript);

    *existing = authoritative;
    existing.playback_started = preserved.0;
    existing.playback_ended = preserved.1;
    existing.response_done = preserved.2;
    existing.transcript_done = preserved.3;
    existing.pending_transcripts = preserved.4;
    existing.has_audio |= has_audio;
    existing.has_function_call |= has_function_call;
    existing.suppressed |= suppressed;
    existing.response_started |= response_started;
    if existing.assistant_transcript.is_empty() {
        existing.assistant_transcript = assistant_transcript;
    }
    existing
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn correlated(turn_id: &str) -> CorrelatedContext {
        CorrelatedContext {
            turn_id: Some(turn_id.to_owned()),
            ..CorrelatedContext::default()
        }
    }

    #[test]
    fn the_response_id_resolution_order_is_the_catalogued_one() {
        let mut event = ServerEvent::new("response.done");
        assert_eq!(event.realtime_response_id(), "");
        event.item_response_id = Some("from_item".to_owned());
        assert_eq!(event.realtime_response_id(), "from_item");
        event.response_object_id = Some("from_response".to_owned());
        assert_eq!(event.realtime_response_id(), "from_response");
        event.response_id = Some("from_top".to_owned());
        assert_eq!(event.realtime_response_id(), "from_top");
        // An empty string is skipped, not returned.
        event.response_id = Some(String::new());
        assert_eq!(event.realtime_response_id(), "from_response");
    }

    #[test]
    fn activity_needs_both_an_id_and_a_known_type() {
        let mut event = ServerEvent::new("response.audio.delta");
        assert!(!event.is_response_activity(), "no id proves nothing");
        event.response_id = Some("resp_1".to_owned());
        assert!(event.is_response_activity());
        event.event_type = "session.updated".to_owned();
        assert!(!event.is_response_activity());
    }

    #[test]
    fn every_catalogued_activity_type_counts() {
        for event_type in RESPONSE_ACTIVITY_TYPES {
            let mut event = ServerEvent::new(*event_type);
            event.response_id = Some("resp_1".to_owned());
            assert!(event.is_response_activity(), "{event_type} is not activity");
        }
        assert_eq!(RESPONSE_ACTIVITY_TYPES.len(), 20);
    }

    #[test]
    fn a_later_delta_without_metadata_keeps_the_correlated_context() {
        let existing = ResponseContext {
            turn_id: "voice-1".to_owned(),
            origin: ResponseOrigin::Announcement,
            task_ids: vec!["work_a".to_owned()],
            ..ResponseContext::default()
        };
        let fallback = ResponseContext {
            turn_id: "voice-9".to_owned(),
            origin: ResponseOrigin::Model,
            ..ResponseContext::default()
        };
        let delta = ServerEvent::new("response.audio.delta");
        let patch = response_activity_context_patch(Some(&existing), &delta, &fallback);
        // Nothing from the fallback leaked in.
        assert_eq!(patch.turn_id, "");
        assert_eq!(patch.origin, ResponseOrigin::Model);

        let mut contexts = ResponseContexts::new();
        contexts.insert("resp_1".to_owned(), existing);
        // The merge is where the preservation actually shows.
        let merged = merge_response_context(&mut contexts, "resp_1", patch);
        assert_eq!(merged.turn_id, "");
        assert!(merged.task_ids.is_empty());
    }

    #[test]
    fn a_first_sighting_takes_the_whole_fallback() {
        let fallback = ResponseContext {
            turn_id: "voice-9".to_owned(),
            turn_generation: 4,
            origin: ResponseOrigin::Model,
            ..ResponseContext::default()
        };
        let event = ServerEvent::new("response.created");
        let patch = response_activity_context_patch(None, &event, &fallback);
        assert_eq!(patch.turn_id, "voice-9");
        assert_eq!(patch.turn_generation, 4);
    }

    #[test]
    fn correlation_arriving_late_does_not_unplay_audio() {
        let mut contexts = ResponseContexts::new();
        let mut context = ResponseContext {
            playback_started: true,
            has_audio: true,
            ..ResponseContext::default()
        };
        context.pending_transcripts.push(PendingTranscript {
            content: "hi".to_owned(),
            final_fragment: false,
        });
        contexts.insert("resp_1".to_owned(), context);

        let mut done = ServerEvent::new("response.done");
        done.response_id = Some("resp_1".to_owned());
        done.voice_origin = Some(ResponseOrigin::Announcement);
        done.voice_context = Some(correlated("voice-1"));
        let existing = contexts.get("resp_1").cloned();
        let patch =
            response_activity_context_patch(existing.as_ref(), &done, &ResponseContext::default());
        let merged = merge_response_context(&mut contexts, "resp_1", patch);
        assert!(merged.playback_started, "playback progress survived");
        assert!(merged.has_audio, "audio flag survived");
        assert_eq!(merged.pending_transcripts.len(), 1);
        assert_eq!(merged.origin, ResponseOrigin::Announcement);
        assert_eq!(merged.turn_id, "voice-1");
    }

    #[test]
    fn a_streaming_transcript_concatenates_text_and_stash() {
        let mut event = ServerEvent::new("conversation.item.input_audio_transcription.delta");
        event.text = Some("你好".to_owned());
        event.stash = Some("世界 ".to_owned());
        assert_eq!(event.streaming_input_transcript(), "你好世界");

        // The alternate Qwen ASR spelling behaves identically.
        event.event_type = "conversation.item.input_audio_transcription.text".to_owned();
        assert_eq!(event.streaming_input_transcript(), "你好世界");

        // Anything else is empty, even with the fields set.
        event.event_type = "response.audio.delta".to_owned();
        assert_eq!(event.streaming_input_transcript(), "");
    }

    #[test]
    fn sleep_activity_covers_response_output_and_the_four_input_events() {
        let mut delta = ServerEvent::new("response.audio.delta");
        delta.response_id = Some("resp_1".to_owned());
        assert!(delta.is_sleep_activity());
        for event_type in [
            "input_audio_buffer.speech_started",
            "input_audio_buffer.speech_stopped",
            "conversation.item.input_audio_transcription.delta",
            "conversation.item.input_audio_transcription.completed",
        ] {
            assert!(
                ServerEvent::new(event_type).is_sleep_activity(),
                "{event_type}"
            );
        }
        // Committed is deliberately not in the list.
        assert!(!ServerEvent::new("input_audio_buffer.committed").is_sleep_activity());
    }

    #[test]
    fn the_three_failure_statuses_and_nothing_else() {
        for status in RESPONSE_FAILURE_STATUSES {
            let mut event = ServerEvent::new("response.done");
            event.response_status = Some((*status).to_owned());
            assert!(event.response_failed(), "{status}");
        }
        let mut ok = ServerEvent::new("response.done");
        ok.response_status = Some("completed".to_owned());
        assert!(!ok.response_failed());
        assert!(!ServerEvent::new("response.done").response_failed());
    }

    #[test]
    fn task_ids_prefers_the_batch_list() {
        let mut context = ResponseContext {
            task_id: Some("work_single".to_owned()),
            ..ResponseContext::default()
        };
        assert_eq!(context.task_ids(), vec!["work_single".to_owned()]);
        context.task_ids = vec!["work_a".to_owned(), "work_b".to_owned()];
        assert_eq!(
            context.task_ids(),
            vec!["work_a".to_owned(), "work_b".to_owned()]
        );
    }

    #[test]
    fn only_all_three_terminal_signals_complete_a_context() {
        let mut context = ResponseContext::default();
        for (playback, response, transcript) in [
            (true, true, false),
            (true, false, true),
            (false, true, true),
        ] {
            context.playback_ended = playback;
            context.response_done = response;
            context.transcript_done = transcript;
            assert!(!context.is_complete());
        }
        context.transcript_done = true;
        context.playback_ended = true;
        context.response_done = true;
        assert!(context.is_complete());
    }

    #[test]
    fn playback_start_confirms_an_announcement_or_a_consuming_status_answer() {
        let announcement = ResponseContext {
            origin: ResponseOrigin::Announcement,
            ..ResponseContext::default()
        };
        assert!(announcement.confirms_task_notification_on_playback_start());
        let consuming = ResponseContext {
            origin: ResponseOrigin::Model,
            consumes_task_notification: true,
            ..ResponseContext::default()
        };
        assert!(consuming.confirms_task_notification_on_playback_start());
        assert!(!ResponseContext::default().confirms_task_notification_on_playback_start());
    }

    #[test]
    fn origin_round_trips_through_the_wire_spelling() {
        for origin in [
            ResponseOrigin::Model,
            ResponseOrigin::Announcement,
            ResponseOrigin::Permission,
            ResponseOrigin::Agent,
            ResponseOrigin::Progress,
        ] {
            assert_eq!(ResponseOrigin::from_wire(origin.as_str()), Some(origin));
        }
        assert_eq!(ResponseOrigin::from_wire("nonsense"), None);
    }
}
