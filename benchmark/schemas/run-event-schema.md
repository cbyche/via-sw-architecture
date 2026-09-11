# Architecture Benchmark Raw Run Event Contract

## Status

Architecture Context Checkpoint 001/002/004 + Top-QA Cross-review — logical runtime experiment schema.

This is a logical event schema for benchmark implementation. It captures enough raw evidence so QA-01, QA-02, QA-04 and future Secondary Metrics can be recomputed without rerunning experiments solely because an intermediate observation was discarded.

QA-03 evolution experiments remain intentionally separated into dedicated evolution schemas because their evidence is source-change/test oriented rather than runtime-event oriented.

> **측정하지 않은 값은 나중에 복구할 수 없지만, raw data로 보존한 값은 나중에 다른 metric으로 재해석할 수 있다.**

## Storage policy

```text
results/raw/      immutable run events
results/derived/  recomputable metrics / aggregates
results/reports/  human-readable summaries and visualizations
```

Raw records are append-only/immutable experimental evidence. If instrumentation or schema meaning changes, increment a schema/benchmark version and create new runs rather than rewriting old events.

Derived metrics such as FTOL p95, AECR and QA-04 Average Model Calls to Commit Execution Route must never be the only persisted evidence.

### QA-02 raw-only reconstruction rule (`canonical-event-v2`)

| Constraint dimension | Actual raw evidence source | Oracle expected source |
| --- | --- | --- |
| `referent` / `referent_binding` | AUT `referent.bound` payload | oracle required/forbidden referent constraint |
| `task_association` | AUT `task.associated` plus product task correlation | oracle task-relation/task-id constraint |
| `execution_path`, `execution_owner`, `delegated_agent`, `routing` | AUT `execution.route_committed`; fixture-owned `execution.started.invocation`; actual model semantic reference and terminal reason for no-commit cases | Required/Allowed/Forbidden route constraints |
| `clarification` | AUT requested/resolved events and request/response turn correlation | oracle clarification constraint |
| `result_binding` | AUT `result.bound` product result/task/execution correlation | oracle expected-result-binding constraint |
| `observable_effect` | `OUTCOME_PROBE` typed effect payload including actual capability and required before/after transition boundaries | success predicate and observable-effect constraint |
| `failure_outcome` | canonical `episode.failed.failure_reason` | oracle allowed/required failure outcome |

An evaluator must not reconstruct missing actuals from a behavior plan, scenario definition, scenario id, path name, run order, or oracle. A missing applicable raw actual is a contract insufficiency/error.

The exhaustive Pilot mapping is versioned at `benchmark/contracts/pilot-v0-constraint-evidence-map.json`. It maps every evaluator-only constraint id to raw event/field paths, authority, and any deterministic derivation rule; it stores no run actuals or expected values.

## Time basis

Latency metrics must use a **monotonic clock**.

Wall-clock time may additionally be logged for operational correlation, but must not be used to calculate sub-run latency because system-clock adjustments can invalidate intervals.

Each run records a monotonic origin and all event timestamps use the same origin/unit.

Recommended unit: integer nanoseconds or microseconds from monotonic origin. Concrete serialization remains an implementation decision.

## Minimum run identity fields

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `schema_version` | string | yes | Raw event contract version |
| `run_id` | string | yes | Globally unique benchmark run identity |
| `episode_id` | string | recommended | Logical user-goal episode; may equal `run_id` in one-episode-per-run harnesses |
| `scenario_id` | string | yes | Stable scenario identifier |
| `scenario_category` | string | yes | e.g. QA-01 `F1`/`F2`/`F3`, QA-02 `C1`~`C8`, QA-04 workload class |
| `scenario_version` | string | recommended | Scenario definition version |
| `scenario_corpus_version` | string | yes | Frozen scoring-corpus version |
| `scenario_taxonomy_version` | string/null | recommended | Taxonomy version where applicable |
| `alternative_id` | string | yes | Architecture alternative, e.g. `DP00-A` |
| `alternative_version` | string/null | recommended | Prototype/architecture implementation version |
| `benchmark_version` | string | yes | Benchmark runner/corpus contract version |
| `source_git_commit` | string | yes | Source commit being evaluated |
| `oracle_trace_id` | string/null | yes | Frozen semantic/oracle trace; null only when not applicable |
| `semantic_trace_class` | string/null | recommended | e.g. correct, ambiguous, wrong-fast, wrong-agent, malformed, timeout |
| `scoring_version` | string/null | yes | Frozen scoring version; may be null during Pilot |
| `monotonic_clock_origin` | string/object | yes | Timestamp origin and unit |

Additional recommended provenance:

- Agent-stub version;
- tool/external dependency latency profile id/version;
- machine profile id;
- OS/build;
- power profile;
- warm/cold state;
- actual-model fidelity-run flag;
- model profile/version;
- prompt profile/version;
- cache policy/version.

## Runtime boundary timestamps

All timestamps are monotonic offsets from the same run origin and are nullable when not applicable.

| Field | Meaning |
| --- | --- |
| `acoustic_eos_ts` | Ground-truth Acoustic End-of-Speech. QA-01 FTOL start. |
| `request_processing_start_ts` | Architecture begins processing the user goal for execution-decision purposes. QA-04 start boundary; may precede EOS for intentional partial/streaming semantics. |
| `semantic_request_start_ts` | Architecture requests semantic interpretation/replay output. |
| `semantic_response_ts` | Semantic dependency output becomes available. |
| `execution_path_selected_ts` | Architecture chooses an execution-path candidate. Candidate selection is not necessarily final route commitment. |
| `execution_owner_confirmed_ts` | Some execution owner is confirmed. Preserved diagnostic; may be intermediate. |
| `agent_dispatch_ts` | Request crosses a Downstream Agent dispatch boundary. |
| `execution_started_ts` | Confirmed/intermediate owner starts or accepts execution. Preserved diagnostic; not QA-04 authoritative end after cross-review. |
| `execution_route_commit_ts` | **QA-04 authoritative end boundary**: final domain execution route is operationally committed and no further owner/delegation decision is required before domain execution continues. |
| `tool_start_ts` | Controlled local/Agent action begins, when observable at benchmark seam. |
| `tool_complete_ts` | Controlled action reaches completion. |
| `first_user_visible_result_ts` | First meaningful user-visible result delivery begins. |
| `useful_outcome_ts` | First observable state satisfying the scenario success predicate. QA-01 FTOL endpoint. |
| `task_complete_ts` | Logical task/workflow completes. |

Do not fabricate timestamps for paths that do not have the corresponding stage.

### QA-04 boundary rule

`execution_path_selected_ts`, `execution_owner_confirmed_ts`, and `execution_started_ts` do **not** independently close QA-04.

QA-04 closes at `execution_route_commit_ts`.

This matters when an intermediate runtime such as ARGO starts and only later decides to delegate to a specialist.

## Execution topology fields

| Field | Type | Meaning |
| --- | --- | --- |
| `execution_owner` | string | Current/selected substantive execution authority where useful |
| `execution_path` | array/string | Observed ownership/handoff path |
| `execution_route` | array/string | Final route committed for QA-04, e.g. `via_fast_path`, `argo`, `argo->network_agent`, `via->file_agent` |
| `final_execution_owner` | string/null | Final domain execution owner after route commit |
| `delegated_agent_if_any` | string/null | Final delegated Agent when route commitment includes delegation |
| `handoff_count` | integer | Ownership-boundary crossings |
| `model_invocation_count` | integer | Legacy/general logical model invocation projection; QA-04 uses ModelCall records instead |
| `agent_invocation_count` | integer | Downstream Agent prompt/turn invocations if separately measurable |
| `tool_invocation_count` | integer | Controlled tool/action invocations |

Streaming token/audio chunks are not separate model invocations. Detailed logical-call telemetry is defined in `benchmark/schemas/model-call-schema.md`.

## QA-01 controlled dependency fields

Architecture Qualification must preserve the dependency profile used for the episode so equality/non-dominance can be audited.

Recommended fields:

```text
dependency_latency_profile_id
dependency_latency_profile_version
controlled_agent_latency_ms
controlled_tool_latency_ms
controlled_service_latency_ms
qa01_dependency_profile_frozen
```

Do not infer these values only from final FTOL. The run must identify the frozen profile that generated the controlled dependency behavior.

Class-specific and overall FTOL can then be checked for fixture-tail dominance during Pilot/calibration.

## QA-04 episode model-call projections

Per-call `ModelCall` records are the source of truth. Episode/run records may cache or derive:

```text
route_commit_orchestration_call_count
route_commit_mixed_call_count
route_commit_total_primary_call_count

total_orchestration_call_count
total_domain_call_count
total_mixed_call_count
total_model_call_count
```

Required relation:

```text
route_commit_total_primary_call_count
  = route_commit_orchestration_call_count
  + route_commit_mixed_call_count
```

QA-04 Primary is computed from `route_commit_total_primary_call_count`, not unclassified total model calls.

For backward-compatible diagnostics, old-schema projections such as:

```text
pre_execution_orchestration_call_count
pre_execution_mixed_call_count
pre_execution_total_primary_call_count
```

may be retained, but they are no longer authoritative after the Execution Route Commit amendment.

Recommended QA-04 population/provenance fields:

```text
qa04_primary_observation_complete
qa04_primary_eligible
qa04_exclusion_reason
qa04_workload_class
model_call_schema_version
model_profile_version
prompt_profile_version
cache_policy_version
usage_weight
```

A complete QA-04 Primary observation requires a valid `request_processing_start_ts` and `execution_route_commit_ts` plus reconstructable route evidence.

Episodes that never commit a valid final route remain raw failure evidence and are not assigned an arbitrary high model-call penalty.

`episode_exact_conform` from QA-02 supports the Secondary slice `Calls per exact-conform episode`.

## Outcome fields

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `success` | boolean | yes | Whether Product/useful-outcome predicate was reached |
| `failure_reason` | string/null | yes | Stable failure class when unsuccessful |
| `useful_outcome_kind` | string/null | recommended | Observable predicate/outcome class reached |
| `cancel_requested` | boolean | recommended | Cancellation requested |
| `cancel_confirmed` | boolean/null | recommended | Execution stop confirmed when relevant |
| `recovery_path` | string/null | recommended | Restart/retry/fallback path if exercised |

`success` is not identical to QA-02 exact conformance. A scenario may reach an external effect while violating a required clarification/task-binding constraint.

## QA-02 constraint-manifest provenance

Required for QA-02 Architecture Qualification episodes:

| Field | Type | Meaning |
| --- | --- | --- |
| `constraint_manifest_version` | string | Exact Architecture Constraint Manifest version |
| `required_constraints` | array | Machine-evaluable required constraints or immutable references |
| `allowed_constraints` | array | Machine-evaluable allowed constraints or immutable references |
| `forbidden_constraints` | array | Machine-evaluable forbidden constraints or immutable references |

If only ids are stored, the referenced frozen manifest must remain resolvable.

## QA-02 canonical actual-decision fields

Every alternative normalizes implementation-specific state into a Canonical Architecture Decision Trace.

Minimum fields:

- `actual_referent_bindings`;
- `actual_task_relation`;
- `actual_task_id`;
- `actual_execution_owner`;
- `actual_execution_path`;
- `actual_delegated_agent`;
- `actual_clarification_action`;
- `actual_result_binding`;
- `actual_observable_effect`;
- `actual_compound_decomposition` when applicable.

These fields describe architecture outcomes, not component names.

## QA-02 per-dimension and exact conformance

Raw data retains per-dimension results using `null/not_applicable` where unconstrained:

- `referent_conform`;
- `task_association_conform`;
- `execution_path_conform`;
- `routing_conform`;
- `clarification_conform`;
- `result_binding_conform`;
- `compound_decomposition_conform`.

Required exact-conformance fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `episode_exact_conform` | boolean | True only when every applicable Required/Allowed/Forbidden constraint is satisfied |
| `constraint_results` | array | Per-constraint id, actual value, conformance, dimension, severity |
| `constraint_failure_ids` | array | Stable violated-constraint ids |
| `constraint_failure_reasons` | array/object | Structured failure evidence |

Only storing `episode_exact_conform` is prohibited.

Recommended population fields:

- `eligible_for_aecr`;
- `primary_correctness_category`;
- `secondary_correctness_categories`;
- `critical_slice_ids`;
- `usage_weight` for Secondary sensitivity only unless a frozen scoring version says otherwise.

## Event-oriented representation

The authoritative raw representation should preserve ordered events such as:

```text
RunStarted
AcousticEos
RequestProcessingStarted
SemanticRequestStarted
SemanticResponseReceived
ModelCallStarted
ModelCallCompleted
ArchitectureDecisionObserved
ExecutionPathSelected
ExecutionOwnerConfirmed
AgentDispatched
ExecutionStarted
ExecutionRouteCommitted
ToolStarted
ToolCompleted
ClarificationRequested
TaskAssociationObserved
ResultBound
UserVisibleResultStarted
UsefulOutcomeObserved
ConstraintEvaluated
TaskCompleted
RunFinished
```

Each event should contain:

- `run_id` and `episode_id`;
- monotonic timestamp where meaningful;
- event type;
- relevant stable ids;
- structured attributes.

Benefits:

- multiple Agent/tool/model invocations remain reconstructable;
- escalation and hidden-routing paths can be reconstructed;
- retries/cancellation/progress can be analyzed later;
- clarification and follow-up state can span turns;
- QA-04 route-commit boundary can be independently audited;
- later derived metrics can be recomputed from immutable evidence.

Detailed ModelCall attributes belong to `benchmark/schemas/model-call-schema.md`; runtime events may reference `model_call_id` rather than duplicate telemetry.

## Suggested event attributes

### `RequestProcessingStarted`

- episode/user-goal id;
- user turn id;
- input modality;
- whether processing begins from partial/finalized input;
- semantic trace/run-plan reference.

### `ModelCallStarted` / `ModelCallCompleted`

- `model_call_id`;
- owner/component;
- purpose;
- call class;
- retry/parent references;
- route-commit state/inclusion flag;
- model/profile version references.

### `ArchitectureDecisionObserved`

- decision dimension;
- canonical actual value;
- candidate/rejected values if available;
- validation reason code;
- associated user turn/task ids.

### `ExecutionPathSelected`

- path/candidate owner;
- selection reason;
- confidence/ambiguity class;
- initial vs escalation.

### `ExecutionOwnerConfirmed`

- owner;
- confirmation mechanism;
- task/work/execution-attempt id;
- related model/dispatch call id.

### `AgentDispatched`

- Agent id;
- harness/adapter type;
- task/work id;
- parent id;
- dispatch attempt.

### `ExecutionStarted`

- current owner;
- execution-attempt id;
- local/Agent/ARGO start evidence;
- dispatch/accept correlation.

### `ExecutionRouteCommitted`

- `execution_route`;
- `final_execution_owner`;
- `delegated_agent_if_any`;
- route-commit reason/mechanism;
- call/dispatch id that caused commitment;
- whether an intermediate owner had already started.

### `ClarificationRequested`

- ambiguity/constraint id;
- clarification dimension;
- logical episode id;
- request/answer turn ids.

### `ResultBound`

- result/progress id;
- logical task/work id;
- episode id;
- source execution owner/Agent.

### `ConstraintEvaluated`

- manifest version;
- constraint id/dimension;
- actual value;
- required/allowed/forbidden reference;
- conform boolean;
- failure reason.

### `ToolStarted` / `ToolCompleted`

- action/tool class;
- controlled latency profile id;
- execution owner;
- side-effect/oracle target;
- result/failure class.

### `UsefulOutcomeObserved`

- success predicate id;
- observed state;
- delivery channel/surface when relevant.

## Acknowledgment vs useful result

If the product acknowledges before useful completion, record acknowledgment separately:

- `acknowledgement_started_ts`;
- `acknowledgement_completed_ts`.

A generic “working on it” is diagnostic interaction latency, not QA-01 completion.

## Progress events

For future task-lifecycle QAs, retain progress with:

- task/work id;
- timestamp;
- source owner/Agent;
- progress kind/status;
- delivery channel;
- generated/queued/delivered/acknowledged state.

Do not reduce progress to only a final count in raw storage.

## Task, session and episode correlation

Recommended ids:

- evaluation episode id;
- conversation id;
- user turn id;
- task/work id;
- parent task/work id;
- downstream session/thread id (hashed/redacted if needed);
- Agent id;
- execution-attempt id;
- clarification-chain id;
- model-call id.

These support QA-02 task/result binding, QA-04 causal inclusion and later recovery/cancellation analysis.

## Privacy / secret handling

Raw benchmark data must not contain credentials, API keys, auth cookies or unrelated personal information.

Prefer synthetic fixtures/stable ids. Normalize/redact sensitive fidelity traces before freezing them for qualification.

## Derived QA-01 calculation

For each successful Fast-task episode:

```text
FTOL_i = useful_outcome_ts - acoustic_eos_ts
```

Then compute:

- FTOL p50;
- **FTOL p95 — QA-01 Primary Metric**;
- FTOL p99;
- F1/F2/F3 distributions;
- sensitivity to controlled dependency-latency profile.

Failure episodes remain raw evidence but are not transformed into fake high latency.

## Derived QA-02 calculation

For the frozen eligible QA-02 population:

```text
AECR = count(episode_exact_conform == true)
       -------------------------------------- × 100
       count(eligible_for_aecr == true)
```

Also derive category/slice correctness diagnostics from the same raw records.

## Derived QA-04 calculation

For each complete/eligible QA-04 episode `i`:

```text
RouteCommitPrimaryCalls_i =
  count(ModelCall where
        included_in_qa04_primary == true
        AND episode_id == i)
```

Equivalent projection:

```text
RouteCommitPrimaryCalls_i =
  route_commit_orchestration_call_count
  + route_commit_mixed_call_count
```

Then:

```text
QA-04 = mean(RouteCommitPrimaryCalls_i)
        over the frozen eligible/completed QA-04 population
```

Also derive:

- total model calls;
- ORCHESTRATION/MIXED/DOMAIN counts;
- critical-path calls;
- retry count;
- calls by purpose;
- request-class distributions;
- Calls per exact-conform episode;
- token/cache/context/resource/cost diagnostics.

## Validation rules

A benchmark runner should reject or flag a run when:

- `useful_outcome_ts < acoustic_eos_ts`;
- timestamps use inconsistent clock domains;
- required identity/version fields are missing;
- `success=true` but no useful-outcome evidence exists;
- alternative/scenario/trace identifiers do not match the frozen run plan;
- a QA-02 eligible episode lacks a resolvable constraint manifest/version;
- `episode_exact_conform=true` while an applicable constraint is false;
- constraint failure lacks stable id/reason/actual evidence;
- QA-01 qualification references an unfrozen/unknown dependency-latency profile;
- a QA-04 complete Primary observation lacks `request_processing_start_ts` or `execution_route_commit_ts`;
- `execution_route_commit_ts < request_processing_start_ts`;
- final route/owner evidence is inconsistent with `ExecutionRouteCommitted` events;
- episode-level QA-04 counts cannot be reproduced from ModelCall records;
- an ORCHESTRATION/MIXED specialist-delegation call is excluded only because an intermediate `execution_started_ts` occurred earlier;
- model/prompt/cache profile versions do not match the frozen QA-04 run plan;
- final qualification uses unfrozen scoring/benchmark/taxonomy/corpus versions.

## Schema evolution

Schema changes are versioned.

Backward-compatible additions may add optional events/attributes. Semantic changes to existing fields require a new schema version and must not rewrite historical `results/raw/` data.

See also:

- `benchmark/schemas/scenario-constraint-schema.md` for QA-02 oracle semantics;
- `benchmark/schemas/model-call-schema.md` for QA-04 per-logical-generation telemetry;
- `benchmark/schemas/evolution-scenario-schema.md` and `benchmark/schemas/evolution-run-schema.md` for QA-03 evolution evidence.
