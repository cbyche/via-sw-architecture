# Session 6.4 — DP-00 Runtime Calibration Execution

## Disposition

Calibration execution and evidence qualification completed without missing,
duplicate, validation, provenance-mismatch, or semantic-mismatch evidence. The
benchmark/calibration protocol is **not freeze-ready** because CAPTURE exhibits
cycle drift and the paired instrumentation delta changes materially with
mode-order. No A/B/C/D architecture ranking is made.

## Identity and environment

| Item | Value |
| --- | --- |
| Calibration ID | `dp00-calibration-20260911T035717Z-49b8bc0` |
| Frozen branch | `exp/dp00-runtime-pilot` |
| Frozen source SHA | `49b8bc0f624c1e2503bc0dcc4ecbc0b772b2a902` |
| Upstream divergence after fetch | 0 ahead / 0 behind |
| Python | 3.14.7; pytest 9.1.1 |
| Rust | rustc 1.94.0 (`4a4ef493e`, 2026-03-02) |
| Cargo | 1.94.0 (`85eff7c80`, 2026-01-15) |
| Target / OS / architecture | `aarch64-apple-darwin` / macOS / aarch64 |
| Tokio / worker policy | 1.53.1 / `tokio-current-thread-v0` |
| Build profile | `qualification-or-release` (optimized) |
| Cargo.lock SHA-256 | `e37217ab528819935c3d03647307222873226a25f6d1c9310db7fb38730be7cf` |
| Latency profile | Profile Z, 0/0/0 µs model/agent/tool controlled delay |
| Warm-up / measured per invocation | 1 / 1 |
| Semantic baseline | `canonical-event-v3`; `dp00-pilot-provenance-v3`; `dp00-analysis-v5`; `DP00-RUNTIME-PILOT-V0 v0.2`; `qa04-route-contract-policy-v1` |

## Preflight

All required preflight gates passed before measured calibration began:

- branch, exact HEAD, fetched-upstream 0/0 divergence, and clean tree: PASS;
- Python regression: 45 passed;
- Rust workspace regression: PASS for all workspace targets;
- fmt and clippy with warnings denied: PASS;
- optimized qualification build: PASS;
- P01–P12 CAPTURE strict validation/derivation: PASS, zero errors/warnings;
- P12 CAPTURE/MINIMAL correctness regression: PASS for A/B/C/D, including
  exact QA-02 conformance, S1→S2 commits, final-required boundary, effects,
  correlation, and QA-04 qualification.

## Execution accounting

| Item | Expected | Actual | Result |
| --- | ---: | ---: | :---: |
| Measured executions | 384 | 384 | PASS |
| Missing executions | 0 | 0 | PASS |
| Duplicate executions | 0 | 0 | PASS |
| CAPTURE/MINIMAL pairs | 192 | 192 observed | PASS |
| Complete pairs | 192 | 192 | PASS |
| Incomplete pairs | 0 | 0 | PASS |
| Semantic mismatch pairs | 0 | 0 | PASS |
| Provenance mismatch pairs | 0 | 0 | PASS |

The analyzer's stricter all-QA qualification admits 140 pairs. QA-01 latency
comparison uses the 48 qualified P01–P03 pairs with authoritative FTOL in both
modes. Non-QA-01 scenarios are not assigned synthetic latency values.

## Semantic invariance and determinism

All 192 complete pairs match on semantic output, QA-02 result, required effects,
route identity/order, parent/child/subgoal correlation, logical model-call
identity/count, and derived QA-04 semantics. All 192 also match shared
provenance, with explicit mode-order slots `{0,1}`.

Across all modes and four cycles, the 48 Alternative × Scenario semantic
condition groups each have exactly one normalized semantic/QA-02/QA-04
signature. No nondeterministic group was observed. Every required stable
runtime/provenance field has one value across all 384 executions.

Two independent `dp00-analysis-v5` derivations from the persistent raw root are
byte-identical.

## QA-01 paired FTOL

Authoritative formula only:

`FTOL = UsefulOutcomeObserved.timestamp - AcousticEos.timestamp`

`episode_elapsed_nanos` was not used. Values below are nearest-rank nanoseconds.

| Population | n | p50 | p95 | p99 diagnostic | mean | min / max |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| CAPTURE FTOL | 48 | 7,333 | 9,000 | 9,250 | 7,305.625 | 4,917 / 9,250 |
| MINIMAL FTOL | 48 | 7,334 | 9,042 | 10,250 | 7,509.625 | 4,834 / 10,250 |
| CAPTURE − MINIMAL | 48 | +42 | +1,000 | +1,082 | −204.000 | −2,083 / +1,082 |

Delta signs are 25 positive, 22 negative, and 1 zero. Therefore the signed
distribution does not support treating CAPTURE as a stable fixed additive
overhead.

### Paired overhead by alternative

| Alternative | n | p50 ns | p95 ns | mean ns |
| :---: | ---: | ---: | ---: | ---: |
| A | 12 | +167 | +1,082 | +45.000 |
| B | 12 | −125 | +417 | −357.667 |
| C | 12 | +83 | +792 | −118.000 |
| D | 12 | −208 | +1,041 | −385.333 |

These are calibration diagnostics, not architecture rankings.

### Paired overhead by QA-01 scenario

| Scenario | n | p50 ns | p95 ns | mean ns |
| :---: | ---: | ---: | ---: | ---: |
| P01 | 16 | −42 | +1,082 | −164.188 |
| P02 | 16 | +42 | +1,000 | −283.688 |
| P03 | 16 | −84 | +792 | −164.125 |

## Position and mode-order

Rotation reproduction is exact: cycle orders are ABCD, BCDA, CDAB, DABC in
both modes. Every alternative appears 24 times at every position over the full
measured population. CAPTURE and MINIMAL each appear 96 times in mode-order
slot 0 and 96 times in slot 1.

Paired-delta p50 by alternative position 0/1/2/3 is respectively
`−166 / +83 / −42 / −42 ns`; p95 is `+458 / +1,082 / +750 / +792 ns`.
The position effect is smaller than the mode-order effect but is not zero.

| CAPTURE mode-order | n | delta p50 ns | delta p95 ns | mean ns |
| --- | ---: | ---: | ---: | ---: |
| slot 0 (CAPTURE first) | 24 | −709 | +458 | −550.292 |
| slot 1 (CAPTURE second) | 24 | +167 | +1,000 | +142.292 |

The median changes by 876 ns and reverses sign. Instrumentation comparison is
therefore order-dependent in this run.

## Drift

CAPTURE FTOL cycle p50 is `6,042 → 7,459 → 7,500 → 7,709 ns`, a monotonic
increase. MINIMAL p50 is `7,500 → 7,000 → 7,417 → 7,333 ns`, not monotonic.
Paired-delta p50 is `−1,375 → +167 → +167 → −42 ns`, also not monotonic.

Early cycles 0–1 versus late cycles 2–3:

| Population | early p50 ns | late p50 ns | late − early |
| --- | ---: | ---: | ---: |
| CAPTURE | 7,125 | 7,709 | +584 ns (+8.2%) |
| MINIMAL | 7,334 | 7,334 | 0 ns |
| paired delta | −417 | +167 | +584 ns |

Every alternative has a positive CAPTURE early→late p50 shift (+292 to +875
ns), while MINIMAL shifts range from −333 to +84 ns. This mode-specific pattern
is consistent with a warm-state, thermal, allocator, cache, or resource
accumulation interaction, but this run cannot distinguish those causes. No
inferential acceptance test or threshold was frozen before observation, so no
post-hoc p-value/threshold is introduced. The visible magnitude and monotonic
CAPTURE sequence are sufficient for conservative freeze-readiness FAIL.

## QA-04 and P12

There are 184 correctness-qualified primary QA-04 observations. Nevertheless,
the Alternative × Profile group qualification is FAIL for A/B/C/D under the
frozen contract because their complete REQUIRED populations do not all have a
committed, QA-02-conformant observation. Counts are preserved in the machine
report; no null is replaced by a penalty and no failed episode is reclassified.

P12 has 32/32 correctness-qualified executions and 16/16 qualified pairs. Every
P12 execution has exact QA-02 conformance, exactly S1 then S2 commits, a common
parent with distinct children, execution after the final commit, both required
effects, final boundary equal to the S2 commit, and the first S1 commit distinct
from the final boundary. No raw/derived inclusion mismatch was observed.

## Failures and exclusions

Calibration integrity failures: none. Analyzer validation errors/warnings,
missing/duplicate evidence, provenance mismatches, pair semantic mismatches,
and P12 failures are all zero.

The frozen architecture baseline reproducibly contains 88 QA-02-nonconformant
executions: P04 A/B/C/D, P06 B, P07 B/D, and P11 A/B/C/D, eight executions per
listed cell. These are semantic architecture outcomes, not latency noise or
calibration evidence corruption. They remain unmodified and are excluded where
the frozen correctness-qualified policies require exclusion.

## Freeze readiness

| Candidate | Result | Evidence |
| --- | :---: | --- |
| Run-order Rule | FAIL | Rotation/balance reproduced, but CAPTURE has monotonic cycle drift and delta is mode-order dependent. |
| Repetition-count | FAIL | Four cycles expose drift/interaction and do not establish stable tail estimates or a defensible stopping rule. |
| Instrumentation-mode | FAIL | Signed overhead is unstable; mode-order changes the median by 876 ns and reverses its sign. |
| Latency-profile | FAIL | Profile Z baseline is measured, but its CAPTURE path drifts; controlled-dependency/real-stack non-dominance remains unresolved. |
| benchmark-v1 | FAIL | Run-order, repetition, instrumentation, and latency-profile prerequisites are not freeze-ready. |
| aggregation-v1 | FAIL | All Alternative × Profile QA-04 primary-comparability qualifications fail; OPTIONAL/final aggregate selection remains unresolved. |
| scoring-v1 | FAIL | No stable final latency population or pre-result score mapping/threshold is available. |
| gate-v1 | FAIL | No owner-backed numeric QA-02 gate exists, and QA-04 group qualification fails. |

## Evidence immutability and repository impact

The two prior Official campaign populations and derived/reported evidence were
never used as calibration input and were not modified. Their before/after
SHA-256 inventory is byte-identical. This calibration's raw, derived, inventory,
and report artifacts are stored only under its new calibration ID. The only
repository changes are new generated calibration evidence/report files; no
source, implementation, runner, analyzer, schema, corpus, scenario, order,
repetition, or threshold file was changed.
