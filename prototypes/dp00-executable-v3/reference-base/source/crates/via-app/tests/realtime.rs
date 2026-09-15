//! `WS /api/realtime`, against a real socket.
//!
//! Ported from `server/test/input-suspend-protocol.test.mjs` plus the
//! upgrade half of `server/src/voice/realtime-gateway.mjs`. The first test is
//! the one that cannot be written any other way: the catalogued contract is
//! *"any other pathname => `socket.destroy()` with no HTTP response"*, and the
//! only way to observe "no HTTP response" is to read from a raw TCP socket and
//! get zero bytes.

mod support;

use std::time::Duration;

use futures::{SinkExt, StreamExt};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use support::Harness;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use via_app::realtime::engine::EngineCall;

type Socket = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>;

/// The raw bytes of a WebSocket upgrade to `path`.
fn upgrade_request(authority: &str, path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
         Host: {authority}\r\n\
         Connection: Upgrade\r\n\
         Upgrade: websocket\r\n\
         Sec-WebSocket-Version: 13\r\n\
         Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
         \r\n"
    )
}

async fn raw_upgrade(authority: &str, path: &str) -> Vec<u8> {
    let mut stream = TcpStream::connect(authority).await.expect("connects");
    stream
        .write_all(upgrade_request(authority, path).as_bytes())
        .await
        .expect("writes the request");
    let mut answer = Vec::new();
    let read = tokio::time::timeout(Duration::from_secs(5), stream.read_to_end(&mut answer))
        .await
        .expect("the server answers or closes");
    let _ = read;
    answer
}

// ── the destroy contract ───────────────────────────────────────────────────

#[tokio::test]
async fn an_upgrade_to_another_path_is_destroyed_with_no_http_response() {
    let harness = Harness::start().await;
    let authority = harness.authority();

    for path in ["/socket", "/api/realtime/extra", "/", "/api/health"] {
        let answer = raw_upgrade(&authority, path).await;
        assert!(
            answer.is_empty(),
            "an upgrade to {path} must be destroyed with no HTTP response; got {:?}",
            String::from_utf8_lossy(&answer),
        );
    }

    harness.stop().await;
}

#[tokio::test]
async fn a_plain_request_to_an_unknown_path_still_gets_a_response() {
    let harness = Harness::start().await;
    let mut stream = TcpStream::connect(harness.authority())
        .await
        .expect("connects");
    stream
        .write_all(
            format!(
                "GET /socket HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
                harness.authority()
            )
            .as_bytes(),
        )
        .await
        .expect("writes");
    let mut answer = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(5), stream.read_to_end(&mut answer)).await;
    let text = String::from_utf8_lossy(&answer);
    assert!(
        text.starts_with("HTTP/1.1 404"),
        "only an *upgrade* to a foreign path is destroyed; got {text}",
    );
    harness.stop().await;
}

// ── the two upgrade refusals ───────────────────────────────────────────────

#[tokio::test]
async fn a_disallowed_origin_is_refused_in_plain_text() {
    let harness = Harness::start().await;
    let mut stream = TcpStream::connect(harness.authority())
        .await
        .expect("connects");
    let request = format!(
        "GET /api/realtime HTTP/1.1\r\n\
         Host: {}\r\n\
         Origin: https://attacker.example\r\n\
         Connection: Upgrade\r\n\
         Upgrade: websocket\r\n\
         Sec-WebSocket-Version: 13\r\n\
         Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
         \r\n",
        harness.authority()
    );
    stream.write_all(request.as_bytes()).await.expect("writes");
    let mut answer = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(5), stream.read_to_end(&mut answer)).await;
    let text = String::from_utf8_lossy(&answer);
    assert!(text.starts_with("HTTP/1.1 403"), "got {text}");
    assert!(
        text.to_lowercase().contains("content-type: text/plain"),
        "the upgrade refusal is plain text, not the routes' JSON; got {text}",
    );
    assert!(text.ends_with("origin not allowed"), "got {text}");
    harness.stop().await;
}

// ── the vocabulary ─────────────────────────────────────────────────────────

async fn connect(harness: &Harness) -> Socket {
    let request = format!("{}/api/realtime?sessionId=main", harness.ws_origin)
        .into_client_request()
        .expect("a request");
    let (socket, _response) = tokio_tungstenite::connect_async(request)
        .await
        .expect("the upgrade is accepted");
    socket
}

async fn send(socket: &mut Socket, frame: Value) {
    socket
        .send(Message::Text(frame.to_string().into()))
        .await
        .expect("sends");
}

async fn wait_for(socket: &mut Socket, kind: &str) -> Value {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let mut seen = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        let message = tokio::time::timeout(remaining, socket.next())
            .await
            .unwrap_or_else(|_| panic!("{kind} never arrived; saw {seen:?}"));
        let Some(Ok(Message::Text(text))) = message else {
            continue;
        };
        let value: Value = serde_json::from_str(&text).expect("a JSON frame");
        let observed = value["type"].as_str().unwrap_or_default().to_owned();
        if observed == kind {
            return value;
        }
        seen.push(observed);
    }
}

fn connect_frame() -> Value {
    json!({
        "type": "connect",
        "timeZone": "Asia/Shanghai",
        "locale": "zh-CN",
        "voiceEnabled": true,
        "inputEnabled": true,
        "outputEnabled": true,
        "clientType": "web",
        "clientLabel": "test",
        "clientInstanceId": "client-1",
    })
}

#[tokio::test]
async fn a_connected_client_is_told_it_owns_the_microphone() {
    let harness = Harness::start().await;
    let mut socket = connect(&harness).await;
    send(&mut socket, connect_frame()).await;

    let ownership = wait_for(&mut socket, "voice.ownership").await;
    assert_eq!(ownership["state"], "active");
    assert_eq!(ownership["holder"]["type"], "web");
    assert_eq!(ownership["holder"]["label"], "test");

    harness.stop().await;
}

#[tokio::test]
async fn the_host_can_command_a_connected_client_to_release_the_microphone() {
    let harness = Harness::start().await;
    let mut socket = connect(&harness).await;
    send(&mut socket, connect_frame()).await;
    let _ = wait_for(&mut socket, "voice.ownership").await;

    harness
        .services
        .input_arbitration
        .suspend("host-app", "dictation", Some(60_000))
        .await
        .expect("an owner was supplied");

    // Playback stops with capture, so a host recording cannot pick up this
    // Gateway's own speech — and it goes out *first*.
    let clear = wait_for(&mut socket, "playback.clear").await;
    assert_eq!(clear["reason"], "input_suspended");
    let suspend = wait_for(&mut socket, "input.suspend").await;
    assert_eq!(suspend["owner"], "host-app");
    assert_eq!(suspend["reason"], "dictation");
    assert!(suspend["expiresAt"].as_i64().unwrap_or(0) > 0);

    harness.services.input_arbitration.resume("host-app").await;
    let _ = wait_for(&mut socket, "input.resume").await;

    harness.stop().await;
}

/// The next `count` frames, in the order they arrived.
async fn next_frames(socket: &mut Socket, count: usize) -> Vec<String> {
    let mut kinds = Vec::new();
    while kinds.len() < count {
        let message = tokio::time::timeout(Duration::from_secs(5), socket.next())
            .await
            .unwrap_or_else(|_| panic!("only {kinds:?} arrived"));
        let Some(Ok(Message::Text(text))) = message else {
            continue;
        };
        let value: Value = serde_json::from_str(&text).expect("a JSON frame");
        kinds.push(value["type"].as_str().unwrap_or_default().to_owned());
    }
    kinds
}

#[tokio::test]
async fn playback_is_cleared_before_the_suspension_is_announced() {
    let harness = Harness::start().await;
    let mut socket = connect(&harness).await;
    send(&mut socket, connect_frame()).await;
    let _ = wait_for(&mut socket, "voice.ownership").await;

    harness
        .services
        .input_arbitration
        .suspend("host-app", "dictation", Some(60_000))
        .await
        .expect("an owner was supplied");

    assert_eq!(
        next_frames(&mut socket, 2).await,
        ["playback.clear", "input.suspend"],
        "playback stops WITH capture, so a host recording cannot pick up this \
         Gateway's own speech — which means the clear goes first",
    );

    harness.stop().await;
}

#[tokio::test]
async fn a_repeated_suspend_by_the_same_owner_does_not_re_announce() {
    let harness = Harness::start().await;
    let mut socket = connect(&harness).await;
    send(&mut socket, connect_frame()).await;
    let _ = wait_for(&mut socket, "voice.ownership").await;

    for _ in 0..3 {
        harness
            .services
            .input_arbitration
            .suspend("host-app", "dictation", Some(60_000))
            .await
            .expect("an owner was supplied");
    }
    // Two frames for the first suspension, then nothing for the two renewals.
    assert_eq!(
        next_frames(&mut socket, 2).await,
        ["playback.clear", "input.suspend"],
    );

    harness.services.input_arbitration.resume("host-app").await;
    assert_eq!(
        next_frames(&mut socket, 1).await,
        ["input.resume"],
        "a renewal must not stack a second announcement in front of the resume",
    );
    assert_eq!(
        harness
            .services
            .input_arbitration
            .status()
            .await
            .holders
            .len(),
        0,
        "repeated cycles must not leak holders",
    );

    harness.stop().await;
}

#[tokio::test]
async fn a_client_connecting_mid_suspension_is_told_before_it_opens_a_microphone() {
    let harness = Harness::start().await;
    harness
        .services
        .input_arbitration
        .suspend("host-app", "", Some(60_000))
        .await
        .expect("an owner was supplied");

    let mut socket = connect(&harness).await;
    send(&mut socket, connect_frame()).await;
    let suspend = wait_for(&mut socket, "input.suspend").await;
    assert_eq!(suspend["owner"], "host-app");

    harness.stop().await;
}

#[tokio::test]
async fn an_expired_suspension_resumes_the_client_without_a_host_request() {
    let harness = Harness::start().await;
    let mut socket = connect(&harness).await;
    send(&mut socket, connect_frame()).await;
    let _ = wait_for(&mut socket, "voice.ownership").await;

    // A host that crashes after suspending must not silence the Gateway.
    harness
        .services
        .input_arbitration
        .suspend("crashed-host", "", Some(120))
        .await
        .expect("an owner was supplied");
    let _ = wait_for(&mut socket, "input.suspend").await;
    let _ = wait_for(&mut socket, "input.resume").await;
    assert!(!harness.services.input_arbitration.suspended().await);

    harness.stop().await;
}

#[tokio::test]
async fn a_typed_turn_starts_a_turn_and_reaches_the_model_layer() {
    let harness = Harness::start().await;
    let mut socket = connect(&harness).await;
    send(&mut socket, connect_frame()).await;
    let _ = wait_for(&mut socket, "voice.ownership").await;

    send(
        &mut socket,
        json!({ "type": "text.message", "text": "what time is it" }),
    )
    .await;

    let clear = wait_for(&mut socket, "playback.clear").await;
    assert_eq!(clear["reason"], "user_interruption");
    let started = wait_for(&mut socket, "turn.started").await;
    let turn_id = started["turnId"].as_str().unwrap_or_default().to_owned();
    assert!(turn_id.starts_with("text_"), "got {turn_id}");
    let state = wait_for(&mut socket, "voice.state").await;
    assert_eq!(state["state"], "processing");
    assert_eq!(state["turnId"], turn_id);

    // The engine seam saw it.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        if harness.engine.calls().iter().any(
            |call| matches!(call, EngineCall::SubmitText { text, .. } if text == "what time is it"),
        ) {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "the turn never reached the engine"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    harness.stop().await;
}

#[tokio::test]
async fn an_unparseable_frame_is_ignored_rather_than_closing_the_socket() {
    let harness = Harness::start().await;
    let mut socket = connect(&harness).await;
    socket
        .send(Message::Text("not json at all".into()))
        .await
        .expect("sends");
    socket
        .send(Message::Text("{\"type\":\"audio.delta\"}".into()))
        .await
        .expect("sends a server-direction event");
    send(&mut socket, connect_frame()).await;

    let ownership = wait_for(&mut socket, "voice.ownership").await;
    assert_eq!(
        ownership["state"], "active",
        "the socket survived two frames it could not act on",
    );
    harness.stop().await;
}

#[tokio::test]
async fn a_second_client_cannot_take_the_slot_without_asking() {
    let harness = Harness::start().await;
    let mut first = connect(&harness).await;
    send(&mut first, connect_frame()).await;
    let held = wait_for(&mut first, "voice.ownership").await;
    assert_eq!(held["state"], "active");

    let mut second = connect(&harness).await;
    let mut frame = connect_frame();
    frame["clientInstanceId"] = json!("client-2");
    send(&mut second, frame.clone()).await;
    let refused = wait_for(&mut second, "voice.ownership").await;
    assert_eq!(
        refused["state"], "busy",
        "a live holder refuses a claim that is not a takeover",
    );

    frame["takeover"] = json!(true);
    send(&mut second, frame).await;
    let taken = wait_for(&mut second, "voice.ownership").await;
    assert_eq!(taken["state"], "active");
    let lost = wait_for(&mut first, "voice.deactivated").await;
    assert_eq!(
        lost["holder"]["instanceId"], "client-2",
        "the loser is told who took it",
    );

    harness.stop().await;
}

#[tokio::test]
async fn the_work_plane_reaches_the_socket_and_control_work_does_not() {
    let harness = Harness::start().await;
    let mut socket = connect(&harness).await;
    send(&mut socket, connect_frame()).await;
    let _ = wait_for(&mut socket, "voice.ownership").await;

    let owner = harness.services.config.personal_owner_id.clone();
    let accepted = harness
        .services
        .work
        .create(via_work::NewWork::new("summarise the diff", &owner))
        .await
        .expect("the manager is running");

    // Nothing runs it, so it settles; either way the frame carries the Work.
    let failed = wait_for(&mut socket, "task.failed").await;
    assert_eq!(failed["task"]["id"], accepted.work.id);
    assert_eq!(failed["task"]["objective"], "summarise the diff");

    harness.stop().await;
}

/// A runner whose outcome carries an inline presentation block, so a real
/// `task.completed` drives the whole pipeline: `via-work` projects the block,
/// `via-app` pushes `timeline.inline`, and `ConversationSync.record` picks up
/// the result — nothing here hand-builds the frame or the stored message.
fn runner_with_inline() -> std::sync::Arc<dyn via_work::WorkRunner> {
    std::sync::Arc::new(
        |_objective: String, _context: via_work::WorkContext| async move {
            Ok(
                via_work::WorkOutcome::content("the diff is attached").with_metadata(json!({
                    "presentation": {
                        "speech": "done",
                        "inline": {"title": "Diff", "format": "code", "content": "+ line"},
                    },
                })),
            )
        },
    )
}

#[tokio::test]
async fn a_completed_works_inline_result_reaches_the_socket_as_timeline_inline() {
    let harness = Harness::start().await;
    let mut socket = connect(&harness).await;
    send(&mut socket, connect_frame()).await;
    let _ = wait_for(&mut socket, "voice.ownership").await;

    let owner = harness.services.config.personal_owner_id.clone();
    let accepted = harness
        .services
        .work
        .create(
            via_work::NewWork::new("summarise the diff", &owner)
                .session("main")
                .turn("voice-1")
                .runner(runner_with_inline()),
        )
        .await
        .expect("the manager is running");

    let completed = wait_for(&mut socket, "task.completed").await;
    assert_eq!(completed["task"]["id"], accepted.work.id);

    let inline = wait_for(&mut socket, "timeline.inline").await;
    assert_eq!(
        inline["item"],
        json!({
            "id": format!("inline_{}", accepted.work.id),
            "taskId": accepted.work.id,
            "turnId": "voice-1",
            "title": "Diff",
            "format": "code",
            "content": "+ line",
        }),
    );

    // The same terminal result reached the conversation store, under
    // upstream's own id and source — `recordTaskResult`'s contract.
    let recorded = harness
        .services
        .conversation_sync
        .list(&via_conversation::SessionRef::new(owner, "main"))
        .await
        .expect("the conversation task is running");
    let message = recorded
        .iter()
        .find(|message| message.id == format!("agent:{}", accepted.work.id))
        .unwrap_or_else(|| panic!("no agent: record among {recorded:?}"));
    assert_eq!(message.role, via_conversation::MessageRole::Assistant);
    assert_eq!(message.content, "the diff is attached");
    assert_eq!(message.source, via_conversation::MessageSource::AgentResult);
    assert_eq!(message.turn_id.as_deref(), Some("voice-1"));
    assert_eq!(message.task_id.as_deref(), Some(accepted.work.id.as_str()));

    harness.stop().await;
}

#[tokio::test]
async fn a_failed_work_with_no_inline_block_records_the_error_and_sends_no_timeline_frame() {
    let harness = Harness::start().await;
    let mut socket = connect(&harness).await;
    send(&mut socket, connect_frame()).await;
    let _ = wait_for(&mut socket, "voice.ownership").await;

    let owner = harness.services.config.personal_owner_id.clone();
    // No runner at all: the manager fails it with no resultMetadata, exactly
    // like `the_work_plane_reaches_the_socket_and_control_work_does_not`.
    let accepted = harness
        .services
        .work
        .create(
            via_work::NewWork::new("summarise the diff", &owner)
                .session("main")
                .turn("voice-2"),
        )
        .await
        .expect("the manager is running");

    let failed = wait_for(&mut socket, "task.failed").await;
    assert_eq!(failed["task"]["id"], accepted.work.id);

    let recorded = harness
        .services
        .conversation_sync
        .list(&via_conversation::SessionRef::new(owner, "main"))
        .await
        .expect("the conversation task is running");
    let message = recorded
        .iter()
        .find(|message| message.id == format!("agent:{}", accepted.work.id))
        .unwrap_or_else(|| panic!("no agent: record among {recorded:?}"));
    assert_eq!(message.source, via_conversation::MessageSource::AgentResult);
    assert!(
        !message.content.is_empty(),
        "the manager's own failure text"
    );

    // No inline block, so no `timeline.inline` — the next frame this owner's
    // socket produces is unrelated background chatter, never that type.
    let mut saw = Vec::new();
    let extra = tokio::time::timeout(Duration::from_millis(200), socket.next()).await;
    if let Ok(Some(Ok(Message::Text(text)))) = extra {
        let value: Value = serde_json::from_str(&text).expect("a JSON frame");
        saw.push(value["type"].as_str().unwrap_or_default().to_owned());
    }
    assert!(!saw.contains(&"timeline.inline".to_owned()), "got {saw:?}");

    harness.stop().await;
}

#[tokio::test]
async fn a_client_that_declares_no_provider_gets_the_gateways_default() {
    let harness = Harness::start().await;
    let mut socket = connect(&harness).await;
    send(&mut socket, connect_frame()).await;
    send(&mut socket, json!({ "type": "unmute" })).await;

    let ready = wait_for(&mut socket, "voice.ready").await;
    assert_eq!(ready["provider"], via_app::testing::TEST_PROVIDER);
    assert_eq!(ready["inputSampleRate"], 16_000);

    harness.stop().await;
}

#[tokio::test]
async fn an_unknown_provider_is_reported_rather_than_silently_replaced() {
    let harness = Harness::start().await;
    let mut socket = connect(&harness).await;
    let mut frame = connect_frame();
    frame["provider"] = json!("not-a-provider");
    send(&mut socket, frame).await;

    let error = wait_for(&mut socket, "error").await;
    assert!(
        error["message"]
            .as_str()
            .is_some_and(|message| { message.contains("not-a-provider") || !message.is_empty() }),
        "a typo must not look like a working session on the wrong provider",
    );

    harness.stop().await;
}

// ── the wake-word lifecycle ─────────────────────────────────────────────────

/// An opener that hands out one [`via_wake_word::ScriptedDetector`], silent
/// for its first chunk and then a match on `keyword`'s own label — so a test
/// can assert that a silent chunk produces no lifecycle frame before the one
/// that actually wakes the session.
#[derive(Debug)]
struct WakesOnSecondChunk {
    label: String,
}

#[async_trait::async_trait]
impl via_voice::WakeWordDetectorOpener for WakesOnSecondChunk {
    async fn open(
        &self,
        _keyword: &via_wake_word::Keyword,
    ) -> via_wake_word::Result<Box<dyn via_wake_word::WakeWordDetector>> {
        Ok(Box::new(via_wake_word::ScriptedDetector::after(
            1,
            via_wake_word::Detection::new(self.label.clone()),
        )))
    }
}

/// `Services` with wake word switched on for `locale`, driven through
/// `opener` — the seam `via-app`'s composition root wires, exercised here
/// with a scripted double instead of `--features sherpa`.
fn wake_word_services(
    keywords: via_voice::KeywordSet,
    opener: std::sync::Arc<dyn via_voice::WakeWordDetectorOpener>,
) -> via_app::Services {
    let env: via_core::EnvMap = [
        ("DASHSCOPE_API_KEY", "sk-test"),
        ("VIA_AUTH_SECRET", &"a".repeat(64)),
        ("PORT", "0"),
        ("VIA_WAKE_WORD_ENABLED", "true"),
        ("VIA_WAKE_WORD", "hey via"),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_owned(), value.to_owned()))
    .collect();
    let overrides = via_core::config::Overrides {
        home_directory: std::env::temp_dir().join("via-app-tests-wake-word"),
        working_directory: std::env::temp_dir().join("via-app-tests-wake-word"),
        ..via_core::config::Overrides::default()
    };
    let config = via_core::config::resolve(&env, None, &overrides).unwrap_or_default();
    let wake_word = via_voice::WakeWordLifecycle::from_config(&config, keywords, opener);
    let identity = via_core::IdentityManager::new(
        &"a".repeat(64),
        config.identity_mode,
        &config.personal_owner_id,
    )
    .expect("a 64-character secret clears the minimum");
    via_app::Services::builder(config)
        .identity(std::sync::Arc::new(identity))
        .realtime_registry(std::sync::Arc::new(via_app::testing::test_registry()))
        .realtime_provider(via_app::testing::TEST_PROVIDER)
        .logger(std::sync::Arc::new(via_log::Logger::with_sinks(
            via_log::LoggerOptions::detached("gateway"),
            Vec::new(),
        )))
        .wake_word(wake_word)
        .build()
        .expect("every default is constructible from the test configuration")
}

#[tokio::test]
async fn a_wake_word_only_connect_disables_when_no_phrase_is_configured() {
    // Wake word is switched on, but no keyword table backs it — the clean
    // "wake word disabled" `docs/architecture.md` §16 asks for, never a
    // panic, and never a silent no-op either: the client is told why.
    let services = wake_word_services(
        via_voice::KeywordSet::new(),
        std::sync::Arc::new(via_voice::NoWakeWordEngine),
    );
    let harness = Harness::with(services, via_app::InstanceIdentity::default()).await;
    let mut socket = connect(&harness).await;
    let mut frame = connect_frame();
    frame["wakeWordOnly"] = json!(true);
    send(&mut socket, frame).await;

    let preparing = wait_for(&mut socket, "voice.sleep").await;
    assert_eq!(preparing["state"], "preparing");
    assert_eq!(preparing["wakeWord"], "hey via");

    let disabled = wait_for(&mut socket, "voice.sleep").await;
    assert_eq!(disabled["state"], "disabled");
    assert!(
        disabled.get("wakeWord").is_none(),
        "disabled carries no wakeWord, got {disabled:?}",
    );
    assert!(
        disabled["message"]
            .as_str()
            .is_some_and(|message| message.contains("no wake phrase is configured")),
        "got {disabled:?}",
    );

    harness.stop().await;
}

#[tokio::test]
async fn a_wake_word_only_connect_sleeps_then_wakes_on_the_configured_phrase() {
    let keyword =
        via_wake_word::Keyword::zh_en("hey via", "HH EY1 V IY1 AH0").expect("a valid fixture");
    let keywords = via_voice::KeywordSet::new().with(via_i18n::Locale::En, keyword.clone());
    let services = wake_word_services(
        keywords,
        std::sync::Arc::new(WakesOnSecondChunk {
            label: keyword.label(),
        }),
    );
    let harness = Harness::with(services, via_app::InstanceIdentity::default()).await;
    let mut socket = connect(&harness).await;
    let mut frame = connect_frame();
    frame["wakeWordOnly"] = json!(true);
    send(&mut socket, frame).await;

    // `preparing`, then `enabled` — a detector was actually built.
    let preparing = wait_for(&mut socket, "voice.sleep").await;
    assert_eq!(preparing["state"], "preparing");
    let enabled = wait_for(&mut socket, "voice.sleep").await;
    assert_eq!(enabled["state"], "enabled");
    assert_eq!(enabled["wakeWord"], "hey via");
    assert!(enabled["timeoutMs"].is_number(), "got {enabled:?}");

    // `requestExplicitSleep` enters sleep once the detector exists — no
    // `ensureFrontend()` first: nothing has reached the model layer yet.
    let sleeping = wait_for(&mut socket, "voice.sleep").await;
    assert_eq!(sleeping["state"], "sleeping");
    assert!(
        harness.engine.calls().is_empty(),
        "a wake-word-only client must not open a realtime session before it wakes",
    );

    // Sleeping audio that does *not* match the phrase never reaches the
    // engine, and produces no lifecycle frame at all.
    let silent = via_realtime_mock::encode_audio(&[0u8; 640]);
    send(
        &mut socket,
        json!({ "type": "audio.append", "audio": silent }),
    )
    .await;

    // The scripted detector fires on the first non-empty chunk, so the next
    // `audio.append` is the one that wakes the session.
    let waking_audio = via_realtime_mock::encode_audio(&[0u8; 640]);
    send(
        &mut socket,
        json!({ "type": "audio.append", "audio": waking_audio }),
    )
    .await;

    let detected = wait_for(&mut socket, "voice.sleep").await;
    assert_eq!(detected["state"], "detected");
    assert_eq!(detected["wakeWord"], "hey via");

    let waking = wait_for(&mut socket, "voice.sleep").await;
    assert_eq!(waking["state"], "waking");

    // Waking really did open a realtime session — `voice.ready` only ever
    // goes out from inside `ensure_engine`, and this is the only path that
    // could have reached it.
    let ready = wait_for(&mut socket, "voice.ready").await;
    assert_eq!(ready["provider"], via_app::testing::TEST_PROVIDER);

    let awake = wait_for(&mut socket, "voice.sleep").await;
    assert_eq!(awake["state"], "awake");

    harness.stop().await;
}
