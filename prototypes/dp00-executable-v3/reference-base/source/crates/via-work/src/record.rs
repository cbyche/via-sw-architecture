//! The Work record and its two projections.
//!
//! One record, four kinds ([`via_protocol::WorkKind`]), three shapes:
//!
//! | Shape | Built by | Goes to |
//! | --- | --- | --- |
//! | [`WorkRecord`] | the manager | nowhere — it is the in-process state |
//! | [`PublicWork`] | [`WorkRecord::to_public`] | `GET /api/tasks`, SSE, every `task.*` frame |
//! | [`PersistedWork`] | [`WorkRecord::to_persisted`] | `tasks.json` |
//!
//! `publicTask` is `server/src/task/task-manager.mjs:53-99` and `persistedTask`
//! is `:233-247`. The catalogue records both, and the differences between them
//! are all deliberate:
//!
//! - the public shape adds `workId` (a duplicate of `id`) and `workState` (the
//!   collapsed status); the persisted shape deletes both;
//! - the persisted shape adds `submissionKey`, **last**, because JavaScript
//!   appends a newly-assigned key;
//! - the persisted shape replaces the *public* delegation with the full
//!   [`DelegationRef`], deep-copied, because restart recovery needs the ids the
//!   public shape removes;
//! - `resultMetadata` is the projection in **both**, so a runner's `backendRef`
//!   never reaches the disk either;
//! - and `notificationDeliveredAt` is `undefined` until a delivery happens, so
//!   `JSON.stringify` omits the key entirely. `Option` plus
//!   `skip_serializing_if` is that, exactly; every other absent field is an
//!   explicit `null`.
//!
//! # What is deliberately *not* here
//!
//! `docs/architecture.md` §5 and §17's third review question: no execution
//! mode, no delivery mode, no subagent state, no backend permission identifier,
//! no backend topology, no backend cancellation internal. The two internal
//! fields that name a backend — the delegation's `id` and `sessionId` — are on
//! [`DelegationRef`] and are dropped by [`DelegationRef::to_public`].

use serde::{Deserialize, Serialize};
use serde_json::Value;
use via_protocol::{WorkKind, WorkState, WorkStatus};

use crate::activity::Activity;
use crate::delegation::{DelegationRef, PublicDelegation};
use crate::permission::PendingPermission;
use crate::presentation::{self, PublicResultMetadata};
use crate::schedule::Schedule;

/// The default `sessionId` when a caller names none.
///
/// Contract — `server/src/task/task-manager.mjs:366,421`
/// (`String(sessionId || 'main')`).
pub const DEFAULT_SESSION_ID: &str = "main";

/// The `work_` prefix every Work id carries.
///
/// Contract — `server/src/task/task-manager.mjs:359,416`
/// (`` `work_${randomUUID()}` ``), catalogued as *Work id format*. It is
/// returned to the model as `work_id` / `reminder_id` and accepted back
/// verbatim by `cancel_agent_task` and `get_agent_task_status`.
pub const WORK_ID_PREFIX: &str = "work_";

/// Mint a Work id.
///
/// `` `work_${randomUUID()}` `` — a lower-case UUID v4 **with** dashes.
#[must_use]
pub fn new_work_id() -> String {
    format!("{WORK_ID_PREFIX}{}", uuid::Uuid::new_v4())
}

/// Where a terminal Work's notification is in its delivery lease.
///
/// `server/src/task/task-manager.mjs:382,690,852,873,915`. The four values are
/// a lease protocol, not a status display: `pending` is claimable, `delivering`
/// is claimed by exactly one client for at most `notificationClaimTtlMs`, and
/// `delivered` is final. A cancelled Work goes straight to `none` — the user
/// asked for it to stop, so there is nothing to announce (`:782`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationStatus {
    /// Nothing to deliver.
    #[default]
    None,
    /// Waiting for a client to claim it.
    Pending,
    /// Claimed, and the lease is running.
    Delivering,
    /// Spoken or shown; `notificationDeliveredAt` is set.
    Delivered,
}

impl NotificationStatus {
    /// The wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Pending => "pending",
            Self::Delivering => "delivering",
            Self::Delivered => "delivered",
        }
    }

    /// Whether a delivery is still owed.
    ///
    /// `['pending','delivering'].includes(task.notificationStatus)`
    /// (`server/src/task/task-manager.mjs:948`) — the test that decides which
    /// TTL a terminal Work is retained under, and which keeps an undelivered
    /// result out of the per-owner history cap.
    #[must_use]
    pub const fn is_awaiting_delivery(self) -> bool {
        matches!(self, Self::Pending | Self::Delivering)
    }
}

/// The Work record, in process.
///
/// Everything that is either published or persisted. The manager keeps the
/// rest — runner, canceler, timers, waiters, scheduler lease — beside it,
/// because none of that survives a restart and none of it is anybody's
/// business.
#[derive(Debug, Clone)]
pub struct WorkRecord {
    /// `work_<uuidv4>`.
    pub id: String,
    /// Where in the nine-state lifecycle this Work is.
    pub status: WorkStatus,
    /// Which of the four kinds.
    pub kind: WorkKind,
    /// The Work this one was raised on behalf of — a `control` query names the
    /// `work` it is querying.
    pub parent_work_id: Option<String>,
    /// Scheduler priority; higher runs first. Never persisted, so a restored
    /// Work has priority 0.
    pub priority: i64,
    /// The user's request, trimmed.
    pub objective: String,
    /// Whose Work this is.
    pub owner_id: String,
    /// Which voice/text session raised it.
    pub session_id: String,
    /// The turn that raised it.
    pub turn_id: Option<String>,
    /// The duplicate-submission key. Persisted, and **the only field the
    /// persisted shape has that the public one does not**.
    pub submission_key: Option<String>,
    /// The serialization lane. Never persisted: a restored Work holds no lane.
    pub lane_key: Option<String>,
    /// How many Work items that lane admits at once.
    pub lane_limit: usize,
    /// Epoch ms.
    pub created_at: i64,
    /// Epoch ms, or `None` while queued or scheduled.
    pub started_at: Option<i64>,
    /// Epoch ms, or `None` until terminal.
    pub completed_at: Option<i64>,
    /// The frozen elapsed time. Live-computed while active — see
    /// [`WorkRecord::elapsed_ms`].
    pub elapsed_ms: i64,
    /// The final result text.
    pub result: Option<String>,
    /// The final error text.
    pub error: Option<String>,
    /// The runner's metadata, **raw**. Projected on the way out.
    pub result_metadata: Option<Value>,
    /// The activity ring, newest last.
    pub activity: Vec<Activity>,
    /// The delegated project session, in full.
    pub delegation: Option<DelegationRef>,
    /// The one pending permission, if any.
    pub authorization: Option<PendingPermission>,
    /// Where the terminal notification is.
    pub notification_status: NotificationStatus,
    /// Who holds the delivery lease. Never persisted.
    pub notification_claimant_id: Option<String>,
    /// When the lease was taken. Never persisted.
    pub notification_claimed_at: Option<i64>,
    /// When the notification was delivered. `None` means the key is **absent**
    /// from the JSON, not `null`.
    pub notification_delivered_at: Option<i64>,
    /// The due time, for a `reminder` or `scheduled_task`.
    pub schedule: Option<Schedule>,
    /// The hard wall-clock budget. `scheduled_task` only.
    pub timeout_ms: Option<i64>,
    /// The long-running announcement cadence. `work` only.
    pub progress_check_ms: Option<i64>,
}

impl WorkRecord {
    /// `task.startedAt && ACTIVE.has(task.status) ? now - task.startedAt : task.elapsedMs`.
    ///
    /// `server/src/task/task-manager.mjs:69-71`. Note the JavaScript `&&`: a
    /// `startedAt` of `0` is falsy, so it falls through to the stored value.
    #[must_use]
    pub fn elapsed_ms(&self, now: i64) -> i64 {
        match self.started_at {
            Some(started_at) if started_at != 0 && self.status.is_active() => now - started_at,
            _ => self.elapsed_ms,
        }
    }

    /// The published projection.
    #[must_use]
    pub fn to_public(&self, now: i64) -> PublicWork {
        PublicWork {
            id: self.id.clone(),
            work_id: self.id.clone(),
            work_state: self.status.state(),
            status: self.status,
            kind: self.kind,
            parent_work_id: self.parent_work_id.clone(),
            objective: self.objective.clone(),
            owner_id: self.owner_id.clone(),
            session_id: self.session_id.clone(),
            turn_id: self.turn_id.clone(),
            created_at: self.created_at,
            started_at: self.started_at,
            completed_at: self.completed_at,
            elapsed_ms: self.elapsed_ms(now),
            result: self.result.clone(),
            error: self.error.clone(),
            result_metadata: presentation::project(self.result_metadata.as_ref()),
            activity: self.activity.clone(),
            delegation: self.delegation.as_ref().map(DelegationRef::to_public),
            authorization: self.authorization.clone(),
            notification_status: self.notification_status,
            notification_delivered_at: self.notification_delivered_at,
            schedule: self.schedule.clone(),
            timeout_ms: self.timeout_ms,
            progress_check_ms: self.progress_check_ms,
        }
    }

    /// The persisted projection.
    ///
    /// Built from [`Self::to_public`], exactly as upstream builds it — which is
    /// why `resultMetadata` is projected on disk too.
    #[must_use]
    pub fn to_persisted(&self, now: i64) -> PersistedWork {
        let public = self.to_public(now);
        PersistedWork {
            id: public.id,
            status: public.status,
            kind: public.kind,
            parent_work_id: public.parent_work_id,
            objective: public.objective,
            owner_id: public.owner_id,
            session_id: public.session_id,
            turn_id: public.turn_id,
            created_at: public.created_at,
            started_at: public.started_at,
            completed_at: public.completed_at,
            elapsed_ms: public.elapsed_ms,
            result: public.result,
            error: public.error,
            result_metadata: public
                .result_metadata
                .and_then(|metadata| serde_json::to_value(metadata).ok()),
            activity: public.activity,
            delegation: self.delegation.clone(),
            authorization: public.authorization,
            notification_status: public.notification_status,
            notification_delivered_at: public.notification_delivered_at,
            schedule: public.schedule,
            timeout_ms: public.timeout_ms,
            progress_check_ms: public.progress_check_ms,
            submission_key: self.submission_key.clone(),
        }
    }

    /// The snapshot the recovery seam is handed.
    ///
    /// `server/src/task/task-manager.mjs:256-259`:
    /// `{...publicTask(task), delegation: {...task.delegation}}` — the public
    /// record with the *full* delegation put back, because
    /// `canRecover`/`runner`/`canceler` all address the backend by its ids.
    #[must_use]
    pub fn to_snapshot(&self, now: i64) -> WorkSnapshot {
        WorkSnapshot {
            work: self.to_public(now),
            delegation: self.delegation.clone(),
        }
    }
}

/// The Work record as every client sees it.
///
/// Contract — `docs/reference/contracts.json` (`json-field` / *publicTask /
/// Work record*). Field order is the catalogued order, which is what
/// `GET /api/tasks`, `GET /api/tasks/:id`, `GET /api/tasks/:id/events` and
/// every WebSocket `task.*` frame put on the wire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicWork {
    /// `work_<uuidv4>`.
    pub id: String,
    /// A duplicate of [`id`](Self::id). Upstream publishes both names.
    pub work_id: String,
    /// The collapsed status.
    pub work_state: WorkState,
    /// The full status.
    pub status: WorkStatus,
    /// Which of the four kinds.
    pub kind: WorkKind,
    /// The Work this one serves, or `null`.
    pub parent_work_id: Option<String>,
    /// The user's request.
    pub objective: String,
    /// Whose Work this is.
    pub owner_id: String,
    /// Which session raised it.
    pub session_id: String,
    /// Which turn raised it.
    pub turn_id: Option<String>,
    /// Epoch ms.
    pub created_at: i64,
    /// Epoch ms, or `null`.
    pub started_at: Option<i64>,
    /// Epoch ms, or `null`.
    pub completed_at: Option<i64>,
    /// Live while active, frozen once terminal.
    pub elapsed_ms: i64,
    /// The final result text, or `null`.
    pub result: Option<String>,
    /// The final error text, or `null`.
    pub error: Option<String>,
    /// The projected presentation, or `null`.
    pub result_metadata: Option<PublicResultMetadata>,
    /// The activity ring.
    pub activity: Vec<Activity>,
    /// The delegation, without its ids.
    pub delegation: Option<PublicDelegation>,
    /// The pending permission, or `null`.
    pub authorization: Option<PendingPermission>,
    /// Where the notification is.
    pub notification_status: NotificationStatus,
    /// **Absent** until a delivery happens — `JSON.stringify` omits an
    /// `undefined`, and this reproduces that rather than writing `null`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notification_delivered_at: Option<i64>,
    /// The due time, or `null`.
    pub schedule: Option<Schedule>,
    /// The wall-clock budget, or `null`.
    pub timeout_ms: Option<i64>,
    /// The announcement cadence, or `null`.
    pub progress_check_ms: Option<i64>,
}

/// The Work record as `tasks.json` holds it.
///
/// Contract — `docs/reference/contracts.json` (`json-field` / *persisted task
/// record (persistedTask)*): the public fields minus `workId` and `workState`,
/// plus `submissionKey`, with the delegation deep-copied in full.
///
/// **Not persisted**, and therefore lost across a restart: `priority`,
/// `laneKey`, `laneLimit`, `cancellation`, `notificationClaimantId`,
/// `notificationClaimedAt`, `terminalHandled`, `runner`, `canceler`. Restored
/// Work has no lane and priority 0, so lane serialization does not survive a
/// restart — which is upstream's stated consequence, not an omission.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedWork {
    /// `work_<uuidv4>`.
    pub id: String,
    /// The status as of the last save.
    #[serde(default = "default_persisted_status")]
    pub status: WorkStatus,
    /// Which of the four kinds.
    #[serde(default)]
    pub kind: WorkKind,
    /// The Work this one serves.
    #[serde(default)]
    pub parent_work_id: Option<String>,
    /// The user's request.
    #[serde(default)]
    pub objective: String,
    /// Whose Work this is.
    #[serde(default)]
    pub owner_id: String,
    /// Which session raised it.
    #[serde(default = "default_session_id")]
    pub session_id: String,
    /// Which turn raised it.
    #[serde(default)]
    pub turn_id: Option<String>,
    /// Epoch ms.
    #[serde(default)]
    pub created_at: i64,
    /// Epoch ms.
    #[serde(default)]
    pub started_at: Option<i64>,
    /// Epoch ms.
    #[serde(default)]
    pub completed_at: Option<i64>,
    /// Frozen elapsed time.
    #[serde(default)]
    pub elapsed_ms: i64,
    /// The final result text.
    #[serde(default)]
    pub result: Option<String>,
    /// The final error text.
    #[serde(default)]
    pub error: Option<String>,
    /// The **projected** presentation — never the runner's raw metadata.
    ///
    /// Held as raw JSON rather than as a [`PublicResultMetadata`] for one
    /// reason: an already-persisted `tasks.json` may carry the legacy
    /// `{decision: {presentation}}` shape, which a typed field would refuse to
    /// parse and would therefore drop the whole record. It is re-projected on
    /// read — [`crate::presentation::project`] handles both shapes and is
    /// idempotent — and re-persisted in the new shape.
    #[serde(default)]
    pub result_metadata: Option<Value>,
    /// The activity ring.
    #[serde(default)]
    pub activity: Vec<Activity>,
    /// The delegation, in full.
    #[serde(default)]
    pub delegation: Option<DelegationRef>,
    /// The pending permission.
    #[serde(default)]
    pub authorization: Option<PendingPermission>,
    /// Where the notification was.
    #[serde(default)]
    pub notification_status: NotificationStatus,
    /// Absent until a delivery happened.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notification_delivered_at: Option<i64>,
    /// The due time.
    #[serde(default)]
    pub schedule: Option<Schedule>,
    /// The wall-clock budget.
    #[serde(default)]
    pub timeout_ms: Option<i64>,
    /// The announcement cadence.
    #[serde(default)]
    pub progress_check_ms: Option<i64>,
    /// The duplicate-submission key, **last**.
    #[serde(default)]
    pub submission_key: Option<String>,
}

/// A record read off disk with no `status` is treated as `queued`, which is the
/// status `create()` mints and therefore the one a truncated write is most
/// likely to have been mid-way through.
///
/// It is also the safe answer: `queued` is active, so `restore()` force-fails
/// it with the restart reason rather than silently resurrecting it.
fn default_persisted_status() -> WorkStatus {
    WorkStatus::Queued
}

fn default_session_id() -> String {
    DEFAULT_SESSION_ID.to_owned()
}

/// A [`PublicWork`] with the full delegation put back.
///
/// What the delegated-work recovery seam is handed. It exists because
/// `canRecover` must read the delegation's `id` and `sessionId`, which
/// [`PublicWork`] deliberately does not carry.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkSnapshot {
    /// The public record.
    pub work: PublicWork,
    /// The delegation, in full.
    pub delegation: Option<DelegationRef>,
}

impl WorkSnapshot {
    /// Whether this snapshot names a delegation both ids can address.
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        self.delegation
            .as_ref()
            .is_some_and(DelegationRef::is_addressable)
    }
}

#[cfg(test)]
mod tests {
    use super::{NotificationStatus, PersistedWork, WorkRecord, new_work_id};
    use crate::delegation::DelegationRef;
    use serde_json::{Value, json};
    use via_protocol::{WorkKind, WorkStatus};

    fn record() -> WorkRecord {
        WorkRecord {
            id: "work_one".to_owned(),
            status: WorkStatus::Queued,
            kind: WorkKind::Work,
            parent_work_id: None,
            priority: 0,
            objective: "objective".to_owned(),
            owner_id: "owner".to_owned(),
            session_id: "voice".to_owned(),
            turn_id: None,
            submission_key: None,
            lane_key: None,
            lane_limit: 1,
            created_at: 1_000,
            started_at: None,
            completed_at: None,
            elapsed_ms: 0,
            result: None,
            error: None,
            result_metadata: None,
            activity: Vec::new(),
            delegation: None,
            authorization: None,
            notification_status: NotificationStatus::None,
            notification_claimant_id: None,
            notification_claimed_at: None,
            notification_delivered_at: None,
            schedule: None,
            timeout_ms: None,
            progress_check_ms: Some(300_000),
        }
    }

    fn keys(value: &Value) -> Vec<String> {
        value
            .as_object()
            .map(|object| object.keys().cloned().collect())
            .unwrap_or_default()
    }

    #[test]
    fn a_work_id_is_the_prefix_and_a_dashed_uuid() {
        let id = new_work_id();
        let uuid = id.strip_prefix("work_").expect("prefixed");
        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.matches('-').count(), 4);
        assert_eq!(uuid, uuid.to_lowercase());
        assert_ne!(id, new_work_id());
    }

    #[test]
    fn the_persisted_shape_drops_two_keys_and_appends_one() {
        let record = record();
        let public = serde_json::to_value(record.to_public(2_000)).expect("serializes");
        let persisted = serde_json::to_value(record.to_persisted(2_000)).expect("serializes");
        let public_keys = keys(&public);
        let persisted_keys = keys(&persisted);

        assert!(public_keys.contains(&"workId".to_owned()));
        assert!(public_keys.contains(&"workState".to_owned()));
        assert!(!persisted_keys.contains(&"workId".to_owned()));
        assert!(!persisted_keys.contains(&"workState".to_owned()));
        assert_eq!(persisted_keys.last(), Some(&"submissionKey".to_owned()));

        let expected: Vec<String> = public_keys
            .iter()
            .filter(|key| *key != "workId" && *key != "workState")
            .cloned()
            .chain(std::iter::once("submissionKey".to_owned()))
            .collect();
        assert_eq!(persisted_keys, expected);
    }

    #[test]
    fn notification_delivered_at_is_absent_until_it_is_set() {
        let mut record = record();
        assert!(
            !keys(&serde_json::to_value(record.to_public(0)).expect("ok"))
                .contains(&"notificationDeliveredAt".to_owned())
        );
        record.notification_delivered_at = Some(9);
        let rendered = serde_json::to_value(record.to_public(0)).expect("ok");
        assert_eq!(rendered["notificationDeliveredAt"], json!(9));
    }

    #[test]
    fn every_other_absent_field_is_an_explicit_null() {
        let rendered = serde_json::to_value(record().to_public(0)).expect("ok");
        for field in [
            "parentWorkId",
            "turnId",
            "startedAt",
            "completedAt",
            "result",
            "error",
            "resultMetadata",
            "delegation",
            "authorization",
            "schedule",
            "timeoutMs",
        ] {
            assert_eq!(
                rendered[field],
                Value::Null,
                "{field} must be null, not absent"
            );
        }
    }

    #[test]
    fn elapsed_is_live_while_active_and_frozen_otherwise() {
        let mut record = record();
        record.started_at = Some(1_000);
        record.status = WorkStatus::Running;
        record.elapsed_ms = 7;
        assert_eq!(record.elapsed_ms(4_000), 3_000);
        record.status = WorkStatus::Completed;
        assert_eq!(record.elapsed_ms(4_000), 7);
        record.status = WorkStatus::Scheduled;
        assert_eq!(record.elapsed_ms(4_000), 7, "scheduled is not active");
        record.status = WorkStatus::Running;
        record.started_at = Some(0);
        assert_eq!(
            record.elapsed_ms(4_000),
            7,
            "a falsy startedAt falls through"
        );
    }

    #[test]
    fn the_persisted_delegation_keeps_the_ids_the_public_one_drops() {
        let mut record = record();
        record.delegation = Some(
            DelegationRef::new("run-one", "agent:child:one").with_directory("/private/project"),
        );
        let public = serde_json::to_value(record.to_public(0)).expect("ok");
        let persisted = serde_json::to_value(record.to_persisted(0)).expect("ok");
        assert!(!public.to_string().contains("run-one"));
        assert_eq!(persisted["delegation"]["id"], json!("run-one"));
        assert_eq!(
            persisted["delegation"]["sessionId"],
            json!("agent:child:one")
        );
    }

    #[test]
    fn the_persisted_metadata_is_the_projection_not_the_runners_object() {
        let mut record = record();
        record.result_metadata = Some(json!({
            "presentation": {"speech": "done", "inline": null},
            "backendRef": {"directory": "/private/project"},
        }));
        let persisted = serde_json::to_value(record.to_persisted(0)).expect("ok");
        assert!(!persisted.to_string().contains("/private/project"));
        assert_eq!(
            persisted["resultMetadata"]["presentation"]["speech"],
            json!("done")
        );
    }

    #[test]
    fn a_persisted_record_round_trips() {
        let persisted = record().to_persisted(0);
        let rendered = serde_json::to_value(&persisted).expect("ok");
        let parsed: PersistedWork = serde_json::from_value(rendered).expect("parses");
        assert_eq!(parsed, persisted);
    }

    #[test]
    fn a_sparse_persisted_record_still_parses() {
        // Upstream's store filters only on `task && typeof task === 'object'`,
        // so a record written by an older Gateway may be missing anything.
        let parsed: PersistedWork = serde_json::from_value(json!({"id": "work_sparse"}))
            .expect("every field but the id has a default");
        assert_eq!(parsed.id, "work_sparse");
        assert_eq!(parsed.status, WorkStatus::Queued);
        assert_eq!(parsed.kind, WorkKind::Work);
        assert_eq!(parsed.session_id, "main");
    }

    #[test]
    fn awaiting_delivery_is_pending_or_delivering() {
        assert!(NotificationStatus::Pending.is_awaiting_delivery());
        assert!(NotificationStatus::Delivering.is_awaiting_delivery());
        assert!(!NotificationStatus::None.is_awaiting_delivery());
        assert!(!NotificationStatus::Delivered.is_awaiting_delivery());
    }
}
