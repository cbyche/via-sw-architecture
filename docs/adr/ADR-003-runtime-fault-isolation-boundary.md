# ADR-003 — Isolate Integration Workloads in Supervised Processes

- Status: Accepted
- Date: 2026-09-22
- Related DP: EXEC-DP01

> Current measurement-contract impact: 이전 QA-01~04 evidence는 superseded다. Previous-generation QA-04는 concurrency였고 QA-10은 containment였다. 현재 QA-04는 Voice interruption, 현재 QA-32는 fault blast radius다. Process-isolation 결정은 보존하되 QA-31/32/41과 새 Voice contract로 재검증한다.

## Context

The earlier candidate evaluation compared integration code in the VIA process with the same code in a supervised child process. The comparison included whole-process recovery, integration-host fatal faults, and a fixed containment matrix. The `2^4` follow-up evaluated EXEC A/B in all eight contexts formed by IR, TASK, and AGENT choices.

## Decision

Adopt candidate B, Process-isolated Integration Runtime, for integration workloads. Core owns a supervised worker lifecycle and versioned local IPC; canonical Task state and policy remain in Core.

## Alternatives considered

Candidate B adds versioned IPC, worker lifecycle supervision, and an independent deployment/fault domain.

## Quality-attribute rationale

| QA | Result |
|---|---|
| previous-generation QA-09 (current concern: QA-31) | B was about 176.33ms lower in all 8/8 contexts; both score 5 and pass target |
| previous-generation QA-10 (current concern: QA-32) | A/B both 28/28, 100%, score 5; current QA score로 사용하지 않음 |
| QA-01 (historical) | Superseded by the current measurement contract Voice definition; not current evidence |
| previous-generation QA-04 concurrency | Depends on the superseded QA-01 foreground contract; unrelated to current QA-04 |

Candidate B's integration-fatal p95 values were 18/33ms versus A's 545/564ms in the full-factorial run. Previous-generation QA-09 did not split a band, but its raw recovery improvement remains historical evidence for B. The previous Voice/concurrency reference costs are historical and must be remeasured under the current measurement contract.

## Tactics and weakness

The weakness is IPC serialization, uncertain in-flight calls, deployment, and worker lifecycle complexity. Use versioned frames, idempotent submission keys, bounded queues, health supervision, generation fencing, and reconciliation after reconnect.

Revalidate IPC and delivery latency on the Windows named-pipe realization before making an absolute product-performance claim.

Full-factorial evidence: `results/gate2/archive/w12-g1/rebaseline/gate2-factorial-c8869c88/full-factorial.json`.

## Requirement changes

None.
