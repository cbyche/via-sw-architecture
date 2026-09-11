# AA-015 — DP-00 R8/R9 Completion and QA-04 Route-contract Policy

## 1. Scope and motivation

AA-014 left two explicit calibration gaps: Runtime classes R8/R9 were not covered, and QA-04's committed-only numeric population could reward an architecture that fails cheaply before a required route commit. This follow-up completes the taxonomy coverage and specifies a result-independent route contract. It does not run an Official campaign, alter A/B/C/D, select a winner, set a QA-02 gate, or freeze QA-04 overall versus macro aggregation.

The source taxonomy is `benchmark/catalog/runtime-scenario-catalog.md`. Existing Official campaigns `official-1789090497263081000` and `official-1789087890289040000` remain immutable and retain corpus v0.1 plus `dp00-path-aware-evidence-v1` interpretation.

## 2. Reconstructed R1–R10 taxonomy and coverage

| Runtime class | Semantic purpose | Scenario(s) after this follow-up | QA-01 | QA-02 | QA-04 contract | Old → new coverage |
| --- | --- | --- | :---: | :---: | --- | --- |
| R1 | Bounded/local execution ownership | P01 | yes | yes | REQUIRED | 1 → 1 |
| R2 | General-purpose Agent execution | P02 | yes | yes | REQUIRED | 1 → 1 |
| R3 | Direct/delegated specialist routing | P03 | yes | yes | REQUIRED | 1 → 1 |
| R4 | Context/referent grounding | P04 | no | yes | REQUIRED | 1 → 1 |
| R5 | Existing-task follow-up and result binding | P05 | no | yes | REQUIRED | 1 → 1 |
| R6 | Ambiguity and clarification | P06 | no | yes | REQUIRED | 1 → 1 |
| R7 | False-fast/wrong-route validation | P07 | no | yes | REQUIRED | 1 → 1 |
| R8 | Interleaved task/result identity and binding | P11 | no | yes | FORBIDDEN | 0 → 1 |
| R9 | Compound decomposition and multiple execution paths | P12 | no | yes | REQUIRED | 0 → 1 |
| R10 | Dependency/model error and recovery | P08, P09, P10 | no | yes | FORBIDDEN, OPTIONAL, FORBIDDEN | 3 → 3 |

The corpus changes from 10 to 12 scenarios. QA-04 contract distribution changes from REQUIRED/OPTIONAL/FORBIDDEN = 7/1/2 to 8/1/3. QA-01 remains 3 scenarios and QA-02 becomes 12 scenarios.

## 3. R8 and R9 scenario design

### P11 / R8-001

P11 presents two completed tasks whose results arrived out of initiation order: document result R2 for T2, then Wi-Fi result R1 for T1. The semantic obligation is to retain both task identities during delivery. The results already exist, so no new domain execution route is authorized; `route_commit_expectation=FORBIDDEN` follows from the product semantics rather than any alternative's topology.

Constraints are:

- Required: result bindings to T1 and T2.
- Allowed: every delivered binding belongs to `{T1, T2}`.
- Forbidden: a new route commit or an unbound result.

Actual evidence comes from `Architecture.ResultBound` product correlation, terminal trace completeness, and `Architecture.RouteCommitted`. Authorities are AUT committed decisions plus product-correlation evidence. No Oracle value is copied into Actual reconstruction, and closed-world absence is used only for Forbidden predicates.

### P12 / R9-001

P12 uses the catalog's compound shape: pause media and organize Downloads in one utterance. It requires a bounded media subgoal and a planning-required Downloads subgoal while preserving one user goal. `route_commit_expectation=REQUIRED` because successful completion necessarily commits the execution route for every domain-execution subgoal.

Constraints are:

- Required: two-subgoal decomposition, observed media pause, and Downloads organization by a domain executor.
- Allowed: local or ARGO execution for the bounded subgoal and ARGO for the planning subgoal.
- Forbidden: local Fast claiming the planning effect, or collapse into one unrelated generic task.

Actual evidence comes from `RouteCommitted`, `ExecutionStarted`, `ResultBound`, and authoritative `UsefulOutcomeObserved` effects. Task/result correlation and capability/effect identities make all positive and negative predicates deterministically decidable.

Both scenarios are derived from the pre-existing R8/R9 definitions. They do not encode an A/B/C/D component name as the required answer and were not selected to move a Pilot aggregate.

## 4. Development execution and evidence gate

Profile Z, CAPTURE, one development repetition was run for P01–P12 × A/B/C/D. This was not an Official campaign and latency was not interpreted.

| Scenario | A | B | C | D | MISSING | UNEVALUABLE |
| --- | --- | --- | --- | --- | ---: | ---: |
| P11 / R8 | FAIL (1/5 constraints PASS) | FAIL (1/5) | FAIL (1/5) | FAIL (1/5) | 0 | 0 |
| P12 / R9 | FAIL (2/6 constraints PASS) | FAIL (2/6) | FAIL (2/6) | FAIL (2/6) | 0 | 0 |

FAIL is a valid architecture-correctness result. No A/B/C/D implementation was changed to make the new workloads pass. Across the complete development corpus, the new path-aware gate reports 47 constraints × 4 alternatives = 188/188 evaluated, 141 PASS, 47 FAIL, MISSING 0, UNEVALUABLE 0.

The original v0.1 map remains 36 × 4 = 144 cells. The new `dp00-path-aware-evidence-v2` map is selected only for corpus v0.2 evidence. Re-deriving the valid Official v0.1 campaign still selects v1 and returns 144/144 with MISSING 0 and UNEVALUABLE 0.

## 5. QA-04 route-contract taxonomy

The machine-readable policy is `benchmark/contracts/qa04-route-contract-policy-v1.json`.

Contract expectation is a predeclared scenario property and is not observed behavior:

- ROUTE_REQUIRED: all domain-execution subgoal routes must be committed for successful conformance.
- ROUTE_OPTIONAL: a valid commit and a safe terminal non-commit are both permitted.
- ROUTE_FORBIDDEN: no domain-execution route is authorized.

Offline derivation combines that contract with raw route evidence to produce:

`ROUTE_COMMITTED`, `ROUTE_REQUIRED_NOT_COMMITTED`, `ROUTE_OPTIONAL_NOT_COMMITTED`, `ROUTE_FORBIDDEN_NOT_COMMITTED`, or `ROUTE_FORBIDDEN_BUT_COMMITTED`.

The first value is used only for REQUIRED/OPTIONAL contracts. A FORBIDDEN commit has its own violation status. These are derived observations, not new raw fields.

## 6. Candidate no-route policies

| Policy | Semantic validity | Gaming resistance | Neutrality | Interpretability | Corpus sensitivity | No-route behavior |
| --- | --- | --- | --- | --- | --- | --- |
| A: committed-only, all contracts mixed | Low: mixes requests for which commit has different meanings | Low: required early failure disappears | Formula-neutral but contract-blind | Misleading | High | null is reported but does not disqualify comparison |
| B: contract-aware strata | High | Medium: exposes required failures but needs a qualification rule | High | High | Visible by stratum | REQUIRED, OPTIONAL, FORBIDDEN separated; no invented value |
| C: correctness-qualified contract-aware comparison | Highest | High: incomplete/incorrect REQUIRED population cannot produce a comparable primary aggregate | High | High | Explicit and auditable | null plus diagnostic; group qualification fails |

Policy A is rejected as a stand-alone selection rule. Arbitrary penalties such as `max+1` or a fixed call count are rejected because they change resource semantics and numerically mix correctness into QA-04. Policy C using Policy B's strata is selected as the candidate.

## 7. Recommended policy

The primary metric name remains **Average Model Calls to Commit Execution Route**; its call-count meaning is unchanged.

For ROUTE_REQUIRED, a committed and QA-02 exact-conformant episode supplies a numeric observation. A no-route episode supplies no number, receives `ROUTE_REQUIRED_NOT_COMMITTED`, and fails the independent primary-comparability qualification. At the Alternative × profile stratum level, every predeclared REQUIRED episode must have a committed route, numeric observation, and QA-02 exact conformance before a qualified primary aggregate may be compared. Thus P06-A/C demonstrates why QA-02 PASS alone is insufficient.

ROUTE_FORBIDDEN is excluded from the primary population before execution. Correct non-commit is retained as `ROUTE_FORBIDDEN_NOT_COMMITTED`; a commit becomes `ROUTE_FORBIDDEN_BUT_COMMITTED`. Model calls before terminal resolution remain a secondary diagnostic.

ROUTE_OPTIONAL is never mixed with REQUIRED primary observations. Committed call counts and safe non-commit status remain in a separate diagnostic stratum. The final within-OPTIONAL comparison rule is deferred because one anti-gaming scenario is insufficient to justify an estimand.

The resource count and correctness qualification remain separate fields. No wrong/no-route numeric penalty is introduced.

## 8. Valid Official campaign diagnostic

`official-1789090497263081000` was read-only re-derived to a temporary v3 diagnostic. Both Z and C have the same contract counts:

| Alternative | REQUIRED committed / not | OPTIONAL committed / not | FORBIDDEN committed / not |
| :---: | ---: | ---: | ---: |
| A | 5 / 2 | 0 / 1 | 0 / 2 |
| B | 4 / 3 | 0 / 1 | 0 / 2 |
| C | 5 / 2 | 0 / 1 | 0 / 2 |
| D | 7 / 0 | 0 / 1 | 0 / 2 |

All candidate primary qualifications fail: A/B/C have REQUIRED non-commits; D has REQUIRED QA-02 non-conformance. This is a policy sanity check, not a ranking or reinterpretation of the Official result.

## 9. Decisions

| Item | Candidate | Status | Rationale | Remaining risk |
| --- | --- | :---: | --- | --- |
| Route-contract taxonomy | REQUIRED / OPTIONAL / FORBIDDEN as scenario contract | FREEZE | Product-semantic and independent of runtime outcome | Correct compound boundary must remain explicit |
| Arbitrary no-route penalty | Fixed, max+1, or other invented call value | REJECT | Invalid resource unit and correctness mixing | None unless authoritative measured cost creates a new metric |
| REQUIRED no-route | null + diagnostic + group comparability failure | FREEZE | Prevents cheap failure from becoming efficient | Final aggregation still unresolved |
| OPTIONAL route policy | Separate stratum | DEFER | Separation is sound; one scenario cannot establish a comparison estimand | Add representative OPTIONAL cases or authoritative use semantics |
| FORBIDDEN route policy | Pre-exclude primary; retain diagnostic; committed = violation | FREEZE | Route-commit metric is undefined for correct no-route behavior | Secondary diagnostic thresholds remain non-scoring |
| QA-04 overall vs macro | Overall or equal-class macro | DEFER | Complete corpus remains class-imbalanced and no usage frequency is authoritative | Resolve after rotated multi-cycle calibration |

## 10. Versioning and readiness

| Artifact | Old | New |
| --- | --- | --- |
| Corpus | DP00-RUNTIME-PILOT-V0 v0.1 | same identity, v0.2 |
| Scenarios | P01–P10 | P01–P12 |
| Analysis | dp00-analysis-v2 | dp00-analysis-v3 |
| Path-aware evidence | dp00-path-aware-evidence-v1 | v1 preserved; v2 for corpus v0.2 |
| QA-04 policy | none | qa04-route-contract-policy-v1 |
| Raw/event/model-call/provenance schemas | unchanged | unchanged |

R1–R10 corpus coverage, architecture neutrality, raw reconstructability, and path-aware evidence completeness pass. QA-04 route-contract semantics and gaming resistance pass with OPTIONAL and aggregate selection deferred. `benchmark-v1`, `aggregation-v1`, `scoring-v1`, and `gate-v1` remain NOT READY because repetition/order, instrumentation, latency-profile calibration, QA-02 gate, OPTIONAL handling, and aggregation remain unresolved.

The next calibration step is Session 6.2: rotated multi-cycle execution paired with CAPTURE/MINIMAL instrumentation and run-order/position sensitivity. No such experiment is run here.
