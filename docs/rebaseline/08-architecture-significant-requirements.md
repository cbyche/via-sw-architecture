# 8. Architecture Significant Requirements — QA 도출과 ASR 선정

> 상태: **QA 점수·7개 ASR 유지 / 사용자 합의에 따른 측정 정의 보정 · 11-A/B 검토 연결**
> 근거: [01 시스템 정의](./01-system-mission-and-boundary.md) · [02 용어](./02-terms.md) · [03 설계 범위](./03-fixed-architecture-scope.md) · [04 공통 흐름](./04-canonical-interaction-flow.md) · [05 UC](./05-representative-use-cases.md) · [06 비교 조건](./06-fixed-assumptions.md) · [07 변경 집합](./07-intentional-variables.md)

## 8.1 도출 절차

08은 다음 순서로 진행한다.

1. 01~07에서 Quality Attribute 후보를 도출한다.
2. 각 QA의 **Importance**와 **Architecture Difficulty**를 1~10점으로 평가한다.
3. 1~4=L, 5~7=M, 8~10=H로 변환한다.
4. **Importance=H && Difficulty=H인 QA를 ASR Candidate로 선정한다.**
5. 선정된 QA를 **ASR-01~ASR-N**으로 정의한다.
6. 각 ASR에는 대표 metric 하나를 고정한다.
7. grounding, task association, restart recovery 같은 구체 상황은 **ASR을 검증하는 Scenario/Test Case**로 내려간다.

즉 **QA와 ASR Scenario를 같은 레벨의 목록으로 섞지 않는다.**

---

## 8.2 점수 기준

### Importance — 제품 성공에 얼마나 중요한가

| 점수 | 기준 |
| ---: | --- |
| **1–2** | 현재 Mission/UC와 직접 관계가 거의 없음 |
| **3–4** | 현재 범위에서 영향이 제한적이며 있어도 보조적 |
| **5–7** | 여러 UC 또는 운영에 의미 있는 영향이 있으나 제품 핵심 정체성 전체를 좌우하지는 않음 |
| **8–9** | 핵심 UC 또는 전략적 제품 방향을 직접 좌우 |
| **10** | VIA의 존재 이유, 핵심 사용자 가치 또는 신뢰를 정의하는 수준 |

### Architecture Difficulty — SW 구조로 해결하기 얼마나 어려운가

| 점수 | 기준 |
| ---: | --- |
| **1–2** | 설정값·단일 함수·국소 구현으로 해결 가능 |
| **3–4** | 하나의 Component/Module 내부 설계가 중심 |
| **5–7** | 여러 Component/Interface를 연결하지만 state/lifecycle 범위가 비교적 한정 |
| **8–9** | 여러 lifecycle, state authority, runtime, external contract가 교차하며 책임 배치가 결과를 좌우 |
| **10** | 시스템의 핵심 identity/state/orchestration 구조와 여러 trade-off를 동시에 좌우 |

### L / M / H

- **1–4 = L**
- **5–7 = M**
- **8–10 = H**

Importance와 Difficulty는 독립적으로 판단하며 점수를 곱하지 않는다.

---

# 8.3 Quality Attribute Catalog — 한눈에 보기

| QA | Quality Attribute | Importance | Level | Difficulty | Level | ASR Candidate? | 핵심 근거 |
| --- | --- | ---: | :---: | ---: | :---: | :---: | --- |
| **QA-01** | **User-Experienced Responsiveness** | **9** | H | **9** | H | **Yes** | Voice/S2S, Direct Response, Agent 결과·interrupt까지 사용자 체감 반응성이 핵심. model/context/orchestration 경계가 latency에 영향 |
| **QA-02** | **Task Completion Effectiveness** | **10** | H | **9** | H | **Yes** | grounding, refinement, compound request, task association, Agent selection이 틀리면 사용자가 원하는 일을 완료할 수 없음 |
| **QA-03** | **Interaction & Task Continuity** | **10** | H | **10** | H | **Yes** | Direct↔Agent, Voice↔Text, Conversation↔Task, multiple task lifecycle 연속성이 VIA 핵심 |
| **QA-04** | **Agent Ecosystem Interoperability & Substitutability** | **9** | H | **9** | H | **Yes** | Agent-neutral이 고정 Architecture Scope이며 heterogeneous Agent/protocol/lifecycle 변화 대응 필요 |
| **QA-05** | **Evolvability & Maintainability** | **9** | H | **9** | H | **Yes** | Model, Context connector, persistent schema, deployment 변화가 07의 핵심 intentional variable이며 변경 전파를 구조적으로 제어해야 함 |
| **QA-06** | **Cross-Device Portability & Adaptability** | 3 | L | 8 | H | No | 현재 제품 범위는 PC/Windows reference. Mobile/TV/Robot은 명시적으로 제외 |
| **QA-07** | **Compute & Energy Efficiency** | 4 | L | 7 | M | No | target CPU/GPU/RAM, battery, power budget이 제품 constraint로 고정되지 않음 |
| **QA-08** | **Reliability & Recoverability** | **9** | H | **9** | H | **Yes** | async result, cancel, multiple task, failure, process restart 후 재연결이 필수 UC |
| **QA-09** | **Privacy, Security & Action Safety** | **10** | H | **9** | H | **Yes** | 개인 Context, 외부 Model/Agent 전달, Consent, Action Approval이 제품 신뢰와 직접 연결 |
| **QA-10** | **Diagnosability & Operability** | 6 | M | 7 | M | No | trace/evidence는 중요하지만 운영조직/SLO/MTTR 목표가 아직 제품 driver로 확정되지 않음 |

### H/H QA

따라서 ASR Candidate QA는 **7개**이다.

1. QA-01 User-Experienced Responsiveness
2. QA-02 Task Completion Effectiveness
3. QA-03 Interaction & Task Continuity
4. QA-04 Agent Ecosystem Interoperability & Substitutability
5. QA-05 Evolvability & Maintainability
6. QA-08 Reliability & Recoverability
7. QA-09 Privacy, Security & Action Safety

이 7개를 그대로 **ASR-01~ASR-07**로 정의한다.

---

# 8.4 Final ASR Catalog — 7개

| ASR | Parent QA | 정의 | 대표 Metric | 방향 |
| --- | --- | --- | --- | --- |
| **ASR-01 User-Experienced Responsiveness** | QA-01 | 사용자의 Voice/Text 요청, interruption, Agent 결과 전달에서 VIA가 추가하는 대기 시간을 최소화한다. | **User-Experienced Response Latency p95 (ms; VIA 모델 포함·Agent 업무시간 제외)** | 낮을수록 좋음 |
| **ASR-02 Task Completion Effectiveness** | QA-02 | VIA가 사용자 요청을 올바른 대상·요청 구조·Task·Agent에 연결하여 사용자가 의도한 처리 결과까지 도달하게 한다. | **Task completion success rate (%)** | 높을수록 좋음 |
| **ASR-03 Interaction & Task Continuity** | QA-03 | modality/connection 변화, Direct↔Agent 전환, multiple task 상황에서도 Conversation과 VIA Task의 의미와 identity를 연속적으로 유지한다. | **Continuity scenario pass rate (%)** | 높을수록 좋음 |
| **ASR-04 Agent Ecosystem Interoperability & Substitutability** | QA-04 | 서로 다른 Agent 구현·protocol·lifecycle 계약을 VIA 핵심 책임의 최소 변경으로 추가·교체·공존시킨다. | **Average changed architecture elements per Agent change (count/change)** | 낮을수록 좋음 |
| **ASR-05 Evolvability & Maintainability** | QA-05 | Model, Context integration, persistent-state schema 및 deployment 변화에 기존 기능을 유지하면서 제한된 구조 변경으로 대응한다. | **Average changed architecture elements per non-Agent change (count/change)** | 낮을수록 좋음 |
| **ASR-06 Reliability & Recoverability** | QA-08 | 비동기 event·실패·취소·process restart 상황에서도 Task state를 일관되게 유지하고 복구 가능한 업무를 중복 실행 없이 재연결한다. | **Reliability/recovery scenario pass rate (%)** | 높을수록 좋음 |
| **ASR-07 Privacy, Security & Action Safety** | QA-09 | Context 접근·외부 전달과 state-changing Action approval을 현재 policy/consent 및 올바른 pending Action에만 적용한다. | **Safety violation rate = 100×V/N (%); V건/N개 원자료 유지** | 낮을수록 좋음; 목표 방향 0 |

---

# 8.5 각 ASR의 범위와 검증 Scenario

아래 항목들은 **새 ASR이 아니다.** 각 ASR을 검증하는 scenario family이다.

## ASR-01 User-Experienced Responsiveness

포함할 대표 scenario:

- S2S Direct Response latency
- VIA Core Direct Response latency
- Agent delegation 전 VIA overhead
- Agent result 준비 후 사용자 전달 latency
- Voice interruption stop latency

대표 Metric은 **User-Experienced Response Latency p95 (VIA 모델 포함·Agent 업무시간 제외)**이다. 원격을 포함한 VIA 직접 LLM·S2S·Context·연결·출력 시간은 포함한다. 공개 profile와 실제 prompt token 기반 계산은 추정치로 기록한다. 음성 중단은 유형별 raw latency로 유지하고 일반 응답과 임의 혼합하지 않는다. 상세 측정 계약은 11-A를 따른다.

## ASR-02 Task Completion Effectiveness

포함할 대표 scenario:

- Interaction Grounding
- Referent Resolution
- Request Refinement
- Compound Request decomposition
- Compound Request relation preservation
- Task Association
- Agent Selection
- Clarification이 필요한 경우 올바르게 보류/확인
- Direct Response 또는 Agent Handling 경로에서 요구 결과에 도달

대표 Metric은 **Task completion success rate**이다.

여기서 success는 Downstream Agent의 domain 품질을 뜻하지 않는다. 고정 Agent fixture를 사용하고 VIA가 **올바른 요청·대상·제약·Task·Agent에 연결하여 05의 완료 조건을 충족했는지**를 평가한다.

## ASR-03 Interaction & Task Continuity

포함할 대표 scenario:

- Direct Response 후 follow-up
- Voice → Text / Text → Voice
- Voice Connection 종료·재연결
- Existing Task follow-up/status/correction
- multiple active Task 전환
- Agent run/thread가 달라도 동일 VIA Task identity 유지

대표 Metric은 **Continuity scenario pass rate**이다.

process restart는 ASR-06으로 분리한다.

## ASR-04 Agent Ecosystem Interoperability & Substitutability

07의 Agent change catalog **A-01~09 전체**를 적용한다.

대표 Metric:

**Average changed architecture elements per Agent change**

단, 필수 UC가 유지된 change만 유효한 값으로 인정한다. 기능을 유지하지 못한 채 변경 요소가 적은 후보를 좋은 결과로 보지 않는다.

## ASR-05 Evolvability & Maintainability

07의 **Model change M-01~09 + Context/Storage change C-01~06**을 적용한다.

대표 Metric:

**Average changed architecture elements per non-Agent change**

세부 결과는 Model / Context / Persistent State 분야별로도 보존한다. 그러나 ASR 대표 숫자는 하나로 유지한다.

Agent 변화는 ASR-04에서 별도로 평가하므로 중복하지 않는다.

## ASR-06 Reliability & Recoverability

포함할 대표 scenario:

- async progress/result/failure의 올바른 Task 반영
- cancel 요청과 실제 cancel 결과 구분
- out-of-order/duplicate Agent event
- 부분 실패 및 상태 미확인
- VIA process restart 후 Agent 실행 재연결
- 완료된 외부 실행의 결과 복원
- 외부 상태 확인 불가 시 중복 Action 방지

대표 Metric은 **Reliability/recovery scenario pass rate**이다.

## ASR-07 Privacy, Security & Action Safety

포함할 대표 scenario:

- Context 접근 권한 있음/없음
- 최초 Consent / 거부 / 철회
- 외부 Model/Agent Context egress 제한
- 여러 pending Action Approval
- approval/denial의 올바른 Action binding
- 이전 대화의 “응”을 다른 승인으로 재사용하지 않음

대표 Metric은 **Safety violation rate = 100×V/N**이며 위반 V건과 사전에 고정한 판단 기회 N개를 함께 보존한다. 내부 검사나 재시도로 분모를 늘리지 않는다. 모든 요청을 차단한 후보는 안전성을 충족한 정상 제품으로 인정하지 않는다.

다른 QA 점수로 safety violation을 상쇄하지 않는다.

---

# 8.6 왜 세부 Scenario를 ASR로 늘리지 않는가

예를 들어 Interaction Grounding과 Task Association은 서로 다른 실패 유형이다. 하지만 현재 방법론에서 둘은 **QA-02 Task Completion Effectiveness를 실현하기 위한 평가 Scenario**이다.

따라서:

```text
QA-02 Task Completion Effectiveness
        ↓ H/H
ASR-02 Task Completion Effectiveness
        ↓
Scenario Family
  - Grounding
  - Compound Request
  - Task Association
  - Agent Selection
        ↓
11 Test Cases
```

처럼 관리한다.

이렇게 해야:

- System-level ASR 개수가 QA 선정 결과와 일치하고
- 세부 failure mode를 잃지 않으며
- DP마다 별도의 ASR 정의를 새로 만들지 않고
- 같은 QA에 대해 같은 metric/target/score를 모든 DP에서 사용할 수 있다.

---

# 8.7 H/H가 아닌 QA 처리

| QA | 처리 |
| --- | --- |
| QA-06 Cross-Device Portability | 현재 Core ASR 제외. 제품 scope가 PC 밖으로 확장될 때 재평가 |
| QA-07 Compute & Energy Efficiency | secondary observation. 목표 HW/전력 constraint가 정해지면 재평가 |
| QA-10 Diagnosability & Operability | supporting requirement로 유지. trace/evidence는 모든 후보에 요구하지만 현재 독립 ASR score는 두지 않음 |

---

# 8.8 09~12로 넘길 것

## 09 — QA / ASR / UC / Change Mapping

7개 ASR 각각에 대해:

- Primary UC
- Supporting UC
- 07 Change Scenario
- Scenario Family

를 연결한다.

## 10 — Architecture Element Definition

ASR-04·05의 changed architecture element metric을 공정하게 계산할 element granularity를 확정한다.

## 11 — Test Case & Metric Freeze

7개 ASR 각각에 대해 **대표 metric 하나**를 고정한다.

그리고:

- test input
- scenario mix
- ground truth
- 반복 횟수
- target
- 0~5 score boundary

를 후보 결과 전에 동결한다.

**같은 ASR에는 모든 DP에서 같은 metric/target/scoring rule을 사용한다.**

## 12 — Architecture Decision Points

각 DP는 7개 ASR 중 **실제 인과관계가 큰 Primary ASR 약 3~4개**를 선택한다.

나머지 ASR은 regression constraint / secondary observation으로 유지한다.

---

# 8.9 이번 리뷰 순서

1. **8.3 QA 10개와 Importance/Difficulty 점수**
2. **H/H 7개 선정이 맞는지**
3. **ASR-01~07 정의가 해당 QA의 의미를 정확히 나타내는지**
4. **각 ASR의 representative metric 하나가 적절한지**
5. 세부 scenario family에 빠진 핵심 UC가 없는지

QA scoring이 바뀌면 ASR 목록도 바뀐다. 따라서 08은 이 다섯 항목이 승인된 뒤 닫는다.

## 8.10 기준선 확정 기록

사용자 승인에 따라 QA 10개의 점수, H/H QA 7개와 ASR-01~07의 일대일 대응, 정의 및 대표 지표를 확정한다. 세부 시나리오는 추가 ASR이 아니다. 이번 확정은 11의 실제 시험 입력·표본 구성·반복·목표값·0~5점 구간까지 승인한 것이 아니다.

다음 검토 범위는 [09 연결표](./09-asr-uc-change-mapping.md)와 [10 설계 요소·집계 기준](./10-architecture-element-definition.md)이다. 두 문서의 공동 리뷰 전에는 11·12를 진행하지 않는다.

## 8.11 측정 보정과 11-A/B 연결

사용자 합의에 따라 모델 비용을 실제로 수치화하는 ASR-01 근거 원장과 ASR-07의 고정 분모를 보완한다. ASR-02·03·06은 우선 전체 raw pass/fail을 보존하며 macro/단순 평균을 지금 바꾸지 않는다. ASR-04·05는 10의 변경 요소 집계와 9/15개 전체 변경을 유지한다.

[11-A 측정 기준](./11a-measurement-baseline.md), [11-B 시험 목록](./11b-test-case-catalog.md), [Qwen 근거](./11-evidence/asr01-qwen-evidence.md)를 검토한다. QA 점수와 7개 ASR 선정은 다시 변경하지 않는다. 목표·0~5점·가중치·대표 표본 집합은 11-C에서 별도 승인한다.
