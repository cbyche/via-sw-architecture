# VIA Software Architecture

Samsung PC용 **Voice Interaction Agent (VIA)** 의 SW Architecture 설계와 검증을 위한 engineering repository입니다.

이 repository는 다음 흐름을 추적합니다.

```text
Approved Requirements / vNext Working Baseline
        ↓
Architectural Decision Point
        ↓
Alternatives
        ↓
Quality Attributes / Evaluation Rules
        ↓
Executable Specification / Prototype / Experiment
        ↓
Measured QA Trade-off + Qualification Gates
        ↓
Architecture Decision Record (ADR)
        ↓
requirements-vNext / future Approved Baseline update
```

## Current baseline and working state

- Approved requirements: [`docs/requirements/requirements-v1.1.md`](docs/requirements/requirements-v1.1.md) — **v1.1 Approved Baseline, immutable**
- vNext working baseline: [`docs/requirements/requirements-vNext.md`](docs/requirements/requirements-vNext.md)
- Central QA↔DP navigation: [`docs/architecture/qa-dp-traceability.md`](docs/architecture/qa-dp-traceability.md)
- Central evaluation strategy: [`docs/evaluation/evaluation-strategy.md`](docs/evaluation/evaluation-strategy.md)
- Final-report evidence index: [`docs/architecture/analysis/final-report-evidence-index.md`](docs/architecture/analysis/final-report-evidence-index.md)

The current top-level architecture question is:

> **DP-00 — VIA Primary Execution Boundary**
>
> VIA는 사용자 요청을 어디까지 직접 판단·실행하고, 어디부터 Downstream Agent에 위임할 것인가?

`requirements-v1.1.md` defines the approved **starting** responsibility boundary. vNext does not assume that boundary is already the optimal final answer; DP-00 explicitly evaluates it.

## Responsibility boundary: approved baseline vs vNext evaluation

The v1.1 Approved Baseline places interaction/context/intent/routing/task-lifecycle responsibilities primarily in VIA and domain reasoning/planning/tool execution in Downstream Agents.

DP-00 compares four Integrated Product execution topologies:

| Alternative | Summary | v1.1 compatibility |
| --- | --- | --- |
| **A — Thin VIA** | Agent-neutral VIA orchestration → Downstream Agent execution | **Compatible** |
| **B — ARGO-centric Primary Execution** | thin realtime context → ARGO primary ReAct/tool runtime → optional specialist delegation | **Challenges baseline boundary** |
| **C — Hybrid VIA Fast Path** | bounded/local-safe VIA execution + Agent delegation | **Mostly compatible / extension** |
| **D — Adaptive Per-turn** | per-turn Fast / ARGO / Specialized-Agent topology selection | **Partially compatible / extension likely** |

Alternative B is intentionally a responsibility-boundary challenge. It is not equivalent to merely preferring ARGO as the default Downstream Agent.

Authoritative definition: [`docs/architecture/decision-points/DP-00-primary-execution-boundary.md`](docs/architecture/decision-points/DP-00-primary-execution-boundary.md).

## vNext Top Architectural Drivers

| QA | Reviewer-facing question | Primary Metric |
| --- | --- | --- |
| **QA-01 Fast-task End-to-End Responsiveness** | 빠른가? | **Fast-task Outcome Latency p95 (FTOL p95)** |
| **QA-02 VIA Interaction-Orchestration Correctness** | 정확한가? | **Architecture Episode Exact Conformance Rate (AECR)** |
| **QA-03 Flexibility** | 변경이 잘 격리되는가? | **Change Containment Rate (CCR)** |
| **QA-04 Model Call Overhead** | 실행 경로를 정하기 위해 AI 판단을 얼마나 요구하는가? | **Average Model Calls to Commit Execution Route** |

Detailed QA definitions are under `docs/evaluation/quality-attributes/`.

The older QA-01~11 inside `requirements-v1.1.md` are preserved as **Legacy v1.1 Detailed QA**. They are reclassified in the central traceability as Secondary/Diagnostic concerns, operational QAs, or Mandatory Qualification candidates; they are not deleted.

## Scored drivers vs mandatory gates

```text
Scored Architectural Drivers
  QA-01
  QA-02
  QA-03
  QA-04

Mandatory Qualification Gates / Constraints
  Security
  Privacy / context-sharing policy
  Trusted boundary requirements
  Required cancellation semantics
  Required task-state integrity
  Failure containment
  Mandatory recovery behavior
```

Security/reliability concerns are not omitted because they are less important. Non-compensable obligations are separated from trade-off scoring so another QA's high score cannot offset a mandatory violation.

A minimum QA-02 correctness eligibility gate is also planned; its numeric threshold remains TBD until Pilot/calibration.

## Central evaluation pipeline

```text
Architecture Concern
    ↓
DP-00 A/B/C/D
    ↓
QA / Primary Metric
    ↓
Controlled Architecture Qualification
    ↓
Pilot / Calibration
    ↓
Scoring & Gate Rule Freeze
    ↓
Final A/B/C/D Evaluation
    ↓
Sensitivity / Threats to Validity
    ↓
Trade-off / ADR
```

Architecture Qualification keeps the **SW Architecture Alternative** as the independent variable and controls/fixes scenario semantics, semantic replay, Agent/tool behavior, model/prompt/cache profiles, machine/environment, and dependency latency as applicable.

Actual-model / real-stack validation is a separate external-validity track.

## Repository structure

```text
docs/
  requirements/             Approved and working requirements
  architecture/             System views and QA↔DP traceability
  architecture/analysis/    Architecture reasoning / review checkpoints
  architecture/decision-points/
                            Decision Point definitions and alternatives
  adr/                      Accepted Architecture Decision Records
  experiments/              Experiment definitions
  evaluation/               QA definitions and evaluation methodology

benchmark/
  requests/                 Architecture request suite
  utterances/               Natural spoken evaluation set
  agent-stubs/              Deterministic downstream-agent stubs
  failure-injection/        Failure scenarios
  evolution/                Replaceability/extensibility exercises
  schemas/                  Raw/semantic benchmark contracts
  runners/                  Benchmark code

prototypes/                 DP alternative implementations
results/
  raw/                       Immutable experiment evidence
  derived/                   Recomputable metrics
  reports/                   Human-readable reports / visualizations
scripts/                    Utility scripts
```

## Working model

1. `requirements-v1.1.md` is immutable.
2. Architecture changes and unresolved responsibility questions are recorded in `requirements-vNext.md` and Decision Point artifacts without pretending they are already approved.
3. DP alternatives are specified/implemented under a common Integrated Product comparison boundary.
4. The same frozen benchmark conditions are applied across alternatives.
5. Raw evidence is retained so metrics can be recomputed.
6. Scores/gates are frozen before final comparative results.
7. The accepted decision is recorded as an ADR.
8. Requirement/scope changes justified by the decision are proposed in vNext before any future approved baseline.

## Decision Point lifecycle

```text
Open
→ Alternatives Defined
→ Evaluation QAs / Rules Defined
→ Executable Architecture Specification
→ Prototype Ready
→ Benchmark Complete
→ Decision Proposed
→ Accepted
→ ADR
```

Current DP-00 state: **Alternatives Defined + Top-QA/evaluation rebaseline complete; executable specification/Pilot TBD.**

## Baseline policy

Approved requirements are never edited in place.

```text
v1.1 Approved
     ↓
vNext Working / Architecture Evaluation
     ↓
measured decision + review
     ↓
possible v1.2 Approved candidate
```
