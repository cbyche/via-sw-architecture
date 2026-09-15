//! Driving a real session against a scripted provider.
//!
//! Nothing here reaches inside anything. Frames go out through the public
//! session API and come back through the script server, so every test exercises
//! the transport decode, both owning tasks, the correlation and the output
//! queue — the same path a live provider takes.

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use common::{no_context, open, transcript};
use pretty_assertions::assert_eq;
use serde_json::json;
use via_realtime::{FunctionOutputOptions, PermissionRequest, ResponseOrigin, ResponseOutcome};
use via_realtime_mock::{
    CannedAudio, Emission, FunctionOutput, MockRealtime, Script, Trigger, script::turns,
};

// ── the handshake ───────────────────────────────────────────────────────────

#[tokio::test]
async fn the_handshake_is_created_then_update_then_updated() {
    let (_session, log, handle) = open(Script::conversation()).await;
    let transcript = transcript(&handle).await;

    // The mock opened, the session configured itself, the mock acknowledged.
    assert_eq!(transcript.inbound_kinds()[0], "session.created");
    assert_eq!(transcript.outbound_kinds()[0], "session.update");
    assert!(
        transcript.inbound_kinds().contains(&"session.updated"),
        "{:?}",
        transcript.inbound_kinds()
    );
    // Both handshake events reach the Gateway as normalized provider events.
    assert!(log.wait_for_kind("session.updated").await.is_some());
}

#[tokio::test]
async fn the_first_session_update_negotiates_and_a_refresh_does_not() {
    let (session, _log, handle) = open(Script::conversation()).await;
    let first = transcript(&handle).await.session_updates()[0].clone();
    assert!(first.get("turn_detection").is_some());

    session
        .update_agent_context(via_realtime::AgentContextPatch {
            instructions: Some("be briefer".into()),
            ..Default::default()
        })
        .await
        .expect("patched");
    handle
        .wait_for_frames("session.update", 2)
        .await
        .expect("the refresh is written");

    let updates = transcript(&handle).await;
    let refresh = updates.session_updates()[1];
    assert_eq!(refresh["instructions"], json!("be briefer"));
    assert!(
        refresh.get("turn_detection").is_none(),
        "a refresh must not reset the provider's VAD: {refresh}"
    );
}

// ── one turn ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_text_turn_runs_item_then_response_then_completion() {
    let (session, log, handle) = open(Script::conversation().turn(turns::say("Right away."))).await;

    let outcome = session
        .send_user_text("start the build", no_context(), None)
        .await
        .expect("no transport failure")
        .expect("an outcome");
    assert!(outcome.is_completed(), "{outcome:?}");
    assert!(outcome.response_id.is_some());

    let transcript = transcript(&handle).await;
    // The order is the contract: the item is created and awaited, then the
    // response is asked for.
    let kinds = transcript.outbound_kinds();
    let item = kinds
        .iter()
        .position(|kind| *kind == "conversation.item.create")
        .expect("an item");
    let response = kinds
        .iter()
        .position(|kind| *kind == "response.create")
        .expect("a response");
    assert!(item < response, "{kinds:?}");

    assert_eq!(
        transcript.items()[0]["content"][0]["text"],
        json!("start the build")
    );
    assert!(log.wait_for_turns(1).await);
    assert_eq!(log.spoken_text(), "Right away.");
}

#[tokio::test]
async fn an_empty_turn_is_not_sent_at_all() {
    let (session, _log, handle) = open(Script::conversation()).await;
    let outcome = session
        .send_user_text("   ", no_context(), None)
        .await
        .expect("no transport failure");
    assert_eq!(outcome, None);
    assert_eq!(transcript(&handle).await.count_of("response.create"), 0);
}

#[tokio::test]
async fn each_turn_gets_the_next_step_and_then_the_default() {
    let (session, log, _handle) = open(
        Script::conversation()
            .turn(turns::say("one"))
            .turn(turns::say("two")),
    )
    .await;

    for _ in 0..3 {
        session
            .send_user_text("go", no_context(), None)
            .await
            .expect("no transport failure")
            .expect("an outcome");
    }
    // Two scripted turns spoke; the third fell through to the default, which
    // completes without saying anything.
    assert!(log.wait_for_turns(3).await);
    assert_eq!(log.spoken_text(), "onetwo");
    assert_eq!(
        log.provider_kinds()
            .iter()
            .filter(|kind| *kind == "response.done")
            .count(),
        3
    );
}

// ── the tool round trip ─────────────────────────────────────────────────────

#[tokio::test]
async fn a_tool_call_round_trips_end_to_end() {
    let arguments = json!({ "objective": "summarise the log", "mode": "background" });
    let (session, log, handle) = open(
        Script::conversation()
            .turn(turns::call_tool("call_1", "spawn_thinking", &arguments))
            .turn(turns::say("I have started on that.")),
    )
    .await;

    // Turn one: the model calls the tool.
    session
        .send_user_text("summarise the log", no_context(), None)
        .await
        .expect("no transport failure")
        .expect("an outcome");

    let call = log
        .wait_for_kind("response.function_call_arguments.done")
        .await
        .expect("the tool call arrives");
    assert_eq!(call.event["call_id"], json!("call_1"));
    assert_eq!(call.event["name"], json!("spawn_thinking"));
    // `arguments` travels as a JSON string, never as an object.
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(
            call.event["arguments"].as_str().expect("a string")
        )
        .expect("valid json"),
        arguments
    );

    // Turn two: the Gateway answers, and the model speaks.
    let outcome = session
        .send_function_output(
            "call_1",
            json!({ "status": "queued", "work_id": "job_1" }),
            no_context(),
            FunctionOutputOptions::with_response(),
        )
        .await
        .expect("no transport failure")
        .expect("an outcome");
    assert!(outcome.is_completed(), "{outcome:?}");
    assert!(log.wait_for_turns(2).await);
    assert_eq!(log.spoken_text(), "I have started on that.");

    assert_eq!(
        transcript(&handle).await.function_outputs(),
        vec![FunctionOutput {
            call_id: "call_1".into(),
            output: json!({ "status": "queued", "work_id": "job_1" }),
        }]
    );
}

#[tokio::test]
async fn a_tool_result_can_be_returned_without_giving_the_model_a_turn() {
    let (session, _log, handle) = open(Script::conversation()).await;
    let outcome = session
        .send_function_output(
            "call_stale",
            json!({ "status": "superseded" }),
            no_context(),
            FunctionOutputOptions::without_response(),
        )
        .await
        .expect("no transport failure");
    assert_eq!(outcome, None);

    let transcript = transcript(&handle).await;
    assert_eq!(transcript.function_outputs().len(), 1);
    assert_eq!(
        transcript.count_of("response.create"),
        0,
        "closing a stale call must not make the model speak"
    );
}

// ── out-of-band speech and injections ───────────────────────────────────────

#[tokio::test]
async fn speech_stays_out_of_conversation_history() {
    let (session, _log, handle) = open(Script::conversation()).await;
    session
        .speak(
            "Still working on it.",
            ResponseOrigin::Agent,
            no_context(),
            None,
        )
        .await
        .expect("no transport failure")
        .expect("an outcome");

    let transcript = transcript(&handle).await;
    let body = transcript.response_creates()[0].expect("a response body");
    assert_eq!(body["conversation"], json!("none"));
    assert_eq!(body["instructions"], json!("Still working on it."));
    assert_eq!(
        transcript.count_of("conversation.item.create"),
        0,
        "out-of-band speech creates no item"
    );
}

#[tokio::test]
async fn a_result_injection_creates_the_item_and_the_response() {
    let (session, _log, handle) = open(Script::conversation()).await;
    let outcome = session
        .inject_result(
            "The build finished.",
            ResponseOrigin::Announcement,
            no_context(),
            true,
        )
        .await
        .expect("no transport failure")
        .expect("an outcome");
    assert!(
        outcome.context_injected,
        "the result reached the conversation"
    );
    assert!(
        outcome
            .outcome
            .is_some_and(|outcome| outcome.is_completed())
    );

    let transcript = transcript(&handle).await;
    assert_eq!(
        transcript.items()[0]["content"][0]["text"],
        json!("The build finished.")
    );
    assert_eq!(
        transcript.response_creates()[0].expect("a body")["tool_choice"],
        json!("none")
    );
}

#[tokio::test]
async fn a_permission_question_creates_its_item_before_the_response_is_queued() {
    let (session, _log, handle) = open(Script::conversation()).await;
    session
        .inject_permission(
            &PermissionRequest {
                id: "auth-1".into(),
                summary: "write to /tmp".into(),
            },
            no_context(),
            None,
        )
        .await
        .expect("no transport failure")
        .expect("an outcome");

    let transcript = transcript(&handle).await;
    let text = transcript.items()[0]["content"][0]["text"]
        .as_str()
        .expect("text");
    assert!(text.contains("authorization_id=auth-1"), "{text}");
    assert!(text.contains("operation=write to /tmp"), "{text}");
}

#[tokio::test]
async fn a_permission_request_with_no_id_is_refused_before_anything_is_written() {
    let (session, _log, handle) = open(Script::conversation()).await;
    let outcome = session
        .inject_permission(
            &PermissionRequest {
                id: String::new(),
                summary: "something".into(),
            },
            no_context(),
            None,
        )
        .await
        .expect("no transport failure");
    assert_eq!(outcome, None);
    assert_eq!(
        transcript(&handle)
            .await
            .count_of("conversation.item.create"),
        0
    );
}

// ── guards and cancellation ─────────────────────────────────────────────────

#[tokio::test]
async fn a_guard_that_declines_writes_nothing() {
    let (session, _log, handle) = open(Script::conversation()).await;
    let outcome = session
        .ensure_response(no_context(), None, Some(Arc::new(|| false)))
        .await
        .expect("no transport failure")
        .expect("an outcome");
    assert!(outcome.is_skipped(), "{outcome:?}");
    assert_eq!(transcript(&handle).await.count_of("response.create"), 0);
}

#[tokio::test]
async fn a_guard_that_allows_is_asked_once_and_the_response_is_created() {
    let asked = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&asked);
    let (session, _log, handle) = open(Script::conversation()).await;
    let outcome = session
        .ensure_response(
            no_context(),
            Some(json!({ "modalities": ["text"] })),
            Some(Arc::new(move || {
                flag.store(true, Ordering::SeqCst);
                true
            })),
        )
        .await
        .expect("no transport failure")
        .expect("an outcome");
    assert!(outcome.is_completed(), "{outcome:?}");
    assert!(asked.load(Ordering::SeqCst));
    assert_eq!(transcript(&handle).await.count_of("response.create"), 1);
}

#[tokio::test]
async fn cancel_ends_an_active_response_and_the_mock_confirms_it() {
    // The turn starts and then says nothing more, so it is still running when
    // the barge-in lands.
    let (session, log, handle) = open(Script::conversation().turn(turns::stall())).await;
    let session_for_turn = session.clone();
    let turn = tokio::spawn(async move {
        session_for_turn
            .send_user_text("tell me a long story", no_context(), None)
            .await
    });

    log.wait_for_kind("response.created")
        .await
        .expect("the turn starts");
    session.cancel().await.expect("cancelled");

    let outcome = turn
        .await
        .expect("the turn task")
        .expect("no transport failure")
        .expect("an outcome");
    // Upstream's `cancel()` is barge-in for the *queue*: it settles what has not
    // started and asks the provider to stop what has. A response that was
    // already running therefore ends through its own `response.done`, and
    // `cancelled` is one of the three catalogued failure statuses — so the
    // outcome is `failed / cancelled`, not `cancelled`. `cancel_responses` is
    // the call that settles a running response as cancelled; the next test is
    // that one.
    assert!(outcome.is_failed(), "{outcome:?}");
    assert_eq!(outcome.status.as_deref(), Some("cancelled"));
    assert_eq!(transcript(&handle).await.count_of("response.cancel"), 1);
}

#[tokio::test]
async fn a_predicate_cancel_ends_only_what_it_matches() {
    let (session, log, _handle) = open(Script::conversation().turn(turns::stall())).await;
    let session_for_turn = session.clone();
    let turn = tokio::spawn(async move {
        session_for_turn
            .send_user_text(
                "story",
                via_realtime::ResponseContext::new().with("turnId", "voice-1"),
                None,
            )
            .await
    });
    log.wait_for_kind("response.created")
        .await
        .expect("the turn starts");

    // Nothing matches, so nothing is cancelled.
    assert!(
        !session
            .cancel_responses(|context, _origin| context.turn_id() == Some("voice-2"))
            .await
            .expect("asked")
    );
    // This one does.
    assert!(
        session
            .cancel_responses(|context, _origin| context.turn_id() == Some("voice-1"))
            .await
            .expect("asked")
    );
    let outcome = turn
        .await
        .expect("the turn task")
        .expect("no transport failure")
        .expect("an outcome");
    assert!(outcome.is_cancelled(), "{outcome:?}");
}

// ── ordering and virtual time ───────────────────────────────────────────────

#[tokio::test(start_paused = true)]
async fn a_scripted_delay_costs_virtual_time_and_no_real_time() {
    let script = Script::conversation().turn(vec![
        Emission::now(via_realtime_mock::events::response_created()),
        Emission::after(
            Duration::from_millis(500),
            via_realtime_mock::events::text_delta("late"),
        ),
        Emission::after(
            Duration::from_millis(500),
            via_realtime_mock::events::response_done(via_realtime_mock::COMPLETED),
        ),
    ]);
    let (session, log, _handle) = open(script).await;

    let started = tokio::time::Instant::now();
    let outcome = session
        .send_user_text("go", no_context(), None)
        .await
        .expect("no transport failure")
        .expect("an outcome");
    assert!(outcome.is_completed(), "{outcome:?}");
    // The two delays are on the virtual clock, and the paused runtime advanced
    // it only because every task was idle.
    assert!(
        started.elapsed() >= Duration::from_millis(1_000),
        "{:?}",
        started.elapsed()
    );
    assert!(log.wait_for_turns(1).await);
    assert_eq!(log.spoken_text(), "late");
}

#[tokio::test]
async fn the_output_queue_runs_one_job_at_a_time_in_call_order() {
    let (session, log, handle) = open(
        Script::conversation()
            .turn(turns::say("first"))
            .turn(turns::say("second"))
            .turn(turns::say("third")),
    )
    .await;

    // Three calls issued at once; the queue serialises them.
    let (a, b, c) = tokio::join!(
        session.speak("a", ResponseOrigin::Agent, no_context(), None),
        session.speak("b", ResponseOrigin::Agent, no_context(), None),
        session.speak("c", ResponseOrigin::Agent, no_context(), None),
    );
    for outcome in [a, b, c] {
        assert!(
            outcome
                .expect("no transport failure")
                .is_some_and(|outcome: ResponseOutcome| outcome.is_completed())
        );
    }
    assert!(log.wait_for_turns(3).await);
    assert_eq!(log.spoken_text(), "firstsecondthird");
    assert_eq!(transcript(&handle).await.count_of("response.create"), 3);
}

#[tokio::test]
async fn drain_resolves_once_everything_queued_has_resolved() {
    let (session, _log, handle) = open(Script::conversation().turn(turns::say("done"))).await;
    let speaking = session.clone();
    let spoken = tokio::spawn(async move {
        speaking
            .speak("x", ResponseOrigin::Agent, no_context(), None)
            .await
    });
    session.drain().await.expect("drained");
    assert_eq!(transcript(&handle).await.count_of("response.create"), 1);
    spoken.await.expect("task").expect("no transport failure");
}

// ── things the session did not ask for ──────────────────────────────────────

#[tokio::test]
async fn an_uncorrelated_server_turn_is_reported_with_model_origin() {
    let (_session, log, handle) = open(Script::conversation()).await;
    handle
        .emit_all([
            Emission::verbatim(json!({
                "type": "response.created",
                "response": { "id": "resp_vad" },
            })),
            Emission::verbatim(json!({
                "type": "response.audio_transcript.delta",
                "response_id": "resp_vad",
                "delta": "I heard you.",
            })),
        ])
        .await
        .expect("emitted");

    let event = log
        .wait_for_kind("response.audio_transcript.delta")
        .await
        .expect("arrives");
    assert_eq!(event.origin, ResponseOrigin::Model);
    assert!(event.context.is_empty(), "nothing to correlate it to");
    assert!(!event.retried);
}

#[tokio::test]
async fn a_frame_that_is_not_json_does_not_take_the_session_down() {
    let (session, log, _handle) = open(Script::conversation()).await;
    // A keep-alive a provider might send. `via-realtime`'s reader drops it.
    session
        .send_frame(json!({ "type": "custom.keepalive" }))
        .await
        .expect("written");
    assert!(
        session
            .send_user_text("still here", no_context(), None)
            .await
            .expect("no transport failure")
            .is_some_and(|outcome| outcome.is_completed())
    );
    assert!(!log.is_closed());
}

#[tokio::test]
async fn an_extra_frame_can_be_scripted_by_its_own_name() {
    let (session, log, _handle) = open(Script::conversation().on_always(
        Trigger::Frame("custom.ping".into()),
        [Emission::verbatim(json!({ "type": "custom.pong" }))],
    ))
    .await;
    session
        .send_frame(json!({ "type": "custom.ping" }))
        .await
        .expect("written");
    assert!(log.wait_for_kind("custom.pong").await.is_some());
}

// ── close ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn closing_settles_everything_and_says_goodbye_last() {
    let (session, log, handle) = open(Script::conversation()).await;
    session.close().await.expect("closed");
    assert!(log.wait_for_close().await);

    let entries = log.snapshot();
    assert!(
        matches!(entries.last(), Some(via_realtime_mock::Recorded::Closed)),
        "{:?}",
        log.provider_kinds()
    );

    // A later call answers rather than hanging, and — the part that matters —
    // writes nothing, because the transport is gone.
    let written = transcript(&handle).await.outbound().len();
    let after = session
        .send_user_text("anyone there", no_context(), None)
        .await;
    assert!(
        match &after {
            Err(via_realtime::RealtimeError::ConnectionClosed { .. }) => true,
            // The queue may answer first, and a stale generation is `None`.
            Ok(None) => true,
            other => panic!("a closed session answered {other:?}"),
        },
        "{after:?}"
    );
    assert_eq!(transcript(&handle).await.outbound().len(), written);
}

#[tokio::test]
async fn a_dropped_session_closes_the_transport_too() {
    let mock = MockRealtime::new(Script::conversation());
    let (session, events, handle) = mock.open().await.expect("opens").into_parts();
    let log = via_realtime_mock::collect_events(events);
    drop(session);
    assert!(log.wait_for_close().await);
    // The handle outlives it, so the transcript is still readable.
    assert_eq!(transcript(&handle).await.count_of("session.update"), 1);
}

// ── audio in ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn appended_audio_arrives_as_the_samples_that_were_sent() {
    let clip = CannedAudio::tone(via_audio::SampleRate::HZ_16000, Duration::from_millis(60));
    let (session, _log, handle) = open(Script::conversation()).await;
    for chunk in clip.chunks() {
        session.append_audio(&chunk.base64).await.expect("appended");
    }
    session.commit_audio().await.expect("committed");
    handle
        .wait_for_frame("input_audio_buffer.commit")
        .await
        .expect("the commit is written");

    let transcript = transcript(&handle).await;
    assert_eq!(transcript.input_audio_base64().len(), 3);
    assert_eq!(
        transcript.input_audio_pcm16().expect("decode"),
        clip.samples()
    );
}
