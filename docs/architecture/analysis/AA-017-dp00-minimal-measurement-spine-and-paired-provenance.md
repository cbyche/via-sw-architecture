# AA-017 — DP-00 MINIMAL Measurement Spine and Paired Provenance

## Purpose

Session 6.3 stopped before execution because MINIMAL discarded the raw evidence
needed for authoritative FTOL, correctness qualification, and the P12 QA-04
boundary. Session 6.3R repairs that measurement contract without running a
calibration or changing an architecture alternative.

## Measurement Spine

The common spine contains Acoustic EOS, architecture semantic/lifecycle events,
route commits and correlation, execution starts, useful outcomes, terminal
events, and logical model calls. CAPTURE stores that spine plus model-generation
diagnostic canonical events and detailed fixture events. MINIMAL stores the
spine only.

QA-01 remains exactly:

```text
UsefulOutcomeObserved.monotonic_timestamp
- AcousticEos.monotonic_timestamp
```

No wall-clock episode proxy and no paired CAPTURE evidence may substitute for a
MINIMAL boundary.

## Paired provenance

`dp00-pilot-provenance-v3` adds explicit calibration, cycle, pair, alternative
order slot, mode-order slot, repetition, and source identities. Separate runner
invocations are permitted, but each persisted run carries its own full identity.
Pairing is never inferred from layout or run order. Each calibration invocation
declares its rotation index and contains one measured cycle, so paired mode
invocations reproduce the same alternative positions explicitly.

## Qualification

A pair requires exactly one CAPTURE and one MINIMAL member, shared semantic and
execution-profile provenance, distinct mode-order slots 0/1, matching QA-02
qualification, effects, routes and route ordering, parent/child/subgoal
correlation, logical model-call semantics, and QA-04 boundary semantics.
Missing, duplicate, provenance-mismatched, or semantic-mismatched pairs are not
calibration samples.

P12 remains governed by the Session 6.2R initial route-plan barrier. Both S1 and
S2 must commit exactly once; the evaluator derives the boundary from the final
required commit. Partial or duplicate coverage has no qualified boundary.

## Versioning

- Canonical event: `canonical-event-v3` (unchanged payload; new retention policy)
- Run provenance: `dp00-pilot-provenance-v3`
- Analysis: `dp00-analysis-v5`
- QA-04 policy: `qa04-route-contract-policy-v1` (unchanged)

This decision is provisional calibration methodology, not architecture-ranking
evidence and not a Final Evaluation decision.
