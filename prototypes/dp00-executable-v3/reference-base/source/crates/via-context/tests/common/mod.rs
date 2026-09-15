//! Fixtures shared by the integration tests.
//!
//! Each test binary compiles the whole module and uses part of it, which is
//! what a shared fixture module is for — the same pattern
//! `via-conversation/tests/common/mod.rs` uses.

#![allow(dead_code)]

use via_conversation::sync::{InputReference, Message, MessageRole, MessageSource};
use via_conversation::{ClientContext, MemoryDocument, RawClientContext, normalize_client_context};

/// A memory document in `scope` with `content` and `revision`.
pub fn document(scope: &str, content: &str, revision: &str) -> MemoryDocument {
    MemoryDocument {
        id: format!("{scope}_document"),
        scope: scope.to_owned(),
        content: content.to_owned(),
        format: "markdown".to_owned(),
        revision: revision.to_owned(),
        editable: true,
    }
}

/// A normalized client environment.
pub fn client(time_zone: &str, locale: &str, working_directory: &str) -> ClientContext {
    normalize_client_context(&RawClientContext {
        time_zone: time_zone.to_owned(),
        locale: locale.to_owned(),
        working_directory: working_directory.to_owned(),
    })
}

/// A conversation message.
pub fn message(role: MessageRole, content: &str) -> Message {
    Message {
        seq: 1,
        id: "id".to_owned(),
        role,
        content: content.to_owned(),
        source: MessageSource::VoiceUser,
        turn_id: None,
        task_id: None,
        task_ids: Vec::new(),
        inputs: Vec::<InputReference>::new(),
        created_at: 0,
    }
}
