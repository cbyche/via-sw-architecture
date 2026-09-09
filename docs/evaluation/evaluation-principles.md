# Architecture Evaluation Principles

## Status

Architecture Context Checkpoint 001 — evaluation rules for vNext DP analysis.

These principles extend the current `evaluation-strategy.md` without replacing it. The existing strategy remains valid; a coherent rebaseline will be prepared after QA-01 through QA-04 are formally defined.

## Core scoring principles

1. **Exactly one Primary Metric per QA is used for the 0–5 architecture score.** A QA may have many observations, but one metric must determine the score so trade-offs remain interpretable.
2. **Secondary Metrics are diagnostic evidence and future-primary candidates.** They explain why a Primary Metric moved and preserve options for later methodology improvement.
3. **Every Primary and Secondary Metric must be recomputable from raw observations.** Reports are outputs, not the source of truth.
4. **Raw run data is immutable.** Corrections create new benchmark/scoring versions rather than rewriting historical evidence.
5. **Raw, derived and human report artifacts are separate.** Use `results/raw/`, `results/derived/`, and `results/reports/` respectively.
6. **The independent variable in an architecture benchmark is the SW architecture alternative.** A/B/C/D must differ in architecture topology, ownership or mechanisms—not in model quality or unrelated implementation tuning.
7. **Confounding dependencies are controlled.** Stochastic LLM behavior, Agent variability, live-network variability and uncontrolled machine/background state must not dominate an architecture comparison.
8. **Architecture qualification uses deterministic semantic replay and test doubles.** Model/Agent/tool behavior is injected through controlled traces/stubs so each architecture sees equivalent semantic conditions.
9. **Replay corpora must remain realistic.** Frozen semantic traces are derived from behavior classes observed in repeated runs of real models/Agents rather than invented only from idealized happy paths.
10. **Actual-model validation is separated from the Architecture score.** Real GPT/Qwen/local-model runs provide fidelity/external-validity evidence; they do not silently reintroduce stochasticity into the architecture-only comparison.
11. **Scoring follows a fixed order:** Pilot → threshold calibration → scoring-version freeze → final evaluation.
12. **Score thresholds are not tuned after seeing the final A/B/C/D outcome.** Post-hoc threshold selection would make the score a presentation device rather than an evaluation rule.
13. **Changing a Primary Metric requires an explicit reason and a new scoring version.** All alternatives must then be recalculated from the same raw corpus under that version.

## Raw-data principle

> **측정하지 않은 값은 나중에 복구할 수 없지만, raw data로 보존한 값은 나중에 다른 metric으로 재해석할 수 있다.**

This principle drives benchmark instrumentation. Store architecture-relevant event timestamps, ownership decisions, outcomes and invocation counts even when they are not part of today's Primary Metric.

The cost of retaining a timestamp or categorical event is small; the cost of discovering after an experiment that a needed boundary was never measured can invalidate the run.

## Metric discipline

A QA should separate:

- the **quality question** being evaluated;
- the **Primary Metric** that determines the architecture score;
- **Secondary Metrics** used for diagnosis;
- the **scenario population** over which the metric is valid;
- benchmark controls and exclusions;
- the scoring-version thresholds.

Do not combine independent concerns by arbitrary weighted formulas merely to obtain one number. For example, responsiveness and correctness should remain distinct QAs when a failure can otherwise be hidden as a latency penalty.

## Architecture isolation

During architecture qualification:

```text
Independent variable
    = SW Architecture Alternative

Dependent variables
    = QA Primary Metrics

Controlled / confounding variables
    = model semantic behavior
    = Agent behavior
    = tool/service latency
    = network behavior where possible
    = machine / power / background state
    = scenario corpus
```

The comparison should answer: **if the semantic/dependency behavior is held equivalent, what effect does the architecture topology itself create?**

## Deterministic replay does not mean unrealistic replay

A deterministic corpus should include realistic non-happy-path semantic classes observed from real dependencies, for example:

- correct semantic result;
- ambiguous or low-confidence result;
- partially wrong candidate;
- wrong execution-path/Agent candidate;
- malformed schema;
- timeout/no response.

The corpus is frozen before final A/B/C/D scoring and replayed equally across alternatives.

## External-validity track

Actual models and Agents should also be exercised, but their results answer a different question:

> Does the frozen qualification corpus still represent behavior seen with current real dependencies?

This track can detect replay drift, missing behavior classes and provider-specific surprises. It should not be conflated with the controlled architecture score.

## Versioning

Each benchmark result should identify at least:

- benchmark version;
- scenario corpus version;
- semantic trace/oracle version;
- architecture alternative/version;
- source Git commit;
- scoring version;
- machine/environment profile.

A scoring change must not mutate prior raw runs. Recompute derived metrics into a new version.

## Relationship to current repository documents

- `docs/evaluation/evaluation-strategy.md` remains unchanged in this checkpoint.
- `docs/evaluation/architecture-experiment-methodology.md` explains how these principles are applied.
- QA-specific documents define Primary/Secondary Metrics and scoring status.
- `benchmark/schemas/run-event-schema.md` defines the raw event contract.

The QA and DP index will be coherently rebaselined after QA-01~QA-04 are defined.