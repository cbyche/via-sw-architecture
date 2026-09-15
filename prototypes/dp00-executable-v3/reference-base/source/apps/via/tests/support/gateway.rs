//! A real Gateway, in this process, with the two seams swapped.
//!
//! `docs/architecture.md` §15's phase-5 milestone is *"`via chat` works end to
//! end — no audio hardware, no model weights"*, and this is what makes that
//! sentence testable rather than aspirational. The Gateway under test is the
//! **same** one `via gateway` boots — [`via::gateway::boot`], the same
//! composition root, the same router, the same accept loop. Two things are
//! substituted, each at the seam its own crate provides:
//!
//! | Seam | Production | Here |
//! | --- | --- | --- |
//! | [`via::gateway::SessionOpener`] | a WebSocket to the provider | `via-realtime-mock`'s in-process transport |
//! | [`via_downstream::DownstreamAgent`] | an ACP child process | [`ScriptedHarness`] |
//!
//! Nothing *around* them is stubbed: the realtime session is the real
//! [`via_realtime::RealtimeSession`], the Work queue is the real
//! [`via_work::WorkManager`], the coordinator is the real
//! [`via_coordinator::Coordinator`], and the announcement window is the real
//! [`via_voice::AnnouncementManager`].

#![allow(dead_code)]

use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use tokio::sync::Mutex;
use via::gateway::{Booted, Composition, SessionOpener};
use via_core::{Config, EnvMap};
use via_downstream::DownstreamAgent;
use via_realtime::{
    RealtimeError, RealtimeProvider, RealtimeSession, SessionEvents, SessionOptions,
};
use via_realtime_mock::{MockHandle, MockRealtime, Script};

/// The opener that hands out `via-realtime-mock` sessions.
///
/// It answers **one** session — a second connection gets a refusal rather than
/// a second copy of the same script, because two connections replaying one
/// script is a source of test flake and never a thing under test.
#[derive(Debug)]
pub struct MockOpener {
    script: Mutex<Option<Script>>,
    handle: Mutex<Option<MockHandle>>,
}

impl MockOpener {
    /// An opener that will replay `script` once.
    #[must_use]
    pub fn new(script: Script) -> Arc<Self> {
        Arc::new(Self {
            script: Mutex::new(Some(script)),
            handle: Mutex::new(None),
        })
    }

    /// An opener that refuses every session, as a provider whose endpoint is
    /// unreachable does.
    #[must_use]
    pub fn refusing() -> Arc<Self> {
        Arc::new(Self {
            script: Mutex::new(None),
            handle: Mutex::new(None),
        })
    }

    /// The script server's handle, once a session has been opened.
    pub async fn handle(&self) -> Option<MockHandle> {
        self.handle.lock().await.clone()
    }
}

#[async_trait::async_trait]
impl SessionOpener for MockOpener {
    async fn open(
        &self,
        _provider: &Arc<dyn RealtimeProvider>,
        options: SessionOptions,
    ) -> Result<(RealtimeSession, SessionEvents), RealtimeError> {
        let Some(script) = self.script.lock().await.take() else {
            return Err(RealtimeError::ConnectionClosed {
                label: "MockOpener".to_owned(),
            });
        };
        let (handle, opened) = MockRealtime::new(script)
            .with_options(options)
            .try_open()
            .await;
        *self.handle.lock().await = Some(handle);
        opened
    }
}

/// A Gateway that is listening, and the pieces a test drives it with.
pub struct Harness {
    /// The bound Gateway.
    booted: Option<Booted>,
    /// The task serving it.
    served: Option<tokio::task::JoinHandle<()>>,
    /// The script server, for reading what the session actually wrote.
    pub opener: Arc<MockOpener>,
    /// Where its lease and its files live.
    _root: tempfile::TempDir,
    /// `http://127.0.0.1:<port>`.
    pub origin: String,
}

impl Harness {
    /// Boot a Gateway with `script` in front of the model and `harness` behind
    /// the coordinator.
    pub async fn start(script: Script, harness: Arc<dyn DownstreamAgent>) -> Self {
        Self::with_opener(MockOpener::new(script), harness).await
    }

    /// Boot a Gateway whose realtime front end refuses to open at all.
    pub async fn refusing_realtime(harness: Arc<dyn DownstreamAgent>) -> Self {
        Self::with_opener(MockOpener::refusing(), harness).await
    }

    /// Boot a Gateway over an explicit opener.
    pub async fn with_opener(opener: Arc<MockOpener>, harness: Arc<dyn DownstreamAgent>) -> Self {
        let root = tempfile::TempDir::new().expect("a temporary directory");
        let environment: EnvMap = [
            ("DASHSCOPE_API_KEY", "sk-test"),
            ("VIA_AUTH_SECRET", &"a".repeat(64)),
            (
                "VIA_CONFIG_DIR",
                &root.path().join("config").display().to_string(),
            ),
            (
                "VIA_DATA_DIR",
                &root.path().join("config").display().to_string(),
            ),
            ("HOST", "127.0.0.1"),
            ("PORT", "0"),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect();
        let overrides = via_core::config::Overrides {
            home_directory: root.path().join("home"),
            working_directory: root.path().join("cwd"),
            ..via_core::config::Overrides::default()
        };
        let config: Config = via_core::config::resolve(&environment, None, &overrides)
            .expect("a resolvable configuration");
        std::fs::create_dir_all(config.config_directory()).expect("the configuration directory");

        let provider = MockRealtime::new(Script::conversation())
            .provider()
            .expect("the mock provider validates");
        let composition = Composition {
            opener: Arc::clone(&opener) as Arc<dyn SessionOpener>,
            providers: vec![provider],
            harness: Some(harness),
            logger: Some(Arc::new(via_log::Logger::with_sinks(
                via_log::LoggerOptions::detached("gateway"),
                Vec::new(),
            ))),
        };

        let booted = via::gateway::boot(&config, &environment, composition)
            .await
            .expect("the Gateway boots");
        let origin = booted.origin();
        Self {
            booted: Some(booted),
            served: None,
            opener,
            _root: root,
            origin,
        }
    }

    /// Start serving. Must be called before a client connects.
    pub fn serve(&mut self) {
        let booted = self.booted.take().expect("start() ran");
        let Booted {
            serving,
            composed,
            lease,
            heartbeat,
        } = booted;
        // The lease and the heartbeat are the process's, not the server's; a
        // test owns the process, so it holds them until the drop.
        drop(heartbeat);
        let _ = lease;
        let _ = composed;
        self.served = Some(tokio::spawn(serving.run()));
    }

    /// The realtime provider key this Gateway serves.
    pub const PROVIDER: &'static str = via_realtime_mock::MOCK_PROVIDER_KEY;
}

/// One client socket, and the frames it has seen.
pub struct Client {
    sink: futures::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        tokio_tungstenite::tungstenite::Message,
    >,
    stream: futures::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    /// Everything received, in order.
    pub frames: Vec<serde_json::Value>,
}

impl Client {
    /// Fetch `/api/health`, take the cookie, and open the socket — which is
    /// exactly `via chat`'s own opening sequence.
    pub async fn connect(origin: &str, session_id: &str) -> Self {
        let mut http =
            via::chat::GatewayClient::new(origin, via_i18n::Locale::En).expect("an http client");
        let health = http
            .health()
            .await
            .expect("the Gateway answers")
            .expect("…and it is a Gateway");
        assert_eq!(health["ok"], serde_json::Value::Bool(true));

        let url = via::chat::protocol::websocket_url(origin, session_id).expect("a socket url");
        let mut request =
            tokio_tungstenite::tungstenite::client::IntoClientRequest::into_client_request(
                url.as_str(),
            )
            .expect("a request");
        // `const headers = cookie ? { Cookie: cookie } : {}` — in `personal`
        // identity mode there is one owner and no cookie to carry, and the
        // upgrade resolves the personal identity directly.
        if let Some(cookie) = http.cookie() {
            request
                .headers_mut()
                .insert("cookie", cookie.parse().expect("a header value"));
        }
        let (socket, _response) = tokio_tungstenite::connect_async(request)
            .await
            .expect("the upgrade is accepted");
        let (sink, stream) = socket.split();
        Self {
            sink,
            stream,
            frames: Vec::new(),
        }
    }

    /// Send one frame.
    pub async fn send(&mut self, frame: serde_json::Value) {
        self.sink
            .send(tokio_tungstenite::tungstenite::Message::Text(
                frame.to_string().into(),
            ))
            .await
            .expect("the socket accepts a frame");
    }

    /// Send the `connect` frame `via chat` sends.
    pub async fn hello(&mut self) {
        self.send(via::chat::protocol::connect_frame(
            false,
            "UTC",
            via_i18n::Locale::En,
        ))
        .await;
    }

    /// Type a line, exactly as `via chat` does.
    pub async fn say(&mut self, text: &str) {
        self.send(serde_json::json!({
            "type": "input.message",
            "text": text,
            "textOnly": true,
        }))
        .await;
    }

    /// Read frames until `predicate` matches one, or the budget is spent.
    ///
    /// The budget is wall-clock rather than a frame count: what is being waited
    /// for is a Work item moving through a queue and a coordinator turn, and
    /// neither produces a predictable number of frames.
    pub async fn wait_for(
        &mut self,
        budget: std::time::Duration,
        predicate: impl Fn(&serde_json::Value) -> bool,
    ) -> Option<serde_json::Value> {
        let deadline = tokio::time::Instant::now() + budget;
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return None;
            }
            let Ok(Some(Ok(message))) = tokio::time::timeout(remaining, self.stream.next()).await
            else {
                return None;
            };
            let tokio_tungstenite::tungstenite::Message::Text(text) = message else {
                continue;
            };
            let Ok(frame) = serde_json::from_str::<serde_json::Value>(&text) else {
                continue;
            };
            self.frames.push(frame.clone());
            if predicate(&frame) {
                return Some(frame);
            }
        }
    }

    /// Every frame of `kind` seen so far.
    pub fn seen(&self, kind: &str) -> Vec<&serde_json::Value> {
        self.frames
            .iter()
            .filter(|frame| frame["type"] == kind)
            .collect()
    }

    /// The concatenated assistant text seen so far.
    pub fn assistant_text(&self) -> String {
        self.frames
            .iter()
            .filter(|frame| frame["type"] == "transcript.final" && frame["role"] == "assistant")
            .filter_map(|frame| frame["content"].as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// A frame matcher for one `type`.
pub fn is(kind: &'static str) -> impl Fn(&serde_json::Value) -> bool {
    move |frame| frame["type"] == kind
}

/// A harness whose turn does not finish until it is released.
///
/// A [`ScriptedHarness`](via_downstream::testing::ScriptedHarness) answers
/// instantly, which is right for almost everything and wrong for exactly one
/// thing: a Work that has to still be **live** when a client asks to cancel it.
/// This wraps one and holds `prompt` open on a
/// [`Notify`](tokio::sync::Notify), so `/tasks` sees a `running` delegation and
/// `/cancel` acts on something that has not already finished.
///
/// Everything else is delegated: the descriptor, the session id, the event
/// stream, and — importantly — `cancel`, so the cancellation path under test is
/// the harness's own.
pub struct HeldHarness {
    inner: Arc<dyn DownstreamAgent>,
    release: Arc<tokio::sync::Notify>,
}

impl HeldHarness {
    /// Wrap `inner`, holding every turn until [`release`](Self::release) is
    /// called.
    #[must_use]
    pub fn new(inner: Arc<dyn DownstreamAgent>) -> Arc<Self> {
        Arc::new(Self {
            inner,
            release: Arc::new(tokio::sync::Notify::new()),
        })
    }

    /// Let the turn in flight finish.
    pub fn release(&self) {
        self.release.notify_waiters();
    }
}

#[async_trait::async_trait]
impl DownstreamAgent for HeldHarness {
    fn descriptor(&self) -> &via_downstream::HarnessDescriptor {
        self.inner.descriptor()
    }

    async fn open(
        &self,
        key: &via_downstream::SessionKey,
    ) -> Result<Box<dyn via_downstream::HarnessSession>, via_downstream::HarnessError> {
        Ok(Box::new(HeldSession {
            inner: self.inner.open(key).await?,
            release: Arc::clone(&self.release),
        }))
    }

    async fn health(&self) -> via_downstream::HarnessHealth {
        self.inner.health().await
    }
}

struct HeldSession {
    inner: Box<dyn via_downstream::HarnessSession>,
    release: Arc<tokio::sync::Notify>,
}

#[async_trait::async_trait]
impl via_downstream::HarnessSession for HeldSession {
    fn session_id(&self) -> &str {
        self.inner.session_id()
    }

    async fn prompt(
        &self,
        request: via_downstream::PromptRequest,
    ) -> Result<via_downstream::PromptOutcome, via_downstream::HarnessError> {
        self.release.notified().await;
        self.inner.prompt(request).await
    }

    fn events(&self) -> futures::stream::BoxStream<'static, via_downstream::SessionEvent> {
        self.inner.events()
    }

    async fn cancel(
        &self,
        scope: via_downstream::CancelScope,
    ) -> Result<via_downstream::CancelOutcome, via_downstream::HarnessError> {
        self.inner.cancel(scope).await
    }
}
