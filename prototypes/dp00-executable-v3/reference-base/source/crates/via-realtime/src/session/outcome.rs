//! What a session hands back: the origin of a response, the context it carries,
//! how it ended, and the events it emits along the way.
//!
//! Upstream stores all four as loose properties: `pending.origin`,
//! `pending.context`, the ad-hoc object `settlePending` resolves with, and three
//! `__voice*` properties bolted onto the event object itself
//! (`realtime-provider.mjs:596-597`, `:648`, `:657-658`). Typing them is the one
//! place this port genuinely improves on the original — an outcome that says
//! `{ timedOut: true }` and one that says `{ failed: true }` are the same shape
//! in JavaScript and cannot be confused here.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::RealtimeError;

/// Who asked for a response.
///
/// Upstream's `origin`, a free string with four values the Gateway branches on
/// (`realtime-gateway.mjs:89`, `:690`, `:741`, `:901-906`, `:1278-1297`,
/// `:1366`). Closed here because every branch is a comparison against one of
/// these four.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseOrigin {
    /// The model answering the user. The default for any response the session
    /// did not itself request — including an automatic server-VAD turn.
    #[default]
    Model,
    /// The Gateway speaking on a backend's behalf: a presentation, a progress
    /// update, a refusal.
    Agent,
    /// A finished background result, delivered through the announcement window.
    Announcement,
    /// A spoken authorization question.
    Permission,
}

impl ResponseOrigin {
    /// The wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Agent => "agent",
            Self::Announcement => "announcement",
            Self::Permission => "permission",
        }
    }
}

impl core::fmt::Display for ResponseOrigin {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What the caller attached to a response, handed back when it is correlated.
///
/// Deliberately opaque. Upstream's `context` is an arbitrary object assembled by
/// the Gateway — `turnId`, `taskId`, `authorizationId`, `turnGeneration` and
/// more — and `via-realtime` never reads any of it: it stores the object, and
/// gives it back on the event the response turned out to be. Typing the fields
/// here would make this crate depend on the Gateway's turn model, which is
/// exactly the coupling the correlation seam exists to avoid.
///
/// The three accessors below exist because they are the three keys the Gateway
/// reads on *every* correlated event, not because this crate interprets them.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResponseContext(Map<String, Value>);

impl ResponseContext {
    /// An empty context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether anything was attached.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Attach a value, builder style.
    #[must_use]
    pub fn with(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.0.insert(key.into(), value.into());
        self
    }

    /// Read a value back.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    /// Read a string value back.
    #[must_use]
    pub fn string(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(Value::as_str)
    }

    /// `turnId` — the user turn this response belongs to.
    #[must_use]
    pub fn turn_id(&self) -> Option<&str> {
        self.string("turnId")
    }

    /// `taskId` — the Work this response is speaking about.
    #[must_use]
    pub fn task_id(&self) -> Option<&str> {
        self.string("taskId")
    }

    /// `authorizationId` — the permission request this response is asking about.
    #[must_use]
    pub fn authorization_id(&self) -> Option<&str> {
        self.string("authorizationId")
    }

    /// The underlying map.
    #[must_use]
    pub fn as_map(&self) -> &Map<String, Value> {
        &self.0
    }
}

impl From<Map<String, Value>> for ResponseContext {
    fn from(map: Map<String, Value>) -> Self {
        Self(map)
    }
}

/// How a response ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutcomeKind {
    /// `response.done` arrived with a status that is not one of the three
    /// catalogued failures.
    Completed,
    /// The provider refused it, rejected its input, or ended it in a failing
    /// status.
    Failed,
    /// The caller cancelled it, or the session was reset under it.
    Cancelled,
    /// A guard declined it before anything was written. Nothing reached the
    /// provider.
    Skipped,
    /// A timer fired: either the response never started, or it stopped
    /// producing output.
    TimedOut,
}

/// Where in a response's life it ended.
///
/// Upstream's `phase` string. The seven values are the ones `settlePending` is
/// ever called with, plus [`NoModelTurn`](Self::NoModelTurn), which is VIA's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomePhase {
    /// Before `response.created` — it never started.
    Start,
    /// After `response.created` — it was cancelled mid-flight.
    Completion,
    /// Two starts were outstanding at once, so neither could be correlated.
    Correlation,
    /// A late guard declined it.
    Deduplicated,
    /// The conversation item it depended on was rejected.
    Input,
    /// It started and then stopped producing output.
    Inactivity,
    /// **VIA's own.** The session mode mounts no model turn, so there is nothing
    /// to create a response on — `docs/architecture.md` §2, `dictation`.
    NoModelTurn,
}

/// What a response-creating call resolved to.
///
/// Field-for-field upstream's outcome objects, with the union of their optional
/// keys. The mapping:
///
/// | Upstream | Here |
/// | --- | --- |
/// | `{ completed: true, responseId }` | `Completed` + `response_id` |
/// | `{ failed: true, responseId, status }` | `Failed` + `response_id` + `status` |
/// | `{ failed: true, phase: 'input', error }` | `Failed` + `Input` + `error` |
/// | `{ failed: true, phase: 'correlation', error }` | `Failed` + `Correlation` + `error` |
/// | `{ cancelled: true, phase: 'start' }` | `Cancelled` + `Start` |
/// | `{ cancelled: true, phase: 'completion' }` | `Cancelled` + `Completion` |
/// | `{ cancelled: true }` | `Cancelled`, no phase |
/// | `{ skipped: true, phase: 'deduplicated' }` | `Skipped` + `Deduplicated` |
/// | `{ timedOut: true, phase: 'start' }` | `TimedOut` + `Start` |
/// | `{ timedOut: true, phase: 'inactivity', responseId }` | `TimedOut` + `Inactivity` + `response_id` |
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseOutcome {
    /// How it ended.
    pub kind: OutcomeKind,
    /// Where it ended, when that is meaningful.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<OutcomePhase>,
    /// The provider's response id, when one was ever assigned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_id: Option<String>,
    /// The `response.status` a `response.done` reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// The sentence that explains a failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ResponseOutcome {
    pub(crate) fn new(kind: OutcomeKind) -> Self {
        Self {
            kind,
            phase: None,
            response_id: None,
            status: None,
            error: None,
        }
    }

    pub(crate) fn with_phase(mut self, phase: OutcomePhase) -> Self {
        self.phase = Some(phase);
        self
    }

    pub(crate) fn with_response_id(mut self, response_id: &str) -> Self {
        // Upstream writes `responseId: id` where `id` may be `''`; an empty id
        // is "no id", so it is dropped rather than carried as a blank string.
        if !response_id.is_empty() {
            self.response_id = Some(response_id.to_owned());
        }
        self
    }

    pub(crate) fn with_status(mut self, status: Option<&str>) -> Self {
        self.status = status.map(str::to_owned);
        self
    }

    pub(crate) fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self
    }

    /// Whether the response ran to completion.
    #[must_use]
    pub fn is_completed(&self) -> bool {
        self.kind == OutcomeKind::Completed
    }

    /// Whether the response failed.
    #[must_use]
    pub fn is_failed(&self) -> bool {
        self.kind == OutcomeKind::Failed
    }

    /// Whether the response was cancelled.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.kind == OutcomeKind::Cancelled
    }

    /// Whether a guard declined the response before anything was written.
    #[must_use]
    pub fn is_skipped(&self) -> bool {
        self.kind == OutcomeKind::Skipped
    }

    /// Whether a timer ended the response.
    #[must_use]
    pub fn is_timed_out(&self) -> bool {
        self.kind == OutcomeKind::TimedOut
    }
}

/// What [`RealtimeSession::inject_result`](crate::RealtimeSession::inject_result)
/// answers with.
///
/// Upstream returns `{ ...(outcome || {}), contextInjected }`, so the flag is
/// present even when the queue was stale and there is no outcome at all — which
/// is why it is a field beside an `Option` rather than a field inside one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InjectOutcome {
    /// How the response ended, or `None` when the queue generation had already
    /// moved on and nothing was attempted.
    pub outcome: Option<ResponseOutcome>,
    /// Whether the result reached the conversation as an item.
    ///
    /// Load-bearing on the announcement path: a result whose context was
    /// injected is in the conversation even if the spoken response failed, so a
    /// retry must not inject it twice.
    pub context_injected: bool,
}

/// A normalized provider event, with whatever the session knows about it.
///
/// Upstream bolts the same three facts onto the event object as `__voiceOrigin`,
/// `__voiceContext` and `__voiceRetried`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProviderEvent {
    /// The event as the dialect normalized it.
    pub event: Value,
    /// Who asked for the response this event belongs to.
    ///
    /// [`ResponseOrigin::Model`] both when the model genuinely started the
    /// response and when the session could not correlate it — upstream's
    /// `pending?.origin || 'model'`.
    pub origin: ResponseOrigin,
    /// What the caller attached to that response. Empty when uncorrelated.
    pub context: ResponseContext,
    /// Whether the session handled this event internally by replaying a refused
    /// `response.create`.
    ///
    /// Upstream's comment says what it is for: *"marks the event as internally
    /// handled: the gateway must not surface a transparently retried refusal to
    /// the user."*
    pub retried: bool,
}

impl ProviderEvent {
    /// The event's `type`.
    #[must_use]
    pub fn kind(&self) -> &str {
        self.event
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default()
    }
}

/// A structured observation the session makes about itself.
///
/// Upstream's `onDiagnostic` callback payload
/// (`realtime-provider.mjs:686-693`). The field order is its object literal's,
/// because the payload is logged as JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    /// The diagnostic name. Today there is exactly one:
    /// `realtime.response_timeout`.
    pub event: &'static str,
    /// The provider key.
    pub provider: String,
    /// The response that timed out.
    pub response_id: String,
    /// Which watchdog fired. Today: `inactivity`.
    pub phase: &'static str,
    /// How long output had been silent.
    pub inactivity_ms: u64,
    /// How long the response had been running in total.
    pub elapsed_ms: u64,
}

/// The diagnostic name for an inactive response.
pub const DIAGNOSTIC_RESPONSE_TIMEOUT: &str = "realtime.response_timeout";

/// Everything the session reports.
///
/// Upstream's four callbacks — `onEvent`, `onError`, `onDiagnostic`, `onClose` —
/// as one ordered stream, because the order between them matters: an error that
/// arrives before `Closed` was the reason for the close.
#[derive(Debug)]
pub enum SessionEvent {
    /// A normalized provider event. Upstream `onEvent`.
    Provider(Box<ProviderEvent>),
    /// Something the session failed at. Upstream `onError`.
    ///
    /// A failure whose [`is_provider_event`](RealtimeError::is_provider_event)
    /// is true is *not* emitted here — it is already reported as the response
    /// outcome, and upstream is explicit that it must not be reported twice.
    Error(RealtimeError),
    /// A structured observation. Upstream `onDiagnostic`.
    Diagnostic(Diagnostic),
    /// The socket closed. Upstream `onClose`, and always the last event.
    Closed,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn an_empty_response_id_is_dropped_rather_than_carried_blank() {
        let outcome = ResponseOutcome::new(OutcomeKind::Completed).with_response_id("");
        assert_eq!(outcome.response_id, None);
        let outcome = ResponseOutcome::new(OutcomeKind::Completed).with_response_id("r");
        assert_eq!(outcome.response_id.as_deref(), Some("r"));
    }

    #[test]
    fn an_outcome_serializes_without_its_absent_keys() {
        let outcome = ResponseOutcome::new(OutcomeKind::TimedOut).with_phase(OutcomePhase::Start);
        assert_eq!(
            serde_json::to_value(&outcome).expect("serialize"),
            json!({ "kind": "timedOut", "phase": "start" })
        );
    }

    #[test]
    fn the_context_is_opaque_but_the_three_gateway_keys_read_back() {
        let context = ResponseContext::new()
            .with("turnId", "voice-100-1")
            .with("taskId", "job_1")
            .with("authorizationId", "auth-1")
            .with("turnGeneration", 3);
        assert_eq!(context.turn_id(), Some("voice-100-1"));
        assert_eq!(context.task_id(), Some("job_1"));
        assert_eq!(context.authorization_id(), Some("auth-1"));
        assert_eq!(context.get("turnGeneration"), Some(&json!(3)));
        assert_eq!(context.string("turnGeneration"), None);
        assert!(!context.is_empty());
        assert!(ResponseContext::new().is_empty());
    }

    #[test]
    fn the_context_serializes_as_the_bare_object() {
        let context = ResponseContext::new().with("turnId", "t");
        assert_eq!(
            serde_json::to_value(&context).expect("serialize"),
            json!({ "turnId": "t" })
        );
    }

    #[test]
    fn the_origin_defaults_to_model() {
        assert_eq!(ResponseOrigin::default(), ResponseOrigin::Model);
        assert_eq!(ResponseOrigin::Announcement.as_str(), "announcement");
    }

    #[test]
    fn the_outcome_predicates_agree_with_the_kind() {
        let cases = [
            (OutcomeKind::Completed, [true, false, false, false, false]),
            (OutcomeKind::Failed, [false, true, false, false, false]),
            (OutcomeKind::Cancelled, [false, false, true, false, false]),
            (OutcomeKind::Skipped, [false, false, false, true, false]),
            (OutcomeKind::TimedOut, [false, false, false, false, true]),
        ];
        for (kind, expected) in cases {
            let outcome = ResponseOutcome::new(kind);
            assert_eq!(
                [
                    outcome.is_completed(),
                    outcome.is_failed(),
                    outcome.is_cancelled(),
                    outcome.is_skipped(),
                    outcome.is_timed_out(),
                ],
                expected,
                "{kind:?}"
            );
        }
    }

    #[test]
    fn a_provider_event_reports_its_type() {
        let event = ProviderEvent {
            event: json!({ "type": "response.created" }),
            origin: ResponseOrigin::Agent,
            context: ResponseContext::new(),
            retried: false,
        };
        assert_eq!(event.kind(), "response.created");
        let untyped = ProviderEvent {
            event: json!({}),
            ..event
        };
        assert_eq!(untyped.kind(), "");
    }
}
