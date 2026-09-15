//! Driving a session without a socket.
//!
//! Upstream's tests reach inside `RealtimeFrontend` — they overwrite
//! `frontend.send`, push objects into `frontend.pendingResponses`, and call
//! `handleLifecycle` directly. None of that is available or desirable here, so
//! everything is driven through the two real doors instead: frames in through
//! [`via_realtime::testing::TestPeer`], calls in through the public API. The
//! result is a strictly stronger test — it exercises the transport decode, the
//! command channels and both owning tasks rather than skipping past them.

#![allow(dead_code)]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};
use via_realtime::testing::{TestPeer, TestProvider, test_transport};
use via_realtime::{
    Diagnostic, ProviderEvent, RealtimeError, RealtimeProvider, RealtimeSession, SessionEvent,
    SessionEvents, SessionOptions,
};

/// One event the session reported, in a form a test can keep.
#[derive(Debug, Clone)]
pub enum Recorded {
    Provider(ProviderEvent),
    Error(RealtimeError),
    Diagnostic(Diagnostic),
    Closed,
}

impl Recorded {
    pub fn provider(&self) -> Option<&ProviderEvent> {
        match self {
            Self::Provider(event) => Some(event),
            _ => None,
        }
    }

    pub fn error(&self) -> Option<&RealtimeError> {
        match self {
            Self::Error(error) => Some(error),
            _ => None,
        }
    }

    pub fn diagnostic(&self) -> Option<&Diagnostic> {
        match self {
            Self::Diagnostic(diagnostic) => Some(diagnostic),
            _ => None,
        }
    }
}

/// Everything the session has reported so far.
#[derive(Clone, Default)]
pub struct EventLog {
    entries: Arc<Mutex<Vec<Recorded>>>,
}

impl EventLog {
    fn push(&self, entry: Recorded) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.push(entry);
        }
    }

    pub fn snapshot(&self) -> Vec<Recorded> {
        self.entries
            .lock()
            .map(|entries| entries.clone())
            .unwrap_or_default()
    }

    pub fn provider_events(&self) -> Vec<ProviderEvent> {
        self.snapshot()
            .iter()
            .filter_map(|entry| entry.provider().cloned())
            .collect()
    }

    pub fn provider_kinds(&self) -> Vec<String> {
        self.provider_events()
            .iter()
            .map(|event| event.kind().to_owned())
            .collect()
    }

    pub fn errors(&self) -> Vec<RealtimeError> {
        self.snapshot()
            .iter()
            .filter_map(|entry| entry.error().cloned())
            .collect()
    }

    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.snapshot()
            .iter()
            .filter_map(|entry| entry.diagnostic().cloned())
            .collect()
    }

    pub fn is_closed(&self) -> bool {
        self.snapshot()
            .iter()
            .any(|entry| matches!(entry, Recorded::Closed))
    }

    /// Yield until `predicate` holds, without advancing the clock.
    ///
    /// `yield_now` rather than `sleep` deliberately: a test that means "let the
    /// two tasks make progress" must not silently fire a 30-second watchdog on
    /// the paused clock while it waits.
    pub async fn wait_for<F>(&self, what: &str, predicate: F) -> Vec<Recorded>
    where
        F: Fn(&[Recorded]) -> bool,
    {
        for _ in 0..2000 {
            let snapshot = self.snapshot();
            if predicate(&snapshot) {
                return snapshot;
            }
            tokio::task::yield_now().await;
        }
        panic!("timed out waiting for {what}; saw {:?}", self.snapshot());
    }

    pub async fn wait_for_kind(&self, kind: &str) -> ProviderEvent {
        let entries = self
            .wait_for(kind, |entries| {
                entries
                    .iter()
                    .filter_map(Recorded::provider)
                    .any(|event| event.kind() == kind)
            })
            .await;
        entries
            .iter()
            .filter_map(Recorded::provider)
            .find(|event| event.kind() == kind)
            .cloned()
            .unwrap_or_else(|| panic!("no {kind}"))
    }
}

/// Drain the session's events into a log.
pub fn collect_events(mut events: SessionEvents) -> EventLog {
    let log = EventLog::default();
    let sink = log.clone();
    tokio::spawn(async move {
        while let Some(event) = events.recv().await {
            sink.push(match event {
                SessionEvent::Provider(event) => Recorded::Provider(*event),
                SessionEvent::Error(error) => Recorded::Error(error),
                SessionEvent::Diagnostic(diagnostic) => Recorded::Diagnostic(diagnostic),
                SessionEvent::Closed => Recorded::Closed,
            });
        }
    });
    log
}

/// A session, its peer, and its event log.
pub struct Harness {
    pub session: RealtimeSession,
    pub peer: TestPeer,
    pub events: EventLog,
    /// The `session.update` the provider wrote during the handshake.
    pub session_update: Value,
}

impl Harness {
    /// Open a session and complete the handshake.
    pub async fn open(provider: Arc<dyn RealtimeProvider>, options: SessionOptions) -> Self {
        let (transport, mut peer) = test_transport();
        let acknowledges = provider.capabilities().acknowledges_session_update;
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
        let (session, events) = RealtimeSession::open(provider, options, transport)
            .await
            .expect("the session opens");
        let (peer, session_update) = handshake.await.expect("handshake task");
        Self {
            session,
            peer,
            events: collect_events(events),
            session_update,
        }
    }

    /// Open a session on the default beta-dialect test provider.
    pub async fn beta() -> Self {
        Self::open(
            TestProvider::new("beta").shared(),
            SessionOptions::default(),
        )
        .await
    }

    /// Open a session on the GA-dialect test provider.
    pub async fn ga() -> Self {
        Self::open(TestProvider::ga("ga").shared(), SessionOptions::default()).await
    }
}

/// How long a test will wait for a frame before calling it a failure.
///
/// Generous, and under `start_paused` it costs nothing: the clock only advances
/// to this deadline once every other task is idle, which is exactly the state a
/// hung test is in. It exists so a regression **fails** rather than hanging CI.
pub const FRAME_DEADLINE: Duration = Duration::from_secs(30);

/// Wait for the next frame of a given type, or fail saying what was expected.
pub async fn expect_frame(peer: &mut TestPeer, kind: &str) -> Value {
    match tokio::time::timeout(FRAME_DEADLINE, peer.next_frame_of(kind)).await {
        Ok(Some(frame)) => frame,
        Ok(None) => panic!("the session closed before writing a `{kind}` frame"),
        Err(_) => panic!("timed out waiting for a `{kind}` frame"),
    }
}

/// Wait for the next frame of any type.
pub async fn expect_any_frame(peer: &mut TestPeer) -> Value {
    match tokio::time::timeout(FRAME_DEADLINE, peer.next_frame()).await {
        Ok(Some(frame)) => frame,
        Ok(None) => panic!("the session closed before writing another frame"),
        Err(_) => panic!("timed out waiting for a frame"),
    }
}

/// Read the next `conversation.item.create` and acknowledge it.
///
/// Returns the frame that was written, so the test can assert on the item.
pub async fn acknowledge_item(peer: &mut TestPeer) -> Value {
    let frame = expect_frame(peer, "conversation.item.create").await;
    let id = frame["item"]["id"].clone();
    peer.send(json!({
        "type": "conversation.item.created",
        "item": { "id": id, "status": "completed" },
    }));
    frame
}

/// Acknowledge the next item with an id the provider chose itself.
///
/// The `conversation_item_id_echo: false` path.
pub async fn acknowledge_item_with_id(peer: &mut TestPeer, id: &str) -> Value {
    let frame = expect_frame(peer, "conversation.item.create").await;
    peer.send(json!({
        "type": "conversation.item.created",
        "item": { "id": id, "status": "completed" },
    }));
    frame
}

/// Read the next `response.create` and answer it end to end.
pub async fn complete_response(peer: &mut TestPeer, response_id: &str) -> Value {
    let frame = start_response(peer, response_id).await;
    finish_response(peer, response_id, "completed");
    frame
}

/// Read the next `response.create` and answer only `response.created`.
///
/// Echoes the GA correlation metadata back when the frame carried it, which is
/// what a metadata-correlating provider does.
pub async fn start_response(peer: &mut TestPeer, response_id: &str) -> Value {
    let frame = expect_frame(peer, "response.create").await;
    let mut response = json!({ "id": response_id });
    if let Some(metadata) = frame.pointer("/response/metadata") {
        response["metadata"] = metadata.clone();
    }
    peer.send(json!({ "type": "response.created", "response": response }));
    frame
}

/// End a response with a status.
pub fn finish_response(peer: &TestPeer, response_id: &str, status: &str) {
    peer.send(json!({
        "type": "response.done",
        "response": { "id": response_id, "status": status },
    }));
}

/// Report output activity for a response, which re-opens the inactivity window.
pub fn response_activity(peer: &TestPeer, response_id: &str) {
    peer.send(json!({
        "type": "response.audio.delta",
        "response_id": response_id,
        "delta": "audio",
    }));
}

/// The provider's own `error` event.
pub fn provider_error(peer: &TestPeer, message: &str) {
    peer.send(json!({ "type": "error", "error": { "message": message } }));
}

/// Session options with both watchdogs shortened, for timeout tests.
pub fn quick_timeouts(start: Duration, inactivity: Duration) -> SessionOptions {
    SessionOptions {
        response_start_timeout: Some(start),
        response_inactivity_timeout: Some(inactivity),
        ..SessionOptions::default()
    }
}

/// A dialect adapter that tags every frame with the connection it belongs to.
///
/// The shape a multiplexing proxy needs, and the reason
/// [`via_realtime::RealtimeProvider::create_protocol`] exists: the adapter
/// closes over the connection id, so it cannot be shared between connections and
/// must be built per connection.
#[derive(Debug)]
pub struct RoutedProtocol {
    connection_id: String,
    beta: via_realtime::OpenAiCompatibleProtocol,
}

impl RoutedProtocol {
    pub fn new(connection_id: &str) -> Self {
        Self {
            connection_id: connection_id.to_owned(),
            beta: via_realtime::openai_compatible_protocol(),
        }
    }

    fn route(&self) -> Value {
        Value::String(self.connection_id.clone())
    }
}

impl via_realtime::RealtimeProtocol for RoutedProtocol {
    fn encode_outgoing(&self, payload: Value) -> Value {
        let mut frame = self.beta.encode_outgoing(payload);
        if let Some(fields) = frame.as_object_mut() {
            fields.insert("route".to_owned(), self.route());
        }
        frame
    }

    fn normalize_incoming(&self, event: Value) -> Vec<Value> {
        if event.get("route") != Some(&self.route()) {
            return Vec::new();
        }
        let mut event = event;
        if let Some(fields) = event.as_object_mut() {
            fields.shift_remove("route");
        }
        vec![event]
    }

    fn session_update(&self, session: Value) -> Value {
        self.beta.session_update(session)
    }

    fn audio_append(&self, audio: &str) -> Value {
        self.beta.audio_append(audio)
    }

    fn conversation_item_id(&self, item: &Value) -> String {
        self.beta.conversation_item_id(item)
    }

    fn conversation_item_create(&self, item: Value) -> Value {
        self.beta.conversation_item_create(item)
    }

    fn response_create(&self, response: Option<Value>) -> Value {
        self.beta.response_create(response)
    }

    fn correlate_response_create(&self, payload: Value, request_id: &str) -> Value {
        self.beta.correlate_response_create(payload, request_id)
    }

    fn response_correlation_id(&self, event: &Value) -> String {
        self.beta.response_correlation_id(event)
    }

    fn response_cancel(&self) -> Value {
        self.beta.response_cancel()
    }

    fn user_text_item(&self, text: &str) -> Value {
        self.beta.user_text_item(text)
    }

    fn function_output_item(&self, call_id: &str, output: &Value) -> Value {
        self.beta.function_output_item(call_id, output)
    }

    fn connection_messages(&self) -> Vec<Value> {
        vec![json!({ "type": "start", "route": self.connection_id })]
    }
}
