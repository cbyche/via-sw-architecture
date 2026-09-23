# AA-021 — DP-00 Comparative Architecture Campaign

## Disposition

**VALID COMPARATIVE EVIDENCE. NOT READY FOR ARCHITECTURE DECISION.**

The campaign produces valid A/B/C/D correctness, route-contract, structural,
and controlled Profile Z latency evidence. It does not select a winner or
combine QA scores. A final DP-00 architecture decision is blocked because no
approved architecture-comparison latency profile exists beyond zero-delay
Profile Z, so the central trade between B's reduced model-call topology and
C/D's local execution paths cannot yet be interpreted under realistic model,
Agent, tool, device, or network costs.

## Campaign identity and purpose

| Item | Value |
| --- | --- |
| Campaign ID | `dp00-architecture-20260911T060300Z-83d199d` |
| Branch | `exp/dp00-runtime-pilot` |
| Exact source SHA | `83d199d09c6333feea6dd579fa5334f77d3aca1d` |
| Corpus | `DP00-RUNTIME-PILOT-V0` v0.2, P01–P12 |
| Alternatives | A, B, C, D |
| Profile | Z / `pilot-z-v0`, model/Agent/tool delay 0/0/0 microseconds |
| Build | `qualification-or-release` |
| Analysis | `dp00-analysis-v8` |
| Event / provenance | `canonical-event-v3` / `dp00-pilot-provenance-v4` |
| Model / prompt / cache | `dp00-base@v0` / `pilot-v0-payload-v1` / disabled |
| Python / pytest | 3.14.7 / 9.1.1 |
| Rust / Cargo | 1.94.0 / 1.94.0 |
| Tokio / worker | 1.53.1 / `tokio-current-thread-v0` |
| Host | `aarch64-apple-darwin`; macOS 26.5.1 (25F80), arm64 |

The existing v4 runner records this architecture campaign ID in the frozen
`calibration_id` and `pair_id` fields and in `calibration-manifest.json`; its
legacy `campaign_id` field remains null and `official` remains false. No source
or provenance schema was changed merely to rename that field. Every measured
execution nevertheless carries the same explicit ID, source, alternative,
scenario, profile, repetition, build, model/prompt/cache, corpus, event, and
analysis identities. This compatibility choice is visible rather than masked.

The purpose is per-QA comparison and trade-off evidence, not calibration
revision, a weighted score, an ownerless QA-02 gate, or winner selection.

## Architecture identity freeze

No A/B/C/D source changed between the Session 6.6 frozen source and this source;
the intervening commit contains only accepted calibration evidence and report
artifacts. Current implementation and the authoritative executable
specification agree:

| Alternative | Primary decision owner | Local/delegated responsibility and topology |
| --- | --- | --- |
| A — Thin VIA | VIA Intent Refiner plus Agent Router | VIA owns intent, route, task and correlation; selected ARGO/specialist owns domain execution. New work normally uses two independent pre-route model generations. No VIA local route. |
| B — ARGO-centric | ARGO Primary | Thin VIA packages context and projects tasks; ARGO jointly owns interpretation and self-versus-specialist routing, then executes or delegates. One MIXED pre-route generation normally replaces the split VIA chain. |
| C — Hybrid | VIA Intent Refiner, deterministic Fast Eligibility, Agent Router | A plus bounded VIA Fast Path. Eligible local work stays in VIA; other work routes to ARGO/specialist. |
| D — Adaptive | VIA Intent Refiner plus Execution Path Selector | Selector owns topology and executor choice across VIA Fast, ARGO Primary, and specialist direct. There is no second Agent Router. |

All four preserve the P12 parent plus distinct-child route-plan barrier. S1 and
S2 both commit before either child executes.

## Scope and repetition rationale

P01–P12 × A–D were executed for 16 measured rotated repetitions in both CAPTURE
and MINIMAL. That is 768 CAPTURE architecture observations plus 768 paired
MINIMAL integrity observations, 1,536 measured executions total. W0/W1 each
covered 96 paths, and one invocation warm-up was excluded from every measured
cycle.

The count was not newly invented. The campaign reused the only prospectively
approved and empirically accepted repetition/order population:
`FIXED_16_MEASURED_CYCLES_V1`, cyclic ABCD/BCDA/CDAB/DABC positions, and balanced
adjacent CAPTURE/MINIMAL order. CAPTURE is the architecture reporting
population. MINIMAL is retained only as a paired semantic/provenance diagnostic;
it is not pooled into the reported architecture latency.

Profile C was not used. Existing documentation identifies it as a provisional
calibration parameter, not an approved architecture-comparison profile. Under
the instructed Case B, the campaign is therefore a Profile Z controlled
structural/correctness comparison.

## Evidence qualification and integrity

QA-01 uses only P01–P03, authoritative Acoustic EOS and Outcome Probe useful
outcome boundaries, successful exact-correctness-qualified CAPTURE executions,
and the frozen linear `(n-1)` percentile estimator. QA-02 reports exact raw
counts with no numeric pass threshold. QA-04 uses
`qa04-route-contract-policy-v1`: REQUIRED needs complete route commits and
QA-02 conformance, FORBIDDEN violations are explicit, and OPTIONAL P09 remains
a separate stratum.

| Integrity check | Result |
| --- | --- |
| Expected / actual measured executions | 1,536 / 1,536 |
| Expected / complete pairs | 768 / 768 |
| Missing / duplicate mode | 0 / 0 |
| Provenance / semantic mismatch | 0 / 0 |
| QA-02 / QA-04 paired mismatch | 0 / 0 |
| Strict validation errors / warnings | 0 / 0 |
| Scenario / alternative / profile coverage | 12/12 / 4/4 / Profile Z complete |
| Deterministic replay | byte-identical |
| Derived SHA-256, both derivations | `3154326929b05fa11c7e8529c8760e01e61f4e9c307df6460ef407366f5a31bf` |

The paired analyzer has 560 fully qualified pairs. The other 208 are rejected
for the preserved architecture QA-02/QA-04 outcomes, not missing, duplicate,
semantic, provenance, or infrastructure defects. Cycle drift and mode-order
acceptance also pass, supporting interpretation of nanosecond-scale Profile Z
differences without reopening calibration.

## QA-01 — FTOL latency

All values are nanoseconds. `p99` is diagnostic. Each alternative has 48
CAPTURE observations: P01/P02/P03 × 16.

| Alternative | n | p50 | p95 | p99 diagnostic |
| --- | ---: | ---: | ---: | ---: |
| A | 48 | 5,437.0 | 6,625.0 | 7,210.76 |
| B | 48 | 6,063.0 | 7,024.55 | 7,191.25 |
| C | 48 | 5,459.0 | 6,858.55 | 7,066.25 |
| D | 48 | 5,958.5 | 7,164.25 | 7,313.73 |

Scenario-specific CAPTURE results:

| Scenario | A p50 / p95 | B p50 / p95 | C p50 / p95 | D p50 / p95 |
| --- | ---: | ---: | ---: | ---: |
| P01 | 4,792 / 5,094.25 | 4,937.5 / 5,489.5 | 4,979.5 / 5,406 | 5,333.5 / 5,541.5 |
| P02 | 6,479 / 6,989 | 6,709 / 7,124.75 | 6,542 / 6,969 | 7,041.5 / 7,302.25 |
| P03 | 5,437 / 5,729.25 | 6,041.5 / 6,813 | 5,459 / 6,031 | 5,958.5 / 6,229.25 |

A has the lowest pooled p50 and p95 in this zero-delay software baseline; C is
close at p50, B and D are slower. These are real controlled software-path
differences, but not a realistic end-to-end ranking: Profile Z assigns no time
to B's model-call reduction or C/D's local-versus-Agent placement.

## QA-02 — functional exact conformance

CAPTURE is reported; each cell is 16 deterministic repetitions. No overall
PASS/FAIL gate is inferred.

| Alternative | Conformant / total | Rate | Nonconformant |
| --- | ---: | ---: | ---: |
| A | 160 / 192 | 83.333% | 32 |
| B | 128 / 192 | 66.667% | 64 |
| C | 160 / 192 | 83.333% | 32 |
| D | 144 / 192 | 75.000% | 48 |

| Scenario | A | B | C | D |
| --- | :---: | :---: | :---: | :---: |
| P01 | 16/16 | 16/16 | 16/16 | 16/16 |
| P02 | 16/16 | 16/16 | 16/16 | 16/16 |
| P03 | 16/16 | 16/16 | 16/16 | 16/16 |
| P04 | 0/16 | 0/16 | 0/16 | 0/16 |
| P05 | 16/16 | 16/16 | 16/16 | 16/16 |
| P06 | 16/16 | 0/16 | 16/16 | 16/16 |
| P07 | 16/16 | 0/16 | 16/16 | 0/16 |
| P08 | 16/16 | 16/16 | 16/16 | 16/16 |
| P09 | 16/16 | 16/16 | 16/16 | 16/16 |
| P10 | 16/16 | 16/16 | 16/16 | 16/16 |
| P11 | 0/16 | 0/16 | 0/16 | 0/16 |
| P12 | 16/16 | 16/16 | 16/16 | 16/16 |

The known P04 A/B/C/D, P06 B, P07 B/D, and P11 A/B/C/D pattern is reproduced
exactly, in all 16 CAPTURE and all 16 MINIMAL executions. It is deterministic
architecture behavior, not intermittent noise.

Failure classification:

- P04 A/C: unsupported accepted route/capability produces `INVALID_ROUTE` and
  no required open effect. P04 B additionally loses required referent evidence.
  These are alternative-specific structural/unsupported-route limitations.
- P04 D: the selector chooses `LOCAL_DIRECT:VIA_FAST`, outside the allowed
  document execution routes: a local/delegated placement mismatch.
- P06 B: clarification reply reaches an invalid route and loses the required
  referent: a B-specific clarification/reference handoff limitation.
- P07 B: invalid route prevents the required domain-planned execution: an
  unsupported-route structural limitation. P07 D accepts the forbidden local
  Fast Path and claims domain-planning success: a forbidden-route and execution
  effect/placement mismatch.
- P11 A/B/C/D: results collapse onto the alternative-prefixed T1 identity, T2
  is not independently bound, and a forbidden new route is committed. This is a
  task/correlation and route-commit failure common to all current alternatives.

No observed failure is classified as benchmark/infrastructure defect: raw
evidence is complete, CAPTURE/MINIMAL semantics agree, and all failures repeat.

## QA-04 — correctness-qualified route observation

CAPTURE results under `qa04-route-contract-policy-v1`:

| Alternative | REQUIRED | Qualified observations | Forbidden-route violations | Required route not committed | Required QA-02 exclusions | Alt × Profile Z qualification |
| --- | ---: | ---: | ---: | ---: | ---: | :---: |
| A | 128 | 96 | 16 | 32 | 16 | FAIL |
| B | 128 | 80 | 16 | 48 | 48 | FAIL |
| C | 128 | 96 | 16 | 32 | 16 | FAIL |
| D | 128 | 96 | 16 | 0 | 32 | FAIL |

All four FAIL for decomposable reasons. A/C miss required commits in P04/P06;
B misses them in P04/P06/P07 and has the broadest required QA-02
nonconformance; D commits every required route but P04/P07 are not QA-02
conformant. Each alternative also violates the FORBIDDEN route contract in all
16 P11 repetitions. P09 is OPTIONAL and has no route commit for all
alternatives; it is not converted into a penalty or imputed number.

The route-committed diagnostic mean calls to boundary is A 1.857, B 1.000,
C 1.857, D 2.000. Qualified primary means remain null because every
Alternative × Profile stratum fails the predeclared qualification.

## Structural diagnostics

CAPTURE totals over 192 executions per alternative:

| Alternative | Logical model calls | Pre-boundary | Post-boundary | Local route commits | Agent/delegated route commits | Task / route events | Main topology |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| A | 384 | 384 | 0 | 0 | 128 | 128 / 128 | split VIA intent + Agent routing, then executor |
| B | 208 | 208 | 0 | 0 | 112 | 112 / 112 | fused ARGO-primary interpretation/routing; 32 explicit delegated commits, 80 ARGO-direct |
| C | 384 | 384 | 0 | 16 | 112 | 128 / 128 | A plus bounded local media path |
| D | 384 | 384 | 0 | 80 | 80 | 160 / 160 | selector chooses local, ARGO, or specialist topology |

Every logical call is pre-route or pre-terminal-resolution in this corpus; no
post-boundary domain model generation is represented by the controlled stub.
B uses 45.8% fewer logical model calls than A/C/D. D exercises the most local
routes and route/task events, while A has no VIA-local execution. Clarification
occurs 16 times per alternative; only B lacks a matching clarification-resolved
event because its P06 reply terminates on the B-specific failure path.

## P12 compound regression

P12 passes 64/64 CAPTURE+MINIMAL pairs and 128/128 executions; CAPTURE alone is
16/16 for each A/B/C/D. Every run has S1 then S2 commit, a common parent,
distinct child tasks, both commits before either execution, two required
effects, final boundary at S2, exact QA-02 conformance, and QA-04 qualification.

Topology remains architecture-specific: A routes both children through its
Agent Router; B uses one ARGO-primary decision for both; C uses local media S1
plus ARGO S2; D's selector supplies both routes. No P12 calibration redesign or
semantic change occurred.

## Trade-off and Pareto view

| Alternative | QA-01 Profile Z | QA-02 | QA-04 | Model calls | Structural observation |
| --- | --- | --- | --- | ---: | --- |
| A | best p50/p95 | tied best, 83.333% | 96 qualified; 32 required non-commits | 384 | simplest agent-neutral route set; no VIA local path |
| B | third p50, second p95 | lowest, 66.667% | lowest qualified count; most required gaps | 208 | clearly fewest calls through ARGO-primary ownership |
| C | near-A p50, second p95 | tied best, 83.333% | same counts as A | 384 | bounded local path adds placement option and boundary-growth risk |
| D | second p50, worst p95 | 75.000% | all required routes commit, but correctness excludes 32 | 384 | broadest local/topology selection and highest route/task event count |

No alternative is conclusively Pareto-dominated across all measured and
structural dimensions. B pays correctness/qualification cost for a large
model-call reduction. D trades lower correctness and selector complexity for
complete required-route commitment and much more local execution. A and C are
correctness/QA-04 tied; A is faster in Profile Z and structurally simpler, so C
is **potentially dominated under the observed controlled axes**, but not
conclusively dominated because VIA-local ownership is a qualitative product
choice whose realistic latency/value is unmeasured. A, B, and D are
non-dominated; C remains conditionally non-dominated pending that choice and a
realistic profile.

## Limitations and decision questions

The corpus is deterministic replay/stub architecture qualification, not a real
model/Agent/device/network test. Profile Z resolves software-path differences
but suppresses dependency cost. P01–P03 give 48 observations per alternative;
p99 remains diagnostic. QA-04 OPTIONAL aggregation and final primary aggregate
are unresolved. No owner-backed QA-02 gate, weighting, scoring map, or product
acceptance claim exists.

The Architect must decide, after an approved realistic profile is measured:

1. How much correctness and route-qualification loss, if any, is acceptable in
   exchange for B's 45.8% model-call reduction and ARGO-primary coupling?
2. Is VIA-local ownership in C/D valuable enough to justify added boundary and
   selector complexity, and under what realistic latency/capability mix?
3. Is D's complete required-route commitment more valuable than A/C's higher
   exact conformance, given D's P04/P07 placement errors?
4. Does C's local-path product value prevent A from dominating it once realistic
   model/Agent/tool costs are included?

## Remaining independent blockers

| Blocker | Status | Blocks this architecture decision? |
| --- | --- | :---: |
| QA-02 owner-backed numeric gate | absent; no threshold invented | No — raw counts and failure semantics remain directly comparable |
| QA-04 aggregation | overall/macro/OPTIONAL mapping absent; all strata fail qualification | No for this vector/Pareto analysis; yes for a future scalar QA-04 acceptance claim |
| Scoring mapping / weights | absent | No — scoring is intentionally outside this decision method |
| Real-stack / realistic architecture profile | absent; Profile C is only provisional calibration input | **Yes — core model-call/local/delegated latency trade-off is not measured** |

Consequently the final disposition is:

`NOT READY FOR ARCHITECTURE DECISION`

The blocker is specific and bounded: define and approve a realistic
architecture-comparison profile, then measure it without changing A/B/C/D or
the frozen QA semantics. The valid correctness and Profile Z evidence does not
need to be rerun or returned to calibration.
