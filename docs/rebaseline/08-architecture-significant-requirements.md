# 8. Architecture Significant Requirements — Atomic ASR Catalog

> 상태: **재작성 검토본 — ASR 정의와 선정 근거 검토 필요**  
> 근거: [01 시스템 정의](./01-system-mission-and-boundary.md) · [02 용어](./02-terms.md) · [03 설계 범위](./03-fixed-architecture-scope.md) · [04 공통 흐름](./04-canonical-interaction-flow.md) · [05 UC](./05-representative-use-cases.md) · [06 비교 조건](./06-fixed-assumptions.md) · [07 변경 집합](./07-intentional-variables.md)  
> 07 변경 집합: M-01~09, A-01~09, C-01~06 — 총 24개

## 8.1 재작성 원칙

ASR은 “중요해 보이는 QA 이름”이 아니라 **Architecture 구조를 실제로 좌우하는 구체적인 품질 요구**이다.

이번 재작성에서는 다음 원칙을 고정한다.

1. **하나의 ASR에는 하나의 주요 품질 concern만 둔다.**
2. **하나의 ASR에는 하나의 stimulus class와 하나의 observable response를 둔다.**
3. **하나의 ASR에는 representative metric 하나만 둔다.**
4. 서로 다른 failure mode를 “연속성”, “정합성”, “안전성” 같은 넓은 이름 아래 묶지 않는다.
5. 필수 기능이라는 이유만으로 모두 ASR로 올리지 않고, **중요도 H + Architecture 난이도 H**인 항목을 선정한다.
6. 측정하기 쉽다는 이유로 ASR이 되는 것도 금지한다.
7. ASR 전체 개수를 작게 맞추지 않는다. 대신 **각 Architecture Decision Point에서는 실제 인과관계가 큰 ASR 약 3~4개만 Primary Driver로 선택**한다.

### Atomicity Check

다음 질문 중 하나라도 두 가지 이상의 서로 다른 답을 가지면 ASR을 분리한다.

- 무엇이 발생했을 때 평가하는가?
- 시스템이 무엇을 해야 하는가?
- 무엇이 틀리면 실패인가?
- 대표 숫자는 무엇인가?
- 어떤 Architecture 책임을 바꾸는가?

예를 들어 기존의 **“제어·상태·결과 정합성”**은 하나의 ASR이 아니다.

- 사용자 cancel이 올바른 Task로 갔는가?
- Agent progress/result가 올바른 Task로 들어왔는가?
- 사용자에게 실제보다 앞선 상태를 말하지 않았는가?

는 서로 다른 failure mode이므로 별도 ASR로 분리한다.

---

## 8.2 기존 8.4 H/H 후보를 다시 분해한 결과

기존 8.4에서 H/H였던 11개 덩어리를 그대로 ASR로 쓰지 않는다.

| 기존 concern | 재검토 결과 |
| --- | --- |
| VIA 처리 지연 | **ASR-01**로 유지 |
| 화면 지칭 정확도 | **ASR-03**으로 유지 |
| 업무 연결 정확도 | **ASR-04**로 유지 |
| 모델 변경 대응성 | 제공자 교체 / 배치 이동 / interaction contract 변경이 다른 변화이므로 **ASR-16~18로 분리** |
| Agent 변경 대응성 | Agent 추가·교체 / protocol 변화 / lifecycle contract 변화가 다르므로 **ASR-19~21로 분리** |
| 복합 요청 의미와 관계 보존 | Request 분해와 Request 간 관계 보존은 다른 오류이므로 **ASR-05·06으로 분리** |
| 대화·업무 연속성 | Conversation continuity와 Task identity continuity는 lifecycle이 다르므로 **ASR-08·09로 분리** |
| 재시작 복구 | **ASR-13**으로 유지 |
| 권한·Context 전달·승인 안전성 | Context authorization과 Action approval binding은 다른 안전 문제이므로 **ASR-14·15로 분리** |
| 제어·상태·결과 정합성 | Agent event binding / 사용자 control binding / user-visible state truthfulness로 **ASR-10~12 분리** |
| 음성 중단 반응성 | **ASR-02**로 독립 |

또한 기존 표에서 다른 concern과 섞여 중요도가 낮아 보였던 항목도 다시 본다.

| 기존 처리 | 재평가 |
| --- | --- |
| “의도 정리·Agent 선택”을 하나의 H/M 항목으로 둠 | Intent refinement 모델 품질과 Agent selection architecture를 분리. **Agent Selection은 Agent-neutral 핵심 책임이므로 H/H → ASR-07** |
| “Context·저장 기록 변경 대응성”을 M/H로 묶음 | Context connector 변경과 persistent-state schema 변경은 영향 대상과 복구 방식이 다름. 05의 Context/Memory/Recovery 핵심 기능과 직접 연결되므로 **각각 H/H → ASR-22·23** |

즉 이전처럼 다섯 개 ASR만 먼저 정하고 나머지를 필수 요구로 밀어내지 않는다.

---

## 8.3 ASR 선정 기준

### Importance = H

다음 중 하나 이상이면 H로 판단한다.

- 실패 시 05의 핵심 UC가 직접 깨진다.
- 잘못된 대상·업무·승인·상태를 사용자에게 적용하여 신뢰 또는 안전 문제가 발생한다.
- 07에서 의도적으로 지원하기로 한 생태계 변화의 핵심 목적이 무너진다.

### Architecture Difficulty = H

다음 중 하나 이상이면 H로 판단한다.

- 두 개 이상의 lifecycle 또는 identity 경계를 함께 다뤄야 한다.
- Component 하나의 local logic만이 아니라 책임·interface·state ownership·runtime boundary를 정해야 한다.
- 재시작·비동기 event·외부 protocol 변화처럼 시간적으로 분리된 상태를 다시 연결해야 한다.
- 후보 Architecture의 책임 배치에 따라 변경 영향이나 검증 결과가 달라질 수 있다.

모델 자체의 지능이 어렵다는 이유만으로 H를 주지 않는다. 반대로 semantic model이 영향을 준다고 해서 evidence/state/interface 구조의 Architecture 난이도를 무시하지 않는다.

---

# 8.4 Final ASR Catalog

## A. Responsiveness

| ID | 정확한 요구 | Representative Metric | 주요 근거 |
| --- | --- | --- | --- |
| **ASR-01 VIA Response Latency** | 사용자 입력이 끝난 뒤, Agent 내부 업무 시간을 제외한 VIA 책임 구간에서 유효한 사용자 응답이 시작될 때까지의 지연을 낮춘다. | **VIA response latency p95 (ms)** | UC-01~15, FA-12 |
| **ASR-02 Voice Interruption Stop Latency** | 사용자가 VIA 음성을 끊는 새 발화를 시작하면 진행 중 Voice Response 재생을 빠르게 중단한다. | **interrupt onset → audio stop p95 (ms)** | UC-11 |

ASR-01은 일반 요청/결과 응답 지연이고, ASR-02는 **이미 재생 중인 음성을 멈추는 latency**이다. 두 값을 합치지 않는다.

---

## B. Semantic Binding Correctness

| ID | 정확한 요구 | Representative Metric | 주요 근거 |
| --- | --- | --- | --- |
| **ASR-03 Interaction Grounding Accuracy** | 현재 화면 interaction을 이용한 지칭 표현을 사용자가 지정한 실제 on-screen 대상·집합·영역에 연결한다. | **grounding exact-match test pass rate (%)** | UC-03·04 |
| **ASR-04 Task Association Accuracy** | 현재 VIA Request가 No Tracked / New / Existing 중 무엇인지 판단하고, Existing이면 의도한 VIA Task를 식별한다. | **task-association exact-match pass rate (%)** | UC-07·10·14·15 |
| **ASR-05 Compound Request Decomposition Accuracy** | 하나의 User Turn에 포함된 의미상 VIA Request를 누락·중복·불필요한 분할 없이 식별한다. | **request-set exact-match pass rate (%)** | UC-09·14 |
| **ASR-06 Compound Request Relation Accuracy** | 분해된 VIA Request 사이의 independent / sequential / data-dependent / conditional 관계를 사용자 의도대로 보존한다. | **request-relation graph exact-match pass rate (%)** | UC-09 |
| **ASR-07 Agent Selection Correctness** | Agent 위임이 필요한 VIA Task에 대해 요구 capability·lifecycle·policy 조건을 만족하는 Downstream Agent를 선택한다. | **valid-agent-selection pass rate (%)** | UC-08~10·14·16 |

### 경계

- ASR-03은 **화면상의 Referent**만 다룬다. 파일 검색이나 과거 대화 Referent 전체를 하나의 grounding 정확도에 넣지 않는다.
- ASR-04는 **Task relation/identity**만 다룬다. cancel이 실제로 성공했는지는 포함하지 않는다.
- ASR-05는 “몇 개 Request인가”, ASR-06은 “그 Request들이 어떻게 연결되는가”이다.
- ASR-07은 Agent의 업무 결과 품질을 평가하지 않는다. **선택 시점의 declared capability/contract가 요구를 만족하는지**를 평가한다.

---

## C. Conversation / Task Continuity and Correlation

| ID | 정확한 요구 | Representative Metric | 주요 근거 |
| --- | --- | --- | --- |
| **ASR-08 Conversation Continuity Correctness** | Direct Response, Voice/Text 전환, Voice 재연결 뒤에도 후속 User Turn이 의도한 이전 대화 내용·Referent를 계속 참조할 수 있다. | **conversation-continuity scenario pass rate (%)** | UC-01·05~07·15 |
| **ASR-09 Task Identity Continuity Correctness** | 같은 사용자 업무 목표에 대한 follow-up·status·correction에서 Agent run/thread 변화와 무관하게 동일 VIA Task identity를 유지한다. | **task-identity continuity scenario pass rate (%)** | UC-10·13~15 |
| **ASR-10 Async Agent Event Binding Correctness** | Agent의 progress / clarification / result / failure event를 정확한 VIA Task와 Agent Execution에 연결한다. | **agent-event binding exact-match pass rate (%)** | UC-13·14·18 |
| **ASR-11 User Control Binding Correctness** | follow-up / correction / cancel 등 사용자 control 요청을 의도한 VIA Task와 Agent Execution에 전달하고 다른 업무에는 적용하지 않는다. | **control binding exact-match pass rate (%)** | UC-10~12·14 |
| **ASR-12 User-visible Task State Truthfulness** | Agent의 접수·진행·완료·취소·실패 상태를 확인된 사실보다 앞서거나 다르게 사용자에게 보고하지 않는다. | **truthful-state reporting pass rate (%)** | UC-10·12~14·18 |
| **ASR-13 Restart Recovery Correctness** | VIA process restart 후 복구 가능한 조건에서는 Conversation/Task/Agent Execution 관계를 재연결하고, 상태 불명 조건에서는 중복 state-changing Action 없이 불확실성을 알린다. | **FA-14 recovery scenario pass rate (%)** | UC-18, FA-14 |

### 경계

- ASR-08은 **대화 의미 continuity**, ASR-09는 **업무 identity continuity**이다.
- ASR-10은 Agent → VIA 방향의 event correlation이다.
- ASR-11은 사용자 → Agent 방향의 control correlation이다.
- ASR-12는 correlation이 맞더라도 상태를 과장해 보고하는 별도 failure mode를 잡는다.
- ASR-13은 정상 실행 중 continuity가 아니라 **process memory가 사라진 뒤 복구**를 다룬다.

---

## D. Safety / Authorization

| ID | 정확한 요구 | Representative Metric | 주요 근거 |
| --- | --- | --- | --- |
| **ASR-14 Context Authorization Enforcement** | Context read 또는 외부 Model/Agent로의 Context 전달은 현재 유효한 policy/consent 범위 안에서만 수행한다. | **unauthorized context access/egress count (건)** — 목표 방향 0 | UC-16, FA-15 |
| **ASR-15 Action Approval Binding Correctness** | 사용자의 approval/denial 응답을 정확한 pending Action과 VIA Task에 연결하며 다른 Action에 재사용하지 않는다. | **approval misbinding count (건)** — 목표 방향 0 | UC-16, A-08 |

ASR-14는 **정보 접근·전달 권한**, ASR-15는 **특정 Action 승인 응답의 binding**이다. 한 “Safety” 점수로 합치지 않는다.

---

## E. Model Ecosystem Evolvability

07의 Model change 9개를 하나의 “Model 변경 대응성” 숫자로 합치지 않는다. 변화 종류가 다른 Architecture 결합을 건드리기 때문이다.

| ID | 정확한 변화 요구 | Representative Metric | 적용 Change |
| --- | --- | --- | --- |
| **ASR-16 Model Runtime Substitutability** | 같은 역할·배치 조건에서 S2S 또는 Semantic Model Runtime 제공자를 교체해도 기존 기능을 유지하면서 구조 변경 범위를 제한한다. | **changed architecture elements / model substitution (개)** | M-01·M-02 |
| **ASR-17 Model Deployment Portability** | 같은 역할의 Model Runtime을 Cloud / Private Cloud / user PC 사이에서 이동해도 기존 기능을 유지하면서 구조 변경 범위를 제한한다. | **average changed architecture elements / deployment move (개)** | M-04~06 |
| **ASR-18 Model Interaction Contract Adaptability** | S2S event 또는 semantic-model response lifecycle 계약이 바뀌어도 기존 기능을 유지하면서 영향 범위를 제한한다. | **average changed architecture elements / contract change (개)** | M-07~09 |

M-03의 모델 크기·입력 한도·프로필 변화는 **regression change scenario**로 계속 평가하지만 독립 ASR로 두지 않는다. 기능 적합성 자체가 달라질 수 있고, “작은 모델일수록 좋은 Architecture” 같은 잘못된 결론을 만들 수 있기 때문이다.

---

## F. Agent Ecosystem Evolvability

| ID | 정확한 변화 요구 | Representative Metric | 적용 Change |
| --- | --- | --- | --- |
| **ASR-19 Agent Add/Replace Impact** | 기존 protocol 계열에서 새 Agent를 추가하거나 동일 업무 Agent를 교체할 때 기존 사용자 기능을 유지하면서 구조 변경 범위를 제한한다. | **average changed architecture elements / add-or-replace (개)** | A-01·02 |
| **ASR-20 Agent Protocol Adaptability** | 기존 protocol과 다른 protocol의 Agent를 추가하여 공존시킬 때 VIA 핵심 책임의 변경 범위를 제한한다. | **changed architecture elements for A-03 (개)** | A-03 |
| **ASR-21 Agent Lifecycle Contract Adaptability** | 상태 제공, 실행 identity, 질문/승인 응답, 결과 전달 lifecycle 계약이 변경되어도 Task continuity와 user-facing interaction을 유지하면서 영향 범위를 제한한다. | **average changed architecture elements / lifecycle-contract change (개)** | A-04·05·08·09 |

A-06 capability schema 확장과 A-07 authentication contract 변경은 전체 change catalog에서 계속 평가한다. 현재는 각각 capability metadata evolution과 인증 integration이라는 좁은 변화로, 별도 system-level ASR보다는 ASR-07·14 및 Agent regression analysis의 근거로 유지한다.

---

## G. Context / Persistent-State Evolvability

| ID | 정확한 변화 요구 | Representative Metric | 적용 Change |
| --- | --- | --- | --- |
| **ASR-22 Context Connector Adaptability** | 기존 Context 종류의 제공자를 교체·추가하거나 화면 integration contract가 바뀌어도 Context 의미와 Referent 기능을 유지하면서 변경 범위를 제한한다. | **average changed architecture elements / connector change (개)** | C-01·03·05 |
| **ASR-23 Persistent-State Schema Evolvability** | User Memory 또는 Conversation/Task persistent record schema가 바뀌어도 기존 데이터 의미·삭제 상태·Task recovery 관계를 유지하면서 변경 범위를 제한한다. | **average changed architecture elements / state-schema change (개)** | C-04·06 |

C-02 문서 형식 추가는 계속 change catalog에서 회귀 평가하지만, 현재는 parser/format extension 성격이 더 커 별도 system-level ASR로 승격하지 않는다.

---

# 8.5 왜 23개가 너무 많은 ASR이 아닌가

23개는 **23개의 최종 Architecture Decision**이나 **23개의 점수 가중치**를 뜻하지 않는다.

ASR catalog는 시스템에 Architecture significance가 있는 atomic concern의 목록이다.

~~~text
ASR Catalog 23개
        ↓
09: UC / Change와 trace
        ↓
12의 각 DP
        ↓
그 DP가 실제로 크게 바꾸는 Primary ASR 3~4개 선택
        ↓
나머지는 regression constraint / secondary observation
~~~

예를 들어 Agent integration DP라면 ASR-19~21, ASR-09~11 중 일부가 Primary가 될 수 있고, Interaction Grounding DP라면 ASR-03, ASR-01, ASR-18 등이 Primary가 될 수 있다.

모든 DP에 23개를 동시에 점수화하지 않는다.

---

# 8.6 공통 Correctness Metric 규칙

ASR-03~13 중 correctness/pass-rate 계열은 **각 ASR마다 다른 ground truth를 사용하되 동일한 집계 원칙**을 적용한다.

~~~text
pass rate
= 100 × 모든 필수 조건을 만족한 시험 수 / 고정된 해당 ASR 시험 수
~~~

- 한 시험 안에서 해당 ASR의 정답 조건을 하나라도 틀리면 fail이다.
- 본질적으로 모호하여 clarification이 정답인 시험은 “정확히 하나를 추측”하는 것을 성공으로 세지 않는다.
- 후보 결과를 본 뒤 허용 정답을 늘리지 않는다.
- 다른 ASR의 실패를 중복 계산하지 않도록 Test Case에서 주 채점 ASR을 지정한다.
- 통합 UC에서는 여러 ASR을 동시에 관찰할 수 있으나, 어느 failure가 어느 ASR에 속하는지 기록한다.

ASR-03·04·05·06·07은 semantic Model의 영향을 받을 수 있다. 같은 정보와 같은 모델을 사용하면 후보가 동점일 수 있으며 **동점은 정상 결과**이다. Architecture가 정확도를 바꾼다고 주장하려면 evidence availability, identity/state access 또는 inference responsibility의 차이를 설명해야 한다.

---

# 8.7 Continuity / Correlation ASR 검증 규칙

ASR-08~13은 단순 자연어 답변 정확도와 구분한다.

예:

- ASR-10: Agent result event의 Task binding이 맞는가?
- ASR-12: 완료 event를 아직 받지 않았는데 “완료되었습니다”라고 말했는가?
- ASR-13: restart 뒤 동일한 외부 실행을 새 Task로 중복 시작했는가?

이 영역은 **state identity, authoritative state, event correlation, persistence/recovery contract**의 Architecture 영향을 직접 확인한다.

고정된 Agent fixture와 event timeline을 사용하며 Agent 자체 업무 품질은 평가하지 않는다.

---

# 8.8 Safety ASR 검증 규칙

ASR-14·15는 다른 QA 장점으로 상쇄하지 않는다.

- unauthorized access/egress 1건을 낮은 latency로 보상하지 않는다.
- approval misbinding 1건을 높은 grounding accuracy로 보상하지 않는다.
- 거부·철회·만료·여러 pending approval을 포함한 고정 scenario를 사용한다.
- 시험기가 미리 승인된 Context나 Action ID를 후보에게 몰래 제공하지 않는다.

최종 0~5점 환산을 하더라도 **안전 위반을 허용하는 목표값을 임의로 만들지 않는다.** 목표/점수 규칙은 11에서 별도 승인한다.

---

# 8.9 Evolvability ASR의 변경 요소 측정 규칙

ASR-16~23은 06 FA-16의 동일 element catalog를 사용한다.

~~~text
한 change scenario의 변경 요소 수
= 수정 ∪ 추가 ∪ 제거 architecture element ID의 중복 없는 개수
~~~

Architecture element 유형은 10에서 고정한다.

- Component / Responsibility
- Interface / Contract
- State / Data Schema
- Runtime / Deployment Unit

다음을 지킨다.

- source file, function, crate 수를 세지 않는다.
- 새 adapter 추가를 “기존 수정 없음”이라는 이유로 0으로 세지 않는다.
- 단순 endpoint/config 값 변경으로 기능이 유지되면 0개가 가능하다.
- 기능 유지가 불가능하면 작은 변경값으로 성공 처리하지 않는다.
- 각 change는 동일한 baseline Architecture에서 독립적으로 적용한다.
- 변경 요소 수는 development M/M이 아니다.

ASR-16~23을 하나의 “Evolvability 총점”으로 합치지 않는다. 필요하면 해당 DP가 실제로 영향을 주는 ASR만 Primary Driver로 사용한다.

---

# 8.10 H/H이지만 별도 ASR로 만들지 않은 항목

| Concern | 처리 |
| --- | --- |
| Model profile/size/input limit M-03 | 기능 적합성이 먼저 달라질 수 있으므로 change regression으로 유지 |
| Agent capability schema A-06 | Agent Selection ASR-07과 Agent change regression에서 확인 |
| Agent authentication contract A-07 | Context Authorization ASR-14와 Agent change regression에서 확인 |
| Document format addition C-02 | Context regression으로 유지. 현재 Architecture 전반을 좌우하는 독립 driver로는 부족 |
| User Memory 내용 자체의 “지능적 정확도” | 저장·확인·수정·삭제는 필수 UC로 검증. 별도 memory reasoning 연구 ASR로 확대하지 않음 |
| Resource / cost / max throughput | target hardware·budget·capacity 목표가 아직 제품 제약으로 고정되지 않아 Core ASR로 승격하지 않음 |
| Diagnosability / operability | 모든 후보의 trace/evidence requirement로 유지하되 현재 독립 ASR을 만들 실측 운영 조건이 부족 |

이들은 누락이 아니라 **ASR 선정 기준에서 의도적으로 제외한 항목**이다.

---

# 8.11 09~12로 넘길 것

## 09 — ASR Traceability

각 ASR에 대해:

- Primary UC
- Regression UC
- 07 Change Scenario
- 실패 시 사용자 영향

을 연결한다.

ASR 하나가 너무 많은 unrelated UC를 참조하면 definition이 다시 넓어졌는지 확인한다.

## 10 — Architecture Element Definition

ASR-16~23의 변경 요소 수를 공정하게 세기 위해 element granularity를 고정한다.

또한 ASR-08~13의 state ownership과 correlation boundary를 설명할 수 있는 수준으로 element 책임을 정의한다.

## 11 — Test Case Catalog / Metric Freeze

각 ASR마다:

- exact test input
- ground truth
- 반복 횟수
- metric 계산법
- 목표값
- 0~5 score boundary

를 **후보 최종 결과를 보기 전에** 동결한다.

한 ASR의 metric은 모든 DP에서 동일하게 사용한다.

## 12 — Architecture Decision Points

각 DP마다:

- 후보 구조
- **Primary ASR 약 3~4개**
- regression ASR
- UC/Change evidence
- trade-off

를 비교한다.

DP마다 유리한 ASR 정의나 metric으로 바꾸지 않는다.

---

# 8.12 이번 리뷰에서 확인할 핵심

1. **ASR-01~23 각각이 한 concern만 다루는가?**
2. 기존에 한 덩어리였던 continuity / state / safety / evolvability가 충분히 분리되었는가?
3. H/H가 아닌 항목을 억지로 ASR로 올리거나, H/H를 측정 편의 때문에 제외하지 않았는가?
4. 각 metric이 그 ASR의 failure만 측정하고 다른 ASR의 실패를 몰래 합산하지 않는가?
5. 이후 DP가 이 catalog에서 실제 Primary ASR 3~4개만 선택할 수 있을 정도로 정의가 명확한가?

현재 문서는 **ASR catalog와 metric 방향을 확정하기 위한 검토본**이다. 목표값과 0~5점 경계는 아직 정하지 않았으며 11에서 근거와 함께 동결한다.
