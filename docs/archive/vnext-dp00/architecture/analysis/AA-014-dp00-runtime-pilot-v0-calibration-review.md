# AA-014 — DP-00 Runtime Pilot v0 Calibration Review

## 1. Purpose and Scope

This record reviews measurement-system calibration after DP-00 Runtime Pilot v0. It does not select an A/B/C/D winner, produce a 0–5 architecture score, change an architecture, scenario, Oracle, eligibility rule, raw-schema meaning, or approved requirement baseline. Insufficient evidence is intentionally classified `DEFER`.

The evidence chain is:

```text
Official Pilot Evidence
        ↓
Calibration Analysis
        ↓
Metric / Aggregation Sensitivity
        ↓
Threats to Validity
        ↓
Freeze / Defer Decision Table
        ↓
Final Benchmark Readiness
        ↓
Report-ready Architecture Evidence
```

Machine-readable source: `results/reports/pilot-v0/calibration-review-data.json`. Unless stated otherwise, all measured values below come from committed campaign `official-1789090497263081000`, re-derived from the same immutable raw evidence by `dp00-analysis-v2`.

## 2. Experimental Configuration

| Item | Official Pilot configuration |
| --- | --- |
| Source SHA | `573ac6894bf7e8fba66580bf1fb771ececfe76b2` |
| Campaign ID | `official-1789090497263081000` |
| Analysis | original `dp00-analysis-v1`; deterministic re-derivation `dp00-analysis-v2` |
| Corpus/version | `DP00-RUNTIME-PILOT-V0` / `v0.1` |
| Rust / Cargo | `rustc 1.94.0 (4a4ef493e 2026-03-02)` / `cargo 1.94.0 (85eff7c80 2026-01-15)` |
| Target / build | `aarch64-apple-darwin` / `qualification-or-release` |
| Qualification runtime | Tokio `1.53.1`; `tokio-current-thread-v0` |
| Warm-up / measured repetitions | 1 / 1 |
| Profile Z | `pilot-z-v0`; model/agent/tool delay = 0/0/0 µs; minimal deterministic baseline |
| Profile C | `pilot-c-provisional-v0`; model/agent/tool delay = 1000/1000/1000 µs; provisional calibration parameter |
| Counterbalance rule | `DETERMINISTIC_CYCLIC_V1`; observed single cycle A0/B1/C2/D3 |
| Instrumentation | `CAPTURE` only |
| Schemas | campaign `dp00-pilot-campaign-provenance-v1`; run `dp00-pilot-provenance-v2`; event `canonical-event-v2`; call `model-call-v1` |
| Coverage manifest | `dp00-path-aware-evidence-v1` |
| Base prototype | tag `dp00-base-prototype-v1` → `b63004523a52960e8815213ad54f473ecc26a3e0` |

Architecture Qualification is a controlled Rust replay/stub experiment. Real-stack Validation uses actual models, agents, device and network. Profile C is not production latency, and neither Pilot profile supports a production-PC latency claim.

## 3. Provenance

| Artifact | Identity / disposition |
| --- | --- |
| Valid raw | `results/raw/pilot-v0/official-1789090497263081000/**`; immutable Official identity retained |
| Original valid derived | `results/derived/pilot-v0/official-1789090497263081000/analysis-summary.json`; v1 preserved, SHA-256 `547bb22595c134698263df551a00a2aca5281af7a5ca641a907c6864f0b8c7e1` |
| Deterministic valid derived | `results/derived/pilot-v0/official-1789090497263081000/dp00-analysis-v2/analysis-summary.json`; same raw, serialization-only v2, SHA-256 `0b9693850470e32f8e85b60883d7b0523c200f024a314ddad429052fd7bc8385` |
| Invalidated evidence | campaign `official-1789087890289040000`; forensic raw/derived preserved; never promoted |
| Report data | `results/reports/pilot-v0/calibration-review-data.json`; reproducible from the two summaries and valid raw provenance |

### Invalidated → valid methodology timeline

| Stage | Evidence / action | Methodological meaning |
| ---: | --- | --- |
| 1 | Official attempt #1: `official-1789087890289040000` | Execution completed, but evidence completeness still had to pass |
| 2 | `MISSING_ACTUAL_EVIDENCE = 16` | Campaign invalidated rather than patched or scored |
| 3 | Raw and derived bytes inventoried and preserved | Failed evidence remained auditable |
| 4 | Root cause: `EVALUATOR_PREDICATE_GAP` | Raw emission, Oracle and architecture were not rewritten |
| 5 | 36 constraints × 4 alternatives path-aware gate | 144 topology-aware decidability cells established |
| 6 | `dp00-analysis-v1` | Explicit terminal non-achievement and closed-world forbidden absence made decidable |
| 7 | Official attempt #2: `official-1789090497263081000` | New clean Official campaign, not mutation of attempt #1 |
| 8 | MISSING = 0; UNEVALUABLE = 0; 144/144 evaluated | Valid Official Pilot evidence |
| 9 | `dp00-analysis-v2` re-derivation | Same Official raw and metric semantics; canonical diagnostic serialization |

## 4. Validity Gates

| Gate | Committed evidence | Result |
| --- | ---: | :---: |
| Campaign identity | `official-1789090497263081000` | PASS |
| Source identity | `573ac6894bf7e8fba66580bf1fb771ececfe76b2` | PASS |
| Analysis validation | no errors or warnings | PASS |
| Measured episodes | 80 | PASS |
| Profile coverage | Z 40; C 40 | PASS |
| Missing / duplicate episodes | 0 / 0 | PASS |
| `MISSING_ACTUAL_EVIDENCE` / `UNEVALUABLE` | 0 / 0 | PASS |
| Path-aware coverage | 144 expected; 144 evaluated; 0 missing; 0 unevaluable | PASS |
| Profile semantic status invariance | true | PASS |

The 144 cells contain 129 PASS and 15 FAIL correctness cells. Those counts demonstrate decidability, not architecture ranking.

### Derived byte determinism and semantic identity

The byte instability was caused by set-like `derived_predicates` being converted from a `frozenset` to diagnostic JSON lists without canonical sorting. `sort_keys=True` stabilized object keys but could not stabilize list element order across Python hash seeds. `dp00-analysis-v2` sorts only those set-like Actual diagnostic values before stable JSON serialization.

| Gate | Result |
| --- | --- |
| QA-01 all numeric metrics v1 = v2 | PASS |
| QA-02 AECR, episode verdicts, constraint verdicts and dimension diagnostics v1 = v2 | PASS |
| QA-04 per-episode calls, overall, macro and no-route v1 = v2 | PASS |
| MISSING / UNEVALUABLE v1 = v2 = 0 | PASS |
| Independent run 1 SHA-256 | `0b9693850470e32f8e85b60883d7b0523c200f024a314ddad429052fd7bc8385` |
| Independent run 2 SHA-256 | `0b9693850470e32f8e85b60883d7b0523c200f024a314ddad429052fd7bc8385` |
| Byte-identical | PASS |

No metric formula, eligibility, constraint verdict, Actual/Oracle meaning, percentile value, AECR, QA-04 inclusion, or no-route meaning changed.

## 5. QA-01 Results

All values are milliseconds. Each Profile × Alternative group has 3 eligible, 3 successful and 0 failed episodes.

| Profile | Alt | Eligible / success / failure | Min | Mean | Max | NR p50 | NR p95 | NR p99 | Linear p50 | Linear p95 | Linear p99 |
| --- | :---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| C | A | 3 / 3 / 0 | 4.570083 | 4.599070 | 4.618459 | 4.608667 | 4.618459 | 4.618459 | 4.608667 | 4.617480 | 4.618263 |
| C | B | 3 / 3 / 0 | 3.064458 | 3.095875 | 3.113042 | 3.110125 | 3.113042 | 3.113042 | 3.110125 | 3.112750 | 3.112984 |
| C | C | 3 / 3 / 0 | 4.611208 | 4.630597 | 4.654042 | 4.626541 | 4.654042 | 4.654042 | 4.626541 | 4.651292 | 4.653492 |
| C | D | 3 / 3 / 0 | 4.435541 | 4.556278 | 4.632750 | 4.600542 | 4.632750 | 4.632750 | 4.600542 | 4.629529 | 4.632106 |
| Z | A | 3 / 3 / 0 | 0.006541 | 0.007264 | 0.008458 | 0.006792 | 0.008458 | 0.008458 | 0.006792 | 0.008291 | 0.008425 |
| Z | B | 3 / 3 / 0 | 0.007042 | 0.007930 | 0.008791 | 0.007958 | 0.008791 | 0.008791 | 0.007958 | 0.008708 | 0.008774 |
| Z | C | 3 / 3 / 0 | 0.006584 | 0.007056 | 0.007958 | 0.006625 | 0.007958 | 0.007958 | 0.006625 | 0.007825 | 0.007931 |
| Z | D | 3 / 3 / 0 | 0.006958 | 0.007611 | 0.008791 | 0.007083 | 0.008791 | 0.008791 | 0.007083 | 0.008620 | 0.008757 |

## 6. QA-02 Results

QA-02 is a scored architectural driver; Mandatory qualification gates are separate non-compensable eligibility conditions. The table reports evidence and does not define a minimum AECR gate.

| Profile | Alt | AECR | Constraint PASS / FAIL | MISSING / UNEVALUABLE | Non-conformant scenarios |
| --- | :---: | ---: | ---: | ---: | --- |
| C | A | 9/10 (90%) | 34 / 2 | 0 / 0 | P04 |
| C | B | 7/10 (70%) | 30 / 6 | 0 / 0 | P04, P06, P07 |
| C | C | 9/10 (90%) | 34 / 2 | 0 / 0 | P04 |
| C | D | 8/10 (80%) | 31 / 5 | 0 / 0 | P04, P07 |
| Z | A | 9/10 (90%) | 34 / 2 | 0 / 0 | P04 |
| Z | B | 7/10 (70%) | 30 / 6 | 0 / 0 | P04, P06, P07 |
| Z | C | 9/10 (90%) | 34 / 2 | 0 / 0 | P04 |
| Z | D | 8/10 (80%) | 31 / 5 | 0 / 0 | P04, P07 |

### P04 / P06 / P07 non-conformance matrix

The same semantic outcomes occur in Z and C.

| Scenario | A | B | C | D | Raw-trace interpretation |
| --- | --- | --- | --- | --- | --- |
| P04 ambiguous document | FAIL: required open and allowed route not achieved | FAIL: required referent/open and allowed route not achieved | FAIL: required open and allowed route not achieved | FAIL: committed `LOCAL_DIRECT:VIA_FAST`, outside allowed file route | A/B/C end in authoritative `INVALID_ROUTE`; D has positive committed-route evidence. These are Base behavior correctness outcomes, not instrumentation errors or Oracle fallback. |
| P06 clarification | PASS | FAIL: required post-clarification referent absent; `INVALID_ROUTE` terminal | PASS | PASS | B's model candidate is not promoted to Actual referent evidence. |
| P07 domain planning | PASS | FAIL: required planned effect and allowed route not achieved; `INVALID_ROUTE` terminal | PASS | FAIL: local Fast route/effect positively violates route, owner and local-effect constraints and lacks accepted domain planning | D demonstrates false-fast behavior; B demonstrates terminal non-achievement. Both remain correctness failures after the evidence gate passed. |

### P09 anti-gaming evidence

| Evidence | Observation |
| --- | --- |
| Input behavior | Wrong-but-structurally-valid `MailAgent` candidate proposed in every alternative/profile |
| Route | No `RouteCommitted` evidence |
| Execution/effect | No MailAgent invocation and no MailAgent Wi-Fi effect |
| Terminal | Safe `INVALID_ROUTE` failure |
| Semantics | `MAIL_AGENT_REJECTED_OR_NOT_COMMITTED` and `SAFE_FAILURE:WRONG_CANDIDATE` |
| Exact conformance | PASS for all A/B/C/D in both profiles |

The absence of a route commit is not by itself the reason for PASS. PASS requires the semantically wrong candidate to be rejected, no prohibited MailAgent execution/effect, a complete captured trace, and the accepted safe-failure outcome.

## 7. QA-04 Results

All ten episodes per Profile × Alternative are QA-04 eligible under the Pilot scenario contract. Numeric means include only route-committed episodes; no-route remains null and separately reported.

| Profile | Alt | Route-committed n | Overall mean | Class macro | Overall − macro abs / rel | No-route n / rate | No-route by expectation: REQUIRED; OPTIONAL; FORBIDDEN |
| --- | :---: | ---: | ---: | ---: | ---: | ---: | --- |
| C | A | 5 | 1.8 | 1.8 | 0 / 0% | 5 / 50% | 2/7; 1/1; 2/2 |
| C | B | 4 | 1.0 | 1.0 | 0 / 0% | 6 / 60% | 3/7; 1/1; 2/2 |
| C | C | 5 | 1.8 | 1.8 | 0 / 0% | 5 / 50% | 2/7; 1/1; 2/2 |
| C | D | 7 | 2.0 | 2.0 | 0 / 0% | 3 / 30% | 0/7; 1/1; 2/2 |
| Z | A | 5 | 1.8 | 1.8 | 0 / 0% | 5 / 50% | 2/7; 1/1; 2/2 |
| Z | B | 4 | 1.0 | 1.0 | 0 / 0% | 6 / 60% | 3/7; 1/1; 2/2 |
| Z | C | 5 | 1.8 | 1.8 | 0 / 0% | 5 / 50% | 2/7; 1/1; 2/2 |
| Z | D | 7 | 2.0 | 2.0 | 0 / 0% | 3 / 30% | 0/7; 1/1; 2/2 |

### Per-class composition (identical in Z and C)

| Alt | Included classes: `class n × mean`; each population share; contribution |
| :---: | --- |
| A | R1 1×2, R2 1×2, R3 1×2, R5 1×1, R7 1×2; each share 20%; contributions 0.4/0.4/0.4/0.2/0.4 |
| B | R1 1×1, R2 1×1, R3 1×1, R5 1×1; each share 25%; each contribution 0.25 |
| C | R1 1×2, R2 1×2, R3 1×2, R5 1×1, R7 1×2; each share 20%; contributions 0.4/0.4/0.4/0.2/0.4 |
| D | R1 1×2, R2 1×2, R3 1×2, R4 1×2, R5 1×1, R6 1×3, R7 1×2; each share 14.286%; contributions 0.286/0.286/0.286/0.286/0.143/0.429/0.286 |

Overall equals macro only because each included class has one observation. This Pilot cannot establish general equivalence; future duplicate or imbalanced classes will make the estimands differ.

## 8. Calibration Sensitivity

### QA-01 percentile estimator

| Profile / Alt | p95 absolute delta (ms) | Relative delta |
| --- | ---: | ---: |
| C/A | 0.000979 | 0.0212% |
| C/B | 0.000292 | 0.0094% |
| C/C | 0.002750 | 0.0591% |
| C/D | 0.003221 | 0.0695% |
| Z/A | 0.000167 | 1.9697% |
| Z/B | 0.000083 | 0.9476% |
| Z/C | 0.000133 | 1.6750% |
| Z/D | 0.000171 | 1.9429% |

| Criterion | Nearest-rank | Linear `(n-1)` |
| --- | --- | --- |
| Definition transparency | Selects an observed order statistic; simple rank rule | Requires interpolation convention and produces synthetic values |
| Small-N behavior | Conservative/discontinuous tail; p95 = maximum at n=3 | Smoother but precision is not evidence of tail stability |
| Reportability / reproducibility | Strong; observed value and rank are auditable | Strong only with the exact interpolation convention named |
| Architecture neutrality | Same empirical rule for all alternatives | Same formula for all alternatives |
| Future stability | Stable definition as n grows; values step with observations | Stable definition but may differ across common library defaults |

Recommendation: `FREEZE` nearest-rank for primary FTOL p95, because it is an observed, transparent empirical tail statistic and avoids implying resolution unsupported by n=3. The decision is independent of observed A/B/C/D ordering. Linear `(n-1)` remains a reported sensitivity diagnostic. More repetitions are still required to stabilize the estimate.

### QA-01 repetition and profile sensitivity

Measured repetition is 1. A single cycle fixes A/B/C/D at positions 0/1/2/3, so the Pilot cannot separate architecture, position drift, time drift or carryover. Exact final repetition count is `DEFER` pending multiple measured cycles with cyclic position rotation on the same frozen corpus and analysis.

Profile Z is the software/architecture/instrumentation baseline. Profile C adds provisional controlled dependency cost; it is neither production-representative nor safe to pool with Z. The no-pooling and separate-view rule is methodologically sound, but the exact final latency profile is `DEFER` until dependency non-dominance is calibrated.

### QA-02 gate and corpus sensitivity

Repository sources consistently state that the numeric minimum acceptable AECR remains TBD; no approved product, safety, reliability or business threshold justifies 70%, 80%, 90%, or another number. A threshold inferred from this Pilot distribution would be post-hoc and is rejected. Numeric gate freeze is `DEFER` pending an owner requirement independent of alternative results.

R1–R7 and R10 are covered; R8 Concurrent Task / Result and R9 Compound Request are explicitly `NOT_COVERED`. Their absence can underrepresent task/result binding, decomposition and multi-route behavior. Corpus/constraint balance is `DEFER`; no R8/R9 scenarios are invented in this session.

### QA-04 aggregation and no-route sensitivity

Without usage-frequency evidence, overall episode mean embeds corpus composition as a weight. Scenario-class macro-average gives each represented class equal weight, but the Pilot's one-observation-per-included-class structure cannot distinguish the candidates, and R8/R9 are absent. Primary aggregation is therefore `DEFER`; arbitrary usage weights are rejected.

No-route candidates:

| Candidate | Methodological assessment | Disposition |
| --- | --- | :---: |
| A — committed-only numeric + separate null diagnostic | Preserves metric meaning, but alone can hide cheap failure from the resource population | REJECT as a stand-alone selection rule |
| B — arbitrary penalty | Mixes correctness with resource use through an ungrounded constant | REJECT |
| C — correctness-qualified resource metric | Keeps call-count semantics and prevents wrong/no-route behavior from winning through a smaller numeric population | Preferred principle |
| D — REQUIRED/OPTIONAL/FORBIDDEN contract eligibility | Architecture-neutral when fixed before results; Pilot has one OPTIONAL anti-gaming case, insufficient for a general OPTIONAL population rule | Preferred structure with OPTIONAL unresolved |

The final policy is `DEFER`: freeze Candidate C together with contract-based Candidate D only after the REQUIRED/FORBIDDEN/OPTIONAL rules and non-compensable qualification relationship are fully specified. The Pilot includes seven REQUIRED scenarios, P08/P10 as FORBIDDEN, and P09 as OPTIONAL. Consequently, two of every group's no-route episodes are contract-required safe non-commit and P09 is the anti-gaming safe rejection; the overall A/C 50%, B 60%, D 30% rates cannot be read as failure rates. The remaining REQUIRED no-route differences (A/C 2/7, B 3/7, D 0/7) show why contract-aware policy matters, but are not a reason to favor a policy or architecture.

P06-A/C are especially important calibration evidence: they are QA-02 exact-conformant under the present constraint manifest yet have no route commit even though QA-04 marks the scenario `REQUIRED`. Therefore “QA-02 conformant” alone is not a sufficient QA-04 eligibility condition. The frozen policy must explicitly reconcile route-requirement satisfaction with QA-02 correctness instead of assuming the two are interchangeable.

## 9. Instrumentation / Order Diagnostics

| Profile | Episodes | Event count min / mean / max | Capture append cost ns min / mean / max |
| :---: | ---: | ---: | ---: |
| Z | 40 | 4 / 9.95 / 19 | 42 / 266.825 / 461 |
| C | 40 | 4 / 9.95 / 19 | 374 / 1188.650 / 2087 |

Only `CAPTURE` ran. Append-cost telemetry is not a paired overhead estimate, and no acceptance threshold exists. Instrumentation mode is `DEFER — CAPTURE/MINIMAL paired calibration required` on identical replay inputs.

The counterbalance mechanism exists, but the Official population experienced only A0/B1/C2/D3. It cannot estimate position sensitivity or time drift and cannot claim full counterbalance. Final run-order design is `DEFER` pending multiple cycles that rotate alternatives through every position.

## 10. Threats to Validity

| Threat | Pilot evidence | Impact | Mitigation before final |
| --- | --- | --- | --- |
| Small-N QA-01 | n=3 per Profile × Alternative | unstable p95 and estimator sensitivity | add measured observations before scoring |
| Repetition=1 | one measured cycle | run-to-run variance unknown | multiple measured cycles |
| Position confounding | A0/B1/C2/D3 only | architecture and order/time effects inseparable | rotate cyclic positions; analyze position sensitivity |
| Profile C external validity | synthetic 1000/1000/1000 µs | not production/device representative | retain separate views; add separate real-stack track |
| CAPTURE only | no MINIMAL pair | timing contamination unknown | paired CAPTURE/MINIMAL calibration |
| R8/R9 absent | explicit `NOT_COVERED` | task/result and compound-path bias; corpus balance incomplete | add and validate before `benchmark-v1` |
| QA-02 threshold absent | repository says TBD; no owner requirement | eligibility boundary cannot be defended | obtain product/safety/reliability requirement |
| QA-04 no-route null population | 30–60% no-route | cheap-failure/gaming risk | freeze correctness-qualified contract policy |
| Overall = macro in Pilot | one observation per included class | false equivalence risk | freeze estimand semantics, not observed equality |
| Profile pooling | Z and C measure different controlled conditions | mixed population loses interpretation | prohibit pooling; report separate views |

## 11. Calibration Decision Table

| Decision Item | Current Candidate(s) | Pilot Evidence | Methodological Principle | Recommendation | Status | Required Follow-up | Freeze Version if applicable |
| --- | --- | --- | --- | --- | :---: | --- | --- |
| QA-01 percentile estimator | nearest-rank; linear `(n-1)` | n=3; deltas reported above | transparent observed tail; no synthetic precision | nearest-rank primary, linear sensitivity | FREEZE | retain both outputs; increase n | percentile rule in future `scoring-v1` |
| QA-01 repetition count | 1; exact future count unknown | one fixed-position cycle | estimate variance/order before scoring | additional calibration | DEFER | multiple cycles, rotated positions, stopping rationale | — |
| QA-01 latency profile | Z; C; separate Z+C | large designed profile separation; C provisional | isolate architecture; avoid production claim | separate views/no pooling, exact final profile unresolved | DEFER | calibrate dependency non-dominance; real-stack separate | — |
| QA-02 minimum correctness gate | numeric AECR threshold | observed 70/80/90%, no external threshold | gate must precede and be independent of results | do not infer threshold | DEFER | owner requirement | future `gate-v1` |
| QA-02 corpus/constraint balance | R1–R7,R10; R8/R9 absent | 144-cell decidability for present constraints | representative topology-neutral coverage | complete missing classes | DEFER | add R8/R9 before freeze | future `benchmark-v1` |
| QA-04 aggregation | overall; class macro | equal only under tiny balanced included classes | estimand chosen before results | do not infer equivalence | DEFER | evaluate completed corpus composition and usage assumption | future `aggregation-v1` |
| QA-04 no-route treatment | A/B/C/D candidates | 30–60% overall; REQUIRED 0–3/7; OPTIONAL 1/1; FORBIDDEN 2/2 | contract-aware gaming resistance without invented penalty | C + contract-based D direction; reject A-alone/B | DEFER | specify all three expectation populations and gate relationship; regression tests | future `aggregation-v1`/`gate-v1` |
| Instrumentation mode | CAPTURE; MINIMAL | CAPTURE only; append telemetry only | paired causal comparison | paired calibration | DEFER | same replay CAPTURE/MINIMAL | future `benchmark-v1` |
| Run-order design | one cyclic cycle; rotated cycles | fixed A0/B1/C2/D3 | remove order/time confounding | multiple fully rotated cycles | DEFER | position/drift analysis | future `benchmark-v1` |
| R8/R9 coverage | absent; add before final | explicit coverage gap | workload completeness before scoring | complete and path-gate | DEFER | design in later session without result fitting | future `benchmark-v1` |

`REJECT` applies to the arbitrary QA-04 penalty, stand-alone committed-only interpretation, post-hoc QA-02 thresholds, pooled Z+C population and arbitrary usage weighting. It does not reject the underlying QA metrics.

## 12. Final Benchmark Readiness

| Version candidate | Readiness | Blocker |
| --- | :---: | --- |
| `benchmark-v1` | NOT READY | R8/R9 absent; repetitions/order and CAPTURE/MINIMAL calibration unresolved; exact latency profile unresolved |
| `aggregation-v1` | NOT READY | QA-04 primary aggregate and no-route/OPTIONAL policy unresolved |
| `scoring-v1` | NOT READY | no pre-result 0–5 mappings/thresholds; no final calibrated populations |
| `gate-v1` | NOT READY | numeric QA-02 requirement absent; mandatory gates must remain separate and versioned |

Overall state: **NOT_READY** for Final A/B/C/D Evaluation. This is independent of observed architecture performance and is a successful calibration finding, not a Pilot evidence failure.

### Pre-experiment hypotheses versus observed Pilot behavior

| Pre-experiment expectation | Observed Pilot evidence | Calibration interpretation only |
| --- | --- | --- |
| Stage/route structure may affect QA-01 | Z and C exhibit measurable Profile × Alternative FTOL variation | sample/order/profile limitations prevent final scoring inference |
| B's co-located routing may reduce QA-04 calls | committed-only B rows show fewer calls, alongside the highest no-route rate | supports the need to interpret resource use with correctness; not a winner claim |
| Explicit validation/state boundaries may affect QA-02 | P04/P06/P07 discriminate semantic handling | preserves expected sensitivity; does not make hypothesis correctness a selection rule |
| Fast-path eligibility errors are a QA-02 risk | P07-D records prohibited local route/effect | confirms the scenario discriminates false-fast behavior; not a global architecture verdict |
| Per-turn selection may add QA-04 work | committed D rows contain a three-call R6 class | one Pilot observation is diagnostic, not a stable workload estimate |

## 13. Required Follow-up Before `benchmark-v1`

One roadmap, in order:

```text
1. Add and independently review R8/R9 coverage without fitting to Pilot outcomes.
2. Specify QA-04 REQUIRED/FORBIDDEN/OPTIONAL no-route eligibility and its
   correctness-qualified, non-compensable gate relationship.
3. Run targeted calibration only: multiple fully rotated measured cycles and
   paired CAPTURE/MINIMAL executions on the frozen corpus/analysis.
4. Calibrate the controlled dependency profile for equality/non-dominance;
   keep real-stack validation separate.
5. Obtain an owner-backed QA-02 minimum requirement or explicitly omit the
   numeric correctness gate; never infer it from 70/80/90 Pilot values.
6. Freeze benchmark-v1 and aggregation-v1, then freeze gate-v1/scoring-v1
   before viewing Final Evaluation results.
7. Only then run Final A/B/C/D Evaluation.
```

Final Session 5 verdict:

```text
Official Pilot Evidence Integrity          PASS
Derived Byte Determinism                   PASS
Metric Semantic Preservation               PASS
QA-01 Calibration Review                   PASS WITH DEFER
QA-02 Calibration Review                   PASS WITH DEFER
QA-04 Calibration Review                   PASS WITH DEFER
Threats-to-Validity Review                 PASS
Report-ready Evidence Completeness         PASS
benchmark-v1 Freeze Readiness              FAIL
aggregation-v1 Freeze Readiness            FAIL
scoring-v1 Freeze Readiness                FAIL
gate-v1 Freeze Readiness                   FAIL
Final Evaluation Readiness                 PASS WITH FOLLOW-UP
```
