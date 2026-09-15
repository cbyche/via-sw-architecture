//! The two guards, and the permission relay that lives behind them.
//!
//! `docs/architecture.md` §11, invariants 1 and 2, and §6's *"`register` before
//! emitting, then `wait(…)`, because that ordering is what closes the race."*

mod common;

use std::sync::{Arc, Mutex};

use chrono::Utc;
use common::{
    BACKEND_ID, BACKEND_LABEL, COORDINATOR_SESSION_ID, Reply, RoutingHarness, settle_until,
};
use futures::FutureExt;
use pretty_assertions::assert_eq;
use serde_json::json;
use tokio_util::sync::CancellationToken;
use via_acp::PermissionDecision;
use via_coordinator::testing::RecordingObserver;
use via_coordinator::{
    CoordinationRequest, Coordinator, CoordinatorProfile, PermissionMode, PermissionResponse,
    TurnOptions,
};
use via_i18n::Locale;
use via_work::PermissionStatus;

const OWNER: &str = "owner-one";

fn completed(speech: &str) -> String {
    json!({
        "work_id": "work-one",
        "state": "completed",
        "mode": "respond",
        "presentation": { "speech": speech, "inline": null },
    })
    .to_string()
}

fn profile() -> CoordinatorProfile {
    CoordinatorProfile::default_for(BACKEND_ID, BACKEND_LABEL, Locale::Zh).directory("/coordinator")
}

fn request<'a>() -> CoordinationRequest<'a> {
    CoordinationRequest {
        original_request: "做点事",
        objective: "做点事",
        coordination_run_id: "work-one",
        ..CoordinationRequest::default()
    }
}

// ---------------------------------------------------------------------------
// Guard two: the adapter's own serialization
// ---------------------------------------------------------------------------

#[tokio::test(start_paused = true)]
async fn one_owners_coordinator_writes_are_serial_and_in_order() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(&completed("完成")))
        .build();
    let coordinator = Coordinator::builder(Arc::clone(&harness) as _, profile())
        .locale(Locale::Zh)
        .build();
    let lane = coordinator.coordinator_lane(OWNER);

    // Hold the lane, then queue three turns behind it in a known order.
    let held = coordinator
        .executor()
        .acquire(&lane)
        .await
        .expect("the lane");

    let mut handles = Vec::new();
    for index in 0..3 {
        let queued = coordinator.clone();
        handles.push(tokio::spawn(async move {
            queued
                .run_coordinator(
                    &format!("turn {index}"),
                    &TurnOptions::new(OWNER, &format!("work-{index}")),
                )
                .await
        }));
        // Each is enqueued before the next asks, so arrival order is the
        // assertion rather than a race.
        settle_until(&format!("turn {index} to be queued"), || async {
            coordinator.executor().depth(&lane).await >= index + 2
        })
        .await;
    }
    assert!(coordinator.is_busy(OWNER).await);

    drop(held);
    for handle in handles {
        handle.await.expect("joined").expect("a turn");
    }

    let prompts = harness.coordinator_prompts();
    assert_eq!(prompts.len(), 3);
    for (index, prompt) in prompts.iter().enumerate() {
        assert!(
            prompt.ends_with(&format!("turn {index}")),
            "turn {index} ran out of order: {prompt}",
        );
    }
    assert!(!coordinator.is_busy(OWNER).await);
}

#[tokio::test(start_paused = true)]
async fn a_different_owner_is_a_different_lane() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(&completed("完成")))
        .build();
    let coordinator = Coordinator::builder(Arc::clone(&harness) as _, profile())
        .locale(Locale::Zh)
        .build();

    let held = coordinator
        .executor()
        .acquire(&coordinator.coordinator_lane(OWNER))
        .await
        .expect("the lane");

    // A second owner's turn is not held by the first owner's lane.
    let other = coordinator
        .run_coordinator("other owner", &TurnOptions::new("owner-two", "work-two"))
        .await
        .expect("a turn");
    assert!(other.content.contains("完成"));
    assert!(coordinator.is_busy(OWNER).await);
    assert!(!coordinator.is_busy("owner-two").await);
    drop(held);
}

#[tokio::test(start_paused = true)]
async fn the_two_lane_keys_are_different_things() {
    let coordinator = Coordinator::builder(
        RoutingHarness::builder()
            .fallback(Reply::content("x"))
            .build() as _,
        profile(),
    )
    .build();

    // The Gateway queue's lane is keyed on the owner; the adapter's is keyed on
    // the session. Porting one of the two would collapse them into one key.
    assert_eq!(via_work::coordinator_lane(OWNER), "coordinator:owner-one");
    assert_eq!(
        coordinator.coordinator_lane(OWNER),
        "coordinator:opencode:owner-one:backend",
    );
    assert_ne!(
        via_work::coordinator_lane(OWNER),
        coordinator.coordinator_lane(OWNER),
    );
}

// ---------------------------------------------------------------------------
// The permission relay
// ---------------------------------------------------------------------------

/// Ask for a permission from inside a coordinator turn, and answer it from
/// inside the same turn — which is what a user at a keyboard does.
#[tokio::test(start_paused = true)]
async fn a_permission_raised_inside_a_turn_is_answered_and_announced() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(&completed("完成")))
        .build();
    let coordinator = Coordinator::builder(Arc::clone(&harness) as _, profile())
        .locale(Locale::Zh)
        .build();
    let observer = RecordingObserver::new();
    let answered: Arc<Mutex<Option<PermissionDecision>>> = Arc::new(Mutex::new(None));

    {
        let coordinator = coordinator.clone();
        let answered = Arc::clone(&answered);
        harness.on_prompt(
            "<via_request>",
            Arc::new(move || {
                let coordinator = coordinator.clone();
                let answered = Arc::clone(&answered);
                async move {
                    let asking = {
                        let coordinator = coordinator.clone();
                        tokio::spawn(async move {
                            coordinator
                                .handle_permission(
                                    COORDINATOR_SESSION_ID,
                                    &json!({
                                        "name": "write",
                                        "rawInput": { "path": "/tmp/file" },
                                    }),
                                )
                                .await
                        })
                    };
                    settle_until("the permission to be registered", || async {
                        !coordinator.permissions().pending().await.is_empty()
                    })
                    .await;
                    let pending = coordinator.permissions().pending().await;
                    coordinator
                        .respond_permission(&pending[0].id, PermissionResponse::Always, OWNER)
                        .await
                        .expect("resolved");
                    *answered
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) =
                        asking.await.expect("joined").ok();
                }
                .boxed()
            }),
        );
    }

    coordinator
        .run(
            &request(),
            Utc::now(),
            &TurnOptions::new(OWNER, "work-one")
                .with_permission_observer(Arc::clone(&observer) as _),
        )
        .await
        .expect("a final result");

    assert_eq!(
        *answered
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        Some(PermissionDecision::Approve),
    );
    assert_eq!(
        observer.names(),
        [
            "backend.permission.requested",
            "backend.permission.resolved"
        ],
    );
    let events = observer.events();
    let requested = events[0].permission().expect("a permission");
    assert_eq!(requested.status, PermissionStatus::Pending);
    assert_eq!(requested.work_id.as_deref(), Some("work-one"));
    assert_eq!(requested.category, "write");
    assert_eq!(requested.summary, "write：/tmp/file");
    assert_eq!(requested.patterns.as_deref(), Some([].as_slice()));

    let resolved = events[1].permission().expect("a permission");
    assert_eq!(resolved.id, requested.id);
    assert_eq!(resolved.status, PermissionStatus::Approved);
    assert_eq!(resolved.patterns, None);
}

#[tokio::test(start_paused = true)]
async fn a_permission_that_outlives_its_prompt_is_cancelled_with_it() {
    let harness = RoutingHarness::builder()
        .fallback(Reply::content(&completed("完成")))
        .build();
    let coordinator = Coordinator::builder(Arc::clone(&harness) as _, profile())
        .locale(Locale::Zh)
        .build();
    let observer = RecordingObserver::new();
    let answered: Arc<Mutex<Option<PermissionDecision>>> = Arc::new(Mutex::new(None));
    let asking: Arc<Mutex<Option<tokio::task::JoinHandle<_>>>> = Arc::new(Mutex::new(None));

    {
        let coordinator = coordinator.clone();
        let asking = Arc::clone(&asking);
        harness.on_prompt(
            "<via_request>",
            Arc::new(move || {
                let coordinator = coordinator.clone();
                let asking = Arc::clone(&asking);
                async move {
                    let handle = {
                        let coordinator = coordinator.clone();
                        tokio::spawn(async move {
                            coordinator
                                .handle_permission(
                                    COORDINATOR_SESSION_ID,
                                    &json!({ "name": "shell" }),
                                )
                                .await
                        })
                    };
                    settle_until("the permission to be registered", || async {
                        !coordinator.permissions().pending().await.is_empty()
                    })
                    .await;
                    *asking
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(handle);
                }
                .boxed()
            }),
        );
    }

    coordinator
        .run(
            &request(),
            Utc::now(),
            &TurnOptions::new(OWNER, "work-one")
                .with_permission_observer(Arc::clone(&observer) as _),
        )
        .await
        .expect("a final result");

    let handle = asking
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
        .expect("the asker");
    *answered
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = handle.await.expect("joined").ok();

    assert_eq!(
        *answered
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        Some(PermissionDecision::Cancel),
        "the prompt ended, so the question went with it",
    );
    assert!(coordinator.permissions().pending().await.is_empty());
    let last = observer.events().pop().expect("an event");
    assert_eq!(
        last.permission().map(|permission| permission.status),
        Some(PermissionStatus::Cancelled),
    );
}

#[tokio::test(start_paused = true)]
async fn vias_own_session_tools_never_reach_a_person() {
    let coordinator = Coordinator::builder(
        RoutingHarness::builder()
            .fallback(Reply::content("x"))
            .build() as _,
        profile(),
    )
    .build();

    for name in via_mcp_tools::SESSION_TOOL_NAMES {
        for spelling in [
            name.to_owned(),
            format!("mcp__via__{name}"),
            format!("{name} (build the thing)"),
        ] {
            let decision = coordinator
                .handle_permission(COORDINATOR_SESSION_ID, &json!({ "title": spelling }))
                .await
                .expect("the broker is running");
            assert_eq!(decision, PermissionDecision::Approve, "{spelling}");
        }
    }
    assert!(coordinator.permissions().pending().await.is_empty());
}

#[tokio::test(start_paused = true)]
async fn full_permission_mode_answers_without_a_broker_round_trip() {
    let coordinator = Coordinator::builder(
        RoutingHarness::builder()
            .fallback(Reply::content("x"))
            .build() as _,
        CoordinatorProfile {
            permission_mode: PermissionMode::Full,
            ..profile()
        },
    )
    .build();

    let decision = coordinator
        .handle_permission(COORDINATOR_SESSION_ID, &json!({ "name": "shell" }))
        .await
        .expect("the broker is running");
    assert_eq!(decision, PermissionDecision::Approve);
    assert!(coordinator.permissions().pending().await.is_empty());
}

#[tokio::test(start_paused = true)]
async fn a_permission_from_a_session_with_no_active_prompt_belongs_to_nobody() {
    let coordinator = Coordinator::builder(
        RoutingHarness::builder()
            .fallback(Reply::content("x"))
            .build() as _,
        profile(),
    )
    .build();

    let asking = {
        let coordinator = coordinator.clone();
        tokio::spawn(async move {
            coordinator
                .handle_permission("a-session-nobody-is-prompting", &json!({ "name": "shell" }))
                .await
        })
    };
    settle_until("the permission to be registered", || async {
        !coordinator.permissions().pending().await.is_empty()
    })
    .await;
    let pending = coordinator.permissions().pending().await;
    assert_eq!(pending[0].work_id, None);
    assert!(
        coordinator
            .respond_permission(&pending[0].id, PermissionResponse::Always, OWNER)
            .await
            .is_err(),
        "nobody owns it, so nobody can answer it",
    );
    assert_eq!(coordinator.permissions().cancel_all().await, 1);
    assert_eq!(
        asking.await.expect("joined").expect("running"),
        PermissionDecision::Cancel,
    );
}

#[tokio::test(start_paused = true)]
async fn a_barge_in_takes_the_question_with_it() {
    let coordinator = Coordinator::builder(
        RoutingHarness::builder()
            .fallback(Reply::content("x"))
            .build() as _,
        profile(),
    )
    .build();
    let signal = CancellationToken::new();

    let asking = {
        let permissions = coordinator.permissions().clone();
        let context = via_coordinator::PermissionContext {
            owner_id: OWNER.to_owned(),
            work_id: Some("work-one".to_owned()),
            session_id: COORDINATOR_SESSION_ID.to_owned(),
            scope_id: "prompt-one".to_owned(),
            observer: None,
            signal: Some(signal.clone()),
        };
        tokio::spawn(async move {
            permissions
                .request(&json!({ "name": "shell" }), context)
                .await
        })
    };
    settle_until("the permission to be registered", || async {
        !coordinator.permissions().pending().await.is_empty()
    })
    .await;

    signal.cancel();
    assert_eq!(
        asking.await.expect("joined").expect("running"),
        PermissionDecision::Cancel,
    );
    assert!(coordinator.permissions().pending().await.is_empty());
}
