# DP1/DP4 sidecar stage04 independent static closure

**Verdict: the exact host PROCESS_STDOUT_CLOSED compatibility repair is CLOSED by static inspection. No new material P1/P2 regression found in the bounded change.** Reviewer: GPT-6.1 Sol High, `/root/sol_high_continuation/sidecar_closure_sol`, 2026-10-20 KST. This report preserves stage03 and does not authorize actual v2 extraction.

## Bound artifacts

The complete stage04 README, checkpoint, saved synthetic test source, saved result and saved metadata-preparation failure were read. The complete stage03→stage04 source diff and surrounding extraction/interruption/finalization paths were inspected. Stage03's independent closure was read as the prior review boundary. Active source and the stage04 copy have the same 51,756 bytes and SHA-256.

| Artifact | SHA-256 |
| --- | --- |
| Active and copied stage04 source | `dc3d96a70cc8de27582455edfda47525be78458bbc0069e379d7411287f38a9e` |
| Stage04 CHECKPOINT.json | `58b3f3fac29c5c45500435bfabb7d3b623e3557d543e7befba3834102557ca3f` |
| Stage04 README.md | `2c05dcdab9c5d3c7ab6701f9596e11b6a52977639c7a9f591963c5ad3a0d13da` |
| Stage04 test_fake.py | `454d2203b0aa483ab25af85295e23f64277960c6a8b6c310c5ad4dd147e266e9` |
| Stage04 fake-results-attempt-01.json | `4fbd81be475a42c144a6809341af14b95874d40659e62ecd32b847aff33d17b4` |
| Stage04 metadata-preparation-attempt-01-failure.json | `e6c684af7fab2c9642b784785f6fd8b3b75744c0a4ad8581bd3a23e217422704` |
| Preserved stage03 source | `fdea28eef16c61a10e731e3c67006b28c0b547619db04d63c9796d8fec43fdc6` |
| Preserved stage03 static-sol-closure-stage03.md | `1637e3f4e8851800631f52a288a88208c5970379b1a17bb6bf9a24ff357688ea` |

## Closure reasoning

The source diff has only the v1→v2 output version change and the six inserted lines at384–389. The compatibility branch accepts a dict with exactly two keys, `kind` and `observed_ns`, exactly `PROCESS_STDOUT_CLOSED` as kind, and a nonnegative clock whose exact Python type is int. It then continues the event loop. This matches the narrow normal host marker documented in this preparation stage without treating it as a candidate event requiring a data object.

The exact-key predicate rejects extra keys, including `data:null`; exact `type(...) is int` rejects Boolean clocks as well as floats/strings, and the comparison rejects negative integers. An unknown data-less kind or other malformed event falls through to the unchanged invalid-event branch at390–393, which marks every collection UNSUPPORTED/INTERRUPTED and retains an exact event pointer. Previously extracted facts remain present. A later valid marker cannot undo an earlier interruption, and a later malformed event still interrupts after the marker. `finish_collection` at252–258 still preserves INTERRUPTED state. These facts retain MS2-01's fail-closed repair.

The marker is not a certification shortcut: terminal presence is still separately determined from exact integer `end_ns` and `returncode` at369–370; trace terminal facts retain host/technical failures at373–374; `trace_complete_for_absence_claims` remains false at375. Ignoring this normal lifecycle marker cannot by itself certify inactivity, semantic use, causal savings, successful measurement or product completion.

The entire diff leaves the CAPS assignment byte-for-byte unchanged, including16MiB traces,1MiB requests,100 selected requests/run,16GiB aggregate input,2MiB projected rows and2MiB metadata. This static comparison independently supports the checkpoint's unchanged-caps claim without evaluating code or running an AST/domain parser. The prior capped rows have no increased allowance. Existing identity, association, accounting, bounded reader, collection failure and all-planned-row behavior are unchanged by this repair; this is not a new full-source re-audit.

## Saved synthetic evidence and preserved failure

The saved test source has eight in-memory cases: exact marker supported; null data, Boolean/negative clock, unknown data-less kind, extra key, later invalid event and candidate data:null remain unsupported. Failure assertions check every collection's UNSUPPORTED/INTERRUPTED coverage and the preceding source-read fact. The supported case checks COMPLETE/FINISHED collections, preserved trace reference and absence-claim flag remaining false. The suite calls `extract_run` directly on authored Python objects, not the extractor CLI or campaign inputs.

The saved result binds the reviewed source, reports8 tests,0 failures/errors,0.0003657909110188484 seconds,0 campaign-input reads,0 CLI runs,0 model calls and0 expected fake input files. These are worker-recorded preparation results; this reviewer did not rerun them. The metadata-preparation failure was also read and remains preserved: `ast.literal_eval` rejected arithmetic in CAPS after the source copy had completed. Its recorded repair was comparison of unevaluated original AST expressions, with no extraction or repeated tests. No failed preparation is converted into campaign evidence.

The stage04 README's255 unsupported rows and3 capped rows are historical preparation context reported by the owner; this static review did not read or independently recount actual v1 output. This review does not promise that a v2 run will make those rows COMPLETE or clear any other unsupported shape.

## Limits and handoff

During this stage04 review, actual extraction, synthetic-test rerun, analyzer, domain parser, compiler/build, candidate, model/API/network, controller operation and campaign input reads were all **0**. Only named static preparation/source files were read. No source, checkpoint, saved result, failed v1 output, frozen measurement artifact, contract, endpoint, eligibility, population, QA denominator, prior review or presentation was edited. The sole new artifact is this immutable review.

The checkpoint records actual stage04 extractor CLI runs0 and a separate exact GO pending. This closure does not issue that GO. A fresh exclusive v2 output, preserved v1 failure output, original caps and prior evidence remain the required execution boundary for any separately authorized run. No retry or cap increase follows from this review.
