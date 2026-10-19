# COMMON9 S11 r1: bounded semantic review

**Status:** completed postlaunch diagnostic. No timing, score, technical eligibility or campaign approval is issued by this review.

- Reviewer: GPT-6 Astra Ultra (`/root/method_astra_review`).
- Scope: frozen block ordinal 4, job indexes 28–34, all seven configurations, 35 response obligations and 21 linked goal-relation obligations.
- Detailed immutable review: [COMMON9-S11-r1-v1.json](COMMON9-S11-r1-v1.json), SHA-256 `cbb02dfa2c03d2407b0dce016a6301f54dcdc1307770ed409aee3e6c2c5c8312`.
- Accepted bundle manifest SHA-256 `1ca47352c63165bed1f2f484767205836039ab0fc45d453c6167e773abfc442f`.
- Index SHA-256 `83a27590ff9471342b36247ba3edc1c13a3670863def8b7de2038b162e3f81e1`.
- Extraction receipt SHA-256 `dad29aeadaba2e47145360759256812e84c47f194df4a3e1cf630a1aee54e680`.
- Formal freeze SHA-256 `0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd`.

## 1. Scope and result

The reviewer read the accepted bundle and exact bound fixture, oracle, source and small model request/response files. No new raw trace was read. No evaluator, analyzer, candidate, parser, build, test or API was executed. Source and raw model hashes were checked against their recorded references. Earlier outcomes were not inherited.

Frozen contract 19 §5, line 99, keeps actual content, Task and approval diagnostics separate from observed callback timing. This review follows that separation. A raw callback is not automatically a correct answer or a certified primary endpoint.

| Diagnostic | Result |
| --- | --- |
| Actual displayed body semantics | 29 PASS, 0 observed-body FAIL, 6 UNREVIEWED because no body was displayed |
| Required response delivery | 29 observed, 6 missing U03 responses |
| Current user-goal association | 15 PASS, 6 UNREVIEWED because U03 was not adopted |
| Required goal fulfillment | 15 recorded HOST_COMPLETE with reviewed bodies; 6 missing response and adoption |
| H10 replays | 14 actual callbacks |
| Actual BG progress and result | 14 actual callbacks, separate from user goals |
| Derived H10/H11 relation | CAAA U03 only |
| Approval and new downstream handoff | NOT_APPLICABLE; never counted as PASS |
| Source revision change and stale-result rejection | NOT_EXERCISED; never counted as PASS |

The 21 goal rows describe the same underlying requests and are not 21 additional independent successes. “29 PASS” means the observed bodies satisfy the reviewed content requirements; it does not mean all 35 responses succeeded. All six missing responses and goals remain in the frozen denominators.

## 2. Actual request and source interpretation

The actual inputs are:

> U01: 아까 비용 비교 결과를 다시 보여줘.
> U02: 그 비교 결과를 한 번 더 보여줘.
> U03: 그 결과와 아까 일정 설명의 관계를 정리해줘.

H10 and H11 are authored initial delivered-history sources at revision 1. Their seed display was not measured in this run. Their text is:

> H10: 실제 표시한 비용 비교: Cedar 월96달러, Birch 월64달러. Birch가 월32달러 저렴하다.
> H11: 실제 표시한 일정 설명: Cedar는10월20일 시작해10월30일 완료한다. Birch는11월3일 완료한다.

All seven configurations actually redisplay the exact H10 text for U01 and U02. The request is to show the prior result, so replacing it with the unrelated H12 answer or BG facts would be incorrect; neither replacement occurs in these displayed answers. U02 remains a separate follow-up goal, even where its plan points to U01 as context.

CAAA U03 actually displays:

> 비용과 일정의 관계는 다음과 같습니다: Birch가 Cedar보다 월 32달러 저렴하며(월 64달러 대 96달러), Birch의 완료일(11월 3일)은 Cedar의 완료일(10월 30일)보다 늦습니다. 즉, Birch는 더 저렴하지만 완료 시기는 더 늦습니다.

This relation is supported by the actually acquired H10 and H11 v1 passages. It derives a cost/schedule trade-off; it does not revise either source or claim a changed Task artifact. No source-update stimulus occurs in S11. Original-versus-revised-source behavior therefore remains NOT_EXERCISED. No oracle-only extra requirement or oracle/input mismatch is established for this block.

## 3. Six missing U03 responses: exact support failure

The failed proposals often quote the initial H10/H11 text exactly. Calling those words fabricated or misspelled would be inaccurate. The mismatch is between the claimed source identity and the originals actually supplied for that request.

| Configurations | U03 acquisition response and actual supplied originals | Observed proposal failure |
| --- | --- | --- |
| CAAA | H10 v1 line 1; H11 v1 line 1 | Both acquired and cited; U03 adopted and displayed |
| ABAA, AABA, AAAC | CONVERSATION-LOG v1 lines 1–200; DOC-A v1 lines 1–5 | Proposals keep at least one H10/H11 claim under an original-source ID absent from supplied passages |
| AAAB, BAAA | H10 v1 line 1; CONVERSATION-LOG v1 lines 1–200 | H11 claim names a separate unprovided source, although its words appear in the supplied conversation log |
| AAAA | H10 v1 line 1; CONVERSATION-LOG v1 lines 1–200 | Attempt 1 cites unprovided H11. Attempts 2–3 cite CONVERSATION-LOG but omit that source from `source_revisions` |

The acquisition model responses request these exact ranges; their selected ranges match the actual meaning-request payloads. The candidate did not silently drop a requested H11 read. A source catalog spelling is not a read or authority grant. A fact carried in CONVERSATION-LOG must be claimed against that supplied source/version/range, or the separate original must actually be acquired.

Frozen `runtime.rs` lines 798–808 require the source revision map entry and a supported quote from a supplied passage. `evidence.rs` lines 23–40 match source ID, revision and contained line range before slicing text. The observed rejections follow those checks. The common prompt explicitly requires facts from actually supplied originals with exact quote claims and allows three meaning calls in total.

All six failed U03 requests exhaust those three attempts. Their raw model answers are retained as unadopted proposals, not counted as delivered semantic successes. The exact packet references, decoded source claims, selected ranges, repair reasons and rejection events are in `raw_U03_support_diagnostics` in the JSON. The scoped classification is ordinary source-support/protocol noncompliance followed by missing output. No new validator, fixture or instrumentation contradiction is established.

## 4. Actual BG facts and authority

The fixture supplies an existing Task BG in RUNNING state with `allowed_actions: ["observe"]`. A01 is its actual PROGRESS fact “다른 Task BG의 확인된 진행 50%”; A02 is its actual RESULT fact “다른 Task BG의 최종 파일 완료”. Every configuration captures and displays the matching text, class and Task ID.

The receipt-associated state markers use `phase: ACTUAL_AGENT_FACT` and `task_fact_state_recorded: true`. They do not fabricate a user goal or prove completion of U01–U03. The generic display `content_type: TASK_QUESTION` is retained as raw metadata; it is not evidence of an approval question. The final result envelope does not expose a complete BG Task snapshot, so this review limits the state conclusion to the recorded fact path rather than inventing final Task fields.

The 15 adopted user responses are DIRECT, task null, revision 1, using locally allowed sources. Their permission/control/allowed/completed action lists are empty. Final ingress is empty in all seven runs. No approval, new Agent delegation, external sharing or permission-withdrawal scenario is exercised. The frozen newer-same-Task endpoint clause is preserved; no earliest callback or endpoint time was recalculated.

## 5. Exact callback and goal coverage

Cells identify the original trace event index and event ID. Missing means no matching actual displayed body. The JSON binds every event back to its original trace SHA and every obligation to its frozen oracle and worklist row.

| Configuration / run suffix | RU01 | RU02 | RU03 | RA01 | RA02 |
| --- | --- | --- | --- | --- | --- |
| CAAA / b16060 | 73 / event-89 | 133 / event-160 | 198 / event-237 | 28 / event-37 | 45 / event-57 |
| ABAA / 0aa9de | 51 / event-61 | 86 / event-100 | Missing | 10 / event-13 | 27 / event-33 |
| AABA / 93fbef | 51 / event-61 | 86 / event-100 | Missing | 10 / event-13 | 27 / event-33 |
| AAAB / 925393 | 71 / event-87 | 134 / event-161 | Missing | 30 / event-39 | 47 / event-59 |
| AAAC / 8ea7f2 | 51 / event-61 | 86 / event-100 | Missing | 10 / event-13 | 27 / event-33 |
| AAAA / 0bea6e | 51 / event-61 | 86 / event-100 | Missing | 10 / event-13 | 27 / event-33 |
| BAAA / b5a847 | 48 / event-58 | 82 / event-97 | Missing | 10 / event-13 | 27 / event-33 |

| Configuration | G01 association / recorded completion | G02 association / recorded completion | G03 association / recorded completion |
| --- | --- | --- | --- |
| CAAA | PASS / 78 / event-94 | PASS / 138 / event-165 | PASS / 203 / event-242 |
| ABAA | PASS / 56 / event-66 | PASS / 91 / event-105 | UNREVIEWED / missing |
| AABA | PASS / 56 / event-66 | PASS / 91 / event-105 | UNREVIEWED / missing |
| AAAB | PASS / 76 / event-92 | PASS / 139 / event-166 | UNREVIEWED / missing |
| AAAC | PASS / 56 / event-66 | PASS / 91 / event-105 | UNREVIEWED / missing |
| AAAA | PASS / 56 / event-66 | PASS / 91 / event-105 | UNREVIEWED / missing |
| BAAA | PASS / 53 / event-63 | PASS / 87 / event-102 | UNREVIEWED / missing |

G01, G02 and G03 are respectively U01, U02 and U03. Goal association PASS is a semantic linkage judgment. Recorded completion events do not certify DAG timing, technical eligibility or audio behavior. Those remain outside this review.

## 6. Conclusion and limits

No new technical defect is established by this bounded S11 review. Preserve the six missing U03 responses, all 18 rejected proposals and their costs. This finding does not warrant prompt tuning, oracle changes, reruns, metric filtering or stopping the fixed campaign. It does not establish a DP winner or general superiority from one repetition.

The next ordinal is 5. This artifact does not authorize extraction or semantic review of that block, and does not advance any cursor. Frozen sources, inputs, contracts, review bindings and runtime remain unchanged.

### Frozen references

- `benchmark/architecture/common9_poc_20261019/fixtures.json` — SHA-256 `529d8f41cd72bd861e84475697c5af3cd64e070f559801e71de1d715c83c6d79`. Exact absolute path and byte size are retained in the JSON binding.
- `benchmark/architecture/common9_poc_20261019/oracles.json` — SHA-256 `74b434872444c2a303e0f1bcee1ba162e519386a82620c1a74c2666142f5da52`. Exact absolute path and byte size are retained in the JSON binding.
- `prototypes/candidates/common9_poc_20261019/src/runtime.rs` — SHA-256 `d96f61406119d870bf65bb9045a8fe2bb8eb1aa68ae8319a65a09845a4437402`. Exact absolute path and byte size are retained in the JSON binding.
- `prototypes/candidates/common9_poc_20261019/src/evidence.rs` — SHA-256 `7a2a4f53e45c65d57efe2d08ab1bd18b4be27ae5b5f08dc38a6d19c671c16ab9`. Exact absolute path and byte size are retained in the JSON binding.
- `docs/architecture/12-decisions/decision-packages/seven-problem-review/19-common9-evaluation-contract.md` — SHA-256 `7b3db7b8f358ee6cd441ca37ef13f9b83d1f697322729a0f7bde802bbdc162e7`. Exact absolute path and byte size are retained in the JSON binding.
