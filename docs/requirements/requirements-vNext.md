# VIA Architecture Requirements — vNext Working Baseline

> Status: **Architecture Evaluation Working Baseline — not an Approved Baseline**
>
> `docs/requirements/requirements-v1.1.md` remains the **Approved starting baseline** and must not be edited by this vNext work.

## 1. Purpose

This document records the central vNext working baseline needed to evaluate `DP-00 — VIA Primary Execution Boundary` and its Top Architectural Drivers.

It does **not** replace or retroactively modify v1.1. It separates three things that must remain distinguishable during architecture evaluation:

```text
Approved Requirement Baseline
    = requirements-v1.1.md

Architecture Decision under evaluation
    = DP-00 and its A/B/C/D alternatives

Evaluation hypotheses / working rules
    = QA-01~04, benchmark controls, pre-experiment hypotheses
```

No DP-00 winner, v1.2 requirement change, score, or threshold is declared here.

## 2. Central vNext storyline

The vNext architecture evaluation follows this chain:

```text
Product / Architecture Concern
    ↓
DP-00 — VIA Primary Execution Boundary
    ↓
A / B / C / D Architecture Alternatives
    ↓
Top Architectural Drivers
    QA-01 ~ QA-04
    ↓
One Primary Metric per QA
    ↓
Controlled Architecture Qualification
    ↓
Pilot / Calibration / Scoring & Gate Freeze
    ↓
Final A/B/C/D Evaluation
    ↓
Sensitivity / Threats-to-validity Analysis
    ↓
Trade-off Analysis
    ↓
ADR
    ↓
requirements-vNext responsibility/scope change if justified
    ↓
possible future v1.2 Approved Baseline candidate
```

The final architecture decision must be traceable back to frozen benchmark rules and immutable raw evidence rather than to a post-hoc preference.

## 3. Baseline status and responsibility boundary

### 3.1 v1.1 is the approved starting point

The Approved Baseline currently describes approximately this responsibility model:

```text
User
  ↓
VIA
  - Voice/Text interaction
  - Context grounding / connection
  - Intent refinement
  - Agent routing / delegation
  - Conversation / task interaction and lifecycle
  - Policy / consent / result interaction
  ↓
Downstream Agent
  - Domain reasoning
  - Planning
  - Tool selection / execution
  - Domain workflow completion
```

This remains the historical and approved starting boundary throughout DP-00 evaluation.

### 3.2 vNext does not assume that the v1.1 boundary is already optimal

DP-00 exists specifically to evaluate the responsibility boundary itself at the **Integrated Product** level.

Therefore vNext must not silently convert the current v1.1 boundary into a constraint that makes every alternative conform to it by construction.

The working rule is:

> **Final responsibility placement may change depending on the measured DP-00 trade-off.**

If the selected alternative preserves the v1.1 boundary, no responsibility-boundary change is required. If the selected alternative challenges that boundary, the corresponding requirement/scope changes are proposed in this vNext document only **after** the measured DP-00 decision and review.

`requirements-v1.1.md` is never rewritten retroactively.

## 4. DP-00 — VIA Primary Execution Boundary

### Reviewer-facing question

> **VIA는 사용자 요청을 어디까지 직접 판단·실행하고, 어디부터 Downstream Agent에 위임할 것인가?**

English short name: **VIA Primary Execution Boundary**

Authoritative Decision Point:

- `docs/architecture/decision-points/DP-00-primary-execution-boundary.md`
- rationale: `docs/architecture/analysis/AA-001-qa-rebaseline-and-primary-execution-boundary.md`

The comparison boundary is the **Integrated Product**. Every alternative must satisfy the same user-visible use cases and benchmark scenarios. The principal independent variable is the **SW placement/ownership of reasoning, execution, routing and orchestration capability**.

### 4.1 Alternative A — Thin VIA / Agent-neutral Orchestration

```text
User / Voice / Text
       ↓
VIA interaction + context + intent + routing + task orchestration
       ↓
Downstream Agent
       ↓
Domain reasoning / planning / tools / execution
```

VIA owns the interaction/orchestration boundary and delegates substantive execution to a Downstream Agent.

### 4.2 Alternative B — ARGO-centric Primary Execution

```text
Voice / S2S
    ↓
thin realtime context / interaction layer
    ↓
ARGO primary ReAct reasoning + tool-execution runtime
    ↓
Specialized / other Agent delegation when required
```

ARGO owns the first substantive reasoning/execution path and may delegate onward.

**This is intentionally a v1.1 Responsibility Boundary Challenging Alternative.**

It must not be weakened to “VIA keeps the current routing boundary but prefers ARGO as the default Downstream Agent.” That weaker form is a routing-policy variant of A or D, not the structurally distinct top-level B being evaluated by DP-00.

### 4.3 Alternative C — Hybrid VIA Fast Path

```text
                     ┌─ bounded VIA Fast Path
User → VIA eligibility
                     └─ Downstream Agent
```

VIA may directly own a deliberately narrow class of bounded, local-safe, latency-critical capabilities while delegating substantive work.

Fast Path eligibility is **semantic**, not defined by arbitrary implementation thresholds such as `<3 seconds` or `1 LLM + 1 tool`.

Candidate semantic properties include bounded execution, no domain planning, no durable Agent workflow/state requirement, simple recovery semantics, local-safe ownership, and measurable user-experience benefit.

### 4.4 Alternative D — Adaptive Per-turn Execution Topology

```text
                     ┌─ VIA Fast Path
User turn → selector ├─ ARGO path
                     └─ Specialized Downstream Agent
```

A common interaction/control plane selects an execution topology per user turn.

Directed escalation may be allowed, but arbitrary owner bouncing is not the intended design because repeated transfer complicates task identity, side-effect deduplication, cancellation, provenance, retry and recovery.

### 4.5 v1.1 Boundary Compatibility

| Alternative | v1.1 Boundary Compatibility |
| --- | --- |
| **A — Thin VIA** | **Compatible** |
| **B — ARGO-centric Primary Execution** | **Challenges baseline boundary** |
| **C — Hybrid VIA Fast Path** | **Mostly compatible / extension** |
| **D — Adaptive Per-turn** | **Partially compatible / extension likely** |

Compatibility is an analysis dimension, not a veto rule. A boundary-challenging alternative remains eligible for evaluation.

## 5. Top Architectural Driver QA set

The vNext scored Top Architectural Drivers are:

| QA | Reviewer-facing question | Primary Metric | Authoritative definition |
| --- | --- | --- | --- |
| **QA-01 Fast-task End-to-End Responsiveness** | **빠른가?** | **Fast-task Outcome Latency p95 (FTOL p95)** | `docs/evaluation/quality-attributes/QA-01-fast-task-e2e-responsiveness.md` |
| **QA-02 VIA Interaction-Orchestration Correctness** | **정확한가?** | **Architecture Episode Exact Conformance Rate (AECR)** | `docs/evaluation/quality-attributes/QA-02-via-interaction-orchestration-correctness.md` |
| **QA-03 변경 대응 용이성 (Flexibility)** | **변경이 잘 격리되는가?** | **Change Containment Rate (CCR)** | `docs/evaluation/quality-attributes/QA-03-change-flexibility.md` |
| **QA-04 모델 호출 오버헤드 (Model Call Overhead)** | **실행 경로를 정하기 위해 AI 판단을 얼마나 요구하는가?** | **Average Model Calls to Commit Execution Route** | `docs/evaluation/quality-attributes/QA-04-model-call-overhead.md` |

Exactly one Primary Metric determines each QA's 0–5 score. Secondary metrics remain diagnostic, backup, or sensitivity evidence.

The four QAs were cross-reviewed as an **independent but trade-off-forming** set in:

- `docs/architecture/analysis/AA-005-top-qa-cross-review.md`

The `DP-00 × QA Sensitivity Matrix` in AA-005 is a **pre-experiment architectural sensitivity hypothesis**, not a benchmark result.

## 6. Relationship to v1.1 Detailed QAs

The v1.1 QA set is preserved as historical Approved Baseline content. In vNext central navigation, those IDs are referred to as **Legacy v1.1 Detailed QA** to avoid collision with the new Top QA-01~04 IDs.

They are not deleted; their concerns are reclassified under Top Drivers, Secondary/Diagnostic concerns, or Mandatory Qualification Constraints.

| Legacy v1.1 Detailed QA | vNext role |
| --- | --- |
| **Legacy QA-01 — VIA software processing latency** | Supporting/Secondary diagnostic under **Top QA-01**; VIA-owned software overhead remains useful decomposition evidence. |
| **Legacy QA-02 — concurrent-task capacity & PC co-existence** | Detailed operational architecture QA / constraint; relevant to capacity, scheduling and foreground co-existence rather than one of the DP-00 Top-4 score dimensions. |
| **Legacy QA-03 — conversation/task recovery** | Reliability concern; expected to contribute to **Mandatory Qualification / recovery gates** and detailed lifecycle evaluation. |
| **Legacy QA-04 — dependency failure containment** | Reliability/Fault-Tolerance concern; expected to contribute to **Mandatory Qualification / failure-containment gates**. |
| **Legacy QA-05 — changeability & integration** | Detailed/Secondary evidence under **Top QA-03 Flexibility**. |
| **Legacy QA-06 — external-context minimization** | Privacy/Security supporting QA and likely **Mandatory Qualification** evidence. |
| **Legacy QA-07 — model/token cost efficiency** | Secondary/derived resource/cost evidence under **Top QA-04**; cost/token telemetry must not replace the architecture-only Primary Metric. |
| **Legacy QA-08 — observability & audit overhead** | Cross-cutting supporting QA for analysability/accountability and instrumentation overhead. |
| **Legacy QA-09 — screen/pointer referent-binding accuracy** | Secondary correctness slice under **Top QA-02**. |
| **Legacy QA-10 — user-request / existing-new task association correctness** | Secondary correctness slice under **Top QA-02**. |
| **Legacy QA-11 — Downstream Agent routing correctness** | Secondary correctness slice under **Top QA-02**. |

The authoritative wording and approved targets of these legacy QAs remain in `requirements-v1.1.md`.

## 7. Scored drivers vs Mandatory Qualification Constraints

The vNext evaluation distinguishes **trade-off scores** from **non-compensable acceptance conditions**.

```text
Scored Architectural Drivers
  QA-01 Responsiveness
  QA-02 Interaction-Orchestration Correctness
  QA-03 Flexibility
  QA-04 Model Call Overhead

Mandatory Qualification Constraints / Gates
  Security
  Privacy / Context-sharing policy
  Trusted boundary requirements
  Required cancellation semantics
  Required task-state integrity
  Failure containment
  Mandatory recovery behavior
```

Security and reliability are not absent because they are less important. Selected obligations are separated because violating them must not be compensated by high scores in unrelated QAs.

The exact executable gate set and numeric pass/fail criteria remain **TBD** until requirements traceability and Pilot/calibration are complete.

## 8. QA-02 minimum correctness eligibility gate

Final DP-00 selection is expected to distinguish **Score** from **Eligibility**.

Recommended rule:

```text
if QA-02 AECR < minimum acceptable correctness threshold:
    alternative is not eligible for final DP-00 selection
```

The minimum AECR threshold is **TBD**.

It must be calibrated and frozen before final A/B/C/D results are known.

Correctness is not folded into QA-01 or QA-04 by adding arbitrary penalties. QA independence is preserved while architectures below the minimum acceptable correctness level can be excluded from final selection.

## 9. Evaluation working rules

Architecture Qualification uses the methodology defined in:

- `docs/evaluation/evaluation-principles.md`
- `docs/evaluation/architecture-experiment-methodology.md`
- central strategy: `docs/evaluation/evaluation-strategy.md`

Core working rules:

```text
Independent Variable
  = SW Architecture Alternative

Controlled / Frozen as applicable
  = scenario semantics / corpus
  = semantic replay behavior
  = Agent / tool scripted behavior
  = model profile
  = prompt profile
  = cache policy
  = machine / environment
  = controlled dependency latency
```

Architecture Qualification uses deterministic semantic replay and deterministic/scripted Agent/tool behavior so stochastic dependency quality does not silently become the independent variable.

Real-model repeated runs are still used to construct realistic behavior classes and to perform a separate external-validity/fidelity track.

The replay corpus includes non-happy-path classes such as ambiguous, partially wrong, wrong-path/Agent proposal, malformed output and timeout/no-response behavior rather than only perfect model outputs.

This is a project-controlled evaluation methodology; no claim is made that it is a standardized “LLM architecture replay” benchmark.

## 10. QA-specific neutrality and anti-gaming controls

### QA-01

Controlled external/downstream latency must satisfy:

```text
Equality     = comparable alternatives receive the same controlled behavior
Non-dominance = the fixture itself does not mechanically dominate FTOL p95
```

Failed tasks are not assigned an arbitrary huge latency. Correctness is handled separately.

### QA-02

Correctness uses a topology-neutral **Required / Allowed / Forbidden** Architecture Constraint Manifest.

Constraints derive from functional requirements, policy, capability contracts, task/context truth and scenario semantics—not from the fact that one alternative contains a Fast Path, Router or ARGO-centric path.

### QA-03

Expected Change Area is frozen first at a **common architecture-role level**, followed by alternative-specific role→component/file mapping before implementation/result observation.

The boundary must not be enlarged after seeing the Git diff.

### QA-04

The authoritative end boundary is **Execution Route Commit**, not an intermediate owner-start event.

QA-04 counts ORCHESTRATION/MIXED logical model calls from request-processing start until the final domain execution route is committed. This prevents routing/delegation inference from disappearing merely because one topology moves it inside ARGO or another intermediate runtime.

Pure downstream DOMAIN reasoning after route commit is excluded.

## 11. Scoring, versioning and raw evidence

Required order:

```text
Pilot
  ↓
Calibration
  ↓
Scoring / Gate / Benchmark Rule Freeze
  ↓
Final A/B/C/D Evaluation
```

Do not tune thresholds, gate values, fixture profiles, taxonomy composition or aggregation rules after observing final alternative results merely to favor a preferred topology.

If a Primary Metric or scoring/gate definition changes:

- record the rationale;
- increment the relevant version;
- recompute all alternatives under the same rule where raw evidence permits;
- rerun only when required raw evidence was not captured.

Raw benchmark evidence is immutable.

## 12. Raw → Derived → Decision traceability

```text
Raw Evidence
    ↓
Derived Metric
    ↓
QA Primary Metric
    ↓
0–5 Score / Qualification Gate
    ↓
DP-00 Trade-off Analysis
    ↓
Architecture Decision / ADR
```

Relevant schemas:

- `benchmark/schemas/run-event-schema.md`
- `benchmark/schemas/scenario-constraint-schema.md`
- `benchmark/schemas/model-call-schema.md`
- `benchmark/schemas/evolution-scenario-schema.md`
- `benchmark/schemas/evolution-run-schema.md`

Raw, derived and report artifacts remain separated under `results/raw/`, `results/derived/`, and `results/reports/`.

## 13. Pending / TBD before final evaluation

The following are deliberately unresolved:

- DP-00 winning alternative;
- QA-01 0–5 thresholds;
- QA-01 deterministic dependency-latency profile values;
- QA-02 0–5 thresholds;
- QA-02 minimum correctness-gate value;
- QA-03 0–5 thresholds;
- QA-03 final evolution corpus/role vocabulary;
- QA-04 0–5 thresholds;
- QA-04 final workload taxonomy/class proportions;
- QA-04 overall episode mean vs class macro-average aggregation rule;
- final benchmark scenario corpora;
- final executable mandatory-gate definitions;
- production prototype results;
- final ADR;
- any resulting responsibility/scope change proposed for a future Approved Baseline.

## 14. Future requirement-change rule

No v1.1 requirement is changed merely because an alternative exists or is hypothesized to perform well.

If DP-00 measurement selects an alternative that requires responsibility/scope changes, especially a boundary-challenging alternative such as B, the sequence is:

```text
DP-00 measured result and trade-off
    ↓
ADR / architecture decision
    ↓
requirements-vNext responsibility/scope proposal
    ↓
review
    ↓
possible future v1.2 Approved Baseline candidate
```

Until then, v1.1 remains the Approved Baseline and vNext remains an architecture-evaluation working baseline.
