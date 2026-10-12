# v4 A1의 실제 Request 해결 경로 관찰

읽기 기준: 2026-10-11 20:02 UTC, 2026-10-12 05:02 KST. v4 A1에서 기록이 끝난 네 trial만 관찰했다. 상태 `OBSERVED`는 runner의 기록 상태이며 QA 통과를 뜻하지 않는다. `EPISODE_TIMEOUT`도 기록을 보존한 종료 상태다. 이 문서에는 C1~C6 판정, QA 점수와 A/B 우열 판정이 없다.

[고정 campaign manifest](../manifest.json)의 source digest는 `c2dfc26fa6afbce8df4af3405913031ca440fefa150d6a0739c010b3ed4796a7`다. [A1 원시 trial 원장](../workers/A1/trials-unjudged.jsonl)을 줄 단위로 읽고 해당 Context와 provider 출력만 확인했다. 본 실행 중인 원장 전체의 최종 hash를 주장하지 않는다. 아래 raw 줄 hash는 마지막 LF를 포함하며, Context/출력 hash는 저장 JSON을 UTF-8, `ensure_ascii=False`, 정렬된 key와 공백 없는 구분자로 정규화한 값이다. 19개 호출의 Context/출력 hash를 호출 원장과 대조했다. 코드 실행, 테스트, API 호출과 원시 evidence 변경은 없다.

음성 구간은 공통 출력 모형의 실제 실행 기록이다. 클라우드 음성 모델의 품질이나 실제 스피커 재생 성능을 입증하는 자료가 아니다. `text`와 `voice_text`, 각각의 시점 및 최종 `delivered_receipt`를 구분한다. 초기 TEXT_PRESENTED snapshot의 빈 음성 필드로 최종 음성을 대신 판단하지 않는다.

## 1. 근거와 귀속의 기준

후보가 받은 첫 Context는 source의 ID, revision, 종류, 권한과 조회 가능한 owner metadata를 제공한다. 원문, 실제 들은 문장과 정답은 선주입되지 않는다. 조회 뒤 실제 `bounded_read_results`가 후속 모델 Context에 들어갔는지 확인한다. 같은 문서명에 여러 revision이 있을 때 `source_vectors`와 모델이 명시 선택한 revision을 함께 본다.

[고정 공통 prompt](../frozen-source/prototypes/candidates/dp41_45_poc/dp41/prompts.py)는 `PAST_DELIVERED`, `TASK_RESULT`, `SOURCE_AT_REV`, `READ_FIELD`, `CREATE_PROPOSAL` 및 `USE_RESULT`를 선언한다. [고정 runtime](../frozen-source/prototypes/candidates/dp41_45_poc/shared/runtime.py)은 model meaning을 채택한 뒤 owner 자료로 실제 payload를 조합하고 downstream 명령을 보낸다. 보통의 의미 선택 오류와 operation precondition 오류를 해당 후보의 관측으로 보존한다.

v4의 실제 전달 view 수정은 읽힌 publication scope를 불투명 ref, origin, revision과 내용 hash로 보존한다. 아래 S2 두 사례는 `PAST_DELIVERED`를 실제 호출하지 않아 그 경로를 직접 입증하지 못한다. 조회하지 않은 heard-scope를 공통 코드가 몰래 추가했다는 증거도 없다.

## 2. S1-01: 원문 조회 뒤 모델이 답은 찾았지만 질문응답 연산을 선택함

실제 발화는 “D1에 적힌 마감일 알려줘.”다. 발화 종료 2.002661초와 전사 확정 2.002704초는 별도 사건이다. 처음에는 `D1@v1` handle만 보였다. 첫 CINTERPRET가 EXPLICIT_SOURCE/DOCUMENT/D1을 READ했고, `E105`의 receipt `c469c61c-486b-45b5-84bb-052c06277434`가 원문 “제출 마감일은 2026년 10월 15일입니다. 담당자는 민수입니다.”를 반환했다. 두 번째 Context에 이 실제 receipt가 들어간 뒤 모델이 PROPOSE했다.

`E159`의 adoptedMeaning은 `DIRECT_RESPONSE / ANSWER_QUESTION`, target=null, answer=2026년 10월 15일이다. question_ref와 question_revision도 null이며 source_refs는 없다. `E163`의 GOAL_FAILED는 “Existing Task target unavailable”이다. ANSWER_QUESTION은 실제 기존 Task와 유효한 질문 권위를 요구하는 연산이다. 일반 자료 값 조회를 표현하는 `READ_FIELD(source_refs, field=DEADLINE)`가 공통 계약에 있으므로, 이 경로는 자료에 답이 없거나 일반 조회 연산이 지원되지 않은 경우로 볼 수 없다. 모델이 답의 값과 실행 연산을 다르게 만들었다는 관찰이다.

실제 downstream effects는 없다. CRESPONSE가 답과 내부 거절을 같이 문장화했다. 최종 Text와 Voice는 “2026년 10월 15일 답변 요청이 기존 작업 대상 사용 불가로 인해 거부되었습니다.”이며 전체 음성 전달 receipt=true다. 내부 GOAL_FAILED 하나로 자유문 전달 조건 전체를 자동 실패로 대체해서는 안 된다. 사용자에게 실제 전달된 문장에 대한 명세별 판정은 judge 이후에 한다.

## 3. S2-02: Task 결과는 실제 읽었으나 들은 기준은 읽지 않음

실제 발화는 “아까 들은 기준과 두 업무 결과로 새 제안서를 만들어줘.”다. 발화 종료 3.001578초, 전사 확정 3.001619초다. before-state의 E1 전체 Text는 유지비와 설치 기간, 가격 제외, 추가 보안 등급 문장을 포함한다. 실제 들은 음성 범위는 “평가 기준은 연간 유지비와 설치 기간입니다. 가격은 평가에서 제외합니다.”다. 첫 Context에는 E1/PUBLICATION_ORIGINAL handle 및 실제 publication의 presented/interrupted 상태만 있고 두 본문은 없다.

모델은 `TASK_RESULT(source_ref=T비교)` 및 `TASK_RESULT(source_ref=T견적)`만 READ했다. 이는 Task ID를 명시한 locator이며 lexical query 검색의 순위 문제를 재현하는 경로가 아니다. `E2143`는 `D2@v2`의 설치 2일/4일을, `E2147`는 `D3@v1`의 가격 100/120백만원 및 유지비 3/5백만원을 반환했다. 후속 Context에 두 원문과 read ID가 들어갔다. 이 trial 전체 OWNER_READ는 두 개이며 E1, PAST_DELIVERED와 opaque delivered view의 조회는 없다.

두 번째 CINTERPRET는 기준을 식별하지 못했다며 배경 자료, 독자와 format까지 묻는 CLARIFY를 만들었다(`E2203`). 질문 Voice는 40.986276초에 끝났다. 실제 일반 followup은 44.999645초에 발화를 마치고 44.999695초에 전사 완료됐다. `eligible_planned_reply=false`이며 답은 “새로운 조건이나 권한은 추가하지 않았어. 처음 요청대로 처리해줘.”다. 정답 기준이나 새로운 권한을 보정해서 제공하지 않았다. `E3184` 이후 Context는 이 실제 답과 같은 두 read receipt를 보존했다.

`E3256`의 adoptedMeaning은 NEW_TASK/CREATE_PROPOSAL, source_refs=D2@v2와 D3@v1, properties={}다. instruction은 두 자료로 제품을 비교하되 “명시적인 기준 문서는 없으므로” 설치 기간 단축과 “초기 비용 절감”의 장단점을 제시하라고 한다. 실제 들은 유지비/설치 기간 기준과 가격 제외를 instruction, arguments 또는 constraints에 싣지 않았다. E1 view ref가 payload에 없다. **들었던 기준을 실제 읽은 뒤 범위를 잃은 경로가 아니라 기준 조회를 생략한 경로**다. v4의 opaque scope 수정으로 이 관측을 대신 설명하거나 자동 보정할 근거가 없다.

`E3258` SOURCE_COMPOSITION_READ의 내용 hash는 `be69322ebeb6fb862dedd2335c8a5c7e3ba4fcf8f53b95e86589508c5ec0165c`다. 실제 CREATE_PROPOSAL 명령도 D2와 D3 두 본문만 받았다. `E3269`는 접수이며 이후 `E3477`에서 실제 받은 두 원문을 잇는 artifact가 나왔다. stub은 E1 기준을 추가하거나 잘못된 instruction을 정답으로 바꾸지 않았다. 최종 Text는 두 제품의 설치/가격/유지비를 전달했지만 60초 automatic watchdog의 `E3774`로 Voice가 중간에서 끝나 receipt=false가 됐다. 결과 준비, Text 제시와 Voice 완결은 서로 다른 관측이다.

A는 새 READ와 재판단으로 delivered criteria를 포함한 source_refs/instruction을 제안할 수 있는 구조다. 이번 관측은 그 지원 경로를 사용하지 않았다. 고정 B의 B_PARTIAL_SELECT는 reference 후보 하나를 선택하며 범용 Goal/value 추출을 수행하지 않는다는 별도 지원 한계가 있다. **A1만으로 B의 실제 처리나 해당 한계의 영향이 관측됐다고 주장하지 않는다.**

## 4. S2-05: 세 새 원천이 도착했지만 두 문서만 소비함

입력은 “전에 들은 기준과 방금 도착한 제품 비교, 견적으로 제안서 만들어줘. 가격은 평가하지 마.”다. E새, D새비교와 D새견적은 각각 `E5857` 2.972063초, `E5861` 2.981796초, `E5865` 2.991950초에 실제 사용 가능해졌다. 발화 종료 3.000628초와 전사 확정 3.000667초보다 앞선다. 첫 Context에는 세 handle 및 E새의 delivered publication metadata가 모두 있다. 미래 arrival schedule은 Context에 없다.

모델은 D새비교와 D새견적만 EXPLICIT_SOURCE READ했다. `E5919`의 read ID `8e1678f5-e32a-48fb-89b4-411e3b00ce76`가 납기 A=10일/B=20일을, `E5923`의 `19d99d8a-260f-4ca3-9f9f-578830f4fe78`가 가격 A=30/B=40백만원을 반환했다. E새의 실제 본문 “납기 2주 이내가 우선이고 가격은 평가에서 제외”는 조회하지 않았다.

`E5994`의 adoptedMeaning은 새 CREATE_PROPOSAL이다. arguments에는 두 문서 source_refs와 LANGUAGE=KO, AUDIENCE=CLIENT, FORMAT=PARAGRAPH가 있다. instruction에 납기 10/20일, 가격 30/40백만원과 “Do not evaluate or judge the price”가 있다. 입력의 명시적 가격 제외는 유지됐지만 E새의 2주 이내 우선 조건은 instruction, arguments, constraints 및 실제 payload 모두에 없다. 단순히 납기 원문을 받았다는 사실과 들은 우선 기준을 받았다는 사실을 구별해야 한다.

`E5996` SOURCE_COMPOSITION_READ의 hash는 `5d57f574be271116602610d4ba5001aa57b9862a4aa5b08c14abcf6106ffc642`다. 실제 명령은 기존 Task 수정이 아닌 새 Task `T:973000a2-3a84-4ecb-8f5b-1f12a8ee199c`를 만들었고 실제 sources는 D새비교@v1/D새견적@v1 두 개다. stub의 artifact와 최종 Text/Voice도 이 두 원문에서 나온 납기와 가격만 전달했다. receipt=true는 전달 완료 관측이며 누락된 우선 조건을 보충하는 증거가 아니다. 세 source 도착이 전사 전에 실제 완료되므로 E새 누락을 미래 source 미도착으로 설명할 수 없다.

## 5. S3-02: 현재 producer artifact를 실제 연결했지만 D1 과거 revision을 선택함

입력은 “D1을 요약한 다음, 그 요약을 민수에게 메일로 보내줘.”다. 발화 종료 3.000854초와 전사 확정 3.000902초다. 처음 Context에 D1@v1, D1@v0가 모두 있고 `source_vectors.D1=1`이다. 현재 v1 원문은 “하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다.”이며 v0은 “마감 10월 1일”이다. 모델은 SOURCE_AT_REV/D1/revision=0을 명시 선택했고 `E7597` receipt가 과거 본문을 그대로 반환했다. owner가 모델의 v1 요청을 v0으로 바꾼 경로가 아니다.

두 번째 CINTERPRET의 PROPOSE는 잘못된 meaning version으로 거절됐다(`E7659`). 세 번째 Context에 `version must be dp41-meaning-runtime-1` 오류와 같은 실제 read가 들어갔다. 재출력은 올바른 schema version을 사용했지만 선택한 원본은 그대로 v0이다. `E7706`은 G_SUMMARY NEW_TASK/SUMMARIZE(D1@v0), G_SEND_MAIL NEW_TASK/SEND_MAIL, body={result_of:G_SUMMARY}, recipient_ref=민수@v1 및 USE_RESULT 관계를 채택했다.

`E7708`의 실제 Summary source hash는 `e6e2a4d6b146bec91913f84a0c252221a99bab996d23d98a709b1e496b827069`다. 실제 Summary 명령 뒤 `E7730`이 새 artifact `A:bb0a9791-6d6b-48d3-bee0-318d03d984b5`를 생산했다. 이 artifact의 request_id는 이번 Request, producer_goal_id는 G_SUMMARY, revision은 1이며 본문은 받은 D1@v0의 “마감 10월 1일”이다. SEND_MAIL은 그 actual body artifact ID와 동일 본문을 실제 payload에 받아 minsu@example.invalid에 보내는 모사 명령을 접수했다. `E7980`의 최종 SENT artifact도 같은 body_artifact_id와 본문을 보존한다.

현재 producer의 symbolic body를 실제 새 artifact에 연결하는 실행 메커니즘은 이 trace에서 확인된다. **현재 Request의 새 artifact라는 lineage와 올바른 원본 revision이라는 의미 조건은 다르다.** 이번 artifact가 새로 만들어졌다는 이유로 D1@v0 선택을 올바른 D1@v1 요약으로 바꾸어 해석하지 않는다. stub이 잘못 받은 과거 원문을 현재 내용으로 고치지 않은 것도 확인된다.

기존 R옛/T옛/X옛 결과는 `E7561`에서 ALREADY_CURRENT_COMPLETED로 거절됐다. 현재 Summary 반환의 재적용은 `E7735`에서 같은 이유로 거절됐고 중복 event는 `E7747`에서 무시됐다. `old_job_created=false`, `late_return_exercised=false`이므로 실제 취소된 옛 모델 job의 늦은 반환 분기까지 검증됐다고 주장하지 않는다. 두 실제 Task의 접수 응답, 요약 완료 전달과 최종 SEND 완료 전달은 각각 독립 publication이며 아래 최종 전달 표로 확인한다.

## 6. 주장 범위

네 trace는 A의 READ 결과 재판단, 명세 오류 재출력, 실제 source 기반 명령 조합과 symbolic producer/consumer 연결을 설명하는 예다. S2의 들은 기준 누락은 실제 read 선택과 의미 instruction의 근거로 추적할 수 있고, S3의 source revision 선택과 current artifact lineage도 분리할 수 있다. 이번 범위에서 정답 보정, future source body의 무료 선주입이나 v3의 scope alias 충돌이 실제 소비됐다는 증거는 발견하지 않았다.

이 관찰로 전체 의미 정확성, B의 finite 표현 충분성, 실제 Agent 산출물 품질, 음성 모델 품질, 평균 시간 및 구조의 우열을 확정할 수 없다. C1~C6와 시간 eligibility는 명세, 최종 receipt 및 judge 분쟁 검토를 합쳐 별도로 판정한다. 아래 호출 수는 이 네 실제 경로의 원장 개수이며 계획한 참조 예산이나 최종 QA-04 비용이 아니다.

## 7. 재조회용 ID와 hash
| tuple | raw 줄 | 종료 기록 | root Request | raw 줄 SHA-256 |
| --- | ---: | --- | --- | --- |
| A1/S1-01/repeat=1 | 1, E508 | OBSERVED | `R:a908b322-d28f-4347-b4f3-ecfc7e6bed5e` | `a9521a8996c5c73e046f25b3349554ce375971922c6c0b612954597b641e792d` |
| A1/S2-02/repeat=1 | 4, E3775 | EPISODE_TIMEOUT | `R:c8151ed5-00c3-422c-a19b-9589067bfb4f` | `4837bcca3be1211717db2c8d0dec977147be35de7d4f702c875a05df9066dec0` |
| A1/S2-05/repeat=1 | 7, E6760 | OBSERVED | `R:a0adb2b7-f18e-412a-897c-001a8325279f` | `a5cda39168960b674f30e2bfb479bbaa0ac2d485533271cd1dda5e0a89c7884c` |
| A1/S3-02/repeat=1 | 9, E8687 | OBSERVED | `R:3c77d28e-db64-48e8-a625-7e809040e04f` | `b464890936f6f92657cc1491697014557f1ac88d5830a988ba9d973fec4ddcf2` |

### S1-01 실제 model 호출

CINTERPRET=2, CRESPONSE=1. 아래 호출은 모두 attempt=1, HTTP 200이며 role은 원장의 값이다. schema 거절에 따른 재출력은 새 CINTERPRET 호출로 남는다.

| role 및 call ID | Context SHA-256 | 출력 SHA-256 | input/output token |
| --- | --- | --- | ---: |
| CINTERPRET `e698d348-78e9-40c7-8b1a-262a8cb8c5c2` | [Context](../workers/A1/model-contexts/e698d348-78e9-40c7-8b1a-262a8cb8c5c2.json) `6cb1052c253a0de4c5bd82edb52ba13a59360229b4b76c917498eb2b47ad136f` | [출력](../workers/A1/model-outputs/e698d348-78e9-40c7-8b1a-262a8cb8c5c2.json) `e10ae19bec212fd1a0a6c4cd0384e42cd034a3c99dbe3c31438e6b3899eb4c4d` | 4362/98 |
| CINTERPRET `8bfde7a0-b12a-4912-a5dc-fb0f1450bc52` | [Context](../workers/A1/model-contexts/8bfde7a0-b12a-4912-a5dc-fb0f1450bc52.json) `8df6651d9c65081d93e8abdb7d0bce5c97d6d728ce8491c8fabdade410d34274` | [출력](../workers/A1/model-outputs/8bfde7a0-b12a-4912-a5dc-fb0f1450bc52.json) `c2b4fb2ea8384bb88e4f77176b4d2579b0315729718702ed0293f906cc5913f7` | 4631/230 |
| CRESPONSE `2810abbf-50f8-4979-9f8c-9a69a570a54b` | [Context](../workers/A1/model-contexts/2810abbf-50f8-4979-9f8c-9a69a570a54b.json) `d71343ed4b6215a76ea74eeb4d2029ccec53990330ddea81d9d7c6fcd4915297` | [출력](../workers/A1/model-outputs/2810abbf-50f8-4979-9f8c-9a69a570a54b.json) `398824022745a86c6b22594778e278c74dc3cbde4accf4cfae5d8f0ad068c560` | 392/31 |

실제 downstream 명령 및 source/body:

없음.

최종 전달 receipt. 숫자는 해당 scene의 실제 경계이며 반올림해 표시했다. Text와 Voice가 다르면 둘을 따로 기재한다.

| publication | 종류 | Text 제시 | Voice onset/end | 최종 receipt | 실제 전달 문장 |
| --- | --- | ---: | --- | --- | --- |
| `P:1:efbaee52-4e9d-4fd4-8a6a-20ee9e8cf145` | RESPONSE | 6.551690s | 6.802621s / 14.804779s | true | 2026년 10월 15일 답변 요청이 기존 작업 대상 사용 불가로 인해 거부되었습니다. |

### S2-02 실제 model 호출

CINTERPRET=3, CRESPONSE=2. 아래 호출은 모두 attempt=1, HTTP 200이며 role은 원장의 값이다. schema 거절에 따른 재출력은 새 CINTERPRET 호출로 남는다.

| role 및 call ID | Context SHA-256 | 출력 SHA-256 | input/output token |
| --- | --- | --- | ---: |
| CINTERPRET `661e492f-74dc-4683-99f4-5bf91e5c10fc` | [Context](../workers/A1/model-contexts/661e492f-74dc-4683-99f4-5bf91e5c10fc.json) `c1563bfeb8d70a29ba9284ad3e8097ae572bd346d9325874befe787276206224` | [출력](../workers/A1/model-outputs/661e492f-74dc-4683-99f4-5bf91e5c10fc.json) `ebd3aeba3ad84cebb00894ab097afb5689f2018949df89fc789ec9999f5b2eae` | 4412/145 |
| CINTERPRET `0032782f-d3d6-41bc-bc40-c07a2435af84` | [Context](../workers/A1/model-contexts/0032782f-d3d6-41bc-bc40-c07a2435af84.json) `d3208c8973f907d880749bad1f9c921957ec18ac134043f91220ac5641e32544` | [출력](../workers/A1/model-outputs/0032782f-d3d6-41bc-bc40-c07a2435af84.json) `a7e5ce1f6de6fbbc7e9083f6dcf49c933361acb2700e11b1e44da9e60dd87ca5` | 4902/132 |
| CINTERPRET `71d8b1af-c81e-4a89-8b64-1159b9e99ece` | [Context](../workers/A1/model-contexts/71d8b1af-c81e-4a89-8b64-1159b9e99ece.json) `fb66dc3ef1aa0ff96c5765acd16a8ddad7fd95952f303959cc51e6dc77f34af4` | [출력](../workers/A1/model-outputs/71d8b1af-c81e-4a89-8b64-1159b9e99ece.json) `3847a3a436ec35a401f907d00b6652e303050fd548cabab1d9a624d222efdffa` | 5126/354 |
| CRESPONSE `f5e5996e-76f4-42eb-85a2-10098bbb3a3f` | [Context](../workers/A1/model-contexts/f5e5996e-76f4-42eb-85a2-10098bbb3a3f.json) `649017630c8d429a3e45b745611223e84729c0c2622e2d53b747efd1b226f016` | [출력](../workers/A1/model-outputs/f5e5996e-76f4-42eb-85a2-10098bbb3a3f.json) `d933a28e450ae58efebe3c41861f9ae84d4d2f895d31d048a55acf206f6eac2a` | 2907/19 |
| CRESPONSE `43a9750a-fb98-46e2-b20e-27abeec6433e` | [Context](../workers/A1/model-contexts/43a9750a-fb98-46e2-b20e-27abeec6433e.json) `26b83c84624e938729fa2f70d11e8969b2df37cf3b597df0f059a855d4d6c9e5` | [출력](../workers/A1/model-outputs/43a9750a-fb98-46e2-b20e-27abeec6433e.json) `9471ad009cced0dec6a03c55a5fdacd0b8ec8a64f0ed924ff26f5e0f52575baf` | 969/67 |

실제 downstream 명령 및 source/body:

- `CMD:47e315e5-d078-41cd-a380-fcd9502b3d6b` / CREATE_PROPOSAL / Task `T:b1f45a8f-3d60-43f3-b18f-8bae2efd1e8f` / Execution `X:6ffb479a-34a0-4462-a873-12a816de1874`
  send=47.955790s, ingress=47.955864s, receipt=48.158175s; payload SHA-256 `b8aaee7105bf335975086e07fd3c143b251fcf82e912685f93b5bfc95dd66f5a`. Sources=["D2@v2", "D3@v1"].

최종 전달 receipt. 숫자는 해당 scene의 실제 경계이며 반올림해 표시했다. Text와 Voice가 다르면 둘을 따로 기재한다.

| publication | 종류 | Text 제시 | Voice onset/end | 최종 receipt | 실제 전달 문장 |
| --- | --- | ---: | --- | --- | --- |
| `P:1:92b7ab4b-9b14-4de5-993a-d4694062db1a` | QUESTION | 6.881913s | 7.145292s / 40.986276s | true | 요청하신 '아까 들은 기준'과 '두 업무 결과' 중 '기준'에 해당하는 구체적인 내용이나 문서가 현재 컨텍스트에 명확히 식별되지 않습니다. 제안서 작성을 위해 참조해야 할 특정 배경 자료(B01-B20 등)나 이전 대화의 기준이 있다면 알려주세요. 또한, 생성할 제안서의 대상 독자(Audience)나 주요 포맷(FORMAT) 선호도가 있다면 함께 지정해 주세요. |
| `P:2:98b383a2-ebe4-49c4-a61c-26742c00e953` | RESPONSE | 49.278813s | 49.549356s / 56.054418s | true | CREATE_PROPOSAL 작업은 ACCEPTED(접수) 상태입니다. |
| `P:3:295a7f33-f038-48be-9309-401d74254ee3` | RESULT | 57.273571s | 57.552666s / 67.041037s | false | Text: CREATE_PROPOSAL 작업이 완료되었습니다. 제안서 내용: A는 설치 2일, 가격 100백만원/연간 유지비 3백만원이며, B는 설치 4일, 가격 120백만원/연간 유지비 5백만원입니다.<br>Voice: CREATE_PROPOSAL 작업이 완료되었습니다. 제안서 내용: A는 설치 2일, 가격 100백만원/ |

### S2-05 실제 model 호출

CINTERPRET=2, CRESPONSE=2. 아래 호출은 모두 attempt=1, HTTP 200이며 role은 원장의 값이다. schema 거절에 따른 재출력은 새 CINTERPRET 호출로 남는다.

| role 및 call ID | Context SHA-256 | 출력 SHA-256 | input/output token |
| --- | --- | --- | ---: |
| CINTERPRET `fe5a1cdd-a566-4918-9e9b-05ed776bf3fd` | [Context](../workers/A1/model-contexts/fe5a1cdd-a566-4918-9e9b-05ed776bf3fd.json) `154c0f71c509b11333c04293e6487f54b18355e381d08ff4d67863b14cb077f8` | [출력](../workers/A1/model-outputs/fe5a1cdd-a566-4918-9e9b-05ed776bf3fd.json) `d83f745535bc9ca8654ff6220a209addeab4d25e5b3c417cc57ff2d8b02bb21c` | 4361/197 |
| CINTERPRET `519a3c49-383b-4680-a011-a5ed1c6ab44d` | [Context](../workers/A1/model-contexts/519a3c49-383b-4680-a011-a5ed1c6ab44d.json) `027d9421ab5f323c1deaac43e47c6152edcbce0184f6fcdd9c2057932f01cff8` | [출력](../workers/A1/model-outputs/519a3c49-383b-4680-a011-a5ed1c6ab44d.json) `70819265ac20fb9afc98f11dec7a983ba65e414f80da1ed44cde17679c601adb` | 4873/409 |
| CRESPONSE `37018adc-9326-4baf-9d87-d419319aa14d` | [Context](../workers/A1/model-contexts/37018adc-9326-4baf-9d87-d419319aa14d.json) `15fb2c1ab5433fd740a96d41544479c93f84cee53385718bb252485a45dd3f83` | [출력](../workers/A1/model-outputs/37018adc-9326-4baf-9d87-d419319aa14d.json) `b6add8d9364342556ff6e27f48c41b16e2cf3d56b797ae65a05390cc7f54c67c` | 3175/21 |
| CRESPONSE `1841667b-c0f1-49d2-8e9c-3b9462da73e0` | [Context](../workers/A1/model-contexts/1841667b-c0f1-49d2-8e9c-3b9462da73e0.json) `da9cdb7768f539f8c3ab73aae4f396d238740e662e0a6b043d2e00257f27b628` | [출력](../workers/A1/model-outputs/1841667b-c0f1-49d2-8e9c-3b9462da73e0.json) `352d50febc1dae310838335bd01aa68ee69f07db4153fe0840f449bab632f0d8` | 1010/52 |

실제 downstream 명령 및 source/body:

- `CMD:e94ef11a-01d9-4725-9a6d-cdbcb1ee14c3` / CREATE_PROPOSAL / Task `T:973000a2-3a84-4ecb-8f5b-1f12a8ee199c` / Execution `X:08c33883-f6a5-44fe-89be-58bc569c7b93`
  send=7.564496s, ingress=7.564606s, receipt=7.767058s; payload SHA-256 `f2b5f228b773025b15db38c72e65d82892565bf7f71723ace3606e6cf0e5bec4`. Sources=["D새비교@v1", "D새견적@v1"].

최종 전달 receipt. 숫자는 해당 scene의 실제 경계이며 반올림해 표시했다. Text와 Voice가 다르면 둘을 따로 기재한다.

| publication | 종류 | Text 제시 | Voice onset/end | 최종 receipt | 실제 전달 문장 |
| --- | --- | ---: | --- | --- | --- |
| `P:1:db72271b-5034-4483-8a3a-3aa00e1b44eb` | RESPONSE | 8.888978s | 9.161503s / 17.503368s | true | CREATE_PROPOSAL 업무가 접수(ACCEPTED)되었습니다. 완료되지 않았습니다. |
| `P:2:53b87be6-166e-4a59-89dd-418ce8b8d96a` | RESULT | 16.488191s | 17.789730s / 32.129544s | true | CREATE_PROPOSAL 업무가 완료되었습니다. 결과: 제품 A의 납기는 10일, 제품 B의 납기는 20일<br>제품 A는 30백만원, 제품 B는 40백만원 |

### S3-02 실제 model 호출

CINTERPRET=3, CRESPONSE=4. 아래 호출은 모두 attempt=1, HTTP 200이며 role은 원장의 값이다. schema 거절에 따른 재출력은 새 CINTERPRET 호출로 남는다.

| role 및 call ID | Context SHA-256 | 출력 SHA-256 | input/output token |
| --- | --- | --- | ---: |
| CINTERPRET `e67e86d9-b6a2-4242-b739-e5a42b1463ca` | [Context](../workers/A1/model-contexts/e67e86d9-b6a2-4242-b739-e5a42b1463ca.json) `e4fd00c235a40ce40cba66fc129e43ee7a311ada117b332f4f86edc2cca92f09` | [출력](../workers/A1/model-outputs/e67e86d9-b6a2-4242-b739-e5a42b1463ca.json) `a5a2a52fd154f9ba3782a303f59cd76d0e9b6ad4348d6ee8bcd8f89d7fad5575` | 4407/90 |
| CINTERPRET `36eedb2c-ce6d-46ff-b5ab-dabfe981d8f9` | [Context](../workers/A1/model-contexts/36eedb2c-ce6d-46ff-b5ab-dabfe981d8f9.json) `fe9b9b852c8f3f104ade5c973cbd45dbb493dc6d4b6330d1c0fd5e91cd1b0d46` | [출력](../workers/A1/model-outputs/36eedb2c-ce6d-46ff-b5ab-dabfe981d8f9.json) `5052d95c4586ec24a8d40235ab92163f98fc213942796238b389ba6c0e328007` | 4637/353 |
| CINTERPRET `fab835e5-23ec-44ba-999d-fc51ed4482e8` | [Context](../workers/A1/model-contexts/fab835e5-23ec-44ba-999d-fc51ed4482e8.json) `efe766e4e7a505709dc7fc09618ef4d1f4cb61f3fc9282df03ecff0d0892fd41` | [출력](../workers/A1/model-outputs/fab835e5-23ec-44ba-999d-fc51ed4482e8.json) `7aeb75e468351af34cf7258c38d52ed25714f85e42f96533b56f8cd4b79b5e9d` | 4876/200 |
| CRESPONSE `c86b6a09-f6dd-40aa-9f05-cdfa30731ff9` | [Context](../workers/A1/model-contexts/c86b6a09-f6dd-40aa-9f05-cdfa30731ff9.json) `ce42997a000a5bb9b8bfc62a21c94efcab6997b757813880e0608d2333e6b7dd` | [출력](../workers/A1/model-outputs/c86b6a09-f6dd-40aa-9f05-cdfa30731ff9.json) `fe3a0ef38d7f706b20ec6d9edd83136dd9566188d0fb2b2ee43dff2657b34e58` | 2384/20 |
| CRESPONSE `5064b03a-5dfa-456c-acd1-f084272df86d` | [Context](../workers/A1/model-contexts/5064b03a-5dfa-456c-acd1-f084272df86d.json) `8cd6954feabdd3924a5f0811c4d38c725d39bd84d5a2394efc49bceb5249ccf2` | [출력](../workers/A1/model-outputs/5064b03a-5dfa-456c-acd1-f084272df86d.json) `b5609ca28ea0216fbe29067a74fd808e5ceaa2794f82ec61cd1d948a483ac0c6` | 773/17 |
| CRESPONSE `64cbedb2-2706-4ed8-9b7a-190eced61f8d` | [Context](../workers/A1/model-contexts/64cbedb2-2706-4ed8-9b7a-190eced61f8d.json) `f8834055a2ccc6f642a52bfd51db79c5eef805870d9b1d0444619b066c1b4bac` | [출력](../workers/A1/model-outputs/64cbedb2-2706-4ed8-9b7a-190eced61f8d.json) `0152a76fc4f56bcfa947ba7ded9b3760c9fb91ee6d95c5b4b5db8f094a4a2857` | 2957/19 |
| CRESPONSE `a1d36a83-cbfa-46b5-aa6d-172c79dfd73e` | [Context](../workers/A1/model-contexts/a1d36a83-cbfa-46b5-aa6d-172c79dfd73e.json) `46ff9448ab704aec9e6d797239ca14be55ebce70e6f75a43a375326e22407334` | [출력](../workers/A1/model-outputs/a1d36a83-cbfa-46b5-aa6d-172c79dfd73e.json) `d7dbfd3b674d52ff9e6022bdd3b01a05365110f2faaa60af88e847fa1af71db2` | 897/34 |

실제 downstream 명령 및 source/body:

- `CMD:248dd56b-25e5-413f-bde8-fee6fbe1e4ff` / SUMMARIZE / Task `T:32227ffe-18fa-4574-aef9-a33d79c00e1e` / Execution `X:8633c027-f16c-4b38-aed9-a6ee1bd100a1`
  send=8.710192s, ingress=8.710259s, receipt=8.911909s; payload SHA-256 `aa37a4727cce9c2c663c5f07a88ef0084cde6f69ddf680ec1f71bb6ecfd7f8de`. Sources=["D1@v0"].
- `CMD:63a29d6a-ad46-4682-83b4-d8f5ec0c151b` / SEND_MAIL / Task `T:c72a9b33-b336-408c-83b6-adc4bcba7df3` / Execution `X:b5048246-8b70-4f2d-9c23-2ea6f91f3f97`
  send=9.239174s, ingress=9.239263s, receipt=9.441984s; payload SHA-256 `e5d86b9b1eb07a9d578f2c70b4c615e79b108d06fb04647c0712a996ccdcde6b`. Sources=[].

최종 전달 receipt. 숫자는 해당 scene의 실제 경계이며 반올림해 표시했다. Text와 Voice가 다르면 둘을 따로 기재한다.

| publication | 종류 | Text 제시 | Voice onset/end | 최종 receipt | 실제 전달 문장 |
| --- | --- | ---: | --- | --- | --- |
| `P:1:a72f8b7a-8a44-470a-93ea-b08a604ef0af` | RESPONSE | 9.826820s | 10.092718s / 16.598160s | true | D1 요약 업무는 접수(ACCEPTED)되었으며, 완료되지 않았습니다. |
| `P:2:54642f22-c5fe-4a62-8ac8-4750faef305a` | RESULT | 10.142917s | 16.873542s / 20.047650s | true | D1 요약 완료: 마감 10월 1일 |
| `P:3:6966590e-f839-4df7-bae8-59ee0b3d40fa` | RESPONSE | 10.302921s | 20.319275s / 27.657830s | true | SEND_MAIL 업무는 접수(ACCEPTED)되었습니다. 완료되지 않았습니다. |
| `P:4:2f9aae76-13d2-4886-abc2-9fd669daf1a4` | RESULT | 18.135871s | 27.943460s / 39.948303s | true | SEND_MAIL 업무가 완료되었습니다. 수신자 minsu@example.invalid에게 '마감 10월 1일'이 발송되었습니다. |
