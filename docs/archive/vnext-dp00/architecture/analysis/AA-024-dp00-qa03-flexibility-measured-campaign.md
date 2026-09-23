# AA-024 — DP-00 QA-03 Flexibility / CCR Measured Campaign

## Status and disposition

Architecture Analysis Record — completed QA-03 measured campaign.

**QA-03 FLEXIBILITY CAMPAIGN COMPLETE**

This record reports prospectively frozen flexibility evidence. It does not
change the approved requirements baseline, reinterpret QA-01/QA-02/QA-04,
select an architecture winner, or make DP-00 ready for decision.

## Identity

| Item | Value |
| --- | --- |
| Campaign ID | `dp00-qa03-20260911T091004Z-752c6ac` |
| Contract/control-plane SHA | `752c6ac66fd0a77d7d6efac00d46ad7277bc5247` |
| Measured architecture baseline | `d4059eccd2883b3029b1e8c94fc86d092a5041f8` |
| Protocol | `dp00-qa03-flexibility-protocol-v1` |
| Catalog | `dp00-qa03-evolution-catalog-v1` |
| Role map | `dp00-qa03-role-map-v1` |
| Analysis | `dp00-qa03-analysis-v1` |
| Analyzer | `dp00-qa03-analyzer-v1` |
| Environment | macOS 26.5.1 arm64; Python 3.14.7; pytest 9.1.1; rustc/cargo 1.94.0 |

The orchestration/evidence worktree was
`/private/tmp/via-sw-architecture-qa03-2`, based on the contract SHA. Each of
the 20 measured cells used a unique branch and disposable worktree directly
from the architecture baseline. Acceptance-oracle commits precede production
commits in each branch. Experimental result commits were not merged
cumulatively.

Execution order was frozen before measurement: E1 A/B/C/D; E2 B/C/D/A; E3
C/D/A/B; E4 D/A/B/C; E5 A/B/C/D.

## Matrix and integrity

The measured matrix is five frozen scenarios by four alternatives, 20 cells.

| Integrity property | Result |
| --- | --- |
| Expected / attempted / valid | 20 / 20 / 20 |
| NOT_CONTAINED | 6 |
| INVALID_EXPERIMENT / INCONCLUSIVE | 0 / 0 |
| Provenance or contamination issue | none |
| Primary scenarios | E1, E2, E3, E4, E5 |
| Common denominator | 5 |
| Deterministic evaluation | two byte-identical evaluations per cell |
| Taxonomy coverage | complete |

The full preflight passed: QA-03 qualification 20/20, contract-source Python
115/115, Rust all-target check, format, warnings-denied clippy, and Rust
107/107. A detached architecture-baseline probe produced no baseline mismatch.
All final cell manifests satisfy the frozen v1 structural, constant, enum, and
required-field constraints.

## Scenario matrix

| Scenario | A | B | C | D |
| --- | --- | --- | --- | --- |
| E1 Specialist Agent | CONTAINED | CONTAINED | CONTAINED | CONTAINED |
| E2 Model Provider | CONTAINED | CONTAINED | CONTAINED | CONTAINED |
| E3 Context Source | NOT_CONTAINED | NOT_CONTAINED | NOT_CONTAINED | NOT_CONTAINED |
| E4 Capability Placement | NOT_CONTAINED | NOT_CONTAINED | CONTAINED | CONTAINED |
| E5 Contract Metadata | CONTAINED | CONTAINED | CONTAINED | CONTAINED |

All E1/E2/E3/E5 acceptance criteria passed. E4 acceptance passed for C/D and
failed `ACC-E4-LOCAL` for A/B. Every cell's frozen regression bundle passed:
no new or worsened QA-02/QA-04 defect, P12 remained passing, and the existing
alternative-specific defects remained visible.

## CCR

Equal weights and the common denominator of five were used.

| Alternative | Contained | Denominator | CCR |
| --- | ---: | ---: | ---: |
| A | 3 | 5 | 60% |
| B | 3 | 5 | 60% |
| C | 4 | 5 | 80% |
| D | 4 | 5 | 80% |

No qualitative grade or F1–F5 mapping is attached.

## Expected versus actual propagation

- E1 remained in capability registration for A/C/D and capability
  registration plus Agent delegation for B.
- E2 remained in the common Model Gateway provider role.
- E3 changed Context Source adapter and context handling as expected, but also
  required the architecture-affecting `bench-fixtures/src/lib.rs` module
  registration. That path is unmapped by the frozen role map, so every E3 cell
  has exact unexpected role `UNMAPPED_ARCHITECTURE_CHANGE`.
- E4/A and E4/B made no scoring production change because adding a VIA-local
  control plane/executor would redefine the frozen alternatives; acceptance
  therefore failed. E4/C needed no scoring change because the frozen hybrid
  seam already supports the requested facts. E4/D changed capability
  registration and execution-policy/control plane only.
- E5 remained in the downstream Agent capability contract and Agent delegation
  adapter roles. The optional metadata never influenced routing.

Raw patches, line diagnostics, role assertions, test evidence, manifests, and
both analyzer outputs are retained under the campaign's raw and derived
evidence trees.

## Baseline regressions

The preserved QA-02 failure patterns are A/C P04/P11, B P04/P06/P07/P11, and D
P04/P07/P11. Frozen decomposed QA-04 failures are also unchanged. P12 remains
PASS for A/B/C/D.

An uncommitted first E4/D implementation skipped D's existing selector model
call and worsened the P11 signature. It was rejected before the result commit.
The committed version retains the selector call and passes both acceptance and
regression. This iteration changed implementation only, never the oracle,
Expected Change Area, or evaluator.

## E4 capability-placement analysis

The pressure remained semantically meaningful for every alternative.

- A used its natural Thin VIA delegated boundary. It cannot own the requested
  local execution without adding absent control/local-execution roles and
  changing the alternative definition. Result: valid NOT_CONTAINED.
- B used its natural ARGO Primary path. A VIA-local executor would contradict
  the frozen ARGO-centric ownership boundary. Result: valid NOT_CONTAINED.
- C used existing deterministic Fast Eligibility, capability facts, and local
  execution. The permitted request was LOCAL_DIRECT and disabled requests
  retained fallback. Result: CONTAINED.
- D used its existing per-turn Execution Path Selector. The selector retained
  its model decision, then applied the registered policy/capability facts; the
  disabled request retained ARGO fallback. Result: CONTAINED.

No domain-planning capability became local, P12 did not degrade, and no
alternative was silently converted.

## Architectural interpretation

All four alternatives have effective seams for adding a registered specialist,
selecting a ModelPort provider, and extending optional downstream metadata.
C/D additionally expose natural local-placement seams for the representative
E4 pressure.

E3 identifies a shared module-registration hotspot. Its equal penalty across
A/B/C/D is evaluator-visible common infrastructure propagation, not an
alternative-specific context weakness. It cannot be removed from v1 after
observing results.

B's E1 implementation demonstrates the multi-role caveat: one physical ARGO
Primary file contains registration and delegation responsibilities. Its small
file count is not evidence of narrower semantic propagation. Conversely, the
multiple E3 files primarily express two intended roles; the containment failure
comes from the specific unmapped module-registration hunk.

C/D's higher CCR is attributable to the prospectively represented placement
seam rather than LOC or post-hoc role changes. That evidence is relevant to
decision synthesis, but it is not sufficient to establish total architecture
quality.

## Limitations

The five scenarios are representative expected evolutions, not a complete
measure of all future maintainability. QA-03 does not measure development
speed, developer effort, LOC quality, runtime performance, production-provider
fidelity, or every future change. Binary CCR hides propagation severity;
cell-level role and patch diagnostics must accompany it.

## DP-00 impact

QA-03 now contributes valid comparative flexibility evidence: A/B 60%, C/D
80%, with the scenario-level causes retained. It neither repairs nor overrides
other QA evidence and does not select a final winner.

DP-00 remains:

**NOT READY FOR ARCHITECTURE DECISION**

The remaining evidence tracks—including valid Session 7.2R evidence—must be
completed before DP-00 Decision Synthesis.
