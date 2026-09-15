//! The one thing the in-memory transport cannot prove: that a real socket works.
//!
//! Ported from `server/test/realtime-provider.test.mjs:1824-1968`, which stands
//! up a `ws` server on an ephemeral port. Everything else about the session is
//! tested over [`via_realtime::testing::test_transport`]; what is here is the
//! part that only a real WebSocket exercises — the upgrade request, the
//! provider's headers on it, JSON over text frames, and a refusal that arrives
//! before the session is usable.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::Message;
use via_realtime::testing::TestProvider;
use via_realtime::{RealtimeError, RealtimeSession, SessionOptions};

/// A handshake callback that only remembers the request's headers.
///
/// A closure would do, but `Callback::Error` is a whole `Response`, so a closure
/// returning `Ok(response)` still declares a very large `Err` variant.
struct Recorder<'a>(&'a mut HashMap<String, String>);

impl tokio_tungstenite::tungstenite::handshake::server::Callback for Recorder<'_> {
    fn on_request(
        self,
        request: &tokio_tungstenite::tungstenite::handshake::server::Request,
        response: tokio_tungstenite::tungstenite::handshake::server::Response,
    ) -> Result<
        tokio_tungstenite::tungstenite::handshake::server::Response,
        tokio_tungstenite::tungstenite::handshake::server::ErrorResponse,
    > {
        for (name, value) in request.headers() {
            if let Ok(value) = value.to_str() {
                self.0
                    .insert(name.as_str().to_lowercase(), value.to_owned());
            }
        }
        Ok(response)
    }
}

/// What one accepted connection saw.
struct Accepted {
    headers: HashMap<String, String>,
    frames: Vec<Value>,
}

/// A one-connection realtime server.
///
/// `script` is what the server sends back, keyed by the `type` of the frame that
/// triggered it; `greeting` is written the moment the socket opens.
async fn serve(
    greeting: Vec<Value>,
    script: Vec<(&'static str, Value)>,
) -> (SocketAddr, oneshot::Receiver<Accepted>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("an ephemeral port");
    let address = listener.local_addr().expect("an address");
    let (report, accepted) = oneshot::channel();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("a connection");
        let mut headers = HashMap::new();
        let socket = tokio_tungstenite::accept_hdr_async(stream, Recorder(&mut headers))
            .await
            .expect("a websocket");
        let (mut sink, mut stream) = socket.split();

        for message in greeting {
            let text = message.to_string();
            sink.send(Message::Text(text.into())).await.expect("greet");
        }

        let mut frames = Vec::new();
        let mut script: Vec<(&'static str, Value)> = script;
        while !script.is_empty() {
            let Some(Ok(Message::Text(text))) = stream.next().await else {
                break;
            };
            let Ok(frame) = serde_json::from_str::<Value>(&text) else {
                continue;
            };
            let kind = frame
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            frames.push(frame);
            if let Some(index) = script.iter().position(|(trigger, _)| *trigger == kind) {
                let (_, reply) = script.remove(index);
                let text = reply.to_string();
                sink.send(Message::Text(text.into())).await.expect("reply");
            }
        }
        let _ = report.send(Accepted { headers, frames });
    });

    (address, accepted)
}

#[tokio::test]
async fn a_session_opens_over_a_real_websocket_and_configures_itself() {
    let (address, accepted) = serve(
        vec![json!({ "type": "session.created" })],
        vec![("session.update", json!({ "type": "session.updated" }))],
    )
    .await;

    let provider = TestProvider::new("beta")
        .with_url(&format!("ws://{address}/v1/realtime"))
        .with_headers([("authorization".to_owned(), "Bearer secret".to_owned())])
        .shared();
    let (session, _events) = RealtimeSession::connect(
        Arc::clone(&provider),
        SessionOptions {
            agent_context: via_realtime::AgentContext {
                instructions: "be brief".into(),
                ..Default::default()
            },
            ..SessionOptions::default()
        },
    )
    .await
    .expect("the session opens");

    let seen = accepted.await.expect("the server reported");
    assert_eq!(
        seen.headers.get("authorization").map(String::as_str),
        Some("Bearer secret")
    );
    assert_eq!(
        seen.frames
            .iter()
            .map(|frame| frame["type"].clone())
            .collect::<Vec<_>>(),
        [json!("session.update")]
    );
    assert_eq!(seen.frames[0]["session"]["instructions"], json!("be brief"));
    assert_eq!(session.provider().key(), "beta");
}

#[tokio::test]
async fn a_provider_with_no_credential_sends_no_authorization_header() {
    let (address, accepted) = serve(
        vec![json!({ "type": "session.created" })],
        vec![("session.update", json!({ "type": "session.updated" }))],
    )
    .await;
    let provider = TestProvider::new("s2s-like")
        .with_url(&format!("ws://{address}/v1/realtime"))
        .shared();
    let (_session, _events) = RealtimeSession::connect(provider, SessionOptions::default())
        .await
        .expect("the session opens");

    let seen = accepted.await.expect("the server reported");
    assert_eq!(seen.headers.get("authorization"), None);
}

#[tokio::test]
async fn a_busy_session_slot_rejects_the_connect_immediately() {
    // The upstream case this exists for: a speech-to-speech service releases its
    // single session slot asynchronously, so an immediate reconnect after a
    // wake gets `session_limit_reached`. It has to fail fast — the caller's
    // backoff is what retries, and it cannot back off while it is still waiting.
    let (address, _accepted) = serve(
        vec![json!({
            "type": "error",
            "error": {
                "type": "session_limit_reached",
                "message": "All 1 session slots are in use. Disconnect an existing client first.",
            },
        })],
        Vec::new(),
    )
    .await;

    let provider = TestProvider::ga("s2s-like")
        .with_url(&format!("ws://{address}/v1/realtime"))
        .shared();
    let error = RealtimeSession::connect(provider, SessionOptions::default())
        .await
        .expect_err("the slot is busy");

    assert!(
        matches!(&error, RealtimeError::ProviderRefused { message }
            if message.contains("session slots are in use")),
        "{error:?}"
    );
}

#[tokio::test]
async fn a_refused_socket_is_a_transport_failure_carrying_the_status_line() {
    // A plain HTTP listener that answers 401 rather than upgrading — which is
    // how an expired credential presents, and the text a provider's
    // `classify_error` recognises as `fatal`.
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("an ephemeral port");
    let address = listener.local_addr().expect("an address");
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("a connection");
        use tokio::io::AsyncWriteExt;
        let _ = stream
            .write_all(b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n")
            .await;
        let _ = stream.shutdown().await;
    });

    let provider = TestProvider::new("beta")
        .with_url(&format!("ws://{address}/v1/realtime"))
        .shared();
    let error = RealtimeSession::connect(provider, SessionOptions::default())
        .await
        .expect_err("refused");

    let RealtimeError::Transport { detail } = &error else {
        panic!("expected a transport failure, got {error:?}");
    };
    assert!(
        detail.contains("401"),
        "the status must survive for `classify_error`: {detail}"
    );
    // Worth stating out loud for whoever writes a real provider: `tungstenite`
    // does *not* phrase this the way Node's `ws` does. The catalogued DashScope
    // `fatal` corpus includes `unexpected server response: (?:401|403)`, which
    // is `ws`'s wording; a Rust provider's classifier has to cover both, and
    // this is why the text is carried through untranslated.
    assert!(
        !detail.to_lowercase().contains("unexpected server response"),
        "if this starts passing, tungstenite changed its wording: {detail}"
    );
}

#[tokio::test]
async fn an_unreachable_endpoint_is_a_transport_failure() {
    // Port 1 on the loopback: nothing listens, and the refusal must not be
    // mistaken for a timeout.
    let provider = TestProvider::new("beta")
        .with_url("ws://127.0.0.1:1/v1/realtime")
        .shared();
    let error = RealtimeSession::connect(provider, SessionOptions::default())
        .await
        .expect_err("refused");
    assert_eq!(error.code(), "VIA_REALTIME_TRANSPORT");
}
