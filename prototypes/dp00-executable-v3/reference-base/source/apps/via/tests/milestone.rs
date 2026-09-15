//! **The phase-5 milestone.**
//!
//! `docs/architecture.md` §15: phase 5 ends when *"`via chat` works end to end
//! — no audio hardware, no model weights"*, and §15's prose calls it *"the
//! whole core exercised end to end"*. This file is that sentence, executed.
//!
//! One Gateway, booted through [`via::gateway::boot`] — the same call
//! `via gateway` makes — with the mock realtime provider in front of the model
//! and a [`ScriptedHarness`](via_downstream::testing::ScriptedHarness) behind
//! the coordinator. Everything between them is production code: the real
//! [`via_realtime::RealtimeSession`], the real
//! [`via_voice::ToolCallHandler`], the real [`via_work::WorkManager`] and its
//! admission scheduler, the real [`via_coordinator::Coordinator`], and the real
//! [`via_voice::AnnouncementManager`].
//!
//! The client is `via chat`'s own frames — [`via::chat::protocol`] composes
//! them here exactly as the REPL composes them there, so what is asserted is
//! the client a user runs rather than a re-derivation of it.
//!
//! # What the milestone actually demands
//!
//! 1. a text message the script answers **directly** — the fast path, no Work;
//! 2. a text message that reaches `spawn_thinking`;
//! 3. the Work moving `queued → running → completed`;
//! 4. the result delivered back **into the conversation, through the
//!    announcement window**.
//!
//! Point 4 is the one that is easy to fake and the whole reason the Injection
//! Gate exists, so it is asserted twice: once on the socket (the user hears
//! it) and once against the mock's own transcript (the result reached the
//! model's conversation as an item, which is what
//! `config.announceIntoContext` promises).

mod support;

use std::sync::Arc;
use std::time::Duration;

use pretty_assertions::assert_eq;
use serde_json::json;
use support::gateway::{Client, Harness, HeldHarness, is};
use via_downstream::testing::{ScriptedHarness, ScriptedTurn};
use via_realtime_mock::{Script, script::turns};

/// What the script says to the first message.
const DIRECT_ANSWER: &str = "Nothing is running right now.";

/// What the script says once `spawn_thinking` has been accepted.
const ACKNOWLEDGEMENT: &str = "I have started on that.";

/// What the script says when the finished result is injected.
const ANNOUNCEMENT: &str = "That is done: the tests pass.";

/// What the backend answers with.
const RESULT: &str = "The tests pass.";

/// The objective the model delegates.
const OBJECTIVE: &str = "run the test suite and report";

/// How long a Work item is given to travel the whole queue.
///
/// Generous on purpose: the path crosses four owning tasks (the session, the
/// tool handler, the Work manager and the coordinator's keyed serial executor)
/// and one scheduler admission, and a tight budget here buys nothing but flake
/// on a loaded machine.
const BUDGET: Duration = Duration::from_secs(20);

/// The script the realtime model replays.
///
/// Four turns, scanned in declaration order, each firing once — which is
/// `via-realtime-mock`'s rule 2: *"a script of three `Trigger::ResponseCreate`
/// steps is three consecutive model turns, which is how a tool round trip is
/// written."*
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

/// The backend the coordinator delegates to.
fn harness() -> Arc<dyn via_downstream::DownstreamAgent> {
    Arc::new(
        ScriptedHarness::builder("opencode")
            .turn(ScriptedTurn::completed(
                &json!({
                    "work_id": "work_1",
                    "state": "completed",
                    "mode": "respond",
                    "presentation": { "speech": RESULT, "inline": null },
                })
                .to_string(),
            ))
            .build()
            .expect("the scripted harness declares consistently"),
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn via_chat_works_end_to_end_with_no_audio_and_no_model_weights() {
    let mut gateway = Harness::start(script(), harness()).await;
    gateway.serve();

    let mut client = Client::connect(&gateway.origin, "voice-milestone").await;
    client.hello().await;

    // ── 1. the fast path ────────────────────────────────────────────────────
    client.say("is anything running?").await;
    let answered = client
        .wait_for(BUDGET, |frame| {
            frame["type"] == "transcript.final"
                && frame["role"] == "assistant"
                && frame["content"] == DIRECT_ANSWER
        })
        .await
        .expect("the model answers the first message itself");
    assert!(
        answered["turnId"].as_str().is_some_and(|id| !id.is_empty()),
        "every assistant transcript is correlated to the turn that asked for it",
    );
    assert!(
        client.seen("task.running").is_empty(),
        "a direct answer creates no Work: `docs/architecture.md` §4, \
         *direct answers are answered on the fast path; no task is created*",
    );

    // ── 2. the delegation ───────────────────────────────────────────────────
    client.say("run the tests").await;

    // ── 3. queued → running → completed ─────────────────────────────────────
    // `task.queued` has no counterpart in the client vocabulary — the Work is
    // *created* queued and the first frame a client sees is `task.running`
    // (`via_protocol::GatewayTaskEvent`). What proves it passed through
    // `queued` is the acceptance receipt the tool call returns, which is the
    // Work's status at creation; the socket's evidence is the pair below.
    let running = client
        .wait_for(BUDGET, is("task.running"))
        .await
        .expect("the scheduler admits the delegation");
    let work_id = running["task"]["id"]
        .as_str()
        .expect("a Work event names its Work")
        .to_owned();
    assert_eq!(running["task"]["status"], "running");
    assert_eq!(running["task"]["objective"], OBJECTIVE);
    assert_eq!(running["task"]["sessionId"], "voice-milestone");

    let completed = client
        .wait_for(BUDGET, is("task.completed"))
        .await
        .expect("the coordinator's answer completes the Work");
    assert_eq!(completed["task"]["id"], work_id.as_str());
    assert_eq!(completed["task"]["status"], "completed");
    assert_eq!(
        completed["task"]["result"], RESULT,
        "the Work carries the backend's own words",
    );

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
            .is_some_and(|id| !id.is_empty())
    );

    // …and it reached the model's own conversation as an item, not only the
    // socket. `config.announceIntoContext` is what promises that, and the
    // mock's transcript is the only place it is observable.
    let handle = gateway.opener.handle().await.expect("a session was opened");
    let transcript = handle
        .transcript()
        .await
        .expect("the script server is alive");
    let injected = transcript
        .frames_of("conversation.item.create")
        .iter()
        .any(|frame| frame.to_string().contains(RESULT));
    assert!(
        injected,
        "the result must be written into the conversation, not merely spoken: \
         `docs/architecture.md` §11's third invariant. Frames: {:?}",
        transcript.outbound_kinds(),
    );

    // The whole conversation, as a user of `via chat` read it.
    let text = client.assistant_text();
    assert!(text.contains(DIRECT_ANSWER), "{text}");
    assert!(text.contains(ACKNOWLEDGEMENT), "{text}");
    assert!(text.contains(ANNOUNCEMENT), "{text}");
}

#[tokio::test(flavor = "multi_thread")]
async fn tasks_and_cancel_answer_against_a_running_gateway() {
    // The delegation is held open, so `/tasks` sees something `running` and
    // `/cancel` acts on a Work that has not already finished — which is the
    // only version of this that tests anything.
    let script = Script::conversation()
        .turn(turns::call_tool(
            "call_1",
            "spawn_thinking",
            &json!({ "objective": OBJECTIVE }),
        ))
        .turn(turns::say(ACKNOWLEDGEMENT));
    let held = HeldHarness::new(harness());

    let mut gateway = Harness::start(
        script,
        Arc::clone(&held) as Arc<dyn via_downstream::DownstreamAgent>,
    )
    .await;
    gateway.serve();
    let session_id = "voice-cancel";
    let mut client = Client::connect(&gateway.origin, session_id).await;
    client.hello().await;
    client.say("run the tests").await;
    client
        .wait_for(BUDGET, is("task.running"))
        .await
        .expect("the delegation starts");

    // `/tasks` — the same request the command issues, through the same client.
    let mut http =
        via::chat::GatewayClient::new(&gateway.origin, via_i18n::Locale::En).expect("a client");
    http.health().await.expect("health").expect("a Gateway");
    let tasks = http.tasks(session_id).await.expect("the task list");
    assert!(!tasks.is_empty(), "the delegation is listed");
    let line = via::chat::protocol::task_line(&tasks[0]);
    assert!(line.starts_with(&tasks[0].id), "{line}");
    assert!(line.contains(OBJECTIVE), "{line}");
    assert!(
        line.contains(&std::format!("  {}  ", tasks[0].status.as_str())),
        "{line}"
    );

    // `/cancel` with no id picks the first cancellable one.
    let target = via::chat::protocol::select_cancellable(&tasks, None)
        .expect("a running delegation is cancellable")
        .id
        .clone();
    http.cancel(&target).await.expect("the cancel is accepted");

    // Cancellation is confirmed, not optimistic: the Work goes to
    // `cancelling` and only a confirmed stop makes it `cancelled`
    // (`docs/architecture.md` §4).
    let moved = client
        .wait_for(BUDGET, |frame| {
            frame["type"] == "task.cancelling" || frame["type"] == "task.cancelled"
        })
        .await
        .expect("the cancel reaches the Work plane");
    assert_eq!(moved["task"]["id"], target.as_str());
    held.release();
}

#[tokio::test(flavor = "multi_thread")]
async fn cancelling_a_finished_task_is_the_catalogued_409() {
    // The other half of `/cancel`: `DELETE /api/tasks/:id` answers
    // `{"error":"task is no longer active", …}` with 409 for a terminal Work
    // (`docs/reference/contracts.json`, http-route *Work HTTP surface*), and
    // `via chat` prints it as `[command failed] …`.
    let mut gateway = Harness::start(script(), harness()).await;
    gateway.serve();
    let session_id = "voice-terminal";
    let mut client = Client::connect(&gateway.origin, session_id).await;
    client.hello().await;
    client.say("is anything running?").await;
    client.say("run the tests").await;
    let completed = client
        .wait_for(BUDGET, is("task.completed"))
        .await
        .expect("the delegation finishes");
    let work_id = completed["task"]["id"].as_str().expect("an id").to_owned();

    let mut http =
        via::chat::GatewayClient::new(&gateway.origin, via_i18n::Locale::En).expect("a client");
    http.health().await.expect("health").expect("a Gateway");
    let tasks = http.tasks(session_id).await.expect("the task list");
    assert!(
        via::chat::protocol::select_cancellable(&tasks, None).is_none(),
        "nothing terminal is cancellable, so `/cancel` says so rather than picking one",
    );
    let error = http
        .cancel(&work_id)
        .await
        .expect_err("a finished Work cannot be cancelled");
    assert_eq!(
        error.message(via_i18n::Locale::En),
        via_i18n::t(
            via_i18n::Locale::En,
            via_i18n::keys::GATEWAY_TASK_NOT_ACTIVE
        ),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_task_that_does_not_exist_is_a_404_not_a_403() {
    // The catalogue is explicit: *"a task belonging to another owner reads as
    // 404, not 403."* An id nobody owns is the same answer.
    let mut gateway = Harness::start(Script::conversation(), harness()).await;
    gateway.serve();
    let mut http =
        via::chat::GatewayClient::new(&gateway.origin, via_i18n::Locale::En).expect("a client");
    http.health().await.expect("health").expect("a Gateway");
    let error = http
        .cancel("work_nonesuch")
        .await
        .expect_err("nothing answers to that id");
    assert!(
        error.message(via_i18n::Locale::En).contains("not found"),
        "{}",
        error.message(via_i18n::Locale::En),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_realtime_front_end_that_will_not_open_is_reported_rather_than_silent() {
    // The one failure a text client cannot diagnose on its own: the user types,
    // the turn starts, and nothing else ever comes. Upstream answers a failed
    // `ensureFrontend()` with `send(ws, {type: 'error', message})`
    // (`realtime-gateway.mjs:508`), and `via chat` renders it as `[error] …`.
    //
    // Everything else still works — `docs/deviations/phase-5-via-app.md` is
    // explicit that a Gateway with no usable realtime binding *"still serves
    // `/api/health`, still relays input suspensions, and still forwards the
    // Work plane"* — so the socket stays open and the turn still starts.
    let mut gateway = Harness::refusing_realtime(harness()).await;
    gateway.serve();
    let mut client = Client::connect(&gateway.origin, "voice-refused").await;
    client.hello().await;
    client.say("are you there?").await;

    let started = client
        .wait_for(BUDGET, is("turn.started"))
        .await
        .expect("the turn still starts: the socket is healthy, the model is not");
    assert!(started["turnId"].as_str().is_some_and(|id| !id.is_empty()));

    let refusal = client
        .wait_for(BUDGET, is("error"))
        .await
        .expect("…and the refusal reaches the client rather than being logged and dropped");
    assert!(
        refusal["message"]
            .as_str()
            .is_some_and(|message| !message.is_empty()),
        "an error frame with no sentence is no better than silence: {refusal}",
    );
}
