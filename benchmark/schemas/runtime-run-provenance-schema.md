# Runtime Benchmark Run Provenance Contract

## Status

**Prototype/Harness Specification Checkpoint — raw runtime provenance contract**

This contract complements `benchmark/schemas/run-event-schema.md` by defining the frozen scenario/replay/fixture/profile/runtime references that every DP-00 Runtime Episode run must preserve.

It does not replace the event stream. Raw events remain the source of truth for timing, decisions and outcomes.

Architecture Qualification runtime:

```text
Rust
```

Offline analysis:

```text
Python
```

The implementation-language rationale is recorded in `docs/architecture/analysis/AA-008-qualification-runtime-language-and-harness-rationale.md`.

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
observation_port_contract_version
model_replay_contract_version
prototype_harness_spec_version

benchmark_version
scoring_version
qualification_gate_version
aggregation_version
```

Fields not yet frozen during Pilot may use an explicit `PILOT_TBD`/null convention defined by implementation. Final qualification must not silently omit version identity for a scoring-relevant contract.

---

# 2. Rust qualification runtime provenance

Comparable A/B/C/D runs must record the common runtime substrate.

Minimum/recommended fields:

```text
qualification_runtime_language = rust
rust_toolchain_version
compiler_version
rustc_verbose_identity
rust_edition
target_triple
build_profile
build_flags_profile_version
optimized_release_class_build

async_runtime
async_runtime_version
async_runtime_worker_policy_version

cargo_lock_hash
dependency_lock_hash
workspace_manifest_hash
```

The same runtime/toolchain/build profile must be used for comparable Base Architecture runs unless a separately versioned sensitivity experiment explicitly changes one of these variables.

Reference-direction metadata may note the initial convention used when the prototype was created, but final provenance records the actual resolved implementation values rather than assuming them from documentation.

### Timed qualification requirement

For QA-01 scored timing:

```text
optimized_release_class_build = true
```

Debug/development build results may exist for Smoke/Logical Mode but are not eligible for QA-01 final scoring.

---

# 3. Runtime mode provenance

Required field:

```text
runtime_mode
```

Initial values:

```text
SMOKE_LOGICAL
TIMED_QUALIFICATION
REAL_STACK_VALIDATION
```

Recommended associated fields:

```text
instrumentation_profile_id/version
telemetry_buffer_profile_id/version
logging_profile_id/version
machine_profile_id/version
power_profile_id/version
background_load_profile_id/version
warm_cold_state_profile_id/version
```

This prevents debug/smoke runs from being accidentally mixed with timed qualification results.

---

# 4. Timing / instrumentation provenance

QA-01 requires evidence that both interval endpoints belong to the same monotonic clock domain.

Preserve:

```text
monotonic_clock_kind
monotonic_clock_unit
monotonic_clock_origin
clock_profile_version

acoustic_eos_authority
useful_outcome_authority

in_timed_window_serialization_enabled
in_timed_window_file_io_enabled
in_timed_window_python_enabled
in_timed_window_console_logging_profile
```

For compliant Timed Qualification these should normally establish:

```text
in_timed_window_serialization_enabled = false
in_timed_window_file_io_enabled       = false
in_timed_window_python_enabled        = false
```

Recommended instrumentation-validity fields:

```text
instrumentation_overhead_check_version
instrumentation_overhead_check_status
instrumentation_overhead_notes_ref
```

Pilot must validate that telemetry collection does not dominate or reverse architecture ranking before final timing rules are frozen.

---

# 5. Python offline-analysis provenance

Python does not participate in the runtime timed path, but derived results should record the analysis environment that produced them.

Recommended fields on derived-result provenance, or immutable references from the run bundle:

```text
analysis_language = python
python_interpreter_version
python_environment_id
python_dependency_lock_hash
analysis_source_git_commit
analysis_code_version
derivation_version
statistics_method_version
visualization_source_version
```

The project follows its normal `.venv/` convention for Python tooling.

Changing Python analysis/scoring code creates a new derived/scoring version. It does not mutate runtime raw evidence.

---

# 6. Prototype serialization provenance

Prototype/Harness v0 preferred serialization family:

```text
runtime scenario manifest  = JSON
semantic behavior plan     = JSON
run provenance             = JSON
canonical event stream     = JSONL
model-call stream          = JSONL
fixture-event stream       = JSONL when used
```

Record the concrete serialization profile/version used by the implementation:

```text
serialization_profile_id
serialization_profile_version
```

This is evaluation-infrastructure provenance, not a DP-00 architecture variable.

Raw serialization should occur after the QA-01 timed interval rather than becoming synchronous architecture-path I/O.

---

# 7. QA eligibility / boundary observations

Preserve at run/episode level:

```text
qa01_eligible
qa02_eligible
qa04_eligible

route_commit_expectation
route_commit_observed
execution_route_commit_ts

success
exact_conformance
```

For QA-01 also preserve whether a Voice fixture and ground-truth Acoustic EOS were available.

For QA-04, no-route-commit handling is not decided by this schema. The raw facts remain available for the future frozen aggregation rule.

---

# 8. Semantic operation consumption summary

Per-operation details may live in replay/model-call records, but run provenance should support a summary or reference:

```text
semantic_responsibilities_requested[]
semantic_operations_resolved[]
semantic_operation_attempts_consumed[]
semantic_operations_unused[]
replay_validation_failure_ids[]
```

This allows reviewers to distinguish:

- operations planned by the frozen scenario;
- semantic responsibilities actually requested by the architecture;
- hidden operation keys resolved by the Replay Adapter;
- attempts consumed due to retry;
- deterministic responsibilities that required no model replay.

The AUT itself does not receive `semantic_operations_resolved[]` or behavior-plan identities.

---

# 9. Oracle / benchmark-leakage audit

Recommended raw integrity fields:

```text
stimulus_materialization_version
oracle_materialization_version
oracle_access_violation_detected
oracle_dependency_graph_check_status
scenario_id_decision_access_violation_detected
benchmark_operation_key_leakage_detected
observation_provenance_spoof_detected
```

The implementation should enforce isolation structurally through crate/API boundaries and negative contract tests rather than relying only on post-hoc flags.

These fields provide auditable evidence of those guards.

---

# 10. Outcome authority audit

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

# 11. Canonical route projection

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

# 12. Source-role / QA-03 linkage

The prototype source tree must support a frozen classification:

```text
EVALUATION_SUPPORT
ARCHITECTURE_UNDER_TEST
```

Recommended run/evolution provenance references:

```text
source_role_mapping_version
architecture_source_root
benchmark_support_source_roots[]
```

This supports later QA-03 analysis without counting benchmark/fixture edits as architecture propagation.

Architecture-owned contracts must not be hidden under Evaluation Support to improve CCR.

---

# 13. Raw evidence layout

Recommended runtime run bundle:

```text
results/raw/<run-id>/
├─ provenance.json
├─ canonical-events.jsonl
├─ model-calls.jsonl
└─ fixture-events.jsonl     # optional
```

Derived results remain under versioned `results/derived/`; reports/plots under `results/reports/`.

---

# 14. Raw-data immutability

`results/raw/` is append-only/immutable experimental evidence.

Changing scenario/replay/fixture/profile/runtime meanings creates a new version and new run. It does not rewrite provenance on historical results.

Derived micro/macro averages, QA scores and sensitivity analyses belong under `results/derived/` or `results/reports/`.

---

# 15. Relationship to existing schemas/contracts

- `run-event-schema.md` — ordered runtime events and QA timestamp/decision projections.
- `model-call-schema.md` — per-logical-generation evidence.
- `runtime-scenario-schema.md` — frozen scenario definition.
- `semantic-behavior-plan-schema.md` — responsibility-based semantic behavior.
- `canonical-event-schema.md` — architecture-neutral stored observation vocabulary.
- `benchmark/contracts/model-replay-request-contract.md` — AUT-visible ModelRequest vs hidden ReplayContext.
- `benchmark/contracts/observation-port-contract.md` — AUT observation vs benchmark-owned provenance enrichment.
- `docs/evaluation/prototype-benchmark-harness-spec.md` — Rust/Python implementation boundary and harness/module rules.
- this file — immutable run-level provenance tying those contracts together.

The implementation may serialize these fields into one run header plus references rather than duplicate them on every event.
