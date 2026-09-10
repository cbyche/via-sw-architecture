# Runtime Benchmark Fixture Contracts

## Status

**Canonical Benchmark Contract Checkpoint — shared fixture seams for DP-00 Runtime Episode Benchmark**

This document defines what common benchmark fixtures may do, and what they must not do, while comparing DP-00 Base Architectures A/B/C/D.

It complements:

- `docs/evaluation/dp00-experimental-boundary.md`
- `benchmark/schemas/runtime-scenario-schema.md`
- `benchmark/schemas/semantic-behavior-plan-schema.md`
- `benchmark/schemas/canonical-event-schema.md`

The core rule is:

> **Common fixtures provide controlled environment behavior; they do not perform Architecture-under-Test decision responsibilities.**

---

# 1. Fixture families

Runtime Architecture Qualification may use the following shared fixtures:

```text
Controlled Interaction Front-end
State Seeder Adapter
Context Evidence Fixture
Semantic Replay Provider
Agent / Domain Executor Fixture
Tool / Side-effect Fixture
Outcome Probe
Dependency Latency Profile
```

Each fixture has a narrow seam and explicit non-responsibilities.

---

# 2. Controlled Interaction Front-end

## Purpose

Remove Voice/STT/S2S algorithm variation from DP-00 Base Architecture evaluation.

Conceptual flow:

```text
Audio/Text fixture
    ↓
Controlled Interaction Front-end
    ↓
Canonical User Turn / Transcript handoff
    ↓
Architecture-under-Test
```

## Responsibilities

- play/provide the same input fixture to each alternative;
- emit authoritative `interaction.acoustic_eos` timing from fixture annotation;
- emit `interaction.input_ready` when canonical input is handed to the AUT;
- deliver scripted deterministic user replies only when the AUT produces the required canonical interaction event;
- follow the frozen latency/profile version.

## Non-responsibilities

Must not:

- interpret user intent;
- resolve referents;
- decide task association;
- decide whether clarification is needed;
- choose Fast vs Agent route;
- choose an Agent;
- pre-route based on semantic content;
- perform pre-EOS semantic optimization in DP-00 Base qualification.

Partial-ASR/pre-EOS semantic preparation remains a DP-01/tactic sensitivity concern.

---

# 3. State Seeder Adapter

## Purpose

Give every alternative the same logical initial conversation/task facts without forcing a common persistence model.

## Allowed lifecycle

```text
before episode start only
    ↓
logical initial-state fixture
    ↓
alternative-native state materialization
    ↓
seeder disabled for runtime decision path
```

## Responsibilities

- materialize the frozen logical state into the alternative's native representation;
- preserve fixture ids/version for audit;
- validate that mandatory logical facts were materialized;
- emit seed-completion diagnostic evidence before runtime begins.

## Non-responsibilities

Must not:

- determine whether the current user turn belongs to T1/T2;
- choose/rewrite an execution route for the current request;
- repair task state after seeing an AUT decision;
- inject the evaluator's expected task id as a selected task;
- remain callable by production decision code during the episode.

Alternative-specific logical→native mappings are frozen/versioned before final qualification.

---

# 4. Context Evidence Fixture

## Purpose

Provide equivalent observable environment/context facts while preserving context interpretation responsibility inside the AUT.

## Allowed evidence

Examples:

- window/surface snapshot;
- accessibility nodes;
- bounding boxes;
- pointer location/trajectory;
- focus/selection state;
- synthetic file metadata;
- conversation-visible context records.

## Prohibited shortcuts

Do not expose fields such as:

```text
correct_referent
selected_referent
expected_source_file
expected_destination
clarification_required
expected_task_id
expected_agent
```

Those are evaluator-only oracle data.

---

# 5. Semantic Replay Provider

## Purpose

Control stochastic semantic model behavior while preserving the AUT's model-call topology and semantic responsibility placement.

Authoritative contract:

`benchmark/schemas/semantic-behavior-plan-schema.md`

## Responsibilities

- accept a Replay Model Request only from an allowed AUT decision owner;
- validate `semantic_operation_keys[]` against the frozen owner/responsibility mapping;
- return the frozen payload/behavior attempt for each requested operation;
- support multiple operation keys in one logical generation when the Base architecture assigns them to the same owner;
- support retry attempt sequences;
- preserve operation consumption telemetry;
- ensure each logical generation remains a ModelCall for QA-04.

## Non-responsibilities

Must not:

- call the evaluator oracle to decide a correct route;
- invent task association;
- decide Fast eligibility;
- choose an Agent without a requested semantic operation;
- force every architecture to consume all plan operations;
- prescribe the number/order of global model calls.

---

# 6. Agent / Domain Executor Fixture

## Purpose

Provide common deterministic **post-route domain execution behavior** after the AUT has committed an execution route.

Lifecycle primitives:

```text
start
progress
complete
fail
cancel
```

Recommended script fields:

```text
agent_script_id
agent_script_version
executor_id
accepted_capability_ids[]
start_behavior
progress_events[]
completion_behavior
failure_behavior
cancel_behavior
latency_profile_ref
```

## Selection rule

The runner chooses the script out-of-band by fixture configuration after the AUT invokes an executor. The expected/correct executor is not inserted into the AUT request payload.

The evaluator separately checks whether the invoked executor/route conforms to QA-02 constraints.

## Critical ARGO boundary rule

ARGO is not always a common fixture.

### A/C/D when ARGO is only post-route executor

ARGO domain behavior may be represented by the common Domain Executor Fixture after route commit.

### B ARGO-centric

B's pre-route `ARGOPrimary` responsibilities remain inside the AUT:

```text
B.ARGOPrimary Controller      <-- AUT
  interpretation / initial self-vs-specialist decision
        ↓
execution.route_committed
        ↓
Common Domain Executor / Specialist Fixture
```

A universal ARGO stub must not absorb B's primary architecture responsibility.

---

# 7. Tool / Side-effect Fixture

## Purpose

Provide the same controllable product-side effect independent of which architecture path reaches it.

Candidate controlled capabilities:

```text
volume.set
media.pause
file.open
synthetic.file.operation
synthetic.network.diagnosis_result
```

## Responsibilities

- enforce the same input/output contract for comparable executions;
- use the same behavior/latency profile across alternatives;
- change observable fixture state;
- produce deterministic success/failure states;
- expose state to an external Outcome Probe.

## Non-responsibilities

Must not:

- choose whether a capability should run locally or through an Agent;
- decide which Agent should call it;
- infer user intent;
- expose expected architecture route.

Thus C/D do not receive an extra *product capability* merely because they own a local path. A/B can reach the same capability through their executor path.

---

# 8. Outcome Probe

## Purpose

Provide authoritative useful-outcome evidence without relying on AUT self-report.

Examples:

### Volume

```text
Tool Fixture state changes
    ↓
Outcome Probe observes requested volume
    ↓
useful_outcome.observed
```

### File open

```text
controlled file/UI sink reaches usable/open state
    ↓
Outcome Probe verifies target object
    ↓
useful_outcome.observed
```

### Short response

```text
interaction output sink receives first meaningful result segment
    ↓
Outcome Probe marks useful result start
```

## Authority rule

For QA-01, use the Outcome Probe timestamp whenever the useful outcome can be independently observed.

An AUT event saying `completed=true` is diagnostic and does not replace the probe.

---

# 9. Dependency latency profile

Runtime fixtures reference a versioned controlled latency profile.

Required properties for Architecture Qualification:

```text
deterministic
same across comparable alternatives
calibrated to avoid benchmark-fixture dominance
```

The exact milliseconds remain **TBD until Pilot**.

The profile may contain separate values for:

- interaction handoff;
- replay model generation response timing;
- Agent accept/start;
- progress;
- Tool side effect;
- result delivery.

QA-01 Pilot calibration must check that a class-specific fixture delay does not mechanically control overall FTOL p95.

---

# 10. Base model / prompt / cache profile

Architecture Qualification freezes:

```text
model_profile_id/version
prompt_profile_id/version
cache_policy_id/version
semantic_behavior_plan_id/version
```

The Semantic Replay Provider controls semantic outputs, while the ModelCall/latency/resource profile preserves the logical architecture requirement for Generative AI inference.

Optional prompt/cache/fusion/classifier optimization is not asymmetrically introduced into a Base alternative.

---

# 11. Fixture correlation without oracle leakage

Runner/fixture infrastructure may use hidden out-of-band correlation fields to locate scripts and raw evidence.

Examples:

```text
run_id
episode_id
fixture_instance_id
script_ref
```

Rules:

- these ids are infrastructure metadata;
- correct route/referent/task labels are not serialized into AUT-visible request payloads;
- fixture lookup must not return different behavior solely because the AUT guessed the expected answer unless the scenario contract intentionally defines executor-specific behavior;
- logs distinguish runner-only correlation from AUT-visible input.

---

# 12. Common domain behavior starts after route responsibility

The preferred fairness boundary is:

```text
AUT semantic / deterministic decision responsibility
        ↓
Execution Route Commit
        ↓
common deterministic domain behavior where equivalent
```

This boundary is intentionally different from “stub everything external to VIA.” In B, part of ARGO is architecture-under-test because DP-00 is testing whether ARGO owns primary reasoning/execution placement.

---

# 13. Canonical events fixtures should emit

Fixtures emit or support evidence for:

```text
interaction.acoustic_eos
interaction.input_ready
model.generation.*
execution.accepted
execution.started
progress.observed
useful_outcome.observed
cancel.confirmed
episode fixture failures
```

AUT-owned canonical events include route/task/binding decisions. The canonical event contract defines authoritative producers.

---

# 14. Failure scripts

Agent/Tool/Model fixtures should support deterministic failures needed by R10 and Mandatory Qualification scenarios.

Examples:

```text
MALFORMED semantic output
semantic TIMEOUT / NO_RESPONSE
executor reject/accept failure
domain executor terminal error
tool failure
cancel acknowledgment failure
```

The failure script is controlled; the architecture's validation, retry, fallback, clarification or safe-failure response remains variable.

---

# 15. Validation checklist

A fixture design fails the experimental-boundary review when it:

- exposes evaluator-only oracle data to AUT logic;
- performs referent/task/path/Agent selection for the AUT;
- turns B's ARGO primary decision controller into a common stub;
- supplies different product capability sets to alternatives;
- lets the AUT self-authorize QA-01 useful outcome without an available external probe;
- changes latency/model/prompt/cache profiles by alternative without a declared sensitivity experiment;
- automatically consumes semantic behavior attempts that the AUT never requested;
- performs post-hoc state repair after seeing a wrong architecture decision.

---

# 16. Prototype/Harness handoff

The next Prototype/Harness Specification should define concrete interfaces for:

```text
InteractionFrontEnd
StateSeeder
ContextFixtureSource
ReplayModelAdapter
DomainExecutorStub
ToolFixture
OutcomeProbe
CanonicalEventSink
```

Names may differ in code, but their responsibility boundaries must preserve this contract.