//! [`via_voice::VoiceFrontend`] over [`via_realtime::RealtimeSession`].
//!
//! `docs/deviations/phase-5-via-app.md` names this as the one piece phase 5
//! left owed: *"**Binding `via_realtime::RealtimeSession` to it is the
//! remaining phase-5 step**"*. This is that binding, and it is deliberately
//! thin — every method is one call, because `via-realtime` already implemented
//! upstream's `realtime-provider.mjs` and `via-voice` already declared what the
//! voice layer asks of it. Anything more here would be a second copy of one of
//! the two.
//!
//! # Two vocabularies, one translation
//!
//! The two crates describe the same three things with different types, and each
//! difference is deliberate rather than accidental:
//!
//! | `via-voice` | `via-realtime` | Why they differ |
//! | --- | --- | --- |
//! | [`ResponseRequestContext`] — typed | [`ResponseContext`] — an opaque map | `via-realtime` must not know the Gateway's turn model; it stores the object and hands it back |
//! | [`via_voice::ResponseOutcome`] — six booleans | [`via_realtime::ResponseOutcome`] — a `kind` and a `phase` | the voice layer branches on *"was it skipped"*; the session reports *how it ended* |
//! | [`via_voice::ProviderView`] — four facts | [`via_realtime::RealtimeProvider`] — the whole seam | Layer 1's voice half never opens a socket |
//!
//! [`context`] and [`outcome`] are those two translations, and they are pure
//! functions with their own tests so a mis-mapped field is a unit-test failure
//! rather than a silent behaviour change three layers up.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use via_realtime::{
    FunctionOutputOptions as SessionOutputOptions, OutcomeKind, OutcomePhase, RealtimeProvider,
    RealtimeSession, ResponseContext,
};
use via_voice::{
    FunctionOutputOptions, ResponseOrigin, ResponseOutcome, ResponseRequestContext, VoiceFrontend,
};

/// The four facts the voice layer reads off a provider.
///
/// A snapshot rather than a borrow, because [`via_voice::ProviderView`] is
/// `&self`-returning and the session owns its `Arc<dyn RealtimeProvider>`
/// behind a private field.
#[derive(Clone, Debug)]
pub struct ProviderSnapshot {
    key: String,
    label: String,
    input_sample_rate: u32,
    output_sample_rate: u32,
    capabilities: via_voice::ProviderCapabilities,
}

impl ProviderSnapshot {
    /// Take the snapshot.
    #[must_use]
    pub fn of(provider: &Arc<dyn RealtimeProvider>) -> Self {
        let declared = provider.capabilities();
        Self {
            key: provider.key().to_owned(),
            label: provider.label().to_owned(),
            input_sample_rate: provider.input_sample_rate(),
            output_sample_rate: provider.output_sample_rate(),
            capabilities: via_voice::ProviderCapabilities {
                acknowledges_session_update: declared.acknowledges_session_update,
                single_response_slot: declared.single_response_slot,
                response_metadata_correlation: declared.response_metadata_correlation,
                per_response_instructions: declared.per_response_instructions,
                conversation_item_id_echo: declared.conversation_item_id_echo,
            },
        }
    }
}

impl via_voice::ProviderView for ProviderSnapshot {
    fn key(&self) -> &str {
        &self.key
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn input_sample_rate(&self) -> u32 {
        self.input_sample_rate
    }

    fn output_sample_rate(&self) -> u32 {
        self.output_sample_rate
    }

    fn capabilities(&self) -> via_voice::ProviderCapabilities {
        self.capabilities
    }
}

/// Translate the voice layer's typed context into the session's opaque one.
///
/// **External contract** — the `context` object threaded through
/// `speak` / `injectResult` / `ensureResponse` / `sendFunctionOutput`
/// (`server/src/voice/realtime-gateway.mjs`), echoed back on
/// `response.created` as `__voiceContext` and spread into every `transcript.*`
/// and `response.*` frame. The **key names are the wire's**, so they are
/// spelled here exactly as upstream spells them and an absent field is omitted
/// rather than written as `null` — `{...(x ? {k: x} : {})}`.
#[must_use]
pub fn context(request: &ResponseRequestContext) -> ResponseContext {
    let mut context = ResponseContext::new();
    if let Some(turn_id) = request.turn_id.as_deref().filter(|id| !id.is_empty()) {
        context = context.with("turnId", turn_id);
    }
    if let Some(task_id) = request.task_id.as_deref().filter(|id| !id.is_empty()) {
        context = context.with("taskId", task_id);
    }
    if !request.task_ids.is_empty() {
        context = context.with("taskIds", request.task_ids.clone());
    }
    if !request.turn_ids.is_empty() {
        context = context.with("turnIds", request.turn_ids.clone());
    }
    if let Some(authorization_id) = request
        .authorization_id
        .as_deref()
        .filter(|id| !id.is_empty())
    {
        context = context.with("authorizationId", authorization_id);
    }
    if let Some(generation) = request.turn_generation {
        context = context.with("turnGeneration", generation);
    }
    if let Some(sequence) = request.delivery_sequence {
        context = context.with("deliverySequence", sequence);
    }
    if request.consumes_task_notification {
        context = context.with("consumesTaskNotification", true);
    }
    context
}

/// Translate a session outcome into the voice layer's.
///
/// The `None` arm is upstream's *"the queue generation moved on before it
/// ran"*: nothing was attempted, which is a cancellation rather than a failure.
/// Collapsing it into a failure would make the announcement manager abandon a
/// batch that only needed re-queueing.
#[must_use]
pub fn outcome(settled: Option<via_realtime::ResponseOutcome>) -> ResponseOutcome {
    let Some(settled) = settled else {
        return ResponseOutcome {
            cancelled: true,
            ..ResponseOutcome::default()
        };
    };
    ResponseOutcome {
        completed: settled.kind == OutcomeKind::Completed,
        failed: settled.kind == OutcomeKind::Failed,
        skipped: settled.kind == OutcomeKind::Skipped,
        cancelled: settled.kind == OutcomeKind::Cancelled,
        timed_out: settled.kind == OutcomeKind::TimedOut,
        phase: phase(settled.phase),
        response_id: settled.response_id,
        status: settled.status,
        context_injected: false,
    }
}

/// The `phase` half of [`outcome`].
///
/// `no_model_turn` is `via-realtime`'s own variant for a `dictation` session,
/// and `via-voice`'s [`SettlePhase`](via_voice::SettlePhase) has no counterpart
/// — upstream never had one because it has no `SessionMode`. It maps to
/// `Unspecified`, which is what every caller already treats as *"no more
/// specific reason"*.
#[must_use]
fn phase(phase: Option<OutcomePhase>) -> via_voice::SettlePhase {
    use via_voice::SettlePhase as Settle;
    match phase {
        Some(OutcomePhase::Start) => Settle::Start,
        Some(OutcomePhase::Completion) => Settle::Completion,
        Some(OutcomePhase::Input) => Settle::Input,
        Some(OutcomePhase::Correlation) => Settle::Correlation,
        Some(OutcomePhase::Deduplicated) => Settle::Deduplicated,
        Some(OutcomePhase::Inactivity) => Settle::Inactivity,
        Some(OutcomePhase::NoModelTurn) | None => Settle::Unspecified,
    }
}

/// A live realtime session, as the voice layer consumes it.
#[derive(Debug)]
pub struct SessionFrontend {
    session: RealtimeSession,
    provider: ProviderSnapshot,
}

impl SessionFrontend {
    /// Wrap `session`.
    #[must_use]
    pub fn new(session: RealtimeSession) -> Self {
        let provider = ProviderSnapshot::of(session.provider());
        Self { session, provider }
    }

    /// The session underneath, for the event pump.
    #[must_use]
    pub const fn session(&self) -> &RealtimeSession {
        &self.session
    }

    /// The provider snapshot, for `voice.ready` and `audio.delta`.
    #[must_use]
    pub const fn snapshot(&self) -> &ProviderSnapshot {
        &self.provider
    }
}

#[async_trait]
impl VoiceFrontend for SessionFrontend {
    fn ready(&self) -> bool {
        self.session.provider().is_configured()
    }

    fn provider(&self) -> &dyn via_voice::ProviderView {
        &self.provider
    }

    async fn append_audio(&self, audio_base64: &str) {
        // A closed session is not an error a caller can act on: the connection
        // is already tearing down and the next frame will find no engine.
        let _ = self.session.append_audio(audio_base64).await;
    }

    async fn speak(
        &self,
        text: &str,
        origin: ResponseOrigin,
        request: ResponseRequestContext,
    ) -> ResponseOutcome {
        match self
            .session
            .speak(text, into_origin(origin), context(&request), None)
            .await
        {
            Ok(settled) => outcome(settled),
            Err(error) => failed(&error),
        }
    }

    async fn inject_result(
        &self,
        text: &str,
        origin: ResponseOrigin,
        request: ResponseRequestContext,
        inject_context: bool,
    ) -> ResponseOutcome {
        match self
            .session
            .inject_result(text, into_origin(origin), context(&request), inject_context)
            .await
        {
            Ok(Some(injected)) => {
                let mut settled = outcome(injected.outcome);
                settled.context_injected = injected.context_injected;
                settled
            }
            // An empty result never reached the provider.
            Ok(None) => ResponseOutcome::skipped(),
            Err(error) => failed(&error),
        }
    }

    async fn ensure_response(
        &self,
        request: ResponseRequestContext,
        instructions: Option<String>,
    ) -> ResponseOutcome {
        let response =
            instructions.map(|instructions| serde_json::json!({ "instructions": instructions }));
        match self
            .session
            .ensure_response(context(&request), response, None)
            .await
        {
            Ok(settled) => outcome(settled),
            Err(error) => failed(&error),
        }
    }

    async fn send_function_output(
        &self,
        call_id: &str,
        output: &Value,
        request: ResponseRequestContext,
        options: FunctionOutputOptions,
    ) -> ResponseOutcome {
        let mut session_options = if options.create_response {
            SessionOutputOptions::with_response()
        } else {
            SessionOutputOptions::without_response()
        };
        if let Some(instructions) = options.instructions {
            session_options.response = Some(serde_json::json!({ "instructions": instructions }));
        }
        match self
            .session
            .send_function_output(call_id, output.clone(), context(&request), session_options)
            .await
        {
            Ok(settled) => outcome(settled),
            Err(error) => failed(&error),
        }
    }

    async fn append_user_input_context(
        &self,
        parts: &[via_voice::InputPart],
        accompanies_voice: bool,
    ) -> ResponseOutcome {
        let projection = self.projection(parts, accompanies_voice);
        match self.session.append_user_input_context(projection).await {
            Ok(true) => ResponseOutcome::completed(String::new()),
            Ok(false) => ResponseOutcome::skipped(),
            Err(error) => failed(&error),
        }
    }

    async fn send_user_input(
        &self,
        parts: &[via_voice::InputPart],
        request: ResponseRequestContext,
    ) -> ResponseOutcome {
        let projection = self.projection(parts, false);
        match self
            .session
            .send_user_input(projection, context(&request), None)
            .await
        {
            Ok(settled) => outcome(settled),
            Err(error) => failed(&error),
        }
    }

    async fn append_user_context(&self, text: &str) -> ResponseOutcome {
        match self.session.append_user_context(text).await {
            Ok(true) => ResponseOutcome::completed(String::new()),
            Ok(false) => ResponseOutcome::skipped(),
            Err(error) => failed(&error),
        }
    }

    async fn update_agent_context(&self) {
        let _ = self
            .session
            .update_agent_context(via_realtime::AgentContextPatch::default())
            .await;
    }

    async fn cancel(&self) {
        let _ = self.session.cancel().await;
    }

    async fn cancel_responses(
        &self,
        predicate: &(dyn Fn(&ResponseRequestContext, ResponseOrigin) -> bool + Send + Sync),
    ) -> bool {
        // `cancel_responses` needs a `'static` predicate because it crosses
        // into the session's own task; the borrowed one is evaluated here into
        // the only decision the caller can express against an opaque context —
        // the authorization the question was about.
        let authorizations: Vec<ResponseOrigin> = [
            ResponseOrigin::Model,
            ResponseOrigin::Agent,
            ResponseOrigin::Announcement,
            ResponseOrigin::Permission,
            ResponseOrigin::Progress,
        ]
        .into_iter()
        .filter(|origin| predicate(&PROBE, *origin))
        .collect();
        if authorizations.is_empty() {
            return false;
        }
        self.session
            .cancel_responses(move |context, origin| {
                authorizations.contains(&into_voice_origin(origin))
                    && context.authorization_id().is_some()
            })
            .await
            .unwrap_or(false)
    }

    async fn close(&self) {
        let _ = self.session.close().await;
    }
}

/// The sentinel [`SessionFrontend::cancel_responses`] probes a borrowed
/// predicate with.
///
/// It is never sent anywhere: the predicate is asked *"would you cancel a
/// permission response of this origin"*, and only the answer crosses into the
/// session's task.
pub const PROBE_AUTHORIZATION: &str = "via:probe";

/// The probe context, `'static` so it satisfies the predicate's lifetime.
///
/// `#[async_trait]` gives every elided lifetime in the trait method's signature
/// a name, which turns `dyn Fn(&ResponseRequestContext, …)` from a
/// higher-ranked bound into one tied to the reference itself. A borrow of a
/// local therefore does not live long enough; a `'static` one coerces to
/// whatever the caller picked.
static PROBE: std::sync::LazyLock<ResponseRequestContext> =
    std::sync::LazyLock::new(|| ResponseRequestContext {
        authorization_id: Some(PROBE_AUTHORIZATION.to_owned()),
        ..ResponseRequestContext::default()
    });

impl SessionFrontend {
    /// Ask the provider to project input parts, with the composed text as the
    /// fallback.
    ///
    /// **External contract** — `projectUserInput(parts, options)`; the default
    /// projection is `frontendInputProjection(parts, options)`, which lives in
    /// the prompt layer, so `via-realtime` takes the already-composed text
    /// instead.
    fn projection(
        &self,
        parts: &[via_voice::InputPart],
        accompanies_voice: bool,
    ) -> Option<via_realtime::InputProjection> {
        let text = via_voice::input_text(parts);
        let encoded = serde_json::to_value(parts).unwrap_or(Value::Null);
        let options = serde_json::json!({ "accompaniesVoice": accompanies_voice });
        self.session
            .project_user_input(&encoded, &options, Some(&text))
    }
}

/// A transport failure, as the voice layer's outcome.
///
/// `phase: Completion` rather than `Start`: by the time the session answers an
/// error the request is already past the point a guard could have declined it,
/// and the caller's only question is whether to retry.
fn failed(error: &via_realtime::RealtimeError) -> ResponseOutcome {
    ResponseOutcome {
        failed: true,
        phase: via_voice::SettlePhase::Completion,
        status: Some(error.to_string()),
        ..ResponseOutcome::default()
    }
}

/// `via_voice::ResponseOrigin` → `via_realtime::ResponseOrigin`.
///
/// The two enums differ by one variant, and the difference is deliberate rather
/// than an oversight. `via-realtime`'s is closed at four *"because every branch
/// is a comparison against one of these four"* — the branches inside the session
/// machine. `via-voice` adds [`ResponseOrigin::Progress`], which is a **Gateway**
/// distinction: a spoken progress note and a delegated start announcement are
/// both `agent` on the wire, and only the announcement window tells them apart.
/// So `Progress` maps to `Agent` here, which is what upstream's own
/// `speak(message, 'agent', …)` for a progress check does
/// (`realtime-gateway.mjs:831-859`).
#[must_use]
pub const fn into_origin(origin: ResponseOrigin) -> via_realtime::ResponseOrigin {
    match origin {
        ResponseOrigin::Model => via_realtime::ResponseOrigin::Model,
        ResponseOrigin::Agent | ResponseOrigin::Progress => via_realtime::ResponseOrigin::Agent,
        ResponseOrigin::Announcement => via_realtime::ResponseOrigin::Announcement,
        ResponseOrigin::Permission => via_realtime::ResponseOrigin::Permission,
    }
}

/// `via_realtime::ResponseOrigin` → `via_voice::ResponseOrigin`.
#[must_use]
pub const fn into_voice_origin(origin: via_realtime::ResponseOrigin) -> ResponseOrigin {
    match origin {
        via_realtime::ResponseOrigin::Model => ResponseOrigin::Model,
        via_realtime::ResponseOrigin::Agent => ResponseOrigin::Agent,
        via_realtime::ResponseOrigin::Announcement => ResponseOrigin::Announcement,
        via_realtime::ResponseOrigin::Permission => ResponseOrigin::Permission,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn an_empty_context_carries_no_keys_at_all() {
        assert!(context(&ResponseRequestContext::default()).is_empty());
    }

    #[test]
    fn a_blank_turn_id_is_omitted_rather_than_written_blank() {
        let request = ResponseRequestContext {
            turn_id: Some(String::new()),
            task_id: Some(String::new()),
            authorization_id: Some(String::new()),
            ..ResponseRequestContext::default()
        };
        assert!(
            context(&request).is_empty(),
            "`{{...(x ? {{k: x}} : {{}})}}` drops an empty string, and so must this",
        );
    }

    #[test]
    fn the_context_keys_are_the_wires_own_spelling() {
        let request = ResponseRequestContext {
            turn_id: Some("voice-1".to_owned()),
            task_id: Some("work_1".to_owned()),
            task_ids: vec!["work_1".to_owned(), "work_2".to_owned()],
            turn_ids: vec!["voice-1".to_owned()],
            authorization_id: Some("auth_1".to_owned()),
            turn_generation: Some(3),
            delivery_sequence: Some(7),
            consumes_task_notification: true,
        };
        let context = context(&request);
        assert_eq!(context.turn_id(), Some("voice-1"));
        assert_eq!(context.task_id(), Some("work_1"));
        assert_eq!(context.authorization_id(), Some("auth_1"));
        assert_eq!(context.get("turnGeneration"), Some(&Value::from(3)));
        assert_eq!(context.get("deliverySequence"), Some(&Value::from(7)));
        assert_eq!(
            context.get("consumesTaskNotification"),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            context.get("taskIds"),
            Some(&serde_json::json!(["work_1", "work_2"]))
        );
        assert_eq!(
            context.get("turnIds"),
            Some(&serde_json::json!(["voice-1"]))
        );
    }

    #[test]
    fn a_generation_of_zero_is_carried_because_it_is_a_real_generation() {
        let request = ResponseRequestContext {
            turn_generation: Some(0),
            ..ResponseRequestContext::default()
        };
        assert_eq!(
            context(&request).get("turnGeneration"),
            Some(&Value::from(0))
        );
    }

    fn settled(kind: OutcomeKind) -> via_realtime::ResponseOutcome {
        serde_json::from_value(serde_json::json!({ "kind": kind }))
            .expect("a bare kind is a valid outcome")
    }

    #[test]
    fn each_kind_sets_exactly_one_flag() {
        let cases = [
            (OutcomeKind::Completed, [true, false, false, false, false]),
            (OutcomeKind::Failed, [false, true, false, false, false]),
            (OutcomeKind::Skipped, [false, false, true, false, false]),
            (OutcomeKind::Cancelled, [false, false, false, true, false]),
            (OutcomeKind::TimedOut, [false, false, false, false, true]),
        ];
        for (kind, expected) in cases {
            let translated = outcome(Some(settled(kind)));
            assert_eq!(
                [
                    translated.completed,
                    translated.failed,
                    translated.skipped,
                    translated.cancelled,
                    translated.timed_out,
                ],
                expected,
                "{kind:?}",
            );
        }
    }

    #[test]
    fn a_stale_queue_generation_reads_as_cancelled_not_failed() {
        let translated = outcome(None);
        assert!(translated.cancelled);
        assert!(
            !translated.failed,
            "abandoning a batch that only needed re-queueing is the bug this prevents",
        );
    }

    #[test]
    fn every_phase_maps_and_no_model_turn_degrades_to_unspecified() {
        use via_voice::SettlePhase as Settle;
        let cases = [
            (OutcomePhase::Start, Settle::Start),
            (OutcomePhase::Completion, Settle::Completion),
            (OutcomePhase::Input, Settle::Input),
            (OutcomePhase::Correlation, Settle::Correlation),
            (OutcomePhase::Deduplicated, Settle::Deduplicated),
            (OutcomePhase::Inactivity, Settle::Inactivity),
            (OutcomePhase::NoModelTurn, Settle::Unspecified),
        ];
        for (from, into) in cases {
            assert_eq!(phase(Some(from)), into, "{from:?}");
        }
        assert_eq!(phase(None), Settle::Unspecified);
    }

    #[test]
    fn the_four_shared_origins_round_trip() {
        for origin in [
            ResponseOrigin::Model,
            ResponseOrigin::Agent,
            ResponseOrigin::Announcement,
            ResponseOrigin::Permission,
        ] {
            assert_eq!(into_voice_origin(into_origin(origin)), origin);
        }
    }

    #[test]
    fn progress_speaks_as_agent_because_that_is_what_the_wire_carries() {
        assert_eq!(
            into_origin(ResponseOrigin::Progress),
            via_realtime::ResponseOrigin::Agent,
        );
    }

    #[test]
    fn a_transport_failure_carries_its_sentence() {
        let translated = failed(&via_realtime::RealtimeError::ConnectionClosed {
            label: "Mock".to_owned(),
        });
        assert!(translated.failed);
        assert_eq!(translated.phase, via_voice::SettlePhase::Completion);
        assert!(translated.status.is_some_and(|status| !status.is_empty()));
    }
}
