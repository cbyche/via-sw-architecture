//! `via-work` — the Work subsystem.
//!
//! The heart of Layer 2, and the biggest single reason the port exists:
//! `docs/architecture.md` §1 records that ARGO has **no** equivalent, and that
//! beyond the four boxes the reference architecture draws it also lacks *"a
//! unified Work `kind` taxonomy, coalesced saves, quarantine on corrupt,
//! duplicate-submission suppression, terminal retention caps, and progress-check
//! announcements."*
//!
//! Ported from `qwen-audio-agent` v1.11.0:
//! `server/src/task/{task-manager,task-scheduler,task-store,reminder-scheduler}.mjs`
//! and `server/src/conversation/task-result-projector.mjs`.
//!
//! # One record, four kinds
//!
//! `work | reminder | scheduled_task | control` live in **one** table with a
//! [`WorkKind`](via_protocol::WorkKind) discriminator. That is why one
//! `cancel_agent_task` cancels a reminder and a delegation alike, and
//! `docs/architecture.md` §4 calls it *"the single biggest structural
//! improvement the port brings"* — ARGO's equivalents are three separate
//! registries with three separate cancel paths.
//!
//! | Kind | Timer | Announces progress | Visible in `list()` |
//! | --- | --- | --- | --- |
//! | `work` | — | yes, every `progressCheckMs` | yes |
//! | `reminder` | due time | no | yes |
//! | `scheduled_task` | due time **and** a wall-clock watchdog | no | yes |
//! | `control` | — | no | only with `include_control` |
//!
//! # What is on the record, and what is not
//!
//! The user request, the timestamps, the final result or error, generic tool
//! activity, one bounded pending permission, and the notification state.
//! **No** execution mode, delivery mode, subagent state, backend permission
//! identifier, backend topology or backend cancellation internal —
//! `docs/architecture.md` §5 is explicit, and
//! [`via_downstream::SessionEvent`] already enforces the activity half by having
//! nowhere to put a secret.
//!
//! # The five things this crate is
//!
//! | Module | What | Upstream |
//! | --- | --- | --- |
//! | [`manager`] | the owning task: `work_id`, the nine states, the seams | `task-manager.mjs` |
//! | [`scheduler`] | admission only — global cap, per-owner cap, coordinator lane | `task-scheduler.mjs` |
//! | [`store`] | `tasks.json`, through `via-store` | `task-store.mjs` |
//! | [`reminder`] | one `Sleep`, re-armed; never a poll | `reminder-scheduler.mjs` |
//! | [`projector`] | a terminal result as a conversation message | `task-result-projector.mjs` |
//!
//! plus the record itself ([`record`], [`activity`], [`delegation`],
//! [`permission`], [`presentation`], [`schedule`]), the event vocabulary
//! ([`event`]), the announcement ([`progress`]), the retention numbers
//! ([`limits`]) and the cancellation ledger ([`reconcile`]).
//!
//! # Concurrency
//!
//! One owning task over a bounded `mpsc`, per `docs/architecture.md` §11 — not
//! a mutex, because *"`tokio::sync::Mutex` gives mutual exclusion but not FIFO
//! order, and the ordering IS the contract."* Every mutation, including the
//! ones runners and timers raise, is one crate-private `Command` in one
//! queue.
//!
//! Every Work owns **its own** cancellation scope
//! ([`AbortSignal`]), never a clone of a caller's: `docs/architecture.md` §11
//! records the ARGO defect where a shared `CancellationToken` let one barge-in
//! kill independent background work.
//!
//! # Cancellation is confirmed, not optimistic
//!
//! `cancelling` is a **state**. A Work stays in it until something confirms the
//! stop — the canceler reported [`CancelOutcome::Confirmed`], or the abort it
//! raised actually settled the runner. `queued` and `scheduled` have nothing
//! running and so confirm at once. See [`WorkManager::cancel`].
//!
//! [`CancelOutcome::Confirmed`]: via_downstream::CancelOutcome::Confirmed
//! [`AbortSignal`]: crate::runner::AbortSignal
//!
//! # Example
//!
//! ```
//! use std::sync::Arc;
//! use via_work::testing::ImmediateRunner;
//! use via_work::{NewWork, WorkManager};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let manager = WorkManager::builder()
//!     .runner(Arc::new(ImmediateRunner::completing("done")))
//!     .build();
//!
//! let accepted = manager.create(NewWork::new("summarise the diff", "ana")).await?;
//! assert!(!accepted.reused);
//!
//! let finished = manager.wait(&accepted.work.id).await.expect("terminal");
//! assert_eq!(finished.result.as_deref(), Some("done"));
//! # Ok(())
//! # }
//! ```
//!
//! # Fidelity
//!
//! Every value that reproduces an upstream literal names the file it came from
//! in its own documentation, and `docs/reference/contracts.json` is the
//! acceptance spec — `tests/contracts.rs` parses it rather than retyping it.
//! Deviations are recorded in `docs/deviations/phase-3.md`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod activity;
pub mod clock;
pub(crate) mod command;
pub mod delegation;
pub mod error;
pub mod event;
pub mod limits;
pub mod manager;
pub mod permission;
pub mod presentation;
pub mod progress;
pub mod projector;
pub mod reconcile;
pub mod record;
pub mod reminder;
pub mod runner;
pub mod schedule;
pub mod scheduler;
pub mod store;
pub mod text;

// `test` as well as the feature: the crate's own unit tests build records
// through `testing::blank_record`, so `--no-default-features` must still see
// it while a production build without the feature does not.
#[cfg(any(test, feature = "testing"))]
pub mod testing;

pub use activity::{ACTIVITY_RING, Activity};
pub use clock::{NowFn, system_clock, system_now_ms, tokio_clock};
pub use delegation::{DelegationPresentation, DelegationRef, PublicDelegation};
pub use error::ManagerStopped;
pub use event::{LOG_INFO_EVENT_NAMES, WorkEvent, WorkEventDetails, WorkEventKind};
pub use limits::RetentionPolicy;
pub use manager::{
    NewScheduledWork, NewWork, NotificationClaim, ScheduledKind, ScheduledWork, WorkAcceptance,
    WorkManager, WorkManagerBuilder, WorkQuery, reminder_runner,
};
pub use permission::{PendingPermission, PermissionStatus};
pub use presentation::{InlineBlock, InlineFormat, Presentation, PublicResultMetadata};
pub use progress::{DEFAULT_PROGRESS_CHECK_MS, progress_message};
pub use projector::TaskResultProjection;
pub use reconcile::{CancellationFact, ReconciliationLedger};
pub use record::{
    NotificationStatus, PersistedWork, PublicWork, WorkRecord, WorkSnapshot, new_work_id,
};
pub use reminder::ReminderScheduler;
pub use runner::{
    AbortSignal, CancelRequest, CoordinatorQuery, DelegatedWorkRecovery, RunFailure, RunnerEvent,
    WorkCanceler, WorkContext, WorkEventSink, WorkOutcome, WorkRunner,
};
pub use schedule::Schedule;
pub use scheduler::{AdmissionScheduler, coordinator_lane};
pub use store::{WorkStore, WorkStoreBuilder};
