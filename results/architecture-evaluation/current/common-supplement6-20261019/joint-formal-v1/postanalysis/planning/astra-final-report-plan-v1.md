# Final report and semantic review plan v1

2026-10-19. Judgment and source review: GPT-6 Astra Ultra.

This is a postlaunch reporting plan derived from bounded reads of the frozen contracts and analyzers and current presentation document section 11. It changes no experiment, endpoint, oracle, denominator, frozen source, contract or approval. It does not certify unfinished results. No analyzer, test, candidate, parser, API call or bulk extraction was executed for this plan.

## 1. Separate populations and shared control

Produce separate common9 and supplement6 tables, each containing all seven configurations: `AAAA`, `BAAA`, `CAAA`, `ABAA`, `AABA`, `AAAB`, `AAAC`.

Use the same AAAA observations in DP1 comparisons with BAAA/CAAA, DP2 with ABAA, DP3 with AABA, and DP4 with AAAB/AAAC. Do not count AAAA four times as independent observations or expenditure. Preserve fixed episode and repetition identities and the frozen execution order. Do not pool historical campaigns, probes, QA04 regressions, common9 or supplement6 into one QA average.

| Field | Common9 per configuration | Supplement6 per configuration |
| --- | --- | --- |
| Execution coverage | observed/planned45 episodes; certified, uncertified and unexecuted | observed/planned18 episodes; same states |
| QA01 | certified observed/215; observed mean; missing and unresolved | certified observed/36; same formula |
| QA02 | known-correct/75; adopted incorrect, not adopted, technical unknown and unexecuted | known-correct/18; same distinctions |
| QA03 | certified endpoint observed/115; observed VIA mean | certified endpoint observed/24; same formula |
| QA04 | 11/6 = 1.833; C01–C06 counts1,1,3,3,1,2 | reference to the same42 change experiments |
| QA05 | all known cost; mean of five9-episode bundles; unknown calls/reservations | all known cost; mean of three6-episode bundles; unknown calls/reservations |
| QA06 | highest same-sample concurrent RSS across45 episodes; completeness | highest same-sample concurrent RSS across18 episodes; completeness |

The same QA04 result applies to all seven configurations. Link its final count report, exact module sets and42-copy/630-regression acceptance. Do not inherit the prior1.667 generation, turn630 regressions into model observations, or repeat the42 changes as independent supplement measurements. QA04 memory diagnostics stay separate from formal QA06.

## 2. Endpoint and interpretation rules

Name QA01 **first substantive response host-callback latency** or `첫 구체적 응답의 실제 host callback 지연`. Do not name it first correct answer latency. Frozen19 section5 separates callback timing from referent accuracy, body meaning, Task association and approval/handoff correctness. Publication and clock certification do not establish all semantic conditions.

Keep QA01 timing and its original denominator unchanged after semantic review. A later semantic FAIL must appear alongside the original timing, not erase it, replace its timestamp, or create a new first-correct-answer metric. Generic acknowledgements and other exclusions already enforced by the frozen endpoint evaluator remain enforced.

For QA01/03, use the frozen certified observation predicate. Show raw technically uncertified numeric observations separately. No observations means null/NOT_OBSERVED, not zero seconds; a genuine measured zero is different. Means over a successful subset do not establish whole-population speed superiority. Certified same-obligation paired intersections are supplementary only.

Make obligation status counts disjoint before summing to the planned denominator: unexecuted identity, technically uncertified obligation, certified unresolved endpoint, certified missing endpoint, or certified observed endpoint. Preserve detailed reasons underneath. Do not add overlapping raw unresolved and technical counts as if they partitioned the denominator.

For QA02, retain known-correct/planned even when incomplete, explicitly labeled a lower bound where appropriate. Split a wrong adopted set from ordinary non-adoption. Technical unjudgeability and never-executed rows are different states. No new expression or subset denominator is introduced.

For QA05, disclose fixed comparison rates of input USD0.40/M token and output USD1.60/M token, not a verified invoice. Include actual role/stage cost and token usage, failed/discarded/unused work where evidenced, late usage, unknown calls and reservations. Report call rate classes from preserved price records if they differ; do not silently recompute with a new price. A reservation is not settled expenditure. Reconcile campaign totals with the global ledger and prior spending separately. Local API charge0 does not mean local compute cost0; local monetary compute cost remains NOT_MEASURED. Do not compare9-episode and6-episode bundle totals as a simple savings ratio.

For QA06, use the maximum actual same-sample sum, never the sum of role maxima or episode peaks. State the byte conversion used for MB/MiB, nominal50ms sampling, sample gaps, observed episode coverage, invalid samples, confirmed exit boundaries, startup-zero observations and sampler failures. Incomplete values are observed lower-bound diagnostics. This is not the continuous-time maximum, remote model memory or full-product memory.

## 3. Prediction decisions for all24 DP/QA cells

Retain the preregistered conditions and counterconditions from frozen19/20 and current17 section11. First show the full fixed-population values and failures. Then connect actual mechanism evidence and relevant condition-level observations to one of the existing judgments: conditional support, opposite observation, offsetting costs, condition absent, mechanism not activated, observed tie, or no prior ranking/unresolved.

Do not retrospectively call a numeric ranking a successful prediction where no strict ranking was predicted. Do not select only activated or successful episodes. Small differences and three/five bundle repeats do not establish population-wide statistical superiority.

Supplement DP2 requires correctly associated U01/U02 work, actual request-owned model interval overlap, functional outcome and the relevant QA01/03 observations. Host operation envelopes are not exact worker lifetime evidence. Exact initial free K2 capacity and provider internal compute concurrency remain unresolved in this ABI. Non-overlap with unknown capacity is not falsification of an uncongested-capacity hypothesis.

Supplement DP3 reports dense, sparse and native conditions separately, including parser/spawn/producer/binder/expansion paths, exact-set outcomes, cost and memory. Candidate sufficiency is an outcome condition, not an inclusion filter. A native skip needs the actual native path and absence of helper spawn and parse dispatch. The dense/sparse contrast is the registered size-plus-spacing contrast. DP4 still receives all six QAs on the supplement, but absent intended handoff conditions cannot establish a handoff benefit.

DP1 reuse must be supported by actual original-read/input/recovery evidence; preparation does not imply the common range-selection semantic call disappeared. DP4 amortization requires repeated consumption of the same record and source revision, not consumption of a newly regenerated record. Work after first handling publication cannot be credited to that earlier QA01 endpoint.

## 4. Six concrete reporting gaps in existing output

1. **Joint paired and condition tables:** `common_supplement6_20261019/analyze_joint.py:39` returns separate summaries, per-trace results, activation records and missing identities, but no completed four-DP paired table or condition/repetition summary. Add those in a separate postanalysis output while retaining the primary summaries.
2. **Technical validity of existing paired diagnostics:** `common9_poc_20261019/analyze.py:28–47`, imported by `evaluation_v3/analyze.py`, computes paired timing differences whenever both numbers exist. It does not check `primary_measurement_eligibility`. Final certified paired explanation must require both sides eligible and retain the old raw diagnostic separately. Build the pairing worklist from planned identities so missing sides remain visible.
3. **Semantic judgments remain pending:** common9 `evaluate.py:125–185` generates NEEDS_REVIEW body/Task/handoff ledgers and literal preservation checks, not completed semantic PASS judgments. Supplement `evaluate.py:23–27` explicitly marks required facts MANUAL_REVIEW_REQUIRED_NOT_QA01_FILTER. Complete or honestly expose that review coverage using section5 below.
4. **Summary labels conflate distinct states:** common9 `evaluate.py:236–248` and v3 `evaluate.py:109–115` allow ordinary NOT_ADOPTED rows among incorrect judgments. Split them for interpretation. Common9 `evaluate.py:294–346` uses `calls=len(calls)`, including local helper records; this is not the actual cloud API dispatch count. Use actual dispatch records for that label and retain local records separately.
5. **Activation is not automatic hypothesis certification:** supplement `mechanism.py:45–79` reports observed overlap and local/native paths with explicit limits. Join them to technical certification and reviewed request/goal correspondence before drawing support. DP1/DP4 reuse, record-version continuity and repeated consumption do not have a completed mechanism summary here; connect their existing raw evidence prospectively in postanalysis rather than assume them from numeric QA differences.
6. **Final report linkage:** joint summary does not fully link final QA04 measurements/acceptance, aggregate role/rate-class cost details, condition-specific RSS, and global reservation-aware budget settlement. Add those report links and tables using existing results. Keep per-population QA values, prior spend and global budget reconciliation distinct.

These are reporting gaps, not a request to reopen the running experiment or edit frozen analyzers.

## 5. Semantic and functional review worklist

The existing evaluator diagnostics and trace/event references support bounded Astra review against the frozen source and goal contracts. Literal equality or callback publication alone is insufficient. External commercial Agent work and Voice behavior outside this PoC cannot be certified by these traces.

At full completion, the planned worklist has **1,757 response obligations**:1,505 common9 plus252 supplement6. It also has **973 goal/relation obligations**:805 plus168. Actual handoff/control events add review rows; their eventual count has not been assumed. The651 automatic referent-set obligations remain the existing QA02 measurements, not a new manual denominator.

Use a separate postanalysis worklist with stable keys:

`population, configuration, episode, rep, run_id, obligation_kind, obligation_id`

The planned identity exists even if a run or endpoint is missing. A never-executed row is UNEXECUTED/UNREVIEWED, not a fabricated incorrect model answer. An actually executed, technically complete observation window with no required output may be recorded as explicit functional FAIL_NO_OUTPUT. Technical incompleteness must not be silently converted to a semantic failure.

Each review bundle must include or point to:

| Group | Essential fields |
| --- | --- |
| Provenance | population/configuration/episode/rep/run ID; freeze, evaluator and oracle hash; raw trace path/hash; planned identity |
| Frozen expectation | expected response ID and content conditions; goal start and ordered members; completion conditions; exact source/oracle reference |
| Technical state | primary eligibility, deadline audit, execution completeness, rejection/cancellation and missing-source reasons |
| Observed output | QA01 selected callback ID/time and text; other actual displays with phase/revision; QA03 accepted terminal ID/time; no endpoint replacement |
| Adopted semantics | actual plan, purpose, route, Task/goal/revision, relation and correction scope, bindings, control/permission decisions |
| Source support | relevant actual source/revision/extent, read/authority evidence and bounded quotations needed for the judgment |
| Handoff/control | actual request and recipient/Task/execution ID, mandatory conditions and exclusions, permission context, receipts/results and rejection/cancellation evidence |
| Judgment | PASS/FAIL/UNREVIEWED by body, Task/revision, approval/authority and handoff dimension; explicit reason; supporting event/file references; reviewer and review version |

Preserve raw model request/response paths when a schema rejection or interpretation needs inspection. The helper must not construct a replacement oracle, normalize candidate answers into expected answers, call models or parsers, or rerun an episode. It may arrange already recorded evidence for review.

PASS requires evidence for all applicable frozen conditions in the reviewed dimension. FAIL requires a supported violation, with missing output distinguished from wrong content. Ambiguous or insufficient evidence remains UNREVIEWED. Literal transferred-field equality alone is only the existing preservation result, not semantic PASS. A later correct answer does not silently repair the semantic status of the first selected callback.

Publish coverage separately for response bodies, goal/Task relations, approval/authority and actual handoff events: planned/applicable, executed, reviewed PASS, reviewed FAIL, UNREVIEWED and unexecuted where applicable. Do not call partial semantic review complete or use a reviewed success subset as a new QA01 denominator.

## 6. Incremental closed-block preparation

The next helper plan may cover **one already closed seven-episode configuration block**, with no execution or extraction until separately directed. Use deterministic case/repetition order and retain all seven configurations, including failed rows. Keep enough preceding and following events to assess the same goal and current authority; do not detach a correction from its prior adopted state.

Bound each reading batch and leave remaining rows explicitly UNREVIEWED. Identical-looking prose does not justify inherited PASS across different Task, revision, permission or source state. Review may proceed incrementally on closed blocks without changing the runtime, frozen files or current measurement load. This plan itself authorizes no new bulk scan or heavy processing.

After quiescence, create immutable versioned final postanalysis outputs and link the final source/result hashes. Preserve previous diagnostic versions and all raw outcomes.

## Source references already inspected

Frozen source prefix: `results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/`.

- `docs/architecture/12-decisions/decision-packages/seven-problem-review/19-common9-evaluation-contract.md`, sections2,3,5–8.
- `docs/architecture/12-decisions/decision-packages/seven-problem-review/20-common-supplement-evaluation-contract.md`, sections2,3,6–9.
- `benchmark/architecture/common_supplement6_20261019/analyze_joint.py`, lines12–39.
- `benchmark/architecture/common_supplement6_20261019/evaluate.py`, lines23–64.
- `benchmark/architecture/common_supplement6_20261019/mechanism.py`, lines16–79.
- `benchmark/architecture/common9_poc_20261019/evaluation_v3/evaluate.py`, lines19–73 and91–120.
- `benchmark/architecture/common9_poc_20261019/evaluation_v3/analyze.py` and its preserved `benchmark/architecture/common9_poc_20261019/analyze.py` dependency, lines9–79.
- `benchmark/architecture/common9_poc_20261019/evaluate.py`, lines125–185,236–248,294–364 and390–430.
- Current `docs/architecture/12-decisions/decision-packages/seven-problem-review/17-presentation-results.md`, sections10.3 and11 through11.2, as read for this plan.

No additional source or result scan was performed to save this note.
