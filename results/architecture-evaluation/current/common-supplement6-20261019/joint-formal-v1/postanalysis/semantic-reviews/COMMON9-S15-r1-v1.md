# S15 r1 bounded semantic review

Reviewer: GPT-6 Astra Ultra (`/root/method_astra_review`). Status: postlaunch diagnostic only. Scope is ordinal8 `PRIMARY-r1-S15`, seven configurations. No primary QA metric was reevaluated.

Full immutable evidence: [COMMON9-S15-r1-v1.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/semantic-reviews/COMMON9-S15-r1-v1.json) (SHA256 `daeb3d1c015bfa2aac2c816019295de8d47cc387376ad56d1b46842b4e9c9f54`). It includes all28 response rows and21 goal-relation rows, actual body text, consumed meaning-input history, adopted plans, source reads, ingress, receiver results, matching host receipts and state references.

## Counts and what PASS means

| Dimension | Scoped result |
| --- | --- |
| Captured inputs | 28:21 user inputs and7 actual Agent RESULT inputs |
| Distinct raw callbacks | 28, with28 matching host display receipts |
| Body diagnostics | 28 PASS,0 FAIL,0 UNREVIEWED within the scopes below |
| Body PASS scopes | 7 P1 explanations,7 TCP explanations,7 concrete handling notices,7 faithful synthetic-result relays |
| Goal/Task association | 21 PASS;21 final Request states observed as HOST_COMPLETE |
| Actual downstream handoff | 7 ingress events and7 accepted bounded receiver results |
| Full presentation/PPTX realization | 7 UNREVIEWED; not established by the result-relay PASS |
| Preparation failure | AAAB1, retained with raw truncated output |
| Literal R15 field | 7 NONMATCH; naming requirement UNRESOLVED against actual generated result |
| Approval | Not applicable; no approval PASS credited |
| Concurrent stale-revision rejection | Not exercised |

A goal association row is not an additional independent success. HOST_COMPLETE and callback metadata do not establish full semantic correctness by themselves; the actual content and state relations were inspected. All raw failures, costs and fixed denominators remain.

## Actual requests and historical referent

1. U01: `이 문단의 핵심을 설명해줘.` Actual native selection names DOC-A v1 P1, line4: `P1 | 핵심 설명 | 준비된 파일은 10개이며 결론을 먼저 제시한다.`
2. U02: `잠깐 다른 질문이야. TCP가 뭐야?` This is a separate question, with no paragraph target.
3. U03: `아까 문단 설명으로 발표자료 파일을 만들어줘.` This asks for a new work product from the earlier paragraph explanation, after the TCP detour.

Initial historical_delivered is empty. The relevant earlier explanation is actually displayed during U01 in this same episode. Every U03 meaning request contains both actual U01/P1 and U02/TCP delivered history, and actual P1 original text. The adopted work purpose uses P1 rather than TCP. Every initial Task set lacks TASK-5; each run subsequently adopts new TASK-5 for goal:S15-U03.

Six U03 proposals use NEW. CAAA uses FOLLOWUP with target_goal_id=goal:S15-U01 and target_event_id=S15-U01, while actual final state still has separate goal:S15-U03 and TASK-5. U01 and U02 are preserved as independent completed goals. AABA U01 uses ANSWER with a self-event identifier and likewise resolves to its own U01 goal. Association judgments use actual state and source relationships, not equality of relation-label spelling.

## All28 response rows

Each listed body is observed and scoped PASS. RU03 means a concrete handling intention, not a finished artifact; RA01 means faithful relay of the actual bounded synthetic result, not verification of a physical presentation. Event indexes are zero-based.

| Config | Obligation | Raw display | Adopted source/current goal | Body scope |
| --- | --- | --- | --- | --- |
| AABA | S15-RU01 | `event-44` `/events/37` | `goal:S15-U01` r1 | P1 core explanation |
| AABA | S15-RU02 | `event-82` `/events/71` | `goal:S15-U02` r1 | Independent TCP explanation |
| AABA | S15-RU03 | `event-121` `/events/106` | `goal:S15-U03` r1 | Presentation handling intention |
| AABA | S15-RA01 | `event-166` `/events/145` | `goal:S15-U03` r1 | Actual bounded result relay |
| AAAB | S15-RU01 | `event-62` `/events/52` | `goal:S15-U01` r1 | P1 core explanation |
| AAAB | S15-RU02 | `event-118` `/events/100` | `goal:S15-U02` r1 | Independent TCP explanation |
| AAAB | S15-RU03 | `event-184` `/events/157` | `goal:S15-U03` r1 | Presentation handling intention |
| AAAB | S15-RA01 | `event-232` `/events/199` | `goal:S15-U03` r1 | Actual bounded result relay |
| AAAC | S15-RU01 | `event-42` `/events/35` | `goal:S15-U01` r1 | P1 core explanation |
| AAAC | S15-RU02 | `event-80` `/events/69` | `goal:S15-U02` r1 | Independent TCP explanation |
| AAAC | S15-RU03 | `event-119` `/events/104` | `goal:S15-U03` r1 | Presentation handling intention |
| AAAC | S15-RA01 | `event-159` `/events/139` | `goal:S15-U03` r1 | Actual bounded result relay |
| AAAA | S15-RU01 | `event-42` `/events/35` | `goal:S15-U01` r1 | P1 core explanation |
| AAAA | S15-RU02 | `event-80` `/events/69` | `goal:S15-U02` r1 | Independent TCP explanation |
| AAAA | S15-RU03 | `event-119` `/events/104` | `goal:S15-U03` r1 | Presentation handling intention |
| AAAA | S15-RA01 | `event-164` `/events/143` | `goal:S15-U03` r1 | Actual bounded result relay |
| BAAA | S15-RU01 | `event-39` `/events/32` | `goal:S15-U01` r1 | P1 core explanation |
| BAAA | S15-RU02 | `event-74` `/events/63` | `goal:S15-U02` r1 | Independent TCP explanation |
| BAAA | S15-RU03 | `event-113` `/events/97` | `goal:S15-U03` r1 | Presentation handling intention |
| BAAA | S15-RA01 | `event-158` `/events/136` | `goal:S15-U03` r1 | Actual bounded result relay |
| CAAA | S15-RU01 | `event-58` `/events/48` | `goal:S15-U01` r1 | P1 core explanation |
| CAAA | S15-RU02 | `event-116` `/events/98` | `goal:S15-U02` r1 | Independent TCP explanation |
| CAAA | S15-RU03 | `event-180` `/events/153` | `goal:S15-U03` r1 | Presentation handling intention |
| CAAA | S15-RA01 | `event-225` `/events/192` | `goal:S15-U03` r1 | Actual bounded result relay |
| ABAA | S15-RU01 | `event-42` `/events/35` | `goal:S15-U01` r1 | P1 core explanation |
| ABAA | S15-RU02 | `event-80` `/events/69` | `goal:S15-U02` r1 | Independent TCP explanation |
| ABAA | S15-RU03 | `event-119` `/events/104` | `goal:S15-U03` r1 | Presentation handling intention |
| ABAA | S15-RA01 | `event-164` `/events/143` | `goal:S15-U03` r1 | Actual bounded result relay |

All U01 bodies preserve10 prepared files and conclusion first. All U02 bodies provide an adequate short explanation of TCP reliable/ordered communication; this is not a formal protocol guarantee audit. All U03 handling bodies identify P1 and presentation creation before ingress; no completed action is declared in the adopted plan. Their English language is retained verbatim in the JSON rather than silently translated or scored under a newly added locale rule.

Every RU03 worklist also links the later A01 display through origin_event_id. The actual block has four displays per run, not five. RU03 reviews the first handling publication; RA01 reviews the actual result publication. These raw associations are not a saved or newly calculated QA01 selection.

## Source access and actual handoff

DOC-A remains version1, with actual local and external-read permission. All paragraph claims match supplied line4 exactly. U02 makes no fabricated DOC-A claim for TCP. No permission operation, control operation or completed action is invented by the adopted plans. CAAA additionally supplies P2 at U03; the claim, intended target and receiver-used content remain P1. That extra acquisition is retained without labeling it adopted P2 content.

| Config | Handoff branch | Receiver path | Actual used material |
| --- | --- | --- | --- |
| AABA | `DIRECT_ACQUIRED_OR_RECORD_OR_REFERENCES`; records `[]` | BOUNDED_ORIGINAL_READ; `event-151` `/events/133` | DOC-A v1 line4 |
| AAAB | `DIRECT_ACQUIRED_OR_RECORD_OR_REFERENCES`; records `['REC-005']` | BOUNDED_ORIGINAL_READ; `event-214` `/events/184` | DOC-A v1 line4 |
| AAAC | `MANDATORY_AND_RECEIVER_READS`; records `[]` | BOUNDED_ORIGINAL_READ; `event-144` `/events/127` | DOC-A v1 line4 |
| AAAA | `DIRECT_ACQUIRED_OR_RECORD_OR_REFERENCES`; records `[]` | BOUNDED_ORIGINAL_READ; `event-149` `/events/131` | DOC-A v1 line4 |
| BAAA | `DIRECT_ACQUIRED_OR_RECORD_OR_REFERENCES`; records `[]` | BOUNDED_ORIGINAL_READ; `event-143` `/events/124` | DOC-A v1 line4 |
| CAAA | `DIRECT_ACQUIRED_OR_RECORD_OR_REFERENCES`; records `[]` | BOUNDED_ORIGINAL_READ; `event-210` `/events/180` | DOC-A v1 line4 |
| ABAA | `DIRECT_ACQUIRED_OR_RECORD_OR_REFERENCES`; records `[]` | BOUNDED_ORIGINAL_READ; `event-149` `/events/131` | DOC-A v1 line4 |

AAAB actually selects earlier valid record REC-005, but its receiver still requests/receives the original P1 range. AAAC supplies mandatory information and receiver reads, with an actual served P1 receipt. Producer completion, a record ID, or an ingress capsule is never treated as receipt or as cost elimination. All7 recorded receiver results are accepted with empty errors and actual P1 facts; actions_performed is empty. These are seven observed handoffs, not fourteen successes from the two related response rows.

## Result name and product-completion limit

The oracle RA01 label says `새 Task 결과 R15 링크`; the fixture template says `발표자료 파일 R15 작성 완료.` The actual captured RESULT and displayed body instead reference `artifact-S15-U03.txt`, TASK-5, the current work purpose and P1. This literal R15 NONMATCH is retained separately. The actual user did not require a filename R15; no new literal-name failure or oracle rewrite is introduced.

Frozen harness.py lines29–31 build the result text from actual accepted receiver output and ingress metadata, with a source titled `Actual synthetic downstream artifact`. The receiver prompt requests a bounded draft or handoff verification from supplied originals. Contract19 §4.2–4.3 defines a common synthetic external-work profile and excludes a commercial Agent product.

Therefore RA01 PASS means faithful source/Task/result relay under that frozen profile. A returned conditions_respected string, a synthetic filename or HOST_COMPLETE flag is not independent evidence of a physical usable slide deck. No file-format, PowerPoint layout, disk artifact or presentation usability check was performed. The seven full presentation-product completions remain UNREVIEWED. The BAAA receiver text explicitly says presentation-file creation completed; that wording is preserved as actual supplied synthetic output and is not promoted to verified product completion.

## Preserved AAAB preparation failure

AAAB event-220 `/events/189` is PREPARATION_FAILED for owner handoff. The raw `/calls/11/response_file`, `a5f50d50-6c62-4393-9499-563f848bee22-response.json`, has finish_reason=`length` and ends in an unfinished unknowns array. JSON decoding reports `Expecting value: line 174 column 55 (char 5022)`. Raw response SHA256 is `aff6ec3471da0c57368ce48451456484b3427963d260387ba5b67ca41fc897a8`.

The earlier REC-005 selection, actual receiver read/result, later records and all resource costs remain distinct. Successful scoped user responses do not erase the preparation failure. The observed length-limited malformed producer output does not establish a new runtime/decoder defect.

## All21 goal relations

| Config | G01 P1 | G02 TCP | G03 new presentation task |
| --- | --- | --- | --- |
| AABA | PASS, own U01 goal | PASS, independent U02 goal | PASS, new TASK-5/U03 and matching A01; product realization UNREVIEWED |
| AAAB | PASS, own U01 goal | PASS, independent U02 goal | PASS, new TASK-5/U03 and matching A01; product realization UNREVIEWED |
| AAAC | PASS, own U01 goal | PASS, independent U02 goal | PASS, new TASK-5/U03 and matching A01; product realization UNREVIEWED |
| AAAA | PASS, own U01 goal | PASS, independent U02 goal | PASS, new TASK-5/U03 and matching A01; product realization UNREVIEWED |
| BAAA | PASS, own U01 goal | PASS, independent U02 goal | PASS, new TASK-5/U03 and matching A01; product realization UNREVIEWED |
| CAAA | PASS, own U01 goal | PASS, independent U02 goal | PASS, new TASK-5/U03 and matching A01; product realization UNREVIEWED |
| ABAA | PASS, own U01 goal | PASS, independent U02 goal | PASS, new TASK-5/U03 and matching A01; product realization UNREVIEWED |

All source and request revisions in these relations are1. Actual current associations are supported, but stale-result rejection and revoked-permission behavior were not exercised. Approval is not applicable to this request. No unrelated background Agent fact was substituted for the requested Task result.

## Limits and next step

No new technical contradiction was established. This review did not run or reuse an evaluator and does not certify QA01 endpoints, QA02 scores, QA03 paths, costs or RSS. No callback was removed because of semantics, no denominator or frozen expectation changed, and no tuning, retry or campaign stop is proposed.

Only the authorized bundle and explicit small frozen/model references were read. Original trace files were not newly read. No whole-corpus scan, new extraction, analyzer/evaluator, model/parser, build/test or network execution occurred. Runtime, inputs, contracts and frozen artifacts were unchanged. Next ordinal9 requires separate root authorization; the cursor is not advanced.

## Immutable provenance

| Artifact | SHA256 |
| --- | --- |
| [Formal freeze](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/freeze.json) | `0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd` |
| [Accepted manifest](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON9-S15-r1-v3-r2/manifest.json) | `22d70ed617f80571eacdc2fe4e5f36ebaf6bca46d9ff68814fbd311ce402fb34` |
| [Evidence](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON9-S15-r1-v3-r2/evidence.json) | `ab3e1eaa15130b080596dba32dae2bce39a01e63be05dbd5765769e8c32889bb` |
| [Worklist](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON9-S15-r1-v3-r2/worklist.json) | `82eec797defb86d5218b6e7a5eac1b90a29e28d931826a3d6cf33e9760cba110` |
| [Index](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/block-indexes/COMMON9-S15-r1-v3-r2/index.json) | `d8d6498395e10d7d6cc642af8c90e6677fd61a8a496668fafc8ccda175c278e6` |
| [Execution receipt](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/review-cursors/COMMON9-S15-r1-v3-r2/EXECUTION-RECORD.json) | `bb94b88e5761c62484bcff0b3a4b8cd9566b0889b39e58669db3bbb0d34cc913` |

Saved receipt: prepare0.18273s, extract0.28414s, issues/omissions0. The JSON retains exact source/model hashes, all event pointers, every observed body and the preparation failure. These saved receipt facts are not a new execution.
