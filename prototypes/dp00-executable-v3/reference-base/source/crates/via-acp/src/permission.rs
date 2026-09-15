//! Answering `session/request_permission`.
//!
//! Ported from `server/src/agent/acp-process-client.mjs:174-177,273-287` and
//! the reply mapping in `server/src/agent/permission-broker.mjs:19-27,45-50`.
//!
//! # The kind-preference order is semantics, not style
//!
//! Catalogued under *"session/request_permission"* and *"ACP requestPermission
//! reply mapping"*: an approval prefers `allow_once` and falls back to
//! `allow_always`; a rejection prefers `reject_always` and falls back to
//! `reject_once`. The asymmetry is deliberate and it is the user's
//! **persisted** permission state in the backend that is at stake. Reversing
//! either list turns "yes, this once" into a standing grant, or "no" into a
//! refusal the backend will ask about again every time.
//!
//! # With no handler installed, the answer is `cancelled`
//!
//! Not "reject": a rejection is a decision, and there was nobody to make one.
//! Upstream returns `{outcome:{outcome:'cancelled'}}`
//! (`acp-process-client.mjs:274-276`), and so does this.

use agent_client_protocol::schema::v1::{
    PermissionOption, PermissionOptionId, PermissionOptionKind, RequestPermissionOutcome,
    RequestPermissionResponse, SelectedPermissionOutcome,
};
use serde_json::Value;

/// What the Gateway decided about one permission request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionDecision {
    /// Allow the operation.
    Approve,
    /// Refuse it.
    Reject,
    /// Nobody decided — the turn is going away.
    Cancel,
}

/// The option kinds an approval will accept, in preference order.
///
/// **External contract** — `permission-broker.mjs:45-50`.
pub const APPROVE_KIND_ORDER: [PermissionOptionKind; 2] = [
    PermissionOptionKind::AllowOnce,
    PermissionOptionKind::AllowAlways,
];

/// The option kinds a rejection will accept, in preference order.
///
/// **External contract** — `permission-broker.mjs:45-50`. Note this is
/// `always` **first**, the mirror image of [`APPROVE_KIND_ORDER`].
pub const REJECT_KIND_ORDER: [PermissionOptionKind; 2] = [
    PermissionOptionKind::RejectAlways,
    PermissionOptionKind::RejectOnce,
];

/// One inbound permission request, as the Gateway reads it.
///
/// Read from the raw notification rather than the SDK's typed
/// `RequestPermissionRequest` for the same reason [`crate::session`] does: the
/// `toolCall` an agent sends carries a `name` the v1 schema does not declare,
/// and a typed round-trip would drop it.
#[derive(Debug, Clone, PartialEq)]
pub struct PermissionRequest {
    /// Which session is asking.
    pub session_id: String,
    /// The tool call, verbatim. Upstream reads `name`, `title` and
    /// `rawInput.{description,command,path}` out of it.
    pub tool_call: Value,
    /// The options the agent offered.
    pub options: Vec<PermissionOption>,
}

impl PermissionRequest {
    /// The tool's name, preferring `name` over `title` — the order the UI uses.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        self.tool_call
            .get("name")
            .and_then(Value::as_str)
            .filter(|name| !name.trim().is_empty())
            .or_else(|| self.tool_call.get("title").and_then(Value::as_str))
            .unwrap_or_default()
    }

    /// The human-readable description the agent supplied, if any.
    #[must_use]
    pub fn description(&self) -> &str {
        self.tool_call
            .get("rawInput")
            .and_then(|input| input.get("description"))
            .and_then(Value::as_str)
            .unwrap_or_default()
    }
}

/// Pick the option that expresses `decision`.
///
/// **Contract** — `permission-broker.mjs:45-50`. Returns `None` when the agent
/// offered no option of an acceptable kind, which is answered as `cancelled`:
/// choosing an option of the *wrong* kind would mean answering a question the
/// user was never asked.
#[must_use]
pub fn select_option(
    options: &[PermissionOption],
    decision: PermissionDecision,
) -> Option<PermissionOptionId> {
    let order: &[PermissionOptionKind] = match decision {
        PermissionDecision::Approve => &APPROVE_KIND_ORDER,
        PermissionDecision::Reject => &REJECT_KIND_ORDER,
        PermissionDecision::Cancel => return None,
    };
    order.iter().find_map(|kind| {
        options
            .iter()
            .find(|option| option.kind == *kind)
            .map(|option| option.option_id.clone())
    })
}

/// Build the reply for `decision` against the offered `options`.
///
/// **Contract** — the response is **always** one of exactly two shapes:
/// `{"outcome":{"outcome":"selected","optionId":<id>}}` or
/// `{"outcome":{"outcome":"cancelled"}}`. There is no third answer and no
/// error reply; an agent that offered nothing usable is told the request was
/// cancelled.
#[must_use]
pub fn reply(
    options: &[PermissionOption],
    decision: PermissionDecision,
) -> RequestPermissionResponse {
    match select_option(options, decision) {
        Some(option_id) => RequestPermissionResponse::new(RequestPermissionOutcome::Selected(
            SelectedPermissionOutcome::new(option_id),
        )),
        None => RequestPermissionResponse::new(RequestPermissionOutcome::Cancelled),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn option(id: &str, kind: PermissionOptionKind) -> PermissionOption {
        PermissionOption::new(PermissionOptionId::new(id.to_owned()), id.to_owned(), kind)
    }

    fn all_four() -> Vec<PermissionOption> {
        vec![
            option("reject_once", PermissionOptionKind::RejectOnce),
            option("allow_always", PermissionOptionKind::AllowAlways),
            option("reject_always", PermissionOptionKind::RejectAlways),
            option("allow_once", PermissionOptionKind::AllowOnce),
        ]
    }

    #[test]
    fn approval_prefers_once_and_rejection_prefers_always() {
        // The catalogued asymmetry, asserted in both directions. The options
        // are deliberately in a different order from the preference lists, so
        // a port that simply took the first offered option fails here.
        assert_eq!(
            select_option(&all_four(), PermissionDecision::Approve).map(|id| id.0.to_string()),
            Some("allow_once".to_owned())
        );
        assert_eq!(
            select_option(&all_four(), PermissionDecision::Reject).map(|id| id.0.to_string()),
            Some("reject_always".to_owned())
        );
    }

    #[test]
    fn each_preference_falls_back_to_its_second_choice() {
        let no_once = vec![option("allow_always", PermissionOptionKind::AllowAlways)];
        assert_eq!(
            select_option(&no_once, PermissionDecision::Approve).map(|id| id.0.to_string()),
            Some("allow_always".to_owned())
        );
        let no_always = vec![option("reject_once", PermissionOptionKind::RejectOnce)];
        assert_eq!(
            select_option(&no_always, PermissionDecision::Reject).map(|id| id.0.to_string()),
            Some("reject_once".to_owned())
        );
    }

    #[test]
    fn a_decision_with_no_matching_kind_is_cancelled_not_guessed() {
        let only_allow = vec![option("allow_once", PermissionOptionKind::AllowOnce)];
        assert_eq!(select_option(&only_allow, PermissionDecision::Reject), None);
        assert_eq!(
            serde_json::to_value(reply(&only_allow, PermissionDecision::Reject))
                .expect("serializes"),
            serde_json::json!({ "outcome": { "outcome": "cancelled" } })
        );
    }

    #[test]
    fn the_two_reply_shapes_are_exactly_the_catalogued_ones() {
        assert_eq!(
            serde_json::to_value(reply(&all_four(), PermissionDecision::Approve))
                .expect("serializes"),
            serde_json::json!({
                "outcome": { "outcome": "selected", "optionId": "allow_once" }
            })
        );
        assert_eq!(
            serde_json::to_value(reply(&all_four(), PermissionDecision::Cancel))
                .expect("serializes"),
            serde_json::json!({ "outcome": { "outcome": "cancelled" } })
        );
        assert_eq!(
            serde_json::to_value(reply(&[], PermissionDecision::Approve)).expect("serializes"),
            serde_json::json!({ "outcome": { "outcome": "cancelled" } }),
            "an agent that offered nothing gets `cancelled`, never a fabricated id"
        );
    }

    #[test]
    fn a_permission_request_reads_the_undeclared_name_field() {
        let request = PermissionRequest {
            session_id: "s".into(),
            tool_call: serde_json::json!({
                "name": "shell",
                "title": "Run tests",
                "rawInput": { "description": "验证项目测试", "command": "npm test" },
            }),
            options: Vec::new(),
        };
        assert_eq!(request.tool_name(), "shell");
        assert_eq!(request.description(), "验证项目测试");

        let title_only = PermissionRequest {
            session_id: "s".into(),
            tool_call: serde_json::json!({ "title": "Run tests" }),
            options: Vec::new(),
        };
        assert_eq!(title_only.tool_name(), "Run tests");
        assert_eq!(title_only.description(), "");

        let blank_name = PermissionRequest {
            session_id: "s".into(),
            tool_call: serde_json::json!({ "name": "  ", "title": "Run tests" }),
            options: Vec::new(),
        };
        assert_eq!(blank_name.tool_name(), "Run tests");
    }
}
