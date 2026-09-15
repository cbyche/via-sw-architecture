//! The one pending permission a Work may be carrying.
//!
//! `server/src/task/task-manager.mjs:491-505`: a `backend.permission.requested`
//! event puts the permission on `task.authorization`, a
//! `backend.permission.resolved` for **the same id** takes it off again, and
//! `publicTask` copies it out.
//!
//! # What "bounded" means here
//!
//! The Work record holds **one** permission, never a list. It is replaced by
//! the next request, cleared when the matching resolution arrives, cleared when
//! a cancellation starts (`:722`), cleared when the runner settles (`:676`),
//! and cleared on restore for anything active or terminal (`:203-205`). There
//! is no path by which pending permissions accumulate on a record, which is the
//! bound.
//!
//! The *string* bounds are `PermissionBroker`'s and are not restated here:
//! `category` is `bounded(name, 80) || 'unknown'` and `summary` is
//! `[bounded(name, 80), bounded(detail, 300)].filter(Boolean).join('：')`
//! (`server/src/agent/permission-broker.mjs:51-79`, catalogued under
//! *backend.permission.requested / .resolved events*). Re-bounding to 300 here
//! would truncate a legitimate 381-character summary the broker built.

use serde::{Deserialize, Serialize};

/// Where a permission request is.
///
/// Contract — `docs/reference/contracts.json`
/// (`json-field` / *backend.permission.requested / .resolved events*):
/// `pending` on the request, one of `approved` / `denied` / `cancelled` on the
/// resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionStatus {
    /// Waiting for a decision. The only status that can sit on a Work record.
    Pending,
    /// The user allowed it.
    Approved,
    /// The user refused it.
    Denied,
    /// Nobody decided and the request went away.
    Cancelled,
}

impl PermissionStatus {
    /// The wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Denied => "denied",
            Self::Cancelled => "cancelled",
        }
    }
}

/// A backend's request for authorization, as the Work record carries it.
///
/// Field order is the catalogued payload order:
/// `{id, workId, status, category, summary, patterns}`. `patterns` is absent
/// from the *resolved* shape, so it is skipped when empty rather than written
/// as `[]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingPermission {
    /// `auth_<32 hex>` — the id the voice model quotes back in
    /// `respond_agent_permission`.
    pub id: String,
    /// The Work this permission belongs to, when the adapter correlated one.
    #[serde(rename = "workId", default, skip_serializing_if = "Option::is_none")]
    pub work_id: Option<String>,
    /// Where the request is.
    pub status: PermissionStatus,
    /// The tool name, bounded by the broker, or `unknown`.
    pub category: String,
    /// `<name>：<description|command|path>`, bounded by the broker.
    pub summary: String,
    /// The broker's pattern list. Absent on a resolution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patterns: Option<Vec<String>>,
}

impl PendingPermission {
    /// A pending request.
    #[must_use]
    pub fn pending(id: &str, category: &str, summary: &str) -> Self {
        Self {
            id: id.to_owned(),
            work_id: None,
            status: PermissionStatus::Pending,
            category: category.to_owned(),
            summary: summary.to_owned(),
            patterns: None,
        }
    }

    /// Correlate this permission with a Work.
    #[must_use]
    pub fn with_work_id(mut self, work_id: &str) -> Self {
        self.work_id = Some(work_id.to_owned());
        self
    }

    /// Set the resolution status.
    #[must_use]
    pub fn with_status(mut self, status: PermissionStatus) -> Self {
        self.status = status;
        self
    }

    /// Whether `self` and `other` name the same request.
    ///
    /// `task.authorization?.id === event.permission.id`
    /// (`server/src/task/task-manager.mjs:498`) — a resolution for a *different*
    /// permission is announced but leaves the pending one in place.
    #[must_use]
    pub fn is_same_request(&self, other: &Self) -> bool {
        !self.id.is_empty() && self.id == other.id
    }
}

#[cfg(test)]
mod tests {
    use super::{PendingPermission, PermissionStatus};
    use serde_json::json;

    #[test]
    fn a_pending_request_serializes_in_the_catalogued_order() {
        let permission = PendingPermission::pending("auth_one", "bash", "bash：npm test")
            .with_work_id("work_one");
        let rendered = serde_json::to_string(&permission).expect("serializes");
        assert_eq!(
            rendered,
            r#"{"id":"auth_one","workId":"work_one","status":"pending","category":"bash","summary":"bash：npm test"}"#
        );
    }

    #[test]
    fn patterns_are_carried_when_the_broker_sent_them() {
        let permission = PendingPermission {
            patterns: Some(vec!["npm *".to_owned()]),
            ..PendingPermission::pending("auth_one", "bash", "s")
        };
        let rendered = serde_json::to_value(&permission).expect("serializes");
        assert_eq!(rendered["patterns"], json!(["npm *"]));
    }

    #[test]
    fn only_the_same_id_is_the_same_request() {
        let pending = PendingPermission::pending("auth_one", "bash", "s");
        assert!(
            pending.is_same_request(
                &PendingPermission::pending("auth_one", "bash", "s")
                    .with_status(PermissionStatus::Approved)
            )
        );
        assert!(!pending.is_same_request(&PendingPermission::pending("auth_two", "bash", "s")));
        assert!(
            !PendingPermission::pending("", "bash", "s")
                .is_same_request(&PendingPermission::pending("", "bash", "s")),
            "an empty id matches nothing",
        );
    }

    #[test]
    fn every_status_round_trips() {
        for (status, wire) in [
            (PermissionStatus::Pending, "pending"),
            (PermissionStatus::Approved, "approved"),
            (PermissionStatus::Denied, "denied"),
            (PermissionStatus::Cancelled, "cancelled"),
        ] {
            assert_eq!(status.as_str(), wire);
            assert_eq!(
                serde_json::to_value(status).expect("serializes"),
                json!(wire)
            );
        }
    }
}
