# VIA System Overview — vNext Working View

## Status

This overview distinguishes the **v1.1 Approved starting responsibility boundary** from the **vNext architecture boundary currently under evaluation by DP-00**.

It does not modify `docs/requirements/requirements-v1.1.md` and does not select a DP-00 winner.

## v1.1 Approved starting responsibility boundary

The Approved Baseline currently describes approximately this topology:

```text
User
  ↓
Voice / Text Interaction
  ↓
VIA
  ├─ Context grounding
  ├─ Intent refinement
  ├─ Downstream Agent routing / delegation
  ├─ Context sharing / consent interaction
  ├─ Conversation / Task lifecycle
  ├─ Progress / status / cancel / follow-up
  └─ Result interaction
        ↓
Downstream Agent
  ├─ Domain reasoning
  ├─ Planning
  ├─ Tool selection
  ├─ Tool execution
  └─ Domain result
        ↓
OS / App / Web / External Service
```

This remains the **Approved Baseline at DP-00 start**.

## vNext architectural question

The responsibility split above is no longer treated as an unchallengeable vNext invariant.

DP-00 asks:

> **VIA는 사용자 요청을 어디까지 직접 판단·실행하고, 어디부터 Downstream Agent에 위임할 것인가?**

The comparison boundary is the **Integrated Product** and the principal independent variable is the SW placement/ownership of reasoning, execution, routing and orchestration capability.

Authoritative DP:

- `docs/architecture/decision-points/DP-00-primary-execution-boundary.md`

## DP-00 candidate execution topologies

### A — Thin VIA / Agent-neutral Orchestration

```text
Voice / Text / Context
        ↓
VIA interaction + intent + routing + task orchestration
        ↓
Downstream Agent
        ↓
Domain reasoning / planning / tools
```

v1.1 compatibility: **Compatible**.

### B — ARGO-centric Primary Execution

```text
Voice / S2S
    ↓
thin realtime context / interaction layer
    ↓
ARGO primary ReAct reasoning + tool execution
    ↓
Specialized Agent delegation when required
```

v1.1 compatibility: **Challenges baseline boundary**.

B is intentionally a responsibility-boundary challenge. It is not the same as keeping Thin VIA and merely preferring ARGO as the default downstream route.

### C — Hybrid VIA Fast Path

```text
                     ┌─ bounded VIA Fast Path
User → VIA eligibility
                     └─ Downstream Agent
```

v1.1 compatibility: **Mostly compatible / extension**.

Fast Path eligibility is semantic, not an arbitrary `<3 seconds` or `1 LLM + 1 tool` rule.

### D — Adaptive Per-turn Execution

```text
                     ┌─ VIA Fast Path
User turn → selector ├─ ARGO path
                     └─ Specialized Agent
```

v1.1 compatibility: **Partially compatible / extension likely**.

The intended concept is directed per-turn selection/ownership transfer, not unrestricted owner bouncing.

## Current central architecture storyline

```text
Product / Architecture Concern
    ↓
DP-00 Primary Execution Boundary
    ↓
A / B / C / D
    ↓
Top QA-01 ~ QA-04
    ↓
Controlled Architecture Qualification
    ↓
Pilot / Calibration / Rule Freeze
    ↓
Final Evaluation / Trade-off
    ↓
ADR
```

Central navigation:

- `docs/requirements/requirements-vNext.md`
- `docs/architecture/qa-dp-traceability.md`
- `docs/evaluation/evaluation-strategy.md`

## vNext Top Architectural Drivers

| QA | Core question | Primary Metric |
| --- | --- | --- |
| **QA-01** | 빠른가? | Fast-task Outcome Latency p95 (FTOL p95) |
| **QA-02** | 정확한가? | Architecture Episode Exact Conformance Rate (AECR) |
| **QA-03** | 변경이 잘 격리되는가? | Change Containment Rate (CCR) |
| **QA-04** | 실행 경로를 정하기 위해 AI 판단을 얼마나 요구하는가? | Average Model Calls to Commit Execution Route |

The older QA-01~11 inside v1.1 are preserved as **Legacy v1.1 Detailed QA** and are reclassified in central traceability; they are not deleted.

## Scored drivers and mandatory conditions

```text
Scored Architectural Drivers
  QA-01
  QA-02
  QA-03
  QA-04

Mandatory Qualification Gates / Constraints
  Security
  Privacy / Context-sharing policy
  Trusted boundary requirements
  Required cancellation semantics
  Required task-state integrity
  Failure containment
  Mandatory recovery behavior
```

A mandatory violation cannot be compensated by another QA's score.

## Cross-cutting services / roles under evaluation

The following remain relevant architecture roles, but their exact placement/depth can vary by DP-00 alternative:

- Voice / Text interaction
- Context Engine / interaction evidence
- Intent refinement / semantic normalization
- Execution-path / Agent routing
- Session & Conversation State
- Task / Workflow lifecycle
- Agent Harness / integration contract
- Policy / Consent / Identity
- Model Gateway
- Agent Registry / capability metadata
- Memory
- Observability / Audit / Evaluation
- Notification / progress / result interaction

The presence of a role does not imply that every alternative must implement it as the same named component.

## Working invariants that do not prejudge DP-00

- VIA is Voice-first, not Voice-only.
- Approved v1.1 remains immutable while vNext architecture evaluation proceeds.
- A/B/C/D are compared at the same Integrated Product functional boundary.
- User/context sharing remains subject to privacy/consent/trusted-boundary requirements.
- Conversation/task identity and required cancellation/recovery semantics must satisfy mandatory requirements regardless of topology.
- Provider/model/runtime details should be versioned and controlled in architecture qualification.
- Raw benchmark evidence must remain sufficient to recompute derived metrics.

## Not yet decided

- DP-00 winning topology
- final responsibility placement
- final Intent Refiner / Agent Router / Task Manager placement
- exact Fast Path capability set
- QA score thresholds
- minimum QA-02 correctness gate
- final benchmark corpora / QA-04 aggregation rule
- executable A/B/C/D architecture specifications and prototypes
