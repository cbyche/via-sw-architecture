//! `dictation`: transcripts, and nothing else.
//!
//! `docs/architecture.md` §2 — *"`dictation` mounts no model at all. It is the
//! one mode where the realtime provider may be a plain streaming ASR rather than
//! a speech-to-speech model, which is why `via-realtime`'s provider trait must
//! not assume a model turn exists."*
//!
//! So the assertions here come in pairs: the transcript path works, and every
//! response-creating call answers `skipped / no_model_turn` **without touching
//! the transport**. The second half is the one that would rot silently.

mod common;

use std::time::Duration;

use common::{no_context, open, transcript};
use pretty_assertions::assert_eq;
use serde_json::json;
use via_protocol::SessionMode;
use via_realtime::{
    FunctionOutputOptions, OutcomeKind, OutcomePhase, PermissionRequest, ResponseOrigin,
};
use via_realtime_mock::{
    CannedAudio, Emission, MockRealtime, Script, Trigger, events, script::turns,
};

/// A streaming ASR that answers every committed buffer with one transcript.
fn asr(transcript: &str) -> Script {
    Script::dictation().on_always(
        Trigger::AudioCommit,
        [
            Emission::now(events::input_committed()),
            Emission::now(events::input_transcript_delta(&transcript[..2])),
            Emission::now(events::input_transcript_completed(transcript)),
        ],
    )
}

#[tokio::test]
async fn a_dictation_script_opens_a_dictation_session_on_a_provider_with_no_model() {
    let mock = MockRealtime::new(Script::dictation());
    assert_eq!(mock.options().mode, SessionMode::Dictation);
    let provider = mock.provider().expect("valid");
    assert_eq!(provider.model(), None);
    assert_eq!(provider.voice(), None);
    assert_eq!(provider.model_profile(), None);

    let (session, _log, _handle) = open(Script::dictation()).await;
    assert_eq!(session.mode(), SessionMode::Dictation);
}

#[tokio::test]
async fn audio_in_becomes_a_transcript_out() {
    let clip = CannedAudio::tone(via_audio::SampleRate::HZ_16000, Duration::from_millis(40));
    let (session, log, handle) = open(asr("hello there")).await;

    for chunk in clip.chunks() {
        session.append_audio(&chunk.base64).await.expect("appended");
    }
    session.commit_audio().await.expect("committed");

    let final_transcript = log
        .wait_for_kind("conversation.item.input_audio_transcription.completed")
        .await
        .expect("a final transcript");
    assert_eq!(final_transcript.event["transcript"], json!("hello there"));

    let delta = log
        .wait_for_kind("conversation.item.input_audio_transcription.delta")
        .await
        .expect("a streaming transcript");
    assert_eq!(delta.event["delta"], json!("he"));

    // Every transcript event names the same input turn, which is what the
    // Gateway correlates a turn id against.
    let item = final_transcript.event["item_id"].clone();
    assert!(item.is_string(), "{final_transcript:?}");
    assert_eq!(delta.event["item_id"], item);

    // The audio really crossed the transport.
    assert_eq!(
        transcript(&handle)
            .await
            .input_audio_pcm16()
            .expect("decode"),
        clip.samples()
    );
}

#[tokio::test]
async fn each_committed_buffer_is_its_own_input_turn() {
    let (session, log, _handle) = open(asr("one")).await;
    for _ in 0..2 {
        session.append_audio("AAAA").await.expect("appended");
        session.commit_audio().await.expect("committed");
    }

    let completed: Vec<serde_json::Value> = log
        .wait_for(|entries| {
            entries
                .iter()
                .filter_map(via_realtime_mock::Recorded::provider)
                .filter(|event| {
                    event.kind() == "conversation.item.input_audio_transcription.completed"
                })
                .count()
                >= 2
        })
        .await
        .expect("two transcripts")
        .iter()
        .filter_map(via_realtime_mock::Recorded::provider)
        .filter(|event| event.kind() == "conversation.item.input_audio_transcription.completed")
        .map(|event| event.event["item_id"].clone())
        .collect();
    assert_ne!(completed[0], completed[1], "two turns, two item ids");
}

#[tokio::test]
async fn every_response_creating_call_is_skipped_without_touching_the_transport() {
    let (session, _log, handle) = open(asr("hello")).await;
    let before = transcript(&handle).await.outbound().len();

    let no_model_turn = |outcome: Option<via_realtime::ResponseOutcome>| {
        let outcome = outcome.expect("an outcome");
        assert_eq!(outcome.kind, OutcomeKind::Skipped, "{outcome:?}");
        assert_eq!(
            outcome.phase,
            Some(OutcomePhase::NoModelTurn),
            "{outcome:?}"
        );
    };

    no_model_turn(
        session
            .send_user_text("type this out", no_context(), None)
            .await
            .expect("no transport failure"),
    );
    no_model_turn(
        session
            .speak("anything", ResponseOrigin::Agent, no_context(), None)
            .await
            .expect("no transport failure"),
    );
    no_model_turn(
        session
            .ensure_response(no_context(), None, None)
            .await
            .expect("no transport failure"),
    );
    no_model_turn(
        session
            .send_function_output(
                "call_1",
                json!({}),
                no_context(),
                FunctionOutputOptions::with_response(),
            )
            .await
            .expect("no transport failure"),
    );
    no_model_turn(
        session
            .inject_permission(
                &PermissionRequest {
                    id: "auth-1".into(),
                    summary: "anything".into(),
                },
                no_context(),
                None,
            )
            .await
            .expect("no transport failure"),
    );
    let injected = session
        .inject_result("a result", ResponseOrigin::Announcement, no_context(), true)
        .await
        .expect("no transport failure")
        .expect("an outcome");
    no_model_turn(injected.outcome);
    assert!(!injected.context_injected);

    assert_eq!(
        transcript(&handle).await.outbound().len(),
        before,
        "a mode with no model turn writes nothing to create one"
    );
}

#[tokio::test]
async fn a_dictation_session_never_restores_recent_conversation() {
    // There is no conversation to give context to, so the restore is skipped
    // even when the caller supplied one.
    let mock =
        MockRealtime::new(Script::dictation()).with_agent_context(via_realtime::AgentContext {
            recent_context: Some("we were talking about the build".into()),
            ..Default::default()
        });
    let (_session, _log, handle) = common::open_with(mock).await;
    assert_eq!(
        transcript(&handle)
            .await
            .count_of("conversation.item.create"),
        0
    );
}

#[tokio::test]
async fn a_conversing_session_does_restore_it_exactly_once() {
    // The contrast that makes the test above mean something.
    let mock =
        MockRealtime::new(Script::conversation()).with_agent_context(via_realtime::AgentContext {
            recent_context: Some("we were talking about the build".into()),
            ..Default::default()
        });
    let (_session, _log, handle) = common::open_with(mock).await;
    handle
        .wait_for_frame("conversation.item.create")
        .await
        .expect("the restore is written");
    let transcript = transcript(&handle).await;
    assert_eq!(transcript.count_of("conversation.item.create"), 1);
    let text = transcript.items()[0]["content"][0]["text"]
        .as_str()
        .expect("text");
    assert!(text.contains("<restored_context>"), "{text}");
    assert!(text.contains("we were talking about the build"), "{text}");
}

#[tokio::test]
async fn a_dictation_script_may_still_report_speech_boundaries() {
    let (session, log, _handle) = open(Script::dictation().on_always(
        Trigger::AudioAppend,
        [Emission::now(events::speech_started())],
    ))
    .await;
    session.append_audio("AAAA").await.expect("appended");
    let started = log
        .wait_for_kind("input_audio_buffer.speech_started")
        .await
        .expect("speech started");
    assert!(started.event["item_id"].is_string(), "{started:?}");
}

#[tokio::test]
async fn a_transcription_failure_is_reported_rather_than_swallowed() {
    let (session, log, _handle) = open(Script::dictation().on_always(
        Trigger::AudioCommit,
        [Emission::now(events::input_transcript_failed())],
    ))
    .await;
    session.commit_audio().await.expect("committed");
    assert!(
        log.wait_for_kind("conversation.item.input_audio_transcription.failed")
            .await
            .is_some()
    );
}

#[tokio::test]
async fn a_response_creating_step_in_a_dictation_script_never_fires() {
    // Scriptable, but unreachable: nothing in `dictation` writes a
    // `response.create`, so the step is dead and the mock says so by never
    // consuming it.
    let (session, log, handle) =
        open(Script::dictation().turn(turns::say("should not happen"))).await;
    session
        .send_user_text("hello", no_context(), None)
        .await
        .expect("no transport failure");
    assert_eq!(transcript(&handle).await.count_of("response.create"), 0);
    // Nothing was ever asked for, so nothing can arrive late either.
    assert_eq!(log.completed_turns(), 0);
    assert_eq!(log.spoken_text(), "");
}
