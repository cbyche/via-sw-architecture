//! Wire-payload contracts only a driven session can prove.
//!
//! `crates/via-conformance/tests/voice_wire.rs` asserts every `via-voice`-owned
//! payload reachable as a pure function call into `via-app`'s frame codec. A
//! handful of rows cannot be reached that way at all: the value only exists
//! once a real `via_realtime::ProviderEvent` has crossed
//! [`via::gateway::RealtimeEngine`]'s pump (`src/gateway/engine.rs`), which
//! needs a real session. This file is that: the same Gateway
//! `tests/milestone.rs` boots, with `via-realtime-mock` in front of the model,
//! driven exactly the way a client would drive it, reading the frame that
//! actually reached the socket rather than a hand-built stand-in for one.

mod support;

use std::sync::Arc;
use std::time::Duration;

use pretty_assertions::assert_eq;
use serde_json::json;
use support::gateway::{Client, Harness, is};
use via_downstream::testing::ScriptedHarness;
use via_realtime_mock::{CannedAudio, Emission, Script, Trigger, events, script::turns};

/// How long a turn is given to reach the socket.
const BUDGET: Duration = Duration::from_secs(20);

/// A harness that is never actually asked to do anything: nothing in this
/// file's scripts calls a tool, so no delegation ever reaches it.
fn idle_harness() -> Arc<dyn via_downstream::DownstreamAgent> {
    Arc::new(
        ScriptedHarness::builder("opencode")
            .build()
            .expect("a harness with an empty script still declares consistently"),
    )
}

/// `docs/reference/contracts.json`, `json-field` / *server audio.delta
/// payload*: `{ type, audio, sampleRate, responseId, turnId }`, in that key
/// order, with `sampleRate` falling back to the provider's own output rate
/// when the provider event carries none — which is exactly what the mock's
/// `events::audio_delta` emits (no `sampleRate` field at all).
#[tokio::test(flavor = "multi_thread")]
async fn server_audio_delta_payload_matches_the_catalogued_shape() {
    let clip = CannedAudio::tone(CannedAudio::DEFAULT_RATE, Duration::from_millis(120));
    let script = Script::conversation().turn(turns::speak("hello there", &clip));
    let mut gateway = Harness::start(script, idle_harness()).await;
    gateway.serve();

    let mut client = Client::connect(&gateway.origin, "voice-audio-delta").await;
    client.hello().await;
    client.say("say something").await;

    let frame = client
        .wait_for(BUDGET, |frame| frame["type"] == "audio.delta")
        .await
        .expect("the mock's canned clip reaches the socket as audio.delta");

    let object = frame.as_object().expect("an object");
    assert_eq!(
        object.keys().collect::<Vec<_>>(),
        vec!["type", "audio", "sampleRate", "responseId", "turnId"],
        "docs/reference/contracts.json, json-field / server audio.delta payload",
    );
    assert_eq!(
        frame["sampleRate"], 24_000,
        "no override on the provider event, so the fallback is the mock's own \
         output rate (`CannedAudio::DEFAULT_RATE`)",
    );
    assert!(
        frame["audio"]
            .as_str()
            .is_some_and(|audio| !audio.is_empty()),
        "the base64 PCM16 chunk, forwarded rather than decoded",
    );
    assert!(
        frame["responseId"]
            .as_str()
            .is_some_and(|id| !id.is_empty())
    );
    assert!(frame["turnId"].as_str().is_some_and(|id| !id.is_empty()));

    // And it is real audio, not an empty placeholder: `audio.done` follows,
    // carrying only the two keys the catalogue names for it.
    let done = client
        .wait_for(BUDGET, |frame| frame["type"] == "audio.done")
        .await
        .expect("the clip ends and the response closes its audio");
    let object = done.as_object().expect("an object");
    assert_eq!(
        object.keys().collect::<Vec<_>>(),
        vec!["type", "responseId", "turnId"],
    );
}

/// `realtime-gateway.mjs:601-625` — `emitAssistantTranscript`'s own shape:
/// `role`, `content`, `responseId`, then the `publicResponseContext` spread —
/// `turnId` comes from the spread, not a standalone field, which is the
/// ordering [`server_audio_delta_payload_matches_the_catalogued_shape`]
/// predates and this closes.
#[tokio::test(flavor = "multi_thread")]
async fn assistant_transcript_final_carries_the_public_response_context_spread() {
    let script = Script::conversation().turn(turns::say("ok"));
    let mut gateway = Harness::start(script, idle_harness()).await;
    gateway.serve();

    let mut client = Client::connect(&gateway.origin, "voice-assistant-transcript").await;
    client.hello().await;
    client.say("hello").await;

    let settled = client
        .wait_for(BUDGET, |frame| {
            frame["type"] == "transcript.final" && frame["role"] == "assistant"
        })
        .await
        .expect("the model's answer settles");
    assert_eq!(
        settled
            .as_object()
            .expect("an object")
            .keys()
            .collect::<Vec<_>>(),
        vec![
            "type",
            "role",
            "content",
            "responseId",
            "turnId",
            "taskId",
            "origin",
            "turnGeneration",
        ],
        "docs/reference/contracts.json, json-field / transcript events",
    );
    assert_eq!(settled["content"], json!("ok"));
    assert_eq!(settled["taskId"], serde_json::Value::Null);
    assert_eq!(settled["origin"], json!("model"));
    assert!(settled["turnId"].as_str().is_some_and(|id| !id.is_empty()));
    assert!(
        settled["responseId"]
            .as_str()
            .is_some_and(|id| !id.is_empty())
    );
}

/// `docs/reference/contracts.json`, `ws-event` / *voice.connection / voice.ready
/// / ... payloads*: `connecting` then `connected`, the provider echoed back,
/// and no `message` on either — `realtime-gateway.mjs:1444-1448,1539-1562`.
#[tokio::test(flavor = "multi_thread")]
async fn voice_connection_reports_connecting_then_connected() {
    let script = Script::conversation().turn(turns::say("hi"));
    let mut gateway = Harness::start(script, idle_harness()).await;
    gateway.serve();

    let mut client = Client::connect(&gateway.origin, "voice-connection-connect").await;
    // An explicit provider, rather than `client.hello()`'s implicit default —
    // the config's own default provider name is an environment fact this
    // harness happens to set, not this contract's.
    client
        .send(json!({ "type": "connect", "provider": Harness::PROVIDER }))
        .await;
    client.say("hello").await;

    client
        .wait_for(BUDGET, is("voice.ready"))
        .await
        .expect("a text turn opens the engine too");

    let states: Vec<String> = client
        .seen("voice.connection")
        .into_iter()
        .filter_map(|frame| frame["state"].as_str().map(str::to_owned))
        .collect();
    assert_eq!(
        states,
        vec!["connecting".to_owned(), "connected".to_owned()]
    );
    for frame in client.seen("voice.connection") {
        assert_eq!(frame["provider"], json!(Harness::PROVIDER));
        assert!(
            frame.get("message").is_none(),
            "a healthy connect carries no message: {frame}",
        );
    }
}

/// `realtime-gateway.mjs:1582-1588` — the connect-failure half of
/// `connectFrontendNow`, sent by the factory rather than the socket because
/// only it has the refusal text.
#[tokio::test(flavor = "multi_thread")]
async fn voice_connection_reports_unavailable_when_the_front_end_refuses_to_open() {
    let mut gateway = Harness::refusing_realtime(idle_harness()).await;
    gateway.serve();

    let mut client = Client::connect(&gateway.origin, "voice-connection-refuse").await;
    client
        .send(json!({ "type": "connect", "provider": Harness::PROVIDER }))
        .await;
    client.say("hello").await;

    let frame = client
        .wait_for(BUDGET, |frame| {
            frame["type"] == "voice.connection" && frame["state"] == "unavailable"
        })
        .await
        .expect("a refused open still announces unavailable");
    assert_eq!(frame["provider"], json!(Harness::PROVIDER));
    assert!(
        frame["message"]
            .as_str()
            .is_some_and(|message| !message.is_empty()),
        "{frame}",
    );
}

/// `tool-call-handler.mjs:65,664` — `enter_sleep` echoes the state back
/// through `requestClientState` once the client has declared it manages
/// `sleeping` itself.
#[tokio::test(flavor = "multi_thread")]
async fn client_state_echoes_a_declared_sleep_state_back() {
    let script = Script::conversation().turn(turns::call_tool(
        "call_1",
        via_voice::tools::catalog::ENTER_SLEEP,
        &json!({}),
    ));
    let mut gateway = Harness::start(script, idle_harness()).await;
    gateway.serve();

    let mut client = Client::connect(&gateway.origin, "client-state-sleep").await;
    client
        .send(json!({
            "type": "connect",
            "voiceEnabled": true,
            "clientStates": ["sleeping"],
        }))
        .await;
    client.say("go to sleep").await;

    let frame = client
        .wait_for(BUDGET, is("client.state"))
        .await
        .expect("requestClientState('sleeping')");
    assert_eq!(
        frame,
        json!({ "type": "client.state", "state": "sleeping" })
    );
}

/// `realtime-gateway.mjs:662-680,701-716` — `startPlayback` and
/// `cancelQueuedPlayback` on a barge-in: `voice.state: speaking`, then
/// `response.interrupted` carrying exactly the `publicResponseContext`
/// spread, because `context.playbackStarted` was true when the cancel
/// arrived.
#[tokio::test(flavor = "multi_thread")]
async fn response_interrupted_carries_the_public_response_context_on_a_barge_in() {
    let clip = CannedAudio::tone(CannedAudio::DEFAULT_RATE, Duration::from_millis(200));
    let script = Script::conversation().turn(turns::speak("hello there", &clip));
    let mut gateway = Harness::start(script, idle_harness()).await;
    gateway.serve();

    let mut client = Client::connect(&gateway.origin, "voice-response-interrupted").await;
    client
        .send(json!({ "type": "connect", "voiceEnabled": true }))
        .await;
    client.say("say something").await;

    let started = client
        .wait_for(BUDGET, is("response.started"))
        .await
        .expect("response.created reaches the socket");
    let response_id = started["responseId"]
        .as_str()
        .expect("a response id")
        .to_owned();
    let turn_id = started["turnId"].clone();
    let turn_generation = started["turnGeneration"].clone();

    client
        .send(json!({ "type": "playback.started", "responseId": response_id }))
        .await;
    let speaking = client
        .wait_for(BUDGET, |frame| {
            frame["type"] == "voice.state" && frame["state"] == "speaking"
        })
        .await
        .expect("startPlayback");
    assert_eq!(speaking["turnId"], turn_id);
    assert_eq!(speaking["origin"], json!("model"));

    client
        .send(json!({
            "type": "playback.cancelled",
            "responseId": response_id,
            "reason": "user_interruption",
        }))
        .await;

    let interrupted = client
        .wait_for(BUDGET, is("response.interrupted"))
        .await
        .expect("cancelQueuedPlayback — a started playback cut by a barge-in");
    assert_eq!(
        interrupted
            .as_object()
            .expect("an object")
            .keys()
            .collect::<Vec<_>>(),
        vec![
            "type",
            "responseId",
            "turnId",
            "taskId",
            "origin",
            "turnGeneration"
        ],
        "docs/reference/contracts.json, json-field / publicResponseContext",
    );
    assert_eq!(interrupted["responseId"], json!(response_id));
    assert_eq!(interrupted["turnId"], turn_id);
    assert_eq!(interrupted["taskId"], serde_json::Value::Null);
    assert_eq!(interrupted["origin"], json!("model"));
    assert_eq!(interrupted["turnGeneration"], turn_generation);

    let after = client
        .wait_for(BUDGET, |frame| frame["type"] == "voice.state")
        .await
        .expect("cancelQueuedPlayback always ends in a voice.state");
    assert_eq!(after["state"], json!("idle"));

    // The script's own `response.done` reached the socket before the client's
    // cancel did, so `audio.done` already went out — `realtime-gateway.mjs:1263`,
    // the third shape this contract row names.
    let done = client
        .seen("audio.done")
        .into_iter()
        .next()
        .expect("audio.done");
    assert_eq!(
        done.as_object()
            .expect("an object")
            .keys()
            .collect::<Vec<_>>(),
        vec!["type", "responseId", "turnId"],
    );
}

/// `realtime-gateway.mjs:970-1035` — the audio-input side of `voice.state`:
/// `listening` on `speech_started`, and `transcript.discard` plus `idle` when
/// the provider rules the segment `turn_invalid` on `speech_stopped`.
#[tokio::test(flavor = "multi_thread")]
async fn a_voice_turn_ruled_invalid_discards_its_transcript_instead_of_processing() {
    let script = Script::conversation().on(
        Trigger::AudioAppend,
        [
            Emission::now(events::speech_started()),
            Emission::after(
                Duration::from_millis(10),
                json!({ "type": "input_audio_buffer.speech_stopped", "reason": "turn_invalid" }),
            ),
        ],
    );
    let mut gateway = Harness::start(script, idle_harness()).await;
    gateway.serve();

    let mut client = Client::connect(&gateway.origin, "voice-turn-invalid").await;
    client
        .send(json!({ "type": "connect", "voiceEnabled": true }))
        .await;
    // No `unmute`: `audio.append` opens the engine itself, as upstream's
    // lazy `ensureFrontend()` does (`realtime-gateway.mjs:2028-2040`). This is
    // the shape a real client actually uses — connect with `voiceEnabled` and
    // start streaming — and it is the one that used to lose every chunk.
    client
        .send(json!({ "type": "audio.append", "audio": "AAAA" }))
        .await;

    let listening = client
        .wait_for(BUDGET, |frame| {
            frame["type"] == "voice.state" && frame["state"] == "listening"
        })
        .await
        .expect("input_audio_buffer.speech_started");
    assert!(
        listening.get("origin").is_none(),
        "listening carries no origin: {listening}",
    );
    let turn_id = listening["turnId"].clone();

    let discard = client
        .wait_for(BUDGET, is("transcript.discard"))
        .await
        .expect("speech_stopped(reason: turn_invalid)");
    assert_eq!(
        discard,
        json!({
            "type": "transcript.discard",
            "role": "user",
            "turnId": turn_id,
            "reason": "turn_invalid",
        }),
    );

    let idle = client
        .wait_for(BUDGET, |frame| {
            frame["type"] == "voice.state" && frame["state"] == "idle"
        })
        .await
        .expect("the discard is paired with an idle state");
    assert_eq!(idle["turnId"], turn_id);
    assert_eq!(idle["origin"], json!("model"));
}

/// `realtime-gateway.mjs:1017-1106` — a voice turn that settles normally:
/// `processing` on `speech_stopped`, the running transcript as
/// `transcript.delta` with `replace: true`, and the settled words as
/// `transcript.final`, all three carrying the one turn id `speech_started`
/// minted.
#[tokio::test(flavor = "multi_thread")]
async fn a_completed_voice_turn_produces_its_own_transcript_events() {
    let script = Script::conversation().on(
        Trigger::AudioAppend,
        [
            Emission::now(events::speech_started()),
            Emission::after(Duration::from_millis(10), events::speech_stopped()),
            Emission::after(
                Duration::from_millis(20),
                json!({
                    "type": "conversation.item.input_audio_transcription.delta",
                    "text": "你",
                }),
            ),
            Emission::after(
                Duration::from_millis(30),
                events::input_transcript_completed("你好"),
            ),
        ],
    );
    let mut gateway = Harness::start(script, idle_harness()).await;
    gateway.serve();

    let mut client = Client::connect(&gateway.origin, "voice-turn-completed").await;
    client
        .send(json!({ "type": "connect", "voiceEnabled": true }))
        .await;
    // No `unmute`: `audio.append` opens the engine itself, as upstream's
    // lazy `ensureFrontend()` does (`realtime-gateway.mjs:2028-2040`). This is
    // the shape a real client actually uses — connect with `voiceEnabled` and
    // start streaming — and it is the one that used to lose every chunk.
    client
        .send(json!({ "type": "audio.append", "audio": "AAAA" }))
        .await;

    let listening = client
        .wait_for(BUDGET, |frame| {
            frame["type"] == "voice.state" && frame["state"] == "listening"
        })
        .await
        .expect("speech_started");
    let turn_id = listening["turnId"].clone();

    let processing = client
        .wait_for(BUDGET, |frame| {
            frame["type"] == "voice.state" && frame["state"] == "processing"
        })
        .await
        .expect("speech_stopped with no reason");
    assert_eq!(processing["turnId"], turn_id);
    assert_eq!(processing["origin"], json!("model"));

    let delta = client
        .wait_for(BUDGET, is("transcript.delta"))
        .await
        .expect("streaming ASR");
    assert_eq!(
        delta,
        json!({
            "type": "transcript.delta",
            "role": "user",
            "content": "你",
            "turnId": turn_id,
            "replace": true,
        }),
    );

    let settled = client
        .wait_for(BUDGET, is("transcript.final"))
        .await
        .expect("the completed ASR");
    assert_eq!(
        settled,
        json!({
            "type": "transcript.final",
            "role": "user",
            "content": "你好",
            "turnId": turn_id,
        }),
    );
}
