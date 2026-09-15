# QA-08 — Safe Task Recoverability

## Purpose / Product Concern

Measure time to restore a safe, continuation-ready task state after a recoverable transient fault without duplicating or making an unsafe side effect.

## Official Scalar Metric

**Safe Task Recovery Time p95**. Recovery-stage timings are non-scoring diagnostics.

## Unit / Direction

Seconds; lower is better.

## Measurement Formula

`p95(recovery end - ground-truth fault onset)`. Failure to reach safe recovery by 60 seconds contributes a 60-second latency value.

## Measurement Boundary

Start is ground-truth fault onset. End requires all of: the same logical UserTask retained, executor state reconciled, no duplicate/unsafe side effect, continuation-ready safe state, and user-visible task state restored.

## Frozen Population Reference

`qa08-recovery-v1`: 200 frozen recoverable transient-fault episodes.

## Failure / Missing-data Treatment

No safe recovery by 60 seconds receives the 60-second penalty. Unsafe or incorrect recovery independently fails architecture qualification and cannot be offset by QA scores.

## Product Target

`p95 <=10 seconds`.

## 0–5 Score Mapping

| Score | Value x (seconds) |
| --- | --- |
| 5 | x ≤ 2 |
| 4 | 2 < x ≤ 5 |
| 3 | 5 < x ≤ 10 |
| 2 | 10 < x ≤ 20 |
| 1 | 20 < x ≤ 40 |
| 0 | x > 40 |

## Rationale

Recovery is complete only when logical identity, execution safety, continuation readiness, and user-visible projection agree.

## What This QA Does Not Measure

The p95 target does not mean 5% failure is acceptable. Permanent-fault availability and raw restart time alone are not this metric.

## Typical Architecture Sensitivity

State authority, checkpoints, reconciliation, idempotency, deduplication, supervisor boundaries, and user-task projection affect recovery.

## Evidence / Claim Limits

Only declared recoverable transient faults belong in this population; unsafe/incorrect recovery remains a separate invariant gate.
