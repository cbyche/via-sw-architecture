//! Proof that the reader sees what cargo sees.
//!
//! The rules are tested against synthetic graphs in `src/rules.rs`. That leaves
//! one link unproven: whether [`CrateGraph::load_from`] turns a real workspace
//! into the edges those rules expect. So this file writes a small workspace to
//! a temporary directory — one that breaks three rules on purpose — reads it
//! back through `cargo metadata`, and checks the gate catches all three.
//!
//! It is deliberately the *whole* path: real manifests, real cargo, real
//! `[dev-dependencies]`, a renamed dependency and a `cfg`-gated optional one.
//! Those last two are exactly the shapes upstream's regex over import
//! statements could not see.

use std::fs;
use std::path::Path;

use pretty_assertions::assert_eq;
use via_arch_test::{
    BackendBinding, CrateGraph, EdgeKind, backend_bindings, band_violations, leaf_violations,
};

/// Write `<root>/<name>/{Cargo.toml,src/lib.rs}`.
fn write_crate(root: &Path, name: &str, dependency_tables: &str) {
    let directory = root.join(name);
    fs::create_dir_all(directory.join("src")).expect("create fixture crate directory");
    fs::write(
        directory.join("Cargo.toml"),
        format!(
            "[package]\n\
             name = \"{name}\"\n\
             version = \"0.0.0\"\n\
             edition = \"2024\"\n\
             publish = false\n\
             \n{dependency_tables}"
        ),
    )
    .expect("write fixture manifest");
    fs::write(directory.join("src").join("lib.rs"), "").expect("write fixture lib.rs");
}

/// A workspace that breaks the leaf rule, the band rule and the backend rule at
/// once, so one read exercises all three.
fn write_workspace(root: &Path) {
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\n\
         resolver = \"3\"\n\
         members = [\"via-protocol\", \"via-log\", \"via-acp\", \"via-backends\"]\n",
    )
    .expect("write fixture workspace manifest");

    write_crate(root, "via-protocol", "");

    // Leaf -> leaf. Upstream's row is `shared → ∅`.
    write_crate(
        root,
        "via-log",
        "[dependencies]\nvia-protocol = { path = \"../via-protocol\" }\n",
    );

    // The rule this crate exists for — and declared in the two shapes a regex
    // over source text is worst at: a dev-dependency, under an alias.
    write_crate(
        root,
        "via-acp",
        "[dev-dependencies]\n\
         named = { path = \"../via-backends\", package = \"via-backends\" }\n",
    );

    // An optional, target-gated edge is still an edge. A rule that holds only
    // on one platform, or only with default features, is not a rule.
    write_crate(
        root,
        "via-backends",
        "[target.'cfg(unix)'.dependencies]\n\
         via-protocol = { path = \"../via-protocol\", optional = true }\n",
    );
}

#[test]
fn the_reader_turns_a_real_workspace_into_the_edges_the_rules_expect() {
    let temporary = tempfile::tempdir().expect("create temporary directory");
    let root = temporary.path();
    write_workspace(root);

    let graph = CrateGraph::load_from(root).expect("read the fixture workspace");

    assert_eq!(
        graph.packages(),
        ["via-acp", "via-backends", "via-log", "via-protocol"],
        "every workspace member should be a node, sorted",
    );

    let edges: Vec<(&str, &str, EdgeKind)> = graph
        .edges()
        .iter()
        .map(|edge| (edge.from.as_str(), edge.to.as_str(), edge.kind))
        .collect();
    assert_eq!(
        edges,
        [
            // Read through the alias: `named = { package = "via-backends" }`.
            ("via-acp", "via-backends", EdgeKind::Dev),
            ("via-backends", "via-protocol", EdgeKind::Normal),
            ("via-log", "via-protocol", EdgeKind::Normal),
        ],
    );

    let gated = graph
        .edges_from("via-backends")
        .next()
        .expect("via-backends declares one edge");
    assert!(gated.optional, "the optional flag must survive the read");
    assert_eq!(
        gated.target.as_deref(),
        Some("cfg(unix)"),
        "the target gate must survive the read",
    );
    assert_eq!(
        gated.manifest, "via-backends/Cargo.toml",
        "the manifest path must be workspace-relative, so the message points at a file",
    );
}

#[test]
fn the_gate_catches_every_planted_violation_in_a_real_workspace() {
    let temporary = tempfile::tempdir().expect("create temporary directory");
    let root = temporary.path();
    write_workspace(root);

    let graph = CrateGraph::load_from(root).expect("read the fixture workspace");

    // Keyed on the *source* band: `via-backends -> via-protocol` also ends at
    // a leaf, but via-backends is layer3 and layer3 may depend on leaf.
    let leaf: Vec<String> = leaf_violations(&graph)
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(leaf, ["via-log -> via-protocol"]);

    let bands: Vec<String> = band_violations(&graph)
        .iter()
        .map(|violation| violation.edge.to_string())
        .collect();
    assert_eq!(bands, ["via-log -> via-protocol"]);

    // The point of the separate rule: `via-acp -> via-backends` is *inside* the
    // band table — both are layer3, and layer3 may depend on layer3 — so the
    // band rule cannot see it and upstream's regex over source text would not
    // have seen it either, aliased into a dev-dependency table.
    let bindings = backend_bindings(&graph);
    assert_eq!(bindings.len(), 1, "{bindings:?}");
    assert!(
        matches!(&bindings[0], BackendBinding::Direct { core, edge }
            if core == "via-acp" && edge.kind == EdgeKind::Dev),
        "a dev-dependency under an alias is still a binding: {:?}",
        bindings[0],
    );
    assert!(
        bindings[0].to_string().contains("dev-dependency"),
        "the message must name the table the edge was declared in: {}",
        bindings[0],
    );
}
