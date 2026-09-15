//! The `memory` and `notes` tools — ported from the memory and notes halves of
//! `server/test/tool-call-handler.test.mjs`, with every failure code exercised.

use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use via_conversation::markdown_store::MarkdownContextStore;
use via_conversation::memory_service::FrontendMemoryService;
use via_conversation::memory_tool::{
    INVALID_MEMORY_ACTION, INVALID_MEMORY_DOCUMENT, INVALID_MEMORY_EDIT, MEMORY_UNAVAILABLE,
    MEMORY_WRITE_FAILED, MemoryTool, MemoryToolOutcome, MemoryToolRequest, SENSITIVE_MEMORY,
};
use via_conversation::notes::FrontendNotesStore;
use via_conversation::notes_tool::{
    INVALID_NOTES_ACTION, MISSING_NOTES_ITEMS, MISSING_NOTES_TARGET, NOTES_ACTIONS,
    NOTES_UNAVAILABLE, NotesTool, NotesToolOutcome, NotesToolRequest, SENSITIVE_NOTES,
};
use via_i18n::{Locale, keys, t};

const OWNER: &str = "user_personal";

fn memory_tool() -> (
    tempfile::TempDir,
    MemoryTool,
    MarkdownContextStore,
    MarkdownContextStore,
) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let user = MarkdownContextStore::builder()
        .file_path(dir.path().join("USER.md"))
        .scope("user")
        .template("# USER")
        .locale(Locale::Zh)
        .build();
    let memory = MarkdownContextStore::builder()
        .file_path(dir.path().join("MEMORY.md"))
        .scope("memory")
        .template("# MEMORY")
        .locale(Locale::Zh)
        .build();
    let tool = MemoryTool::new(
        Some(FrontendMemoryService::new(
            Some(user.clone()),
            Some(memory.clone()),
        )),
        Locale::Zh,
    );
    (dir, tool, user, memory)
}

// ── memory ──────────────────────────────────────────────────────────────────

#[test]
fn a_read_reports_ok_or_not_found_with_a_count() {
    let (_dir, tool, _user, _memory) = memory_tool();
    let empty = tool.handle(OWNER, &MemoryToolRequest::read(None));
    assert_eq!(
        empty.to_value(),
        json!({ "status": "not_found", "count": 0, "documents": [] })
    );

    let _ = tool.handle(OWNER, &MemoryToolRequest::append("memory", "- 事实"));
    let filled = tool.handle(OWNER, &MemoryToolRequest::read(None));
    match &filled {
        MemoryToolOutcome::Read {
            status,
            count,
            documents,
        } => {
            assert_eq!(*status, "ok");
            assert_eq!(*count, 1);
            assert_eq!(documents[0].scope, "memory");
        }
        other => panic!("expected a read, got {other:?}"),
    }
}

#[test]
fn a_read_can_name_one_document_or_all_of_them() {
    let (_dir, tool, _user, _memory) = memory_tool();
    let _ = tool.handle(OWNER, &MemoryToolRequest::append("user", "- 称呼：老大"));
    let _ = tool.handle(OWNER, &MemoryToolRequest::append("memory", "- 喜欢篮球"));

    for (document, expected) in [
        (None, 2),
        (Some("all"), 2),
        (Some("user"), 1),
        (Some("facts"), 1),
    ] {
        match tool.handle(OWNER, &MemoryToolRequest::read(document)) {
            MemoryToolOutcome::Read { count, .. } => assert_eq!(count, expected, "{document:?}"),
            other => panic!("expected a read, got {other:?}"),
        }
    }
}

#[test]
fn an_unknown_document_is_refused_differently_for_a_read_and_a_write() {
    let (_dir, tool, _user, _memory) = memory_tool();
    let read = tool.handle(OWNER, &MemoryToolRequest::read(Some("assistant")));
    assert_eq!(read.error_code(), Some(INVALID_MEMORY_DOCUMENT));
    assert_eq!(
        read.to_value()["user_message"],
        json!(t(Locale::Zh, keys::MEMORY_ERROR_INVALID_DOCUMENT_READ))
    );

    let write = tool.handle(OWNER, &MemoryToolRequest::append("assistant", "- x"));
    assert_eq!(write.error_code(), Some(INVALID_MEMORY_DOCUMENT));
    assert_eq!(
        write.to_value()["user_message"],
        json!(t(Locale::Zh, keys::MEMORY_ERROR_INVALID_DOCUMENT_WRITE)),
        "a write says which document it needs"
    );
}

#[test]
fn a_write_with_no_document_is_refused_before_its_content_is_examined() {
    let (_dir, tool, _user, _memory) = memory_tool();
    let outcome = tool.handle(
        OWNER,
        &MemoryToolRequest {
            action: "append".to_owned(),
            ..MemoryToolRequest::default()
        },
    );
    assert_eq!(
        outcome.error_code(),
        Some(INVALID_MEMORY_DOCUMENT),
        "the branch order is exact: document before edit"
    );
}

#[test]
fn an_unrecognised_action_is_refused() {
    let (_dir, tool, _user, _memory) = memory_tool();
    for action in ["", "  ", "delete", "READ ME"] {
        let outcome = tool.handle(
            OWNER,
            &MemoryToolRequest {
                action: action.to_owned(),
                ..MemoryToolRequest::default()
            },
        );
        assert_eq!(
            outcome.error_code(),
            Some(INVALID_MEMORY_ACTION),
            "{action}"
        );
    }
    // The action is trimmed and case-folded first.
    assert!(
        tool.handle(
            OWNER,
            &MemoryToolRequest {
                action: "  READ  ".to_owned(),
                ..MemoryToolRequest::default()
            },
        )
        .error_code()
        .is_none()
    );
}

#[test]
fn append_needs_content_and_replace_needs_both_texts() {
    let (_dir, tool, _user, _memory) = memory_tool();
    for request in [
        MemoryToolRequest::append("memory", ""),
        MemoryToolRequest::append("memory", "   \n "),
    ] {
        let outcome = tool.handle(OWNER, &request);
        assert_eq!(outcome.error_code(), Some(INVALID_MEMORY_EDIT));
        assert_eq!(
            outcome.to_value()["user_message"],
            json!(t(Locale::Zh, keys::MEMORY_ERROR_APPEND_NEEDS_CONTENT))
        );
    }

    // An omitted `new_text` is not an empty one: the key must be present.
    let omitted = tool.handle(
        OWNER,
        &MemoryToolRequest {
            action: "replace".to_owned(),
            document: Some("memory".to_owned()),
            old_text: Some("- 原文".to_owned()),
            new_text: None,
            content: None,
        },
    );
    assert_eq!(omitted.error_code(), Some(INVALID_MEMORY_EDIT));
    assert_eq!(
        omitted.to_value()["user_message"],
        json!(t(Locale::Zh, keys::MEMORY_ERROR_REPLACE_NEEDS_TEXTS))
    );

    let no_fragment = tool.handle(OWNER, &MemoryToolRequest::replace("memory", "", "x"));
    assert_eq!(no_fragment.error_code(), Some(INVALID_MEMORY_EDIT));
}

#[test]
fn an_empty_new_text_is_a_deletion_not_a_missing_argument() {
    let (_dir, tool, _user, memory) = memory_tool();
    let _ = tool.handle(OWNER, &MemoryToolRequest::append("memory", "- 要删掉的"));
    let outcome = tool.handle(
        OWNER,
        &MemoryToolRequest::replace("memory", "- 要删掉的", ""),
    );
    assert_eq!(outcome.changed(), 1);
    assert!(!memory.read(OWNER).contains("要删掉的"));
}

#[test]
fn a_secret_is_rejected_rather_than_failed() {
    let (_dir, tool, _user, memory) = memory_tool();
    for content in [
        "- 我的密码是 123456",
        "- api_key is sk-abcdef",
        "- the access token",
    ] {
        let outcome = tool.handle(OWNER, &MemoryToolRequest::append("memory", content));
        assert_eq!(outcome.error_code(), Some(SENSITIVE_MEMORY));
        let value = outcome.to_value();
        assert_eq!(value["status"], json!("rejected"));
        assert_eq!(value["retryable"], json!(false), "retrying will not help");
        assert_eq!(
            value["user_message"],
            json!(t(Locale::Zh, keys::MEMORY_ERROR_SENSITIVE))
        );
    }
    assert_eq!(memory.read(OWNER), "");

    // The gate reads the *replacement*, not the fragment being removed, so a
    // secret can still be deleted from memory.
    let _ = tool.handle(OWNER, &MemoryToolRequest::append("memory", "- 一段旧内容"));
    let removal = tool.handle(
        OWNER,
        &MemoryToolRequest::replace("memory", "- 一段旧内容", ""),
    );
    assert_eq!(removal.changed(), 1);
}

#[test]
fn a_missing_or_ambiguous_fragment_comes_back_retryable_with_the_documents() {
    let (_dir, tool, _user, _memory) = memory_tool();
    let _ = tool.handle(
        OWNER,
        &MemoryToolRequest::append("memory", "- 重复\n\n## 别处\n\n- 重复内容"),
    );

    for (old_text, code) in [("- 不存在", "edit_not_found"), ("重复", "ambiguous_edit")] {
        let outcome = tool.handle(OWNER, &MemoryToolRequest::replace("memory", old_text, "x"));
        assert_eq!(outcome.error_code(), Some(code));
        let value = outcome.to_value();
        assert_eq!(value["status"], json!("failed"));
        assert_eq!(value["retryable"], json!(true));
        assert_eq!(
            value["user_message"],
            json!(t(Locale::Zh, keys::MEMORY_ERROR_STALE_DOCUMENT))
        );
        // The current documents are re-attached so the model can quote a
        // fragment that exists instead of guessing again.
        let documents = value["documents"].as_array().expect("documents");
        assert_eq!(documents.len(), 1);
        assert!(
            documents[0]["content"]
                .as_str()
                .unwrap_or_default()
                .contains("- 重复")
        );
        assert!(documents[0]["revision"].is_string());
    }
}

#[test]
fn a_write_that_cannot_reach_the_disk_reports_memory_write_failed() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("MEMORY.md");
    std::fs::create_dir(&path).expect("a directory in the file's place");
    let memory = MarkdownContextStore::builder()
        .file_path(&path)
        .scope("memory")
        .template("# MEMORY")
        .locale(Locale::Zh)
        .build();
    let tool = MemoryTool::new(
        Some(FrontendMemoryService::new(None, Some(memory))),
        Locale::Zh,
    );

    let outcome = tool.handle(OWNER, &MemoryToolRequest::append("memory", "- 内容"));
    assert_eq!(outcome.error_code(), Some(MEMORY_WRITE_FAILED));
    let value = outcome.to_value();
    assert_eq!(value["retryable"], json!(true));
    assert!(
        value.get("documents").is_none(),
        "only the re-readable trio re-attaches documents"
    );
}

#[test]
fn an_unchanged_write_is_reported_as_unchanged() {
    let (_dir, tool, _user, _memory) = memory_tool();
    let _ = tool.handle(OWNER, &MemoryToolRequest::append("memory", "- 相同"));
    let repeat = tool.handle(OWNER, &MemoryToolRequest::append("memory", "- 相同"));
    let value = repeat.to_value();
    assert_eq!(value["status"], json!("unchanged"));
    assert_eq!(value["changed"], json!(0));
    assert_eq!(repeat.changed(), 0, "the session is not re-instructed");
}

#[test]
fn a_successful_write_says_how_many_operations_changed_something() {
    let (_dir, tool, _user, _memory) = memory_tool();
    let outcome = tool.handle(OWNER, &MemoryToolRequest::append("user", "- 称呼：老大"));
    let value = outcome.to_value();
    assert_eq!(
        value
            .as_object()
            .expect("an object")
            .keys()
            .collect::<Vec<_>>(),
        vec!["status", "changed", "documents"]
    );
    assert_eq!(value["status"], json!("updated"));
    assert_eq!(outcome.changed(), 1);
}

#[test]
fn an_unavailable_service_is_reported_rather_than_silently_dropped() {
    let tool = MemoryTool::new(None, Locale::Zh);
    let outcome = tool.handle(OWNER, &MemoryToolRequest::append("user", "- 内容"));
    assert_eq!(outcome.error_code(), Some(MEMORY_UNAVAILABLE));
    assert_eq!(
        outcome.to_value()["user_message"],
        json!(t(Locale::Zh, keys::MEMORY_ERROR_UNAVAILABLE)),
        "the model must never claim it remembered"
    );
}

#[test]
fn a_legacy_alias_is_accepted_by_the_tool() {
    let (_dir, tool, user, _memory) = memory_tool();
    let outcome = tool.handle(OWNER, &MemoryToolRequest::append("profile", "- 称呼：老大"));
    assert_eq!(outcome.changed(), 1);
    assert!(user.read(OWNER).contains("老大"));
}

// ── notes ───────────────────────────────────────────────────────────────────

fn notes_tool() -> NotesTool {
    NotesTool::new(Some(FrontendNotesStore::in_memory()), Locale::Zh)
}

fn value_of(outcome: &NotesToolOutcome) -> Value {
    outcome.to_value()
}

#[test]
fn lists_answers_without_a_target() {
    let tool = notes_tool();
    let empty = tool.handle(OWNER, &NotesToolRequest::new("lists", None));
    assert_eq!(value_of(&empty), json!({ "status": "empty", "lists": [] }));

    let _ = tool.handle(
        OWNER,
        &NotesToolRequest::with_items("add", "购物清单", &["牛奶"]),
    );
    let filled = tool.handle(OWNER, &NotesToolRequest::new("lists", None));
    assert_eq!(filled.status(), "ok");
    assert_eq!(value_of(&filled)["lists"][0]["list"], json!("购物清单"),);
}

#[test]
fn every_other_action_needs_a_list_name() {
    let tool = notes_tool();
    for action in NOTES_ACTIONS.iter().filter(|action| **action != "lists") {
        for list in [None, Some(""), Some("   ")] {
            let outcome = tool.handle(OWNER, &NotesToolRequest::new(action, list));
            assert_eq!(
                outcome.error_code(),
                Some(MISSING_NOTES_TARGET),
                "{action} with {list:?}"
            );
            assert_eq!(
                value_of(&outcome)["user_message"],
                json!(t(Locale::Zh, keys::NOTES_ERROR_MISSING_TARGET))
            );
        }
    }
}

#[test]
fn add_and_remove_need_items() {
    let tool = notes_tool();
    for action in ["add", "remove"] {
        let outcome = tool.handle(OWNER, &NotesToolRequest::new(action, Some("购物清单")));
        assert_eq!(outcome.error_code(), Some(MISSING_NOTES_ITEMS));
        // Items that are only whitespace do not count.
        let blank = tool.handle(
            OWNER,
            &NotesToolRequest::with_items(action, "购物清单", &["", "   "]),
        );
        assert_eq!(blank.error_code(), Some(MISSING_NOTES_ITEMS));
    }
}

#[test]
fn a_secret_item_is_rejected_and_nothing_is_stored() {
    let store = FrontendNotesStore::in_memory();
    let tool = NotesTool::new(Some(store.clone()), Locale::Zh);
    let outcome = tool.handle(
        OWNER,
        &NotesToolRequest::with_items("add", "购物清单", &["牛奶", "我的密码是 12345"]),
    );
    assert_eq!(outcome.error_code(), Some(SENSITIVE_NOTES));
    let value = value_of(&outcome);
    assert_eq!(value["status"], json!("rejected"));
    assert_eq!(
        value["user_message"],
        json!(t(Locale::Zh, keys::NOTES_ERROR_SENSITIVE))
    );
    assert!(
        store.lists(OWNER).is_empty(),
        "one bad item rejects the whole call"
    );
}

#[test]
fn an_unrecognised_action_is_refused_before_the_store_is_touched() {
    let tool = notes_tool();
    for action in ["", "delete", "  ", "LISTS!"] {
        assert_eq!(
            tool.handle(OWNER, &NotesToolRequest::new(action, Some("购物清单")))
                .error_code(),
            Some(INVALID_NOTES_ACTION),
            "{action}"
        );
    }
    // Trimmed and case-folded first.
    assert!(
        tool.handle(OWNER, &NotesToolRequest::new(" LISTS ", None))
            .error_code()
            .is_none()
    );
}

#[test]
fn the_tool_caps_items_at_twenty_per_call() {
    let store = FrontendNotesStore::in_memory();
    let tool = NotesTool::new(Some(store.clone()), Locale::Zh);
    let items: Vec<String> = (0..50).map(|index| format!("条目{index}")).collect();
    let outcome = tool.handle(
        OWNER,
        &NotesToolRequest {
            action: "add".to_owned(),
            list: Some("长清单".to_owned()),
            items,
        },
    );
    assert_eq!(
        value_of(&outcome)["added"].as_array().map(Vec::len),
        Some(20)
    );
    assert_eq!(store.lists(OWNER)[0].count, 20);
}

#[test]
fn ambiguity_is_a_result_not_a_failure() {
    let tool = notes_tool();
    let _ = tool.handle(
        OWNER,
        &NotesToolRequest::with_items("add", "书单", &["三体"]),
    );
    let _ = tool.handle(
        OWNER,
        &NotesToolRequest::with_items("add", "购物清单", &["牛奶"]),
    );

    let outcome = tool.handle(OWNER, &NotesToolRequest::new("show", Some("单")));
    assert_eq!(
        outcome.error_code(),
        None,
        "the model asks, it does not retry"
    );
    assert_eq!(outcome.status(), "ambiguous");
    let candidates = value_of(&outcome)["candidates"]
        .as_array()
        .map(|values| values.len());
    assert_eq!(candidates, Some(2));
}

#[test]
fn clear_and_drop_reach_the_store_unconditionally() {
    let store = FrontendNotesStore::in_memory();
    let tool = NotesTool::new(Some(store.clone()), Locale::Zh);
    let _ = tool.handle(
        OWNER,
        &NotesToolRequest::with_items("add", "购物清单", &["牛奶", "面包"]),
    );

    // The destructive-intent gate is in the tool description, not here: the
    // store executes both the moment the model calls them.
    let cleared = tool.handle(OWNER, &NotesToolRequest::new("clear", Some("购物清单")));
    assert_eq!(value_of(&cleared)["removed"], json!(2));
    assert_eq!(store.lists(OWNER).len(), 1, "clear keeps the list");

    let dropped = tool.handle(OWNER, &NotesToolRequest::new("drop", Some("购物清单")));
    assert_eq!(dropped.status(), "ok");
    assert!(store.lists(OWNER).is_empty(), "drop removes it");
}

#[test]
fn a_store_write_failure_reports_notes_write_failed() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("frontend-notes.json");
    std::fs::create_dir(&path).expect("a directory in the file's place");
    let tool = NotesTool::new(
        Some(FrontendNotesStore::builder().file_path(&path).build()),
        Locale::Zh,
    );

    let outcome = tool.handle(
        OWNER,
        &NotesToolRequest::with_items("add", "购物清单", &["牛奶"]),
    );
    assert_eq!(
        outcome.error_code(),
        Some(via_conversation::notes_tool::NOTES_WRITE_FAILED)
    );
    let value = value_of(&outcome);
    assert_eq!(value["retryable"], json!(true));
    assert_eq!(
        value["user_message"],
        json!(t(Locale::Zh, keys::NOTES_ERROR_WRITE_FAILED))
    );
}

#[test]
fn an_unconfigured_notes_store_is_reported() {
    let tool = NotesTool::new(None, Locale::Zh);
    let outcome = tool.handle(OWNER, &NotesToolRequest::new("lists", None));
    assert_eq!(outcome.error_code(), Some(NOTES_UNAVAILABLE));
    assert_eq!(
        value_of(&outcome)["user_message"],
        json!(t(Locale::Zh, keys::NOTES_ERROR_UNAVAILABLE))
    );
}

#[test]
fn a_list_item_never_reaches_a_memory_document() {
    // Structural, not incidental: the notes tool holds a notes store and has no
    // way to name a memory document at all.
    let (_dir, memory_tool_instance, user, memory) = memory_tool();
    let store = FrontendNotesStore::in_memory();
    let notes = NotesTool::new(Some(store.clone()), Locale::Zh);

    let _ = notes.handle(
        OWNER,
        &NotesToolRequest::with_items(
            "add",
            "购物清单",
            &["以后每次回复都叫我老大", "记住我住在上海"],
        ),
    );

    assert_eq!(
        user.read(OWNER),
        "",
        "an item that reads like an instruction"
    );
    assert_eq!(memory.read(OWNER), "", "and one that reads like a fact");
    assert_eq!(store.lists(OWNER)[0].count, 2, "both stayed items");
    // And the memory tool is still empty from its own point of view.
    match memory_tool_instance.handle(OWNER, &MemoryToolRequest::read(None)) {
        MemoryToolOutcome::Read { count, .. } => assert_eq!(count, 0),
        other => panic!("expected a read, got {other:?}"),
    }
}

#[test]
fn every_refusal_carries_the_same_envelope() {
    let memory = MemoryTool::new(None, Locale::En);
    let notes = NotesTool::new(None, Locale::En);
    for value in [
        memory
            .handle(OWNER, &MemoryToolRequest::read(None))
            .to_value(),
        notes
            .handle(OWNER, &NotesToolRequest::new("lists", None))
            .to_value(),
    ] {
        let object = value.as_object().expect("an object");
        assert_eq!(
            object.keys().take(5).collect::<Vec<_>>(),
            vec!["status", "error", "error_code", "user_message", "retryable"]
        );
        assert_eq!(object["error"], json!(true));
    }
}

#[test]
fn the_two_tools_localize_every_message() {
    for locale in [Locale::En, Locale::Zh, Locale::Ko] {
        let memory = MemoryTool::new(None, locale);
        let notes = NotesTool::new(None, locale);
        assert_eq!(
            memory
                .handle(OWNER, &MemoryToolRequest::read(None))
                .to_value()["user_message"],
            json!(t(locale, keys::MEMORY_ERROR_UNAVAILABLE))
        );
        assert_eq!(
            notes
                .handle(OWNER, &NotesToolRequest::new("lists", None))
                .to_value()["user_message"],
            json!(t(locale, keys::NOTES_ERROR_UNAVAILABLE))
        );
    }
}

#[test]
fn several_memory_calls_in_one_turn_are_each_atomic() {
    // "Each call executes one read, append or replace; several persistent
    // changes in one sentence are several calls."
    let (_dir, tool, user, memory) = memory_tool();
    let outcomes = [
        tool.handle(OWNER, &MemoryToolRequest::append("user", "- 称呼：老大")),
        tool.handle(OWNER, &MemoryToolRequest::append("memory", "- 住在上海")),
        tool.handle(
            OWNER,
            &MemoryToolRequest::replace("user", "- 称呼：老大", "- 称呼：船长"),
        ),
    ];
    assert_eq!(
        outcomes
            .iter()
            .map(MemoryToolOutcome::changed)
            .sum::<usize>(),
        3
    );
    assert!(user.read(OWNER).contains("船长"));
    assert!(!user.read(OWNER).contains("老大"));
    assert!(memory.read(OWNER).contains("住在上海"));
}
