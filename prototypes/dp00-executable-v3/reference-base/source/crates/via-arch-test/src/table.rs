//! Which crate is in which band.

use crate::band::Band;

/// Every VIA package and the band it belongs to.
///
/// **External contract.** This is the band table of `docs/architecture.md` §9,
/// entry for entry and in the same order. `tests/architecture_doc.rs` asserts
/// the two against each other, so the document cannot drift away from the gate
/// or the gate away from the document.
///
/// Most of these crates do not exist yet — phase 0 ships the six leaves plus
/// `via-arch-test` and the `via` skeleton. The table is written out in full
/// anyway: a crate that lands must already be classified here, because
/// `tests/crate_graph.rs` refuses to accept a VIA package the table does not
/// mention. Listing a crate that does not exist costs nothing; failing to list
/// one that does is what the gate is for.
///
/// `apps/via` appears here under its **package** name, `via` — §9's table
/// names it by path, because it is the one member outside `crates/`.
pub const CRATE_BANDS: &[(&str, Band)] = &[
    // Leaf — no VIA dependencies at all.
    ("via-protocol", Band::Leaf),
    ("via-catalog", Band::Leaf),
    ("via-log", Band::Leaf),
    ("via-store", Band::Leaf),
    ("via-lock", Band::Leaf),
    ("via-audio", Band::Leaf),
    // Core
    ("via-core", Band::Core),
    ("via-i18n", Band::Core),
    // Layer 1 — Realtime Frontstage
    ("via-realtime", Band::Layer1),
    ("via-realtime-openai", Band::Layer1),
    ("via-realtime-dashscope", Band::Layer1),
    ("via-realtime-local", Band::Layer1),
    ("via-realtime-mock", Band::Layer1),
    ("via-wake-word", Band::Layer1),
    ("via-voice", Band::Layer1),
    // Layer 2 — Middleware & Coordination
    ("via-work", Band::Layer2),
    ("via-coordinator", Band::Layer2),
    ("via-context", Band::Layer2),
    ("via-conversation", Band::Layer2),
    // Layer 3 — Backend Agent Execution
    ("via-downstream", Band::Layer3),
    ("via-acp", Band::Layer3),
    ("via-backends", Band::Layer3),
    ("via-process", Band::Layer3),
    ("via-mcp-tools", Band::Layer3),
    // App
    ("via-app", Band::App),
    ("via", Band::App),
    // Tests
    ("via-conformance", Band::Tests),
    ("via-arch-test", Band::Tests),
    ("via-e2e", Band::Tests),
];

/// The generic cores that must not bind to a named backend.
///
/// **External contract.** Upstream asserts this over source text, with
/// `genericCoreFiles` in `server/test/dependency-boundaries.test.mjs:61-67`
/// scanned for `/\b(?:openclaw|opencode|qoder|qwen|kimi|hermes|codebuddy|
/// codex|claude|pi)\b/i`. Its five files are VIA's two crates:
/// `agent/acp-process-client.mjs` and `agent/acp-backend-adapter.mjs` are
/// `via-acp`; `process/managed-backend.mjs`, `cli/src/runtime.mjs` and
/// `cli/src/launcher.mjs` are `via-process`.
///
/// In Rust the rule needs no regex. Every backend name lives in
/// [`NAMED_BACKEND_REGISTRY`], and a crate that cannot depend on that crate
/// cannot name what is inside it. See `docs/architecture.md` §9, and §17
/// item 8 — "Does `via-acp` or `via-process` now depend on `via-backends`?
/// *(It must not.)*"
pub const GENERIC_CORES: &[&str] = &["via-acp", "via-process"];

/// The crate that holds the named backends: launch specs, model tables,
/// environment allow-lists, and the twelve ACP backend ids.
///
/// See [`GENERIC_CORES`].
pub const NAMED_BACKEND_REGISTRY: &str = "via-backends";

/// The band a package belongs to, or `None` if the table has never heard of it.
///
/// `None` for a workspace member is a failure, not a default — see
/// [`CRATE_BANDS`].
pub fn band_of(package: &str) -> Option<Band> {
    CRATE_BANDS
        .iter()
        .find(|(name, _)| *name == package)
        .map(|(_, band)| *band)
}

/// Whether a package name is one of VIA's own.
///
/// The binary is `via`; every crate is `via-*`. Nothing else is in the
/// workspace, and `tests/crate_graph.rs` asserts that too — a member named
/// outside this convention would slip past every name-keyed rule in this
/// crate.
pub fn is_via_package(package: &str) -> bool {
    package == "via" || package.starts_with("via-")
}
