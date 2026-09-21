# 12-01A. Scope & Coverage Ledger

> W12-G1-v1. **설계 책임의 연결 확인이지 구현/시험 통과 증거가 아니다.**
> `H`는 관찰 우선 인과 가설, `R`은 전수 점검/회귀 또는 공통 배경이다. R을 무관/N/A라고 단정하지 않는다.

## 1. 25개 원래 주제의 처리

| ID | 원래 주제 | 처리 | 소유 DP | 보존할 이유 / Highlight |
|---|---|---|---|---|
| S01 | Agent-neutral vs primary general agent | FIXED | 고정/범위 밖 | 03의 승인 경계. 과거 DP-00를 재개하지 않음. |
| S02 | S2S/Core 직접 응답 중계 | DP | INT-DP01 | 응답과 dispatch 책임 |
| S03 | Voice barge-in·audio buffer·jitter | TACTIC | INT-DP01, EXEC-DP01 | 필수 동작/보조 관찰. 독립 주요 DP 아님. |
| S04 | Model provider abstraction·helper 연동 | EMBEDDED | INT-DP01, IR-DP01, AGENT-DP01, EXEC-DP01 | 별도 11번째 범용 gateway DP로 중복하지 않음. M-01~09 추적. |
| S05 | Interaction evidence timestamp/selection/trajectory | EMBEDDED | CTX-DP01, IR-DP01 | 관측 제공과 의미 연결을 서로 다른 boundary로 유지. |
| S06 | Source access/materialization | DP | CTX-DP01 | Source→소비자 경계 |
| S07 | Conversation/Task context selection/summary | DP | CTX-DP02 | 기준 기록→Model 입력 경계 |
| S08 | Request Refinement·Referent·Task Association | MERGED | IR-DP01 | 서로 연결된 semantic pipeline의 단계 구성 |
| S09 | Agent Selection 판단 | EMBEDDED | IR-DP01, AGENT-DP01 | 판단 topology와 capability 계약은 다른 경계 |
| S10 | bounded Core 처리 vs Agent 위임 | DP | ORCH-DP01 | 허용된 배치 선택 |
| S11 | VIA-local tracked execution path | EMBEDDED | ORCH-DP01, TASK-DP01 | 필요성 판단을 보존하되 필수 Task로 선결정하지 않음. |
| S12 | Compound independent/sequential/data/conditional 관계 | EMBEDDED | IR-DP01, ORCH-DP01, TASK-DP01 | 분해 의미·배치·상태 추적을 보존. 범용 planner/Workflow Engine은 선결정하지 않음. |
| S13 | Task state ownership·supervision | DP | TASK-DP01 | 상태 전이 단위 |
| S14 | Persistence journal/snapshot/outbox/idempotency | TACTIC | TASK-DP01 | 구조 identity를 바꾸면 TASK-DP01 후보로 재분류. 기본 correctness를 위해 필요한 부분은 모든 후보에 포함. |
| S15 | Agent contract integration | DP | AGENT-DP01 | native 차이 흡수 경계 |
| S16 | Push/poll/feedback delivery | DP | TASK-DP02 | 외부 이벤트 수집·표시 경로 |
| S17 | Consent/approval/egress | DP | SEC-DP01 | 사용 시점 enforcement |
| S18 | Process/queue/scheduling/failure domains | DP | EXEC-DP01 | 실행 배치 |
| S19 | User Memory 사용·조회·수정·삭제 | REQUIRED | CTX-DP02, TASK-DP01, SEC-DP01 | UC-17 보존. 자체 QA trade-off가 확인되기 전 독립 DP를 추가하지 않음. |
| S20 | Restart 후 Conversation/Task 재연결 | REQUIRED | TASK-DP01, TASK-DP02, EXEC-DP01 | UC-18 보존. recovery time과 correctness를 구분. |
| S21 | Text/Voice 결과 구성·알림 | EMBEDDED | INT-DP01, TASK-DP02 | UI 렌더링 세부가 아니라 응답 경로와 queue 책임 |
| S22 | Telemetry / diagnostics | REQUIRED | AGENT-DP01, EXEC-DP01 | 모든 후보에 동일 관측 계약. 새 ASR로 승격하지 않음. |
| S23 | OS/package/framework/database 제품 선정 | IMPLEMENTATION | TASK-DP01, EXEC-DP01 | 본질적 계약/배치 차이일 때만 해당 DP에 반영 |
| S24 | Open-ended domain planning/tool execution | OUT_OF_SCOPE | 고정/범위 밖 | Downstream Agent의 책임 |
| S25 | Mobile/TV/Robot portability | OUT_OF_SCOPE | 고정/범위 밖 | 현재 PC 기준선 밖 |

## 2. RC-01~18 전수 연결

| 책임 점검 ID | 담당 DP |
|---|---|
| RC-01 음성 상호작용 | INT-DP01, EXEC-DP01 |
| RC-02 사용자 화면·알림 | EXEC-DP01 |
| RC-03 화면 interaction 수집 | CTX-DP01, EXEC-DP01 |
| RC-04 정보 조회·Context 제공 | CTX-DP01, ORCH-DP01, SEC-DP01 |
| RC-05 지칭 대상 해석 | CTX-DP01, IR-DP01 |
| RC-06 요청 정리·복합 관계 | IR-DP01, TASK-DP01 |
| RC-07 업무 관계 판단 | IR-DP01, CTX-DP02 |
| RC-08 요청 처리 조정 | INT-DP01, IR-DP01, ORCH-DP01 |
| RC-09 모델 연동 | INT-DP01, AGENT-DP01, SEC-DP01, EXEC-DP01 |
| RC-10 Agent capability·선택 | IR-DP01, AGENT-DP01 |
| RC-11 Agent 연결·계약 변환 | AGENT-DP01, TASK-DP02, SEC-DP01, EXEC-DP01 |
| RC-12 대화·요청 기록 | INT-DP01, CTX-DP02, TASK-DP01 |
| RC-13 Task 상태·제어 | TASK-DP01, TASK-DP02, EXEC-DP01 |
| RC-14 사용자 응답 구성 | INT-DP01, ORCH-DP01, TASK-DP02 |
| RC-15 정책·동의·승인 중계 | SEC-DP01 |
| RC-16 User Memory | CTX-DP02, TASK-DP01, SEC-DP01 |
| RC-17 저장·재연결·이행 | TASK-DP01, EXEC-DP01 |
| RC-18 설정·진단 기록 | EXEC-DP01 |

## 3. 10 DP × 12 W-ASR 전수 관찰 지도

| DP | W-01 | W-02 | W-03 | W-04 | W-05 | W-06 | W-07 | W-08 | W-09 | W-10 | W-11 | W-12 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| INT-DP01 | H | H | R | R | R | R | R | H | R | R | R | R |
| CTX-DP01 | H | R | R | R | H | R | R | H | R | R | H | R |
| IR-DP01 | H | H | R | R | H | R | R | H | R | R | R | R |
| CTX-DP02 | H | R | R | R | R | H | R | H | R | R | H | R |
| ORCH-DP01 | H | R | R | R | R | R | H | H | R | R | H | R |
| TASK-DP01 | R | H | R | H | R | R | R | H | H | R | R | R |
| AGENT-DP01 | R | H | H | R | R | R | H | H | R | R | R | R |
| TASK-DP02 | R | R | H | H | R | R | H | R | H | R | R | R |
| SEC-DP01 | R | H | R | R | R | R | R | H | R | R | H | H |
| EXEC-DP01 | H | R | R | H | R | R | R | R | H | H | R | R |

H가 있어도 실제 수치 차이가 입증된 것은 아니다. 특히 W-05/06은 모델 실행, W-10/12는 정상 후보의 동점 가능성을 유지한다. 모든 12개를 확인하며 최종 Primary와 발표 확대 축은 후속 evidence로 정한다.

## 4. 한 ASR당 한 row의 DP 인과 원장

| DP ID / Name | W-ASR ID + 전체 명칭 | 핵심 이유 |
|---|---|---|
| INT-DP01 Voice–Core Interaction Mediation | W-01 Conversational Reaction Responsiveness | Core 사전 왕복·동기화가 첫 응답 경로에 들어가는지 |
| INT-DP01 Voice–Core Interaction Mediation | W-02 Task Handoff Responsiveness | Core handoff 전 해석과 전달 단계가 어디서 중복되는지 |
| INT-DP01 Voice–Core Interaction Mediation | W-08 Evolvability & Maintainability | S2S 이벤트/대화 계약 변경이 다른 책임으로 얼마나 퍼지는지 |
| CTX-DP01 Context Access & Materialization | W-01 Conversational Reaction Responsiveness | 응답 전 자료 조회·직렬화·캐시 검증의 critical path |
| CTX-DP01 Context Access & Materialization | W-05 Task Completion Effectiveness | 참조 당시 자료와 현재 자료를 구분하고 필요한 evidence를 소비자에게 전달하는 정도 |
| CTX-DP01 Context Access & Materialization | W-08 Evolvability & Maintainability | Source/형식/API 변화의 소비자 파급 |
| CTX-DP01 Context Access & Materialization | W-11 Privacy Exposure Minimization | 전체 payload와 scoped 자료가 외부에 노출하는 정보 범위 |
| IR-DP01 Semantic Interpretation Pipeline | W-01 Conversational Reaction Responsiveness | 대화 응답까지 필요한 Model call graph의 길이 |
| IR-DP01 Semantic Interpretation Pipeline | W-02 Task Handoff Responsiveness | 위임 확정까지 의미 호출·검증의 직렬 경로 |
| IR-DP01 Semantic Interpretation Pipeline | W-05 Task Completion Effectiveness | 집중된 단계 판단의 이익과 중간 표현/오류 전파의 손익 |
| IR-DP01 Semantic Interpretation Pipeline | W-08 Evolvability & Maintainability | 의미 책임·출력 계약 변경의 국소성 |
| CTX-DP02 Conversation & Task Context Provisioning | W-01 Conversational Reaction Responsiveness | 이력 조회·요약·prompt prefill의 비용 |
| CTX-DP02 Conversation & Task Context Provisioning | W-06 Interaction & Task Continuity | 짧은 후속 지시·대화 전환에서 필요한 과거 정보의 보존 정도 |
| CTX-DP02 Conversation & Task Context Provisioning | W-08 Evolvability & Maintainability | 모델 이력 계약/저장 기록 변경의 소비자 영향 |
| CTX-DP02 Conversation & Task Context Provisioning | W-11 Privacy Exposure Minimization | 요약·선택 전달로 줄거나 새로 생기는 민감 정보 노출 |
| ORCH-DP01 Bounded Request Execution Placement | W-01 Conversational Reaction Responsiveness | 같은 bounded 답변을 얻는 전체 대기 경로의 차이 |
| ORCH-DP01 Bounded Request Execution Placement | W-07 Agent Ecosystem Interoperability & Substitutability | bounded 기능 Agent의 추가·교체가 VIA에 주는 변경 |
| ORCH-DP01 Bounded Request Execution Placement | W-08 Evolvability & Maintainability | Core 기능·Model·문서 형식 변화가 퍼지는 범위 |
| ORCH-DP01 Bounded Request Execution Placement | W-11 Privacy Exposure Minimization | 자료를 전달받는 실행 경계와 최소 context package |
| TASK-DP01 Task Supervision & Durable Recovery | W-02 Task Handoff Responsiveness | Agent 접수와 재연결 정보 보존까지의 조정 비용 |
| TASK-DP01 Task Supervision & Durable Recovery | W-04 Concurrent Task Performance Isolation | 동시 Task의 state contention과 작업별 진행 독립성 |
| TASK-DP01 Task Supervision & Durable Recovery | W-09 Recovery Timeliness & Recoverability | 올바른 상태·제어가 다시 가능해지는 복구 critical path |
| TASK-DP01 Task Supervision & Durable Recovery | W-08 Evolvability & Maintainability | 저장·correlation schema 변화의 변경 전파 |
| AGENT-DP01 Agent Contract Integration | W-07 Agent Ecosystem Interoperability & Substitutability | 9개 Agent 계약 변화가 gateway/adapter/소비 계약에 주는 변경 |
| AGENT-DP01 Agent Contract Integration | W-02 Task Handoff Responsiveness | 인계 시 변환·검증·접속 계층의 비용 |
| AGENT-DP01 Agent Contract Integration | W-03 Task Feedback Responsiveness | 이벤트·질문·결과 정규화의 경로 비용 |
| AGENT-DP01 Agent Contract Integration | W-08 Evolvability & Maintainability | 공유 연동 기반이 Model/Context 변화와 결합되는 정도 |
| TASK-DP02 Task Feedback Acquisition & Delivery | W-03 Task Feedback Responsiveness | 변화 발생부터 유효한 사용자 표시까지의 탐지·queue 지연 |
| TASK-DP02 Task Feedback Acquisition & Delivery | W-04 Concurrent Task Performance Isolation | polling·event fan-in이 전경 처리에 주는 간섭 |
| TASK-DP02 Task Feedback Acquisition & Delivery | W-07 Agent Ecosystem Interoperability & Substitutability | push/poll 계약 변화 및 결과 계약 변환 범위 |
| TASK-DP02 Task Feedback Acquisition & Delivery | W-09 Recovery Timeliness & Recoverability | 재연결 후 현재 상태/결과를 확보하는 시간 |
| SEC-DP01 Policy, Consent & Context Egress Enforcement | W-02 Task Handoff Responsiveness | 승인 binding과 use-time enforcement의 인계 지연 |
| SEC-DP01 Policy, Consent & Context Egress Enforcement | W-08 Evolvability & Maintainability | 공통/분산 enforcement의 정책·connector 변화 파급 |
| SEC-DP01 Policy, Consent & Context Egress Enforcement | W-11 Privacy Exposure Minimization | 허용된 범위 안에서도 과도한 context를 줄이는 mediation 지점 |
| SEC-DP01 Policy, Consent & Context Egress Enforcement | W-12 Action & Access Safety | 동일 24개 기회의 안전성 회귀. 모두 0이면 비변별 |
| EXEC-DP01 Runtime Scheduling & Fault-Isolation Boundary | W-01 Conversational Reaction Responsiveness | IPC/queue handoff와 전경 실행 경로의 비용 |
| EXEC-DP01 Runtime Scheduling & Fault-Isolation Boundary | W-04 Concurrent Task Performance Isolation | 공유 자원에서 1→4 Task 간섭이 증가하는 정도 |
| EXEC-DP01 Runtime Scheduling & Fault-Isolation Boundary | W-10 Dependency Failure Containment & Graceful Degradation | 한 endpoint failure가 무관 기능까지 전파되는 범위 |
| EXEC-DP01 Runtime Scheduling & Fault-Isolation Boundary | W-09 Recovery Timeliness & Recoverability | 복구해야 할 프로세스·연결·state의 범위 |

## 5. 검사 결과와 한계

로컬 도우미 검사는 10개 DP가 RC 18개, UC 18개, Change 24개를 적어도 한 번 연결하는지 확인한다. 25개 주제는 DP/병합/필수 기능/tactic/구현/범위 밖으로 모두 분류했다. 숫자 연결이 있다고 실제 기능 coverage가 입증되는 것은 아니며 Gate 2에서 계약·flow로 확인해야 한다.

UI framework, 특정 DB 제품, 모델 양자화 설정의 미세 조정은 독립 DP로 늘리지 않는다. 선택이 process/state/contract를 바꾸면 해당 owner DP의 candidate revision으로 올린다.