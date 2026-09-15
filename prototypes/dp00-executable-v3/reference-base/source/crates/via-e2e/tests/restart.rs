//! **Restart recovery** — SIGKILL, then read `tasks.json` back with a new
//! process.
//!
//! `via-work/tests/restart.rs` already drives `restore()` against a
//! synthetic store, and `via-work/tests/contracts.rs` already compares both
//! restart sentences to the catalogue byte for byte. Neither is repeated. What
//! only a process can show is the join between them: that a Work interrupted by
//! a **real** crash is written to a **real** `tasks.json` by one process and
//! read back, force-failed and re-published over HTTP by a **different** one,
//! carrying the sentence the catalogue names.
//!
//! # The two sentences, and why the distinction matters
//!
//! The catalogue holds two:
//!
//! | Contract | When |
//! | --- | --- |
//! | *restart force-fail error (interactive work)* | any active Work that cannot be recovered |
//! | *restart force-fail error (unrecoverable delegated work)* | a `delegated` Work that **was** addressable, offered to the recovery seam, and declined |
//!
//! They are different sentences because they describe different losses — one
//! never started with a backend, the other lost a backend session that is still
//! out there. A build that collapsed them would tell a user to resubmit work
//! that is still running somewhere. [`the_two_restart_reasons_stay_distinct`]
//! pins the distinction, and
//! `docs/deviations/phase-9-via-e2e.md` records that the *delegated* arm is not
//! reachable through `via gateway` today, because nothing calls
//! `WorkManager::recover_delegated`.

mod support;

use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use support::{BUDGET, FILE_BUDGET, Machine, Socket, is, wait_until};
use via_i18n::{Locale, keys, t};
use via_protocol::{WorkState, WorkStatus};
use via_realtime_mock::{Script, script::turns};

/// The objective the model delegates.
const OBJECTIVE: &str = "reindex everything";

/// What a backend that is allowed to answer says.
const RESULT: &str = "Reindexed.";

/// A backend with one answering turn.
fn harness() -> Value {
    json!({
        "backend": "opencode",
        "turns": [{
            "kind": "completed",
            "content": json!({
                "work_id": "work_1",
                "state": "completed",
                "mode": "respond",
                "presentation": { "speech": RESULT, "inline": null },
            }).to_string(),
        }],
    })
}

/// A backend with nothing to say — what the *second* process runs, because the
/// Work it inherits must be force-failed rather than re-run.
fn silent_harness() -> Value {
    json!({ "backend": "opencode", "turns": [] })
}

/// A script that delegates once and acknowledges.
fn delegating_script() -> Script {
    Script::conversation()
        .turn(turns::call_tool(
            "call_1",
            "spawn_thinking",
            &json!({ "objective": OBJECTIVE }),
        ))
        .turn(turns::say("Working on it."))
        .turn(turns::say("That is done."))
}

/// The catalogued sentence for `name`, with the product rename applied.
fn restart_sentence(name: &str) -> String {
    support::catalogued("prompt-text", name)
}

#[tokio::test(flavor = "multi_thread")]
async fn work_a_crash_interrupted_is_force_failed_with_the_catalogued_reason() {
    let machine = Machine::new();

    // ── 1. a Gateway with a backend turn that never returns ─────────────────
    // The hold file is never created, so the delegation is still live when the
    // process dies. That is the whole premise: a Work that was *finished* would
    // be restored untouched and prove nothing.
    let hold = machine.hold_file().display().to_string();
    let first = machine.start_harness_gateway(
        &delegating_script(),
        Some(&harness()),
        &[("VIA_E2E_HOLD", &hold)],
    );
    first.require_serving();

    let session = "voice-e2e-restart";
    let mut client = Socket::connect(&first.origin(), session).await;
    client.hello(Locale::En).await;
    client.say("reindex everything").await;
    let running = client
        .wait_for(BUDGET, is("task.running"))
        .await
        .expect("the delegation starts");
    let work_id = running["task"]["id"].as_str().expect("an id").to_owned();

    // ── 2. …and it reaches the disk ─────────────────────────────────────────
    // `via-work` coalesces saves on a 250 ms timer, so the assertion has to
    // wait for the write rather than assume it. Killing before the write would
    // test the store's durability window, which is a different property and
    // belongs to `via-store`.
    let persisted = wait_until(FILE_BUDGET, || {
        machine
            .persisted_tasks()
            .iter()
            .any(|task| task["id"] == work_id.as_str())
    })
    .await;
    assert!(persisted, "`tasks.json` never carried {work_id}");
    let before = machine
        .persisted_tasks()
        .into_iter()
        .find(|task| task["id"] == work_id.as_str())
        .expect("the record just asserted");
    let saved_status = before["status"].as_str().unwrap_or_default().to_owned();
    assert!(
        matches!(
            saved_status.as_str(),
            "queued" | "running" | "delegated" | "finalizing"
        ),
        "the persisted Work must still be active for the restart to have anything to \
         recover: {before}",
    );

    // ── 3. the crash ────────────────────────────────────────────────────────
    // SIGKILL: no `taskStore.flush()`, no lease release, no close sequence.
    let killed = first.kill();
    assert_eq!(
        killed.code, None,
        "a SIGKILLed process runs nothing on its way out"
    );
    drop(client);

    // ── 4. a new process reads it back ──────────────────────────────────────
    // `zh` so the sentence can be compared with the catalogue's own bytes.
    let second = machine.start_harness_gateway(
        &Script::conversation(),
        Some(&silent_harness()),
        &[("VIA_LOCALE", "zh")],
    );
    second.require_serving();
    assert_eq!(
        machine.lease().map(|lease| lease.pid),
        Some(second.pid()),
        "the stale lease was reclaimed on the way in",
    );

    let listed = second
        .api()
        .get(&format!("/api/tasks?sessionId={session}"))
        .await;
    assert_eq!(listed.status, 200);
    let recovered = listed.body["tasks"]
        .as_array()
        .and_then(|tasks| tasks.iter().find(|task| task["id"] == work_id.as_str()))
        .cloned()
        .unwrap_or_else(|| panic!("the Work did not survive the crash: {:?}", listed.body));

    assert_eq!(
        recovered["status"],
        WorkStatus::Failed.as_str(),
        "*a persisted `queued` or `running` task becomes status `failed`* — nothing \
         survived that knows what the backend was doing, so it is force-failed rather \
         than silently resurrected",
    );
    assert_eq!(recovered["workState"], WorkState::Failed.as_str());
    assert_eq!(
        recovered["error"],
        restart_sentence("restart force-fail error (interactive work)"),
        "the reason is the catalogued sentence, in the locale the *reading* process \
         runs in",
    );
    assert_ne!(
        recovered["error"],
        restart_sentence("restart force-fail error (unrecoverable delegated work)"),
        "the two reasons describe different losses and must not collapse",
    );
    assert_eq!(
        recovered["notificationStatus"], "pending",
        "*guarantees the user still hears about work that a crash interrupted*",
    );
    assert!(
        recovered["completedAt"].is_i64(),
        "a force-failed Work is terminal, and a terminal Work has an end: {recovered}",
    );
    // Everything the user supplied is still theirs.
    assert_eq!(recovered["objective"], OBJECTIVE);
    assert_eq!(recovered["sessionId"], session);
    assert_eq!(recovered["ownerId"], before["ownerId"]);
    assert_eq!(recovered["createdAt"], before["createdAt"]);

    let _ = second.stop();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_finished_result_and_its_notification_state_survive_the_crash_untouched() {
    let machine = Machine::new();
    let session = "voice-e2e-survives";

    // ── 1. a delegation that actually completes, and is announced ───────────
    let first = machine.start_harness_gateway(&delegating_script(), Some(&harness()), &[]);
    first.require_serving();
    let mut client = Socket::connect(&first.origin(), session).await;
    client.hello(Locale::En).await;
    client.say("reindex everything").await;
    let completed = client
        .wait_for(BUDGET, is("task.completed"))
        .await
        .expect("the delegation finishes");
    let work_id = completed["task"]["id"].as_str().expect("an id").to_owned();
    assert_eq!(completed["task"]["result"], RESULT);

    // The terminal record has to be on disk before the crash, or this asserts
    // the coalescing window rather than the restore. What the *next* process
    // reads is the file, so the file is what is captured — not the HTTP view,
    // which is a projection of memory the crash is about to destroy.
    let persisted = wait_until(FILE_BUDGET, || {
        machine
            .persisted_tasks()
            .iter()
            .any(|task| task["id"] == work_id.as_str() && task["status"] == "completed")
    })
    .await;
    assert!(persisted, "`tasks.json` never carried the finished Work");
    let before = machine
        .persisted_tasks()
        .into_iter()
        .find(|task| task["id"] == work_id.as_str())
        .expect("the record just asserted");
    assert_eq!(before["result"], RESULT);
    let notification = before["notificationStatus"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert!(
        !notification.is_empty(),
        "a completed Work carries a notification state: {before}",
    );

    // ── 2. crash, and read it back ──────────────────────────────────────────
    let _ = first.kill();
    drop(client);

    let second =
        machine.start_harness_gateway(&Script::conversation(), Some(&silent_harness()), &[]);
    second.require_serving();
    let after = second.api().get(&format!("/api/tasks/{work_id}")).await;
    assert_eq!(after.status, 200, "{:?}", after.body);
    assert_eq!(
        after.body["status"],
        WorkStatus::Completed.as_str(),
        "a Work that was already terminal is restored, not force-failed",
    );
    assert_eq!(
        after.body["result"], RESULT,
        "the backend's own words survive the crash",
    );
    assert_eq!(after.body["error"], Value::Null);
    assert_eq!(after.body["objective"], OBJECTIVE);

    // ── the notification state survives, with one deliberate exception ──────
    // A *claim* cannot outlive the process that made it: `delivering` means
    // some claimant leased this result and was going to speak it, and that
    // claimant is now dead. `restore()` re-queues it as `pending` so the user
    // is still told; anything else silently loses a finished result. Every
    // other state is carried across untouched.
    let restored = after.body["notificationStatus"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert_ne!(
        restored, "delivering",
        "a delivery claim must not survive the process that made it: {:?}",
        after.body,
    );
    if notification == "delivering" {
        assert_eq!(
            restored, "pending",
            "an in-flight claim is re-queued rather than dropped",
        );
    } else {
        assert_eq!(
            restored, notification,
            "a notification state nothing was holding is carried across unchanged",
        );
    }
    assert!(
        matches!(restored.as_str(), "pending" | "delivered"),
        "{restored}",
    );

    let _ = second.stop();
}

#[test]
fn the_two_restart_reasons_stay_distinct() {
    // Both are catalogued `prompt-text` rows, both reach `task.error`, both are
    // persisted, and both are spoken to the user. `via-work` compares each to
    // the catalogue; what is asserted here is the property that makes having
    // two of them worth anything — that they are two.
    let interactive = restart_sentence("restart force-fail error (interactive work)");
    let delegated = restart_sentence("restart force-fail error (unrecoverable delegated work)");
    assert_ne!(interactive, delegated);
    assert_eq!(
        interactive,
        t(Locale::Zh, keys::WORK_RESTART_INTERACTIVE_INCOMPLETE)
    );
    assert_eq!(delegated, t(Locale::Zh, keys::WORK_RESTART_DELEGATED_LOST));

    // …and the rename is not a no-op: both catalogued values name the upstream
    // product, and neither shipped one does.
    for name in [
        "restart force-fail error (interactive work)",
        "restart force-fail error (unrecoverable delegated work)",
    ] {
        assert!(
            support::catalogued_raw("prompt-text", name).contains("qwen-audio-agent"),
            "the catalogue moved: {name}",
        );
    }
    for shipped in [&interactive, &delegated] {
        assert!(!shipped.contains("qwen"), "{shipped}");
        assert!(shipped.starts_with("VIA "), "{shipped}");
    }
}
