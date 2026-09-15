//! Both providers driven through a real [`RealtimeSession`], with no socket.
//!
//! The unit tests assert what each provider *builds*; this file asserts what
//! actually reaches the wire once `via-realtime` has encoded it — which is the
//! only place three separate contracts meet: the provider's payload, the
//! dialect's envelope, and the capability flag the session branches on.
//!
//! Every test is `start_paused` because the session arms two watchdogs and a
//! 25 s connect budget. On a paused clock a test that would hang instead
//! advances to the deadline and **fails**, which is what makes
//! `s2s_is_ready_the_moment_the_update_is_written` an assertion rather than a
//! coincidence.

use std::sync::Arc;
use std::time::Duration;

use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use via_core::Secret;
use via_realtime::testing::{TestPeer, test_transport};
use via_realtime::{
    AgentContext, PermissionRequest, RealtimeProvider, RealtimeSession, ResponseContext,
    ResponseOrigin, SessionOptions,
};
use via_realtime_dashscope::{
    DashScopeProvider, DashScopeSettings, SpeechToSpeechProvider, SpeechToSpeechSettings,
};

/// Long enough that a real stall reaches it, short enough to stay inside the
/// session's own 25 s connect budget.
const SETTLE: Duration = Duration::from_secs(5);

fn options() -> SessionOptions {
    SessionOptions {
        agent_context: AgentContext {
            instructions: "you are VIA".to_owned(),
            tools: vec![json!({
                "type": "function",
                "function": { "name": "spawn_thinking", "description": "d", "parameters": {} },
            })],
            ..AgentContext::default()
        },
        ..SessionOptions::default()
    }
}

fn dashscope() -> Arc<dyn RealtimeProvider> {
    Arc::new(DashScopeProvider::new(DashScopeSettings {
        api_key: Secret::new("sk-test"),
        ..DashScopeSettings::default()
    }))
}

fn speech_to_speech() -> Arc<dyn RealtimeProvider> {
    Arc::new(SpeechToSpeechProvider::new(SpeechToSpeechSettings {
        configured: true,
        ..SpeechToSpeechSettings::default()
    }))
}

/// Open a session, completing the handshake the provider's capabilities call for.
async fn open(provider: Arc<dyn RealtimeProvider>) -> (RealtimeSession, TestPeer, Value) {
    let acknowledges = provider.capabilities().acknowledges_session_update;
    let (transport, mut peer) = test_transport();
    let handshake = tokio::spawn(async move {
        peer.send(json!({ "type": "session.created" }));
        let update = peer
            .next_frame_of("session.update")
            .await
            .expect("the session configures itself");
        if acknowledges {
            peer.send(json!({ "type": "session.updated" }));
        }
        (peer, update)
    });
    let (session, _events) = RealtimeSession::open(provider, options(), transport)
        .await
        .expect("the session opens");
    let (peer, update) = handshake.await.expect("handshake");
    (session, peer, update)
}

async fn next_frame_of(peer: &mut TestPeer, kind: &str) -> Value {
    tokio::time::timeout(Duration::from_secs(20), peer.next_frame_of(kind))
        .await
        .unwrap_or_else(|_| panic!("timed out waiting for a `{kind}` frame"))
        .unwrap_or_else(|| panic!("the session closed before writing a `{kind}` frame"))
}

// ── the handshake ───────────────────────────────────────────────────────────

#[tokio::test(start_paused = true)]
async fn the_dashscope_handshake_writes_the_providers_own_payload() {
    let provider = dashscope();
    let expected = provider.build_session(&via_realtime::SessionRequest {
        configured: false,
        agent_context: &options().agent_context,
    });
    let (_session, _peer, update) = open(provider).await;

    assert_eq!(update["session"], expected);
    // The dialect's envelope stamps the id first, then the frame type.
    let keys: Vec<&str> = update
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, ["event_id", "type", "session"]);
    assert!(
        update["event_id"]
            .as_str()
            .is_some_and(|id| id.starts_with("event_")),
        "{update}"
    );
    assert_eq!(
        update["session"]["turn_detection"],
        json!({ "type": "smart_turn" })
    );
}

#[tokio::test(start_paused = true)]
async fn dashscope_waits_for_the_acknowledgement_it_declares() {
    // `acknowledges_session_update: true` — the session is not usable until
    // `session.updated` arrives, so a provider that stopped sending it would be
    // caught here rather than by a silent half-configured session.
    let (transport, mut peer) = test_transport();
    // Spawned rather than held as a future: the session has to be *running* for
    // "it has not finished yet" to mean anything.
    let mut opening = tokio::spawn(RealtimeSession::open(dashscope(), options(), transport));

    peer.send(json!({ "type": "session.created" }));
    let update = tokio::time::timeout(SETTLE, peer.next_frame_of("session.update"))
        .await
        .expect("the update is written")
        .expect("a frame");
    assert_eq!(update["type"], json!("session.update"));

    assert!(
        tokio::time::timeout(SETTLE, &mut opening).await.is_err(),
        "the session must not become ready before it is acknowledged"
    );

    peer.send(json!({ "type": "session.updated" }));
    let opened = tokio::time::timeout(SETTLE, opening)
        .await
        .expect("the acknowledgement opens it")
        .expect("the open task");
    assert!(opened.is_ok(), "{:?}", opened.err());
}

#[tokio::test(start_paused = true)]
async fn s2s_is_ready_the_moment_the_update_is_written() {
    // `realtime-provider.test.mjs:1426-1436`. Nothing is ever sent after
    // `session.update`; if the session waited for an acknowledgement it would
    // burn the whole 25 s connect budget and fail.
    let (transport, mut peer) = test_transport();
    let opening = tokio::spawn(RealtimeSession::open(
        speech_to_speech(),
        options(),
        transport,
    ));

    peer.send(json!({ "type": "session.created" }));
    let update = tokio::time::timeout(SETTLE, peer.next_frame_of("session.update"))
        .await
        .expect("the update is written")
        .expect("a frame");
    assert_eq!(update["session"]["type"], json!("realtime"));

    let opened = tokio::time::timeout(SETTLE, opening)
        .await
        .expect("no acknowledgement is needed")
        .expect("the open task");
    assert!(opened.is_ok());
}

// ── responses on the wire ───────────────────────────────────────────────────

#[tokio::test(start_paused = true)]
async fn a_dashscope_speak_reaches_the_wire_in_the_beta_shape() {
    let (session, mut peer, _) = open(dashscope()).await;
    let speaking = tokio::spawn(async move {
        session
            .speak(
                "三点有个会",
                ResponseOrigin::Agent,
                ResponseContext::new(),
                None,
            )
            .await
    });

    let frame = next_frame_of(&mut peer, "response.create").await;
    let body = &frame["response"];
    assert_eq!(body["conversation"], json!("none"));
    // The beta dialect leaves `modalities` alone.
    assert_eq!(body["modalities"], json!(["text", "audio"]));
    assert_eq!(body.get("output_modalities"), None);
    assert!(
        body["instructions"]
            .as_str()
            .is_some_and(|text| text.contains("三点有个会")),
        "{body}"
    );
    // No correlation metadata: the beta dialect has none, so the session falls
    // back to FIFO correlation.
    assert_eq!(body.get("metadata"), None);

    peer.send(json!({ "type": "response.created", "response": { "id": "resp_1" } }));
    peer.send(json!({
        "type": "response.done",
        "response": { "id": "resp_1", "status": "completed" },
    }));
    let outcome = speaking.await.expect("task").expect("speaks");
    assert_eq!(
        outcome.map(|outcome| outcome.kind),
        Some(via_realtime::OutcomeKind::Completed)
    );
}

#[tokio::test(start_paused = true)]
async fn an_s2s_speak_is_rewritten_into_the_ga_shape_and_correlated() {
    let (session, mut peer, _) = open(speech_to_speech()).await;
    let speaking = tokio::spawn(async move {
        session
            .speak(
                "任务完成",
                ResponseOrigin::Agent,
                ResponseContext::new(),
                None,
            )
            .await
    });

    let frame = next_frame_of(&mut peer, "response.create").await;
    let body = &frame["response"];
    // The GA dialect moves `modalities` to `output_modalities`; the provider
    // writes the beta key and the dialect does the rewrite exactly once.
    assert_eq!(body["output_modalities"], json!(["audio"]));
    assert_eq!(body.get("modalities"), None);
    assert_eq!(body["tool_choice"], json!("none"));

    // `response_metadata_correlation: true` — the request id rides in metadata
    // so an automatic server-VAD turn cannot be mistaken for this response.
    let request_id = body["metadata"][via_realtime::RESPONSE_CORRELATION_KEY]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert!(!request_id.is_empty(), "{body}");

    peer.send(json!({
        "type": "response.created",
        "response": {
            "id": "resp_1",
            "metadata": { via_realtime::RESPONSE_CORRELATION_KEY: request_id },
        },
    }));
    peer.send(json!({
        "type": "response.done",
        "response": { "id": "resp_1", "status": "completed" },
    }));
    let outcome = speaking.await.expect("task").expect("speaks");
    assert_eq!(
        outcome.map(|outcome| outcome.kind),
        Some(via_realtime::OutcomeKind::Completed)
    );
}

#[tokio::test(start_paused = true)]
async fn an_s2s_permission_item_is_minted_in_the_ga_message_namespace() {
    // The GA service rejects an id from the wrong namespace outright, so the
    // provider must not mint its own — it asks the dialect. This is the test
    // that the permission item goes through that path.
    let (session, mut peer, _) = open(speech_to_speech()).await;
    let permission = PermissionRequest {
        id: "perm_1".to_owned(),
        summary: "run tests".to_owned(),
    };
    let asking = tokio::spawn(async move {
        session
            .inject_permission(&permission, ResponseContext::new(), None)
            .await
    });

    let item = next_frame_of(&mut peer, "conversation.item.create").await;
    let id = item["item"]["id"].as_str().unwrap_or_default().to_owned();
    assert!(id.starts_with("msg_"), "{item}");
    assert_eq!(
        item["item"]["content"][0]["text"],
        json!(
            "<backend_permission_request>\n\
             authorization_id=perm_1\n\
             operation=run tests\n\
             </backend_permission_request>"
        )
    );

    peer.send(json!({
        "type": "conversation.item.created",
        "item": { "id": id, "status": "completed" },
    }));
    let response = next_frame_of(&mut peer, "response.create").await;
    assert_eq!(response["response"]["tool_choice"], json!("none"));
    assert_eq!(response["response"]["output_modalities"], json!(["audio"]));

    let request_id = response["response"]["metadata"][via_realtime::RESPONSE_CORRELATION_KEY]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    peer.send(json!({
        "type": "response.created",
        "response": {
            "id": "resp_1",
            "metadata": { via_realtime::RESPONSE_CORRELATION_KEY: request_id },
        },
    }));
    peer.send(json!({
        "type": "response.done",
        "response": { "id": "resp_1", "status": "completed" },
    }));
    assert!(asking.await.expect("task").is_ok());
}

#[tokio::test(start_paused = true)]
async fn a_dashscope_result_injection_writes_the_item_then_the_response() {
    let (session, mut peer, _) = open(dashscope()).await;
    let injecting = tokio::spawn(async move {
        session
            .inject_result(
                "构建完成",
                ResponseOrigin::Announcement,
                ResponseContext::new(),
                true,
            )
            .await
    });

    let item = next_frame_of(&mut peer, "conversation.item.create").await;
    // The beta dialect keeps one id namespace for every item type.
    let id = item["item"]["id"].as_str().unwrap_or_default().to_owned();
    assert!(id.starts_with("item_"), "{item}");
    assert_eq!(item["item"]["content"][0]["text"], json!("构建完成"));
    peer.send(json!({
        "type": "conversation.item.created",
        "item": { "id": id, "status": "completed" },
    }));

    let response = next_frame_of(&mut peer, "response.create").await;
    assert_eq!(response["response"]["tool_choice"], json!("none"));
    assert_eq!(response["response"]["modalities"], json!(["text", "audio"]));

    peer.send(json!({ "type": "response.created", "response": { "id": "resp_1" } }));
    peer.send(json!({
        "type": "response.done",
        "response": { "id": "resp_1", "status": "completed" },
    }));
    let outcome = injecting.await.expect("task").expect("injects");
    assert!(outcome.is_some_and(|outcome| outcome.context_injected));
}
