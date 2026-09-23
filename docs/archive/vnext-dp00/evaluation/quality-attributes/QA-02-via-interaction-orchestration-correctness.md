# QA-02 — VIA Interaction-Orchestration Correctness

## Status

**Proposed vNext — QA definition agreed, scoring thresholds TBD**

This document is part of the QA rebaseline work. It does not replace the Approved Baseline QA numbering yet. `docs/requirements/requirements-v1.1.md` remains unchanged until a coherent vNext rebaseline is reviewed.

## Name

**QA-02 VIA Interaction-Orchestration Correctness**

Korean short name: **VIA 상호작용·실행 조정 정확성**

## ISO quality characteristic

**Functional Suitability / Functional Correctness**

## Architectural concern

> 동일하게 통제된 semantic/context/task-state 입력이 주어졌을 때, VIA Integrated Product의 SW architecture가 사용자 goal을 처리하기 위해 요구되는 context grounding, task association, execution ownership, Agent delegation, clarification, result binding 등의 구조적 제약을 끝까지 정확하고 일관되게 만족하는가?

This QA evaluates correctness that is sensitive to VIA architecture and execution ownership. It is intended to compare DP-00 alternatives whose internal component structures may differ materially.

## What this QA does not measure

The Architecture Qualification score intentionally does **not** measure:

- the underlying LLM's general natural-language understanding quality;
- ARGO's domain-reasoning quality;
- a Downstream Agent's document/analysis/content quality;
- live network/service correctness outside the controlled benchmark contract.

Those variables may affect Product E2E quality, but they are confounding variables when the purpose is to isolate the effect of SW architecture.

Actual-model/Agent behavior is still used to generate realistic frozen traces and to validate external fidelity, as defined in `docs/evaluation/architecture-experiment-methodology.md`.

## Primary Metric

### Architecture Episode Exact Conformance Rate — AECR

For each eligible evaluation episode, evaluate all declared architecture constraints.

```text
AECR = N(episodes satisfying all declared architecture constraints)
       ----------------------------------------------------------- × 100
       N(eligible evaluation episodes)
```

- Unit: **percent (%)**
- Direction: **higher is better**
- Architecture score: one 0–5 score derived from AECR using a frozen scoring version
- Primary Metric rule: **AECR alone determines the QA-02 0–5 score**

An episode contributes `1` only when every required/allowed/forbidden constraint is satisfied. Otherwise it contributes `0`.

## Episode scope

An evaluation episode represents **one user goal**, not necessarily one user turn.

The episode may include:

- the initial utterance/input;
- context grounding;
- clarification request;
- one or more clarification answers;
- task association;
- execution-path/owner selection;
- Agent delegation;
- result/progress correlation;
- final useful effect or delivery binding.

Example:

```text
User: "그 파일 김대리한테 보내줘."
 -> ambiguity detected
 -> clarification requested

User: "아까 열었던 PDF, 개발팀 김대리요."
 -> file/contact resolved
 -> correct execution owner chosen
 -> result bound to the same logical goal
```

The complete interaction is one correctness episode.

## Why final task success is not the Primary Metric

A final external task can fail because of:

- model semantic quality;
- Agent domain reasoning;
- external service behavior;
- live network conditions;
- tool implementation bugs outside VIA's responsibility boundary.

Using final task success directly would make those dependencies dominate the architecture score.

QA-02 instead asks whether the architecture correctly performs the decisions and state coordination it owns, given controlled semantic/dependency inputs.

Product E2E success remains important and should be reported in integration/fidelity evaluation, but it is not the architecture-only Primary Metric here.

## Architecture Constraint Manifest

A scenario does not have to prescribe one exact internal path.

Each scenario has a versioned **Architecture Constraint Manifest** with three classes.

### Required

Conditions that must hold.

Example:

```text
source_referent = file_A
destination_referent = folder_C
task_relation = FOLLOW_UP
task_id = T1
result_binding = T1
```

### Allowed

A set of architecture outcomes that are all semantically correct.

Example:

```text
execution_owner ∈ {ARGO, FileAgent}
```

### Forbidden

Outcomes that must not occur.

Example:

```text
execution_owner != VIA_FAST
delegated_agent != MailAgent
```

The exact manifest schema is defined in:

`benchmark/schemas/scenario-constraint-schema.md`

## Why a single expected path is not used

A single `expected_path` can encode a preferred topology into the benchmark and make the correctness result circular.

Example:

```text
"볼륨 줄여줘."
```

Depending on the alternative, both of these may be correct:

- VIA Fast Path;
- ARGO execution.

If the benchmark declared `expected_path = VIA_FAST`, it would structurally favor alternatives C/D. If it declared `expected_path = ARGO`, it would favor B or an ARGO-routed variant.

Instead the scenario can state:

```text
Allowed execution_owner = {VIA_FAST, ARGO}
Forbidden delegated_agent = unrelated_specialized_agent
```

Then both alternatives can receive full QA-02 correctness credit, while their latency difference appears in QA-01 FTOL.

This separation prevents correctness scoring from secretly rewarding one topology's performance strategy.

## Canonical Architecture Decision Trace

A/B/C/D may not have the same internal components. For example, an ARGO-centric topology may not contain a VIA `AgentRouter` component at all.

Therefore the oracle must not require implementation-specific fields such as:

- `router.selected_agent`;
- `intent_refiner.output`;
- presence/absence of a particular internal service call.

Each alternative instead projects its observable architecture decisions into a **Canonical Architecture Decision Trace**.

Minimum normalized fields include:

- `referent_bindings`;
- `task_relation`;
- `task_id`;
- `execution_owner`;
- `delegated_agent`;
- `clarification_action`;
- `result_binding`;
- `observable_effect`;
- compound-decomposition structure when applicable.

The benchmark evaluates scenario constraints against this canonical trace.

This makes the comparison topology-neutral while retaining architecture-sensitive outcomes.

## Architecture Qualification pipeline

```text
Scenario Fixture
       |
       v
Frozen Semantic Replay
       |
       v
Architecture A / B / C / D
       |
       v
Deterministic Agent / Tool Stubs
       |
       v
Canonical Architecture Decision Trace
       |
       v
Constraint Evaluator
       |
       +--> per-constraint results
       +--> episode_exact_conform
```

The architecture alternative is the independent variable.

## Realistic replay includes errors

Deterministic replay does **not** mean replaying only perfect semantic answers.

The frozen corpus should include behavior classes observed from actual models/Agents, such as:

- correct candidate;
- ambiguous/low-confidence candidate;
- partial or wrong referent;
- wrong Fast Path recommendation;
- wrong Agent recommendation;
- malformed structured output;
- timeout/no response.

The same erroneous semantic input is replayed to every comparable alternative.

The experiment can therefore ask:

> **동일하게 잘못된 model output이 들어왔을 때 어떤 architecture가 그것을 더 잘 검증하고 복구하는가?**

This is a direct test of policy, validation, state ownership, fallback and clarification structure rather than model quality.

## Required scenario taxonomy

The architecture-qualification corpus must cover at least:

### C1 — Context / Referent Grounding

Correctly bind screen/pointer/selection/referent evidence to the user goal.

### C2 — Task Association / Follow-up

Distinguish new task vs continuation/follow-up and preserve the correct logical task/session relationship.

### C3 — Execution-path Eligibility

Accept a valid Fast/Agent path and reject an architecturally invalid or unsafe path.

### C4 — Agent Routing / Delegation

Choose or delegate to an allowed execution owner/Agent without requiring one specific topology when several are valid.

### C5 — Ambiguity / Clarification

Detect when execution would be under-specified and perform the required clarification behavior.

### C6 — Concurrent Task / Result Binding

Maintain progress/result/cancel ownership and bind interleaved outcomes to the correct task/user goal.

### C7 — Compound Request Decomposition

Preserve independent/sequential/data-dependent/conditional structure across multiple requested actions.

### C8 — Dependency-error / Invalid-model-output Handling

Validate, reject, clarify, fall back or fail safely when semantic/model/Agent output is malformed, wrong, missing or timed out.

## Corpus composition principle

Architecture Qualification prioritizes **architecture-sensitive structural coverage** over blindly reproducing production usage frequency.

A corpus dominated by simple requests can hide failures in task association, referent binding, clarification, concurrent-result ownership and error recovery.

Therefore the scoring corpus should be **coverage-balanced** across architecture-sensitive categories and versioned/frozen before final evaluation.

A usage-frequency-weighted result may be computed as a **Secondary sensitivity analysis**, but it does not replace the frozen coverage-balanced AECR score unless a future scoring version explicitly changes the methodology.

Because AECR depends on scenario mix, the following are part of the metric definition and must be frozen:

- taxonomy version;
- scenario corpus version;
- category composition;
- eligibility rules;
- constraint-manifest versions;
- scoring version.

## Secondary Metrics

The following are diagnostic and must remain recomputable from raw data:

- Referent Binding Accuracy;
- Task Association Accuracy;
- Execution-path Conformance;
- Agent Routing Conformance;
- Clarification Correctness;
- Result Binding Accuracy;
- Compound Decomposition Accuracy;
- False Fast-path Rate;
- Unnecessary Delegation Rate;
- Constraint-level Macro Average;
- Critical-slice Exact Conformance;
- category-specific AECR for C1~C8;
- usage-weighted AECR sensitivity result.

These metrics explain where an alternative fails and preserve candidates for future Primary-Metric revisions.

## Why AECR is intentionally strict

AECR is an all-constraints episode metric. One violation makes the episode non-conformant.

Advantages:

- prevents a high average from hiding one broken architecture responsibility;
- matches the user-goal episode, where correct routing but wrong result binding is still an incorrect orchestration outcome;
- discourages compensating one severe failure with many easy component-level successes.

Disadvantage:

- it may compress alternatives toward low scores when episodes contain many constraints;
- it can hide partial improvement unless diagnostic metrics are preserved.

For this reason, **per-constraint raw outcomes and Secondary Metrics are mandatory**. Only storing final AECR is prohibited.

If pilot evidence shows AECR has unusable discrimination or construct validity, a Primary-Metric change is allowed only by recording the rationale, creating a new scoring version and recomputing all alternatives equally from preserved raw observations.

## Scoring

QA-02 uses six score bands: **0, 1, 2, 3, 4, 5**.

Thresholds are currently TBD.

```text
AECR >= C5        -> 5
C4 <= AECR < C5   -> 4
C3 <= AECR < C4   -> 3
C2 <= AECR < C3   -> 2
C1 <= AECR < C2   -> 1
AECR < C1         -> 0
```

Here `C1`~`C5` are **score cutoffs**, not the scenario-taxonomy category labels C1~C8.

Required process:

```text
Pilot
 -> inspect AECR distribution / strictness / instrumentation
 -> threshold calibration
 -> scoring-v1 freeze
 -> final A/B/C/D evaluation
```

Thresholds must not be tuned after viewing the final comparative result.

## Raw-data requirements

Raw events must preserve at least:

- `constraint_manifest_version`;
- required/allowed/forbidden constraints;
- actual canonical decision fields;
- per-constraint conformance booleans;
- `episode_exact_conform`;
- constraint failure ids/reasons;
- scenario taxonomy/corpus versions;
- semantic replay trace/class;
- architecture alternative/version.

The detailed contract is defined in:

- `benchmark/schemas/run-event-schema.md`
- `benchmark/schemas/scenario-constraint-schema.md`

Final AECR is a **derived metric**, not raw evidence.

## Related Decision Points

Primary current relevance:

- DP-00 — VIA Primary Execution Boundary
- interaction context representation
- existing-task vs new-task association
- intent refinement
- execution-path/capability placement
- Agent routing/delegation
- task/concurrency/result ownership

Exact QA↔DP numbering will be updated coherently after QA-01~QA-04 formalization.
