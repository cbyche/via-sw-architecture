# Final Report Evidence Index

## Status

Living index for final SW Architecture review/report assembly.

This document does **not** create a new architecture decision, QA definition, requirement, benchmark rule, or experiment result. It maps reviewer questions to authoritative architecture/evaluation artifacts and, later, to raw/derived result evidence.

`docs/requirements/requirements-v1.1.md` remains the Approved Baseline.

## How to use this index

```text
Reviewer Question
    ↓
Central vNext / Architecture Analysis
    ↓
Decision Point / Executable Specification / QA Definition
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

Use paths from this index instead of copying full reasoning into multiple documents.

---

# 0. Central vNext Rebaseline Spine

## Reviewer questions

- 현재 architecture evaluation의 중앙 기준 문서는 무엇인가?
- v1.1과 vNext의 관계는 무엇인가?
- DP-00, Top QA, methodology가 어디에서 연결되는가?

## Primary evidence

- `docs/requirements/requirements-vNext.md`
- `docs/architecture/qa-dp-traceability.md`
- `docs/evaluation/evaluation-strategy.md`
- `docs/evaluation/evaluation-principles.md`
- `docs/evaluation/architecture-experiment-methodology.md`

## Report-ready narrative

```text
Product / Architecture Concern
    ↓
DP-00 VIA Primary Execution Boundary
    ↓
A / B / C / D Base Architectures
    ↓
Top QA-01 ~ QA-04
    ↓
Controlled Architecture Qualification
    ↓
Pilot / Calibration / Rule Freeze
    ↓
Final Evaluation / Trade-off
    ↓
Tactic Mitigation where justified
    ↓
ADR
```

---

# 1. Problem / Architectural Concern

## Reviewer questions

- 왜 DP-00이 중요한가?
- 왜 lower-level DP보다 responsibility boundary를 먼저 결정해야 하는가?
- Thin VIA와 ARGO-centric topology의 tension은 무엇인가?

## Primary evidence

- `docs/architecture/analysis/AA-001-qa-rebaseline-and-primary-execution-boundary.md`
- `docs/architecture/decision-points/DP-00-primary-execution-boundary.md`
- `docs/requirements/requirements-vNext.md`

## Report-ready material

- DP-00 reviewer-facing question
- v1.1 Approved starting boundary vs vNext evaluation boundary
- why Alternative B remains boundary-challenging
- Integrated Product comparison boundary

---

# 2. DP-00 Conceptual Alternatives

## Reviewer questions

- 왜 A/B/C/D인가?
- 각 대안은 어떤 responsibility boundary를 가정하는가?
- ARGO preferred route와 ARGO-centric Primary Execution은 왜 다른가?

## Authoritative evidence

- `docs/architecture/decision-points/DP-00-primary-execution-boundary.md`
- `docs/requirements/requirements-vNext.md` §4
- `docs/architecture/qa-dp-traceability.md`

## Report-ready compatibility table

| Alternative | Topology summary | v1.1 compatibility |
| --- | --- | --- |
| **A — Thin VIA** | VIA interaction/context/intent/routing/task orchestration → Agent execution | **Compatible** |
| **B — ARGO-centric Primary Execution** | thin realtime/context → ARGO primary reasoning/tool runtime → optional specialist delegation | **Challenges baseline boundary** |
| **C — Hybrid VIA Fast Path** | A + bounded VIA-owned local execution | **Mostly compatible / extension** |
| **D — Adaptive Per-turn** | VIA Intent Refiner + Execution Path Selector → Fast / ARGO Primary / Specialist Direct | **Partially compatible / extension likely** |

Important defense point: “ARGO as preferred/default Downstream Agent” is an A/D routing-policy variant, not top-level B.

---

# 3. DP-00 Executable Base Architecture

This section indexes the **pre-prototype specification** created after the conceptual DP and Top-QA rebaseline.

## Reviewer questions

- A/B/C/D를 실제로 어떻게 구현할 것인가?
- 동일 benchmark에서 무엇을 common으로 두고 무엇을 architecture variable로 둘 것인가?
- component 수와 model-call 수를 혼동하지 않았는가?
- D가 단순히 A에 selector 하나를 얹은 모호한 구조는 아닌가?
- B의 execution state는 누가 authoritative하게 소유하는가?

## Primary evidence

- `docs/architecture/decision-points/DP-00-executable-architecture-spec.md`
- `docs/architecture/analysis/AA-006-dp00-executable-walkthrough.md`
- `docs/evaluation/base-architecture-vs-tactic-evaluation.md`

## Report-ready candidate: A/B/C/D executable topology diagrams

Use the Base topology ASCII diagrams from `DP-00-executable-architecture-spec.md`.

Recommended slide purpose:

**DP-00 Alternatives — Responsibility Placement Comparison**

Key reviewer-facing distinctions:

```text
A: 어느 Agent에게 맡길 것인가?
   Context -> Intent -> Agent Router -> Agent

B: ARGO가 primary execution runtime으로 먼저 판단/실행하고 필요 시 specialist에 위임

C: A + bounded deterministic Fast Path eligibility + VIA Fast execution

D: 어떤 execution topology 자체를 사용할 것인가?
   Context -> Intent -> Execution Path Selector
                    ├─ VIA Fast
                    ├─ ARGO Primary
                    └─ Specialist Direct
```

D does not introduce a new “Route-neutral Control Plane” abstraction and is not followed by a second top-level Agent Router.

## Report-ready candidate: Responsibility Placement Matrix

Location:

- `DP-00-executable-architecture-spec.md` §10

Use for:

- showing that Product Capability is common while decision ownership differs;
- explaining A vs D directly;
- explaining why B is boundary-challenging;
- showing C as A + bounded Fast Path.

## Report-ready candidate: State Ownership Matrix

Location:

- `DP-00-executable-architecture-spec.md` §11

Critical B distinction:

```text
ARGO
  = authoritative execution route/thread/plan/domain state

VIA
  = user-facing task projection / correlation / result interaction
```

Use for reviewer questions about task continuity, result binding, and whether VIA is secretly duplicating ARGO execution state.

## Report-ready candidate: Alternative structural risk table

Location:

- `DP-00-executable-architecture-spec.md` §14

This is a **pre-experiment risk hypothesis**, not measured failure evidence.

---

# 4. Base Architecture vs Tactic

## Reviewer questions

- 특정 Alternative에만 optimization을 적용해 비교한 것은 아닌가?
- 왜 Intent+Routing fusion을 Base에 넣지 않았는가?
- classifier, speculative routing, prompt/cache optimization은 어디에서 평가하는가?

## Primary evidence

- `docs/evaluation/base-architecture-vs-tactic-evaluation.md`
- `docs/architecture/decision-points/DP-00-executable-architecture-spec.md` §3–4, §13
- `docs/architecture/analysis/AA-006-dp00-executable-walkthrough.md` §11

## Report-ready distinction

```text
Architecture
  = 누가 그 decision responsibility를 소유하는가?

Tactic
  = 그 responsibility를 더 빠르게/정확하게/효율적으로 수행하기 위해
    어떤 mechanism을 사용하는가?
```

Base excludes optional:

- cross-component GenAI fusion;
- classical/ML classifier routing optimization;
- embedding routing optimization;
- speculative execution/routing;
- parallel inference optimization;
- prompt/cache optimization;
- partial-ASR semantic pre-routing;
- semantic precomputation.

Recommended slide narrative:

```text
Base Architecture Trade-off
        ↓
Tactic Mitigation
        ↓
Improved Architecture / Final Decision
```

---

# 5. S1~S5 Executable Walkthrough / Architecture Neutrality

## Reviewer questions

- A/B/C/D 모두 같은 product request를 실제로 처리할 수 있는가?
- expected model-call difference가 component count를 임의로 세어서 나온 것인가?
- Follow-up에서 D가 매 turn route를 다시 선택하는가?
- initial ARGO delegation이 QA-04에서 숨겨지지 않는가?

## Primary evidence

- `docs/architecture/analysis/AA-006-dp00-executable-walkthrough.md`

### Scenarios

- S1 Local-capable — “볼륨 조금 줄여줘.”
- S2 General Agent — “다운로드 폴더를 정리해줘.”
- S3 Specialized Agent — “현재 Wi-Fi 문제를 진단해줘.”
- S4 Existing-task Follow-up — T1 Wi-Fi analysis → “그럼 DNS도 확인해봐.”
- S5 Ambiguous Referent / Clarification — “그 문서 열어줘.” → “오른쪽에 있는 거.”

## Report-ready candidate: Pre-experiment expected logical Generative Model Call trace

| Scenario | A | B | C | D |
| --- | ---: | ---: | ---: | ---: |
| **S1 Local** | **2** | **1** | **1** | **2** |
| **S2 General Agent** | **2** | **1** | **2** | **2** |
| **S3 Specialized** | **2** | **1** | **2** | **2** |
| **S4 Follow-up** | **1** | **1** | **1** | **1** |
| **S5 Clarification + Local** | **3** | **2** | **2** | **3** |

Mandatory label when reused:

> **Pre-experiment specification walkthrough — Expected logical Generative Model Call trace — Not measured benchmark result.**

## Report-ready candidate: Architecture Neutrality stress-test

Location:

- `AA-006` §9

Current specification-readiness verdict:

```text
Same Product Capability                   PASS
No capability advantage for B             PASS
C ownership difference, not capability    PASS
D topology decision distinct from A       PASS
No optional tactic in Base                PASS
Component != Model Call                   PASS
Follow-up route reuse coherent            PASS
Execution Route Commit neutral            PASS
```

This PASS is **prototype specification readiness**, not QA/benchmark success.

---

# 6. Top Architectural Drivers

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

---

# 7. DP-00 × QA Sensitivity Matrix

## Reviewer questions

- 왜 이 4개 QA가 DP-00에 민감한가?
- QA를 결과에 맞춰 선택한 것은 아닌가?
- component 수가 많으면 QA-04가 자동으로 나빠지는가?

## Primary evidence

- `docs/architecture/analysis/AA-005-top-qa-cross-review.md` §9.3

The `●` matrix is a **Pre-experiment architectural sensitivity hypothesis**.

Important executable-spec clarification now preserved in AA-005:

- deterministic stage/component separation alone does **not** add QA-04 calls;
- QA-04 sensitivity arises when the architecture requires additional logical **Generative AI decisions** before Execution Route Commit;
- cross-component GenAI fusion is a later Tactic, not silently part of A/C/D Base.

---

# 8. Pre-experiment A/B/C/D Hypotheses

Primary evidence:

- `AA-005-top-qa-cross-review.md` §9.4

Every statement is labeled:

```text
Hypothesis
Expected tendency
Not a measured result
```

Use final report to compare prediction vs actual result without rewriting history.

---

# 9. Evaluation Methodology

## Reviewer questions

- LLM randomness를 어떻게 통제했는가?
- architecture 외 변수가 결과를 지배하지 않는가?
- score threshold를 결과에 맞춰 정한 것은 아닌가?
- Base Architecture와 optimization maturity를 어떻게 분리했는가?

## Primary evidence

- `docs/evaluation/evaluation-strategy.md`
- `docs/evaluation/evaluation-principles.md`
- `docs/evaluation/architecture-experiment-methodology.md`
- `docs/evaluation/base-architecture-vs-tactic-evaluation.md`

## Report-ready method

```text
Independent variable = SW Architecture Alternative

Controlled/Frozen as applicable
  scenario/corpus semantics
  deterministic semantic replay
  deterministic Agent/tool behavior
  model/prompt/cache profiles
  machine/environment
  controlled dependency latency
  Base Architecture version
```

```text
Executable Base Specification
  -> Canonical Benchmark Contract
  -> Prototype
  -> Pilot
  -> Calibration
  -> Scoring/Gate/Benchmark Freeze
  -> Final Evaluation
  -> Optional Tactic Evaluation
```

---

# 10. Benchmark Neutrality / Anti-gaming Controls

## QA-01 — dependency-latency domination

Evidence:

- QA-01 authoritative document
- AA-005 §9.7
- central evaluation strategy

Controls:

```text
Equality
+ Non-dominance
```

Failed episodes do not receive arbitrary huge-latency penalties.

## QA-02 — topology-neutral correctness oracle

Evidence:

- QA-02 authoritative document
- AA-002
- `benchmark/schemas/scenario-constraint-schema.md`

Oracle:

```text
Required
Allowed
Forbidden
```

Constraints derive from requirements/policy/capability/scenario truth, not topology.

## QA-03 — Expected Change Area gaming prevention

Evidence:

- QA-03 authoritative document
- AA-003
- evolution schemas

Sequence:

```text
Common Expected Change Roles
 -> Alternative mapping
 -> freeze
 -> implementation
 -> diff/test result
```

## QA-04 — hidden-routing and component-count gaming prevention

Evidence:

- QA-04 authoritative document
- AA-004
- AA-005 §9.3 clarification
- DP-00 executable spec
- AA-006 walkthrough
- model-call/run-event schemas

Controls:

- authoritative end boundary = **Execution Route Commit**;
- initial ARGO specialist delegation remains visible;
- later DOMAIN sub-task delegation is excluded;
- component/stage count is not Model Call count;
- deterministic rule/state/contract lookup contributes zero Generative Model Calls;
- optional cross-component fusion/classifier optimization is separated into Tactic evaluation.

---

# 11. Legacy v1.1 QA Reclassification

Primary evidence:

- `requirements-vNext.md` §6
- `qa-dp-traceability.md`
- original wording: `requirements-v1.1.md`

Report-use summary:

- Legacy QA-01 → Top QA-01 diagnostic
- Legacy QA-05 → Top QA-03 detail
- Legacy QA-07 → Top QA-04 resource/cost diagnostics
- Legacy QA-09/10/11 → Top QA-02 correctness slices
- Legacy QA-03/04/06 → reliability/privacy/security gate candidates
- Legacy QA-02/08 → operational/cross-cutting supporting concerns

---

# 12. Scored QA vs Must-pass Gates

## Primary evidence

- AA-005 §9.8–9.9
- requirements-vNext §7–8
- qa-dp-traceability
- evaluation-strategy

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

Security/trust/recovery are not less important; selected obligations are non-compensable conditions.

QA-02 minimum correctness eligibility gate remains TBD until Pilot/calibration.

---

# 13. Raw Data → Derived Metrics → Decision

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
DP-00 Trade-off
    ↓
Tactic delta where evaluated
    ↓
Architecture Decision / ADR
```

Runtime schemas:

- `benchmark/schemas/run-event-schema.md`
- `benchmark/schemas/scenario-constraint-schema.md`
- `benchmark/schemas/model-call-schema.md`

Evolution schemas:

- `benchmark/schemas/evolution-scenario-schema.md`
- `benchmark/schemas/evolution-run-schema.md`

Next benchmark-contract checkpoint should add/index the canonical scenario/common-variable boundary artifacts here.

---

# 14. Threats to Validity

| Threat | Existing control/evidence |
| --- | --- |
| Replay fidelity | Actual-model/Real-stack validation track |
| Stub vs real Agent | deterministic qualification + real Agent validation |
| Scenario representativeness | coverage-balanced/versioned corpora |
| Workload weighting | frozen populations + class/category slices |
| Model-profile dependency | frozen profile/version + telemetry |
| Hardware dependency | controlled reference environment |
| Prompt/cache optimization | frozen profiles; also separated as Tactic in Base comparison |
| External dependency latency | QA-01 equality + non-dominance |
| Prototype fidelity | executable spec + common Product Obligations + future common/variable boundary |
| Oracle neutrality | QA-02 constraint manifest |
| Evolution-boundary gaming | pre-frozen common Expected Change Roles |
| Hidden-routing gaming | Execution Route Commit |
| Component-count/model-call confusion | explicit decision mechanism and ModelCall telemetry |
| Optimization-maturity bias | Base Architecture vs Tactic separation |
| Threshold hindsight | Pilot → calibration → freeze → final |

Residual validity threats must still be reported after experimentation.

---

# 15. Final Trade-off / Decision Material

No final A/B/C/D benchmark result exists yet.

Current pre-result report assets now available:

- DP-00 problem statement;
- conceptual A/B/C/D + v1.1 compatibility;
- executable A/B/C/D topology diagrams;
- Responsibility Placement Matrix;
- State Ownership Matrix;
- Top QA + Primary Metric table;
- DP-00 × QA Sensitivity Matrix;
- Pre-experiment A/B/C/D hypotheses;
- Base Architecture vs Tactic distinction;
- S1~S5 20-path walkthrough;
- expected logical Generative Model Call trace;
- Alternative structural risk table;
- Architecture Neutrality readiness verdict;
- controlled evaluation pipeline;
- anti-gaming controls;
- Scored QA vs Must-pass Gates;
- Raw → Metric → Score/Gate → Decision traceability;
- Threats-to-Validity checklist.

Future placeholders:

- Canonical Benchmark Scenario Contract — **TBD**
- Common vs Variable Experimental Boundary — **TBD**
- prototypes — **TBD**
- raw results — `results/raw/` **TBD**
- derived QA metrics — `results/derived/` **TBD**
- final QA score/gate table — **TBD**
- tactic experiment results — **TBD**
- sensitivity analysis — **TBD**
- Actual-model/Real-stack validation — **TBD**
- selected/rejected Alternative rationale — **TBD**
- ADR — **TBD**

---

# Reviewer Question → Evidence Map

| Reviewer question | Primary evidence |
| --- | --- |
| 왜 DP-00이 중요한가? | AA-001 + DP-00 + requirements-vNext |
| 왜 A/B/C/D인가? | DP-00 conceptual definition + executable spec |
| 실제 구현 구조는 어떻게 다른가? | DP-00 executable spec Responsibility/State matrices |
| 왜 C = A + Fast Path인가? | executable spec §8 |
| D와 A의 차이는 무엇인가? | executable spec §9 + Responsibility Matrix |
| B execution state는 누가 소유하는가? | executable spec §7/§11 + AA-006 |
| 왜 이 4개 QA인가? | AA-005 sensitivity/cross-review |
| 왜 Primary Metric이 이 값인가? | QA-01~04 + AA-002/003/004 |
| component가 많으면 QA-04가 불리한가? | AA-005 §9.3 clarification + executable spec §4/§13 |
| optimization이 특정 Alternative에만 섞이지 않았는가? | base-architecture-vs-tactic-evaluation.md + AA-006 neutrality stress-test |
| S1~S5가 모든 topology에서 가능한가? | AA-006 20-path walkthrough |
| LLM randomness는 어떻게 통제하는가? | evaluation strategy/principles/methodology |
| 보안/신뢰성은 왜 Top 4가 아닌가? | AA-005 + requirements-vNext + evaluation strategy |
| raw evidence가 decision으로 어떻게 연결되는가? | evaluation strategy + schemas + this index §13 |
| 최종 trade-off 근거는 무엇인가? | future raw/derived/gates/tactic results + ADR |

## Maintenance rule

Update this index whenever a new artifact becomes report-relevant, while keeping the original artifact authoritative.

The index should answer:

> **심사위원의 질문 → 근거 문서 → 실험 데이터 → 결정**

It must remain an index, not a duplicate final report.
