//! Real PCM, through a real session, into a real `PlaybackCursor`.
//!
//! The Injection Gate's blocking predicate is
//! `userSpeaking || turnPending || audioResponses.nonEmpty`
//! (`docs/architecture.md` §3, §11), and the third term is
//! [`via_audio::PlaybackCursor::is_draining`], which counts frames. A mock that
//! streamed a placeholder string would let a gate test pass with no audio in it,
//! so these tests take the base64 the session actually received, decode it, and
//! check the frame arithmetic end to end.

mod common;

use std::time::Duration;

use common::{no_context, open, transcript};
use pretty_assertions::assert_eq;
use serde_json::json;
use via_audio::{ChannelCount, PlaybackCursor, SampleRate};
use via_realtime::ResponseOrigin;
use via_realtime_mock::{CannedAudio, Script, decode_audio, script::turns};

/// `samples.len()` minus `frames`, which is zero for the mono clips here.
fn bytes_per_frame_check(samples: &[i16], frames: u64) -> u64 {
    samples.len() as u64 - frames
}

/// Every `response.audio.delta` the session reported, decoded to frames.
fn played_frames(log: &via_realtime_mock::EventLog) -> (u64, Vec<i16>) {
    let mut frames = 0u64;
    let mut samples = Vec::new();
    for event in log.provider_events() {
        if event.kind() != "response.audio.delta" {
            continue;
        }
        let delta = event.event["delta"].as_str().expect("base64 audio");
        let bytes = decode_audio(delta).expect("valid base64");
        let decoded = via_audio::pcm16le_to_i16(&bytes);
        frames += decoded.len() as u64;
        samples.extend(decoded);
    }
    // Every clip this file builds is a whole number of `BLOCK_MILLIS` blocks
    // plus at most one short tail, so the total is never a fraction of a frame.
    assert_eq!(bytes_per_frame_check(&samples, frames), 0);
    (frames, samples)
}

#[tokio::test]
async fn scripted_speech_arrives_as_the_samples_the_clip_holds() {
    let clip = CannedAudio::tone(CannedAudio::DEFAULT_RATE, Duration::from_millis(200));
    let (session, log, _handle) =
        open(Script::conversation().turn(turns::speak("Two hundred milliseconds.", &clip))).await;

    session
        .speak("go", ResponseOrigin::Agent, no_context(), None)
        .await
        .expect("no transport failure")
        .expect("an outcome");
    assert!(log.wait_for_turns(1).await);

    let (frames, samples) = played_frames(&log);
    assert_eq!(frames, clip.frames() as u64);
    assert_eq!(samples, clip.samples());
    assert_eq!(log.spoken_text(), "Two hundred milliseconds.");
}

#[tokio::test]
async fn the_deltas_drain_a_playback_cursor_to_exactly_the_clips_duration() {
    let clip = CannedAudio::tone(CannedAudio::DEFAULT_RATE, Duration::from_millis(200));
    let (session, log, _handle) =
        open(Script::conversation().turn(turns::speak("draining", &clip))).await;
    session
        .speak("go", ResponseOrigin::Agent, no_context(), None)
        .await
        .expect("no transport failure")
        .expect("an outcome");
    assert!(log.wait_for_turns(1).await);

    // The host enqueues what it receives …
    let mut cursor = PlaybackCursor::new(clip.rate());
    for event in log.provider_events() {
        if event.kind() != "response.audio.delta" {
            continue;
        }
        let bytes = decode_audio(event.event["delta"].as_str().expect("base64")).expect("base64");
        cursor.enqueue((bytes.len() / 2) as u64);
    }
    // … and the Injection Gate is blocked until it has all been played.
    assert!(cursor.is_draining(), "the speaker still has audio to emit");
    assert_eq!(cursor.pending_ms(), 200);

    cursor.advance(cursor.pending_frames() / 2);
    assert!(cursor.is_draining());
    assert_eq!(cursor.audio_end_ms(), 100);

    cursor.advance(cursor.pending_frames());
    assert!(!cursor.is_draining(), "the gate may speak now");
    assert_eq!(cursor.audio_end_ms(), 200);
    assert_eq!(cursor.played_duration(), clip.duration());
}

#[tokio::test]
async fn a_barge_in_mid_clip_reports_what_the_user_actually_heard() {
    // The number `conversation.item.truncate` needs: how much of the model's
    // speech reached the user before the interruption.
    let clip = CannedAudio::tone(CannedAudio::DEFAULT_RATE, Duration::from_millis(500));
    let mut cursor = PlaybackCursor::new(clip.rate());
    for chunk in clip.chunks() {
        cursor.enqueue(chunk.frames as u64);
    }
    // Seven 20 ms blocks played, the rest thrown away.
    cursor.advance(clip.rate().millis_to_frames(140));
    assert_eq!(cursor.audio_end_ms(), 140);
    let discarded = cursor.clear();
    assert_eq!(clip.rate().frames_to_millis(discarded), 360);
    assert!(!cursor.is_draining());
}

#[tokio::test]
async fn the_deltas_are_block_sized_and_the_last_one_is_short() {
    // 50 ms at 24 kHz: two 20 ms blocks and a 10 ms remainder, so the frame
    // counts sum to the clip rather than being padded.
    let clip = CannedAudio::silence(CannedAudio::DEFAULT_RATE, Duration::from_millis(50));
    let (session, log, _handle) =
        open(Script::conversation().turn(turns::speak("short", &clip))).await;
    session
        .speak("go", ResponseOrigin::Agent, no_context(), None)
        .await
        .expect("no transport failure");
    assert!(log.wait_for_turns(1).await);

    let sizes: Vec<usize> = log
        .provider_events()
        .iter()
        .filter(|event| event.kind() == "response.audio.delta")
        .map(|event| {
            decode_audio(event.event["delta"].as_str().expect("base64"))
                .expect("base64")
                .len()
                / 2
        })
        .collect();
    assert_eq!(sizes, [480, 480, 240]);
}

#[tokio::test]
async fn every_delta_binds_to_the_response_it_belongs_to() {
    let clip = CannedAudio::tone(CannedAudio::DEFAULT_RATE, Duration::from_millis(40));
    let (session, log, _handle) =
        open(Script::conversation().turn(turns::speak("bound", &clip))).await;
    session
        .speak("go", ResponseOrigin::Agent, no_context(), None)
        .await
        .expect("no transport failure");
    assert!(log.wait_for_turns(1).await);

    let created = log
        .wait_for_kind("response.created")
        .await
        .expect("the response starts");
    let response_id = created.event["response"]["id"].clone();
    let mut deltas = 0;
    for event in log.provider_events() {
        if event.kind() != "response.audio.delta" {
            continue;
        }
        deltas += 1;
        assert_eq!(event.event["response_id"], response_id, "{event:?}");
        // Only the two *lifecycle* events carry the caller's origin and
        // context; intermediate activity keeps the default, because the Gateway
        // keys its own response context off the id rather than off the event.
        assert_eq!(event.origin, ResponseOrigin::Model, "{event:?}");
        assert!(event.context.is_empty(), "{event:?}");
    }
    assert_eq!(deltas, clip.chunks().len());
    assert_eq!(created.origin, ResponseOrigin::Agent);
}

#[tokio::test]
async fn a_wav_fixture_can_be_the_speech_a_script_plays() {
    let source = CannedAudio::tone(SampleRate::HZ_24000, Duration::from_millis(60));
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("speech.wav");
    via_audio::write_wav_file(&path, &source.to_wav_audio().expect("wav")).expect("write");

    let clip = CannedAudio::from_wav_file(&path).expect("read");
    assert_eq!(clip, source);

    let (session, log, _handle) =
        open(Script::conversation().turn(turns::speak("from a file", &clip))).await;
    session
        .speak("go", ResponseOrigin::Agent, no_context(), None)
        .await
        .expect("no transport failure");
    assert!(log.wait_for_turns(1).await);
    let (frames, samples) = played_frames(&log);
    assert_eq!(frames, 1_440);
    assert_eq!(samples, source.samples());
}

#[tokio::test]
async fn a_stereo_clip_keeps_its_frame_count_on_the_wire() {
    let clip = CannedAudio::from_samples(
        SampleRate::HZ_48000,
        ChannelCount::STEREO,
        (0..1_920).map(|n| (n % 2_000) as i16).collect(),
    )
    .expect("stereo");
    assert_eq!(clip.frames(), 960);
    assert_eq!(
        clip.chunks()
            .iter()
            .map(|chunk| chunk.frames)
            .sum::<usize>(),
        clip.frames()
    );
}

#[tokio::test]
async fn audio_travels_in_both_directions_over_the_same_transport() {
    let heard = CannedAudio::tone(SampleRate::HZ_16000, Duration::from_millis(60));
    let spoken = CannedAudio::tone(SampleRate::HZ_24000, Duration::from_millis(60));
    let (session, log, handle) =
        open(Script::conversation().turn(turns::speak("both ways", &spoken))).await;

    for chunk in heard.chunks() {
        session.append_audio(&chunk.base64).await.expect("appended");
    }
    session
        .speak("go", ResponseOrigin::Agent, no_context(), None)
        .await
        .expect("no transport failure");
    assert!(log.wait_for_turns(1).await);

    assert_eq!(
        transcript(&handle)
            .await
            .input_audio_pcm16()
            .expect("decode"),
        heard.samples()
    );
    assert_eq!(played_frames(&log).1, spoken.samples());
}

#[test]
fn the_two_realtime_rates_are_the_ones_the_provider_declares() {
    let provider = via_realtime_mock::MockProviderSpec::default()
        .build()
        .expect("valid");
    assert_eq!(provider.input_sample_rate(), SampleRate::HZ_16000.hz());
    assert_eq!(provider.output_sample_rate(), SampleRate::HZ_24000.hz());
    assert_eq!(
        CannedAudio::DEFAULT_RATE.hz(),
        provider.output_sample_rate(),
        "canned speech plays at the rate the provider says it will"
    );
}

#[test]
fn audio_that_is_not_base64_is_reported_rather_than_decoded_to_noise() {
    assert_eq!(
        decode_audio("not base64!!")
            .map(|_| ())
            .map_err(|e| e.code()),
        Err("VIA_MOCK_NOT_BASE64")
    );
    // The empty string is legal base64 for no bytes, and must not be an error.
    assert_eq!(decode_audio(""), Ok(Vec::new()));
    assert_eq!(json!(via_realtime_mock::encode_audio(&[])), json!(""));
}
