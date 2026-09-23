# AA-009 — Specification Branch Readiness Review

## Status

**Architecture Analysis Record — Specification Closure Review — Pre-Implementation**

This record evaluates whether `arch/qa-rebaseline-dp00` contains a sufficiently coherent architecture/evaluation specification to close the specification phase and begin DP-00 Rust prototype implementation on a separate implementation branch after human review.

It is **not** a DP-00 benchmark result, not a winner decision, not an ADR, and not an approval to merge automatically.

`docs/requirements/requirements-v1.1.md` remains unchanged as the Approved Baseline.

---

# 1. Review scope

The readiness review covers these specification layers:

```text
DP-00 architecture question
    ↓
Top QA-01~04
    ↓
Executable Base A/B/C/D topology
    ↓
Base Architecture vs Tactic separation
    ↓
Canonical Benchmark Contract
    ↓
Experimental Boundary / Oracle isolation
    ↓
Prototype / Benchmark Harness Specification
    ↓
Implementation-language/runtime boundary
```

The review asks whether an implementation team can proceed without reopening an unresolved **architecture-level** question merely to construct the prototype/harness.

---

# 2. DP-00 definition completeness

**Verdict: PASS**

Evidence:

- `docs/architecture/decision-points/DP-00-primary-execution-boundary.md`
- `docs/architecture/analysis/AA-001-qa-rebaseline-and-primary-execution-boundary.md`

The central question is stable:

> **VIA는 사용자 요청을 어디까지 직접 판단·실행하고, 어디부터 Downstream Agent에 위임할 것인가?**

A/B/C/D remain structurally distinct, and B remains the v1.1 responsibility-boundary challenging alternative.

No DP-00 winner is assumed.

---

# 3. Top QA definition completeness

**Verdict: PASS**

Authoritative scored drivers:

| QA | Primary Metric |
| --- | --- |
| QA-01 | Fast-task Outcome Latency p95 (FTOL p95) |
| QA-02 | Architecture Episode Exact Conformance Rate (AECR) |
| QA-03 | Change Containment Rate (CCR) |
| QA-04 | Average Model Calls to Commit Execution Route |

Evidence:

- `docs/evaluation/quality-attributes/QA-01-fast-task-e2e-responsiveness.md`
- `docs/evaluation/quality-attributes/QA-02-via-interaction-orchestration-correctness.md`
- `docs/evaluation/quality-attributes/QA-03-change-flexibility.md`
- `docs/evaluation/quality-attributes/QA-04-model-call-overhead.md`
- `docs/architecture/analysis/AA-005-top-qa-cross-review.md`

Threshold numbers remain intentionally TBD for Pilot/calibration; that does not block prototype construction.

---

# 4. Executable architecture completeness

**Verdict: PASS**

Evidence:

- `docs/architecture/decision-points/DP-00-executable-architecture-spec.md`
- `docs/architecture/analysis/AA-006-dp00-executable-walkthrough.md`

The Base alternatives define:

- responsibility placement;
- state ownership;
- decision mechanism rules;
- execution route semantics;
- B ARGO authoritative execution-state boundary;
- C = A + bounded Fast Path;
- D = Intent Refiner + Execution Path Selector without a second top-level Agent Router;
- clear follow-up route reuse;
- S1~S5 20-path consistency walkthrough;
- expected pre-experiment logical ModelCall assertions.

No Base-topology choice required merely to start implementation remains unresolved.

---

# 5. Base Architecture vs Tactic completeness

**Verdict: PASS**

Evidence:

- `docs/evaluation/base-architecture-vs-tactic-evaluation.md`

The specification distinguishes:

```text
Architecture
= who owns the responsibility

Tactic
= optional mechanism to optimize that responsibility
```

Optional fusion/classifier/speculative/cache/prompt/parallelization tactics are excluded from the Base comparison.

This prevents implementation maturity from silently becoming part of the architecture alternative.

---

# 6. Benchmark contract completeness

**Verdict: PASS WITH PILOT-TIME TBDs**

Evidence:

- `docs/evaluation/dp00-experimental-boundary.md`
- `benchmark/schemas/runtime-scenario-schema.md`
- `benchmark/schemas/semantic-behavior-plan-schema.md`
- `benchmark/schemas/canonical-event-schema.md`
- `benchmark/schemas/runtime-run-provenance-schema.md`
- `benchmark/contracts/model-replay-request-contract.md`
- `benchmark/contracts/runtime-fixture-contracts.md`
- `benchmark/contracts/observation-port-contract.md`
- `benchmark/catalog/runtime-scenario-catalog.md`
- `docs/architecture/analysis/AA-007-benchmark-contract-neutrality-review.md`

The contract is sufficient to implement:

- AUT-visible Stimulus vs evaluator-only Oracle separation;
- responsibility-keyed semantic replay;
- separate/fused topology fault equivalence;
- attempt/retry behavior;
- canonical architecture-neutral events;
- Outcome Probe authority;
- architecture-neutral ExecutionRoute;
- common post-route Agent/Tool fixtures;
- B.ARGOPrimary pre-route responsibility preservation;
- raw provenance/versioning.

Remaining corpus size, weighting, QA-04 aggregation and latency numbers are Pilot/calibration choices rather than architecture blockers.

---

# 7. Prototype / Harness specification completeness

**Verdict: PASS**

Evidence:

- `docs/evaluation/prototype-benchmark-harness-spec.md`
- `docs/architecture/analysis/AA-008-qualification-runtime-language-and-harness-rationale.md`

The implementation boundary is now explicit:

```text
Architecture Qualification Runtime = Rust
Offline Analysis = Python
```

The spec defines:

- logical Rust workspace/crate boundaries;
- Evaluation Support vs Architecture Under Test source roles;
- narrow Product-boundary AUT interface;
- neutral Model/Capability/Policy/Executor/Tool/Observation/Clock ports;
- `bench-oracle` dependency exclusion from alternative crates;
- State Seeder lifecycle boundary;
- AUT-visible ModelRequest vs hidden ReplayContext;
- AUT-visible ObservationPort vs benchmark-owned provenance enrichment;
- monotonic QA-01 timing;
- lightweight in-memory telemetry inside the timed window;
- JSON/JSONL v0 raw-evidence boundary;
- S1~S5 smoke gate;
- negative anti-gaming contract tests;
- Prototype Conformance Review before Pilot.

---

# 8. Runtime substrate readiness

**Verdict: PASS WITH IMPLEMENTATION FREEZE REQUIRED**

The inspected Rust reference provides a reasonable starting convention:

```text
Rust edition 2024
rust-version 1.94
Tokio 1.x
```

The exact toolchain patch/channel, resolved Tokio version, worker policy, `Cargo.lock`, target and optimized qualification build flags are intentionally frozen in the implementation branch before Pilot.

These are controlled evaluation-infrastructure choices shared across A/B/C/D, not DP-00 alternatives.

Timed QA-01 runs must use an optimized release-class profile. Debug runs are Smoke/Logical only.

---

# 9. Architecture-level blocker review

| Potential blocker | Status |
| --- | --- |
| DP-00 question ambiguous | **No** |
| A/B/C/D responsibility placement incomplete | **No** |
| B weakened to default Agent routing | **No** |
| D requires undefined extra Router/control-plane abstraction | **No** |
| Base vs Tactic boundary unresolved | **No** |
| Benchmark harness owns semantic decision logic | **No — prohibited by contract** |
| Oracle visible to AUT | **No — prohibited structurally/API-level** |
| Separate/fused semantic replay fairness unresolved | **No** |
| B ARGO responsibility swallowed by common stub | **No — prohibited** |
| Canonical observation depends on internal component names | **No** |
| Runtime language unresolved | **No** |
| QA-01 timing path includes Python by design | **No** |
| QA-03 cannot distinguish AUT vs benchmark code | **No — source-role mapping defined** |

No open architecture-level blocker was identified that requires user decision before implementation.

---

# 10. Intentional non-blocking TBDs

The following remain intentionally unresolved and do not prevent prototype/harness construction:

- final R1~R10 scenario count;
- scoring corpus weighting;
- QA-04 overall mean vs class macro-average;
- terminal no-route-commit aggregation rule;
- final dependency-latency milliseconds;
- QA-01~04 scoring thresholds;
- QA-02 correctness-gate value;
- Mandatory Qualification Suite final executable scenarios;
- actual-model behavior corpus;
- exact performance build flags;
- exact serialization structs/file writer details;
- tactic implementation;
- DP-00 winner.

The relevant ones follow:

```text
Prototype
  -> Smoke
  -> Conformance Review
  -> Pilot
  -> Calibration
  -> Freeze
  -> Final Evaluation
```

---

# 11. Merge / implementation recommendation

This analysis does **not** merge the branch.

Recommended workflow after user consistency review:

```text
arch/qa-rebaseline-dp00
  specification branch
        ↓ human review
merge to main if accepted
        ↓
separate implementation branch
        ↓
Rust Prototype / Harness implementation
```

Keeping implementation on a separate branch preserves a stable pre-experiment specification checkpoint and makes later deviations auditable.

---

# 12. Final verdict

```text
DP-00 Definition Complete                = PASS
Top QA Definition Complete               = PASS
Executable Architecture Spec Complete    = PASS
Benchmark Contract Complete              = PASS WITH PILOT-TIME TBDs
Prototype/Harness Spec Complete          = PASS
Open Architecture-level Blocker          = NONE IDENTIFIED

Specification Branch Readiness           = PASS
```

This `PASS` means **ready for human review and subsequent implementation**, not benchmark success and not architecture selection.