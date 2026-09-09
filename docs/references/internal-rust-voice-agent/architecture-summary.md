# Internal Rust Voice Agent — Architecture Summary

## Status and Scope

This document describes architecture observations from the internal Rust-based Voice Interaction Agent prototype captured in `cbyche/via-internal-rust-reference`.

The prototype is an **Existing Reference Implementation**. It is not the VIA approved architecture baseline, and implementation choices described here are not requirements. They are evidence to be evaluated against `docs/requirements/requirements-v1.1.md`, the Quality Attributes (QA), and the open Decision Points (DP-01 through DP-12) in `docs/architecture/qa-dp-traceability.md`.

Reference snapshot used for this analysis:

- Source branch: `main`
- Source commit: `b6032a4c472e18ec216b3e8592346ab269495342`
- Snapshot repository commit: `37b69d8`
- Imported: `2026-09-09`

No reference source code is copied into this repository; source paths below identify evidence only.

## Architecture Overview

The prototype is a standalone Rust Cargo workspace organized around a layered voice gateway:

```text
Voice / Text Clients
        |
        v
Gateway / WebSocket + HTTP composition root              via-app, apps/via
        |
        +---------------- Realtime Frontstage ----------- via-voice, via-realtime*
        |                   |                                |
        |                   | direct answer/control tools    + cloud/local providers
        |                   v
        |              realtime model/session
        |                   |
        |                   +-- optional delegation request
        |                           |
        v                           v
Context Engine                  Work / Coordination         via-context, via-work,
(referents, fencing,            (lifecycle, admission,      via-coordinator,
surface generation)             cancellation, delivery)    via-conversation
                                    |
                                    v
                              Downstream Agent seam         via-downstream
                                    |
                          +---------+----------+
                          |                    |
                         ACP             CLI/other adapters
                       via-acp            via-backends
```

The root workspace uses crate boundaries as architecture boundaries. The application layer is the composition root allowed to see across Layer 1, Layer 2, and Layer 3; lower layers expose narrow traits rather than directly reaching into higher or named-backend implementations.

Evidence:

- `source/README.md`
- `source/docs/architecture.md`
- `source/Cargo.toml`
- `source/crates/via-app/src/lib.rs`
- `source/crates/via-voice/src/lib.rs`
- `source/crates/via-work/src/lib.rs`
- `source/crates/via-downstream/src/lib.rs`

## Runtime Modes

A session has one protocol-level mode selected at connect time. `via-voice::ModePlan` converts the requested mode plus environmental facts into the effective runtime composition.

| Mode | Observed runtime behavior |
| --- | --- |
| `dictation` | Streaming capture/ASR only; no model turn, tools, delegation, or speech output. |
| `direct` | Realtime model turn plus control tools; no delegated Work. |
| `agent` | Realtime model plus control tools and delegation/Work queue. If no harness is configured, it degrades to `direct` and exposes the degradation in health. |
| `interface` | Realtime model plus control tools and Context Engine for host/UI interaction; does not delegate project Work. |

Evidence: `source/crates/via-voice/src/mode.rs`, `source/docs/architecture.md` §2.

This is an observed composition strategy, not a VIA requirement. In particular, VIA DP-08 and DP-06 remain open even though the prototype has concrete modality and runtime choices.

## Major Components and Responsibilities

### Realtime Frontstage

`via-realtime` defines a provider/protocol seam. A `RealtimeProvider` describes provider identity, configuration, model/capability metadata, sample rates, preflight behavior, and session creation; the protocol side owns dialect-specific event handling. `RealtimeProviderRegistry` resolves an explicitly requested provider key/alias or a configured default.

The prototype includes cloud-provider adapters, a mock provider, and a local provider. The local pipeline can decompose voice processing into VAD, streaming ASR, reasoning, and TTS while presenting the result upward as one realtime session.

Evidence:

- `source/crates/via-realtime/src/provider.rs`
- `source/crates/via-realtime/src/registry.rs`
- `source/crates/via-realtime-local/src/lib.rs`
- `source/crates/via-realtime-local/src/stages.rs`

### Voice Runtime

`via-voice` owns voice-specific runtime policy rather than provider-specific transport details. Important responsibilities include:

- session-mode plan;
- input normalization and attachment/reference handling;
- turn/transcript correlation;
- tool-call dispatch and stale/duplicate suppression;
- input/voice arbitration;
- permission relay surface;
- result announcement and playback-aware Injection Gate.

The Injection Gate prevents a finished delegated result from being spoken while the user is speaking, a model turn is pending, prior audio is still playing, the session is sleeping/waking, or the client does not own output.

Evidence:

- `source/crates/via-voice/src/mode.rs`
- `source/crates/via-voice/src/input.rs`
- `source/crates/via-voice/src/tools/handler.rs`
- `source/crates/via-voice/src/tools/transcripts.rs`
- `source/crates/via-voice/src/gate.rs`
- `source/crates/via-voice/src/arbitration.rs`

### Work and Coordination

`via-work` owns durable/managed Work semantics above any particular downstream harness. It uses one unified record for `work`, `reminder`, `scheduled_task`, and `control`, with an explicit multi-state lifecycle. Admission considers a global cap, per-owner cap, and optional lane limit. Voice delegations use an owner-scoped coordinator lane with width one.

Cancellation is modeled as a state transition that requires confirmation rather than treating a sent cancel request as completion.

`via-coordinator` owns the backend conversation around a Work item: prompt envelope, fixed coordinator session, serialization, delegation result handling, permission coordination, and result composition. Backend-internal planning/tool selection is intentionally not placed in the coordinator contract.

Evidence:

- `source/crates/via-work/src/lib.rs`
- `source/crates/via-work/src/scheduler.rs`
- `source/crates/via-work/src/manager/`
- `source/crates/via-coordinator/src/lib.rs`
- `source/crates/via-coordinator/src/runtime.rs`

### Downstream Agent Integration

`via-downstream` defines the common harness seam. The observed contract separates:

- static identity/capabilities (`HarnessDescriptor`);
- dynamic runtime health (`HarnessHealth`);
- opening/re-attaching a long-lived session (`DownstreamAgent::open`);
- request/reply (`HarnessSession::prompt`);
- normalized progress event stream (`HarnessSession::events`);
- cancellation request/outcome (`HarnessSession::cancel`).

Queue ownership, `work_id`, cancellation state, and permission policy remain above the harness seam.

ACP is implemented as an adapter behind this contract. Backend-specific launch/profile data is supplied as data rather than by adding backend-specific branches to generic coordination code.

Evidence:

- `source/crates/via-downstream/src/agent.rs`
- `source/crates/via-downstream/src/descriptor.rs`
- `source/crates/via-downstream/src/capability.rs`
- `source/crates/via-downstream/src/health.rs`
- `source/crates/via-downstream/src/registry.rs`
- `source/crates/via-acp/src/downstream.rs`

### Context Engine

`via-context` is separate from conversation history. It introduces typed context sections with provenance/trust metadata, stable referents, untrusted-content fencing, and ordered deixis over a host-supplied surface snapshot/generation.

This is **not equivalent to VIA DP-02's timestamped turn-level Interaction Timeline**. The inspected prototype evidence is centered on current host-surface generations and referent handles, rather than a speech-start-to-speech-end timeline containing timestamped screen evidence and pointer/gesture history. It should therefore be evaluated as a distinct reference design rather than mapped directly to DP-02 Alternative B or C.

Evidence:

- `source/crates/via-context/src/lib.rs`
- `source/crates/via-context/src/pack.rs`
- `source/crates/via-context/src/referent.rs`
- `source/crates/via-context/src/deixis.rs`
- `source/docs/deviations/phase-7.md`

## Major Control and Data Flows

### 1. Direct Voice Interaction

1. Client opens the realtime WebSocket and selects a session mode/provider.
2. Voice runtime and provider establish the realtime session.
3. Audio/transcript events are correlated to a turn.
4. In `direct` mode, the realtime model answers without creating Work.
5. Playback receipts feed the voice/announcement state so later output does not overlap incorrectly.

This is the prototype's low-latency conversational path.

### 2. Delegated Work

1. In `agent` mode, the realtime model may invoke the delegation tool (`spawn_thinking`).
2. `ToolCallHandler` performs stale-call, permission, backend-availability, and same-turn duplicate checks.
3. The model-supplied objective is normally used immediately. If it is blank, the handler waits briefly for a settled transcript and falls back to the user's final utterance.
4. A `Work` is created with owner/session/turn correlation, duplicate-submission key, owner coordinator lane, runner, and canceler.
5. The tool returns an acceptance receipt without waiting for backend completion.
6. WorkManager admission/lifecycle drives the runner.
7. Coordinator opens/reuses the owner's fixed backend coordinator session and interacts with the harness through `DownstreamAgent`/`HarnessSession`.
8. Terminal result returns to Work state; the announcement path waits for the Injection Gate before presentation.

Evidence: `source/crates/via-voice/src/tools/handler.rs`, `source/crates/via-work/`, `source/crates/via-coordinator/`, `source/crates/via-downstream/`.

### 3. Streaming/Partial Input

The prototype clearly has streaming ASR and running transcript updates, especially in the local pipeline. However, inspected delegation code does not show partial ASR driving semantic intent preparation, capability prefetch, or routing. Delegation normally uses the realtime model's objective; final/settled transcript is a late correlation/fallback source.

For VIA DP-01, the prototype is therefore closest to **final-turn semantic processing**, despite supporting streaming transcription for transport/UI/runtime purposes.

Evidence: `source/crates/via-realtime-local/src/stages.rs`, `source/crates/via-voice/src/tools/transcripts.rs`, `source/crates/via-voice/src/tools/handler.rs`.

### 4. Text Interaction

`via chat` uses the same `/api/realtime` WebSocket and Work plane as voice. It declares itself `textOnly`, sends text as an input-message event, receives the same assistant transcript stream and task events, and even sends playback receipts so the shared announcement machinery advances correctly.

This demonstrates substantial runtime unification, although the inspected prototype does not establish VIA's proposed canonical `UserTurn`/`InteractionAnchor` abstraction as an explicit architecture type.

Evidence: `source/apps/via/src/chat/protocol.rs`, `source/apps/via/src/chat/mod.rs`, `source/apps/via/src/chat/client.rs`.

### 5. Context / Interface Interaction

`interface` mode mounts the Context Engine and does not delegate project Work. Host surface state is represented by a generation-tagged snapshot; stable referent handles can be resolved against the current generation. Ordered deixis maps expressions such as ordered references to host-supplied object ordering.

This is useful evidence for referent stability and stale-target handling, but it does not prove the temporal interaction-evidence model required by VIA FR-03/FR-04.

### 6. Failure and Recovery

Observed mechanisms include:

- agent mode degrading to direct when no harness is configured;
- provider/harness health surfaces with transient startup vs failure distinction;
- Work persistence/reconciliation and explicit lifecycle state;
- confirmed cancellation rather than optimistic cancellation;
- ACP session re-attachment from a persisted session key, with fallback to a fresh session when resume fails;
- provider preflight/configuration refusal before entering an unusable session;
- stale tool-call rejection by turn id/generation;
- duplicate Work suppression by submission key.

These mechanisms are reference evidence for QA-03 and QA-04; they are not automatically VIA recovery requirements beyond what the Approved Baseline already states.

## Voice, Session, Context, Agent, Task, and Model Integration

| Concern | Prototype integration choice |
| --- | --- |
| Voice session | Protocol-level mode fixed at connect; provider/session hidden behind realtime traits. |
| Logical ownership | Owner/session/turn identifiers are carried into Work; backend coordinator session has a stable owner/backend identity that can survive voice sessions. |
| Context | Separate `ContextPack`/referent engine plus conversation/memory inputs; current-surface generation is distinct from a timestamped Interaction Timeline. |
| Local capability | Realtime model answers directly and runs a bounded set of control tools; larger work is delegated in `agent` mode. |
| Task | `WorkManager` owns lifecycle, admission, persistence, cancellation, and notification state independent of a backend harness. |
| Agent | One common downstream seam; configured backend id resolves to a harness. Backend internal execution strategy is opaque. |
| Model/provider | Provider registry + model profile/capability metadata; explicit/default provider resolution rather than a cost/privacy/telemetry optimizer. |
| Result delivery | Work result is decoupled from the realtime turn and injected only when voice/playback state permits. |

## Important Discrepancies and Analysis Limits

1. **The snapshot is explicitly non-authoritative.** `SNAPSHOT.md` states that prototype choices must be independently evaluated.
2. **Architecture target reference missing.** `source/docs/architecture.md` references `VIA_REFERENCE_ARCHITECTURE.html`, but that file is not present in the captured snapshot. Any rationale that depends exclusively on that artifact cannot be independently checked here.
3. **ADR coverage is limited.** The captured `source/docs/adr/` contains `0001-placement.md`; other design rationale must be taken from architecture/deviation docs or code comments and should not be assumed to have ADR status.
4. **Intent fan-out wiring requires more inspection.** `via-coordinator::IntentSet` and conversion to multiple Work requests are implemented, but a production runtime call site was not confirmed during this analysis. The primitive is confirmed; end-to-end fan-out support is not asserted here.
5. **ACP progress-event seam is not fully wired.** `AcpHarnessSession::events()` currently returns an empty stream and states that activity-to-Work fan-out is another layer's responsibility. This is an implementation gap relative to the otherwise generic progress-event contract.
6. **Resource arbitration is narrower than VIA DP-05.** The prototype has concurrency caps and keyed lanes, but inspected evidence does not establish Agent-declared OS/PC resource requirements or per-resource arbitration as required by VIA FR-41.
7. **Agent routing is configuration lookup, not semantic routing.** The harness registry resolves a configured backend id; inspected code does not implement VIA DP-11's request-time capability/semantic Agent router.
8. **Documented scale claims were not independently re-run.** README counts such as crate/test/contract totals are treated as source-document claims, not independently measured facts.

## Architectural Value as a Reference

The strongest reusable reference inputs are not individual Rust types but the boundaries demonstrated by the implementation:

- realtime transport/provider policy separated from voice interaction policy;
- asynchronous Work lifecycle separated from conversational turn lifecycle;
- downstream harness execution hidden behind a common contract;
- static capability declaration separated from dynamic health;
- backend-internal reasoning kept outside the gateway contract;
- safe result injection explicitly coordinated with voice/playback state;
- common task/runtime plane shared by voice and text clients;
- implementation-enforced crate dependency boundaries.

Each of these remains an input to VIA Decision Points rather than a decided VIA architecture choice.