# Session 6.6 — DP-00 Prospective Calibration Re-run and Exit Assessment

## Disposition

**CALIBRATION PASS. DP-00 CALIBRATION SUFFICIENT FOR ARCHITECTURE EXPERIMENTATION.**

The prospective calibration completed from the exact frozen source with a valid
acceptance population. Execution, pair, semantic, provenance, QA equality, P12,
cycle-drift, mode-order, and deterministic-replay checks pass. No further
calibration-infrastructure revision or benchmark micro-optimization is required
for Architecture DP experimentation. Independent benchmark/scoring/gate blockers
remain explicit and do not prevent starting actual DP-00 prototype measurement.

This report does not rank A/B/C/D.

## Frozen identity and environment

| Item | Value |
| --- | --- |
| Calibration ID | `dp00-calibration-20260911T054901Z-61d476f` |
| Branch | `exp/dp00-runtime-pilot` |
| Frozen source SHA | `61d476fd73dbb2d99a2c91447aded02073f15535` |
| Protocol / manifest | `dp00-calibration-protocol-v1` / `dp00-calibration-manifest-v1` |
| Acceptance / analysis | `dp00-calibration-acceptance-v1` / `dp00-analysis-v8` |
| Event / provenance | `canonical-event-v3` / `dp00-pilot-provenance-v4` |
| Python / pytest | 3.14.7 / 9.1.1 |
| Rust / Cargo | 1.94.0 / 1.94.0 |
| Target / OS / architecture | `aarch64-apple-darwin` / macOS 26.5.1 / arm64 |
| Tokio / worker | 1.53.1 / `tokio-current-thread-v0` |
| Build / latency profile | `qualification-or-release` / Profile Z (0/0/0 µs) |

Before calibration, branch, SHA, clean-tree, and 0-ahead/0-behind checks passed.
Python regression passed 93 tests. Rust workspace/all-target regression, fmt,
warnings-denied clippy, qualification build, schedule and acceptance regressions,
provenance-v4 CAPTURE/MINIMAL and P12 integrations, QA-01 authoritative FTOL,
QA-02, QA-04, and P01–P12 strict validation all passed. The separate P01–P12
preflight population had zero validation errors and warnings.

## Protocol completion and integrity

W0 and W1 each completed exactly 96/96 P01–P12 × A–D × CAPTURE/MINIMAL paths.
They and the retained one-invocation warm-up are excluded from measured results.

| Check | Expected | Actual | Result |
| --- | ---: | ---: | :---: |
| Measured cycles | 16 | 16 | PASS |
| Measured executions | 1,536 | 1,536 | PASS |
| Adjacent pairs | 768 | 768 complete | PASS |
| QA-01 P01–P03 pairs | 192 | 192 qualified | PASS |
| Incomplete pair | 0 | 0 | PASS |
| Duplicate mode | 0 | 0 | PASS |
| Provenance mismatch | 0 | 0 | PASS |
| Semantic mismatch | 0 | 0 | PASS |
| QA-02 mismatch | 0 | 0 | PASS |
| QA-04 mismatch | 0 | 0 | PASS |

The frozen schedule validates pair adjacency, 24/24 first-mode balance per cycle,
6/6 per alternative/cycle, 2/2 per scenario/cycle, 8/8 per identical pair,
next-cycle inversion, independent ABCD/BCDA/CDAB/DABC rotation, and four visits
by every alternative to every position. Adaptive stopping was disabled.

## QA-01 calibration statistics

All values below are nanoseconds and use `linear-interpolated-(n-1)` with
`NO_EXCLUSION`.

| Population | p50 | p95 | p99 diagnostic |
| --- | ---: | ---: | ---: |
| CAPTURE | 5,854.0 | 7,060.45 | 7,336.78 |
| MINIMAL | 5,792.0 | 6,976.9 | 7,685.72 |
| Paired delta (CAPTURE − MINIMAL) | 42.0 | 662.35 | 1,443.28 |

The pooled correctness-qualified MINIMAL p50 is `M = 5,792.0 ns`; the frozen
5% practical-equivalence margin is `289.6 ns`.

Alternative, scenario, position, cycle, and mode-order breakdowns are preserved
in `analysis-summary.json`; these diagnostics are not Architecture rankings.

## Cycle drift acceptance

| Mode / subcriterion | Early p50 | Late p50 | Signed / absolute shift | % of M | Result |
| --- | ---: | ---: | ---: | ---: | :---: |
| CAPTURE early/late | 5,854.0 | 5,854.0 | 0.0 / 0.0 | 0.000% | PASS |
| MINIMAL early/late | 5,833.0 | 5,729.0 | −104.0 / 104.0 | 1.796% | PASS |

| Mode / subcriterion | Theil–Sen slope per cycle | Predicted 0→15 shift | Absolute % of M | Result |
| --- | ---: | ---: | ---: | :---: |
| CAPTURE | +11.5 | +172.5 | 2.978% | PASS |
| MINIMAL | −3.40909 | −51.13636 | 0.883% | PASS |

Both subcriteria pass for both modes. **Cycle Drift Acceptance = PASS.**

## Mode-order interaction

The QA-01 qualified population contains 96 CAPTURE-first and 96 MINIMAL-first
pairs. `D_CF = +125.0 ns`, `D_MF = −41.0 ns`; signed `D_CF − D_MF = +166.0 ns`.
The absolute interaction is `166.0 ns`, or `2.866%` of M, within the `289.6 ns`
margin. **Mode-order Interaction = PASS.**

## Bootstrap diagnostics

Frozen paired, early/late × first-mode-stratified bootstrap; 10,000 resamples,
seed 20260911, 95% CI. These intervals are diagnostic and do not change gates.

| Statistic | 95% CI (ns) |
| --- | ---: |
| Pooled CAPTURE p50 | [5,563.0, 6,041.5] |
| Pooled MINIMAL p50 | [5,500.0, 5,958.0] |
| Pooled paired-delta p50 | [0.0, 83.0] |
| CAPTURE signed late-minus-early p50 | [−437.5, 520.5] |
| MINIMAL signed late-minus-early p50 | [−500.0, 334.0] |
| Signed `D_CF − D_MF` | [105.0, 228.5] |

## Correctness, P12, and architecture blockers

Semantic invariance and paired provenance/QA integrity pass for all 768 pairs.
QA-02 has 1,184/1,536 conformant executions and 352 nonconformant executions.
This is the unchanged frozen architecture pattern: exactly 88 nonconformances
per four-cycle block, repeated four times by the 16-cycle protocol. It is neither
repaired nor reclassified, and CAPTURE/MINIMAL QA-02 mismatch is zero.

P12 passes 64/64 pairs and 128/128 executions. Every execution has S1 and S2
committed in order, a common parent and distinct children, execution only after
the final route-plan commit, two required effects, final boundary at S2 rather
than first S1, exact QA-02 conformance, and QA-04 qualification. CAPTURE/MINIMAL
semantic, QA-02, QA-04, and provenance equality all pass.

Under `qa04-route-contract-policy-v1`, Alternative × Profile Z primary
comparability qualification remains FAIL for A, B, C, and D. This is an
Architecture correctness/aggregation blocker, not a calibration-stability
failure.

## Freeze readiness

| Candidate | Result | Basis |
| --- | :---: | --- |
| Run-order Rule | PASS | Schedule invariants, cycle drift, and mode-order interaction pass. |
| Repetition-count | PASS | 16 cycles and 768 pairs complete; stability gates pass. |
| Instrumentation-mode | PASS | Semantic/provenance/QA pair integrity, drift, and interaction pass. |
| Latency-profile | PASS (Profile Z calibration only) | Profile Z stability passes; future real-stack profile remains separate scope. |
| benchmark-v1 | NOT READY — independent blockers | Calibration is sufficient, but QA-04 qualification and real-stack/final benchmark scope remain. |
| aggregation-v1 | FAIL | QA-04 Alternative × Profile qualification fails for A/B/C/D. |
| scoring-v1 | FAIL | Scoring mapping is not frozen. |
| gate-v1 | FAIL | Owner-backed QA-02 numeric gate and QA-04 qualification remain unresolved. |

## Determinism and evidence immutability

Re-derivation from the same raw root was byte-identical. Both derived summaries
have SHA-256 `2c15bd581f7392380e5f4a36aa72ac2b790620f88670b3ba3dd92438efdb0295`.
Strict raw validation reports zero errors and warnings.

Session 6.4 `dp00-calibration-20260911T035717Z-49b8bc0` and Session 6.5 rejected
`dp00-calibration-20260911T050356Z-28aedd0` were not used as population input and
have no diffs. Session 6.5 remains permanently `REJECTED — PROVENANCE-V4 ANALYZER
QUALIFICATION DEFECT`. New raw, derived, report, and inventory artifacts exist
only under this calibration ID; no source, analyzer, protocol, schema, QA
semantics, scenario, or A/B/C/D implementation was changed.

## DP-00 exit assessment and next step

Execution/pair/provenance integrity, semantic invariance, acceptance-population
formation, cycle drift, mode-order interaction, P12 qualification, and
deterministic derivation all pass. The remaining measurement variation is inside
the prospectively frozen practical-equivalence margin and does not invalidate the
correctness population.

Therefore **DP-00 CALIBRATION SUFFICIENT FOR ARCHITECTURE EXPERIMENTATION**.
Additional calibration revision is not needed for Architecture judgment and is
not the recommended next step. Proceed to actual DP-00 Architecture prototype
measurement/analysis while resolving QA-04 aggregation, owner-backed QA-02 gate,
scoring mapping, and future real-stack validation as independent workstreams.
