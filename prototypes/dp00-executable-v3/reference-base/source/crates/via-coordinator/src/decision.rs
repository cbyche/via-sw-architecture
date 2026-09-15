//! The two response shapes, and how a reply is read back.
//!
//! `server/src/agent/coordinator.mjs:7-118`. Three things live here:
//!
//! * [`coordinator_decision_schema`] — the JSON Schema handed to the backend as
//!   `outputSchema`, which a structured-output model may enforce against;
//! * [`DecisionState`] / [`DecisionMode`] — the two catalogued shapes,
//!   `completed`/`respond` and `delegated`/`delegate`;
//! * [`parse_coordinator_decision`] — what the Gateway actually keeps.
//!
//! # The delegated shape is never a completion
//!
//! `docs/reference/contracts.json` (`json-field` / *coordinator 'delegated'
//! decision*) is explicit: *"It is NEVER a user-visible completion; the adapter
//! treats it as a lock-release signal."* That is why
//! [`parse_coordinator_decision`] hard-codes `completed`/`respond` rather than
//! reading them: by the time a decision is being parsed for delivery the turn
//! ladder in [`crate::Coordinator`] has already refused anything else, and a
//! `state` field that could still say `delegated` at that point would be a
//! second place for the refusal to be forgotten.
//!
//! # Reading the reply is not this module's job
//!
//! The unwrapping algorithm — three iterations, a `json` fence, a
//! double-encoded string, the first-`{`-to-last-`}` narrowing and its
//! no-progress guard — is
//! [`via_acp::parse_coordinator_payload`](via_acp::session::parse_coordinator_payload),
//! and the legacy string-`inline` upgrade is
//! [`via_acp::normalize_coordinator_content`](via_acp::session::normalize_coordinator_content).
//! Both are catalogued algorithms that `via-acp` already ships; a second copy
//! here would be a second answer to *"which malformed model output is
//! accepted"*.

use serde_json::{Value, json};
use via_acp::session::parse_coordinator_payload;
use via_downstream::text::clean;
use via_work::text::slice_units;
use via_work::{InlineBlock, InlineFormat, Presentation};

/// The bound on a decision's `presentation.inline.title`.
///
/// **External contract** — `coordinator.mjs:91` (`.slice(0, 120)`). The same
/// number [`via_work::presentation::INLINE_TITLE_BOUND`] carries, applied a
/// second time downstream; the two are separate slices in upstream and stay
/// separate here.
pub const INLINE_TITLE_BOUND: usize = 120;

/// The `state` a decision reports.
///
/// **External contract** — the `enum` in each branch of
/// [`coordinator_decision_schema`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecisionState {
    /// The work is done and the presentation is the answer.
    Completed,
    /// The work has been handed to a Layer-3 Session. Not a completion.
    Delegated,
}

impl DecisionState {
    /// The wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Delegated => "delegated",
        }
    }

    /// Parse a wire spelling, or `None` for anything else.
    ///
    /// Upstream lower-cases before comparing
    /// (`coordinatorResponseState`, `coordinator.mjs:80-82`), so this does too.
    #[must_use]
    pub fn from_wire(value: &str) -> Option<Self> {
        match clean(value).to_lowercase().as_str() {
            "completed" => Some(Self::Completed),
            "delegated" => Some(Self::Delegated),
            _ => None,
        }
    }

    /// The `mode` that always accompanies this state.
    #[must_use]
    pub const fn mode(self) -> DecisionMode {
        match self {
            Self::Completed => DecisionMode::Respond,
            Self::Delegated => DecisionMode::Delegate,
        }
    }
}

/// The `mode` a decision reports.
///
/// Paired with [`DecisionState`] one-to-one, which is why
/// [`DecisionState::mode`] exists and there is no way to build a
/// `completed`/`delegate` decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecisionMode {
    /// Speak the presentation.
    Respond,
    /// A Layer-3 Session is running; wait for it.
    Delegate,
}

impl DecisionMode {
    /// The wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Respond => "respond",
            Self::Delegate => "delegate",
        }
    }
}

/// The `format` values an inline block may declare.
///
/// **External contract** — `coordinator.mjs:14,92`. Spelled through
/// [`InlineFormat`] rather than restated, so the schema and the parser cannot
/// disagree about which three are legal.
#[must_use]
pub fn inline_formats() -> [&'static str; 3] {
    [
        InlineFormat::Markdown.as_str(),
        InlineFormat::Code.as_str(),
        InlineFormat::Link.as_str(),
    ]
}

/// The JSON Schema the backend is given as `outputSchema`.
///
/// **External contract** — `docs/reference/contracts.json` (`json-field` /
/// *COORDINATOR_DECISION_SCHEMA*), `server/src/agent/coordinator.mjs:7-70`.
/// *"Passed verbatim to the ACP prompt as `outputSchema`; the backend model may
/// enforce structured output against it. Must be byte-reproducible."*
///
/// Both branches set `additionalProperties: false`, and the delegated branch
/// requires all six keys — a model that answered `state: "delegated"` without
/// naming the session it delegated to would leave the Gateway with nothing to
/// correlate a completion against.
#[must_use]
pub fn coordinator_decision_schema() -> Value {
    let [markdown, code, link] = inline_formats();
    let inline = json!({
        "anyOf": [
            { "type": "null" },
            {
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "format": { "type": "string", "enum": [markdown, code, link] },
                    "content": { "type": "string" },
                },
                "required": ["title", "format", "content"],
                "additionalProperties": false,
            },
        ],
    });
    let presentation = json!({
        "type": "object",
        "properties": {
            "speech": { "type": "string" },
            "inline": inline,
        },
        "required": ["speech", "inline"],
        "additionalProperties": false,
    });
    json!({
        "type": "object",
        "oneOf": [
            {
                "type": "object",
                "properties": {
                    "work_id": { "type": "string" },
                    "state": { "type": "string", "enum": [DecisionState::Completed.as_str()] },
                    "mode": { "type": "string", "enum": [DecisionMode::Respond.as_str()] },
                    "presentation": presentation,
                },
                "required": ["work_id", "state", "mode", "presentation"],
                "additionalProperties": false,
            },
            {
                "type": "object",
                "properties": {
                    "work_id": { "type": "string" },
                    "state": { "type": "string", "enum": [DecisionState::Delegated.as_str()] },
                    "mode": { "type": "string", "enum": [DecisionMode::Delegate.as_str()] },
                    "delegation_id": { "type": "string" },
                    "target_session_id": { "type": "string" },
                    "presentation": presentation,
                },
                "required": [
                    "work_id",
                    "state",
                    "mode",
                    "delegation_id",
                    "target_session_id",
                    "presentation",
                ],
                "additionalProperties": false,
            },
        ],
    })
}

/// The `state` a reply reports, lower-cased, or `""` when it reports none.
///
/// **External contract** — `coordinatorResponseState`,
/// `server/src/agent/coordinator.mjs:80-82`. It is a `String` rather than an
/// `Option<DecisionState>` because the turn ladder branches on *three* cases,
/// not two: an empty state (a reply with no `state` at all, which is accepted),
/// `completed` (accepted), and **anything else** — which is refused, and whose
/// literal text is interpolated into the retry block the model is shown. A
/// typed parse would collapse the third case into the first.
#[must_use]
pub fn coordinator_response_state(content: &str) -> String {
    parse_coordinator_payload(content)
        .and_then(|payload| {
            payload
                .get("state")
                .map(|state| clean_value(state).to_lowercase())
        })
        .unwrap_or_default()
}

/// `String(value || '').trim()` over a JSON value.
///
/// The same coercion [`via_acp::session::clean_value`] performs; called through
/// rather than restated.
fn clean_value(value: &Value) -> String {
    via_acp::session::clean_value(Some(value))
}

/// Whether a reply may be delivered as the turn's final answer.
///
/// **External contract** — `coordinator.mjs:268-280`: a reply is deliverable
/// when its state is empty **or** `completed`. Anything else — `active`,
/// `delegated`, a progress note, an invented value — triggers the retry block
/// and, if it survives that, the refusal.
#[must_use]
pub fn is_deliverable_state(state: &str) -> bool {
    state.is_empty() || state == DecisionState::Completed.as_str()
}

/// A parsed coordinator decision, as the Gateway keeps it.
///
/// **External contract** — `parseCoordinatorDecision`,
/// `server/src/agent/coordinator.mjs:105-118`. Upstream's return object also
/// carries `task: null` and `targetSession: null`, two fields that are
/// unconditionally null at every call site and are read by nothing; they are
/// not reproduced, and `docs/deviations/phase-3.md` records that.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoordinatorDecision {
    /// The expected Work id, or — when the caller named none — the `work_id`
    /// the model echoed back.
    pub work_id: String,
    /// What the user is told and shown.
    pub presentation: Presentation,
}

impl CoordinatorDecision {
    /// The state a parsed decision always reports.
    ///
    /// See the module docs: by the time a decision is parsed for delivery the
    /// ladder has already refused every other state.
    #[must_use]
    pub const fn state(&self) -> DecisionState {
        DecisionState::Completed
    }

    /// The mode a parsed decision always reports.
    #[must_use]
    pub const fn mode(&self) -> DecisionMode {
        DecisionMode::Respond
    }
}

/// Normalize a model-supplied `inline` block.
///
/// **External contract** — `normalizeInline`, `coordinator.mjs:84-95`:
///
/// * anything that is not an object becomes `None` — including the legacy
///   **string** form, which [`via_acp::normalize_coordinator_content`] has
///   already had its chance to upgrade;
/// * a blank `content` becomes `None`, so an empty block is never shown;
/// * `title` is cleaned **and** sliced to [`INLINE_TITLE_BOUND`];
/// * an unrecognised `format` becomes `markdown` rather than being rejected.
///
/// Note the asymmetry with [`via_work::presentation::project`], which slices
/// the title but does **not** clean it and keeps `content` verbatim. The two
/// are different upstream functions applied one after the other, and collapsing
/// them would change what a task record holds.
#[must_use]
pub fn normalize_inline(value: Option<&Value>) -> Option<InlineBlock> {
    let object = value?.as_object()?;
    let content = clean_value(object.get("content").unwrap_or(&Value::Null));
    if content.is_empty() {
        return None;
    }
    Some(InlineBlock {
        title: slice_units(
            &clean_value(object.get("title").unwrap_or(&Value::Null)),
            INLINE_TITLE_BOUND,
        ),
        format: InlineFormat::from_wire_or_default(object.get("format").and_then(Value::as_str)),
        content,
    })
}

/// Normalize a model-supplied `presentation`, falling back to `fallback` for
/// the speech.
///
/// **External contract** — `normalizePresentation`, `coordinator.mjs:97-103`.
/// The fallback is reached whenever the model's own `speech` cleans to nothing,
/// which is how a backend that answered in plain prose still gets spoken.
#[must_use]
pub fn normalize_presentation(value: Option<&Value>, fallback: &str) -> Presentation {
    let object = value.and_then(Value::as_object);
    let speech = object.map_or_else(String::new, |presentation| {
        clean_value(presentation.get("speech").unwrap_or(&Value::Null))
    });
    Presentation {
        speech: if speech.is_empty() {
            clean(fallback).to_owned()
        } else {
            speech
        },
        inline: normalize_inline(object.and_then(|presentation| presentation.get("inline"))),
    }
}

/// Read a backend reply into the decision the Gateway keeps.
///
/// **External contract** — `parseCoordinatorDecision`, `coordinator.mjs:105-118`.
///
/// * `work_id` is the *expected* id when the caller supplied one, and only
///   otherwise the one the model echoed — a model that mislabels its answer
///   cannot re-address it to another Work.
/// * the speech falls back twice: to the payload's own legacy `response` field,
///   and then to the **whole raw reply**, so a backend that ignored the
///   response contract entirely still says something.
///
/// It is total: an unparseable reply yields a decision whose speech is the
/// cleaned raw text.
#[must_use]
pub fn parse_coordinator_decision(content: &str, expected_work_id: &str) -> CoordinatorDecision {
    let payload = parse_coordinator_payload(content);
    let expected = clean(expected_work_id);
    let work_id = if expected.is_empty() {
        payload
            .as_ref()
            .and_then(|payload| payload.get("work_id"))
            .map(clean_value)
            .unwrap_or_default()
    } else {
        expected.to_owned()
    };
    let response = payload
        .as_ref()
        .and_then(|payload| payload.get("response"))
        .map(clean_value)
        .unwrap_or_default();
    let fallback = if response.is_empty() {
        clean(content).to_owned()
    } else {
        response
    };
    CoordinatorDecision {
        work_id,
        presentation: normalize_presentation(
            payload
                .as_ref()
                .and_then(|payload| payload.get("presentation")),
            &fallback,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn the_schema_declares_both_shapes_and_closes_both() {
        let schema = coordinator_decision_schema();
        assert_eq!(schema["type"], "object");
        let branches = schema["oneOf"].as_array().expect("two branches");
        assert_eq!(branches.len(), 2);

        assert_eq!(
            branches[0]["properties"]["state"]["enum"],
            json!(["completed"])
        );
        assert_eq!(
            branches[0]["properties"]["mode"]["enum"],
            json!(["respond"])
        );
        assert_eq!(
            branches[0]["required"],
            json!(["work_id", "state", "mode", "presentation"]),
        );
        assert_eq!(branches[0]["additionalProperties"], json!(false));

        assert_eq!(
            branches[1]["properties"]["state"]["enum"],
            json!(["delegated"])
        );
        assert_eq!(
            branches[1]["properties"]["mode"]["enum"],
            json!(["delegate"])
        );
        assert_eq!(
            branches[1]["required"],
            json!([
                "work_id",
                "state",
                "mode",
                "delegation_id",
                "target_session_id",
                "presentation"
            ]),
        );
        assert_eq!(branches[1]["additionalProperties"], json!(false));
    }

    #[test]
    fn the_presentation_schema_requires_both_halves_and_allows_a_null_inline() {
        let schema = coordinator_decision_schema();
        let presentation = &schema["oneOf"][0]["properties"]["presentation"];
        assert_eq!(presentation["required"], json!(["speech", "inline"]));
        assert_eq!(presentation["additionalProperties"], json!(false));
        let inline = &presentation["properties"]["inline"];
        assert_eq!(inline["anyOf"][0], json!({ "type": "null" }));
        assert_eq!(
            inline["anyOf"][1]["properties"]["format"]["enum"],
            json!(["markdown", "code", "link"]),
        );
        assert_eq!(
            inline["anyOf"][1]["required"],
            json!(["title", "format", "content"]),
        );
        assert_eq!(
            schema["oneOf"][1]["properties"]["presentation"], *presentation,
            "both branches share one presentation schema",
        );
    }

    #[test]
    fn the_schemas_property_order_is_the_declaration_order() {
        let schema = coordinator_decision_schema();
        assert_eq!(
            schema["oneOf"][1]["properties"]
                .as_object()
                .expect("object")
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            [
                "work_id",
                "state",
                "mode",
                "delegation_id",
                "target_session_id",
                "presentation"
            ],
        );
    }

    #[test]
    fn a_reply_state_is_lower_cased_and_absent_reads_as_empty() {
        assert_eq!(
            coordinator_response_state(r#"{"state":" DELEGATED "}"#),
            "delegated"
        );
        assert_eq!(
            coordinator_response_state(r#"{"state":"Completed"}"#),
            "completed"
        );
        assert_eq!(coordinator_response_state(r#"{"work_id":"w"}"#), "");
        assert_eq!(coordinator_response_state("not json at all"), "");
        assert_eq!(coordinator_response_state(r#"{"state":null}"#), "");
    }

    #[test]
    fn only_an_empty_or_completed_state_is_deliverable() {
        assert!(is_deliverable_state(""));
        assert!(is_deliverable_state("completed"));
        for refused in ["delegated", "active", "in_progress", "respond"] {
            assert!(!is_deliverable_state(refused), "{refused}");
        }
    }

    #[test]
    fn the_expected_work_id_wins_over_the_models_own() {
        let decision = parse_coordinator_decision(
            r#"{"work_id":"work-other","presentation":{"speech":"done","inline":null}}"#,
            " work-one ",
        );
        assert_eq!(decision.work_id, "work-one");
        assert_eq!(decision.state(), DecisionState::Completed);
        assert_eq!(decision.mode(), DecisionMode::Respond);

        let echoed = parse_coordinator_decision(
            r#"{"work_id":" work-other ","presentation":{"speech":"done","inline":null}}"#,
            "   ",
        );
        assert_eq!(echoed.work_id, "work-other");
    }

    #[test]
    fn a_double_encoded_reply_is_unwrapped_before_the_speech_is_chosen() {
        let inner = json!({
            "work_id": "work-one",
            "state": "completed",
            "mode": "respond",
            "presentation": { "speech": "小画板项目已经做好了。", "inline": null },
        })
        .to_string();
        let encoded = Value::String(inner).to_string();
        let decision = parse_coordinator_decision(&encoded, "work-one");
        assert_eq!(decision.presentation.speech, "小画板项目已经做好了。");
    }

    #[test]
    fn the_speech_falls_back_to_response_then_to_the_whole_reply() {
        let from_response =
            parse_coordinator_decision(r#"{"response":"  plain answer  "}"#, "work-one");
        assert_eq!(from_response.presentation.speech, "plain answer");

        let from_raw = parse_coordinator_decision("  just talking  ", "work-one");
        assert_eq!(from_raw.presentation.speech, "just talking");
        assert_eq!(from_raw.presentation.inline, None);

        let empty_speech = parse_coordinator_decision(
            r#"{"response":"fallback","presentation":{"speech":"   ","inline":null}}"#,
            "work-one",
        );
        assert_eq!(empty_speech.presentation.speech, "fallback");
    }

    #[test]
    fn an_inline_block_is_cleaned_bounded_and_defaulted() {
        let long_title = "题".repeat(INLINE_TITLE_BOUND + 30);
        let decision = parse_coordinator_decision(
            &json!({
                "presentation": {
                    "speech": "done",
                    "inline": {
                        "title": format!("  {long_title}  "),
                        "format": "html",
                        "content": "  ## 完成  ",
                    },
                },
            })
            .to_string(),
            "work-one",
        );
        let inline = decision.presentation.inline.expect("an inline block");
        assert_eq!(inline.title.chars().count(), INLINE_TITLE_BOUND);
        assert_eq!(inline.format, InlineFormat::Markdown);
        assert_eq!(inline.content, "## 完成");
    }

    #[test]
    fn a_blank_or_non_object_inline_is_dropped() {
        for inline in [
            json!(null),
            json!("a legacy string"),
            json!(42),
            json!([{"content": "x"}]),
            json!({ "content": "   " }),
            json!({ "title": "t", "format": "code" }),
        ] {
            let decision = parse_coordinator_decision(
                &json!({ "presentation": { "speech": "s", "inline": inline } }).to_string(),
                "work-one",
            );
            assert_eq!(decision.presentation.inline, None, "{inline}");
        }
    }

    #[test]
    fn a_missing_title_becomes_empty_rather_than_absent() {
        let decision = parse_coordinator_decision(
            &json!({
                "presentation": { "speech": "s", "inline": { "content": "body", "format": "link" } },
            })
            .to_string(),
            "work-one",
        );
        let inline = decision.presentation.inline.expect("an inline block");
        assert_eq!(inline.title, "");
        assert_eq!(inline.format, InlineFormat::Link);
    }

    #[test]
    fn every_state_pairs_with_exactly_one_mode() {
        assert_eq!(DecisionState::Completed.mode(), DecisionMode::Respond);
        assert_eq!(DecisionState::Delegated.mode(), DecisionMode::Delegate);
        assert_eq!(
            DecisionState::from_wire("  COMPLETED "),
            Some(DecisionState::Completed)
        );
        assert_eq!(DecisionState::from_wire("respond"), None);
        assert_eq!(DecisionState::from_wire(""), None);
    }
}
