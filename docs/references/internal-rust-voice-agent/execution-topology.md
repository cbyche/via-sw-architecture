# Internal Rust Voice Agent — Execution Topology by Session Mode

## Status and Scope

This document is an architecture observation of the existing Rust prototype in `cbyche/via-internal-rust-reference`. It is **not** a VIA architecture decision or requirement.

The analysis follows the current implementation wiring rather than assuming that architecture documentation and runtime behavior are identical. Where the implementation only suggests an interpretation, the statement is marked **Inferred**. Where documentation and code disagree, both are recorded explicitly.

The most important conclusion is that the prototype contains **two different notions of the four session modes**:

1. an **intended mode topology** expressed in `SessionMode`, `ModePlan`, `RealtimeSession`, documentation, and tests; and
2. the **currently wired production topology** in `apps/via`, where the connect-selected mode is not propagated into `RealtimeEngine` or `RealtimeSession` and the production binary does not compose `via-context`.

The distinction is architecture-relevant. The mode enum is designed as a structural boundary, but the current runnable composition does not enforce that boundary end to end.

Primary evidence:

- `source/docs/architecture.md` §2–§6
- `source/crates/via-protocol/src/session.rs`
- `source/crates/via-voice/src/mode.rs`
- `source/crates/via-app/src/realtime/frames.rs`
- `source/crates/via-app/src/realtime/connection.rs`
- `source/crates/via-app/src/realtime/engine.rs`
- `source/apps/via/src/gateway/engine.rs`
- `source/apps/via/src/gateway/compose.rs`
- `source/apps/via/src/gateway/delegation.rs`
- `source/apps/via/src/gateway/frontend.rs`
- `source/crates/via-realtime/src/session/mod.rs`
- `source/crates/via-voice/src/tools/{catalog,handler,transcripts}.rs`
- `source/crates/via-voice/src/announcement/manager.rs`
- `source/crates/via-coordinator/src/{runtime,envelope}.rs`
- `source/crates/via-downstream/src/agent.rs`
- `source/crates/via-acp/src/downstream.rs`
- `source/apps/via/Cargo.toml`
- `source/crates/via-voice/Cargo.toml`

---

## 1. Executive Finding

### Intended architecture

The prototype documentation and core types define the four modes as a **connect-time choice of which layers are mounted**:

| Mode | Intended execution topology |
| --- | --- |
| `dictation` | capture / VAD / ASR only; no model response, no tools, no speech output, no Work/harness |
| `direct` | realtime model + local/control tools; no `spawn_thinking`, no harness delegation |
| `agent` | realtime model + full tool surface including `spawn_thinking` + WorkManager + Coordinator + DownstreamAgent |
| `interface` | realtime model + Context Engine + control tools; no normal downstream delegation |

Evidence: `source/docs/architecture.md` §2; `source/crates/via-protocol/src/session.rs`; `source/crates/via-voice/src/mode.rs`.

### Current production wiring

The current `apps/via` composition does **not** propagate `connect.mode` into the actual model session:

1. `ClientFrame::Connect` parses a `SessionMode`.
2. `via-app::realtime::connection::Session::on_connect` stores a `ModePlan` on the connection.
3. `Session::ensure_engine` builds `EngineContext`, but `EngineContext` contains **no mode field**.
4. `apps/via/src/gateway/engine.rs::RealtimeEngine::open` constructs a new mode with:

   `ModePlan::new(SessionMode::default(), gateway.backend.enabled())`

   where `SessionMode::default()` is `Agent`.
5. The `SessionOptions` passed to `RealtimeSession` also leaves `mode` at its default `Agent` value.

**Confirmed:** the connect-selected mode currently affects connection metadata / health reporting but does not determine the production realtime model/tool topology.

There is a second production gap for `interface`: `apps/via/Cargo.toml` has no `via-context` dependency, while `via-voice` lists `via-context` only as a **dev-dependency** for tests. The composition root does not instantiate a `ContextEngine`.

**Confirmed:** the current runnable binary does not execute the documented `interface → Context Engine` path.

---

## 2. Connect and Session Mode Decision

### 2.1 Where the mode comes from

The WebSocket connection is established first on `/api/realtime`; `sessionId` is a query parameter and defaults to `main`. The session mode is not part of the URL. It arrives later in the client `connect` frame.

Call path:

```text
GET /api/realtime?sessionId=...
  -> via_app::realtime::upgrade
  -> connection::run(... owner_id, session_id)
  -> ClientFrame::Connect
  -> Connect.mode : SessionMode
  -> Session::on_connect
  -> ModePlan::new(connect.mode, backend.enabled())
```

Evidence:

- `source/crates/via-app/src/realtime/mod.rs`
- `source/crates/via-app/src/realtime/frames.rs`
- `source/crates/via-app/src/realtime/connection.rs`

A missing or invalid mode is normalized to `SessionMode::default()`, which is `Agent`.

### 2.2 Requested versus effective mode

`ModePlan` distinguishes requested and effective mode. The two documented degradations are:

- `agent` + no harness → `direct`;
- `interface` + no Context Engine → `direct`.

In a normal workspace build `ModePlan::new` assumes the Context Engine exists and only the harness fact is supplied. `ModePlan::with_context_engine` is the explicit path for reporting a missing Context Engine.

Evidence: `source/crates/via-voice/src/mode.rs`.

### 2.3 Can the mode change at runtime?

**Documented intent: No.**

`SessionMode` documentation states that the mode is chosen at connect time and fixed for the session lifetime. `source/docs/architecture.md` calls this the reason the mode is a protocol type rather than a runtime flag.

**Current code behavior: the fixed-lifetime rule is not enforced.**

`connection::Session::handle` accepts `ClientFrame::Connect` whenever such a frame arrives. `on_connect` unconditionally replaces `self.mode`. No inspected guard rejects a second `connect` frame or a changed mode.

A second `connect` with the same provider can therefore update connection-reported mode while leaving the already-open engine intact. A provider change closes the engine, but a subsequently opened engine still reconstructs its mode from `SessionMode::default()` rather than the new `connect.mode`.

**Confirmed discrepancy:** mode immutability is a documented contract, not an enforced runtime invariant in the current connection implementation.

---

## 3. The Shared Runtime Path

Before separating the four intended modes, the current production path can be summarized as follows.

```text
Client socket
   |
   | connect(mode, provider, client states)
   v
via-app connection Session
   |  owns connection state, voice slot, SharedGate
   |
   | audio.append / input.message
   v
ensure_engine()
   |
   v
RealtimeEngineFactory
   |
   | EngineContext(provider, owner_id, session_id, outbound, gate, states)
   | NOTE: no mode
   v
apps/via RealtimeEngine::open
   |
   +-> mode = ModePlan(default Agent, backend.enabled)
   +-> SessionOptions(default mode = Agent)
   +-> packaged prompt + tool catalog
   v
via-realtime RealtimeSession
   |
   +-> provider WebSocket / local pipeline
   +-> serialized response queue
   +-> provider event stream
   v
apps/via provider pump
   |
   +-> audio/transcript to client
   +-> function_call -> ToolCallHandler
   +-> speech/turn correlation
   |
   +------------------------+
                            |
                    spawn_thinking only
                            v
                        WorkManager
                            |
                            v
                     DelegationRunner
                            |
                            v
                       Coordinator
                            |
                            v
                    DownstreamAgent /
                     HarnessSession
                            |
                            v
                      Work terminal
                            |
                            v
                  AnnouncementManager
                            |
                            v
                  realtime inject/speak
                            |
                            v
                         client
```

The connection loop, provider pump, tool handling, Work execution, and announcement delivery intentionally run in separate tasks or actors so a long model/tool/backend operation does not stop the socket from processing playback receipts and interruption events.

---

## 4. Dictation Mode

### 4.1 Intended topology

```text
user speech
   -> audio append
   -> VAD / streaming ASR
   -> transcript delta/final
   -> client

NO response.create
NO frontend LLM turn
NO frontend tools
NO Work
NO DownstreamAgent
NO TTS
```

`RealtimeSession::response` explicitly checks the session mode. In `Dictation`, response-producing operations return `Skipped / NoModelTurn` without writing a response request to the provider. Audio append and audio commit remain available.

Evidence:

- `source/crates/via-protocol/src/session.rs`
- `source/crates/via-realtime/src/session/mod.rs`
- `source/crates/via-voice/src/mode.rs`

### 4.2 Model invocation count

**Intended: 0 model invocations.**

The documentation is explicit that a dictation provider may be plain streaming ASR. The local pipeline description calls this VAD + ASR with no LLM and no TTS.

### 4.3 Context and tool path

**Intended:** no tool catalog and no model prompt are needed for the dictation path. Conversation turn correlation and transcript events still exist at the gateway/voice layer.

### 4.4 Task/progress/cancel/session ownership

- Media/realtime session: `via-realtime`.
- Voice turn/transcript correlation: `apps/via::gateway::engine` + `via-voice` turn types.
- Current-interaction cancel/barge-in: realtime/voice session.
- Durable Work: intended not mounted.
- Downstream session: intended absent.

### 4.5 Latency/resource intent

**Confirmed:** dictation has the clearest explicit optimization rationale of all four modes. `source/docs/architecture.md` and `SessionMode` call it the cheapest local path because it removes LLM and TTS entirely.

### 4.6 Current production reality

**Confirmed discrepancy:** requested `dictation` is not propagated into `RealtimeEngine` or `SessionOptions`. The production engine opens with `SessionMode::Agent` and therefore does not activate `RealtimeSession`'s `NoModelTurn` short-circuit.

`NoModelEngine` exists as an app-level abstraction/test shape, but no production `apps/via` composition path was found that selects it from `connect.mode`.

Therefore the prototype contains a working **dictation-capable lower-level session primitive**, but the current production connection-to-engine wiring does not realize the documented dictation topology.

---

## 5. Direct Mode

### 5.1 Intended topology

```text
user speech
   -> realtime provider/model
   -> model answer
      OR
      -> one of the control tools
      -> local / Work-control operation
      -> function output
      -> realtime model continuation
   -> audio/transcript response
   -> client

NO spawn_thinking
NO ordinary downstream handoff
```

`ModePlan::declares_tool` removes `spawn_thinking` in Direct mode but keeps the other control tools.

The intended Direct tool surface includes:

- `get_current_time`
- `memory`
- `notes`
- `schedule_reminder`
- `cancel_agent_task`
- `get_agent_task_status`
- `respond_agent_permission`
- conditionally `enter_sleep`

Evidence: `source/crates/via-voice/src/tools/catalog.rs`, `source/crates/via-voice/src/mode.rs`.

### 5.2 Does Direct own real tool execution?

**Yes, for the control-tool surface.**

Direct is not a pure model-only path. `ToolCallHandler` executes or initiates real operations:

- time is computed locally;
- memory and notes call `via-conversation` implementations;
- reminders create scheduled Work;
- task status/cancel operate on the global `WorkManager`;
- permission replies may relay to a backend-owned pending authorization;
- sleep alters the client interaction state.

What Direct is intended **not** to own is arbitrary domain/device execution behind `spawn_thinking`.

### 5.3 Boundary leak: scheduled task

There is an important exception to the statement “Direct never delegates.”

`schedule_reminder` remains available in Direct. When its `type` is `task`, `ToolCallHandler` asks `DelegationRunners::scheduled_task_runner()`. In a Gateway with a configured Coordinator, `CoordinatorRunners` returns the same downstream-backed `DelegationRunner` used by normal delegation.

Therefore a Direct session can schedule a future task that later executes through the downstream Coordinator even though `spawn_thinking` is not declared.

**Confirmed discrepancy / boundary leak:** Direct prevents immediate model-selected `spawn_thinking`, but the current control-tool surface does not form a strict “no downstream ever” boundary.

The same observation applies to `interface`, because it uses the same control-tool set.

### 5.4 Model invocation count

For the intended Direct path:

| Scenario | VIA-visible frontend model response cycles |
| --- | ---: |
| Plain conversational answer | normally 1 |
| One local/control tool | normally at least 2: initial tool call + continuation after function output |
| Several sequential tool calls | 2+; depends on model/tool chain |
| Progress/result spoken from pre-existing Work | additional response cycle(s) may occur |

A `send_function_output` with `create_response=true` creates another realtime response. Most ordinary tool results use this form.

**Inferred:** “model invocation” is counted here as one frontend realtime response cycle. Provider internals may implement that response with a different number of physical model calls; the prototype contract does not expose that detail.

### 5.5 Context propagation

Intended prompt assembly can include policy, persona, user preferences, memory, and runtime context.

Current production `RealtimeEngine::open`, however, calls `build_packaged_frontend_instructions` with:

- `RawClientContext::default()` after normalization;
- an empty memory slice.

Only client `states` are copied into the separate tool-client context. `timeZone`, locale, and working directory from the connect frame are explicitly documented in code as “not threaded through yet.”

**Confirmed:** current production Direct/Agent frontend context is materially thinner than the prompt architecture supports.

### 5.6 Ownership

- Frontend reasoning: realtime model.
- Control-tool dispatch: `via-voice::ToolCallHandler`.
- Memory/notes behavior: `via-conversation`.
- Reminder/task state: `via-work::WorkManager`.
- Existing Work status/cancel: `WorkManager`, with Coordinator involvement for already-delegated Work.
- Realtime session state: `via-realtime`.
- Voice turn/output arbitration: `via-voice` / app gateway engine.

This means Direct avoids ordinary new delegation but does **not** detach from shared Work state.

### 5.7 Latency intent

**Confirmed:** architecture documentation explicitly calls direct answers the “fast path” and says no task is created for them. Eliminating Work admission, Coordinator, downstream transport, and background result reinjection is the primary intended latency advantage.

The implementation also keeps text submission, provider event handling, and tool dispatch off the socket loop so interaction-control events remain responsive.

---

## 6. Agent Mode

### 6.1 Intended topology: direct answer branch

Agent starts exactly like Direct. The realtime model remains free to answer itself without creating Work.

```text
user speech
   -> realtime model
   -> direct response
   -> client
```

This is why the architecture describes Agent as “fast-path answers plus delegated work,” not “always delegate.”

### 6.2 Intended topology: delegation branch

```text
user speech
   -> realtime model                         [frontstage reasoning]
   -> function call: spawn_thinking
   -> ToolCallHandler
      - stale/duplicate checks
      - permission/backend checks
      - objective + input references
   -> WorkManager.create
   -> immediate function-output receipt
   -> realtime model continuation            [short acknowledgement]

BACKGROUND:
WorkManager admission
   -> DelegationRunner
   -> Coordinator.run
   -> fixed per-owner/backend HarnessSession
   -> downstream agent prompt                [task reasoning/execution]
      -> optional downstream/native/project delegation
      -> result
   -> Coordinator finalization if needed
   -> Work completed/failed
   -> AnnouncementManager
   -> inject_result / speak
   -> realtime model response                [user-facing delivery]
   -> client
```

### 6.3 When does the downstream handoff happen?

The handoff does **not** happen at speech end and does not happen merely because the session is Agent mode.

It occurs only after the **frontend realtime model chooses `spawn_thinking`** and the handler accepts that call. The handler creates a Work containing a downstream-backed runner. Actual Layer-3 entry happens when WorkManager admits the Work and `DelegationRunner::run` calls `Coordinator::run`, which eventually calls `HarnessSession::prompt`.

This is an important architecture property: the realtime model is the first decision point for “answer here versus hand off.”

### 6.4 Where is primary reasoning authority?

There are two authorities at different stages.

#### Interaction / handoff authority

The **frontend realtime model** has primary authority over the immediate interaction. It receives the user turn and decides whether to:

- answer directly;
- call a local/control tool;
- call `spawn_thinking` with an objective.

The current prototype does not place a separate deterministic Agent Router in front of that choice.

#### Delegated task authority

Once `spawn_thinking` has been admitted, the **Downstream Agent / HarnessSession** becomes the primary task reasoning and execution authority.

`Coordinator` wraps the request, serializes turns, enforces the response protocol, coordinates permission/delegation lifecycle, and parses the result. It does not prescribe which downstream tools to use, how to plan, or whether to sub-agent.

This is explicit in `via-coordinator` and `via-downstream` documentation.

**Confirmed conclusion:** Agent mode is a two-stage authority model — realtime model for conversational routing/handoff, downstream agent for delegated task reasoning/execution.

### 6.5 Model invocation count

The prototype exposes two different model boundaries, so counts must be separated.

#### Frontend realtime model

For a normal delegated request, the visible sequence is typically:

1. initial user-turn response that emits `spawn_thinking`;
2. continuation after the handler returns the fast `accepted` function output;
3. later result delivery via `inject_result` or `speak` after Work completion.

Therefore a successful delegated request normally implies **at least three frontend realtime response cycles**, in addition to downstream reasoning.

Retries, progress speech, permission interaction, or extra tool calls can add more cycles.

#### Downstream agent

VIA can count calls to `HarnessSession::prompt`, not the agent's internal model invocations.

A simple Coordinator run needs at least one downstream prompt. The Coordinator may add:

- up to two protocol-retry prompts when the result is not deliverable;
- a result-presentation prompt after a detected/nested delegation;
- one recovery retry in a fresh coordinator session for a pristine empty response.

If the downstream agent opens its own project/subagent execution, its internal model/tool-call count is opaque to VIA.

**Confirmed:** the exact physical model invocation count inside a Downstream Agent cannot be determined from the common harness contract.

### 6.6 Context propagation to the frontend model

The type-level architecture supports a rich frontend prompt, but the current production engine initializes it with packaged policy/persona, default client context, and no memory documents.

Per-turn ASR is recorded in `TurnTranscripts`. Streaming transcript deltas are sent to clients but inspected code does not use them for speculative semantic processing. Settled ASR is primarily a correlation/fallback source for delegation.

### 6.7 Context propagation to the downstream agent

`CoordinationRequest` is capable of carrying:

- original ASR;
- objective;
- user memories/preferences;
- recent conversation;
- active Work;
- time zone;
- working directory;
- attachments;
- task/session/turn identifiers.

However the current `DelegationRunner` constructs `CoordinationRequest` with only:

- `original_request`;
- `objective`;
- Work id;
- voice session id;
- turn id;
- default delivery settings.

The richer fields are left at their defaults. In addition, `DelegationRequest.input_parts` are not converted to `TurnOptions.attachments` in the inspected production runner.

**Confirmed discrepancy:** the downstream envelope type supports substantially richer context than the current production handoff actually supplies.

### 6.8 Task, progress, cancel, and session ownership

| State | Owner |
| --- | --- |
| Realtime provider socket / response correlation | `via-realtime::RealtimeSession` |
| Current voice turn / ASR correlation | app gateway engine + `via-voice` turn types |
| Playback / interruption / result-delivery gate | `via-voice` + connection-owned `SharedGate` |
| Work identity / lifecycle / persistence / admission | `via-work::WorkManager` |
| Downstream coordination conversation | `via-coordinator::Coordinator` |
| Backend coordinator session identity | fixed `SessionKey::coordinator(protocol, owner)` |
| Backend execution internals | Downstream Agent |
| Cancel state | WorkManager; Coordinator/Harness only supplies cancellation outcome/confirmation path |
| User-visible background result delivery | AnnouncementManager + realtime frontend |

The downstream coordinator session is deliberately **not** the voice WebSocket session. It survives voice-session and Work-id changes.

### 6.9 Latency intent

Agent contains several explicit latency-oriented choices:

- direct answers remain on the same fast path as Direct mode;
- `spawn_thinking` returns an intake receipt without waiting for backend completion;
- the frontend model can speak an acknowledgement after that receipt;
- Work runs outside the conversational turn;
- when a downstream delegation starts, the Coordinator releases its session lane while the target runs;
- terminal result delivery is asynchronous and Injection-Gate controlled;
- provider/tool pumps are not blocked by Work execution.

The optimization target is therefore not “one model invocation.” It is **short time-to-first-reaction while long work proceeds asynchronously**.

---

## 7. Interface Mode

### 7.1 Intended topology

The documented design is:

```text
user speech
   -> realtime model
   -> Context Engine
      - ContextPack
      - stable referents
      - ordered deixis
      - host-supplied surface snapshot
   -> control/host affordance execution
   -> response

NO normal DownstreamAgent delegation
```

The stated purpose is to turn expressions such as “that one” into concrete host UI targets.

### 7.2 Intended model invocation count

A simple interaction should require one frontend realtime response cycle, with additional cycles if a control tool is called. No downstream-agent prompt is intended for ordinary Interface operation.

**Inferred:** the intended latency advantage over Agent is removal of downstream handoff while retaining richer local interaction context. Documentation says the Context Engine “pays for itself” on the fast path, but no mode-specific latency SLO is stated.

### 7.3 Current production reality

The documented Interface execution path is not composed in the runnable binary.

Evidence:

- `apps/via/Cargo.toml` does not depend on `via-context`.
- `via-voice/Cargo.toml` places `via-context` under dev-dependencies only, for an equality/drift test.
- `gateway/compose.rs` does not instantiate a `ContextEngine` or `SurfaceSource`.
- `RealtimeEngine::open` does not receive a ContextEngine or mode.

**Confirmed:** production `interface` currently has no execution path from speech to `via-context` to a host affordance.

### 7.4 Context and action ownership gap

`via-context` itself implements ContextPack/referents/deixis/fencing, but the inspected production code does not show a host UI action dispatcher consuming those referents.

Therefore the statement “Interface voice drives the host UI” is currently an architectural target/type-level design, not a demonstrated end-to-end production path.

### 7.5 Same scheduled-task boundary leak as Direct

If Interface were wired through the existing `ModePlan`, it would receive the control tools, including `schedule_reminder`. With a globally configured Coordinator, `type=task` can obtain a downstream-backed scheduled runner.

Thus even the intended Interface tool surface is not a mathematically strict “Layer 3 unreachable” boundary without an additional guard.

---

## 8. Current Production Topology by Requested Mode

Because the connect-selected mode does not cross `EngineContext`, requested modes collapse at the production engine boundary.

### Backend/harness configured

`RealtimeEngine::open` reconstructs `ModePlan(default Agent, true)`.

| Requested connect mode | Connection/health mode | Actual engine tool topology | Actual RealtimeSession mode | Context Engine |
| --- | --- | --- | --- | --- |
| `dictation` | reports Dictation | Agent tool surface including `spawn_thinking` | Agent | absent |
| `direct` | reports Direct | Agent tool surface including `spawn_thinking` | Agent | absent |
| `agent` | reports Agent | Agent tool surface including `spawn_thinking` | Agent | absent |
| `interface` | reports Interface | Agent tool surface including `spawn_thinking` | Agent | absent |

### No backend/harness configured

`RealtimeEngine::open` reconstructs `ModePlan(default Agent, false)`, whose **tool plan** degrades to Direct. `SessionOptions.mode`, however, still defaults to Agent; for `RealtimeSession` this mainly matters for the Dictation/no-model distinction.

| Requested connect mode | Connection/health mode | Actual engine tool topology | Actual RealtimeSession mode |
| --- | --- | --- | --- |
| `dictation` | reports Dictation | Direct/control tools | Agent |
| `direct` | reports Direct | Direct/control tools | Agent |
| `agent` | reports Direct degradation | Direct/control tools | Agent |
| `interface` | reports Interface | Direct/control tools | Agent |

**Confirmed conclusion:** in current production code, harness presence — not the requested session mode — is the material switch for the model-visible tool catalog.

---

## 9. Model Invocation Accounting

The following table describes the **intended mode behavior**, not the current mode-propagation bug.

| Mode / scenario | Frontend realtime response cycles | Downstream visible prompts | Notes |
| --- | ---: | ---: | --- |
| Dictation | 0 | 0 | ASR only |
| Direct, plain answer | ~1 | 0 | one realtime answer |
| Direct, one control tool | >=2 | normally 0 | initial tool call + function-output continuation |
| Agent, direct answer | ~1 | 0 | same fast path as Direct |
| Agent, delegated task | typically >=3 | >=1 | initial delegation call + receipt continuation + later result delivery; downstream internals opaque |
| Interface, plain contextual action | ~1 intended | 0 intended | end-to-end production path currently absent |

`~` and `>=` are intentional. Realtime providers can internally implement one logical response in different ways, and tool chains/retries add turns.

**Inferred:** counting `response.create`/logical realtime response cycles is the most architecture-stable definition available at the VIA boundary; counting GPU/model forward passes is not observable through the provider contract.

---

## 10. Context Reach by Stage

### Current frontend model

Currently receives:

- packaged core policy;
- packaged assistant profile;
- normalized **default** client runtime context;
- tool catalog selected from a mode reconstructed from default Agent + harness presence.

Currently not shown in the inspected production initial session construction:

- actual connect `timeZone`;
- actual connect locale/working directory;
- persisted user memory documents;
- a production `ContextPack` from `via-context`;
- surface/referent/deixis context.

### Per-turn voice path

- streaming ASR → client transcript delta;
- settled ASR → turn commit and `TurnTranscripts`;
- settled transcript may be used as delegation fallback when the model omitted an objective.

No inspected code shows streaming ASR performing early capability lookup, routing, or speculative Work creation.

### Current downstream handoff

The envelope type can carry rich state, but production `DelegationRunner` currently populates a narrow subset. Therefore the downstream agent principally receives the frontend-authored objective plus identifiers rather than the full context surface the type permits.

---

## 11. State Ownership Does Not Actually Follow the Four Modes Cleanly

At the type/documentation level, modes are described as layer-mounting boundaries. At runtime, several state owners are global or connection-independent:

- `WorkManager` is composed once per Gateway, not once per Agent-mode connection.
- every connection subscribes to Work events;
- every production `RealtimeEngine` starts a Work-plane pump;
- control tools can inspect/cancel Work created by another interaction on the same owner/session;
- the Coordinator is one per Gateway and its backend coordinator session is fixed per owner/backend, not per voice mode;
- result claims/notifications survive connection changes.

This is useful architecture separation for durable work, but it also means “mode” is not equivalent to “state subsystem exists or does not exist.” A better description of the intended implementation is that mode should control **which operations are reachable from the current interaction**, while durable state services remain process-wide.

**Inferred:** this reachability interpretation is more consistent with the actual service composition than literal physical mounting/unmounting of crates or service instances.

---

## 12. Latency Optimizations Observed in Code

### Dictation

**Confirmed:** remove LLM and TTS completely; VAD + ASR only.

### Direct

**Confirmed:** no normal Work creation or downstream round trip for direct answers. Architecture documentation explicitly calls this the fast path.

### Agent

**Confirmed:** preserve Direct fast path for simple answers and make delegation receipt-based/asynchronous for long work. Backend completion is decoupled from the conversational turn.

### Interface

**Inferred:** intended to avoid downstream handoff while supplying richer local interaction context, trading local context processing for lower action latency and better referent accuracy. Production evidence is insufficient to measure this because the path is not composed.

### Cross-mode optimizations

The following are not specific to one mode but materially shape latency:

- realtime engine opens lazily on first actual input/wake need rather than on the `connect` frame;
- wake-word-only operation can avoid a live realtime connection until activation;
- `submit_text` spawns model input rather than awaiting it in the socket loop;
- provider tool calls are dispatched outside the provider-event pump;
- Work runs outside the realtime response path;
- announcement delivery attempts run outside the announcement actor;
- response queues serialize provider writes while leaving socket/event pumps responsive;
- result announcement is gated by user speech/playback state rather than injected immediately.

These choices show that the implementation prioritizes **interaction-loop responsiveness and decoupled long work** more strongly than minimizing total model-call count.

---

## 13. Is Connect-Time Mode Separation an Architecture Boundary?

### Design intent: yes

The documentation is unusually explicit:

- mode is fixed for the session lifetime;
- it decides which layers are mounted;
- therefore it is a protocol type, not a runtime flag.

`RealtimeSession` also contains real mode-dependent enforcement: Dictation short-circuits every model-response operation.

This is stronger than a cosmetic configuration label. The intended design is clearly trying to use connect-time mode as an architecture/reachability boundary.

### Current implementation: no, not end to end

The production implementation currently fails the tests an architecture boundary would need to satisfy:

1. **Immutability is not enforced.** A later `connect` frame can overwrite the connection's `ModePlan`.
2. **Mode does not cross the engine boundary.** `EngineContext` contains no mode.
3. **Production engine rebuilds default Agent.** Tool topology is selected from `SessionMode::default()` plus harness presence.
4. **RealtimeSession also receives default Agent.** Dictation's lower-level no-model enforcement is bypassed.
5. **Interface dependency is absent from production composition.** `via-context` is not linked into `apps/via` execution.
6. **Control-tool reachability is porous.** Direct/Interface can schedule a downstream-backed `scheduled_task` when a Coordinator exists.
7. **Durable Work/Coordinator services are globally composed anyway.** Mode cannot be interpreted as physical component existence.

### Classification

**Confirmed conclusion:**

> The prototype's **design intent** treats connect-time mode as an architecture boundary, but the **current production wiring realizes it primarily as runtime/connection configuration and health metadata rather than as an enforced end-to-end execution boundary**.

It should therefore be used by VIA as evidence for a **candidate boundary design**, not as evidence that connect-time mode separation has already solved the architecture-boundary problem.

For VIA decision work, the useful question is not simply “should VIA have four modes?” It is:

> What invariant must a modality/mode boundary enforce, and at which layer must that invariant be carried so that model invocation, context visibility, local capability reachability, Work creation, and downstream handoff cannot diverge from the selected mode?

---

## 14. Architecture Implications for VIA

This inspection changes the value of the Rust prototype as a reference in several areas.

### Strong reference evidence

- distinct lifecycles for realtime interaction, durable Work, and downstream session;
- frontend fast path versus asynchronous delegation;
- model-visible tool gating as a capability-placement mechanism;
- WorkManager ownership independent of downstream execution;
- DownstreamAgent as the task-execution boundary;
- result delivery separated from Work completion;
- actor/task separation to keep the interaction loop responsive.

### Weaker or incomplete reference evidence

- connect-time mode as an actually enforced layer boundary;
- production Dictation topology;
- production Interface/ContextEngine topology;
- full context propagation to frontend/downstream models;
- strict “Direct has no downstream execution” semantics;
- exact model-invocation budgeting per mode.

### Particularly relevant VIA Decision Points

- **DP-03 Capability Placement Boundary:** Direct has meaningful local/control capabilities but the scheduled-task path shows why capability reachability needs a precise boundary.
- **DP-04 Downstream Agent Integration:** Agent handoff is cleanly separated once Work enters Coordinator/DownstreamAgent.
- **DP-06 Voice Runtime Composition:** RealtimeSession already supports a no-model Dictation mode, but composition must carry the mode correctly.
- **DP-08 Voice/Text Unification Boundary:** interaction transport and durable Work are separated, but mode/state propagation must not be tied only to connection metadata.
- **DP-09 Existing vs New Task Association:** control tools operate a process-wide Work plane; mode does not itself determine task association.
- **DP-10 Intent Refinement:** frontend realtime model currently authors the delegation objective; no separate refinement pipeline precedes handoff.
- **DP-11 Agent Routing:** the frontend model chooses whether to invoke `spawn_thinking`; no request-time multi-Agent router was found.
- **DP-12 Capability Contract:** the downstream seam is strong, but session-mode capability reachability is a separate contract from Agent capability declaration.

---

## 15. Further Inspection Still Needed

1. Verify whether any non-`apps/via` host/embedder constructs `EngineContext` or `SessionOptions` differently and therefore exercises Dictation/Interface correctly outside the primary binary.
2. Inspect whether a host branch not captured in the production composition registers a real `SurfaceSource` / ContextEngine and UI affordance executor.
3. Trace `schedule_reminder(type=task)` end to end under an explicitly Direct `ToolCallHandler` to confirm the downstream-backed scheduled-runner boundary leak with an integration test rather than code composition alone.
4. Inspect whether reconnect/conversation restoration later patches `AgentContext.recent_context`; the initial production engine construction leaves it unset.
5. Inspect whether memory/context refresh paths patch the live session after connection; current `update_agent_context` binding sends an empty patch.
6. Quantify logical `response.create` counts from traces for plain Direct, one-control-tool Direct, and delegated Agent scenarios to validate the invocation accounting above.

Until those checks are complete, statements about alternative embedders or host-specific Interface behavior should be treated as **Unknown**, not extrapolated from the primary `via` binary.
