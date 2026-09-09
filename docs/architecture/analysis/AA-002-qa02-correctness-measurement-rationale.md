# AA-002 — QA-02 Correctness Measurement Rationale

## Status

Architecture Analysis Record — Context Checkpoint 002

This record preserves the reasoning behind `QA-02 — VIA Interaction-Orchestration Correctness`, including the stress tests that shaped its metric and benchmark contract.

It is not an ADR and does not change `docs/requirements/requirements-v1.1.md`.

## Why this record exists

A correctness QA can look deceptively simple: run a task and check whether it succeeded. That is insufficient for DP-00 because the alternatives deliberately move reasoning/execution ownership across VIA, ARGO and specialized Downstream Agents.

The benchmark must distinguish:

- correctness attributable to architecture;
- correctness attributable to stochastic model output;
- correctness attributable to Agent domain reasoning;
- correctness attributable to external tools/services.

Without that separation, the experiment cannot explain whether an observed difference came from the architecture or from a dependency.

## Why final task success is not the architecture Primary Metric

The Integrated Product ultimately matters to the user, but final task success is not controlled solely by VIA architecture.

For example, a request can be correctly grounded, associated, routed and bound by VIA while a Downstream Agent later produces a poor document. Conversely, a lucky Agent may compensate for an architecturally wrong initial path.

If final content/task success were the architecture-only Primary Metric, the result would conflate:

- LLM language understanding;
- ARGO planning quality;
- specialized Agent capability;
- tool/service behavior;
- VIA context/task/routing/state correctness.

QA-02 therefore measures the correctness that the architecture owns: the chain of interaction and orchestration decisions required to carry one user goal safely and consistently through the Integrated Product.

Product E2E success remains a separate integration/fidelity observation.

## Why component accuracy averages are not the Primary Metric

A first idea is to score components separately and average them:

```text
referent accuracy
+ task association accuracy
+ routing accuracy
+ result binding accuracy
------------------------------
component average
```

This is useful diagnostically but weak as the Primary Metric.

A user-goal episode can be unusable when one critical decision is wrong even if the other components are correct. Examples:

- correct source referent + wrong destination;
- correct routing + wrong task association;
- correct task execution + result delivered to the wrong active task;
- correct Agent + execution performed despite required clarification.

A mean can hide these structural failures because many easy sub-decisions compensate for one severe violation.

The selected Primary Metric therefore uses **episode-level exact conformance**. Component/constraint accuracy remains Secondary evidence.

## Why the episode, not the turn, is the unit

Architecture correctness often spans multiple turns.

A clarification flow is the simplest example:

```text
Turn 1: ambiguous request
Architecture: detect ambiguity and ask
Turn 2: user resolves ambiguity
Architecture: bind the corrected target and continue the same goal
```

Scoring each turn independently would reward the clarification request but fail to test whether the subsequent answer was correctly associated with the same logical goal.

The unit of evaluation is therefore one **user-goal episode**, which may span multiple turns and task-state transitions.

## Why one `expected_path` is architecture-biased

DP-00 alternatives intentionally differ in SW placement/ownership.

Consider:

```text
"볼륨 줄여줘."
```

A Hybrid/Adaptive architecture may use `VIA_FAST`; an ARGO-centric architecture may send the goal to ARGO and still produce a semantically correct outcome.

If the benchmark declares:

```text
expected_path = VIA_FAST
```

then the oracle already assumes the C/D topology. If it declares `ARGO`, it biases toward B.

That would turn the experiment into a test of agreement with the benchmark author's preferred architecture rather than correctness.

## Required / Allowed / Forbidden constraints

The solution is an **Architecture Constraint Manifest**.

A scenario states architecture semantics rather than a single implementation path.

Example:

```text
Required:
  source_referent = file_A
  destination_referent = folder_C
  task_relation = FOLLOW_UP
  task_id = T1
  result_binding = T1

Allowed:
  execution_owner in {ARGO, FileAgent}

Forbidden:
  execution_owner = VIA_FAST
  delegated_agent = MailAgent
```

This allows multiple valid topologies while still making incorrect architecture decisions explicit.

The rule also separates QA concerns cleanly. If both `VIA_FAST` and `ARGO` are allowed for a simple local task, QA-02 gives both full correctness credit; QA-01 reveals which path is faster.

## How structurally different A/B/C/D are compared

The alternatives cannot be compared using internal component names because some components do not exist in every topology.

For example:

- A may have an explicit VIA Agent Router;
- B may make ARGO the primary reasoning authority and have no equivalent VIA router call;
- C may perform local eligibility checks before delegation;
- D may select among multiple owners per turn.

The benchmark therefore requires each implementation to emit a **Canonical Architecture Decision Trace** containing topology-neutral outcomes such as:

- referent bindings;
- task relation/id;
- execution owner;
- delegated Agent;
- clarification action;
- result binding;
- observable effect;
- decomposition relations for compound requests.

The constraint evaluator operates on this canonical trace, not on component call graphs.

This preserves a common comparison boundary at the Integrated Product while keeping the independent variable as **reasoning/execution capability placement and ownership**.

## Why wrong model outputs are replayed instead of removed

Controlling model variability does not mean assuming the model is perfect.

If every architecture receives only the ideal semantic answer, the benchmark cannot test architecture mechanisms such as:

- validation;
- confidence/ambiguity policy;
- safe fallback;
- clarification;
- path eligibility checks;
- schema validation;
- timeout handling.

Real model/Agent runs are therefore used to observe behavior classes, including failures. Those outputs are normalized and frozen.

The same erroneous trace is then replayed to A/B/C/D.

Example:

```text
Frozen semantic output:
  recommended_owner = VIA_FAST
  confidence = high

Scenario truth:
  task requires planning and durable Agent state
```

The question is not whether the model was wrong—it is fixed as wrong for every alternative. The question is whether the architecture blindly executes the proposal, validates it, escalates it, or asks for clarification.

That comparison is strongly architecture-sensitive.

## Why AECR is strict

Architecture Episode Exact Conformance Rate treats an episode as conformant only if **all declared constraints** are satisfied.

This strictness is intentional because the architecture is coordinating one user goal. Correct routing cannot compensate for wrong result binding, and correct referent binding cannot compensate for forbidden execution without clarification.

However, strictness creates a known risk: as episodes become constraint-rich, all alternatives can accumulate low exact scores and differences may be harder to interpret.

Therefore the methodology requires preservation of:

- every constraint result;
- component/slice metrics;
- failure ids and reasons;
- category-specific AECR;
- macro averages and critical slices.

If AECR later proves unsuitable as the Primary Metric, the project must not switch metrics merely because the final ranking is inconvenient. The reason must be recorded, a new scoring version created, and all alternatives recomputed equally from raw evidence.

## Why scenario mix is part of the metric

AECR is a population statistic. Its value changes with the scenario distribution.

If 80% of the benchmark consists of easy local requests, an architecture can appear highly correct while failing systematically on:

- temporal referent binding;
- existing-task follow-up;
- clarification;
- concurrent result ownership;
- malformed model outputs.

Architecture Qualification therefore uses a **coverage-balanced architecture-sensitive corpus** rather than blindly mirroring production usage frequency.

The taxonomy and corpus mix are versioned and frozen before final A/B/C/D scoring.

A production-frequency-weighted result is still useful, but it is a Secondary sensitivity analysis rather than the architecture qualification score.

## QA-02 scenario taxonomy

The minimum structural categories are:

- **C1 Context / Referent Grounding**
- **C2 Task Association / Follow-up**
- **C3 Execution-path Eligibility**
- **C4 Agent Routing / Delegation**
- **C5 Ambiguity / Clarification**
- **C6 Concurrent Task / Result Binding**
- **C7 Compound Request Decomposition**
- **C8 Dependency-error / Invalid-model-output Handling**

These categories are architecture coverage dimensions, not mutually exclusive product use-case labels. A scenario may exercise multiple dimensions but should have a declared primary category for corpus balancing/reporting.

## Stress-test scenarios that shaped the metric

The following scenarios were used as feasibility tests for the proposed metric and should be retained as benchmark-design inputs.

### 1. Simple Local Task

Example: reduce volume.

- `VIA_FAST` and `ARGO` may both be allowed.
- QA-02 should mark both correct.
- QA-01 should expose the latency difference.

This scenario demonstrated why one expected path is invalid.

### 2. False Fast-path Temptation

The frozen model trace recommends Fast Path, but the task actually requires planning or durable external state.

The scenario tests whether the architecture independently enforces eligibility semantics rather than trusting a semantic recommendation blindly.

### 3. Multi-point Referent

One utterance identifies source, destination and a correction using changing pointer/context evidence.

The scenario tests whether context representation/binding preserves the required referents without making one internal Context Engine implementation mandatory.

### 4. Existing Task Follow-up

A user instruction must continue an existing task/ARGO thread rather than start new Work.

This tests task-state authority, association and downstream-session continuity.

### 5. Specialized Agent Required

Both of the following may be allowed:

```text
ARGO -> Specialized Agent
Direct Specialized Agent
```

The correctness oracle should require the right eventual execution capability/Agent while remaining neutral to the top-level topology.

### 6. Required Clarification

The requested file/contact is genuinely ambiguous.

Execution without clarification is forbidden even if the resulting external action accidentally targets one plausible candidate.

This exposes the correctness cost of overly aggressive architectures.

### 7. Concurrent Result Binding

Several tasks produce interleaved progress/results.

The architecture must attach each result to the correct logical task and user goal. This is close to pure software-state correctness and is especially useful for isolating architecture effects.

### 8. Wrong Model Execution-path Proposal

Every alternative receives the same incorrect `VIA_FAST` candidate.

The benchmark measures validator/policy/fallback differences rather than model quality.

Together these scenarios demonstrate that AECR is implementable, topology-neutral and architecture-sensitive.

## Relationship to DP-00

DP-00 compares execution ownership across the **Integrated Product**. All A/B/C/D alternatives must satisfy the same user-visible goals and benchmark scenarios.

The independent variable is not whether a component named `IntentRefiner` or `AgentRouter` exists. It is where reasoning/execution capability is placed and who owns the corresponding decisions/state.

QA-02 therefore evaluates architecture outcomes at that common boundary.

## Relationship to QA-01

QA-01 and QA-02 are intentionally separate.

```text
QA-01: when a valid outcome is reached, how quickly is the useful outcome reached?
QA-02: does the architecture satisfy the required interaction/orchestration constraints?
```

A slower but valid ARGO path and a faster valid Fast Path can both score equally on QA-02 and differently on QA-01.

A fast forbidden path can score well on raw latency but fails QA-02.

This separation prevents one arbitrary penalty formula from hiding the trade-off.

## Checkpoint decisions

Agreed for QA-02 methodology:

- Use one user-goal episode as the unit.
- Use AECR as the Primary Metric.
- Use Required/Allowed/Forbidden scenario constraints.
- Do not prescribe one implementation path when multiple are architecturally valid.
- Normalize alternatives into a Canonical Architecture Decision Trace.
- Replay realistic correct and erroneous semantic behavior identically across alternatives.
- Preserve all per-constraint raw evidence because AECR is intentionally strict.
- Use a coverage-balanced, versioned/frozen architecture-sensitive corpus for scoring.
- Treat usage-frequency weighting as Secondary sensitivity analysis.
- Calibrate and freeze score thresholds before final comparative evaluation.

Not decided:

- AECR score cutoffs;
- exact scenario counts per category;
- concrete JSON/Parquet event serialization;
- final QA numbering in the Approved Baseline;
- DP-00 winner.
