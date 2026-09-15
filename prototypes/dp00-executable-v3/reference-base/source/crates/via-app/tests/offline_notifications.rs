//! The delayed hand-off, on a paused clock.
//!
//! Ported from `server/test/offline-notifications.test.mjs`. Upstream injects a
//! `setTimer` seam so its test can fire the timer by hand; `start_paused = true`
//! advances every `tokio::time::sleep` at once, so the seam is not needed and
//! the production path is the one under test.

use std::sync::Arc;
use std::time::Duration;

use pretty_assertions::assert_eq;
use tokio::sync::mpsc;
use via_app::offline::{CHANNEL_DEPTH, OfflineNotification, install};
use via_work::testing::ImmediateRunner;
use via_work::{NewWork, NotificationClaim, WorkManager};

const DELAY: Duration = Duration::from_millis(100);

fn manager(result: &str) -> Arc<WorkManager> {
    Arc::new(
        WorkManager::builder()
            .runner(Arc::new(ImmediateRunner::completing(result)))
            .build(),
    )
}

async fn settle() {
    // Let the manager's owning task drain before the clock is advanced.
    tokio::task::yield_now().await;
    tokio::time::sleep(Duration::from_millis(1)).await;
    tokio::task::yield_now().await;
}

#[tokio::test(start_paused = true)]
async fn a_terminal_notification_nobody_claimed_reaches_the_host() {
    let work = manager("done");
    let (sink, mut notifications) = mpsc::channel::<OfflineNotification>(CHANNEL_DEPTH);
    let subscription = install(work.clone(), DELAY, sink);

    let accepted = work
        .create(NewWork::new("summarise the diff", "user_1"))
        .await
        .expect("the manager is running");
    let finished = work.wait(&accepted.work.id).await.expect("terminal");
    assert_eq!(finished.result.as_deref(), Some("done"));

    settle().await;
    tokio::time::advance(DELAY * 2).await;
    settle().await;

    let notification = notifications.try_recv().expect("one hand-off");
    assert_eq!(notification.kind, "via:offline-notification");
    assert_eq!(notification.task.id, accepted.work.id);
    assert_eq!(notification.task.objective, "summarise the diff");
    assert_eq!(notification.task.result.as_deref(), Some("done"));
    assert_eq!(notification.task.status, "completed");
    assert_eq!(
        notification.task.error,
        Some(None),
        "the terminal literal always carries the key, null included",
    );

    subscription.close().await;
}

#[tokio::test(start_paused = true)]
async fn a_notification_a_live_session_claimed_is_never_handed_off() {
    let work = manager("done");
    let (sink, mut notifications) = mpsc::channel::<OfflineNotification>(CHANNEL_DEPTH);
    let subscription = install(work.clone(), DELAY, sink);

    let accepted = work
        .create(NewWork::new("summarise the diff", "user_1"))
        .await
        .expect("the manager is running");
    let _ = work.wait(&accepted.work.id).await;
    settle().await;

    // A voice session claims it inside the delay window — which is the whole
    // reason the window exists.
    let claimed = work
        .claim_notifications(NotificationClaim::new("user_1", "main", "voice_1"))
        .await;
    assert_eq!(claimed.len(), 1);
    let delivered = work
        .mark_notifications_delivered(std::slice::from_ref(&accepted.work.id), Some("voice_1"))
        .await;
    assert!(delivered > 0, "the claim was honoured");

    tokio::time::advance(DELAY * 2).await;
    settle().await;

    assert!(
        notifications.try_recv().is_err(),
        "the re-check at fire time is the whole design",
    );

    subscription.close().await;
}

#[tokio::test(start_paused = true)]
async fn nothing_is_handed_off_before_the_delay_elapses() {
    let work = manager("done");
    let (sink, mut notifications) = mpsc::channel::<OfflineNotification>(CHANNEL_DEPTH);
    let subscription = install(work.clone(), DELAY, sink);

    let accepted = work
        .create(NewWork::new("summarise the diff", "user_1"))
        .await
        .expect("the manager is running");
    let _ = work.wait(&accepted.work.id).await;
    settle().await;

    tokio::time::advance(DELAY / 2).await;
    settle().await;
    assert!(
        notifications.try_recv().is_err(),
        "half the window is not the window",
    );

    tokio::time::advance(DELAY).await;
    settle().await;
    assert!(notifications.try_recv().is_ok());

    subscription.close().await;
}

#[tokio::test(start_paused = true)]
async fn closing_the_subscription_cancels_a_timer_that_has_not_fired() {
    let work = manager("done");
    let (sink, mut notifications) = mpsc::channel::<OfflineNotification>(CHANNEL_DEPTH);
    let subscription = install(work.clone(), DELAY, sink);

    let accepted = work
        .create(NewWork::new("summarise the diff", "user_1"))
        .await
        .expect("the manager is running");
    let _ = work.wait(&accepted.work.id).await;
    settle().await;

    // `unsubscribeOfflineNotifications?.()` on the way out.
    subscription.close().await;

    tokio::time::advance(DELAY * 2).await;
    settle().await;
    assert!(
        notifications.try_recv().is_err(),
        "a Gateway that stopped serving must not post a notification afterwards",
    );
}
