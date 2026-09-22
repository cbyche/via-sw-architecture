# ADR-004 — Defer Semantic Decision Ownership Selection

- Status: Deferred
- Date: 2026-09-22
- Related DP: IR-DP01

## Context

Gate 2 compares an integrated semantic authority with staged grounding, Task association, and handling authorities using the same hosted Qwen3-8B reference profile.

## Decision

Do not declare an architecture winner. Retain candidate A, Integrated Semantic Authority, as the interim reference until the frozen hosted-model comparison can run.

## Alternatives considered

Candidate B creates independent stage contracts and provenance, allowing correction of a suffix but adding boundaries and possible serial model calls.

## Quality-attribute rationale

| QA | Result |
|---|---|
| W-08 | A/B both 1.933, score 4 |
| W-05 | BLOCKED_NOT_RUN because the OpenRouter credential was absent |
| W-01 | BLOCKED_NOT_RUN without actual user delivery |
| W-02 | NOT_RUN |

No measured metric differentiated the candidates.

## Interim tactics

Use a versioned output schema, client-side validation, bounded repair cycles, source provenance, and prompt/schema hashing. These limit coupling and invalid output without pretending the hosted-model trade-off was measured.

## Revisit condition

Run the frozen `qwen/qwen3-8b` OpenRouter profile with provider `alibaba`, fallback disabled, then add actual delivery evidence under a new freeze where required.

## Requirement changes

None.
