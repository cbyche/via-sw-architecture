//! Reading `docs/reference/contracts.json`, plus the two lines every test here
//! would otherwise repeat.
//!
//! Every contract value this crate asserts is **parsed out of the catalogue**
//! rather than retyped into a test. A retyped literal proves the code matches
//! the test; a parsed one proves the code matches the specification, and it
//! fails loudly when a contract is renamed or removed.

#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::OnceLock;

use serde::Deserialize;
use via_realtime::{RealtimeSession, ResponseContext};
use via_realtime_mock::{EventLog, MockHandle, MockRealtime, Script, Transcript};

/// One catalogued contract.
#[derive(Clone, Debug, Deserialize)]
pub struct Contract {
    /// `error-code`, `prompt-text`, `default-value`, …
    pub kind: String,
    /// The contract's name.
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

/// The one contract with this name and kind.
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
        [] => panic!("no `{kind}` contract named `{name}`"),
        many => panic!("{} `{kind}` contracts named `{name}`", many.len()),
    }
}

impl Contract {
    /// The integer that follows `marker`.
    #[must_use]
    pub fn number_after(&self, marker: &str) -> i64 {
        let start = self
            .exact_value
            .find(marker)
            .unwrap_or_else(|| panic!("`{marker}` is not in contract `{}`", self.name))
            + marker.len();
        let digits: String = self.exact_value[start..]
            .chars()
            .skip_while(|c| *c == ' ' || *c == '=')
            .take_while(|c| c.is_ascii_digit() || *c == '_')
            .filter(|c| *c != '_')
            .collect();
        digits
            .parse()
            .unwrap_or_else(|_| panic!("no number after `{marker}` in `{}`", self.name))
    }

    /// Every `'…'`-quoted fragment, in order.
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

    /// The catalogue's `\n` escapes as real newlines.
    #[must_use]
    pub fn unescaped(&self) -> String {
        self.exact_value.replace("\\n", "\n")
    }

    /// Assert the rationale mentions `needle`.
    ///
    /// `why` is where the catalogue records *what breaks* when a value drifts,
    /// so a test that asserts a behaviour rather than a literal anchors here.
    pub fn assert_why_mentions(&self, needle: &str) {
        assert!(
            self.why.contains(needle),
            "contract `{}` rationale no longer mentions `{needle}`:\n{}",
            self.name,
            self.why
        );
    }

    /// Assert the value mentions `needle`.
    pub fn assert_mentions(&self, needle: &str) {
        assert!(
            self.exact_value.contains(needle),
            "contract `{}` no longer mentions `{needle}`:\n{}",
            self.name,
            self.exact_value
        );
    }
}

/// Open a mock and start draining its events.
///
/// The shape every session test wants: the log drains on its own task, so the
/// test task is free to await session methods — `via-realtime`'s one rule about
/// its bounded event stream.
pub async fn open(script: Script) -> (RealtimeSession, EventLog, MockHandle) {
    MockRealtime::new(script)
        .open()
        .await
        .expect("the mock session opens")
        .with_event_log()
}

/// Open a mock whose two watchdogs are short enough to fire inside a test.
pub async fn open_with(mock: MockRealtime) -> (RealtimeSession, EventLog, MockHandle) {
    mock.open()
        .await
        .expect("the mock session opens")
        .with_event_log()
}

/// The transcript, or a test failure that says why there is none.
pub async fn transcript(handle: &MockHandle) -> Transcript {
    handle
        .transcript()
        .await
        .expect("the script server answers")
}

/// An empty response context — most tests do not correlate anything.
#[must_use]
pub fn no_context() -> ResponseContext {
    ResponseContext::new()
}
