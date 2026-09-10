# AA-005 — Top QA Cross-review

## Status

Architecture Analysis Record — Top Architectural Driver Cross-review Checkpoint

This record cross-reviews QA-01 through QA-04 as the proposed scored Top Architectural Driver set for DP-00.

It is an **evaluation-design readiness review**, not an experiment result, not an ADR, and not a requirements-baseline change.

`docs/requirements/requirements-v1.1.md` remains the Approved Baseline and is not modified by this record.

## Purpose

QA-01~04 were initially defined one by one. Before central vNext rebaseline, they must also be reviewed as a set.

The cross-review asks:

- Do the four QAs measure distinct architecture concerns rather than the same signal four times?
- Are they sensitive to the actual structural differences in DP-00 A/B/C/D?
- Can they compare structurally different alternatives without making one topology the oracle?
- Are measurement boundaries resistant to obvious metric gaming?
- Can stochastic model/Agent behavior and fixture behavior be controlled well enough to isolate architecture effects?
- Are important security/reliability obligations accidentally being treated as optional because they are not in the scored Top 4?

Two amendments were required by this review:

1. QA-01 Architecture Qualification must control external/dependency latency for both **equality and non-dominance**.
2. QA-04 must end at **Execution Route Commit**, not merely at intermediate owner confirmation/execution start.

With those controls, the Top QA set is considered ready for Pilot/calibration work.

# 9.1 Final Top QA Set

| QA | Architectural question | Primary Metric |
| --- | --- | --- |
| **QA-01 Fast-task End-to-End Responsiveness** | 빠른가? | **Fast-task Outcome Latency p95 (FTOL p95)** |
| **QA-02 VIA Interaction-Orchestration Correctness** | 정확한가? | **Architecture Episode Exact Conformance Rate (AECR)** |
| **QA-03 변경 대응 용이성 (Flexibility)** | 변경이 잘 격리되는가? | **Change Containment Rate (CCR)** |
| **QA-04 모델 호출 오버헤드 (Model Call Overhead)** | 실행 경로를 정하기 위해 AI 판단을 얼마나 요구하는가? | **Average Model Calls to Commit Execution Route** |

Reviewer-facing summary:

```text
QA-01 = 빠른가?
QA-02 = 정확한가?
QA-03 = 변경이 잘 격리되는가?
QA-04 = 실행 경로를 정하기 위해 AI 판단을 얼마나 요구하는가?
```

These are intentionally different views of the same DP-00 alternatives:

- observed response time;
- architectural correctness;
- change containment;
- structural dependency on AI decision stages.

No single score is intended to replace the others.

# 9.2 QA Independence Review

## QA-01 vs QA-04

Model-call count can correlate with latency, but the metrics are not equivalent.

Example:

```text
Architecture A
  large model: 1 route-decision call
  latency: 900 ms

Architecture B
  two smaller/parallel route-decision calls
  latency: 400 ms
```

Possible outcome:

```text
QA-01: B better
QA-04: A better
```

The dimensions are therefore distinct:

```text
QA-01 = observed user-perceived time to useful outcome
QA-04 = structural AI decision dependency count to commit execution route
```

QA-04 intentionally does not claim that one logical model call has the same physical cost as another. Physical latency/compute remains Secondary telemetry.

## QA-01 vs QA-02

A very fast wrong route is not a good architecture outcome.

QA-01 measures responsiveness **only over successful Fast-task episodes**. QA-02 separately evaluates whether architecture constraints were satisfied.

Failure is not converted into a fake latency penalty because that would mix the concerns and introduce an arbitrary constant.

## QA-01 vs QA-03

A low-latency direct-coupled architecture can be difficult to evolve, while an adapter-rich architecture can be slower but contain change well.

Therefore:

```text
빠른 구조 != 반드시 변경하기 쉬운 구조
```

QA-01 and QA-03 form an explicit performance-vs-flexibility trade-off.

## QA-02 vs QA-03

Correctness evaluates runtime architecture behavior under a fixed design; Flexibility evaluates how that design reacts to evolution.

A tightly coupled implementation can be correct today but require widespread changes tomorrow. Conversely, a flexible architecture can still make a wrong task-association or routing decision.

The metrics are independent.

## QA-02 vs QA-04

A deterministic 0-call selector can be structurally efficient but wrong. A multi-call validator can be correct but expensive in route-decision generations.

Therefore QA-04 is not interpreted without QA-02.

## QA-03 vs QA-04

Fusing decision stages can reduce logical calls while increasing coupling; decomposing responsibilities behind stable contracts can improve change containment while increasing route-decision stages.

This is another intended trade-off rather than metric duplication.

## Independence verdict

The four QAs are **independent but trade-off-forming**.

They may correlate on some workloads, but each can produce a different ordering of alternatives because each measures a different architectural property.

# 9.3 DP-00 Sensitivity Review

The matrix below is a **structural-impact hypothesis**, not a measured result.

| DP-00 structural change | QA-01 sensitivity hypothesis | QA-02 sensitivity hypothesis | QA-03 sensitivity hypothesis | QA-04 sensitivity hypothesis |
| --- | --- | --- | --- | --- |
| More synchronous processing stages | Can increase FTOL | Can improve validation if stages add checks; can also add failure surfaces | Can improve separation or add coupling depending on boundaries | Can increase calls when stages are model-based |
| Intent/Router split | May add latency | Can make validation/routing explicit | Can isolate routing changes if contracts are stable | May add logical calls vs fused reasoning |
| ARGO direct coupling | Can shorten dominant path | Correctness depends on ARGO ownership/state semantics | Can increase ARGO-centric coupling | Can reduce calls when decisions are fused |
| Bounded VIA Fast Path | Can strongly improve local-task FTOL | Creates false-fast eligibility risk | Can expand local boundary and evolution surface | Can be 0-call if eligibility is deterministic |
| Per-turn path selector | Can optimize path but adds selection overhead | Adds path-selection correctness responsibility | Adds selector/ownership-transfer evolution surface | Adds a call if selector is model-based |
| Generic Agent abstraction/adapter | Can add dispatch boundary overhead | Can make ownership/contracts explicit | Expected to contain Agent evolution if seam is stable | Usually neutral unless model-based mediation is added |
| Central task/context ownership | Can add state/serialization work | Can improve follow-up/result binding consistency | Can either stabilize or centralize coupling | Can add calls only if state association uses models |
| Model-based validation | Adds inference latency | Can recover from bad semantic proposals | Adds model/prompt evolution dependency | Adds ORCHESTRATION call(s) before route commit |

The purpose of this matrix is to verify **construct sensitivity**: the selected QAs should react when DP-00 changes architecture structure.

It does not predict final scores reliably because actual prototypes may fuse, parallelize, cache or implement deterministic alternatives differently.

# 9.4 A/B/C/D Pre-experiment Hypotheses

Every statement in this section is:

- **Hypothesis**;
- an **Expected tendency**;
- **Not a measured result**.

No alternative is considered the winner before implementation and controlled evaluation.

## A — Thin VIA

**Hypothesis / expected tendency — not measured result**

- QA-01: may be disadvantaged by extra intent/routing/dispatch stages.
- QA-02: may benefit from explicit validation, task-state and Agent boundary ownership.
- QA-03: may benefit from adapter/common-contract containment.
- QA-04: may be disadvantaged if intent, routing and validation are separate model generations.

Counterexamples are possible: deterministic routing can reduce QA-04; parallel processing can reduce QA-01; an unstable common contract can hurt QA-03.

## B — ARGO-centric Primary Execution

**Hypothesis / expected tendency — not measured result**

- QA-01: may benefit from a shorter dominant ARGO-native path.
- QA-02: may carry risk if fused ownership/state semantics make validation, follow-up or result binding less explicit.
- QA-03: may be disadvantaged if ARGO becomes a strongly coupled architectural center.
- QA-04: may benefit from fused reasoning/route decisions.

The QA-04 Execution Route Commit amendment ensures that specialist-delegation inference inside ARGO is still counted; B is not given free credit for hiding routing after an intermediate start event.

## C — Hybrid VIA Fast Path

**Hypothesis / expected tendency — not measured result**

- QA-01: may perform well for bounded local tasks.
- QA-02: false-fast eligibility is a key correctness risk.
- QA-03: fast-path capability growth may expand VIA's change surface and boundary complexity.
- QA-04: may perform well when eligibility/local selection is deterministic.

The fast-path allow-list/eligibility semantics must be governed so C does not simply move more and more domain logic into VIA.

## D — Adaptive Per-turn Execution

**Hypothesis / expected tendency — not measured result**

- QA-01: may optimize routes per request but pays selector overhead.
- QA-02: execution-path selection correctness is a primary risk.
- QA-03: selector and ownership-transfer complexity may reduce change containment.
- QA-04: a model-based selector can add call overhead; deterministic selection can avoid it.

D's flexibility in runtime routing does not imply QA-03 Flexibility automatically; runtime adaptiveness and code-change containment are different properties.

# 9.5 Benchmark Neutrality Review

The benchmark must not define the preferred architecture as the correct answer.

## QA-02 path neutrality

A scenario does **not** set one exact execution path merely because one alternative contains that path.

Use:

```text
Required
Allowed
Forbidden
```

Architecture Constraint Manifest semantics.

Example:

```text
"볼륨 줄여줘"

Allowed execution owner:
- VIA_FAST
- ARGO
```

If both routes are functionally/policy-correct, QA-02 gives both correctness credit and QA-01/QA-04 expose their performance/decision-overhead differences.

Constraint evidence must come from:

- functional requirement;
- policy/security rule;
- capability contract;
- scenario semantics;
- task/context truth.

Forbidden reasoning:

```text
"Alternative D has a Fast Path, therefore FAST is the expected correct path."
```

Topology-derived expected answers are not valid correctness oracles.

## Common comparison boundary

A/B/C/D are compared at the Integrated Product user-goal boundary. Internal component names do not have to match.

QA-02 normalizes runtime architecture outcomes into a Canonical Architecture Decision Trace. QA-03 normalizes evolution boundaries into common architecture roles. QA-04 normalizes routing/delegation accounting with Execution Route Commit.

These three normalization mechanisms serve the same fairness goal without forcing internal structural equivalence.

# 9.6 QA-03 Gaming Prevention

Potential attack:

> “If an alternative defines Expected Change Area broadly enough, can it get CCR = 100%?”

Yes, if the boundary is allowed to be defined after seeing the implementation diff. Therefore that is prohibited.

Expected Change Area is defined first at **common architecture-role level**, not by alternative-specific file/component names.

Required order:

```text
Evolution Scenario
    ↓
Common Expected Change Roles
    ↓
A/B/C/D Role Mapping
    ↓
freeze role boundary + mapping
    ↓
Implementation
    ↓
Git Diff / Acceptance / Regression result
```

The role vocabulary, Expected Change Area and per-alternative mapping are frozen before result observation.

If the mapping must be corrected later, it creates a new scenario/benchmark version. Existing result evidence is not retroactively relabeled.

This keeps CCR falsifiable.

# 9.7 QA-01 Gaming / Confounder Prevention

Two failure modes require explicit controls.

## Failure mode 1 — fake failure latency

Do not assign an arbitrary huge latency to failed tasks.

Why:

- the chosen penalty constant would dominate the score;
- correctness and responsiveness would be mixed;
- a failure could be counted differently merely by changing the penalty value.

QA-01 remains success-only for FTOL. QA-02 captures correctness, and final selection can apply a separate correctness gate.

## Failure mode 2 — dependency fixture dominates FTOL p95

Example:

```text
F1 dependency fixture      50 ms
F2 dependency fixture     200 ms
F3 dependency fixture     800 ms
```

If all alternatives have similar architecture overhead, overall p95 may be mechanically controlled by F3's fixed external delay.

Architecture Qualification therefore requires:

```text
Equality:
  same controlled external/downstream behavior for comparable alternatives

Non-dominance:
  fixture latency calibrated so external dependency class does not overwhelm
  the architecture-induced FTOL differences under study
```

Exact deterministic fixture values are TBD and are calibrated in Pilot.

Actual production Agent/network/tool latency belongs in the separate fidelity/real-stack track.

# 9.8 Correctness Qualification Gate

A strong QA-01, QA-03 or QA-04 score cannot make an architecturally incorrect alternative acceptable.

Recommended final-selection rule:

```text
QA-02 AECR < minimum acceptable correctness threshold
    -> alternative is not eligible for final DP-00 selection
```

The numeric threshold is **TBD**.

Required process:

```text
Pilot
  -> inspect AECR metric behavior
  -> calibrate minimum acceptable correctness gate
  -> freeze gate rule/version
  -> final A/B/C/D evaluation
```

This is a **qualification gate**, not a hidden correctness penalty inside QA-01 or QA-04.

The separation preserves QA independence:

- QA-01 still means responsiveness;
- QA-04 still means route-decision model-call dependency;
- QA-02 decides whether the alternative is correct enough to remain eligible.

The gate must be frozen before final comparative results are known.

# 9.9 Scored QAs vs Must-pass Architecture Constraints

The Top 4 are the scored Architectural Drivers for DP-00 trade-off analysis.

That does **not** mean security, privacy, cancellation, reliability or recovery are unimportant.

Some concerns should not be treated as compensable score dimensions.

```text
Scored Architectural Drivers
  QA-01 Fast-task responsiveness
  QA-02 Interaction-orchestration correctness
  QA-03 Change flexibility
  QA-04 Model-call overhead

Must-pass Architecture Constraints / Gates
  Security
  Privacy / Context-sharing policy
  Required cancellation semantics
  Required task-state integrity
  Failure containment
  Mandatory recovery behavior
  Trusted boundary requirements
```

A mandatory constraint violation cannot be compensated by excellent stars in another QA.

The rationale is:

> Security/trust/recovery concerns are not excluded because they are less important. They are separated because selected requirements may be **mandatory qualification conditions rather than trade-off attributes**.

The exact must-pass constraint set and executable gates remain to be traced from the Approved Baseline during central vNext rebaseline/benchmark design.

# 9.10 Final Cross-review Verdict

| Review item | Verdict | Meaning |
| --- | --- | --- |
| QA Selection | **PASS** | Four concerns cover the intended DP-00 top trade-off dimensions. |
| Primary Metric Singularity | **PASS** | Each QA has one Primary Metric for 0–5 scoring. |
| QA Independence | **PASS** | Metrics are distinct although intentionally trade-off-forming. |
| DP-00 Structural Sensitivity | **PASS** | Each metric is expected to react to meaningful execution-topology changes. |
| Benchmark Neutrality | **PASS with controls** | Requires topology-neutral QA-02 constraints, role-normalized QA-03, route-normalized QA-04. |
| Architecture Isolation | **PASS with controlled replay/stubs/profiles** | Confounders must remain frozen/controlled. |
| Testability | **PASS** | Required observations have explicit schemas/boundaries and can be instrumented. |
| Reviewer Defensibility | **PASS after QA-01 and QA-04 amendments** | Dependency-latency non-dominance and Execution Route Commit close the two identified design gaps. |

These PASS judgments mean **evaluation-design readiness**, not that any architecture alternative has passed its future benchmark.

No A/B/C/D result has been measured in this checkpoint.

## Cross-review controls to carry forward

Before final DP-00 evaluation, freeze/version at least:

- QA-01 deterministic dependency-latency profile and non-dominance calibration;
- QA-01 score thresholds;
- QA-02 taxonomy/corpus/constraint manifests;
- QA-02 score thresholds and minimum correctness gate;
- QA-03 evolution taxonomy, Expected Change Roles and alternative mappings;
- QA-03 score thresholds;
- QA-04 workload taxonomy/population;
- QA-04 episode mean vs class macro-average aggregation rule;
- QA-04 model/prompt/cache profiles;
- QA-04 score thresholds;
- mandatory qualification constraints/gates;
- benchmark and scoring versions.

Required sequence remains:

```text
Pilot
  -> calibration
  -> scoring/gate/benchmark rule freeze
  -> final A/B/C/D evaluation
```

## Relationship to central rebaseline

This cross-review intentionally does not perform the central QA numbering/traceability/requirements migration.

After this checkpoint, a separate central vNext rebaseline can align:

- DP-00;
- QA-01~04 definitions;
- QA↔DP traceability;
- evaluation strategy;
- working requirements;
- must-pass architecture constraints.

`requirements-v1.1.md` remains unchanged as the Approved Baseline.
