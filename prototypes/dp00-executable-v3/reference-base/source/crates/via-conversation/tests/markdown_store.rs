//! [`MarkdownContextStore`] — ported from
//! `server/test/markdown-context-store.test.mjs`, plus every refusal path.

use std::sync::Arc;

use pretty_assertions::assert_eq;
use via_conversation::markdown_store::{
    DEFAULT_PERSONAL_OWNER_ID, DocumentWarning, EditRequest, MarkdownContextStore, MarkdownEdit,
    MemoryEditError, digest, normalize_markdown,
};
use via_i18n::{Locale, keys, t};

const OWNER: &str = DEFAULT_PERSONAL_OWNER_ID;

struct Fixture {
    _dir: tempfile::TempDir,
    path: std::path::PathBuf,
    store: MarkdownContextStore,
}

fn fixture(scope: &str) -> Fixture {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join(if scope == "user" {
        "USER.md"
    } else {
        "MEMORY.md"
    });
    let store = MarkdownContextStore::builder()
        .file_path(&path)
        .scope(scope)
        .template(if scope == "user" {
            "# USER"
        } else {
            "# MEMORY"
        })
        .locale(Locale::Zh)
        .build();
    Fixture {
        _dir: dir,
        path,
        store,
    }
}

fn append(text: &str) -> EditRequest {
    EditRequest {
        append: text.to_owned(),
        ..EditRequest::default()
    }
}

#[test]
fn memory_stays_ordinary_human_readable_markdown() {
    let fixture = fixture("memory");
    let result = fixture
        .store
        .edit(OWNER, &append("## 兴趣\n\n- 用户喜欢打篮球。"))
        .expect("the edit applies");

    assert_eq!(result.changed, 1);
    assert_eq!(
        std::fs::read_to_string(&fixture.path).expect("readable"),
        "# MEMORY\n\n## 兴趣\n\n- 用户喜欢打篮球。\n"
    );
    // No envelope, no ids, no timestamps — which is what makes `replace` able
    // to quote a fragment the user could have typed themselves.
    let documents = fixture.store.list(OWNER);
    assert_eq!(documents.len(), 1);
    assert_eq!(documents[0].format, "markdown");
    assert_eq!(documents[0].scope, "memory");
}

#[test]
fn several_exact_edits_apply_together_including_a_deletion() {
    let fixture = fixture("memory");
    let first = fixture
        .store
        .edit(OWNER, &append("## 用户\n\n- 称呼为老板。\n- 喜欢篮球。"))
        .expect("seeded");

    let result = fixture
        .store
        .edit(
            OWNER,
            &EditRequest {
                edits: vec![
                    MarkdownEdit {
                        old_text: "- 称呼为老板。".to_owned(),
                        new_text: "- 称呼为船长。".to_owned(),
                    },
                    MarkdownEdit {
                        old_text: "- 喜欢篮球。".to_owned(),
                        new_text: String::new(),
                    },
                ],
                expected_revision: first.document.revision.clone(),
                ..EditRequest::default()
            },
        )
        .expect("the edits apply");

    assert_eq!(result.changed, 2);
    assert!(result.document.content.contains("称呼为船长"));
    assert!(!result.document.content.contains("喜欢篮球"));
}

#[test]
fn a_stale_revision_is_refused_before_anything_is_written() {
    let fixture = fixture("memory");
    fixture
        .store
        .edit(OWNER, &append("- 原有内容"))
        .expect("seeded");
    let before = std::fs::read_to_string(&fixture.path).expect("readable");

    let error = fixture
        .store
        .edit(
            OWNER,
            &EditRequest {
                append: "- 新内容".to_owned(),
                expected_revision: "stale".to_owned(),
                ..EditRequest::default()
            },
        )
        .expect_err("a stale revision is refused");

    assert_eq!(error.code(), Some("stale_document"));
    assert!(error.is_retryable_read_again());
    assert_eq!(
        error.to_string(),
        t(Locale::Zh, keys::MEMORY_STALE_DOCUMENT_CODE)
    );
    assert_eq!(
        std::fs::read_to_string(&fixture.path).expect("readable"),
        before,
        "a refused edit must not touch the file"
    );
}

#[test]
fn duplicate_and_empty_bullets_are_removed_and_a_no_op_reports_zero() {
    let fixture = fixture("memory");
    let first = fixture
        .store
        .edit(OWNER, &append("- \n- 相同\n- 相同"))
        .expect("seeded");

    let unchanged = fixture
        .store
        .edit(
            OWNER,
            &EditRequest {
                append: "- 相同".to_owned(),
                expected_revision: first.document.revision.clone(),
                ..EditRequest::default()
            },
        )
        .expect("the append is a no-op");

    assert_eq!(unchanged.changed, 0);
    assert_eq!(
        std::fs::read_to_string(&fixture.path).expect("readable"),
        "# MEMORY\n\n- 相同\n"
    );
}

#[test]
fn non_personal_owners_get_their_own_files() {
    let fixture = fixture("memory");
    fixture
        .store
        .edit("owner-a", &append("- A 的记忆"))
        .expect("a writes");
    fixture
        .store
        .edit("owner-b", &append("- B 的记忆"))
        .expect("b writes");

    assert_eq!(fixture.store.read(OWNER), "");
    assert!(fixture.store.read("owner-a").contains("A 的记忆"));
    assert!(fixture.store.read("owner-b").contains("B 的记忆"));
    assert!(
        !fixture.path.exists(),
        "the personal file was never created"
    );
    // And neither can see the other.
    assert!(!fixture.store.read("owner-a").contains("B 的记忆"));
}

#[test]
fn a_missing_fragment_and_an_ambiguous_one_are_told_apart() {
    let fixture = fixture("memory");
    fixture
        .store
        .edit(OWNER, &append("- 重复\n\n## 别处\n\n- 重复内容"))
        .expect("seeded");

    let missing = fixture
        .store
        .edit(
            OWNER,
            &EditRequest {
                edits: vec![MarkdownEdit {
                    old_text: "- 不存在".to_owned(),
                    new_text: String::new(),
                }],
                ..EditRequest::default()
            },
        )
        .expect_err("a missing fragment is refused");
    assert_eq!(missing.code(), Some("edit_not_found"));
    assert_eq!(
        missing.to_string(),
        t(Locale::Zh, keys::MEMORY_EDIT_NOT_FOUND_CODE)
    );

    let ambiguous = fixture
        .store
        .edit(
            OWNER,
            &EditRequest {
                edits: vec![MarkdownEdit {
                    old_text: "重复".to_owned(),
                    new_text: "改过".to_owned(),
                }],
                ..EditRequest::default()
            },
        )
        .expect_err("an ambiguous fragment is refused");
    assert_eq!(ambiguous.code(), Some("ambiguous_edit"));
    assert_eq!(
        ambiguous.to_string(),
        t(Locale::Zh, keys::MEMORY_AMBIGUOUS_EDIT_CODE)
    );

    // The document is untouched by both.
    assert!(fixture.store.read(OWNER).contains("- 重复"));
    assert!(fixture.store.read(OWNER).contains("- 重复内容"));
}

#[test]
fn an_edit_with_no_old_text_is_refused() {
    let fixture = fixture("memory");
    let error = fixture
        .store
        .edit(
            OWNER,
            &EditRequest {
                edits: vec![MarkdownEdit::default()],
                ..EditRequest::default()
            },
        )
        .expect_err("an empty fragment is refused");
    assert_eq!(error.code(), Some("invalid_edit"));
    assert!(!error.is_retryable_read_again());
    assert_eq!(
        error.to_string(),
        t(Locale::Zh, keys::MEMORY_INVALID_EDIT_CODE)
    );
}

#[test]
fn an_over_long_document_is_refused_with_its_name_and_cap() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("MEMORY.md");
    let store = MarkdownContextStore::builder()
        .file_path(&path)
        .scope("memory")
        .max_chars(20)
        .template("# MEMORY")
        .locale(Locale::Zh)
        .build();

    let error = store
        .edit(OWNER, &append(&"字".repeat(40)))
        .expect_err("the cap is enforced");
    assert_eq!(error.code(), Some("document_too_large"));
    assert!(
        !error.is_retryable_read_again(),
        "re-reading will not make the document shorter"
    );
    assert_eq!(
        error.to_string(),
        via_i18n::format(
            Locale::Zh,
            keys::MEMORY_DOCUMENT_TOO_LARGE,
            &[("name", "MEMORY.md"), ("max", "20")]
        )
    );
    assert!(!path.exists(), "nothing was written");
}

#[test]
fn a_read_over_the_cap_is_truncated_with_the_marker_but_keeps_its_revision() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("MEMORY.md");
    let body = "字".repeat(50);
    std::fs::write(&path, &body).expect("seeded");
    let store = MarkdownContextStore::builder()
        .file_path(&path)
        .scope("memory")
        .max_chars(10)
        .locale(Locale::Zh)
        .build();

    let read = store.read(OWNER);
    assert!(read.starts_with(&"字".repeat(10)));
    assert!(read.ends_with(t(Locale::Zh, keys::MEMORY_TRUNCATION_MARKER)));

    // The revision identifies the whole file, not the prefix the model saw, so
    // an edit against a tail the model never read is still refused.
    let documents = store.list(OWNER);
    assert_eq!(documents[0].revision, digest(&body));
    assert_ne!(documents[0].revision, digest(&read));
}

#[test]
fn a_store_with_no_path_reads_empty_and_refuses_to_persist() {
    let store = MarkdownContextStore::builder().scope("memory").build();
    assert_eq!(store.read("owner"), "");
    assert!(store.list("owner").is_empty());
    assert_eq!(store.path_for("owner"), None);

    let error = store
        .persist("owner", "body")
        .expect_err("there is nowhere to write");
    assert!(matches!(error, MemoryEditError::Unavailable));
    assert_eq!(error.to_string(), "memory document is unavailable");
    assert_eq!(error.code(), None);

    let health = store.health();
    assert!(health.ok);
    assert!(!health.configured);
}

#[test]
fn an_unreadable_document_warns_once_and_still_answers() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    // A directory where a file is expected: readable as an entry, not as text.
    let path = dir.path().join("MEMORY.md");
    std::fs::create_dir(&path).expect("a directory in the file's place");

    let warnings: Arc<std::sync::Mutex<Vec<DocumentWarning>>> = Arc::default();
    let sink = warnings.clone();
    let store = MarkdownContextStore::builder()
        .file_path(&path)
        .scope("memory")
        .locale(Locale::Zh)
        .now(Arc::new(|| 4242))
        .on_warning(Arc::new(move |warning| {
            sink.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(warning.clone());
        }))
        .build();

    assert_eq!(store.read(OWNER), "", "a voice session must still work");
    let health = store.health();
    assert!(!health.ok);
    assert!(health.configured);
    let warning = health.warning.expect("a warning was raised");
    assert_eq!(warning.at, 4242);
    assert!(warning.message.contains("MEMORY.md"));
    assert_eq!(
        warnings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len(),
        1
    );
}

#[test]
fn a_panicking_warning_sink_does_not_break_the_read() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("MEMORY.md");
    std::fs::create_dir(&path).expect("a directory in the file's place");
    let store = MarkdownContextStore::builder()
        .file_path(&path)
        .scope("memory")
        .on_warning(Arc::new(|_| panic!("a misbehaving diagnostics sink")))
        .build();

    // Diagnostics must not break the voice service.
    assert_eq!(store.read(OWNER), "");
    assert!(!store.health().ok);
}

#[test]
fn a_template_is_used_only_until_the_document_exists() {
    let fixture = fixture("user");
    // The template is the base an edit is applied to, so the first append
    // produces a document with the heading already in it.
    let result = fixture
        .store
        .edit(OWNER, &append("- 称呼：老大"))
        .expect("applies");
    assert!(result.document.content.starts_with("# USER"));
    assert_eq!(
        std::fs::read_to_string(&fixture.path).expect("readable"),
        "# USER\n\n- 称呼：老大\n"
    );
}

#[test]
fn a_crlf_fragment_matches_a_document_written_with_newlines() {
    let fixture = fixture("memory");
    fixture
        .store
        .edit(OWNER, &append("- 第一行\n- 第二行"))
        .expect("seeded");

    let result = fixture
        .store
        .edit(
            OWNER,
            &EditRequest {
                edits: vec![MarkdownEdit {
                    old_text: "- 第一行\r\n- 第二行".to_owned(),
                    new_text: "- 合并".to_owned(),
                }],
                ..EditRequest::default()
            },
        )
        .expect("CRLF folds to LF on both sides");
    assert_eq!(result.changed, 1);
    assert!(result.document.content.contains("- 合并"));
}

#[test]
fn a_nul_in_a_fragment_cannot_reach_the_file() {
    let fixture = fixture("memory");
    fixture
        .store
        .edit(OWNER, &append("- 内容"))
        .expect("seeded");
    let result = fixture
        .store
        .edit(
            OWNER,
            &EditRequest {
                edits: vec![MarkdownEdit {
                    old_text: "- 内\0容".to_owned(),
                    new_text: "- 新\0内容".to_owned(),
                }],
                ..EditRequest::default()
            },
        )
        .expect("NUL is stripped from both sides before matching");
    assert_eq!(result.changed, 1);
    let raw = std::fs::read_to_string(&fixture.path).expect("readable");
    assert!(!raw.contains('\0'));
    assert!(raw.contains("- 新内容"));
}

#[test]
fn surplus_edits_past_the_cap_are_dropped_rather_than_refused() {
    let fixture = fixture("memory");
    let seed: String = (0..25)
        .map(|index| format!("- 条目{index:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    fixture.store.edit(OWNER, &append(&seed)).expect("seeded");

    let edits: Vec<MarkdownEdit> = (0..25)
        .map(|index| MarkdownEdit {
            old_text: format!("- 条目{index:02}"),
            new_text: format!("- 改过{index:02}"),
        })
        .collect();
    let result = fixture
        .store
        .edit(
            OWNER,
            &EditRequest {
                edits,
                ..EditRequest::default()
            },
        )
        .expect("the first twenty apply");
    assert_eq!(
        result.changed,
        via_conversation::markdown_store::MAX_EDIT_ITEMS
    );
    assert!(result.document.content.contains("- 改过19"));
    assert!(result.document.content.contains("- 条目20"));
}

#[test]
fn normalize_markdown_deduplicates_across_headings() {
    // Document-wide deduplication is what makes "do not repeat what is already
    // covered" enforceable rather than advisory.
    assert_eq!(
        normalize_markdown("## A\n\n- 相同\n\n## B\n\n- 相同"),
        "## A\n\n- 相同\n\n## B"
    );
}

#[test]
fn a_document_at_exactly_the_cap_is_accepted() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let store = MarkdownContextStore::builder()
        .file_path(dir.path().join("MEMORY.md"))
        .scope("memory")
        // "# M" + "\n\n" + 5 characters = 10 code points.
        .max_chars(10)
        .template("# M")
        .build();
    let result = store
        .edit(OWNER, &append("字字字字字"))
        .expect("exactly at the cap is fine");
    assert_eq!(result.changed, 1);
    assert_eq!(result.document.content.chars().count(), 10);
}
