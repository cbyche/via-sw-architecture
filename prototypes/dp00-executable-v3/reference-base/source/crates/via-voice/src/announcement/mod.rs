//! The Injection Gate's two halves.
//!
//! [`window`] decides whether a finished result may be spoken *now*;
//! [`manager`] decides what is spoken, when it is retried and when it counts as
//! delivered. [`format`] is the model-visible envelope both produce.
//!
//! [`crate::gate::InjectionGate`] wraps the window with the three gateway flags
//! and the playback cursor, which is the predicate `docs/architecture.md` §11
//! invariant 3 names.

pub mod format;
pub mod manager;
pub mod window;

pub use format::{
    Announcement, AnnouncementEvent, COMPLETE_MARKER, PROGRESS_CLOSE_TAG, PROGRESS_MARKER,
    PROGRESS_OPEN_TAG, WORK_RESULTS_CLOSE_TAG, WORK_RESULTS_OPEN_TAG, asleep_message,
    format_progress, format_restored_context, format_work_results, permission_resolved_note,
    truncate_result,
};
pub use manager::{
    AnnouncementManager, AnnouncementManagerConfig, DEFAULT_ACKNOWLEDGEMENT_TIMEOUT_MS,
    DEFAULT_BATCH_WINDOW_MS, DEFAULT_LEASE_RENEW_INTERVAL_MS, DEFAULT_MAX_BATCH_ITEMS,
    DEFAULT_MAX_RETRY_ATTEMPTS, DEFAULT_RESULT_CONTEXT_MAX_CHARS, DEFAULT_RETRY_BASE_MS,
    DEFAULT_RETRY_MAX_MS, DeliveryBlocked, FrontendSource, NoClaims, NotificationClaims,
    RETRY_EXPONENT_CAP,
};
pub use window::{AnnouncementWindow, ResponseDone};
