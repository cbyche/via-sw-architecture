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

- If evaluating intent/routing topology, replay the model semantic result but still execute the alternative's router/task/dispatch code.
- If evaluating Agent integration/lifecycle, replay Agent progress/result/failure events through the real Harness/Task boundary.
- If evaluating tool/action latency, use a controlled fake adapter that emits the same observable action completion timing.

A replay that replaces the architecture component being measured is invalid.

## Actual-model trace generation and fidelity

Real models/Agents are used separately to obtain realistic behavior classes and validate that the replay corpus remains representative.

Repeated runs may use GPT, Qwen, local models or relevant first-party Agents. The purpose is not to score architecture directly but to observe semantic behavior such as:

- correct semantic result;
- ambiguous / low-confidence result;
- partially wrong candidate;
- wrong execution-path or Agent candidate;
- malformed schema;
- timeout / no response;
- provider-specific edge behavior that affects an architecture seam.

These observations are normalized into a **frozen semantic trace corpus**.

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
2. Inspect distributions and instrumentation
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

## QA-01 application

For Fast-task End-to-End Responsiveness, Architecture Qualification uses ground-truth acoustic EOS as the start timestamp and scenario-defined useful outcome as the endpoint. Semantic replay and deterministic Agent/tool behavior ensure that A/B/C/D latency differences primarily represent architecture path selection, dispatch and ownership topology.

The raw event schema is defined in `benchmark/schemas/run-event-schema.md`.