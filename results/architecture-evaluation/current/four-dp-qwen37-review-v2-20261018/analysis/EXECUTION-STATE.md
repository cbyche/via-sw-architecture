# Review v2 execution state

- Lead Sol High agent `/root/poc_runtime_sol`; parent does review only.
- Actual BASIC orchestrator is running in unified exec session **12135**. Poll with write_stdin. Fresh Python for each serial episode; internal structural concurrency only.
- Parent explicitly conditionally authorized all120; conditions met after immutable calibration-v1(12),v2(12),v3(2). No extra approval needed to finish120.
- BASIC phase: `basic-v1`, phase string BASIC. Freeze SHA `50e4116e1fa86807c19abecac6586110e88968087542eececcfc4e379a7cecf2`; binary SHA `8229089b04b66231f58cbb44b35c12950735cfbbad83dfe03993d5d34cd773e9`.
- Baseline source/harness/scorers/fixtures must remain unchanged while BASIC. Pertrial source + pre/post binary hash guards enforced. Old generation and all calibration evidence preserved.
- Model Qwen3.7-plus-2026-05-26, thinking false, $0.40/M input/$1.60/M output; campaign-root atomic shared cap $4. No other actual model jobs allowed.
- `analysis/phase_control.py run basic-v1` resumes only missing exact planned jobs if infrastructure is interrupted. Never rerun completed jobs for performance.
- Last notified:13/120 completed, global221calls/$0.3567772. Read root budget.json for newer actual cost. Runtime estimate1–2hours, provider-variable.

## Next work

1. Continue progress every60–90seconds with actual counts/calls/cost; all120 mandatory. No source tuning for model failures.
2. At DP01 36, run lightweight export, tell owner to finish all108 final-text manual review, send parent all options + failures/coverage/cost. Parent chooses tactics.
3. At DP02 60 total, create `analysis/dp02-explanation.md`: all6QAs; casesE1nonoverlap/E2K2mixedcontinue+shortnew/E3sameK1/E4correction+Agentquestion+answerreceipt. Perstimfirst + pergoal latestcompletion, actual policywait, admission/queue combined, HTTPsend→terminal overlap, calls/tokens/cost/recovery/N-R/content. QA02 N/A. Parent edits docs15 only.
4. At DP04 84 total, screen owner actual target/geometry review + explanation; no tuning. At DP05 120 total, handoff owner actual conditional semantic review.
5. After120 parent selects optional tactics. No tactic calls before parent selection. Existing A2scratch not automatically selected.
6. Final exports and standalone replay, independent audit, reports to parent. No commits/push.

## Derived tools (outside frozen source)

- `python3 analysis/export_details.py basic-v1` (use full paths from campaign root): calls frozen-population analyzer then presentation-only exports. Preserves original analyzer report and provenance. Includes case/individual/minmax/median, paired endpoint coverage, goal/response/failure/preparation/policywait/stagecalls/memory CSV, separate pinned DP01/05 equivalence. DP02 requiredrevision/receipt scorer unchanged.
- `analysis/build_replay.py <campaign-root>` and `analysis/replay_template.html` by screen owner; builds demo.html with embedded trace/PNGs. Final invoke after all required phases. No model calls. Current only bounded static checks; browser rendering still to verify.
- `qa04/summary.json` four equal-weight tasks C01/C03 + functional2. Main counts01A/B1.75 C2;02AB2;04AB2.5;05A/C1.5 B1.75. Feature means separate. DP04 v3 real scratch acceptance done; all other relevant source modules byte-identical per transition proofs.
- Independent analysis reviewer files `analysis/metric-review-v2/` 9+5 correctedchecksPASS.

## Interpretation cautions

- Callback DISPLAY_PROXY explicitly accepted; no physical UI claim. Actual adopted intention progress vs detailed completion separate. First=final legitimate for directcomplete/DP04grounding.
- Missing coverage fixedQA01=261/QA03=243. Conditional means cannot establish winner if missing. Wrong meaningful response times preserved, no invented timeout penalty. Failed-attempt callback times separate.
- `errors` is contract/completion failure episodes, not program crashes; external report/UI relabel and raw failure cause list separate.
- `enqueue_ns→send_ns` includes bridge serialization/recording/admission plus semaphore wait. No separate slot-acquire timestamp; do not call it pure slotqueue. Rust POLICY_WAIT is measured separately.
- DP05 QA03 scoped API-boundary receiver-shim proxy excludes its internal HTTP dependency including network; full wall + excluded interval adjacent, local serving included. No pure inference/network split claim.
- RSS same-sample Python+Rust, freshpertrial,50ms nominal with actualgaps. Show same-run initial/delta/perrole and separate logical retained_evidence buffers. Logical serialization bytes do not partition RSS; no product512MB claim.
- DP01 initial B E1q2 quote exact text but wrong source line numbers(2/3 claimed3/4), bounded failure preserved. A E2q3 added false appointment date despite marker pass; manual semantics separate.

## Additional export/readiness notes

- Derived tools now include `analysis/dp02_case_analysis.py` (run only after core export; groups actualfirst/goal/stepwait/provideroverlap/calls/tokens/cost).
- Parent wants median/min/max and individual3values, same-endpoint paired observation counts, all rejection/finalfailure timestamps and raw causes. export_details.py includes these. Source remains frozen.
- Source-span failure labels must not imply process crash. Parent approved external presentation relabel only.
- Replay owner expanded standalone actual request system/user text/settings, raw providerresponses, dedup PNGs. Final invoke `build_replay.py <root> --include-calibration`; initial selected phase BASIC. Full 50ms arrays stay in raw/CSV, not all in HTML.
- Important DP02 causal review to verify from actualfuturetraces: U2arrives120ms while first semantic/adoption stage may still occupy K1. B U2may already queue before worker's detailedresponse is enqueued, so do not assume old-generation 'longdetailblocksU2' explanation. Astepwait versusBslot admission must use actual predecessor/order/HTTPspans. Do not alter timings to manufacture a gap.

## Update after DP04 complete / DP05 running

- BASIC unified exec session remains `12135`; last status91/120,global555calls,$0.9233876 known,unknown0. Use phase_control status for current truth.
- DP02 explanation ready `analysis/dp02-explanation.md`; doc15 DP02 facts checked clear.
- DP04 complete24: A first10/15,QA02 17/24,final9/12 (free3 failures; correction initiallyfailed2 but finalrev2 all3complete); B15/15,24/24,12/12. Source/callback intact.
- Parent approved **27 post-BASIC**: DP04 A4/B4 eachsame4cases×3(24); DP01 A2 E3×3. No extra02/05 tactic.
- `analysis/tactics_control.py` prepared, not executed. prepare/run refuse until exact120. ActualDP04variantA4/B4. A2 copies exactBASICcrate+onlyapproveddp01.rs,pins effective-candidate-source,binary andtests/offline. AllAPI uses same root$4ledger.
- `analysis/tactic_comparison.py` ready, compares only matchingcase×3; all27afteractualrun.
- `analysis/memory_option_summary.py` exports same-time peak/roles/initial/delta/logical categories/raw; rerun after final. DP05 currentlypartial; JSON includes trials counts.
- `analysis/campaign_totals.py` ready globalphase/UUID/cost/tokens/elapsed reconciliation; run atfinal.
- Screenagent finishing24DP04 explanation and finalaudit with effectiveA2source-map support; noheavycompilewhileBASIC.
- Evidenceagent DP01 manual108bodyreview complete; DP05 actual54bodyreview after full36.
- Final:export_details all actualphases;memorysummary;campaign_totals;tactic_comparison;build_replay.py root --include-calibration;askscreenfinalaudit+UI. No candidate changes/quietmodelrerun.

## Canonical post-BASIC runner update and extra9 authorized

- Root additionally approved DP05 E4×ABC×3 compact original-line input representation diagnostic ONLYafter120+27; same $4, guard/oracle/cap/recovery unchanged. Final actualepisodes182=26cal+120BASIC+27tactics+9line-diagnostic.
- **Execute only canonical `benchmark/architecture/four_dp_poc_v2/post_basic/`** scripts. Results analysis same-name preparation files are historical/provenance and MUSTNOTbe executed.
- Canonical tactics: `python3 benchmark/architecture/four_dp_poc_v2/post_basic/tactics_control.py prepare|run dp04-A4|dp04-B4|dp01-A2` (after120guard).
- Canonical diagnostic: `python3 benchmark/architecture/four_dp_poc_v2/post_basic/line_number_diagnostic.py prepare|run --root ABS_CAMPAIGN` (after120+27guard).
- New canonical4files are added to per-new-phase source manifest and snapshot. Existing BASICsourcehashesallverifiedunchanged afteraddition.
- Compact formatter is Python Client.call deep-copy only. Rawsource/host/Rustbinary unchanged. Passage.text gets compact ORIGINAL_NO| prefix; summaries/statements untouched; quote excludesprefix. Originalpayloadfile/SHA and actual transmittedrequest retained. Explicit effective_harness_source pin (phase/frozen/harness worker+formatter).
- Pure11checks+actual32E4r1request fullcoverage PASS, addedtext10268bytes. FullJSON+instruction extra27318bytes. Samebranch9costproxy$.3095352, NOTanactualmeasurement. Superseded duplicate-array $.934 estimate labeledSUPERSEDED.
- `analysis/input_representation_comparison.py` separate matchedE4×3 baseline/diag export. `analysis/finalize_exports.py` nowrequires9too and expects182.
- DP05 E4 A r1 failures actualsource line-off-by-one; initial69081.447041ms/correction78234.059708ms, noquietrerun. Evidenceowner readingallactualbodies.
