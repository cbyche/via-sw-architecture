# Internal Rust Voice Agent — Design Assumptions and Observed Choices

## Interpretation Rules

The items below are **observations about the existing prototype**, not VIA architecture decisions or requirements.

- `Confirmed` means the inspected source/document directly establishes the behavior or structure.
- `Inferred` means the interpretation is useful architecturally but is not directly stated as intent/rationale by the source.
- `Rationale` is included only when the prototype documentation or code comments explicitly state one. Otherwise it says `Not explicitly documented`.
- Related VIA DP/QA links indicate where the observation may provide evidence; they do not mean the prototype satisfies the VIA requirement or QA scenario.

Reference snapshot: source commit `b6032a4c472e18ec216b3e8592346ab269495342`.

## IRA-01 — Session Mode Is Fixed at Connection Time

**Observation**  
The session selects one of `dictation`, `direct`, `agent`, or `interface` at connect time, and `ModePlan` determines which model/tools/delegation/output capabilities are mounted for that session.

**Evidence**  
`source/docs/architecture.md` §2; `source/crates/via-voice/src/mode.rs`; `source/README.md`.

**Rationale**  
Explicitly documented: mode determines which layers are mounted, so it is represented as a protocol type rather than a mutable runtime flag.

**Related VIA DP**  
DP-06, DP-08

**Related QA**  
QA-01, QA-05, QA-10

**Confidence**  
Confirmed

## IRA-02 — Realtime Provider Identity Is Separated from Wire Dialect

**Observation**  
The realtime layer separates provider description/configuration from protocol/dialect behavior and resolves providers through a registry.

**Evidence**  
`source/crates/via-realtime/src/provider.rs`; `source/crates/via-realtime/src/registry.rs`; `source/docs/architecture.md` §3.

**Rationale**  
Explicitly documented: multiple provider families share voice/session policy while retaining dialect-specific transport behavior.

**Related VIA DP**  
DP-06, DP-07

**Related QA**  
QA-01, QA-05, QA-07

**Confidence**  
Confirmed

## IRA-03 — Partial ASR Is Not the Primary Semantic Delegation Input

**Observation**  
Streaming transcription exists, but the inspected delegation path normally uses the realtime model's completed `objective`. A settled transcript is used as a bounded fallback/correlation source when the objective is missing; no partial-ASR-driven intent preparation or capability prefetch was found in the inspected path.

**Evidence**  
`source/crates/via-realtime-local/src/stages.rs`; `source/crates/via-voice/src/tools/transcripts.rs`; `source/crates/via-voice/src/tools/handler.rs` (`spawn_thinking`).

**Rationale**  
The transcript module explicitly documents that settled transcript may arrive after the model tool call and therefore uses a bounded wait rather than making every delegation wait for ASR settlement.

**Related VIA DP**  
DP-01

**Related QA**  
QA-01, QA-09, QA-10

**Confidence**  
Confirmed

## IRA-04 — Interaction Context Uses Typed Packs and Surface Generations, Not a Turn Timeline

**Observation**  
The Context Engine models typed context sections with provenance/trust metadata, stable referents, and host `SurfaceSnapshot` generations. Ordered deixis resolves against host object ordering. The inspected model is not the same as VIA's timestamped speech-turn Interaction Timeline.

**Evidence**  
`source/crates/via-context/src/lib.rs`; `source/crates/via-context/src/pack.rs`; `source/crates/via-context/src/referent.rs`; `source/crates/via-context/src/deixis.rs`; `source/docs/deviations/phase-7.md`.

**Rationale**  
Explicitly documented: referents and ordered deixis are new prototype components, and changed host surfaces mint a new generation so stale handles can be detected rather than guessed.

**Related VIA DP**  
DP-02, DP-08

**Related QA**  
QA-01, QA-06, QA-09

**Confidence**  
Confirmed

## IRA-05 — Conversational Fast Path and Delegated Work Are Separate Execution Paths

**Observation**  
The realtime model can answer directly and use a bounded set of control tools. In `agent` mode, project/long-running work is converted into managed Work and delegated to a downstream harness.

**Evidence**  
`source/crates/via-voice/src/mode.rs`; `source/crates/via-voice/src/tools/handler.rs`; `source/docs/architecture.md` §§2–4.

**Rationale**  
Explicitly documented: questions the realtime model can answer remain immediate, while work requiring tools/files/longer execution is handed to a backend harness without blocking the conversational turn.

**Related VIA DP**  
DP-03

**Related QA**  
QA-01, QA-05, QA-07

**Confidence**  
Confirmed

## IRA-06 — Downstream Harnesses Share One Common Port

**Observation**  
`DownstreamAgent` and `HarnessSession` form the common execution seam for downstream harnesses. ACP is an adapter behind the seam; backend-specific launch/profile data is supplied as data rather than introducing a second orchestration path.

**Evidence**  
`source/crates/via-downstream/src/agent.rs`; `source/crates/via-downstream/src/registry.rs`; `source/crates/via-acp/src/downstream.rs`; `source/docs/architecture.md` §6.

**Rationale**  
Explicitly documented as “one dispatcher”; the architecture aims to keep a new backend shape from creating a parallel dispatch path.

**Related VIA DP**  
DP-04, DP-11, DP-12

**Related QA**  
QA-05, QA-11

**Confidence**  
Confirmed

## IRA-07 — Work/Queue/Permission Semantics Stay Above the Harness Contract

**Observation**  
A harness answers prompts, emits normalized events, and reports cancellation outcome. It does not own the Work queue, `work_id`, permission policy, or the meaning of cancellation state.

**Evidence**  
`source/crates/via-downstream/src/agent.rs`; `source/crates/via-work/src/lib.rs`; `source/crates/via-coordinator/src/lib.rs`.

**Rationale**  
Explicitly documented to make Work lifecycle and permission behavior consistent regardless of the plugged-in harness.

**Related VIA DP**  
DP-04, DP-05, DP-12

**Related QA**  
QA-03, QA-05, QA-11

**Confidence**  
Confirmed

## IRA-08 — A Stable Backend Coordinator Session Is Reused Across Voice Sessions

**Observation**  
Coordination uses an owner/backend-scoped coordinator session identity that is longer-lived than an individual voice turn or Work item. ACP attempts to reattach a persisted backend session and falls back to a fresh session when resume fails.

**Evidence**  
`source/crates/via-coordinator/src/lib.rs`; `source/crates/via-downstream/src/session_key.rs`; `source/crates/via-acp/src/downstream.rs`.

**Rationale**  
Explicitly documented: the coordinator identity is intended to survive voice sessions, Work IDs, and Gateway restarts so the backend conversation can continue.

**Related VIA DP**  
DP-04, DP-09

**Related QA**  
QA-03, QA-10

**Confidence**  
Confirmed

## IRA-09 — Asynchronous Operations Use a Unified Work Record and Explicit Lifecycle

**Observation**  
`work`, `reminder`, `scheduled_task`, and `control` share one Work subsystem. WorkManager owns lifecycle, persistence, notification state, runner/canceler correlation, and event emission.

**Evidence**  
`source/crates/via-work/src/lib.rs`; `source/crates/via-work/src/record.rs`; `source/crates/via-work/src/manager/`.

**Rationale**  
Explicitly documented: a unified record gives cancellation/query/lifecycle behavior one authority rather than separate registries for each asynchronous operation type.

**Related VIA DP**  
DP-05, DP-09

**Related QA**  
QA-02, QA-03, QA-10

**Confidence**  
Confirmed

## IRA-10 — Admission Uses Global, Per-Owner, and Keyed Lane Limits

**Observation**  
The scheduler checks a global concurrency cap, a per-owner cap, and an optional lane limit. Voice coordinator Work uses an owner-scoped lane with width one, serializing Work inside one backend coordinator session while permitting unrelated owners/work to proceed.

**Evidence**  
`source/crates/via-work/src/scheduler.rs`; `source/crates/via-coordinator/src/intent.rs`; `source/docs/architecture.md` §§4, 11.

**Rationale**  
Explicitly documented: exclusion alone is insufficient for the coordinator session; ordering is treated as part of the contract.

**Related VIA DP**  
DP-05

**Related QA**  
QA-02, QA-03

**Confidence**  
Confirmed

## IRA-11 — Prototype Concurrency Lanes Are Not a General PC Resource Model

**Observation**  
The inspected scheduler can serialize Work through arbitrary lane keys, but no inspected contract binds those lanes to Agent-declared microphone/screen/GPU/file/etc. resource requirements. Therefore the existing mechanism cannot be assumed to implement VIA FR-41 resource arbitration.

**Evidence**  
`source/crates/via-work/src/scheduler.rs`; `source/crates/via-downstream/src/capability.rs`; `source/docs/architecture.md` §4.

**Rationale**  
Not explicitly documented as a VIA-style resource model; this limitation is an architectural comparison inferred from the inspected interfaces.

**Related VIA DP**  
DP-05

**Related QA**  
QA-02, QA-03

**Confidence**  
Inferred

## IRA-12 — Cancellation Requires Confirmation

**Observation**  
A Work can remain in `cancelling` after a cancel request. Transport adapters distinguish “requested” from “confirmed”; ACP cancellation is a notification and therefore reports `Requested` until a later terminal outcome confirms it.

**Evidence**  
`source/crates/via-work/src/lib.rs`; `source/crates/via-downstream/src/cancel.rs`; `source/crates/via-acp/src/downstream.rs`.

**Rationale**  
Explicitly documented: sending a cancel message does not prove execution has stopped, so cancellation is state, not an optimistic action.

**Related VIA DP**  
DP-05, DP-09

**Related QA**  
QA-03

**Confidence**  
Confirmed

## IRA-13 — Result Delivery Is Playback-Aware and Decoupled from Work Completion

**Observation**  
A completed Work result is not necessarily spoken immediately. The Injection Gate combines user-speaking/turn-pending/audio-playback state with sleeping/waking/output ownership and notification claims before allowing result presentation.

**Evidence**  
`source/crates/via-voice/src/gate.rs`; `source/crates/via-voice/src/announcement/`; `source/apps/via/src/chat/mod.rs`; `source/docs/architecture.md` §3.

**Rationale**  
Explicitly documented: delegated results can become ready while either side is speaking; playback receipts provide the missing information needed to avoid injecting a result mid-sentence.

**Related VIA DP**  
DP-06, DP-08

**Related QA**  
QA-01, QA-09

**Confidence**  
Confirmed

## IRA-14 — Voice and Text Share the Gateway Session and Work Plane

**Observation**  
`via chat` uses the same `/api/realtime` endpoint, assistant transcript events, task events, identity/session concepts, and result-announcement machinery as voice. Modality differences are expressed with client flags such as `textOnly`, `voiceEnabled`, and `outputEnabled`.

**Evidence**  
`source/apps/via/src/chat/protocol.rs`; `source/apps/via/src/chat/mod.rs`; `source/apps/via/src/chat/client.rs`.

**Rationale**  
Explicitly documented: the text client is an end-to-end harness for the same Gateway/Work stack without requiring audio hardware/model weights.

**Related VIA DP**  
DP-08

**Related QA**  
QA-03, QA-05, QA-10

**Confidence**  
Confirmed

## IRA-15 — Provider Selection Is Explicit/Configured Rather Than Dynamically Optimized

**Observation**  
`RealtimeProviderRegistry` resolves an explicitly requested provider name/alias or a configured default. It publishes model/provider capabilities and configuration state, but inspected selection logic does not score privacy, cost, latency telemetry, or model fitness at request time.

**Evidence**  
`source/crates/via-realtime/src/registry.rs`; `source/crates/via-realtime/src/provider.rs`; `source/crates/via-catalog/`.

**Rationale**  
Not explicitly documented as rejection of dynamic optimization; the selection behavior follows directly from registry code.

**Related VIA DP**  
DP-07

**Related QA**  
QA-01, QA-05, QA-07

**Confidence**  
Confirmed

## IRA-16 — Static Harness Capability Declaration Is Separate from Dynamic Health

**Observation**  
A harness has a startup-validated static descriptor with seven boolean capability flags. Separately, runtime health reports states/codes such as starting, ready, failed, not configured, transient startup, and retry timing.

**Evidence**  
`source/crates/via-downstream/src/capability.rs`; `source/crates/via-downstream/src/descriptor.rs`; `source/crates/via-downstream/src/health.rs`.

**Rationale**  
Explicitly documented: incomplete/inconsistent capability declarations should fail at registration/startup rather than mid-turn, while runtime readiness is an operational state that changes independently.

**Related VIA DP**  
DP-12

**Related QA**  
QA-05, QA-11

**Confidence**  
Confirmed

## IRA-17 — Harness Selection Is Configured-ID Lookup, Not Semantic Agent Routing

**Observation**  
`HarnessRegistry` resolves a configured backend protocol/id to one registered harness. The inspected registry does not perform request-time capability eligibility filtering, semantic scoring, or multi-Agent reranking.

**Evidence**  
`source/crates/via-downstream/src/registry.rs`; `source/crates/via-app/src/backend.rs`; `source/crates/via-coordinator/src/profile.rs`.

**Rationale**  
Explicitly documented that registration is data and one dispatcher resolves the configured id. No explicit rationale was found for excluding future semantic routing.

**Related VIA DP**  
DP-11, DP-12

**Related QA**  
QA-05, QA-11

**Confidence**  
Confirmed

## IRA-18 — Same-Turn Delegation Is Treated as Duplicate Work

**Observation**  
The voice tool handler caches a `turn_id → work_id` association and uses a session/turn submission key. A repeated `spawn_thinking` call for the same turn is answered as duplicate rather than creating another Work item.

**Evidence**  
`source/crates/via-voice/src/tools/handler.rs` (`turn_tasks`, `spawn_thinking`, `submission_key`).

**Rationale**  
Explicitly documented as duplicate-submission suppression so repeated model tool calls do not create duplicate backend work.

**Related VIA DP**  
DP-09

**Related QA**  
QA-03, QA-10

**Confidence**  
Confirmed

## IRA-19 — Existing-Task Follow-Up Selection Is Primarily Model-Directed

**Observation**  
The realtime model chooses whether to call delegation, task-status, cancellation, or other control tools. Once a status/cancel tool is selected, deterministic code resolves an explicit `work_id` or falls back to a Work in the current owner/session. A delegated status question is represented as a separate high-priority `control` Work parented to the target Work.

**Evidence**  
`source/crates/via-voice/src/tools/handler.rs` (`get_agent_task_status`, `query_delegated`, `cancel_agent_task`, `spawn_thinking`).

**Rationale**  
Not explicitly documented as a general task-association strategy. The architecture implication is inferred from the tool surface and selection code.

**Related VIA DP**  
DP-09

**Related QA**  
QA-03, QA-10

**Confidence**  
Confirmed for implementation behavior; Inferred as a general association strategy

## IRA-20 — Intent Representation Is Objective-Centric; Fan-Out Primitive Exists

**Observation**  
`via-coordinator::Intent` contains a cleaned objective string. `IntentSet` can represent multiple ordered objectives and convert them into independently identified Work requests sharing the owner's coordinator lane. During this inspection, a production runtime call site for `IntentSet` fan-out was not confirmed.

**Evidence**  
`source/crates/via-coordinator/src/intent.rs`; `source/docs/architecture.md` §4.

**Rationale**  
Explicitly documented: the type widens an upstream single-objective contract while preserving byte-equivalent behavior for the one-intent case. No evidence was found that this primitive implements VIA FR-06's goal/action/target/parameter/capability canonical request model.

**Related VIA DP**  
DP-10

**Related QA**  
QA-01, QA-07, QA-10

**Confidence**  
Confirmed for the primitive; Inferred/Unconfirmed for end-to-end runtime fan-out

## IRA-21 — Local Voice Composition Is Hidden Behind the Same Realtime Interface

**Observation**  
The local pipeline decomposes VAD, streaming ASR, reasoning, and TTS into traits and an owning session machine, but layers above consume it as another realtime provider. A local endpoint mode can alternatively present a local OpenAI-compatible realtime service under the same provider key.

**Evidence**  
`source/crates/via-realtime-local/src/lib.rs`; `source/crates/via-realtime-local/src/stages.rs`; `source/crates/via-realtime-local/src/machine.rs`.

**Rationale**  
Explicitly documented: the componentized local pipeline should preserve the same event vocabulary so code above the provider cannot distinguish it from a cloud realtime session.

**Related VIA DP**  
DP-06, DP-07

**Related QA**  
QA-01, QA-05, QA-07

**Confidence**  
Confirmed

## IRA-22 — Backend Internal Reasoning and Tool Selection Remain Opaque to VIA

**Observation**  
The downstream seam transports prompts/outcomes/events/cancellation and capability/health metadata but does not expose or centralize the downstream agent's internal tool graph or reasoning policy.

**Evidence**  
`source/crates/via-downstream/src/agent.rs`; `source/crates/via-coordinator/src/lib.rs`; `source/docs/architecture.md` §6.

**Rationale**  
Explicitly documented: VIA ships no agent harness of its own; a harness answers prompts and emits events while coordination/lifecycle remain above it.

**Related VIA DP**  
DP-04, DP-11

**Related QA**  
QA-05, QA-11

**Confidence**  
Confirmed

## IRA-23 — ACP Progress Event Contract Exists but the Adapter Stream Is Currently Empty

**Observation**  
The common `HarnessSession` contract includes normalized session events, but `AcpHarnessSession::events()` currently returns an empty stream and states that activity-to-Work fan-out belongs to another integration layer.

**Evidence**  
`source/crates/via-downstream/src/agent.rs`; `source/crates/via-acp/src/downstream.rs`; `source/docs/deviations/phase-2.md`.

**Rationale**  
Explicitly documented in the ACP adapter: returning an ended/empty stream is preferred to pretending a live progress stream exists; Work correlation is expected to be wired where session activity can be associated with a Work.

**Related VIA DP**  
DP-04, DP-12

**Related QA**  
QA-05, QA-08, QA-11

**Confidence**  
Confirmed

## IRA-24 — Architecture Documentation and Snapshot Are Not Fully Self-Contained

**Observation**  
The snapshot architecture document refers to `VIA_REFERENCE_ARCHITECTURE.html`, which is not present in the captured repository, and the captured ADR directory contains only `0001-placement.md`.

**Evidence**  
`source/docs/architecture.md`; `source/docs/adr/`; `SNAPSHOT.md`.

**Rationale**  
Not explicitly documented as intentional; this is a snapshot-analysis limitation.

**Related VIA DP**  
All DPs where missing rationale could affect comparison

**Related QA**  
QA-05

**Confidence**  
Confirmed
