# AA-023 — DP-00 QA-03 Flexibility Experiment Contract and Prospective Freeze

## Status

Architecture Analysis Record — QA03-1 finalized prospective experiment contract, including the pre-campaign comparative-validity clarification.

This record freezes the experiment contract; it does not report the final scored A/B/C/D experiment, change the approved requirements baseline, select a DP-00 winner, or merge any Session 7.2 runtime work.

## Identity

| Item | Value |
| --- | --- |
| Branch | `exp/dp00-qa03-flexibility` |
| Source SHA | `d4059eccd2883b3029b1e8c94fc86d092a5041f8` |
| Protocol | `dp00-qa03-flexibility-protocol-v1` |
| Catalog | `dp00-qa03-evolution-catalog-v1` |
| Role map | `dp00-qa03-role-map-v1` |
| Analysis | `dp00-qa03-analysis-v1` |
| Evaluator | `dp00-qa03-analyzer-v1` |
| Regression baseline | `dp00-qa03-baseline-regression-v1` |

Normative artifacts:

- `docs/evaluation/dp00-qa03-flexibility-experiment-protocol-v1.md`
- `benchmark/contracts/dp00-qa03-evolution-catalog-v1.json`
- `benchmark/contracts/dp00-qa03-role-map-v1.json`
- `benchmark/contracts/dp00-qa03-baseline-regression-v1.json`
- `benchmark/schemas/dp00-qa03-execution-manifest-schema-v1.json`
- `benchmark/analysis/dp00_analysis/qa03.py`

## QA definition and metric

`QA-03` is vNext **Flexibility / Change Containment**. **Legacy v1.1 QA-03** is conversation/task recovery and is not the subject of this experiment.

The exact binary rule is:

```text
CONTAINED = acceptance PASS
            AND baseline-relative regression PASS
            AND no required architecture-affecting change outside
                the prospectively frozen Expected Change Area
```

The Primary Metric is unweighted Change Containment Ratio:

```text
S_primary = scenarios with exactly one valid, complete, comparable cell
            for each of A/B/C/D

CCR_alt = CONTAINED cells for alt within S_primary / |S_primary| * 100
```

It is reported independently for A/B/C/D using the same `S_primary` denominator. An invalid/inconclusive cell excludes that scenario symmetrically rather than removing one alternative's cell. No measured CCR is produced by this freeze session.

## Evolution taxonomy and scenario catalog

The authoritative E1-E5 taxonomy is preserved.

| ID | Category | Frozen change request | Expected architecture roles |
| --- | --- | --- | --- |
| `QA03-E1-SPECIALIST-AGENT-001` | E1 Agent | Add a capability-registered DocumentSummaryAgent with canonical downstream result behavior. | capability registration; Agent delegation adapter |
| `QA03-E2-MODEL-PROVIDER-001` | E2 Voice/Model | Add selectable deterministic REPLAY_MODEL_V2 behind Model Gateway/ModelPort. | Model Gateway provider; provider configuration |
| `QA03-E3-CONTEXT-SOURCE-001` | E3 Context/Input | Add a provenance-preserving active-window-title Context Source. | Context Source adapter; context handling |
| `QA03-E4-CAPABILITY-PLACEMENT-001` | E4 Placement | Make bounded document-open VIA-local when allowed/registered, with non-local fallback. | capability registration; execution-policy/control plane; local capability execution |
| `QA03-E5-CONTRACT-METADATA-001` | E5 Contract | Add optional resource-class metadata while retaining v1 descriptor and routing behavior. | downstream capability contract; capability registration; Agent delegation adapter |

The catalog freezes complete rationale, alternative-neutral semantics, acceptance test IDs, regression criteria, forbidden roles, and evidence requirements. E4 does not grant permission to turn A into C or B into D. It tests whether the bounded placement evolution can be absorbed through intended change mechanisms. A meaningful valid request that requires out-of-area propagation, regression degradation, or impermissible boundary change is `NOT_CONTAINED`, including for a Thin VIA boundary. Only unequal/non-applicable comparative semantics or contract-integrity failure makes E4 invalid. Silently converting an alternative to pass is invalid; pressure on its unchanged boundary is flexibility evidence.

## Expected Change Areas and role normalization

Expected Change Areas are semantic role sets, not file counts. The role map resolves those sets against the actual prototype topology:

- A separates context, intent, Agent routing, Agent harness, task manager, and orchestration core; local execution is absent.
- B combines interpretation, route choice, and specialist delegation in its ARGO-primary controller, with separate thin context, task projection, and orchestration core; VIA-local execution is absent.
- C retains A-like seams and adds fast eligibility/control plane and local execution.
- D retains separate context/intent while its selector combines route and placement decisions, with local executor and Agent client.

Shared ModelPort, execution/capability contract, replay provider, fixtures, observation, and evaluation paths are mapped explicitly. Voice Runtime is external to the executable prototypes and is explicitly absent/common, not assigned to a convenient file.

Some physical files implement multiple normalized roles. The map prospectively enumerates their candidate roles. A measured hunk can assert a subset only with review evidence and only from that candidate set. Unmapped architecture changes are unexpected. This prevents a coarse file map from either automatically penalizing every co-located role or allowing post-hoc role invention.

## Regression model

The frozen signature retains known deterministic defects rather than hiding or relabeling them:

- A/C: QA-02 P04 and P11;
- B: QA-02 P04, P06, P07, and P11;
- D: QA-02 P04, P07, and P11;
- all alternatives: the decomposed QA-04 qualification failures recorded by AA-021;
- all alternatives: P12 currently passes.

An evolution fails regression only for a newly introduced failure, a previously passing requirement becoming false, a contractually worsened existing failure, a new forbidden effect/missing observation, a P12 degradation, or a newly disqualified required QA-04 observation. It need not repair unrelated baseline defects. A repaired baseline defect cannot compensate for a new one.

Regression includes all existing unit/integration tests, DP-00 behavior, QA-02, QA-04, P12, and architecture/profile invariance. QA-01 latency ranking is outside this experiment.

## Change classification and scoring

Production source, contract/schema, runtime configuration, registration, and generator-source changes count. Tests, evidence, documentation, true fixture-only data, true formatting-only edits, and generated outputs do not independently count. A fixture that defines runtime registration/policy/contract behavior is classified by that function and counts. A mixed semantic/formatting edit counts.

`CONTAINED` and `NOT_CONTAINED` are valid Primary outcomes. Architecture resistance to a valid evolution is `NOT_CONTAINED`, not a reason for invalidation. `INVALID_EXPERIMENT` is limited to experiment-contract failure: wrong provenance/version, contamination, unequal semantics, silent alternative redefinition, post-freeze mutation, mapping/evaluator drift, or genuine non-applicability. `INCONCLUSIVE` covers unresolved missing or ambiguous evidence after non-mutating recovery is attempted. Unfavorable valid outcomes cannot be selectively excluded.

One invalid/inconclusive cell removes its entire scenario across A/B/C/D from Primary `S_primary`; all attempted cells remain in Secondary diagnostics. The report identifies the scenario, affected cells/reasons, symmetric exclusion, and resulting common denominator. With one v1 scenario per E1-E5 category, losing any scenario normally requires `QA-03 COMPARATIVE CAMPAIGN INCOMPLETE — TAXONOMY COVERAGE LOST`. A replacement requires a new prospective protocol/catalog and consistent rerun. The desired matrix remains 20 valid cells with denominator 5 for every alternative.

CCR alone is Primary. Roles/files/components/LOC/contracts/propagation depth are Secondary diagnostics. **QA-03 numeric grading thresholds remain unfrozen; this experiment reports CCR directly.** No category weights or overall architecture score are created.

## Isolation model

Every Alternative x Scenario cell starts in a unique disposable worktree/branch at the exact source SHA. QA03-2's dedicated worktree cannot be shared with Session 7.2, another Codex session, or an actively switched main working directory. Path, branch, HEAD, and clean status are recorded before every campaign stage, and another worktree's checkout cannot change QA03-2's source. No result branch is the parent of another. Acceptance tests are materialized before production work. Codex receives the identical frozen request and is instructed to make the minimum architecture-consistent change through the natural extension mechanism—not to optimize the diff for CCR.

The 20 cells preserve baseline/result SHAs and raw manifests. A worktree is discarded only after evidence is committed/archived. No reset, rebase, merge, or branch reuse is part of the protocol. Alternative order should rotate to reduce familiarity bias.

## Evaluator qualification

Synthetic qualification covers in-area success, valid E4 boundary pressure as `NOT_CONTAINED`, silent redefinition as invalid, explicit and unmapped out-of-area propagation, acceptance failure, new/worsened regression, unchanged baseline failures, non-scoring test changes, A/B/C/D role resolution, invalid isolation/baseline, incomplete/ambiguous evidence, schema/version alignment, equal denominators, scenario-wide exclusion, taxonomy-loss blocking, a normal 20-cell matrix, and deterministic reruns. These fixtures do not implement or consume the five measured evolution scenarios.

Validation at the freeze commit passed:

- QA-03 evaluator qualification: 20/20;
- full Python suite: 115/115;
- Rust workspace/all-target check: PASS;
- Rust format check: PASS;
- Rust clippy with warnings denied: PASS;
- full Rust workspace suite: 107/107;
- two simultaneous clean detached worktrees at the exact source SHA: PASS;
- diff check and protected DP-00/QA/profile contract invariance: PASS.

## Limitations

The experiment measures expected-change containment for a small E1-E5 sample against the present DP-00 executable prototypes. It does not measure developer speed, effort, LOC quality, general maintainability, runtime latency, production-provider fidelity, or all possible future changes. Binary CCR intentionally loses propagation severity; Secondary diagnostics retain it. E2 samples Model-provider evolution because Voice Runtime is not implemented within the four prototype crates.

## Prospective freeze statement

The scenario catalog, role map, Expected Change Areas, binary classifications, baseline comparison, CCR formula, isolation protocol, manifest, and evaluator are frozen before measured implementation.

With the validation results above, the disposition is:

`QA-03 EXPERIMENT CONTRACT FINALIZED — READY FOR FLEXIBILITY CAMPAIGN`

The overall decision state remains:

`NOT READY FOR ARCHITECTURE DECISION`
