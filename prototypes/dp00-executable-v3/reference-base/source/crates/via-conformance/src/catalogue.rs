//! The contract catalogue, compiled in.
//!
//! `docs/reference/contracts.json` is the acceptance spec for the whole port
//! (`docs/architecture.md` §14). It is embedded with [`include_str!`] rather
//! than read at runtime so the test binary carries its own specification: a
//! conformance run cannot silently pass because the file was missing, moved, or
//! stale relative to the binary.

use std::sync::OnceLock;

use serde::Deserialize;

/// The catalogue's bytes, exactly as they sit in the tree.
///
/// `docs/reference/contracts.json`, surveyed from upstream
/// `QwenAudio/qwen-audio-agent` v1.11.0 on 2026-08-22.
pub const CONTRACTS_JSON: &str = include_str!("../../../docs/reference/contracts.json");

/// One catalogued external contract.
///
/// Field names mirror the JSON exactly; `exactValue` is the only one that needs
/// a rename, because Rust has no `camelCase` field convention.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Contract {
    /// The contract's kind, e.g. `ws-event`, `model-id`, `default-value`. One
    /// of the 31 kinds tabulated at the head of `docs/reference/contracts.md`.
    pub kind: String,
    /// The contract's name within its kind. Unique together with [`Self::kind`]
    /// for all but the 25 doubly-catalogued contracts — see the crate docs.
    pub name: String,
    /// The value an external party observes, verbatim where it is a literal and
    /// as prose where the contract describes behaviour.
    #[serde(rename = "exactValue")]
    pub exact_value: String,
    /// The upstream file and line range the value was read from.
    pub file: String,
    /// Why the value cannot change without breaking someone.
    pub why: String,
}

impl Contract {
    /// This contract's registry key.
    pub fn key(&self) -> ContractKey<'_> {
        ContractKey {
            kind: &self.kind,
            name: &self.name,
        }
    }
}

/// A registry key: a `(kind, name)` pair.
///
/// Borrowed rather than owned so a `&'static str` literal in the registry and a
/// `String` parsed out of the catalogue compare without allocating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContractKey<'a> {
    /// The contract's kind.
    pub kind: &'a str,
    /// The contract's name.
    pub name: &'a str,
}

impl core::fmt::Display for ContractKey<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}/{}", self.kind, self.name)
    }
}

/// Every catalogued contract, in catalogue order.
///
/// Parsed once. Order is the JSON array's order and is preserved, so a summary
/// or a failure message can quote a contract's position in the file.
///
/// # Panics
///
/// If `docs/reference/contracts.json` is not a JSON array of contract objects.
/// The file is compiled in, so this is a malformed tree rather than a runtime
/// condition — exactly the kind of breakage this crate exists to surface, and
/// it is surfaced by failing loudly rather than by reporting zero contracts.
pub fn contracts() -> &'static [Contract] {
    static CONTRACTS: OnceLock<Vec<Contract>> = OnceLock::new();
    CONTRACTS.get_or_init(|| match serde_json::from_str(CONTRACTS_JSON) {
        Ok(parsed) => parsed,
        Err(error) => panic!("docs/reference/contracts.json is not a valid catalogue: {error}"),
    })
}

/// Every catalogue record filed under `kind` / `name`, in catalogue order.
///
/// Returns more than one record for the 25 doubly-catalogued contracts. Empty
/// when the catalogue has no such contract — callers that need one should use
/// [`expect_contract`], which says so loudly.
pub fn records_for(kind: &str, name: &str) -> Vec<&'static Contract> {
    contracts()
        .iter()
        .filter(|contract| contract.kind == kind && contract.name == name)
        .collect()
}

/// The single catalogue record filed under `kind` / `name`.
///
/// # Panics
///
/// When the catalogue holds no such contract. A test that asks for a contract
/// by name has hard-coded that name; if the catalogue no longer carries it, the
/// test is asserting nothing and must fail rather than pass vacuously.
///
/// Returns the first record when a contract is catalogued more than once. Use
/// [`records_for`] where every spelling matters — the wire-vocabulary tests do.
#[track_caller]
pub fn expect_contract(kind: &str, name: &str) -> &'static Contract {
    match contracts()
        .iter()
        .find(|contract| contract.kind == kind && contract.name == name)
    {
        Some(contract) => contract,
        None => panic!(
            "docs/reference/contracts.json has no `{kind}` contract named `{name}`; \
             a test is asserting against a contract that no longer exists"
        ),
    }
}
