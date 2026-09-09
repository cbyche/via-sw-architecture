# AA-001 — QA Rebaseline and Primary Execution Boundary

## Status

Architecture Analysis Record — Context Checkpoint 001

This record preserves the reasoning that led to introducing `DP-00 — VIA Primary Execution Boundary` and to re-formalizing the architecture evaluation Quality Attributes. It is a checkpoint of architecture analysis, not an ADR and not an approved requirements change.

`docs/requirements/requirements-v1.1.md` remains the **Approved Baseline** and is not modified by this record.

## Why this analysis was needed

The current repository originally begins architecture analysis with DP-01/DP-02-level questions such as partial input processing and interaction-context representation. Those are important decisions, but discussion exposed a more fundamental dependency: before deciding how deeply VIA refines intent, how much context it builds, which Agent it routes to, or how it represents task association, the architecture must first define **where substantive execution authority lives**.

The primary question is:

> **Where does VIA stop directly judging/executing a user request, and where does a Downstream Agent begin?**

A practical wording is:

> **VIA는 사용자 요청의 판단과 실행을 어디까지 직접 소유하고, 어디부터 Downstream Agent에 위임할 것인가?**

This question is more fundamental than the previous ordering of Decision Points because it changes the component responsibility model that the later DPs assume.

## Why the previous Interaction Context DP is not the first decision

The initial architecture work naturally emphasized screen/pointer grounding. The Approved Baseline contains strong requirements around pointer, focus, selection, multi-pointing, and turn-level interaction evidence. This led to Interaction Context Representation being treated as an early structural DP.

However, context representation does not exist in isolation. The required representation depends on **who consumes that context and for what authority**:

- If VIA itself performs deep intent interpretation and capability selection, the Context Engine must supply sufficiently rich semantic material to VIA's own reasoning path.
- If VIA is thin and delegates most substantive interpretation/execution, the Context Engine may primarily package trustworthy interaction evidence for downstream execution.
- If ARGO becomes the primary reasoning authority, context may need to cross the VIA→ARGO boundary early and in a form optimized for ARGO.
- If execution topology is selected per turn, the Context Engine may need a canonical representation usable by multiple execution owners without reconstruction.

Therefore the execution-ownership decision constrains the correct answer for context representation, not vice versa.

## Relation to the Approved Baseline

The Approved Baseline already establishes two important principles that must be preserved while evaluating DP-00:

1. VIA owns voice/text interaction, context connection, intent/refinement, routing/delegation, conversation/task lifecycle, progress/cancel/follow-up/result interaction, policy/consent, memory, model/provider abstraction, and observability.
2. Downstream Agents own domain reasoning, planning, tool selection, tool invocation, computer-use/external API integration, and domain workflow execution.

At the same time, UC-01 explicitly permits an allowed **local fast path** for conversational/general-knowledge/interaction-control requests that do not require external data or action.

DP-00 therefore does not automatically replace the Approved Baseline. It makes explicit the unresolved topology between these principles: **how much bounded execution may remain inside VIA before the request crosses the trusted Downstream Agent boundary?**

## Why this is the highest-level Decision Point

The Primary Execution Boundary affects all of the following.

### VIA's reason for existence and responsibility boundary

If VIA owns only modality adaptation and context packaging, it becomes a multimodal shell. If it owns too much reasoning and execution, it duplicates Downstream Agent capabilities and violates the desired agent-neutral boundary. The selected topology defines the durable role of VIA.

### ARGO and VIA relationship

ARGO may be:

- an implementation dependency inside VIA,
- a preferred first-party Downstream Agent,
- a general reasoning authority through which other Agents are reached,
- or one peer Agent selected by VIA.

These choices produce materially different coupling and replacement properties.

### Intent Refiner depth

A thin VIA needs enough intent structure to route and maintain lifecycle safely, but not necessarily a domain plan. A richer VIA fast path requires enough semantics to determine bounded local execution safely. An ARGO-first topology may collapse some VIA-side refinement into ARGO.

### Agent Router need and placement

Routing may occur:

- in VIA before any substantive Agent reasoning,
- inside ARGO after an initial reasoning turn,
- or adaptively between a small set of directed execution paths.

This directly changes DP-11.

### Task/Workflow Manager ownership

The architecture must decide whether Work/Task is always a VIA-owned lifecycle object or whether some short executions can remain purely conversational. Per-turn execution switching further requires stable task ownership across escalation.

### Context Engine scope

The context representation required for a local fast path differs from one used only for safe delegation. A per-turn topology needs context that can be reused across candidate paths without ambiguous re-interpretation.

### Agent Harness role

A Thin VIA makes the Harness the primary substantive execution seam. An ARGO-first topology may make the ARGO contract strategically privileged. A hybrid/adaptive topology requires a clear distinction between local capabilities and external Agent execution.

### Model invocation count and latency

The execution boundary directly determines whether a fast task requires:

- zero/one local model calls,
- a VIA model call plus an Agent call,
- an ARGO reasoning turn plus a further Agent delegation,
- or additional planning/routing turns.

This is why the QA rebaseline begins with end-to-end responsiveness rather than only VIA-internal software overhead.

### Agent ecosystem extensibility

A peer-Agent architecture makes replacement/addition easier. An ARGO-first architecture can simplify the preferred path while making other Agents subordinate to ARGO's delegation model. The trade-off must be measured rather than assumed.

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

**Advantages**

- Agent-neutral architecture.
- Strong and understandable VIA/Agent responsibility boundary.
- Easier Agent replacement and ecosystem expansion.
- Lower risk of duplicating domain planning/tool runtimes in VIA.

**Risks / costs**

- Short operations can pay serialization, dispatch and Agent startup/turn overhead.
- Even obvious bounded actions may take an unnecessarily long path.
- Routing and task creation can become mandatory ceremony.

### Alternative B — ARGO-first

```text
S2S / Interaction Frontend
        |
        v
Thin realtime/context VIA layer
        |
        v
ARGO — primary reasoning/execution authority
        |
        +----> optional specialized Agent delegation
```

ARGO receives the request as the primary ReAct-style reasoning authority. If another Agent is required, ARGO delegates onward.

**Advantages**

- Shortest path for work ARGO can already execute well.
- Avoids duplicating a rich planning layer in VIA.
- Potentially reduces VIA-side intent/planning complexity.

**Risks / costs**

- VIA may collapse toward a multimodal shell rather than a durable orchestration layer.
- Strong coupling to ARGO semantics and lifecycle.
- Other Agents may become ARGO sub-agents rather than peers.
- A non-ARGO request may pay an ARGO reasoning turn before being delegated elsewhere.
- Agent replacement and independent evolution may become harder.

### Alternative C — Hybrid VIA Fast Path

```text
                         +--> bounded VIA Fast Path
User -> VIA classify ----+
                         +--> Downstream Agent
```

VIA directly executes a narrowly defined set of local, latency-critical capabilities and delegates the rest.

**Fast Path eligibility must not be defined by arbitrary timing/count thresholds** such as “under 3 seconds” or “one LLM + one tool.” Those are measurements, not architecture semantics.

Eligibility should instead be based on properties such as:

- bounded execution;
- no domain planning requirement;
- no durable workflow requirement;
- no external Agent state requirement;
- simple failure/recovery semantics;
- capability is local-safe and appropriately owned by VIA;
- direct ownership creates a measurable latency benefit.

Examples may eventually include interaction controls or other allow-listed local operations, but the list is an architectural artifact to be justified, not inferred from convenience.

**Advantages**

- Preserves an Agent boundary for substantive work.
- Allows high-value latency optimization where VIA ownership is semantically justified.
- Avoids making all simple actions pay Agent dispatch cost.

**Risks / costs**

- Capability-placement policy becomes a first-class architecture concern.
- Risk of local capability creep and duplicate implementation.
- Requires clear fallback/escalation semantics when a fast-path attempt becomes non-bounded.

### Alternative D — Adaptive Per-turn Execution

```text
                       +--> VIA Fast Path
User -> VIA selector --+--> ARGO preferred Agent
                       +--> Specialized Downstream Agent
```

The execution topology is selected per turn rather than being fixed for the connection/session.

A likely safety constraint is **directed escalation rather than arbitrary path bouncing**.

Potentially acceptable:

```text
VIA Fast Path -> ARGO -> Specialized Agent
```

Undesirable:

```text
VIA -> ARGO -> VIA -> Agent B -> ARGO
```

Arbitrary ownership bouncing complicates:

- task ownership;
- context authority and provenance;
- side-effect deduplication;
- cancellation scope;
- retry semantics;
- progress ownership;
- recovery and audit.

ARGO is a plausible first-party preferred Downstream Agent in this topology, but that is **not yet decided**.

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

The subsequent deep inspection in `docs/references/internal-rust-voice-agent/execution-topology.md` is especially important. It shows that the prototype's **design intent** treats session mode as a layer-mounting boundary, while current production wiring does not enforce the connect-selected mode end-to-end. That discrepancy reinforces the need to treat the prototype as evidence, not as a baseline.

The prototype is most useful for demonstrating possible boundaries—Work ownership, downstream seams, safe result injection, provider abstraction—not for deciding DP-00 by precedent.

## Why connect-time mode is not automatically the DP-00 solution

The Rust reference's four modes are relevant because they attempt to make execution ownership explicit. However, DP-00 concerns the **architecture of request ownership**, not merely how a connection is configured.

A connect-time mode can be the correct answer only if the product semantics genuinely require a session to commit to one execution authority for its lifetime. If a single natural conversation routinely contains direct interaction, short local action, ARGO-capable work and specialized Agent work, a fixed session mode may force either unnecessary reconnect/mode switching or an overly broad mode.

Therefore the current hypothesis is that **per-turn selection may be a more natural abstraction than connect-time mode selection**, but this remains to be tested. It must be evaluated against complexity, predictability, state ownership and latency—not selected by intuition.

## Evaluation consequences

DP-00 cannot be evaluated by functional demos alone. Several alternatives can produce the same answer while differing materially in:

- latency;
- semantic correctness;
- task/lifecycle correctness;
- fault isolation;
- extensibility;
- resource use;
- model invocation count;
- coupling.

This led to the QA rebaseline methodology captured under `docs/evaluation/`.

The experiment must isolate **architecture effects** from stochastic model/Agent behavior. The independent variable should be the architecture alternative, while stochastic reasoning, network latency and Agent implementation variance are controlled during architecture qualification.

## QA-01 consequence

The first re-formalized QA is Fast-task End-to-End Responsiveness. It measures from **ground-truth acoustic end-of-speech** to the first **useful observable outcome**, not to an acknowledgment and not merely to an internal routing milestone.

This was chosen because DP-00 alternatives primarily alter the number and placement of architecture hops on tasks that could otherwise finish quickly.

Long-running task completion is intentionally not made the primary metric of QA-01 because external work duration can dominate the architecture effect.

## Reasoning checkpoint: what is decided and what is not

### Decided for the architecture-analysis process

- Introduce DP-00 as a prerequisite execution-boundary question.
- Preserve Approved Baseline v1.1 unchanged.
- Do not renumber/replace the existing QA/DP index yet.
- Define QA metrics independently before scoring A/B/C/D.
- Use one Primary Metric per QA for the 0–5 architecture score.
- Preserve sufficient raw observations so later metrics can be recomputed.
- Use deterministic semantic replay / controlled dependencies for architecture qualification.
- Keep actual-model validation separate as fidelity/external-validity evidence.

### Not decided

- Winner among DP-00 A/B/C/D.
- Whether ARGO is preferred, mandatory, or simply peer Downstream Agent.
- Exact VIA Fast Path capability allow-list.
- Whether per-turn topology is preferable to session-level topology after measurement.
- QA-01 score thresholds.
- Final QA numbering/rebaseline in `requirements-vNext.md` and `qa-dp-traceability.md`.

## Follow-on dependencies

DP-00 should be treated as an input to later decisions concerning:

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

This checkpoint intentionally adds new analysis/evaluation artifacts without modifying:

- `docs/requirements/requirements-v1.1.md`;
- the existing QA-01~QA-11 definitions/numbering in the Approved Baseline;
- `docs/architecture/qa-dp-traceability.md`;
- `docs/evaluation/evaluation-strategy.md`.

After QA-01 through QA-04 are formally defined, the working requirements, QA↔DP traceability and evaluation strategy should be coherently rebaselined together rather than incrementally creating inconsistent numbering.