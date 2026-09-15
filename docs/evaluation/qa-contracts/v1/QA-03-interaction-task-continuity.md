# QA-03 — Interaction & Task Continuity

## Purpose / Product Concern

Measure exact preservation of identity and lifecycle relationships across turns, executions, results, responses, delivery, and control events.

## Official Scalar Metric

**Continuity Episode Exact Success Rate**. Relation-specific outcomes are non-scoring diagnostics.

## Unit / Direction

Percent; higher is better.

## Measurement Formula

`100 × episodes with every required relation correct / 400 frozen episodes`.

## Measurement Boundary

From the first event in a frozen lifecycle/event-interleaving episode through all required checks for Turn ↔ Task, Task ↔ Execution, Execution ↔ Result, Result ↔ Response, Response ↔ Delivery, and approval/cancel/follow-up relationships.

## Frozen Population Reference

`qa03-continuity-v1`: 400 fixed lifecycle/event-interleaving episodes.

## Failure / Missing-data Treatment

Every required relation must be correct. One wrong or missing relation fails the episode.

## Product Target

`>=99%`.

## 0–5 Score Mapping

| Score | Value x (%) |
| --- | --- |
| 5 | x = 100 |
| 4 | 99.5 ≤ x < 100 |
| 3 | 99 ≤ x < 99.5 |
| 2 | 95 ≤ x < 99 |
| 1 | 90 ≤ x < 95 |
| 0 | x < 90 |

## Rationale

User trust depends on preserving the complete causal and identity chain, especially under concurrency, correction, approval, cancellation, and follow-up.

## What This QA Does Not Measure

Physical barge-in latency is QA-12. Task-cancellation correctness and delivery/history continuity remain here.

## Typical Architecture Sensitivity

State ownership, correlation IDs, event ordering, lifecycle arbitration, response versioning, and modality convergence affect continuity.

## Evidence / Claim Limits

The score applies only to the frozen interleavings; relation-level diagnostics must accompany it to expose failure modes.
