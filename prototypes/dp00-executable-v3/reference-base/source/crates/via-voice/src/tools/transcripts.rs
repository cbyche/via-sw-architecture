//! Per-turn transcripts and the attachments that travelled with them.
//!
//! Ported from `server/src/voice/tools/turn-transcripts.mjs`.
//!
//! # Why anything waits at all
//!
//! A realtime model authors `spawn_thinking` from what it *heard*; the ASR's
//! settled transcript for the same turn arrives independently and often a beat
//! later. Two places need the settled text — the rare model slip that submits
//! no objective, and the verbatim request pinned onto every delegation — so
//! [`TurnTranscripts::transcript`] waits, briefly and with a bound.
//!
//! **The wait never fails.** It resolves to `""` on timeout and on close, so a
//! delegation that cannot get a transcript falls back to the model's own
//! objective rather than stalling the tool call. A tool receipt that waits on
//! ASR is a tool receipt that sometimes never arrives.
//!
//! # Why an owning task
//!
//! `docs/architecture.md` §11. The map is written by the provider event loop
//! and read by the tool handler, and a waiter registered after the record it
//! is waiting for would hang for the full timeout. One task owns both, so
//! registration and recording are totally ordered.

use std::collections::HashMap;
use std::time::Duration;

use indexmap::IndexMap;
use tokio::sync::{mpsc, oneshot};
use tokio_util::task::TaskTracker;

use crate::input::InputPart;
use crate::text::clean;

/// How long [`TurnTranscripts::transcript`] waits.
///
/// **External contract** — `turn-transcripts.mjs:2` (`waitMs = 800`).
pub const DEFAULT_WAIT_MS: u64 = 800;

/// How many turns are remembered.
///
/// **External contract** — `turn-transcripts.mjs:2` (`maxTurns = 20`).
pub const DEFAULT_MAX_TURNS: usize = 20;

/// What a delegation was pinned with.
///
/// **External contract** — `turn-transcripts.mjs:60-69`
/// (`resolveDelegation`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResolvedDelegation {
    /// What the user actually said, or the model's objective when the ASR had
    /// nothing.
    pub original_request: String,
    /// The objective, or the transcript when the model supplied none.
    pub objective: String,
    /// The attachments that travelled with the turn.
    pub input_parts: Vec<InputPart>,
}

#[derive(Debug)]
enum Command {
    Record {
        turn_id: String,
        transcript: String,
    },
    RecordParts {
        turn_id: String,
        parts: Vec<InputPart>,
    },
    Parts {
        turn_id: String,
        reply: oneshot::Sender<Vec<InputPart>>,
    },
    Await {
        turn_id: String,
        reply: oneshot::Sender<String>,
    },
    Close,
}

/// The handle every caller holds.
#[derive(Clone, Debug)]
pub struct TurnTranscripts {
    commands: mpsc::Sender<Command>,
    wait: Duration,
}

/// How many commands may queue before a caller waits.
const COMMAND_BUFFER: usize = 64;

impl TurnTranscripts {
    /// Start the owning task with the contract bounds.
    #[must_use]
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Configure the owning task.
    #[must_use]
    pub fn builder() -> TurnTranscriptsBuilder {
        TurnTranscriptsBuilder {
            wait_ms: DEFAULT_WAIT_MS,
            max_turns: DEFAULT_MAX_TURNS,
            tracker: None,
        }
    }

    /// Record a turn's settled transcript and release every waiter on it.
    ///
    /// An empty `turn_id` records nothing.
    pub async fn record(&self, turn_id: &str, transcript: &str) {
        if turn_id.is_empty() {
            return;
        }
        let _ = self
            .commands
            .send(Command::Record {
                turn_id: turn_id.to_owned(),
                transcript: crate::text::trim(transcript).to_owned(),
            })
            .await;
    }

    /// Record the attachments a turn carried.
    ///
    /// An empty list **removes** the entry rather than storing an empty one,
    /// which is upstream's behaviour and keeps the bounded map honest.
    pub async fn record_parts(&self, turn_id: &str, parts: Vec<InputPart>) {
        if turn_id.is_empty() {
            return;
        }
        let _ = self
            .commands
            .send(Command::RecordParts {
                turn_id: turn_id.to_owned(),
                parts,
            })
            .await;
    }

    /// The attachments a turn carried.
    pub async fn parts(&self, turn_id: &str) -> Vec<InputPart> {
        let (reply, answer) = oneshot::channel();
        if self
            .commands
            .send(Command::Parts {
                turn_id: turn_id.to_owned(),
                reply,
            })
            .await
            .is_err()
        {
            return Vec::new();
        }
        answer.await.unwrap_or_default()
    }

    /// The turn's settled transcript, waiting up to the configured bound.
    ///
    /// Never fails: a timeout, a closed session and an unknown turn all answer
    /// `""`.
    pub async fn transcript(&self, turn_id: &str) -> String {
        let (reply, answer) = oneshot::channel();
        if self
            .commands
            .send(Command::Await {
                turn_id: turn_id.to_owned(),
                reply,
            })
            .await
            .is_err()
        {
            return String::new();
        }
        match tokio::time::timeout(self.wait, answer).await {
            Ok(Ok(value)) => value,
            // Both the timeout and a dropped sender mean "no transcript",
            // which is a fallback rather than a failure.
            _ => String::new(),
        }
    }

    /// Pin what a delegation should carry.
    ///
    /// **External contract** — `turn-transcripts.mjs:60-69`. The ASR transcript
    /// wins for `original_request`, because that is what the *user* asked; the
    /// model's objective wins for `objective`, because that is what it decided
    /// to do. Each falls back to the other.
    pub async fn resolve_delegation(
        &self,
        turn_id: &str,
        supplied_objective: &str,
    ) -> ResolvedDelegation {
        let objective = clean(supplied_objective);
        let transcript = self.transcript(turn_id).await;
        let original_request = clean(if transcript.is_empty() {
            objective.as_str()
        } else {
            transcript.as_str()
        });
        let parts = self.parts(turn_id).await;
        ResolvedDelegation {
            objective: if objective.is_empty() {
                original_request.clone()
            } else {
                objective
            },
            original_request,
            input_parts: parts,
        }
    }

    /// Release every waiter with `""` and drop the attachments.
    ///
    /// **External contract** — `turn-transcripts.mjs:71-75`. Called on socket
    /// close, so a delegation still in flight is not left waiting on a session
    /// that has gone.
    pub async fn close(&self) {
        let _ = self.commands.send(Command::Close).await;
    }
}

impl Default for TurnTranscripts {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds a [`TurnTranscripts`].
#[derive(Debug)]
pub struct TurnTranscriptsBuilder {
    wait_ms: u64,
    max_turns: usize,
    tracker: Option<TaskTracker>,
}

impl TurnTranscriptsBuilder {
    /// Override the wait bound.
    #[must_use]
    pub const fn wait_ms(mut self, wait_ms: u64) -> Self {
        self.wait_ms = wait_ms;
        self
    }

    /// Override how many turns are remembered.
    #[must_use]
    pub const fn max_turns(mut self, max_turns: usize) -> Self {
        self.max_turns = max_turns;
        self
    }

    /// Register the owning task on a shared tracker.
    #[must_use]
    pub fn tracker(mut self, tracker: TaskTracker) -> Self {
        self.tracker = Some(tracker);
        self
    }

    /// Start the owning task.
    #[must_use]
    pub fn build(self) -> TurnTranscripts {
        let (commands, inbox) = mpsc::channel(COMMAND_BUFFER);
        let actor = Actor {
            max_turns: self.max_turns,
            values: IndexMap::new(),
            input_parts: IndexMap::new(),
            waiters: HashMap::new(),
        };
        let future = actor.run(inbox);
        match self.tracker {
            Some(tracker) => {
                tracker.spawn(future);
            }
            None => {
                tokio::spawn(future);
            }
        }
        TurnTranscripts {
            commands,
            wait: Duration::from_millis(self.wait_ms),
        }
    }
}

struct Actor {
    max_turns: usize,
    values: IndexMap<String, String>,
    input_parts: IndexMap<String, Vec<InputPart>>,
    waiters: HashMap<String, Vec<oneshot::Sender<String>>>,
}

impl Actor {
    async fn run(mut self, mut inbox: mpsc::Receiver<Command>) {
        while let Some(command) = inbox.recv().await {
            match command {
                Command::Record {
                    turn_id,
                    transcript,
                } => {
                    self.values.insert(turn_id.clone(), transcript.clone());
                    while self.values.len() > self.max_turns {
                        if self.values.shift_remove_index(0).is_none() {
                            break;
                        }
                    }
                    for waiter in self.waiters.remove(&turn_id).unwrap_or_default() {
                        let _ = waiter.send(transcript.clone());
                    }
                }
                Command::RecordParts { turn_id, parts } => {
                    if parts.is_empty() {
                        self.input_parts.shift_remove(&turn_id);
                    } else {
                        self.input_parts.insert(turn_id, parts);
                    }
                    while self.input_parts.len() > self.max_turns {
                        if self.input_parts.shift_remove_index(0).is_none() {
                            break;
                        }
                    }
                }
                Command::Parts { turn_id, reply } => {
                    let _ = reply.send(self.input_parts.get(&turn_id).cloned().unwrap_or_default());
                }
                Command::Await { turn_id, reply } => {
                    if let Some(value) = self.values.get(&turn_id) {
                        let _ = reply.send(value.clone());
                    } else {
                        self.waiters.entry(turn_id).or_default().push(reply);
                    }
                }
                Command::Close => {
                    for waiters in std::mem::take(&mut self.waiters).into_values() {
                        for waiter in waiters {
                            let _ = waiter.send(String::new());
                        }
                    }
                    self.input_parts.clear();
                }
            }
        }
        // The handle was dropped: release anything still waiting.
        for waiters in std::mem::take(&mut self.waiters).into_values() {
            for waiter in waiters {
                let _ = waiter.send(String::new());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn image(seed: &str) -> InputPart {
        InputPart::file("image/png", format!("data:image/png;base64,{seed}"))
    }

    #[tokio::test(start_paused = true)]
    async fn a_recorded_transcript_is_returned_immediately() {
        let transcripts = TurnTranscripts::new();
        transcripts.record("voice-1", "  查一下天气  ").await;
        assert_eq!(transcripts.transcript("voice-1").await, "查一下天气");
    }

    #[tokio::test(start_paused = true)]
    async fn a_waiter_registered_first_is_released_by_the_record() {
        let transcripts = TurnTranscripts::new();
        let waiting = {
            let transcripts = transcripts.clone();
            tokio::spawn(async move { transcripts.transcript("voice-1").await })
        };
        tokio::time::sleep(Duration::from_millis(100)).await;
        transcripts.record("voice-1", "晚点提醒我").await;
        assert_eq!(waiting.await.expect("the waiter joined"), "晚点提醒我");
    }

    #[tokio::test(start_paused = true)]
    async fn the_wait_never_fails_it_times_out_to_empty() {
        let transcripts = TurnTranscripts::builder().wait_ms(200).build();
        let started = tokio::time::Instant::now();
        assert_eq!(transcripts.transcript("never-recorded").await, "");
        assert!(started.elapsed() >= Duration::from_millis(200));
    }

    #[tokio::test(start_paused = true)]
    async fn several_waiters_on_one_turn_are_all_released() {
        let transcripts = TurnTranscripts::new();
        let waiters: Vec<_> = (0..3)
            .map(|_| {
                let transcripts = transcripts.clone();
                tokio::spawn(async move { transcripts.transcript("voice-1").await })
            })
            .collect();
        tokio::time::sleep(Duration::from_millis(50)).await;
        transcripts.record("voice-1", "好").await;
        for waiter in waiters {
            assert_eq!(waiter.await.expect("joined"), "好");
        }
    }

    #[tokio::test(start_paused = true)]
    async fn close_releases_every_waiter_with_empty() {
        let transcripts = TurnTranscripts::builder().wait_ms(60_000).build();
        let waiting = {
            let transcripts = transcripts.clone();
            tokio::spawn(async move { transcripts.transcript("voice-1").await })
        };
        tokio::time::sleep(Duration::from_millis(50)).await;
        transcripts.close().await;
        assert_eq!(
            waiting.await.expect("joined"),
            "",
            "no waiter outlives the session"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn attachments_round_trip_and_an_empty_list_removes_them() {
        let transcripts = TurnTranscripts::new();
        transcripts
            .record_parts("voice-1", vec![image("AAAA")])
            .await;
        assert_eq!(transcripts.parts("voice-1").await.len(), 1);
        transcripts.record_parts("voice-1", Vec::new()).await;
        assert!(transcripts.parts("voice-1").await.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn the_transcript_map_is_bounded_oldest_first() {
        let transcripts = TurnTranscripts::builder().max_turns(2).wait_ms(10).build();
        for turn in ["a", "b", "c"] {
            transcripts.record(turn, turn).await;
        }
        assert_eq!(transcripts.transcript("a").await, "", "evicted");
        assert_eq!(transcripts.transcript("b").await, "b");
        assert_eq!(transcripts.transcript("c").await, "c");
    }

    #[tokio::test(start_paused = true)]
    async fn the_parts_map_is_bounded_too() {
        let transcripts = TurnTranscripts::builder().max_turns(2).build();
        for turn in ["a", "b", "c"] {
            transcripts.record_parts(turn, vec![image(turn)]).await;
        }
        assert!(transcripts.parts("a").await.is_empty());
        assert_eq!(transcripts.parts("c").await.len(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn the_delegation_prefers_the_transcript_for_what_the_user_said() {
        let transcripts = TurnTranscripts::new();
        transcripts.record("voice-1", "帮我看看  昨天的日志").await;
        transcripts
            .record_parts("voice-1", vec![image("AAAA")])
            .await;
        let resolved = transcripts
            .resolve_delegation("voice-1", "  检查 昨天的  错误日志 ")
            .await;
        assert_eq!(resolved.original_request, "帮我看看 昨天的日志");
        assert_eq!(resolved.objective, "检查 昨天的 错误日志");
        assert_eq!(resolved.input_parts.len(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn each_field_falls_back_to_the_other() {
        let transcripts = TurnTranscripts::builder().wait_ms(20).build();
        // No transcript: the objective supplies both.
        let no_transcript = transcripts.resolve_delegation("voice-1", "查天气").await;
        assert_eq!(no_transcript.original_request, "查天气");
        assert_eq!(no_transcript.objective, "查天气");

        // No objective: the transcript supplies both.
        transcripts.record("voice-2", "查天气").await;
        let no_objective = transcripts.resolve_delegation("voice-2", "  ").await;
        assert_eq!(no_objective.original_request, "查天气");
        assert_eq!(no_objective.objective, "查天气");
    }

    #[tokio::test(start_paused = true)]
    async fn a_delegation_on_a_closed_session_still_answers() {
        let transcripts = TurnTranscripts::builder().wait_ms(60_000).build();
        let resolving = {
            let transcripts = transcripts.clone();
            tokio::spawn(async move { transcripts.resolve_delegation("voice-1", "查天气").await })
        };
        tokio::time::sleep(Duration::from_millis(50)).await;
        transcripts.close().await;
        let resolved = resolving.await.expect("joined");
        assert_eq!(resolved.objective, "查天气");
        assert!(resolved.input_parts.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn an_empty_turn_id_records_nothing() {
        let transcripts = TurnTranscripts::builder().wait_ms(10).build();
        transcripts.record("", "ignored").await;
        transcripts.record_parts("", vec![image("AAAA")]).await;
        assert_eq!(transcripts.transcript("").await, "");
        assert!(transcripts.parts("").await.is_empty());
    }
}
