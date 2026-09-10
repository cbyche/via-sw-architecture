# VIA Architecture Benchmark

## Status

Central benchmark navigation for the DP-00 Base Architecture evaluation work.

This benchmark structure supports Architecture Qualification before any DP-00 winner is selected. `docs/requirements/requirements-v1.1.md` remains unchanged.

---

# 1. Benchmark families

```text
Architecture Evaluation
|
+-- Runtime Episode Benchmark
|    +-- QA-01 Fast-task End-to-End Responsiveness
|    +-- QA-02 VIA Interaction-Orchestration Correctness
|    +-- QA-04 Model Call Overhead
|
+-- Evolution Benchmark
|    +-- QA-03 Flexibility
|
+-- Mandatory Qualification Suite
     +-- Security / Privacy
     +-- Cancellation / Task-state integrity
     +-- Failure containment
     +-- Required recovery behavior
```

Do not force these families into one universal scenario schema.

Measurement unit differs:

```text
Runtime QA-01/02/04 -> one user-goal Runtime Episode
QA-03              -> one Architecture Evolution / Change Scenario
Mandatory Gate     -> one qualification scenario / PASS-FAIL
```

---

# 2. Runtime Episode Benchmark

Primary contracts:

- `schemas/runtime-scenario-schema.md`
- `schemas/semantic-behavior-plan-schema.md`
- `contracts/model-replay-request-contract.md`
- `contracts/runtime-fixture-contracts.md`
- `schemas/canonical-event-schema.md`
- `schemas/run-event-schema.md`
- `schemas/model-call-schema.md`
- `schemas/runtime-run-provenance-schema.md`
- `schemas/scenario-constraint-schema.md`
- initial catalog: `catalog/runtime-scenario-catalog.md`

Experimental-boundary definition:

- `../docs/evaluation/dp00-experimental-boundary.md`

Neutrality stress-test:

- `../docs/architecture/analysis/AA-007-benchmark-contract-neutrality-review.md`

---

# 3. Evolution Benchmark

QA-03 remains separate because its unit is a source/configuration evolution scenario rather than a runtime user-goal episode.

Contracts:

- `schemas/evolution-scenario-schema.md`
- `schemas/evolution-run-schema.md`
- `evolution/`

Primary derived metric:

- Change Containment Rate (CCR)

---

# 4. Mandatory Qualification Suite

Security/privacy/reliability obligations are non-compensable gates rather than Top-QA trade-off scores where appropriate.

Current gate categories:

- Security;
- Privacy / Context-sharing policy;
- Trusted boundary requirements;
- Required cancellation semantics;
- Required task-state integrity;
- Failure containment;
- Mandatory recovery behavior.

Executable gate scenario schemas and thresholds are **TBD** and are not invented in the Runtime Scenario schema.

---

# 5. Experimental-boundary rule

Reviewer-facing summary:

> **동일한 사용자 상황, 동일한 semantic model behavior, 동일한 Agent/Tool behavior를 A/B/C/D에 제공하고 responsibility placement와 execution topology를 독립변수로 비교한다.**

Shared benchmark infrastructure may control evidence, fixtures and dependency behavior. It must not perform AUT-owned referent/task/intent/route/Agent decisions.

---

# 6. Stimulus vs Oracle

Runtime scenarios logically contain two channels:

```text
AUT-visible Stimulus
  user input
  raw context evidence
  logical initial state
  capability/health/policy facts
  dependency responses

Evaluator-only Oracle
  ground-truth referent/task association
  Required/Allowed/Forbidden constraints
  expected result binding
  success predicate
  scoring eligibility
```

The benchmark knows the oracle; the AUT must not receive it.

---

# 7. Semantic replay

Replay is keyed by semantic responsibility/operation rather than global model-call ordinal.

This allows the same semantic condition to be applied to:

- A/C/D architectures with separate model generations;
- B ARGO-centric architecture where one generation can own multiple semantic responsibilities.

One replay-backed logical Generative AI generation remains one ModelCall for QA-04.

---

# 8. Canonical observations

Benchmark scoring consumes architecture-neutral observations such as:

```text
interaction.acoustic_eos
interaction.processing_started
model.generation.*
clarification.*
task.created / task.reused
execution.route_committed
execution.dispatched / accepted / started
result.bound
useful_outcome.observed
cancel.*
episode.completed / failed
```

Internal component debug events may exist but are not the common benchmark oracle.

---

# 9. Runtime taxonomy

```text
S1~S5 = executable architecture smoke / consistency walkthrough
R1~R10 = runtime scoring taxonomy / initial catalog
```

R1~R10:

- R1 Local / Bounded
- R2 General Agent
- R3 Specialized Agent
- R4 Context / Referent
- R5 Existing-task Follow-up
- R6 Ambiguity / Clarification
- R7 Execution-path Trap
- R8 Concurrent Task / Result
- R9 Compound Request
- R10 Dependency / Model Error

The current catalog is an initial skeleton, not a final frozen scoring corpus.

---

# 10. Raw evidence policy

```text
results/raw/      immutable evidence
results/derived/  recomputable metrics / aggregation
results/reports/  human-readable report material
```

Scenario, behavior plan, fixture/profile, schema, architecture and scoring versions are preserved in raw run provenance.

Final corpus weighting, QA-04 aggregation and no-route-commit treatment are frozen only after Pilot/calibration and before final A/B/C/D evaluation.

The Rust Pilot runner writes one create-new directory per measured episode under
`results/raw/pilot-v0/<run-id>/`. Warm-up episodes exercise the same runtime path
but are excluded from the measured population and are not persisted by the v0
runner. JSON serialization, file I/O, Python, reporting, and scoring remain
outside the timed interval.
