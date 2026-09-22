# ADR-002 — Use Per-Task Durable Supervisors

- Status: Accepted
- Date: 2026-09-22
- Related DP: TASK-DP01

## Context

Gate 2 compared a shared transactional Task service with per-Task durable supervisors. Both used the same repository, WAL/FULL persistence, idempotency, and recovery endpoints.

## Decision

Adopt candidate B, Per-Task Durable Supervisors. Each Task has one fenced writer and mailbox while durable command identity, state, and effects remain in the common repository.

## Alternatives considered

Candidate B assigns each Task to a durable single-writer supervisor with activation fencing and mailbox delivery.

## Quality-attribute rationale

| QA | Result |
|---|---|
| W-08 | A/B both 1.933, score 4 |
| W-09 | A 550.833ms vs B 551.167ms, both score 5 |
| W-02 | NOT_RUN |
| W-04 | A 1.06/score 4 vs B 1.05/score 5 in the frozen reference harness |

W-04 passes G3 in the explicitly simulated-reference campaign. This is an Architecture mockup decision, not a production throughput claim.

## Tactics and weakness

The weakness is activation, fencing, and cross-Task coordination complexity. Persist command/outbox identities, fence stale epochs, keep external calls outside the mailbox critical section, and use a common RelationScheduler for cross-Task commands.

Revalidate on the Windows product runtime before treating the reference ratio as an absolute performance result.

## Requirement changes

None.
