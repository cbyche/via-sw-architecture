# ADR-002 — Use Per-Task Durable Supervisors

- Status: Accepted
- Date: 2026-09-22
- Related DP: TASK-DP01

## Context

Gate 2 compared a shared transactional Task service with per-Task durable supervisors. Both used the same repository, WAL/FULL persistence, idempotency, and recovery endpoints. The `2^4` follow-up evaluated TASK A/B in all eight contexts formed by IR, AGENT, and EXEC choices.

## Decision

Adopt candidate B, Per-Task Durable Supervisors. Each Task has one fenced writer and mailbox while durable command identity, state, and effects remain in the common repository.

## Alternatives considered

Candidate B assigns each Task to a durable single-writer supervisor with activation fencing and mailbox delivery.

## Quality-attribute rationale

| QA | Result |
|---|---|
| W-08 | A/B both 1.933, score 4 |
| W-09 | B was about 0.17ms lower on this run; both score 5 and effectively tied |
| W-02 | A raw value was lower in 8/8 contexts; all score 5 |
| W-04 | B raw ratio was lower in 8/8 contexts; score split in 4/8 contexts |

W-04 supports B in the explicitly simulated-reference campaign, including a frozen TASK×EXEC interaction of `-0.01`. The score split is context-conditional rather than universal, so this remains an Architecture reference decision, not a production throughput claim.

## Tactics and weakness

The weakness is activation, fencing, and cross-Task coordination complexity. Persist command/outbox identities, fence stale epochs, keep external calls outside the mailbox critical section, and use a common RelationScheduler for cross-Task commands.

Revalidate on the Windows product runtime before treating the reference ratio as an absolute performance result.

Full-factorial evidence: `results/rebaseline/gate2-factorial-c8869c88/full-factorial.json`.

## Requirement changes

None.
