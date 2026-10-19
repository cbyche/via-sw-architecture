# S12 r2 bounded semantic review

Reviewer: GPT-6.1 Sol High (`/root/sol_high_continuation/sidecar_closure_sol`). Independent review of ordinal21, `PRIMARY-r2-S12`, jobs147–153, all seven configurations. Status: postlaunch diagnostic only, not metric/endpoint certification. The prior S12 r1 Markdown supplied method context only; its outcomes were not inherited.

[Final JSON with all14 response and14 goal-relation rows](COMMON9-S12-r2-sol-v1.json), SHA256 `d3aed7f4344ae4c4e8d44f74a52bda49d76ec88df5c069850d958779774f2634`.

## Coverage and judgment

| Dimension | Observed result |
| --- | --- |
| Actual user inputs captured | 14/14 |
| Unique actual displayed bodies | 13/14 |
| Actual-request body semantics | 13 PASS,0 FAIL,1 UNREVIEWED because absent |
| Goal association/currentness | 13 PASS,0 FAIL,1 UNREVIEWED because unadopted |
| Required U02 response and goal unmet | 1, AAAB |
| U01 original DOC-A v1 P1 body | 7 PASS |
| U02 current DOC-A v2 P1 body | 6 PASS,1 absent |
| Source transition observed | 7, including AAAB's supplied v2 input |
| Rejected U02 proposals | 3, all AAAB |
| Workflow failures | 1, AAAB U02 |
| Discarded preparation | 2, AAAB and CAAA |
| Approval and Agent handoff | Not applicable; no success credited |

Every planned response and goal key is retained once. Relation rows concern the same responses and are not additional independent successes. The missing AAAB body receives no invented content verdict; its response/goal fulfillment remains explicitly unmet. No raw callback becomes a certified QA01 selection.

## Actual input, source and displayed facts

U01 is `아까 설명했던 원본 부분을 보여줘.` The actual H20 seed history says DOC-A v1 P1 described ten prepared files and a conclusion-first presentation, with provenance DOC-A/revision1/line4. H20 is authored prior delivered context; its earlier delivery is not newly measured in this episode.

All seven actual U01 meaning inputs acquire DOC-A v1 line4 and all seven display the original:

> P1 | 핵심 설명 | 준비된 파일은 10개이며 결론을 먼저 제시한다.

U02 is `지금 바뀐 그 부분을 다시 설명해줘.` After U01 final, each run records DOC-A SOURCE_REGISTER and SOURCE_ACTIVITY_STATE effects. Those markers themselves contain source identity, not a revision/body. The subsequent actual reads and meaning packets establish the new revision: DOC-A v2 P1 says twelve prepared files while retaining conclusion-first presentation. Each U02 source catalog marks v1 historical and v2 current; its actual prior-delivery context still contains the old ten-file text.

Six U02 bodies correctly give twelve and preserve the conclusion-first fact. AABA repeats the complete updated P1 sentence, and CAAA says `DOC-A v2 P1: 준비된 파일은 12개이며 결론을 먼저 제시한다.` Neither requires an additional old/new comparison sentence to satisfy the actual request. AAAC, AAAA, BAAA and ABAA also correctly compare ten with twelve. No delivered body claims that VIA edited the document.

| Configuration / run suffix | U01 display | Source-register marker | U02 actual v2 read | U02 display | Final goal association |
| --- | --- | --- | --- | --- | --- |
| AABA /117bd0 | `/events/31` | `/events/39` | `/events/59` | `/events/73` | 2 PASS |
| AAAB /ab5253 | `/events/51` | `/events/59` | `/events/124` | Absent | U01 PASS; U02 UNREVIEWED/unmet |
| AAAC /bfa7ad | `/events/31` | `/events/39` | `/events/59` | `/events/73` | 2 PASS |
| AAAA /569d88 | `/events/31` | `/events/39` | `/events/59` | `/events/73` | 2 PASS |
| BAAA /92e2e4 | `/events/28` | `/events/36` | `/events/56` | `/events/67` | 2 PASS |
| CAAA /4cb96d | `/events/47` | `/events/55` | `/events/118` | `/events/129` | 2 PASS |
| ABAA /128522 | `/events/31` | `/events/39` | `/events/59` | `/events/73` | 2 PASS |

These are original event pointers carried by the accepted bundle, not new raw-trace reads. All sixteen actual meaning-request packets were independently inspected. AAAB U02 receives current DOC-A v2 lines1–5, including P1; the other six U02 packets receive v2 line4. Every U01 packet receives v1 line4. These are actual supplied originals rather than catalog-only availability.

## AAAB: correct values in rejected drafts do not provide an answer

AAAB's three U02 proposals at `/events/133`, `/events/138` and `/events/143` contain factually plausible ten-to-twelve explanations, but each declares `source_revisions={"DOC-A":2}` while its source claims include DOC-A revision1 and revision2. Each is rejected with `claim source/version missing from source_revisions map`. The actual supplied current original is v2; the old ten-file text remains in delivered history, rather than a newly supplied DOC-A v1 original. Historical catalog availability does not substitute for that original support.

The three exact raw response files linked by AAAB `/calls/6/response_file`, `/calls/7/response_file` and `/calls/8/response_file` were hash/size checked and read. All have `finish_reason=stop`, retain the same revision-map mismatch and are not truncated JSON outputs. Their completion-token counts are498,502 and495. The second and third actual request packets include the prior validation error as their bounded repair reason. Those were within-run attempts already present in the evidence; this review requests no retry.

The actual system instruction defines `source_revisions:{actual_catalog_source_ID:number}`: one numeric revision per source-ID key. AAAB chose two DOC-A revision claims while that map named only revision2, and the observed validation rejected that particular representation. The other six U02 adoptions claim only current DOC-A revision2; four still explain the correct old/new difference using the actual delivered history. This distinguishes the PoC representation/support-contract restriction and model's chosen claim form from a general inability to handle two versions. This block does not establish that the candidate architecture inherently cannot compare versions or is structurally inferior because of this failure.

`/events/144` records `MEANING_TOTAL_LIMIT_EXHAUSTED: claim source/version missing from source_revisions map`. There is no U02 adoption, display, host completion or final U02 Request. The terminal at `/events/146` retains only `goal:S12-U01`. Thus the correct current source reached interpretation, but the required second response and goal are unmet. Rejected draft facts are not promoted to a displayed answer or a relation PASS. This is a different actual r2 outcome from the method-reference r1 report.

## Separate source revision, goal identity and authority

All thirteen actual adoptions use FOLLOWUP and DIRECT with Task null. U02 contextually targets U01, but its successful adoptions belong to separate `goal:S12-U02` at request revision1, matching the frozen separate G01/U01 and G02/U02 memberships. DOC-A source revision2 is not request revision2 or a same-goal correction. The absent AAAB U02 cannot receive that association judgment merely from its proposed FOLLOWUP label.

Each of the thirteen displays has an actual matching host receipt and recorded GOAL_COMPLETE. Final successful Request records retain matching origin/last event and revision1. Those host facts support the reviewed information response scope, not physical audio or a domain work product. The source update occurs after U01's actual final, so this block does not exercise an in-flight obsolete-answer rejection race.

Local reading is allowed in the actual source registry. All adopted allowed/completed actions, permission operations and control operations are empty, and every final ingress map is empty. No edit, sending, approval or downstream Agent work is requested or observed. Registry permission does not itself prove an external action, and no unexercised authority mechanism earns PASS.

Preparation costs remain visible. AAAB `/events/111` discards activity2 work after activity3 becomes current (`owner=DP4_B`); CAAA `/events/110` discards superseded activity2 work (`owner=DP1_C`). Their complete preparation lifecycle references and failed calls are preserved in the JSON and accepted evidence. No source reuse, producer work or cost is treated as free, skipped or causally saved by this review.

## Provenance and execution limits

| Artifact | SHA256 |
| --- | --- |
| Formal freeze | `0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd` |
| Ordinal21 accepted index | `259e7c785709ed7635e8517ab83ae4587ef12bb7041a5dc35c72240136783182` |
| Accepted bundle manifest | `70bddd2618b583b7c99cc2e1335af3f553b4d287e1b4b514a85381b4a26607a5` |
| Accepted evidence | `1576576ad1d8fbe5b7c50ea86318775fdcd8e38635196b0679dd4422cb29196f` |
| Accepted worklist | `a9493a8edab1b5a6224b07de2d7542a96baf5c0be31b7d27371a9e4252aa4026` |
| Root-bound extraction receipt | `b37342c93a0177c54f1b2c3dd969d49999ef7d44062fbb0dd5a1827deab8fcdf` |

Only accepted block files, the prior method Markdown and exact small model files linked by the accepted evidence were read. Named request files total844,892bytes per named pass; the three selected raw response files total6,588bytes. The named request projection finished in0.01091s, response projection in0.00565s, and final evidence serialization in0.02402s. Every operation stayed below1s and every file below16MiB. Total repeated reads remain below a conservative64MiB, within the128MiB authorization. Some first display projections exceeded the tool's text-output budget and were truncated; narrower projections were subsequently inspected. No file/read-time cap was exceeded and no evidence extraction was repeated.

All seven recorded technical states have returncode0, empty technical_failures, no technical_abort_reason and no unprovided mandatory events. No new mandatory technical-stop condition is established by this bounded review. This does not independently certify clocks, endpoint eligibility or the controller implementation. The ordinary support-contract/workflow failure remains fully recorded.

No original trace, corpus or active `runs.jsonl` was read. No evaluator/analyzer, domain parser, candidate, compiler/build, test, model/API/network or actual extractor was executed. No frozen source, prompt, contract, metric, score, timestamp, denominator, failure or eligibility was changed. No presentation edit, campaign action, staging, commit or push occurred. Missing certified endpoint facts remain unavailable. No structural winner, causal savings or product completion is inferred. Ordinal22 requires a separate exact root GO after these final artifacts are fully read; this review does not advance the cursor.
