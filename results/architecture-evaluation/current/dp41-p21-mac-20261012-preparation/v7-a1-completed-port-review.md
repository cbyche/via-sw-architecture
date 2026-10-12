# V7 A1 완료 worker 실제 포트와 오류 분류

상태: READ_ONLY_ACTUAL_TRACE_REVIEW. A1 COMPLETE의 21개 작은 trial 행과 관련 모델 output, 실제 owner receipt, 명령 payload 및 최종 전달을 읽었다. source/fixture/oracle 수정, 테스트/API 실행, 전수 hash와 메모리/CPU 검사는 하지 않았다. judge 전이므로 C1~C6 결과나 QA 점수, A/B 우열을 확정하지 않는다.

[worker-summary](../dp41-p21-mac-text-v7-20261012/workers/A1/worker-summary.jsonl)는 21case COMPLETE, 실제 provider usage×price $0.125378625를 기록했다. [trials-unjudged](../dp41-p21-mac-text-v7-20261012/workers/A1/trials-unjudged.jsonl)의 83call은 모두 HTTP200 및 알려진 usage/cost이다. 이는 accepted 모집단 전체 비용이나 5반복 결과가 아니다. OBSERVED는 활동 수집 종료이며 성공 판정이 아니다.

## 21개 원시 활동 개요

| 사례 | trial evidence | 원시 상태 | calls | effects | deliveries |
| --- | --- | --- | --- | --- | --- |
| S1-01 | E500 | OBSERVED | 3 | 0 | 1 |
| S1-02 | E1528 | OBSERVED | 5 | 1 | 2 |
| S2-01 | E3337 | EPISODE_TIMEOUT | 6 | 0 | 3 |
| S2-02 | E3995 | OBSERVED | 3 | 0 | 1 |
| S2-03 | E4891 | OBSERVED | 5 | 1 | 2 |
| S2-04 | E5869 | OBSERVED | 3 | 1 | 2 |
| S2-05 | E6513 | OBSERVED | 4 | 0 | 1 |
| S3-01 | E7268 | OBSERVED | 5 | 0 | 2 |
| S3-02 | E8570 | OBSERVED | 5 | 2 | 4 |
| S3-03 | E9277 | OBSERVED | 2 | 0 | 1 |
| S4-01 | E10595 | OBSERVED | 5 | 1 | 3 |
| S4-02 | E11042 | OBSERVED | 3 | 0 | 2 |
| S4-03 | E11523 | OBSERVED | 8 | 0 | 0 |
| S5-01 | E11766 | OBSERVED | 2 | 0 | 2 |
| S5-02 | E12299 | OBSERVED | 2 | 0 | 2 |
| S5-03 | E12727 | OBSERVED | 4 | 0 | 2 |
| S5-04 | E14193 | OBSERVED | 8 | 1 | 8 |
| S6-01 | E14531 | OBSERVED | 2 | 0 | 1 |
| S6-02 | E15138 | OBSERVED | 2 | 0 | 1 |
| S6-03 | E15608 | OBSERVED | 4 | 0 | 1 |
| S6-04 | E16219 | OBSERVED | 2 | 1 | 1 |

## 요청된 세 실패/호출 경로

S4-03 Request `R:cfbaf282-a27c-4271-9011-f5ccb9ad1ad8`은 “메일 업무만 취소해줘.”였다. 8 CINTERPRET output 모두 Goal kind=CANCEL_TASK/target=T메일을 내면서 operation 필드를 빠뜨렸다. 두 번째 output은 잘못된 version2까지 반환했다. 실제 지원하는 operation=CANCEL_TASK를 만들지 못해 E11140~E11417 거절을 반복했고 E11418의 budget exhaustion 및 E11422 REQUEST_FAILED 뒤 명령/전달은 없었다. 이는 취소 기능의 구조 미지원으로 분류하지 않는다. KEEP_PROPERTY STATUS와 Task ID를 scope에 둔 다른 잘못된 출력도 남아 있지만 최초 거절은 operation 누락이며 코드를 고쳐 올바른 취소로 해석하지 않는다.

S5-04 Request `R:79574cfb-462d-4a43-9bba-bb21d80f4423`의 8call은 CINTERPRET 1과 CRESPONSE 7이다. 8번의 의미 재시도나 예산 실패가 아니다. 4개 동시 사건의 질문/결과와 후보가 만든 두 Goal 및 완료 응답이 각각 실제 응답 생성 비용을 발생시켰다. 8 delivery record에는 LOCAL_STOP 1, actual 질문2, external result2, 접수/설명2, 후보 작업 결과1이 있다. delivery record 수를 모두 성공한 응답 관측 수로 쓰지 않는다.

모델은 UPDATE_TASK+CREATE_TABLE/target=T보고서, properties={}와 DIRECT_RESPONSE+EXPLAIN을 택하고 T메일 NO_NEW_COMMAND를 별도로 보존했다. 실제 CREATE_TABLE command `CMD:624b6a12-5396-4f57-bb7c-2301fc56646b`는 새 `T:bdcfebff-a4be-4b78-b0da-26783332703b`에 actual D1 원문 “2026년 3분기 매출 150억원”과 Goal을 전달했다. 결과 artifact `A:5f35462d-0743-49fc-a86a-856cc75eff46`도 이 실제 원문 및 빈 properties를 유지한다. 같은 Task의 FORMAT 변경 또는 실제 표 완성으로 평가자가 고치지 않는다. SET_PROPERTY라는 지원 경로와 이번 CREATE_TABLE 선택의 실제 효과를 분리하여 이후 판정한다.

S6-01 Request `R:c3d8310f-3205-4b15-94c0-d362f4a5af66`은 preflight와 다른 실행이다. 모델이 실제 현재 질문이 아닌 `Q-clarify-report-target`을 CLARIFY.scope.question_refs에 지정했고 E14289는 no_answer/QUESTION_NOT_CURRENT였다. 실제 질문 제시나 예정 답변은 발생하지 않았다. 후보는 NO_CLARIFICATION_ANSWER Meaning을 내고 “명확한 답변이 없어 추가 확인이 필요합니다.”를 전달했으며 effects0이다. 존재하지 않는 질문 locator를 실제 현재 질문으로 보정하지 않았다.

- S4-03 call `d14716d4-e1dc-4f6f-8ffb-30302b402cfa`: [Context](../dp41-p21-mac-text-v7-20261012/workers/A1/model-contexts/d14716d4-e1dc-4f6f-8ffb-30302b402cfa.json), digest `4c59c6b0c027bc370fa225623c5d5e7c743bc2b968d31a8b51ee802cb768af72`; [output](../dp41-p21-mac-text-v7-20261012/workers/A1/model-outputs/d14716d4-e1dc-4f6f-8ffb-30302b402cfa.json), digest `40d4234b5cbc5e62b502902c89ce895a844772c732d8a9e6db900fa9eaddc034`.
- S4-03 call `398c6986-d713-4f52-99e6-40b75837b84b`: [Context](../dp41-p21-mac-text-v7-20261012/workers/A1/model-contexts/398c6986-d713-4f52-99e6-40b75837b84b.json), digest `220b19d793ef74752329656bf842fcbfea30561a57721543ad664eaf4146f9d2`; [output](../dp41-p21-mac-text-v7-20261012/workers/A1/model-outputs/398c6986-d713-4f52-99e6-40b75837b84b.json), digest `6f1b64a08af4f796846d0f7f8cb83e3898743d4cf5dc729c8c92c1948e32bd57`.
- S5-04 call `6b18619a-0abe-4124-a1f7-e5b692e3c6d4`: [Context](../dp41-p21-mac-text-v7-20261012/workers/A1/model-contexts/6b18619a-0abe-4124-a1f7-e5b692e3c6d4.json), digest `f151a22eac497f75fd029c86966fbc351efb7ef6a68a9098f34d1b1a9bdbd813`; [output](../dp41-p21-mac-text-v7-20261012/workers/A1/model-outputs/6b18619a-0abe-4124-a1f7-e5b692e3c6d4.json), digest `dc441c906a913a811a3f70646554bf9eafaf2be68d6886df39188e089d73d33b`.
- S6-01 call `81e98613-eb51-4cd6-a965-615cefd4cd24`: [Context](../dp41-p21-mac-text-v7-20261012/workers/A1/model-contexts/81e98613-eb51-4cd6-a965-615cefd4cd24.json), digest `a5128eca9e4d3489efc9943bb0d222cb53d7b3969c9e89a45c84c66c244c4716`; [output](../dp41-p21-mac-text-v7-20261012/workers/A1/model-outputs/81e98613-eb51-4cd6-a965-615cefd4cd24.json), digest `bacb362bd62910794463afaac55a7dd54448bf60a1f30e3db225cf5e5d101f4c`.

## 실제 실행된 공통 경계와 미발생 범위

| 경계 | 실제 A1 evidence | 해석 경계 |
| --- | --- | --- |
| 늦은 실제 모델 반환 | S4-01 R옛 call=d2f74741-4600-4321-a2c7-8288a5a12a68 실제 완료 뒤 E9363 MEANING_DISCARDED/SUPERSEDED_INPUT | old_job_created=true, late_return_exercised=true이며 오래된 Meaning으로 명령하지 않음. 신규 모델 의미의 적합성은 별도 |
| 늦은 출력 generation | S4-02 old-output-ready E10654→E10655 STALE_GENERATION, LOCAL_STOP | 오래된 출력과 현재 response를 구분. 실제 Task 취소 확인으로 확장하지 않음 |
| 질문 철회 | S5-01 E11543 Q보고서 WITHDRAWN, 해당 delivery voice_text 빈 값/receipt=false/discard QUESTION_WITHDRAWN_OR_STALE; Q메일만 실제 제시 | 초기 prepared question과 actual presented question을 혼동하지 않음 |
| 중복/Task revision | S5-02 Email1 중복 E11786 무시, Ereport1 E11791 STALE_REVISION, 현재 Ereport2와 Email1 전달 | rejected/duplicate event가 있다는 이유만으로 실패나 성공을 결정하지 않음 |
| 현재 producer 결과 | S3-02 E7426 summary artifact→SEND_MAIL body의 같은 artifact ID/request/producer/revision | 옛 Task 결과와 current symbolic result를 구분. 이미 완료 반영된 RESULT의 ALREADY_CURRENT_COMPLETED 및 duplicate는 별도 원장 |
| Voice OFF | S5-03 external result delivery voice_on=false/voice_text빈 값/Text receipt=true, 이후 Voice ON summary 전달 | Voice OFF의 Text 완료를 빈 음성 실패로 오인하지 않음 |
| 원천 currentness | S3-03 CALENDAR_WINDOW는 Cal16@v2 반환; S2-05 fresh source 사건 후 D새비교/D새견적 원천 실제 조회 | current source를 얻은 사실과 이후 Goal의 조건/범위 처리는 별도 |
| 원천 접근 거절 | S6-03 E15231 DENIED/items[]이며 source body 미제공 | 접근 거절 안내의 의미/전달과 무단 접근 없음은 별도 조건 |
| 미제시/철회 동의 | S6-02 E14539 Q1v1 철회, 모델이 denied 민수 READ_FIELD NEW_TASK를 내어 E14645 policy guard/E14646 GOAL_FAILED, effect0 | 동의 절차를 완료한 증거가 아님. 읽기/제공/SEND 승인을 무료로 부여하지 않음 |
| 현재 질문 답변 route | S6-04 실제 모델은 ANSWER_QUESTION 대신 SET_PROPERTY PERIOD=2026-H1를 택함, Q기간v7 상태는 VALID로 남음 | Task patch와 Question answer receipt를 합쳐 성공으로 만들지 않음 |

Owner 조회는 EXPLICIT_SOURCE6, SOURCE_AT_REV2, INPUT_SELECTION2, TASK_RESULT12, CALENDAR_WINDOW2였다. A1에서 PAST_DELIVERED opaque view와 CURRENT_QUESTION의 actual OWNER_READ 경로는 발생하지 않았다. 구현이 그 경로를 지원한다는 회귀 자료와 이번 실제 branch coverage를 구분한다. 미발생 경로를 실행 성공 또는 0비용 관측으로 만들지 않는다.

S3-02에서 actual SUMMARIZE payload는 D1 원문을 받았고 SEND_MAIL은 그 actual summary artifact와 actual recipient=minsu@example.invalid를 받았다. S2-04 CREATE_PROPOSAL은 실제 D2@v2와 D3@v1만 받고 그 원문을 합친 결과를 반환했다. 누락한 heard criteria를 stub이 정답 관계로 채우지 않았다. 이들은 원시 payload 소비의 예시이며 모든 oracle 조건을 만족했다는 판정이 아니다.

## 완료 10workers와 judge 이후에 재사용할 원장 체크리스트

1. accepted worker당 P21 21행, 동일 반복의 A/B 연결, actual Request/Input/generation/Task/Execution/revision을 먼저 고정한다. partial/INVALID와 accepted를 섞지 않는다.
2. 모든 model call을 실제 Context/output/role/purpose/usage/retry/discard에 연결한다. 여러 CRESPONSE, 늦은 실제 R옛 호출과 rejected schema 비용도 포함한다. known usage와 unknown 비용을 분리한다.
3. 원천 조회 status/items/ref/revision/read_id와 actual command sources/body/recipient/properties를 연결한다. 모델의 query/limit 선택, common retrieval 한계와 실제 지원 연산 누락을 분리한다.
4. 골든 oracle는 evaluator 판정에만 사용한다. actual command가 잘못된 Task, 빈 property 또는 source를 보냈으면 그 payload와 결과를 유지한다. downstream final artifact를 예상 정답으로 교체하지 않는다.
5. C5 상태/effects와 C6 actual channel receipt를 각각 본다. REQUEST_FAILED와 무관한 성공 state, late/duplicate rejection과 정상 current result가 같은 사례에 함께 있을 수 있다.
6. TEXT_PRESENTED snapshot의 빈 voice_text를 이후 최종 receipt와 혼동하지 않는다. Voice ON은 실제 재생된 문자와 중단 범위, Voice OFF는 실제 Text publication을 본다. LOCAL_STOP record를 semantic answer로 세지 않는다.
7. judge의 핵심 사실 quote는 실제 최종 채널 text의 정확한 부분문자열이어야 한다. 오답/미완료/미전달이면 frozen QA02/03 cap을 유지한다. 잘못된 빠른 오류 안내를 정상 completion으로 선택하지 않는다.
8. 반응과 사이클의 고정 population ID를 사용하고 후속 입력, 발화 종료, 전사 완료, manual pause와 자동 episode timeout을 구별한다. episode 종료 시각을 임의의 valid answer endpoint로 쓰지 않는다.
9. critical 판단은 actual effect/permission/question currentness/duplicate 원장으로 확인한다. 진짜 모델 실패를 collector bug로 INVALID 전환하지 않는다. 공통 지원 계약과 actual wire가 충돌한 증거가 있을 때만 별도로 보고한다.

이번 읽기에서 새로 확정된 공통 포트 P1/P2는 없다. 실제 Task destination과 모델 Goal target이 다른 route는 literal 사실로 보존했으며, 아직 judge에 의한 조건별 의미 판정이나 전체 A/B 비교를 하지 않았다.
