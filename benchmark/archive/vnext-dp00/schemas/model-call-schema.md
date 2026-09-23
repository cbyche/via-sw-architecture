# Model Call Raw Telemetry Schema

## Status

Architecture Context Checkpoint 004, amended by Top-QA Cross-review — raw model-inference evidence contract for QA-04 Model Call Overhead.

This schema defines one record per **logical model generation**. It is the source of truth for QA-04 call counting and for Secondary analysis of token, cache, latency and resource behavior.

The concrete serialization format may later be JSON/JSONL/typed events/Parquet projections. The semantic fields and inclusion rules below are the contract.

Pilot-v0 persists these records in `model-calls.jsonl` after the timed window.
The Rust record includes logical start, optional first output, terminal timestamp
and status/failure timestamp, attempt, decision owner, semantic responsibilities,
call classification/inclusion metadata, and the related route-commit event id
when a commit exists. No-commit P08/P09/P10 evidence is retained without a
fabricated relation.

`model-call-v1` may additionally preserve a compact typed `semantic_output_reference` derived from the response actually returned by the Model Fixture. Pilot-v0 uses `EXECUTOR_CANDIDATE` plus the observed executor id so a no-commit wrong-candidate path (for example, an actual `MailAgent` proposal) remains reconstructable. This field is not an oracle value, expected executor, or permission to treat the model proposal as a committed architecture route. Full generative response bodies remain unnecessary.

## Core principle

QA-04 counts architecture-required logical generations, not transport requests or stream chunks.

```text
one new logical inference generation = one ModelCall record
```

A deterministic semantic replay/model double still emits a ModelCall record when the architecture requests a generation. The replay controls output stochasticity; it does not erase the architecture's requirement for inference.

## Minimum identity and correlation

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `schema_version` | string | yes | Model-call telemetry schema version |
| `model_call_id` | string | yes | Globally unique logical generation id |
| `run_id` | string | yes | Parent benchmark run id |
| `episode_id` | string | yes | Logical evaluation episode/user goal |
| `scenario_id` | string | yes | Scenario identifier |
| `scenario_class` | string | yes | QA-04 workload class, e.g. W1~W4 |
| `alternative_id` | string | yes | Architecture alternative, e.g. DP00-A |
| `alternative_version` | string/null | recommended | Prototype version |
| `parent_call_id` | string/null | recommended | Parent causal model call when applicable |
| `retry_of_call_id` | string/null | recommended | Original call id if this is a new retry generation |

Recommended correlation:

- user turn id;
- conversation id;
- task/work id;
- execution-attempt id;
- semantic trace id/class;
- benchmark/scoring version.

## Owner, component and purpose

Minimum fields:

```text
owner
component
purpose
```

### `owner`

Architecture/runtime authority that requested or owns the generation, for example:

- `via_frontend`;
- `via_intent`;
- `via_router`;
- `via_validator`;
- `argo`;
- `specialized_agent:<id>`;
- `voice_runtime`.

This is a logical architecture owner, not necessarily a process name.

### `component`

Concrete implementation component/module responsible for the call. This is diagnostic and may differ across A/B/C/D.

### `purpose`

Stable reason code(s) describing what the generation is doing, for example:

- `intent_refinement`;
- `referent_reasoning`;
- `execution_path_selection`;
- `agent_routing`;
- `validation`;
- `clarification_decision`;
- `task_association`;
- `result_association`;
- `execution_owner_selection`;
- `delegation_decision`;
- `architecture_retry`;
- `domain_planning`;
- `domain_reasoning`;
- `tool_result_interpretation`;
- `voice_output_only`.

`purpose` provides audit evidence for the required call classification.

## Call classification

Required field:

```text
call_class = ORCHESTRATION | DOMAIN | MIXED
```

### ORCHESTRATION

The generation performs architecture-level interpretation or coordination needed to establish or commit the final execution route.

Examples:

- intent/refinement;
- referent reasoning;
- execution-path selection;
- Agent routing;
- model-based validation;
- clarification decision;
- task/result association when route-relevant;
- execution-owner selection;
- delegation selection;
- architecture-level retry/fallback.

### DOMAIN

The generation performs substantive domain/task reasoning after the final execution route has been committed.

Examples:

- downstream Agent planning;
- domain ReAct loop;
- NetworkAgent diagnosis;
- MailAgent content analysis;
- report reasoning;
- domain tool-result interpretation.

### MIXED

One logical generation combines architecture-level route/owner/delegation decision and substantive domain reasoning.

Example: ARGO reasons about the task and in the same generation decides whether ARGO owns final execution or delegates to another Agent.

MIXED is counted once, not decomposed artificially.

Required field:

```text
classification_reason
```

This should contain a stable reason code and/or concise structured explanation supporting the classification.

## Model/runtime profile provenance

Minimum/recommended fields:

```text
model_role
model_profile_id
model_profile_version
provider
deployment
```

Possible `model_role` examples:

- `realtime_frontend`;
- `intent_model`;
- `router_model`;
- `validator_model`;
- `argo_primary`;
- `specialized_agent_model`;
- `voice_generation`.

Possible `deployment` values:

- `cloud`;
- `on_device_cpu`;
- `on_device_gpu`;
- `on_device_npu`;
- `hybrid`;
- `deterministic_replay`.

For Architecture Qualification, model/profile versions and replay behavior must be frozen according to the run plan.

## Prompt and cache provenance

Recommended fields:

```text
prompt_profile_version
cache_policy_version
cache_enabled
cache_hit
```

These are not Primary scoring variables but are required to interpret token/resource differences and to reproduce controlled experiments.

## Logical timing fields

All timestamps use the run's monotonic clock domain.

Minimum fields:

```text
logical_start_ts
first_output_ts
completion_ts
```

Definitions:

- `logical_start_ts`: new inference generation begins;
- `first_output_ts`: first semantic/token/audio model output is available;
- `completion_ts`: logical generation completes or terminates.

Transport frames/chunks are not additional model calls.

## Token/context telemetry

Preserve when available:

```text
input_tokens
cached_input_tokens
uncached_input_tokens
output_tokens
context_bytes
```

Validation where all fields are available:

```text
cached_input_tokens + uncached_input_tokens <= input_tokens
```

Provider/tokenizer differences must be identified by profile/version; token counts are diagnostic rather than architecture-only Primary scoring values.

## Resource telemetry

Preserve when measurable:

```text
cpu_time
gpu_time
npu_time
peak_memory
energy_if_available
provider_cost_if_available
```

Recommended accompanying units/profile fields:

- CPU/GPU/NPU time unit;
- memory unit;
- energy unit/method;
- provider pricing-table version/currency for derived monetary cost.

Absence of hardware/provider telemetry must be represented as unavailable/null, never fabricated as zero.

## Critical-path telemetry

Required/recommended field:

```text
on_critical_path
```

This enables Secondary metrics such as Critical-path Model Calls and Critical-path Inference Time.

The concrete critical-path reconstruction algorithm must be versioned if used for scoring in the future.

## Execution-route boundary evidence

QA-04's authoritative end boundary is **Execution Route Commit**.

For a compound request, this means the final commit among the complete
predeclared set of required initial subgoal routes. A first subgoal commit does
not close QA-04 while another required initial subgoal is uncommitted. The
evaluator derives this boundary from subgoal-correlated canonical events; it is
not represented by a synthetic parent commit or boundary flag. Partial required
coverage has no numeric boundary and is `ROUTE_REQUIRED_NOT_COMMITTED`.

The ModelCall record must preserve whether that route had already been committed before the call and whether the call caused or crossed that boundary:

```text
route_committed_before_call
route_committed_after_call
call_caused_route_commit
```

Recommended associated fields:

```text
execution_route_before_call
execution_route_after_call
final_execution_owner_after_call
delegated_agent_after_call
```

The parent episode/run records:

```text
request_processing_start_ts
execution_route_commit_ts
execution_route
final_execution_owner
delegated_agent_if_any
```

### Preserved owner/start diagnostics

The previous checkpoint fields remain useful and must not be deleted solely because they no longer define the QA-04 boundary:

```text
execution_owner_confirmed_before_call
execution_owner_confirmed_after_call
execution_owner_before_call
execution_owner_after_call
execution_started_before_call
execution_started_after_call
```

The parent episode also retains:

```text
execution_owner_confirmed_ts
execution_started_ts
```

These fields make it possible to detect the hidden-routing case where an intermediate owner starts before the final route is committed.

## QA-04 Primary inclusion fields

Required:

```text
included_in_qa04_primary
exclusion_reason
```

A call is included when all are true:

1. it belongs causally to the episode's request-processing-to-route-commit decision chain;
2. `call_class` is `ORCHESTRATION` or `MIXED`;
3. the route was not committed before the call, **or** this call is the generation whose output causes `Execution Route Commit`;
4. it is not pure voice/output-only inference.

Plain rule:

```text
included_in_qa04_primary =
  call_class in {ORCHESTRATION, MIXED}
  AND causal_to_episode_route_decision
  AND (
        route_committed_before_call == false
        OR call_caused_route_commit == true
      )
  AND purpose != voice_output_only
```

A `DOMAIN` call is excluded from Primary even though it remains raw telemetry.

An ORCHESTRATION/MIXED call after a previously committed final route is excluded from this episode's QA-04 Primary.

### Example exclusion reasons

- `domain_reasoning`;
- `after_execution_route_commit`;
- `pure_voice_output`;
- `not_attributable_to_episode_route_decision`;
- `transport_retry_no_new_generation`;
- `diagnostic_or_shadow_call_not_used_by_architecture`.

### Retry rule

If an architecture retry starts a new logical generation before the final route is committed:

```text
retry_of_call_id = <prior-call>
included_in_qa04_primary = true
```

when the retry is ORCHESTRATION/MIXED and causally part of route commitment.

A network retry that resumes the same logical generation creates no additional ModelCall record.

## Hidden-routing / boundary-gaming audit rule

A final qualification run should explicitly detect:

```text
execution_started_before_call == true
AND route_committed_before_call == false
AND call_class in {ORCHESTRATION, MIXED}
```

This is a valid shape in an ARGO-centric or other layered runtime: an intermediate execution owner may already be running while the architecture still has a specialist-delegation decision to make.

Such a call remains QA-04 eligible until the final route is committed.

Do **not** exclude it merely because `execution_started_ts` occurred earlier.

## Pure Voice/S2S handling

A generation whose only purpose is voice rendering/interaction output:

```text
purpose = voice_output_only
included_in_qa04_primary = false
exclusion_reason = pure_voice_output
```

If the same S2S generation also performs intent/routing/validation/delegation/owner selection:

```text
call_class = MIXED
```

and it is included when it occurs before or causes Execution Route Commit.

## Deterministic replay handling

Architecture Qualification may replace a live model with a deterministic replay/model double.

Required rule:

```text
architecture requests a new model generation
  -> create one logical ModelCall record
  -> provider/deployment identifies replay profile
  -> call is classified/included by normal route-commit rules
```

Do **not** set model-call count to zero merely because no cloud/local neural inference physically ran during replay.

Conversely, a deterministic architecture branch that makes a decision without requesting a model generation creates no ModelCall record.

## Episode-level projections

Per-call events are the source of truth. The benchmark may derive/cache episode projections:

```text
route_commit_orchestration_call_count
route_commit_mixed_call_count
route_commit_total_primary_call_count

total_orchestration_call_count
total_domain_call_count
total_mixed_call_count
total_model_call_count
```

Required derived relation:

```text
route_commit_total_primary_call_count
  = route_commit_orchestration_call_count
  + route_commit_mixed_call_count
```

The QA-04 Primary Metric is the mean of `route_commit_total_primary_call_count` over the frozen eligible/completed population.

Previous projection names such as `pre_execution_*` may be retained in old raw-schema versions for compatibility/diagnosis, but they are not the authoritative Primary projection after this amendment.

## Secondary metrics supported

Raw ModelCall records must support derivation of at least:

- Total Model Calls / Episode;
- ORCHESTRATION/MIXED/DOMAIN counts;
- Critical-path Model Calls;
- calls by purpose;
- retry count;
- input/output/cached/uncached token totals;
- context bytes;
- cache hit rate;
- model/prompt/cache profile slices;
- model inference latency;
- CPU/GPU/NPU time;
- peak memory;
- energy when measurable;
- provider monetary cost when available;
- Calls per exact-conform episode;
- request-class-specific call counts;
- time from intermediate execution start to final route commit;
- count of route-selection/delegation generations after intermediate execution start.

## Validation rules

A final qualification run should reject/flag model-call evidence when:

- a logical generation has no unique `model_call_id`;
- classification is missing;
- a retry generation has no causal link when one is expected;
- `included_in_qa04_primary=true` for a pure `DOMAIN` call without an explicit schema-versioned exception;
- a voice-output-only call is included without evidence that it also performed orchestration;
- one streaming generation is split into multiple ModelCall records solely because it emitted multiple chunks;
- a new retry generation is hidden inside the original call record;
- route-commit timestamps/correlation are missing for a QA-04 eligible episode;
- a call that causes route commit is excluded solely because the route becomes committed after that call;
- an ORCHESTRATION/MIXED delegation call is excluded solely because an intermediate owner already started;
- model/prompt/cache profile versions do not match the frozen run plan;
- raw records cannot reproduce the episode-level Primary count.

## Schema evolution

Historical `results/raw/` records are immutable.

Semantic changes to call classification, inclusion rules, Execution Route Commit boundary, model-profile meaning or aggregation require new schema/benchmark/scoring versions as appropriate.

Do not change classification rules after final A/B/C/D results are known merely to alter ranking.

## Relationship to other schemas

- `benchmark/schemas/run-event-schema.md` owns runtime episode identity, request-processing/route-commit timestamps and episode projections.
- This file owns per-logical-generation model telemetry and QA-04 inclusion evidence.
- `benchmark/schemas/scenario-constraint-schema.md` supplies QA-02 exact-conformance evidence used for slices such as Calls per exact-conform episode.
- QA-03 evolution schemas remain separate because they describe source-code/change-set experiments rather than runtime model calls.
