//! The activity ring — generic tool progress, and nothing else.
//!
//! `server/src/task/task-manager.mjs:527-536` is the whole of it: a
//! `backend.activity` event carrying an already-projected activity object is
//! merged into `task.activity` by `id`, the list is cut to the last twenty
//! entries, and a non-persisting `task.progress` is emitted.
//!
//! # Why this type exists rather than [`SessionEvent`]
//!
//! `via_downstream::SessionEvent` **is** the projection, and it is the only way
//! an [`Activity`] is ever built — [`Activity::from`] is the sole constructor
//! that a caller can reach with a real event. But the ring is persisted, and
//! `SessionEvent`'s three payload shapes have private fields and no
//! deserializer, by design: the Layer-3 boundary must not be reconstructible
//! from arbitrary JSON.
//!
//! So the ring holds this: the same nine fields, in the same order, skipping
//! exactly the ones each of the three shapes omits, and readable back off disk.
//! `tests/activity.rs` asserts that serializing an `Activity` built from a
//! `SessionEvent` is byte-identical to serializing the `SessionEvent` itself,
//! for all three shapes — which is what keeps the two from drifting.
//!
//! [`SessionEvent`]: via_downstream::SessionEvent

use serde::{Deserialize, Serialize};
use via_downstream::SessionEvent;

/// How many activity entries a Work keeps.
///
/// Contract — `server/src/task/task-manager.mjs:534`
/// (`task.activity.slice(-20)`), catalogued as *"activity ring=last 20
/// entries"*.
pub const ACTIVITY_RING: usize = 20;

/// One entry of the activity ring.
///
/// Field order is `via_downstream::SessionEvent`'s serialization order, which
/// is the catalogued `backend.activity` payload order. Every field a given
/// shape does not carry is skipped, except `id`, which is always present and
/// `null` for a text activity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Activity {
    /// The id the ring dedupes on: `acp-plan` for a plan, the tool call id for
    /// a tool call, absent for text.
    #[serde(default)]
    pub id: Option<String>,
    /// `plan` | `tool` | `text`.
    ///
    /// Defaulted on read so one malformed entry written by an older Gateway
    /// cannot make a whole Work record unparseable.
    #[serde(default)]
    pub kind: String,
    /// The tool's name. Tool activities only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    /// The tool call's human label. Tool activities only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Where the activity is: `pending` | `in_progress` | `completed` |
    /// `failed` | `running`.
    ///
    /// A string rather than an enum because it is interpolated verbatim into
    /// the progress-check announcement (`task-manager.mjs:607`) and read back
    /// off disk from records an older Gateway wrote.
    #[serde(default)]
    pub status: String,
    /// One of the five generic tool categories. Tool activities only, and the
    /// key into the progress-check verb table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// The bounded, user-safe detail. Plan and tool activities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// How many plan entries are done. Plan activities only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed: Option<usize>,
    /// How many plan entries there are. Plan activities only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<usize>,
}

impl From<&SessionEvent> for Activity {
    /// Project an adapter-normalised event into a ring entry.
    ///
    /// Everything bounded stays bounded: this reads
    /// [`SessionEvent`]'s accessors and copies, it never re-derives.
    fn from(event: &SessionEvent) -> Self {
        let base = Self {
            id: event.id().map(str::to_owned),
            kind: event.kind().to_owned(),
            tool: None,
            label: None,
            status: event.status().as_str().to_owned(),
            category: None,
            detail: None,
            completed: None,
            total: None,
        };
        match event {
            SessionEvent::Plan(plan) => Self {
                detail: Some(plan.detail().to_owned()),
                completed: Some(plan.completed()),
                total: Some(plan.total()),
                ..base
            },
            SessionEvent::Tool(tool) => Self {
                tool: Some(tool.tool().to_owned()),
                label: Some(tool.label().to_owned()),
                category: Some(tool.category().as_str().to_owned()),
                detail: Some(tool.detail().to_owned()),
                ..base
            },
            SessionEvent::Text => base,
        }
    }
}

impl From<SessionEvent> for Activity {
    fn from(event: SessionEvent) -> Self {
        Self::from(&event)
    }
}

/// Merge `activity` into `ring`, then cut the ring to [`ACTIVITY_RING`].
///
/// `server/src/task/task-manager.mjs:529-534`: an entry with an `id` that is
/// already present **replaces it in place**, keeping its position; an entry
/// with no id, or with an unseen one, is appended. Then the last twenty
/// survive.
///
/// The in-place replacement is why a long-running tool call does not push the
/// rest of the ring out by updating: `tool_call` and every `tool_call_update`
/// that follows share one id.
pub fn merge(ring: &mut Vec<Activity>, activity: Activity) {
    let index = activity.id.as_deref().and_then(|id| {
        ring.iter()
            .position(|entry| entry.id.as_deref() == Some(id))
    });
    match index {
        Some(index) => ring[index] = activity,
        None => ring.push(activity),
    }
    if ring.len() > ACTIVITY_RING {
        ring.drain(..ring.len() - ACTIVITY_RING);
    }
}

#[cfg(test)]
mod tests {
    use super::{ACTIVITY_RING, Activity, merge};
    use via_downstream::{RawSessionUpdate, SessionEvent};

    fn tool(id: &str) -> Activity {
        let mut tracker = via_downstream::ActivityTracker::new();
        let update = RawSessionUpdate {
            name: Some("bash".to_owned()),
            ..RawSessionUpdate::tool_call(id)
        };
        Activity::from(&tracker.project(&update).expect("a tool call projects"))
    }

    #[test]
    fn an_update_replaces_its_earlier_entry_in_place() {
        let mut ring = vec![tool("one"), tool("two")];
        let mut updated = tool("one");
        updated.status = "completed".to_owned();
        merge(&mut ring, updated);
        assert_eq!(ring.len(), 2);
        assert_eq!(ring[0].id.as_deref(), Some("one"));
        assert_eq!(ring[0].status, "completed");
        assert_eq!(ring[1].id.as_deref(), Some("two"));
    }

    #[test]
    fn an_entry_with_no_id_is_always_appended() {
        let mut ring = Vec::new();
        for _ in 0..3 {
            merge(&mut ring, Activity::from(&SessionEvent::Text));
        }
        assert_eq!(ring.len(), 3);
    }

    #[test]
    fn the_ring_keeps_the_last_twenty() {
        let mut ring = Vec::new();
        for index in 0..(ACTIVITY_RING + 5) {
            merge(&mut ring, tool(&format!("call-{index}")));
        }
        assert_eq!(ring.len(), ACTIVITY_RING);
        assert_eq!(ring[0].id.as_deref(), Some("call-5"));
        assert_eq!(ring[ACTIVITY_RING - 1].id.as_deref(), Some("call-24"));
    }
}
