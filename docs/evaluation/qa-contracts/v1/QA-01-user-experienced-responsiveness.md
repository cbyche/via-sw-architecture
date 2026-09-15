# QA-01 — User-Experienced Responsiveness

## Purpose / Product Concern

Measure how quickly a committed Voice or Text request produces the useful facts the user can actually consume, including synchronized Voice/Text delivery.

## Official Scalar Metric

**Fast Useful Outcome Latency p95**. Endpoint decompositions are non-scoring diagnostics.

## Unit / Direction

Seconds; lower is better.

## Measurement Formula

`p95(end - start)` over `qa01-fast-v1`. A wrong or missing useful outcome at 30 seconds contributes a 30-second latency value.

## Measurement Boundary

Start is ground-truth acoustic EOS for Voice and explicit input commit for Text. End is the later of required voice facts actually audibly delivered and required text-detail fields from the same result version available. ACK, first token, internal Agent result, and TTS-generation completion are not endpoints.

## Frozen Population Reference

`qa01-fast-v1`: 150 unique semantic goal instances—50 bounded/read-oriented, 50 short general-Agent, and 50 short specialist-Agent.

## Failure / Missing-data Treatment

Wrong or missing useful outcome at 30 seconds receives the 30-second latency penalty; it is not removed from the population.

## Product Target

`p95 <= 3.0 seconds`.

## 0–5 Score Mapping

| Score | Value x (seconds) |
| --- | --- |
| 5 | x ≤ 1.5 |
| 4 | 1.5 < x ≤ 2.0 |
| 3 | 2.0 < x ≤ 3.0 |
| 2 | 3.0 < x ≤ 4.5 |
| 1 | 4.5 < x ≤ 6.0 |
| 0 | x > 6.0 |

## Rationale

The later user-visible endpoint prevents an architecture from scoring on partial delivery or a Voice/Text version mismatch.

## What This QA Does Not Measure

It does not measure ACK speed, first-token latency, internal completion, long-running feedback freshness, or task correctness beyond penalty treatment.

## Typical Architecture Sensitivity

Semantic-routing stages, execution hops, endpointing, response binding, serialization, and delivery topology can change this latency.

## Evidence / Claim Limits

Qualification results describe the frozen corpus and reference environment, not production latency or a production workload distribution.
