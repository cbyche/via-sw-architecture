//! The coverage registry: every contract, and who asserts it.
//!
//! One row per `(kind, name)` in `docs/reference/contracts.json`, 682 rows for
//! 707 records. The table is exhaustive by construction and kept exhaustive by
//! `tests/coverage.rs`, which fails the build both when the catalogue grows a
//! contract this table does not classify and when the table keeps a row the
//! catalogue no longer has.
//!
//! Rows are grouped by owning crate, in the band order of
//! `docs/architecture.md` §9 (leaf → core → layer 1 → layer 2 → layer 3 → app),
//! and sorted by `(kind, name)` inside a group. The owner was derived from the
//! upstream module each contract was read from and then corrected by hand where
//! the module maps to more than one VIA crate — for instance every Chinese
//! user-facing message is owned by `via-i18n` rather than by the crate that
//! throws it, because `docs/architecture.md` §16 hoists the message and leaves
//! only the code behind.

use crate::catalogue::ContractKey;

/// A crate in VIA's graph — the unit that owns a contract.
///
/// The 25 crates and one binary of `docs/architecture.md` §9, declared in that
/// document's band order so the coverage summary reads bottom-up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Crate {
    /// Leaf: the frozen contract surface.
    ViaProtocol,
    /// Leaf: the static tables.
    ViaCatalog,
    /// Leaf: structured logging.
    ViaLog,
    /// Leaf: atomic, versioned JSON persistence.
    ViaStore,
    /// Leaf: the instance lease and the file transaction lock.
    ViaLock,
    /// Leaf: PCM handling. No upstream counterpart.
    ViaAudio,
    /// Core: configuration, the setup gate, the runtime environment.
    ViaCore,
    /// Core: the localized string catalogue. No upstream counterpart.
    ViaI18n,
    /// Layer 1: the provider seam, the floor policy, the reconnect ladder.
    ViaRealtime,
    /// Layer 1: the OpenAI / Azure / litellm realtime client, ported from ARGO.
    ViaRealtimeOpenai,
    /// Layer 1: the DashScope realtime client.
    ViaRealtimeDashscope,
    /// Layer 1: the on-device path. No upstream counterpart.
    ViaRealtimeLocal,
    /// Layer 1: deterministic replay. No upstream counterpart.
    ViaRealtimeMock,
    /// Layer 1: wake-word detection.
    ViaWakeWord,
    /// Layer 1: the voice session — the gateway socket, the floor, the
    /// announcement window, the frontend tool surface.
    ViaVoice,
    /// Layer 2: the Work record, its lifecycle, its store and its scheduler.
    ViaWork,
    /// Layer 2: the coordinator turn and the intent set.
    ViaCoordinator,
    /// Layer 2: the Context Engine. No upstream counterpart.
    ViaContext,
    /// Layer 2: memory, notes and conversation sync.
    ViaConversation,
    /// Layer 3: the `DownstreamAgent` seam. No upstream counterpart.
    ViaDownstream,
    /// Layer 3: the ACP client and its adapters.
    ViaAcp,
    /// Layer 3: backend launch specs, installation and authentication.
    ViaBackends,
    /// Layer 3: child process supervision.
    ViaProcess,
    /// Layer 3: the five coordination tools exposed over MCP.
    ViaMcpTools,
    /// App: the HTTP surface and the composition root.
    ViaApp,
    /// App: the `via` binary.
    AppsVia,
}

impl Crate {
    /// Every crate, in `docs/architecture.md` §9 band order.
    ///
    /// The order the coverage summary groups by. Includes the five crates that
    /// own no upstream contract at all — that they own none is itself worth
    /// seeing, because four of them are the boxes the reference architecture
    /// draws that the qwen port alone would not have produced.
    pub const ALL: &'static [Self] = &[
        Self::ViaProtocol,
        Self::ViaCatalog,
        Self::ViaLog,
        Self::ViaStore,
        Self::ViaLock,
        Self::ViaAudio,
        Self::ViaCore,
        Self::ViaI18n,
        Self::ViaRealtime,
        Self::ViaRealtimeOpenai,
        Self::ViaRealtimeDashscope,
        Self::ViaRealtimeLocal,
        Self::ViaRealtimeMock,
        Self::ViaWakeWord,
        Self::ViaVoice,
        Self::ViaWork,
        Self::ViaCoordinator,
        Self::ViaContext,
        Self::ViaConversation,
        Self::ViaDownstream,
        Self::ViaAcp,
        Self::ViaBackends,
        Self::ViaProcess,
        Self::ViaMcpTools,
        Self::ViaApp,
        Self::AppsVia,
    ];

    /// The crate's Cargo package name, or its workspace path for the binary.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ViaProtocol => "via-protocol",
            Self::ViaCatalog => "via-catalog",
            Self::ViaLog => "via-log",
            Self::ViaStore => "via-store",
            Self::ViaLock => "via-lock",
            Self::ViaAudio => "via-audio",
            Self::ViaCore => "via-core",
            Self::ViaI18n => "via-i18n",
            Self::ViaRealtime => "via-realtime",
            Self::ViaRealtimeOpenai => "via-realtime-openai",
            Self::ViaRealtimeDashscope => "via-realtime-dashscope",
            Self::ViaRealtimeLocal => "via-realtime-local",
            Self::ViaRealtimeMock => "via-realtime-mock",
            Self::ViaWakeWord => "via-wake-word",
            Self::ViaVoice => "via-voice",
            Self::ViaWork => "via-work",
            Self::ViaCoordinator => "via-coordinator",
            Self::ViaContext => "via-context",
            Self::ViaConversation => "via-conversation",
            Self::ViaDownstream => "via-downstream",
            Self::ViaAcp => "via-acp",
            Self::ViaBackends => "via-backends",
            Self::ViaProcess => "via-process",
            Self::ViaMcpTools => "via-mcp-tools",
            Self::ViaApp => "via-app",
            Self::AppsVia => "apps/via",
        }
    }

    /// Whether the crate ships today, and is therefore a crate whose contracts
    /// somebody could assert now.
    ///
    /// A [`Coverage::Pending`] row against a crate for which this returns
    /// `true` is work available now; one against a crate nobody has written is
    /// a plan. `tests/coverage.rs::gaps_in_shipped_crates` snapshots the first
    /// set so it cannot disappear into the second's much larger number, which
    /// is the whole reason this predicate exists.
    ///
    /// All twenty-six ship as of phase 9. `via-realtime-openai` and
    /// `via-wake-word` were scaffolds (a manifest and an empty `lib.rs`) when
    /// this predicate was written; `via-realtime-local` and `via-context` were
    /// phases 8 and 7, not yet landed. All four are now real crates with real
    /// tests, so every one of their rows is work available today.
    ///
    /// `via-audio` and `via-realtime-mock` ship and own no catalogued contract,
    /// so they appear in the list below for accuracy and contribute nothing to
    /// either count. This predicate is kept — rather than deleted now that it
    /// is `true` for every variant — because a 27th crate lands the same way
    /// the last four did: as a scaffold first, and `gaps_in_shipped_crates`
    /// needs somewhere to say so without waiting on it.
    pub const fn exists_today(self) -> bool {
        true
    }
}

impl core::fmt::Display for Crate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How — and whether — a contract is asserted.
///
/// See the crate documentation for the table and for why [`Self::Asserted`] is
/// deliberately strict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    /// A named test compares every literal in the contract's `exactValue`
    /// against the shipped value.
    Asserted {
        /// `tests/<file>.rs::<fn>` — checked to exist by
        /// `tests/coverage.rs::asserting_tests_exist`.
        test: &'static str,
    },
    /// VIA deliberately does not reproduce the upstream literal. A named test
    /// asserts both VIA's value and the upstream value it deviates from, so the
    /// deviation cannot drift into an accident.
    Divergent {
        /// `tests/<file>.rs::<fn>`.
        test: &'static str,
        /// Where the deviation is written down — `docs/fidelity.md`,
        /// `docs/rebrand.md`, or the architecture section that decided it.
        record: &'static str,
    },
    /// The part VIA owns today is asserted; the rest of the `exactValue` lands
    /// with another crate. Never counted as done.
    Partial {
        /// `tests/<file>.rs::<fn>` asserting the part VIA owns.
        test: &'static str,
        /// The crate that will assert what is left.
        remainder: Crate,
    },
    /// The comparison lives somewhere other than this crate's `tests/`.
    ///
    /// Two cases, and the row does not distinguish them because the honest
    /// answer to both is the same — *this* crate does not assert it, and here
    /// is what does:
    ///
    /// 1. The `exactValue` is prose describing behaviour rather than a literal
    ///    to string-compare, so a behavioural test owns it.
    /// 2. The owning crate already compares the catalogued value against its
    ///    own code. Duplicating that byte comparison here would mean two places
    ///    to update and two ways to be wrong; what this crate needs is to
    ///    *know* the comparison exists and where.
    ///
    /// Never counted as done, in either case:
    /// [`is_locked`](Coverage::is_locked) answers `false`, so the headline
    /// number stays "what this gate itself proves".
    Behavioural {
        /// `<package>: <path>::<fn>` — for example
        /// `via-work: tests/contracts.rs::work_id_format`. Resolved and checked
        /// to be a real `#[test]` by
        /// `tests/coverage.rs::behavioural_tests_exist`, so a renamed test in
        /// another crate fails here rather than orphaning the contract.
        test: &'static str,
    },
    /// Not asserted. The row's owner names the crate that will own it.
    Pending,
}

impl Coverage {
    /// The one-word status used by the coverage summary.
    pub const fn status(&self) -> &'static str {
        match self {
            Self::Asserted { .. } => "asserted",
            Self::Divergent { .. } => "divergent",
            Self::Partial { .. } => "partial",
            Self::Behavioural { .. } => "behavioural",
            Self::Pending => "pending",
        }
    }

    /// Whether this classification means the contract is locked.
    ///
    /// True for [`Asserted`](Self::Asserted) and [`Divergent`](Self::Divergent)
    /// only. A partial assertion is not a lock, and neither is a promise.
    pub const fn is_locked(&self) -> bool {
        matches!(self, Self::Asserted { .. } | Self::Divergent { .. })
    }

    /// The in-crate test path, when the classification names one that this
    /// crate's `tests/` directory should contain.
    pub const fn in_crate_test(&self) -> Option<&'static str> {
        match self {
            Self::Asserted { test } | Self::Divergent { test, .. } | Self::Partial { test, .. } => {
                Some(*test)
            }
            // Behavioural tests live in the crate that owns the behaviour.
            Self::Behavioural { .. } | Self::Pending => None,
        }
    }
}

/// One registry row: a contract key, its owner, and its coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    /// The contract's kind, matching `docs/reference/contracts.json`.
    pub kind: &'static str,
    /// The contract's name, matching `docs/reference/contracts.json`.
    pub name: &'static str,
    /// The crate that owns the value — the one whose code has to carry it.
    pub owner: Crate,
    /// How, and whether, it is asserted.
    pub coverage: Coverage,
}

impl Entry {
    /// This row's contract key.
    pub const fn key(&self) -> ContractKey<'static> {
        ContractKey {
            kind: self.kind,
            name: self.name,
        }
    }

    /// A row asserted byte for byte by a named test in this crate.
    const fn asserted(
        kind: &'static str,
        name: &'static str,
        owner: Crate,
        test: &'static str,
    ) -> Self {
        Self {
            kind,
            name,
            owner,
            coverage: Coverage::Asserted { test },
        }
    }

    /// A row where VIA deliberately deviates, with the deviation recorded.
    const fn divergent(
        kind: &'static str,
        name: &'static str,
        owner: Crate,
        test: &'static str,
        record: &'static str,
    ) -> Self {
        Self {
            kind,
            name,
            owner,
            coverage: Coverage::Divergent { test, record },
        }
    }

    /// A row only partly assertable today.
    const fn partial(
        kind: &'static str,
        name: &'static str,
        owner: Crate,
        test: &'static str,
        remainder: Crate,
    ) -> Self {
        Self {
            kind,
            name,
            owner,
            coverage: Coverage::Partial { test, remainder },
        }
    }

    /// A row whose `exactValue` is prose, owned by a behavioural test.
    const fn behavioural(
        kind: &'static str,
        name: &'static str,
        owner: Crate,
        test: &'static str,
    ) -> Self {
        Self {
            kind,
            name,
            owner,
            coverage: Coverage::Behavioural { test },
        }
    }

    /// A row nothing asserts yet.
    const fn pending(kind: &'static str, name: &'static str, owner: Crate) -> Self {
        Self {
            kind,
            name,
            owner,
            coverage: Coverage::Pending,
        }
    }
}

/// The registry row for a contract key, if the table has one.
pub fn entry_for(kind: &str, name: &str) -> Option<&'static Entry> {
    REGISTRY
        .iter()
        .find(|entry| entry.kind == kind && entry.name == name)
}

include!("registry_rows.rs");
