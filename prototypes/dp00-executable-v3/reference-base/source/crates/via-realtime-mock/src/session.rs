//! The front door: a script in, a live session out.
//!
//! [`RealtimeSession::open`] takes any [`via_realtime::Transport`], which is the seam
//! `via-realtime` put there for exactly this
//! (`docs/deviations/phase-5-via-realtime.md`, *"`Transport` is a seam"*). So a
//! mock session is a real session in every respect that matters — the same two
//! owning tasks, the same correlation, the same watchdogs, the same busy-retry
//! ladder — with a task where the socket would be.
//!
//! ```
//! use via_realtime_mock::{MockRealtime, Script, script::turns};
//! use via_realtime::ResponseContext;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # tokio::runtime::Runtime::new()?.block_on(async {
//! let mock = MockRealtime::new(Script::conversation().turn(turns::say("Right away.")));
//! let (session, events, handle) = mock.open().await?.into_parts();
//! let log = via_realtime_mock::collect_events(events);
//!
//! let outcome = session
//!     .send_user_text("start the build", ResponseContext::new(), None)
//!     .await?;
//! assert!(outcome.is_some_and(|outcome| outcome.is_completed()));
//!
//! // A settled outcome is not a drained log — see [`crate::record`].
//! assert!(log.wait_for_turns(1).await);
//! assert_eq!(log.spoken_text(), "Right away.");
//!
//! // Everything the session wrote is readable, in order.
//! let transcript = handle.transcript().await?;
//! assert_eq!(transcript.count_of("response.create"), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # })
//! # }
//! ```
//!
//! # `RealtimeSession::connect` is the wrong door
//!
//! It dials [`via_realtime::RealtimeProvider::url`] over
//! TCP, and the mock's endpoint names no socket. That is the correct answer to
//! "connect to the mock over the network", not an omission: the point of this
//! crate is that there is no network.

use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;
use tokio::sync::{mpsc, oneshot};
use via_i18n::Locale;
use via_protocol::SessionMode;
use via_realtime::{
    AgentContext, RealtimeError, RealtimeProvider, RealtimeSession, SessionEvents, SessionOptions,
};

use crate::error::MockError;
use crate::record::{EventLog, POLL_BUDGET, collect_events};
use crate::script::{Emission, Script};
use crate::server::{self, MockCommand, Transcript};

/// A script, and the options the session it opens will take.
#[derive(Debug, Clone)]
pub struct MockRealtime {
    script: Script,
    options: SessionOptions,
}

impl MockRealtime {
    /// A mock built from a script.
    ///
    /// The session mode comes from the script — [`Script::session_mode`] — so a
    /// `dictation` script opens a `dictation` session without the caller
    /// restating it.
    #[must_use]
    pub fn new(script: Script) -> Self {
        let options = SessionOptions {
            mode: script.session_mode(),
            ..SessionOptions::default()
        };
        Self { script, options }
    }

    /// A mock built from a script written as JSON text.
    ///
    /// # Errors
    ///
    /// [`MockError::ScriptInvalid`].
    pub fn from_json_str(json: &str) -> Result<Self, MockError> {
        Ok(Self::new(Script::from_json_str(json)?))
    }

    /// A mock built from a JSON fixture file, so a test in another crate keeps
    /// its script beside itself.
    ///
    /// # Errors
    ///
    /// [`MockError::FixtureUnreadable`] or [`MockError::FixtureInvalid`], both
    /// naming the path.
    pub fn from_json_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, MockError> {
        Ok(Self::new(Script::from_json_file(path)?))
    }

    /// The script this mock will replay.
    #[must_use]
    pub fn script(&self) -> &Script {
        &self.script
    }

    /// The options the session will take.
    #[must_use]
    pub fn options(&self) -> &SessionOptions {
        &self.options
    }

    /// Replace the session options wholesale.
    ///
    /// Overrides the mode the script implied, which is why the narrow setters
    /// below exist: they leave it alone.
    #[must_use]
    pub fn with_options(mut self, options: SessionOptions) -> Self {
        self.options = options;
        self
    }

    /// Open in a different session mode than the script implies.
    #[must_use]
    pub fn with_mode(mut self, mode: SessionMode) -> Self {
        self.options.mode = mode;
        self
    }

    /// The locale every sentence the session composes is rendered in.
    #[must_use]
    pub fn with_locale(mut self, locale: Locale) -> Self {
        self.options.locale = locale;
        self
    }

    /// The conversation context the provider builds `session.update` from.
    #[must_use]
    pub fn with_agent_context(mut self, agent_context: AgentContext) -> Self {
        self.options.agent_context = agent_context;
        self
    }

    /// Shorten the connect budget.
    #[must_use]
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.options.connect_timeout = Some(timeout);
        self
    }

    /// Shorten the response-start watchdog.
    #[must_use]
    pub fn with_response_start_timeout(mut self, timeout: Duration) -> Self {
        self.options.response_start_timeout = Some(timeout);
        self
    }

    /// Shorten the output-inactivity watchdog.
    #[must_use]
    pub fn with_response_inactivity_timeout(mut self, timeout: Duration) -> Self {
        self.options.response_inactivity_timeout = Some(timeout);
        self
    }

    /// The provider this script declares, validated.
    ///
    /// For a test that wants the provider without a session: a registry entry, a
    /// `/api/health` descriptor, a configuration signature.
    ///
    /// # Errors
    ///
    /// Whatever [`via_realtime::validate_realtime_provider`] refuses.
    pub fn provider(&self) -> Result<Arc<dyn RealtimeProvider>, RealtimeError> {
        self.script.provider.clone().build()
    }

    /// Open the session.
    ///
    /// Resolves when the session is *usable*, which for the default script means
    /// after `session.created` → `session.update` → `session.updated`.
    ///
    /// # Errors
    ///
    /// Everything a real connection can refuse with, because the session is the
    /// real one: [`RealtimeError::UnsupportedModel`] from the preflight gate,
    /// [`RealtimeError::NotConfigured`], [`RealtimeError::ProviderRefused`] when
    /// the script answers the handshake with an `error`, and
    /// [`RealtimeError::ConnectTimeout`] when it answers with nothing.
    pub async fn open(self) -> Result<MockSession, RealtimeError> {
        let (handle, opened) = self.try_open().await;
        let (session, events) = opened?;
        Ok(MockSession {
            session,
            events,
            handle,
        })
    }

    /// Open the session, keeping the handle whichever way it goes.
    ///
    /// The door for a test that has to read the transcript of a connection that
    /// was *refused* — which frames were written before the provider said no.
    pub async fn try_open(
        self,
    ) -> (
        MockHandle,
        Result<(RealtimeSession, SessionEvents), RealtimeError>,
    ) {
        let provider = match self.script.provider.clone().build() {
            Ok(provider) => provider,
            Err(error) => return (MockHandle::stopped(), Err(error)),
        };
        let correlates = self.script.provider.dialect.correlates();
        let (transport, commands) = server::spawn(self.script, correlates);
        let handle = MockHandle { commands };
        let opened = RealtimeSession::open(provider, self.options, transport).await;
        (handle, opened)
    }
}

/// An open mock session.
#[derive(Debug)]
pub struct MockSession {
    /// The session under test. A real [`RealtimeSession`].
    pub session: RealtimeSession,
    /// Everything it reports, in order.
    pub events: SessionEvents,
    /// The script server behind it.
    pub handle: MockHandle,
}

impl MockSession {
    /// The three pieces, unpacked.
    #[must_use]
    pub fn into_parts(self) -> (RealtimeSession, SessionEvents, MockHandle) {
        (self.session, self.events, self.handle)
    }

    /// The session and its handle, with the events already draining into a log.
    ///
    /// The shape most tests want: it satisfies `via-realtime`'s rule that the
    /// task draining the stream must not be the task awaiting session methods,
    /// without the caller having to think about it.
    #[must_use]
    pub fn with_event_log(self) -> (RealtimeSession, EventLog, MockHandle) {
        (self.session, collect_events(self.events), self.handle)
    }
}

/// The door to the script server.
///
/// Cheap to clone — every clone talks to the same server.
#[derive(Debug, Clone)]
pub struct MockHandle {
    commands: mpsc::Sender<MockCommand>,
}

impl MockHandle {
    /// A handle to nothing, for a mock that never started a server.
    fn stopped() -> Self {
        let (commands, _receiver) = mpsc::channel(1);
        Self { commands }
    }

    /// A copy of everything that has crossed the transport, in order.
    ///
    /// Taken inside the server task, so it can never be torn across a frame.
    ///
    /// # Errors
    ///
    /// [`MockError::ServerStopped`] once the session has closed *and* every
    /// handle has been dropped.
    pub async fn transcript(&self) -> Result<Transcript, MockError> {
        let (reply, answer) = oneshot::channel();
        self.commands
            .send(MockCommand::Snapshot(reply))
            .await
            .map_err(|_| MockError::ServerStopped)?;
        answer.await.map_err(|_| MockError::ServerStopped)
    }

    /// Send one event to the session, out of band.
    ///
    /// Stamped like a scripted one — see the crate docs' rule 4 — against
    /// whatever the server currently believes is in flight. This is how a
    /// server-VAD turn nobody asked for is injected, or a mid-session `error`.
    ///
    /// # Errors
    ///
    /// [`MockError::ServerStopped`].
    pub async fn emit(&self, event: Value) -> Result<(), MockError> {
        self.emit_all([Emission::now(event)]).await
    }

    /// Send several events, with their delays.
    ///
    /// Resolves once they are *queued*, not once they are sent: a delayed
    /// emission's whole point is that the caller carries on meanwhile.
    ///
    /// # Errors
    ///
    /// [`MockError::ServerStopped`].
    pub async fn emit_all<I>(&self, emissions: I) -> Result<(), MockError>
    where
        I: IntoIterator<Item = Emission>,
    {
        let (reply, queued) = oneshot::channel();
        self.commands
            .send(MockCommand::Emit {
                emissions: emissions.into_iter().collect(),
                reply,
            })
            .await
            .map_err(|_| MockError::ServerStopped)?;
        queued.await.map_err(|_| MockError::ServerStopped)
    }

    /// Yield until the session has written a frame of this type, then answer
    /// with the first one.
    ///
    /// Needed only for the fire-and-forget calls — `append_audio`,
    /// `update_agent_context`, `cancel` — because every other session method
    /// already awaits its own round trip, so its frames are in the transcript by
    /// the time it returns.
    ///
    /// Yields rather than sleeps: waiting must not advance a paused clock into a
    /// watchdog the test never meant to fire.
    ///
    /// # Errors
    ///
    /// [`MockError::FrameNotWritten`] when it never arrives, or
    /// [`MockError::ServerStopped`].
    pub async fn wait_for_frame(&self, kind: &str) -> Result<Value, MockError> {
        for _ in 0..POLL_BUDGET {
            if let Some(frame) = self.transcript().await?.first_of(kind).cloned() {
                return Ok(frame);
            }
            tokio::task::yield_now().await;
        }
        Err(MockError::FrameNotWritten {
            kind: kind.to_owned(),
        })
    }

    /// Yield until the session has written `count` frames of this type.
    ///
    /// # Errors
    ///
    /// [`MockError::FrameNotWritten`] when they never arrive, or
    /// [`MockError::ServerStopped`].
    pub async fn wait_for_frames(&self, kind: &str, count: usize) -> Result<Vec<Value>, MockError> {
        for _ in 0..POLL_BUDGET {
            let transcript = self.transcript().await?;
            let frames = transcript.frames_of(kind);
            if frames.len() >= count {
                return Ok(frames.into_iter().cloned().collect());
            }
            tokio::task::yield_now().await;
        }
        Err(MockError::FrameNotWritten {
            kind: kind.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::provider::{MOCK_MODEL, MOCK_PROVIDER_KEY, MockProviderSpec};
    use crate::script::turns;

    #[test]
    fn the_script_kind_chooses_the_session_mode() {
        assert_eq!(
            MockRealtime::new(Script::conversation()).options().mode,
            SessionMode::Agent
        );
        assert_eq!(
            MockRealtime::new(Script::dictation()).options().mode,
            SessionMode::Dictation
        );
        assert_eq!(
            MockRealtime::new(Script::conversation())
                .with_mode(SessionMode::Interface)
                .options()
                .mode,
            SessionMode::Interface
        );
    }

    #[test]
    fn the_narrow_setters_leave_the_mode_alone() {
        let mock = MockRealtime::new(Script::dictation())
            .with_locale(Locale::Ko)
            .with_connect_timeout(Duration::from_millis(50))
            .with_response_start_timeout(Duration::from_millis(60))
            .with_response_inactivity_timeout(Duration::from_millis(70));
        assert_eq!(mock.options().mode, SessionMode::Dictation);
        assert_eq!(mock.options().locale, Locale::Ko);
        assert_eq!(
            mock.options().connect_timeout,
            Some(Duration::from_millis(50))
        );
        assert_eq!(
            mock.options().response_start_timeout,
            Some(Duration::from_millis(60))
        );
        assert_eq!(
            mock.options().response_inactivity_timeout,
            Some(Duration::from_millis(70))
        );
    }

    #[test]
    fn a_mock_built_from_json_carries_the_scripts_provider() {
        let mock = MockRealtime::from_json_str(
            r#"{ "kind": "dictation", "steps": [{ "on": "audioCommit", "emit": [] }] }"#,
        )
        .expect("parse");
        assert_eq!(mock.options().mode, SessionMode::Dictation);
        assert_eq!(mock.script().steps().len(), 1);
        assert_eq!(mock.script().provider.model, None);
    }

    #[test]
    fn the_provider_is_reachable_without_a_session() {
        let provider = MockRealtime::new(Script::conversation())
            .provider()
            .expect("valid");
        assert_eq!(provider.key(), MOCK_PROVIDER_KEY);
        assert_eq!(provider.model(), Some(MOCK_MODEL));
    }

    #[tokio::test]
    async fn a_handle_to_a_provider_that_never_started_answers_stopped() {
        let (handle, opened) = MockRealtime::new(
            Script::conversation().map_provider(|spec| spec.with_key("Not A Key")),
        )
        .try_open()
        .await;
        assert!(opened.is_err());
        assert_eq!(
            handle.transcript().await.map(|_| ()),
            Err(MockError::ServerStopped)
        );
        assert_eq!(
            handle.emit(serde_json::json!({ "type": "error" })).await,
            Err(MockError::ServerStopped)
        );
    }

    #[tokio::test]
    async fn a_default_script_opens_and_answers_a_turn() {
        let (session, log, handle) = MockRealtime::new(Script::conversation())
            .open()
            .await
            .expect("opens")
            .with_event_log();

        let outcome = session
            .send_user_text("hello", via_realtime::ResponseContext::new(), None)
            .await
            .expect("no transport failure")
            .expect("an outcome");
        assert!(outcome.is_completed(), "{outcome:?}");

        let transcript = handle.transcript().await.expect("transcript");
        assert_eq!(transcript.count_of("session.update"), 1);
        assert_eq!(transcript.count_of("conversation.item.create"), 1);
        assert_eq!(transcript.count_of("response.create"), 1);
        assert!(log.wait_for_kind("response.done").await.is_some());
    }

    #[tokio::test]
    async fn a_scripted_turn_replaces_the_default_answer() {
        let (session, log, _handle) =
            MockRealtime::new(Script::conversation().turn(turns::say("Right away.")))
                .open()
                .await
                .expect("opens")
                .with_event_log();
        session
            .send_user_text("go", via_realtime::ResponseContext::new(), None)
            .await
            .expect("no transport failure");
        assert!(log.wait_for_turns(1).await);
        assert_eq!(log.spoken_text(), "Right away.");
    }

    /// Two user turns against the same script, answering with everything the
    /// mock sent.
    async fn replay(spec: MockProviderSpec) -> Vec<serde_json::Value> {
        let (session, _log, handle) = MockRealtime::new(
            Script::conversation()
                .with_provider(spec)
                .turn(turns::say("one"))
                .turn(turns::call_tool("call_1", "tool", &serde_json::json!({}))),
        )
        .open()
        .await
        .expect("opens")
        .with_event_log();
        for _ in 0..2 {
            let _ = session
                .send_user_text("go", via_realtime::ResponseContext::new(), None)
                .await;
        }
        handle
            .transcript()
            .await
            .expect("transcript")
            .inbound()
            .to_vec()
    }

    #[tokio::test]
    async fn the_same_script_produces_the_same_bytes_twice() {
        // Every id in the stream is the mock's own — `resp_N`, `item_mock_N`,
        // `event_mock_N` — once the provider stops echoing the client's item
        // ids, so the two runs are byte-identical.
        let spec =
            MockProviderSpec::default().with_capabilities(via_realtime::ProviderCapabilities {
                conversation_item_id_echo: false,
                ..via_realtime::ProviderCapabilities::DEFAULT
            });
        assert_eq!(replay(spec.clone()).await, replay(spec).await);
    }

    #[tokio::test]
    async fn the_only_thing_that_differs_between_runs_is_an_id_the_client_minted() {
        // With the baseline `conversation_item_id_echo: true` the mock echoes
        // the item id back, and that id is a v4 uuid the *session* minted
        // (`RealtimeProtocol::conversation_item_id`). So the boundary of this
        // crate's determinism is exactly there, and nowhere else.
        let first = replay(MockProviderSpec::default()).await;
        let second = replay(MockProviderSpec::default()).await;
        assert_ne!(first, second, "the echoed uuid differs");

        let redact = |events: Vec<serde_json::Value>| -> Vec<serde_json::Value> {
            events
                .into_iter()
                .map(|mut event| {
                    if let Some(id) = event.pointer_mut("/item/id") {
                        *id = serde_json::json!("<echoed>");
                    }
                    event
                })
                .collect()
        };
        assert_eq!(redact(first), redact(second));
    }

    #[tokio::test]
    async fn an_out_of_band_emission_reaches_the_session() {
        let (_session, log, handle) = MockRealtime::new(Script::conversation())
            .open()
            .await
            .expect("opens")
            .with_event_log();
        handle
            .emit(crate::script::events::speech_started())
            .await
            .expect("emitted");
        assert!(
            log.wait_for_kind("input_audio_buffer.speech_started")
                .await
                .is_some()
        );
    }

    #[tokio::test]
    async fn waiting_for_a_frame_that_never_comes_answers_rather_than_hangs() {
        let (_session, _log, handle) = MockRealtime::new(Script::conversation())
            .open()
            .await
            .expect("opens")
            .with_event_log();
        assert_eq!(
            handle.wait_for_frame("never.written").await.map(|_| ()),
            Err(MockError::FrameNotWritten {
                kind: "never.written".into()
            })
        );
    }

    #[tokio::test]
    async fn the_transcript_survives_the_session_closing() {
        let (session, log, handle) = MockRealtime::new(Script::conversation())
            .open()
            .await
            .expect("opens")
            .with_event_log();
        session.close().await.expect("closes");
        assert!(log.wait_for_close().await);
        // The handle is still alive, so the server is still answering.
        let transcript = handle.transcript().await.expect("transcript");
        assert_eq!(transcript.count_of("session.update"), 1);
    }

    #[test]
    fn a_replaced_options_block_is_taken_whole() {
        let options = SessionOptions {
            mode: SessionMode::Direct,
            locale: Locale::Zh,
            ..SessionOptions::default()
        };
        let mock = MockRealtime::new(Script::dictation()).with_options(options.clone());
        assert_eq!(mock.options().mode, options.mode);
        assert_eq!(mock.options().locale, options.locale);
    }

    #[test]
    fn a_spec_the_registry_would_refuse_never_reaches_a_session() {
        let mock = MockRealtime::new(
            Script::conversation()
                .with_provider(MockProviderSpec::default().with_input_sample_rate(0)),
        );
        assert_eq!(
            mock.provider().map(|_| ()),
            Err(RealtimeError::ProviderMissingInputSampleRate {
                key: MOCK_PROVIDER_KEY.into()
            })
        );
    }
}
