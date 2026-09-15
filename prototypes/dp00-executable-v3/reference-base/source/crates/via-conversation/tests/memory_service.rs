//! [`FrontendMemoryService`] — ported from
//! `server/test/frontend-memory-service.test.mjs`, plus the authority rules the
//! two documents exist to keep apart.

use pretty_assertions::assert_eq;
use via_conversation::markdown_store::{MarkdownContextStore, MarkdownEdit};
use via_conversation::memory_service::{FrontendMemoryService, MemoryChange, MemoryServiceError};
use via_i18n::Locale;

const OWNER: &str = "user_personal";

struct Fixture {
    _dir: tempfile::TempDir,
    user: MarkdownContextStore,
    memory: MarkdownContextStore,
    service: FrontendMemoryService,
}

fn fixture() -> Fixture {
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
    let service = FrontendMemoryService::new(Some(user.clone()), Some(memory.clone()));
    Fixture {
        _dir: dir,
        user,
        memory,
        service,
    }
}

fn append(document: &str, text: &str) -> MemoryChange {
    MemoryChange {
        document: document.to_owned(),
        append: text.to_owned(),
        ..MemoryChange::default()
    }
}

#[test]
fn both_documents_are_listed_separately_and_reachable_by_alias() {
    let fixture = fixture();
    fixture
        .service
        .apply(OWNER, &[append("user", "- 称呼：船长")])
        .expect("user write");
    fixture
        .service
        .apply(OWNER, &[append("memory", "- 喜欢篮球")])
        .expect("memory write");

    let all = fixture.service.list(OWNER, None).expect("list");
    assert_eq!(
        all.iter()
            .map(|document| document.scope.as_str())
            .collect::<Vec<_>>(),
        vec!["user", "memory"],
        "the order is the scope table's order"
    );
    assert!(
        fixture.service.list(OWNER, Some("user")).expect("list")[0]
            .content
            .contains("船长")
    );
    assert!(
        fixture
            .service
            .list(OWNER, Some("long_term"))
            .expect("list")[0]
            .content
            .contains("篮球")
    );
}

#[test]
fn a_legacy_alias_writes_to_the_document_it_resolves_to() {
    let fixture = fixture();
    let result = fixture
        .service
        .apply(OWNER, &[append("profile", "## 称呼\n\n- 船长")])
        .expect("the alias resolves");

    assert_eq!(result.changed, 1);
    assert_eq!(result.documents.len(), 1);
    assert_eq!(result.documents[0].scope, "user");
    assert!(fixture.user.read(OWNER).contains("船长"));
    assert!(
        !fixture.memory.read(OWNER).contains("船长"),
        "a preference must never land in factual memory"
    );
}

#[test]
fn a_change_with_no_concrete_document_is_refused() {
    let fixture = fixture();
    assert_eq!(
        fixture.service.apply(OWNER, &[append("", "- 内容")]),
        Err(MemoryServiceError::NoConcreteDocument)
    );
    assert_eq!(
        fixture.service.apply(OWNER, &[append("all", "- 内容")]),
        Err(MemoryServiceError::NoConcreteDocument)
    );
    assert_eq!(
        fixture.service.apply(OWNER, &[]),
        Err(MemoryServiceError::NoChanges)
    );
    assert_eq!(fixture.user.read(OWNER), "");
    assert_eq!(fixture.memory.read(OWNER), "");
}

#[test]
fn an_unknown_document_is_refused_by_name() {
    let fixture = fixture();
    assert_eq!(
        fixture
            .service
            .apply(OWNER, &[append("assistant", "- 内容")]),
        Err(MemoryServiceError::UnsupportedScope {
            scope: "assistant".to_owned()
        }),
        "`ASSISTANT.md` is not reachable through the memory service"
    );
    assert_eq!(
        fixture.service.list(OWNER, Some("assistant")),
        Err(MemoryServiceError::UnsupportedScope {
            scope: "assistant".to_owned()
        })
    );
}

#[test]
fn two_changes_naming_the_same_document_are_refused() {
    let fixture = fixture();
    assert_eq!(
        fixture.service.apply(
            OWNER,
            &[append("memory", "- 第一"), append("facts", "- 第二")]
        ),
        Err(MemoryServiceError::DuplicateDocument {
            document: "memory".to_owned()
        }),
        "the alias resolves before the duplicate check, so it is caught"
    );
    assert_eq!(fixture.memory.read(OWNER), "");
}

#[test]
fn a_document_with_no_store_behind_it_is_refused() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let memory = MarkdownContextStore::builder()
        .file_path(dir.path().join("MEMORY.md"))
        .scope("memory")
        .build();
    let service = FrontendMemoryService::new(None, Some(memory));

    assert_eq!(
        service.apply(OWNER, &[append("user", "- 内容")]),
        Err(MemoryServiceError::DocumentUnavailable {
            document: "user".to_owned()
        })
    );
    // Listing everything simply omits the missing document.
    assert!(service.list(OWNER, None).expect("list").is_empty());
    // And health reports it as unconfigured rather than broken.
    let health = service.health();
    assert!(health.ok);
    assert_eq!(
        health.documents["user"]["configured"],
        serde_json::json!(false)
    );
    assert_eq!(
        health.documents["memory"]["configured"],
        serde_json::json!(true)
    );
}

#[test]
fn a_cross_document_reconciliation_applies_to_both_or_neither() {
    let fixture = fixture();
    fixture
        .service
        .apply(OWNER, &[append("memory", "- 用户希望被称呼为老板")])
        .expect("seeded");

    let result = fixture
        .service
        .apply(
            OWNER,
            &[
                append("user", "- 助手称呼用户：老大"),
                MemoryChange {
                    document: "memory".to_owned(),
                    edits: vec![MarkdownEdit {
                        old_text: "- 用户希望被称呼为老板".to_owned(),
                        new_text: String::new(),
                    }],
                    ..MemoryChange::default()
                },
            ],
        )
        .expect("both apply");

    assert_eq!(result.changed, 2);
    assert_eq!(result.documents.len(), 2);
    assert!(fixture.user.read(OWNER).contains("老大"));
    assert!(!fixture.memory.read(OWNER).contains("老板"));
}

#[test]
fn a_refusal_on_the_second_document_leaves_the_first_untouched() {
    let fixture = fixture();
    fixture
        .service
        .apply(OWNER, &[append("memory", "- 已有内容")])
        .expect("seeded");
    let before = fixture.user.read(OWNER);

    let error = fixture
        .service
        .apply(
            OWNER,
            &[
                append("user", "- 助手称呼用户：老大"),
                MemoryChange {
                    document: "memory".to_owned(),
                    edits: vec![MarkdownEdit {
                        old_text: "- 不存在的原文".to_owned(),
                        new_text: String::new(),
                    }],
                    ..MemoryChange::default()
                },
            ],
        )
        .expect_err("the second change is refused");

    match error {
        MemoryServiceError::Edit {
            code,
            retryable_read_again,
            ..
        } => {
            assert_eq!(code, Some("edit_not_found"));
            assert!(retryable_read_again);
        }
        other => panic!("expected an edit refusal, got {other:?}"),
    }
    assert_eq!(
        fixture.user.read(OWNER),
        before,
        "the first document is prepared but never persisted"
    );
    assert!(fixture.memory.read(OWNER).contains("- 已有内容"));
}

#[test]
fn a_stale_revision_on_either_document_stops_the_whole_apply() {
    let fixture = fixture();
    fixture
        .service
        .apply(OWNER, &[append("memory", "- 已有内容")])
        .expect("seeded");

    let error = fixture
        .service
        .apply(
            OWNER,
            &[MemoryChange {
                document: "memory".to_owned(),
                append: "- 新内容".to_owned(),
                expected_revision: "0000000000000000".to_owned(),
                ..MemoryChange::default()
            }],
        )
        .expect_err("the revision moved");
    match error {
        MemoryServiceError::Edit { code, .. } => assert_eq!(code, Some("stale_document")),
        other => panic!("expected a staleness refusal, got {other:?}"),
    }
    assert!(!fixture.memory.read(OWNER).contains("新内容"));
}

#[test]
fn an_unchanged_document_is_reported_without_being_rewritten() {
    let fixture = fixture();
    fixture
        .service
        .apply(OWNER, &[append("memory", "- 相同")])
        .expect("seeded");
    let first = fixture.memory.list(OWNER)[0].revision.clone();

    let result = fixture
        .service
        .apply(OWNER, &[append("memory", "- 相同")])
        .expect("the append deduplicates to nothing");
    assert_eq!(result.changed, 0);
    assert_eq!(fixture.memory.list(OWNER)[0].revision, first);
}

#[test]
fn health_is_not_ok_once_a_document_warns() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("MEMORY.md");
    std::fs::create_dir(&path).expect("a directory in the file's place");
    let memory = MarkdownContextStore::builder()
        .file_path(&path)
        .scope("memory")
        .build();
    let service = FrontendMemoryService::new(None, Some(memory));

    assert!(service.health().ok, "no read has happened yet");
    let _ = service.list(OWNER, Some("memory"));
    let health = service.health();
    assert!(!health.ok);
    assert_eq!(health.documents["memory"]["ok"], serde_json::json!(false));
    assert_eq!(health.documents["user"]["ok"], serde_json::json!(true));
}
