//! A WebSocket peer that can be told to misbehave in each documented way.
//!
//! ARGO bring-up §4 lists four properties of the litellm gateway, and each one
//! silently broke something. They are *server* behaviours, so a provider double
//! cannot express any of them — this is a real listener on a real socket.
//!
//! | [`Behaviour`] field | The property it reproduces |
//! | --- | --- |
//! | [`binary_frames`](Behaviour::binary_frames) | frames the session as Binary, not Text |
//! | [`require_subprotocol`](Behaviour::require_subprotocol) | accepts the upgrade without `Sec-WebSocket-Protocol: realtime` but never routes it |
//! | [`reject_ga_schema`](Behaviour::reject_ga_schema) | speaks the pre-GA schema on GA's own route |
//! | [`silent`](Behaviour::silent) | accepts a socket on a route it does not bridge, and pings forever |
//!
//! plus [`reject`](Behaviour::reject) for a refused handshake and
//! [`after_ready`](Behaviour::after_ready) for the §8b duplicate-`response.create`
//! shape.

#![allow(dead_code)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};

/// A refused upgrade.
#[derive(Clone, Debug)]
pub struct Rejection {
    /// The HTTP status to answer with.
    pub status: u16,
    /// The response body, if any. `None` reproduces Azure's empty-body 403.
    pub body: Option<String>,
    /// The `x-ms-error-code` header Azure sometimes uses instead of a body.
    pub azure_error_code: Option<String>,
}

impl Rejection {
    /// A bare status with no body — the shape a resource's gateway returns for a
    /// route it does not serve.
    #[must_use]
    pub fn status(status: u16) -> Self {
        Self {
            status,
            body: None,
            azure_error_code: None,
        }
    }

    /// A status with a body.
    #[must_use]
    pub fn with_body(status: u16, body: &str) -> Self {
        Self {
            status,
            body: Some(body.to_owned()),
            azure_error_code: None,
        }
    }

    /// A status with Azure's machine-readable header and no body.
    #[must_use]
    pub fn with_azure_code(status: u16, code: &str) -> Self {
        Self {
            status,
            body: None,
            azure_error_code: Some(code.to_owned()),
        }
    }
}

/// How one route behaves.
#[derive(Clone, Debug, Default)]
pub struct Behaviour {
    /// Refuse the upgrade.
    pub reject: Option<Rejection>,
    /// Accept the socket and then send nothing but Ping keepalives.
    ///
    /// The 101-that-is-not-a-session: a relay accepting a path it does not
    /// bridge. The pings are the second half of it — they kept an earlier
    /// version of the probe alive indefinitely.
    pub silent: bool,
    /// Frame every protocol event as Binary rather than Text.
    pub binary_frames: bool,
    /// Answer a `session.update` carrying `session.type` with
    /// `Unknown parameter: 'session.type'.`
    pub reject_ga_schema: bool,
    /// Refuse the upgrade unless `Sec-WebSocket-Protocol: realtime` is offered.
    pub require_subprotocol: bool,
    /// Do not send `session.created` on connect; wait to be spoken to first.
    pub no_unprompted_created: bool,
    /// Send `session.created` and then never acknowledge a `session.update`.
    ///
    /// A socket that passes the first-frame probe and then leaves the session
    /// unconfigured — which is what the session's own connect timeout is the
    /// backstop for.
    pub never_acknowledge: bool,
    /// Events to emit once the session is configured.
    pub after_ready: Vec<Value>,
    /// Milliseconds to wait before answering a client frame.
    pub reply_delay_ms: u64,
}

impl Behaviour {
    /// A well-behaved GA endpoint.
    #[must_use]
    pub fn healthy() -> Self {
        Self::default()
    }
}

/// One upgrade request the peer saw.
#[derive(Clone, Debug)]
pub struct ObservedRequest {
    /// Path plus query, exactly as dialled.
    pub target: String,
    /// Lowercased header names to values, in arrival order.
    pub headers: Vec<(String, String)>,
}

impl ObservedRequest {
    /// The first value of a header, case-insensitively.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// Whether a header is present at all.
    #[must_use]
    pub fn has_header(&self, name: &str) -> bool {
        self.header(name).is_some()
    }
}

/// A running fake gateway.
pub struct FakeGateway {
    /// The address to point a provider's endpoint at.
    pub addr: SocketAddr,
    requests: Arc<Mutex<Vec<ObservedRequest>>>,
    frames: Arc<Mutex<Vec<Value>>>,
    handle: tokio::task::JoinHandle<()>,
}

impl Drop for FakeGateway {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl FakeGateway {
    /// Listen on `127.0.0.1:0`, routing by path prefix.
    ///
    /// A route matches when the dialled path **starts with** its key; the
    /// longest match wins, and an unmatched path falls back to `default`.
    pub async fn spawn(routes: HashMap<&'static str, Behaviour>, default: Behaviour) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a loopback port");
        let addr = listener.local_addr().expect("a bound address");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let frames = Arc::new(Mutex::new(Vec::new()));

        let handle = tokio::spawn(serve(
            listener,
            routes,
            default,
            Arc::clone(&requests),
            Arc::clone(&frames),
        ));

        Self {
            addr,
            requests,
            frames,
            handle,
        }
    }

    /// One route, everything else refused with a `404`.
    pub async fn only(path: &'static str, behaviour: Behaviour) -> Self {
        Self::spawn(
            HashMap::from([(path, behaviour)]),
            Behaviour {
                reject: Some(Rejection::status(404)),
                ..Behaviour::default()
            },
        )
        .await
    }

    /// A single well-behaved endpoint on every route.
    pub async fn healthy() -> Self {
        Self::spawn(HashMap::new(), Behaviour::healthy()).await
    }

    /// The endpoint string a provider is configured with.
    #[must_use]
    pub fn base_url(&self) -> String {
        format!("ws://{}", self.addr)
    }

    /// Every upgrade the peer saw, in order.
    #[must_use]
    pub fn requests(&self) -> Vec<ObservedRequest> {
        self.requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Every client frame the peer decoded, in order.
    #[must_use]
    pub fn client_frames(&self) -> Vec<Value> {
        self.frames
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Every client frame of one `type`.
    #[must_use]
    pub fn client_frames_of(&self, kind: &str) -> Vec<Value> {
        self.client_frames()
            .into_iter()
            .filter(|frame| frame.get("type").and_then(Value::as_str) == Some(kind))
            .collect()
    }

    /// Wait until at least `count` client frames of `kind` have arrived.
    ///
    /// Returns `false` on timeout, so a test can assert the negative too.
    pub async fn wait_for(&self, kind: &str, count: usize, within: Duration) -> bool {
        let deadline = tokio::time::Instant::now() + within;
        while tokio::time::Instant::now() < deadline {
            if self.client_frames_of(kind).len() >= count {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        self.client_frames_of(kind).len() >= count
    }
}

/// A listener that accepts TCP connections and never speaks HTTP.
///
/// `connect_async` has no timeout of its own, so this is the shape that proves
/// the walk bounds its own upgrade: without that bound one hung handshake
/// outlives the whole connect budget and "connect is bounded" is not true.
pub struct HangingPeer {
    /// The address to point a provider's endpoint at.
    pub addr: SocketAddr,
    handle: tokio::task::JoinHandle<()>,
}

impl Drop for HangingPeer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl HangingPeer {
    /// Listen on `127.0.0.1:0` and hold every connection open, silently.
    pub async fn spawn() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a loopback port");
        let addr = listener.local_addr().expect("a bound address");
        let handle = tokio::spawn(async move {
            let mut held = Vec::new();
            while let Ok((stream, _peer)) = listener.accept().await {
                // Held, not dropped: dropping would close the socket and the
                // client would see a transport fault instead of a hang.
                held.push(stream);
            }
        });
        Self { addr, handle }
    }

    /// The endpoint string a provider is configured with.
    #[must_use]
    pub fn base_url(&self) -> String {
        format!("ws://{}", self.addr)
    }
}

fn route_for<'a>(
    routes: &'a HashMap<&'static str, Behaviour>,
    default: &'a Behaviour,
    path: &str,
) -> &'a Behaviour {
    routes
        .iter()
        .filter(|(prefix, _)| path.starts_with(*prefix))
        .max_by_key(|(prefix, _)| prefix.len())
        .map_or(default, |(_, behaviour)| behaviour)
}

async fn serve(
    listener: TcpListener,
    routes: HashMap<&'static str, Behaviour>,
    default: Behaviour,
    requests: Arc<Mutex<Vec<ObservedRequest>>>,
    frames: Arc<Mutex<Vec<Value>>>,
) {
    loop {
        let Ok((stream, _peer)) = listener.accept().await else {
            return;
        };
        let routes = routes.clone();
        let default = default.clone();
        let requests = Arc::clone(&requests);
        let frames = Arc::clone(&frames);
        tokio::spawn(async move {
            handle(stream, routes, default, requests, frames).await;
        });
    }
}

async fn handle(
    stream: TcpStream,
    routes: HashMap<&'static str, Behaviour>,
    default: Behaviour,
    requests: Arc<Mutex<Vec<ObservedRequest>>>,
    frames: Arc<Mutex<Vec<Value>>>,
) {
    let chosen: Arc<Mutex<Option<Behaviour>>> = Arc::new(Mutex::new(None));
    let chosen_for_callback = Arc::clone(&chosen);

    // `result_large_err`: the `Err` variant is a whole HTTP response, which is
    // large — and is exactly what `tungstenite::handshake::server::Callback`
    // requires. Boxing it does not type-check against the trait.
    #[allow(clippy::result_large_err)]
    let callback = move |request: &Request, mut response: Response| {
        let target = request
            .uri()
            .path_and_query()
            .map_or_else(|| request.uri().path().to_owned(), ToString::to_string);
        let headers: Vec<(String, String)> = request
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    name.as_str().to_ascii_lowercase(),
                    String::from_utf8_lossy(value.as_bytes()).into_owned(),
                )
            })
            .collect();
        let offered_subprotocol = headers
            .iter()
            .any(|(name, value)| name == "sec-websocket-protocol" && value.contains("realtime"));

        requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(ObservedRequest {
                target: target.clone(),
                headers,
            });

        let behaviour = route_for(&routes, &default, request.uri().path()).clone();

        if behaviour.require_subprotocol && !offered_subprotocol {
            // The bring-up shape, made loud: the real gateway accepted the
            // socket and never routed it. A test peer that answered the same way
            // could not tell a missing header from a silent route, so this one
            // refuses instead — the assertion is about the header being sent.
            let mut error = ErrorResponse::new(Some(
                "Sec-WebSocket-Protocol: realtime was not offered".to_owned(),
            ));
            *error.status_mut() = tokio_tungstenite::tungstenite::http::StatusCode::from_u16(426)
                .unwrap_or(tokio_tungstenite::tungstenite::http::StatusCode::BAD_REQUEST);
            return Err(error);
        }

        if let Some(rejection) = &behaviour.reject {
            let mut error = ErrorResponse::new(rejection.body.clone());
            *error.status_mut() =
                tokio_tungstenite::tungstenite::http::StatusCode::from_u16(rejection.status)
                    .unwrap_or(tokio_tungstenite::tungstenite::http::StatusCode::BAD_REQUEST);
            if let Some(code) = &rejection.azure_error_code
                && let Ok(value) = code.parse()
            {
                error.headers_mut().insert("x-ms-error-code", value);
            }
            return Err(error);
        }

        if offered_subprotocol && let Ok(value) = "realtime".parse() {
            response
                .headers_mut()
                .insert("Sec-WebSocket-Protocol", value);
        }
        *chosen_for_callback
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(behaviour);
        Ok(response)
    };

    let Ok(socket) = tokio_tungstenite::accept_hdr_async(stream, callback).await else {
        return;
    };
    let behaviour = chosen
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
        .unwrap_or_default();

    let (mut write, mut read) = socket.split();

    let frame = |event: &Value| -> Message {
        let body = event.to_string();
        if behaviour.binary_frames {
            Message::Binary(body.into_bytes().into())
        } else {
            Message::Text(body.into())
        }
    };

    if behaviour.silent {
        // The 101 that is not a session: keepalives, forever, and nothing else.
        loop {
            if write.send(Message::Ping(Vec::new().into())).await.is_err() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(120)).await;
        }
    }

    if !behaviour.no_unprompted_created {
        // A real endpoint sends this unprompted, immediately — which is what
        // makes the first-frame probe free.
        if write
            .send(frame(&json!({ "type": "session.created", "session": {} })))
            .await
            .is_err()
        {
            return;
        }
    }

    let mut ready_emitted = false;
    while let Some(Ok(message)) = read.next().await {
        let text = match &message {
            Message::Text(text) => text.to_string(),
            Message::Binary(bytes) => match core::str::from_utf8(bytes) {
                Ok(text) => text.to_owned(),
                Err(_) => continue,
            },
            Message::Close(_) => return,
            _ => continue,
        };
        let Ok(event) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        frames
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(event.clone());

        if behaviour.reply_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(behaviour.reply_delay_ms)).await;
        }

        if event.get("type").and_then(Value::as_str) != Some("session.update") {
            continue;
        }

        let carries_discriminator = event
            .get("session")
            .and_then(|session| session.get("type"))
            .is_some();
        if behaviour.reject_ga_schema && carries_discriminator {
            let rejection = json!({
                "type": "error",
                "error": {
                    "type": "invalid_request_error",
                    "message": "Unknown parameter: 'session.type'."
                }
            });
            if write.send(frame(&rejection)).await.is_err() {
                return;
            }
            continue;
        }

        if behaviour.never_acknowledge {
            continue;
        }
        if write
            .send(frame(&json!({ "type": "session.updated", "session": {} })))
            .await
            .is_err()
        {
            return;
        }
        if !ready_emitted {
            ready_emitted = true;
            for event in &behaviour.after_ready {
                if write.send(frame(event)).await.is_err() {
                    return;
                }
            }
        }
    }
}
