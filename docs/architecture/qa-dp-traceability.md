# QA-v1 ↔ Active vNext Decision Point Traceability

## Authority

This is the active DP traceability view. QA semantics come only from [`../evaluation/qa-contracts/v1/README.md`](../evaluation/qa-contracts/v1/README.md) and `benchmark/contracts/qa-v1/qa-evaluation-contract-v1.json`. A Primary designation does not change a metric, target, population, score band, or gate.

Every active DP evaluates **QA-01 through QA-12**. The table identifies only the strongest expected causal discriminators.

| DP | Primary QAs | Causal link | Dependencies |
| --- | --- | --- | --- |
| DP-00 | QA-01, QA-02, QA-04, QA-05 | primary semantic/execution ownership changes latency, completion, Agent evolution, and product-change containment | QA-v1 contract/environment/corpus |
| DP-01 | QA-01, QA-02, QA-03, QA-05 | turn commit/revision authority affects latency, correct binding, continuity, and adapter change | DP-00 |
| DP-02 | QA-01, QA-02, QA-04, QA-05 | semantic authority topology affects critical path, grounded completion, Agent integration, and change surface | DP-00/R1 |
| DP-03 | QA-01, QA-02, QA-05, QA-07 | historical evidence ownership affects retrieval latency/correctness, source evolution, and memory | DP-00 |
| DP-04 | QA-01, QA-02, QA-04, QA-05 | context admission/lifecycle dependency affects latency, completion, Agent contracts, and change | DP-00, DP-03 |
| DP-05 | QA-01, QA-02, QA-03, QA-04 | session state sharing affects reuse latency, isolation correctness, continuity, and Agent adaptation | DP-00, DP-04 |
| DP-06 | QA-03, QA-05, QA-08, QA-11 | supervisor runtime/failure boundary affects continuity, deployability, recovery, and feedback | DP-00, DP-05 |
| DP-07 | QA-01, QA-02, QA-03, QA-05 | response authority affects timely consistent meaning, cross-channel continuity, and evolution | DP-00, DP-01, TaskResult |
| DP-08 | QA-01, QA-07, QA-11, QA-12 | enforceable inference isolation affects realtime latency, resource amplification, feedback, and barge-in | DP-00, DP-06 |
| DP-09 | QA-06, QA-05, QA-02, QA-03 | device-semantic ownership directly changes adaptation containment, common evolution, correctness, and continuity | DP-00, DP-01, DP-03, DP-04 |

## DP-00 selection ordering

1. Qualify functional obligations, architecture invariants, and QA-09 hard gate.
2. Compare QA-01 and QA-02 target satisfaction, score, raw metric, and relevant confidence/sensitivity separately.
3. Examine QA-03/04/05/06/07/08/10/11/12 according to causal relevance.
4. Re-evaluate any tactic remediation as a full integrated candidate over all twelve QAs.
5. Select from measured evidence with no default family preference.

Historical pre-contract traceability is preserved in Git history and the legacy documents/results linked by [`decision-points/vnext/MIGRATION.md`](decision-points/vnext/MIGRATION.md). It is not current QA-v1 scoring authority.
