# AA-013 — DP-00 Path-Aware Evidence Readiness Review

## Scope and non-goals

This review records Session 4.1 for DP-00 Runtime Pilot v0. Campaign
`official-1789087890289040000` remains an **INVALIDATED DIAGNOSTIC CAMPAIGN**.
Its raw validation and episode completeness passed, but `dp00-analysis-v0`
reported 16 `MISSING_ACTUAL_EVIDENCE` observations. The preserved campaign is
not promoted to Official evidence and its raw or derived bytes are not changed.

This session improves evidence decidability. It does not attempt to make every
architecture constraint pass, change the approved Oracle, or rerun an Official
campaign.

## Pre-fix forensic matrix

All rows below were inspected independently in Profile Z and Profile C. The two
profiles have identical semantic event sequences; only monotonic timings differ.
`Terminal failure` means the authoritative benchmark event
`EpisodeFailed(reason=INVALID_ROUTE)`. A missing route/invocation/effect is not
treated as evidence by itself.

| Profile | Scenario | Alternative | Constraint ID | Expected predicate | Relevant actual raw events | Actual semantic fact present? | Evidence authority | Current analyzer lookup | Root-cause category | Required fix |
|---|---|---|---|---|---|---|---|---|---|---|
| Z | P04 | A | P04-ALLOW-ROUTE | committed file-capable route is in the allowed set | `ReferentBound(doc-right)`; two completed model calls; terminal failure; no route/invocation/result/effect | Allowed route: no. Explicit non-achievement: yes | BENCHMARK terminal + complete canonical stream | `execution_routes` only; empty becomes MISSING | EVALUATOR_PREDICATE_GAP | Map explicit terminal failure to deterministic FAIL for positive/allowed predicates |
| C | P04 | A | P04-ALLOW-ROUTE | same | same semantic sequence as Z | same | same | same | EVALUATOR_PREDICATE_GAP | same |
| Z | P04 | B | P04-REQ-REFERENT | source referent is `doc-right` | one completed ARGO-primary model call; terminal failure; no binding/invocation/effect | Required referent: no. Explicit non-achievement: yes | BENCHMARK terminal + complete canonical stream | `ReferentBound` only; empty becomes MISSING | EVALUATOR_PREDICATE_GAP | Use authoritative binding or outcome subject when present; otherwise explicit terminal failure yields FAIL |
| C | P04 | B | P04-REQ-REFERENT | same | same semantic sequence as Z | same | same | same | EVALUATOR_PREDICATE_GAP | same |
| Z | P04 | B | P04-ALLOW-ROUTE | committed file-capable route is in the allowed set | one completed ARGO-primary model call; terminal failure; no route/invocation/effect | Allowed route: no. Explicit non-achievement: yes | BENCHMARK terminal + complete canonical stream | `execution_routes` only; empty becomes MISSING | EVALUATOR_PREDICATE_GAP | Terminal non-achievement yields FAIL; route/invocation remain primary strategies |
| C | P04 | B | P04-ALLOW-ROUTE | same | same semantic sequence as Z | same | same | same | EVALUATOR_PREDICATE_GAP | same |
| Z | P04 | B | P04-FORBID-LEFT | `doc-left` must not be selected/opened | terminal failure; no binding, execution, or document effect | Forbidden left action: absent in a complete terminal trace | BENCHMARK terminal + captured canonical referent/execution/outcome boundaries | `ReferentBound` only; empty becomes MISSING | EVALUATOR_PREDICATE_GAP | Permit CLOSED_WORLD_ABSENCE only after all completeness conditions pass |
| C | P04 | B | P04-FORBID-LEFT | same | same semantic sequence as Z | same | same | same | EVALUATOR_PREDICATE_GAP | same |
| Z | P04 | C | P04-ALLOW-ROUTE | committed file-capable route is in the allowed set | `ReferentBound(doc-right)`; two completed model calls; terminal failure; no route/invocation/result/effect | Allowed route: no. Explicit non-achievement: yes | BENCHMARK terminal + complete canonical stream | `execution_routes` only; empty becomes MISSING | EVALUATOR_PREDICATE_GAP | Map explicit terminal failure to deterministic FAIL |
| C | P04 | C | P04-ALLOW-ROUTE | same | same semantic sequence as Z | same | same | same | EVALUATOR_PREDICATE_GAP | same |
| Z | P06 | B | P06-REQ-REFERENT | resolved source referent after U1→U2 is `doc-right` | clarification requested at U1; U2 processing/model call; terminal failure; no resolution/binding/execution/effect | Required resolved referent: no. Explicit non-achievement: yes | BENCHMARK terminal + complete canonical stream | `ReferentBound` only; empty becomes MISSING | EVALUATOR_PREDICATE_GAP | Do not use Oracle fallback; explicit terminal failure yields FAIL when no authoritative binding/outcome exists |
| C | P06 | B | P06-REQ-REFERENT | same | same semantic sequence as Z | same | same | same | EVALUATOR_PREDICATE_GAP | same |
| Z | P07 | B | P07-ALLOW-ROUTE | direct ARGO execution path | completed ARGO-primary model call naming ARGO candidate; terminal failure; no commit/invocation/effect | Committed ARGO route: no. Explicit non-achievement: yes | BENCHMARK terminal + complete canonical stream; model candidate is not an Actual route | `execution_routes` only; empty becomes MISSING | EVALUATOR_PREDICATE_GAP | Terminal non-achievement yields FAIL; never promote model candidate to committed Actual |
| C | P07 | B | P07-ALLOW-ROUTE | same | same semantic sequence as Z | same | same | same | EVALUATOR_PREDICATE_GAP | same |
| Z | P07 | B | P07-FORBID-FAST | accepted execution owner is not VIA Fast/local | completed ARGO-primary model call; terminal failure; no route/invocation/effect | Fast execution: absent in a complete terminal trace | BENCHMARK terminal + captured route/execution/outcome boundaries | route-derived owner only; empty becomes MISSING | EVALUATOR_PREDICATE_GAP | CLOSED_WORLD_ABSENCE after terminal/stream/boundary/integrity checks |
| C | P07 | B | P07-FORBID-FAST | same | same semantic sequence as Z | same | same | same | EVALUATOR_PREDICATE_GAP | same |

## Root-cause verdict

Each of the 16 missing observations is an `EVALUATOR_PREDICATE_GAP`.
`reconstruct_actual` already preserved the explicit `INVALID_ROUTE` terminal
outcome, so this is not a raw-emission or analyzer-projection gap. The old
evaluator required a value in the constraint's primary dimension before it
could decide. That rule confused a terminally demonstrated non-achievement with
missing evidence and did not gate negative absence on closed-world conditions.

The runtime traces also expose correctness failures: several replay outputs are
not accepted by the current architecture parser/route implementation. Those
failures are actual behavior and must remain FAIL diagnostics. Changing them is
outside this evidence-completeness session and would improperly conflate
correctness with decidability.

## Closed-world absence rule

`CLOSED_WORLD_ABSENCE` is allowed only for a Forbidden predicate and only when
all of the following hold:

1. exactly one terminal event exists and is the final canonical event;
2. provenance `event_count` equals the captured canonical stream length;
3. instrumentation mode is `CAPTURE` under a recognized canonical schema;
4. the manifest names the relevant captured decision/execution/effect stream;
5. no episode integrity error exists.

If positive forbidden evidence is present, the constraint is FAIL regardless of
closed-world eligibility. If it is absent and any completeness condition is
false, the result is `MISSING_ACTUAL_EVIDENCE`; it is never silently passed.
Required and Allowed positive predicates never pass by absence. An explicit
terminal failure can independently prove their non-achievement and therefore
produces FAIL, not PASS and not Oracle-derived Actual.

## Coverage model

The prior 36-row audit proves only that every Oracle constraint has one generic
mapping. It does not prove that all four architecture paths expose a decidable
representation. The replacement readiness gate is 36 constraints × four
alternatives = 144 cells. Each cell records its semantic predicate reference,
authoritative raw strategy, derivation rule, polarity, absence policy, and an
executable representative test. Profile Z/C invariance is checked separately;
profile identity never changes the semantic evidence contract.

## Implementation and verification

The implementation is analysis-only:

- `dp00-analysis-v1` replaces dimension-presence gating with semantic-value
  matching, terminal non-achievement, and closed-world forbidden evaluation;
- dictionary predicates compare their declared semantic fields rather than
  failing because Actual carries extra provenance fields;
- referent reconstruction accepts explicit AUT binding and authoritative
  document-outcome subject as equivalent representations of the same fact;
- execution path/owner reconstruction accepts committed route identity first
  and accepted invocation identity as a topology-neutral fallback;
- the historical 36-row map remains present, while the new path-aware manifest
  contains 144 explicit constraint-alternative cells and no Actual values;
- Rust raw emission, routing, retry, Fast eligibility, task ownership, B ARGO
  responsibility, D selection, and the Oracle are unchanged.

## Sixteen final verdicts

The eight semantic cases below apply independently to both Profile Z and
Profile C, producing 16 explicit verdicts.

| Missing item | Root cause | Actual evidence | Authority | New result | Architecture changed | Oracle changed |
|---|---|---|---|---|---|---|
| P04-A / P04-ALLOW-ROUTE | EVALUATOR_PREDICATE_GAP | terminal `INVALID_ROUTE`, no committed route/invocation/effect | BENCHMARK terminal + complete captured stream | FAIL / `TERMINAL_FAILURE_NON_ACHIEVEMENT` | NO | NO |
| P04-B / P04-REQ-REFERENT | EVALUATOR_PREDICATE_GAP | terminal `INVALID_ROUTE`, no binding or outcome subject | BENCHMARK terminal + complete captured stream | FAIL / `TERMINAL_FAILURE_NON_ACHIEVEMENT` | NO | NO |
| P04-B / P04-ALLOW-ROUTE | EVALUATOR_PREDICATE_GAP | terminal `INVALID_ROUTE`, no committed route/invocation | BENCHMARK terminal + complete captured stream | FAIL / `TERMINAL_FAILURE_NON_ACHIEVEMENT` | NO | NO |
| P04-B / P04-FORBID-LEFT | EVALUATOR_PREDICATE_GAP | no left binding/execution/effect in complete terminal trace | AUT/execution/outcome boundaries + BENCHMARK terminal | PASS / `CLOSED_WORLD_ABSENCE` | NO | NO |
| P04-C / P04-ALLOW-ROUTE | EVALUATOR_PREDICATE_GAP | terminal `INVALID_ROUTE`, no committed route/invocation/effect | BENCHMARK terminal + complete captured stream | FAIL / `TERMINAL_FAILURE_NON_ACHIEVEMENT` | NO | NO |
| P06-B / P06-REQ-REFERENT | EVALUATOR_PREDICATE_GAP | clarification requested, then terminal `INVALID_ROUTE`; no resolution/binding/outcome | BENCHMARK terminal + complete captured stream | FAIL / `TERMINAL_FAILURE_NON_ACHIEVEMENT` | NO | NO |
| P07-B / P07-ALLOW-ROUTE | EVALUATOR_PREDICATE_GAP | model candidate ARGO is not Actual; terminal `INVALID_ROUTE`, no committed route/invocation | BENCHMARK terminal + complete captured stream | FAIL / `TERMINAL_FAILURE_NON_ACHIEVEMENT` | NO | NO |
| P07-B / P07-FORBID-FAST | EVALUATOR_PREDICATE_GAP | no Fast route/invocation/effect in complete terminal trace | route/execution/outcome boundaries + BENCHMARK terminal | PASS / `CLOSED_WORLD_ABSENCE` | NO | NO |

Category totals across both profiles are:

```text
RAW_EMISSION_GAP             0
ANALYZER_MAPPING_GAP         0
EVALUATOR_PREDICATE_GAP     16
CONTRACT_MAPPING_GAP         0
ORACLE_OVER_SPECIFICATION    0
OTHER                        0
```

## Diagnostic replay

The preserved raw directory was read without modification and derived output
was written only to a temporary development location. Results:

```text
original dp00-analysis-v0 MISSING_ACTUAL_EVIDENCE = 16
diagnostic dp00-analysis-v1 MISSING_ACTUAL_EVIDENCE = 0
diagnostic dp00-analysis-v1 UNEVALUABLE            = 0

constraints                      = 36
alternatives                     = 4
expected/evaluated cells         = 144 / 144
PASS correctness cells           = 129
FAIL correctness cells           = 15
profile semantic status invariant = true
```

Across the two-profile campaign constraint observations, 258 are PASS and 30
are FAIL. These values are readiness diagnostics, not architecture ranking
evidence. The campaign remains invalidated because its original frozen analysis
contract was `dp00-analysis-v0`.

For Profile Z, every P04/P06/P07 A/B/C/D path is evidence-complete. The original
eight semantic missing cases resolve as six FAIL outcomes and two PASS outcomes;
Profile C produces the same statuses.

## Version disposition

```text
canonical event schema    canonical-event-v2 -> canonical-event-v2
model-call schema         model-call-v1      -> model-call-v1
analysis version          dp00-analysis-v0   -> dp00-analysis-v1
coverage manifest         dp00-path-aware-evidence-v1
run provenance schema     dp00-pilot-provenance-v2 (unchanged)
campaign provenance       dp00-pilot-campaign-provenance-v1 (unchanged)
```

No raw schema bump is needed because no Rust payload or emission semantics
changed.

## Regression result

All required checks passed:

```text
cargo fmt --check                                             PASS
cargo clippy --workspace --all-targets --all-features         PASS
cargo test --workspace                                        PASS
cargo test -p bench-smoke --test smoke_matrix                 PASS (20 paths)
cargo test -p bench-smoke --test fault_paths                  PASS
cargo test -p bench-events --test contract_invariants         PASS
cargo test -p bench-replay --test hidden_context              PASS
cargo build --workspace --profile qualification               PASS
.venv/bin/python -m pytest benchmark/analysis                 PASS (32 tests)
```

The workspace run includes pilot asset validation (8 tests), campaign/runtime
tests (12 tests), and the QA-02 raw/path-aware contract suite (9 tests). The
wrong-behavior checks remain discriminating: P07-D produces four constraint
FAIL results, and terminally failed P04/P06/P07 paths are not converted to
correctness PASS merely to complete evidence.

No file under `alternative-a`, `alternative-b`, `alternative-c`, or
`alternative-d` changed. Therefore A remains Thin VIA, B remains ARGO-centric
Primary Execution, C remains A plus bounded VIA Fast Path, and D remains
adaptive per-turn execution. QA-01 timed-path code is unchanged.

## Revised Official Pilot readiness

| Gate | Verdict |
|---|---|
| Invalidated Campaign Preservation | PASS |
| 16-case Root Cause Resolution | PASS |
| Topology-neutral Evidence Semantics | PASS |
| 36×4 Constraint-Alternative Coverage | PASS |
| Missing Actual Evidence Gate | PASS |
| Actual/Oracle Independence | PASS |
| Closed-world Absence Semantics | PASS |
| QA-01 Timed-path Preservation | PASS |
| Base Architecture Regression | PASS |
| Official Pilot Re-run Readiness | PASS after commit/push and clean-tree closure |

The next Official campaign must run in a separate Session 4.2 clean session.
No Official campaign was run in Session 4.1.
