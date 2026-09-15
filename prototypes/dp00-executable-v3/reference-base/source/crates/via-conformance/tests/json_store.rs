//! `VersionedJsonStore`'s on-disk format.
//!
//! Every durable document VIA writes — `tasks.json`, `frontend-notes.json`,
//! `state/acp-sessions.json` — goes through one store, so one format contract
//! covers all of them. Three details in it are load-bearing and easy to lose:
//!
//! * `version` is the **first** key, before the spread payload, so a reader
//!   that streams the document sees the version before it sees anything that
//!   depends on it;
//! * two-space indent and a trailing newline, because these files are read and
//!   diffed by humans and by `git`;
//! * the staging file is `<path>.<pid>.tmp` — the pid is in the name so two
//!   processes writing the same document cannot share a staging file and
//!   corrupt each other's write.

use std::fs;
use std::sync::Arc;

use pretty_assertions::assert_eq;
use serde_json::{Value, json};

use via_conformance::expect_contract;
use via_store::{DEFAULT_VERSION, FILE_MODE, VersionedJsonStore};

/// A fixed epoch-millisecond clock, so the quarantine filename is a literal.
const FROZEN_NOW: i64 = 1_755_859_000_000;

#[test]
fn versioned_json_store_on_disk_format() {
    let contract = expect_contract("file-format", "VersionedJsonStore on-disk format");
    let value = &contract.exact_value;

    // The four clauses, quoted from the catalogue so a reworded contract fails
    // here rather than silently ceasing to be checked.
    assert!(
        value.contains("${JSON.stringify({version:<n>, ...value}, null, 2)}\\n"),
        "{value}"
    );
    assert!(
        value.contains("`${filePath}.${process.pid}.tmp`"),
        "{value}"
    );
    assert!(value.contains("at mode 0o600"), "{value}");
    assert!(
        value.contains("parent directory created recursive"),
        "{value}"
    );
    assert!(
        value.contains("`${filePath}.corrupt-${Date.now()}`"),
        "{value}"
    );

    let dir = tempfile::TempDir::new().expect("tempdir");
    // Nested, so "parent directory created recursive" is exercised rather than
    // assumed: neither `state/` nor `nested/` exists yet.
    let path = dir.path().join("nested").join("state").join("doc.json");
    let store = VersionedJsonStore::builder()
        .file_path(path.clone())
        .now(Arc::new(|| FROZEN_NOW))
        .build();

    assert_eq!(
        store.sync_temp_path(),
        Some(path.with_file_name(format!("doc.json.{}.tmp", std::process::id()))),
        "the staging name carries this process's pid"
    );

    // `b` before `a` on purpose: insertion order is preserved, and `version`
    // is inserted ahead of both.
    assert!(store.save(&json!({ "b": 2, "a": 1 })));
    let raw = fs::read_to_string(&path).expect("the parent directories were created");
    assert_eq!(
        raw, "{\n  \"version\": 1,\n  \"b\": 2,\n  \"a\": 1\n}\n",
        "two-space indent, `version` first, trailing newline"
    );
    assert_eq!(DEFAULT_VERSION, 1);
    assert!(
        !store
            .sync_temp_path()
            .expect("a configured store has a temp path")
            .exists(),
        "the staging file is renamed away, never left behind"
    );

    // Read back through the parser too, so the key order is asserted as data
    // and not only as bytes.
    let parsed: Value = serde_json::from_str(&raw).expect("valid JSON");
    assert_eq!(
        parsed
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["version", "b", "a"]
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&path).expect("metadata").permissions().mode() & 0o777,
            FILE_MODE,
            "mode 0o600"
        );
    }

    // A payload key named `version` cannot displace the store's own, and
    // cannot move it out of first position either — JS object spread
    // overwrites in place, and so does this.
    assert!(store.save(&json!({ "version": 99, "a": 1 })));
    let raw = fs::read_to_string(&path).expect("read");
    assert_eq!(raw, "{\n  \"version\": 99,\n  \"a\": 1\n}\n");

    // Quarantine. The filename embeds the clock, which is why the clock is
    // injectable at all.
    fs::write(&path, "{ not json").expect("corrupt the document");
    assert_eq!(
        store.load(|_| true),
        None,
        "a corrupt document reads as absent"
    );
    let quarantined = path.with_file_name(format!("doc.json.corrupt-{FROZEN_NOW}"));
    assert!(
        quarantined.exists(),
        "the original must be moved aside, not overwritten"
    );
    assert_eq!(
        fs::read_to_string(&quarantined).expect("read"),
        "{ not json",
        "quarantine preserves the bytes verbatim"
    );
    assert!(
        !path.exists(),
        "the corrupt document is renamed, not copied"
    );

    // A version mismatch takes the same path.
    assert!(store.save(&json!({ "a": 1 })));
    let store_v2 = VersionedJsonStore::builder()
        .file_path(path.clone())
        .version(2)
        .now(Arc::new(|| FROZEN_NOW + 1))
        .build();
    assert_eq!(store_v2.load(|_| true), None, "version 1 is not version 2");
    assert!(
        path.with_file_name(format!("doc.json.corrupt-{}", FROZEN_NOW + 1))
            .exists()
    );
}

#[test]
fn backend_session_state_file_format() {
    let contract = expect_contract("file-path", "backend session state file");
    let value = &contract.exact_value;

    // The half `via-store` owns: the serialization, the mode, the staging
    // name and the quarantine name are the store's, and are identical to the
    // generic contract asserted above.
    assert!(
        value.contains("JSON.stringify(...,null,2) plus a trailing newline"),
        "{value}"
    );
    assert!(value.contains("mode 0o600"), "{value}");
    assert!(value.contains("`<path>.<pid>.tmp`"), "{value}");
    assert!(value.contains("`<path>.corrupt-<Date.now()>`"), "{value}");
    assert!(value.contains("\"version\":1"), "{value}");
    assert_eq!(DEFAULT_VERSION, 1);

    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("state").join("acp-sessions.json");
    let store = VersionedJsonStore::builder()
        .file_path(path.clone())
        .build();
    assert!(store.save(&json!({
        "coordinators": { "k": { "sessionId": "", "cwd": "", "updatedAt": 0 } },
        "projects": {},
    })));
    let raw = fs::read_to_string(&path).expect("read");
    assert!(raw.starts_with("{\n  \"version\": 1,\n"), "{raw}");
    assert!(raw.ends_with("}\n"), "{raw}");
    let parsed: Value = serde_json::from_str(&raw).expect("valid JSON");
    assert_eq!(
        parsed
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["version", "coordinators", "projects"],
        "`version` first, then the payload in the order it was given"
    );

    // The half `via-store` does NOT own, named here so the Partial
    // classification states what is still owed. The registry itself, its
    // location under the configuration directory, its environment override and
    // the two record shapes are `via-acp`'s: upstream reads them in
    // `server/src/agent/acp-session-registry.mjs`, which is the ACP session
    // index, not the store.
    assert!(
        value.contains("resolve(runtimeEnvironment.configDirectory, 'state/acp-sessions.json')"),
        "{value}"
    );
    let upstream_override = "QWEN_AUDIO_AGENT_BACKEND_SESSION_STATE_PATH";
    assert!(value.contains(upstream_override), "{value}");
    assert_eq!(
        via_conformance::value::rebranded(upstream_override),
        "VIA_BACKEND_SESSION_STATE_PATH",
        "docs/rebrand.md renames the prefix; reading the variable is not this crate's job"
    );
    for owed in [
        "coordinators",
        "projects",
        "sessionId",
        "cwd",
        "updatedAt",
        "title",
    ] {
        assert!(
            value.contains(owed),
            "`{owed}` is no longer catalogued; the Partial row's remainder is stale"
        );
    }
    assert!(contract.file.contains("acp-session-registry.mjs"));
}
