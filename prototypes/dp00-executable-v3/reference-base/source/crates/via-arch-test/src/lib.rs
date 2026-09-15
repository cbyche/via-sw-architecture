//! The crate-graph gate.
//!
//! VIA's layering is a rule with an owner. Upstream qwen-audio-agent enforced
//! it with `server/test/dependency-boundaries.test.mjs`, which walks import
//! statements with a regex against a fixed adjacency table. This crate asserts
//! the same rule against **real `cargo metadata` edges**, so it cannot be
//! fooled by a re-export, a macro, a renamed dependency or a string that a
//! regex happens not to match.
//!
//! The rule itself, and the bands it is written in terms of, are
//! `docs/architecture.md` §9. There are three assertions worth naming:
//!
//! 1. **Every VIA package's VIA dependencies are inside its band's allowed
//!    set.** The allowed sets are not written down — they are *derived* from
//!    upstream's adjacency table in [`UPSTREAM_ADJACENCY`], so upstream stays
//!    the source of the rule.
//! 2. **`via-acp` and `via-process` do not depend on `via-backends`.** This is
//!    the compile-time form of upstream's "generic ACP and process cores do not
//!    bind to named backends", which in JavaScript needed a regex over source
//!    text. In Rust, a crate that cannot depend on the backend registry cannot
//!    name what is inside it. See `docs/architecture.md` §17 item 8.
//! 3. **Every VIA package in the workspace appears in [`CRATE_BANDS`].** Most
//!    of §9's crates do not exist yet. Asserting only over what exists would
//!    let the gate rot as crates land; asserting that everything that exists is
//!    classified makes it ratchet instead — a new crate cannot be added without
//!    being put in a band.
//!
//! # Layout
//!
//! The library is the vocabulary: bands, the upstream table, the crate table,
//! and a `cargo metadata` reader. The assertions live in `tests/`, because a
//! gate that can be silenced by a feature flag is not a gate.
//!
//! `via-arch-test` is `publish = false` and depends on no VIA crate. It sits in
//! the [`Band::Tests`] band, which may reach every other band and which no
//! other band may reach.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod band;
mod graph;
mod rules;
mod table;

pub use band::{Band, UPSTREAM_ADJACENCY, UpstreamLayer, render_bands};
pub use graph::{CrateGraph, Edge, EdgeKind, LoadError, workspace_root};
pub use rules::{
    BackendBinding, BandViolation, backend_bindings, band_violations, bullet_list, leaf_violations,
    non_via_members, unclassified_packages,
};
pub use table::{CRATE_BANDS, GENERIC_CORES, NAMED_BACKEND_REGISTRY, band_of, is_via_package};

/// Where a human is sent when a rule in this crate fails.
///
/// Every failure message names it. It is a constant so the pointer cannot
/// drift from one assertion to the next, and so renumbering the section is a
/// one-line change rather than a grep.
pub const RULE_SOURCE: &str = "docs/architecture.md §9 (crate graph)";
