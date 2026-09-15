//! The announcement manager — batching, delivery, retry and the claim.
//!
//! Ported from `server/src/voice/announcement/announcement-manager.mjs`.
//!
//! This is the delivery half of the Injection Gate. The blocking half is
//! [`crate::announcement::window::AnnouncementWindow`]; this half decides *what*
//! is spoken, *when* it is retried, and — the hard part — *when it counts as
//! delivered*.
//!
//! # Delivered means played, not generated
//!
//! `response.done` proves the provider generated the audio. It does not prove
//! the client played it: desktop audio queues behind earlier speech, a socket
//! can drop mid-stream, and a barge-in discards what was queued. So a batch is
//! marked delivered only when the client reports playback — through
//! [`AnnouncementManager::confirm`], driven by the `playback.started` receipt
//! and, for a non-voice client, by `response.done`. Everything else is a retry.
//!
//! # Retries are bounded
//!
//! [`AnnouncementManagerConfig::max_retry_attempts`] caps them, and exhausting
//! it **abandons** the batch: the claims are released so another frontend can
//! take them, and the queue moves on. Without the cap one malformed result
//! blocks every later completion behind it forever, which is the failure this
//! bound exists to prevent.
//!
//! # The renewable claim
//!
//! Two live frontends for the same owner must not present the same result.
//! The Work manager's notification claim is the lock, and it has a TTL so a
//! frontend that dies does not hold it forever — which means a frontend that is
//! *alive* has to renew. [`AnnouncementManagerConfig::lease_renew_interval_ms`]
//! is that heartbeat, armed the moment a batch becomes active and cleared the
//! moment it is not.

use std::sync::Arc;
use std::time::Duration;

use indexmap::IndexMap;
use tokio::sync::{Mutex, mpsc};
use tokio_util::task::TaskTracker;
use via_i18n::Locale;

use crate::announcement::format::{Announcement, format_work_results, truncate_result};
use crate::frontend::{ResponseRequestContext, VoiceFrontend};
use crate::response::ResponseOrigin;

/// How many finished Work items may be presented at once.
///
/// **External contract** — `announcement-manager.mjs:7` (`maxBatchItems = 8`).
pub const DEFAULT_MAX_BATCH_ITEMS: usize = 8;

/// How long the manager waits for a second result before speaking the first.
///
/// **External contract** — `announcement-manager.mjs:8` (`batchWindowMs = 120`).
pub const DEFAULT_BATCH_WINDOW_MS: u64 = 120;

/// The first retry delay.
///
/// **External contract** — `announcement-manager.mjs:9` (`retryBaseMs = 1000`).
pub const DEFAULT_RETRY_BASE_MS: u64 = 1_000;

/// The retry delay ceiling.
///
/// **External contract** — `announcement-manager.mjs:10` (`retryMaxMs = 10000`).
pub const DEFAULT_RETRY_MAX_MS: u64 = 10_000;

/// How long a generated-but-unplayed batch waits before being re-delivered.
///
/// **External contract** — `announcement-manager.mjs:11`
/// (`acknowledgementTimeoutMs = 120000`). Two minutes, because long or queued
/// desktop audio may legitimately exceed any shorter window.
pub const DEFAULT_ACKNOWLEDGEMENT_TIMEOUT_MS: u64 = 120_000;

/// How many times one batch is retried before it is abandoned.
///
/// **External contract** — `announcement-manager.mjs:12`
/// (`maxRetryAttempts = 8`).
pub const DEFAULT_MAX_RETRY_ATTEMPTS: u32 = 8;

/// How often the notification claim is renewed while a batch is active.
///
/// **External contract** — `announcement-manager.mjs:13`
/// (`leaseRenewIntervalMs = 20000`); the gateway supplies
/// `max(1000, floor(claim_ttl / 3))`.
pub const DEFAULT_LEASE_RENEW_INTERVAL_MS: u64 = 20_000;

/// How many code points of result text one batch may carry.
///
/// **External contract** — `config.resultContextMaxChars` (6000).
pub const DEFAULT_RESULT_CONTEXT_MAX_CHARS: usize = 6_000;

/// The exponent cap on the retry backoff.
///
/// **External contract** — `announcement-manager.mjs:226`
/// (`2 ** Math.min(retryCount - 1, 4)`).
pub const RETRY_EXPONENT_CAP: u32 = 4;

/// What the manager does with the notification claims it holds.
///
/// The Work manager owns the lease; this is the seam so the manager can be
/// tested without one.
#[async_trait::async_trait]
pub trait NotificationClaims: Send + Sync + std::fmt::Debug {
    /// The batch was played. Mark the notifications delivered.
    async fn delivered(&self, work_ids: &[String]);
    /// The batch is still in flight. Extend the lease.
    async fn renew(&self, work_ids: &[String]);
    /// The batch was abandoned or paused. Let another frontend take it.
    async fn release(&self, work_ids: &[String]);
}

/// A claim sink that does nothing, for a Gateway with no Work manager.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoClaims;

#[async_trait::async_trait]
impl NotificationClaims for NoClaims {
    async fn delivered(&self, _work_ids: &[String]) {}
    async fn renew(&self, _work_ids: &[String]) {}
    async fn release(&self, _work_ids: &[String]) {}
}

/// How the manager is configured.
#[derive(Clone, Debug)]
pub struct AnnouncementManagerConfig {
    /// The locale the envelope is rendered in.
    pub locale: Locale,
    /// Whether the results are written into the model's conversation as well
    /// as spoken. `config.announceIntoContext`.
    pub announce_into_context: bool,
    /// The code-point budget for one batch's text.
    pub result_context_max_chars: usize,
    /// How many items one batch may carry.
    pub max_batch_items: usize,
    /// How long to wait for a second result.
    pub batch_window_ms: u64,
    /// The first retry delay.
    pub retry_base_ms: u64,
    /// The retry delay ceiling.
    pub retry_max_ms: u64,
    /// How long a generated batch waits for a playback receipt.
    pub acknowledgement_timeout_ms: u64,
    /// How many retries before abandoning.
    pub max_retry_attempts: u32,
    /// The claim heartbeat.
    pub lease_renew_interval_ms: u64,
}

impl Default for AnnouncementManagerConfig {
    fn default() -> Self {
        Self {
            locale: Locale::En,
            announce_into_context: true,
            result_context_max_chars: DEFAULT_RESULT_CONTEXT_MAX_CHARS,
            max_batch_items: DEFAULT_MAX_BATCH_ITEMS,
            batch_window_ms: DEFAULT_BATCH_WINDOW_MS,
            retry_base_ms: DEFAULT_RETRY_BASE_MS,
            retry_max_ms: DEFAULT_RETRY_MAX_MS,
            acknowledgement_timeout_ms: DEFAULT_ACKNOWLEDGEMENT_TIMEOUT_MS,
            max_retry_attempts: DEFAULT_MAX_RETRY_ATTEMPTS,
            lease_renew_interval_ms: DEFAULT_LEASE_RENEW_INTERVAL_MS,
        }
    }
}

/// Answers "is delivery blocked right now?".
///
/// The gateway supplies
/// `sleeping || waking || !output_enabled || window.is_blocked()`.
pub type DeliveryBlocked = Arc<dyn Fn() -> bool + Send + Sync>;

/// Supplies the live frontend, or `None` while reconnecting.
pub type FrontendSource = Arc<dyn Fn() -> Option<Arc<dyn VoiceFrontend>> + Send + Sync>;

#[derive(Clone, Debug)]
struct Batch {
    announcements: Vec<Announcement>,
    work_ids: Vec<String>,
    turn_ids: Vec<String>,
    delivery_sequence: u64,
    context_injected: bool,
    response_completed: bool,
    retry_requested: bool,
    acknowledged: Vec<String>,
}

impl Batch {
    fn fully_acknowledged(&self) -> bool {
        self.work_ids
            .iter()
            .all(|work_id| self.acknowledged.contains(work_id))
    }
}

/// Indices into [`Actor::epochs`].
struct Timer;

impl Timer {
    const DELIVERY: usize = 0;
    const RETRY: usize = 1;
    const ACKNOWLEDGEMENT: usize = 2;
    const LEASE: usize = 3;
}

#[derive(Debug)]
enum Command {
    Queue(Box<Announcement>),
    Remove(String),
    Confirm(String),
    ConfirmMany(Vec<String>),
    DismissActive,
    RetryMany(Vec<String>),
    Flush,
    Pause,
    Close,
    Deliver {
        epoch: u64,
    },
    /// One delivery attempt settled. Carries the batch it belongs to so the
    /// epilogue can tell whether that batch is still the active one.
    Delivered {
        generation: u64,
        work_ids: Vec<String>,
        outcome: Box<crate::frontend::ResponseOutcome>,
    },
    Retry {
        epoch: u64,
    },
    Acknowledgement {
        epoch: u64,
    },
    Renew {
        epoch: u64,
    },
}

/// The announcement manager's handle.
#[derive(Clone, Debug)]
pub struct AnnouncementManager {
    commands: mpsc::Sender<Command>,
    /// Mirrors the actor's pending set so `has` can answer without a round
    /// trip. Only ever read for diagnostics and the `has` check.
    pending: Arc<Mutex<IndexMap<String, ()>>>,
}

/// How many commands may queue before a caller waits.
const COMMAND_BUFFER: usize = 128;

impl AnnouncementManager {
    /// Start the owning task.
    #[must_use]
    pub fn new(
        config: AnnouncementManagerConfig,
        frontend: FrontendSource,
        blocked: DeliveryBlocked,
        claims: Arc<dyn NotificationClaims>,
    ) -> Self {
        Self::with_tracker(config, frontend, blocked, claims, None)
    }

    /// Start the owning task on a shared tracker.
    #[must_use]
    pub fn with_tracker(
        config: AnnouncementManagerConfig,
        frontend: FrontendSource,
        blocked: DeliveryBlocked,
        claims: Arc<dyn NotificationClaims>,
        tracker: Option<TaskTracker>,
    ) -> Self {
        let (commands, inbox) = mpsc::channel(COMMAND_BUFFER);
        let pending = Arc::new(Mutex::new(IndexMap::new()));
        let actor = Actor {
            config,
            frontend,
            blocked,
            claims,
            commands: commands.clone(),
            mirror: Arc::clone(&pending),
            pending: IndexMap::new(),
            active: None,
            delivering: false,
            delivery_generation: 0,
            epochs: [0; 4],
            sequence: 0,
            retry_count: 0,
            delivery_armed: false,
            retry_armed: false,
            acknowledgement_armed: false,
            lease_armed: false,
            closed: false,
        };
        let future = actor.run(inbox);
        match tracker {
            Some(tracker) => {
                tracker.spawn(future);
            }
            None => {
                tokio::spawn(future);
            }
        }
        Self { commands, pending }
    }

    /// Whether `work_id` is queued.
    pub async fn has(&self, work_id: &str) -> bool {
        self.pending.lock().await.contains_key(work_id)
    }

    /// Queue an announcement.
    pub async fn queue(&self, announcement: Announcement) {
        let _ = self
            .commands
            .send(Command::Queue(Box::new(announcement)))
            .await;
    }

    /// Drop a queued announcement without confirming it.
    ///
    /// Used when the Work was cancelled: there is nothing to say, but an
    /// active batch containing it must still be able to finish.
    pub async fn remove(&self, work_id: &str) {
        let _ = self
            .commands
            .send(Command::Remove(work_id.to_owned()))
            .await;
    }

    /// Mark one Work delivered.
    pub async fn confirm(&self, work_id: &str) {
        let _ = self
            .commands
            .send(Command::Confirm(work_id.to_owned()))
            .await;
    }

    /// Mark several Work items delivered.
    pub async fn confirm_many(&self, work_ids: Vec<String>) {
        let _ = self.commands.send(Command::ConfirmMany(work_ids)).await;
    }

    /// Mark the whole active batch delivered.
    ///
    /// Called on barge-in: the user cut the announcement off, which counts as
    /// having heard it. Re-speaking a result the user deliberately interrupted
    /// is worse than dropping it.
    pub async fn dismiss_active(&self) {
        let _ = self.commands.send(Command::DismissActive).await;
    }

    /// Ask for a redelivery of a batch that did not land.
    pub async fn retry_many(&self, work_ids: Vec<String>) {
        let _ = self.commands.send(Command::RetryMany(work_ids)).await;
    }

    /// Deliver now, if nothing is blocking.
    pub async fn flush(&self) {
        let _ = self.commands.send(Command::Flush).await;
    }

    /// Drop everything and release every claim.
    ///
    /// Called on sleep, deactivation and mute. The results are not lost: the
    /// claims are released, so the next frontend to connect re-claims them.
    pub async fn pause(&self) {
        let _ = self.commands.send(Command::Pause).await;
    }

    /// Stop permanently.
    pub async fn close(&self) {
        let _ = self.commands.send(Command::Close).await;
    }
}

struct Actor {
    config: AnnouncementManagerConfig,
    frontend: FrontendSource,
    blocked: DeliveryBlocked,
    claims: Arc<dyn NotificationClaims>,
    commands: mpsc::Sender<Command>,
    mirror: Arc<Mutex<IndexMap<String, ()>>>,
    pending: IndexMap<String, Announcement>,
    active: Option<Batch>,
    delivering: bool,
    /// Bumped only by [`Actor::pause`], which also clears `pending` — so a
    /// delivery whose epilogue finds the generation moved has nothing left to
    /// schedule. Retiring a batch does **not** bump it: the epilogue is the
    /// only thing that starts the next delivery, and skipping it there stranded
    /// anything queued while the delivery was in flight.
    delivery_generation: u64,
    /// One epoch per timer kind. Cancelling a retry must not silently cancel
    /// the lease heartbeat, which is what a single shared counter did.
    epochs: [u64; 4],
    sequence: u64,
    retry_count: u32,
    delivery_armed: bool,
    retry_armed: bool,
    acknowledgement_armed: bool,
    lease_armed: bool,
    closed: bool,
}

impl Actor {
    async fn run(mut self, mut inbox: mpsc::Receiver<Command>) {
        while let Some(command) = inbox.recv().await {
            match command {
                Command::Queue(announcement) => self.queue(*announcement).await,
                Command::Remove(work_id) => self.remove(&work_id).await,
                Command::Confirm(work_id) => self.confirm(&work_id).await,
                Command::ConfirmMany(work_ids) => {
                    for work_id in work_ids {
                        self.confirm(&work_id).await;
                    }
                }
                Command::DismissActive => {
                    let work_ids = self
                        .active
                        .as_ref()
                        .map(|batch| batch.work_ids.clone())
                        .unwrap_or_default();
                    for work_id in work_ids {
                        self.confirm(&work_id).await;
                    }
                }
                Command::RetryMany(work_ids) => self.retry_many(&work_ids),
                Command::Flush => {
                    if !(self.blocked)() {
                        self.delivery_armed = false;
                        self.deliver().await;
                    }
                }
                Command::Pause => self.pause().await,
                Command::Close => {
                    self.closed = true;
                    self.pause().await;
                }
                Command::Deliver { epoch } => {
                    if epoch == self.epochs[Timer::DELIVERY] {
                        self.delivery_armed = false;
                        self.deliver().await;
                    }
                }
                Command::Delivered {
                    generation,
                    work_ids,
                    outcome,
                } => self.on_delivered(generation, &work_ids, &outcome),
                Command::Retry { epoch } => {
                    if epoch == self.epochs[Timer::RETRY] {
                        self.retry_armed = false;
                        self.deliver().await;
                    }
                }
                Command::Acknowledgement { epoch } => {
                    if epoch == self.epochs[Timer::ACKNOWLEDGEMENT] {
                        self.acknowledgement_armed = false;
                        self.on_acknowledgement_timeout();
                    }
                }
                Command::Renew { epoch } => {
                    if epoch == self.epochs[Timer::LEASE] {
                        self.lease_armed = false;
                        self.on_lease_tick().await;
                    }
                }
            }
        }
    }

    async fn sync_mirror(&self) {
        let mut mirror = self.mirror.lock().await;
        mirror.clear();
        for work_id in self.pending.keys() {
            mirror.insert(work_id.clone(), ());
        }
    }

    async fn queue(&mut self, mut announcement: Announcement) {
        self.sequence += 1;
        announcement.sequence = self.sequence;
        self.pending
            .insert(announcement.work_id.clone(), announcement);
        self.sync_mirror().await;
        self.schedule_delivery(self.config.batch_window_ms);
    }

    async fn remove(&mut self, work_id: &str) {
        self.pending.shift_remove(work_id);
        self.sync_mirror().await;
        if let Some(batch) = &mut self.active
            && batch.work_ids.iter().any(|id| id == work_id)
            && !batch.acknowledged.iter().any(|id| id == work_id)
        {
            batch.acknowledged.push(work_id.to_owned());
        }
        self.finish_acknowledged_batch();
    }

    async fn confirm(&mut self, work_id: &str) {
        let tracked = self.pending.contains_key(work_id)
            || self
                .active
                .as_ref()
                .is_some_and(|batch| batch.work_ids.iter().any(|id| id == work_id));
        self.remove(work_id).await;
        if tracked {
            self.claims.delivered(&[work_id.to_owned()]).await;
        }
    }

    /// Ask for a redelivery.
    ///
    /// **External contract** — `announcement-manager.mjs:104-117`. The
    /// `retry_requested` flag exists because a retry can be asked for *while a
    /// delivery is in flight*: setting it lets the delivery finish and re-arm
    /// itself in its own epilogue rather than starting a second one.
    fn retry_many(&mut self, work_ids: &[String]) {
        let Some(batch) = &mut self.active else {
            return;
        };
        if !work_ids
            .iter()
            .any(|work_id| batch.work_ids.iter().any(|id| id == work_id))
        {
            return;
        }
        batch.response_completed = false;
        batch.retry_requested = true;
        // Only these two. Bumping a shared counter here also cancelled the
        // lease heartbeat — leaving a retrying batch whose claim quietly
        // expired — and stranded the in-flight delivery's epilogue, which is
        // what honours `retry_requested`.
        self.cancel(Timer::ACKNOWLEDGEMENT);
        self.cancel(Timer::RETRY);
        self.acknowledgement_armed = false;
        self.retry_armed = false;
        if !self.delivering {
            if let Some(batch) = &mut self.active {
                batch.retry_requested = false;
            }
            self.schedule_retry();
        }
    }

    fn finish_acknowledged_batch(&mut self) {
        let done = self
            .active
            .as_ref()
            .is_some_and(super::manager::Batch::fully_acknowledged);
        if !done {
            return;
        }
        self.active = None;
        self.retry_count = 0;
        self.cancel_batch_timers();
        if !self.delivering && !self.pending.is_empty() {
            self.schedule_delivery(0);
        }
    }

    /// Cancel every timer that belongs to an active batch.
    fn cancel_batch_timers(&mut self) {
        self.retry_armed = false;
        self.acknowledgement_armed = false;
        self.lease_armed = false;
        self.cancel(Timer::RETRY);
        self.cancel(Timer::ACKNOWLEDGEMENT);
        self.cancel(Timer::LEASE);
    }

    fn schedule_delivery(&mut self, delay_ms: u64) {
        if self.closed
            || self.delivery_armed
            || self.retry_armed
            || self.active.is_some()
            || self.pending.is_empty()
        {
            return;
        }
        self.delivery_armed = true;
        self.arm(Timer::DELIVERY, delay_ms, |epoch| Command::Deliver {
            epoch,
        });
    }

    fn schedule_retry(&mut self) {
        if self.closed || self.active.is_none() || self.retry_armed {
            return;
        }
        if self.retry_count >= self.config.max_retry_attempts {
            // Bounded, so one malformed result cannot block every later
            // completion behind it.
            self.retry_count = 0;
            self.abandon_now();
            return;
        }
        self.retry_count += 1;
        let delay = self.config.retry_max_ms.min(
            self.config
                .retry_base_ms
                .saturating_mul(1u64 << (self.retry_count - 1).min(RETRY_EXPONENT_CAP)),
        );
        self.retry_armed = true;
        self.arm(Timer::RETRY, delay, |epoch| Command::Retry { epoch });
    }

    fn abandon_now(&mut self) {
        let Some(batch) = self.active.take() else {
            return;
        };
        for work_id in &batch.work_ids {
            self.pending.shift_remove(work_id);
        }
        self.cancel_batch_timers();
        let claims = Arc::clone(&self.claims);
        let mirror = Arc::clone(&self.mirror);
        let remaining: Vec<String> = self.pending.keys().cloned().collect();
        let work_ids = batch.work_ids.clone();
        tokio::spawn(async move {
            let mut mirror = mirror.lock().await;
            mirror.clear();
            for work_id in remaining {
                mirror.insert(work_id, ());
            }
            drop(mirror);
            if !work_ids.is_empty() {
                claims.release(&work_ids).await;
            }
        });
        if !self.delivering && !self.pending.is_empty() {
            self.schedule_delivery(0);
        }
    }

    /// Cancel one timer kind. A command already in flight for it is ignored
    /// when it arrives.
    fn cancel(&mut self, timer: usize) {
        self.epochs[timer] += 1;
    }

    /// Arm one timer kind, superseding anything already armed for it.
    fn arm(&mut self, timer: usize, delay_ms: u64, build: fn(u64) -> Command) {
        self.epochs[timer] += 1;
        let epoch = self.epochs[timer];
        let commands = self.commands.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            let _ = commands.send(build(epoch)).await;
        });
    }

    fn schedule_acknowledgement_timeout(&mut self) {
        self.acknowledgement_armed = true;
        self.arm(
            Timer::ACKNOWLEDGEMENT,
            self.config.acknowledgement_timeout_ms,
            |epoch| Command::Acknowledgement { epoch },
        );
    }

    /// The generated batch was never reported as played.
    ///
    /// **External contract** — `announcement-manager.mjs:167-188`. When the
    /// gate is still blocked the timeout **re-arms** instead of retrying:
    /// creating a second spoken response while the first is queued, playing or
    /// blocked by live speech is exactly the failure the gate exists for.
    fn on_acknowledgement_timeout(&mut self) {
        let completed = self
            .active
            .as_ref()
            .is_some_and(|batch| batch.response_completed);
        if !completed {
            return;
        }
        if (self.blocked)() {
            self.schedule_acknowledgement_timeout();
            return;
        }
        if let Some(batch) = &mut self.active {
            batch.response_completed = false;
            batch.retry_requested = true;
        }
        if !self.delivering {
            if let Some(batch) = &mut self.active {
                batch.retry_requested = false;
            }
            self.schedule_retry();
        }
    }

    async fn ensure_lease_renewal(&mut self) {
        if self.lease_armed || self.active.is_none() {
            return;
        }
        if let Some(batch) = &self.active {
            self.claims.renew(&batch.work_ids).await;
        }
        self.lease_armed = true;
        self.arm(Timer::LEASE, self.config.lease_renew_interval_ms, |epoch| {
            Command::Renew { epoch }
        });
    }

    async fn on_lease_tick(&mut self) {
        let Some(batch) = &self.active else {
            return;
        };
        self.claims.renew(&batch.work_ids).await;
        self.lease_armed = true;
        self.arm(Timer::LEASE, self.config.lease_renew_interval_ms, |epoch| {
            Command::Renew { epoch }
        });
    }

    /// Build the next batch, respecting both bounds.
    ///
    /// **External contract** — `announcement-manager.mjs:235-267`. The size
    /// probe formats the *candidate* batch each time, because the envelope and
    /// the per-event headers are part of what has to fit. The first item is
    /// always taken, however large: dropping it would leave a result that can
    /// never be announced.
    fn create_batch(&self) -> Option<Batch> {
        let mut pending: Vec<Announcement> = self.pending.values().cloned().collect();
        pending.sort_by_key(|announcement| announcement.sequence);
        if pending.is_empty() {
            return None;
        }
        let mut queued: Vec<Announcement> = Vec::new();
        for item in pending {
            let mut candidate = queued.clone();
            candidate.push(item.clone());
            let text = format_work_results(&candidate, self.config.locale);
            if !queued.is_empty()
                && (queued.len() >= self.config.max_batch_items
                    || crate::text::code_point_len(&text) > self.config.result_context_max_chars)
            {
                break;
            }
            queued.push(item);
        }
        let turn_ids: Vec<String> = queued
            .iter()
            .filter_map(|item| item.turn_id.clone())
            .filter(|turn_id| !turn_id.is_empty())
            .collect();
        Some(Batch {
            work_ids: queued.iter().map(|item| item.work_id.clone()).collect(),
            delivery_sequence: queued.first().map_or(0, |item| item.sequence),
            announcements: queued,
            turn_ids,
            context_injected: false,
            response_completed: false,
            retry_requested: false,
            acknowledged: Vec::new(),
        })
    }

    async fn deliver(&mut self) {
        if self.closed || self.delivering || (self.blocked)() {
            return;
        }
        let frontend = (self.frontend)();
        let ready = frontend.as_ref().is_some_and(|frontend| frontend.ready());
        if !ready {
            // Claim the batch even while disconnected: the results are ours to
            // present once the socket comes back, and releasing them here would
            // let a second frontend take them mid-reconnect.
            if self.active.is_none() {
                self.active = self.create_batch();
            }
            self.ensure_lease_renewal().await;
            self.schedule_retry();
            return;
        }
        let Some(frontend) = frontend else { return };
        if self.active.is_none() {
            self.active = self.create_batch();
        }
        let Some(batch) = self.active.clone() else {
            return;
        };
        let generation = self.delivery_generation;
        self.ensure_lease_renewal().await;
        self.delivering = true;

        let text = truncate_result(
            &format_work_results(&batch.announcements, self.config.locale),
            self.config.result_context_max_chars,
            self.config.locale,
        );
        let context = ResponseRequestContext {
            turn_id: (batch.turn_ids.len() == 1).then(|| batch.turn_ids[0].clone()),
            task_id: (batch.work_ids.len() == 1).then(|| batch.work_ids[0].clone()),
            task_ids: batch.work_ids.clone(),
            turn_ids: batch.turn_ids.clone(),
            delivery_sequence: Some(batch.delivery_sequence),
            ..ResponseRequestContext::default()
        };

        // The attempt runs *outside* the actor loop. Awaiting it here would
        // hold the actor for as long as the provider takes to settle a
        // response — up to the response-start timeout — during which a
        // `playback.started` receipt could not confirm delivery, a barge-in
        // could not dismiss the batch, and `pause` could not release the claim
        // on sleep. The outcome comes back as a command.
        let inject_context = !batch.context_injected;
        let announce_into_context = self.config.announce_into_context;
        let commands = self.commands.clone();
        let work_ids = batch.work_ids.clone();
        tokio::spawn(async move {
            let outcome = if announce_into_context {
                frontend
                    .inject_result(&text, ResponseOrigin::Announcement, context, inject_context)
                    .await
            } else {
                frontend
                    .speak(&text, ResponseOrigin::Announcement, context)
                    .await
            };
            let _ = commands
                .send(Command::Delivered {
                    generation,
                    work_ids,
                    outcome: Box::new(outcome),
                })
                .await;
        });
    }

    /// One delivery attempt settled.
    ///
    /// **External contract** — `announcement-manager.mjs:308-342`, including
    /// its `finally`. The outcome applies only to the batch that produced it;
    /// the tail runs regardless, because it is the only thing that starts the
    /// next delivery once `delivering` goes false.
    fn on_delivered(
        &mut self,
        generation: u64,
        work_ids: &[String],
        outcome: &crate::frontend::ResponseOutcome,
    ) {
        self.delivering = false;
        if generation != self.delivery_generation {
            // Only `pause` moves the generation, and it clears `pending` too,
            // so there is nothing left to schedule.
            return;
        }

        let still_active = self
            .active
            .as_ref()
            .is_some_and(|active| active.work_ids == work_ids);
        if still_active {
            if let Some(active) = &mut self.active {
                if outcome.context_injected {
                    active.context_injected = true;
                }
                if outcome.completed {
                    // Realtime generated the response; the client may still
                    // have it queued behind earlier audio. Delivery is
                    // confirmed only when the client reports that playback
                    // actually started.
                    active.response_completed = true;
                }
            }
            if outcome.completed {
                self.schedule_acknowledgement_timeout();
            } else {
                self.schedule_retry();
            }
        }

        self.finish_acknowledged_batch();
        let retry_requested = self
            .active
            .as_ref()
            .is_some_and(|batch| batch.retry_requested);
        if retry_requested {
            if let Some(batch) = &mut self.active {
                batch.retry_requested = false;
            }
            self.schedule_retry();
        }
        if self.active.is_none() && !self.pending.is_empty() {
            self.schedule_delivery(0);
        }
    }

    async fn pause(&mut self) {
        let mut work_ids: Vec<String> = self.pending.keys().cloned().collect();
        if let Some(batch) = &self.active {
            for work_id in &batch.work_ids {
                if !work_ids.contains(work_id) {
                    work_ids.push(work_id.clone());
                }
            }
        }
        self.delivery_armed = false;
        self.cancel(Timer::DELIVERY);
        self.cancel_batch_timers();
        self.delivery_generation += 1;
        self.pending.clear();
        self.sync_mirror().await;
        self.active = None;
        self.delivering = false;
        self.retry_count = 0;
        if !work_ids.is_empty() {
            self.claims.release(&work_ids).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ResponseOutcome;
    use crate::provider::ProviderView;
    use pretty_assertions::assert_eq;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    #[derive(Debug, Default)]
    struct RecordingClaims {
        delivered: Mutex<Vec<Vec<String>>>,
        renewed: AtomicUsize,
        released: Mutex<Vec<Vec<String>>>,
    }

    #[async_trait::async_trait]
    impl NotificationClaims for RecordingClaims {
        async fn delivered(&self, work_ids: &[String]) {
            self.delivered.lock().await.push(work_ids.to_vec());
        }
        async fn renew(&self, _work_ids: &[String]) {
            self.renewed.fetch_add(1, Ordering::SeqCst);
        }
        async fn release(&self, work_ids: &[String]) {
            self.released.lock().await.push(work_ids.to_vec());
        }
    }

    struct FakeFrontend {
        ready: AtomicBool,
        completes: AtomicBool,
        spoken: Mutex<Vec<String>>,
        injections: AtomicUsize,
        context_injections: AtomicUsize,
        /// While set, a delivery attempt parks instead of settling — the
        /// window in which the client's own receipts arrive.
        held: AtomicBool,
        release: tokio::sync::Notify,
    }

    impl FakeFrontend {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                ready: AtomicBool::new(true),
                completes: AtomicBool::new(true),
                spoken: Mutex::new(Vec::new()),
                injections: AtomicUsize::new(0),
                context_injections: AtomicUsize::new(0),
                held: AtomicBool::new(false),
                release: tokio::sync::Notify::new(),
            })
        }

        async fn park_if_held(&self) {
            while self.held.load(Ordering::SeqCst) {
                self.release.notified().await;
            }
        }

        fn release_delivery(&self) {
            self.held.store(false, Ordering::SeqCst);
            self.release.notify_waiters();
        }
    }

    #[async_trait::async_trait]
    impl VoiceFrontend for FakeFrontend {
        fn ready(&self) -> bool {
            self.ready.load(Ordering::SeqCst)
        }
        fn provider(&self) -> &dyn ProviderView {
            unimplemented!("the announcement manager never reads the provider")
        }
        async fn append_audio(&self, _audio_base64: &str) {}
        async fn speak(
            &self,
            text: &str,
            _origin: ResponseOrigin,
            _context: ResponseRequestContext,
        ) -> ResponseOutcome {
            self.spoken.lock().await.push(text.to_owned());
            self.park_if_held().await;
            if self.completes.load(Ordering::SeqCst) {
                ResponseOutcome::completed("resp_1")
            } else {
                ResponseOutcome::skipped()
            }
        }
        async fn inject_result(
            &self,
            text: &str,
            _origin: ResponseOrigin,
            _context: ResponseRequestContext,
            inject_context: bool,
        ) -> ResponseOutcome {
            self.injections.fetch_add(1, Ordering::SeqCst);
            if inject_context {
                self.context_injections.fetch_add(1, Ordering::SeqCst);
            }
            self.spoken.lock().await.push(text.to_owned());
            self.park_if_held().await;
            let mut outcome = if self.completes.load(Ordering::SeqCst) {
                ResponseOutcome::completed("resp_1")
            } else {
                ResponseOutcome::skipped()
            };
            outcome.context_injected = inject_context;
            outcome
        }
        async fn ensure_response(
            &self,
            _context: ResponseRequestContext,
            _instructions: Option<String>,
        ) -> ResponseOutcome {
            ResponseOutcome::default()
        }
        async fn send_function_output(
            &self,
            _call_id: &str,
            _output: &serde_json::Value,
            _context: ResponseRequestContext,
            _options: crate::frontend::FunctionOutputOptions,
        ) -> ResponseOutcome {
            ResponseOutcome::default()
        }
        async fn append_user_input_context(
            &self,
            _parts: &[crate::input::InputPart],
            _accompanies_voice: bool,
        ) -> ResponseOutcome {
            ResponseOutcome::default()
        }
        async fn send_user_input(
            &self,
            _parts: &[crate::input::InputPart],
            _context: ResponseRequestContext,
        ) -> ResponseOutcome {
            ResponseOutcome::default()
        }
        async fn append_user_context(&self, _text: &str) -> ResponseOutcome {
            ResponseOutcome::default()
        }
        async fn update_agent_context(&self) {}
        async fn cancel(&self) {}
        async fn cancel_responses(
            &self,
            _predicate: &(dyn Fn(&ResponseRequestContext, ResponseOrigin) -> bool + Send + Sync),
        ) -> bool {
            false
        }
        async fn close(&self) {}
    }

    struct Harness {
        manager: AnnouncementManager,
        frontend: Arc<FakeFrontend>,
        claims: Arc<RecordingClaims>,
        blocked: Arc<AtomicBool>,
    }

    fn harness(config: AnnouncementManagerConfig) -> Harness {
        let frontend = FakeFrontend::new();
        let claims = Arc::new(RecordingClaims::default());
        let blocked = Arc::new(AtomicBool::new(false));
        let source_frontend = Arc::clone(&frontend);
        let blocked_read = Arc::clone(&blocked);
        let manager = AnnouncementManager::new(
            config,
            Arc::new(move || Some(Arc::clone(&source_frontend) as Arc<dyn VoiceFrontend>)),
            Arc::new(move || blocked_read.load(Ordering::SeqCst)),
            Arc::clone(&claims) as Arc<dyn NotificationClaims>,
        );
        Harness {
            manager,
            frontend,
            claims,
            blocked,
        }
    }

    fn config() -> AnnouncementManagerConfig {
        AnnouncementManagerConfig {
            locale: Locale::Zh,
            ..AnnouncementManagerConfig::default()
        }
    }

    async fn settle() {
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    #[tokio::test(start_paused = true)]
    async fn a_queued_result_is_spoken_after_the_batch_window() {
        let harness = harness(config());
        harness
            .manager
            .queue(Announcement::completed("work_1", "查天气", "晴"))
            .await;
        settle().await;
        assert!(
            harness.manager.has("work_1").await,
            "queued until a playback receipt confirms it",
        );
        let spoken = harness.frontend.spoken.lock().await;
        assert_eq!(spoken.len(), 1);
        assert!(spoken[0].contains("work_id: work_1"));
        assert!(spoken[0].starts_with("[COMPLETE]"));
    }

    #[tokio::test(start_paused = true)]
    async fn nothing_is_spoken_while_the_gate_is_blocked() {
        let harness = harness(config());
        harness.blocked.store(true, Ordering::SeqCst);
        harness
            .manager
            .queue(Announcement::completed("work_1", "x", "y"))
            .await;
        settle().await;
        assert!(harness.frontend.spoken.lock().await.is_empty());
        harness.blocked.store(false, Ordering::SeqCst);
        harness.manager.flush().await;
        settle().await;
        assert_eq!(harness.frontend.spoken.lock().await.len(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn delivery_is_confirmed_only_by_a_playback_receipt() {
        let harness = harness(config());
        harness
            .manager
            .queue(Announcement::completed("work_1", "x", "y"))
            .await;
        settle().await;
        assert!(
            harness.claims.delivered.lock().await.is_empty(),
            "response.done is not playback",
        );
        harness.manager.confirm("work_1").await;
        settle().await;
        assert_eq!(
            harness.claims.delivered.lock().await.as_slice(),
            &[vec!["work_1".to_owned()]],
        );
        assert!(!harness.manager.has("work_1").await);
    }

    #[tokio::test(start_paused = true)]
    async fn several_results_are_batched_into_one_envelope() {
        let harness = harness(config());
        for index in 1..=3 {
            harness
                .manager
                .queue(Announcement::completed(
                    &format!("work_{index}"),
                    "objective",
                    "result",
                ))
                .await;
        }
        settle().await;
        let spoken = harness.frontend.spoken.lock().await;
        assert_eq!(spoken.len(), 1, "one response, not three");
        assert!(spoken[0].contains("--- event 1 ---"));
        assert!(spoken[0].contains("--- event 3 ---"));
    }

    #[tokio::test(start_paused = true)]
    async fn the_batch_respects_the_item_cap() {
        let harness = harness(AnnouncementManagerConfig {
            max_batch_items: 2,
            ..config()
        });
        for index in 1..=4 {
            harness
                .manager
                .queue(Announcement::completed(
                    &format!("work_{index}"),
                    "objective",
                    "result",
                ))
                .await;
        }
        settle().await;
        let spoken = harness.frontend.spoken.lock().await;
        assert_eq!(spoken[0].matches("--- event ").count(), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn the_batch_respects_the_character_budget_and_always_takes_the_first() {
        let harness = harness(AnnouncementManagerConfig {
            result_context_max_chars: 10,
            ..config()
        });
        harness
            .manager
            .queue(Announcement::completed("work_1", "a", &"阿".repeat(500)))
            .await;
        harness
            .manager
            .queue(Announcement::completed("work_2", "b", "short"))
            .await;
        settle().await;
        let spoken = harness.frontend.spoken.lock().await;
        assert_eq!(spoken.len(), 1);
        assert_eq!(
            crate::text::code_point_len(&spoken[0]),
            10,
            "truncated to the budget",
        );
        assert!(
            harness.manager.has("work_2").await,
            "the second stayed queued"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn the_context_item_is_written_once_and_not_again_on_retry() {
        let harness = harness(AnnouncementManagerConfig {
            retry_base_ms: 20,
            retry_max_ms: 20,
            ..config()
        });
        harness
            .manager
            .queue(Announcement::completed("work_1", "x", "y"))
            .await;
        settle().await;
        assert_eq!(
            harness.frontend.context_injections.load(Ordering::SeqCst),
            1
        );
        harness.manager.retry_many(vec!["work_1".to_owned()]).await;
        settle().await;
        assert_eq!(
            harness.frontend.context_injections.load(Ordering::SeqCst),
            1,
            "re-writing it would duplicate the results in the conversation",
        );
        assert!(harness.frontend.injections.load(Ordering::SeqCst) >= 2);
    }

    #[tokio::test(start_paused = true)]
    async fn a_result_queued_during_a_delivery_is_not_stranded_by_the_confirmation() {
        // The epilogue is the only thing that starts the next delivery once
        // `delivering` goes false. Retiring the active batch used to bump the
        // generation the epilogue checks, so a result queued while a delivery
        // was in flight sat there until some unrelated event called `flush`.
        let harness = harness(config());
        harness.frontend.held.store(true, Ordering::SeqCst);
        harness
            .manager
            .queue(Announcement::completed("work_1", "first", "one"))
            .await;
        settle().await;
        assert_eq!(
            harness.frontend.spoken.lock().await.len(),
            1,
            "the first delivery started and is parked",
        );

        // The actor is still responsive while the attempt is parked — which is
        // the point of running it outside the loop.
        harness
            .manager
            .queue(Announcement::completed("work_2", "second", "two"))
            .await;
        harness.manager.confirm("work_1").await;
        settle().await;
        assert!(
            !harness.manager.has("work_1").await,
            "the receipt was processed while the delivery was still in flight",
        );

        harness.frontend.release_delivery();
        settle().await;
        settle().await;

        let spoken = harness.frontend.spoken.lock().await;
        assert!(
            spoken.iter().any(|text| text.contains("work_id: work_2")),
            "the queued result was stranded: {spoken:?}",
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_retrying_batch_keeps_renewing_its_claim() {
        // The four timers used to share one counter, so cancelling the
        // acknowledgement timer on a retry silently cancelled the lease
        // heartbeat too — and a batch that kept retrying let its claim expire,
        // which is exactly how two frontends end up presenting the same result.
        let harness = harness(AnnouncementManagerConfig {
            lease_renew_interval_ms: 100,
            retry_base_ms: 50,
            retry_max_ms: 50,
            ..config()
        });
        harness
            .manager
            .queue(Announcement::completed("work_1", "x", "y"))
            .await;
        settle().await;
        let before = harness.claims.renewed.load(Ordering::SeqCst);
        assert!(before >= 1);

        harness.manager.retry_many(vec!["work_1".to_owned()]).await;
        settle().await;
        assert!(
            harness.claims.renewed.load(Ordering::SeqCst) > before,
            "a retrying batch must keep its claim alive",
        );
    }

    #[tokio::test(start_paused = true)]
    async fn retries_are_bounded_and_the_batch_is_abandoned() {
        let harness = harness(AnnouncementManagerConfig {
            max_retry_attempts: 2,
            retry_base_ms: 10,
            retry_max_ms: 10,
            ..config()
        });
        harness.frontend.completes.store(false, Ordering::SeqCst);
        harness
            .manager
            .queue(Announcement::completed("work_1", "x", "y"))
            .await;
        for _ in 0..8 {
            settle().await;
        }
        assert!(
            !harness.claims.released.lock().await.is_empty(),
            "an unspeakable result must release its claim, not block the queue",
        );
        assert!(!harness.manager.has("work_1").await);
    }

    #[tokio::test(start_paused = true)]
    async fn a_later_result_is_delivered_after_an_abandoned_one() {
        let harness = harness(AnnouncementManagerConfig {
            max_retry_attempts: 1,
            retry_base_ms: 10,
            retry_max_ms: 10,
            ..config()
        });
        harness.frontend.completes.store(false, Ordering::SeqCst);
        harness
            .manager
            .queue(Announcement::completed("work_1", "x", "y"))
            .await;
        for _ in 0..4 {
            settle().await;
        }
        harness.frontend.completes.store(true, Ordering::SeqCst);
        harness
            .manager
            .queue(Announcement::completed("work_2", "a", "b"))
            .await;
        for _ in 0..4 {
            settle().await;
        }
        let spoken = harness.frontend.spoken.lock().await;
        assert!(
            spoken.iter().any(|text| text.contains("work_id: work_2")),
            "the queue moved on: {spoken:?}",
        );
    }

    #[tokio::test(start_paused = true)]
    async fn the_claim_is_renewed_while_a_batch_is_active() {
        let harness = harness(AnnouncementManagerConfig {
            lease_renew_interval_ms: 100,
            ..config()
        });
        harness
            .manager
            .queue(Announcement::completed("work_1", "x", "y"))
            .await;
        settle().await;
        let first = harness.claims.renewed.load(Ordering::SeqCst);
        assert!(first >= 1, "the claim is taken at once");
        settle().await;
        assert!(
            harness.claims.renewed.load(Ordering::SeqCst) > first,
            "a live frontend keeps renewing",
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_disconnected_frontend_claims_the_batch_and_retries() {
        // The retry interval has to outlast the observation window: with the
        // default eight attempts a 10 ms interval would exhaust the budget and
        // abandon the batch before the socket could come back.
        let harness = harness(AnnouncementManagerConfig {
            retry_base_ms: 200,
            retry_max_ms: 200,
            ..config()
        });
        harness.frontend.ready.store(false, Ordering::SeqCst);
        harness
            .manager
            .queue(Announcement::completed("work_1", "x", "y"))
            .await;
        tokio::time::sleep(Duration::from_millis(400)).await;
        assert!(harness.frontend.spoken.lock().await.is_empty());
        assert!(
            harness.claims.renewed.load(Ordering::SeqCst) >= 1,
            "the batch is claimed even while disconnected",
        );
        harness.frontend.ready.store(true, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(400)).await;
        assert_eq!(harness.frontend.spoken.lock().await.len(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn pause_releases_every_claim_so_another_frontend_can_take_them() {
        let harness = harness(config());
        harness
            .manager
            .queue(Announcement::completed("work_1", "x", "y"))
            .await;
        settle().await;
        harness.manager.pause().await;
        settle().await;
        let released = harness.claims.released.lock().await;
        assert_eq!(released.len(), 1);
        assert!(released[0].contains(&"work_1".to_owned()));
        assert!(!harness.manager.has("work_1").await);
    }

    #[tokio::test(start_paused = true)]
    async fn dismiss_active_confirms_the_whole_batch() {
        let harness = harness(config());
        harness
            .manager
            .queue(Announcement::completed("work_1", "x", "y"))
            .await;
        harness
            .manager
            .queue(Announcement::completed("work_2", "a", "b"))
            .await;
        settle().await;
        harness.manager.dismiss_active().await;
        settle().await;
        let delivered = harness.claims.delivered.lock().await;
        let flattened: Vec<String> = delivered.iter().flatten().cloned().collect();
        assert!(flattened.contains(&"work_1".to_owned()));
        assert!(flattened.contains(&"work_2".to_owned()));
    }

    #[tokio::test(start_paused = true)]
    async fn removing_a_cancelled_work_does_not_confirm_its_notification() {
        let harness = harness(config());
        harness
            .manager
            .queue(Announcement::completed("work_1", "x", "y"))
            .await;
        settle().await;
        harness.manager.remove("work_1").await;
        settle().await;
        assert!(
            harness.claims.delivered.lock().await.is_empty(),
            "a removed result was never spoken",
        );
        assert!(!harness.manager.has("work_1").await);
    }

    #[tokio::test(start_paused = true)]
    async fn a_closed_manager_speaks_nothing_further() {
        let harness = harness(config());
        harness.manager.close().await;
        harness
            .manager
            .queue(Announcement::completed("work_1", "x", "y"))
            .await;
        settle().await;
        assert!(harness.frontend.spoken.lock().await.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn the_acknowledgement_timeout_re_arms_while_the_gate_is_blocked() {
        let harness = harness(AnnouncementManagerConfig {
            acknowledgement_timeout_ms: 1_000,
            retry_base_ms: 20,
            retry_max_ms: 20,
            ..config()
        });
        harness
            .manager
            .queue(Announcement::completed("work_1", "x", "y"))
            .await;
        // Long enough to deliver, short enough that the acknowledgement window
        // has not yet expired.
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(harness.frontend.spoken.lock().await.len(), 1);

        harness.blocked.store(true, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(4_000)).await;
        assert_eq!(
            harness.frontend.spoken.lock().await.len(),
            1,
            "a second spoken response while blocked is the failure the gate exists for",
        );

        harness.blocked.store(false, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(2_000)).await;
        assert!(
            harness.frontend.spoken.lock().await.len() > 1,
            "and it does redeliver once the gate opens",
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_failure_announcement_carries_the_error_line() {
        let harness = harness(config());
        harness
            .manager
            .queue(Announcement::failed("work_1", "跑测试", "端口被占用"))
            .await;
        settle().await;
        let spoken = harness.frontend.spoken.lock().await;
        assert!(spoken[0].contains("type: task.failed"));
        assert!(spoken[0].contains("error:\n端口被占用"));
    }

    #[tokio::test(start_paused = true)]
    async fn speak_is_used_when_context_injection_is_off() {
        let harness = harness(AnnouncementManagerConfig {
            announce_into_context: false,
            ..config()
        });
        harness
            .manager
            .queue(Announcement::completed("work_1", "x", "y"))
            .await;
        settle().await;
        assert_eq!(harness.frontend.injections.load(Ordering::SeqCst), 0);
        assert_eq!(harness.frontend.spoken.lock().await.len(), 1);
    }
}
