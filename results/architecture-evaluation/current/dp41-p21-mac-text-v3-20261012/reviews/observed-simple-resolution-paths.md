# v3의 단순 조회와 현재 산출물 연결: 실제 A1/B1 기제 비교

상태: **READ_ONLY_MECHANISM_OBSERVATION / NO_QA_VERDICT**. 공식 v3의 `(A,1,S1-01)/(B,1,S1-01)`과 `(A,1,S3-02)/(B,1,S3-02)`만 읽어 원천 → 모델 출력 → 실제 처리/효과 → 전달을 연결했다. S1-01을 양안 모두 성공한 실행으로 만들기 위해 후보 오류를 숨기지 않는다. S3-02는 양안의 실제 현재 producer→consumer payload 연결을 확인할 수 있는 보완 사례다. 최종 C1~C6나 QA 점수는 판정하지 않는다.

기준은 [manifest](../manifest.json)의 `DP41-P21-MAC-TEXT-v3`, frozen source digest `f521d6bea196342dab6c0f66268134e9c33f25dfa86a0aac874afc160cb6a20a`와 동결된 [ASR-QA-v6 명세](../frozen-source/docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)다. 실제 의미 모델 호출과 전사/Voice 모형을 사용한 증거이며 실제 음성 품질/Windows 실행의 증거가 아니다. API 호출, 후보/테스트 실행, 후보/fixture/정답 수정과 재채점은 수행하지 않았다.

[A1 실패 경로 관찰](./observed-resolution-paths-A1.md)과 [B1 실패 경로 관찰](./observed-resolution-paths-B1.md)에서 드러난 모델 선택/공통 검색 한계와 함께 읽는다. 사례의 호출 수나 한 번의 정상 처리만으로 구조 고유 우열을 일반화하지 않는다.

## 1. stable trial의 식별과 원본

| 실제 tuple / raw trial | Request / input | raw JSONL 행 SHA-256 / meaning SHA-256 |
| --- | --- | --- |
| `(A, 1, S1-01)`<br>A1 line 1, E485, OBSERVED | `R:20e39dab-a7b3-4c76-94f1-3a9bb318c277`<br>`I:72d24a25-361c-45c0-8ad3-ee4c1244a97f` | raw: `04de6e0b34d3d7ad1e812bdb0e2a63b56ea7df5e01bee7ab51e8c94769ace065`<br>meaning: `1aa279044fddd45e909625866b5106dc41a9ac5309e94f337b263c9abd61c55f` |
| `(A, 1, S3-02)`<br>A1 line 9, E9194, OBSERVED | `R:6fc2ba60-28a8-4a3f-81c2-2cc7ac95c474`<br>`I:74115b4d-4e20-4e87-a0e3-dda3036e6ecb` | raw: `64f00edd16675d2a02ace592c750e2aa566f4961c351f5da29e74fffa17d7070`<br>meaning: `37651cf9b10d2bf9984fcb4fff2b06a6ac013e8a259c4da9b4339937d12d3131` |
| `(B, 1, S1-01)`<br>B1 line 1, E391, OBSERVED | `R:06fe70d8-4e32-4cbd-b094-b74c00c36acc`<br>`I:9fdfd08e-15e7-4df8-a2d3-489ad644bca9` | raw: `c27cc45c677250c4920d4d3f3dca33d2bcd94f0f1f243f952053c78e9b594881`<br>meaning: `1b52525a58f022feb4176fd2c5d18c60b0726084d44a5740e4ac3e5495f3c688` |
| `(B, 1, S3-02)`<br>B1 line 9, E9228, OBSERVED | `R:364a7d06-2bfd-4141-bc25-95a40fe99bb8`<br>`I:ea30a4c5-d9f3-46e4-a63d-288c8acde5b8` | raw: `b68c4080a9e58322630abf3a425a4e2570c37a5ce4c32ca0c3a58a88e5e1358c`<br>meaning: `9073d6a7dedeb1af6150a2f8c30ba23a6c207221439302087692bebe00329c06` |

원본은 [A1 trial](../workers/A1/trials-unjudged.jsonl), [B1 trial](../workers/B1/trials-unjudged.jsonl)이다. raw hash는 해당 UTF-8 행의 끝 LF까지 포함하고 meaning hash는 채택 meaning의 canonical JSON bytes다. 원장 전체가 append 중인 시점에 전체 파일을 고정됐다고 주장하지 않는다.

## 2. S1-01 — 같은 D1 read 뒤 A의 재판단과 B의 code binding

### 2.1 원문 요구와 최초 입력

원문은 “D1에 적힌 마감일 알려줘.”이며 실제 D1@v1 본문은 “제출 마감일은 2026년 10월 15일입니다. 담당자는 민수입니다.”다. 관련 기존 Task는 없고 read/model/agent 제공은 허용된다. 직접 조회/응답이나 같은 한정 조회 Agent 위임이 허용되며 문서 변경/새 조사/일정 생성은 요구되지 않는다. D1@v2나 미래 사건은 이 사례에 없다.

양안의 최초 모델 Context는 동일 원천 D1의 ID/revision/title/권한 metadata만 받았고 본문이나 날짜 정답은 없었다. 날짜가 실제 read/source에서 도출되는지와 조회 목표를 어떤 연산으로 표현하는지를 분리한다. 공통 계약에는 `DIRECT_RESPONSE/READ_FIELD` + `field=DEADLINE`이 존재한다.

### 2.2 A: 모델이 READ를 선택하고 실제 결과에서 whole meaning을 다시 생산

| Event / scene seconds | 실제 처리 |
| --- | --- |
| E101 / 3.676083 | 모델의 READ: EXPLICIT_SOURCE(DOCUMENT, D1), limit=1 |
| E103 / 3.677016 | owner read `3d7f9d3d-a850-4cfe-a055-61d41b6aa50f`, status=OK, D1@v1 본문 반환 |
| 두 번째 CINTERPRET Context | 최초 입력과 실제 `bounded_read_results`가 함께 전달됨. 날짜 정답을 평가자가 첨가한 Context가 아니다. |
| E154/E155 / 5.729187~5.729313 | 모델이 PROPOSE whole meaning 생산. answer에 “2026년 10월 15일”을 넣었으나 goal kind/operation은 ANSWER_QUESTION, question_ref=null, target=null |
| E159 / 5.731213 | 위 meaning 채택 |
| E163 / 5.731576 | 실제 GOAL_FAILED: `Existing Task target unavailable` |

이 경로는 **READ → 실제 read 결과의 모델 재판단 → PROPOSE**를 보여준다. 하지만 ANSWER_QUESTION은 기존 owner 질문을 처리하는 연산이고, 이 사용자 조회에는 그런 대상이 없다. 모델이 조회한 날짜를 이해한 사실과 실행 가능한 직접 조회 목표를 선택한 사실은 서로 다르다. schema에 맞는 JSON을 만들었다는 사실로 목표 처리 성공을 주장하지 않는다. 지원된 READ_FIELD 경로를 선택하지 않은 것을 새로운 deadline 연산 부재로 분류할 근거도 없다.

실제 effect는 0이다. publication `P:1:dba816ac-ce80-42dc-8fab-21db332e8151`은 “2026년 10월 15일 답변 요청이 기존 작업 대상 부재로 거절되었습니다.”를 Text 6.670883초, Voice 6.923485~13.925800초로 전부 전달했다. 전달 receipt=true다. 날짜가 실제 문장에 있지만 거부 안내도 함께 전달된 기록을 그대로 유지한다. 이 문장의 각 정답 조건 충족 여부는 이후 judge의 사실별 판정 대상이다.

### 2.3 B: finite frame과 단일 후보의 code binding, 실제 source에서 응답 생산

| Event / scene seconds | 실제 처리 |
| --- | --- |
| E120 / 4.654375 | 모델 B_FRAME: goal DIRECT_RESPONSE/READ_FIELD; field=DEADLINE; source_refs=[r-document-d1] |
| E121/E122 | r-document-d1: UNRESOLVED→READ_PENDING |
| E124 / 4.655431 | code가 EXPLICIT_SOURCE(DOCUMENT, D1), outer query=D1, limit=10 조회. read `4a37a7e6-ac52-4cf4-9d49-6485e8e30fde`: D1@v1 실제 본문, status=OK, 후보 1 |
| E127 / 4.655662 | 단일 actual 후보 D1@v1을 code가 BOUND로 갱신. 부분 모델 호출 없음 |
| E128/E131 / 4.655753~4.657868 | code가 source_refs=[D1@v1]의 meaning을 결합해 제안/채택 |
| E133 / 4.660419 | 실제 SOURCE_COMPOSITION_READ: D1@v1, canonical source payload hash `f2cf4ecd79fc834d452ba0089d14b6f39e342ad70c1419440f6a5b7797d5f95e` |

최종 meaning은 날짜 문장을 담지 않고 조회 연산/field/source binding을 담았다. [공통 runtime](../frozen-source/prototypes/candidates/dp41_45_poc/shared/runtime.py)은 실제 source를 CRESPONSE Context에 제공했고 그 모델이 “D1의 마감일은 2026년 10월 15일입니다.”를 생산했다. 이 값을 B code가 oracle에서 꺼내거나 code 자체가 의미를 완전히 판정했다고 주장하지 않는다. 항목 binding은 code가 소유하고 최종 자유문 생산은 공통 응답 모델 의존성에 남는다.

effect는 0이고 tracked 새 Task도 만들지 않았다. publication `P:1:440390d4-957d-46fe-86b8-27f5e5010213`은 Text 5.804761초, Voice 6.066908~10.406196초로 위 문장 전체를 전달했고 receipt=true다.

### 2.4 호출과 구조 해석의 한계

| worker / 원장 line / role | 호출 ID와 실제 단계 | Context / Output |
| --- | --- | --- |
| A1 / 1 / CINTERPRET | `55713777-5a0e-421f-b2e6-a7cc6fd43155`<br>READ; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/55713777-5a0e-421f-b2e6-a7cc6fd43155.json)<br>[Output](../workers/A1/model-outputs/55713777-5a0e-421f-b2e6-a7cc6fd43155.json) |
| A1 / 2 / CINTERPRET | `f15a4025-db9d-408f-8a55-909098551248`<br>PROPOSE; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/f15a4025-db9d-408f-8a55-909098551248.json)<br>[Output](../workers/A1/model-outputs/f15a4025-db9d-408f-8a55-909098551248.json) |
| A1 / 3 / CRESPONSE | `a260973c-8bb3-45b8-aa27-9173cd41c314`<br>CRESPONSE; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/a260973c-8bb3-45b8-aa27-9173cd41c314.json)<br>[Output](../workers/A1/model-outputs/a260973c-8bb3-45b8-aa27-9173cd41c314.json) |
| B1 / 1 / CINTERPRET | `a7eb780f-3afd-4fa0-bccd-89f378982cf2`<br>B_FRAME; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/a7eb780f-3afd-4fa0-bccd-89f378982cf2.json)<br>[Output](../workers/B1/model-outputs/a7eb780f-3afd-4fa0-bccd-89f378982cf2.json) |
| B1 / 2 / CRESPONSE | `81a3b2ea-57c6-4483-ad84-4bc18c7cc0b8`<br>CRESPONSE; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/81a3b2ea-57c6-4483-ad84-4bc18c7cc0b8.json)<br>[Output](../workers/B1/model-outputs/81a3b2ea-57c6-4483-ad84-4bc18c7cc0b8.json) |

A1의 CINTERPRET 2회와 B1의 CINTERPRET 1회는 이 실제 경로의 사실이다. 공통 CRESPONSE는 각 1회였다. 모든 호출 HTTP 200/attempt 1이며 확인 질문/부분 모델/transport 재시도는 없었다. B의 단일 read 후보가 충분해 부분 호출이 생략된 사례이고 B가 항상 한 번의 의미 호출로 끝난다는 증거는 아니다.

A가 항상 원문 조회 뒤 두 번 호출하도록 강제된 것도 아니다. A는 metadata로 지원된 goal을 바로 제안할 수 있다. 실제 S3-02에서도 그런 선택을 했다. 이 사례의 호출 차이는 구조가 허용하는 다음 행동을 모델이 어떻게 선택했는지와 결합해 설명해야 한다. A의 잘못된 ANSWER_QUESTION 선택을 삭제하거나 합법 READ_FIELD로 바꾼 가상 실행을 실측에 합치지 않는다.

## 3. S3-02 — 양안이 실제 현재 요약을 Mail consumer payload에 연결

### 3.1 원문 요구와 원천

입력은 “D1을 요약한 다음, 그 요약을 민수에게 메일로 보내줘.”다. 실제 source D1@v1은 “하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다.”, 민수@v1은 `minsu@example.invalid`다. 원문 명세는 현재 요청의 실제 요약 산출물을 쓰고 옛 R옛/K옛 결과를 사용하지 않으며 요약 준비 뒤 고유 SEND 효과 한 번을 요구한다. Core 요약/Agent 요약과 여러 Task 구성은 허용된다. §8.2는 SEND 접수+8초 최종 SENT tail도 포함한다.

이 주소는 모사 주소이고 실제 메일 발송 API를 사용하지 않았다. 요약과 최종 메일 artifact는 실제 후보가 보낸 payload/source에서 공통 producer가 생성한다. 현재 요약의 ID/내용을 사전 정답으로 주입하지 않는다. 원문이 짧아 이 stub은 사실상 원문과 동일한 요약 문장을 생산했다. **현재 산출물 identity/사용/순서**를 검증하는 기제 증거이며 압축된 요약의 domain 품질 증거는 아니다.

### 3.2 A의 실제 whole meaning과 B의 실제 finite frame/binding

A의 첫 모델 Context에는 D1@v1/D1@v0/민수@v1 metadata가 있었고 body는 없었다. E7901 / 5.532215초의 첫 PROPOSE는 두 NEW_TASK 목표 SUMMARIZE/SEND_MAIL, AFTER 관계와 `body={result_of:G_SUMMARY}`, 수신자 민수@v1을 명시했다. 의미 판단 과정의 OWNER_READ는 없었다. A 모델은 제공된 실제 handle에서 전체 의미 결합을 제안했고 실제 Agent 제공 시 SOURCE_COMPOSITION_READ가 original body를 공급했다.

B의 최초 frame E7727 / 5.847650초는 SUMMARIZE/SEND_MAIL, AFTER, `body={result_of:G-summarize}` 및 unresolved source reference IDs `r-doc-D1/r-contact-minsu`를 생산했다. code는 E7731/E7737에 D1과 민수를 실제 read했고 각각 후보 1을 받아 E7740/E7741에서 BOUND로 확정했다. read IDs는 `44a70540-c96e-4b5a-930e-1ea5783f8fed`, `9bdf49d3-3322-4f30-8e8a-55cd0c1a904f`다. 부분 선택 모델은 불필요했다.

B `_build_meaning`은 실제 source/수신자 ref를 code로 교체하고 body를 `GOAL_RESULT(producer_goal_id=G-summarize, request_id=R:364a7d06-2bfd-4141-bc25-95a40fe99bb8, producer_revision=1)`로 정규화했다. A의 result_of와 B의 typed symbolic 결과는 실행 시 실제 현재 producer artifact를 기다리는 공통 경로로 연결됐다. B가 마지막에 전체 의미를 다시 만드는 모델을 부르지 않았다.

| 단계 | A1 | B1 |
| --- | --- | --- |
| semantic 전체 생산 | actual PROPOSE E7901 | actual unbound frame E7727 |
| semantic body read | 없음; provided handle 사용 | E7731 D1@v1 / E7737 민수@v1 |
| 최종 결합/채택 | 모델 전체 meaning; E7905 / 5.534873초 | code bindings/symbolic normalization; E7745 / 5.853498초 |
| 실제 요약 source 제공 | E7907 SOURCE_COMPOSITION_READ | E7747 SOURCE_COMPOSITION_READ |
| 요약 원문 배열 canonical hash | `6cd83340306cd9e8924c114e9db86aaec28466f8abf29ce493af2b492f1aca8f` | 동일 |
| 실제 수신자 source 제공 | E7938 | E7779 |
| 수신자 원문 배열 canonical hash | `2808105d68a671e13d0d0eb44a02a7b656ba13334d1b2e18af1c2e075b3d7a1b` | 동일 |

이 사례는 강한 A가 실제 metadata 기반 binding으로 바로 실행할 수 있고 B는 frame에 명시한 reference를 code 경로에서 조회한다는 관측을 보여준다. B의 추가 read나 A의 read 생략을 숨기지 않는다. 양안 모두 실제 source/수신자의 권한과 제공 여부는 공통 실행 경계에서 검사됐다.

### 3.3 실제 command, 산출물과 payload

| worker / goal | 실제 command / Task / Execution | send / 외부 receipt seconds | payload SHA-256 |
| --- | --- | --- | --- |
| A1 / G_SUMMARY<br>SUMMARIZE | `CMD:147b0505-b3cd-4334-b225-08cbae21489d`<br>`T:cb410d18-ed9f-44d3-b963-e9fbe267dfc5`<br>`X:f0ee7c51-192a-4669-a55d-321f742ab638` | 5.541288 / 5.743727 | `8d866d8e735dfdbd3b6a424c02c9a09a4f8b977b844b248d64e1a7ed1bf2ec19` |
| A1 / G_SEND_MAIL<br>SEND_MAIL | `CMD:2bf37299-5420-4848-9dc8-4c3618e85738`<br>`T:981d47b0-ee3a-4d08-ae6d-ef06b18578e0`<br>`X:577c4ba6-1a6b-41c5-8f27-fdb7d6afce2f` | 6.064674 / 6.267074 | `3fcb8d0d388cfead5142fad1b332a37cd2c6160ad3956bedb4eb20e97043504f` |
| B1 / G-summarize<br>SUMMARIZE | `CMD:8d97483e-4388-4b05-80f0-b8e230152513`<br>`T:806ed843-72a4-48dd-908d-9fdc58dd4c9d`<br>`X:4fdf4c1b-b439-420a-9de8-ef62bc44ab8b` | 5.858067 / 6.059089 | `6dc6a5336305b25e143fee28fcd4d133e1e0bfdfd94d5687cf3f6a23aa10ac23` |
| B1 / G-send-mail<br>SEND_MAIL | `CMD:361cc9db-168f-4d98-a401-597d45c7a5ef`<br>`T:9dcc80bd-31e8-4795-809f-7b97209bc315`<br>`X:0ef14e62-68a3-4733-b7c4-6a8e2b1851fc` | 6.380544 / 6.582525 | `ed2c7728f62de5126663af8767cc3966f941bc0f98d504b1a05d1f652304f60d` |

A 요약 artifact `A:98f76c8f-be4d-48eb-a632-d13881009555@v1`은 E7929 / 6.046372초에 actual-ready로 기록됐다. A SEND payload.body_artifact는 바로 그 ID, producer_goal=G_SUMMARY, 현재 request_id, source_refs=D1@v1과 실제 요약 text를 포함한다. SEND는 6.064674초에 실제 전송돼 ready 이후다.

B 요약 artifact `A:8a2dc466-ff67-4780-bb24-937f92fafcba@v1`은 E7769 / 6.364150초에 actual-ready로 기록됐다. B SEND payload.body_artifact는 그 ID, producer_goal=G-summarize, 현재 request_id, 실제 D1@v1/요약 text를 포함한다. SEND는 6.380544초에 실제 전송돼 ready 이후다. 양안의 recipient는 실제 `minsu@example.invalid`다.

양안 각각 실제 SUMMARIZE command 1회, SEND_MAIL command 1회였다. task와 실행은 서로 다른 고유 ID이며 옛 Task/T보호를 대상으로 바꾸지 않았다. canonical K현재 표기는 평가용 역할 label이므로 실제 UUID를 문자 그대로 K현재와 비교하는 판단은 하지 않는다.

### 3.4 옛 결과와 중복의 actual 기록

old-summary-result 외부 사건은 약 3.1초에 도착했다. A E7846, B E7666은 `RESULT_REJECTED/ALREADY_CURRENT_COMPLETED`를 기록했다. 실제 현재 SEND body에는 옛 artifact가 아니라 위 현재 UUID가 들어 있다. `old_job_created=false`, `late_return_exercised=false`는 후보의 취소된 옛 모델 job probe가 이 장면에 발생하지 않았다는 별도 coverage 사실이다. 이 명세상의 옛 Agent event와 취소된 모델 job의 늦은 반환을 같은 사건으로 표현하지 않는다.

실제 현재 artifact는 먼저 `_store_artifact`에서 ready/owner 상태를 반영했다. 그 뒤 같은 현재 결과 event의 재반영은 A E7934/B E7775에서 `ALREADY_CURRENT_COMPLETED`, 다음 동일 event ID 재전달은 A E7945/B E7786에서 `DUPLICATE_IGNORED`였다. 실제 ready artifact와 SEND payload가 존재하므로 첫 재반영 거부를 “현재 산출물이 없었다”로 읽으면 안 된다. 중복 사건이 고유 SEND를 두 번 만들지 않았다는 actual effect 원장을 함께 확인한다.

최종 mail artifact는 A `A:acfe63fe-7358-4ce6-a6d3-b28181ff5df4@v1`(E8175 / 14.071577초), B `A:74f14ade-1ef1-4c47-8401-02d03a1f8d94@v1`(E8015 / 14.388756초)였다. 두 artifact 모두 external_status=SENT, body_artifact_id는 위 현재 summary ID이고 실제 recipient/source refs를 유지했다. 실제 발송 서비스를 통한 delivery 보장이 아닌 모사 Agent의 결과다.

### 3.5 실제 사용자 Text/Voice 전달

| worker / publication | 실제 Text | Text / Voice onset / Voice end seconds | receipt |
| --- | --- | --- | --- |
| A1 / `P:1:f349d7a4-ffdc-4d06-b49c-a970457df4e7`<br>RESPONSE | SUMMARIZE 업무는 접수(ACCEPTED)되었으며, 현재 실행 중(RUNNING)입니다. | 6.718441 / 6.979745 / 15.651137 | true |
| A1 / `P:2:39efead0-1c62-4189-b354-0db308980567`<br>RESULT | D1 요약 완료: 하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다. | 6.775240 / 15.922135 / 23.426556 | true |
| A1 / `P:3:6e0f3d9d-203e-4ffc-98b0-162fc43c2f3a`<br>RESPONSE | SEND_MAIL 업무는 접수(ACCEPTED)되었습니다. 요약 결과 '하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다.'를 민수에게 메일로 보냅니다. | 7.351202 / 23.692047 / 39.031567 | true |
| A1 / `P:4:a344c17d-8840-4035-a5b4-6a0ebada422e`<br>RESULT | 민수에게 메일 발송 완료. 내용은 '하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다.' | 14.934861 / 39.312223 / 48.655847 | true |
| B1 / `P:1:b006ab40-8ce0-4a80-a4a8-ae79c28b342f`<br>RESPONSE | D1 요약 업무가 접수(ACCEPTED)되었습니다. 완료되지 않았습니다. | 7.186459 / 7.444705 / 14.119204 | true |
| B1 / `P:2:246cca73-8e62-471d-afd7-65f744e12fb9`<br>RESULT | SUMMARIZE 업무가 완료되었습니다. 결과: 하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다. | 7.274344 / 14.406007 / 24.744865 | true |
| B1 / `P:3:0ce205e9-8c31-411b-8965-5ddfdd1aa7d1`<br>RESPONSE | SEND_MAIL 업무는 접수(ACCEPTED)되었습니다. 요약 결과 '하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다.'를 수신자(minsu@example.invalid)에게 발송 중이며, 아직 완료되지 않았습니다. | 7.773738 / 25.009844 / 46.513759 | true |
| B1 / `P:4:de86c2c0-5a92-4f1e-8f51-890e769340b5`<br>RESULT | SEND_MAIL 작업이 완료되었습니다. 하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다. | 15.459262 / 46.796175 / 56.468867 | true |

각 행의 Voice는 Text와 같은 전체 문장이 실제 모형 sink에 전달됐다. 시작의 TEXT_PRESENTED event snapshot에서 voice_text가 비어 있던 상태를 이후 최종 receipt와 혼동하지 않는다. 접수 단계와 SENT 최종 tail을 다른 publication으로 보존하며 각 사실의 최종 정확성 평가는 별도 judge 작업이다. 여기서 시간을 QA 점수/실패 cap/전체 사이클 평균으로 계산하지 않는다.

### 3.6 실제 호출

| worker / 원장 line / role | 호출 ID와 실제 단계 | Context / Output |
| --- | --- | --- |
| A1 / 34 / CINTERPRET | `fa88a846-ee09-42e6-bbda-c7d34e97ffb3`<br>PROPOSE; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/fa88a846-ee09-42e6-bbda-c7d34e97ffb3.json)<br>[Output](../workers/A1/model-outputs/fa88a846-ee09-42e6-bbda-c7d34e97ffb3.json) |
| A1 / 35 / CRESPONSE | `200d68df-c747-409b-a69b-2b58b38500fd`<br>CRESPONSE; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/200d68df-c747-409b-a69b-2b58b38500fd.json)<br>[Output](../workers/A1/model-outputs/200d68df-c747-409b-a69b-2b58b38500fd.json) |
| A1 / 36 / CRESPONSE | `22f49a7f-723d-4334-ae72-1e47b8bb7868`<br>CRESPONSE; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/22f49a7f-723d-4334-ae72-1e47b8bb7868.json)<br>[Output](../workers/A1/model-outputs/22f49a7f-723d-4334-ae72-1e47b8bb7868.json) |
| A1 / 37 / CRESPONSE | `72fd65a0-6b37-4ae5-81ff-f481ed2cf89b`<br>CRESPONSE; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/72fd65a0-6b37-4ae5-81ff-f481ed2cf89b.json)<br>[Output](../workers/A1/model-outputs/72fd65a0-6b37-4ae5-81ff-f481ed2cf89b.json) |
| A1 / 38 / CRESPONSE | `b8474e49-e009-44b5-9a35-714d08cdfbc3`<br>CRESPONSE; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/b8474e49-e009-44b5-9a35-714d08cdfbc3.json)<br>[Output](../workers/A1/model-outputs/b8474e49-e009-44b5-9a35-714d08cdfbc3.json) |
| B1 / 39 / CINTERPRET | `b659a894-895b-4877-92cb-19de9f882d9a`<br>B_FRAME; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/b659a894-895b-4877-92cb-19de9f882d9a.json)<br>[Output](../workers/B1/model-outputs/b659a894-895b-4877-92cb-19de9f882d9a.json) |
| B1 / 40 / CRESPONSE | `37d711d6-48db-4f2f-ac92-5c6c1bc60360`<br>CRESPONSE; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/37d711d6-48db-4f2f-ac92-5c6c1bc60360.json)<br>[Output](../workers/B1/model-outputs/37d711d6-48db-4f2f-ac92-5c6c1bc60360.json) |
| B1 / 41 / CRESPONSE | `25af0187-4ce6-4bfe-b519-1e7e307967f1`<br>CRESPONSE; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/25af0187-4ce6-4bfe-b519-1e7e307967f1.json)<br>[Output](../workers/B1/model-outputs/25af0187-4ce6-4bfe-b519-1e7e307967f1.json) |
| B1 / 42 / CRESPONSE | `b6f25466-aee4-4bf7-b839-ec173054123e`<br>CRESPONSE; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/b6f25466-aee4-4bf7-b839-ec173054123e.json)<br>[Output](../workers/B1/model-outputs/b6f25466-aee4-4bf7-b839-ec173054123e.json) |
| B1 / 43 / CRESPONSE | `e4cd99c6-caa5-4b5c-a415-27cd7c77391d`<br>CRESPONSE; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/e4cd99c6-caa5-4b5c-a415-27cd7c77391d.json)<br>[Output](../workers/B1/model-outputs/e4cd99c6-caa5-4b5c-a415-27cd7c77391d.json) |

양안 CINTERPRET는 각각 1회이고 CRESPONSE는 각각 4회였다. body/recipient 참조가 단일 후보로 해결돼 B_PARTIAL_SELECT는 없었다. B에 부분 호출을 강제로 추가하거나 A에 재판단 read를 강제로 넣지 않았다. 모두 HTTP 200/attempt 1이다.

## 4. 이 사례로 설명할 수 있는 것과 남는 한계

- S1-01은 A의 실제 bounded-read 재판단과 B의 finite reference code binding을 보여준다. 원문 날짜 확보, 연산 선택, 사용자 전달은 서로 다른 사실이며 A의 잘못된 goal을 숨기지 않는다.
- S3-02는 양안이 symbolic 현재 요약 참조를 actual artifact/consumer payload로 연결하고 같은 공통 외부 조건 아래 고유 SEND 한 번을 수행한 실제 기제 증거다. A의 즉시 whole meaning과 B의 code read/binding은 다른 경로였지만 이 후속 execution 책임은 공통이다.
- 이 결과는 A/B의 의미 정확성/시간/비용/변경 용이성/메모리 우열이나 최종 QA 점수가 아니다. 모사 요약/메일은 실제 domain 품질과 외부 서비스 안전성을 검증하지 않는다. B의 부분 선택이 발생하지 않은 사실로 부분 선택 자체의 품질을 주장하지 않는다.
- 관찰된 차이가 작거나 같은 결과로 이어져도 동일 효과/공통 execution 조건을 그대로 남긴다. 이전 S2 실패와 균형을 맞춘다는 이유로 성공 판정이나 숫자를 바꾸지 않는다.

## 5. Context/Output artifact hash 검수

4개 stable trial raw 행/채택 meaning과 4개 실제 command payload, 관련 모델 호출 15개의 Context/Output artifact 30개 canonical hash를 읽기 검증했다. artifact hash는 JSON을 `ensure_ascii=False`, `sort_keys=True`, `separators=(',', ':')`로 직렬화한다. `context.sha256`와 별도 caller fingerprint `context_sha256`는 다르다.

```text
A1 S1-01 CINTERPRET 55713777-5a0e-421f-b2e6-a7cc6fd43155
  context_artifact_sha256=04e724c1041bdbbdb5fe2d0f4f6dfcb3422ef3267264766f5d80c232f087f475
  output_artifact_sha256=0836bd968bb9d52c5a7047e91d3331ec87d9f895d4fbc2963d572bfe9deb83ac
A1 S1-01 CINTERPRET f15a4025-db9d-408f-8a55-909098551248
  context_artifact_sha256=89314b6bb930378c16107896175f8f6ff77653564230cd413e2f49f46ff74cc8
  output_artifact_sha256=9ee83b01eed9865b1ff92d61c882237d411a1dd82a36976a25a51ffd6180638a
A1 S1-01 CRESPONSE a260973c-8bb3-45b8-aa27-9173cd41c314
  context_artifact_sha256=424831d5c44a8867f2d1051f6142ffaa86b5bc6137eb199bcb68e17cffc002df
  output_artifact_sha256=a7f35bcfb16776bb8e7ac1da2b144e2201831d920548ec2942db6dd8d1bfb737
A1 S3-02 CINTERPRET fa88a846-ee09-42e6-bbda-c7d34e97ffb3
  context_artifact_sha256=5eb533b743901f444376f92f1cab942ee11836f9557b223064659845e0abde4a
  output_artifact_sha256=825ea2e10445fdcacfbdd1067b2c1c0a451d4e76845251ef3bb4c541061958c8
A1 S3-02 CRESPONSE 200d68df-c747-409b-a69b-2b58b38500fd
  context_artifact_sha256=d6d85b284af5e9dc30e286acb874bb6c657158b966ccd3bf3b6d2562731f39aa
  output_artifact_sha256=50ba965819177e5cd07822d94d87075c4473ad05135ec5d243f0d9dcede7dee6
A1 S3-02 CRESPONSE 22f49a7f-723d-4334-ae72-1e47b8bb7868
  context_artifact_sha256=67f0f25b39939c2046467af358e8ce6da90b668fbaea607b2d88a34fd53054ba
  output_artifact_sha256=1a1f6746cc0d3019a9d0b3effa3f1889f38a58a3902545289b1a552fe88e613e
A1 S3-02 CRESPONSE 72fd65a0-6b37-4ae5-81ff-f481ed2cf89b
  context_artifact_sha256=b7f3ff442305bf6b2ff0907d2442197ca584c00019d1fc6399c9f1d380f0541c
  output_artifact_sha256=1711fa058d93b9d6223657dab7a002ccf21e0d03643f2f2c652974fa38f2ba9a
A1 S3-02 CRESPONSE b8474e49-e009-44b5-9a35-714d08cdfbc3
  context_artifact_sha256=f6c31bfcf166099163ed28cec630c6db51f744ffe7df283489640fb363dff8fc
  output_artifact_sha256=2269e425dcffd55d31e9a5bf1e842913cee3272fa0783b59e3030d1ffd79682b
B1 S1-01 CINTERPRET a7eb780f-3afd-4fa0-bccd-89f378982cf2
  context_artifact_sha256=3aba5c878acb7e1f49d9c1b84f6b8e0fdd3d7ec32c321966c0b9263636028a52
  output_artifact_sha256=998e7a0b9ed4fbfed5219ca51da4baa36eea17df9a89e33230c771005b18b6f1
B1 S1-01 CRESPONSE 81a3b2ea-57c6-4483-ad84-4bc18c7cc0b8
  context_artifact_sha256=af7df9ff8f40355743804c2da53e1af78cf599e862e226daaa0d418f6d8db6a3
  output_artifact_sha256=8e328d4019c2c9f38aad2ebd3af11b0b00b2ccb2d5b04de7713ac502938a756c
B1 S3-02 CINTERPRET b659a894-895b-4877-92cb-19de9f882d9a
  context_artifact_sha256=46454c38b309e1f089602137d941f8699c85bf7555c52f68618be683326b1e7d
  output_artifact_sha256=fe537cac32d346e77bd842fc53321d7849fd1f7e57245a8375cf6810046976f5
B1 S3-02 CRESPONSE 37d711d6-48db-4f2f-ac92-5c6c1bc60360
  context_artifact_sha256=1483cc5f47abe8081e6fbc998439a570c3fe6460d2d4c747a60130b130b52ca4
  output_artifact_sha256=2547c062a569ca5ecfc24ee6c09c111b29f64dd5f0ecabeb824503782e2cedeb
B1 S3-02 CRESPONSE 25af0187-4ce6-4bfe-b519-1e7e307967f1
  context_artifact_sha256=85ee72a09e9f024653dfa72c8b60a89bdd095ced671fb9bf8566151af988c16d
  output_artifact_sha256=f6d79b5671f30928747b219ce6c8596218e0e5638715160291887acf6ae4f5f3
B1 S3-02 CRESPONSE b6f25466-aee4-4bf7-b839-ec173054123e
  context_artifact_sha256=16498813b0a0a828e0d0d7f67714d4dfb9b43a78fac12bf6cf560265912460b4
  output_artifact_sha256=30b481bf88b72909aeb1763d198888c48d98084d07bd629d46e6c95265274d4b
B1 S3-02 CRESPONSE e4cd99c6-caa5-4b5c-a415-27cd7c77391d
  context_artifact_sha256=743cf9b34dcd005fb26df3d6f0726f8af78578d98076a54dab2fe8d539dd3c38
  output_artifact_sha256=58fdf1d2ac3e8b4794faa23212177122108f0ea6f3807e8b2fb0e96586131c28
```

[A1 events](../workers/A1/events.jsonl)/[B1 events](../workers/B1/events.jsonl), [A1 effects](../workers/A1/effects.jsonl)/[B1 effects](../workers/B1/effects.jsonl), [A1 deliveries](../workers/A1/deliveries.jsonl)/[B1 deliveries](../workers/B1/deliveries.jsonl)를 trial/context/output과 대조했다. memory.jsonl과 큰 buffer 원문은 읽지 않았다. 후보 source와 측정 원장에는 쓰지 않았으며 이 리뷰 파일 하나만 추가했다.
