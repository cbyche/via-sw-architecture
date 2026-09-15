//! [`MarkdownContextStore`] — one Markdown document, edited by exact fragment.
//!
//! Ported from `server/src/conversation/markdown-context-store.mjs`. Two
//! instances exist per install: `USER.md` (scope `user`) and `MEMORY.md`
//! (scope `memory`). The store knows nothing about the difference — the
//! *authority* split lives in [`via_core::memory_scopes`] and in how
//! [`crate::context`] renders each one — and that separation is deliberate:
//! a store that knew which document was directive would be a store that could
//! be talked into moving content between them.
//!
//! # The three properties that make `replace` safe
//!
//! 1. **A fragment must be unique.** `old_text` occurring zero times is
//!    [`MemoryEditError::EditNotFound`]; occurring more than once is
//!    [`MemoryEditError::AmbiguousEdit`]. It is never "the first match wins",
//!    because the model cannot see which one it got.
//! 2. **A revision is checked before anything is written.** The `revision` the
//!    model round-trips is `sha256(raw)[..16]`; a mismatch is
//!    [`MemoryEditError::StaleDocument`] and no bytes move.
//! 3. **The whole edit is prepared before any of it is persisted.**
//!    [`MarkdownContextStore::prepare_edit`] is pure; only
//!    [`MarkdownContextStore::persist`] touches the disk, and
//!    [`MarkdownContextStore::edit`] runs both inside one
//!    [`via_store::with_file_transaction`] so a second Gateway sharing the same
//!    profile directory cannot interleave.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use via_core::paths::owner_sharded_path;
use via_i18n::{Locale, format as i18n_format, keys, t};

use crate::text::{
    bounded_code_points, clean, code_point_len, count_occurrences, is_js_whitespace, normalize,
    trim, trim_end,
};

/// Hex characters of the SHA-256 digest used as a document revision.
///
/// **External contract** — `markdown-context-store.mjs:20-22`
/// (`createHash('sha256').update(value).digest('hex').slice(0, 16)`). The value
/// is model-visible: it is the `revision` attribute on `<user_preferences>` and
/// `<user_memory>`, and the model round-trips it as `expectedRevision`.
pub const REVISION_HEX_LENGTH: usize = 16;

/// Cap on how many exact edits one call may carry.
///
/// **External contract** — `markdown-context-store.mjs:14` (`MAX_EDIT_ITEMS`).
/// Surplus edits are **dropped silently**, not rejected — `edits.slice(0, 20)`.
pub const MAX_EDIT_ITEMS: usize = 20;

/// The default character cap on one document.
///
/// **External contract** — `markdown-context-store.mjs:70` (`maxChars = 8000`).
/// Counted in code points, as everything else in this crate's scope is.
pub const DEFAULT_MAX_CHARS: usize = 8000;

/// The `format` field every document carries.
///
/// **External contract** — `markdown-context-store.mjs:29`. It is a promise to
/// the model that the body is ordinary human Markdown with no envelope, which
/// is what lets `replace` quote a fragment verbatim.
pub const DOCUMENT_FORMAT: &str = "markdown";

/// Suffix appended to a scope to form a document id: `<scope>_document`.
///
/// **External contract** — `markdown-context-store.mjs:26`.
pub const DOCUMENT_ID_SUFFIX: &str = "_document";

/// `sha256(value)` truncated to [`REVISION_HEX_LENGTH`] hex characters.
///
/// Also the owner-shard name (`markdown-context-store.mjs:86`), which is why
/// [`via_core::paths::owner_shard`] produces the same 16 characters.
#[must_use]
pub fn digest(value: &str) -> String {
    let full = hex::encode(Sha256::digest(value.as_bytes()));
    full[..REVISION_HEX_LENGTH].to_owned()
}

/// One memory document as the model and `/api/health` see it.
///
/// **External contract** — the catalogued *memory document (publicDocument)*,
/// `markdown-context-store.mjs:24-33`. Field order is the serialization order
/// below and reaches the model verbatim inside the `memory` tool's `read`
/// output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryDocument {
    /// `<scope>_document`.
    pub id: String,
    /// `user` or `memory`.
    pub scope: String,
    /// The document body, already truncated if it was over the cap.
    pub content: String,
    /// Always [`DOCUMENT_FORMAT`].
    pub format: String,
    /// `sha256(raw)[..16]` — of the **untruncated** bytes, so a document the
    /// model only saw a prefix of still round-trips a revision that identifies
    /// the whole file.
    pub revision: String,
    /// Always `true`; the model is told this document is writable.
    pub editable: bool,
}

impl MemoryDocument {
    /// Build the public record for `scope`, deriving the revision from
    /// `revision_source`.
    ///
    /// Upstream's `publicDocument(scope, content, revisionSource = content)`.
    /// The two arguments differ in exactly one place —
    /// [`MarkdownContextStore::list`] passes the raw bytes with the truncated
    /// body — and that is the whole reason the parameter exists.
    #[must_use]
    pub fn new(scope: &str, content: impl Into<String>, revision_source: &str) -> Self {
        Self {
            id: format!("{scope}{DOCUMENT_ID_SUFFIX}"),
            scope: scope.to_owned(),
            content: content.into(),
            format: DOCUMENT_FORMAT.to_owned(),
            revision: digest(revision_source),
            editable: true,
        }
    }
}

/// A diagnostic raised by a memory document, as `/api/health` shows it.
///
/// **External contract** — `markdown-context-store.mjs:204-206`:
/// `{message, at}`. Unlike [`via_store::StoreWarning`] there is no
/// `quarantinePath`, because a Markdown document is never quarantined — an
/// unreadable one is reported and the store keeps serving an empty body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentWarning {
    /// The rendered sentence.
    pub message: String,
    /// Epoch milliseconds.
    pub at: i64,
}

/// A memory document's health triple.
///
/// **External contract** — the catalogued */api/health.frontendMemory*:
/// `{ok, configured, warning}`, in that order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentHealth {
    /// `true` while no warning stands.
    pub ok: bool,
    /// Whether a file path is configured at all.
    pub configured: bool,
    /// The most recent warning, or `null`.
    pub warning: Option<DocumentWarning>,
}

/// Why an edit was refused.
///
/// **External contract** — the catalogued *MarkdownContextStore edit error
/// codes*. [`Self::code`] is the string the `memory` tool puts in
/// `error_code`, and the first three are the ones the tool re-attaches the
/// current documents to; everything else becomes `memory_write_failed`.
#[derive(Debug, thiserror::Error)]
pub enum MemoryEditError {
    /// `expectedRevision` did not match the document on disk.
    #[error("{0}")]
    StaleDocument(&'static str),
    /// An edit arrived with no `old_text`.
    #[error("{0}")]
    InvalidEdit(&'static str),
    /// `old_text` occurs more than once.
    #[error("{0}")]
    AmbiguousEdit(&'static str),
    /// `old_text` does not occur.
    #[error("{0}")]
    EditNotFound(&'static str),
    /// The result would exceed the document's character cap.
    #[error("{message}")]
    DocumentTooLarge {
        /// The rendered sentence, naming the file and the cap.
        message: String,
    },
    /// The store has no configured path, so there is nothing to write to.
    ///
    /// Contract — `markdown-context-store.mjs:194`
    /// (`'memory document is unavailable'`).
    #[error("memory document is unavailable")]
    Unavailable,
    /// The write, or the transaction lock, failed.
    #[error("{0}")]
    Io(String),
}

impl MemoryEditError {
    /// The stable code, or `None` for the two failures upstream throws
    /// untyped.
    ///
    /// **External contract** — catalogued *MarkdownContextStore edit error
    /// codes*: `stale_document | invalid_edit | ambiguous_edit |
    /// edit_not_found | document_too_large`.
    #[must_use]
    pub fn code(&self) -> Option<&'static str> {
        match self {
            Self::StaleDocument(_) => Some(STALE_DOCUMENT_CODE),
            Self::InvalidEdit(_) => Some(INVALID_EDIT_CODE),
            Self::AmbiguousEdit(_) => Some(AMBIGUOUS_EDIT_CODE),
            Self::EditNotFound(_) => Some(EDIT_NOT_FOUND_CODE),
            Self::DocumentTooLarge { .. } => Some(DOCUMENT_TOO_LARGE_CODE),
            Self::Unavailable | Self::Io(_) => None,
        }
    }

    /// Whether the `memory` tool answers this one by re-attaching the current
    /// documents and inviting a retry.
    ///
    /// **External contract** — `tool-call-handler.mjs:1047`: exactly
    /// `stale_document`, `edit_not_found` and `ambiguous_edit`.
    #[must_use]
    pub fn is_retryable_read_again(&self) -> bool {
        matches!(
            self,
            Self::StaleDocument(_) | Self::EditNotFound(_) | Self::AmbiguousEdit(_)
        )
    }
}

/// `stale_document`.
pub const STALE_DOCUMENT_CODE: &str = "stale_document";
/// `invalid_edit`.
pub const INVALID_EDIT_CODE: &str = "invalid_edit";
/// `ambiguous_edit`.
pub const AMBIGUOUS_EDIT_CODE: &str = "ambiguous_edit";
/// `edit_not_found`.
pub const EDIT_NOT_FOUND_CODE: &str = "edit_not_found";
/// `document_too_large`.
pub const DOCUMENT_TOO_LARGE_CODE: &str = "document_too_large";

/// The owner whose copy of a document lives at the unsharded path.
///
/// **External contract** — `markdown-context-store.mjs:68`
/// (`personalOwnerId = 'user_personal'`), which is also
/// `Config::personal_owner_id`'s default (`shared/runtime-environment.mjs:533`,
/// `VIA_PERSONAL_OWNER_ID`). The store carries its own default because
/// upstream's constructor does; a Gateway passes the configured value.
pub const DEFAULT_PERSONAL_OWNER_ID: &str = "user_personal";

/// One exact replacement.
///
/// **External contract** — the `memory` tool's `old_text` / `new_text` and the
/// extractor's `edits[]`. An empty `new_text` is a deletion, which is why the
/// tool checks for the *presence* of the key rather than a non-empty value.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownEdit {
    /// The fragment to locate. Must occur exactly once.
    pub old_text: String,
    /// What replaces it. Empty deletes.
    pub new_text: String,
}

/// One document's worth of change: exact edits, then an append.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditRequest {
    /// Exact replacements, applied in order. Only the first
    /// [`MAX_EDIT_ITEMS`] are considered.
    pub edits: Vec<MarkdownEdit>,
    /// A Markdown block appended after the edits.
    pub append: String,
    /// The revision the caller last saw. Empty skips the check, which is what
    /// the realtime `memory` tool does.
    pub expected_revision: String,
}

/// The outcome of [`MarkdownContextStore::prepare_edit`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedEdit {
    /// How many operations changed something. `0` means nothing is written.
    pub changed: usize,
    /// The full document body to persist, **without** its trailing newline.
    pub content: String,
    /// The public record the caller reports.
    pub document: MemoryDocument,
}

/// The outcome of [`MarkdownContextStore::edit`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditOutcome {
    /// How many operations changed something.
    pub changed: usize,
    /// The public record after the edit.
    pub document: MemoryDocument,
}

/// Where warnings go. Defaults to a no-op.
pub type WarningSink = Arc<dyn Fn(&DocumentWarning) + Send + Sync>;

/// Epoch-millisecond clock, injectable so a warning's `at` is deterministic.
pub type NowFn = Arc<dyn Fn() -> i64 + Send + Sync>;

/// One Markdown document, per owner.
///
/// Cheap to clone; every clone shares the same warning state.
#[derive(Clone)]
pub struct MarkdownContextStore {
    inner: Arc<Inner>,
}

struct Inner {
    file_path: Option<PathBuf>,
    scope: String,
    personal_owner_id: String,
    max_chars: usize,
    template: String,
    locale: Locale,
    now: NowFn,
    on_warning: WarningSink,
    warning: Mutex<Option<DocumentWarning>>,
}

impl std::fmt::Debug for MarkdownContextStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MarkdownContextStore")
            .field("file_path", &self.inner.file_path)
            .field("scope", &self.inner.scope)
            .field("max_chars", &self.inner.max_chars)
            .field("health", &self.health())
            .finish()
    }
}

/// Builder for [`MarkdownContextStore`].
pub struct MarkdownContextStoreBuilder {
    file_path: Option<PathBuf>,
    scope: String,
    personal_owner_id: String,
    max_chars: usize,
    template: String,
    locale: Locale,
    now: Option<NowFn>,
    on_warning: Option<WarningSink>,
}

impl std::fmt::Debug for MarkdownContextStoreBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MarkdownContextStoreBuilder")
            .field("file_path", &self.file_path)
            .field("scope", &self.scope)
            .finish_non_exhaustive()
    }
}

impl Default for MarkdownContextStoreBuilder {
    fn default() -> Self {
        Self {
            file_path: None,
            scope: String::new(),
            personal_owner_id: DEFAULT_PERSONAL_OWNER_ID.to_owned(),
            max_chars: DEFAULT_MAX_CHARS,
            template: String::new(),
            locale: Locale::default(),
            now: None,
            on_warning: None,
        }
    }
}

impl MarkdownContextStoreBuilder {
    /// The unsharded document path. `None` makes every read empty and every
    /// persist [`MemoryEditError::Unavailable`].
    #[must_use]
    pub fn file_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// `user` or `memory`.
    #[must_use]
    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = scope.into();
        self
    }

    /// The owner whose copy lives at the unsharded path.
    #[must_use]
    pub fn personal_owner_id(mut self, owner: impl Into<String>) -> Self {
        self.personal_owner_id = owner.into();
        self
    }

    /// The character cap. Defaults to [`DEFAULT_MAX_CHARS`].
    #[must_use]
    pub fn max_chars(mut self, max_chars: usize) -> Self {
        self.max_chars = max_chars;
        self
    }

    /// What an absent document is treated as while preparing an edit.
    #[must_use]
    pub fn template(mut self, template: &str) -> Self {
        self.template = trim_end(&normalize(template)).to_owned();
        self
    }

    /// Which locale the warnings are rendered in.
    #[must_use]
    pub fn locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// Override the clock.
    #[must_use]
    pub fn now(mut self, now: NowFn) -> Self {
        self.now = Some(now);
        self
    }

    /// Where warnings go.
    #[must_use]
    pub fn on_warning(mut self, on_warning: WarningSink) -> Self {
        self.on_warning = Some(on_warning);
        self
    }

    /// Finish the store.
    #[must_use]
    pub fn build(self) -> MarkdownContextStore {
        MarkdownContextStore {
            inner: Arc::new(Inner {
                file_path: self.file_path,
                scope: self.scope,
                personal_owner_id: self.personal_owner_id,
                max_chars: self.max_chars,
                template: self.template,
                locale: self.locale,
                now: self.now.unwrap_or_else(|| Arc::new(system_now_ms)),
                on_warning: self
                    .on_warning
                    .unwrap_or_else(|| Arc::new(|_: &DocumentWarning| {})),
                warning: Mutex::new(None),
            }),
        }
    }
}

fn system_now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

impl MarkdownContextStore {
    /// Start building a store.
    #[must_use]
    pub fn builder() -> MarkdownContextStoreBuilder {
        MarkdownContextStoreBuilder::default()
    }

    /// This store's scope: `user` or `memory`.
    #[must_use]
    pub fn scope(&self) -> &str {
        &self.inner.scope
    }

    /// The character cap.
    #[must_use]
    pub fn max_chars(&self) -> usize {
        self.inner.max_chars
    }

    /// Where `owner_id`'s copy of the document lives.
    ///
    /// **External contract** — `markdown-context-store.mjs:82-87` and the
    /// catalogued *non-personal owner memory sharding*: the personal owner uses
    /// the unsharded path, everyone else gets
    /// `<dir>/users/<sha256(owner)[..16]>/<basename>`. An empty owner id is the
    /// personal owner (`String(ownerId || this.personalOwnerId)`).
    #[must_use]
    pub fn path_for(&self, owner_id: &str) -> Option<PathBuf> {
        let path = self.inner.file_path.as_deref()?;
        let owner = if owner_id.is_empty() {
            self.inner.personal_owner_id.as_str()
        } else {
            owner_id
        };
        if owner == self.inner.personal_owner_id {
            return Some(path.to_path_buf());
        }
        Some(owner_sharded_path(path, owner))
    }

    /// The document body as the model should see it, truncated with the
    /// catalogued marker when it is over the cap.
    #[must_use]
    pub fn read(&self, owner_id: &str) -> String {
        let content = self.read_raw(owner_id);
        self.truncate(&content)
    }

    /// The document body exactly as it is on disk, normalized and trimmed.
    ///
    /// An absent file is `""` and raises no warning — that is the first run.
    /// Any other read failure raises `memory.read_failed` and still answers
    /// `""`, because a voice session that cannot read a memory file must still
    /// be able to talk.
    #[must_use]
    pub fn read_raw(&self, owner_id: &str) -> String {
        let Some(path) = self.path_for(owner_id) else {
            return String::new();
        };
        match fs::read_to_string(&path) {
            Ok(raw) => {
                *self.warning_slot() = None;
                trim(&normalize(&raw)).to_owned()
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
            Err(error) => {
                let name = file_name(&path);
                self.warn(i18n_format(
                    self.inner.locale,
                    keys::MEMORY_READ_FAILED,
                    &[("name", &name), ("detail", &error.to_string())],
                ));
                String::new()
            }
        }
    }

    /// The public record for `owner_id`, or an empty list when the document is
    /// empty.
    ///
    /// The revision is computed from the **raw** bytes even when `content` was
    /// truncated, so a model that read a prefix still round-trips a revision
    /// that identifies the whole file — and therefore still fails the staleness
    /// check if someone else edited the tail.
    #[must_use]
    pub fn list(&self, owner_id: &str) -> Vec<MemoryDocument> {
        let raw = self.read_raw(owner_id);
        if raw.is_empty() {
            return Vec::new();
        }
        let content = self.truncate(&raw);
        vec![MemoryDocument::new(&self.inner.scope, content, &raw)]
    }

    /// Apply `request` inside one cross-process transaction.
    ///
    /// # Errors
    ///
    /// Any [`MemoryEditError`]. Nothing is written unless the whole request
    /// prepared cleanly.
    pub fn edit(
        &self,
        owner_id: &str,
        request: &EditRequest,
    ) -> Result<EditOutcome, MemoryEditError> {
        let path = self.path_for(owner_id);
        via_store::with_file_transaction(path.as_deref(), via_store::LockOptions::default(), || {
            let prepared = self.prepare_edit(owner_id, request)?;
            if prepared.changed == 0 {
                return Ok(EditOutcome {
                    changed: 0,
                    document: prepared.document,
                });
            }
            self.persist(owner_id, &format!("{}\n", prepared.content))?;
            Ok(EditOutcome {
                changed: prepared.changed,
                document: prepared.document,
            })
        })
        .map_err(|error| MemoryEditError::Io(error.to_string()))?
    }

    /// Compute what `request` would produce, without touching the disk.
    ///
    /// **External contract** — `markdown-context-store.mjs:134-190`. The order
    /// is exact: revision check, then the edits in order, then the append, then
    /// [`normalize_markdown`], then the size cap. Each step can only refuse.
    ///
    /// # Errors
    ///
    /// Any [`MemoryEditError`] except [`MemoryEditError::Unavailable`] and
    /// [`MemoryEditError::Io`], which only a write can raise.
    pub fn prepare_edit(
        &self,
        owner_id: &str,
        request: &EditRequest,
    ) -> Result<PreparedEdit, MemoryEditError> {
        let read = self.read_raw(owner_id);
        let current = if read.is_empty() {
            self.inner.template.clone()
        } else {
            read
        };

        if !request.expected_revision.is_empty() && request.expected_revision != digest(&current) {
            return Err(MemoryEditError::StaleDocument(t(
                self.inner.locale,
                keys::MEMORY_STALE_DOCUMENT_CODE,
            )));
        }

        let mut next = current.clone();
        let mut changed = 0usize;
        for operation in request.edits.iter().take(MAX_EDIT_ITEMS) {
            let old_text = normalize(&operation.old_text);
            let new_text = normalize(&operation.new_text);
            if old_text.is_empty() {
                return Err(MemoryEditError::InvalidEdit(t(
                    self.inner.locale,
                    keys::MEMORY_INVALID_EDIT_CODE,
                )));
            }
            match count_occurrences(&next, &old_text) {
                1 => {}
                0 => {
                    return Err(MemoryEditError::EditNotFound(t(
                        self.inner.locale,
                        keys::MEMORY_EDIT_NOT_FOUND_CODE,
                    )));
                }
                _ => {
                    return Err(MemoryEditError::AmbiguousEdit(t(
                        self.inner.locale,
                        keys::MEMORY_AMBIGUOUS_EDIT_CODE,
                    )));
                }
            }
            next = next.replacen(&old_text, &new_text, 1);
            changed += 1;
        }

        let addition = trim(&normalize(&request.append)).to_owned();
        if !addition.is_empty() {
            next = format!("{}\n\n{addition}\n", trim_end(&next));
            changed += 1;
        }

        next = normalize_markdown(&next);

        // Two ways to have done nothing: nothing was asked for, or everything
        // asked for was already true. Both report `changed: 0` and the
        // *current* document — note the document is built from `current`, not
        // from `current.trimEnd()`, which is upstream's own asymmetry.
        if changed == 0 || next == trim_end(&current) {
            return Ok(PreparedEdit {
                changed: 0,
                content: trim_end(&current).to_owned(),
                document: MemoryDocument::new(&self.inner.scope, current.clone(), &current),
            });
        }

        if code_point_len(&next) > self.inner.max_chars {
            let name = self
                .inner
                .file_path
                .as_deref()
                .map(file_name)
                .unwrap_or_default();
            return Err(MemoryEditError::DocumentTooLarge {
                message: i18n_format(
                    self.inner.locale,
                    keys::MEMORY_DOCUMENT_TOO_LARGE,
                    &[("name", &name), ("max", &self.inner.max_chars.to_string())],
                ),
            });
        }

        Ok(PreparedEdit {
            document: MemoryDocument::new(&self.inner.scope, next.clone(), &next),
            content: next,
            changed,
        })
    }

    /// Write `content` to `owner_id`'s copy.
    ///
    /// **External contract** — `markdown-context-store.mjs:192-201`: create the
    /// parent at `0700`, stage through `<path>.<pid>.tmp`, replace, `chmod`
    /// `0600`. Delegated to [`via_core::runtime::write_file`], which performs
    /// exactly that sequence through `via-store` (and adds the two `fsync`
    /// barriers upstream skips).
    ///
    /// # Errors
    ///
    /// [`MemoryEditError::Unavailable`] with no configured path, otherwise
    /// [`MemoryEditError::Io`].
    pub fn persist(&self, owner_id: &str, content: &str) -> Result<(), MemoryEditError> {
        let path = self
            .path_for(owner_id)
            .ok_or(MemoryEditError::Unavailable)?;
        via_core::runtime::write_file(&path, content, via_core::runtime::IfExists::Replace)
            .map_err(|error| MemoryEditError::Io(error.to_string()))?;
        *self.warning_slot() = None;
        Ok(())
    }

    /// The health triple for `/api/health.frontendMemory.documents.<scope>`.
    #[must_use]
    pub fn health(&self) -> DocumentHealth {
        let warning = self.warning_slot().clone();
        DocumentHealth {
            ok: warning.is_none(),
            configured: self.inner.file_path.is_some(),
            warning,
        }
    }

    /// The most recent warning, if any.
    #[must_use]
    pub fn warning(&self) -> Option<DocumentWarning> {
        self.warning_slot().clone()
    }

    fn truncate(&self, content: &str) -> String {
        if code_point_len(content) <= self.inner.max_chars {
            return content.to_owned();
        }
        format!(
            "{}{}",
            bounded_code_points(content, self.inner.max_chars),
            t(self.inner.locale, keys::MEMORY_TRUNCATION_MARKER)
        )
    }

    fn warn(&self, message: String) {
        let warning = DocumentWarning {
            message,
            at: (self.inner.now)(),
        };
        *self.warning_slot() = Some(warning.clone());
        // "Diagnostics must not break the voice service" —
        // `markdown-context-store.mjs:206-209` swallows a throwing sink.
        let on_warning = self.inner.on_warning.clone();
        let _ =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || on_warning(&warning)));
    }

    /// A poisoned lock guards one `Option<DocumentWarning>`. Refusing to serve
    /// memory because a diagnostics sink panicked would turn one panic into a
    /// mute assistant.
    fn warning_slot(&self) -> MutexGuard<'_, Option<DocumentWarning>> {
        self.inner
            .warning
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Tidy a Markdown body the way upstream does before writing it.
///
/// **External contract** — `markdown-context-store.mjs:46-62`. Three passes,
/// and all three are visible to a human opening the file:
///
/// 1. Drop bullet lines with no content (`- `, `*`, `+   `).
/// 2. Drop a bullet whose whitespace-collapsed item text was already seen.
///    This is what makes appending the same fact twice a no-op, and therefore
///    what makes the extractor's "do not repeat what is already covered" rule
///    enforceable rather than advisory.
/// 3. Collapse three or more blank lines to one, and trim the end.
///
/// Deduplication is **document-wide and order-preserving**: the first spelling
/// of an item wins and later ones vanish, even under a different heading.
#[must_use]
pub fn normalize_markdown(content: &str) -> String {
    let normalized = normalize(content);
    let mut seen: Vec<String> = Vec::new();
    let mut kept: Vec<&str> = Vec::new();
    for line in normalized.split('\n') {
        match bullet_item(line) {
            BulletLine::Empty => {}
            BulletLine::Item(item) => {
                let key = clean(item);
                if seen.iter().any(|entry| entry == &key) {
                    continue;
                }
                seen.push(key);
                kept.push(line);
            }
            BulletLine::NotABullet => kept.push(line),
        }
    }
    collapse_blank_runs(&kept.join("\n"))
}

enum BulletLine<'a> {
    /// `^\s*[-*+]\s*$`
    Empty,
    /// `^\s*[-*+]\s+(.+?)\s*$` — the captured item text.
    Item(&'a str),
    NotABullet,
}

fn bullet_item(line: &str) -> BulletLine<'_> {
    let rest = line.trim_start_matches(is_js_whitespace);
    let Some(after_marker) = rest.strip_prefix(['-', '*', '+']) else {
        return BulletLine::NotABullet;
    };
    let body = after_marker.trim_start_matches(is_js_whitespace);
    if body.is_empty() {
        // `^\s*[-*+]\s*$` — an empty bullet, whether or not any space followed.
        return BulletLine::Empty;
    }
    // `\s+` after the marker is required: `-item` is prose, not a bullet.
    if body.len() == after_marker.len() {
        return BulletLine::NotABullet;
    }
    BulletLine::Item(trim_end(body))
}

/// `replace(/\n{3,}/g, '\n\n').trimEnd()`.
fn collapse_blank_runs(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut run = 0usize;
    for c in value.chars() {
        if c == '\n' {
            run += 1;
            continue;
        }
        push_newlines(&mut out, run);
        run = 0;
        out.push(c);
    }
    push_newlines(&mut out, run);
    trim_end(&out).to_owned()
}

fn push_newlines(out: &mut String, run: usize) {
    let emitted = if run >= 3 { 2 } else { run };
    for _ in 0..emitted {
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_revision_is_sixteen_hex_characters_of_sha256() {
        let revision = digest("# MEMORY");
        assert_eq!(revision.len(), REVISION_HEX_LENGTH);
        assert!(revision.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(revision, digest("# USER"));
    }

    #[test]
    fn normalize_markdown_drops_empty_and_duplicate_bullets() {
        assert_eq!(normalize_markdown("- \n- 相同\n- 相同"), "- 相同");
        assert_eq!(normalize_markdown("*\n+   \n- a"), "- a");
        // Different bullet markers, same item text: still a duplicate.
        assert_eq!(normalize_markdown("- a\n* a"), "- a");
        // Whitespace inside the item is collapsed before comparison.
        assert_eq!(normalize_markdown("- a  b\n- a b"), "- a  b");
    }

    #[test]
    fn normalize_markdown_leaves_prose_and_headings_alone() {
        assert_eq!(
            normalize_markdown("# MEMORY\n\n## 兴趣\n\n- 篮球"),
            "# MEMORY\n\n## 兴趣\n\n- 篮球"
        );
        // `-item` has no whitespace after the marker, so it is not a bullet
        // and is never deduplicated.
        assert_eq!(normalize_markdown("-item\n-item"), "-item\n-item");
    }

    #[test]
    fn normalize_markdown_collapses_blank_runs_and_trims_the_end() {
        assert_eq!(normalize_markdown("a\n\n\n\n\nb\n\n\n"), "a\n\nb");
        assert_eq!(normalize_markdown("a\n\nb"), "a\n\nb");
    }

    #[test]
    fn the_document_id_is_the_scope_plus_the_suffix() {
        let document = MemoryDocument::new("user", "body", "body");
        assert_eq!(document.id, "user_document");
        assert_eq!(document.format, DOCUMENT_FORMAT);
        assert!(document.editable);
    }
}
