//! Each of the five flags, scripted both ways.
//!
//! `ProviderCapabilities` exists so the frontend never branches on a provider
//! name, which means every flag is a behaviour some provider really has. A mock
//! that only *declared* them would let the branch that reads them rot untested,
//! so four of the five change what the script server does and this file proves
//! each one from both sides.
//!
//! The fifth, `per_response_instructions`, has no session behaviour at all —
//! nothing in `via-realtime` branches on it — and that is asserted too, because
//! "declared only" is a claim that can go stale.

mod common;

use common::{no_context, open, transcript};
use pretty_assertions::assert_eq;
use serde_json::json;
use via_realtime::{
    CAPABILITY_FLAGS, ProviderCapabilities, RESPONSE_CORRELATION_KEY, ResponseOrigin,
};
use via_realtime_mock::{
    Emission, MockDialect, MockProviderSpec, Script, events, messages, script::turns,
};

/// A conversing script whose provider takes `capabilities`.
fn with(capabilities: ProviderCapabilities) -> Script {
    Script::conversation().map_provider(|spec| spec.with_capabilities(capabilities))
}

/// The baseline, with one flag flipped.
fn flipped(edit: impl FnOnce(&mut ProviderCapabilities)) -> ProviderCapabilities {
    let mut capabilities = ProviderCapabilities::DEFAULT;
    edit(&mut capabilities);
    capabilities
}

#[test]
fn the_five_flags_are_the_catalogued_ones_and_the_mock_can_set_each() {
    // The names come from `via-realtime`; nothing is retyped here.
    assert_eq!(CAPABILITY_FLAGS.len(), 5);
    let all_flipped = ProviderCapabilities {
        acknowledges_session_update: !ProviderCapabilities::DEFAULT.acknowledges_session_update,
        single_response_slot: !ProviderCapabilities::DEFAULT.single_response_slot,
        response_metadata_correlation: !ProviderCapabilities::DEFAULT.response_metadata_correlation,
        per_response_instructions: !ProviderCapabilities::DEFAULT.per_response_instructions,
        conversation_item_id_echo: !ProviderCapabilities::DEFAULT.conversation_item_id_echo,
    };
    let spec = MockProviderSpec::default().with_capabilities(all_flipped);
    assert_eq!(
        spec.capabilities.as_array(),
        ProviderCapabilities::DEFAULT.as_array().map(|flag| !flag)
    );
}

// ── 1. acknowledgesSessionUpdate ────────────────────────────────────────────

#[tokio::test]
async fn a_provider_that_acknowledges_becomes_ready_on_session_updated() {
    let (_session, log, handle) = open(with(ProviderCapabilities::DEFAULT)).await;
    assert!(
        transcript(&handle)
            .await
            .inbound_kinds()
            .contains(&"session.updated")
    );
    assert!(log.wait_for_kind("session.updated").await.is_some());
}

#[tokio::test]
async fn a_provider_that_does_not_acknowledge_is_ready_the_moment_the_update_is_written() {
    // No `session.updated` ever arrives, and yet the session opened — which is
    // the whole point of the flag. `huggingface/speech-to-speech` is this
    // provider.
    let (session, _log, handle) = open(with(flipped(|capabilities| {
        capabilities.acknowledges_session_update = false;
    })))
    .await;

    let transcript = transcript(&handle).await;
    assert!(
        !transcript.inbound_kinds().contains(&"session.updated"),
        "{:?}",
        transcript.inbound_kinds()
    );
    assert_eq!(transcript.count_of("session.update"), 1);
    // …and the session is genuinely usable.
    assert!(
        session
            .send_user_text("hello", no_context(), None)
            .await
            .expect("no transport failure")
            .is_some_and(|outcome| outcome.is_completed())
    );
}

// ── 2. singleResponseSlot ───────────────────────────────────────────────────

#[tokio::test(start_paused = true)]
async fn one_response_slot_makes_a_refusal_a_transparent_retry() {
    // The provider refuses the first `response.create` the way a service whose
    // one slot is taken by a server-VAD turn does, and answers the replay.
    let script = Script::conversation()
        .map_provider(|spec| {
            spec.with_capabilities(flipped(|capabilities| {
                capabilities.single_response_slot = true;
            }))
        })
        .turn(turns::refuse(messages::RESPONSE_SLOT_BUSY))
        .turn(turns::say("finally"));

    let (session, log, handle) = open(script).await;
    let started = tokio::time::Instant::now();
    let outcome = session
        .speak("please", ResponseOrigin::Agent, no_context(), None)
        .await
        .expect("no transport failure")
        .expect("an outcome");

    assert!(outcome.is_completed(), "{outcome:?}");
    assert!(log.wait_for_turns(1).await);
    assert_eq!(log.spoken_text(), "finally");
    // The first rung of the catalogued ladder, on the virtual clock.
    assert!(
        started.elapsed() >= via_realtime::BUSY_RETRY_DELAYS[0],
        "{:?}",
        started.elapsed()
    );

    let refusal = log
        .wait_for_kind("error")
        .await
        .expect("the busy refusal reached the Gateway");
    assert!(
        refusal.retried,
        "a transparently retried refusal is marked so the Gateway hides it"
    );
    assert_eq!(
        refusal.event["error"]["message"],
        json!(messages::RESPONSE_SLOT_BUSY)
    );
    // Two `response.create` frames: the refused one and its replay.
    assert_eq!(transcript(&handle).await.count_of("response.create"), 2);
}

#[tokio::test]
async fn a_queueing_provider_surfaces_the_same_refusal_instead_of_replaying_it() {
    // Byte-for-byte the same refusal, one flag the other way: it is a failure
    // rather than a retry, because a provider that queues has no slot to be
    // busy and the error means something else.
    let script = Script::conversation()
        .map_provider(|spec| {
            spec.with_capabilities(flipped(|capabilities| {
                capabilities.single_response_slot = false;
            }))
        })
        .turn(turns::refuse(messages::RESPONSE_SLOT_BUSY))
        .turn(turns::say("never reached"));

    let (session, log, handle) = open(script).await;
    let outcome = session
        .speak("please", ResponseOrigin::Agent, no_context(), None)
        .await
        .expect("no transport failure")
        .expect("an outcome");

    assert!(outcome.is_failed(), "{outcome:?}");
    // A refused turn never reaches `response.done`, so the refusal itself is
    // what the log is waited on.
    assert!(log.wait_for_kind("error").await.is_some());
    assert_eq!(log.spoken_text(), "");
    assert_eq!(transcript(&handle).await.count_of("response.create"), 1);
    let refusal = log
        .provider_events()
        .into_iter()
        .find(|event| event.kind() == "error")
        .expect("the refusal reached the Gateway");
    assert!(
        !refusal.retried,
        "nothing was retried, so nothing is hidden"
    );
}

#[tokio::test(start_paused = true)]
async fn an_input_busy_refusal_is_retried_for_a_model_turn_whatever_the_slot_flag_says() {
    // `input_busy` is the other half of the ladder and it is keyed on the
    // response's *origin*, not on a capability: the user was still speaking, so
    // the model's own turn is replayed.
    let script = Script::conversation()
        .map_provider(|spec| {
            spec.with_capabilities(flipped(|capabilities| {
                capabilities.single_response_slot = false;
            }))
        })
        .turn(turns::refuse(messages::INPUT_BUSY))
        .turn(turns::say("after you"));

    let (session, log, handle) = open(script).await;
    let outcome = session
        .send_user_text("go on then", no_context(), None)
        .await
        .expect("no transport failure")
        .expect("an outcome");
    assert!(outcome.is_completed(), "{outcome:?}");
    assert!(log.wait_for_turns(1).await);
    assert_eq!(log.spoken_text(), "after you");
    assert_eq!(transcript(&handle).await.count_of("response.create"), 2);
}

#[tokio::test(start_paused = true)]
async fn the_ladder_gives_up_after_the_catalogued_number_of_retries() {
    let script = Script::conversation()
        .map_provider(|spec| {
            spec.with_capabilities(flipped(|capabilities| {
                capabilities.single_response_slot = true;
            }))
        })
        // Refuses forever.
        .on_always(
            via_realtime_mock::Trigger::ResponseCreate,
            turns::refuse(messages::RESPONSE_SLOT_BUSY),
        );

    let (session, _log, handle) = open(script).await;
    let outcome = session
        .speak("please", ResponseOrigin::Agent, no_context(), None)
        .await
        .expect("no transport failure")
        .expect("an outcome");
    assert!(outcome.is_failed(), "{outcome:?}");
    // The original plus `MAX_BUSY_RETRIES` replays, and not one more.
    assert_eq!(
        transcript(&handle).await.count_of("response.create"),
        1 + via_realtime::MAX_BUSY_RETRIES as usize
    );
}

// ── 3. responseMetadataCorrelation ──────────────────────────────────────────

#[tokio::test]
async fn a_correlating_provider_echoes_the_request_id_back() {
    let (session, log, handle) = open(
        Script::conversation()
            .map_provider(|_| MockProviderSpec::speech_to_speech_shaped())
            .turn(turns::say("correlated")),
    )
    .await;

    session
        .speak(
            "hello",
            ResponseOrigin::Agent,
            via_realtime::ResponseContext::new().with("turnId", "voice-1"),
            None,
        )
        .await
        .expect("no transport failure")
        .expect("an outcome");

    // The session stamped a request id on the way out …
    let transcript = transcript(&handle).await;
    let sent = transcript.response_creates()[0].expect("a body")["metadata"]
        [RESPONSE_CORRELATION_KEY]
        .as_str()
        .expect("a request id")
        .to_owned();
    assert!(!sent.is_empty());

    // … the mock echoed it back on `response.created` …
    let created = log
        .wait_for_kind("response.created")
        .await
        .expect("the response starts");
    assert_eq!(
        created.event["response"]["metadata"][RESPONSE_CORRELATION_KEY],
        json!(sent)
    );
    // … and the session correlated the event to the caller's context.
    assert_eq!(created.origin, ResponseOrigin::Agent);
    assert_eq!(created.context.turn_id(), Some("voice-1"));
}

#[tokio::test]
async fn a_non_correlating_provider_falls_back_to_fifo_and_still_correlates() {
    let (session, log, handle) = open(
        Script::conversation()
            .map_provider(|spec| {
                spec.with_capabilities(flipped(|capabilities| {
                    capabilities.response_metadata_correlation = false;
                }))
            })
            .turn(turns::say("fifo")),
    )
    .await;

    session
        .speak(
            "hello",
            ResponseOrigin::Agent,
            via_realtime::ResponseContext::new().with("turnId", "voice-2"),
            None,
        )
        .await
        .expect("no transport failure")
        .expect("an outcome");

    // Nothing was stamped, and nothing was echoed …
    let transcript = transcript(&handle).await;
    assert!(
        transcript.response_creates()[0]
            .expect("a body")
            .get("metadata")
            .is_none()
    );
    let created = log
        .wait_for_kind("response.created")
        .await
        .expect("the response starts");
    assert!(created.event["response"].get("metadata").is_none());
    // … and the first `response.created` after the request is still its answer.
    assert_eq!(created.origin, ResponseOrigin::Agent);
    assert_eq!(created.context.turn_id(), Some("voice-2"));
}

// ── 4. conversationItemIdEcho ───────────────────────────────────────────────

#[tokio::test]
async fn an_echoing_provider_confirms_the_id_the_client_chose() {
    let (session, _log, handle) = open(with(ProviderCapabilities::DEFAULT)).await;
    session
        .append_user_context("some context")
        .await
        .expect("no transport failure");

    let transcript = transcript(&handle).await;
    let sent = transcript.items()[0]["id"]
        .as_str()
        .expect("an id")
        .to_owned();
    let confirmed = transcript
        .inbound()
        .iter()
        .find(|event| event["type"] == json!("conversation.item.created"))
        .expect("a receipt");
    assert_eq!(confirmed["item"]["id"], json!(sent));
}

#[tokio::test]
async fn a_replacing_provider_is_still_matched_through_the_single_waiter() {
    let (session, _log, handle) = open(with(flipped(|capabilities| {
        capabilities.conversation_item_id_echo = false;
    })))
    .await;
    // The item is acknowledged with an id the client never sent, and the
    // session still resolves its waiter — the fallback the flag exists for.
    assert!(
        session
            .append_user_context("some context")
            .await
            .expect("no transport failure")
    );

    let transcript = transcript(&handle).await;
    let sent = transcript.items()[0]["id"].as_str().expect("an id");
    let confirmed = transcript
        .inbound()
        .iter()
        .find(|event| event["type"] == json!("conversation.item.created"))
        .expect("a receipt");
    assert_ne!(confirmed["item"]["id"], json!(sent));
    assert!(
        confirmed["item"]["id"]
            .as_str()
            .is_some_and(|id| id.starts_with(via_realtime_mock::ITEM_ID_PREFIX))
    );
}

// ── 5. perResponseInstructions ──────────────────────────────────────────────

#[tokio::test]
async fn per_response_instructions_is_declared_and_changes_nothing_in_the_session() {
    // Both settings produce the same frames, which is what "declared only"
    // means. It is asserted rather than assumed because the day something in
    // `via-realtime` starts branching on it, this test is the one that notices.
    async fn frames(per_response: bool) -> Vec<String> {
        let (session, _log, handle) = open(with(flipped(|capabilities| {
            capabilities.per_response_instructions = per_response;
        })))
        .await;
        session
            .speak("hello", ResponseOrigin::Agent, no_context(), None)
            .await
            .expect("no transport failure");
        transcript(&handle)
            .await
            .outbound_kinds()
            .iter()
            .map(|kind| (*kind).to_owned())
            .collect()
    }
    assert_eq!(frames(true).await, frames(false).await);
}

// ── all four at once ────────────────────────────────────────────────────────

#[tokio::test]
async fn the_speech_to_speech_capability_set_opens_and_converses() {
    let spec = MockProviderSpec::speech_to_speech_shaped();
    assert_eq!(
        spec.capabilities.as_array(),
        [false, true, true, true, true]
    );
    assert_eq!(spec.dialect, MockDialect::Ga);

    let (session, log, handle) = open(
        Script::conversation()
            .with_provider(spec)
            .turn(turns::say("ga")),
    )
    .await;
    assert!(
        session
            .send_user_text("hello", no_context(), None)
            .await
            .expect("no transport failure")
            .is_some_and(|outcome| outcome.is_completed())
    );
    assert!(log.wait_for_turns(1).await);
    assert_eq!(log.spoken_text(), "ga");
    // The GA dialect's own session shape.
    assert!(
        transcript(&handle).await.session_updates()[0]
            .get("output_modalities")
            .is_some()
    );
}

#[tokio::test]
async fn the_ga_dialect_renames_its_text_events_on_the_way_in() {
    // `response.output_text.*` must reach the Gateway as `response.text.*`, or a
    // GA provider looks mute.
    let (session, log, _handle) = open(
        Script::conversation()
            .map_provider(|_| MockProviderSpec::speech_to_speech_shaped())
            .turn(vec![
                Emission::now(events::response_created()),
                Emission::now(json!({ "type": "response.output_text.delta", "delta": "ga text" })),
                Emission::now(events::response_done(via_realtime_mock::COMPLETED)),
            ]),
    )
    .await;
    session
        .send_user_text("hello", no_context(), None)
        .await
        .expect("no transport failure");
    assert!(log.wait_for_turns(1).await);
    assert_eq!(log.spoken_text(), "ga text");
    assert!(
        log.provider_kinds()
            .iter()
            .all(|kind| kind != "response.output_text.delta")
    );
}
