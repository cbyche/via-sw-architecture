# AA-005 — Top QA Cross-review

## Status

Architecture Analysis Record — Top Architectural Driver Cross-review Checkpoint

This record cross-reviews QA-01 through QA-04 as the scored Top Architectural Driver set for DP-00.

It is an **evaluation-design readiness review**, not an experiment result, not an ADR, and not a requirements-baseline change.

`docs/requirements/requirements-v1.1.md` remains the Approved Baseline and is not modified by this record.

## Purpose

QA-01~04 were initially defined one by one and then cross-reviewed as a set before the Central vNext Rebaseline. The central rebaseline is now complete; this record remains the authoritative pre-experiment rationale for why these four QAs form a coherent DP-00 driver set.

The cross-review asks:

- Do the four QAs measure distinct architecture concerns rather than the same signal four times?
- Are they sensitive to the actual structural differences in DP-00 A/B/C/D?
- Can they compare structurally different alternatives without making one topology the oracle?
- Are measurement boundaries resistant to obvious metric gaming?
- Can stochastic model/Agent behavior and fixture behavior be controlled well enough to isolate architecture effects?
- Are important security/reliability obligations accidentally being treated as optional because they are not in the scored Top 4?

Two amendments were required by the initial review:

1. QA-01 Architecture Qualification must control external/dependency latency for both **equality and non-dominance**.
2. QA-04 must end at **Execution Route Commit**, not merely at intermediate owner confirmation/execution start.

The later executable-architecture checkpoint adds one further interpretation rule without changing the sensitivity levels:

> **Component/stage separation is not model-call count. QA-04 is sensitive only when the structural responsibility requires additional logical Generative AI generations before Execution Route Commit.**

With these controls, the Top QA set is considered ready for executable Base Architecture and Pilot preparation.

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
- structural dependency on Generative AI decision stages.

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
QA-04 = structural Generative AI decision dependency count to commit execution route
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

Fusing decision stages can reduce logical calls while increasing coupling; decomposing responsibilities behind stable contracts can improve change containment while increasing route-decision stages **only when those stages actually require separate Generative AI decisions**.

This is another intended trade-off rather than metric duplication.

## Independence verdict

The four QAs are **independent but trade-off-forming**.

They may correlate on some workloads, but each can produce a different ordering of alternatives because each measures a different architectural property.

# 9.3 DP-00 × QA Sensitivity Matrix

## Purpose and interpretation

This matrix records a **Pre-experiment architectural sensitivity hypothesis**: before A/B/C/D results are measured, which Top QA is expected to be most sensitive to each major DP-00 structural design choice?

This is report-ready analysis material intended to explain why QA-01~04 were selected as Top Architectural Drivers. It was deliberately recorded before final benchmark results so later scoring cannot be used to retroactively justify the QA set.

Important interpretation rules:

- The number of `●` symbols does **not** mean good or bad.
- It indicates expected **sensitivity / strength of architectural influence**.
- It is **not** an A/B/C/D experiment result.
- It does **not** predict which alternative will win.
- Component count or stage count is **not** automatically Model Call count.
- Actual effects must be established by controlled benchmark evidence.

Legend:

```text
○    = 영향이 작거나 간접적
●    = 영향 있음
●●   = 중간 수준의 sensitivity
●●●  = 주요 sensitivity point
```

| 구조적 차이 | QA-01 Responsiveness | QA-02 Correctness | QA-03 Flexibility | QA-04 Model Call Overhead |
| --- | :---: | :---: | :---: | :---: |
| synchronous processing stage 수 | ●●● | ● | ○ | ●●● |
| Intent/Router 분리 여부 | ●● | ●● | ●● | ●●● |
| ARGO direct coupling | ●●● | ●● | ●●● | ●●● |
| bounded Fast Path | ●●● | ●●● | ●● | ●●● |
| per-turn selector | ●● | ●●● | ●●● | ●●● |
| Agent abstraction / adapter | ● | ● | ●●● | ● |
| task / context ownership | ● | ●●● | ●● | ● |
| model validation layer | ●● | ●●● | ● | ●●● |

## Row rationale

### synchronous processing stage 수

**QA-01 ●●●:** each mandatory synchronous stage can extend the critical path from user input to useful outcome, so responsiveness is highly sensitive to stage count and serialization. **QA-04 ●●●:** sensitivity is high when additional architectural stages introduce additional logical **Generative AI inference dependencies** before Execution Route Commit. A deterministic state/contract/policy stage can increase code-path length without increasing QA-04 at all. Therefore the `●●●` rating reflects the risk of model-based sequential decision staging, not a claim that every component or synchronous software stage equals one Model Call. Correctness can be affected by added checks, but stage count alone does not guarantee correctness; Flexibility impact is indirect unless stages imply new coupling boundaries.

### Intent/Router 분리 여부

**QA-04 ●●●:** separating intent and routing into independent semantic responsibilities has high sensitivity **when each responsibility requires its own sequential Generative AI decision** before Execution Route Commit. Simple component/code separation does not itself increase Model Calls. For example, `Intent Refiner = GENAI_DEDICATED` plus `Agent Router = deterministic capability/contract lookup` can still produce only one Generative Model Call. **QA-01/QA-02/QA-03 ●●:** the responsibility split can add latency, make validation/routing boundaries explicit, and isolate routing changes, but the observed effect depends on decision mechanisms and coupling. Cross-component intent+routing GenAI fusion is treated as an optimization tactic in the executable Base experiment rather than silently changing this architectural sensitivity hypothesis.

### ARGO direct coupling

**QA-01 ●●●:** direct coupling can remove VIA-side orchestration/dispatch hops on ARGO-native paths. **QA-03 ●●●:** making ARGO structurally central can cause Agent/context/contract evolution to propagate through ARGO-specific integration. **QA-04 ●●●:** ARGO may structurally co-locate request interpretation and self-vs-specialist initial routing in one primary runtime generation, while specialist delegation still counts until Execution Route Commit. QA-02 remains materially sensitive because task ownership, clarification, validation, and result binding may move or become fused.

### bounded Fast Path

**QA-01 ●●●:** bypassing normal orchestration/delegation can directly reduce user-visible latency for eligible bounded requests. **QA-02 ●●●:** eligibility mistakes can become false-fast execution, making correctness highly sensitive to the boundary and validator. **QA-04 ●●●:** deterministic eligibility and local selection can reach route commit with zero additional Generative Model Calls, while semantic/model-based eligibility can add calls. QA-03 is also affected because expanding the fast-path capability set expands the VIA-owned change surface.

### per-turn selector

**QA-02 ●●●:** selecting the wrong topology or owner per turn is itself a new correctness failure mode. **QA-03 ●●●:** selector policy, owner-transfer contracts, and per-path integration create an additional evolution surface. **QA-04 ●●●:** a GenAI-based selector directly adds a route-decision generation; a deterministic selector would not. QA-01 remains moderately sensitive because selector overhead can be offset by choosing a faster downstream path.

### Agent abstraction / adapter

**QA-03 ●●●:** the Agent integration seam is a principal extension boundary for adding/replacing Agents, so it directly determines whether evolution stays contained. QA-01, QA-02, and QA-04 usually see weaker direct sensitivity: an abstraction can add small dispatch overhead or make contracts explicit, but by itself does not require extra Generative Model Calls.

### task / context ownership

**QA-02 ●●●:** ownership strongly influences follow-up association, concurrent result binding, clarification continuity, and stale-context handling. **QA-03 ●●:** central ownership can stabilize shared contracts or create a coupling hub that changes must cross. QA-01/QA-04 usually experience only indirect impact unless ownership logic introduces synchronous Generative AI decisions.

### model validation layer

**QA-02 ●●●:** validation can reject wrong Fast Path/Agent/referent proposals and trigger clarification or safe fallback, so correctness is highly sensitive. **QA-04 ●●●:** a **GenAI-based** validation layer directly adds an ORCHESTRATION generation before Execution Route Commit; deterministic contract/schema validation does not. **QA-01 ●●:** the same validation can add latency, though deterministic checks, parallelization, or later tactics may reduce that cost. QA-03 impact is generally lower unless validation becomes a broad model/prompt dependency across the architecture.

## Matrix conclusion

The four QAs do not repeatedly measure the same structural characteristic. They create distinct architecture pressures:

```text
QA-01 -> Latency pressure
QA-02 -> Correctness pressure
QA-03 -> Flexibility / containment pressure
QA-04 -> Generative-AI-decision structural overhead pressure
```

Therefore a topology optimized in one direction does not automatically become best on all four QAs. For example, semantic-stage fusion may improve QA-01/QA-04 while weakening QA-03, and added model-based validation may improve QA-02 while increasing QA-01/QA-04 cost.

This is precisely the behavior expected from a useful Top Architectural Driver set: the QAs expose the trade-offs created by DP-00 rather than collapsing them into one preferred architecture pattern.

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
- QA-04: may be disadvantaged **if** intent, routing and validation require separate Generative AI generations; deterministic stages do not add calls merely because they are separate components.

Counterexamples are possible: deterministic routing can reduce QA-04; parallel processing can reduce QA-01; an unstable common contract can hurt QA-03.

## B — ARGO-centric Primary Execution

**Hypothesis / expected tendency — not measured result**

- QA-01: may benefit from a shorter dominant ARGO-native path.
- QA-02: may carry risk if fused ownership/state semantics make validation, follow-up or result binding less explicit.
- QA-03: may be disadvantaged if ARGO becomes a strongly coupled architectural center.
- QA-04: may benefit because request interpretation and initial self-vs-specialist execution choice are structurally co-located in ARGO.

The QA-04 Execution Route Commit amendment ensures that specialist-delegation inference inside ARGO is still counted; B is not given free credit for hiding routing after an intermediate start event.

## C — Hybrid VIA Fast Path

**Hypothesis / expected tendency — not measured result**

- QA-01: may perform well for bounded local tasks.
- QA-02: false-fast eligibility is a key correctness risk.
- QA-03: fast-path capability growth may expand VIA's change surface and boundary complexity.
- QA-04: may perform well when eligibility/local selection is deterministic.

The fast-path capability contract/eligibility semantics must be governed so C does not simply move more and more domain logic into VIA.

## D — Adaptive Per-turn Execution

**Hypothesis / expected tendency — not measured result**

- QA-01: may optimize routes per request but pays selector overhead.
- QA-02: execution-path selection correctness is a primary risk.
- QA-03: selector and ownership-transfer complexity may reduce change containment.
- QA-04: a Generative-AI-based selector can add call overhead; component existence alone is not a call.

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

The executable Base Architecture checkpoint additionally separates architecture responsibility placement from optional optimization tactics so one alternative is not benchmarked with hidden fusion/classifier/speculation advantages.

These normalization mechanisms serve the same fairness goal without forcing internal structural equivalence.

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
- QA-04 still means route-decision Generative Model Call dependency;
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

The exact must-pass constraint set and executable gates remain to be traced/frozen during benchmark-contract and Pilot preparation.

# 9.10 Final Cross-review Verdict

| Review item | Verdict | Meaning |
| --- | --- | --- |
| QA Selection | **PASS** | Four concerns cover the intended DP-00 top trade-off dimensions. |
| Primary Metric Singularity | **PASS** | Each QA has one Primary Metric for 0–5 scoring. |
| QA Independence | **PASS** | Metrics are distinct although intentionally trade-off-forming. |
| DP-00 Structural Sensitivity | **PASS** | Each metric is expected to react to meaningful execution-topology changes. |
| Benchmark Neutrality | **PASS with controls** | Requires topology-neutral QA-02 constraints, role-normalized QA-03, route-normalized QA-04, and Base-vs-Tactic separation. |
| Architecture Isolation | **PASS with controlled replay/stubs/profiles** | Confounders must remain frozen/controlled. |
| Testability | **PASS** | Required observations have explicit schemas/boundaries and can be instrumented. |
| Reviewer Defensibility | **PASS after QA-01/QA-04 and executable-Base clarifications** | Dependency-latency non-dominance, Execution Route Commit, and component-vs-model-call clarification close identified design gaps. |

These PASS judgments mean **evaluation-design readiness**, not that any architecture alternative has passed its future benchmark.

No A/B/C/D benchmark result has been measured in this checkpoint.

## Cross-review controls to carry forward

Before final DP-00 evaluation, freeze/version at least:

- DP-00 executable Base Architecture version;
- common-vs-variable experimental boundary;
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
Executable Base Specification
  -> Canonical Benchmark Contract
  -> Prototype
  -> Pilot
  -> calibration
  -> scoring/gate/benchmark rule freeze
  -> final A/B/C/D evaluation
```

## Relationship to central rebaseline and executable specification

The Central vNext Rebaseline is complete and is reflected in:

- `docs/requirements/requirements-vNext.md`;
- `docs/architecture/qa-dp-traceability.md`;
- `docs/evaluation/evaluation-strategy.md`.

The DP-00 executable Base definition is maintained in:

- `docs/architecture/decision-points/DP-00-executable-architecture-spec.md`;
- `docs/architecture/analysis/AA-006-dp00-executable-walkthrough.md`;
- `docs/evaluation/base-architecture-vs-tactic-evaluation.md`.

`requirements-v1.1.md` remains unchanged as the Approved Baseline.