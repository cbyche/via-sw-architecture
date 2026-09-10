# AA-007 — DP-00 Benchmark Contract Neutrality Review

## Status

**Architecture Analysis Record — Canonical Benchmark Contract Stress-test — Pre-Prototype / Pre-Pilot**

This record stress-tests the benchmark/evaluation contract for DP-00 Base Architectures A/B/C/D before prototype implementation.

It is not a benchmark result, not a QA score, and not a DP-00 winner selection.

The review asks whether the benchmark itself could accidentally decide, simplify, or bias the architecture behavior it is supposed to measure.

Primary artifacts under review:

- `docs/evaluation/dp00-experimental-boundary.md`
- `benchmark/schemas/runtime-scenario-schema.md`
- `benchmark/schemas/semantic-behavior-plan-schema.md`
- `benchmark/schemas/canonical-event-schema.md`
- `benchmark/contracts/runtime-fixture-contracts.md`
- `benchmark/catalog/runtime-scenario-catalog.md`

---

# 1. Review criteria

The contract is acceptable only if all of the following hold:

1. A/B/C/D receive the same product problem and controlled dependency behavior.
2. The AUT cannot read evaluator ground truth.
3. The harness does not perform architecture-owned decision responsibilities.
4. Same semantic faults can be injected across different model-call topologies.
5. Deterministic architecture logic remains allowed and observable.
6. Replay does not erase logical model-call accounting.
7. B's ARGO-centric decision authority is not replaced by a common stub.
8. Useful-outcome timing cannot be improved by AUT self-report.
9. Local capability ownership does not become a product-capability advantage.
10. Optional tactics do not contaminate the Base comparison.
11. Canonical events remain meaningful across all alternatives.
12. Corpus/aggregation rules can be frozen without losing raw evidence needed for alternate analyses.

---

# 2. Attack A — Oracle leakage

## Attack

A Runtime Scenario source file necessarily contains evaluator truth such as:

- correct referent;
- required clarification;
- allowed/forbidden route;
- expected result binding;
- success predicate.

If the AUT sees these fields, QA-02 and QA-01 become trivial and invalid.

## Control

The Runtime Scenario contract separates:

```text
AUT-visible Stimulus
    vs
Evaluator-only Oracle
```

AUT-visible:

- user input;
- raw context evidence;
- logical initial state;
- capability/health facts;
- policy state;
- dependency responses.

Evaluator-only:

- ground-truth referent/task relation;
- Required/Allowed/Forbidden constraints;
- expected result binding;
- success predicate internals;
- scoring eligibility and expected architecture-sensitive labels.

`scenario_id` is correlation metadata only and cannot be used as a decision feature.

## Verdict

**PASS — provided the Prototype/Harness Specification enforces separate materialization paths.**

Residual risk: implementation code can accidentally share one deserialized scenario object with AUT adapters. The next harness spec must define separate stimulus/oracle types or access boundaries.

---

# 3. Attack B — Harness responsibility leakage

## Attack

The runner could make the experiment easier by resolving:

- referents;
- task association;
- clarification need;
- Fast eligibility;
- route selection;
- Agent selection.

Then A/B/C/D would no longer differ in the responsibilities DP-00 is testing.

## Control

The experimental boundary explicitly reserves these decisions to the AUT.

Shared fixtures provide facts/evidence and frozen dependency behavior only.

The State Seeder runs before the episode and may not resolve the current turn.

The Semantic Replay Provider returns requested semantic output but does not independently invoke routing/task logic.

## Verdict

**PASS.**

Prototype acceptance tests should include negative tests proving that no fixture API exposes a pre-resolved route/task/referent result.

---

# 4. Attack C — Fused vs separate model topology bias

## Attack

If replay is keyed by model-call ordinal:

```text
Call #1 = correct
Call #2 = wrong route
```

then A with two calls and B with one fused ARGO call cannot receive the same semantic condition.

## Control

Replay is keyed by semantic responsibility/operation:

```text
turn1.intent = CORRECT
turn1.route  = WRONG_CANDIDATE
```

A may request the two operations in separate logical generations.

B may request both operation keys in one ARGO generation.

The same semantic fault is therefore injected without requiring the same call topology.

## Verdict

**PASS.**

This is a central benchmark-neutrality result of the checkpoint.

Reviewer-facing explanation:

> **동일한 semantic success/error condition을 responsibility 단위로 A/B/C/D에 주입하여 Model intelligence를 통제하고 SW topology의 validation/recovery 차이를 비교한다.**

---

# 5. Attack D — Deterministic architecture is forced to call a model

## Attack

A replay corpus could accidentally force every alternative to consume each semantic operation, erasing legitimate deterministic mechanism differences.

## Control

Semantic operations are consumed only when an allowed AUT owner requests a logical Generative AI generation.

If the architecture performs a fact/state/contract/policy decision deterministically, the corresponding semantic operation can remain unused.

Unused operations are diagnostic evidence, not errors.

## Verdict

**PASS.**

This preserves the Base Decision Mechanism rule from the executable architecture specification.

---

# 6. Attack E — Replay makes QA-04 Model Calls disappear

## Attack

Because the benchmark uses deterministic replay instead of a live neural model, a naive implementation might report zero model calls.

## Control

Every AUT-requested logical Generative AI generation creates one ModelCall record before replay returns the frozen semantic payload.

```text
replay-backed logical generation = 1 ModelCall
```

Streaming chunks do not increase the count.

Deterministic architecture code with no generation remains 0.

## Verdict

**PASS.**

The existing QA-04 ModelCall contract remains authoritative.

---

# 7. Attack F — Semantic-responsibility metadata gaming

## Attack

An AUT could label a route-selection generation as `domain_reasoning` or omit a route operation key to avoid a frozen wrong-route behavior and/or QA-04 inclusion.

## Control

Before qualification, freeze:

```text
component / decision owner
  -> allowed semantic responsibilities
```

from the executable Base Architecture specification.

The Replay Model Adapter validates `decision_owner`, `semantic_operation_keys[]` and expected output schema against this mapping.

The AUT cannot arbitrarily redefine responsibility metadata at runtime.

## Verdict

**PASS WITH IMPLEMENTATION REQUIREMENT.**

Machine-readable owner/responsibility mapping format is deferred to Prototype/Harness Specification, but semantic ownership is already fixed.

---

# 8. Attack G — B's ARGO responsibility is hidden inside a common stub

## Attack

A/C use ARGO as a downstream executor, while B uses ARGO as primary interpretation/routing/execution authority.

If benchmark code replaces all ARGO behavior with one common stub, B loses its defining architecture responsibility and becomes artificially thin.

## Control

Split the seam:

```text
Architecture-owned pre-route decision
    ↓
Execution Route Commit
    ↓
Common deterministic domain behavior
```

For B, `ARGOPrimary` interpretation and self-vs-specialist decision remain AUT code.

Only post-route domain behavior may be fixture-controlled when equivalent.

## Verdict

**PASS.**

This is a required prototype boundary, not optional implementation advice.

---

# 9. Attack H — QA-01 useful-outcome self-report gaming

## Attack

An architecture could emit `completed=true` before the product-visible side effect actually occurs and appear to have lower FTOL.

## Control

`useful_outcome.observed` is authoritative from an external `OUTCOME_PROBE` whenever the outcome is independently observable.

Examples:

- actual fixture volume state;
- usable/open file state;
- interaction output sink receiving first meaningful result.

AUT completion events remain diagnostic only.

## Verdict

**PASS.**

This complements existing QA-01 Acoustic-EOS and fixture-latency controls.

---

# 10. Attack I — Local capability fairness

## Attack

C/D appear to have extra functionality because they contain VIA Fast Path executors that A/B do not.

## Control

The same product capability/tool fixture exists for all alternatives.

Difference:

```text
A/B
  reach capability through ARGO/Agent execution ownership

C/D
  may own the same bounded capability locally
```

The capability is common; **execution ownership** differs.

## Verdict

**PASS.**

This preserves `Common Product Obligation != Common Internal Component Structure`.

---

# 11. Attack J — Canonical route vocabulary favors a topology

## Attack

Enums such as:

```text
VIA_FAST
ARGO_PRIMARY
SPECIALIST_DIRECT
```

are useful architecture labels but encode topology into the benchmark representation.

## Control

Canonical route uses:

```text
LOCAL_DIRECT
EXECUTOR_DIRECT
EXECUTOR_DELEGATED

initial_executor_id
final_executor_id_if_known
delegation_chain[]
```

All A/B/C/D paths can be represented without adding one alternative's vocabulary to the oracle.

## Verdict

**PASS.**

---

# 12. Attack K — Provisional route commit gaming

## Attack

An architecture could emit `execution.route_committed` early, then synchronously discover that the executor rejects the request and choose another route. This could shorten QA-04 by moving later routing calls outside the boundary.

## Control

Route Commit occurs only when the initial operational route is stable enough that domain execution can proceed without another initial owner/delegation decision.

A candidate synchronously rejected before execution can proceed is not the final commit.

Post-commit domain subtask delegation remains separate DOMAIN behavior.

## Verdict

**PASS WITH PROTOTYPE VALIDATION REQUIRED.**

Harness tests must validate the commit event against subsequent dispatch/accept evidence.

---

# 13. Attack L — Corpus imbalance

## Attack

Overall p95/mean/rate can be dominated by one scenario class, hiding architecture differences elsewhere.

## Control

The R1~R10 catalog preserves:

- scenario class;
- architecture-sensitivity tags;
- QA eligibility;
- raw per-episode evidence.

Final scenario counts and aggregation remain open through Pilot.

QA-04 overall mean vs class macro-average is intentionally not frozen now.

Usage-weighted sensitivity can be recomputed later.

## Verdict

**PASS WITH TBD.**

TBD: final class composition, weighting and QA-04 aggregation-v1.

---

# 14. Attack M — QA-04 terminal no-route-commit ambiguity

## Attack

A malformed/timeout path may never reach Execution Route Commit. Assigning zero calls rewards failure; assigning an arbitrary huge count mixes correctness with efficiency.

## Control

Scenario/run evidence preserves:

```text
qa04_eligible
route_commit_expected
route_commit_observed
ModelCall events
failure reason
exact conformance
```

No arbitrary Primary aggregation rule is selected in this checkpoint.

Pilot/calibration will freeze the treatment before final A/B/C/D results.

## Verdict

**PASS WITH TBD.**

The ambiguity is explicitly preserved rather than hidden in an arbitrary rule.

---

# 15. Attack N — Tactic contamination

## Attack

One Base could receive:

- classifier routing;
- fused Intent+Router generation;
- cache optimization;
- speculative routing;
- pre-EOS semantic preparation;

while another remains unoptimized.

The result would compare architecture + tactic bundles.

## Control

`base-architecture-vs-tactic-evaluation.md` remains authoritative.

Base qualification excludes optional optimization tactics. Base model/prompt/cache profiles and common front-end are frozen.

Tactics are evaluated later as Base + Tactic deltas.

## Verdict

**PASS.**

---

# 16. Attack O — Interaction front-end becomes a hidden DP variable

## Attack

If A uses final transcript while B uses pre-EOS partial semantics, QA-01/QA-04 may mainly measure DP-01/Voice composition rather than DP-00 execution boundary.

## Control

DP-00 Base qualification uses a common controlled interaction front-end and canonical input handoff.

Pre-EOS/partial-ASR semantic optimization is not enabled in Base.

B's Thin Context Packaging vs A/C/D Context Engine remains AUT-owned after canonical input handoff.

## Verdict

**PASS.**

---

# 17. Runtime / Evolution / Mandatory Gate separation

## Review

Trying to put QA-03 code-diff experiments and Security/Recovery gates into the same RuntimeScenario schema would obscure measurement units and evidence semantics.

The checkpoint keeps:

```text
Runtime Episode Benchmark -> QA-01/02/04
Evolution Benchmark       -> QA-03
Mandatory Qualification   -> PASS/FAIL gate scenarios
```

## Verdict

**PASS.**

Mandatory-gate executable schemas remain future work; no universal runtime schema is forced onto them.

---

# 18. Final verdict

| Review item | Verdict | Meaning |
| --- | --- | --- |
| **Canonical Benchmark Contract** | **PASS WITH TBD** | Core contracts are implementable; final corpus/serialization/aggregation remain open for Pilot. |
| **Architecture Neutrality** | **PASS** | Common product problem and controlled behavior are separated from topology-specific responsibility ownership. |
| **Experimental Boundary** | **PASS** | Shared fixtures do not intentionally absorb DP-00 decision responsibilities. |
| **Stimulus / Oracle Separation** | **PASS WITH IMPLEMENTATION REQUIREMENT** | Semantic separation is defined; harness must enforce separate access paths. |
| **Semantic Replay Neutrality** | **PASS** | Responsibility-based operations support separate/fused call topologies and equal fault injection. |
| **QA-04 Accounting Consistency** | **PASS WITH TBD** | Logical calls/route commit remain consistent; no-route-commit aggregation is intentionally unresolved. |
| **QA-01 Outcome Authority** | **PASS** | External Outcome Probe prevents useful-outcome self-report gaming. |
| **ARGO Fixture Boundary** | **PASS** | B's pre-route ARGO responsibility remains AUT-owned. |
| **Prototype/Harness Readiness** | **PASS** | Required conceptual interfaces and hidden/shared boundaries are sufficiently defined for the next specification stage. |

`PASS` here means **benchmark-design readiness**, not successful architecture benchmark performance.

---

# 19. Remaining TBD before final scoring

- final R1~R10 scenario count and exact corpus;
- class weighting/population mix;
- QA-04 overall mean vs class macro-average;
- QA-04 terminal no-route-commit treatment;
- exact dependency latency profile values;
- score thresholds;
- QA-02 correctness gate value;
- concrete serialization format;
- actual-model trace corpus collection;
- machine-readable owner/responsibility mapping;
- Mandatory Qualification Suite executable schemas.

These TBDs do not require redefining A/B/C/D Base Architecture or the benchmark neutrality boundary.