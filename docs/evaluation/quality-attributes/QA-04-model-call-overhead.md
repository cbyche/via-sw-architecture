# QA-04 — 모델 호출 오버헤드 (Model Call Overhead)

## Status

**Proposed vNext — QA definition agreed, scoring thresholds TBD**

This document is part of the QA rebaseline work. It does not replace the Approved Baseline QA numbering yet. `docs/requirements/requirements-v1.1.md` remains unchanged until a coherent vNext rebaseline is reviewed.

## Name

**QA-04 모델 호출 오버헤드 (Model Call Overhead)**

## ISO quality characteristic

**Performance Efficiency / Resource Utilization**

## 핵심 질문

> **사용자 요청의 최종 실행 경로를 확정하기까지, SW 구조가 몇 번의 AI 모델 호출을 요구하는가?**

This QA evaluates AI inference generations required by the architecture to commit the request to its final domain execution route.

It does **not** evaluate how many reasoning calls the final execution owner later needs to complete the domain task.

## Primary Metric

### 평균 실행경로 결정 모델 호출 수 — Average Model Calls to Commit Execution Route

Plain-text definition for one evaluation episode:

```text
실행경로 결정 모델 호출 수 =
  사용자 요청 처리를 시작한 시점부터
  Execution Route Commit 시점까지 발생한
  ORCHESTRATION 또는 MIXED logical model call 수
```

Overall QA-04 Primary Metric:

```text
QA-04 = 평가 대상 episode들의 실행경로 결정 모델 호출 수 평균
```

Equivalent English definition:

```text
Average Model Calls to Commit Execution Route =
  mean(count of ORCHESTRATION or MIXED logical model generations
       attributable to each eligible episode
       from request-processing start
       through the Execution Route Commit,
       including the generation that causes the commit)
```

- Unit: **model calls / episode**
- Direction: **lower is better**
- Architecture score: one 0–5 score derived from this metric using a frozen scoring version
- Primary Metric rule: **this metric alone determines the QA-04 0–5 score**

## Measurement start boundary

The measurement begins at:

```text
request_processing_start_ts
```

This is the benchmark-designated point when the architecture begins processing the user goal for execution-decision purposes.

For voice scenarios this is not automatically identical to Acoustic End-of-Speech. If an alternative intentionally begins semantic processing from partial/streaming input, causally attributable model generations remain inside QA-04 rather than disappearing because they happened before EOS.

For text scenarios it normally corresponds to the submitted request becoming available to the architecture.

Raw data keeps the QA-01 `acoustic_eos_ts` so pre-EOS inference can be correlated with responsiveness without redefining QA-04's start boundary.

## Measurement end boundary — Execution Route Commit

The authoritative QA-04 end boundary is **실행 경로 확정 (Execution Route Commit)**.

Definition:

> **실제 domain 작업을 수행할 최종 실행 경로가 결정되어, 이후에는 해당 user request에 대해 추가적인 execution-owner 선택이나 delegation 판단 없이 domain execution을 계속할 수 있게 된 최초 시점.**

Plain English:

> The earliest point at which the final domain execution route is operationally committed, such that no further execution-owner selection or delegation decision is required for that request before domain execution can continue.

Raw authoritative timestamp:

```text
execution_route_commit_ts
```

Related route fields:

```text
execution_route
final_execution_owner
delegated_agent_if_any
```

Existing timestamps remain preserved for diagnostics:

```text
execution_owner_confirmed_ts
execution_started_ts
```

They are no longer the authoritative QA-04 end boundary.

### Why owner confirmation / execution start was insufficient

An intermediate runtime may temporarily become the current owner and only later decide whether to delegate the request.

Example:

```text
User
  -> ARGO execution begins
  -> ARGO model inference: "delegate to NetworkAgent"
  -> NetworkAgent accepts
```

If QA-04 stopped when ARGO first started, the delegation inference would disappear from the metric even though it performs the same architectural responsibility as an explicit Router inference in another topology.

Execution Route Commit normalizes this responsibility independent of where it is placed.

## Route-commit examples

### Thin VIA

```text
Intent
  -> Router
  -> NetworkAgent selected
  -> NetworkAgent accepts
                     ^ Execution Route Commit
```

All ORCHESTRATION/MIXED logical generations required to reach that accepted final route are included.

### ARGO-centric — ARGO executes directly

```text
ARGO inference
  -> "I will execute this task directly"
       ^ Execution Route Commit
```

If that same generation also performs substantive domain reasoning, classify it as `MIXED` and count it once.

### ARGO-centric — specialist delegation

```text
ARGO starts processing
  -> ARGO inference decides NetworkAgent delegation
  -> NetworkAgent accepts
                     ^ Execution Route Commit
```

The ARGO delegation-decision generation is inside QA-04 and must be counted as ORCHESTRATION or MIXED according to its content.

### VIA Fast Path

```text
deterministic eligibility
  -> local capability selected
  -> local execution starts
                    ^ Execution Route Commit
```

If no model generation is required, QA-04 call count can be `0`.

## Primary inclusion rule

Every logical model call is classified as:

```text
ORCHESTRATION
DOMAIN
MIXED
```

A call is included in QA-04 Primary when:

```text
call_class in {ORCHESTRATION, MIXED}
AND
(
  the call occurs before Execution Route Commit
  OR the call is the generation whose output causes Execution Route Commit
)
```

A DOMAIN-only call is excluded from Primary.

An ORCHESTRATION/MIXED call after route commit is also excluded because the route has already been finalized for the measured request.

## Why Fast-task restriction is not used

An earlier candidate was “average model-call overhead per Fast-task.” That restriction is removed.

Once downstream DOMAIN reasoning is excluded and measurement ends at Execution Route Commit, task duration itself no longer dominates the metric.

Example:

```text
Short Task
User -> Intent -> Router -> ARGO final route
QA-04 = 2 calls

Long Task
User -> Intent -> Router -> ARGO final route
QA-04 = 2 calls

ARGO performs 30 later ReAct generations
QA-04 remains 2 calls
```

Therefore QA-04 may include representative short and long tasks. Duration is not a population exclusion criterion.

This differs intentionally from QA-01, where long task execution duration would dominate the user-visible latency metric.

## Logical Model Call definition

A **Logical Model Call** is one new logical inference generation that begins producing a new model output.

Plain-text counting rules:

```text
Realtime connection/session creation      = 0
One streaming generation                  = 1
Number of streaming token/audio chunks    = irrelevant
New generation after tool/result feedback = +1
Actual new retry generation                = +1
```

The metric does not count HTTP requests, websocket frames, token chunks, audio chunks or provider transport messages.

A retry counts only when it starts a genuinely new logical generation.

## Model Call Classification

### ORCHESTRATION

Architecture-level inference used to establish or commit the final execution route.

Examples:

- intent refinement;
- referent reasoning;
- execution-path selection;
- Agent routing;
- model-based validation;
- clarification decision;
- task/result association when needed to choose the route;
- execution-owner selection;
- delegation decision;
- architecture-level retry/fallback decision.

### DOMAIN

Substantive reasoning for task execution after the final route is committed.

Examples:

- NetworkAgent root-cause analysis;
- MailAgent content analysis;
- report-generation reasoning;
- Downstream Agent domain planning;
- Agent ReAct loop;
- tool-result interpretation/replanning after route commit.

### MIXED

One generation performs both architecture-level route/owner/delegation decision and substantive domain reasoning.

Example:

```text
ARGO reasons about the request while also deciding
whether ARGO executes it or delegates to another Agent.
```

This is counted once, not split into artificial sub-calls.

If its output commits the route, it is included even if part of the generation contains domain reasoning.

## Pure Voice / S2S model calls

A model call that performs only voice generation or non-decision interaction output is excluded from the QA-04 Primary Metric.

Example:

```text
S2S generation used only to produce spoken output -> excluded
```

However, when the same inference also performs intent interpretation, execution-path decision, routing, validation, delegation or owner selection, it is classified as `MIXED` and included when it is before or causes Execution Route Commit.

The metric intentionally removes a common voice-generation baseline and focuses on **model inference required by execution-route topology**.

## Architecture Qualification with deterministic replay

Architecture scoring continues to use controlled semantic replay/test doubles where needed.

A critical accounting rule is:

> **A logical model call is still counted when the architecture initiates a model generation but the benchmark fulfills it through deterministic replay.**

The replay controls the output; it does not erase the architecture's requirement for an inference generation.

```text
architecture requests model generation
  -> logical ModelCall recorded
  -> frozen output replayed
  -> inclusion decided by class + Execution Route Commit boundary

architecture uses deterministic code only
  -> no logical model call
```

This lets QA-04 measure architecture-required inference stages while keeping stochastic model behavior controlled.

## Why “original/intrinsic required model calls” is not used

The rejected candidate was:

```text
architecture-added calls = total calls - intrinsic task calls
```

There is no neutral way to define the task's “intrinsic” number of calls.

The same request can use:

- one large fused model generation;
- several small-model stages;
- deterministic classification plus one model;
- one multimodal generation;
- multiple specialist models.

The baseline itself would encode an architecture/model-design preference.

QA-04 therefore measures observed route-commit decision calls directly and performs no hypothetical subtraction.

## Why total model calls over the task lifetime are not used

Whole-lifetime model-call count is dominated by downstream domain complexity, especially for long-running work:

- planning;
- ReAct iteration;
- tool-result processing;
- replanning;
- downstream retry;
- workflow-specific reasoning.

That would primarily measure the task/Agent rather than the DP-00 execution topology.

Total Model Calls / Episode remains a Secondary Metric only.

## Why Dollar/Token Cost is not the Primary Metric

Monetary/token cost is affected by many variables outside the architecture topology:

- future on-device inference;
- provider pricing changes;
- input/output pricing differences;
- prompt size;
- prompt optimization;
- context pruning/compression;
- prompt caching policy;
- cache hit rate;
- provider cached-token discount policy;
- tokenizer differences;
- model size/quantization/runtime.

Conceptually:

```text
Cost
= Architecture
+ Model Choice
+ Prompt Engineering
+ Cache Optimization
+ Provider Pricing
```

Therefore monetary cost is not used as the architecture-only Primary Metric.

Token/cache/resource data is still retained in raw telemetry for later derived analysis.

## Why arbitrary model-category weights are not used

A small classifier generation and a large multimodal generation do not have equal physical compute cost.

QA-04's logical call metric **does not claim physical compute equivalence**.

However, arbitrary weights such as:

```text
small model = 0.2
large model = 3.0
```

would reintroduce assumptions about model size, hardware, prompt length, cache state, quantization and runtime.

Therefore each qualifying logical generation contributes one call to the Primary Metric.

If a future weighted metric is needed, it should use measured and versioned model-profile resource cost rather than hand-authored category weights.

## Workload population

QA-04 uses request classes that exercise execution topology rather than task duration.

Candidate workload taxonomy:

### W1 — Local-capable request

Can legitimately execute through a bounded VIA/local path in alternatives that support it.

### W2 — General Agent request

Requires a general execution Agent/runtime.

### W3 — Specialized Agent request

Requires or allows specialized Agent delegation/routing.

### W4 — Existing-task follow-up request

Requires execution-state/task continuation logic before the final route is committed.

Exact taxonomy and class proportions are **TBD**.

Because an arithmetic mean is sensitive to scenario mix, final benchmark qualification must freeze:

- workload taxonomy/version;
- class scenario counts;
- episode eligibility;
- aggregation rule;
- semantic replay mix;
- model/prompt/cache profiles;
- scoring version.

If Pilot evidence shows that one request class dominates the overall mean, a class-level macro-average remains a candidate aggregation rule. Any such change must be selected and frozen before final A/B/C/D evaluation.

Usage-frequency-weighted results may be calculated as Secondary sensitivity analysis.

## Episode eligibility and correctness

QA-04 should not reward an architecture for failing before committing a valid route and therefore producing an artificially low count.

A complete Primary observation requires a valid `execution_route_commit_ts` and route evidence.

Episodes that never reach Execution Route Commit remain raw failure/non-completion evidence and are reported beside the QA-04 population, but are not converted into an arbitrary model-call penalty.

Correctness is evaluated separately in QA-02. A low call count does not compensate for a wrong route.

A useful Secondary slice is:

```text
Calls per exact-conform episode
```

## Stress-test interpretation

The agreed rules produce the intended results for the following cases:

```text
1. Short/Long tasks with identical route-decision topology
   -> same QA-04 Primary count

2. Long task with 30–40 downstream ReAct calls after route commit
   -> later DOMAIN calls excluded

3. Deterministic selector vs LLM selector
   -> 0 vs 1 call

4. intent + routing + validation as separate generations vs one fused generation
   -> 3 vs 1 calls

5. ARGO domain + delegation decision in one inference
   -> MIXED, counted once

6. Common pure voice generation
   -> excluded

7. Specialized Agent internal reasoning after route commit
   -> excluded

8. New logical retry generation before route commit
   -> +1

9. Different physical model costs
   -> raw resource telemetry retained, no arbitrary Primary weight

10. ARGO starts, then later uses a model call to choose a specialist
    -> delegation call remains inside QA-04 until final route commit
```

These cases demonstrate that the metric observes architecture topology rather than downstream workload size or the physical location of routing logic.

## Quality Attribute Scenario

| 항목 | 정의 |
| --- | --- |
| **Source** | Samsung PC 사용자 |
| **Stimulus** | VIA가 처리해야 하는 representative user request |
| **Environment** | 동일 scenario, frozen semantic replay / scripted dependency, fixed model/prompt/cache profiles |
| **Artifact** | DP-00 Alternative의 VIA Integrated Product SW architecture |
| **Response** | 요청을 해석하고 추가 owner/delegation 판단이 필요 없는 최종 execution route를 commit |
| **Response Measure** | **평균 실행경로 결정 모델 호출 수 (Average Model Calls to Commit Execution Route)** |

## Secondary / Backup Metrics

The following do not determine the QA-04 score but must remain recomputable from raw telemetry:

- Total Model Calls / Episode;
- ORCHESTRATION call count;
- MIXED call count;
- DOMAIN call count;
- Critical-path Model Calls;
- model calls by purpose;
- retry count;
- input tokens;
- output tokens;
- cached/uncached input tokens;
- context bytes;
- cache hit rate;
- prompt profile/version;
- model profile/version;
- model inference latency;
- CPU/GPU/NPU time;
- peak memory;
- energy if measurable;
- provider monetary cost if available;
- Calls per exact-conform episode;
- request-class call distributions/means;
- usage-frequency-weighted sensitivity result;
- time from intermediate owner confirmation/start to final Execution Route Commit;
- number of owner/delegation decisions after intermediate execution start.

Backup Primary candidates, if future evidence requires a new scoring methodology:

- Critical-path Inference Time / Episode;
- Measured Normalized Inference Work / Episode;
- Uncached Tokens / Episode;
- On-device Accelerator Time / Episode.

A future Primary change requires a documented rationale and a new scoring version applied equally to all alternatives.

## Scoring

QA-04 uses six score bands: **0, 1, 2, 3, 4, 5**.

Threshold values are currently **TBD**. Lower is better.

```text
Metric <= I5        -> 5점
I5 < x <= I4        -> 4점
I4 < x <= I3        -> 3점
I3 < x <= I2        -> 2점
I2 < x <= I1        -> 1점
x > I1              -> 0점
```

Required process:

```text
Pilot
 -> metric behavior 확인
 -> threshold / aggregation calibration
 -> scoring-v1 freeze
 -> final A/B/C/D evaluation
```

Thresholds must not be adjusted after seeing final alternative results merely to favor a preferred topology.

## Raw-data requirements

Per-model-call raw telemetry is defined in:

`benchmark/schemas/model-call-schema.md`

The runtime episode schema is connected through:

`benchmark/schemas/run-event-schema.md`

At minimum, raw evidence must allow reconstruction of:

```text
request_processing_start_ts
acoustic_eos_ts

execution_route_commit_ts
execution_route
final_execution_owner
delegated_agent_if_any

execution_owner_confirmed_ts
execution_started_ts

route_commit_orchestration_call_count
route_commit_mixed_call_count
route_commit_total_primary_call_count

total_orchestration_call_count
total_domain_call_count
total_mixed_call_count
total_model_call_count

exact_conformance
```

Existing owner-confirmation/execution-start timestamps remain useful diagnostics. They no longer define QA-04's authoritative end boundary.

Per-call ModelCall events are the source of truth. Episode-level counts should be derived/projection fields where possible rather than the only evidence.

## Relationship to other QAs and DP-00

### QA-01

QA-01 measures **observed user-visible useful-outcome latency** for Fast tasks. QA-04 measures **structural AI decision dependency count required to commit the final execution route**.

A topology can use fewer calls yet have poor latency because one call is expensive, or use more calls but run them cheaply/parallel. The two QAs are related but not equivalent.

### QA-02

QA-02 evaluates interaction/orchestration correctness. QA-04 must not reward an incorrect zero/low-call route as architecturally desirable. The trade-off is read across separate scores and diagnostic slices.

### QA-03

QA-03 evaluates change containment. A fused low-call architecture may reduce QA-04 overhead while increasing coupling and lowering flexibility; decomposed stable seams may show the opposite trade-off.

### DP-00

DP-00 alternatives differ directly in route-commit AI decision topology:

- Thin VIA can introduce explicit intent/routing stages before specialist route commit;
- ARGO-centric execution can fuse first substantive reasoning and owner/delegation choice, but any later specialist-delegation inference still counts until route commit;
- Hybrid Fast Path can use deterministic or model-based eligibility before local/Agent route commit;
- Adaptive Per-turn execution can add a selector but may avoid later hops.

QA-04 therefore measures one architecture consequence of execution-capability placement without assuming which topology should win.

## Unresolved / TBD

- `I1`~`I5` score thresholds;
- final W1~W4 taxonomy and scenario counts;
- final episode aggregation rule if macro-average is selected after Pilot;
- exact eligibility handling for rare aborted/non-committed episodes beyond retaining them outside complete Primary observations;
- concrete serialization format for model-call telemetry;
- final model/prompt/cache profile set for controlled qualification;
- final QA numbering/rebaseline after QA-01~QA-04 integration.
