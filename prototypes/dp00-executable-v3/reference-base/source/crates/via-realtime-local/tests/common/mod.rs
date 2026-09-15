//! A harness for driving the pipeline, and the contract-catalogue reader.
//!
//! # Why the event log is a task and not a `Vec`
//!
//! `via-realtime`'s own docs state the rule: *"do not await a session method
//! from the same task that drains the stream"* — the session's methods wait on
//! its state task, and the state task may be waiting on the event channel. So
//! the log is drained by a task of its own, and a test asserts against a
//! snapshot.
//!
//! # Every wait has a bound
//!
//! [`EventLog::wait_for`] is a deadline, not a poll loop with a prayer.
//! Phase 6's lesson was that a test whose property is "this terminates" needs
//! its own bound, or a mutant that hangs takes the suite with it rather than
//! failing it. Every wait in this crate's suite goes through it.

#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;
use tokio::sync::Notify;
use via_realtime::{ResponseContext, ResponseOrigin, SessionEvent, SessionEvents};
use via_realtime_local::{PIPELINE_INPUT_RATE, decode_audio, encode_audio};

// ── the catalogue ───────────────────────────────────────────────────────────

/// One catalogued contract.
#[derive(Clone, Debug, Deserialize)]
pub struct Contract {
    /// `default-value`, `json-field`, `env-var`, …
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
    static CATALOGUE: std::sync::OnceLock<Vec<Contract>> = std::sync::OnceLock::new();
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
            .skip_while(|c| *c == ' ' || *c == '=' || *c == ':')
            .take_while(|c| c.is_ascii_digit() || *c == '_' || *c == ',')
            .filter(char::is_ascii_digit)
            .collect();
        digits
            .parse()
            .unwrap_or_else(|_| panic!("no number after `{marker}` in contract `{}`", self.name))
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
}

// ── the event log ───────────────────────────────────────────────────────────

/// One normalized provider event, with what the session attached to it.
#[derive(Clone, Debug)]
pub struct Recorded {
    /// The event as the dialect normalized it.
    pub event: Value,
    /// Who asked for the response it belongs to.
    pub origin: ResponseOrigin,
    /// What the caller attached to that response, when the session could
    /// correlate it.
    pub context: ResponseContext,
    /// Whether the session replayed a refused `response.create` for it.
    pub retried: bool,
}

impl Recorded {
    /// The event's `type`.
    #[must_use]
    pub fn kind(&self) -> &str {
        self.event
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default()
    }
}

#[derive(Debug, Default)]
struct Records {
    events: Vec<Recorded>,
    errors: Vec<String>,
}

/// Everything the session reported, drained by a task of its own.
#[derive(Clone, Debug, Default)]
pub struct EventLog {
    records: Arc<Mutex<Records>>,
    notify: Arc<Notify>,
    closed: Arc<AtomicBool>,
}

/// The longest any wait in this suite will run.
///
/// The bound. Nothing in the pipeline sleeps, so a wait that reaches this is a
/// failure rather than a slow machine.
pub const WAIT_BUDGET: Duration = Duration::from_secs(5);

fn lock(records: &Mutex<Records>) -> MutexGuard<'_, Records> {
    records
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Drain a session's events into a log, on a task of its own.
#[must_use]
pub fn collect_events(mut events: SessionEvents) -> EventLog {
    let log = EventLog::default();
    let sink = log.clone();
    tokio::spawn(async move {
        while let Some(event) = events.recv().await {
            match event {
                SessionEvent::Provider(provider) => {
                    lock(&sink.records).events.push(Recorded {
                        event: provider.event.clone(),
                        origin: provider.origin,
                        context: provider.context.clone(),
                        retried: provider.retried,
                    });
                }
                SessionEvent::Error(error) => {
                    lock(&sink.records).errors.push(error.to_string());
                }
                SessionEvent::Diagnostic(_) => {}
                SessionEvent::Closed => sink.closed.store(true, Ordering::SeqCst),
            }
            sink.notify.notify_waiters();
        }
        sink.closed.store(true, Ordering::SeqCst);
        sink.notify.notify_waiters();
    });
    log
}

impl EventLog {
    /// Every event, in order.
    #[must_use]
    pub fn events(&self) -> Vec<Recorded> {
        lock(&self.records).events.clone()
    }

    /// Every event type, in order.
    #[must_use]
    pub fn types(&self) -> Vec<String> {
        lock(&self.records)
            .events
            .iter()
            .map(|record| record.kind().to_owned())
            .collect()
    }

    /// Every session-level error, in order.
    #[must_use]
    pub fn errors(&self) -> Vec<String> {
        lock(&self.records).errors.clone()
    }

    /// How many events of this type arrived.
    #[must_use]
    pub fn count_of(&self, kind: &str) -> usize {
        lock(&self.records)
            .events
            .iter()
            .filter(|record| record.kind() == kind)
            .count()
    }

    /// The first event of this type.
    #[must_use]
    pub fn first(&self, kind: &str) -> Option<Value> {
        lock(&self.records)
            .events
            .iter()
            .find(|record| record.kind() == kind)
            .map(|record| record.event.clone())
    }

    /// The last event of this type.
    #[must_use]
    pub fn last(&self, kind: &str) -> Option<Value> {
        lock(&self.records)
            .events
            .iter()
            .rev()
            .find(|record| record.kind() == kind)
            .map(|record| record.event.clone())
    }

    /// Every event of this type.
    #[must_use]
    pub fn all(&self, kind: &str) -> Vec<Value> {
        lock(&self.records)
            .events
            .iter()
            .filter(|record| record.kind() == kind)
            .map(|record| record.event.clone())
            .collect()
    }

    /// Whether the session has closed.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    /// Wait until `predicate` holds, or the budget runs out.
    ///
    /// Answers `false` on the deadline rather than hanging: the bound is the
    /// point.
    pub async fn wait_for<F>(&self, predicate: F) -> bool
    where
        F: Fn(&EventLog) -> bool,
    {
        let deadline = tokio::time::Instant::now() + WAIT_BUDGET;
        loop {
            // `enable()` registers the waiter **before** the predicate is
            // checked. Without it, a `notify_waiters()` that lands between the
            // check and the first poll is missed, and the wait rides the full
            // budget for an event that already arrived.
            let notified = self.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if predicate(self) {
                return true;
            }
            if tokio::time::timeout_at(deadline, notified).await.is_err() {
                return predicate(self);
            }
        }
    }

    /// Wait for `count` events of this type.
    pub async fn wait_for_count(&self, kind: &str, count: usize) -> bool {
        let kind = kind.to_owned();
        self.wait_for(move |log| log.count_of(&kind) >= count).await
    }

    /// Wait for one event of this type.
    pub async fn wait_for_type(&self, kind: &str) -> bool {
        self.wait_for_count(kind, 1).await
    }

    /// The assistant's words, as the transcript deltas carried them.
    #[must_use]
    pub fn spoken_text(&self) -> String {
        self.all("response.audio_transcript.delta")
            .iter()
            .filter_map(|event| event.get("delta").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .concat()
    }

    /// Every PCM16 sample the session emitted, in order.
    #[must_use]
    pub fn spoken_samples(&self) -> Vec<i16> {
        self.all("response.audio.delta")
            .iter()
            .filter_map(|event| event.get("delta").and_then(Value::as_str))
            .filter_map(decode_audio)
            .flatten()
            .collect()
    }

    /// Every completed input transcript, in order.
    #[must_use]
    pub fn input_transcripts(&self) -> Vec<String> {
        self.all("conversation.item.input_audio_transcription.completed")
            .iter()
            .filter_map(|event| event.get("transcript").and_then(Value::as_str))
            .map(str::to_owned)
            .collect()
    }

    /// The running input transcript, rendered the way `via-voice` renders it:
    /// `` `${text}${stash}`.trim() ``.
    #[must_use]
    pub fn running_input_transcripts(&self) -> Vec<String> {
        self.all("conversation.item.input_audio_transcription.delta")
            .iter()
            .map(|event| {
                let text = event
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let stash = event
                    .get("stash")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                format!("{text}{stash}").trim().to_owned()
            })
            .collect()
    }

    /// The `response.status` of the last `response.done`.
    #[must_use]
    pub fn last_response_status(&self) -> Option<String> {
        self.last("response.done").and_then(|event| {
            event
                .get("response")
                .and_then(|response| response.get("status"))
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
    }
}

// ── driving audio ───────────────────────────────────────────────────────────

/// One 20 ms block at the pipeline's input rate.
#[must_use]
pub fn block_samples() -> usize {
    PIPELINE_INPUT_RATE.capture_block_frames()
}

/// A block of audible PCM16 — a deterministic ramp, never silence, so a stage
/// that keys on "is this block empty" cannot pass by accident.
#[must_use]
pub fn audible_block() -> Vec<i16> {
    (0..block_samples())
        .map(|index| i16::try_from(index % 1_000).unwrap_or(0))
        .collect()
}

/// Feed `blocks` blocks of audible audio into a session.
pub async fn feed_blocks(
    session: &via_realtime::RealtimeSession,
    blocks: usize,
) -> Result<(), via_realtime::RealtimeError> {
    let encoded = encode_audio(&audible_block());
    for _ in 0..blocks {
        session.append_audio(&encoded).await?;
    }
    Ok(())
}
