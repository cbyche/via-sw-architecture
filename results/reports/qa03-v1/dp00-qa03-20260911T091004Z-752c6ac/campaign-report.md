# DP-00 QA-03 Flexibility / CCR Measured Campaign

## Disposition

**QA-03 FLEXIBILITY CAMPAIGN COMPLETE**

This campaign contains 20/20 valid, complete, comparable cells. It reports
Change Containment Ratio directly and does not select a DP-00 architecture
winner. DP-00 remains **NOT READY FOR ARCHITECTURE DECISION**.

## Identity and isolation

| Item | Value |
| --- | --- |
| Campaign | `dp00-qa03-20260911T091004Z-752c6ac` |
| Contract source | `752c6ac66fd0a77d7d6efac00d46ad7277bc5247` |
| Architecture baseline | `d4059eccd2883b3029b1e8c94fc86d092a5041f8` |
| Protocol | `dp00-qa03-flexibility-protocol-v1` |
| Catalog | `dp00-qa03-evolution-catalog-v1` |
| Role map | `dp00-qa03-role-map-v1` |
| Analyzer | `dp00-qa03-analyzer-v1` |
| Evidence worktree | `/private/tmp/via-sw-architecture-qa03-2` |
| Cell isolation | one unique branch/worktree per cell, each directly from `d4059ecc…` |
| Environment | macOS 26.5.1 arm64; Python 3.14.7; pytest 9.1.1; rustc/cargo 1.94.0 |

The execution order was frozen before the first cell: E1 A/B/C/D; E2
B/C/D/A; E3 C/D/A/B; E4 D/A/B/C; E5 A/B/C/D. Every acceptance oracle was
committed before its production implementation. Cell result commits were not
merged into this evidence branch.

## Preflight

| Gate | Result |
| --- | --- |
| Remote contract source | exact `752c6ac…` |
| Disposable `d4059ecc…` probe | clean; analyzer returned no baseline mismatch |
| QA-03 evaluator qualification | 20/20 PASS |
| Python full suite at contract source | 115/115 PASS |
| Rust workspace/all-target check | PASS |
| Rust fmt | PASS |
| Rust clippy, warnings denied | PASS |
| Rust workspace tests | 107/107 PASS |

## Integrity

| Item | Result |
| --- | ---: |
| Expected / attempted cells | 20 / 20 |
| Valid cells | 20 |
| NOT_CONTAINED | 6 |
| INVALID_EXPERIMENT | 0 |
| INCONCLUSIVE | 0 |
| Common denominator | 5 |
| Deterministic cell evaluations | 40/40; both runs byte-identical per cell |
| Primary taxonomy coverage | E1/E2/E3/E4/E5 complete |

## Cell matrix

| Scenario | A | B | C | D |
| --- | --- | --- | --- | --- |
| E1 Agent | CONTAINED | CONTAINED | CONTAINED | CONTAINED |
| E2 Model | CONTAINED | CONTAINED | CONTAINED | CONTAINED |
| E3 Context | NOT_CONTAINED | NOT_CONTAINED | NOT_CONTAINED | NOT_CONTAINED |
| E4 Placement | NOT_CONTAINED | NOT_CONTAINED | CONTAINED | CONTAINED |
| E5 Contract | CONTAINED | CONTAINED | CONTAINED | CONTAINED |

## Acceptance and regression

| Cell group | Acceptance | Regression | P12 / baseline defects |
| --- | --- | --- | --- |
| E1 A/B/C/D | PASS | PASS | baseline-equivalent |
| E2 A/B/C/D | PASS | PASS | baseline-equivalent |
| E3 A/B/C/D | PASS | PASS | hidden-context and baseline signatures unchanged |
| E4 A/B | FAIL: `ACC-E4-LOCAL` | PASS | P12 PASS; existing non-local behavior unchanged |
| E4 C/D | PASS | PASS | P12 and P11 model-call/route signatures baseline-equivalent |
| E5 A/B/C/D | PASS | PASS | routes and effects unchanged |

Pre-existing QA-02 failures remain A/C P04/P11, B P04/P06/P07/P11, and D
P04/P07/P11. The frozen decomposed QA-04 failures also remain unchanged. No
cell added or worsened a required regression.

An early uncommitted E4/D implementation skipped D's existing placement-model
call and failed the P11 regression signature. It was rejected before the result
commit. The committed implementation retains D's per-turn selector call; its
acceptance and regression both pass.

## Change containment

| Cell | Expected Change Area | Actual scoring roles | Out of area | Result |
| --- | --- | --- | --- | --- |
| E1/A | capability registration; Agent adapter | capability registration | none | CONTAINED |
| E1/B | capability registration; Agent adapter | capability registration; Agent adapter | none | CONTAINED |
| E1/C | capability registration; Agent adapter | capability registration | none | CONTAINED |
| E1/D | capability registration; Agent adapter | capability registration | none | CONTAINED |
| E2/A | Model provider; provider configuration | Model provider | none | CONTAINED |
| E2/B | Model provider; provider configuration | Model provider | none | CONTAINED |
| E2/C | Model provider; provider configuration | Model provider | none | CONTAINED |
| E2/D | Model provider; provider configuration | Model provider | none | CONTAINED |
| E3/A | Context Source adapter; context handling | Context Source adapter; context handling; unmapped module registration | `UNMAPPED_ARCHITECTURE_CHANGE` | NOT_CONTAINED |
| E3/B | Context Source adapter; context handling | Context Source adapter; context handling; unmapped module registration | `UNMAPPED_ARCHITECTURE_CHANGE` | NOT_CONTAINED |
| E3/C | Context Source adapter; context handling | Context Source adapter; context handling; unmapped module registration | `UNMAPPED_ARCHITECTURE_CHANGE` | NOT_CONTAINED |
| E3/D | Context Source adapter; context handling | Context Source adapter; context handling; unmapped module registration | `UNMAPPED_ARCHITECTURE_CHANGE` | NOT_CONTAINED |
| E4/A | registration; control plane; local execution | none (test evidence only) | none; acceptance failed | NOT_CONTAINED |
| E4/B | registration; control plane; local execution | none (test evidence only) | none; acceptance failed | NOT_CONTAINED |
| E4/C | registration; control plane; local execution | none required; seam already supported request | none | CONTAINED |
| E4/D | registration; control plane; local execution | capability registration; control plane | none | CONTAINED |
| E5/A | contract; registration; Agent adapter | downstream contract; Agent adapter | none | CONTAINED |
| E5/B | contract; registration; Agent adapter | downstream contract; Agent adapter | none | CONTAINED |
| E5/C | contract; registration; Agent adapter | downstream contract; Agent adapter | none | CONTAINED |
| E5/D | contract; registration; Agent adapter | downstream contract; Agent adapter | none | CONTAINED |

Tests, reports, evidence, and acceptance-oracle hunks are non-scoring. Exact
patches and hunk reviews are retained with each manifest.

## CCR

`S_primary` contains E1, E2, E3, E4, and E5. Equal scenario weighting and the
common denominator of five apply to every alternative.

| Alternative | Contained | Valid scenarios | CCR |
| --- | ---: | ---: | ---: |
| A | 3 | 5 | 60% |
| B | 3 | 5 | 60% |
| C | 4 | 5 | 80% |
| D | 4 | 5 | 80% |

## Scenario interpretation

### E1 — Agent evolution

All alternatives absorbed the registered DocumentSummaryAgent without changing
intent, context, task, or orchestration roles. A/C use their Agent Router
registration seam, D uses its execution-path selector's registration facet,
and B extends ARGO Primary's existing delegated-capability facet. B's physical
one-file change represents two semantic roles—capability registration and Agent
delegation—so its small file count must not be mistaken for a narrower
responsibility surface.

### E2 — Model/provider evolution

All alternatives use the common ModelPort boundary. The selectable
REPLAY_MODEL_V2 wrapper preserved semantic responses, decision-owner audit data,
and replay context provenance; the existing provider remained selectable.
There was no alternative-specific bypass or architecture propagation.

### E3 — Context/input evolution

All acceptance and regression criteria passed: one provenance-preserving item
was produced, ordering and existing values remained stable, and each
alternative consumed it through its context seam. Nevertheless all four cells
are NOT_CONTAINED because making the new adapter a compiled public fixture
module required `bench-fixtures/src/lib.rs`, which has no candidate role in
the frozen role map. The analyzer therefore deterministically scored
`UNMAPPED_ARCHITECTURE_CHANGE`.

This is a recurring common role-map/module-wiring hotspot, not an observed
difference among A/B/C/D. It lowers every CCR equally and must not be removed
post hoc.

### E4 — Capability placement

The scenario remained meaningful for all four alternatives. C's existing
deterministic Fast Eligibility plus local executor already supported the
requested facts, so only acceptance evidence was required. D naturally extended
its per-turn execution-path selector; it preserved the selector's model call,
committed LOCAL_DIRECT when enabled, and kept the ARGO fallback otherwise.

Thin VIA A and ARGO-centric B have no VIA-local execution role by frozen
definition. Their existing non-local paths remained intact, but
`ACC-E4-LOCAL` failed. No local executor or control plane was added, because
doing so would redefine the alternatives as C/D-like architectures. These are
valid NOT_CONTAINED architecture-resistance results, not invalid experiments.

### E5 — Contract evolution

The optional serde-defaulted resource_class contract round-tripped IO_BOUND
metadata while old descriptors remained valid. Diagnostic adapter hooks exposed
the metadata without consulting it for routing. Full route/effect regressions
were unchanged for all alternatives.

## Cross-alternative interpretation

Agent registration and Model provider substitution are strong seams in all four
alternatives. The shared optional downstream contract also limited propagation
to the contract and Agent adapter roles.

C and D differ from A and B on the representative placement pressure because
they already own explicit local execution and placement-policy seams. A and B
correctly resisted the request within their frozen boundaries; this is the only
comparative CCR difference.

B centralizes interpretation, routing, registration, and delegation in ARGO
Primary. E1 required only one physical production file, but hunk-level
classification shows two semantic roles. Centralization therefore did not
receive artificial credit from raw file count. Conversely, E3 touched multiple
physical files while its intended semantics remained only two expected roles;
the penalty comes specifically from the unmapped module-registration hunk, not
from file count.

The higher C/D CCR is supported by a genuine E4 extension seam, but CCR alone
does not establish overall architecture quality or a final winner.

## Limitations

Five scenarios are representative expected evolutions, not a complete measure
of future maintainability. QA-03 does not measure development speed, developer
effort, LOC quality, runtime latency, production dependency fidelity, or every
unexpected change category. Binary CCR hides propagation severity, so the
per-cell patches and semantic-role diagnostics remain essential.

## DP-00 impact and next action

QA-03 adds prospective, baseline-relative flexibility evidence to the eventual
decision synthesis. It does not override QA-01/QA-02/QA-04 or select a winner.

DP-00 remains **NOT READY FOR ARCHITECTURE DECISION** pending the remaining
evidence tracks, including valid Session 7.2R evidence and final decision
synthesis.
