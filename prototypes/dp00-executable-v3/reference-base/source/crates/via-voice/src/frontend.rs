//! The realtime frontend, as the voice layer consumes it.
//!
//! The implementation is `via-realtime`'s `RealtimeFrontend`, ported from
//! `server/src/voice/realtime-provider.mjs`. This is the surface the gateway
//! and the announcement manager actually call, expressed as a trait so both
//! are testable with no socket, no model and no network — which is what
//! `docs/architecture.md` §15 phase 5 means by *`via chat` works end to end*.
//!
//! # Why the outcome is a value and not a `Result`
//!
//! Upstream's `enqueueResponse` resolves — never rejects — with one of six
//! shapes: completed, failed, skipped, cancelled, timed out, or a correlation
//! conflict. The distinction matters to the caller: a *skipped* announcement
//! must be retried, a *completed* one must wait for playback, and a
//! *cancelled* one must be released. Collapsing them into `Err` would lose
//! exactly the information the Injection Gate is built on.

use async_trait::async_trait;

use crate::response::ResponseOrigin;

/// Why a response settled the way it did.
///
/// **External contract** — the `phase` field of `realtime-provider.mjs`'s
/// `settlePending` calls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SettlePhase {
    /// Nothing more specific.
    #[default]
    Unspecified,
    /// Waiting for `response.created`.
    Start,
    /// Waiting for `response.done`.
    Completion,
    /// Building the conversation item that precedes the response.
    Input,
    /// A second response was already waiting for `response.created`.
    Correlation,
    /// A `should_speak` / `should_create` predicate declined.
    Deduplicated,
    /// The response streamed nothing for the inactivity budget.
    Inactivity,
}

/// How one enqueued response ended.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResponseOutcome {
    /// The provider generated the whole response.
    pub completed: bool,
    /// The provider refused it, or `response.done` reported a failure.
    pub failed: bool,
    /// A predicate declined to create it. Not a failure — the caller asked for
    /// it to be skipped.
    pub skipped: bool,
    /// The queue generation moved on before it ran.
    pub cancelled: bool,
    /// It never started, or never finished.
    pub timed_out: bool,
    /// Which phase it settled in.
    pub phase: SettlePhase,
    /// The provider's response id, once one exists.
    pub response_id: Option<String>,
    /// The `response.done` status, for a failure.
    pub status: Option<String>,
    /// Whether the accompanying context item was written into the
    /// conversation. Only [`VoiceFrontend::inject_result`] sets it.
    pub context_injected: bool,
}

impl ResponseOutcome {
    /// A response the provider generated in full.
    #[must_use]
    pub fn completed(response_id: impl Into<String>) -> Self {
        Self {
            completed: true,
            response_id: Some(response_id.into()),
            ..Self::default()
        }
    }

    /// A response a predicate declined to create.
    #[must_use]
    pub fn skipped() -> Self {
        Self {
            skipped: true,
            phase: SettlePhase::Deduplicated,
            ..Self::default()
        }
    }

    /// A response the provider refused.
    #[must_use]
    pub fn failed(phase: SettlePhase) -> Self {
        Self {
            failed: true,
            phase,
            ..Self::default()
        }
    }
}

/// The correlation metadata attached to a response the Gateway creates.
///
/// **External contract** — the `context` argument threaded through
/// `speak` / `injectResult` / `ensureResponse` / `sendFunctionOutput`, echoed
/// back on `response.created` as `__voiceContext` and spread into every
/// `transcript.*` and `response.*` frame.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResponseRequestContext {
    /// The turn this response belongs to.
    pub turn_id: Option<String>,
    /// The single Work, when there is exactly one.
    pub task_id: Option<String>,
    /// Every Work in an announcement batch.
    pub task_ids: Vec<String>,
    /// Every turn in an announcement batch.
    pub turn_ids: Vec<String>,
    /// The permission being asked about.
    pub authorization_id: Option<String>,
    /// The turn generation, for staleness.
    pub turn_generation: Option<i64>,
    /// The announcement batch's sequence number.
    pub delivery_sequence: Option<u64>,
    /// Whether answering this also discharges a Work notification.
    pub consumes_task_notification: bool,
}

/// Options for [`VoiceFrontend::send_function_output`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FunctionOutputOptions {
    /// Whether to create a response after writing the output item.
    ///
    /// `false` for a stale call and for `enter_sleep`: the model must see the
    /// output, but nothing should be said about it.
    pub create_response: bool,
    /// Per-response instructions for that response.
    pub instructions: Option<String>,
}

impl FunctionOutputOptions {
    /// Write the output and let the provider close the turn.
    #[must_use]
    pub fn with_response() -> Self {
        Self {
            create_response: true,
            instructions: None,
        }
    }

    /// Write the output and say nothing.
    #[must_use]
    pub fn silent() -> Self {
        Self {
            create_response: false,
            instructions: None,
        }
    }

    /// Attach per-response instructions.
    #[must_use]
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }
}

/// The realtime session the voice layer drives.
///
/// Every method that creates a response goes through the frontend's serial
/// output queue, so the ordering invariant `docs/architecture.md` §11 calls
/// *the keyed serial executor* holds without this crate taking a lock.
#[async_trait]
pub trait VoiceFrontend: Send + Sync {
    /// Whether the session is configured and usable.
    fn ready(&self) -> bool;

    /// The provider behind this session.
    fn provider(&self) -> &dyn crate::provider::ProviderView;

    /// The provider's five behavioural flags.
    fn capabilities(&self) -> crate::provider::ProviderCapabilities {
        self.provider().capabilities()
    }

    /// Forward a chunk of captured audio.
    async fn append_audio(&self, audio_base64: &str);

    /// Speak `text` as a fresh response.
    ///
    /// `should_speak` is evaluated when the response reaches the front of the
    /// queue, not when it is enqueued — which is what lets the delegated-start
    /// announcement check, at the last possible moment, whether the
    /// acknowledgement transcript already said the same thing.
    async fn speak(
        &self,
        text: &str,
        origin: ResponseOrigin,
        context: ResponseRequestContext,
    ) -> ResponseOutcome;

    /// Write `text` into the conversation and speak it.
    ///
    /// `inject_context` is false on a retry whose context item already landed:
    /// re-writing it would put the same results into the model's conversation
    /// twice.
    async fn inject_result(
        &self,
        text: &str,
        origin: ResponseOrigin,
        context: ResponseRequestContext,
        inject_context: bool,
    ) -> ResponseOutcome;

    /// Ask the model to produce a response for a turn that already has its
    /// input, optionally with per-response instructions.
    async fn ensure_response(
        &self,
        context: ResponseRequestContext,
        instructions: Option<String>,
    ) -> ResponseOutcome;

    /// Write a tool result back, and optionally let the model answer it.
    async fn send_function_output(
        &self,
        call_id: &str,
        output: &serde_json::Value,
        context: ResponseRequestContext,
        options: FunctionOutputOptions,
    ) -> ResponseOutcome;

    /// Write user input parts into the conversation without answering them.
    async fn append_user_input_context(
        &self,
        parts: &[crate::input::InputPart],
        accompanies_voice: bool,
    ) -> ResponseOutcome;

    /// Write user input parts into the conversation and answer them.
    async fn send_user_input(
        &self,
        parts: &[crate::input::InputPart],
        context: ResponseRequestContext,
    ) -> ResponseOutcome;

    /// Write a plain text note into the conversation without answering it.
    async fn append_user_context(&self, text: &str) -> ResponseOutcome;

    /// Rebuild the session instructions from a changed agent context.
    async fn update_agent_context(&self);

    /// Cancel everything in flight. Barge-in and a new user turn both use it.
    async fn cancel(&self);

    /// Cancel only the responses matching `predicate`.
    ///
    /// Used when a permission is resolved elsewhere: the question about *that*
    /// authorization is dropped, and nothing else is.
    async fn cancel_responses(
        &self,
        predicate: &(dyn Fn(&ResponseRequestContext, ResponseOrigin) -> bool + Send + Sync),
    ) -> bool;

    /// Close the socket and settle everything pending.
    async fn close(&self);
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn a_skipped_outcome_is_not_a_failure() {
        let outcome = ResponseOutcome::skipped();
        assert!(outcome.skipped);
        assert!(!outcome.failed);
        assert!(!outcome.completed);
        assert_eq!(outcome.phase, SettlePhase::Deduplicated);
    }

    #[test]
    fn a_completed_outcome_carries_its_response_id() {
        let outcome = ResponseOutcome::completed("resp_1");
        assert!(outcome.completed);
        assert_eq!(outcome.response_id.as_deref(), Some("resp_1"));
        assert!(!outcome.context_injected);
    }

    #[test]
    fn function_output_options_default_to_silent() {
        assert!(!FunctionOutputOptions::default().create_response);
        assert!(!FunctionOutputOptions::silent().create_response);
        assert!(FunctionOutputOptions::with_response().create_response);
        assert_eq!(
            FunctionOutputOptions::with_response()
                .instructions("say it")
                .instructions
                .as_deref(),
            Some("say it"),
        );
    }
}
