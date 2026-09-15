//! The wrapped transport, frame by frame.
//!
//! [`wire`](via_realtime_openai::wire) is what sits between the proved socket
//! and `via-realtime`'s session machine, and it makes four decisions no layer
//! above it can: replay the frame the probe consumed, normalise Binary framing,
//! answer a GA-discriminator rejection instead of surfacing it, and drop the
//! relay's own duplicate `response.create` conflicts.
//!
//! Driven over in-memory channels rather than a socket, because every one of
//! those decisions is about a frame and none of them is about TCP —
//! `tests/gateway.rs` is where the socket is real.

use std::time::Duration;

use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message;
use via_realtime::Transport;
use via_realtime_openai::{RealtimeSchema, protocol_for_url, wire};

/// The pre-GA payload the repair resends. Its exact bytes are the contract:
/// the repair is a **byte-identical** resend of the payload the walk built.
const PREVIEW_SETUP: &str = r#"{"event_id":"event_preview","type":"session.update","session":{"modalities":["text","audio"],"instructions":"be brief","voice":"alloy"}}"#;

struct Wired {
    transport: Transport,
    /// Frames the peer would have received.
    written: mpsc::UnboundedReceiver<Message>,
    /// Frames to hand the peer's read half.
    peer: mpsc::UnboundedSender<Result<Message, String>>,
}

fn wired(url: &str, first_frame: Message) -> Wired {
    let (write, written) = mpsc::unbounded::<Message>();
    let (peer, read) = mpsc::unbounded::<Result<Message, String>>();
    let transport = wire(write, read, first_frame, url, PREVIEW_SETUP.to_owned());
    Wired {
        transport,
        written,
        peer,
    }
}

fn event(value: &Value) -> Message {
    Message::Text(value.to_string().into())
}

fn binary(value: &Value) -> Message {
    Message::Binary(value.to_string().into_bytes().into())
}

/// The next frame the session would see, or `None` within a short grace period.
async fn next_inbound(wired: &mut Wired) -> Option<Message> {
    match tokio::time::timeout(Duration::from_millis(150), wired.transport.stream.next()).await {
        Ok(Some(Ok(message))) => Some(message),
        Ok(Some(Err(detail))) => Some(Message::Text(format!("<error> {detail}").into())),
        Ok(None) | Err(_) => None,
    }
}

/// The next frame the peer would see, or `None` within a short grace period.
async fn next_written(wired: &mut Wired) -> Option<Message> {
    tokio::time::timeout(Duration::from_millis(150), wired.written.next())
        .await
        .unwrap_or_default()
}

fn kind_of(message: &Message) -> String {
    let Message::Text(text) = message else {
        panic!("the session is handed Text framing, got {message:?}");
    };
    serde_json::from_str::<Value>(text)
        .ok()
        .and_then(|value| value.get("type").and_then(Value::as_str).map(str::to_owned))
        .unwrap_or_else(|| text.to_string())
}

#[tokio::test]
async fn the_frame_the_probe_consumed_is_replayed() {
    // On a real endpoint it is `session.created`, which is what makes the
    // session machine configure itself at all. It could as easily be an `error`
    // explaining a rejected configuration, and dropping that would turn a
    // diagnosable failure back into silence.
    let mut wired = wired(
        "wss://replay.invalid/v1/realtime",
        event(&json!({ "type": "session.created", "session": {} })),
    );
    let first = next_inbound(&mut wired).await.expect("the replay");
    assert_eq!(kind_of(&first), "session.created");
}

#[tokio::test]
async fn a_replayed_error_frame_is_not_swallowed() {
    let mut wired = wired(
        "wss://replay-error.invalid/v1/realtime",
        event(&json!({ "type": "error", "error": { "message": "model not found" } })),
    );
    let first = next_inbound(&mut wired).await.expect("the replay");
    assert_eq!(kind_of(&first), "error");
}

#[tokio::test]
async fn binary_framing_is_normalised_so_no_layer_above_has_to_know() {
    let mut wired = wired(
        "wss://binary.invalid/v1/realtime",
        binary(&json!({ "type": "session.created" })),
    );
    assert_eq!(
        kind_of(&next_inbound(&mut wired).await.expect("the replay")),
        "session.created"
    );

    wired
        .peer
        .unbounded_send(Ok(binary(
            &json!({ "type": "response.output_audio.delta", "delta": "QUJD" }),
        )))
        .expect("queued");
    let delta = next_inbound(&mut wired).await.expect("the delta");
    assert_eq!(kind_of(&delta), "response.output_audio.delta");
    // Byte for byte, not re-serialised: a payload the session machine has to
    // read must not be rewritten on the way through.
    let Message::Text(text) = &delta else {
        panic!("normalised to Text");
    };
    assert_eq!(
        text.as_str(),
        json!({ "type": "response.output_audio.delta", "delta": "QUJD" }).to_string()
    );
}

#[tokio::test]
async fn keepalives_are_dropped_and_a_close_is_forwarded() {
    let mut wired = wired(
        "wss://keepalive.invalid/v1/realtime",
        event(&json!({ "type": "session.created" })),
    );
    let _ = next_inbound(&mut wired).await;

    for keepalive in [
        Message::Ping(Vec::new().into()),
        Message::Pong(Vec::new().into()),
        Message::Binary(vec![0xff].into()),
    ] {
        wired.peer.unbounded_send(Ok(keepalive)).expect("queued");
    }
    wired
        .peer
        .unbounded_send(Ok(event(&json!({ "type": "session.updated" }))))
        .expect("queued");

    // The keepalives never reach the session; the next frame it sees is the one
    // that follows them.
    assert_eq!(
        kind_of(&next_inbound(&mut wired).await.expect("the update")),
        "session.updated"
    );

    wired
        .peer
        .unbounded_send(Ok(Message::Close(None)))
        .expect("queued");
    assert!(matches!(
        wired.transport.stream.next().await,
        Some(Ok(Message::Close(_)))
    ));
}

#[tokio::test]
async fn text_that_is_not_json_is_forwarded_rather_than_swallowed() {
    // `via-realtime` skips what it cannot parse; hiding it here would conceal a
    // relay that started emitting something else entirely.
    let mut wired = wired(
        "wss://garbage.invalid/v1/realtime",
        event(&json!({ "type": "session.created" })),
    );
    let _ = next_inbound(&mut wired).await;

    wired
        .peer
        .unbounded_send(Ok(Message::Text("<html>502</html>".into())))
        .expect("queued");
    let forwarded = next_inbound(&mut wired).await.expect("forwarded");
    assert_eq!(forwarded, Message::Text("<html>502</html>".into()));
}

#[tokio::test]
async fn a_read_failure_reaches_the_session_as_a_transport_error() {
    let mut wired = wired(
        "wss://readfail.invalid/v1/realtime",
        event(&json!({ "type": "session.created" })),
    );
    let _ = next_inbound(&mut wired).await;
    wired
        .peer
        .unbounded_send(Err("connection reset".to_owned()))
        .expect("queued");
    assert_eq!(
        wired.transport.stream.next().await,
        Some(Err("connection reset".to_owned()))
    );
}

// ── the in-session schema repair ─────────────────────────────────────────────

#[tokio::test]
async fn the_ga_rejection_is_answered_on_the_same_socket_and_never_surfaces() {
    let url = "wss://transport-repair.invalid/v1/realtime?model=m";
    let mut wired = wired(url, event(&json!({ "type": "session.created" })));
    let _ = next_inbound(&mut wired).await;

    wired
        .peer
        .unbounded_send(Ok(event(&json!({
            "type": "error",
            "error": { "message": "Unknown parameter: 'session.type'." }
        }))))
        .expect("queued");

    // The repair goes out, byte-identically.
    let resent = next_written(&mut wired).await.expect("the repair");
    assert_eq!(resent, Message::Text(PREVIEW_SETUP.into()));
    // And nothing reached the session — an `error` before the session is ready
    // is what rejects `connect()`, so surfacing it would kill the session the
    // repair just fixed.
    assert!(next_inbound(&mut wired).await.is_none());
    // The endpoint's verdict is remembered for the process.
    assert_eq!(protocol_for_url(url), RealtimeSchema::Preview);
}

#[tokio::test]
async fn the_repair_is_sent_once_and_the_second_rejection_resends_nothing() {
    // "Once, not in a loop": if the pre-GA schema is refused too, the endpoint
    // disagrees with both generations. VIA still swallows the second rejection,
    // because it is a knock-on from the payload the session machine had already
    // put on the wire — but it does not answer it.
    let url = "wss://transport-repair-once.invalid/v1/realtime?model=m";
    let mut wired = wired(url, event(&json!({ "type": "session.created" })));
    let _ = next_inbound(&mut wired).await;

    let rejection = event(&json!({
        "type": "error",
        "error": { "message": "Unknown parameter: 'session.type'." }
    }));
    wired
        .peer
        .unbounded_send(Ok(rejection.clone()))
        .expect("queued");
    assert_eq!(
        next_written(&mut wired).await.expect("the repair"),
        Message::Text(PREVIEW_SETUP.into())
    );

    wired.peer.unbounded_send(Ok(rejection)).expect("queued");
    assert!(
        next_written(&mut wired).await.is_none(),
        "the second rejection must not resend"
    );
    assert!(next_inbound(&mut wired).await.is_none());
}

#[tokio::test]
async fn an_unrelated_parameter_rejection_is_not_a_schema_repair() {
    // Over-matching would swallow a real configuration error and resend over it.
    let mut wired = wired(
        "wss://transport-other-param.invalid/v1/realtime",
        event(&json!({ "type": "session.created" })),
    );
    let _ = next_inbound(&mut wired).await;

    wired
        .peer
        .unbounded_send(Ok(event(&json!({
            "type": "error",
            "error": { "message": "Unknown parameter: 'session.voice'." }
        }))))
        .expect("queued");
    assert_eq!(
        kind_of(&next_inbound(&mut wired).await.expect("surfaced")),
        "error"
    );
    assert!(next_written(&mut wired).await.is_none());
}

// ── the duplicate-`response.create` filter ───────────────────────────────────

#[tokio::test]
async fn the_gateways_own_duplicate_is_dropped_and_ours_is_not() {
    let mut wired = wired(
        "wss://transport-conflict.invalid/v1/realtime",
        event(&json!({ "type": "session.created" })),
    );
    let _ = next_inbound(&mut wired).await;

    // The bring-up §8b shape: server VAD opens a response, then the relay's own
    // duplicate bounces off it, with zero outbound frames from us in between.
    wired
        .peer
        .unbounded_send(Ok(event(
            &json!({ "type": "response.created", "response": { "id": "resp_x" } }),
        )))
        .expect("queued");
    assert_eq!(
        kind_of(&next_inbound(&mut wired).await.expect("the response")),
        "response.created"
    );

    let conflict = event(&json!({
        "type": "error",
        "error": { "message": "Conversation already has an active response in progress: resp_x." }
    }));
    wired
        .peer
        .unbounded_send(Ok(conflict.clone()))
        .expect("queued");
    assert!(
        next_inbound(&mut wired).await.is_none(),
        "a conflict with zero creates unaccounted for cannot be ours"
    );

    // Now the session asks for a response of its own. The very next conflict
    // could concern it, so it surfaces — a client regression that double-sends
    // must stay visible.
    wired
        .transport
        .sink
        .send(event(
            &json!({ "event_id": "event_1", "type": "response.create" }),
        ))
        .await
        .expect("written");
    assert_eq!(
        next_written(&mut wired).await.expect("the create"),
        event(&json!({ "event_id": "event_1", "type": "response.create" }))
    );

    wired
        .peer
        .unbounded_send(Ok(conflict.clone()))
        .expect("queued");
    assert_eq!(
        kind_of(&next_inbound(&mut wired).await.expect("surfaced")),
        "error"
    );

    // And it consumed its slot: the next one is noise again.
    wired.peer.unbounded_send(Ok(conflict)).expect("queued");
    assert!(next_inbound(&mut wired).await.is_none());
}

#[tokio::test]
async fn a_burst_of_server_responses_cannot_make_a_later_conflict_look_like_ours() {
    // The decrement is saturating. An unsaturated counter would go negative on
    // the first server-VAD turn and never recover, and then every conflict for
    // the rest of the session would be attributed to the client.
    let mut wired = wired(
        "wss://transport-saturate.invalid/v1/realtime",
        event(&json!({ "type": "session.created" })),
    );
    let _ = next_inbound(&mut wired).await;

    for _ in 0..5 {
        wired
            .peer
            .unbounded_send(Ok(event(&json!({ "type": "response.created" }))))
            .expect("queued");
        assert_eq!(
            kind_of(&next_inbound(&mut wired).await.expect("the response")),
            "response.created"
        );
    }

    wired
        .peer
        .unbounded_send(Ok(event(&json!({
            "type": "error",
            "error": { "code": "conversation_already_has_active_response" }
        }))))
        .expect("queued");
    assert!(next_inbound(&mut wired).await.is_none());
}

#[tokio::test]
async fn audio_appends_never_touch_the_ledger() {
    // Fifty frames a second, none of which asks for a response. If they counted,
    // the very first utterance would make every later conflict look like ours.
    let mut wired = wired(
        "wss://transport-audio.invalid/v1/realtime",
        event(&json!({ "type": "session.created" })),
    );
    let _ = next_inbound(&mut wired).await;

    for _ in 0..20 {
        wired
            .transport
            .sink
            .send(event(
                &json!({ "type": "input_audio_buffer.append", "audio": "QUJD" }),
            ))
            .await
            .expect("written");
        let _ = next_written(&mut wired).await;
    }

    wired
        .peer
        .unbounded_send(Ok(event(&json!({
            "type": "error",
            "error": { "message": "already has an active response" }
        }))))
        .expect("queued");
    assert!(next_inbound(&mut wired).await.is_none());
}

#[tokio::test]
async fn a_close_written_by_the_session_reaches_the_peer_behind_everything_queued() {
    // The write half is a single FIFO drained by one task, so a `send` followed
    // by a `close` is flushed before the close frame goes out. No separate drain
    // step is needed anywhere above this.
    let mut wired = wired(
        "wss://transport-close.invalid/v1/realtime",
        event(&json!({ "type": "session.created" })),
    );
    let _ = next_inbound(&mut wired).await;

    wired
        .transport
        .sink
        .send(event(&json!({ "type": "response.cancel" })))
        .await
        .expect("written");
    wired
        .transport
        .sink
        .send(Message::Close(None))
        .await
        .expect("written");

    assert_eq!(
        kind_of(&next_written(&mut wired).await.expect("the cancel")),
        "response.cancel"
    );
    assert!(matches!(
        next_written(&mut wired).await,
        Some(Message::Close(_))
    ));
}
