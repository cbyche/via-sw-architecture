# ADR-002 — Use Per-Task Durable Supervisors

- Status: Accepted
- Date: 2026-09-22
- Related DP: TASK-DP01

> Current measurement-contract impact: 이전 QA-02 responsiveness와 QA-01 기반 QA-04 reference evidence는 superseded다. QA-04는 현행 독립 QA가 아니며 동시성은 QA-01/03 등의 workload condition으로 이동했다. 이 ADR의 선택은 유지하되 새 Voice와 QA-09 contract로 재검증해야 한다.

## Context

The earlier candidate evaluation compared a shared transactional Task service with per-Task durable supervisors. Both used the same repository, WAL/FULL persistence, idempotency, and recovery endpoints. The `2^4` follow-up evaluated TASK A/B in all eight contexts formed by IR, AGENT, and EXEC choices.

## Decision

Adopt candidate B, Per-Task Durable Supervisors. Each Task has one fenced writer and mailbox while durable command identity, state, and effects remain in the common repository.

## Alternatives considered

Candidate B assigns each Task to a durable single-writer supervisor with activation fencing and mailbox delivery.

## Quality-attribute rationale

| QA | Result |
|---|---|
| QA-08 | A/B both 1.933, score 4 |
| QA-09 | B was about 0.17ms lower on this run; both score 5 and effectively tied |
| QA-02 (historical) | Superseded handoff endpoint; not current evidence |
| QA-04 (historical, retired draft) | Depends on the superseded QA-01 foreground contract; not a current QA |

At the time of the 2026-09-22 decision, the explicitly simulated-reference QA-04 campaign supported B, including a frozen TASK×EXEC interaction of `-0.01`. That result is now historical because its denominator used the superseded QA-01 contract. The accepted decision is preserved for continuity, but it must not be presented as current performance evidence until revalidation.

## Tactics and weakness

The weakness is activation, fencing, and cross-Task coordination complexity. Persist command/outbox identities, fence stale epochs, keep external calls outside the mailbox critical section, and use a common RelationScheduler for cross-Task commands.

Revalidate on the Windows product runtime before treating the reference ratio as an absolute performance result.

Full-factorial evidence: `results/gate2/archive/w12-g1/rebaseline/gate2-factorial-c8869c88/full-factorial.json`.

## Requirement changes

None.
