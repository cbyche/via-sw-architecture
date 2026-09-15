//! **Cancellation, across the process boundary.**
//!
//! `docs/architecture.md` §4: *"Cancellation is confirmed, not optimistic. Work
//! stays `cancelling` until a path confirms the stop."* That sentence is only
//! meaningful if `cancelling` is a state something can **observe**, and the one
//! place it is observable is the event stream of a Gateway that is a different
//! process from the thing cancelling it.
//!
//! `apps/via/tests/milestone.rs` cancels in-process and settles for
//! *"`task.cancelling` or `task.cancelled`"*, because in one process the two can
//! arrive in the same scheduler tick. Here the ordering is asserted: the socket
//! sees `cancelling` **before** `cancelled`, and `DELETE` does not answer until
//! the stop is confirmed.
//!
//! # One taxonomy, one cancel path
//!
//! `docs/architecture.md` §4 calls the unified record *"the single biggest
//! structural improvement the port brings"* — `work | reminder |
//! scheduled_task | control` in one table with a `kind` discriminator, which is
//! why *one* `cancel_agent_task` cancels a reminder and a delegation alike. The
//! second case cancels a **reminder** through the identical endpoint, with the
//! identical id shape, and asserts only the `kind` differs.

mod support;

use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use support::{BUDGET, Machine, Socket, is};
use via_protocol::{WorkKind, WorkState, WorkStatus};
use via_realtime_mock::{Script, script::turns};

/// The objective the model delegates.
const OBJECTIVE: &str = "watch the build";

/// What the reminder says.
const REMINDER: &str = "stand up";

/// A backend that would answer, if it were ever let go.
fn harness() -> Value {
    json!({
        "backend": "opencode",
        "turns": [{
            "kind": "completed",
            "content": json!({
                "work_id": "work_1",
                "state": "completed",
                "mode": "respond",
                "presentation": { "speech": "done", "inline": null },
            }).to_string(),
        }],
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn a_running_delegation_goes_cancelling_first_and_cancelled_only_on_confirmation() {
    let machine = Machine::new();
    // The backend turn is pinned open until a file appears, so the Work is
    // genuinely **live** when the cancel arrives. Cancelling something that has
    // already finished is the 409 case, and it tests nothing about states.
    let hold = machine.hold_file().display().to_string();
    let script = Script::conversation()
        .turn(turns::call_tool(
            "call_1",
            "spawn_thinking",
            &json!({ "objective": OBJECTIVE }),
        ))
        .turn(turns::say("I have started on that."));
    let gateway =
        machine.start_harness_gateway(&script, Some(&harness()), &[("VIA_E2E_HOLD", &hold)]);
    gateway.require_serving();
    let api = gateway.api();

    let mut client = Socket::connect(&gateway.origin(), "voice-e2e-cancel").await;
    client.hello(via_i18n::Locale::En).await;
    client.say("watch the build").await;

    let running = client
        .wait_for(BUDGET, is("task.running"))
        .await
        .expect("the delegation starts");
    let work_id = running["task"]["id"].as_str().expect("an id").to_owned();

    // ── the cancel ──────────────────────────────────────────────────────────
    // `DELETE` does not answer on the *request*; it answers on the confirmed
    // stop. That is why `via-work`'s canceler raises the abort before
    // reporting: a canceler that only reported would leave this request hanging
    // until the held turn timed out.
    let cancelled = api.delete(&format!("/api/tasks/{work_id}")).await;
    assert_eq!(cancelled.status, 200, "{:?}", cancelled.body);
    assert_eq!(cancelled.body["id"], work_id.as_str());
    assert_eq!(
        cancelled.body["status"],
        WorkStatus::Cancelled.as_str(),
        "the answer is the *confirmed* stop, not the request for one",
    );
    assert_eq!(cancelled.body["workState"], WorkState::Cancelled.as_str());
    assert_eq!(
        cancelled.body["error"],
        Value::Null,
        "a cancelled Work carries no error and queues no notification",
    );

    // ── and the socket saw the state in between ─────────────────────────────
    client
        .wait_for(BUDGET, is("task.cancelled"))
        .await
        .expect("the terminal frame reaches the client");
    let cancelling_at = client
        .position_of("task.cancelling")
        .expect("`cancelling` is a state, not an action: it must be published");
    let cancelled_at = client
        .position_of("task.cancelled")
        .expect("…and it must be left again once the stop is confirmed");
    assert!(
        cancelling_at < cancelled_at,
        "`cancelling` ({cancelling_at}) must precede `cancelled` ({cancelled_at}); \
         frames: {:?}",
        client
            .frames
            .iter()
            .map(|frame| frame["type"].clone())
            .collect::<Vec<_>>(),
    );

    // The held turn is released so the Gateway can close without waiting out
    // the hold's own bound.
    machine.release();
    let run = gateway.stop();
    assert_eq!(run.code, Some(0), "stderr: {}", run.stderr);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_reminder_is_cancelled_through_the_very_same_endpoint() {
    let machine = Machine::new();
    // Far enough out that the timer cannot fire during the test, and expressed
    // as RFC 3339 because that is what `schedule_reminder` parses.
    let due = chrono::Utc::now() + chrono::Duration::hours(6);
    let script = Script::conversation()
        .turn(turns::call_tool(
            "call_1",
            "schedule_reminder",
            &json!({
                "execute_at": due.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                "reminder": REMINDER,
            }),
        ))
        .turn(turns::say("I will remind you."));
    let gateway = machine.start_harness_gateway(&script, Some(&harness()), &[]);
    gateway.require_serving();
    let api = gateway.api();

    let session = "voice-e2e-reminder";
    let mut client = Socket::connect(&gateway.origin(), session).await;
    client.hello(via_i18n::Locale::En).await;
    client.say("remind me to stand up in six hours").await;

    let scheduled = client
        .wait_for(BUDGET, is("task.scheduled"))
        .await
        .expect("a reminder is accepted and waits for its timer");
    let reminder_id = scheduled["task"]["id"].as_str().expect("an id").to_owned();
    assert_eq!(scheduled["task"]["kind"], WorkKind::Reminder.as_str());
    assert_eq!(scheduled["task"]["status"], WorkStatus::Scheduled.as_str());
    assert_eq!(
        scheduled["task"]["workState"],
        WorkState::Scheduled.as_str(),
        "`scheduled` reaches the wire because it is not one of the five statuses \
         `ACTIVE` collapses",
    );
    assert_eq!(scheduled["task"]["objective"], REMINDER);

    // One table, one id shape: a reminder is listed beside a delegation and is
    // addressed the same way.
    let listed = api.get(&format!("/api/tasks?sessionId={session}")).await;
    assert_eq!(listed.status, 200);
    let tasks = listed.body["tasks"].as_array().expect("a list");
    assert_eq!(tasks.len(), 1, "{:?}", listed.body);
    assert_eq!(tasks[0]["id"], reminder_id.as_str());
    assert_eq!(tasks[0]["kind"], WorkKind::Reminder.as_str());
    assert!(
        reminder_id.starts_with("work_"),
        "one record, four kinds — a reminder carries a `work_` id like everything \
         else: {reminder_id}",
    );

    // ── the same route, the same verb ───────────────────────────────────────
    let cancelled = api.delete(&format!("/api/tasks/{reminder_id}")).await;
    assert_eq!(cancelled.status, 200, "{:?}", cancelled.body);
    assert_eq!(cancelled.body["id"], reminder_id.as_str());
    assert_eq!(cancelled.body["kind"], WorkKind::Reminder.as_str());
    assert_eq!(
        cancelled.body["status"],
        WorkStatus::Cancelled.as_str(),
        "nothing is running, so the stop confirms at once",
    );

    // A `scheduled` Work has nothing in flight, so it never passes through
    // `cancelling` — the state exists for work that has to be stopped, not for
    // work that has not started.
    let terminal = client
        .wait_for(BUDGET, is("task.cancelled"))
        .await
        .expect("the client is told");
    assert_eq!(terminal["task"]["id"], reminder_id.as_str());
    assert!(
        client.position_of("task.cancelling").is_none(),
        "frames: {:?}",
        client
            .frames
            .iter()
            .map(|frame| frame["type"].clone())
            .collect::<Vec<_>>(),
    );

    // …and cancelling it twice is the catalogued 409, not a second cancellation.
    let again = api.delete(&format!("/api/tasks/{reminder_id}")).await;
    assert_eq!(again.status, 409);
    assert_eq!(
        again.body["error"],
        via_i18n::t(
            via_i18n::Locale::En,
            via_i18n::keys::GATEWAY_TASK_NOT_ACTIVE
        ),
    );

    let _ = gateway.stop();
}
