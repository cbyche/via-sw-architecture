# VIA 핵심 Architecture Decision 후보

> 상태: **USER REVIEW DRAFT**
>
> 이 문서는 VIA 전체를 관통하는 Architecture Decision Point 후보를 처음부터 도출한 결과다. 아직 최종 DP 목록, Architecture Decision, 구현 계획 또는 측정 결과가 아니다. 기존 ADR도 이 문서만으로 변경되지 않는다.

## 1. 이 문서가 답하려는 질문

VIA는 Voice·Text·화면 interaction을 하나의 대화로 이어가고, 직접 답하거나 Downstream Agent에 업무를 맡긴 뒤 진행 상황과 결과를 다시 사용자에게 연결한다.

이 시스템을 설계할 때 먼저 답해야 하는 질문은 다음과 같다.

> VIA가 직접 책임질 범위는 어디까지인가?
>
> 대화와 업무 상태의 진실은 누가 소유하는가?
>
> Voice Runtime, Model, Agent와 Core 사이의 권한과 계약은 어디에 두는가?
>
> 장애 후 무엇을 근거로 복구하고, 실행 결과를 어떻게 다시 검증할 수 있는가?

이 질문을 서로 독립적으로 비교할 수 있는 구조 축으로 나눈 결과가 이 문서의 12개 후보다. 첫 상세 검토인 [DP-01 보고서](./via-dp-01-direct-handling.md)에서는 구조 차이는 확인했지만 충분한 QA trade-off는 입증하지 못했다. 아래 목록은 계속 후보이며, 모든 항목이 선정 기준을 통과했다는 뜻이 아니다.

문서는 다음 순서로 읽으면 된다.

1. 용어와 후보 선정 기준을 이해한다.
2. 12개 후보의 전체 지도를 본다.
3. 각 후보의 질문과 A/B trade-off를 순서대로 읽는다.
4. 마지막에 QA가 빠짐없이 연결되는지, 후보가 서로 중복되지 않는지, 기존 DP와 어떤 관계인지 확인한다.

## 2. 이 문서에서 사용하는 용어

### VIA 시스템 용어

이 문서만 읽어도 후보를 이해할 수 있도록 꼭 필요한 시스템 용어만 먼저 정리한다.

| 용어 | 뜻 |
| --- | --- |
| **VIA Core** | Voice Runtime을 제외한 VIA의 요청 이해, Context 사용, 대화·업무 상태와 Agent orchestration 책임 영역. 하나의 Component나 Process를 미리 뜻하지 않는다. |
| **Voice Runtime** | 음성 입력·출력, Voice Connection, 음성 중단과 S2S 연결을 관리하는 VIA 내부 책임 영역 |
| **S2S Model** | 음성을 입력받아 음성으로 응답할 수 있는 기본 Voice Model dependency |
| **Model Runtime** | Model 실행, stream, 연결 수명, health와 local 자원을 관리하는 실행 환경 |
| **Context** | 화면·선택·pointer, 대화 기록, Task 상태, 파일·메일·일정, 공개 정보와 허용된 User Memory 등 요청 이해에 필요한 정보 |
| **VIA Request** | 한 User Turn에서 분리한 하나의 논리적 처리 단위 |
| **VIA Task** | 여러 Request에 걸쳐 계속 상태를 추적해야 하는 사용자 업무 목표 |
| **Agent Execution** | Downstream Agent가 실제 업무를 수행하는 한 번의 실행 |
| **Downstream Agent** | 업무 reasoning, planning, Tool 선택·실행과 외부 상태 변경을 담당하는 VIA 외부 실행 주체 |
| **범위가 정해진 정보 처리** | 대상과 자료가 미리 정해진 조회·검색·내용 이해. 스스로 조사 범위나 업무 계획을 만들지 않는다. |

### Decision Point와 대안

- **Decision Point(DP)**: 책임, 권한, 상태 소유권, 계약 또는 Process 경계를 어디에 둘지 결정하는 Architecture 질문이다.
- **대안 A/B**: 같은 사용자 기능을 제공하지만 내부 구조가 다른 두 가지 정상적인 설계다.
- **상호 배타적(mutually exclusive)**: 같은 결정 범위에서 A와 B의 최종 결정 규칙을 동시에 채택할 수 없다는 뜻이다.
- **Steelman**: 같은 조건에서 각 대안을 실제로 채택할 만한 최선의 설계로 구성한다는 뜻이다. 두 대안은 서로에게 steelman이 되어야 한다.
- **QA trade-off**: A와 B 중 어느 한쪽이 모든 면에서 항상 좋은 것이 아니라, 서로 다른 Quality Attribute에서 장단점이 생기는 관계다.

### 후보 ID

이 문서에서는 `VIA-DP-01`부터 `VIA-DP-12`까지의 임시 ID를 사용한다.

- `VIA-DP`는 **VIA 전체에 영향을 주는 Architecture Decision Point**라는 뜻이다.
- 번호는 이 문서에서 논의할 순서다.
- 문서 전체가 검토 초안이므로, 이 ID 역시 아직 승인된 최종 ID가 아니다.
- 최종 후보가 합쳐지거나 제외되어도 검토 이력을 이해할 수 있도록 이 문서 안에서는 번호를 유지한다.

### Architecture가 달라진다는 의미

후보 검토에서는 다음 네 종류 중 무엇이 달라지는지 확인한다.

| 종류 | 이 문서에서의 의미 | 예 |
| --- | --- | --- |
| **Component** | 독립된 책임과 변경 이유를 가진 실행 단위 | Context Broker, Task State Owner |
| **Interface** | Component 또는 외부 dependency 사이의 계약 | Agent Observation, Context Package |
| **State** | 별도 수명·복구·갱신 규칙을 가진 정보 | Task revision, pending approval |
| **Deployment** | Process, device 또는 local/remote 실행 경계 | Core process, integration worker |

단순히 함수나 class 이름이 달라지는 것은 Architecture 차이로 세지 않는다.

### 후보 설명에 나오는 기술어

| 표현 | 이 문서에서의 뜻 |
| --- | --- |
| **단일 변경 권한자(single writer)** | 특정 상태를 최종적으로 바꿀 수 있는 주체가 하나뿐이라는 뜻 |
| **일관성 경계** | 여러 상태 변경을 전부 성공시키거나 전부 취소할 수 있는 범위 |
| **불변(immutable) 기록** | 만든 뒤 내용을 덮어쓰지 않고, 변경이 필요하면 새 version을 만드는 기록 |
| **출처 정보(provenance)** | 어떤 정보가 어디에서 언제 어떤 version으로 왔는지를 나타내는 정보 |
| **이벤트 이력(event journal)** | 상태를 직접 덮어쓰는 대신, 상태를 바꾼 사건을 순서대로 계속 추가한 기록 |
| **현재 상태 보기(projection)** | 이벤트 이력을 읽어 현재 상태 형태로 계산한 결과 |
| **outbox/inbox** | 외부 요청이나 응답을 잃거나 중복 처리하지 않도록 저장해 두는 송·수신 기록 |
| **멱등 식별자(idempotency identity)** | 같은 요청을 재시도해도 외부 동작이 한 번만 일어나게 하는 요청 식별자 |
| **Gateway** | 여러 Component가 외부 dependency를 같은 계약으로 사용하게 만드는 공통 연결 지점 |
| **평가 근거 영역(Evidence Plane)** | 실행 기록과 평가 근거를 수집·연결·보관하는 공통 책임 영역 |

## 3. Core DP 후보를 남기는 기준

후보는 다음 조건을 모두 만족해야 한다.

1. 책임, 권한, 상태 소유권, 계약, dependency 방향 또는 Process/Deployment 경계를 바꾼다.
2. A와 B가 같은 사용자 기능을 제공할 수 있다.
3. Component·Interface·State·Deployment 중 둘 이상이 달라진다.
4. 둘 이상의 QA에 직접 영향을 줄 수 있다.
5. A와 B 각각에 유리할 수 있는 QA가 있어 한쪽이 정의상 지배하지 않는다.
6. 나중에 반대안으로 바꾸려면 여러 Architecture Element와 데이터·계약을 함께 바꿔야 한다.
7. 특정 Model·Agent·DB 제품 선택이나 timeout, retry, buffer 크기 같은 tuning 문제가 아니다.
8. 같은 대상·조건에서 A와 B가 상호 배타적이며, 각각 상대안과 대등한 steelman으로 구성되어 있다.
9. hybrid와 유력한 제3안을 검토했다. hybrid가 양쪽 장점을 취할 수 있으면 이를 새 A로 삼고, 상호 배타적인 강한 B를 다시 도출한다. 그런 B가 없으면 DP를 재구성하거나 제외한다.

“A가 이 QA에서 유리할 수 있다”는 표현은 승자를 미리 정한다는 뜻이 아니다. 실제 결과는 반대이거나 동점일 수 있다. 중요한 것은 **왜 차이가 생길 수 있는지를 구조로 설명할 수 있는가**이다.

모든 후보의 상세 논의는 [DP 공통 검토 절차](./dp-review-protocol.md)를 따른다. 상호 배타성은 구현 기술의 공존 여부가 아니라 같은 범위에서의 권한·계약·기준 상태·실행 경계로 확인한다. snapshot·cache·로그 같은 정상적인 보완책을 한쪽에서 금지하여 차이를 만들지 않는다.

## 4. 12개 후보의 전체 지도

후보는 앞 결정이 뒤 결정의 범위를 정의하는 순서로 배치했다.

```text
VIA의 책임 범위와 상태의 주인
  VIA-DP-01 직접 처리 범위
  VIA-DP-02 대화·요청·업무 상태 소유 구조

Voice와 의미 이해 경로
  VIA-DP-03 음성 처리 구성
  VIA-DP-04 음성 응답 확정 권한
  VIA-DP-05 Context 전달 방식
  VIA-DP-06 의미 판단 구조

업무 위임과 복구
  VIA-DP-07 복합 요청 실행 조정 책임
  VIA-DP-08 상태 저장 및 복구 기준

외부 dependency와 실행 경계
  VIA-DP-09 Agent 차이 흡수 위치
  VIA-DP-10 Model Runtime 연결 구조
  VIA-DP-11 연동 코드 Process 격리

연구와 검증 기반
  VIA-DP-12 실행 기록과 평가 근거 소유 구조
```

| ID | 한 문장 질문 | 대안 A | 대안 B |
| --- | --- | --- | --- |
| VIA-DP-01 | 범위가 정해진 정보 처리의 실행 책임을 VIA에도 둘 것인가? | 선택적 직접 처리 + Agent 위임 | 정보 처리 실행의 Agent 일원화 |
| VIA-DP-02 | 대화·요청·업무 관계의 상태를 누가 확정하는가? | 하나의 통합 상태 소유자 | 수명별로 분리된 상태 소유자 |
| VIA-DP-03 | S2S만으로 Voice Runtime을 구성할 것인가? | S2S 중심 단일 구성 | S2S와 보조 음성 Component 조합 |
| VIA-DP-04 | S2S 직접 응답을 누가 최종 확정하는가? | Core 승인 후 응답 | Voice Runtime이 직접 확정 |
| VIA-DP-05 | 필요한 Context를 언제 어떤 형태로 전달하는가? | 요청 시점의 불변 Context 묶음 | 필요할 때 범위가 제한된 Context 조회 |
| VIA-DP-06 | 요청 의미를 한 번에 판단할 것인가? | 하나의 통합 의미 판단 | 계약으로 분리된 단계별 판단 |
| VIA-DP-07 | 복합 요청의 실행 관계를 누가 조정하는가? | VIA가 요청 관계를 조정 | Agent가 복합 업무를 조정 |
| VIA-DP-08 | 재시작 후 무엇을 복구의 기준 기록으로 삼는가? | 현재 상태 저장 | 이벤트 이력 저장 |
| VIA-DP-09 | Agent별 기능과 수명 차이를 어디서 해석하는가? | Agent 경계에서 공통 형식으로 변환 | Core가 유형별 차이를 해석 |
| VIA-DP-10 | 여러 Model Runtime을 각 Component가 직접 연결하는가? | Component별 직접 연결 | 공통 Model 연결부 사용 |
| VIA-DP-11 | 외부 연동 코드의 치명적 장애를 어디까지 격리하는가? | Core와 같은 Process | 별도 연동 Process |
| VIA-DP-12 | 실행 기록과 평가 근거를 누가 소유하는가? | 각 Component가 기록 | 공통 평가 근거 영역이 기록 |

12라는 숫자 자체가 목표는 아니다. 상세 검토에서 대안 하나가 정상적으로 성립하지 않거나 다른 후보와 독립적이지 않으면 합치거나 제외한다.

### 상세 논의 전에 발견한 보완 사항

**12개 모두 상호 배타성·hybrid·steelman 검토 완료 전의 초안이다.** 아래는 현재 설명에 대한 예비 점검이며, 후보 재설계나 최종 판정을 완료한 표가 아니다.

| 후보 | 상세 논의에서 먼저 해결할 문제 |
| --- | --- |
| VIA-DP-01 직접 처리 범위 | 요청별 직접 처리·위임 혼합이 가능하다. 비교할 기능 범위와 처리 책임을 먼저 정하고, 업무를 외부로 옮겨 얻은 수치 이점을 분리한다. |
| VIA-DP-02 상태 소유 | 일부 관계만 통합하는 혼합을 검토한다. 같은 상태·관계의 최종 변경 권한으로 구분하며, 논리적 소유자 분리를 Process 장애 격리와 동일시하지 않는다. |
| VIA-DP-03 음성 처리 | 양쪽이 같은 음성 기능을 제공할 수 있는지 확인한다. 보조 기능을 선택적으로 사용하는 안과 S2S의 실제 제공 기능을 반영해 결정 축을 좁힌다. |
| VIA-DP-04 응답 확정 | 요청별 승인 생략과 Core가 미리 부여한 권한을 검토한다. 같은 요청 범위에서 누가 최종 확정하는지, 권한 이전과 fallback까지 정의한다. |
| VIA-DP-05 Context 전달 | 불변 묶음과 필요 시 조회는 결합 가능하다. 혼합안을 구성하고 같은 정보 시점·권한에서 무엇을 서로 배타적으로 결정하는지 재정의한다. |
| VIA-DP-06 의미 판단 | 여러 내부 판단을 하나의 최종 권한자가 취합할 수 있다. Model 호출 수와 최종 판단 권한을 구분하고 부분 통합안을 검토한다. |
| VIA-DP-07 복합 요청 | VIA와 Agent가 서로 다른 요청 관계를 조정하는 혼합이 가능하다. 같은 관계의 조정 권한을 비교하고 양쪽에 동일한 Agent 기능을 제공한다. |
| VIA-DP-08 상태 저장 | 양쪽 모두 현재 상태와 이력을 저장할 수 있다. snapshot을 허용한 뒤 불일치·복구 시 최종 기준 기록으로 대안을 구분한다. |
| VIA-DP-09 Agent 차이 | 공통 계약과 기능별 확장을 함께 사용할 수 있다. 기능 손실 없이 차이를 해석하는 최종 책임 위치를 분명히 한다. |
| VIA-DP-10 Model 연결 | 공통 관리와 Component별 stream 직접 연결을 결합할 수 있다. 연결·스케줄링·자원 공유·Process 배치를 한꺼번에 묶었는지 확인한다. |
| VIA-DP-11 Process 격리 | 일부 연동 코드만 격리할 수 있다. 비교할 동일 코드·fault 범위를 고정하고 선택적 격리안을 검토한다. |
| VIA-DP-12 실행 기록 | Component별 기록과 공통 수집은 공존 가능하다. 기록 계약·완전성·보관 책임으로 축을 재검토하고, 제품 로그와 독립적인 평가 관측을 구분한다. |

## 5. 후보별 설명과 QA trade-off

아래 A/B와 QA 유불리는 도출 당시의 가설이다. 위 보완 사항과 공통 절차를 통과해야 비교안으로 확정할 수 있다. 특히 상대안에 정상적인 보완책을 적용하면 사라지는 장점은 상세 검토에서 철회한다.

### VIA-DP-01 — VIA 직접 처리 범위

**질문:** 대상과 범위가 명확한 read/search/understand 요청을 VIA가 직접 처리할 것인가, Downstream Agent에 맡길 것인가?

이 결정은 요청별 routing 규칙 하나를 고르는 문제가 아니다. VIA 안에 범위가 정해진 정보를 읽고 답하는 기능을 둘지, VIA를 interaction과 orchestration 중심의 얇은 시스템으로 만들지를 결정한다.

상세 사고실험은 [DP-01 독립 검토 보고서](./via-dp-01-direct-handling.md)에 있다. 아래는 그 보고서의 v1 요약이며, 초기 후보 단계의 QA 우세 주장은 재검토했다.

#### 대안 A — 선택적 직접 처리 + Agent 위임

VIA가 선언된 제한된 정보 처리 기능을 직접 소유하고 요청 조건에 따라 Agent 위임과 조합한다. 직접 실패 시 허용된 fallback도 포함한다. 첫 상세 검토는 지정 문서 구간의 발췌·요약으로 범위를 고정하며, 열린 조사·실제 행동은 Agent에 위임한다.

#### 대안 B — 정보 처리 실행의 Agent 일원화

VIA는 사용자 요청 이해, Context 연결, Task 관리와 Agent 선택을 담당한다. 대상 정보 처리는 모두 Agent 실행 책임으로 두되, 빠른 전용 Agent와 최소 Context 전달을 허용한다. S2S 직접 응답과 VIA가 이미 보유한 Task 상태 응답은 A와 같이 유지한다.

**달라지는 구조:** 범위가 정해진 정보에 답하는 Component의 존재, Context와 Model dependency 방향, 직접 응답과 Agent execution 경로, Task 생성 범위가 달라진다.

| 초기 기대의 재검토 | v1 판단 |
| --- | --- |
| A의 QA-21 Agent 변화 영향 범위 우세 | 철회. A도 공통 Agent 연동을 유지하며, 직접 요청 수 감소는 변경 요소 수 감소가 아님 |
| A의 QA-51 불필요한 보호정보 노출 우세 | 철회. B도 최소 범위만 전달할 수 있으며 총 전달량과 초과 노출은 다름 |
| B의 QA-22 변경 범위·QA-41 PC 메모리 우세 | 조건부 가설. 공유 모델·공통 계약으로 차이가 줄거나 역전될 수 있어 실제 변경 ledger·자원 경계 확인 필요 |

이 DP에서는 A가 QA-02 Direct 경로, B가 QA-01 Delegated 경로를 사용할 수 있으므로 두 responsiveness QA를 서로 맞대어 승패를 내지 않는다. Agent 실행시간을 측정 구간 밖으로 옮긴 것만으로 B가 더 빠르다고 결론내리지 않고 사용자 전체 대기시간과 **QA-11 전체 요청 처리 정확도**를 함께 본다.

**Core DP 확정은 보류한다.** 실행 책임의 구조 차이는 있으나, 양쪽 steelman 이후 여러 QA에서 충분한 trade-off가 남는지는 아직 입증하지 못했다. 독립 검토 보고서의 전체 19개 QA 사고실험과 후속 확인 조건을 따른다.

### VIA-DP-02 — 대화·요청·업무 상태 소유 구조

**질문:** Conversation, VIA Request, VIA Task, Pending Interaction과 Agent Execution 연결 관계를 하나의 상태 소유자가 확정할 것인가, 수명이 다른 상태 소유자들이 나누어 확정할 것인가?

#### 대안 A — 통합 상태 소유자

하나의 상태 소유자가 대화, 요청, 업무, 대기 질문·승인과 외부 실행 연결 관계를 함께 소유한다. 관계 변경을 가능한 한 하나의 일관성 경계에서 검사한다.

#### 대안 B — 분리된 상태 소유자

대화·요청 상태 소유자와 Task 상태 소유자가 각각 자기 상태의 단일 변경 권한자가 된다. 두 상태는 version과 identity가 명시된 계약과 event로 연결한다.

**달라지는 구조:** 상태를 쓸 수 있는 Component, 상태 전이 Interface, 관계를 저장하는 State, 장애가 전파되는 범위가 달라진다.

| A가 유리할 수 있는 QA | B가 유리할 수 있는 QA |
| --- | --- |
| **QA-13 올바른 Task·Interaction 연결, QA-14 비동기 상태 수렴, QA-15 대화·Task 연속성:** 관계를 한 일관성 경계에서 검사할 수 있음. **QA-61 실행 trace 완전성:** 전체 실행을 한 authority에서 연결할 수 있음. | **QA-22 State 변화 영향 범위:** 상태 종류별 변경을 국소화할 수 있음. **QA-31 복구시간과 QA-32 장애 영향 범위:** 영향받은 owner만 복구·격리할 수 있음. |

기존 TASK-DP01은 Task를 최종 변경할 주체만 비교한다. 이 후보는 그보다 먼저 Conversation·Request·Task 전체의 소유 구조를 묻는다.

### VIA-DP-03 — 음성 처리 구성

**질문:** S2S Model이 제공하는 기능을 중심으로 Voice Runtime을 단순하게 구성할 것인가, 명시적인 VAD·ASR·시간 정렬·TTS 또는 helper model을 함께 구성할 것인가?

두 대안 모두 S2S를 기본 Voice Model로 사용한다. 특정 helper 제품을 고르는 문제는 이 DP에 포함하지 않는다.

#### 대안 A — S2S 중심 단일 구성

S2S가 speech understanding, generation과 허용된 direct response의 주 dependency다. 별도 speech model과 중간 계약을 최소화한다.

#### 대안 B — S2S와 보조 음성 Component 조합

S2S를 유지하면서 제품에 필요한 turn detection, transcript revision, word timing, interruption control 또는 Core 응답 재생을 명시적인 보조 Component와 계약으로 제공한다.

**달라지는 구조:** Voice Runtime 내부 Component, streaming event Interface, 음성 관련 State, Model Runtime dependency와 call graph가 달라진다.

| A가 유리할 수 있는 QA | B가 유리할 수 있는 QA |
| --- | --- |
| **QA-02 직접 Voice 응답시간:** Model 호출과 경계가 적을 수 있음. **QA-41 target PC 메모리:** PC 내부 실행 요소가 적을 수 있음. **QA-22 변화 영향 범위:** 음성 dependency 변경 요소가 적을 수 있음. | **QA-03 Agent 상태 Voice 전달시간과 QA-04 음성 중단시간:** 명시적인 시간·제어 신호를 사용할 수 있음. **QA-12 의미 해석 정확도:** 안정된 전사문 version을 사용할 수 있음. **QA-61 실행 trace 완전성:** 세부 음성 event의 출처 정보를 남길 수 있음. |

### VIA-DP-04 — 음성 응답 확정 권한

**질문:** S2S가 직접 답할 수 있는 요청에서도 Core가 응답을 승인해야 하는가, Voice Runtime이 직접 응답을 확정할 수 있는가?

이 결정은 단순 latency 최적화가 아니다. 같은 User Turn을 누가 완료로 선언하고, Core의 중복 처리와 정정·중단을 어떻게 막는지 결정한다.

#### 대안 A — Core 승인 후 응답

S2S가 streaming 결과를 만들 수 있지만 Core가 direct/escalated route와 policy를 확정한 뒤 meaningful response를 사용자에게 내보낸다.

#### 대안 B — Voice Runtime이 직접 응답 확정

허용된 direct request는 Voice Runtime이 response를 확정한다. Core에는 같은 Request의 중복 실행을 막고 Conversation에 기록하기 위한 명시적인 commit·escalation event를 전달한다.

**달라지는 구조:** response release authority, route/response State, Voice–Core Interface와 foreground call graph가 달라진다.

| A가 유리할 수 있는 QA | B가 유리할 수 있는 QA |
| --- | --- |
| **QA-11 전체 요청 처리, QA-12 의미 해석, QA-15 연속성:** 동일한 semantic/policy 경계를 통과함. **QA-51 불필요한 보호정보 노출:** 응답 전에 Context 제공 범위를 확인할 수 있음. | **QA-02 직접 Voice 응답시간:** Core 왕복을 피할 수 있음. **QA-41 target PC 메모리:** foreground route State와 처리 hop을 줄일 수 있음. |

VIA-DP-03은 어떤 음성 dependency를 조합하는지, VIA-DP-04는 누가 응답을 확정하는지를 결정하므로 서로 다른 DP다.

### VIA-DP-05 — Context 전달 방식

**질문:** 요청 처리에 필요한 화면·선택·대화·Task Context를 요청 초기에 하나의 묶음으로 확정할 것인가, 각 소비자가 필요한 시점에 제한된 범위만 조회할 것인가?

#### 대안 A — 요청 시점의 불변 Context 묶음

Request version마다 필요한 Context를 수집하고 허용 범위를 적용한 뒤, 값과 출처 정보가 포함된 불변 묶음을 만든다. 각 Model·Core·Agent에는 그 소비자에게 허용된 묶음만 전달한다.

#### 대안 B — 필요할 때 범위가 제한된 Context 조회

Request에는 범위와 version이 명시된 handle/capability를 연결한다. 각 소비자는 Context Broker를 통해 필요한 항목만 허용된 시점에 조회한다.

사용자가 대상을 지정한 시점이 중요한 화면 Context는 B에서도 당시 source version/time에 고정한다. on-demand 조회를 현재 화면으로 임의 치환하지 않는다.

**달라지는 구조:** Context 묶음을 만드는 Component, Context 전달 Interface, 출처·cache State, 외부 제공 dependency 방향이 달라진다.

| A가 유리할 수 있는 QA | B가 유리할 수 있는 QA |
| --- | --- |
| **QA-01 위임 경로 VIA 처리시간과 QA-02 직접 Voice 응답시간:** 반복 조회를 줄일 수 있음. **QA-12 의미 해석, QA-61 trace 완전성, QA-62 평가 재현성:** 동일한 Context 묶음을 사용할 수 있음. | **QA-41 target PC 메모리와 QA-51 불필요한 보호정보 노출:** 필요한 정보만 실제 값으로 가져올 수 있음. **QA-12 의미 해석:** 최신 상태가 필요한 요청에서 새 값을 읽을 수 있음. **QA-22 변화 영향 범위:** 정보 출처의 변화를 Broker에 국소화할 수 있음. |

양쪽의 접근·제공 policy는 동일하게 고정한다. Policy enforcement 위치가 Context 방식과 독립적인 A/B를 만들 때만 별도 DP로 분리한다.

### VIA-DP-06 — 의미 판단 구조

**질문:** referent, 복합 Request 관계, 기존 Task 연결, direct/delegated 처리와 Agent capability 요구를 하나의 판단이 함께 결정할 것인가, 독립된 단계가 계약을 주고받으며 결정할 것인가?

#### 대안 A — 하나의 통합 의미 판단

하나의 semantic authority가 필요한 Context와 Task view를 받아 최종 Semantic Decision을 함께 생성한다.

#### 대안 B — 계약으로 분리된 단계별 판단

grounding/refinement, Task association, handling/Agent selection을 독립된 responsibility와 intermediate contract로 나눈다. 앞 단계의 오류를 수정할 수 있는 correction contract를 둔다.

**달라지는 구조:** semantic Component 수와 책임, 중간 Interface와 State, Model 호출 순서와 correction call graph가 달라진다.

| A가 유리할 수 있는 QA | B가 유리할 수 있는 QA |
| --- | --- |
| **QA-01 위임 경로 VIA 처리시간, QA-02 직접 Voice 응답시간, QA-41 target PC 메모리:** 호출과 경계를 줄일 수 있음. **QA-12 의미 해석 정확도:** 전체 판단 근거를 한 번에 함께 볼 수 있음. | **QA-22 변화 영향 범위:** 단계별 변경을 국소화할 수 있음. **QA-61 실행 trace 완전성:** 단계별 판단 근거와 출처가 명확함. **QA-12 의미 해석 정확도:** 한 가지 판단에 집중된 입력 형식이 Model에 더 적합할 수 있음. |

QA-12의 방향은 Model 능력과 실제 corpus에 따라 달라질 수 있다. 단계가 많으면 항상 정확하거나 통합 판단이면 항상 빠르다고 가정하지 않는다.

### VIA-DP-07 — 복합 요청 실행 조정 책임

**질문:** 사용자가 여러 요청을 독립 실행, 순차 실행, 앞선 결과를 이용한 실행 또는 조건부 실행으로 연결했을 때, VIA가 각 실행을 조정할 것인가, 하나의 복합 업무로 Agent에 맡길 것인가?

두 대안 모두 사용자가 말한 Request 관계를 보존한다. VIA가 Agent 내부 domain plan을 만드는 구조는 허용하지 않는다.

#### 대안 A — VIA가 요청 관계 조정

VIA가 각 Request node의 readiness와 결과 관계를 관리하고 node별 Agent execution을 연결한다.

#### 대안 B — Agent가 복합 업무 조정

VIA는 사용자 요청 관계를 기록하지만, 서로 연결된 요청들은 이를 처리할 수 있는 Agent에 하나의 복합 실행으로 위임한다.

B도 사용자가 특정 부분만 조회·정정·취소할 수 있도록 request-node correlation, 부분 결과와 control capability를 제공해야 한다.

**달라지는 구조:** Request graph scheduler Component, Task–Execution cardinality, node control Interface와 부분 결과 State가 달라진다.

| A가 유리할 수 있는 QA | B가 유리할 수 있는 QA |
| --- | --- |
| **QA-05 Task 제어 응답시간과 QA-13 binding 정확도:** node별 control과 binding을 직접 관리함. **QA-14 상태 수렴과 QA-15 연속성:** 부분 상태를 명시적으로 추적함. **QA-61 trace 완전성:** node와 execution 관계가 VIA trace에 드러남. | **QA-01 위임 경로 VIA 처리시간과 QA-41 target PC 메모리:** VIA의 handoff와 상태 수를 줄일 수 있음. **QA-22 변화 영향 범위:** graph orchestration 변경을 Agent 내부에 둘 수 있음. |

B를 시험할 때 복합 업무 기능이 없는 Agent를 제공해 의도적으로 실패시키지 않는다. 같은 사용자 기능을 지원하는 Agent 시험 조건이 먼저 필요하다.

### VIA-DP-08 — 상태 저장 및 복구 기준

**질문:** 재시작 후 현재 상태를 직접 읽어 복구할 것인가, 발생한 이벤트 이력을 재생해 현재 상태를 다시 만들 것인가?

이 결정은 VIA-DP-02의 상태 소유자와 독립적이다. 통합 상태 소유자도 이벤트 이력을 사용할 수 있고, 분리된 상태 소유자도 현재 상태 저장소를 사용할 수 있다.

#### 대안 A — 현재 상태 저장

권위 있는 현재 상태, version, 대기 중 동작과 outbox/inbox를 저장한다. 외부 실행은 Agent 조회와 멱등 식별자로 맞춘다.

#### 대안 B — 이벤트 이력 저장

상태를 바꾼 사건을 순서대로 추가한 이벤트 이력을 복구의 기준 기록으로 저장한다. 현재 Task 보기, 복구 상태와 평가 입력은 그 이력으로부터 다시 만든다.

**달라지는 구조:** 저장 Interface, durable State 형식, migration과 replay Component, recovery call graph가 달라진다.

| A가 유리할 수 있는 QA | B가 유리할 수 있는 QA |
| --- | --- |
| **QA-05 Task 제어 응답시간과 QA-31 복구시간:** current state를 바로 읽을 수 있음. **QA-41 target PC 메모리:** replay State를 줄일 수 있음. **QA-22 변화 영향 범위:** 단순 current-schema 변경이면 수정 범위가 작을 수 있음. | **QA-14 상태 수렴과 QA-15 연속성:** event 순서와 correction을 재생할 수 있음. **QA-61 trace 완전성과 QA-62 평가 재현성:** 원시 실행 이력을 보존할 수 있음. |

특정 DB 제품은 이 DP와 분리하여 비교 조건으로 고정한다.

### VIA-DP-09 — Agent 차이 흡수 위치

**질문:** Agent마다 다른 submit, follow-up, cancel, question, status와 result 의미를 Agent 경계에서 공통 형식으로 바꿀 것인가, Core가 유형별 차이를 직접 해석할 것인가?

#### 대안 A — Agent 경계에서 공통 형식으로 변환

integration adapter가 provider lifecycle 차이를 versioned canonical operation과 observation으로 바꾼다. Core는 공통 계약만 사용한다.

#### 대안 B — Core가 Agent 유형 차이를 해석

integration boundary는 provider-neutral typed variation을 전달한다. Core의 capability handler가 follow-up, cancel, status, artifact mode의 차이를 해석한다.

**달라지는 구조:** Agent semantic adapter와 Core handler 책임, Agent Interface, capability와 execution-mode State가 달라진다.

| A가 유리할 수 있는 QA | B가 유리할 수 있는 QA |
| --- | --- |
| **QA-21 Agent 변화와 QA-22 관련 계약 변화 영향 범위:** 추가·교체를 integration edge에 국소화할 수 있음. **QA-13 binding 정확도:** Core가 한 가지 binding rule을 사용함. | **QA-05 Task 제어, QA-11 전체 요청 처리, QA-14 상태 수렴:** Agent capability 차이를 Core가 명시적으로 볼 수 있음. **QA-61 trace 완전성:** provider mode가 trace에 직접 나타남. |

A가 Agent capability를 잃거나 B가 native payload를 Core 전체에 누출하는 나쁜 후보를 만들지 않는다. B의 유일한 차이가 Core 분기 증가뿐이라면 이 후보는 Core DP에서 제외한다.

### VIA-DP-10 — Model Runtime 연결 구조

**질문:** Voice와 semantic Component가 각자 Model Runtime을 직접 연결하고 관리할 것인가, 하나의 공통 Model Gateway가 연결·배치·자원을 관리할 것인가?

#### 대안 A — Component별 Model 직접 연결

각 Component가 자기 역할에 맞는 Model adapter, stream, cancellation과 health 처리를 소유한다.

#### 대안 B — 공통 Model 연결부 사용

공통 Model Gateway/Runtime Manager가 Model 호출, stream 형식 통일, 실행 순서, PC 내부·외부 배치 연결, health와 자원 공유를 소유한다.

**달라지는 구조:** Model adapter와 gateway Component, invocation Interface, scheduling/health State, local/remote Deployment binding이 달라진다.

| A가 유리할 수 있는 QA | B가 유리할 수 있는 QA |
| --- | --- |
| **QA-01/02 responsiveness와 QA-04 음성 중단시간:** 추가 gateway hop이 없음. **QA-32 장애 영향 범위:** 하나의 gateway fault가 여러 기능으로 퍼지지 않음. | **QA-22 변화 영향 범위:** provider·deployment 변경을 국소화할 수 있음. **QA-41 target PC 메모리:** Model instance를 공유할 수 있음. **QA-31 복구, QA-51 정보 노출, QA-61 trace 완전성:** 공통 health·filter·trace를 적용할 수 있음. |

특정 Model이나 local/cloud 위치를 하나 고르는 것은 이 DP가 아니다. Architecture가 그러한 변화를 어디에서 흡수하는지가 질문이다.

### VIA-DP-11 — 연동 코드 Process 격리

**질문:** 외부 Agent와 Context Source를 연결하는 코드에서 치명적 장애가 발생할 때 Core와 함께 종료되도록 둘 것인가, 별도 Process 안에서 격리할 것인가?

#### 대안 A — Core와 같은 Process

Core와 외부 연동 코드를 하나의 OS process에 둔다. 크기 제한 queue, 비동기 작업, timeout과 cancellation으로 논리적으로 격리한다.

#### 대안 B — 별도 연동 Process

동일한 외부 연동 코드를 supervised worker process에 둔다. Core state와 policy authority는 Core process에 유지하고 versioned IPC로 연결한다.

**달라지는 구조:** Process Deployment, local bridge/IPC Interface, in-flight operation State, worker supervision Component가 달라진다.

| A가 유리할 수 있는 QA | B가 유리할 수 있는 QA |
| --- | --- |
| **QA-01 위임 경로, QA-03 Agent 상태 전달, QA-05 Task 제어 응답시간과 실제 Context 연동을 사용하는 QA-02 경로:** Process 간 통신과 데이터 복사를 피할 수 있음. **QA-41 target PC 메모리:** 중복 Process 실행 요소가 적음. | **QA-32 장애 영향 범위:** 치명적 장애를 worker에 가둘 수 있음. **QA-31 복구시간:** worker만 재시작할 수 있음. **QA-61 trace 완전성:** Process별 기록 출처가 명확해질 수 있음. |

IPC 종류, worker 수와 queue 크기는 이 DP의 대안이 아니라 후보 구현 전에 고정할 조건이다.

### VIA-DP-12 — 실행 기록과 평가 근거 소유 구조

**질문:** 각 Component가 자신의 trace를 독립적으로 남기고 나중에 결합할 것인가, 공통 Evidence Plane이 실행 중에 end-to-end trace와 평가 근거를 구성할 것인가?

양쪽 모두 QA 측정에 필요한 최소 event와 correlation identity를 제공해야 한다. 로그가 부족한 A를 만들어 B가 이기게 해서는 안 된다.

#### 대안 A — 각 Component가 기록

각 Component가 versioned local trace를 소유한다. evaluator와 exporter가 공통 identity를 사용해 실행 후 trace를 결합한다.

#### 대안 B — 공통 평가 근거 영역이 기록

공통 event 형식, 수집·임시 저장 Component와 불변 근거 저장소가 실행 중에 처음부터 끝까지 이어진 trace를 구성한다.

**달라지는 구조:** trace producer Interface, correlation State, collector와 evidence store Component, evidence Deployment와 dependency 방향이 달라진다.

| A가 유리할 수 있는 QA | B가 유리할 수 있는 QA |
| --- | --- |
| **QA-41 target PC 메모리:** 공통 수집 Component와 buffer가 적음. **QA-32 장애 영향 범위:** 기록 수집 장애가 실행 경로에 퍼지지 않게 만들기 쉬움. **QA-51 불필요한 보호정보 노출:** 중앙 전송을 줄일 수 있음. | **QA-23 실험·로그 변경 영향 범위:** 계측 변경을 공통 영역에 모을 수 있음. **QA-61 실행 trace 완전성과 QA-62 평가 재현성:** 공통 기록 형식과 불변 근거를 사용할 수 있음. |

## 6. 전체 문제 영역이 빠짐없이 연결되는가

시스템 이해 과정에서 확인한 구조적 난제와 이를 주로 다루는 후보를 연결했다.

| 구조적 난제 | 관련 후보 |
| --- | --- |
| 음성·전사·pointer·화면의 서로 다른 시간축 결합 | VIA-DP-03 음성 처리, VIA-DP-05 Context 전달, VIA-DP-06 의미 판단 |
| 빠른 Voice 경로와 하나의 사용자 의미 양립 | VIA-DP-03 음성 처리, VIA-DP-04 응답 확정, VIA-DP-06 의미 판단, VIA-DP-02 상태 소유 |
| VIA Task와 Agent Execution 식별자 분리 | VIA-DP-02 상태 소유, VIA-DP-07 복합 요청 조정, VIA-DP-09 Agent 계약, VIA-DP-08 상태 저장 |
| 비동기 상태의 권위와 진실성 | VIA-DP-02 상태 소유, VIA-DP-07 복합 요청 조정, VIA-DP-08 상태 저장, VIA-DP-09 Agent 계약 |
| 재시작 복구와 외부 동작 중복 방지 | VIA-DP-08 상태 저장, VIA-DP-11 Process 격리, VIA-DP-09 Agent 계약 |
| 충분한 Context와 최소 노출 | VIA-DP-01 직접 처리, VIA-DP-05 Context 전달, VIA-DP-10 Model 연결 |
| 복합 요청 관계와 Agent planning 경계 | VIA-DP-01 직접 처리, VIA-DP-06 의미 판단, VIA-DP-07 복합 요청 조정 |
| 실시간 Voice와 비동기 결과 중재 | VIA-DP-04 응답 확정, VIA-DP-02 상태 소유, VIA-DP-11 Process 격리 |
| Model·Agent·Context 생태계 변화 격리 | VIA-DP-05 Context 전달, VIA-DP-09 Agent 계약, VIA-DP-10 Model 연결 |
| 측정 가능한 Architecture 인과관계 | VIA-DP-12 평가 근거 소유, VIA-DP-08 상태 저장, VIA-DP-02 상태 소유 |

현재 후보 단계에서는 위 구조적 문제가 빠짐없이 적어도 하나의 후보와 연결된다. 이것이 12개 모두를 자동으로 최종 DP로 승인한다는 뜻은 아니다.

## 7. QA 전체를 빠짐없이 검토했는가

아래 표는 각 QA가 두 대안을 직접 구분할 가능성이 있는 후보를 보여준다.

- **주 비교 지표:** A와 B의 구조 차이가 이 QA에 직접 영향을 주므로 승패 평가에 사용한다.
- **회귀 확인 지표:** 두 대안 모두 반드시 지켜야 하지만, 이 DP의 승패를 가르는 지표로 사용하지 않는다.
- **해당 없음:** 그 QA가 측정하는 처리 경로가 이 DP에 존재하지 않거나 구조적으로 달라지지 않는다.

후보를 상세화한 뒤 실제 처리 경로나 Architecture Element가 바뀌지 않으면, 아래에 연결했더라도 회귀 확인 지표 또는 해당 없음으로 내린다.

| QA | 구조 인과가 있을 수 있는 후보 |
| --- | --- |
| QA-01 위임 경로 VIA 처리시간 | VIA-DP-05 Context, VIA-DP-06 의미 판단, VIA-DP-07 복합 요청, VIA-DP-09 Agent 계약, VIA-DP-10 Model 연결, VIA-DP-11 Process 격리 |
| QA-02 직접 Voice 응답시간 | VIA-DP-03 음성 처리, VIA-DP-04 응답 확정, VIA-DP-05 Context, VIA-DP-06 의미 판단, VIA-DP-10 Model 연결, VIA-DP-11 Process 격리 중 실제 Context 연동 경로 |
| QA-03 Agent 상태 Voice 전달시간 | VIA-DP-02 상태 소유, VIA-DP-03 음성 처리, VIA-DP-09 Agent 계약, VIA-DP-10 Model 연결, VIA-DP-11 Process 격리 |
| QA-04 음성 중단시간 | VIA-DP-03 음성 처리, VIA-DP-10 Model 연결 중 실제 음성 중단 전달 경로 |
| QA-05 Task 제어 응답시간 | VIA-DP-02 상태 소유, VIA-DP-07 복합 요청, VIA-DP-08 상태 저장, VIA-DP-09 Agent 계약, VIA-DP-11 Process 격리 |
| QA-11 전체 요청 처리 정확도 | VIA-DP-01 직접 처리, VIA-DP-04 응답 확정, VIA-DP-05 Context, VIA-DP-06 의미 판단, VIA-DP-07 복합 요청, VIA-DP-09 Agent 계약 |
| QA-12 의미 해석 정확도 | VIA-DP-03 음성 처리, VIA-DP-04 응답 확정, VIA-DP-05 Context, VIA-DP-06 의미 판단, VIA-DP-10 Model 연결 |
| QA-13 Task·Interaction 연결 정확도 | VIA-DP-02 상태 소유, VIA-DP-07 복합 요청, VIA-DP-08 상태 저장, VIA-DP-09 Agent 계약 |
| QA-14 비동기 상태 수렴 정확도 | VIA-DP-02 상태 소유, VIA-DP-07 복합 요청, VIA-DP-08 상태 저장, VIA-DP-09 Agent 계약 |
| QA-15 대화·Task 연속성 | VIA-DP-02 상태 소유, VIA-DP-04 응답 확정, VIA-DP-05 Context, VIA-DP-07 복합 요청, VIA-DP-08 상태 저장 |
| QA-21 Agent 변화 영향 범위 | VIA-DP-01 직접 처리, VIA-DP-07 복합 요청, VIA-DP-09 Agent 계약 |
| QA-22 Model·Context·State 변화 영향 범위 | VIA-DP-01~10 중 실제 변경의 영향을 받는 후보 |
| QA-23 실험·로그 변화 영향 범위 | VIA-DP-12 평가 근거 소유 |
| QA-31 올바른 Task 복구시간 | VIA-DP-02 상태 소유, VIA-DP-08 상태 저장, VIA-DP-09 Agent 계약, VIA-DP-10 Model 연결, VIA-DP-11 Process 격리 |
| QA-32 장애 영향 범위 | VIA-DP-02 상태 소유, VIA-DP-10 Model 연결, VIA-DP-11 Process 격리, VIA-DP-12 평가 근거 소유 |
| QA-41 target PC 메모리 | VIA-DP-01~12 중 target PC의 Component·Process·buffer가 실제로 달라지는 후보 |
| QA-51 불필요한 보호정보 노출 | VIA-DP-01 직접 처리, VIA-DP-04 응답 확정, VIA-DP-05 Context, VIA-DP-10 Model 연결, VIA-DP-12 평가 근거 소유 |
| QA-61 실행 trace 완전성 | VIA-DP-02~12 중 기록을 만드는 주체·식별자·Process가 실제로 달라지는 후보 |
| QA-62 평가 재현성 | VIA-DP-05 Context, VIA-DP-08 상태 저장, VIA-DP-12 평가 근거 소유 |

한 QA가 여러 후보에 등장해도 같은 점수를 여러 번 가산하지 않는다. 이 표는 DP별 QA applicability를 결정하기 위한 사전 sweep이다. DP-01의 상세 검토에서는 QA-21·51의 초기 우세 기대를 철회했으며, 해당 행에 DP-01이 남아 있는 것은 회귀 검토 연결이지 주 비교 지표로 확정했다는 뜻이 아니다.

## 8. 후보들이 서로 같은 결정을 중복하고 있지 않은가

각 후보를 비교할 때 다음 항목은 다른 DP의 선택으로 고정한다.

| 후보 | 이 후보가 결정하지 않는 것 |
| --- | --- |
| VIA-DP-01 직접 처리 범위 | Context 전달 방식, 의미 판단 단계 수, Agent 계약 형식 |
| VIA-DP-02 상태 소유 구조 | 상태를 현재 값 또는 이벤트 이력으로 저장하는 방식, Process 배치 |
| VIA-DP-03 음성 처리 구성 | S2S 직접 응답의 최종 승인 권한 |
| VIA-DP-04 음성 응답 확정 권한 | VAD·ASR·TTS·보조 Model 구성과 의미 판단 단계 수 |
| VIA-DP-05 Context 전달 방식 | Context 접근 허용 범위와 의미 판단 책임 |
| VIA-DP-06 의미 판단 구조 | 사용할 Model 제공자, Context 전달 방식, 직접 처리 범위 |
| VIA-DP-07 복합 요청 조정 책임 | Agent 고유 통신 형식과 Task 상태 저장 방식 |
| VIA-DP-08 상태 저장·복구 기준 | 상태 변경 권한자의 위치와 DB 제품 |
| VIA-DP-09 Agent 차이 흡수 위치 | Agent 기능 수준, Task 소유자와 Process 배치 |
| VIA-DP-10 Model Runtime 연결 | 의미 판단 책임과 실제 Model 실행 조건 |
| VIA-DP-11 Process 격리 | 내부 업무 로직, 저장 방식과 전체 worker 자원 한도 |
| VIA-DP-12 평가 근거 소유 | 각 QA가 요구하는 최소 event 의미와 보호정보 처리 규칙 |

다음 조합은 서로 다른 결정이지만 interaction이 강하므로, 한 DP의 결론이 다른 DP 선택에 따라 뒤집힐 가능성이 있을 때만 제한된 교차 확인을 한다.

- VIA-DP-02 상태 소유 × VIA-DP-08 상태 저장
- VIA-DP-03 음성 처리 × VIA-DP-04 응답 확정
- VIA-DP-04 응답 확정 × VIA-DP-06 의미 판단
- VIA-DP-05 Context 전달 × VIA-DP-06 의미 판단
- VIA-DP-07 복합 요청 조정 × VIA-DP-09 Agent 계약
- VIA-DP-09 Agent 계약 × VIA-DP-11 Process 격리
- VIA-DP-10 Model 연결 × VIA-DP-11 Process 격리
- VIA-DP-08 상태 저장 × VIA-DP-12 평가 근거 소유

모든 12개 DP의 전체 조합을 실행해 하나의 전체 우승 구성을 고르는 방식은 사용하지 않는다.

## 9. Core DP로 만들지 않은 항목

| 항목 | 처리 | 이유 |
| --- | --- | --- |
| 특정 Model·Agent·DB·IPC 제품 | 제외 | dependency 또는 구현 선택이며 구조 축이 아님 |
| polling과 event 중 하나 선택 | 공통 tactic | 빠른 update와 reconnect/gap query는 상호 보완적 |
| timeout, retry, buffer, worker 수 | 측정 전 고정할 tuning 값 | 책임이나 상태 소유권을 바꾸지 않음 |
| PC 내부 또는 cloud Model 하나 선택 | VIA-DP-10의 배치 조건 | Architecture는 위치 변화를 흡수해야 함 |
| Context 허용 범위를 검사하는 위치 | VIA-DP-05에 우선 포함 | Context 방식과 독립적인 정상 A/B가 확인되면 분리 |
| 음성 출력 순서 조정 알고리즘 | VIA-DP-03/04의 구현 전술 | 별도 권한·State 소유자가 필요할 때만 재검토 |
| Agent state event 또는 query | 공통 tactic | 둘은 보완 관계이며 delivery mechanism만으로는 Core DP가 아님 |

## 10. 기존 DP와 비교하면 무엇이 달라지는가

기존 DP는 이번 도출의 출발점이 아니라 결과를 확인한 뒤 비교한 참고자료다.

| 기존 항목 | 새 후보와의 관계 | 제안 |
| --- | --- | --- |
| IR-DP01 | VIA-DP-06 의미 판단 구조와 거의 동일 | 새 QA trade-off로 다시 검토 |
| TASK-DP01 | VIA-DP-02 상태 소유 구조 중 Task 변경 권한자 부분만 다룸 | VIA-DP-02 범위를 먼저 확정한 뒤 독립 유지 여부 판단 |
| AGENT-DP01 | VIA-DP-09 Agent 차이 흡수 위치와 거의 동일 | Agent 기능을 잃지 않는 상태에서 B의 장점이 실제로 있는지 확인 |
| EXEC-DP01 | VIA-DP-11 Process 격리의 구체적인 형태 | 격리할 연동 코드 범위를 명시하고 재사용 가능 |
| S2S Direct Fast Path 원칙 | VIA-DP-04의 대안 B를 미리 선택한 형태 | VIA-DP-04 A/B 검토 전에는 고정 원칙으로 두지 않음 |
| Event-first + Query Reconciliation | 이번 검토에서도 공통 tactic | DP로 승격하지 않음 |
| 기존 Context 관련 선택 | VIA-DP-05의 부분 후보 | 현재 후보 계약으로 다시 정의 |

현재 승인된 ADR은 이 검토 초안만으로 변경되지 않는다. 후보 집합과 각 후보의 상세 정의를 승인한 뒤에만 유지·재검증·대체 여부를 판단한다.

## 11. 다음 논의 순서

먼저 전체 후보의 예비 점검에서 드러난 hybrid 가능성·중복·선행 의존을 정리하고 논의 순서를 확인한다. 이후 `VIA-DP-01`부터 하나씩 [DP 공통 검토 절차의 동일한 목차](./dp-review-protocol.md#6-모든-dp를-논의할-동일한-순서)를 적용한다.

상세 구조와 QA 표를 작성하기 전에 **hybrid 검토 → 양쪽 steelman 구성 → 상호 배타성 확인**을 완료한다. 성립하지 않으면 A/B를 재구성한다. 후보 수와 순서는 검토 결과에 따라 달라질 수 있고, 기존 ID는 이력 확인을 위해 유지한다.

각 DP에는 [전체 QA 사고실험 비교표](./dp-review-protocol.md#73-전체-qa-사고실험-비교표)를 포함한다. 전체 19개 QA를 대상으로 어느 안이 얼마나 우세한지, 비슷한지, 판단이 어려운지를 구조적 이유·조건과 함께 예측한다. 현재 5절의 유불리 가설을 나열한 표는 이 전체 비교표를 대신하지 않는다.

각 DP 상세 문서는 **그림을 포함한 배경 인트로 1페이지 → 대안 A/B 설명과 Mermaid 구조 그림 → 구조 차이 요약 → QA 사고실험 표**로 읽히도록 구성한다. 배경은 중요성·필요성·고민할 지점을 설명하고, A/B 그림은 동일한 VIA 전체 또는 일부 영역에서 무엇이 같고 다른지 보여준다. 구체적인 표현과 검토 기준은 [DP 그림 작성 기준](./dp-diagram-guide.md)을 따른다.

각 DP 페이지 자체가 완성된 보고서여야 한다. 필요한 시스템 맥락·용어·비교 조건을 본문에서 설명하고, 구조와 QA 근거에서 현재 판단·검증 계획까지 이어지게 한다. 처음 보는 SW Architect 심사관이 이전 대화나 다른 DP를 읽었다고 가정하지 않는다. [독립 보고서 목차와 읽기 검토](./dp-review-protocol.md#62-독립된-완성-보고서의-읽기-순서)를 완료 기준으로 적용한다.

모든 후보가 이 검토를 통과한 뒤에만 최종 Core DP inventory를 확정한다. 구현·측정·ADR 변경은 그 이후 단계다.
