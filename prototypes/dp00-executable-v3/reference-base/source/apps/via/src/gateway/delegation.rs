//! [`via_voice::tools::DelegationRunners`] over [`via_coordinator::Coordinator`].
//!
//! `docs/architecture.md` §9 gives the voice band
//! `voice → conversation, core, shared, task, voice` — **no Layer 3 at all**.
//! So `via-voice` declares what a `spawn_thinking` needs as a trait and
//! somebody above both bands supplies it. That somebody is this binary, which
//! is the only crate in the graph allowed to see Layer 1 and Layer 3 at once.
//!
//! # What a runner is, and what it is not
//!
//! A [`via_work::WorkRunner`] is *"carry out this objective and answer with
//! what to say"*. It does not own the Work's identity, its state, its lane or
//! its cancellation — [`via_work::WorkManager`] owns all four, and this module
//! calls into it rather than reproducing any of it. The whole of the runner is
//! [`Coordinator::run`] plus the translation of its two failure shapes into a
//! [`RunFailure`], whose message is spoken to the user verbatim.
//!
//! # The observer is the state machine
//!
//! Upstream's `onEvent({type: 'backend.delegated'})` is what moves a Work to
//! `delegated` and **releases its scheduler lane** — which is the whole reason
//! `docs/architecture.md` §4 says *"the delegated reply is not a completion"*.
//! The observer this module installs forwards those two events into the Work's own
//! [`via_work::WorkEventSink`], so the lane release is `via-work`'s decision
//! and not this module's.
//!
//! [`CoordinationObserver`] is **synchronous** and the sink is `async`, so the
//! observer writes into a bounded channel that one forwarding task drains in
//! order. A `tokio::spawn` per event would have reordered `delegated` and
//! `delegation.completed` under load, and correlation is exactly what
//! `docs/architecture.md` §11's fourth invariant protects.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use via_coordinator::{
    CoordinationObserver, CoordinationOutcome, CoordinationRequest, Coordinator, CoordinatorError,
    Delivery, TurnOptions,
};
use via_downstream::CancelOutcome;
use via_i18n::{Locale, keys, t};
use via_voice::tools::{DelegationRequest, DelegationRunners, StatusQueryRequest};
use via_work::{
    CancelRequest, DelegatedWorkRecovery, DelegationRef, RunFailure, RunnerEvent, WorkContext,
    WorkEventSink, WorkOutcome, WorkRunner, WorkSnapshot,
};

/// How many runner events may queue before the observer drops one.
///
/// A drop is only reachable when the forwarding task is behind by this many
/// events, and a Work raises exactly two — so the bound is generous for its
/// purpose and small enough that a wedged manager cannot grow it without limit.
pub const OBSERVER_QUEUE_DEPTH: usize = 32;

/// The runners a `spawn_thinking` and a delegated status query need.
///
/// One per Gateway, shared by every connection: the coordinator's fixed session
/// is per *owner*, not per socket
/// (`via_downstream::SessionKey::coordinator` — *"THE fixed identity that
/// survives voice sessions, Work IDs, and Gateway restarts"*), so a second
/// instance here would be a second conversation with the same backend.
#[derive(Clone)]
pub struct CoordinatorRunners {
    coordinator: Arc<Coordinator>,
    locale: Locale,
}

impl std::fmt::Debug for CoordinatorRunners {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoordinatorRunners")
            .field("backend", &self.coordinator.profile().label)
            .field("locale", &self.locale)
            .finish()
    }
}

impl CoordinatorRunners {
    /// Runners over `coordinator`.
    #[must_use]
    pub const fn new(coordinator: Arc<Coordinator>, locale: Locale) -> Self {
        Self {
            coordinator,
            locale,
        }
    }

    /// The coordinator underneath, for the permission relay and for shutdown.
    #[must_use]
    pub fn coordinator(&self) -> &Arc<Coordinator> {
        &self.coordinator
    }
}

impl DelegationRunners for CoordinatorRunners {
    fn delegation_runner(&self, request: &DelegationRequest) -> Arc<dyn WorkRunner> {
        Arc::new(DelegationRunner {
            coordinator: Arc::clone(&self.coordinator),
            locale: self.locale,
            original_request: request.objective.clone(),
        })
    }

    fn delegation_canceler(&self, _request: &DelegationRequest) -> Arc<dyn via_work::WorkCanceler> {
        Arc::new(CoordinatorCanceler {
            coordinator: Arc::clone(&self.coordinator),
        })
    }

    fn status_query_runner(&self, request: &StatusQueryRequest) -> Arc<dyn WorkRunner> {
        Arc::new(StatusQueryRunner {
            coordinator: Arc::clone(&self.coordinator),
            parent_work_id: request.parent_work_id.clone(),
            question: request.question.clone(),
        })
    }

    fn status_query_canceler(
        &self,
        _request: &StatusQueryRequest,
    ) -> Arc<dyn via_work::WorkCanceler> {
        Arc::new(CoordinatorCanceler {
            coordinator: Arc::clone(&self.coordinator),
        })
    }

    /// A scheduled `type: 'task'` runs through the same coordinator as a
    /// `spawn_thinking`, because it *is* one — the only difference is that a
    /// timer submitted it. `None` here would make a scheduled task speak its
    /// own objective back, which is the `reminder` behaviour.
    fn scheduled_task_runner(&self) -> Option<Arc<dyn WorkRunner>> {
        Some(Arc::new(DelegationRunner {
            coordinator: Arc::clone(&self.coordinator),
            locale: self.locale,
            original_request: String::new(),
        }))
    }
}

/// One delegation, from objective to spoken answer.
#[derive(Debug)]
struct DelegationRunner {
    coordinator: Arc<Coordinator>,
    locale: Locale,
    /// The user's own words, when the tool call carried them.
    ///
    /// `input.final_asr` and `input.objective` are *different fields* in the
    /// envelope: the first is what the user said, the second the frontend's
    /// conservative summary. A scheduled task has no utterance, so it sends the
    /// objective as both — which is what upstream does when `finalAsr` is
    /// absent.
    original_request: String,
}

#[async_trait]
impl WorkRunner for DelegationRunner {
    async fn run(
        &self,
        objective: String,
        context: WorkContext,
    ) -> Result<WorkOutcome, RunFailure> {
        let original = if self.original_request.is_empty() {
            objective.as_str()
        } else {
            self.original_request.as_str()
        };
        let turn_id = context.turn_id.clone().unwrap_or_default();
        let request = CoordinationRequest {
            original_request: original,
            objective: &objective,
            coordination_run_id: &context.work_id,
            voice_session_id: &context.session_id,
            turn_id: &turn_id,
            // A `spawn_thinking` exists because a voice client asked for it, so
            // a voice client is listening. `allow_status` stays at upstream's
            // default: interim status is not welcome unless it is meaningful.
            delivery: Delivery::default(),
            ..CoordinationRequest::default()
        };

        let (observer, forwarding) = SinkObserver::install(context.events.clone());
        let options = TurnOptions::new(&context.owner_id, &context.work_id)
            .with_observer(observer)
            .with_signal(context.signal.token().clone());

        // **The runner owns the race, not the coordinator.**
        // `Coordinator::turn` awaits `session.prompt` without watching the
        // signal, because a *delegated* stop is the backend's to confirm —
        // `docs/architecture.md` §4, *"cancellation is confirmed, not
        // optimistic"*. A coordinator turn that has not delegated yet has
        // nobody to ask, and upstream's task manager aborts it locally
        // (`task-manager.mjs:727-733`: *"`delegated` goes to the coordinator,
        // everything else aborts locally"*). This is that local abort, run
        // against the Work's **own** cancellation scope — which
        // `docs/architecture.md` §11 requires each Work to have precisely so
        // that one barge-in cannot kill independent background work.
        let outcome = tokio::select! {
            outcome = self.coordinator.run(&request, Utc::now(), &options) => outcome,
            () = context.signal.aborted() => Err(CoordinatorError::Harness(
                via_downstream::HarnessError::Cancelled,
            )),
        };
        // Stop the forwarder before answering: an event that arrives after the
        // Work is terminal is a mutation against a record that has moved on.
        forwarding.abort();

        outcome
            .map(via_coordinator::CoordinationOutcome::into_work_outcome)
            .map_err(|error| self.failure(&error))
    }
}

impl DelegationRunner {
    /// A coordinator refusal, as the sentence the Work carries.
    ///
    /// [`RunFailure`] documents its own message as *"already-localized … it
    /// becomes `task.error` verbatim, is persisted, and is spoken to the
    /// user"*. [`CoordinatorError`]'s `Display` is developer English for the
    /// variants that describe a protocol fault, so those are replaced by the
    /// catalogued sentence and the rest are passed through — the same rule
    /// `crate::error::localize_core` follows for `via-core`.
    fn failure(&self, error: &CoordinatorError) -> RunFailure {
        let message = match error {
            CoordinatorError::NotFinalResult { .. } => {
                t(self.locale, keys::VOICE_ERROR_SUBMIT_FAILED).to_owned()
            }
            other => other.to_string(),
        };
        RunFailure { message }
    }
}

/// A delegated status query — `coordinator.query_delegated_work`.
#[derive(Debug)]
struct StatusQueryRunner {
    coordinator: Arc<Coordinator>,
    parent_work_id: String,
    question: String,
}

#[async_trait]
impl WorkRunner for StatusQueryRunner {
    async fn run(
        &self,
        _objective: String,
        context: WorkContext,
    ) -> Result<WorkOutcome, RunFailure> {
        self.coordinator
            .query_delegated_work(&self.parent_work_id, &self.question, &context.owner_id)
            .await
            .map(via_coordinator::CoordinationOutcome::into_work_outcome)
            .map_err(|error| RunFailure {
                message: error.to_string(),
            })
    }
}

/// Stopping a delegation, confirmed rather than optimistic.
///
/// `docs/architecture.md` §4: *"Cancellation is confirmed, not optimistic.
/// Work stays `cancelling` until a path confirms the stop."* That decision is
/// [`Coordinator::cancel_delegated_work`]'s — it asks the model when the
/// coordinator session is idle and goes straight down the transport when it is
/// not — and the outcome is returned unchanged so `via-work` can tell
/// `Requested` from `Confirmed`.
#[derive(Debug)]
struct CoordinatorCanceler {
    coordinator: Arc<Coordinator>,
}

#[async_trait]
impl via_work::WorkCanceler for CoordinatorCanceler {
    async fn cancel(&self, request: CancelRequest) -> Result<CancelOutcome, RunFailure> {
        // **The abort comes first, and it comes in both branches.** A canceler
        // that only *reports* leaves the runner running: `via-work`'s
        // no-canceler path raises the abort itself — *"No canceler: abort
        // directly, and let the runner settling be the confirmation"* — so a
        // canceler that is installed inherits that responsibility. Skipping it
        // makes `DELETE /api/tasks/:id` never answer, because the reply waits
        // on the *confirmed* stop rather than on the request.
        request.abort();

        // Everything that is not `delegated` stops there: there is no backend
        // session to ask, and the abort is what settles the runner.
        if request.previous_status != via_protocol::WorkStatus::Delegated {
            return Ok(CancelOutcome::requested(
                via_downstream::CancelRoute::Adapter,
                via_downstream::CancelTarget::default(),
            ));
        }
        self.coordinator
            .cancel_delegated_work(
                &request.work.work.id,
                &request.work.work.owner_id,
                &via_lock::iso8601(Utc::now()),
            )
            .await
            .map_err(|error| RunFailure {
                message: error.to_string(),
            })
    }
}

/// A [`CoordinationObserver`] that forwards into a Work's event sink.
#[derive(Debug)]
struct SinkObserver {
    events: tokio::sync::mpsc::Sender<RunnerEvent>,
}

impl SinkObserver {
    /// An observer and the queue it writes into.
    fn new() -> (Self, tokio::sync::mpsc::Receiver<RunnerEvent>) {
        let (events, inbox) = tokio::sync::mpsc::channel(OBSERVER_QUEUE_DEPTH);
        (Self { events }, inbox)
    }

    /// Start the forwarding task and hand back the observer.
    ///
    /// The returned handle must be aborted once the turn is over — see the
    /// module documentation.
    fn install(
        sink: WorkEventSink,
    ) -> (Arc<dyn CoordinationObserver>, tokio::task::JoinHandle<()>) {
        let (observer, mut inbox) = Self::new();
        let forwarding = tokio::spawn(async move {
            while let Some(event) = inbox.recv().await {
                if !sink.emit(event).await {
                    break;
                }
            }
        });
        (Arc::new(observer), forwarding)
    }
}

impl CoordinationObserver for SinkObserver {
    fn delegated(&self, delegation: &DelegationRef) {
        let _ = self
            .events
            .try_send(RunnerEvent::Delegated(delegation.clone()));
    }

    fn delegation_completed(&self, delegation: &DelegationRef) {
        let _ = self
            .events
            .try_send(RunnerEvent::DelegationCompleted(delegation.clone()));
    }
}

/// [`DelegatedWorkRecovery`] over [`Coordinator::recover_native_delegation`].
///
/// This is what `WorkManager::recover_delegated`'s documented "call once, at
/// composition" is calling — wired in `apps/via/src/gateway/compose.rs`
/// beside the coordinator itself. Without it, a `delegated` or `finalizing`
/// Work that survives a restart with both delegation ids is restored `queued`
/// and left in `via_work`'s `recovery_candidates` forever
/// (`docs/deviations/phase-9-via-e2e.md` #4); with it, every candidate is
/// actually offered to [`can_recover`](DelegatedWorkRecovery::can_recover),
/// and one of the two catalogued restart sentences is reachable either way.
///
/// [`can_recover`] is [`Coordinator::native_delegation_configured`]. VIA's own
/// composition wires no [`via_coordinator::NativeDelegationAdapter`] (native,
/// backend-detected delegation is not part of the shipped Gateway — only the
/// `spawn_thinking` MCP-tool delegation is), so every candidate is declined
/// today and reaches the *unrecoverable delegated work* restart sentence
/// rather than being stranded `queued`. The reattachment path itself
/// ([`run`](DelegatedWorkRecovery::run)) is real, not a stub: if a future
/// composition roots a native adapter, recovery works without touching this
/// type again.
#[derive(Clone)]
pub struct CoordinatorRecovery {
    coordinator: Arc<Coordinator>,
    locale: Locale,
}

impl std::fmt::Debug for CoordinatorRecovery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoordinatorRecovery")
            .field(
                "native_delegation_configured",
                &self.coordinator.native_delegation_configured(),
            )
            .field("locale", &self.locale)
            .finish_non_exhaustive()
    }
}

impl CoordinatorRecovery {
    /// Recovery over `coordinator`.
    #[must_use]
    pub const fn new(coordinator: Arc<Coordinator>, locale: Locale) -> Self {
        Self {
            coordinator,
            locale,
        }
    }
}

#[async_trait]
impl DelegatedWorkRecovery for CoordinatorRecovery {
    fn can_recover(&self, work: &WorkSnapshot) -> bool {
        self.coordinator.native_delegation_configured() && work.is_recoverable()
    }

    async fn run(
        &self,
        work: WorkSnapshot,
        context: WorkContext,
    ) -> Result<WorkOutcome, RunFailure> {
        let Some(delegation) = work.delegation.as_ref() else {
            // Unreachable through `via_work::WorkManager::recover_delegated`:
            // it only calls `run` for a candidate `can_recover` just accepted,
            // and `can_recover` requires `work.is_recoverable()`, which
            // requires a delegation. Kept as a real refusal rather than a
            // panic because a trait method cannot enforce that at compile
            // time.
            return Err(RunFailure {
                message: CoordinatorError::NotRecoverable {
                    label: self.coordinator.profile().label.clone(),
                }
                .message(self.locale),
            });
        };
        // The same relay `DelegationRunner::run` installs: `finish_delegation`
        // announces `delegated`/`delegation_completed` through
        // `options.observer`, and those two calls are what move the restored
        // Work through `WorkStatus::Delegated`/`Finalizing` again —
        // `via-work`'s own transitions, not a fact this type may assume.
        // Skipping this would still let the Work reach `completed`
        // (`Running -> Completed` is a legal edge on its own), but silently,
        // with neither `task.delegated` nor `task.finalizing` ever emitted.
        let (observer, forwarding) = SinkObserver::install(context.events.clone());
        let options = TurnOptions::new(&context.owner_id, &context.work_id)
            .with_observer(observer)
            .with_signal(context.signal.token().clone());
        let outcome = self
            .coordinator
            .recover_native_delegation(delegation, &options)
            .await;
        forwarding.abort();
        outcome
            .map(CoordinationOutcome::into_work_outcome)
            .map_err(|error| RunFailure {
                message: error.message(self.locale),
            })
    }

    async fn cancel(
        &self,
        work: WorkSnapshot,
        request: CancelRequest,
    ) -> Result<CancelOutcome, RunFailure> {
        // Same rule as `CoordinatorCanceler`: the abort comes first, in both
        // branches, because it is what settles the runner when nothing else
        // does.
        request.abort();
        self.coordinator
            .cancel_delegated_work(
                &work.work.id,
                &work.work.owner_id,
                &via_lock::iso8601(Utc::now()),
            )
            .await
            .map_err(|error| RunFailure {
                message: error.message(self.locale),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn the_observer_queue_is_bounded() {
        assert_eq!(OBSERVER_QUEUE_DEPTH, 32);
    }

    #[tokio::test]
    async fn the_two_delegation_events_queue_in_the_order_they_were_raised() {
        let (observer, mut inbox) = SinkObserver::new();
        let reference = DelegationRef::new("delegation_1", "session_1");
        observer.delegated(&reference);
        observer.delegation_completed(&reference);
        assert!(matches!(
            inbox.recv().await,
            Some(RunnerEvent::Delegated(_))
        ));
        assert!(matches!(
            inbox.recv().await,
            Some(RunnerEvent::DelegationCompleted(_))
        ));
    }

    #[tokio::test]
    async fn a_full_queue_drops_rather_than_blocking_the_coordinator() {
        let (observer, _inbox) = SinkObserver::new();
        let reference = DelegationRef::new("delegation_1", "session_1");
        for _ in 0..(OBSERVER_QUEUE_DEPTH * 2) {
            // `try_send` never awaits, so a coordinator turn cannot be stalled
            // by a Work manager that stopped reading.
            observer.delegated(&reference);
        }
    }

    /// The end-to-end wiring `via-coordinator`'s own tests cannot see: that
    /// [`CoordinatorRecovery::run`] actually installs an observer, so
    /// [`via_coordinator::Coordinator::recover_native_delegation`]'s
    /// `delegated`/`delegation_completed` announcements reach a **real**
    /// `via_work::WorkManager` as `task.delegated`/`task.finalizing`, exactly
    /// as a live-detected native delegation would.
    ///
    /// Mutation-checked: removing `.with_observer(observer)` from
    /// `CoordinatorRecovery::run` (restoring the bug this test was written
    /// against) drops `task.delegated`/`task.finalizing` from the sequence
    /// below while the Work still completes — a regression this test catches
    /// and the two-`assert_eq!` sentence-and-registry tests elsewhere in this
    /// codebase cannot, because none of them drive `CoordinatorRecovery`
    /// through a real `WorkManager`.
    #[tokio::test(start_paused = true)]
    async fn recovering_a_delegation_through_a_real_work_manager_announces_both_events() {
        use via_coordinator::testing::ScriptedNativeDelegation;
        use via_downstream::DownstreamAgent;
        use via_downstream::testing::{ScriptedHarness, ScriptedTurn};
        use via_protocol::WorkStatus;
        use via_work::{WorkEventKind, WorkManager, WorkStore};

        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("tasks.json");
        let store = WorkStore::builder().file_path(&path).build();
        let record: via_work::PersistedWork = serde_json::from_value(serde_json::json!({
            "id": "work-delegated",
            "status": "delegated",
            "objective": "继续项目",
            "ownerId": "owner-one",
            "delegation": {
                "id": "run-one",
                "sessionId": "agent:child:one",
                "directory": "/project",
                "title": "项目任务",
            },
        }))
        .expect("a valid persisted record");
        assert!(store.save(&[record]), "seeds the restart fixture");

        // A fresh manager over the same file is the restart: `restore()` reads
        // `work-delegated` back as `queued`, addressable, and a recovery
        // candidate — via-work's own concern, already covered by
        // `via-work/tests/restart.rs`. What is new here is what happens next.
        let work = WorkManager::builder()
            .store(WorkStore::builder().file_path(&path).build())
            .build();
        let mut events = work.subscribe();

        let harness = ScriptedHarness::builder("opencode")
            .turn(ScriptedTurn::completed(
                &serde_json::json!({
                    "work_id": "work-delegated",
                    "state": "completed",
                    "mode": "respond",
                    "presentation": { "speech": "恢复后的结果", "inline": null },
                })
                .to_string(),
            ))
            .build()
            .expect("opencode is catalogued");
        let coordinator = Arc::new(
            Coordinator::builder(
                Arc::new(harness) as Arc<dyn DownstreamAgent>,
                via_coordinator::CoordinatorProfile::default_for(
                    "opencode",
                    "OpenCode",
                    Locale::Zh,
                ),
            )
            .locale(Locale::Zh)
            .native_delegation(ScriptedNativeDelegation::completing("原生结果") as _)
            .build(),
        );
        let recovery = Arc::new(CoordinatorRecovery::new(coordinator, Locale::Zh));

        let recovered = work.recover_delegated(recovery).await;
        assert_eq!(recovered, 1);

        let finished = work.wait("work-delegated").await.expect("terminal");
        assert_eq!(finished.status, WorkStatus::Completed);
        assert_eq!(finished.result.as_deref(), Some("恢复后的结果"));

        let mut kinds = Vec::new();
        while let Ok(event) = events.try_recv() {
            if event.task.id == "work-delegated" {
                kinds.push(event.kind);
            }
        }
        assert!(
            kinds.contains(&WorkEventKind::Delegated),
            "the recovered Work must announce `task.delegated`, not just complete \
             silently: {kinds:?}",
        );
        assert!(
            kinds.contains(&WorkEventKind::Finalizing),
            "and `task.finalizing` once the native delegation answers: {kinds:?}",
        );
        assert!(kinds.contains(&WorkEventKind::Completed), "{kinds:?}");
        let delegated_at = kinds
            .iter()
            .position(|kind| *kind == WorkEventKind::Delegated)
            .expect("just asserted present");
        let finalizing_at = kinds
            .iter()
            .position(|kind| *kind == WorkEventKind::Finalizing)
            .expect("just asserted present");
        let completed_at = kinds
            .iter()
            .position(|kind| *kind == WorkEventKind::Completed)
            .expect("just asserted present");
        assert!(
            delegated_at < finalizing_at && finalizing_at < completed_at,
            "the three must land in lifecycle order: {kinds:?}",
        );
    }
}
