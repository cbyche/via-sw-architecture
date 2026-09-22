# AA-028 — Gate 2 Sensitivity and Decision Closure

> Historical note: this analysis predates the 16-configuration campaign. Current combined evidence and decisions are documented in `docs/rebaseline/12-07-full-factorial-results.md` and ADR-001 through ADR-004.
>
> W12-G2 note: the W-01/W-02 names, endpoints, and availability statements below are historical. The current pre-implementation Voice definitions are in `docs/rebaseline/11e-voice-responsiveness-measurement-redefinition.md`; they have not yet been measured.

- Status: Complete
- Date: 2026-09-22
- Scope: IR-DP01, TASK-DP01, AGENT-DP01, EXEC-DP01

## Question

Does the approved Gate 2 evidence differentiate the four candidate pairs strongly enough to make architecture decisions without changing the pre-result scoring contract?

## Evidence basis

The campaign is bound to source commit `b6ff0b073185a5a08ac1dfe0c5c94ad9510b0ba1` and freeze fingerprint `9b8cf4bf8633e10b10c05b91f05bb5c31fc67bf29e4ae3bb07ae3d0b4ff7084d`.

It contains:

- 192 raw W-07/W-08 change rows and their post-freeze aggregate scores;
- 1,200 W-09 recovery trials across whole-process and integration-fatal strata;
- 560 W-10 trials across two execution candidates and 28 cells;
- an explicit coverage record for every W-01 through W-12 metric.

Missing evidence was not converted to zero, inferred from smoke tests, or replaced by synthetic S2S latency. No weighted total was calculated.

## Analysis

AGENT-DP01 is the only pair with observable sensitivity under the frozen Differentiation Gate. W-07 changes from score 4 for edge normalization to score 3 for Core-visible typed contracts. The three raw differences occur exactly where a new protocol, pending-input lifecycle, or artifact lifecycle propagates into the Core handler/interface. W-08 remains tied, so the result is not a general claim that candidate A always changes fewer elements.

TASK-DP01 is flat in the available evidence. W-09 differs by 0.333ms between the shared service and per-Task owner and remains score 5 for both. W-08 is also tied. W-02 and W-04 are not available.

EXEC-DP01 has a meaningful raw recovery observation but no frozen-band differentiation. Process isolation reduces the six-strata W-09 composite from 550.833ms to 374.667ms because the integration-fatal strata recover at 18/33ms rather than 545/563ms. Both candidates still score 5 and pass the same target; W-10 is 28/28 for both. This is strong secondary evidence for the isolation mechanism, not permission to rewrite G3 after seeing the result.

IR-DP01 has only a tied W-08 result. The hosted reference run needed for W-05 was not possible without the environment credential; W-01/W-02 representative paths are also absent.

## Outcome

- Accept AGENT-DP01 A, Edge-normalized Canonical Contract.
- Defer IR-DP01, TASK-DP01, and EXEC-DP01. Retain their A references only as interim baselines, not measured winners.
- Preserve EXEC B's recovery benefit as secondary evidence and the first candidate to reconsider when delivery evidence or a newly frozen discrimination contract is available.

This is a completed evaluation outcome even though three choices are deferred: the method produced a falsifiable distinction between “selected” and “not yet supported” rather than forcing four winners.

## Traceability

- Method: `docs/rebaseline/12-00-evaluation-method.md`
- Results: `docs/rebaseline/12-05-gate2-measurement-results.md`
- Machine ledger: `results/rebaseline/gate2-freeze-b6ff0b07/sensitivity-sweep.json`
- Accepted decision: `docs/adr/ADR-001-agent-integration-contract-boundary.md`
- Deferred decisions: ADR-002 through ADR-004
