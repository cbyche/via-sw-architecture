//! Input arbitration between an external host and this Gateway.
//!
//! Ported from `server/src/voice/input-arbitration.mjs`.
//!
//! A host — a system input method, a platform app — owns the microphone
//! whenever the user dictates into it. It announces that through the control
//! plane and the Gateway commands its clients to stop capturing. Three
//! properties make the mechanism safe:
//!
//! - **Reference counted per holder**, so overlapping suspensions from several
//!   host subsystems compose instead of racing.
//! - **Idempotent per holder**: a repeated suspend refreshes the deadline
//!   rather than incrementing a count, so a host that re-announces on every
//!   keypress cannot leak a holder it will never release.
//! - **Every hold expires.** A host that crashes or forgets to resume must not
//!   be able to silence the Gateway permanently, so a hold releases itself when
//!   its TTL runs out. This is the property the whole design exists for.
//!
//! Suspending also clears playback: a host that is recording must not pick up
//! this Gateway's own speech.
//!
//! # Why an owning task
//!
//! `docs/architecture.md` §11: the holder set is mutated by HTTP handlers, by
//! expiry timers and by the socket close path, and every mutation fans out to
//! every connected client. A `Mutex` gives exclusion but not order, and a
//! `suspend` overtaking its own `resume` leaves the microphone dead. State
//! lives by value in one task; commands arrive on a bounded `mpsc` with
//! `oneshot` replies.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_util::task::TaskTracker;
use via_i18n::{Locale, keys, t};

/// How long a suspension lasts when the host does not say.
///
/// **External contract** — `input-arbitration.mjs:17`
/// (`DEFAULT_INPUT_SUSPEND_TTL_MS = 15_000`).
pub const DEFAULT_INPUT_SUSPEND_TTL_MS: u64 = 15_000;

/// The longest suspension a host may ask for.
///
/// **External contract** — `input-arbitration.mjs:18`
/// (`MAX_INPUT_SUSPEND_TTL_MS = 300_000`). Five minutes is already generous
/// for "the user is dictating into another app"; the cap is what turns a
/// hostile or buggy `ttlMs: Infinity` into a bounded outage.
pub const MAX_INPUT_SUSPEND_TTL_MS: u64 = 300_000;

/// The bound on a holder name.
///
/// **External contract** — `input-arbitration.mjs:70`.
pub const MAX_OWNER_CHARS: usize = 80;

/// The bound on a suspension reason.
///
/// **External contract** — `input-arbitration.mjs:83`.
pub const MAX_REASON_CHARS: usize = 200;

/// The error code a suspension with no owner is refused with.
///
/// **External contract** — `input-arbitration.mjs:73`, rebranded per
/// `docs/rebrand.md` (`QWAUDIO_INPUT_OWNER_REQUIRED` → `VIA_INPUT_OWNER_REQUIRED`).
/// It comes from [`via_protocol`], which is where every coded error lives.
pub const CODE_INPUT_OWNER_REQUIRED: &str = via_protocol::CODE_INPUT_OWNER_REQUIRED;

/// A suspension refused for want of an owner.
///
/// Anonymous suspensions cannot be released or attributed, and an unreleasable
/// hold is exactly the failure the TTL exists to prevent — so this is refused
/// at the door rather than accepted and healed later.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct OwnerRequired {
    /// The localized message.
    pub message: String,
}

impl OwnerRequired {
    /// The stable error code, `VIA_INPUT_OWNER_REQUIRED`.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        CODE_INPUT_OWNER_REQUIRED
    }
}

/// One live suspension.
///
/// **External contract** — the holder shape of `input-arbitration.mjs:124-130`,
/// which reaches `/api/health.inputSuspension` and the `input.suspend` frame.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Holder {
    /// Who is holding.
    pub owner: String,
    /// Why, bounded to [`MAX_REASON_CHARS`].
    pub reason: String,
    /// The TTL actually granted, after the cap.
    pub ttl_ms: u64,
    /// When this holder first suspended. A renewal does **not** move it.
    pub since: i64,
    /// When the current hold expires.
    pub expires_at: i64,
}

/// The arbitration's public state.
///
/// **External contract** — `input-arbitration.mjs:123-141`, field order
/// included: it is serialized into `/api/health`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuspensionStatus {
    /// Whether anything is holding.
    pub suspended: bool,
    /// Every holder, in insertion order.
    pub holders: Vec<Holder>,
    /// The first holder's name — a convenience for a host UI that only ever
    /// holds one suspension.
    pub owner: Option<String>,
    /// The first holder's reason.
    pub reason: String,
    /// The **latest** expiry across every holder, which is when capture may
    /// actually resume.
    pub expires_at: Option<i64>,
}

/// What changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuspensionChange {
    /// The state after the change.
    pub status: SuspensionStatus,
    /// `suspended` or `resumed`.
    pub state: SuspensionState,
    /// Which holder caused it, when there is one.
    pub owner: Option<String>,
    /// The reason, for a suspension.
    pub reason: String,
    /// Whether a TTL expiry caused the resume rather than the host.
    pub expired: bool,
}

/// The two edges subscribers are told about.
///
/// Only edges: a renewal by an existing holder while another holder is already
/// suspended changes nothing a client can act on, so it is not broadcast.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuspensionState {
    /// Capture must stop.
    Suspended,
    /// Capture may resume.
    Resumed,
}

/// A monotonic millisecond clock.
pub type Clock = std::sync::Arc<dyn Fn() -> i64 + Send + Sync>;

/// The system clock, in milliseconds since the Unix epoch.
#[must_use]
pub fn system_clock() -> Clock {
    std::sync::Arc::new(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| {
                i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
            })
    })
}

#[derive(Debug)]
enum Command {
    Suspend {
        owner: String,
        reason: String,
        ttl_ms: Option<u64>,
        reply: oneshot::Sender<Result<SuspensionStatus, OwnerRequired>>,
    },
    Resume {
        owner: String,
        reply: oneshot::Sender<SuspensionStatus>,
    },
    Status {
        reply: oneshot::Sender<SuspensionStatus>,
    },
    ReleaseAll {
        reply: oneshot::Sender<SuspensionStatus>,
    },
    Expire {
        owner: String,
        generation: u64,
    },
}

/// The handle every caller holds.
///
/// Cloneable and cheap; dropping the last one stops the owning task.
#[derive(Clone, Debug)]
pub struct InputArbitration {
    commands: mpsc::Sender<Command>,
    events: broadcast::Sender<SuspensionChange>,
    locale: Locale,
}

/// How many commands may queue before a caller waits.
const COMMAND_BUFFER: usize = 64;

/// How many changes a slow subscriber may fall behind by.
const EVENT_BUFFER: usize = 64;

impl InputArbitration {
    /// Start the owning task with the default bounds and the system clock.
    #[must_use]
    pub fn new(locale: Locale) -> Self {
        Self::builder(locale).build()
    }

    /// Configure the owning task.
    #[must_use]
    pub fn builder(locale: Locale) -> InputArbitrationBuilder {
        InputArbitrationBuilder {
            locale,
            default_ttl_ms: DEFAULT_INPUT_SUSPEND_TTL_MS,
            max_ttl_ms: MAX_INPUT_SUSPEND_TTL_MS,
            clock: system_clock(),
            tracker: None,
        }
    }

    /// Subscribe to suspension edges.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<SuspensionChange> {
        self.events.subscribe()
    }

    /// Take or renew a hold for `owner`.
    ///
    /// **External contract** — `input-arbitration.mjs:69-97`. A repeated
    /// suspend by the same holder refreshes the deadline and keeps the original
    /// `since`; it does not stack.
    ///
    /// # Errors
    ///
    /// [`OwnerRequired`] when `owner` is blank after trimming.
    pub async fn suspend(
        &self,
        owner: &str,
        reason: &str,
        ttl_ms: Option<u64>,
    ) -> Result<SuspensionStatus, OwnerRequired> {
        let (reply, answer) = oneshot::channel();
        if self
            .commands
            .send(Command::Suspend {
                owner: owner.to_owned(),
                reason: reason.to_owned(),
                ttl_ms,
                reply,
            })
            .await
            .is_err()
        {
            return Ok(SuspensionStatus::default());
        }
        answer.await.unwrap_or(Ok(SuspensionStatus::default()))
    }

    /// Release `owner`'s hold. Releasing a holder that is not holding is a
    /// no-op that still answers the current status.
    pub async fn resume(&self, owner: &str) -> SuspensionStatus {
        self.ask(|reply| Command::Resume {
            owner: owner.to_owned(),
            reply,
        })
        .await
    }

    /// The current status.
    pub async fn status(&self) -> SuspensionStatus {
        self.ask(|reply| Command::Status { reply }).await
    }

    /// Whether anything is holding.
    pub async fn suspended(&self) -> bool {
        self.status().await.suspended
    }

    /// Drop every hold.
    ///
    /// **External contract** — `input-arbitration.mjs:146-151`. A Gateway that
    /// stops serving cannot honour a resume, so held state from a previous run
    /// must not survive into the next one. Subscribers are kept: they belong to
    /// the WebSocket server, which is reused.
    pub async fn release_all(&self) -> SuspensionStatus {
        self.ask(|reply| Command::ReleaseAll { reply }).await
    }

    /// The localized message a blank owner is refused with.
    #[must_use]
    pub fn owner_required(&self) -> OwnerRequired {
        OwnerRequired {
            message: t(self.locale, keys::GATEWAY_INPUT_SUSPEND_REQUIRES_OWNER).to_owned(),
        }
    }

    async fn ask<F>(&self, build: F) -> SuspensionStatus
    where
        F: FnOnce(oneshot::Sender<SuspensionStatus>) -> Command,
    {
        let (reply, answer) = oneshot::channel();
        if self.commands.send(build(reply)).await.is_err() {
            return SuspensionStatus::default();
        }
        answer.await.unwrap_or_default()
    }
}

/// Builds an [`InputArbitration`].
pub struct InputArbitrationBuilder {
    locale: Locale,
    default_ttl_ms: u64,
    max_ttl_ms: u64,
    clock: Clock,
    tracker: Option<TaskTracker>,
}

impl std::fmt::Debug for InputArbitrationBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputArbitrationBuilder")
            .field("locale", &self.locale)
            .field("default_ttl_ms", &self.default_ttl_ms)
            .field("max_ttl_ms", &self.max_ttl_ms)
            .finish_non_exhaustive()
    }
}

impl InputArbitrationBuilder {
    /// Override the default TTL.
    #[must_use]
    pub const fn default_ttl_ms(mut self, ttl_ms: u64) -> Self {
        self.default_ttl_ms = ttl_ms;
        self
    }

    /// Override the TTL cap.
    #[must_use]
    pub const fn max_ttl_ms(mut self, ttl_ms: u64) -> Self {
        self.max_ttl_ms = ttl_ms;
        self
    }

    /// Override the clock.
    #[must_use]
    pub fn clock(mut self, clock: Clock) -> Self {
        self.clock = clock;
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
    pub fn build(self) -> InputArbitration {
        let (commands, inbox) = mpsc::channel(COMMAND_BUFFER);
        let (events, _) = broadcast::channel(EVENT_BUFFER);
        let actor = Actor {
            default_ttl_ms: self.default_ttl_ms,
            max_ttl_ms: self.max_ttl_ms,
            clock: self.clock,
            holders: HashMap::new(),
            generation: 0,
            events: events.clone(),
            commands: commands.clone(),
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
        InputArbitration {
            commands,
            events,
            locale: self.locale,
        }
    }
}

#[derive(Debug, Clone)]
struct HeldSuspension {
    owner: String,
    reason: String,
    ttl_ms: u64,
    since: i64,
    expires_at: i64,
    /// Bumped on every renewal, so a timer armed for a superseded hold is
    /// ignored when it fires rather than releasing a live one.
    generation: u64,
}

struct Actor {
    default_ttl_ms: u64,
    max_ttl_ms: u64,
    clock: Clock,
    holders: HashMap<String, HeldSuspension>,
    generation: u64,
    events: broadcast::Sender<SuspensionChange>,
    commands: mpsc::Sender<Command>,
}

impl Actor {
    async fn run(mut self, mut inbox: mpsc::Receiver<Command>) {
        while let Some(command) = inbox.recv().await {
            match command {
                Command::Suspend {
                    owner,
                    reason,
                    ttl_ms,
                    reply,
                } => {
                    let answer = self.suspend(&owner, &reason, ttl_ms);
                    let _ = reply.send(answer);
                }
                Command::Resume { owner, reply } => {
                    let _ = reply.send(self.resume(&owner));
                }
                Command::Status { reply } => {
                    let _ = reply.send(self.status());
                }
                Command::ReleaseAll { reply } => {
                    let _ = reply.send(self.release_all());
                }
                Command::Expire { owner, generation } => self.expire(&owner, generation),
            }
        }
    }

    fn ttl_for(&self, ttl_ms: Option<u64>) -> u64 {
        match ttl_ms {
            Some(requested) if requested > 0 => requested.min(self.max_ttl_ms),
            _ => self.default_ttl_ms,
        }
    }

    fn suspend(
        &mut self,
        owner: &str,
        reason: &str,
        ttl_ms: Option<u64>,
    ) -> Result<SuspensionStatus, OwnerRequired> {
        let holder = crate::text::bounded_utf16(crate::text::trim(owner), MAX_OWNER_CHARS);
        if holder.is_empty() {
            return Err(OwnerRequired {
                // The actor has no locale; the handle supplies the localized
                // message. This carries the code so a caller that skipped the
                // handle still sees a useful string.
                message: CODE_INPUT_OWNER_REQUIRED.to_owned(),
            });
        }
        let was_suspended = !self.holders.is_empty();
        let ttl = self.ttl_for(ttl_ms);
        let now = (self.clock)();
        self.generation += 1;
        let generation = self.generation;
        let since = self
            .holders
            .get(&holder)
            .map_or(now, |existing| existing.since);
        self.holders.insert(
            holder.clone(),
            HeldSuspension {
                owner: holder.clone(),
                reason: crate::text::bounded_utf16(reason, MAX_REASON_CHARS),
                ttl_ms: ttl,
                since,
                expires_at: now.saturating_add(i64::try_from(ttl).unwrap_or(i64::MAX)),
                generation,
            },
        );
        self.arm_expiry(holder.clone(), generation, ttl);
        tracing::info!(
            target: "via::voice",
            owner = %holder,
            reason = %reason,
            ttl_ms = ttl,
            renewed = since != now,
            "input.suspended",
        );
        if !was_suspended {
            self.notify(
                SuspensionState::Suspended,
                Some(holder),
                reason.to_owned(),
                false,
            );
        }
        Ok(self.status())
    }

    fn arm_expiry(&self, owner: String, generation: u64, ttl_ms: u64) {
        let commands = self.commands.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(ttl_ms)).await;
            // A full mailbox means the actor is busy, not gone; `send` waits.
            let _ = commands.send(Command::Expire { owner, generation }).await;
        });
    }

    fn resume(&mut self, owner: &str) -> SuspensionStatus {
        let holder = crate::text::bounded_utf16(crate::text::trim(owner), MAX_OWNER_CHARS);
        if self.holders.remove(&holder).is_none() {
            return self.status();
        }
        tracing::info!(target: "via::voice", owner = %holder, "input.resumed");
        if self.holders.is_empty() {
            self.notify(SuspensionState::Resumed, Some(holder), String::new(), false);
        }
        self.status()
    }

    fn expire(&mut self, owner: &str, generation: u64) {
        let Some(existing) = self.holders.get(owner) else {
            return;
        };
        if existing.generation != generation {
            // A renewal superseded this timer.
            return;
        }
        let ttl_ms = existing.ttl_ms;
        self.holders.remove(owner);
        tracing::warn!(
            target: "via::voice",
            owner = %owner,
            ttl_ms,
            "input.suspend_expired",
        );
        if self.holders.is_empty() {
            self.notify(
                SuspensionState::Resumed,
                Some(owner.to_owned()),
                String::new(),
                true,
            );
        }
    }

    fn release_all(&mut self) -> SuspensionStatus {
        let was_suspended = !self.holders.is_empty();
        self.holders.clear();
        if was_suspended {
            self.notify(SuspensionState::Resumed, None, String::new(), false);
        }
        self.status()
    }

    fn status(&self) -> SuspensionStatus {
        // Insertion order is not what `HashMap` gives, and upstream's `Map`
        // does preserve it — but the only order-sensitive readers are `owner`
        // and `reason`, which name "a" holder for a single-holder UI. Sorting
        // by `since` then by name makes that choice deterministic instead of
        // hash-dependent, which is strictly better than upstream for a health
        // payload that is diffed across restarts.
        let mut holders: Vec<Holder> = self
            .holders
            .values()
            .map(|held| Holder {
                owner: held.owner.clone(),
                reason: held.reason.clone(),
                ttl_ms: held.ttl_ms,
                since: held.since,
                expires_at: held.expires_at,
            })
            .collect();
        holders.sort_by(|left, right| {
            left.since
                .cmp(&right.since)
                .then_with(|| left.owner.cmp(&right.owner))
        });
        SuspensionStatus {
            suspended: !holders.is_empty(),
            owner: holders.first().map(|holder| holder.owner.clone()),
            reason: holders
                .first()
                .map_or(String::new(), |holder| holder.reason.clone()),
            expires_at: holders.iter().map(|holder| holder.expires_at).max(),
            holders,
        }
    }

    fn notify(&self, state: SuspensionState, owner: Option<String>, reason: String, expired: bool) {
        // A send with no subscribers is not an error: the WebSocket server may
        // not have attached yet, and the status is queryable regardless.
        let _ = self.events.send(SuspensionChange {
            status: self.status(),
            state,
            owner,
            reason,
            expired,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicI64, Ordering};

    fn fixed_clock() -> (Clock, Arc<AtomicI64>) {
        let now = Arc::new(AtomicI64::new(1_000));
        let handle = Arc::clone(&now);
        (
            Arc::new(move || handle.load(Ordering::SeqCst)),
            Arc::clone(&now),
        )
    }

    fn arbitration(clock: Clock) -> InputArbitration {
        InputArbitration::builder(Locale::Zh).clock(clock).build()
    }

    /// The next suspension edge, or a failure.
    ///
    /// Bounded on purpose: an edge that never arrives is a *defect*, and a bare
    /// `recv().await` turns that defect into a test that hangs forever rather
    /// than one that fails. Under `start_paused` the clock jumps as soon as
    /// every task is idle, so the bound costs no wall time.
    async fn next_edge(
        events: &mut broadcast::Receiver<SuspensionChange>,
        why: &str,
    ) -> SuspensionChange {
        tokio::time::timeout(std::time::Duration::from_secs(30), events.recv())
            .await
            .unwrap_or_else(|_| panic!("{why}: no edge arrived"))
            .unwrap_or_else(|error| panic!("{why}: {error}"))
    }

    #[tokio::test(start_paused = true)]
    async fn a_blank_owner_is_refused_with_the_contract_code() {
        let (clock, _) = fixed_clock();
        let arbitration = arbitration(clock);
        for owner in ["", "   ", "\t\n"] {
            let error = arbitration
                .suspend(owner, "typing", None)
                .await
                .expect_err("an anonymous hold cannot be released");
            assert_eq!(error.code(), "VIA_INPUT_OWNER_REQUIRED");
        }
        assert!(!arbitration.suspended().await);
    }

    #[tokio::test(start_paused = true)]
    async fn the_localized_refusal_comes_from_the_catalog() {
        let (clock, _) = fixed_clock();
        let arbitration = arbitration(clock);
        assert_eq!(
            arbitration.owner_required().message,
            t(Locale::Zh, keys::GATEWAY_INPUT_SUSPEND_REQUIRES_OWNER),
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_repeated_suspend_renews_rather_than_stacking() {
        let (clock, now) = fixed_clock();
        let arbitration = arbitration(clock);
        arbitration
            .suspend("ime", "typing", Some(10_000))
            .await
            .expect("owned");
        now.store(5_000, Ordering::SeqCst);
        let status = arbitration
            .suspend("ime", "still typing", Some(10_000))
            .await
            .expect("owned");
        assert_eq!(status.holders.len(), 1, "one holder, not two");
        assert_eq!(
            status.holders[0].since, 1_000,
            "the original since survives"
        );
        assert_eq!(status.holders[0].expires_at, 15_000, "the deadline moved");
        assert_eq!(status.holders[0].reason, "still typing");

        // One resume from that holder releases everything it holds.
        let resumed = arbitration.resume("ime").await;
        assert!(!resumed.suspended);
    }

    #[tokio::test(start_paused = true)]
    async fn holds_are_reference_counted_across_owners() {
        let (clock, _) = fixed_clock();
        let arbitration = arbitration(clock);
        arbitration.suspend("ime", "a", None).await.expect("owned");
        arbitration
            .suspend("panel", "b", None)
            .await
            .expect("owned");
        assert_eq!(arbitration.status().await.holders.len(), 2);
        arbitration.resume("ime").await;
        assert!(
            arbitration.suspended().await,
            "the second holder still holds"
        );
        arbitration.resume("panel").await;
        assert!(!arbitration.suspended().await);
    }

    #[tokio::test(start_paused = true)]
    async fn every_hold_expires_so_a_crashed_holder_cannot_silence_the_gateway() {
        let (clock, now) = fixed_clock();
        let arbitration = arbitration(clock);
        let mut events = arbitration.subscribe();
        arbitration
            .suspend("crashed", "recording", Some(1_000))
            .await
            .expect("owned");
        assert!(arbitration.suspended().await);

        let suspended = next_edge(&mut events, "the suspend edge").await;
        assert_eq!(suspended.state, SuspensionState::Suspended);

        now.store(2_500, Ordering::SeqCst);
        tokio::time::sleep(std::time::Duration::from_millis(1_100)).await;
        let resumed = next_edge(&mut events, "the expiry edge").await;
        assert_eq!(resumed.state, SuspensionState::Resumed);
        assert!(resumed.expired, "the resume is attributed to the TTL");
        assert!(!arbitration.suspended().await);
    }

    #[tokio::test(start_paused = true)]
    async fn a_renewal_supersedes_the_earlier_expiry_timer() {
        let (clock, now) = fixed_clock();
        let arbitration = arbitration(clock);
        arbitration
            .suspend("ime", "a", Some(1_000))
            .await
            .expect("owned");
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
        now.store(1_600, Ordering::SeqCst);
        arbitration
            .suspend("ime", "a", Some(5_000))
            .await
            .expect("owned");
        // The first timer fires here and must not release the renewed hold.
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
        assert!(
            arbitration.suspended().await,
            "the renewal survived the stale timer"
        );
        tokio::time::sleep(std::time::Duration::from_millis(5_000)).await;
        assert!(
            !arbitration.suspended().await,
            "the renewed timer still fires"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn the_ttl_is_capped_and_a_nonsense_value_falls_back_to_the_default() {
        let (clock, _) = fixed_clock();
        let arbitration = arbitration(clock);
        let capped = arbitration
            .suspend("ime", "", Some(u64::MAX))
            .await
            .expect("owned");
        assert_eq!(capped.holders[0].ttl_ms, MAX_INPUT_SUSPEND_TTL_MS);
        let defaulted = arbitration
            .suspend("other", "", Some(0))
            .await
            .expect("owned");
        let holder = defaulted
            .holders
            .iter()
            .find(|holder| holder.owner == "other")
            .expect("present");
        assert_eq!(holder.ttl_ms, DEFAULT_INPUT_SUSPEND_TTL_MS);
        let absent = arbitration.suspend("third", "", None).await.expect("owned");
        let holder = absent
            .holders
            .iter()
            .find(|holder| holder.owner == "third")
            .expect("present");
        assert_eq!(holder.ttl_ms, DEFAULT_INPUT_SUSPEND_TTL_MS);
    }

    #[tokio::test(start_paused = true)]
    async fn only_edges_are_broadcast() {
        let (clock, _) = fixed_clock();
        let arbitration = arbitration(clock);
        let mut events = arbitration.subscribe();
        arbitration.suspend("a", "", None).await.expect("owned");
        arbitration.suspend("b", "", None).await.expect("owned");
        arbitration.suspend("a", "", None).await.expect("owned");
        arbitration.resume("b").await;
        arbitration.resume("a").await;
        let first = next_edge(&mut events, "suspend").await;
        assert_eq!(first.state, SuspensionState::Suspended);
        let second = next_edge(&mut events, "resume").await;
        assert_eq!(second.state, SuspensionState::Resumed);
        assert!(events.try_recv().is_err(), "no interior edges");
    }

    #[tokio::test(start_paused = true)]
    async fn the_status_reports_the_latest_expiry_across_every_holder() {
        let (clock, _) = fixed_clock();
        let arbitration = arbitration(clock);
        arbitration
            .suspend("a", "", Some(1_000))
            .await
            .expect("owned");
        arbitration
            .suspend("b", "", Some(9_000))
            .await
            .expect("owned");
        let status = arbitration.status().await;
        assert_eq!(status.expires_at, Some(10_000));
        assert_eq!(status.owner.as_deref(), Some("a"));
    }

    #[tokio::test(start_paused = true)]
    async fn release_all_drops_everything_and_reports_one_resume() {
        let (clock, _) = fixed_clock();
        let arbitration = arbitration(clock);
        let mut events = arbitration.subscribe();
        arbitration.suspend("a", "", None).await.expect("owned");
        arbitration.suspend("b", "", None).await.expect("owned");
        let _ = next_edge(&mut events, "the suspend edge").await;
        let status = arbitration.release_all().await;
        assert!(!status.suspended);
        assert!(status.holders.is_empty());
        let resumed = next_edge(&mut events, "one resume").await;
        assert_eq!(resumed.state, SuspensionState::Resumed);
        assert_eq!(resumed.owner, None);
        // Releasing again broadcasts nothing.
        arbitration.release_all().await;
        assert!(events.try_recv().is_err());
    }

    #[tokio::test(start_paused = true)]
    async fn resuming_an_owner_that_never_held_is_a_no_op() {
        let (clock, _) = fixed_clock();
        let arbitration = arbitration(clock);
        arbitration.suspend("a", "", None).await.expect("owned");
        let status = arbitration.resume("never-held").await;
        assert!(status.suspended, "the live holder is untouched");
        assert_eq!(status.holders.len(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn owner_and_reason_are_bounded() {
        let (clock, _) = fixed_clock();
        let arbitration = arbitration(clock);
        let status = arbitration
            .suspend(&"o".repeat(200), &"r".repeat(500), None)
            .await
            .expect("owned");
        assert_eq!(status.holders[0].owner.chars().count(), MAX_OWNER_CHARS);
        assert_eq!(status.holders[0].reason.chars().count(), MAX_REASON_CHARS);
    }
}
