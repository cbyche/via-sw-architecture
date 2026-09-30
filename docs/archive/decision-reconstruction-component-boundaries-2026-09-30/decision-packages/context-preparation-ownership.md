# 필요한 Context를 공통 Component가 준비할 것인가

> 상태: **우선 검토 후보 / 사용자 선정 전** · [목록](./README.md)

**질문:** 대화·업무·자료에서 필요한 근거를 모아 구성하는 책임을 Context Manager에 모을 것인가, 이를 사용하는 Component가 각각 소유할 것인가?

> **발표 구성:** 배경 1장 → 설계 비교 1장. 아래 배경 문구와 도식 구성은 슬라이드 제작용 원고이며 새 배경 그림이나 발표 파일을 생성한 것은 아니다. 사례는 고정 UC를 설명하기 위한 예시이고, 발생 빈도·수치·대안의 우위를 측정한 결과가 아니다.

## 1. 배경 페이지 — 왜 중요한 Architecture 문제인가

### 문제가 드러나는 사용자 상황

“아까 설명한 표로 발표자료를 만들어줘”를 이해하려면 실제로 보여준 응답, 표의 원본과 version, 관련 업무 후보가 필요하다. 이후 “완료됐어?”에 답할 때는 최신 업무 상태와 결과 참조가 필요하다. 여러 처리 경로가 일부 같은 정보를 사용하지만 필요한 형태와 시점은 다르다.

**모든 소비자가 같은 큰 Context를 받는 것과, 각자 제멋대로 정보를 읽는 것 사이에서 근거 구성의 책임을 어디에 둘지**가 문제다.

### 이 과제에서 왜 중요한가

VIA는 “이 표”, “아까 설명한 것”, “진행 중인 보고서”처럼 서로 다른 시점과 source를 가리키는 말을 업무 요청으로 연결해야 한다. 같은 문장도 당시 화면, 실제 전달한 답변, 업무 결과 version에 따라 대상이 달라진다. 따라서 Context는 모델 앞에 붙이는 문자열 묶음이 아니라 **사용자 표현과 실제 대상·업무를 연결하는 근거**다.

모델이 잘 추론해도 잘못된 시점의 화면이나 삭제된 선호, 생성만 되고 전달되지 않은 답변을 받으면 입력 전제부터 어긋날 수 있다. 반대로 모든 원본을 매번 모으면 읽기·보관·추론 입력 비용이 늘고 소비 목적에 불필요한 정보까지 전달한다. 필요한 근거를 정확한 범위로 구성하고 수명을 관리하는 위치가 VIA의 핵심 구조 문제가 된다.

### Architecture적으로 어려운 이유

| 동시에 만족해야 할 요구 | 구조적으로 부딪히는 지점 | 단순 처리로 남는 문제 |
| --- | --- | --- |
| 당시 지칭 대상을 유지한다 / 현재 사용할 수 있는 version·권한도 확인한다 | 관측 시점의 근거와 사용 시점의 유효성은 다름 | 현재 화면 재조회만으로 과거 지칭을 대체하거나, 과거 자료를 현재에도 유효하다고 가정 |
| 여러 owner의 근거를 결합한다 / 원본 상태의 권위는 각 owner에 남는다 | 공개 read port·revision·조회 receipt를 조합하는 책임 | 편의를 위해 내부 DB를 직접 읽으면 owner 계약과 변경 경계를 침범 |
| 해석과 응답이 근거를 공유한다 / 소비 목적과 필요한 상세는 다르다 | 공통 evidence 구성과 소비자별 view의 소유권 | 단일 거대 package는 변경을 묶고, 독립 복사는 누락·불일치 책임을 분산 |
| 재사용·요약한다 / 삭제·권한 철회는 모든 파생 사용에 반영한다 | cache·summary·dependency index의 수명과 무효화 경로 | 원본만 삭제하고 파생 view가 남으면 오래된 정보가 다시 사용될 수 있음 |

Cache TTL이나 검색 알고리즘을 바꾸는 것만으로는 이 선택이 해결되지 않는다. 구성 중 읽은 source·revision·누락을 누가 기록하고, 원본 변경을 어떤 소비자의 view에 전파할지가 필요하다. 공통 구성과 소비자별 구성 모두 같은 원본·권한 요구를 지키되, **근거 결합과 파생 상태를 소유하는 위치**가 달라진다.

### 배경 슬라이드 1장에 사용할 내용

**제목:** 같은 자료를 사용해도 필요한 Context와 유효한 시점은 처리 목적마다 다르다.

| 배치 | 슬라이드에 남길 핵심 문구 | 함께 보여줄 도식 |
| --- | --- | --- |
| 왼쪽 — 분산된 근거 | 당시 화면·실제 응답·Task 결과·허용된 기억의 원본 owner가 다르다 | 각 source에 identity·revision·관측 시점을 붙인 네 근거 묶음 |
| 가운데 — 다른 소비 목적 | “아까 표로 만들어줘”는 지칭 근거, “완료됐어?”는 최신 상태가 필요하다 | 동일 source에서 요청 해석용 view와 응답용 view로 갈라지는 요구. 특정 구성 owner는 아직 선택하지 않음 |
| 오른쪽 — 변화와 수명 | 원본 수정·기억 삭제·권한 철회가 파생 view 사용에 영향을 준다 | 원본 revision 변경 → summary/cache의 재검증·사용 차단 경로 |

**하단 Challenge:** 필요한 근거의 정확한 범위와 수명을 보존하면서 소비자별 변화에 대응하려면, Context 구성과 파생 view를 누가 소유해야 하는가?

**발표자 설명:** “이 과제에서는 같은 자료라도 무엇을 판단하느냐에 따라 필요한 Context가 다릅니다. 과거 지칭에는 당시 근거가 필요하고, 진행 확인에는 현재 상태가 필요합니다. 이 둘을 매번 각자 만들면 revision과 삭제 전파가 흩어지고, 한 계층에 모두 모으면 소비자의 변화가 공통 계약을 바꿉니다. 다음 페이지에서 원본 owner는 유지한 채 구성 책임의 위치를 비교합니다.”

**배경 근거:** [UC-04 발화 중 지칭](../../05-representative-use-cases.md#uc-04), [UC-05 과거 대상](../../05-representative-use-cases.md#uc-05), [UC-07 응답 후속](../../05-representative-use-cases.md#uc-07), [UC-17 기억 제어](../../05-representative-use-cases.md#uc-17), [기억 계약 §2~5](../target-architecture/memory-and-context-lifecycle.md#2-context-획득과-지칭의-정확성).

## 2. 설계 비교 페이지 — 원본 owner를 유지하고 근거 구성·파생 view 책임을 이동한다

**배경에서 이어받는 질문:** 문서 첫머리의 구조적 질문을 같은 사용자 목표·조건에서 비교한다. 먼저 책임 경계, 다음 상태와 계약, 마지막 이점·비용의 순서로 설명한다.

### 두 방안

**방안 1 — 공통 근거 구성.** Context Manager가 각 owner의 versioned read port와 source adapter를 통해 후보·원문·출처·조회 receipt를 구성한다. Request Controller가 허용 범위와 예산을 정한다. Request Interpreter는 전달된 근거를 해석하며 owner 저장소를 직접 읽지 않는다. 소비 목적별 view가 가능하므로 모든 경로에 동일한 거대 package를 강제하는 구조가 아니다.

**방안 2 — 소비자별 근거 구성.** Request Controller는 요청 해석용 view, Response Manager는 허용된 source에서 응답 구성용 view를 각각 만든다. Context Manager는 공통 source 접근·cache·User Memory owner로 남되, 여러 owner의 근거를 합치는 권한과 상태는 소비자에게 이동한다. 소비자는 owner의 공개 read port를 사용하고 source revision·누락·policy를 보존한다. Request Interpreter는 이 대안에서도 Request Controller를 통해 호출한다.

![공통 Context 구성과 소비자별 구성 비교](./diagrams/context-preparation-ownership.svg)

[draw.io 편집 원본](./diagrams/context-preparation-ownership.drawio) · [표기 규칙](./diagrams/README.md)

### 그림에서 확인할 흐름과 구조 차이

위쪽은 동일한 원본 owner의 공개 read port다. 방안 1에서는 조회 receipt·근거 package·파생 view 상태가 Context Manager 안에 모이고, 방안 2에서는 요청용·응답용 준비 모듈과 dependency index가 각 소비자 안으로 이동한다. 파란 조회 경로와 view 상태가 핵심 차이다. 아래 Policy Manager·State Store·Model Access는 공통 지원 계약이며 모든 접근 화살표를 반복하지 않았다. 원본 권한이나 명시적 User Memory owner가 이동하는 대안은 아니다.

| 구조 차이 | 방안 1 | 방안 2 |
| --- | --- | --- |
| 구성 책임 | Context Manager | Request Controller·Response Manager의 내부 준비 모듈 |
| owner 조회 | Context Manager로 모임 | 소비자별 read port 연결로 분기 |
| 파생 view 상태 | Context Manager가 공통 관리 | 소비자별 view·dependency 기록과 무효화 책임 |
| 공통으로 유지 | source 접근 계약·권한·원본 owner·User Memory | 동일; 다른 owner의 내부 DB schema 직접 접근 금지 |

### 어느 쪽이 설득력 있는가

방안 1은 원문·revision·삭제 전파와 중복 읽기를 공통으로 관리하기 좋다. 대신 새로운 소비자의 요구가 Context Manager의 package·cache 계약을 확장할 수 있다. 공통 구성이라고 모든 읽기가 직렬 실행되어야 하는 것은 아니다.

방안 2는 각 소비자가 필요한 근거와 형식을 직접 바꿀 수 있어 소비 목적이 크게 다를 때 유리할 수 있다. 공통 adapter와 무효화 알림을 재사용해 중복 코드를 줄일 수도 있다. 그러나 view·cache가 여러 곳으로 퍼져 비용이 늘 수 있고, 삭제·권한 철회를 모든 소비자에 적용해야 한다.

### 설계 비교 페이지 발표 설명

**그림에서 짚을 순서:** 위쪽 원본 read port는 공통임을 먼저 설명한다. 방안 1의 Context Manager 내부 evidence·view 상태와 방안 2의 소비자별 준비 모듈·dependency index를 비교한 뒤 삭제·revision 변경의 전파 경로로 돌아온다.

**발표자 설명:** “두 방안은 같은 자료를 같은 권한으로 읽습니다. 차이는 읽은 근거를 조합하고 파생 상태를 관리하는 위치입니다. 공통 구성은 receipt와 무효화를 모으지만 소비자 요구 변경이 함께 모이고, 소비자별 구성은 독립성을 얻지만 view의 유효성을 여러 곳에서 지켜야 합니다. 단순히 모델에 더 많은 Context를 넣는 비교가 아닙니다.”

## 3. 자체 검토와 남은 질문

- 화면의 당시 근거 수집은 두 방안 모두 Interaction Manager가 유지한다. 발화가 끝난 뒤 현재 화면만 읽는 대안은 UC-04를 충족하지 못하므로 제외한다.
- Response Manager는 새 업무 의미를 확정하지 않는다. admission한 사실을 표현하기 위한 제한된 읽기만 한다. 새 판단이 필요하면 Request Controller에 돌려준다.
- view 분산은 원본 권위를 분산하는 것이 아니다. Task는 Task Manager, Conversation은 Request Controller, 실제 전달 기록은 Response Manager가 소유한다.
- **후보로 남길 이유:** 공통 근거 구성 책임과 조회 연결이 실제로 이동한다. 동일한 통합 package를 소비자별 adapter로 이름만 바꾸는 대안은 제외한다.
- 공통 구성이 소비자 변화까지 과도하게 묶거나 사용하지 않는 근거를 계속 운반한다면 방안 1을 재검토할 이유가 된다. 소비자별 구성이 서로 다른 revision·삭제 상태를 사용한다면 방안 2의 비용이 커진다. 아직 확인한 결과는 아니다.

기준선 근거: [전체 구조 §9](../target-architecture/architecture.md), [기억 계약 §1~5](../target-architecture/memory-and-context-lifecycle.md). 주요 사용자 상황: UC-02~07·10·15~17.
