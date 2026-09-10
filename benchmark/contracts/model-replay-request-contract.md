# Replay Model Request Contract

## Status

**Canonical Benchmark Contract Checkpoint — logical model-test-double seam**

This contract defines the request/response metadata between an Architecture-under-Test and the deterministic Semantic Replay Provider.

It does not replace `benchmark/schemas/model-call-schema.md`. Every request that initiates a new logical Generative AI generation still creates one ModelCall record under that schema.

Authoritative semantic behavior plan:

- `benchmark/schemas/semantic-behavior-plan-schema.md`

---

# 1. Request shape

Minimum logical request:

```text
ReplayModelRequest
  model_call_id
  run_id
  episode_id
  turn_id

  alternative_id
  component
  decision_owner

  semantic_behavior_plan_id
  semantic_behavior_plan_version
  semantic_operation_keys[]

  expected_output_schema_id
  model_profile_id
  model_profile_version
  prompt_profile_version
  cache_policy_version

  attempt_context
```

`scenario_id` may exist in runner-side correlation metadata but should not be required as an AUT decision input. The replay adapter resolves the correct hidden plan through episode/run correlation.

---

# 2. Semantic operation keys

`semantic_operation_keys[]` identifies the responsibility-level semantic outputs requested in this **one logical generation**.

Examples:

```text
A.IntentRefiner:
  [turn1.intent]

A.AgentRouter:
  [turn1.agent_selection]

D.ExecutionPathSelector:
  [turn1.route, turn1.agent_selection]

B.ARGOPrimary:
  [turn1.intent, turn1.route]
```

The list does not determine QA-04 call count. The request as a whole is one logical ModelCall.

---

# 3. Attempt context

Minimum:

```text
attempt_context:
  logical_attempt
  retry_of_model_call_id
  operation_attempts
```

Example:

```text
logical_attempt: 2
retry_of_model_call_id: MC-17
operation_attempts:
  turn1.route: 2
```

A transport retry that resumes the same logical generation does not increment `logical_attempt` or consume another behavior-plan attempt.

---

# 4. Validation before replay

Before returning a payload, the replay adapter validates:

```text
component / decision_owner
  -> allowed semantic responsibility mapping

semantic_operation_keys[]
  -> present in the frozen behavior plan
  -> belong to the current episode/turn

output schema
  -> compatible with requested operation set

attempt
  -> matches deterministic consumption state
```

Validation failure is benchmark evidence and must not be silently repaired.

---

# 5. Response shape

Conceptual response:

```text
ReplayModelResponse
  model_call_id
  semantic_results[]
  composite_output
  timing_profile_ref
  replay_validation_result
```

Each semantic result contains:

```text
operation_key
responsibility
attempt_consumed
behavior_class
payload_ref
payload
failure_behavior
```

`composite_output` may match the actual interface expected by the AUT, while `semantic_results[]` preserves benchmark provenance.

---

# 6. Fused request example

Frozen plan:

```text
turn1.intent -> CORRECT
turn1.route  -> WRONG_CANDIDATE(MailAgent)
```

B requests:

```text
model_call_id: MC-B-1
component: B.ARGOPrimary
decision_owner: ARGOPrimary
semantic_operation_keys:
  - turn1.intent
  - turn1.route
```

Replay returns one composite generation containing both frozen results.

QA-04:

```text
logical Generative Model Calls = 1
```

A can request the same two operation keys in two separate generations and therefore record two ModelCalls.

The behavior fault is common; topology is not normalized away.

---

# 7. Deterministic decision case

If C Fast Path Eligibility checks a normalized intent against a capability/policy contract with deterministic code:

```text
no ReplayModelRequest
no ModelCall
no behavior-plan attempt consumed
```

This is expected and is part of the architecture mechanism difference.

---

# 8. Failure behavior

Replay responses may deterministically represent:

```text
MALFORMED
TIMEOUT
NO_RESPONSE
```

For timeout/no-response, the replay adapter follows the frozen model/dependency timing profile and emits the corresponding ModelCall failure/termination evidence.

If the AUT creates a new logical retry generation, it issues a new `ReplayModelRequest` and consumes the next operation attempt.

---

# 9. ModelCall linkage

Each ReplayModelRequest must map 1:1 to one new logical ModelCall identity.

The ModelCall raw evidence should be able to reference or derive:

```text
semantic_behavior_plan_id
semantic_behavior_plan_version
semantic_operation_keys[]
operation_attempt_refs[]
replay_validation_result
```

The existing ModelCall classification remains:

```text
ORCHESTRATION
DOMAIN
MIXED
```

Semantic operation keys do not replace that classification.

A B.ARGOPrimary generation can be `MIXED` and contain both interpretation and initial route selection.

---

# 10. Oracle isolation

The replay request/response must not expose:

```text
Required/Allowed/Forbidden evaluator constraints
correct route label
correct Agent label
correct task id
correct referent label
success predicate truth
QA score eligibility
```

Frozen semantic payloads represent **model behavior**, including wrong behavior. They are not generated by consulting the evaluator oracle at run time.

---

# 11. Raw diagnostic fields

Recommended replay-request audit evidence:

```text
request_received_ts
validation_completed_ts
response_started_ts
response_completed_ts
requested_operation_keys
consumed_attempt_refs
unused_operation_count_after_episode
validation_status
validation_failure_reason
```

These diagnostics support benchmark-integrity review and do not become Primary QA metrics by themselves.

---

# 12. Prototype handoff

The Prototype/Harness Specification must define a concrete typed interface equivalent to this logical contract and must preserve:

- 1 logical generation -> 1 ModelCall;
- responsibility-based operation keys;
- multi-operation fused request support;
- deterministic attempt consumption;
- owner/responsibility validation;
- strict oracle isolation.