# DP-00 Experimental Boundary

## Status

**Canonical Benchmark Contract Checkpoint — Pre-Prototype / Pre-Pilot**

This document defines the controlled experimental boundary used to compare the four DP-00 Base Architectures before prototype implementation.

It is not a benchmark result, not a score, not a tactic evaluation, and not a DP-00 winner selection. `docs/requirements/requirements-v1.1.md` remains the immutable Approved Baseline.

Authoritative architecture inputs:

- `docs/architecture/decision-points/DP-00-primary-execution-boundary.md`
- `docs/architecture/decision-points/DP-00-executable-architecture-spec.md`
- `docs/evaluation/base-architecture-vs-tactic-evaluation.md`
- `docs/evaluation/evaluation-strategy.md`
- `docs/evaluation/evaluation-principles.md`
- `docs/evaluation/architecture-experiment-methodology.md`

---

# 1. Benchmark families

VIA architecture evaluation uses three benchmark families because the measurement units and evidence are different.

```text
Architecture Evaluation
|
+-- Runtime Episode Benchmark
|    +-- QA-01 Responsiveness
|    +-- QA-02 Correctness
|    +-- QA-04 Model Call Overhead
|
+-- Evolution Benchmark
|    +-- QA-03 Flexibility
|
+-- Mandatory Qualification Suite
     +-- Security / Privacy
     +-- Cancellation / Task-state integrity
     +-- Failure containment
     +-- Required recovery behavior
```

Do not force these into one universal scenario schema.

Measurement units:

```text
QA-01 / QA-02 / QA-04
  = one user-goal Runtime Episode

QA-03
  = one architecture Evolution / Change Scenario

Mandatory Qualification
  = one qualification scenario with PASS / FAIL semantics
```

This checkpoint primarily defines the **Runtime Episode Benchmark**. Existing QA-03 evolution contracts remain authoritative for evolution experiments.

---

# 2. Independent variable

The independent variable is:

```text
DP-00 Base Architecture Alternative

A = Thin VIA / Agent-neutral Orchestration
B = ARGO-centric Primary Execution
C = Hybrid VIA Fast Path
D = Adaptive Per-turn Execution Topology
```

The benchmark must compare Base Architectures before optional optimization tactics are introduced.

Reviewer-facing statement:

> **동일한 사용자 상황, 동일한 semantic model behavior, 동일한 Agent/Tool behavior를 A/B/C/D에 제공하고 responsibility placement와 execution topology를 독립변수로 비교한다.**

---

# 3. Three-layer experimental model

## Layer 1 — Shared Benchmark Environment

Common/frozen across alternatives:

```text
Scenario / User goal
Interaction fixture
Logical initial conversation/task state
Context evidence
Capability registry
Agent health
Policy / consent state

Semantic behavior plan
Downstream Agent scripted behavior
Tool behavior

Base model profile
Prompt profile
Cache policy

Controlled external dependency latency
Machine / runtime environment

QA constraint manifest
Success predicate
Canonical observation semantics
```

These inputs define equivalent conditions, not a shared implementation of architecture decisions.

## Layer 2 — Base Implementation Rules

Applied equally to every Base Architecture:

```text
fact/state/contract/policy decision
  -> deterministic code

semantic decision
  -> GenAI owned by the architecture component responsible for that decision

optional optimization tactic
  -> excluded from Base comparison
```

Cross-component GenAI fusion, learned classifier routing, embedding routing, speculative execution, parallel inference optimization, prompt/cache optimization and pre-EOS semantic routing remain tactic/other-DP work.

## Layer 3 — Architecture-specific Structure

Variable by A/B/C/D:

- component topology;
- responsibility placement;
- context consumption/interpretation placement;
- intent interpretation placement;
- execution-path decision placement;
- Agent-selection responsibility;
- ARGO placement;
- Fast Path placement;
- Specialist delegation ownership;
- task/execution-state ownership;
- model-call topology;
- serialization / IPC / internal control flow.

The benchmark harness must not normalize these differences away.

---

# 4. Controlled / frozen inputs

The following are frozen for a comparable qualification run unless the scenario explicitly makes one item the object of a separate sensitivity experiment.

| Controlled item | Rule |
| --- | --- |
| User goal | Same semantic user goal across alternatives. |
| Interaction fixture | Same voice/text content and branchable deterministic user replies. |
| Logical initial state | Same facts; native state representation may differ. |
| Context evidence | Same raw/logical evidence; grounding result is not supplied. |
| Capability registry | Same capability facts and identifiers. |
| Agent health | Same health/availability facts. |
| Policy / consent | Same policy facts and initial consent state. |
| Semantic model behavior | Same responsibility-level frozen behavior plan. |
| Agent behavior | Same post-route scripted execution behavior where comparable. |
| Tool behavior | Same deterministic side-effect behavior/profile. |
| Base model profile | Same versioned profile unless topology inherently requires a different role profile declared in the frozen run plan. |
| Prompt profile | Frozen/versioned per declared model role. |
| Cache policy | Frozen/versioned; optimization is not introduced asymmetrically. |
| Dependency latency | Deterministic, equal for comparable seams, calibrated for QA-01 non-dominance. |
| Machine/runtime environment | Same reference environment/profile. |
| QA oracle | Same constraint/success-predicate semantics. |
| Canonical observation contract | Same event meanings independent of internal component names. |

---

# 5. Architecture-under-Test responsibilities

The following must remain inside the **Architecture-under-Test (AUT)** when they exist in the alternative.

The benchmark must not perform them on behalf of the architecture:

- referent grounding;
- task association / follow-up determination;
- clarification-necessity decision;
- intent/goal interpretation;
- Fast Path eligibility;
- execution-route selection;
- Agent selection;
- ARGO self-vs-specialist delegation decision;
- deterministic architecture validation;
- task/execution-state ownership and mapping;
- result binding;
- consent workflow decisions required by architecture;
- retry/fallback decisions before Execution Route Commit.

A benchmark dependency can return a frozen semantic output **only after the AUT requests the corresponding semantic operation**.

---

# 6. Stimulus vs evaluator-only oracle separation

A scenario has two logically separate channels.

## 6.1 AUT-visible Stimulus

The architecture may receive only data that the real product could legitimately observe:

```text
User input
Raw/logical context evidence
Initial product state
Capability / health facts
Policy / consent state
External dependency responses
```

## 6.2 Evaluator-only Oracle

The following are not visible to the AUT:

```text
Ground-truth referent
Ground-truth task association
Required / Allowed / Forbidden constraints
Expected result binding
Success predicate internals
QA scoring eligibility
Expected architecture-sensitive properties
Expected route/conformance labels
```

Core anti-leakage rule:

> **Benchmark harness는 ground truth를 알고 있지만 Architecture-under-Test에는 전달하지 않는다.**

The runner must prevent direct or indirect oracle leakage through scenario payloads, environment variables, fixture metadata or convenience APIs.

### Scenario-id anti-hardcoding rule

`scenario_id`, catalog ids and fixture ids are benchmark correlation metadata, not decision inputs.

An AUT must not branch on known benchmark ids to produce expected answers. Prototype interfaces should avoid exposing evaluator-only ids to architecture decision code where practical. If an id must be present for telemetry correlation, its use is restricted to observability and must not be consulted by decision logic.

---

# 7. Controlled interaction front-end for DP-00 Base evaluation

DP-00 Base evaluation isolates the **Primary Execution Boundary**, not Voice/STT/S2S algorithm design.

Therefore the Base qualification front-end is common and controlled:

```text
Audio fixture
    ↓
Controlled common interaction front-end
    ↓
Canonical User Turn / Transcript handoff
    ↓
A / B / C / D AUT
```

Rules:

- same audio/text fixture across alternatives;
- authoritative `ground_truth_acoustic_eos_ts` comes from the benchmark fixture;
- common front-end latency follows a deterministic/frozen profile;
- partial-ASR semantic pre-routing and pre-EOS semantic preparation are excluded from DP-00 Base qualification;
- those optimizations remain DP-01/tactic sensitivity candidates;
- B's Thin Context Packaging vs A/C/D Context Engine remains **inside the AUT** and is not normalized away.

This common front-end does not perform referent grounding, intent interpretation, task association, path selection or Agent routing.

---

# 8. QA population separation

QA-01, QA-02 and QA-04 operate on Runtime Episodes but do not require identical populations.

## QA-01

Primary-eligible episodes require:

```text
voice fixture
AND ground_truth_acoustic_eos_ts
AND Fast-task semantics
AND successful useful outcome
```

Text-only episodes are not forced into FTOL Primary. Text responsiveness may be retained as a Secondary metric.

## QA-02

Primary-eligible episodes require a versioned Architecture Constraint Manifest and sufficient canonical observations to evaluate it.

## QA-04

Primary-eligible episodes require an initial Execution Route Commit observation under the frozen eligibility/aggregation rule.

The handling of terminal no-route-commit episodes remains **TBD until Pilot/calibration**. Raw evidence is mandatory regardless of eventual aggregation treatment.

---

# 9. Logical initial-state seeding

Every alternative receives the same **logical state facts**, but no shared internal persistence representation is required.

Example:

```text
active_tasks:
  T1:
    goal: diagnose_wifi
    executor: NetworkAgent
    status: WAITING_USER

  T2:
    goal: summarize_document
    executor: ARGO
    status: RUNNING
```

A test-only **State Seeder Adapter** is allowed with strict scope:

```text
before episode start only
    ↓
logical fixture state
    ↓
alternative-native state representation
```

The State Seeder must not:

- run during the decision path;
- decide task association;
- select an executor;
- infer referents;
- create a route on behalf of the AUT;
- repair state after observing the AUT's runtime decision.

Alternative-specific state mappings are versioned/frozen before final evaluation.

---

# 10. Raw context-evidence rule

Context fixtures provide evidence, not interpreted ground truth.

Allowed AUT-visible example:

```text
surface_snapshot_generation: 17
objects:
  file_A:
    bounding_box: [x1, y1, x2, y2]
  file_B:
    bounding_box: [x1, y1, x2, y2]
pointer:
  x: ...
  y: ...
```

Forbidden AUT-visible shortcut:

```text
selected_referent = file_A
```

`selected_referent = file_A` belongs in the evaluator-only oracle/constraint manifest.

This preserves referent-grounding responsibility inside the architecture topology being tested.

---

# 11. Semantic behavior injection boundary

Model behavior is controlled by **semantic responsibility**, not by global model-call ordinal.

Forbidden replay design:

```text
Model Call #1 -> output X
Model Call #2 -> output Y
```

This is topology-biased because A may have separate Intent/Router calls while B may perform multiple responsibilities in one ARGO generation.

Required design:

```text
semantic operation / responsibility
    ↓
frozen behavior attempt sequence
    ↓
AUT model-generation request
```

The schema is defined in:

`benchmark/schemas/semantic-behavior-plan-schema.md`

### Same semantic fault across different call topologies

Frozen condition:

```text
INTENT_INTERPRETATION = CORRECT
EXECUTION_ROUTE_SELECTION = WRONG_CANDIDATE
```

A may consume these through two logical generations:

```text
Intent Refiner -> CORRECT
Agent Router   -> WRONG_CANDIDATE
```

B may consume both in one ARGO generation:

```text
ARGO Primary -> [intent=CORRECT, route=WRONG_CANDIDATE]
```

The semantic fault condition is held constant even though the model-call topology differs.

A fused B call is still **one logical ModelCall** for QA-04.

---

# 12. Replay does not make architecture decisions

The Semantic Replay Provider is a model test double, not an orchestration engine.

It must not independently decide:

- route selection;
- task association;
- clarification necessity;
- Fast eligibility;
- Agent selection;
- result binding.

It returns the frozen output for semantic operation keys explicitly requested by an allowed architecture owner.

If the architecture resolves a responsibility with deterministic logic and does not request semantic generation, the corresponding behavior-plan operation may remain unused. That is a valid architecture-mechanism difference and must not be treated as a replay error.

Unused operations may be retained as diagnostic evidence.

---

# 13. Semantic-responsibility metadata anti-gaming

An AUT must not evade replay faults or QA-04 counting by relabeling a model call at runtime.

Before final qualification, freeze a mapping from executable-spec decision owner to allowed semantic responsibilities.

Illustrative mapping from the current Base specification:

```text
A.IntentRefiner
  -> INTENT_INTERPRETATION
  -> REFERENT_RESOLUTION
  -> CLARIFICATION_DECISION when owned there

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
  -> CLARIFICATION_DECISION when the Base assigns it to ARGO
```

The exact machine-readable mapping/version is frozen with the Prototype/Harness Specification. `DP-00-executable-architecture-spec.md` is authoritative for responsibility ownership.

The model adapter validates each replay request against this mapping and emits a validation failure rather than silently accepting unsupported responsibility labels.

---

# 14. Architecture-neutral execution route

The executable architecture documents may use topology-specific labels such as `VIA_FAST`, `ARGO_PRIMARY` and `SPECIALIST_DIRECT` for human explanation.

The **canonical benchmark route representation** must not require those alternative-specific enum values.

Use architecture-neutral concepts:

```text
route_kind:
  LOCAL_DIRECT
  EXECUTOR_DIRECT
  EXECUTOR_DELEGATED

initial_executor_id
final_executor_id_if_known
delegation_chain[]
```

Examples:

```text
C Fast
  route_kind = LOCAL_DIRECT
  initial_executor_id = VIA_LOCAL_VOLUME

A -> NetworkAgent
  route_kind = EXECUTOR_DIRECT
  initial_executor_id = NetworkAgent

B -> ARGO direct
  route_kind = EXECUTOR_DIRECT
  initial_executor_id = ARGO

B -> ARGO -> NetworkAgent
  route_kind = EXECUTOR_DELEGATED
  initial_executor_id = ARGO
  final_executor_id_if_known = NetworkAgent
  delegation_chain = [ARGO, NetworkAgent]
```

QA-02 constraints may allow multiple valid route shapes without encoding one topology as the answer.

---

# 15. Execution Route Commit semantics

This checkpoint preserves the QA-04 boundary while making the runtime event contract explicit.

`execution.route_committed` means:

> **user request의 initial execution-routing/delegation 판단이 완료되어, 해당 request를 실제로 수행할 초기 execution path가 확정된 최초 시점.**

Rules:

- candidate/provisional routes do not emit route commit;
- synchronous dispatch rejection requiring another route means the rejected candidate was not the final commit;
- the route-commit event occurs only once the initial operational path is stable enough for domain execution to proceed without another initial owner/delegation decision;
- a model generation that causes initial ARGO→Specialist delegation is pre-commit `MIXED`/`ORCHESTRATION` and remains QA-04 eligible;
- later sub-task delegation produced by domain reasoning after the initial route commit is `DOMAIN` execution and does not reopen QA-04 Primary for the original request.

---

# 16. ARGO / Downstream Agent seam

ARGO cannot be represented by one universal common stub across all alternatives because its architectural role changes.

In A/C, ARGO can be a downstream executor.

In B, ARGO owns pre-route architecture responsibility.

Required separation:

```text
Architecture-owned pre-route decision layer
        ↓
Execution Route Commit
        ↓
Common scripted Domain Executor behavior
```

For B:

```text
B.ARGOPrimary Controller   <-- AUT
  - request interpretation
  - self vs specialist decision
        ↓
Execution Route Commit
        ↓
Common ARGO-domain / Specialist execution fixture where applicable
```

For A/C/D:

```text
VIA decision components   <-- AUT
        ↓
Execution Route Commit
        ↓
Common Domain Executor / Specialist fixture
```

A common fixture must never absorb B's request interpretation or initial delegation authority.

---

# 17. Agent and Tool fixture seams

Detailed fixture semantics are defined in:

`benchmark/contracts/runtime-fixture-contracts.md`

## Agent fixture

Must support deterministic/scripted lifecycle behavior:

```text
start
progress
complete
fail
cancel
```

It is selected through the actual AUT route. The fixture does not decide whether it should have been selected.

## Tool fixture

The same side-effect fixture is reused where possible across alternatives, for example:

- volume;
- file-open;
- media-pause.

The product capability is common; the architecture path that reaches it is variable.

## Outcome Probe

QA-01 useful outcome is observed externally where possible. The AUT does not get to lower FTOL by self-reporting “useful outcome complete.”

---

# 18. Controlled dependency latency

Architecture Qualification uses dependency profiles that are:

```text
deterministic
same across comparable alternatives
calibrated to avoid fixture dominance
```

Exact milliseconds are **not frozen in this checkpoint**.

Pilot determines the concrete profile while preserving QA-01 equality and non-dominance rules.

---

# 19. Base model / prompt / cache profiles

Architecture Qualification freezes and records:

- base model profile/version;
- prompt profile/version;
- cache policy/version;
- replay behavior-plan/version;
- logical model latency/resource profile where simulated.

Semantic replay controls model intelligence/behavior. It does not erase the logical model generation requested by the architecture.

Optional fusion/classifier/cache/speculation optimization remains outside Base evaluation.

---

# 20. Smoke vs scoring taxonomy

Keep these populations conceptually separate:

```text
S1~S5
  = executable-architecture consistency / smoke walkthroughs

R1~R10
  = Runtime scoring benchmark taxonomy
```

S1~S5 verifies the Base architecture specification before implementation.

R1~R10 organizes the future scoring corpus and is defined in `benchmark/catalog/runtime-scenario-catalog.md`.

---

# 21. Raw evidence and version provenance

Raw Event data remains the immutable source of truth.

Every run must identify or resolve at least:

- runtime scenario schema/version;
- scenario id/version/class/tags;
- semantic behavior-plan id/version;
- consumed and unused semantic operation attempts where captured;
- Agent fixture script/version;
- Tool fixture script/version;
- dependency latency profile/version;
- state/context/capability/health/policy fixture versions;
- model/prompt/cache profiles;
- architecture alternative/version/source commit;
- constraint manifest and success-predicate versions;
- canonical event schema/version;
- benchmark/scoring/gate versions as applicable.

This preserves recomputation of alternative aggregations without mutating raw evidence.

---

# 22. Unresolved before Pilot

The following remain intentionally TBD:

- final R1~R10 scenario counts;
- final class weighting/population mix;
- QA-04 overall episode mean vs class macro-average;
- QA-04 terminal no-route-commit aggregation treatment;
- exact deterministic dependency-latency values;
- score thresholds;
- QA-02 minimum correctness gate;
- concrete JSON/YAML/JSONL serialization;
- exact Prototype/Harness API types;
- actual-model repeated-run trace corpus.

These are resolved through:

```text
Prototype/Harness Specification
  -> smoke implementation
  -> Pilot
  -> calibration
  -> rule/version freeze
  -> final qualification
```

---

# 23. Readiness criterion

This experimental boundary is sufficient for prototype design only if the implementer can answer without hidden assumptions:

```text
What may the AUT see?
What must be hidden from it?
Which decisions remain AUT-owned?
How is model behavior injected without prescribing call topology?
Where does common deterministic domain behavior begin?
Which observations are canonical across all alternatives?
```

The accompanying runtime scenario, semantic behavior, canonical event and fixture contracts complete those answers.