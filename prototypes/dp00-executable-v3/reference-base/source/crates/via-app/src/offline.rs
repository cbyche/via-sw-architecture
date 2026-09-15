//! Offline notifications — the delayed hand-off to the host.
//!
//! A full port of `server/src/app/offline-notifications.mjs`, all 44 lines of
//! it.
//!
//! The idea is one sentence: *if a voice session does not claim a pending
//! notification within the delay window, deliver it some other way.* Upstream's
//! other way is `parentPort.postMessage` to an Electron host, which shows an OS
//! notification. VIA has no Electron host, so the message becomes
//! [`OfflineNotification`] on a [`tokio::sync::mpsc`] channel and whoever
//! embeds the Gateway decides what to do with it.
//!
//! # The re-check is the whole design
//!
//! Both arms fire a timer and then **look the Work up again**. The delay exists
//! precisely so a live session can claim first, so the state at *fire* time is
//! the only state that matters:
//!
//! | Event | Re-check | Payload |
//! | --- | --- | --- |
//! | `task.progress.check` | `workState === 'active'` | `{id, objective, result: <the event's message>, status: 'progress'}` |
//! | `task.notification.pending` | `notificationStatus === 'pending'` | `{id, objective, result, error, status: <task.status>}` |
//!
//! Two different payload shapes under one message type, and the progress
//! variant carries the **subscription event's `message`** as `result` while
//! hard-coding `status: 'progress'`. Both are catalogued
//! (`docs/reference/contracts.json`, `ipc-message` / *offline notification*).
//!
//! A Work that finished during the window produces nothing at all — upstream's
//! own test is named *"drops delayed progress after the task has completed"*.

use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tokio::sync::mpsc;
use tokio_util::task::TaskTracker;
use via_protocol::WorkState;
use via_work::{NotificationStatus, WorkEventDetails, WorkEventKind, WorkManager};

/// The message type both payloads travel under.
///
/// **External contract** — `offline-notifications.mjs:20,36`. Upstream's string
/// is `qwen-audio-agent:offline-notification`; `docs/rebrand.md` renames the
/// product prefix and keeps the suffix.
pub const OFFLINE_NOTIFICATION_TYPE: &str = "via:offline-notification";

/// The `status` a progress hand-off carries.
///
/// **External contract** — `offline-notifications.mjs:25`: hard-coded
/// `'progress'`, not the Work's own status.
pub const PROGRESS_STATUS: &str = "progress";

/// How many hand-offs may queue before the oldest is dropped.
pub const CHANNEL_DEPTH: usize = 64;

/// One hand-off to the host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OfflineNotification {
    /// [`OFFLINE_NOTIFICATION_TYPE`].
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// The Work, in one of the two shapes.
    pub task: OfflineTask,
}

/// The two payload shapes, as one type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OfflineTask {
    /// The Work id.
    pub id: String,
    /// The user's request.
    pub objective: String,
    /// The progress message, or the final result.
    pub result: Option<String>,
    /// The final error.
    ///
    /// Three states, not two, because upstream has three: the progress literal
    /// has **no `error` key at all**, while the terminal literal always has one
    /// and writes `null` when the Work did not fail. `None` is the absent key,
    /// `Some(None)` is `null`, `Some(Some(_))` is the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<Option<String>>,
    /// `progress`, or the Work's own terminal status.
    pub status: String,
}

/// A running offline-notification subscriber.
///
/// Dropping it stops the subscription, which is
/// `unsubscribeOfflineNotifications?.()` in
/// `gateway-application.mjs:512`'s close path.
#[derive(Debug)]
pub struct OfflineNotifications {
    tracker: TaskTracker,
    cancel: tokio_util::sync::CancellationToken,
}

impl OfflineNotifications {
    /// Stop the subscription and wait for its timers to unwind.
    pub async fn close(self) {
        self.cancel.cancel();
        self.tracker.close();
        self.tracker.wait().await;
    }
}

/// Subscribe, and deliver anything nobody claimed within `delay`.
///
/// Upstream's `installOfflineNotifications({taskManager, parentPort, delayMs})`.
/// The `setTimer` seam its test injects is not needed here: every timer is a
/// `tokio::time::sleep`, and `#[tokio::test(start_paused = true)]` advances all
/// of them without a seam.
#[must_use]
pub fn install(
    work: Arc<WorkManager>,
    delay: Duration,
    sink: mpsc::Sender<OfflineNotification>,
) -> OfflineNotifications {
    let tracker = TaskTracker::new();
    let cancel = tokio_util::sync::CancellationToken::new();
    let mut events = work.subscribe();

    let subscriber_cancel = cancel.clone();
    let timers = tracker.clone();
    tracker.spawn(async move {
        loop {
            let event = tokio::select! {
                () = subscriber_cancel.cancelled() => break,
                event = events.recv() => event,
            };
            let Ok(event) = event else {
                // `Lagged` drops edges; a missed notification is re-raised by
                // the manager's own retention pass, and stopping the
                // subscription here would lose every later one.
                continue;
            };
            let arm = match event.kind {
                WorkEventKind::ProgressCheck => Arm::Progress {
                    message: match &event.details {
                        WorkEventDetails::ProgressCheck { message, .. } => message.clone(),
                        _ => String::new(),
                    },
                },
                WorkEventKind::NotificationPending => Arm::Terminal,
                _ => continue,
            };
            let work = work.clone();
            let sink = sink.clone();
            let owner_id = event.owner_id.clone();
            let work_id = event.task.id.clone();
            let fired = subscriber_cancel.clone();
            timers.spawn(async move {
                tokio::select! {
                    () = fired.cancelled() => return,
                    () = tokio::time::sleep(delay) => {}
                }
                // The re-check: the delay exists so a live session can claim
                // first, so only the state at fire time counts.
                let Some(current) = work.get(&work_id, Some(&owner_id)).await else {
                    return;
                };
                let notification = match arm {
                    Arm::Progress { message } => {
                        if current.work_state != WorkState::Active {
                            return;
                        }
                        OfflineNotification {
                            kind: OFFLINE_NOTIFICATION_TYPE,
                            task: OfflineTask {
                                id: current.id,
                                objective: current.objective,
                                result: Some(message),
                                error: None,
                                status: PROGRESS_STATUS.to_owned(),
                            },
                        }
                    }
                    Arm::Terminal => {
                        if current.notification_status != NotificationStatus::Pending {
                            return;
                        }
                        OfflineNotification {
                            kind: OFFLINE_NOTIFICATION_TYPE,
                            task: OfflineTask {
                                id: current.id,
                                objective: current.objective,
                                result: current.result,
                                error: Some(current.error),
                                status: current.status.as_str().to_owned(),
                            },
                        }
                    }
                };
                let _ = sink.try_send(notification);
            });
        }
    });

    OfflineNotifications { tracker, cancel }
}

enum Arm {
    Progress { message: String },
    Terminal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn the_progress_payload_has_no_error_key_at_all() {
        let notification = OfflineNotification {
            kind: OFFLINE_NOTIFICATION_TYPE,
            task: OfflineTask {
                id: "work_1".to_owned(),
                objective: "build".to_owned(),
                result: Some("still working".to_owned()),
                error: None,
                status: PROGRESS_STATUS.to_owned(),
            },
        };
        assert_eq!(
            serde_json::to_value(&notification).expect("serializes"),
            json!({
                "type": "via:offline-notification",
                "task": {
                    "id": "work_1",
                    "objective": "build",
                    "result": "still working",
                    "status": "progress",
                },
            }),
        );
    }

    #[test]
    fn the_terminal_payload_carries_error_even_when_it_is_null() {
        let notification = OfflineNotification {
            kind: OFFLINE_NOTIFICATION_TYPE,
            task: OfflineTask {
                id: "work_1".to_owned(),
                objective: "build".to_owned(),
                result: Some("done".to_owned()),
                error: Some(None),
                status: "completed".to_owned(),
            },
        };
        let value = serde_json::to_value(&notification).expect("serializes");
        assert_eq!(
            value["task"]["error"],
            serde_json::Value::Null,
            "upstream's terminal literal always has the key, null included",
        );
        assert!(
            value["task"]
                .as_object()
                .is_some_and(|task| task.contains_key("error")),
        );
    }

    #[test]
    fn the_message_type_is_rebranded_and_the_suffix_is_kept() {
        assert_eq!(OFFLINE_NOTIFICATION_TYPE, "via:offline-notification");
        assert!(OFFLINE_NOTIFICATION_TYPE.ends_with(":offline-notification"));
    }
}
