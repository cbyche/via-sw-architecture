# Canonical Benchmark Observation Event Contract

## Status

**Canonical Benchmark Contract Checkpoint — architecture-neutral runtime observation contract**

This contract defines the event vocabulary used to observe DP-00 Runtime Episodes without requiring any alternative to expose the same internal component graph.

It complements, rather than replaces, `benchmark/schemas/run-event-schema.md` and `benchmark/schemas/model-call-schema.md`.

The canonical layer normalizes benchmark-relevant observations. Alternative-specific debug telemetry may exist separately.

---

# 1. Core principle

The benchmark observes **architecture outcomes and lifecycle boundaries**, not internal component names.

Forbidden as canonical requirements:

```text
router.selected_agent
intent_refiner.completed
argo.delegate.called
fast_path_classifier.returned
```

Those may be useful debug events, but the shared evaluator must not require them.

Canonical observations describe externally comparable semantics such as:

- input readiness;
- model generation;
- clarification;
- task create/reuse;
- execution route commit;
- execution dispatch/accept/start;
- consent;
- progress/result binding;
- externally observed useful outcome;
- cancellation;
- episode completion/failure.

---

# 2. Canonical event envelope

Minimum fields:

```text
schema_version
event_id
event_type

run_id
episode_id
scenario_id
alternative_id

sequence_number
monotonic_timestamp

emitter
causation_event_id
correlation_id

task_id          # nullable when not applicable
payload
```

Recommended provenance:

```text
scenario_version
benchmark_version
source_git_commit
canonical_event_schema_version
```

### `sequence_number`

Monotonically increasing within the run/event stream. It is diagnostic ordering evidence and does not replace monotonic timestamps.

### `causation_event_id`

Links the immediate event that caused this event where known.

### `correlation_id`

Groups a logical operation such as one ModelCall, one dispatch attempt, one clarification chain, one result, or one cancellation propagation.

---

# 3. Emitter vocabulary

Initial emitter values:

```text
BENCHMARK
ARCHITECTURE_UNDER_TEST
MODEL_FIXTURE
AGENT_FIXTURE
TOOL_FIXTURE
OUTCOME_PROBE
INTERACTION_FIXTURE
```

The emitter allows the evaluator to distinguish:

- benchmark-generated timing/fixture evidence;
- AUT-reported architecture decisions;
- dependency-fixture lifecycle;
- independent external outcome observations.

The emitter field does not imply trust by itself. Certain event types have an authoritative producer defined below.

---

# 4. Canonical event vocabulary

Use dotted names in the implementation if convenient; the semantic names below are authoritative.

```text
interaction.acoustic_eos
interaction.input_ready
interaction.processing_started

model.generation.started
model.generation.first_output
model.generation.completed
model.generation.failed

clarification.requested
clarification.resolved

task.created
task.reused

execution.route_candidate_observed
execution.route_committed
execution.dispatched
execution.accepted
execution.started

consent.requested
consent.resolved

progress.observed
result.bound
useful_outcome.observed

cancel.requested
cancel.propagated
cancel.confirmed

episode.completed
episode.failed
```

Existing run-event names such as `AcousticEos`, `ModelCallStarted`, `ExecutionRouteCommitted` may remain in raw implementation. The adapter must map them unambiguously to these canonical semantics rather than proliferating competing meanings.

---

# 5. Authoritative producer rules

Not every event may be self-reported by the AUT for scoring.

| Event | Authoritative producer/source |
| --- | --- |
| `interaction.acoustic_eos` | `INTERACTION_FIXTURE`, sourced from the benchmark audio annotation (`BENCHMARK`) |
| `interaction.input_ready` | `INTERACTION_FIXTURE` or runner |
| `interaction.processing_started` | AUT instrumentation |
| model generation events | Model adapter / `MODEL_FIXTURE` plus ModelCall identity |
| clarification events | AUT interaction output observed by runner |
| task create/reuse | AUT instrumentation normalized to canonical task semantics |
| `execution.route_committed` | AUT decision boundary event, validated against subsequent dispatch/route evidence |
| dispatch/accepted/started | AUT + Agent/Tool fixture correlation |
| consent events | AUT interaction/policy boundary observations |
| progress | Agent/AUT/result-channel observations |
| `result.bound` | AUT binding event with task/result correlation |
| `useful_outcome.observed` | **OUTCOME_PROBE whenever feasible** |
| cancellation propagation | AUT + dependency fixture correlation |
| episode completed/failed | benchmark evaluator based on termination contract |

This distinction prevents a self-reporting architecture from manufacturing lower latency or false success.

---

# 6. Interaction events

## `interaction.acoustic_eos`

Payload:

```text
turn_id
audio_fixture_ref
ground_truth_acoustic_eos_offset
```

The timestamp is benchmark-authoritative and feeds QA-01 FTOL start.

It is not supplied to AUT semantic logic.

## `interaction.input_ready`

Indicates the canonical user-turn/transcript handoff from the controlled interaction front-end to the AUT.

Payload:

```text
turn_id
modality
input_fixture_ref
```

For DP-00 Base evaluation, pre-EOS semantic optimization is not enabled.

## `interaction.processing_started`

Maps to QA-04 `request_processing_start_ts` when applicable.

Payload:

```text
turn_id
episode_id
```

---

# 7. Model generation events

All model generation events reference `model_call_id`.

Minimum payload:

```text
model_call_id
decision_owner
semantic_operation_keys[]
call_class
```

Additional ModelCall details remain in `model-call-schema.md`.

One logical generation produces one logical ModelCall even if it streams many chunks or serves multiple semantic operation keys.

---

# 8. Clarification events

## `clarification.requested`

Payload:

```text
clarification_id
turn_id
clarification_dimension
prompt_output_ref
```

The evaluator uses this event to decide whether a scripted deterministic user reply should be released.

The runner does not expose a prior `clarification_required` flag to the AUT.

## `clarification.resolved`

Payload:

```text
clarification_id
request_turn_id
response_turn_id
```

---

# 9. Task events

## `task.created`

Payload:

```text
task_id
user_goal_id
execution_mapping_ref_if_available
```

## `task.reused`

Used for clear continuation/follow-up semantics.

Payload:

```text
task_id
prior_execution_route_ref
continuation_ref
```

The benchmark does not generate `task.reused` on behalf of the AUT. It observes the architecture's task-association outcome.

---

# 10. Canonical ExecutionRoute

Canonical benchmark representation:

```text
ExecutionRoute
  route_kind
  initial_executor_id
  final_executor_id_if_known
  delegation_chain[]
```

Allowed `route_kind` vocabulary:

```text
LOCAL_DIRECT
EXECUTOR_DIRECT
EXECUTOR_DELEGATED
```

Do not use A/B/C/D-specific route kinds such as `ARGO_PRIMARY` or `SPECIALIST_DIRECT` in the canonical schema.

Examples:

```text
C local volume:
  LOCAL_DIRECT
  initial_executor_id = VIA_LOCAL_VOLUME

A direct NetworkAgent:
  EXECUTOR_DIRECT
  initial_executor_id = NetworkAgent

B ARGO direct:
  EXECUTOR_DIRECT
  initial_executor_id = ARGO

B ARGO -> NetworkAgent:
  EXECUTOR_DELEGATED
  initial_executor_id = ARGO
  final_executor_id_if_known = NetworkAgent
  delegation_chain = [ARGO, NetworkAgent]
```

---

# 11. Route events

## `execution.route_candidate_observed`

Optional diagnostic event for a provisional candidate.

It must not be interpreted as QA-04 Execution Route Commit.

Payload:

```text
candidate_route
reason_ref
provisional = true
```

## `execution.route_committed`

Authoritative canonical meaning:

> **user request의 initial execution-routing/delegation 판단이 완료되어, 해당 request를 실제로 수행할 초기 execution path가 확정된 최초 시점.**

Payload:

```text
execution_route
route_commit_reason
causing_model_call_id_if_any
causing_dispatch_id_if_any
intermediate_owner_started_before_commit
```

Rules:

- candidate selection alone is not commit;
- synchronous dispatch rejection that requires choosing another initial route means the rejected candidate was not the final commit;
- initial ARGO→specialist delegation required to establish the route remains before/at this event;
- post-commit domain subtask delegation does not create a second QA-04 initial route commit.

## `execution.dispatched`

Payload:

```text
dispatch_id
executor_id
parent_executor_id_if_delegated
execution_route_ref
```

## `execution.accepted`

Payload:

```text
dispatch_id
executor_id
acceptance_status
```

## `execution.started`

Payload:

```text
execution_id
executor_id
dispatch_id_if_any
```

These remain useful diagnostics even though QA-04 uses route commit as its authoritative end boundary.

---

# 12. Result and progress events

## `progress.observed`

Payload:

```text
progress_id
source_executor_id
task_id
progress_kind
status
```

## `result.bound`

Payload:

```text
result_id
source_executor_id
task_id
user_goal_id
binding_reason
```

QA-02 may compare this against evaluator-only expected result-binding constraints.

---

# 13. Useful Outcome authority

QA-01 must not rely on a bare AUT self-report such as:

```text
useful_outcome = true
```

Whenever practical, `useful_outcome.observed` is emitted by an `OUTCOME_PROBE` that observes the controlled execution sink.

Examples:

```text
Volume:
  Tool Fixture changes observable volume state
  -> Outcome Probe confirms requested state

File Open:
  controlled UI/file fixture enters usable/open state
  -> Outcome Probe confirms target

Short answer:
  interaction output sink sees first meaningful result delivery
  -> Outcome Probe marks first meaningful result start
```

Payload:

```text
success_predicate_id
observed_state_ref
outcome_kind
channel_or_surface_if_relevant
```

The AUT may emit diagnostic completion events, but the evaluator uses the authoritative probe timestamp when available.

---

# 14. Cancellation events

Canonical lifecycle:

```text
cancel.requested
  -> cancel.propagated
  -> cancel.confirmed
```

Payloads preserve task/execution ids and source/target owner.

Mandatory cancellation semantics are not converted into a compensable Top-QA score unless a future approved evaluation rule says otherwise.

---

# 15. Episode termination events

## `episode.completed`

Emitted by evaluator when the scenario's termination/success contract is satisfied or the episode is otherwise complete under the run plan.

## `episode.failed`

Payload:

```text
failure_class
failure_reason
task_id_if_any
route_commit_observed
```

Failure does not imply a fabricated latency or call-count penalty; each QA applies its own authoritative population rule.

---

# 16. Event provenance and trust

Raw events should preserve both:

```text
emitter
source_evidence_ref
```

where `source_evidence_ref` can reference:

- fixture observation;
- ModelCall record;
- Agent script event;
- Tool state change;
- AUT instrumentation trace;
- evaluator constraint result.

This allows later audit of whether a derived metric depended on self-report or external observation.

---

# 17. Relationship to existing run-event schema

`benchmark/schemas/run-event-schema.md` remains the central raw runtime evidence contract.

This canonical schema adds two constraints:

1. benchmark-facing event semantics are architecture-neutral;
2. authoritative producer/source is explicit for events used in scoring.

Implementations may map existing event names as follows:

```text
AcousticEos              -> interaction.acoustic_eos
RequestProcessingStarted -> interaction.processing_started
ModelCallStarted         -> model.generation.started
ModelCallCompleted       -> model.generation.completed
ClarificationRequested   -> clarification.requested
ExecutionRouteCommitted  -> execution.route_committed
AgentDispatched          -> execution.dispatched
ExecutionStarted         -> execution.started
ResultBound              -> result.bound
UsefulOutcomeObserved    -> useful_outcome.observed
TaskCompleted            -> episode/task completion projection
```

The next Prototype/Harness Specification should pick one implementation naming convention and publish the mapping.

---

# 18. Validation rules

Reject/flag benchmark evidence when:

- a canonical event requires an internal component name to be meaningful;
- `useful_outcome.observed` used for QA-01 is based only on AUT self-report when an external probe exists;
- the AUT emits `interaction.acoustic_eos` as its own timing authority;
- `execution.route_committed` is emitted for a provisional candidate that is synchronously rejected before execution can proceed;
- a post-commit domain subtask delegation overwrites the initial route-commit timestamp;
- event ordering/causation is internally inconsistent;
- `scenario_id` or evaluator-only metadata appears in a decision payload exposed to AUT logic;
- canonical route uses topology-specific enum values that cannot represent all alternatives.

---

# 19. Versioning

Canonical event semantics are versioned.

Changing the meaning of a scoring-relevant event such as `execution.route_committed` or `useful_outcome.observed` requires a new schema/benchmark version and must not rewrite historical raw events.

## 19.1 `canonical-event-v1` semantic evidence payloads

`canonical-event-v1` adds actual-value evidence needed to project a QA-02 Canonical Architecture Decision Trace without consulting a scenario, behavior plan, filename, ordering convention, or oracle:

| Canonical event | Required actual value | Authority |
| --- | --- | --- |
| `referent.bound` | `referent_role`, `resolved_referent_id`, product `turn_id` correlation | `ARCHITECTURE_UNDER_TEST` |
| `task.associated` | `task_relation`, correlated `task_id` | `ARCHITECTURE_UNDER_TEST` task boundary |
| `execution.route_committed` | `route_kind`, `initial_executor_id`, `final_executor_id_if_known`, `delegation_chain[]` | `ARCHITECTURE_UNDER_TEST` route-commit boundary |
| `clarification.requested` | architecture-owned reason plus request-turn correlation | `ARCHITECTURE_UNDER_TEST` |
| `clarification.resolved` | `request_turn_id`, `response_turn_id` | `ARCHITECTURE_UNDER_TEST` |
| `result.bound` | correlated `result_id`, `task_id`, `execution_id` | `ARCHITECTURE_UNDER_TEST` product binding boundary |
| `useful_outcome.observed` | typed effect subject/target/value/state/executor and `authoritative_source` | `OUTCOME_PROBE` |
| `episode.failed` | topology-neutral `failure_reason` | benchmark runtime lifecycle |

The failure vocabulary is `MODEL_MALFORMED`, `MODEL_TIMEOUT`, `MODEL_NO_RESPONSE`, `INVALID_ROUTE`, `DISPATCH_REJECTED`, and `EXECUTION_FAILURE`.

The outcome payload describes the controlled sink reached by the actual execution request. It is not populated from `success_predicate` or another expected value.

```text
canonical-event-v1 raw actual trace
  compared offline with
evaluator-only constraint/oracle expected truth
```

`canonical-event-v1` must never carry required referents, expected task/executor/effect values, constraint pass/fail, or any other oracle truth.

## 19.2 `canonical-event-v2` execution and transition evidence

`canonical-event-v2` retains the v1 semantic payloads and makes two raw facts explicit:

| Canonical event | Required actual value | Authority |
| --- | --- | --- |
| `execution.started` | non-empty `invocation.capability_id`, non-empty `invocation.executor_id` | accepting `TOOL_FIXTURE` or `AGENT_FIXTURE` |
| `useful_outcome.observed` | non-empty `effect.capability_id`; `before_value` and `after_value` for `VOLUME_CHANGED` | `OUTCOME_PROBE` |

For volume direction, the evaluator derives `DECREASED` exactly when `after_value < before_value`. `VOLUME_CHANGED`, the requested action, the scenario id, and the Oracle state are not substitutes for this comparison. A raw `20 -> 35` observation remains an increase even when an Oracle expects a decrease.

The payload remains compact and typed. It does not include prompts, full model output, Oracle values, or constraint verdicts. Cross-field validation rejects a volume transition missing either boundary and an execution invocation missing capability or executor identity.
