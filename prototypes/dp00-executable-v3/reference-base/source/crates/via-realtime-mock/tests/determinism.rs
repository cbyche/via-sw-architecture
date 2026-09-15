//! Same script, same bytes — every run, on every host.
//!
//! This is the property the whole crate is for. A mock whose output wobbles
//! between runs is worse than no mock: it turns a real regression into "flaky
//! test" and gets muted.
//!
//! Two things could break it and both are asserted here: an id minted from a
//! random source, and an ordering that depends on the scheduler rather than on
//! the script.

mod common;

use std::time::Duration;

use common::{no_context, transcript};
use pretty_assertions::assert_eq;
use serde_json::Value;
use via_audio::SampleRate;
use via_realtime::{ProviderCapabilities, ResponseOrigin};
use via_realtime_mock::{
    CannedAudio, EVENT_ID_PREFIX, Emission, ITEM_ID_PREFIX, MockProviderSpec, MockRealtime,
    RESPONSE_ID_PREFIX, Script, Trigger, events, script::turns,
};

/// A provider that mints every id itself.
///
/// The baseline echoes the client's item id back, and *that* id is a v4 uuid the
/// session minted — the one thing in the loop the mock does not control. Turning
/// the echo off makes the whole inbound stream the mock's own.
fn self_minting() -> MockProviderSpec {
    MockProviderSpec::default().with_capabilities(ProviderCapabilities {
        conversation_item_id_echo: false,
        ..ProviderCapabilities::DEFAULT
    })
}

/// A script that exercises every id namespace and every delay path.
fn everything() -> Script {
    let clip = CannedAudio::tone(SampleRate::HZ_24000, Duration::from_millis(60));
    Script::conversation()
        .with_provider(self_minting())
        .turn(turns::call_tool(
            "call_1",
            "spawn_thinking",
            &serde_json::json!({ "objective": "x" }),
        ))
        .turn(turns::speak("here you go", &clip))
        .turn(vec![
            Emission::now(events::response_created()),
            Emission::after(Duration::from_millis(30), events::text_delta("late")),
            Emission::now(events::response_done(via_realtime_mock::COMPLETED)),
        ])
        .on_always(
            Trigger::AudioCommit,
            [Emission::now(events::input_transcript_completed("heard"))],
        )
}

/// Drive the script and answer with everything the mock sent.
async fn run() -> Vec<Value> {
    let (session, log, handle) = common::open_with(MockRealtime::new(everything())).await;

    session
        .send_user_text("go", no_context(), None)
        .await
        .expect("no transport failure");
    session
        .send_function_output(
            "call_1",
            serde_json::json!({ "status": "queued" }),
            no_context(),
            via_realtime::FunctionOutputOptions::with_response(),
        )
        .await
        .expect("no transport failure");
    session
        .speak("again", ResponseOrigin::Agent, no_context(), None)
        .await
        .expect("no transport failure");
    session.append_audio("AAAA").await.expect("appended");
    session.commit_audio().await.expect("committed");
    // Wait for the *last* thing the script emits, not for the frame that asked
    // for it: snapshotting between the two would compare a truncated stream and
    // read as flakiness in the very property this file asserts.
    log.wait_for_kind("conversation.item.input_audio_transcription.completed")
        .await
        .expect("the transcript arrives");
    transcript(&handle).await.inbound().to_vec()
}

#[tokio::test]
async fn a_script_that_uses_every_id_namespace_replays_byte_for_byte() {
    let first = run().await;
    let second = run().await;
    assert_eq!(first, second);
    assert!(
        first.len() > 10,
        "the script is worth comparing: {}",
        first.len()
    );
}

#[tokio::test]
async fn every_id_the_mock_mints_is_a_counter_rather_than_a_random_value() {
    let stream = run().await;

    let ids = |pointer: &str| -> Vec<String> {
        stream
            .iter()
            .filter_map(|event| event.pointer(pointer))
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect()
    };

    // `event_mock_1`, `event_mock_2`, … with no gaps and no repeats.
    let event_ids = ids("/event_id");
    assert!(!event_ids.is_empty());
    for (index, id) in event_ids.iter().enumerate() {
        assert_eq!(*id, format!("{EVENT_ID_PREFIX}{}", index + 1));
    }

    // Response ids count from one and never repeat out of order.
    let mut response_ids = ids("/response/id");
    response_ids.dedup();
    assert_eq!(
        response_ids,
        (1..=response_ids.len())
            .map(|n| format!("{RESPONSE_ID_PREFIX}{n}"))
            .collect::<Vec<_>>()
    );

    // Every item id is the mock's, because the echo is off.
    let item_ids = ids("/item/id");
    assert!(!item_ids.is_empty());
    assert!(
        item_ids.iter().all(|id| id.starts_with(ITEM_ID_PREFIX)),
        "{item_ids:?}"
    );
}

#[tokio::test]
async fn the_transcript_of_the_events_it_sent_is_the_stream_it_sent() {
    // The recorder is not a second source of truth: what `Transcript::inbound`
    // reports is exactly what crossed the transport, in order.
    let (session, log, handle) = common::open_with(MockRealtime::new(
        Script::conversation().turn(turns::say("x")),
    ))
    .await;
    session
        .send_user_text("go", no_context(), None)
        .await
        .expect("no transport failure");
    assert!(log.wait_for_turns(1).await);

    let sent: Vec<String> = transcript(&handle)
        .await
        .inbound_kinds()
        .iter()
        .map(|kind| (*kind).to_owned())
        .collect();
    let received = log.provider_kinds();
    assert_eq!(sent, received);
}

#[tokio::test(start_paused = true)]
async fn a_delayed_stream_arrives_in_the_order_it_was_written() {
    // Later emissions with shorter delays must not overtake earlier ones: the
    // queue is one FIFO with cumulative deadlines, not N independent timers.
    let (session, log, _handle) =
        common::open_with(MockRealtime::new(Script::conversation().turn(vec![
            Emission::now(events::response_created()),
            Emission::after(Duration::from_millis(300), events::text_delta("a")),
            Emission::after(Duration::from_millis(1), events::text_delta("b")),
            Emission::after(Duration::from_millis(1), events::text_delta("c")),
            Emission::after(
                Duration::from_millis(1),
                events::response_done(via_realtime_mock::COMPLETED),
            ),
        ])))
        .await;
    session
        .send_user_text("go", no_context(), None)
        .await
        .expect("no transport failure");
    assert!(log.wait_for_turns(1).await);
    assert_eq!(log.spoken_text(), "abc");
}

#[test]
fn canned_audio_is_the_same_waveform_on_every_run() {
    let first = CannedAudio::tone(SampleRate::HZ_24000, Duration::from_millis(120));
    let second = CannedAudio::tone(SampleRate::HZ_24000, Duration::from_millis(120));
    assert_eq!(first, second);
    assert_eq!(first.to_base64(), second.to_base64());
    // Integer arithmetic, so the peaks are nameable rather than approximate.
    assert_eq!(first.samples()[0], -via_realtime_mock::TONE_AMPLITUDE);
    assert_eq!(first.samples()[60], via_realtime_mock::TONE_AMPLITUDE);
}

#[test]
fn a_script_carries_no_hidden_state_between_two_mocks_built_from_it() {
    // `Script` is data, so two mocks built from one script do not share a
    // cursor. The step exhaustion lives in the server task, per session.
    let script = Script::conversation().turn(turns::say("once"));
    let first = MockRealtime::new(script.clone());
    let second = MockRealtime::new(script.clone());
    assert_eq!(first.script(), second.script());
    assert_eq!(first.script(), &script);
}
