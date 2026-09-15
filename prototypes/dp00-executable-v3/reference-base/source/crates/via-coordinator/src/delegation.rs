//! The Layer-3 delegations one coordinator owns, and the five tools that
//! create them.
//!
//! `server/src/agent/acp-backend-adapter.mjs:696-938` — `createDelegation`,
//! `startProjectSession`, `continueProjectSession`, `statusForDelegation`,
//! `cancelDelegation`, `findDelegation` and the `toolContext(run)` that wires
//! them to MCP.
//!
//! # The lock that is released, and the one that is not
//!
//! A delegated prompt is serialized on `target:<session id>`
//! ([`target_lane`]) — its **own** lane, not the coordinator's. That is what
//! lets `docs/reference/contracts.json`'s *"the adapter … releases both the
//! serialization lock and the Work scheduler lane so other voice requests can
//! use the coordinator while the target runs"* be true: the coordinator turn
//! that started the delegation ends, gives its lane back, and the delegated
//! prompt keeps running on a lane nobody else wants.
//!
//! # One turn delegates once
//!
//! [`DelegationRegistry::start`] refuses a second delegation for the same Work
//! with [`CoordinatorError::DelegationAlreadyStarted`] — upstream's
//! `当前协调轮次已经启动了一个第三层任务`. The model is told the rule in
//! [`via_i18n::keys::COORDINATOR_DELEGATION_NOTE`]; this is what happens when it
//! does it anyway.
//!
//! # Owner scoping is not-found, never forbidden
//!
//! The catalogue is explicit (`error-code` / *delegation lookup / lifecycle
//! errors*): *"a delegation belonging to another owner is reported as
//! not-found, never as forbidden."* [`DelegationRegistry::for_owner`] is the
//! only lookup the cancel and query paths use, and it answers `None` for both
//! causes.

use std::sync::{Arc, Mutex, MutexGuard};

use indexmap::IndexMap;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use via_downstream::text::{bounded, clean};
use via_downstream::{CancelOutcome, CancelRoute, CancelTarget, TerminalState};
use via_mcp_tools::{
    DelegationLookupInput, DelegationOutcome, DelegationStarted, DelegationStatus,
    SessionStatusResult,
};
use via_work::DelegationRef;

use crate::error::CoordinatorError;
use crate::prompts::DelegationResult;

/// What separates a backend id from the UUID in a delegation id.
///
/// **External contract** — `acp-backend-adapter.mjs:751`:
/// `` `${this.protocol}_run_${randomUUID()}` ``, e.g. `opencode_run_9f1c…`.
/// The UUID keeps its dashes on this path, and the native path uses the
/// backend's own `runId` instead — *"the two paths produce structurally
/// different ids and both must round-trip"*.
pub const DELEGATION_ID_INFIX: &str = "_run_";

/// The prefix on a permission scope id.
///
/// **External contract** — `acp-backend-adapter.mjs:765,1027`:
/// `` `prompt_${randomUUID()}` ``, one per coordinator turn and one per
/// delegated project prompt. It is what bounds permission cancellation to the
/// prompt that raised it.
pub const PERMISSION_SCOPE_PREFIX: &str = "prompt_";

/// The prefix on a delegated prompt's serialization lane.
///
/// **External contract** — `acp-backend-adapter.mjs:764`
/// (`` this.serialize(`target:${record.sessionId}`, …) ``).
pub const TARGET_LANE_PREFIX: &str = "target:";

/// The prefix on a coordinator turn's serialization lane.
///
/// **External contract** — `acp-backend-adapter.mjs:1223`
/// (`` this.serialize(`coordinator:${key}`, …) ``), where `key` is the
/// [`SessionKey::coordinator`](via_downstream::SessionKey::coordinator) string.
/// Note this is *not* [`via_work::coordinator_lane`]: that one is keyed on the
/// owner id and gates Work **admission**, this one is keyed on the session key
/// and gates session **writes**. `docs/architecture.md` §11 calls the pair a
/// deliberate double guard.
pub const COORDINATOR_LANE_PREFIX: &str = "coordinator:";

/// The bound on a delegation title.
///
/// **External contract** — `acp-backend-adapter.mjs:754`
/// (`bounded(title || prompt, 160)`).
pub const TITLE_BOUND: usize = 160;

/// A fresh delegation id for the MCP path.
#[must_use]
pub fn new_delegation_id(protocol: &str) -> String {
    format!("{protocol}{DELEGATION_ID_INFIX}{}", Uuid::new_v4())
}

/// A fresh permission scope id.
#[must_use]
pub fn new_permission_scope_id() -> String {
    format!("{PERMISSION_SCOPE_PREFIX}{}", Uuid::new_v4())
}

/// The lane a delegated prompt is serialized on.
#[must_use]
pub fn target_lane(session_id: &str) -> String {
    format!("{TARGET_LANE_PREFIX}{}", clean(session_id))
}

/// The lane a coordinator turn is serialized on.
#[must_use]
pub fn coordinator_session_lane(session_key: &str) -> String {
    format!("{COORDINATOR_LANE_PREFIX}{session_key}")
}

/// One delegated Layer-3 Session.
///
/// The internal record. Its `id`, `session_id` and `directory` are the three
/// fields [`via_work::PublicDelegation`] deliberately drops on the way to a
/// client, and the three [`via_work::DelegationRef`] persists because restart
/// recovery needs them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegationRecord {
    /// `<protocol>_run_<uuid>`, or the backend's own `runId`.
    pub id: String,
    /// The Layer-3 Session's id.
    pub session_id: String,
    /// That Session's project directory.
    pub directory: String,
    /// Its title, bounded to [`TITLE_BOUND`].
    pub title: String,
    /// Whose delegation this is. The lookup is owner-scoped.
    pub owner_id: String,
    /// The Work it was delegated for — upstream's `coordinationRunId`.
    pub work_id: String,
    /// Where it is.
    pub status: DelegationStatus,
    /// What it answered, once it has.
    pub result: Option<String>,
    /// Why it failed, if it did.
    pub error: Option<String>,
}

impl DelegationRecord {
    /// A running delegation.
    #[must_use]
    pub fn new(id: &str, session_id: &str, owner_id: &str, work_id: &str) -> Self {
        Self {
            id: clean(id).to_owned(),
            session_id: clean(session_id).to_owned(),
            directory: String::new(),
            title: String::new(),
            owner_id: clean(owner_id).to_owned(),
            work_id: clean(work_id).to_owned(),
            status: DelegationStatus::Running,
            result: None,
            error: None,
        }
    }

    /// Set the project directory.
    #[must_use]
    pub fn directory(mut self, directory: &str) -> Self {
        self.directory = clean(directory).to_owned();
        self
    }

    /// Set the title, applying upstream's `bounded(title || prompt, 160)` and
    /// its fallback to the profile's own.
    #[must_use]
    pub fn title(mut self, title: &str, prompt: &str, fallback: &str) -> Self {
        let chosen = if title.is_empty() { prompt } else { title };
        let bounded_title = bounded(chosen, TITLE_BOUND);
        self.title = if bounded_title.is_empty() {
            fallback.to_owned()
        } else {
            bounded_title
        };
        self
    }

    /// The identity half, as the MCP tools report it.
    #[must_use]
    pub fn identity(&self) -> via_mcp_tools::DelegationRecord {
        via_mcp_tools::DelegationRecord::new(
            &self.id,
            &self.session_id,
            &self.title,
            &self.directory,
        )
    }

    /// The `started` answer `via_session_start` / `via_session_send` return.
    #[must_use]
    pub fn started(&self) -> DelegationStarted {
        self.identity().started()
    }

    /// The Work-record shape, for persistence and restart recovery.
    ///
    /// [`via_work::DelegationRef`] is the persisted form; this is the only
    /// conversion, so the two cannot disagree about which fields survive.
    #[must_use]
    pub fn to_ref(&self) -> DelegationRef {
        let mut reference = DelegationRef::new(&self.id, &self.session_id).with_title(&self.title);
        if !self.directory.is_empty() {
            reference = reference.with_directory(&self.directory);
        }
        reference.status = Some(self.status.as_str().to_owned());
        reference
    }

    /// The status answer `via_session_status` returns.
    ///
    /// **External contract** — `statusForDelegation`,
    /// `acp-backend-adapter.mjs:889-905`: `result` appears only on a completed
    /// delegation and `error` only on a failed one, and the clip to 4 000
    /// characters is [`via_mcp_tools::STATUS_RESULT_BOUND`]'s.
    #[must_use]
    pub fn status_result(&self) -> SessionStatusResult {
        let outcome = match self.status {
            DelegationStatus::Completed => {
                DelegationOutcome::completed(self.result.as_deref().unwrap_or_default())
            }
            DelegationStatus::Failed => {
                DelegationOutcome::failed(self.error.as_deref().unwrap_or_default())
            }
            DelegationStatus::Running => DelegationOutcome::Running,
            DelegationStatus::Cancelling => DelegationOutcome::Cancelling,
            DelegationStatus::Cancelled => DelegationOutcome::Cancelled,
        };
        SessionStatusResult::known(self.identity(), outcome)
    }

    /// The finished result, for the delegation-result prompt.
    #[must_use]
    pub fn to_result(&self) -> DelegationResult {
        DelegationResult {
            delegation_id: self.id.clone(),
            target_session_id: self.session_id.clone(),
            directory: self.directory.clone(),
            content: self.result.clone().unwrap_or_default(),
        }
    }
}

/// What a delegated Session produced.
pub type DelegationCompletion = Result<String, CoordinatorError>;

/// One entry in the registry: the record, the way to stop it, and the way to
/// wait for it.
struct Entry {
    record: DelegationRecord,
    signal: CancellationToken,
    completion: Option<oneshot::Receiver<DelegationCompletion>>,
}

/// The delegations one coordinator is responsible for, keyed by Work id.
///
/// Upstream's `delegatedWorkRuns` Map (`acp-backend-adapter.mjs:209`).
///
/// # Why a mutex rather than an owning task
///
/// `docs/architecture.md` §11 asks for an owning task where **order** is the
/// contract. This is a lookup table: every operation is a single insert,
/// remove or field write with no await inside the critical section, and no
/// caller can observe two of them out of order because each is one map key.
/// The three things here that *are* ordering invariants — the coordinator lane,
/// the target lane and the permission ledger — each have their own owning task
/// in [`crate::executor`] and [`crate::permission`].
#[derive(Clone, Default)]
pub struct DelegationRegistry {
    inner: Arc<Mutex<IndexMap<String, Entry>>>,
}

impl std::fmt::Debug for DelegationRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DelegationRegistry")
            .field("len", &self.len())
            .finish()
    }
}

/// Lock, treating poisoning as "the table is still there".
///
/// A poisoned lock means some other task panicked while holding it. Refusing to
/// read the table because of that would turn one panic into a delegation that
/// can never be cancelled — the same call [`via_work`] records for its abort
/// reason.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

impl DelegationRegistry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// How many delegations are live.
    #[must_use]
    pub fn len(&self) -> usize {
        lock(&self.inner).len()
    }

    /// Whether any delegation is live.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        lock(&self.inner).is_empty()
    }

    /// Register a delegation for `record.work_id`.
    ///
    /// # Errors
    ///
    /// [`CoordinatorError::DelegationAlreadyStarted`] when that Work already
    /// has one — upstream's `if (run.delegation) throw`.
    pub fn start(
        &self,
        record: DelegationRecord,
        signal: CancellationToken,
        completion: oneshot::Receiver<DelegationCompletion>,
    ) -> Result<(), CoordinatorError> {
        let mut runs = lock(&self.inner);
        if runs.contains_key(&record.work_id) {
            return Err(CoordinatorError::DelegationAlreadyStarted);
        }
        runs.insert(
            record.work_id.clone(),
            Entry {
                record,
                signal,
                completion: Some(completion),
            },
        );
        Ok(())
    }

    /// The delegation registered for `work_id`.
    #[must_use]
    pub fn snapshot(&self, work_id: &str) -> Option<DelegationRecord> {
        lock(&self.inner)
            .get(clean(work_id))
            .map(|entry| entry.record.clone())
    }

    /// The delegation registered for `work_id`, **if it is this owner's**.
    ///
    /// The only lookup the cancel and query paths use. Another owner's
    /// delegation answers `None`, which becomes a not-found refusal rather than
    /// a forbidden one.
    #[must_use]
    pub fn for_owner(&self, work_id: &str, owner_id: &str) -> Option<DelegationRecord> {
        let owner = clean(owner_id);
        self.snapshot(work_id)
            .filter(|record| record.owner_id == owner)
    }

    /// The delegation `input` names, by either id.
    ///
    /// **External contract** — `findDelegation`,
    /// `acp-backend-adapter.mjs:729-736`. Matching is
    /// [`via_mcp_tools::DelegationLookupInput::matches`], which is where the
    /// *"a lookup that names neither matches nothing"* rule already lives.
    #[must_use]
    pub fn find(&self, input: &DelegationLookupInput) -> Option<DelegationRecord> {
        lock(&self.inner)
            .values()
            .map(|entry| &entry.record)
            .find(|record| input.matches(&record.identity()))
            .cloned()
    }

    /// Take the completion channel for `work_id`.
    ///
    /// `None` the second time: exactly one caller may await a delegation, which
    /// is the runtime that started it.
    #[must_use]
    pub fn take_completion(
        &self,
        work_id: &str,
    ) -> Option<oneshot::Receiver<DelegationCompletion>> {
        lock(&self.inner)
            .get_mut(clean(work_id))
            .and_then(|entry| entry.completion.take())
    }

    /// The cancellation scope of the delegation `input` names.
    #[must_use]
    pub fn signal(&self, input: &DelegationLookupInput) -> Option<CancellationToken> {
        lock(&self.inner)
            .values()
            .find(|entry| input.matches(&entry.record.identity()))
            .map(|entry| entry.signal.clone())
    }

    /// Record that a delegation finished.
    ///
    /// Terminal states are sticky: a result arriving after a cancellation was
    /// confirmed does not un-cancel it, which is the same rule
    /// [`via_downstream::CancelOutcome`] enforces one layer down.
    pub fn settle(&self, work_id: &str, outcome: &DelegationCompletion, locale: via_i18n::Locale) {
        let mut runs = lock(&self.inner);
        let Some(entry) = runs.get_mut(clean(work_id)) else {
            return;
        };
        if entry.record.status.is_terminal() {
            return;
        }
        match outcome {
            Ok(content) => {
                entry.record.status = DelegationStatus::Completed;
                entry.record.result = Some(content.clone());
            }
            Err(error) if error.is_cancelled() || entry.signal.is_cancelled() => {
                entry.record.status = DelegationStatus::Cancelled;
                entry.record.error = Some(error.message(locale));
            }
            Err(error) => {
                entry.record.status = DelegationStatus::Failed;
                entry.record.error = Some(error.message(locale));
            }
        }
    }

    /// Ask a delegation to stop.
    ///
    /// **External contract** — `cancelDelegation`,
    /// `acp-backend-adapter.mjs:907-928`, with `docs/deviations/phase-2.md`'s
    /// recorded divergence: upstream writes `record.status = 'cancelled'`
    /// **before** it aborts anything and reports that; VIA writes `cancelling`
    /// and leaves the confirmation to whatever actually settles the runner.
    ///
    /// A delegation that is already terminal is left alone and its existing
    /// status is reported, exactly as upstream's
    /// `['completed','failed','cancelled'].includes(record.status)` short-circuit
    /// does.
    pub fn cancel(&self, input: &DelegationLookupInput, route: CancelRoute) -> CancelOutcome {
        let mut runs = lock(&self.inner);
        let Some(entry) = runs
            .values_mut()
            .find(|entry| input.matches(&entry.record.identity()))
        else {
            return CancelOutcome::NotFound;
        };
        let target = CancelTarget {
            delegation_id: Some(entry.record.id.clone()),
            session_id: Some(entry.record.session_id.clone()),
        };
        match entry.record.status {
            DelegationStatus::Completed => CancelOutcome::AlreadyFinished {
                target,
                state: TerminalState::Completed,
            },
            DelegationStatus::Failed => CancelOutcome::AlreadyFinished {
                target,
                state: TerminalState::Failed,
            },
            DelegationStatus::Cancelled => CancelOutcome::AlreadyFinished {
                target,
                state: TerminalState::Cancelled,
            },
            DelegationStatus::Running | DelegationStatus::Cancelling => {
                entry.record.status = DelegationStatus::Cancelling;
                entry.signal.cancel();
                CancelOutcome::requested(route, target)
            }
        }
    }

    /// The status answer for the delegation `input` names.
    ///
    /// [`SessionStatusResult::NotFound`] — which serializes **bare** — when
    /// nothing matches.
    #[must_use]
    pub fn status(&self, input: &DelegationLookupInput) -> SessionStatusResult {
        self.find(input)
            .map_or(SessionStatusResult::NotFound, |record| {
                record.status_result()
            })
    }

    /// Forget `work_id`'s delegation.
    ///
    /// Upstream's `finally { this.delegatedWorkRuns.delete(workId) }`
    /// (`acp-backend-adapter.mjs:1269`): the entry lives for exactly as long as
    /// the coordination run that owns it.
    pub fn remove(&self, work_id: &str) -> Option<DelegationRecord> {
        lock(&self.inner)
            .shift_remove(clean(work_id))
            .map(|entry| entry.record)
    }

    /// Cancel every live delegation.
    ///
    /// Upstream's `close()` (`acp-backend-adapter.mjs:1459-1463`), which aborts
    /// each controller with *"the backend is shutting down"*.
    pub fn cancel_all(&self) -> usize {
        let mut runs = lock(&self.inner);
        let mut cancelled = 0;
        for entry in runs.values_mut() {
            // Only `running` counts: a delegation already asked to stop has
            // been asked, and asking twice is not two cancellations.
            if entry.record.status == DelegationStatus::Running {
                entry.record.status = DelegationStatus::Cancelling;
                entry.signal.cancel();
                cancelled += 1;
            }
        }
        cancelled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    fn lookup(delegation_id: Option<&str>, session_id: Option<&str>) -> DelegationLookupInput {
        DelegationLookupInput {
            delegation_id: delegation_id.map(str::to_owned),
            session_id: session_id.map(str::to_owned),
        }
    }

    fn registered(registry: &DelegationRegistry) -> (DelegationRecord, CancellationToken) {
        let signal = CancellationToken::new();
        let (_settle, completion) = oneshot::channel();
        let record = DelegationRecord::new("opencode_run_1", "project-1", "owner-one", "work-one")
            .directory("/project")
            .title("Project", "build project", "OpenCode 项目任务");
        registry
            .start(record.clone(), signal.clone(), completion)
            .expect("first delegation");
        (record, signal)
    }

    #[test]
    fn a_delegation_id_names_its_backend_and_keeps_the_uuids_dashes() {
        let id = new_delegation_id("opencode");
        assert!(id.starts_with("opencode_run_"), "{id}");
        let uuid = id.strip_prefix("opencode_run_").expect("the prefix");
        assert_eq!(uuid.len(), 36, "a dashed UUID v4");
        assert_eq!(uuid.matches('-').count(), 4);
        assert_ne!(id, new_delegation_id("opencode"));
    }

    #[test]
    fn the_three_lane_keys_are_the_upstream_spellings() {
        assert_eq!(target_lane(" project-1 "), "target:project-1");
        assert_eq!(
            coordinator_session_lane("opencode:owner%20one:backend"),
            "coordinator:opencode:owner%20one:backend",
        );
        assert!(new_permission_scope_id().starts_with("prompt_"));
    }

    #[test]
    fn a_title_falls_back_to_the_prompt_and_then_to_the_profile() {
        let from_title = DelegationRecord::new("d", "s", "o", "w").title("T", "P", "F");
        assert_eq!(from_title.title, "T");
        let from_prompt = DelegationRecord::new("d", "s", "o", "w").title("", "P", "F");
        assert_eq!(from_prompt.title, "P");
        let from_profile = DelegationRecord::new("d", "s", "o", "w").title("", "   ", "F");
        assert_eq!(from_profile.title, "F");
        let long = DelegationRecord::new("d", "s", "o", "w").title(
            &"t".repeat(TITLE_BOUND + 40),
            "P",
            "F",
        );
        assert_eq!(long.title.chars().count(), TITLE_BOUND);
    }

    #[test]
    fn one_work_delegates_once() {
        let registry = DelegationRegistry::new();
        let (record, _) = registered(&registry);
        let (_settle, completion) = oneshot::channel();
        let error = registry
            .start(record, CancellationToken::new(), completion)
            .expect_err("a second delegation");
        assert!(matches!(error, CoordinatorError::DelegationAlreadyStarted));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn a_lookup_matches_by_either_id_and_neither_matches_nothing() {
        let registry = DelegationRegistry::new();
        registered(&registry);
        assert!(
            registry
                .find(&lookup(Some("opencode_run_1"), None))
                .is_some()
        );
        assert!(registry.find(&lookup(None, Some("project-1"))).is_some());
        assert!(registry.find(&lookup(Some("other"), None)).is_none());
        assert!(registry.find(&DelegationLookupInput::default()).is_none());
    }

    #[test]
    fn another_owners_delegation_is_not_found_rather_than_forbidden() {
        let registry = DelegationRegistry::new();
        registered(&registry);
        assert!(registry.for_owner("work-one", "owner-one").is_some());
        assert!(registry.for_owner("work-one", "owner-two").is_none());
        assert!(registry.for_owner("work-two", "owner-one").is_none());
    }

    #[test]
    fn only_the_first_caller_may_await_a_delegation() {
        let registry = DelegationRegistry::new();
        registered(&registry);
        assert!(registry.take_completion("work-one").is_some());
        assert!(registry.take_completion("work-one").is_none());
    }

    #[test]
    fn a_status_query_reports_result_only_when_completed() {
        let registry = DelegationRegistry::new();
        registered(&registry);
        let running = serde_json::to_value(registry.status(&lookup(Some("opencode_run_1"), None)))
            .expect("json");
        assert_eq!(running["status"], "running");
        assert!(running.get("result").is_none());

        registry.settle(
            "work-one",
            &Ok("  built it  ".to_owned()),
            via_i18n::Locale::En,
        );
        let completed =
            serde_json::to_value(registry.status(&lookup(None, Some("project-1")))).expect("json");
        assert_eq!(completed["status"], "completed");
        assert_eq!(completed["result"], "built it");
        assert!(completed.get("error").is_none());
    }

    #[test]
    fn a_failed_delegation_reports_its_message_and_nothing_else() {
        let registry = DelegationRegistry::new();
        registered(&registry);
        registry.settle(
            "work-one",
            &Err(CoordinatorError::NotConfigured),
            via_i18n::Locale::Zh,
        );
        let failed = serde_json::to_value(registry.status(&lookup(Some("opencode_run_1"), None)))
            .expect("json");
        assert_eq!(failed["status"], "failed");
        assert_eq!(failed["error"], "当前未配置后台 Agent");
        assert!(failed.get("result").is_none());
    }

    #[test]
    fn an_unknown_delegation_is_a_bare_not_found() {
        let registry = DelegationRegistry::new();
        assert_eq!(
            serde_json::to_value(registry.status(&lookup(Some("nope"), None))).expect("json"),
            json!({ "status": "not_found" }),
        );
        assert_eq!(
            registry.cancel(&lookup(Some("nope"), None), CancelRoute::Adapter),
            CancelOutcome::NotFound,
        );
    }

    #[test]
    fn a_cancel_is_requested_rather_than_claimed_and_aborts_the_scope() {
        let registry = DelegationRegistry::new();
        let (_, signal) = registered(&registry);
        let outcome = registry.cancel(&lookup(Some("opencode_run_1"), None), CancelRoute::Adapter);
        assert_eq!(outcome.status_str(), "cancelling");
        assert!(!outcome.is_confirmed());
        assert_eq!(outcome.route(), Some(CancelRoute::Adapter));
        assert!(
            signal.is_cancelled(),
            "the delegated prompt is asked to stop"
        );
        assert_eq!(
            registry.snapshot("work-one").map(|record| record.status),
            Some(DelegationStatus::Cancelling),
        );
    }

    #[test]
    fn terminal_work_is_left_alone_by_a_cancel() {
        let registry = DelegationRegistry::new();
        registered(&registry);
        registry.settle("work-one", &Ok("done".to_owned()), via_i18n::Locale::En);
        let outcome = registry.cancel(&lookup(Some("opencode_run_1"), None), CancelRoute::Adapter);
        assert!(matches!(
            outcome,
            CancelOutcome::AlreadyFinished {
                state: TerminalState::Completed,
                ..
            }
        ));
        assert!(
            !outcome.is_confirmed(),
            "completed work was never cancelled"
        );
    }

    #[test]
    fn a_result_that_arrives_after_a_confirmed_stop_does_not_un_cancel_it() {
        let registry = DelegationRegistry::new();
        registered(&registry);
        registry.settle(
            "work-one",
            &Err(CoordinatorError::Harness(
                via_downstream::HarnessError::Cancelled,
            )),
            via_i18n::Locale::En,
        );
        assert_eq!(
            registry.snapshot("work-one").map(|record| record.status),
            Some(DelegationStatus::Cancelled),
        );
        registry.settle("work-one", &Ok("late".to_owned()), via_i18n::Locale::En);
        assert_eq!(
            registry.snapshot("work-one").map(|record| record.status),
            Some(DelegationStatus::Cancelled),
        );
    }

    #[test]
    fn an_aborted_scope_makes_a_plain_failure_a_cancellation() {
        let registry = DelegationRegistry::new();
        let (_, signal) = registered(&registry);
        signal.cancel();
        registry.settle(
            "work-one",
            &Err(CoordinatorError::NotConfigured),
            via_i18n::Locale::En,
        );
        assert_eq!(
            registry.snapshot("work-one").map(|record| record.status),
            Some(DelegationStatus::Cancelled),
        );
    }

    #[test]
    fn the_persisted_shape_keeps_the_three_fields_a_client_never_sees() {
        let registry = DelegationRegistry::new();
        let (record, _) = registered(&registry);
        let reference = record.to_ref();
        assert_eq!(reference.id, "opencode_run_1");
        assert_eq!(reference.session_id, "project-1");
        assert_eq!(reference.directory.as_deref(), Some("/project"));
        assert_eq!(reference.status.as_deref(), Some("running"));

        let public = serde_json::to_string(&reference.to_public()).expect("json");
        assert!(!public.contains("opencode_run_1"), "{public}");
        assert!(!public.contains("project-1"), "{public}");
        assert!(!public.contains("/project"), "{public}");
    }

    #[test]
    fn shutdown_asks_every_live_delegation_to_stop() {
        let registry = DelegationRegistry::new();
        let (_, signal) = registered(&registry);
        assert_eq!(registry.cancel_all(), 1);
        assert!(signal.is_cancelled());
        assert_eq!(
            registry.cancel_all(),
            0,
            "a cancelling delegation is not re-cancelled"
        );
        assert!(!registry.is_empty());
        assert!(registry.remove("work-one").is_some());
        assert!(registry.is_empty());
    }

    #[test]
    fn the_started_answer_is_the_catalogued_shape() {
        let registry = DelegationRegistry::new();
        let (record, _) = registered(&registry);
        assert_eq!(
            serde_json::to_string(&record.started()).expect("json"),
            r#"{"status":"started","delegation_id":"opencode_run_1","session_id":"project-1","title":"Project","directory":"/project"}"#,
        );
    }
}
