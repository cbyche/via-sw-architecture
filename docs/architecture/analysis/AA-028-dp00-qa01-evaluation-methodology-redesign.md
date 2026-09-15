# AA-028 — DP-00 QA-01 Evaluation Methodology Redesign

> **HISTORICAL ANALYSIS — LEGACY QA SEMANTICS.** QA identifiers, metrics, populations, targets, and scores in this record belong to the pre-QA-Contract-v1 DP-00 work. Preserve them for provenance; use `docs/evaluation/qa-contracts/v1/` for every future evaluation.

## Disposition

**DESIGN-TIME REFERENCE METHODOLOGY IMPLEMENTED — DP-00 SELECTION REMAINS DEFERRED.**

This record adds stronger QA-01 design-time evidence. It does not select A/B/C/D, alter their ownership boundaries, modify the approved requirements baseline, or reinterpret QA-02/03/04.

## Baseline and versioning

The requested experimental baseline is Git commit `5f00cea12fadeaf5dc5508691c0d4bfbe080ceaa`. The working repository also contains later decision-catalog documentation; this change preserves that newer state. The new evidence identity is `DP00-QA01-REFERENCE-V1` and is additive:

- `benchmark/scenarios/pilot-v0/`, its behavior plans, oracles, fixtures, and P01–P12 remain unchanged;
- Profile Z and R1–R4 contracts and reports remain unchanged historical evidence;
- the v1 analyzer for captured runtime episodes remains unchanged;
- the reference simulator consumes a separate semantic corpus, trace contract, and latency contract;
- QA-02/03/04 populations, denominators, oracles, qualification rules, and Execution Route Commit semantics remain unchanged.

## Two-stage architecture-analysis framing

The same benchmark evidence answers two distinct architectural questions and is
therefore reported through two derived views.

### Stage 1 — Primary Execution Boundary: A0 vs B0

`A0` is the existing A result interpreted as Agent-neutral VIA orchestration.
VIA owns intent/goal interpretation, top-level Agent selection, initial route,
task lifecycle, and result binding; ARGO is a peer Agent. `B0` is the existing B
result interpreted as ARGO-centric primary execution. ARGO owns primary semantic
interpretation and initial execution/delegation authority and authoritative
execution state; VIA retains interaction and task projection/correlation.

This is a genuine ownership, authority, and state-boundary comparison:

```text
A0: Context -> Intent Refiner -> Agent Router -> ARGO or Specialist
B0: Thin VIA -> ARGO -> direct execution or Specialist delegation
```

F1 remains valid Stage-1 evidence. It shows how each base execution boundary
handles a local-capable request without introducing a VIA Fast bypass.

### Why D0 is not a Stage-1 candidate

Removing `VIA_FAST` from D removes the execution-topology class that materially
distinguishes the selector from A's Agent Router. Under the current executable
specification, A0 and the remaining D0 structure both keep route authority and
user-facing task state in VIA, select ARGO or a Specialist as top-level
executor, reuse an existing route for clear follow-up, and place domain state in
the selected executor. `ARGO_PRIMARY` versus `ARGO`, or `SPECIALIST_DIRECT`
versus selecting a Specialist, does not create an independent architecture when
authority, state, and interface semantics are otherwise unchanged.

D0 is therefore treated as **structurally equivalent to A0 for this trade-space
purpose**. This is not a mathematical identity, and no new D0 responsibility is
invented merely to retain a third Stage-1 candidate.

### Stage 2 — Bounded Fast-capability Accommodation: C vs B1 vs D1

Stage 2 asks how bounded latency-sensitive capability is accommodated while
each architecture preserves its defining ownership boundary:

- `C` is A0 plus deterministic Fast eligibility and VIA-local execution;
- `B1` is B0 with bounded capability execution remaining inside ARGO's existing
  execution/tool authority; B1 is not a VIA Fast Path or a new primary
  architecture;
- `D1` uses one semantic Execution Path Selector across VIA Fast, ARGO Primary,
  and Specialist Direct.

F1 is the primary Stage-2 population. C and D1 are not distinguished merely by
sequential versus simultaneous routing. C decomposes deterministic local
eligibility from semantic non-local Agent selection. D1 places local, ARGO, and
Specialist selection in one semantic responsibility. The observed `C F1 p95 <
D1 F1 p95` is causally consistent with that decomposition because D1 incurs the
additional semantic route-decision generation.

F2, F3, and the balanced pooled aggregate remain secondary diagnostics in Stage
2. The C/D pooled tie demonstrates tail masking, not equivalent Fast-capability
latency: their common F2/F3 paths occupy the tail while their F1 paths remain
materially different.

## Gap analysis

The earlier campaign completed its frozen contract, but it answers only a narrow question: how A/B/C/D behaved for P01–P03 under fixed synthetic dependency-cost points.

1. **Three semantic tasks are insufficient.** P01, P02, and P03 are the only QA-01 scenarios. Sixteen executions of each primarily reproduce runtime/timer behavior; they do not create new semantic workload coverage.
2. **C's defining Fast Path was absent.** P01–P03 produced zero eligible C VIA-Fast observations, while D exercised local execution in part of the cohort. This asymmetric defining-path coverage prevents the campaign from directly testing C's main latency hypothesis.
3. **The fixed R1–R4 values are not user-latency distributions.** Their 20/50/100 ms Model/Agent/Tool constants are useful controlled sensitivity points, but they are explicit assumptions and collapse semantically different model operations into one charge.
4. **A pooled p95 is not explanatory enough.** Without F1/F2/F3 and route breakdowns, a tail can be driven by category/effect composition rather than the architecture steps under study.

What remains valid:

- the historical FTOL boundary and captured values for the frozen experiment;
- Profile Z as software/framework-path evidence;
- R1–R4 as fixed-cost topology sensitivity evidence;
- the canonical observation, provenance, and route-commit contracts;
- QA-02/03/04 results and their stated limitations.

Nothing in this redesign relabels historical samples as production data or as samples from the new distribution.

## Revised QA-01 contract

### Measurement boundary

For every successful design-reference realization:

```text
FTOL = earliest Useful Outcome - ground-truth Acoustic EOS
```

The simulator composes that interval from V, semantic model generations, execution handoffs, and the useful effect. Route-commit time, framework overhead, call count, and handoff count remain diagnostics rather than the Primary Metric.

### Workload

`DP00-QA01-REFERENCE-V1` contains 36 unique semantic scenarios:

- F1: 12 local/direct-capable tasks — four obvious local, four context-bearing local, four valid boundary-near local;
- F2: 12 short general-executor tasks — four simple, four short reasoning, four context-dependent;
- F3: 12 short specialist tasks distributed across the repository-established NetworkAgent, MailAgent, and FileAgent domains, with four distinct capabilities per domain.

The primary **Design Reference Mix v1** weights F1/F2/F3 equally. It is not a production workload mix. Each scenario receives 1,000 Monte Carlo latency realizations per alternative and model profile. Thus 432,000 primary realizations represent 36 tasks, not 432,000 different tasks.

### Architecture-neutral scenario oracle

The common oracle judges the useful outcome, binding, target/subject, and absence of prohibited effects. It does not force all candidates through one topology. Candidate-specific conformance is evaluated separately against the frozen architecture trace mapping. In particular, F3 may be Specialist Direct in A/C/D and ARGO-mediated in B without changing the common outcome.

### DP-00 defining-path completion gate

This is a DP-00 evaluation-policy gate, not a universal QA-01 definition. The contract requires at least eight C VIA-Fast and eight D VIA-Fast scenarios, and at least four scenarios for every other major path:

- A: General Agent and Specialist Direct;
- B: ARGO Primary and ARGO → Specialist;
- C: VIA Fast, General Agent, and Specialist Direct;
- D: VIA Fast, ARGO Primary, and Specialist Direct.

The v1 corpus supplies 12 scenarios for every listed route class. Zero defining-path coverage is a hard failure.

## Primitive model

The architecture-neutral vocabulary is:

- `M:SEMANTIC_INTERPRETATION` — 24 generated tokens;
- `M:EXECUTION_ROUTE_DECISION` — 16 generated tokens;
- `M:SHORT_EXECUTOR_REASONING` — 32 generated tokens;
- `M:COMBINED_EXECUTION_DECISION` — 48 generated tokens;
- `H` — execution handoff;
- `X1/X2/X3` — useful effect/query class;
- `V` — common voice end-of-speech detection reference.

The token budgets are P3 engineering assumptions. Deterministic Fast eligibility, capability/policy lookup, state lookup, and result binding receive no artificial model charge.

For one semantic operation:

```text
ModelLatency = TTFT + (output_tokens - 1) × TPOT
```

TTFT and TPOT are independently sampled log-normal variables. Parameters are fit from the stated p50/p95 values. A SHA-256-derived keyed seed makes every sample deterministic and gives the same scenario/effect realization to every candidate. The seed is `20260914`.

## Evidence and assumptions

### P1 — published/public evidence

The [official Qwen3 speed benchmark](https://github.com/QwenLM/Qwen3/blob/main/docs/source/getting_started/speed_benchmark.md) provides SGLang 0.4.6.post1, BF16, batch-1 results on an NVIDIA H20 96 GB GPU. At input length 1 it reports 227.80 tok/s for Qwen3-1.7B, 81.73 tok/s for Qwen3-8B, and 137.18 tok/s for Qwen3-30B-A3B. Qwen's published metric is `(prompt tokens + generated tokens) / elapsed time` for 2,048 generated tokens. Its reciprocal is therefore used only as a first-order TPOT approximation, not described as a measured TPOT distribution.

The [OpenAI Realtime API reference](https://platform.openai.com/docs/api-reference/realtime-server-events/conversation/item/input_audio_transcription/completed) documents `server_vad.silence_duration_ms = 500` as the default. `V = 500 ms` is a design-time Acoustic-EOS/end-of-speech detection reference derived from that configuration, not “OpenAI measured latency.”

### P2 — repository-derived structural facts

The DP-00 executable specification determines which responsibilities are semantic generations, which steps are deterministic, and where handoffs exist. `benchmark/qa01-reference-v1/traces.json` maps those responsibilities onto neutral primitives. It does not assign a final latency directly to an alternative.

### P3 — engineering assumptions

- TTFT p50/p95: 35/70 ms (1.7B), 70/140 ms (8B), 50/100 ms (30B-A3B);
- TPOT p95 = 1.25 × the published-throughput reciprocal used as p50;
- H = 5 ms, sensitivity 1/5/20 ms — deployment mechanism not frozen;
- X1/X2/X3 p50 = 20/50/100 ms and p95 = 30/75/150 ms;
- X sensitivity = 0.5x/1x/2x;
- operation output-token budgets = 24/16/32/48.

No matching official TTFT percentile set was located for all three selected models under the same serving conditions. Using explicit assumptions is more defensible than combining incomparable public observations or fabricating measured traces.

## Fairness and trace interpretation

Within each Qwen profile, every semantic generation in every candidate uses the same distribution. The first-order comparison does not assign different models to VIA, ARGO, or Specialists. The same scenario effect realization is shared across A/B/C/D. Only repository-derived topology composition differs.

C and D are not described as fundamentally different after route commitment. Their F2 and F3 execution graphs can converge. Their F1 difference is control-plane decomposition: C applies deterministic eligibility after semantic interpretation, while D performs a semantic execution-path selection.

## Reporting and completion

For each model profile the generated report includes a Stage-1 A0/B0 view, a
Stage-2 F1-primary C/B1/D1 view, F2/F3 secondary diagnostics, and a secondary
cross-stage aggregate. Machine-readable aliases point back to the existing
A/B/C/D results; they do not copy or reweight latency realizations. The report
also preserves semantic scenario count, realization count, H/X sensitivity,
crossover observations, provenance, and limitations.

Completion requires:

- 12 scenarios in each category and passing defining-path gates;
- reproducibility from committed inputs and fixed seed;
- common distributions and effect samples across candidates;
- explicit P1/P2/P3 provenance;
- no production-latency, workload, SLO, or universal-winner claim;
- no mutation or reinterpretation of old evidence;
- no QA-02/03/04 semantic change.

## Final disposition

The redesigned QA-01 is a stronger design-time architecture experiment. Its
primary analysis is now `A0 vs B0` for the execution boundary and `C vs B1 vs
D1` for bounded capability accommodation. The old pooled A/B/C/D presentation
is retained only as a cross-stage diagnostic. QA-01 remains one evidence
dimension in the conditional DP-00 trade space and selects no winner.
