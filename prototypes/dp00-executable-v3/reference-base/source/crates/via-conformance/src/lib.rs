//! The byte-equality gate for VIA's **707 external contracts**.
//!
//! `docs/reference/contracts.md` is the readable index and
//! `docs/reference/contracts.json` is the machine-readable catalogue; this crate
//! compiles the JSON in with [`include_str!`] ([`catalogue::CONTRACTS_JSON`]) so
//! the test binary is self-contained, and pairs every entry in it with a
//! [`Coverage`](registry::Coverage) classification.
//!
//! # Why the registry exists
//!
//! A conformance crate that asserts 60 contracts and says nothing about the
//! other 647 is worse than useless, because it reads as *conformant*. So the
//! catalogue is not merely a source of expected values here — it is also the
//! denominator:
//!
//! - [`registry::REGISTRY`] classifies **every** `(kind, name)` in the
//!   catalogue. There is no default and no fallthrough.
//! - `tests/coverage.rs::every_contract_is_classified` fails the build when a
//!   catalogue entry has no registry row, so adding a contract to the JSON
//!   without deciding who owns it does not compile.
//! - `tests/coverage.rs::registry_has_no_dead_rows` fails when a registry row
//!   names a contract the catalogue no longer has, so deletions do not rot the
//!   table in the other direction.
//! - `tests/coverage.rs::coverage_summary` snapshots the counts with `insta`,
//!   so the number moving is a reviewable diff rather than a silent change.
//!
//! # The five classifications
//!
//! | Coverage | Means | Counts as done |
//! | --- | --- | --- |
//! | [`Asserted`] | a named test in this crate compares the shipped value to the catalogue literal | yes |
//! | [`Divergent`] | VIA deliberately does not reproduce the literal; a named test asserts the recorded deviation *and* the upstream value it deviates from | yes |
//! | [`Partial`] | the part VIA already owns is asserted; the rest lands with another crate | no |
//! | [`Behavioural`] | the comparison is somewhere else — either prose owned by a behavioural test, or a byte comparison the owning crate already makes | no |
//! | [`Pending`] | not asserted; the row names the crate that will own it | no |
//!
//! The rule for [`Asserted`] is deliberately strict: a contract is asserted only
//! when a test compares **every literal in its `exactValue`** against a shipped
//! value. Anything less is [`Partial`], which the summary counts separately so
//! that no partial assertion can be mistaken for a complete one.
//!
//! [`Asserted`]: registry::Coverage::Asserted
//! [`Divergent`]: registry::Coverage::Divergent
//! [`Partial`]: registry::Coverage::Partial
//! [`Behavioural`]: registry::Coverage::Behavioural
//! [`Pending`]: registry::Coverage::Pending
//!
//! # Scope, and the division of labour
//!
//! This crate does **not** try to be the only place a contract is compared.
//! Twenty-one crates ship, and most of them carry a `tests/contracts.rs` that
//! parses the catalogue and compares it against their own code — which is the
//! right place for it, because the comparison sits next to the value and fails
//! in the build that would break it.
//!
//! So the division is:
//!
//! - **Asserted here** — contracts whose owner is a leaf crate the byte gate
//!   was built around (`via-protocol`, `via-catalog`, `via-log`, `via-store`,
//!   `via-lock`), plus the ones no shipped crate compares and this crate can:
//!   the `zh` frontend-agent documents (`tests/frontend_prompt.rs`), five voice
//!   session bounds `via-voice` only checked the *presence* of
//!   (`tests/voice_constants.rs`), and the leaf crates' Chinese literals
//!   against the `via-i18n` catalog (`tests/localized_leaf_messages.rs`).
//! - **[`Behavioural`] here** — contracts the owning crate already compares.
//!   The row names that crate's test, `tests/coverage.rs::behavioural_tests_exist`
//!   resolves it against the tree, and the count stays out of `locked` because
//!   this gate did not prove it.
//! - **[`Pending`]** — nothing anywhere compares it yet. The row names the crate
//!   that will, and `tests/coverage.rs::gaps_in_shipped_crates` snapshots the
//!   ones whose owner already ships, so "work available now" and "waiting on a
//!   crate nobody has written" never blur together.
//!
//! (`via-audio`, `via-downstream`, `via-realtime-mock` and `via-context` own no
//! catalogued contract at all — see the coverage summary. That they own none is
//! itself worth seeing.)
//!
//! # Contract identity
//!
//! The catalogue holds 707 records under 682 distinct `(kind, name)` pairs: 25
//! contracts are catalogued twice, once from the defining module and once from
//! the upstream test that locks it, and the two records spell the same value
//! differently (`"'2.0.0'"` and `2.0.0`, or a comma-separated list and a
//! pipe-separated one). A registry row is therefore keyed by `(kind, name)` and
//! covers **all** records under that key; the summary's denominator stays 707
//! records so the catalogue's own count is what is reported.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod catalogue;
pub mod registry;
pub mod summary;
pub mod value;

pub use catalogue::{
    CONTRACTS_JSON, Contract, ContractKey, contracts, expect_contract, records_for,
};
pub use registry::{Coverage, Crate, Entry, REGISTRY, entry_for};
pub use summary::{CrateRow, StatusCounts, Summary, coverage_summary};
