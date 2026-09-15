//! Projecting a terminal Work into the conversation.
//!
//! `server/src/conversation/task-result-projector.mjs`, all eighteen lines of
//! it. It exists so that the *one* place a Work result becomes a conversation
//! message is a named function rather than three call sites that drifted:
//! upstream calls it from the task-completed subscriber, from the notification
//! claim, and again on reconnect.
//!
//! # The whole of the idempotency
//!
//! The message id is `` `agent:${task.id}` ``. Recording it twice is a no-op
//! because the conversation store updates in place on a repeated id, which is
//! what makes re-projection on reconnect free
//! (`docs/reference/contracts.json`, *conversation message id conventions*).
//! Nothing here needs to remember what it has already projected.
//!
//! # Why this returns a value instead of writing one
//!
//! `via-conversation` owns the store. This crate owns which Work is
//! projectable, what its content is, and the two field values that make the
//! projection recognisable — so it produces the record and hands it over.

use serde::{Deserialize, Serialize};
use via_protocol::WorkStatus;

use crate::record::PublicWork;

/// The `id` prefix a projected Work result carries.
///
/// Contract — `server/src/conversation/task-result-projector.mjs:10`
/// (`` `agent:${task.id}` ``), catalogued under *conversation message id
/// conventions*: *"the `agent:` prefix is what makes re-projection on reconnect
/// a no-op."*
pub const PROJECTION_ID_PREFIX: &str = "agent:";

/// The `source` a projected Work result carries.
///
/// Contract — `docs/reference/contracts.json` (`json-field` / *conversation
/// source values*). `agent-result` is included in the frontend context **only**
/// when it has a `taskId` that no `agent-presentation` message already covered,
/// which is why the `task_id` field below is not optional.
pub const PROJECTION_SOURCE: &str = "agent-result";

/// The `role` a projected Work result carries.
pub const PROJECTION_ROLE: &str = "assistant";

/// A terminal Work result, ready to be recorded as a conversation message.
///
/// Field order and names are `conversationSync.record`'s
/// (`task-result-projector.mjs:8-17`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskResultProjection {
    /// Whose conversation this belongs to.
    #[serde(rename = "ownerId")]
    pub owner_id: String,
    /// Which session's conversation.
    #[serde(rename = "sessionId")]
    pub session_id: String,
    /// `agent:<work id>` — the idempotency key.
    pub id: String,
    /// Always [`PROJECTION_ROLE`].
    pub role: String,
    /// The result on success, the error on failure.
    ///
    /// **Not** the presentation: upstream records `task.result`, the runner's
    /// raw content, because the presentation is what is *spoken* and the
    /// content is what the model is later shown.
    pub content: Option<String>,
    /// Always [`PROJECTION_SOURCE`].
    pub source: String,
    /// The turn that raised the Work.
    #[serde(rename = "turnId")]
    pub turn_id: Option<String>,
    /// The Work id, again — the field the frontend-context filter keys on.
    #[serde(rename = "taskId")]
    pub task_id: String,
}

/// Project `work` into a conversation message, if it is projectable.
///
/// `recordTaskResult` — `server/src/conversation/task-result-projector.mjs:7`:
/// only [`WorkStatus::Completed`] and [`WorkStatus::Failed`] project.
///
/// **[`WorkStatus::Cancelled`] deliberately does not.** The user asked for the
/// Work to stop; putting "cancelled" into the transcript would give the model
/// something to narrate about a decision the user already made. It is the same
/// reason a cancelled Work's `notificationStatus` goes to `none`.
#[must_use]
pub fn project(
    owner_id: &str,
    session_id: &str,
    work: &PublicWork,
) -> Option<TaskResultProjection> {
    let content = match work.status {
        WorkStatus::Completed => work.result.clone(),
        WorkStatus::Failed => work.error.clone(),
        _ => return None,
    };
    Some(TaskResultProjection {
        owner_id: owner_id.to_owned(),
        session_id: session_id.to_owned(),
        id: format!("{PROJECTION_ID_PREFIX}{}", work.id),
        role: PROJECTION_ROLE.to_owned(),
        content,
        source: PROJECTION_SOURCE.to_owned(),
        turn_id: work.turn_id.clone(),
        task_id: work.id.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::{PROJECTION_ROLE, PROJECTION_SOURCE, project};
    use crate::record::WorkRecord;
    use crate::testing::blank_record;
    use via_protocol::WorkStatus;

    fn work(status: WorkStatus) -> WorkRecord {
        WorkRecord {
            status,
            result: Some("the result".to_owned()),
            error: Some("the error".to_owned()),
            turn_id: Some("turn-one".to_owned()),
            ..blank_record("work_one", "owner")
        }
    }

    #[test]
    fn a_completed_work_projects_its_result() {
        let projected = project("owner", "main", &work(WorkStatus::Completed).to_public(0))
            .expect("completed projects");
        assert_eq!(projected.id, "agent:work_one");
        assert_eq!(projected.task_id, "work_one");
        assert_eq!(projected.content.as_deref(), Some("the result"));
        assert_eq!(projected.source, PROJECTION_SOURCE);
        assert_eq!(projected.role, PROJECTION_ROLE);
        assert_eq!(projected.turn_id.as_deref(), Some("turn-one"));
        assert_eq!(projected.owner_id, "owner");
        assert_eq!(projected.session_id, "main");
    }

    #[test]
    fn a_failed_work_projects_its_error() {
        let projected = project("owner", "main", &work(WorkStatus::Failed).to_public(0))
            .expect("failed projects");
        assert_eq!(projected.content.as_deref(), Some("the error"));
    }

    #[test]
    fn nothing_else_projects() {
        for status in [
            WorkStatus::Scheduled,
            WorkStatus::Queued,
            WorkStatus::Running,
            WorkStatus::Delegated,
            WorkStatus::Finalizing,
            WorkStatus::Cancelling,
            WorkStatus::Cancelled,
        ] {
            assert!(
                project("owner", "main", &work(status).to_public(0)).is_none(),
                "{status} must not reach the transcript",
            );
        }
    }

    #[test]
    fn two_projections_of_one_work_are_identical() {
        let work = work(WorkStatus::Completed).to_public(0);
        assert_eq!(
            project("owner", "main", &work),
            project("owner", "main", &work),
            "re-projection on reconnect must be a no-op",
        );
    }

    #[test]
    fn the_wire_names_are_camel_case() {
        let projected =
            project("owner", "main", &work(WorkStatus::Completed).to_public(0)).expect("projects");
        let rendered = serde_json::to_value(projected).expect("serializes");
        let keys: Vec<&String> = rendered.as_object().expect("object").keys().collect();
        assert_eq!(
            keys,
            [
                "ownerId",
                "sessionId",
                "id",
                "role",
                "content",
                "source",
                "turnId",
                "taskId"
            ]
        );
    }
}
