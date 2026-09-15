//! [`FrontendMemoryService`] — the one door onto both memory documents.
//!
//! Ported from `server/src/conversation/frontend-memory-service.mjs`.
//!
//! # Why everything goes through here
//!
//! Two writers exist: the realtime `memory` tool, which the model calls
//! mid-conversation, and the session-end extractor, which a text model drives
//! after the connection closes. Both write through this service and neither
//! touches a Markdown file directly. That is the invariant the extractor's own
//! header calls out first — *"the extractor never writes either Markdown file
//! directly"* — and it is what makes the scope rules, the revision check and
//! the fragment-uniqueness rule impossible to route around.
//!
//! # The two scopes are separated by authority, not topic
//!
//! `user` is [`ScopeKind::Directive`](via_core::memory_scopes::ScopeKind):
//! standing instructions the user authored, injected as
//! `<user_preferences>` and followed. `memory` is
//! [`ScopeKind::Data`](via_core::memory_scopes::ScopeKind): durable facts,
//! injected as `<user_memory>` and used for understanding and answers, never as
//! instructions. Neither can authorize leaking internal structure, skipping a
//! permission check, or changing task and safety protocol — `PROMPT.md` is core
//! policy and outranks both. `ASSISTANT.md` is never written through this
//! service at all: it is not one of the two documents, and there is no code
//! path that could make it one.
//!
//! # `apply` is prepare-all-then-persist-all
//!
//! A cross-document reconciliation — move a directive out of `MEMORY.md` and
//! into `USER.md` — is one call with two changes. Every change is prepared
//! first, so a refusal on the second document means the first is never written
//! either. Upstream does not hold a file transaction across the two persists;
//! that is reproduced rather than "fixed", because taking two cross-process
//! locks in sequence introduces a deadlock the single-document path does not
//! have. See `docs/deviations/phase-4.md`.

use serde::{Deserialize, Serialize};
use via_core::memory_scopes::{ALL_SCOPE, canonical_scope, is_memory_document, memory_documents};

use crate::markdown_store::{
    EditRequest, MarkdownContextStore, MarkdownEdit, MemoryDocument, MemoryEditError,
};

/// One document's worth of change, as the caller submits it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemoryChange {
    /// `user` or `memory`, or one of the four legacy aliases.
    pub document: String,
    /// Exact replacements.
    pub edits: Vec<MarkdownEdit>,
    /// A Markdown block to append.
    pub append: String,
    /// The revision the caller last saw. Empty skips the check.
    pub expected_revision: String,
}

/// What [`FrontendMemoryService::apply`] wrote.
///
/// **External contract** — `frontend-memory-service.mjs:51-54`, reaching the
/// model as the `memory` tool's `{status, changed, documents}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplyOutcome {
    /// The sum of every change's `changed` count.
    pub changed: usize,
    /// The public record of each document, in the order the changes arrived.
    pub documents: Vec<MemoryDocument>,
}

/// Why a memory service call was refused before any store saw it.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MemoryServiceError {
    /// A scope that is neither a document nor `all`.
    ///
    /// Contract — `frontend-memory-service.mjs:10`.
    #[error("unsupported memory scope: {scope}")]
    UnsupportedScope {
        /// The alias-resolved scope that was rejected.
        scope: String,
    },
    /// `apply` was called with no changes.
    ///
    /// Contract — `frontend-memory-service.mjs:28`.
    #[error("at least one memory change is required")]
    NoChanges,
    /// A change named `all`, or named nothing at all.
    ///
    /// Contract — `frontend-memory-service.mjs:33`.
    #[error("each change requires a concrete document")]
    NoConcreteDocument,
    /// Two changes named the same document.
    ///
    /// Contract — `frontend-memory-service.mjs:34`. One change per document is
    /// what makes the second document's edits apply to a body the first change
    /// has not moved under them.
    #[error("duplicate memory document: {document}")]
    DuplicateDocument {
        /// The repeated document name.
        document: String,
    },
    /// The named document has no store behind it.
    ///
    /// Contract — `frontend-memory-service.mjs:37`.
    #[error("{document} memory document is unavailable")]
    DocumentUnavailable {
        /// The document that has no store.
        document: String,
    },
    /// A store refused the edit.
    #[error("{message}")]
    Edit {
        /// The stable code, where the failure has one.
        code: Option<&'static str>,
        /// Whether the `memory` tool answers by re-attaching the documents.
        retryable_read_again: bool,
        /// The store's own message.
        message: String,
    },
}

impl From<MemoryEditError> for MemoryServiceError {
    fn from(error: MemoryEditError) -> Self {
        Self::Edit {
            code: error.code(),
            retryable_read_again: error.is_retryable_read_again(),
            message: error.to_string(),
        }
    }
}

/// The `/api/health.frontendMemory` payload.
///
/// **External contract** — the catalogued */api/health.frontendMemory*:
/// `{ok, documents: {user: {...}, memory: {...}}}`. The document keys are
/// exactly the [`via_core::memory_scopes::MEMORY_SCOPES`] keys, in table order,
/// which `serde_json`'s `preserve_order` keeps.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryServiceHealth {
    /// `true` unless some configured document reported `ok: false`.
    pub ok: bool,
    /// Per-document health, keyed by scope.
    pub documents: serde_json::Map<String, serde_json::Value>,
}

/// Both memory documents behind one interface.
#[derive(Clone, Debug, Default)]
pub struct FrontendMemoryService {
    user: Option<MarkdownContextStore>,
    memory: Option<MarkdownContextStore>,
}

impl FrontendMemoryService {
    /// Build a service over the two stores. Either may be absent, which makes
    /// that document unavailable rather than making the service unusable.
    #[must_use]
    pub fn new(user: Option<MarkdownContextStore>, memory: Option<MarkdownContextStore>) -> Self {
        Self { user, memory }
    }

    /// The store for a canonical document name.
    #[must_use]
    fn store(&self, document: &str) -> Option<&MarkdownContextStore> {
        match document {
            via_core::memory_scopes::USER_SCOPE => self.user.as_ref(),
            via_core::memory_scopes::MEMORY_SCOPE => self.memory.as_ref(),
            _ => None,
        }
    }

    /// `canonicalScope(String(value || fallback))`, rejecting anything that is
    /// neither a document nor `all`.
    ///
    /// **External contract** — `frontend-memory-service.mjs:7-13`. The four
    /// legacy aliases (`profile`, `rules` → `user`; `facts`, `long_term` →
    /// `memory`) resolve here, which is what stops an upgrade losing memories
    /// written under the older spellings.
    ///
    /// # Errors
    ///
    /// [`MemoryServiceError::UnsupportedScope`].
    pub fn normalize_scope(value: Option<&str>) -> Result<String, MemoryServiceError> {
        let raw = value.filter(|value| !value.is_empty()).unwrap_or(ALL_SCOPE);
        let scope = canonical_scope(raw);
        if scope != ALL_SCOPE && !is_memory_document(&scope) {
            return Err(MemoryServiceError::UnsupportedScope { scope });
        }
        Ok(scope)
    }

    /// Every document, or just the one `scope` names.
    ///
    /// **External contract** — `frontend-memory-service.mjs:20-24`. `None`
    /// means "all", and an *empty* document contributes no entry at all, which
    /// is what makes the tool's `read` answer `not_found` on a fresh install.
    ///
    /// # Errors
    ///
    /// [`MemoryServiceError::UnsupportedScope`].
    pub fn list(
        &self,
        owner_id: &str,
        scope: Option<&str>,
    ) -> Result<Vec<MemoryDocument>, MemoryServiceError> {
        let target = Self::normalize_scope(scope)?;
        let scopes: Vec<&'static str> = if target == ALL_SCOPE {
            memory_documents()
        } else {
            memory_documents()
                .into_iter()
                .filter(|scope| *scope == target)
                .collect()
        };
        Ok(scopes
            .into_iter()
            .flat_map(|name| {
                self.store(name)
                    .map(|store| store.list(owner_id))
                    .unwrap_or_default()
            })
            .collect())
    }

    /// Apply one change per document, atomically across the preparation phase.
    ///
    /// **External contract** — `frontend-memory-service.mjs:26-55`. The order
    /// is: validate every change, prepare every change, then persist the ones
    /// that changed. A refusal anywhere in the first two phases writes nothing.
    ///
    /// # Errors
    ///
    /// [`MemoryServiceError::NoChanges`] for an empty list,
    /// [`MemoryServiceError::NoConcreteDocument`] for a change that names `all`
    /// or nothing, [`MemoryServiceError::DuplicateDocument`],
    /// [`MemoryServiceError::DocumentUnavailable`], or
    /// [`MemoryServiceError::Edit`] from a store.
    pub fn apply(
        &self,
        owner_id: &str,
        changes: &[MemoryChange],
    ) -> Result<ApplyOutcome, MemoryServiceError> {
        if changes.is_empty() {
            return Err(MemoryServiceError::NoChanges);
        }

        let mut seen: Vec<String> = Vec::new();
        let mut prepared = Vec::with_capacity(changes.len());
        for change in changes {
            let document = Self::normalize_scope(Some(change.document.as_str()))?;
            if document == ALL_SCOPE {
                return Err(MemoryServiceError::NoConcreteDocument);
            }
            if seen.iter().any(|entry| entry == &document) {
                return Err(MemoryServiceError::DuplicateDocument { document });
            }
            let store = self
                .store(&document)
                .ok_or_else(|| MemoryServiceError::DocumentUnavailable {
                    document: document.clone(),
                })?
                .clone();
            seen.push(document);
            let result = store.prepare_edit(
                owner_id,
                &EditRequest {
                    edits: change.edits.clone(),
                    append: change.append.clone(),
                    expected_revision: change.expected_revision.clone(),
                },
            )?;
            prepared.push((store, result));
        }

        let mut changed = 0usize;
        let mut documents = Vec::with_capacity(prepared.len());
        for (store, result) in prepared {
            if result.changed > 0 {
                store.persist(owner_id, &format!("{}\n", result.content))?;
            }
            changed += result.changed;
            documents.push(result.document);
        }
        Ok(ApplyOutcome { changed, documents })
    }

    /// The `/api/health.frontendMemory` payload.
    ///
    /// **External contract** — `frontend-memory-service.mjs:57-69`. An absent
    /// store reports `{ok: true, configured: false, warning: null}` rather than
    /// failing health: a Gateway with no data directory is unconfigured, not
    /// broken.
    #[must_use]
    pub fn health(&self) -> MemoryServiceHealth {
        let mut documents = serde_json::Map::new();
        let mut ok = true;
        for scope in memory_documents() {
            let health = self.store(scope).map_or_else(
                || crate::markdown_store::DocumentHealth {
                    ok: true,
                    configured: false,
                    warning: None,
                },
                MarkdownContextStore::health,
            );
            ok = ok && health.ok;
            documents.insert(
                scope.to_owned(),
                serde_json::to_value(&health).unwrap_or(serde_json::Value::Null),
            );
        }
        MemoryServiceHealth { ok, documents }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_scope_resolves_every_legacy_alias() {
        for (alias, expected) in [
            ("profile", "user"),
            ("rules", "user"),
            ("facts", "memory"),
            ("long_term", "memory"),
            ("USER", "user"),
            (" memory ", "memory"),
        ] {
            assert_eq!(
                FrontendMemoryService::normalize_scope(Some(alias)).expect("known alias"),
                expected
            );
        }
    }

    #[test]
    fn an_absent_or_empty_scope_is_the_all_sentinel() {
        assert_eq!(
            FrontendMemoryService::normalize_scope(None).expect("all"),
            ALL_SCOPE
        );
        assert_eq!(
            FrontendMemoryService::normalize_scope(Some("")).expect("all"),
            ALL_SCOPE
        );
    }

    #[test]
    fn an_unknown_scope_is_refused_by_name() {
        assert_eq!(
            FrontendMemoryService::normalize_scope(Some("assistant")),
            Err(MemoryServiceError::UnsupportedScope {
                scope: "assistant".to_owned()
            })
        );
    }

    #[test]
    fn health_reports_unconfigured_documents_as_ok() {
        let health = FrontendMemoryService::default().health();
        assert!(health.ok);
        assert_eq!(
            health.documents.keys().collect::<Vec<_>>(),
            vec!["user", "memory"]
        );
        assert_eq!(
            health.documents["user"]["configured"],
            serde_json::json!(false)
        );
    }
}
