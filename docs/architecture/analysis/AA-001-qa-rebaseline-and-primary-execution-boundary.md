# AA-001 — QA Rebaseline and Primary Execution Boundary

## Status

Architecture Analysis Record — Context Checkpoint 001

This record preserves the reasoning that led to introducing `DP-00 — VIA Primary Execution Boundary` and to re-formalizing the architecture evaluation Quality Attributes. It is a checkpoint of architecture analysis, not an ADR and not an approved requirements change.

`docs/requirements/requirements-v1.1.md` remains the **Approved Baseline** and is not modified by this record.

## Why this analysis was needed

The repository originally began architecture analysis with lower-level questions such as partial input processing and interaction-context representation. Those decisions remain important, but discussion exposed a more fundamental dependency: before deciding how deeply VIA refines intent, how much context it builds, which Agent it routes to, or how it represents task association, the architecture must first decide **where primary reasoning and execution authority belongs**.

The central question is:

> **VIA는 사용자 요청의 판단과 실행을 어디까지 직접 소유하고, 어디부터 Downstream Agent 또는 다른 execution runtime에 위임할 것인가?**

This question is more fundamental than the previous Decision Point ordering because it determines the component responsibility model assumed by the later DPs.

## Why the previous Interaction Context DP is not the first decision

The initial architecture work naturally emphasized screen/pointer grounding. The Approved Baseline contains strong requirements around pointer, focus, selection, multi-pointing and turn-level interaction evidence. This led to Interaction Context Representation being treated as an early structural DP.

However, context representation does not exist in isolation. The required representation depends on **who consumes that context and for what authority**:

- If VIA itself performs deep intent interpretation and capability selection, the Context Engine must supply rich semantic material to VIA's reasoning path.
- If VIA is thin and delegates substantive interpretation/execution, the Context Engine may primarily package trustworthy interaction evidence for a downstream execution owner.
- If ARGO is the primary reasoning/execution runtime, context crosses the VIA→ARGO boundary early and must support ARGO's first substantive reasoning turn.
- If execution topology is selected per turn, the Context Engine needs a canonical representation usable by multiple candidate owners without reconstructing or silently changing meaning.

Therefore the execution-ownership decision constrains the correct answer for context representation, not vice versa.

## Approved Baseline v1.1 is the starting point, not the answer constraint

The Approved Baseline establishes an explicit responsibility split:

1. VIA owns voice/text interaction, context connection, intent/refinement, routing/delegation, conversation/task lifecycle, progress/cancel/follow-up/result interaction, policy/consent, memory, model/provider abstraction and observability.
2. Downstream Agents own domain reasoning, planning, tool selection, tool invocation, computer-use/external API integration and domain workflow execution.
3. UC-01 also permits an allowed local fast path for bounded requests that do not require external data/action.

These statements are important baseline evidence. They define the architecture being challenged or extended at the start of DP-00.

But DP-00 was introduced precisely because we need to verify whether that boundary is optimal for the **Integrated Product**. If every alternative were first rewritten to conform to v1.1, DP-00 would no longer test the responsibility boundary; it would only test implementation variants inside an already-decided boundary.

Accordingly:

- `requirements-v1.1.md` remains immutable throughout DP-00 analysis.
- v1.1 is treated as the Approved Baseline at experiment start.
- Alternatives may be compatible with, extend, or explicitly challenge that boundary.
- A boundary challenge is not automatically disqualified.
- Only if a boundary-challenging alternative wins should `requirements-vNext.md` evaluate the corresponding scope/responsibility change.
- A reviewed vNext change could later become a v1.2 approval candidate; v1.1 itself is never rewritten retroactively.

## Why Alternative B must not be weakened to fit v1.1

During refinement there is a tempting way to make the ARGO alternative look baseline-compatible:

```text
VIA owns routing
    |
    +--> choose ARGO as preferred/default Downstream Agent
```

That can be a useful product policy, but it is **not structurally different enough to be the top-level Alternative B for DP-00**.

If VIA still owns the execution-boundary decision and simply ranks ARGO first among eligible Downstream Agents, the architecture remains essentially Thin VIA with an ARGO-biased routing policy. That belongs as:

- a routing-policy variant under Alternative A, or
- a selector/ranking variant under Alternative D.

Using that weaker form as B would collapse the key distinction between alternatives and bias the analysis toward the current responsibility boundary.

Therefore Alternative B remains deliberately strong:

> **ARGO-centric Primary Execution** — `Voice/S2S → thin realtime Context → ARGO`, where ARGO is the primary ReAct reasoning/tool-execution runtime and delegates to other Agents when needed.

This form may conflict with v1.1. That conflict is intentional and is labeled explicitly as a **v1.1 Responsibility Boundary Challenging Alternative**.

The purpose is not to advocate B. The purpose is to preserve a genuinely different topology so the architecture experiment can answer whether the current boundary is actually worth its latency, complexity, replaceability and lifecycle benefits.

## Integrated Product is the comparison boundary

DP-00 does not compare VIA components in isolation.

The comparison boundary is the **Integrated Product** experienced by the user. A/B/C/D must all satisfy the same user-visible use cases, request scenarios and success predicates.

The independent variable is:

```text
SW placement / ownership of reasoning and execution capability
```

Examples of what changes across alternatives:

- who performs the first substantive reasoning turn;
- who owns planning;
- where tool execution occurs;
- who selects/delegates to another Agent;
- where task/workflow ownership sits;
- how context crosses boundaries;
- how many model/runtime transitions are needed.

Examples of what should not change just to favor an alternative:

- the user request;
- the expected useful outcome;
- benchmark scenario difficulty;
- controlled Agent/tool/service behavior;
- machine/environment conditions.

This prevents a Thin VIA prototype from being penalized for solving a broader problem, or an ARGO-centric prototype from being credited for a reduced product scope.

## Why this is the highest-level Decision Point

The Primary Execution Boundary affects all of the following.

### VIA's reason for existence and responsibility boundary

If VIA owns only modality adaptation and context packaging, it may become a multimodal shell. If it owns too much reasoning and execution, it may duplicate Agent runtimes. DP-00 determines the durable role of VIA rather than assuming it.

### ARGO and VIA relationship

ARGO may be:

- a peer Downstream Agent selected by VIA;
- a preferred/default Downstream Agent under a Thin-VIA routing policy;
- one path selected adaptively per turn;
- or the primary reasoning/execution runtime through which other Agents are reached.

These are not equivalent architectures.

### Intent Refiner depth

A Thin VIA needs enough intent structure to route and manage lifecycle safely, but not necessarily a domain plan. A richer VIA Fast Path needs enough semantics to determine bounded execution safely. An ARGO-centric topology can move much of substantive refinement/planning into ARGO.

### Agent Router need and placement

Routing can happen:

- in VIA before any substantive Agent reasoning;
- inside ARGO after an initial reasoning turn;
- or adaptively between a small number of directed execution paths.

This directly changes later routing DPs.

### Task/Workflow Manager ownership

The architecture must decide whether Work/Task is always a VIA-owned lifecycle object, whether ARGO can own the dominant execution state, and how ownership is transferred when one runtime delegates to another.

### Context Engine scope

The context representation required for a VIA-local decision differs from one consumed first by ARGO. Adaptive execution additionally requires context that can move between candidate paths without ambiguous re-interpretation.

### Agent Harness role

A Thin VIA makes the Harness the principal substantive execution seam. ARGO-centric execution strategically privileges the ARGO boundary. Hybrid/adaptive execution requires explicit local-vs-external capability ownership and escalation semantics.

### Model invocation count and latency

The execution boundary determines whether a fast task requires:

- a local VIA model/tool path;
- a VIA routing/model step plus an Agent turn;
- an ARGO primary reasoning/tool turn;
- an ARGO turn plus further specialized delegation;
- or a selector followed by one of several owners.

This is why QA rebaseline begins with user-visible end-to-end responsiveness rather than only VIA-internal software overhead.

### Agent ecosystem extensibility

A peer-Agent architecture favors independent replacement. ARGO-centric execution can simplify the dominant first-party path while making other Agents structurally subordinate to ARGO's delegation model. The trade-off must be measured rather than normalized away before evaluation.

## Candidate execution topologies

No winner is selected in this checkpoint.

### Alternative A — Thin VIA

```text
User / Voice / Text
        |
        v
VIA Interaction + Context + Intent/Task + Routing
        |
        v
Downstream Agent
        |
        v
Domain reasoning / planning / tools / execution
```

VIA owns interaction, grounding, request normalization, routing, lifecycle and mediation. Substantive execution crosses the Downstream Agent boundary.

**v1.1 Boundary Compatibility:** **Compatible**.

### Alternative B — ARGO-centric Primary Execution

```text
Voice / S2S
    |
    v
thin realtime Context / interaction layer
    |
    v
ARGO primary ReAct reasoning + tool execution
    |
    +--> Specialized / other Agent delegation when required
```

ARGO owns the first substantive reasoning/planning/tool-execution path. Other Agents are reached from that primary runtime when necessary.

**v1.1 Boundary Compatibility:** **Challenges baseline boundary**.

This is intentionally a boundary-challenging alternative and must not be reduced to “VIA prefers ARGO as a Downstream Agent.”

### Alternative C — Hybrid VIA Fast Path

```text
                         +--> bounded VIA Fast Path
User -> VIA eligibility -+
                         +--> Downstream Agent
```

VIA directly executes a narrowly governed class of bounded, local, latency-critical capabilities and delegates substantive work.

Fast-path eligibility is semantic—bounded execution, no domain planning, no durable workflow, no external Agent state, simple recovery, local-safe ownership and measurable latency benefit—not a rule such as “<3 seconds” or “1 LLM + 1 tool.”

**v1.1 Boundary Compatibility:** **Mostly compatible / extension**.

### Alternative D — Adaptive Per-turn Execution

```text
                       +--> VIA Fast Path
User -> VIA selector --+--> ARGO path
                       +--> Specialized Downstream Agent
```

Execution topology is selected per turn. Directed escalation may be allowed, but arbitrary ownership bouncing should be avoided because it makes task identity, context provenance, side-effect deduplication, cancellation, retry, progress and recovery difficult to reason about.

**v1.1 Boundary Compatibility:** **Partially compatible / extension likely**.

A D variant can remain close to v1.1 if VIA selects peer Agents directly; a D variant that gives ARGO primary reasoning authority on some turns may require a larger vNext boundary change.

## Boundary Compatibility Summary

| Alternative | v1.1 Boundary Compatibility | Meaning |
| --- | --- | --- |
| A — Thin VIA | **Compatible** | Preserves the current responsibility split. |
| B — ARGO-centric Primary Execution | **Challenges baseline boundary** | Deliberately tests moving primary reasoning/execution authority into ARGO. |
| C — Hybrid Fast Path | **Mostly compatible / extension** | Preserves downstream authority for substantive work while extending bounded VIA execution. |
| D — Adaptive Per-turn | **Partially compatible / extension likely** | Compatibility depends on which per-turn owners/transfer rules are selected. |

Compatibility is an analysis dimension, not a veto criterion. Final selection still depends on measured QA trade-offs and architectural consequences.

## Existing Rust Reference Implementation as input

`docs/references/internal-rust-voice-agent/` records an Existing Reference Implementation, not an approved VIA architecture.

Relevant observed choices include:

- realtime/voice and downstream execution separated by Work/Coordinator layers;
- common `DownstreamAgent` / `HarnessSession` contract;
- conversation-turn lifecycle separated from asynchronous Work lifecycle;
- global/per-owner/keyed-lane admission;
- `ContextPack`, stable referents and `SurfaceSnapshot` generations;
- static capability declaration separated from dynamic health;
- configured backend-id routing rather than semantic multi-Agent routing;
- `dictation`, `direct`, `agent`, `interface` connect-time modes.

The deeper inspection in `docs/references/internal-rust-voice-agent/execution-topology.md` found that the prototype's **design intent** treats session mode as a layer-mounting boundary, while current production wiring does not enforce the connect-selected mode end-to-end. That discrepancy reinforces the need to evaluate topology directly rather than copy the prototype.

The reference is most useful for demonstrating possible boundaries—Work ownership, downstream seams, result injection, provider abstraction—not for deciding DP-00 by precedent.

## Why connect-time mode is not automatically the DP-00 solution

The Rust reference's four modes attempt to make execution ownership explicit. DP-00, however, concerns the architecture of request ownership, not merely a connection configuration mechanism.

A connect-time mode is appropriate only if a session should genuinely commit to one execution topology for its lifetime. If one conversation mixes direct interaction, bounded local action, ARGO-suitable work and specialized Agent work, per-turn selection may fit the Integrated Product better.

That remains a hypothesis to evaluate against:

- state ownership;
- model/tool surface;
- context reuse;
- task continuity;
- latency;
- predictability;
- recovery complexity.

## Evaluation consequences

DP-00 cannot be evaluated by functional demos alone. Several alternatives can produce the same useful outcome while differing materially in:

- latency;
- semantic correctness;
- task/lifecycle correctness;
- fault isolation;
- extensibility;
- resource use;
- model invocation count;
- coupling;
- degree of v1.1 responsibility-boundary change.

The experiment must isolate **architecture effects** from stochastic model/Agent behavior. The independent variable is the architecture alternative / execution-capability placement, while stochastic reasoning, network latency and Agent implementation variance are controlled during architecture qualification.

## QA-01 consequence

The first re-formalized QA is Fast-task End-to-End Responsiveness. It measures from **ground-truth acoustic end-of-speech** to the first **useful observable outcome**, not to an acknowledgment and not merely to an internal routing milestone.

This is appropriate because DP-00 alternatives directly alter the number and placement of architecture/runtime hops on tasks that could otherwise finish quickly.

Long-running task completion is intentionally not the primary metric of QA-01 because external work duration can dominate the architecture effect.

## Reasoning checkpoint: what is decided and what is not

### Decided for the architecture-analysis process

- Introduce DP-00 as the prerequisite execution-boundary question.
- Preserve Approved Baseline v1.1 unchanged.
- Treat v1.1 as the starting baseline, while allowing DP-00 to challenge the boundary itself.
- Compare A/B/C/D at the Integrated Product boundary with the same user-visible scenarios.
- Keep Alternative B as the structurally strong ARGO-centric Primary Execution topology.
- Label B explicitly as a v1.1 Responsibility Boundary Challenging Alternative.
- Treat “ARGO as preferred/default Downstream Agent” as an A/D routing-policy variant, not as top-level B.
- Do not renumber/replace the existing QA/DP index yet.
- Define QA metrics independently before scoring A/B/C/D.
- Use one Primary Metric per QA for the 0–5 architecture score.
- Preserve sufficient raw observations so later metrics can be recomputed.
- Use deterministic semantic replay / controlled dependencies for architecture qualification.
- Keep actual-model validation separate as fidelity/external-validity evidence.

### Not decided

- Winner among DP-00 A/B/C/D.
- Whether the final architecture preserves or changes the v1.1 responsibility boundary.
- Whether ARGO is a peer, preferred route, per-turn owner, or primary runtime in the selected architecture.
- Exact VIA Fast Path capability allow-list.
- Whether per-turn topology is preferable to session-level topology after measurement.
- QA-01 score thresholds.
- Final QA numbering/rebaseline in `requirements-vNext.md` and `qa-dp-traceability.md`.

## Requirement-change rule after DP-00

No responsibility-boundary change is written into the Approved Baseline during evaluation.

If the selected DP-00 alternative requires a different boundary—most clearly Alternative B—the sequence is:

```text
DP-00 measured decision
        ↓
requirements-vNext.md responsibility/scope proposal
        ↓
review
        ↓
possible v1.2 Approved Baseline candidate
```

This preserves v1.1 as the historical Approved Baseline against which the boundary-changing decision was evaluated.

## Follow-on dependencies

DP-00 is an input to later decisions concerning:

- capability placement;
- generic vs specialized Agent integration;
- Agent routing;
- intent refinement;
- existing-task association;
- voice runtime composition;
- context representation and context egress;
- model invocation policy;
- task/workflow ownership.

Those DPs remain valid questions, but their alternatives and evaluation interpretation may need adjustment once DP-00 is decided.

## Repository update policy for this checkpoint

This checkpoint intentionally does not modify:

- `docs/requirements/requirements-v1.1.md`;
- the existing QA-01~QA-11 definitions/numbering in the Approved Baseline;
- `docs/architecture/qa-dp-traceability.md`;
- `docs/evaluation/evaluation-strategy.md`.

After QA-01 through QA-04 are formally defined, the working requirements, QA↔DP traceability and evaluation strategy should be coherently rebaselined together rather than incrementally creating inconsistent numbering.