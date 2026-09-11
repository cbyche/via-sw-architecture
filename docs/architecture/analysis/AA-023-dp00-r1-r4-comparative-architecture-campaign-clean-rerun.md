# AA-023 — DP-00 R1–R4 Comparative Architecture Campaign Clean Rerun

## Disposition

`COMPARATIVE CAMPAIGN COMPLETE — READY FOR DP-00 DECISION SYNTHESIS`

The clean rerun supplies complete QA-01, QA-02, and QA-04 comparative evidence
without selecting a DP-00 winner. The full auditable tables and diagnostics are
in the [campaign report](../../../results/reports/pilot-v0/dp00-r1-r4-20260911T101059Z-d4059ecc/campaign-report.md).

## Identity and rejected predecessor

Campaign `dp00-r1-r4-20260911T101059Z-d4059ecc` ran in the dedicated
`/private/tmp/via-sw-architecture-7-2r` worktree on
`exp/dp00-r1-r4-clean-rerun`, rooted exactly at frozen source
`d4059eccd2883b3029b1e8c94fc86d092a5041f8`. The rejected predecessor
`dp00-r1-r4-20260911T072300Z-d4059ecc` and all of its records, schedules,
metrics, and hashes were excluded.

The matrix was P01–P12 × A/B/C/D × R1–R4 × 16, for CAPTURE and paired
MINIMAL: 6,144 measured executions and 3,072 complete pairs. R1–R4 use frozen
`dp00-realistic-sensitivity-v1`; Profile Z remains 0/0/0 microseconds.

## Source and campaign integrity

Remote `origin/exp/dp00-runtime-pilot` resolved to the frozen SHA. Preflight
passed Python 95/95 and every required Rust check/test. Before and after every
profile, HEAD, clean source, and worktree path matched the freeze. Cross-profile
validation found the frozen SHA in all four roots and zero source, profile,
campaign, provenance, semantic, QA-02-pair, or QA-04-pair mismatches. Missing,
duplicate, strict-error, and strict-warning counts are all zero.

Per-profile derivation was byte-identical. SHA-256 values are R1
`1e9c30728e9fbff481a538e7d0b5bd475c375873f9e6528be230aa7a3ececc87`,
R2 `099513e8cb0fb06dc074b539ecc0f5046c70d5408c7bbf57ad679b620a47d647`,
R3 `a2aca41e3fa59bcbaf9cc904757d0b69ffe6a49bdcd3a82b0bdb7e5c1b570116`,
R4 `f06ba1adfef2952b35203bd1a6a99d4f67c7c382e4b4b270ae80145b94d1e278`,
and combined
`e82aea138202d7737709eb52cef3b66d607b46ae8e0799c5be15763ad024bd9c`.

## QA-01 and structural interpretation

B is fastest in every R1–R4 pooled p50 and p95 because its one-generation
topology reduces the configured model budget. Its largest advantage is in R2.
D replaces one third of QA-01-cohort delegated executions with local tool
execution; that helps its dependency budget in R3, is neutral in R1/R2, and
hurts its R4 tail. C's bounded local path is not exercised in P01–P03, so no
QA-01 local benefit is directly measured for C.

CAPTURE structural totals per profile are A 384/128/0, B 208/112/0,
C 384/112/16, and D 384/80/80 for Model/Agent/Tool semantic events. Full
Profile × Alternative FTOL percentiles, budgets, and residuals are retained in
the campaign report and machine-readable report data.

## QA-02 and QA-04

QA-02 is profile invariant and exactly reproduces Profile Z. CAPTURE conformance
is A 160/192 (83.333%), B 128/192 (66.667%), C 160/192 (83.333%), and
D 144/192 (75%). P04/P11 fail for all, P06 additionally fails for B, and P07
additionally fails for B/D. Every failure repeats 16/16 and matches MINIMAL;
these are deterministic architecture behaviors, with zero infrastructure or
intermittent-noise failures.

QA-04 is also profile invariant. For each profile, A/C have 128 required, 96
qualified, 16 forbidden violations, 32 required routes not committed, and 16
QA-02 exclusions. B has 128/80/16/48/48. D has 128/96/16/0/32. Every stratum
FAILs the frozen qualification for explicit architecture reasons. There is no
scalar QA-04 score.

## P12 and Profile Z

All 512 P12 executions and 256 pairs pass S1→S2 ordering, common-parent and
distinct-child identity, both commits before either execution, execution after
the barrier, two effects, final S2 boundary, QA-02 exactness, QA-04
qualification, and semantic/provenance equality.

Profile Z previously ranked A first on microsecond-scale software overhead.
R1–R4 directly measure B's crossover to first once dependency costs are added.
D improves relative to A/C when Agent cost dominates and loses sharply when
Tool cost dominates. C has no QA-01-cohort local crossover. No exact unmeasured
threshold is fabricated.

## Pareto and limitations

With QA-01, QA-02, QA-04, and dependency structure kept separate, B trades
lower latency/model calls for worse correctness and route qualification; A/C
lead correctness; D uniquely has zero required-route gaps and the largest local
ownership. R3 makes A potentially dominated by C on measured QA/latency axes,
but structural ownership remains an explicit product preference. No alternative
is consistently dominated across all four profiles.

R1–R4 are synthetic sensitivity profiles, not production latency. The corpus
uses deterministic replay and stubs; p99 is diagnostic. No arbitrary QA weights,
utility function, scalar score, or winner is introduced.

## Completion and next action

QA-01 comparative evaluation: complete. QA-02 comparative evaluation: complete,
with architecture defects remaining. QA-04 comparative evaluation: complete,
with all alternatives failing qualification. Evaluation completeness is not a
claim that a QA is satisfied.

Together with the separate existing QA-03 evidence track, the package is ready
for DP-00 Decision Synthesis. No further architecture experiment is required
before that step. The next action is owner-led, unweighted vector/Pareto
synthesis across QA-01, QA-02, QA-03, QA-04, dependency topology, ownership,
and product preference.
