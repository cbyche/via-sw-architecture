# Target Architecture 설계 완결성 점검

> 상태: **설계 검토 / 사용자 합의 전 / 구현·측정 없음**
> 점검일: 2026-09-29
> [Architecture 본문](./architecture.md) · [그림 목록](./diagrams/README.md) · [검토 기록](./review-log.md)

이 문서는 그림을 작성하며 확인한 기능 경로·상태 소유권·장애 경계를 정리한다. 아래의 “경로 있음”은 문서에 해당 동작을 설명했다는 의미다. 구현 성공, 모델 capability 확보 또는 ASR 검증을 뜻하지 않는다. 기존 Decision Package를 재구성하거나 새 비교 실험을 동결하는 문서도 아니다.

## 1. 대표 사용자 행동의 경로

기준은 [확정된 대표 UC](../../05-representative-use-cases.md)다. 신규 필수 기능을 추가하지 않는다.

| UC | 목표 Architecture의 경로 | 중요한 예외·보존할 조건 | 본문 / 그림 |
| --- | --- | --- | --- |
| UC-01 일반 질문·후속 대화 | S2S 직접 응답 + Controller admission + 실제 응답 기록 | 이중 응답 방지; 후속 지칭에 실제 전달 내용 사용; 기본 admission 정책 확인 중 | §8·13 / 08 |
| UC-02 한정 자료 조회·설명 | Context read → 해석 → Core 직접 응답 | source·기준 시각 보존; 조사·업무 분석으로 확장되면 Agent | §6·7·9 / 03·10 |
| UC-03 먼저 선택하고 말하기 | 사전 선택의 유효 구간 + 입력 timeline | 발화 전 선택도 유지; pointer·selection 충돌은 미확정 | §6·9 / 04 |
| UC-04 말하면서 지칭하기 | 표현별 acoustic interval과 당시 화면 결합 | 복수 대상·정정·전사 지연·capture gap을 마지막 화면으로 덮지 않음 | §6·9 / 04 |
| UC-05 숨은 자료·이전 대상 | bounded source 검색 + Conversation·Task owner read port | background identity와 현재 보이는 화면 구분; 후보 coverage 필요 | §9 / 03·10 |
| UC-06 요청 보완 | Pending User Interaction → 새 Turn → 기존 Request revision | 이미 확정한 제약 보존; 짧은 답의 질문 binding이 모호하면 재확인 | §5·6·12 / 02·03 |
| UC-07 직접 응답에서 위임 | Response Record 참조 → 새 Request·Task → Agent | Voice session이 끝나도 참조 보존; 미전달 음성은 들은 내용으로 사용 금지 | §5·13 / 02·08 |
| UC-08 새 업무 위임 | Controller → Task Manager command → Gateway → Agent | capability·권한·precondition 확인; 적합한 Agent 없으면 미지원 | §11·12 / 01·05 |
| UC-09 복합 요청 | Request Graph → 의존 결과 version → 조건별 node 해제 | 불명 조건은 거짓이 아님; 독립 업무 계속; Agent 내부 계획은 범위 밖 | §12 / 09 |
| UC-10 기존 업무 조회·수정 | 기존 Task binding → 확인 상태 또는 후속 command | 완료 Execution 재사용 불가 시 같은 Task에 후속 Execution 연결 | §5·12 / 02·09 |
| UC-11 음성 중단·정정 | Voice local stop → epoch 무효화 → 새 요청 해석 | Core·모델·Agent 취소를 기다리지 않음; 이전 release 거절 | §6·13·14 / 08 |
| UC-12 특정 업무 취소 | 미전송 hold / 실행 중 취소 command / 상태 조정 | 취소 접수·완료·불가·불명 구분; 외부 rollback이라고 주장하지 않음 | §11·12 / 05 |
| UC-13 비동기 진행·결과 | inbox → projection·domain outbox → Controller → publication | 새 발화 없이 전달; 중간 crash에도 결과·질문 인계 복원 | §12·13 / 05·08 |
| UC-14 여러 업무 관리 | Conversation별 Request 전이 + Task별 업무 전이 | 다른 Task의 대기·질문·실패가 새 입력과 독립 업무를 막지 않음 | §11·12 / 09 |
| UC-15 채널 전환·재연결 | Voice Connection과 Conversation·Task 수명 분리 | 재연결·새 대화가 Task 취소·재실행을 유발하지 않음 | §5·14 / 02 |
| UC-16 정보 동의·실행 승인 | 현재 Policy State + Pending User Interaction + 사용 port gate | Consent와 Agent Action Approval 구분; 오래된 “응” 재사용 금지 | §9·10·12 / 10 |
| UC-17 User Memory 제어 | Controller 의미 확정 → Context owner 변경 → 파생 view 무효화 | 삭제 tombstone; 과거 history로 삭제 선호 재생 금지; 현재 요청 우선 | §9 / 10 |
| UC-18 문제 안내·복구 | fault 격리 → durable state → source 조정 → 사용자 상태 복원 | 접수 불명 재실행 금지; 음성 전달 불명 보존; 실제 기능 복원까지 추적 | §11~14 / 05·06 |

## 2. 이번 점검에서 교정한 공백

| 발견한 공백 | 현재 주 설계 | 비용 또는 남는 한계 |
| --- | --- | --- |
| Task 상태는 저장됐지만 Controller callback 전에 crash하면 알림 유실 | projection + domain outbox 원자 기록; consumer event ID와 publication ID로 멱등 인계 | transaction·outbox 관리 비용; schema·crash 검증 필요 |
| “전송 직전 취소”의 실제 순서가 불분명 | PENDING→DISPATCHING CAS와 Conversation hold·command epoch·policy revision을 같은 Store 경계에서 검사 | acoustic 시작부터 hold 기록까지의 인식·IPC 지연은 남음 |
| 물리적 음성 재생과 기록을 원자적으로 복구하는 듯한 설명 | 확인된 audible 범위와 DELIVERY_UNKNOWN 분리; 불명 구간 자동 재생 금지 | 완벽한 exactly-once 청취 보장은 없음 |
| Model Access를 Core에만 그려 Voice 독립성이 불분명 | 하나의 논리 계약, Voice의 S2S adapter와 Core의 semantic adapter로 배치 | bounded IPC·session owner·lease 구현 필요 |
| Core 장애 중 UI control의 보장 범위가 과도함 | local Voice stop은 유지; Task 취소는 내구 접수 전까지 접수 성공으로 표시하지 않음 | Core 장애 중 업무 제어 가용성 제한 |
| Context가 다른 owner 상태를 어떻게 읽는지 불명확 | versioned owner read ports, source별 revision, commit 시 dependency 검증 | 여러 read를 하나의 동시 snapshot으로 가정할 수 없음 |
| 권한 확인 후 실제 제공까지의 경쟁 | Use Envelope와 현재 revision을 read/model/Agent/publication port에서 검사 | 이미 외부로 나간 정보의 소급 회수 불가 |
| 선행 결과가 바뀌거나 event가 재전달되면 후속 업무 중복 가능 | dependency result version + graph revision + node ID로 해제 identity 고정 | 외부 source 경쟁은 Agent precondition도 필요 |
| 공유 모델·event queue가 포화될 때의 동작 부재 | 유한 queue·입력 우선·progress 병합·terminal/question 보존·backpressure | 실제 예산·선점 capability는 아직 미정 |
| 긴 Core 답변 생성 비용이 해석 예산에 숨겨짐 | 해석 최대 2회와 별도 구성 최대 1회를 구분하고 총 deadline 적용 | 이는 초기 resource policy이며 성능 근거가 아님 |

## 3. 닫지 않은 질문과 책임

| 항목 | 현재 제안·대응 | 확정에 필요한 것 |
| --- | --- | --- |
| S2S direct admission의 기본 방식 | 매 요청 semantic 확인의 overhead 우려로 재검토; 맥락·범위 기반 조건부 직접 경로 제안 | 직접 범위·S2S 인계 계약 합의; 실제 판정 정확도·지연 검증 |
| S2S capability | time-aligned 입력, host 출력 제어, Text/audio 대응, 의미 보존 음성화, barge-in 필요 | 실제 모델 연결 확인; 미지원 기능을 숨겨 helper 모델을 추가하지 않음 |
| 시각 근거 처리 | UI 구조가 없으면 공유 semantic LLM의 이미지 처리 필요 | 목표 모델 capability와 화면 입력 계약 |
| Agent capability | 상태 조회·source revision·중복 방지·precondition·취소 지원을 profile로 노출 | 실제 Agent adapter 연결 확인; 미지원일 때의 보장 수준 표시 |
| 자원·시간 예산 | 유한 queue·buffer·deadline, pin된 evidence의 한도 초과도 gap으로 처리 | 목표 PC·모델 배치·동시 workload에 맞춘 수치; 이번 작업에서 측정 freeze하지 않음 |
| 개인정보 보관·삭제 | 최소 evidence 보존, memory tombstone, derived view 폐기, 외부 삭제 한계 명시 | 제품의 보관기간·백업 삭제·provider 정책; 현재 숫자나 법적 보장 미확정 |
| 발화 차례·중단 후 재개 | 사용자 발화 중 음성 알림 금지와 채널별 요약은 사용자 지정; 불명확한 재개만 확인하는 안 | 새 요청/대기 결과의 순서, Voice 재연결·다른 Conversation의 음성 안내 범위 |
| 기억 계층 | 단기 작업 Context·중기 대화/업무 view·장기 허용 기억과 내구 원본을 분리하는 제안 | 기억 승격·보관·삭제 정책, 요약 누락·원본 복원·저장 계약 |
| 실행 가능한 계약 | owner·revision·transaction·호출 경로를 문서로 지정 | schema, migration, provider adapter, crash/race 구현 검증 |

문서에 경로가 있다는 이유로 위 질문을 `완료`로 바꾸지 않는다. 새 모델·Agent가 필수 capability를 충족하지 못하거나 그림의 내구·권한 경계를 구현할 수 없다면 해당 구조를 다시 설계한다.

[이번 대화의 합의와 검토 제안](./interaction-and-memory-design.md)은 S2S·모델 확인 책임·복합 실패·기억을 구체화한다. 기능 경로가 문서에 있다는 이유만으로 제품 행동·상태 계약의 합의가 끝난 것은 아니다.

## 4. 다음 사용자 리뷰 순서

1. 그림 02·03·04: 사용자 입력, 요청·Task의 구분과 지칭 해석이 직관적인가?
2. 그림 08·09: 직접 응답·중단·복합 업무가 기대한 제품 행동과 맞는가?
3. 그림 05·06·10: 장애·권한 철회에서 어디까지 진행하고 어디서 보류하는가?
4. 그림 07: 네 품질 경로의 비용과 약점이 빠짐없이 설명되는가?

그 다음 전체 구조를 합의한다. Decision Package와 steelman 비교는 그 이후에 진행한다.
