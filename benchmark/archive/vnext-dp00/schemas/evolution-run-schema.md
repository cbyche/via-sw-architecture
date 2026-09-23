# Evolution Experiment Raw Run Schema

## Status

Architecture Context Checkpoint 003 — raw evidence contract for QA-03 Flexibility.

This schema is intentionally separate from `benchmark/schemas/run-event-schema.md`.

Runtime interaction benchmarks primarily record time-ordered execution events. QA-03 evolution experiments primarily record a **code/configuration change set, architecture-role propagation, and acceptance/regression evidence** between a frozen baseline and a result commit. Forcing these into one event vocabulary would make both schemas less clear.

The concrete storage format may later be JSON/YAML/JSONL/structured records. The semantic fields below are the contract.

## Storage policy

Use the existing repository policy:

```text
results/raw/      immutable experiment evidence
results/derived/  recomputable CCR and diagnostic metrics
results/reports/  human-readable summaries/visualizations
```

Do not store only final CCR. Raw evidence must be sufficient to independently recompute every scenario's containment result.

## Minimum run identity

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `schema_version` | string | yes | Evolution raw-run schema version |
| `run_id` | string | yes | Unique evolution experiment run |
| `evolution_scenario_id` | string | yes | Stable scenario id |
| `scenario_version` | string | yes | Frozen scenario definition version |
| `taxonomy_version` | string | yes | E1~E5 taxonomy version |
| `corpus_version` | string | yes | Frozen evolution scoring corpus |
| `alternative_id` | string | yes | Architecture alternative, e.g. DP00-A |
| `alternative_version` | string/null | recommended | Prototype architecture version |
| `benchmark_version` | string | yes | Evolution benchmark harness version |
| `scoring_version` | string/null | yes | Frozen scoring version; may be null during pilot |
| `baseline_git_commit` | string | yes | Commit before applying the evolution requirement |
| `result_git_commit` | string | yes | Commit representing the completed change attempt |

Recommended additional provenance:

- architecture-role vocabulary version;
- Expected Change Area version;
- role-mapping version;
- acceptance suite version;
- regression suite version;
- implementation runner/agent/tooling version if automation is used;
- operator/developer identifier class if needed for audit, without treating it as a scoring variable.

## Frozen scenario boundary fields

Raw evidence must include or immutably reference the exact frozen scenario contract used for the run.

Minimum fields:

```text
expected_change_roles
forbidden_change_roles
alternative_role_mapping
```

These must be identical to the versions frozen before implementation/result observation.

Recommended:

```text
expected_change_area_version
architecture_role_vocabulary_version
role_mapping_version
scenario_contract_hash
```

A run whose mapping was changed after its result diff became visible is invalid for final CCR scoring.

## Actual change-set fields

Minimum raw fields:

```text
changed_files
changed_architecture_roles
changed_existing_roles
added_roles
removed_roles
unexpected_changed_roles
```

### `changed_files`

Store structured entries rather than only a count where possible:

```text
path
change_type   # added / modified / deleted / renamed
lines_added
lines_deleted
mapped_architecture_roles
is_existing_file
```

Generated/binary artifacts should be classified explicitly rather than silently included in LOC diagnostics.

### `changed_architecture_roles`

All architecture roles affected by the change, derived from the frozen alternative role mapping plus any newly introduced roles.

### `changed_existing_roles`

Subset of changed roles that existed in the baseline architecture.

This distinction matters because adding a new Agent-specific adapter role can be expected while modifying an unrelated existing core role is propagation.

### `added_roles` / `removed_roles`

Architecture roles introduced or removed by the evolution change where relevant.

A new role is not automatically a containment failure. Its classification depends on whether the scenario permits role introduction and whether it remains inside the semantic Expected Change Area.

### `unexpected_changed_roles`

The core containment evidence.

This is the set of changed existing architecture roles not permitted by the frozen Expected Change Area, plus explicitly forbidden roles that changed.

Each entry should preserve:

```text
role_id
mapped_files
change_reason_if_known
constraint/boundary_violation_id
```

## Test-result fields

A scenario counts as contained success only if both requested functionality and prior functionality pass.

Minimum fields:

```text
acceptance_test_result
regression_test_result
```

Recommended structured form:

```text
acceptance_test_result:
  suite_version
  pass
  tests_total
  tests_passed
  tests_failed
  failed_test_ids

regression_test_result:
  suite_version
  pass
  tests_total
  tests_passed
  tests_failed
  failed_test_ids
```

Do not replace raw test evidence with one prose sentence.

## Contract/adapter change diagnostics

Minimum/recommended fields:

```text
stable_contract_changed
stable_contract_change_ids
existing_adapter_changed
existing_adapter_change_ids
```

These are diagnostic signals, not automatic containment failures by themselves.

Whether a contract/adapter change is expected depends on the scenario's frozen role boundary. For an E5 contract-evolution scenario, a common contract can be inside Expected Change Area. For E1 “add Agent,” the same contract can be forbidden.

## LOC/file diagnostics

Store at least:

```text
lines_added
lines_deleted
changed_file_count
changed_existing_role_count
unexpected_changed_role_count
```

Optional:

```text
lines_modified_estimate
renamed_file_count
generated_file_count
```

These do not determine the Primary score. They allow later severity/sensitivity analysis.

## Propagation-depth diagnostics

Recommended field:

```text
change_propagation_stages
```

Possible representation:

```text
[
  {
    "from_role": "AGENT_SPECIFIC_INTEGRATION",
    "to_role": "COMMON_AGENT_CONTRACT",
    "reason": "new metadata not expressible through current seam"
  },
  {
    "from_role": "COMMON_AGENT_CONTRACT",
    "to_role": "TASK_MANAGER_CORE",
    "reason": "core stores backend-specific field"
  }
]
```

Derived metric candidates:

- propagation stage count;
- maximum propagation depth;
- number of unexpected role transitions.

These are Secondary only.

## Scenario containment result

Required fields:

```text
acceptance_pass
regression_pass
scenario_change_contained
failure_reason
```

Plain-text rule:

```text
scenario_change_contained =
  acceptance_pass
  AND regression_pass
  AND unexpected_changed_roles is empty
```

Equivalently:

```text
Feature PASS + Regression PASS + No unexpected propagation
= Contained Change Success
```

### `failure_reason`

Use structured/stable failure classes where possible, for example:

- `acceptance_failed`
- `regression_failed`
- `unexpected_role_propagation`
- `acceptance_and_propagation_failed`
- `regression_and_propagation_failed`
- `scenario_not_completed`
- `invalid_role_mapping`
- `invalid_or_unfrozen_scenario_contract`

Detailed fields should preserve the actual failed tests and unexpected roles rather than relying on this summary alone.

## Unexpected Changed Architecture Areas Count

Derived directly from:

```text
count(unexpected_changed_roles)
```

This is a key Secondary diagnostic because CCR is binary per scenario.

Example:

```text
Scenario A: scenario_change_contained = false, unexpected roles = 1
Scenario B: scenario_change_contained = false, unexpected roles = 5
```

Both contribute zero to CCR numerator, but the second has materially more severe propagation.

## Derived Change Containment Rate

For the frozen eligible QA-03 population:

```text
CCR (%) =
  count(scenario_change_contained == true AND eligible_for_ccr == true)
  ------------------------------------------------------------------ × 100
  count(eligible_for_ccr == true)
```

The denominator is the number of eligible evolution scenarios, not files/components/LOC.

Example:

```text
10 eligible scenarios
8 contained successes
CCR = (8 / 10) × 100 = 80%
```

## Secondary derived metrics

From the same raw evidence, derive at least:

- Unexpected Changed Architecture Areas Count distribution;
- changed existing architecture areas/roles count;
- changed file count;
- LOC added/deleted;
- stable contract change count;
- existing adapter change count;
- regression failure count;
- propagation-stage/depth metrics;
- E1~E5 category-specific CCR;
- optional usage-weighted CCR sensitivity result.

## Validation rules

A final qualification run should be rejected/flagged when:

- scenario/version/corpus ids are missing or not frozen;
- baseline/result commits are missing or equal when a code change was required;
- Expected Change Area or role mapping cannot be resolved;
- role mapping was edited after implementation/result observation;
- acceptance/regression suite versions do not match the frozen scenario;
- `scenario_change_contained=true` while acceptance or regression failed;
- `scenario_change_contained=true` while `unexpected_changed_roles` is non-empty;
- changed files cannot be mapped/classified well enough to determine affected architecture roles;
- final qualification uses unfrozen scoring/benchmark/taxonomy versions.

## Schema evolution

Schema changes are versioned. Historical `results/raw/` records remain immutable.

Backward-compatible additions may add optional diagnostics. Semantic changes to containment fields, role mapping, or scenario eligibility require a new schema/scenario/benchmark version.

## Relationship to other schemas

- `benchmark/schemas/evolution-scenario-schema.md` defines the frozen **input/oracle** for a change requirement.
- This file defines the raw **result evidence** after applying that change.
- `benchmark/schemas/run-event-schema.md` remains focused on runtime interaction/correctness event evidence and is not overloaded with Git-diff/evolution semantics.
