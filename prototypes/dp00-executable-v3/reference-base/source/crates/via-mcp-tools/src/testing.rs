//! A deterministic [`SessionToolContext`] double.
//!
//! Behind the default-on `testing` feature, matching `via-downstream`'s
//! [`ScriptedHarness`](via_downstream::testing). Upstream's tests hand
//! `AcpSessionToolServer.register` an object literal of five async closures
//! (`server/test/acp-session-tools.test.mjs:16-28`); this is that literal,
//! typed, with the calls recorded so a test can assert *what the model asked
//! for* and not only what it got back.
//!
//! It reads no clock, spawns nothing and never blocks, so a
//! `#[tokio::test(start_paused = true)]` around it stays deterministic.

use std::sync::{Mutex, PoisonError};

use async_trait::async_trait;
use via_downstream::{CancelOutcome, HarnessError};

use crate::context::{
    DelegationLookupInput, DelegationOutcome, DelegationRecord, DelegationStarted,
    SessionSendInput, SessionStartInput, SessionStatusResult, SessionSummary, SessionToolContext,
    SessionsListInput, SessionsListResult, matches_query,
};

/// One call the double received, with the arguments as parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordedCall {
    /// `via_sessions_list`.
    ListSessions(SessionsListInput),
    /// `via_session_start`.
    StartSession(SessionStartInput),
    /// `via_session_send`.
    SendSession(SessionSendInput),
    /// `via_session_status`.
    SessionStatus(DelegationLookupInput),
    /// `via_session_cancel`.
    CancelSession(DelegationLookupInput),
}

impl RecordedCall {
    /// The tool the call came in on, as a name.
    #[must_use]
    pub const fn tool(&self) -> crate::SessionTool {
        match self {
            Self::ListSessions(_) => crate::SessionTool::SessionsList,
            Self::StartSession(_) => crate::SessionTool::SessionStart,
            Self::SendSession(_) => crate::SessionTool::SessionSend,
            Self::SessionStatus(_) => crate::SessionTool::SessionStatus,
            Self::CancelSession(_) => crate::SessionTool::SessionCancel,
        }
    }
}

/// How a scripted refusal fails.
///
/// Described rather than held, because [`HarnessError`] is not `Clone`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// A sentence the harness already localized —
    /// [`HarnessError::Agent`], the arm a driver puts an unforeseen failure
    /// in.
    Agent(String),
    /// [`HarnessError::Cancelled`]: localized by the seam, so which sentence
    /// the model sees depends on the *server's* locale.
    Cancelled,
    /// [`HarnessError::DelegationNotFound`], likewise localized.
    DelegationNotFound(String),
}

impl Refusal {
    fn build(&self, label: &str) -> HarnessError {
        match self {
            Self::Agent(message) => HarnessError::Agent {
                message: message.clone(),
                status: 0,
                body: String::new(),
                protocol: label.to_owned(),
            },
            Self::Cancelled => HarnessError::Cancelled,
            Self::DelegationNotFound(label) => HarnessError::DelegationNotFound {
                label: label.clone(),
            },
        }
    }
}

/// A [`SessionToolContext`] that answers from a fixture and records what it
/// was asked.
#[derive(Debug, Default)]
pub struct RecordingContext {
    label: String,
    sessions: Vec<SessionSummary>,
    delegation: Option<(DelegationRecord, DelegationOutcome)>,
    cancel: Option<CancelOutcome>,
    refusal: Option<Refusal>,
    calls: Mutex<Vec<RecordedCall>>,
}

impl RecordingContext {
    /// A double that lists nothing, knows no delegation and fails nothing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Tag every delegation id this double issues, so two contexts registered
    /// against one server are told apart.
    #[must_use]
    pub fn labelled(mut self, label: &str) -> Self {
        self.label = label.to_owned();
        self
    }

    /// The Sessions `via_sessions_list` answers with, before filtering.
    #[must_use]
    pub fn with_sessions(mut self, sessions: Vec<SessionSummary>) -> Self {
        self.sessions = sessions;
        self
    }

    /// The one delegation `via_session_status` knows about.
    #[must_use]
    pub fn with_delegation(mut self, record: DelegationRecord, outcome: DelegationOutcome) -> Self {
        self.delegation = Some((record, outcome));
        self
    }

    /// What `via_session_cancel` answers with. Defaults to
    /// [`CancelOutcome::NotFound`].
    #[must_use]
    pub fn with_cancel(mut self, outcome: CancelOutcome) -> Self {
        self.cancel = Some(outcome);
        self
    }

    /// Make every tool fail with this already-localized sentence.
    #[must_use]
    pub fn failing(mut self, message: &str) -> Self {
        self.refusal = Some(Refusal::Agent(message.to_owned()));
        self
    }

    /// Make every tool fail with a [`HarnessError`] the seam localizes itself.
    ///
    /// Separate from [`Self::failing`] because a `HarnessError` is not
    /// `Clone` — the refusal is described here and constructed per call, which
    /// is what lets a test assert that the *server's* locale, not the
    /// double's, decides the sentence the model is shown.
    #[must_use]
    pub fn refusing(mut self, refusal: Refusal) -> Self {
        self.refusal = Some(refusal);
        self
    }

    /// Every call received, oldest first.
    #[must_use]
    pub fn calls(&self) -> Vec<RecordedCall> {
        self.locked().clone()
    }

    /// The most recent call, if there was one.
    #[must_use]
    pub fn last_call(&self) -> Option<RecordedCall> {
        self.locked().last().cloned()
    }

    /// How many calls have been received.
    #[must_use]
    pub fn call_count(&self) -> usize {
        self.locked().len()
    }

    /// A poisoned recording buffer is still a usable one: the data is a `Vec`
    /// of plain values and no invariant spans a panic.
    fn locked(&self) -> std::sync::MutexGuard<'_, Vec<RecordedCall>> {
        self.calls.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn record(&self, call: RecordedCall) {
        self.locked().push(call);
    }

    fn refuse<T>(&self) -> Option<Result<T, HarnessError>> {
        self.refusal
            .as_ref()
            .map(|refusal| Err(refusal.build(&self.label)))
    }

    fn started(&self, session_id: &str, title: &str) -> DelegationStarted {
        let prefix = if self.label.is_empty() {
            "test".to_owned()
        } else {
            self.label.clone()
        };
        DelegationRecord::new(
            &format!("{prefix}_run_1"),
            session_id,
            title,
            "/srv/project",
        )
        .started()
    }
}

#[async_trait]
impl SessionToolContext for RecordingContext {
    async fn list_sessions(
        &self,
        input: SessionsListInput,
    ) -> Result<SessionsListResult, HarnessError> {
        self.record(RecordedCall::ListSessions(input.clone()));
        if let Some(refusal) = self.refuse() {
            return refusal;
        }
        let needle = input.needle();
        Ok(SessionsListResult::new(
            self.sessions
                .iter()
                .filter(|summary| matches_query(summary, needle.as_deref()))
                .cloned()
                .collect(),
        ))
    }

    async fn start_session(
        &self,
        input: SessionStartInput,
    ) -> Result<DelegationStarted, HarnessError> {
        self.record(RecordedCall::StartSession(input.clone()));
        if let Some(refusal) = self.refuse() {
            return refusal;
        }
        Ok(self.started(
            "session_started",
            input.title.as_deref().unwrap_or(&input.prompt),
        ))
    }

    async fn send_session(
        &self,
        input: SessionSendInput,
    ) -> Result<DelegationStarted, HarnessError> {
        self.record(RecordedCall::SendSession(input.clone()));
        if let Some(refusal) = self.refuse() {
            return refusal;
        }
        Ok(self.started(&input.session_id, &input.prompt))
    }

    async fn session_status(
        &self,
        input: DelegationLookupInput,
    ) -> Result<SessionStatusResult, HarnessError> {
        self.record(RecordedCall::SessionStatus(input.clone()));
        if let Some(refusal) = self.refuse() {
            return refusal;
        }
        Ok(self
            .delegation
            .as_ref()
            .filter(|(record, _)| input.matches(record))
            .map_or(SessionStatusResult::NotFound, |(record, outcome)| {
                SessionStatusResult::known(record.clone(), outcome.clone())
            }))
    }

    async fn cancel_session(
        &self,
        input: DelegationLookupInput,
    ) -> Result<CancelOutcome, HarnessError> {
        self.record(RecordedCall::CancelSession(input));
        if let Some(refusal) = self.refuse() {
            return refusal;
        }
        Ok(self.cancel.clone().unwrap_or(CancelOutcome::NotFound))
    }
}
