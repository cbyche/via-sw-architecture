//! The session's handshake, its conversation items, and how it ends.
//!
//! Ported from `server/test/realtime-provider.test.mjs`. Where upstream reaches
//! into the object — overwriting `frontend.send`, pushing into
//! `frontend.pendingResponses`, calling `handleLifecycle` directly — these drive
//! the real transport and the real public API instead, so the command channels
//! and both owning tasks are exercised rather than skipped.

mod common;

use std::sync::Arc;
use std::time::Duration;

use common::harness::{
    Harness, RoutedProtocol, acknowledge_item, acknowledge_item_with_id, collect_events,
    complete_response, expect_any_frame, expect_frame, quick_timeouts,
};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use via_i18n::Locale;
use via_protocol::SessionMode;
use via_realtime::testing::{TestProvider, test_transport};
use via_realtime::{
    AgentContext, AgentContextPatch, CONNECT_TIMEOUT, FunctionOutputOptions, OutcomeKind,
    OutcomePhase, ProviderCapabilities, RealtimeError, RealtimeSession, ResponseContext,
    ResponseOrigin, SessionOptions,
};

fn agent_context() -> AgentContext {
    AgentContext {
        instructions: "# Role\nyou are a voice agent".into(),
        tools: vec![json!({ "type": "function", "function": { "name": "spawn_thinking" } })],
        ..AgentContext::default()
    }
}

// ── the handshake ───────────────────────────────────────────────────────────

#[tokio::test]
async fn the_first_session_update_negotiates_the_whole_session() {
    let harness = Harness::open(
        TestProvider::new("beta").shared(),
        SessionOptions {
            agent_context: agent_context(),
            ..SessionOptions::default()
        },
    )
    .await;

    let session = &harness.session_update["session"];
    assert_eq!(
        session["instructions"],
        json!("# Role\nyou are a voice agent")
    );
    assert_eq!(
        session["tools"][0]["function"]["name"],
        json!("spawn_thinking")
    );
    // The negotiated half is present exactly once, on the first update.
    assert_eq!(session["turn_detection"], json!({ "type": "server_vad" }));
    assert_eq!(session["modalities"], json!(["text", "audio"]));
    // Every frame carries the dialect's envelope.
    assert!(
        harness.session_update["event_id"]
            .as_str()
            .is_some_and(|id| id.starts_with("event_"))
    );
}

#[tokio::test]
async fn a_provider_that_never_acknowledges_becomes_ready_on_write() {
    // `acknowledges_session_update: false` — huggingface/speech-to-speech applies
    // the update silently, so waiting for `session.updated` would hang forever.
    let harness = Harness::ga().await;
    assert_eq!(harness.session_update["session"]["instructions"], json!(""));
    assert!(!harness.session.capabilities().acknowledges_session_update);
}

#[tokio::test]
async fn a_provider_error_before_readiness_rejects_the_connect() {
    let (transport, peer) = test_transport();
    peer.send(json!({
        "type": "error",
        "error": {
            "type": "session_limit_reached",
            "message": "All 1 session slots are in use. Disconnect an existing client first.",
        },
    }));

    let error = RealtimeSession::open(
        TestProvider::ga("s2s-like").shared(),
        SessionOptions::default(),
        transport,
    )
    .await
    .expect_err("the slot is busy");

    // Fast, not a hang: the caller's backoff is what retries once the slot is
    // released, and it cannot back off while it is still waiting.
    assert_eq!(
        error,
        RealtimeError::ProviderRefused {
            message: "session_limit_reached: All 1 session slots are in use. \
                      Disconnect an existing client first."
                .into(),
        }
    );
}

#[tokio::test]
async fn the_composed_error_carries_every_distinct_field() {
    let (transport, peer) = test_transport();
    peer.send(json!({
        "type": "error",
        "error": {
            "code": "AllocationQuota.FreeTierOnly",
            "type": "insufficient_quota",
            "message": "The free tier of the model has been exhausted.",
        },
    }));
    let error = RealtimeSession::open(
        TestProvider::new("beta").shared(),
        SessionOptions::default(),
        transport,
    )
    .await
    .expect_err("refused");
    assert_eq!(
        error.message(Locale::En),
        "AllocationQuota.FreeTierOnly: insufficient_quota: \
         The free tier of the model has been exhausted."
    );
}

#[tokio::test]
async fn preflight_refuses_before_a_socket_is_opened() {
    let provider = TestProvider::new("beta")
        .with_preflight_error(RealtimeError::UnsupportedModel {
            id: "qwen3.5-omni-flash-realtime-future".into(),
            label: "beta".into(),
        })
        .shared();
    let (transport, mut peer) = test_transport();

    let error = RealtimeSession::open(provider, SessionOptions::default(), transport)
        .await
        .expect_err("unknown model");
    assert_eq!(
        error,
        RealtimeError::UnsupportedModel {
            id: "qwen3.5-omni-flash-realtime-future".into(),
            label: "beta".into(),
        }
    );
    // Nothing was written: the check is upstream of the transport.
    assert_eq!(peer.drain_frames(), Vec::<Value>::new());
}

#[tokio::test]
async fn connect_refuses_before_it_opens_a_socket_too() {
    // The endpoint is a port nothing listens on, so a `connect` that reached the
    // socket would answer with a transport failure. It answers with the
    // preflight refusal instead, which is the proof that the gate runs first.
    let provider = TestProvider::new("beta")
        .with_url("ws://127.0.0.1:1/v1/realtime")
        .with_preflight_error(RealtimeError::UnsupportedModel {
            id: "qwen3.5-omni-flash-realtime-future".into(),
            label: "beta".into(),
        })
        .shared();
    assert_eq!(
        RealtimeSession::connect(provider, SessionOptions::default())
            .await
            .map(|_| ())
            .expect_err("unknown model"),
        RealtimeError::UnsupportedModel {
            id: "qwen3.5-omni-flash-realtime-future".into(),
            label: "beta".into(),
        }
    );
}

#[tokio::test]
async fn connect_refuses_an_unconfigured_provider_before_it_opens_a_socket() {
    let provider = TestProvider::new("beta")
        .with_url("ws://127.0.0.1:1/v1/realtime")
        .with_configured(false)
        .shared();
    assert_eq!(
        RealtimeSession::connect(provider, SessionOptions::default())
            .await
            .map(|_| ())
            .expect_err("unconfigured"),
        RealtimeError::NotConfigured {
            provider: "beta".into(),
            message: "beta is not configured".into(),
        }
    );
}

#[tokio::test]
async fn an_unconfigured_provider_is_refused_with_its_own_sentence() {
    let provider = TestProvider::new("beta").with_configured(false).shared();
    let (transport, _peer) = test_transport();
    let error = RealtimeSession::open(provider, SessionOptions::default(), transport)
        .await
        .expect_err("unconfigured");
    assert_eq!(
        error,
        RealtimeError::NotConfigured {
            provider: "beta".into(),
            message: "beta is not configured".into(),
        }
    );
}

#[tokio::test(start_paused = true)]
async fn the_connect_budget_covers_the_whole_handshake() {
    // The socket is fine; the provider simply never says `session.created`.
    let (transport, _peer) = test_transport();
    let started = tokio::time::Instant::now();
    let error = RealtimeSession::open(
        TestProvider::new("beta").shared(),
        SessionOptions::default(),
        transport,
    )
    .await
    .expect_err("timed out");

    assert_eq!(
        error,
        RealtimeError::ConnectTimeout {
            provider: "beta".into(),
            message: "beta connect timed out".into(),
        }
    );
    assert_eq!(started.elapsed(), CONNECT_TIMEOUT);
}

#[tokio::test]
async fn connection_messages_are_written_before_anything_else() {
    let provider = TestProvider::new("routed")
        .with_protocol(Box::new(RoutedProtocol::new("fixed-route")))
        .shared();
    let (transport, mut peer) = test_transport();
    let handshake = tokio::spawn(async move {
        let first = expect_any_frame(&mut peer).await;
        peer.send(json!({ "type": "session.created", "route": "fixed-route" }));
        let second = expect_any_frame(&mut peer).await;
        peer.send(json!({ "type": "session.updated", "route": "fixed-route" }));
        (first, second)
    });
    let (_session, _events) = RealtimeSession::open(provider, SessionOptions::default(), transport)
        .await
        .expect("opens");
    let (first, second) = handshake.await.expect("handshake");

    assert_eq!(first["type"], json!("start"));
    assert_eq!(second["type"], json!("session.update"));
    assert_eq!(second["route"], json!("fixed-route"));
}

#[tokio::test]
async fn a_frame_that_is_not_json_is_ignored_rather_than_fatal() {
    let harness = Harness::beta().await;
    harness.peer.send_raw("not json at all");
    harness.peer.send(json!({ "type": "session.updated" }));
    // The session is still usable afterwards.
    harness
        .events
        .wait_for("session.updated", |entries| {
            entries
                .iter()
                .filter_map(common::harness::Recorded::provider)
                .any(|event| event.kind() == "session.updated")
        })
        .await;
    assert!(!harness.events.is_closed());
}

// ── restored context ────────────────────────────────────────────────────────

#[tokio::test]
async fn recent_conversation_is_restored_exactly_once() {
    let mut harness = Harness::open(
        TestProvider::new("beta").shared(),
        SessionOptions {
            agent_context: AgentContext {
                recent_context: Some("User: where were we".into()),
                ..agent_context()
            },
            ..SessionOptions::default()
        },
    )
    .await;

    let restored = expect_frame(&mut harness.peer, "conversation.item.create").await;
    let text = restored["item"]["content"][0]["text"]
        .as_str()
        .expect("text")
        .to_owned();
    assert!(text.starts_with("<restored_context>\n"), "{text}");
    assert!(text.ends_with("\n</restored_context>"), "{text}");
    assert!(text.contains("User: where were we"), "{text}");
    assert!(
        text.contains("It is context only, not a new user request."),
        "{text}"
    );

    // A second `session.updated` must not restore it again.
    harness.peer.send(json!({ "type": "session.updated" }));
    harness
        .events
        .wait_for("the second update", |entries| {
            entries
                .iter()
                .filter_map(common::harness::Recorded::provider)
                .filter(|event| event.kind() == "session.updated")
                .count()
                >= 2
        })
        .await;
    assert_eq!(harness.peer.drain_frames(), Vec::<Value>::new());
}

#[tokio::test]
async fn a_session_with_no_recent_conversation_restores_nothing() {
    let mut harness = Harness::open(
        TestProvider::new("beta").shared(),
        SessionOptions {
            agent_context: agent_context(),
            ..SessionOptions::default()
        },
    )
    .await;
    assert_eq!(harness.peer.drain_frames(), Vec::<Value>::new());
}

#[tokio::test]
async fn dictation_restores_no_conversation_context_at_all() {
    // `docs/architecture.md` §2: `dictation` mounts no model, so there is no
    // conversation for a restored block to be context *for*.
    let mut harness = Harness::open(
        TestProvider::new("asr").shared(),
        SessionOptions {
            mode: SessionMode::Dictation,
            agent_context: AgentContext {
                recent_context: Some("User: where were we".into()),
                ..AgentContext::default()
            },
            ..SessionOptions::default()
        },
    )
    .await;
    assert_eq!(harness.peer.drain_frames(), Vec::<Value>::new());
}

// ── audio ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn audio_is_appended_and_committed_in_every_mode() {
    for mode in [
        SessionMode::Dictation,
        SessionMode::Direct,
        SessionMode::Agent,
        SessionMode::Interface,
    ] {
        let mut harness = Harness::open(
            TestProvider::new("beta").shared(),
            SessionOptions {
                mode,
                ..SessionOptions::default()
            },
        )
        .await;
        harness.session.append_audio("cGNt").await.expect("append");
        harness.session.commit_audio().await.expect("commit");

        let append = expect_any_frame(&mut harness.peer).await;
        assert_eq!(append["type"], json!("input_audio_buffer.append"), "{mode}");
        assert_eq!(append["audio"], json!("cGNt"), "{mode}");
        let commit = expect_any_frame(&mut harness.peer).await;
        assert_eq!(commit["type"], json!("input_audio_buffer.commit"), "{mode}");
    }
}

#[tokio::test]
async fn dictation_refuses_every_call_that_would_create_a_response() {
    let harness = Harness::open(
        TestProvider::new("asr").shared(),
        SessionOptions {
            mode: SessionMode::Dictation,
            ..SessionOptions::default()
        },
    )
    .await;
    let session = &harness.session;

    let outcomes = vec![
        session
            .speak("hello", ResponseOrigin::Agent, ResponseContext::new(), None)
            .await
            .expect("call"),
        session
            .send_user_text("hello", ResponseContext::new(), None)
            .await
            .expect("call"),
        session
            .ensure_response(ResponseContext::new(), None, None)
            .await
            .expect("call"),
        session
            .send_user_input(None, ResponseContext::new(), None)
            .await
            .expect("call"),
        session
            .send_function_output(
                "call-1",
                json!({}),
                ResponseContext::new(),
                FunctionOutputOptions::with_response(),
            )
            .await
            .expect("call"),
        session
            .inject_permission(
                &via_realtime::PermissionRequest {
                    id: "auth-1".into(),
                    summary: "run it".into(),
                },
                ResponseContext::new(),
                None,
            )
            .await
            .expect("call"),
    ];

    for outcome in outcomes {
        let outcome = outcome.expect("an outcome, not silence");
        assert_eq!(outcome.kind, OutcomeKind::Skipped);
        assert_eq!(outcome.phase, Some(OutcomePhase::NoModelTurn));
    }

    let injected = session
        .inject_result(
            "done",
            ResponseOrigin::Announcement,
            ResponseContext::new(),
            true,
        )
        .await
        .expect("call")
        .expect("an outcome");
    assert_eq!(
        injected.outcome.map(|outcome| outcome.phase),
        Some(Some(OutcomePhase::NoModelTurn))
    );
    assert!(!injected.context_injected);
}

// ── conversation items ──────────────────────────────────────────────────────

#[tokio::test]
async fn text_is_written_as_an_item_and_only_then_as_a_response() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;

    let driver = tokio::spawn(async move {
        let item = acknowledge_item(&mut peer).await;
        let response = complete_response(&mut peer, "response-text").await;
        (item, response, peer)
    });
    let outcome = session
        .send_user_text(
            "  hello, typed mode  ",
            ResponseContext::new().with("turnId", "text-1"),
            Some(vec!["text".into()]),
        )
        .await
        .expect("call")
        .expect("an outcome");
    let (item, response, _peer) = driver.await.expect("driver");

    assert_eq!(item["item"]["type"], json!("message"));
    assert_eq!(item["item"]["role"], json!("user"));
    assert_eq!(
        item["item"]["content"],
        json!([{ "type": "input_text", "text": "hello, typed mode" }])
    );
    assert_eq!(response["response"], json!({ "modalities": ["text"] }));
    assert_eq!(outcome.kind, OutcomeKind::Completed);
    assert_eq!(outcome.response_id.as_deref(), Some("response-text"));
}

#[tokio::test]
async fn empty_text_never_reaches_the_provider() {
    let mut harness = Harness::beta().await;
    assert_eq!(
        harness
            .session
            .send_user_text("   \n ", ResponseContext::new(), None)
            .await
            .expect("call"),
        None
    );
    assert_eq!(
        harness
            .session
            .speak("  ", ResponseOrigin::Agent, ResponseContext::new(), None)
            .await
            .expect("call"),
        None
    );
    assert_eq!(
        harness
            .session
            .inject_result(
                "",
                ResponseOrigin::Announcement,
                ResponseContext::new(),
                true
            )
            .await
            .expect("call")
            .is_none(),
        true
    );
    assert!(
        !harness
            .session
            .append_user_context("  ")
            .await
            .expect("call")
    );
    assert_eq!(harness.peer.drain_frames(), Vec::<Value>::new());
}

#[tokio::test]
async fn a_rejected_item_never_triggers_inference() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;

    let driver = tokio::spawn(async move {
        expect_frame(&mut peer, "conversation.item.create").await;
        peer.send(json!({
            "type": "error",
            "error": { "message": "invalid conversation item" },
        }));
        peer
    });
    let outcome = session
        .send_user_text("failing input", ResponseContext::new(), None)
        .await
        .expect("call")
        .expect("an outcome");
    let mut peer = driver.await.expect("driver");

    assert_eq!(outcome.kind, OutcomeKind::Failed);
    assert_eq!(outcome.phase, Some(OutcomePhase::Input));
    assert_eq!(outcome.error.as_deref(), Some("invalid conversation item"));
    // The response was never created: an error while an item receipt is
    // outstanding belongs to the item.
    assert_eq!(peer.drain_frames(), Vec::<Value>::new());
}

#[tokio::test]
async fn a_failure_that_came_from_a_provider_event_is_reported_once() {
    let Harness {
        session,
        mut peer,
        events,
        ..
    } = Harness::beta().await;
    let driver = tokio::spawn(async move {
        expect_frame(&mut peer, "conversation.item.create").await;
        peer.send(json!({ "type": "error", "error": { "message": "nope" } }));
        peer
    });
    session
        .send_user_text("failing input", ResponseContext::new(), None)
        .await
        .expect("call");
    let _peer = driver.await.expect("driver");

    // The refusal is the outcome; emitting it again as a session error would
    // surface one provider refusal twice.
    assert_eq!(events.errors(), Vec::new());
}

#[tokio::test]
async fn an_item_receipt_with_a_provider_assigned_id_is_still_accepted() {
    // `conversation_item_id_echo: false` — some providers acknowledge the item
    // but replace its id, so the single pending waiter is matched instead.
    let provider = TestProvider::new("omni-like")
        .with_capabilities(ProviderCapabilities {
            conversation_item_id_echo: false,
            ..ProviderCapabilities::DEFAULT
        })
        .shared();
    let Harness {
        session, mut peer, ..
    } = Harness::open(provider, SessionOptions::default()).await;

    let driver = tokio::spawn(async move {
        let written = acknowledge_item_with_id(&mut peer, "item_provider_assigned").await;
        complete_response(&mut peer, "r1").await;
        (written, peer)
    });
    let outcome = session
        .send_user_text("hi", ResponseContext::new(), None)
        .await
        .expect("call")
        .expect("an outcome");
    let (written, _peer) = driver.await.expect("driver");

    assert_eq!(outcome.kind, OutcomeKind::Completed);
    assert_ne!(written["item"]["id"], json!("item_provider_assigned"));
}

#[tokio::test(start_paused = true)]
async fn an_echoing_provider_ignores_a_receipt_for_someone_elses_item() {
    let Harness {
        session, mut peer, ..
    } = Harness::open(
        TestProvider::new("beta").shared(),
        quick_timeouts(Duration::from_millis(40), Duration::from_secs(60)),
    )
    .await;

    let driver = tokio::spawn(async move {
        expect_frame(&mut peer, "conversation.item.create").await;
        // A receipt for an id this session never minted.
        peer.send(json!({
            "type": "conversation.item.created",
            "item": { "id": "item_someone_else" },
        }));
        peer
    });
    let outcome = session
        .send_user_text("hi", ResponseContext::new(), None)
        .await
        .expect("call")
        .expect("an outcome");
    let _peer = driver.await.expect("driver");

    assert_eq!(outcome.kind, OutcomeKind::Failed);
    assert_eq!(outcome.phase, Some(OutcomePhase::Input));
    assert!(
        outcome
            .error
            .as_deref()
            .is_some_and(|error| error.contains("did not confirm conversation item")),
        "{outcome:?}"
    );
}

#[tokio::test]
async fn a_stale_tool_call_is_closed_without_a_new_response() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;

    let driver = tokio::spawn(async move {
        let item = acknowledge_item(&mut peer).await;
        (item, peer)
    });
    let outcome = session
        .send_function_output(
            "call-stale",
            json!({ "status": "superseded" }),
            ResponseContext::new().with("turnId", "turn-old"),
            FunctionOutputOptions::without_response(),
        )
        .await
        .expect("call");
    let (item, mut peer) = driver.await.expect("driver");

    assert_eq!(outcome, None);
    assert_eq!(item["item"]["type"], json!("function_call_output"));
    assert_eq!(item["item"]["call_id"], json!("call-stale"));
    assert_eq!(item["item"]["output"], json!(r#"{"status":"superseded"}"#));
    assert_eq!(peer.drain_frames(), Vec::<Value>::new());
}

#[tokio::test]
async fn a_tool_result_can_carry_follow_up_instructions() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;

    let driver = tokio::spawn(async move {
        acknowledge_item(&mut peer).await;
        let response = complete_response(&mut peer, "response-followup").await;
        (response, peer)
    });
    let outcome = session
        .send_function_output(
            "call-accepted",
            json!({ "status": "accepted" }),
            ResponseContext::new().with("turnId", "turn-one"),
            FunctionOutputOptions {
                create_response: true,
                response: Some(json!({ "instructions": "decide whether to confirm" })),
            },
        )
        .await
        .expect("call")
        .expect("an outcome");
    let (response, _peer) = driver.await.expect("driver");

    assert_eq!(
        response["response"],
        json!({ "instructions": "decide whether to confirm" })
    );
    assert_eq!(outcome.kind, OutcomeKind::Completed);
    assert_eq!(outcome.response_id.as_deref(), Some("response-followup"));
}

#[tokio::test]
async fn appended_context_is_written_without_asking_for_an_answer() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    let driver = tokio::spawn(async move {
        let item = acknowledge_item(&mut peer).await;
        (item, peer)
    });
    assert!(
        session
            .append_user_context("  the user attached a file  ")
            .await
            .expect("call")
    );
    let (item, mut peer) = driver.await.expect("driver");
    assert_eq!(
        item["item"]["content"],
        json!([{ "type": "input_text", "text": "the user attached a file" }])
    );
    assert_eq!(peer.drain_frames(), Vec::<Value>::new());
}

// ── projection ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn projection_falls_back_to_the_callers_composed_text() {
    let harness = Harness::beta().await;
    let projection = harness
        .session
        .project_user_input(&json!([]), &json!({}), Some("[Image 1] what is this?"))
        .expect("a projection");
    assert_eq!(
        projection.conversation_item,
        Some(json!({
            "type": "message",
            "role": "user",
            "content": [{ "type": "input_text", "text": "[Image 1] what is this?" }],
        }))
    );
    assert!(projection.before_events.is_empty());
    assert!(projection.after_events.is_empty());

    assert!(
        harness
            .session
            .project_user_input(&json!([]), &json!({}), Some(""))
            .is_none()
    );
    assert!(
        harness
            .session
            .project_user_input(&json!([]), &json!({}), None)
            .is_none()
    );
}

#[tokio::test]
async fn a_projection_writes_before_events_the_item_and_after_events_in_order() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    let projection = via_realtime::InputProjection {
        before_events: vec![json!({ "type": "input_audio_buffer.clear" })],
        conversation_item: Some(json!({ "type": "message", "role": "user", "content": [] })),
        after_events: vec![json!({ "type": "input_audio_buffer.commit" })],
    };

    let driver = tokio::spawn(async move {
        let first = expect_any_frame(&mut peer).await;
        let item = acknowledge_item(&mut peer).await;
        let last = expect_any_frame(&mut peer).await;
        (first, item, last, peer)
    });
    assert!(
        session
            .append_user_input_context(Some(projection))
            .await
            .expect("call")
    );
    let (first, item, last, _peer) = driver.await.expect("driver");

    assert_eq!(first["type"], json!("input_audio_buffer.clear"));
    assert_eq!(item["type"], json!("conversation.item.create"));
    assert_eq!(last["type"], json!("input_audio_buffer.commit"));
}

#[tokio::test]
async fn no_projection_writes_nothing_and_answers_false() {
    let mut harness = Harness::beta().await;
    assert!(
        !harness
            .session
            .append_user_input_context(None)
            .await
            .expect("call")
    );
    assert_eq!(harness.peer.drain_frames(), Vec::<Value>::new());
}

// ── the live session payload ────────────────────────────────────────────────

#[tokio::test]
async fn a_context_patch_refreshes_the_live_session_without_renegotiating() {
    let Harness {
        session, mut peer, ..
    } = Harness::open(
        TestProvider::new("beta").shared(),
        SessionOptions {
            agent_context: agent_context(),
            ..SessionOptions::default()
        },
    )
    .await;

    session
        .update_agent_context(AgentContextPatch {
            instructions: Some("# Role\nthe user is called Xiaoming".into()),
            ..AgentContextPatch::default()
        })
        .await
        .expect("patch");
    session.drain().await.expect("drain");

    let update = expect_frame(&mut peer, "session.update").await;
    assert_eq!(
        update["session"]["instructions"],
        json!("# Role\nthe user is called Xiaoming")
    );
    // The negotiated half is absent on every update after the first: re-sending
    // `turn_detection` would reset the provider's VAD mid-conversation.
    assert_eq!(
        update["session"]
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["instructions", "tools"]
    );
}

#[tokio::test]
async fn a_barge_in_does_not_cancel_the_session_refresh() {
    // `updateAgentContext`'s refresh checks `ready` and deliberately **not** the
    // queue generation: a barge-in cancels responses, it does not invalidate the
    // instructions the model is about to be given. Cancelling it would leave the
    // model configured from a context the user has already moved past.
    let Harness {
        session, mut peer, ..
    } = Harness::open(
        TestProvider::new("beta").shared(),
        SessionOptions {
            agent_context: agent_context(),
            ..SessionOptions::default()
        },
    )
    .await;

    session
        .update_agent_context(AgentContextPatch {
            instructions: Some("# Role\nthe user is called Xiaoming".into()),
            ..AgentContextPatch::default()
        })
        .await
        .expect("patch");
    session.cancel().await.expect("cancel");
    session.drain().await.expect("drain");

    let update = expect_frame(&mut peer, "session.update").await;
    assert_eq!(
        update["session"]["instructions"],
        json!("# Role\nthe user is called Xiaoming")
    );
}

#[tokio::test]
async fn a_patch_is_a_one_level_merge() {
    let Harness {
        session, mut peer, ..
    } = Harness::open(
        TestProvider::new("beta").shared(),
        SessionOptions {
            agent_context: agent_context(),
            ..SessionOptions::default()
        },
    )
    .await;

    session
        .update_agent_context(AgentContextPatch {
            tools: Some(Vec::new()),
            ..AgentContextPatch::default()
        })
        .await
        .expect("patch");
    session.drain().await.expect("drain");

    let update = expect_frame(&mut peer, "session.update").await;
    // Tools were replaced with nothing; instructions were not touched.
    assert_eq!(
        update["session"]["instructions"],
        json!("# Role\nyou are a voice agent")
    );
    assert!(update["session"].get("tools").is_none());
}

// ── closing ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn closing_settles_everything_and_says_closed_last() {
    let Harness {
        session,
        mut peer,
        events,
        ..
    } = Harness::beta().await;

    let driver = tokio::spawn(async move {
        expect_frame(&mut peer, "response.create").await;
        peer
    });
    let speaking = tokio::spawn({
        let session = session.clone();
        async move {
            session
                .speak(
                    "background result",
                    ResponseOrigin::Announcement,
                    ResponseContext::new(),
                    None,
                )
                .await
        }
    });
    let _peer = driver.await.expect("driver");

    session.close().await.expect("close");
    let outcome = speaking
        .await
        .expect("task")
        .expect("call")
        .expect("an outcome");
    assert_eq!(outcome.kind, OutcomeKind::Cancelled);

    events
        .wait_for("closed", |entries| {
            entries
                .iter()
                .any(|entry| matches!(entry, common::harness::Recorded::Closed))
        })
        .await;
    let snapshot = events.snapshot();
    assert!(matches!(
        snapshot.last(),
        Some(common::harness::Recorded::Closed)
    ));
}

#[tokio::test]
async fn a_closed_socket_settles_everything_in_flight() {
    let Harness {
        session,
        mut peer,
        events,
        ..
    } = Harness::beta().await;

    let speaking = tokio::spawn({
        let session = session.clone();
        async move {
            session
                .speak("hello", ResponseOrigin::Agent, ResponseContext::new(), None)
                .await
        }
    });
    expect_frame(&mut peer, "response.create").await;
    peer.close();

    let outcome = speaking
        .await
        .expect("task")
        .expect("call")
        .expect("an outcome");
    assert_eq!(outcome.kind, OutcomeKind::Cancelled);
    events
        .wait_for("closed", |entries| {
            entries
                .iter()
                .any(|entry| matches!(entry, common::harness::Recorded::Closed))
        })
        .await;

    // Every later call answers with the same fact rather than hanging.
    assert_eq!(
        session.append_audio("cGNt").await,
        Err(RealtimeError::ConnectionClosed {
            label: "beta".into()
        })
    );
}

#[tokio::test]
async fn a_transport_failure_is_reported_with_its_own_text() {
    let Harness {
        session: _session,
        peer,
        events,
        ..
    } = Harness::beta().await;
    peer.fail("Unexpected server response: 401");

    let entries = events
        .wait_for("a transport error", |entries| {
            entries.iter().any(|entry| entry.error().is_some())
        })
        .await;
    let error = entries
        .iter()
        .find_map(common::harness::Recorded::error)
        .expect("an error");
    // Not localized: the text is what a provider's `classify_error` corpus reads.
    assert_eq!(
        error,
        &RealtimeError::Transport {
            detail: "Unexpected server response: 401".into()
        }
    );
}

#[tokio::test]
async fn dropping_the_last_handle_closes_the_socket() {
    let provider = TestProvider::new("beta").shared();
    let (transport, mut peer) = test_transport();
    let handshake = tokio::spawn(async move {
        peer.send(json!({ "type": "session.created" }));
        expect_frame(&mut peer, "session.update").await;
        peer.send(json!({ "type": "session.updated" }));
        peer
    });
    let (session, events) = RealtimeSession::open(provider, SessionOptions::default(), transport)
        .await
        .expect("opens");
    let _peer = handshake.await.expect("handshake");
    let log = collect_events(events);

    let clone = session.clone();
    drop(session);
    // One handle is still alive, so nothing has closed yet.
    assert!(!log.is_closed());
    drop(clone);

    log.wait_for("closed", |entries| {
        entries
            .iter()
            .any(|entry| matches!(entry, common::harness::Recorded::Closed))
    })
    .await;
}

#[tokio::test]
async fn the_session_reports_what_it_is_talking_to() {
    let harness = Harness::open(
        Arc::clone(
            &TestProvider::new("beta")
                .with_input_sample_rate(24_000)
                .shared(),
        ),
        SessionOptions {
            mode: SessionMode::Interface,
            ..SessionOptions::default()
        },
    )
    .await;
    assert_eq!(harness.session.provider().key(), "beta");
    assert_eq!(harness.session.input_sample_rate(), 24_000);
    assert_eq!(harness.session.mode(), SessionMode::Interface);
    assert_eq!(harness.session.connection_id().len(), 32);
    assert_eq!(
        harness.session.capabilities(),
        ProviderCapabilities::DEFAULT
    );
}
