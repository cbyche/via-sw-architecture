# Quantitative final-report adapter specification v1

2026-10-19. Independent method/source judgment: GPT-6 Astra Ultra, `/root/method_astra_review`.

Status: **planning only**. Implement outside the frozen experiment, for example under `scripts/architecture/common9_postanalysis/`. Full execution follows campaign quiescence and separately authorized frozen analysis. This document authorizes no execution, test, model call, evaluator import, input change, score revision or replacement of original artifacts.

This specification supplements [final-report-plan-v1](astra-final-report-plan-v1.md), SHA256 `bb23355ba50776e1096ccb009e1fd8d170d46a9d55abff8fe8eae273b9234e41`. It makes the six reporting gaps there implementable without changing the frozen QA algorithms.

## 1. Inputs and evidence boundary

Use explicit paths supplied in an immutable input manifest. Never discover a preferred run by recursive search, choose the latest-looking directory, or mix phases, probes, historical campaigns or QA04 regression traces with formal observations.

| Input | Required binding and use |
| --- | --- |
| Formal `freeze.json` | SHA256 `0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd`; all441 ordered jobs, population, configuration, episode, rep, block ID, binary and112 source hashes |
| `approval/APPROVAL-BINDING.json` | SHA256 `d3f78606f7a8bad161ee58a0c0fb5e7f7bf83e287140dbc9a4bbe2c028ce93bd`; exact operative contracts, reviews, final QA04 and provenance copies |
| Final frozen analysis | A new immutable output from the unchanged frozen `analyze_joint.py`, or equivalently proven per-run strict results. Preserve the original file and every source/provenance record. Adapter imports no evaluator. |
| Evaluation provenance | Exact analysis file/hash, command/source identity, strict mode, formal freeze, binary, chosen population oracle and evaluator/dependency hashes, and trace path/hash for each result |
| Campaign progress and terminal status | Quiescent immutable snapshots of the exact controller progress and status files, sufficient to distinguish never started, started, terminated and unavailable evidence |
| Original traces | Only the explicit trace index, sequentially, for call-level cost/rate/role details and peak references absent from summaries. No model request/image bodies need be loaded for quantitative reporting. |
| Budget | A quiescent snapshot of `freeze.ledger_path`, not the live mutable file during measurement; preserve every call UUID/state/amount/scope and the40USD limit |
| QA04 | The bound final report, count classification, final checkpoint, independent review and630 raw-acceptance manifest; reuse one42-change dataset |
| Mechanism and semantic reviews | Bound activation output and separate immutable review records. Missing reviews remain UNREVIEWED. No automatic semantic judgment or hypothesis verdict. |

Quiescence evidence must identify the stopped/completed controller and final progress snapshot, confirm no active episode process and no in-flight RESERVED call, or explicitly retain incomplete termination/settlement. Unknown usage can remain after settlement; it does not prevent a truthful incomplete report. Do not infer quiescence from elapsed time or a missing PID alone. The adapter consumes this evidence rather than controlling processes.

Preserve the explicit planned order and actual start/end records. Produce no retry, model call, fixture, candidate command or upload. Copy no credentials, environment, weights, binaries or base64 media into report outputs. Raw links remain references to the already authorized experiment.

### Existing result ABI and a concrete limitation

Frozen joint output contains `version`, `freeze_sha256`, `primary_common9.{planned_episodes,observed_episodes,results,summary}`, `supplement6.{planned_episodes,observed_episodes,results,summary,mechanism_activation}`, `trace_provenance`, `total_actual_recorded_API_cost_usd`, `unknown_call_count`, and `unexecuted_planned_identities` (`analyze_joint.py:12–39`). Its `results` are the numerical source of truth for this adapter.

One small existing strict envelope was read, without its raw trace or re-evaluation: `closed-episode-light-evaluations/AABA-S84-r1-STRICT-ENVELOPE-v1.json`,26396bytes, SHA256 `bdc94ef8fda96b0a512ecaf034f263bdc90656fa7bfb5d4fd5d880cdf7fd45bd`. It uses `identity`, `trace.{path,sha256,bytes}`, `evaluator.{mode,version,sources,formal_freeze_sha256}` and `result`. Its result has `version=COMMON9-EVALUATOR-v3`, `evaluation_mode=strict`, eligibility and the usual QA fields. It does not bind the supplied supplemental oracle or supplemental adapter and has no `result.population`. Therefore it is an ABI example and a bounded technical diagnostic, not a complete population-analysis envelope. Do not silently upgrade it. Derive population only from the exact frozen job mapping and require the missing oracle/analysis-generation provenance before treating an isolated envelope as final quantitative input.

Historical identity-only lightweight evaluations are likewise not newly certified by hashing them now. They may remain diagnostic references. A later, independently recorded association must establish which exact trace/oracle/evaluator produced a result; the adapter cannot infer that fact.

## 2. Fixed populations and report shape

Emit both populations and all seven configurations even when no result exists. The primary common9 episode IDs are S02,S08,S11,S12,S15,S18,S19,S20,S26; supplement IDs are S80–S85. Use the frozen job list rather than regenerating order.

| Per configuration | COMMON9 | COMMON_SUPPLEMENT6 |
| --- | ---: | ---: |
| Planned episodes | 45 | 18 |
| Repetitions | 5 | 3 |
| Episodes per bundle | 9 | 6 |
| QA01 response obligations | 215 | 36 |
| QA02 binding obligations | 75 | 18 |
| QA03 goal obligations | 115 | 24 |
| QA04 tasks | Same C01–C06 | Same C01–C06, no extra measurement |

The complete planned worklist is441episodes,1757responses,651bindings and973goals. Keep these as separate measures, not a pooled success denominator. Totals across configurations are workload/coverage counts, not independent user situations.

Generate:

1. `report.json`: canonical structured output with provenance, partitions, per-population/configuration summaries, paired diagnostics, cost reconciliation, QA04 reference and limitations.
2. `planned-episodes.jsonl` and `obligations.jsonl`: every planned identity, original result pointer and disjoint reporting state. Do not omit failed or unexecuted rows.
3. `case-repetition.jsonl`: each case/rep/configuration, and each complete or incomplete fixed case bundle by rep.
4. `paired.jsonl`: all planned comparison identities and exclusion reasons; certified paired intersections separate from preserved raw paired diagnostics.
5. `calls.jsonl`: deduplicated call-UUID provenance and financial/resource classifications, without model input/output bodies.
6. `prediction-worklist.json`: exactly24 DP×QA cells with both population evidence references and UNREVIEWED judgment fields.
7. `report.md`: two seven-row six-QA overview tables, separate coverage/failure tables and bounded condition/pair explanation. Never render null as0.
8. `manifest.json`: source/input/output hashes and sizes, tool version/hash, approved spec hash, actual command, scope, warnings and completion status. Write last with exclusive creation.

Use stable keys `(population, configuration, episode, rep)` and `(same, obligation_kind, obligation_id)`. `run_id` is evidence attached to that identity, not the identity itself. QA01 IDs use `response_id`; QA02 and QA03 use `id`. Keep the actual run ID and trace SHA on every observed row. The frozen populations have disjoint episode IDs, but still retain population explicitly.

The seven configurations are AAAA,BAAA,CAAA,ABAA,AABA,AAAB,AAAC. Reuse the exact same AAAA records in four DP comparisons. Do not multiply their costs or regard reuse as four independent experiments.

## 3. Validation and state partitions

### Artifact acceptance

Before producing certified tables, verify exact input hashes, supported versions, FORMAL phase, binary hash, planned membership, unique result identity, unique obligation IDs and expected per-case oracle IDs/counts. Match `trace_provenance` to each explicit preserved trace and result identity. Validate both original path and recorded root association; a relocated clean-clone package needs an explicit verified restore mapping rather than silently accepting an unrelated path.

Duplicate, unplanned, mixed-generation, stale-hash or inconsistent result records remain in diagnostic output and cause `INVALID_REPORT_INPUT`; never select one to make a complete table. An absent result is not a bad numerical result. Do not consume legacy_diagnostic as strict certification.

Frozen primary eligibility is `result.primary_measurement_eligibility == ELIGIBLE`, produced only for strict mode and `deadline_audit.status == CONSISTENT` (`evaluation_v3/evaluate.py:63–72`). Require these fields to agree. An internally inconsistent saved result is an input contradiction, not a new opportunity to rerun or reinterpret the evaluator.

### Episode states

Use the exact quiescent progress records. Frozen controller states include STARTED,DISPATCHED,TERMINATED,BLOCK_ADMITTED,BLOCK_BUDGET_STOP (`joint_campaign.py:137–193`). Do not call a trace missing from the analysis unexecuted merely because `analyze_joint.unexecuted_planned_identities` lists it.

| Reporting state | Evidence |
| --- | --- |
| UNEXECUTED_VERIFIED | Complete quiescent progress snapshot has no STARTED for the planned identity |
| STARTED_UNCLOSED | STARTED/DISPATCHED exists, no supported terminal evidence |
| TERMINATED_TRACE_MISSING | TERMINATED exists, referenced trace/result is missing or unavailable |
| CLOSED_RESULT_UNVERIFIED | Closed trace/result exists but provenance or required strict analysis evidence is incomplete |
| CLOSED_TECHNICALLY_UNCERTIFIED | Provenance-valid strict result is NOT_CERTIFIED; retain audit reason |
| CLOSED_CERTIFIED | Provenance-valid strict ELIGIBLE/CONSISTENT result |
| IDENTITY_OR_SOURCE_CONFLICT | Duplicate, unplanned or contradictory records; report input invalid |

Keep process exit code, ordinary functional failures, missing sources, cancellation, budget stop and technical abort as independent factual fields. Exit0 does not establish semantic success; ordinary functional failure does not automatically remove a technically certified episode.

### Obligation partitions

Populate all expected obligations from the frozen population oracle, then attach observed results. For QA01/QA03 apply the following precedence once, producing disjoint counts that sum to the fixed denominator:

1. `UNEXECUTED_VERIFIED` when the episode is verifiably unstarted.
2. `EVIDENCE_UNAVAILABLE` for started-unclosed, missing trace/result, unverified provenance or an absent expected result row.
3. `TECHNICALLY_UNCERTIFIED` for a proven NOT_CERTIFIED result, retaining any raw number as diagnostic.
4. `CERTIFIED_UNRESOLVED` for a certified row whose frozen status is UNRESOLVED or starts with UNRESOLVED.
5. `CERTIFIED_NOT_OBSERVED` for frozen NOT_OBSERVED with no exact metric.
6. `CERTIFIED_OBSERVED` for frozen OBSERVED with finite nonnegative exact `latency_ns` or `via_ns`.

Unexpected status/number combinations are `EVIDENCE_UNAVAILABLE` with `ABI_CONTRADICTION`, preserve the original row and invalidate the affected certified aggregate. Do not repair them. Frozen summaries currently aggregate eligible non-null numbers, so valid normal inputs must reproduce those summary counts/means. This adapter adds the explicit partition, not a new endpoint rule.

For QA02, retain the first frozen judgment. After the first three episode/evidence categories above, partition into `TECHNICALLY_UNJUDGEABLE_ADOPTION` for UNRESOLVED_OR_LATE_ADOPTION, `KNOWN_CORRECT` for ADOPTED/correct=true, `ADOPTED_INCORRECT` for ADOPTED/correct=false, and `NOT_ADOPTED` for ordinary non-adoption. A NOT_ADOPTED row is not a technical failure. Never substitute `pre_deadline_correct` for `correct`.

QA02 known-correct/fixed-planned is an observed success lower bound when unknown/unexecuted rows remain, not a complete accuracy estimate. Retain the frozen `fixed_denominator_accuracy=null` when judgment coverage is incomplete. No S02 binding cases or supplementary DP2 document names are added.

## 4. Metrics and units

### QA01 and QA03

Use original per-obligation values and selected IDs. Sum integer nanoseconds, retain `sum_ns` and `count`, and derive `mean_ms=sum_ns/count/1e6` only for a nonempty certified observed set. Preserve exact0. No observation yields null and its state, not0 or a penalty time. Never replace a QA01 callback after content review or use midpoint/lower-bound latency from an unresolved earliest-boundary row as an exact latency.

QA01 label: **first substantive response host-callback latency**. Publication certification is not semantic correctness. QA03 label: **frozen causal VIA processing time to the accepted current goal completion**. Keep per-goal `via_ns`, original start/member IDs, terminal ID, wall/excluded/DAG fields if present and raw pointer. Do not rebuild the DAG in this adapter.

Per-case and per-repetition tables use the same sums/counts and fixed oracle counts. Overall means are micro means across observed obligations, not the mean of case means or repetition means when coverage differs. Present all failures alongside means. An observed-subset speed advantage is not whole-population success speed.

### QA04

Read the final accepted report through the approval binding. Fields are `all_counts_by_configuration`, `mean_changed_operational_modules`, and42 `rows` with configuration,task,count,modules,operational_module_paths,count_classification and acceptance references.

For all seven configurations the accepted C01–C06 vector is `[1,1,3,3,1,2]`, sum11, mean11/6=1.8333333333333333. Display1.833 with exact numerator/denominator. Preserve raw changed-file counts and formatter exclusions as separate evidence, not the QA numerator. The bound report SHA is `f9dee15d44e02f4ea95826cd16d73812755e756cebd00b5afed5f103cbd27f0e`; classification SHA `55f87e66f4b511ed74686f6a5e66e0f42f3287febddb28cadbb3ecd5f5841130`; final independent review SHA `43bdd5ef5f37378ae4f26f78e0038d21f40d85091a4f7b9f412866bfd20cea41`; independent630 audit SHA `26cd58d883958a7e1b77e612c90acc299fb40d809de22cf6f506873bc74287ab`.

Select artifacts by exact role/path/hash and final acceptance linkage. The binding intentionally preserves an earlier `count_v2_classification_pending_independent_review` status; do not mistake that historical role label for the later final acceptance state. Preserve both.

This is an observed tie for six specified changes in this PoC's shared Rust module boundaries and configuration branches. It does not establish intrinsic structural equality of fully separated production implementations. QA04 has42 independent changes and630 deterministic regression episodes, not additional paid441 observations. Its three RSS-incomplete regression diagnostics remain separate from formal QA06 and do not silently become OBSERVED.

### QA05

Keep all known `qa05.known_api_cost_usd`, unknown calls/reservations and role totals, including technical failures and ordinary semantic failures. Never drop cost with an excluded timing observation. Preserve frozen cost status and execution comparability as separate fields.

For each fixed rep, sum the9 common9 or6 supplement episodes. The representative mean bundle cost follows the frozen predicates: complete planned identity population, all technically certified, and no unknown usage. Otherwise leave representative bundle mean null; report actual known expenditure and any explicitly labeled partial/lower-bound diagnostic. Do not divide a partial known total by5/3 and call it a complete bundle estimate. Never compare9-case and6-case bundle cost as a direct savings ratio.

`qa05.calls` is total call records, not actual cloud requests. Extract explicit `trace.calls` by UUID and retain:

`call_uuid, population, configuration, episode, rep, run_id, stage, role, evidence_kind, status, actual_api_sent, input_tokens, output_tokens, raw_usage_ref, cost_usd, cost_state, reservation_usd, price, scope_root, initialization_ns, inference_ns, request/response file refs, trace_pointer`.

Partition record kinds into actual cloud dispatch (`actual_api_sent=true`), local model (`evidence_kind=LOCAL_MODEL`), and administrative/unsent. Classify roles independently as VIA,RECEIVER,LOCAL_MODEL or recorded other. Failed/late/discarded/retried work stays included; semantic usefulness is not a cost inclusion filter. Do not guess discarded/unused labels if no actual evidence records that condition.

Group pricing classes using the full recorded price identity: model, currency, input/output rates, region, input bracket, source and date. The frozen comparison rate is0.40USD/M input and1.60USD/M output, not an invoice. Preserve any different recorded class and flag it; do not silently reprice. Local API charge0 does not mean CPU/RAM cost0. Keep local initialization/inference time and RSS evidence; `local_compute_monetized=NOT_MEASURED`. Local token fields containing0 are not proof of measured token usage unless their record class establishes that meaning.

### QA06

Use each frozen result's `peak_concurrent_sum_bytes` and exact `peak_same_sample`, then take the maximum across that fixed population/configuration. All maxima refer to one observed sample, not sums of role maxima or episode peaks. Retain all tied peak references, process PID/role contributions at that same sample, start/end bracket, method and scope. Display decimal MB=`bytes/1e6`; optional MiB=`bytes/1048576` must have a distinct label.

Keep `status`, policy_version, sample_count,valid_sample_count,invalid_sample_count,confirmed_exit_boundary_count,startup_zero_rss_count,sampler_failures,incomplete_registered_process_samples,max_sample_gap_ns and lower-bound flag. Do not reclassify raw samples or infer a normal0 for a missing PID. An incomplete episode/population has an observed lower-bound diagnostic, never a complete memory maximum. Exact counts may be summed, but do not sum gaps between separate episodes as sampler gaps.

Population completeness requires all planned episodes technically certified and each QA06 status OBSERVED, following the frozen summaries. Also show the maximum over all recorded valid samples with incomplete-state labeling; preserve technically uncertified traces' numeric peaks as diagnostics.

Scope is fresh local Python runner/bridge, Rust candidate and registered actual local helper. It excludes remotely hosted Qwen/backbone weights, server/KV memory, browser/replay controller and outer orchestrator, and is not full product/device memory. It is nominal50ms sampled process-tree RSS, not a guaranteed continuous-time peak. Shared physical-page deduplication is unverified. Logical buffers and stored file sizes are not RSS.

## 5. Global cost reconciliation

Consume the quiescent ledger snapshot by its explicit freeze path. Frozen ledger ABI is `limit_usd` and `calls[call_uuid]={amount_usd,state,scope_root}`. RESERVED and UNKNOWN retain reservation amounts; other recorded states contribute their actual stored known amount (`joint_campaign.py:54–64`, `bridge.py:25–35,120–136`). Preserve state names, including USAGE_RECORDED and zero-valued CANCELLED_UNSENT_RESERVATION_RELEASED.

Produce a UUID join table and report:

1. Frozen analysis known formal cost and unknown count.
2. Sum of uniquely identified formal call records by population/configuration/role/rate class.
3. Current formal `scope_root` ledger known cost, RESERVED amount and UNKNOWN amount.
4. Other ledger scopes' known cost and reservations, preserving prior campaigns/probes as separate spending.
5. Global known+RESERVED+UNKNOWN, remaining reservation-aware budget and cap40.
6. Unmatched ledger UUIDs, unmatched call UUIDs, conflicting duplicate records, amount/state disagreements and residuals.

Do not count the same AAAA call again for each DP, or the same UUID twice if present in both a trace and global call log. An absent ledger row is acceptable only for a recorded class that never reserves, such as actual local compute or a blocked unsent administrative call; record the reason rather than forcing equality. Conflicting copies invalidate reconciliation, not a reason to choose the cheaper value.

A later ledger settlement that is absent from a saved trace is an explicit reconciliation discrepancy. Preserve both states and late-known spend. Do not silently rewrite a frozen per-trace evaluation or allocate unmatched spend to a guessed episode. The global ledger remains the budget accounting record; per-population comparison remains incomplete until supported linkage is available.

Use decimal arithmetic over recorded decimal representations for derived accounting. Keep exact source floats/strings and report any residual; presentation rounding must not hide an unmatched call or amount. This creates no invoice claim, price update, budget increase or new dispatch permission. Initial settled0.7150616USD is historical context only; derive final other-scope totals from the final snapshot.

## 6. Certified pairs and condition tables

Create all planned pair identities, not only the union of present result rows. Within each population use identical episode,rep and obligation ID:

| DP | Alternatives | Pair rows |
| --- | --- | --- |
| DP1 evidence supply | AAAA=A, BAAA=B, CAAA=C | A/B,A/C,B/C |
| DP2 interpretation execution | AAAA=A, ABAA=B | A/B |
| DP3 screen/speech grounding | AAAA=A, AABA=B | A/B |
| DP4 handoff | AAAA=A, AAAB=B, AAAC=C | A/B,A/C,B/C |

The additional B/C rows are descriptive comparisons of existing same-case observations, not new experiments or a substitute baseline. Derive each directly from its own same-obligation pair; do not subtract A/B and A/C means computed on different intersections.

For QA01/03, include a delta only if both sides are provenance-valid, CLOSED_CERTIFIED and that same obligation is CERTIFIED_OBSERVED. Retain side states and exact exclusion reason otherwise. `delta_ns = right - left`; lower latency is favorable but an observed timing sign is not a final winner. Report paired count/fixed planned pair count, sum/mean delta and both sides' fixed-population coverage. No semantic PASS gate is added to pairing.

Keep the frozen analyzer's `paired_case_level_diagnostics` unchanged, when supplied, under `raw_analyzer_paired_diagnostics`. Its original algorithm pairs any two non-null numbers without primary eligibility (`common9/analyze.py:28–47`); never label it certified. Joint analysis itself contains no completed paired table.

For QA02, show both full fixed-denominator counts and a paired categorical table for known-correct/adopted-incorrect/not-adopted/unknown states. Do not drop non-adoption because it is inconvenient. For QA05, show actual spent values for every side and a separate comparable-pair flag requiring both complete technical execution and settled usage. For QA06, preserve each recorded peak/lower-bound state; a comparable peak difference needs both complete observations. These cost/RSS pairs are supplementary; representative QA05 is fixed-bundle mean and QA06 is population maximum.

Emit per-case/rep tables for all15 cases. Supplement conditions are fixed: S80 independent overlap, S81 sequential countercondition, S82 dependent correction, S83 small/dense, S84 larger/sparse, S85 native selection on the dense screen. Do not exclude a condition when its mechanism fails to activate. Include all7 configurations in each condition table, even when its main hypothesis concerns only one DP. Micro aggregation and fixed denominators remain unchanged.

## 7. Prediction evidence worklist

Produce exactly24 top-level rows, one for each DP1–DP4 × QA01–QA06. Each holds all relevant alternative pairs and separate COMMON9/COMMON_SUPPLEMENT6 evidence. It is not48 replacement hypotheses or a weighted ranking.

Required fields:

`dp, qa, alternatives, preregistered_statement_ref, prior_direction_or_NO_PRIOR_RANKING, favorable_conditions, counterconditions, fixed_population_summary_refs, case_rep_refs, certified_pair_refs, raw_pair_refs, actual_mechanism_refs, semantic_review_coverage_refs, limitations, judgment_status, judgment, reviewer, reason`.

The adapter initializes `judgment_status=UNREVIEWED`, `judgment=null`. Copy or reference the exact preregistered condition from frozen19 section7 and20 section6. If there is no strict prior ranking for a cell, retain NO_PRIOR_RANKING rather than deriving a prediction from the observed sign. Final Astra judgment uses the established vocabulary: conditional support, opposite observation, offsetting costs, condition absent, mechanism not activated, observed tie, no prior ranking, unresolved or unmeasured, with the cited document's distinctions. Do not create an automatic winner or statistical significance threshold.

Mechanism evidence is diagnostic:

- DP2: actual request-owned cloud dispatch-to-terminal overlap can support the realized path when corresponding request/goal interpretation is correct. It includes transport and is not provider internal compute concurrency. Preserve semantic INPUT_CAPTURE commit versus raw host receipt proxy; exact initial free K2 capacity is UNRESOLVED because no slot release clock exists. Only completed calls support the existing interval/slot proof; failed/cancelled terminal records remain unresolved. Non-overlap with unknown capacity does not falsify a contention-free-capacity hypothesis.
- DP3: link actual parser spawn/dispatch, screen/speech producer, binder/expansion, native events and exact-set results. Native skip needs actual native evidence and absence of both spawn and parse dispatch with fields present. S83/S84 is the registered combined size-plus-spacing contrast, not a general isolated geometry effect. B's screen producer still uses PNG evidence.
- DP1: reuse/preparation requires actual source/revision/range acquisition and subsequent reuse evidence. These are not summarized sufficiently by `mechanism.py`; leave missing associations UNREVIEWED and link a bounded future manual/extraction task. Preparation alone does not prove a shared semantic range-selection call disappeared.
- DP4: repeated consumption must refer to the same retained record/version. Do not infer amortization from repeated consumption of newly regenerated records or credit post-first-response work to QA01. Supplemental conditions without intended handoff activation do not establish handoff benefit.

The semantic review join is annotation and support for an interpretation, never retrospective QA01 filtering. Keep actual-request correctness, literal oracle-field match and unresolved oracle/input mismatch distinct. Existing interpretation guidance can be referenced after verifying current frozen source equivalence; no historical run outcome is inherited. Approval/handoff NOT_APPLICABLE must not become PASS, and shared response/goal evidence must not be double-counted.

## 8. Core JSON schema and table contracts

Implement a versioned JSON Schema for the final output. The following is the required core; extend it with the field contracts above, not alternative meanings. File refs always include exact path, SHA256, byte size and optional JSON pointer. Values without evidence are null with a reason, never default0.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "urn:via:joint441:quantitative-report:v1",
  "type": "object",
  "required": ["version", "status", "provenance", "populations", "qa04_reference", "pairs", "cost_reconciliation", "prediction_worklist", "limitations"],
  "properties": {
    "version": {"const": "JOINT441_QUANTITATIVE_REPORT_v1"},
    "status": {"enum": ["COMPLETE_INPUT_REPORT", "INCOMPLETE_EVIDENCE_REPORT", "INVALID_REPORT_INPUT"]},
    "provenance": {"type": "object"},
    "populations": {"type": "array", "minItems": 2, "maxItems": 2, "items": {"$ref": "#/$defs/population"}},
    "qa04_reference": {"type": "object"},
    "pairs": {"type": "array"},
    "cost_reconciliation": {"type": "object"},
    "prediction_worklist": {"type": "array", "minItems": 24, "maxItems": 24},
    "limitations": {"type": "array", "items": {"type": "string"}}
  },
  "$defs": {
    "population": {
      "type": "object",
      "required": ["population", "planned", "configurations", "case_repetition_refs"],
      "properties": {
        "population": {"enum": ["COMMON9", "COMMON_SUPPLEMENT6"]},
        "planned": {"type": "object"},
        "configurations": {"type": "array", "minItems": 7, "maxItems": 7, "items": {"$ref": "#/$defs/configuration"}},
        "case_repetition_refs": {"type": "array"}
      }
    },
    "configuration": {
      "type": "object",
      "required": ["configuration", "execution_partition", "qa01", "qa02", "qa03", "qa04_ref", "qa05", "qa06", "semantic_coverage"],
      "properties": {
        "configuration": {"enum": ["AAAA", "BAAA", "CAAA", "ABAA", "AABA", "AAAB", "AAAC"]},
        "qa01": {"$ref": "#/$defs/timing"},
        "qa03": {"$ref": "#/$defs/timing"}
      }
    },
    "timing": {
      "type": "object",
      "required": ["planned", "partition", "observed_count", "sum_ns", "mean_ms", "frozen_summary_ref", "raw_diagnostic_ref"],
      "properties": {
        "planned": {"type": "integer", "minimum": 0},
        "partition": {"type": "object"},
        "observed_count": {"type": "integer", "minimum": 0},
        "sum_ns": {"type": ["integer", "null"], "minimum": 0},
        "mean_ms": {"type": ["number", "null"], "minimum": 0},
        "frozen_summary_ref": {"type": ["object", "null"]},
        "raw_diagnostic_ref": {"type": ["object", "null"]}
      }
    }
  }
}
```

The implementation must additionally enforce uniqueness by population/configuration/prediction key, the planned constants, partition sums, obligation IDs and referenced file hashes; JSON Schema cardinalities alone do not prove those invariants. `COMPLETE_INPUT_REPORT` means input/provenance coverage is complete, not all model goals succeeded or all semantic reviews are complete. Render those separate states prominently.

For empty observations, `sum_ns=null`,count0,mean=null. For a nonempty observed set containing a true zero, sum0 and mean0 are valid. The QA02 schema must expose all categories from section3 and both complete accuracy and incomplete lower-bound fields. QA05/QA06 schemas must require their comparison/completeness state and scope, not allow a bare number to stand alone.

## 9. Processing algorithm

1. Load and hash the explicit manifests; validate versions and approved snapshot identities. Preserve an immutable diagnostics file for any rejection.
2. Build ordered planned identities and all oracle obligations. Preallocate both populations/all configurations even when no result exists.
3. Join quiescent progress, exact result index and trace provenance. Detect conflicts before aggregation. Do not infer never-executed from an absent result alone.
4. Copy original frozen result fields and provenance into reference rows. Create the disjoint reporting partitions without changing the underlying values/statuses.
5. Produce per-case/rep and fixed-population QA01/02/03 tables; compare arithmetic/counts to supplied frozen summaries and report discrepancies. Do not normalize a discrepancy away.
6. Link the accepted QA04 report and its exact per-task module counts. Keep the one42-change dataset separate from both formal populations.
7. Read explicit call records sequentially, deduplicate by UUID, group role/stage/rate class, and reconcile with the final ledger. Preserve unknown and late settlement conflicts.
8. Reference frozen per-run peak samples and aggregate actual maxima/completeness; do not rerun memory classification or sum independent role peaks.
9. Generate all planned paired rows with explicit side state. Compute certified intersections separately, preserve original raw paired output, and retain fixed population denominators.
10. Attach activation and semantic coverage references; create24 UNREVIEWED prediction cells. Do not assign a winner or add a content-correctness filter.
11. Render concise tables with null/status distinctions and scope footnotes. Hash every output, including Markdown; write the final manifest last. Existing output directories are immutable.

Process sequentially and cap individual reads with explicit configurable limits recorded in the manifest. Stream JSONL or use an explicit per-run result index; do not retain441 full raw traces and RSS arrays in memory. Oversized or malformed input must fail explicitly with its path and planned rows retained, not be silently skipped. No recursive filesystem scan, image decode or raw model-body extraction is needed. No frozen source imports with side effects.

## 10. Required synthetic acceptance before real postanalysis

Sol may implement small synthetic fixtures after separate authorization. No test is run by this specification.

1. Empty441 plan produces two populations,all7 configurations and correct215/75/115 versus36/18/24 denominators, with no false0 timing.
2. Never-started, started-but-missing, terminated-but-missing and provenance-unverified states stay distinct; every partition sum equals planned.
3. Duplicate run/identity, mixed phase/binary/freeze/oracle, bad hash and unknown obligation are rejected without selecting a favored record.
4. True observed zero survives; null, missing, nonfinite and negative values never become0 or an exact observation.
5. Technically uncertified numeric endpoints remain raw diagnostics and are excluded only from certified timing/paired summaries; their cost remains included.
6. NOT_ADOPTED differs from ADOPTED incorrect and UNRESOLVED_OR_LATE_ADOPTION; fixed-denominator lower bound and complete accuracy differ.
7. Unequal case/repetition coverage uses summed nanoseconds/count, not a mean of means.
8. A raw legacy pair with two numbers but one uncertified side appears only in raw diagnostics. Planned missing sides remain rows.
9. B/C pairs use direct same-obligation values; shared AAAA is reused without duplicated expenditure or independent sample counts.
10. Failed/discarded/cloud/local/admin calls remain distinct. `qa05.calls` differs from actual_api_sent count. Unknown reservations are not settled cost or0.
11. Duplicate UUID copies agree or create a conflict; late ledger settlement, unmatched scope and different rate classes remain explicit.
12. QA05 bundle mean is null for incomplete/uncertified/unknown-usage populations while known expenditure stays visible.
13. Same-sample RSS max differs from sum of role maxima; ties retain all references. Incomplete samples/population retain lower-bound labeling, remote memory absent and local compute not monetized.
14. QA04 binds final accepted11/6 vectors and one42-change dataset; historical1.667 or pending-status-only records cannot replace it.
15. Supplement six condition rows are retained when activation is absent/fails. DP2 raw receipt proxy does not become semantic commit or free-K2 proof; native skip needs both dispatch/spawn conditions.
16.24 prediction cells remain UNREVIEWED with explicit absent prior ranking. Numeric signs and ties never create semantic/causal proof automatically.
17. A semantic FAIL or oracle/input mismatch annotates the saved callback without replacing/removing its QA01 value. NOT_APPLICABLE approval/handoff does not count as PASS.
18. Output is exclusive/immutable, all outputs including Markdown are hashed, path escapes are rejected, oversized inputs fail visibly, and no evaluator/model/parser/network/candidate command is reachable from the adapter.

## 11. Evidence that remains unavailable or requires later work

The adapter cannot supply provider internal compute concurrency, exact worker-return/free-K2 boundaries, remote Qwen/server memory, shared-page deduplicated physical memory, continuous-time RSS maximum, local compute monetary cost, actual invoice or production UI/audio behavior. Mark these unavailable rather than estimate them from unrelated records.

Most semantic judgments and DP1/DP4 mechanism associations are not yet completed. Final raw analysis, quiescent ledger reconciliation and full441 coverage are not asserted here. A partial campaign or partial manual review must remain visibly incomplete. Missing evidence is not proof of a model error or of an architectural tie.

The additional S02 oracle fields identified in the manual reference are descriptive-label/request disagreements. Preserve the original oracle, actual input and interpretation guidance. Do not silently add requirements to the user request, rewrite a reference, or reuse a prior generation's semantic outcome.

## 12. Exact inspected sources

All paths below are relative to this formal root's `frozen/source/`; the original files remain unchanged.

| Source | Relevant lines | SHA256 |
| --- | --- | --- |
| `benchmark/architecture/common_supplement6_20261019/analyze_joint.py` |12–39 | `469a5c8a57ee1a3e6812b8c85182cc0e4b2fbd59b69fbca8eeb8591a0fcecb49` |
| `benchmark/architecture/common_supplement6_20261019/evaluate.py` |23–64 | `fbf42b62ff755cb1f903e025a06dbef1ebd7e642aef7c94319fb5ce4970335ef` |
| `benchmark/architecture/common_supplement6_20261019/mechanism.py` |16–79 | `9eba155a9de36a2b098e11b92abf0fe17cff692e6bfe66ee69163ea8c55c6fe1` |
| `benchmark/architecture/common_supplement6_20261019/joint_campaign.py` |32–75,137–193 | `9314f4f36f58ae2254e0f2d0cb33d4fe5debba5c43f450b696154068745c8d50` |
| `benchmark/architecture/common9_poc_20261019/evaluation_v3/evaluate.py` |19–73,91–120 | `7b51b6e3b96adf54f9dfff880698d7bda8eb228a2e62dfe0ebb37273c7da88ce` |
| `benchmark/architecture/common9_poc_20261019/evaluation_v3/analyze.py` |9–27 | `b3de7f8db03f3617899bd13b872568e742160a11f9f289eec45208fc17994627` |
| `benchmark/architecture/common9_poc_20261019/evaluate.py` |190–364,390–430 | `cc555971e996821e48f75cd5828fa5dfdce6f761e9d4cfade3bd547e321e9dd2` |
| `benchmark/architecture/common9_poc_20261019/analyze.py` |9–79 | `7f4f9ba359e250084565747b60b70a788af80ac0121c99c53d3b04575a6789d1` |
| `benchmark/architecture/common9_poc_20261019/bridge.py` |25–35,120–136 ledger records | `27f4200ff02d8c2f0dfca4e7b09c7ebd9ce076b4d1d2c570ebb712c6afb6ce64` |
| `docs/architecture/12-decisions/decision-packages/seven-problem-review/19-common9-evaluation-contract.md` |sections2,3,5–7 | `7b3db7b8f358ee6cd441ca37ef13f9b83d1f697322729a0f7bde802bbdc162e7` |
| `docs/architecture/12-decisions/decision-packages/seven-problem-review/20-common-supplement-evaluation-contract.md` |sections2,3,6–9 | `4be703679b161af9312b136f4056d121e4f09465bc39e0d87391cd5b068878d3` |

Additional bounded reads: the one strict envelope identified in section1, approval binding metadata, bound QA04 report metadata/schema, and the already written final-report plan. No corpus extraction, analyzer/evaluator, test, API, model, parser or build ran for this specification.
