# Phase 3 deviations

Layer 2 — the Work subsystem, the coordinator, and the MCP coordination tools.

Read [`phase-0.md`](phase-0.md) for what a deviation record is and is not.


## `via-work`

*234 tests · clippy clean · 32/32 mutations killed · 14 deviations*

Ported from
`server/src/task/{task-manager,task-scheduler,task-store,reminder-scheduler}.mjs`
and `server/src/conversation/task-result-projector.mjs`, against
`server/test/{task-manager,task-manager-scheduled,task-store,task-result-projector,result-delivery,reminder-scheduler,progress-check,offline-notifications}.test.mjs`.

`via_protocol::{WorkStatus, WorkState, WorkKind}` and its 17-edge
`can_transition_to` graph already shipped and are used, not restated;
`via_store::VersionedJsonStore` is `tasks.json`'s whole implementation;
`via_downstream::SessionEvent` is the activity vocabulary and
`via_downstream::CancelOutcome` is what a cancellation answers in.

### The two things this crate exists to hold apart

**A Work record carries no backend.** The user request, the timestamps, the
final result or error, generic tool activity, one bounded pending permission,
and the notification state. The delegation's `id`, `sessionId` and `directory`
are on the *persisted* record because restart recovery needs them, and are
dropped by `DelegationRef::to_public`;
`tests/delegation.rs::a_delegations_ids_never_reach_a_client` walks every event
of a delegated Work's life and asserts none of the three appears in any of
them.

**Cancellation is a state.** `cancelling` is where a Work sits until something
*confirms* the stop. That is the deliberate divergence below, and
`tests/cancellation.rs::a_deaf_runner_leaves_the_work_cancelling_rather_than_lying`
is the case that names it: upstream would report `cancelled`.

### Deviations

- **Cancellation is confirmed, not optimistic.** Upstream's `cancel()` calls
  `finishCancellation` as soon as its canceler returns
  (`task-manager.mjs:740`), and the canceler's `session/cancel` is an ACP
  *notification* — fire and forget. VIA finishes only when the canceler
  reported `CancelOutcome::Confirmed` (the coordinator route: the model's own
  cancel tool returned) or the abort it raised actually settled the runner (the
  local route). Until then the Work is `cancelling`, which clients can see.
  `WorkManager::confirm_cancellation` is the out-of-band third path, for the
  adapter route where the backend's own `stopReason` arrives later. This is
  `docs/architecture.md` §4's *"cancellation is confirmed, not optimistic"* and
  matches the divergence `via-downstream` already recorded for
  `CancelOutcome::Requested`. Five tests, including a runner that never notices
  its abort.

- **A timed-out `scheduled_task` resolves its waiters.** Upstream's watchdog
  path (`task-manager.mjs:555-576`) emits `task.failed` and
  `task.notification.pending` but never calls `task.resolve`, and
  `terminalHandled` then makes the runner's own `finally` return early — so
  every `wait()` on a timed-out scheduled task hangs for the life of the
  process. Fixed; asserted in
  `tests/scheduled.rs::the_watchdog_aborts_before_it_force_fails`.

- **`task.scheduled.fired` is emitted on both fire paths.** Upstream emits it
  from `restoreOverdue` (`reminder-scheduler.mjs:67`) and *not* from `fire()`
  (`:98-113`), so a reminder that fires on time is invisible to every client
  while one that fires as restart catch-up is not. The event is a declared
  `GatewayTaskEvent` and the two paths are the same transition, so VIA emits it
  from both. The catalogue's own entry cites both line numbers.

- **One `Sleep` for the overdue backlog too.** Upstream arms N extra
  `setTimeout`s for the overdue queue (`reminder-scheduler.mjs:62-72`) and its
  single timer deliberately ignores anything already due (`:86`,
  `schedule.at > now`). VIA expresses the stagger as a *deadline override* —
  an overdue Work's effective due time becomes `now + index × stagger` — and
  one `Sleep` serves both. Same order, same spacing, one timer.
  `ReminderScheduler::rearms()` makes "never polls" assertable:
  `tests/reminder.rs` proves a day passes on ≤ 8 recomputations.

- **A `RunnerEvent` for a terminal Work is dropped.** Upstream's `onEvent`
  closure outlives the runner and has no terminal guard, so a late
  `backend.permission.requested` puts a pending permission back onto a
  *completed* record (`task-manager.mjs:492-496` runs before the
  cancelling check and has no `terminalHandled` test). Tested both ways in
  `tests/lifecycle.rs::a_late_permission_never_lands_on_settled_work` and
  `tests/transitions.rs::nothing_moves_a_terminal_work`.

- **Every status change goes through `WorkStatus::transition_to`, and a refused
  edge is dropped rather than applied.** Upstream assigns `task.status`
  directly in eleven places. Three consequences that upstream does not have:
  `backend.delegated` from `finalizing` is refused,
  `backend.delegation.completed` from `running` is refused (there is no
  `running → finalizing` edge), and a second completion for the same
  delegation does not re-finalize. Each is a test in `tests/delegation.rs`.
  A refusal logs `task.illegal_transition` at `warn`.

- **Delegation correlation is enforced on the record, not only in the
  adapter.** `DelegationRef::correlates_with` gates every
  `backend.delegation.completed`: the delegation id must match, and the session
  id must match when both sides carry one. Upstream's `task-manager.mjs`
  overwrites `task.delegation` with whatever it is handed and relies on the
  adapter's `delegatedWorkRuns` map to have correlated first. `docs/architecture.md`
  §11 invariant 4 asks for the property, so it is stated where the Work is.

- **`recover_delegated`'s refusal is a restore-time rewrite, not a transition.**
  `queued → failed` is not an edge in `via-protocol`'s graph, and the graph's
  own documentation excludes crash recovery for exactly this reason. The record
  is from a dead process until `can_recover` accepts it, so the field is set
  directly.

- **A persisted record with no usable `id` is dropped on load.** Upstream's
  filter is only `task && typeof task === 'object'`
  (`task-store.mjs:84`), so a record missing `id` becomes
  `this.tasks.set(undefined, task)`. Dropping it is not a quarantine —
  the rest of the file still loads — and is asserted in
  `src/store.rs::a_non_object_or_idless_entry_is_dropped`.

- **`PersistedWork::result_metadata` is raw JSON, not a typed
  `PublicResultMetadata`.** A typed field would refuse the legacy
  `{decision: {presentation}}` shape that already-persisted `tasks.json` files
  carry, and refusing it would drop the whole record. It is re-projected on
  read — the projection handles both shapes and is idempotent — and
  re-persisted in the new shape.

- **Subscription is a `tokio::sync::broadcast`, so a slow observer can lag.**
  Upstream's listener set is synchronous and cannot lag; it also cannot tell a
  slow observer that it missed something. A lagged receiver is told, and the
  reminder scheduler treats a lag as "recompute". Upstream's *"one observer
  must not break the work queue"* is kept: a send with no receivers is not an
  error.

- **`create` and `create_scheduled` return `Result<_, ManagerStopped>`.**
  Every other method answers `None`/empty on a stopped manager, because a
  stopped manager genuinely has nothing. A *submission* is the one call that
  must not answer plausibly: the caller would tell the model its Work was
  accepted and nothing would ever run. `VIA_WORK_MANAGER_STOPPED` carries the
  `VIA_WORK_` prefix rather than a bare `VIA_` one so it is never mistaken for
  an inherited contract.

- **The six `tasks.json` warnings are built from `via-i18n` keys rather than
  from `via_store::StoreMessages::TASK_STORE`.** `StoreMessages` holds plain
  `fn` pointers, which cannot capture a locale — but a `fn` *can* name a
  catalog key, so `store::task_store_messages(locale)` supplies one set per
  locale. The `zh` column renders the catalogued sentences byte for byte
  (`tests/contracts.rs::the_six_store_warnings_render_the_catalogued_sentences`);
  `en` and `ko` are their authored peers, which `via-acp`'s use of
  `StoreMessages::DEFAULT` does not yet have.

- **`text::slice_units` exists beside `via_downstream::text::bounded`.**
  `bounded` collapses whitespace runs before cutting, because it is the
  sanitising bound on Layer 3's public progress surface. The Work record's own
  bounds are plain `String.prototype.slice` (`task-manager.mjs:79,84`), and
  reusing `bounded` would rewrite a delegation title the coordinator chose.
  Both count UTF-16 code units and both stop one character short of splitting a
  surrogate pair, which is the trade `via-downstream` already recorded.

### Not deviations, recorded so they are not mistaken for one

- `notificationDeliveredAt` is **absent** from a fresh record's JSON rather than
  `null`, because upstream never initialises it and `JSON.stringify` omits an
  `undefined`. Every other absent field is an explicit `null`.
  `tests/contracts.rs` asserts 24 keys before a delivery and 25 after.
- `task.created` is in the catalogued info-log list and is never emitted;
  `create()` emits `task.accepted`. `LOG_INFO_EVENT_NAMES` keeps it so the
  catalogued list still round-trips, and no `WorkEventKind` maps to it.
- The receipt `create()` returns always says `queued`, even for Work the
  scheduler starts in the same call — upstream defers `drain()` to a
  microtask, and a receipt that said `running` would be a different contract.
- Lane and priority are not persisted, so lane serialization does not survive a
  restart. That is the catalogued consequence, not an omission.
- `scripts/risky_unwrap.py` gained one entry: `manager` is listed under
  `KNOWN_NON_ADVERSARIAL_DIRS`, with the rationale that every byte that could
  be adversarial is parsed by `via-store` or `via-downstream` before it reaches
  the owning task. The audit total is unchanged (0).


## `via-coordinator`

*186 tests · clippy clean · 22/22 mutations killed · 15 deviations*

Ported from
`server/src/agent/{coordinator,backend-agent-instructions,permission-broker,keyed-serial-executor}.mjs`
and the **coordination half** of `server/src/agent/acp-backend-adapter.mjs`,
against `server/test/{coordinator,acp-backend-adapter,session-permission-policy}.test.mjs`.

### What is called rather than re-implemented

`via-acp` already ships four of `acp-backend-session-utils.mjs`'s functions —
`parse_coordinator_payload`, `native_tool_output`,
`normalize_coordinator_content`, `coordinator_presentation` — and
`permission-broker.mjs`'s option-kind mapping (`APPROVE_KIND_ORDER` /
`REJECT_KIND_ORDER`, `select_option`, `reply`). All five are catalogued
algorithms, so this crate **depends on `via-acp`** and calls them. A second copy
would be a second answer to *"which malformed model output is accepted"* and
*"which option expresses this decision"*. The edge is `layer2 -> layer3`, which
`via-arch-test`'s adjacency table already allows (upstream's `task -> agent`
row).

Likewise: `via_mcp_tools::is_session_tool` is the permission broker's whole
allow-list; `via_work::{PendingPermission, Presentation, InlineBlock,
InlineFormat, CancellationFact, ReconciliationLedger, NewWork,
coordinator_lane, text::slice_units}` are used as-is;
`via_conversation::{MemoryDocument, Message}` are the exact shapes upstream's
`buildCoordinatorPrompt` reads; `via_core::memory_scopes` decides which document
is a preference and which is a memory.

### Deviations

- **The catalogue says the backend instructions are "16 lines"; the catalogued
  value has fifteen.** Upstream's array literal runs from source line 2 to line
  16 and line 17 is the `join`. The **value** is treated as authoritative and
  `BACKEND_AGENT_INSTRUCTION_LINE_COUNT` is 15, with
  `tests/contracts.rs::the_backend_instructions_are_the_catalogued_text_rebranded`
  counting the catalogued text rather than trusting either number. Same class
  as `via-acp`'s "47 OS names / 39 listed", and resolved the same way.

- **`parseCoordinatorDecision`'s `task` and `targetSession` are not
  reproduced.** Upstream returns them unconditionally `null` at every call site
  and nothing reads them; `CoordinatorDecision` carries `work_id` and
  `presentation`, and `state`/`mode` are `const fn`s because by the time a
  decision is parsed the ladder has already refused every other state.

- **`resultEnvelope.raw` is not reproduced.** Upstream carries the whole
  `session/prompt` result. `via_downstream::PromptOutcome` does not expose one —
  the transport has already reduced it to content plus a stop reason — so
  `ResultEnvelope::stop_reason` carries what survived.

- **`is_busy` is the lane's depth, not a set of in-flight session ids.**
  Upstream's `activeCoordinatorTurns` counts only turns that have *started*;
  the lane also counts one that is queued. The difference makes an urgent cancel
  take the transport route slightly more often rather than slightly less, which
  is the safe side of the trade — upstream's own comment on that fallback is
  *"cancellation is urgent"*.

- **`CancelOutcome::Requested` on the coordinator route is promoted to
  `Confirmed` with a caller-supplied timestamp.** Upstream reports `cancelled`
  from both routes; `via-downstream` already refuses to call an unconfirmed
  cancel done (`docs/deviations/phase-2.md`). The coordinator route *is* a
  confirmation — the model's own cancel tool returned — so it is the one route
  allowed to confirm, and only through `CancelOutcome::confirm`.

- **`PermissionBroker::respond` takes a typed decision.** Upstream's broker is
  looser than its route: `const approved = decision === 'always'` treats every
  other value as a rejection, and the catalogued 400 lives in
  `gateway-application.mjs`. `PermissionResponse::from_wire` moves the refusal to
  the boundary the 400 is already at, so there is no path by which a typo
  silently denies.

- **`PermissionBroker::request` answers in `via_acp::PermissionDecision`, not in
  an ACP reply.** The kind-preference order is `via-acp`'s and is the same
  answer for both call sites; the broker decides *what* and `via-acp` decides
  *which option says it*.

- **A permission request carries its scope, rather than reading it off a
  mutable session object.** Upstream sets `session.permissionScopeId` before a
  prompt and clears it in a `finally` guarded by *"if it is still the current
  value"*. `PermissionContext` is passed per request, so a request cannot be
  re-scoped after the fact; the guard survives only where it still matters —
  `Coordinator::leave_scope`, which is the map of *active* prompts.

- **`NativeToolUpdate` is not `via_downstream::RawSessionUpdate`.** That type is
  the public progress surface's input and deliberately has nowhere to put a
  `rawOutput` — which is exactly the field a session id hides in. Detection reads
  it and publishes none of it. Its merge treats an explicit JSON `null` as
  absent, matching `RawSessionUpdate::merged_over`; the five merged fields are
  each read in a way that cannot tell the two apart.

- **`SESSION_ID_KEYS` is the adapter's order, not the traversal's.** Upstream
  has two nearly-identical `||` chains — `acp-backend-adapter.mjs:671-675` reads
  `childSessionKey || sessionKey || session_id || sessionId` and
  `acp-backend-session-utils.mjs:159-164` tests
  `… || sessionId || session_id`. They disagree only for output carrying both
  spellings with different values. Both are reproduced, in their own crates, and
  `native::tests::the_session_id_key_order_is_the_adapters_not_the_traversals`
  pins the difference.

- **`work_lines`' `'unknown'` status fallback is unrepresentable.** Upstream
  writes `clean(task.status) || 'unknown'`; `via_work::PublicWork::status` is a
  `via_protocol::WorkStatus`, which always spells itself. The arm is dropped
  rather than shipped as dead code.

- **`DelegationRegistry` is a `Mutex`, not an owning task.** `docs/architecture.md`
  §11 asks for an owning task where **order** is the contract. This is a lookup
  table: every operation is one insert, remove or field write with no await
  inside the critical section. The three things here that *are* ordering
  invariants each have their own owning task — the coordinator lane and the
  target lane in `executor`, the pending/resolved ledger in `permission`.

- **`KeyedSerialExecutor` releases on an unbounded channel.** Commands are
  bounded as §11 asks; a release is sent from `Drop`, which cannot await, and a
  bounded `try_send` that hit a full channel would drop the release and wedge
  the lane forever. The release channel carries one `String` per outstanding
  permit and is bounded in practice by the number of live lanes.

- **The MCP tool context is built per turn and registered by the caller.**
  Upstream caches one `AcpSessionToolServer` registration per owner and
  `update()`s its context each turn, because an ACP agent may cache the first
  MCP connection for a Session and the descriptor has to stay stable
  (`acp-backend-adapter.mjs:398-422`). `Coordinator::tool_context` hands back a
  fresh `Arc<dyn SessionToolContext>` and leaves the caching to whoever owns the
  `via_mcp_tools::SessionToolServer` — which is where
  `SessionToolRegistration::update` already lives, and which is the only place
  that knows whether the descriptor has been handed to a session yet.

- **`ProjectSessionDirectory` is a seam.** Upstream reaches straight for
  `client.listSessions()`, which is an ACP call; `DownstreamAgent` has no such
  method because not every harness shape has a session list. A harness that
  cannot list answers with no sessions and no directory, which makes
  `via_session_send` refuse with the catalogued
  `… Session 的项目目录未知 …` rather than resume a Session into a guessed
  directory.

### Not deviations, recorded so they are not mistaken for one

- The coordinator prompt keeps **two blank lines** where the `user_preferences`
  note and the `trusted_backend_event` note would be when neither applies.
  Upstream builds the prompt as an array whose two conditional entries evaluate
  to `''` and `join('\n')`s it; the blank lines are what the model sees.
  Asserted in `envelope::tests::the_two_conditional_notes_leave_their_blank_line_behind`.
- The non-`markdown` branch of the preferences/memory renderer is reproduced
  even though `MemoryDocument::new` always stamps `markdown`. The field is
  public, so the branch is reachable, and it is one `?:` in a catalogued line.
- `IntentSet` is `docs/architecture.md` §4's widening, and a one-element set is
  byte-identical to the single-objective path **including its submission key** —
  which is what `via-work`'s duplicate-submission suppression is keyed on. A
  fan-out suffixes every key, first included, so a fanned-out key can never be
  mistaken for a single-objective one.
- One `via-i18n` key was added: `coordinator.not_final_result`, for upstream's
  `Coordinator did not return a final result (state=…)`. It becomes `task.error`,
  is persisted and is spoken, so it cannot stay developer English. Its `zh` value
  is Chinese prose, so `NOT_CHINESE_PROSE` is untouched. The other four bare
  `Error`s in `coordinator.mjs` reuse keys `via-acp` and `via-downstream`
  already ship.
- `docs/architecture.md` §4's *"the Scheduler admits each intent
  independently"* and §11's *"one item per owner inside the backend session at a
  time"* are not in tension: every intent is its own Work with its own
  `work_id`, cancellation scope and retention, and they share one **lane**,
  which is a property of the session rather than of the utterance.
