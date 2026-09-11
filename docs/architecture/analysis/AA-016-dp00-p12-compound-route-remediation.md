# AA-016 — DP-00 P12 Compound-route Remediation

## 1. Scope

Session 6.2R repairs the P12 runtime correctness gap before any rotated
instrumentation calibration. P12 stays in corpus v0.2 and its two required
effects and exact-conformance constraints are unchanged.

## 2. Provisional architecture decision

One compound user goal is a parent task/group. S1 (media pause) and S2
(Downloads organization) are distinct child tasks. Both required initial routes
commit before either child starts domain execution; this is the initial
route-plan barrier. Interleaved routing/execution is deferred as a potential
future architecture alternative.

`canonical-event-v3` preserves `episode_id`, `parent_task_id`, `child_task_id`,
`subgoal_id`, route, and commit timestamp. Distinct subgoal commits are valid;
recommit of the same subgoal is a duplicate. No parent-level commit or final
flag exists.

## 3. QA-04 boundary

The scenario manifest declares the required subgoal set. The evaluator derives
the P12 boundary only after exact set coverage and uses the latest required
commit timestamp. S1 commit alone does not close the window. Partial coverage
is `ROUTE_REQUIRED_NOT_COMMITTED`. A numeric Primary candidate additionally
requires QA-02 exact conformance under `qa04-route-contract-policy-v1`.

## 4. Architecture-preserving implementation

| Alternative | Existing responsibility retained | P12 route plan |
| --- | --- | --- |
| A | VIA Intent Refiner + Agent Router | VIA creates parent/children; Agent Router selects ARGO; commits S1 then S2; executes after barrier. |
| B | ARGO-primary interpretation and routing | One ARGO-primary decision supplies both child routes; projection creates parent/children; execution follows the barrier. |
| C | Bounded VIA local fast path plus Agent Router | S1 uses the bounded local media route; S2 uses the existing Agent Router/ARGO route; both commit before execution. |
| D | Intent Refiner + Execution Path Selector | The selector returns the two child routes; both commit before local/ARGO execution. |

No runner special case creates route events, no analyzer reinterprets a
premature commit, and no P12 oracle or requirement is relaxed.

## 5. Regression obligations

The runtime regression executes P12 through A/B/C/D and verifies S1/S2 route
identity, stable commit order, a common parent and distinct children, last
required commit as the QA-04 reference, two post-barrier executions, media and
Downloads effects, and QA-04 raw inclusion. Contract regressions separately
cover unique subgoal commits, duplicate recommit rejection, pre-barrier
execution rejection, partial required-set coverage, and final-boundary
derivation. Offline derivation must show QA-02 exact conformance and one
correctness-qualified QA-04 observation for every alternative.

## 6. Status

This is a provisional DP-00 execution-semantics decision, not a Final A/B/C/D
evaluation result. Calibration remains blocked until the targeted and full
regression suites pass. Results and commit provenance are reported in the
Session 6.2R completion record rather than frozen here before verification.
