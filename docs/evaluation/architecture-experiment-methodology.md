# Architecture Experiment Methodology

## Purpose

This document defines how VIA compares SW architecture alternatives without allowing stochastic AI/dependency behavior to dominate the conclusion.

It is a project methodology, not a claim that there is a single standardized “LLM architecture replay” method.

## Experimental model

For an architecture Decision Point such as DP-00:

```text
Independent variable = SW Architecture Alternative
Dependent variables   = QA Primary Metrics
Confounding variables = stochastic LLM output
                        Agent variability
                        network/service variability
                        machine/background/power state
                        scenario sampling differences
```

The experiment is useful only when the observed metric difference can reasonably be attributed to the architecture alternative.

For DP-00 specifically, the comparison boundary is the **Integrated Product**. A/B/C/D must satisfy the same user-visible scenarios; the architectural independent variable is the SW placement/ownership of reasoning, execution, routing and orchestration capability.

## Architecture Qualification

Architecture Qualification is the controlled comparison used for architecture scoring.

Use:

- the same frozen scenario corpus;
- deterministic semantic replay;
- deterministic Downstream Agent stubs;
- deterministic or replayed tool/service latency;
- fixed Reference Development Machine;
- fixed power/background conditions;
- the same semantic trace for each comparable episode across A/B/C/D.

This intentionally removes model creativity/quality as a source of experimental variance. The architecture still has to react correctly to the semantic input it receives.

### Semantic replay boundary

The replay should occur at an architecture seam that represents the dependency's semantic output, not by bypassing the architecture mechanism under evaluation.

Examples:

- If evaluating intent/routing topology, replay the model semantic result but still execute the alternative's real validation/routing/task/dispatch code.
- If evaluating Agent integration/lifecycle, replay Agent progress/result/failure events through the real Harness/Task boundary.
- If evaluating tool/action latency, use a controlled fake adapter that emits the same observable action completion timing.

A replay that replaces the architecture component being measured is invalid.

## Actual-model trace generation and fidelity

Real models/Agents are used separately to obtain realistic behavior classes and validate that the replay corpus remains representative.

Repeated runs may use GPT, Qwen, local models or relevant first-party Agents. The purpose is not to score architecture directly but to observe semantic behavior such as:

- correct semantic result;
- ambiguous / low-confidence result;
- partially wrong referent/candidate;
- wrong execution-path recommendation;
- wrong Agent recommendation;
- malformed structured output/schema;
- timeout / no response;
- provider-specific edge behavior that affects an architecture seam.

These observations are normalized into a **frozen semantic trace corpus**.

### Replay errors intentionally

The qualification corpus must not contain only ideal answers.

A useful architecture may distinguish itself by detecting or recovering from the same wrong semantic output better than another architecture. Therefore realistic incorrect traces are frozen and replayed identically to A/B/C/D.

Example:

```text
semantic replay says: VIA_FAST candidate
scenario semantics say: planning + durable Agent state required
```

The model error is controlled and equal. The measured difference is whether the architecture validates eligibility, escalates safely, asks for clarification, or blindly executes the invalid proposal.

## Trace corpus size

Do not claim an arbitrary fixed `N` as a universal standard.

Use a pilot collection and inspect **semantic behavior-class saturation**: continue sampling while meaningful new behavior classes or materially different failure shapes are still appearing. Record the collection policy and final corpus composition.

This is a pragmatic experimental design rule, not a statistical theorem that saturation alone establishes completeness.

## Two-track evaluation

### Track A — Architecture Qualification

Purpose: isolate architecture-only effects.

```text
Frozen semantic traces
+ deterministic Agent/tool behavior
+ controlled environment
        |
        v
A / B / C / D alternatives
        |
        v
QA Primary + Secondary Metrics
```

Only this track feeds the architecture score unless a QA explicitly defines otherwise.

### Track B — Actual-model Validation

Purpose: check external validity/fidelity.

Questions include:

- Do real dependencies still exhibit the semantic classes represented in the frozen corpus?
- Are important classes missing?
- Do provider/model upgrades materially change behavior distributions?
- Does an architecture assumption rely on behavior that is not stable in real systems?

These results are reported separately from architecture qualification scores.

## Pilot and scoring freeze

The required sequence is:

```text
1. Pilot
2. Inspect distributions, strictness and instrumentation
3. Calibrate thresholds / scenario composition
4. Freeze scoring-v1 and benchmark versions
5. Run final A/B/C/D evaluation
6. Derive scores without changing thresholds
```

Thresholds may be changed after a pilot if the calibration rationale is recorded. They must not be changed to favor an alternative after the final comparative result is visible.

If a Primary Metric must later change:

1. record why;
2. define a new scoring version;
3. recompute all alternatives from preserved raw data where possible;
4. rerun only if the required raw observation was not captured.

## Raw, derived and report separation

```text
results/raw/
    immutable event records

results/derived/
    metrics recomputed from raw data

results/reports/
    human-readable summaries, charts and decisions
```

Derived and report artifacts can be regenerated. Raw events are the experimental evidence.

## Controlled dependency patterns

### Deterministic semantic model double

Input: scenario/trace identifier.

Output: predefined semantic result, confidence/ambiguity class, candidate path/Agent, optional delay/failure shape.

### Deterministic Agent stub

Expose the same Harness/Agent contract as the prototype alternative and reproduce selected behavior:

- fixed result delay;
- progress sequence;
- permission request;
- cancellation acknowledgment/confirmation;
- timeout;
- failure;
- restart/recovery behavior where applicable.

### Controlled tool/service adapter

Produce a machine-observable outcome using a fixed/replayed latency profile. Avoid live cloud/API variance in architecture qualification unless the external service itself is the architecture variable.

## Topology-neutral correctness normalization

A/B/C/D may have different internal component graphs. A correctness benchmark must therefore not use component presence/call sequence as the common oracle.

Each alternative projects its architecture-relevant outcome into a **Canonical Architecture Decision Trace** containing common semantic fields such as:

- referent bindings;
- task relation and logical task id;
- execution owner/path;
- delegated Agent;
- clarification action;
- result binding;
- observable effect;
- compound-request decomposition.

The implementation may internally use an Intent Refiner, an Agent Router, ARGO-first reasoning, a local Fast Path, or per-turn adaptive selection. The evaluator sees the normalized architecture outcome.

This is the key mechanism for comparing structurally different alternatives without making one component architecture the oracle.

## Constraint-based scenario oracle

QA-02 scenarios use a versioned **Architecture Constraint Manifest** rather than one `expected_path`.

The manifest contains:

- **Required** constraints — must hold;
- **Allowed** constraints — sets/alternatives that are all valid;
- **Forbidden** constraints — must not occur.

Example:

```text
Required:
  task_relation = FOLLOW_UP
  task_id = T1
  result_binding = T1

Allowed:
  execution_owner in {ARGO, FILE_AGENT}

Forbidden:
  execution_owner = VIA_FAST
```

A single expected execution path would bias the benchmark toward a particular DP-00 topology. Constraint sets preserve topology neutrality while still detecting incorrect grounding, task ownership, routing, clarification and result binding.

The manifest contract is defined in `benchmark/schemas/scenario-constraint-schema.md`.

## QA-02 qualification pipeline

```text
Scenario Fixture
       |
       v
Frozen Semantic Replay
       |
       v
Architecture A / B / C / D
       |
       v
Deterministic Agent Stub / Controlled Tool
       |
       v
Canonical Architecture Decision Trace
       |
       v
Constraint Evaluator
       |
       +--> per-constraint evidence
       +--> episode_exact_conform
       |
       v
AECR derived over frozen eligible corpus
```

The replay may be correct, ambiguous, partially wrong, wrong-path, wrong-Agent, malformed or timed out. Every alternative receives the same comparable trace.

## Episode-level correctness

Correctness often spans more than one turn. Clarification and follow-up are examples where turn-local scoring can miss whether state is carried correctly across the logical user goal.

QA-02 therefore uses one **user-goal episode** as the unit. The episode may contain clarification turns, task continuation, execution handoff and result binding.

The Primary Metric is exact at the episode level; component/dimension metrics are Secondary diagnostics.

## Scenario taxonomy and coverage balance

QA-02 requires architecture-sensitive structural coverage across at least:

- C1 Context / Referent Grounding;
- C2 Task Association / Follow-up;
- C3 Execution-path Eligibility;
- C4 Agent Routing / Delegation;
- C5 Ambiguity / Clarification;
- C6 Concurrent Task / Result Binding;
- C7 Compound Request Decomposition;
- C8 Dependency-error / Invalid-model-output Handling.

The final qualification corpus should be **coverage-balanced**, not merely proportional to observed usage frequency. Otherwise high-frequency simple requests can hide structural failures in lower-frequency but architecture-critical scenarios.

Because AECR depends on scenario population, freeze and version:

- taxonomy;
- category composition;
- scenario eligibility;
- scenario/constraint manifests;
- semantic trace mix;
- scoring version.

A production-frequency-weighted result can be computed as a Secondary sensitivity analysis.

## Strict Primary Metrics and diagnostic evidence

An all-or-nothing episode metric such as AECR has a known trade-off: it protects against severe errors being averaged away, but may compress alternatives toward low values as constraint count increases.

Therefore raw qualification data must retain:

- actual canonical trace;
- every applicable constraint result;
- dimension-level conformance;
- failure ids/reasons;
- category and critical-slice identifiers.

If pilot evidence shows AECR has poor construct validity or discrimination, any replacement must use a new scoring version and be applied equally to all alternatives. Final rankings must not trigger ad-hoc metric changes.

## Reference machine and environment

Record enough metadata to reproduce time/resource measurements, including:

- hardware profile;
- OS/build;
- power mode;
- benchmark process priority if controlled;
- relevant background-load policy;
- source commit;
- benchmark and trace versions.

Warm/cold state must be defined by the QA/benchmark rather than left incidental.

## Methodological grounding

The methodology draws on established ideas rather than claiming a new formal standard:

### SEI ATAM

ATAM motivates scenario-based analysis of quality-attribute trade-offs and identifying architecture-sensitive decisions. VIA's DP/QA/scenario structure follows that spirit, while numeric benchmark scoring is a project-specific extension.

### Controlled experiments

The independent-variable/confounding-variable discipline is conventional experimental design: hold non-target causes stable enough that measured differences can be attributed to the architecture alternative.

### Test doubles

Fakes/stubs allow deterministic control of dependencies and failure modes while exercising the real architecture seam.

### Record/replay testing

Captured/normalized behavior can be replayed deterministically to reproduce realistic dependency interaction without requiring the original live service on every architecture run.

No claim is made that one of these sources defines a standardized LLM architecture replay benchmark.

## Validity threats to report

Every final evaluation should discuss at least:

- **construct validity** — does the Primary Metric measure the intended QA?
- **internal validity** — were model/network/machine confounders controlled?
- **external validity** — do frozen traces still resemble real model/Agent behavior?
- **implementation fidelity** — are A/B/C/D prototypes comparably mature and using equivalent shared infrastructure?
- **instrumentation effect** — could tracing itself materially change latency/resource results?
- **oracle neutrality** — do constraints permit all semantically valid topologies rather than encode a preferred alternative?
- **population sensitivity** — how dependent is AECR on the frozen category/scenario mix?

## QA-01 application

For Fast-task End-to-End Responsiveness, Architecture Qualification uses ground-truth acoustic EOS as the start timestamp and scenario-defined useful outcome as the endpoint. Semantic replay and deterministic Agent/tool behavior ensure that A/B/C/D latency differences primarily represent architecture path selection, dispatch and ownership topology.

## QA-02 application

For VIA Interaction-Orchestration Correctness, Architecture Qualification evaluates a Canonical Architecture Decision Trace against a versioned Required/Allowed/Forbidden constraint manifest over a frozen coverage-balanced corpus.

The Primary Metric is **Architecture Episode Exact Conformance Rate (AECR)**. Per-dimension and per-constraint metrics remain Secondary evidence.

The raw event schema is defined in `benchmark/schemas/run-event-schema.md` and the scenario oracle contract in `benchmark/schemas/scenario-constraint-schema.md`.
