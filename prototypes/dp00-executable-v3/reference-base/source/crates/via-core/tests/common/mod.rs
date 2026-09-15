//! Shared test helpers: the contract catalogue, and a deterministic host.
//!
//! Every contract value asserted by this crate's tests is **parsed out of
//! `docs/reference/contracts.json`**, never retyped. A retyped literal proves
//! only that two copies of the same mistake agree.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;
use via_core::config::Overrides;
use via_core::env::EnvMap;

/// A deterministic home directory, so a snapshot is stable on any machine.
pub const HOME: &str = "/home/via";
/// A deterministic working directory.
pub const CWD: &str = "/srv/via";
/// A deterministic installation root.
pub const ROOT: &str = "/opt/via";

/// Overrides that pin every host fact.
#[must_use]
pub fn overrides() -> Overrides {
    Overrides {
        home_directory: PathBuf::from(HOME),
        working_directory: PathBuf::from(CWD),
        runtime_root: Some(PathBuf::from(ROOT)),
        gateway: via_core::GatewayOptions::default(),
    }
}

/// Build an [`EnvMap`] from pairs.
#[must_use]
pub fn env(pairs: &[(&str, &str)]) -> EnvMap {
    pairs.iter().copied().collect()
}

/// The repository root, found by walking up from this crate.
#[must_use]
pub fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/via-core sits two levels below the repository root")
        .to_path_buf()
}

/// The whole contract catalogue.
///
/// # Panics
///
/// If `docs/reference/contracts.json` is missing or malformed. That is a
/// deliberate hard failure: the catalogue *is* the specification, and a test
/// run that silently skipped it would report success while asserting nothing.
#[must_use]
pub fn contracts() -> Vec<Value> {
    let path = repository_root().join("docs/reference/contracts.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("contract catalogue at {}: {error}", path.display()));
    serde_json::from_str::<Vec<Value>>(&raw)
        .unwrap_or_else(|error| panic!("contract catalogue is not a JSON array: {error}"))
}

/// The `exactValue` of the one contract with this `kind` and `name`.
///
/// # Panics
///
/// If there is no such contract. The catalogue is versioned alongside the code,
/// so a missing entry means the test is asserting something that is no longer a
/// contract, which must fail loudly rather than pass vacuously.
#[must_use]
pub fn contract_value(kind: &str, name: &str) -> String {
    contracts()
        .iter()
        .find(|entry| entry["kind"] == kind && entry["name"] == name)
        .and_then(|entry| entry["exactValue"].as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| panic!("no `{kind}` contract named `{name}`"))
}

/// The `why` of the one contract with this `kind` and `name`.
///
/// # Panics
///
/// If there is no such contract.
#[must_use]
pub fn contract_why(kind: &str, name: &str) -> String {
    contracts()
        .iter()
        .find(|entry| entry["kind"] == kind && entry["name"] == name)
        .and_then(|entry| entry["why"].as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| panic!("no `{kind}` contract named `{name}`"))
}

/// The concatenated `exactValue` of every contract in the catalogue.
///
/// Several numeric settings are stated in more than one entry; scanning the
/// whole catalogue picks each up wherever it is recorded.
#[must_use]
pub fn all_exact_values() -> String {
    contracts()
        .iter()
        .filter_map(|entry| entry["exactValue"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// One numeric setting as the catalogue states it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CataloguedSetting {
    /// The default value.
    pub default: i64,
    /// The lower clamp, when the catalogue states one.
    pub min: Option<i64>,
    /// The upper clamp, when the catalogue states one.
    pub max: Option<i64>,
}

/// Every `<NAME> default <n> [min <n>] [range [<lo>,<hi>]]` the catalogue
/// states, keyed by the **VIA** environment-variable name.
///
/// The upstream name in the catalogue is pushed through the documented rename
/// rule rather than compared against a retyped VIA name, so a *partial* rename
/// fails this test rather than passing it.
#[must_use]
pub fn catalogued_settings() -> BTreeMap<String, CataloguedSetting> {
    let text = all_exact_values();
    let mut settings: BTreeMap<String, CataloguedSetting> = BTreeMap::new();
    for (name, setting) in scan_settings(&text) {
        let renamed = rebranded(&name);
        // A repeated statement of the same setting must agree with itself.
        if let Some(existing) = settings.get(&renamed) {
            assert_eq!(
                *existing, setting,
                "the catalogue states `{name}` twice, inconsistently"
            );
        }
        settings.insert(renamed, setting);
    }
    settings
}

/// Apply `docs/rebrand.md`'s environment-variable rule.
///
/// `QWEN_AUDIO_AGENT_*`, `QWEN_AUDIO_*` and `QWAUDIO_*` all become `VIA_*`;
/// everything else is a vendor namespace and is left alone. Longest prefix
/// first, or `QWEN_AUDIO_AGENT_X` would become `VIA_AGENT_X`.
#[must_use]
pub fn rebranded(name: &str) -> String {
    for prefix in ["QWEN_AUDIO_AGENT_", "QWEN_AUDIO_", "QWAUDIO_"] {
        if let Some(rest) = name.strip_prefix(prefix) {
            return format!("VIA_{rest}");
        }
    }
    name.to_owned()
}

/// Scan prose for `NAME default N`, with optional `min N` / `range [a,b]`.
fn scan_settings(text: &str) -> Vec<(String, CataloguedSetting)> {
    let mut found = Vec::new();
    let bytes: Vec<char> = text.chars().collect();
    let mut index = 0usize;
    while index < bytes.len() {
        let Some(start) = find_from(&bytes, index, " default ") else {
            break;
        };
        // Walk back over the variable name.
        let mut name_start = start;
        while name_start > 0 && is_name_char(bytes[name_start - 1]) {
            name_start -= 1;
        }
        let name: String = bytes[name_start..start].iter().collect();
        let mut cursor = start + " default ".len();
        let Some(default) = read_number(&bytes, &mut cursor) else {
            index = start + 1;
            continue;
        };
        if !name.starts_with("QWEN_") && !name.starts_with("QWAUDIO_") {
            index = cursor;
            continue;
        }
        let mut setting = CataloguedSetting {
            default,
            min: None,
            max: None,
        };
        // ` min N` and ` max N`, in either order, or ` range [lo,hi]`.
        loop {
            if let Some(rest) = starts_with_at(&bytes, cursor, " min ") {
                let mut after = rest;
                if let Some(value) = read_number(&bytes, &mut after) {
                    setting.min = Some(value);
                    cursor = after;
                    continue;
                }
            }
            if let Some(rest) = starts_with_at(&bytes, cursor, " max ") {
                let mut after = rest;
                if let Some(value) = read_number(&bytes, &mut after) {
                    setting.max = Some(value);
                    cursor = after;
                    continue;
                }
            }
            if let Some(rest) = starts_with_at(&bytes, cursor, " range [") {
                let mut after = rest;
                if let Some(low) = read_number(&bytes, &mut after)
                    && bytes.get(after) == Some(&',')
                {
                    after += 1;
                    while bytes.get(after) == Some(&' ') {
                        after += 1;
                    }
                    if let Some(high) = read_number(&bytes, &mut after) {
                        setting.min = Some(low);
                        setting.max = Some(high);
                        cursor = after;
                        continue;
                    }
                }
            }
            break;
        }
        found.push((name, setting));
        index = cursor;
    }
    found
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'
}

fn find_from(haystack: &[char], from: usize, needle: &str) -> Option<usize> {
    let needle: Vec<char> = needle.chars().collect();
    (from..haystack.len().saturating_sub(needle.len() - 1))
        .find(|start| haystack[*start..*start + needle.len()] == needle[..])
}

fn starts_with_at(haystack: &[char], at: usize, needle: &str) -> Option<usize> {
    let needle: Vec<char> = needle.chars().collect();
    if at + needle.len() > haystack.len() {
        return None;
    }
    (haystack[at..at + needle.len()] == needle[..]).then_some(at + needle.len())
}

fn read_number(haystack: &[char], cursor: &mut usize) -> Option<i64> {
    let start = *cursor;
    let mut digits = String::new();
    while let Some(c) = haystack.get(*cursor) {
        if c.is_ascii_digit() {
            digits.push(*c);
            *cursor += 1;
        } else if *c == '_' && !digits.is_empty() {
            *cursor += 1;
        } else {
            break;
        }
    }
    if digits.is_empty() {
        *cursor = start;
        return None;
    }
    digits.parse().ok()
}
