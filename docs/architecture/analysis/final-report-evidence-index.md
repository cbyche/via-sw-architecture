# Final Report Evidence Index

## Status

Living index for final SW Architecture review/report assembly.

This document does **not** create a new architecture decision, QA definition, requirement, benchmark result, or score. It maps reviewer questions to authoritative architecture/evaluation artifacts and, later, to raw/derived result evidence.

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
Immutable Raw Evidence
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

Primary evidence:

- `docs/requirements/requirements-vNext.md`
- `docs/architecture/qa-dp-traceability.md`
- `docs/evaluation/evaluation-strategy.md`
- `docs/evaluation/evaluation-principles.md`
- `docs/evaluation/architecture-experiment-methodology.md`

Report-ready narrative:

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
Optional Tactic Mitigation
    ↓
ADR
```

---

# 1. Problem / Architectural Concern

Reviewer questions:

- 왜 DP-00이 중요한가?
- 왜 lower-level DP보다 responsibility boundary를 먼저 결정하는가?
- Thin VIA와 ARGO-centric topology의 tension은 무엇인가?

Primary evidence:

- `docs/architecture/analysis/AA-001-qa-rebaseline-and-primary-execution-boundary.md`
- `docs/architecture/decision-points/DP-00-primary-execution-boundary.md`
- `docs/requirements/requirements-vNext.md`

Report-ready material:

- DP-00 reviewer-facing question;
- v1.1 Approved starting boundary vs vNext evaluation boundary;
- why Alternative B remains boundary-challenging;
- Integrated Product comparison boundary.

---

# 2. DP-00 Conceptual Alternatives

Authoritative evidence:

- `docs/architecture/decision-points/DP-00-primary-execution-boundary.md`
- `docs/requirements/requirements-vNext.md`
- `docs/architecture/qa-dp-traceability.md`

Report-ready compatibility table:

| Alternative | Topology summary | v1.1 compatibility |
| --- | --- | --- |
| **A — Thin VIA** | VIA interaction/context/intent/routing/task orchestration → Agent execution | **Compatible** |
| **B — ARGO-centric Primary Execution** | thin realtime/context → ARGO primary reasoning/tool runtime → optional specialist delegation | **Challenges baseline boundary** |
| **C — Hybrid VIA Fast Path** | A + bounded VIA-owned local execution | **Mostly compatible / extension** |
| **D — Adaptive Per-turn** | VIA Intent Refiner + Execution Path Selector → Fast / ARGO Primary / Specialist Direct | **Partially compatible / extension likely** |

Important defense point: “ARGO as preferred/default Downstream Agent” is an A/D routing-policy variant, not top-level B.

---

# 3. DP-00 Executable Base Architecture

Primary evidence:

- `docs/architecture/decision-points/DP-00-executable-architecture-spec.md`
- `docs/architecture/analysis/AA-006-dp00-executable-walkthrough.md`
- `docs/evaluation/base-architecture-vs-tactic-evaluation.md`

Report-ready candidates:

1. **A/B/C/D executable topology diagrams** — executable spec Base topology sections.
2. **Responsibility Placement Matrix** — executable spec.
3. **State Ownership Matrix** — executable spec.
4. **Alternative structural-risk table** — executable spec.
5. **A vs D reviewer-facing distinction**:

```text
A: 어느 Agent에게 맡길 것인가?
D: 어떤 execution topology 자체를 사용할 것인가?
```

6. **B authoritative state distinction**:

```text
ARGO = authoritative execution route/thread/plan/domain state
VIA  = user-facing task projection/correlation/result interaction
```

7. **C definition**: A + bounded VIA Fast Path.

---

# 4. Base Architecture vs Tactic

Primary evidence:

- `docs/evaluation/base-architecture-vs-tactic-evaluation.md`
- `docs/architecture/decision-points/DP-00-executable-architecture-spec.md`
- `docs/architecture/analysis/AA-006-dp00-executable-walkthrough.md`

Report-ready distinction:

```text
Architecture
  = 누가 그 decision responsibility를 소유하는가?

Tactic
  = 그 responsibility를 더 빠르게/정확하게/효율적으로 수행하기 위해
    어떤 mechanism을 사용하는가?
```

Base excludes optional:

- cross-component GenAI fusion;
- classical/ML classifier routing;
- embedding routing;
- speculative execution/routing;
- parallel inference optimization;
- prompt/cache optimization;
- partial-ASR semantic pre-routing;
- semantic precomputation.

Recommended report narrative:

```text
Base Architecture Trade-off
        ↓
Tactic Mitigation
        ↓
Improved Architecture / Final Decision
```

---

# 5. S1~S5 Executable Walkthrough

Primary evidence:

- `docs/architecture/analysis/AA-006-dp00-executable-walkthrough.md`

Smoke scenarios:

- S1 Local-capable — “볼륨 조금 줄여줘.”
- S2 General Agent — “다운로드 폴더를 정리해줘.”
- S3 Specialized Agent — “현재 Wi-Fi 문제를 진단해줘.”
- S4 Existing-task Follow-up — T1 → “그럼 DNS도 확인해봐.”
- S5 Ambiguous Referent / Clarification — “그 문서 열어줘.” → “오른쪽에 있는 거.”

Report-ready expected logical Generative Model Call trace:

| Scenario | A | B | C | D |
| --- | ---: | ---: | ---: | ---: |
| **S1 Local** | **2** | **1** | **1** | **2** |
| **S2 General Agent** | **2** | **1** | **2** | **2** |
| **S3 Specialized** | **2** | **1** | **2** | **2** |
| **S4 Follow-up** | **1** | **1** | **1** | **1** |
| **S5 Clarification + Local** | **3** | **2** | **2** | **3** |

Mandatory label when reused:

> **Pre-experiment specification walkthrough — Expected logical Generative Model Call trace — Not measured benchmark result.**

---

# 6. Top Architectural Drivers

| QA | Reviewer-facing question | Primary Metric |
| --- | --- | --- |
| **QA-01** | 빠른가? | **Fast-task Outcome Latency p95 (FTOL p95)** |
| **QA-02** | 정확한가? | **Architecture Episode Exact Conformance Rate (AECR)** |
| **QA-03** | 변경이 잘 격리되는가? | **Change Containment Rate (CCR)** |
| **QA-04** | 실행 경로를 정하기 위해 AI 판단을 얼마나 요구하는가? | **Average Model Calls to Commit Execution Route** |

Primary evidence:

- `docs/evaluation/quality-attributes/QA-01-fast-task-e2e-responsiveness.md`
- `docs/evaluation/quality-attributes/QA-02-via-interaction-orchestration-correctness.md`
- `docs/evaluation/quality-attributes/QA-03-change-flexibility.md`
- `docs/evaluation/quality-attributes/QA-04-model-call-overhead.md`
- `docs/architecture/analysis/AA-005-top-qa-cross-review.md`

---

# 7. DP-00 × QA Sensitivity Matrix

Primary evidence:

- `docs/architecture/analysis/AA-005-top-qa-cross-review.md` §9.3

The `●` matrix is a **Pre-experiment architectural sensitivity hypothesis**.

Important nuance:

- component/stage separation itself does not automatically add QA-04 calls;
- QA-04 changes when the architecture requires additional logical Generative AI decisions before Execution Route Commit;
- cross-component GenAI fusion is a later Tactic unless one Base component intrinsically owns the combined responsibility, as in B.ARGOPrimary.

---

# 8. Canonical Benchmark Contract / Experimental Boundary

This section indexes the benchmark-contract checkpoint that follows the executable Base specification.

## Reviewer questions

- 무엇을 모든 Alternative에 동일하게 제공했는가?
- 무엇은 Architecture-under-Test 내부 책임으로 남겼는가?
- benchmark harness가 architecture 판단을 대신하지 않았는가?
- Ground truth가 AUT에 노출되지 않았는가?

## Primary evidence

- `docs/evaluation/dp00-experimental-boundary.md`
- `docs/architecture/analysis/AA-007-benchmark-contract-neutrality-review.md`

## Report-ready candidate: three-layer experimental boundary

```text
Layer 1 — Shared Benchmark Environment
  Scenario / context-state fixtures / semantic replay / Agent-tool fixtures
  policy / machine / controlled latency / evaluator oracle
            ↓
Layer 2 — Base Implementation Rules
  deterministic facts/state/contracts
  semantic decision ownership
  no optional tactic
            ↓
Layer 3 — Architecture-specific Structure
  A / B / C / D responsibility placement + state ownership + topology
```

Reviewer-facing summary:

> **동일한 사용자 상황, 동일한 semantic model behavior, 동일한 Agent/Tool behavior를 A/B/C/D에 제공하고 responsibility placement와 execution topology를 독립변수로 비교한다.**

---

# 9. Runtime / Evolution / Mandatory Gate — Three-track Evaluation

Primary evidence:

- `docs/evaluation/dp00-experimental-boundary.md` §1
- existing evolution schemas under `benchmark/schemas/`
- central `evaluation-strategy.md`

Report-ready diagram:

```text
Architecture Evaluation
|
+-- Runtime Episode Benchmark
|    +-- QA-01
|    +-- QA-02
|    +-- QA-04
|
+-- Evolution Benchmark
|    +-- QA-03
|
+-- Mandatory Qualification Suite
     +-- Security / Privacy
     +-- Cancellation / Task-state integrity
     +-- Failure containment
     +-- Required recovery
```

Rationale: these have different measurement units and should not be forced into one universal scenario schema.

---

# 10. Stimulus vs Evaluator-only Oracle

Primary evidence:

- `docs/evaluation/dp00-experimental-boundary.md` §6
- `benchmark/schemas/runtime-scenario-schema.md`
- `docs/architecture/analysis/AA-007-benchmark-contract-neutrality-review.md`

Report-ready diagram:

```text
Scenario Source
  ├─ AUT-visible Stimulus
  │    User input
  │    raw context evidence
  │    logical initial state
  │    capability/health/policy facts
  │    dependency responses
  │
  └─ Evaluator-only Oracle
       ground-truth referent/task relation
       Required / Allowed / Forbidden constraints
       expected result binding
       success predicate
       scoring eligibility
```

Key sentence:

> **Benchmark harness는 ground truth를 알고 있지만 Architecture-under-Test에는 전달하지 않는다.**

Also preserve the scenario-id anti-hardcoding rule.

---

# 11. Runtime Scenario Manifest

Primary evidence:

- `benchmark/schemas/runtime-scenario-schema.md`
- `benchmark/catalog/runtime-scenario-catalog.md`

Key capabilities:

- one user-goal episode, not just one turn;
- branchable clarification/follow-up interaction script;
- Voice/Text modality;
- ground-truth acoustic EOS metadata for QA-01;
- logical initial state refs;
- raw context-evidence refs;
- capability/health/policy profiles;
- semantic behavior-plan reference;
- Agent/Tool/latency fixture refs;
- evaluator-only constraint/success-predicate refs;
- separate QA-01/02/04 eligibility.

Report use: **Benchmark Scenario Contract** slide or appendix.

---

# 12. Semantic Responsibility Replay

Primary evidence:

- `benchmark/schemas/semantic-behavior-plan-schema.md`
- `benchmark/contracts/model-replay-request-contract.md`
- `benchmark/schemas/model-call-schema.md`
- `docs/architecture/analysis/AA-007-benchmark-contract-neutrality-review.md`

## Reviewer question

- LLM randomness를 어떻게 통제하면서 A의 separate calls와 B의 fused call을 공정하게 비교했는가?

## Report-ready diagram

```text
Frozen semantic condition
  INTENT_INTERPRETATION = CORRECT
  EXECUTION_ROUTE_SELECTION = WRONG_CANDIDATE
              ↓
      responsibility-keyed replay
         /                 \
A separate calls          B one ARGO call
Intent -> CORRECT         [intent, route]
Router -> WRONG           -> [CORRECT, WRONG]
         \                 /
          same semantic fault
```

Core rule:

> **동일한 semantic success/error condition을 responsibility 단위로 A/B/C/D에 주입하여 Model intelligence를 통제하고 SW topology의 validation/recovery 차이를 비교한다.**

Important details:

- replay is not keyed by global model-call ordinal;
- attempt sequences support retry/recovery;
- unused semantic operations are allowed for deterministic architecture logic;
- one fused generation with multiple operation keys is still one QA-04 logical ModelCall;
- owner→allowed semantic responsibilities are frozen to prevent metadata gaming.

---

# 13. Canonical Observation Events

Primary evidence:

- `benchmark/schemas/canonical-event-schema.md`
- `benchmark/schemas/run-event-schema.md`
- `benchmark/schemas/model-call-schema.md`

Report-ready candidate: **Canonical Events → QA** diagram.

```text
interaction.acoustic_eos --------┐
                                 ├─ QA-01 FTOL
useful_outcome.observed ---------┘

clarification / task / route / result events
                                 └─ QA-02 constraint evaluation

model.generation.*
execution.route_committed
                                 └─ QA-04 call/route-commit accounting
```

Canonical events do not require internal names such as `router.selected_agent` or `argo.delegate.called`.

Authoritative-producer distinction is report-relevant:

- Acoustic EOS → benchmark fixture;
- Model generation → model adapter/fixture + ModelCall;
- route commit → AUT event validated against route evidence;
- useful outcome → external `OUTCOME_PROBE` whenever feasible.

---

# 14. Canonical ExecutionRoute

Primary evidence:

- `docs/evaluation/dp00-experimental-boundary.md`
- `benchmark/schemas/canonical-event-schema.md`
- `benchmark/schemas/runtime-scenario-schema.md`

Architecture-neutral representation:

```text
route_kind:
  LOCAL_DIRECT
  EXECUTOR_DIRECT
  EXECUTOR_DELEGATED

initial_executor_id
final_executor_id_if_known
delegation_chain[]
```

Avoid using `VIA_FAST`, `ARGO_PRIMARY`, `SPECIALIST_DIRECT` as the canonical benchmark enum. Those remain useful architecture-document labels only.

---

# 15. Runtime Fixture Seams / ARGO Boundary

Primary evidence:

- `benchmark/contracts/runtime-fixture-contracts.md`
- `docs/evaluation/dp00-experimental-boundary.md`
- AA-007 ARGO-stub stress-test

Report-ready fairness boundary:

```text
Architecture-owned pre-route decision
        ↓
Execution Route Commit
        ↓
Common deterministic domain behavior where equivalent
```

Critical B rule:

```text
B.ARGOPrimary Controller = AUT
post-route ARGO/Specialist domain behavior = common fixture where comparable
```

This prevents the common stub from deleting B's defining architecture responsibility.

Tool/product capability is shared; local-vs-Agent ownership remains the variable.

---

# 16. S1~S5 Smoke vs R1~R10 Scoring Taxonomy

Primary evidence:

- S1~S5: `AA-006-dp00-executable-walkthrough.md`
- R1~R10: `benchmark/catalog/runtime-scenario-catalog.md`

Report-ready distinction:

```text
S1~S5
  = architecture specification smoke / consistency

R1~R10
  = scoring benchmark taxonomy / coverage skeleton
```

R1~R10 classes:

- R1 Local / Bounded
- R2 General Agent
- R3 Specialized Agent
- R4 Context / Referent
- R5 Existing-task Follow-up
- R6 Ambiguity / Clarification
- R7 Execution-path Trap
- R8 Concurrent Task / Result
- R9 Compound Request
- R10 Dependency / Model Error

The initial catalog is **not** the final frozen scoring corpus.

---

# 17. Benchmark Neutrality / Anti-gaming Review

Primary evidence:

- `docs/architecture/analysis/AA-007-benchmark-contract-neutrality-review.md`

Report-ready attack/control table candidates:

| Attack | Control |
| --- | --- |
| Oracle leakage | Stimulus vs evaluator-only Oracle separation |
| Harness responsibility leakage | AUT-owned decision list + fixture non-responsibilities |
| Separate vs fused replay bias | semantic responsibility operation keys |
| Deterministic logic forced to model | operations consumed only on AUT request |
| Replay erases ModelCall | 1 logical generation = 1 ModelCall even under replay |
| Responsibility metadata gaming | frozen owner→responsibility mapping + adapter validation |
| B ARGO stub leakage | B pre-route ARGO stays AUT |
| QA-01 self-report | external Outcome Probe |
| Local capability asymmetry | common tool/product capability, variable ownership |
| Topology-specific route enum | architecture-neutral ExecutionRoute |
| Provisional route commit | route commit validated against operational dispatch/accept evidence |
| Corpus imbalance | class/tags/raw preservation; aggregation frozen after Pilot |
| Tactic contamination | Base vs Tactic separation |

Current design-readiness verdict:

```text
Canonical Benchmark Contract  = PASS WITH TBD
Architecture Neutrality       = PASS
Experimental Boundary         = PASS
Prototype/Harness Readiness   = PASS
```

These are design-readiness judgments, not benchmark results.

---

# 18. QA-specific Anti-gaming Controls

## QA-01

- ground-truth Acoustic EOS from benchmark fixture;
- useful outcome from external Outcome Probe where feasible;
- dependency latency equality + non-dominance;
- failures are not assigned arbitrary huge latency.

## QA-02

- Required / Allowed / Forbidden constraints;
- topology-neutral canonical outcomes;
- constraints derived from functional semantics/policy/capability truth, not preferred topology;
- semantic error replay includes wrong/ambiguous outputs.

## QA-03

- common role-level Expected Change Area;
- alternative mapping frozen before implementation/result observation;
- acceptance + regression + no unexpected propagation required for CCR success.

## QA-04

- logical Generative AI generation accounting;
- component count is not model-call count;
- replay does not erase ModelCalls;
- authoritative end boundary = Execution Route Commit;
- hidden ARGO initial delegation remains counted;
- pure post-commit DOMAIN reasoning is excluded;
- terminal no-route-commit aggregation remains Pilot TBD rather than an arbitrary penalty.

---

# 19. Legacy v1.1 QA Reclassification

Primary evidence:

- `requirements-vNext.md`
- `qa-dp-traceability.md`
- original wording: `requirements-v1.1.md`

Report-use summary:

- Legacy QA-01 → Top QA-01 diagnostic;
- Legacy QA-05 → Top QA-03 detail;
- Legacy QA-07 → Top QA-04 resource/cost diagnostics;
- Legacy QA-09/10/11 → Top QA-02 correctness slices;
- Legacy QA-03/04/06 → reliability/privacy/security gate candidates;
- Legacy QA-02/08 → operational/cross-cutting supporting concerns.

---

# 20. Scored QA vs Mandatory Gates

Primary evidence:

- AA-005;
- requirements-vNext;
- qa-dp-traceability;
- evaluation-strategy.

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

Security/trust/recovery are non-compensable conditions, not less-important concerns.

QA-02 minimum correctness eligibility gate remains TBD until Pilot/calibration.

---

# 21. Raw Data → Derived Metrics → Decision

Report-ready chain:

```text
Scenario / Behavior Plan / Frozen Profiles
    ↓
Immutable Raw Events + ModelCalls + Run Provenance
    ↓
Derived Metric
    ↓
QA Primary Metric
    ↓
0–5 Score / Qualification Gate
    ↓
DP-00 Trade-off
    ↓
Optional Tactic delta
    ↓
Architecture Decision / ADR
```

Runtime contracts:

- `benchmark/schemas/runtime-scenario-schema.md`
- `benchmark/schemas/semantic-behavior-plan-schema.md`
- `benchmark/contracts/model-replay-request-contract.md`
- `benchmark/contracts/runtime-fixture-contracts.md`
- `benchmark/schemas/canonical-event-schema.md`
- `benchmark/schemas/run-event-schema.md`
- `benchmark/schemas/model-call-schema.md`
- `benchmark/schemas/runtime-run-provenance-schema.md`
- `benchmark/schemas/scenario-constraint-schema.md`

Evolution contracts:

- `benchmark/schemas/evolution-scenario-schema.md`
- `benchmark/schemas/evolution-run-schema.md`

Storage:

```text
results/raw/      immutable source evidence
results/derived/  recomputable metric / score inputs
results/reports/  human-readable results / visualizations
```

---

# 22. Threats to Validity

| Threat | Existing control/evidence |
| --- | --- |
| Replay fidelity | Actual-model/Real-stack validation track |
| Stub vs real Agent | deterministic qualification + real Agent validation |
| Scenario representativeness | R1~R10 skeleton + future frozen corpus review |
| Workload weighting | scenario class/tags/raw preservation; aggregation frozen after Pilot |
| Model-profile dependency | frozen profile/version + ModelCall telemetry |
| Hardware dependency | controlled reference environment |
| Prompt/cache optimization | frozen profiles + Tactic separation |
| External dependency latency | QA-01 equality + non-dominance |
| Prototype fidelity | executable spec + experimental-boundary contract |
| Oracle leakage | Stimulus / evaluator-only Oracle separation |
| Oracle neutrality | QA-02 Required/Allowed/Forbidden constraints |
| Evolution-boundary gaming | pre-frozen common Expected Change Roles |
| Hidden-routing gaming | Execution Route Commit |
| Responsibility metadata gaming | owner→semantic-responsibility mapping validation |
| Useful-outcome self-report | external Outcome Probe |
| B ARGO stub leakage | pre-route B.ARGOPrimary remains AUT |
| Component-count/model-call confusion | logical generation contract |
| Optimization maturity bias | Base vs Tactic separation |
| Threshold hindsight | Pilot → calibration → freeze → final |

Residual threats must still be reported after experimentation.

---

# 23. Final Trade-off / Decision Material

No final A/B/C/D benchmark result exists yet.

Current pre-result report assets:

- DP-00 problem statement;
- conceptual A/B/C/D + v1.1 compatibility;
- executable topology diagrams;
- Responsibility Placement Matrix;
- State Ownership Matrix;
- Top QA + Primary Metric table;
- DP-00 × QA Sensitivity Matrix;
- Pre-experiment A/B/C/D hypotheses;
- Base Architecture vs Tactic distinction;
- S1~S5 20-path walkthrough;
- expected logical Generative Model Call trace;
- Alternative structural-risk table;
- Runtime/Evolution/Mandatory Gate three-track diagram;
- Shared/Controlled vs Architecture-owned experimental-boundary diagram;
- Stimulus vs Evaluator-only Oracle diagram;
- Semantic Responsibility Replay diagram;
- Canonical Events → QA-01/02/04 diagram;
- S1~S5 Smoke vs R1~R10 Scoring taxonomy;
- Benchmark Neutrality/Anti-gaming review table;
- Scored QA vs Mandatory Gates;
- Raw → Metric → Score/Gate → Decision traceability;
- Threats-to-Validity checklist.

Future placeholders:

- Prototype/Harness Specification — **TBD**
- Base A/B/C/D prototypes — **TBD**
- S1~S5 executable smoke results — **TBD**
- final runtime scoring corpus — **TBD**
- actual-model behavior corpus — **TBD**
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
| 왜 A/B/C/D인가? | DP-00 conceptual + executable spec |
| 실제 구현 구조는 어떻게 다른가? | executable spec Responsibility/State matrices |
| 왜 C = A + Fast Path인가? | executable spec |
| D와 A의 차이는 무엇인가? | executable spec + AA-006 |
| B execution state는 누가 소유하는가? | executable spec + AA-006 |
| 왜 이 4개 QA인가? | AA-005 sensitivity/cross-review |
| component가 많으면 QA-04가 불리한가? | AA-005 nuance + executable spec Base Decision Mechanism |
| optimization이 특정 Alternative에 섞이지 않았는가? | base-architecture-vs-tactic-evaluation.md |
| S1~S5가 모든 topology에서 가능한가? | AA-006 |
| 무엇이 controlled이고 무엇이 architecture variable인가? | dp00-experimental-boundary.md |
| Ground truth leakage를 어떻게 막았는가? | runtime-scenario-schema + AA-007 |
| LLM randomness를 어떻게 통제했는가? | semantic-behavior-plan + model-replay contract + methodology |
| separate/fused model topology에 동일 error를 어떻게 넣는가? | semantic-behavior-plan + AA-007 |
| replay 때문에 QA-04 call이 사라지지 않는가? | model-replay contract + model-call-schema |
| benchmark가 B의 ARGO responsibility를 stub으로 지우지 않는가? | runtime-fixture-contracts + AA-007 |
| QA-01 useful outcome을 AUT가 조작할 수 없는가? | canonical-event-schema Outcome Probe rule |
| canonical route가 특정 topology에 편향되지 않았는가? | canonical-event-schema ExecutionRoute |
| S1~S5와 R1~R10의 차이는 무엇인가? | AA-006 + runtime-scenario-catalog |
| 보안/신뢰성은 왜 Top 4가 아닌가? | AA-005 + requirements-vNext + evaluation strategy |
| raw evidence가 decision으로 어떻게 연결되는가? | runtime-run-provenance + run/model-call schemas + this index §21 |
| 최종 trade-off 근거는 무엇인가? | future raw/derived/gates/tactic results + ADR |

## Maintenance rule

Update this index whenever a new artifact becomes report-relevant while keeping the original artifact authoritative.

The index should answer:

> **심사위원의 질문 → 근거 문서 → 실험 데이터 → 결정**

It must remain an index, not a duplicate final report.