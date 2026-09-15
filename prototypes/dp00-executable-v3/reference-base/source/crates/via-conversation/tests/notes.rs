//! [`FrontendNotesStore`] — ported from `server/test/frontend-notes.test.mjs`,
//! plus the rollback, retention and shared-file paths.

use std::sync::{Arc, Mutex};

use pretty_assertions::assert_eq;
use via_conversation::notes::{
    AmbiguousItem, FrontendNotesStore, MAX_ITEMS_PER_LIST, MAX_LISTS_PER_OWNER, NotesError,
    NotesStatus, PublicItem,
};
use via_store::StoreWarning;

fn owned(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

fn items_of(status: &NotesStatus) -> Vec<String> {
    match status {
        NotesStatus::Show { items, .. } => items.iter().map(|item| item.text.clone()).collect(),
        other => panic!("expected a show result, got {other:?}"),
    }
}

fn list_names(store: &FrontendNotesStore, owner: &str) -> Vec<String> {
    store
        .lists(owner)
        .into_iter()
        .map(|entry| entry.list)
        .collect()
}

#[test]
fn adding_deduplicates_and_isolates_owners() {
    let store = FrontendNotesStore::in_memory();
    let result = store
        .add("owner-a", "购物清单", &owned(&["牛奶", "面包", "牛奶"]))
        .expect("add");

    assert_eq!(
        result,
        NotesStatus::Added {
            status: "ok",
            list: "购物清单".to_owned(),
            added: owned(&["牛奶", "面包"]),
            duplicates: owned(&["牛奶"]),
        }
    );

    store
        .add("owner-a", "购物清单", &owned(&["鸡蛋"]))
        .expect("add");
    assert_eq!(
        items_of(&store.show("owner-a", "购物清单")),
        owned(&["牛奶", "面包", "鸡蛋"])
    );
    assert_eq!(list_names(&store, "owner-a"), owned(&["购物清单"]));
    assert!(store.lists("owner-b").is_empty());
    assert!(matches!(
        store.show("owner-b", "购物清单"),
        NotesStatus::NotFound { .. }
    ));
}

#[test]
fn a_spoken_name_resolves_exactly_then_uniquely_then_asks() {
    let store = FrontendNotesStore::in_memory();
    store
        .add("owner-a", "购物清单", &owned(&["牛奶"]))
        .expect("add");

    // A unique substring resolves to the stored spelling.
    match store.show("owner-a", "购物") {
        NotesStatus::Show { list, .. } => assert_eq!(list, "购物清单"),
        other => panic!("expected a resolution, got {other:?}"),
    }
    match store.show("owner-a", "购物清单") {
        NotesStatus::Show { list, .. } => assert_eq!(list, "购物清单"),
        other => panic!("expected a resolution, got {other:?}"),
    }

    // A name that matches nothing reports what does exist.
    assert_eq!(
        store.show("owner-a", "书单"),
        NotesStatus::NotFound {
            status: "not_found",
            candidates: owned(&["购物清单"]),
        }
    );

    store
        .add("owner-a", "书单", &owned(&["三体"]))
        .expect("add");
    match store.show("owner-a", "书") {
        NotesStatus::Show { list, .. } => assert_eq!(list, "书单"),
        other => panic!("expected a resolution, got {other:?}"),
    }

    // Two matches is a question, never a guess.
    let ambiguous = store.show("owner-a", "单");
    match ambiguous {
        NotesStatus::Ambiguous { candidates, .. } => {
            let mut sorted = candidates;
            sorted.sort();
            assert_eq!(sorted, owned(&["书单", "购物清单"]));
        }
        other => panic!("expected ambiguity, got {other:?}"),
    }
}

#[test]
fn removing_matches_exactly_then_by_substring_and_reports_the_rest() {
    let store = FrontendNotesStore::in_memory();
    store
        .add("owner-a", "购物清单", &owned(&["牛奶", "酸奶", "面包"]))
        .expect("add");

    let result = store
        .remove("owner-a", "购物清单", &owned(&["牛", "酸奶", "咖啡"]))
        .expect("remove");
    match result {
        NotesStatus::Removed {
            status,
            removed,
            not_found,
            ambiguous,
            ..
        } => {
            assert_eq!(status, "ok");
            assert_eq!(removed, owned(&["牛奶", "酸奶"]));
            assert_eq!(not_found, owned(&["咖啡"]));
            assert!(ambiguous.is_empty());
        }
        other => panic!("expected a removal, got {other:?}"),
    }
    assert_eq!(
        items_of(&store.show("owner-a", "购物清单")),
        owned(&["面包"])
    );
}

#[test]
fn an_ambiguous_item_is_reported_with_its_candidates_and_nothing_is_removed() {
    let store = FrontendNotesStore::in_memory();
    store
        .add("owner-a", "购物清单", &owned(&["牛奶", "酸奶"]))
        .expect("add");

    let result = store
        .remove("owner-a", "购物清单", &owned(&["奶"]))
        .expect("remove");
    assert_eq!(
        result,
        NotesStatus::Removed {
            status: "ambiguous",
            list: "购物清单".to_owned(),
            removed: Vec::new(),
            not_found: Vec::new(),
            ambiguous: vec![AmbiguousItem {
                text: "奶".to_owned(),
                candidates: owned(&["牛奶", "酸奶"]),
            }],
        }
    );
    assert_eq!(
        items_of(&store.show("owner-a", "购物清单")),
        owned(&["牛奶", "酸奶"]),
        "an ambiguous removal destroys nothing"
    );
}

#[test]
fn a_partly_ambiguous_removal_still_reports_ambiguous() {
    let store = FrontendNotesStore::in_memory();
    store
        .add("owner-a", "购物清单", &owned(&["牛奶", "酸奶", "面包"]))
        .expect("add");

    let result = store
        .remove("owner-a", "购物清单", &owned(&["面包", "奶"]))
        .expect("remove");
    match result {
        NotesStatus::Removed {
            status,
            removed,
            ambiguous,
            ..
        } => {
            // The unambiguous half is applied; the ambiguous half comes back as
            // a question, and the status says a question is outstanding.
            assert_eq!(status, "ambiguous");
            assert_eq!(removed, owned(&["面包"]));
            assert_eq!(ambiguous.len(), 1);
        }
        other => panic!("expected a removal, got {other:?}"),
    }
}

#[test]
fn both_bounds_are_enforced() {
    let store = FrontendNotesStore::in_memory();
    let items: Vec<String> = (0..120).map(|index| format!("条目-{index}")).collect();
    let result = store.add("owner-a", "长清单", &items).expect("add");
    match result {
        NotesStatus::Added { added, .. } => assert_eq!(added.len(), MAX_ITEMS_PER_LIST),
        other => panic!("expected an add, got {other:?}"),
    }
    assert_eq!(
        items_of(&store.show("owner-a", "长清单")).len(),
        MAX_ITEMS_PER_LIST
    );

    for index in 1..MAX_LISTS_PER_OWNER {
        store
            .add("owner-a", &format!("清单-{index:02}"), &owned(&["占位"]))
            .expect("add");
    }
    assert_eq!(
        store
            .add("owner-a", "第二十一份", &owned(&["占位"]))
            .expect("add"),
        NotesStatus::ListFull {
            status: "list_full",
            list: "第二十一份".to_owned(),
        }
    );
}

#[test]
fn adding_to_a_full_list_reports_list_full_and_changes_nothing() {
    let store = FrontendNotesStore::in_memory();
    let items: Vec<String> = (0..MAX_ITEMS_PER_LIST)
        .map(|index| format!("条目-{index}"))
        .collect();
    store.add("owner-a", "满", &items).expect("add");

    let result = store
        .add("owner-a", "满", &owned(&["再一个"]))
        .expect("add");
    assert_eq!(
        result,
        NotesStatus::ListFull {
            status: "list_full",
            list: "满".to_owned(),
        }
    );
    assert_eq!(
        items_of(&store.show("owner-a", "满")).len(),
        MAX_ITEMS_PER_LIST
    );
}

#[test]
fn clear_empties_a_list_and_keeps_it_while_drop_removes_it() {
    let store = FrontendNotesStore::in_memory();
    store
        .add("owner-a", "购物清单", &owned(&["牛奶", "面包"]))
        .expect("add");

    assert_eq!(
        store.clear("owner-a", "购物清单").expect("clear"),
        NotesStatus::Cleared {
            status: "ok",
            list: "购物清单".to_owned(),
            removed: 2,
        }
    );
    assert!(items_of(&store.show("owner-a", "购物清单")).is_empty());
    assert_eq!(list_names(&store, "owner-a"), owned(&["购物清单"]));

    // Clearing again is a no-op that still reports success.
    assert_eq!(
        store.clear("owner-a", "购物清单").expect("clear"),
        NotesStatus::Cleared {
            status: "ok",
            list: "购物清单".to_owned(),
            removed: 0,
        }
    );

    store
        .add("owner-a", "书单", &owned(&["三体"]))
        .expect("add");
    assert_eq!(
        store.drop_list("owner-a", "购物清单").expect("drop"),
        NotesStatus::Dropped {
            status: "ok",
            list: "购物清单".to_owned(),
        }
    );
    assert_eq!(list_names(&store, "owner-a"), owned(&["书单"]));
    assert_eq!(
        store.drop_list("owner-a", "书单").expect("drop").status(),
        "ok"
    );
    assert!(store.lists("owner-a").is_empty());
}

#[test]
fn a_missing_name_is_a_question_for_every_action() {
    let store = FrontendNotesStore::in_memory();
    store
        .add("owner-a", "书单", &owned(&["三体"]))
        .expect("add");

    for status in [
        store.show("owner-a", "购物"),
        store
            .remove("owner-a", "购物", &owned(&["牛奶"]))
            .expect("remove"),
        store.clear("owner-a", "购物").expect("clear"),
        store.drop_list("owner-a", "购物").expect("drop"),
    ] {
        assert_eq!(
            status,
            NotesStatus::NotFound {
                status: "not_found",
                candidates: owned(&["书单"]),
            },
            "every action reports the same question"
        );
    }
    // And the list survived all four.
    assert_eq!(list_names(&store, "owner-a"), owned(&["书单"]));
}

#[test]
fn add_and_remove_refuse_an_empty_name_or_no_items() {
    let store = FrontendNotesStore::in_memory();
    assert_eq!(
        store.add("owner-a", "   ", &owned(&["牛奶"])),
        Err(NotesError::ListNameRequired)
    );
    assert_eq!(
        store.add("owner-a", "购物清单", &owned(&["", "  "])),
        Err(NotesError::ListItemsRequired)
    );
    assert_eq!(
        store.remove("owner-a", "购物清单", &[]),
        Err(NotesError::ListItemsRequired)
    );
}

#[test]
fn notes_persist_atomically_and_reload() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("frontend-notes.json");
    let first = FrontendNotesStore::builder().file_path(&path).build();
    first
        .add("owner-a", "购物清单", &owned(&["牛奶"]))
        .expect("add");

    let second = FrontendNotesStore::builder().file_path(&path).build();
    assert_eq!(
        items_of(&second.show("owner-a", "购物清单")),
        owned(&["牛奶"])
    );
    assert!(second.health().ok);
    assert!(second.health().persistence_enabled);
    assert_eq!(second.health().owners, 1);
}

#[test]
fn a_corrupt_file_is_quarantined_and_the_store_keeps_working() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("frontend-notes.json");
    std::fs::write(&path, "{not-json").expect("seeded");

    let warnings: Arc<Mutex<Vec<StoreWarning>>> = Arc::default();
    let sink = warnings.clone();
    let store = FrontendNotesStore::builder()
        .file_path(&path)
        .now(Arc::new(|| 12345))
        .on_warning(Arc::new(move |warning| {
            sink.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(warning.clone());
        }))
        .build();

    assert!(store.lists("owner-a").is_empty());
    assert_eq!(
        warnings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len(),
        1
    );
    let quarantined = dir.path().join("frontend-notes.json.corrupt-12345");
    assert!(quarantined.exists(), "the original was moved aside");
    assert_eq!(
        std::fs::read_to_string(&quarantined).expect("readable"),
        "{not-json",
        "and kept byte for byte"
    );

    // The store is still writable, and the warning stands.
    store
        .add("owner-a", "购物清单", &owned(&["牛奶"]))
        .expect("add");
    let health = store.health();
    assert!(!health.ok);
    assert!(health.persistence_enabled);
    assert!(
        health
            .warning
            .and_then(|warning| warning.quarantine_path)
            .is_some_and(|path| path.ends_with("corrupt-12345"))
    );
}

#[test]
fn a_file_with_the_wrong_version_is_quarantined() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("frontend-notes.json");
    std::fs::write(&path, r#"{"version":99,"owners":{}}"#).expect("seeded");
    let store = FrontendNotesStore::builder()
        .file_path(&path)
        .now(Arc::new(|| 7))
        .build();

    assert!(store.lists("owner-a").is_empty());
    assert!(dir.path().join("frontend-notes.json.corrupt-7").exists());
    assert!(!store.health().ok);
}

#[test]
fn a_file_with_no_owners_object_is_quarantined() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("frontend-notes.json");
    std::fs::write(&path, r#"{"version":1,"owners":[]}"#).expect("seeded");
    let store = FrontendNotesStore::builder()
        .file_path(&path)
        .now(Arc::new(|| 8))
        .build();

    assert!(store.lists("owner-a").is_empty());
    assert!(dir.path().join("frontend-notes.json.corrupt-8").exists());
}

#[test]
fn an_addition_is_rolled_back_when_persistence_is_unavailable() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    // A directory in the file's place: every write fails.
    let path = dir.path().join("frontend-notes.json");
    std::fs::create_dir(&path).expect("a directory in the file's place");

    let store = FrontendNotesStore::builder().file_path(&path).build();
    let error = store
        .add("owner-a", "购物清单", &owned(&["牛奶"]))
        .expect_err("the write fails");
    assert!(matches!(error, NotesError::PersistenceUnavailable(_)));

    // Nothing survived in memory either, so a later read cannot report a list
    // that is not on disk.
    assert!(store.lists("owner-a").is_empty());
    assert!(!store.health().persistence_enabled);
}

#[test]
fn a_removal_is_rolled_back_when_the_save_fails_mid_session() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("frontend-notes.json");
    let store = FrontendNotesStore::builder().file_path(&path).build();
    store
        .add("owner-a", "购物清单", &owned(&["牛奶", "面包"]))
        .expect("add");

    // Occupy the staging name `via-store` writes through. The reload still
    // succeeds and the lock is still available, so this is the narrow window
    // the rollback exists for: a save that fails after the state changed in
    // memory.
    let temp = dir
        .path()
        .join(format!("frontend-notes.json.{}.tmp", std::process::id()));
    std::fs::create_dir(&temp).expect("a directory in the temp file's place");

    let error = store.remove("owner-a", "购物清单", &owned(&["牛奶"]));
    assert_eq!(
        error,
        Err(NotesError::PersistenceUnavailable(via_i18n::t(
            via_i18n::Locale::En,
            via_i18n::keys::STORE_NOTES_UNAVAILABLE
        )))
    );

    std::fs::remove_dir(&temp).expect("removable");
    // The item is still on disk, and the store never told anyone otherwise.
    let reloaded = FrontendNotesStore::builder().file_path(&path).build();
    assert_eq!(
        items_of(&reloaded.show("owner-a", "购物清单")),
        owned(&["牛奶", "面包"])
    );
}

#[cfg(unix)]
#[test]
fn a_transaction_that_cannot_take_the_lock_is_reported_as_such() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::TempDir::new().expect("tempdir");
    let inner = dir.path().join("profile");
    std::fs::create_dir(&inner).expect("mkdir");
    let path = inner.join("frontend-notes.json");
    let store = FrontendNotesStore::builder().file_path(&path).build();
    store.add("owner-a", "l", &owned(&["i"])).expect("add");

    std::fs::set_permissions(&inner, std::fs::Permissions::from_mode(0o555)).expect("chmod");
    let error = store.add("owner-a", "l", &owned(&["j"]));
    std::fs::set_permissions(&inner, std::fs::Permissions::from_mode(0o755)).expect("chmod back");

    assert!(
        matches!(error, Err(NotesError::Lock(_))),
        "a lock that cannot be taken is not a write failure: {error:?}"
    );
}

#[test]
fn a_file_that_became_unreadable_disables_persistence_rather_than_clobbering() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("frontend-notes.json");
    let store = FrontendNotesStore::builder().file_path(&path).build();
    store
        .add("owner-a", "购物清单", &owned(&["牛奶", "面包"]))
        .expect("add");

    // A directory where the document was. It is not corrupt — it may not even
    // be ours — so it is never quarantined; persistence goes off instead.
    std::fs::remove_file(&path).expect("removable");
    std::fs::create_dir(&path).expect("a directory in the file's place");

    let result = store
        .remove("owner-a", "购物清单", &owned(&["牛奶"]))
        .expect("the transaction completes");
    assert!(matches!(result, NotesStatus::NotFound { .. }));
    let health = store.health();
    assert!(!health.persistence_enabled);
    assert!(!health.ok);
    assert!(path.is_dir(), "the thing we could not read was left alone");
}

#[test]
fn two_stores_sharing_one_file_see_each_others_writes() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("frontend-notes.json");
    // The Desktop and CLI Gateways are two processes over one file. Two store
    // instances reproduce the same hazard in one process.
    let cli = FrontendNotesStore::builder().file_path(&path).build();
    let desktop = FrontendNotesStore::builder().file_path(&path).build();

    cli.add("owner-a", "购物清单", &owned(&["牛奶"]))
        .expect("add");
    desktop
        .add("owner-a", "购物清单", &owned(&["面包"]))
        .expect("add");

    assert_eq!(
        items_of(&cli.show("owner-a", "购物清单")),
        owned(&["牛奶", "面包"]),
        "the CLI must not overwrite the desktop's write with its stale cache"
    );

    cli.add("owner-a", "购物清单", &owned(&["鸡蛋"]))
        .expect("add");
    assert_eq!(
        items_of(&desktop.show("owner-a", "购物清单")),
        owned(&["牛奶", "面包", "鸡蛋"])
    );
}

#[test]
fn a_hand_edited_file_cannot_smuggle_content_past_the_bounds() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("frontend-notes.json");
    let long_item = "字".repeat(400);
    let long_name = "名".repeat(80);
    let mut items = Vec::new();
    for index in 0..300 {
        items.push(serde_json::json!({
            "id": format!("item_{index:012}"),
            "text": format!("条目{index}"),
            "addedAt": 1,
        }));
    }
    items.push(serde_json::json!({ "text": long_item }));
    let document = serde_json::json!({
        "version": 1,
        "owners": {
            "owner-a": {
                long_name.clone(): { "name": long_name, "items": items, "createdAt": 1, "updatedAt": 1 },
            },
        },
        "ownerAccess": { "owner-a": 1 },
    });
    std::fs::write(&path, serde_json::to_string(&document).expect("json")).expect("seeded");

    let store = FrontendNotesStore::builder().file_path(&path).build();
    let lists = store.lists("owner-a");
    assert_eq!(lists.len(), 1);
    assert_eq!(lists[0].list.chars().count(), 30, "the name is re-bounded");
    assert_eq!(lists[0].count, MAX_ITEMS_PER_LIST, "so are the items");
}

#[test]
fn an_owner_past_the_cap_is_evicted_least_recently_used_first() {
    let mut clock = 0i64;
    let ticks = Arc::new(Mutex::new(0i64));
    let hand = ticks.clone();
    let store = FrontendNotesStore::builder()
        .max_owners(2)
        .now(Arc::new(move || {
            let mut guard = hand.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            *guard += 1;
            *guard
        }))
        .build();
    clock += 1;
    let _ = clock;

    for owner in ["a", "b", "c"] {
        store.add(owner, "l", &owned(&["i"])).expect("add");
    }
    // `a` was touched longest ago, so it is the one that went.
    assert!(store.lists("a").is_empty());
    assert!(!store.lists("b").is_empty());
    assert!(!store.lists("c").is_empty());
}

#[test]
fn an_expired_owner_is_dropped_once_the_ttl_passes() {
    let now = Arc::new(Mutex::new(1_000i64));
    let hand = now.clone();
    let store = FrontendNotesStore::builder()
        .owner_ttl_ms(100)
        .now(Arc::new(move || {
            *hand.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
        }))
        .build();
    store.add("owner-a", "l", &owned(&["i"])).expect("add");
    assert!(!store.lists("owner-a").is_empty());

    *now.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = 1_100;
    assert!(store.lists("owner-a").is_empty(), "the TTL is inclusive");
}

#[test]
fn a_zero_ttl_keeps_lists_forever() {
    let now = Arc::new(Mutex::new(1_000i64));
    let hand = now.clone();
    let store = FrontendNotesStore::builder()
        .owner_ttl_ms(0)
        .now(Arc::new(move || {
            *hand.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
        }))
        .build();
    store.add("owner-a", "l", &owned(&["i"])).expect("add");

    *now.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = i64::from(u32::MAX);
    assert_eq!(list_names(&store, "owner-a"), owned(&["l"]));
}

#[test]
fn the_lists_result_is_ordered_by_most_recent_change() {
    let now = Arc::new(Mutex::new(1_000i64));
    let hand = now.clone();
    let store = FrontendNotesStore::builder()
        .now(Arc::new(move || {
            *hand.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
        }))
        .build();
    store.add("owner-a", "第一", &owned(&["a"])).expect("add");
    *now.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = 2_000;
    store.add("owner-a", "第二", &owned(&["b"])).expect("add");
    assert_eq!(list_names(&store, "owner-a"), owned(&["第二", "第一"]));

    *now.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = 3_000;
    store.add("owner-a", "第一", &owned(&["c"])).expect("add");
    assert_eq!(list_names(&store, "owner-a"), owned(&["第一", "第二"]));
}

#[test]
fn a_shown_item_id_is_derived_from_its_text() {
    let store = FrontendNotesStore::in_memory();
    store.add("owner-a", "l", &owned(&["牛奶"])).expect("add");
    match store.show("owner-a", "l") {
        NotesStatus::Show { items, .. } => assert_eq!(
            items,
            vec![PublicItem {
                id: via_conversation::notes::item_id("牛奶"),
                text: "牛奶".to_owned(),
            }]
        ),
        other => panic!("expected a show result, got {other:?}"),
    }
}

#[test]
fn an_item_that_collapses_to_nothing_is_dropped_before_the_bound() {
    let store = FrontendNotesStore::in_memory();
    let result = store
        .add("owner-a", "l", &owned(&["  \n\t ", "牛奶"]))
        .expect("add");
    match result {
        NotesStatus::Added { added, .. } => assert_eq!(added, owned(&["牛奶"])),
        other => panic!("expected an add, got {other:?}"),
    }
}

#[test]
fn independent_gateways_writing_at_once_lose_nothing() {
    // Upstream spawns twelve Node processes over one file. Twelve threads with
    // twelve independent stores reproduce the same hazard: the `mkdir`
    // transaction is what serializes them, and the reload-inside-the-lock is
    // what stops each one's stale cache overwriting the rest.
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("frontend-notes.json");
    let items: Vec<String> = (0..12).map(|index| format!("item-{index}")).collect();

    std::thread::scope(|scope| {
        for item in &items {
            let path = path.clone();
            scope.spawn(move || {
                FrontendNotesStore::builder()
                    .file_path(&path)
                    .build()
                    .add("owner-a", "shared", std::slice::from_ref(item))
                    .expect("add");
            });
        }
    });

    let mut stored = items_of(
        &FrontendNotesStore::builder()
            .file_path(&path)
            .build()
            .show("owner-a", "shared"),
    );
    stored.sort();
    let mut expected = items;
    expected.sort();
    assert_eq!(stored, expected);
}

#[test]
fn dropping_an_owners_last_list_removes_the_owner() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("frontend-notes.json");
    let store = FrontendNotesStore::builder().file_path(&path).build();
    store
        .add("owner-a", "书单", &owned(&["三体"]))
        .expect("add");
    store
        .add("owner-b", "书单", &owned(&["三体"]))
        .expect("add");
    assert_eq!(store.health().owners, 2);

    store.drop_list("owner-a", "书单").expect("drop");
    assert_eq!(
        store.health().owners,
        1,
        "an owner with no lists is not retained"
    );

    // And the owner is gone from the file too, not just from memory.
    let document: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("readable")).expect("json");
    let owners = document["owners"].as_object().expect("an owners map");
    assert!(!owners.contains_key("owner-a"));
    assert!(owners.contains_key("owner-b"));
}

#[test]
fn dropping_a_list_keeps_the_order_of_the_rest() {
    // The file's key order is insertion order, and it is observable twice: in
    // the document itself, and in the capped candidate list an ambiguous name
    // reports. A removal must not reshuffle it.
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("frontend-notes.json");
    let store = FrontendNotesStore::builder()
        .file_path(&path)
        .now(Arc::new(|| 1_000))
        .build();
    for name in ["清单甲", "清单乙", "清单丙", "清单丁"] {
        store.add("owner-a", name, &owned(&["占位"])).expect("add");
    }

    store.drop_list("owner-a", "清单乙").expect("drop");

    let document: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("readable")).expect("json");
    assert_eq!(
        document["owners"]["owner-a"]
            .as_object()
            .expect("a lists map")
            .keys()
            .collect::<Vec<_>>(),
        vec!["清单甲", "清单丙", "清单丁"]
    );
    assert_eq!(
        list_names(&store, "owner-a"),
        owned(&["清单甲", "清单丙", "清单丁"])
    );

    // And a reload sees the same order, so two Gateways agree on candidates.
    let reloaded = FrontendNotesStore::builder().file_path(&path).build();
    match reloaded.show("owner-a", "清单") {
        NotesStatus::Ambiguous { candidates, .. } => {
            assert_eq!(candidates, owned(&["清单甲", "清单丙", "清单丁"]));
        }
        other => panic!("expected ambiguity, got {other:?}"),
    }
}

#[test]
fn health_is_reachable_from_inside_a_warning_sink() {
    // `VersionedJsonStore` invokes the sink while this store holds its cache
    // guard across a load. A Gateway logger that wants health context must not
    // deadlock the process for asking.
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("frontend-notes.json");
    std::fs::write(&path, "{not-json").expect("seeded");

    let seen: Arc<Mutex<Vec<usize>>> = Arc::default();
    let recorded = seen.clone();
    let handle: Arc<Mutex<Option<FrontendNotesStore>>> = Arc::default();
    let inner = handle.clone();
    let store = FrontendNotesStore::builder()
        .file_path(&path)
        .on_warning(Arc::new(move |_warning| {
            if let Some(store) = inner
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .as_ref()
            {
                recorded
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .push(store.health().owners);
            }
        }))
        .build();
    *handle
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(store.clone());

    // Force a second warning, this time with the sink able to see the store.
    std::fs::write(&path, "{still-not-json").expect("re-seeded");
    store.add("owner-a", "l", &owned(&["i"])).expect("add");

    assert_eq!(
        seen.lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len(),
        1,
        "the sink ran and returned"
    );
}
