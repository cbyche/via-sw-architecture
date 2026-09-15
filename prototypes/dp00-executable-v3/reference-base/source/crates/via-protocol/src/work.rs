//! The Work record's public shape: its status, its collapsed state, and its
//! kind.
//!
//! Ported from `server/src/task/task-manager.mjs`. All three vocabularies are
//! external contracts — they appear in the `publicTask` payload returned by
//! `GET /api/tasks`, `GET /api/tasks/:id`, the SSE stream
//! `GET /api/tasks/:id/events` and every [`GatewayTaskEvent`] frame, and they
//! are persisted verbatim in `tasks.json`.
//!
//! Upstream carries both a `status` and a `workState` on every record, where
//! `workState` collapses the five active statuses to a single `active`
//! (`task-manager.mjs:58`). Both are on the wire, so both are modelled:
//! [`WorkStatus`] is the authority and [`WorkState`] is its projection, produced
//! by [`WorkStatus::state`].
//!
//! [`GatewayTaskEvent`]: crate::GatewayTaskEvent

use crate::macros::wire_enum;

wire_enum! {
    /// The nine Work statuses.
    ///
    /// External contract, from `server/src/task/task-manager.mjs:7-16`
    /// (`docs/reference/contracts.json`, "Work status values"). Clients branch
    /// on these strings.
    ///
    /// ```text
    /// scheduled → queued → running ───────────────────────► completed
    ///               │         └→ delegated → finalizing ──────┘
    ///               └───────────────► cancelling → cancelled
    ///                                           ↘ failed
    /// ```
    ///
    /// ARGO's nearest equivalent, `tinihost::schedule::JobState`, has four
    /// variants (`Pending`/`Running`/`Done`/`Failed`): no `delegated`, no
    /// `finalizing`, and critically no `cancelling` — so "cancellation is
    /// confirmed, not optimistic" has no representation there at all
    /// (`docs/architecture.md` §4).
    pub enum WorkStatus {
        /// `scheduled` — accepted and waiting for its timer. The only
        /// non-terminal status that is not [`active`](Self::is_active): it
        /// holds no scheduler lane and consumes no concurrency budget.
        ///
        /// Survives a Gateway restart intact.
        Scheduled = "scheduled",
        /// `queued` — admitted to the queue, waiting on the scheduler's global
        /// cap, per-owner cap or lane.
        Queued = "queued",
        /// `running` — the scheduler admitted it and the runner is executing.
        Running = "running",
        /// `delegated` — handed to a backend session. **The scheduler lane is
        /// released here**, so a delegated Work no longer occupies a slot while
        /// the backend works.
        Delegated = "delegated",
        /// `finalizing` — the delegated session reported completion and the
        /// coordinator is composing the result.
        Finalizing = "finalizing",
        /// `cancelling` — a stop was requested and is in flight.
        ///
        /// Cancellation is a *state, not an action*: the Work stays here until
        /// a path confirms the stop, and only then becomes
        /// [`Cancelled`](Self::Cancelled).
        Cancelling = "cancelling",
        /// `completed` — terminal success; `result` is populated.
        Completed = "completed",
        /// `failed` — terminal failure; `error` is populated.
        Failed = "failed",
        /// `cancelled` — terminal, confirmed cancellation. `error` is cleared
        /// and no notification is queued.
        Cancelled = "cancelled",
    }
}

wire_enum! {
    /// The collapsed Work state, published as the `workState` field.
    ///
    /// External contract, from `server/src/task/task-manager.mjs:58`
    /// (`docs/reference/contracts.json`, "workState values"). Upstream computes
    /// it as `ACTIVE.has(status) ? 'active' : status`, so the observable values
    /// are `active`, `scheduled`, `completed`, `failed` and `cancelled` —
    /// [`Scheduled`](Self::Scheduled) reaches the wire because `scheduled` is
    /// not one of the five statuses `ACTIVE` collapses.
    ///
    /// Consumers branch on `workState === 'active'` where they do not care
    /// which of the five active statuses a Work is in.
    pub enum WorkState {
        /// `active` — the Work is in flight: one of [`WorkStatus::Queued`],
        /// [`Running`](WorkStatus::Running), [`Delegated`](WorkStatus::Delegated),
        /// [`Finalizing`](WorkStatus::Finalizing) or
        /// [`Cancelling`](WorkStatus::Cancelling).
        ///
        /// While active, `elapsedMs` is computed live rather than read from
        /// storage.
        Active = "active",
        /// `scheduled` — waiting for its timer; the projection of
        /// [`WorkStatus::Scheduled`].
        Scheduled = "scheduled",
        /// `completed` — terminal success.
        Completed = "completed",
        /// `failed` — terminal failure.
        Failed = "failed",
        /// `cancelled` — terminal, confirmed cancellation.
        Cancelled = "cancelled",
    }
}

wire_enum! {
    /// The Work taxonomy — one record, four kinds.
    ///
    /// External contract, from `server/src/task/task-manager.mjs:361,414` and
    /// `server/src/voice/tools/tool-call-handler.mjs:886`
    /// (`docs/reference/contracts.json`, "Work kind values").
    ///
    /// `docs/architecture.md` §4 calls the unified record **the single biggest
    /// structural improvement the port brings**: one `cancel_agent_task`
    /// cancels a reminder and a delegation alike, where ARGO's equivalents are
    /// three separate registries with three separate cancel paths.
    pub enum WorkKind {
        /// `work` — a delegated objective. The only kind that gets a
        /// progress-check interval.
        Work = "work",
        /// `reminder` — spoken at its due time. Its runner only replays the
        /// stored text, so a reminder that had already fired when the Gateway
        /// stopped is restored as `scheduled` and re-fired as overdue catch-up
        /// rather than being lost.
        Reminder = "reminder",
        /// `scheduled_task` — real work with a due time. The only kind that
        /// gets a hard timeout watchdog.
        ScheduledTask = "scheduled_task",
        /// `control` — an internal control turn (status query, cancellation).
        /// Hidden from listings unless explicitly requested, and **never
        /// forwarded to a client** as a task event.
        Control = "control",
    }
}

impl Default for WorkKind {
    /// [`Work`](WorkKind::Work), matching upstream's
    /// `String(kind || 'work')` (`task-manager.mjs:361`).
    fn default() -> Self {
        Self::Work
    }
}

impl WorkKind {
    /// Whether Work of this kind is visible to clients.
    ///
    /// [`Control`](Self::Control) Work is filtered out of task-event delivery
    /// (`server/src/voice/realtime-gateway.mjs:873`) and hidden from `list()`
    /// unless `includeControl` is set.
    pub const fn is_client_visible(&self) -> bool {
        !matches!(self, Self::Control)
    }
}

impl WorkStatus {
    /// The five statuses upstream's `ACTIVE` set collapses to
    /// [`WorkState::Active`], in declaration order.
    ///
    /// External contract, from `task-manager.mjs:7-13`.
    pub const ACTIVE: &'static [Self] = &[
        Self::Queued,
        Self::Running,
        Self::Delegated,
        Self::Finalizing,
        Self::Cancelling,
    ];

    /// The statuses from which a cancellation may be requested, in declaration
    /// order.
    ///
    /// External contract, upstream's `CANCELLABLE` set
    /// (`task-manager.mjs:14`). Note that [`Cancelling`](Self::Cancelling) is
    /// *not* a member: a second cancel of an already-cancelling Work is
    /// admitted by a separate explicit check and joins the in-flight
    /// cancellation rather than starting a new one.
    pub const CANCELLABLE: &'static [Self] = &[
        Self::Scheduled,
        Self::Queued,
        Self::Running,
        Self::Delegated,
        Self::Finalizing,
    ];

    /// The three terminal statuses, in declaration order.
    ///
    /// External contract, upstream's `TERMINAL` set (`task-manager.mjs:15`).
    pub const TERMINAL: &'static [Self] = &[Self::Completed, Self::Failed, Self::Cancelled];

    /// The statuses a [`WorkKind::Reminder`] may be restored from by replaying
    /// it as [`Scheduled`](Self::Scheduled), in declaration order.
    ///
    /// External contract, upstream's `REPLAYABLE_REMINDER` set
    /// (`task-manager.mjs:16`).
    pub const REPLAYABLE_REMINDER: &'static [Self] = &[Self::Queued, Self::Running];

    /// The collapsed [`WorkState`] published as the record's `workState` field.
    ///
    /// Reproduces `ACTIVE.has(task.status) ? 'active' : task.status`
    /// (`server/src/task/task-manager.mjs:58`).
    pub const fn state(&self) -> WorkState {
        match self {
            Self::Queued
            | Self::Running
            | Self::Delegated
            | Self::Finalizing
            | Self::Cancelling => WorkState::Active,
            Self::Scheduled => WorkState::Scheduled,
            Self::Completed => WorkState::Completed,
            Self::Failed => WorkState::Failed,
            Self::Cancelled => WorkState::Cancelled,
        }
    }

    /// Whether this status is one of the five [`ACTIVE`](Self::ACTIVE) ones.
    pub const fn is_active(&self) -> bool {
        matches!(self.state(), WorkState::Active)
    }

    /// Whether a cancellation may be *requested* from this status.
    ///
    /// See [`CANCELLABLE`](Self::CANCELLABLE) for why
    /// [`Cancelling`](Self::Cancelling) is excluded.
    pub const fn is_cancellable(&self) -> bool {
        matches!(
            self,
            Self::Scheduled | Self::Queued | Self::Running | Self::Delegated | Self::Finalizing
        )
    }

    /// Whether this status is terminal — no further transition is legal.
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// Whether a [`WorkKind::Reminder`] in this status is replayed as
    /// [`Scheduled`](Self::Scheduled) on restore.
    pub const fn is_replayable_reminder(&self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }

    /// Whether `self → next` is a legal lifecycle transition.
    ///
    /// The graph is `docs/architecture.md` §4, reconciled against the
    /// transitions `server/src/task/task-manager.mjs` and
    /// `reminder-scheduler.mjs` actually perform:
    ///
    /// | From | To | Where |
    /// | --- | --- | --- |
    /// | `scheduled` | `queued` | `reminder-scheduler.mjs:66,103` — the timer fires |
    /// | `scheduled` | `cancelled` | `task-manager.mjs:718` — short-circuits straight to confirmed |
    /// | `queued` | `running` | `task-manager.mjs:479` — the scheduler admits it |
    /// | `queued` | `cancelled` | `task-manager.mjs:718` — short-circuits straight to confirmed |
    /// | `running` | `delegated` | `task-manager.mjs:508` — `backend.delegated` |
    /// | `running` | `completed` / `failed` | `task-manager.mjs:657,666` — the runner settled |
    /// | `running` | `cancelling` | `task-manager.mjs:721` |
    /// | `delegated` | `finalizing` | `task-manager.mjs:522` — `backend.delegation.completed` |
    /// | `delegated` | `completed` / `failed` / `cancelling` | the runner settles or a stop is requested |
    /// | `finalizing` | `completed` / `failed` / `cancelling` | as above |
    /// | `cancelling` | `cancelled` | `task-manager.mjs:775` — the stop was confirmed |
    /// | `cancelling` | `failed` | `task-manager.mjs:752` — the canceler itself threw |
    ///
    /// Three consequences worth stating, because each is a *missing* edge that
    /// a plausible implementation would add:
    ///
    /// - **`scheduled` and `queued` never pass through `cancelling`.** Nothing
    ///   has started, so there is nothing to confirm; upstream short-circuits
    ///   to [`Cancelled`](Self::Cancelled).
    /// - **`cancelling` cannot reach [`Completed`](Self::Completed).** A result
    ///   arriving after a stop was requested is dropped, not published.
    /// - **`running` cannot reach [`Finalizing`](Self::Finalizing) directly.**
    ///   Finalizing means "a *delegated* session reported completion", so it is
    ///   only reachable through [`Delegated`](Self::Delegated).
    ///
    /// Crash recovery is deliberately **not** modelled here. `restore()`
    /// rewrites a persisted record (active → `failed`, or a recoverable
    /// `delegated`/`finalizing` → `queued`) as it is read off disk; that is a
    /// store-level rewrite of a record from a dead process, not a transition of
    /// a live one, and treating it as an edge would legalise
    /// `running → queued` for everyone.
    ///
    /// A status is never a legal transition to itself.
    pub const fn can_transition_to(&self, next: Self) -> bool {
        use WorkStatus::{
            Cancelled, Cancelling, Completed, Delegated, Failed, Finalizing, Queued, Running,
            Scheduled,
        };

        matches!(
            (*self, next),
            (Scheduled, Queued)
                | (Scheduled, Cancelled)
                | (Queued, Running)
                | (Queued, Cancelled)
                | (Running, Delegated)
                | (Running, Completed)
                | (Running, Failed)
                | (Running, Cancelling)
                | (Delegated, Finalizing)
                | (Delegated, Completed)
                | (Delegated, Failed)
                | (Delegated, Cancelling)
                | (Finalizing, Completed)
                | (Finalizing, Failed)
                | (Finalizing, Cancelling)
                | (Cancelling, Cancelled)
                | (Cancelling, Failed)
        )
    }

    /// [`can_transition_to`](Self::can_transition_to) as a checked operation.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::IllegalTransition`] when the edge is not in the
    /// graph, so a caller can refuse the change instead of corrupting the
    /// record.
    ///
    /// [`ProtocolError::IllegalTransition`]: crate::ProtocolError::IllegalTransition
    pub fn transition_to(&self, next: Self) -> Result<Self, crate::error::ProtocolError> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err(crate::error::ProtocolError::IllegalTransition {
                from: *self,
                to: next,
            })
        }
    }
}
