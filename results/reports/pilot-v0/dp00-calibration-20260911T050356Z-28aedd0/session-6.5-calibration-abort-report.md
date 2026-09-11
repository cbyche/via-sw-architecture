# Session 6.5 — DP-00 Calibration Acceptance Freeze and Aborted Re-run

## Disposition

**REJECTED — PROVENANCE-V4 ANALYZER QUALIFICATION DEFECT.**

The prospective acceptance contract was committed and pushed before any new
calibration result was generated. The counterbalanced run then completed its
physical schedule and passed strict raw/schedule validation, but the frozen
analyzer rejected the acceptance population. Per the no-mid-run-change rule,
the session stopped without modifying the source, analyzer, protocol, threshold,
or observed evidence. Session 6.5R preserves these rejected artifacts in a
dedicated evidence-only commit before any analyzer remediation.

This is not Official calibration PASS evidence, is not architecture-ranking
input, will not be reused as an acceptance population, and must never be
promoted by re-analysis with a later analyzer. Its rejected disposition is
permanent.

## Frozen identity and preflight

| Item | Value |
| --- | --- |
| Acceptance contract | `dp00-calibration-acceptance-v1` |
| Analysis | `dp00-analysis-v7` |
| Acceptance implementation commit / frozen source | `28aedd0d42acc74bb5b67a6b95fb6c7d8a268c78` |
| Calibration ID | `dp00-calibration-20260911T050356Z-28aedd0` |
| Branch / upstream before run | `exp/dp00-runtime-pilot`; 0 ahead / 0 behind |
| Python / pytest | 3.14.7 / 9.1.1 |
| Rust / Cargo | 1.94.0 / 1.94.0 |
| Target / OS / architecture | `aarch64-apple-darwin` / macOS / aarch64 |
| Tokio / worker | 1.53.1 / `tokio-current-thread-v0` |
| Build | `qualification-or-release` |
| Profile | Z; 0/0/0 µs model/agent/tool delay |

Before execution the working tree was clean, upstream divergence was 0/0, Python
reported 72 passing tests, and Rust workspace tests, fmt, warnings-denied clippy,
qualification build, schedule regressions, P01–P12 CAPTURE/MINIMAL strict
validation, and the Rust P12 CAPTURE/MINIMAL regression passed.

## Execution and evidence accounting

W0 and W1 each completed all 96 P01–P12 × A–D × CAPTURE/MINIMAL paths. The
measured schedule completed exactly 16 cycles, 1,536/1,536 executions and
768/768 adjacent pairs. There are no missing or duplicate pairs, no provenance
mismatches, and no semantic-signature mismatches. Schedule validation confirms
24/24 first-mode balance per cycle, 6/6 per alternative, 2/2 per scenario, 8/8
per identical pair, and four appearances of each alternative in each position.

Two independent derivations are byte-identical at SHA-256
`8d1971bfc71494a2a08fbef340e2a191be30a8e6bf2136db4f2211f543b60359`.

## Stop condition

`dp00-analysis-v7` inherits a closed-world recognition check that admits
CAPTURE/MINIMAL Measurement Spines only when provenance is exactly
`dp00-pilot-provenance-v3`. The new frozen protocol correctly emits
`dp00-pilot-provenance-v4`. Consequently every MINIMAL episode is marked QA-02
nonconformant for absence-based constraints even though all 768 CAPTURE/MINIMAL
semantic signatures match.

This produces 592 paired QA-02 mismatches and zero fully qualified pairs. The
legacy `paired_calibration.semantic_mismatches` collection also contains those
592 pair ids because that field combines semantic, QA-02, and QA-04 comparison;
the actual `semantic_match` flag is true for all 768 pairs.

The acceptance input is therefore rejected. `M`, the 5% margin, drift values,
Theil–Sen shifts, mode-order medians/interaction, bootstrap intervals, and
correctness-qualified QA-01 percentiles are undefined and were not computed
from an unqualified substitute population.

P12 likewise cannot pass the frozen analyzer's paired correctness gate: all 64
CAPTURE P12 executions are exact-conformant, all 64 MINIMAL P12 executions are
misclassified by the v3-only recognition check, and all 64 P12 pairs fail QA-02
equality. The earlier Rust P12 Measurement Spine regression still passes.

## Freeze readiness

| Candidate | Result | Reason |
| --- | :---: | --- |
| Run-order Rule | FAIL / not evaluable | Schedule passes; drift and interaction populations are rejected. |
| Repetition-count | FAIL / not evaluable | Fixed population is complete but stability cannot be evaluated. |
| Instrumentation-mode | FAIL | Paired correctness-qualified evidence is empty. |
| Latency-profile | FAIL / not evaluable | Profile Z stability cannot be evaluated. |
| benchmark-v1 | FAIL | Calibration stability is unresolved. |
| aggregation-v1 | FAIL | QA-04 Alternative × Profile qualification remains FAIL for A/B/C/D. |
| scoring-v1 | FAIL | No accepted stability population or frozen scoring mapping. |
| gate-v1 | FAIL | No owner-backed QA-02 numeric gate; QA-04 qualification remains unresolved. |

## Immutability and next action

All previous Official and Session 6.4 evidence paths are unchanged. New raw,
derived, abort-report, and inventory artifacts exist only under the new
calibration ID and remain uncommitted because normal calibration completion was
not achieved.

A new prospective source commit must extend the closed-world Measurement Spine
recognition contract to v4 (with an integration regression that derives QA-02
and P12 from a complete v4 CAPTURE/MINIMAL pair), increment the analysis version,
commit/push, and use a new calibration ID. The existing run must not be repaired
or reinterpreted after that change.
