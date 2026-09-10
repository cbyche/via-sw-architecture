# DP-00 Runtime Pilot v0 Corpus

## Status

**Pilot calibration input — not a final evaluation corpus or score.**

This directory freezes the ten Session-1 input scenarios used to prepare the
DP-00 Runtime Pilot. It does not freeze repetition count, run order, dependency
latency, percentile estimator, QA-04 aggregation, scoring thresholds, or a
winning architecture.

The corpus entry point is `index.json`.

The array order in the index is discovery order only. It does not define Pilot
execution order, counterbalancing, warm-up, or repetitions.

## Asset separation

```text
benchmark/scenarios/pilot-v0/       benchmark-owned scenario and stimulus refs
benchmark/behavior-plans/pilot-v0/ responsibility-keyed semantic replay plans
benchmark/fixtures/pilot-v0/        controlled AUT-visible environment fixtures
benchmark/oracles/pilot-v0/         evaluator-only truth, constraints and probes
```

Scenario files contain evaluator asset identifiers so the runner can assemble
an episode, but do not contain the referenced ground truth. A future runner must
materialize only `stimulus` and dependency responses into AUT-facing types. It
must load `benchmark/oracles/pilot-v0/oracles.json` exclusively through the
evaluator path.

Context fixtures contain raw surface objects, positions, pointer events, focus
facts and stable object IDs. Correct referents and task associations occur only
in the evaluator-only oracle registry. Initial-state fixtures contain logical
facts; they do not select the current turn's task.

## QA eligibility

| Pilot scenario | Catalog class | QA-01 | QA-02 | QA-04 | Route commit expectation |
| --- | --- | :---: | :---: | :---: | :---: |
| P01 | R1 | yes | yes | yes | REQUIRED |
| P02 | R2 | yes | yes | yes | REQUIRED |
| P03 | R3 | yes | yes | yes | REQUIRED |
| P04 | R4 | no | yes | yes | REQUIRED |
| P05 | R5 | no | yes | yes | REQUIRED |
| P06 | R6 | no | yes | yes | REQUIRED |
| P07 | R7 | no | yes | yes | REQUIRED |
| P08 | R10 | no | yes | yes | FORBIDDEN |
| P09 | R10 | no | yes | yes | OPTIONAL |
| P10 | R10 | no | yes | yes | FORBIDDEN |

QA-01 eligibility is fixed by workload semantics, not observed duration. P01,
P02 and P03 reference controlled Voice fixtures with benchmark-authoritative
ground-truth acoustic EOS offsets. Runtime VAD is explicitly non-authoritative.

QA-04 eligibility does not decide how terminal no-route-commit episodes enter a
future aggregate. `REQUIRED` denotes normal route-oriented scenarios,
`FORBIDDEN` denotes P08/P10 terminal faults before commit, and `OPTIONAL` keeps
P09 neutral: the architecture may reject the wrong proposal, commit it and fail
QA-02 conformance, or recover through an existing Base mechanism.

## Schema and validation policy

Pilot-v0 uses explicit JSON identities and versions. The Rust loader in
`bench-fixtures::pilot_assets` applies these rules:

1. Every typed required field must be present; there are no Serde defaults.
2. Unknown fields are rejected at every typed contract boundary using
   `deny_unknown_fields`.
3. Schema and vocabulary enums are explicit and case-sensitive.
4. Every asset carries `schema_version`, `asset_id`, and `asset_version` (or the
   equivalent corpus identity fields).
5. References and versions are resolved across the scenario, behavior-plan,
   fixture, and oracle registries.
6. Fixture `data`, constraint `expected`, and oracle ground-truth values are
   intentionally versioned structured payloads. Their typed domain adapters
   belong to the Session-2 runner; the generic Session-1 loader does not invent
   implicit defaults for them.

Golden parity fixtures live under `benchmark/fixtures/pilot-v0/golden/`:

- `valid-runtime-scenario.json`
- `valid-behavior-plan.json`
- `invalid-missing-required-field.json`
- `invalid-unknown-field.json`
- `invalid-behavior-enum.json`

The same files are intended for the later Python loader tests.

## Fault conditions

- P07 supplies a wrong `LOCAL_DIRECT/VIA_FAST` route proposal for a request
  whose intent payload explicitly requires domain planning.
- P08 supplies `MALFORMED` attempt 1 and a reproducible `CORRECT` attempt 2.
  Attempt 2 remains unused when the Base architecture does not retry.
- P09 proposes `MailAgent` for Wi-Fi diagnosis. MailAgent exists, is registered,
  is healthy, and the route/request objects are structurally valid, but the
  capability profile and evaluator oracle make the semantic mismatch explicit.
- P10 supplies route `TIMEOUT` and Agent-selection `NO_RESPONSE` terminal
  behavior under a future Session-2 latency profile.

Semantic replay keys use stable turn/responsibility operations. Global model
call number is not part of any replay key.

## Coverage gaps

The machine-readable corpus index records:

```text
R8 Concurrent Task / Result = NOT_COVERED
R9 Compound Request         = NOT_COVERED
```

Both must be added before final benchmark-v1 freeze. Existing catalog entries
remain unchanged.
