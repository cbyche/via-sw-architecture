# AA-022 — DP-00 Realistic Execution Profile Definition and Prospective Freeze

## Disposition

**REALISTIC EXECUTION PROFILE FROZEN — READY FOR COMPARATIVE CAMPAIGN.**

This is a measurement-contract and experiment-design decision, not a DP-00
architecture selection. Session 7.0 remains **NOT READY FOR ARCHITECTURE
DECISION**, and no A/B/C/D winner, weighted score, QA weight, or synthetic QA-03
score is created here.

## Identity

| Item | Value |
| --- | --- |
| Session | 7.1 |
| Branch | `exp/dp00-runtime-pilot` |
| Starting/source SHA reviewed | `0da0ae058a137977ee4d60de92126655fa4e9590` |
| Architecture record | AA-022 |
| Profile contract | `dp00-realistic-execution-profiles-v1` |
| Profile implementation version | `dp00-realistic-sensitivity-v1` |
| Runner | `bench-runner` 0.1.0 plus Session 7.1 frozen-profile support |
| Analyzer | `dp00-analysis-v9`; v8 QA formulas unchanged, R1–R4 validation and dependency decomposition added |
| Provenance | `dp00-pilot-provenance-v4` unchanged |
| Event / model-call schemas | `canonical-event-v3` / `model-call-v1` unchanged |
| Corpus | `DP00-RUNTIME-PILOT-V0` v0.2, P01–P12 |
| Model / prompt / cache | `dp00-base@v0` / `pilot-v0-payload-v1` / disabled |
| Qualification runtime | Rust 1.94.0; qualification profile; Tokio 1.53.1 current-thread policy |

The Git commit containing this record is the implementation freeze identity and
is reported externally after commit; embedding a commit's own SHA in its content
would be self-referential. The next campaign must record that exact committed
source SHA in every run and manifest.

## Motivation

Session 7.0 produced 1,536 executions and 768 complete CAPTURE/MINIMAL pairs with
zero provenance, semantic, or pair mismatches. Its Profile Z latency was
model/Agent/tool = 0/0/0 microseconds. That evidence validly exposes software
topology and deterministic correctness, but it cannot show whether B's lower
model-call count or C/D local execution offsets framework overhead when
dependencies are non-zero. Profile Z therefore remains the immutable structural
reference and is not a production-representative end-to-end ranking.

## Strategy decision

A small sensitivity matrix is selected rather than one representative profile.
The repository has no empirical production latency dataset from which a single
representative point could be justified, while AA-021 identifies three distinct
placement questions: model-call reduction, delegation avoidance, and local Tool
execution. Four profiles are the smallest symmetric design that includes a
balanced case and isolates dominance of each existing dependency class. This is
not a parameter sweep.

Configuration alone was insufficient: the generic runner exposed three delay
fields, but only Z and provisional C had stable constructors; the paired
qualification runner was hard-coded to Z; and strict analysis rejected any
other profile ID. Session 7.1 therefore makes only localized profile selection,
strict-validation, decomposition, and test changes. The canonical event schema,
run-provenance shape, pair contract, and A/B/C/D sources do not change.

All values below are **synthetic sensitivity-analysis values**. They are simple
engineering assumptions, not measured or externally sourced production facts.

| ID | Regime | Model | Agent/delegation | Tool | Unit | Numeric provenance |
| --- | --- | ---: | ---: | ---: | --- | --- |
| R1 | Balanced | 50,000 | 50,000 | 50,000 | µs/event | sensitivity-analysis value |
| R2 | Model-dominant | 100,000 | 20,000 | 20,000 | µs/event | sensitivity-analysis value |
| R3 | Agent/delegation-dominant | 20,000 | 100,000 | 20,000 | µs/event | sensitivity-analysis value |
| R4 | Tool-dominant | 20,000 | 20,000 | 100,000 | µs/event | sensitivity-analysis value |

The 20/50/100 ms levels avoid pseudo-precision, make balanced and 5:1 dominant
regimes legible, and keep a full deterministic qualification campaign practical.
They support relative sensitivity and crossover analysis only. Absolute FTOL
must not be described as production-representative.

Each profile uses a constant distribution per event. There is no random sampling
and therefore no seed. Deterministic sample assignment is the identity mapping:
every occurrence of a semantic event class receives that class's configured
constant. Wall-clock scheduling noise can still affect observed elapsed time;
the configured dependency budget and semantic replay remain exactly
reproducible.

## Charging semantics and application points

### Model invocation

One charge applies to every call to the common `ModelPort::generate` boundary.
The runner records `ModelGenerationStarted`, sleeps for the configured constant,
performs deterministic replay, timestamps completion, and records
`ModelGenerationCompleted`. A failed generation is still a dependency attempt
and is charged. All pre-boundary and post-boundary calls are charged when they
actually occur; the current P01–P12 Base corpus happens to contain only
pre-route or pre-terminal-resolution logical calls. Multiple calls are additive.

### Agent/delegation

One charge applies to every `ExecutionPort::execute` invocation whose committed
route is not `LOCAL_DIRECT`. This represents one accepted interaction with the
selected ARGO or specialist execution boundary, not every internal downstream
ReAct turn. `ExecutionStarted` and the fixture start-acceptance observation occur
before the sleep; useful outcome/effect observation occurs after it. Multiple
delegated executions are additive.

### Tool

One charge applies to every `ExecutionPort::execute` invocation whose committed
route is `LOCAL_DIRECT`. `ExecutionStarted` and fixture start acceptance occur
before the sleep; side effect and Outcome Probe observation occur after it.
Multiple local Tool effects are additive. A local path is cheaper than a
delegated path only in a profile where its actual Tool charge is lower than the
avoided Agent charge; there is no alternative-specific exception.

For counts `(M, A, T)`, the exact configured dependency budget is:

```text
M × model_delay + A × agent_delay + T × tool_delay
```

The calculation saturates rather than overflowing. Delays are sequential in the
current synchronous Base runner.

### Failed, forbidden, and non-committed routes

A dependency is charged if and only if its common port is invoked. A forbidden
route that reaches execution incurs the corresponding semantic charge and
remains a QA-02/QA-04 violation. A failed or non-committed route that never calls
an execution port incurs no Agent/Tool charge. Delay configuration does not
create, suppress, validate, or reinterpret route events.

### QA-01 timestamp and decomposition contract

QA-01 remains Acoustic EOS from `INTERACTION_FIXTURE` to the earliest useful
outcome from `OUTCOME_PROBE`, using successful exact-correctness-qualified
CAPTURE observations and the frozen linear `(n-1)` p95 estimator. Reports must
show, per profile/scenario/alternative:

- observed FTOL distribution;
- logical Model call count and configured Model budget;
- delegated execution count and configured Agent budget;
- local execution count and configured Tool budget;
- residual observed FTOL after subtracting the configured budget, labeled as
  framework/runtime plus timer/scheduler residual rather than pure framework
  time.

Raw monotonic timestamps remain authoritative. Configured costs are explanatory
decomposition, not replacement timestamps.

## Architecture neutrality

The implementation accepts only semantic event counts and a profile. It never
accepts or branches on alternative identity when choosing a charge. Equivalent
model, delegated-execution, and local-tool operations receive identical costs
across A/B/C/D. Differences arise only because the preserved architectures
invoke different semantic boundaries or invoke them a different number of
times.

Alternative B remains ARGO-centric primary execution, not a preferred/default
Downstream Agent. C/D Fast Path eligibility remains the approved semantic
capability/policy boundary; no time, model-call-count, or tool-count eligibility
rule is introduced. No alternative implementation file is changed.

## Included and excluded dimensions

Included dimensions are Model invocation, accepted downstream Agent/delegation
execution, and local Tool execution. They correspond directly to existing common
ports and measurable architecture roles.

No independent synthetic dimension is added for voice/realtime boundary,
context acquisition, route/task commit, or IPC/serialization. The current
qualification corpus supplies Acoustic EOS as the common start fixture; context,
commit, and in-process serialization work are ordinary measured runner/AUT
overhead, not separately represented dependencies. There is no separate
cross-process or voice dependency port whose charge could be applied without
inventing an architecture role. Adding one later requires a new profile version
and experiment identity.

## P12 compound protection

P12 remains one parent with distinct S1/S2 children. Both routes must commit in
S1 then S2 order before either execution begins. Each of its two executions is
then independently charged by actual route class, so charges are additive. The
final QA-04 boundary remains S2; both required effects, exact QA-02 conformance,
and QA-04 qualification remain mandatory. Delay begins only inside execution,
after `ExecutionStarted`, which itself can occur only after the preserved
route-plan barrier. Focused R1 regression executes P12 through A/B/C/D and checks
this ordering and semantic equality against Profile Z.

## QA treatment

- **QA-01:** architecture-sensitive FTOL is reported as a vector with the
  configured dependency decomposition above. Profile rankings and crossovers are
  compared without treating raw framework nanoseconds as total E2E latency.
- **QA-02:** exact semantic comparison is unchanged. No numeric acceptance
  threshold is invented. The Session 7.0 P04/P06/P07/P11 defects remain evidence,
  not timing noise.
- **QA-03:** explicitly out of scope. Runtime results do not produce CCR or a
  synthetic flexibility score; the evolution/flexibility experiment remains a
  separate track.
- **QA-04:** `qa04-route-contract-policy-v1` remains unchanged. The Session 7.0
  Alternative × Profile Z failures for A/B/C/D remain visible. Latency injection
  cannot alter route correctness, and no aggregate QA-04 score is introduced.

## Session 7.1 qualification evidence

All checks ran before the freeze commit. The Python 3.14.7 / pytest 9.1.1 suite
passed 95/95 tests. Rust 1.94.0 passed workspace/all-target check, format check,
clippy with warnings denied, qualification build, and 107/107 qualification
tests. These include P01–P12 validation, raw contract and provenance checks,
QA-02 and QA-04 analysis tests, deterministic schedule/pair tests, canonical
event invariants, and P12 CAPTURE/MINIMAL route-plan regressions.

A small, non-campaign preflight ran P01 and P12 through A/B/C/D once for each of
R1–R4: 32 development executions, all with no architecture error. Strict v9
validation returned zero errors and warnings. Two independent derivations from
the same transient raw root produced identical SHA-256
`71e7b8b6c10e9877e2d84687c2550ad57070d4ac9e5ed63f4088007c716093c0`.
The committed evidence summary is
`results/reports/pilot-v0/session-7.1-profile-qualification.md`; transient raw
preflight files are not part of the measured campaign and are not committed.

The existing Session 7.0 raw root also passes strict v9 validation with zero
errors and warnings, demonstrating Profile Z/provenance backward compatibility.

## Future comparative campaign contract

The next campaign shall use the committed Session 7.1 source without edits and:

1. run R1, R2, R3, and R4 as four separate immutable raw roots, in that order;
2. use P01–P12 and A/B/C/D unchanged;
3. use two full prewarm cycles, one invocation warm-up, 16 fixed measured cycles,
   deterministic cyclic alternative rotation, and parity-inverted adjacent
   CAPTURE/MINIMAL pairs from `dp00-calibration-protocol-v1`;
4. retain 1,536 measured executions and 768 complete pairs per profile;
5. record `dp00-pilot-provenance-v4`, exact source SHA, profile ID/version and
   values, corpus/model/prompt/cache/toolchain identities, and qualification build;
6. use `dp00-analysis-v9` strict validation and existing QA-01/02/04 semantics;
7. require zero missing/duplicate/provenance/semantic/pair mismatches, Profile Z
   regression, P12 barrier qualification, and byte-identical re-derivation from
   each immutable raw root;
8. make no profile or architecture change after any A/B/C/D result is inspected.

Each root is invoked with:

```shell
cargo run --profile qualification -p bench-runner \
  --bin dp00-calibration-runner -- \
  --calibration-id <unique-campaign-id>-<R1|R2|R3|R4> \
  --output-root <new-immutable-root> \
  --latency-profile <R1|R2|R3|R4>
```

The report must answer whether B's call reduction becomes advantageous, whether
C/D local execution saves latency, the correctness costs accompanying savings,
Pareto dominance, rank stability, and crossover points. It must use vector/Pareto
analysis and must not create a weighted overall score or declare a winner solely
from this contract.

## Prospective freeze and change control

`benchmark/contracts/dp00-realistic-execution-profiles-v1.json` and the matching
runner constructors are frozen before measurement. Profile Z (`pilot-z-v0`,
0/0/0 µs) and provisional Profile C are unchanged. If any value, distribution,
semantic application point, profile ordering, or population rule changes, the
owner must create a new profile-contract version and new experiment identity;
old raw and derived evidence remains immutable.

## Limitations

R1–R4 do not measure provider inference, network, device, Agent ReAct, or actual
tool-service latency. Constant sleeps exercise dependency placement but omit
real distributions, correlation, queuing, concurrency, streaming, and tail
behavior. The sensitivity matrix supports relative topology and crossover
reasoning only. Production-representative absolute E2E claims require separately
controlled real-stack evidence.

## Decision status

`REALISTIC EXECUTION PROFILE FROZEN — READY FOR COMPARATIVE CAMPAIGN`

This does not change the DP-00 disposition:

`NOT READY FOR ARCHITECTURE DECISION`
