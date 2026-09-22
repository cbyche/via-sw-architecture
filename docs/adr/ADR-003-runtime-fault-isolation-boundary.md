# ADR-003 — Defer Runtime Fault-Isolation Boundary Selection

- Status: Deferred
- Date: 2026-09-22
- Related DP: EXEC-DP01

## Context

Gate 2 compared integration code in the VIA process with the same code in a supervised child process. The comparison included whole-process recovery, integration-host fatal faults, and a fixed containment matrix.

## Decision

Do not declare an architecture winner under the current frozen score contract. Retain candidate A, Single-process Partitioned Runtime, as the interim reference. Keep candidate B, Process-isolated Integration Runtime, as the preferred re-evaluation candidate because it has strong raw recovery evidence.

## Alternatives considered

Candidate B adds versioned IPC, worker lifecycle supervision, and an independent deployment/fault domain.

## Quality-attribute rationale

| QA | Result |
|---|---|
| W-09 | A 550.833ms vs B 374.667ms; both score 5 and pass target |
| W-10 | A/B both 28/28, 100%, score 5 |
| W-01 | BLOCKED_NOT_RUN without actual user delivery |
| W-04 | BLOCKED_NOT_RUN without actual user delivery |

Candidate B's integration-fatal p95 values were 18/33ms versus A's 545/563ms. This demonstrates process-boundary causality but does not meet the pre-result G3 rule because the aggregate score band and target outcome do not split.

## Interim tactics

Use bounded workers, timeout, backpressure, cancellation, and process-wide restart supervision for the A reference. Do not describe these tactics as equivalent to OS process isolation.

## Revisit condition

Measure W-01/W-04 at actual delivery endpoints, or approve a new pre-result score contract that captures the product value of integration-only recovery before rerunning the campaign.

## Requirement changes

None.
