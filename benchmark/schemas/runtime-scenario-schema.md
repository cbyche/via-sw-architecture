# Runtime Scenario Semantic Schema

## Status

**Canonical Benchmark Contract Checkpoint — logical Runtime Episode scenario contract**

This schema defines one **user-goal Runtime Episode** for DP-00 Architecture Qualification. It does not replace QA-03 evolution schemas or future Mandatory Qualification Suite schemas.

The concrete serialization format is intentionally not fixed yet. JSON/YAML/typed structs may be used later if they preserve this semantic contract.

Authoritative related contracts:

- `docs/evaluation/dp00-experimental-boundary.md`
- `benchmark/schemas/scenario-constraint-schema.md`
- `benchmark/schemas/semantic-behavior-plan-schema.md`
- `benchmark/schemas/run-event-schema.md`
- `benchmark/contracts/runtime-fixture-contracts.md`

---

# 1. RuntimeScenario

Minimum conceptual structure:

```text
RuntimeScenario
  identity
  qa_eligibility
  stimulus
  semantic_behavior_plan_ref
  dependency_fixture_refs
  evaluator_oracle
  termination
  provenance
```

A RuntimeScenario is a **logical user goal**, not necessarily one turn. Clarification, follow-up and branchable user interaction may span multiple turns inside one episode.

---

# 2. Identity

| Field | Required | Meaning |
| --- | --- | --- |
| `scenario_id` | yes | Stable scenario id, e.g. `R6-001`. |
| `scenario_version` | yes | Semantic version of scenario content. |
| `title` | yes | Human-readable title. |
| `scenario_class` | yes | Runtime scoring taxonomy class `R1`~`R10`. |
| `tags` | yes | Architecture-sensitivity/coverage tags; never oracle answers. |
| `description` | recommended | User-goal description. |
| `catalog_version` | yes for final qualification | Frozen runtime catalog version. |

Recommended sensitivity tags include:

```text
FAST_PATH
ARGO_PRIMARY
SPECIALIST_ROUTING
REFERENT
FOLLOW_UP
CLARIFICATION
CONCURRENCY
ERROR_RECOVERY
MODEL_VALIDATION
COMPOUND
RESULT_BINDING
```

Tags support coverage analysis only. They must not be exposed to AUT decision logic as shortcuts to an expected answer.

---

# 3. QA eligibility

Conceptual structure:

```text
qa_eligibility:
  qa01:
    eligible
    exclusion_reason
  qa02:
    eligible
    exclusion_reason
  qa04:
    eligible
    route_commit_expected
    exclusion_reason
  mandatory_gate_refs[]
```

Rules:

## QA-01

Primary eligibility requires:

- Fast-task semantics;
- Voice fixture;
- `ground_truth_acoustic_eos_offset`;
- machine-observable useful outcome;
- successful episode at run time for FTOL inclusion.

Text-only scenarios may still be useful for QA-02/QA-04 or Secondary text responsiveness.

## QA-02

Eligibility requires a versioned Architecture Constraint Manifest.

## QA-04

Scenario definition records whether an initial route commit is expected.

Minimum fields:

```text
qa04_eligible
route_commit_expected
```

Runtime records later preserve:

```text
route_commit_observed
execution_route_commit_ts
```

How terminal no-route-commit episodes enter the Primary aggregate remains **TBD until Pilot/calibration**.

The QA populations do not have to be identical.

---

# 4. AUT-visible stimulus

Only this section is eligible to be materialized into AUT-facing inputs, subject to fixture adapters.

```text
stimulus:
  interaction
  context_evidence_ref
  initial_state_ref
  capability_profile_ref
  health_profile_ref
  policy_profile_ref
```

The runtime runner may additionally provide dependency responses generated from referenced scripts when the AUT invokes those dependencies.

Ground truth and scoring constraints do not belong here.

---

# 5. Interaction Script

`stimulus.interaction` contains an ordered, branchable interaction script.

Minimum turn fields:

```text
turn_id
modality
text_fixture
audio_fixture_ref
ground_truth_acoustic_eos_offset
deterministic_user_reply_if_prompted
```

Recommended fields:

```text
turn_role              # USER / SYSTEM_FIXTURE
input_available_offset
reply_trigger
reply_timeout
fixture_version
```

### Modality

Candidate values:

```text
VOICE
TEXT
```

A turn may reference both normalized text and an audio fixture for reproducibility, but only the channel defined by the scenario/run plan is presented as user input.

### Ground-truth Acoustic EOS

`ground_truth_acoustic_eos_offset` is evaluator/benchmark timing metadata used to generate authoritative `interaction.acoustic_eos` evidence.

It is **not semantic ground truth** and must not be available to AUT decision code.

### Deterministic clarification reply

Example:

```text
turn_id: U1
text_fixture: "그 문서 열어줘."

deterministic_user_reply_if_prompted:
  trigger: clarification.requested
  next_turn:
    turn_id: U2
    text_fixture: "오른쪽에 있는 거."
```

The runner provides U2 **only if the AUT actually requests clarification** according to the canonical interaction contract.

Whether clarification was required is evaluator-only correctness truth, not an AUT-visible branch annotation.

---

# 6. Initial logical state

`initial_state_ref` points to a fixture that contains logical product facts rather than one implementation's database layout.

Example:

```text
conversation:
  conversation_id: C1

active_tasks:
  T1:
    goal: diagnose_wifi
    executor_id: NetworkAgent
    status: WAITING_USER
    continuation_ref: EX1

  T2:
    goal: summarize_document
    executor_id: ARGO
    status: RUNNING
    continuation_ref: EX2
```

The same logical facts are supplied to all alternatives.

A per-alternative test-only State Seeder may materialize these facts into native state before the episode starts, as defined in `runtime-fixture-contracts.md`.

The fixture must not encode the correct current-turn task association as an already-resolved answer.

---

# 7. Context evidence fixture

`context_evidence_ref` points to raw/logical evidence.

Allowed examples:

- surface/window snapshots;
- accessibility objects;
- bounding boxes;
- pointer trajectory/events;
- selection/focus facts;
- file metadata visible to the scenario;
- conversation-visible context snippets.

Forbidden AUT-facing fields include:

```text
selected_referent
correct_referent
expected_task_id
expected_execution_owner
expected_agent
clarification_required
```

Those belong to evaluator-only oracle/constraints.

---

# 8. Capability / health / policy profiles

The following fixture references provide facts shared across alternatives:

```text
capability_profile_ref
health_profile_ref
policy_profile_ref
```

Examples:

- which executors support which capability contracts;
- whether an Agent is available/healthy;
- whether a context class requires consent;
- whether a bounded local capability contract exists.

These facts may enable deterministic filtering inside an AUT. The benchmark does not perform the architecture's semantic selection on its behalf.

---

# 9. Semantic behavior plan reference

Required field:

```text
semantic_behavior_plan_ref
```

The referenced plan controls semantic model behavior by **semantic responsibility/operation**, not by global model-call ordinal.

The architecture requests one or more operation keys when it initiates a logical Generative AI call.

Unused operations are permitted when the architecture resolves that responsibility deterministically or never reaches that branch.

---

# 10. Dependency fixture references

Conceptual structure:

```text
dependency_fixture_refs:
  agent_scripts[]
  tool_scripts[]
  latency_profile_ref
  interaction_frontend_profile_ref
  outcome_probe_profile_ref
```

The referenced Agent/Tool fixtures are deterministic/scripted and shared where equivalent.

They must not encode routing decisions into AUT request payloads.

---

# 11. Evaluator-only oracle

Conceptual structure:

```text
evaluator_oracle:
  constraint_manifest_ref
  success_predicate_ref
  expected_result_binding_ref
  optional_ground_truth_refs
```

This object belongs exclusively to runner/evaluator code.

It must never be supplied to the AUT or semantic replay provider as decision context.

### Constraint Manifest

QA-02 uses the existing Required / Allowed / Forbidden contract from:

`benchmark/schemas/scenario-constraint-schema.md`

### Success Predicate

Defines the external observable condition used for Product success and, where applicable, QA-01 Useful Outcome.

A success predicate must be architecture-neutral. Do not define success by internal calls such as `AgentRouter.dispatch()`.

---

# 12. Termination

Minimum fields:

```text
termination:
  success_condition_ref
  failure_condition_refs[]
  timeout
```

Recommended failure conditions:

- terminal AUT error;
- prohibited side effect;
- dependency terminal failure where scenario says no recovery is possible;
- unresolved required clarification timeout;
- global episode timeout.

Timeout is a runner safety boundary, not automatically a QA score rule. Primary-metric treatment follows each authoritative QA definition.

---

# 13. Provenance and freeze

Minimum/recommended fields for final qualification:

```text
scenario_schema_version
scenario_version
catalog_version
semantic_behavior_plan_version
constraint_manifest_version
success_predicate_version
context_fixture_version
initial_state_fixture_version
capability_profile_version
health_profile_version
policy_profile_version
agent_fixture_version
tool_fixture_version
latency_profile_version
interaction_frontend_profile_version
model_profile_version
prompt_profile_version
cache_policy_version
canonical_event_schema_version
benchmark_version
```

Pilot may use explicitly marked unfrozen versions. Final scoring must use frozen versions.

---

# 14. Architecture-neutral route expectation

Scenario constraints must use the canonical execution-route representation rather than alternative-specific labels.

Canonical route shape:

```text
route_kind = LOCAL_DIRECT | EXECUTOR_DIRECT | EXECUTOR_DELEGATED
initial_executor_id
final_executor_id_if_known
delegation_chain[]
```

A scenario may allow multiple route shapes.

Example — Wi-Fi diagnosis:

```text
Required:
  goal = diagnose_wifi

Allowed routes:
  EXECUTOR_DIRECT(initial=NetworkAgent)
  EXECUTOR_DELEGATED(initial=ARGO, final=NetworkAgent)

Forbidden:
  MailAgent
  invalid LOCAL_DIRECT
```

This expresses product semantics without turning one topology into the oracle.

---

# 15. Validation rules

Reject/flag a scenario when:

- evaluator-only oracle fields are embedded in AUT-visible stimulus;
- a scenario exposes `clarification_required` before the AUT acts;
- the interaction script automatically supplies a clarification answer without a canonical clarification request;
- initial state pre-resolves the current-turn task association;
- context evidence supplies a selected/correct referent rather than raw evidence;
- scenario tags are used as runtime route hints;
- a QA-01 eligible scenario has no Voice fixture or acoustic EOS annotation;
- a QA-02 eligible scenario has no resolvable constraint manifest;
- a QA-04 eligible scenario does not define `route_commit_expected`;
- semantic behavior is specified only by model-call ordinal;
- final qualification uses unfrozen scenario/catalog/profile versions.

---

# 16. Example skeleton

```text
scenario_id: R6-001
scenario_version: v1
scenario_class: R6_AMBIGUITY_CLARIFICATION
tags: [CLARIFICATION, REFERENT]

qa_eligibility:
  qa01: false
  qa02: true
  qa04: true

stimulus:
  interaction:
    - turn_id: U1
      modality: TEXT
      text_fixture: "그 문서 열어줘."
      deterministic_user_reply_if_prompted:
        trigger: clarification.requested
        next_turn:
          turn_id: U2
          text_fixture: "오른쪽에 있는 거."
  context_evidence_ref: CTX-R6-001-v1
  initial_state_ref: STATE-EMPTY-v1
  capability_profile_ref: CAP-DP00-v1
  health_profile_ref: HEALTH-ALL-UP-v1
  policy_profile_ref: POLICY-BASE-v1

semantic_behavior_plan_ref: SBP-R6-001-v1

dependency_fixture_refs:
  tool_scripts: [TOOL-FILE-OPEN-v1]
  latency_profile_ref: LAT-PILOT-TBD

evaluator_oracle:
  constraint_manifest_ref: CONSTRAINT-R6-001-v1
  success_predicate_ref: OUTCOME-R6-001-v1

termination:
  timeout: PILOT_TBD
```

The oracle references in this source artifact are consumed only by evaluator infrastructure, never serialized into the AUT-facing request.