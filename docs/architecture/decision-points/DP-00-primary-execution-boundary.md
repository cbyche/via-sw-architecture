# DP-00 — VIA Primary Execution Boundary

## Status

**Alternatives Defined — Evaluation QAs being formalized**

This is a proposed top-level Decision Point for the vNext architecture analysis. It does not modify or override `docs/requirements/requirements-v1.1.md`.

## Problem

### 쉬운 질문

> **VIA는 사용자 요청을 어디까지 직접 판단·실행하고, 어디부터 Downstream Agent에 위임할 것인가?**

The Approved Baseline gives VIA responsibility for interaction, context, intent refinement, routing/delegation and task lifecycle while assigning domain reasoning/planning/tool execution to Downstream Agents. It also permits a local fast path for bounded requests.

The unresolved architecture problem is the exact **primary execution boundary** between those responsibilities.

## Why this is architectural

This decision changes system topology and ownership rather than one implementation detail. It determines:

- whether VIA is primarily an interaction/orchestration layer or a substantive execution runtime;
- whether ARGO is a preferred Agent, a primary reasoning runtime, or a peer Agent;
- how deep VIA's Intent Refiner must be;
- where Agent routing occurs;
- which operations become VIA-owned Work/Task;
- when durable workflow semantics are required;
- how much Context Engine material must remain local vs cross an Agent boundary;
- how many model/Agent invocations occur on common paths;
- what cancellation/retry/progress state VIA must own;
- how easily new Agents can become peers;
- latency on fast, bounded tasks.

Because these consequences span components, state ownership, trusted execution boundaries, runtime topology and multiple Quality Attributes, the decision must precede lower-level choices such as capability placement, routing and intent refinement.

See `docs/architecture/analysis/AA-001-qa-rebaseline-and-primary-execution-boundary.md` for the reasoning checkpoint.

## Relationship to the Approved Baseline

This DP must be evaluated without silently changing v1.1.

Relevant baseline principles include:

- VIA owns Voice/Text interaction, context, intent refinement, Agent routing/delegation, conversation/task lifecycle, consent/policy and result interaction.
- Downstream Agents own domain reasoning, planning, tool selection/tool execution and domain workflow completion.
- UC-01 permits allowed direct/local response when no external data/action is required.
- VIA does not centrally reimplement or approve each Downstream Agent tool call.

The alternatives below explore the practical topology consistent—or potentially in tension—with those principles. Any accepted choice that requires changing requirements must first be recorded in `requirements-vNext.md` and reviewed before a new approved baseline.

## Related requirements

Indicative existing requirements/areas affected:

- UC-01 Real-time Voice Interaction & Local Response
- UC-04 Downstream Agent Delegation
- UC-05 PC/Application/Service Action Request
- UC-07 Stateful / Long-running Task
- UC-10 Result / Progress Delivery
- UC-11 Compound Utterance Handling
- UC-12 Underspecified / Contextual Request Resolution
- UC-13 Task Continuation & Follow-up
- UC-16 Concurrent Task Handling
- FR-06~FR-10 Intent refinement family
- FR-11 Agent selection/delegation
- FR-27 task association/follow-up area
- FR-35 Agent/capability registry area
- FR-37 mixed interaction/task continuity area
- FR-40~FR-43 task/concurrency/Agent integration area
- FR-44 model/voice runtime abstraction area

Exact traceability will be coherently rebaselined after QA-01~QA-04 definitions are complete.

## Alternatives

### Alternative A — Thin VIA

**Structure**

```text
Voice / Text / Interaction Evidence
               |
               v
+------------------------------------+
| VIA                                |
| interaction + context + intent     |
| routing + task lifecycle + policy  |
+------------------------------------+
               |
               v
        Downstream Agent
               |
               v
 reasoning + planning + tools + execution
```

**Execution authority**

VIA determines sufficient semantics to ground, route and safely manage the request, but substantive domain execution is handed to a Downstream Agent.

**VIA owns**

- modality/session interaction;
- context collection and provenance;
- request normalization/refinement required for routing;
- Agent selection;
- task identity/lifecycle;
- progress/cancel/follow-up mediation;
- context egress/consent policy;
- result delivery.

**Downstream Agent owns**

- domain reasoning;
- plan formation;
- tool selection and execution;
- domain workflow completion.

**Pros**

- Clean Agent-neutral boundary.
- Strong alignment with the Approved Baseline's trusted Agent execution boundary.
- Easier Agent replacement and peer ecosystem growth.
- Low risk of VIA accumulating duplicate domain/tool logic.
- Clear security/audit boundary for downstream execution.

**Cons**

- Short bounded operations still pay request serialization, routing, Agent dispatch/startup and result-return overhead.
- May create Work/Agent ceremony for tasks that do not need domain reasoning.
- User-perceived latency can be dominated by the boundary rather than the operation.

### Alternative B — ARGO-first

**Structure**

```text
Voice / Text
     |
     v
Thin VIA realtime/context shell
     |
     v
ARGO primary reasoning authority
     |
     +------> Specialized Agent when required
```

**Execution authority**

ARGO receives the user's substantive request first and acts as the primary ReAct reasoning/execution authority. It delegates to another Agent when needed.

**Pros**

- Minimal extra orchestration for work ARGO already handles.
- Can reuse existing ARGO reasoning/tool capability rather than rebuilding planning in VIA.
- Potentially reduces duplicate intent/planning stages.
- May offer a very short preferred first-party path.

**Cons**

- VIA risks becoming primarily a multimodal shell rather than a durable Agent-neutral orchestration layer.
- Strong coupling to ARGO request semantics, lifecycle and capability model.
- Other Agents may effectively become ARGO sub-agents rather than peer Downstream Agents.
- A request best handled by another Agent may first incur an ARGO reasoning/dispatch turn.
- ARGO replacement may become an architecture rewrite rather than an adapter change.

**Important note**

Selecting this alternative would require especially careful consistency review against the Approved Baseline statement that VIA selects the appropriate Downstream Agent and that Downstream Agent internals are outside VIA.

### Alternative C — Hybrid VIA Fast Path

**Structure**

```text
                         +----------------------+
                         | bounded VIA Fast Path|
                         +----------------------+
                        /
User -> VIA eligibility
                        \
                         +----------------------+
                         | Downstream Agent     |
                         +----------------------+
```

**Execution authority**

VIA directly owns a deliberately small class of bounded, local, latency-critical capability. Everything else crosses the Downstream Agent boundary.

**Fast Path eligibility semantics**

Eligibility is defined architecturally, not by post-hoc timing or arbitrary invocation-count thresholds. A candidate Fast Path should generally have all of the following properties:

- bounded execution;
- no domain planning required;
- no durable workflow required;
- no external Agent state/thread required;
- simple, locally understandable failure/recovery semantics;
- capability is local-safe and appropriate for VIA ownership;
- a direct path provides measurable latency/user-experience benefit.

“Less than 3 seconds” or “1 LLM + 1 tool” may be useful observed characteristics, but are **not** the definition.

**Pros**

- Preserves a clear downstream boundary for substantive work.
- Avoids Agent dispatch overhead for semantically bounded local operations.
- Makes latency optimization explicit and measurable.
- Can keep local interaction controls close to the realtime loop.

**Cons**

- Requires a stable capability-placement policy.
- Local allow-list can grow until VIA duplicates an Agent runtime unless actively governed.
- Escalation from Fast Path to Agent must preserve task/context/side-effect semantics.
- Fast-path eligibility itself may require semantic reasoning whose cost offsets the benefit.

### Alternative D — Adaptive Per-turn Execution

**Structure**

```text
                         +--> VIA Fast Path
                        /
User turn -> VIA selector ---> ARGO preferred Agent
                        \
                         +--> Specialized Downstream Agent
```

**Execution authority**

Every user turn is classified into one of a small number of execution topologies. The choice is per turn rather than fixed for an entire connection/session.

Possible execution owners:

- VIA Fast Path;
- ARGO as a first-party preferred Downstream Agent;
- a specialized Downstream Agent selected directly.

**Directed escalation principle**

Adaptive selection must not become unrestricted ownership bouncing.

A directed escalation may be valid:

```text
Fast Path -> ARGO -> Specialized Agent
```

An arbitrary cycle should be avoided:

```text
VIA -> ARGO -> VIA -> Agent B -> ARGO
```

Ownership bouncing makes side-effect deduplication, task identity, context provenance, cancellation, retry, progress and recovery much harder to reason about.

**Pros**

- Can choose the lowest-overhead appropriate path per request.
- Avoids forcing a whole session into a mode that does not match every turn.
- ARGO can be preferred without making all Agents structurally subordinate to it.
- Specialized requests can route directly when confidence/capability evidence is sufficient.

**Cons**

- Execution selection becomes a critical semantic component.
- Harder lifecycle and observability model than A or B.
- Requires explicit ownership transfer/escalation contracts.
- Incorrect path selection can affect both correctness and latency.
- May introduce extra routing-model invocation unless carefully designed.

## ARGO positioning under evaluation

A current architectural direction worth testing is:

> **ARGO remains outside VIA core as a first-party preferred Downstream Agent, not an internal VIA runtime.**

This is not a decision. It is an evaluation hypothesis because it may preserve peer-Agent architecture while retaining a high-quality first-party path.

The alternatives must measure whether the extra VIA routing boundary is worth the resulting decoupling.

## Existing Rust Reference Implementation

The internal Rust prototype is an **Existing Reference Implementation**, not the selected architecture.

Relevant reference observations include:

- realtime/voice vs Work/Coordinator vs downstream-harness separation;
- common `DownstreamAgent` / `HarnessSession` seam;
- asynchronous Work lifecycle separate from conversation turns;
- bounded local frontend/control tools plus `spawn_thinking` delegation;
- static capability + dynamic health;
- connect-time `dictation/direct/agent/interface` mode concept.

The deeper execution-path inspection in `docs/references/internal-rust-voice-agent/execution-topology.md` found an important discrepancy: the design documents/types describe connect-time mode as a layer-mounting architecture boundary, but the current production composition does not consistently propagate the selected mode into the realtime engine/session. This evidence must therefore be used to compare concepts, not copied as a baseline.

## Connect-time mode vs per-turn execution

The Rust reference suggests one way to express execution boundary: choose a mode once at connect time.

DP-00 intentionally does **not** assume this is the right mechanism.

A session-level mode is architecture-significant only if fixing an execution topology for the full session is itself a required invariant. If users naturally mix direct conversation, bounded local action, general Agent work and specialized Agent work in one conversation, per-turn selection may fit the problem better.

This must be measured and reasoned about in terms of:

- state ownership;
- model/tool surface;
- context reuse;
- task continuity;
- latency;
- predictability;
- recovery complexity.

## Expected QA trade-offs

QA numbering is undergoing vNext formalization. Existing Approved Baseline QA IDs remain unchanged until a coherent rebaseline is prepared.

| Quality concern | A Thin VIA | B ARGO-first | C Hybrid Fast Path | D Adaptive Per-turn |
| --- | --- | --- | --- | --- |
| Fast-task E2E responsiveness | Risk of boundary overhead | Strong for ARGO-suitable tasks; weaker if reroute needed | Potentially strongest for allow-listed fast tasks | Potentially strong if selector overhead is low |
| Execution correctness | Clear single downstream authority | Depends strongly on ARGO first-hop judgment | Must correctly separate local vs delegated work | Adds path-selection correctness as a new failure mode |
| Task/lifecycle correctness | Strong central VIA ownership | Needs clear ARGO/VIA task boundary | Central ownership plus local exceptions | Hardest ownership-transfer model |
| Agent replaceability | Strong | Weakest if ARGO privileged structurally | Strong for downstream side | Strong if selector/contract is Agent-neutral |
| Architecture complexity | Low/moderate | Low VIA complexity, high strategic coupling | Moderate | Highest |
| Model invocation efficiency | Can require VIA + Agent hops | Efficient on ARGO path, possible extra hop elsewhere | Efficient for fast path | Depends on selector implementation |
| Failure isolation | Strong boundary | ARGO becomes broad blast-radius dependency | Good if Fast Path remains narrow | Requires explicit per-path isolation |

This table is qualitative only. It is not a decision score.

## Prototype plan

Create functionally equivalent prototype topologies for A/B/C/D with a common request/task/result contract.

At minimum, prototypes should expose raw events sufficient to observe:

- acoustic end of speech;
- semantic processing start/result;
- execution-path selection;
- execution owner;
- Agent dispatch;
- tool/action start/completion;
- first user-visible meaningful result;
- useful outcome;
- task completion;
- success/failure;
- model invocation count.

Use deterministic semantic traces and Agent/tool doubles for architecture qualification.

## Benchmark plan

### Dataset

Include at least:

- F1 Local / Direct-capable fast tasks;
- F2 short general-Agent tasks;
- F3 short specialized-Agent delegation tasks;
- later QA corpora for correctness, lifecycle/recovery and evolution.

### Primary architecture experiment rule

```text
Independent variable = SW Architecture Alternative (A/B/C/D)
```

Model stochasticity, Agent implementation variance, network variance and uncontrolled machine state must not become hidden independent variables during architecture qualification.

### QA-01

Use `Fast-task Outcome Latency p95 (FTOL p95)` as defined in:

`docs/evaluation/quality-attributes/QA-01-fast-task-e2e-responsiveness.md`

### Scoring

Do not set or tune 0–5 thresholds after seeing the final comparative result. Follow:

```text
Pilot -> threshold calibration -> scoring version freeze -> final evaluation
```

## Results

TBD.

## Decision

TBD. No alternative is selected in this checkpoint.

## Consequences

TBD after measured trade-off analysis.

## Requirement changes discovered

None committed to vNext by this checkpoint. Potential implications will be accumulated and coherently reviewed after QA-01 through QA-04 are defined.