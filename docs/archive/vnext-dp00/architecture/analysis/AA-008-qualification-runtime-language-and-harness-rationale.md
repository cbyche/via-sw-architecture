# AA-008 — Qualification Runtime Language and Harness Rationale

## Status

**Architecture Analysis Record — Prototype/Harness Specification Checkpoint — Pre-Implementation / Pre-Pilot**

This record explains why the DP-00 Architecture Qualification runtime is implemented in **Rust**, while offline metric derivation, statistics, calibration and visualization are implemented in **Python**.

It is not a benchmark result, not a claim about absolute production latency, not an ADR selecting the final DP-00 architecture, and not a change to `docs/requirements/requirements-v1.1.md`.

Authoritative context:

- `docs/architecture/decision-points/DP-00-executable-architecture-spec.md`
- `docs/evaluation/dp00-experimental-boundary.md`
- `docs/evaluation/evaluation-strategy.md`
- `docs/evaluation/evaluation-principles.md`
- `docs/evaluation/architecture-experiment-methodology.md`
- `benchmark/schemas/runtime-scenario-schema.md`
- `benchmark/schemas/canonical-event-schema.md`
- `benchmark/schemas/model-call-schema.md`
- `benchmark/schemas/run-event-schema.md`

---

# 1. Decision

The implementation-language boundary for DP-00 Architecture Qualification is:

```text
Architecture Qualification Runtime
= Rust

Offline Analysis / Statistics / Visualization
= Python
```

Responsibility split:

```text
Rust
├─ A/B/C/D Architecture Under Test
├─ Benchmark Runner
├─ Controlled Interaction Front-end
├─ Semantic Replay Runtime
├─ Agent / Tool Fixtures
├─ Outcome Probe
├─ Canonical Event Collection
├─ ModelCall Recording
├─ Monotonic Timing
└─ Immutable Raw Evidence Recording

Python
├─ Raw-data / provenance validation
├─ FTOL p50 / p95 / p99 derivation
├─ AECR derivation and diagnostics
├─ QA-04 aggregation and breakdowns
├─ QA-03 evolution-result analysis
├─ statistics / sensitivity analysis
├─ Pilot distribution inspection
├─ scoring / gate calibration support
└─ plots / final-report tables
```

Python does **not** participate in the Architecture-under-Test decision path or QA-01 timed path.

---

# 2. Why the runtime language matters

The Top QA set contains different measurement types.

QA-02 and QA-04 are mostly logical/structural measurements, but QA-01 is explicitly based on **elapsed time**:

```text
FTOL_i
= useful_outcome_timestamp_i
  - ground_truth_acoustic_end_of_speech_timestamp_i
```

QA-03 also evaluates actual source-code change propagation across the architecture implementation.

Therefore the implementation substrate can become a confounder if A/B/C/D are realized on a runtime that differs materially from the intended production implementation language and scheduling model.

The concern is not that Python is generally “slow.” The concern is **differential distortion**:

- an alternative with more synchronous stages may pay more interpreter overhead;
- an alternative with more small async transitions may pay different scheduling overhead;
- object allocation/dynamic dispatch characteristics can affect topology shapes differently;
- cross-language boundaries introduced only for the experiment can add artificial serialization and IPC latency;
- QA-03 change propagation measured on disposable Python architecture code may not represent the maintainability properties of the Rust architecture that would actually be evolved.

A common Rust runtime reduces, but does not eliminate, these threats.

---

# 3. Candidates considered

## Candidate A — Python-only

```text
A/B/C/D + benchmark runtime + analysis
all implemented in Python
```

### Advantages

- fastest prototype iteration;
- excellent data-processing ecosystem;
- easy fixture and scenario scripting;
- low implementation overhead.

### Problems

- QA-01 elapsed-time comparison would include Python interpreter/runtime characteristics that are not representative of the intended Rust implementation substrate;
- stage-count/control-flow differences could interact with interpreter and async-runtime overhead asymmetrically;
- QA-03 would measure change containment in Python source rather than the architecture implementation language expected for production;
- a later Rust rewrite could change both performance and module/coupling structure after the architecture decision had already been made.

### Verdict

**Rejected for Architecture Qualification runtime.**

Python remains appropriate for offline analysis.

---

## Candidate B — Python prototype + QA-01 Rust reimplementation

```text
QA-02 / QA-04 topology prototype = Python
QA-01 timing prototype            = separate Rust implementation
analysis                           = Python
```

### Advantages

- reduces initial Rust implementation effort;
- attempts to obtain a more representative latency path for QA-01.

### Problems

- QA-01 would be measured on a different implementation from QA-02 and QA-04;
- architecture fidelity could drift between the Python logical prototype and Rust timing prototype;
- differences in responsibility placement, state ownership, retry behavior, and route-commit semantics would have to be synchronized manually;
- QA-03 would have an ambiguous source-of-truth implementation;
- defects could be incorrectly attributed to architecture when they are actually differences between two prototype implementations.

### Verdict

**Rejected.** One executable architecture implementation should generate the runtime evidence for QA-01/02/04.

---

## Candidate C — Rust qualification runtime + Python offline analysis

```text
Runtime architecture + harness = Rust
Raw evidence                   = versioned immutable files
Offline analysis               = Python
```

### Advantages

- A/B/C/D share one compiled runtime substrate;
- the same implementation produces QA-01, QA-02 and QA-04 runtime evidence;
- QA-03 can evolve the same Rust architecture source that was evaluated at runtime;
- no Python interpreter or cross-language RPC is required inside the timed path;
- Rust provides a precise monotonic-timing implementation and inexpensive in-memory event capture;
- Python remains available where it has the highest leverage: statistics, corpus inspection, derived metrics, sensitivity analysis and visualization;
- raw-data separation keeps scoring logic independent from runtime implementation.

### Limitations

- more implementation effort than Python-only;
- Rust prototype quality can still differ from production-quality code;
- deterministic replay and fixtures still simplify real model/Agent behavior;
- chosen async runtime/build flags can still affect timing and therefore must be frozen as controlled variables.

### Verdict

**Selected.**

---

## Candidate D — Rust-only

```text
Runtime + all analysis/statistics/plots = Rust
```

### Advantages

- one language/toolchain;
- no analysis-language handoff.

### Problems

- little architecture-validity benefit from implementing percentile calculation, exploratory statistics, calibration notebooks/scripts and plots in Rust;
- slower iteration for sensitivity and report analysis;
- increases evaluation-support code without improving the AUT fidelity;
- creates pressure to couple runtime and analysis logic rather than preserve immutable raw evidence as the boundary.

### Verdict

**Rejected.** Rust is required where runtime behavior is measured; Python is more appropriate after the raw-evidence boundary.

---

# 4. Correct interpretation of the decision

The decision supports this claim:

> **동일 Rust runtime 및 controlled dependency 조건에서 DP-00 execution topology가 FTOL과 runtime architecture evidence에 미치는 상대적 차이를 Architecture Qualification으로 측정한다.**

It does **not** support this claim:

> `Rust prototype FTOL = production VIA absolute latency`

The qualification runtime still uses:

- controlled interaction front-end;
- deterministic Semantic Replay;
- deterministic Agent/Tool fixtures;
- synthetic/calibrated dependency latency;
- prototype-level implementations;
- benchmark instrumentation;
- controlled machine conditions.

Therefore QA-01 is interpreted as a **controlled relative architecture latency comparison**.

Actual production-like Voice/Model/ARGO/network behavior belongs to the separate Actual-model / Real-stack Validation track.

---

# 5. Timed-path language boundary

The QA-01 timed path must not contain Python or cross-language analysis IPC.

Required runtime shape:

```text
Rust Qualification Process

Scenario Driver
    ↓
Controlled Interaction Front-end
    ↓
AUT A / B / C / D
    ↕
Model Replay Adapter
Agent Fixture
Tool Fixture
Outcome Probe
Observation Adapter
Monotonic Clock
    ↓
In-memory raw observations

--- timed episode ends ---

Raw serialization / file flush
    ↓
Python offline analysis
```

Prohibited inside the QA-01 timed interval:

- Python process invocation;
- Python callback/RPC;
- cross-language serialization for architecture decisions;
- report generation;
- JSON/JSONL file output;
- synchronous console/file logging used as the event transport.

Process startup, build time, scenario-file loading and analysis startup are outside FTOL.

---

# 6. Common Rust runtime is a controlled variable

A/B/C/D must not use different async runtimes, compiler modes or executor frameworks merely because one is convenient for a particular topology.

The qualification run records and freezes:

```text
rust_toolchain_version
rustc_version_verbose / compiler_version
rust_edition
target_triple
build_profile
build_flags_profile_version
async_runtime
async_runtime_version
Cargo.lock identity / dependency_lock_hash
```

The same frozen runtime/toolchain profile is used for comparable A/B/C/D runs.

## Reference convention inspected

The internal Rust reference workspace currently records:

```text
edition = 2024
rust-version = 1.94
async runtime = Tokio 1.x
```

The qualification prototype should align with this convention where practical because it reduces unnecessary substrate divergence.

However, the reference workspace's current `release` profile is optimized primarily for binary size (`opt-level = "z"`). That profile is evidence about the reference implementation, not a mandatory qualification timing profile.

For QA-01 Timed Qualification:

> **use an optimized release-class build profile intended for stable performance comparison, and freeze the exact profile before Pilot/final qualification.**

Do not use debug builds for scored timing.

The exact optimization flags remain implementation/Pilot configuration, not a DP-00 architecture variable.

---

# 7. Async runtime choice

The async runtime is **evaluation infrastructure**, not a DP-00 alternative.

Given the reference convention, Tokio 1.x is the preferred v0 implementation direction, subject to implementation verification.

Rules:

- one runtime family/version for all A/B/C/D;
- no alternative-specific scheduler tuning in Base Architecture evaluation;
- runtime worker-count/thread-affinity policy is versioned/frozen where it could affect timing;
- alternative-specific optimization of scheduling belongs to a later tactic/sensitivity experiment if ever needed.

The exact resolved crate versions are frozen by `Cargo.lock` in the implementation branch.

---

# 8. Why release-class build is required

Debug-mode overhead is not representative and can amplify differences in abstraction, generics, bounds checks, logging and allocation.

Therefore:

```text
Smoke / Logical Mode
  may use development builds for developer iteration
  does not produce QA-01 score

Timed Qualification Mode
  uses frozen optimized release-class build
  uses real monotonic clock
  uses controlled calibrated dependency latency
```

A release build is necessary for timing validity, but it still does not turn the prototype into a production-latency benchmark.

---

# 9. Instrumentation threat

Instrumentation can distort QA-01 even in Rust.

Potential confounders:

- JSON serialization in the critical path;
- file flushing;
- console logging;
- lock contention in a shared tracing sink;
- timestamp acquisition overhead;
- allocation caused by high-volume telemetry;
- different instrumentation paths for different alternatives.

Required mitigation:

```text
inside timed window:
  capture timestamp
  append compact typed event to in-memory buffer

outside timed window:
  normalize/enrich as needed
  serialize JSON/JSONL
  flush raw evidence
```

Pilot must perform an instrumentation-overhead check, such as instrumentation-on/off or minimal-event baseline comparison, and verify that collection overhead does not dominate or reverse architecture ranking.

The result of that check is validity evidence, not a new QA score.

---

# 10. QA-03 rationale

QA-03 measures whether evolution stays inside the Expected Change Area.

Using Rust for the AUT means the evolution benchmark modifies the same language/module boundaries that produced runtime evidence.

This improves internal consistency between:

```text
Runtime architecture structure
    ↕
Evolution/change-containment structure
```

Evaluation-support code remains excluded from AUT change-propagation scoring according to the frozen source-role mapping.

This does not imply that Rust itself causes better or worse CCR; language is controlled across A/B/C/D.

---

# 11. Python analysis boundary

Python starts **after immutable runtime evidence exists**.

The project uses its normal Python virtual-environment convention (`.venv/`). The exact Python interpreter/package environment used for a derived result must be recorded/versioned with the analysis output.

Python responsibilities include:

- validate schema/provenance consistency;
- load canonical/raw ModelCall streams;
- calculate QA-01 FTOL distributions;
- calculate QA-02 AECR and per-constraint diagnostics;
- calculate QA-04 overall mean and candidate class macro-average;
- analyze QA-03 evolution records;
- perform Pilot/calibration and sensitivity calculations;
- generate report tables/plots.

Python must not mutate `results/raw/`.

A change in Python analysis/scoring code creates a new derived/scoring version; it does not rewrite the original Rust runtime observations.

---

# 12. Serialization rationale

For Prototype/Harness v0, the preferred evaluation-infrastructure serialization family is:

```text
Runtime Scenario Manifest / Semantic Behavior Plan
  -> JSON

Run provenance
  -> JSON

Canonical event stream
  -> JSONL

Model-call stream
  -> JSONL
```

Rationale:

- straightforward `serde` support in Rust;
- direct loading in Python;
- JSONL suits appendable/event-oriented raw evidence;
- avoids introducing multiple serialization families before evidence shows a need.

This is **evaluation infrastructure**, not a product architecture decision.

Every artifact carries schema/version identity.

The exact file schema implementation is finalized in the prototype branch and contract-tested before Pilot.

---

# 13. Decision consequences

The Prototype/Harness Specification must now ensure:

1. the Rust AUT public boundary does not expose architecture-specific decomposition;
2. evaluator/oracle crates are not dependencies of alternative crates;
3. benchmark ids and semantic replay operation ids are hidden from AUT decision APIs;
4. ModelPort knows architectural semantic responsibilities but not benchmark answers;
5. ObservationPort accepts semantic observations while benchmark provenance is enriched outside the AUT;
6. QA-01 timing and Outcome Probe use one monotonic clock domain;
7. event capture in the timed path is lightweight/in-memory;
8. raw serialization happens after the timed window;
9. S1~S5 smoke and negative anti-gaming contract tests run before Pilot;
10. Prototype Conformance Review is a gate between smoke completion and Pilot.

---

# 14. Threats that remain

Choosing Rust does not eliminate:

- replay fidelity threat;
- deterministic Agent/Tool simplification;
- synthetic latency sensitivity;
- machine/hardware dependence;
- async scheduler variance;
- prototype maturity/fidelity differences;
- instrumentation overhead;
- corpus representativeness;
- model/prompt/cache profile dependence.

These remain explicit Threats to Validity and require controlled configuration, Pilot inspection, sensitivity analysis and separate Real-stack Validation.

---

# 15. Final verdict

| Question | Verdict |
| --- | --- |
| One runtime implementation should serve QA-01/02/04 | **YES** |
| Runtime architecture should be Rust | **YES** |
| Python should enter the QA-01 timed path | **NO** |
| Python should be used for offline metric/statistical analysis | **YES** |
| Rust runtime choice proves absolute production latency | **NO** |
| Common Rust runtime/toolchain must be frozen across A/B/C/D | **YES** |
| Reference Rust 1.94 / edition 2024 / Tokio 1 convention is a suitable starting point | **YES, subject to implementation freeze** |
| Reference size-optimized release flags should be blindly reused for timed scoring | **NO** |

**Decision: Candidate C — Rust qualification runtime + Python offline analysis.**

This decision closes the implementation-language boundary needed to write the Prototype / Benchmark Harness Specification.