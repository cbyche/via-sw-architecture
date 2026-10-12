# V7 B1 완료 worker의 실제 Frame와 코드 해결 경로

상태: READ_ONLY_ACTUAL_TRACE_REVIEW. B1 COMPLETE의 21개 작은 trial 행, 선택한 실제 Context/output, owner receipt, 명령 payload와 최종 전달을 읽었다. source/fixture/oracle/freeze 변경, 테스트/API 실행, 전수 hash와 메모리/CPU 검사는 하지 않았다. judge 전이므로 C1~C6, QA 점수 및 후보 우열을 확정하지 않는다.

[trials-unjudged](../dp41-p21-mac-text-v7-20261012/workers/B1/trials-unjudged.jsonl)에 실제 84call이 있고 모두 HTTP200 및 알려진 usage를 기록한다. 아래 Frame 수는 `dp41_b_frame_produced` 이벤트 수이며, 이후 계약 거절된 Frame도 포함한다. deliveries는 LOCAL_STOP, 중단된 질문, 접수 및 완료 전달의 원시 기록 수다. 이 수들을 성공 관측 수로 쓰지 않는다. OBSERVED는 수집 종료이며 성공 판정이 아니다.

## 전체 원시 활동과 실제 모델 경로

| 사례 | trial evidence | 원시 상태 | calls | Frame | partial SELECT | CLARIFY | effects | deliveries |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| S1-01 | E397 | OBSERVED | 2 | 1 | 0 | 0 | 0 | 1 |
| S1-02 | E1200 | OBSERVED | 3 | 1 | 0 | 0 | 1 | 2 |
| S2-01 | E2622 | OBSERVED | 8 | 3 | 5 | 2 | 0 | 2 |
| S2-02 | E3885 | OBSERVED | 3 | 1 | 0 | 0 | 1 | 2 |
| S2-03 | E4792 | OBSERVED | 4 | 2 | 0 | 0 | 1 | 2 |
| S2-04 | E5701 | OBSERVED | 3 | 2 | 0 | 1 | 0 | 2 |
| S2-05 | E6421 | OBSERVED | 8 | 8 | 0 | 0 | 0 | 0 |
| S3-01 | E7050 | OBSERVED | 3 | 1 | 0 | 0 | 0 | 2 |
| S3-02 | E8474 | OBSERVED | 5 | 1 | 0 | 0 | 2 | 4 |
| S3-03 | E9384 | OBSERVED | 3 | 2 | 0 | 1 | 0 | 2 |
| S4-01 | E10235 | OBSERVED | 5 | 3 | 0 | 0 | 1 | 2 |
| S4-02 | E11659 | OBSERVED | 5 | 4 | 0 | 2 | 0 | 4 |
| S4-03 | E12020 | OBSERVED | 2 | 1 | 0 | 0 | 1 | 1 |
| S5-01 | E12273 | OBSERVED | 2 | 0 | 0 | 0 | 0 | 2 |
| S5-02 | E12757 | OBSERVED | 2 | 0 | 0 | 0 | 0 | 2 |
| S5-03 | E13156 | OBSERVED | 3 | 1 | 0 | 0 | 0 | 2 |
| S5-04 | E14506 | OBSERVED | 8 | 1 | 0 | 0 | 1 | 8 |
| S6-01 | E16157 | OBSERVED | 4 | 3 | 0 | 1 | 0 | 2 |
| S6-02 | E17313 | OBSERVED | 6 | 5 | 0 | 1 | 0 | 2 |
| S6-03 | E18080 | OBSERVED | 3 | 2 | 0 | 1 | 0 | 2 |
| S6-04 | E18651 | OBSERVED | 2 | 1 | 0 | 0 | 0 | 1 |

partial SELECT 호출은 실제 role=CINTERPRET/purpose=FOREGROUND로 기록된다. 역할별 Context와 `dp41_b_partial_returned` 이벤트로 Frame 생성과 구별한다. 존재하지 않는 별도 purpose를 원장에 있었다고 설명하지 않는다. S5-01/02는 외부 질문/결과 경로의 CRESPONSE만 발생했으며 의미 Frame 실행 관측으로 세지 않는다.

## A1과 다른 실제 해결 구조

S1-01 Request `R:7da6bbfb-2ac8-4c34-a324-74d37b348d4b`는 Input `I:81e155ef-f943-4e97-a61b-5805cd3b5ffe`의 D1 마감 조회다. E116 Frame은 `READ_FIELD/DEADLINE`, `source_refs=[r-document-d1]` 및 EXPLICIT_SOURCE D1/ONE을 생성했다. E117 UNRESOLVED→E118 READ_PENDING→E120 owner read `10622db3-edc0-460d-8777-bcbe6e99af00`→E123 BOUND `D1@v1`로 연결된다. 코드가 실제 ref를 Meaning에 넣은 뒤 로컬 READ_FIELD가 원문 사실을 얻고 CRESPONSE가 “D1의 제출 마감일은 2026년 10월 15일입니다.”를 실제 Voice/Text에 전달했다. read 뒤 전체 Meaning을 모델이 다시 작성하는 호출은 없었다. [A1 검토](v7-a1-completed-port-review.md)의 S1-01에서는 모델 READ 후 모델이 다시 PROPOSE를 선택했으나 goals=[]로 종료했다. 이 단일 실행 차이를 일반적인 정확성 우월성으로 확대하지 않는다.

S2-01 Request `R:b37c0770-fadf-4ae4-933e-297671f51595`에서는 코드가 두 INPUT_SELECTION ref에 실제 A/B/C 원천과 pointer observation을 조회했다. E1357/E1388 partial SELECT는 각각 `candidate_ref=null, uncertain=true`였고 E1358/E1389의 상태는 AMBIGUOUS였다. 코드의 질문 E1390/E1965에 실제 답변 E1834/E2409가 돌아와 새 finite Frame을 만들었지만 추가 partial도 미확정이었다. 실제 3 Frames+5 partials=8 의미 호출 예산을 소모했고 E2516 budget exhaustion/E2520 REQUEST_FAILED, effects0이다. B를 무조건 한 번 호출하는 구조로 구현했다는 설명과 다르다. 두 원천 COMPARE는 표현 가능한 연산이며 이번 미확정 선택을 미지원 연산으로 분류하지 않는다. 두 ref의 A/B/C 후보가 질문에 중복 표시되는 구현 선택과 긴 재질문 비용은 B 구조의 필수 비용이라고 주장하지 않는다.

S3-02 Request `R:686874bb-ff97-427e-babd-e6d15528c079`의 E7193 Frame은 SUMMARIZE와 SEND_MAIL, AFTER 및 `body.result_of=G-summarize`를 표현했다. Owner E7197/E7203가 실제 D1/민수를 반환하고 E7206/E7207가 BOUND였다. 코드의 Meaning은 symbolic GOAL_RESULT에 현재 Request/producer/revision을 넣었다. SUMMARIZE command `CMD:47dc4f2e-dbc6-4ac5-a7c2-ae9b8e05c526`은 실제 D1 원문을 받았다. SEND command `CMD:f7a5221b-8bf9-4353-8053-00a33758b297`은 실제 생성 artifact `A:44ac8b36-4f17-4e6f-afc5-85db047277b2` 및 `minsu@example.invalid`를 받았다. body payload digest는 `2ceb244e74afed5206e5c384d4d590220d826eaddef92c37b7b545d3f1b42cb1`이다. 접수/요약 결과/메일 접수/메일 결과의 네 실제 전달이 남아 있다. A1도 현재 생성 artifact를 사용했으며, 차이는 전체 binding의 모델 생성과 unbound Frame의 코드 binding에 있다. 옛 Task 결과를 무료로 새 요약으로 대체한 경로가 아니다.

S4-03 Request `R:a54f4401-45a5-4465-a391-a4779177095d`에서 E11759 Frame은 CANCEL_TASK operation을 명시하고 KNOWN_REF `T메일`을 `r-task-mail`로 선언했다. E11760→E11761 HOST_METADATA binding은 실제 제공된 현재 Task metadata를 사용한다. command `CMD:8829e19d-4911-4ab1-a39e-d920179febcd`가 T메일/X메일에 보내졌고 실제 상태는 CANCEL_PENDING, 전달은 “안내문 발송 취소 명령이 접수되었습니다.”였다. A1의 operation 누락 재시도와 달리 이번 B1은 지원된 취소 wire를 만들었다. 접수와 외부 취소 완료는 구별하며 실제 취소 완료를 주장하지 않는다.

## 잘못된 선택, 형식 오류와 지원 범위

S2-02 Request `R:959d7583-84f4-4574-a8bb-370bd48e169e`의 모델은 “아까 들은 기준”에 `PAST_DELIVERED source_ref=B01, query=background criteria documents, cardinality=MANY`를 선택했다. E2892/read `8c874347-c12c-460e-964a-239fd477060e`는 H0001/H0021 등 10개 실제 배경 전달 view를 반환했다. E2898/T비교와 E2904/T견적은 actual D2@v2/D3@v1을 반환했다. E2907~E2909 binding 이후 CREATE_PROPOSAL command `CMD:00210675-ac17-43ea-81d4-cca3a5c302e2`는 이 12개 actual sources를 받았다. H0001의 payload는 actual voice “배경 기록 H0001: B01@v1의 값은 101이다.”, provenance ACTUAL_VOICE, publication/revision/read_id/record hash 및 역할별 permission을 보존한다. 실제 원천은 E1 기준으로 고쳐지지 않았다. 결과와 최종 전달에도 값101 및 실제 비교/견적 사실이 남았다. 잘못된 원천 선택, 공통 lexical bounded retrieval의 일반 한계와 실제 heard view materialization을 분리한다.

위 경로는 finite source_refs에 actual heard view를 바인딩해 Agent에 넘길 수 있음을 보여준다. 원래 기준을 선택했다거나 그 기준을 적용한 올바른 제안서가 만들어졌다는 증거는 아니다. B_PARTIAL_SELECT는 참조 선택만 갱신하고 일반적인 read 후 새 Goal/조건값 추출 기능을 제공하지 않는다는 선언된 한계도 유지한다. 이번 S2-02는 기준과 결과를 원천으로 위임하는 표현을 사용했으므로 그 한계만으로 이번 배경 원천 선택을 설명하지 않는다.

S2-05 Request `R:12a8fb12-1ffd-46f2-8ebb-6d009a4d9135`의 8 Frames는 proposal Goal의 `target_ref=G-compare`를 반복했다. G-compare는 Goal ID이며 선언된 reference ID가 아니다. E5878~E6315의 계약 거절 뒤 E6316/E6320 budget/REQUEST_FAILED로 종료되어 owner read, binding, 명령 및 전달이 발생하지 않았다. Goal 결과 의존성은 USE_RESULT/AFTER 및 body.result_of로 지원하며 Task target과 다르다. 이번 실패를 원천 검색 coverage 부족이나 finite 계약의 제안서 연산 미지원으로 재분류하지 않는다.

S6-04 Request `R:a9690d08-988b-40d8-ae93-f59130cf845b`의 actual Context에는 T보고서/X보고서와 Q기간v7/PRESENTED/Task=T보고서/제시값2026-H1이 있었다. E18208 Frame은 r-q-period와 r-task-report를 모두 선언했지만 ANSWER Goal만 `target_ref=null`이었다. E18211/E18212는 실제 제공된 Q기간/T보고서를 HOST_METADATA로 바인딩했고 Meaning도 그 null target을 그대로 유지했다. E18220의 거절 Goal은 `G-answer-Q기간/ANSWER_QUESTION`이며 “Existing Task target unavailable”이다. 이후 UPDATE Goal은 AFTER 의존성 때문에 실행되지 않았다. [동결 COMMON prompt](../dp41-p21-mac-text-v7-20261012/frozen-source/prototypes/candidates/dp41_45_poc/dp41/prompts.py)는 기존 Task target과 new/direct의 null을 구별하고, [runtime._dispatch](../dp41-p21-mac-text-v7-20261012/frozen-source/prototypes/candidates/dp41_45_poc/shared/runtime.py)는 existing operation의 실제 Task를 검증한다. 이번 target 누락은 후보 생성 오류로 기록한다. consent.read=true도 기간 질문의 scope와 다르지만 코드가 해당 검사에 도달하지 않았으므로 최초 원인으로 앞당기지 않는다. 후보의 null을 질문 lineage로 자동 수정하지 않았다.

S2-04/S3-03/S6-01/02/03의 일부 clarification 후 Frame에는 input_ref/preservation/frame reference 계약 거절이 실제로 남아 있다. 오류 안내가 전달된 경우에도 정상 Goal completion으로 합치지 않는다. 기존 좁은 [현재 Input 대조](v7-initial-actual-route-review.md)의 규칙처럼 actual Context에 제공된 현재 Input과 model output을 대조하며, 답변 read item ID를 current root Input ID로 잘못 택한 결과를 collector가 고치지 않는다.

## 실제 공통 경계와 payload 소비

| 경계 | B1 실제 evidence | 해석 경계 |
| --- | --- | --- |
| 늦은 실제 모델 반환 | S4-01 R옛 call=e6d0456d-dfc4-41d0-a3b2-6b588e42525f 실제 완료 후 E9485 SUPERSEDED_INPUT | old_job_created/late_return_exercised 모두 true. 오래된 Meaning으로 명령하지 않음 |
| 오래된 출력 | S4-02 E10295 STALE_GENERATION 및 LOCAL_STOP | 이후 두 질문/예산 오류 안내와 구별. 실제 Task 취소 아님 |
| 철회 질문 | S5-01 E12040 Q보고서 WITHDRAWN, 첫 delivery voice_text='', receipt=false/QUESTION_WITHDRAWN_OR_STALE | Q메일만 실제 Voice 제시. 준비된 질문을 실제 제시로 세지 않음 |
| 중복/옛 Task revision | S5-02 E12293 DUPLICATE_IGNORED 및 E12298 STALE_REVISION | 현재 report와 mail 결과 두 전달을 각각 보존 |
| Voice OFF | S5-03 첫 RESULT Voice OFF/voice_text=''/receipt=true, 이후 Voice ON response | actual Text receipt와 빈 음성을 구별 |
| 접근 거절 | S6-03 E17432/read=4d9b90d4-3970-42ea-bf56-2cc7fdedf86a DENIED/items[] | denied source body를 partial/Agent에 제공한 증거 없음. 거절 질문과 input_ref 실패는 별도 |
| 동의 currentness | S6-02 E16165 Q1v1 철회, 이후 generic 질문과 계약 오류 안내, effects0 | Q1v2 실제 동의 완료 또는 source 제공 성공으로 주장하지 않음 |
| 동시 사건과 실제 Task patch | S5-04 SET_PROPERTY command CMD:71b259c4-efa8-4de6-943a-e5261cb82a35 | T보고서 FORMAT=TABLE 및 YEAR/QUARTER/원문 보존, T메일 NO_NEW_COMMAND. 두 외부 질문과 두 외부 결과 등 원시8delivery는 독립 기록 |

S5-04에서 A1의 CREATE_TABLE/new Task route와 B1의 SET_PROPERTY/existing Task route는 모델이 선택한 실제 operation 차이다. 공통 stub는 A1의 빈 properties를 TABLE로 고치지 않았고, B1에서는 실제 SET_PROPERTY payload의 TABLE 및 기존 YEAR/QUARTER를 사용했다. payload가 올바른 표 문서 품질을 만들어냈다는 평가와 이 실제 property patch 관측은 구별한다.

## 재현용 실제 모델 artifact 연결

아래 digest는 model-call 원장의 저장 artifact SHA이며 이번 timing 중 다시 계산하지 않았다. Context에는 실제 wire messages/model/parameters가 보존되어 있다. token/usage/retry는 같은 call의 원장과 연결한다.

| 사례/역할 | actual call | Context artifact digest | output artifact digest |
| --- | --- | --- | --- |
| S1-01 Frame | 11728b32-6818-42eb-aaeb-667466ac6384 | f55c19014d49ca866f9959ad5ac6da6287d573e05d3b506d21e1a414b67cedac | fb073df1ce343820f77677749acdc69aa935dd80457efbd3b68eeebded1a0b14 |
| S2-01 Frame | 2afc06cf-4e57-422f-9402-bc7fe067e464 | 2929c4c33dfba25f43e5cf3dfcc993b7ad3838f0a2a19b7ab834dcad1660ab57 | 45da5c590abd1dfaa68c32ca5744b616781f958f10d00f306bba8aff3b282319 |
| S2-01 partial | c4cd12d9-1473-4aac-9203-ac47e9d302b7 | 2cc5512fa46962bbf37d6293359f562178da3edc533bf19b7d8a244640bcb8a9 | 19d67524cac28997cfff5d24612ff698b5b40374da3d4273136eb694e9f79aeb |
| S2-02 Frame | c29fe6b3-ea73-413f-a1e5-9a0468d8211f | a1f1f3e59995dc9fdd7e193440823288b436a6f147f9f46b6b50bc3709099029 | 3e7ae32253dc5044583ad56e4decad6d8b15bcf582403a55181ebd1e2b3cfef4 |
| S2-05 Frame | 60e26432-4ca6-4819-a2e9-d70c5f9fdd29 | f305bc7df6155f0115d76b8ba0b3fb915d42aed4b5e1941d2f154ab5d00b53e8 | 87d825b1ce652e062c4a6629925f867c53371eb9fb3ec6afd3ad7d134efdce27 |
| S3-02 Frame | 91b5b94a-e14e-4599-83b5-b6ceb25346a4 | 3574f5061ec3dfbd9c8c7176f7d5a919cc11a794a59935ffea464bdff1516e97 | 14525d57c31c13f8f0ec12ce744328322d37f2523fa7a085099cd75e429a16bd |
| S4-03 Frame | c59ee1e9-bfa3-4846-9fe5-e1f264ec5fc7 | e0e8969a371b4f10320c4d15730bf25c7183a19fb72215c40a1bc1fa7835be1e | 8a0231981a66a2e0116c7eb97b7de84c4f0166dbea0fb466990b3908cc1034c3 |
| S6-04 Frame | d4b5aaff-3cb8-49dd-95ca-98d1d8bd24fd | ccfb8923072e360fff2bc04842252632f62694985deb90c5ddab46e2f78e3974 | 5c989f84da50dbb78fe4eced1a0df442379b8ebd385607bbcb6ce2bc99789f1e |

파일은 worker의 [model-contexts](../dp41-p21-mac-text-v7-20261012/workers/B1/model-contexts) 및 [model-outputs](../dp41-p21-mac-text-v7-20261012/workers/B1/model-outputs)에 `{actual call}.json`으로 존재한다. R옛 call의 Context artifact digest는 `106c5779b7e29c2dbab2d78f73c4294537c860ddcda48704612130769a7f3bb8`, output artifact digest는 `b98721ef9e1ef74120a02f7e64f355f17af21b5f0361f9d9a3af901146ad95f6`이다.

후속 10worker/judge audit에는 [A1 원장 체크리스트](v7-a1-completed-port-review.md)의 accepted/INVALID 분리, actual channel receipt, first-core quote, 질문 currentness, actual command/effect와 비용 연결 규칙을 재사용한다. B에는 Frame 생성/거절, field state, partial uncertainty, binding provenance 및 symbolic artifact를 추가한다. 모델의 잘못된 query/operation/target/input_ref와 선언한 지원 한계를 구별하고, 실제 후보 failure를 그대로 보존한다. 이번 완료 B1 읽기에서 새로 확정된 공통 포트 P1/P2는 없다.
