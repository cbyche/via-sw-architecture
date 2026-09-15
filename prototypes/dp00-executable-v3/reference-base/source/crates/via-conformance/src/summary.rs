//! The coverage summary — the number that must not be allowed to lie.
//!
//! Rendered by `tests/coverage.rs::coverage_summary` into an `insta` snapshot,
//! so every change to what VIA asserts arrives as a reviewable diff instead of
//! as a silently better-looking test run.
//!
//! Two denominators are reported side by side and they are not the same thing:
//!
//! - **records** — the 707 entries in `docs/reference/contracts.json`. This is
//!   the number `docs/architecture.md` §14 promises, so it is the number the
//!   summary leads with.
//! - **rows** — the 682 registry rows. Fewer, because 25 contracts are
//!   catalogued twice under one `(kind, name)`.

use std::collections::BTreeMap;

use crate::catalogue::contracts;
use crate::registry::{Coverage, Crate, REGISTRY};
use crate::value::is_prose_shaped;

/// A tally across the five classifications.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StatusCounts {
    /// Asserted byte for byte.
    pub asserted: usize,
    /// Deliberately divergent, with the deviation asserted.
    pub divergent: usize,
    /// Partly asserted; the rest belongs to another crate.
    pub partial: usize,
    /// Prose, owned by a behavioural test.
    pub behavioural: usize,
    /// Not asserted.
    pub pending: usize,
}

impl StatusCounts {
    /// Add `n` to the bucket `coverage` names.
    fn add(&mut self, coverage: &Coverage, n: usize) {
        match coverage {
            Coverage::Asserted { .. } => self.asserted += n,
            Coverage::Divergent { .. } => self.divergent += n,
            Coverage::Partial { .. } => self.partial += n,
            Coverage::Behavioural { .. } => self.behavioural += n,
            Coverage::Pending => self.pending += n,
        }
    }

    /// Everything counted.
    pub const fn total(&self) -> usize {
        self.asserted + self.divergent + self.partial + self.behavioural + self.pending
    }

    /// The contracts that are actually locked: asserted plus divergent.
    ///
    /// A partial assertion is not a lock and a promise is not a lock, so
    /// neither is counted here.
    pub const fn locked(&self) -> usize {
        self.asserted + self.divergent
    }
}

/// One crate's line in the summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrateRow {
    /// The owning crate.
    pub owner: Crate,
    /// Catalogue records owned by it, by classification.
    pub records: StatusCounts,
    /// Registry rows owned by it, by classification.
    pub rows: StatusCounts,
}

/// The whole coverage picture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    /// Catalogue records, by classification.
    pub records: StatusCounts,
    /// Registry rows, by classification.
    pub rows: StatusCounts,
    /// Records whose `exactValue` looks like a literal. Heuristic — see
    /// [`is_prose_shaped`].
    pub literal_shaped: usize,
    /// Records whose `exactValue` looks like prose. Heuristic.
    pub prose_shaped: usize,
    /// Per-crate lines, in `docs/architecture.md` §9 band order.
    pub crates: Vec<CrateRow>,
}

/// Compute the coverage summary from the catalogue and the registry.
///
/// Records whose key has no registry row are not counted anywhere; they are the
/// failure `tests/coverage.rs::every_contract_is_classified` exists to catch,
/// and quietly folding them into a bucket here would hide it.
pub fn coverage_summary() -> Summary {
    let mut records_per_key: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    let mut literal_shaped = 0;
    let mut prose_shaped = 0;
    for contract in contracts() {
        *records_per_key
            .entry((contract.kind.as_str(), contract.name.as_str()))
            .or_default() += 1;
        if is_prose_shaped(&contract.exact_value) {
            prose_shaped += 1;
        } else {
            literal_shaped += 1;
        }
    }

    let mut totals_records = StatusCounts::default();
    let mut totals_rows = StatusCounts::default();
    let mut per_crate: BTreeMap<Crate, (StatusCounts, StatusCounts)> = BTreeMap::new();

    for entry in REGISTRY {
        let records = records_per_key
            .get(&(entry.kind, entry.name))
            .copied()
            .unwrap_or(0);
        totals_records.add(&entry.coverage, records);
        totals_rows.add(&entry.coverage, 1);
        let slot = per_crate.entry(entry.owner).or_default();
        slot.0.add(&entry.coverage, records);
        slot.1.add(&entry.coverage, 1);
    }

    let crates = Crate::ALL
        .iter()
        .map(|owner| {
            let (records, rows) = per_crate.get(owner).copied().unwrap_or_default();
            CrateRow {
                owner: *owner,
                records,
                rows,
            }
        })
        .collect();

    Summary {
        records: totals_records,
        rows: totals_rows,
        literal_shaped,
        prose_shaped,
        crates,
    }
}

impl core::fmt::Display for Summary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "VIA external contract coverage")?;
        writeln!(f, "==============================")?;
        writeln!(f)?;
        writeln!(f, "catalogue          docs/reference/contracts.json")?;
        writeln!(f, "catalogue records  {}", self.records.total())?;
        writeln!(f, "registry rows      {}", self.rows.total())?;
        writeln!(f)?;
        writeln!(f, "status                    records     rows")?;
        for (label, records, rows) in [
            ("asserted", self.records.asserted, self.rows.asserted),
            ("divergent", self.records.divergent, self.rows.divergent),
            ("partial", self.records.partial, self.rows.partial),
            (
                "behavioural",
                self.records.behavioural,
                self.rows.behavioural,
            ),
            ("pending", self.records.pending, self.rows.pending),
        ] {
            writeln!(f, "  {label:<22}{records:>8} {rows:>8}")?;
        }
        writeln!(f, "  {:-<38}", "")?;
        writeln!(
            f,
            "  {:<22}{:>8} {:>8}   asserted + divergent",
            "locked",
            self.records.locked(),
            self.rows.locked()
        )?;
        writeln!(f)?;
        writeln!(f, "exactValue shape (heuristic; reported, never gated)")?;
        writeln!(f, "  {:<22}{:>8}", "literal-shaped", self.literal_shaped)?;
        writeln!(f, "  {:<22}{:>8}", "prose-shaped", self.prose_shaped)?;
        writeln!(f)?;
        writeln!(
            f,
            "owning crate               records  asserted  divergent   partial  behav.   pending"
        )?;
        for row in &self.crates {
            let shipped = if row.owner.exists_today() { "*" } else { " " };
            writeln!(
                f,
                "  {:<23}{shipped}{:>8}{:>10}{:>11}{:>10}{:>8}{:>10}",
                row.owner.as_str(),
                row.records.total(),
                row.records.asserted,
                row.records.divergent,
                row.records.partial,
                row.records.behavioural,
                row.records.pending,
            )?;
        }
        writeln!(f)?;
        write!(f, "  * = crate exists in the tree today")
    }
}
