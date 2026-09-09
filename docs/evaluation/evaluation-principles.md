# Architecture Evaluation Principles

## Status

Architecture Context Checkpoint 001/002/003 — evaluation rules for vNext DP analysis.

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
14. **Correctness oracles must be topology-neutral.** When alternatives have different internal components, score canonical architecture outcomes rather than requiring the presence/call sequence of one implementation-specific component.
15. **Do not encode one valid execution topology as the only expected path.** Where more than one architecture outcome is semantically correct, scenarios use Required/Allowed/Forbidden constraints rather than a single `expected_path`.
16. **Architecture correctness is evaluated at user-goal episode scope when state spans turns.** Clarification, follow-up, task association and result binding must be tested across the complete logical episode rather than averaged as isolated turn/component successes.
17. **Scenario taxonomy and corpus composition are part of the metric definition.** A population metric such as AECR changes with scenario mix, so taxonomy/corpus/eligibility/manifest versions are frozen before final scoring.
18. **Architecture qualification prioritizes architecture-sensitive structural coverage.** Production usage-frequency weighting is useful as a Secondary sensitivity analysis, but must not silently replace a frozen coverage-balanced scoring corpus.
19. **Exact metrics require diagnostic preservation.** If the Primary Metric is all-or-nothing at episode level, every constraint-level result and failure reason must still be stored so partial improvements and construct-validity problems remain observable.
20. **Evolution Flexibility evaluation freezes the Expected Change Area and alternative-specific architecture-role mapping before implementation/result observation.** A boundary that can be redefined after the diff is visible cannot serve as a valid containment oracle.
21. **Development time and LOC are not architecture-only Primary Metrics when human/tool/coding-style variables can dominate them.** They may be preserved as diagnostics, but not used to determine the QA score without a separately justified methodology.

## Raw-data principle

> **측정하지 않은 값은 나중에 복구할 수 없지만, raw data로 보존한 값은 나중에 다른 metric으로 재해석할 수 있다.**

This principle drives benchmark instrumentation. Store architecture-relevant event timestamps, ownership decisions, canonical correctness outcomes, constraint results and invocation counts even when they are not part of today's Primary Metric.

The cost of retaining a timestamp or categorical event is small; the cost of discovering after an experiment that a needed boundary was never measured can invalidate the run.

For QA-02 specifically, storing only final `episode_exact_conform` or AECR is prohibited. Preserve the manifest version, actual canonical trace, each applicable constraint result, and failure ids/reasons.

For QA-03, storing only final CCR or `scenario_change_contained` is likewise prohibited. Preserve the frozen Expected Change Area, role mapping, actual changed roles/files, acceptance/regression results, and unexpected propagation evidence.

## Metric discipline

A QA should separate:

- the **quality question** being evaluated;
- the **Primary Metric** that determines the architecture score;
- **Secondary Metrics** used for diagnosis;
- the **scenario population** over which the metric is valid;
- benchmark controls and exclusions;
- the scoring-version thresholds.

Do not combine independent concerns by arbitrary weighted formulas merely to obtain one number. For example, responsiveness and correctness should remain distinct QAs when a failure can otherwise be hidden as a latency penalty.

Likewise, do not make a component-accuracy average the correctness Primary Metric when one critical error can invalidate the whole user-goal orchestration. Component/slice metrics remain valuable Secondary diagnostics.

For Flexibility, do not use topology size, component count, LOC or developer time as a substitute for the architecture question of whether a change crossed responsibility boundaries unexpectedly.

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

For evolution experiments, the controlled inputs are different but the principle is the same. Freeze the common evolution requirement, architecture-role Expected Change Area, alternative role mappings, acceptance tests and regression suite; then observe how each architecture must actually change.

## Deterministic replay does not mean unrealistic replay

A deterministic corpus should include realistic non-happy-path semantic classes observed from real dependencies, for example:

- correct semantic result;
- ambiguous or low-confidence result;
- partially wrong candidate;
- wrong execution-path/Agent candidate;
- malformed schema;
- timeout/no response.

The corpus is frozen before final A/B/C/D scoring and replayed equally across alternatives.

For correctness evaluation, replaying the same wrong model output is especially useful: it tests whether validation, clarification, eligibility policy, safe fallback and state ownership differ across architectures without making model quality the independent variable.

## Canonical normalization principle

A/B/C/D may not contain the same internal components. Architecture qualification therefore normalizes observable decisions into a common contract such as:

- referent bindings;
- task relation/id;
- execution owner/path;
- delegated Agent;
- clarification action;
- result binding;
- compound-decomposition relations;
- observable effect.

Scenario constraints are evaluated against this **Canonical Architecture Decision Trace**, not against fields such as `router.selected_agent` or `intent_refiner.output` that may not exist in every topology.

## Constraint-oracle principle

When one scenario permits several correct implementations, the oracle should express:

```text
Required   — outcomes/relations that must hold
Allowed    — acceptable alternatives/sets
Forbidden  — outcomes/actions that must not occur
```

This prevents the benchmark from prejudging DP-00 by declaring one architecture's preferred path as the only correct path.

Example: a simple local operation may allow both `VIA_FAST` and `ARGO`; correctness can be equal while QA-01 measures the latency trade-off.

## Corpus composition and sensitivity

Architecture Qualification uses a coverage-balanced corpus that deliberately represents architecture-sensitive failure surfaces. This avoids easy, high-frequency requests overwhelming rare but structurally important cases such as multi-point referents, task follow-up, clarification, concurrent result binding or malformed semantic output.

The following must be versioned/frozen for final qualification:

- taxonomy;
- corpus composition;
- eligibility rules;
- scenario/constraint manifests;
- semantic replay set;
- scoring version.

Usage-frequency-weighted results may be derived separately to answer a Product E2E sensitivity question.

## Evolution-benchmark boundary discipline

Flexibility experiments compare semantically equivalent change requirements across structurally different alternatives.

Do not define the Expected Change Area with one topology's filenames. Define common **architecture roles**, then map those roles to each alternative's actual components/files before implementation.

Required order:

```text
Common Evolution Requirement
  -> Expected Change Roles
  -> alternative-specific role mapping
  -> freeze
  -> implementation
  -> diff + acceptance/regression evaluation
```

If the Expected Change Area or role mapping is changed after the result is visible, create a new scenario/benchmark version. Do not retroactively relabel unexpected propagation as expected change.

## External-validity track

Actual models and Agents should also be exercised, but their results answer a different question:

> Does the frozen qualification corpus still represent behavior seen with current real dependencies?

This track can detect replay drift, missing behavior classes and provider-specific surprises. It should not be conflated with the controlled architecture score.

## Versioning

Each benchmark result should identify at least:

- benchmark version;
- scenario corpus version;
- scenario taxonomy version where applicable;
- semantic trace/oracle version;
- constraint-manifest version where applicable;
- architecture alternative/version;
- source Git commit;
- scoring version;
- machine/environment profile.

Evolution experiments additionally identify:

- evolution scenario/taxonomy version;
- Expected Change Area role-set version;
- alternative role-mapping version;
- baseline and result Git commits;
- acceptance/regression suite versions.

A scoring change must not mutate prior raw runs. Recompute derived metrics into a new version.

## Relationship to current repository documents

- `docs/evaluation/evaluation-strategy.md` remains unchanged in this checkpoint.
- `docs/evaluation/architecture-experiment-methodology.md` explains how these principles are applied.
- `docs/evaluation/quality-attributes/QA-01-fast-task-e2e-responsiveness.md` defines the responsiveness Primary Metric.
- `docs/evaluation/quality-attributes/QA-02-via-interaction-orchestration-correctness.md` defines AECR and its episode/constraint semantics.
- `docs/evaluation/quality-attributes/QA-03-change-flexibility.md` defines CCR and Expected Change Area semantics.
- `benchmark/schemas/run-event-schema.md` defines runtime/interaction raw run evidence.
- `benchmark/schemas/scenario-constraint-schema.md` defines topology-neutral correctness manifests.
- `benchmark/schemas/evolution-scenario-schema.md` defines topology-neutral evolution requirements and Expected Change Areas.
- `benchmark/schemas/evolution-run-schema.md` defines immutable QA-03 change/test/diff evidence.

The QA and DP index will be coherently rebaselined after QA-01~QA-04 are defined.
