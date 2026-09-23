# Final Report Evidence Index

## Status

Living index for final SW Architecture review/report assembly.

DP-00 comparative characterization is complete. Final A/B/C/D selection is
intentionally deferred pending downstream architecture evidence; AA-026 is the
authoritative conditional synthesis.

The downstream DP catalog was structurally reviewed after AA-026. Use
`docs/architecture/decision-points/catalog.md` for current DP scopes and
`docs/architecture/analysis/AA-027-structural-decision-catalog-review.md` for
before/after mapping, six-question coverage, and the revised conditional
roadmap. AA-026 remains unchanged historical synthesis.

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
Benchmark / Prototype-Harness Contract
    ↓
Immutable Raw Evidence
    ↓
Python-derived Metric / Qualification Result
    ↓
Unweighted Conditional Trade-space
    ↓
Downstream Evidence → Architecture Decision / ADR
```

Use paths from this index instead of copying full reasoning into multiple documents.

---

# 0. Central vNext Spine

Primary evidence:

- `docs/requirements/requirements-vNext.md`
- `docs/architecture/qa-dp-traceability.md`
- `docs/architecture/decision-points/catalog.md`
- `docs/architecture/qa-legacy-migration.md`
- `docs/architecture/analysis/AA-027-structural-decision-catalog-review.md`
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
Canonical Benchmark + Rust Qualification Runtime
    ↓
Prototype / Smoke / Conformance Review
    ↓
Pilot / Calibration / Rule Freeze
    ↓
Final Comparative Evaluation
    ↓
Conditional Trade-space / Downstream DP Evidence
    ↓
Final Commitment / ADR
```

## 0.1 Completed DP-00 evidence chain

| Stage | Authoritative record | Evidence role |
| --- | --- | --- |
| Profile Z / Session 7.0 | retained runtime evidence referenced by AA-025 | framework/software-path overhead only |
| R1–R4 profile freeze | `docs/architecture/analysis/AA-022-dp00-realistic-execution-profile-freeze.md` | synthetic Model/Agent/Tool sensitivity contract |
| QA-03 contract | `docs/architecture/analysis/AA-023-dp00-qa03-flexibility-experiment-contract.md` | prospective evolution scenarios and role mapping |
| QA-03 campaign | `docs/architecture/analysis/AA-024-dp00-qa03-flexibility-measured-campaign.md` | official CCR: A/B 60%, C/D 80% |
| 7.2R clean rerun | `docs/architecture/analysis/AA-025-dp00-r1-r4-comparative-architecture-campaign-clean-rerun.md` | complete QA-01/02/04, structure, and P12 evidence |
| Trade-space synthesis | `docs/architecture/analysis/AA-026-dp00-tradespace-synthesis.md` | conditional guidance; final selection deferred |

Evidence identities: runtime branch head
`760679b5d542827de6aa307b87256e8a03afaed6`, runtime evidence commit
`83cc4070decdd8a4305317cbe0bca6b0f86543dd`, QA-03 measured branch head
`5e6eb627401a2e0e18736d1377ed835f9f050aed`, and QA-03 contract head
`752c6ac66fd0a77d7d6efac00d46ad7277bc5247`.

The evidence chain supports an unweighted conditional trade-space, not an
overall winner ranking. QA-02 defects and QA-04 universal qualification failure
remain visible remediation evidence.

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

- A/B/C/D executable topology diagrams;
- Responsibility Placement Matrix;
- State Ownership Matrix;
- Alternative structural-risk table;
- A vs D reviewer-facing distinction;
- B ARGO authoritative execution state vs VIA task projection;
- C = A + bounded Fast Path.

Reviewer-facing A/D distinction:

```text
A: 어느 Agent에게 맡길 것인가?
D: 어떤 execution topology 자체를 사용할 것인가?
```

B state distinction:

```text
ARGO = authoritative execution route/thread/plan/domain state
VIA  = user-facing task projection/correlation/result interaction
```

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

Base excludes optional cross-component GenAI fusion, ML/classifier/embedding routing, speculative execution, parallel inference optimization, prompt/cache optimization, partial-ASR semantic pre-routing, and semantic precomputation.

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

- S1 Local-capable;
- S2 General Agent;
- S3 Specialized Agent;
- S4 Existing-task Follow-up;
- S5 Ambiguous Referent / Clarification.

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

- component/stage separation does not automatically add QA-04 calls;
- QA-04 changes when architecture requires additional logical Generative AI decisions before Execution Route Commit;
- cross-component GenAI fusion is a later Tactic unless one Base component intrinsically owns the combined responsibility, as in B.ARGOPrimary.

---

# 8. Canonical Benchmark Contract / Experimental Boundary

Primary evidence:

- `docs/evaluation/dp00-experimental-boundary.md`
- `docs/architecture/analysis/AA-007-benchmark-contract-neutrality-review.md`

## Phase 3/4 executable qualification evidence

Primary evidence:

- `docs/architecture/analysis/AA-010-dp00-prototype-conformance-review.md`
- `prototypes/dp00/crates/bench-runner/tests/dependency_guard.rs`
- `prototypes/dp00/crates/bench-replay/tests/hidden_context.rs`
- `prototypes/dp00/crates/bench-events/tests/contract_invariants.rs`
- `prototypes/dp00/crates/bench-smoke/tests/fault_paths.rs`
- `prototypes/dp00/crates/bench-smoke/tests/smoke_matrix.rs`

PPT/report candidates:

- 20-path Positive Smoke PASS and specification ModelCall matrix;
- N1~N9 Negative Anti-gaming Test Matrix;
- MALFORMED/TIMEOUT/NO_RESPONSE/WRONG_CANDIDATE fault injection flow;
- NetworkAgent rejection → AgentRouter retry → ARGO acceptance → one route commit;
- Specification → Rust Prototype formal conformance matrix;
- Oracle isolation, route-commit boundary, external Outcome Probe and provenance safeguards;
- Pilot Readiness Gate.

Reviewer-facing evidence statement:

> 평가 공정성 원칙을 단순 문서 규칙이 아니라 executable contract tests로 검증했다.

Report-ready three-layer boundary:

```text
Layer 1 — Shared Benchmark Environment
  Scenario / fixtures / semantic replay / policy / machine / oracle
            ↓
Layer 2 — Base Implementation Rules
  deterministic facts/state/contracts
  semantic decision owned by architecture
  no optional tactic
            ↓
Layer 3 — Architecture-specific Structure
  A / B / C / D responsibility placement + state ownership + topology
```

Reviewer-facing statement:

> **동일한 사용자 상황, 동일한 semantic model behavior, 동일한 Agent/Tool behavior를 A/B/C/D에 제공하고 responsibility placement와 execution topology를 독립변수로 비교한다.**

---

# 9. Runtime / Evolution / Mandatory Gate — Three-track Evaluation

Primary evidence:

- `docs/evaluation/dp00-experimental-boundary.md`
- `docs/evaluation/evaluation-strategy.md`
- evolution schemas under `benchmark/schemas/`

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

---

# 10. Stimulus vs Evaluator-only Oracle

Primary evidence:

- `docs/evaluation/dp00-experimental-boundary.md`
- `benchmark/schemas/runtime-scenario-schema.md`
- `docs/architecture/analysis/AA-007-benchmark-contract-neutrality-review.md`
- `docs/evaluation/prototype-benchmark-harness-spec.md`

Report-ready diagram:

```text
Scenario Source
  ├─ AUT-visible Stimulus
  │    User input / raw context / logical initial state
  │    capability-health-policy facts / dependency responses
  │
  └─ Evaluator-only Oracle
       ground-truth referent/task relation
       Required / Allowed / Forbidden
       expected result binding
       success predicate / scoring eligibility
```

Key sentence:

> **Benchmark harness는 ground truth를 알고 있지만 Architecture-under-Test에는 전달하지 않는다.**

Prototype-level structural defense:

```text
bench-core   -> AUT-visible neutral types
bench-oracle -> evaluator-only types
alternative-* MUST NOT depend on bench-oracle
```

---

# 11. Runtime Scenario Manifest / R1~R10

Primary evidence:

- `benchmark/schemas/runtime-scenario-schema.md`
- `benchmark/catalog/runtime-scenario-catalog.md`

S1~S5 and R1~R10 are intentionally different:

```text
S1~S5
  = architecture specification + smoke/conformance set

R1~R10
  = runtime scoring taxonomy / initial coverage skeleton
```

R1~R10 classes:

- R1 Local / Bounded;
- R2 General Agent;
- R3 Specialized Agent;
- R4 Context / Referent;
- R5 Existing-task Follow-up;
- R6 Ambiguity / Clarification;
- R7 Execution-path Trap;
- R8 Concurrent Task / Result;
- R9 Compound Request;
- R10 Dependency / Model Error.

The initial catalog is not the final frozen scoring corpus.

---

# 12. Semantic Responsibility Replay

Primary evidence:

- `benchmark/schemas/semantic-behavior-plan-schema.md`
- `benchmark/contracts/model-replay-request-contract.md`
- `benchmark/schemas/model-call-schema.md`
- `docs/architecture/analysis/AA-007-benchmark-contract-neutrality-review.md`

Report-ready diagram:

```text
Frozen semantic condition
  INTENT_INTERPRETATION = CORRECT
  EXECUTION_ROUTE_SELECTION = WRONG
              ↓
      responsibility-keyed replay
         /                 \
A separate calls          B one ARGO call
Intent -> CORRECT         [intent, route]
Router -> WRONG           -> [CORRECT, WRONG]
         \                 /
          same semantic fault
```

Reviewer-facing statement:

> **동일한 semantic success/error condition을 responsibility 단위로 A/B/C/D에 주입하여 Model intelligence를 통제하고 SW topology의 validation/recovery 차이를 비교한다.**

---

# 13. AUT-visible ModelRequest vs Benchmark-hidden ReplayContext

Primary evidence:

- `benchmark/contracts/model-replay-request-contract.md`
- `docs/evaluation/prototype-benchmark-harness-spec.md`

Report-ready implementation boundary:

```text
AUT-visible ModelRequest
  decision_owner
  semantic_responsibilities[]
  semantic input/context
  output schema / model profile
        ↓
Benchmark-owned Replay Adapter
        ↕ hidden ReplayContext
  run / episode / scenario
  behavior plan
  semantic operation keys
  attempt state
        ↓
Frozen semantic payload
```

Key defense point:

> **Architecture는 자신이 수행할 semantic responsibility는 알지만 scenario ID, semantic operation key, behavior plan 또는 정답은 모른다.**

This closes a benchmark-key leakage risk that remained abstract in the earlier canonical contract.

---

# 14. Canonical Observation Events / ObservationPort

Primary evidence:

- `benchmark/schemas/canonical-event-schema.md`
- `benchmark/contracts/observation-port-contract.md`
- `benchmark/schemas/run-event-schema.md`
- `benchmark/schemas/model-call-schema.md`

Report-ready two-sided API:

```text
AUT
  ObservationPort.emit(semantic event + product correlation)
        ↓
Benchmark Observation Adapter
  + run / episode / scenario / alternative
  + sequence number
  + monotonic timestamp
  + emitter provenance
        ↓
Canonical Event Buffer
```

Architecture does not choose benchmark provenance or evaluator truth.

Canonical Events → QA:

```text
interaction.acoustic_eos -------┐
                                ├─ QA-01 FTOL
useful_outcome.observed --------┘

clarification / task / route / result
                                └─ QA-02 AECR evidence

model.generation.*
execution.route_committed
                                └─ QA-04 accounting
```

---

# 15. Canonical ExecutionRoute / ARGO Fixture Boundary

Primary evidence:

- `benchmark/schemas/canonical-event-schema.md`
- `benchmark/contracts/runtime-fixture-contracts.md`
- `docs/evaluation/dp00-experimental-boundary.md`

Architecture-neutral route representation:

```text
route_kind:
  LOCAL_DIRECT
  EXECUTOR_DIRECT
  EXECUTOR_DELEGATED

initial_executor_id
final_executor_id_if_known
delegation_chain[]
```

Critical B seam:

```text
B.ARGOPrimary Controller  = AUT
        ↓
Execution Route Commit
        ↓
post-route deterministic ARGO/Specialist domain fixture where comparable
```

This prevents a common stub from erasing B's architecture responsibility.

---

# 16. Benchmark Neutrality / Anti-gaming Review

Primary evidence:

- `docs/architecture/analysis/AA-007-benchmark-contract-neutrality-review.md`
- negative contract tests specified in `docs/evaluation/prototype-benchmark-harness-spec.md`

Report-ready attack/control examples:

| Attack | Control |
| --- | --- |
| Oracle leakage | separate `bench-oracle`; alternative dependency guard |
| Scenario/operation-key leakage | narrow AUT `ModelRequest`; hidden ReplayContext |
| Harness decision leakage | AUT-owned responsibility list |
| Separate vs fused replay bias | responsibility-keyed semantic behavior |
| Deterministic path forced to model | replay only on AUT ModelPort request |
| Replay erases ModelCall | 1 logical generation = 1 ModelCall |
| Responsibility metadata gaming | frozen owner→responsibility mapping |
| B ARGO stub leakage | B.ARGOPrimary remains AUT |
| QA-01 self-report | external Outcome Probe |
| Provenance spoofing | benchmark-owned Observation Adapter |
| Provisional route commit | route-commit coherence validation |
| Tactic contamination | Base vs Tactic separation |

Design-readiness verdict from AA-007:

```text
Canonical Benchmark Contract  = PASS WITH TBD
Architecture Neutrality       = PASS
Experimental Boundary         = PASS
Prototype/Harness Readiness   = PASS
```

---

# 17. Qualification Runtime Language Decision

Primary evidence:

- `docs/architecture/analysis/AA-008-qualification-runtime-language-and-harness-rationale.md`
- `docs/evaluation/prototype-benchmark-harness-spec.md`

Final decision:

```text
Architecture Qualification Runtime = Rust
Offline Analysis                    = Python
```

Report-ready responsibility diagram:

```text
Rust
  A/B/C/D AUT
  Runner / Replay / Fixtures / Outcome Probe
  Canonical Event + ModelCall capture
  Monotonic timed path
  Immutable raw evidence
            ↓
results/raw/
            ↓
Python
  validation / FTOL / AECR / QA-04 / QA-03
  statistics / calibration / sensitivity
  report tables / plots
```

Reviewer-facing sentence:

> **Architecture-under-Test와 QA-01 timed path는 production implementation language인 Rust로 구현하고, Python은 측정 종료 후 immutable raw evidence의 분석에만 사용한다.**

Correct interpretation:

> Rust qualification results support a **controlled relative architecture comparison**, not a claim that prototype FTOL equals absolute production VIA latency.

---

# 18. Why not Python-only?

Primary evidence:

- `AA-008` candidate comparison.

Report-ready candidate table:

| Candidate | Main issue / conclusion |
| --- | --- |
| Python-only | interpreter/scheduling substrate can distort topology-dependent elapsed timing; QA-03 measured on disposable language structure |
| Python + separate Rust timing prototype | runtime/correctness evidence comes from different implementations; fidelity drift risk |
| **Rust runtime + Python analysis** | **selected: one AUT for QA-01/02/04 + same Rust source for QA-03; analysis remains productive** |
| Rust-only | little validity gain from implementing statistics/plots in Rust |

Do not claim “Rust is always faster” as the rationale. The rationale is control of implementation substrate and consistency across QAs.

---

# 19. Rust Qualification Runtime / Timed Path

Primary evidence:

- `AA-008`
- `prototype-benchmark-harness-spec.md`

Inspected reference convention:

```text
Rust edition 2024
rust-version 1.94
Tokio 1.x
```

This is the starting convention, subject to exact implementation freeze.

Important caveat: the reference release profile is size-oriented; timed qualification must use a **frozen optimized performance-comparison release-class profile**, not blindly copy size-oriented flags.

Report-ready timed boundary:

```text
Rust Scenario Driver
    ↓
AUT + Replay / Fixtures / Outcome Probe
    ↓
in-memory timestamp/event append
--- QA-01 timed window ends ---
JSON/JSONL serialization / file flush
    ↓
Python offline analysis
```

Python, JSON serialization, file I/O, report generation and external analysis RPC are excluded from the QA-01 timed path.

---

# 20. Prototype Rust Module / Crate Boundary

Primary evidence:

- `docs/evaluation/prototype-benchmark-harness-spec.md`

Report-ready conceptual workspace:

```text
prototypes/dp00/crates/
  bench-core
  bench-events
  bench-fixtures
  bench-replay
  bench-runner
  bench-oracle
  alternative-a
  alternative-b
  alternative-c
  alternative-d
```

Key rule:

> **공통화할 것은 실험 인프라이지 Architecture decision responsibility가 아니다.**

Architecture decision logic not extracted into common implementation:

- Intent Refiner;
- Agent Router;
- Execution Path Selector;
- Fast Eligibility;
- Task-association decision logic;
- ARGO Primary Controller.

QA-03 source-role classification:

```text
EVALUATION_SUPPORT
ARCHITECTURE_UNDER_TEST
```

is frozen so fixture/harness edits do not become false architecture propagation evidence.

---

# 21. AUT Public Boundary and External Ports

Primary evidence:

- `prototype-benchmark-harness-spec.md`

Common AUT boundary represents only product interaction:

```text
ArchitectureUnderTest
  setup
  handle_user_turn
  handle_cancel
  teardown
```

The common interface does not contain `interpret_intent`, `select_agent`, `select_execution_path`, or `fast_eligible`, because B does not share A/C/D internal decomposition.

Neutral external ports:

```text
ModelPort
CapabilityRegistryPort
HealthPort
PolicyPort
DomainExecutorPort
ToolPort
ObservationPort
Clock
```

These provide facts/fixtures, not architecture decisions.

---

# 22. S1~S5 Smoke + Prototype Conformance Review

Primary evidence:

- `AA-006`
- `prototype-benchmark-harness-spec.md`
- `docs/architecture/analysis/AA-009-specification-branch-readiness-review.md`

Implementation lifecycle:

```text
Specification
    ↓
Rust Prototype
    ↓
S1~S5 × A/B/C/D = 20 smoke paths
    ↓
Prototype Conformance Review
    ↓
PASS
    ↓
Pilot
```

Smoke verifies route/state/clarification/event order/ModelCall accounting/oracle isolation but produces no star score.

Report-ready negative tests include:

- Oracle crate dependency leakage;
- scenario-id / replay-operation-key leakage;
- semantic responsibility violation;
- B ARGO responsibility leakage;
- premature / duplicate initial route commit;
- fake useful-outcome self-report;
- observation provenance spoofing.

---

# 23. Raw Evidence → Python Analysis → Decision

Primary evidence:

- `benchmark/schemas/runtime-run-provenance-schema.md`
- `prototype-benchmark-harness-spec.md`
- `run-event-schema.md`
- `model-call-schema.md`

Recommended raw structure:

```text
results/raw/<run-id>/
  provenance.json
  canonical-events.jsonl
  model-calls.jsonl
  fixture-events.jsonl     # optional
```

Preferred v0 serialization:

```text
Scenario / Behavior Plan  -> JSON
Provenance                -> JSON
Canonical / ModelCall     -> JSONL
```

Decision trace:

```text
Rust immutable Raw Evidence
    ↓
Python derived metrics / validation
    ↓
QA Primary Metric
    ↓
0–5 Score / Gate
    ↓
DP-00 Trade-off
    ↓
Optional Tactic delta
    ↓
ADR
```

Changing Python derivation/scoring logic does not rewrite raw Rust evidence.

Session 4.1 diagnostic evidence:

- `AA-013-dp00-path-aware-evidence-readiness-review.md`;
- `benchmark/contracts/pilot-v0-constraint-alternative-evidence-map.json`;
- `results/reports/pilot-v0/official-1789087890289040000-invalidation.json`;
- `results/reports/pilot-v0/official-1789087890289040000-sha256-inventory.json`.

Report-ready narrative:

```text
Static constraint-level audit
        ↓
Official Pilot exposed topology-specific evidence gap
        ↓
constraint × alternative coverage gate introduced
        ↓
missing evidence prevented from becoming silent correctness score
```

The invalidated campaign's metric values are diagnostic only and must not be
indexed as architecture results.

---

# 24. Scored QA vs Mandatory Gates

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

# 25. Threats to Validity

| Threat | Existing control/evidence |
| --- | --- |
| Replay fidelity | Actual-model / Real-stack validation track |
| Stub vs real Agent | deterministic qualification + real Agent validation |
| Scenario representativeness | R1~R10 skeleton + future frozen corpus review |
| Workload weighting | class/tags/raw preservation; aggregation frozen after Pilot |
| Model-profile dependency | frozen profile/version + ModelCall telemetry |
| Hardware/runtime dependency | common Rust runtime/toolchain/reference environment |
| Python/interpreter timed-path contamination | Python excluded from Rust timed path |
| Prompt/cache optimization | frozen profiles + Tactic separation |
| External dependency latency | QA-01 equality + non-dominance |
| Prototype fidelity | executable spec + Prototype Conformance Review |
| Instrumentation overhead | in-memory timed telemetry + Pilot overhead check |
| Oracle leakage | crate/API separation + negative test |
| Scenario/replay-key leakage | hidden ReplayContext |
| Oracle neutrality | QA-02 Required/Allowed/Forbidden |
| Evolution-boundary gaming | pre-frozen Expected Change Roles/source-role mapping |
| Hidden-routing gaming | Execution Route Commit |
| Responsibility metadata gaming | owner→responsibility validation |
| Useful-outcome self-report | external Outcome Probe |
| B ARGO stub leakage | B.ARGOPrimary pre-route stays AUT |
| Optimization maturity bias | Base vs Tactic separation |
| Threshold hindsight | Pilot → calibration → freeze → final |

Residual threats remain reportable after experimentation; Rust does not make them disappear.

---

# 26. Specification Branch Readiness

Primary evidence:

- `docs/architecture/analysis/AA-009-specification-branch-readiness-review.md`

Current pre-implementation verdict:

```text
DP-00 Definition Complete                = PASS
Top QA Definition Complete               = PASS
Executable Architecture Spec Complete    = PASS
Benchmark Contract Complete              = PASS WITH PILOT-TIME TBDs
Prototype/Harness Spec Complete          = PASS
Open Architecture-level Blocker          = NONE IDENTIFIED

Specification Branch Readiness           = PASS
```

This is readiness for human review and implementation, **not** benchmark success or DP-00 selection.

Recommended workflow:

```text
specification branch
    ↓ user consistency review
merge to main if accepted
    ↓
separate implementation branch
    ↓
Rust Prototype / Harness
```

---

# 27. Final Trade-off / Decision Material

The comparative A/B/C/D evidence and conditional trade-space synthesis now
exist. No final topology has been selected.

Completed report assets include:

- DP-00 problem statement;
- A/B/C/D conceptual + executable topology;
- v1.1 compatibility table;
- Responsibility Placement Matrix;
- State Ownership Matrix;
- Top QA + Primary Metric table;
- DP-00 × QA Sensitivity Matrix;
- measured R1–R4 QA-01 sensitivity and structural topology totals;
- deterministic QA-02 conformance and remediation patterns;
- measured QA-03 CCR and scenario-level containment evidence;
- comparative QA-04 qualification behavior with universal `FAIL` preserved;
- P12 common-invariant evidence;
- Base Architecture vs Tactic distinction;
- S1~S5 20-path walkthrough + expected ModelCall trace;
- Runtime/Evolution/Mandatory Gate three-track diagram;
- Shared/Controlled vs Architecture-owned boundary;
- Stimulus vs Evaluator-only Oracle;
- Semantic Responsibility Replay diagram;
- AUT ModelRequest vs hidden ReplayContext;
- Canonical ObservationPort enrichment diagram;
- Canonical Events → QA mapping;
- Rust Qualification Runtime + Python Analysis diagram;
- Python-only vs hybrid language rationale table;
- QA-01 Timed Path boundary;
- Rust workspace/crate boundary;
- S1~S5 and P01–P12 executable conformance evidence;
- negative anti-gaming contract-test table;
- Raw → Python Analysis → unweighted QA vector → conditional decision pipeline;
- Architecture Qualification vs Real-stack validity distinction;
- Threats-to-Validity checklist;
- Specification Branch Readiness verdict;
- AA-026 conditional recommendation matrix and downstream DP roadmap.

Remaining future work:

- tactic experiment results — **TBD**;
- Real-stack validation — **TBD**;
- DP-13 all-family state/recovery investigation, DP-16 conditional
  hosting/isolation, and other discriminating downstream evidence — **TBD**;
- final conditional trigger resolution and A/B/C/D commitment — **TBD**;
- ADR — **TBD**.

---

# 28. Official Runtime Pilot v0 / Calibration Freeze Readiness

Primary report-ready analysis:

- `docs/architecture/analysis/AA-014-dp00-runtime-pilot-v0-calibration-review.md`

Authoritative evidence:

- valid Official raw campaign: `results/raw/pilot-v0/official-1789090497263081000/`;
- original committed `dp00-analysis-v1`: `results/derived/pilot-v0/official-1789090497263081000/analysis-summary.json`;
- deterministic semantic-equivalent `dp00-analysis-v2`: `results/derived/pilot-v0/official-1789090497263081000/dp00-analysis-v2/analysis-summary.json`;
- reproducible report tables: `results/reports/pilot-v0/calibration-review-data.json`;
- invalidated forensic campaign: `official-1789087890289040000` plus its external invalidation/inventory records;
- path-aware gate: `benchmark/contracts/pilot-v0-constraint-alternative-evidence-map.json` (`dp00-path-aware-evidence-v1`).

Report-ready assets in AA-014:

| Asset | Evidence preserved |
| --- | --- |
| Official Pilot Configuration Table | source/campaign/corpus/runtime/profile/order/instrumentation/schema identities |
| Validity Gate Table | 80 episodes; Z 40/C 40; missing/duplicate 0; MISSING/UNEVALUABLE 0; 144/144 |
| QA-01 tables | Profile × Alternative descriptive statistics and estimator sensitivity |
| QA-02 tables | AECR, constraint status, non-conformance, P04/P06/P07/P09 detail |
| QA-04 tables | overall/macro, no-route rates, class shares and contributions |
| Methodology timeline | invalidated attempt → forensic preservation → predicate-gap fix → valid Official Pilot |
| Calibration Decision Table | explicit `FREEZE` / `DEFER` / `REJECT` decisions without architecture selection |
| Threats to Validity | small-N/order/profile/instrumentation/coverage/gate/no-route/aggregation limitations |
| Freeze Readiness Table | independent `benchmark-v1`, `aggregation-v1`, `scoring-v1`, `gate-v1` readiness |
| Pre-experiment traceability | sensitivity hypotheses compared with observed Pilot behavior, never used as winner logic |

Current measurement-system conclusion:

```text
Official evidence integrity       PASS
Derived byte determinism          PASS (`dp00-analysis-v2`)
Metric semantic preservation      PASS (QA-01/02/04)
Final benchmark freeze            NOT READY
Required path                     targeted calibration → version freeze → Final Evaluation
```

No Best Architecture, winner, recommendation, final DP-00 decision or 0–5 score is established by this evidence.

---

# 29. DP-00 Targeted Calibration Follow-up — R8/R9 and QA-04 Contract

Primary methodology evidence:

- `docs/architecture/analysis/AA-015-dp00-r8-r9-and-qa04-contract-policy.md`;
- corpus v0.2 P11/R8 and P12/R9 assets under `benchmark/`;
- `benchmark/contracts/pilot-v0.2-constraint-alternative-evidence-map.json` (`dp00-path-aware-evidence-v2`);
- `benchmark/contracts/qa04-route-contract-policy-v1.json`;
- `dp00-analysis-v3` candidate-only `qa04_contract_diagnostics`.

The Pilot exposed incomplete R1–R10 coverage and large no-route populations.
This follow-up adds R8/R9 from the pre-existing taxonomy, makes route contracts
explicit, rejects arbitrary no-route penalties, and introduces a contract-aware
comparability qualification so cheap required-route failure cannot look
efficient. This is methodology evidence, not an A/B/C/D ranking. Existing
Official v0.1 raw and derived evidence remains unchanged and continues to use
the 36×4 v1 manifest.

Current conclusion:

```text
R1–R10 coverage                  PASS
v0.2 path-aware gate             PASS (188/188; MISSING 0; UNEVALUABLE 0)
QA-04 contract taxonomy          FREEZE
arbitrary no-route penalty       REJECT
OPTIONAL and overall vs macro    DEFER
Final benchmark/version freeze   NOT READY
```

---

# Reviewer Question → Evidence Map

| Reviewer question | Primary evidence |
| --- | --- |
| 왜 DP-00이 중요한가? | AA-001 + DP-00 + requirements-vNext |
| 왜 A/B/C/D인가? | DP-00 conceptual + executable spec |
| 실제 구현 구조는 어떻게 다른가? | executable spec Responsibility/State matrices |
| D와 A의 차이는 무엇인가? | executable spec + AA-006 |
| B execution state는 누가 소유하는가? | executable spec + AA-006 |
| 왜 이 4개 QA인가? | AA-005 sensitivity/cross-review |
| component가 많으면 QA-04가 불리한가? | AA-005 + Base Decision Mechanism rule |
| optimization이 특정 Alternative에 섞이지 않았는가? | base-architecture-vs-tactic-evaluation.md |
| 무엇이 controlled이고 무엇이 architecture variable인가? | dp00-experimental-boundary.md |
| Ground truth가 AUT에 새지 않는가? | runtime-scenario schema + prototype spec + AA-007 |
| LLM randomness를 어떻게 통제했는가? | Semantic Behavior Plan + replay contract |
| separate/fused topology에 동일 error를 어떻게 주입하는가? | replay contract + AA-007 |
| AUT가 benchmark operation/scenario key를 볼 수 있는가? | refined model-replay contract: **NO** |
| canonical event provenance를 AUT가 조작할 수 있는가? | observation-port contract: **NO** |
| benchmark가 B ARGO responsibility를 stub으로 지우지 않는가? | runtime-fixture contract + prototype spec |
| QA-01 useful outcome을 AUT가 조작할 수 없는가? | Outcome Probe + observation contract |
| 왜 runtime을 Rust로 구현하는가? | AA-008 candidate comparison |
| 왜 Python은 timed path에 없는가? | AA-008 + prototype harness spec |
| Rust prototype FTOL이 production latency인가? | **NO** — controlled relative comparison; AA-008 |
| 어떤 Rust runtime/toolchain을 쓰는가? | reference convention + implementation freeze; AA-008/provenance |
| instrumentation이 latency를 왜곡하지 않는가? | in-memory telemetry + Pilot overhead check |
| Official Pilot evidence는 완전한가? | AA-014 Validity Gates + valid campaign v2 derived evidence |
| 실패한 Official attempt를 어떻게 처리했는가? | AA-013 + AA-014 invalidated→valid methodology timeline |
| 동일 raw의 derived JSON이 byte-stable한가? | AA-014 determinism gate + `dp00-analysis-v2` |
| QA-01 estimator/repetition/profile을 고정할 수 있는가? | AA-014 QA-01 calibration + decision table |
| QA-02 threshold를 Pilot 점수로 정했는가? | **NO** — AA-014 numeric gate `DEFER` |
| QA-04 cheap failure를 어떻게 방지하는가? | AA-014 no-route candidate analysis; final policy `DEFER` |
| benchmark/aggregation/scoring/gate version이 freeze-ready인가? | AA-014 Freeze Readiness Table |
| S1~S5 implementation gate는 무엇인가? | prototype harness spec |
| anti-gaming 원칙을 executable test로 만들었는가? | prototype harness spec negative tests |
| QA-03에서 benchmark code change가 섞이지 않는가? | EVALUATION_SUPPORT vs ARCHITECTURE_UNDER_TEST source roles |
| spec branch가 구현 준비되었는가? | AA-009 Specification Branch Readiness = PASS |
| 보안/신뢰성은 왜 Top 4가 아닌가? | AA-005 + requirements-vNext + evaluation strategy |
| raw evidence가 decision으로 어떻게 연결되는가? | runtime provenance + schemas + prototype spec |
| 최종 trade-off 근거는 무엇인가? | future raw/derived/gates/tactic results + ADR |

## Maintenance rule

Update this index whenever a new artifact becomes report-relevant while keeping the original artifact authoritative.

The index should answer:

> **심사위원의 질문 → 근거 문서 → 실험 데이터 → 결정**

It must remain an index, not a duplicate final report.
