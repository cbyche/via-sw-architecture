# QA-05 — Evolution Change Containment

## Purpose / Product Concern

Measure whether standard product evolution stays inside pre-authorized, architecture-independent semantic concern zones and extension seams.

## Official Scalar Metric

**Change Containment Rate**. LOC, file count, and raw component count are non-scoring diagnostics.

## Unit / Direction

Percent; higher is better.

## Measurement Formula

`100 × contained passing standard changes / 60 frozen standard evolution changes`.

## Measurement Boundary

Before evaluation, each change declares expected ownership zones and approved extension seams. End follows completed functionality, common regression checks, and review that semantic changes remain authorized with no dependency leak. Zones are Z1 Voice/Turn, Z2 Temporal Context/Evidence, Z3 Semantic Decision, Z4 Agent Integration, Z5 User Task/Execution Lifecycle, Z6 Response/Delivery, Z7 Runtime/Resource, and Z8 Device/Platform.

## Frozen Population Reference

`qa05-product-evolution-v1`: 60 frozen standard evolution changes.

## Failure / Missing-data Treatment

A case fails if functionality is incomplete, regressions fail, semantic changes leave pre-authorized zones, or a new semantic dependency leaks into unrelated zones.

## Product Target

`>=90%`.

## 0–5 Score Mapping

| Score | Value x (%) |
| --- | --- |
| 5 | x = 100 |
| 4 | 95 ≤ x < 100 |
| 3 | 90 ≤ x < 95 |
| 2 | 70 ≤ x < 90 |
| 1 | 50 ≤ x < 70 |
| 0 | x < 50 |

## Rationale

Semantic zones provide architecture-independent ownership boundaries and avoid rewarding arbitrary repository layout.

## What This QA Does Not Measure

LOC, file count, development time, and Agent-specific evolution are not the official metric; Agent ecosystem containment is QA-04.

## Typical Architecture Sensitivity

Responsibility cohesion, contract direction, extension seams, shared state, and cross-zone semantic dependencies affect containment.

## Evidence / Claim Limits

Zone and seam declarations must precede implementation; post-hoc boundary changes invalidate comparison.
