//! A script read off disk drives the same session a struct does.
//!
//! This is the half of the crate that exists for *other* crates: a test in
//! `via-voice` or `via-app` keeps its script in a `.json` file beside itself
//! rather than in a Rust literal, so the fixture can be diffed, reviewed and
//! reused without recompiling anything.
//!
//! Every fixture under `tests/fixtures/` is loaded and *run* here, not merely
//! parsed. A fixture that parses and then does nothing is the failure mode this
//! file exists to prevent.

mod common;

use std::path::{Path, PathBuf};

use common::{no_context, transcript};
use pretty_assertions::assert_eq;
use serde_json::json;
use via_protocol::SessionMode;
use via_realtime::{FunctionOutputOptions, RESPONSE_CORRELATION_KEY};
use via_realtime_mock::{MockDialect, MockRealtime, Script, ScriptKind};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[tokio::test]
async fn a_fixture_drives_a_whole_tool_round_trip() {
    let mock = MockRealtime::from_json_file(fixture("tool_round_trip.json")).expect("loads");
    assert_eq!(mock.options().mode, SessionMode::Agent);
    let (session, log, handle) = common::open_with(mock).await;

    session
        .send_user_text("summarise the log", no_context(), None)
        .await
        .expect("no transport failure")
        .expect("an outcome");
    let call = log
        .wait_for_kind("response.function_call_arguments.done")
        .await
        .expect("the tool call");
    assert_eq!(call.event["call_id"], json!("call_fixture_1"));
    assert_eq!(call.event["name"], json!("spawn_thinking"));
    assert_eq!(
        call.event["arguments"],
        json!(r#"{"objective":"summarise the log"}"#)
    );

    let outcome = session
        .send_function_output(
            "call_fixture_1",
            json!({ "status": "queued" }),
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
        transcript(&handle).await.function_outputs()[0].output,
        json!({ "status": "queued" })
    );
}

#[tokio::test]
async fn a_dictation_fixture_needs_only_its_kind_to_get_a_provider_with_no_model() {
    let mock = MockRealtime::from_json_file(fixture("dictation.json")).expect("loads");
    assert_eq!(mock.script().kind, ScriptKind::Dictation);
    assert_eq!(mock.options().mode, SessionMode::Dictation);
    // The fixture names no provider at all, and still gets one that mounts no
    // model — the reason `kind` is a script kind rather than a session mode.
    assert_eq!(mock.script().provider.model, None);
    assert_eq!(mock.script().provider.voice, None);

    let (session, log, _handle) = common::open_with(mock).await;
    session.append_audio("AAAA").await.expect("appended");
    session.commit_audio().await.expect("committed");
    let completed = log
        .wait_for_kind("conversation.item.input_audio_transcription.completed")
        .await
        .expect("a transcript");
    assert_eq!(completed.event["transcript"], json!("open the door"));
}

#[tokio::test]
async fn a_fixture_can_declare_the_whole_speech_to_speech_capability_set() {
    let mock =
        MockRealtime::from_json_file(fixture("speech_to_speech_shaped.json")).expect("loads");
    let spec = &mock.script().provider;
    assert_eq!(spec.dialect, MockDialect::Ga);
    assert_eq!(
        spec.capabilities.as_array(),
        [false, true, true, true, false]
    );
    assert_eq!(spec.response_start_timeout_ms, Some(60_000));

    let (session, log, handle) = common::open_with(mock).await;
    session
        .speak(
            "hello",
            via_realtime::ResponseOrigin::Agent,
            no_context(),
            None,
        )
        .await
        .expect("no transport failure")
        .expect("an outcome");

    // The GA text spelling was renamed on the way in …
    assert!(log.wait_for_turns(1).await);
    assert_eq!(log.spoken_text(), "ga hello");
    // … the session negotiated with the GA key …
    assert!(
        transcript(&handle).await.session_updates()[0]
            .get("output_modalities")
            .is_some()
    );
    // … and the correlation id round-tripped.
    let created = log
        .wait_for_kind("response.created")
        .await
        .expect("the response starts");
    assert!(
        created.event["response"]["metadata"][RESPONSE_CORRELATION_KEY].is_string(),
        "{created:?}"
    );
}

#[tokio::test]
async fn a_fixture_can_refuse_the_handshake() {
    let mock = MockRealtime::from_json_file(fixture("refuses_the_handshake.json")).expect("loads");
    let error = mock.open().await.map(|_| ()).expect_err("refused");
    assert_eq!(error.code(), "VIA_REALTIME_PROVIDER_REFUSED");
    let message = error.message(via_i18n::Locale::En);
    assert!(
        message.starts_with("SessionLimit: server_error: "),
        "{message}"
    );
}

#[test]
fn a_fixture_that_is_not_a_script_names_the_path_and_the_field() {
    let path = fixture("not_a_script.json");
    let error = Script::from_json_file(&path).expect_err("refused");
    assert_eq!(error.code(), "VIA_MOCK_SCRIPT_INVALID");
    let message = error.to_string();
    assert!(message.contains("not_a_script.json"), "{message}");
    assert!(message.contains("somethingNobodyDeclared"), "{message}");
}

#[test]
fn a_missing_fixture_is_told_apart_from_an_invalid_one() {
    let error = Script::from_json_file(fixture("no_such_fixture.json")).expect_err("refused");
    assert_eq!(error.code(), "VIA_MOCK_FIXTURE_UNREADABLE");
    assert!(
        error.to_string().contains("no_such_fixture.json"),
        "{error}"
    );
}

#[test]
fn every_fixture_in_the_directory_parses() {
    // A fixture that stopped parsing after a schema change should fail here
    // rather than in whichever test happens to load it.
    let directory = fixture(".");
    let mut seen = 0;
    for entry in std::fs::read_dir(&directory).expect("the fixture directory exists") {
        let path = entry.expect("a readable entry").path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        seen += 1;
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let parsed = Script::from_json_file(&path);
        if name == "not_a_script.json" {
            assert!(parsed.is_err(), "{name} is the deliberately broken one");
            continue;
        }
        assert!(parsed.is_ok(), "{name} no longer parses: {parsed:?}");
    }
    assert!(seen >= 5, "only {seen} fixtures found in {directory:?}");
}

#[test]
fn a_script_written_in_rust_round_trips_through_the_fixture_form() {
    // The two doors are the same door: whatever a builder produces can be
    // written out and read back unchanged, which is what lets a test capture a
    // script it built and keep it as a fixture.
    for script in [
        Script::conversation(),
        Script::dictation(),
        Script::conversation()
            .map_provider(|_| via_realtime_mock::MockProviderSpec::speech_to_speech_shaped())
            .turn(via_realtime_mock::turns::say("hello"))
            .on_always(
                via_realtime_mock::Trigger::AudioCommit,
                [via_realtime_mock::Emission::after(
                    std::time::Duration::from_millis(25),
                    via_realtime_mock::events::input_transcript_completed("hi"),
                )],
            )
            .silent_on_connect(),
    ] {
        let text = script.to_json_string().expect("serialize");
        assert_eq!(Script::from_json_str(&text).expect("parse"), script);
    }
}

#[tokio::test]
async fn a_fixture_and_the_equivalent_builder_produce_the_same_session() {
    let from_file = MockRealtime::from_json_file(fixture("dictation.json")).expect("loads");
    let text = std::fs::read_to_string(fixture("dictation.json")).expect("readable");
    let from_text = MockRealtime::from_json_str(&text).expect("parses");
    assert_eq!(from_file.script(), from_text.script());
    assert_eq!(from_file.options().mode, from_text.options().mode);
}
