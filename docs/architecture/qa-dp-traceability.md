# QA ↔ Decision Point Traceability — vNext Working

## Status

Central architecture navigation for the **vNext working baseline**.

`docs/requirements/requirements-v1.1.md` remains the Approved Baseline. This file does not rewrite that baseline; it reclassifies architecture-analysis navigation around DP-00 and the vNext Top Architectural Drivers.

## Terminology: Top QA vs Legacy v1.1 Detailed QA

The IDs `QA-01`~`QA-04` now refer to the **vNext Top Architectural Driver QAs** in central vNext architecture work.

The QA IDs embedded in `requirements-v1.1.md` remain valid historical Approved Baseline IDs and are referred to here as:

```text
Legacy v1.1 QA-01
...
Legacy v1.1 QA-11
```

This disambiguation is mandatory when both generations are discussed in the same document.

## Central traceability storyline

```text
Product / Architecture Concern
    ↓
DP-00 — VIA Primary Execution Boundary
    ↓
A / B / C / D Alternatives
    ↓
Top QA-01 ~ QA-04
    ↓
Primary Metrics
    ↓
Controlled Architecture Qualification
    ↓
Pilot / Calibration / Scoring & Gate Freeze
    ↓
Final A/B/C/D Evaluation
    ↓
Trade-off / ADR
```

## DP-00 — top-level Decision Point

### Reviewer-facing question

> **VIA는 사용자 요청을 어디까지 직접 판단·실행하고, 어디부터 Downstream Agent에 위임할 것인가?**

English: **VIA Primary Execution Boundary**

Authoritative definition:

- `docs/architecture/decision-points/DP-00-primary-execution-boundary.md`
- rationale: `docs/architecture/analysis/AA-001-qa-rebaseline-and-primary-execution-boundary.md`

### DP-00 alternatives

| Alternative | Structural summary | v1.1 Boundary Compatibility |
| --- | --- | --- |
| **A — Thin VIA / Agent-neutral Orchestration** | VIA owns context/intent/routing/task orchestration; substantive execution is delegated. | **Compatible** |
| **B — ARGO-centric Primary Execution** | Voice/S2S → thin realtime context → ARGO primary ReAct/tool runtime → specialist delegation when required. | **Challenges baseline boundary** |
| **C — Hybrid VIA Fast Path** | Bounded/local-safe capabilities may execute in VIA; substantive work remains delegated. | **Mostly compatible / extension** |
| **D — Adaptive Per-turn Execution** | Common control plane selects Fast/ARGO/Specialized-Agent topology per turn with directed ownership transfer. | **Partially compatible / extension likely** |

Alternative B is intentionally a **responsibility-boundary challenging alternative**. It must not be reduced to “ARGO is the preferred/default Downstream Agent,” which is only a routing-policy variant under a Thin/Adaptive topology.

Fast Path eligibility is semantic; arbitrary definitions such as `<3 seconds` or `1 LLM + 1 tool` are not authoritative eligibility rules.

## Top Architectural Drivers

| Top QA | Reviewer-facing question | Primary Metric | DP-00 structural sensitivity | Authoritative QA |
| --- | --- | --- | --- | --- |
| **QA-01 Fast-task End-to-End Responsiveness** | **빠른가?** | **Fast-task Outcome Latency p95 (FTOL p95)** | synchronous stages, delegation depth, Fast Path and execution-path hops | `docs/evaluation/quality-attributes/QA-01-fast-task-e2e-responsiveness.md` |
| **QA-02 VIA Interaction-Orchestration Correctness** | **정확한가?** | **Architecture Episode Exact Conformance Rate (AECR)** | context/task association, eligibility, routing/delegation, clarification, result binding | `docs/evaluation/quality-attributes/QA-02-via-interaction-orchestration-correctness.md` |
| **QA-03 변경 대응 용이성 (Flexibility)** | **변경이 잘 격리되는가?** | **Change Containment Rate (CCR)** | responsibility boundaries, adapters, coupling, extension points | `docs/evaluation/quality-attributes/QA-03-change-flexibility.md` |
| **QA-04 모델 호출 오버헤드** | **실행 경로를 정하기 위해 AI 판단을 얼마나 요구하는가?** | **Average Model Calls to Commit Execution Route** | model-based intent/routing/validation/delegation topology before final route commit | `docs/evaluation/quality-attributes/QA-04-model-call-overhead.md` |

DP-00 is directly evaluated by all four Top QAs:

```text
          QA-01
            ↕
QA-03  ↔ DP-00 ↔  QA-02
            ↕
          QA-04
```

The detailed pre-experiment structural sensitivity analysis is authoritative in:

- `docs/architecture/analysis/AA-005-top-qa-cross-review.md` §9.3 — **DP-00 × QA Sensitivity Matrix**

That matrix is a **pre-experiment architectural sensitivity hypothesis**, not an A/B/C/D result.

## Scored QAs vs Mandatory Qualification Gates

```text
Scored Architectural Drivers
  QA-01
  QA-02
  QA-03
  QA-04

Mandatory Qualification Constraints / Gates
  Security
  Privacy / Context-sharing policy
  Trusted boundary requirements
  Required cancellation semantics
  Required task-state integrity
  Failure containment
  Mandatory recovery behavior
```

Security/reliability concerns are not downgraded by this separation. Selected obligations are non-compensable acceptance conditions and therefore should not be hidden inside a trade-off star score.

A separate minimum QA-02 correctness eligibility gate is also recommended:

```text
QA-02 AECR < frozen minimum acceptable threshold
    -> alternative is ineligible for final DP-00 selection
```

The numeric gate remains **TBD** until Pilot/calibration and must be frozen before final results.

## Legacy v1.1 Detailed QA → vNext classification

The legacy QA set remains authoritative as part of v1.1. The following table describes its role in vNext evaluation/navigation; it does not change the approved legacy definitions or targets.

| Legacy v1.1 Detailed QA | vNext classification | Relationship |
| --- | --- | --- |
| **Legacy QA-01 — VIA software processing latency** | **Top QA-01 Secondary / diagnostic** | VIA-owned software overhead decomposes FTOL and helps explain architecture-induced latency. |
| **Legacy QA-02 — concurrent-task capacity & PC co-existence** | **Detailed operational architecture QA / supporting constraint** | Capacity, resource arbitration and foreground co-existence remain important but are not one of DP-00's four scored top drivers. |
| **Legacy QA-03 — conversation/task recovery** | **Reliability / Mandatory Qualification candidate** | Recovery obligations are non-compensable where required by the baseline. |
| **Legacy QA-04 — dependency failure containment** | **Reliability / Mandatory Qualification candidate** | Failure isolation/containment should be a gate where violation makes an alternative unacceptable. |
| **Legacy QA-05 — changeability & integration** | **Top QA-03 detailed / Secondary evidence** | Existing modifiability/replaceability/interoperability concerns contribute to Flexibility diagnosis. |
| **Legacy QA-06 — external Context minimization** | **Privacy/Security supporting QA / Mandatory Qualification candidate** | Context minimization and egress policy are privacy/trusted-boundary obligations. |
| **Legacy QA-07 — Model/Token cost efficiency** | **Top QA-04 Secondary / derived resource metric** | Token/cache/provider cost remains diagnostic; logical route-decision call count is the Top QA Primary. |
| **Legacy QA-08 — observability & audit overhead** | **Cross-cutting supporting QA** | Analysability/accountability and instrumentation overhead support all evaluation tracks. |
| **Legacy QA-09 — screen/pointer referent-binding accuracy** | **Top QA-02 Secondary correctness slice** | Referent binding is one canonical correctness dimension. |
| **Legacy QA-10 — user-request / existing-new task association correctness** | **Top QA-02 Secondary correctness slice** | Task relation/id and follow-up association are canonical correctness dimensions. |
| **Legacy QA-11 — Downstream Agent routing correctness** | **Top QA-02 Secondary correctness slice** | Agent/execution-owner routing conformance is a canonical correctness dimension. |

## Decision Point hierarchy

DP-00 does **not** delete or invalidate DP-01~DP-12. It is the higher-order responsibility/placement decision that constrains or reshapes their alternatives and interpretation.

Examples:

- Intent Refiner existence/depth depends on how much semantic authority remains in VIA.
- Agent Router placement/importance depends on whether routing occurs in VIA, ARGO, or an adaptive selector.
- Fast Path scope depends on the selected responsibility boundary.
- ARGO may be a peer Agent, preferred route, per-turn owner, or primary runtime depending on DP-00.
- Task/context ownership and handoff semantics depend on where primary execution authority resides.

The existing lower-level DP documents remain open unless separately decided.

## DP ↔ Top-QA working map

`Related Legacy QA` refers to the v1.1 QA IDs as originally used by the older central traceability. `Top QA` is the vNext driver mapping.

| DP | Problem | Relationship to DP-00 | Related FR | Top QA | Related Legacy v1.1 QA | Status |
| --- | --- | --- | --- | --- | --- | --- |
| **DP-00** | **VIA Primary Execution Boundary** | **Top-level** | UC-01/04/05/07/10/11/12/13/16; FR-06~11/27/35/37/40~44 areas | **QA-01, QA-02, QA-03, QA-04** | QA-01/03/05/06/07/09/10/11 and supporting QA-02/04/08 | Alternatives Defined; evaluation QAs formalized; Pilot TBD |
| DP-01 | Partial / streaming input processing | constrained by owner/semantic-placement choice | FR-01, FR-03, FR-04 | QA-01, QA-02, QA-04 | QA-01, QA-09, QA-10 | Open |
| DP-02 | Interaction context representation | context consumer/authority depends on DP-00 | FR-03, FR-04 | QA-01, QA-02, QA-03 | QA-01, QA-09 | Open |
| DP-03 | Capability placement boundary | directly refined/subsumed by DP-00 placement decision | FR-11, FR-42 | QA-01, QA-02, QA-03, QA-04 | QA-01, QA-05, QA-07 | Open; alternatives may need reshaping after DP-00 |
| DP-04 | Generic vs specialized Downstream Agent integration | Agent boundary changes with DP-00 owner model | FR-11, FR-43 | QA-02, QA-03, QA-04 | QA-01, QA-05, QA-07 | Open |
| DP-05 | Concurrent task resource arbitration | task/execution ownership constrains scheduler authority | FR-40, FR-41 | QA-01, QA-02 | QA-02, QA-03 | Open |
| DP-06 | Voice runtime composition & ASR side channel | semantic work placement/early processing depends on DP-00 | FR-01, FR-03, FR-44 | QA-01, QA-02, QA-03, QA-04 | QA-01, QA-05, QA-09 | Open |
| DP-07 | Model gateway selection policy | model stages/profiles depend on owner topology | FR-44 | QA-01, QA-03, QA-04 | QA-01, QA-05, QA-07 | Open |
| DP-08 | Voice/Text interaction unification boundary | common turn/control-plane ownership depends on DP-00 | FR-37, FR-39 | QA-01, QA-02, QA-03 | QA-03, QA-05, QA-10 | Open |
| DP-09 | Existing-task vs new-task association | association authority/state owner depends on DP-00 | FR-27, FR-37, FR-43 | QA-02, QA-04 | QA-03, QA-10 | Open |
| DP-10 | Intent refinement architecture | existence/depth/placement directly constrained by DP-00 | FR-06~10 | QA-01, QA-02, QA-03, QA-04 | QA-01, QA-07, QA-09, QA-10 | Open |
| DP-11 | Downstream Agent routing architecture | router may live in VIA, ARGO or adaptive selector | FR-11, FR-35 | QA-01, QA-02, QA-03, QA-04 | QA-01, QA-05, QA-11 | Open |
| DP-12 | Downstream Agent capability contract | contract role varies with route/owner topology | FR-11, FR-35, FR-43 | QA-02, QA-03 | QA-04, QA-05, QA-11 | Open |

## Evaluation and benchmark traceability

| Top QA | Primary raw evidence | Primary derived metric | Key neutrality / anti-gaming control |
| --- | --- | --- | --- |
| QA-01 | `benchmark/schemas/run-event-schema.md` | FTOL p95 | controlled dependency latency must satisfy **equality + non-dominance** |
| QA-02 | `run-event-schema.md` + `scenario-constraint-schema.md` | AECR | Required/Allowed/Forbidden constraints derived from semantics/requirements, not topology |
| QA-03 | `evolution-scenario-schema.md` + `evolution-run-schema.md` | CCR | Expected Change Roles + per-alternative mapping frozen before implementation/result observation |
| QA-04 | `run-event-schema.md` + `model-call-schema.md` | Average Model Calls to Commit Execution Route | authoritative boundary = **Execution Route Commit**, preventing hidden-routing gaming |

Common derivation chain:

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
Architecture Decision / ADR
```

## Report-ready evidence

- Top-QA independence, sensitivity, anti-gaming review: `docs/architecture/analysis/AA-005-top-qa-cross-review.md`
- report assembly/navigation: `docs/architecture/analysis/final-report-evidence-index.md`
- evaluation methodology: `docs/evaluation/architecture-experiment-methodology.md`
- central evaluation pipeline: `docs/evaluation/evaluation-strategy.md`

## Current unresolved items

- DP-00 winner: **TBD**
- QA-01~04 scoring thresholds: **TBD**
- QA-02 minimum correctness gate: **TBD**
- QA-01 deterministic dependency-latency profile: **TBD**
- QA-03 final evolution corpus/role vocabulary: **TBD**
- QA-04 final workload taxonomy and aggregation rule: **TBD**
- final benchmark corpora: **TBD**
- executable mandatory qualification gates: **TBD**
- A/B/C/D executable architecture specifications/prototypes: **TBD**
