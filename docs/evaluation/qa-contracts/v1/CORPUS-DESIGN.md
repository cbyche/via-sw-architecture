# QA-v1 Corpus Design

## Authority and claim boundary

The frozen corpus manifest, population manifests, materialized instances,
oracles, and fixtures form one DP-neutral measurement instrument. QA Contract
v1 remains the sole authority for metrics, targets, boundaries, penalties, and
score mappings. Corpus materialization neither measures an architecture nor
claims production-frequency representativeness.

Historical reuse follows the Historical Evidence Registry. E2 artifacts seed
scenario semantics only, E3 artifacts may calibrate reference timing, and E4
artifacts inform structural coverage. No historical observation is promoted to
QA-v1 evidence; the registry contains no E1 evidence.

## Generation model

The hierarchy is semantic family → frozen template → meaningful variation →
materialized stable instance. `qa-v1-materializer-1.0.0` uses no runtime
randomness. Materialized JSON, rather than runtime generation, is authoritative.
Each population manifest records the family, instance, oracle, and reference
fixture hashes.

The 600-entry master goal catalog contains 60 semantic families. Each family
has ten variants that change modality, fixture identity, context version,
capability profile, result-fact cardinality, or approval/task state. Wording-only
paraphrases are not a variation dimension. QA-01 references 150 of these goals:
five variants from each of the 30 fast families, yielding exactly 50 bounded,
50 short general-Agent, and 50 short specialist-Agent goals.

## Population designs

- QA-03 uses 20 lifecycle families crossed with 20 causally distinct event
  interleavings. Each oracle fixes Turn, Task, Execution, Result, Response,
  Delivery, Approval, Cancel, and follow-up relations.
- QA-04 freezes ten Agent-evolution families with six contract-profile
  variations. Ownership areas, seams, forbidden Core zones, and regressions are
  declared before evaluation.
- QA-05 freezes 60 realistic changes across Z1–Z8 (8/8/8/8/7/7/7/7).
- QA-06 freezes 20 actual requirement cells for each of Mobile, TV, and Robot.
- QA-07 is one non-counted fixed concurrency/residency timeline, P0–P5.
- QA-08 crosses ten recoverable fault families with 20 lifecycle timings.
- QA-09 crosses 15 sensitive-scope families with 20 ground-truth variants and
  includes all three non-offsettable security-gate triggers.
- QA-10 has ten success and ten fault/edge families with ten instances each;
  reconstruction uses expected causal graph comparison.
- QA-11 has exactly 40 obligations in each of its five event strata.
- QA-12 crosses ten contention/playback families with 20 deterministic audible
  timelines.

## Shared fixtures and observations

Reference contexts and device profiles, semantic replay outcomes, anonymous
Agent capability profiles, and the evaluator self-test are frozen under
`benchmark/fixtures/qa-v1/` and `benchmark/agent-stubs/qa-v1/`. The canonical
event schema is DP-neutral and includes input, evidence, decision, task,
delivery, sensitive-scope, resource, recovery, and causal-link events.

Future candidates must emit canonical observations with exact frozen case
identities and the common provenance envelope. An incomplete population returns
`NOT_EVALUATED`, never zero.
