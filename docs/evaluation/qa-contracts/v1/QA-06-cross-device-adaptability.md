# QA-06 — Cross-Device Adaptability

## Purpose / Product Concern

Measure delivery of Mobile, TV, and Robot requirements without semantically modifying platform-independent VIA Core contracts.

## Official Scalar Metric

**Core-Preserving Device Adaptation Rate**. Device-specific change counts are non-scoring diagnostics.

## Unit / Direction

Percent; higher is better.

## Measurement Formula

`100 × delivered requirement cells with no VIA Core contract semantic modification / 60 frozen device requirement cells`.

## Measurement Boundary

From a frozen device requirement cell through delivered function and Core-contract semantic review. Device-specific adapters and providers are allowed.

## Frozen Population Reference

`qa06-device-adaptation-v1`: 20 Mobile, 20 TV, and 20 Robot requirement cells, totaling 60.

## Failure / Missing-data Treatment

Omitting the device requirement is not success. Missing functionality or semantic modification of platform-independent VIA Core contracts fails the cell.

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

The architecture should absorb device variation at platform seams while keeping shared product semantics stable.

## What This QA Does Not Measure

It does not require identical UI, media, sensors, hardware providers, or deployment mechanisms across devices.

## Typical Architecture Sensitivity

Platform abstractions, adapter boundaries, Core contract scope, capability discovery, response rendering, and resource-provider interfaces affect adaptation.

## Evidence / Claim Limits

The score covers only the frozen requirement cells and does not imply support for unrepresented device classes.
