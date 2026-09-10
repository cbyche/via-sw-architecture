# AA-010 — DP-00 Prototype Conformance Review

## Status

**Formal prototype conformance review — PASS**

This review compares the Rust Base prototype at `prototypes/dp00/` with the approved DP-00 executable architecture specification. It is not a performance result, QA score, or A/B/C/D ranking.

Review baseline:

```text
Starting implementation SHA: bd33e1ddef6a09d440f806c17f88ed4b84462638
Specification: docs/architecture/decision-points/DP-00-executable-architecture-spec.md
Positive regression: S1~S5 × A/B/C/D = 20 paths
Qualification tests after Phase 3/4: 43 passed, 0 failed
```

## 1. Review method

The review used four evidence classes:

1. source and Cargo dependency inspection;
2. the 20-path positive smoke matrix and expected ModelCall trace;
3. executable N1~N9 negative-contract tests;
4. fault injection for model-status failures, wrong candidates, synchronous dispatch rejection and retry/reselection.

The frozen specification was not changed to fit implementation results.

## 2. Alternative conformance

### A — Thin VIA

**PASS.** A retains Context Engine → Intent Refiner → Agent Router → Task Manager → Agent Harness. VIA owns top-level Agent routing; ARGO remains an ordinary downstream candidate and is not a privileged primary runtime. There is no VIA Fast Path. On synchronous candidate rejection, reselection stays with `A.AgentRouter`; it creates one new logical generation before the final accepted route is committed.

### B — ARGO-centric Primary

**PASS.** B has no VIA Intent Refiner or VIA Agent Router. `B.ARGOPrimary` makes one combined request carrying interpretation, referent, initial execution-route, and self-vs-specialist Agent-selection responsibilities. The runtime test proves that thin context cannot directly reach the common executor without this request. VIA retains only task projection/correlation; the ARGO route remains authoritative for execution.

### C — Hybrid VIA Fast Path

**PASS.** C remains independently implemented rather than importing A. Its deterministic Fast Eligibility is bounded to explicit local capabilities; non-fast work still flows through C's Agent Router. The Fast Executor has not expanded into a domain planner or general-purpose Agent runtime.

### D — Adaptive Per-turn

**PASS.** D retains Intent Refiner + Execution Path Selector and has no second top-level Agent Router. The Selector returns both route kind and executor. Clear follow-up reuses task state and skips the Selector; new-route cases use it.

## 3. Common harness and isolation

| Control | Verdict | Executable evidence |
| --- | --- | --- |
| Architecture decision logic is not commonized | PASS | Alternative crates own intent/routing/selection/topology code; common crates expose only neutral ports, fixtures, replay, events and runner mechanics. |
| Oracle dependency isolation | PASS | Cargo metadata traverses direct and transitive dependencies for exactly `alternative-a`, `alternative-b`, `alternative-c`, and `alternative-d`. |
| Scenario/benchmark-key isolation | PASS | AUT-visible request serialization whitelist plus source guards exclude scenario, behavior-plan, semantic-operation, expected-route, QA and ground-truth identifiers. |
| Frozen responsibility mapping | PASS | `ResponsibilityMapping::dp00_base()` rejects unsupported owner/responsibility pairs and empty self-labelled requests. |
| B responsibility isolation | PASS | Runtime trace requires the single `B.ARGOPrimary` combined pre-route request before execution. |
| Observation provenance isolation | PASS | AUT supplies semantic observations only; the benchmark adapter assigns run/scenario/alternative ids, sequence, monotonic timestamp and emitter. |

## 4. Negative anti-gaming matrix

| Test | Verdict | Failure enforced |
| --- | --- | --- |
| N1 Oracle dependency leakage | PASS | Any direct/transitive Alternative → `bench-oracle` reachability fails. |
| N2 Scenario-id / benchmark-key leakage | PASS | Forbidden AUT input/source fields fail API/source guards. |
| N3 Semantic responsibility violation | PASS | Invalid A.AgentRouter and D.ExecutionPathSelector responsibility requests are rejected. |
| N4 B ARGO responsibility leakage | PASS | B must issue the fused ARGOPrimary pre-route responsibility request. |
| N5 Premature Execution Route Commit | PASS | A rejected candidate is not committed; the checker rejects commitment of a synchronously rejected candidate. |
| N6 Duplicate initial Route Commit | PASS | More than one initial commit is a contract violation. |
| N7 Candidate rejection → reselection | PASS | NetworkAgent reject → A.AgentRouter retry → ARGO accept → exactly one commit. |
| N8 Fake Useful Outcome | PASS | AUT-emitted success is non-authoritative; completed episodes require OutcomeProbe evidence. |
| N9 Observation provenance spoofing | PASS | Provenance fields do not exist on the AUT observation API and are assigned by the adapter. |

These controls demonstrate that evaluation-neutrality rules are executable safeguards rather than document-only conventions.

## 5. Fault-path semantics

The same route-decision fault classes are injected into A/B/C/D without exposing fault identity or benchmark keys to Alternative code.

| Fault | Observed behavior | Verdict |
| --- | --- | --- |
| MALFORMED | Every Alternative rejects the non-completed model status, emits no route commit, performs no execution, produces no useful outcome, and terminates FAILED. | PASS |
| TIMEOUT | Deterministic FAILED terminal; no hang, commit, execution or useful outcome. | PASS |
| NO_RESPONSE | Deterministic FAILED terminal; no hang, commit, execution or useful outcome. | PASS |
| WRONG_CANDIDATE | Unsupported common candidate is visible in raw request/response evidence and fails without fabricated recovery. | PASS |
| synchronous dispatch reject | Rejected candidate remains provisional; A reselects through its owning Agent Router and commits only accepted ARGO. | PASS |

The Replay Adapter independently proves `MALFORMED attempt 1 → CORRECT attempt 2` consumption. Two calls to `generate` are two logical generations and two ordered attempts. The Base specification does not mandate a universal MALFORMED retry policy, so the A/B/C/D Base implementations safely terminate rather than adding an optional recovery tactic. The candidate-reselection case does require an architectural retry and its additional ModelCall is recorded.

## 6. ModelCall and terminal evidence

`LogicalModelCall` preserves logical sequence, decision owner, semantic responsibilities, attempt, completion status, timestamps, and route-commit-before/after relation. The enforced accounting rule remains:

```text
1 ReplayModelRequest = 1 logical Generative AI Model Call
```

Positive smoke retains the specification trace:

| Scenario | A | B | C | D |
| --- | ---: | ---: | ---: | ---: |
| S1 Local | 2 | 1 | 1 | 2 |
| S2 General | 2 | 1 | 2 | 2 |
| S3 Specialized | 2 | 1 | 2 | 2 |
| S4 Follow-up | 1 | 1 | 1 | 1 |
| S5 Clarification | 3 | 2 | 2 | 3 |

For A candidate rejection/reselection, the trace is three calls: Intent Refiner, Agent Router attempt 1, and Agent Router retry attempt 2. The rejected NetworkAgent candidate has no commit relation; the accepted ARGO route has exactly one commit. This is specification-conformance evidence, not a QA-04 aggregate or performance score.

Terminal evidence distinguishes `EpisodeCompleted` and `EpisodeFailed`. A failed episode may have zero route commits and must not contain a useful outcome. How such episodes enter a later QA-04 aggregate remains intentionally unresolved until Pilot/calibration.

## 7. Canonical invariants

The common checker enforces only topology-neutral partial orders:

```text
success:
processing_started → model generation(s) → one route_committed
→ execution_started → OutcomeProbe useful_outcome → completed

clarification:
clarification_requested → scripted reply → clarification_resolved

failure-before-route:
processing_started → model failure → no commit → failed

reselection:
candidate observed → rejected → new candidate → one final commit
```

Component-private event ordering is not promoted into a common architecture contract.

## 8. Specification conflict and unresolved scope

No implementation/specification conflict was found. No approved requirements or executable architecture baseline was modified.

The specification deliberately leaves two later decisions open:

- a universal model-failure retry/recovery policy is not part of the Base; and
- the QA-04 aggregation rule for terminal no-route-commit episodes remains TBD until Pilot/calibration.

Neither gap prevents raw fault behavior, attempts, call accounting, terminal state, route-commit absence, or outcome absence from being preserved. No scoring threshold, correctness gate, final corpus, actual LLM/ARGO, production integration, or optional optimization tactic was added.

## 9. Formal verdict

```text
A Architecture Conformance                 PASS
B Architecture Conformance                 PASS
C Architecture Conformance                 PASS
D Architecture Conformance                 PASS

Benchmark Responsibility Isolation         PASS
Oracle Isolation                           PASS
Semantic Replay Contract                   PASS
Route-Commit Semantics                     PASS
Outcome-Probe Authority                    PASS
ModelCall Accounting                       PASS
Negative Anti-gaming Enforcement           PASS
Fault-path Smoke                           PASS

Prototype Conformance Review               PASS
Pilot Readiness                            PASS
```

`Pilot Readiness = PASS` means only that no known evaluation-validity blocker prevents starting Pilot calibration. It does not claim superior performance, correctness scores, or an A/B/C/D winner.
