# AA-003 — QA-03 Flexibility Measurement Rationale

## Status

Architecture Analysis Record — Context Checkpoint 003

This record preserves the reasoning behind `QA-03 — 변경 대응 용이성 (Flexibility)` and its Primary Metric, Change Containment Rate (CCR).

It is not an ADR and does not modify `docs/requirements/requirements-v1.1.md`.

## Why Flexibility is a top architectural driver

VIA sits between fast-moving dependencies and product-facing interaction semantics. The architecture must expect change in at least four directions:

- Downstream Agent ecosystem and Agent capabilities;
- Model and realtime voice providers;
- context/input sources and PC integration surfaces;
- capability placement and common Agent contracts.

A topology can work well today yet impose high future change cost if a nominally local evolution requirement propagates into unrelated core responsibilities. For VIA, this is not a cosmetic maintainability concern. It affects whether Agent, Model, Voice Runtime and Context Source evolution can happen independently without repeatedly redesigning the product core.

This is especially important for DP-00 because the alternatives deliberately place reasoning/execution authority differently across VIA, ARGO and specialized Agents. Placement changes coupling. Coupling changes where future modifications must propagate.

## Product context: the ecosystem will move

The relevant product assumption is not that one specific Agent or Model will remain stable. The opposite is more realistic:

- a preferred general Agent can change;
- specialized Agents can be added or removed;
- a realtime S2S provider can be replaced;
- an auxiliary model provider can change;
- new context sources can become available;
- local capabilities can move across the VIA/Agent boundary;
- shared progress/cancel/follow-up/resource contracts can evolve.

An architecture that requires unrelated existing core modules to change for these expected evolutions is structurally less flexible, even if the immediate feature can eventually be implemented.

## Relationship to DP-00

DP-00 compares reasoning/execution ownership at the Integrated Product boundary.

Examples of possible consequences:

- **Thin VIA** may pay extra runtime hops but can confine Agent replacement to adapters/registry/configuration if the common boundary is stable.
- **ARGO-centric Primary Execution** may be fast and operationally simple for ARGO-native work, but changes to Agents, context semantics or execution contracts may propagate into ARGO-centric integration/core logic if that topology makes ARGO structurally privileged.
- **Hybrid Fast Path** introduces an additional evolution dimension: moving bounded capability between Agent ownership and VIA local ownership.
- **Adaptive Per-turn Execution** adds selector/ownership-transfer contracts that must remain stable as execution owners evolve.

The relevant trade-off is therefore not “how many files does an alternative have?” It is whether a change remains inside the architecture roles intended to absorb that kind of change.

## What the metric should measure

The central Flexibility question became:

> **When an expected evolution requirement arrives, does the change finish inside the architecture area designed to absorb it, or does it spread into unrelated existing core responsibilities?**

This points toward a containment metric rather than a size, effort or raw-diff metric.

## Candidate metric 1 — Change Propagation Ratio (CPR)

A candidate considered was **Change Propagation Ratio (CPR)**.

One possible formulation compares observed propagation with propagation in a directly coupled reference architecture.

Why it was rejected as the Primary Metric:

1. It requires a separate `direct-coupled reference architecture` as denominator/reference.
2. A/B/C/D comparison then depends on a fifth architecture that is not itself one of the DP-00 alternatives.
3. The denominator can have different meaning across evolution scenarios.
4. The explanation becomes harder: an architecture score depends on how a synthetic reference topology was defined.
5. Disagreement about the reference can dominate disagreement about the alternatives.

CPR may remain a Secondary or analytical metric if a useful reference model later exists, but it should not define the QA-03 score.

## Candidate metric 2 — Ratio of modified components

Another candidate was:

```text
modified existing components / total components
```

This creates a denominator artifact when architecture sizes differ.

Example:

```text
Alternative A: 1 / 10 components changed = 10%
Alternative D: 1 / 20 components changed = 5%
```

If both alternatives changed exactly one well-isolated adapter, D would appear twice as good simply because it has more components.

DP-00 alternatives can legitimately have different topology sizes, so total component count is not a neutral denominator.

This metric was rejected as Primary.

## Candidate metric 3 — Raw modified component count

A raw count removes the denominator problem but creates a granularity problem.

```text
one large monolithic core changed = 1 component
one small adapter changed         = 1 component
```

Those changes are not architecturally equivalent. Component boundaries are themselves part of the architecture under test, so counting them without semantic roles can reward coarse-grained packaging.

Raw counts remain diagnostic but not Primary.

## Candidate metric 4 — Development time

Development time sounds intuitive but is dominated by confounding variables such as:

- developer skill and familiarity;
- use of Codex/AI coding tools;
- IDE/tooling;
- repository familiarity;
- test-writing experience;
- debugging strategy.

The experiment aims to measure SW architecture, not human/tool productivity. Development time may be recorded operationally but is unsuitable as an architecture-only Primary Metric.

## Candidate metric 5 — Lines of code changed

LOC is sensitive to:

- coding style;
- programming language;
- boilerplate;
- generated code;
- refactoring style;
- formatting and code movement.

A strongly isolated change can still have many lines, while a dangerously coupled change can alter a few strategically central lines.

LOC is therefore Secondary diagnostic evidence only.

## Why Change Containment Rate was selected

The selected Primary Metric is **변경 영향 제한율 (Change Containment Rate, CCR)**.

The metric is intentionally scenario-level and architecture-role-based.

Plain-text definition:

```text
CCR (%) =
  (예상 변경 범위 안에서만 성공적으로 완료된 변경 시나리오 수
   / 전체 평가 대상 변경 시나리오 수)
  × 100
```

A scenario counts as a contained success only when all three conditions hold:

```text
1. Feature Acceptance Test PASS
2. Existing Regression Test PASS
3. No modification propagates outside the pre-frozen Expected Change Area
```

Therefore:

```text
Feature PASS + Regression PASS + No unexpected propagation
= Contained Change Success
```

CCR was selected because the measured property is directly tied to architecture structure:

- responsibility separation;
- abstraction boundary quality;
- coupling;
- stable extension points;
- independence of existing responsibilities.

It does not depend on a synthetic reference architecture or on topology component count.

## Expected Change Area

The benchmark uses the plain term **예상 변경 범위 (Expected Change Area)**.

Definition:

> 특정 변경 요구를 적용할 때, architecture 설계상 수정되어도 된다고 사전에 예상한 역할/영역.

Example — add a new Specialized Agent:

```text
Expected Change Area
- Agent-specific integration
- capability / registry information
- scenario-specific tests

Outside the Expected Change Area
- existing Voice Engine
- existing Context Engine
- existing Task Manager core
- unrelated Agent adapter
- stable common Agent contract
```

If the implementation needs to modify Router/Task/Voice/Context core that was not part of the predeclared role set, the change has propagated unexpectedly.

## Why role-based boundaries are required

A/B/C/D do not have identical components or file layouts. The benchmark must not encode one topology as the standard by saying:

```text
src/router.rs must not change
```

Some alternatives may not contain that file or even an explicit Router component.

The fair comparison flow is:

```text
Common Evolution Requirement
        ↓
Common architecture-role Expected Change Area
        ↓
Alternative-specific role → component/file mapping
        ↓
Freeze mapping before implementation/result observation
        ↓
Implement change
        ↓
Analyze diff + tests
```

Example common roles for “New Specialized Agent”:

```text
Expected roles
- AGENT_SPECIFIC_INTEGRATION
- CAPABILITY_REGISTRATION
- SCENARIO_TEST
```

Alternative A might map these roles to a generic Agent adapter directory and registry table. Alternative B may map them to ARGO delegation integration points. Both are judged against their frozen role mapping rather than against identical filenames.

## Why the mapping must be frozen before results

If Expected Change Area or role mapping can be edited after seeing a diff, every unexpected propagation can be reclassified as “expected.” CCR would become unfalsifiable.

Therefore final qualification requires these artifacts to be frozen before implementation:

- evolution scenario version;
- expected change roles;
- forbidden/outside roles;
- alternative-specific role mappings;
- acceptance tests;
- regression suite/version;
- scoring version.

A change to these after final results requires a new experiment/scenario version rather than retroactive reinterpretation.

## Binary CCR: advantage and limitation

CCR is intentionally binary per scenario:

```text
contained change success = 1
otherwise                = 0
```

Advantages:

- easy to explain;
- avoids arbitrary weighting among coupling symptoms;
- requires feature correctness and regression safety, not just a small diff;
- directly answers whether the designed boundary contained the change.

Known limitation:

Two failed scenarios can have very different propagation severity.

Example:

```text
Alternative A failed scenario:
  1 unexpected architecture area changed

Alternative B failed scenario:
  5 unexpected architecture areas changed
```

Both contribute `0` to CCR.

This is why **Unexpected Changed Architecture Areas Count** and other detailed raw metrics are mandatory. CCR is the score; raw/Secondary evidence explains severity.

## Why Secondary/raw metrics must be retained

Store enough evidence to calculate at least:

- Unexpected Changed Architecture Areas Count;
- changed existing architecture roles count;
- changed files count;
- lines added/deleted;
- stable contract changes;
- existing adapter changes;
- regression failures;
- change-propagation depth/stages.

This follows the project-wide principle:

> 측정하지 않은 값은 나중에 복구할 수 없지만, raw data로 보존한 값은 나중에 다른 metric으로 재해석할 수 있다.

If CCR later has insufficient discrimination, the methodology can be revised only from preserved raw evidence and with a new scoring version.

## Why balanced evolution taxonomy is necessary

A flexibility benchmark can easily be biased by choosing only changes naturally favored by one topology.

For example:

- only “replace Agent” scenarios might favor a strong generic Agent abstraction;
- only “move capability into VIA Fast Path” scenarios might favor a topology already structured around local execution;
- only ARGO-specific changes could unfairly reward the ARGO-centric topology.

The scoring corpus must therefore cover multiple change dimensions.

Minimum taxonomy:

- **E1 Agent changes** — replace general Agent, add specialized Agent;
- **E2 Voice / Model changes** — replace S2S runtime or auxiliary model provider;
- **E3 Context / Input changes** — add Context Source or Interaction Input Adapter;
- **E4 Capability-placement changes** — move a capability into VIA Fast Path or add a bounded local capability;
- **E5 Contract changes** — extend capability/resource metadata or progress/cancel/follow-up contracts.

The exact count is not frozen yet. Pilot must check:

- implementability;
- architecture sensitivity;
- category coverage;
- whether one topology is overrepresented by scenario design.

Production usage-frequency weighting may be calculated as a Secondary sensitivity analysis, but does not silently redefine the Primary CCR population.

## Quality Attribute Scenario interpretation

QA-03 evaluates an **evolution episode**, not a runtime user interaction episode.

The artifact under change is the **VIA Integrated Product SW architecture**, because DP-00 alternatives place responsibilities differently across the product topology.

The test asks whether each alternative can satisfy the same evolution requirement and preserve existing behavior while containing change to its predeclared roles.

## Relationship to QA-01

QA-03 intentionally creates a major trade-off with responsiveness.

Examples:

- An ARGO-centric topology can reduce direct-path hops and improve QA-01 for ARGO-native work, yet score poorly on QA-03 if Agent/context/runtime evolution propagates through ARGO-centric core integration.
- A Thin VIA topology can incur adapter/routing overhead in QA-01, yet score highly on QA-03 if Agent or Runtime changes remain confined to adapter/registry roles.

The architectural point is:

> **빠른 구조가 반드시 변경하기 쉬운 구조는 아니다.**

The purpose of separate QAs is to expose this trade-off rather than collapse it into one weighted score.

## Checkpoint decisions

Agreed:

- QA-03 characteristic is Flexibility.
- Primary Metric is Change Containment Rate (CCR).
- CCR uses scenario-level contained-change success.
- Expected Change Area is role-based, not component-count/file-name based.
- Alternative role mappings are frozen before change implementation and result observation.
- Feature acceptance + regression + containment are all required for numerator membership.
- CPR, component ratio/count, development time and LOC are not Primary Metrics.
- Balanced E1~E5 evolution coverage is required before final scoring.
- Binary CCR limitations are accepted only with mandatory detailed raw/Secondary evidence.
- Runtime `run-event-schema.md` is not overloaded with evolution-change data; evolution receives separate schema contracts.

Not yet decided:

- exact number of scenarios per E1~E5 category;
- final CCR thresholds F1~F5;
- concrete JSON/YAML serialization format;
- exact architecture-role vocabulary/version;
- final QA renumbering/rebaseline in requirements/traceability;
- DP-00 winning alternative.
