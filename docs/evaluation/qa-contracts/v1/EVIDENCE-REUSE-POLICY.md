# QA-v1 Historical Evidence Reuse Policy

## Authority

QA Evaluation Contract v1 remains frozen. Historical evidence never changes its original identity merely because a current evaluator can read it. The machine-readable classification is `benchmark/contracts/qa-v1/historical-evidence-registry-v1.json`.

## Evidence classes

| Class | Meaning | Permitted use | Prohibited use |
| --- | --- | --- | --- |
| E0 | HISTORICAL_ONLY | Rationale, learning, regression history, reviewer provenance | QA-v1 score, denominator, or winner claim |
| E1 | RAW_RECOMPUTABLE | Re-evaluation from compatible raw evidence after all checks pass | Reusing the old derived score |
| E2 | SCENARIO_REUSABLE | Seed/adapt a semantically eligible QA-v1 corpus case | Treating its historical execution as a new observation |
| E3 | CALIBRATION_REFERENCE | Calibrate synthetic/reference models and sensitivity assumptions | Treating a reference or simulation as executed QA-v1 evidence |
| E4 | STRUCTURAL_EVIDENCE | Inform ownership, routes, state, interfaces, and candidate implementation | Convert structure directly into a QA-v1 score |
| E5 | INVALID_OR_UNVERIFIABLE | Preserve as negative provenance or validation history | Any current quality or winner claim |

Classification applies to an artifact's strongest safe reuse role. A report may cite weaker roles, but it may not promote an artifact above its registry class.

## Mandatory compatibility test

An artifact can be E1 only when all of these independently pass:

1. Current measurement start and end facts exist.
2. Current population eligibility is proven without consulting the result.
3. Architecture identity and source commit are unambiguous.
4. Required outcome, failure, and missing-data semantics are observable.
5. Reference Environment treatment is compatible or explicitly transformable.
6. Complete provenance exists.

One failed check prevents E1. Derived artifacts inherit the weakest provenance of every contributing observation.

## Non-negotiable reuse rules

- Never translate a legacy score. AECR is not current QA-02 completion; legacy CCR is not current QA-03, QA-04, or QA-05.
- Never promote old FTOL/p95 when either required audible Voice facts or same-version Text details are absent.
- Historical C is not current `R1 + @`. Current `@` is bounded, deterministic, and read-only; P01 volume change is state-changing.
- Historical A and B can inform R1 and R3 realization, respectively, but numerical equivalence is not assumed.
- Select corpus seeds by predeclared semantic fit, never candidate outcome.
- Adapted scenarios must be re-run when any output obligation, deadline, read-only rule, oracle, or Voice/Text behavior changes.
- Simulated, replayed, executed, and structural numbers must carry distinct `evidence_mode` values.

## Promotion procedure

To propose an E1 promotion, add a new registry revision with evidence for every compatibility check, validate the original raw hashes, and run the QA-v1 evaluator against the frozen population contract. Do not edit the old report or score.

## Cloud policy

This foundation uses no Cloud API. Future optional Cloud generation remains subject to Reference Environment v1 provenance and latency-exclusion rules.
