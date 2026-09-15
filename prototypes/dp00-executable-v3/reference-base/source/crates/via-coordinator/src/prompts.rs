//! The four out-of-band blocks the coordinator session is sent.
//!
//! Every one is model-visible and catalogued, and every one is an *injection*
//! into a session whose history the model also reads — so the tag around each
//! is what tells the model where Gateway-verified fact ends and user request
//! begins.
//!
//! | Block | What it says | Upstream |
//! | --- | --- | --- |
//! | [`protocol_retry_prompt`] | *that reply's `state` is not deliverable; finish the work* | `coordinator.mjs:270-276` |
//! | [`delegation_result_prompt`] | *here is a verified Layer-3 result; present it* | `acp-backend-adapter.mjs:1139-1155` |
//! | [`reconciliation_prompt`] | *the Gateway already did this; do not repeat it* | `acp-backend-adapter.mjs:1007-1017` |
//! | [`cancel_control_prompt`] / [`status_control_prompt`] | the two hidden control turns | `acp-backend-adapter.mjs:1388-1395,1433-1443` |
//!
//! All five tags are renamed per `docs/rebrand.md` rows 31-36. The prose inside
//! them comes from `via-i18n`; only the tags and the two size bounds are
//! literals here.

use serde_json::{Map, Value};
use via_downstream::text::clean;
use via_i18n::{Locale, format, keys, t};
use via_work::CancellationFact;
use via_work::text::slice_units;

/// The opening tag of the protocol-retry block.
///
/// **External contract**, renamed. Upstream `<qwen_audio_agent_protocol_retry>`
/// (`server/src/agent/coordinator.mjs:271`); `docs/rebrand.md` row 35.
pub const PROTOCOL_RETRY_OPEN_TAG: &str = "<via_protocol_retry>";

/// The closing tag of the protocol-retry block.
pub const PROTOCOL_RETRY_CLOSE_TAG: &str = "</via_protocol_retry>";

/// The opening tag of the delegation-result block.
///
/// **External contract**, renamed. Upstream
/// `<qwen_audio_agent_delegation_result>`
/// (`acp-backend-adapter.mjs:1141`); `docs/rebrand.md` row 33.
pub const DELEGATION_RESULT_OPEN_TAG: &str = "<via_delegation_result>";

/// The closing tag of the delegation-result block.
pub const DELEGATION_RESULT_CLOSE_TAG: &str = "</via_delegation_result>";

/// The opening tag of the reconciliation block.
///
/// **External contract**, renamed. Upstream `<qwen_audio_agent_reconciliation>`
/// (`acp-backend-adapter.mjs:1010`); `docs/rebrand.md` row 36.
pub const RECONCILIATION_OPEN_TAG: &str = "<via_reconciliation>";

/// The closing tag of the reconciliation block.
pub const RECONCILIATION_CLOSE_TAG: &str = "</via_reconciliation>";

/// The opening tag of the cancel control turn.
///
/// **External contract**, renamed. Upstream
/// `<qwen_audio_agent_control kind="cancel">` (`acp-backend-adapter.mjs:1391`);
/// `docs/rebrand.md` row 32. The `kind` attribute is part of the tag, not a
/// parameter: the model is shown two distinct openings, never one with a
/// variable in it.
pub const CONTROL_CANCEL_OPEN_TAG: &str = "<via_control kind=\"cancel\">";

/// The opening tag of the status control turn.
///
/// **External contract**, renamed. Upstream
/// `<qwen_audio_agent_control kind="status">` (`acp-backend-adapter.mjs:1436`).
pub const CONTROL_STATUS_OPEN_TAG: &str = "<via_control kind=\"status\">";

/// The closing tag of both control turns.
pub const CONTROL_CLOSE_TAG: &str = "</via_control>";

/// The bound on a delegated result before it is shown to the coordinator.
///
/// **External contract** — `MAX_DELEGATION_RESULT_CHARS`,
/// `server/src/agent/acp-backend-adapter.mjs:34`, catalogued under *timeouts
/// and limits*.
pub const MAX_DELEGATION_RESULT_CHARS: usize = 12_000;

/// The `request_id=` prefix inside the protocol-retry block.
///
/// **External contract** — `coordinator.mjs:272`. It is a wire-shaped token
/// rather than prose, so it is a literal here: the model correlates the retry
/// with the envelope it already has by matching this exact string.
pub const RETRY_REQUEST_ID_PREFIX: &str = "request_id=";

/// The verified Layer-3 result being carried back into a coordinator turn.
///
/// **External contract** — the JSON body of the delegation-result block,
/// `acp-backend-adapter.mjs:1141-1148`. The key order is catalogued and is
/// reproduced by insertion order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DelegationResult {
    /// The delegation id VIA issued, or the backend's own `runId` on the
    /// native path.
    pub delegation_id: String,
    /// The Layer-3 Session that produced it.
    pub target_session_id: String,
    /// That Session's project directory.
    pub directory: String,
    /// What it answered.
    pub content: String,
}

impl DelegationResult {
    /// The JSON body, with the catalogued key order and the catalogued clip.
    #[must_use]
    pub fn to_json(&self, coordination_run_id: &str) -> Value {
        let mut map = Map::new();
        map.insert(
            "request_id".to_owned(),
            Value::String(clean(coordination_run_id).to_owned()),
        );
        map.insert(
            "delegation_id".to_owned(),
            Value::String(self.delegation_id.clone()),
        );
        map.insert(
            "target_session_id".to_owned(),
            Value::String(self.target_session_id.clone()),
        );
        map.insert(
            "directory".to_owned(),
            Value::String(self.directory.clone()),
        );
        map.insert(
            "result".to_owned(),
            Value::String(slice_units(
                clean(&self.content),
                MAX_DELEGATION_RESULT_CHARS,
            )),
        );
        Value::Object(map)
    }
}

/// The retry sent when a reply's `state` is not deliverable.
///
/// **External contract** — `docs/reference/contracts.json` (`prompt-text` /
/// *coordinator protocol retry block*), `coordinator.mjs:270-276`. Sent **at
/// most twice**; see [`crate::Coordinator`] for the ladder.
///
/// `state` is interpolated verbatim: the model is told which value it returned,
/// which is the only way it can tell a rejected `active` from a rejected
/// `delegated`.
#[must_use]
pub fn protocol_retry_prompt(coordination_run_id: &str, state: &str, locale: Locale) -> String {
    [
        PROTOCOL_RETRY_OPEN_TAG.to_owned(),
        format!("{RETRY_REQUEST_ID_PREFIX}{}", clean(coordination_run_id)),
        format(
            locale,
            keys::COORDINATOR_PROTOCOL_RETRY_UNSUPPORTED_STATE,
            &[("state", state)],
        ),
        t(locale, keys::COORDINATOR_PROTOCOL_RETRY_INSTRUCTIONS).to_owned(),
        PROTOCOL_RETRY_CLOSE_TAG.to_owned(),
    ]
    .join("\n")
}

/// The block that carries a finished Layer-3 result back to the coordinator.
///
/// **External contract** — `docs/reference/contracts.json` (`prompt-text` /
/// *delegated result injection block*), `acp-backend-adapter.mjs:1139-1155`.
/// The JSON is **2-space indented** and its key order is
/// `request_id, delegation_id, target_session_id, directory, result`; the
/// result is `clean(content).slice(0, 12000)`.
///
/// The four sentences under it are one `via-i18n` key
/// ([`keys::ACP_DELEGATION_RESULT_INSTRUCTIONS`]) carrying all four lines,
/// because they are one instruction and translating them apart would let three
/// of the four drift.
#[must_use]
pub fn delegation_result_prompt(
    result: &DelegationResult,
    coordination_run_id: &str,
    locale: Locale,
) -> String {
    let body = result.to_json(coordination_run_id);
    let rendered = serde_json::to_string_pretty(&body).unwrap_or_else(|_| body.to_string());
    [
        DELEGATION_RESULT_OPEN_TAG,
        &rendered,
        DELEGATION_RESULT_CLOSE_TAG,
        t(locale, keys::ACP_DELEGATION_RESULT_INSTRUCTIONS),
    ]
    .join("\n")
}

/// The block that tells the model what the Gateway already did without it.
///
/// **External contract** — `docs/reference/contracts.json` (`prompt-text` /
/// *reconciliation block*), `acp-backend-adapter.mjs:1007-1017`. One
/// **compact** JSON line per fact, then the instruction, then a blank line,
/// then the turn's own text.
///
/// The facts come from [`via_work::ReconciliationLedger`], which caps them at
/// twenty per owner and clears them only once a turn actually carried them.
/// Returns `content` unchanged when there is nothing to reconcile, so a caller
/// can apply it unconditionally.
#[must_use]
pub fn reconciliation_prompt(facts: &[CancellationFact], content: &str, locale: Locale) -> String {
    if facts.is_empty() {
        return content.to_owned();
    }
    let mut lines = vec![RECONCILIATION_OPEN_TAG.to_owned()];
    for fact in facts {
        lines.push(serde_json::to_string(fact).unwrap_or_else(|_| String::from("{}")));
    }
    lines.push(RECONCILIATION_CLOSE_TAG.to_owned());
    lines.push(t(locale, keys::ACP_RECONCILIATION_INSTRUCTIONS).to_owned());
    lines.push(String::new());
    lines.push(content.to_owned());
    lines.join("\n")
}

/// The hidden turn that asks the model to cancel a delegated Session.
///
/// **External contract** — `docs/reference/contracts.json` (`prompt-text` /
/// *cancel control turn*), `acp-backend-adapter.mjs:1388-1395`.
///
/// `instruction` is the profile's sentence. Passing `None` renders the default,
/// [`keys::ACP_CANCEL_INSTRUCTION_DEFAULT`], which names `via_session_cancel` —
/// one of [`via_mcp_tools::SESSION_TOOL_NAMES`]. A backend whose delegation is
/// native rather than MCP overrides it (`via-backends`), because the default
/// would name a tool it has never been given.
#[must_use]
pub fn cancel_control_prompt(
    delegation_id: &str,
    instruction: Option<&str>,
    locale: Locale,
) -> String {
    let sentence = instruction.map_or_else(
        || {
            format(
                locale,
                keys::ACP_CANCEL_INSTRUCTION_DEFAULT,
                &[("delegation_id", delegation_id)],
            )
        },
        str::to_owned,
    );
    [
        CONTROL_CANCEL_OPEN_TAG,
        &sentence,
        t(locale, keys::ACP_CANCEL_CONTROL_TAIL),
        CONTROL_CLOSE_TAG,
    ]
    .join("\n")
}

/// The hidden turn that asks the model how a delegated Session is doing.
///
/// **External contract** — `docs/reference/contracts.json` (`prompt-text` /
/// *status control turn*), `acp-backend-adapter.mjs:1433-1443`.
///
/// The user's own question is interpolated when there is one and replaced by
/// [`keys::ACP_STATUS_CONTROL_NO_QUESTION`] when there is not. The tail is the
/// load-bearing line: *answer from the tool result only, do not scan the
/// project* — which is the same rule
/// [`via_mcp_tools::SessionTool::SessionStatus`]'s description states, said
/// where the model is about to act on it.
#[must_use]
pub fn status_control_prompt(
    delegation_id: &str,
    question: &str,
    instruction: Option<&str>,
    locale: Locale,
) -> String {
    let sentence = instruction.map_or_else(
        || {
            format(
                locale,
                keys::ACP_STATUS_INSTRUCTION_DEFAULT,
                &[("delegation_id", delegation_id)],
            )
        },
        str::to_owned,
    );
    let question = clean(question);
    let asked = if question.is_empty() {
        t(locale, keys::ACP_STATUS_CONTROL_NO_QUESTION).to_owned()
    } else {
        format(
            locale,
            keys::ACP_STATUS_CONTROL_QUESTION,
            &[("question", question)],
        )
    };
    [
        CONTROL_STATUS_OPEN_TAG,
        &sentence,
        &asked,
        t(locale, keys::ACP_STATUS_CONTROL_TAIL),
        CONTROL_CLOSE_TAG,
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn fact(index: usize) -> CancellationFact {
        CancellationFact::delegated_session_cancelled(
            &format!("work_{index}"),
            &format!("run-{index}"),
            &format!("agent:child:{index}"),
            "2026-08-22T00:00:00.000Z",
        )
    }

    #[test]
    fn the_retry_block_names_the_request_and_the_refused_state() {
        let block = protocol_retry_prompt(" work-one ", "active", Locale::Zh);
        let lines: Vec<&str> = block.split('\n').collect();
        assert_eq!(lines[0], PROTOCOL_RETRY_OPEN_TAG);
        assert_eq!(lines[1], "request_id=work-one");
        assert!(lines[2].contains("state=active"));
        assert_eq!(
            lines[3],
            t(Locale::Zh, keys::COORDINATOR_PROTOCOL_RETRY_INSTRUCTIONS),
        );
        assert_eq!(lines[4], PROTOCOL_RETRY_CLOSE_TAG);
        assert_eq!(lines.len(), 5);
    }

    #[test]
    fn the_delegation_result_block_is_two_space_indented_in_key_order() {
        let result = DelegationResult {
            delegation_id: "opencode_run_1".to_owned(),
            target_session_id: "project-1".to_owned(),
            directory: "/project".to_owned(),
            content: "  built it  ".to_owned(),
        };
        let block = delegation_result_prompt(&result, "work-one", Locale::Zh);
        assert!(block.starts_with(&format!(
            "{DELEGATION_RESULT_OPEN_TAG}\n{{\n  \"request_id\""
        )));
        let body = block
            .split_once('\n')
            .and_then(|(_, rest)| rest.split_once(&format!("\n{DELEGATION_RESULT_CLOSE_TAG}")))
            .map(|(body, _)| body.to_owned())
            .expect("a JSON body");
        let parsed: Value = serde_json::from_str(&body).expect("valid JSON");
        assert_eq!(
            parsed
                .as_object()
                .expect("object")
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            [
                "request_id",
                "delegation_id",
                "target_session_id",
                "directory",
                "result"
            ],
        );
        assert_eq!(parsed["result"], "built it");
        assert!(block.ends_with(t(Locale::Zh, keys::ACP_DELEGATION_RESULT_INSTRUCTIONS)));
    }

    #[test]
    fn a_long_delegated_result_is_clipped_to_twelve_thousand() {
        let result = DelegationResult {
            content: "x".repeat(MAX_DELEGATION_RESULT_CHARS + 500),
            ..DelegationResult::default()
        };
        let body = result.to_json("work-one");
        assert_eq!(
            body["result"].as_str().map(str::len),
            Some(MAX_DELEGATION_RESULT_CHARS),
        );
    }

    #[test]
    fn the_reconciliation_block_is_one_compact_line_per_fact() {
        let facts = [fact(1), fact(2)];
        let block = reconciliation_prompt(&facts, "the next request", Locale::Zh);
        let lines: Vec<&str> = block.split('\n').collect();
        assert_eq!(lines[0], RECONCILIATION_OPEN_TAG);
        assert!(
            lines[1].starts_with(r#"{"kind":"delegated_session_cancelled","work_id":"work_1""#)
        );
        assert!(lines[2].contains(r#""work_id":"work_2""#));
        assert_eq!(lines[3], RECONCILIATION_CLOSE_TAG);
        assert_eq!(
            lines[4],
            t(Locale::Zh, keys::ACP_RECONCILIATION_INSTRUCTIONS)
        );
        assert_eq!(lines[5], "");
        assert_eq!(lines[6], "the next request");
        assert_eq!(lines.len(), 7);
    }

    #[test]
    fn nothing_to_reconcile_leaves_the_prompt_untouched() {
        assert_eq!(
            reconciliation_prompt(&[], "the next request", Locale::Zh),
            "the next request",
        );
    }

    #[test]
    fn the_cancel_control_turn_defaults_to_the_session_tool_sentence() {
        let block = cancel_control_prompt("opencode_run_1", None, Locale::Zh);
        let lines: Vec<&str> = block.split('\n').collect();
        assert_eq!(lines[0], CONTROL_CANCEL_OPEN_TAG);
        assert!(lines[1].contains("via_session_cancel"));
        assert!(lines[1].contains("delegation_id=opencode_run_1"));
        assert_eq!(lines[2], t(Locale::Zh, keys::ACP_CANCEL_CONTROL_TAIL));
        assert_eq!(lines[3], CONTROL_CLOSE_TAG);

        let overridden = cancel_control_prompt("ignored", Some("stop it natively"), Locale::Zh);
        assert!(overridden.contains("stop it natively"));
        assert!(!overridden.contains("via_session_cancel"));
    }

    #[test]
    fn the_default_cancel_sentence_names_a_real_session_tool() {
        let block = cancel_control_prompt("d-1", None, Locale::En);
        assert!(
            via_mcp_tools::SESSION_TOOL_NAMES
                .iter()
                .any(|name| block.contains(name)),
            "the sentence must name a tool the model was actually given: {block}",
        );
    }

    #[test]
    fn the_status_control_turn_carries_the_question_or_says_there_is_none() {
        let asked = status_control_prompt("d-1", "  做到哪了  ", None, Locale::Zh);
        assert!(asked.contains("用户的具体问题：做到哪了"));
        assert!(asked.ends_with(&format!(
            "{}\n{CONTROL_CLOSE_TAG}",
            t(Locale::Zh, keys::ACP_STATUS_CONTROL_TAIL),
        )));

        let unasked = status_control_prompt("d-1", "   ", None, Locale::Zh);
        assert!(unasked.contains(t(Locale::Zh, keys::ACP_STATUS_CONTROL_NO_QUESTION)));
        assert!(!unasked.contains("用户的具体问题"));
    }

    #[test]
    fn every_block_renders_in_all_three_locales() {
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            for rendered in [
                protocol_retry_prompt("w", "active", locale),
                delegation_result_prompt(&DelegationResult::default(), "w", locale),
                reconciliation_prompt(&[fact(1)], "next", locale),
                cancel_control_prompt("d", None, locale),
                status_control_prompt("d", "q", None, locale),
            ] {
                assert!(
                    !rendered.contains("<via-i18n:"),
                    "an unrenderable template reached the model: {rendered}",
                );
            }
        }
    }
}
