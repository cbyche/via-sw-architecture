//! Shared test helpers: the contract catalogue, and the twelve upstream
//! capability declarations.
//!
//! Every catalogued value this crate's tests assert is **parsed out of
//! `docs/reference/contracts.json`**, never retyped. A retyped literal proves
//! only that two copies of the same mistake agree.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use serde_json::Value;
use via_downstream::BackendCapabilities;

/// The repository root, found by walking up from this crate.
///
/// # Panics
///
/// If the crate is not two levels below the workspace root.
#[must_use]
pub fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/via-downstream sits two levels below the repository root")
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
/// If there is no such contract, or if more than one matches. Both are a sign
/// the test is asserting something the catalogue no longer states in the shape
/// the test assumes, which must fail loudly rather than pass vacuously.
#[must_use]
pub fn contract_value(kind: &str, name: &str) -> String {
    let matches: Vec<String> = contracts()
        .iter()
        .filter(|entry| entry["kind"] == kind && entry["name"] == name)
        .filter_map(|entry| entry["exactValue"].as_str().map(str::to_owned))
        .collect();
    match matches.len() {
        1 => matches.into_iter().next().unwrap_or_default(),
        0 => panic!("no `{kind}` contract named `{name}`"),
        count => panic!("{count} `{kind}` contracts named `{name}`; the test must disambiguate"),
    }
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

/// Every `"key":` in `text`, in order, without duplicates removed.
///
/// The catalogue states a JSON payload's shape as prose that happens to be
/// JSON-ish; scanning the quoted keys out of it is how a test compares field
/// *order* against a real `serde_json` rendering without retyping the payload.
#[must_use]
pub fn json_keys(text: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let characters: Vec<char> = text.chars().collect();
    let mut index = 0;
    while index < characters.len() {
        if characters[index] != '"' {
            index += 1;
            continue;
        }
        let start = index + 1;
        let Some(end) = (start..characters.len()).find(|position| characters[*position] == '"')
        else {
            break;
        };
        let after = characters[end + 1..]
            .iter()
            .position(|character| !character.is_whitespace())
            .map(|offset| characters[end + 1 + offset]);
        if after == Some(':') {
            keys.push(characters[start..end].iter().collect());
        }
        index = end + 1;
    }
    keys
}

/// Every single-quoted token in `text`, in order.
///
/// The catalogue states string vocabularies as `'a' | 'b' | 'c'`.
#[must_use]
pub fn quoted_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('\'') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('\'') else { break };
        tokens.push(after[..close].to_owned());
        rest = &after[close + 1..];
    }
    tokens
}

/// Every double-quoted token in `text`, in order.
#[must_use]
pub fn double_quoted_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('"') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('"') else { break };
        tokens.push(after[..close].to_owned());
        rest = &after[close + 1..];
    }
    tokens
}

/// The substring of `text` between `start` and the next `end`, exclusive.
///
/// # Panics
///
/// If either marker is missing — the catalogue no longer says what the test
/// is reading out of it.
#[must_use]
pub fn between<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    let head = text
        .split_once(start)
        .unwrap_or_else(|| panic!("the catalogue entry has no `{start}`"))
        .1;
    match head.split_once(end) {
        Some((slice, _)) => slice,
        None => head,
    }
}

/// Every `<name>: <true|false>` pair in `text`, in order.
#[must_use]
pub fn boolean_pairs(text: &str) -> Vec<(String, bool)> {
    let mut pairs = Vec::new();
    for chunk in text.split(',') {
        let Some((name, value)) = chunk.split_once(':') else {
            continue;
        };
        let name: String = name
            .trim()
            .trim_matches(|character: char| !character.is_alphanumeric())
            .to_owned();
        let value = value.trim().trim_matches(|c: char| !c.is_alphanumeric());
        match value {
            "true" => pairs.push((name, true)),
            "false" => pairs.push((name, false)),
            _ => {}
        }
    }
    pairs
}

/// The twelve capability declarations upstream ships, keyed by backend id.
///
/// Transcribed from `server/src/agent/backends/*.mjs` — the one place in these
/// tests where upstream values are written out, because the catalogue records
/// only DeepSeek's (which `tests/contracts.rs` checks this table against). The
/// four `local-acp` backends share one declaration
/// (`local-acp.mjs:13-21`); `qoder`, `qwen`, `kimi` and `hermes` are those
/// four.
#[must_use]
pub fn upstream_declarations() -> Vec<(&'static str, BackendCapabilities)> {
    const MCP_SESSION: BackendCapabilities = BackendCapabilities {
        delegation: true,
        permissions: true,
        backend_ui: false,
        native_session_history: true,
        external_mcp: true,
        native_delegation: false,
        session_mcp: true,
    };
    vec![
        // opencode.mjs:31-39 — the only session-MCP backend with a web UI.
        (
            "opencode",
            BackendCapabilities {
                backend_ui: true,
                ..MCP_SESSION
            },
        ),
        // openclaw.mjs:39-47 — the only native-delegation backend.
        (
            "openclaw",
            BackendCapabilities {
                delegation: true,
                permissions: true,
                backend_ui: true,
                native_session_history: true,
                external_mcp: false,
                native_delegation: true,
                session_mcp: false,
            },
        ),
        // local-acp.mjs:13-21, shared by the four bundled ACP CLIs.
        ("qoder", MCP_SESSION),
        ("qwen", MCP_SESSION),
        ("kimi", MCP_SESSION),
        ("hermes", MCP_SESSION),
        // codebuddy.mjs:6-14
        ("codebuddy", MCP_SESSION),
        // codex.mjs:9-17
        ("codex", MCP_SESSION),
        // claude.mjs:7-15
        ("claude", MCP_SESSION),
        // deepseek-harness.mjs:7-15 — the one the catalogue records.
        (
            "deepseek",
            BackendCapabilities {
                delegation: false,
                permissions: true,
                backend_ui: false,
                native_session_history: false,
                external_mcp: false,
                native_delegation: false,
                session_mcp: false,
            },
        ),
        // pi.mjs:7-15 — the only backend with no permission gate at all.
        (
            "pi",
            BackendCapabilities {
                delegation: false,
                permissions: false,
                backend_ui: false,
                native_session_history: true,
                external_mcp: false,
                native_delegation: false,
                session_mcp: false,
            },
        ),
        // generic-acp.mjs:10-18
        ("acp", MCP_SESSION),
    ]
}
