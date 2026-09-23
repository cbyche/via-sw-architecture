# Evaluation Strategy — vNext Central Architecture Evaluation

## Status

Central vNext evaluation strategy for `DP-00 — VIA Primary Execution Boundary` and the Top Architectural Driver QA set.

`docs/requirements/requirements-v1.1.md` remains the Approved Baseline. This strategy defines how architecture alternatives are evaluated; it does not select a winner or change approved requirements.

Detailed rules are authoritative in:

- `docs/evaluation/evaluation-principles.md`
- `docs/evaluation/architecture-experiment-methodology.md`
- QA-specific documents under `docs/evaluation/quality-attributes/`
- benchmark semantic schemas under `benchmark/schemas/`

## 1. Evaluation storyline

```text
Architecture Concern
    ↓
Decision Point / Alternatives
    ↓
QA / one Primary Metric
    ↓
Architecture Qualification
    ↓
Pilot
    ↓
Calibration
    ↓
Scoring / Gate Rule Freeze
    ↓
Final A/B/C/D Evaluation
    ↓
Sensitivity / Threats-to-validity Analysis
    ↓
Trade-off Analysis
    ↓
ADR
```

The purpose is to make the final architecture decision auditable from predeclared hypotheses, frozen evaluation rules and immutable raw evidence.

## 2. Evaluation boundary and independent variable

DP-00 compares the **Integrated Product** behavior of structurally different execution topologies.

All A/B/C/D alternatives must satisfy the same user-visible scenarios. The intended independent variable is:

```text
Independent Variable
  = SW Architecture Alternative
  = placement / ownership of reasoning, execution, routing and orchestration capability
```

Controlled or frozen variables include, as applicable:

```text
scenario semantics / corpus
semantic model behavior replay
Agent / tool scripted behavior
model profile
prompt profile
cache policy
machine / power / background state
controlled external dependency latency
```

The experiment is valid only when observed differences can reasonably be attributed to architecture rather than to a different model, Agent, prompt, cache state, network condition or benchmark fixture.

## 3. Three quality views

Do not conflate these evaluation questions:

1. **Architecture Qualification** — what difference does the SW topology itself create under controlled conditions?
2. **Actual-model / Real-stack Validation** — does the controlled setup remain representative of real models, Agents, networks and tools?
3. **Product E2E quality** — what does the integrated user experience look like in realistic deployment conditions?

Only Architecture Qualification feeds the DP-00 architecture score unless a future frozen scoring version explicitly states otherwise.

## 4. Top Architectural Driver QA set

| QA | Reviewer-facing question | Primary Metric |
| --- | --- | --- |
| **QA-01 Fast-task End-to-End Responsiveness** | **빠른가?** | **Fast-task Outcome Latency p95 (FTOL p95)** |
| **QA-02 VIA Interaction-Orchestration Correctness** | **정확한가?** | **Architecture Episode Exact Conformance Rate (AECR)** |
| **QA-03 변경 대응 용이성 (Flexibility)** | **변경이 잘 격리되는가?** | **Change Containment Rate (CCR)** |
| **QA-04 모델 호출 오버헤드** | **실행 경로를 정하기 위해 AI 판단을 얼마나 요구하는가?** | **Average Model Calls to Commit Execution Route** |

Rule:

```text
One QA = One Primary Metric for the 0–5 score
```

Secondary Metrics are diagnostic, backup, severity or sensitivity evidence. They do not silently become weighted terms inside the Primary score.

The Top QA set was cross-reviewed for independence, DP-00 sensitivity, neutrality and testability in:

- `docs/architecture/analysis/AA-005-top-qa-cross-review.md`

Its sensitivity matrix and A/B/C/D tendency statements are **pre-experiment hypotheses**, not results.

## 5. Architecture Qualification

Architecture Qualification is the controlled scoring track.

```text
Frozen Scenario / Evolution Requirement
        +
Frozen Semantic Replay
        +
Deterministic Agent / Tool Behavior
        +
Frozen Model / Prompt / Cache Profiles
        +
Controlled Machine / Dependency Conditions
        ↓
DP-00 Alternative A / B / C / D
        ↓
Immutable Raw Evidence
        ↓
QA Primary + Secondary Metrics
```

### Deterministic semantic replay

Architecture Qualification uses deterministic semantic replay where model semantics would otherwise introduce stochastic variance.

Replay occurs at the dependency semantic seam without bypassing the architecture mechanism being evaluated.

The same comparable scenario/trace is supplied to each alternative.

### Realistic replay includes errors

The frozen corpus is not restricted to perfect model outputs. Real-model repeated runs are used to observe realistic behavior classes such as:

```text
correct
ambiguous / low confidence
partial / wrong candidate
wrong execution-path proposal
wrong Agent proposal
malformed structured output
timeout / no response
```

Those behavior classes are normalized and frozen so A/B/C/D can be compared under the **same correct and incorrect dependency conditions**.

This lets the experiment test whether architecture-level validation, fallback, clarification and state ownership respond differently to the same model error.

No claim is made that this is a standardized “LLM architecture replay” benchmark; it is a project-specific controlled-experiment method using record/replay and test-double principles.

### Deterministic Agent / tool behavior

During qualification, Downstream Agent and tool/service behavior is deterministic or scripted where necessary to prevent their own intelligence/latency from becoming the architecture variable.

Relevant behaviors can include:

- fixed result timing;
- progress sequence;
- permission request;
- cancel request/confirmation;
- timeout/failure;
- recovery/restart behavior;
- machine-observable tool outcome.

## 6. QA-specific measurement and anti-gaming controls

### QA-01 — FTOL p95

Primary interval:

```text
ground-truth acoustic EOS
    -> first useful observable outcome
```

Population: successful Fast-task episodes only.

A failed task is **not** assigned an arbitrary huge latency. Correctness remains QA-02.

Controlled external/downstream latency must satisfy both:

```text
Equality
  = comparable alternatives receive the same controlled dependency behavior

Non-dominance
  = fixture latency does not mechanically dominate FTOL p95 and hide architecture overhead
```

Exact deterministic latency values are **TBD** and calibrated in Pilot before the final benchmark version is frozen.

Real Agent/network/tool latency is reported in the separate validation/Product-E2E track.

### QA-02 — AECR

Unit of evaluation: one user-goal episode, including clarification/follow-up turns when required.

Correctness oracle:

```text
Required
Allowed
Forbidden
```

The Architecture Constraint Manifest is evaluated against a topology-neutral Canonical Architecture Decision Trace.

Constraint truth must come from:

- functional requirements;
- policy/security rules;
- capability contracts;
- scenario semantics;
- task/context truth.

It must **not** be derived from the topology under test. For example, the existence of a Fast Path in Alternative D is not evidence that `VIA_FAST` is the uniquely correct path.

The replay corpus includes wrong/ambiguous semantic outputs so architecture validation/recovery behavior remains observable.

### QA-03 — CCR

Flexibility is evaluated with controlled evolution scenarios rather than runtime timing events.

Required order:

```text
Common Evolution Requirement
    ↓
Common Expected Change Roles
    ↓
Alternative-specific role → component/file mapping
    ↓
freeze boundary + mapping
    ↓
Implement change
    ↓
Git diff + Acceptance + Regression evaluation
```

Expected Change Area is defined first at a common architecture-role level. It cannot be enlarged after seeing an alternative's diff.

A scenario counts as contained only when:

```text
Feature Acceptance PASS
AND Regression PASS
AND No unexpected propagation outside Expected Change Area
```

### QA-04 — Average Model Calls to Commit Execution Route

Measurement start:

```text
request_processing_start_ts
```

The start is intentionally not forced to Acoustic EOS because an architecture may legitimately perform partial/streaming semantic work before EOS.

Authoritative end boundary:

```text
Execution Route Commit
```

Execution Route Commit is the earliest point at which the final domain execution route is operationally committed and no further execution-owner/delegation decision is required before domain execution can continue.

Primary inclusion:

```text
call_class in {ORCHESTRATION, MIXED}
AND
(call occurs before route commit OR the call causes route commit)
```

Pure downstream DOMAIN reasoning after route commit is excluded.

This boundary prevents **hidden-routing gaming**. Moving a specialist-routing decision inside ARGO after an intermediate `execution_started` event does not make the delegation inference disappear from QA-04.

Logical model-call count does not claim physical compute equivalence. Model, prompt and cache profiles are frozen for qualification, while token/cache/latency/CPU/GPU/NPU/memory/energy/cost telemetry is retained for Secondary analysis.

## 7. Scored QAs vs eligibility and mandatory gates

### Score != Eligibility

A low-latency or low-model-call architecture cannot be selected if its correctness is below an acceptable minimum.

Recommended final-selection gate:

```text
if QA-02 AECR < minimum acceptable correctness threshold:
    alternative is ineligible for final DP-00 selection
```

The numeric threshold is **TBD**.

It follows the same calibration/freeze discipline as scoring thresholds and must be frozen before final results.

Correctness is not inserted as an arbitrary penalty inside QA-01 or QA-04.

### Mandatory Qualification Constraints / Gates

```text
Scored Architectural Drivers
  QA-01
  QA-02
  QA-03
  QA-04

Mandatory Qualification Gates
  Security
  Privacy / Context-sharing policy
  Trusted boundary requirements
  Required cancellation semantics
  Required task-state integrity
  Failure containment
  Mandatory recovery behavior
```

These concerns are not outside the Top 4 because they are unimportant. Where the requirement is non-compensable, a must-pass gate is more appropriate than a trade-off score.

The exact executable gate set and pass/fail thresholds are **TBD** and must be traced to approved/working requirements before final evaluation.

## 8. Pilot, calibration and freeze discipline

Required order:

```text
Pilot
    ↓
Inspect metric behavior / instrumentation / confounders
    ↓
Calibration
    ↓
Freeze scoring, gate, corpus, profiles and benchmark rules
    ↓
Final A/B/C/D Evaluation
```

During Pilot the project may adjust, with documented rationale:

- score thresholds;
- correctness-gate threshold;
- scenario taxonomy/composition;
- QA-01 dependency-latency profile;
- QA-03 evolution scenario set/role vocabulary;
- QA-04 workload taxonomy and aggregation rule;
- model/prompt/cache profiles;
- instrumentation/schema details.

Once final evaluation begins, these rules must not be changed merely because the resulting ranking is inconvenient.

If a Primary Metric, score rule or gate later changes:

1. record the reason;
2. increment the scoring/benchmark/gate version;
3. recompute all alternatives equally from immutable raw evidence where possible;
4. rerun only if the needed raw observations were not captured.

## 9. Raw → Derived → Decision traceability

```text
Raw Evidence
    ↓
Derived Metric
    ↓
QA Primary Metric
    ↓
0–5 Score / Qualification Gate
    ↓
DP-00 Trade-off Analysis
    ↓
Architecture Decision / ADR
```

### Runtime evidence

- `benchmark/schemas/run-event-schema.md`
- `benchmark/schemas/scenario-constraint-schema.md`
- `benchmark/schemas/model-call-schema.md`

Derived Primary Metrics:

- QA-01 FTOL p95
- QA-02 AECR
- QA-04 Average Model Calls to Commit Execution Route

### Evolution evidence

- `benchmark/schemas/evolution-scenario-schema.md`
- `benchmark/schemas/evolution-run-schema.md`

Derived Primary Metric:

- QA-03 CCR

### Storage policy

```text
results/raw/      immutable source evidence
results/derived/  recomputable metrics and score inputs
results/reports/  human-readable results / visualizations / decisions
```

Final QA score tables are not the source of truth.

## 10. Scenario population and sensitivity

Population composition can change p95, exact-conformance rate, containment rate and model-call averages.

Therefore applicable taxonomy, scenario counts, eligibility rules, manifest versions, workload mix and aggregation rules are versioned and frozen before final scoring.

Architecture Qualification favors **architecture-sensitive structural coverage** rather than blindly mirroring production usage frequency.

Production-frequency weighting may be calculated separately as a Secondary sensitivity analysis.

For QA-04, final use of overall episode mean vs class-level macro-average remains **TBD** until Pilot.

## 11. Actual-model / Real-stack Validation

The validation track asks whether the controlled architecture experiment remains externally credible.

It should report questions such as:

- Do real model/Agent runs still produce the behavior classes represented in frozen replay traces?
- How do real provider/network/tool latencies alter Product E2E experience?
- Are model/prompt/cache assumptions stable?
- Are local CPU/GPU/NPU and memory effects materially different from controlled profiles?
- Do real Agent progress/cancel/failure semantics match the stubs closely enough for the architecture conclusion to remain plausible?

These observations may motivate a new benchmark version, but they do not retroactively rewrite final scoring rules.

## 12. Threats to validity

Final evaluation must explicitly discuss at least:

- replay fidelity;
- deterministic stub vs real Agent behavior;
- benchmark scenario representativeness;
- workload weighting/population sensitivity;
- model-profile dependency;
- hardware dependency;
- prompt/cache optimization dependency;
- external dependency-latency dominance;
- prototype fidelity to production architecture;
- correctness-oracle neutrality;
- Expected Change Area gaming;
- hidden-routing measurement gaming;
- instrumentation effect;
- threshold hindsight / post-hoc tuning.

Controls reduce these threats; they do not prove that the threats disappear.

## 13. Relationship to Legacy v1.1 Detailed QAs

The v1.1 QA definitions and approved targets remain unchanged inside `requirements-v1.1.md`.

Central vNext navigation classifies them as retained supporting/diagnostic
concerns or retained mandatory obligations. The obligations remain approved;
only a future executable vNext gate encoding may be TBD. See:

- `docs/requirements/requirements-vNext.md`
- `docs/architecture/qa-dp-traceability.md`
- `docs/architecture/qa-legacy-migration.md`

When referring to an old ID in vNext evaluation documents, use `Legacy v1.1 QA-xx` where ambiguity is possible.

## 14. Current unresolved / TBD

Not fixed by this rebaseline:

- DP-00 winner;
- QA-01~04 0–5 thresholds;
- QA-02 minimum correctness gate;
- QA-01 deterministic dependency-latency values;
- final benchmark scenario corpora;
- QA-03 final evolution taxonomy/role vocabulary/scenario set;
- QA-04 final workload taxonomy/class proportions;
- QA-04 overall mean vs class macro-average;
- executable mandatory-gate definitions;
- A/B/C/D prototype implementation details.

All follow:

```text
Pilot -> Calibration -> Scoring/Gate/Benchmark Freeze -> Final Evaluation
```

## 15. Final report navigation

Report-ready reasoning and evidence locations are indexed in:

- `docs/architecture/analysis/final-report-evidence-index.md`

That index should be updated as executable specifications, raw results, derived score tables, sensitivity analyses and the final ADR are added.
