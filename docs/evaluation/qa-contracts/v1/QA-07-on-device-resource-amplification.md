# QA-07 — On-device Resource Amplification

## Purpose / Product Concern

Measure memory multiplied by architecture topology beyond the unavoidable minimum required AI stack for the same workload.

## Official Scalar Metric

**Peak Memory Amplification Ratio**. Absolute memory and energy are non-scoring diagnostics.

## Unit / Direction

Ratio (`x`); lower is better.

## Measurement Formula

`candidate peak physically resident required AI-stack memory / frozen minimum mandatory AI-stack memory` for the same workload. Include model weights, true replicas, KV/cache state, S2S/ASR/TTS, required Agent runtime, context/evidence/provisional state, Task state, and response/delivery/audio buffers. Shared physical pages count once; true replicas count separately; another process is not an exclusion.

## Measurement Boundary

Take peak physical residency from the start through the end of the frozen required AI-stack workload's residency interval.

## Frozen Population Reference

`qa07-resource-v1`: the non-counted fixed workload timeline in
`benchmark/fixtures/qa-v1/resource/qa07-resource-workload-v1.json`.

The population is frozen by workload identity and timeline content, not by an
integer case denominator. This corpus-manifest finalization does not alter the
official metric, denominator semantics, target, or scoring bands.

## Failure / Missing-data Treatment

Missing resident-memory evidence fails evidence completeness and yields no score; it is never imputed as low memory.

## Product Target

`<=1.5x`.

## 0–5 Score Mapping

| Score | Value x |
| --- | --- |
| 5 | x ≤ 1.10 |
| 4 | 1.10 < x ≤ 1.25 |
| 3 | 1.25 < x ≤ 1.50 |
| 2 | 1.50 < x ≤ 1.75 |
| 1 | 1.75 < x ≤ 2.00 |
| 0 | x > 2.00 |

## Rationale

Physical accounting captures replicas and cross-process duplication without double-counting genuinely shared pages.

## What This QA Does Not Measure

It does not measure absolute device fit, total capacity, latency, or energy efficiency.

## Typical Architecture Sensitivity

Process boundaries, model replication, cache ownership, Agent hosting, duplicated context/task state, and media buffering affect amplification.

## Evidence / Claim Limits

The ratio is meaningful only with the same frozen workload and denominator; real hardware calibration belongs to a Reference Environment version.
