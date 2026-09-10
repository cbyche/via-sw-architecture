# DP-00 Prototype / Benchmark Harness Specification

## Status

**Prototype/Harness Specification — Pre-Implementation / Pre-Pilot**

This document defines the implementation contract for the minimum DP-00 Architecture Qualification prototype and benchmark harness.

The runtime implementation language decision is:

```text
Architecture Qualification Runtime = Rust
Offline Analysis / Statistics / Visualization = Python
```

Rationale is recorded in:

- `docs/architecture/analysis/AA-008-qualification-runtime-language-and-harness-rationale.md`

This specification is **not** production VIA implementation, not a benchmark result, not a score, and not a DP-00 winner selection.

The prototype optimizes for:

```text
architecture responsibility/topology fidelity
reproducible controlled evaluation
canonical observability
raw-evidence traceability
```

It does **not** target production feature completeness.

`docs/requirements/requirements-v1.1.md` remains unchanged.

Authoritative inputs:

- `docs/architecture/decision-points/DP-00-executable-architecture-spec.md`
- `docs/architecture/analysis/AA-006-dp00-executable-walkthrough.md`
- `docs/architecture/analysis/AA-007-benchmark-contract-neutrality-review.md`
- `docs/evaluation/dp00-experimental-boundary.md`
- `docs/evaluation/base-architecture-vs-tactic-evaluation.md`
- `benchmark/schemas/runtime-scenario-schema.md`
- `benchmark/schemas/semantic-behavior-plan-schema.md`
- `benchmark/schemas/canonical-event-schema.md`
- `benchmark/schemas/runtime-run-provenance-schema.md`
- `benchmark/contracts/model-replay-request-contract.md`
- `benchmark/contracts/runtime-fixture-contracts.md`

---

# 1. Implementation objective

The prototype must execute A/B/C/D under one controlled Rust qualification runtime and generate sufficient raw evidence for:

```text
Runtime Episode Benchmark
  QA-01 Responsiveness
  QA-02 Correctness
  QA-04 Model Call Overhead

Evolution Benchmark
  QA-03 Flexibility
  -> uses the same alternative Rust architecture source tree
```

The benchmark harness must preserve the experimental boundary:

> **공통화할 것은 실험 인프라이지 Architecture decision responsibility가 아니다.**

---

# 2. Runtime / analysis boundary

## 2.1 Rust qualification runtime

Rust owns all execution-time behavior relevant to Architecture Qualification:

- Scenario Driver;
- controlled Interaction Front-end;
- A/B/C/D Architecture Under Test;
- State Seeder adapters;
- Replay Model adapter/runtime;
- capability/policy/health fixtures;
- Agent/Domain Executor fixtures;
- Tool fixtures;
- Outcome Probes;
- canonical observation collection;
- logical ModelCall recording;
- monotonic timing;
- in-memory timed telemetry;
- immutable raw evidence serialization after the timed interval.

## 2.2 Python offline analysis

Python operates only after raw evidence has been finalized for the episode/run.

Python owns:

- raw schema/provenance validation;
- FTOL p50/p95/p99 derivation;
- AECR and constraint diagnostics;
- QA-04 overall mean / class macro-average candidates and breakdowns;
- QA-03 evolution aggregation;
- Pilot distribution inspection;
- statistics/sensitivity analysis;
- scoring/gate calibration support;
- final report tables/plots.

Python is not an AUT dependency and is not invoked from the QA-01 timed path.

---

# 3. Qualification runtime/toolchain profile

The Rust runtime substrate is a **controlled evaluation variable**, not a DP-00 alternative.

Initial v0 convention should align with the inspected Rust reference where practical:

```text
Rust edition       = 2024
minimum rust-version direction = 1.94
async runtime      = Tokio 1.x
```

The implementation branch must freeze the exact qualification profile and record:

```text
rust_toolchain_version
compiler_version / rustc -Vv identity
target_triple
rust_edition
build_profile
build_flags_profile_version
async_runtime
async_runtime_version
runtime_worker_policy_version
Cargo.lock hash / dependency_lock_hash
```

## Timed qualification build

QA-01 Timed Qualification MUST use an **optimized release-class build**.

Do not use debug-build latency for scoring.

The reference implementation's size-oriented release profile must not be copied blindly into the timing experiment. The prototype implementation defines and freezes a performance-comparison profile before Pilot/final qualification.

The same profile applies to A/B/C/D.

---

# 4. Recommended source layout

The exact Cargo workspace is created in the implementation branch, but the following logical structure is the specification target:

```text
prototypes/dp00/
├─ Cargo.toml
├─ Cargo.lock
├─ rust-toolchain.toml                # if chosen by implementation
├─ crates/
│  ├─ bench-core/
│  ├─ bench-events/
│  ├─ bench-fixtures/
│  ├─ bench-replay/
│  ├─ bench-runner/
│  ├─ bench-oracle/
│  │
│  ├─ alternative-a/
│  ├─ alternative-b/
│  ├─ alternative-c/
│  └─ alternative-d/
│
└─ tests/
   ├─ smoke/
   ├─ contract/
   └─ negative/

benchmark/
├─ scenarios/
├─ behavior-plans/
├─ fixtures/
├─ catalog/
├─ contracts/
├─ schemas/
└─ analysis/                           # Python analysis code after implementation

results/
├─ raw/
├─ derived/
└─ reports/
```

Names may be adjusted for Cargo/workspace ergonomics, but **dependency direction and responsibility separation are normative**.

---

# 5. Source-role classification for QA-03

Every source area in the prototype must be classifiable as:

```text
EVALUATION_SUPPORT
ARCHITECTURE_UNDER_TEST
```

## EVALUATION_SUPPORT

Examples:

- `bench-core`;
- `bench-events`;
- `bench-fixtures`;
- `bench-replay`;
- `bench-runner`;
- `bench-oracle`;
- scenario/behavior-plan fixtures;
- Python analysis;
- report generation.

## ARCHITECTURE_UNDER_TEST

Examples:

- `alternative-a`;
- `alternative-b`;
- `alternative-c`;
- `alternative-d`;
- architecture-owned contracts/state types that are intentionally part of a specific alternative.

QA-03 evolution analysis must not count benchmark scenario/fixture edits as architecture propagation.

Conversely, an architecture-owned common contract must not be mislabeled as evaluation support simply to hide a propagated change.

The source-role mapping/version is frozen before QA-03 final evolution evaluation.

---

# 6. Common-code limit

Shared crates may contain **evaluation infrastructure and neutral product-boundary types only**.

Allowed common content:

- canonical IDs and neutral data types;
- `UserTurn` / product-interaction boundary types;
- context evidence transport types that do not contain interpreted ground truth;
- neutral execution/result/progress/cancel port contracts;
- clock abstraction;
- benchmark fixture infrastructure;
- replay infrastructure;
- canonical event types/envelope infrastructure;
- Tool/Agent fixtures;
- Outcome Probes;
- runner;
- raw recorder.

The following decision logic MUST NOT be extracted into one common implementation shared by A/B/C/D:

```text
Intent Refiner
Agent Router
Execution Path Selector
Fast Eligibility
Task-association decision logic
ARGO Primary Controller
```

They may implement common neutral traits/ports, but their decision behavior remains inside the relevant alternative architecture artifact.

### C does not import A's architecture implementation as its architecture core

C is conceptually **A + bounded Fast Path**, but QA-03 requires C to remain independently inspectable/evolvable.

Therefore `alternative-c` must not simply re-export/import `alternative-a` as an opaque architecture implementation and add one wrapper.

Common neutral utility code is allowed; architecture decision code remains owned by C.

---

# 7. AUT public boundary

The common public interface represents only the **product interaction boundary**, not internal decomposition.

Conceptual contract:

```text
trait ArchitectureUnderTest {
    setup(...)
    handle_user_turn(UserTurn, ...)
    handle_cancel(CancelRequest, ...)
    teardown(...)
}
```

Exact Rust async trait/static-dispatch form is implementation detail, subject to equivalent wrapper overhead across A/B/C/D.

The common contract MUST NOT require architecture-specific calls such as:

```text
interpret_intent()
select_agent()
select_execution_path()
fast_eligible()
argo_delegate()
```

Those would encode A/C/D decomposition into B and invalidate the comparison.

Avoid unnecessary per-component dynamic dispatch/boxing merely for benchmark symmetry. The common **outer** boundary must have equivalent overhead; internal decomposition remains architecture-owned.

---

# 8. Neutral external ports

The harness may provide the following neutral dependency ports to an AUT:

```text
ModelPort
CapabilityRegistryPort
HealthPort
PolicyPort
DomainExecutorPort
ToolPort
ObservationPort
Clock
```

Ports provide facts or controlled dependency behavior. They do not make architecture decisions.

## CapabilityRegistryPort

Allowed:

```text
NetworkAgent supports diagnose_wifi
ARGO supports general_file_work
local_volume supports bounded volume change
```

Forbidden:

```text
Therefore choose NetworkAgent
Therefore this request is Fast eligible
```

## PolicyPort

May answer deterministic policy facts such as allow/deny/consent-required for a classified action/context property.

It must not infer the user's semantic goal unless that responsibility is explicitly a separate architecture decision using the appropriate ModelPort path.

## DomainExecutorPort

Represents the post-route scripted execution seam. It must not choose which executor should receive a request.

---

# 9. Oracle crate/type isolation

Stimulus vs Oracle separation is enforced not only by convention but by code dependency structure.

Recommended dependency rule:

```text
bench-core
  contains AUT-visible neutral product/stimulus types

bench-oracle
  contains evaluator-only constraints, expected bindings,
  ground truth and success-predicate implementation

alternative-a/b/c/d
  MUST NOT depend on bench-oracle
```

`bench-oracle` may depend on neutral canonical event/data types required for evaluation.

## Compile/dependency guard

Contract/CI tests must fail when any alternative crate depends directly or transitively on evaluator-only oracle packages.

The implementation should inspect Cargo metadata/dependency graph rather than relying only on code review.

## AUT-visible type guard

Public inputs given to an AUT must not contain:

- `scenario_id` for decision use;
- expected route;
- expected Agent;
- ground-truth referent;
- ground-truth task association;
- constraint manifest;
- QA eligibility;
- success-predicate truth.

Correlation metadata is injected outside the AUT decision API.

---

# 10. Logical Initial State and State Seeder

Each alternative may have a test-only State Seeder that materializes the common logical fixture into its native state representation.

```text
Logical Initial State
        ↓
Alternative-specific State Seeder
        ↓
Native Architecture State
        ↓
Seeder access removed
------------------------------
Episode Start
```

Rules:

- seeding occurs before `interaction.processing_started`;
- seeder does not participate in task association, routing, clarification or runtime repair;
- state mapping is versioned/frozen;
- harness may verify post-seed structural readiness but not rewrite state after the episode begins.

B may materialize:

```text
VIA Task Projection T1
    ↕
ARGO Execution/Thread E17
```

without moving ARGO execution authority into the benchmark.

---

# 11. ModelPort — AUT-visible API

The AUT knows **what semantic responsibility it owns**, but not benchmark operation ids or expected answers.

Conceptual AUT-visible request:

```text
ModelRequest {
    decision_owner,
    semantic_responsibilities[],
    semantic_input,
    expected_output_schema,
    model_profile,
}
```

AUT-visible request MUST NOT contain:

```text
scenario_id
behavior_plan_id
semantic_operation_key
turn1.intent / turn1.route benchmark keys
expected/correct route
expected/correct Agent
QA eligibility
```

A software component may know a product `turn_id`/task id if that is naturally part of product state, but benchmark-only semantic operation ids remain hidden.

---

# 12. Replay Adapter — benchmark-hidden context

The benchmark-owned ModelPort implementation holds hidden `ReplayContext` associated out-of-band with the active run/episode.

Conceptual hidden state:

```text
ReplayContext {
    run_id,
    episode_id,
    scenario_id,
    scenario_version,
    current_turn_fixture,
    semantic_behavior_plan_id,
    behavior_plan_version,
    operation_resolution_state,
    attempt_consumption_state,
    alternative_id,
    owner_responsibility_mapping_version,
}
```

Flow:

```text
AUT
  ModelRequest(
    decision_owner,
    semantic_responsibilities[],
    semantic_input,
    expected_output_schema
  )
        ↓
Benchmark-owned Replay Adapter
  validates owner/responsibilities
  resolves hidden semantic operation key(s)
  resolves attempt(s)
  emits ModelCall started
  returns frozen semantic payload
        ↓
AUT
```

The AUT never chooses the frozen behavior-plan entry.

The logical contract in `benchmark/contracts/model-replay-request-contract.md` is implemented through this two-sided separation.

---

# 13. Responsibility mapping validation

Before smoke/final qualification, freeze:

```text
decision owner
  -> allowed semantic responsibilities
```

Current Base mapping includes at least:

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
  -> CLARIFICATION_DECISION where owned

C.AgentRouter
  -> AGENT_SELECTION

D.IntentRefiner
  -> INTENT_INTERPRETATION
  -> REFERENT_RESOLUTION
  -> CLARIFICATION_DECISION where owned

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

Unsupported owner/responsibility requests are **contract violations**, not dynamically accepted metadata.

The mapping comes from `DP-00-executable-architecture-spec.md`, not from the alternative's self-description at runtime.

---

# 14. ModelCall accounting

Every new logical Generative AI generation requested through `ModelPort` creates exactly one logical ModelCall record, even when deterministic replay supplies the output.

```text
1 ModelRequest generation
= 1 logical ModelCall
```

A single request may contain several semantic responsibilities when one Base component intrinsically owns them, such as B.ARGOPrimary.

```text
B.ARGOPrimary ModelRequest
  responsibilities = [INTENT_INTERPRETATION, EXECUTION_ROUTE_SELECTION]

QA-04 logical Model Calls = 1
```

A deterministic fact/state/contract/policy lookup creates no ModelCall.

Transport retries that do not start a new logical generation create no new ModelCall. A true architecture retry generation creates a new ModelCall and consumes the next replay attempt.

---

# 15. ObservationPort — AUT-visible API

The AUT emits **semantic architecture observations**, not benchmark provenance.

Conceptual API:

```text
ObservationPort.emit(
    Observation {
        event_kind,
        architecture_payload,
        task_or_execution_correlation_if_product_owned,
    }
)
```

AUT MUST NOT supply or choose:

```text
run_id
scenario_id
alternative_id
canonical sequence_number
benchmark timestamp provenance
expected result
expected route
constraint result
QA score/eligibility
```

---

# 16. Observation Adapter — benchmark enrichment

The benchmark-owned Observation Adapter enriches AUT observations into the stored canonical envelope:

```text
AUT semantic observation
        ↓
Observation Adapter
  + run_id
  + episode_id
  + scenario_id
  + alternative_id
  + sequence_number
  + monotonic_timestamp
  + emitter provenance
  + schema/benchmark versions
        ↓
Canonical Event Buffer
```

This preserves the canonical event contract while preventing benchmark identity/oracle metadata from leaking into architecture decision code.

For events whose authoritative source is a fixture/probe, such as Acoustic EOS or useful outcome, the fixture/probe emits through the benchmark side directly rather than asking the AUT to self-report the scoring timestamp.

---

# 17. Authoritative monotonic clock

QA-01 runtime timing uses one monotonic clock domain in the Rust process.

Required relation:

```text
interaction.acoustic_eos
  = benchmark interaction fixture timestamp

useful_outcome.observed
  = external Outcome Probe timestamp when feasible

both timestamps
  = same monotonic clock domain
```

Wall-clock/calendar time may be recorded for operational correlation but is not used for FTOL interval arithmetic.

The clock representation must preserve ordering and sufficient precision without cross-language conversion inside the timed path.

---

# 18. Timed telemetry rule

Inside the QA-01 timed measurement window, event recording should be approximately:

```text
read monotonic timestamp
construct compact typed event
append to preallocated/in-memory buffer
return
```

Do not perform inside the timed interval:

- JSON serialization;
- JSONL file I/O;
- synchronous filesystem flush;
- report generation;
- Python invocation;
- external analysis RPC;
- verbose console logging on the measured path.

After the episode timed interval closes, the runner may enrich/serialize/flush the raw evidence.

## Instrumentation validity check

Pilot must compare instrumentation-on/off or a minimal-event baseline sufficient to estimate telemetry overhead.

If instrumentation overhead dominates or reverses alternative ranking, the event collection implementation must be corrected and versioned before final scoring.

---

# 19. Prototype serialization v0

Unless implementation evidence requires otherwise, use one simple evaluation-infrastructure serialization family:

```text
Runtime Scenario Manifest   -> JSON
Semantic Behavior Plan      -> JSON
Run provenance              -> JSON
Canonical raw event stream  -> JSONL
ModelCall raw stream        -> JSONL
Optional fixture event log  -> JSONL
```

All files carry schema/version identity.

This is an **evaluation infrastructure v0 contract**, not a VIA product architecture decision.

Python analysis reads these files only after they are finalized under `results/raw/`.

---

# 20. Raw evidence layout

Recommended run structure:

```text
results/raw/<run-id>/
├─ provenance.json
├─ canonical-events.jsonl
├─ model-calls.jsonl
└─ fixture-events.jsonl       # optional when separate evidence is useful
```

Derived output:

```text
results/derived/<evaluation-version>/
├─ qa01/
├─ qa02/
├─ qa03/
└─ qa04/
```

Human/report output:

```text
results/reports/
```

Rule:

> **Changing Python derivation/scoring code never rewrites the immutable raw run.**

---

# 21. Agent / Tool / ARGO seam

The Canonical Benchmark Contract requires:

```text
Architecture-owned pre-route responsibility
        ↓
Execution Route Commit
        ↓
Common scripted domain execution behavior where comparable
```

## B

```text
B.ARGOPrimary Controller       <-- ARCHITECTURE_UNDER_TEST
  request interpretation
  referent/ambiguity reasoning
  self-vs-specialist selection
  initial specialist delegation decision
        ↓
Execution Route Commit
        ↓
Common Domain Executor / Specialist Fixture
```

Do not replace B.ARGOPrimary with the common domain fixture.

## A / C / D

Their VIA decision components remain AUT through route commitment. After the route is committed, a common deterministic executor fixture may reproduce equivalent domain behavior.

## Tool fixture

Common Tool fixtures implement controlled side effects such as:

- volume change;
- file open;
- media pause;
- synthetic diagnosis/result effect.

The difference being measured is how the architecture reaches the correct executor/action, not whether one alternative receives a more capable tool implementation.

---

# 22. Alternative-specific module boundaries

The implementation should preserve the following logical decomposition.

## A — Thin VIA

```text
alternative-a/
├─ architecture
├─ context_engine
├─ intent_refiner
├─ agent_router
├─ task_manager
└─ agent_harness
```

ARGO/Specialist execution is reached through the neutral post-route executor seam.

## B — ARGO-centric

```text
alternative-b/
├─ architecture
├─ thin_context_packager
├─ argo_primary_controller
└─ task_projection
```

There is **no VIA Intent Refiner + Agent Router pipeline** in B.

## C — Hybrid

```text
alternative-c/
├─ architecture
├─ context_engine
├─ intent_refiner
├─ fast_eligibility
├─ fast_executor
├─ agent_router
├─ task_manager
└─ agent_harness
```

C is A's responsibility pattern plus bounded Fast ownership, while remaining an independently inspectable AUT artifact.

## D — Adaptive

```text
alternative-d/
├─ architecture
├─ context_engine
├─ intent_refiner
├─ execution_path_selector
├─ fast_executor
├─ task_manager
└─ agent_client
```

Do **not** add a second top-level Agent Router after D.ExecutionPathSelector.

The selector decides both top-level route kind and executor identity.

---

# 23. Base decision mechanism metadata

Each architecture-owned decision point must have traceable identity:

```text
decision_owner
decision_responsibility
decision_mechanism
```

Base mechanisms:

```text
fact/state/contract/policy lookup
  -> DETERMINISTIC_RULE

architecture-owned semantic interpretation
  -> GENAI_DEDICATED

one Base component intrinsically owning multiple responsibilities
  -> may request those responsibilities in one generation
     when the executable spec assigns them to the same component
```

Cross-component GenAI fusion, classifier routing, embedding routing, speculative routing and cache/prompt optimization remain Tactics.

B.ARGOPrimary is not mislabeled as cross-component fusion: B structurally assigns the combined initial interpretation/route responsibility to one primary runtime.

---

# 24. Two runtime modes

## 24.1 Smoke / Logical Mode

Purpose:

```text
topology
correctness contract
route/state semantics
event ordering
ModelCall accounting
fixture/oracle isolation
```

Characteristics:

- fixture latency zero or minimal;
- development build allowed for iteration;
- no QA-01 star/score produced;
- all S1~S5 paths run here first.

## 24.2 Timed Qualification Mode

Purpose:

```text
QA-01 FTOL Architecture Qualification
```

Required:

- optimized release-class Rust build;
- same frozen runtime/toolchain across A/B/C/D;
- real monotonic clock;
- calibrated deterministic dependency latency;
- lightweight in-memory timed telemetry;
- same common front-end profile;
- raw serialization after the timed interval.

Exact latency-profile values remain Pilot TBD.

---

# 25. S1~S5 smoke completion gate

The first executable completion gate is the **S1~S5 smoke set**, not the R1~R10 scoring catalog.

Scenarios:

```text
S1 Local-capable
S2 General Agent
S3 Specialized Agent
S4 Existing-task Follow-up
S5 Ambiguous Referent / Clarification
```

Every alternative executes every smoke scenario: **20 paths total**.

Required smoke assertions include:

- episode completes under expected scenario semantics;
- canonical ExecutionRoute shape is valid;
- expected executor/domain fixture is reached;
- task created or reused correctly;
- S4 preserves existing-task/route continuity;
- clarification lifecycle is correct in S5;
- canonical event ordering constraints hold;
- expected logical ModelCall trace matches the Base specification;
- observable Outcome Probe reaches the expected state where applicable;
- oracle/evaluator dependency is inaccessible from AUT crates.

Smoke tests do not compute QA star scores.

---

# 26. Expected ModelCall specification assertions

AA-006 defines the pre-experiment Base trace:

| Scenario | A | B | C | D |
| --- | ---: | ---: | ---: | ---: |
| **S1 Local** | **2** | **1** | **1** | **2** |
| **S2 General Agent** | **2** | **1** | **2** | **2** |
| **S3 Specialized** | **2** | **1** | **2** | **2** |
| **S4 Follow-up** | **1** | **1** | **1** | **1** |
| **S5 Clarification + Local** | **3** | **2** | **2** | **3** |

These values are:

> **Specification conformance assertions — not measured benchmark performance and not QA-04 scores.**

If implementation differs, first classify the discrepancy as:

```text
specification mismatch
implementation bug
replay mapping error
instrumentation/accounting error
```

Do not reinterpret an unexpected smoke call count immediately as an architecture performance result.

---

# 27. Canonical event-order contract tests

The shared evaluator may require topology-neutral partial-order invariants.

Typical successful execution:

```text
interaction.processing_started
        ↓
zero or more model.generation.*
        ↓
execution.route_committed
        ↓
execution.started / accepted as applicable
        ↓
useful_outcome.observed
        ↓
episode.completed
```

The exact ordering of `execution.started` vs route commit follows the existing Execution Route Commit definition; an intermediate owner may start before final route commitment in B, but the final committed route evidence must remain coherent.

Clarification:

```text
clarification.requested
        ↓
deterministic user reply released
        ↓
clarification.resolved
```

Follow-up:

```text
task.reused
```

must be observed for S4's clear existing-task continuation.

The tests assert semantic ordering/causation, not internal component event names.

---

# 28. Negative anti-gaming contract tests

The prototype must include executable negative tests before Pilot.

## T1 — Oracle dependency leakage

Fail when an alternative crate depends directly or transitively on `bench-oracle` or evaluator-only packages.

Preferred guard: Cargo metadata/dependency-graph test plus compile boundary.

## T2 — Scenario-id leakage

Fail when AUT-visible product/stimulus request types expose `scenario_id` or benchmark answer identifiers for decision logic.

Runner/event enrichment may retain the id outside the AUT API.

## T3 — Semantic responsibility violation

Fail a Replay Model request when `decision_owner` asks for a responsibility not present in the frozen owner→responsibility mapping.

## T4 — B ARGO responsibility leakage

Fail conformance when B routes directly from thin context into the common domain executor without B.ARGOPrimary performing the architecture-owned interpretation/initial self-vs-specialist decision defined by the Base spec.

## T5 — Premature Route Commit

Fail when `execution.route_committed` is emitted while another initial owner/delegation decision is still required, or before a synchronously rejected route has been replaced.

## T6 — Invalid duplicate initial Route Commit

Fail when the same user-goal initial route is committed multiple times without a schema-defined recovery/recommit transition. The Base contract expects one authoritative initial commit.

## T7 — Fake Useful Outcome

An AUT-only `useful_outcome` self-report cannot become QA-01 authoritative evidence when an Outcome Probe is configured and has not confirmed the observable state.

## T8 — Replay benchmark-key leakage

Fail if alternative code receives or branches on `semantic_operation_key`, `behavior_plan_id`, expected fault class or attempt-plan identity.

## T9 — Observation provenance spoofing

Fail if AUT API attempts to choose benchmark `run_id`, `scenario_id`, `alternative_id`, canonical `sequence_number`, authoritative timestamp source, or evaluator conformance result.

These negative tests turn methodology controls into executable architecture-evaluation guards.

---

# 29. Python analysis contract

The offline Python layer receives only finalized raw evidence and frozen schema/profile definitions.

Minimum analysis responsibilities:

## Validation

- provenance completeness;
- schema versions;
- event sequence/correlation;
- ModelCall linkage;
- oracle-access violation flags;
- source Git/toolchain/profile identity.

## QA-01

- `FTOL_i = useful_outcome_ts - acoustic_eos_ts`;
- p50/p95/p99;
- class slices;
- instrumentation/dependency-latency sensitivity diagnostics.

## QA-02

- Required/Allowed/Forbidden constraint evaluation projection verification;
- AECR;
- per-constraint/category diagnostics.

## QA-04

- route-commit Primary call count;
- overall episode mean;
- class-level macro-average candidate;
- call class/purpose breakdown;
- retries;
- model/token/cache/resource diagnostics.

## QA-03

- load evolution scenario/run contracts;
- CCR;
- Unexpected Changed Architecture Areas Count;
- category/sensitivity diagnostics.

Python analysis/scoring code is versioned separately from Rust runtime evidence.

---

# 30. Prototype Conformance Review gate

Pilot must not start immediately after code compiles.

Required lifecycle:

```text
Specification
    ↓
Rust Prototype
    ↓
S1~S5 Smoke
    ↓
Prototype Conformance Review
    ↓
PASS
    ↓
Pilot
```

Review questions:

### Alternative fidelity

- Is A still Thin VIA with explicit Agent Router ownership?
- Has B accidentally gained a VIA Intent Refiner/Agent Router layer?
- Has C Fast Path expanded into a general-purpose second Agent/runtime?
- Is D still Intent Refiner + Execution Path Selector without a second top-level Agent Router?
- Does D clear follow-up route reuse avoid unnecessary reselection?

### Harness neutrality

- Does the harness avoid intent/routing/task-association decisions?
- Is evaluator-only Oracle data structurally inaccessible to AUT code?
- Are semantic behavior plans resolved by the Replay Adapter rather than the AUT?
- Is B.ARGOPrimary architecture responsibility still in B rather than the common fixture?
- Are common Agent/Tool fixtures post-route wherever required?

### Instrumentation fidelity

- Is timed event collection lightweight and equivalent across alternatives?
- Is Python/file serialization outside QA-01 timed interval?
- Do canonical observations normalize outcomes without forcing common internal components?

A failed conformance review is a prototype/spec defect, not a negative architecture score.

---

# 31. QA-01 Threats to Validity after language choice

Rust reduces one implementation-substrate confounder but does not eliminate QA-01 validity threats.

Remaining factors include:

- replay-model implementation;
- deterministic Agent/Tool fixtures;
- synthetic dependency latency;
- reference-machine configuration;
- async scheduling variance;
- prototype code maturity;
- memory allocation differences;
- instrumentation overhead;
- controlled front-end abstraction.

Therefore report QA-01 Architecture Qualification as:

> **Controlled relative architecture latency comparison.**

Do not claim that prototype FTOL is the final absolute production VIA latency.

Real-stack validation remains required for external validity.

---

# 32. Specification-to-implementation traceability

Each implementation item should trace back to an authoritative contract.

| Implementation area | Primary specification source |
| --- | --- |
| A/B/C/D responsibility topology | `DP-00-executable-architecture-spec.md` |
| S1~S5 expected path/calls | `AA-006-dp00-executable-walkthrough.md` |
| Shared vs AUT boundary | `dp00-experimental-boundary.md` |
| Scenario input/oracle split | `runtime-scenario-schema.md` |
| Semantic replay behavior | `semantic-behavior-plan-schema.md` |
| AUT/Replay request separation | `model-replay-request-contract.md` + this spec |
| Fixture seams | `runtime-fixture-contracts.md` |
| Canonical observations | `canonical-event-schema.md` + this spec |
| Raw provenance | `runtime-run-provenance-schema.md` |
| Model-call evidence | `model-call-schema.md` |
| Runtime episode evidence | `run-event-schema.md` |
| Base vs Tactic | `base-architecture-vs-tactic-evaluation.md` |
| Runtime language boundary | `AA-008` |

---

# 33. Implementation completion criteria before Pilot

Prototype/Harness implementation is ready for Pilot only when all are true:

```text
[ ] one frozen Rust workspace/toolchain/runtime profile exists
[ ] alternative A builds and passes S1~S5 smoke
[ ] alternative B builds and passes S1~S5 smoke
[ ] alternative C builds and passes S1~S5 smoke
[ ] alternative D builds and passes S1~S5 smoke
[ ] 20-path expected ModelCall conformance assertions pass
[ ] canonical event ordering/correlation contract tests pass
[ ] oracle dependency guard passes
[ ] benchmark-key leakage guards pass
[ ] owner/responsibility replay validation passes
[ ] B ARGO responsibility conformance guard passes
[ ] route-commit negative tests pass
[ ] Outcome Probe authority test passes
[ ] raw provenance contains runtime/toolchain/schema/profile identities
[ ] raw JSON/JSONL can be validated by offline Python tooling
[ ] instrumentation-overhead Pilot check plan is executable
[ ] Prototype Conformance Review = PASS
```

---

# 34. Unresolved / Pilot-time configuration

This specification intentionally does not freeze:

- final R1~R10 scoring corpus size;
- production-frequency weighting;
- QA-04 overall mean vs class macro-average;
- terminal no-route-commit QA-04 aggregation rule;
- final dependency-latency milliseconds;
- QA-01~04 scoring thresholds;
- QA-02 correctness-gate value;
- final Mandatory Qualification Suite details;
- actual-model repeated-run corpus;
- tactic implementations;
- DP-00 winner.

Implementation-level details still to lock in the prototype branch include:

- exact Rust toolchain channel/patch identity consistent with the 1.94 baseline direction;
- exact Tokio resolved version and worker policy;
- exact optimized qualification build flags/profile name;
- concrete Rust trait/static-dispatch signatures;
- concrete JSON/JSONL field encoding and file-write strategy;
- exact Python package lock/environment identity.

These are controlled implementation choices, not alternative-specific architecture decisions.

---

# 35. Specification readiness verdict

This document closes the **Prototype / Benchmark Harness Specification** boundary.

The next step is implementation, beginning with:

```text
Rust workspace / shared harness skeleton
    ↓
AUT public ports + Oracle dependency guard
    ↓
Replay / Event / Fixture infrastructure
    ↓
A/B/C/D Base modules
    ↓
S1~S5 smoke + negative contract tests
    ↓
Prototype Conformance Review
    ↓
Pilot
```

No prototype code is created by this checkpoint.