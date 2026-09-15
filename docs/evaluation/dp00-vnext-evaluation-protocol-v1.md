# DP-00 vNext Evaluation Protocol v1 — Skeleton

## Status

**READY FOR THE NEXT TASK — NO CAMPAIGN RUN — NO CANDIDATE RESULTS**

This protocol binds R1, R3, and the R1+@ tactic realization to QA Evaluation Contract v1 without changing its metrics, targets, score bands, populations, corpus counts, Reference Environment, or semantics. It requires no Cloud/model API and no API key.

## Candidate and comparison boundary

- R1 contract: `benchmark/contracts/dp-vnext/dp00-r1-candidate-v1.json`
- R3 contract: `benchmark/contracts/dp-vnext/dp00-r3-candidate-v1.json`
- R1+@ tactic: `benchmark/contracts/dp-vnext/dp00-r1-readonly-fastpath-tactic-v1.json`

Every integrated candidate must receive:

- the same frozen QA-v1 corpus and populations;
- the same Reference Environment;
- the same semantic capability envelope;
- the same downstream Agent capability profiles;
- the same Voice and Text obligations;
- the same fault and event schedules; and
- the same device and resource workload.

Candidate-specific structural work is allowed only when caused by the architecture. Do not inject candidate-specific error probabilities, hidden oracle information, favorable corpus selection, or different dependency behavior to manufacture a trade-off.

## Pre-run controls

1. Record candidate contract identity and architecture commit.
2. Record hashes for the QA contract, Reference Environment, corpus manifest, candidate contract, workload profiles, and fault/event schedules.
3. Prove all functional obligations and architecture invariants are represented by conformance checks.
4. Verify R1+@ rejects every forbidden behavior before it can enter the measured campaign.
5. Use frozen semantic replay/local deterministic fixtures; do not invoke Cloud or model APIs.
6. Counterbalance ordering and preserve raw observations using the shared QA-v1 provenance/result envelope.

## Decision procedure frozen before results

### Step 1 — Hard qualification

Each candidate must satisfy functional obligations, the QA-09 security hard gate, and its architecture invariants. A failure is not offset by another score.

### Step 2 — Primary product quality

Evaluate QA-01 and QA-02 first. Do not collapse them into an arbitrary weighted sum. Compare target satisfaction, contract score, raw metric, and confidence/sensitivity where relevant.

### Step 3 — Trade-off

For candidates still viable, examine QA-03, QA-04, QA-05, QA-06, QA-07, QA-08, QA-10, QA-11, and QA-12 according to causal relevance. QA-09 remains a hard qualification result, not a compensable trade-off.

### Step 4 — Tactic remediation

If a weakness can be mitigated without changing architecture identity, apply the tactic and re-evaluate the full integrated candidate across all twelve QAs. Never add per-DP or per-tactic metric improvements arithmetically.

### Step 5 — Final selection

Select from actual measured evidence. Neither R1 nor R3 has a default preference. R1+@ may be selected only as the evaluated realization of R1 when the bounded tactic materially improves the final integrated architecture.

## Required next-task outputs

The campaign must produce provenance-bound raw observations, complete QA-v1 result envelopes for all twelve QAs per integrated candidate, hard-gate and invariant results, sensitivity/confidence material required by each QA, and a decision record that follows the five steps above. This skeleton contains no scores and authorizes no campaign in the current task.
