//! The delegation lifecycle: what a `state=delegated` reply actually costs,
//! what it releases, and who is allowed to end it.
//!
//! Ports `server/test/acp-backend-adapter.test.mjs:1113-1158` (both the `start`
//! and the `send` shapes), `:1309-1384` (the two cancellation routes and the
//! reconciliation that follows the second), and `:1780-1885` (the native path).

mod common;

use std::sync::Arc;

use chrono::Utc;
use common::{
    BACKEND_ID, BACKEND_LABEL, COORDINATOR_SESSION_ID, Reply, RoutingHarness, settle_until,
};
use futures::FutureExt;
use pretty_assertions::assert_eq;
use serde_json::json;
use via_coordinator::testing::{
    FixedSessionDirectory, RecordedEvent, RecordingObserver, ScriptedNativeDelegation,
};
use via_coordinator::{
    CoordinationRequest, Coordinator, CoordinatorError, CoordinatorProfile, NativeToolUpdate,
    TurnOptions,
};
use via_i18n::Locale;
use via_mcp_tools::{SessionSendInput, SessionStartInput, SessionsListInput};
use via_work::DelegationRef;

const WORK_ID: &str = "work-one";
const OWNER: &str = "owner-one";
const CONFIRMED_AT: &str = "2026-08-22T00:00:00.000Z";

fn completed(speech: &str) -> String {
    json!({
        "work_id": WORK_ID,
        "state": "completed",
        "mode": "respond",
        "presentation": { "speech": speech, "inline": null },
    })
    .to_string()
}

fn delegated() -> String {
    json!({
        "work_id": WORK_ID,
        "state": "delegated",
        "mode": "delegate",
        "delegation_id": "opencode_run_x",
        "target_session_id": "project-1",
        "presentation": { "speech": "已经交给独立项目处理。", "inline": null },
    })
    .to_string()
}

fn profile() -> CoordinatorProfile {
    CoordinatorProfile::default_for(BACKEND_ID, BACKEND_LABEL, Locale::Zh).directory("/coordinator")
}

fn options(observer: &Arc<RecordingObserver>) -> TurnOptions {
    TurnOptions::new(OWNER, WORK_ID).with_observer(Arc::clone(observer) as _)
}

fn request<'a>() -> CoordinationRequest<'a> {
    CoordinationRequest {
        original_request: "帮我做个小画板",
        objective: "创建小画板项目",
        coordination_run_id: WORK_ID,
        ..CoordinationRequest::default()
    }
}

/// Wire a harness so that answering the *first* coordinator turn calls
/// `via_session_start`, exactly as a backend agent would.
fn delegating_coordinator(
    target: Reply,
) -> (Arc<RoutingHarness>, Coordinator, Arc<RecordingObserver>) {
    let harness = RoutingHarness::builder()
        .route(
            "<via_delegation_result>",
            Reply::content(&completed("第三层结果已整理")),
        )
        .route("<via_control", Reply::content(&completed("已确认")))
        .route("build project", target)
        .fallback(Reply::content(&delegated()))
        .build();
    let coordinator = Coordinator::builder(Arc::clone(&harness) as _, profile())
        .locale(Locale::Zh)
        .build();
    let observer = RecordingObserver::new();

    let tools = coordinator.tool_context(&options(&observer));
    harness.on_prompt(
        "<via_request>",
        Arc::new(move || {
            let tools = Arc::clone(&tools);
            async move {
                tools
                    .start_session(SessionStartInput {
                        prompt: "build project".to_owned(),
                        title: Some("Project".to_owned()),
                    })
                    .await
                    .expect("the session tool answers");
            }
            .boxed()
        }),
    );
    (harness, coordinator, observer)
}

#[tokio::test(start_paused = true)]
async fn a_delegated_turn_finalizes_through_a_second_coordinator_turn() {
    let (harness, coordinator, observer) =
        delegating_coordinator(Reply::content("built the drawing board"));

    let outcome = coordinator
        .run(&request(), Utc::now(), &options(&observer))
        .await
        .expect("a final result");

    assert_eq!(outcome.content(), "第三层结果已整理");
    assert_eq!(
        observer.names(),
        ["backend.delegated", "backend.delegation.completed"],
    );

    let prompts = harness.coordinator_prompts();
    assert_eq!(
        prompts.len(),
        2,
        "the envelope turn and the presentation turn"
    );
    assert!(prompts[0].contains("<via_request>"));
    assert!(prompts[1].contains("<via_delegation_result>"));
    assert!(prompts[1].contains("built the drawing board"));
    assert!(prompts[1].contains(r#""request_id": "work-one""#));
    assert!(prompts[1].contains(r#""target_session_id": "project-1""#));

    // The delegation is forgotten once the run that owned it is over.
    assert!(coordinator.delegations().is_empty());
    assert_eq!(
        outcome
            .envelope
            .delegation
            .as_ref()
            .map(|record| record.session_id.clone()),
        Some("project-1".to_owned()),
    );
}

#[tokio::test(start_paused = true)]
async fn the_start_confirmation_is_announced_before_the_target_finishes() {
    let (_, coordinator, observer) = delegating_coordinator(Reply::content("done"));

    coordinator
        .run(&request(), Utc::now(), &options(&observer))
        .await
        .expect("a final result");

    let events = observer.events();
    let announced = events[0].delegation().expect("a delegation");
    let presentation = announced.presentation.as_ref().expect("a presentation");
    assert_eq!(presentation["speech"], "已经交给独立项目处理。");
    assert_eq!(
        announced.status.as_deref(),
        Some("running"),
        "the delegation is running when it is announced, not completed",
    );

    let finished = events[1].delegation().expect("a delegation");
    assert_eq!(finished.status.as_deref(), Some("completed"));
    assert_eq!(finished.id, announced.id);
    assert_eq!(finished.session_id, announced.session_id);
}

#[tokio::test(start_paused = true)]
async fn a_delegations_ids_never_reach_a_client() {
    let (_, coordinator, observer) = delegating_coordinator(Reply::content("done"));
    coordinator
        .run(&request(), Utc::now(), &options(&observer))
        .await
        .expect("a final result");

    for event in observer.events() {
        let Some(delegation) = event.delegation() else {
            continue;
        };
        let published =
            serde_json::to_string(&delegation.to_public()).expect("the public projection");
        assert!(!published.contains("opencode_run_"), "{published}");
        assert!(!published.contains("project-1"), "{published}");
        assert!(!published.contains("/coordinator"), "{published}");
    }
}

#[tokio::test(start_paused = true)]
async fn the_coordinator_lane_is_free_while_the_target_runs() {
    // The target parks, so the delegation never finishes on its own. If the
    // coordinator still held its lane, the second turn below would deadlock.
    let (harness, coordinator, observer) = delegating_coordinator(Reply::Park);

    let running = {
        let coordinator = coordinator.clone();
        let observer = Arc::clone(&observer);
        tokio::spawn(async move {
            coordinator
                .run(&request(), Utc::now(), &options(&observer))
                .await
        })
    };
    settle_until("the delegation to be announced", || async {
        observer.saw("backend.delegated")
    })
    .await;

    // A different voice request, same owner, same coordinator session.
    let other = coordinator
        .run_coordinator("another request", &TurnOptions::new(OWNER, "work-two"))
        .await
        .expect("the lane was released");
    assert!(other.content.contains("delegated"));

    // Now let the delegated Session go, and the first run finishes.
    coordinator
        .cancel_delegated_work(WORK_ID, OWNER, CONFIRMED_AT)
        .await
        .expect("a cancellable delegation");
    let error = running
        .await
        .expect("joined")
        .expect_err("the delegated Session was cancelled");
    assert!(error.is_cancelled(), "{error:?}");
    assert!(harness.coordinator_prompts().len() >= 2);
}

#[tokio::test(start_paused = true)]
async fn one_turn_delegates_once() {
    let (_, coordinator, observer) = delegating_coordinator(Reply::content("done"));
    let tools = coordinator.tool_context(&options(&observer));

    tools
        .start_session(SessionStartInput {
            prompt: "first".to_owned(),
            title: None,
        })
        .await
        .expect("the first delegation");
    let refused = tools
        .start_session(SessionStartInput {
            prompt: "second".to_owned(),
            title: None,
        })
        .await
        .expect_err("a second delegation in one turn");
    assert_eq!(
        refused.message(Locale::Zh),
        "当前协调轮次已经启动了一个第三层任务",
    );
}

#[tokio::test(start_paused = true)]
async fn continuing_a_session_needs_a_directory_and_resumes_the_named_one() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::content("continued"))
        .build();
    let coordinator = Coordinator::builder(Arc::clone(&harness) as _, profile())
        .locale(Locale::Zh)
        .session_directory(FixedSessionDirectory::with_session(
            "previous-session",
            "Previous project",
            "/previous",
        ))
        .build();
    let observer = RecordingObserver::new();
    let tools = coordinator.tool_context(&options(&observer));

    let started = tools
        .send_session(SessionSendInput {
            session_id: "previous-session".to_owned(),
            prompt: "continue previous work".to_owned(),
        })
        .await
        .expect("the Session's directory is known");
    let rendered = serde_json::to_value(&started).expect("json");
    assert_eq!(rendered["status"], "started");
    assert_eq!(rendered["session_id"], "previous-session");
    assert_eq!(rendered["directory"], "/previous");
    assert_eq!(rendered["title"], "continue previous work");

    assert!(
        harness
            .opened()
            .contains(&"opencode:previous-session".to_owned()),
        "{:?}",
        harness.opened(),
    );
}

#[tokio::test(start_paused = true)]
async fn continuing_an_unknown_session_is_refused_rather_than_guessed() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::content("continued"))
        .build();
    let coordinator = Coordinator::builder(Arc::clone(&harness) as _, profile())
        .locale(Locale::Zh)
        .build();
    let observer = RecordingObserver::new();
    let tools = coordinator.tool_context(&options(&observer));

    let refused = tools
        .send_session(SessionSendInput {
            session_id: "never-heard-of-it".to_owned(),
            prompt: "carry on".to_owned(),
        })
        .await
        .expect_err("no directory is known");
    assert_eq!(
        refused.message(Locale::Zh),
        "OpenCode Session 的项目目录未知，请先查询 Session 列表后再继续",
    );
    assert!(
        !harness
            .opened()
            .iter()
            .any(|key| key.contains("never-heard-of-it")),
        "nothing was resumed into a guessed directory",
    );
}

#[tokio::test(start_paused = true)]
async fn listing_sessions_filters_by_title_and_directory() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::content("x"))
        .build();
    let coordinator = Coordinator::builder(Arc::clone(&harness) as _, profile())
        .session_directory(FixedSessionDirectory::new(vec![
            via_mcp_tools::SessionSummary::new("s-1", "Drawing board", "/srv/board", "t"),
            via_mcp_tools::SessionSummary::new("s-2", "Invoices", "/srv/money", "t"),
        ]))
        .build();
    let observer = RecordingObserver::new();
    let tools = coordinator.tool_context(&options(&observer));

    let all = tools
        .list_sessions(SessionsListInput::default())
        .await
        .expect("a listing");
    assert_eq!(all.sessions.len(), 2);

    let filtered = tools
        .list_sessions(SessionsListInput {
            query: Some("  MONEY ".to_owned()),
            limit: None,
        })
        .await
        .expect("a listing");
    assert_eq!(filtered.sessions.len(), 1);
    assert_eq!(filtered.sessions[0].session_id, "s-2");
}

#[tokio::test(start_paused = true)]
async fn an_idle_coordinator_cancels_through_the_model_and_the_answer_is_the_confirmation() {
    let (harness, coordinator, observer) = delegating_coordinator(Reply::Park);
    let running = {
        let coordinator = coordinator.clone();
        let observer = Arc::clone(&observer);
        tokio::spawn(async move {
            coordinator
                .run(&request(), Utc::now(), &options(&observer))
                .await
        })
    };
    settle_until("the delegation to be announced", || async {
        observer.saw("backend.delegated")
    })
    .await;

    let outcome = coordinator
        .cancel_delegated_work(WORK_ID, OWNER, CONFIRMED_AT)
        .await
        .expect("a cancellable delegation");

    assert!(outcome.is_confirmed(), "the model's own tool call returned");
    assert_eq!(outcome.status_str(), "cancelled");
    assert_eq!(
        outcome.route(),
        Some(via_downstream::CancelRoute::Coordinator),
    );
    assert_eq!(outcome.confirmed_at(), Some(CONFIRMED_AT));
    assert!(
        harness
            .coordinator_prompts()
            .iter()
            .any(|prompt| prompt.contains(r#"<via_control kind="cancel">"#)),
    );
    // Nothing to reconcile: the model was there when it happened.
    assert!(coordinator.pending_facts(OWNER).is_empty());
    let _ = running.await;
}

#[tokio::test(start_paused = true)]
async fn a_busy_coordinator_is_cancelled_through_the_transport_and_told_next_turn() {
    let (harness, coordinator, observer) = delegating_coordinator(Reply::Park);
    let running = {
        let coordinator = coordinator.clone();
        let observer = Arc::clone(&observer);
        tokio::spawn(async move {
            coordinator
                .run(&request(), Utc::now(), &options(&observer))
                .await
        })
    };
    settle_until("the delegation to be announced", || async {
        observer.saw("backend.delegated")
    })
    .await;

    // Hold the coordinator lane, exactly as a second voice turn would.
    let held = coordinator
        .executor()
        .acquire(&coordinator.coordinator_lane(OWNER))
        .await
        .expect("the lane");

    let outcome = coordinator
        .cancel_delegated_work(WORK_ID, OWNER, CONFIRMED_AT)
        .await
        .expect("a cancellable delegation");
    assert_eq!(
        outcome.route(),
        Some(via_downstream::CancelRoute::Adapter),
        "an urgent cancel does not queue behind a busy coordinator",
    );
    assert!(
        !outcome.is_confirmed(),
        "a transport notification confirms nothing",
    );
    assert_eq!(outcome.status_str(), "cancelling");

    let facts = coordinator.pending_facts(OWNER);
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].kind, "delegated_session_cancelled");
    assert_eq!(facts[0].work_id, WORK_ID);
    assert_eq!(facts[0].target_session_id, "project-1");
    assert_eq!(facts[0].confirmed_at, CONFIRMED_AT);

    drop(held);
    let _ = running.await;

    // The next coordinator turn carries the fact, and clears it.
    coordinator
        .run_coordinator("plain follow-up", &TurnOptions::new(OWNER, "work-two"))
        .await
        .expect("a follow-up turn");
    let follow_up = harness
        .coordinator_prompts()
        .into_iter()
        .find(|prompt| prompt.contains("plain follow-up"))
        .expect("the follow-up prompt");
    assert!(follow_up.contains("<via_reconciliation>"), "{follow_up}");
    assert!(follow_up.contains("delegated_session_cancelled"));
    assert!(coordinator.pending_facts(OWNER).is_empty());
}

#[tokio::test(start_paused = true)]
async fn a_status_query_goes_through_the_coordinator() {
    let (harness, coordinator, observer) = delegating_coordinator(Reply::Park);
    let running = {
        let coordinator = coordinator.clone();
        let observer = Arc::clone(&observer);
        tokio::spawn(async move {
            coordinator
                .run(&request(), Utc::now(), &options(&observer))
                .await
        })
    };
    settle_until("the delegation to be announced", || async {
        observer.saw("backend.delegated")
    })
    .await;

    let answer = coordinator
        .query_delegated_work(WORK_ID, "做到哪了", OWNER)
        .await
        .expect("a delegation to ask about");
    assert_eq!(answer.content(), "已确认");
    let asked = harness
        .coordinator_prompts()
        .into_iter()
        .find(|prompt| prompt.contains(r#"<via_control kind="status">"#))
        .expect("a status control turn");
    assert!(asked.contains("via_session_status"));
    assert!(asked.contains("用户的具体问题：做到哪了"));

    coordinator
        .cancel_delegated_work(WORK_ID, OWNER, CONFIRMED_AT)
        .await
        .expect("cancellable");
    let _ = running.await;
}

#[tokio::test(start_paused = true)]
async fn another_owners_delegation_is_not_found_rather_than_forbidden() {
    let (_, coordinator, observer) = delegating_coordinator(Reply::Park);
    let tools = coordinator.tool_context(&options(&observer));
    tools
        .start_session(SessionStartInput {
            prompt: "build project".to_owned(),
            title: None,
        })
        .await
        .expect("a delegation");

    let cancel = coordinator
        .cancel_delegated_work(WORK_ID, "owner-two", CONFIRMED_AT)
        .await
        .expect_err("not this owner's");
    assert!(matches!(cancel, CoordinatorError::NotCancellable { .. }));
    assert_eq!(
        cancel.message(Locale::Zh),
        "没有找到可取消的 OpenCode 项目任务",
    );

    let query = coordinator
        .query_delegated_work(WORK_ID, "?", "owner-two")
        .await
        .expect_err("not this owner's");
    assert!(matches!(query, CoordinatorError::DelegationNotFound { .. }));
    assert_eq!(
        query.message(Locale::Zh),
        "没有找到对应的 OpenCode 项目任务",
    );

    coordinator
        .cancel_delegated_work(WORK_ID, OWNER, CONFIRMED_AT)
        .await
        .expect("this owner's");
}

#[tokio::test(start_paused = true)]
async fn a_native_delegation_is_detected_and_finalized_the_same_way() {
    let harness = RoutingHarness::builder()
        .route(
            "<via_delegation_result>",
            Reply::content(&completed("原生结果已整理")),
        )
        .fallback(Reply::content(&delegated()))
        .build();
    let native = ScriptedNativeDelegation::completing("child completed successfully");
    let coordinator =
        Coordinator::builder(Arc::clone(&harness) as _, profile().native_delegation(true))
            .locale(Locale::Zh)
            .native_delegation(Arc::clone(&native) as _)
            .build();
    let observer = RecordingObserver::new();
    let options = options(&observer);

    let update = NativeToolUpdate::tool_call("spawn-one")
        .name("sessions_spawn")
        .status("completed")
        .raw_input(json!({ "task": "build project", "cwd": "/project" }))
        .raw_output(json!({
            "content": [{
                "type": "text",
                "text": json!({
                    "status": "accepted",
                    "runId": "run-one",
                    "childSessionKey": "agent:child:one",
                })
                .to_string(),
            }],
        }));
    {
        let coordinator = coordinator.clone();
        let options = options.clone();
        harness.on_prompt(
            "<via_request>",
            Arc::new(move || {
                let coordinator = coordinator.clone();
                let options = options.clone();
                let update = update.clone();
                async move {
                    coordinator
                        .observe_native(&options, &update)
                        .expect("the predicate fires");
                }
                .boxed()
            }),
        );
    }

    let outcome = coordinator
        .run(&request(), Utc::now(), &options)
        .await
        .expect("a final result");
    assert_eq!(outcome.content(), "原生结果已整理");
    assert_eq!(
        observer.names(),
        ["backend.delegated", "backend.delegation.completed"],
    );
    let delegation = outcome.envelope.delegation.expect("a delegation");
    assert_eq!(
        delegation.id, "run-one",
        "the native path keeps the backend's own runId verbatim",
    );
    assert_eq!(delegation.session_id, "agent:child:one");
    assert_eq!(delegation.directory, "/project");
    assert_eq!(delegation.title, "build project");

    let prompts = harness.coordinator_prompts();
    assert!(prompts[1].contains("child completed successfully"));
    assert_eq!(prompts[0].split(COORDINATOR_SESSION_ID).count(), 1);
}

#[tokio::test(start_paused = true)]
async fn a_backend_that_does_not_delegate_natively_detects_nothing() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(&completed("完成")))
        .build();
    let coordinator = Coordinator::builder(Arc::clone(&harness) as _, profile())
        .locale(Locale::Zh)
        .native_delegation(ScriptedNativeDelegation::completing("never") as _)
        .build();
    let observer = RecordingObserver::new();
    let options = options(&observer);

    let update = NativeToolUpdate::tool_call("spawn-one")
        .name("sessions_spawn")
        .status("completed")
        .raw_output(json!({ "runId": "run-one", "sessionKey": "agent:child:one" }));
    assert_eq!(
        coordinator.observe_native(&options, &update),
        None,
        "the profile does not declare native delegation",
    );

    let outcome = coordinator
        .run(&request(), Utc::now(), &options)
        .await
        .expect("a final result");
    assert_eq!(outcome.content(), "完成");
    assert!(observer.events().is_empty());
}

/// `docs/deviations/phase-9-via-e2e.md` #4 — VIA's own composition wires no
/// [`via_coordinator::NativeDelegationAdapter`], so a restart-recovered
/// delegation is refused rather than resurrected with nothing behind it. This
/// is `agent.canRecoverDelegatedWork`'s `Boolean(nativeDelegationAdapter)`
/// half, at the point it actually matters.
#[tokio::test(start_paused = true)]
async fn recovering_a_delegation_with_no_native_adapter_is_refused() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(&completed("不应该被使用")))
        .build();
    let coordinator = Coordinator::builder(Arc::clone(&harness) as _, profile())
        .locale(Locale::Zh)
        .build();
    assert!(!coordinator.native_delegation_configured());
    let observer = RecordingObserver::new();
    let options = options(&observer);
    let delegation = DelegationRef::new("run-one", "agent:child:one");

    let error = coordinator
        .recover_native_delegation(&delegation, &options)
        .await
        .expect_err("no adapter means no reattachment");
    assert!(matches!(error, CoordinatorError::NotRecoverable { .. }));
    assert_eq!(error.message(Locale::Zh), "OpenCode 无法恢复这项第三层任务");
    assert!(
        observer.events().is_empty(),
        "a refused recovery never announces a delegation that was not reattached",
    );
}

/// The other half: when a native adapter *is* configured, recovery reattaches
/// to the saved ids, replays the saved presentation rather than composing a
/// new one, and finalizes through the same delegation-result turn a
/// freshly-detected native delegation would.
#[tokio::test(start_paused = true)]
async fn recovering_a_delegation_reattaches_and_finalizes_through_the_result_turn() {
    let harness = RoutingHarness::builder()
        .route(
            "<via_delegation_result>",
            Reply::content(&completed("恢复后的结果")),
        )
        .build();
    let native = ScriptedNativeDelegation::completing("child completed successfully");
    let coordinator =
        Coordinator::builder(Arc::clone(&harness) as _, profile().native_delegation(true))
            .locale(Locale::Zh)
            .native_delegation(Arc::clone(&native) as _)
            .build();
    assert!(coordinator.native_delegation_configured());
    let observer = RecordingObserver::new();
    let options = options(&observer);
    let delegation = DelegationRef::new("run-one", "agent:child:one")
        .with_title("build project")
        .with_directory("/project")
        .with_presentation(json!({ "speech": "已经交给独立项目处理。" }));

    let outcome = coordinator
        .recover_native_delegation(&delegation, &options)
        .await
        .expect("the adapter is configured, so recovery reattaches");
    assert_eq!(outcome.content(), "恢复后的结果");
    assert_eq!(
        observer.names(),
        ["backend.delegated", "backend.delegation.completed"],
        "a recovered delegation is announced exactly as a freshly detected one is",
    );
    let announced = match &observer.events()[0] {
        RecordedEvent::Delegated(reference) => reference.clone(),
        other => panic!("expected the delegated event first: {other:?}"),
    };
    assert_eq!(
        announced.presentation,
        Some(json!({ "speech": "已经交给独立项目处理。" })),
        "the presentation the user already heard is replayed, not recomposed",
    );
    let delegation = outcome.envelope.delegation.expect("a delegation record");
    assert_eq!(delegation.id, "run-one");
    assert_eq!(delegation.session_id, "agent:child:one");
    assert_eq!(delegation.directory, "/project");

    // No coordinator lane was ever asked a request prompt: the only turn a
    // recovered delegation takes is the delegation-result presentation.
    let prompts = harness.coordinator_prompts();
    assert_eq!(prompts.len(), 1);
    assert!(prompts[0].contains("child completed successfully"));
}
