# AA-006 — DP-00 Executable Architecture Walkthrough

## Status

**Architecture Analysis Record — Executable Structure Stress-test — Pre-Prototype / Pre-Pilot**

This record stress-tests the four Base Architectures defined in:

`docs/architecture/decision-points/DP-00-executable-architecture-spec.md`

against five common scenarios.

It is **pre-experiment specification evidence**. It is not a benchmark result, QA score, measured latency, or DP-00 winner selection.

The purpose is to detect structural contradictions before implementation: missing responsibility owners, hidden routing, accidental capability asymmetry, ambiguous state authority, inconsistent Execution Route Commit boundaries, or model-call accounting that depends on component count rather than actual Generative AI generations.

---

# 1. Walkthrough conventions

## 1.1 Decision mechanisms

```text
DETERMINISTIC_RULE
  state / fact / contract / policy lookup or validation

GENAI_DEDICATED
  one logical Generative AI inference generation owned by one Base semantic responsibility

GENAI_FUSED
  cross-component semantic fusion; not used in Base walkthroughs

CLASSICAL_ML
  classifier/embedding optimization; not used in Base walkthroughs

EXECUTOR_INTERNAL
  domain reasoning/execution owned by ARGO or selected Agent after route commit
```

`Component != Model Call`.

A deterministic Agent Registry lookup is a component action with zero Generative Model Calls. A GenAI semantic generation is one logical call regardless of streaming chunk count.

## 1.2 Execution Route Commit

The walkthrough marks the first point where the **initial domain execution route is committed** and no further top-level owner/initial-delegation decision is needed before domain execution continues.

Later sub-task delegation caused by domain reasoning after this point is `DOMAIN` behavior and is outside QA-04 Primary.

## 1.3 Expected call trace

Each walkthrough records **expected logical Generative Model Calls implied by the Base specification**.

These are used to check architecture-definition consistency.

They are explicitly:

```text
Pre-experiment specification walkthrough
Expected logical Generative Model Call trace
Not measured benchmark result
Not a QA-04 score
```

Actual QA-04 is measured later from `ModelCall` telemetry under the frozen benchmark.

---

# 2. Common scenarios

### S1 — Local-capable request

```text
"볼륨 조금 줄여줘."
```

The same system-volume capability is reachable in all alternatives. Only C/D may own it directly as VIA Fast Path in the Base.

### S2 — General Agent request

```text
"다운로드 폴더를 정리해줘."
```

Fixture semantics require judgment across multiple files and a plan. It is **not** a bounded Fast Path scenario.

### S3 — Specialized Agent request

```text
"현재 Wi-Fi 문제를 진단해줘."
```

The benchmark capability set includes `NetworkAgent` and ARGO for all alternatives.

### S4 — Existing-task Follow-up

```text
Initial:  "Wi-Fi 문제 분석해줘."
Follow-up: "그럼 DNS도 확인해봐."
```

The walkthrough below focuses on the **follow-up turn** after Task `T1` and its execution mapping already exist.

### S5 — Ambiguous Referent / Clarification

```text
User: "그 문서 열어줘."
```

Two documents are equally plausible. Execution before clarification is forbidden.

Deterministic clarification answer:

```text
"오른쪽에 있는 거."
```

After clarification, the requested open-document action is local-capable, but the same capability remains reachable through ARGO/Agent paths in alternatives without VIA Fast ownership.

---

# 3. S1 — Local-capable request

## S1-A — Thin VIA

Expected path:

```text
Voice/Text
 -> Context Engine
 -> Intent Refiner
 -> Agent Router
 -> ARGO
 -> volume action
```

| Step | Component | Decision responsibility | Mechanism | State read/write | GenAI call | Clarification | Route-commit note |
| --- | --- | --- | --- | --- | ---: | --- | --- |
| 1 | Context Engine | capture current interaction/context evidence | `DETERMINISTIC_RULE` | write context evidence | 0 | no | — |
| 2 | Intent Refiner | understand “reduce volume” and normalize goal | `GENAI_DEDICATED` | read context/session | **1** | no | route not chosen |
| 3 | Agent Router | select eligible general Agent for this capability under A | `GENAI_DEDICATED` after deterministic capability filtering | read registry/policy | **1** | no | selects ARGO |
| 4 | Task / Workflow Manager + Harness | create/correlate task and dispatch ARGO | `DETERMINISTIC_RULE` | write T1→ARGO mapping | 0 | no | **Commit when ARGO accepts initial execution** |
| 5 | ARGO | perform requested domain/local tool action | `EXECUTOR_INTERNAL` | ARGO state | post-commit DOMAIN | no | outside route-decision accounting |

Expected Base logical Generative Model Call trace: **2**.

Rationale: A does not own a VIA local execution path. Intent interpretation and top-level Agent choice are independent Base semantic responsibilities.

## S1-B — ARGO-centric

```text
Voice/Text
 -> thin Context Packaging
 -> ARGO
 -> volume action
```

| Step | Component | Decision responsibility | Mechanism | State read/write | GenAI call | Clarification | Route-commit note |
| --- | --- | --- | --- | --- | ---: | --- | --- |
| 1 | VIA thin interaction/context | capture/package evidence | `DETERMINISTIC_RULE` | VIA evidence package | 0 | no | — |
| 2 | ARGO | interpret request and decide ARGO can execute directly | `GENAI_DEDICATED` / QA-04 class `MIXED` because route ownership is decided in same ARGO generation | write ARGO execution/thread | **1** | no | **Commit when ARGO chooses direct ownership / accepts task** |
| 3 | VIA task projection | map user-facing T1 to ARGO execution E1 | `DETERMINISTIC_RULE` | write T1↔E1 | 0 | no | projection, not routing |
| 4 | ARGO | perform volume action | `EXECUTOR_INTERNAL` | ARGO state | post-commit DOMAIN | no | — |

Expected Base logical Generative Model Call trace: **1**.

## S1-C — Hybrid

```text
Context Engine
 -> Intent Refiner
 -> Fast Path Eligibility
 -> VIA Fast Path
```

| Step | Component | Decision responsibility | Mechanism | State read/write | GenAI call | Clarification | Route-commit note |
| --- | --- | --- | --- | --- | ---: | --- | --- |
| 1 | Context Engine | capture evidence | `DETERMINISTIC_RULE` | context | 0 | no | — |
| 2 | Intent Refiner | normalize volume goal | `GENAI_DEDICATED` | context/session | **1** | no | — |
| 3 | Fast Path Eligibility | verify normalized goal against Fast Capability Contract + policy | `DETERMINISTIC_RULE` | read fast contract/policy | 0 | no | selects `local_volume` |
| 4 | VIA Fast Path | start bounded volume execution | deterministic local executor | local effect state | 0 | no | **Commit at local execution start after eligibility** |

Expected Base logical Generative Model Call trace: **1**.

## S1-D — Adaptive

```text
Context Engine
 -> Intent Refiner
 -> Execution Path Selector
 -> VIA Fast Path
```

| Step | Component | Decision responsibility | Mechanism | State read/write | GenAI call | Clarification | Route-commit note |
| --- | --- | --- | --- | --- | ---: | --- | --- |
| 1 | Context Engine | capture evidence | `DETERMINISTIC_RULE` | context | 0 | no | — |
| 2 | Intent Refiner | normalize request | `GENAI_DEDICATED` | context/session | **1** | no | — |
| 3 | Execution Path Selector | choose topology + executor: `VIA_FAST/local_volume` | `GENAI_DEDICATED` with deterministic contract/policy filtering | read capability/policy | **1** | no | route selected |
| 4 | Task state + VIA Fast Path | persist route and start local action | `DETERMINISTIC_RULE` | write T1.ExecutionRoute | 0 | no | **Commit when selected local route starts** |

Expected Base logical Generative Model Call trace: **2**.

D uses a semantic Selector because its architecture responsibility is broader than C's deterministic Fast eligibility: D chooses among Fast, ARGO Primary, and Specialist Direct.

---

# 4. S2 — General Agent request

## S2-A — Thin VIA

```text
Context
 -> Intent Refiner
 -> Agent Router
 -> ARGO
```

| Step | Component | Decision responsibility | Mechanism | State | GenAI call | Commit |
| --- | --- | --- | --- | --- | ---: | --- |
| 1 | Context Engine | evidence capture | deterministic | context | 0 | — |
| 2 | Intent Refiner | interpret cleanup goal and planning requirement | `GENAI_DEDICATED` | read context | **1** | — |
| 3 | Agent Router | choose ARGO as eligible general Agent | `GENAI_DEDICATED` | registry/policy | **1** | route chosen |
| 4 | Task/Harness | dispatch ARGO | deterministic | write T1→ARGO | 0 | **ARGO accept** |
| 5 | ARGO | inspect files, plan, execute | `EXECUTOR_INTERNAL` | ARGO execution state | DOMAIN | post-commit |

Expected trace: **2**.

## S2-B — ARGO-centric

```text
thin context -> ARGO -> plan/execute
```

| Step | Component | Decision responsibility | Mechanism | State | GenAI call | Commit |
| --- | --- | --- | --- | --- | ---: | --- |
| 1 | VIA thin context | package evidence | deterministic | evidence package | 0 | — |
| 2 | ARGO | interpret request; choose ARGO direct execution | `GENAI_DEDICATED`, `MIXED` for route ownership | ARGO thread | **1** | **ARGO accepts direct route** |
| 3 | VIA projection | T1↔ARGO execution mapping | deterministic | projection | 0 | already committed |
| 4 | ARGO | domain planning/cleanup workflow | `EXECUTOR_INTERNAL` | ARGO state | DOMAIN | — |

Expected trace: **1**.

## S2-C — Hybrid

```text
Context
 -> Intent Refiner
 -> Fast Eligibility = false
 -> Agent Router
 -> ARGO
```

| Step | Component | Decision responsibility | Mechanism | State | GenAI call | Commit |
| --- | --- | --- | --- | --- | ---: | --- |
| 1 | Context Engine | evidence | deterministic | context | 0 | — |
| 2 | Intent Refiner | normalize goal / planning semantics | `GENAI_DEDICATED` | context | **1** | — |
| 3 | Fast Path Eligibility | reject: requires judgment/planning | `DETERMINISTIC_RULE` against normalized intent + contract | fast contract | 0 | — |
| 4 | Agent Router | select ARGO | `GENAI_DEDICATED` | registry | **1** | route selected |
| 5 | Harness | ARGO accept | deterministic | T1 mapping | 0 | **Commit** |

Expected trace: **2**.

## S2-D — Adaptive

```text
Context
 -> Intent Refiner
 -> Execution Path Selector
 -> ARGO Primary
```

| Step | Component | Decision responsibility | Mechanism | State | GenAI call | Commit |
| --- | --- | --- | --- | --- | ---: | --- |
| 1 | Context Engine | evidence | deterministic | context | 0 | — |
| 2 | Intent Refiner | normalize planning-required goal | `GENAI_DEDICATED` | context | **1** | — |
| 3 | Execution Path Selector | choose `{ARGO_PRIMARY, ARGO}` | `GENAI_DEDICATED` | capability/policy | **1** | route selected |
| 4 | Task/Harness | persist route, ARGO accept | deterministic | T1 route | 0 | **Commit** |

Expected trace: **2**.

---

# 5. S3 — Specialized Agent request

## S3-A — Thin VIA

```text
Context -> Intent Refiner -> Agent Router -> NetworkAgent
```

| Step | Component | Decision responsibility | Mechanism | State | GenAI call | Commit |
| --- | --- | --- | --- | --- | ---: | --- |
| 1 | Context Engine | capture network/context evidence | deterministic | context | 0 | — |
| 2 | Intent Refiner | interpret Wi-Fi diagnosis goal | `GENAI_DEDICATED` | context | **1** | — |
| 3 | Agent Router | select `NetworkAgent` | `GENAI_DEDICATED` after capability filter | registry/health | **1** | route selected |
| 4 | Harness | dispatch/accept NetworkAgent | deterministic | T1→NetworkAgent | 0 | **Commit** |
| 5 | NetworkAgent | diagnose Wi-Fi | `EXECUTOR_INTERNAL` | Agent state | DOMAIN | — |

Expected trace: **2**.

## S3-B — ARGO-centric

```text
thin context -> ARGO -> initial specialist delegation -> NetworkAgent
```

| Step | Component | Decision responsibility | Mechanism | State | GenAI call | Commit |
| --- | --- | --- | --- | --- | ---: | --- |
| 1 | VIA thin context | package context | deterministic | evidence | 0 | — |
| 2 | ARGO | interpret Wi-Fi request and decide initial delegation to NetworkAgent | `GENAI_DEDICATED`, QA-04 `MIXED` | ARGO thread/route | **1** | delegation decision made |
| 3 | ARGO delegation adapter | dispatch NetworkAgent | deterministic | ARGO execution mapping | 0 | **Commit when NetworkAgent accepts** |
| 4 | VIA task projection | map T1 to ARGO execution and downstream route | deterministic | T1↔E3 | 0 | — |
| 5 | NetworkAgent | domain diagnosis | `EXECUTOR_INTERNAL` | specialist state | DOMAIN | — |

Expected trace: **1**.

This is the critical hidden-routing case: the ARGO delegation inference remains before Execution Route Commit and is therefore visible to QA-04.

## S3-C — Hybrid

```text
Context -> Intent Refiner -> Fast Eligibility=false -> Agent Router -> NetworkAgent
```

| Step | Component | Decision responsibility | Mechanism | State | GenAI call | Commit |
| --- | --- | --- | --- | --- | ---: | --- |
| 1 | Context Engine | evidence | deterministic | context | 0 | — |
| 2 | Intent Refiner | interpret diagnosis goal | `GENAI_DEDICATED` | context | **1** | — |
| 3 | Fast Eligibility | reject: domain diagnosis/planning required | deterministic contract check | fast contract | 0 | — |
| 4 | Agent Router | choose NetworkAgent | `GENAI_DEDICATED` | registry | **1** | route selected |
| 5 | Harness | NetworkAgent accept | deterministic | T1 mapping | 0 | **Commit** |

Expected trace: **2**.

## S3-D — Adaptive

```text
Context -> Intent Refiner -> Execution Path Selector -> Specialist Direct / NetworkAgent
```

| Step | Component | Decision responsibility | Mechanism | State | GenAI call | Commit |
| --- | --- | --- | --- | --- | ---: | --- |
| 1 | Context Engine | evidence | deterministic | context | 0 | — |
| 2 | Intent Refiner | interpret diagnosis goal | `GENAI_DEDICATED` | context | **1** | — |
| 3 | Execution Path Selector | choose `{SPECIALIST_DIRECT, NetworkAgent}` | `GENAI_DEDICATED` | capability/health/policy | **1** | route selected |
| 4 | Task/Harness | persist route and dispatch NetworkAgent | deterministic | T1.ExecutionRoute | 0 | **Commit on accept** |

Expected trace: **2**.

---

# 6. S4 — Existing-task Follow-up

Precondition shared by all alternatives:

```text
Task T1 exists.
The initial Wi-Fi analysis route was already committed.
The follow-up clearly refers to T1.
```

The critical Base rule is **route reuse for a clear continuation**.

## S4-A — Thin VIA

```text
Follow-up
 -> Context Engine
 -> Intent Refiner
 -> Task Manager resolves T1
 -> previous Agent route reuse
 -> existing NetworkAgent/ARGO execution continuation
```

| Step | Component | Decision responsibility | Mechanism | State | GenAI call | Commit |
| --- | --- | --- | --- | --- | ---: | --- |
| 1 | Context Engine | current context evidence | deterministic | context | 0 | — |
| 2 | Intent Refiner | interpret “그럼 DNS도…” as continuation/subgoal | `GENAI_DEDICATED` | read conversation/T1 hints | **1** | — |
| 3 | Task Manager | resolve FOLLOW_UP to T1 and previous executor | `DETERMINISTIC_RULE` | read T1 route/thread mapping | 0 | **previous route re-committed/reused for this turn** |
| 4 | Harness | continue existing Agent execution/thread | deterministic protocol action | T1 | 0 | — |
| 5 | Agent | domain DNS diagnosis | `EXECUTOR_INTERNAL` | existing Agent state | DOMAIN | — |

**Agent Router is not called again.**

Expected follow-up trace: **1**.

## S4-B — ARGO-centric

```text
Follow-up
 -> VIA T1↔ARGO mapping
 -> ARGO existing execution/thread
 -> ARGO interprets follow-up in that thread
 -> continue domain work
```

| Step | Component | Decision responsibility | Mechanism | State | GenAI call | Commit |
| --- | --- | --- | --- | --- | ---: | --- |
| 1 | VIA task projection | recover T1→ARGO Execution E17 mapping | deterministic | read mapping | 0 | candidate existing route |
| 2 | ARGO existing thread | interpret follow-up semantics and confirm it belongs to E17 | `GENAI_DEDICATED`; may be `MIXED` because B owns request interpretation and route-continuation authority | read/write E17 thread | **1** | **existing ARGO route confirmed for follow-up** |
| 3 | VIA projection | keep T1 correlated with E17 | deterministic | projection | 0 | — |
| 4 | ARGO | domain DNS reasoning | `EXECUTOR_INTERNAL` after continuation commit | E17 | DOMAIN | — |

Expected follow-up trace: **1**.

B does not create a VIA Agent Router decision for the follow-up. The user-facing task mapping identifies the candidate execution, while ARGO remains authoritative for interpreting/continuing its execution thread.

## S4-C — Hybrid

Same continuation rule as A.

| Step | Component | Decision responsibility | Mechanism | State | GenAI call | Commit |
| --- | --- | --- | --- | --- | ---: | --- |
| 1 | Context Engine | evidence | deterministic | context | 0 | — |
| 2 | Intent Refiner | interpret clear continuation | `GENAI_DEDICATED` | conversation/T1 hints | **1** | — |
| 3 | Task Manager | resolve T1 and previous route | deterministic | T1 mapping | 0 | **reuse existing route** |
| 4 | Existing executor | continue task | executor internal | existing state | DOMAIN | — |

Fast Path Eligibility and Agent Router are not rerun for a continuation whose executor is already established.

Expected follow-up trace: **1**.

## S4-D — Adaptive

```text
Follow-up
 -> Context Engine
 -> Intent Refiner
 -> Task Manager resolves T1
 -> Previous ExecutionRoute reuse
 -> existing executor
```

| Step | Component | Decision responsibility | Mechanism | State | GenAI call | Commit |
| --- | --- | --- | --- | --- | ---: | --- |
| 1 | Context Engine | evidence | deterministic | context | 0 | — |
| 2 | Intent Refiner | interpret clear continuation | `GENAI_DEDICATED` | conversation/task hints | **1** | — |
| 3 | Task Manager | resolve T1 and stored ExecutionRoute | `DETERMINISTIC_RULE` | read T1.ExecutionRoute | 0 | **reuse previous route** |
| 4 | selected existing executor | continue task/thread | executor internal | executor state | DOMAIN | — |

**Execution Path Selector is not called.**

Expected follow-up trace: **1**.

This is not a speed optimization. Running the Selector again would risk changing ownership of an ongoing logical task and breaking continuity semantics.

---

# 7. S5 — Ambiguous Referent / Clarification

Precondition:

```text
Document A and Document B are both plausible.
Execution before clarification is forbidden.
User clarification: "오른쪽에 있는 거."
```

## S5-A — Thin VIA

```text
Intent Refiner #1 -> ambiguity / clarification
user answer
Intent Refiner #2 -> referent resolved
Agent Router -> ARGO
```

| Step | Component | Decision responsibility | Mechanism | State | GenAI call | Clarification | Commit |
| --- | --- | --- | --- | --- | ---: | --- | --- |
| 1 | Context Engine | capture candidate documents/anchors | deterministic | context | 0 | — | — |
| 2 | Intent Refiner | detect ambiguity; request clarification | `GENAI_DEDICATED` | read context | **1** | **ask which document** | no route |
| 3 | Context/Conversation | record reply "오른쪽에 있는 거" | deterministic | append turn/context | 0 | reply | — |
| 4 | Intent Refiner | resolve right-side document and normalize open action | `GENAI_DEDICATED` | read updated context | **1** | resolved | — |
| 5 | Agent Router | choose ARGO/general Agent under A | `GENAI_DEDICATED` | registry | **1** | no | route selected |
| 6 | Harness | dispatch/ARGO accept | deterministic | T1 mapping | 0 | no | **Commit** |

Expected trace: **3**.

## S5-B — ARGO-centric

```text
ARGO #1 -> ambiguity / clarification
user answer
ARGO #2 -> resolve referent + commit direct ARGO route
```

| Step | Component | Decision responsibility | Mechanism | State | GenAI call | Clarification | Commit |
| --- | --- | --- | --- | --- | ---: | --- | --- |
| 1 | VIA thin context | package both candidates | deterministic | evidence | 0 | — | — |
| 2 | ARGO | interpret request and detect ambiguity | `GENAI_DEDICATED` | E/thread context | **1** | **asks clarification** | no route commit |
| 3 | VIA interaction bridge | deliver clarification answer to same ARGO interaction | deterministic | conversation/projection | 0 | reply | — |
| 4 | ARGO | resolve referent and choose direct ARGO/tool execution | `GENAI_DEDICATED`, `MIXED` for initial route decision | ARGO thread/route | **1** | resolved | **Commit when ARGO accepts direct execution** |
| 5 | VIA projection | bind result to T1 | deterministic | mapping | 0 | — | — |

Expected trace: **2**.

## S5-C — Hybrid

```text
Intent Refiner #1 -> clarify
Intent Refiner #2 -> resolve
Fast Eligibility -> VIA Fast open-document
```

| Step | Component | Decision responsibility | Mechanism | State | GenAI call | Clarification | Commit |
| --- | --- | --- | --- | --- | ---: | --- | --- |
| 1 | Context Engine | candidates/evidence | deterministic | context | 0 | — | — |
| 2 | Intent Refiner | detect ambiguity | `GENAI_DEDICATED` | context | **1** | **ask** | — |
| 3 | Conversation/context | record right-side answer | deterministic | context | 0 | reply | — |
| 4 | Intent Refiner | resolve document + normalize bounded open action | `GENAI_DEDICATED` | context | **1** | resolved | — |
| 5 | Fast Eligibility | match local open-document capability + policy | deterministic | fast contract | 0 | no | route selected |
| 6 | VIA Fast Path | start open action | local executor | local state | 0 | no | **Commit** |

Expected trace: **2**.

## S5-D — Adaptive

```text
Intent Refiner #1 -> clarify
Intent Refiner #2 -> resolve
Execution Path Selector -> VIA Fast
```

| Step | Component | Decision responsibility | Mechanism | State | GenAI call | Clarification | Commit |
| --- | --- | --- | --- | --- | ---: | --- | --- |
| 1 | Context Engine | capture candidates | deterministic | context | 0 | — | — |
| 2 | Intent Refiner | detect ambiguity | `GENAI_DEDICATED` | context | **1** | **ask** | — |
| 3 | Conversation/context | capture reply | deterministic | context | 0 | reply | — |
| 4 | Intent Refiner | resolve right-side document and goal | `GENAI_DEDICATED` | context | **1** | resolved | — |
| 5 | Execution Path Selector | choose `{VIA_FAST, local_open_document}` | `GENAI_DEDICATED` | capability/policy | **1** | no | route selected |
| 6 | Task/Fast executor | persist route and start action | deterministic | T1 route | 0 | no | **Commit** |

Expected trace: **3**.

---

# 8. Pre-experiment expected logical Generative Model Call trace

The walkthroughs produce the requested Base reference trace:

| Scenario | A | B | C | D |
| --- | ---: | ---: | ---: | ---: |
| **S1 Local** | **2** | **1** | **1** | **2** |
| **S2 General Agent** | **2** | **1** | **2** | **2** |
| **S3 Specialized** | **2** | **1** | **2** | **2** |
| **S4 Follow-up** | **1** | **1** | **1** | **1** |
| **S5 Clarification + Local** | **3** | **2** | **2** | **3** |

Label for all uses of this table:

> **Pre-experiment specification walkthrough — Expected logical Generative Model Call trace — Not measured benchmark result.**

## Why the counts have this shape

- **A**: new work normally uses dedicated Intent Refiner + semantic Agent Router; clear follow-up reuses the existing Agent route and skips Router.
- **B**: ARGO owns initial interpretation plus self-vs-specialist execution choice in its primary runtime responsibility, so the initial route can usually be committed with one ARGO generation; clarification requires another generation after the user answer.
- **C**: Intent Refiner is the semantic stage; Fast eligibility is deterministic. Non-Fast new work then adds Agent Router. This makes S1/S5 lower than A while S2/S3 remain A-like.
- **D**: new work uses Intent Refiner + semantic Execution Path Selector. Clear follow-up reuses the stored route and skips Selector. Clarification causes a second Intent Refiner generation before the Selector.

These counts are a structural prediction of the specified Base mechanisms, not proof of performance or correctness.

---

# 9. Architecture Neutrality Stress-test

## 9.1 동일 Product Capability인가?

**PASS — specification readiness.**

A/B/C/D can all satisfy S1~S5 and the common Product Obligations. The route differs; the requested capability does not.

## 9.2 B가 더 많은 capability를 가진 구조인가?

**NO.**

B uses the same ARGO/Specialized Agent capability set available to the other alternatives. The distinction is that ARGO is the primary reasoning/execution authority rather than one Agent chosen by VIA.

## 9.3 C만 local 기능이 있는가?

**NO at the product-capability level.**

The user-visible action is common. A/B can perform the same action through their Agent/ARGO path. C's unique property is **VIA ownership of bounded local execution**, not exclusive product capability.

## 9.4 D는 A + 불필요한 Selector인가?

**NO.**

A's top-level question is:

```text
Which Agent?
```

D's is:

```text
Which execution topology + top-level executor?
  VIA Fast
  ARGO Primary
  Specialist Direct
```

D can choose execution ownership classes that A's Agent Router does not represent.

## 9.5 특정 Alternative에만 optimization Tactic이 적용됐는가?

**PASS — no optional tactic is used in the Base walkthrough.**

Not used:

- cross-component GenAI fusion;
- ML classifier/embedding routing;
- speculative routing/execution;
- parallel inference optimization;
- prompt/cache optimization;
- partial-ASR semantic pre-routing;
- semantic precomputation.

## 9.6 Component-count fairness

**PASS.**

No call count is inferred from component count. Deterministic components contribute zero Generative Model Calls; semantic generations are counted explicitly.

## 9.7 Follow-up continuity

**PASS.**

A/C skip Agent Router for a clear continuation. D skips Execution Path Selector and reuses the stored route. B uses VIA task projection + ARGO execution/thread mapping rather than constructing a new VIA routing decision.

## 9.8 Execution Route Commit neutrality

**PASS.**

Initial specialist delegation inside ARGO remains inside the route-commit boundary. Later executor-internal sub-task delegation after commit remains DOMAIN behavior. This matches QA-04's intended measurement responsibility rather than component location.

---

# 10. Structural risk hypotheses exposed by the walkthrough

| Alternative | Walkthrough-visible risk hypothesis |
| --- | --- |
| **A** | Repeated dedicated VIA semantic stages can create latency/model-call pressure; if Intent Refiner grows into domain semantics, VIA can become Agent-like. |
| **B** | User-facing task projection and ARGO-authoritative execution state require a clean mapping; ARGO coupling is structurally broad. |
| **C** | Fast eligibility is clean only while Fast Capability Contract remains bounded; capability creep can expand VIA execution ownership. |
| **D** | Execution Path Selector owns a broad semantic decision surface and must coexist with task-state route reuse without contradictory ownership. |

These are **pre-experiment hypotheses** to be tested, not observed failures.

---

# 11. Base vs Tactic stress-test

The walkthrough intentionally preserves unfused independent responsibilities.

Example A:

```text
Intent Refiner GenAI
 -> normalized goal
Agent Router GenAI
 -> Agent
```

A later tactic may test:

```text
Fused Semantic Decision GenAI
 -> normalized goal + Agent
```

but this is not silently used in Base because it changes more than call count:

- tighter semantic coupling;
- reduced independent replaceability;
- broader prompt/schema change surface;
- larger validation blast radius;
- potentially lower QA-01/QA-04 cost.

The same principle applies to D's Intent Refiner + Execution Path Selector.

C's deterministic Fast eligibility is not a tactic; it is part of C's architecture because the bounded capability-placement boundary is C's defining topology.

B's combined ARGO interpretation/self-vs-specialist decision is also not an optional fusion tactic: B intentionally places those responsibilities inside one primary execution runtime.

---

# 12. Walkthrough verdict

```text
Common Product Capability                PASS
Responsibility ownership completeness    PASS
State-ownership coherence                PASS
Follow-up continuity                     PASS
Execution Route Commit observability     PASS
Base-vs-Tactic separation                PASS
Model-call accounting consistency        PASS
Architecture neutrality                  PASS
20 executable scenario paths             PASS
```

Final checkpoint verdict:

> **Executable Architecture Structure = PASS**

This means **prototype specification readiness only**. It does not mean any A/B/C/D alternative passed QA evaluation.

---

# 13. Next contract required before prototype

The next checkpoint should define:

```text
Canonical Benchmark Scenario Contract
+ Common vs Variable Experimental Boundary
```

At minimum it should freeze semantics for:

- common `ScenarioInput` / fixture contract;
- common canonical architecture events;
- semantic replay/model-adapter contract;
- Agent/tool stub contract;
- Product Capability fixtures shared across A/B/C/D;
- alternative-specific module boundary;
- exact measurement events for QA-01/02/04;
- evolution baseline/mapping inputs for QA-03;
- which fields/mechanisms are common test harness vs architecture-under-test.

Only after that boundary is explicit should implementation begin, so shared benchmark infrastructure does not accidentally erase or add architecture differences.
