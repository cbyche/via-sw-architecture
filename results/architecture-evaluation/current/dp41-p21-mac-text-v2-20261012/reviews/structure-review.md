# DP41 frozen 후보의 독립 구조 검토

검토자: `/root/poc_cases`. 범위: v2 frozen-source 읽기 전용. 실제 의미·성능 점수 판정과 구분한다. 본 후보 변경이나 API 실행은 이 검토에서 하지 않았다.

정답/oracle/미래 사건 누출, 잘못된 payload를 정답으로 보정, B의 1회 호출 강제, A의 순차 강제 또는 공통 실행 경로의 차별처럼 측정을 중단해야 할 P1/P2를 발견하지 못했다.

- A는 실제 READ/CLARIFY/PROPOSE loop를 모델이 선택한다. 독립 READ는 병렬이며 조회 결과는 다음 모델 Context에 들어간다.
- B는 Frame, reference의 필요한 부분 모델 선택, 실제 확인 질문과 갱신 Frame을 사용한다. 최종 binding은 코드가 구성한다.
- 모델 입력의 allowlist에 oracle/case ID/미래 사건 계획이 없으며 최초 source metadata에 본문을 무료로 제공하지 않는다.
- 공통 stub은 실제 수신 source와 payload를 소비한다. 후보의 직접 응답 문장도 그대로 전달한다.
- bounded 작업의 DIRECT_RESPONSE와 NEW_TASK 위임을 모두 허용한다.

현재 구현의 해석 범위:

| 구현 선택 | 결과 해석 |
| --- | --- |
| B source KNOWN_REF에도 실제 owner read, 이미 제공된 Task/질문/publication은 HOST_METADATA 결합 | 실제 조회 비용이며 모든 B 구현의 최소 비용으로 일반화하지 않는다. |
| B 독립 read는 병렬, 여러 reference 부분 모델 선택은 순서대로 수행 | 직렬 대기는 이번 구현 선택이다. |
| B 부분 판단은 reference 후보 선택에 한정 | 임의 속성/부분 관계의 의미 해석까지 구현했다고 주장하지 않는다. |
| 불확실성 후 확인 질문·Frame 재생성으로 추가 조회 | 사용자 확인 없이 재조회하는 모든 B 전략을 평가한 것은 아니다. |
| 양안 모두 유한 Goal 계약 | 이 계약/P21 내 비교이며 임의 자연어·전체 제품 비교로 확대하지 않는다. |

지원 범위는 Goal 최대4, B reference 최대8, read 최대16, Request 물리 모델 호출 최대8, 확인 질문 최대2다. 최종 Goal/연산/selector/WHEN 조건은 frozen `dp41/contract.py`, A/B 모델 규약은 `dp41/prompts.py`에 보존했다.

오류 분류는 실제 모델 출력과 합법적인 대체 계약 표현, 코드 결합, owner read receipt, adoption/currentness/permission, 받은 payload와 실제 전달을 연결한다. 모델이 `unsupported`를 냈다는 이유만으로 구조의 표현 한계로 단정하지 않는다. 표현 가능한 의미의 잘못된 생성은 모델 오류, 적법한 계약의 잘못된 처리는 구현 결함, 필요한 의미 자체를 계약으로 표현할 수 없는 경우는 이번 지원 범위의 한계로 구분한다.

A1의 읽기 연결 검수에서는 Context/output artifact164개 hash, CINTERPRET41회의 요청/반환, candidate read21개/OWNER_READ 영수증, 확인 질문/답변, S4-01 이전 job/현재 job 분리 연결을 확인했다. 점수 판정이나 실제 replay 실행을 대신하는 검수가 아니다.
