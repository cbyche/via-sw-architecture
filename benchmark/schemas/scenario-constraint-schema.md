# Architecture Scenario Constraint Manifest Contract

## Status

Architecture Context Checkpoint 002 — logical benchmark schema for QA-02.

This document defines the topology-neutral scenario oracle used by `QA-02 — VIA Interaction-Orchestration Correctness`.

It is a semantic contract for benchmark implementation. A concrete serialization format may later use JSON/YAML/typed structs, but must preserve the meanings below.

## Design goals

The schema must:

- compare structurally different architecture alternatives fairly;
- avoid encoding one preferred topology as the only correct path;
- express required, allowed and forbidden architecture outcomes;
- support multi-turn user-goal episodes;
- normalize implementation-specific behavior into canonical architecture outcomes;
- preserve constraint-level evidence so AECR and Secondary Metrics can be recomputed;
- remain versioned and frozen for final qualification runs.

## Scenario identity

Each scenario must contain at least:

| Field | Type | Meaning |
| --- | --- | --- |
| `scenario_id` | string | Stable scenario identifier |
| `scenario_version` | string | Version of this scenario definition |
| `taxonomy_version` | string | QA-02 taxonomy version |
| `primary_category` | enum | One of C1~C8 |
| `secondary_categories` | array | Additional architecture dimensions exercised |
| `corpus_version` | string | Frozen scoring-corpus version |
| `eligible_for_aecr` | boolean | Whether this episode belongs to the QA-02 scoring population |
| `fixture_id` | string | User/context/task-state fixture identifier |
| `semantic_trace_set_id` | string | Frozen semantic behavior set used by qualification |
| `constraint_manifest_version` | string | Version of the manifest semantics/content |

### QA-02 taxonomy

- `C1_CONTEXT_REFERENT_GROUNDING`
- `C2_TASK_ASSOCIATION_FOLLOWUP`
- `C3_EXECUTION_PATH_ELIGIBILITY`
- `C4_AGENT_ROUTING_DELEGATION`
- `C5_AMBIGUITY_CLARIFICATION`
- `C6_CONCURRENT_TASK_RESULT_BINDING`
- `C7_COMPOUND_REQUEST_DECOMPOSITION`
- `C8_DEPENDENCY_ERROR_INVALID_OUTPUT`

The category mix is part of the benchmark definition and must be frozen before final evaluation.

## Episode fixture

The fixture describes inputs shared by all alternatives, not internal component calls.

Recommended fields:

- user utterance/input sequence;
- ground-truth acoustic annotations where voice is used;
- interaction/context evidence fixture;
- existing conversation/task/session state;
- available Agent/capability registry state;
- policy/consent state;
- deterministic tool/service behavior;
- semantic replay traces;
- success/useful-outcome predicate where applicable.

A clarification episode may contain multiple user turns in one fixture.

## Constraint Manifest

The manifest contains three ordered sets:

```text
required_constraints[]
allowed_constraints[]
forbidden_constraints[]
```

Every constraint must have a stable id and machine-evaluable predicate.

### Common constraint fields

| Field | Type | Meaning |
| --- | --- | --- |
| `constraint_id` | string | Stable unique id within scenario/version |
| `dimension` | enum/string | e.g. `referent`, `task_association`, `execution_owner`, `routing`, `clarification`, `result_binding`, `decomposition` |
| `operator` | enum | equality/set membership/relation/predicate operator |
| `expected` | structured value | Required or allowed value/set/relation |
| `severity` | enum | `normal` / `critical` for diagnostic slices; does not change AECR semantics |
| `description` | string | Human-readable rationale |

Constraint descriptions are explanatory only. The machine predicate is authoritative.

## Required constraints

Required constraints express architecture outcomes that must occur.

Examples:

```text
REQ-REFERENT-SOURCE:
  dimension: referent
  operator: equals
  expected:
    role: source
    object_id: file_A

REQ-TASK-RELATION:
  dimension: task_association
  operator: equals
  expected: FOLLOW_UP

REQ-RESULT-BINDING:
  dimension: result_binding
  operator: equals
  expected: T1
```

If any required constraint is false, `episode_exact_conform = false`.

## Allowed constraints

Allowed constraints express a set of outcomes that are all correct.

Example:

```text
ALLOW-EXECUTION-OWNER:
  dimension: execution_owner
  operator: in
  expected: [ARGO, FILE_AGENT]
```

The actual value must satisfy the allowed set when the constraint applies.

Allowed sets are the primary mechanism for topology-neutral correctness. They must not be narrowed merely to favor the expected performance profile of one alternative.

## Forbidden constraints

Forbidden constraints express states/actions that must not occur.

Examples:

```text
FORBID-FAST:
  dimension: execution_owner
  operator: not_equals
  expected: VIA_FAST

FORBID-MAIL-AGENT:
  dimension: delegated_agent
  operator: not_equals
  expected: MAIL_AGENT
```

Any forbidden condition being violated makes the episode non-conformant.

## Canonical Architecture Decision Trace

Each architecture alternative must normalize implementation-specific activity into one canonical trace.

Minimum canonical fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `referent_bindings` | array | Role/mention → canonical fixture object ids |
| `task_relation` | enum/null | `NEW`, `FOLLOW_UP`, `STATUS`, `CANCEL`, etc. |
| `task_id` | string/null | Logical task selected/created |
| `execution_owner` | string/null | Owner that receives substantive execution authority |
| `execution_path` | array | Ordered ownership path, without requiring common internal components |
| `delegated_agent` | string/null | Final/next Downstream Agent when relevant |
| `clarification_action` | structured/null | Whether/what clarification was requested |
| `result_binding` | string/null | Logical task/user-goal to which progress/result was bound |
| `observable_effect` | structured/null | Canonical machine-observable effect/result reference |
| `decomposition` | array/null | Sub-goals and relations for compound requests |

Optional diagnostic fields may include confidence, validation decisions, rejected candidates and escalation reasons, but the oracle must not require component names that are absent in another valid topology.

## Topology-neutrality rule

Scenario constraints must describe **architecture semantics**, not implementation structure.

Do not require fields such as:

- `agent_router_called = true`;
- `intent_refiner_stage_count = 3`;
- `argo_called = false` merely because a different alternative is expected to be faster;
- presence of a named internal queue/service unless that existence is itself the DP under test and is expressed as an externally comparable property.

Instead constrain the normalized outcome:

- which referent was bound;
- whether the request was a follow-up;
- which execution owners are acceptable;
- which Agent is allowed/forbidden;
- whether clarification was required;
- where the result must be attached.

## Evaluation semantics

For an eligible episode:

```text
required_conform
  = all(required_constraints evaluate true)

allowed_conform
  = all(applicable allowed_constraints evaluate true)

forbidden_conform
  = all(forbidden constraints are not violated)

episode_exact_conform
  = required_conform
    && allowed_conform
    && forbidden_conform
```

There is no partial credit in AECR.

Per-constraint results are nevertheless mandatory raw evidence.

## Per-dimension conformance projection

The evaluator should derive booleans such as:

- `referent_conform`;
- `task_association_conform`;
- `execution_path_conform`;
- `routing_conform`;
- `clarification_conform`;
- `result_binding_conform`;
- `compound_decomposition_conform`.

A dimension with no applicable constraints should be represented as `null/not_applicable`, not automatically `true`, so macro metrics do not count absent tests as successes.

## Failure recording

For every non-conformant episode, raw output must include:

- `constraint_failure_ids`;
- `constraint_failure_reasons`;
- actual observed value(s);
- relevant canonical trace segment;
- semantic trace id/class;
- architecture alternative/version.

Do not store only `episode_exact_conform=false`.

## Example 1 — Simple local task

Scenario:

```text
"볼륨 줄여줘."
```

Manifest:

```text
Required:
  observable_effect.volume = requested_value

Allowed:
  execution_owner in {VIA_FAST, ARGO}

Forbidden:
  delegated_agent in {MAIL_AGENT, FILE_AGENT}
```

A Fast Path and ARGO-centric implementation can both conform. QA-01 captures their responsiveness difference.

## Example 2 — Existing task follow-up

Fixture:

- existing task `T1` owned by an ARGO execution/thread;
- unrelated task `T2` also active;
- user says “거기에 표 하나 더 넣어줘.”

Manifest:

```text
Required:
  task_relation = FOLLOW_UP
  task_id = T1
  result_binding = T1

Allowed:
  execution_path preserves T1 continuation semantics

Forbidden:
  task_id = NEW_TASK
  result_binding = T2
```

The benchmark does not require a specific internal task-association component.

## Example 3 — Required clarification

Fixture has two equally plausible PDFs and two contacts named 김대리.

Manifest:

```text
Required:
  clarification_action.required = true

Forbidden:
  observable_effect.external_send before ambiguity is resolved
```

A lucky action against one plausible candidate is still architecture incorrect.

## Semantic replay variants

One scenario may have multiple frozen semantic traces representing observed dependency behavior classes.

Example trace set:

- `trace-correct`;
- `trace-ambiguous`;
- `trace-wrong-fast`;
- `trace-wrong-agent`;
- `trace-malformed`;
- `trace-timeout`.

All alternatives must receive the same trace id for a comparable qualification episode.

The scoring corpus/version specifies which scenario×trace combinations are included.

## Coverage-balanced corpus rule

The final AECR corpus should ensure meaningful representation of C1~C8 architecture-sensitive categories.

Production-frequency weighting may be computed separately as a sensitivity analysis but must not silently change the frozen AECR population.

## Versioning and freeze

Final qualification requires frozen:

- scenario schema version;
- taxonomy version;
- scenario version;
- corpus version/composition;
- constraint manifest version;
- semantic trace set/version;
- scoring version.

Any semantic change to a scenario constraint requires a new scenario/manifest version. Historical raw results remain immutable.

## Validation rules

A qualification runner should reject or flag a scenario when:

- an eligible scenario has no constraints;
- a constraint lacks a stable id;
- Required/Allowed/Forbidden predicates contradict each other without an explicit conditional scope;
- a manifest references implementation-specific component state that cannot be normalized across alternatives;
- the actual trace omits a canonical field required by an applicable constraint;
- the scenario/corpus/manifest version does not match the frozen run plan;
- final qualification uses an unfrozen taxonomy/corpus version.

## Relationship to raw events

`benchmark/schemas/run-event-schema.md` stores:

- the manifest/version identifiers;
- actual canonical trace values;
- per-dimension/per-constraint conformance;
- failure ids/reasons;
- `episode_exact_conform`.

AECR is computed later under `results/derived/` and must not be treated as the only raw result.
