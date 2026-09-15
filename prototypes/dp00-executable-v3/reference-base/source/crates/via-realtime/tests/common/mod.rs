//! Reading `docs/reference/contracts.json`.
//!
//! Every contract value this crate asserts is **parsed out of the catalogue**
//! rather than retyped into a test. A retyped literal proves that the code
//! matches the test; a parsed one proves the code matches the specification, and
//! it fails loudly when a contract is renamed or removed.

#![allow(dead_code)]

pub mod harness;

use std::path::PathBuf;
use std::sync::OnceLock;

use serde::Deserialize;

/// One catalogued contract.
#[derive(Clone, Debug, Deserialize)]
pub struct Contract {
    /// `error-code`, `prompt-text`, `default-value`, …
    pub kind: String,
    /// The contract's name, unique enough to look up by.
    pub name: String,
    /// The value or the description of it.
    #[serde(rename = "exactValue")]
    pub exact_value: String,
    /// Where upstream defines it.
    pub file: String,
    /// Why it is external.
    pub why: String,
}

fn catalogue() -> &'static Vec<Contract> {
    static CATALOGUE: OnceLock<Vec<Contract>> = OnceLock::new();
    CATALOGUE.get_or_init(|| {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("reference")
            .join("contracts.json");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
        serde_json::from_str(&text)
            .unwrap_or_else(|error| panic!("{} must be a contract array: {error}", path.display()))
    })
}

/// The one contract with this exact name.
///
/// Panics when it is missing or ambiguous, because either means the catalogue
/// moved under a test that claims to assert it.
#[must_use]
pub fn contract(name: &str) -> &'static Contract {
    let matches: Vec<&Contract> = catalogue()
        .iter()
        .filter(|contract| contract.name == name)
        .collect();
    match matches.as_slice() {
        [only] => only,
        [] => panic!("no contract named `{name}` in docs/reference/contracts.json"),
        many => panic!(
            "{} contracts named `{name}`; the lookup is ambiguous",
            many.len()
        ),
    }
}

/// The one contract with this name **and** kind.
#[must_use]
pub fn contract_of_kind(kind: &str, name: &str) -> &'static Contract {
    let matches: Vec<&Contract> = catalogue()
        .iter()
        .filter(|contract| contract.name == name && contract.kind == kind)
        .collect();
    match matches.as_slice() {
        [only] => only,
        [] => panic!("no `{kind}` contract named `{name}`"),
        many => panic!("{} `{kind}` contracts named `{name}`", many.len()),
    }
}

/// Every contract whose `file` mentions `needle`.
#[must_use]
pub fn contracts_touching(needle: &str) -> Vec<&'static Contract> {
    catalogue()
        .iter()
        .filter(|contract| contract.file.contains(needle))
        .collect()
}

impl Contract {
    /// The integer that follows `name=` in this contract's value.
    ///
    /// `MAX_ITEMS_PER_LIST=100` → `100`. Panics when the assignment is absent,
    /// which is the point: a renamed constant fails the test rather than
    /// silently asserting nothing.
    #[must_use]
    pub fn number(&self, name: &str) -> i64 {
        let needle = format!("{name}=");
        let start = self
            .exact_value
            .find(&needle)
            .unwrap_or_else(|| panic!("`{name}=` is not in contract `{}`", self.name))
            + needle.len();
        let digits: String = self.exact_value[start..]
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '_')
            .filter(|c| *c != '_')
            .collect();
        digits
            .parse()
            .unwrap_or_else(|_| panic!("`{name}=` in `{}` is not a number", self.name))
    }

    /// The integer that follows `marker` in this contract's value.
    ///
    /// The looser sibling of [`Contract::number`], for values the catalogue
    /// writes as prose (`debounceMs = 30 * 60_000 (1_800_000)`).
    #[must_use]
    pub fn number_after(&self, marker: &str) -> i64 {
        let start = self
            .exact_value
            .find(marker)
            .unwrap_or_else(|| panic!("`{marker}` is not in contract `{}`", self.name))
            + marker.len();
        let digits: String = self.exact_value[start..]
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '_')
            .filter(|c| *c != '_')
            .collect();
        digits
            .parse()
            .unwrap_or_else(|_| panic!("no number after `{marker}` in `{}`", self.name))
    }

    /// The contract's value with the catalogue's own `\n` escapes turned into
    /// real newlines.
    ///
    /// Multi-line prompt text is stored escaped so one contract is one line of
    /// `contracts.json`; comparing it to a real prompt needs it unescaped.
    #[must_use]
    pub fn unescaped(&self) -> String {
        self.exact_value.replace("\\n", "\n")
    }

    /// Whether the contract's value mentions `needle`.
    #[must_use]
    pub fn mentions(&self, needle: &str) -> bool {
        self.exact_value.contains(needle)
    }

    /// Whether the contract's rationale mentions `needle`.
    ///
    /// `why` is where the catalogue records *what breaks* if a value drifts, so
    /// a test that asserts a behaviour rather than a literal anchors itself
    /// here.
    #[must_use]
    pub fn why_mentions(&self, needle: &str) -> bool {
        self.why.contains(needle)
    }

    /// Assert the contract's rationale mentions `needle`.
    pub fn assert_why_mentions(&self, needle: &str) {
        assert!(
            self.why_mentions(needle),
            "contract `{}` rationale no longer mentions `{needle}`:\n{}",
            self.name,
            self.why
        );
    }

    /// Assert the contract's value mentions `needle`.
    pub fn assert_mentions(&self, needle: &str) {
        assert!(
            self.mentions(needle),
            "contract `{}` no longer mentions `{needle}`:\n{}",
            self.name,
            self.exact_value
        );
    }

    /// Every `'…'`-quoted fragment in the contract's value.
    ///
    /// The catalogue quotes error messages and enum members this way, so it is
    /// how a message table is read back out.
    #[must_use]
    pub fn single_quoted(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = self.exact_value.as_str();
        while let Some(open) = rest.find('\'') {
            rest = &rest[open + 1..];
            let Some(close) = rest.find('\'') else { break };
            out.push(rest[..close].to_owned());
            rest = &rest[close + 1..];
        }
        out
    }

    /// The `|`-separated members of a code list, trimmed of any parenthesised
    /// message that follows each one.
    ///
    /// `a | b ('x') | c` → `["a", "b", "c"]`.
    #[must_use]
    pub fn code_list(&self) -> Vec<String> {
        self.exact_value
            .split('|')
            .map(|part| {
                part.trim()
                    .split(['(', ' '])
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .trim_end_matches(',')
                    .to_owned()
            })
            .filter(|code| !code.is_empty())
            .collect()
    }
}
