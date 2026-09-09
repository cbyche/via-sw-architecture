# Internal Rust Voice Agent — Behavioral Scenarios

## Purpose

These scenarios describe end-to-end behaviors that are implemented or explicitly assumed by the internal Rust prototype. They are **reference scenarios**, not VIA acceptance criteria. Their purpose is to expose architecture behavior that can be compared with the Approved Baseline and open Decision Points.

Reference snapshot: source commit `b6032a4c472e18ec216b3e8592346ab269495342`.

## BS-01 — Direct Realtime Voice Answer

**Preconditions**

- A realtime provider is configured and usable.
- Session mode is `direct`, or `agent` with no delegation required.
- Client owns the active voice/output slot.

**Flow**

1. Client connects to `/api/realtime` and establishes session capabilities/mode.
2. Audio enters the realtime session; provider emits normalized voice/transcript events.
3. The realtime model takes the conversational turn.
4. The model answers directly without creating a Work record.
5. Audio/transcript output is streamed to the client.
6. Playback receipts update announcement/playback state.

**Architecture observations**

- Fast conversational work is intentionally kept off the asynchronous Work path.
- Provider dialect details remain below the generic voice runtime.
- The client playback state is part of runtime correctness, not only UI telemetry.

**Evidence**

- `source/crates/via-voice/src/mode.rs`
- `source/crates/via-realtime/src/provider.rs`
- `source/crates/via-voice/src/gate.rs`

**VIA relevance**

DP-03, DP-06, DP-07; QA-01.

## BS-02 — Voice Request Delegates Long-Running Work

**Preconditions**

- Session effective mode is `agent`.
- A backend harness is configured/available.
- No blocking permission decision prevents new work.

**Flow**

1. Realtime model decides that the request requires backend work and invokes `spawn_thinking` with an objective.
2. Tool handler rejects stale/duplicate calls and checks backend availability.
3. Current-turn input parts and referenced historical assets are merged for the delegation request.
4. A new Work record is submitted with owner, session, turn, submission key, coordinator lane, runner, and canceler.
5. Tool handler immediately returns an `accepted` receipt and `work_id` to the realtime model; it does not wait for backend completion.
6. WorkManager admits/runs the Work according to scheduler limits.
7. Coordinator opens/reuses the owner/backend coordinator session.
8. Coordinator prompts the downstream harness through the common `HarnessSession` contract.
9. Result is stored in Work state.
10. Announcement machinery later presents the result when the Injection Gate permits.

**Architecture observations**

- Conversational latency and backend execution latency are explicitly decoupled.
- Work lifecycle and harness lifecycle are separate.
- Per-owner coordinator serialization protects a long-lived downstream session.

**Evidence**

- `source/crates/via-voice/src/tools/handler.rs`
- `source/crates/via-work/src/lib.rs`
- `source/crates/via-work/src/scheduler.rs`
- `source/crates/via-coordinator/src/lib.rs`
- `source/crates/via-downstream/src/agent.rs`

**VIA relevance**

DP-03, DP-04, DP-05; QA-01, QA-02, QA-03.

## BS-03 — Settled Transcript Arrives After Delegation Tool Call

**Preconditions**

- Realtime model emits a delegation tool call before settled ASR is available.

**Flow**

1. Realtime model tool call is correlated to a turn.
2. If the tool call already contains a non-empty objective, delegation can proceed without waiting for settled transcript.
3. If objective is missing, transcript correlation waits for the settled transcript for a bounded period.
4. When transcript arrives, it can provide the original user request/fallback objective.
5. If the bounded wait expires, the handler does not block indefinitely; normal validation/failure behavior applies.

**Architecture observations**

- Streaming/final transcript timing is treated as asynchronous with respect to model tool calls.
- Semantic delegation is not shown to be incrementally prepared from partial ASR.
- Final transcript is a correlation/fallback mechanism rather than the primary request-time intent pipeline.

**Evidence**

- `source/crates/via-voice/src/tools/transcripts.rs`
- `source/crates/via-voice/src/tools/handler.rs`
- `source/crates/via-realtime-local/src/stages.rs`

**VIA relevance**

DP-01; QA-01, QA-09, QA-10.

## BS-04 — Completed Work Waits for a Safe Announcement Window

**Preconditions**

- A delegated Work has reached a terminal result that should be presented.

**Flow**

1. Work completion makes a result eligible for notification.
2. Notification claim prevents multiple live frontends from presenting the same result.
3. Injection Gate checks sleeping/waking state, output ownership, user speaking, pending turn state, and outstanding playback.
4. If blocked, result remains pending instead of interrupting current speech.
5. Client `playback.started` / `playback.ended` / `playback.cancelled` receipts update the gate/window and playback cursor.
6. Result is presented only when the gate becomes safe.

**Architecture observations**

- Task completion and conversational delivery are distinct states.
- Client playback acknowledgment is a correctness signal.
- This is a strong reference for asynchronous voice result injection.

**Evidence**

- `source/crates/via-voice/src/gate.rs`
- `source/crates/via-voice/src/announcement/`
- `source/crates/via-work/src/presentation.rs`

**VIA relevance**

DP-06, DP-08; QA-01, QA-09.

## BS-05 — Text Client Uses the Same Gateway and Work Plane

**Preconditions**

- Gateway is running or `via chat` can autostart it.

**Flow**

1. Text client obtains its identity through `/api/health` and carries the issued cookie.
2. It opens `/api/realtime` with the same session-id mechanism as voice.
3. Connect frame declares `voiceEnabled: false`, `outputEnabled: true`, and `textOnly: true`.
4. User lines are sent as realtime input-message events.
5. Assistant transcript delta/final events are rendered from the shared event stream.
6. Task-plane events are visible and `/tasks`/`/cancel` operate on the same Work subsystem.
7. The text client sends playback receipts for audio events even though it does not play audio, allowing the shared announcement state machine to advance.

**Architecture observations**

- Voice/text convergence occurs early at the Gateway/realtime-session boundary.
- Modality-specific capture/output behavior remains explicit in client capabilities.
- The prototype demonstrates a common execution/task plane, but not necessarily VIA's proposed canonical `UserTurn` abstraction.

**Evidence**

- `source/apps/via/src/chat/protocol.rs`
- `source/apps/via/src/chat/mod.rs`
- `source/apps/via/src/chat/client.rs`

**VIA relevance**

DP-08, DP-09; QA-03, QA-05, QA-10.

## BS-06 — Agent Mode Starts Without a Configured Harness

**Preconditions**

- Client requests `agent` mode.
- No downstream harness is configured.

**Flow**

1. `ModePlan` detects that the requested agent capability cannot be mounted.
2. Effective mode becomes `direct`.
3. Delegation tool is not declared.
4. Direct/control functionality remains available.
5. Health surface reports the degradation rather than silently claiming full agent operation.

**Architecture observations**

- Missing Layer-3 execution is modeled as an explicit degraded mode rather than a process-wide failure.
- Capability exposure to the model changes with effective runtime composition.

**Evidence**

- `source/crates/via-voice/src/mode.rs`
- `source/crates/via-app/src/backend.rs`
- `source/crates/via-downstream/src/health.rs`

**VIA relevance**

DP-03, DP-04, DP-12; QA-04, QA-05.

## BS-07 — User Cancels Active Work

**Preconditions**

- An owner/session has cancellable Work.

**Flow**

1. Realtime model invokes `cancel_agent_task`, optionally with a `work_id`.
2. If no id is supplied, handler chooses a cancellable Work within the current owner/session according to the prototype's deterministic list behavior.
3. Related status-control Work is cancelled first when present.
4. WorkManager moves the target through cancellation semantics.
5. Adapter may report `Requested` rather than `Confirmed` if transport only sent a cancellation notification.
6. Work remains non-terminal until stop is confirmed by cancellation outcome or execution settlement.

**Architecture observations**

- Cancellation is a protocol/state reconciliation problem, not a fire-and-forget side effect.
- Natural-language target selection is largely delegated to the model's tool choice; deterministic code operates after the tool is selected.

**Evidence**

- `source/crates/via-voice/src/tools/handler.rs`
- `source/crates/via-work/src/lib.rs`
- `source/crates/via-downstream/src/cancel.rs`
- `source/crates/via-acp/src/downstream.rs`

**VIA relevance**

DP-05, DP-09; QA-03, QA-10.

## BS-08 — User Asks About an Existing Delegated Task

**Preconditions**

- A delegated Work exists.

**Flow**

1. Model invokes `get_agent_task_status` and may provide `work_id`/question.
2. Handler resolves explicit id or defaults to a Work in the current owner/session.
3. For a Work still in `delegated`, the prototype creates a separate high-priority `control` Work with `parent_work_id` pointing to the target.
4. Control Work uses the same owner coordinator lane and asks the downstream coordinator/harness for status.
5. Duplicate in-flight status-control Work for the same parent is suppressed.
6. Response is correlated back to the target Work for presentation.

**Architecture observations**

- Follow-up/control activity can be represented as separate Work parented to an existing task.
- This differs from VIA FR-43's open design question about reusing the same logical Task and downstream thread for substantive follow-up; the prototype's status query is evidence, not proof of that broader requirement.

**Evidence**

- `source/crates/via-voice/src/tools/handler.rs` (`get_agent_task_status`, `query_delegated`).

**VIA relevance**

DP-09; QA-03, QA-10.

## BS-09 — Concurrent Work from Multiple Owners

**Preconditions**

- Multiple Work items are ready.

**Flow**

1. Admission checks global concurrency limit.
2. Admission checks per-owner concurrency limit.
3. If Work has a lane, admission checks lane width.
4. Voice delegations for one owner use `coordinator:<owner>` with width one, so they serialize within that backend coordinator session.
5. Work for another owner or unlaned Work may proceed concurrently if global/per-owner caps permit.
6. WorkManager's owning actor preserves mutation/order semantics around scheduling.

**Architecture observations**

- Prototype has explicit admission and serialization mechanisms.
- Inspected lanes represent logical coordination scopes; they are not proven to be a generic PC resource arbitration model with Agent-declared resource requirements.

**Evidence**

- `source/crates/via-work/src/scheduler.rs`
- `source/crates/via-work/src/manager/`
- `source/docs/architecture.md` §11.

**VIA relevance**

DP-05; QA-02, QA-03.

## BS-10 — Local Componentized Voice Pipeline with Barge-In

**Preconditions**

- Local provider is configured in pipeline mode.
- Required local stages/weights are available.

**Flow**

1. Audio is fed to VAD and streaming ASR.
2. Running transcript updates contain committed text plus recognizer stash.
3. Final transcript produces a reasoning turn.
4. Reasoning emits text and/or tool calls as a stream.
5. Sentence-level text is synthesized while generation continues.
6. If user speech starts during output, barge-in drops the active reasoning/speech streams so stale output becomes inert.
7. Local pipeline emits the same normalized realtime event vocabulary expected above the provider seam.

**Architecture observations**

- The prototype can implement a decomposed VAD+ASR+reasoning+TTS runtime without exposing decomposition to upper layers.
- Runtime composition and provider abstraction are orthogonal: a componentized implementation can still satisfy a single realtime-session interface.

**Evidence**

- `source/crates/via-realtime-local/src/lib.rs`
- `source/crates/via-realtime-local/src/stages.rs`
- `source/crates/via-realtime-local/src/machine.rs`

**VIA relevance**

DP-06, DP-07; QA-01, QA-05, QA-07.

## BS-11 — Interface Mode Resolves a Host Referent

**Preconditions**

- Session mode is `interface`.
- Context Engine is mounted.
- Host supplies a current `SurfaceSnapshot` containing ordered objects.

**Flow**

1. Context Engine receives/reads the current surface generation.
2. Referent handles represent host-visible objects with stable identities within their validity rules.
3. Model/context logic refers to an object or ordered position.
4. Resolver attempts exact/unique resolution and detects stale/missing/ambiguous references rather than blindly guessing.
5. Host action is targeted through the resolved object identity.
6. When the surface changes, a new generation prevents stale handles from being silently treated as current.

**Architecture observations**

- Stable referent identity and staleness are explicit concepts.
- The scenario is based on a current/generation snapshot and is not equivalent to reconstructing “what was under the pointer when the user said that” from a timestamped interaction timeline.

**Evidence**

- `source/crates/via-context/src/lib.rs`
- `source/crates/via-context/src/referent.rs`
- `source/crates/via-context/src/deixis.rs`
- `source/docs/deviations/phase-7.md`

**VIA relevance**

DP-02, DP-08; QA-09, QA-10.

## BS-12 — Gateway Restart / Downstream Session Resume

**Preconditions**

- A stable downstream `SessionKey` has a persisted ACP session record.

**Flow**

1. Coordinator requests the session identified by owner/backend/session role.
2. ACP adapter checks its persisted session registry.
3. If a record exists, adapter attempts `session/resume` using the recorded session id/cwd.
4. If resume succeeds, downstream conversation continues.
5. If resume fails because backend state disappeared or changed, the record is deleted and a fresh backend session is created rather than failing the entire interaction.

**Architecture observations**

- Durable logical coordination identity is intentionally separate from a single voice WebSocket lifetime.
- Recovery is designed to tolerate loss of backend-native session state.

**Evidence**

- `source/crates/via-acp/src/downstream.rs`
- `source/crates/via-downstream/src/session_key.rs`
- `source/crates/via-coordinator/src/lib.rs`

**VIA relevance**

DP-04, DP-09; QA-03, QA-04.

## BS-13 — Capability/Health Validation Before or During Dispatch

**Preconditions**

- A downstream harness is registered/configured.

**Flow**

1. Static backend id/label/capability declaration is validated when descriptor is constructed/registered.
2. Inconsistent static capabilities are rejected before normal turn execution.
3. Dynamic health separately reports starting/ready/failed/not-configured and retry/backoff information.
4. Voice delegation can treat an unknown probe optimistically, a known unavailable backend as retryable/refused, and an unconfigured backend as no delegation capability.

**Architecture observations**

- Static contract and dynamic operational state are deliberately distinct.
- Capability manifest is not the same as semantic Agent routing; the prototype still dispatches to a configured backend id.

**Evidence**

- `source/crates/via-downstream/src/capability.rs`
- `source/crates/via-downstream/src/descriptor.rs`
- `source/crates/via-downstream/src/health.rs`
- `source/crates/via-voice/src/tools/handler.rs`

**VIA relevance**

DP-11, DP-12; QA-05, QA-11.

## Scenarios Requiring Additional Inspection

The following cannot yet be stated as confirmed end-to-end prototype scenarios from the inspected evidence:

1. **Multi-intent utterance fan-out in production runtime.** `IntentSet` and conversion to multiple Work items exist, but a production call site was not confirmed.
2. **ACP progress streaming into Work activity.** The generic session-event seam exists, but `AcpHarnessSession::events()` currently returns an empty stream.
3. **Generic PC resource conflict resolution.** Scheduler lanes exist, but no complete Agent resource-requirement → per-resource arbitration flow was identified.
4. **Request-time multi-Agent routing.** Harness registry resolves a configured id; no semantic eligibility/reranking flow was identified.
5. **Timestamped speech/screen/pointer interaction replay.** Context Engine provides surface generations/referents, but no VIA-style turn Interaction Timeline was identified.
