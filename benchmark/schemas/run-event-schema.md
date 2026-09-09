# Architecture Benchmark Raw Run Event Contract

## Status

Architecture Context Checkpoint 001 — initial schema contract.

This is a logical event schema for benchmark implementation. It intentionally captures more than QA-01 needs so QA-02~QA-04 and future Secondary Metrics can be derived without rerunning experiments solely because an intermediate observation was discarded.

> **측정하지 않은 값은 나중에 복구할 수 없지만, raw data로 보존한 값은 나중에 다른 metric으로 재해석할 수 있다.**

## Storage policy

```text
results/raw/      immutable run events
results/derived/  recomputable metrics / aggregates
results/reports/  human-readable summaries and visualizations
```

Raw records are append-only/immutable experimental evidence. If instrumentation or schema meaning changes, increment a schema/benchmark version and create new runs rather than rewriting old events.

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
| `scenario_category` | string | yes | e.g. `F1`, `F2`, `F3`, later QA categories |
| `alternative_id` | string | yes | Architecture alternative under test, e.g. `DP00-A` |
| `benchmark_version` | string | yes | Runner/corpus contract version |
| `source_git_commit` | string | yes | Source commit being evaluated |
| `oracle_trace_id` | string/null | yes | Frozen semantic/oracle trace; null only when not applicable |
| `scoring_version` | string/null | yes | Frozen scoring version used later; may be null at raw-run time |
| `monotonic_clock_origin` | string/object | yes | Description/identifier of timestamp origin and unit |

Additional recommended provenance:

- scenario corpus version;
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
| `success` | boolean | yes | Whether scenario success predicate was reached |
| `failure_reason` | string/null | yes | Stable failure class when unsuccessful |
| `useful_outcome_kind` | string/null | recommended | Observable predicate/outcome class reached |
| `final_execution_owner` | string/null | recommended | Owner at useful outcome/task completion |
| `cancel_requested` | boolean | recommended | Cancellation was requested in this episode |
| `cancel_confirmed` | boolean/null | recommended | Execution stop was confirmed when relevant |
| `recovery_path` | string/null | recommended | Restart/retry/fallback path if exercised |

## QA-02 correctness reservation

QA-02 is not yet formally defined, but the raw contract should reserve fields so correctness can be assessed without redesigning the event format.

Candidate fields:

- `expected_outcome` / `expected_outcome_id`;
- `actual_outcome` / `actual_outcome_id`;
- `expected_execution_owner` when scenario semantics define one;
- `actual_execution_owner`;
- `expected_agent_id`;
- `actual_agent_id`;
- `semantic_trace_class`;
- `correctness_predicate_id`;
- `correctness_result`;
- structured mismatch/failure classification.

Exact fields and semantics will be frozen when QA-02 is defined. Implementations should prefer extensible structured metadata over an unparseable prose-only result.

## Event-oriented representation

The minimum fields above can be stored as one episode record, but the implementation should strongly consider an event log as the authoritative raw representation:

```text
RunStarted
AcousticEos
SemanticRequestStarted
SemanticResponseReceived
ExecutionPathSelected
AgentDispatched
ToolStarted
ToolCompleted
UserVisibleResultStarted
UsefulOutcomeObserved
TaskCompleted
RunFinished
```

Each event should contain:

- `run_id`;
- monotonic timestamp;
- event type;
- relevant stable ids;
- structured attributes.

A derived episode table can then project the first/last timestamp of each event type.

Benefits:

- multiple Agent/tool invocations are retained rather than flattened;
- escalation paths can be reconstructed;
- retries/cancellation/progress can be analyzed later;
- a new derived metric does not require changing old raw records if its source events were captured.

## Suggested event attributes

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

## Task and session correlation

Recommended correlation ids where applicable:

- logical conversation id;
- user turn id;
- task/work id;
- parent task/work id;
- downstream session/thread id (hashed/redacted if necessary);
- Agent id;
- execution-attempt id.

These identifiers are essential for later QA work on existing-task association, retries, cancellation and recovery.

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

## Validation rules

A benchmark runner should reject or flag a run when:

- `useful_outcome_ts < acoustic_eos_ts`;
- timestamps use inconsistent clock domains;
- required identity/version fields are missing;
- `success=true` but no useful-outcome evidence exists;
- alternative/scenario/trace identifiers do not match the frozen run plan;
- a final qualification run uses an unfrozen scoring/benchmark version.

## Schema evolution

Schema changes are versioned.

Backward-compatible additions may add optional events/attributes. Semantic changes to existing fields require a new schema version and must not rewrite historical `results/raw/` data.

The implementation format (JSONL, structured JSON, Parquet-derived projection, etc.) remains an implementation decision. The semantic contract above is the checkpoint requirement.