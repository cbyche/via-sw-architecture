# Independent review of review-v2 metrics

Scope: read-only review of `timing.py`, `analyze.py`, `memory.py`, and DP01/02/05 content/contract scorers after calibration-v1. No candidate edits, API calls, or formal performance reruns. Reviewed source and hashes are preserved in `reviewed-source/` and `reviewed-source-hashes.json`. These are the pre-fix analyzer generation, not a claim about a later runtime revision.

## Required fixes reported to runtime

1. **Per-option QA04 attribution.** Reviewed `analyze.py` lines 137–141 assigns one global `mean_changed_operational_modules` to every BASIC row. Functional C05/C06/etc. have genuinely different applicability and variant-specific patch counts. Use the frozen per-DP/per-option applicable task records. Do not reuse the former format-adapter tie. Runtime acknowledged replacement.
2. **Preserve completed content before later episode failure.** Lines 51–61 recover actual adoption/admission/read events but not completed detailed outputs when EPISODE_RESULT is absent. A real earlier GOAL_COMPLETE followed by later EPISODE_ERROR is scored with empty outputs. Recover only actual committed output events, retaining goal/revision/source/receipt fields; no inferred answer text. Runtime acknowledged this correction. Actual calibration-v1 `calibration-02-E4-A-r1-16fe21` and `calibration-02-E4-B-r1-d36439` additionally show that EPISODE_RESULT.outputs contains no goal_id, whereas actual GOAL_COMPLETE carries the correct goal_id, revision and receipt. Thus canonical actual completion outputs must use those committed events even when EPISODE_RESULT exists; the current goal-only scorer otherwise falsely misses both correctly supplied cost/Kim conditions. Raw EPISODE_RESULT must remain preserved.
3. **Select the episode's frozen evaluator generation.** Reviewed `timing.population()` reads current live goal manifests; `analyze()` reads current live oracles. Neither ties these to the record's freeze. Phase-frozen source/evaluator files or a strict hash guard plus explicit reanalysis version are required. Scorer version/hash must be recorded. Otherwise old calibration could silently acquire formal-generation meaning. Runtime acknowledged phase-frozen selection.
4. **Content revision selection.** `score_dp02.score()` keys outputs by goal only and ignores the already-frozen content check `revision`. A revision1 output containing `120 75` passes a revision2 check despite no revision2 output. Match actual `(goal_id, revision)` and preserve missing-revision status. QA03 already independently refuses the stale completion; this bug concerns the separate content diagnostic. Reported to runtime.

These are derived-analysis corrections; frozen inputs, raw traces, targets, oracle intent, model calls and failure facts remain unchanged. Later fixed-analyzer verification is still required.

## Fixed-analyzer independent recheck

The runtime's subsequent fixed source is independently preserved in `fixed-reviewed-source/` with `fixed-reviewed-source-hashes.json`. All four required calculation corrections above are **CLOSED for this reviewed generation**: canonical actual GOAL_COMPLETE outputs always supersede auxiliary output shape for scoring; content checks require the frozen revision; evaluator selection and hashes are phase-frozen; QA04 lookup selects actual DP/variant task records.

The same nine checks were independently rerun as `check_fixed_review.py` / `fixed-checks.json`: **9/9 PASS**, with the two earlier bug cases now asserting correct preservation/rejection rather than reproducing defects. `check_fixed_extra.py` / `fixed-extra-checks.json` adds **5/5 PASS**: an extra actual intent fails exact utterance multiset; N-H uses fixed expected opportunities while extras/duplicates fail functionally; phase-frozen evaluation ignores live files; frozen file tampering is rejected; and QA04 A/B use distinct option-specific counts instead of the deliberately invalid global tie value.

Actual unchanged calibration-v1 E4 A/B records were also rechecked with their frozen oracle: each has content **2/2** and QA03 completion **2/2**; exact utterance N-R remains **A 2/3, B 3/3**. The A extra relation is preserved, not erased by completion recovery. Full details are in `fixed-actual-calibration-checks.json`. This is derived calibration diagnosis, not a formal A/B result.

One evidence-provenance improvement was requested separately: record the hash map of all actually used analyzer/timing/evidence/scorer modules, not only analyze.py. The frozen evaluator inventory is already checked. The independent snapshot hashes here make this review reproducible; runtime report-wide provenance is to be confirmed in the final audit. No BASIC candidate or raw evidence was changed by these rechecks.

## Verified timing/resource properties

- Manifest entries, rather than emitted GOAL_START/GOAL_COMPLETE events, determine each recorded episode's QA01 and QA03 opportunities. Missing captures/callbacks/completions stay NOT_OBSERVED/INCOMPLETE; elapsed means are conditional and coverage remains explicit.
- QA01 uses the first nonempty admitted handling/detail/question display for that stimulus. It does not replace a wrong first answer with a later oracle-correct answer.
- QA03 begins at the manifest origin stimulus. Late GOAL_START does not reset it. Final required revision is enforced independently of earlier successful revisions.
- Without external work the whole input-to-completion elapsed duration is retained. With DP05 receiver work, only actual RECEIVER send-to-terminal intervals causally scoped to that same goal are excluded, using the union of overlaps and clipping to the measured goal window. Other-goal and unassigned receiver calls are not subtracted. The explicit profile is an API-boundary receiver-shim proxy, not a full sparse-parent critical-path proof.
- Current USER_WAIT events occur outside the applicable new answer-delivery goal or belong to an unrelated input while R1 still runs. Keeping those intervals in R1 is correct. This does not establish general user-wait attribution support for future goal shapes.
- RSS peak uses one concurrent sample's Python+Rust sum, not independent per-process maxima. Peak sample, role components, sampling gap and incomplete process visibility are retained. Shared physical pages, UI and cloud model weights remain outside this RSS estimate. Logical serialized-buffer bytes are separately labeled, not added to RSS.
- Phase/DP/variant/tactic grouping is separate. Comparison status and missing counts accompany conditional means. Missing entire scheduled episode records require the campaign plan/completeness audit in addition to per-record goal manifests; this file alone does not certify full campaign completeness.

## N-R / N-H / content scope

Reviewed N-R expected-intent matching is recall-like and does not penalize additional actual intents. Root/runtime already approved a separately versioned utterance-level exact multiset analysis with the original measure preserved. Do not call expected-intent recall a general utterance accuracy rate.

Reviewed N-H appends unexpected and duplicate actual contracts to its expected-contract detail list, then uses the resulting length as denominator. The policy must be explicitly frozen; if the intended population is scheduled authorized handoffs, keep its denominator fixed and report extra/duplicate contracts separately. This is a contract clarification request, not evidence of a current numeric discrepancy. Actual source/currentness/range checks and receiver-content diagnostics remain distinct.

Finite content-marker groups are not semantic entailment or general answer correctness. An actual callback/GOAL_COMPLETE instrumented event is not itself proof that evaluator content conditions passed. DP02 E4 uses the specifically scoped answer-delivery subgoal to an existing Task, not the lifetime of the entire pre-existing Task.

## Reproduction

`python3 results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/analysis/metric-review-v2/check_review.py`

Nine synthetic checks pass against the preserved reviewed source: fixed missing opportunities; first callback; final-revision incompleteness; original input origin; same-goal receiver union; full wall without external work; two explicit bug reproductions for late-output loss and stale-revision content; and same-sample RSS peak. `REPRO_*` PASS means the defect was reproduced, not that the defect is acceptable. Output: `checks.json`. These checks never call a candidate or model.
