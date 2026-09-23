# AA-011 — DP-00 Session 2.5 Raw Evidence and Campaign Readiness Review

## Status

Implementation review for DP-00 Runtime Pilot v0 Session 2.5. This document does not calculate AECR/FTOL, set thresholds, or run an Official Pilot.

## Raw contract change

Before this session, raw evidence retained route, task, clarification, result, useful-outcome, and failure markers, but not every semantic value needed to reconstruct the actual QA-02 trace. A later evaluator would have had to consult behavior plans or oracles for referents, task relation, effect details, or terminal reason.

After this session, `canonical-event-v1`, `model-call-v1`, and run provenance v2 preserve actual observations independently:

| Constraint dimension | Actual raw source | Expected source used later |
| --- | --- | --- |
| Referent binding | AUT `ReferentBound` role/id plus turn correlation | evaluator-only oracle referent constraint |
| Task association | AUT `TaskAssociated` relation plus task correlation | evaluator-only task constraint |
| Route/executor | AUT `RouteCommitted(ExecutionRoute)` | evaluator-only route/agent constraints |
| Clarification | AUT request reason and request/response turns | evaluator-only clarification constraint |
| Result binding | AUT `ResultBound` result/task/execution correlation | evaluator-only result-binding constraint |
| Observable effect | Outcome Probe typed effect and fixture evidence | evaluator-only success/effect predicate |
| Failure outcome | runtime lifecycle typed failure reason | evaluator-only allowed/required failure outcome |

For P09 no-commit behavior, `model-call-v1.semantic_output_reference` preserves the executor candidate actually returned by the Model Fixture (`MailAgent`) while the canonical trace separately preserves that no route was committed and the runtime terminated with `INVALID_ROUTE`. A proposal is never upgraded to a committed route.

## Authority and isolation review

- Referent, task, route, clarification, and result binding are emitted by `ARCHITECTURE_UNDER_TEST` at their product decision boundaries.
- External effects are emitted by `OUTCOME_PROBE`; the AUT cannot claim authoritative success.
- Failure outcome is emitted by the benchmark runtime lifecycle from the actual terminal error.
- The AUT-visible port still has no run/scenario/alternative id, canonical sequence/timestamp authority, emitter selection, QA expected value, or oracle truth.
- The projection helper consumes only canonical raw events. It accepts no scenario, behavior-plan, or oracle argument.
- The synthetic wrong-referent test preserves `file_B` verbatim; it cannot substitute an oracle value.

## Timed-path review

The timed path adds small typed enum/id/value observations and in-memory vector appends. Canonical envelope enrichment and JSON/JSONL serialization remain after timed execution. No Python, file I/O, or JSON object construction was added to the timed path. Model response bodies are not persisted; only a bounded typed candidate reference is retained where available.

## Official campaign design

Official mode now has a campaign coordinator above the existing single-profile episode runner:

```text
one clean-tree/source attestation
  -> campaign provenance
  -> profile Z runs
  -> profile C runs
  -> campaign complete
```

The output layout is:

```text
results/raw/pilot-v0/<campaign-id>/
├─ campaign-provenance.json
├─ profile-z/<run-id>/...
└─ profile-c/<run-id>/...
```

Campaign provenance records source commit, initial cleanliness, corpus id/version, ordered full latency profiles, runner version, and common run configuration. Per-run provenance retains profile identity and adds campaign correlation. Cleanliness is checked once before the campaign writes output, so Profile Z output does not block Profile C. Existing campaign/profile/run directories are rejected. Development mode retains a single-profile path.

## Raw-analysis readiness answers

1. P04 actual referent from raw only: **YES** (`SOURCE`, `doc-right`).
2. P05 actual task association from raw only: **YES** (`FOLLOW_UP`, `T1`).
3. Execution route/executor from raw only: **YES** (full canonical `ExecutionRoute`; absence remains absence).
4. Clarification action from raw only: **YES** (requested/resolved and U1/U2 correlation).
5. Result binding pair from raw only: **YES** (result/task/execution product correlation).
6. Observable effect actual from raw only: **YES** (typed Outcome Probe subject/value/state/executor).
7. Terminal failure reason from raw only: **YES** (typed lifecycle reason).
8. P09 semantic wrong route without oracle: **YES** (`MailAgent` actual proposal + no commit + `INVALID_ROUTE`).
9. Must Python infer actuals from behavior plans: **NO**.

## Architecture conformance regression

- A remains Thin VIA.
- B remains ARGO-centric.
- C remains A plus bounded Fast Path.
- D remains Adaptive Per-turn.
- Route selection, Fast eligibility, selector/controller decisions, retry policy, and dispatch decisions are unchanged.
- Alternative source changes are instrumentation and product-correlation capture only.
- Hidden replay turn correlation and scripted clarification release are benchmark-fixture repairs; they do not make semantic decisions for an alternative.
- No oracle dependency or scenario-id decision access was added to AUT crates.

## Session 2.5 abbreviated instruction

1. Verify clean `exp/dp00-runtime-pilot` at the specified HEAD and zero upstream divergence.
2. Add topology-neutral actual referent/task/route/clarification/result/effect/failure evidence without oracle leakage.
3. Prove raw-only projection, wrong-actual preservation, P04/P05/P09 persistence/reload, and P08/P10 reasons in Rust.
4. Add strict semantic-event golden fixtures and explicit schema versions.
5. Add one official campaign invocation for ordered Z+C profiles with one initial clean-tree attestation, shared source/corpus provenance, immutable output directories, and retained development mode.
6. Run all specified Rust gates; do not run Python analysis or an Official Pilot.
7. Review A/B/C/D decision-flow invariants, commit logical milestones, push, and finish clean/synchronized.

## Codex result review criteria

- A reviewer can identify P04 `doc-right`, P05 `FOLLOW_UP/T1`, and P09 `MailAgent` proposal/no-commit directly from reloaded raw files.
- No raw actual is copied from an oracle or inferred from scenario id, filename, or order.
- Outcome authority is external and failure reason is typed.
- `ResultBound` has result/task/execution identifiers.
- Invalid/missing/unknown semantic payload fixtures fail strict Rust parsing.
- Official default profile order is exactly Z then C; both runs share commit/corpus and retain distinct profile identity.
- Dirty source and existing output directories are rejected, while Z output does not trigger a second cleanliness guard.
- No Official Pilot, Python scoring, thresholds, recovery tactic, or architecture winner appears in the change.
- Full fmt/clippy/workspace/smoke/fault/replay/contract/qualification gates pass.
- The final branch is clean and synchronized with upstream.

## Conflict review

No conflict with the approved DP-00 executable architecture specification, experimental boundary, QA-02, QA-04, or benchmark contracts was found.
