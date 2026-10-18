# Final independent evidence and replay review

Reviewed after the runtime's finalization-complete signal on 2026-10-18 KST. No model calls, candidate edits, fixture edits, oracle edits or reruns of experimental episodes were performed. This is an evidence consistency review, not a claim of product or content correctness.

## Reproducible derived checks

```sh
python3 results/architecture-evaluation/current/four-dp-qwen37-20261018/audit_campaign.py --final
python3 results/architecture-evaluation/current/four-dp-qwen37-20261018/audit/check_offline_bundle.py
python3 benchmark/architecture/four_dp_poc/check_analysis.py results/architecture-evaluation/current/four-dp-qwen37-20261018
python3 benchmark/architecture/four_dp_poc/check_dp04_identity.py
```

- Final campaign audit: [final-20261017T195221Z.json](final-20261017T195221Z.json), `PASS_WITH_LIMITATIONS`, 32 PASS, 1 LIMITED, 0 FAIL.
- Final standalone static check: [offline-bundle-check.json](offline-bundle-check.json), all 11 checks PASS.
- Analysis regressions: 8 passed; DP04 target-identity regressions: 14 passed; zero model calls.
- Primary populations are exactly 120 BASIC, 12 TACTIC and 3 DIAGNOSTIC, plus 4 separately labelled SMOKE. BASIC has 120 unique expected keys, no missing, duplicate or unexpected keys. Each of the five before/after comparisons uses the same episode with three baseline and three supplementary repetitions.
- BASIC uses the declared original 84 and DP05-v2 36 populations. Critical original source, fixture, oracle, image and bridge pins are unchanged; v2 source pins are unchanged. All recorded original binary hashes agree with the current original binary; all v2 hashes agree with its pinned binary. **The original freeze did not independently pin a binary before execution**; source pins and consistent recorded/current binary hashes are separate evidence, not a retrospective pre-run binary freeze.
- 579 unique global provider calls have request/response files and known provider usage, fixed model/settings, terminal ledger entries and one shared budget. Total standard-price usage is **USD 0.720764**, not an invoice amount. Presence-only persisted-secret scan found zero matching headers/keys/patterns across 1,436 files; no values were emitted.

## Preserved failures and scope limits

- DP02 B's six missing displayed responses/completions remain missing; they are not zero-latency successes. Frozen N-R remains 21/27 for both basics. The 21/21 unambiguous post hoc subset is a separate diagnostic; mixed continue-plus-new-question wording prevents claiming all six frozen mismatches are clear model relation errors. Actual TCP/UDP content omission count is zero for each basic.
- DP04 basics remain A = B = 15/24. Dense and free-region failures remain failures after source-backed identity normalization; no oracle matching or fuzzy identity repair was used.
- A4 retains two runtime failures (repetitions 1 and 3): actual HTTP 200, `finish_reason=length`, 1,700 output tokens, JSON decode failure, no adoption/display. The successful repetition also fails both referents: total 0/6. The supplementary 35,066.85 ms QA01 average has only one observed response and must be read with the two missing responses. All producer and failed-call usage remains charged.
- B4 is DIAGNOSTIC only: dense 2/6 versus its own BASIC dense 0/6, same total cost USD 0.0096588 over three runs. Speech/screen calls actually overlap by approximately 5.01, 5.47 and 5.35 seconds; this supports a bounded scheduling sensitivity observation, not a general structure accuracy/latency claim.
- QA06 has five BASIC trials without simultaneous Python/Rust observations (DP05 A:2, C:3). Observed process-pair identifiers do not repeat; missing observations do not establish process absence or memory zero. Sampling covers the scoped Python bridge plus Rust process and excludes browser/cloud weights; the mean module-change QA04 remains a scoped actual-change tie of 1.5.
- The frozen DP01 E3 content conditions were not part of the primary QA calculations. All 12 applicable displayed responses (9 BASIC plus 3 A2) omit all three required fact markers (0/36); forbidden literal hits are zero. These are separately disclosed frozen conditions, with exact-string/paraphrase limits. Other runs have NOT_DEFINED content checks, not passes. GOAL_COMPLETE is an instrumented event and does not prove that these conditions were met.
- A2 source reads increase 6→9 across its matched three runs, input tokens and cost increase, and content omissions remain. A1 improves the bounded side question while original R1 varies, including a regression in repetition 3. C2 verifies cache reuse but does not establish a material end-to-end benefit. Final metadata correctly selects basics A/A/A/C, adopts only bounded DP02 A1, does not adopt A2/A4, holds C2 and separates B4.

## Standalone bundle versus browser observation

The requested direct Chrome `file://` opening was blocked by the browser URL security policy (only HTTP/HTTPS permitted). No workaround was attempted. **Actual offline browser rendering is therefore unverified.**

Static standalone checks establish:

- 6,764,358 bytes, SHA256 `8b5f850bf59d2455cf63e8799feff6f751f63b1b2e991b4ce77c8b008ff0b2a4`.
- 139 embedded input/event traces, equal to durable raw records, with final summary equality and all 12 content-condition entries.
- Five embedded actual PNGs match frozen files by SHA256.
- All 578 episode-call raw response JSON objects are embedded and equal to persisted provider responses. The one additional global call is ACCESS_SMOKE (USD 0.000034), outside the 139 episode records; it is preserved in the global ledger/raw files and not claimed to be a replay episode.
- The render script is the same reviewed live script with only offline data/trace/image substitutions; zero fetch dependencies and no external scripts/stylesheets.

Separately, Chrome at the already existing `http://127.0.0.1:8768/` was visually and interactively reviewed:

- Final table shows BASIC/TACTIC/DIAGNOSTIC/SMOKE labels, correct run counts, missing response/completion and runtime error columns, and separate content-condition counts/NOT_DEFINED.
- Selecting DP01 A E3 loads the actual two displayed referring phrases and ALL_REQUIRED_FACT_MARKERS_ABSENT alongside preserved GOAL_COMPLETE events.
- Selecting A4 dense repetition 1 loads the actual 1000×650 PNG, input JSON, JSON error result, and expandable model-finish payload containing the actual provider response, usage and length finish reason. Image and trace are readable in the two panels.
- Selecting DP05 v2 C E2 loads input, actual preparation/result text, receiver receipts and expandable provider result JSON from the separate v2 path (HTTP 200, actual fixed model, stop finish reason, usage).
- Header correctly states Voice OFF and DISPLAY_PROXY / recorded replay.

No concrete calculation or replay display bug remained in this final review. Document prose/link validation is separate and awaits the parent's final document-ready signal.
