# QA-10 — Trace Reconstructability for Diagnosis & Improvement

## Purpose / Product Concern

Measure whether candidate telemetry lets an external evaluator reconstruct the full causal decision and delivery chain for both successes and controlled faults/edges.

## Official Scalar Metric

**End-to-End Decision Trace Reconstruction Rate**. Chain-element rates are non-scoring diagnostics.

## Unit / Direction

Percent; higher is better.

## Measurement Formula

`100 × fully reconstructable traces / 200 frozen traces`, using candidate telemetry only.

## Measurement Boundary

Reconstruct User input/revision → temporal evidence/provenance → semantic/model decision → Agent selection → Agent session/execution mapping → UserTask lifecycle → approval/progress/result → response facts/version → Voice/Text delivery → relevant latency/resource attribution. Hidden oracle or fault labels must not appear in telemetry.

## Frozen Population Reference

`qa10-trace-v1`: 100 successful execution traces and 100 controlled fault/edge traces.

## Failure / Missing-data Treatment

One missing required causal-chain element makes the trace fail reconstruction.

## Product Target

`>=98%`.

## 0–5 Score Mapping

| Score | Value x (%) |
| --- | --- |
| 5 | x = 100 |
| 4 | 99 ≤ x < 100 |
| 3 | 98 ≤ x < 99 |
| 2 | 95 ≤ x < 98 |
| 1 | 90 ≤ x < 95 |
| 0 | x < 90 |

## Rationale

Useful traces support both failure diagnosis and learning from successful behavior without relying on evaluator-only truth embedded in telemetry.

## What This QA Does Not Measure

It does not score logging volume, dashboard aesthetics, or hidden-oracle availability.

## Typical Architecture Sensitivity

Correlation identity, event schemas, provenance, state transitions, cross-process propagation, response versioning, and resource attribution affect reconstruction.

## Evidence / Claim Limits

Passing frozen traces does not guarantee every future incident is diagnosable; report successful and fault/edge strata separately.
