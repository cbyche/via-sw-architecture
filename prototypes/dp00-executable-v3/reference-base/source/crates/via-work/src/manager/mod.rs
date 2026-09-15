//! [`WorkManager`] — the authority for `work_id` and the nine-state lifecycle.
//!
//! `server/src/task/task-manager.mjs`. Every method here is one crate-private
//! `Command` sent to a single owning task, which holds the whole Work table by
//! value.
//!
//! # Why an owning task and not a mutex
//!
//! `docs/architecture.md` §11: *"`tokio::sync::Mutex` gives mutual exclusion
//! but not FIFO order, and three of the four need order."* The Work queue is
//! the first of the four invariants, and the order **is** the contract: a
//! coordinator lane that admitted B before A would still be exclusive and still
//! be wrong. One task, one bounded channel, everything in arrival order —
//! including the events a runner raises and the ticks a timer raises, which is
//! what makes "the activity was recorded before the lane was released"
//! something the type system can be responsible for rather than a review note.
//!
//! Shutdown is dropping the sender: the actor drains what is queued, flushes
//! `tasks.json` and exits.
//!
//! # What the manager does not know
//!
//! No harness, no session, no permission decision, no prompt. It runs
//! [`WorkRunner`]s. That is why this crate depends on `via-downstream` for two
//! vocabularies and on no ACP crate at all.

mod actor;

use std::sync::Arc;

use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_util::task::TaskTracker;
use via_core::Config;
use via_i18n::Locale;
use via_log::Logger;
use via_protocol::WorkKind;
use via_store::StoreHealth;

use crate::clock::{NowFn, system_clock};
use crate::command::Command;
use crate::error::ManagerStopped;
use crate::event::WorkEvent;
use crate::limits::RetentionPolicy;
use crate::progress::DEFAULT_PROGRESS_CHECK_MS;
use crate::reconcile::CancellationFact;
use crate::record::{DEFAULT_SESSION_ID, PublicWork};
use crate::runner::{CoordinatorQuery, DelegatedWorkRecovery, WorkCanceler, WorkRunner};
use crate::schedule::Schedule;
use crate::scheduler::{
    COORDINATOR_LANE_LIMIT, DEFAULT_MAX_CONCURRENT, DEFAULT_MAX_CONCURRENT_PER_OWNER,
};
use crate::store::WorkStore;

pub use actor::reminder_runner;

/// How many commands may be in flight before a sender waits.
///
/// Bounded, per `docs/architecture.md` §11. Large enough that a burst of
/// activity events from four concurrent runners never blocks one of them behind
/// the other three, small enough that a wedged actor is felt rather than
/// silently buffering.
pub const DEFAULT_COMMAND_CAPACITY: usize = 256;

/// How many events one subscriber may fall behind before it is told it lagged.
///
/// A `task.progress` per active Work per second, plus the lifecycle events, so
/// a subscriber that stalls for a minute across four Work items still catches
/// up. Upstream's listener set is synchronous and cannot lag; a broadcast can,
/// and reports it rather than dropping silently.
pub const DEFAULT_EVENT_CAPACITY: usize = 512;

/// A request to create a Work.
///
/// `taskManager.create({...})` — `server/src/task/task-manager.mjs:335-348`.
pub struct NewWork {
    /// The user's request. Trimmed on the way in.
    pub objective: String,
    /// Whose Work this is.
    pub owner_id: String,
    /// Which session raised it. [`DEFAULT_SESSION_ID`] when `None`.
    pub session_id: Option<String>,
    /// Which turn raised it.
    pub turn_id: Option<String>,
    /// The duplicate-submission key.
    ///
    /// A non-empty key that another Work of the same owner already carries
    /// makes this a no-op: the existing Work is returned with
    /// [`WorkAcceptance::reused`] set. Trimmed; an empty key is no key.
    pub submission_key: Option<String>,
    /// The serialization lane. `coordinator:<owner>` for everything a voice
    /// session submits.
    pub lane_key: Option<String>,
    /// How wide that lane is. One, for the coordinator lane.
    pub lane_limit: usize,
    /// Which of the four kinds.
    pub kind: WorkKind,
    /// The Work this one serves, for a `control` query.
    pub parent_work_id: Option<String>,
    /// Higher runs first. A `control` status query is submitted at 100 so it
    /// overtakes ordinary queued Work in the same lane.
    pub priority: i64,
    /// What runs it. Falls back to the manager's default runner.
    pub runner: Option<Arc<dyn WorkRunner>>,
    /// What stops it. Without one, cancellation aborts the runner directly.
    pub canceler: Option<Arc<dyn WorkCanceler>>,
}

impl std::fmt::Debug for NewWork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NewWork")
            .field("objective", &self.objective)
            .field("owner_id", &self.owner_id)
            .field("session_id", &self.session_id)
            .field("kind", &self.kind)
            .field("priority", &self.priority)
            .field("lane_key", &self.lane_key)
            .field("submission_key", &self.submission_key)
            .finish_non_exhaustive()
    }
}

impl NewWork {
    /// A `work`-kind request with no lane and no runner of its own.
    #[must_use]
    pub fn new(objective: &str, owner_id: &str) -> Self {
        Self {
            objective: objective.to_owned(),
            owner_id: owner_id.to_owned(),
            session_id: None,
            turn_id: None,
            submission_key: None,
            lane_key: None,
            lane_limit: COORDINATOR_LANE_LIMIT,
            kind: WorkKind::Work,
            parent_work_id: None,
            priority: 0,
            runner: None,
            canceler: None,
        }
    }

    /// Set the session.
    #[must_use]
    pub fn session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_owned());
        self
    }

    /// Set the turn.
    #[must_use]
    pub fn turn(mut self, turn_id: &str) -> Self {
        self.turn_id = Some(turn_id.to_owned());
        self
    }

    /// Set the duplicate-submission key.
    #[must_use]
    pub fn submission_key(mut self, key: &str) -> Self {
        self.submission_key = Some(key.to_owned());
        self
    }

    /// Put this Work in a lane.
    #[must_use]
    pub fn lane(mut self, lane_key: &str, lane_limit: usize) -> Self {
        self.lane_key = Some(lane_key.to_owned());
        self.lane_limit = lane_limit;
        self
    }

    /// Set the kind.
    #[must_use]
    pub const fn kind(mut self, kind: WorkKind) -> Self {
        self.kind = kind;
        self
    }

    /// Name the Work this one serves.
    #[must_use]
    pub fn parent(mut self, parent_work_id: &str) -> Self {
        self.parent_work_id = Some(parent_work_id.to_owned());
        self
    }

    /// Set the scheduler priority.
    #[must_use]
    pub const fn priority(mut self, priority: i64) -> Self {
        self.priority = priority;
        self
    }

    /// Set the runner.
    #[must_use]
    pub fn runner(mut self, runner: Arc<dyn WorkRunner>) -> Self {
        self.runner = Some(runner);
        self
    }

    /// Set the canceler.
    #[must_use]
    pub fn canceler(mut self, canceler: Arc<dyn WorkCanceler>) -> Self {
        self.canceler = Some(canceler);
        self
    }
}

/// Which of the two scheduled kinds a request is for.
///
/// `type === 'task' ? 'scheduled_task' : 'reminder'`
/// (`server/src/task/task-manager.mjs:414`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledKind {
    /// A reminder: its runner only speaks the stored text back.
    #[default]
    Reminder,
    /// Real work with a due time — the only kind that gets a timeout watchdog.
    Task,
}

impl ScheduledKind {
    /// The [`WorkKind`] this becomes.
    #[must_use]
    pub const fn work_kind(self) -> WorkKind {
        match self {
            Self::Reminder => WorkKind::Reminder,
            Self::Task => WorkKind::ScheduledTask,
        }
    }
}

/// A request to schedule a Work.
///
/// `taskManager.createScheduled({...})` —
/// `server/src/task/task-manager.mjs:404-413`.
pub struct NewScheduledWork {
    /// What to do, or what to say.
    pub objective: String,
    /// Whose Work this is.
    pub owner_id: String,
    /// Which session raised it.
    pub session_id: Option<String>,
    /// Which turn raised it.
    pub turn_id: Option<String>,
    /// When it is due.
    pub schedule: Schedule,
    /// Reminder or task.
    pub kind: ScheduledKind,
    /// The wall-clock budget, for a task. Falls back to the manager's
    /// configured `scheduled_task_timeout_ms`.
    pub timeout_ms: Option<i64>,
    /// What runs it. A reminder with no runner gets
    /// [`reminder_runner`], which speaks the objective back.
    pub runner: Option<Arc<dyn WorkRunner>>,
}

impl std::fmt::Debug for NewScheduledWork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NewScheduledWork")
            .field("objective", &self.objective)
            .field("owner_id", &self.owner_id)
            .field("schedule", &self.schedule)
            .field("kind", &self.kind)
            .field("timeout_ms", &self.timeout_ms)
            .finish_non_exhaustive()
    }
}

impl NewScheduledWork {
    /// A reminder due at `schedule`.
    #[must_use]
    pub fn reminder(objective: &str, owner_id: &str, schedule: Schedule) -> Self {
        Self {
            objective: objective.to_owned(),
            owner_id: owner_id.to_owned(),
            session_id: None,
            turn_id: None,
            schedule,
            kind: ScheduledKind::Reminder,
            timeout_ms: None,
            runner: None,
        }
    }

    /// A scheduled task due at `schedule`.
    #[must_use]
    pub fn task(objective: &str, owner_id: &str, schedule: Schedule) -> Self {
        Self {
            kind: ScheduledKind::Task,
            ..Self::reminder(objective, owner_id, schedule)
        }
    }

    /// Set the session.
    #[must_use]
    pub fn session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_owned());
        self
    }

    /// Set the turn.
    #[must_use]
    pub fn turn(mut self, turn_id: &str) -> Self {
        self.turn_id = Some(turn_id.to_owned());
        self
    }

    /// Set the runner.
    #[must_use]
    pub fn runner(mut self, runner: Arc<dyn WorkRunner>) -> Self {
        self.runner = Some(runner);
        self
    }

    /// Override the wall-clock budget.
    #[must_use]
    pub const fn timeout_ms(mut self, timeout_ms: i64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
}

/// What `create` and `create_scheduled` answer.
///
/// `{ ...publicTask(task), reused }` — `server/src/task/task-manager.mjs:356,401`.
/// `reused: true` means the submission key matched something that already
/// exists and **nothing new was started**; the voice tool turns that into
/// `{"status":"duplicate", …}`.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkAcceptance {
    /// The Work — the existing one, when `reused`.
    pub work: PublicWork,
    /// Whether this is an existing Work rather than a new one.
    pub reused: bool,
}

/// Which Work to list.
///
/// `taskManager.list({...})` — `server/src/task/task-manager.mjs:808-824`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkQuery {
    /// Restrict to one owner.
    pub owner_id: Option<String>,
    /// Restrict to one session.
    pub session_id: Option<String>,
    /// Only the five active statuses.
    pub active_only: bool,
    /// Include `control` Work, which is hidden by default.
    pub include_control: bool,
}

impl WorkQuery {
    /// Everything belonging to `owner_id`.
    #[must_use]
    pub fn owner(owner_id: &str) -> Self {
        Self {
            owner_id: Some(owner_id.to_owned()),
            ..Self::default()
        }
    }

    /// Restrict to one session.
    #[must_use]
    pub fn session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_owned());
        self
    }

    /// Only active Work.
    #[must_use]
    pub const fn active(mut self) -> Self {
        self.active_only = true;
        self
    }

    /// Include `control` Work.
    #[must_use]
    pub const fn with_control(mut self) -> Self {
        self.include_control = true;
        self
    }
}

/// A request to claim terminal notifications for delivery.
///
/// `taskManager.claimNotifications({...})` —
/// `server/src/task/task-manager.mjs:831-859`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationClaim {
    /// Whose notifications.
    pub owner_id: String,
    /// Which session is asking. `None` claims across every session.
    pub session_id: Option<String>,
    /// Whether a session may claim another session's notifications.
    ///
    /// A reconnecting client sets this so results raised by a voice session
    /// that has since gone away are still spoken.
    pub include_other_sessions: bool,
    /// Who is claiming — the lease holder.
    pub claimant_id: String,
    /// Restrict to these Work ids.
    pub work_ids: Option<Vec<String>>,
}

impl NotificationClaim {
    /// A claim by `claimant_id` for `owner_id`'s notifications in
    /// `session_id`.
    #[must_use]
    pub fn new(owner_id: &str, session_id: &str, claimant_id: &str) -> Self {
        Self {
            owner_id: owner_id.to_owned(),
            session_id: Some(session_id.to_owned()),
            include_other_sessions: false,
            claimant_id: claimant_id.to_owned(),
            work_ids: None,
        }
    }

    /// Also claim notifications raised by other sessions of the same owner.
    #[must_use]
    pub const fn across_sessions(mut self) -> Self {
        self.include_other_sessions = true;
        self
    }

    /// Restrict the claim to these Work ids.
    #[must_use]
    pub fn only(mut self, work_ids: Vec<String>) -> Self {
        self.work_ids = Some(work_ids);
        self
    }
}

/// A scheduled Work, as the reminder scheduler sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledWork {
    /// The Work id.
    pub work_id: String,
    /// Whose Work it is.
    pub owner_id: String,
    /// When it is due, in epoch milliseconds.
    pub at: i64,
}

/// Builds a [`WorkManager`].
pub struct WorkManagerBuilder {
    store: Option<WorkStore>,
    runner: Option<Arc<dyn WorkRunner>>,
    retention: RetentionPolicy,
    max_concurrent: i64,
    max_concurrent_per_owner: i64,
    progress_check_ms: i64,
    scheduled_task_timeout_ms: i64,
    locale: Locale,
    now: Option<NowFn>,
    logger: Option<Logger>,
    command_capacity: usize,
    event_capacity: usize,
    tracker: Option<TaskTracker>,
}

impl std::fmt::Debug for WorkManagerBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkManagerBuilder")
            .field("retention", &self.retention)
            .field("max_concurrent", &self.max_concurrent)
            .field("max_concurrent_per_owner", &self.max_concurrent_per_owner)
            .field("progress_check_ms", &self.progress_check_ms)
            .field("locale", &self.locale)
            .finish_non_exhaustive()
    }
}

impl Default for WorkManagerBuilder {
    fn default() -> Self {
        Self {
            store: None,
            runner: None,
            retention: RetentionPolicy::default(),
            max_concurrent: DEFAULT_MAX_CONCURRENT as i64,
            max_concurrent_per_owner: DEFAULT_MAX_CONCURRENT_PER_OWNER as i64,
            progress_check_ms: DEFAULT_PROGRESS_CHECK_MS,
            scheduled_task_timeout_ms: crate::limits::DEFAULT_SCHEDULED_TASK_TIMEOUT_MS,
            locale: Locale::default(),
            now: None,
            logger: None,
            command_capacity: DEFAULT_COMMAND_CAPACITY,
            event_capacity: DEFAULT_EVENT_CAPACITY,
            tracker: None,
        }
    }
}

impl WorkManagerBuilder {
    /// Where Work is persisted. Without one the manager is in-memory.
    #[must_use]
    pub fn store(mut self, store: WorkStore) -> Self {
        self.store = Some(store);
        self
    }

    /// The runner every Work uses unless it brings its own.
    #[must_use]
    pub fn runner(mut self, runner: Arc<dyn WorkRunner>) -> Self {
        self.runner = Some(runner);
        self
    }

    /// The retention policy. Clamped on the way in.
    #[must_use]
    pub const fn retention(mut self, retention: RetentionPolicy) -> Self {
        self.retention = retention;
        self
    }

    /// The two admission caps.
    #[must_use]
    pub const fn concurrency(mut self, max_concurrent: i64, per_owner: i64) -> Self {
        self.max_concurrent = max_concurrent;
        self.max_concurrent_per_owner = per_owner;
        self
    }

    /// The long-running announcement cadence. Zero disables it entirely, which
    /// is upstream's `Math.max(0, Number(progressCheckMs) || 0)`.
    #[must_use]
    pub const fn progress_check_ms(mut self, progress_check_ms: i64) -> Self {
        self.progress_check_ms = progress_check_ms;
        self
    }

    /// The default wall-clock budget for a `scheduled_task`.
    #[must_use]
    pub const fn scheduled_task_timeout_ms(mut self, timeout_ms: i64) -> Self {
        self.scheduled_task_timeout_ms = timeout_ms;
        self
    }

    /// The locale every message the manager composes renders in.
    #[must_use]
    pub const fn locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// The clock. Use [`crate::clock::tokio_clock`] under
    /// `#[tokio::test(start_paused = true)]`.
    #[must_use]
    pub fn now(mut self, now: NowFn) -> Self {
        self.now = Some(now);
        self
    }

    /// Where lifecycle events are logged, at the catalogued info/debug split.
    #[must_use]
    pub fn logger(mut self, logger: Logger) -> Self {
        self.logger = Some(logger);
        self
    }

    /// Override the command channel's depth.
    #[must_use]
    pub const fn command_capacity(mut self, capacity: usize) -> Self {
        self.command_capacity = capacity;
        self
    }

    /// Override the event channel's depth.
    #[must_use]
    pub const fn event_capacity(mut self, capacity: usize) -> Self {
        self.event_capacity = capacity;
        self
    }

    /// Register every task the manager spawns on a shared tracker, so a host
    /// can join them at shutdown (`docs/architecture.md` §11).
    #[must_use]
    pub fn tracker(mut self, tracker: TaskTracker) -> Self {
        self.tracker = Some(tracker);
        self
    }

    /// Take every Work setting from a [`Config`].
    ///
    /// The one place the operator-facing environment surface reaches this
    /// crate. `via_core` has already applied the clamps.
    #[must_use]
    pub fn configured(self, config: &Config) -> Self {
        let store = WorkStore::builder()
            .file_path(config.task_state_path.clone())
            .locale(config.locale)
            .self_scheduling(true)
            .build();
        self.store(store)
            .locale(config.locale)
            .retention(RetentionPolicy {
                terminal_ttl_ms: config.task_terminal_ttl_ms,
                pending_notification_ttl_ms: config.task_pending_notification_ttl_ms,
                notification_claim_ttl_ms: config.task_notification_claim_ttl_ms,
                max_terminal_tasks_per_owner: config.max_terminal_tasks_per_owner,
            })
            .concurrency(
                config.task_max_concurrent,
                config.task_max_concurrent_per_owner,
            )
            .progress_check_ms(config.background_task_progress_check_ms)
            .scheduled_task_timeout_ms(config.scheduled_task_timeout_ms)
    }

    /// Start the manager.
    ///
    /// Reads `tasks.json`, applies restart recovery, and spawns the owning
    /// task on the current runtime.
    ///
    /// # Panics
    ///
    /// Spawning needs a tokio runtime; call this from inside one.
    #[must_use]
    pub fn build(self) -> WorkManager {
        actor::spawn(self)
    }

    pub(crate) fn parts(self) -> ManagerParts {
        ManagerParts {
            store: self.store,
            runner: self.runner,
            retention: self.retention.clamped(),
            max_concurrent: self.max_concurrent,
            max_concurrent_per_owner: self.max_concurrent_per_owner,
            progress_check_ms: self.progress_check_ms.max(0),
            scheduled_task_timeout_ms: self.scheduled_task_timeout_ms,
            locale: self.locale,
            now: self.now.unwrap_or_else(system_clock),
            logger: self.logger,
            command_capacity: self.command_capacity,
            event_capacity: self.event_capacity,
            tracker: self.tracker.unwrap_or_default(),
        }
    }
}

/// A finished [`WorkManagerBuilder`], handed to the owning task.
pub(crate) struct ManagerParts {
    pub(crate) store: Option<WorkStore>,
    pub(crate) runner: Option<Arc<dyn WorkRunner>>,
    pub(crate) retention: RetentionPolicy,
    pub(crate) max_concurrent: i64,
    pub(crate) max_concurrent_per_owner: i64,
    pub(crate) progress_check_ms: i64,
    pub(crate) scheduled_task_timeout_ms: i64,
    pub(crate) locale: Locale,
    pub(crate) now: NowFn,
    pub(crate) logger: Option<Logger>,
    pub(crate) command_capacity: usize,
    pub(crate) event_capacity: usize,
    pub(crate) tracker: TaskTracker,
}

/// The Work subsystem's public handle.
///
/// Cheap to clone; every clone talks to the same owning task. Dropping the last
/// clone shuts the manager down: the actor drains, flushes `tasks.json` and
/// exits.
#[derive(Clone)]
pub struct WorkManager {
    commands: mpsc::Sender<Command>,
    events: broadcast::Sender<WorkEvent>,
    tracker: TaskTracker,
}

impl std::fmt::Debug for WorkManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkManager")
            .field("closed", &self.commands.is_closed())
            .field("subscribers", &self.events.receiver_count())
            .finish_non_exhaustive()
    }
}

/// Tell the actor something and wait for it to have been applied.
macro_rules! tell {
    ($self:expr, |$reply:ident| $command:expr) => {{
        let ($reply, response) = oneshot::channel::<()>();
        if $self.commands.send($command).await.is_ok() {
            let _ = response.await;
        }
    }};
}

/// Ask the actor a question, or answer `fallback` if it has shut down.
macro_rules! ask {
    ($self:expr, $fallback:expr, |$reply:ident| $command:expr) => {{
        let ($reply, response) = oneshot::channel();
        if $self.commands.send($command).await.is_err() {
            return $fallback;
        }
        response.await.unwrap_or($fallback)
    }};
}

impl WorkManager {
    /// Start building a manager.
    #[must_use]
    pub fn builder() -> WorkManagerBuilder {
        WorkManagerBuilder::default()
    }

    /// An in-memory manager on the shipped defaults.
    ///
    /// # Panics
    ///
    /// Needs a tokio runtime.
    #[must_use]
    pub fn in_memory() -> Self {
        Self::builder().build()
    }

    /// Subscribe to every lifecycle event.
    ///
    /// Events are delivered in the order the owning task raised them. A
    /// subscriber that falls more than the configured capacity behind receives
    /// [`broadcast::error::RecvError::Lagged`] and then resumes — upstream's
    /// synchronous listener set cannot lag, and cannot tell a slow observer
    /// that it missed something either.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<WorkEvent> {
        self.events.subscribe()
    }

    /// Whether the owning task is still running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        !self.commands.is_closed()
    }

    /// The tracker every task the manager spawned is registered on.
    #[must_use]
    pub fn tracker(&self) -> &TaskTracker {
        &self.tracker
    }

    /// Accept a Work.
    ///
    /// Returns immediately with a receipt; the Work is `queued` and the
    /// scheduler decides when it runs. A matching `submission_key` returns the
    /// existing Work with [`WorkAcceptance::reused`] set and starts nothing.
    ///
    /// # Errors
    ///
    /// [`ManagerStopped`] when the owning task has exited. A submission is the
    /// one call that must not answer plausibly on a dead manager: the caller
    /// would tell the model its Work was accepted and nothing would ever run.
    pub async fn create(&self, request: NewWork) -> Result<WorkAcceptance, ManagerStopped> {
        let (reply, response) = oneshot::channel();
        if self
            .commands
            .send(Command::Create {
                request: Box::new(request),
                reply,
            })
            .await
            .is_err()
        {
            return Err(ManagerStopped);
        }
        response.await.map_err(|_| ManagerStopped)
    }

    /// Schedule a Work for later.
    ///
    /// Does **not** drain: a scheduled Work waits for its timer
    /// (`server/src/task/task-manager.mjs:461`).
    ///
    /// # Errors
    ///
    /// [`ManagerStopped`], for the reason [`Self::create`] gives.
    pub async fn create_scheduled(
        &self,
        request: NewScheduledWork,
    ) -> Result<WorkAcceptance, ManagerStopped> {
        let (reply, response) = oneshot::channel();
        if self
            .commands
            .send(Command::CreateScheduled {
                request: Box::new(request),
                reply,
            })
            .await
            .is_err()
        {
            return Err(ManagerStopped);
        }
        response.await.map_err(|_| ManagerStopped)
    }

    /// Read one Work.
    ///
    /// `owner_id` is an authorization filter: a Work belonging to somebody else
    /// answers `None`, exactly as if it did not exist.
    pub async fn get(&self, work_id: &str, owner_id: Option<&str>) -> Option<PublicWork> {
        ask!(self, None, |reply| Command::Get {
            work_id: work_id.to_owned(),
            owner_id: owner_id.map(str::to_owned),
            reply,
        })
    }

    /// List Work, newest first.
    pub async fn list(&self, query: WorkQuery) -> Vec<PublicWork> {
        ask!(self, Vec::new(), |reply| Command::List { query, reply })
    }

    /// Ask a Work to stop.
    ///
    /// Answers `None` when there is nothing to cancel: no such Work, somebody
    /// else's Work, or a Work that has already finished.
    ///
    /// Otherwise it resolves when the cancellation is **confirmed** — the
    /// canceler reported a confirmed stop, or the abort it raised settled the
    /// runner. A `queued` or `scheduled` Work has nothing running, so it
    /// confirms at once. Until then the Work is `cancelling`, which is a state
    /// clients can see.
    pub async fn cancel(&self, work_id: &str, owner_id: Option<&str>) -> Option<PublicWork> {
        ask!(self, None, |reply| Command::Cancel {
            work_id: work_id.to_owned(),
            owner_id: owner_id.map(str::to_owned),
            reply,
        })
    }

    /// Confirm a cancellation that was only *requested*.
    ///
    /// For the case `docs/architecture.md` §4 names: the adapter sent
    /// `session/cancel` straight down the transport, and the confirmation —
    /// the backend's own `stopReason: 'cancelled'` — arrives later. `None`
    /// when the Work is not `cancelling`.
    pub async fn confirm_cancellation(
        &self,
        work_id: &str,
        confirmed_at: &str,
    ) -> Option<PublicWork> {
        ask!(self, None, |reply| Command::ConfirmCancellation {
            work_id: work_id.to_owned(),
            confirmed_at: confirmed_at.to_owned(),
            reply,
        })
    }

    /// Wait for a Work to reach a terminal status.
    ///
    /// `taskManager.wait(id)`. `None` for an unknown id — including one that
    /// retention has already dropped.
    pub async fn wait(&self, work_id: &str) -> Option<PublicWork> {
        ask!(self, None, |reply| Command::Wait {
            work_id: work_id.to_owned(),
            reply
        })
    }

    /// Claim pending notifications for delivery, oldest first.
    ///
    /// Each claimed Work moves to `delivering` and is invisible to a second
    /// claimant until the lease expires, is released, or is marked delivered.
    /// That is what makes a `task.completed` subscriber and a
    /// `task.notification.pending` subscriber deliver a result **once** between
    /// them.
    pub async fn claim_notifications(&self, request: NotificationClaim) -> Vec<PublicWork> {
        ask!(self, Vec::new(), |reply| Command::ClaimNotifications {
            request: Box::new(request),
            reply,
        })
    }

    /// Mark claimed notifications delivered. Returns how many moved.
    ///
    /// Only the lease holder may: a `claimant_id` that does not match is
    /// ignored.
    pub async fn mark_notifications_delivered(
        &self,
        work_ids: &[String],
        claimant_id: Option<&str>,
    ) -> usize {
        ask!(self, 0, |reply| Command::MarkDelivered {
            work_ids: work_ids.to_vec(),
            claimant_id: claimant_id.map(str::to_owned),
            reply,
        })
    }

    /// Renew a delivery lease. Returns how many were renewed.
    ///
    /// A long announcement outlives the 60-second lease; renewing is how a
    /// client that is still speaking keeps it.
    pub async fn renew_notification_claims(
        &self,
        work_ids: &[String],
        claimant_id: Option<&str>,
    ) -> usize {
        ask!(self, 0, |reply| Command::RenewClaims {
            work_ids: work_ids.to_vec(),
            claimant_id: claimant_id.map(str::to_owned),
            reply,
        })
    }

    /// Give a delivery lease back unplayed. Returns how many were released.
    pub async fn release_notification_claims(
        &self,
        work_ids: &[String],
        claimant_id: Option<&str>,
    ) -> usize {
        ask!(self, 0, |reply| Command::ReleaseClaims {
            work_ids: work_ids.to_vec(),
            claimant_id: claimant_id.map(str::to_owned),
            reply,
        })
    }

    /// Reclaim every lease older than the claim TTL. Returns how many.
    pub async fn reclaim_expired_claims(&self) -> usize {
        ask!(self, 0, |reply| Command::ReclaimExpiredClaims { reply })
    }

    /// Replace the retention policy and prune immediately.
    ///
    /// `taskManager.configureRetention({...})`, called once at composition.
    pub async fn configure_retention(&self, retention: RetentionPolicy) {
        tell!(self, |reply| Command::ConfigureRetention {
            policy: retention.clamped(),
            reply,
        });
    }

    /// Install the runner restored `scheduled_task` Work uses.
    ///
    /// A `scheduled_task` cannot persist its runner, so one is supplied here
    /// and attached when the Work starts
    /// (`server/src/task/task-manager.mjs:540-545`).
    pub async fn configure_scheduled_task_runner(&self, runner: Arc<dyn WorkRunner>) {
        tell!(self, |reply| Command::ConfigureScheduledTaskRunner {
            runner,
            reply
        });
    }

    /// Install the coordinator status query the progress check uses for
    /// `delegated` Work.
    pub async fn configure_coordinator_query(&self, query: Arc<dyn CoordinatorQuery>) {
        tell!(self, |reply| Command::ConfigureCoordinatorQuery {
            query,
            reply
        });
    }

    /// Offer every recoverable delegated Work to `recovery`.
    ///
    /// Returns how many were reattached. The rest fail with the *unrecoverable
    /// delegated work* restart message and queue a notification, so the user
    /// still hears about them.
    ///
    /// Call once, at composition: the candidate list is drained.
    pub async fn recover_delegated(&self, recovery: Arc<dyn DelegatedWorkRecovery>) -> usize {
        ask!(self, 0, |reply| Command::RecoverDelegated {
            recovery,
            reply
        })
    }

    /// Apply retention now. Returns how many Work items were dropped.
    pub async fn prune(&self) -> usize {
        ask!(self, 0, |reply| Command::Prune { reply })
    }

    /// Write `tasks.json` now.
    pub async fn persist(&self) -> bool {
        ask!(self, false, |reply| Command::Persist { reply })
    }

    /// Write any coalesced state now.
    ///
    /// The durability guarantee upstream's shutdown depends on: `flush()`
    /// before the server closes.
    pub async fn flush(&self) -> bool {
        ask!(self, false, |reply| Command::Flush { reply })
    }

    /// The store's health triple, as `/api/health` publishes it.
    pub async fn store_health(&self) -> StoreHealth {
        ask!(self, actor::unavailable_health(), |reply| {
            Command::StoreHealth { reply }
        })
    }

    /// Every Work waiting for its timer.
    ///
    /// The reminder scheduler's only read.
    pub async fn scheduled(&self) -> Vec<ScheduledWork> {
        ask!(self, Vec::new(), |reply| Command::ScheduledSnapshot {
            reply
        })
    }

    /// Move `work_ids` from `scheduled` to `queued` and drain.
    ///
    /// Returns how many actually fired: a Work that was cancelled between the
    /// timer being armed and it firing is skipped.
    pub async fn fire_scheduled(&self, work_ids: &[String]) -> usize {
        ask!(self, 0, |reply| Command::FireScheduled {
            work_ids: work_ids.to_vec(),
            reply,
        })
    }

    /// Record a cancellation fact the coordinator has not been told about.
    pub async fn record_cancellation_fact(&self, owner_id: &str, fact: CancellationFact) {
        tell!(self, |reply| Command::RecordCancellationFact {
            owner_id: owner_id.to_owned(),
            fact: Box::new(fact),
            reply,
        });
    }

    /// What `owner_id` still owes the coordinator. A peek.
    pub async fn pending_cancellation_facts(&self, owner_id: &str) -> Vec<CancellationFact> {
        ask!(self, Vec::new(), |reply| Command::PendingFacts {
            owner_id: owner_id.to_owned(),
            reply,
        })
    }

    /// Forget `owner_id`'s facts, once a coordinator turn has carried them.
    pub async fn clear_cancellation_facts(&self, owner_id: &str) -> usize {
        ask!(self, 0, |reply| Command::ClearFacts {
            owner_id: owner_id.to_owned(),
            reply,
        })
    }

    /// Write any coalesced state and release this handle.
    ///
    /// Dropping the last handle shuts the owning task down on its own; this is
    /// the spelling that also guarantees `tasks.json` is current first, which
    /// is upstream's `await taskStore.flush()` before `server.close()`.
    ///
    /// A host that must *join* the spawned tasks calls
    /// [`TaskTracker::close`] and [`TaskTracker::wait`] on its own
    /// [`Self::tracker`]; this does not, because a tracker handed in by the
    /// caller may still be carrying work that is not the manager's.
    pub async fn close(self) {
        self.flush().await;
        drop(self);
    }
}

/// The default session id, re-exported so a caller can spell the same value.
pub const MAIN_SESSION_ID: &str = DEFAULT_SESSION_ID;
