# V5 전체 scene 포트 읽기 검토

상태: `V5_INVALIDATED_ACTUAL_READ_ONLY_AUDIT`. Root는 A1 S6-02의 실제 QUESTION_PRESENT 후속 입력 누락 P2를 독립 확인하고 20:53:36 UTC에 V5를 중단했다. 이 자료는 진단 근거이며 공식 QA 점수 또는 후보 우열의 근거가 아니다. 이후 코드의 수정 결과를 이 V5 actual trace로 성공했다고 표시하지 않는다.

검토 범위는 A1 완료 21개 scene과 B1의 완료된 5개 scene이다. B1 S2-04의 진행 중 일부 입력 원장은 존재하지만 완료 trial이 없으므로 완료된 semantic 경로로 포함하지 않았다. 작은 raw row, 현재 원문, V5 frozen worker/runtime/owner 포트만 읽었다. API, 테스트, 후보 실행, 후보 또는 fixture 수정, 메모리 raw 전체 읽기는 하지 않았다.

근거: [V5 manifest](../dp41-p21-mac-text-v5-20261012/manifest.json), [A1 actual trace 검토](v5-a1-actual-trace-review.md), [QA 정규화 정적 검토](v5-metric-normalization-review.md). Source digest는 `2143b39761bd0b74e8d9d973d8b59c1e8ba722ccc24ebafcc0d7e03d85cda4b0`다. 원문은 현재 ASR-QA-v6와 common workload의 P21 및 [공통 실행 계약](../../../../docs/architecture/12-decisions/decision-packages/04-40-common-execution-contract.md)이다.

## 결론

확정된 공통 P2는 QUESTION_PRESENT의 실제 유효 제시 뒤 지정 후속 발화가 입력되지 않은 한 건이다. 같은 입력 공급 조건을 합법적 질문 제시 경로 전체에 연결해야 한다. 아래 나머지 actual port 대조에서 추가 확정 P1/P2는 발견하지 않았다. 모델이 선택하지 않은 경로의 성공을 주장하지 않는다.

## 입력과 source 사건

A1 21개와 B1 완료 5개의 `case.events`와 `observed_source_availability[*].event`는 event ID, type, 예정 offset, payload 및 순서가 모두 같았다. 단순 event ID 개수 일치만 확인한 것이 아니다. SOURCE_AVAILABILITY의 actual offset은 원시로 별도 보존되며 예정 offset을 측정 시각으로 대체하지 않는다.

S2-02 준비 P0~P4도 A1/B1 모두 실제 PREPARATION_AVAILABILITY의 원문 event와 순서대로 같다. 초기 publication 원본, 실제 전달 receipt, T비교/D2v1, T비교/D2v2, T견적/D3v1을 owner에 처리한다. A1의 이후 S2-04는 실제 S2-02 owner/cache/history를 넘겼다. 기대한 successful predecessor를 fixture로 새로 주입한 경로가 아니다.

A1에서 S2-04 이후 B01~B04의 갱신 4개와 maintenance tick의 원장이 존재한다. B1은 그 시점에 도달하지 않아 해당 활동 갱신/마지막 철회/drain/idle의 완전한 관측을 갖지 않는다. 중단된 B1을 완전한 P21 비용 또는 메모리 활동으로 집계하면 안 된다.

source 갱신과 historical read의 구분은 실제 port에 보존된다. S2-03은 현재 D1v2가 도착했어도 SOURCE_AT_REV/기존 v1 조회 경로를 가진다. S3-03은 calendar 갱신 뒤 최신 BUSY의 실제 source를 사용한다. S2-05의 설명/두 제품 document 세 사건은 availability 이후 owner가 가진 사실이며 후보에 미래 schedule을 주는 자료가 아니다. 실제 A1은 설명 source를 읽지 않았으므로 그 조건 누락을 원천 주입 결함으로 바꾸지 않는다.

## 질문, 답변과 Voice ON/OFF

| 실제 경로 | 실제 확인 | 후속 공급 조건 |
| --- | --- | --- |
| A1 S6-01 생성 clarification | 실제 질문 제시와 Q/revision receipt 뒤 canonical 3분기 보고서 후속 발화가 입력됐다. | 보고서 대상/수정 scope의 실제 질문에만 지정 답변. 정답 Task 선택을 후보 출력에 보정하지 않는다. |
| A1 S6-02 goal QUESTION_PRESENT | 현재 Q1v2의 whole Voice/receipt가 있는데 지정 후속 발화가 입력되지 않았다. | 확정 P2. 기존 CLARIFY 안에서만 AnswerSource가 호출돼 실행 경로에 따라 입력 공급이 달랐다. |
| A1 S5-01/S5-04 automatic AGENT 질문 | Q메일 및 두 burst 질문은 실제 Voice 제시 receipt를 가지며 답변은 0개다. 철회된 report 질문은 유효 전달되지 않았다. | 원문에 추가 답변이 없으므로 질문 제시 사실만으로 답변을 새로 생산하지 않는다. |
| B1 S2-01/S2-03 생성 clarification | VIA가 만든 질문의 실제 receipt 뒤 일반 no-hint reply가 1~2회 공급됐다. | 어떤 표/조건이 정답인지 또는 새 권한을 주지 않았다. clarification가 길거나 실패한 경우를 구조의 필수 비용으로 일반화하지 않는다. |
| A1 S5-03 Voice OFF Text | 실제 결과 Text 표시 뒤 재연결 입력과 Voice 결과가 별도로 존재한다. | 원래 2초 재연결 발화는 이 장면의 사용자 목표이며 QA03 시작이 최초 Agent 사건이 아니다. |

V5 fixture의 일반 unexpected reply에는 `only_for_origin=VIA_CLARIFICATION`, `never_answer_agent_questions=true`가 명시돼 있다. 기존 V5 callable은 origin을 직접 검사하지 않고 호출 위치인 CLARIFY에 의존했다. 새 공통 presentation hook에서는 이 정책을 소비해야 한다. 외부 Agent 질문, 단순 승인 대기 질문 또는 원문에 추가 답변이 없는 장면에 일반 reply를 자동 제공하면 입력 조건이 달라진다. 이는 확인한 수정 필요조건이며 실제 V5 A1에 그런 무료 Agent 답변이 발생했다는 별도 P2는 아니다.

Voice ON에서는 actual Text와 whole/partial Voice, presented Q scope 및 current revision을 함께 확인해야 한다. Text-only snapshot의 빈 Voice를 후기 실패로 오판하거나 Voice onset만 있는 미완료 질문을 whole presentation으로 보정하지 않는다. Voice OFF에서는 Text-only 실제 제시를 같은 Voice ON 실패로 취급하지 않는다. 이번 P21 actual scene의 지정 consent와 report clarification은 Voice ON이었으므로 일반 Voice OFF 질문 followup 성공까지 실측한 증거는 없다.

S6-02 fix에 필요한 수용 조건은 실제 Q1v2 목적, T메일, 민수, READ_CONTACT/PROVIDE_TO_TASK 및 receipt를 보존한 once-only 후속 입력이다. 최초 '응.'을 새 동의로 재사용하지 않고, source release는 실제 허용된 ANSWER 이후다. 새로운 모델 Request를 만든다는 이유로 기존 logical Request 예산을 초기화하거나 후보의 후속 의미를 성공으로 채워 넣으면 안 된다. 답변의 model 처리가 올바를지는 actual 새 freeze에서 관측해야 한다.

## 실제 state, payload와 오류 귀속

| scene 범위 | 포트 대조 | 후보 오류 또는 범위 제한과의 구분 |
| --- | --- | --- |
| S1-01/S1-02 | 허용 원문 read 또는 metadata ID에서 실제 원문 위임, actual acceptance와 tail result가 있다. | A S1-01은 input ID를 question_ref로 잘못 사용해 거절됐다. S1-02의 CREATE_TABLE 선택은 원래 report 목표와 구분해야 한다. |
| S2-01 | 세 pointer/observation이 원시 시각을 갖고 입력 동안 실제 공급됐다. INPUT_SELECTION은 모든 원천 후보를 주며 말의 span을 정답 source로 미리 결합하지 않는다. | A의 자유문 수치와 B의 모호한 후보 선택, 추가 질문은 실제 모델 판단 대상이다. |
| S2-02/04 | Task-result actual locator, 현재 result revision과 실제 E1의 Text/Voice channel view를 읽을 수 있다. | A S202 heard criteria instruction과 수치 변환 오류, B의 긴 미완료 질문을 별도로 관측한다. source가 있다는 사실만으로 모델이 실제 활용했다고 주장하지 않는다. |
| S2-03/05 | 원본 history와 신규 source availability가 보존됐다. | B S203의 background Task references, A S205의 E새 미조회는 후보의 읽기/표현 선택이다. |
| S3-01 | 독립 두 goal의 실제 결과 publication이 있다. | QA02/03 두 결과 endpoint는 필요한 선행 전달을 포함해 확인해야 한다. 단일 더 빠른 publication으로 모든 목표를 대체하지 않는다. |
| S3-02 | 현재 producer artifact→symbolic body→SEND_MAIL actual payload와 recipient, 중복/옛 result 거절을 대조했다. | currentness 검사가 성공했다는 사실과 자유 응답의 수신자/본문/완료 사실 정확성을 구분한다. |
| S3-03 | 최신 calendar owner read, BUSY와 미생성 disposition의 분기 조건을 가진다. | 원문 갱신과 모델의 현재 조건 선택은 별개다. |
| S4-01/02 | 실제 old model job/held return 폐기 및 이전 output generation 폐기와 local stop가 있다. | A S401의 current CREATE_TABLE 선택을 old job 폐기 성공으로 정답 처리하지 않는다. |
| S4-03 | CANCEL_TASK 실제 명령/receipt/종료 미확인 경로가 지원돼 있다. | A는 operation이 없는 schema를 반복해 예산 소진했다. 실행되지 않은 정상 CANCEL 명령을 만들어내지 않는다. |
| S5-01/02 | Task/Execution/revision 확인, 질문 철회, 중복과 이전 result 거절의 actual 흔적이 있다. | 다른 Task의 빠른 결과 또는 생성됐으나 실제 표시되지 않은 질문은 지정 endpoint를 대체하지 않는다. |
| S5-03/04 | Voice connection 상태, result 보존, 재연결 및 독립 4개 source 사건과 사용자 정정이 있다. | sequential playback 차례 비용을 보존하고 잘못된 후기 state 문장도 actual delivery에 남긴다. |
| S6-01/02 | clarification scope와 consent Q/revision/currentness는 owner 상태에 있다. | S601 모델 이후 처리와 S602 확정 harness followup 누락을 같은 후보 실패로 묶지 않는다. |
| S6-03/04 | 권한 거부는 actual allowed port를 막으며 Q기간의 현재 scope와 ANSWER 계약을 지원한다. | A S604의 operation 누락/version 오류는 후보 schema 실패이며 질문 정보 원천 부족으로 관측되지 않았다. |

common runtime의 각 Agent payload는 actual goal/arguments, relations, constraints, preservations와 실제 source 원문을 운반한다. actual SEND_MAIL은 current body artifact를 받았다. domain stub은 canonical case ID로 expected artifact 또는 missing criteria를 만들어주지 않는다. report/proposal의 실제 문서 품질이나 메일의 실제 외부 송신을 이 모사 경로로 주장하지 않는다. instruction의 잘못된 의미와 actual source의 맞는 숫자가 공존하면 둘을 judge에 제공하고 사후 보정하지 않는다.

유한 B의 partial selector가 일반 Goal/value를 새로 추출하지 않는 선언된 범위, 공통 bounded lexical 검색 relevance 한계, 모델의 잘못된 enum/locator/query 선택은 입력 replay P2와 다르다. 이번 검토는 이를 추가 P2 또는 구조의 본질적 정확성 한계로 재분류하지 않았다.

## 확인한 raw row 목록

모든 행의 예정 scene event는 SOURCE_AVAILABILITY의 actual event envelope와 순서대로 일치했다. raw SHA는 newline을 포함한다. count는 실행 흔적의 개수이며 정답률, 통과 수 또는 QA 분모가 아니다.

| tuple | 예정 event 수 | 실제 input 수 | actual command 수 | raw row SHA-256 |
| --- | --- | --- | --- | --- |
| A1/S1-01 | 0 | 1 | 0 | `41d8f906f087ee8f70bca6ca01c9e4011b443ef6e5bc0df17163738724146821` |
| A1/S1-02 | 0 | 1 | 1 | `060cce52dbe036581a6d30bfe6c4cd6b95baa8331bfe950cd8700ddd78f15c0e` |
| A1/S2-01 | 3 | 1 | 0 | `bf428a8b77bcbbd306f0e30233a616911ea0f43ba901cb371e729f9b2c3bdfe1` |
| A1/S2-02 | 0 | 1 | 1 | `5bdc0b3e51cbdb8137070824a1820bc4f35f2544d96ce1ed0797e3ce5b72efce` |
| A1/S2-03 | 1 | 1 | 1 | `c85fe0373717db5935c0e30840312db7316a8dcb11e164e0b6aaa302cd93d37f` |
| A1/S2-04 | 0 | 1 | 1 | `64e28cfae0df0230993fee5103697abbee1e8788c2431dbc6c414d9a0a925a6e` |
| A1/S2-05 | 3 | 1 | 1 | `94014c7ad59c4ebd1beb53b73b20ed15c96421a3a05268c30f15f30440ee0a2b` |
| A1/S3-01 | 0 | 1 | 0 | `e23849c667c93117e794a5b2f7899f63d7d04d2efd27842a5c443886fbe232bd` |
| A1/S3-02 | 1 | 1 | 2 | `ecb3c161f6b8b634a7251610e1eeecdecb71aec19a5630a50de7bc336278dadc` |
| A1/S3-03 | 1 | 1 | 0 | `5a1b9a03123cad782a5661b8bebdf3683b926b96ad3c648c49f4255f1ca09896` |
| A1/S4-01 | 1 | 1 | 1 | `37873843b6ccba06dae2af3410036530fa230261b1624b2ced96b8abbfa3113f` |
| A1/S4-02 | 1 | 1 | 0 | `b894e09110eca1431fa918a19db8e66b127a953da3c5cd379c9d19231a6e308e` |
| A1/S4-03 | 0 | 1 | 0 | `7844109f24b16ea69a2e0db43eef461f4547ec44b4c375a28b0f8cad27644983` |
| A1/S5-01 | 3 | 0 | 0 | `aee781d46dc5a4464c2f030d9bb5402894f9846b98c3ea92d2feac89de1221c5` |
| A1/S5-02 | 4 | 0 | 0 | `085824c789467cbbee38af7c72b275b1792372e5f26bb5423829920144f633ab` |
| A1/S5-03 | 2 | 1 | 0 | `de184278c26ca08a76fa5d8d7bf10c767bfad19b0812bef231339be866456e3f` |
| A1/S5-04 | 4 | 1 | 1 | `7d88c19259e9f40e7827cae426816dad1c8745b689237da6a20f263336ae4418` |
| A1/S6-01 | 0 | 2 | 1 | `576d32ad7fa2008e8d2e05d4888f6a9e7e3e653ea71bf748eae35c12f676d13f` |
| A1/S6-02 | 2 | 1 | 0 | `36845c9a8b73ca54648f7a8180c56134d7a8a67f23d1d50485612f543d9a59b7` |
| A1/S6-03 | 0 | 1 | 0 | `5fe0958ddde7f6562d2a1156da66c222d09e0a3e9ea4cf86798baa915b6720ed` |
| A1/S6-04 | 0 | 1 | 0 | `92755567a5f9803458640e77311fb402b7f7adc0012d8fa9e58919ed1be44d0c` |
| B1/S1-01 | 0 | 1 | 0 | `c4bbf0d7c44f1bece78d3e89864026c1c82503d7e9167436fb30e6065d83535a` |
| B1/S1-02 | 0 | 1 | 1 | `07ddc3657a21ab946a94c56d7a745dd504d6b2be03396c1151cfea684ba522db` |
| B1/S2-01 | 3 | 3 | 0 | `3f8ed6bb535497bc9cd7fa45feef7714af4ad0cd8ed31bfe819b5f90cf6fdaed` |
| B1/S2-02 | 0 | 1 | 0 | `17b36658ab6968225e19f76915f02eef8cf6df6315e1a4c003e4e47e22795400` |
| B1/S2-03 | 1 | 2 | 0 | `b5fa0472ffd5762c5758214744b1657ef2aba2769dbae06ef2c67c4ee9a3adfe` |

후속 독립 확인은 수정자가 완료한 최소 공통 replay patch와 관련 Voice ON/OFF, current/withdrawn question, 중복 및 no-answer 회귀를 읽는 단계다. 이 문서는 그러한 테스트를 실행했거나 수정된 후보의 성능을 확인했다고 표시하지 않는다.
