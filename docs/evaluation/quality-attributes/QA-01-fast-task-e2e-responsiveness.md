# QA-01 — Fast-task End-to-End Responsiveness

## Status

**Proposed vNext — QA definition agreed, scoring thresholds TBD**

This document is part of the QA rebaseline work. It does not replace the Approved Baseline QA numbering yet. `docs/requirements/requirements-v1.1.md` remains unchanged until a coherent vNext rebaseline is reviewed.

## Name

**QA-01 Fast-task End-to-End Responsiveness**

## ISO quality characteristic

**Performance Efficiency / Time Behaviour**

## Architectural concern

For tasks that can reasonably finish quickly under controlled dependency conditions, how much user-perceived latency is introduced by the SW architecture itself?

The concern is specifically important for DP-00 because alternative execution topologies may add or remove semantic-routing, serialization, dispatch, Agent and result-return hops.

Long-running task overall completion time is **not** the Primary Metric scope of this QA. Long/background work is dominated by task execution duration and needs separate lifecycle/progress/reliability evaluation.

## Primary Metric

### Fast-task Outcome Latency p95 — FTOL p95

For successful Fast-task episode `i`:

```text
FTOL_i = useful_outcome_timestamp_i
         - acoustic_end_of_speech_timestamp_i
```

Then:

```text
QA-01 Primary Metric = p95(FTOL_i)
                       over successful Fast-task episodes
```

- Unit: **milliseconds**
- Direction: **lower is better**
- Architecture score: one 0–5 score derived from FTOL p95 using a frozen scoring version

## Why the start point is Acoustic End-of-Speech

The start timestamp is **not** the runtime's `speech_stopped` callback/event.

It is the ground-truth **Acoustic End-of-Speech (EOS)** pre-annotated in the benchmark audio fixture.

Rationale:

- VAD/endpointer delay is user-perceived latency.
- If measurement starts only when software decides speech stopped, architecture/runtime endpointing latency disappears from the metric.
- All alternatives should receive the same audio fixture and EOS annotation.

The benchmark runner should use a monotonic clock and map the fixture's acoustic EOS offset into that run's monotonic timeline.

## Why the endpoint is Useful Outcome

The endpoint is the **first externally observable state that satisfies the scenario success predicate**.

It is not a generic acknowledgment such as:

> “네, 처리할게요.”

Acknowledgment can be fast while the useful action/result remains delayed. Counting acknowledgment would incentivize architecture that speaks early rather than finishes quickly.

### Examples

| Scenario | Useful outcome timestamp |
| --- | --- |
| Change system volume | First observation that actual system-volume state equals the requested value |
| Pause music | First observation that playback state is paused |
| Open file | Target file/window becomes usable/open according to scenario oracle |
| Short factual/general query | First meaningful result delivery begins |
| Short Agent task | Deterministic Agent result delivery begins |

The success predicate belongs to the scenario definition, not to the architecture alternative.

## Fast-task population

A task is classified as Fast-task **before execution**, from scenario semantics. It is not classified because an observed run happened to finish quickly.

A Fast-task scenario should satisfy:

- bounded interaction/execution;
- no intentional long-running/background wait;
- useful outcome is machine-observable;
- external dependency latency can be controlled equivalently across alternatives;
- the same success condition applies to every architecture alternative.

Agent delegation is **not** an inclusion/exclusion criterion. A short Agent operation may still be a Fast-task scenario and is essential for measuring delegation overhead.

## Required dataset categories

The QA-01 corpus must include at least three categories.

### F1 — Local / Direct-capable fast task

A bounded operation that at least one architecture could appropriately complete without a substantive Downstream Agent path.

Purpose: expose local-vs-delegated boundary overhead.

### F2 — Short general-Agent task

A bounded request that requires a general Downstream Agent but has controlled short execution.

Purpose: compare Agent dispatch/topology without long-running-work noise.

### F3 — Short specialized-Agent delegation task

A bounded request whose appropriate execution owner is a specialized Agent.

Purpose: expose costs of first-hop routing, ARGO-first rerouting or direct specialized-Agent dispatch.

Report category distributions as Secondary Metrics even though the overall FTOL p95 is the QA Primary Metric.

## Success and failure handling

FTOL is computed for **successful episodes only**.

A failed episode must not be transformed into an arbitrary large latency value. Doing so would mix two quality questions and make the score depend on an invented penalty constant.

The intended separation is:

```text
QA-01 = when the architecture succeeds, how responsive is the useful outcome?
QA-02 = how correctly/reliably does it reach the expected outcome?
```

The final report must always show the success population/count beside QA-01 so a very small successful subset cannot be misread as good overall system quality.

## Secondary Metrics

The following do not determine the QA-01 0–5 score, but raw data should make them recomputable:

- FTOL p50;
- FTOL p99;
- F1/F2/F3 FTOL distributions and p95;
- VIA software overhead;
- Acoustic EOS → execution-path selection latency;
- execution-path selection → dispatch latency;
- semantic/model wait time;
- Downstream Agent wait time;
- tool/action duration;
- first user-visible response latency;
- full task-completion latency;
- model invocation count;
- execution-owner/path distribution.

These metrics diagnose why FTOL moved and may become candidates for future QA definitions.

## Suggested event decomposition

For episode `i`, when available:

```text
Acoustic EOS
   |
   +--> semantic_request_start
   +--> semantic_response
   +--> execution_path_selected
   +--> agent_dispatch
   +--> tool_start
   +--> tool_complete
   +--> first_user_visible_result
   +--> useful_outcome       <-- FTOL endpoint
   +--> task_complete
```

Not every path emits every intermediate event. Missing/not-applicable events should remain explicitly absent rather than fabricated.

## Architecture Qualification controls

Final architecture scoring uses controlled qualification runs with:

- deterministic semantic replay;
- deterministic Downstream Agent stubs;
- deterministic/replayed tool latency;
- fixed Reference Development Machine;
- fixed power/background condition;
- identical scenario corpus;
- identical semantic/oracle trace per comparable episode;
- consistent warm/cold-state rules.

The architecture alternative must be the intended independent variable.

## Actual-model validation

Real GPT/Qwen/local-model/Agent runs are used separately for trace generation and fidelity/external-validity checks.

They are not mixed into the architecture-only FTOL score unless a future QA/scoring version explicitly changes this methodology.

## Scoring

QA-01 uses **six score bands: 0, 1, 2, 3, 4, 5**.

Threshold values are currently **TBD**.

Required process:

```text
Pilot
  -> inspect FTOL distributions and instrumentation
  -> threshold calibration
  -> scoring-v1 freeze
  -> final A/B/C/D evaluation
```

Thresholds must not be changed after inspecting final alternative results merely to favor a preferred topology.

A threshold or Primary-Metric change requires a new scoring version and equal recomputation for all alternatives.

## Raw-data requirements

The raw event contract is defined in:

`benchmark/schemas/run-event-schema.md`

At minimum, QA-01 requires reliable capture of:

- run/scenario/alternative identity;
- benchmark/source version;
- monotonic time origin;
- acoustic EOS;
- execution path and owner;
- relevant semantic/dispatch/tool/Agent timestamps;
- first user-visible result;
- useful outcome;
- task completion;
- success/failure;
- model invocation count.

## Interpretation cautions

- Faster acknowledgment is not necessarily faster useful outcome.
- A p95 improvement cannot compensate for incorrect results; correctness belongs to another QA.
- A Fast-task corpus dominated by F1 can hide Agent-routing overhead; category distributions must be reported.
- Live-network/provider measurements are useful Product E2E observations but are not substitutes for controlled architecture qualification.
- Model invocation count is diagnostic; fewer calls are not automatically better if correctness/lifecycle quality degrades.

## Related Decision Points

Primary current relevance:

- DP-00 — VIA Primary Execution Boundary
- capability placement
- intent refinement
- Agent routing
- voice runtime composition
- Model Gateway selection

Exact QA↔DP numbering will be updated coherently after QA-01~QA-04 formalization.

## Scoring thresholds

TBD — to be calibrated in pilot and frozen as `scoring-v1` before final comparative evaluation.