# Quantitative adapter stage 02: independent bounded static review

2026-10-19. Reviewer: GPT-6 Astra Ultra, `/root/quant_adapter_astra`.

**CHANGES_REQUESTED: four scoped P2 findings and one P3 source-fidelity finding. No actual quantitative-report acceptance.** This is a review of the immutable stage-02 copies. Later repairs require a new checkpoint; do not rewrite this generation.

## Scope and evidence

The checkpoint and its five source copies match the declared hashes. The specification, final-report plan and prior review also match their referenced hashes:

| Artifact | SHA256 |
| --- | --- |
| Stage-02 checkpoint | `d610b1a2143d69f29f681448f75cdbbb3328923dbbde2c887aa16a18070fc7e9` |
| `report_adapter.py` | `d9fd4f6ff6f7cc260b0685737dd750ff8723269278ea13267a0dddf59794bb6d` |
| `test_report_adapter.py` | `2b38ea1d16d47aa17c3095607a437b276e6fbd214a507d57df1bce0a4a872c40` |
| Output schema | `84d185b187771b7f6df48d991abeef4889a332b5bc94b6ff8c28e9f9a97d07f0` |
| Input schema | `91f2353d321c58f2d5b2a3b0821ce6e842e9c5bc89a599c6af2a3ecbf4476546` |
| Adapter README | `b20973407d836bbcd8eaa0cefbadedf01f31ed8558366e4f49648b313b44e74e` |
| Quantitative adapter specification v1 | `b4bb8735e67ec84dbef1cb193ac3a36f6ea70411094515a7f5d616b0ee82a09e` |
| Final-report plan v1 | `bb23355ba50776e1096ccb009e1fd8d170d46a9d55abff8fe8eae273b9234e41` |
| Stage-01 independent review | `906e79a0987d9accc34f613b0c7160d47de26188c410b7aedbb275f7b5ade1d6` |

The five saved synthetic batches report 19 passing checks, with maximum batch duration 0.12491912487894297 seconds. Their records and test source were read, not rerun. The review also read bounded portions of the exact frozen joint controller, analyzers, evaluators, endpoint selector, bridge, harness and grounding adapter, plus the bound QA04 report metadata, final checkpoint and independent acceptance record. It did not scan the formal trace corpus or read a live ledger.

No test, adapter, analyzer, evaluator, model, parser, candidate, compiler, network request or process-control operation was executed. File hashing and static text reads do not establish actual final-result compatibility or campaign completeness. The sole new artifact from this review is this prose file.

## Stage-01 findings

- **A01 repaired in the reviewed scope:** result-body run/population/strict-integer repetition and copied frozen obligation declarations are checked before certification. Exact trace/result/oracle/source associations are required separately.
- **A02 partly repaired:** duplicate STARTED/TERMINATED and unstarted TERMINATED now invalidate affected episodes and obligations. Remaining lifecycle cases are S2-01 below.
- **A03 repaired:** canonical RFC6901 array tokens reject negative, signed, leading-zero, out-of-range and malformed escape cases.
- **A04 repaired:** QA01 accepts exactly `UNRESOLVED_EARLIEST_BOUNDARY`; QA03 accepts exactly `UNRESOLVED`. Unknown timing statuses are contradictions rather than new frozen states.

## Findings

### S2-01 / P2 — Complete progress can contradict a certified result without invalidating it

`report_adapter.py:245–246` checks only duplicate STARTED/TERMINATED and TERMINATED without STARTED. Once a record exists, `episode_state:114–125` does not reconcile it with progress. Therefore a provenance-valid strict result can become CLOSED_CERTIFIED when a declared complete progress snapshot contains no STARTED, an unsupported state, duplicate DISPATCHED, or an impossible ordering. The result can then enter certified obligations and pairs. The terminal count check at `:784–786` does not close this gap: an empty snapshot and status completed=0 can agree with each other while the result still certifies.

The exact frozen controller writes STARTED, then DISPATCHED, then TERMINATED for each completed dispatch (`joint_campaign.py:179–188`), with BLOCK_ADMITTED/BLOCK_BUDGET_STOP kept separately. The specification section3 requires contradictions in that evidence to affect identity certification.

Repair: validate supported per-identity lifecycle membership, multiplicity and order, with the frozen job identity/block association. A complete snapshot that contradicts a result is IDENTITY_OR_SOURCE_CONFLICT. A supported but incomplete progress prefix remains explicitly unverified or unclosed; it must not silently certify the result. Preserve result numbers, source refs, progress facts and actual costs. Apply the resulting state to obligations and every comparison. Do not infer lifecycle completion from exit0 or missing PID alone.

Small synthetic cases should cover no progress plus a proven result, duplicate/unknown/out-of-order DISPATCHED, complete and incomplete progress prefixes, and a valid complete lifecycle. The prior saved checks cover only duplicate TERMINATED and unstarted TERMINATED.

### S2-02 / P2 — Missing evidence is classified as an identity/source contradiction

`load_inputs:781–782` handles every `OSError`, including an absent or inaccessible referenced trace/result, by adding `record.conflicts`. `episode_state:115–116` then returns IDENTITY_OR_SOURCE_CONFLICT. A TERMINATED identity with a missing artifact therefore becomes invalid input instead of the required TERMINATED_TRACE_MISSING state. An index placeholder with missing refs also becomes CLOSED_RESULT_UNVERIFIED, because `TERMINATED_TRACE_MISSING` is reachable only when there are no record wrappers (`:106–108`).

Repair: retain distinct availability outcomes for trace, result and independent association. Missing or unavailable trace/result after supported termination is TERMINATED_TRACE_MISSING. Missing association for otherwise closed evidence is CLOSED_RESULT_UNVERIFIED. Incomplete progress without terminal proof remains unclosed/unverified. Stale hashes, malformed paths, contradictory identities, duplicate records and unsupported contents remain genuine conflicts. Preserve any successfully loaded raw result and calls when another referenced artifact is unavailable. Do not call any such identity unexecuted.

Small loader cases should distinguish a missing trace, missing result, absent association, missing-file index placeholder and stale-hash contradiction under the same terminal progress.

### S2-03 / P2 — RSS completeness trusts the status label without checking its required observation

`rss_summary:194` ignores invalid/missing peak values, but `:198` decides completeness solely from episode certification and `qa06.status == OBSERVED`. An OBSERVED record with no numeric peak can therefore yield COMPLETE_OBSERVED_MAXIMUM with null, or a partial maximum taken from other episodes. The summary and pair code also do not verify that `peak_same_sample` exists and agrees with the reported peak. A fabricated or incomplete sample association can consequently be called a complete same-sample maximum.

The frozen producer (`common9/evaluate.py:342–364`) supplies the selected numeric peak and the actual selected sample together. The saved RSS test in `test_report_adapter.py:221–227` omits the sample's `concurrent_sum_bytes` while asserting complete status, so it does not exercise that supported ABI.

Repair: check the recorded peak and same-sample payload for supported finite nonnegative byte values and consistent association before claiming complete/comparable QA06. Preserve invalid raw observations and the reason, with complete status/delta withheld. Do not rerun sample classification, sum role maxima, select a replacement sample or infer a missing PID's RSS. Continue to retain all tied valid peak references and distinguish incomplete diagnostics from full-population maxima.

Small cases should include absent peak, absent sample, mismatched sample sum, true observed zero, normal valid tied samples and a valid incomplete lower bound. This is an input-consistency check, not a new memory evaluator.

### S2-04 / P2 — Repetition bundles omit QA01–QA03 observations and coverage

The specification section4 requires per-case and per-repetition tables to retain the same fixed counts and integer-sum timing calculation. `build_report:357–361` emits FIXED_REPETITION_BUNDLE rows containing QA05 and QA06 only. QA01, QA02 and QA03 appear only in the individual CASE_REPETITION rows and overall configuration summary. The report therefore lacks the required repetition-level timing/accuracy partitions and cannot directly show whether a repetition's mean reflects a different observed subset.

Repair: project each fixed bundle's existing obligations with the same timing_summary/binding_summary rules. Per configuration and repetition the fixed counts are common9 QA01/02/03 =43/15/23 and supplement6 =12/6/8. Keep missing, unresolved, uncertified, ordinary non-adoption and unexecuted partitions, exact sums/counts, true zero and null. Do not average case means or pool populations. Unequal-observation synthetic rows should demonstrate that bundle means are micro means.

### S2-05 / P3 — Recorded post-completion discard metadata is omitted

The frozen harness sets `discarded_after_candidate_end=true` when delivery of a model result fails after candidate termination (`harness.py:99`). `project_call:578` copies only `discarded`, `unused`, `retried`, `retry_of` and `cancelled`, so its `recorded_work_disposition` omits the actual frozen field. The call's cost is retained, but the projection loses the recorded explanation that the completed result was discarded.

Repair: preserve the exact existing `discarded_after_candidate_end` field and other specifically supported recorded disposition fields when present, without guessing them from status or treating absent as false. Keep differences between raw trace and earlier global-log copies explicit; a later trace-only annotation should not manufacture a different financial call or duplicate its cost.

## Positive observations and remaining limits

The adapter keeps both fixed populations and all seven configurations, 441 planned episode rows and 3381 obligation rows. It retains fixed QA02 denominators and ordinary non-adoption, exact timing zero versus null, micro means, technical failures, and unchanged QA01 values after semantic annotation. The eight direct comparison groups include B/C without subtracting means from different intersections. All24 prediction cells remain UNREVIEWED with no automatic architecture ranking.

The QA04 loader's exact five artifact hashes and acceptance/checkpoint links match the inspected bound metadata. It reuses the same42 changes and630 regressions, retains 11/6 and separates regression RSS diagnostics from formal QA06. Financial projection uses Decimal, deduplicates UUIDs, retains late settlement/reservation disagreements and does not reprice calls. These observations do not certify a completed final reconciliation.

The source exposes no evaluator import, model/parser invocation, network call, subprocess or runtime control. Output creation is exclusive, payloads including Markdown are hashed, and the manifest is written last. No schema or implementation change is proposed for the disclosed diagnostics-object shape of `pairs` or the separate population file references.

Actual integration still requires independently recorded result associations, final quiescence/snapshot evidence and a root-authorized quantitative run. The adapter does not manufacture those prerequisites. A final joint analysis can be referenced through explicit JSON pointers or equivalently proven bounded per-run results; repeated reads of one large shared result file consume the configured cumulative read budget, so source-sized input packaging must be established at that later integration step. No actual final corpus size, full441 ingestion, semantic review, mechanism judgment or finished-report acceptance has been established here.

All findings concern reporting-only source. They require no frozen runtime/oracle/contract/QA04 modification and no intervention in the ongoing experiment.
