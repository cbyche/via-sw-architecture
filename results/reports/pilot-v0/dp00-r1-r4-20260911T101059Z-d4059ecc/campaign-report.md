# DP-00 R1–R4 Comparative Architecture Campaign Clean Rerun

## Disposition

`COMPARATIVE CAMPAIGN COMPLETE — READY FOR DP-00 DECISION SYNTHESIS`

This disposition means that QA-01, QA-02, and QA-04 have complete comparative
evidence. It does not mean that every alternative satisfies QA-02 or QA-04, and
it does not select a DP-00 winner.

## Identity and source integrity

| Item | Value |
| --- | --- |
| Campaign | `dp00-r1-r4-20260911T101059Z-d4059ecc` |
| Rejected predecessor | `dp00-r1-r4-20260911T072300Z-d4059ecc` — not reused |
| Runtime branch at freeze | `exp/dp00-runtime-pilot` |
| Frozen source | `d4059eccd2883b3029b1e8c94fc86d092a5041f8` |
| Measurement branch | `exp/dp00-r1-r4-clean-rerun` |
| Dedicated worktree | `/private/tmp/via-sw-architecture-7-2r` |
| Branch/worktree transition during measurement | No |
| Profile contract | `dp00-realistic-sensitivity-v1` |
| Runner / analyzer / provenance | `bench-runner 0.1.0` / `dp00-analysis-v9` / `dp00-pilot-provenance-v4` |
| Protocol | `dp00-calibration-protocol-v1`, 2 full prewarms, 1 invocation warmup, 16 fixed cycles |
| Python / pytest | 3.14.7 / 9.1.1 |
| Rust / Cargo / Tokio | 1.94.0 / 1.94.0 / 1.53.1 |
| Target / runtime | `aarch64-apple-darwin` / `tokio-current-thread-v0` |

The frozen v4 runner records the common campaign identity in every execution's
`calibration_id` and `pair_id`; its legacy `campaign_id` field remains null and
`official` remains false. This is the existing frozen compatibility convention
documented by AA-021, not a post-measurement rewrite. Every execution records
the same source, profile, scenario, alternative, repetition, mode, pair,
protocol, toolchain, and provenance identity.

Preflight passed: 95/95 Python tests; Rust workspace/all-target check; rustfmt;
warning-denied clippy; and all qualification-profile workspace tests. Those
tests include runner, analyzer, profile, provenance, validator, deterministic
serialization, P01–P12, QA-02, QA-04, P12 route-plan barrier, pairing, Profile
Z, exact R1–R4 constants, and semantic dependency charging.

All before/after profile guards observed the same worktree and frozen HEAD with
a clean source tree. The independent cross-profile scan found exactly one SHA
in each root, always `d4059eccd2883b3029b1e8c94fc86d092a5041f8`.

## Matrix and integrity

| Check | Result |
| --- | ---: |
| CAPTURE | 3,072 / 3,072 |
| MINIMAL | 3,072 / 3,072 |
| Total executions | 6,144 / 6,144 |
| Complete profile-scoped pairs | 3,072 / 3,072 |
| Missing / duplicates | 0 / 0 |
| Profile / source mismatch | 0 / 0 |
| Campaign / provenance mismatch | 0 / 0 |
| Semantic mismatch | 0 |
| QA-02 / QA-04 pair mismatch | 0 / 0 |
| Strict errors / warnings | 0 / 0 |
| Runtime identities | 1 |

The paired analyzer reports 560 fully qualified pairs per profile. The other
208 pairs per profile preserve deterministic architecture QA-02/QA-04 defects;
they are not incomplete, duplicate, semantic, provenance, or infrastructure
failures.

Replay was byte-identical for each raw profile and for the canonical combined
report:

| Output | SHA-256 |
| --- | --- |
| R1 | `1e9c30728e9fbff481a538e7d0b5bd475c375873f9e6528be230aa7a3ececc87` |
| R2 | `099513e8cb0fb06dc074b539ecc0f5046c70d5408c7bbf57ad679b620a47d647` |
| R3 | `a2aca41e3fa59bcbaf9cc904757d0b69ffe6a49bdcd3a82b0bdb7e5c1b570116` |
| R4 | `f06ba1adfef2952b35203bd1a6a99d4f67c7c382e4b4b270ae80145b94d1e278` |
| Combined | `e82aea138202d7737709eb52cef3b66d607b46ae8e0799c5be15763ad024bd9c` |

## QA-01

CAPTURE is the architecture reporting population; MINIMAL is the paired
control. FTOL and residual values are milliseconds. Calls are mean semantic
events per eligible episode. Budgets are mean configured milliseconds per
eligible episode. Each row has 48 observations (P01–P03 × 16); p99 is
diagnostic.

| Profile | Alt | n | p50 | p95 | p99 | Model | Agent | Tool | Model budget | Agent budget | Tool budget | Total budget | Residual p50 |
| --- | :---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| R1 | A | 48 | 172.584 | 180.135 | 180.153 | 2.000 | 1.000 | 0.000 | 100.000 | 50.000 | 0.000 | 150.000 | 22.584 |
| R1 | B | 48 | 115.573 | 120.132 | 120.150 | 1.000 | 1.000 | 0.000 | 50.000 | 50.000 | 0.000 | 100.000 | 15.573 |
| R1 | C | 48 | 174.057 | 180.143 | 180.187 | 2.000 | 1.000 | 0.000 | 100.000 | 50.000 | 0.000 | 150.000 | 24.057 |
| R1 | D | 48 | 173.664 | 179.644 | 180.147 | 2.000 | 0.667 | 0.333 | 100.000 | 33.333 | 16.667 | 150.000 | 23.664 |
| R2 | A | 48 | 243.041 | 250.080 | 250.146 | 2.000 | 1.000 | 0.000 | 200.000 | 20.000 | 0.000 | 220.000 | 23.041 |
| R2 | B | 48 | 135.260 | 140.111 | 140.117 | 1.000 | 1.000 | 0.000 | 100.000 | 20.000 | 0.000 | 120.000 | 15.260 |
| R2 | C | 48 | 241.033 | 250.132 | 250.166 | 2.000 | 1.000 | 0.000 | 200.000 | 20.000 | 0.000 | 220.000 | 21.033 |
| R2 | D | 48 | 241.950 | 249.166 | 250.150 | 2.000 | 0.667 | 0.333 | 200.000 | 13.333 | 6.667 | 220.000 | 21.950 |
| R3 | A | 48 | 162.676 | 170.127 | 170.181 | 2.000 | 1.000 | 0.000 | 40.000 | 100.000 | 0.000 | 140.000 | 22.676 |
| R3 | B | 48 | 133.094 | 140.098 | 140.114 | 1.000 | 1.000 | 0.000 | 20.000 | 100.000 | 0.000 | 120.000 | 13.094 |
| R3 | C | 48 | 162.146 | 170.125 | 170.150 | 2.000 | 1.000 | 0.000 | 40.000 | 100.000 | 0.000 | 140.000 | 22.146 |
| R3 | D | 48 | 160.049 | 170.121 | 170.134 | 2.000 | 0.667 | 0.333 | 40.000 | 66.667 | 6.667 | 113.333 | 23.214 |
| R4 | A | 48 | 83.425 | 90.084 | 90.129 | 2.000 | 1.000 | 0.000 | 40.000 | 20.000 | 0.000 | 60.000 | 23.425 |
| R4 | B | 48 | 56.752 | 60.105 | 60.110 | 1.000 | 1.000 | 0.000 | 20.000 | 20.000 | 0.000 | 40.000 | 16.752 |
| R4 | C | 48 | 81.862 | 89.974 | 90.120 | 2.000 | 1.000 | 0.000 | 40.000 | 20.000 | 0.000 | 60.000 | 21.862 |
| R4 | D | 48 | 85.442 | 169.520 | 170.142 | 2.000 | 0.667 | 0.333 | 40.000 | 13.333 | 33.333 | 86.667 | 22.391 |

Pooled p50 ordering is R1 `B < A < D < C`, R2 `B < C < D < A`,
R3 `B < D < C < A`, and R4 `B < C < A < D`. B's one-call topology wins
every nonzero-cost profile and its advantage is largest in model-dominant R2.
C's bounded local path is not exercised by the QA-01 P01–P03 eligible cohort,
so it has no directly measured local-execution latency benefit here. D replaces
one third of eligible Agent interactions with local Tool executions: this
reduces its mean dependency budget in R3, is budget-neutral in R1/R2, and
increases it in R4. R4's D p95 exposes that tool-dominant tail.

These are controlled synthetic sensitivity results, not production latency.

## QA-02

Every cell below is CAPTURE-only. All listed failures occurred in 16/16
repetitions and match MINIMAL. No threshold is invented.

| Profile | Alt | Conformant / total | Rate | Failing scenarios | Stable |
| --- | :---: | ---: | ---: | --- | :---: |
| R1 | A | 160 / 192 | 83.333% | P04, P11 | yes |
| R1 | B | 128 / 192 | 66.667% | P04, P06, P07, P11 | yes |
| R1 | C | 160 / 192 | 83.333% | P04, P11 | yes |
| R1 | D | 144 / 192 | 75.000% | P04, P07, P11 | yes |
| R2 | A | 160 / 192 | 83.333% | P04, P11 | yes |
| R2 | B | 128 / 192 | 66.667% | P04, P06, P07, P11 | yes |
| R2 | C | 160 / 192 | 83.333% | P04, P11 | yes |
| R2 | D | 144 / 192 | 75.000% | P04, P07, P11 | yes |
| R3 | A | 160 / 192 | 83.333% | P04, P11 | yes |
| R3 | B | 128 / 192 | 66.667% | P04, P06, P07, P11 | yes |
| R3 | C | 160 / 192 | 83.333% | P04, P11 | yes |
| R3 | D | 144 / 192 | 75.000% | P04, P07, P11 | yes |
| R4 | A | 160 / 192 | 83.333% | P04, P11 | yes |
| R4 | B | 128 / 192 | 66.667% | P04, P06, P07, P11 | yes |
| R4 | C | 160 / 192 | 83.333% | P04, P11 | yes |
| R4 | D | 144 / 192 | 75.000% | P04, P07, P11 | yes |

This is exactly the Profile Z pattern. P04 and P11 affect all alternatives;
P06 additionally affects B; P07 additionally affects B and D. All are
classified as deterministic architecture behavior. Infrastructure defects: 0.
Intermittent-noise failures: 0.

## QA-04

There is no scalar score. Each row is CAPTURE-only; the profile-independent
result is expected because latency must not change routing correctness.

| Profile | Alt | Required | Qualified | Forbidden violations | Route not committed | QA-02 exclusions | Qualification |
| --- | :---: | ---: | ---: | ---: | ---: | ---: | :---: |
| R1 | A | 128 | 96 | 16 | 32 | 16 | FAIL |
| R1 | B | 128 | 80 | 16 | 48 | 48 | FAIL |
| R1 | C | 128 | 96 | 16 | 32 | 16 | FAIL |
| R1 | D | 128 | 96 | 16 | 0 | 32 | FAIL |
| R2 | A | 128 | 96 | 16 | 32 | 16 | FAIL |
| R2 | B | 128 | 80 | 16 | 48 | 48 | FAIL |
| R2 | C | 128 | 96 | 16 | 32 | 16 | FAIL |
| R2 | D | 128 | 96 | 16 | 0 | 32 | FAIL |
| R3 | A | 128 | 96 | 16 | 32 | 16 | FAIL |
| R3 | B | 128 | 80 | 16 | 48 | 48 | FAIL |
| R3 | C | 128 | 96 | 16 | 32 | 16 | FAIL |
| R3 | D | 128 | 96 | 16 | 0 | 32 | FAIL |
| R4 | A | 128 | 96 | 16 | 32 | 16 | FAIL |
| R4 | B | 128 | 80 | 16 | 48 | 48 | FAIL |
| R4 | C | 128 | 96 | 16 | 32 | 16 | FAIL |
| R4 | D | 128 | 96 | 16 | 0 | 32 | FAIL |

All four alternatives fail the frozen qualification for decomposable,
deterministic architecture reasons. No profile-dependent QA-04 change exists.

## Structural diagnostics

Counts below are CAPTURE totals per profile over all 192 executions per
alternative. Semantic topology is identical across R1–R4.

| Alt | Model calls | Agent interactions | Tool executions |
| --- | ---: | ---: | ---: |
| A | 384 | 128 | 0 |
| B | 208 | 112 | 0 |
| C | 384 | 112 | 16 |
| D | 384 | 80 | 80 |

B uses 45.8% fewer model calls than A/C/D. C owns one bounded local population;
D owns the broadest local population and has the fewest delegated
interactions. Charges depend only on semantic event type, never alternative ID.

## P12 regression

P12 has 512/512 executions and 256/256 profile-scoped pairs. Every execution
passes S1→S2 ordering, common parent, distinct children, both commits before
execution, execution after the route-plan barrier, two required effects, final
boundary at S2, exact QA-02 conformance, QA-04 qualification, and paired
semantic/provenance equality. There is no new P12 regression.

## Profile Z and crossover analysis

The valid Session 7.0 Profile Z evidence remains immutable. Its CAPTURE p50/p95
milliseconds were A 0.005437/0.006625, B 0.006063/0.007025,
C 0.005459/0.006859, and D 0.005959/0.007164. Z therefore ranked A first at
p50/p95 on software-path overhead alone.

Introducing dependency costs produces a directly measured crossover: B moves
from third at Z p50 to first in R1–R4 because it removes one model generation.
The benefit is strongest in R2, where the model charge is 100 ms. D's local
ownership creates another crossover: it improves relative to A/C in R3's
Agent-dominant regime, while R4's expensive tool charge makes D worst and
widens its p95. C has no QA-01-cohort local crossover; its local route is visible
structurally outside P01–P03. Exact unmeasured threshold values are not inferred.

Analytical extrapolation, kept separate from measurement: B's advantage grows
with the marginal model-generation cost; D benefits when avoided Agent cost
exceeds the substituted Tool cost on its local-eligible mix and loses when the
inequality reverses. Production crossover thresholds require production cost
distributions and are not claimed here.

## Pareto analysis

The vector keeps QA-01, QA-02, QA-04, and structural dependency behavior
separate. No weights or utility function are introduced.

| Profile | Non-dominated | Potentially dominated | Rationale |
| --- | --- | --- | --- |
| R1 | A, B, C, D | none conclusively | B wins latency/calls but loses correctness; A/C lead QA-02; D has zero required-route gaps and most local ownership. |
| R2 | A, B, C, D | none conclusively | B's model advantage is largest; A/C retain correctness; D retains route completeness/locality. |
| R3 | B, C, D | A by C on measured QA/latency axes | C matches A's QA-02/04 and is marginally faster; its different local ownership keeps the conclusion conditional on structural preference. |
| R4 | A, B, C, D | none conclusively | C is faster than A on measured QA-01 with equal QA-02/04, but its extra tool ownership is costly outside the QA-01 cohort; D exposes the tool-cost tail. |

No alternative is consistently dominated across R1–R4. Treating local
ownership, coupling, or call classes as automatically good or bad would be an
unapproved utility function, so the campaign does not do so.

## Completion, limitations, and readiness

- QA-01 comparative evaluation complete: **yes**. The synthetic R1–R4 matrix
  resolves the frozen model/Agent/tool sensitivity question.
- QA-02 comparative evaluation complete: **yes**. Deterministic defects remain
  architecture findings, not missing measurement.
- QA-04 comparative evaluation complete: **yes**. Every alternative fails the
  frozen qualification for explicit architecture reasons; evaluation is still
  complete.

The sensitivity constants are not production measurements. The deterministic
stub corpus does not provide real model, network, Agent, device, or tool latency
distributions, and p99 is diagnostic. No QA weight, scalar score, ownerless
acceptance threshold, or final winner is asserted.

Session 7.2R plus the separate existing QA-03 evidence (A/B 60%, C/D 80%) is
complete enough for `DP-00 DECISION SYNTHESIS`. QA-03 artifacts were neither
modified nor used as runtime inputs, and no additional architecture experiment
is necessary before synthesis. The exact next step is an owner-led vector/Pareto
Decision Synthesis that explicitly adjudicates correctness, route
qualification, dependency topology, local ownership, flexibility, and product
preference without hidden weights.
