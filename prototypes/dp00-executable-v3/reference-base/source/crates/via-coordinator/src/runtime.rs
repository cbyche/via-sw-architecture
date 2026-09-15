//! The coordinator itself: one fixed session, one turn ladder, one delegation
//! lifecycle.
//!
//! The coordination half of
//! `server/src/agent/{coordinator,acp-backend-adapter}.mjs`. The ACP transport
//! half is `via-acp`'s; what is here is everything above it.
//!
//! # The fixed coordinator session
//!
//! `docs/reference/contracts.json` (`state-name` / *coordinator session key
//! format*) calls
//! [`SessionKey::coordinator`](via_downstream::SessionKey::coordinator) *"THE
//! fixed identity that survives voice sessions, Work IDs, and Gateway
//! restarts"*. One per owner per backend, and nothing about a turn changes it:
//! a new voice conversation continues in the same backend context, which is why
//! *"continue what we were doing"* works at all.
//!
//! # The two guards
//!
//! `docs/architecture.md` §11, invariant 2. Every write to the coordinator
//! session goes through [`crate::executor`] on
//! [`coordinator_session_lane`], and every write to a delegated session goes
//! through it on [`target_lane`]. Above that, `via-work`'s per-owner
//! coordinator lane bounds *admission*. Porting one of the two looks correct
//! until it is under load: the Work lane does not exist for the hidden control
//! turns in [`crate::prompts`], and those write to the same session.
//!
//! # The delegated turn is not a completion
//!
//! When a turn answers `state: "delegated"`, [`Coordinator::run_coordinator`]:
//!
//! 1. lets the turn finish — the model's short post-tool sentence *is* the
//!    start confirmation the user hears;
//! 2. announces it as [`CoordinationObserver::delegated`], which is what moves
//!    the Work to `delegated` and **releases the scheduler lane**;
//! 3. **drops the coordinator lane** and waits outside it, so another voice
//!    request can use the coordinator while the target runs;
//! 4. re-takes the lane for one more turn carrying
//!    [`crate::prompts::delegation_result_prompt`], which is the only turn
//!    allowed to produce the user-visible completion.
//!
//! `docs/architecture.md` §11, invariant 4: the request timeout applies to the
//! coordinator turn and the presentation turn — **not** while waiting on the
//! delegated session, which is why [`via_downstream::PromptRequest::timeout_ms`]
//! is `None` for a delegated prompt.

use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde_json::Value;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use via_acp::PermissionDecision;
use via_acp::session::{coordinator_presentation, normalize_coordinator_content};
use via_downstream::text::clean;
use via_downstream::{
    CancelOutcome, CancelRoute, DownstreamAgent, HarnessSession, PromptAttachment, PromptRequest,
    SessionKey,
};
use via_i18n::Locale;
use via_mcp_tools::DelegationLookupInput;
use via_work::{CancellationFact, DelegationRef, ReconciliationLedger, WorkOutcome};

use crate::decision::{
    CoordinatorDecision, coordinator_response_state, is_deliverable_state,
    parse_coordinator_decision,
};
use crate::delegation::{
    DelegationRecord, DelegationRegistry, coordinator_session_lane, new_permission_scope_id,
    target_lane,
};
use crate::envelope::{CoordinationRequest, build_coordinator_prompt};
use crate::error::CoordinatorError;
use crate::executor::{ExecutorStopped, KeyedSerialExecutor};
use crate::instructions::coordinator_instructions;
use crate::native::{
    NativeDelegation, NativeDelegationDefaults, NativeDelegationDetector, NativeToolUpdate,
};
use crate::permission::{
    PermissionBroker, PermissionContext, PermissionObserver, PermissionResponse,
};
use crate::profile::CoordinatorProfile;
use crate::prompts::{
    cancel_control_prompt, delegation_result_prompt, protocol_retry_prompt, reconciliation_prompt,
    status_control_prompt,
};

/// How many times a non-deliverable reply is sent back for another try.
///
/// **External contract** — `server/src/agent/coordinator.mjs:267`
/// (`for (let attempt = 0; attempt < 2; attempt += 1)`). Two retries, then the
/// refusal. An off-by-one here changes how long a model that keeps answering
/// `active` holds a voice session open.
pub const PROTOCOL_RETRY_ATTEMPTS: usize = 2;

/// The `role` on every result envelope's backend reference.
///
/// **External contract** — `acp-backend-adapter.mjs:1165`
/// (`role: 'backend'`), a fixed literal.
pub const BACKEND_REF_ROLE: &str = "backend";

/// Lock, treating poisoning as "the value is still there".
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// Where the coordinator's backend reference points.
///
/// **External contract** — the `metadata.backendRef` half of
/// `resultEnvelope` (`acp-backend-adapter.mjs:1163-1168`). It carries a
/// filesystem path, and `via-work`'s
/// [`presentation::project`](via_work::presentation::project) is what stops it
/// reaching a client — *"this is an information-disclosure boundary"*. Nothing
/// here publishes it; it exists so a Gateway can log which session answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendRef {
    /// The backend id.
    pub provider: String,
    /// Always [`BACKEND_REF_ROLE`].
    pub role: &'static str,
    /// The coordinator session's own id.
    pub session_id: String,
    /// Its working directory.
    pub directory: String,
}

/// What one coordination run produced.
///
/// **External contract** — `resultEnvelope`,
/// `acp-backend-adapter.mjs:1157-1181`.
///
/// # One field is not reproduced
///
/// Upstream also carries `raw`, the whole `session/prompt` result.
/// [`via_downstream::PromptOutcome`] does not expose one — the transport has
/// already reduced it to content plus a stop reason — so [`Self::stop_reason`]
/// carries what survived, and `docs/deviations/phase-3.md` records the rest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultEnvelope {
    /// The coordinator's answer, already through
    /// [`via_acp::normalize_coordinator_content`].
    pub content: String,
    /// Why the turn stopped.
    pub stop_reason: String,
    /// The backend id.
    pub protocol: String,
    /// Which session answered.
    pub backend_ref: BackendRef,
    /// The delegation this run went through, if it went through one.
    pub delegation: Option<DelegationRecord>,
}

/// What [`Coordinator::run`] answers with.
///
/// Upstream returns `{content, metadata: {presentation}}`
/// (`coordinator.mjs:286-291`); this adds the envelope, because a caller that
/// wants to log which session answered should not have to parse the content to
/// find out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoordinationOutcome {
    /// The decision, already normalized.
    pub decision: CoordinatorDecision,
    /// Where it came from.
    pub envelope: ResultEnvelope,
}

impl CoordinationOutcome {
    /// The spoken answer. Upstream's `result.content`.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.decision.presentation.speech
    }

    /// This outcome as a [`via_work::WorkOutcome`].
    ///
    /// The metadata is `{"presentation": …}`, which is the shape
    /// [`via_work::presentation::project`] reads — so the Work record ends up
    /// with the projection and nothing else.
    #[must_use]
    pub fn into_work_outcome(self) -> WorkOutcome {
        let presentation = serde_json::to_value(&self.decision.presentation).unwrap_or(Value::Null);
        let mut metadata = serde_json::Map::new();
        metadata.insert("presentation".to_owned(), presentation);
        WorkOutcome::content(&self.decision.presentation.speech)
            .with_metadata(Value::Object(metadata))
    }
}

/// Where the two delegation events go.
///
/// Upstream's `onEvent({type: 'backend.delegated' | 'backend.delegation.completed'})`.
/// The payload is [`via_work::DelegationRef`], which is what
/// [`via_work::RunnerEvent`] already carries — so a Work runner forwards one
/// straight through and the *"a delegation's ids never reach a client"*
/// property is `via-work`'s to keep, not this crate's to re-derive.
pub trait CoordinationObserver: Send + Sync + fmt::Debug {
    /// `backend.delegated` — the Work becomes `delegated` and releases its
    /// scheduler lane. The presentation is spoken immediately as a start
    /// confirmation.
    fn delegated(&self, delegation: &DelegationRef);

    /// `backend.delegation.completed` — the Work becomes `finalizing`. No
    /// presentation: the completion has not been composed yet.
    fn delegation_completed(&self, delegation: &DelegationRef);
}

/// Listing and locating the backend's own project Sessions.
///
/// Upstream reaches straight for `client.listSessions()`
/// (`acp-backend-adapter.mjs:696-719,847-854`), which is an ACP call.
/// [`DownstreamAgent`] deliberately has no such method — not every harness
/// shape has a session list — so it is a seam. A harness that cannot list
/// answers with [`Self::empty`]'s behaviour: no sessions, and no directory,
/// which makes `via_session_send` refuse with
/// [`CoordinatorError::SessionDirectoryUnknown`] rather than resume a Session
/// into the wrong directory.
#[async_trait::async_trait]
pub trait ProjectSessionDirectory: Send + Sync + fmt::Debug {
    /// The backend's project Sessions, newest first, at most `limit`.
    ///
    /// # Errors
    ///
    /// Whatever the harness reports.
    async fn list(
        &self,
        limit: i64,
    ) -> Result<Vec<via_mcp_tools::SessionSummary>, CoordinatorError>;

    /// The working directory of one Session, if the backend knows it.
    ///
    /// # Errors
    ///
    /// Whatever the harness reports.
    async fn directory_of(&self, session_id: &str) -> Result<Option<String>, CoordinatorError>;
}

/// A directory that knows about no Sessions at all.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptySessionDirectory;

#[async_trait::async_trait]
impl ProjectSessionDirectory for EmptySessionDirectory {
    async fn list(
        &self,
        _limit: i64,
    ) -> Result<Vec<via_mcp_tools::SessionSummary>, CoordinatorError> {
        Ok(Vec::new())
    }

    async fn directory_of(&self, _session_id: &str) -> Result<Option<String>, CoordinatorError> {
        Ok(None)
    }
}

/// Waiting on a Layer-3 Session the backend opened by itself.
///
/// Upstream's `nativeDelegationAdapter` (`acp-backend-adapter.mjs:1113-1137`).
/// It exists because a natively delegated Session is not one VIA prompted: the
/// backend owns it, and the only way to learn it finished is to ask the
/// backend.
#[async_trait::async_trait]
pub trait NativeDelegationAdapter: Send + Sync + fmt::Debug {
    /// Wait for `delegation_id` to finish, and return what it said.
    ///
    /// # Errors
    ///
    /// [`CoordinatorError::Harness`] with
    /// [`HarnessError::Cancelled`](via_downstream::HarnessError::Cancelled)
    /// when `signal` fires, and whatever the backend reports otherwise.
    async fn wait(
        &self,
        delegation_id: &str,
        session_id: &str,
        signal: CancellationToken,
    ) -> Result<String, CoordinatorError>;

    /// Ask the backend to stop it.
    ///
    /// # Errors
    ///
    /// Whatever the backend reports. Upstream swallows this failure
    /// (`.catch(() => {})`) because the abort that follows is the real
    /// cancellation; a caller here may do the same.
    async fn cancel(&self, delegation_id: &str, session_id: &str) -> Result<(), CoordinatorError>;
}

/// One coordination turn's observable activity.
///
/// The four inputs to the catalogued empty-response recovery predicate
/// (`error-code` / *empty coordinator response*):
/// `!receivedUpdate && !delegation && nativeToolCalls.size === 0 &&
/// toolCalls.size === 0`.
#[derive(Debug, Default)]
struct TurnActivity {
    received_update: bool,
    tool_calls: usize,
    detector: NativeDelegationDetector,
}

impl TurnActivity {
    /// Whether nothing at all has been observed, so a retry cannot duplicate a
    /// side effect.
    fn is_pristine(&self, delegated: bool) -> bool {
        !self.received_update
            && !delegated
            && self.tool_calls == 0
            && self.detector.tool_calls() == 0
    }
}

/// What one turn is being run for.
#[derive(Clone, Default)]
pub struct TurnOptions {
    /// Whose turn this is. The coordinator session key is built from it.
    pub owner_id: String,
    /// The Work id — upstream's `coordinationRunId`, and the envelope's
    /// `request_id`.
    pub work_id: String,
    /// Files the user attached to the turn.
    pub attachments: Vec<PromptAttachment>,
    /// Where the two delegation events go.
    pub observer: Option<Arc<dyn CoordinationObserver>>,
    /// Where permission events go.
    pub permissions: Option<Arc<dyn PermissionObserver>>,
    /// The Work's own cancellation scope.
    pub signal: Option<CancellationToken>,
}

impl fmt::Debug for TurnOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TurnOptions")
            .field("owner_id", &self.owner_id)
            .field("work_id", &self.work_id)
            .field("attachments", &self.attachments.len())
            .field("observer", &self.observer.is_some())
            .finish_non_exhaustive()
    }
}

impl TurnOptions {
    /// A turn for `owner_id`'s `work_id`.
    #[must_use]
    pub fn new(owner_id: &str, work_id: &str) -> Self {
        Self {
            owner_id: owner_id.to_owned(),
            work_id: work_id.to_owned(),
            ..Self::default()
        }
    }

    /// Attach a file.
    #[must_use]
    pub fn with_attachment(mut self, attachment: PromptAttachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    /// Set the delegation observer.
    #[must_use]
    pub fn with_observer(mut self, observer: Arc<dyn CoordinationObserver>) -> Self {
        self.observer = Some(observer);
        self
    }

    /// Set the permission observer.
    #[must_use]
    pub fn with_permission_observer(mut self, observer: Arc<dyn PermissionObserver>) -> Self {
        self.permissions = Some(observer);
        self
    }

    /// Set the cancellation scope.
    #[must_use]
    pub fn with_signal(mut self, signal: CancellationToken) -> Self {
        self.signal = Some(signal);
        self
    }
}

/// One coordinator turn's result, before it becomes an envelope.
struct Turn {
    session_id: String,
    directory: String,
    content: String,
    stop_reason: String,
}

/// Everything the coordinator holds by value.
#[derive(Default)]
struct State {
    sessions: IndexMap<String, Arc<dyn HarnessSession>>,
    scopes: IndexMap<String, PermissionContext>,
    turns: IndexMap<String, TurnActivity>,
    ledger: ReconciliationLedger,
}

/// The coordinator.
///
/// Cheap to clone; every clone is the same coordinator.
#[derive(Clone)]
pub struct Coordinator {
    agent: Arc<dyn DownstreamAgent>,
    profile: Arc<CoordinatorProfile>,
    locale: Locale,
    executor: KeyedSerialExecutor,
    permissions: PermissionBroker,
    registry: DelegationRegistry,
    directory: Arc<dyn ProjectSessionDirectory>,
    native: Option<Arc<dyn NativeDelegationAdapter>>,
    tracker: TaskTracker,
    state: Arc<Mutex<State>>,
}

impl fmt::Debug for Coordinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Coordinator")
            .field("protocol", &self.profile.protocol)
            .field("label", &self.profile.label)
            .field("delegations", &self.registry.len())
            .finish_non_exhaustive()
    }
}

/// Builds a [`Coordinator`].
pub struct CoordinatorBuilder {
    agent: Arc<dyn DownstreamAgent>,
    profile: CoordinatorProfile,
    locale: Locale,
    directory: Arc<dyn ProjectSessionDirectory>,
    native: Option<Arc<dyn NativeDelegationAdapter>>,
    tracker: Option<TaskTracker>,
}

impl fmt::Debug for CoordinatorBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoordinatorBuilder")
            .field("profile", &self.profile)
            .field("locale", &self.locale)
            .finish_non_exhaustive()
    }
}

impl CoordinatorBuilder {
    /// Set the locale every sentence is rendered in.
    #[must_use]
    pub const fn locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// Supply the project-Session directory.
    #[must_use]
    pub fn session_directory(mut self, directory: Arc<dyn ProjectSessionDirectory>) -> Self {
        self.directory = directory;
        self
    }

    /// Supply the native-delegation adapter.
    #[must_use]
    pub fn native_delegation(mut self, adapter: Arc<dyn NativeDelegationAdapter>) -> Self {
        self.native = Some(adapter);
        self
    }

    /// Register every spawned task on `tracker`, so shutdown joins them.
    #[must_use]
    pub fn tracker(mut self, tracker: TaskTracker) -> Self {
        self.tracker = Some(tracker);
        self
    }

    /// Build, starting the two owning tasks.
    #[must_use]
    pub fn build(self) -> Coordinator {
        let tracker = self.tracker.unwrap_or_default();
        Coordinator {
            agent: self.agent,
            executor: KeyedSerialExecutor::with_tracker(&tracker),
            permissions: PermissionBroker::with_tracker(self.profile.permission_mode, &tracker),
            profile: Arc::new(self.profile),
            locale: self.locale,
            registry: DelegationRegistry::new(),
            directory: self.directory,
            native: self.native,
            tracker,
            state: Arc::new(Mutex::new(State::default())),
        }
    }
}

impl Coordinator {
    /// Start building a coordinator over `agent`.
    #[must_use]
    pub fn builder(
        agent: Arc<dyn DownstreamAgent>,
        profile: CoordinatorProfile,
    ) -> CoordinatorBuilder {
        CoordinatorBuilder {
            agent,
            profile,
            locale: Locale::En,
            directory: Arc::new(EmptySessionDirectory),
            native: None,
            tracker: None,
        }
    }

    /// The backend this coordinator drives.
    #[must_use]
    pub fn profile(&self) -> &CoordinatorProfile {
        &self.profile
    }

    /// The permission broker, for the Gateway's `POST /api/permissions/:id`.
    #[must_use]
    pub const fn permissions(&self) -> &PermissionBroker {
        &self.permissions
    }

    /// The delegations this coordinator owns.
    #[must_use]
    pub const fn delegations(&self) -> &DelegationRegistry {
        &self.registry
    }

    /// Whether a [`NativeDelegationAdapter`] is configured.
    ///
    /// Upstream's `Boolean(this.nativeDelegationAdapter)` half of
    /// `agent.canRecoverDelegatedWork(task)` (`acp-backend-adapter.mjs:1273-1278`)
    /// — the other half, that the persisted delegation carries both ids, is
    /// `via_work::DelegationRef::is_addressable`, checked by `via-work` itself
    /// before a Work is ever offered to
    /// [`via_work::DelegatedWorkRecovery::can_recover`]. VIA's own
    /// composition wires no adapter, so this answers `false` for every
    /// candidate today — see [`Self::recover_native_delegation`].
    #[must_use]
    pub fn native_delegation_configured(&self) -> bool {
        self.native.is_some()
    }

    /// The coordinator session key for `owner_id`.
    ///
    /// **The fixed identity.** Voice session ids and Work ids never enter it.
    #[must_use]
    pub fn session_key(&self, owner_id: &str) -> SessionKey {
        SessionKey::coordinator(&self.profile.protocol, owner_id)
    }

    /// The lane every write to `owner_id`'s coordinator session is serialized
    /// on.
    ///
    /// Not [`via_work::coordinator_lane`]: that one gates Work *admission* and
    /// is keyed on the owner; this one gates session *writes* and is keyed on
    /// the session. `docs/architecture.md` §11 calls the pair a deliberate
    /// double guard.
    #[must_use]
    pub fn coordinator_lane(&self, owner_id: &str) -> String {
        coordinator_session_lane(self.session_key(owner_id).as_str())
    }

    /// The executor guarding session writes.
    ///
    /// Exposed so a Gateway can report queue depth, and so a test can hold the
    /// lane the way a second voice turn would.
    #[must_use]
    pub const fn executor(&self) -> &KeyedSerialExecutor {
        &self.executor
    }

    /// Whether a turn is in flight on `owner_id`'s coordinator session.
    ///
    /// Upstream's `activeCoordinatorTurns.has(session.sessionId)`
    /// (`acp-backend-adapter.mjs:1384-1385`). It is the lane's own depth here,
    /// which also counts a turn that is *queued* — upstream's set does not, and
    /// the difference makes an urgent cancel take the transport route slightly
    /// more often rather than slightly less. Recorded in
    /// `docs/deviations/phase-3.md`.
    pub async fn is_busy(&self, owner_id: &str) -> bool {
        self.executor.depth(&self.coordinator_lane(owner_id)).await > 0
    }

    // ── the turn ladder ────────────────────────────────────────────────────

    /// Run one coordination request to a final answer.
    ///
    /// **External contract** — `Coordinator.run`,
    /// `server/src/agent/coordinator.mjs:244-292`:
    ///
    /// 1. build the envelope prompt;
    /// 2. take a turn;
    /// 3. while the reply's state is neither empty nor `completed`, send
    ///    [`crate::prompts::protocol_retry_prompt`] — at most
    ///    [`PROTOCOL_RETRY_ATTEMPTS`] times;
    /// 4. refuse if it still is not deliverable;
    /// 5. parse the decision against the **expected** Work id.
    ///
    /// # Errors
    ///
    /// [`CoordinatorError::EmptyResponse`] when a turn answers with nothing,
    /// [`CoordinatorError::NotFinalResult`] when the ladder runs out, and
    /// whatever the harness refused.
    pub async fn run(
        &self,
        request: &CoordinationRequest<'_>,
        timestamp: DateTime<Utc>,
        options: &TurnOptions,
    ) -> Result<CoordinationOutcome, CoordinatorError> {
        let prompt = build_coordinator_prompt(request, timestamp, self.locale);
        let envelope = self.run_coordinator(&prompt, options).await?;
        self.finalize(envelope, options).await
    }

    /// The retry ladder and decision parsing shared by [`Self::run`] and
    /// [`Self::recover_native_delegation`] — everything after the *first*
    /// envelope either produced, because a restart-recovered delegation has
    /// no request prompt to build but still owes the model the same number of
    /// chances to produce a deliverable reply.
    async fn finalize(
        &self,
        mut envelope: ResultEnvelope,
        options: &TurnOptions,
    ) -> Result<CoordinationOutcome, CoordinatorError> {
        for _ in 0..PROTOCOL_RETRY_ATTEMPTS {
            let state = coordinator_response_state(&envelope.content);
            if is_deliverable_state(&state) {
                break;
            }
            let retry = protocol_retry_prompt(&options.work_id, &state, self.locale);
            envelope = self.run_coordinator(&retry, options).await?;
        }

        let state = coordinator_response_state(&envelope.content);
        if !is_deliverable_state(&state) {
            return Err(CoordinatorError::NotFinalResult { state });
        }
        let decision = parse_coordinator_decision(&envelope.content, &options.work_id);
        Ok(CoordinationOutcome { decision, envelope })
    }

    /// Take one turn on the coordinator session, delegation lifecycle included.
    ///
    /// **External contract** — `runCoordinator`,
    /// `acp-backend-adapter.mjs:1202-1271`.
    ///
    /// # Errors
    ///
    /// As [`Self::run`].
    pub async fn run_coordinator(
        &self,
        prompt: &str,
        options: &TurnOptions,
    ) -> Result<ResultEnvelope, CoordinatorError> {
        let key = self.session_key(&options.owner_id);
        let lane = self.coordinator_lane(&options.owner_id);

        let initial = self
            .executor
            .run(&lane, self.turn_with_recovery(&key, prompt, options))
            .await??;

        let Some(record) = self.registry.snapshot(&options.work_id) else {
            return Ok(self.envelope(&initial, None));
        };

        // The start confirmation the model just composed, spoken at once.
        let presentation = presentation_value(&initial.content);
        self.finish_delegation(&key, &lane, options, record, presentation)
            .await
    }

    /// Reattach to a delegated Work whose delegation survived a restart.
    ///
    /// **External contract** — `agent.recoverDelegatedWork`,
    /// `acp-backend-adapter.mjs:1281-1345`: recreate the native delegation
    /// from the ids and directory the persisted Work carried, wait for it
    /// exactly as a freshly detected one would, and feed the result back to
    /// the coordinator the same way [`Self::run_coordinator`]'s own
    /// delegation tail does. There is no request prompt here — the
    /// coordinator was never asked anything this process; the only new turn
    /// is the delegation-result presentation.
    ///
    /// The presentation the caller already spoke before the crash
    /// (`delegation.presentation`) is replayed rather than recomposed —
    /// upstream's `presentation: saved.presentation || null`.
    ///
    /// # Errors
    ///
    /// [`CoordinatorError::NotRecoverable`] when this coordinator has no
    /// [`NativeDelegationAdapter`] configured — [`Self::run`]'s [`WorkSnapshot`]
    /// callers are expected to have refused this already
    /// (`docs/deviations/phase-9-via-e2e.md`'s restart-recovery gap; VIA's own
    /// composition wires no adapter today, so this is the sentence that keeps
    /// the loss honest rather than resurrecting a Work with nothing behind
    /// it). Otherwise whatever the adapter or the result turn refuses.
    ///
    /// [`WorkSnapshot`]: via_work::WorkSnapshot
    pub async fn recover_native_delegation(
        &self,
        delegation: &DelegationRef,
        options: &TurnOptions,
    ) -> Result<CoordinationOutcome, CoordinatorError> {
        let Some(adapter) = self.native.clone() else {
            return Err(CoordinatorError::NotRecoverable {
                label: self.profile.label.clone(),
            });
        };
        let record = DelegationRecord::new(
            &delegation.id,
            &delegation.session_id,
            &options.owner_id,
            &options.work_id,
        )
        .directory(delegation.directory.as_deref().unwrap_or_default())
        .title(
            delegation.title.as_deref().unwrap_or_default(),
            "",
            &self.profile.delegation_title,
        );
        let signal = CancellationToken::new();
        let (settle, completion) = oneshot::channel();
        self.registry
            .start(record.clone(), signal.clone(), completion)?;
        let delegation_id = record.id.clone();
        let session_id = record.session_id.clone();
        self.tracker.spawn(async move {
            let outcome = adapter.wait(&delegation_id, &session_id, signal).await;
            let _ = settle.send(outcome);
        });

        let key = self.session_key(&options.owner_id);
        let lane = self.coordinator_lane(&options.owner_id);
        let presentation = delegation.presentation.clone().unwrap_or(Value::Null);
        let envelope = self
            .finish_delegation(&key, &lane, options, record, presentation)
            .await?;
        self.finalize(envelope, options).await
    }

    /// Announce, wait for and finalize one delegation — the shared second
    /// half of [`Self::run_coordinator`] and [`Self::recover_native_delegation`].
    ///
    /// **External contract** — the delegation half of `runCoordinator`,
    /// `acp-backend-adapter.mjs:1230-1268`.
    async fn finish_delegation(
        &self,
        key: &SessionKey,
        lane: &str,
        options: &TurnOptions,
        record: DelegationRecord,
        presentation: Value,
    ) -> Result<ResultEnvelope, CoordinatorError> {
        let announced = record.to_ref().with_presentation(presentation);
        if let Some(observer) = options.observer.as_ref() {
            observer.delegated(&announced);
        }

        // Outside the lane from here: the coordinator is free while the target
        // runs, which is the whole point of the delegated response.
        // A dropped sender means the delegation runner went away without
        // answering — a shutdown, not a backend failure — and a second caller
        // asking for a completion somebody else already took is the same class
        // of wiring bug. Neither may be reported as a result.
        let outcome = match self.registry.take_completion(&options.work_id) {
            Some(completion) => completion
                .await
                .unwrap_or(Err(CoordinatorError::Stopped(ExecutorStopped))),
            None => Err(CoordinatorError::DelegationNotFound {
                label: self.profile.label.clone(),
            }),
        };
        self.registry
            .settle(&options.work_id, &outcome, self.locale);
        let settled = self
            .registry
            .snapshot(&options.work_id)
            .unwrap_or_else(|| record.clone());
        if let Err(error) = outcome {
            self.registry.remove(&options.work_id);
            return Err(error);
        }

        if let Some(observer) = options.observer.as_ref() {
            observer.delegation_completed(&settled.to_ref());
        }

        let result_prompt =
            delegation_result_prompt(&settled.to_result(), &options.work_id, self.locale);
        let finalized = self
            .executor
            .run(lane, self.turn_with_recovery(key, &result_prompt, options))
            .await?;
        self.registry.remove(&options.work_id);
        Ok(self.envelope(&finalized?, Some(settled)))
    }

    /// One turn, retried **once** in a fresh session when the reply was empty
    /// and nothing at all had happened yet.
    ///
    /// **External contract** — `coordinatorTurnWithRecovery`,
    /// `acp-backend-adapter.mjs:1192-1200`, gated by the four-part purity
    /// predicate on [`CoordinatorError::EmptyResponse`].
    async fn turn_with_recovery(
        &self,
        key: &SessionKey,
        prompt: &str,
        options: &TurnOptions,
    ) -> Result<Turn, CoordinatorError> {
        match self.turn(key, prompt, options).await {
            Err(error) if error.is_recoverable() => {
                self.discard_session(key);
                self.turn(key, prompt, options).await
            }
            other => other,
        }
    }

    /// One turn on the fixed coordinator session.
    ///
    /// **External contract** — `coordinatorTurn`,
    /// `acp-backend-adapter.mjs:989-1084`.
    async fn turn(
        &self,
        key: &SessionKey,
        prompt: &str,
        options: &TurnOptions,
    ) -> Result<Turn, CoordinatorError> {
        let facts = self.pending_facts(&options.owner_id);
        let reconciled = reconciliation_prompt(&facts, prompt, self.locale);
        let wrapped = coordinator_instructions(&self.profile.session_instructions, &reconciled);

        let session = self.ensure_session(key).await?;
        let session_id = session.session_id().to_owned();
        let scope_id = new_permission_scope_id();
        self.enter_scope(&session_id, &scope_id, options);
        self.begin_turn(&options.work_id);

        let mut request = PromptRequest::text(&options.owner_id, &wrapped);
        request.attachments = options.attachments.clone();
        request.timeout_ms = self.profile.timeout_ms();
        if !options.work_id.is_empty() {
            request = request.with_work_id(&options.work_id);
        }

        let outcome = session.prompt(request).await;

        // Upstream's `finally`: the prompt's permissions go away with the
        // prompt, whether it returned, failed, or was cancelled.
        self.permissions.cancel_scope(&scope_id).await;
        self.leave_scope(&session_id, &scope_id);

        let outcome = outcome.map_err(CoordinatorError::Harness);
        let outcome = match outcome {
            Ok(outcome) => outcome,
            Err(error) => {
                self.end_turn(&options.work_id);
                return Err(error);
            }
        };

        if clean(outcome.content()).is_empty() {
            let recoverable = self.turn_is_pristine(&options.work_id);
            self.end_turn(&options.work_id);
            return Err(CoordinatorError::EmptyResponse {
                label: self.profile.label.clone(),
                recoverable,
            });
        }
        self.end_turn(&options.work_id);

        // Cleared only after a turn actually carried them: a turn that returned
        // nothing has told the model nothing.
        if !facts.is_empty() {
            self.clear_facts(&options.owner_id);
        }

        Ok(Turn {
            session_id,
            directory: self.profile.directory.clone(),
            content: normalize_coordinator_content(outcome.content(), self.locale),
            stop_reason: outcome.stop_reason().as_str().to_owned(),
        })
    }

    fn envelope(&self, turn: &Turn, delegation: Option<DelegationRecord>) -> ResultEnvelope {
        ResultEnvelope {
            content: turn.content.clone(),
            stop_reason: turn.stop_reason.clone(),
            protocol: self.profile.protocol.clone(),
            backend_ref: BackendRef {
                provider: self.profile.protocol.clone(),
                role: BACKEND_REF_ROLE,
                session_id: turn.session_id.clone(),
                directory: turn.directory.clone(),
            },
            delegation,
        }
    }

    // ── the fixed session ──────────────────────────────────────────────────

    /// Open, or reuse, `key`'s session.
    ///
    /// There is no double-open guard here and none is needed: every path that
    /// reaches this is already inside [`coordinator_session_lane`], so two
    /// turns for one owner cannot race. Upstream needs a
    /// `coordinatorSessionPromises` map because its health probe opens sessions
    /// outside the queue; VIA's availability probe is
    /// [`DownstreamAgent::health`], which opens nothing.
    async fn ensure_session(
        &self,
        key: &SessionKey,
    ) -> Result<Arc<dyn HarnessSession>, CoordinatorError> {
        if let Some(session) = lock(&self.state).sessions.get(key.as_str()) {
            return Ok(Arc::clone(session));
        }
        let session: Arc<dyn HarnessSession> = Arc::from(self.agent.open(key).await?);
        lock(&self.state)
            .sessions
            .insert(key.as_str().to_owned(), Arc::clone(&session));
        Ok(session)
    }

    /// Forget `key`'s session, so the next turn opens a fresh one.
    ///
    /// **External contract** — `discardCoordinatorSession`,
    /// `acp-backend-adapter.mjs:1183-1190`.
    pub fn discard_session(&self, key: &SessionKey) {
        lock(&self.state).sessions.shift_remove(key.as_str());
    }

    // ── reconciliation ─────────────────────────────────────────────────────

    /// Record something the Gateway did that the model does not know about.
    ///
    /// Delegates to [`via_work::ReconciliationLedger`], which caps at twenty per
    /// owner and keeps the newest.
    pub fn record_fact(&self, owner_id: &str, fact: CancellationFact) {
        lock(&self.state).ledger.record(owner_id, fact);
    }

    /// What `owner_id` still owes the coordinator.
    #[must_use]
    pub fn pending_facts(&self, owner_id: &str) -> Vec<CancellationFact> {
        lock(&self.state).ledger.pending(owner_id).to_vec()
    }

    fn clear_facts(&self, owner_id: &str) -> usize {
        lock(&self.state).ledger.clear(owner_id)
    }

    // ── permissions ────────────────────────────────────────────────────────

    fn enter_scope(&self, session_id: &str, scope_id: &str, options: &TurnOptions) {
        let context = PermissionContext {
            owner_id: options.owner_id.clone(),
            work_id: (!options.work_id.is_empty()).then(|| options.work_id.clone()),
            session_id: session_id.to_owned(),
            scope_id: scope_id.to_owned(),
            observer: options.permissions.clone(),
            signal: options.signal.clone(),
        };
        lock(&self.state)
            .scopes
            .insert(session_id.to_owned(), context);
    }

    /// Upstream's `if (session.permissionScopeId === permissionScopeId)` guard:
    /// a scope that has already been replaced by a later prompt is not cleared
    /// by an earlier one finishing.
    fn leave_scope(&self, session_id: &str, scope_id: &str) {
        let mut state = lock(&self.state);
        if state
            .scopes
            .get(session_id)
            .is_some_and(|context| context.scope_id == scope_id)
        {
            state.scopes.shift_remove(session_id);
        }
    }

    /// Answer one `session/request_permission` from `session_id`.
    ///
    /// The entry point a transport wires its permission handler to. A request
    /// from a session with no active prompt still gets an answer — with a blank
    /// owner, so nobody can respond to it and it is cancelled with its scope —
    /// which is upstream's behaviour when `session` is undefined.
    ///
    /// # Errors
    ///
    /// [`CoordinatorError::Stopped`] when the broker's task is gone.
    pub async fn handle_permission(
        &self,
        session_id: &str,
        tool_call: &Value,
    ) -> Result<PermissionDecision, CoordinatorError> {
        let context = lock(&self.state)
            .scopes
            .get(clean(session_id))
            .cloned()
            .unwrap_or_default();
        Ok(self.permissions.request(tool_call, context).await?)
    }

    /// Record a person's decision on a pending permission.
    ///
    /// # Errors
    ///
    /// [`CoordinatorError::PermissionRequestUnknown`].
    pub async fn respond_permission(
        &self,
        id: &str,
        response: PermissionResponse,
        owner_id: &str,
    ) -> Result<via_work::PendingPermission, CoordinatorError> {
        self.permissions.respond(id, response, owner_id).await
    }

    // ── turn activity ──────────────────────────────────────────────────────

    fn begin_turn(&self, work_id: &str) {
        lock(&self.state)
            .turns
            .insert(work_id.to_owned(), TurnActivity::default());
    }

    fn end_turn(&self, work_id: &str) {
        lock(&self.state).turns.shift_remove(work_id);
    }

    fn turn_is_pristine(&self, work_id: &str) -> bool {
        let delegated = self.registry.snapshot(work_id).is_some();
        lock(&self.state)
            .turns
            .get(work_id)
            .is_none_or(|activity| activity.is_pristine(delegated))
    }

    /// Fold one generic activity event into the running turn.
    ///
    /// It is the `run.receivedUpdate = true` and `run.toolCalls` half of
    /// upstream's `onSessionUpdate` — the half that decides whether an empty
    /// reply may be retried. The *projection* into
    /// [`via_downstream::SessionEvent`] is `via-downstream`'s and happens before
    /// this is called.
    pub fn observe_activity(&self, work_id: &str, is_tool_call: bool) {
        let mut state = lock(&self.state);
        if let Some(activity) = state.turns.get_mut(work_id) {
            activity.received_update = true;
            if is_tool_call {
                activity.tool_calls += 1;
            }
        }
    }

    /// Fold one raw tool-call notification into the running turn, and start a
    /// delegation if it turns out to be one.
    ///
    /// **External contract** — the detection predicate in [`crate::native`],
    /// gated by [`CoordinatorProfile::native_delegation`] exactly as upstream's
    /// `if (!this.profile.nativeDelegation) return` gates it.
    ///
    /// Returns the delegation when this update created one.
    pub fn observe_native(
        &self,
        options: &TurnOptions,
        update: &NativeToolUpdate,
    ) -> Option<NativeDelegation> {
        if !self.profile.native_delegation {
            return None;
        }
        let defaults = NativeDelegationDefaults {
            directory: self.profile.directory.clone(),
            title: self.profile.delegation_title.clone(),
        };
        let detected = {
            let mut state = lock(&self.state);
            let activity = state.turns.get_mut(&options.work_id)?;
            activity.received_update = true;
            activity.observe(update, &defaults)
        }?;
        self.start_native_delegation(options, &detected);
        Some(detected)
    }

    fn start_native_delegation(&self, options: &TurnOptions, detected: &NativeDelegation) {
        let Some(adapter) = self.native.clone() else {
            return;
        };
        let record = DelegationRecord::new(
            &detected.delegation_id,
            &detected.session_id,
            &options.owner_id,
            &options.work_id,
        )
        .directory(&detected.directory)
        .title(&detected.title, "", &self.profile.delegation_title);
        let signal = CancellationToken::new();
        let (settle, completion) = oneshot::channel();
        if self
            .registry
            .start(record.clone(), signal.clone(), completion)
            .is_err()
        {
            return;
        }
        let delegation_id = record.id.clone();
        let session_id = record.session_id.clone();
        self.tracker.spawn(async move {
            let outcome = adapter.wait(&delegation_id, &session_id, signal).await;
            let _ = settle.send(outcome);
        });
    }

    // ── the MCP tool context ───────────────────────────────────────────────

    /// The five coordination tools, wired to this coordinator for one turn.
    ///
    /// Upstream's `toolContext(run)`
    /// (`acp-backend-adapter.mjs:930-938`). Hand it to
    /// [`via_mcp_tools::SessionToolServer::register`]; the descriptor that comes
    /// back goes into the backend session's `mcpServers`.
    #[must_use]
    pub fn tool_context(
        &self,
        options: &TurnOptions,
    ) -> Arc<dyn via_mcp_tools::SessionToolContext> {
        Arc::new(CoordinatorTools {
            coordinator: self.clone(),
            options: options.clone(),
        })
    }

    /// Open a project Session and delegate `prompt` to it.
    async fn start_project_session(
        &self,
        options: &TurnOptions,
        prompt: &str,
        title: &str,
    ) -> Result<DelegationRecord, CoordinatorError> {
        // An empty project key asks the harness for a **new** Session; it
        // reports the id it chose through `session_id()`.
        let key = SessionKey::project(&self.profile.protocol, "");
        let session: Arc<dyn HarnessSession> = Arc::from(self.agent.open(&key).await?);
        let directory = self.profile.directory.clone();
        self.delegate(options, session, prompt, title, &directory)
            .await
    }

    /// Resume a project Session and delegate `prompt` to it.
    async fn continue_project_session(
        &self,
        options: &TurnOptions,
        session_id: &str,
        prompt: &str,
    ) -> Result<DelegationRecord, CoordinatorError> {
        let remembered = self
            .registry
            .find(&DelegationLookupInput {
                delegation_id: None,
                session_id: Some(session_id.to_owned()),
            })
            .map(|record| record.directory)
            .filter(|directory| !directory.is_empty());
        let directory = match remembered {
            Some(directory) => directory,
            None => self
                .directory
                .directory_of(session_id)
                .await?
                .map(|directory| clean(&directory).to_owned())
                .filter(|directory| !directory.is_empty())
                .ok_or_else(|| CoordinatorError::SessionDirectoryUnknown {
                    label: self.profile.label.clone(),
                })?,
        };
        let key = SessionKey::project(&self.profile.protocol, session_id);
        let session: Arc<dyn HarnessSession> = Arc::from(self.agent.open(&key).await?);
        self.delegate(options, session, prompt, prompt, &directory)
            .await
    }

    /// Register a delegation and start the prompt that carries it out.
    ///
    /// The prompt runs on [`target_lane`] — its own lane — and carries
    /// **no timeout**: `docs/architecture.md` §11's fourth invariant says the
    /// request timeout does not apply while waiting on a delegated session.
    async fn delegate(
        &self,
        options: &TurnOptions,
        session: Arc<dyn HarnessSession>,
        prompt: &str,
        title: &str,
        directory: &str,
    ) -> Result<DelegationRecord, CoordinatorError> {
        let record = DelegationRecord::new(
            &crate::delegation::new_delegation_id(&self.profile.protocol),
            session.session_id(),
            &options.owner_id,
            &options.work_id,
        )
        .directory(directory)
        .title(title, prompt, &self.profile.delegation_title);

        let signal = CancellationToken::new();
        let (settle, completion) = oneshot::channel();
        self.registry
            .start(record.clone(), signal.clone(), completion)?;

        let lane = target_lane(&record.session_id);
        let executor = self.executor.clone();
        let permissions = self.permissions.clone();
        let owner = options.owner_id.clone();
        let work = options.work_id.clone();
        let text = prompt.to_owned();
        let coordinator = self.clone();
        let session_id = record.session_id.clone();
        // The delegated prompt inherits the coordination run's permission
        // observer, so a project Session's permission prompt reaches the same
        // user — but it gets its **own** scope, so cancelling the coordinator's
        // prompt does not cancel the project's.
        let scoped = TurnOptions {
            permissions: options.permissions.clone(),
            ..TurnOptions::new(&options.owner_id, &options.work_id)
        };
        self.tracker.spawn(async move {
            let scope_id = new_permission_scope_id();
            let run = async {
                let mut request = PromptRequest::text(&owner, &text);
                request.timeout_ms = None;
                if !work.is_empty() {
                    request = request.with_work_id(&work);
                }
                coordinator.enter_scope(&session_id, &scope_id, &scoped);
                let outcome = tokio::select! {
                    outcome = session.prompt(request) => outcome,
                    () = signal.cancelled() => Err(via_downstream::HarnessError::Cancelled),
                };
                permissions.cancel_scope(&scope_id).await;
                coordinator.leave_scope(&session_id, &scope_id);
                outcome
                    .map(|outcome| outcome.content().to_owned())
                    .map_err(CoordinatorError::Harness)
            };
            let outcome = executor
                .run(&lane, run)
                .await
                .unwrap_or_else(|stopped| Err(CoordinatorError::Stopped(stopped)));
            let _ = settle.send(outcome);
        });
        Ok(record)
    }

    // ── control turns ──────────────────────────────────────────────────────

    /// Ask the coordinator to cancel a delegated Session, or tell the transport
    /// to.
    ///
    /// **External contract** — `cancelDelegatedWork`,
    /// `acp-backend-adapter.mjs:1373-1423`:
    ///
    /// * when the coordinator session is **idle**, ask the model — its own
    ///   cancel tool returning *is* the confirmation, so the outcome is
    ///   [`CancelRoute::Coordinator`];
    /// * when it is busy, or when the control turn threw, go straight down the
    ///   transport, because *"cancellation is urgent"*, and leave a
    ///   reconciliation fact so the model is told next turn.
    ///
    /// # Errors
    ///
    /// [`CoordinatorError::NotCancellable`] when nothing of this owner's
    /// answers to `work_id`.
    pub async fn cancel_delegated_work(
        &self,
        work_id: &str,
        owner_id: &str,
        confirmed_at: &str,
    ) -> Result<CancelOutcome, CoordinatorError> {
        let record = self.registry.for_owner(work_id, owner_id).ok_or_else(|| {
            CoordinatorError::NotCancellable {
                label: self.profile.label.clone(),
            }
        })?;
        let lookup = DelegationLookupInput {
            delegation_id: Some(record.id.clone()),
            session_id: None,
        };

        // Upstream asks the model only when the coordinator session is idle
        // (`activeCoordinatorTurns.has(...)`). Waiting for a busy one would
        // queue an urgent cancel behind the very turn it is trying to stop, so
        // a non-empty lane goes straight to the transport.
        let busy = self.is_busy(owner_id).await;
        let instruction = self.profile.cancel_instruction.as_deref();
        let prompt = cancel_control_prompt(&record.id, instruction, self.locale);
        let options = TurnOptions::new(owner_id, work_id);
        if !busy && self.control_turn(&prompt, &options).await.is_ok() {
            // The model's own tool call came back, so this is a confirmation
            // rather than a request. `via-downstream` refuses to let it be one
            // without a timestamp.
            if let Some(native) = self.native.as_ref() {
                let _ = native.cancel(&record.id, &record.session_id).await;
            }
            let outcome = self.registry.cancel(&lookup, CancelRoute::Coordinator);
            return Ok(if matches!(outcome, CancelOutcome::Requested { .. }) {
                outcome
                    .confirm(confirmed_at)
                    .unwrap_or(CancelOutcome::NotFound)
            } else {
                outcome
            });
        }

        if let Some(native) = self.native.as_ref() {
            let _ = native.cancel(&record.id, &record.session_id).await;
        }
        let outcome = self.registry.cancel(&lookup, CancelRoute::Adapter);
        self.record_fact(
            owner_id,
            CancellationFact::delegated_session_cancelled(
                work_id,
                &record.id,
                &record.session_id,
                confirmed_at,
            ),
        );
        Ok(outcome)
    }

    /// Ask the coordinator how a delegated Session is doing.
    ///
    /// **External contract** — `queryDelegatedWork`,
    /// `acp-backend-adapter.mjs:1425-1445`. The answer is parsed as a decision,
    /// exactly as a completion is, because the control turn is instructed to
    /// return the same `completed`/`respond` JSON.
    ///
    /// # Errors
    ///
    /// [`CoordinatorError::DelegationNotFound`] when nothing of this owner's
    /// answers to `work_id`.
    pub async fn query_delegated_work(
        &self,
        work_id: &str,
        question: &str,
        owner_id: &str,
    ) -> Result<CoordinationOutcome, CoordinatorError> {
        let record = self.registry.for_owner(work_id, owner_id).ok_or_else(|| {
            CoordinatorError::DelegationNotFound {
                label: self.profile.label.clone(),
            }
        })?;
        let prompt = status_control_prompt(
            &record.id,
            question,
            self.profile.status_instruction.as_deref(),
            self.locale,
        );
        let options = TurnOptions::new(owner_id, work_id);
        let turn = self.control_turn(&prompt, &options).await?;
        let envelope = self.envelope(&turn, Some(record));
        let decision = parse_coordinator_decision(&envelope.content, work_id);
        Ok(CoordinationOutcome { decision, envelope })
    }

    /// One hidden control turn, on the coordinator lane with no event
    /// observers.
    ///
    /// **External contract** — `coordinatorControl`,
    /// `acp-backend-adapter.mjs:1357-1371`, which passes `onEvent: null`: a
    /// control turn is not the user's Work and its activity is not the user's
    /// progress.
    async fn control_turn(
        &self,
        prompt: &str,
        options: &TurnOptions,
    ) -> Result<Turn, CoordinatorError> {
        let key = self.session_key(&options.owner_id);
        let lane = self.coordinator_lane(&options.owner_id);
        self.executor
            .run(&lane, self.turn(&key, prompt, options))
            .await?
    }

    /// Cancel everything and stop the owning tasks.
    ///
    /// **External contract** — `close`, `acp-backend-adapter.mjs:1458-1479`:
    /// every live delegation is aborted and every pending permission is
    /// cancelled.
    pub async fn close(self) {
        self.registry.cancel_all();
        self.permissions.cancel_all().await;
        lock(&self.state).sessions.clear();
        self.tracker.close();
    }
}

/// `coordinatorPresentation(initial.result.content)` as JSON, or `null`.
fn presentation_value(content: &str) -> Value {
    coordinator_presentation(content)
        .and_then(|presentation| serde_json::to_value(presentation).ok())
        .unwrap_or(Value::Null)
}

impl TurnActivity {
    fn observe(
        &mut self,
        update: &NativeToolUpdate,
        defaults: &NativeDelegationDefaults,
    ) -> Option<NativeDelegation> {
        self.detector.observe(update, defaults)
    }
}

/// The five tools, wired to one coordination run.
#[derive(Debug)]
struct CoordinatorTools {
    coordinator: Coordinator,
    options: TurnOptions,
}

#[async_trait::async_trait]
impl via_mcp_tools::SessionToolContext for CoordinatorTools {
    async fn list_sessions(
        &self,
        input: via_mcp_tools::SessionsListInput,
    ) -> Result<via_mcp_tools::SessionsListResult, via_downstream::HarnessError> {
        let sessions = self
            .coordinator
            .directory
            .list(input.effective_limit())
            .await
            .map_err(|error| into_harness(error, self.coordinator.locale))?;
        let needle = input.needle();
        Ok(via_mcp_tools::SessionsListResult::new(
            sessions
                .into_iter()
                .filter(|summary| via_mcp_tools::matches_query(summary, needle.as_deref()))
                .collect(),
        ))
    }

    async fn start_session(
        &self,
        input: via_mcp_tools::SessionStartInput,
    ) -> Result<via_mcp_tools::DelegationStarted, via_downstream::HarnessError> {
        self.coordinator
            .start_project_session(
                &self.options,
                clean(&input.prompt),
                clean(input.title.as_deref().unwrap_or_default()),
            )
            .await
            .map(|record| record.started())
            .map_err(|error| into_harness(error, self.coordinator.locale))
    }

    async fn send_session(
        &self,
        input: via_mcp_tools::SessionSendInput,
    ) -> Result<via_mcp_tools::DelegationStarted, via_downstream::HarnessError> {
        self.coordinator
            .continue_project_session(
                &self.options,
                clean(&input.session_id),
                clean(&input.prompt),
            )
            .await
            .map(|record| record.started())
            .map_err(|error| into_harness(error, self.coordinator.locale))
    }

    async fn session_status(
        &self,
        input: DelegationLookupInput,
    ) -> Result<via_mcp_tools::SessionStatusResult, via_downstream::HarnessError> {
        Ok(self.coordinator.registry.status(&input))
    }

    async fn cancel_session(
        &self,
        input: DelegationLookupInput,
    ) -> Result<CancelOutcome, via_downstream::HarnessError> {
        Ok(self
            .coordinator
            .registry
            .cancel(&input, CancelRoute::Adapter))
    }
}

/// A coordination refusal, as the MCP tool envelope carries it.
///
/// `via_mcp_tools`' handlers fail with [`via_downstream::HarnessError`], which
/// already carries a localized sentence and a status; a
/// [`CoordinatorError::Harness`] passes straight through and everything else
/// becomes [`HarnessError::Agent`](via_downstream::HarnessError::Agent) with
/// this crate's own code as the protocol.
fn into_harness(error: CoordinatorError, locale: Locale) -> via_downstream::HarnessError {
    match error {
        CoordinatorError::Harness(harness) => harness,
        other => via_downstream::HarnessError::Agent {
            message: other.message(locale),
            status: other.http_status(),
            body: String::new(),
            protocol: other.code().to_owned(),
        },
    }
}
