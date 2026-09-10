# Replay Model Request Contract

## Status

**Prototype/Harness Specification Checkpoint — logical model-test-double seam**

This contract defines the implementation boundary between an Architecture-under-Test (AUT) and the deterministic Semantic Replay Provider.

It does not replace `benchmark/schemas/model-call-schema.md`. Every request that initiates a new logical Generative AI generation still creates one ModelCall record under that schema.

Authoritative semantic behavior plan:

- `benchmark/schemas/semantic-behavior-plan-schema.md`

Prototype implementation rules:

- `docs/evaluation/prototype-benchmark-harness-spec.md`

---

# 1. Critical implementation separation

The AUT must know **which semantic responsibility it is performing**, but it must not know which benchmark scenario/operation/fault entry will satisfy that request.

Therefore the runtime implementation has two different objects:

```text
AUT-visible ModelRequest
        ↓
Benchmark-owned Replay Adapter
        ↕ hidden ReplayContext
        ↓
Resolved ReplayModelRequest / audit record
        ↓
Frozen semantic payload
```

The old conceptual shape that placed `run_id`, `behavior_plan_id`, or `semantic_operation_keys[]` directly in the AUT request is **not** the implementation API.

Those fields remain valid benchmark-internal correlation/audit metadata after the Replay Adapter resolves them.

Core rule:

> **Architecture는 자신이 수행해야 하는 semantic responsibility는 알지만 benchmark scenario의 정답/operation ID는 모른다.**

---

# 2. AUT-visible `ModelRequest`

Minimum conceptual request visible to architecture code:

```text
ModelRequest
  decision_owner
  semantic_responsibilities[]
  semantic_input
  expected_output_schema_id
  model_profile_id
  model_profile_version
```

Optional product-native correlation such as a real conversation turn/task identity may be carried when it is naturally available to the architecture, but benchmark-only identifiers are excluded.

The AUT-visible request MUST NOT contain:

```text
run_id
scenario_id
alternative_id
semantic_behavior_plan_id
semantic_behavior_plan_version
semantic_operation_key
semantic_operation_keys[]
operation_attempt number chosen from the benchmark plan
behavior_class
expected/correct route
expected/correct Agent
expected/correct referent
constraint manifest
success predicate
QA eligibility
```

A component may not select the replay case it wants to receive.

---

# 3. Benchmark-hidden `ReplayContext`

The benchmark-owned Replay Adapter maintains hidden context associated with the active run/episode.

Conceptual state:

```text
ReplayContext
  run_id
  episode_id
  scenario_id
  scenario_version
  alternative_id

  current_turn_fixture
  semantic_behavior_plan_id
  semantic_behavior_plan_version
  owner_responsibility_mapping_version

  operation_resolution_state
  operation_attempt_consumption_state
  replay_payload_registry_version
```

This state is not exposed through the AUT public API.

The adapter uses it to resolve an architecture request to one or more frozen semantic operations.

---

# 4. Benchmark-internal resolved request / audit record

After resolving an AUT-visible `ModelRequest`, the Replay Adapter may persist an internal record equivalent to:

```text
ResolvedReplayModelRequest
  model_call_id
  run_id
  episode_id
  turn_id
  scenario_id
  alternative_id

  component
  decision_owner
  requested_semantic_responsibilities[]

  semantic_behavior_plan_id
  semantic_behavior_plan_version
  semantic_operation_keys[]

  expected_output_schema_id
  model_profile_id
  model_profile_version
  prompt_profile_version
  cache_policy_version

  logical_attempt
  operation_attempts
```

This record is **benchmark evidence**, not an AUT input.

---

# 5. Semantic responsibility → operation resolution

Replay is keyed by semantic responsibility/operation, not global model-call ordinal.

Forbidden implementation:

```text
Model Call #1 -> output X
Model Call #2 -> output Y
```

Required flow:

```text
AUT requests semantic responsibility
        ↓
Replay Adapter validates owner/responsibility
        ↓
Hidden ReplayContext identifies current scenario/turn
        ↓
Semantic Behavior Plan resolves operation key(s)
        ↓
Frozen attempt payload returned
```

Example hidden plan entries:

```text
turn1.intent
  responsibility = INTENT_INTERPRETATION

turn1.route
  responsibility = EXECUTION_ROUTE_SELECTION
```

The operation keys are benchmark identifiers and remain hidden from the architecture.

---

# 6. Responsibility mapping validation

Before returning any replay result, the adapter validates:

```text
decision_owner
  -> allowed semantic responsibilities
```

The mapping is frozen from `DP-00-executable-architecture-spec.md` and the Prototype/Harness Specification.

Illustrative Base mapping:

```text
A.IntentRefiner
  -> INTENT_INTERPRETATION
  -> REFERENT_RESOLUTION
  -> CLARIFICATION_DECISION where owned

A.AgentRouter
  -> AGENT_SELECTION

C.IntentRefiner
  -> INTENT_INTERPRETATION
  -> REFERENT_RESOLUTION

C.AgentRouter
  -> AGENT_SELECTION

D.IntentRefiner
  -> INTENT_INTERPRETATION
  -> REFERENT_RESOLUTION

D.ExecutionPathSelector
  -> EXECUTION_ROUTE_SELECTION
  -> AGENT_SELECTION

B.ARGOPrimary
  -> INTENT_INTERPRETATION
  -> REFERENT_RESOLUTION
  -> EXECUTION_ROUTE_SELECTION
  -> AGENT_SELECTION
  -> CLARIFICATION_DECISION where owned
```

An unsupported request is a **contract violation**. The adapter must not silently relabel or repair it.

This prevents runtime metadata gaming such as declaring a routing call to be an unrelated semantic operation merely to avoid a frozen fault or QA-04 classification.

---

# 7. Separate vs fused topology

The same frozen semantic condition must be injectable across different model-call topologies.

Frozen condition:

```text
INTENT_INTERPRETATION     = CORRECT
EXECUTION_ROUTE_SELECTION = WRONG_CANDIDATE(MailAgent)
```

Alternative A may issue two generations:

```text
A.IntentRefiner
  responsibilities = [INTENT_INTERPRETATION]
  -> resolves hidden operation turn1.intent
  -> CORRECT

A.AgentRouter
  responsibilities = [AGENT_SELECTION / route-equivalent operation]
  -> resolves hidden route/agent operation
  -> WRONG_CANDIDATE
```

Alternative B may issue one generation because one Base component intrinsically owns several responsibilities:

```text
B.ARGOPrimary
  responsibilities = [INTENT_INTERPRETATION, EXECUTION_ROUTE_SELECTION]
  -> resolves hidden operations [turn1.intent, turn1.route]
  -> composite output [CORRECT, WRONG_CANDIDATE]
```

The semantic fault condition is common while the architecture's generation topology remains different.

QA-04 accounting:

```text
A: two new logical generations -> two ModelCalls
B: one new logical generation  -> one ModelCall
```

The Replay Adapter does not artificially split or merge generations to equalize call count.

---

# 8. Fused request support

One `ModelRequest` may list multiple semantic responsibilities only when the executable Base specification assigns those responsibilities to the same decision owner/component.

This is necessary for B.ARGOPrimary.

It does **not** permit optional cross-component fusion in A/C/D Base architectures.

Cross-component GenAI fusion remains a later Tactic according to `base-architecture-vs-tactic-evaluation.md`.

---

# 9. Attempt and retry behavior

Each semantic operation can define an attempt sequence.

Example hidden behavior plan:

```text
turn1.route
  attempt 1 = MALFORMED
  attempt 2 = CORRECT
```

The AUT does not request `attempt 2` by number.

Instead:

```text
first logical generation
  -> adapter consumes attempt 1

AUT chooses to issue a new architecture retry generation
  -> new ModelRequest
  -> adapter consumes next attempt
```

A transport retry that resumes the same logical generation does not consume another semantic attempt and does not create another ModelCall.

A true new logical generation creates a new ModelCall.

---

# 10. Deterministic architecture decision

If the architecture performs a responsibility with deterministic code and therefore does not call `ModelPort`:

```text
no ModelRequest
no ModelCall
no semantic attempt consumed
```

The corresponding Semantic Behavior Plan operation may remain unused.

That is a legitimate architecture-mechanism difference, not a replay error.

Unused operations are retained in raw diagnostics so the final report can distinguish “planned semantic fault not requested” from “replay contract failure.”

---

# 11. Replay response shape

AUT-visible conceptual response:

```text
ModelResponse
  composite_output
  model_status
```

The architecture receives only the model-like output/failure behavior appropriate to its requested output schema.

Benchmark-side audit metadata may additionally retain:

```text
model_call_id
resolved_semantic_operation_keys[]
resolved_responsibilities[]
consumed_attempt_refs[]
behavior_classes[]
payload_refs[]
replay_validation_result
timing_profile_ref
```

Do not expose those benchmark labels to architecture decision logic.

---

# 12. Failure behavior

Replay may deterministically produce:

```text
CORRECT
AMBIGUOUS
LOW_CONFIDENCE
WRONG_CANDIDATE
PARTIAL
MALFORMED
TIMEOUT
NO_RESPONSE
```

The actual payload/schema/failure timing is defined by the frozen Semantic Behavior Plan and profile references.

For `TIMEOUT`/`NO_RESPONSE`, the Rust Replay Adapter emits the appropriate ModelCall termination evidence under the frozen timing profile.

If the AUT decides to retry, the new generation consumes the next attempt.

---

# 13. ModelCall linkage

Every AUT request that starts a new logical Generative AI generation maps 1:1 to one ModelCall record.

Required benchmark-side linkage supports:

```text
model_call_id
decision_owner
requested semantic responsibilities[]
resolved semantic_operation_keys[]
operation_attempt_refs[]
replay_validation_result
```

Existing ModelCall class remains:

```text
ORCHESTRATION
DOMAIN
MIXED
```

Semantic responsibilities do not replace QA-04 call classification.

A B.ARGOPrimary generation that both interprets the request and commits initial execution ownership may be `MIXED` and is counted once.

---

# 14. Oracle isolation

Neither `ModelRequest` nor `ModelResponse` may expose evaluator-only information:

```text
Required / Allowed / Forbidden constraints
correct route label
correct Agent label
correct task id
correct referent label
success predicate truth
QA score eligibility
```

Frozen model behavior is created before execution and may be intentionally wrong. The Replay Adapter does not consult the evaluator oracle at runtime to manufacture the “correct” answer.

---

# 15. Raw diagnostic evidence

Recommended benchmark-side diagnostics:

```text
request_received_ts
owner_responsibility_validation_ts
operation_resolution_ts
response_started_ts
response_completed_ts
requested_responsibilities[]
resolved_operation_keys[]
consumed_attempt_refs[]
unused_operation_count_after_episode
validation_status
validation_failure_reason
```

These diagnostics support benchmark-integrity review but do not become Primary QA metrics by themselves.

---

# 16. Negative contract requirements

Prototype contract tests must detect at least:

1. AUT-visible request contains `scenario_id` or behavior-plan identity.
2. AUT requests a semantic operation key directly.
3. decision owner requests a disallowed semantic responsibility.
4. Replay Adapter resolves an operation from the wrong episode/turn.
5. new logical retry fails to consume the next configured attempt.
6. one logical generation is split into several ModelCalls because it contains several responsibilities.
7. deterministic code path is forced to consume a semantic operation merely for benchmark symmetry.
8. evaluator constraints are exposed through model request/response payloads.

---

# 17. Implementation handoff

The Rust Prototype/Harness implementation must preserve:

```text
AUT-visible ModelRequest
  contains architectural responsibility, not benchmark answer identity

ReplayContext
  benchmark-owned and hidden

Resolved semantic_operation_keys
  benchmark-side evidence only

1 new logical generation
  = 1 ModelCall

same semantic fault condition
  = responsibility-keyed across A/B/C/D

unused replay operation
  = allowed when architecture uses deterministic logic
```

This boundary is required before S1~S5 smoke and Pilot.