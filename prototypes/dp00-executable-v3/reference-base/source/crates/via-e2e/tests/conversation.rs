//! **The full conversation, and the Work HTTP surface beside it.**
//!
//! One Gateway *process*, one WebSocket, one `reqwest` client. The socket
//! watches a Work move; the HTTP client asks the same Gateway about the same
//! Work through every route the catalogue lists, and the two answers have to
//! agree.
//!
//! `apps/via/tests/milestone.rs` already proves the pipeline in-process. What
//! it cannot prove is that the five HTTP routes serialize their catalogued
//! shapes over a real socket to a client that never linked against `via-work`
//! — and that is the whole of this file. Every field name asserted below is
//! read out of `docs/reference/contracts.json` or off a shipped constant, never
//! retyped.

mod support;

use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use support::{BUDGET, Machine, Socket, is};
use via_protocol::{GatewayTaskEvent, WorkKind, WorkState, WorkStatus};
use via_realtime_mock::{Script, script::turns};

/// What the script answers the first message with, itself.
const DIRECT_ANSWER: &str = "Nothing is running right now.";

/// What it says once `spawn_thinking` has been accepted.
const ACKNOWLEDGEMENT: &str = "I have started on that.";

/// What it says when the finished result is injected.
const ANNOUNCEMENT: &str = "That is done: the tests pass.";

/// What the backend answers with.
const RESULT: &str = "The tests pass.";

/// The inline block the backend attaches — the one thing `/api/timeline`
/// selects on.
const INLINE_TITLE: &str = "Diff";
/// No trailing newline: the coordinator's `normalize_inline` trims what the
/// backend supplied, so a fixture that carried one would be asserting the trim
/// rather than the round trip.
const INLINE_CONTENT: &str = "--- a\n+++ b";

/// The objective the model delegates.
const OBJECTIVE: &str = "run the test suite and report";

/// The session this conversation belongs to.
const SESSION: &str = "voice-e2e-conversation";

/// The script the realtime model replays: four consecutive turns, each firing
/// once, which is `via-realtime-mock`'s rule 2.
fn script() -> Script {
    Script::conversation()
        .turn(turns::say(DIRECT_ANSWER))
        .turn(turns::call_tool(
            "call_1",
            "spawn_thinking",
            &json!({ "objective": OBJECTIVE }),
        ))
        .turn(turns::say(ACKNOWLEDGEMENT))
        .turn(turns::say(ANNOUNCEMENT))
}

/// The backend, as the harness Gateway's fixture file spells it.
///
/// One turn, answering the coordinator's decision envelope: `completed` /
/// `respond`, with a presentation carrying both a spoken sentence and an inline
/// block. The inline block is what makes `/api/timeline` non-empty, and an
/// empty timeline would assert nothing.
fn harness() -> Value {
    json!({
        "backend": "opencode",
        "turns": [{
            "kind": "completed",
            "content": json!({
                "work_id": "work_1",
                "state": "completed",
                "mode": "respond",
                "presentation": {
                    "speech": RESULT,
                    "inline": {
                        "title": INLINE_TITLE,
                        "format": "code",
                        "content": INLINE_CONTENT,
                    },
                },
            }).to_string(),
        }],
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn one_conversation_answers_directly_delegates_once_and_serves_every_work_route() {
    let machine = Machine::new();
    let gateway = machine.start_harness_gateway(&script(), Some(&harness()), &[]);
    gateway.require_serving();
    let origin = gateway.origin();
    let api = gateway.api();

    let mut client = Socket::connect(&origin, SESSION).await;
    client.hello(via_i18n::Locale::En).await;

    // ── 1. the fast path creates no Work ────────────────────────────────────
    client.say("is anything running?").await;
    client
        .wait_for(BUDGET, |frame| {
            frame["type"] == "transcript.final"
                && frame["role"] == "assistant"
                && frame["content"] == DIRECT_ANSWER
        })
        .await
        .map_or_else(
            || panic!("the model never answered the first message itself"),
            drop,
        );
    assert!(
        client.seen(GatewayTaskEvent::Running.as_str()).is_empty(),
        "`docs/architecture.md` §4: *direct answers are answered on the fast path; \
         no task is created*",
    );
    let listed = api.get(&format!("/api/tasks?sessionId={SESSION}")).await;
    assert_eq!(listed.status, 200);
    assert_eq!(
        listed.body["tasks"],
        json!([]),
        "…and the Work plane agrees: nothing was created",
    );

    // ── 2. the delegation ───────────────────────────────────────────────────
    // Sent only after the assertions above, so "no Work yet" is a statement
    // about the first turn rather than a race with the second.
    client.say("run the tests").await;

    // ── 3. queued → running → completed ─────────────────────────────────────
    // `queued` has no counterpart in the client vocabulary: a Work is *created*
    // queued and the first frame a socket ever sees is `task.running`
    // (`via_protocol::GatewayTaskEvent` declares no `task.queued`). What proves
    // it passed through `queued` is the acceptance receipt the tool call
    // returns, which is the Work's status at creation and never leaves the
    // model's conversation; the socket's evidence is the pair below, and the
    // record's is `createdAt` preceding `startedAt`.
    let running = client
        .wait_for(BUDGET, is("task.running"))
        .await
        .expect("the scheduler admits the delegation");
    let work_id = running["task"]["id"]
        .as_str()
        .expect("a Work event names its Work")
        .to_owned();
    assert_eq!(running["task"]["status"], WorkStatus::Running.as_str());
    assert_eq!(running["task"]["workState"], WorkState::Active.as_str());
    assert_eq!(running["task"]["kind"], WorkKind::Work.as_str());
    assert_eq!(running["task"]["objective"], OBJECTIVE);
    assert_eq!(running["task"]["sessionId"], SESSION);

    let completed = client
        .wait_for(BUDGET, is("task.completed"))
        .await
        .expect("the coordinator's answer completes the Work");
    assert_eq!(completed["task"]["id"], work_id.as_str());
    assert_eq!(completed["task"]["status"], WorkStatus::Completed.as_str());
    assert_eq!(completed["task"]["result"], RESULT);

    // ── 4. delivered through the announcement window ────────────────────────
    let announced = client
        .wait_for(BUDGET, |frame| {
            frame["type"] == "transcript.final"
                && frame["role"] == "assistant"
                && frame["content"] == ANNOUNCEMENT
        })
        .await
        .expect("the finished result is spoken back into the conversation");
    assert!(
        announced["responseId"]
            .as_str()
            .is_some_and(|id| !id.is_empty()),
        "an announcement is a model turn and carries the turn's id: {announced}",
    );
    // The order is the contract, not merely the presence: a result is spoken
    // only from a *final* backend result, so the announcement cannot precede
    // the Work becoming terminal (`docs/architecture.md` §17 item 5).
    let announced_at = client
        .position_where(|frame| frame["content"] == ANNOUNCEMENT)
        .expect("the announcement is in the log");
    let completed_at = client
        .position_of("task.completed")
        .expect("the completion is in the log");
    assert!(
        completed_at < announced_at,
        "the announcement ({announced_at}) preceded the completion ({completed_at}); \
         frames: {:?}",
        client
            .frames
            .iter()
            .map(|frame| frame["type"].clone())
            .collect::<Vec<_>>(),
    );

    // ── 5. `GET /api/tasks` ─────────────────────────────────────────────────
    let listed = api.get(&format!("/api/tasks?sessionId={SESSION}")).await;
    assert_eq!(listed.status, 200);
    assert!(
        listed.request_id().is_some_and(|id| !id.is_empty()),
        "`X-Request-Id` is an observable response header on all routes",
    );
    let tasks = listed.body["tasks"]
        .as_array()
        .expect("`{tasks: [...]}` is the catalogued envelope");
    assert_eq!(tasks.len(), 1, "{:?}", listed.body);
    let task = &tasks[0];
    assert_eq!(task["id"], work_id.as_str());
    assert_eq!(
        task["workId"],
        work_id.as_str(),
        "upstream publishes both names, and a client may read either",
    );
    assert!(
        task["createdAt"].as_i64() <= task["startedAt"].as_i64(),
        "the Work existed before it ran, which is the `queued` the socket has no \
         frame for: {task}",
    );
    assert_no_backend_internals(task);

    // ── 6. `GET /api/tasks/{id}`, and its 404 ───────────────────────────────
    let one = api.get(&format!("/api/tasks/{work_id}")).await;
    assert_eq!(one.status, 200);
    assert_eq!(&one.body, task, "the single lookup is the same record");

    let missing = api.get("/api/tasks/work_nonesuch").await;
    assert_eq!(
        missing.status, 404,
        "*a task belonging to another owner reads as 404, not 403* — and one \
         nobody owns is the same answer",
    );
    assert_eq!(
        missing.body["error"],
        via_i18n::t(via_i18n::Locale::En, via_i18n::keys::GATEWAY_TASK_NOT_FOUND),
    );

    // ── 7. `GET /api/tasks/{id}/events` — the SSE stream ────────────────────
    let (status, content_type, frame) = api
        .first_sse_frame(&format!("/api/tasks/{work_id}/events"), BUDGET)
        .await
        .expect("the event stream opens and speaks");
    assert_eq!(status, 200);
    assert!(
        content_type.starts_with("text/event-stream"),
        "{content_type}",
    );
    assert_eq!(
        frame,
        via_app::http::tasks::snapshot_frame(
            &serde_json::from_value(task.clone()).expect("the listed record is a PublicWork"),
        ),
        "the synthetic first frame is `task.snapshot` carrying the Work as it stands",
    );
    // …and the 404 arm comes first, before any header is written.
    let (status, _, body) = api
        .first_sse_frame("/api/tasks/work_nonesuch/events", BUDGET)
        .await
        .expect("an unknown id answers rather than hanging");
    assert_eq!(status, 404);
    assert_eq!(
        body["error"],
        via_i18n::t(via_i18n::Locale::En, via_i18n::keys::GATEWAY_TASK_NOT_FOUND),
    );

    // ── 8. `GET /api/timeline` ──────────────────────────────────────────────
    let timeline = api.get(&format!("/api/timeline?sessionId={SESSION}")).await;
    assert_eq!(timeline.status, 200);
    let items = timeline.body["items"]
        .as_array()
        .expect("`{items: [...]}` is the catalogued envelope");
    assert_eq!(items.len(), 1, "{:?}", timeline.body);
    assert_eq!(
        items[0]["id"],
        format!("{}{work_id}", via_app::http::tasks::TIMELINE_ID_PREFIX),
    );
    assert_eq!(items[0]["taskId"], work_id.as_str());
    assert_eq!(items[0]["title"], INLINE_TITLE);
    assert_eq!(items[0]["format"], "code");
    assert_eq!(items[0]["content"], INLINE_CONTENT);
    assert_eq!(
        items[0]["createdAt"], task["completedAt"],
        "`completedAt` wins where there is one",
    );

    // ── 9. `DELETE /api/tasks/{id}` on a terminal Work is the 409 ───────────
    let refused = api.delete(&format!("/api/tasks/{work_id}")).await;
    assert_eq!(refused.status, 409);
    assert_eq!(
        refused.body["error"],
        via_i18n::t(
            via_i18n::Locale::En,
            via_i18n::keys::GATEWAY_TASK_NOT_ACTIVE
        ),
    );
    assert_eq!(
        refused.body["task"], *task,
        "the 409 carries the existing task beside the error, which is what lets a \
         client re-render without a second request",
    );

    // …and a DELETE for an id nobody owns is the 404, not the 409: the two
    // lookups upstream makes are not redundant.
    let missing = api.delete("/api/tasks/work_nonesuch").await;
    assert_eq!(missing.status, 404);

    let run = gateway.stop();
    assert_eq!(run.code, Some(0), "stderr: {}", run.stderr);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_gateway_with_no_harness_serves_the_work_routes_and_refuses_to_delegate() {
    // `docs/architecture.md` §2: *"`agent` degrades to `direct` when no harness
    // is configured, rather than failing … The degradation is reported on
    // `/api/health`, never silent."* A frontend-only Gateway is what a fresh
    // install is, so the routes still have to answer.
    let machine = Machine::new();
    let gateway = machine.start_harness_gateway(&script(), None, &[]);
    gateway.require_serving();
    let api = gateway.api();

    let health = api.get("/api/health").await;
    assert_eq!(health.status, 200);
    assert_eq!(health.body["ok"], json!(true));
    assert_eq!(
        health.body["backend"]["enabled"],
        json!(false),
        "the degradation is on `/api/health` rather than only in the logs: {}",
        health.body["backend"],
    );

    let listed = api.get("/api/tasks").await;
    assert_eq!(listed.status, 200);
    assert_eq!(listed.body["tasks"], json!([]));
    let timeline = api.get("/api/timeline").await;
    assert_eq!(timeline.status, 200);
    assert_eq!(timeline.body["items"], json!([]));

    let _ = gateway.stop();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_broken_script_reaches_the_client_as_an_error_frame_rather_than_as_silence() {
    // The one failure a text client cannot diagnose on its own: the user types,
    // the turn starts, and nothing else ever comes. A realtime front end that
    // will not open is answered with an `error` frame, and everything else
    // keeps working — which is exactly what a Gateway whose provider endpoint
    // is unreachable does in production.
    let machine = Machine::new();
    let gateway = machine.start_harness_gateway(&script(), Some(&harness()), &[]);
    gateway.require_serving();

    // Corrupt the script *after* the process started: the opener re-reads it on
    // every open, so this is the state the first session finds.
    std::fs::write(machine.script_path(), b"{ not json").expect("corrupt the script");

    let mut client = Socket::connect(&gateway.origin(), "voice-e2e-refused").await;
    client.hello(via_i18n::Locale::En).await;
    client.say("are you there?").await;

    let started = client
        .wait_for(BUDGET, is("turn.started"))
        .await
        .expect("the turn still starts: the socket is healthy, the model is not");
    assert!(started["turnId"].as_str().is_some_and(|id| !id.is_empty()));

    let refusal = client
        .wait_for(BUDGET, is("error"))
        .await
        .expect("the refusal reaches the client rather than being logged and dropped");
    assert!(
        refusal["message"]
            .as_str()
            .is_some_and(|message| !message.is_empty()),
        "an error frame with no sentence is no better than silence: {refusal}",
    );

    // …and the Gateway is still a Gateway.
    let health = gateway.api().get("/api/health").await;
    assert_eq!(health.status, 200);
    assert_eq!(health.body["ok"], json!(true));

    let _ = gateway.stop();
}

/// A published Work must carry nothing about how the backend did the work.
///
/// `docs/architecture.md` §5 and §17 item 3: no execution mode, no delivery
/// mode, no sub-agent state, no backend permission identifier, no backend
/// topology, no cancellation internal. `via-work` enforces it by having nowhere
/// to put them; this checks the bytes that actually left the process, which is
/// the only place a future field could appear without anyone noticing.
fn assert_no_backend_internals(task: &Value) {
    let object = task.as_object().expect("a Work is an object");
    for forbidden in [
        "executionMode",
        "deliveryMode",
        "subagent",
        "subagents",
        "sessionKey",
        "backendSessionId",
        "permissionId",
        "cancellation",
        "runner",
        "canceler",
        "priority",
        "laneKey",
    ] {
        assert!(
            !object.contains_key(forbidden),
            "`{forbidden}` reached a client: {task}",
        );
    }
    // The delegation is published *without* its ids — the public shape is a
    // presentation, not an address.
    if let Some(delegation) = object.get("delegation").and_then(Value::as_object) {
        assert!(!delegation.contains_key("id"), "{task}");
        assert!(!delegation.contains_key("sessionId"), "{task}");
    }
}
