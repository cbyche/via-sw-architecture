# ADR-003 — Isolate Integration Workloads in Supervised Processes

- Status: Accepted
- Date: 2026-09-22
- Related DP: EXEC-DP01

> W12-G2 impact: 이전 W-01~W-03 latency와 W-01 기반 W-04 evidence는 superseded/review-required다. Process-isolation 결정은 W-09 recovery와 W-10 containment 근거로 유지하고 새 Voice latency는 재측정한다.

## Context

Gate 2 compared integration code in the VIA process with the same code in a supervised child process. The comparison included whole-process recovery, integration-host fatal faults, and a fixed containment matrix. The `2^4` follow-up evaluated EXEC A/B in all eight contexts formed by IR, TASK, and AGENT choices.

## Decision

Adopt candidate B, Process-isolated Integration Runtime, for integration workloads. Core owns a supervised worker lifecycle and versioned local IPC; canonical Task state and policy remain in Core.

## Alternatives considered

Candidate B adds versioned IPC, worker lifecycle supervision, and an independent deployment/fault domain.

## Quality-attribute rationale

| QA | Result |
|---|---|
| W-09 | B was about 176.33ms lower in all 8/8 contexts; both score 5 and pass target |
| W-10 | A/B both 28/28, 100%, score 5 |
| W-01 (historical) | Superseded by the W12-G2 Voice definition; not current evidence |
| W-04 (historical) | Depends on the superseded W-01 foreground contract; review required |

Candidate B's integration-fatal p95 values were 18/33ms versus A's 545/564ms in the full-factorial run. W-09 did not split a band, but its raw recovery improvement remains direct evidence for B. The previous W-01/W-02/W-03 and W-04 reference costs are historical and must be remeasured under W12-G2.

## Tactics and weakness

The weakness is IPC serialization, uncertain in-flight calls, deployment, and worker lifecycle complexity. Use versioned frames, idempotent submission keys, bounded queues, health supervision, generation fencing, and reconciliation after reconnect.

Revalidate IPC and delivery latency on the Windows named-pipe realization before making an absolute product-performance claim.

Full-factorial evidence: `results/gate2/archive/w12-g1/rebaseline/gate2-factorial-c8869c88/full-factorial.json`.

## Requirement changes

None.
