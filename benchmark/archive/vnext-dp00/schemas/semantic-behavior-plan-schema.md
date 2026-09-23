# Semantic Behavior Plan Schema

## Status

**Canonical Benchmark Contract Checkpoint — responsibility-based semantic replay contract**

This schema defines how Architecture Qualification injects the **same semantic success/error condition** into structurally different DP-00 alternatives without prescribing a common model-call topology.

It is not a prompt format and not a live-model API standard. It is a benchmark semantic contract.

Related artifacts:

- `docs/evaluation/dp00-experimental-boundary.md`
- `docs/architecture/decision-points/DP-00-executable-architecture-spec.md`
- `benchmark/schemas/model-call-schema.md`
- `benchmark/schemas/runtime-scenario-schema.md`

---

# 1. Why replay is responsibility-based

Global model-call ordinal is forbidden as the primary replay key.

Bad design:

```text
Model Call #1 -> output X
Model Call #2 -> output Y
```

Reason: A can separate Intent and Agent routing into two calls while B may perform both responsibilities in one ARGO generation.

The controlled variable is therefore **semantic behavior per responsibility**, not call position.

Reviewer-facing rule:

> **동일한 semantic success/error condition을 responsibility 단위로 A/B/C/D에 주입하여 Model intelligence를 통제하고 SW topology의 validation/recovery 차이를 비교한다.**

---

# 2. Canonical semantic responsibility vocabulary

Initial benchmark vocabulary:

```text
INTENT_INTERPRETATION
REFERENT_RESOLUTION
TASK_ASSOCIATION
CLARIFICATION_DECISION
EXECUTION_ROUTE_SELECTION
AGENT_SELECTION
COMPOUND_DECOMPOSITION
VALIDATION_DECISION
RESULT_BINDING
RESPONSE_GENERATION
```

This vocabulary describes semantic responsibilities, not component names.

It may be extended through a versioned schema update when a new benchmark dimension cannot be expressed with the existing set.

### Relationship to QA-02

These dimensions should remain semantically compatible with the canonical architecture outcomes and constraint dimensions used by QA-02. They are replay-operation categories, not QA-02 conformance results.

---

# 3. SemanticBehaviorPlan structure

Minimum conceptual structure:

```text
SemanticBehaviorPlan
  identity
  operations[]
  payload_registry
  allowed_owner_mapping_ref
  provenance
```

Identity fields:

| Field | Required | Meaning |
| --- | --- | --- |
| `behavior_plan_id` | yes | Stable plan id. |
| `behavior_plan_version` | yes | Version of the behavior plan. |
| `scenario_id` | yes | Runtime Scenario using the plan. |
| `scenario_version` | yes | Scenario version. |
| `semantic_schema_version` | yes | This schema/vocabulary version. |
| `payload_registry_version` | yes | Replay payload registry version. |
| `allowed_owner_mapping_version` | yes | Frozen owner→responsibility mapping version. |

---

# 4. Semantic operation

Each semantic operation represents one responsibility-level behavior to be supplied if the AUT requests it.

Minimum fields:

```text
operation_key
turn_id
responsibility
attempts[]
```

Recommended fields:

```text
input_contract_id
output_schema_id
operation_group
notes
```

Example:

```text
operation_key: turn1.intent
turn_id: U1
responsibility: INTENT_INTERPRETATION
attempts:
  - attempt: 1
    behavior_class: CORRECT
    payload_ref: PAYLOAD-R3-001-INTENT-CORRECT-v1
```

`operation_key` must be stable within the plan and cannot depend on which alternative executes it.

---

# 5. Behavior classes

Initial vocabulary:

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

A behavior class is diagnostic metadata. It is not sufficient by itself to replay the behavior.

Every attempt that returns semantic content must reference or embed a reproducible payload.

---

# 6. Attempt sequence

Retry/recovery scenarios use a deterministic sequence of attempts per operation.

Conceptual structure:

```text
attempts:
  - attempt: 1
    behavior_class: MALFORMED
    payload_ref: PAYLOAD-ROUTE-MALFORMED-v1

  - attempt: 2
    behavior_class: CORRECT
    payload_ref: PAYLOAD-ROUTE-CORRECT-v1
```

Consumption semantics:

- first valid request for the operation consumes attempt 1;
- a new logical Generative AI retry for the same operation consumes attempt 2;
- if the AUT does not retry, attempt 2 remains unused;
- transport retry of the same logical generation does not consume another semantic attempt;
- attempts are scoped by operation and logical retry semantics, not global call count.

Unused attempts remain raw diagnostic evidence.

---

# 7. Replay payload

A replay plan must preserve enough information to reproduce the exact semantic output seen by the AUT.

Each attempt contains one of:

```text
payload_ref
inline_payload
failure_behavior
```

Recommended payload metadata:

```text
payload_schema_id
payload_version
canonical_fields
raw_structured_output_if_needed
confidence_if_applicable
candidate_set_if_applicable
error_shape_if_applicable
```

Examples:

### Correct intent

```text
responsibility: INTENT_INTERPRETATION
behavior_class: CORRECT
payload:
  normalized_goal: diagnose_wifi
  requires_domain_planning: true
```

### Wrong route candidate

```text
responsibility: EXECUTION_ROUTE_SELECTION
behavior_class: WRONG_CANDIDATE
payload:
  proposed_route:
    route_kind: EXECUTOR_DIRECT
    initial_executor_id: MailAgent
  confidence: 0.91
```

### Malformed output

```text
behavior_class: MALFORMED
failure_behavior:
  raw_output_ref: RAW-MALFORMED-17
  schema_validation_expected_to_fail: true
```

### Timeout

```text
behavior_class: TIMEOUT
failure_behavior:
  terminal_after_profile_delay: true
```

No runtime model intelligence is used to regenerate the payload during Architecture Qualification.

---

# 8. Same fault across separate and fused model-call topology

Frozen semantic condition:

```text
turn1.intent = CORRECT
turn1.route  = WRONG_CANDIDATE(MailAgent)
```

## Alternative with separate generations

```text
A.IntentRefiner
  requests [turn1.intent]
  -> receives CORRECT intent

A.AgentRouter
  requests [turn1.route]
  -> receives WRONG_CANDIDATE MailAgent
```

Two logical ModelCalls may be emitted.

## Alternative with one architecture-owned fused generation

B's ARGO Primary owns several semantic responsibilities within one Base component.

```text
B.ARGOPrimary
  requests [turn1.intent, turn1.route]
  -> receives one composite replay response:
       intent = CORRECT
       route  = WRONG_CANDIDATE MailAgent
```

This is still **one logical ModelCall** for QA-04.

The semantic fault condition is equivalent across A and B even though call topology differs.

---

# 9. Multi-operation replay request

A logical model generation may request multiple semantic operations only when the executable Base Architecture assigns those responsibilities to the same decision owner for that generation.

Minimum request shape:

```text
ReplayModelRequest
  model_call_id
  episode_id
  turn_id
  component
  decision_owner
  semantic_operation_keys[]
  expected_output_schema_id
  model_profile_id
  prompt_profile_version
  attempt_context
```

The Replay Provider returns one composite result containing one result envelope per requested operation key.

Example:

```text
semantic_operation_keys = [turn1.intent, turn1.route]
```

Operation order inside a single logical generation does not create multiple QA-04 calls.

---

# 10. Model-call accounting

Deterministic replay controls output behavior; it does not remove the logical generation requirement.

Required sequence:

```text
AUT requests a logical Generative AI generation
    -> create/emit ModelCall identity
    -> validate semantic operation metadata
    -> Replay Provider supplies frozen payload(s)
    -> ModelCall completion/failure telemetry emitted
```

Therefore:

```text
replay-backed logical generation = 1 logical ModelCall
```

A deterministic architecture decision that does not request Generative AI creates no ModelCall and consumes no semantic operation.

---

# 11. Owner/responsibility mapping

Runtime code may not arbitrarily label its own call responsibilities.

A versioned mapping is frozen before final qualification.

Illustrative Base mapping:

```text
DP00-A.IntentRefiner
  allowed:
    INTENT_INTERPRETATION
    REFERENT_RESOLUTION
    CLARIFICATION_DECISION when specified by the Base decision table

DP00-A.AgentRouter
  allowed:
    AGENT_SELECTION

DP00-C.IntentRefiner
  allowed:
    INTENT_INTERPRETATION
    REFERENT_RESOLUTION
    CLARIFICATION_DECISION when specified

DP00-C.AgentRouter
  allowed:
    AGENT_SELECTION

DP00-D.IntentRefiner
  allowed:
    INTENT_INTERPRETATION
    REFERENT_RESOLUTION
    CLARIFICATION_DECISION when specified

DP00-D.ExecutionPathSelector
  allowed:
    EXECUTION_ROUTE_SELECTION
    AGENT_SELECTION

DP00-B.ARGOPrimary
  allowed:
    INTENT_INTERPRETATION
    REFERENT_RESOLUTION
    TASK_ASSOCIATION where B owns it
    CLARIFICATION_DECISION
    EXECUTION_ROUTE_SELECTION
    AGENT_SELECTION
    COMPOUND_DECOMPOSITION where pre-route
```

The executable architecture specification is authoritative when this illustrative list and a later machine-readable mapping disagree.

The Prototype/Harness Specification must publish the concrete frozen mapping.

---

# 12. Metadata-validation anti-gaming

The replay adapter validates:

```text
decision_owner
component
semantic_operation_keys[]
attempt
expected_output_schema_id
```

against the frozen Base mapping and scenario plan.

Reject/flag when:

- an owner requests a responsibility it is not allowed to own;
- an AUT omits a known route responsibility from a fused call to avoid a frozen fault;
- an AUT splits one logical generation into fake subcalls solely to change replay attempts;
- an AUT reports a deterministic decision as replay-backed without a logical generation;
- an AUT attempts to query evaluator-only ground truth through the replay API;
- an operation key belongs to another episode/turn/plan.

This prevents runtime self-labeling from becoming a way to game replay or QA-04 accounting.

---

# 13. Replay Provider non-responsibilities

The Replay Provider must not:

- choose a route that was not requested as a semantic operation;
- decide task association independently;
- decide whether clarification is required unless `CLARIFICATION_DECISION` was explicitly requested by an allowed owner;
- evaluate Fast Path eligibility for architectures where it is deterministic contract logic;
- inspect Required/Allowed/Forbidden evaluator constraints to manufacture a correct answer;
- choose an Agent because the scenario catalog says which Agent is expected;
- bind results to tasks on behalf of the AUT.

It is a **semantic model test double**, not an architecture controller.

---

# 14. Operation consumption telemetry

Raw benchmark evidence should retain or allow derivation of:

```text
behavior_plan_id
behavior_plan_version
operation_key
responsibility
requested_by_owner
model_call_id
attempt_consumed
behavior_class
payload_ref
consumed_timestamp
unused_at_episode_end
validation_result
```

This is useful for diagnosing architecture mechanisms.

For example, a deterministic validator may never consume `turn1.validation`, while a GenAI validator does. That difference must remain observable rather than normalized away.

---

# 15. Relationship to ModelCall schema

Each replay-backed logical generation corresponds to one ModelCall record.

The ModelCall record should reference:

```text
semantic_behavior_plan_id
semantic_behavior_plan_version
semantic_operation_keys[]
semantic_attempt_refs[]
replay_validation_result
```

These fields extend provenance; they do not change QA-04's authoritative definition of a logical Model Call.

---

# 16. Versioning and freeze

Final qualification freezes:

- semantic responsibility vocabulary version;
- behavior plan/version;
- replay payload registry/version;
- owner/responsibility mapping/version;
- model profile/version;
- prompt profile/version;
- replay latency/resource profile where used.

Changing a semantic payload, attempt order or owner mapping after final results are known requires a new benchmark/plan version and equal re-evaluation.

---

# 17. Example plan

```text
behavior_plan_id: SBP-R3-001-v1
behavior_plan_version: v1
scenario_id: R3-001
scenario_version: v1

operations:
  - operation_key: turn1.intent
    turn_id: U1
    responsibility: INTENT_INTERPRETATION
    attempts:
      - attempt: 1
        behavior_class: CORRECT
        payload_ref: PAYLOAD-R3-001-INTENT-v1

  - operation_key: turn1.route
    turn_id: U1
    responsibility: EXECUTION_ROUTE_SELECTION
    attempts:
      - attempt: 1
        behavior_class: WRONG_CANDIDATE
        payload_ref: PAYLOAD-R3-001-WRONG-ROUTE-v1

  - operation_key: turn1.route_retry
    # Prefer an attempt-2 on the same stable operation key in implementation;
    # shown separately only to illustrate that retries have reproducible payloads.
```

Implementation should normally model retry as attempt 2 under the same stable operation key rather than inventing a new responsibility id.