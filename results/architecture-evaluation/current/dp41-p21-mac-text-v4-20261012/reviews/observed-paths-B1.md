# v4 B1의 실제 Request 해결 경로 관찰

읽기 기준: 2026-10-11 20:18 UTC, 2026-10-12 05:18 KST. B1에서 완료 기록이 나온 S1-01, S2-02, S2-05, S3-02만 읽었다. `OBSERVED`와 `EPISODE_TIMEOUT`은 runner의 종료 기록이다. C1~C6의 통과/실패, QA 점수, 정답 판정과 A/B 우열은 확정하지 않는다. [A1 관찰](./observed-paths-A1.md)과 같은 근거 경계를 사용한다.

[Campaign manifest](../manifest.json)의 고정 source digest는 `c2dfc26fa6afbce8df4af3405913031ca440fefa150d6a0739c010b3ed4796a7`다. [B1 원시 trial 원장](../workers/B1/trials-unjudged.jsonl)의 해당 줄과 작은 Context/provider 출력만 읽었다. 마지막 LF를 포함한 raw 줄 hash 및 정규 JSON Context/출력 hash를 아래에 보존한다. 13개 실제 호출의 Context/출력 hash와 3개 실제 downstream 명령 payload hash를 대조했다. 모든 해당 호출의 원장은 attempt=1, HTTP 200이며 의미 판정 phase는 실제 출력/Context와 candidate 사건에서 확인한다. ledger의 purpose=FOREGROUND만으로 Frame와 부분 판단을 구별하지 않는다.

**P2 발견:** 유효한 `selector.query`가 B의 Frame→read 변환에서 `mention`으로 대체되어 실제 owner 검색에 전달되는 계약 통합 결함을 확인했다. 별도 계약 agent도 독립 재확인했다. §2에 raw 연결과 위치를 기재했다. 원장을 고치거나 같은 출력의 결과를 보정하지 않는다. 이 실행 관찰의 QA 취급 및 새 freeze 여부는 campaign owner가 결정한다. 검색 품질 한계나 B의 구조상 필수 비용으로 이 결함을 설명하지 않는다.

보고서 작성 종료 시 campaign owner는 위 P2로 v4 실행을 중단하고 원본을 보존했다. 추가로 B 질문 표시의 candidate branch가 실제 item.metadata의 label을 소비하지 않는 연결 문제도 최소 수정 범위로 지정했다. 아래 긴 ref의 실제 표시 비용을 구조의 필수 비용으로 확대하지 않는다. owner interaction item에는 publication_id와 실제 channel text가 있으나 title/instruction/name이 없다는 shape도 후속 계약 검토에 전달했다. 이 보고서는 수정 후 실행 결과를 포함하지 않는다.

음성은 공통 출력 모형의 실행이다. 실제 클라우드 음성 품질의 증거가 아니다. 최종 Text, actual Voice와 전달 receipt를 별도로 본다. 실제 질문 Voice가 끝나지 않았으면 예정된 사용자 답을 미리 주입하지 않는다. 아래 네 trial에서는 취소된 옛 모델 job이 생성되지 않았고 늦은 모델 반환 경로도 발생하지 않았다.

## 1. 지원 표현과 실제 모델 관측의 구별

[고정 prompts](../frozen-source/prototypes/candidates/dp41_45_poc/dp41/prompts.py)는 Frame에서 Goal, 유한 operation, reference, 관계와 조건을 만들도록 하고, [고정 B 후보](../frozen-source/prototypes/candidates/dp41_45_poc/dp41/b.py)는 실제 owner read를 예약하고 항목 상태를 갱신한다. 부분 모델은 하나의 reference 후보를 선택하거나 uncertain을 반환한다. 부분 판단으로 Goal/value, 관계와 전체 Meaning을 새로 만드는 기능은 없다. 확인 답 이후 새 Frame을 만드는 경로는 있으나 S2 두 trial에서는 그 답의 도착까지 실행되지 않았다.

S1-01과 S3-02는 reference가 실제 하나의 후보로 반환되어 한 번의 초기 CINTERPRET 이후 code binding이 이루어진 경로다. B의 모든 Request가 한 번의 의미 호출로 끝난다는 증거가 아니다. S2-02는 초기 Frame와 세 부분 판단, S2-05는 초기 Frame와 한 부분 판단을 실제 호출했다.

현재 계약으로 CREATE_PROPOSAL Goal에 실제 source refs를 넣고 도메인 작성 지시를 위임할 수 있다. 들은 범위를 원천으로 넘길 때 `PAST_DELIVERED(source_ref=publication_id)`를 사용하여 실제 Voice 범위 source를 받는 지원 경로도 선언되어 있다. S2-02 모델이 선택한 `RECENT_INTERACTION`은 Text와 실제 Voice의 두 channel을 보존하는 view이며 자동으로 heard-only source가 되는 연산이 아니다. 표현 가능한 경로의 존재와 실제 그 경로를 사용했다는 관측은 구별한다.

## 2. P2: 지원 query의 Frame→owner-read 전달 불일치

[고정 contract](../frozen-source/prototypes/candidates/dp41_45_poc/dp41/contract.py) `reference_spec` 285~296행은 `query=ref.get('query', ref.get('mention',''))`를 만든다. [고정 Owner](../frozen-source/prototypes/candidates/dp41_45_poc/shared/owners.py) `read` 339~343행은 `spec.query`를 `selector.query`보다 우선한다. 이 조합은 Frame reference에 top-level query가 없을 때 명시된 selector.query를 mention으로 가린다. B는 이 mapping을 사용하며 A READ는 직접 read spec을 전달한다.

지원 근거는 COMMON prompt 16/18/19행의 `RECENT_INTERACTION={op,query}`, `PAST_DELIVERED={op,query?,...}`, `TASK_RESULT={op,query?,...}`, B prompt 68~73행의 reference selector 및 contract의 validate_read 94~100행에 있는 selector.query 허용이다. [현재 04-41](../../../../../docs/architecture/12-decisions/decision-packages/04-41-request-resolution-control.md)의 B Frame 예시도 selector.query를 사용한다. 따라서 이번 값은 지원되지 않는 임의 expression으로 볼 수 없다.

| 실제 tuple | Frame 모델의 유효 selector.query | 실제 OWNER_READ spec.query | 증거 |
| --- | --- | --- | --- |
| B1/S2-02/r-criteria | criteria or standards discussed in recent interactions | 아까 들은 기준 (criteria heard earlier) | Frame E2761, OWNER_READ E2766, read_id `c36ecc3b-1366-480f-aa9c-a0b6ee327ec5` |
| B1/S2-02/r-task-result-1 | completed task result related to comparison or quote | 두 업무 결과 중 첫 번째 (first of two work results) | E2761→E2772, read_id `417d112b-64fb-4aa2-887e-505bc063e294` |
| B1/S2-02/r-task-result-2 | completed task result related to comparison or quote | 두 업무 결과 중 두 번째 (second of two work results) | E2761→E2778, read_id `2e403c81-ef5d-4ff1-a87c-2956c53c0634` |
| B1/S2-05/r-baseline | baseline criteria discussed in recent interactions | 전에 들은 기준 (Previously heard baseline) | Frame E6295, OWNER_READ E6300, read_id `4a66755a-91d7-41bf-a35a-5934acf95825` |

S2-02의 실제 Frame call은 `24b6eab6-e71e-4f20-9c04-b38c93858b38`, S2-05는 `237fed4b-27de-476d-9d2c-fec09add6446`이다. 아래 Context/출력 원장 링크와 hash로 원래 모델 값을 재확인할 수 있다. contract SHA-256=`5415d2ed95b55dccb62740d3dc7b5502be3567c31add4f9a14a7820938bb392f`, prompts=`be366f35926aebc73ba6ce9f1c9ba526366145518090b99d2aa28eee4ee8590c`, owners=`08d4e5d421c8e19eeafb01b214fd7af2b894aef23a64794f9d9f9666e8fc376e`.

정확한 selector.query를 전달하면 더 나은 후보가 나올지는 실행하지 않았다. 이 P2는 검색 결과가 나쁘다는 추정이나 정답 source를 강제해야 한다는 주장이 아니다. 지원 input과 실제 실행 input이 달라진 통합 문제다. Owner의 lexical ranking, bounded coverage와 모델의 selector 선택은 별도 관측으로 남는다.

## 3. S1-01: Frame의 READ_FIELD를 code가 실제 D1에 결합함

입력은 “D1에 적힌 마감일 알려줘.”다. 첫 Context에는 D1@v1 handle와 권한만 있고 본문은 없다. `E113` Frame은 DIRECT_RESPONSE/READ_FIELD, field=DEADLINE, source_refs=[r-document-d1] 및 EXPLICIT_SOURCE/DOCUMENT/D1 reference를 만든다. `E117`의 read receipt `db396e62-9e28-4242-986c-df112c78e386`는 실제 D1 원문 “제출 마감일은 2026년 10월 15일입니다. 담당자는 민수입니다.”를 반환한다.

`E120`의 code 상태가 r-document-d1을 D1@v1에 BOUND로 바꾸고 `E124`가 adoptedMeaning을 만든다. operation, field, Goal ID와 instruction은 원 Frame 값이며 source_refs만 실제 ref로 결합됐다. 원문 조회 뒤 전체 Meaning을 모델에게 재작성시키지 않았다. source composition 뒤 CRESPONSE Context는 READ_FIELD Goal과 실제 D1 source 본문을 받는다. 이 역할은 사용자가 듣는 자유 응답 문장을 만든다. 따라서 “조회 뒤 모델 호출 없음”으로 표시하지 않는다.

Downstream 명령은 없다. 최종 Text와 Voice는 “D1의 제출 마감일은 2026년 10월 15일입니다.”이며 전달 receipt=true다. A1이 선택했던 ANSWER_QUESTION과 B1이 선택한 READ_FIELD의 차이는 실제 model output의 operation 차이다. 이번 단일 경로만으로 구조가 항상 같은 선택을 만들거나 전체 정확성이 더 높다고 결론낼 수 없다.

## 4. S2-02: 실제 channel view 선택 후 두 Task 결과 참조가 미해결로 남음

첫 Frame `E2761`은 NEW_TASK/CREATE_PROPOSAL 하나와 r-criteria, r-task-result-1, r-task-result-2 source refs를 만든다. 실제 들은 기준과 두 결과가 무엇인지 명확하지 않다는 unresolved 두 개도 모델이 넣었다. 처음 Context에는 E1 handle와 interrupted/presented metadata, T비교/T견적의 상태가 있지만 정답 기준 본문은 없다.

`E2766`는 RECENT_INTERACTION에서 E1 및 배경 publication 후보를 실제 읽었다. E1 receipt의 opaque ref는 `via-interaction:RECENT_INTERACTION:E1@v1:caabb613aeb7298e2512d1b400d4646e66c9244e95d27a35329a610744323e67`이다. content.text는 유지비/설치 기간, 가격 제외와 보안 등급 추가 문장까지의 전체 Text다. content.delivered_voice_text는 유지비/설치 기간과 가격 제외 두 문장이다. provenance.scope=INTERACTION_CHANNELS, origin publication=E1, revision=1, actual Text hash=`fcab5103bf61b64cba35e2c4d2bb67627508ff7161a00a859fb185bd52900661`, actual Voice hash=`d42c02f5913d24e6dd1d645eabead8f51cd361809c9a2bbddc621f5e16fe244c`를 보존한다.

실제 partial Context에 이 두 channel과 read provenance가 들어갔고, `E2821` 부분 모델은 해당 E1 opaque ref를 uncertain=false로 선택했다. `E2822` code binding도 같은 view를 보존한다. E1 원본 문자열 ref로 손실 변환한 흔적은 없다. 다만 RECENT 채널 view를 선택한 사실만으로 실제 들은 범위만 downstream에 위임됐다고 주장할 수 없다. 아직 명령 payload는 없다.

두 Task reference는 동일한 TASK_RESULT query selector이며 각 actual receipt `E2772`/`E2778`가 D2@v2와 D3@v1을 함께 반환한다. 두 partial Context에 실제 설치 및 견적 본문과 read ID가 들어갔다. `E2854`, `E2888`의 출력은 모두 candidate_ref=null, uncertain=true이며 code는 각 reference를 AMBIGUOUS로 남겼다. 임의로 첫 결과와 두 번째 결과를 서로 다른 정답으로 채워 넣지 않았다. 이 출력은 실제 ambiguous 요청/후보 집합에 대한 좁은 선택 판단이다. Task 결과 원천을 전달할 유한 source_refs 기능이 없는 경우로 볼 수 없다. 지원되는 explicit Task locator도 있었다. 단, §2의 실제 query 대체 결함이 있으므로 현재 반환 후보를 원래 모델 query의 결과로 표시하지 않는다.

`E2890` code가 질문을 만들었고 내부 opaque ref 여덟 개를 질문 Text에 그대로 넣었다. 이는 이번 B 후보의 표시 구현 선택이다. B 구조가 반드시 이 긴 ref를 사용자에게 읽어야 하는 것은 아니다. 최종 Voice는 63.048313초까지 질문 일부를 읽다가 automatic watchdog으로 중단되어 receipt=false다. 최종 Text는 전체 질문이 있다. 실제 followup input, 새 Frame, adoptedMeaning 및 downstream command는 없다. 따라서 실제 CREATE_PROPOSAL 소비를 성공했다거나 잘못된 payload를 보냈다고 만들어 해석하지 않는다.

## 5. S2-05: 세 원천의 실제 도착과 검색 coverage 및 Frame 구성은 다른 단계임

E새, D새비교, D새견적이 `E6198` 2.972087초, `E6202` 2.981804초, `E6206` 2.991931초에 실제 도착했다. 발화 종료 3.000654초와 전사 완료 3.000703초보다 앞선다. 초기 Context에 세 source handle와 E새 delivered metadata가 모두 있다. source 도착 schedule이나 본문을 무료로 미리 제공하지 않았다.

`E6295` Frame은 COMPARE Goal와 CREATE_PROPOSAL Goal 두 개다. r-baseline의 selector는 RECENT_INTERACTION이며 두 새 문서는 EXPLICIT_SOURCE다. 두 instruction에는 가격 평가 제외가 있다. 모델이 만든 KEEP_PROPERTY의 property는 `exclude_price_evaluation`, value=true다. 이 도메인 flag는 원 Frame에 기록된 값이며 CREATE_PROPOSAL에서 보장된 실행 연산을 뜻하지 않는다. 이번 경로는 adoption 전 종료되어 이 constraint의 실행 효과가 관측되지 않았다. 2주 이내 우선 조건도 아직 owner 결과에서 얻지 못했다.

`E6300`의 기준 검색은 H0005~H0010의 실제 배경 channel view 여섯 개를 반환했다. E새는 포함되지 않았다. §2의 nested query 전달 결함을 빼고도 공통 Owner의 lexical scoring, limit 전후의 권한 처리 및 coverage가 실제 반환 집합을 제한한다. 이것은 source 자체가 없거나 아직 도착하지 않았다는 뜻이 아니다. E새의 실제 handle을 사용한 PAST_DELIVERED 경로는 지원되어 있다. 이번 모델이 선택한 selector와 반환 후보에서 그 경로가 실행되지 않았다.

`E6306`과 `E6312`는 D새비교@v1의 A=10일/B=20일 및 D새견적@v1의 A=30/B=40백만원 본문을 실제 읽었다. r-baseline partial의 실제 Context에는 배경 후보만 있으며 `E6348`은 null/uncertain=true를 반환했다. code는 기준을 AMBIGUOUS로, 두 문서 ref를 BOUND로 남긴 뒤 `E6352`에서 질문했다.

Frame의 COMPARE source_refs에는 r-baseline이 있지만 CREATE_PROPOSAL에는 r-new-product와 r-quotation만 있다. 두 Goal 사이에 USE_RESULT나 AFTER 관계도 없다. 이 원 Frame의 source/관계 설계는 query 결함과 별개로 보존해야 하는 모델 선택이다. 그러나 actual Meaning/명령이 없으므로 후보가 실제로 이 잘못된 위임 효과를 발생시켰다고 단정하지 않는다.

질문은 배경 opaque ref와 문서 ref를 노출하며 Voice 일부만 끝나 receipt=false다. followup, adoptedMeaning, downstream payload 및 Agent 산출물은 없다. 참조 선택 부분 모델의 Goal/value 미지원 한계는 고정 구현의 선언한 범위지만, 이번 timeout을 모두 그 한계의 필연적 결과로 설명할 수 없다. actual query 통합 결함, 후보 Frame의 selector/source 구성과 질문 표시 구현을 각각 구분한다.

## 6. S3-02: 현재 원본과 현재 Goal 결과를 각각 code가 결합함

입력은 “D1을 요약한 다음, 그 요약을 민수에게 메일로 보내줘.”다. 초기 Context에 D1@v1, D1@v0, 민수@v1 handle와 current source vector D1=1이 있다. `E9261` Frame은 EXPLICIT_SOURCE/D1 및 EXPLICIT_SOURCE/CONTACT/민수, SUMMARIZE Goal와 SEND_MAIL Goal, AFTER 관계 및 body={result_of:G-summarize}를 만든다.

`E9265`는 실제 현재 D1@v1 원문 “하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다.”를, `E9271`은 실제 연락처 minsu@example.invalid를 반환했다. 각 후보가 하나라 부분 모델 호출 없이 BOUND가 됐다. `E9279` adoptedMeaning은 Goal의 kind/operation/instruction과 AFTER를 그대로 보존하면서 두 ref를 실제 refs에 결합했다. SEND body는 code가 GOAL_RESULT로 정규화하여 producer_goal_id=G-summarize, 이번 request_id 및 producer_revision=1을 고정했다. 모델이 마지막 전체 Meaning을 재작성한 경로가 아니다.

실제 Summary 명령은 D1@v1 source 본문을 받았고 `E9303`이 새 artifact `A:54d554e7-2324-4d0e-b3c5-29aecfd12f6f`를 생산했다. SEND_MAIL의 실제 body_artifact는 이 artifact 전체와 동일 text, actual source_refs 및 payload hash를 보존했다. 실제 recipient는 minsu@example.invalid다. `E9549`의 SENT artifact도 같은 body_artifact_id와 본문을 가진다. A1과 B1 모두 현재 producer의 실제 artifact 연결 경로가 있었고, 모델의 원본 selector는 A1의 revision=0과 B1의 현재 EXPLICIT_SOURCE로 달랐다. 이것으로 반복 전체에서의 우열을 단정하지 않는다.

옛 결과 `E9183`, 현재 결과의 재적용 `E9309`, 중복 `E9320`은 각각 거절/무시됐다. old_job_created=false, late_return_exercised=false라 취소된 모델 job의 late-return 분기는 직접 관측되지 않았다. 접수/요약완료/발송접수/발송완료 publication 네 개는 실제 전달 receipt=true다.

발송 접수 CRESPONSE의 actual Text/Voice에는 “민수@example.invalid”라고 표기되어 있지만 실제 명령의 recipient는 “minsu@example.invalid”다. 명령 payload와 자유 응답 문장 사이의 차이를 보존하며 명세별 판정은 judge에서 별도로 확인한다. stub이나 명령의 수신자를 자유문에 맞춰 바꾸지 않는다.

## 7. 해석 한계

S1과 S3은 지원되는 유한 표현으로 실제 읽기, code binding, 원본 payload와 현재 producer/consumer를 연결한 메커니즘 증거다. S2는 Frame와 부분 판단까지만 실행됐다. source refs와 Goal/조건을 모델이 실제로 선언한 값, 코드가 실제 정규화한 값, actual Agent가 소비한 값을 구별해서 표에 보존한다.

이번 P2는 declared query의 전달 문제이며 구조의 필연적 지원 한계가 아니다. B 확인 질문의 내부 ref 노출도 표시 구현 선택이다. 공통 Owner의 bounded lexical 검색은 semantic search를 보장하지 않으므로 해당 coverage 한계를 별도로 공개해야 한다. P2를 바로잡은 실행을 하지 않았고 불확실한 결과를 정답으로 고치지 않았다. 전체 QA, 구조적 의미 정확성의 우열과 실제 Agent 작업 품질은 이 네 경로 리뷰만으로 확정하지 않는다.

## 8. 재조회용 ID, Frame, 명령 및 channel receipt
| tuple | raw 줄 | 종료 기록 | root Request | raw 줄 SHA-256 |
| --- | ---: | --- | --- | --- |
| B1/S1-01/repeat=1 | 1, E395 | OBSERVED | `R:423310c2-2c8b-465f-aac4-a9f94473efb0` | `214f3f6ae0b7ed81a916fdcdcbe708dab2fd975e8eedaa6c13ecb1caf3bdb260` |
| B1/S2-02/repeat=1 | 4, E4263 | EPISODE_TIMEOUT | `R:5e61b796-1101-4287-88ec-b8b864c2dd9f` | `b3a7419e2ccff2eddc29dbaeb8f760f0bbb180dd2b0ae52956dbc7a1f21c7fed` |
| B1/S2-05/repeat=1 | 7, E7795 | EPISODE_TIMEOUT | `R:a2724967-b855-40b0-bf1b-0dccc36811db` | `68770fda2aea698c6fd054f36da21c053ee7e3ea0d81c660c3b67b6f4ce39cf7` |
| B1/S3-02/repeat=1 | 9, E10704 | OBSERVED | `R:86aee18a-a18a-4cda-975d-97ab64338907` | `53058239454d35194aaad59dd15028dd9957195d5138295e9d234cfd381821af` |

### S1-01 실제 호출과 유한 값

CINTERPRET=1, CRESPONSE=1.

| phase, call ID | Context SHA-256 | 출력 SHA-256 | input/output token |
| --- | --- | --- | ---: |
| FRAME `d8064fe4-59c8-4f3a-a60e-fab833f0d619` | [Context](../workers/B1/model-contexts/d8064fe4-59c8-4f3a-a60e-fab833f0d619.json) `f68ee4b54a474ea66317ac589036e918054bb967c4c1bc2684ce98064d40a553` | [출력](../workers/B1/model-outputs/d8064fe4-59c8-4f3a-a60e-fab833f0d619.json) `25e5ee4b5edd238a0c7233693a928f0e692b2021caa4c99f31659a522c8749bc` | 4462/276 |
| CRESPONSE `3ce16648-fb64-4cd7-aecf-f9b022dcfa6e` | [Context](../workers/B1/model-contexts/3ce16648-fb64-4cd7-aecf-f9b022dcfa6e.json) `403187208c7346a59226b0e17fdb97d2c2a9927f205716e76a7fcccb56114e88` | [출력](../workers/B1/model-outputs/3ce16648-fb64-4cd7-aecf-f9b022dcfa6e.json) `a090b2d6ca63737d70764b7d4ec52cd6336c133f4e448e025713a377fbbac4f3` | 1281/26 |

원장에 남은 실제 Frame와 부분 판단. 조회 전 Goal/value 및 선택한 query를 그대로 보존한다.

`E113` dp41_b_frame_produced, scene=4.300140s:

```json
{
  "constraints": [],
  "goals": [
    {
      "arguments": {
        "field": "DEADLINE",
        "source_refs": [
          "r-document-d1"
        ]
      },
      "id": "G-read-d1-deadline",
      "instruction": "Read the DEADLINE field from document D1 to answer the user's question.",
      "kind": "DIRECT_RESPONSE",
      "operation": "READ_FIELD",
      "target_ref": null
    }
  ],
  "input_ref": "I:8032ab5a-5120-4886-b149-e59cac8cc752",
  "preservations": [],
  "references": [
    {
      "id": "r-document-d1",
      "mention": "D1",
      "selector": {
        "name": "D1",
        "op": "EXPLICIT_SOURCE",
        "source_kind": "DOCUMENT"
      }
    }
  ],
  "relations": [],
  "response": null,
  "unresolved": [],
  "version": "dp41-frame-runtime-1"
}
```

채택 Meaning의 실제 Goal 값:

```json
{
  "meaning": {
    "constraints": [],
    "evidence_refs": [
      "db396e62-9e28-4242-986c-df112c78e386"
    ],
    "goals": [
      {
        "arguments": {
          "field": "DEADLINE",
          "source_refs": [
            "D1@v1"
          ]
        },
        "id": "G-read-d1-deadline",
        "instruction": "Read the DEADLINE field from document D1 to answer the user's question.",
        "kind": "DIRECT_RESPONSE",
        "operation": "READ_FIELD",
        "target": null
      }
    ],
    "preservations": [],
    "relations": [],
    "response": null,
    "unresolved": [],
    "version": "dp41-meaning-runtime-1"
  },
  "request_id": "R:423310c2-2c8b-465f-aac4-a9f94473efb0"
}
```

실제 downstream 명령:

없음. DIRECT_RESPONSE 또는 adoption 전 종료로 구별해서 본문에 설명했다.

최종 실제 channel receipt. scene 경계는 반올림했으며 Text와 Voice가 다르면 둘 다 보존한다.

| publication | 종류 | Text 제시 | Voice onset/end | 최종 receipt | 실제 전달 문장 |
| --- | --- | ---: | --- | --- | --- |
| `P:1:3bf300f8-48de-480d-93c6-70998c7154dc` | RESPONSE | 5.340135s | 5.603750s / 10.442023s | true | D1의 제출 마감일은 2026년 10월 15일입니다. |

### S2-02 실제 호출과 유한 값

CINTERPRET=4, CRESPONSE=0.

| phase, call ID | Context SHA-256 | 출력 SHA-256 | input/output token |
| --- | --- | --- | ---: |
| FRAME `24b6eab6-e71e-4f20-9c04-b38c93858b38` | [Context](../workers/B1/model-contexts/24b6eab6-e71e-4f20-9c04-b38c93858b38.json) `42d263c20f7a1813f02eedb650edf5b91632b032cdf345bf37f0c367823c09ee` | [출력](../workers/B1/model-outputs/24b6eab6-e71e-4f20-9c04-b38c93858b38.json) `f427dad87891d354193ee1225f84bcff9f11ee05d4f7052591abdabd27d6f9a8` | 4505/570 |
| B_PARTIAL_SELECT `1dcdde94-e7f8-4518-adca-c84c2e524bbf` | [Context](../workers/B1/model-contexts/1dcdde94-e7f8-4518-adca-c84c2e524bbf.json) `dacce2a26261e65e3b7a7cfe6d115cb914146b22b4c157dd1c7ab8a92533fe32` | [출력](../workers/B1/model-outputs/1dcdde94-e7f8-4518-adca-c84c2e524bbf.json) `44d5e9a9024d9983392bcddbf12753fba0b4c80dacd11beee8b356e08117093f` | 6211/148 |
| B_PARTIAL_SELECT `e021ab0b-899c-4a9b-8eed-877f727e852e` | [Context](../workers/B1/model-contexts/e021ab0b-899c-4a9b-8eed-877f727e852e.json) `a98f88d4688b78f7ca55366a8a2e14091ccae88bf9a84aa0ef4aa0d5fb873cd9` | [출력](../workers/B1/model-outputs/e021ab0b-899c-4a9b-8eed-877f727e852e.json) `e04056aa62e967e8da57040f2b5b1a6506a62f4541755f3b3619b44e1c1473ff` | 445/79 |
| B_PARTIAL_SELECT `cfac1b5b-3479-46ea-90cf-6f07b6a0b73e` | [Context](../workers/B1/model-contexts/cfac1b5b-3479-46ea-90cf-6f07b6a0b73e.json) `6b444937b36a54f7523c9883e74f74717da5ba1b1ed2499dfe4ce9109072008e` | [출력](../workers/B1/model-outputs/cfac1b5b-3479-46ea-90cf-6f07b6a0b73e.json) `bbd77ff0f7ed5c2ed93f868ca6aba480bc7244e562d7ff9a1df417a7d3219095` | 445/79 |

원장에 남은 실제 Frame와 부분 판단. 조회 전 Goal/value 및 선택한 query를 그대로 보존한다.

`E2761` dp41_b_frame_produced, scene=6.703008s:

```json
{
  "constraints": [],
  "goals": [
    {
      "arguments": {
        "properties": {},
        "source_refs": [
          "r-criteria",
          "r-task-result-1",
          "r-task-result-2"
        ]
      },
      "id": "G-create-proposal",
      "instruction": "Create a new proposal based on the criteria and two task results.",
      "kind": "NEW_TASK",
      "operation": "CREATE_PROPOSAL",
      "target_ref": null
    }
  ],
  "input_ref": "I:859a1b90-9ffc-49f4-8dec-ccfb43df7eb4",
  "preservations": [],
  "references": [
    {
      "id": "r-criteria",
      "mention": "아까 들은 기준 (criteria heard earlier)",
      "selector": {
        "op": "RECENT_INTERACTION",
        "query": "criteria or standards discussed in recent interactions"
      }
    },
    {
      "id": "r-task-result-1",
      "mention": "두 업무 결과 중 첫 번째 (first of two work results)",
      "selector": {
        "op": "TASK_RESULT",
        "query": "completed task result related to comparison or quote"
      }
    },
    {
      "id": "r-task-result-2",
      "mention": "두 업무 결과 중 두 번째 (second of two work results)",
      "selector": {
        "op": "TASK_RESULT",
        "query": "completed task result related to comparison or quote"
      }
    }
  ],
  "relations": [],
  "response": null,
  "unresolved": [
    {
      "field_ref": null,
      "reason": "The specific 'criteria' mentioned as 'heard earlier' are not explicitly identified in the current context. The system needs to know which interaction or document contains the criteria.",
      "source_ref": null
    },
    {
      "field_ref": null,
      "reason": "The user refers to 'two work results' but does not specify which tasks constitute these results. There are multiple completed tasks (HT193-HT200, T비교, T견적). The system needs clarification on which specific task results to use.",
      "source_ref": null
    }
  ],
  "version": "dp41-frame-runtime-1"
}
```

`E2821` dp41_b_partial_returned, scene=8.330781s:

```json
{
  "candidate_ref": "via-interaction:RECENT_INTERACTION:E1@v1:caabb613aeb7298e2512d1b400d4646e66c9244e95d27a35329a610744323e67",
  "evidence_refs": [
    "c36ecc3b-1366-480f-aa9c-a0b6ee327ec5"
  ],
  "field_ref": "r-criteria",
  "uncertain": false
}
```

`E2854` dp41_b_partial_returned, scene=9.451208s:

```json
{
  "candidate_ref": null,
  "evidence_refs": [
    "417d112b-64fb-4aa2-887e-505bc063e294"
  ],
  "field_ref": "r-task-result-1",
  "uncertain": true
}
```

`E2888` dp41_b_partial_returned, scene=10.586407s:

```json
{
  "candidate_ref": null,
  "evidence_refs": [
    "2e403c81-ef5d-4ff1-a87c-2956c53c0634"
  ],
  "field_ref": "r-task-result-2",
  "uncertain": true
}
```

채택 Meaning의 실제 Goal 값:

없음. Frame 이후 unresolved/질문 대기에서 종료됨.

실제 downstream 명령:

없음. DIRECT_RESPONSE 또는 adoption 전 종료로 구별해서 본문에 설명했다.

최종 실제 channel receipt. scene 경계는 반올림했으며 Text와 Voice가 다르면 둘 다 보존한다.

| publication | 종류 | Text 제시 | Voice onset/end | 최종 receipt | 실제 전달 문장 |
| --- | --- | ---: | --- | --- | --- |
| `P:1:a8d6a243-d117-4fd7-8d30-052fe01f04c7` | QUESTION | 10.617374s | 10.956860s / 63.048313s | false | Text: 다음 후보 중 어떤 대상이고 무엇을 변경할까요? via-interaction:RECENT_INTERACTION:E1@v1:caabb613aeb7298e2512d1b400d4646e66c9244e95d27a35329a610744323e67; via-interaction:RECENT_INTERACTION:H0001@v1:e5d0f9218bf05034140c148e4f165aa2747d281b60b8e5f7e24460abe8cc98ad; via-interaction:RECENT_INTERACTION:H0002@v1:227cd8dc4bbcc24198fde44e9172283b08962d0c0a22dbfb56376f20fc5434b9; via-interaction:RECENT_INTERACTION:H0003@v1:0657bd0c6e9fa10b662d3ddb405afc8ca2a6fd9a822eb79c5cbc2f0da09d6155; via-interaction:RECENT_INTERACTION:H0004@v1:f03847675ca1202f6d256c9bc2a0b1ec40b89b1fefe600a17169676441ce7e91; via-interaction:RECENT_INTERACTION:H0005@v1:7a85c313121cea94a6d853d0c3d116d66189031061784c008ff3072f0888d88a; via-interaction:RECENT_INTERACTION:H0006@v1:86a5509bf81902a0af46961362cf9dfcdba1d442ab1ab89bfb2296b04ead3602; via-interaction:RECENT_INTERACTION:H0007@v1:3401ed00f5c17168df97b1fb1ed301d160a428f6f3ca00f0a2c95899cd5cc3fa<br>Voice: 다음 후보 중 어떤 대상이고 무엇을 변경할까요? via-interaction:RECENT_INTERACTION:E1@v1:caabb613aeb7298e2512d1b400d4646e66c9244e95d27a35329a610744323e67; via-interaction:RECENT_INTERACTION:H0001@v1:e5d0f9218bf05034140c148e4f165aa2747d281b60b8e5f7e24460abe8cc98ad; via-interaction:RECENT_INTERACTION:H0002@v1:227cd8dc4bbcc24198fde44e9 |

### S2-05 실제 호출과 유한 값

CINTERPRET=2, CRESPONSE=0.

| phase, call ID | Context SHA-256 | 출력 SHA-256 | input/output token |
| --- | --- | --- | ---: |
| FRAME `237fed4b-27de-476d-9d2c-fec09add6446` | [Context](../workers/B1/model-contexts/237fed4b-27de-476d-9d2c-fec09add6446.json) `2ea53e6e03c1dc1b50d5a18a3bcee5ba4f2e5f2fe49c7dcad0d767403c57a085` | [출력](../workers/B1/model-outputs/237fed4b-27de-476d-9d2c-fec09add6446.json) `90db9c01cac51f2afe8a13ad6e4a194e10b78e0031f86fba1e087e1cd5478e13` | 4459/611 |
| B_PARTIAL_SELECT `0bc2ab39-509e-4806-aade-3b6500931923` | [Context](../workers/B1/model-contexts/0bc2ab39-509e-4806-aade-3b6500931923.json) `16f385961dd4ca45f67f19b9c3375405ce177e52ee047807d20d193e096c9191` | [출력](../workers/B1/model-outputs/0bc2ab39-509e-4806-aade-3b6500931923.json) `f52c20d0213c5180d9698bd40f84179be01a8ec5e6e72ac378162d895a144ba8` | 3806/76 |

원장에 남은 실제 Frame와 부분 판단. 조회 전 Goal/value 및 선택한 query를 그대로 보존한다.

`E6295` dp41_b_frame_produced, scene=6.606340s:

```json
{
  "constraints": [
    {
      "op": "KEEP_PROPERTY",
      "property": "exclude_price_evaluation",
      "scope": [
        "G-compare-baseline-new",
        "G-proposal-draft"
      ],
      "value": true
    }
  ],
  "goals": [
    {
      "arguments": {
        "properties": {},
        "source_refs": [
          "r-baseline",
          "r-new-product"
        ]
      },
      "id": "G-compare-baseline-new",
      "instruction": "Compare the previously heard baseline with the newly arrived product. Do not evaluate price.",
      "kind": "NEW_TASK",
      "operation": "COMPARE",
      "target_ref": null
    },
    {
      "arguments": {
        "properties": {},
        "source_refs": [
          "r-new-product",
          "r-quotation"
        ]
      },
      "id": "G-proposal-draft",
      "instruction": "Create a proposal based on the new product and quotation. Exclude price evaluation.",
      "kind": "NEW_TASK",
      "operation": "CREATE_PROPOSAL",
      "target_ref": null
    }
  ],
  "input_ref": "I:de186ac7-8aeb-4078-a060-e124ba4f151e",
  "preservations": [],
  "references": [
    {
      "cardinality": "ONE",
      "id": "r-baseline",
      "mention": "전에 들은 기준 (Previously heard baseline)",
      "selector": {
        "op": "RECENT_INTERACTION",
        "query": "baseline criteria discussed in recent interactions"
      }
    },
    {
      "cardinality": "ONE",
      "id": "r-new-product",
      "mention": "방금 도착한 제품 (Newly arrived product)",
      "selector": {
        "name": "D새비교",
        "op": "EXPLICIT_SOURCE",
        "source_kind": "DOCUMENT"
      }
    },
    {
      "cardinality": "ONE",
      "id": "r-quotation",
      "mention": "견적 (Quotation)",
      "selector": {
        "name": "D새견적",
        "op": "EXPLICIT_SOURCE",
        "source_kind": "DOCUMENT"
      }
    }
  ],
  "relations": [],
  "response": null,
  "unresolved": [],
  "version": "dp41-frame-runtime-1"
}
```

`E6348` dp41_b_partial_returned, scene=7.906144s:

```json
{
  "candidate_ref": null,
  "evidence_refs": [
    "4a66755a-91d7-41bf-a35a-5934acf95825"
  ],
  "field_ref": "r-baseline",
  "uncertain": true
}
```

채택 Meaning의 실제 Goal 값:

없음. Frame 이후 unresolved/질문 대기에서 종료됨.

실제 downstream 명령:

없음. DIRECT_RESPONSE 또는 adoption 전 종료로 구별해서 본문에 설명했다.

최종 실제 channel receipt. scene 경계는 반올림했으며 Text와 Voice가 다르면 둘 다 보존한다.

| publication | 종류 | Text 제시 | Voice onset/end | 최종 receipt | 실제 전달 문장 |
| --- | --- | ---: | --- | --- | --- |
| `P:1:8f2f82d3-75c0-48e3-bdc6-1b2b7c399ca0` | QUESTION | 7.928382s | 8.242386s / 63.045435s | false | Text: 다음 후보 중 어떤 대상이고 무엇을 변경할까요? via-interaction:RECENT_INTERACTION:H0005@v1:7a85c313121cea94a6d853d0c3d116d66189031061784c008ff3072f0888d88a; via-interaction:RECENT_INTERACTION:H0006@v1:86a5509bf81902a0af46961362cf9dfcdba1d442ab1ab89bfb2296b04ead3602; via-interaction:RECENT_INTERACTION:H0007@v1:3401ed00f5c17168df97b1fb1ed301d160a428f6f3ca00f0a2c95899cd5cc3fa; via-interaction:RECENT_INTERACTION:H0008@v1:a31a8176e04e048edd44fbfefb1e8799a9e9cfc5ba20494d64fb9dd4c33d9de0; via-interaction:RECENT_INTERACTION:H0009@v1:0e52f940a0c00b285c2dcbb369286fbebf68d78743449dbca665ab3c82aea797; via-interaction:RECENT_INTERACTION:H0010@v1:0c6746f5d3eacb754626fd8dc4a77e98f8266fe433d64cb6ae6b74b55b53b243; D새비교@v1; D새견적@v1<br>Voice: 다음 후보 중 어떤 대상이고 무엇을 변경할까요? via-interaction:RECENT_INTERACTION:H0005@v1:7a85c313121cea94a6d853d0c3d116d66189031061784c008ff3072f0888d88a; via-interaction:RECENT_INTERACTION:H0006@v1:86a5509bf81902a0af46961362cf9dfcdba1d442ab1ab89bfb2296b04ead3602; via-interaction:RECENT_INTERACTION:H0007@v1:3401ed00f5c17168df97b1fb1ed301d160a428 |

### S3-02 실제 호출과 유한 값

CINTERPRET=1, CRESPONSE=4.

| phase, call ID | Context SHA-256 | 출력 SHA-256 | input/output token |
| --- | --- | --- | ---: |
| FRAME `61f495d6-2783-4d65-80cd-f710446000dc` | [Context](../workers/B1/model-contexts/61f495d6-2783-4d65-80cd-f710446000dc.json) `d6aff17bf90d7a152491b958d53874ad51571f07d383847bae7a88e6b8389cff` | [출력](../workers/B1/model-outputs/61f495d6-2783-4d65-80cd-f710446000dc.json) `3ec5e3c5b40c242b77b490fbd489e7b7822f452711fb5b4524f63ee013249389` | 4500/472 |
| CRESPONSE `8a48eecd-d0b4-4420-ab23-0867f4b84cb4` | [Context](../workers/B1/model-contexts/8a48eecd-d0b4-4420-ab23-0867f4b84cb4.json) `be1877c30157b0bc3aee89eab4625c8753087b7a2c5914cc89269bfae2ac3a53` | [출력](../workers/B1/model-outputs/8a48eecd-d0b4-4420-ab23-0867f4b84cb4.json) `e6c23f917d8f3a5fbf7991f60ce308e2f047acaf58eadd2df3c028f9e46f1c4a` | 2383/40 |
| CRESPONSE `0845f970-c69e-4862-8259-b55632f7d0e8` | [Context](../workers/B1/model-contexts/0845f970-c69e-4862-8259-b55632f7d0e8.json) `1d83d0ba0c9745ae10a2b93629dc15c01828e6359d44dda0077247cd0db9d3fc` | [출력](../workers/B1/model-outputs/0845f970-c69e-4862-8259-b55632f7d0e8.json) `f908cbee8eedc3c9f721e47e54137225a7f1e89802e0aa3443332beb329021fa` | 790/37 |
| CRESPONSE `4881bf47-dd0a-4e34-8963-a67318293c36` | [Context](../workers/B1/model-contexts/4881bf47-dd0a-4e34-8963-a67318293c36.json) `a4bef23ad4dce5ede3425e8c697855083767eb92a639fc7f55fcb2427ad33bf0` | [출력](../workers/B1/model-outputs/4881bf47-dd0a-4e34-8963-a67318293c36.json) `1ac324e8cfcb105bdb2d7cc64cacfce35e5972025359f2bb8b8aecafd4d952f7` | 3097/39 |
| CRESPONSE `2710f360-a689-4ed2-b920-183cd58a2808` | [Context](../workers/B1/model-contexts/2710f360-a689-4ed2-b920-183cd58a2808.json) `e4c3ad61125c2189ad22f5180a1b57faaa6abd97b86e73d3155bc62bea7987a7` | [출력](../workers/B1/model-outputs/2710f360-a689-4ed2-b920-183cd58a2808.json) `1ec13fb703755b682839d34c5d65fd5cba5dd738fb94f50b34d24d969d25b333` | 894/30 |

원장에 남은 실제 Frame와 부분 판단. 조회 전 Goal/value 및 선택한 query를 그대로 보존한다.

`E9261` dp41_b_frame_produced, scene=6.722578s:

```json
{
  "constraints": [],
  "goals": [
    {
      "arguments": {
        "source_refs": [
          "r-d1"
        ]
      },
      "id": "G-summarize",
      "instruction": "D1을 요약합니다.",
      "kind": "NEW_TASK",
      "operation": "SUMMARIZE",
      "target_ref": null
    },
    {
      "arguments": {
        "body": {
          "result_of": "G-summarize"
        },
        "recipient_ref": "r-minsu"
      },
      "id": "G-send-mail",
      "instruction": "요약 결과를 민수에게 메일로 보냅니다.",
      "kind": "NEW_TASK",
      "operation": "SEND_MAIL",
      "target_ref": null
    }
  ],
  "input_ref": "I:7bdceb62-aa89-496e-895e-cb3bd236fc0a",
  "preservations": [],
  "references": [
    {
      "cardinality": "ONE",
      "id": "r-d1",
      "mention": "D1",
      "selector": {
        "name": "D1",
        "op": "EXPLICIT_SOURCE",
        "source_kind": "DOCUMENT"
      }
    },
    {
      "cardinality": "ONE",
      "id": "r-minsu",
      "mention": "민수",
      "selector": {
        "name": "민수",
        "op": "EXPLICIT_SOURCE",
        "source_kind": "CONTACT"
      }
    }
  ],
  "relations": [
    {
      "from_goal": "G-summarize",
      "op": "AFTER",
      "to_goal": "G-send-mail"
    }
  ],
  "response": null,
  "unresolved": [],
  "version": "dp41-frame-runtime-1"
}
```

채택 Meaning의 실제 Goal 값:

```json
{
  "meaning": {
    "constraints": [],
    "evidence_refs": [
      "04ad3d57-4c4c-4cc7-9065-586f3a78ea32",
      "44e295d4-390d-4f09-b353-c1e37c3e1652"
    ],
    "goals": [
      {
        "arguments": {
          "source_refs": [
            "D1@v1"
          ]
        },
        "id": "G-summarize",
        "instruction": "D1을 요약합니다.",
        "kind": "NEW_TASK",
        "operation": "SUMMARIZE",
        "target": null
      },
      {
        "arguments": {
          "body": {
            "kind": "GOAL_RESULT",
            "producer_goal_id": "G-summarize",
            "producer_revision": 1,
            "request_id": "R:86aee18a-a18a-4cda-975d-97ab64338907"
          },
          "recipient_ref": "민수@v1"
        },
        "id": "G-send-mail",
        "instruction": "요약 결과를 민수에게 메일로 보냅니다.",
        "kind": "NEW_TASK",
        "operation": "SEND_MAIL",
        "target": null
      }
    ],
    "preservations": [],
    "relations": [
      {
        "from_goal": "G-summarize",
        "op": "AFTER",
        "to_goal": "G-send-mail"
      }
    ],
    "response": null,
    "unresolved": [],
    "version": "dp41-meaning-runtime-1"
  },
  "request_id": "R:86aee18a-a18a-4cda-975d-97ab64338907"
}
```

실제 downstream 명령:

- `CMD:29a0b490-a8ff-4d03-ad97-7f11fd712d56` / SUMMARIZE / Task `T:c062ac1f-330d-430f-ac8d-2a605ca3acc7` / Execution `X:45b3ab16-072f-41cf-85b2-f856c789e069`
  send=6.735446s, ingress=6.735525s, receipt=6.937132s; payload SHA-256 `6dc6a5336305b25e143fee28fcd4d133e1e0bfdfd94d5687cf3f6a23aa10ac23`.

```json
{
  "constraints": [],
  "goal": {
    "arguments": {
      "source_refs": [
        "D1@v1"
      ]
    },
    "id": "G-summarize",
    "instruction": "D1을 요약합니다.",
    "kind": "NEW_TASK",
    "operation": "SUMMARIZE",
    "target": null
  },
  "preservations": [],
  "relations": [
    {
      "from_goal": "G-summarize",
      "op": "AFTER",
      "to_goal": "G-send-mail"
    }
  ],
  "sources": [
    {
      "kind": "DOCUMENT",
      "metadata": {},
      "permission": {
        "agent": "ALLOWED",
        "model": "ALLOWED",
        "read": "ALLOWED"
      },
      "revision": 1,
      "source_id": "D1",
      "text": "하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다.",
      "title": "D1"
    }
  ]
}
```
- `CMD:3bfee4e6-ada8-4924-ae7b-e108b89fcb97` / SEND_MAIL / Task `T:d150cf57-8258-4cc8-a889-4eceae5edf8e` / Execution `X:9eec5fdd-1a58-4239-bea4-094a1a345a6b`
  send=7.261338s, ingress=7.261407s, receipt=7.463739s; payload SHA-256 `9498cafadb1a3d74070d9ac0ff0b4b333e152cd6e9c4f30f4dad8536191118ef`.

```json
{
  "body_artifact": {
    "artifact_id": "A:54d554e7-2324-4d0e-b3c5-29aecfd12f6f",
    "external_status": "COMPLETED",
    "payload_sha256": "6dc6a5336305b25e143fee28fcd4d133e1e0bfdfd94d5687cf3f6a23aa10ac23",
    "producer_goal_id": "G-summarize",
    "properties": {},
    "request_id": "R:86aee18a-a18a-4cda-975d-97ab64338907",
    "revision": 1,
    "source_refs": [
      "D1@v1"
    ],
    "text": "하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다."
  },
  "constraints": [],
  "goal": {
    "arguments": {
      "body": {
        "kind": "GOAL_RESULT",
        "producer_goal_id": "G-summarize",
        "producer_revision": 1,
        "request_id": "R:86aee18a-a18a-4cda-975d-97ab64338907"
      },
      "recipient_ref": "민수@v1"
    },
    "id": "G-send-mail",
    "instruction": "요약 결과를 민수에게 메일로 보냅니다.",
    "kind": "NEW_TASK",
    "operation": "SEND_MAIL",
    "target": null
  },
  "preservations": [],
  "recipient": "minsu@example.invalid",
  "relations": [
    {
      "from_goal": "G-summarize",
      "op": "AFTER",
      "to_goal": "G-send-mail"
    }
  ],
  "sources": []
}
```

최종 실제 channel receipt. scene 경계는 반올림했으며 Text와 Voice가 다르면 둘 다 보존한다.

| publication | 종류 | Text 제시 | Voice onset/end | 최종 receipt | 실제 전달 문장 |
| --- | --- | ---: | --- | --- | --- |
| `P:1:6edd6292-a45a-46ff-b03f-0667f18a0f8b` | RESPONSE | 8.167206s | 8.431553s / 20.103729s | true | D1 요약 업무가 접수(ACCEPTED)되었습니다. 원문: '하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다.' |
| `P:2:06ba7b2b-1126-4afc-978e-5c7a25158aaa` | RESULT | 8.184654s | 20.376491s / 31.709957s | true | SUMMARIZE 작업이 완료되었습니다. 결과는 '하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다.'입니다. |
| `P:3:719e7bc9-0bdc-4847-9869-76ee404f61c7` | RESPONSE | 8.577093s | 31.982515s / 45.156226s | true | 민수@example.invalid에게 요약 결과(하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다.) 발송 명령이 접수되었습니다. |
| `P:4:df1e64d5-4ec8-4af4-a025-15c5c6d9429f` | RESULT | 16.278058s | 45.442032s / 55.117884s | true | SEND_MAIL 작업이 완료되었습니다. 하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다. |
