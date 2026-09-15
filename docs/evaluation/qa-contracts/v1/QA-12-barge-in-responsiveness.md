# QA-12 — Barge-in Responsiveness

## Purpose / Product Concern

Measure how quickly prior audible output stops after the user begins a valid barge-in.

## Official Scalar Metric

**Barge-in Audible Stop Latency p95**. Cancellation and continuity outcomes are non-scoring diagnostics for this QA.

## Unit / Direction

Milliseconds; lower is better.

## Measurement Formula

`p95(t_last_audible_sample_of_previous_response - t_ground_truth_user_speech_onset)`. Failure to stop within two seconds contributes 2000 ms.

## Measurement Boundary

Start is ground-truth user speech onset. End is the last audible sample of the previous response. Cancel API completion, inference stop, and TTS-generation stop are not substitutes.

## Frozen Population Reference

`qa12-barge-in-v1`: 200 frozen positive barge-in episodes.

## Failure / Missing-data Treatment

Failure to stop within two seconds receives the 2000 ms penalty and remains in the population.

## Product Target

`p95 <=200 ms`.

## 0–5 Score Mapping

| Score | Value x (milliseconds) |
| --- | --- |
| 5 | x ≤ 100 |
| 4 | 100 < x ≤ 133 |
| 3 | 133 < x ≤ 200 |
| 2 | 200 < x ≤ 300 |
| 1 | 300 < x ≤ 400 |
| 0 | x > 400 |

## Rationale

The user experiences acoustic overlap, so the physical audible boundary—not internal cancellation progress—is authoritative.

## What This QA Does Not Measure

Task-cancellation correctness and delivery/history continuity are QA-03 concerns.

## Typical Architecture Sensitivity

Voice activity detection, audio buffering, playback ownership, cross-process control, TTS streaming, and media-session arbitration affect audible stop time.

## Evidence / Claim Limits

The population contains positive barge-in episodes; false interruption or non-barge-in detection quality requires separate diagnostic coverage.
