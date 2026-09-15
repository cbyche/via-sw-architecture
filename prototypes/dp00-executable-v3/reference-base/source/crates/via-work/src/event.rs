//! The sixteen event names the Work manager emits, and the envelope they
//! travel in.
//!
//! Contract — `docs/reference/contracts.json` (`ws-event` / *TaskManager event
//! type names*), from `server/src/task/task-manager.mjs:296-333,399,460,…` and
//! `reminder-scheduler.mjs:27,67`.
//!
//! # Sixteen here, fourteen on the socket
//!
//! [`via_protocol::GatewayTaskEvent`] declares **fourteen** names, and the two
//! vocabularies differ in both directions. That is contract, not drift:
//!
//! - `task.accepted`, `task.notification.pending` and
//!   `task.notification.delivered` are emitted here and are **not** on the
//!   socket. They are the Work manager's own signals: `task.accepted` is the
//!   receipt `create()` returns synchronously, and the two notification events
//!   drive the announcement lease.
//! - `task.notification.offline` is on the socket and is never emitted by
//!   anything — the offline hand-off travels as a host IPC message.
//!
//! [`WorkEventKind::gateway_event`] is the mapping, and it is the only place
//! the two vocabularies meet.
//!
//! # The log split
//!
//! The catalogue also pins which events log at `info` and which at `debug`,
//! *including* a name that is never emitted:
//!
//! > NOTE: `'task.created'` appears in that log list but is never emitted —
//! > `create()` emits `'task.accepted'`.
//!
//! [`LOG_INFO_EVENT_NAMES`] reproduces the list verbatim, `task.created`
//! included, so the catalogued value stays assertable; no [`WorkEventKind`]
//! maps to it.

use serde::{Deserialize, Serialize};
use via_log::LogLevel;
use via_protocol::GatewayTaskEvent;

use crate::permission::PendingPermission;
use crate::record::PublicWork;

/// The names upstream logs at `info`; everything else logs at `debug`.
///
/// Contract — `server/src/task/task-manager.mjs:304-314`, quoted in
/// `docs/reference/contracts.json`. `task.created` is a member of upstream's
/// list and is never emitted; it is kept so the list itself round-trips.
pub const LOG_INFO_EVENT_NAMES: &[&str] = &[
    "task.scheduled",
    "task.created",
    "task.running",
    "task.delegated",
    "task.permission.requested",
    "task.permission.resolved",
    "task.completed",
    "task.failed",
    "task.cancelled",
];

macro_rules! work_event_kind {
    ($( $(#[$meta:meta])* $variant:ident = $wire:literal ),+ $(,)?) => {
        /// One of the sixteen Work lifecycle events.
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub enum WorkEventKind {
            $( $(#[$meta])* #[serde(rename = $wire)] $variant, )+
        }

        impl WorkEventKind {
            /// Every event name, in the catalogued order.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// The exact wire string.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $wire),+ }
            }

            /// Parse a wire string.
            #[must_use]
            pub fn from_wire(wire: &str) -> Option<Self> {
                match wire { $($wire => Some(Self::$variant),)+ _ => None }
            }
        }
    };
}

work_event_kind! {
    /// `task.accepted` — `create()` admitted the Work. **Manager-only**: it is
    /// not on the socket, because the receipt is the tool call's return value.
    Accepted = "task.accepted",
    /// `task.scheduled` — a reminder or scheduled task is waiting for its timer.
    Scheduled = "task.scheduled",
    /// `task.scheduled.fired` — the timer fired; `scheduled` → `queued`.
    ScheduledFired = "task.scheduled.fired",
    /// `task.running` — the scheduler admitted it.
    Running = "task.running",
    /// `task.progress` — the one-second liveness tick. Never persisted.
    Progress = "task.progress",
    /// `task.progress.check` — the long-running announcement. Carries
    /// `message` and `delegated`.
    ProgressCheck = "task.progress.check",
    /// `task.delegated` — handed to a backend session; the lane is released.
    Delegated = "task.delegated",
    /// `task.finalizing` — the delegated session reported completion.
    Finalizing = "task.finalizing",
    /// `task.permission.requested` — a backend wants authorization.
    PermissionRequested = "task.permission.requested",
    /// `task.permission.resolved` — the decision landed. Carries `permission`.
    PermissionResolved = "task.permission.resolved",
    /// `task.completed` — terminal success.
    Completed = "task.completed",
    /// `task.failed` — terminal failure.
    Failed = "task.failed",
    /// `task.cancelling` — a stop was requested and is in flight.
    Cancelling = "task.cancelling",
    /// `task.cancelled` — the stop was confirmed.
    Cancelled = "task.cancelled",
    /// `task.notification.pending` — a terminal result is claimable.
    /// **Manager-only.**
    NotificationPending = "task.notification.pending",
    /// `task.notification.delivered` — the lease holder spoke or showed it.
    /// **Manager-only.**
    NotificationDelivered = "task.notification.delivered",
}

impl WorkEventKind {
    /// The socket event this maps to, or `None` when it never leaves the
    /// process.
    ///
    /// The three `None`s are `task.accepted`, `task.notification.pending` and
    /// `task.notification.delivered`.
    #[must_use]
    pub const fn gateway_event(self) -> Option<GatewayTaskEvent> {
        match self {
            Self::Scheduled => Some(GatewayTaskEvent::Scheduled),
            Self::ScheduledFired => Some(GatewayTaskEvent::ScheduledFired),
            Self::Running => Some(GatewayTaskEvent::Running),
            Self::Progress => Some(GatewayTaskEvent::Progress),
            Self::ProgressCheck => Some(GatewayTaskEvent::ProgressCheck),
            Self::Delegated => Some(GatewayTaskEvent::Delegated),
            Self::Finalizing => Some(GatewayTaskEvent::Finalizing),
            Self::PermissionRequested => Some(GatewayTaskEvent::PermissionRequested),
            Self::PermissionResolved => Some(GatewayTaskEvent::PermissionResolved),
            Self::Completed => Some(GatewayTaskEvent::Completed),
            Self::Failed => Some(GatewayTaskEvent::Failed),
            Self::Cancelling => Some(GatewayTaskEvent::Cancelling),
            Self::Cancelled => Some(GatewayTaskEvent::Cancelled),
            Self::Accepted | Self::NotificationPending | Self::NotificationDelivered => None,
        }
    }

    /// Which level this event logs at.
    ///
    /// [`LogLevel::Info`] for the nine names in [`LOG_INFO_EVENT_NAMES`],
    /// [`LogLevel::Debug`] for the rest.
    #[must_use]
    pub fn log_level(self) -> LogLevel {
        if LOG_INFO_EVENT_NAMES.contains(&self.as_str()) {
            LogLevel::Info
        } else {
            LogLevel::Debug
        }
    }

    /// Whether emitting this event writes `tasks.json` synchronously.
    ///
    /// `emit(type, task, { persist = true })`
    /// (`server/src/task/task-manager.mjs:296`): every event persists except
    /// the three that are pure liveness or already persisted by their caller —
    /// `task.progress`, `task.progress.check` and
    /// `task.notification.delivered`, whose caller batches one `persist()`
    /// after the loop (`:878-880`).
    #[must_use]
    pub const fn persists(self) -> bool {
        !matches!(
            self,
            Self::Progress | Self::ProgressCheck | Self::NotificationDelivered
        )
    }
}

impl core::fmt::Display for WorkEventKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What a subscriber receives.
///
/// Contract — `docs/reference/contracts.json` (`json-field` / *TaskManager
/// event envelope*): `{ type, ownerId, task, ...details }`, where `details` is
/// `{message, delegated}` for `task.progress.check` and `{permission}` for
/// `task.permission.resolved`.
///
/// Modelled as a struct with a typed [`WorkEventDetails`] rather than an
/// open map, because those are the only two shapes upstream ever adds and a
/// map would be a place to put a session id.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WorkEvent {
    /// The event name.
    #[serde(rename = "type")]
    pub kind: WorkEventKind,
    /// Whose Work moved.
    #[serde(rename = "ownerId")]
    pub owner_id: String,
    /// The Work, as of the moment the event was raised.
    pub task: PublicWork,
    /// The two extra shapes, flattened into the envelope as upstream spreads
    /// them.
    #[serde(flatten)]
    pub details: WorkEventDetails,
}

/// The optional half of a [`WorkEvent`].
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
#[serde(untagged)]
pub enum WorkEventDetails {
    /// No extra fields — fourteen of the sixteen events.
    #[default]
    None,
    /// `task.progress.check` only.
    ProgressCheck {
        /// The announcement, already localized and already interpolated.
        message: String,
        /// Whether the message came from the coordinator's own status query
        /// rather than from the activity ring.
        delegated: bool,
    },
    /// `task.permission.resolved` only.
    PermissionResolved {
        /// The decision, as the broker reported it.
        permission: PendingPermission,
    },
}

impl WorkEvent {
    /// An event with no extra fields.
    #[must_use]
    pub fn new(kind: WorkEventKind, task: PublicWork) -> Self {
        Self {
            kind,
            owner_id: task.owner_id.clone(),
            task,
            details: WorkEventDetails::None,
        }
    }

    /// The progress-check announcement.
    #[must_use]
    pub fn progress_check(task: PublicWork, message: String, delegated: bool) -> Self {
        Self {
            kind: WorkEventKind::ProgressCheck,
            owner_id: task.owner_id.clone(),
            task,
            details: WorkEventDetails::ProgressCheck { message, delegated },
        }
    }

    /// A resolved permission.
    #[must_use]
    pub fn permission_resolved(task: PublicWork, permission: PendingPermission) -> Self {
        Self {
            kind: WorkEventKind::PermissionResolved,
            owner_id: task.owner_id.clone(),
            task,
            details: WorkEventDetails::PermissionResolved { permission },
        }
    }

    /// The announcement text, for a `task.progress.check`.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        match &self.details {
            WorkEventDetails::ProgressCheck { message, .. } => Some(message),
            _ => None,
        }
    }

    /// Whether the announcement came from the coordinator.
    #[must_use]
    pub fn is_delegated_message(&self) -> bool {
        matches!(
            self.details,
            WorkEventDetails::ProgressCheck {
                delegated: true,
                ..
            }
        )
    }

    /// The decision, for a `task.permission.resolved`.
    #[must_use]
    pub fn permission(&self) -> Option<&PendingPermission> {
        match &self.details {
            WorkEventDetails::PermissionResolved { permission } => Some(permission),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LOG_INFO_EVENT_NAMES, WorkEventKind};
    use via_log::LogLevel;
    use via_protocol::GatewayTaskEvent;

    #[test]
    fn there_are_sixteen_names_and_they_round_trip() {
        assert_eq!(WorkEventKind::ALL.len(), 16);
        for kind in WorkEventKind::ALL {
            assert_eq!(WorkEventKind::from_wire(kind.as_str()), Some(*kind));
            assert!(kind.as_str().starts_with("task."));
        }
    }

    #[test]
    fn exactly_three_names_never_reach_the_socket() {
        let internal: Vec<&str> = WorkEventKind::ALL
            .iter()
            .filter(|kind| kind.gateway_event().is_none())
            .map(|kind| kind.as_str())
            .collect();
        assert_eq!(
            internal,
            [
                "task.accepted",
                "task.notification.pending",
                "task.notification.delivered"
            ]
        );
    }

    #[test]
    fn the_socket_vocabulary_has_exactly_one_name_nothing_emits() {
        let emitted: Vec<GatewayTaskEvent> = WorkEventKind::ALL
            .iter()
            .filter_map(|kind| kind.gateway_event())
            .collect();
        let never: Vec<&str> = GatewayTaskEvent::ALL
            .iter()
            .filter(|event| !emitted.contains(event))
            .map(|event| event.as_str())
            .collect();
        assert_eq!(never, ["task.notification.offline"]);
    }

    #[test]
    fn the_gateway_mapping_never_renames_an_event() {
        for kind in WorkEventKind::ALL {
            if let Some(event) = kind.gateway_event() {
                assert_eq!(kind.as_str(), event.as_str());
            }
        }
    }

    #[test]
    fn the_info_list_keeps_the_name_that_is_never_emitted() {
        assert!(LOG_INFO_EVENT_NAMES.contains(&"task.created"));
        assert!(
            WorkEventKind::from_wire("task.created").is_none(),
            "create() emits task.accepted; task.created exists only in the log list",
        );
    }

    #[test]
    fn the_log_split_matches_the_catalogued_list() {
        for kind in WorkEventKind::ALL {
            let expected = if LOG_INFO_EVENT_NAMES.contains(&kind.as_str()) {
                LogLevel::Info
            } else {
                LogLevel::Debug
            };
            assert_eq!(kind.log_level(), expected, "{kind}");
        }
        assert_eq!(WorkEventKind::Accepted.log_level(), LogLevel::Debug);
        assert_eq!(WorkEventKind::Completed.log_level(), LogLevel::Info);
    }

    #[test]
    fn exactly_three_events_skip_the_synchronous_save() {
        let deferred: Vec<&str> = WorkEventKind::ALL
            .iter()
            .filter(|kind| !kind.persists())
            .map(|kind| kind.as_str())
            .collect();
        assert_eq!(
            deferred,
            [
                "task.progress",
                "task.progress.check",
                "task.notification.delivered"
            ]
        );
    }
}
