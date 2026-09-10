# Architecture Evaluation Principles

## Status

Architecture Context Checkpoint 001/002/003/004 + Top-QA Cross-review — detailed evaluation rules for the vNext architecture baseline.

These principles are the detailed rules behind the central `docs/evaluation/evaluation-strategy.md`. The Central vNext Rebaseline now aligns the working requirements, QA↔DP traceability and central evaluation strategy with these rules.

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
10. **Actual-model validation is separated from the Architecture score.** Real GPT/Qwen/local-model/Agent runs provide fidelity/external-validity evidence; they do not silently reintroduce stochasticity into the architecture-only comparison.
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
22. **A Primary Metric's measurement boundary must follow the same architectural responsibility across topology locations.** Moving routing/delegation logic behind an intermediate component start event must not remove it from measurement. QA-04 therefore uses Execution Route Commit rather than intermediate execution start.
23. **Controlled benchmark fixtures must not become the dominant cause of a Primary Metric.** Equality across alternatives is necessary but not sufficient; external/downstream fixture behavior must also be calibrated so it does not overwhelm the architecture effect under study.
24. **QA-02 constraints must not be derived from the evaluated topology.** Required/Allowed/Forbidden rules come from functional requirements, policy, capability contracts and scenario semantics—not from the fact that one alternative happens to contain a Fast Path, Router or ARGO-first path.
25. **Model/provider/prompt/cache profiles used for architecture qualification are versioned and frozen.** Logical model-call count does not claim physical compute equivalence; token/cache/resource/cost telemetry is retained for later derived analysis.
26. **Scored QAs and mandatory qualification gates may be separated.** A security/privacy/recovery/trusted-boundary requirement that is non-compensable should be a must-pass condition rather than another trade-off star score.
27. **Any minimum correctness qualification gate is calibrated and frozen before final alternative results are known.** Do not hide correctness as a penalty inside latency or model-call formulas; use a separate gate when final selection requires minimum acceptable AECR.

## Raw-data principle

> **측정하지 않은 값은 나중에 복구할 수 없지만, raw data로 보존한 값은 나중에 다른 metric으로 재해석할 수 있다.**

This principle drives benchmark instrumentation. Store architecture-relevant event timestamps, ownership decisions, canonical correctness outcomes, constraint results, model-call events, profile/version metadata and resource telemetry even when they are not part of today's Primary Metric.

For QA-02, storing only final `episode_exact_conform` or AECR is prohibited. Preserve the manifest version, actual canonical trace, each applicable constraint result and failure ids/reasons.

For QA-03, storing only final CCR or `scenario_change_contained` is prohibited. Preserve the frozen Expected Change Area, role mapping, actual changed roles/files, acceptance/regression results and unexpected propagation evidence.

For QA-04, storing only an episode call count is prohibited. Preserve per-logical-generation ModelCall evidence, call class, route-commit state, model/prompt/cache profile metadata and resource/token diagnostics.

## Metric discipline

A QA should separate:

- the **quality question** being evaluated;
- the **Primary Metric** that determines the architecture score;
- **Secondary Metrics** used for diagnosis;
- the **scenario population** over which the metric is valid;
- the **measurement start/end boundary** where applicable;
- benchmark controls and exclusions;
- the scoring-version thresholds.

Do not combine independent concerns by arbitrary weighted formulas merely to obtain one number.

Examples:

- responsiveness and correctness remain separate; failed tasks are not converted into fabricated large latency values;
- component-accuracy averages are not the correctness Primary when one critical error invalidates the user-goal episode;
- topology size, component count, LOC and developer time do not replace the Flexibility question of unexpected change propagation;
- logical model-call count does not replace measured latency or physical compute cost.

## Architecture isolation

During runtime Architecture Qualification:

```text
Independent variable
    = SW Architecture Alternative

Dependent variables
    = QA Primary Metrics

Controlled / confounding variables
    = model semantic behavior
    = Agent behavior
    = tool/service behavior and latency
    = network behavior where possible
    = machine / power / background state
    = scenario corpus
    = model / prompt / cache profiles where relevant
```

The comparison should answer: **if the semantic/dependency behavior is held equivalent and non-dominant, what effect does the architecture topology itself create?**

For evolution experiments, freeze the common evolution requirement, architecture-role Expected Change Area, alternative role mappings, acceptance tests and regression suite; then observe how each architecture must actually change.

## Dependency-fixture non-dominance

For latency QAs, deterministic dependency latency can still confound the result if its fixed magnitude dominates architecture overhead.

Architecture Qualification therefore requires:

```text
Equality:
  comparable alternatives receive the same dependency behavior.

Non-dominance:
  controlled dependency behavior is calibrated so the fixture does not
  mechanically determine the Primary Metric instead of the architecture.
```

Exact controlled latency values are selected during Pilot and frozen before final evaluation.

Actual production dependency latency belongs in the separate fidelity/real-stack track.

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

When one scenario permits several correct implementations, the oracle expresses:

```text
Required   — outcomes/relations that must hold
Allowed    — acceptable alternatives/sets
Forbidden  — outcomes/actions that must not occur
```

Constraint sources must be architecture-neutral evidence such as functional requirements, policy, capability contracts and scenario semantics.

A topology-derived claim such as “D has a Fast Path, therefore FAST is correct” is invalid.

## Corpus composition and sensitivity

Architecture Qualification uses coverage-balanced corpora that deliberately represent architecture-sensitive failure surfaces.

The following must be versioned/frozen where applicable:

- taxonomy;
- corpus composition;
- eligibility rules;
- scenario/constraint manifests;
- semantic replay set;
- dependency-latency profile;
- model/prompt/cache profiles;
- scoring version;
- qualification-gate version.

Usage-frequency-weighted results may be derived separately as Secondary sensitivity analysis.

## Evolution-benchmark boundary discipline

Flexibility experiments compare semantically equivalent change requirements across structurally different alternatives.

Do not define Expected Change Area with one topology's filenames. Define common **architecture roles**, then map those roles to each alternative's actual components/files before implementation.

Required order:

```text
Common Evolution Requirement
  -> Expected Change Roles
  -> alternative-specific role mapping
  -> freeze
  -> implementation
  -> diff + acceptance/regression evaluation
```

If Expected Change Area or role mapping is changed after the result is visible, create a new scenario/benchmark version. Do not retroactively relabel unexpected propagation as expected change.

## Execution-route measurement discipline

For QA-04, the boundary is **Execution Route Commit**:

> final domain execution route is operationally committed and no further owner/delegation decision is needed before domain execution continues.

This prevents an alternative from hiding routing/delegation calls merely by starting an intermediate runtime earlier.

Preserve intermediate `execution_owner_confirmed_ts` and `execution_started_ts` as diagnostics, but do not use them as the authoritative QA-04 end boundary.

## Scored QAs and mandatory gates

A final architecture decision may use both:

```text
Scored Architectural Drivers
  QA-01
  QA-02
  QA-03
  QA-04

Must-pass Architecture Constraints / Gates
  security
  privacy/context-sharing policy
  cancellation semantics
  task-state integrity
  failure containment
  mandatory recovery
  trusted-boundary requirements
```

Non-compensable constraints are not made less important by keeping them outside the Top-4 star score.

A minimum QA-02 correctness gate may also be used for final DP-00 eligibility. Its numeric threshold remains TBD until Pilot/calibration and must be frozen before final results.

## External-validity track

Actual models, Agents, networks and tools are exercised separately to ask:

> Does the frozen qualification setup still represent behavior seen with real dependencies?

This track detects replay drift, missing behavior classes, provider-specific surprises and real-stack latency/resource effects. It should not be conflated with the controlled architecture score.

## Versioning

Each benchmark result should identify at least:

- benchmark version;
- scenario corpus/taxonomy version;
- semantic trace/oracle version;
- constraint-manifest version where applicable;
- architecture alternative/version;
- source Git commit;
- scoring version;
- qualification-gate version where applicable;
- machine/environment profile;
- dependency-latency profile where applicable;
- model/prompt/cache profiles where applicable.

Evolution experiments additionally identify Expected Change Area/role mapping, baseline/result commits, and acceptance/regression suite versions.

A scoring or gate change must not mutate prior raw runs. Recompute derived metrics into a new version.

## Relationship to current repository documents

Central vNext navigation is now aligned around these artifacts:

- `docs/requirements/requirements-vNext.md` — vNext Architecture Evaluation Working Baseline;
- `docs/architecture/qa-dp-traceability.md` — DP-00 / Top-QA / Legacy-QA mapping;
- `docs/evaluation/evaluation-strategy.md` — central evaluation pipeline and gate/scoring policy;
- `docs/evaluation/architecture-experiment-methodology.md` — detailed controlled-experiment methodology;
- `docs/evaluation/quality-attributes/QA-01-fast-task-e2e-responsiveness.md` — FTOL p95 and dependency-latency non-dominance;
- `docs/evaluation/quality-attributes/QA-02-via-interaction-orchestration-correctness.md` — AECR and constraint semantics;
- `docs/evaluation/quality-attributes/QA-03-change-flexibility.md` — CCR and Expected Change Area semantics;
- `docs/evaluation/quality-attributes/QA-04-model-call-overhead.md` — Average Model Calls to Commit Execution Route;
- `docs/architecture/analysis/AA-005-top-qa-cross-review.md` — Top-QA independence/neutrality/readiness review;
- `docs/architecture/analysis/final-report-evidence-index.md` — reviewer-question/evidence navigation;
- `benchmark/schemas/run-event-schema.md` — runtime episode evidence;
- `benchmark/schemas/scenario-constraint-schema.md` — topology-neutral correctness manifests;
- `benchmark/schemas/model-call-schema.md` — QA-04 per-logical-generation evidence;
- `benchmark/schemas/evolution-scenario-schema.md` and `evolution-run-schema.md` — QA-03 evolution evidence.

`requirements-v1.1.md` remains the immutable Approved Baseline; central vNext rebaseline does not change it.
