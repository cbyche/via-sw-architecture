//! The nine frontend tools.
//!
//! Ported from `server/src/voice/frontend-tools.mjs`.
//!
//! These are model-visible and therefore **byte-exact**: the name, the
//! description and the whole JSON Schema are catalogued contracts. The
//! descriptions come from [`via_i18n`], whose `zh` column is upstream's own
//! text reproduced verbatim; the schema *shape* is reproduced here.
//!
//! # Two quirks that are contract, not oversight
//!
//! **`cancel_agent_task.parameters` has no `required` key at all** — not
//! `required: []`, absent. `spawn_thinking` and `schedule_reminder` list their
//! required fields, `memory` and `notes` require only `action`, and
//! `respond_agent_permission` requires both of its fields; `cancel_agent_task`,
//! `get_agent_task_status`, `get_current_time` and `enter_sleep` have none.
//! Emitting `required: []` would be a different schema, and a strict validator
//! on the provider side treats the two differently.
//!
//! **`enter_sleep` is conditional.** It is appended to the tool list only when
//! the client declared the `sleeping` state, because a tool the entry point
//! cannot honour is a tool the model will promise and fail to use.
//!
//! # Where the implementations live
//!
//! `memory` and `notes` are implemented in [`via_conversation`]; this module
//! owns their schemas and descriptions, and
//! [`crate::tools::handler::ToolCallHandler`] calls into that crate.

use serde_json::{Map, Value, json};
use via_core::memory_scopes::{ALL_SCOPE, memory_documents};
use via_i18n::{Locale, keys, t};

/// `spawn_thinking` — delegate real work.
///
/// **External contract** — `frontend-tools.mjs:8`.
pub const SPAWN_THINKING: &str = "spawn_thinking";
/// `schedule_reminder` — a reminder or a timed task.
///
/// **External contract** — `frontend-tools.mjs:9`.
pub const SCHEDULE_REMINDER: &str = "schedule_reminder";
/// `cancel_agent_task` — cancel work, a scheduled task or a reminder.
///
/// **External contract** — `frontend-tools.mjs:10`.
pub const CANCEL_AGENT_TASK: &str = "cancel_agent_task";
/// `get_agent_task_status` — query or list work.
///
/// **External contract** — `frontend-tools.mjs:11`.
pub const GET_AGENT_TASK_STATUS: &str = "get_agent_task_status";
/// `get_current_time` — the clock in the user's zone.
///
/// **External contract** — `frontend-tools.mjs:12`.
pub const GET_CURRENT_TIME: &str = "get_current_time";
/// `memory` — the two long-term documents.
///
/// **External contract** — `frontend-tools.mjs:13`.
pub const MEMORY: &str = "memory";
/// `notes` — the named lists.
///
/// **External contract** — `frontend-tools.mjs:14`.
pub const NOTES: &str = "notes";
/// `respond_agent_permission` — relay the user's permission decision.
///
/// **External contract** — `frontend-tools.mjs:15`.
pub const RESPOND_AGENT_PERMISSION: &str = "respond_agent_permission";
/// `enter_sleep` — drop this entry point to wake-word-only.
///
/// **External contract** — `frontend-tools.mjs:16`.
pub const ENTER_SLEEP: &str = "enter_sleep";

/// The eight always-declared tools, in declaration order.
///
/// **External contract** — `frontend-tools.mjs:228-237` (`TOOLS`). The order
/// reaches the model in `session.update`, and a model's tool choice is
/// order-sensitive in practice, so it is reproduced rather than sorted.
pub const ALWAYS_DECLARED: [&str; 8] = [
    SPAWN_THINKING,
    SCHEDULE_REMINDER,
    CANCEL_AGENT_TASK,
    GET_AGENT_TASK_STATUS,
    GET_CURRENT_TIME,
    MEMORY,
    NOTES,
    RESPOND_AGENT_PERMISSION,
];

/// Every tool name this crate knows.
pub const ALL_TOOL_NAMES: [&str; 9] = [
    SPAWN_THINKING,
    SCHEDULE_REMINDER,
    CANCEL_AGENT_TASK,
    GET_AGENT_TASK_STATUS,
    GET_CURRENT_TIME,
    MEMORY,
    NOTES,
    RESPOND_AGENT_PERMISSION,
    ENTER_SLEEP,
];

/// The client-declared state that unlocks [`ENTER_SLEEP`].
///
/// **External contract** — `frontend-tools.mjs:243`.
pub const SLEEPING_CLIENT_STATE: &str = "sleeping";

/// The cap on `spawn_thinking.input_refs`.
///
/// **External contract** — `frontend-tools.mjs:33`.
pub const MAX_INPUT_REFS: usize = 8;

/// The cap on `notes.items`.
///
/// **External contract** — `frontend-tools.mjs:148`, and enforced again by the
/// handler at `tool-call-handler.mjs:1072`.
pub const MAX_NOTES_ITEMS: usize = 20;

/// The three `memory.action` values.
///
/// **External contract** — `frontend-tools.mjs:110`.
pub const MEMORY_ACTIONS: [&str; 3] = ["read", "append", "replace"];

/// The six `notes.action` values.
///
/// **External contract** — `frontend-tools.mjs:138`.
pub const NOTES_ACTIONS: [&str; 6] = ["lists", "show", "add", "remove", "clear", "drop"];

/// The two `respond_agent_permission.decision` values.
///
/// **External contract** — `frontend-tools.mjs:172`.
pub const PERMISSION_DECISIONS: [&str; 2] = ["always", "reject"];

/// The two `schedule_reminder.type` values.
///
/// **External contract** — `frontend-tools.mjs:213`.
pub const REMINDER_TYPES: [&str; 2] = ["reminder", "task"];

/// The four `schedule_reminder.recurrence` values.
///
/// **External contract** — `frontend-tools.mjs:218`.
pub const REMINDER_RECURRENCES: [&str; 4] = ["once", "daily", "weekly", "weekdays"];

fn object_schema(properties: Value, required: Option<&[&str]>) -> Value {
    let mut schema = Map::new();
    schema.insert("type".to_owned(), json!("object"));
    schema.insert("properties".to_owned(), properties);
    // `required` is inserted before `additionalProperties` when present, and
    // omitted entirely when absent — which is what `cancel_agent_task` needs.
    if let Some(required) = required {
        schema.insert("required".to_owned(), json!(required));
    }
    schema.insert("additionalProperties".to_owned(), json!(false));
    Value::Object(schema)
}

fn function_tool(name: &str, description: &str, parameters: Value) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": name,
            "description": description,
            "parameters": parameters,
        },
    })
}

/// The `spawn_thinking` declaration.
#[must_use]
pub fn spawn_thinking_tool(locale: Locale) -> Value {
    function_tool(
        SPAWN_THINKING,
        t(locale, keys::VOICE_TOOL_SPAWN_THINKING_DESCRIPTION),
        object_schema(
            json!({
                "objective": {
                    "type": "string",
                    "description": t(locale, keys::VOICE_TOOL_SPAWN_THINKING_OBJECTIVE),
                },
                "input_refs": {
                    "type": "array",
                    "items": { "type": "string" },
                    "maxItems": MAX_INPUT_REFS,
                    "description": t(locale, keys::VOICE_TOOL_SPAWN_THINKING_INPUT_REFS),
                },
            }),
            Some(&["objective"]),
        ),
    )
}

/// The `schedule_reminder` declaration.
#[must_use]
pub fn schedule_reminder_tool(locale: Locale) -> Value {
    function_tool(
        SCHEDULE_REMINDER,
        t(locale, keys::VOICE_TOOL_SCHEDULE_REMINDER_DESCRIPTION),
        object_schema(
            json!({
                "execute_at": {
                    "type": "string",
                    "description": t(locale, keys::VOICE_TOOL_SCHEDULE_REMINDER_EXECUTE_AT),
                },
                "reminder": {
                    "type": "string",
                    "description": t(locale, keys::VOICE_TOOL_SCHEDULE_REMINDER_REMINDER),
                },
                "type": {
                    "type": "string",
                    "enum": REMINDER_TYPES,
                    "description": t(locale, keys::VOICE_TOOL_SCHEDULE_REMINDER_TYPE),
                },
                "recurrence": {
                    "type": "string",
                    "enum": REMINDER_RECURRENCES,
                    "description": t(locale, keys::VOICE_TOOL_SCHEDULE_REMINDER_RECURRENCE),
                },
            }),
            Some(&["execute_at", "reminder"]),
        ),
    )
}

/// The `cancel_agent_task` declaration.
///
/// Note the `None`: this schema has **no `required` key**, which is upstream's
/// shape at `frontend-tools.mjs:48-57`.
#[must_use]
pub fn cancel_agent_task_tool(locale: Locale) -> Value {
    function_tool(
        CANCEL_AGENT_TASK,
        t(locale, keys::VOICE_TOOL_CANCEL_AGENT_TASK_DESCRIPTION),
        object_schema(
            json!({
                "work_id": {
                    "type": "string",
                    "description": t(locale, keys::VOICE_TOOL_CANCEL_AGENT_TASK_WORK_ID),
                },
            }),
            None,
        ),
    )
}

/// The `get_agent_task_status` declaration.
#[must_use]
pub fn get_agent_task_status_tool(locale: Locale) -> Value {
    function_tool(
        GET_AGENT_TASK_STATUS,
        t(locale, keys::VOICE_TOOL_GET_AGENT_TASK_STATUS_DESCRIPTION),
        object_schema(
            json!({
                "work_id": {
                    "type": "string",
                    "description": t(locale, keys::VOICE_TOOL_GET_AGENT_TASK_STATUS_WORK_ID),
                },
                "question": {
                    "type": "string",
                    "description": t(locale, keys::VOICE_TOOL_GET_AGENT_TASK_STATUS_QUESTION),
                },
                "list_all": {
                    "type": "boolean",
                    "description": t(locale, keys::VOICE_TOOL_GET_AGENT_TASK_STATUS_LIST_ALL),
                },
            }),
            None,
        ),
    )
}

/// The `get_current_time` declaration.
#[must_use]
pub fn get_current_time_tool(locale: Locale) -> Value {
    function_tool(
        GET_CURRENT_TIME,
        t(locale, keys::VOICE_TOOL_GET_CURRENT_TIME_DESCRIPTION),
        object_schema(json!({}), None),
    )
}

/// The `memory` declaration.
///
/// The `document` enum is `[...MEMORY_DOCUMENTS, 'all']`, sourced from
/// [`via_core::memory_scopes`] rather than retyped, so the two cannot drift.
#[must_use]
pub fn memory_tool(locale: Locale) -> Value {
    let mut documents = memory_documents();
    documents.push(ALL_SCOPE);
    function_tool(
        MEMORY,
        t(locale, keys::VOICE_TOOL_MEMORY_DESCRIPTION),
        object_schema(
            json!({
                "action": {
                    "type": "string",
                    "enum": MEMORY_ACTIONS,
                    "description": t(locale, keys::VOICE_TOOL_MEMORY_ACTION),
                },
                "document": {
                    "type": "string",
                    "enum": documents,
                    "description": t(locale, keys::VOICE_TOOL_MEMORY_DOCUMENT),
                },
                "old_text": {
                    "type": "string",
                    "description": t(locale, keys::VOICE_TOOL_MEMORY_OLD_TEXT),
                },
                "new_text": {
                    "type": "string",
                    "description": t(locale, keys::VOICE_TOOL_MEMORY_NEW_TEXT),
                },
                "content": {
                    "type": "string",
                    "description": t(locale, keys::VOICE_TOOL_MEMORY_CONTENT),
                },
            }),
            Some(&["action"]),
        ),
    )
}

/// The `notes` declaration.
#[must_use]
pub fn notes_tool(locale: Locale) -> Value {
    function_tool(
        NOTES,
        t(locale, keys::VOICE_TOOL_NOTES_DESCRIPTION),
        object_schema(
            json!({
                "action": {
                    "type": "string",
                    "enum": NOTES_ACTIONS,
                    "description": t(locale, keys::VOICE_TOOL_NOTES_ACTION),
                },
                "list": {
                    "type": "string",
                    "description": t(locale, keys::VOICE_TOOL_NOTES_LIST),
                },
                "items": {
                    "type": "array",
                    "items": { "type": "string" },
                    "maxItems": MAX_NOTES_ITEMS,
                    "description": t(locale, keys::VOICE_TOOL_NOTES_ITEMS),
                },
            }),
            Some(&["action"]),
        ),
    )
}

/// The `respond_agent_permission` declaration.
#[must_use]
pub fn respond_agent_permission_tool(locale: Locale) -> Value {
    function_tool(
        RESPOND_AGENT_PERMISSION,
        t(
            locale,
            keys::VOICE_TOOL_RESPOND_AGENT_PERMISSION_DESCRIPTION,
        ),
        object_schema(
            json!({
                "authorization_id": {
                    "type": "string",
                    "description": t(
                        locale,
                        keys::VOICE_TOOL_RESPOND_AGENT_PERMISSION_AUTHORIZATION_ID,
                    ),
                },
                "decision": {
                    "type": "string",
                    "enum": PERMISSION_DECISIONS,
                    "description": t(locale, keys::VOICE_TOOL_RESPOND_AGENT_PERMISSION_DECISION),
                },
            }),
            Some(&["authorization_id", "decision"]),
        ),
    )
}

/// The `enter_sleep` declaration.
#[must_use]
pub fn enter_sleep_tool(locale: Locale) -> Value {
    function_tool(
        ENTER_SLEEP,
        t(locale, keys::VOICE_TOOL_ENTER_SLEEP_DESCRIPTION),
        object_schema(json!({}), None),
    )
}

/// The eight always-declared tools.
///
/// **External contract** — `frontend-tools.mjs:228-237`.
#[must_use]
pub fn tools(locale: Locale) -> Vec<Value> {
    vec![
        spawn_thinking_tool(locale),
        schedule_reminder_tool(locale),
        cancel_agent_task_tool(locale),
        get_agent_task_status_tool(locale),
        get_current_time_tool(locale),
        memory_tool(locale),
        notes_tool(locale),
        respond_agent_permission_tool(locale),
    ]
}

/// The tools this session declares.
///
/// **External contract** — `frontend-tools.mjs:239-246`
/// (`frontendTools`). [`ENTER_SLEEP`] is appended only when the client
/// declared [`SLEEPING_CLIENT_STATE`].
///
/// `mode` further narrows the set (`docs/architecture.md` §2): see
/// [`crate::mode::ModePlan::declares_tool`].
#[must_use]
pub fn frontend_tools(
    locale: Locale,
    client_states: &[String],
    mode: crate::mode::ModePlan,
) -> Vec<Value> {
    let supports_sleep = client_states
        .iter()
        .any(|state| state == SLEEPING_CLIENT_STATE);
    let mut declared = tools(locale);
    if supports_sleep {
        declared.push(enter_sleep_tool(locale));
    }
    declared.retain(|tool| {
        tool.get("function")
            .and_then(|function| function.get("name"))
            .and_then(Value::as_str)
            .is_some_and(|name| mode.declares_tool(name))
    });
    declared
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mode::ModePlan;
    use pretty_assertions::assert_eq;
    use via_protocol::SessionMode;

    fn tool_named(tools: &[Value], name: &str) -> Value {
        tools
            .iter()
            .find(|tool| tool["function"]["name"] == name)
            .cloned()
            .unwrap_or_else(|| panic!("{name} is not declared"))
    }

    fn parameters(tool: &Value) -> &Value {
        &tool["function"]["parameters"]
    }

    #[test]
    fn the_eight_always_declared_tools_are_in_upstream_order() {
        let declared = tools(Locale::Zh);
        let names: Vec<&str> = declared
            .iter()
            .map(|tool| {
                tool["function"]["name"]
                    .as_str()
                    .expect("every tool is named")
            })
            .collect();
        assert_eq!(names, ALWAYS_DECLARED.to_vec());
    }

    #[test]
    fn cancel_agent_task_has_no_required_key_at_all() {
        let tool = cancel_agent_task_tool(Locale::Zh);
        let schema = parameters(&tool)
            .as_object()
            .expect("the parameters are an object");
        assert!(
            !schema.contains_key("required"),
            "upstream omits the key entirely; `required: []` is a different schema",
        );
        // And the key order is `type`, `properties`, `additionalProperties`.
        let keys: Vec<&String> = schema.keys().collect();
        assert_eq!(keys, vec!["type", "properties", "additionalProperties"]);
    }

    #[test]
    fn every_tools_required_list_matches_upstream() {
        let cases: [(&str, Option<Vec<&str>>); 9] = [
            (SPAWN_THINKING, Some(vec!["objective"])),
            (SCHEDULE_REMINDER, Some(vec!["execute_at", "reminder"])),
            (CANCEL_AGENT_TASK, None),
            (GET_AGENT_TASK_STATUS, None),
            (GET_CURRENT_TIME, None),
            (MEMORY, Some(vec!["action"])),
            (NOTES, Some(vec!["action"])),
            (
                RESPOND_AGENT_PERMISSION,
                Some(vec!["authorization_id", "decision"]),
            ),
            (ENTER_SLEEP, None),
        ];
        let mut declared = tools(Locale::Zh);
        declared.push(enter_sleep_tool(Locale::Zh));
        for (name, required) in cases {
            let tool = tool_named(&declared, name);
            let actual = parameters(&tool)
                .get("required")
                .and_then(Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<&str>>()
                });
            assert_eq!(actual, required, "{name}");
        }
    }

    #[test]
    fn every_tool_forbids_additional_properties() {
        let mut declared = tools(Locale::Zh);
        declared.push(enter_sleep_tool(Locale::Zh));
        for tool in &declared {
            assert_eq!(
                parameters(tool)["additionalProperties"],
                Value::Bool(false),
                "{}",
                tool["function"]["name"],
            );
            assert_eq!(tool["type"], "function");
        }
    }

    #[test]
    fn the_two_parameterless_tools_have_an_empty_properties_object() {
        for tool in [
            get_current_time_tool(Locale::Zh),
            enter_sleep_tool(Locale::Zh),
        ] {
            assert_eq!(parameters(&tool)["properties"], json!({}));
        }
    }

    #[test]
    fn the_enum_vocabularies_are_the_catalogued_ones() {
        let memory = memory_tool(Locale::Zh);
        assert_eq!(
            parameters(&memory)["properties"]["action"]["enum"],
            json!(["read", "append", "replace"]),
        );
        assert_eq!(
            parameters(&memory)["properties"]["document"]["enum"],
            json!(["user", "memory", "all"]),
        );
        let notes = notes_tool(Locale::Zh);
        assert_eq!(
            parameters(&notes)["properties"]["action"]["enum"],
            json!(["lists", "show", "add", "remove", "clear", "drop"]),
        );
        let permission = respond_agent_permission_tool(Locale::Zh);
        assert_eq!(
            parameters(&permission)["properties"]["decision"]["enum"],
            json!(["always", "reject"]),
        );
        let reminder = schedule_reminder_tool(Locale::Zh);
        assert_eq!(
            parameters(&reminder)["properties"]["type"]["enum"],
            json!(["reminder", "task"]),
        );
        assert_eq!(
            parameters(&reminder)["properties"]["recurrence"]["enum"],
            json!(["once", "daily", "weekly", "weekdays"]),
        );
    }

    #[test]
    fn the_array_bounds_are_the_catalogued_ones() {
        assert_eq!(
            parameters(&spawn_thinking_tool(Locale::Zh))["properties"]["input_refs"]["maxItems"],
            json!(8),
        );
        assert_eq!(
            parameters(&notes_tool(Locale::Zh))["properties"]["items"]["maxItems"],
            json!(20),
        );
    }

    #[test]
    fn the_zh_descriptions_are_upstreams_own_text() {
        // The catalog is the single source; this asserts the wiring, not the
        // sentence — `via-conformance` asserts the sentence against
        // `contracts.json`.
        let tool = spawn_thinking_tool(Locale::Zh);
        assert_eq!(
            tool["function"]["description"],
            json!(t(Locale::Zh, keys::VOICE_TOOL_SPAWN_THINKING_DESCRIPTION)),
        );
        assert!(
            tool["function"]["description"]
                .as_str()
                .is_some_and(|text| text.contains("get_agent_task_status")),
            "the description names the sibling tool by name",
        );
    }

    #[test]
    fn descriptions_follow_the_locale() {
        let zh = spawn_thinking_tool(Locale::Zh);
        let en = spawn_thinking_tool(Locale::En);
        let ko = spawn_thinking_tool(Locale::Ko);
        assert_ne!(zh["function"]["description"], en["function"]["description"]);
        assert_ne!(en["function"]["description"], ko["function"]["description"]);
        // The name and the schema shape never move.
        assert_eq!(zh["function"]["name"], en["function"]["name"]);
        assert_eq!(parameters(&zh)["required"], parameters(&ko)["required"],);
    }

    #[test]
    fn enter_sleep_is_declared_only_for_a_client_that_can_sleep() {
        let plan = ModePlan::new(SessionMode::Agent, true);
        let without = frontend_tools(Locale::Zh, &[], plan);
        assert_eq!(without.len(), 8);
        assert!(
            !without
                .iter()
                .any(|tool| tool["function"]["name"] == ENTER_SLEEP)
        );

        let with = frontend_tools(Locale::Zh, &["sleeping".to_owned()], plan);
        assert_eq!(with.len(), 9);
        assert_eq!(with[8]["function"]["name"], ENTER_SLEEP, "appended last");

        // An unrelated declared state does not unlock it.
        let other = frontend_tools(Locale::Zh, &["hidden".to_owned()], plan);
        assert_eq!(other.len(), 8);
    }

    #[test]
    fn dictation_declares_no_tools_at_all() {
        let plan = ModePlan::new(SessionMode::Dictation, true);
        assert!(frontend_tools(Locale::Zh, &["sleeping".to_owned()], plan).is_empty());
    }

    #[test]
    fn direct_declares_the_control_tools_but_never_spawn_thinking() {
        let plan = ModePlan::new(SessionMode::Direct, true);
        let names: Vec<String> = frontend_tools(Locale::Zh, &[], plan)
            .iter()
            .map(|tool| {
                tool["function"]["name"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned()
            })
            .collect();
        assert!(!names.contains(&SPAWN_THINKING.to_owned()));
        assert!(names.contains(&GET_AGENT_TASK_STATUS.to_owned()));
        assert!(names.contains(&MEMORY.to_owned()));
        assert_eq!(names.len(), 7);
    }

    #[test]
    fn the_tool_name_table_covers_every_declaration() {
        assert_eq!(ALL_TOOL_NAMES.len(), 9);
        for name in ALWAYS_DECLARED {
            assert!(ALL_TOOL_NAMES.contains(&name), "{name}");
        }
        assert!(ALL_TOOL_NAMES.contains(&ENTER_SLEEP));
    }

    #[test]
    fn no_tool_name_carries_the_old_brand() {
        for name in ALL_TOOL_NAMES {
            assert!(!name.contains("qwen"), "{name}");
        }
    }
}
