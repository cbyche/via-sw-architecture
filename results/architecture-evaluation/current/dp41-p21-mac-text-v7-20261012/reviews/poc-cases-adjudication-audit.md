# DP41 V7 A안 실제 판정 독립 audit

상태: COMPLETE_A105_READONLY_AUDIT_PENDING_ROOT_LEDGER. 범위는 A1~A5의 P21 105개 실제 판정이다. B안 105개는 별도 검토자가 맡는다. 추가 모델/API 호출은 0이고 후보, frozen source, 정답, 원시 trial, 원래 adjudication은 수정하지 않았다.

## 기준

현재 ASR-QA-v6의 각 C1~C6 mandatory fact, 금지 효과, reaction/cycle endpoint를 실제 adopted Meaning, 명령 payload, owner read receipt, 최종 Voice/Text publication과 대조한다. 초기 TEXT_PRESENTED snapshot의 빈 voice_text를 최종 delivered_receipt 이후의 actual Voice와 혼동하지 않는다. 미채택 proposed goal, 과거 Task 목록, 내부 오류, 거절된 명령은 각각 실제 채택 목표, 신규 외부 효과, 사용자 응답, 실행 명령과 구분한다. C6의 tail 실패는 별도 checkpoint QA02 endpoint를 자동으로 무효화하지 않는다.

정정 제안은 [poc-cases-adjudication-proposals.jsonl](poc-cases-adjudication-proposals.jsonl)에 원래 canonical row hash, tuple, evidence ID, 변경 경로, 원래 값과 제안 값을 보존한다. status=PROPOSED_NOT_APPLIED이며 root가 근거를 검토한 뒤 별도 override ledger에 수용 여부를 기록한다. WITHDRAWN_AFTER_CANONICAL_SCOPE_REVIEW 행 및 그 행이 철회한 최초 제안은 적용 대상이 아니다.

## 현재 핵심 검토

| Tuple | 실제 근거 | 독립 검토 |
| --- | --- | --- |
| A1 S1-01 | D1 actual read 뒤 goals=[]와 response=null이 채택됐고 full Voice는 NO_ACTION 결론이다 | C1을 미채택 조회 의도만으로 PASS시키지 않는다. 해당 substantive NO_ACTION은 잘못된 최초 core 응답이며 valid endpoint는 없다. C2~C5의 독립 source/scope/NoTask/효과 사실은 유지한다. |
| A1 S2-01 | 실제 effects=[]이고 T보호 신규 명령이 없다. P3는 기존 Task 목록을 말한 잘못된 응답이다 | UNRELATED_TASK_EFFECT는 실제 효과가 없어 삭제 제안한다. 잘못된 응답과 나머지 실패는 유지한다. |
| A1 S2-02 | goals=[]/NOCLARIFICATIONANSWER, 새 Proposal Task/effect 없음, 마지막 full Voice NO_ACTION | 원래 0/6과 invalid completion 근거를 확인했다. 준비된 E1/완료된 원본 Task만으로 새로운 요구를 통과시키지 않는다. |
| A1 S2-04 | 새 Task 접수 publication은 사실이나 payload에 실제 E1 heard criteria/가격 제외가 없다 | 좁은 실제 접수 사실은 기록하되 지정 근거를 만족하지 못한 QA02는 valid=null/wrong_first=true다. C6의 새 접수/현재 두 번째 결과 참조와 C3의 실제 새 Task 관계는 별도로 통과 제안한다. 가격 문자열 자체를 금지하지 않는다. |
| A1 S4-01 | 기존 T보고서 UPDATE 대신 새 CREATE_TABLE Task 효과가 발생했다 | 초기 critical 삭제안은 철회했다. 원문 잘못된 질문/Task 연결은 broad Request→Task association도 포함하므로 새 잘못된 Task 효과를 숨기지 않는다. 질문이 없었다는 좁은 근거로 실제 잘못된 Task association을 지우지 않는다. |
| A1 S5-01 | actual question은 수신자 선택지만 말하며 어떤 업무=메일인지 식별하지 않는다 | C6의 해당 실패 사실을 그대로 적용하면 reaction에도 동일한 식별 사실이 필요하다. valid null/wrong_first true를 조건부 제안한다. 표시 전에 철회된 다른 질문에는 철회 공지 의무를 새로 만들지 않는다. |
| A2 S1-01 | actual D1 read 뒤 ANSWER_QUESTION에 현재 Request ID를 question_ref로 썼고 adoption이 거절됐다 | C1 temporary proposed intent PASS를 actual adopted handling 기준 FAIL로 제안한다. 나머지 조건을 일괄 실패로 바꾸지 않는다. |
| A2 S4-03 | T메일 CANCEL 1회뿐이며 HT191~HT200은 retained common history다 | 과거 목록을 신규 UNRELATED_TASK_EFFECT로 오인한 critical만 삭제 제안한다. 보고서 계속 사실을 말하지 않은 C6 실패는 유지한다. |
| A2 S6-01 | 답변 뒤 기존 T3분기 FORMAT=TABLE SET_PROPERTY 1회, 4분기 유지 | FORBID_ACTION SEND는 SEND_MAIL이고 UPDATE 명령 wire 전송 금지가 아니다. C5 PASS를 제안한다. C6 checkpoint의 4분기 유지 설명 누락은 실패로 유지한다. 실제 TABLE/2026/Q3/SalesQ3@v1/revision 2 결과와 full final delivery는 확인돼 cycle valid를 제안한다. 초기 cycle-null 유지 이유 교체안은 철회했다. |

## 관계와 phase 해석 확인

Root가 현재 §3.3을 확인한 결과 S5-03 C3은 V이전/V새 예시 ID 문자열 자체를 요구하지 않는다. actual OFF, Text 표시 후 재연결 입력, followup Context C현재, 기존 T보고서/X보고서 결과 read, 원래 완료 상태와 새 Task/Conversation 없음으로 관계를 판정한다. 그 증거를 실제 확인한 A1/A2의 C3 PASS를 제안했다. 재연결 응답 누락은 C1/C6/endpoint에서 독립 유지한다.

S1-02 C3도 원래 요청의 새 Task/Execution 고유 연결과 T보호 분리를 판정한다. 초기에는 CREATE_TABLE 명칭에 따라 C1/C5 실패를 유지했으나 아래 기능 동등성 재검토에서 다섯 실제 표 형식 매출 보고서 작성 기능을 확인해 C1/C5 PASS를 제안했다. 이 관계 기준으로 실제 evidence가 맞는 A1/A3 C3 PASS를 제안했다. 기존 T보고서 수정 요청을 새 Task로 잘못 연결한 S4-01은 별도 실제 금지 효과이며 A3의 누락된 broad Task association critical을 추가 제안했다.

S6-01 cycle은 실제 유효 수정 payload로 생성된 현재 revision 2 결과의 최종 전달이다. A2의 최종 artifact와 full Text/Voice를 확인했고 Root도 동일 해석에 동의했다. SEND enum 오해로 인한 invalid 판정과 초기 audit의 C6 checkpoint 누락을 cycle에 복제한 null 유지안은 철회했다. 원본 C6 checkpoint 실패는 유지한다.

## 완전성

진행 중 adjudication은 오류 후 재시도로 늦게 도착할 수 있다. A1~A5 각 21개 tuple이 모두 실제 adjudication을 갖는지 최종 확인한다. judge-audit 불일치나 uncertain은 audit 대상으로 유지하며, 오류 원장에만 있는 case를 완료된 판정으로 세지 않는다. 최종 완료 때 검토 ledger 및 정정 제안 파일 hash를 추가한다.

## 중간 완전성 확인

A66개 실제 adjudication을 대조했다. 각 tuple의 원래 grade canonical hash와 실제 trial line hash/pointer 및 제안 ID는 [poc-cases-audit-coverage.jsonl](poc-cases-audit-coverage.jsonl)에 연결했다. A4 S4-03 uncertain 표시는 실제 예산 소진/채택 목표 없음/명령 및 전달 없음/기존 상태 유지로 실패 근거를 확인했으며 점수 제안은 없다.

A4 S2-02와 S2-04에서는 원래100/120백만원 가격을 instruction에서10/12 million KRW로 잘못 재진술한 실제 인계를 확인해 C2 FAIL을 제안했다. payload의 원문과 실제 join artifact에는 올바른100/120이 남는다. 서로 다른 실제 표현을 함께 보존하고 이 숫자 오류를 C5/C4/C6/cycle에 자동 복제하지 않는다. Stub가 oracle로 정답을 교정한 것으로 설명하지 않는다.

S2-05의 QA02 자체는 새 근거를 확보한 실제 제안서 접수다. E새의 새 기준이 adopted/인계에서 확보되지 않은 A2/A4에 valid null 및 wrong_first true를 제안했다. 실제 접수 사실/최초 publication과 부분 사실은 기록대로 남으며, 이는 지정 반응의 필수 사실 미충족을 뜻한다. C6 tail 실패의 일괄 적용은 아니다.


## A86개 확인과 최초 반응 추가 대조

A1=18, A2=16, A3=17, A4=18, A5=17개로 실제 판정 86개를 검토했다. 아직 없는 19개는 오류 원장 이후 재시도 대상이며 원시 trial의 예비 대조만으로 판정 완료에 포함하지 않는다.

명확한 요청에 특정 source/Task를 다시 요구한 실제 QUESTION은 단순 진행 filler가 아니다. 실제 최초 질문이 후기 결론보다 먼저인 A1 S2-01, A3 S2-02, A5 S2-01에서 first_actual을 질문 publication으로 바로잡고 wrong_first=true를 제안했다. A5 S2-01의 후기 올바른 비교 전달과 유효 cycle은 유지한다.

A4/A5 S3-03은 최신 BUSY(+일부 FREE 대안)를 전달했으나 지정 반응의 일정 미생성 사실을 사용자에게 전달하지 않았다. 실제 효과=0을 미생성 안내로 대체하지 않는다. Root가 확인한 지정 반응 필수 사실 기준에 따라 wrong_first=true를 제안하고 valid=null을 유지한다. 실제 거절/실패 안내와 일부 사실을 말한 불완전 core의 구분은 별도 이유에 남겼다.

A5 S5-03은 후속 Context의 conversation_id=C현재, 기존 Task 결과의 actual read, 동일 T보고서/X보고서 완료 상태와 신규 Task/Execution 없음으로 C3 관계를 확인했다. 예시 Voice session ID가 모든 Meaning 필드에 없다는 이유로 관계 실패로 세지 않는다. A5 S6-01의 유효 revision 2 수정과 final result cycle은 C6의 checkpoint 설명 누락과 분리해 원래 판정을 유지한다.


### 정상 단계 진행과 첫 반응 오류의 구별

A5 S2-02의 올바른 source/criteria payload와 현재 ACCEPTED/완료미확인 안내는 정상 단계 진행이다. Root 확인 후 wrong_first=true 제안을 철회해 original false를 유지했다. 과거 근거/조건을 알리는 지정 핵심 사실 부족의 valid=null은 유지한다. payload 자체가 틀린 A2 S2-02 접수나, 같은 atomic 목표를 종결하면서 BUSY/미생성 중 한 사실을 빠뜨린 S3-03과 구별한다. S3-01/S3-02의 올바른 독립 목표 답변 또는 유효 단계 안내도 다른 목표/단계가 남았다는 이유로 첫 반응 오류로 바꾸지 않는다.

A1 S5-03의 대화 연결 근거 부족은 actual Context C현재, 기존 Task/Execution 결과 read, OFF/재연결 상태를 대조해 해소했으므로 C3 PASS와 연결해 uncertain=false도 제안했다. 원래 judge passes와 불일치 목록은 그대로 보존한다.


A4 S4-03도 실제 8회 반환 거절과 BudgetExceeded 실패 fallback, 취소 Goal 미채택, 명령/효과/전달 0으로 여섯 FAIL과 null endpoint가 명확하다. Root 요청에 따라 uncertain=false 해소를 별도 제안했다. 실제 채택 기록에는 goals=[]의 실패 fallback 하나가 있으므로 모든 Meaning 채택이 없었다고 기술하지 않는다. 취소 처리 방향은 확정되지 않았고 실제 후보 실패를 evaluator 불확실성으로 대체하지 않는다.


## S1-02 실제 산출 기능의 동등성 재검토

Root와 독립 검토자가 실제 runtime/Agent wire를 대조한 결과 CREATE_TABLE은 DIRECT_RESPONSE label에도 외부 새 Task/Execution, 고유 접수와 source 기반 artifact 생성을 수행한다. 다섯 A 실행 모두 실제 2026년 Q3 매출 자료와 표 작성 instruction을 인계했다. 월별 Q3 매출 표는 요청한 표 형식 매출 보고서의 기능과 동등하며, 도메인 디자인 품질을 제외하는 이 PoC에서 report 단어/CREATE_REPORT enum 부재를 C1/C5 실패 근거로 사용할 수 없다.

이에 A1~A5 C1/C5 PASS 및 actual current Q3 artifact 최종 전달 cycle valid를 제안했다. QA02의 좁은 같은 Request의 실제 접수/진행 안내도 실제 전체 재생된 최소 Task 접수 구절로 valid/firstwrong=false를 제안했다. C6는 별도다. A1~3는 실제 checkpoint에 2026년 Q3 매출/표 작성 접수 요청이 전달돼 PASS를 제안했고 A4/5의 generic 접수 안내는 기간/매출 scope 누락으로 FAIL을 유지했다. 후속 artifact가 그 checkpoint 누락을 보정하지 않는다.

앞 C3/C4 PASS 제안의 이유에 포함됐던 CREATE_TABLE 목표/효과 실패 문구는 이 기능 대조에 따라 네 최신 FUNCTION-REFINEMENT 제안으로 supersede했다. 초기 제안과 원래 judge 판정은 그대로 보존하며 최신 기능 해석을 적용한다. 기존 T보고서 수정 요청을 새 Task로 바꾸는 S4-01의 실제 잘못된 association은 이번 S1-02 새 보고서 Task 생성과 구별하고 해당 금지 효과 판정은 유지한다.


S3-02의 phase와 역할을 재대조했다. A1은 실제 현재 요약 산출물과 민수 To를 소비한 메일 접수 P3를 QA02 endpoint로 제안하며, P1 정상 요약 접수의 wrong_first=false와 P4 최종 SENT cycle은 유지한다. 다만 발화에서 민수는 문서 담당자로만 등장하고 수신자 접수 역할이 없으므로 C6 FAIL을 별도로 제안한다. A2/A4/A5의 P3는 수신자 역할을 명시하며 기존 C6와 P3 endpoint, P4 cycle을 유지한다. A3의 P3 역할은 명확하나 P1 요약 완료 뒤 P2 현재 RUNNING 전달이 있어 단계 진술 모순의 추가 검토를 별도로 요청했다.

A3 S3-02의 P1 실제 요약 완료 뒤 P2 현재 RUNNING 전달은 동일 요약의 단계 사실 역전으로 C6 FAIL을 제안했다. 첫 P1의 정상 결과와 wrong_first=false, P3의 민수 수신자 요약 메일 접수 endpoint, P4 실제 SENT cycle은 유지한다. 두 publication의 actual Text/Voice 시각과 사실 snapshot을 제안 근거에 보존했다.


S6-03 actual local disposition을 다섯 A 반복에서 재확인했다. 각 OWNER_READ(E15231/E17091/E17743/E15189/E14800)는 DENIED이며 권한 부족을 이유로 둔 unresolved Meaning이 실제 채택(E15329/E17179/E17828/E15240/E14888)됐다. source 본문/금액과 외부 효과가 없고 full 권한 거부 안내가 전달됐다. generic ContractError/input_ref 실패 fallback으로 끝난 경우와 구별되므로 기존 C5 PASS를 유지한다. S6-04 A1/A3/A4의 SET_PROPERTY H1, A2/A5의 ANSWER goal에는 read:true 동의 확대가 없으며 consent는 비어 있다. C4 PASS를 유지하고 질문 미처리/실행 실패의 C5 및 지정 endpoint 실패를 그대로 보존한다.


A1/A3 S2-04의 E1 기준/가격 제외가 actual 인계에서 확보되지 않은 접수는 QA02 valid=null/wrong_first=true다. A3의 current artifact는 해당 필수 근거/조건이 없어 cycle=null을 제안하고 A1의 원래 null도 유지한다. C6는 독립 actual 접수 및 이번 두 번째 결과의 참조 전달로 A3 PASS를 유지하며 A1 PASS를 제안한다. A1 C3도 S202 actual effects=0로 첫 Proposal이 생성되지 않은 준비 실패를 보존하고, 현재 고유 새 Task/Execution과 존재하던 T보호 유지의 실제 관계로 PASS를 제안한다. 성공한 firstTask를 선주입하지 않았다. A5의 실제 동일 기준/가격 제외를 포함한 인계와 새 결과는 기존 판정과 endpoint 근거를 확인했다.


S2-03 A1의 actual P1→D1@v1 공개 연결과 실제100 read/명령/산출물은 C1~C5 및 좁은 QA02 접수/current final cycle 근거를 충족한다. checkpoint P1의 발화는 generic 접수만이어서 당시100을 맡겼다는 C6 필수 사실 누락은 tail100으로 보충할 수 없으므로 C6 FAIL을 제안한다. A2 S2-03은 실제 Meaning 채택 없이 불필요 질문만 제시해 C1 FAIL과 맨 첫 질문 core/wrong=true를 제안하고 효과/완료 실패를 유지한다. S4-02 A1/A2는 모두 실제 stop와 late old discard 및 업무 유지가 있으나 새150억원 설명 처리를 하지 못했다. C5의 모든 필수 효과 기준으로 A1 C5 FAIL을 제안하며 단순 새 실패 publication만으로 설명 처리를 통과시키지 않는다. local-stop의 성공 반응은 독립 유지한다.


S4-02 A3의 실제 150억원 새 직접 응답은 내부 ANSWER target 실패와 구별해 C1/C2/C5를 유지한다. A4는 EXPLAIN 방향이 실제 채택됐으나 final빈값/receipt=false와 설명본문 없는 outcome이므로 C5 FAIL을 제안한다. A5는 채택0/새발화0으로 rejected 의도/범위의 C1/C4 FAIL을 제안한다. S2-03 A3/A5도 generic checkpoint의 당시100 사실 누락을 C6 FAIL로 제안하고 정상 v1접수/산출물 cycle을 유지한다. S5-04 A3/A4/A5의 proper 기존Task UPDATE 및 current revision2 결과와 네 독립 전달을 확인했다. A4는 P5 메일 RUNNING 유지와 P6 실제 report접수가 같은 Request/generation에 나눠 전달됐으므로 마지막 필수사실 P6에서 valid endpoint를 제안한다. 단일publication 요구는 원문에 없고 첫 정상진행/wrong=false를 유지한다. A3 기존메일 접수중 문구는 RUNNING 및 신규mail효과0의 실제 진행과 연결하며 새발송 접수로 확대하지 않는다.


## A105 검토 완료와 최종 원장

A1~A5 각각 21개, 전체 105개의 원판정과 실제 trial을 검토했다. main 원판정은 전체 A105/B105=210이며, 늦게 도착한 A1/A2 S5-04도 같은 원문 및 실제 기존 업무 관계 기준으로 확인했다. coverage는 각 현재 trial 원문 줄 hash/pointer와 main canonical row hash, 실제 Request 및 제안 ID를 보존한다. 모든 105개 tuple이 고유하며 누락과 중복이 없다.

A1/A2 S5-04 원판정은 모두 통과였지만 실제 CREATE_TABLE은 UPDATE_TASK/target=T보고서 표기와 달리 새 Task/Execution을 만들었다. 기존 T보고서는 revision 1, PARAGRAPH, RUNNING 그대로이고 마지막 P7도 새 Task의 revision 1 산출물이다. C1/C3/C5/C6 및 update QA02/cycle, 실제 잘못된 Task 연결을 정정 제안했다. C2의 D1/네 사건 값과 C4의 기간/메일/보호조건, 네 독립 반응과 local stop은 유지했다. 실제 완료 후 P7 발화이므로 허위 조기완료로 만들지 않는다. 이는 incompatible kind/operation과 공통 실행/최종 검증의 실제 처리 한계이며 A 구조 자체의 본질적 불가능성으로 일반화하지 않는다.

제안은 113행이다. 철회 행 3개 및 superseded target 8개를 제외한 수용 검토 대상은 102개다. [Disposition 목록](poc-cases-proposal-disposition.json)은 각 철회/대체 edge와 제외 ID 및 검토 대상 ID를 모두 명시한다. 원래 판정과 모든 초기 제안은 보존했다. S204 C3 근거 refinement는 선행 S202 E3995를 audit 비교 근거에만 남기고 현재 condition에는 S204 CMD/E5039/E5046만 사용한다. 실제 C3 결론과 선행 준비 실패는 그대로 남긴다.

113행 중 경로를 가진 제안 110개의 original canonical anchor 및 old_value 불일치는 0건이다. coverage 105개의 main anchor와 실제 raw 줄 hash 불일치도 0건이다. 추가 API/모델 호출, 후보/정답/frozen source/원시 trial/원판정 수정은 모두 0이다. 이 검토는 실제 근거에 따른 판정 정확성 검토이며 후보 정확성을 다시 측정하거나 QA 우열을 확정한 결과가 아니다. 실제 점수와 cap/정답만/오답 포함 평균은 root가 독립 수용 원장을 적용한 뒤 산출한다.

최종 SHA-256:

| 파일 | SHA-256 |
| --- | --- |
| `poc-cases-adjudication-proposals.jsonl` | `aaf1c65e716d636b54e718bd1054d42201bb2c7ce875032880da4e4944957392` |
| `poc-cases-audit-coverage.jsonl` | `00cc4a961080eda449eb00d2c2edba55a5012ac9004e942923b6e2469273a847` |
| `poc-cases-proposal-disposition.json` | `ef887b2fae368c6672da4c6d205d5c2191b6509b978290f94237c3f1c336df3d` |
