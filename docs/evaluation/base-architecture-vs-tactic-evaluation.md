# Base Architecture vs Tactic Evaluation

## Status

**DP-00 Evaluation Methodology — Pre-Prototype / Pre-Pilot**

This document defines how the DP-00 experiment separates **Base Architecture** effects from **optimization tactic** effects.

It complements:

- `docs/evaluation/evaluation-strategy.md`
- `docs/evaluation/evaluation-principles.md`
- `docs/evaluation/architecture-experiment-methodology.md`
- `docs/architecture/decision-points/DP-00-executable-architecture-spec.md`

It does not select an alternative or modify the Approved Baseline.

---

# 1. Why the separation is required

DP-00 is intended to answer a structural question:

> Where should reasoning/execution responsibilities live?

If one alternative is evaluated with speculative routing, fused prompts, classifier routing, aggressive caching, or partial-ASR precomputation while another is left unoptimized, the experiment no longer isolates the architecture topology.

The independent variable becomes a mixture of:

```text
Architecture placement
+ optimization maturity
+ model/prompt design
+ cache behavior
+ classifier choice
```

That would make the DP-00 result difficult to defend.

Therefore the evaluation is explicitly two-phase.

---

# 2. Phase 1 — Base Architecture Evaluation

```text
A / B / C / D Base Architecture
        ↓
Controlled benchmark
        ↓
QA-01 / QA-02 / QA-03 / QA-04
        ↓
Base architecture trade-off
```

The Base must contain everything required to make the alternative **functionally complete and architecturally correct**, but not optional mechanisms whose main purpose is to optimize the score.

## Base includes

- responsibilities inherent to the alternative;
- required software components/seams;
- state lookup and state ownership;
- capability/contract lookup;
- policy/consent check;
- deterministic validation required by an explicit contract/policy/state invariant;
- task identity, correlation, follow-up, result binding;
- dedicated semantic GenAI decision where that semantic responsibility is part of the architecture;
- executor-internal reasoning where the alternative places domain execution authority there.

## Base excludes optional optimization tactics

- Cross-component GenAI fusion;
- dedicated classical/ML classifier routing optimization;
- embedding-based semantic router optimization;
- speculative execution/routing;
- parallel inference optimization;
- prompt optimization specific to one alternative;
- cache optimization specific to one alternative;
- partial-ASR semantic pre-routing;
- semantic result precomputation;
- architecture-specific prefetch solely added to improve measured latency.

A mechanism that is necessary for correctness is not removed merely because it has a performance cost. Conversely, a mechanism is not placed into Base merely because it could improve one alternative's benchmark score.

---

# 3. Phase 2 — Tactic Evaluation

After Base trade-offs are understood, a tactic can be evaluated against a frozen Base:

```text
Base Architecture
    ↓
Base result

Base Architecture + Tactic
    ↓
new result

Delta
    = tactic effect on QA trade-off
```

Example:

```text
A Base
  Intent Refiner GenAI
  -> Agent Router GenAI

A + Fused Semantic Decision tactic
  one GenAI generation
  -> intent + Agent decision
```

The tactic may reduce QA-01 latency and QA-04 logical call count, but can also:

- increase semantic coupling;
- enlarge prompt/schema blast radius;
- reduce independent replaceability;
- change QA-03 containment;
- change QA-02 validation/error-isolation behavior.

Those effects should be measured as the tactic's trade-off rather than retrospectively attributed to the Base architecture.

---

# 4. Architecture vs Tactic test

Use this reviewer-facing test for every proposed mechanism.

```text
Architecture question:
  Who owns this decision responsibility and state?

Tactic question:
  How does that owner implement the responsibility more efficiently,
  accurately, or responsively?
```

Examples:

| Statement | Classification | Reason |
| --- | --- | --- |
| VIA Agent Router owns top-level Agent selection in A | Architecture | Changes responsibility placement. |
| D Execution Path Selector owns topology + executor selection | Architecture | Defines the alternative's execution boundary. |
| C VIA Fast Path owns bounded local capability | Architecture | Capability ownership is C's defining placement choice. |
| B ARGO owns primary request interpretation and initial specialist delegation | Architecture | This is B's boundary-challenging topology. |
| Agent Router uses an LLM vs classifier | Tactic/mechanism choice, after Base responsibility is fixed | Owner remains the same; mechanism changes. |
| Intent and Router decisions are fused into one cross-component generation | Tactic | Changes inference optimization/coupling without changing nominal responsibility labels. |
| Semantic result is prefetched before EOS | Tactic | Optimizes timing rather than basic responsibility ownership. |
| Prompt cache is enabled | Tactic | Runtime efficiency mechanism. |

---

# 5. Base decision-mechanism discipline

Base implementations follow:

```text
state / fact / contract / policy truth
    -> deterministic software

open-ended natural-language semantic judgment
    -> GENAI_DEDICATED at the architecture owner

cross-component semantic fusion
    -> not Base
    -> GENAI_FUSED tactic

classical ML / embedding selection
    -> not Base
    -> tactic
```

This rule is not an argument that deterministic code is always better than learned methods. It is an **experimental control**: obvious structured truths should not be converted into artificial GenAI calls merely to make alternatives look symmetrical.

Likewise, semantic decisions should not be pre-solved by benchmark code; the Base owner must still exercise the real architecture decision seam.

---

# 6. Component count and model-call count are independent

Do not infer QA-04 from a component diagram.

```text
Component exists
    != Generative Model Call occurred
```

Example:

```text
Intent Refiner
  semantic GenAI = 1 call

Agent Router
  capability contract filter = deterministic
  semantic ranking not required in this fixture = 0 calls
```

A two-component path can therefore produce one Generative Model Call.

Conversely, one component can produce multiple logical generations through clarification or retry.

QA-04 is reconstructed from ModelCall telemetry, not component count.

---

# 7. Cross-component GenAI fusion rule

Independent Base semantic responsibilities must remain distinguishable during Phase 1.

If A specifies:

```text
Intent Refiner
  owns intent/goal semantic normalization

Agent Router
  owns top-level Agent semantic selection
```

then a single model generation that simultaneously decides both and leaves the components as passive field-forwarders would move actual semantic authority into a fused inference seam.

That is `GENAI_FUSED` and is evaluated later as a tactic.

The same rule applies to D's Intent Refiner + Execution Path Selector.

## B exception is structural, not an optimization exemption

B deliberately places primary request interpretation, domain reasoning, and self-vs-specialist initial execution choice in **ARGO as one primary execution runtime**.

An ARGO generation that combines domain interpretation and initial delegation is `MIXED` for QA-04, but it is not an artificial cross-component fusion tactic because those responsibilities are structurally co-located by Alternative B itself.

The experiment must not force B to invent a VIA Intent Refiner/Router split merely to mimic A.

---

# 8. Tactic catalog for later experiments

The list below is provisional and does not imply all tactics will be implemented.

| Tactic candidate | Primary reason to evaluate | Likely QA interactions |
| --- | --- | --- |
| Cross-component GenAI fusion | reduce sequential semantic calls | QA-01, QA-03, QA-04, possibly QA-02 |
| Classical ML path classifier | reduce GenAI route decision | QA-01, QA-02, QA-04 + resource diagnostics |
| Embedding semantic router | faster candidate selection | QA-01, QA-02, QA-03, QA-04/resource |
| Partial-ASR semantic pre-routing | overlap decision with speech | QA-01, QA-02, state complexity |
| Parallel inference | reduce critical-path latency | QA-01, resources; QA-04 call count may not decrease |
| Speculative execution | lower latency when prediction is right | QA-01 vs QA-02/safety/rollback complexity |
| Prompt/cache optimization | reduce inference latency/tokens | QA-01, QA-04 Secondary resource metrics |
| Semantic precomputation | avoid repeated work | QA-01/resource vs freshness/state correctness |

Each tactic requires its own hypothesis, controls, raw telemetry, and comparison version.

---

# 9. Fairness rules for tactic evaluation

A tactic experiment must state:

- which Base Architecture version it extends;
- the tactic's exact responsibility/mechanism change;
- whether the tactic is applicable to all alternatives or topology-specific;
- which QA effects are hypothesized;
- which additional confounders are frozen;
- whether scenario corpus/scoring versions remain unchanged;
- whether a tactic changes the semantic oracle or only mechanism/timing.

If a tactic is only meaningful for one architecture, it can still be evaluated, but it must not be retroactively treated as part of that architecture's Base score.

A final product architecture may select a Base plus tactics. The report should preserve both steps:

```text
Base Architecture Trade-off
        ↓
Tactic Mitigation
        ↓
Improved Architecture / Final Decision
```

---

# 10. Relationship to QA-01~04

## QA-01

Base latency should show the cost/benefit of topology before speculative, caching, fusion, or parallelization tactics hide it.

## QA-02

Base correctness must include required validation/state semantics. Correctness-preserving mechanisms cannot be stripped out merely to reduce Base latency/calls.

## QA-03

Separating tactics is especially important because optimizations such as fusion can improve performance while increasing coupling/change propagation.

## QA-04

Base QA-04 counts actual logical **Generative AI generations** required to commit the execution route. Deterministic lookups contribute zero. Optional learned/classifier routing is kept out of Base and analyzed separately with resource telemetry.

---

# 11. Versioning

Recommended identifiers for future experiment metadata:

```text
base_architecture_spec_version
tactic_id
tactic_version
alternative_id
benchmark_version
scenario_corpus_version
scoring_version
```

A Base definition change that moves responsibility ownership is not a tactic version bump; it is an architecture-spec revision and must be explicitly reviewed.

A tactic change must not silently mutate historical Base results.

---

# 12. Checkpoint decision

Agreed for DP-00 evaluation:

- Phase 1 compares **Base Architecture** A/B/C/D.
- Phase 2 evaluates optional optimization tactics separately.
- All Base alternatives provide the same Product Capability.
- Base differences are responsibility placement and execution topology.
- Structured state/fact/contract/policy truth uses deterministic code.
- Semantic judgment uses the Base responsibility owner.
- Component count is not model-call count.
- Cross-component GenAI fusion is not used in Base.
- B's ARGO-co-located mixed reasoning/delegation remains part of B's structural definition.
- Actual tactic benefit is measured later rather than assumed.

## Next step

The next benchmark-design checkpoint must define the common scenario contract and the common-vs-variable experimental boundary so prototypes cannot accidentally share architecture logic that is supposed to remain alternative-specific.
