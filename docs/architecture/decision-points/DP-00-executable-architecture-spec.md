# DP-00 — Executable Architecture Specification

## Status

**Base Architecture Specification — Pre-Prototype / Pre-Pilot**

This document turns `DP-00 — VIA Primary Execution Boundary` into four **implementable, benchmark-comparable Base Architectures**.

It is not a benchmark result, not an ADR, and not a DP-00 winner selection. `docs/requirements/requirements-v1.1.md` remains the immutable Approved Baseline.

Authoritative context:

- `docs/architecture/decision-points/DP-00-primary-execution-boundary.md`
- `docs/requirements/requirements-vNext.md`
- `docs/evaluation/evaluation-strategy.md`
- `docs/evaluation/evaluation-principles.md`
- `docs/evaluation/architecture-experiment-methodology.md`
- QA-01~04 authoritative definitions under `docs/evaluation/quality-attributes/`

The next implementation stage should be able to derive prototype modules, canonical benchmark events, replay seams, Agent/tool stubs, and route-commit instrumentation from this specification without redefining the alternatives.

---

# 1. Specification objective

DP-00 asks:

> **VIA는 사용자 요청을 어디까지 직접 판단·실행하고, 어디부터 Downstream Agent에 위임할 것인가?**

The conceptual alternatives are already defined. This checkpoint fixes the **Base Architecture shape** used for controlled comparison.

The experiment must not compare one complete architecture against another architecture that has already received extra optimizations. Therefore the specification separates:

```text
Base Architecture
    = responsibility placement + required execution topology

Tactic
    = optional mechanism used to improve a Base Architecture
```

The Base Architecture comparison is the first experiment. Tactics are evaluated later against an already-defined Base.

---

# 2. Common Product Obligation

A/B/C/D must expose the **same product capability envelope** to the benchmark.

| Product Obligation | Common Requirement |
| --- | --- |
| **Interaction** | Support the same Voice/Text user requests. |
| **Context** | Be able to consume the same benchmark context evidence. |
| **Referent** | Resolve the same screen/file/task referents when required. |
| **Execution** | Be capable of achieving the same user goal. |
| **Agent capability** | Have access to the same ARGO and Specialized Agent capability set. |
| **Consent** | Satisfy the same sensitive-context/action policy requirements. |
| **Task** | Maintain user-facing task identity/tracking. |
| **Follow-up** | Continue an existing task when scenario semantics require it. |
| **Progress** | Expose equivalent progress/result semantics. |
| **Cancel** | Support cancellation of in-flight execution according to the common contract. |
| **Result binding** | Bind progress/result to the correct user goal/task. |
| **Observability** | Emit the same canonical benchmark observations required by QA instrumentation. |

The core fairness rule is:

> **Common Product Obligation != Common Internal Component Structure**

A/B/C/D may realize these obligations through different responsibility owners and state authorities. That difference is the independent architecture variable.

The benchmark must not reduce B's product capability, give C exclusive access to capabilities unavailable to A/B, or give D an extra specialist capability. The alternatives differ in **where capability ownership and execution decisions live**, not in what the user can ultimately request.

---

# 3. Base Architecture vs Tactic

## 3.1 Phase 1 — Base Architecture Evaluation

```text
A / B / C / D Base Architecture
        ↓
QA-01 / QA-02 / QA-03 / QA-04
        ↓
Base architecture trade-off
```

Base Architecture includes mechanisms that are necessary to realize the topology and preserve product correctness:

- components/responsibilities inherent to the alternative;
- deterministic state lookup;
- capability/contract lookup;
- policy/consent check;
- task identity and existing-route lookup;
- architecture-correctness validation that can be expressed deterministically from frozen state/contract/policy;
- dedicated semantic reasoning where the alternative structurally assigns that semantic decision responsibility.

## 3.2 Phase 2 — Tactic Evaluation

```text
Selected or interesting Base Architecture
        +
Optional Tactic
        ↓
QA change relative to the same Base
```

Examples treated as **Tactics**, not baked into a Base merely to improve its score:

- Cross-component GenAI fusion;
- dedicated ML/classifier routing optimization;
- embedding-based routing optimization;
- speculative execution/routing;
- parallel inference optimization;
- prompt optimization;
- cache optimization;
- partial-ASR semantic pre-routing;
- semantic result precomputation.

These mechanisms may later improve latency, correctness, cost, or call count, but they can also change coupling, validation blast radius, or change containment. They therefore deserve separate tactic-level evaluation.

Reviewer-facing distinction:

```text
Architecture
  = 누가 그 decision responsibility를 소유하는가?

Tactic
  = 그 책임을 더 빠르게/정확하게/효율적으로 수행하기 위해
    어떤 mechanism을 사용하는가?
```

Example:

```text
"VIA Agent Router가 top-level Agent routing을 책임진다"
  -> Architecture

"Agent Router를 GenAI 대신 classifier로 최적화한다"
  -> Tactic
```

---

# 4. Base Decision Mechanism Rule

The Base alternatives use the same mechanism-selection discipline so the experiment does not silently optimize one topology more than another.

```text
명백한 state / fact / contract / policy 판단
    -> DETERMINISTIC_RULE

자연어 의미 판단이 필요한 semantic decision
    -> 해당 Architecture에서 그 responsibility를 소유하는 component의
       GENAI_DEDICATED

독립 semantic decision responsibility 간 Cross-component GenAI fusion
    -> Base에서 사용하지 않음
    -> GENAI_FUSED tactic evaluation 대상
```

## 4.1 Component is not a model call

A software component and a Generative Model Call are different concepts.

Example:

```text
Intent Refiner
    -> GENAI_DEDICATED semantic interpretation
    -> 1 Generative Model Call

Agent Router
    -> deterministic capability/health/contract lookup only
    -> 0 Generative Model Calls

Components = 2
Generative Model Calls = 1
```

Conversely, if one GenAI generation simultaneously decides intent semantics and top-level Agent selection, and separate components merely forward fields afterward, the **actual semantic decision authority is fused**. Such cross-component fusion is not part of the Base comparison; it is `GENAI_FUSED` tactic evaluation.

This rule prevents the Base experiment from confusing code decomposition with inference decomposition.

## 4.2 Decision mechanism vocabulary

| Mechanism | Base/Tactic status | Meaning |
| --- | --- | --- |
| `DETERMINISTIC_RULE` | Base allowed | State/fact/contract/policy evaluation with deterministic code. |
| `GENAI_DEDICATED` | Base allowed when semantic responsibility requires it | One logical Generative AI inference generation dedicated to the owning semantic responsibility. |
| `GENAI_FUSED` | Tactic | One GenAI generation fuses independent semantic responsibilities across component boundaries. |
| `CLASSICAL_ML` | Tactic | Classifier/embedding/other non-generative learned mechanism used to optimize a decision. |
| `EXECUTOR_INTERNAL` | Base where responsibility belongs to execution runtime | Decision/reasoning owned internally by ARGO or the selected Downstream Agent during domain execution. |

A component may use more than one mechanism over its lifetime. The specification identifies the mechanism for each decision responsibility, not a permanent implementation technology label for the component.

---

# 5. Common benchmark-visible concepts

These are **observation semantics**, not mandatory shared runtime components.

Every alternative must expose enough information to normalize into:

- user-goal / episode id;
- context/referent bindings;
- task relation and user-facing task id;
- execution route;
- final execution owner;
- delegated Agent when applicable;
- clarification action;
- consent/policy action;
- progress/result binding;
- Execution Route Commit;
- logical Generative Model Call telemetry;
- useful outcome and task completion.

Internal component names may differ, especially in B. Benchmark instrumentation maps implementation-specific events into the canonical schemas already defined under `benchmark/schemas/`.

---

# 6. Alternative A — Thin VIA / Agent-neutral Orchestration

## 6.1 Base topology

```text
Voice / Text
    ↓
Context Engine
    ↓
Intent Refiner
    ↓
Agent Router
    ↓
Task / Workflow Manager
    ↓
Agent Harness
    ↓
ARGO / Specialized Agent
```

Reviewer-facing question:

> **어느 Agent에게 맡길 것인가?**

ARGO is one general-purpose Agent in the ecosystem. It is **not** a privileged primary execution runtime in A.

## 6.2 Responsibility placement

- Context evidence capture — VIA Context Engine
- Referential grounding — VIA Intent Refiner using Context Engine evidence
- Intent/goal normalization — VIA Intent Refiner
- Top-level Agent selection — VIA Agent Router
- Execution route decision — VIA
- Domain planning — Downstream Agent
- Tool selection/execution — Downstream Agent
- User-facing task lifecycle — VIA Task / Workflow Manager
- Domain execution state — selected Agent
- Consent interaction — VIA
- Result/task binding — VIA
- Follow-up association — VIA using conversation/task state

## 6.3 Base decision table

| Decision responsibility | Owner | Input | Output | Mechanism | State read/write |
| --- | --- | --- | --- | --- | --- |
| Context evidence capture | Context Engine | voice/text interaction anchors, PC context fixture | context evidence package | `DETERMINISTIC_RULE` capture/normalization | write context evidence |
| Referent + intent/goal interpretation | Intent Refiner | user input + context + conversation/task hints | normalized intent, referents, ambiguity flags | `GENAI_DEDICATED` | read context/conversation; no domain state |
| Clear existing-task continuation lookup | Task / Workflow Manager | normalized follow-up relation + task state | existing task/executor or no match | `DETERMINISTIC_RULE` | read task map |
| Top-level Agent selection for new/unrouted work | Agent Router | normalized intent + capability/health/policy data | Agent id | `GENAI_DEDICATED` for semantic Agent choice; deterministic filtering/lookups may precede it | read Agent registry/policy |
| Agent capability/health eligibility | Agent Router | capability contract + health | eligible candidates | `DETERMINISTIC_RULE` | read registry |
| Consent/policy check | Policy / Consent | context/action classification + policy state | allow / ask / deny | `DETERMINISTIC_RULE` unless scenario explicitly requires semantic clarification already owned by Intent Refiner | read/write consent state |
| User-facing task creation/correlation | Task / Workflow Manager | normalized goal + selected Agent | task id + execution mapping | `DETERMINISTIC_RULE` | write task map |
| Domain reasoning/planning/tools | selected Agent | delegated request + permitted context | execution/progress/result | `EXECUTOR_INTERNAL` | Agent-owned domain state |
| Result binding | Task / Workflow Manager | result/progress + execution mapping | user-facing task/episode binding | `DETERMINISTIC_RULE` | read/write task state |

A's Base semantic chain for a new task normally contains two independent semantic responsibilities: Intent Refiner and Agent Router. They are not fused in Phase 1.

---

# 7. Alternative B — ARGO-centric Primary Execution

## 7.1 Base topology

```text
Voice / Text
    ↓
Thin Realtime Interaction
    ↓
Thin Context Packaging
    ↓
ARGO
  ├─ request interpretation
  ├─ referential/domain reasoning
  ├─ planning
  ├─ tool execution
  └─ Specialist delegation when required
          ↓
     Specialized Agent
```

Reviewer-facing summary:

> **ARGO가 우선 요청을 받아 직접 처리하거나 필요하면 다른 Agent에 위임한다.**

B does **not** re-introduce a VIA Intent Refiner + Agent Router pipeline in front of ARGO. Doing so would collapse B toward A and undermine the responsibility-boundary experiment.

B remains the **v1.1 Responsibility Boundary Challenging Alternative**.

## 7.2 Responsibility placement

- Interaction ingress — VIA thin realtime interaction
- Context evidence capture — VIA thin capture/packaging
- Referential grounding — ARGO
- Intent/goal interpretation — ARGO
- Top-level execution decision — ARGO
- General domain reasoning/planning/tools — ARGO
- Specialist delegation — ARGO
- User-facing task identity/progress/result interaction — VIA projection/correlation
- Authoritative domain execution state/thread/plan — ARGO
- Consent interaction — VIA, with request/response bridged to ARGO execution
- Result/task binding — VIA projection mapped to ARGO execution identity

## 7.3 Task/state model

B explicitly separates **user-facing task projection** from **authoritative execution state**.

```text
VIA Task T1
    ↕ mapping/correlation
ARGO Execution E17 / thread / plan
```

VIA keeps enough task projection to support user-facing progress, cancellation, follow-up, notification, and result binding. ARGO remains authoritative for the execution plan/thread/state that actually performs the work.

The mapping is not a second VIA planning/routing authority.

## 7.4 Base decision table

| Decision responsibility | Owner | Input | Output | Mechanism | State read/write |
| --- | --- | --- | --- | --- | --- |
| Context capture/packaging | VIA thin context path | interaction + PC context fixture | evidence package for ARGO | `DETERMINISTIC_RULE` | VIA context package |
| Request/referent/goal interpretation | ARGO | user input + context + mapped task/thread state | interpreted goal/referents/ambiguity | `GENAI_DEDICATED` within ARGO's primary reasoning responsibility | read/write ARGO thread context |
| Initial self-vs-specialist execution decision | ARGO | interpreted goal + available capability/delegation knowledge | ARGO direct or Specialist Agent | same ARGO generation may be `MIXED` because B structurally owns interpretation + delegation in one primary runtime responsibility | write execution route/thread |
| Capability/policy fact lookup | ARGO integration + VIA policy boundary as applicable | capability/contract/policy state | admissible action/delegation | `DETERMINISTIC_RULE` | read registry/policy |
| User-facing task projection | VIA Task / Workflow projection | ARGO execution id + user goal | VIA task id ↔ ARGO execution mapping | `DETERMINISTIC_RULE` | write VIA mapping |
| Specialist domain execution after initial delegation | Specialized Agent | delegated subgoal/request | domain execution/result | `EXECUTOR_INTERNAL` | specialist state |
| ARGO direct domain planning/tools | ARGO | committed ARGO-owned goal | plan/actions/results | `EXECUTOR_INTERNAL` after route commit | ARGO authoritative state |
| Result binding | VIA projection | ARGO/specialist result + mapping | task/episode result binding | `DETERMINISTIC_RULE` | read/write mapping/task projection |

B's first ARGO generation may be `MIXED` by design because ARGO owns both initial semantic interpretation and the self-vs-specialist routing responsibility. This is **not** cross-component fusion introduced as a tactic; it is one execution runtime owning the combined B responsibility boundary.

---

# 8. Alternative C — Hybrid VIA Fast Path

## 8.1 Base topology

C is explicitly:

> **A + bounded VIA Fast Path**

```text
Voice / Text
    ↓
Context Engine
    ↓
Intent Refiner
    ↓
Fast Path Eligibility
    ├─ Eligible
    │    ↓
    │  VIA Fast Path
    │
    └─ Not Eligible
         ↓
       Agent Router
         ↓
       Agent Harness
         ↓
       ARGO / Specialized Agent
```

## 8.2 Fast Path Eligibility

Base eligibility is a deterministic check over:

```text
Normalized Intent
+ Fast Capability Contract
+ Policy
+ relevant deterministic state
```

Open-ended semantic interpretation remains owned by the Intent Refiner.

Fast Path semantic constraints:

- bounded execution;
- no domain planning;
- no durable Agent workflow;
- no external Agent state/thread requirement;
- simple failure/recovery semantics;
- local-safe capability;
- explicit Fast Capability Contract.

The Base **does not** use arbitrary definitions such as `<3 seconds` or `1 LLM + 1 tool`.

## 8.3 Responsibility placement

C inherits A's responsibilities except that eligible bounded local capability is owned and executed by VIA.

- Context/referent/intent — VIA
- Deterministic Fast eligibility — VIA
- Eligible local bounded execution — VIA Fast Path
- Non-eligible Agent selection — VIA Agent Router
- Domain planning/tools for delegated work — Agent
- User-facing task lifecycle/result binding — VIA

## 8.4 Base decision table

| Decision responsibility | Owner | Input | Output | Mechanism | State read/write |
| --- | --- | --- | --- | --- | --- |
| Context evidence | Context Engine | interaction/context fixture | evidence | `DETERMINISTIC_RULE` capture | context state |
| Referent + intent/goal interpretation | Intent Refiner | user input + context | normalized intent | `GENAI_DEDICATED` | read context/conversation |
| Existing task continuation | Task / Workflow Manager | normalized relation + task state | previous route/executor | `DETERMINISTIC_RULE` | task state |
| Fast eligibility | Fast Path Eligibility | normalized intent + Fast Capability Contract + policy | eligible / not eligible + local capability id | `DETERMINISTIC_RULE` | capability/policy state |
| Agent selection when not Fast | Agent Router | normalized intent + eligible Agent contracts | Agent id | `GENAI_DEDICATED` semantic selection, with deterministic candidate filtering | registry/policy |
| Local bounded execution | VIA Fast Path | normalized local command | effect/result | deterministic/local executor; no domain planning | local execution state if needed |
| Delegated domain execution | selected Agent | request + context | plan/actions/result | `EXECUTOR_INTERNAL` | Agent state |
| Result binding | Task / Workflow Manager | local/Agent result | user task/episode | `DETERMINISTIC_RULE` | task state |

## 8.5 Structural risk

The defining risk is **Fast Path boundary growth**: if more capabilities and planning responsibilities accumulate in VIA, the Fast Path can become a second general-purpose execution runtime. That risk belongs to QA-02/QA-03 trade-off analysis and must not be normalized away by silently expanding the Base capability set per scenario.

---

# 9. Alternative D — Adaptive Per-turn Execution Topology

## 9.1 Base topology

D uses existing VIA vocabulary. It does not introduce a new abstract runtime such as a “Route-neutral Control Plane.”

```text
Voice / Text
    ↓
Context Engine
    ↓
Intent Refiner
    ↓
Execution Path Selector
    ├────────► VIA Fast Path
    ├────────► ARGO Primary
    └────────► Specialized Agent Direct

Task / Workflow Manager
Session / Conversation State
Policy / Consent / Identity
Observability
    ↕
selected execution path
```

Reviewer-facing distinction:

```text
A: 어느 Agent에게 맡길 것인가?

D: 어떤 실행 topology 자체를 사용할 것인가?
```

A:

```text
Context -> Intent -> Agent Router -> Agent
```

D:

```text
Context -> Intent -> Execution Path Selector
                  ├─ VIA Fast
                  ├─ ARGO Primary
                  └─ Specialist Direct
```

## 9.2 Execution Path Selector responsibility

D does **not** place another top-level Agent Router after the Selector.

The Selector determines both:

- top-level execution topology; and
- top-level executor.

Minimum output:

```text
ExecutionRoute {
    route_kind
    executor_id
}
```

Examples:

```text
route_kind = VIA_FAST
executor_id = local_volume

route_kind = ARGO_PRIMARY
executor_id = ARGO

route_kind = SPECIALIST_DIRECT
executor_id = NetworkAgent
```

The broad selector responsibility is intentional. Its correctness, coupling, and model-call cost are part of D's QA-02/QA-03/QA-04 trade-off.

## 9.3 Base decision table

| Decision responsibility | Owner | Input | Output | Mechanism | State read/write |
| --- | --- | --- | --- | --- | --- |
| Context evidence | Context Engine | interaction/context fixture | evidence | `DETERMINISTIC_RULE` capture | context state |
| Referent + intent/goal interpretation | Intent Refiner | user input + context/task hints | normalized intent, referents, ambiguity | `GENAI_DEDICATED` | read context/conversation |
| Clear existing-task continuation lookup | Task / Workflow Manager | normalized follow-up relation + task state | previous ExecutionRoute or no reusable route | `DETERMINISTIC_RULE` | read task/route state |
| New-request execution topology + executor selection | Execution Path Selector | normalized intent + Fast/ARGO/Specialist capability contracts + policy/health | `ExecutionRoute` | `GENAI_DEDICATED` semantic route selection; deterministic contract/policy filtering may precede | read registry/policy/task context; write selected route to task state |
| VIA Fast execution | VIA Fast Path | normalized bounded command | effect/result | local executor | local state if applicable |
| ARGO Primary execution | ARGO | delegated normalized goal/context | reasoning/plan/tools/result, optional later domain subtask delegation | `EXECUTOR_INTERNAL` | ARGO state |
| Specialist Direct execution | Specialized Agent | delegated goal/context | domain execution/result | `EXECUTOR_INTERNAL` | specialist state |
| User-facing lifecycle/result binding | Task / Workflow Manager | route + progress/result | task state/result binding | `DETERMINISTIC_RULE` | task state |

## 9.4 Follow-up route reuse is Base behavior

D does not run the Execution Path Selector on every turn blindly.

For an unambiguous existing-task continuation:

```text
User Follow-up
    ↓
Context Engine + task state
    ↓
Intent Refiner confirms continuation semantics
    ↓
Task / Workflow Manager resolves existing Task T1
    ↓
Previous ExecutionRoute reuse
    ↓
continue existing executor/thread
```

The Selector is skipped because selecting a new topology would violate task-continuity semantics.

This is **not an optimization tactic**. It is the Base architecture behavior required to preserve an already-established task/execution relationship.

When follow-up semantics genuinely require a new execution topology rather than continuation, that is a new route-decision case and must be represented explicitly in the scenario contract.

---

# 10. Responsibility Placement Matrix

This table is report-ready **pre-experiment architecture specification**, not a performance result.

| Responsibility | A Thin VIA | B ARGO-centric | C Hybrid | D Adaptive |
| --- | --- | --- | --- | --- |
| Interaction ingress | VIA | VIA | VIA | VIA |
| Context evidence capture | VIA | VIA thin | VIA | VIA |
| Referential grounding | VIA Intent Refiner | ARGO | VIA Intent Refiner | VIA Intent Refiner / Context evidence |
| Intent/goal interpretation | VIA Intent Refiner | ARGO | VIA Intent Refiner | VIA Intent Refiner |
| Fast eligibility | none | none | VIA deterministic | represented inside route decision using Fast Capability Contract; deterministic contract filtering may precede semantic Selector |
| Top-level execution decision | VIA Agent Router | ARGO | Fast eligibility + Agent Router | Execution Path Selector |
| General domain reasoning | selected Agent | ARGO | selected Agent | selected executor |
| Specialist direct routing | VIA Router | none at VIA level | VIA Router | possible via Selector |
| Specialist delegation | VIA selects specialist as top-level Agent | ARGO | VIA selects specialist as top-level Agent | Selector may choose specialist directly; later executor-internal domain delegation remains executor-owned |
| Local bounded execution | achieved through Agent path | ARGO/tool path | VIA Fast Path | VIA Fast Path possible |
| User-facing task lifecycle | VIA | VIA projection/correlation | VIA | VIA |
| Domain execution state | selected Agent | ARGO | selected Agent / Fast local state | selected executor |
| Consent interaction | VIA | VIA | VIA | VIA |
| Result binding | VIA | VIA projection/correlation | VIA | VIA |
| Follow-up route reuse | VIA Task Manager + existing Agent mapping | VIA task projection + ARGO execution mapping | VIA Task Manager | VIA task state + previous ExecutionRoute |

The matrix intentionally describes responsibilities, not implementation file count or process count.

---

# 11. State Ownership Matrix

| State | A | B | C | D |
| --- | --- | --- | --- | --- |
| Conversation / Session | VIA | VIA | VIA | VIA |
| Context evidence | VIA | VIA package / evidence | VIA | VIA |
| User-facing Task ID | VIA | VIA projection | VIA | VIA |
| Execution route | VIA | ARGO authoritative + VIA mapping/projection | VIA | VIA |
| Domain execution state | Agent | ARGO | Agent / Fast | selected executor |
| Follow-up association | VIA | VIA mapping + ARGO thread/execution | VIA | VIA |
| Consent state | VIA | VIA | VIA | VIA |
| Result binding | VIA | VIA projection | VIA | VIA |

B requires special care:

```text
actual execution topology / thread / plan state
    = ARGO authoritative

user-facing task identity / progress / result correlation
    = VIA projection
```

The VIA projection must be sufficient for product obligations but must not become a hidden second source of truth for ARGO planning/execution state.

---

# 12. Execution Route Commit

QA-04's latest boundary is used consistently in the executable specification.

For Base walkthroughs, **Execution Route Commit** means:

> 사용자 요청과 pre-execution context에 근거한 initial execution-routing/delegation decision이 완료되어 실제 domain 작업을 수행할 초기 실행 경로가 확정된 최초 시점.

Equivalent operational rule:

```text
No further top-level execution-owner / initial delegation decision
is required before the chosen domain execution path can continue.
```

Examples:

### A / C delegated path

```text
Agent Router selects NetworkAgent
-> NetworkAgent accepts initial task
^ Execution Route Commit
```

### B specialist path

```text
ARGO inference decides "delegate initial request to NetworkAgent"
-> NetworkAgent accepts
^ Execution Route Commit
```

The ARGO generation is `MIXED` when it combines request/domain interpretation with this initial delegation decision, and is included in QA-04.

### D

```text
Execution Path Selector emits
{ route_kind = SPECIALIST_DIRECT, executor_id = NetworkAgent }
-> NetworkAgent accepts
^ Execution Route Commit
```

## 12.1 Initial routing vs domain sub-task delegation

QA-04 must not count every delegation that ever occurs inside a long-running Agent workflow.

Distinction:

```text
Initial execution routing/delegation required to commit the user's request route
    -> ORCHESTRATION/MIXED before or causing Execution Route Commit
    -> QA-04 Primary eligible

Later sub-task delegation caused by domain reasoning after route commit
    -> DOMAIN execution behavior
    -> QA-04 Primary excluded
```

This preserves QA-04's architecture-overhead scope while preventing hidden initial routing from escaping measurement.

---

# 13. Base architecture model-call discipline

The executable Base uses **Generative Model Call** to mean one logical Generative AI inference generation, consistent with QA-04.

```text
Deterministic rule / state / contract lookup
    -> Generative Model Call = 0

GENAI_DEDICATED generation
    -> Generative Model Call = 1

CLASSICAL_ML classifier
    -> not used in Base
    -> future Tactic/resource analysis

GENAI_FUSED
    -> not used across independent component responsibilities in Base
    -> future Tactic evaluation
```

Component count does not determine model-call count.

Cross-component semantic fusion is excluded from Base because it can improve QA-01/QA-04 while changing QA-03 coupling and QA-02 validation boundaries. The later tactic experiment can explicitly compare:

```text
Base
vs
Base + Fused Semantic Decision
```

without contaminating the Base topology comparison.

B is not in violation of this rule when ARGO's single primary generation combines request interpretation with its own self-vs-specialist choice: those responsibilities are intentionally co-owned by the same ARGO-centric runtime in Alternative B. What is excluded is adding an optional fused generation across independently specified A/C/D component responsibilities merely to optimize score.

---

# 14. Alternative structural risks

These are **pre-experiment risk hypotheses**, not measured results.

| Alternative | Main Structural Risk |
| --- | --- |
| **A Thin VIA** | VIA orchestration may absorb progressively deeper domain semantics and grow into a second Agent-like reasoning layer. |
| **B ARGO-centric** | ARGO coupling can grow while VIA loses an independent top-level routing/control point; replacement or specialist evolution may become ARGO-centric. |
| **C Hybrid** | Fast Path can keep expanding until it becomes a separate general-purpose execution runtime. |
| **D Adaptive** | Selector + multiple execution routes + ownership/state transitions can create high control complexity and a broad decision surface. |

These risks are expected to appear differently in QA-02 Correctness and QA-03 Flexibility and should be carried into prototype review.

---

# 15. Base specification invariants

The prototype implementation must preserve the following invariants unless a new architecture checkpoint changes the Base version.

1. **Same Product Capability** — all four alternatives support the same benchmark-visible obligations.
2. **Placement is the independent variable** — do not add capabilities only to one alternative.
3. **No optional optimization tactic in Base** — especially cross-component GenAI fusion, classifier routing, speculative/parallel inference, prompt/cache tuning, partial-ASR pre-routing.
4. **Deterministic facts stay deterministic** — state/contract/policy lookups do not consume GenAI merely to equalize call counts.
5. **Semantic responsibility uses the owning architecture component/runtime** — no hidden external semantic oracle that bypasses the mechanism under test.
6. **A keeps Agent-neutral routing in VIA.**
7. **B keeps primary request interpretation/execution authority in ARGO; no duplicate VIA Intent+Router pipeline.**
8. **C = A + bounded deterministic Fast Path eligibility and VIA local execution.**
9. **D uses Intent Refiner + Execution Path Selector; no second top-level Agent Router after the Selector.**
10. **D clear follow-up reuses the existing route rather than reselecting topology.**
11. **B execution state is ARGO-authoritative while VIA maintains a user-facing task projection/correlation.**
12. **Execution Route Commit is observable for every new-route case.**
13. **Canonical benchmark observations are available independent of internal topology.**

---

# 16. Prototype-facing minimum interfaces

This specification intentionally avoids freezing language/framework APIs, but the next stage should define interfaces sufficient for these semantic contracts.

## 16.1 Common scenario input

Conceptually:

```text
ScenarioInput {
  episode_id
  user_input
  modality
  context_fixture
  conversation_state_fixture
  task_state_fixture
  capability_contract_fixture
  policy_fixture
  semantic_replay_profile
}
```

## 16.2 Execution route observation

Conceptually:

```text
ExecutionRoute {
  route_kind
  executor_id
  task_id
  existing_route_reused
}
```

`route_kind` needs to distinguish at least the benchmark semantics required by A/B/C/D, such as Agent-neutral delegated, ARGO primary, VIA Fast, and Specialist Direct. This is a benchmark/canonical value vocabulary, not necessarily one shared runtime type.

## 16.3 Canonical events

The next scenario-contract checkpoint should make all alternatives emit/mappable-to events for:

- request processing start;
- semantic decision call start/end;
- clarification request/answer;
- task association;
- execution route selected;
- execution route committed;
- Agent/ARGO/Fast execution accepted/started;
- progress/result binding;
- useful outcome;
- task complete/cancel/failure.

Per-call telemetry continues to follow `benchmark/schemas/model-call-schema.md`.

---

# 17. Relationship to S1~S5 walkthrough

The detailed 20-path specification stress test is preserved in:

`docs/architecture/analysis/AA-006-dp00-executable-walkthrough.md`

That document is the pre-prototype feasibility check for:

- component/responsibility sequence;
- deterministic vs GenAI mechanism;
- task/state reads and writes;
- clarification behavior;
- Execution Route Commit;
- expected logical Generative Model Call trace.

The expected trace is a **specification prediction**, not a benchmark result or QA score.

---

# 18. Prototype specification readiness

The Base Architecture is specification-ready when all of the following hold:

- each S1~S5 scenario has a valid A/B/C/D path;
- common Product Obligations remain equivalent;
- no alternative depends on an optional tactic to function;
- state authority is explicit enough to implement follow-up/result/cancel semantics;
- every new-route path has an observable Execution Route Commit;
- model-call accounting can be derived from actual logical GenAI generations rather than inferred from component count;
- benchmark canonical observations can be mapped from each topology.

The accompanying AA-006 records the current walkthrough verdict.

## Next step

```text
DP-00 Executable Architecture Specification
        ↓
Canonical Benchmark Scenario Contract
+ Common vs Variable Experimental Boundary
        ↓
Prototype Structure
        ↓
Pilot
```

No prototype or benchmark result is created by this checkpoint.
