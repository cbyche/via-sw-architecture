# Final Report Evidence Index

## Status

Living index for final SW Architecture review/report assembly.

This document does **not** create a new architecture decision, QA definition, or benchmark rule. Its purpose is to make existing reasoning, methodology, benchmark contracts, and future results easy to locate from reviewer-facing questions.

`docs/requirements/requirements-v1.1.md` remains the Approved Baseline and is not modified by this index.

## How to use this index

The intended report assembly flow is:

```text
Reviewer question
    -> architecture reasoning / QA definition
    -> benchmark contract / control
    -> raw evidence
    -> derived metric
    -> QA score / trade-off
    -> architecture decision / ADR
```

When future experiments/results are added, update this index with links/paths rather than duplicating the full content here.

# 1. Problem / Architectural Concern

## Reviewer questions

- 왜 DP-00이 중요한가?
- 왜 기존 lower-level DP보다 execution responsibility boundary를 먼저 결정해야 하는가?
- VIA와 ARGO의 관계가 왜 top-level architecture issue인가?
- Thin VIA와 ARGO-centric topology의 핵심 tension은 무엇인가?

## Primary evidence

- `docs/architecture/analysis/AA-001-qa-rebaseline-and-primary-execution-boundary.md`
  - DP-00이 상위 Decision Point로 승격된 reasoning
  - Approved Baseline을 시작점으로 유지하면서 responsibility boundary 자체를 검증하는 이유
  - Alternative B를 baseline-compatible variant로 약화시키지 않은 이유
  - Integrated Product를 비교 boundary로 잡은 이유
- `docs/architecture/decision-points/DP-00-primary-execution-boundary.md`
  - 공식 DP 정의
  - A/B/C/D 구조
  - responsibility ownership
  - v1.1 boundary compatibility
  - expected QA trade-offs

## Report-ready material

- AA-001: “왜 기존 Interaction Context DP보다 DP-00이 먼저인가” reasoning
- DP-00: A/B/C/D 구조 및 boundary compatibility 비교

# 2. DP-00 Alternatives

## Reviewer questions

- 왜 A/B/C/D인가?
- 각 대안은 실제로 무엇이 다른가?
- ARGO를 preferred default Agent로 두는 것과 ARGO-centric execution은 왜 다른가?

## Primary evidence

`docs/architecture/decision-points/DP-00-primary-execution-boundary.md`

Report extraction points:

### A — Thin VIA

- 구조: VIA가 interaction/context/intent/routing/task lifecycle을 소유하고 substantive execution은 Downstream Agent에 위임
- 핵심 책임 배치: VIA orchestration / Agent domain execution
- v1.1 compatibility: **Compatible**

### B — ARGO-centric Primary Execution

- 구조: Voice/S2S → thin realtime context → ARGO primary ReAct reasoning/tool execution → optional specialist delegation
- 핵심 책임 배치: ARGO가 primary execution authority
- v1.1 compatibility: **Challenges baseline boundary**
- 중요: “ARGO를 preferred/default Downstream Agent로 route”하는 것은 top-level B가 아니라 A/D routing-policy variant

### C — Hybrid VIA Fast Path

- 구조: bounded/local-safe capability는 VIA Fast Path, 나머지는 Agent
- 핵심 책임 배치: explicit bounded local execution + downstream substantive execution
- v1.1 compatibility: **Mostly compatible / extension**

### D — Adaptive Per-turn Execution

- 구조: turn별로 VIA Fast / ARGO / Specialized Agent topology 선택
- 핵심 책임 배치: per-turn selection + explicit ownership transfer
- v1.1 compatibility: **Partially compatible / extension likely**

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

## Why these four belong together

Use AA-005 §9.2 and §9.3 for the explanation that the four QAs are **independent but trade-off-forming**, not duplicate measurements.

# 4. DP-00 × QA Sensitivity Matrix

## Reviewer questions

- 왜 이 4개 QA가 DP-00에 민감한가?
- 어떤 구조적 decision이 어느 QA를 자극하는가?
- QA를 결과에 맞춰 선택한 것은 아닌가?

## Primary evidence

- `docs/architecture/analysis/AA-005-top-qa-cross-review.md`
  - Section **9.3 DP-00 × QA Sensitivity Matrix**
  - `●` sensitivity matrix
  - row-by-row rationale
  - matrix conclusion

## Report-use note

This is a **Pre-experiment architectural sensitivity hypothesis**, not an A/B/C/D result.

The matrix was recorded before final benchmark results and should be presented as evidence that QA selection was tied to structural hypotheses in advance.

# 5. Pre-experiment A/B/C/D Hypothesis Material

## Reviewer questions

- 실험 전에 각 Alternative가 어떤 QA에서 유리/불리할 것으로 예상했는가?
- 결과를 본 뒤 narrative를 바꾼 것은 아닌가?

## Primary evidence

- `docs/architecture/analysis/AA-005-top-qa-cross-review.md`
  - Section **9.4 A/B/C/D Pre-experiment Hypotheses**

Every hypothesis there is explicitly labeled:

```text
Hypothesis
Expected tendency
Not a measured result
```

## Report-use note

Keep this material separate from final result tables. It is useful for showing where actual results confirmed or contradicted pre-experiment architectural expectations.

# 6. Evaluation Methodology

## Reviewer questions

- LLM randomness를 어떻게 통제했는가?
- architecture 외 변수가 결과를 지배하지 않는가?
- score threshold를 결과에 맞춰 정한 것은 아닌가?
- 실제 model behavior와 deterministic replay의 관계는 무엇인가?

## Primary evidence

- `docs/evaluation/evaluation-principles.md`
- `docs/evaluation/architecture-experiment-methodology.md`

Key report-ready principles:

```text
Independent variable = SW Architecture Alternative
Dependent variables  = QA Primary Metrics
```

- deterministic semantic replay for architecture qualification
- deterministic Agent/tool stubs
- realistic semantic-error classes derived from actual-model runs
- actual-model/real-stack validation separated from architecture scoring
- raw-data immutability
- raw / derived / report separation
- Pilot → calibration → scoring/gate rule freeze → final evaluation
- final A/B/C/D results must not be used to retune thresholds
- Primary Metric changes require explicit rationale/version and equal recomputation

# 7. Benchmark Neutrality / Anti-gaming Controls

## Reviewer question

- 평가가 특정 Alternative에 편향되지 않았는가?

## QA-01 — dependency-latency domination control

Evidence:

- `docs/evaluation/quality-attributes/QA-01-fast-task-e2e-responsiveness.md`
- `docs/architecture/analysis/AA-005-top-qa-cross-review.md` §9.7
- `docs/evaluation/evaluation-principles.md`
- `docs/evaluation/architecture-experiment-methodology.md`

Controls:

```text
Equality:
  comparable alternatives receive the same deterministic dependency behavior

Non-dominance:
  fixture latency is calibrated so external/downstream delay does not dominate FTOL p95
```

Also: failed episodes are not converted into arbitrary huge-latency penalties.

## QA-02 — topology-neutral correctness oracle

Evidence:

- `docs/evaluation/quality-attributes/QA-02-via-interaction-orchestration-correctness.md`
- `docs/architecture/analysis/AA-002-qa02-correctness-measurement-rationale.md`
- `benchmark/schemas/scenario-constraint-schema.md`

Controls:

```text
Required
Allowed
Forbidden
```

Constraints must derive from functional requirements, policy, capability contract, scenario semantics, or task/context truth — not from one alternative's topology.

## QA-03 — Expected Change Area gaming prevention

Evidence:

- `docs/evaluation/quality-attributes/QA-03-change-flexibility.md`
- `docs/architecture/analysis/AA-003-qa03-flexibility-measurement-rationale.md`
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

Expected Change Area is not defined from post-hoc filenames/components.

## QA-04 — hidden-routing gaming prevention

Evidence:

- `docs/evaluation/quality-attributes/QA-04-model-call-overhead.md`
- `docs/architecture/analysis/AA-004-qa04-model-call-overhead-rationale.md`
- `benchmark/schemas/model-call-schema.md`
- `benchmark/schemas/run-event-schema.md`

Control:

**Execution Route Commit** is the authoritative end boundary, so a delegation/routing inference cannot escape the metric merely by moving inside ARGO after an intermediate execution-start event.

Also freeze model/prompt/cache profiles for qualification so those variables do not silently become independent variables.

# 8. Scored QA vs Must-pass Gates

## Reviewer question

- 보안/신뢰성/복구는 왜 Top 4에 없는가?

## Primary evidence

- `docs/architecture/analysis/AA-005-top-qa-cross-review.md` §9.8–9.9
- `docs/evaluation/evaluation-principles.md`

Report-ready separation:

```text
Scored Architectural Drivers
  QA-01
  QA-02
  QA-03
  QA-04

Mandatory Qualification Gates / Constraints
  Security
  Privacy / context-sharing policy
  Required task-state integrity
  Required cancellation semantics
  Failure containment
  Mandatory recovery behavior
  Trusted boundary requirements
```

Rationale:

Security/trust/recovery are **not omitted because they are unimportant**. Selected obligations are treated as mandatory qualification conditions rather than compensable score dimensions.

The cross-review also recommends a minimum acceptable QA-02 correctness gate before an alternative remains eligible for final DP-00 selection. The numeric threshold is still TBD and must be frozen before final results.

# 9. Raw Data → Derived Metrics Traceability

## Reviewer questions

- 최종 별점은 어떤 raw evidence에서 계산되었는가?
- metric을 나중에 다시 계산할 수 있는가?

## Report-ready traceability chain

```text
Raw Evidence
    ↓
Derived Metric
    ↓
QA Score
    ↓
DP-00 Trade-off Analysis
    ↓
Architecture Decision / ADR
```

## QA-01 / QA-02 / QA-04 runtime evidence

Raw schema:

- `benchmark/schemas/run-event-schema.md`

Additional QA-02 oracle/schema:

- `benchmark/schemas/scenario-constraint-schema.md`

Additional QA-04 per-call source of truth:

- `benchmark/schemas/model-call-schema.md`

Derived targets:

- FTOL p95
- AECR
- Average Model Calls to Commit Execution Route
- diagnostic latency/correctness/model-call/resource slices

## QA-03 evolution evidence

- `benchmark/schemas/evolution-scenario-schema.md`
- `benchmark/schemas/evolution-run-schema.md`

Derived target:

- CCR
- Unexpected Changed Architecture Areas Count and other propagation diagnostics

## Storage policy

```text
results/raw/      immutable source evidence
results/derived/  recomputable metrics
results/reports/  human-readable summaries/visualizations
```

Do not use final QA score tables as the sole source of truth.

# 10. Threats to Validity

This section is a living checklist for the final report.

| Threat | Why it matters | Existing control/evidence |
| --- | --- | --- |
| Replay fidelity | Frozen traces may stop representing current real models | `architecture-experiment-methodology.md`: separate actual-model fidelity track and behavior-class refresh |
| Stub vs real Agent behavior | Deterministic stubs simplify real scheduling/failure behavior | deterministic qualification + separate real-stack/Agent validation |
| Scenario representativeness | A narrow corpus can hide architecture failure surfaces | coverage-balanced QA-02 taxonomy; balanced QA-03 evolution taxonomy; versioned corpora |
| Workload weighting | Overall mean/p95 can be dominated by scenario mix | freeze corpus composition; report class slices; usage-weighted sensitivity separately |
| Model-profile dependency | QA-04/latency may shift with model choice | freeze model-profile versions; retain model-call telemetry |
| Hardware dependency | latency/resource values depend on machine/runtime | fixed Reference Development Machine for qualification; record environment metadata |
| Prompt/cache optimization dependency | token/latency/cost can shift without topology change | freeze prompt/cache profiles; record cache/token telemetry |
| External dependency latency | fixture delay can dominate QA-01 | equality + non-dominance calibration |
| Prototype fidelity | simplified A/B/C/D prototypes may not reflect production-quality architecture | report implementation-fidelity threat; use equivalent shared infrastructure; validate critical seams against reference/production behavior |
| Oracle neutrality | correctness oracle can accidentally encode preferred topology | Required/Allowed/Forbidden constraints based on semantics/requirements, not topology |
| Evolution-boundary gaming | broad post-hoc Expected Change Area can inflate CCR | common role-level boundary + pre-frozen mapping |
| Hidden-routing gaming | routing moved after intermediate owner/start event can escape QA-04 | Execution Route Commit boundary |
| Threshold hindsight | score bands can be tuned to favor observed winner | Pilot → calibration → freeze → final evaluation |

Future result/report work should add actual sensitivity analyses and residual limitations rather than claiming these controls eliminate all validity threats.

# 11. Final Trade-off / Decision Material

No final A/B/C/D benchmark result exists in this index yet.

Reserve/update this section as evidence becomes available.

## Future result artifacts

- A/B/C/D raw results: `results/raw/` — **TBD**
- derived QA metrics: `results/derived/` — **TBD**
- final QA 0~5 score table: **TBD**
- visualization source data: **TBD**
- radar/spider chart or other trade-off visualization: **TBD**
- class/category sensitivity analysis: **TBD**
- actual-model/real-stack fidelity validation: **TBD**
- must-pass gate results: **TBD**
- selected Alternative: **TBD**
- rejected Alternative rationale: **TBD**
- final DP-00 trade-off analysis: **TBD**
- ADR: `docs/adr/` — **TBD**

## Decision traceability target

The final report should make the following chain auditable:

```text
Pre-experiment architecture hypothesis
    -> frozen benchmark/scoring rules
    -> immutable raw data
    -> derived QA metrics
    -> QA score + mandatory gates
    -> trade-off interpretation
    -> selected/rejected alternative rationale
    -> ADR
    -> requirements-vNext / future approved baseline impact
```

# Reviewer Question → Evidence Map

| Reviewer question | Primary evidence |
| --- | --- |
| 왜 DP-00이 중요한가? | AA-001, DP-00 |
| 왜 A/B/C/D인가? | DP-00 alternatives + AA-001 boundary reasoning |
| 왜 이 4개 QA인가? | AA-005 §9.1–9.3 |
| 왜 이 Primary Metric인가? | QA-01~04 definitions + AA-002/003/004 metric rationale records |
| 평가가 특정 Alternative에 편향되지 않았는가? | AA-005 §9.5–9.7 + QA-02 constraint schema + QA-03 evolution schemas + QA-04 route-commit telemetry |
| LLM randomness를 어떻게 통제했는가? | evaluation-principles + architecture-experiment-methodology |
| 점수 기준을 결과에 맞춰 정한 것은 아닌가? | evaluation-principles + methodology Pilot/calibration/freeze rule |
| 보안/신뢰성은 왜 Top 4에 없는가? | AA-005 §9.8–9.9 |
| 실험 결과가 실제 제품에도 유효한가? | methodology external-validity track + Threats to Validity section + future real-stack validation |
| 최종 Alternative 선택의 trade-off 근거는 무엇인가? | future results/derived scores + AA-005 pre-experiment hypotheses + DP-00 + future ADR |

## Maintenance rule

Update this file when a new artifact becomes report-relevant, but keep the original artifact as the authoritative source.

The index should answer:

> **어떤 심사 질문을 어떤 근거 문서와 실험 데이터가 뒷받침하는가?**

It should not become a second copy of every analysis document.
