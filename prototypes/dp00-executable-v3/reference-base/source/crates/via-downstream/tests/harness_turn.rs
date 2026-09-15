//! A prompt turn, end to end through the traits.
//!
//! `docs/architecture.md` §15 ends phase 2 when *"a scripted fake ACP agent
//! completes a prompt turn through the trait"*. This is that test, driven only
//! through [`DownstreamAgent`] and [`HarnessSession`] — nothing here reaches
//! past the seam into the double.

mod common;

use std::sync::Arc;

use futures::StreamExt;
use pretty_assertions::assert_eq;
use serde_json::json;
use via_downstream::testing::{ScriptedHarness, ScriptedTurn};
use via_downstream::{
    ActivityStatus, CancelOutcome, CancelRoute, CancelScope, CancelTarget, DownstreamAgent,
    HarnessError, HarnessHealth, HarnessRegistry, HarnessStatusCode, PromptAttachment,
    PromptRequest, RawSessionUpdate, SessionEvent, SessionKey, StopReason, UPDATE_TOOL_CALL,
};

/// The whole path: register, resolve, open, prompt, read the outcome.
#[tokio::test]
async fn a_scripted_harness_completes_a_prompt_turn_through_the_trait() {
    let harness = ScriptedHarness::builder("codex")
        .turn(ScriptedTurn::completed("the answer"))
        .build()
        .expect("declares");

    let mut registry = HarnessRegistry::new();
    registry.register(Arc::new(harness)).expect("registers");

    let key = SessionKey::coordinator("codex", "ana");
    let session = registry.open(&key).await.expect("opens");
    assert_eq!(session.session_id(), "codex-session-1");

    let outcome = session
        .prompt(PromptRequest::text("ana", "what is the answer?").with_work_id("work-1"))
        .await
        .expect("the turn completes");
    assert_eq!(outcome.content(), "the answer");
    assert_eq!(outcome.stop_reason(), &StopReason::EndTurn);
    assert!(outcome.is_end_turn());
}

/// The request the harness received is the request that was sent, work id and
/// attachments included.
#[tokio::test]
async fn the_request_reaches_the_harness_intact() {
    let harness = Arc::new(
        ScriptedHarness::builder("claude")
            .turn(ScriptedTurn::completed("ok"))
            .build()
            .expect("declares"),
    );
    let session = harness
        .open(&SessionKey::coordinator("claude", "ana"))
        .await
        .expect("opens");

    let request = PromptRequest::text("ana", "look at this")
        .with_work_id("work-9")
        .with_attachment(PromptAttachment {
            url: "data:image/png;base64,AAAA".to_owned(),
            filename: "shot.png".to_owned(),
            mime: "image/png".to_owned(),
        });
    session.prompt(request.clone()).await.expect("completes");

    let observed = harness.sessions();
    assert_eq!(observed.len(), 1);
    assert_eq!(observed[0].prompts(), vec![request]);
    assert_eq!(observed[0].key(), SessionKey::coordinator("claude", "ana"));
}

/// Events emitted during a turn arrive on the stream, already projected — and
/// a subscriber that arrives afterwards sees the same sequence.
#[tokio::test]
async fn a_turn_emits_projected_events() {
    let harness = ScriptedHarness::builder("codex")
        .turn(
            ScriptedTurn::completed("done")
                .emitting(RawSessionUpdate {
                    session_update: UPDATE_TOOL_CALL.to_owned(),
                    tool_call_id: Some("call-1".to_owned()),
                    name: Some("grep".to_owned()),
                    title: None,
                    status: Some("in_progress".to_owned()),
                    raw_input: Some(json!({ "path": "/srv/via" })),
                    entries: Vec::new(),
                })
                // Reasoning is silent, so this adds no event.
                .emitting(RawSessionUpdate {
                    session_update: "agent_thought_chunk".to_owned(),
                    ..RawSessionUpdate::default()
                })
                .emitting(RawSessionUpdate {
                    session_update: "agent_message_chunk".to_owned(),
                    ..RawSessionUpdate::default()
                }),
        )
        .build()
        .expect("declares");
    let session = harness
        .open(&SessionKey::coordinator("codex", "ana"))
        .await
        .expect("opens");

    let early = session.events();
    session
        .prompt(PromptRequest::text("ana", "go"))
        .await
        .expect("completes");
    let late = session.events();

    let from_early: Vec<SessionEvent> = early.take(2).collect().await;
    let from_late: Vec<SessionEvent> = late.take(2).collect().await;
    assert_eq!(from_early, from_late);
    assert_eq!(from_early.len(), 2);
    assert_eq!(from_early[0].id(), Some("call-1"));
    assert_eq!(from_early[0].status(), ActivityStatus::InProgress);
    assert_eq!(from_early[1], SessionEvent::Text);
}

/// A cancelled turn is an error, not a completed one — through the trait, not
/// only through the constructor.
#[tokio::test]
async fn a_cancelled_turn_fails_the_prompt() {
    let harness = ScriptedHarness::builder("codex")
        .turn(ScriptedTurn::cancelled())
        .build()
        .expect("declares");
    let session = harness
        .open(&SessionKey::coordinator("codex", "ana"))
        .await
        .expect("opens");

    let error = session
        .prompt(PromptRequest::text("ana", "go"))
        .await
        .expect_err("a cancelled turn is not a completed one");
    assert!(error.is_cancelled());
    assert_eq!(error.code(), "VIA_HARNESS_CANCELLED");
}

/// A transport failure arrives as `AgentError` with its status intact.
#[tokio::test]
async fn a_transport_failure_keeps_its_status() {
    let harness = ScriptedHarness::builder("codex")
        .turn(ScriptedTurn::failing("upstream said no", 502))
        .build()
        .expect("declares");
    let session = harness
        .open(&SessionKey::coordinator("codex", "ana"))
        .await
        .expect("opens");

    let error = session
        .prompt(PromptRequest::text("ana", "go"))
        .await
        .expect_err("the turn fails");
    assert_eq!(error.http_status(), 502);
    match &error {
        HarnessError::Agent {
            message, protocol, ..
        } => {
            assert_eq!(message, "upstream said no");
            assert_eq!(protocol, "codex");
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

/// Turns are consumed in order, and running past the end of the script is a
/// failure rather than a silent success.
#[tokio::test]
async fn the_script_is_consumed_in_order_and_then_runs_out() {
    let harness = Arc::new(
        ScriptedHarness::builder("codex")
            .turn(ScriptedTurn::completed("first"))
            .turn(ScriptedTurn::stopping("second", StopReason::MaxTokens))
            .build()
            .expect("declares"),
    );
    let session = harness
        .open(&SessionKey::coordinator("codex", "ana"))
        .await
        .expect("opens");

    assert_eq!(harness.remaining_turns(), 2);
    let first = session
        .prompt(PromptRequest::text("ana", "one"))
        .await
        .expect("completes");
    assert_eq!(first.content(), "first");
    assert!(first.is_end_turn());

    let second = session
        .prompt(PromptRequest::text("ana", "two"))
        .await
        .expect("completes");
    assert_eq!(second.content(), "second");
    assert_eq!(second.stop_reason(), &StopReason::MaxTokens);
    assert!(!second.is_end_turn());

    assert_eq!(harness.remaining_turns(), 0);
    let error = session
        .prompt(PromptRequest::text("ana", "three"))
        .await
        .expect_err("the script ran out");
    assert!(matches!(error, HarnessError::Agent { .. }));
}

/// One script, many sessions: the coordinator session and a project session
/// draw from the same queue, which is what makes a multi-session test
/// deterministic.
#[tokio::test]
async fn sessions_share_one_script() {
    let harness = Arc::new(
        ScriptedHarness::builder("codex")
            .turn(ScriptedTurn::completed("coordinator"))
            .turn(ScriptedTurn::completed("project"))
            .build()
            .expect("declares"),
    );
    let coordinator = harness
        .open(&SessionKey::coordinator("codex", "ana"))
        .await
        .expect("opens");
    let project = harness
        .open(&SessionKey::project("codex", "p-1"))
        .await
        .expect("opens");
    assert_eq!(coordinator.session_id(), "codex-session-1");
    assert_eq!(project.session_id(), "codex-session-2");

    assert_eq!(
        coordinator
            .prompt(PromptRequest::text("ana", "a"))
            .await
            .expect("completes")
            .content(),
        "coordinator",
    );
    assert_eq!(
        project
            .prompt(PromptRequest::text("ana", "b"))
            .await
            .expect("completes")
            .content(),
        "project",
    );
}

/// Cancelling reports what happened, and the default is the honest answer.
#[tokio::test]
async fn cancel_defaults_to_requested_not_confirmed() {
    let harness = Arc::new(ScriptedHarness::builder("codex").build().expect("declares"));
    let session = harness
        .open(&SessionKey::coordinator("codex", "ana"))
        .await
        .expect("opens");

    let outcome = session.cancel(CancelScope::Turn).await.expect("cancels");
    assert!(!outcome.is_confirmed());
    assert_eq!(outcome.route(), Some(CancelRoute::Adapter));
    assert_eq!(outcome.status_str(), "cancelling");
    assert_eq!(harness.sessions()[0].cancels(), vec![CancelScope::Turn]);
}

/// A harness that confirms says so, and the confirmation carries its instant.
#[tokio::test]
async fn a_confirming_harness_reports_a_confirmed_cancel() {
    let harness = ScriptedHarness::builder("codex")
        .cancel_outcome(
            CancelOutcome::requested(CancelRoute::Coordinator, CancelTarget::delegation("d-1"))
                .confirm("2026-08-22T09:00:00.000Z")
                .expect("the one legal edge"),
        )
        .build()
        .expect("declares");
    let session = harness
        .open(&SessionKey::coordinator("codex", "ana"))
        .await
        .expect("opens");

    let outcome = session
        .cancel(CancelScope::Delegation {
            delegation_id: "d-1".to_owned(),
        })
        .await
        .expect("cancels");
    assert!(outcome.is_confirmed());
    assert_eq!(outcome.confirmed_at(), Some("2026-08-22T09:00:00.000Z"));
    assert_eq!(outcome.route(), Some(CancelRoute::Coordinator));
}

/// A harness that cannot delegate refuses a delegation cancel, with upstream's
/// sentence.
#[tokio::test]
async fn a_non_delegating_harness_refuses_a_delegation_cancel() {
    let harness = ScriptedHarness::builder("pi")
        .capabilities(via_downstream::BackendCapabilities {
            delegation: false,
            permissions: false,
            backend_ui: false,
            native_session_history: true,
            external_mcp: false,
            native_delegation: false,
            session_mcp: false,
        })
        .build()
        .expect("declares");
    let session = harness
        .open(&SessionKey::coordinator("pi", "ana"))
        .await
        .expect("opens");

    let error = session
        .cancel(CancelScope::Delegation {
            delegation_id: "d-1".to_owned(),
        })
        .await
        .expect_err("pi does not delegate");
    assert!(matches!(error, HarnessError::CancelUnsupported));
    assert_eq!(
        error.message(via_i18n::Locale::Zh),
        "当前后台 Agent 不支持取消第三层 Session",
    );

    // A turn-level cancel is still available: it is transport-level and every
    // harness owes it.
    assert!(session.cancel(CancelScope::Turn).await.is_ok());
}

/// Health is reported through the trait, and a cold start is not a failure.
#[tokio::test]
async fn health_travels_through_the_trait() {
    let harness = ScriptedHarness::builder("codex")
        .health(HarnessHealth::backend_starting())
        .build()
        .expect("declares");
    let health = harness.health().await;
    assert!(!health.is_ok());
    assert!(health.is_transient());
    assert_eq!(health.code(), HarnessStatusCode::BackendStarting);

    let ready = ScriptedHarness::builder("codex")
        .build()
        .expect("declares")
        .health()
        .await;
    assert!(ready.is_ok());
    assert!(!ready.is_transient());
}

/// The double is validated by the same path a real harness is, so a test
/// cannot accidentally exercise a descriptor no real driver could declare.
#[test]
fn the_double_cannot_declare_an_impossible_backend() {
    let error = ScriptedHarness::builder("ghost")
        .build()
        .expect_err("`ghost` is not a catalogued backend");
    assert!(matches!(error, HarnessError::DriverNotRegistered { .. }));
    assert_eq!(
        error.message(via_i18n::Locale::Zh),
        "后台 Driver 未在目录注册：ghost",
    );
}

/// A second prompt while one is in flight is upstream's 409, with upstream's
/// sentence.
///
/// `docs/architecture.md` §11: one item per owner inside the backend session at
/// a time. The double runs a turn to completion synchronously, so the race is
/// staged with `hold`.
#[tokio::test]
async fn a_second_prompt_on_a_busy_session_is_refused() {
    let harness = Arc::new(
        ScriptedHarness::builder("codex")
            .turn(ScriptedTurn::completed("ok"))
            .build()
            .expect("declares"),
    );
    let session = harness
        .open(&SessionKey::coordinator("codex", "ana"))
        .await
        .expect("opens");
    let observed = harness.sessions();
    let handle = &observed[0];

    handle.hold();
    assert!(handle.is_busy());
    let error = session
        .prompt(PromptRequest::text("ana", "go"))
        .await
        .expect_err("a turn is already in flight");
    match &error {
        HarnessError::SessionBusy { label, session_id } => {
            assert_eq!(label, "Codex");
            assert_eq!(session_id, "codex-session-1");
        }
        other => panic!("wrong variant: {other:?}"),
    }
    assert_eq!(error.http_status(), via_downstream::SESSION_BUSY_STATUS);
    assert_eq!(
        error.message(via_i18n::Locale::Zh),
        "Codex Session codex-session-1 已有正在执行的请求",
    );
    // The refused prompt consumed no turn.
    assert_eq!(harness.remaining_turns(), 1);

    handle.release();
    assert!(!handle.is_busy());
    assert_eq!(
        session
            .prompt(PromptRequest::text("ana", "go"))
            .await
            .expect("completes")
            .content(),
        "ok",
    );
}
