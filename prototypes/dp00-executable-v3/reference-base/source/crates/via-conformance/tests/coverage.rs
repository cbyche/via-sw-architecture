//! The tests that stop the catalogue from rotting.
//!
//! Everything else in this crate asserts a value. These four assert that the
//! *set* of assertions is honestly described — which is the harder half, since
//! an incomplete gate that reports itself as complete is worse than no gate.
//!
//! - [`every_contract_is_classified`] — a contract added to
//!   `docs/reference/contracts.json` without a registry row does not compile
//!   past this test. There is no default classification, so nothing can be
//!   added and forgotten.
//! - [`registry_has_no_dead_rows`] — a row naming a contract the catalogue no
//!   longer carries also fails, so the table cannot quietly accumulate
//!   assertions about nothing.
//! - [`asserting_tests_exist`] — every `tests/<file>.rs::<fn>` the registry
//!   names is really a `#[test]` in this crate. Renaming a test without
//!   updating the row fails here rather than silently orphaning a contract.
//! - [`behavioural_tests_exist`] — the same check for the rows whose comparison
//!   lives in *another* crate. Those references are the only thing standing
//!   between "asserted over there" and a sentence nobody can check, so they are
//!   resolved against the tree and required to be real `#[test]`s.
//! - [`coverage_summary`] and [`gaps_in_shipped_crates`] — `insta` snapshots,
//!   so the numbers moving is a reviewable diff.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use via_conformance::registry::REGISTRY;
use via_conformance::{Coverage, contracts};

/// The test sources, compiled in so `asserting_tests_exist` can look inside
/// them without a filesystem walk that could quietly find nothing.
const TEST_SOURCES: &[(&str, &str)] = &[
    ("tests/coverage.rs", include_str!("coverage.rs")),
    ("tests/wire_events.rs", include_str!("wire_events.rs")),
    (
        "tests/gateway_protocol.rs",
        include_str!("gateway_protocol.rs"),
    ),
    ("tests/work_lifecycle.rs", include_str!("work_lifecycle.rs")),
    (
        "tests/realtime_catalog.rs",
        include_str!("realtime_catalog.rs"),
    ),
    (
        "tests/backend_catalog.rs",
        include_str!("backend_catalog.rs"),
    ),
    ("tests/client_input.rs", include_str!("client_input.rs")),
    ("tests/gateway_lease.rs", include_str!("gateway_lease.rs")),
    ("tests/instance_locks.rs", include_str!("instance_locks.rs")),
    ("tests/log_records.rs", include_str!("log_records.rs")),
    ("tests/json_store.rs", include_str!("json_store.rs")),
    (
        "tests/frontend_prompt.rs",
        include_str!("frontend_prompt.rs"),
    ),
    (
        "tests/voice_constants.rs",
        include_str!("voice_constants.rs"),
    ),
    (
        "tests/localized_leaf_messages.rs",
        include_str!("localized_leaf_messages.rs"),
    ),
    ("tests/core_other.rs", include_str!("core_other.rs")),
    ("tests/voice_wire.rs", include_str!("voice_wire.rs")),
    (
        "tests/realtime_providers.rs",
        include_str!("realtime_providers.rs"),
    ),
    ("tests/acp_wire.rs", include_str!("acp_wire.rs")),
    ("tests/backends_paths.rs", include_str!("backends_paths.rs")),
    (
        "tests/core_environment.rs",
        include_str!("core_environment.rs"),
    ),
    ("tests/app_http.rs", include_str!("app_http.rs")),
    (
        "tests/apps_via_packaging.rs",
        include_str!("apps_via_packaging.rs"),
    ),
];

/// The workspace root, from this crate's manifest directory.
///
/// `include_str!` is not available for [`behavioural_tests_exist`] — the files
/// it reads are named by the registry at runtime, not by a literal — so the one
/// filesystem walk in this crate is here. It is not a *search*: every path is
/// derived from a registry row and a missing file is a failure, so there is no
/// way for it to quietly find nothing.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn every_contract_is_classified() {
    let classified: BTreeSet<(&str, &str)> = REGISTRY
        .iter()
        .map(|entry| (entry.kind, entry.name))
        .collect();

    let mut missing: Vec<String> = Vec::new();
    for contract in contracts() {
        if !classified.contains(&(contract.kind.as_str(), contract.name.as_str())) {
            missing.push(format!(
                "    Entry::pending({:?}, {:?}, Crate::???),  // {}",
                contract.kind, contract.name, contract.file
            ));
        }
    }
    missing.dedup();

    assert!(
        missing.is_empty(),
        "{} catalogued contract(s) have no row in crates/via-conformance/src/registry_rows.rs.\n\
         Every contract must be either asserted or explicitly pending against the crate that \
         will own it — an unclassified contract reads as conformant, which is the failure this \
         test exists to prevent.\n\nAdd:\n{}",
        missing.len(),
        missing.join("\n")
    );
}

#[test]
fn registry_has_no_dead_rows() {
    let catalogued: BTreeSet<(&str, &str)> = contracts()
        .iter()
        .map(|contract| (contract.kind.as_str(), contract.name.as_str()))
        .collect();

    let dead: Vec<String> = REGISTRY
        .iter()
        .filter(|entry| !catalogued.contains(&(entry.kind, entry.name)))
        .map(|entry| format!("    {}/{}", entry.kind, entry.name))
        .collect();
    assert!(
        dead.is_empty(),
        "{} registry row(s) name a contract docs/reference/contracts.json no longer has:\n{}",
        dead.len(),
        dead.join("\n")
    );

    // One row per key, so a status cannot be stated twice and disagree.
    let mut seen: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for entry in REGISTRY {
        *seen.entry((entry.kind, entry.name)).or_default() += 1;
    }
    let duplicated: Vec<String> = seen
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|((kind, name), count)| format!("    {kind}/{name} ×{count}"))
        .collect();
    assert!(
        duplicated.is_empty(),
        "duplicate registry rows:\n{}",
        duplicated.join("\n")
    );

    assert_eq!(REGISTRY.len(), catalogued.len(), "one row per contract key");
}

#[test]
fn asserting_tests_exist() {
    let sources: BTreeMap<&str, &str> = TEST_SOURCES.iter().copied().collect();

    for entry in REGISTRY {
        let Some(test) = entry.coverage.in_crate_test() else {
            continue;
        };
        let Some((file, function)) = test.split_once("::") else {
            panic!(
                "`{test}` is not `tests/<file>.rs::<fn>` ({}/{})",
                entry.kind, entry.name
            );
        };
        let Some(source) = sources.get(file) else {
            panic!(
                "`{test}` names `{file}`, which is not one of this crate's test sources \
                 ({}/{})",
                entry.kind, entry.name
            );
        };
        let needle = format!("\nfn {function}(");
        let at = source.find(&needle).unwrap_or_else(|| {
            panic!(
                "`{file}` has no `fn {function}(`, but the registry says it asserts {}/{}",
                entry.kind, entry.name
            )
        });
        assert!(
            source[..at].trim_end().ends_with("#[test]"),
            "`{function}` in `{file}` is not annotated `#[test]`"
        );
    }
}

/// Every [`Coverage::Behavioural`] row names a real `#[test]` in the tree.
///
/// The format is `<package>: <path>::<fn>`, where `<package>` is a directory
/// under `crates/` or the literal `apps/via`, and `<path>` is relative to that
/// package. Resolving all three parts is what makes a `Behavioural` row a
/// claim somebody can check rather than a sentence: it is the only guard on
/// 261 references into fifteen other crates' test suites, and without it a
/// renamed test would silently orphan every contract that points at it.
#[test]
fn behavioural_tests_exist() {
    let root = workspace_root();
    let mut sources: BTreeMap<PathBuf, String> = BTreeMap::new();
    let mut checked = 0usize;

    for entry in REGISTRY {
        let Coverage::Behavioural { test } = entry.coverage else {
            continue;
        };
        let Some((package, rest)) = test.split_once(": ") else {
            panic!(
                "`{test}` is not `<package>: <path>::<fn>` ({}/{})",
                entry.kind, entry.name
            );
        };
        let Some((file, function)) = rest.rsplit_once("::") else {
            panic!("`{test}` names no function ({}/{})", entry.kind, entry.name);
        };
        // The asserting package is deliberately *not* required to be the
        // owning one: a Chinese message owned by `via-i18n` is compared where
        // it is rendered, and a payload owned by `via-voice` is compared where
        // the frame is built.
        let path = if package == "apps/via" {
            root.join("apps").join("via").join(file)
        } else {
            root.join("crates").join(package).join(file)
        };
        if !sources.contains_key(&path) {
            let read = std::fs::read_to_string(&path).unwrap_or_else(|error| {
                panic!(
                    "`{test}` names {}, which cannot be read: {error} ({}/{})",
                    path.display(),
                    entry.kind,
                    entry.name
                )
            });
            sources.insert(path.clone(), read);
        }
        let source = &sources[&path];

        let at = ["\nfn ", "\n    fn ", "\nasync fn ", "\n    async fn "]
            .iter()
            .find_map(|prefix| source.find(&format!("{prefix}{function}(")))
            .unwrap_or_else(|| {
                panic!(
                    "`{}` has no `fn {function}(`, but the registry says it owns {}/{}",
                    path.display(),
                    entry.kind,
                    entry.name
                )
            });
        let head = &source[at.saturating_sub(400)..at];
        assert!(
            head.contains("#[test]") || head.contains("#[tokio::test"),
            "`{function}` in `{}` is not annotated `#[test]` ({}/{})",
            path.display(),
            entry.kind,
            entry.name
        );
        checked += 1;
    }

    // A floor, so a registry that lost every behavioural row would fail here
    // rather than pass by checking nothing.
    assert!(
        checked > 300,
        "only {checked} behavioural rows were checked; the registry has shrunk"
    );
}

#[test]
fn coverage_summary() {
    insta::assert_snapshot!(via_conformance::summary::coverage_summary().to_string());
}

/// The gaps that could be closed today.
///
/// A [`Coverage::Pending`] row against a crate that already exists is a
/// different thing from one waiting on a crate nobody has written: the first is
/// work available now, the second is a plan. Snapshotting the first keeps it
/// from disappearing into the second's much larger number.
#[test]
fn gaps_in_shipped_crates() {
    let mut lines: Vec<String> = REGISTRY
        .iter()
        .filter(|entry| entry.coverage == Coverage::Pending && entry.owner.exists_today())
        .map(|entry| format!("{}  {}/{}", entry.owner, entry.kind, entry.name))
        .collect();
    lines.sort();
    insta::assert_snapshot!(lines.join("\n"));
}
