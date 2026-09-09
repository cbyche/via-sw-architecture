# Internal Rust Voice Agent — Component Map

## Purpose

This document maps architecture-relevant Rust crates/modules in the internal prototype to their observed responsibilities. It is a reference map only; crate boundaries are not proposed VIA component boundaries unless separately selected through VIA architecture work.

Reference snapshot: `cbyche/via-internal-rust-reference`, source commit `b6032a4c472e18ec216b3e8592346ab269495342`.

## Workspace Bands

The prototype's own architecture documentation groups the workspace into dependency bands. The root `Cargo.toml` uses `members = ["crates/*", "apps/*"]`, Rust edition 2024, and a pinned Rust toolchain.

| Band | Important crates | Observed role |
| --- | --- | --- |
| Leaf / contracts | `via-protocol`, `via-catalog`, `via-log`, `via-store`, `via-lock`, `via-audio` | Stable wire/domain types, registries/catalog data, persistence/logging/audio primitives. |
| Core | `via-core`, `via-i18n` | Process-independent configuration/security/common services and localized messages. |
| Realtime / Voice | `via-realtime`, `via-realtime-openai`, `via-realtime-dashscope`, `via-realtime-local`, `via-realtime-mock`, `via-wake-word`, `via-voice` | Provider abstraction, realtime dialects, local/cloud runtime, turn/voice policy. |
| Middleware / Coordination | `via-work`, `via-coordinator`, `via-context`, `via-conversation` | Durable Work lifecycle, coordinator sessions, context/referents, memory/conversation state. |
| Downstream execution | `via-downstream`, `via-acp`, `via-backends`, `via-process`, `via-mcp-tools` | Common harness contract, ACP/process adapters, backend profile/catalog wiring, session tools. |
| Composition | `via-app`, `apps/via` | Gateway composition root, HTTP/WS server, CLI/text client, service lifecycle. |
| Architecture/test gates | `via-conformance`, `via-arch-test`, `via-e2e` | Contract fidelity, dependency-boundary checks, end-to-end verification. |

Evidence: `source/Cargo.toml`, `source/docs/architecture.md` §9.

## Component Responsibility Map

| Crate / module | Architecture responsibility it owns | Explicitly does not own / boundary | Important direction |
| --- | --- | --- | --- |
| `via-protocol` | Session mode, gateway event vocabulary, Work status/kind and cross-layer protocol types. | Provider implementation, Work execution, backend routing. | Imported by upper layers; intended as a leaf contract. |
| `via-catalog` | Static provider/backend/model catalog data and capability metadata used for validation/description. | Runtime health or semantic routing decisions. | Referenced by provider and downstream descriptors. |
| `via-audio` | Sample-rate/playback cursor and audio utility primitives. | Voice turn policy or provider semantics. | Used by voice/realtime/local pipeline. |
| `via-core` | Configuration, setup/security/common process services. | Realtime provider behavior or agent execution policy. | Core dependency below app/provider adapters. |
| `via-realtime` | Realtime provider and dialect/protocol contracts, provider registry, active provider description, session construction seams. | Conversation-level delegation policy and Work lifecycle. | Provider adapters depend on it; `via-voice` consumes its normalized surface. |
| `via-realtime-openai` | OpenAI/Azure/litellm realtime transport/dialect implementation and connection behavior. | Generic provider registry policy. | Adapter behind `via-realtime`. |
| `via-realtime-dashscope` | DashScope/Qwen realtime adapter. | Cross-provider interaction policy. | Adapter behind `via-realtime`. |
| `via-realtime-local` | On-device provider modes; componentized VAD/ASR/reasoning/TTS pipeline and local endpoint façade. | Global voice policy or Work semantics. | Implements `via-realtime` provider surface; consumes `via-audio`, catalog/core. |
| `via-realtime-mock` | Deterministic realtime provider for tests/replay. | Production model selection. | Adapter behind same provider seam. |
| `via-wake-word` | Wake-word/model-manager seam and optional engine integration. | General ASR/model turn. | Voice edge capability. |
| `via-voice::mode` | Session-mode-to-runtime plan, including no-harness/context-engine degradation. | Backend execution details. | Uses protocol mode + facts injected from app. |
| `via-voice::input` | Normalized input parts, attachment/reference merge and source-text anchors. | Timestamped screen interaction timeline. | Feeds delegation/model context. |
| `via-voice::tools` | Realtime control/delegation tool handling, stale/duplicate checks, Work submission, task status/cancel/permission surfaces. | Backend runner implementation; it consumes injected runner/availability traits. | Layer 1 calls `via-work`; app bridges Layer 3-specific runners. |
| `via-voice::tools::transcripts` | Correlates settled transcript to turn and allows bounded wait/fallback for delegation. | Partial-ASR semantic planning. | Supplies final user wording to delegation path when needed. |
| `via-voice::gate` + `announcement` | Playback-aware Injection Gate and announcement window/notification claim behavior. | Work completion itself. | Consumes Work notification state and client playback/turn state. |
| `via-voice::arbitration` | Active voice/input arbitration and ownership policy. | Global Work scheduler. | Session/frontstage coordination. |
| `via-work` | Work record, lifecycle, actor/command queue, persistence, scheduler, reminders, progress, notification state, cancellation/reconciliation. | Backend's internal plan/tools/session protocol. | Above downstream execution; runners/cancelers are injected. |
| `via-work::scheduler` | Admission using global cap, per-owner cap, optional lane key/limit. | Execution or generic OS-resource model. | Called by Work manager; lane key can serialize selected work classes. |
| `via-work::store` / `via-store` | Persistent Work state with atomic/coalesced persistence/recovery behavior. | Conversation/session ownership. | Durable state below Work manager. |
| `via-coordinator` | Coordination envelope, fixed coordinator session, turn serialization, delegation/result/control turns, permission broker integration. | Backend-internal capability/tool planning. | Depends on common downstream contract; used by app-provided Work runners. |
| `via-coordinator::intent` | Objective-only `Intent`/`IntentSet` and conversion into one Work request per intent. | Canonical VIA intent schema, staged refinement/validation. | Produces `via-work::NewWork`; runtime production call site was not confirmed. |
| `via-context` | ContextPack sections with provenance/trust/token metadata, stable referents, ordered deixis, fencing, host surface snapshots/generations. | Full turn-level timestamped Interaction Timeline as defined by VIA FR-03/FR-04. | Independent context service mounted by app/interface runtime. |
| `via-conversation` | Conversation history, memory/notes and synchronization utilities. | UI surface referent resolution; Work scheduling. | Supplies context/persistent conversational state. |
| `via-downstream` | Generic DownstreamAgent/HarnessSession contract, descriptor, capabilities, health, cancel/session/event vocabulary, registry. | Work queue, Work state, permission policy, semantic request routing. | Layer-3 port consumed by coordinator/app. |
| `via-downstream::registry` | Registration and configured id → harness resolution. | Capability/semantic multi-agent router. | Static/configured lookup only. |
| `via-acp` | ACP transport/process/session adapter implementing the common downstream seam. | Named backend policy; profile arrives as data. | Depends on `via-downstream`; generic coordination should not depend on ACP details. |
| `via-backends` | Backend-specific launch/profile/catalog wiring and adapter selection. | Cross-layer orchestration. | Supplies data/adapters to generic downstream/coordinator seams. |
| `via-mcp-tools` | Session/coordination tools exposed to compatible downstream agents. | Realtime model control tools. | Downstream integration support. |
| `via-process` | Process management primitives for launched harnesses. | Agent semantics. | Infrastructure under adapters. |
| `via-app` | Composition root, service injection, Gateway HTTP/WS surface, health, runtime startup/shutdown, bridges Layer 1 traits to Layer 3. | Policy already owned in lower crates; module docs explicitly avoid restating it. | Only app band intentionally sees multiple architectural layers. |
| `apps/via` | CLI/process boundary, `via gateway`, `via chat`, configuration/backend/service commands. | Core behavior; main is intentionally thin. | Calls `via-app` and library crates. |
| `via-conformance` | Asserts external/reference contracts catalogued by the prototype. | VIA requirement conformance. | Prototype-specific fidelity gate. |
| `via-arch-test` | Enforces selected dependency/dispatcher architecture invariants. | VIA DP closure. | Test-only architecture guard. |
| `via-e2e` | End-to-end prototype scenarios. | Approved VIA acceptance criteria unless separately adopted. | Test-only consumer of composed system. |

## Key Dependency Directions

The important architectural dependency tendencies observed are:

1. **Contracts and infrastructure point upward only through use, not callbacks into app.** Protocol/catalog/audio/core types sit below runtime crates.
2. **Provider-specific crates implement `via-realtime` seams.** Generic voice logic does not need a concrete OpenAI/DashScope/local type.
3. **Layer 1 does not directly own Layer 3.** `via-voice` takes traits such as backend availability and delegation runners; `via-app` bridges those traits to coordinator/harness implementations.
4. **Work lifecycle is above harness execution.** `via-work` owns `work_id`, state, admission, persistence, cancellation semantics, and notification state; a harness receives correlated prompt requests but does not own Work policy.
5. **Coordinator depends on a common downstream shape.** Backend-specific profiles/configuration are supplied as data rather than introducing alternate coordinator dispatch paths.
6. **Context is separate from conversation and Work.** `via-context` has different referent/surface lifetimes and responsibilities from durable conversation memory and task state.
7. **The composition root is intentionally privileged.** `via-app` is where cross-layer wiring occurs, reducing the need for lower crates to violate dependency direction.

These directions are supported by `source/docs/architecture.md`, crate-level module documentation, and the root Cargo workspace. They should be treated as implementation evidence for VIA QA-05, not as approved VIA layering.

## Ownership Boundaries Relevant to VIA Decision Points

### Partial / Streaming Input — DP-01

- Streaming ASR data exists in realtime/local components.
- `via-voice::tools::transcripts` owns the settled transcript correlation used by delegation.
- `via-voice::tools::handler` normally trusts the realtime model's delegation objective and uses final transcript only as a bounded fallback.
- No inspected component owns an incremental intent-prefetch pipeline.

### Interaction Context — DP-02

- `via-context` owns typed context, referents, surface generation, ordered deixis, and fencing.
- `via-voice::input` owns text/file input parts and anchors within text.
- No inspected component is the VIA baseline's timestamped turn Interaction Timeline.

### Capability Placement / Delegation — DP-03, DP-04

- Fast conversational/control operations remain in realtime/voice.
- Long-running/project Work is delegated through `via-work`/`via-coordinator` to `via-downstream`.
- Downstream adapters remain behind one common seam with backend-specific profile data/extensions.

### Concurrent Task Arbitration — DP-05

- `via-work::scheduler` owns global/per-owner/lane admission.
- `via-coordinator` adds per-session serialization.
- No inspected crate establishes an Agent-declared generic resource-requirement model for microphone/screen/GPU/etc.; therefore the mechanism should not be over-mapped to VIA's resource arbitration alternatives.

### Voice Runtime / Model Provider — DP-06, DP-07

- `via-realtime` abstracts provider/session/dialect.
- Cloud paths can be direct realtime S2S-style providers.
- `via-realtime-local` can decompose into VAD/ASR/reasoning/TTS but hides that behind the same realtime interface.
- Registry selection is explicit/default key resolution, not a dynamic policy optimizer.

### Voice/Text Unification — DP-08

- `via chat` and voice share the Gateway realtime WebSocket, transcript/event vocabulary, Work plane, and result-announcement mechanics.
- Client capability flags keep text-specific capture/playback behavior distinct.
- No explicit VIA `UserTurn` + `InteractionAnchor` canonical abstraction was confirmed.

### Task Association / Intent / Routing / Capability — DP-09..DP-12

- Same-turn delegation duplication is suppressed with `turn_id → work_id` cache and a submission key.
- Status/cancel tools select/query Work after the model has chosen the operation.
- `IntentSet` exists as an objective fan-out primitive but is not a staged canonical intent-refinement pipeline.
- Harness routing is configured id lookup, not semantic multi-Agent selection.
- Static capability descriptor plus dynamic health is a strong explicit contract, but runtime resource discovery is not part of the inspected seven-flag capability shape.

## Evidence Cautions

- `source/docs/architecture.md` references `VIA_REFERENCE_ARCHITECTURE.html`, which is not present in the snapshot.
- The snapshot's ADR directory contains only `source/docs/adr/0001-placement.md`.
- `AcpHarnessSession::events()` currently returns an empty stream even though the common downstream contract contains progress events; this should be treated as an implementation gap, not hidden by the component map.
- `via-coordinator::IntentSet` is implemented and tested, but its production runtime wiring was not confirmed in this inspection.
