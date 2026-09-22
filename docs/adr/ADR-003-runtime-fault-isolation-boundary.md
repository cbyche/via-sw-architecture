# ADR-003 — Isolate Integration Workloads in Supervised Processes

- Status: Accepted
- Date: 2026-09-22
- Related DP: EXEC-DP01

## Context

Gate 2 compared integration code in the VIA process with the same code in a supervised child process. The comparison included whole-process recovery, integration-host fatal faults, and a fixed containment matrix.

## Decision

Adopt candidate B, Process-isolated Integration Runtime, for integration workloads. Core owns a supervised worker lifecycle and versioned local IPC; canonical Task state and policy remain in Core.

## Alternatives considered

Candidate B adds versioned IPC, worker lifecycle supervision, and an independent deployment/fault domain.

## Quality-attribute rationale

| QA | Result |
|---|---|
| W-09 | A 550.833ms vs B 374.667ms; both score 5 and pass target |
| W-10 | A/B both 28/28, 100%, score 5 |
| W-01 | BLOCKED_NOT_RUN without actual user delivery |
| W-04 | A 1.06/score 4 vs B 1.05/score 5 in the frozen reference harness |

Candidate B's integration-fatal p95 values were 18/33ms versus A's 545/563ms. W-09 alone did not split a band, but the later pre-frozen W-04 reference campaign does; both independent observations point toward B.

## Tactics and weakness

The weakness is IPC serialization, uncertain in-flight calls, deployment, and worker lifecycle complexity. Use versioned frames, idempotent submission keys, bounded queues, health supervision, generation fencing, and reconciliation after reconnect.

Revalidate IPC and delivery latency on the Windows named-pipe realization before making an absolute product-performance claim.

## Requirement changes

None.
