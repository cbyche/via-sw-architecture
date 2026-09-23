# ADR-004 — Defer Semantic Decision Ownership Selection

- Status: Deferred
- Date: 2026-09-22
- Related DP: IR-DP01

> Current measurement-contract impact: 이전 W-01/W-02 수식 기반 latency는 superseded되어 현재 근거에서 제외한다. Deferred 상태는 유지하며 새 delegated/direct Voice token ledger와 call graph로 다시 비교한다.

## Context

The earlier candidate evaluation compares an integrated semantic authority with staged grounding, Task association, and handling authorities using the same hosted Qwen3-8B reference profile. The `2^4` reference campaign evaluated IR A/B in all eight TASK/AGENT/EXEC contexts, but the hosted W-05 run remains blocked on the OpenRouter credential.

## Decision

Do not declare an architecture winner. Retain candidate A, Integrated Semantic Authority, as the interim reference until the frozen hosted-model comparison can run.

## Alternatives considered

Candidate B creates independent stage contracts and provenance, allowing correction of a suffix but adding boundaries and possible serial model calls.

## Quality-attribute rationale

| QA | Result |
|---|---|
| W-08 | A/B both 1.933, score 4 |
| W-05 | BLOCKED_NOT_RUN because the OpenRouter credential was absent |
| W-01 (historical) | Superseded by the current delegated Voice definition |
| W-02 (historical) | Superseded by the current direct Voice definition |

The prior reference model favored A on historical W-01/W-02 formulas, but those values no longer support the decision. W-05 remains unmeasured and the new Voice call graphs are not frozen, so A remains only the interim reference rather than an accepted winner.

## Interim tactics

Use a versioned output schema, client-side validation, bounded repair cycles, source provenance, and prompt/schema hashing. These limit coupling and invalid output without pretending the hosted-model trade-off was measured.

## Revisit condition

Run the frozen `qwen/qwen3-8b` OpenRouter profile with provider `alibaba`, fallback disabled. Full-factorial evidence: `results/gate2/archive/w12-g1/rebaseline/gate2-factorial-c8869c88/full-factorial.json`.

## Requirement changes

None.
