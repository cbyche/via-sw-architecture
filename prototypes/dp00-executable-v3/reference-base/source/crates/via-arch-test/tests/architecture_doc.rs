//! The gate against the document, and the document against upstream.
//!
//! `tests/crate_graph.rs` asserts that the workspace obeys [`CRATE_BANDS`] and
//! [`UPSTREAM_ADJACENCY`]. That is only worth something if those two tables are
//! the ones `docs/architecture.md` §9 publishes — otherwise the gate enforces a
//! private rule and the document describes a different, unenforced one.
//!
//! So: parse §9 and compare. Both directions fail loudly, and the failure says
//! which side to edit.

use std::fs;
use std::path::PathBuf;

use pretty_assertions::assert_eq;
use via_arch_test::{Band, CRATE_BANDS, RULE_SOURCE, UPSTREAM_ADJACENCY, workspace_root};

fn architecture_md() -> (PathBuf, String) {
    let path = workspace_root().join("docs").join("architecture.md");
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => panic!(
            "could not read {path}: {error}\n\n\
             {RULE_SOURCE} is the source of the layer table; without it this gate has \
             nothing to check itself against.",
            path = path.display(),
        ),
    };
    (path, text)
}

/// The body of `## 9. Crate graph`, up to the next `## ` heading.
fn section_9(text: &str) -> &str {
    const HEADING: &str = "\n## 9. Crate graph\n";
    let Some(start) = text.find(HEADING) else {
        panic!(
            "could not find the heading `## 9. Crate graph` in docs/architecture.md. \
             If the section was renumbered or renamed, update this parser — the table \
             it reads is the contract, and losing sight of it silently is the one \
             outcome worth failing over."
        );
    };
    let body = &text[start + HEADING.len()..];
    match body.find("\n## ") {
        Some(end) => &body[..end],
        None => body,
    }
}

/// Every backtick-quoted token in a line, in order.
fn backticked(line: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find('`') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('`') else {
            break;
        };
        tokens.push(&after[..close]);
        rest = &after[close + 1..];
    }
    tokens
}

/// `**Layer 1**` -> `layer1`, which is exactly [`Band::name`].
fn normalise_band_label(cell: &str) -> String {
    cell.replace('*', "")
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>()
        .to_lowercase()
}

/// §9's band table is the published form of [`CRATE_BANDS`]. Same crates, same
/// bands, same order.
///
/// `apps/via` is the one entry that differs by construction: §9 names it by
/// path, because it is the sole workspace member outside `crates/`, while the
/// table keys on the **package** name, `via`.
#[test]
fn the_crate_table_matches_the_band_table_in_architecture_md() {
    let (path, text) = architecture_md();
    let section = section_9(&text);

    let mut documented: Vec<(String, Band)> = Vec::new();
    for line in section.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("| **") {
            continue;
        }
        let cells: Vec<&str> = trimmed.split('|').collect();
        let Some(label) = cells.get(1) else { continue };
        let label = normalise_band_label(label);
        let Some(band) = Band::ALL.into_iter().find(|band| band.name() == label) else {
            panic!(
                "docs/architecture.md §9 has a band table row labelled `{label}`, which is \
                 not one of {known:?}. Either the row is a typo or a new band was added \
                 without a Band variant. See {path}.",
                known = Band::ALL.map(Band::name),
                path = path.display(),
            );
        };
        for crate_name in backticked(cells.get(2).copied().unwrap_or_default()) {
            // §9 names the binary by path; the package is `via`.
            let package = if crate_name == "apps/via" {
                "via"
            } else {
                crate_name
            };
            documented.push((package.to_owned(), band));
        }
    }

    assert!(
        !documented.is_empty(),
        "parsed no crates out of the §9 band table in {path}. The table's shape changed; \
         update this parser rather than deleting the assertion.",
        path = path.display(),
    );

    let encoded: Vec<(String, Band)> = CRATE_BANDS
        .iter()
        .map(|(name, band)| ((*name).to_owned(), *band))
        .collect();

    assert_eq!(
        encoded, documented,
        "\nCRATE_BANDS (left) and the band table in {RULE_SOURCE} (right) disagree.\n\n\
         Both are the same contract written twice, deliberately: the document is what a \
         human reads and the table is what CI enforces. Edit both, in the same commit, \
         in the same order.\n",
    );
}

/// §9 quotes upstream's adjacency table because it is "the honest source,
/// because it is executable rather than aspirational". [`UPSTREAM_ADJACENCY`]
/// is that quote, machine-readable — so it has to still be that quote.
#[test]
fn the_upstream_adjacency_table_matches_the_one_quoted_in_architecture_md() {
    let (path, text) = architecture_md();
    let section = section_9(&text);

    let mut documented: Vec<(String, Vec<String>)> = Vec::new();
    for line in section.lines() {
        let Some((layer, targets)) = line.split_once('→') else {
            continue;
        };
        let layer = layer.trim();
        if layer.is_empty() || layer.contains(' ') {
            continue;
        }
        let targets = targets.trim();
        let targets: Vec<String> = if targets == "∅" {
            Vec::new()
        } else {
            targets
                .split(',')
                .map(|target| target.trim().to_owned())
                .collect()
        };
        documented.push((layer.to_owned(), targets));
    }

    assert!(
        !documented.is_empty(),
        "parsed no rows out of the adjacency block in §9 of {path}. The block's shape \
         changed; update this parser rather than deleting the assertion.",
        path = path.display(),
    );

    // §9 quotes the eight rows of upstream's `allowedDependencies` object.
    // UPSTREAM_ADJACENCY carries a ninth, `root`, from the same file's test
    // body — see the constant's own documentation.
    let quoted: Vec<(String, Vec<String>)> = UPSTREAM_ADJACENCY
        .iter()
        .take(documented.len())
        .map(|(layer, targets)| {
            (
                layer.name().to_owned(),
                targets
                    .iter()
                    .map(|target| target.name().to_owned())
                    .collect(),
            )
        })
        .collect();

    assert_eq!(
        quoted, documented,
        "\nUPSTREAM_ADJACENCY (left) and the table quoted in {RULE_SOURCE} (right) \
         disagree.\n\n\
         Both come from server/test/dependency-boundaries.test.mjs:10-18 in \
         qwen-audio-agent v1.11.0. If upstream's table changed, change all three \
         together; if it did not, one of these two was edited by hand.\n",
    );

    let extra: Vec<&str> = UPSTREAM_ADJACENCY
        .iter()
        .skip(documented.len())
        .map(|(layer, _)| layer.name())
        .collect();
    assert_eq!(
        extra,
        vec!["root"],
        "UPSTREAM_ADJACENCY may carry exactly one row beyond the eight §9 quotes: \
         `root`, which upstream special-cases in its test body \
         (dependency-boundaries.test.mjs:48) rather than in the table.",
    );
}
