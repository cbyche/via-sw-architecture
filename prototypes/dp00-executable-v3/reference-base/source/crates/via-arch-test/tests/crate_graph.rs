//! The gate itself: `docs/architecture.md` §9 asserted against real
//! `cargo metadata` edges.
//!
//! Every assertion here collects *all* of its violations before failing. A
//! gate that reports the first offending edge and stops turns one bad merge
//! into five red CI runs.
//!
//! The rules themselves live in the library, where they are exercised against
//! synthetic graphs — including graphs cargo would refuse to resolve. These
//! tests point them at the workspace.

use std::collections::BTreeSet;

use pretty_assertions::assert_eq;
use via_arch_test::{
    Band, CRATE_BANDS, CrateGraph, RULE_SOURCE, UPSTREAM_ADJACENCY, backend_bindings,
    band_violations, bullet_list, leaf_violations, non_via_members, unclassified_packages,
};

/// Read the workspace once per test. `cargo metadata --no-deps` is cheap and
/// needs no lockfile, registry or network.
fn graph() -> CrateGraph {
    match CrateGraph::load() {
        Ok(graph) => graph,
        Err(error) => panic!(
            "could not read the crate graph: {error}\n\
             caused by: {source}\n\n\
             This is a broken manifest somewhere in the workspace, not a layering \
             violation. Fix the manifest and the gate will have something to check.",
            source = std::error::Error::source(&error)
                .map_or_else(|| "(no cause)".to_owned(), ToString::to_string),
        ),
    }
}

#[test]
fn the_gate_can_see_the_workspace() {
    let graph = graph();
    assert!(
        graph.packages().contains(&"via-arch-test".to_owned()),
        "the gate did not find itself in the workspace at {root}; \
         workspace_root() resolved somewhere unexpected. Members seen: {members:?}",
        root = graph.root().display(),
        members = graph.packages(),
    );
}

/// The ratchet. §9 lists 25 crates plus one binary and three test crates; most
/// do not exist yet. Asserting the table against a future would fail today;
/// asserting that everything in the *present* is classified makes the gate
/// tighten as crates land instead of rotting.
#[test]
fn every_via_package_in_the_workspace_is_classified() {
    let unclassified = unclassified_packages(&graph());

    assert!(
        unclassified.is_empty(),
        "{count} workspace package(s) are in no layer band:\n\n{list}\n\n\
         A new VIA crate must be classified before it can be depended on. Add it to \
         CRATE_BANDS in crates/via-arch-test/src/table.rs *and* to the band table in \
         {RULE_SOURCE} — tests/architecture_doc.rs asserts the two agree.",
        count = unclassified.len(),
        list = unclassified
            .iter()
            .map(|package| format!("  {package}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

/// Every rule in this crate is keyed on a package name, so a member outside the
/// `via` / `via-*` convention would be invisible to all of them.
#[test]
fn every_workspace_member_is_a_via_package() {
    let strangers = non_via_members(&graph());

    assert!(
        strangers.is_empty(),
        "workspace member(s) outside the `via` / `via-*` naming convention: {strangers:?}\n\n\
         Every rule in via-arch-test is keyed on the package name, so a member named \
         otherwise is exempt from all of them. Rename it, or teach is_via_package() \
         about it deliberately. See {RULE_SOURCE}.",
    );
}

/// The main rule. Upstream walked import statements with a regex; this walks
/// resolver edges, so a re-export, a macro or a renamed dependency cannot hide
/// one.
#[test]
fn via_dependencies_stay_inside_their_band() {
    let violations = band_violations(&graph());

    assert!(
        violations.is_empty(),
        "{count} crate-graph violation(s):\n\n{list}\n\n\
         The allowed sets are derived from upstream's own adjacency table \
         (server/test/dependency-boundaries.test.mjs:10-18), quoted in {RULE_SOURCE}. \
         If an edge here is genuinely right, the band table is what changes — never \
         this assertion.",
        count = violations.len(),
        list = bullet_list(&violations, "\n\n"),
    );
}

/// The single most valuable assertion in the crate.
///
/// Upstream's version is a regex for `openclaw|opencode|qoder|qwen|kimi|...`
/// over five specific source files
/// (`dependency-boundaries.test.mjs:60-73`). Here it is a crate boundary: the
/// generic ACP client and the generic managed-process core cannot depend on the
/// crate that holds the named backends, so they cannot name one.
#[test]
fn via_acp_and_via_process_do_not_depend_on_via_backends() {
    let bindings = backend_bindings(&graph());

    assert!(
        bindings.is_empty(),
        "the generic cores are bound to the named-backend registry:\n\n{list}\n\n\
         `via-acp` is the generic ACP client and `via-process` is the generic \
         managed-subprocess core. Neither may know that OpenCode, Qwen Code, Codex or \
         any other backend exists; `via-backends` is where names live, and a harness is \
         resolved from configuration by id. Upstream asserts the same thing with a \
         regex over source text — see server/test/dependency-boundaries.test.mjs:60-73, \
         {RULE_SOURCE}, and §17 item 8.\n\n\
         If a test needs a real backend fixture, the test belongs in `via-backends` or \
         `via-conformance`, not in a dev-dependency here.",
        list = bullet_list(&bindings, "\n"),
    );
}

/// Stated separately from the band rule so the failure reads as the rule a
/// human broke, rather than as set arithmetic.
#[test]
fn leaf_crates_depend_on_no_via_crate() {
    let violations = leaf_violations(&graph());

    assert!(
        violations.is_empty(),
        "leaf crates must depend on no VIA crate at all, but:\n\n{list}\n\n\
         A leaf is upstream's `shared`, whose adjacency row is `shared → ∅`. If two \
         leaves need the same type, the type belongs in whichever leaf is more \
         primitive, or the pair belongs in `via-core`. See {RULE_SOURCE}.",
        list = violations
            .iter()
            .map(|edge| format!(
                "  {edge}  ({kind}, declared in {manifest})",
                kind = edge.kind.label(),
                manifest = edge.manifest,
            ))
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

#[test]
fn the_production_crate_graph_is_acyclic() {
    let cycles = graph().production_cycles();

    assert!(
        cycles.is_empty(),
        "{count} dependency cycle(s) among VIA crates:\n\n{list}\n\n\
         Bands are only meaningful on an acyclic graph. Dev-dependency back-edges are \
         deliberately not counted here — cargo permits them and `via-realtime` \
         dev-depending on `via-realtime-mock` is a normal pattern — so a cycle reported \
         here is between crates that ship. See {RULE_SOURCE}.",
        count = cycles.len(),
        list = cycles
            .iter()
            .map(|cycle| format!("  {}", cycle.join(" -> ")))
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

/// No band may reach the test band: `via-conformance`, `via-arch-test` and
/// `via-e2e` are consumers of the workspace, never part of it.
#[test]
fn no_shipping_band_may_depend_on_a_test_crate() {
    for band in Band::ALL {
        if band == Band::Tests {
            continue;
        }
        assert!(
            !band.allows(Band::Tests),
            "band `{band}` may depend on `tests`; a test crate is a consumer of the \
             workspace, never part of it. See {RULE_SOURCE}.",
        );
    }
    assert!(
        Band::Tests.allows(Band::Tests),
        "a test crate must be able to depend on another test crate — `via-e2e` on \
         `via-conformance`, for instance.",
    );
}

/// The derived allow-sets, spelled out. This is the one place the answer is
/// written twice: once as a derivation from `UPSTREAM_ADJACENCY` and once as a
/// literal. Both have to agree, so neither can be edited quietly.
#[test]
fn band_allow_sets_match_the_upstream_adjacency_table() {
    let expected: [(Band, &[Band]); 7] = [
        // shared → ∅
        (Band::Leaf, &[]),
        // core → core, shared
        (Band::Core, &[Band::Leaf, Band::Core]),
        // voice → conversation, core, shared, task, voice
        (
            Band::Layer1,
            &[Band::Leaf, Band::Core, Band::Layer1, Band::Layer2],
        ),
        // conversation → conversation, core, shared  ∪  task → agent, core, task
        (
            Band::Layer2,
            &[Band::Leaf, Band::Core, Band::Layer2, Band::Layer3],
        ),
        // agent → agent, core, shared  ∪  process → process, shared
        (Band::Layer3, &[Band::Leaf, Band::Core, Band::Layer3]),
        // app → agent, app, conversation, core, task, voice
        //   ∪  root → app, process, shared
        (
            Band::App,
            &[
                Band::Leaf,
                Band::Core,
                Band::Layer1,
                Band::Layer2,
                Band::Layer3,
                Band::App,
            ],
        ),
        // no upstream counterpart: tests reach everything
        (
            Band::Tests,
            &[
                Band::Leaf,
                Band::Core,
                Band::Layer1,
                Band::Layer2,
                Band::Layer3,
                Band::App,
                Band::Tests,
            ],
        ),
    ];

    for (band, allowed) in expected {
        let expected: BTreeSet<Band> = allowed.iter().copied().collect();
        assert_eq!(
            band.allowed(),
            expected,
            "band `{band}`'s derived allow-set no longer matches the documented one. \
             Either UPSTREAM_ADJACENCY was edited away from \
             server/test/dependency-boundaries.test.mjs, or the mapping in \
             UpstreamLayer::band() changed. See {RULE_SOURCE}.",
        );
    }
}

/// Layer 1 talks to Layer 3 through Layer 2 and only through Layer 2 —
/// `docs/architecture.md` §6, "sessions, events, cancellation and permissions
/// come from Layer 2". It falls out of upstream's `voice` row, which lists no
/// `agent`; this test says so out loud so a future edit has to argue with it.
#[test]
fn layer1_does_not_reach_layer3_directly() {
    assert!(
        !Band::Layer1.allows(Band::Layer3),
        "Layer 1 may now depend on Layer 3 directly. Upstream's `voice` row is \
         `voice → conversation, core, shared, task, voice` — no `agent`. A harness is \
         Layer 2's to open, and the realtime layer never holds a session, a work_id or \
         a permission decision. See {RULE_SOURCE} and §6.",
    );
}

/// Guards the mapping itself: if a band ever loses its upstream row, its
/// allow-set silently becomes empty and every edge out of it starts failing for
/// the wrong reason.
#[test]
fn every_band_except_tests_maps_from_an_upstream_layer() {
    for band in Band::ALL {
        if band == Band::Tests {
            continue;
        }
        assert!(
            UPSTREAM_ADJACENCY
                .iter()
                .any(|(layer, _)| layer.band() == band),
            "band `{band}` has no row in UPSTREAM_ADJACENCY, so its allow-set derives to \
             nothing. Every band except `tests` maps from at least one upstream layer. \
             See {RULE_SOURCE}.",
        );
    }
}

/// A name appearing twice with two bands would make `band_of` answer by
/// declaration order, which is not an answer.
#[test]
fn the_crate_table_has_no_duplicate_entries() {
    let mut seen = BTreeSet::new();
    let duplicates: Vec<&str> = CRATE_BANDS
        .iter()
        .filter(|(name, _)| !seen.insert(*name))
        .map(|(name, _)| *name)
        .collect();
    assert!(
        duplicates.is_empty(),
        "CRATE_BANDS lists these packages more than once: {duplicates:?}",
    );
}
