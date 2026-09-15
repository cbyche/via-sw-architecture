# QA-11 — Long-running Task Feedback Freshness

## Purpose / Product Concern

Measure how quickly truthful, meaningful task events become feedback the user actually receives through the required channel.

## Official Scalar Metric

**Task Event-to-Useful-Feedback Latency p95**. Event- and channel-specific latencies are non-scoring diagnostics.

## Unit / Direction

Seconds; lower is better.

## Measurement Formula

`p95(user receipt - reportable event availability at VIA boundary)`. Missing useful feedback contributes a 30-second latency value.

## Measurement Boundary

Start is when the truthful/reportable event becomes available at the VIA boundary. End is when the user actually receives meaningful feedback through the scenario-required channel. Generic ACK, fabricated progress, and stale repetition are not useful.

## Frozen Population Reference

`qa11-feedback-v1`: 200 events—40 accepted/queued, 40 meaningful progress, 40 approval-needed, 40 completion, and 40 blocked/failure.

## Failure / Missing-data Treatment

Missing useful feedback receives the 30-second penalty and remains in the population.

## Product Target

`p95 <=1.5 seconds`.

## 0–5 Score Mapping

| Score | Value x (seconds) |
| --- | --- |
| 5 | x ≤ 0.75 |
| 4 | 0.75 < x ≤ 1.0 |
| 3 | 1.0 < x ≤ 1.5 |
| 2 | 1.5 < x ≤ 2.25 |
| 1 | 2.25 < x ≤ 3.0 |
| 0 | x > 3.0 |

## Rationale

Long-running tasks need fresh truthful feedback even when final completion latency is dominated by external execution.

## What This QA Does Not Measure

It does not measure final task duration, generic acknowledgement speed, fabricated activity, or repeated stale status.

## Typical Architecture Sensitivity

Agent event contracts, task projections, event coalescing, response policy, channel delivery, backpressure, and reconnect behavior affect freshness.

## Evidence / Claim Limits

The event is timed only after it is truthfully reportable at the VIA boundary; upstream Agent silence is visible as a separate limitation, not silently charged to VIA.
