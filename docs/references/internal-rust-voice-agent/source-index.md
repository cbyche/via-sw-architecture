# Internal Rust Voice Agent — Source Index

## Purpose

This index identifies the highest-value source and documentation locations to inspect when using the internal Rust prototype as architecture evidence. The list is ordered by architectural usefulness, not by build order.

Reference snapshot: `cbyche/via-internal-rust-reference`, source commit `b6032a4c472e18ec216b3e8592346ab269495342`.

## Start Here

| Path | Why it matters |
| --- | --- |
| `SNAPSHOT.md` | Establishes provenance, source commit/date, exclusions, and the critical rule that this prototype is a reference input rather than approved VIA architecture. |
| `source/README.md` | High-level product/runtime description: modes, downstream-harness principle, build/run surface, and documented implementation status. |
| `source/docs/architecture.md` | Primary design narrative for layers, modes, Work, Context Engine, downstream seam, provider/local runtime, crate graph, concurrency, and placement. |
| `source/Cargo.toml` | Concrete workspace/crate inventory and dependency/tooling choices; useful for checking whether documented component boundaries have corresponding build boundaries. |
| `source/docs/adr/0001-placement.md` | The only ADR present in the snapshot; explains why VIA is a standalone workspace and how ARGO knowledge is consumed without a default code dependency. |
| `source/docs/deviations/README.md` | Index for port/adaptation deviations and pending fidelity items; useful when README/architecture claims need implementation qualification. |

## Architecture/Fidelity Documents

| Path | Why it matters |
| --- | --- |
| `source/docs/fidelity.md` | Describes what was ported/adapted/dropped and is important for distinguishing inherited behavior from prototype-specific design. |
| `source/docs/reference/contracts.md` | Human-readable external contract catalogue; useful for understanding which prototype behaviors were intentionally preserved as externally observable contracts. |
| `source/docs/reference/contracts.json` | Machine-readable contract source used by conformance tests; useful for evidence on exact prototype behavior, but not a VIA requirements source. |
| `source/docs/deviations/phase-2.md` | Downstream seam/ACP-related deviations; important for capability/adapter/cancellation analysis. |
| `source/docs/deviations/phase-3.md` | Work/coordinator deviations; important for task lifecycle, persistence, scheduling, and cancellation. |
| `source/docs/deviations/phase-4.md` | Conversation/memory/notes behavior and synchronization differences. |
| `source/docs/deviations/phase-5-via-voice.md` | Voice runtime/tool/transcript/input/announcement deviations. |
| `source/docs/deviations/phase-5-via-app.md` | Gateway composition, HTTP/WS, identity/session and application-layer deviations. |
| `source/docs/deviations/phase-7.md` | ContextPack/referent/ordered-deixis implementation evidence and context-specific limitations. |
| `source/docs/deviations/phase-8-via-realtime-local.md` | Local provider/pipeline design and verification limitations. |

> Analysis note: `source/docs/architecture.md` references `VIA_REFERENCE_ARCHITECTURE.html`, but that artifact is not present in this snapshot. Do not treat conclusions that depend solely on that missing artifact as independently verified.

## Composition Root and Executable Surface

| Path | Why it matters |
| --- | --- |
| `source/crates/via-app/src/lib.rs` | Best entry point for system composition. Documents which lower-layer responsibilities are wired together and which policies remain owned elsewhere. |
| `source/crates/via-app/src/services.rs` | Service injection/composition map; useful for concrete runtime dependency inspection. |
| `source/crates/via-app/src/runtime.rs` | Startup/shutdown order and lifecycle behavior. |
| `source/crates/via-app/src/backend.rs` | Gateway's Layer-3 view; distinguishes frontend-only vs configured harness and exposes static description/dynamic health. |
| `source/crates/via-app/src/realtime/` | WebSocket-side integration between Gateway, voice engine, provider/session, and client frames. |
| `source/apps/via/src/main.rs` | Confirms the process entry point is thin and delegates behavior to library code. |
| `source/apps/via/src/commands/` | CLI command composition for gateway/chat/config/backend/service/MCP surfaces. |
| `source/apps/via/src/chat/mod.rs` | End-to-end text client over the same realtime Gateway; important for voice/text unification analysis. |
| `source/apps/via/src/chat/protocol.rs` | Text connect flags, Work commands, task event formatting, and shared realtime session protocol. |
| `source/apps/via/src/chat/client.rs` | Identity cookie, health, task listing/cancel HTTP behavior; helps separate logical session/task lifetimes from transport. |

## Layer 1 — Realtime and Voice

| Path | Why it matters |
| --- | --- |
| `source/crates/via-realtime/src/provider.rs` | Core provider/protocol/session abstraction. Read first for DP-06/DP-07. |
| `source/crates/via-realtime/src/registry.rs` | Provider key/alias/default resolution and advertised provider/model capabilities; key evidence that current selection is configured rather than an optimizer. |
| `source/crates/via-realtime/src/session/` | Realtime session state/watchdog/correlation behavior. |
| `source/crates/via-realtime-openai/` | Concrete OpenAI/Azure/litellm adapter and dialect behavior. Useful to test whether generic provider abstractions leak provider-specific assumptions. |
| `source/crates/via-realtime-dashscope/` | Second cloud adapter; useful for comparing provider portability. |
| `source/crates/via-realtime-local/src/lib.rs` | Local provider overview and the two local modes (componentized pipeline vs local endpoint). |
| `source/crates/via-realtime-local/src/stages.rs` | VAD/streaming-ASR/reasoning/TTS traits and partial transcript representation. Central for DP-01 and DP-06. |
| `source/crates/via-realtime-local/src/machine.rs` | Full-duplex illusion, barge-in, stream cancellation, event scheduling for the decomposed local pipeline. |
| `source/crates/via-voice/src/mode.rs` | Exact session-mode composition and degradation behavior. |
| `source/crates/via-voice/src/input.rs` | Normalized input parts, attachments and source-text anchors. Important for understanding what is and is not the interaction-context model. |
| `source/crates/via-voice/src/tools/transcripts.rs` | Settled transcript/turn correlation and bounded waiting; primary evidence for partial-vs-final semantic processing. |
| `source/crates/via-voice/src/tools/handler.rs` | Most important voice control-flow file: tool selection result handling, Work creation, same-turn deduplication, task status/cancel, permissions, delegation inputs. |
| `source/crates/via-voice/src/gate.rs` | Injection Gate predicate and playback-aware result delivery. Important for asynchronous voice completion. |
| `source/crates/via-voice/src/announcement/` | Result claim/window/delivery mechanics around the gate. |
| `source/crates/via-voice/src/arbitration.rs` | Active voice/input ownership and arbitration behavior. |
| `source/crates/via-voice/src/prompt.rs` | Realtime model instruction/context composition; useful when checking which context reaches the realtime model before delegation. |

## Layer 2 — Work, Coordination, Conversation, Context

| Path | Why it matters |
| --- | --- |
| `source/crates/via-work/src/lib.rs` | Architectural overview of unified Work lifecycle, ownership, concurrency and cancellation semantics. |
| `source/crates/via-work/src/record.rs` | Concrete Work record fields and what state is persisted/public. |
| `source/crates/via-work/src/manager/` | Work actor/command processing and lifecycle authority. Important for QA-03 and DP-05. |
| `source/crates/via-work/src/scheduler.rs` | Global/per-owner/lane admission rules; primary evidence for concurrency arbitration. |
| `source/crates/via-work/src/store.rs` | Persistence, atomic/coalesced save, quarantine/recovery behavior. |
| `source/crates/via-work/src/reconcile.rs` | Cancellation/recovery reconciliation evidence. |
| `source/crates/via-work/src/projector.rs` | How terminal Work result re-enters conversation state. |
| `source/crates/via-coordinator/src/lib.rs` | Coordinator architecture: fixed session, serialization, backend envelope and boundary with harness internals. |
| `source/crates/via-coordinator/src/intent.rs` | Objective-only `IntentSet` and Work fan-out primitive. Important for DP-10, with runtime-wiring caveat. |
| `source/crates/via-coordinator/src/profile.rs` | Backend facts supplied to generic coordinator as data; key DP-04/DP-11 evidence. |
| `source/crates/via-coordinator/src/permission.rs` | Permission broker/coordination behavior. |
| `source/crates/via-conversation/` | Conversation/memory/notes/synchronization responsibilities; inspect when separating conversational state from task/context state. |
| `source/crates/via-context/src/lib.rs` | Context Engine ownership and public surface. |
| `source/crates/via-context/src/pack.rs` | ContextPack section/provenance/trust/token structure. |
| `source/crates/via-context/src/referent.rs` | Stable referent IDs, lifetimes/staleness and target resolution. |
| `source/crates/via-context/src/deixis.rs` | Ordered-deixis resolution over a host surface snapshot/generation. |
| `source/crates/via-context/src/fence.rs` | Untrusted-content fencing/provenance handling. |

## Layer 3 — Downstream Agent Integration

| Path | Why it matters |
| --- | --- |
| `source/crates/via-downstream/src/agent.rs` | Canonical generic downstream port: descriptor/open/health + prompt/events/cancel session contract. Start here for DP-04. |
| `source/crates/via-downstream/src/capability.rs` | Seven static backend capability flags and consistency rules. Start here for DP-12. |
| `source/crates/via-downstream/src/descriptor.rs` | Startup-time validation of backend identity/capability declaration. |
| `source/crates/via-downstream/src/health.rs` | Dynamic runtime readiness/failure/backoff vocabulary; complements static manifest. |
| `source/crates/via-downstream/src/registry.rs` | Configured backend-id → harness lookup. Critical evidence that this is not a request-time semantic Agent router. |
| `source/crates/via-downstream/src/session_key.rs` | Stable backend session addressing; important for recovery/follow-up analysis. |
| `source/crates/via-downstream/src/event.rs` | Normalized harness activity/progress vocabulary. |
| `source/crates/via-downstream/src/cancel.rs` | Cancellation scope/outcome contract, including requested vs confirmed. |
| `source/crates/via-acp/src/downstream.rs` | Concrete ACP adapter behind the common port; session resume/fallback and currently empty `events()` stream are important implementation evidence. |
| `source/crates/via-acp/src/client.rs` | ACP process/session protocol behavior beneath the adapter. |
| `source/crates/via-acp/src/registry.rs` | Persisted/reusable ACP session records. |
| `source/crates/via-backends/` | Named backend profile/launch data; important for seeing how backend-specific variation stays out of generic ports. |
| `source/crates/via-mcp-tools/` | Session coordination tools exposed to compatible backend agents. |

## Architecture/Test Gates

| Path | Why it matters |
| --- | --- |
| `source/crates/via-arch-test/` | Executable architecture-boundary tests. Prefer these over prose when checking whether dependency restrictions are actually enforced. |
| `source/crates/via-conformance/` | External-contract fidelity tests for the prototype's port/reference behavior. Do not confuse these with VIA requirement tests. |
| `source/crates/via-e2e/` | End-to-end assembled-system scenarios; useful to confirm whether documented component primitives are actually wired in production-like paths. |
| `source/.github/workflows/ci.yml` | Shows which test/architecture gates are expected to run in CI. |

## Recommended Inspection Order by VIA Decision Point

| VIA DP | Read first | Then inspect |
| --- | --- | --- |
| DP-01 Partial/streaming input | `via-voice/tools/transcripts.rs`, `via-realtime-local/stages.rs` | `via-voice/tools/handler.rs`, realtime session/event handlers |
| DP-02 Interaction context | `via-context/lib.rs`, `referent.rs`, `deixis.rs` | `via-voice/input.rs`, `via-voice/prompt.rs`, host/surface integration in app |
| DP-03 Capability placement | `via-voice/mode.rs`, `tools/handler.rs` | `via-coordinator`, `via-mcp-tools` |
| DP-04 Downstream integration | `via-downstream/agent.rs` | `via-acp/downstream.rs`, `via-backends/`, `via-coordinator/profile.rs` |
| DP-05 Concurrency/resource arbitration | `via-work/scheduler.rs`, manager | coordinator serialization, any future resource metadata in downstream/backend definitions |
| DP-06 Voice runtime composition | `via-realtime/provider.rs`, local `lib.rs`/`stages.rs` | cloud provider adapters, voice arbitration/gate |
| DP-07 Model gateway selection | `via-realtime/registry.rs`, `via-catalog/` | provider settings/health and app composition |
| DP-08 Voice/text unification | `apps/via/chat/mod.rs`, `chat/protocol.rs` | Gateway realtime handling and `via-voice/input.rs` |
| DP-09 Task association | `via-voice/tools/handler.rs` | Work query/order semantics, conversation/task projector, downstream session key |
| DP-10 Intent refinement | `via-coordinator/intent.rs`, voice tool handler | e2e/runtime call sites; any schema validation/planning modules |
| DP-11 Agent routing | `via-downstream/registry.rs`, `via-app/backend.rs` | backend profiles/catalog and any coordinator selection code |
| DP-12 Capability contract | `via-downstream/capability.rs`, `descriptor.rs`, `health.rs` | backend catalog definitions and adapter capability projection |

## Known Follow-Up Inspection Targets

The following areas deserve another code-level pass before drawing stronger conclusions:

- production call sites of `via_coordinator::IntentSet` and whether one utterance truly fans out to multiple Work items;
- the integration point that should project ACP/session activity into `HarnessSession::events()`/Work activity;
- any resource metadata outside `via-downstream::BackendCapabilities` that could affect DP-05;
- any request-time agent selection code outside `HarnessRegistry` that could change the DP-11 assessment;
- host-side `SurfaceSource` integration that feeds real screen objects into `via-context`;
- whether any event carries timestamped screen/pointer evidence not exposed through the inspected Context Engine APIs;
- e2e tests covering substantive follow-up on an existing delegated task rather than status/cancel control Work.
