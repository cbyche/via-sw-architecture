# Active vNext Decision Point Catalog

## Status and policy

This directory is the sole authoritative human-readable catalog for active vNext Decision Points. It does not alter the approved requirements baseline or QA Evaluation Contract v1.

> No base architecture family is preselected. DP-00 selection will be driven first by measured QA-01 and QA-02 results under QA Evaluation Contract v1, with the remaining QAs used for trade-off, regression, hard-gate, and mitigation analysis.

R1 and R3 are both legitimate final outcomes. R1+@ is not a third base family; it is an R1 realization with a bounded deterministic read-only local tactic.

## Admission rule

An active DP must change at least one component responsibility, authority/source of truth, persistent-state owner, interface/dependency direction, runtime/deployment/failure boundary, or lifecycle/publication authority. A model/provider choice, prompt layout, threshold, retry/timeout number, queue priority, cache size, publication granularity, speculation toggle, or classifier-versus-LLM choice is a tactic or policy unless it changes one of those structural boundaries.

Every active DP evaluates QA-01 through QA-12 under the frozen contract, Reference Environment, and populations. Primary QA lists do not redefine the contract.

## Catalog and dependencies

| DP | Depends on / constrained by |
| --- | --- |
| DP-00 | QA Evaluation Contract v1; approved product obligations |
| DP-01 | DP-00 interaction/semantic owner constraints |
| DP-02 | DP-00; primarily applicable within R1 |
| DP-03 | DP-00 context/grounding boundary |
| DP-04 | DP-00 and DP-03 |
| DP-05 | DP-00 and DP-04 |
| DP-06 | DP-00 and DP-05 |
| DP-07 | DP-00, DP-01, and TaskResult contract |
| DP-08 | DP-00 and DP-06 workload/admission obligations |
| DP-09 | DP-00, DP-01, DP-03, and DP-04 |

Machine-readable source: `benchmark/contracts/dp-vnext/dp-catalog-v1.json`. Legacy disposition: [`MIGRATION.md`](MIGRATION.md).
