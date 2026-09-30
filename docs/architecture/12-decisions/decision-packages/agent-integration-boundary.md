# 서로 다른 Agent의 실행 연동을 공통으로 관리할 것인가

> 상태: **우선 검토 후보 / 사용자 선정 전** · [목록](./README.md)

**질문:** 서로 다른 Agent의 전송·수신·재접속 lifecycle을 공통 Agent Gateway가 관리할 것인가, Agent별 독립 연동 Component가 소유할 것인가?

> **발표 구성:** 배경 1장 → 설계 비교 1장. 아래 배경 문구와 도식 구성은 슬라이드 제작용 원고이며 새 배경 그림이나 발표 파일을 생성한 것은 아니다. 사례는 고정 UC를 설명하기 위한 예시이고, 발생 빈도·수치·대안의 우위를 측정한 결과가 아니다.

## 1. 배경 페이지 — 왜 중요한 Architecture 문제인가

### 문제가 드러나는 사용자 상황

어떤 Agent는 실행 ID와 순서 있는 event를 제공하고, 다른 Agent는 상태 polling이 필요하다. “취소 요청을 받았다”와 “실제로 취소됐다”의 의미도 다를 수 있다. VIA는 이런 차이를 흡수하면서 사용자의 Task와 승인·결과를 유지해야 한다.

**Agent별 차이를 어디까지 공통 구조가 책임지고, 어디부터 개별 연동이 책임지는가**가 문제다. 어떤 Agent를 고르는 순위나 matching 알고리즘을 비교하는 후보는 아니다.

### 이 과제에서 왜 중요한가

VIA는 특정 Agent를 최상위 runtime으로 삼지 않는 Agent-neutral interaction 계층이다. 사용자는 어느 Agent가 처리하든 같은 VIA 창구에서 업무를 맡기고, 질문에 답하고, 결과를 확인해야 한다. Agent 연동 경계가 약하면 새 Agent를 추가하거나 기존 protocol을 바꿀 때 대화·Task 처리까지 provider별 분기로 바뀌어 과제의 독립성이 무너진다.

여기서 흡수해야 하는 차이는 JSON 필드명이나 SDK 호출 방식에 그치지 않는다. 접수 확인, 상태의 순서, 중복 제출 방지, 취소 확인, 승인 결합처럼 **어디까지 사실로 확인할 수 있는가**도 다를 수 있다. 예를 들어 전송 응답을 잃었을 때 외부 업무는 이미 시작됐을 수 있다. VIA는 연결 실패를 미실행으로 단정하고 다시 보내서는 안 된다.

### Architecture적으로 어려운 이유

| 동시에 만족해야 할 요구 | 구조적으로 부딪히는 지점 | 단순 처리로 남는 문제 |
| --- | --- | --- |
| VIA는 일관된 업무 의미를 제공한다 / Agent의 상태·제어 능력은 다를 수 있다 | 공통 contract와 capability별 보장 범위 | 공통 API 이름만 같다고 취소 접수·실제 종료를 같은 의미로 취급할 위험 |
| 연결 실패에서 회복한다 / 외부 업무를 중복 실행하지 않는다 | command key·전송 시도·접수 불명과 source query의 상태 소유권 | 무조건 retry하면 이미 실행된 업무를 다시 만들 수 있음 |
| event를 빠르게 반영한다 / 순서·누락·중복을 처리해야 한다 | inbox·cursor·snapshot 확인을 어느 연동 owner가 책임질지 | event 도착 순서만으로 완료 상태를 되돌리거나 다른 실행에 반영할 수 있음 |
| provider별 변경을 국소화한다 / 공통 lifecycle 규칙을 중복하지 않는다 | adapter를 넘어 상태·복구·version까지 나눌 범위 | 공통 계층의 예외 증가와 개별 계층의 규칙 중복 사이에 비용 발생 |

Task의 의미 상태와 전송·수신 상태를 구분해야 한다. 업무 완료 여부는 Task Manager가 확인된 Agent 사실로 관리하지만, 전송 시도·inbox·cursor는 연동 계층의 책임이다. 이 후보는 그 연동 상태를 공통 Agent Gateway가 소유할지, Agent별 Component로 나눌지의 문제다. Agent 내부의 업무 planning을 VIA로 가져오는 선택은 포함하지 않는다.

### 배경 슬라이드 1장에 사용할 내용

**제목:** Agent-neutral 경험을 제공하려면 protocol뿐 아니라 상태·복구 보장의 차이도 흡수해야 한다.

| 배치 | 슬라이드에 남길 핵심 문구 | 함께 보여줄 도식 |
| --- | --- | --- |
| 왼쪽 — 과제 환경 | 사용자는 같은 VIA Task로 여러 Agent를 이용한다 | 공통 사용자 창구와 VIA Task 아래 Agent A·B 연결; 업무 reasoning은 외부 경계에 둠 |
| 가운데 — 차이의 예 | push event와 polling, command key 조회·취소 확인의 지원 범위가 다를 수 있다 | 가상의 A/B capability 표와 상태 수신 경로. 실제 provider 조사 결과로 표기하지 않음 |
| 오른쪽 — 실패 상황 | submit 응답 유실은 미실행의 증거가 아니다 | VIA 전송 → Agent 접수 → 응답 유실 → UNKNOWN → 확인 가능한 조회 경로; 무조건 재전송은 차단 |

**하단 Challenge:** Agent별 차이와 불명 상태를 정직하게 보존하면서 공통 업무 경험을 유지하려면, 전송·수신·복구 상태의 책임을 어디까지 공통화해야 하는가?

**발표자 설명:** “Adapter는 이미 양쪽에 있습니다. 여기서 고민하는 것은 API 변환을 어디에 쓰느냐가 아니라, 전송을 시도했고 응답을 잃었을 때 누가 그 불명 상태를 기억하고 해결하느냐입니다. 공통 owner에 모으면 규칙을 일관되게 관리할 수 있지만 provider별 예외가 쌓일 수 있고, 개별 owner로 나누면 독립성이 생기는 대신 lifecycle 규칙의 정합성을 유지해야 합니다.”

**배경 근거:** [고정 범위 §3.7 Agent-neutral](../../03-fixed-architecture-scope.md#37-agent-neutral-원칙), [UC-08 위임](../../05-representative-use-cases.md#uc-08), [UC-12 취소](../../05-representative-use-cases.md#uc-12), [UC-18 실패·불명](../../05-representative-use-cases.md#uc-18), [제어 계약 §5 capability별 처리](../target-architecture/control-and-lifecycle.md#agent-capability별-처리).

## 2. 설계 비교 페이지 — Adapter 위의 전송·수신 lifecycle을 공통 또는 Agent별로 소유한다

**배경에서 이어받는 질문:** 문서 첫머리의 구조적 질문을 같은 사용자 목표·조건에서 비교한다. 먼저 책임 경계, 다음 상태와 계약, 마지막 이점·비용의 순서로 설명한다.

### 두 방안

**방안 1 — 공통 실행 연동.** Agent Gateway가 capability, 전송 시도, 접수 불명, inbox cursor와 canonical 변환을 소유한다. Agent별 adapter는 그 안에서 provider protocol 차이를 흡수한다. Task Manager는 공통 Agent Command·Agent Event와 확인된 capability를 사용한다. 이미 adapter가 있는 구조를 단순한 단일 구현으로 그리지 않는다.

**방안 2 — Agent별 실행 연동.** 공통 Agent Gateway Component를 두지 않고 대안 전용 Agent Integration A·Agent Integration B 등이 각 Agent의 capability·command delivery·inbox·reconciliation을 소유한다. Task Manager는 versioned 공통 최소 port와 명시적 capability extension으로 각각 연결한다. 공통 전송 도구·schema·검증 library는 재사용할 수 있다.

![공통 Agent Gateway와 Agent별 연동 Component 비교](./diagrams/agent-integration-boundary.svg)

[draw.io 편집 원본](./diagrams/agent-integration-boundary.drawio) · [표기 규칙](./diagrams/README.md)

### 그림에서 확인할 흐름과 구조 차이

① Task Manager의 업무 command에서 ② 연동 port, ③ Agent별 submit/event/poll, ④ canonical event 반환으로 읽는다. 양쪽 모두 adapter가 있으므로 adapter 유무를 차이로 주장하지 않는다. 파란 전송 시도·inbox·cursor·capability와 원장 owner가 공통 Agent Gateway에서 Agent별 Component로 분리되는 것이 차이다. Task payload 의미와 외부 Agent의 업무 reasoning은 그대로 남는다. command 전달과 접수 불명 조회의 상세 왕복은 해당 내부 책임 블록에 묶어 표시했다.

| 구조 차이 | 방안 1 | 방안 2 |
| --- | --- | --- |
| Component | Agent Gateway 내부 여러 adapter | Agent별 독립 연동 Component |
| 전송·수신 상태 | 공통 lifecycle owner | Agent별 lifecycle owner·namespace |
| Task Manager의 연결 | 공통 port 한 종류 | 공통 최소 port의 여러 구현 + 명시적 extension |
| protocol 변화 | 공통 lifecycle와 맞추어 adapter 변경 | 해당 연동 owner가 수용, 필요하면 extension 계약 변경 |

### 어느 쪽이 설득력 있는가

방안 1은 동일한 전송·수신 규칙과 capability 해석을 한곳에서 유지하기 좋다. 대신 서로 다른 Agent의 기능이 공통 lifecycle에 잘 맞지 않으면 확장과 예외가 쌓일 수 있다. 공통 계약이 필연적으로 최소 기능만 제공하는 것은 아니다.

방안 2는 Agent의 event·control·recovery 방식이 크게 다르고 독립적인 변경 주기가 필요할 때 유리할 수 있다. Task Manager 안에 vendor SDK를 직접 넣는 대안이 아니므로 공통 최소 계약과 강한 캡슐화를 유지할 수 있다. 대신 lifecycle 구현·내구 기록·버전 호환 관리가 여러 곳으로 퍼지고 extension이 Task Manager에 영향을 줄 수 있다.

### 설계 비교 페이지 발표 설명

**그림에서 짚을 순서:** 양쪽의 검정 adapter가 공통으로 존재함을 먼저 보여준다. 파란 전송 시도·inbox·cursor·capability와 그 원장이 공통 Agent Gateway에서 Agent별 Component로 이동하는 흐름을 따라간다.

**발표자 설명:** “SDK를 감추는 adapter는 두 방안 모두 있습니다. 핵심은 그 위의 접수 불명·재조회·event 수신 상태를 누가 책임지는가입니다. 공통 계약이 provider 차이를 자연스럽게 수용한다면 분리 이점이 작고, 예외가 누적돼 다른 Agent까지 함께 수정해야 한다면 개별 owner의 근거가 강해집니다.”

## 3. 자체 검토와 남은 질문

- 각 연동은 VIA 소속이다. 외부 Downstream Agent가 VIA Task identity나 최상위 orchestration을 소유하지 않는다.
- Agent 선택은 두 방안 모두 Request Interpreter의 제안과 Request Controller의 검증으로 수행한다. capability 조회 대상만 공통 owner에서 개별 owner로 달라진다.
- 업무 payload의 의미는 Task Manager가 소유하고, 전송·수신 상태만 연동 owner가 가진다. 접수 불명일 때 상태 확인 없이 재전송하는 대안은 허용하지 않는다.
- **후보로 남길 조건:** 단지 현재 adapter를 밖에 그리는 것이 아니라 전송·수신 lifecycle의 상태·버전·변경 책임이 실제로 이동해야 한다. 현재 adapter와 차이가 사라지면 독립 후보에서 제외한다.
- 공통 계약이 Agent별 요구를 자연스럽게 수용한다면 분산 owner의 필요성이 약하다. 반대로 예외가 누적되어 모든 Agent 변경을 함께 조정해야 한다면 공통 owner를 재검토할 이유가 된다. 아직 확인된 결과는 아니다.

기준선 근거: [전체 구조 §4·11·12](../target-architecture/architecture.md), [제어 계약 §5](../target-architecture/control-and-lifecycle.md). 주요 사용자 상황: UC-08·10·12~14·16·18.
