# QA ↔ Decision Point Traceability — vNext Working

## Status

Central architecture navigation for the **vNext working baseline**. DP-00
comparative experimentation and trade-space characterization are complete;
final A/B/C/D selection is intentionally deferred pending discriminating
downstream architecture evidence.

The reviewed downstream catalog is authoritative in
`docs/architecture/decision-points/catalog.md`; its before/after rationale is
`docs/architecture/analysis/AA-027-structural-decision-catalog-review.md`.

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

The detailed generation mapping and preservation rules are maintained once in
`docs/architecture/qa-legacy-migration.md`.

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
Completed A/B/C/D Comparative Evaluation
    ↓
Conditional Trade-space / Downstream DP Evidence
    ↓
Final Commitment / ADR when decision conditions are concrete
```

## DP-00 — top-level Decision Point

### Reviewer-facing question

> **VIA는 사용자 요청을 어디까지 직접 판단·실행하고, 어디부터 Downstream Agent에 위임할 것인가?**

English: **VIA Primary Execution Boundary**

Authoritative definition:

- `docs/architecture/decision-points/DP-00-primary-execution-boundary.md`
- rationale: `docs/architecture/analysis/AA-001-qa-rebaseline-and-primary-execution-boundary.md`
- final comparative synthesis: `docs/architecture/analysis/AA-026-dp00-tradespace-synthesis.md`

Precise state:

> **DP-00 TRADE SPACE CHARACTERIZED — FINAL SELECTION DEFERRED PENDING DOWNSTREAM ARCHITECTURE EVIDENCE**

QA-01 and QA-02 comparative evaluation, QA-03 flexibility measurement, and
QA-04 comparative evaluation are complete. Deterministic QA-02 defects and
universal QA-04 qualification failures remain explicit remediation work; they
do not make the comparative evidence incomplete.

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

The legacy QA set and its approved targets remain authoritative in v1.1.
Current navigation separates:

1. the four scored Top QAs;
2. retained mandatory/supporting obligations, including recovery, failure
   containment, privacy, cancellation, task-state integrity, resource
   co-existence, and audit;
3. retained diagnostics/correctness slices whose approved targets are not
   erased by Top-QA classification; and
4. historical identifier explanation and superseded duplicate prose.

Only a future executable vNext gate encoding can remain TBD; the Approved
Baseline obligation is not optional. See
`docs/architecture/qa-legacy-migration.md` for the complete mapping.

## Decision Point hierarchy

DP-00 is the parent responsibility/ownership decision. The catalog review did
not preserve every preliminary row automatically: it kept, reframed, split,
absorbed, or demoted each item using an explicit structural test.

Logical ownership, process/runtime hosting, common execution interface, state
and recovery authority, registration authority, request-time routing, lifecycle
control, and failure containment are separate axes. See the authoritative
catalog and AA-027 for exact mappings and conditional applicability.

## DP ↔ Top-QA working map

Legacy and mandatory/supporting mappings are maintained in
`docs/architecture/qa-legacy-migration.md`. `Top QA` below is the current driver
mapping.

| DP | Current structural scope | Relationship to DP-00 | Top QA | Status |
| --- | --- | --- | --- | --- |
| **DP-00** | **Primary execution ownership/topology** | **Parent A/B/C/D** | **01/02/03/04** | **Characterized; selection deferred** |
| DP-01 | Speculative input artifact ownership and commit/cancel | owner varies by family | 01/02/04 | Open |
| DP-02 | Interaction-evidence authority and transfer | all families | 01/02/03 | Open |
| DP-03 | Capability Placement Boundary | Ownership question absorbed into DP-00 | 01/02/03/04 | **Inactive; historical identity preserved** |
| DP-04 | Execution-contract unification boundary | C/D primarily; Agent commonality A/B | 01/02/03 | Open |
| DP-05 | Cross-task resource arbitration authority | where conflicting concurrency exists | 01/02/03 | Open |
| DP-06 | Voice runtime composition boundary | all families | 01/02/03/04 | Open |
| DP-08 | Voice/Text canonical-turn boundary | all families | 01/02/03 | Open |
| DP-09 | Existing-task association authority | all; strongest for D | 02/04 | Open |
| DP-10 | Intent-refinement responsibility decomposition | family-constrained | 01/02/03/04 | Open |
| DP-11 | Eligibility/semantic selection/route-commit split | owner fixed by DP-00 | 01/02/03/04 | Open |
| DP-12 | Capability catalog/registration authority | all families | 02/03 | Open |
| DP-13 | Task projection/executor state reconciliation and recovery | all; strongest for B/D | 02/03 | Open; recommended next investigation |
| DP-14 | Lifecycle command/cancellation authority | all families | 01/02/03 | Open |
| DP-15 | Failure containment/resilience authority | all families | 02/03 | Open |
| DP-16 | VIA-local executor hosting/isolation | C/D conditional | 01/03 | Open; next C/D-specific investigation |

DP-07 is `POLICY_OR_CONFIGURATION_NOT_STANDALONE_DP`. Model/provider choice,
timeout, retry count/backoff/profile, and ranking values remain governed by FR-44/AP-02 unless a
future structural policy-authority proposal passes the admission test.

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
Derived QA Metric / Qualification Result
    ↓
Unweighted Comparative Vector
    ↓
Conditional DP-00 Trade-space
    ↓
Discriminating Downstream DP Evidence
    ↓
Architecture Decision / ADR when priorities are explicit
```

## Report-ready evidence

- Top-QA independence, sensitivity, anti-gaming review: `docs/architecture/analysis/AA-005-top-qa-cross-review.md`
- report assembly/navigation: `docs/architecture/analysis/final-report-evidence-index.md`
- evaluation methodology: `docs/evaluation/architecture-experiment-methodology.md`
- central evaluation pipeline: `docs/evaluation/evaluation-strategy.md`

## Current DP-00 state and unresolved items

- QA-01 comparative evaluation: **complete**; R1–R4 establish synthetic
  topology sensitivity, not production latency.
- QA-02 comparative evaluation: **complete**; deterministic candidate defects
  remain.
- QA-03 flexibility campaign: **complete**; official CCR is A/B 60%, C/D 80%.
- QA-04 comparative evaluation: **complete**; every candidate fails frozen
  qualification and requires remediation.
- DP-00 trade-space characterization: **complete** in AA-026.
- Final A/B/C/D selection: **intentionally deferred pending downstream
  architecture evidence**.
- AA-026's historical next step was old-scope DP-03 Capability Placement. The
  catalog review found that ownership question absorbed by DP-00.
- Current all-family next investigation: **DP-13 Execution State and Recovery
  Authority**. Current C/D-specific next investigation: **DP-16 Local
  Execution Hosting and Isolation**.
- QA-02 minimum correctness eligibility and executable mandatory qualification
  gates remain unresolved; no post-result threshold is invented.
- Production Model/Agent/Tool latency values remain unresolved and will refine
  conditional preference rather than invalidate the sensitivity campaign.
