# Architecture Benchmark Raw Run Event Contract

## Status

Architecture Context Checkpoint 001/002 — logical raw experiment schema.

This is a logical event schema for benchmark implementation. It intentionally captures more than one QA needs so QA-01~QA-04 and future Secondary Metrics can be derived without rerunning experiments solely because an intermediate observation was discarded.

> **측정하지 않은 값은 나중에 복구할 수 없지만, raw data로 보존한 값은 나중에 다른 metric으로 재해석할 수 있다.**

## Storage policy

```text
results/raw/      immutable run events
results/derived/  recomputable metrics / aggregates
results/reports/  human-readable summaries and visualizations
```

Raw records are append-only/immutable experimental evidence. If instrumentation or schema meaning changes, increment a schema/benchmark version and create new runs rather than rewriting old events.

Derived metrics such as FTOL p95 and AECR must never be the only persisted evidence.

## Time basis

Latency metrics must use a **monotonic clock**.

Wall-clock time may additionally be logged for operational correlation, but must not be used to calculate sub-run latency because system-clock adjustments can invalidate intervals.

Each run records a monotonic origin and all event timestamps use the same origin/unit.

Recommended unit: integer nanoseconds or microseconds from monotonic origin. The concrete serialization format will be finalized with benchmark implementation.

## Minimum run identity fields

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `schema_version` | string | yes | Raw event contract version |
| `run_id` | string | yes | Globally unique benchmark episode/run identity |
| `scenario_id` | string | yes | Stable scenario identifier |
| `scenario_category` | string | yes | e.g. QA-01 `F1`/`F2`/`F3`, QA-02 `C1`~`C8` |
| `scenario_version` | string | recommended | Scenario definition version |
| `scenario_corpus_version` | string | yes | Frozen scoring-corpus version |
| `scenario_taxonomy_version` | string/null | recommended | Taxonomy version where applicable |
| `alternative_id` | string | yes | Architecture alternative under test, e.g. `DP00-A` |
| `alternative_version` | string/null | recommended | Prototype/architecture implementation version |
| `benchmark_version` | string | yes | Runner/corpus contract version |
| `source_git_commit` | string | yes | Source commit being evaluated |
| `oracle_trace_id` | string/null | yes | Frozen semantic/oracle trace; null only when not applicable |
| `semantic_trace_class` | string/null | recommended | e.g. correct, ambiguous, wrong-fast, wrong-agent, malformed, timeout |
| `scoring_version` | string/null | yes | Frozen scoring version used later; may be null at raw-run time |
| `monotonic_clock_origin` | string/object | yes | Description/identifier of timestamp origin and unit |

Additional recommended provenance:

- Agent-stub version;
- tool-latency profile version;
- machine profile id;
- OS/build;
- power profile;
- warm/cold state;
- actual-model fidelity-run flag.

## Core QA-01 timestamps

All timestamps are monotonic offsets from the run origin and are nullable when not applicable.

| Field | Meaning |
| --- | --- |
| `acoustic_eos_ts` | Ground-truth Acoustic End-of-Speech mapped from fixture annotation into run time. FTOL start. |
| `semantic_request_start_ts` | Architecture requests semantic interpretation/replay result. |
| `semantic_response_ts` | Semantic dependency result becomes available to architecture. |
| `execution_path_selected_ts` | Architecture commits to the execution path/owner for this request. |
| `agent_dispatch_ts` | Request crosses the Downstream Agent dispatch boundary. |
| `tool_start_ts` | Controlled local/Agent action begins, when observable at benchmark seam. |
| `tool_complete_ts` | Controlled action reaches its completion state. |
| `first_user_visible_result_ts` | First meaningful user-visible result delivery begins. Not a generic acknowledgment unless it satisfies the scenario oracle. |
| `useful_outcome_ts` | First observable state satisfying the scenario success predicate. FTOL endpoint. |
| `task_complete_ts` | Architecture marks the logical task/workflow complete. May differ from useful outcome. |

Do not fabricate timestamps for paths that do not have the corresponding stage. A direct local path may have no `agent_dispatch_ts`; an answer-only path may have no `tool_start_ts`.

## Execution topology fields

| Field | Type | Meaning |
| --- | --- | --- |
| `execution_owner` | string | Authority selected for substantive execution, e.g. `via_fast_path`, `argo`, `specialized_agent:<id>` |
| `execution_path` | array/string | Ordered ownership/handoff path, e.g. `via_fast_path`, `via->argo`, `via->argo->agent_x` |
| `handoff_count` | integer | Number of ownership-boundary crossings |
| `model_invocation_count` | integer | Count of model invocations observable/defined by benchmark contract |
| `agent_invocation_count` | integer | Downstream Agent prompt/turn invocations if separately measurable |
| `tool_invocation_count` | integer | Controlled tool/action invocations if applicable |

`model_invocation_count` requires a benchmark definition of what constitutes an invocation for each dependency type. Streaming tokens/events inside one model response are not automatically separate invocations.

## Outcome fields

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `success` | boolean | yes | Whether scenario Product/useful-outcome predicate was reached |
| `failure_reason` | string/null | yes | Stable failure class when unsuccessful |
| `useful_outcome_kind` | string/null | recommended | Observable predicate/outcome class reached |
| `final_execution_owner` | string/null | recommended | Owner at useful outcome/task completion |
| `cancel_requested` | boolean | recommended | Cancellation was requested in this episode |
| `cancel_confirmed` | boolean/null | recommended | Execution stop was confirmed when relevant |
| `recovery_path` | string/null | recommended | Restart/retry/fallback path if exercised |

`success` is not identical to QA-02 exact conformance. A scenario may accidentally reach a useful external effect while violating a required clarification/task-binding constraint.

## QA-02 constraint-manifest provenance

The following fields are required for QA-02 Architecture Qualification episodes:

| Field | Type | Meaning |
| --- | --- | --- |
| `constraint_manifest_version` | string | Exact Architecture Constraint Manifest version |
| `required_constraints` | array | Machine-evaluable required constraints or stable ids plus frozen manifest reference |
| `allowed_constraints` | array | Machine-evaluable allowed constraints or stable ids plus frozen manifest reference |
| `forbidden_constraints` | array | Machine-evaluable forbidden constraints or stable ids plus frozen manifest reference |

If the raw record stores only ids, the referenced immutable/frozen manifest artifact must be resolvable by version. A final report alone is not sufficient provenance.

## QA-02 canonical actual-decision fields

Every alternative normalizes its implementation-specific state into a Canonical Architecture Decision Trace.

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

These fields must describe common architecture outcomes, not require internal component names such as `router.selected_agent`.

## QA-02 per-dimension conformance

Raw data must retain per-dimension results, using `null/not_applicable` when the dimension was not constrained in the scenario:

- `referent_conform`;
- `task_association_conform`;
- `execution_path_conform`;
- `routing_conform`;
- `clarification_conform`;
- `result_binding_conform`;
- `compound_decomposition_conform`.

Additional dimensions may be added in later schema versions without deleting historical evidence.

## QA-02 exact-conformance fields

| Field | Type | Meaning |
| --- | --- | --- |
| `episode_exact_conform` | boolean | True only when every applicable Required/Allowed/Forbidden constraint is satisfied |
| `constraint_results` | array | Per-constraint id, actual value, conform boolean, dimension, severity |
| `constraint_failure_ids` | array | Stable ids of violated constraints |
| `constraint_failure_reasons` | array/object | Structured reasons/actual-vs-expected evidence |

Only storing `episode_exact_conform` is prohibited. AECR is calculated later from eligible episodes and must be recomputable from raw evidence.

## QA-02 scenario population fields

Recommended/required fields for population sensitivity and corpus freeze:

- `eligible_for_aecr`;
- `primary_correctness_category` (`C1`~`C8`);
- `secondary_correctness_categories`;
- `critical_slice_ids`;
- `usage_weight` if a separate production-frequency sensitivity analysis is planned.

`usage_weight` must not silently affect the coverage-balanced Primary AECR unless the frozen scoring version explicitly says so.

## Event-oriented representation

The minimum fields above can be stored as one episode record, but the implementation should strongly consider an event log as the authoritative raw representation:

```text
RunStarted
AcousticEos
SemanticRequestStarted
SemanticResponseReceived
ArchitectureDecisionObserved
ExecutionPathSelected
AgentDispatched
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

- `run_id`;
- monotonic timestamp where meaningful;
- event type;
- relevant stable ids;
- structured attributes.

A derived episode table can then project the first/last timestamp and final canonical decision values.

Benefits:

- multiple Agent/tool invocations are retained rather than flattened;
- escalation paths can be reconstructed;
- retries/cancellation/progress can be analyzed later;
- clarification and follow-up state can span multiple turns;
- a new derived metric does not require changing old raw records if its source events were captured.

## Suggested event attributes

### `ArchitectureDecisionObserved`

- decision dimension;
- canonical actual value;
- candidate/rejected values if available;
- validation reason code;
- associated user turn/task ids.

### `ExecutionPathSelected`

- path id;
- selected owner;
- candidate owners if available;
- selection reason code;
- confidence/ambiguity class from frozen semantic trace;
- whether this is initial selection or directed escalation.

### `AgentDispatched`

- Agent id;
- harness/adapter type;
- task/work id;
- parent task/work id;
- dispatch attempt number.

### `ClarificationRequested`

- ambiguity/constraint id;
- clarification target/dimension;
- logical episode id;
- request/answer turn ids when resolved.

### `ResultBound`

- result/progress id;
- logical task/work id;
- user-goal/episode id;
- source execution owner/Agent.

### `ConstraintEvaluated`

- constraint manifest version;
- constraint id;
- dimension;
- actual value;
- expected/allowed/forbidden reference;
- conform boolean;
- failure reason code.

### `ToolStarted` / `ToolCompleted`

- action/tool class;
- controlled latency profile id;
- execution owner;
- side-effect/oracle target id;
- result/failure class.

### `UsefulOutcomeObserved`

- success predicate id;
- observed state identifier;
- channel/surface if delivery is part of the predicate.

## Acknowledgment vs useful result

If the product speaks or renders an acknowledgment before completing the task, record it separately rather than overloading `first_user_visible_result_ts` or `useful_outcome_ts`.

Recommended optional events/fields:

- `acknowledgement_started_ts`;
- `acknowledgement_completed_ts`.

A generic “working on it” response is diagnostic interaction latency, not QA-01 completion.

## Progress events

For future long-running/task-lifecycle QAs, raw event logs should retain progress events with:

- task/work id;
- timestamp;
- source owner/Agent;
- progress kind/status;
- delivery channel;
- whether progress was generated, queued, delivered and acknowledged.

Do not reduce progress to only the final count in raw storage.

## Task, session and episode correlation

Recommended correlation ids where applicable:

- logical evaluation episode id;
- logical conversation id;
- user turn id;
- task/work id;
- parent task/work id;
- downstream session/thread id (hashed/redacted if necessary);
- Agent id;
- execution-attempt id;
- clarification-chain id.

These identifiers are essential for QA-02 task association/result binding and later QAs on retries, cancellation and recovery.

## Privacy / secret handling

Raw benchmark data must not contain credentials, API keys, auth cookies or unrelated personal information.

Prefer stable scenario identifiers and synthetic benchmark fixtures. If real-model fidelity traces contain sensitive content, normalize/redact them before turning them into the frozen qualification corpus.

## Derived QA-01 calculation

For each successful Fast-task episode:

```text
FTOL_i = useful_outcome_ts - acoustic_eos_ts
```

Then compute:

- FTOL p50;
- **FTOL p95 — QA-01 Primary Metric**;
- FTOL p99;
- category-specific F1/F2/F3 distributions.

Failure episodes remain in the raw population with `success=false` but are not converted into fake high-latency samples.

## Derived QA-02 calculation

For the frozen population of eligible QA-02 episodes:

```text
AECR = count(episode_exact_conform == true)
       -------------------------------------- × 100
       count(eligible_for_aecr == true)
```

Also derive diagnostic metrics from the same raw records:

- category-specific AECR;
- Referent Binding Accuracy;
- Task Association Accuracy;
- Execution-path Conformance;
- Agent Routing Conformance;
- Clarification Correctness;
- Result Binding Accuracy;
- Compound Decomposition Accuracy;
- False Fast-path Rate;
- Unnecessary Delegation Rate;
- Constraint-level Macro Average;
- Critical-slice Exact Conformance;
- usage-weighted sensitivity results when weights are provided.

## Validation rules

A benchmark runner should reject or flag a run when:

- `useful_outcome_ts < acoustic_eos_ts`;
- timestamps use inconsistent clock domains;
- required identity/version fields are missing;
- `success=true` but no useful-outcome evidence exists;
- alternative/scenario/trace identifiers do not match the frozen run plan;
- a QA-02 eligible episode has no resolvable constraint manifest/version;
- `episode_exact_conform=true` while any applicable constraint result is false;
- a constraint failure exists without a stable id/reason/actual evidence;
- final qualification uses an unfrozen scoring/benchmark/taxonomy/corpus version.

## Schema evolution

Schema changes are versioned.

Backward-compatible additions may add optional events/attributes. Semantic changes to existing fields require a new schema version and must not rewrite historical `results/raw/` data.

The implementation format (JSONL, structured JSON, Parquet-derived projection, etc.) remains an implementation decision. The semantic contract above is the checkpoint requirement.

See also `benchmark/schemas/scenario-constraint-schema.md` for QA-02 oracle semantics.
