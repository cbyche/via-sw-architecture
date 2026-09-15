//! The layer bands and the upstream adjacency table they are derived from.

use std::collections::BTreeSet;
use std::fmt;

/// A layer band from [`docs/architecture.md` §9].
///
/// The bands are VIA's names for upstream qwen-audio-agent's source layers.
/// They are ordered leaf-first: a band may only depend on bands at or below
/// its own position, plus its own band and — for `layer2` — the band *above*
/// it, because the Work queue dispatches down into Layer 3 while Layer 1
/// reaches Layer 3 only through Layer 2. That asymmetry is upstream's, not
/// ours; see [`UPSTREAM_ADJACENCY`].
///
/// [`docs/architecture.md` §9]: ../../../docs/architecture.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Band {
    /// `via-protocol` · `via-catalog` · `via-log` · `via-store` · `via-lock` ·
    /// `via-audio`. Upstream's `shared`. Depends on no VIA crate at all.
    Leaf,
    /// `via-core` · `via-i18n`. Upstream's `core`.
    Core,
    /// Realtime Frontstage — `via-realtime*`, `via-wake-word`, `via-voice`.
    /// Upstream's `voice`.
    Layer1,
    /// Middleware & Coordination — `via-work`, `via-coordinator`,
    /// `via-context`, `via-conversation`. Upstream's `task` + `conversation`.
    Layer2,
    /// Backend Agent Execution — `via-downstream`, `via-acp`, `via-backends`,
    /// `via-process`, `via-mcp-tools`. Upstream's `agent` + `process`.
    Layer3,
    /// `via-app` and the `via` binary. Upstream's `app` + its `root` entry
    /// files.
    App,
    /// `via-conformance` · `via-arch-test` · `via-e2e`. No upstream
    /// counterpart — upstream's tests live beside the sources they exercise.
    /// A test crate may reach every band; no band may reach a test crate.
    Tests,
}

impl Band {
    /// Every band, in leaf-first order.
    pub const ALL: [Band; 7] = [
        Band::Leaf,
        Band::Core,
        Band::Layer1,
        Band::Layer2,
        Band::Layer3,
        Band::App,
        Band::Tests,
    ];

    /// The band's name as it is written in failure messages and in
    /// `docs/architecture.md` §9's table.
    pub const fn name(self) -> &'static str {
        match self {
            Band::Leaf => "leaf",
            Band::Core => "core",
            Band::Layer1 => "layer1",
            Band::Layer2 => "layer2",
            Band::Layer3 => "layer3",
            Band::App => "app",
            Band::Tests => "tests",
        }
    }

    /// The bands this band's crates may depend on.
    ///
    /// Derived, not written down: the set is the union of the
    /// [`UPSTREAM_ADJACENCY`] rows that map to this band, with each row's
    /// targets mapped through [`UpstreamLayer::band`]. Changing the upstream
    /// table changes this answer, which is the point — the adjacency table is
    /// the contract, and this function is a projection of it.
    ///
    /// [`Band::Tests`] is the one band with no upstream row, because upstream
    /// has no test *layer*: its tests sit in `server/test/` and import freely.
    /// It is therefore given every band explicitly.
    pub fn allowed(self) -> BTreeSet<Band> {
        if self == Band::Tests {
            return Band::ALL.into_iter().collect();
        }
        UPSTREAM_ADJACENCY
            .iter()
            .filter(|(layer, _)| layer.band() == self)
            .flat_map(|(_, targets)| targets.iter().map(|target| target.band()))
            .collect()
    }

    /// Whether a crate in this band may depend on a crate in `target`.
    pub fn allows(self, target: Band) -> bool {
        self.allowed().contains(&target)
    }
}

impl fmt::Display for Band {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Renders a band set the way the failure messages want it: leaf-first,
/// comma-separated, and `(nothing)` rather than an empty string.
pub fn render_bands(bands: &BTreeSet<Band>) -> String {
    if bands.is_empty() {
        return "(nothing)".to_owned();
    }
    bands
        .iter()
        .map(|band| band.name())
        .collect::<Vec<_>>()
        .join(", ")
}

/// A source layer of upstream qwen-audio-agent v1.11.0.
///
/// These are the directory names under `server/src/`, plus `shared` (the
/// repo-level `shared/` directory) and `root` (the `server/src/*.mjs` entry
/// files, which upstream's test special-cases).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UpstreamLayer {
    /// `shared/` — the repo-level directory, imported by everything.
    Shared,
    /// `server/src/core/`.
    Core,
    /// `server/src/process/`.
    Process,
    /// `server/src/agent/`.
    Agent,
    /// `server/src/conversation/`.
    Conversation,
    /// `server/src/task/`.
    Task,
    /// `server/src/voice/`.
    Voice,
    /// `server/src/app/`.
    App,
    /// `server/src/*.mjs` — the entry files, which upstream's test treats as a
    /// layer of their own with its own allow-list.
    Root,
}

impl UpstreamLayer {
    /// The layer's name as upstream writes it.
    pub const fn name(self) -> &'static str {
        match self {
            UpstreamLayer::Shared => "shared",
            UpstreamLayer::Core => "core",
            UpstreamLayer::Process => "process",
            UpstreamLayer::Agent => "agent",
            UpstreamLayer::Conversation => "conversation",
            UpstreamLayer::Task => "task",
            UpstreamLayer::Voice => "voice",
            UpstreamLayer::App => "app",
            UpstreamLayer::Root => "root",
        }
    }

    /// The VIA band this upstream layer became.
    ///
    /// Four of the mappings merge two upstream layers into one band, because
    /// VIA's split is by *layer of the architecture* while upstream's is by
    /// directory:
    ///
    /// - `agent` + `process` → [`Band::Layer3`] (the ACP client, the managed
    ///   subprocess, and the backend registry are all Layer 3).
    /// - `conversation` + `task` → [`Band::Layer2`] (`via-conversation` sits
    ///   beside `via-work`, not inside it).
    /// - `app` + `root` → [`Band::App`] (`apps/via` is the process entry point,
    ///   which is what upstream's `server/src/*.mjs` files are).
    pub const fn band(self) -> Band {
        match self {
            UpstreamLayer::Shared => Band::Leaf,
            UpstreamLayer::Core => Band::Core,
            UpstreamLayer::Process | UpstreamLayer::Agent => Band::Layer3,
            UpstreamLayer::Conversation | UpstreamLayer::Task => Band::Layer2,
            UpstreamLayer::Voice => Band::Layer1,
            UpstreamLayer::App | UpstreamLayer::Root => Band::App,
        }
    }
}

impl fmt::Display for UpstreamLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Upstream's layer adjacency table, reproduced exactly.
///
/// **External contract.** The first eight rows are
/// `allowedDependencies` in `server/test/dependency-boundaries.test.mjs:10-18`
/// (qwen-audio-agent v1.11.0), quoted verbatim in `docs/architecture.md` §9.
/// The ninth row is the `root` special case in the same file's test body,
/// `dependency-boundaries.test.mjs:48` — `sourceLayer === 'root'` is checked
/// against `new Set(['app', 'process', 'shared'])` rather than against the
/// table. It is included here because `apps/via` is VIA's counterpart to those
/// entry files, and folding it in is what lets the `app` band depend on `leaf`
/// (upstream's `app` row does not list `shared`; upstream's `root` rule does).
///
/// Row order is upstream's own, so a diff against the source file reads
/// straight down.
pub const UPSTREAM_ADJACENCY: &[(UpstreamLayer, &[UpstreamLayer])] = &[
    // shared → ∅
    (UpstreamLayer::Shared, &[]),
    // core → core, shared
    (
        UpstreamLayer::Core,
        &[UpstreamLayer::Core, UpstreamLayer::Shared],
    ),
    // process → process, shared
    (
        UpstreamLayer::Process,
        &[UpstreamLayer::Process, UpstreamLayer::Shared],
    ),
    // agent → agent, core, shared
    (
        UpstreamLayer::Agent,
        &[
            UpstreamLayer::Agent,
            UpstreamLayer::Core,
            UpstreamLayer::Shared,
        ],
    ),
    // conversation → conversation, core, shared
    (
        UpstreamLayer::Conversation,
        &[
            UpstreamLayer::Conversation,
            UpstreamLayer::Core,
            UpstreamLayer::Shared,
        ],
    ),
    // task → agent, core, task
    (
        UpstreamLayer::Task,
        &[
            UpstreamLayer::Agent,
            UpstreamLayer::Core,
            UpstreamLayer::Task,
        ],
    ),
    // voice → conversation, core, shared, task, voice
    (
        UpstreamLayer::Voice,
        &[
            UpstreamLayer::Conversation,
            UpstreamLayer::Core,
            UpstreamLayer::Shared,
            UpstreamLayer::Task,
            UpstreamLayer::Voice,
        ],
    ),
    // app → agent, app, conversation, core, task, voice
    (
        UpstreamLayer::App,
        &[
            UpstreamLayer::Agent,
            UpstreamLayer::App,
            UpstreamLayer::Conversation,
            UpstreamLayer::Core,
            UpstreamLayer::Task,
            UpstreamLayer::Voice,
        ],
    ),
    // root → app, process, shared   (test body, not the table)
    (
        UpstreamLayer::Root,
        &[
            UpstreamLayer::App,
            UpstreamLayer::Process,
            UpstreamLayer::Shared,
        ],
    ),
];
