# Runtime Benchmark Run Provenance Contract

## Status

**Canonical Benchmark Contract Checkpoint — raw runtime provenance extension**

This contract complements `benchmark/schemas/run-event-schema.md` by defining the frozen scenario/replay/fixture/profile references that every DP-00 Runtime Episode run must preserve.

It does not replace the event stream. Raw events remain the source of truth for timing, decisions and outcomes.

---

# 1. Minimum provenance

Each runtime run must identify or immutably reference:

```text
run_id
episode_id
alternative_id
alternative_version
source_git_commit

runtime_scenario_schema_version
scenario_id
scenario_version
scenario_class
architecture_sensitivity_tags[]
runtime_catalog_version

semantic_behavior_plan_id
semantic_behavior_plan_version
semantic_responsibility_vocabulary_version
owner_responsibility_mapping_version
replay_payload_registry_version

context_fixture_id/version
initial_state_fixture_id/version
state_seeder_mapping_version
capability_profile_id/version
health_profile_id/version
policy_profile_id/version

interaction_frontend_profile_id/version
agent_fixture_profile_id/version
tool_fixture_profile_id/version
outcome_probe_profile_id/version
dependency_latency_profile_id/version

model_profile_id/version
prompt_profile_id/version
cache_policy_id/version

constraint_manifest_id/version
success_predicate_id/version
canonical_event_schema_version
model_call_schema_version
run_event_schema_version

benchmark_version
scoring_version
qualification_gate_version
aggregation_version
```

Fields not yet frozen during Pilot may use an explicit `PILOT_TBD`/null convention defined by the implementation. Final qualification must not silently omit version identity for a scoring-relevant contract.

---

# 2. QA eligibility / boundary observations

Preserve at run/episode level:

```text
qa01_eligible
qa02_eligible
qa04_eligible

route_commit_expected
route_commit_observed
execution_route_commit_ts

success
exact_conformance
```

For QA-01 also preserve whether a Voice fixture and ground-truth Acoustic EOS were available.

For QA-04, no-route-commit handling is not decided by this schema. The raw facts remain available for the future frozen aggregation rule.

---

# 3. Semantic operation consumption summary

Per-operation details may live in replay/model-call records, but run provenance should support a summary or reference:

```text
semantic_operations_requested[]
semantic_operation_attempts_consumed[]
semantic_operations_unused[]
replay_validation_failure_ids[]
```

This allows reviewers to distinguish:

- operations planned by the frozen scenario;
- operations actually requested by the architecture;
- attempts consumed due to retry;
- deterministic responsibilities that required no model replay.

---

# 4. Oracle separation audit

Recommended raw integrity fields:

```text
stimulus_materialization_version
oracle_materialization_version
oracle_access_violation_detected
scenario_id_decision_access_violation_detected
```

The implementation may enforce isolation structurally rather than through flags, but final qualification should retain evidence that Stimulus and evaluator-only Oracle were materialized through separate access paths.

---

# 5. Outcome authority audit

Recommended:

```text
useful_outcome_authority
useful_outcome_source_event_id
useful_outcome_probe_version
```

Candidate values:

```text
OUTCOME_PROBE
INTERACTION_OUTPUT_PROBE
FIXTURE_STATE_PROBE
AUT_ONLY_DIAGNOSTIC
```

For QA-01, `AUT_ONLY_DIAGNOSTIC` is insufficient when the outcome is externally observable through a configured probe.

---

# 6. Canonical route projection

Preserve the architecture-neutral route projection:

```text
route_kind
initial_executor_id
final_executor_id_if_known
delegation_chain[]
```

Do not make A/B/C/D-specific route labels the sole raw representation.

Alternative-specific debug fields may be retained separately.

---

# 7. Raw-data immutability

`results/raw/` is append-only/immutable experimental evidence.

Changing scenario/replay/fixture/profile meanings creates a new version and new run. It does not rewrite provenance on historical results.

Derived micro/macro averages, QA scores and sensitivity analyses belong under `results/derived/` or `results/reports/`.

---

# 8. Relationship to existing schemas

- `run-event-schema.md` — ordered runtime events and QA timestamp/decision projections.
- `model-call-schema.md` — per-logical-generation evidence.
- `runtime-scenario-schema.md` — frozen scenario definition.
- `semantic-behavior-plan-schema.md` — responsibility-based semantic behavior.
- `canonical-event-schema.md` — architecture-neutral observation vocabulary.
- this file — immutable run-level provenance tying those contracts together.

The Prototype/Harness Specification may serialize these fields into one run header plus references rather than duplicate them on every event.