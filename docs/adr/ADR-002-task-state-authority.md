# ADR-002 — Defer Task State Authority Selection

- Status: Deferred
- Date: 2026-09-22
- Related DP: TASK-DP01

## Context

Gate 2 compared a shared transactional Task service with per-Task durable supervisors. Both used the same repository, WAL/FULL persistence, idempotency, and recovery endpoints.

## Decision

Do not declare an architecture winner. Retain candidate A, Shared Transactional Task Service, as the interim reference until representative W-02/W-04 evidence is available. This retention is continuity, not evidence of superiority.

## Alternatives considered

Candidate B assigns each Task to a durable single-writer supervisor with activation fencing and mailbox delivery.

## Quality-attribute rationale

| QA | Result |
|---|---|
| W-08 | A/B both 1.933, score 4 |
| W-09 | A 550.833ms vs B 551.167ms, both score 5 |
| W-02 | NOT_RUN |
| W-04 | BLOCKED_NOT_RUN without actual user delivery |

No metric passed G3 Observable Sensitivity.

## Interim tactics

Keep transactions short, enforce expected revision/CAS, persist outbox/inbox identities, and perform external calls outside database locks. These tactics reduce conflict and duplicate effects without changing the interim authority topology.

## Revisit condition

Capture frozen representative Agent-acceptance and actual user-visible foreground traces, then issue a new Measurement Freeze.

## Requirement changes

None.
