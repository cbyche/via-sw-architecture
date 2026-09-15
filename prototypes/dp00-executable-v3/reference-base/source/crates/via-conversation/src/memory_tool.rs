//! The `memory` tool — one flat interface, one atomic operation per call.
//!
//! Ported from `server/src/voice/tools/tool-call-handler.mjs:989-1066`.
//!
//! # One call, one operation
//!
//! `read`, `append` or `replace`. There is no batch form and no transaction
//! across calls: the tool description tells the model that several persistent
//! changes in one sentence are several calls, and the Gateway merges their
//! follow-up response so the user hears one reply rather than three.
//!
//! # `replace` fails safely
//!
//! It locates a **unique** source fragment. Missing is
//! [`EDIT_NOT_FOUND`](crate::markdown_store::EDIT_NOT_FOUND_CODE), ambiguous is
//! [`AMBIGUOUS_EDIT`](crate::markdown_store::AMBIGUOUS_EDIT_CODE), and both come
//! back **retryable with the current documents re-attached** so the model can
//! quote a fragment that actually exists rather than guessing again. It never
//! edits the first of several matches, because the model cannot see which one
//! it got and a wrong replacement in `USER.md` changes how the assistant
//! behaves from then on.
//!
//! # The nine failure codes
//!
//! Every one of them reaches the model in `error_code`, and all nine are
//! catalogued:
//!
//! | Code | When |
//! | --- | --- |
//! | [`MEMORY_UNAVAILABLE`] | no memory service is configured |
//! | [`INVALID_MEMORY_ACTION`] | the action is not one of the three |
//! | [`INVALID_MEMORY_DOCUMENT`] | an unknown document, or none on a write |
//! | [`INVALID_MEMORY_EDIT`] | `append` with no content, `replace` with no fragment |
//! | [`SENSITIVE_MEMORY`] | the proposal tripped the credential gate |
//! | `stale_document` | the revision moved under the model |
//! | `edit_not_found` | the fragment is not there |
//! | `ambiguous_edit` | the fragment is there more than once |
//! | [`MEMORY_WRITE_FAILED`] | anything else, including the size cap |
//!
//! The three middle codes are the store's own
//! ([`crate::markdown_store::MemoryEditError`]) and are forwarded verbatim
//! rather than flattened, because the model's next move differs: re-read and
//! retry, versus give up and tell the user.

use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use via_core::memory_scopes::{ALL_SCOPE, canonical_scope, is_memory_document};
use via_i18n::{Locale, keys, t};

use crate::markdown_store::{MarkdownEdit, MemoryDocument};
use crate::memory_service::{FrontendMemoryService, MemoryChange, MemoryServiceError};
use crate::tool::{ToolFailure, is_sensitive};

/// The tool name the realtime model calls.
///
/// **External contract** — `server/src/voice/frontend-tools.mjs:13`
/// (`MEMORY_TOOL_NAME`), referenced by name in `PROMPT.md`.
pub const MEMORY_TOOL_NAME: &str = "memory";

/// The three actions, in schema order.
///
/// **External contract** — the catalogued *memory.parameters.action*.
pub const MEMORY_ACTIONS: [&str; 3] = ["read", "append", "replace"];

/// No memory service is configured.
pub const MEMORY_UNAVAILABLE: &str = "memory_unavailable";
/// The action was not one of [`MEMORY_ACTIONS`].
pub const INVALID_MEMORY_ACTION: &str = "invalid_memory_action";
/// The document was unknown, or a write named none.
pub const INVALID_MEMORY_DOCUMENT: &str = "invalid_memory_document";
/// `append` carried no content, or `replace` carried no fragment.
pub const INVALID_MEMORY_EDIT: &str = "invalid_memory_edit";
/// The proposal tripped the credential gate.
pub const SENSITIVE_MEMORY: &str = "sensitive_memory";
/// The write failed for a reason the model cannot fix by re-reading.
pub const MEMORY_WRITE_FAILED: &str = "memory_write_failed";

/// Every code this tool can emit, in the order the handler tests them.
///
/// **External contract** — the catalogued *memory tool failure codes* and the
/// `tool-call-handler error code inventory`.
pub const MEMORY_FAILURE_CODES: [&str; 9] = [
    MEMORY_UNAVAILABLE,
    INVALID_MEMORY_ACTION,
    INVALID_MEMORY_DOCUMENT,
    INVALID_MEMORY_EDIT,
    SENSITIVE_MEMORY,
    crate::markdown_store::STALE_DOCUMENT_CODE,
    crate::markdown_store::EDIT_NOT_FOUND_CODE,
    crate::markdown_store::AMBIGUOUS_EDIT_CODE,
    MEMORY_WRITE_FAILED,
];

/// One `memory` tool call, as the model sent it.
///
/// `new_text` is an [`Option`] on purpose: the contract is that `replace`
/// requires the **key to be present**, so `""` is a valid deletion and an
/// omitted key is not. Collapsing the two would make "forget this" and "I
/// forgot to fill this in" the same request.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemoryToolRequest {
    /// `read`, `append` or `replace`.
    pub action: String,
    /// `user`, `memory`, `all`, or one of the legacy aliases.
    pub document: Option<String>,
    /// The fragment to locate, for `replace`.
    pub old_text: Option<String>,
    /// The replacement, for `replace`. `Some("")` deletes.
    pub new_text: Option<String>,
    /// The Markdown block, for `append`.
    pub content: Option<String>,
}

impl MemoryToolRequest {
    /// A `read` of every document.
    #[must_use]
    pub fn read(document: Option<&str>) -> Self {
        Self {
            action: "read".to_owned(),
            document: document.map(ToOwned::to_owned),
            ..Self::default()
        }
    }

    /// An `append` to one document.
    #[must_use]
    pub fn append(document: &str, content: &str) -> Self {
        Self {
            action: "append".to_owned(),
            document: Some(document.to_owned()),
            content: Some(content.to_owned()),
            ..Self::default()
        }
    }

    /// A `replace` in one document. An empty `new_text` deletes.
    #[must_use]
    pub fn replace(document: &str, old_text: &str, new_text: &str) -> Self {
        Self {
            action: "replace".to_owned(),
            document: Some(document.to_owned()),
            old_text: Some(old_text.to_owned()),
            new_text: Some(new_text.to_owned()),
            ..Self::default()
        }
    }
}

/// What the model is told.
#[derive(Clone, Debug, PartialEq)]
pub enum MemoryToolOutcome {
    /// A `read`: `{status: 'ok'|'not_found', count, documents}`.
    Read {
        /// `ok` when at least one document had content.
        status: &'static str,
        /// How many documents came back.
        count: usize,
        /// The documents.
        documents: Vec<MemoryDocument>,
    },
    /// A write: `{status: 'updated'|'unchanged', changed, documents}`.
    Applied {
        /// `updated` when anything changed, otherwise `unchanged`.
        status: &'static str,
        /// How many operations changed something.
        changed: usize,
        /// The documents after the write.
        documents: Vec<MemoryDocument>,
    },
    /// A refusal.
    Failed(ToolFailure),
}

impl MemoryToolOutcome {
    /// Whether a document actually changed, which is what tells the Gateway to
    /// re-inject the memory blocks into the live session.
    ///
    /// **External contract** — `tool-call-handler.mjs:1040`
    /// (`if (result.changed) this.notifyMemoryChanged()`). Returned rather than
    /// signalled through a callback: the session belongs to `via-voice`, and
    /// this crate has no business holding a handle to it.
    #[must_use]
    pub fn changed(&self) -> usize {
        match self {
            Self::Applied { changed, .. } => *changed,
            Self::Read { .. } | Self::Failed(_) => 0,
        }
    }

    /// The stable code, when this is a refusal.
    #[must_use]
    pub fn error_code(&self) -> Option<&str> {
        match self {
            Self::Failed(failure) => Some(&failure.error_code),
            Self::Read { .. } | Self::Applied { .. } => None,
        }
    }

    /// The wire object, with the catalogued field order.
    #[must_use]
    pub fn to_value(&self) -> Value {
        match self {
            Self::Read {
                status,
                count,
                documents,
            } => {
                let mut out = Map::new();
                out.insert("status".into(), Value::from(*status));
                out.insert("count".into(), Value::from(*count));
                out.insert("documents".into(), documents_value(documents));
                Value::Object(out)
            }
            Self::Applied {
                status,
                changed,
                documents,
            } => {
                let mut out = Map::new();
                out.insert("status".into(), Value::from(*status));
                out.insert("changed".into(), Value::from(*changed));
                out.insert("documents".into(), documents_value(documents));
                Value::Object(out)
            }
            Self::Failed(failure) => failure.to_value(),
        }
    }
}

impl Serialize for MemoryToolOutcome {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_value().serialize(serializer)
    }
}

fn documents_value(documents: &[MemoryDocument]) -> Value {
    serde_json::to_value(documents).unwrap_or_else(|_| Value::Array(Vec::new()))
}

/// The `memory` tool.
#[derive(Clone, Debug)]
pub struct MemoryTool {
    service: Option<FrontendMemoryService>,
    locale: Locale,
}

impl MemoryTool {
    /// Build the tool. `None` makes every call [`MEMORY_UNAVAILABLE`], which is
    /// what a Gateway with no data directory gets — the model is told the
    /// feature is off rather than being allowed to claim it remembered.
    #[must_use]
    pub fn new(service: Option<FrontendMemoryService>, locale: Locale) -> Self {
        Self { service, locale }
    }

    /// Handle one call.
    ///
    /// **External contract** — `tool-call-handler.mjs:989-1066`. The branch
    /// order is exact and it matters: the document check runs before the edit
    /// check, so `{action: 'append'}` with neither a document nor content
    /// reports [`INVALID_MEMORY_DOCUMENT`], not [`INVALID_MEMORY_EDIT`].
    #[must_use]
    pub fn handle(&self, owner_id: &str, request: &MemoryToolRequest) -> MemoryToolOutcome {
        let action = request.action.trim().to_lowercase();
        let old_text = request.old_text.clone().unwrap_or_default();
        let new_text = request.new_text.clone().unwrap_or_default();
        let has_new_text = request.new_text.is_some();
        let content = crate::text::trim(request.content.as_deref().unwrap_or_default()).to_owned();

        let Some(service) = self.service.as_ref() else {
            return self.fail(MEMORY_UNAVAILABLE, keys::MEMORY_ERROR_UNAVAILABLE);
        };
        if !MEMORY_ACTIONS.contains(&action.as_str()) {
            return self.fail(INVALID_MEMORY_ACTION, keys::MEMORY_ERROR_INVALID_ACTION);
        }

        // `String(args.document || (action === 'read' ? 'all' : ''))`: a read
        // with no document means every document; a write with no document
        // means nothing at all, and is refused below.
        let raw_document = request
            .document
            .as_deref()
            .filter(|value| !value.is_empty())
            .unwrap_or(if action == "read" { ALL_SCOPE } else { "" });
        let document = canonical_scope(raw_document);

        if action == "read" {
            let scope = if document == ALL_SCOPE {
                None
            } else {
                Some(document.as_str())
            };
            if scope.is_some_and(|scope| !is_memory_document(scope)) {
                return self.fail(
                    INVALID_MEMORY_DOCUMENT,
                    keys::MEMORY_ERROR_INVALID_DOCUMENT_READ,
                );
            }
            let documents = service.list(owner_id, scope).unwrap_or_default();
            return MemoryToolOutcome::Read {
                status: if documents.is_empty() {
                    "not_found"
                } else {
                    "ok"
                },
                count: documents.len(),
                documents,
            };
        }

        if !is_memory_document(&document) {
            return self.fail(
                INVALID_MEMORY_DOCUMENT,
                keys::MEMORY_ERROR_INVALID_DOCUMENT_WRITE,
            );
        }
        if action == "append" && content.is_empty() {
            return self.fail(INVALID_MEMORY_EDIT, keys::MEMORY_ERROR_APPEND_NEEDS_CONTENT);
        }
        if action == "replace" && (old_text.is_empty() || !has_new_text) {
            return self.fail(INVALID_MEMORY_EDIT, keys::MEMORY_ERROR_REPLACE_NEEDS_TEXTS);
        }

        let proposed = if action == "append" {
            content.as_str()
        } else {
            new_text.as_str()
        };
        if is_sensitive(proposed) {
            return MemoryToolOutcome::Failed(
                ToolFailure::new(
                    SENSITIVE_MEMORY,
                    t(self.locale, keys::MEMORY_ERROR_SENSITIVE),
                )
                .rejected(),
            );
        }

        let change = MemoryChange {
            document,
            edits: if action == "replace" {
                vec![MarkdownEdit { old_text, new_text }]
            } else {
                Vec::new()
            },
            append: if action == "append" {
                content
            } else {
                String::new()
            },
            expected_revision: String::new(),
        };

        match service.apply(owner_id, std::slice::from_ref(&change)) {
            Ok(outcome) => MemoryToolOutcome::Applied {
                status: if outcome.changed > 0 {
                    "updated"
                } else {
                    "unchanged"
                },
                changed: outcome.changed,
                documents: outcome.documents,
            },
            Err(MemoryServiceError::Edit {
                code: Some(code),
                retryable_read_again: true,
                ..
            }) => MemoryToolOutcome::Failed(
                ToolFailure::new(code, t(self.locale, keys::MEMORY_ERROR_STALE_DOCUMENT))
                    .retryable()
                    // Re-attaching the documents is what turns a refusal into a
                    // recoverable turn: the model gets the current bytes and
                    // the current revision in the same breath.
                    .detail(
                        "documents",
                        documents_value(&service.list(owner_id, None).unwrap_or_default()),
                    ),
            ),
            Err(_) => MemoryToolOutcome::Failed(
                ToolFailure::new(
                    MEMORY_WRITE_FAILED,
                    t(self.locale, keys::MEMORY_ERROR_WRITE_FAILED),
                )
                .retryable(),
            ),
        }
    }

    fn fail(&self, code: &'static str, key: via_i18n::Key) -> MemoryToolOutcome {
        MemoryToolOutcome::Failed(ToolFailure::new(code, t(self.locale, key)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unconfigured_tool_refuses_every_action() {
        let tool = MemoryTool::new(None, Locale::En);
        for request in [
            MemoryToolRequest::read(None),
            MemoryToolRequest::append("user", "- x"),
            MemoryToolRequest::replace("memory", "a", "b"),
            MemoryToolRequest::default(),
        ] {
            assert_eq!(
                tool.handle("owner", &request).error_code(),
                Some(MEMORY_UNAVAILABLE)
            );
        }
    }

    #[test]
    fn the_nine_codes_are_distinct() {
        let mut seen = MEMORY_FAILURE_CODES.to_vec();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), MEMORY_FAILURE_CODES.len());
    }
}
