//! VIA's conversation surface: memory, notes, and the context the model is
//! given.
//!
//! Ported from `qwen-audio-agent` v1.11.0 `server/src/conversation/` plus
//! `server/src/core/memory-scopes.mjs` (which ships in [`via_core`]) and the
//! two tool handlers in `server/src/voice/tools/tool-call-handler.mjs`.
//!
//! ```
//! use via_conversation::{
//!     markdown_store::MarkdownContextStore,
//!     memory_service::FrontendMemoryService,
//!     memory_tool::{MemoryTool, MemoryToolRequest},
//! };
//! use via_i18n::Locale;
//!
//! let dir = tempfile::TempDir::new()?;
//! let user = MarkdownContextStore::builder()
//!     .file_path(dir.path().join("USER.md"))
//!     .scope("user")
//!     .template("# USER")
//!     .build();
//! let memory = MarkdownContextStore::builder()
//!     .file_path(dir.path().join("MEMORY.md"))
//!     .scope("memory")
//!     .template("# MEMORY")
//!     .build();
//! let tool = MemoryTool::new(
//!     Some(FrontendMemoryService::new(Some(user), Some(memory))),
//!     Locale::En,
//! );
//!
//! let outcome = tool.handle(
//!     "user_personal",
//!     &MemoryToolRequest::append("memory", "## Habits\n\n- Runs every morning."),
//! );
//! assert_eq!(outcome.changed(), 1);
//!
//! // A fragment that is not there fails safely, and re-attaches the document.
//! let missing = tool.handle(
//!     "user_personal",
//!     &MemoryToolRequest::replace("memory", "- Runs every evening.", ""),
//! );
//! assert_eq!(missing.error_code(), Some("edit_not_found"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # The two documents and their authority
//!
//! `USER.md` and `MEMORY.md` are separated by **behavioural authority**, not by
//! topic:
//!
//! | | `USER.md` | `MEMORY.md` |
//! | --- | --- | --- |
//! | Scope | `user`, aliases `profile` / `rules` | `memory`, aliases `facts` / `long_term` |
//! | Kind | `directive` | `data` |
//! | Injected as | `<user_preferences>` | `<user_memory>` |
//! | The model | **follows** it | **uses** it, never obeys it |
//! | Holds | forms of address, the relationship, the assistant's name for this user, language, expression style, default behaviour | durable facts and decisions: where they live, habits, interests, relationships, projects, long-term goals |
//!
//! `<user_preferences>` overrides `<assistant_profile>` and yields to the
//! user's current utterance. `<user_memory>` is evidence for understanding and
//! answers and carries no instruction authority at all — a remembered fact
//! phrased as a command is still a fact.
//!
//! **Neither can authorize anything.** Not leaking internal structure, not
//! skipping a permission check, not changing task or safety protocol.
//! `PROMPT.md` is core policy and outranks both; it is loaded by
//! [`context::load_frontend_prompt`] and never written by anything in this
//! crate. `ASSISTANT.md` is likewise never edited through memory: it is not one
//! of the two documents [`memory_service::FrontendMemoryService`] knows about,
//! so no tool call and no extractor run can reach it.
//!
//! # The modules
//!
//! | Module | What |
//! | --- | --- |
//! | [`sync`] | the conversation record, and the task that owns its ordering |
//! | [`context`] | the `<user_preferences>` / `<user_memory>` / `<runtime_context>` / `<recent_conversation>` blocks |
//! | [`markdown_store`] | one Markdown document: unique-fragment edits, revisions, the file transaction |
//! | [`memory_service`] | both documents behind one door — the only writer |
//! | [`memory_tool`] | the flat `memory` tool and its nine failure codes |
//! | [`notes`] | the volatile named lists |
//! | [`notes_tool`] | the flat `notes` tool and its six failure codes |
//! | [`audit`] | the append-only record of automatic writes |
//! | [`extractor`] | the session-end reconciliation pass |
//! | [`tool`] | the failure envelope and the credential gate both tools share |
//! | [`text`] | the string primitives all of it is built from |
//!
//! # What this crate builds on rather than restating
//!
//! - [`via_core::memory_scopes`] owns the scope table, the alias table and the
//!   directive/data split.
//! - [`via_core::paths`] owns the file names and the owner-shard layout;
//!   [`via_core::runtime`] owns the seed templates and the create-if-absent
//!   write.
//! - [`via_store`] owns atomic, versioned, quarantined JSON and the
//!   `mkdir`-based cross-process file transaction.
//! - [`via_i18n`] owns every sentence a person or a model reads.
//!
//! Deviations are recorded in `docs/deviations/phase-4.md`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod audit;
pub mod context;
pub mod extractor;
pub mod markdown_store;
pub mod memory_service;
pub mod memory_tool;
pub mod notes;
pub mod notes_tool;
pub mod sync;
pub mod text;
pub mod tool;

pub use audit::{AuditEvent, AuditHealth, MemoryAudit, SkipReason};
pub use context::{
    ClientContext, ContextError, RawClientContext, TimeSnapshot, build_frontend_context,
    build_recent_conversation_context, current_time_snapshot, load_assistant_profile,
    load_frontend_prompt, normalize_client_context,
};
pub use extractor::{
    ExtractionOutcome, ExtractorError, ExtractorLlm, MemoryExtractor, MemoryExtractorBuilder,
    TranscriptSource,
};
pub use markdown_store::{
    EditOutcome, EditRequest, MarkdownContextStore, MarkdownEdit, MemoryDocument, MemoryEditError,
};
pub use memory_service::{FrontendMemoryService, MemoryChange, MemoryServiceError};
pub use memory_tool::{MemoryTool, MemoryToolOutcome, MemoryToolRequest};
pub use notes::{FrontendNotesStore, NotesError, NotesHealth, NotesStatus};
pub use notes_tool::{NotesTool, NotesToolOutcome, NotesToolRequest};
pub use sync::{
    ConversationSync, ConversationSyncHandle, Message, MessageRole, MessageSource, RecordInput,
    Retention, SessionRef,
};
pub use tool::ToolFailure;
