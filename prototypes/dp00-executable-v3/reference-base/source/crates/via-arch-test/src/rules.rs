//! The rules, as functions over a graph.
//!
//! Each returns *every* violation rather than the first, and each knows how to
//! render itself for someone who has just broken it. The assertions in
//! `tests/` are thin wrappers around these; keeping the logic here is what lets
//! it be tested against graphs cargo would refuse to resolve — a cycle, most
//! obviously.

use std::fmt;

use crate::band::{Band, render_bands};
use crate::graph::{CrateGraph, Edge};
use crate::table::{GENERIC_CORES, NAMED_BACKEND_REGISTRY, band_of, is_via_package};

/// A dependency edge that leaves its band's allowed set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BandViolation {
    /// The offending edge.
    pub edge: Edge,
    /// The band the depending crate is in.
    pub from: Band,
    /// The band the depended-on crate is in.
    pub to: Band,
}

impl fmt::Display for BandViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut gates = Vec::new();
        if self.edge.optional {
            gates.push("optional".to_owned());
        }
        if let Some(target) = &self.edge.target {
            gates.push(format!("target `{target}`"));
        }
        let gate_note = if gates.is_empty() {
            String::new()
        } else {
            format!(" ({})", gates.join(", "))
        };

        write!(
            f,
            "  {from} -> {to}\n\
             \x20     {from} is in band `{from_band}`, which may depend on: {allowed}\n\
             \x20     {to} is in band `{to_band}`\n\
             \x20     edge: {kind}{gate_note}, declared in {manifest}",
            from = self.edge.from,
            to = self.edge.to,
            from_band = self.from,
            to_band = self.to,
            allowed = render_bands(&self.from.allowed()),
            kind = self.edge.kind.label(),
            manifest = self.edge.manifest,
        )
    }
}

/// Every edge whose target band is outside the source band's allowed set.
///
/// Edges touching an unclassified package are skipped — that is
/// [`unclassified_packages`]' failure, with its own message.
pub fn band_violations(graph: &CrateGraph) -> Vec<BandViolation> {
    graph
        .edges()
        .iter()
        .filter_map(|edge| {
            let from = band_of(&edge.from)?;
            let to = band_of(&edge.to)?;
            (!from.allows(to)).then(|| BandViolation {
                edge: edge.clone(),
                from,
                to,
            })
        })
        .collect()
}

/// Every VIA package in the workspace that [`crate::CRATE_BANDS`] does not
/// classify. The ratchet: a crate cannot land unclassified.
pub fn unclassified_packages(graph: &CrateGraph) -> Vec<String> {
    graph
        .packages()
        .iter()
        .filter(|package| is_via_package(package))
        .filter(|package| band_of(package).is_none())
        .cloned()
        .collect()
}

/// Every workspace member outside the `via` / `via-*` naming convention.
///
/// Every rule in this crate is keyed on a package name, so a member named
/// otherwise would be exempt from all of them.
pub fn non_via_members(graph: &CrateGraph) -> Vec<String> {
    graph
        .packages()
        .iter()
        .filter(|package| !is_via_package(package))
        .cloned()
        .collect()
}

/// Every edge out of a leaf crate. Upstream's row is `shared → ∅`; a leaf
/// depends on no VIA crate at all, of any kind.
pub fn leaf_violations(graph: &CrateGraph) -> Vec<Edge> {
    graph
        .edges()
        .iter()
        .filter(|edge| band_of(&edge.from) == Some(Band::Leaf))
        .cloned()
        .collect()
}

/// How a generic core came to be bound to the named-backend registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendBinding {
    /// The core declares the dependency itself, in any dependency table.
    Direct {
        /// The generic core.
        core: String,
        /// The declaring edge.
        edge: Edge,
    },
    /// The core reaches the registry through other crates that ship.
    Transitive {
        /// The generic core.
        core: String,
        /// The shortest production path, endpoints included.
        path: Vec<String>,
    },
}

impl fmt::Display for BackendBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendBinding::Direct { core, edge } => write!(
                f,
                "  {core} -> {NAMED_BACKEND_REGISTRY}  (direct {kind}, declared in {manifest})",
                kind = edge.kind.label(),
                manifest = edge.manifest,
            ),
            BackendBinding::Transitive { core, path } => write!(
                f,
                "  {core} -> {NAMED_BACKEND_REGISTRY}  (reached through: {path})",
                path = path.join(" -> "),
            ),
        }
    }
}

/// The compile-time form of upstream's "generic ACP and process cores do not
/// bind to named backends".
///
/// Both the direct edge — any kind, dev included — and any production path are
/// reported. A generic core that reaches the backend registry through an
/// intermediary is just as bound to it, only less obviously.
///
/// Upstream's version is a regex for
/// `openclaw|opencode|qoder|qwen|kimi|hermes|codebuddy|codex|claude|pi` over
/// five source files (`server/test/dependency-boundaries.test.mjs:60-73`).
pub fn backend_bindings(graph: &CrateGraph) -> Vec<BackendBinding> {
    let mut bindings = Vec::new();
    for core in GENERIC_CORES {
        for edge in graph.edges_from(core) {
            if edge.to == NAMED_BACKEND_REGISTRY {
                bindings.push(BackendBinding::Direct {
                    core: (*core).to_owned(),
                    edge: edge.clone(),
                });
            }
        }
        if let Some(path) = graph.production_path(core, NAMED_BACKEND_REGISTRY) {
            bindings.push(BackendBinding::Transitive {
                core: (*core).to_owned(),
                path,
            });
        }
    }
    bindings
}

/// Joins rendered violations into the body of a failure message.
pub fn bullet_list<T: fmt::Display>(items: &[T], separator: &str) -> String {
    items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(separator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::EdgeKind;

    fn graph(packages: &[&str], edges: Vec<Edge>) -> CrateGraph {
        CrateGraph::from_parts(
            "/fixture",
            packages.iter().map(|name| (*name).to_owned()).collect(),
            edges,
        )
    }

    #[test]
    fn a_clean_graph_has_no_violations() {
        let graph = graph(
            &["via-protocol", "via-core", "via-work", "via-acp"],
            vec![
                Edge::new("via-core", "via-protocol", EdgeKind::Normal),
                Edge::new("via-acp", "via-core", EdgeKind::Normal),
                Edge::new("via-work", "via-acp", EdgeKind::Normal),
                Edge::new("via-work", "via-protocol", EdgeKind::Dev),
            ],
        );
        assert!(band_violations(&graph).is_empty());
        assert!(leaf_violations(&graph).is_empty());
        assert!(backend_bindings(&graph).is_empty());
        assert!(graph.production_cycles().is_empty());
        assert!(unclassified_packages(&graph).is_empty());
    }

    #[test]
    fn an_upward_edge_is_a_band_violation() {
        let graph = graph(
            &["via-core", "via-work"],
            vec![Edge::new("via-core", "via-work", EdgeKind::Normal)],
        );
        let violations = band_violations(&graph);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].from, Band::Core);
        assert_eq!(violations[0].to, Band::Layer2);

        let rendered = violations[0].to_string();
        assert!(rendered.contains("via-core -> via-work"), "{rendered}");
        assert!(rendered.contains("band `core`"), "{rendered}");
        assert!(rendered.contains("band `layer2`"), "{rendered}");
        assert!(rendered.contains("leaf, core"), "{rendered}");
    }

    #[test]
    fn layer1_reaching_layer3_is_a_band_violation() {
        let graph = graph(
            &["via-voice", "via-acp"],
            vec![Edge::new("via-voice", "via-acp", EdgeKind::Normal)],
        );
        assert_eq!(band_violations(&graph).len(), 1);
    }

    #[test]
    fn every_violation_is_reported_not_just_the_first() {
        let graph = graph(
            &["via-protocol", "via-core", "via-voice", "via-acp"],
            vec![
                Edge::new("via-protocol", "via-core", EdgeKind::Normal),
                Edge::new("via-core", "via-voice", EdgeKind::Normal),
                Edge::new("via-voice", "via-acp", EdgeKind::Dev),
            ],
        );
        assert_eq!(band_violations(&graph).len(), 3);
    }

    #[test]
    fn a_dev_dependency_is_still_an_edge() {
        let graph = graph(
            &["via-protocol", "via-log"],
            vec![Edge::new("via-protocol", "via-log", EdgeKind::Dev)],
        );
        assert_eq!(leaf_violations(&graph).len(), 1);
        assert_eq!(band_violations(&graph).len(), 1);
    }

    #[test]
    fn a_direct_backend_dependency_is_caught_in_either_generic_core() {
        for core in GENERIC_CORES {
            let graph = graph(
                &[core, NAMED_BACKEND_REGISTRY],
                vec![Edge::new(*core, NAMED_BACKEND_REGISTRY, EdgeKind::Normal)],
            );
            let bindings = backend_bindings(&graph);
            assert_eq!(bindings.len(), 2, "{core}: {bindings:?}");
            assert!(matches!(bindings[0], BackendBinding::Direct { .. }));
            assert!(matches!(bindings[1], BackendBinding::Transitive { .. }));
        }
    }

    /// The subtle case: `via-acp` never names `via-backends`, but reaches it
    /// through `via-downstream`. Same binding, less obvious.
    #[test]
    fn a_backend_dependency_through_an_intermediary_is_caught() {
        let graph = graph(
            &["via-acp", "via-downstream", "via-backends"],
            vec![
                Edge::new("via-acp", "via-downstream", EdgeKind::Normal),
                Edge::new("via-downstream", "via-backends", EdgeKind::Normal),
            ],
        );
        let bindings = backend_bindings(&graph);
        assert_eq!(bindings.len(), 1);
        assert_eq!(
            bindings[0],
            BackendBinding::Transitive {
                core: "via-acp".to_owned(),
                path: vec![
                    "via-acp".to_owned(),
                    "via-downstream".to_owned(),
                    "via-backends".to_owned(),
                ],
            },
        );
        assert!(
            bindings[0]
                .to_string()
                .contains("via-acp -> via-downstream"),
            "the message must show the path: {}",
            bindings[0],
        );
    }

    /// A dev-dependency is reported as a direct binding but does not extend a
    /// path — otherwise every crate would "reach" everything through
    /// `via-conformance`, which dev-depends on the whole workspace.
    #[test]
    fn a_dev_edge_binds_directly_but_does_not_lengthen_a_path() {
        let through_a_test_crate = graph(
            &["via-acp", "via-conformance", "via-backends"],
            vec![
                Edge::new("via-acp", "via-conformance", EdgeKind::Dev),
                Edge::new("via-conformance", "via-backends", EdgeKind::Normal),
            ],
        );
        assert!(backend_bindings(&through_a_test_crate).is_empty());

        let direct = graph(
            &["via-process", "via-backends"],
            vec![Edge::new("via-process", "via-backends", EdgeKind::Dev)],
        );
        assert_eq!(
            backend_bindings(&direct),
            vec![BackendBinding::Direct {
                core: "via-process".to_owned(),
                edge: Edge::new("via-process", "via-backends", EdgeKind::Dev),
            }],
        );
    }

    #[test]
    fn a_production_cycle_is_found_and_reported_once() {
        let graph = graph(
            &["via-work", "via-coordinator", "via-context"],
            vec![
                Edge::new("via-work", "via-coordinator", EdgeKind::Normal),
                Edge::new("via-coordinator", "via-context", EdgeKind::Normal),
                Edge::new("via-context", "via-work", EdgeKind::Normal),
            ],
        );
        let cycles = graph.production_cycles();
        assert_eq!(cycles.len(), 1, "{cycles:?}");
        let cycle = &cycles[0];
        assert_eq!(cycle.first(), cycle.last());
        assert_eq!(cycle.len(), 4);
    }

    #[test]
    fn a_dev_dependency_back_edge_is_not_a_cycle() {
        let graph = graph(
            &["via-realtime", "via-realtime-mock"],
            vec![
                Edge::new("via-realtime-mock", "via-realtime", EdgeKind::Normal),
                Edge::new("via-realtime", "via-realtime-mock", EdgeKind::Dev),
            ],
        );
        assert!(graph.production_cycles().is_empty());
        assert!(band_violations(&graph).is_empty());
    }

    #[test]
    fn a_build_dependency_counts_as_production() {
        let graph = graph(
            &["via-protocol", "via-core"],
            vec![Edge::new("via-protocol", "via-core", EdgeKind::Build)],
        );
        assert_eq!(band_violations(&graph).len(), 1);
        assert_eq!(
            graph.production_path("via-protocol", "via-core"),
            Some(vec!["via-protocol".to_owned(), "via-core".to_owned()]),
        );
    }

    #[test]
    fn an_unlisted_crate_is_reported_as_unclassified() {
        let graph = graph(&["via-protocol", "via-brand-new"], Vec::new());
        assert_eq!(unclassified_packages(&graph), vec!["via-brand-new"]);
        assert!(non_via_members(&graph).is_empty());
    }

    #[test]
    fn a_member_outside_the_naming_convention_is_reported() {
        let graph = graph(&["via-protocol", "gateway-helper"], Vec::new());
        assert_eq!(non_via_members(&graph), vec!["gateway-helper"]);
        assert!(unclassified_packages(&graph).is_empty());
    }
}
