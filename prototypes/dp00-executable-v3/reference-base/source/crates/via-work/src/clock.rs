//! The injectable epoch-millisecond clock.
//!
//! Upstream reads `Date.now()` in eleven places — the Work id's `createdAt`,
//! `elapsedMs`, every TTL, the notification lease, the quarantine filename and
//! the reminder scheduler's due test. All of them are observable, so all of
//! them take their reading from here.
//!
//! The reason it is a parameter rather than a call is
//! `#[tokio::test(start_paused = true)]`: paused time advances
//! [`tokio::time::Instant`] but never [`std::time::SystemTime`], so a manager
//! built on the system clock would sit at one instant while its timers ran a
//! week ahead. [`tokio_clock`] is the clock that agrees with the timers.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Epoch milliseconds.
///
/// The same shape as [`via_store::NowFn`], so one clock drives both the manager
/// and the store it writes through.
pub type NowFn = Arc<dyn Fn() -> i64 + Send + Sync>;

/// The wall clock.
///
/// `Date.now()`. Saturates rather than wrapping for an unrepresentable instant,
/// and answers `0` for a system clock set before 1970 — neither is reachable in
/// practice, and both keep the crate free of `unwrap`.
#[must_use]
pub fn system_now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// The default clock: [`system_now_ms`] behind an [`Arc`].
#[must_use]
pub fn system_clock() -> NowFn {
    Arc::new(system_now_ms)
}

/// A clock anchored to [`tokio::time::Instant`], which advances with paused
/// time.
///
/// `base_ms` is the epoch-millisecond reading the moment this is called; every
/// later reading is `base_ms` plus however far tokio's clock has moved. Under
/// `#[tokio::test(start_paused = true)]` that means auto-advance moves the
/// Work subsystem's idea of "now" in lockstep with its timers, so a 24-hour
/// retention ladder runs in microseconds and deterministically.
///
/// # Panics
///
/// [`tokio::time::Instant::now`] requires a tokio runtime context. Call this
/// from inside one — which is where a manager is built.
#[must_use]
pub fn tokio_clock(base_ms: i64) -> NowFn {
    let anchor = tokio::time::Instant::now();
    Arc::new(move || {
        base_ms.saturating_add(i64::try_from(anchor.elapsed().as_millis()).unwrap_or(i64::MAX))
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{system_now_ms, tokio_clock};

    #[test]
    fn the_system_clock_is_after_the_epoch() {
        assert!(system_now_ms() > 1_600_000_000_000);
    }

    #[tokio::test(start_paused = true)]
    async fn the_tokio_clock_advances_with_paused_time() {
        let clock = tokio_clock(1_000);
        assert_eq!(clock(), 1_000);
        tokio::time::sleep(Duration::from_secs(86_400)).await;
        assert_eq!(clock(), 1_000 + 86_400_000);
    }
}
