//! Shared test scaffolding: the contract catalogue, a deterministic clock, and
//! an event recorder.
//!
//! Every catalogued value this crate's tests assert is **parsed out of
//! `docs/reference/contracts.json`**, never retyped. A retyped literal proves
//! only that two copies of the same mistake agree.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;
use tokio::sync::broadcast;
use via_work::{NowFn, WorkEvent, WorkEventKind, tokio_clock};

/// The epoch instant every test's clock starts at: 2023-11-14T22:13:20Z.
///
/// A fixed, obviously-synthetic value, so an `elapsedMs` in a failure message
/// is readable arithmetic rather than a wall-clock reading.
pub const BASE_MS: i64 = 1_700_000_000_000;

/// A clock that advances with `#[tokio::test(start_paused = true)]`.
///
/// # Panics
///
/// Needs a tokio runtime.
#[must_use]
pub fn clock() -> NowFn {
    tokio_clock(BASE_MS)
}

/// Let every ready task run, and every already-due timer fire.
///
/// Under paused time the runtime only advances the clock once nothing can make
/// progress, so a one-millisecond sleep is "run everything that is runnable,
/// then come back" — deterministic, and independent of how many hops a
/// runner → actor → timer round trip happens to take.
pub async fn settle() {
    tokio::time::sleep(Duration::from_millis(1)).await;
}

/// Advance the clock by `millis`, letting everything due fire.
pub async fn advance(millis: u64) {
    tokio::time::sleep(Duration::from_millis(millis)).await;
    settle().await;
}

/// Records every event the manager publishes.
pub struct Events {
    receiver: broadcast::Receiver<WorkEvent>,
    seen: Vec<WorkEvent>,
    taken: usize,
}

impl Events {
    /// Start recording from now.
    #[must_use]
    pub fn new(receiver: broadcast::Receiver<WorkEvent>) -> Self {
        Self {
            receiver,
            seen: Vec::new(),
            taken: 0,
        }
    }

    /// Take everything published since the last call, and remember it.
    ///
    /// # Panics
    ///
    /// If the recorder lagged. A test that drops events is a test that proves
    /// nothing about ordering, so this fails loudly rather than continuing.
    pub fn pump(&mut self) -> &[WorkEvent] {
        loop {
            match self.receiver.try_recv() {
                Ok(event) => self.seen.push(event),
                Err(
                    broadcast::error::TryRecvError::Empty | broadcast::error::TryRecvError::Closed,
                ) => break,
                Err(broadcast::error::TryRecvError::Lagged(count)) => {
                    panic!("the event recorder lagged by {count}; raise the event capacity")
                }
            }
        }
        &self.seen
    }

    /// Every event so far, in order.
    pub fn all(&mut self) -> Vec<WorkEvent> {
        self.pump().to_vec()
    }

    /// Every event since the last [`Self::take`], in order.
    ///
    /// For a harness that acts on events: acting on the same event twice is
    /// exactly the duplication these tests exist to catch, so a subscriber
    /// consumes rather than re-reads.
    pub fn take(&mut self) -> Vec<WorkEvent> {
        self.pump();
        let fresh = self.seen[self.taken..].to_vec();
        self.taken = self.seen.len();
        fresh
    }

    /// Every event name so far, in order.
    pub fn kinds(&mut self) -> Vec<WorkEventKind> {
        self.pump().iter().map(|event| event.kind).collect()
    }

    /// Every event name raised for `work_id`, in order.
    pub fn kinds_for(&mut self, work_id: &str) -> Vec<WorkEventKind> {
        self.pump()
            .iter()
            .filter(|event| event.task.id == work_id)
            .map(|event| event.kind)
            .collect()
    }

    /// The first event of this kind, if any.
    pub fn first(&mut self, kind: WorkEventKind) -> Option<WorkEvent> {
        self.pump().iter().find(|event| event.kind == kind).cloned()
    }

    /// How many events of this kind were raised.
    pub fn count(&mut self, kind: WorkEventKind) -> usize {
        self.pump()
            .iter()
            .filter(|event| event.kind == kind)
            .count()
    }

    /// Whether an event of this kind was raised.
    pub fn saw(&mut self, kind: WorkEventKind) -> bool {
        self.count(kind) > 0
    }
}

/// Write a `tasks.json` holding `tasks`, and answer its path.
///
/// The document shape is the catalogued one, so a seeded store is read back by
/// exactly the path a real one is.
///
/// # Panics
///
/// If the file cannot be written.
#[must_use]
pub fn seed_tasks(directory: &Path, tasks: Value) -> PathBuf {
    let path = directory.join("tasks.json");
    let document = serde_json::json!({ "version": 1, "tasks": tasks });
    let body = serde_json::to_string_pretty(&document).expect("serializes");
    std::fs::write(&path, format!("{body}\n")).expect("seeded");
    path
}

/// Read a `tasks.json` back as an array of records.
///
/// # Panics
///
/// If the file is missing or is not the catalogued document.
#[must_use]
pub fn read_tasks(path: &Path) -> Vec<Value> {
    let raw = std::fs::read_to_string(path).expect("written");
    let document: Value = serde_json::from_str(&raw).expect("valid JSON");
    document["tasks"].as_array().expect("a tasks array").clone()
}

// ── the contract catalogue ─────────────────────────────────────────────────

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
        .expect("crates/via-work sits two levels below the repository root")
        .to_path_buf()
}

/// The whole contract catalogue.
///
/// # Panics
///
/// If `docs/reference/contracts.json` is missing or malformed. A test run that
/// silently skipped the specification would report success while asserting
/// nothing.
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
/// If there is no such contract, or more than one.
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

/// Every `exactValue` with this `kind` and `name`, in catalogue order.
///
/// Two entries share a name where the catalogue records the same artefact from
/// two call sites; a test that wants both reads them here.
///
/// # Panics
///
/// If there is no such contract.
#[must_use]
pub fn contract_values(kind: &str, name: &str) -> Vec<String> {
    let matches: Vec<String> = contracts()
        .iter()
        .filter(|entry| entry["kind"] == kind && entry["name"] == name)
        .filter_map(|entry| entry["exactValue"].as_str().map(str::to_owned))
        .collect();
    assert!(!matches.is_empty(), "no `{kind}` contract named `{name}`");
    matches
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

/// Every `"key":` in `text`, in order.
///
/// The catalogue states a JSON payload's shape as prose that happens to be
/// JSON-ish; scanning the quoted keys out of it is how a test compares field
/// *order* against a real rendering without retyping the payload.
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

/// The `a | b | c` vocabulary in `text`, trimmed.
#[must_use]
pub fn bar_list(text: &str) -> Vec<String> {
    text.split('|')
        .map(|token| token.trim().trim_matches('\'').trim().to_owned())
        .filter(|token| !token.is_empty())
        .collect()
}

/// Every single-quoted token in `text`, in order.
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

/// Every `NAME` immediately followed by `(<number>` in `text`, as
/// `(name, value)`.
///
/// The catalogue states the environment surface as
/// `VIA_TASK_TERMINAL_TTL_MS (86_400_000, min 60_000); …`, so the numbers can
/// be read out of it rather than retyped.
#[must_use]
pub fn numbers_in(text: &str) -> Vec<i64> {
    let mut numbers = Vec::new();
    let mut digits = String::new();
    for character in text.chars() {
        if character.is_ascii_digit() {
            digits.push(character);
        } else if character == '_' && !digits.is_empty() {
            // `86_400_000` — the separator is part of the number.
        } else {
            if !digits.is_empty()
                && let Ok(value) = digits.parse::<i64>()
            {
                numbers.push(value);
            }
            digits.clear();
        }
    }
    if !digits.is_empty()
        && let Ok(value) = digits.parse::<i64>()
    {
        numbers.push(value);
    }
    numbers
}

/// The text between `marker` and the next `end`, trimmed.
///
/// # Panics
///
/// If `marker` is absent.
#[must_use]
pub fn segment_after(text: &str, marker: &str, end: char) -> String {
    let rest = text
        .split_once(marker)
        .unwrap_or_else(|| panic!("the catalogue entry has no `{marker}`"))
        .1;
    rest.split(end).next().unwrap_or(rest).trim().to_owned()
}

/// The clause of a `;`-separated catalogue entry that mentions `needle`.
///
/// # Panics
///
/// If no clause mentions it.
#[must_use]
pub fn clause_with(text: &str, needle: &str) -> String {
    text.split(';')
        .find(|clause| clause.contains(needle))
        .map(|clause| clause.trim().to_owned())
        .unwrap_or_else(|| panic!("no clause of the catalogue entry mentions `{needle}`"))
}
