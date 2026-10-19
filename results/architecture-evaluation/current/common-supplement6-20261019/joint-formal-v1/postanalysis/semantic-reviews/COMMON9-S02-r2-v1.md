# COMMON9 S02 r2 bounded semantic review v1

## Scope and provenance

Reviewer: `/root/method_astra_review`, GPT-6 Astra Ultra. Fixed ordinal 15, `PRIMARY-r2-S02`, jobs 105–111. This is a postlaunch semantic diagnostic of seven already closed runs. No r1 outcome was inherited. No trace was re-extracted or reread, evaluator/analyzer executed, or frozen artifact changed. All event references below resolve through the accepted bundle to the original trace identities.

| Bound artifact | SHA-256 |
| --- | --- |
| [bundle_manifest](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON9-S02-r2-v3-r2/manifest.json) | `03ada9c7f565b1a051743cab5937b9edbb124b7a294fc6d24f520bde99b31605` |
| [evidence](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON9-S02-r2-v3-r2/evidence.json) | `a47b4d3054de444c22cbde3b95d944be64d3013b1575348ca9aad25af42e3528` |
| [worklist](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON9-S02-r2-v3-r2/worklist.json) | `de8a21af4f4522df50c017bc5b2db19d8c0a4f00f27c77843d6bd6ad14018023` |
| [index](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/block-indexes/COMMON9-S02-r2-v3-r2/index.json) | `0a7e2a8de44ad7cefc00e7f548d91edba292611f4b87388717b34aad9c73f441` |
| [execution_receipt](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/review-cursors/COMMON9-S02-r2-v3-r2/EXECUTION-RECORD.json) | `52f485375ddd417d53da71b9989da8c593245775236af97533d14c0ff060182d` |
| [freeze](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/freeze.json) | `0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd` |
| [fixture](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common9_poc_20261019/fixtures.json) | `529d8f41cd72bd861e84475697c5af3cd64e070f559801e71de1d715c83c6d79` |
| [oracle](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common9_poc_20261019/oracles.json) | `74b434872444c2a303e0f1bcee1ba162e519386a82620c1a74c2666142f5da52` |

Complete machine-readable review: [COMMON9-S02-r2-v1.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/semantic-reviews/COMMON9-S02-r2-v1.json), SHA-256 `334b7f00ef780925a53678c5107fc5557690bc37eb47d9d7eb92fc5e45d71e99`. Its 42 response rows and 42 goal rows bind actual captures, model request/response hashes, supplied originals, adoptions, displays, receipts and final request records.

## Exact coverage and interpretation

| Dimension | Result |
| --- | --- |
| Planned response obligations | 42 |
| Unique actual displayed bodies | 42 |
| Response callback references | 42, all distinct |
| Actual requested body | 42 PASS, 0 FAIL, 0 UNREVIEWED |
| Goal member association/current revision | 42 PASS, 0 FAIL, 0 UNREVIEWED |
| Recorded HOST_COMPLETE request states | 42; not 42 additional body successes |
| Literal descriptive-oracle fields | 28 match; 14 nonmatch with requirement unresolved |
| Ancillary exact-spoken binding instruction | 37 deviations; 5 rows with no binding to assess |
| Auxiliary preparation failure | 1, AAAB; preserved separately |
| Missing, superseded and duplicate actual bodies | 0 each |
| Action approval / outgoing handoff | NOT_APPLICABLE / NOT_EXERCISED |

All 84 planned response/goal keys occur once. A response and its goal row refer to the same actual body; the counts are not added into 84 independent successes. Source-read correctness, outgoing export and action approval remain separate dimensions.

## Actual requests and displayed content

The six requested source bodies and revision-1 local/read rights are present in each run’s actual candidate input. Each meaning request contains the corresponding original, not only source metadata. Its answer equals its own actual displayed text. All seven configurations produced the following text independently in r2:

| Input | Actual request | Actual displayed body | Scoped judgment |
| --- | --- | --- | --- |
| U01 | F01 결론을 두 문장으로 요약해줘. | 작은 읽기 작업을 병합하면 준비 비용을 줄일 수 있다. 기존 권한과 판본 검사는 유지해야 한다. | Actual-request body PASS |
| U02 | 어제 민수가 보낸 M01 메일의 제목을 알려줘. | 10월 검토 일정 | Actual-request body PASS |
| U03 | 오늘 오후 CAL01 일정을 알려줘. | 제목: 구조 검토 회의<br>시간: 2026-10-19 15:00-16:00 | Actual-request body PASS |
| U04 | WEB01 가격과 기준 시각을 읽어줘. | 제품 가격: 129000원<br>기준 시각: 2026-10-19 09:00 | Actual-request body PASS |
| U05 | PUB01에 게시된 오늘 운영 시간을 확인해줘. | 오늘 운영 시간: 09:00-18:00 | Actual-request body PASS |
| U06 | 북마크 BK01의 제품 설명을 알려줘. | 제품명: Cedar 도킹 스테이션<br>설명: USB-C와 외부 화면 두 개를 연결하는 허브 | Actual-request body PASS |

### Oracle-field differences remain separate

- **RU02:** every actual U02 asks for the email subject. The answer `10월 검토 일정` is supported by M01 line 2. The descriptive oracle also mentions the sent time `2026-10-18 16:30`; that text is absent from all seven answers. The generic goal completion condition refers to what the input requests and does not independently add a sent-time requirement. Record seven literal-field nonmatches with `ORACLE_INPUT_MISMATCH_REQUIREMENT_UNRESOLVED`, not seven invented user-request failures.
- **RU05:** every actual U05 asks the operating hours posted in PUB01. The answer gives `09:00-18:00`; the visible string `공개 안내 PUB01` is absent in all seven. Actual source IDs, revision and exact quote claims remain present. The meaning instruction requires claims tied to supplied originals; a mandatory visible citation is not established by that wording. Retain seven literal-field nonmatches with the extra visible-citation requirement unresolved.
- No oracle text, timing denominator, QA02 set denominator or original outcome is altered. S02 has no frozen referent-set obligations. These diagnostic distinctions do not retrospectively filter QA01 callbacks.

### Ancillary instruction deviations are not hidden

The actual system instruction says `bindings:[{expression:exact current spoken substring,...}]`. In 37 adopted plans the expression instead quotes the source text. For example, U01 is `F01 결론을 두 문장으로 요약해줘.`, while the binding expression begins `결론: 작은 읽기 작업을 병합하면 준비 비용을 줄일 수 있다.` This source quotation is not a substring of the user utterance. BAAA, CAAA, ABAA, AAAB, AAAC and AAAA have six such deviations each; AABA has the U01 deviation and no bindings in U02–U06. The correct source targets and factual bodies do not erase this separate instruction failure. This note does not introduce a new scored QA02 set or a semantic gate on QA01.

## Per-run exact event references

Each cell lists original trace `/events/N` pointers in U01–U06 order. All six adoptions in every run are `NEW` / `DIRECT`, distinct `goal:S02-U01` through `goal:S02-U06`, revision 1, `task_id=null`, empty permission and control operations. Each final record has matching origin and last event, no actual ingress, active count 0 and no pending input.

| Configuration / run | Capture events | Adoption events | Display events | Final EPISODE_RESULT |
| --- | --- | --- | --- | --- |
| BAAA / `BAAA-S02-r2-89dd59` | 6, 40, 74, 108, 142, 176 | 26, 60, 94, 128, 162, 197 | 28, 62, 96, 130, 164, 199 | 210 |
| CAAA / `CAAA-S02-r2-4253dd` | 30, 98, 167, 239, 314, 391 | 57, 123, 192, 264, 332, 410 | 59, 125, 194, 266, 334, 412 | 478 |
| ABAA / `ABAA-S02-r2-ea3f3d` | 6, 43, 80, 117, 154, 191 | 29, 66, 103, 140, 177, 215 | 31, 68, 105, 142, 179, 217 | 228 |
| AABA / `AABA-S02-r2-62dedb` | 6, 43, 80, 117, 154, 191 | 29, 66, 103, 140, 177, 215 | 31, 68, 105, 142, 179, 217 | 228 |
| AAAB / `AAAB-S02-r2-be8098` | 32, 104, 171, 249, 327, 408 | 61, 125, 195, 270, 348, 436 | 63, 127, 197, 272, 350, 438 | 499 |
| AAAC / `AAAC-S02-r2-0c469b` | 6, 43, 80, 117, 154, 191 | 29, 66, 103, 140, 177, 215 | 31, 68, 105, 142, 179, 217 | 228 |
| AAAA / `AAAA-S02-r2-7e02d9` | 6, 43, 80, 117, 154, 191 | 29, 66, 103, 140, 177, 215 | 31, 68, 105, 142, 179, 217 | 228 |

All actual source ranges retain revision 1 and pre-existing local permission. There is no observed new permission grant or user-requested downstream execution. Prepared/reused evidence is not proof that acquisition, producer work or cost disappeared. Each JSON response row preserves the exact actually supplied original and acquisition/producer metadata.

## Preserved AAAB preparation failure

AAAB’s `/events/179` (`event-217`) ends `operation-122`, stage `handoff_publish_work_record`, with `FAILED`. `/events/180` (`event-218`) records `PREPARATION_FAILED`, owner `handoff`, with `JSONDecodeError: Unterminated string starting at: line 161 column 11 (char 4803)`.

The directly read raw response is [`9297823b-0cfb-46b2-b0a7-3a679cf5e32f-response.json`](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/model/9297823b-0cfb-46b2-b0a7-3a679cf5e32f-response.json), SHA-256 `e6a050cb255f1be2cebf1733fec05e8d6123e97b490b9595f2a879d24d693063`, recorded by AAAB `/calls/5/response_file`. It has `finish_reason="length"`, 1500 completion tokens and ends inside an `end_line` JSON key. This supports an actual truncated model output followed by rejected preparation, not an invented external Agent result. No handoff was requested by S02; all six direct answers were still displayed. The failed producer call, its inputs, expenditure and delay stay in the frozen evidence. The terminal’s empty workflow failures list does not erase this separately recorded preparation failure.

## Existing stop-rule assessment and limits

All seven bound copied `TERMINATED` records report return code 0, `technical_failure=false` and `budget_stop=false`. The accepted extracts record no technical failure, technical abort or unprovided mandatory event. Frozen `harness.py` lines 295–299 distinguish host technical failures; `joint_campaign.py` lines 177–194 stop on nonzero exit, technical failure or budget stop. This bounded review establishes no additional existing mandatory technical-stop condition. It does not independently certify clocks or endpoint eligibility.

The meaning/source-binding deviations and AAAB preparation failure remain candidate/model diagnostics. No controller change, prompt repair, retry or rerun follows. No next cursor is authorized by this note.

Other limits: text is a common host callback proxy; Voice is OFF; real mailbox/calendar/web access and physical audio/product completion are not verified. HOST_COMPLETE is a recorded host state, not full product verification. No timing, accuracy-set, cost or memory metric was recomputed. This block does not establish a population winner or architecture causal effect.
