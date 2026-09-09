# Evolution Scenario Semantic Schema

## Status

Architecture Context Checkpoint 003 — logical benchmark schema for QA-03 Flexibility.

This document defines the **scenario contract** for evolution-based architecture evaluation. It answers: *what change is being requested, what architecture roles are expected to absorb it, and what tests prove the change worked without regression?*

It intentionally does not prescribe JSON/YAML yet. A future serialization format must preserve the semantics below.

## Design goals

The schema must:

- compare structurally different DP-00 alternatives fairly;
- define Expected Change Areas by architecture role rather than file/component count;
- freeze alternative-specific role mappings before implementation/result observation;
- require both feature acceptance and regression safety;
- separate scenario definition from raw run evidence;
- support recomputation of Change Containment Rate (CCR);
- remain versioned and immutable for final qualification.

## Minimum scenario identity

| Field | Type | Meaning |
| --- | --- | --- |
| `evolution_scenario_id` | string | Stable scenario identifier |
| `version` | string | Scenario definition version |
| `taxonomy_version` | string | E1~E5 taxonomy version |
| `category` | enum | Primary evolution category |
| `description` | string | Human-readable change requirement |
| `required_change` | structured text/object | Functional evolution requirement all alternatives must satisfy |
| `eligible_for_ccr` | boolean | Whether this scenario belongs to the Primary CCR population |
| `corpus_version` | string | Frozen final evaluation corpus version |

## Evolution taxonomy

Minimum categories:

- `E1_AGENT_CHANGE`
- `E2_VOICE_MODEL_CHANGE`
- `E3_CONTEXT_INPUT_CHANGE`
- `E4_CAPABILITY_PLACEMENT_CHANGE`
- `E5_CONTRACT_CHANGE`

A scenario may name secondary categories, but one primary category is required for corpus balancing/reporting.

## Common Expected Change Area

The scenario defines the expected boundary using **architecture roles**, not files.

Required fields:

```text
expected_change_roles
forbidden_change_roles
```

### `expected_change_roles`

Architecture roles that are allowed/expected to change for this evolution requirement.

Example — New Specialized Agent:

```text
expected_change_roles:
- AGENT_SPECIFIC_INTEGRATION
- CAPABILITY_REGISTRATION
- SCENARIO_TEST
```

### `forbidden_change_roles`

Existing architecture roles that should remain stable for this scenario unless the scenario itself explicitly changes their contract.

Example:

```text
forbidden_change_roles:
- VOICE_ENGINE_CORE
- CONTEXT_ENGINE_CORE
- TASK_MANAGER_CORE
- UNRELATED_AGENT_INTEGRATION
- STABLE_COMMON_AGENT_CONTRACT
```

A role may be omitted from both sets when it is irrelevant/not present. The exact role vocabulary must itself be versioned.

## Alternative-specific role mapping

Because A/B/C/D may implement the same role with different components/files, each scenario stores a mapping for each alternative.

Conceptual field:

```text
alternative_role_mapping
```

Example shape:

```text
DP00-A:
  AGENT_SPECIFIC_INTEGRATION:
    - prototypes/dp00/a/agents/<new-agent>/
  CAPABILITY_REGISTRATION:
    - prototypes/dp00/a/registry/agents.*

DP00-B:
  AGENT_SPECIFIC_INTEGRATION:
    - prototypes/dp00/b/argo/delegation/<new-agent>/
  CAPABILITY_REGISTRATION:
    - prototypes/dp00/b/argo/capabilities.*
```

The concrete path syntax may evolve, but the semantics are mandatory.

### Freeze rule

`alternative_role_mapping` is frozen **before change implementation and before result/diff observation**.

Changing it after seeing the diff invalidates containment comparability. A correction requires a new scenario/mapping version and new qualification run.

## Acceptance test contract

Field:

```text
acceptance_tests
```

These tests prove the requested evolution is implemented.

Each test entry should contain:

- stable test id;
- purpose;
- executable selector/command or harness reference;
- expected pass condition;
- version.

A scenario cannot count as contained success if acceptance tests fail, even if its diff is small.

## Regression test contract

Field:

```text
regression_tests
```

The regression suite verifies pre-existing behavior required to remain stable.

Each scenario should reference:

- regression suite id/version;
- scenario-specific critical regression tests where applicable;
- expected pass condition.

Regression scope must be equivalent across alternatives where the product behavior is equivalent.

## Baseline provenance

Recommended frozen scenario fields:

```text
baseline_git_commit
baseline_architecture_version
benchmark_version
scoring_version
architecture_role_vocabulary_version
role_mapping_version
acceptance_suite_version
regression_suite_version
```

The baseline commit may differ physically across alternative branches if prototypes are separate, but each must represent the declared same experiment baseline/version.

## Scenario weighting

Primary CCR should use the frozen qualification population without silent production-frequency weighting.

Optional fields:

```text
usage_weight
criticality_tags
secondary_sensitivity_group
```

These support Secondary sensitivity analysis only unless a future scoring version explicitly changes the Primary population definition.

## Example scenario — add a Specialized Agent

```text
evolution_scenario_id: E1-ADD-SPECIALIZED-001
version: v1
category: E1_AGENT_CHANGE
required_change:
  Add a specialized File Agent that can be selected/delegated for the fixture tasks.

expected_change_roles:
- AGENT_SPECIFIC_INTEGRATION
- CAPABILITY_REGISTRATION
- SCENARIO_TEST

forbidden_change_roles:
- VOICE_ENGINE_CORE
- CONTEXT_ENGINE_CORE
- TASK_MANAGER_CORE
- UNRELATED_AGENT_INTEGRATION
- STABLE_COMMON_AGENT_CONTRACT

acceptance_tests:
- agent can be discovered/selected where allowed
- fixture request completes through the new Agent

regression_tests:
- existing general Agent scenarios pass
- existing specialized Agent scenarios pass
- task/progress/cancel contract tests pass
```

If an alternative requires changing `TASK_MANAGER_CORE` to add this Agent, the scenario may still pass functionally, but `scenario_change_contained` must be false.

## Example scenario — contract extension

E5 scenarios are intentionally different: a stable common contract may itself be the object of change.

Example:

```text
required_change:
  Add agent resource-requirement metadata.

expected_change_roles:
- COMMON_AGENT_CONTRACT
- RESOURCE_METADATA_ADAPTERS
- CONTRACT_TESTS

forbidden_change_roles:
- VOICE_CAPTURE_CORE
- UNRELATED_CONTEXT_SOURCE
```

This demonstrates why Expected Change Area is scenario-specific. The same role can be forbidden in one scenario and expected in another.

## Scenario validation rules

Before final qualification, reject/flag a scenario when:

- no versioned `expected_change_roles` exists;
- no alternative-specific role mapping exists for one evaluated alternative;
- mappings were created/modified after implementation/result observation;
- no acceptance tests are declared;
- no regression suite/version is declared;
- expected and forbidden role sets conflict without explicit rationale;
- scenario category/corpus/scoring versions are unfrozen;
- one alternative is given materially broader semantic Expected Change Roles than another without documented topology-neutral justification.

## Relationship to QA-03

For each eligible scenario, raw run evidence later determines:

```text
acceptance_pass
regression_pass
unexpected_changed_roles
scenario_change_contained
```

A scenario contributes to CCR numerator only when:

```text
acceptance_pass == true
AND regression_pass == true
AND unexpected_changed_roles is empty
```

See `benchmark/schemas/evolution-run-schema.md` for the raw result contract.
