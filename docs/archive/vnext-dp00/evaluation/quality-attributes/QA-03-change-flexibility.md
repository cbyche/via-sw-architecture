# QA-03 — 변경 대응 용이성 (Flexibility)

## Status

**Proposed vNext — QA definition agreed, scoring thresholds TBD**

This document is part of the QA rebaseline work. It does not replace the Approved Baseline QA numbering yet. `docs/requirements/requirements-v1.1.md` remains unchanged until a coherent vNext rebaseline is reviewed.

## Name

**QA-03 변경 대응 용이성 (Flexibility)**

## ISO quality characteristic

**ISO/IEC 25010:2023 — Flexibility**

## 핵심 질문

> **Agent, Model, Voice Runtime, Context Source 같은 요소가 바뀌었을 때, 그 변경이 다른 기존 SW 영역까지 얼마나 적게 번지는가?**

This QA evaluates whether the SW architecture contains expected product evolution inside the roles designed to absorb that change.

The focus is not how quickly a developer types code or how few lines happen to change. The focus is whether responsibility separation, abstraction boundaries and extension points prevent one evolution requirement from propagating into unrelated existing architecture.

## Primary Metric

### 변경 영향 제한율 — Change Containment Rate (CCR)

Plain-text definition:

```text
CCR (%) =
  (예상 변경 범위 안에서만 성공적으로 완료된 변경 시나리오 수
   / 전체 평가 대상 변경 시나리오 수)
  × 100
```

Example:

```text
10개의 변경 시나리오 중 8개가 예상 변경 범위 안에서 성공적으로 완료되었다면:
CCR = (8 / 10) × 100 = 80%
```

- Unit: **percent (%)**
- Direction: **higher is better**
- Architecture score: one 0–5 score derived from CCR using a frozen scoring version
- Primary Metric rule: **CCR alone determines the QA-03 0–5 score**

A scenario contributes to the CCR numerator only when it satisfies all of the contained-change success conditions below.

## 예상 변경 범위 — Expected Change Area

**예상 변경 범위 (Expected Change Area)** means:

> 특정 변경 요구를 적용할 때, architecture 설계상 수정되어도 된다고 사전에 예상한 역할/영역.

Example — add a new Specialized Agent:

```text
예상 변경 범위
- 새 Agent Adapter / Agent-specific integration
- capability / registry 정보
- 해당 Agent용 test

예상 범위 밖
- 기존 Voice Engine
- 기존 Context Engine
- 기존 Task Manager core
- unrelated Agent adapter
- stable common Agent contract
```

If the change can be completed by modifying only the expected roles, the architecture contained the change.

If unrelated existing roles such as core routing/task/voice/context responsibilities must also change, the change propagated beyond the expected area.

## Architecture alternatives are mapped by role, not file name

DP-00 alternatives A/B/C/D can have different component names and counts. Therefore the benchmark must not define containment using one alternative's physical layout.

Wrong:

```text
src/router.rs 수정 금지
```

Correct:

```text
Evolution Requirement: New Specialized Agent

Expected Change Roles
- AGENT_SPECIFIC_INTEGRATION
- CAPABILITY_REGISTRATION
- SCENARIO_TEST
```

Each alternative then declares a mapping from these common architecture roles to its actual components/files before the experiment.

Required flow:

```text
공통 Evolution Requirement
       ↓
공통 Architecture Role 기준 Expected Change Area
       ↓
Alternative별 실제 component/file mapping
       ↓
mapping freeze
       ↓
Change implementation
       ↓
Diff + acceptance/regression analysis
```

The mapping must be frozen **before** implementation/result observation. It must not be rewritten after seeing which files changed.

## Contained Change Success

Change containment alone is not enough. A tiny diff that fails the requested feature is not flexibility.

A scenario counts in the CCR numerator only when all three conditions pass:

```text
1. 변경 요구 기능의 Acceptance Test PASS
2. 기존 기능 Regression Test PASS
3. 실제 변경이 Expected Change Area 밖의 기존 architecture로 전파되지 않음
```

Equivalently:

```text
Feature PASS + Regression PASS + No unexpected propagation
= Contained Change Success
```

If any condition fails, `scenario_change_contained = false` and the scenario contributes `0` to the CCR numerator.

## Why CPR is not the Primary Metric

**Change Propagation Ratio (CPR)** was considered but rejected as the Primary Metric because:

- it requires an additional `direct-coupled reference architecture` or similar denominator;
- A/B/C/D would then depend on a fifth reference topology;
- denominator meaning can vary by evolution scenario;
- the metric becomes harder to explain and easier to dispute than the architecture property itself.

CPR may remain a Secondary/analysis candidate if a stable reference is later useful.

## Why modified-component ratio is not the Primary Metric

A ratio such as:

```text
modified existing components / total components
```

is biased by topology size.

Example:

```text
Alternative A: 1 / 10 changed = 10%
Alternative D: 1 / 20 changed = 5%
```

If both architectures changed one properly isolated adapter, D should not automatically receive twice the flexibility credit because it has more components.

## Why raw component count is not the Primary Metric

Raw component count has a granularity problem:

```text
large monolithic core changed = 1 component
small isolated adapter changed = 1 component
```

Those are architecturally different changes but produce the same count.

## Why development time is not the Primary Metric

Development time is strongly affected by confounding variables:

- developer skill;
- repository familiarity;
- Codex/AI coding-tool capability;
- IDE/tooling;
- test/debugging experience.

These factors can dominate the architecture effect.

## Why LOC is not the Primary Metric

LOC is sensitive to:

- coding style;
- programming language;
- boilerplate/generated code;
- refactoring method;
- formatting/movement.

LOC remains useful diagnostic evidence, but is not used for the architecture score.

## Why CCR was selected

The intended Flexibility property is:

> **Does an expected change stop at the boundary intended to absorb it, or does it leak into unrelated existing core architecture?**

This is directly influenced by:

- responsibility separation;
- abstraction boundaries;
- coupling;
- extension points;
- stability of common contracts.

CCR is therefore more directly architecture-centered than component counts, LOC, development time or a reference-topology propagation ratio.

## Balanced Evolution Scenario Taxonomy

The final architecture-qualification corpus must include balanced change types rather than only changes favored by one topology.

### E1 — Agent changes

Examples:

- replace the general-purpose Agent;
- add a new Specialized Agent.

### E2 — Voice / Model changes

Examples:

- replace the S2S Voice Runtime provider;
- replace an auxiliary Model provider.

### E3 — Context / Input changes

Examples:

- add a new Context Source;
- add a new Interaction Input Adapter.

### E4 — Capability-placement changes

Examples:

- move an existing Agent capability to the VIA Fast Path;
- add a new bounded local capability.

### E5 — Contract changes

Examples:

- extend Agent capability/resource metadata;
- extend progress/cancel/follow-up event contracts.

The exact scenario count is **TBD**.

Pilot evaluation must inspect:

- implementability;
- architecture sensitivity;
- category balance;
- whether the scenario set implicitly favors one topology.

Before final evaluation, freeze:

- taxonomy version;
- scenario set/version;
- Expected Change Areas;
- alternative role mappings;
- acceptance/regression tests;
- scoring version.

Production usage-frequency weighting is not applied silently to Primary CCR. If useful, it is reported as a Secondary sensitivity analysis.

## Quality Attribute Scenario

| 항목 | 정의 |
| --- | --- |
| **Source** | 제품/Agent/Model/Platform evolution 요구 |
| **Stimulus** | 기존 시스템에 사전 정의된 변경 요구 발생 |
| **Environment** | 각 DP alternative의 동일 baseline version, Expected Change Area와 architecture-role mapping이 사전에 freeze된 상태 |
| **Artifact** | VIA Integrated Product의 SW architecture |
| **Response** | 요구된 변경을 구현하고 기존 동작을 유지하면서 Expected Change Area 밖의 기존 architecture를 수정하지 않음 |
| **Response Measure** | **변경 영향 제한율 (Change Containment Rate, CCR)** |

## Secondary Metrics

The following do not determine the QA-03 score but must remain derivable from raw data:

- **Unexpected Changed Architecture Areas Count**;
- changed existing architecture areas/roles count;
- changed file count;
- added LOC;
- deleted LOC;
- stable contract changes count;
- existing adapter changes count;
- regression test failure count;
- change-propagation depth/stages;
- category-specific CCR for E1~E5;
- usage-weighted CCR sensitivity result if weights are defined.

### Important backup metric: Unexpected Changed Architecture Areas Count

CCR is binary at scenario level. Two alternatives can both obtain 80% while failed scenarios have very different propagation severity.

Example:

```text
Alternative A failed scenario -> 1 unexpected architecture area changed
Alternative B failed scenario -> 5 unexpected architecture areas changed
```

This metric is therefore important diagnostic evidence, but it does **not** determine the Primary score.

## Scoring

QA-03 uses six score bands: **0, 1, 2, 3, 4, 5**.

Thresholds are currently **TBD**.

Plain-text scoring contract:

```text
CCR >= F5                  -> 5점
F4 <= CCR < F5             -> 4점
F3 <= CCR < F4             -> 3점
F2 <= CCR < F3             -> 2점
F1 <= CCR < F2             -> 1점
CCR < F1                   -> 0점
```

Required process:

```text
Pilot
 -> metric behavior 확인
 -> threshold calibration
 -> scoring-v1 freeze
 -> final A/B/C/D evaluation
```

Thresholds must not be changed after inspecting final alternative results to favor a preferred topology.

## Relationship to DP-00 and QA-01

QA-03 creates an important trade-off with QA-01.

Example:

- An ARGO-centric topology can reduce direct-path orchestration and improve QA-01 latency for ARGO-native requests.
- If replacing Agents, Runtime or Context semantics requires changes across ARGO-centric core integration, QA-03 can be lower.

Conversely:

- Thin VIA can add abstraction/adapter hops that hurt QA-01.
- If Agent/Runtime changes remain inside adapter/registry roles, it can score higher on QA-03.

Therefore:

> **빠른 구조가 반드시 변경하기 쉬운 구조는 아니다.**

QA-01 and QA-03 remain separate so this architecture trade-off is visible rather than hidden in one weighted number.

## Raw-data requirements

Runtime interaction event data and evolution experiment data have different semantics.

QA-03 therefore uses dedicated evolution schemas rather than forcing file-diff/test evidence into `benchmark/schemas/run-event-schema.md`.

See:

- `benchmark/schemas/evolution-scenario-schema.md`
- `benchmark/schemas/evolution-run-schema.md`

At minimum preserve:

- evolution/scenario/alternative/version identity;
- baseline and result Git commits;
- expected change roles;
- frozen alternative role mapping;
- changed files and changed architecture roles;
- unexpected changed roles;
- acceptance/regression results;
- stable contract/existing adapter change evidence;
- lines added/deleted;
- `scenario_change_contained`;
- structured failure reason.

Final CCR is a **derived metric**, not raw evidence.

## Related Decision Points

Primary relevance:

- DP-00 — VIA Primary Execution Boundary;
- capability placement;
- generic/specialized Agent integration;
- Model/Voice Runtime abstraction;
- Context Source/Input Adapter structure;
- Agent capability contract;
- progress/cancel/follow-up contract evolution.

Exact QA↔DP numbering will be updated coherently after QA-01~QA-04 formalization.

## Unresolved / TBD

- Final `F1`~`F5` CCR thresholds;
- exact E1~E5 scenario count and corpus composition;
- final architecture-role vocabulary/version;
- concrete schema serialization format;
- final QA numbering/rebaseline in Approved requirements/traceability.
