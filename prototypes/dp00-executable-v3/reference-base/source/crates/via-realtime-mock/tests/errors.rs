//! Every way a scripted provider can say no.
//!
//! The point of a mock that can fail is that the failures are the expensive part
//! to reach with a real service: a session slot that is already taken, an
//! expired credential, a model that stops mid-sentence, a provider that accepts
//! the socket and then says nothing at all. Each one is one line of script here.

mod common;

use std::time::Duration;

use common::{no_context, open, open_with, transcript};
use pretty_assertions::assert_eq;
use serde_json::json;
use via_i18n::Locale;
use via_realtime::{
    DIAGNOSTIC_RESPONSE_TIMEOUT, ErrorClass, OutcomeKind, OutcomePhase, RealtimeError,
    ResponseOrigin,
};
use via_realtime_mock::{
    Emission, MockError, MockProviderSpec, MockRealtime, Script, Trigger, events, messages,
    script::turns,
};

// ── refused before the transport ────────────────────────────────────────────

#[tokio::test]
async fn an_unconfigured_provider_is_refused_before_anything_is_written() {
    let (handle, opened) =
        MockRealtime::new(Script::conversation().map_provider(|spec| spec.with_configured(false)))
            .try_open()
            .await;
    let error = opened.map(|_| ()).expect_err("refused");
    assert_eq!(error.code(), "VIA_REALTIME_NOT_CONFIGURED");
    // The provider's own sentence, in the session's locale.
    assert_eq!(
        error.message(Locale::En),
        via_realtime_mock::MockProviderSpec::default()
            .build()
            .expect("valid")
            .missing_configuration_message(Locale::En)
    );
    // Nothing reached the transport — the mock never even saw a `session.update`.
    assert_eq!(transcript(&handle).await.count_of("session.update"), 0);
}

#[tokio::test]
async fn preflight_refuses_ahead_of_the_configuration_check() {
    // The ordering is contract: `preflight()`, then `is_configured()`, then the
    // socket. A provider that is *both* unsupported and unconfigured reports the
    // unsupported model.
    let (handle, opened) = MockRealtime::new(
        Script::conversation().map_provider(|spec| spec.unsupported().with_configured(false)),
    )
    .try_open()
    .await;
    let error = opened.map(|_| ()).expect_err("refused");
    assert_eq!(error.code(), "VIA_REALTIME_MODEL_UNSUPPORTED");
    assert_eq!(transcript(&handle).await.outbound().len(), 0);
}

#[tokio::test]
async fn a_provider_the_registry_would_refuse_never_opens() {
    let error =
        MockRealtime::new(Script::conversation().map_provider(|spec| spec.with_key("NOT A KEY")))
            .open()
            .await
            .map(|_| ())
            .expect_err("refused");
    assert_eq!(error.code(), "VIA_REALTIME_PROVIDER_KEY_INVALID");
}

// ── refused during the handshake ────────────────────────────────────────────

#[tokio::test]
async fn an_error_before_the_session_is_ready_rejects_the_connection_at_once() {
    // A speech-to-speech session slot that is still occupied answers this way,
    // and answering fast is the point: the caller's backoff can retry rather
    // than waiting out the 25 s connect budget.
    let script = Script::conversation()
        .with_on_connect([Emission::now(events::error(messages::CAPACITY_BUSY))]);
    let error = MockRealtime::new(script)
        .open()
        .await
        .map(|_| ())
        .expect_err("refused");
    assert_eq!(error.code(), "VIA_REALTIME_PROVIDER_REFUSED");
    assert!(
        error.message(Locale::En).contains("session slots"),
        "{}",
        error.message(Locale::En)
    );
}

#[tokio::test]
async fn the_composed_error_message_joins_code_type_and_message() {
    // `realtimeEventErrorMessage` joins the deduped non-empty values of
    // `error.code`, `error.type` and `error.message` with `": "`.
    let script = Script::conversation().with_on_connect([Emission::now(events::error_detailed(
        "AllocationQuota.FreeTierOnly",
        "insufficient_quota",
        "The free tier of the model has been exhausted.",
    ))]);
    let error = MockRealtime::new(script)
        .open()
        .await
        .map(|_| ())
        .expect_err("refused");
    assert_eq!(
        error.message(Locale::En),
        "AllocationQuota.FreeTierOnly: insufficient_quota: The free tier of the model has been \
         exhausted."
    );
}

#[tokio::test(start_paused = true)]
async fn a_provider_that_says_nothing_times_the_connect_budget_out() {
    let mock = MockRealtime::new(Script::conversation().silent_on_connect())
        .with_connect_timeout(Duration::from_millis(200));
    let started = tokio::time::Instant::now();
    let error = mock.open().await.map(|_| ()).expect_err("timed out");
    assert_eq!(error.code(), "VIA_REALTIME_CONNECT_TIMEOUT");
    assert!(started.elapsed() >= Duration::from_millis(200));
    // The provider's own sentence, localized.
    for locale in [Locale::En, Locale::Zh, Locale::Ko] {
        assert!(
            !MockProviderSpec::default()
                .build()
                .expect("valid")
                .connect_timeout_message(locale)
                .is_empty()
        );
    }
}

#[tokio::test(start_paused = true)]
async fn a_provider_that_opens_and_never_acknowledges_also_times_out() {
    // The budget covers the whole handshake, not just the first frame: the mock
    // says `session.created` and then goes quiet.
    let script = Script::conversation()
        .with_on_connect([Emission::now(events::session_created())])
        .on_always(Trigger::SessionUpdate, []);
    let (handle, opened) = MockRealtime::new(script)
        .with_connect_timeout(Duration::from_millis(200))
        .try_open()
        .await;
    assert_eq!(
        opened.map(|_| ()).expect_err("timed out").code(),
        "VIA_REALTIME_CONNECT_TIMEOUT"
    );
    // It got as far as configuring itself, which is what distinguishes this
    // from the silent case.
    assert_eq!(transcript(&handle).await.count_of("session.update"), 1);
}

// ── refused mid-session ─────────────────────────────────────────────────────

#[tokio::test]
async fn a_failing_response_status_is_a_failed_outcome_carrying_the_status() {
    for status in via_realtime::FAILED_RESPONSE_STATUSES {
        let (session, _log, _handle) = open(Script::conversation().turn(turns::fail(status))).await;
        let outcome = session
            .send_user_text("go", no_context(), None)
            .await
            .expect("no transport failure")
            .expect("an outcome");
        assert_eq!(outcome.kind, OutcomeKind::Failed, "{status}");
        assert_eq!(outcome.status.as_deref(), Some(status));
    }
}

#[tokio::test]
async fn a_status_the_catalogue_does_not_know_is_treated_as_success() {
    // Deliberate, and upstream's: only the three catalogued statuses are
    // failures.
    let (session, _log, _handle) = open(Script::conversation().turn(turns::fail("weird"))).await;
    let outcome = session
        .send_user_text("go", no_context(), None)
        .await
        .expect("no transport failure")
        .expect("an outcome");
    assert!(outcome.is_completed(), "{outcome:?}");
}

#[tokio::test]
async fn a_rejected_conversation_item_fails_the_turn_at_the_input_phase() {
    // An `error` while an item receipt is outstanding belongs to the item, and
    // must not also settle a response.
    let script = Script::conversation().on(
        Trigger::ItemCreate,
        [Emission::now(events::error("Invalid item content"))],
    );
    let (session, log, handle) = open(script).await;
    let outcome = session
        .send_user_text("go", no_context(), None)
        .await
        .expect("no transport failure")
        .expect("an outcome");
    assert_eq!(outcome.kind, OutcomeKind::Failed, "{outcome:?}");
    assert_eq!(outcome.phase, Some(OutcomePhase::Input), "{outcome:?}");
    assert_eq!(outcome.error.as_deref(), Some("Invalid item content"));
    // The item was never confirmed, so no response was asked for.
    assert_eq!(transcript(&handle).await.count_of("response.create"), 0);
    // …and the refusal is reported once, as the outcome, not twice. Waiting for
    // the provider event first is what makes this an *absence* assertion rather
    // than a race the drainer wins.
    assert!(log.wait_for_kind("error").await.is_some());
    assert!(
        log.errors()
            .iter()
            .all(|error| !matches!(error, RealtimeError::ItemRejected { .. })),
        "{:?}",
        log.errors()
    );
}

#[tokio::test(start_paused = true)]
async fn an_item_that_is_never_acknowledged_times_out() {
    let script = Script::conversation().on_always(Trigger::ItemCreate, []);
    let mock = MockRealtime::new(script).with_response_start_timeout(Duration::from_millis(100));
    let (session, _log, _handle) = open_with(mock).await;
    let confirmed = session
        .append_user_context("some context")
        .await
        .map(|_| ())
        .expect_err("never confirmed");
    assert_eq!(confirmed.code(), "VIA_REALTIME_ITEM_UNCONFIRMED");
}

#[tokio::test(start_paused = true)]
async fn a_response_that_never_starts_times_out_at_the_start_phase() {
    let mock = MockRealtime::new(Script::conversation().turn(turns::never_start()))
        .with_response_start_timeout(Duration::from_millis(100));
    let (session, _log, _handle) = open_with(mock).await;
    let started = tokio::time::Instant::now();
    let outcome = session
        .speak("hello", ResponseOrigin::Agent, no_context(), None)
        .await
        .expect("no transport failure")
        .expect("an outcome");
    assert_eq!(outcome.kind, OutcomeKind::TimedOut, "{outcome:?}");
    assert_eq!(outcome.phase, Some(OutcomePhase::Start), "{outcome:?}");
    assert!(started.elapsed() >= Duration::from_millis(100));
}

#[tokio::test(start_paused = true)]
async fn a_response_that_stops_producing_output_is_cancelled_and_diagnosed() {
    let mock = MockRealtime::new(Script::conversation().turn(turns::stall()))
        .with_response_inactivity_timeout(Duration::from_millis(300));
    let (session, log, handle) = open_with(mock).await;
    let outcome = session
        .speak("tell me a story", ResponseOrigin::Agent, no_context(), None)
        .await
        .expect("no transport failure")
        .expect("an outcome");

    assert_eq!(outcome.kind, OutcomeKind::TimedOut, "{outcome:?}");
    assert_eq!(outcome.phase, Some(OutcomePhase::Inactivity), "{outcome:?}");
    assert_eq!(outcome.response_id.as_deref(), Some("resp_1"));

    let diagnostic = log
        .diagnostics()
        .into_iter()
        .find(|diagnostic| diagnostic.event == DIAGNOSTIC_RESPONSE_TIMEOUT)
        .expect("a diagnostic");
    assert_eq!(diagnostic.provider, via_realtime_mock::MOCK_PROVIDER_KEY);
    assert_eq!(diagnostic.response_id, "resp_1");
    assert_eq!(diagnostic.phase, "inactivity");
    assert!(diagnostic.inactivity_ms >= 300, "{diagnostic:?}");

    // The session asked the provider to stop.
    assert_eq!(transcript(&handle).await.count_of("response.cancel"), 1);
}

#[tokio::test(start_paused = true)]
async fn output_reopens_the_inactivity_window_rather_than_extending_a_fixed_budget() {
    // A sliding window, not a duration cap: long speech stays valid as long as
    // the provider keeps streaming. Five deltas 200 ms apart outlive a 300 ms
    // window.
    let mut emissions = vec![Emission::now(events::response_created())];
    for index in 0..5 {
        emissions.push(Emission::after(
            Duration::from_millis(200),
            events::audio_transcript_delta(&format!("{index} ")),
        ));
    }
    emissions.push(Emission::after(
        Duration::from_millis(200),
        events::response_done(via_realtime_mock::COMPLETED),
    ));

    let mock = MockRealtime::new(Script::conversation().turn(emissions))
        .with_response_inactivity_timeout(Duration::from_millis(300));
    let (session, log, _handle) = open_with(mock).await;
    let outcome = session
        .speak("a long one", ResponseOrigin::Agent, no_context(), None)
        .await
        .expect("no transport failure")
        .expect("an outcome");
    assert!(outcome.is_completed(), "{outcome:?}");
    assert!(log.wait_for_turns(1).await);
    assert_eq!(log.spoken_text(), "0 1 2 3 4 ");
    assert!(log.diagnostics().is_empty(), "nothing timed out");
}

// ── classification ──────────────────────────────────────────────────────────

#[tokio::test]
async fn every_catalogued_class_is_reachable_from_a_scripted_message() {
    let provider = MockProviderSpec::default().build().expect("valid");
    for (message, class) in [
        (messages::INACTIVITY, ErrorClass::Inactivity),
        (messages::INPUT_BUSY, ErrorClass::InputBusy),
        (messages::NO_ACTIVE_RESPONSE, ErrorClass::NoActiveResponse),
        (messages::FATAL, ErrorClass::Fatal),
        (messages::CAPACITY_BUSY, ErrorClass::CapacityBusy),
        (messages::RESPONSE_SLOT_BUSY, ErrorClass::ResponseSlotBusy),
        ("something nobody has seen before", ErrorClass::Other),
    ] {
        assert_eq!(provider.classify_error(message), class, "{message}");
    }
}

#[tokio::test]
async fn a_script_can_reclassify_its_own_provider_error() {
    let provider = MockProviderSpec::default()
        .classifying("tea break", ErrorClass::Fatal)
        .build()
        .expect("valid");
    assert_eq!(
        provider.classify_error("The service is on a tea break."),
        ErrorClass::Fatal
    );
    // A `fatal` classification is one the Gateway suppresses and blocks on.
    assert!(ErrorClass::Fatal.is_suppressed());
}

// ── the mock's own refusals ─────────────────────────────────────────────────

#[tokio::test]
async fn a_handle_whose_server_has_stopped_says_so_rather_than_hanging() {
    let mock = MockRealtime::new(Script::conversation());
    let (session, events, handle) = mock.open().await.expect("opens").into_parts();
    drop(session);
    drop(events);
    // Draining the handle's last clone stops the server.
    let orphan = handle.clone();
    drop(handle);
    // The transport is gone and only this handle is left, so the server is on
    // its way out; either answer is correct, and neither hangs.
    let answer = orphan.transcript().await;
    assert!(
        answer.is_ok() || answer == Err(MockError::ServerStopped),
        "{answer:?}"
    );
}

#[test]
fn a_fixture_that_is_not_a_script_names_the_file() {
    let error = Script::from_json_file("/nonexistent/via-mock-script.json").expect_err("refused");
    assert_eq!(error.code(), "VIA_MOCK_FIXTURE_UNREADABLE");
    assert!(
        error.to_string().contains("via-mock-script.json"),
        "{error}"
    );
}

#[test]
fn a_script_with_an_unknown_trigger_is_refused_rather_than_ignored() {
    let error =
        Script::from_json_str(r#"{ "steps": [{ "on": "somethingElse" }] }"#).expect_err("refused");
    assert_eq!(error.code(), "VIA_MOCK_SCRIPT_INVALID");
}

#[test]
fn a_step_with_no_emissions_is_legal_and_means_silence() {
    let script =
        Script::from_json_str(r#"{ "steps": [{ "on": "responseCreate" }] }"#).expect("parses");
    assert!(script.steps()[0].emit.is_empty());
}

#[tokio::test]
async fn an_event_the_script_wrote_by_hand_reaches_the_session_unchanged() {
    let (_session, log, handle) = open(Script::conversation()).await;
    handle
        .emit(json!({ "type": "custom.event", "payload": { "n": 1 } }))
        .await
        .expect("emitted");
    let event = log.wait_for_kind("custom.event").await.expect("arrives");
    assert_eq!(event.event["payload"], json!({ "n": 1 }));
    // An event outside the response-activity table is never given a response
    // id, however busy the session is.
    assert!(event.event.get("response_id").is_none(), "{event:?}");
}
