# QA-v1 Common Evaluation Engine

This package is the only shared scoring/evaluator implementation for future Decision Point work. It reads the frozen QA contract, reference environment, and corpus manifest directly.

It provides:

- artifact-hashed result provenance with explicit `evidence_mode`;
- contract-driven target and 0–5 scoring;
- non-offsettable hard-gate handling;
- one evaluator interface and one scalar output per QA;
- hash-verifying frozen-population loading with exact case identities;
- an explicit `INCOMPLETE_POPULATION` result with no metric or score when the corpus is not complete.

The engine performs no Cloud/API calls and accepts no DP-specific target or
score table. Complete observation sets must exactly cover the frozen case
identities; diagnostics explain the scalar but do not modify it.

Future runners should submit
`benchmark/schemas/qa-v1-canonical-observation.schema.json` envelopes through
`evaluate_canonical_observations`. The adapter binds QA, population, case,
evidence mode, and measurement before invoking the same evaluator and global
score engine.

The percentile evaluators use deterministic nearest-rank p95 over the complete frozen population. This is an implementation convention for the contract's `p95` formula, not a new QA target or score definition.
