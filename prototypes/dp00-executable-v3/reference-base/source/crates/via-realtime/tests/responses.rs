//! Correlation, the output queue, the two watchdogs and the busy-retry ladder.
//!
//! Ported from `server/test/realtime-provider.test.mjs`. Everything with a timer
//! runs under `start_paused` so the 30-second and 120-second budgets it asserts
//! cost microseconds.

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use common::harness::{
    Harness, Recorded, acknowledge_item, complete_response, expect_any_frame, expect_frame,
    finish_response, provider_error, quick_timeouts, response_activity, start_response,
};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use via_realtime::testing::TestProvider;
use via_realtime::{
    BUSY_RETRY_DELAYS, DIAGNOSTIC_RESPONSE_TIMEOUT, InputProjection, OutcomeKind, OutcomePhase,
    POST_CANCEL_RECOVERY, PermissionRequest, ProviderCapabilities, RESPONSE_CORRELATION_KEY,
    ResponseContext, ResponseOrigin, SessionOptions,
};

/// Let the two owning tasks make progress without advancing the clock.
async fn settle() {
    for _ in 0..64 {
        tokio::task::yield_now().await;
    }
}

// ── correlation ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_queued_announcement_completes_only_after_response_done() {
    let Harness {
        session,
        mut peer,
        events,
        ..
    } = Harness::beta().await;

    let driver = tokio::spawn(async move {
        let frame = start_response(&mut peer, "response-1").await;
        // The response exists but has not finished; the caller must still be
        // waiting.
        (frame, peer)
    });
    let speaking = tokio::spawn({
        let session = session.clone();
        async move {
            session
                .speak(
                    "the task is done",
                    ResponseOrigin::Agent,
                    ResponseContext::new()
                        .with("turnId", "voice-100-1")
                        .with("taskId", "job-1"),
                    None,
                )
                .await
        }
    });
    let (frame, peer) = driver.await.expect("driver");

    // `response.created` alone is not completion.
    let created = events.wait_for_kind("response.created").await;
    assert_eq!(created.origin, ResponseOrigin::Agent);
    assert_eq!(created.context.turn_id(), Some("voice-100-1"));
    assert_eq!(created.context.task_id(), Some("job-1"));
    assert!(!speaking.is_finished());

    finish_response(&peer, "response-1", "completed");
    let outcome = speaking
        .await
        .expect("task")
        .expect("call")
        .expect("an outcome");
    assert_eq!(outcome.kind, OutcomeKind::Completed);
    assert_eq!(outcome.response_id.as_deref(), Some("response-1"));

    // Out of band: the utterance never becomes conversation history.
    assert_eq!(frame["response"]["conversation"], json!("none"));
    assert_eq!(frame["response"]["instructions"], json!("the task is done"));

    let done = events.wait_for_kind("response.done").await;
    assert_eq!(done.origin, ResponseOrigin::Agent);
    assert_eq!(done.context.task_id(), Some("job-1"));
}

#[tokio::test]
async fn an_uncorrelated_response_is_reported_as_the_models_own() {
    let Harness {
        peer,
        events,
        session: _session,
        ..
    } = Harness::beta().await;
    peer.send(json!({
        "type": "response.created",
        "response": { "id": "resp-automatic" },
    }));
    let created = events.wait_for_kind("response.created").await;
    assert_eq!(created.origin, ResponseOrigin::Model);
    assert!(created.context.is_empty());
    assert!(!created.retried);
}

#[tokio::test]
async fn each_failing_status_is_a_failed_outcome() {
    for (status, expected) in [
        ("completed", OutcomeKind::Completed),
        ("failed", OutcomeKind::Failed),
        ("cancelled", OutcomeKind::Failed),
        ("incomplete", OutcomeKind::Failed),
        // Not one of the three catalogued failures, so upstream treats it as
        // success — reproduced deliberately.
        ("error", OutcomeKind::Completed),
    ] {
        let Harness {
            session, mut peer, ..
        } = Harness::beta().await;
        let status_owned = status.to_owned();
        let driver = tokio::spawn(async move {
            start_response(&mut peer, "r1").await;
            finish_response(&peer, "r1", &status_owned);
            peer
        });
        let outcome = session
            .speak("hello", ResponseOrigin::Agent, ResponseContext::new(), None)
            .await
            .expect("call")
            .expect("an outcome");
        let _peer = driver.await.expect("driver");

        assert_eq!(outcome.kind, expected, "{status}");
        assert_eq!(outcome.response_id.as_deref(), Some("r1"), "{status}");
        if expected == OutcomeKind::Failed {
            assert_eq!(outcome.status.as_deref(), Some(status));
        }
    }
}

#[tokio::test]
async fn an_unscoped_error_is_associated_with_the_sole_active_response() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    let driver = tokio::spawn(async move {
        start_response(&mut peer, "response-error").await;
        provider_error(&peer, "provider failed");
        peer
    });
    let outcome = session
        .speak("test", ResponseOrigin::Agent, ResponseContext::new(), None)
        .await
        .expect("call")
        .expect("an outcome");
    let _peer = driver.await.expect("driver");

    assert_eq!(outcome.kind, OutcomeKind::Failed);
    assert_eq!(outcome.response_id.as_deref(), Some("response-error"));
    assert_eq!(outcome.status, None);
}

#[tokio::test]
async fn an_error_before_correlation_belongs_to_the_start_that_is_waiting() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    let driver = tokio::spawn(async move {
        expect_frame(&mut peer, "response.create").await;
        provider_error(&peer, "provider refused outright");
        peer
    });
    let outcome = session
        .speak("test", ResponseOrigin::Agent, ResponseContext::new(), None)
        .await
        .expect("call")
        .expect("an outcome");
    let _peer = driver.await.expect("driver");

    assert_eq!(outcome.kind, OutcomeKind::Failed);
    assert_eq!(outcome.response_id, None);
}

#[tokio::test]
async fn a_response_created_with_no_id_fails_closed_instead_of_hanging() {
    // Upstream leaves this pending unreachable — removed from the correlation
    // queue, timer cleared, never registered — so the caller's promise never
    // settles and the whole output queue stops. Neither shipped dialect can
    // reach it; VIA settles it rather than hanging.
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    let driver = tokio::spawn(async move {
        expect_frame(&mut peer, "response.create").await;
        peer.send(json!({ "type": "response.created", "response": {} }));
        peer
    });
    let outcome = session
        .speak("test", ResponseOrigin::Agent, ResponseContext::new(), None)
        .await
        .expect("call")
        .expect("an outcome");
    let _peer = driver.await.expect("driver");

    assert_eq!(outcome.kind, OutcomeKind::Failed);
    assert_eq!(outcome.phase, Some(OutcomePhase::Correlation));
}

#[tokio::test]
async fn responses_are_serialized_so_only_one_start_awaits_correlation() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    let first = tokio::spawn({
        let session = session.clone();
        async move {
            session
                .speak("first", ResponseOrigin::Agent, ResponseContext::new(), None)
                .await
        }
    });
    let second = tokio::spawn({
        let session = session.clone();
        async move {
            session
                .speak(
                    "second",
                    ResponseOrigin::Agent,
                    ResponseContext::new(),
                    None,
                )
                .await
        }
    });

    let one = expect_frame(&mut peer, "response.create").await;
    assert_eq!(one["response"]["instructions"], json!("first"));
    settle().await;
    // The second is queued behind the first's *outcome*, not its write.
    assert_eq!(peer.drain_frames(), Vec::<Value>::new());
    assert!(!second.is_finished());

    peer.send(json!({ "type": "response.created", "response": { "id": "r1" } }));
    finish_response(&peer, "r1", "completed");
    assert_eq!(
        first
            .await
            .expect("task")
            .expect("call")
            .expect("outcome")
            .kind,
        OutcomeKind::Completed
    );

    let two = expect_frame(&mut peer, "response.create").await;
    assert_eq!(two["response"]["instructions"], json!("second"));
    peer.send(json!({ "type": "response.created", "response": { "id": "r2" } }));
    finish_response(&peer, "r2", "completed");
    assert_eq!(
        second
            .await
            .expect("task")
            .expect("call")
            .expect("outcome")
            .kind,
        OutcomeKind::Completed
    );
}

#[tokio::test]
async fn an_implicit_response_holds_the_queue_until_it_finishes() {
    // A compatible provider may emit output activity without ever sending
    // `response.created`; the activity alone proves a response exists.
    let Harness {
        session,
        mut peer,
        events,
        ..
    } = Harness::beta().await;
    peer.send(json!({
        "type": "response.output_audio_transcript.done",
        "response_id": "resp-implicit",
        "transcript": "hello",
    }));
    events
        .wait_for_kind("response.output_audio_transcript.done")
        .await;

    let speaking = tokio::spawn({
        let session = session.clone();
        async move {
            session
                .speak(
                    "queued",
                    ResponseOrigin::Agent,
                    ResponseContext::new(),
                    None,
                )
                .await
        }
    });
    settle().await;
    assert_eq!(peer.drain_frames(), Vec::<Value>::new());

    finish_response(&peer, "resp-implicit", "completed");
    let frame = start_response(&mut peer, "r1").await;
    assert_eq!(frame["response"]["instructions"], json!("queued"));
    finish_response(&peer, "r1", "completed");
    assert_eq!(
        speaking
            .await
            .expect("task")
            .expect("call")
            .expect("outcome")
            .kind,
        OutcomeKind::Completed
    );
}

// ── the two watchdogs ───────────────────────────────────────────────────────

#[tokio::test(start_paused = true)]
async fn an_unstarted_response_times_out_as_uncertain_rather_than_successful() {
    let Harness {
        session, mut peer, ..
    } = Harness::open(
        TestProvider::new("beta").shared(),
        quick_timeouts(Duration::from_secs(30), Duration::from_secs(120)),
    )
    .await;
    let driver = tokio::spawn(async move {
        expect_frame(&mut peer, "response.create").await;
        peer
    });
    let started = tokio::time::Instant::now();
    let outcome = session
        .speak("hello", ResponseOrigin::Agent, ResponseContext::new(), None)
        .await
        .expect("call")
        .expect("an outcome");
    let _peer = driver.await.expect("driver");

    assert_eq!(outcome.kind, OutcomeKind::TimedOut);
    assert_eq!(outcome.phase, Some(OutcomePhase::Start));
    assert_eq!(outcome.response_id, None);
    assert_eq!(started.elapsed(), Duration::from_secs(30));
}

#[tokio::test(start_paused = true)]
async fn a_provider_response_start_budget_overrides_the_default() {
    let provider = TestProvider::new("slow-local")
        .with_response_start_timeout(Some(Duration::from_secs(60)))
        .shared();
    let Harness {
        session, mut peer, ..
    } = Harness::open(provider, SessionOptions::default()).await;
    let driver = tokio::spawn(async move {
        expect_frame(&mut peer, "response.create").await;
        peer
    });
    let started = tokio::time::Instant::now();
    session
        .speak("hello", ResponseOrigin::Agent, ResponseContext::new(), None)
        .await
        .expect("call");
    let _peer = driver.await.expect("driver");
    assert_eq!(started.elapsed(), Duration::from_secs(60));
}

#[tokio::test(start_paused = true)]
async fn a_long_response_stays_alive_while_output_continues() {
    let Harness {
        session, mut peer, ..
    } = Harness::open(
        TestProvider::new("beta").shared(),
        quick_timeouts(Duration::from_secs(30), Duration::from_millis(80)),
    )
    .await;

    let driver = tokio::spawn(async move {
        start_response(&mut peer, "response-long").await;
        for _ in 0..5 {
            tokio::time::sleep(Duration::from_millis(50)).await;
            response_activity(&peer, "response-long");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
        finish_response(&peer, "response-long", "completed");
        peer
    });
    let outcome = session
        .speak(
            "a long spoken answer",
            ResponseOrigin::Agent,
            ResponseContext::new(),
            None,
        )
        .await
        .expect("call")
        .expect("an outcome");
    let mut peer = driver.await.expect("driver");

    assert_eq!(outcome.kind, OutcomeKind::Completed);
    // The inactivity window is a sliding one: 300 ms of speech under an 80 ms
    // window is fine as long as something arrives inside every window.
    assert!(
        !peer
            .drain_frames()
            .iter()
            .any(|frame| frame["type"] == json!("response.cancel")),
        "a live response must never be cancelled"
    );
}

#[tokio::test(start_paused = true)]
async fn an_inactive_response_is_cancelled_diagnosed_and_retired() {
    let Harness {
        session,
        mut peer,
        events,
        ..
    } = Harness::open(
        TestProvider::new("beta").shared(),
        quick_timeouts(Duration::from_secs(30), Duration::from_millis(10)),
    )
    .await;

    let driver = tokio::spawn(async move {
        start_response(&mut peer, "response-stalled").await;
        peer
    });
    let outcome = session
        .speak(
            "this response stops talking",
            ResponseOrigin::Agent,
            ResponseContext::new(),
            None,
        )
        .await
        .expect("call")
        .expect("an outcome");
    let mut peer = driver.await.expect("driver");

    assert_eq!(outcome.kind, OutcomeKind::TimedOut);
    assert_eq!(outcome.phase, Some(OutcomePhase::Inactivity));
    assert_eq!(outcome.response_id.as_deref(), Some("response-stalled"));

    let entries = events
        .wait_for("a diagnostic", |entries| {
            entries.iter().any(|entry| entry.diagnostic().is_some())
        })
        .await;
    let diagnostics: Vec<_> = entries.iter().filter_map(Recorded::diagnostic).collect();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].event, DIAGNOSTIC_RESPONSE_TIMEOUT);
    assert_eq!(diagnostics[0].provider, "beta");
    assert_eq!(diagnostics[0].response_id, "response-stalled");
    assert_eq!(diagnostics[0].phase, "inactivity");
    assert!(diagnostics[0].inactivity_ms >= 10);

    // The provider is asked to stop …
    let frames = peer.drain_frames();
    assert_eq!(
        frames.last().map(|frame| frame["type"].clone()),
        Some(json!("response.cancel"))
    );

    // … and the id is retired even if it never confirms, so the session does
    // not stay busy for ever.
    let after = tokio::time::Instant::now();
    let next = tokio::spawn({
        let session = session.clone();
        async move {
            session
                .speak("next", ResponseOrigin::Agent, ResponseContext::new(), None)
                .await
        }
    });
    expect_frame(&mut peer, "response.create").await;
    assert!(after.elapsed() <= POST_CANCEL_RECOVERY);
    next.abort();
}

#[tokio::test(start_paused = true)]
async fn activity_that_lands_after_the_timer_reopens_the_window() {
    // The watchdog re-arms for the remaining time rather than cancelling a
    // response that spoke a moment ago.
    let Harness {
        session, mut peer, ..
    } = Harness::open(
        TestProvider::new("beta").shared(),
        quick_timeouts(Duration::from_secs(30), Duration::from_millis(100)),
    )
    .await;
    let driver = tokio::spawn(async move {
        start_response(&mut peer, "r1").await;
        tokio::time::sleep(Duration::from_millis(90)).await;
        response_activity(&peer, "r1");
        tokio::time::sleep(Duration::from_millis(90)).await;
        response_activity(&peer, "r1");
        finish_response(&peer, "r1", "completed");
        peer
    });
    let outcome = session
        .speak("hello", ResponseOrigin::Agent, ResponseContext::new(), None)
        .await
        .expect("call")
        .expect("an outcome");
    let _peer = driver.await.expect("driver");
    assert_eq!(outcome.kind, OutcomeKind::Completed);
}

// ── guards ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_late_guard_skips_the_response_without_writing_anything() {
    let mut harness = Harness::beta().await;
    let outcome = harness
        .session
        .speak(
            "a duplicate project note",
            ResponseOrigin::Agent,
            ResponseContext::new().with("taskId", "job-1"),
            Some(Arc::new(|| false)),
        )
        .await
        .expect("call")
        .expect("an outcome");

    assert_eq!(outcome.kind, OutcomeKind::Skipped);
    assert_eq!(outcome.phase, Some(OutcomePhase::Deduplicated));
    assert_eq!(harness.peer.drain_frames(), Vec::<Value>::new());
}

#[tokio::test]
async fn ensure_response_writes_the_body_it_was_given() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    let driver = tokio::spawn(async move {
        let frame = complete_response(&mut peer, "response-permission").await;
        (frame, peer)
    });
    let outcome = session
        .ensure_response(
            ResponseContext::new().with("turnId", "permission-turn"),
            Some(json!({ "instructions": "decide again", "modalities": ["text"] })),
            Some(Arc::new(|| true)),
        )
        .await
        .expect("call")
        .expect("an outcome");
    let (frame, _peer) = driver.await.expect("driver");

    assert_eq!(
        frame["response"],
        json!({ "instructions": "decide again", "modalities": ["text"] })
    );
    assert_eq!(outcome.kind, OutcomeKind::Completed);
}

// ── injection ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_work_result_is_injected_as_an_item_and_a_tool_free_response() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    let driver = tokio::spawn(async move {
        let item = acknowledge_item(&mut peer).await;
        let response = complete_response(&mut peer, "response-result").await;
        (item, response, peer)
    });
    let injected = session
        .inject_result(
            "the background task finished",
            ResponseOrigin::Announcement,
            ResponseContext::new()
                .with("turnId", "turn-result")
                .with("taskId", "job-result"),
            true,
        )
        .await
        .expect("call")
        .expect("an outcome");
    let (item, response, _peer) = driver.await.expect("driver");

    assert_eq!(item["item"]["role"], json!("user"));
    assert_eq!(
        item["item"]["content"],
        json!([{ "type": "input_text", "text": "the background task finished" }])
    );
    assert_eq!(response["response"]["tool_choice"], json!("none"));
    assert!(response["response"].get("conversation").is_none());
    assert!(injected.context_injected);
    assert_eq!(
        injected.outcome.map(|outcome| outcome.kind),
        Some(OutcomeKind::Completed)
    );
}

#[tokio::test]
async fn an_injection_without_context_writes_only_the_response() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    let driver = tokio::spawn(async move {
        // Only one frame is written, so it must be answered rather than
        // waited for a second time.
        let first = expect_any_frame(&mut peer).await;
        peer.send(json!({ "type": "response.created", "response": { "id": "r1" } }));
        finish_response(&peer, "r1", "completed");
        (first, peer)
    });
    let injected = session
        .inject_result(
            "already in the conversation",
            ResponseOrigin::Announcement,
            ResponseContext::new(),
            false,
        )
        .await
        .expect("call")
        .expect("an outcome");
    let (first, _peer) = driver.await.expect("driver");

    assert_eq!(first["type"], json!("response.create"));
    assert!(!injected.context_injected);
}

#[tokio::test]
async fn a_permission_item_is_created_before_the_question_is_queued() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;

    // A response is already speaking, so the spoken question has to wait — but
    // the item must not, because the user can already answer it in the UI.
    peer.send(json!({
        "type": "response.created",
        "response": { "id": "resp-busy" },
    }));

    let asking = tokio::spawn({
        let session = session.clone();
        async move {
            session
                .inject_permission(
                    &PermissionRequest {
                        id: "permission-one".into(),
                        summary: "read the system memory".into(),
                    },
                    ResponseContext::new().with("authorizationId", "permission-one"),
                    None,
                )
                .await
        }
    });

    let item = acknowledge_item(&mut peer).await;
    let text = item["item"]["content"][0]["text"].as_str().expect("text");
    assert!(text.starts_with("<backend_permission_request>"), "{text}");
    assert!(text.contains("authorization_id=permission-one"), "{text}");
    assert!(text.contains("operation=read the system memory"), "{text}");

    settle().await;
    assert_eq!(peer.drain_frames(), Vec::<Value>::new());

    finish_response(&peer, "resp-busy", "completed");
    let response = complete_response(&mut peer, "resp-permission").await;
    assert_eq!(response["response"]["tool_choice"], json!("none"));
    assert_eq!(
        asking
            .await
            .expect("task")
            .expect("call")
            .expect("outcome")
            .kind,
        OutcomeKind::Completed
    );
}

#[tokio::test]
async fn a_permission_answered_during_injection_is_never_spoken() {
    // The item reaches the model at once, but the spoken question waits behind
    // an active response — and by the time it reaches the head of the queue the
    // user has already answered it in the UI. Asking aloud then is worse than
    // saying nothing, which is what the late guard is for.
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    peer.send(json!({
        "type": "response.created",
        "response": { "id": "resp-busy" },
    }));

    let still_pending = Arc::new(AtomicBool::new(true));
    let guard = {
        let still_pending = Arc::clone(&still_pending);
        Arc::new(move || still_pending.load(Ordering::SeqCst))
    };
    let asking = tokio::spawn({
        let session = session.clone();
        async move {
            session
                .inject_permission(
                    &PermissionRequest {
                        id: "permission-race".into(),
                        summary: "edit snake.py".into(),
                    },
                    ResponseContext::new().with("authorizationId", "permission-race"),
                    Some(guard),
                )
                .await
        }
    });
    acknowledge_item(&mut peer).await;
    settle().await;

    still_pending.store(false, Ordering::SeqCst);
    finish_response(&peer, "resp-busy", "completed");

    let outcome = asking
        .await
        .expect("task")
        .expect("call")
        .expect("an outcome");
    assert_eq!(outcome.kind, OutcomeKind::Skipped);
    assert_eq!(outcome.phase, Some(OutcomePhase::Deduplicated));
    assert!(
        !peer
            .drain_frames()
            .iter()
            .any(|frame| frame["type"] == json!("response.create")),
        "the question must never have been asked"
    );
}

#[tokio::test]
async fn a_permission_with_no_identity_is_refused_without_a_frame() {
    let mut harness = Harness::beta().await;
    for permission in [
        PermissionRequest {
            id: String::new(),
            summary: "x".into(),
        },
        PermissionRequest {
            id: "a".into(),
            summary: String::new(),
        },
    ] {
        assert_eq!(
            harness
                .session
                .inject_permission(&permission, ResponseContext::new(), None)
                .await
                .expect("call"),
            None
        );
    }
    assert_eq!(harness.peer.drain_frames(), Vec::<Value>::new());
}

// ── cancellation ────────────────────────────────────────────────────────────

#[tokio::test]
async fn cancelling_before_response_created_releases_the_queue_entry() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    let speaking = tokio::spawn({
        let session = session.clone();
        async move {
            session
                .speak(
                    "announce later",
                    ResponseOrigin::Announcement,
                    ResponseContext::new(),
                    None,
                )
                .await
        }
    });
    expect_frame(&mut peer, "response.create").await;
    session.cancel().await.expect("cancel");

    let outcome = speaking
        .await
        .expect("task")
        .expect("call")
        .expect("an outcome");
    assert_eq!(outcome.kind, OutcomeKind::Cancelled);
    assert_eq!(outcome.phase, Some(OutcomePhase::Start));
    // Something was in flight, so the provider is told to stop.
    assert_eq!(
        expect_frame(&mut peer, "response.cancel").await["type"],
        json!("response.cancel")
    );
}

#[tokio::test]
async fn cancelling_with_nothing_in_flight_writes_nothing() {
    let mut harness = Harness::beta().await;
    harness.session.cancel().await.expect("cancel");
    settle().await;
    assert_eq!(harness.peer.drain_frames(), Vec::<Value>::new());
}

#[tokio::test]
async fn cancelling_an_active_response_asks_the_provider_to_stop() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    let speaking = tokio::spawn({
        let session = session.clone();
        async move {
            session
                .speak(
                    "in flight",
                    ResponseOrigin::Permission,
                    ResponseContext::new().with("authorizationId", "auth-one"),
                    None,
                )
                .await
        }
    });
    start_response(&mut peer, "r1").await;
    settle().await;

    let cancelled = session
        .cancel_responses(|context, origin| {
            origin == ResponseOrigin::Permission && context.authorization_id() == Some("auth-one")
        })
        .await
        .expect("cancel");
    assert!(cancelled, "a started response was cancelled");

    let outcome = speaking
        .await
        .expect("task")
        .expect("call")
        .expect("an outcome");
    assert_eq!(outcome.kind, OutcomeKind::Cancelled);
    assert_eq!(outcome.phase, Some(OutcomePhase::Completion));
    assert!(
        peer.drain_frames()
            .iter()
            .any(|frame| frame["type"] == json!("response.cancel"))
    );
}

#[tokio::test]
async fn cancel_responses_leaves_everything_that_does_not_match() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    // One in flight, one queued behind it.
    let permission = tokio::spawn({
        let session = session.clone();
        async move {
            session
                .speak(
                    "permission question",
                    ResponseOrigin::Permission,
                    ResponseContext::new().with("authorizationId", "auth-one"),
                    None,
                )
                .await
        }
    });
    expect_frame(&mut peer, "response.create").await;
    settle().await;

    let cancelled = session
        .cancel_responses(|_context, origin| origin == ResponseOrigin::Agent)
        .await
        .expect("cancel");
    assert!(!cancelled);
    settle().await;
    assert!(
        !permission.is_finished(),
        "the permission response is untouched"
    );

    peer.send(json!({ "type": "response.created", "response": { "id": "r1" } }));
    finish_response(&peer, "r1", "completed");
    assert_eq!(
        permission
            .await
            .expect("task")
            .expect("call")
            .expect("outcome")
            .kind,
        OutcomeKind::Completed
    );
}

#[tokio::test]
async fn a_job_queued_before_a_cancel_is_dropped_rather_than_run() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    let first = tokio::spawn({
        let session = session.clone();
        async move {
            session
                .speak("first", ResponseOrigin::Agent, ResponseContext::new(), None)
                .await
        }
    });
    expect_frame(&mut peer, "response.create").await;

    let second = tokio::spawn({
        let session = session.clone();
        async move {
            session
                .speak(
                    "second",
                    ResponseOrigin::Agent,
                    ResponseContext::new(),
                    None,
                )
                .await
        }
    });
    // Let the second job read the queue generation and park in the queue.
    settle().await;

    session.cancel().await.expect("cancel");
    assert_eq!(
        first
            .await
            .expect("task")
            .expect("call")
            .expect("outcome")
            .kind,
        OutcomeKind::Cancelled
    );
    // The second never ran: its captured generation no longer matches.
    assert_eq!(second.await.expect("task").expect("call"), None);
    settle().await;
    assert!(
        !peer
            .drain_frames()
            .iter()
            .any(|frame| frame["response"]["instructions"] == json!("second"))
    );
}

// ── the busy-retry ladder ───────────────────────────────────────────────────

#[tokio::test(start_paused = true)]
async fn a_refused_response_is_replayed_with_its_original_correlation() {
    let Harness {
        session,
        mut peer,
        events,
        ..
    } = Harness::ga().await;

    let driver = tokio::spawn(async move {
        let refused = expect_frame(&mut peer, "response.create").await;
        provider_error(
            &peer,
            "Cannot create response while another response is in progress.",
        );
        let replayed = start_response(&mut peer, "resp-1").await;
        finish_response(&peer, "resp-1", "completed");
        (refused, replayed, peer)
    });
    let started = tokio::time::Instant::now();
    let outcome = session
        .speak(
            "the result is ready",
            ResponseOrigin::Announcement,
            ResponseContext::new().with("turnId", "turn-1"),
            None,
        )
        .await
        .expect("call")
        .expect("an outcome");
    let (refused, replayed, _peer) = driver.await.expect("driver");

    assert_eq!(outcome.kind, OutcomeKind::Completed);
    // Byte-identical apart from the envelope's own id: rebuilding the payload
    // would lose the correlation metadata the GA dialect answers with.
    assert_eq!(refused["response"], replayed["response"]);
    assert_ne!(refused["event_id"], replayed["event_id"]);
    assert!(
        refused["response"]["metadata"][RESPONSE_CORRELATION_KEY]
            .as_str()
            .is_some_and(|id| !id.is_empty())
    );
    // The first rung of the ladder, because nothing was active to wait on.
    assert!(started.elapsed() >= BUSY_RETRY_DELAYS[0]);

    // The refusal was handled internally and must not reach the user as an
    // error.
    let error = events
        .snapshot()
        .iter()
        .filter_map(Recorded::provider)
        .find(|event| event.kind() == "error")
        .cloned()
        .expect("the refusal was still forwarded");
    assert!(error.retried);
    assert_eq!(events.errors(), Vec::new());
}

#[tokio::test]
async fn a_provider_with_no_single_response_slot_surfaces_the_refusal() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    let driver = tokio::spawn(async move {
        expect_frame(&mut peer, "response.create").await;
        provider_error(
            &peer,
            "Cannot create response while another response is in progress.",
        );
        peer
    });
    let outcome = session
        .speak(
            "the result is ready",
            ResponseOrigin::Agent,
            ResponseContext::new(),
            None,
        )
        .await
        .expect("call")
        .expect("an outcome");
    let _peer = driver.await.expect("driver");
    assert_eq!(outcome.kind, OutcomeKind::Failed);
}

#[tokio::test(start_paused = true)]
async fn typed_input_that_races_the_end_of_a_turn_is_retried() {
    // `input_busy` retries a **model**-origin response on any provider, because
    // the collision is with the user's own speech rather than with a response
    // slot.
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    let driver = tokio::spawn(async move {
        acknowledge_item(&mut peer).await;
        expect_frame(&mut peer, "response.create").await;
        provider_error(&peer, "Cannot create response while user is speaking.");
        start_response(&mut peer, "resp-1").await;
        finish_response(&peer, "resp-1", "completed");
        peer
    });
    let outcome = session
        .send_user_text(
            "typed while speaking",
            ResponseContext::new().with("turnId", "text-turn"),
            None,
        )
        .await
        .expect("call")
        .expect("an outcome");
    let _peer = driver.await.expect("driver");
    assert_eq!(outcome.kind, OutcomeKind::Completed);
}

#[tokio::test]
async fn an_agent_origin_response_is_not_retried_on_input_busy() {
    // Only `model` origin retries: the user speaking does not make an
    // announcement worth forcing through.
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    let driver = tokio::spawn(async move {
        expect_frame(&mut peer, "response.create").await;
        provider_error(&peer, "Cannot create response while user is speaking.");
        peer
    });
    let outcome = session
        .speak(
            "announcement",
            ResponseOrigin::Announcement,
            ResponseContext::new(),
            None,
        )
        .await
        .expect("call")
        .expect("an outcome");
    let _peer = driver.await.expect("driver");
    assert_eq!(outcome.kind, OutcomeKind::Failed);
}

#[tokio::test(start_paused = true)]
async fn the_retry_ladder_is_bounded_and_climbs() {
    let Harness {
        session, mut peer, ..
    } = Harness::ga().await;
    let driver = tokio::spawn(async move {
        let mut rungs = Vec::new();
        let mut previous = tokio::time::Instant::now();
        for _ in 0..4 {
            expect_frame(&mut peer, "response.create").await;
            rungs.push(previous.elapsed());
            previous = tokio::time::Instant::now();
            provider_error(
                &peer,
                "Cannot create response while another response is in progress.",
            );
        }
        (rungs, peer)
    });
    let outcome = session
        .speak(
            "keeps being refused",
            ResponseOrigin::Agent,
            ResponseContext::new(),
            None,
        )
        .await
        .expect("call")
        .expect("an outcome");
    let (rungs, _peer) = driver.await.expect("driver");

    // Three replays, then the fourth refusal is surfaced.
    assert_eq!(outcome.kind, OutcomeKind::Failed);
    assert!(!outcome.is_completed());
    assert_eq!(rungs.len(), 4);
    assert_eq!(rungs[1], BUSY_RETRY_DELAYS[0]);
    assert_eq!(rungs[2], BUSY_RETRY_DELAYS[1]);
    assert_eq!(rungs[3], BUSY_RETRY_DELAYS[2]);
}

#[tokio::test]
async fn an_automatic_turn_is_never_mistaken_for_the_response_we_asked_for() {
    let Harness {
        session,
        mut peer,
        events,
        ..
    } = Harness::ga().await;

    let speaking = tokio::spawn({
        let session = session.clone();
        async move {
            session
                .speak(
                    "the task finished",
                    ResponseOrigin::Announcement,
                    ResponseContext::new().with("turnId", "turn-announcement"),
                    None,
                )
                .await
        }
    });
    let requested = expect_frame(&mut peer, "response.create").await;
    assert!(
        requested["response"]["metadata"][RESPONSE_CORRELATION_KEY]
            .as_str()
            .is_some_and(|id| !id.is_empty())
    );

    // A server-VAD turn on the same session, with no metadata of ours.
    peer.send(json!({
        "type": "response.created",
        "response": { "id": "resp-automatic" },
    }));
    let automatic = events.wait_for_kind("response.created").await;
    assert_eq!(automatic.origin, ResponseOrigin::Model);
    assert!(automatic.context.is_empty());
    settle().await;
    assert!(!speaking.is_finished(), "ours is still uncorrelated");

    // Ours then arrives, correlated by the echoed metadata.
    let request_id = requested["response"]["metadata"][RESPONSE_CORRELATION_KEY].clone();
    peer.send(json!({
        "type": "response.created",
        "response": { "id": "resp-ours", "metadata": { RESPONSE_CORRELATION_KEY: request_id } },
    }));
    let ours = events
        .wait_for("our own response.created", |entries| {
            entries
                .iter()
                .filter_map(Recorded::provider)
                .filter(|event| event.kind() == "response.created")
                .count()
                >= 2
        })
        .await;
    let ours = ours
        .iter()
        .filter_map(Recorded::provider)
        .filter(|event| event.kind() == "response.created")
        .nth(1)
        .cloned()
        .expect("the second");
    assert_eq!(ours.origin, ResponseOrigin::Announcement);
    assert_eq!(ours.context.turn_id(), Some("turn-announcement"));

    finish_response(&peer, "resp-ours", "completed");
    assert_eq!(
        speaking
            .await
            .expect("task")
            .expect("call")
            .expect("outcome")
            .response_id
            .as_deref(),
        Some("resp-ours")
    );
}

// ── the GA dialect end to end ───────────────────────────────────────────────

#[tokio::test]
async fn ga_text_events_reach_the_caller_under_the_shared_names() {
    let Harness {
        peer,
        events,
        session: _session,
        ..
    } = Harness::ga().await;
    peer.send(json!({ "type": "response.output_text.delta", "delta": "hello" }));
    let delta = events.wait_for_kind("response.text.delta").await;
    assert_eq!(delta.event["delta"], json!("hello"));
}

#[tokio::test]
async fn the_ga_dialect_namespaces_item_ids_by_type() {
    let Harness {
        session, mut peer, ..
    } = Harness::ga().await;
    let driver = tokio::spawn(async move {
        let output = acknowledge_item(&mut peer).await;
        (output, peer)
    });
    session
        .send_function_output(
            "call-1",
            json!({}),
            ResponseContext::new(),
            via_realtime::FunctionOutputOptions::without_response(),
        )
        .await
        .expect("call");
    let (output, mut peer) = driver.await.expect("driver");
    let id = output["item"]["id"].as_str().expect("id");
    assert!(id.starts_with("fco_"), "{id}");

    let driver = tokio::spawn(async move {
        let message = acknowledge_item(&mut peer).await;
        (message, peer)
    });
    session.append_user_context("hello").await.expect("call");
    let (message, _peer) = driver.await.expect("driver");
    let id = message["item"]["id"].as_str().expect("id");
    assert!(id.starts_with("msg_"), "{id}");
}

#[tokio::test]
async fn the_ga_dialect_rewrites_modalities_on_the_way_out() {
    let Harness {
        session, mut peer, ..
    } = Harness::ga().await;
    let driver = tokio::spawn(async move {
        acknowledge_item(&mut peer).await;
        let frame = complete_response(&mut peer, "r1").await;
        (frame, peer)
    });
    session
        .send_user_text(
            "hello",
            ResponseContext::new(),
            Some(vec!["text".into(), "audio".into()]),
        )
        .await
        .expect("call");
    let (frame, _peer) = driver.await.expect("driver");

    assert_eq!(
        frame["response"]["output_modalities"],
        json!(["text", "audio"])
    );
    assert!(frame["response"].get("modalities").is_none());
}

#[tokio::test]
async fn a_projection_can_be_answered_in_one_call() {
    let Harness {
        session, mut peer, ..
    } = Harness::beta().await;
    let projection = InputProjection::text(json!({
        "type": "message",
        "role": "user",
        "content": [{ "type": "input_text", "text": "look at this" }],
    }));
    let driver = tokio::spawn(async move {
        acknowledge_item(&mut peer).await;
        let response = complete_response(&mut peer, "r1").await;
        (response, peer)
    });
    let outcome = session
        .send_user_input(Some(projection), ResponseContext::new(), None)
        .await
        .expect("call")
        .expect("an outcome");
    let (response, _peer) = driver.await.expect("driver");

    assert_eq!(outcome.kind, OutcomeKind::Completed);
    assert!(
        response.get("response").is_none(),
        "no body, not an empty one"
    );
}

#[tokio::test]
async fn an_absent_projection_never_asks_for_an_answer() {
    let mut harness = Harness::beta().await;
    let outcome = harness
        .session
        .send_user_input(None, ResponseContext::new(), None)
        .await
        .expect("call")
        .expect("an outcome");
    assert_eq!(outcome.kind, OutcomeKind::Skipped);
    assert_eq!(outcome.phase, Some(OutcomePhase::Deduplicated));
    assert_eq!(harness.peer.drain_frames(), Vec::<Value>::new());
}

#[tokio::test]
async fn capabilities_are_snapshotted_when_the_session_opens() {
    let provider = TestProvider::new("beta")
        .with_capabilities(ProviderCapabilities {
            per_response_instructions: true,
            ..ProviderCapabilities::DEFAULT
        })
        .shared();
    let harness = Harness::open(Arc::clone(&provider), SessionOptions::default()).await;
    assert_eq!(
        harness.session.capabilities(),
        ProviderCapabilities {
            per_response_instructions: true,
            ..ProviderCapabilities::DEFAULT
        }
    );
}
