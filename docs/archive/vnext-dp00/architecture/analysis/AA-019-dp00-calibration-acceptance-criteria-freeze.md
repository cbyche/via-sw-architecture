# AA-019 — DP-00 Calibration Stability Acceptance Freeze

## Status and scope

**APPROVED BEFORE CALIBRATION EXECUTION.** This decision freezes the stability
acceptance criteria for the counterbalanced calibration introduced by AA-018.
It is prospective: no Session 6.5 calibration result existed or was inspected
when this contract, implementation, tests, and document were committed.

This decision does not change A/B/C/D, scenario or corpus semantics,
`canonical-event-v3`, the Measurement Spine, QA-01/02/04 qualification, P12
compound routing, Profile Z, the outlier policy, or the existing 88 QA-02
nonconformances. The threshold is a project calibration engineering threshold,
not an external standard and not an architecture scoring gate.

## Versioned decision

The machine-readable contract is
`benchmark/contracts/dp00-calibration-acceptance-v1.json`. The implementing
analyzer is `dp00-analysis-v7`. The percentile estimator for all p50/p95/p99
values in this acceptance result is linear interpolation on `(n-1)`; no
observation is removed as an outlier.

Only complete CAPTURE/MINIMAL pairs that pass provenance equality, semantic
invariance, QA-02 equality, QA-04 equality/qualification, authoritative FTOL
boundary presence, and exact QA-01 correctness qualification enter the
stability populations. FTOL remains:

```text
UsefulOutcomeObserved.monotonic_timestamp
- AcousticEos.monotonic_timestamp
```

Define `M` and the single absolute margin as:

```text
M = pooled QA-01 correctness-qualified MINIMAL p50 FTOL
margin = 0.05 × M
```

For CAPTURE and MINIMAL independently, cycles 0–7 are the early half and cycles
8–15 are the late half. Each mode passes drift only when both conditions hold:

```text
abs(late-half p50 - early-half p50) <= margin
abs(Theil-Sen slope over 16 cycle p50s × 15) <= margin
```

The Theil–Sen estimator is the median of all pairwise slopes between the 16
explicit per-cycle QA-01 qualified FTOL p50s. Missing cycle coverage is rejected;
cycles cannot be dropped. Cycle Drift Acceptance passes only if both conditions
pass for both modes.

For each qualified pair, `delta = FTOL_CAPTURE - FTOL_MINIMAL`. Let `D_CF` be
the median delta for CAPTURE-first pairs and `D_MF` the median for MINIMAL-first
pairs. Mode-order Interaction passes only when:

```text
abs(D_CF - D_MF) <= margin
```

The fixed schedule must retain eight CAPTURE-first and eight MINIMAL-first
observations for every identical QA-01 `(scenario, alternative)` identity.
Malformed, incomplete, duplicate, semantically mismatched, or provenance-
mismatched pairs cause rejection rather than repair or inference.

## Bootstrap diagnostic

The analyzer reports two-sided 95% bootstrap confidence intervals from exactly
10,000 resamples with deterministic seed `20260911`. A complete pair is the
sampling unit; each CAPTURE and MINIMAL value therefore travels together.
Sampling is stratified by early/late half and first mode to retain the four
observed population sizes. Nearest-rank endpoints are used for the interval.

Intervals are reported for pooled CAPTURE p50, pooled MINIMAL p50, pooled paired-
delta p50, each mode's signed late-minus-early p50 difference, and signed
`D_CF - D_MF`. Bootstrap is diagnostic only. Its interval cannot replace,
relax, tighten, or otherwise modify the fixed 5% gate.

## Freeze-readiness derivation

- Run-order may pass only when schedule validation, Cycle Drift Acceptance, and
  Mode-order Interaction all pass.
- Repetition-count may pass when all 16 cycles and 768 pairs are complete and
  the stability criteria pass; tail and bootstrap diagnostics remain visible.
- Instrumentation-mode additionally requires semantic invariance and complete
  paired evidence.
- Latency-profile may pass from the calibration perspective when Profile Z
  satisfies stability; controlled Profile Z versus a future real-stack profile
  remains a separately stated scope boundary.
- `benchmark-v1`, `aggregation-v1`, `scoring-v1`, and `gate-v1` do not
  automatically pass. QA-04 Alternative × Profile qualification, an owner-backed
  numeric QA-02 gate, scoring mapping, and other independent blockers remain.

If source, analyzer, protocol, formula, threshold, estimator, split, cycle count,
pre-warm count, bootstrap method, order formula, qualification semantics,
outlier policy, or percentile estimator needs alteration after execution starts,
the calibration stops. A changed contract requires a new version and a new
prospective calibration; it cannot reinterpret the observed run.
