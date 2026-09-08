# VIA Software Architecture

Samsung PC용 **Voice Interaction Agent (VIA)** 의 SW Architecture 설계와 검증을 위한 engineering repository입니다.

이 repository의 목적은 단순 문서 보관이 아니라 다음 흐름을 한 곳에서 추적하는 것입니다.

```text
Requirements / Quality Attributes
        ↓
Architectural Decision Point
        ↓
Alternatives
        ↓
Prototype / Experiment
        ↓
Measured QA Trade-off
        ↓
Architecture Decision Record (ADR)
        ↓
Requirements / Architecture Update
```

## Current baseline

- Requirements: [`docs/requirements/requirements-v1.1.md`](docs/requirements/requirements-v1.1.md)
- Status: **v1.1 Approved**
- Working requirements: [`docs/requirements/requirements-vNext.md`](docs/requirements/requirements-vNext.md)

## Scope

VIA는 Voice-first PC interaction layer입니다.

VIA가 주로 소유하는 책임:

- Voice / Text interaction
- PC interaction context grounding
- Intent refinement
- Downstream Agent selection and delegation
- Conversation / Task lifecycle
- Progress / status / cancel / follow-up
- Context sharing / consent interaction
- Memory
- Model / Voice Runtime abstraction
- Observability / audit

Downstream Agent가 주로 소유하는 책임:

- Domain reasoning
- Planning
- Tool selection
- Tool execution
- External API / Computer-use execution
- Domain workflow completion

## Repository structure

```text
docs/
  requirements/             Approved and working requirements
  architecture/             System views and QA↔DP traceability
  architecture/decision-points/
                            Decision Point analyses
  adr/                      Accepted Architecture Decision Records
  experiments/              Experiment definitions
  evaluation/               QA definitions and evaluation methodology

benchmark/
  requests/                 Architecture request suite
  utterances/               Natural spoken evaluation set
  agent-stubs/              Deterministic downstream-agent stubs
  failure-injection/        Failure scenarios
  evolution/                Replaceability/extensibility exercises
  runners/                  Benchmark code

prototypes/                 DP alternative implementations
results/                    Experiment outputs
scripts/                    Utility scripts
```

## Working model

1. `requirements-v1.1.md` is immutable.
2. A Decision Point is analyzed in `docs/architecture/decision-points/`.
3. Alternatives are implemented under `prototypes/`.
4. The same benchmark suite is run against alternatives.
5. Results are stored under `results/`.
6. The accepted decision is recorded as an ADR.
7. Any requirement/QA changes go into `requirements-vNext.md`.

## Primary quality themes

Current architecture work focuses on:

- Functional correctness
  - Screen / pointer referent binding accuracy
  - Intent and task-association correctness
  - Downstream Agent routing correctness
- Performance efficiency
  - VIA software overhead
  - Barge-in latency
  - Concurrent task capacity / PC co-existence
- Reliability
  - Conversation / task recovery
  - Dependency failure isolation
- Maintainability / flexibility / interoperability
  - Voice runtime replacement
  - Model endpoint replacement
  - Downstream Agent integration
- Security / confidentiality
  - External context minimization
- Resource efficiency
  - Model / token cost efficiency

## Decision Point lifecycle

```text
Open
→ Alternatives Defined
→ Prototype Ready
→ Benchmark Complete
→ Decision Proposed
→ Accepted
→ ADR
```

## Commit convention

Suggested examples:

```text
dp01: define interaction-context alternatives
dp01: implement timeline prototype
bench: add multi-pointing scenarios
exp01: record pointer-grounding benchmark results
adr001: select interaction timeline
req: update QA-09 after DP-01
```

## Baseline policy

Approved requirements are never edited in place.

```text
v1.1 Approved
     ↓
vNext Working
     ↓
review
     ↓
v1.2 Approved
```
