# Architecture Experiment Methodology

## Purpose

This document defines how VIA compares SW architecture alternatives without allowing stochastic AI/dependency behavior, benchmark-fixture artifacts or topology-specific measurement boundaries to dominate the conclusion.

It is a project methodology, not a claim that there is a single standardized “LLM architecture replay” method.

## Experimental model

For DP-00:

```text
Independent variable = SW Architecture Alternative
Dependent variables   = QA Primary Metrics
Confounding variables = stochastic LLM output
                        Agent variability
                        network/service variability
                        machine/background/power state
                        scenario sampling differences
                        model/prompt/cache profile differences
                        uncontrolled fixture latency
```

The comparison boundary is the **Integrated Product**. A/B/C/D must satisfy the same user-visible scenarios; the architectural independent variable is the SW placement/ownership of reasoning, execution, routing and orchestration capability.

The experiment is useful only when metric differences can reasonably be attributed to architecture.

## Architecture Qualification

Architecture Qualification is the controlled comparison used for architecture scoring.

Use:

- the same frozen scenario corpus;
- deterministic semantic replay;
- deterministic Downstream Agent stubs;
- deterministic/replayed tool/service behavior;
- fixed Reference Development Machine;
- fixed power/background conditions;
- the same semantic trace for each comparable episode across A/B/C/D;
- frozen model/prompt/cache profiles where model-call behavior is evaluated;
- calibrated dependency-latency profiles where latency is evaluated.

This intentionally removes dependency creativity/variance as the dominant cause while preserving the architecture's real decision structure.

## Semantic replay boundary

Replay must occur at an architecture seam that represents dependency semantic output without bypassing the architecture component being evaluated.

Examples:

- If evaluating intent/routing topology, replay model semantic output but still execute the alternative's real validation/routing/task/dispatch path.
- If evaluating Agent lifecycle/integration, replay Agent progress/result/failure events through the real Harness/Task seam.
- If evaluating tool/action latency, use a controlled fake adapter that emits the frozen behavior profile.
- If QA-04 counts a model generation, deterministic replay still emits one logical ModelCall record when the architecture requested that generation.

A replay that replaces the architecture mechanism being measured is invalid.

## Actual-model trace generation and fidelity

Real models/Agents are used separately to obtain realistic behavior classes and validate that frozen traces remain representative.

Repeated runs may use GPT, Qwen, local models or first-party Agents to observe:

- correct semantic output;
- ambiguous/low-confidence output;
- partially wrong referent/candidate;
- wrong execution-path recommendation;
- wrong Agent recommendation;
- malformed structured output;
- timeout/no response;
- provider-specific edge behavior.

These observations are normalized into a **frozen semantic trace corpus**.

Architecture Qualification intentionally replays both correct and erroneous traces. The question is not whether the model made the error; the error is held equal. The question is whether one architecture validates/recovers better than another.

## Trace corpus size

Do not claim an arbitrary fixed `N` as a universal standard.

Use Pilot collection and inspect semantic behavior-class saturation: continue while meaningful new classes/failure shapes are still appearing. Record the collection policy and final corpus composition.

This is a pragmatic design rule, not a theorem that saturation proves completeness.

## Two-track evaluation

### Track A — Architecture Qualification

Purpose: isolate architecture effects.

```text
Frozen semantic traces
+ deterministic Agent/tool behavior
+ controlled and calibrated dependency profiles
+ frozen model/prompt/cache profiles
+ controlled machine state
        |
        v
A / B / C / D
        |
        v
QA Primary + Secondary Metrics
```

Only this track feeds architecture scoring unless a QA explicitly defines otherwise.

### Track B — Actual-model / Real-stack Validation

Purpose: external validity/fidelity.

Questions include:

- Do real dependencies still exhibit the behavior classes represented in frozen traces?
- Are important behavior classes missing?
- Do provider/model upgrades change distributions materially?
- What are the real Agent/network/tool latency differences?
- Are prompt/cache/resource assumptions stable on the intended deployment?

These results are reported separately from controlled architecture scores.

## Pilot and freeze sequence

Required sequence:

```text
1. Pilot
2. Inspect distributions, strictness, instrumentation and confounders
3. Calibrate thresholds, populations, dependency profiles and gates
4. Freeze scoring/gate/benchmark versions
5. Run final A/B/C/D evaluation
6. Derive results without changing the frozen rules
```

If a Primary Metric or gate later changes:

1. record why;
2. define a new version;
3. recompute all alternatives equally from preserved raw evidence where possible;
4. rerun only when required observations were not captured.

Final ranking must not drive post-hoc rule changes.

## Raw, derived and report separation

```text
results/raw/
    immutable experiment evidence

results/derived/
    metrics recomputed from raw data

results/reports/
    human-readable summaries, charts and decisions
```

Raw evidence is the source of truth.

## Controlled dependency patterns

### Deterministic semantic model double

Input: scenario/trace id.

Output: frozen semantic result, ambiguity/confidence class, candidate path/Agent, optional failure/delay shape.

### Deterministic Agent stub

Expose the same Agent/Harness contract and reproduce selected behavior:

- fixed result delay;
- progress sequence;
- permission request;
- cancellation request/confirmation;
- timeout/failure;
- restart/recovery where relevant.

### Controlled tool/service adapter

Produce the machine-observable effect using a frozen latency/behavior profile. Avoid live cloud variance during Architecture Qualification unless that dependency itself is the architecture variable.

## QA-01 dependency-latency control

Equality of dependency behavior across A/B/C/D is necessary but not sufficient for QA-01.

A fixed fixture can still dominate FTOL p95 if one class is much slower than the architecture overhead.

Example risk:

```text
F1 controlled dependency       50 ms
F2 controlled dependency      200 ms
F3 controlled dependency      800 ms
```

Even if all alternatives have the same SW overhead, F3 can mechanically dominate the overall p95.

Architecture Qualification therefore requires:

```text
Equality:
  comparable alternatives receive the same controlled dependency behavior.

Non-dominance:
  dependency profile is calibrated so a fixture class does not overwhelm
  the architecture-induced FTOL difference being measured.
```

Pilot must inspect class-specific distributions, p95 tail composition and sensitivity to reasonable controlled latency profiles.

Exact dependency-latency numbers are TBD until calibration and are frozen before final scoring.

Actual production dependency latency remains a Track-B result.

## Topology-neutral correctness normalization

A/B/C/D may have different internal component graphs. QA-02 therefore does not use component presence/call sequence as the common oracle.

Each alternative emits a **Canonical Architecture Decision Trace** containing common outcomes such as:

- referent bindings;
- task relation/id;
- execution owner/path;
- delegated Agent;
- clarification action;
- result binding;
- observable effect;
- compound decomposition.

The evaluator applies a versioned Required/Allowed/Forbidden Architecture Constraint Manifest.

Constraint evidence must come from requirement/policy/capability/scenario semantics, not from the topology under test.

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
Deterministic Agent / Controlled Tool
       |
       v
Canonical Architecture Decision Trace
       |
       v
Constraint Evaluator
       |
       +--> per-constraint evidence
       +--> episode_exact_conform
```

The Primary Metric is AECR; per-constraint/dimension metrics are mandatory diagnostics.

## Scenario taxonomy and coverage balance

Architecture-scoring corpora prioritize architecture-sensitive coverage over blindly reproducing production frequency.

Freeze/version applicable taxonomy, category mix, eligibility, manifests, trace mix and aggregation rules before final evaluation.

Usage-frequency-weighted results may be computed as Secondary sensitivity analysis.

## Evolution Flexibility methodology

QA-03 evaluates code/configuration evolution rather than runtime events.

Fair comparison sequence:

```text
Common Evolution Requirement
  -> Common Expected Change Roles
  -> alternative-specific role mapping
  -> freeze
  -> implementation
  -> diff + acceptance/regression evaluation
```

Expected Change Area is defined by architecture role, not one alternative's filenames/components. Mapping and role boundaries cannot be enlarged after observing the diff.

QA-03 uses dedicated evolution scenario/run schemas.

## QA-04 measurement boundary normalization

QA-04 counts architecture-required logical ORCHESTRATION/MIXED generations from:

```text
request_processing_start_ts
```

to:

```text
execution_route_commit_ts
```

**Execution Route Commit** is the earliest point at which the final domain execution route is operationally committed and no further owner/delegation decision is required before domain execution can continue.

This boundary is chosen because component-local events such as `execution_started_ts` are topology-dependent.

### Hidden-routing example

```text
A: Intent Model -> Router Model -> Specialist accept
B: ARGO starts -> ARGO Model chooses Specialist -> Specialist accept
```

Stopping at intermediate execution start can make B's routing call disappear. Execution Route Commit keeps equivalent owner/delegation responsibility inside measurement regardless of component placement.

The rule is normalization, not a penalty against B.

### QA-04 call inclusion

```text
Primary include when:
  call_class in {ORCHESTRATION, MIXED}
  AND call is before route commit or causes route commit
```

Pure DOMAIN reasoning after route commit is excluded.

Pure voice-output generation is excluded unless the same logical generation also performs route/owner/delegation decision, in which case it is MIXED.

Logical model-call count does not claim physical compute equivalence. Model/prompt/cache profiles are frozen and token/cache/latency/CPU/GPU/NPU/memory/energy/cost telemetry is retained.

## Correctness qualification gate

The Top QA set intentionally keeps correctness independent from latency, flexibility and model-call overhead.

However, final DP-00 selection should not allow an incorrect architecture to win because it is fast or uses few calls.

Recommended selection rule:

```text
if QA-02 AECR < frozen minimum acceptable threshold:
    alternative is ineligible for final selection
```

The numeric threshold is TBD and follows:

```text
Pilot -> calibration -> gate-rule freeze -> final evaluation
```

Do not fold this gate into QA-01/QA-04 formulas as an arbitrary penalty.

## Scored QAs vs must-pass constraints

Not every important requirement belongs in a compensable 0–5 score.

Final evaluation may separate:

```text
Scored drivers:
  QA-01 responsiveness
  QA-02 correctness
  QA-03 flexibility
  QA-04 model-call overhead

Must-pass gates:
  security
  privacy/context-sharing policy
  cancellation semantics
  task-state integrity
  failure containment
  mandatory recovery
  trusted-boundary requirements
```

A must-pass violation cannot be compensated by another QA's high score.

The exact executable gate set is traced/frozen in a later central rebaseline/benchmark checkpoint.

## Reference machine and environment

Record enough metadata to reproduce time/resource measurements:

- hardware profile;
- OS/build;
- power mode;
- process priority where controlled;
- background-load policy;
- source commit;
- benchmark/trace versions;
- dependency-latency profile;
- model/prompt/cache profiles.

Warm/cold state must be defined rather than incidental.

## Methodological grounding

The methodology draws on established ideas without claiming a new formal standard.

### SEI ATAM

ATAM motivates scenario-based quality-attribute trade-off analysis and identifying architecture-sensitive decisions. VIA's numeric scoring is project-specific.

### Controlled experiments

Hold non-target causes stable enough that measured differences can be attributed to the architecture alternative.

### Test doubles

Fakes/stubs provide deterministic dependency and failure behavior while exercising real architecture seams.

### Record/replay testing

Captured/normalized dependency behavior can be replayed deterministically to reproduce realistic interaction without requiring live services for every run.

No claim is made that these sources define a standardized LLM architecture replay benchmark.

## Validity threats to report

Every final evaluation should discuss at least:

- **construct validity** — does each Primary Metric measure its intended QA?
- **internal validity** — are model/network/machine/fixture confounders controlled?
- **external validity** — do frozen traces/profiles still resemble real systems?
- **implementation fidelity** — are A/B/C/D prototypes comparably mature?
- **instrumentation effect** — does tracing change measured behavior materially?
- **oracle neutrality** — do QA-02 constraints permit all valid topologies?
- **population sensitivity** — how dependent are AECR/QA-04 mean on corpus mix?
- **fixture dominance** — does controlled external latency dominate QA-01?
- **boundary gaming** — can equivalent responsibility move outside a metric boundary because of topology placement?

## QA applications

### QA-01

Ground-truth acoustic EOS to useful outcome; successful Fast-task episodes only; dependency behavior controlled for equality and non-dominance.

### QA-02

Canonical Architecture Decision Trace evaluated against Required/Allowed/Forbidden constraints; Primary = AECR.

### QA-03

Evolution scenarios evaluated against pre-frozen Expected Change Roles and alternative role mappings; Primary = CCR.

### QA-04

Logical ORCHESTRATION/MIXED model calls from request-processing start through Execution Route Commit; Primary = Average Model Calls to Commit Execution Route.

## Central rebaseline status

This methodology update does not modify:

- `docs/requirements/requirements-v1.1.md`;
- central QA↔DP traceability;
- existing central `evaluation-strategy.md`.

Those are intentionally deferred to a separate central vNext rebaseline checkpoint.
