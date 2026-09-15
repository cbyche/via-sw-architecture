# DP-00 causal offline evaluation v2 protocol

Campaign v1 validated QA-v1 population handling, provenance, scoring, candidate pre-registration, and evaluation plumbing. Its executable candidate runtimes did not causally materialize enough R1/R3 architecture behavior to support R1-vs-R3 architecture equivalence or selection claims.

The v1 numbers remain immutable historical preliminary evidence. They must not be used to select DP-00.

V2 freezes three explicit boundary types: `CandidateVisibleInput`, `FrozenPrimitiveReplay`, and evaluator-only `EvaluatorOracle`. Candidate execution accepts only the first two concepts. The oracle is constructed after execution and enters only the independent observation adapter.

The official causal chain is frozen scenario → candidate architecture execution → typed events/state/dependencies → independent observation → unchanged QA-v1 evaluator. Common deterministic costs are keyed by operation and scenario semantic signature, never candidate identity. Candidate differences therefore arise only from operations required by their architecture graphs.

Mutation-validity tests are non-campaign evidence. Official candidates use no mutation flags. QA-07 remains UNEVALUABLE pending evidence-anchored frozen memory calibration. No final ADR is authorized by this protocol.
