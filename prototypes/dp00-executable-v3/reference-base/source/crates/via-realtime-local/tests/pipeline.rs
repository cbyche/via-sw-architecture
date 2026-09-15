//! The duplex illusion, end to end.
//!
//! Every test here drives the **real** [`RealtimeSession`] — the same two owning
//! tasks, the same correlation, the same watchdogs, the same busy-retry ladder —
//! over the in-process pipeline, with no weights, no GPU, no network and no
//! audio hardware. The stages are [`via_realtime_local::scripted`]'s, which is
//! the only seam CI can ever run.
//!
//! The thesis under test is negative and is stated once, in
//! [`the_event_vocabulary_is_one_the_shipped_gateway_already_reads`]: **nothing
//! above this crate can tell the difference** between the local pipeline and a
//! cloud provider.
//!
//! [`RealtimeSession`]: via_realtime::RealtimeSession

mod common;

use std::sync::Arc;

use common::{EventLog, collect_events, feed_blocks};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use via_protocol::SessionMode;
use via_realtime::{
    AgentContext, RESPONSE_ACTIVITY_TYPES, RealtimeSession, ResponseContext, ResponseOrigin,
    SessionOptions,
};
use via_realtime_local::{
    EMITTED_EVENT_TYPES, LocalPipeline, LocalPipelineProvider, LocalSettings, ScriptedResponder,
    ScriptedSpeaker, ScriptedTranscriber, ScriptedTurn, ScriptedVoiceActivity, Stages, ToolCall,
};

// ── building a session ──────────────────────────────────────────────────────

struct Doubles {
    responder: Arc<ScriptedResponder>,
    speaker: Arc<ScriptedSpeaker>,
}

fn agent_context() -> AgentContext {
    AgentContext {
        instructions: "You are VIA.".to_owned(),
        tools: vec![json!({
            "type": "function",
            "function": { "name": "request_delegation", "description": "Delegate.", "parameters": {} },
        })],
        ..AgentContext::default()
    }
}

async fn open(
    voice_activity: ScriptedVoiceActivity,
    transcriber: ScriptedTranscriber,
    responder: ScriptedResponder,
    speaker: ScriptedSpeaker,
    mode: SessionMode,
) -> (RealtimeSession, EventLog, Doubles) {
    let responder = Arc::new(responder);
    let speaker = Arc::new(speaker);
    let (session, events) = LocalPipeline::new(
        LocalPipelineProvider::with_supplied_stages(LocalSettings::default()),
        Stages {
            voice_activity: Box::new(voice_activity),
            transcriber: Box::new(transcriber),
            responder: Arc::clone(&responder) as Arc<dyn via_realtime_local::Responder>,
            speaker: Arc::clone(&speaker) as Arc<dyn via_realtime_local::Speaker>,
        },
    )
    .with_options(SessionOptions {
        mode,
        agent_context: agent_context(),
        ..SessionOptions::default()
    })
    .open()
    .await
    .expect("the pipeline opens");
    let log = collect_events(events);
    (session, log, Doubles { responder, speaker })
}

/// One utterance, answered in one sentence.
async fn one_turn() -> (RealtimeSession, EventLog, Doubles) {
    open(
        ScriptedVoiceActivity::utterance(1),
        ScriptedTranscriber::hearing("turn on the lights"),
        ScriptedResponder::saying("On it. "),
        ScriptedSpeaker::new(),
        SessionMode::Agent,
    )
    .await
}

/// The index of the first event of `kind`, or `None`.
fn index_of(types: &[String], kind: &str) -> Option<usize> {
    types.iter().position(|found| found == kind)
}

/// Assert `types` contains all of `expected`, in this relative order.
fn assert_ordered(types: &[String], expected: &[&str]) {
    let mut cursor = 0usize;
    for kind in expected {
        let found = types[cursor..]
            .iter()
            .position(|seen| seen == kind)
            .unwrap_or_else(|| panic!("{kind} is missing or out of order in {types:?}"));
        cursor += found + 1;
    }
}

// ── the handshake ───────────────────────────────────────────────────────────

#[tokio::test]
async fn the_session_opens_on_the_pipelines_own_handshake() {
    let (session, log, _) = one_turn().await;
    assert!(
        log.wait_for_type("session.updated").await,
        "{:?}",
        log.types()
    );
    assert_ordered(&log.types(), &["session.created", "session.updated"]);

    // The session is bound to *this* provider, not to anything underneath it.
    assert_eq!(session.provider().key(), "local-omni");
    assert_eq!(session.input_sample_rate(), 16_000);
    assert_eq!(session.mode(), SessionMode::Agent);

    // And the payload it acknowledged is the one the provider built: turn
    // detection present, which is the field upstream's all-false profile leaves
    // null and the whole of "connects and hears nothing".
    let updated = log.first("session.updated").expect("acknowledged");
    assert_eq!(
        updated["session"]["audio"]["input"]["turn_detection"]["type"],
        json!("server_vad")
    );
    assert_eq!(updated["session"]["instructions"], json!("You are VIA."));
}

// ── one spoken turn ─────────────────────────────────────────────────────────

#[tokio::test]
async fn a_spoken_turn_produces_a_transcript_a_response_and_audio() {
    let (session, log, doubles) = one_turn().await;
    feed_blocks(&session, 2).await.expect("audio is accepted");

    assert!(
        log.wait_for(|log| log.last_response_status().is_some())
            .await,
        "the turn never completed: {:?}",
        log.types()
    );

    assert_ordered(
        &log.types(),
        &[
            "input_audio_buffer.speech_started",
            "conversation.item.input_audio_transcription.delta",
            "input_audio_buffer.speech_stopped",
            "input_audio_buffer.committed",
            "conversation.item.input_audio_transcription.completed",
            "response.created",
            "response.output_item.added",
            "response.audio_transcript.delta",
            "response.audio.delta",
            "response.audio.done",
            "response.audio_transcript.done",
            "response.done",
        ],
    );

    assert_eq!(log.input_transcripts(), ["turn on the lights"]);
    assert_eq!(log.spoken_text(), "On it. ");
    assert_eq!(log.last_response_status().as_deref(), Some("completed"));
    assert!(
        !log.spoken_samples().is_empty(),
        "no audio reached the wire"
    );
    assert_eq!(
        log.spoken_samples(),
        ScriptedSpeaker::chunks_for("On it.").concat(),
        "the audio is the synthesizer's, byte for byte"
    );
    assert_eq!(doubles.responder.turn_count(), 1);
    assert_eq!(doubles.speaker.spoken_text(), ["On it."]);
    assert!(log.errors().is_empty(), "{:?}", log.errors());
}

#[tokio::test]
async fn the_reasoning_turn_is_given_the_session_instructions_the_tools_and_the_transcript() {
    let (session, log, doubles) = one_turn().await;
    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(
        log.wait_for(|log| log.last_response_status().is_some())
            .await
    );

    let asked = doubles.responder.asked();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].instructions, "You are VIA.");
    assert_eq!(asked[0].transcript, "turn on the lights");
    assert_eq!(asked[0].response_instructions, None);
    assert_eq!(
        asked[0].tools.len(),
        1,
        "the tool catalog reached the stage"
    );
    // The user's own words are the turn's **input**, not also a history entry.
    // Every stage renders both, so a transcript that were also the last history
    // message would show the model its own user turn twice.
    assert!(asked[0].history.is_empty(), "{:?}", asked[0].history);
}

#[tokio::test]
async fn the_second_turn_sees_the_first_exchange_as_history_exactly_once() {
    let (session, log, doubles) = open(
        ScriptedVoiceActivity::utterances(2, 1),
        ScriptedTranscriber::hearing_each(["first", "second"]),
        ScriptedResponder::saying("Sure. "),
        ScriptedSpeaker::new(),
        SessionMode::Agent,
    )
    .await;
    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(log.wait_for_type("response.done").await);
    feed_blocks(&session, 3).await.expect("audio is accepted");
    assert!(log.wait_for(|log| log.count_of("response.done") >= 2).await);

    let asked = doubles.responder.asked();
    assert_eq!(asked.len(), 2);
    assert_eq!(asked[1].transcript, "second");
    let history: Vec<(via_realtime_local::TurnRole, String)> = asked[1]
        .history
        .iter()
        .map(|message| (message.role, message.text.clone()))
        .collect();
    assert_eq!(
        history,
        vec![
            (via_realtime_local::TurnRole::User, "first".to_owned()),
            (via_realtime_local::TurnRole::Assistant, "Sure.".to_owned()),
        ]
    );
}

#[tokio::test]
async fn an_item_nobody_answered_is_still_history_by_the_next_turn() {
    // `append_user_context` creates an item and asks for nothing. It is
    // conversation the model was told about, so it must not be dropped when the
    // *next* input replaces it as the turn's transcript.
    let (session, log, doubles) = one_turn().await;
    assert!(
        session
            .append_user_context("here is a file")
            .await
            .expect("the call succeeds")
    );
    assert!(log.wait_for_type("conversation.item.created").await);

    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(
        log.wait_for(|log| log.last_response_status().is_some())
            .await
    );

    let asked = doubles.responder.asked();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].transcript, "turn on the lights");
    assert_eq!(asked[0].history.len(), 1);
    assert_eq!(asked[0].history[0].text, "here is a file");
}

#[tokio::test]
async fn the_running_transcript_is_text_plus_stash_the_way_via_voice_renders_it() {
    let (session, log, _) = open(
        // Four voiced blocks, then the end: `utterance(n)` scripts n voiced
        // blocks and one `Ended`, so it takes n + 1 blocks to complete.
        ScriptedVoiceActivity::utterance(4),
        ScriptedTranscriber::hearing("turn on the lights"),
        ScriptedResponder::saying("Done. "),
        ScriptedSpeaker::new(),
        SessionMode::Agent,
    )
    .await;
    feed_blocks(&session, 5).await.expect("audio is accepted");
    assert!(
        log.wait_for(|log| log.last_response_status().is_some())
            .await
    );

    // `input-transcript.mjs:6-9` — the uncommitted tail is concatenated, not
    // dropped, so the running transcript never lags a word behind.
    assert_eq!(
        log.running_input_transcripts(),
        ["turn", "turn on", "turn on the", "turn on the lights"]
    );
}

#[tokio::test]
async fn synthesis_is_per_sentence_so_speech_starts_before_generation_ends() {
    // The whole reason `crate::sentence` exists: waiting for the reasoning turn
    // to finish before speaking adds the entire generation to the
    // time-to-first-audio.
    let (session, log, doubles) = open(
        ScriptedVoiceActivity::utterance(1),
        ScriptedTranscriber::hearing("status"),
        ScriptedResponder::turns([ScriptedTurn::saying(
            "The build is green. Two tests were added. ",
        )]),
        ScriptedSpeaker::new(),
        SessionMode::Agent,
    )
    .await;
    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(
        log.wait_for(|log| log.last_response_status().is_some())
            .await
    );

    assert_eq!(
        doubles.speaker.spoken_text(),
        ["The build is green.", "Two tests were added."],
        "each sentence is synthesized on its own"
    );
    assert_eq!(
        log.spoken_text(),
        "The build is green. Two tests were added. "
    );
    assert_eq!(log.last_response_status().as_deref(), Some("completed"));
}

// ── barge-in ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn barge_in_cuts_speech_mid_utterance_and_discards_the_reasoning_turn() {
    let (session, log, doubles) = open(
        // Two utterances: the second interrupts the answer to the first.
        ScriptedVoiceActivity::utterances(2, 1),
        ScriptedTranscriber::hearing_each(["first", "second"]),
        // A turn that says one sentence and then never finishes, over a speaker
        // whose utterance never ends: both are still live when the user speaks.
        ScriptedResponder::turns([ScriptedTurn::saying_then_stalling("Working on it. ")]),
        ScriptedSpeaker::stalling(),
        SessionMode::Agent,
    )
    .await;
    let cancelled_turns = doubles.responder.cancellations();
    let cancelled_speech = doubles.speaker.cancellations();

    // The first utterance: `Started`, then `Ended`.
    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(
        log.wait_for_type("response.audio.delta").await,
        "the answer never started speaking: {:?}",
        log.types()
    );
    assert_eq!(cancelled_turns.get(), 0);
    assert_eq!(cancelled_speech.get(), 0);

    // The silent block, then the barge-in edge.
    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(
        log.wait_for_type("response.done").await,
        "the response was never closed out: {:?}",
        log.types()
    );

    assert_eq!(
        log.last_response_status().as_deref(),
        Some("cancelled"),
        "a barge-in must settle the caller's outcome, not leave it to the \
         inactivity watchdog two minutes later"
    );
    assert_eq!(
        cancelled_turns.get(),
        1,
        "the in-flight turn was not dropped"
    );
    assert_eq!(cancelled_speech.get(), 1, "the utterance was not cut");

    // The order is a cloud provider's: the speech-start edge — the Gateway's
    // `userSpeaking`, and the Injection Gate's first blocking input — is
    // published *before* the cancellation it caused.
    let types = log.types();
    let started = index_of(&types, "input_audio_buffer.speech_started").expect("an edge");
    let second_started = types
        .iter()
        .enumerate()
        .filter(|(_, kind)| *kind == "input_audio_buffer.speech_started")
        .nth(1)
        .map(|(index, _)| index)
        .expect("two edges");
    let done = index_of(&types, "response.done").expect("a close");
    assert!(started < second_started);
    assert!(
        second_started < done,
        "the barge-in edge must precede the cancellation: {types:?}"
    );

    // And the audio stream is closed out before the response is, so a client
    // that is still draining knows there is no more coming.
    assert_ordered(&types, &["response.audio.done", "response.done"]);
}

#[tokio::test]
async fn a_second_utterance_is_answered_after_the_barge_in() {
    let (session, log, doubles) = open(
        ScriptedVoiceActivity::utterances(2, 1),
        ScriptedTranscriber::hearing_each(["first", "second"]),
        ScriptedResponder::turns([
            ScriptedTurn::saying_then_stalling("Working. "),
            ScriptedTurn::saying("Right away. "),
        ]),
        ScriptedSpeaker::new(),
        SessionMode::Agent,
    )
    .await;

    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(log.wait_for_type("response.created").await);
    feed_blocks(&session, 3).await.expect("audio is accepted");

    assert!(
        log.wait_for(|log| log.count_of("response.done") >= 2).await,
        "the second turn never completed: {:?}",
        log.types()
    );
    assert_eq!(log.input_transcripts(), ["first", "second"]);
    assert_eq!(doubles.responder.turn_count(), 2);
    assert_eq!(log.last_response_status().as_deref(), Some("completed"));
    assert!(
        log.spoken_text().contains("Right away."),
        "{}",
        log.spoken_text()
    );
}

// ── dictation mounts no model ───────────────────────────────────────────────

#[tokio::test]
async fn dictation_transcribes_and_never_creates_a_response() {
    // `docs/architecture.md` §2: the one mode that mounts no model at all, and
    // on the local pipeline the cheapest path by a wide margin.
    let (session, log, doubles) = open(
        ScriptedVoiceActivity::utterance(1),
        ScriptedTranscriber::hearing("dictate this please"),
        ScriptedResponder::saying("never asked"),
        ScriptedSpeaker::new(),
        SessionMode::Dictation,
    )
    .await;
    feed_blocks(&session, 2).await.expect("audio is accepted");

    assert!(
        log.wait_for_type("conversation.item.input_audio_transcription.completed")
            .await
    );
    assert_eq!(log.input_transcripts(), ["dictate this please"]);
    assert_eq!(log.count_of("response.created"), 0);
    assert_eq!(
        doubles.responder.turn_count(),
        0,
        "no model turn was mounted"
    );
    assert!(
        doubles.speaker.spoken().is_empty(),
        "nothing was synthesized"
    );

    // And a response-creating call is answered without touching the transport.
    let outcome = session
        .speak(
            "say this",
            ResponseOrigin::Agent,
            ResponseContext::new(),
            None,
        )
        .await
        .expect("the call succeeds")
        .expect("an outcome");
    assert!(outcome.is_skipped());
    assert_eq!(doubles.responder.turn_count(), 0);
}

// ── the Gateway's own calls ─────────────────────────────────────────────────

#[tokio::test]
async fn an_agent_utterance_carries_its_own_instructions_and_its_own_origin() {
    // `speak` is how the Gateway makes the model read something aloud without it
    // entering the conversation (`conversation: 'none'`), so the content travels
    // in `response.create.instructions`. A provider that dropped them would
    // answer the previous user turn again.
    let (session, log, doubles) = one_turn().await;
    let outcome = session
        .speak(
            "the build finished",
            ResponseOrigin::Agent,
            ResponseContext::new().with("taskId", "job_1"),
            None,
        )
        .await
        .expect("the call succeeds")
        .expect("an outcome");
    assert!(outcome.is_completed(), "{outcome:?}");

    let asked = doubles.responder.asked();
    assert_eq!(asked.len(), 1);
    assert!(
        asked[0]
            .response_instructions
            .as_deref()
            .is_some_and(|instructions| instructions.contains("the build finished")),
        "{:?}",
        asked[0].response_instructions
    );
    assert_eq!(asked[0].transcript, "", "no new user input");

    // The correlation the pipeline echoes is what lets the session attribute the
    // response to the caller rather than to an automatic turn-detection turn.
    // `via-realtime` stamps the origin and the context onto `response.created`
    // and `response.done` — the two events the correlation is carried on.
    assert!(log.wait_for_type("response.done").await);
    for kind in ["response.created", "response.done"] {
        let record = log
            .events()
            .into_iter()
            .find(|record| record.kind() == kind)
            .unwrap_or_else(|| panic!("no {kind} in {:?}", log.types()));
        assert_eq!(record.origin, ResponseOrigin::Agent, "{kind}");
        assert_eq!(record.context.task_id(), Some("job_1"), "{kind}");
        assert!(!record.retried, "{kind}");
    }
}

#[tokio::test]
async fn a_result_injection_reaches_the_conversation_and_is_spoken() {
    let (session, log, doubles) = one_turn().await;
    let outcome = session
        .inject_result(
            "the build finished",
            ResponseOrigin::Announcement,
            ResponseContext::new(),
            true,
        )
        .await
        .expect("the call succeeds")
        .expect("an outcome");
    assert!(outcome.context_injected, "the item never landed");
    assert!(
        outcome
            .outcome
            .is_some_and(|outcome| outcome.is_completed()),
        "the spoken half did not complete"
    );

    assert!(log.wait_for_type("conversation.item.created").await);
    // The item the session created is echoed with the id it minted, which is
    // what `conversation_item_id_echo: true` declares.
    let created = log.first("conversation.item.created").expect("an echo");
    assert!(
        created["item"]["id"]
            .as_str()
            .is_some_and(|id| !id.is_empty()),
        "{created}"
    );
    assert_eq!(created["item"]["role"], json!("user"));

    // And the injected text is the *transcript* of the next turn, because that
    // is what a conversation item is.
    let asked = doubles.responder.asked();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].transcript, "the build finished");
}

#[tokio::test]
async fn a_user_text_turn_round_trips_through_the_item_and_the_response() {
    let (session, log, doubles) = one_turn().await;
    let outcome = session
        .send_user_text("what is the status", ResponseContext::new(), None)
        .await
        .expect("the call succeeds")
        .expect("an outcome");
    assert!(outcome.is_completed(), "{outcome:?}");

    assert!(log.wait_for_type("response.done").await);
    assert_eq!(
        doubles.responder.asked()[0].transcript,
        "what is the status"
    );
    assert_eq!(log.spoken_text(), "On it. ");
}

// ── tool calls ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_tool_call_is_published_as_an_output_item_and_its_arguments() {
    let (session, log, _) = open(
        ScriptedVoiceActivity::utterance(1),
        ScriptedTranscriber::hearing("build the thing"),
        ScriptedResponder::turns([ScriptedTurn::calling(ToolCall::new(
            "call_1",
            "request_delegation",
            r#"{"objective":"build the thing"}"#,
        ))]),
        ScriptedSpeaker::new(),
        SessionMode::Agent,
    )
    .await;
    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(
        log.wait_for_type("response.function_call_arguments.done")
            .await
    );
    assert!(log.wait_for_type("response.done").await);

    let arguments = log
        .first("response.function_call_arguments.done")
        .expect("published");
    assert_eq!(arguments["call_id"], json!("call_1"));
    assert_eq!(arguments["name"], json!("request_delegation"));
    assert_eq!(
        arguments["arguments"],
        json!(r#"{"objective":"build the thing"}"#)
    );

    // The item is on the response, so the Gateway can read the call off
    // `response.done` as well as off the streaming event.
    let done = log.last("response.done").expect("a close");
    assert_eq!(
        done["response"]["output"][0]["type"],
        json!("function_call")
    );
    assert_eq!(done["response"]["output"][0]["call_id"], json!("call_1"));
    assert_eq!(log.last_response_status().as_deref(), Some("completed"));

    // A tool call is not speech: nothing was synthesized for it.
    assert_eq!(log.count_of("response.audio.delta"), 0);
}

#[tokio::test]
async fn a_tool_result_is_accepted_and_answered() {
    let (session, log, doubles) = open(
        ScriptedVoiceActivity::silent(),
        ScriptedTranscriber::deaf(),
        ScriptedResponder::saying("All done. "),
        ScriptedSpeaker::new(),
        SessionMode::Agent,
    )
    .await;
    let outcome = session
        .send_function_output(
            "call_1",
            json!({ "status": "ok" }),
            ResponseContext::new(),
            via_realtime::FunctionOutputOptions::with_response(),
        )
        .await
        .expect("the call succeeds");
    assert!(
        outcome.is_some_and(|outcome| outcome.is_completed()),
        "the tool result was not answered"
    );
    assert!(log.wait_for_type("response.done").await);
    assert_eq!(doubles.responder.turn_count(), 1);
}

// ── cancellation ────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_cancel_with_nothing_active_is_suppressed_rather_than_shown() {
    // ARGO bring-up §2: with server VAD the user speaks *before* the model does,
    // so a host wiring barge-in to speech-start cancels an idle session — and an
    // error on screen for every first utterance is what that used to look like.
    //
    // `RealtimeSession::cancel` is careful enough not to write `response.cancel`
    // when it knows nothing started, so the frame is written directly — the
    // pipeline still has to answer a client (or a dialect adapter) that does.
    let (session, log, _) = one_turn().await;
    session
        .send_frame(json!({ "type": "response.cancel" }))
        .await
        .expect("accepted");
    assert!(log.wait_for_type("error").await, "{:?}", log.types());
    let error = log.first("error").expect("a refusal");
    assert_eq!(error["error"]["code"], json!("response_cancel_not_active"));
    assert!(
        log.errors().is_empty(),
        "a benign cancel race must not surface to the Gateway: {:?}",
        log.errors()
    );
    assert_eq!(
        session
            .provider()
            .classify_error(error["error"]["message"].as_str().unwrap_or_default()),
        via_realtime::ErrorClass::NoActiveResponse,
    );
}

// ── stage failures ──────────────────────────────────────────────────────────

#[tokio::test]
async fn a_detector_that_refuses_is_reported_rather_than_swallowed() {
    let (session, log, _) = open(
        ScriptedVoiceActivity::failing("onnxruntime refused the graph"),
        ScriptedTranscriber::hearing("never heard"),
        ScriptedResponder::saying("never said"),
        ScriptedSpeaker::new(),
        SessionMode::Agent,
    )
    .await;
    feed_blocks(&session, 1).await.expect("audio is accepted");
    assert!(log.wait_for_type("error").await, "{:?}", log.types());
    let error = log.first("error").expect("an error");
    assert_eq!(error["error"]["code"], json!("local_stage_unavailable"));
    assert!(
        error["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("vad")),
        "{error}"
    );
}

#[tokio::test]
async fn a_recognizer_that_refuses_to_close_the_utterance_names_its_stage() {
    let (session, log, _) = open(
        ScriptedVoiceActivity::utterance(1),
        ScriptedTranscriber::failing_on_finish("endpoint not reached"),
        ScriptedResponder::saying("never said"),
        ScriptedSpeaker::new(),
        SessionMode::Agent,
    )
    .await;
    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(log.wait_for_type("error").await, "{:?}", log.types());
    assert!(
        log.first("error")
            .and_then(|error| error["error"]["message"].as_str().map(str::to_owned))
            .is_some_and(|message| message.contains("asr")),
        "{:?}",
        log.first("error")
    );
    assert_eq!(log.count_of("response.created"), 0);
}

#[tokio::test]
async fn a_reasoning_stage_that_cannot_start_fails_the_callers_response() {
    let (session, _log, doubles) = open(
        ScriptedVoiceActivity::silent(),
        ScriptedTranscriber::deaf(),
        ScriptedResponder::failing("no GGUF loaded"),
        ScriptedSpeaker::new(),
        SessionMode::Agent,
    )
    .await;
    let outcome = session
        .speak(
            "say this",
            ResponseOrigin::Agent,
            ResponseContext::new(),
            None,
        )
        .await
        .expect("the call succeeds")
        .expect("an outcome");
    assert!(
        outcome.is_failed(),
        "a stage that cannot start must fail the caller's response: {outcome:?}"
    );
    assert_eq!(doubles.responder.turn_count(), 1);
}

#[tokio::test]
async fn a_reasoning_stage_that_dies_midway_closes_the_response_as_failed() {
    let (session, log, _) = open(
        ScriptedVoiceActivity::utterance(1),
        ScriptedTranscriber::hearing("status"),
        ScriptedResponder::turns([ScriptedTurn::failing_midway(
            "Half a sentence ",
            "context overflow",
        )]),
        ScriptedSpeaker::new(),
        SessionMode::Agent,
    )
    .await;
    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(
        log.wait_for_type("response.done").await,
        "{:?}",
        log.types()
    );
    assert_eq!(log.last_response_status().as_deref(), Some("failed"));
    assert!(
        log.first("error")
            .and_then(|error| error["error"]["message"].as_str().map(str::to_owned))
            .is_some_and(|message| message.contains("context overflow")),
        "{:?}",
        log.first("error")
    );
}

#[tokio::test]
async fn a_synthesizer_that_refuses_closes_the_response_as_failed() {
    let (session, log, _) = open(
        ScriptedVoiceActivity::utterance(1),
        ScriptedTranscriber::hearing("status"),
        ScriptedResponder::saying("Speaking now. "),
        ScriptedSpeaker::failing("voice pack missing"),
        SessionMode::Agent,
    )
    .await;
    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(
        log.wait_for_type("response.done").await,
        "{:?}",
        log.types()
    );
    assert_eq!(log.last_response_status().as_deref(), Some("failed"));
    assert!(
        log.first("error")
            .and_then(|error| error["error"]["message"].as_str().map(str::to_owned))
            .is_some_and(|message| message.contains("tts")),
        "{:?}",
        log.first("error")
    );
}

#[tokio::test]
async fn a_synthesizer_that_dies_after_it_has_spoken_still_closes_its_audio() {
    let (session, log, _) = open(
        ScriptedVoiceActivity::utterance(1),
        ScriptedTranscriber::hearing("status"),
        ScriptedResponder::saying("A longer answer here. "),
        ScriptedSpeaker::failing_midway("phonemizer died"),
        SessionMode::Agent,
    )
    .await;
    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(
        log.wait_for_type("response.done").await,
        "{:?}",
        log.types()
    );
    assert!(log.count_of("response.audio.delta") > 0, "it did speak");
    assert_ordered(&log.types(), &["response.audio.done", "response.done"]);
    assert_eq!(log.last_response_status().as_deref(), Some("failed"));
}

// ── adversarial input ───────────────────────────────────────────────────────

#[tokio::test]
async fn an_unusable_audio_append_is_dropped_rather_than_breaking_the_session() {
    let (session, log, _) = one_turn().await;
    for payload in ["not base64!!", "AAA", ""] {
        session.append_audio(payload).await.expect("accepted");
    }
    // Nothing was heard, nothing failed, and the session is still usable.
    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(
        log.wait_for(|log| log.last_response_status().is_some())
            .await
    );
    assert_eq!(log.input_transcripts(), ["turn on the lights"]);
    assert!(log.errors().is_empty(), "{:?}", log.errors());
}

#[tokio::test]
async fn a_frame_the_pipeline_has_no_opinion_about_is_ignored_not_refused() {
    let (session, log, _) = one_turn().await;
    session
        .send_frame(json!({ "type": "output_audio_buffer.clear" }))
        .await
        .expect("accepted");
    session
        .send_frame(json!({ "type": "conversation.item.truncate", "item_id": "msg_local_1", "audio_end_ms": 120 }))
        .await
        .expect("accepted");

    assert!(log.wait_for_type("conversation.item.truncated").await);
    let truncated = log.first("conversation.item.truncated").expect("an ack");
    assert_eq!(truncated["item_id"], json!("msg_local_1"));
    assert_eq!(truncated["audio_end_ms"], json!(120));
    assert_eq!(log.count_of("error"), 0, "{:?}", log.types());
}

#[tokio::test]
async fn a_commit_with_nothing_buffered_is_not_an_error() {
    // A push-to-talk client that releases the key twice.
    let (session, log, _) = one_turn().await;
    session.commit_audio().await.expect("accepted");
    session.commit_audio().await.expect("accepted");
    feed_blocks(&session, 1).await.expect("audio is accepted");
    assert!(log.wait_for_type("input_audio_buffer.speech_started").await);
    assert_eq!(log.count_of("error"), 0, "{:?}", log.types());
}

#[tokio::test]
async fn an_explicit_commit_closes_an_utterance_turn_detection_has_not_ended() {
    let (session, log, doubles) = open(
        // The VAD opens the utterance and never closes it.
        ScriptedVoiceActivity::new([
            via_realtime_local::SpeechEvent::Started,
            via_realtime_local::SpeechEvent::Speaking,
        ]),
        ScriptedTranscriber::hearing("push to talk"),
        ScriptedResponder::saying("Heard you. "),
        ScriptedSpeaker::new(),
        SessionMode::Agent,
    )
    .await;
    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(log.wait_for_type("input_audio_buffer.speech_started").await);
    assert_eq!(log.count_of("input_audio_buffer.committed"), 0);

    session.commit_audio().await.expect("accepted");
    assert!(
        log.wait_for(|log| log.last_response_status().is_some())
            .await
    );
    assert_eq!(log.input_transcripts(), ["push to talk"]);
    assert_eq!(doubles.responder.turn_count(), 1);
}

#[tokio::test]
async fn an_utterance_that_transcribes_to_nothing_is_published_but_not_answered() {
    let (session, log, doubles) = open(
        ScriptedVoiceActivity::utterance(1),
        ScriptedTranscriber::deaf(),
        ScriptedResponder::saying("never said"),
        ScriptedSpeaker::new(),
        SessionMode::Agent,
    )
    .await;
    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(
        log.wait_for_type("conversation.item.input_audio_transcription.completed")
            .await
    );
    assert_eq!(log.input_transcripts(), [""]);
    assert_eq!(log.count_of("response.created"), 0);
    assert_eq!(doubles.responder.turn_count(), 0);
}

// ── the thesis ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn the_event_vocabulary_is_one_the_shipped_gateway_already_reads() {
    // The whole point of the crate, as one assertion: every frame the pipeline
    // emits is a frame a cloud provider emits, spelled the way `via-realtime`
    // and `via-voice` spell it. A name invented here would be a name nothing
    // above this crate reads.
    let (session, log, _) = open(
        ScriptedVoiceActivity::utterance(1),
        ScriptedTranscriber::hearing("status"),
        ScriptedResponder::turns([ScriptedTurn::deltas([
            Ok(via_realtime_local::ResponseDelta::Text("Done. ".to_owned())),
            Ok(via_realtime_local::ResponseDelta::Tool(ToolCall::new(
                "call_1",
                "get_status",
                "{}",
            ))),
        ])]),
        ScriptedSpeaker::new(),
        SessionMode::Agent,
    )
    .await;
    feed_blocks(&session, 2).await.expect("audio is accepted");
    session
        .send_frame(json!({ "type": "conversation.item.truncate", "item_id": "x" }))
        .await
        .expect("accepted");
    session.cancel().await.expect("accepted");
    assert!(log.wait_for_type("conversation.item.truncated").await);
    assert!(log.wait_for_type("response.done").await);

    let seen: std::collections::BTreeSet<String> = log.types().into_iter().collect();
    assert!(
        seen.len() >= 10,
        "the session was barely exercised: {seen:?}"
    );
    for kind in &seen {
        assert!(
            EMITTED_EVENT_TYPES.contains(&kind.as_str()),
            "{kind} is not in this crate's declared vocabulary"
        );
    }
    // Every response-scoped name is one the shipped activity table recognises.
    for kind in seen.iter().filter(|kind| kind.starts_with("response.")) {
        assert!(
            RESPONSE_ACTIVITY_TYPES.contains(&kind.as_str()),
            "{kind} would not count as response activity to the Gateway"
        );
    }
}

#[tokio::test]
async fn dropping_the_session_stops_the_pipeline() {
    let (session, log, doubles) = open(
        ScriptedVoiceActivity::utterance(1),
        ScriptedTranscriber::hearing("status"),
        ScriptedResponder::turns([ScriptedTurn::saying_then_stalling("Working. ")]),
        ScriptedSpeaker::stalling(),
        SessionMode::Agent,
    )
    .await;
    let cancelled = doubles.responder.cancellations();
    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(log.wait_for_type("response.created").await);

    drop(session);
    assert!(
        log.wait_for(EventLog::is_closed).await,
        "the session never closed"
    );
    // The machine drops both stages' streams on the way out rather than leaving
    // an engine generating into a socket nobody is reading.
    assert!(
        log.wait_for(move |_| cancelled.get() >= 1).await,
        "the in-flight turn outlived the session"
    );
}

/// A frame carrying no `type` at all, and one that is not JSON.
///
/// The transport is in-process here, so these can only come from a dialect
/// adapter above — but the machine must answer them the same way: by ignoring
/// them, not by putting a protocol complaint on screen.
#[tokio::test]
async fn a_typeless_frame_is_ignored() {
    let (session, log, _) = one_turn().await;
    for frame in [json!({}), json!({ "type": 7 }), Value::Null] {
        session.send_frame(frame).await.expect("accepted");
    }
    feed_blocks(&session, 2).await.expect("audio is accepted");
    assert!(
        log.wait_for(|log| log.last_response_status().is_some())
            .await
    );
    assert_eq!(log.count_of("error"), 0, "{:?}", log.types());
}
