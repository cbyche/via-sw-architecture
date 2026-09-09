# DP-00 — VIA Primary Execution Boundary

## Status

**Alternatives Defined — Evaluation QAs being formalized**

This is a proposed top-level Decision Point for the vNext architecture analysis. It does not modify or override `docs/requirements/requirements-v1.1.md`.

## Problem

### 쉬운 질문

> **VIA는 사용자 요청을 어디까지 직접 판단·실행하고, 어디부터 Downstream Agent에 위임할 것인가?**

The Approved Baseline v1.1 is the starting point for this analysis. It gives VIA responsibility for interaction, context, intent refinement, routing/delegation and task lifecycle while assigning domain reasoning/planning/tool execution to Downstream Agents. It also permits a local fast path for bounded requests.

DP-00 exists because the architecture must test whether that responsibility boundary itself is optimal for the Integrated Product, rather than constraining all alternatives to fit it in advance.

The unresolved architecture problem is therefore the **primary reasoning/execution boundary**: where substantive judgment, planning, tool execution and delegation authority should live for the product as a whole.

## Why this is architectural

This decision changes system topology and ownership rather than one implementation detail. It determines:

- whether VIA is primarily an interaction/orchestration layer or a substantive execution runtime;
- whether ARGO is a downstream peer, a preferred route, or the primary reasoning/execution authority;
- how deep VIA's Intent Refiner must be;
- where Agent routing occurs;
- which operations become VIA-owned Work/Task;
- when durable workflow semantics are required;
- how much Context Engine material remains inside VIA vs crosses an execution boundary;
- how many model/Agent invocations occur on common paths;
- what cancellation/retry/progress state VIA must own;
- how easily new Agents can become peers;
- latency on fast, bounded tasks;
- whether the v1.1 responsibility boundary should be preserved, extended, or changed in vNext.

Because these consequences span component boundaries, trusted execution boundaries, runtime topology, state ownership and multiple Quality Attributes, this decision precedes lower-level choices such as capability placement, routing and intent refinement.

See `docs/architecture/analysis/AA-001-qa-rebaseline-and-primary-execution-boundary.md` for the reasoning checkpoint.

## Evaluation Boundary — Integrated Product

DP-00 compares **Integrated Product execution topologies**, not isolated VIA component implementations.

All alternatives A/B/C/D must satisfy the same user-visible use cases and the same benchmark scenarios. The experiment must not make one alternative solve an easier product problem than another.

The intended controlled comparison is:

```text
Independent variable
    = SW placement / ownership of reasoning and execution capability

Held functionally equivalent
    = user-visible use case
    = scenario input
    = expected useful outcome
    = benchmark dependency behavior
    = machine/environment controls
```

Accordingly, the comparison asks where capabilities are placed and who owns execution authority, while evaluating the same Integrated Product outcome.

This matters especially for Alternative B: ARGO-centric execution is not rejected merely because it moves responsibility across the current VIA boundary. Challenging that boundary is the point of including B.

## Relationship to Approved Baseline v1.1

`docs/requirements/requirements-v1.1.md` remains immutable and is the **Approved Baseline at DP-00 start**.

Relevant v1.1 principles include:

- VIA owns Voice/Text interaction, context, intent refinement, Agent routing/delegation, conversation/task lifecycle, consent/policy and result interaction.
- Downstream Agents own domain reasoning, planning, tool selection/tool execution and domain workflow completion.
- UC-01 permits allowed direct/local response when no external data/action is required.
- VIA does not centrally reimplement or approve each Downstream Agent tool call.

These principles are baseline evidence and constraints at the start of analysis. They are **not used to weaken a structurally distinct DP-00 alternative until all alternatives become baseline-compatible by construction**.

If the final decision preserves the baseline boundary, no responsibility change is required. If a boundary-challenging alternative such as B is selected, only then should `requirements-vNext.md` evaluate the corresponding responsibility/scope change, followed by review as a possible v1.2 baseline candidate.

The Approved Baseline itself is not edited during DP-00 evaluation.

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

**v1.1 Boundary Compatibility**

**Compatible.** This is the strongest expression of the responsibility boundary already described by v1.1.

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

### Alternative B — ARGO-centric Primary Execution

**Classification**

> **v1.1 Responsibility Boundary Challenging Alternative**

**Structure**

```text
Voice / S2S
    |
    v
thin realtime Context / interaction layer
    |
    v
ARGO — primary ReAct reasoning + tool execution runtime
    |
    +------> Specialized / other Agent delegation when required
```

**Execution authority**

ARGO is the primary substantive reasoning and execution runtime. The front layer supplies voice/realtime interaction and enough context to ground the request, then ARGO performs the first major reasoning step, planning and tool execution. When the request requires another execution specialist, ARGO delegates onward.

This is intentionally stronger than “VIA routes to ARGO first.” The structural distinction is that primary reasoning/execution authority moves into the ARGO-centric path rather than remaining in a VIA-owned Agent-neutral orchestration boundary.

**v1.1 Boundary Compatibility**

**Challenges baseline boundary.** v1.1 assigns routing/delegation and lifecycle authority to VIA while placing domain reasoning/planning/tool execution behind the Downstream Agent boundary. B explicitly tests a topology where ARGO becomes the primary reasoning/execution authority through which other Agents may be reached.

That conflict is not an error in the alternative definition. It is a deliberate hypothesis to evaluate.

**Pros**

- Shortest structural path for work ARGO can already reason about and execute.
- Reuses a mature ReAct/tool runtime rather than duplicating equivalent planning capability in VIA.
- Can reduce duplicate VIA intent/planning layers if ARGO already provides the required semantic authority.
- May reduce model/orchestration hops for the dominant first-party task set.

**Cons**

- VIA may become primarily a multimodal/realtime context shell rather than a durable Agent-neutral orchestration layer.
- Strong strategic coupling to ARGO request semantics, lifecycle, tools and failure model.
- Other Agents may become ARGO-mediated sub-agents rather than peer execution authorities.
- Non-ARGO tasks may incur an ARGO reasoning/delegation hop before reaching the best specialist.
- ARGO replacement can become a topology change rather than an adapter replacement.
- Task ownership, permission mediation, recovery and audit boundaries may have to move or be redefined.

**Requirement consequence if selected**

No requirement file is changed during evaluation. If B wins the measured trade-off, `requirements-vNext.md` must explicitly evaluate changes to the VIA/Downstream Agent responsibility and scope boundary. Only after review would those changes become candidates for a future approved v1.2 baseline.

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

**v1.1 Boundary Compatibility**

**Mostly compatible / extension.** v1.1 already permits a local fast path, but C may broaden or formalize the set of VIA-owned bounded actions and therefore may require vNext clarification depending on the chosen capability set.

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
User turn -> VIA selector ---> ARGO path
                        \
                         +--> Specialized Downstream Agent
```

**Execution authority**

Every user turn is classified into one of a small number of execution topologies. The choice is per turn rather than fixed for an entire connection/session.

Candidate execution owners may include:

- VIA Fast Path;
- ARGO path;
- a specialized Downstream Agent selected directly.

The exact authority assigned to the ARGO path is itself a design variable inside D. D's defining property is per-turn topology selection and explicit ownership transfer, not a requirement that ARGO always be first.

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

**v1.1 Boundary Compatibility**

**Partially compatible / extension likely.** Direct selection of peer Downstream Agents and a bounded VIA Fast Path can fit v1.1. A D variant that grants ARGO primary reasoning authority for some turns may challenge parts of the current boundary and would need explicit vNext treatment if selected.

**Pros**

- Can choose the lowest-overhead appropriate path per request.
- Avoids forcing a whole session into a mode that does not match every turn.
- Can route specialized requests directly when confidence/capability evidence is sufficient.
- Makes topology choice explicit and measurable per episode.

**Cons**

- Execution selection becomes a critical semantic component.
- Harder lifecycle and observability model than A or B.
- Requires explicit ownership transfer/escalation contracts.
- Incorrect path selection can affect both correctness and latency.
- May introduce extra routing-model invocation unless carefully designed.

## Routing-policy variants are not separate top-level alternatives

A policy such as:

> “Keep VIA's existing Thin-VIA responsibility model, but prefer ARGO as the default Downstream Agent whenever it is eligible.”

is **not Alternative B**.

Structurally, that policy preserves VIA as the component that owns routing and chooses a Downstream Agent before substantive execution. It is therefore much closer to:

- a routing policy variant inside **Alternative A**, or
- a path-selection/ranking variant inside **Alternative D**.

Promoting “preferred ARGO route” to its own top-level alternative would create alternatives that differ mainly in routing preference rather than in the responsibility boundary DP-00 is meant to test.

Alternative B is retained specifically because it changes where primary reasoning/execution authority sits.

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

| Quality concern | A Thin VIA | B ARGO-centric Primary Execution | C Hybrid Fast Path | D Adaptive Per-turn |
| --- | --- | --- | --- | --- |
| Fast-task E2E responsiveness | Risk of boundary overhead | Potentially strong for ARGO-native tasks; reroute cost for specialists | Potentially strongest for allow-listed fast tasks | Potentially strong if selector overhead is low |
| Execution correctness | Clear downstream authority | ARGO becomes primary semantic/execution authority | Must correctly separate local vs delegated work | Adds path-selection correctness as a new failure mode |
| Task/lifecycle correctness | Strong central VIA ownership | Requires redesigned VIA/ARGO ownership if baseline boundary moves | Central ownership plus local exceptions | Hardest ownership-transfer model |
| Agent replaceability | Strong | Weakest if ARGO is structurally central | Strong for downstream side | Strong only if selector/contract remains neutral |
| Architecture complexity | Low/moderate | Thin VIA, but high strategic coupling | Moderate | Highest |
| Model invocation efficiency | Can require VIA + Agent hops | Efficient for ARGO-owned path, possible delegation hop elsewhere | Efficient for fast path | Depends on selector implementation |
| Failure isolation | Strong boundary | ARGO becomes broad blast-radius dependency | Good if Fast Path remains narrow | Requires explicit per-path isolation |
| v1.1 boundary compatibility | Compatible | **Challenges baseline boundary** | Mostly compatible / extension | Partially compatible / extension likely |

This table is qualitative only. It is not a decision score.

## Prototype plan

Create functionally equivalent Integrated Product prototype topologies for A/B/C/D with a common request/task/result contract.

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

### Functional equivalence requirement

A/B/C/D are compared against the same user-visible scenarios and success predicates. The architecture alternative may change which component owns reasoning/execution, but not the expected product behavior.

### Dataset

Include at least:

- F1 Local / Direct-capable fast tasks;
- F2 short general-Agent tasks;
- F3 short specialized-Agent delegation tasks;
- later QA corpora for correctness, lifecycle/recovery and evolution.

### Primary architecture experiment rule

```text
Independent variable = SW Architecture Alternative (A/B/C/D)
                      = reasoning/execution placement and ownership
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

None committed to vNext by this checkpoint.

If a final DP-00 decision requires a responsibility/scope change—most clearly if Alternative B is selected—the proposed change is first recorded in `requirements-vNext.md`, reviewed, and only later considered for an approved v1.2 baseline. `requirements-v1.1.md` remains the preserved starting baseline.