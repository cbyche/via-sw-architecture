//! Keeping what the session reported.
//!
//! [`SessionEvents`] is a bounded channel and `via-realtime` documents the one
//! rule that comes with it: *do not await a session method from the same task
//! that drains the stream* — those methods wait on the state task, and the state
//! task may be waiting on the stream. Every test that drives a session therefore
//! needs a drainer on its own task, and every one of them would otherwise write
//! the same twenty lines.
//!
//! So it lives here, once. [`collect_events`] spawns the drainer and hands back
//! an [`EventLog`] the test task can read without ever blocking the session.
//!
//! # The one thing to know before reading the log
//!
//! **A settled outcome does not mean a drained log.** A session method resolves
//! when its response *outcome* settles, and the session settles it inside its
//! own state task — before the drainer has necessarily forwarded the events that
//! led to it. Everything up to and including that turn's `response.done` is
//! already queued on the event channel at that moment, so it is only a question
//! of the drainer catching up.
//!
//! Reading [`EventLog::spoken_text`] straight after an awaited call is therefore
//! a race, and the fix is one line: wait for the end of the turn first.
//! [`EventLog::wait_for_turns`] is that line, and because the channel is FIFO,
//! seeing the `response.done` proves every delta before it is in the log too.
//!
//! ```text
//! session.send_user_text(…).await?;   // the outcome settled
//! log.wait_for_turns(1).await;        // …and the log has caught up
//! assert_eq!(log.spoken_text(), "…");
//! ```
//!
//! # Why a mutex is right here and nowhere else
//!
//! `docs/architecture.md` §11 asks for an owning task per *ordering invariant*.
//! This log holds none: [`SessionEvents`] already ordered the events, and the
//! log only appends in the order it receives them. What is shared is a `Vec` a
//! reader copies, so a mutex is exactly the tool — unlike
//! [`crate::server`]'s transcript, which is read *while* the script cursor and
//! the id counters are moving and therefore lives in a task.

use std::sync::{Arc, Mutex, MutexGuard};

use via_realtime::{Diagnostic, ProviderEvent, RealtimeError, SessionEvent, SessionEvents};

/// How many times a `wait_for*` helper yields before giving up.
///
/// Shared by [`EventLog::wait_for`] and
/// [`MockHandle::wait_for_frame`](crate::MockHandle::wait_for_frame): both are
/// waiting for the same tasks to make the same progress.
///
/// Generous: a yield costs nothing and does **not** advance a paused clock, so a
/// test that waits for something that will never arrive fails rather than
/// silently firing a 30-second watchdog on the way.
pub const POLL_BUDGET: usize = 4_096;

/// One thing the session reported, in a form a test can keep.
///
/// The four variants are `via-realtime`'s four, which are upstream's four
/// callbacks — `onEvent`, `onError`, `onDiagnostic`, `onClose` — in the order
/// they happened, because the order between them is information.
#[derive(Debug, Clone)]
pub enum Recorded {
    /// A normalized provider event.
    Provider(ProviderEvent),
    /// Something the session failed at.
    Error(RealtimeError),
    /// A structured observation the session made about itself.
    Diagnostic(Diagnostic),
    /// The socket closed. Always last.
    Closed,
}

impl Recorded {
    /// The provider event, if this is one.
    #[must_use]
    pub fn provider(&self) -> Option<&ProviderEvent> {
        match self {
            Self::Provider(event) => Some(event),
            _ => None,
        }
    }

    /// The failure, if this is one.
    #[must_use]
    pub fn error(&self) -> Option<&RealtimeError> {
        match self {
            Self::Error(error) => Some(error),
            _ => None,
        }
    }

    /// The diagnostic, if this is one.
    #[must_use]
    pub fn diagnostic(&self) -> Option<&Diagnostic> {
        match self {
            Self::Diagnostic(diagnostic) => Some(diagnostic),
            _ => None,
        }
    }
}

/// Everything the session has reported so far.
///
/// Cheap to clone — every clone reads the same log.
#[derive(Clone, Default, Debug)]
pub struct EventLog {
    entries: Arc<Mutex<Vec<Recorded>>>,
}

impl EventLog {
    /// A poison-tolerant lock.
    ///
    /// A panic elsewhere while this guard was held would otherwise turn every
    /// later read into an error and wedge an unrelated test; the log is an
    /// append-only `Vec` with no invariant a partial write could break.
    fn entries(&self) -> MutexGuard<'_, Vec<Recorded>> {
        match self.entries.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn push(&self, entry: Recorded) {
        self.entries().push(entry);
    }

    /// A copy of everything reported so far.
    #[must_use]
    pub fn snapshot(&self) -> Vec<Recorded> {
        self.entries().clone()
    }

    /// Every provider event so far.
    #[must_use]
    pub fn provider_events(&self) -> Vec<ProviderEvent> {
        self.snapshot()
            .iter()
            .filter_map(|entry| entry.provider().cloned())
            .collect()
    }

    /// The `type` of every provider event so far.
    #[must_use]
    pub fn provider_kinds(&self) -> Vec<String> {
        self.provider_events()
            .iter()
            .map(|event| event.kind().to_owned())
            .collect()
    }

    /// Every failure the session reported.
    #[must_use]
    pub fn errors(&self) -> Vec<RealtimeError> {
        self.snapshot()
            .iter()
            .filter_map(|entry| entry.error().cloned())
            .collect()
    }

    /// Every diagnostic the session reported.
    #[must_use]
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.snapshot()
            .iter()
            .filter_map(|entry| entry.diagnostic().cloned())
            .collect()
    }

    /// Whether the session has said goodbye.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.snapshot()
            .iter()
            .any(|entry| matches!(entry, Recorded::Closed))
    }

    /// The concatenated `delta` of every text or transcript delta, which is what
    /// the model actually said.
    ///
    /// Read it **after** [`wait_for_turns`](Self::wait_for_turns), not straight
    /// after the session call that produced it — see the module docs.
    #[must_use]
    pub fn spoken_text(&self) -> String {
        self.provider_events()
            .iter()
            .filter(|event| {
                matches!(
                    event.kind(),
                    "response.text.delta" | "response.audio_transcript.delta"
                )
            })
            .filter_map(|event| {
                event
                    .event
                    .get("delta")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned)
            })
            .collect()
    }

    /// Yield until `predicate` holds, then answer with the snapshot it held on.
    ///
    /// `None` means it never held inside [`POLL_BUDGET`] yields.
    ///
    /// Yields rather than sleeps, deliberately: a test that means *let the tasks
    /// make progress* must not advance a paused clock, or it fires a watchdog it
    /// never meant to test.
    pub async fn wait_for<F>(&self, predicate: F) -> Option<Vec<Recorded>>
    where
        F: Fn(&[Recorded]) -> bool,
    {
        for _ in 0..POLL_BUDGET {
            let snapshot = self.snapshot();
            if predicate(&snapshot) {
                return Some(snapshot);
            }
            tokio::task::yield_now().await;
        }
        None
    }

    /// Yield until a provider event of this type arrives.
    pub async fn wait_for_kind(&self, kind: &str) -> Option<ProviderEvent> {
        self.wait_for(|entries| {
            entries
                .iter()
                .filter_map(Recorded::provider)
                .any(|event| event.kind() == kind)
        })
        .await?
        .iter()
        .filter_map(Recorded::provider)
        .find(|event| event.kind() == kind)
        .cloned()
    }

    /// Yield until `count` provider events of this type have been reported.
    pub async fn wait_for_kinds(&self, kind: &str, count: usize) -> Option<Vec<ProviderEvent>> {
        let matching = |entries: &[Recorded]| -> Vec<ProviderEvent> {
            entries
                .iter()
                .filter_map(Recorded::provider)
                .filter(|event| event.kind() == kind)
                .cloned()
                .collect()
        };
        let entries = self
            .wait_for(|entries| matching(entries).len() >= count)
            .await?;
        Some(matching(&entries))
    }

    /// How many model turns have been reported all the way to `response.done`.
    #[must_use]
    pub fn completed_turns(&self) -> usize {
        self.provider_events()
            .iter()
            .filter(|event| event.kind() == "response.done")
            .count()
    }

    /// Yield until `count` model turns have been reported end to end.
    ///
    /// **The line to write between an awaited session call and an assertion
    /// about what the model said** — see the module docs for why. `false` means
    /// they never arrived inside [`POLL_BUDGET`] yields.
    ///
    /// A turn the provider *refused* never reaches `response.done`; wait for
    /// [`wait_for_kind("error")`](Self::wait_for_kind) instead.
    pub async fn wait_for_turns(&self, count: usize) -> bool {
        self.wait_for_kinds("response.done", count).await.is_some()
    }

    /// Yield until the session says goodbye.
    pub async fn wait_for_close(&self) -> bool {
        self.wait_for(|entries| {
            entries
                .iter()
                .any(|entry| matches!(entry, Recorded::Closed))
        })
        .await
        .is_some()
    }
}

/// Drain a session's events into a log, on their own task.
///
/// The task ends when the session does.
#[must_use]
pub fn collect_events(mut events: SessionEvents) -> EventLog {
    let log = EventLog::default();
    let sink = log.clone();
    tokio::spawn(async move {
        while let Some(event) = events.recv().await {
            sink.push(match event {
                SessionEvent::Provider(event) => Recorded::Provider(*event),
                SessionEvent::Error(error) => Recorded::Error(error),
                SessionEvent::Diagnostic(diagnostic) => Recorded::Diagnostic(diagnostic),
                SessionEvent::Closed => Recorded::Closed,
            });
        }
    });
    log
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use via_realtime::{ResponseContext, ResponseOrigin};

    use super::*;

    fn provider_event(kind: &str, delta: &str) -> Recorded {
        Recorded::Provider(ProviderEvent {
            event: json!({ "type": kind, "delta": delta }),
            origin: ResponseOrigin::Model,
            context: ResponseContext::new(),
            retried: false,
        })
    }

    #[test]
    fn the_log_separates_the_four_kinds() {
        let log = EventLog::default();
        log.push(provider_event("response.text.delta", "he"));
        log.push(Recorded::Error(RealtimeError::SessionReset));
        log.push(Recorded::Diagnostic(Diagnostic {
            event: via_realtime::DIAGNOSTIC_RESPONSE_TIMEOUT,
            provider: "mock".into(),
            response_id: "resp_1".into(),
            phase: "inactivity",
            inactivity_ms: 1,
            elapsed_ms: 2,
        }));
        log.push(Recorded::Closed);

        assert_eq!(log.provider_kinds(), ["response.text.delta"]);
        assert_eq!(log.errors(), vec![RealtimeError::SessionReset]);
        assert_eq!(log.diagnostics().len(), 1);
        assert!(log.is_closed());
        assert_eq!(log.snapshot().len(), 4);
    }

    #[test]
    fn spoken_text_joins_both_delta_spellings_and_ignores_everything_else() {
        let log = EventLog::default();
        log.push(provider_event("response.text.delta", "he"));
        log.push(provider_event("response.audio_transcript.delta", "llo"));
        log.push(provider_event("response.audio.delta", "<base64>"));
        log.push(Recorded::Error(RealtimeError::SessionReset));
        assert_eq!(log.spoken_text(), "hello");
    }

    #[tokio::test]
    async fn waiting_for_something_that_never_arrives_answers_none_rather_than_hanging() {
        let log = EventLog::default();
        assert!(log.wait_for_kind("response.done").await.is_none());
        assert!(!log.wait_for_close().await);
    }

    #[test]
    fn completed_turns_counts_response_done_and_nothing_else() {
        let log = EventLog::default();
        assert_eq!(log.completed_turns(), 0);
        log.push(provider_event("response.done", ""));
        log.push(provider_event("response.created", ""));
        log.push(provider_event("response.done", ""));
        assert_eq!(log.completed_turns(), 2);
    }

    #[tokio::test]
    async fn waiting_for_more_turns_than_arrive_answers_false() {
        let log = EventLog::default();
        log.push(provider_event("response.done", ""));
        assert!(log.wait_for_turns(1).await);
        assert!(!log.wait_for_turns(2).await);
        assert_eq!(
            log.wait_for_kinds("response.done", 1)
                .await
                .map(|events| events.len()),
            Some(1)
        );
        assert!(log.wait_for_kinds("response.created", 1).await.is_none());
    }

    #[tokio::test]
    async fn waiting_finds_an_event_another_task_appends() {
        let log = EventLog::default();
        let writer = log.clone();
        tokio::spawn(async move {
            tokio::task::yield_now().await;
            writer.push(provider_event("response.done", ""));
        });
        let event = log.wait_for_kind("response.done").await.expect("arrives");
        assert_eq!(event.kind(), "response.done");
    }

    #[tokio::test]
    async fn a_dropped_session_stream_ends_the_drainer() {
        // A `SessionEvents` this crate cannot construct directly is reached the
        // only honest way: through a real session. That is `tests/session.rs`'s
        // job; here the empty log stands for "nothing was ever reported".
        let log = collect_events(open_and_drop().await);
        assert!(log.wait_for_close().await);
    }

    /// Open a mock session and immediately close it, yielding its event stream.
    async fn open_and_drop() -> SessionEvents {
        let session = crate::MockRealtime::new(crate::Script::conversation())
            .open()
            .await
            .expect("the mock opens");
        let (handle, events, _mock) = session.into_parts();
        handle.close().await.expect("closes");
        events
    }
}
