# QA-04 — 모델 호출 오버헤드 (Model Call Overhead)

## Status

**Proposed vNext — QA definition agreed, scoring thresholds TBD**

This document is part of the QA rebaseline work. It does not replace the Approved Baseline QA numbering yet. `docs/requirements/requirements-v1.1.md` remains unchanged until a coherent vNext rebaseline is reviewed.

## Name

**QA-04 모델 호출 오버헤드 (Model Call Overhead)**

## ISO quality characteristic

**Performance Efficiency / Resource Utilization**

## 핵심 질문

> **사용자 요청을 처리할 실행 주체를 결정하고 실제 실행을 시작시키기까지, SW 구조가 몇 번의 AI 모델 호출을 요구하는가?**

This QA evaluates AI inference generations required by the architecture's execution-decision topology before substantive execution begins.

It does **not** evaluate how many reasoning calls a Downstream Agent later needs to complete the domain task.

## Primary Metric

### 평균 실행 전 모델 호출 수 — Average Pre-execution Model Calls per Episode

Plain-text definition for one evaluation episode:

```text
실행 전 모델 호출 수 =
  사용자 요청 처리 시작부터
  해당 요청의 execution owner가 확정되고 실제 실행이 시작될 때까지 발생한
  ORCHESTRATION 또는 MIXED logical model call 수
```

Overall QA-04 Primary Metric:

```text
QA-04 = 평가 대상 episode들의 실행 전 모델 호출 수 평균
```

Equivalent English definition:

```text
Average Pre-execution Model Calls per Episode =
  mean(count of ORCHESTRATION or MIXED logical model generations
       attributable to each eligible episode
       from request-processing start
       through confirmed execution start)
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

For voice scenarios this is not automatically identical to Acoustic End-of-Speech. If an alternative intentionally begins semantic processing from partial/streaming input, those causally attributable model generations remain inside QA-04 rather than disappearing because they happened before EOS.

For text scenarios it normally corresponds to the submitted request becoming available to the architecture.

## Measurement end boundary

QA-04 does **not** count model calls across the whole task lifetime.

The measurement ends when:

```text
1. execution owner is confirmed
AND
2. that owner has actually started or accepted execution
```

Raw timestamps:

```text
execution_owner_confirmed_ts
execution_started_ts
```

The episode boundary closes at `execution_started_ts` once ownership has been confirmed.

### Downstream Agent delegation

```text
request
  -> orchestration/routing
  -> Downstream Agent selected
  -> dispatch/accept
  -> Agent execution starts   <-- QA-04 end
```

After this point, the Agent's domain planning, ReAct loops, tool interpretation and retries are excluded from the Primary Metric.

### VIA Fast Path

```text
request
  -> eligibility/decision
  -> VIA local capability selected
  -> local action starts      <-- QA-04 end
```

### ARGO-centric topology

```text
request
  -> thin realtime/context path
  -> ARGO becomes execution owner
  -> ARGO task execution starts   <-- QA-04 end
```

If ARGO's generation itself combines task understanding with the ownership/delegation decision before execution begins, that generation is classified as `MIXED` and counted once.

## Why Fast-task restriction is not used

An earlier candidate was “average model-call overhead per Fast-task.” That restriction is removed.

Once downstream DOMAIN reasoning is excluded and measurement ends at execution start, task duration itself no longer dominates the metric.

Example:

```text
Short Task
User -> Intent -> Router -> ARGO dispatch
QA-04 = 2 calls

Long Task
User -> Intent -> Router -> ARGO dispatch
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

Every logical model call is classified as exactly one of:

```text
ORCHESTRATION
DOMAIN
MIXED
```

Primary inclusion rule:

```text
ORCHESTRATION -> included when inside the pre-execution boundary
MIXED         -> included once when inside the pre-execution boundary
DOMAIN        -> excluded from QA-04 Primary
```

### ORCHESTRATION

Architecture-level inference used to connect the request to an execution owner.

Examples:

- intent refinement;
- referent reasoning;
- execution-path selection;
- Agent routing;
- model-based validation;
- clarification decision;
- task/result association;
- execution-owner selection;
- architecture-level retry/fallback decision.

### DOMAIN

Substantive reasoning for task execution after ownership is established.

Examples:

- NetworkAgent root-cause analysis;
- MailAgent content analysis;
- report-generation reasoning;
- Downstream Agent domain planning;
- Agent ReAct loop;
- tool-result interpretation/replanning.

### MIXED

One generation performs both architecture-level ownership/delegation decision and substantive domain reasoning.

Example:

```text
ARGO reasons about the request while also deciding
whether ARGO executes it or delegates to another Agent.
```

This is counted once, not split into artificial sub-calls.

## Pure Voice / S2S model calls

A model call that performs only voice generation or non-decision interaction output is excluded from the QA-04 Primary Metric.

Example:

```text
S2S generation used only to produce spoken output -> excluded
```

However, when the same inference also performs any of the following:

- intent interpretation;
- execution-path decision;
- routing;
- validation;
- delegation;
- execution-owner selection;

it is classified as `MIXED` and included once.

The metric intentionally removes a common voice-generation baseline and focuses on **model inference required by execution-decision topology**.

## Architecture Qualification with deterministic replay

The architecture score continues to use controlled semantic replay/test doubles where needed.

A critical accounting rule is:

> **A logical model call is still counted when the architecture initiates a model generation but the benchmark fulfills it through deterministic replay.**

The replay controls the output; it does not erase the architecture's requirement for an inference generation.

Therefore:

```text
architecture requests model generation -> logical call event recorded -> count according to class/boundary
architecture uses deterministic code only -> no model call event -> count 0
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

QA-04 therefore measures actual logical pre-execution decision calls and performs no hypothetical subtraction.

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

Requires execution-state/task continuation logic before ownership is confirmed.

Exact taxonomy and class proportions are **TBD**.

Because an arithmetic mean is sensitive to scenario mix, final benchmark qualification must freeze:

- workload taxonomy/version;
- class scenario counts;
- episode eligibility;
- aggregation rule;
- semantic replay mix;
- model/prompt/cache profiles;
- scoring version.

If pilot evidence shows that one request class dominates the overall mean, a class-level macro-average remains a candidate aggregation rule. Any such change must be selected and frozen before final A/B/C/D evaluation.

Usage-frequency-weighted results may be calculated as Secondary sensitivity analysis.

## Episode eligibility and correctness

QA-04 should not reward an architecture for failing before execution and therefore producing zero calls.

A complete Primary observation requires a valid measurement end boundary: execution ownership was confirmed and execution actually started.

Episodes that never reach execution start are retained as raw failure/non-completion evidence and reported beside the QA-04 population, but are not converted into an arbitrary model-call penalty.

Correctness is evaluated separately in QA-02. A low call count does not compensate for a wrong execution owner.

A useful Secondary slice is:

```text
Calls per exact-conform episode
```

## Stress-test interpretation

The agreed rules produce the intended results for the following cases:

```text
1. Short/Long tasks with identical routing topology
   -> same QA-04 Primary count

2. Long task with 30–40 downstream ReAct calls
   -> later DOMAIN calls excluded

3. Deterministic selector vs LLM selector
   -> 0 vs 1 call

4. intent + routing + validation as separate generations vs one fused generation
   -> 3 vs 1 calls

5. ARGO domain + delegation decision in one inference
   -> MIXED, counted once

6. Common pure voice generation
   -> excluded

7. Specialized Agent internal reasoning after ownership
   -> excluded

8. New logical retry generation before execution
   -> +1

9. Different physical model costs
   -> raw resource telemetry retained, no arbitrary Primary weight
```

These cases demonstrate that the metric observes architecture topology rather than downstream workload size.

## Quality Attribute Scenario

| 항목 | 정의 |
| --- | --- |
| **Source** | Samsung PC 사용자 |
| **Stimulus** | VIA가 처리해야 하는 representative user request |
| **Environment** | 동일 scenario, frozen semantic replay / scripted dependency, fixed model/prompt/cache profiles |
| **Artifact** | DP-00 Alternative의 VIA Integrated Product SW architecture |
| **Response** | 요청을 해석하고 적절한 execution owner를 결정하여 실행을 시작 |
| **Response Measure** | **평균 실행 전 모델 호출 수 (Average Pre-execution Model Calls per Episode)** |

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
- usage-frequency-weighted sensitivity result.

Backup Primary candidates, if future evidence requires a new scoring methodology:

- Critical-path Inference Time / Episode;
- Measured Normalized Inference Work / Episode;
- Uncached Tokens / Episode;
- On-device Accelerator Time / Episode.

A future Primary change requires a documented rationale and a new scoring version applied equally to all alternatives.

## Scoring

QA-04 uses six score bands: **0, 1, 2, 3, 4, 5**.

Threshold values are currently **TBD**. Lower is better.

Plain-text scoring contract:

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
 -> threshold calibration
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
execution_owner
execution_owner_confirmed_ts
execution_started_ts

pre_execution_orchestration_call_count
pre_execution_mixed_call_count
pre_execution_total_primary_call_count

total_orchestration_call_count
total_domain_call_count
total_mixed_call_count
total_model_call_count

exact_conformance
```

Per-call model events are the source of truth. Episode-level counts should be derived/projection fields where possible rather than the only evidence.

## Relationship to other QAs and DP-00

### QA-01

QA-01 measures **user-visible useful-outcome latency** for Fast tasks. QA-04 measures **how many architecture decision generations occur before execution starts** across representative request classes.

A topology can use fewer calls yet still have poor latency because one call is expensive, or use more calls but run them cheaply/parallel. The two QAs are related but not equivalent.

### QA-02

QA-02 evaluates interaction/orchestration correctness. QA-04 must not reward an incorrect zero/low-call path as architecturally desirable. The trade-off is read across separate scores and diagnostic slices.

### QA-03

QA-03 evaluates change containment. A fused low-call architecture may reduce QA-04 overhead while increasing coupling and lowering flexibility; decomposed stable seams may show the opposite trade-off.

### DP-00

DP-00 alternatives differ directly in pre-execution AI decision topology:

- Thin VIA can introduce explicit intent/routing stages before Agent execution;
- ARGO-centric execution can fuse first substantive reasoning and owner/delegation choice;
- Hybrid Fast Path can use deterministic or model-based eligibility before local/Agent execution;
- Adaptive Per-turn execution can add a selector but may avoid later hops.

QA-04 therefore measures one architecture consequence of execution-capability placement without assuming which topology should win.

## Unresolved / TBD

- `I1`~`I5` score thresholds;
- final W1~W4 taxonomy and scenario counts;
- final episode aggregation rule if macro-average is selected after pilot;
- exact eligibility handling for rare aborted/non-started episodes beyond retaining them outside complete Primary observations;
- concrete serialization format for model-call telemetry;
- final model/prompt/cache profile set for controlled qualification;
- final QA numbering/rebaseline after QA-01~QA-04 integration.
