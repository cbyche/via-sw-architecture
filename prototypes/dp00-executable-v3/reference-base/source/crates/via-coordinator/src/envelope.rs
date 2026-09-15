//! The delegation envelope — what the backend agent is actually asked.
//!
//! `buildCoordinatorPrompt`, `server/src/agent/coordinator.mjs:143-237`. One
//! function, and almost every byte of it is an external contract: the JSON
//! envelope's shape is catalogued (`json-field` / *coordination request
//! envelope*), the example line under it is catalogued (`prompt-text` /
//! *coordinator expected-JSON example line*), and the prose around both is what
//! makes the difference between a backend that delegates and one that answers
//! from the coordinator session's memory.
//!
//! # What the envelope carries, and what it does not
//!
//! The user's request, recent voice context, the user's local preferences and
//! memory, the Work already in flight, and a **final response shape**. It does
//! **not** tell the backend how to use its own capabilities: which tools to
//! reach for, how to plan, whether to subagent. `docs/architecture.md` §6 —
//! *"a harness answers prompts and emits events"* — and the corollary here is
//! that the backend owns its execution strategy. The one thing VIA does
//! constrain is *routing*: new independent work opens a Layer-3 Session,
//! continuation finds the existing one, and everything else runs in the
//! coordinator Session.
//!
//! # Two literal-looking details that are load-bearing
//!
//! **The blank lines are part of the contract.** Upstream builds the prompt as
//! an array and `join('\n')`s it, and two of the entries are conditional
//! expressions that evaluate to `''` when their condition is false — so a
//! prompt with no `user_preferences` still carries the blank line where its
//! note would have been. Reproduced, and asserted.
//!
//! **`timestamp` is supplied, not read.** Upstream calls
//! `new Date().toISOString()` inline. This module takes a
//! [`chrono::DateTime<Utc>`] instead, for the reason `via-downstream`'s
//! [`CancelOutcome::confirm`](via_downstream::CancelOutcome::confirm) already
//! records: a contract module that reads a clock cannot be asserted byte for
//! byte.

use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::{Map, Value};
use via_conversation::markdown_store::MemoryDocument;
use via_conversation::{Message, MessageRole};
use via_core::memory_scopes::{MEMORY_SCOPE, canonical_scope, is_directive_scope};
use via_downstream::text::clean;
use via_i18n::{Locale, format, keys, t};
use via_work::PublicWork;
use via_work::text::slice_units;

/// The envelope's `protocol` field.
///
/// **External contract**, renamed. Upstream `qwen-audio-agent.coordination.v1`
/// (`server/src/agent/coordinator.mjs:181`); `docs/rebrand.md` row 81. The
/// model reads it together with
/// [`BACKEND_INSTRUCTIONS_OPEN_TAG`](crate::instructions::BACKEND_INSTRUCTIONS_OPEN_TAG),
/// so the two renames have to land together.
pub const COORDINATION_PROTOCOL: &str = "via.coordination.v1";

/// The envelope's `owner_scope` field. Always this literal.
///
/// **External contract** — `coordinator.mjs:183`. It says *whose* request this
/// is without naming them: the owner id itself never reaches the backend.
pub const OWNER_SCOPE: &str = "current_authenticated_user";

/// The `client_context.working_directory_scope` field. Always this literal.
///
/// **External contract** — `coordinator.mjs:190`. It is what tells the model
/// that the directory belongs to the *client process* that raised the turn and
/// is therefore context rather than an instruction.
pub const WORKING_DIRECTORY_SCOPE: &str = "client_process";

/// The `delivery.completion` field. Always this literal.
///
/// **External contract** — `coordinator.mjs:204`. Completion is delivered by
/// the Gateway; the backend never decides whether the user hears the answer.
pub const COMPLETION_AUTOMATIC: &str = "automatic";

/// `delivery.status` when the caller allows interim status.
///
/// **External contract** — `coordinator.mjs:205`.
pub const DELIVERY_STATUS_MEANINGFUL_ONLY: &str = "meaningful_only";

/// `delivery.status` when it does not.
///
/// **External contract** — `coordinator.mjs:205`, and the default: upstream's
/// test is `delivery.allowStatus === true`, so anything but an explicit `true`
/// is silence.
pub const DELIVERY_STATUS_SILENT: &str = "silent";

/// The `timezone` an envelope falls back to.
///
/// **External contract** — `coordinator.mjs:187` (`clean(timeZone) || 'UTC'`).
pub const DEFAULT_TIME_ZONE: &str = "UTC";

/// The `kind` a trusted backend event falls back to.
///
/// **External contract** — `coordinator.mjs:174`
/// (`clean(backendEvent.kind) || 'native_task_result'`).
pub const TRUSTED_BACKEND_EVENT_KIND: &str = "native_task_result";

/// The bound on a trusted backend event's `content`.
///
/// **External contract** — `coordinator.mjs:176` (`.slice(0, 12000)`). The same
/// number as
/// [`MAX_DELEGATION_RESULT_CHARS`](crate::prompts::MAX_DELEGATION_RESULT_CHARS)
/// and for the same reason: both carry a verified third-layer result into a
/// coordinator turn.
pub const TRUSTED_BACKEND_EVENT_CONTENT_BOUND: usize = 12_000;

/// The opening tag around the JSON envelope.
///
/// **External contract**, renamed. Upstream `<qwen_audio_agent_request>`
/// (`coordinator.mjs:210`); `docs/rebrand.md` row 37.
pub const REQUEST_OPEN_TAG: &str = "<via_request>";

/// The closing tag around the JSON envelope.
///
/// **External contract**, renamed. Upstream `</qwen_audio_agent_request>`
/// (`coordinator.mjs:212`).
pub const REQUEST_CLOSE_TAG: &str = "</via_request>";

/// The tag around the user's directive preferences.
///
/// The same tag `via-conversation` puts the document behind for the *realtime*
/// model ([`via_conversation::context::USER_PREFERENCES_TAG`]), because it is
/// the same document with the same authority — `docs/rebrand.md` lists it under
/// KEEP, noting that `PROMPT.md` refers to these tags by name.
pub const USER_PREFERENCES_TAG: &str = via_conversation::context::USER_PREFERENCES_TAG;

/// The tag around the user's long-term memory.
///
/// As [`USER_PREFERENCES_TAG`].
pub const USER_MEMORY_TAG: &str = via_conversation::context::USER_MEMORY_TAG;

/// The tag around the last few voice turns.
///
/// **External contract** — `coordinator.mjs:217`. Brand-free, so it is KEEP.
pub const RECENT_VOICE_CONTEXT_TAG: &str = "recent_voice_context";

/// The tag around the Work already in flight.
///
/// **External contract** — `coordinator.mjs:218`. Brand-free, so it is KEEP.
pub const VOICE_WORK_CONTEXT_TAG: &str = "voice_work_context";

/// How many conversation turns reach the envelope.
///
/// **External contract** — `coordinator.mjs:122` (`messages.slice(-10)`): the
/// **last** ten, not the first.
pub const MAX_CONTEXT_MESSAGES: usize = 10;

/// The bound on one conversation turn's text.
///
/// **External contract** — `coordinator.mjs:126` (`.slice(0, 1000)`).
pub const CONTEXT_CONTENT_BOUND: usize = 1_000;

/// How many memory records reach the envelope.
///
/// **External contract** — `coordinator.mjs:166` (`.slice(0, 20)`).
pub const MAX_MEMORY_RECORDS: usize = 20;

/// How many in-flight Work items reach the envelope.
///
/// **External contract** — `coordinator.mjs:134` (`tasks.slice(0, 10)`): the
/// **first** ten, which is the opposite end from the conversation slice.
pub const MAX_WORK_LINES: usize = 10;

/// The bound on an in-flight Work item's result.
///
/// **External contract** — `coordinator.mjs:138` (`.slice(0, 500)`).
pub const WORK_RESULT_BOUND: usize = 500;

/// The separator joining the three fields of one `voice_work_context` line.
///
/// **External contract** — `coordinator.mjs:139` (`join('；')`), a full-width
/// semicolon. It is punctuation rather than prose, so it is a `const` here
/// rather than a `via-i18n` key — the same call `via-process` records for its
/// failure/stderr separator, and the same one `via-conversation` makes for
/// [`INPUT_SUMMARY_SEPARATOR`](via_conversation::context::INPUT_SUMMARY_SEPARATOR).
pub const WORK_FIELD_SEPARATOR: char = '；';

/// The separator between two memory records.
///
/// **External contract** — `coordinator.mjs:170` (`join('\n\n')`). A blank line
/// between records, where preferences are joined by a single newline.
pub const MEMORY_RECORD_SEPARATOR: &str = "\n\n";

/// A backend result the Gateway has already verified, carried back into a
/// coordinator turn.
///
/// **External contract** — the `input.trusted_backend_event` half of
/// `coordinator.mjs:172-179`. The field names are the model's, and the note
/// beside it ([`keys::COORDINATOR_TRUSTED_BACKEND_EVENT_NOTE`]) is what stops
/// the model from reading it as a new user instruction.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrustedBackendEvent {
    /// Defaults to [`TRUSTED_BACKEND_EVENT_KIND`] when blank.
    pub kind: String,
    /// The request this result answers.
    pub parent_request_id: String,
    /// The result, bounded to [`TRUSTED_BACKEND_EVENT_CONTENT_BOUND`].
    pub content: String,
    /// The failure, when the backend reported one.
    pub error: String,
}

impl TrustedBackendEvent {
    /// The JSON the envelope carries, with the catalogued field order.
    #[must_use]
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        let kind = clean(&self.kind);
        map.insert(
            "kind".to_owned(),
            Value::String(
                if kind.is_empty() {
                    TRUSTED_BACKEND_EVENT_KIND
                } else {
                    kind
                }
                .to_owned(),
            ),
        );
        map.insert(
            "parent_request_id".to_owned(),
            Value::String(clean(&self.parent_request_id).to_owned()),
        );
        map.insert(
            "content".to_owned(),
            Value::String(slice_units(
                clean(&self.content),
                TRUSTED_BACKEND_EVENT_CONTENT_BOUND,
            )),
        );
        map.insert(
            "error".to_owned(),
            Value::String(clean(&self.error).to_owned()),
        );
        Value::Object(map)
    }
}

/// One attachment, described rather than embedded.
///
/// **External contract** — `inputAttachmentMetadata`,
/// `shared/input-parts.mjs:277-284`, as the envelope carries it:
/// `{label, name, mime_type, bytes}`.
///
/// The values are supplied rather than derived. Deriving `label` means
/// reproducing `inputPartReference`'s `[Image 1]` / `[File 2]` de-duplication,
/// and deriving `bytes` means decoding a `data:` URL — both belong to whoever
/// ports `shared/input-parts.mjs`, which `docs/deviations/phase-2.md` already
/// records as somebody else's. The bytes themselves reach the backend as
/// [`via_downstream::PromptAttachment`]s on the same turn; this is only their
/// description.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvelopeAttachment {
    /// `[Image 1]` and friends — the token the model can refer to.
    pub label: String,
    /// The file name, or `None`, which renders as `null`.
    pub name: Option<String>,
    /// The MIME type.
    pub mime_type: String,
    /// The decoded size, or `None`, which renders as `null`.
    pub bytes: Option<u64>,
}

impl EnvelopeAttachment {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("label".to_owned(), Value::String(self.label.clone()));
        map.insert(
            "name".to_owned(),
            self.name.clone().map_or(Value::Null, Value::String),
        );
        map.insert(
            "mime_type".to_owned(),
            Value::String(self.mime_type.clone()),
        );
        map.insert(
            "bytes".to_owned(),
            self.bytes.map_or(Value::Null, Value::from),
        );
        Value::Object(map)
    }
}

/// How this turn's answer will be delivered.
///
/// **External contract** — the `delivery` block, `coordinator.mjs:202-206`.
/// Both fields are stated as their upstream *defaults*: upstream writes
/// `delivery.voiceConnected !== false` and `delivery.allowStatus === true`, so
/// an absent `delivery` object means *connected* and *silent*. [`Default`]
/// reproduces exactly that.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Delivery {
    /// Whether a voice client is listening.
    pub voice_connected: bool,
    /// Whether interim status is welcome.
    pub allow_status: bool,
}

impl Default for Delivery {
    fn default() -> Self {
        Self {
            voice_connected: true,
            allow_status: false,
        }
    }
}

impl Delivery {
    /// The `status` literal this delivery renders as.
    #[must_use]
    pub const fn status(self) -> &'static str {
        if self.allow_status {
            DELIVERY_STATUS_MEANINGFUL_ONLY
        } else {
            DELIVERY_STATUS_SILENT
        }
    }
}

/// Everything one coordination turn is built from.
///
/// The argument object of `buildCoordinatorPrompt`
/// (`server/src/agent/coordinator.mjs:143-157`), with upstream's two callers'
/// options folded in — `Coordinator.run` merges `options.coordinationRunId`,
/// `options.sessionId` and `options.turnId` into the same object before
/// calling, so they are fields here rather than a second parameter.
#[derive(Debug, Clone, Default)]
pub struct CoordinationRequest<'a> {
    /// `input.final_asr` — the user's own words this turn.
    pub original_request: &'a str,
    /// `input.objective` — the voice frontend's conservative summary.
    pub objective: &'a str,
    /// A verified backend result being carried back in.
    pub backend_event: Option<&'a TrustedBackendEvent>,
    /// The user's memory documents. `user` / `profile` / `rules` become
    /// `<user_preferences>`; `memory` / `facts` / `long_term` become
    /// `<user_memory>`.
    pub user_memories: &'a [MemoryDocument],
    /// The conversation so far; the last [`MAX_CONTEXT_MESSAGES`] are used.
    pub conversation_context: &'a [Message],
    /// The Work already in flight; the first [`MAX_WORK_LINES`] are used.
    pub active_work: &'a [PublicWork],
    /// The user's IANA zone, or blank for [`DEFAULT_TIME_ZONE`].
    pub time_zone: &'a str,
    /// The client process's launch directory, or blank for `null`.
    pub working_directory: &'a str,
    /// `request_id` — the Work id this turn is being run for.
    pub coordination_run_id: &'a str,
    /// Which voice session raised it.
    pub voice_session_id: &'a str,
    /// Which turn raised it.
    pub turn_id: &'a str,
    /// How the answer will be delivered.
    pub delivery: Delivery,
    /// The attachments' descriptions.
    pub attachments: &'a [EnvelopeAttachment],
}

/// Build the JSON envelope.
///
/// **External contract** — `docs/reference/contracts.json` (`json-field` /
/// *coordination request envelope*). Field order is insertion order and is what
/// the model reads; `serde_json`'s `preserve_order` is what keeps it.
///
/// `attachments` and `trusted_backend_event` are **conditionally present**:
/// upstream spreads them into `input` only when they are non-empty, and an
/// always-present `"attachments": []` would be a different contract.
#[must_use]
pub fn coordination_envelope(request: &CoordinationRequest<'_>, timestamp: DateTime<Utc>) -> Value {
    let mut client_context = Map::new();
    let directory = clean(request.working_directory);
    client_context.insert(
        "working_directory".to_owned(),
        if directory.is_empty() {
            Value::Null
        } else {
            Value::String(directory.to_owned())
        },
    );
    client_context.insert(
        "working_directory_scope".to_owned(),
        Value::String(WORKING_DIRECTORY_SCOPE.to_owned()),
    );

    let mut input = Map::new();
    input.insert(
        "final_asr".to_owned(),
        Value::String(clean(request.original_request).to_owned()),
    );
    input.insert(
        "objective".to_owned(),
        Value::String(clean(request.objective).to_owned()),
    );
    if !request.attachments.is_empty() {
        input.insert(
            "attachments".to_owned(),
            Value::Array(
                request
                    .attachments
                    .iter()
                    .map(EnvelopeAttachment::to_json)
                    .collect(),
            ),
        );
    }
    if let Some(event) = request.backend_event {
        input.insert("trusted_backend_event".to_owned(), event.to_json());
    }

    let mut delivery = Map::new();
    delivery.insert(
        "voice_connected".to_owned(),
        Value::Bool(request.delivery.voice_connected),
    );
    delivery.insert(
        "completion".to_owned(),
        Value::String(COMPLETION_AUTOMATIC.to_owned()),
    );
    delivery.insert(
        "status".to_owned(),
        Value::String(request.delivery.status().to_owned()),
    );

    let time_zone = clean(request.time_zone);
    let mut envelope = Map::new();
    envelope.insert(
        "protocol".to_owned(),
        Value::String(COORDINATION_PROTOCOL.to_owned()),
    );
    envelope.insert(
        "request_id".to_owned(),
        Value::String(clean(request.coordination_run_id).to_owned()),
    );
    envelope.insert(
        "owner_scope".to_owned(),
        Value::String(OWNER_SCOPE.to_owned()),
    );
    envelope.insert(
        "voice_session_id".to_owned(),
        Value::String(clean(request.voice_session_id).to_owned()),
    );
    envelope.insert(
        "turn_id".to_owned(),
        Value::String(clean(request.turn_id).to_owned()),
    );
    envelope.insert(
        "timestamp".to_owned(),
        Value::String(timestamp.to_rfc3339_opts(SecondsFormat::Millis, true)),
    );
    envelope.insert(
        "timezone".to_owned(),
        Value::String(
            if time_zone.is_empty() {
                DEFAULT_TIME_ZONE
            } else {
                time_zone
            }
            .to_owned(),
        ),
    );
    envelope.insert("client_context".to_owned(), Value::Object(client_context));
    envelope.insert("input".to_owned(), Value::Object(input));
    envelope.insert("delivery".to_owned(), Value::Object(delivery));
    Value::Object(envelope)
}

/// The `<user_preferences>` lines, or an empty vector when there are none.
///
/// **External contract** — `coordinator.mjs:158-162`. Selection is by
/// [`is_directive_scope`], so the legacy `profile` and `rules` spellings still
/// count; a `markdown` document is used whole and anything else becomes a
/// bullet.
#[must_use]
fn preference_lines(memories: &[MemoryDocument]) -> Vec<String> {
    memories
        .iter()
        .filter(|memory| is_directive_scope(clean(&memory.scope)))
        .map(|memory| {
            if memory.format == via_conversation::markdown_store::DOCUMENT_FORMAT {
                clean(&memory.content).to_owned()
            } else {
                format!("- {}", clean(&memory.content))
            }
        })
        .collect()
}

/// The body of the `<user_memory>` block.
///
/// **External contract** — `coordinator.mjs:163-171`. A non-markdown record is
/// rendered as `- [<scope>] <content>`, and the scope falls back to
/// [`MEMORY_SCOPE`] when [`canonical_scope`] cannot name one — which it can
/// always do here, because the filter above it already required the scope to
/// canonicalise to `memory`. The fallback is reproduced anyway rather than
/// dropped: it is one `||` in a catalogued line.
#[must_use]
fn memory_body(memories: &[MemoryDocument], locale: Locale) -> String {
    let records: Vec<&MemoryDocument> = memories
        .iter()
        .filter(|memory| canonical_scope(clean(&memory.scope)) == MEMORY_SCOPE)
        .take(MAX_MEMORY_RECORDS)
        .collect();
    if records.is_empty() {
        return t(locale, keys::COORDINATOR_TASK_NONE).to_owned();
    }
    records
        .iter()
        .map(|memory| {
            if memory.format == via_conversation::markdown_store::DOCUMENT_FORMAT {
                clean(&memory.content).to_owned()
            } else {
                let scope = canonical_scope(clean(&memory.scope));
                let scope = if scope.is_empty() {
                    MEMORY_SCOPE.to_owned()
                } else {
                    scope
                };
                format!("- [{scope}] {}", clean(&memory.content))
            }
        })
        .collect::<Vec<_>>()
        .join(MEMORY_RECORD_SEPARATOR)
}

/// The body of the `<recent_voice_context>` block.
///
/// **External contract** — `contextLines`, `coordinator.mjs:120-130`. The last
/// [`MAX_CONTEXT_MESSAGES`] turns, each `"<role>: <text>"` with an ASCII colon
/// and a space, blank turns dropped, and [`keys::COORDINATOR_TASK_NONE`] when
/// nothing survives.
///
/// The two role words come from [`keys::REALTIME_ROLE_USER`] /
/// [`keys::REALTIME_ROLE_ASSISTANT`] — the same keys `via-conversation` renders
/// `<recent_conversation>` with, because they are the same two words for the
/// same two speakers.
#[must_use]
fn context_lines(messages: &[Message], locale: Locale) -> String {
    let start = messages.len().saturating_sub(MAX_CONTEXT_MESSAGES);
    let rendered: Vec<String> = messages[start..]
        .iter()
        .filter_map(|message| {
            let content = slice_units(clean(&message.content), CONTEXT_CONTENT_BOUND);
            if content.is_empty() {
                return None;
            }
            let role = match message.role {
                MessageRole::Assistant => keys::REALTIME_ROLE_ASSISTANT,
                MessageRole::User => keys::REALTIME_ROLE_USER,
            };
            Some(format!("{}: {content}", t(locale, role)))
        })
        .collect();
    if rendered.is_empty() {
        return t(locale, keys::COORDINATOR_TASK_NONE).to_owned();
    }
    rendered.join("\n")
}

/// The body of the `<voice_work_context>` block.
///
/// **External contract** — `runLines`, `coordinator.mjs:132-141`. One line per
/// Work: the objective (or [`keys::COORDINATOR_TASK_UNNAMED`]), the status, and
/// the result when there is one, joined by [`WORK_FIELD_SEPARATOR`].
///
/// # One upstream branch is unrepresentable
///
/// Upstream writes `clean(task.status) || 'unknown'`. [`PublicWork::status`] is
/// a [`via_protocol::WorkStatus`], which always spells itself, so the
/// `'unknown'` arm has no input that reaches it. Recorded in
/// `docs/deviations/phase-3.md` rather than reproduced as dead code.
#[must_use]
fn work_lines(work: &[PublicWork], locale: Locale) -> String {
    let rendered: Vec<String> = work
        .iter()
        .take(MAX_WORK_LINES)
        .map(|item| {
            let objective = clean(&item.objective);
            let objective = if objective.is_empty() {
                t(locale, keys::COORDINATOR_TASK_UNNAMED).to_owned()
            } else {
                objective.to_owned()
            };
            let mut fields = vec![
                format!("- {objective}"),
                format(
                    locale,
                    keys::COORDINATOR_TASK_STATUS_FIELD,
                    &[("status", item.status.as_str())],
                ),
            ];
            if let Some(result) = item.result.as_deref().filter(|value| !value.is_empty()) {
                fields.push(format(
                    locale,
                    keys::COORDINATOR_TASK_RESULT_FIELD,
                    &[("result", &slice_units(clean(result), WORK_RESULT_BOUND))],
                ));
            }
            fields.join(&WORK_FIELD_SEPARATOR.to_string())
        })
        .collect();
    if rendered.is_empty() {
        return t(locale, keys::COORDINATOR_TASK_NONE).to_owned();
    }
    rendered.join("\n")
}

/// Build the whole coordinator prompt.
///
/// **External contract** — `buildCoordinatorPrompt`,
/// `server/src/agent/coordinator.mjs:209-236`, joined with `\n`.
///
/// The two conditional prose lines evaluate to the empty string when their
/// condition is false and are still joined, so a request without preferences
/// and without a trusted backend event carries **two blank lines** where those
/// notes would be. That is upstream's array-join, and it is asserted in this
/// module's tests rather than tidied away.
#[must_use]
pub fn build_coordinator_prompt(
    request: &CoordinationRequest<'_>,
    timestamp: DateTime<Utc>,
    locale: Locale,
) -> String {
    let preferences = preference_lines(request.user_memories);
    let envelope = coordination_envelope(request, timestamp);
    let envelope_json =
        serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| envelope.to_string());

    let mut lines: Vec<String> = vec![
        REQUEST_OPEN_TAG.to_owned(),
        envelope_json,
        REQUEST_CLOSE_TAG.to_owned(),
    ];
    if !preferences.is_empty() {
        lines.push(format!(
            "<{USER_PREFERENCES_TAG}>\n{}\n</{USER_PREFERENCES_TAG}>",
            preferences.join("\n"),
        ));
    }
    lines.push(format!(
        "<{USER_MEMORY_TAG}>\n{}\n</{USER_MEMORY_TAG}>",
        memory_body(request.user_memories, locale),
    ));
    lines.push(format!(
        "<{RECENT_VOICE_CONTEXT_TAG}>\n{}\n</{RECENT_VOICE_CONTEXT_TAG}>",
        context_lines(request.conversation_context, locale),
    ));
    lines.push(format!(
        "<{VOICE_WORK_CONTEXT_TAG}>\n{}\n</{VOICE_WORK_CONTEXT_TAG}>",
        work_lines(request.active_work, locale),
    ));
    lines.push(String::new());
    lines.push(t(locale, keys::COORDINATOR_INTERFACE_NOTE).to_owned());
    lines.push(t(locale, keys::COORDINATOR_CLIENT_CONTEXT_NOTE).to_owned());
    lines.push(t(locale, keys::COORDINATOR_USER_MEMORY_NOTE).to_owned());
    lines.push(if preferences.is_empty() {
        String::new()
    } else {
        t(locale, keys::COORDINATOR_USER_PREFERENCES_NOTE).to_owned()
    });
    lines.push(if request.backend_event.is_some() {
        t(locale, keys::COORDINATOR_TRUSTED_BACKEND_EVENT_NOTE).to_owned()
    } else {
        String::new()
    });
    lines.push(t(locale, keys::COORDINATOR_RETURN_JSON_HEADER).to_owned());
    lines.push(expected_json_example(locale));
    lines.push(t(locale, keys::COORDINATOR_PRESENTATION_NOTE).to_owned());
    lines.push(t(locale, keys::COORDINATOR_ROUTING_NOTE).to_owned());
    lines.push(t(locale, keys::COORDINATOR_DELEGATION_NOTE).to_owned());
    lines.push(t(locale, keys::COORDINATOR_STATUS_NOTE).to_owned());
    lines.push(t(locale, keys::COORDINATOR_FINAL_ONLY_NOTE).to_owned());
    lines.join("\n")
}

/// The literal JSON example shown to the model under
/// [`keys::COORDINATOR_RETURN_JSON_HEADER`].
///
/// **External contract** — `docs/reference/contracts.json` (`prompt-text` /
/// *coordinator expected-JSON example line*), `coordinator.mjs:230`.
///
/// It goes through [`via_i18n::format`] rather than [`via_i18n::t`] because the
/// catalog entry escapes its braces as `{{` / `}}`; `t` would hand back the
/// escaped template and the model would be shown JSON it cannot parse.
#[must_use]
pub fn expected_json_example(locale: Locale) -> String {
    format(locale, keys::COORDINATOR_EXPECTED_JSON_EXAMPLE, &[])
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use via_conversation::sync::MessageSource;

    fn at(millis: i64) -> DateTime<Utc> {
        Utc.timestamp_millis_opt(millis)
            .single()
            .unwrap_or_else(Utc::now)
    }

    fn message(role: MessageRole, content: &str) -> Message {
        Message {
            seq: 1,
            id: "id".to_owned(),
            role,
            content: content.to_owned(),
            source: MessageSource::VoiceUser,
            turn_id: None,
            task_id: None,
            task_ids: Vec::new(),
            inputs: Vec::new(),
            created_at: 0,
        }
    }

    fn document(scope: &str, content: &str) -> MemoryDocument {
        MemoryDocument::new(scope, content.to_owned(), content)
    }

    #[test]
    fn the_envelope_field_order_is_the_catalogued_order() {
        let request = CoordinationRequest {
            original_request: "  继续改刚才那个页面  ",
            objective: "继续修改此前讨论的页面",
            coordination_run_id: " work-one ",
            working_directory: "/Users/me/codes/current-project",
            ..CoordinationRequest::default()
        };
        let envelope = coordination_envelope(&request, at(0));
        let object = envelope.as_object().expect("object");
        assert_eq!(
            object.keys().map(String::as_str).collect::<Vec<_>>(),
            [
                "protocol",
                "request_id",
                "owner_scope",
                "voice_session_id",
                "turn_id",
                "timestamp",
                "timezone",
                "client_context",
                "input",
                "delivery",
            ],
        );
        assert_eq!(envelope["protocol"], COORDINATION_PROTOCOL);
        assert_eq!(envelope["request_id"], "work-one");
        assert_eq!(envelope["owner_scope"], OWNER_SCOPE);
        assert_eq!(envelope["timestamp"], "1970-01-01T00:00:00.000Z");
        assert_eq!(envelope["timezone"], DEFAULT_TIME_ZONE);
        assert_eq!(
            envelope["client_context"]["working_directory"],
            "/Users/me/codes/current-project",
        );
        assert_eq!(
            envelope["client_context"]["working_directory_scope"],
            WORKING_DIRECTORY_SCOPE,
        );
        assert_eq!(envelope["input"]["final_asr"], "继续改刚才那个页面");
        assert_eq!(envelope["delivery"]["voice_connected"], true);
        assert_eq!(envelope["delivery"]["completion"], COMPLETION_AUTOMATIC);
        assert_eq!(envelope["delivery"]["status"], DELIVERY_STATUS_SILENT);
    }

    #[test]
    fn an_absent_working_directory_is_null_not_an_empty_string() {
        let envelope = coordination_envelope(&CoordinationRequest::default(), at(0));
        assert_eq!(envelope["client_context"]["working_directory"], Value::Null);
    }

    #[test]
    fn attachments_and_trusted_events_are_absent_rather_than_empty() {
        let bare = coordination_envelope(&CoordinationRequest::default(), at(0));
        let input = bare["input"].as_object().expect("object");
        assert_eq!(
            input.keys().map(String::as_str).collect::<Vec<_>>(),
            ["final_asr", "objective"]
        );

        let event = TrustedBackendEvent {
            kind: String::new(),
            parent_request_id: " work-one ".to_owned(),
            content: format!(
                "  {}  ",
                "x".repeat(TRUSTED_BACKEND_EVENT_CONTENT_BOUND + 40)
            ),
            error: "  ".to_owned(),
        };
        let attachments = [EnvelopeAttachment {
            label: "[Image 1]".to_owned(),
            name: Some("reference.png".to_owned()),
            mime_type: "image/png".to_owned(),
            bytes: Some(5),
        }];
        let full = coordination_envelope(
            &CoordinationRequest {
                backend_event: Some(&event),
                attachments: &attachments,
                ..CoordinationRequest::default()
            },
            at(0),
        );
        let input = full["input"].as_object().expect("object");
        assert_eq!(
            input.keys().map(String::as_str).collect::<Vec<_>>(),
            [
                "final_asr",
                "objective",
                "attachments",
                "trusted_backend_event"
            ],
        );
        assert_eq!(
            full["input"]["trusted_backend_event"]["kind"],
            TRUSTED_BACKEND_EVENT_KIND,
        );
        assert_eq!(full["input"]["trusted_backend_event"]["error"], "");
        assert_eq!(
            full["input"]["trusted_backend_event"]["content"]
                .as_str()
                .map(|content| content.chars().count()),
            Some(TRUSTED_BACKEND_EVENT_CONTENT_BOUND),
        );
        let attachment = &full["input"]["attachments"][0];
        assert_eq!(
            attachment
                .as_object()
                .expect("object")
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["label", "name", "mime_type", "bytes"],
        );
        assert_eq!(attachment["bytes"], 5);
    }

    #[test]
    fn an_attachment_with_no_name_or_size_renders_nulls() {
        let attachments = [EnvelopeAttachment {
            label: "[File 1]".to_owned(),
            name: None,
            mime_type: "text/plain".to_owned(),
            bytes: None,
        }];
        let envelope = coordination_envelope(
            &CoordinationRequest {
                attachments: &attachments,
                ..CoordinationRequest::default()
            },
            at(0),
        );
        assert_eq!(envelope["input"]["attachments"][0]["name"], Value::Null);
        assert_eq!(envelope["input"]["attachments"][0]["bytes"], Value::Null);
    }

    #[test]
    fn delivery_defaults_to_connected_and_silent() {
        assert_eq!(
            Delivery::default(),
            Delivery {
                voice_connected: true,
                allow_status: false,
            },
        );
        assert_eq!(Delivery::default().status(), DELIVERY_STATUS_SILENT);
        assert_eq!(
            Delivery {
                allow_status: true,
                ..Delivery::default()
            }
            .status(),
            DELIVERY_STATUS_MEANINGFUL_ONLY,
        );
    }

    #[test]
    fn preferences_are_selected_by_scope_kind_not_by_name() {
        let memories = [
            document("rules", "代码注释一律用中文"),
            document("memory", "用户喜欢苹果"),
        ];
        let prompt = build_coordinator_prompt(
            &CoordinationRequest {
                user_memories: &memories,
                ..CoordinationRequest::default()
            },
            at(0),
            Locale::Zh,
        );
        assert!(prompt.contains("<user_preferences>\n代码注释一律用中文\n</user_preferences>"));
        let memory_block = prompt
            .split_once("<user_memory>")
            .and_then(|(_, rest)| rest.split_once("</user_memory>"))
            .map(|(body, _)| body.to_owned())
            .expect("a user_memory block");
        assert!(!memory_block.contains("代码注释一律用中文"));
        assert!(memory_block.contains("用户喜欢苹果"));
    }

    #[test]
    fn the_two_conditional_notes_leave_their_blank_line_behind() {
        let prompt = build_coordinator_prompt(&CoordinationRequest::default(), at(0), Locale::Zh);
        let lines: Vec<&str> = prompt.split('\n').collect();
        let header = lines
            .iter()
            .position(|line| *line == t(Locale::Zh, keys::COORDINATOR_RETURN_JSON_HEADER))
            .expect("the return-JSON header");
        assert_eq!(
            lines[header - 2..header],
            ["", ""],
            "the preferences note and the trusted-event note are both absent, \
             and both still occupy a line",
        );
        assert!(!prompt.contains(t(Locale::Zh, keys::COORDINATOR_USER_PREFERENCES_NOTE)));
        assert!(!prompt.contains(t(Locale::Zh, keys::COORDINATOR_TRUSTED_BACKEND_EVENT_NOTE)));
    }

    #[test]
    fn the_expected_json_example_renders_real_braces() {
        let example = expected_json_example(Locale::Zh);
        assert_eq!(
            example,
            "{\"work_id\":\"request_id\",\"state\":\"completed\",\"mode\":\"respond\",\
             \"presentation\":{\"speech\":\"适合语音表达的最终结果\",\"inline\":null}}",
        );
        assert!(
            serde_json::from_str::<Value>(&example).is_ok(),
            "the model is shown JSON it can parse",
        );
        for locale in [Locale::En, Locale::Ko] {
            assert!(serde_json::from_str::<Value>(&expected_json_example(locale)).is_ok());
        }
    }

    #[test]
    fn the_context_block_keeps_the_last_ten_turns_in_order() {
        let messages: Vec<Message> = (0..13)
            .map(|index| {
                message(
                    if index % 2 == 0 {
                        MessageRole::User
                    } else {
                        MessageRole::Assistant
                    },
                    &format!("turn {index}"),
                )
            })
            .collect();
        let rendered = context_lines(&messages, Locale::Zh);
        let lines: Vec<&str> = rendered.split('\n').collect();
        assert_eq!(lines.len(), MAX_CONTEXT_MESSAGES);
        assert_eq!(lines[0], "助手: turn 3");
        assert_eq!(lines[MAX_CONTEXT_MESSAGES - 1], "用户: turn 12");
    }

    #[test]
    fn a_blank_turn_is_dropped_and_an_empty_history_says_none() {
        assert_eq!(
            context_lines(&[], Locale::Zh),
            t(Locale::Zh, keys::COORDINATOR_TASK_NONE),
        );
        assert_eq!(
            context_lines(&[message(MessageRole::User, "   ")], Locale::Zh),
            t(Locale::Zh, keys::COORDINATOR_TASK_NONE),
        );
    }

    #[test]
    fn a_long_turn_is_sliced_to_a_thousand_units() {
        let long = message(MessageRole::User, &"字".repeat(CONTEXT_CONTENT_BOUND + 50));
        let rendered = context_lines(&[long], Locale::Zh);
        let body = rendered.strip_prefix("用户: ").expect("the role prefix");
        assert_eq!(body.chars().count(), CONTEXT_CONTENT_BOUND);
    }

    #[test]
    fn a_non_markdown_record_becomes_a_bullet_in_both_blocks() {
        // `MemoryDocument::new` always stamps `markdown`; the other branch is
        // upstream's legacy row shape, reachable only by building one by hand.
        let legacy = |scope: &str, content: &str| MemoryDocument {
            format: "text".to_owned(),
            ..document(scope, content)
        };
        let memories = [
            legacy("rules", "  一律用中文  "),
            legacy("facts", "喜欢苹果"),
        ];
        let prompt = build_coordinator_prompt(
            &CoordinationRequest {
                user_memories: &memories,
                ..CoordinationRequest::default()
            },
            at(0),
            Locale::Zh,
        );
        assert!(prompt.contains("<user_preferences>\n- 一律用中文\n</user_preferences>"));
        assert!(
            prompt.contains("<user_memory>\n- [memory] 喜欢苹果\n</user_memory>"),
            "the alias `facts` canonicalises to `memory` in the rendered tag: {prompt}",
        );
    }

    #[test]
    fn the_memory_block_says_none_when_there_is_nothing() {
        assert_eq!(
            memory_body(&[], Locale::Zh),
            t(Locale::Zh, keys::COORDINATOR_TASK_NONE),
        );
        assert_eq!(
            memory_body(&[document("rules", "只读偏好")], Locale::Zh),
            t(Locale::Zh, keys::COORDINATOR_TASK_NONE),
            "a directive document is not a memory record",
        );
    }

    #[test]
    fn at_most_twenty_memory_records_reach_the_model() {
        let memories: Vec<MemoryDocument> = (0..MAX_MEMORY_RECORDS + 5)
            .map(|index| document("memory", &format!("fact {index}")))
            .collect();
        let body = memory_body(&memories, Locale::Zh);
        assert_eq!(
            body.split(MEMORY_RECORD_SEPARATOR).count(),
            MAX_MEMORY_RECORDS,
        );
        assert!(body.contains("fact 19"));
        assert!(!body.contains("fact 20"));
    }
}
