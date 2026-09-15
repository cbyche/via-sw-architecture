//! Reading `docs/reference/contracts.json`.
//!
//! Every catalogued value this crate asserts is **parsed out of the
//! catalogue** rather than retyped into a test. A retyped literal proves the
//! code matches the test; a parsed one proves the code matches the
//! specification, and it fails loudly when a contract is renamed or removed.
//!
//! The one thing here that is not in the sibling crates' loaders is
//! [`Contract::js_regexes`]: two of this crate's contracts *are* JavaScript
//! `RegExp` literals, and the only honest way to assert a Rust transcription of
//! them is to compile the catalogued source and compare the two classifiers on
//! the same corpus.

#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::OnceLock;

use regex::Regex;
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
    /// Every `'…'`-quoted fragment, in order.
    #[must_use]
    pub fn single_quoted(&self) -> Vec<String> {
        delimited(&self.exact_value, '\'', '\'')
    }

    /// The `'…'`-quoted fragment that follows `marker`.
    ///
    /// How a table like `a = 'x'; b = 'y'` is read one entry at a time, without
    /// depending on how many other quoted fragments sit between them.
    #[must_use]
    pub fn quoted_after(&self, marker: &str) -> String {
        self.after(marker, '\'', '\'')
    }

    /// The `` `…` ``-quoted template that follows `marker`.
    #[must_use]
    pub fn template_after(&self, marker: &str) -> String {
        self.after(marker, '`', '`')
    }

    fn after(&self, marker: &str, open: char, close: char) -> String {
        let start = self
            .exact_value
            .find(marker)
            .unwrap_or_else(|| panic!("`{marker}` is not in contract `{}`", self.name))
            + marker.len();
        delimited(&self.exact_value[start..], open, close)
            .into_iter()
            .next()
            .unwrap_or_else(|| {
                panic!(
                    "nothing delimited by `{open}` after `{marker}` in contract `{}`",
                    self.name
                )
            })
    }

    /// The integer that follows `marker`, `_` separators ignored.
    #[must_use]
    pub fn number_after(&self, marker: &str) -> u64 {
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
            .unwrap_or_else(|_| panic!("no number after `{marker}` in contract `{}`", self.name))
    }

    /// The contract's value with the catalogue's own `\n` escapes turned into
    /// real newlines.
    #[must_use]
    pub fn unescaped(&self) -> String {
        self.exact_value.replace("\\n", "\n")
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

    /// Assert the contract's rationale mentions `needle`.
    pub fn assert_why_mentions(&self, needle: &str) {
        assert!(
            self.why.contains(needle),
            "contract `{}` rationale no longer mentions `{needle}`:\n{}",
            self.name,
            self.why
        );
    }

    /// The part of this contract between two markers, as a contract of its own.
    ///
    /// The catalogue writes a two-provider table as one record
    /// (`dashscope: {…}; s2s: {…}`), and asserting a key order against the
    /// whole record would let one branch's keys satisfy the other's.
    #[must_use]
    pub fn section(&self, from: &str, to: Option<&str>) -> Self {
        let start = self
            .exact_value
            .find(from)
            .unwrap_or_else(|| panic!("`{from}` is not in contract `{}`", self.name))
            + from.len();
        let rest = &self.exact_value[start..];
        let end = to.map_or(rest.len(), |marker| {
            rest.find(marker)
                .unwrap_or_else(|| panic!("`{marker}` is not after `{from}` in `{}`", self.name))
        });
        Self {
            exact_value: rest[..end].to_owned(),
            name: format!("{} [{from}]", self.name),
            ..self.clone()
        }
    }

    /// Assert `keys` appear in the contract's value in exactly this order.
    ///
    /// The catalogue writes a JSON payload as a JavaScript object literal, so
    /// key order in the record *is* the key order on the wire. Comparing the
    /// order rather than a rendered string keeps the assertion honest about the
    /// one property that matters while staying immune to the catalogue's
    /// shorthand (`<…>` placeholders, prose after the closing brace).
    ///
    /// A key the catalogue marks optional is written `name?:`, and both
    /// spellings are accepted — the `?` says *when* the key is emitted, not
    /// where.
    pub fn assert_key_order(&self, keys: &[String]) {
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

    /// Every `/…/i` JavaScript regex in the contract's value, keyed by the
    /// classification that precedes it.
    ///
    /// The catalogue writes an error corpus as
    /// `name: /a/i; name: /b/i OR /c/i`, so this returns
    /// `[("name", [a]), ("name", [b, c])]` — the order is upstream's evaluation
    /// order and the test depends on it.
    ///
    /// The patterns are compiled with `(?i)`, which is Rust's nearest
    /// equivalent of JavaScript's non-`u` `i` flag; every message in the corpus
    /// this is compared against is ASCII, where the two agree exactly.
    #[must_use]
    pub fn js_regexes(&self) -> Vec<(String, Vec<Regex>)> {
        self.exact_value
            .split("; ")
            .filter_map(|chunk| {
                let (name, rest) = chunk.split_once(": ")?;
                let patterns: Vec<Regex> = delimited(rest, '/', '/')
                    .into_iter()
                    // `delimited` stops at the closing `/`; the JavaScript flag
                    // that follows it is always `i`, which becomes `(?i)`.
                    .map(|source| {
                        Regex::new(&format!("(?i){source}")).unwrap_or_else(|error| {
                            panic!(
                                "`{source}` in contract `{}` must compile: {error}",
                                self.name
                            )
                        })
                    })
                    .collect();
                (!patterns.is_empty()).then(|| (name.trim().to_owned(), patterns))
            })
            .collect()
    }
}

/// Every `open`…`close`-delimited fragment of `text`, in order, non-overlapping.
fn delimited(text: &str, open: char, close: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find(open) {
        rest = &rest[start + open.len_utf8()..];
        let Some(end) = rest.find(close) else { break };
        out.push(rest[..end].to_owned());
        rest = &rest[end + close.len_utf8()..];
    }
    out
}
