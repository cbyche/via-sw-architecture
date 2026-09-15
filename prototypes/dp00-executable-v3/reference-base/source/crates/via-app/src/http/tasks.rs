//! The Work HTTP surface — `/api/tasks`, `/api/timeline`, and the SSE stream.
//!
//! **External contract** — `docs/reference/contracts.json`, `http-route` /
//! *Work HTTP surface*, described there as *"byte-for-byte client contract for
//! WebUI/Desktop/CLI"*. Ported from
//! `server/src/app/gateway-application.mjs:315-421`.
//!
//! Every lookup is owner-scoped, and the catalogue is explicit about what that
//! means: *"a task belonging to another owner reads as 404, not 403."*

use std::convert::Infallible;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use via_i18n::{keys, t};
use via_protocol::GatewayTaskEvent;
use via_work::{PublicWork, WorkQuery};

use crate::http::middleware::RequestIdentity;
use crate::state::AppState;

/// The query string `/api/tasks` and `/api/timeline` accept.
///
/// **External contract** — `gateway-application.mjs:315-323`: `active` is
/// compared to the **literal string** `'true'`, so `?active=1` and `?active`
/// both disable the filter. Reproducing that is why the field is a `String`
/// and not a `bool`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TaskListQuery {
    /// Restrict to one session.
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    /// `"true"`, and nothing else, enables the active filter.
    pub active: Option<String>,
}

impl TaskListQuery {
    fn to_work_query(&self, owner_id: &str) -> WorkQuery {
        let mut query = WorkQuery::owner(owner_id);
        query.session_id = self.session_id.clone();
        query.active_only = self.active.as_deref() == Some("true");
        query
    }
}

/// `{ "tasks": [...] }`.
#[derive(Debug, Clone, Serialize)]
pub struct TaskListBody {
    /// The owner's Work, newest last.
    pub tasks: Vec<PublicWork>,
}

/// `GET /api/tasks`.
pub async fn list(
    State(state): State<AppState>,
    identity: RequestIdentity,
    Query(query): Query<TaskListQuery>,
) -> Json<TaskListBody> {
    let tasks = state
        .services()
        .work
        .list(query.to_work_query(&identity.owner_id))
        .await;
    Json(TaskListBody { tasks })
}

/// One inline presentation block, as `/api/timeline` publishes it.
///
/// **External contract** — `gateway-application.mjs:325-340`. The `inline_`
/// prefix is reproduced verbatim by the `timeline.inline` WebSocket event, so
/// the two surfaces agree on an id.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineItem {
    /// `inline_<taskId>`.
    pub id: String,
    /// The Work this block came from.
    pub task_id: String,
    /// The turn that raised it, or `null`.
    pub turn_id: Option<String>,
    /// `completedAt` when there is one, otherwise `createdAt`.
    pub created_at: i64,
    /// The block's title.
    pub title: String,
    /// `markdown` | `code` | `link`.
    pub format: &'static str,
    /// The block's body.
    pub content: String,
}

/// The `inline_` id prefix.
///
/// **External contract** — `gateway-application.mjs:334`.
pub const TIMELINE_ID_PREFIX: &str = "inline_";

/// `{ "items": [...] }`.
#[derive(Debug, Clone, Serialize)]
pub struct TimelineBody {
    /// The inline blocks, ascending by `createdAt`.
    pub items: Vec<TimelineItem>,
}

/// `GET /api/timeline`.
///
/// Note there is **no** `active` filter here: upstream calls `list()` with only
/// `ownerId` and `sessionId` (`gateway-application.mjs:326-329`), because a
/// timeline of only-running Work would be empty of exactly the finished results
/// it exists to show.
pub async fn timeline(
    State(state): State<AppState>,
    identity: RequestIdentity,
    Query(query): Query<TaskListQuery>,
) -> Json<TimelineBody> {
    let mut work_query = WorkQuery::owner(&identity.owner_id);
    work_query.session_id = query.session_id.clone();
    let tasks = state.services().work.list(work_query).await;
    Json(TimelineBody {
        items: project_timeline(tasks),
    })
}

/// The pure half of [`timeline`].
///
/// **External contract** — `gateway-application.mjs:330-338`, four decisions:
/// keep only Work whose `resultMetadata.presentation.inline.content` exists;
/// prefix the id with `inline_`; take `completedAt` when there is one and
/// `createdAt` otherwise; and sort **ascending** by that value.
///
/// Split out because every one of the four is a one-token mutation away from a
/// plausible-looking wrong answer, and a socket is a poor place to notice.
#[must_use]
pub fn project_timeline(tasks: Vec<PublicWork>) -> Vec<TimelineItem> {
    let mut items: Vec<TimelineItem> = tasks
        .into_iter()
        .filter_map(|task| {
            let inline = task.result_metadata.as_ref()?.presentation.inline.clone()?;
            Some(TimelineItem {
                id: format!("{TIMELINE_ID_PREFIX}{}", task.id),
                task_id: task.id.clone(),
                turn_id: task.turn_id.clone(),
                created_at: task.completed_at.unwrap_or(task.created_at),
                title: inline.title,
                format: inline.format.as_str(),
                content: inline.content,
            })
        })
        .collect();
    items.sort_by_key(|item| item.created_at);
    items
}

/// The 404 body of every Work lookup.
///
/// **External contract** — `gateway-application.mjs:344,351`:
/// `{"error":"task not found"}`, in English in every locale, because clients
/// branch on the string.
fn not_found(state: &AppState) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": t(state.locale(), keys::GATEWAY_TASK_NOT_FOUND) })),
    )
        .into_response()
}

/// `GET /api/tasks/{id}`.
pub async fn get(
    State(state): State<AppState>,
    identity: RequestIdentity,
    Path(id): Path<String>,
) -> Response {
    match state
        .services()
        .work
        .get(&id, Some(&identity.owner_id))
        .await
    {
        Some(task) => Json(task).into_response(),
        None => not_found(&state),
    }
}

/// `DELETE /api/tasks/{id}`.
///
/// **External contract** — `gateway-application.mjs:348-363`, all three arms:
/// 404 when unknown, **409 with the existing task beside the error** when
/// `cancel` answers nothing, 200 with the cancelled task otherwise.
///
/// The two lookups are upstream's own and are not redundant: the first decides
/// 404-versus-409 and the second is the cancellation. A single `cancel` call
/// could not tell "no such Work" from "already terminal".
pub async fn cancel(
    State(state): State<AppState>,
    identity: RequestIdentity,
    Path(id): Path<String>,
) -> Response {
    let work = &state.services().work;
    let Some(existing) = work.get(&id, Some(&identity.owner_id)).await else {
        return not_found(&state);
    };
    match work.cancel(&id, Some(&identity.owner_id)).await {
        Some(task) => Json(task).into_response(),
        None => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": t(state.locale(), keys::GATEWAY_TASK_NOT_ACTIVE),
                "task": existing,
            })),
        )
            .into_response(),
    }
}

/// The synthetic first frame of the SSE stream.
///
/// **External contract** — `gateway-application.mjs:414`. `task.snapshot` is
/// **not** a member of [`GatewayTaskEvent`]: it exists only on this endpoint,
/// which is why it is a literal here and not a protocol constant.
pub const SSE_SNAPSHOT_TYPE: &str = "task.snapshot";

/// `GET /api/tasks/{id}/events` — Server-Sent Events.
///
/// **External contract** — `gateway-application.mjs:406-421`. The 404 arm comes
/// first, then the three headers, then `data: {…}\n\n` frames: a
/// `task.snapshot` carrying the Work as it stands, and afterwards one frame per
/// Work event that matches **both** this owner and this id.
///
/// # Deviation: no keep-alive comment upstream, one here
///
/// Upstream writes nothing between events. A `tokio` SSE stream behind a proxy
/// with an idle timeout is closed silently, so [`KeepAlive`] emits a comment
/// line. Comments are not events: a client's `EventSource` never surfaces them,
/// and the frame format the catalogue names is untouched.
pub async fn events(
    State(state): State<AppState>,
    identity: RequestIdentity,
    Path(id): Path<String>,
) -> Response {
    let work = &state.services().work;
    let Some(snapshot) = work.get(&id, Some(&identity.owner_id)).await else {
        return not_found(&state);
    };
    let owner_id = identity.owner_id.clone();
    let subscription = work.subscribe();

    let first = futures::stream::once(async move {
        Ok::<Event, Infallible>(sse_frame(SSE_SNAPSHOT_TYPE, &snapshot))
    });
    let rest = BroadcastStream::new(subscription).filter_map(move |event| {
        let event = event.ok()?;
        if event.owner_id != owner_id || event.task.id != id {
            return None;
        }
        Some(Ok::<Event, Infallible>(sse_frame(
            event.kind.as_str(),
            &event.task,
        )))
    });

    Sse::new(first.chain(rest))
        .keep_alive(KeepAlive::default())
        .into_response()
}

/// `data: {"type":…,"task":…}`.
fn sse_frame(kind: &str, task: &PublicWork) -> Event {
    let payload = json!({ "type": kind, "task": task });
    Event::default().data(payload.to_string())
}

/// Whether a Work event is one a client may see on the socket.
///
/// **External contract** — `shared/realtime-events.mjs:52-67` as narrowed by
/// [`GatewayTaskEvent`]'s own documentation: `task.accepted`,
/// `task.notification.pending` and `task.notification.delivered` are emitted
/// internally and never declared, and `task.progress.check` is declared but
/// intercepted at the socket. The SSE endpoint applies **neither** filter — it
/// forwards `event.type` verbatim, which is upstream's behaviour and the reason
/// this helper is public rather than used here.
#[must_use]
pub fn is_client_visible(kind: via_work::WorkEventKind) -> bool {
    GatewayTaskEvent::from_wire(kind.as_str()).is_some()
}

/// The JSON a `task.snapshot` frame carries, for tests and for `via-e2e`.
#[must_use]
pub fn snapshot_frame(task: &PublicWork) -> Value {
    json!({ "type": SSE_SNAPSHOT_TYPE, "task": task })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use via_work::WorkEventKind;
    use via_work::presentation::{InlineBlock, InlineFormat, Presentation, PublicResultMetadata};

    fn work(
        id: &str,
        created_at: i64,
        completed_at: Option<i64>,
        content: Option<&str>,
    ) -> PublicWork {
        let mut task = via_work::testing::blank_record(id, "user_1").to_public(0);
        task.created_at = created_at;
        task.completed_at = completed_at;
        task.turn_id = Some("voice-1".to_owned());
        task.result_metadata = content.map(|content| PublicResultMetadata {
            presentation: Presentation {
                speech: "done".to_owned(),
                inline: Some(InlineBlock {
                    title: "Diff".to_owned(),
                    format: InlineFormat::Code,
                    content: content.to_owned(),
                }),
            },
        });
        task
    }

    #[test]
    fn active_is_compared_to_the_literal_string_true() {
        let query = |value: Option<&str>| TaskListQuery {
            session_id: None,
            active: value.map(str::to_owned),
        };
        assert!(query(Some("true")).to_work_query("user_1").active_only);
        assert!(!query(Some("1")).to_work_query("user_1").active_only);
        assert!(!query(Some("TRUE")).to_work_query("user_1").active_only);
        assert!(!query(None).to_work_query("user_1").active_only);
    }

    #[test]
    fn the_timeline_id_prefix_is_the_one_the_socket_uses() {
        assert_eq!(TIMELINE_ID_PREFIX, "inline_");
        assert_eq!(format!("{TIMELINE_ID_PREFIX}work_1"), "inline_work_1");
    }

    #[test]
    fn the_timeline_keeps_only_work_with_an_inline_block() {
        let items = project_timeline(vec![
            work("work_1", 10, None, Some("diff")),
            work("work_2", 20, None, None),
        ]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].task_id, "work_1");
        assert_eq!(items[0].id, "inline_work_1");
        assert_eq!(items[0].format, "code");
        assert_eq!(items[0].title, "Diff");
        assert_eq!(items[0].turn_id.as_deref(), Some("voice-1"));
    }

    #[test]
    fn the_timeline_prefers_completed_at_and_sorts_ascending() {
        let items = project_timeline(vec![
            work("work_late", 1, Some(300), Some("b")),
            work("work_early", 2, Some(100), Some("a")),
            work("work_unfinished", 200, None, Some("c")),
        ]);
        let order: Vec<(&str, i64)> = items
            .iter()
            .map(|item| (item.task_id.as_str(), item.created_at))
            .collect();
        assert_eq!(
            order,
            [
                ("work_early", 100),
                ("work_unfinished", 200),
                ("work_late", 300),
            ],
            "completedAt wins where there is one, and the sort is ascending",
        );
    }

    #[test]
    fn the_three_manager_only_events_are_not_client_visible() {
        assert!(!is_client_visible(WorkEventKind::Accepted));
        assert!(!is_client_visible(WorkEventKind::NotificationPending));
        assert!(!is_client_visible(WorkEventKind::NotificationDelivered));
        assert!(is_client_visible(WorkEventKind::Completed));
        assert!(
            is_client_visible(WorkEventKind::ProgressCheck),
            "declared in the vocabulary; the socket intercepts it, the SSE stream does not"
        );
    }
}
