//! Reading `docs/reference/contracts.json`.
//!
//! Every catalogued value this crate asserts is **parsed out of the catalogue**
//! rather than retyped into a test. A retyped literal proves the code matches
//! the test; a parsed one proves the code matches the specification, and it
//! fails loudly when a contract is renamed or removed.

#![allow(dead_code)]

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
    /// The value, or a description of it.
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

/// The one contract with this name **and** kind.
///
/// Panics when it is missing or ambiguous, because either means the catalogue
/// moved under a test that claims to assert it.
#[must_use]
pub fn contract_of_kind(kind: &str, name: &str) -> &'static Contract {
    let matches: Vec<&Contract> = catalogue()
        .iter()
        .filter(|contract| contract.name == name && contract.kind == kind)
        .collect();
    match matches.as_slice() {
        [only] => only,
        [] => panic!("no `{kind}` contract named `{name}` in docs/reference/contracts.json"),
        many => panic!("{} `{kind}` contracts named `{name}`", many.len()),
    }
}

impl Contract {
    /// The integer that follows `marker`, `_` and `,` separators ignored.
    #[must_use]
    pub fn number_after(&self, marker: &str) -> u64 {
        let start = self
            .exact_value
            .find(marker)
            .unwrap_or_else(|| panic!("`{marker}` is not in contract `{}`", self.name))
            + marker.len();
        let digits: String = self.exact_value[start..]
            .chars()
            .skip_while(|c| *c == ' ' || *c == '=')
            .take_while(|c| c.is_ascii_digit() || *c == '_' || *c == ',')
            .filter(|c| c.is_ascii_digit())
            .collect();
        digits
            .parse()
            .unwrap_or_else(|_| panic!("no number after `{marker}` in contract `{}`", self.name))
    }

    /// Every `'…'`-quoted fragment, in order.
    #[must_use]
    pub fn single_quoted(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = self.exact_value.as_str();
        while let Some(start) = rest.find('\'') {
            rest = &rest[start + 1..];
            let Some(end) = rest.find('\'') else { break };
            out.push(rest[..end].to_owned());
            rest = &rest[end + 1..];
        }
        out
    }

    /// Assert the contract's value mentions `needle`.
    pub fn assert_mentions(&self, needle: &str) {
        assert!(
            self.exact_value.contains(needle),
            "contract `{}` no longer mentions `{needle}`:\n{}",
            self.name,
            self.exact_value
        );
    }

    /// Assert `keys` appear in the contract's value in exactly this order.
    ///
    /// The catalogue writes a JSON payload as a JavaScript object literal, so
    /// key order in the record *is* the key order on the wire.
    pub fn assert_key_order(&self, keys: &[&str]) {
        let mut cursor = 0usize;
        for key in keys {
            let found = [format!("{key}:"), format!("{key}?:")]
                .iter()
                .filter_map(|needle| {
                    self.exact_value[cursor..]
                        .find(needle)
                        .map(|at| at + needle.len())
                })
                .min()
                .unwrap_or_else(|| {
                    panic!(
                        "key `{key}` is missing from contract `{}` after byte {cursor}:\n{}",
                        self.name, self.exact_value
                    )
                });
            cursor += found;
        }
    }
}
