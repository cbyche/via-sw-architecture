//! The delegated project session a Work is waiting on, and the correlation
//! that decides which completion may finish it.
//!
//! Two shapes, deliberately different:
//!
//! | | Carries | Where it goes |
//! | --- | --- | --- |
//! | [`DelegationRef`] | id, session id, directory, title, presentation, status | `tasks.json`, and the recovery seam |
//! | [`PublicDelegation`] | status, bounded title, bounded presentation | every client, every SSE frame |
//!
//! `publicTask` (`server/src/task/task-manager.mjs:76-89`) drops the delegation
//! id, the backend session id and the working directory on the way out.
//! `persistedTask` (`:238-245`) then puts the **full** record back, deep-copied,
//! because restart recovery needs exactly the three fields the public shape
//! removed. That asymmetry is the contract, not an accident.
//!
//! # Correlation
//!
//! `docs/architecture.md` §11, invariant 4:
//!
//! > **Delegation correlation.** Only the completion correlated to that
//! > delegation id may complete the Work.
//!
//! [`DelegationRef::correlates_with`] is that test, and the Work manager
//! applies it to every `backend.delegation.completed` it receives. A stale
//! result, a result for a sibling delegation, or a result that arrives after
//! the Work has been re-delegated is *dropped*, not published.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use via_downstream::text::clean;

use crate::text::slice_units;

/// Bound on the published delegation title.
///
/// Contract — `server/src/task/task-manager.mjs:79`.
pub const DELEGATION_TITLE_BOUND: usize = 160;

/// Bound on the published delegation speech.
///
/// Contract — `server/src/task/task-manager.mjs:84`.
pub const DELEGATION_SPEECH_BOUND: usize = 1200;

/// The status a delegation defaults to when the adapter names none.
///
/// Contract — `server/src/task/task-manager.mjs:78`
/// (`task.delegation.status || 'running'`).
pub const DELEGATION_DEFAULT_STATUS: &str = "running";

/// The status a completed delegation is stamped with.
///
/// Contract — `server/src/task/task-manager.mjs:523`
/// (`{...event.delegation, status: 'completed'}`).
pub const DELEGATION_COMPLETED_STATUS: &str = "completed";

/// A delegated project session, in full.
///
/// This is the internal and persisted shape. `id` is the delegation id the
/// adapter minted — `<protocol>_run_<uuid>` on the MCP-tool path, or the
/// backend's own `runId` on the native path — and `session_id` is the backend's
/// own session id. Both round-trip through `tasks.json` because restart
/// recovery reattaches with them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationRef {
    /// The delegation id. **The correlation key.**
    pub id: String,
    /// The backend's session id for the delegated target.
    #[serde(rename = "sessionId")]
    pub session_id: String,
    /// The target's working directory, when the adapter reported one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,
    /// The human title the coordinator gave the delegated work.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// `running` until the delegated session reports completion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// What the user was told when the delegation was announced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presentation: Option<Value>,
}

impl DelegationRef {
    /// A delegation naming a target.
    #[must_use]
    pub fn new(id: &str, session_id: &str) -> Self {
        Self {
            id: id.to_owned(),
            session_id: session_id.to_owned(),
            directory: None,
            title: None,
            status: None,
            presentation: None,
        }
    }

    /// Set the human title.
    #[must_use]
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_owned());
        self
    }

    /// Set the working directory.
    #[must_use]
    pub fn with_directory(mut self, directory: &str) -> Self {
        self.directory = Some(directory.to_owned());
        self
    }

    /// Set the announcement presentation.
    #[must_use]
    pub fn with_presentation(mut self, presentation: Value) -> Self {
        self.presentation = Some(presentation);
        self
    }

    /// Whether this delegation is a usable correlation target at all.
    ///
    /// `canRecoverDelegation` (`server/src/task/task-manager.mjs:183-187`)
    /// requires **both** ids: a delegation with only one of them names no
    /// session anybody can reattach to or cancel, so it is not recoverable and
    /// nothing may correlate against it.
    #[must_use]
    pub fn is_addressable(&self) -> bool {
        !clean(&self.id).is_empty() && !clean(&self.session_id).is_empty()
    }

    /// Whether `other` reports on **this** delegation.
    ///
    /// The delegation id is the key. The session id is checked too when both
    /// sides carry one, because a backend that reused a delegation id across
    /// two sessions would otherwise let a stale target close live Work.
    #[must_use]
    pub fn correlates_with(&self, other: &Self) -> bool {
        if clean(&self.id).is_empty() || clean(&self.id) != clean(&other.id) {
            return false;
        }
        let mine = clean(&self.session_id);
        let theirs = clean(&other.session_id);
        mine.is_empty() || theirs.is_empty() || mine == theirs
    }

    /// Stamp this delegation as completed, keeping everything else.
    #[must_use]
    pub fn completed(mut self) -> Self {
        self.status = Some(DELEGATION_COMPLETED_STATUS.to_owned());
        self
    }

    /// The published projection.
    ///
    /// `server/src/task/task-manager.mjs:76-89`.
    #[must_use]
    pub fn to_public(&self) -> PublicDelegation {
        PublicDelegation {
            status: self
                .status
                .clone()
                .filter(|status| !status.is_empty())
                .unwrap_or_else(|| DELEGATION_DEFAULT_STATUS.to_owned()),
            title: slice_units(
                self.title.as_deref().unwrap_or_default(),
                DELEGATION_TITLE_BOUND,
            ),
            presentation: self.presentation.as_ref().and_then(|presentation| {
                let object = presentation.as_object()?;
                Some(DelegationPresentation {
                    speech: slice_units(
                        object
                            .get("speech")
                            .and_then(Value::as_str)
                            .unwrap_or_default(),
                        DELEGATION_SPEECH_BOUND,
                    ),
                    inline: object
                        .get("inline")
                        .filter(|inline| !inline.is_null())
                        .cloned(),
                })
            }),
        }
    }
}

/// The delegation as a client sees it.
///
/// No delegation id, no backend session id, no directory —
/// `docs/architecture.md` §17's third review question, enforced by the type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicDelegation {
    /// `running` until completion.
    pub status: String,
    /// Bounded to [`DELEGATION_TITLE_BOUND`].
    pub title: String,
    /// What the user was told, bounded.
    pub presentation: Option<DelegationPresentation>,
}

/// The published announcement for a delegation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationPresentation {
    /// Bounded to [`DELEGATION_SPEECH_BOUND`].
    pub speech: String,
    /// Passed through unprojected, or `null`.
    ///
    /// Contract — `server/src/task/task-manager.mjs:85`
    /// (`inline: task.delegation.presentation.inline || null`): unlike
    /// `resultMetadata`'s inline block this one is **not** re-shaped, so a
    /// caller reads it as opaque JSON.
    pub inline: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::{DELEGATION_SPEECH_BOUND, DELEGATION_TITLE_BOUND, DelegationRef};
    use serde_json::json;

    #[test]
    fn the_public_shape_names_no_session() {
        let delegation = DelegationRef::new("run-one", "agent:child:one")
            .with_title("project")
            .with_directory("/private/project");
        let rendered = serde_json::to_value(delegation.to_public()).expect("serializes");
        let text = rendered.to_string();
        assert!(!text.contains("run-one"), "{text}");
        assert!(!text.contains("agent:child:one"), "{text}");
        assert!(!text.contains("/private/project"), "{text}");
    }

    #[test]
    fn the_public_status_defaults_to_running() {
        let public = DelegationRef::new("d", "s").to_public();
        assert_eq!(public.status, "running");
        assert_eq!(public.title, "");
        assert!(public.presentation.is_none());
    }

    #[test]
    fn the_public_title_and_speech_are_bounded() {
        let delegation = DelegationRef::new("d", "s")
            .with_title(&"t".repeat(DELEGATION_TITLE_BOUND + 40))
            .with_presentation(json!({
                "speech": "s".repeat(DELEGATION_SPEECH_BOUND + 40),
                "inline": {"anything": true},
            }));
        let public = delegation.to_public();
        assert_eq!(public.title.chars().count(), DELEGATION_TITLE_BOUND);
        let presentation = public.presentation.expect("present");
        assert_eq!(presentation.speech.chars().count(), DELEGATION_SPEECH_BOUND);
        assert_eq!(presentation.inline, Some(json!({"anything": true})));
    }

    #[test]
    fn only_the_correlated_delegation_reports_on_this_one() {
        let ours = DelegationRef::new("run-one", "target-one");
        assert!(ours.correlates_with(&DelegationRef::new("run-one", "target-one")));
        assert!(
            !ours.correlates_with(&DelegationRef::new("run-two", "target-one")),
            "a sibling delegation is not this one",
        );
        assert!(
            !ours.correlates_with(&DelegationRef::new("run-one", "target-two")),
            "a reused id against a different session is not this one",
        );
        assert!(
            ours.correlates_with(&DelegationRef {
                session_id: String::new(),
                ..DelegationRef::new("run-one", "")
            }),
            "an adapter that reports only the delegation id still correlates",
        );
    }

    #[test]
    fn an_empty_delegation_id_correlates_with_nothing() {
        let empty = DelegationRef::new("", "target");
        assert!(!empty.correlates_with(&DelegationRef::new("", "target")));
        assert!(!empty.is_addressable());
        assert!(!DelegationRef::new("run", "  ").is_addressable());
        assert!(DelegationRef::new("run", "target").is_addressable());
    }

    #[test]
    fn the_full_shape_round_trips_through_json() {
        let delegation = DelegationRef::new("run-one", "agent:child:one")
            .with_directory("/project")
            .with_title("项目任务");
        let rendered = serde_json::to_value(&delegation).expect("serializes");
        assert_eq!(rendered["sessionId"], "agent:child:one");
        let parsed: DelegationRef = serde_json::from_value(rendered).expect("parses");
        assert_eq!(parsed, delegation);
    }
}
