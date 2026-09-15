//! A deterministic harness, so nobody has to invent one twice.
//!
//! `docs/architecture.md` §15 makes phase 2 end when *"a scripted fake ACP
//! agent completes a prompt turn through the trait"*, and phases 3 and 5 need
//! the same thing for `via-work` and `via-app`. Three private copies of a fake
//! harness would drift, and the one that drifted would be the one asserting
//! that the seam behaves — so the double lives beside the seam it doubles.
//!
//! Everything here is deterministic: no clock, no randomness, no task
//! scheduling. A [`ScriptedHarness`] hands out turns from a queue in order, and
//! every event it emits goes through the real projection in [`crate::event`],
//! so a test that passes against this double is a test against the same
//! bounding and the same boundary a real backend meets.
//!
//! ```
//! # use via_downstream::testing::{ScriptedHarness, ScriptedTurn};
//! # use via_downstream::{HarnessRegistry, PromptRequest, SessionKey};
//! # use std::sync::Arc;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let harness = ScriptedHarness::builder("codex")
//!     .turn(ScriptedTurn::completed("done"))
//!     .build()?;
//!
//! let mut registry = HarnessRegistry::new();
//! registry.register(Arc::new(harness))?;
//!
//! let key = SessionKey::coordinator("codex", "ana");
//! let session = registry.open(&key).await?;
//! let outcome = session.prompt(PromptRequest::text("ana", "hello")).await?;
//! assert_eq!(outcome.content(), "done");
//! # Ok(())
//! # }
//! ```
//!
//! This module is behind the `testing` feature, which is **on by default** so
//! that a sibling crate can reach it without restating the feature and so this
//! crate's own integration tests run under a bare `cargo test -p
//! via-downstream`. A build that wants none of it takes
//! `default-features = false`.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

use async_trait::async_trait;
use futures::channel::mpsc::{UnboundedSender, unbounded};
use futures::stream::{BoxStream, StreamExt};
use via_catalog::{BackendDefinition, backend_definition};
use via_i18n::{Locale, format, keys};

use crate::agent::{DownstreamAgent, HarnessSession};
use crate::cancel::{CancelOutcome, CancelRoute, CancelScope, CancelTarget};
use crate::capability::BackendCapabilities;
use crate::descriptor::HarnessDescriptor;
use crate::error::HarnessError;
use crate::event::{ActivityTracker, RawSessionUpdate, SessionEvent};
use crate::health::HarnessHealth;
use crate::prompt::{PromptOutcome, PromptRequest, StopReason};
use crate::session_key::SessionKey;

/// Lock a mutex, treating poisoning as "the data is still there".
///
/// A poisoned lock here means a test panicked while holding it; the test has
/// already failed and there is nothing to protect. Recovering keeps the double
/// free of `unwrap` (`scripts/risky_unwrap.py`) without pretending the panic
/// did not happen.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// A capability declaration that is consistent for this backend.
///
/// Derived, not invented: `permissions` and `backendUi` are read straight off
/// the catalog entry, because those two are exactly the flags
/// [`HarnessDescriptor::declare`] cross-checks against it. The rest describe
/// the shape ten of the twelve upstream drivers share — MCP session tools over
/// an external MCP connection, with the backend keeping its own history.
#[must_use]
pub fn consistent_capabilities(definition: &BackendDefinition) -> BackendCapabilities {
    BackendCapabilities {
        delegation: true,
        permissions: !definition.always_full_permission,
        backend_ui: definition.default_base_url.is_some(),
        native_session_history: true,
        external_mcp: true,
        native_delegation: false,
        session_mcp: true,
    }
}

/// How one scripted turn ends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptedResult {
    /// The turn produced `content` and stopped for `stop_reason`.
    Stopped {
        /// The agent's text.
        content: String,
        /// Why it stopped. [`StopReason::Cancelled`] makes the turn fail with
        /// [`HarnessError::Cancelled`], because that is what
        /// [`PromptOutcome::new`] does and the double must not be gentler than
        /// the seam.
        stop_reason: StopReason,
    },
    /// The turn failed the way a transport failure arrives:
    /// [`HarnessError::Agent`].
    Failed {
        /// The already-localized sentence.
        message: String,
        /// An HTTP status, or `0`.
        status: u16,
    },
}

/// One turn of a script: what the harness emits, then how the turn ends.
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptedTurn {
    updates: Vec<RawSessionUpdate>,
    result: ScriptedResult,
}

impl ScriptedTurn {
    /// A turn that produces `content` and ends normally.
    #[must_use]
    pub fn completed(content: &str) -> Self {
        Self::stopping(content, StopReason::EndTurn)
    }

    /// A turn that produces `content` and stops for a specific reason.
    #[must_use]
    pub fn stopping(content: &str, stop_reason: StopReason) -> Self {
        Self {
            updates: Vec::new(),
            result: ScriptedResult::Stopped {
                content: content.to_owned(),
                stop_reason,
            },
        }
    }

    /// A turn that is cancelled — the case a port must not report as success.
    #[must_use]
    pub fn cancelled() -> Self {
        Self::stopping("", StopReason::Cancelled)
    }

    /// A turn that fails with a transport error.
    #[must_use]
    pub fn failing(message: &str, status: u16) -> Self {
        Self {
            updates: Vec::new(),
            result: ScriptedResult::Failed {
                message: message.to_owned(),
                status,
            },
        }
    }

    /// Emit one `session/update` notification during this turn.
    ///
    /// It goes through [`ActivityTracker::project`], so an update that projects
    /// to nothing emits nothing here either.
    #[must_use]
    pub fn emitting(mut self, update: RawSessionUpdate) -> Self {
        self.updates.push(update);
        self
    }
}

/// Builds a [`ScriptedHarness`].
#[derive(Debug, Clone)]
pub struct ScriptedHarnessBuilder {
    id: String,
    capabilities: Option<BackendCapabilities>,
    turns: Vec<ScriptedTurn>,
    health: HarnessHealth,
    locale: Locale,
    session_id: Option<String>,
    cancel: Option<CancelOutcome>,
}

impl ScriptedHarnessBuilder {
    /// Override the capability declaration.
    ///
    /// The default is [`consistent_capabilities`] for the chosen backend. An
    /// override is still validated, so this is how a test reaches
    /// [`HarnessError::IncompleteCapabilities`] through the real path.
    #[must_use]
    pub const fn capabilities(mut self, capabilities: BackendCapabilities) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    /// Append a turn. Turns are consumed in order, across every open session.
    #[must_use]
    pub fn turn(mut self, turn: ScriptedTurn) -> Self {
        self.turns.push(turn);
        self
    }

    /// Set what [`DownstreamAgent::health`] answers. Defaults to
    /// [`HarnessHealth::ready`].
    #[must_use]
    pub fn health(mut self, health: HarnessHealth) -> Self {
        self.health = health;
        self
    }

    /// Set the locale the double renders its own messages in.
    #[must_use]
    pub const fn locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// Force the session id every opened session reports.
    ///
    /// Pass `""` to exercise
    /// [`HarnessError::InvalidSession`] through
    /// [`HarnessRegistry::open`](crate::HarnessRegistry::open).
    #[must_use]
    pub fn session_id(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_owned());
        self
    }

    /// Set what [`HarnessSession::cancel`] answers.
    ///
    /// Defaults to [`CancelOutcome::Requested`] over
    /// [`CancelRoute::Adapter`] — the honest answer for a fire-and-forget
    /// transport, and the one a caller most needs to be able to handle.
    #[must_use]
    pub fn cancel_outcome(mut self, outcome: CancelOutcome) -> Self {
        self.cancel = Some(outcome);
        self
    }

    /// Validate the descriptor and build.
    ///
    /// # Errors
    ///
    /// Whatever [`HarnessDescriptor::declare`] refuses — an unknown backend id,
    /// or a capability declaration that contradicts itself or the catalog.
    pub fn build(self) -> Result<ScriptedHarness, HarnessError> {
        let definition =
            backend_definition(&self.id).ok_or_else(|| HarnessError::DriverNotRegistered {
                id: self.id.clone(),
            })?;
        let capabilities = self
            .capabilities
            .unwrap_or_else(|| consistent_capabilities(definition));
        let descriptor = HarnessDescriptor::declare(definition.id, definition.label, capabilities)?;
        Ok(ScriptedHarness {
            descriptor,
            locale: self.locale,
            health: self.health,
            forced_session_id: self.session_id,
            cancel: self.cancel,
            script: Arc::new(Mutex::new(self.turns.into_iter().collect())),
            sessions: Mutex::new(Vec::new()),
        })
    }
}

/// A harness that answers from a script.
pub struct ScriptedHarness {
    descriptor: HarnessDescriptor,
    locale: Locale,
    health: HarnessHealth,
    forced_session_id: Option<String>,
    cancel: Option<CancelOutcome>,
    script: Arc<Mutex<VecDeque<ScriptedTurn>>>,
    sessions: Mutex<Vec<ScriptedSession>>,
}

impl ScriptedHarness {
    /// Start building a harness for a catalogued backend id.
    #[must_use]
    pub fn builder(id: &str) -> ScriptedHarnessBuilder {
        ScriptedHarnessBuilder {
            id: id.to_owned(),
            capabilities: None,
            turns: Vec::new(),
            health: HarnessHealth::ready(),
            locale: Locale::En,
            session_id: None,
            cancel: None,
        }
    }

    /// Every session opened so far, in the order they were opened.
    #[must_use]
    pub fn sessions(&self) -> Vec<ScriptedSession> {
        lock(&self.sessions).clone()
    }

    /// How many scripted turns are left unconsumed.
    #[must_use]
    pub fn remaining_turns(&self) -> usize {
        lock(&self.script).len()
    }
}

impl std::fmt::Debug for ScriptedHarness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptedHarness")
            .field("id", &self.descriptor.id())
            .field("remaining_turns", &self.remaining_turns())
            .finish()
    }
}

#[async_trait]
impl DownstreamAgent for ScriptedHarness {
    fn descriptor(&self) -> &HarnessDescriptor {
        &self.descriptor
    }

    async fn open(&self, key: &SessionKey) -> Result<Box<dyn HarnessSession>, HarnessError> {
        let mut sessions = lock(&self.sessions);
        let session_id = self.forced_session_id.clone().unwrap_or_else(|| {
            format!(
                "{protocol}-session-{ordinal}",
                protocol = self.descriptor.id(),
                ordinal = sessions.len() + 1,
            )
        });
        let session = ScriptedSession {
            inner: Arc::new(SessionInner {
                session_id,
                key: key.clone(),
                capabilities: self.descriptor.capabilities(),
                label: self.descriptor.label(),
                locale: self.locale,
                cancel: self.cancel.clone(),
                script: Arc::clone(&self.script),
                state: Mutex::new(SessionState::default()),
            }),
        };
        sessions.push(session.clone());
        Ok(Box::new(session))
    }

    async fn health(&self) -> HarnessHealth {
        self.health.clone()
    }
}

/// Everything one scripted session remembers.
#[derive(Debug, Default)]
struct SessionState {
    prompts: Vec<PromptRequest>,
    cancels: Vec<CancelScope>,
    emitted: Vec<SessionEvent>,
    subscribers: Vec<UnboundedSender<SessionEvent>>,
    tracker: ActivityTracker,
    busy: bool,
}

struct SessionInner {
    session_id: String,
    key: SessionKey,
    capabilities: BackendCapabilities,
    label: &'static str,
    locale: Locale,
    cancel: Option<CancelOutcome>,
    script: Arc<Mutex<VecDeque<ScriptedTurn>>>,
    state: Mutex<SessionState>,
}

/// One session on a [`ScriptedHarness`]. Cheap to clone; every clone is the
/// same session.
#[derive(Clone)]
pub struct ScriptedSession {
    inner: Arc<SessionInner>,
}

impl ScriptedSession {
    /// Every prompt this session was given, in order.
    #[must_use]
    pub fn prompts(&self) -> Vec<PromptRequest> {
        lock(&self.inner.state).prompts.clone()
    }

    /// Every cancel this session was asked for, in order.
    #[must_use]
    pub fn cancels(&self) -> Vec<CancelScope> {
        lock(&self.inner.state).cancels.clone()
    }

    /// Every event this session has emitted, in order.
    ///
    /// The same events [`HarnessSession::events`] yields, available without a
    /// stream for tests that only want to assert on them.
    #[must_use]
    pub fn emitted(&self) -> Vec<SessionEvent> {
        lock(&self.inner.state).emitted.clone()
    }

    /// The key this session was opened for.
    #[must_use]
    pub fn key(&self) -> SessionKey {
        self.inner.key.clone()
    }

    /// Pin the session as having a turn in flight.
    ///
    /// The double runs a turn to completion synchronously, so the
    /// one-turn-at-a-time invariant `docs/architecture.md` §11 states would
    /// otherwise be unobservable. `hold` makes it observable without a second
    /// task: the next [`HarnessSession::prompt`] is refused with
    /// [`HarnessError::SessionBusy`] until [`Self::release`].
    pub fn hold(&self) {
        lock(&self.inner.state).busy = true;
    }

    /// Undo [`Self::hold`].
    pub fn release(&self) {
        lock(&self.inner.state).busy = false;
    }

    /// Whether a turn is in flight, or the session is held.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        lock(&self.inner.state).busy
    }

    /// Push one event out, recording it and fanning it out to live
    /// subscribers. Closed subscribers are dropped.
    fn emit(state: &mut SessionState, event: SessionEvent) {
        state.emitted.push(event.clone());
        state
            .subscribers
            .retain(|subscriber| subscriber.unbounded_send(event.clone()).is_ok());
    }
}

impl std::fmt::Debug for ScriptedSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptedSession")
            .field("session_id", &self.inner.session_id)
            .field("key", &self.inner.key.as_str())
            .finish()
    }
}

#[async_trait]
impl HarnessSession for ScriptedSession {
    fn session_id(&self) -> &str {
        &self.inner.session_id
    }

    async fn prompt(&self, request: PromptRequest) -> Result<PromptOutcome, HarnessError> {
        {
            let mut state = lock(&self.inner.state);
            if state.busy {
                return Err(HarnessError::SessionBusy {
                    label: self.inner.label.to_owned(),
                    session_id: self.inner.session_id.clone(),
                });
            }
            state.busy = true;
            state.prompts.push(request);
        }
        let turn = lock(&self.inner.script).pop_front();
        let mut state = lock(&self.inner.state);
        state.busy = false;
        let Some(turn) = turn else {
            // The script ran out. Upstream's nearest refusal is the one for a
            // session that answered with nothing at all.
            return Err(HarnessError::Agent {
                message: format(
                    self.inner.locale,
                    keys::ACP_SESSION_RETURNED_NOTHING,
                    &[("label", self.inner.label)],
                ),
                status: 0,
                body: String::new(),
                protocol: self.inner.key.protocol().to_owned(),
            });
        };
        for update in &turn.updates {
            if let Some(event) = state.tracker.project(update) {
                Self::emit(&mut state, event);
            }
        }
        match turn.result {
            ScriptedResult::Stopped {
                content,
                stop_reason,
            } => PromptOutcome::new(&content, stop_reason),
            ScriptedResult::Failed { message, status } => Err(HarnessError::Agent {
                message,
                status,
                body: String::new(),
                protocol: self.inner.key.protocol().to_owned(),
            }),
        }
    }

    fn events(&self) -> BoxStream<'static, SessionEvent> {
        let (sender, receiver) = unbounded();
        let mut state = lock(&self.inner.state);
        // Replay first, so a subscriber that arrives after a turn sees the same
        // sequence as one that arrived before it.
        for event in &state.emitted {
            if sender.unbounded_send(event.clone()).is_err() {
                break;
            }
        }
        state.subscribers.push(sender);
        receiver.boxed()
    }

    async fn cancel(&self, scope: CancelScope) -> Result<CancelOutcome, HarnessError> {
        if !scope.is_supported_by(self.inner.capabilities) {
            return Err(HarnessError::CancelUnsupported);
        }
        lock(&self.inner.state).cancels.push(scope.clone());
        Ok(self.inner.cancel.clone().unwrap_or_else(|| {
            let target = scope.delegation_id().map_or_else(
                || CancelTarget::session(&self.inner.session_id),
                CancelTarget::delegation,
            );
            CancelOutcome::requested(CancelRoute::Adapter, target)
        }))
    }
}
