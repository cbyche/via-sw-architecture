//! The turn ladder: what the model is shown, how many chances it gets, and
//! what happens when it answers with nothing.
//!
//! Ports `server/test/coordinator.test.mjs` and the four coordinator-recovery
//! cases in `server/test/acp-backend-adapter.test.mjs:327-490`.

mod common;

use std::sync::Arc;

use chrono::{TimeZone, Utc};
use common::{BACKEND_ID, BACKEND_LABEL, COORDINATOR_SESSION_ID, Reply, RoutingHarness};
use pretty_assertions::assert_eq;
use serde_json::json;
use via_conversation::markdown_store::MemoryDocument;
use via_coordinator::{
    CoordinationRequest, Coordinator, CoordinatorError, CoordinatorProfile, Delivery,
    EnvelopeAttachment, TrustedBackendEvent, TurnOptions,
};
use via_downstream::PromptAttachment;
use via_i18n::{Locale, keys, t};

const WORK_ID: &str = "work-one";
const OWNER: &str = "owner-one";

fn completed(speech: &str) -> String {
    json!({
        "work_id": WORK_ID,
        "state": "completed",
        "mode": "respond",
        "presentation": { "speech": speech, "inline": null },
    })
    .to_string()
}

fn coordinator(harness: Arc<RoutingHarness>) -> Coordinator {
    Coordinator::builder(
        harness,
        CoordinatorProfile::default_for(BACKEND_ID, BACKEND_LABEL, Locale::Zh)
            .directory("/coordinator"),
    )
    .locale(Locale::Zh)
    .build()
}

fn options() -> TurnOptions {
    TurnOptions::new(OWNER, WORK_ID)
}

fn request<'a>() -> CoordinationRequest<'a> {
    CoordinationRequest {
        original_request: "继续改刚才那个页面",
        objective: "继续修改此前讨论的页面",
        coordination_run_id: WORK_ID,
        voice_session_id: "voice-one",
        turn_id: "turn-one",
        working_directory: "/Users/me/codes/current-project",
        ..CoordinationRequest::default()
    }
}

#[tokio::test(start_paused = true)]
async fn a_completed_reply_is_the_answer() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(&completed("页面已经修改并通过检查。")))
        .build();
    let coordinator = coordinator(Arc::clone(&harness));

    let outcome = coordinator
        .run(&request(), Utc::now(), &options())
        .await
        .expect("a final result");

    assert_eq!(outcome.content(), "页面已经修改并通过检查。");
    assert_eq!(outcome.decision.work_id, WORK_ID);
    assert_eq!(outcome.envelope.protocol, BACKEND_ID);
    assert_eq!(outcome.envelope.backend_ref.role, "backend");
    assert_eq!(
        outcome.envelope.backend_ref.session_id,
        COORDINATOR_SESSION_ID
    );
    assert_eq!(outcome.envelope.backend_ref.directory, "/coordinator");
    assert!(outcome.envelope.delegation.is_none());
    assert_eq!(harness.coordinator_prompts().len(), 1);
}

#[tokio::test(start_paused = true)]
async fn the_turn_carries_the_instructions_the_envelope_and_the_context() {
    let memories = [
        MemoryDocument::new("rules", "代码注释一律用中文".to_owned(), "r"),
        MemoryDocument::new("memory", "用户喜欢苹果".to_owned(), "m"),
    ];
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(&completed("好的")))
        .build();
    let coordinator = coordinator(Arc::clone(&harness));

    coordinator
        .run(
            &CoordinationRequest {
                user_memories: &memories,
                ..request()
            },
            Utc.timestamp_millis_opt(0).single().expect("an instant"),
            &options(),
        )
        .await
        .expect("a final result");

    let prompt = harness.coordinator_prompts().pop().expect("one prompt");

    // The wrapper, first, with the profile's session paragraph inside it.
    assert!(
        prompt.starts_with("<via_backend_instructions>\n"),
        "{prompt}"
    );
    assert!(prompt.contains(via_coordinator::BACKEND_AGENT_INSTRUCTIONS));
    assert!(prompt.contains(via_mcp_tools::DEFAULT_SESSION_INSTRUCTIONS));
    assert!(prompt.contains("</via_backend_instructions>\n\n<via_request>"));

    // The envelope, and the identity substitutions inside it.
    assert!(
        prompt.contains(r#""protocol": "via.coordination.v1""#),
        "{prompt}"
    );
    assert!(prompt.contains(r#""request_id": "work-one""#));
    assert!(prompt.contains(r#""owner_scope": "current_authenticated_user""#));
    assert!(prompt.contains(r#""timestamp": "1970-01-01T00:00:00.000Z""#));
    assert!(prompt.contains("/Users/me/codes/current-project"));
    assert!(prompt.contains("</via_request>"));

    // The four context blocks and the twelve notes.
    // A `markdown` document is used whole; only a legacy non-markdown record
    // becomes a bullet (`envelope::tests::a_non_markdown_record_becomes_a_bullet`).
    assert!(prompt.contains("<user_preferences>\n代码注释一律用中文\n</user_preferences>"));
    assert!(prompt.contains("<user_memory>\n用户喜欢苹果\n</user_memory>"));
    assert!(prompt.contains("<recent_voice_context>"));
    assert!(prompt.contains("<voice_work_context>"));
    assert!(prompt.contains(t(Locale::Zh, keys::COORDINATOR_CLIENT_CONTEXT_NOTE)));
    assert!(prompt.contains(t(Locale::Zh, keys::COORDINATOR_USER_PREFERENCES_NOTE)));
    assert!(prompt.contains(t(Locale::Zh, keys::COORDINATOR_ROUTING_NOTE)));
    assert!(prompt.contains(t(Locale::Zh, keys::COORDINATOR_DELEGATION_NOTE)));
    assert!(prompt.contains(t(Locale::Zh, keys::COORDINATOR_STATUS_NOTE)));
    assert!(prompt.contains(t(Locale::Zh, keys::COORDINATOR_FINAL_ONLY_NOTE)));

    // Nothing upstream-branded survives into what the model reads.
    assert!(!prompt.contains("qwen"), "{prompt}");
}

#[tokio::test(start_paused = true)]
async fn attachment_metadata_travels_in_the_envelope_and_bytes_travel_beside_it() {
    let attachments = [EnvelopeAttachment {
        label: "[Image 1]".to_owned(),
        name: Some("reference.png".to_owned()),
        mime_type: "image/png".to_owned(),
        bytes: Some(5),
    }];
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(&completed("完成")))
        .build();
    let coordinator = coordinator(Arc::clone(&harness));

    coordinator
        .run(
            &CoordinationRequest {
                attachments: &attachments,
                ..request()
            },
            Utc::now(),
            &options().with_attachment(PromptAttachment {
                url: "data:image/png;base64,aGVsbG8=".to_owned(),
                filename: "reference.png".to_owned(),
                mime: "image/png".to_owned(),
            }),
        )
        .await
        .expect("a final result");

    let prompt = harness.coordinator_prompts().pop().expect("one prompt");
    assert!(prompt.contains(r#""attachments""#));
    assert!(prompt.contains("reference.png"));
    assert!(
        !prompt.contains("aGVsbG8="),
        "the bytes travel as a PromptAttachment, never inside the envelope",
    );
}

#[tokio::test(start_paused = true)]
async fn a_trusted_backend_event_brings_its_own_note() {
    let event = TrustedBackendEvent {
        kind: String::new(),
        parent_request_id: "work-zero".to_owned(),
        content: "the target finished".to_owned(),
        error: String::new(),
    };
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(&completed("完成")))
        .build();
    let coordinator = coordinator(Arc::clone(&harness));

    coordinator
        .run(
            &CoordinationRequest {
                backend_event: Some(&event),
                delivery: Delivery {
                    voice_connected: false,
                    allow_status: true,
                },
                ..request()
            },
            Utc::now(),
            &options(),
        )
        .await
        .expect("a final result");

    let prompt = harness.coordinator_prompts().pop().expect("one prompt");
    assert!(prompt.contains(r#""kind": "native_task_result""#));
    assert!(prompt.contains(r#""parent_request_id": "work-zero""#));
    assert!(prompt.contains(t(Locale::Zh, keys::COORDINATOR_TRUSTED_BACKEND_EVENT_NOTE)));
    assert!(prompt.contains(r#""voice_connected": false"#));
    assert!(prompt.contains(r#""status": "meaningful_only""#));
}

#[tokio::test(start_paused = true)]
async fn a_non_deliverable_state_is_retried_twice_and_then_refused() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(
            &json!({ "work_id": WORK_ID, "state": "active" }).to_string(),
        ))
        .build();
    let coordinator = coordinator(Arc::clone(&harness));

    let error = coordinator
        .run(&request(), Utc::now(), &options())
        .await
        .expect_err("never final");
    assert!(
        matches!(&error, CoordinatorError::NotFinalResult { state } if state == "active"),
        "{error:?}",
    );
    assert_eq!(error.code(), "VIA_COORDINATOR_NOT_FINAL_RESULT");

    let prompts = harness.coordinator_prompts();
    assert_eq!(prompts.len(), 3, "one turn plus exactly two retries");
    for retry in &prompts[1..] {
        assert!(retry.contains("<via_protocol_retry>"), "{retry}");
        assert!(retry.contains("request_id=work-one"));
        assert!(retry.contains("state=active"));
        assert!(retry.contains("</via_protocol_retry>"));
    }
}

#[tokio::test(start_paused = true)]
async fn a_retry_that_lands_is_the_answer() {
    let harness = RoutingHarness::builder()
        .route(
            "<via_protocol_retry>",
            Reply::content(&completed("终于完成")),
        )
        .fallback(Reply::content(
            &json!({ "work_id": WORK_ID, "state": "active" }).to_string(),
        ))
        .build();
    let coordinator = coordinator(Arc::clone(&harness));

    let outcome = coordinator
        .run(&request(), Utc::now(), &options())
        .await
        .expect("the retry landed");
    assert_eq!(outcome.content(), "终于完成");
    assert_eq!(harness.coordinator_prompts().len(), 2);
}

#[tokio::test(start_paused = true)]
async fn a_reply_with_no_state_at_all_is_deliverable() {
    // Upstream accepts an empty state: `if (!state || state === 'completed')`.
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(
            &json!({ "presentation": { "speech": "没有 state 字段", "inline": null } }).to_string(),
        ))
        .build();
    let coordinator = coordinator(Arc::clone(&harness));

    let outcome = coordinator
        .run(&request(), Utc::now(), &options())
        .await
        .expect("deliverable");
    assert_eq!(outcome.content(), "没有 state 字段");
    assert_eq!(harness.coordinator_prompts().len(), 1, "no retry was sent");
}

#[tokio::test(start_paused = true)]
async fn an_empty_reply_is_retried_once_in_a_fresh_session() {
    // Upstream: 'retries an empty ACP coordinator response in a fresh Session'.
    let harness = RoutingHarness::builder()
        .route("<via_protocol_retry>", Reply::content(&completed("unused")))
        .fallback(Reply::Content(String::new()))
        .build();
    let coordinator = coordinator(Arc::clone(&harness));

    let error = coordinator
        .run(&request(), Utc::now(), &options())
        .await
        .expect_err("still empty");
    assert!(
        matches!(&error, CoordinatorError::EmptyResponse { recoverable, .. } if *recoverable),
        "{error:?}",
    );
    assert_eq!(error.http_status(), 502);
    assert_eq!(
        error.message(Locale::Zh),
        "OpenCode ACP Session 未返回任何内容",
    );
    assert_eq!(
        harness.opened().len(),
        2,
        "the first session was discarded and a fresh one opened",
    );
    assert_eq!(harness.coordinator_prompts().len(), 2);
}

#[tokio::test(start_paused = true)]
async fn a_recovered_session_answers_the_same_question() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(&completed("新 Session 已恢复")))
        .build();
    let coordinator = coordinator(Arc::clone(&harness));
    let key = coordinator.session_key(OWNER);

    // Prime the cache, then throw the session away exactly as the recovery path
    // does, and assert the next turn opens a new one and still answers.
    coordinator
        .run(&request(), Utc::now(), &options())
        .await
        .expect("a final result");
    coordinator.discard_session(&key);
    let outcome = coordinator
        .run(&request(), Utc::now(), &options())
        .await
        .expect("a final result");

    assert_eq!(outcome.content(), "新 Session 已恢复");
    assert_eq!(harness.opened(), [key.as_str(), key.as_str()]);
}

#[tokio::test(start_paused = true)]
async fn the_fixed_session_survives_every_voice_session_and_work_id() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(&completed("完成")))
        .build();
    let coordinator = coordinator(Arc::clone(&harness));

    for (voice, work) in [("voice-one", "work-one"), ("voice-two", "work-two")] {
        coordinator
            .run(
                &CoordinationRequest {
                    voice_session_id: voice,
                    coordination_run_id: work,
                    ..request()
                },
                Utc::now(),
                &TurnOptions::new(OWNER, work),
            )
            .await
            .expect("a final result");
    }

    assert_eq!(
        harness.opened(),
        [coordinator.session_key(OWNER).as_str()],
        "one open, for one identity, across two voice sessions and two Work ids",
    );
    assert_eq!(harness.coordinator_prompts().len(), 2);
}

#[tokio::test(start_paused = true)]
async fn two_owners_are_two_sessions() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(&completed("完成")))
        .build();
    let coordinator = coordinator(Arc::clone(&harness));

    for owner in ["owner-one", "owner-two"] {
        coordinator
            .run(&request(), Utc::now(), &TurnOptions::new(owner, WORK_ID))
            .await
            .expect("a final result");
    }
    assert_eq!(
        harness.opened(),
        [
            "opencode:owner-one:backend".to_owned(),
            "opencode:owner-two:backend".to_owned(),
        ],
    );
}

#[tokio::test(start_paused = true)]
async fn a_double_encoded_reply_is_unwrapped_before_the_speech_is_chosen() {
    let inner = completed("小画板项目已经做好了。");
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(
            &serde_json::Value::String(inner).to_string(),
        ))
        .build();
    let coordinator = coordinator(Arc::clone(&harness));

    let outcome = coordinator
        .run(&request(), Utc::now(), &options())
        .await
        .expect("a final result");
    assert_eq!(outcome.content(), "小画板项目已经做好了。");
}

#[tokio::test(start_paused = true)]
async fn a_legacy_string_inline_is_upgraded_before_the_decision_is_read() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(
            &json!({
                "work_id": WORK_ID,
                "state": "completed",
                "mode": "respond",
                "presentation": { "speech": "完成", "inline": "## 报告" },
            })
            .to_string(),
        ))
        .build();
    let coordinator = coordinator(Arc::clone(&harness));

    let outcome = coordinator
        .run(&request(), Utc::now(), &options())
        .await
        .expect("a final result");
    let inline = outcome
        .decision
        .presentation
        .inline
        .as_ref()
        .expect("upgraded from the legacy string form");
    assert_eq!(inline.title, t(Locale::Zh, keys::ACP_AGENT_RESULT_TITLE));
    assert_eq!(inline.content, "## 报告");
}

#[tokio::test(start_paused = true)]
async fn a_harness_refusal_is_not_retried() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::Failing(
            "OpenCode ACP session/prompt 失败".to_owned(),
        ))
        .build();
    let coordinator = coordinator(Arc::clone(&harness));

    let error = coordinator
        .run(&request(), Utc::now(), &options())
        .await
        .expect_err("the transport failed");
    assert_eq!(error.code(), "VIA_HARNESS_AGENT");
    assert!(!error.is_recoverable());
    assert_eq!(harness.coordinator_prompts().len(), 1);
    assert_eq!(harness.opened().len(), 1);
}

#[tokio::test(start_paused = true)]
async fn a_cancelled_turn_is_reported_as_cancelled_not_completed() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::Stopping(
            "half an answer".to_owned(),
            via_downstream::StopReason::Cancelled,
        ))
        .build();
    let coordinator = coordinator(Arc::clone(&harness));

    let error = coordinator
        .run(&request(), Utc::now(), &options())
        .await
        .expect_err("cancelled is never a result");
    assert!(error.is_cancelled());
    assert_eq!(error.code(), "VIA_HARNESS_CANCELLED");
}

#[tokio::test(start_paused = true)]
async fn a_work_outcome_publishes_the_presentation_and_nothing_else() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(
            &json!({
                "work_id": WORK_ID,
                "state": "completed",
                "mode": "respond",
                "presentation": {
                    "speech": "完成",
                    "inline": { "title": "结果", "format": "markdown", "content": "## 完成" },
                },
            })
            .to_string(),
        ))
        .build();
    let coordinator = coordinator(Arc::clone(&harness));

    let outcome = coordinator
        .run(&request(), Utc::now(), &options())
        .await
        .expect("a final result");
    let work = outcome.into_work_outcome();
    assert_eq!(work.content, "完成");
    let projected = via_work::presentation::project(work.metadata.as_ref()).expect("a projection");
    assert_eq!(
        serde_json::to_value(&projected).expect("json"),
        json!({
            "presentation": {
                "speech": "完成",
                "inline": { "title": "结果", "format": "markdown", "content": "## 完成" },
            }
        }),
    );
}
