# Internal Rust Voice Agent — VIA DP / QA Traceability

## Purpose and Rules

This document maps observed choices in the internal Rust prototype to the open Decision Points in `docs/architecture/qa-dp-traceability.md`.

It does **not** close any DP and does **not** change the Approved Baseline in `docs/requirements/requirements-v1.1.md`.

Interpretation terms:

- **Closest alternative** means “the prototype behavior most resembles this alternative,” not “VIA should choose it.”
- **Partial** means the prototype contains a useful mechanism but does not implement the full VIA decision scope.
- **Unknown / no direct match** means the inspected prototype solves the problem differently or the available evidence is insufficient to place it honestly into the VIA alternatives.
- QA references use the IDs already defined by the Approved Baseline and `qa-dp-traceability.md`.

Reference snapshot: `cbyche/via-internal-rust-reference`, source commit `b6032a4c472e18ec216b3e8592346ab269495342`.

## Summary Matrix

| DP | Prototype has relevant choice? | Closest VIA alternative | Evidence summary | Related QA | Confidence |
| --- | --- | --- | --- | --- | --- |
| DP-01 Partial / streaming input processing | Yes | **A — final-turn semantic processing**, with streaming transcription used elsewhere | Streaming ASR/running transcript exists, but `spawn_thinking` normally uses model objective; settled transcript is bounded fallback. No incremental intent/prefetch path confirmed. | QA-01, QA-09, QA-10 | Confirmed |
| DP-02 Interaction context representation | Yes, but different problem shape | **Unknown / no direct match** | `ContextPack` + stable referents + generation-tagged `SurfaceSnapshot`; no inspected timestamped turn Interaction Timeline/pointer history. | QA-01, QA-09 | Confirmed for observed model; Unknown for full-repo absence |
| DP-03 Capability placement boundary | Yes | **B — Hybrid allow-listed/local fast path + delegation** | Realtime model answers directly and runs bounded control tools; project/long work uses managed delegation. | QA-01, QA-05, QA-07 | Confirmed |
| DP-04 Generic vs specialized Downstream Agent integration | Yes | **B — common port + internal/backend-specific extension data** | One `DownstreamAgent` seam; ACP/CLI adapters; backend-specific profiles/native-delegation flags remain behind common dispatcher rather than dedicated bypass. | QA-01, QA-05, QA-07 | Confirmed |
| DP-05 Concurrent task resource arbitration | Partially | **Unknown; mechanism is B-like keyed queue/admission but scope is narrower** | Global/per-owner/lane admission and coordinator serialization exist; no Agent-declared generic PC resource requirements/arbitration confirmed. | QA-02, QA-03 | Confirmed for scheduler; Unknown for full resource DP |
| DP-06 Voice runtime composition & ASR side channel | Yes | **Mixed B/C support** | Cloud realtime providers can behave like S2S with transcript events; local provider can be decomposed VAD+streaming ASR+reasoning+TTS behind same interface. | QA-01, QA-05, QA-09 | Confirmed |
| DP-07 Model gateway selection policy | Yes | **A — component/configuration-fixed binding** | Registry resolves requested provider/alias or configured default; no privacy/cost/telemetry scoring optimizer found. | QA-01, QA-05, QA-07 | Confirmed |
| DP-08 Voice/Text interaction unification boundary | Yes | **B-like common early runtime**, but not canonical VIA `UserTurn` proof | Text and voice use same realtime Gateway/session/event/Work plane; modality flags remain distinct. | QA-03, QA-10, QA-05 | Confirmed behavior; mapping partly Inferred |
| DP-09 Existing-task vs new-task association | Yes | **A — model-only at semantic decision point** | Model chooses `spawn_thinking` vs status/cancel tools; deterministic code resolves ids/defaults after tool choice. Same-turn new delegation is deduplicated. | QA-10, QA-03 | Confirmed |
| DP-10 Intent refinement architecture | Yes, but limited | **A — single-shot/model-authored objective** | Delegation is objective-centric. `IntentSet` fan-out primitive exists but no canonical goal/action/target/params/capability refinement/validator pipeline; production fan-out call site not confirmed. | QA-01, QA-10, QA-07 | Confirmed primitive; runtime fan-out needs inspection |
| DP-11 Downstream Agent routing architecture | Relevant integration exists, routing choice does not match | **Unknown / no direct match** | `HarnessRegistry` is configured backend-id lookup, not LLM-only or capability/semantic multi-Agent routing. | QA-01, QA-11, QA-05 | Confirmed |
| DP-12 Downstream Agent capability contract | Yes | **C-like partial — stable manifest + dynamic health** | Seven startup-validated static capability flags plus dynamic health/backoff. No dynamic capability discovery or Agent resource-requirement contract confirmed. | QA-11, QA-05 | Confirmed |

## DP-01 — Partial / Streaming Input Processing

**VIA decision scope**  
Whether partial ASR is ignored semantically until final input, used for read-only prefetch, or used for incremental intent preparation.

**Prototype observation**

- Local pipeline exposes cumulative streaming transcript updates (`text` + recognizer `stash`).
- Settled transcript is correlated independently from realtime model tool calls.
- Delegation normally proceeds from the realtime model's `objective` without waiting for settled ASR.
- Only when the objective is empty does the handler wait briefly for the settled transcript and use the final user request as fallback.
- No inspected code shows partial transcript driving capability lookup, context prefetch, routing, or incremental canonical intent state.

**Closest alternative**  
**A — final-turn semantic processing**, while still using streaming transcription for runtime/UI/provider behavior.

**Evidence**

- `source/crates/via-realtime-local/src/stages.rs`
- `source/crates/via-voice/src/tools/transcripts.rs`
- `source/crates/via-voice/src/tools/handler.rs`

**Related QA**  
QA-01, QA-09, QA-10.

**Architecture use**  
Useful baseline evidence for latency/complexity tradeoffs, but it does not validate the VIA assumption that partial input should or should not be semantically exploited.

## DP-02 — Interaction Context Representation

**VIA decision scope**  
Speech-start snapshot vs timestamped turn-level Interaction Timeline vs timeline plus gesture/vision analysis.

**Prototype observation**

- `ContextPack` carries typed context sections with provenance/trust/tier/token metadata.
- Referents have stable handles and stale-resolution rules.
- Ordered deixis operates on a host-supplied `SurfaceSnapshot` with a generation and ordered objects.
- `via-voice::input` has source-text anchors for text/file input parts.
- The inspected APIs do not provide a speech-turn timeline of timestamped screen snapshots/pointer/gesture evidence.

**Closest alternative**  
**Unknown / no direct match.** It is snapshot/generation-oriented, but it is not honestly equivalent to DP-02 Alternative A's specified speech-start snapshot or Alternatives B/C's timestamped Interaction Timeline.

**Evidence**

- `source/crates/via-context/src/lib.rs`
- `source/crates/via-context/src/pack.rs`
- `source/crates/via-context/src/referent.rs`
- `source/crates/via-context/src/deixis.rs`
- `source/crates/via-voice/src/input.rs`
- `source/docs/deviations/phase-7.md`

**Related QA**  
QA-01, QA-09.

**Architecture use**  
Strong reference for referent identity/staleness/provenance; weak evidence for temporal grounding. Additional host/realtime event inspection is required before claiming absence of timeline evidence across the entire codebase.

## DP-03 — Capability Placement Boundary

**VIA decision scope**  
Thin VIA delegation vs hybrid local fast path vs richer local capabilities.

**Prototype observation**

- `direct`/`agent` modes expose realtime-model control tools.
- Direct conversational answers do not create Work.
- In `agent` mode, `spawn_thinking` creates managed backend Work.
- Context/memory/notes/reminder/status/cancel/permission control behavior is retained above the downstream harness.
- Backend harness owns its internal execution strategy.

**Closest alternative**  
**B — Hybrid VIA**.

**Evidence**

- `source/crates/via-voice/src/mode.rs`
- `source/crates/via-voice/src/tools/handler.rs`
- `source/crates/via-work/src/lib.rs`
- `source/crates/via-downstream/src/agent.rs`

**Related QA**  
QA-01, QA-05, QA-07.

**Architecture use**  
Provides concrete evidence that a bounded fast path can coexist with opaque downstream execution. It does not define which VIA capabilities should be allow-listed locally.

## DP-04 — Generic vs Specialized Downstream Agent Integration

**VIA decision scope**  
Generic protocol only vs common port with internal extension vs dedicated internal fast path.

**Prototype observation**

- Common `DownstreamAgent`/`HarnessSession` port is the only generic Layer-3 seam.
- Backend static capability/profile data can describe native delegation, session MCP, permissions, native history, UI, etc.
- ACP adapter implements the common port.
- Named backend differences are supplied by profile/catalog data in `via-backends` rather than by bypassing the common coordinator/dispatcher.

**Closest alternative**  
**B — common port + backend/internal extension**, because backend-specific/native features exist but stay expressed behind the common port/profile rather than a dedicated direct path.

**Evidence**

- `source/crates/via-downstream/src/agent.rs`
- `source/crates/via-downstream/src/capability.rs`
- `source/crates/via-acp/src/downstream.rs`
- `source/crates/via-coordinator/src/profile.rs`
- `source/docs/architecture.md` §6

**Related QA**  
QA-01, QA-05, QA-07.

**Architecture use**  
Strong reference input for preserving AP-05-style downstream autonomy while retaining a stable common contract.

## DP-05 — Concurrent Task Resource Arbitration

**VIA decision scope**  
Global lease vs per-resource lock/queue vs cooperative resource-aware scheduler vs user-mediated conflict resolution.

**Prototype observation**

- Work scheduler uses global and per-owner concurrency caps.
- Optional lane key + lane width supports explicit serialization domains.
- Voice coordinator Work uses an owner-specific lane with width one.
- Work manager/actors preserve lifecycle mutation order.
- The seven downstream capability flags inspected do not describe required PC resources, and no inspected path converts Agent resource requirements into locks/leases.

**Closest alternative**  
**Unknown for the VIA decision.** The mechanism resembles a keyed queue/per-resource primitive, but the observed keys are logical coordination lanes, not a confirmed generic resource model.

**Evidence**

- `source/crates/via-work/src/scheduler.rs`
- `source/crates/via-work/src/manager/`
- `source/crates/via-coordinator/src/intent.rs`
- `source/crates/via-downstream/src/capability.rs`

**Related QA**  
QA-02, QA-03.

**Architecture use**  
Useful implementation reference for ordered/admitted concurrency, but additional resource-contract code inspection is needed before using it as evidence for FR-41.

## DP-06 — Voice Runtime Composition & ASR Side Channel

**VIA decision scope**  
S2S + final transcript vs S2S + streaming ASR/delta side-channel vs decomposed VAD+ASR+LLM+TTS.

**Prototype observation**

- `via-realtime` exposes one generic realtime provider/session interface.
- Cloud provider adapters can supply full realtime speech-model behavior and transcript events.
- Local pipeline explicitly decomposes VAD, streaming ASR, reasoning, and TTS.
- The local machine presents this decomposition upward using the same normalized realtime event/session surface.
- `dictation` can mount ASR with no model turn.

**Closest alternative**  
**Mixed B/C support rather than one global choice.** The architecture allows provider implementations with different internal compositions behind one upper-layer interface.

**Evidence**

- `source/crates/via-realtime/src/provider.rs`
- `source/crates/via-realtime-local/src/lib.rs`
- `source/crates/via-realtime-local/src/stages.rs`
- `source/crates/via-voice/src/mode.rs`

**Related QA**  
QA-01, QA-05, QA-09.

**Architecture use**  
Important evidence that the VIA decision may need to distinguish the **upper voice-runtime contract** from the **provider-specific internal pipeline composition**.

## DP-07 — Model Gateway Selection Policy

**VIA decision scope**  
Fixed binding vs prequalified policy selection vs telemetry-driven dynamic optimization.

**Prototype observation**

- Provider registry resolves explicit requested provider/alias or configured default.
- Provider/model descriptors publish capabilities/configuration/model catalog.
- No inspected selection code scores cost, privacy, latency telemetry, or quality at request time.

**Closest alternative**  
**A — component/configuration-fixed model binding**.

**Evidence**

- `source/crates/via-realtime/src/registry.rs`
- `source/crates/via-realtime/src/provider.rs`
- `source/crates/via-catalog/`

**Related QA**  
QA-01, QA-05, QA-07.

**Architecture use**  
Reference for provider abstraction/registration, not evidence that VIA should use static selection.

## DP-08 — Voice/Text Interaction Unification Boundary

**VIA decision scope**  
Late modality merge vs early canonical UserTurn vs common Conversation Turn Manager plus modality-specific InteractionAnchor.

**Prototype observation**

- Voice and `via chat` connect to the same realtime Gateway endpoint/session mechanism.
- Text uses `InputMessage` and capability flags rather than a separate task pipeline.
- Both observe the same assistant transcript and Work/task plane.
- Result-announcement state is shared enough that text client emits playback receipts even without playing audio.
- `InputPart` provides a normalized input representation, but the inspected code does not establish the Approved Baseline's canonical `UserTurn` + InteractionAnchor abstraction.

**Closest alternative**  
**B-like early/common runtime convergence**, with explicit modality flags. Mapping to B is architectural similarity, not type-level equivalence.

**Evidence**

- `source/apps/via/src/chat/protocol.rs`
- `source/apps/via/src/chat/mod.rs`
- `source/crates/via-voice/src/input.rs`
- `source/crates/via-app/src/lib.rs`

**Related QA**  
QA-03, QA-10, QA-05.

**Architecture use**  
Strong reference for common downstream execution/task state across modalities; insufficient to decide how VIA should represent modality-specific interaction evidence.

## DP-09 — Existing-Task vs New-Task Association

**VIA decision scope**  
Model-only classification vs deterministic candidate filter + model classify vs explicit-anchor-first + ambiguity scoring/clarification.

**Prototype observation**

- Realtime model chooses whether to invoke `spawn_thinking`, status, cancellation, etc.
- After the chosen tool, handler can use explicit `work_id` or a deterministic current owner/session Work fallback.
- Same-turn repeated delegation is deduplicated with a `turn_id → work_id` cache/submission key.
- Status on a delegated Work can create a child `control` Work; this is not the same as a general semantic follow-up reusing one logical Task.
- No inspected candidate-ranking/ambiguity/clarification engine sits before the model tool choice.

**Closest alternative**  
**A — model-only association at the semantic decision point**.

**Evidence**

- `source/crates/via-voice/src/tools/handler.rs`
- `source/crates/via-work/`
- `source/crates/via-downstream/src/session_key.rs`

**Related QA**  
QA-10, QA-03.

**Architecture use**  
Useful negative reference: deterministic id handling exists after intent selection, but does not solve VIA's broader existing-vs-new Task association problem.

## DP-10 — Intent Refinement Architecture

**VIA decision scope**  
Single-shot model vs staged normalizer/temporal binder/model/validator vs iterative retrieval/refinement.

**Prototype observation**

- Voice delegation receives an objective string authored by the realtime model.
- Settled transcript can provide fallback original request/objective.
- `via-coordinator::Intent` is objective-centric; `IntentSet` can hold multiple objectives and generate multiple Work submissions.
- No inspected staged canonical request model separates goal/action/target/parameters/capability or performs schema/state validation equivalent to VIA FR-06 through FR-10.
- Production runtime wiring of `IntentSet` fan-out was not confirmed in this inspection.

**Closest alternative**  
**A — single-shot/model-authored intent**, despite the existence of a multi-objective fan-out primitive.

**Evidence**

- `source/crates/via-voice/src/tools/handler.rs`
- `source/crates/via-coordinator/src/intent.rs`
- `source/crates/via-voice/src/tools/transcripts.rs`

**Related QA**  
QA-01, QA-10, QA-07.

**Architecture use**  
Prototype can inform fan-out/work submission mechanics after intent exists; it is weaker reference evidence for the VIA intent-refinement pipeline itself.

## DP-11 — Downstream Agent Routing Architecture

**VIA decision scope**  
LLM-only routing vs capability eligibility + semantic reranker vs ontology/deterministic scoring + reranker.

**Prototype observation**

- `HarnessRegistry` registers multiple harness implementations by backend id.
- Runtime `resolve(protocol)` is configured-id lookup after normalization.
- `via-app::HarnessBackend` represents one configured harness to the Gateway.
- Static capabilities validate what a harness can do, but the inspected route does not compare multiple eligible agents against the user request.

**Closest alternative**  
**Unknown / no direct match.** This prototype largely assumes backend selection/configuration has already occurred; it does not implement the VIA DP-11 request-time routing problem in the inspected path.

**Evidence**

- `source/crates/via-downstream/src/registry.rs`
- `source/crates/via-app/src/backend.rs`
- `source/crates/via-coordinator/src/profile.rs`

**Related QA**  
QA-01, QA-11, QA-05.

**Architecture use**  
Strong evidence for the *post-routing execution port*; little direct evidence for the *routing algorithm* VIA must choose.

## DP-12 — Downstream Agent Capability Contract

**VIA decision scope**  
Static manifest vs runtime discovery vs stable manifest + dynamic health/resource hybrid.

**Prototype observation**

- `BackendCapabilities` is a fixed seven-flag static declaration validated before normal execution.
- Descriptor validation checks identity and capability consistency.
- Runtime `HarnessHealth` separately exposes readiness/startup/failure/backoff.
- Capabilities can influence tools/prompts/behavior.
- No inspected path performs runtime capability discovery or exposes generic per-request resource requirements/availability as part of the capability contract.

**Closest alternative**  
**C-like partial — stable manifest + dynamic health**, but without the full resource/dynamic discovery dimension implied by VIA DP-12 Alternative C.

**Evidence**

- `source/crates/via-downstream/src/capability.rs`
- `source/crates/via-downstream/src/descriptor.rs`
- `source/crates/via-downstream/src/health.rs`
- `source/crates/via-downstream/src/registry.rs`

**Related QA**  
QA-11, QA-05.

**Architecture use**  
One of the strongest prototype references: it demonstrates why stable declared capability and changing runtime health should be modeled separately. VIA still needs to decide the resource and dynamic-discovery portions of its own contract.

## Cross-DP Observations

### Strongest reference evidence

The prototype is especially useful for:

- **DP-04:** common downstream port with backend-specific data/extensions;
- **DP-05:** Work lifecycle/admission/serialization mechanics, while leaving VIA resource semantics open;
- **DP-06:** provider-independent voice runtime despite different provider compositions;
- **DP-08:** common voice/text Gateway and Work plane;
- **DP-12:** separation of stable capability manifest and dynamic health.

### Areas where prototype evidence should be treated cautiously

- **DP-02:** prototype referents/surface generations do not implement the Approved Baseline's turn Interaction Timeline.
- **DP-09:** task association is largely implicit in model tool choice rather than an explicit association subsystem.
- **DP-10:** objective fan-out is not the same as canonical request refinement/validation.
- **DP-11:** configured harness lookup is not request-time multi-Agent routing.

## Implementation Discrepancies / Gaps Relevant to Traceability

1. `source/docs/architecture.md` describes multi-intent coordination, and `via-coordinator::IntentSet` is implemented, but production runtime fan-out wiring was not confirmed by code search during this analysis.
2. The common downstream contract includes progress events, but `source/crates/via-acp/src/downstream.rs` currently returns an empty event stream for `AcpHarnessSession::events()`.
3. The snapshot architecture references `VIA_REFERENCE_ARCHITECTURE.html`, but the file is absent from the snapshot; rationale depending on it cannot be independently checked here.
4. The captured ADR directory contains only `0001-placement.md`; design intent elsewhere should be attributed to implementation/docs, not assumed to be an approved ADR.

These gaps do not invalidate the prototype as a reference. They limit which conclusions can be marked `Confirmed` and prevent implementation presence from being promoted into VIA requirements without independent evaluation.
