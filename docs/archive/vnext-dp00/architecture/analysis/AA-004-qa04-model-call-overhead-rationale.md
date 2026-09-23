# AA-004 — QA-04 Model Call Overhead Measurement Rationale

## Status

Architecture Analysis Record — Context Checkpoint 004, amended by Top-QA Cross-review

This record preserves the reasoning behind `QA-04 — 모델 호출 오버헤드 (Model Call Overhead)` and the rules needed to measure model-call overhead without conflating VIA/DP-00 topology with downstream task complexity, model price, prompt tuning or hardware efficiency.

It is not an ADR and does not modify `docs/requirements/requirements-v1.1.md`.

## Why Model Call Overhead is a top architectural driver

DP-00 deliberately changes where request interpretation, execution-path selection, routing and substantive execution authority live across VIA, ARGO and specialized Downstream Agents.

Those topologies can require materially different numbers of AI decision stages before the final execution route is committed:

- a deterministic local eligibility rule may require no model call;
- a model-based intent stage followed by a model-based Agent router may require multiple calls;
- intent, routing and validation may be fused into one generation;
- ARGO may perform domain reasoning and delegation selection in the same generation;
- a retry or model-based validator may add another generation;
- an intermediate execution runtime may begin processing and only later use a model to choose a specialist.

These calls matter because they consume inference capacity and usually add latency, memory/accelerator occupancy, token/context processing and operational complexity.

The architecture question is not simply “which system uses the fewest model calls over the whole task?” A long-running Agent can legitimately perform many domain reasoning iterations after the architecture has already committed the final route.

The QA therefore needs a measurement boundary that captures **AI decision overhead required to commit execution topology**, regardless of which component contains that decision.

## Initial candidate — Fast-task model calls

An early candidate was:

```text
average model-call overhead per Fast-task
```

This followed QA-01, where Fast-task restriction is necessary to keep long external work from dominating end-to-end latency.

That restriction is not needed once QA-04 excludes downstream DOMAIN reasoning after route commit.

Example:

```text
Short Task:
User -> Intent Model -> Router Model -> ARGO final route
QA-04 count = 2

Long Task:
User -> Intent Model -> Router Model -> ARGO final route
QA-04 count = 2

ARGO then performs 30 ReAct generations
QA-04 count remains 2
```

The duration and internal complexity of the downstream task no longer dominate the Primary Metric. Therefore short/long duration is not a QA-04 population exclusion criterion.

## Why total lifetime model-call count is not the Primary Metric

Counting every model generation until task completion would mix two different causes:

```text
route-selection/orchestration complexity
+
downstream domain reasoning complexity
```

For long-running tasks, the second term can dominate through:

- Agent planning;
- ReAct iteration;
- tool-result interpretation;
- replanning;
- downstream retry;
- workflow-specific reasoning.

A NetworkAgent diagnosing a difficult fault may need many more calls than a MailAgent doing a simple action even under identical VIA topology. That is primarily task/Agent complexity, not the DP-00 architecture boundary.

Therefore QA-04 excludes pure DOMAIN reasoning after the final execution route is committed.

## Why the original execution-start boundary was not sufficient

The first QA-04 checkpoint ended measurement when:

```text
execution owner confirmed
AND execution started/accepted
```

That seemed reasonable because it separated orchestration from later task execution.

Cross-review exposed a topology-location bias: an intermediate runtime can become an execution owner before the final specialist route has been chosen.

### Hidden-routing / boundary gaming case

Consider two architectures serving the same specialist request.

Alternative A — explicit Thin-VIA routing:

```text
Intent model
  -> Router model
  -> Specialist Agent accept

QA-04 under old rule = 2 calls
```

Alternative B — ARGO-centric hidden routing:

```text
ARGO starts
  -> ARGO model inference decides Specialist delegation
  -> Specialist Agent accept
```

If measurement stopped at the moment ARGO first started, the later delegation inference could be outside the Primary boundary.

In an extreme accounting outcome:

```text
A = 2
B = 0
```

That would not mean B requires zero AI decision overhead. It would mean the same routing responsibility was moved behind the measurement boundary.

This creates a metric-gaming opportunity: architecture could improve QA-04 by relocating owner/delegation decisions after an early “execution start” event rather than by eliminating them.

## Why Execution Route Commit is the corrected boundary

The corrected end boundary is **Execution Route Commit**.

Definition:

> **실제 domain 작업을 수행할 최종 실행 경로가 결정되어, 이후에는 해당 user request에 대해 추가적인 execution-owner 선택이나 delegation 판단 없이 domain execution을 계속할 수 있게 된 최초 시점.**

This boundary follows the architectural responsibility being measured rather than a component lifecycle event.

The normalization principle is:

```text
same architectural responsibility
-> same QA-04 accounting
regardless of whether the decision lives in VIA, ARGO, or another runtime
```

This amendment is **not intended to penalize ARGO-centric Alternative B**. It prevents any alternative from receiving free credit merely because its routing/delegation logic is physically located after an intermediate owner/start boundary.

### Thin VIA

```text
Intent -> Router -> NetworkAgent selected -> NetworkAgent accept
                                        ^ Execution Route Commit
```

### ARGO-centric, direct execution

```text
ARGO inference -> "I will execute directly"
                  ^ Execution Route Commit
```

If the same inference also performs domain reasoning, it is `MIXED` and counted once.

### ARGO-centric, specialist delegation

```text
ARGO begins processing
  -> ARGO inference decides NetworkAgent delegation
  -> NetworkAgent accepts
                     ^ Execution Route Commit
```

The delegation decision remains inside QA-04.

### VIA Fast Path

```text
deterministic eligibility
  -> local capability selected
  -> local execution begins
                    ^ Execution Route Commit
```

No model generation means zero QA-04 calls.

## Measurement start boundary

The logical start remains `request_processing_start_ts`: the benchmark-designated moment when the architecture begins processing the user goal for execution-decision purposes.

For voice, this is not necessarily identical to Acoustic End-of-Speech. An alternative may perform partial semantic work while speech is still arriving. Such architecture-requested model generations must not escape QA-04 merely because they started before EOS.

For text, the start is normally when the submitted user request becomes available to the architecture.

Model-call inclusion is based on timestamp plus causal episode correlation. `acoustic_eos_ts` is preserved so QA-01 and QA-04 can be correlated without forcing the same start boundary.

## Logical Model Call definition

A **Logical Model Call** is one new logical inference generation that begins producing a new model output.

Plain rules:

```text
Realtime connection/session creation          = 0
One streaming generation                      = 1
Number of streamed tokens/audio chunks        = irrelevant
New generation after tool/result feedback     = +1
Actual new retry generation                    = +1
```

The metric does not count HTTP requests, websocket frames, token chunks or audio packets.

A retry is counted only when it initiates a new inference generation. Transport retries that do not start a new generation are not additional model calls.

## Architecture Qualification and deterministic replay

Architecture Qualification uses deterministic semantic replay to control model stochasticity. This creates an important accounting rule:

> A model call is counted when the architecture **requires and initiates a logical model generation**, even if the benchmark fulfills that call through a deterministic replay/model double rather than a live provider.

Otherwise a replay-based qualification would incorrectly report zero model calls for every topology.

The model-call event records the logical generation requested by the architecture; the controlled replay supplies the frozen output. A deterministic code path that makes the same decision without requesting model inference correctly counts as zero.

This preserves the architecture as the independent variable while controlling semantic output.

## Why an “intrinsic/original task call count” baseline was rejected

A candidate formulation was:

```text
architecture-added calls = total calls - intrinsic task calls
```

The baseline is not objectively defined.

The same task may be implemented using:

- one large-model fused generation;
- several smaller-model generations;
- a deterministic classifier plus one model;
- one multimodal model;
- decomposed specialist models.

There is no architecture-neutral answer to “how many calls did the task intrinsically require?” The supposed baseline is itself a design decision.

Therefore QA-04 counts observed logical route-commit generations directly and does not subtract a hypothetical intrinsic count.

## Model-call classification

Every logical model call is classified as exactly one of:

```text
ORCHESTRATION
DOMAIN
MIXED
```

### ORCHESTRATION

The generation performs architecture-level interpretation/coordination needed to establish or commit the final execution route.

Examples:

- intent refinement;
- referent reasoning;
- execution-path selection;
- Agent routing;
- model-based validation;
- clarification decision;
- task/result association when relevant to route choice;
- execution-owner selection;
- delegation selection;
- architecture-level retry/recovery decision.

ORCHESTRATION calls are included when they occur before route commit or cause the commit.

### DOMAIN

The generation performs substantive task/domain reasoning after route commitment, such as:

- NetworkAgent root-cause analysis;
- MailAgent content reasoning;
- report-generation reasoning;
- downstream Agent planning/ReAct loops;
- tool-result interpretation for domain execution.

DOMAIN calls are excluded from QA-04 Primary even though their telemetry is retained.

### MIXED

A single generation performs both architecture-level route/owner/delegation decision and substantive domain reasoning.

Example:

```text
ARGO interprets the goal, reasons about the task,
and in the same generation decides whether ARGO executes it
or delegates it to a specialized Agent.
```

Splitting this generation into two artificial calls would misrepresent the topology. It is therefore recorded once as `MIXED` and included if it occurs before route commit or its output causes the commit.

## ARGO-centric mixed-call problem

Alternative B is intentionally ARGO-centric. Its first substantive ARGO inference can combine domain understanding with the decision “I will execute this” or “delegate to Agent X.”

If MIXED were excluded as DOMAIN, B would receive zero architecture decision cost for a generation that commits execution routing. If it were counted twice, B would be penalized for fusion.

The chosen rule is:

```text
one logical generation = one logical call
MIXED route-decision generation = included once
```

Execution Route Commit additionally ensures that a later ARGO delegation inference cannot disappear merely because ARGO started earlier.

## Pure Voice/S2S generation

A model generation used only for voice rendering or interaction output is excluded from QA-04 Primary.

Example:

```text
pure TTS/S2S response generation only -> excluded
```

However, if that same generation also performs intent interpretation, routing, validation, delegation or execution-owner selection, it is `MIXED` and included when it is before or causes route commit.

The purpose is to measure **execution-route decision model overhead**, not a common voice-output baseline.

## Why monetary cost is not the Primary Metric

Dollar cost is concrete but combines architecture with many unstable or non-architectural variables:

- future on-device inference may have no provider price;
- provider pricing changes;
- input/output token pricing differs;
- prompt sizes differ;
- prompt optimization changes token usage;
- context pruning/compression changes token usage;
- prompt caching policy and cache hit rate change billable input;
- providers use different cached-token discounts;
- tokenizers differ;
- model size, quantization and runtime differ.

Conceptually:

```text
Monetary Cost
= Architecture
+ Model Choice
+ Prompt Engineering
+ Cache Optimization
+ Provider Pricing
```

That makes provider cost unsuitable as the architecture-only Primary Metric.

Cost is still useful Secondary evidence and should be derivable from raw model-call telemetry where available.

## Why tokens are not the Primary Metric

Token counts are less volatile than dollar price but still depend heavily on prompt/context policy, tokenizer, caching and model interface. Two topologies can make the same number of architecture decisions with very different prompt packaging.

Input/output/cached/uncached tokens and context bytes are therefore telemetry, not the QA score.

## Why arbitrary model-category weights were rejected

One small classifier generation and one large multimodal generation do not have equal physical compute cost. Logical call count intentionally does **not** claim they are physically equivalent.

However, weights such as:

```text
small model = 0.2
large model = 3.0
```

would require arbitrary assumptions about:

- model size;
- hardware;
- prompt length;
- cache state;
- quantization;
- provider/runtime implementation.

The weighting model could dominate the ranking.

Therefore the Primary Metric assigns one count to each qualifying ORCHESTRATION/MIXED logical generation.

If a future weighted metric is needed, it should use measured, versioned model-profile resource data rather than hand-authored category weights.

## Workload population

QA-04 should be driven by request classes that exercise execution topology, not task duration.

Candidate classes:

- **W1 Local-capable request**;
- **W2 General Agent request**;
- **W3 Specialized Agent request**;
- **W4 Existing-task follow-up request**.

The exact taxonomy and class proportions are not frozen yet.

Because an arithmetic mean is population-sensitive, final qualification must freeze:

- workload taxonomy/version;
- scenarios per class;
- episode eligibility rules;
- aggregation rule;
- semantic replay mix;
- model/prompt/cache profiles;
- scoring version.

If one class would dominate the overall mean, class macro-average remains a Pilot-stage aggregation candidate. Any change must be frozen before final A/B/C/D results are produced.

Usage-frequency-weighted results may be derived separately as Secondary sensitivity analysis.

## Stress tests that shaped QA-04

### 1. Short and long task with identical routing topology

Both should produce the same Primary count when route-decision topology is identical. Task duration must not change QA-04 by itself.

### 2. Long task with 30–40 downstream ReAct generations

Those DOMAIN calls after route commit are excluded from the Primary count.

### 3. Deterministic selector vs LLM selector

```text
Deterministic path selector -> 0 model calls
LLM path selector           -> 1 model call
```

The metric exposes this structural difference.

### 4. Decomposed vs fused orchestration

```text
intent call -> routing call -> validation call = 3
fused intent+routing+validation generation     = 1
```

The metric captures orchestration-stage fusion/decomposition.

### 5. ARGO domain + delegation decision in one inference

Classify as `MIXED`; count once. This avoids both free credit and double counting.

### 6. Common pure voice generation

If it performs only interaction/voice output, exclude it from Primary for all alternatives.

### 7. Specialized Agent internal reasoning

Once the final route is committed to that Agent, its subsequent domain reasoning complexity is excluded.

### 8. Retry creates a new generation

A real architecture-level retry that initiates a new logical model generation contributes `+1` when it remains before or causes route commit.

### 9. Different physical model costs

Preserve latency/token/cache/CPU/GPU/NPU/memory/energy/provider-cost telemetry, but do not insert arbitrary weights into Primary scoring.

### 10. Hidden-routing / boundary gaming

```text
A: Intent Model -> Router Model -> Specialist accept
B: ARGO starts -> ARGO Model decides Specialist -> Specialist accept
```

Stopping at intermediate execution start can hide B's delegation call. Stopping at Execution Route Commit includes both topologies' route-selection responsibility until the final route is committed.

This stress test caused the QA-04 boundary amendment.

## Correctness interaction

Fewer calls are not automatically better if the architecture makes a wrong execution decision.

QA-04 therefore remains separate from QA-02.

An architecture that commits an incorrect route after zero calls may look efficient in isolation but fail AECR. Final architecture selection must read QA-02 and QA-04 together rather than combining them through an undocumented penalty.

Episodes that never reach a valid route-commit boundary cannot produce a complete QA-04 Primary observation. They must be retained as raw failures and reported with the eligible/completed population rather than assigned a fabricated high call count.

A useful Secondary metric is `Calls per exact-conform episode`.

## Why broad raw telemetry is mandatory

Logical call count is deliberately simple and explainable, but it does not describe physical inference cost.

Raw telemetry therefore preserves enough information to later derive or test alternatives such as:

- Critical-path Inference Time / Episode;
- Measured Normalized Inference Work / Episode;
- Uncached Tokens / Episode;
- On-device Accelerator Time / Episode;
- provider monetary cost under a specified price table.

Changing the Primary Metric later requires an explicit methodology/scoring version. It must not be driven by a preferred final ranking.

## Checkpoint decisions

Agreed:

- QA-04 measures model-call overhead required to commit the final execution route, not full task reasoning.
- Primary Metric is **Average Model Calls to Commit Execution Route**.
- Fast-task restriction is removed.
- Measurement begins at `request_processing_start_ts`.
- Authoritative end boundary is `execution_route_commit_ts`.
- `execution_owner_confirmed_ts` and `execution_started_ts` remain diagnostic raw timestamps, not the Primary boundary.
- A logical model call is one new inference generation, independent of streaming chunks.
- ORCHESTRATION and MIXED calls before or causing route commit are included.
- DOMAIN calls are excluded from Primary.
- Pure voice/output-only generations are excluded; decision-bearing S2S generations are MIXED and included.
- Deterministic replay still records/counts the logical model generation requested by the architecture.
- No hypothetical intrinsic/original task-call subtraction is used.
- Total lifetime model-call count is Secondary only.
- Dollar/token cost and arbitrary model-size weights are not Primary.
- Broad token/cache/profile/resource telemetry is retained for diagnostics and future metric recomputation.
- Execution Route Commit is a topology-neutral normalization intended to prevent hidden-routing/boundary gaming.

Not yet decided:

- final W1~W4 taxonomy and scenario proportions;
- whether final aggregation remains episode mean or uses class macro-average after Pilot;
- score thresholds `I1`~`I5`;
- concrete model-call telemetry serialization format;
- exact eligibility rules for rare aborted/non-committed episodes;
- final QA renumbering/rebaseline;
- DP-00 winning alternative.
