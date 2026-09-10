# Final Report Evidence Index

## Status

Living index for final SW Architecture review/report assembly.

This document does **not** create a new architecture decision, QA definition, requirement, or benchmark rule. It maps reviewer questions to the authoritative architecture/evaluation artifacts and, later, to raw/derived result evidence.

`docs/requirements/requirements-v1.1.md` remains the Approved Baseline.

## How to use this index

The target report-assembly chain is:

```text
Reviewer Question
    ↓
Central vNext Baseline / Architecture Reasoning
    ↓
Decision Point / QA Definition
    ↓
Benchmark Contract / Evaluation Control
    ↓
Raw Evidence
    ↓
Derived Metric / Score / Gate
    ↓
Trade-off
    ↓
Architecture Decision / ADR
```

Use links/paths from this index instead of copying full reasoning into multiple documents.

# 0. Central vNext Rebaseline Spine

## Reviewer questions

- 현재 architecture evaluation의 중앙 기준 문서는 무엇인가?
- v1.1과 vNext의 관계는 무엇인가?
- DP-00, Top QA, benchmark methodology가 어디에서 한 흐름으로 연결되는가?

## Primary evidence

- `docs/requirements/requirements-vNext.md`
  - v1.1 = Approved starting baseline
  - vNext = Architecture Evaluation Working Baseline
  - DP-00 boundary status
  - Top QA set
  - Legacy v1.1 QA reclassification
  - Scored QA vs Mandatory Qualification Gates
- `docs/architecture/qa-dp-traceability.md`
  - DP-00 top-level placement
  - DP-00 ↔ QA-01~04 mapping
  - DP-01~12 dependency relationship
  - Legacy QA disambiguation
  - raw/benchmark traceability
- `docs/evaluation/evaluation-strategy.md`
  - central controlled-evaluation pipeline
  - Architecture Qualification vs Actual-model/Real-stack Validation
  - Pilot/calibration/freeze discipline
  - QA-specific anti-gaming controls
  - Score vs Eligibility
  - Raw → Derived → Decision chain

## Report-ready central narrative

```text
Product / Architecture Concern
    ↓
DP-00 VIA Primary Execution Boundary
    ↓
A / B / C / D
    ↓
Top Architectural Drivers QA-01 ~ QA-04
    ↓
Controlled Architecture Qualification
    ↓
Pilot / Calibration / Rule Freeze
    ↓
Final Evaluation / Trade-off
    ↓
ADR
```

# 1. Problem / Architectural Concern

## Reviewer questions

- 왜 DP-00이 중요한가?
- 왜 기존 lower-level DP보다 execution responsibility boundary를 먼저 결정해야 하는가?
- VIA와 ARGO의 관계가 왜 top-level architecture issue인가?
- Thin VIA와 ARGO-centric topology의 핵심 tension은 무엇인가?

## Primary evidence

- `docs/architecture/analysis/AA-001-qa-rebaseline-and-primary-execution-boundary.md`
  - DP-00이 상위 Decision Point가 된 reasoning
  - v1.1을 시작 baseline으로 유지하면서 boundary 자체를 검증하는 이유
  - Alternative B를 baseline-compatible routing variant로 약화시키지 않은 이유
  - Integrated Product comparison boundary
- `docs/architecture/decision-points/DP-00-primary-execution-boundary.md`
  - 공식 DP 정의
  - A/B/C/D 구조와 responsibility placement
  - v1.1 boundary compatibility
- `docs/requirements/requirements-vNext.md`
  - central working-baseline treatment of the boundary

# 2. DP-00 Alternatives

## Reviewer questions

- 왜 A/B/C/D인가?
- 각 대안의 구조적 차이는 무엇인가?
- ARGO preferred route와 ARGO-centric Primary Execution은 왜 다른가?

## Authoritative evidence

- `docs/architecture/decision-points/DP-00-primary-execution-boundary.md`
- `docs/requirements/requirements-vNext.md` §4
- `docs/architecture/qa-dp-traceability.md` — DP-00 alternatives summary

## Report-ready extraction

| Alternative | Responsibility / topology summary | v1.1 compatibility |
| --- | --- | --- |
| **A — Thin VIA** | VIA interaction/context/intent/routing/task orchestration → Downstream Agent substantive execution | **Compatible** |
| **B — ARGO-centric Primary Execution** | Voice/S2S → thin realtime context → ARGO primary ReAct/tool runtime → specialist delegation | **Challenges baseline boundary** |
| **C — Hybrid VIA Fast Path** | bounded/local-safe capability in VIA; substantive work delegated | **Mostly compatible / extension** |
| **D — Adaptive Per-turn** | per-turn Fast / ARGO / Specialized Agent execution-topology selection | **Partially compatible / extension likely** |

Important defense point: “ARGO as preferred/default Downstream Agent” is an A/D routing-policy variant, **not** top-level Alternative B.

# 3. Top Architectural Drivers

## Reviewer-facing summary

| QA | Reviewer-facing question | Primary Metric |
| --- | --- | --- |
| **QA-01** | 빠른가? | **Fast-task Outcome Latency p95 (FTOL p95)** |
| **QA-02** | 정확한가? | **Architecture Episode Exact Conformance Rate (AECR)** |
| **QA-03** | 변경이 잘 격리되는가? | **Change Containment Rate (CCR)** |
| **QA-04** | 실행 경로를 정하기 위해 AI 판단을 얼마나 요구하는가? | **Average Model Calls to Commit Execution Route** |

## Primary evidence

- `docs/evaluation/quality-attributes/QA-01-fast-task-e2e-responsiveness.md`
- `docs/evaluation/quality-attributes/QA-02-via-interaction-orchestration-correctness.md`
- `docs/evaluation/quality-attributes/QA-03-change-flexibility.md`
- `docs/evaluation/quality-attributes/QA-04-model-call-overhead.md`
- `docs/architecture/analysis/AA-005-top-qa-cross-review.md`
- central summary: `docs/requirements/requirements-vNext.md` and `docs/architecture/qa-dp-traceability.md`

Use AA-005 for the conclusion that the four QAs are **independent but trade-off-forming**.

# 4. DP-00 × QA Sensitivity Matrix

## Reviewer questions

- 왜 이 4개 QA가 DP-00에 민감한가?
- 어떤 구조적 decision이 어느 QA를 자극하는가?
- QA를 결과에 맞춰 선택한 것은 아닌가?

## Primary evidence

- `docs/architecture/analysis/AA-005-top-qa-cross-review.md`
  - §9.3 **DP-00 × QA Sensitivity Matrix**
  - `●` sensitivity table
  - row-by-row rationale

## Report-use note

This is a **Pre-experiment architectural sensitivity hypothesis** recorded before final A/B/C/D benchmark results. The `●` count means sensitivity strength, not goodness/badness or measured score.

# 5. Pre-experiment A/B/C/D Hypothesis Material

## Reviewer questions

- 실험 전에 각 Alternative가 어느 QA에서 유리/불리할 것으로 예상했는가?
- 결과를 본 뒤 narrative를 바꾼 것은 아닌가?

## Primary evidence

- `docs/architecture/analysis/AA-005-top-qa-cross-review.md` §9.4

Every tendency is explicitly:

```text
Hypothesis
Expected tendency
Not a measured result
```

Keep this separate from final result tables so the report can compare **prediction vs observation** without rewriting history.

# 6. Evaluation Methodology

## Reviewer questions

- LLM randomness를 어떻게 통제했는가?
- architecture 외 변수가 결과를 지배하지 않는가?
- score threshold를 결과에 맞춰 정한 것은 아닌가?
- deterministic replay와 actual-model validation은 어떻게 구분되는가?

## Primary evidence

- central: `docs/evaluation/evaluation-strategy.md`
- detailed principles: `docs/evaluation/evaluation-principles.md`
- detailed method: `docs/evaluation/architecture-experiment-methodology.md`

## Report-ready principles

```text
Independent variable = SW Architecture Alternative
Dependent variables  = QA Primary Metrics
```

Controlled/frozen as applicable:

- scenario/corpus semantics
- deterministic semantic replay
- deterministic Agent/tool behavior
- model/prompt/cache profiles
- machine/environment
- controlled dependency latency

Method discipline:

```text
Pilot
  -> Calibration
  -> Scoring/Gate/Benchmark Freeze
  -> Final A/B/C/D Evaluation
```

Final results must not be used to retune thresholds or redefine the metric boundary.

# 7. Benchmark Neutrality / Anti-gaming Controls

## Reviewer question

- 평가가 특정 Alternative에 편향되지 않았는가?

### QA-01 — dependency-latency domination

Evidence:

- `QA-01-fast-task-e2e-responsiveness.md`
- AA-005 §9.7
- central `evaluation-strategy.md`

Controls:

```text
Equality:
  same controlled dependency behavior across comparable alternatives

Non-dominance:
  fixture latency does not mechanically dominate FTOL p95
```

Failed episodes do not receive arbitrary huge-latency penalties.

### QA-02 — topology-neutral correctness oracle

Evidence:

- `QA-02-via-interaction-orchestration-correctness.md`
- `AA-002-qa02-correctness-measurement-rationale.md`
- `benchmark/schemas/scenario-constraint-schema.md`

Oracle:

```text
Required
Allowed
Forbidden
```

Constraint sources: functional requirement, policy/security rule, capability contract, scenario semantics and task/context truth — **not evaluated topology**.

### QA-03 — Expected Change Area gaming prevention

Evidence:

- `QA-03-change-flexibility.md`
- `AA-003-qa03-flexibility-measurement-rationale.md`
- `benchmark/schemas/evolution-scenario-schema.md`
- `benchmark/schemas/evolution-run-schema.md`

Required sequence:

```text
Evolution Scenario
    -> Common Expected Change Roles
    -> A/B/C/D role mapping
    -> freeze
    -> implementation
    -> Git diff / acceptance / regression result
```

### QA-04 — hidden-routing gaming prevention

Evidence:

- `QA-04-model-call-overhead.md`
- `AA-004-qa04-model-call-overhead-rationale.md`
- `benchmark/schemas/model-call-schema.md`
- `benchmark/schemas/run-event-schema.md`

Control: **Execution Route Commit** is the authoritative end boundary. Routing/delegation inference remains measured even if it moves inside ARGO after an intermediate owner/start event.

Model/prompt/cache profiles are frozen so they do not silently become independent variables.

# 8. Legacy v1.1 QA Reclassification

## Reviewer questions

- 기존 QA-01~11은 왜 사라졌는가?
- 보안/복구/동시성은 누락된 것인가?

They did **not** disappear. vNext central navigation calls them **Legacy v1.1 Detailed QA** and reclassifies their roles.

Primary evidence:

- `docs/requirements/requirements-vNext.md` §6
- `docs/architecture/qa-dp-traceability.md` — Legacy QA mapping table
- authoritative original definitions: `docs/requirements/requirements-v1.1.md`

Report-use summary:

- Legacy QA-01 → Top QA-01 diagnostic
- Legacy QA-05 → Top QA-03 detail
- Legacy QA-07 → Top QA-04 resource/cost diagnostics
- Legacy QA-09/10/11 → Top QA-02 correctness slices
- Legacy QA-03/04/06 → reliability/privacy/security gate candidates
- Legacy QA-02/08 → detailed operational/cross-cutting supporting concerns

# 9. Scored QA vs Must-pass Gates

## Reviewer question

- 보안/신뢰성/복구는 왜 Top 4 점수에 없는가?

## Primary evidence

- `docs/architecture/analysis/AA-005-top-qa-cross-review.md` §9.8–9.9
- `docs/requirements/requirements-vNext.md` §7–8
- `docs/architecture/qa-dp-traceability.md`
- `docs/evaluation/evaluation-strategy.md`

Report-ready separation:

```text
Scored Architectural Drivers
  QA-01
  QA-02
  QA-03
  QA-04

Mandatory Qualification Gates
  Security
  Privacy / Context-sharing policy
  Trusted boundary requirements
  Required task-state integrity
  Required cancellation semantics
  Failure containment
  Mandatory recovery behavior
```

Rationale: security/trust/recovery are not considered less important. Selected obligations are **non-compensable conditions**, so they should not be traded away through a higher star score elsewhere.

Also preserve the separate QA-02 minimum correctness eligibility gate. Numeric threshold: **TBD until Pilot/calibration**.

# 10. Raw Data → Derived Metrics → Decision Traceability

## Reviewer questions

- 최종 별점은 어떤 raw evidence에서 계산되었는가?
- metric을 재계산할 수 있는가?
- raw 결과가 어떻게 architecture decision까지 연결되는가?

## Report-ready chain

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

Primary central evidence:

- `docs/evaluation/evaluation-strategy.md` §9
- `docs/architecture/qa-dp-traceability.md` — Evaluation and benchmark traceability

Runtime schemas:

- `benchmark/schemas/run-event-schema.md`
- `benchmark/schemas/scenario-constraint-schema.md`
- `benchmark/schemas/model-call-schema.md`

Evolution schemas:

- `benchmark/schemas/evolution-scenario-schema.md`
- `benchmark/schemas/evolution-run-schema.md`

Derived primary targets:

- QA-01 FTOL p95
- QA-02 AECR
- QA-03 CCR
- QA-04 Average Model Calls to Commit Execution Route

Storage policy:

```text
results/raw/      immutable source evidence
results/derived/  recomputable metric / score inputs
results/reports/  human-readable results and visualizations
```

# 11. Threats to Validity

This remains a living checklist for the final report.

| Threat | Why it matters | Existing control / evidence |
| --- | --- | --- |
| Replay fidelity | Frozen traces may stop representing real model behavior | separate Actual-model/Real-stack validation; behavior-class refresh |
| Stub vs real Agent behavior | deterministic stubs simplify real execution semantics | scripted qualification + separate real Agent validation |
| Scenario representativeness | narrow corpus can hide failure surfaces | coverage-balanced/versioned corpora and class/category slices |
| Workload weighting | p95/mean/rate can change with mix | freeze population/aggregation; usage-weighted sensitivity separately |
| Model-profile dependency | QA-04 and latency may shift by model | freeze profile/version; preserve model-call telemetry |
| Hardware dependency | runtime values depend on deployment | fixed/reference machine and environment metadata |
| Prompt/cache optimization | token/latency/resource values can change without topology change | freeze prompt/cache profiles; preserve telemetry |
| External dependency latency | fixture can dominate QA-01 | equality + non-dominance calibration |
| Prototype fidelity | simplified prototypes may misrepresent final product structure | comparable maturity/shared infrastructure + fidelity discussion |
| Oracle neutrality | correctness oracle may encode preferred topology | Required/Allowed/Forbidden from semantics/requirements |
| Evolution-boundary gaming | broad post-hoc Expected Change Area inflates CCR | common role-level boundary + pre-frozen mapping |
| Hidden-routing gaming | internal routing can escape old measurement boundary | Execution Route Commit |
| Threshold hindsight | scores can be tuned after results | Pilot → calibration → freeze → final |

Controls reduce threats; they do not prove external validity. Residual limitations and sensitivity results must be reported.

# 12. Final Trade-off / Decision Material

No final A/B/C/D benchmark result exists yet.

Future links/placeholders:

- executable A/B/C/D specifications: **TBD**
- prototypes: **TBD**
- A/B/C/D raw results: `results/raw/` — **TBD**
- derived QA metrics: `results/derived/` — **TBD**
- QA 0~5 score table: **TBD**
- correctness / mandatory-gate results: **TBD**
- visualization source data: **TBD**
- radar/spider or other trade-off visualization: **TBD**
- class/category sensitivity analysis: **TBD**
- Actual-model/Real-stack validation: **TBD**
- selected Alternative: **TBD**
- rejected Alternative rationale: **TBD**
- final DP-00 trade-off analysis: **TBD**
- ADR: `docs/adr/` — **TBD**

Target decision chain:

```text
Pre-experiment hypothesis
    -> frozen benchmark/scoring/gate rules
    -> immutable raw data
    -> derived QA metrics
    -> QA score + mandatory gates
    -> trade-off interpretation
    -> selected/rejected alternative rationale
    -> ADR
    -> requirements-vNext impact
    -> possible future Approved Baseline
```

# Reviewer Question → Evidence Map

| Reviewer question | Primary evidence |
| --- | --- |
| 왜 DP-00이 중요한가? | AA-001 + DP-00 + requirements-vNext §3–4 |
| 왜 A/B/C/D인가? | DP-00 + requirements-vNext §4 + central traceability |
| 왜 이 4개 QA인가? | AA-005 §9.1–9.3 + central Top-QA tables |
| 왜 이 Primary Metric인가? | QA-01~04 + AA-002/003/004 rationale records |
| 기존 QA-01~11은 어떻게 되었는가? | requirements-vNext §6 + qa-dp-traceability Legacy mapping |
| 평가가 특정 Alternative에 편향되지 않았는가? | AA-005 §9.5–9.7 + evaluation-strategy §6 + schemas |
| LLM randomness를 어떻게 통제했는가? | evaluation-strategy §5 + evaluation-principles + methodology |
| 점수 기준을 결과에 맞춰 정한 것은 아닌가? | evaluation-strategy §8 + versioning principles |
| 보안/신뢰성은 왜 Top 4에 없는가? | AA-005 §9.8–9.9 + requirements-vNext §7 + evaluation-strategy §7 |
| 실험 결과가 실제 제품에도 유효한가? | evaluation-strategy §11 + methodology validity threats + future real-stack validation |
| raw data가 score로 어떻게 연결되는가? | evaluation-strategy §9 + qa-dp-traceability + benchmark schemas |
| 최종 Alternative 선택의 trade-off 근거는 무엇인가? | future raw/derived/gates + AA-005 hypotheses + DP-00 + future ADR |

## Maintenance rule

Update this file whenever a new artifact becomes report-relevant, but keep the original artifact authoritative.

The index should answer:

> **어떤 심사 질문을 어떤 근거 문서와 실험 데이터가 뒷받침하는가?**

It must remain an index, not a duplicate architecture report.
