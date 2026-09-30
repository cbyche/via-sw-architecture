# 대화 상태와 업무 상태의 소유자를 나눌 것인가

> 상태: **우선 검토 후보 / 사용자 선정 전** · [목록](./README.md)

**질문:** 사용자의 대화·요청과 오래가는 업무를 별도 Component가 관리할 것인가, 한 Component가 서로 다른 lifecycle로 관리할 것인가?

## 해결할 문제

보고서 작성 중 사용자가 새 대화를 시작하고, 이후 다른 대화에서 “그 보고서 어디까지 됐어?”라고 묻는다. Agent 질문·완료와 사용자의 취소가 교차할 수도 있다. VIA는 대화가 바뀌어도 업무 identity와 상태를 유지하고, 질문·결과를 올바른 곳에 연결해야 한다.

대화와 업무의 수명이 다르다는 제품 요구는 고정이다. **수명이 다르므로 상태 소유 Component도 반드시 달라야 하는지**가 구조적 질문이다.

## 두 방안

**방안 1 — 대화와 업무의 owner 분리.** Request Controller가 Conversation·Request·대기 질문을, Task Manager가 Task·Execution·확인된 업무 상태를 소유한다. 확정 요청은 Task Manager로 넘어가고, Agent event로 갱신한 업무 상태는 Request Controller를 통해 대화·알림에 연결된다. 원자적 변경이 필요하면 두 owner가 State Store의 Unit of Work에 참여한다.

**방안 2 — 상태 owner 통합.** Task Manager의 책임을 Request Controller에 통합한다. Conversation별 요청 처리와 Task별 실행 상태는 별도 typed aggregate와 mailbox로 관리하지만 상태 의미와 transaction 조정은 한 Component가 소유한다. Task는 대화의 자식 수명에 묶지 않으며 여러 Conversation에서 참조할 수 있다. Agent Gateway와 Response Manager는 유지한다.

![대화 및 업무 상태 소유권의 분리와 통합 비교](./diagrams/conversation-task-ownership.svg)

[draw.io 편집 원본](./diagrams/conversation-task-ownership.drawio) · [표기 규칙](./diagrams/README.md)

### 그림에서 확인할 흐름과 구조 차이

① 취소/후속 입력, ② admission, ③ Agent event, ④ 대화와 업무의 결합, ⑤ 내구 기록을 따라간다. Conversation·Request와 Task·Execution의 상태 종류는 양쪽에 남는다. 파랑은 그 상태를 변경하는 owner 경계와 command/event 계약의 변화다. State Store의 원통 두 개는 논리 기록 구분이며 별도 DB나 분산 transaction을 뜻하지 않는다. 질문 종료와 Task 변경의 원자성 요구는 공통이다.

| 구조 차이 | 방안 1 | 방안 2 |
| --- | --- | --- |
| Component | Request Controller + Task Manager | 업무 관리 책임까지 가진 Request Controller |
| 상태 권위 | Conversation/Request와 Task/Execution의 owner 분리 | 한 owner 아래 독립 aggregate·identity·lifecycle 유지 |
| 상태 인계 | dispatch admission, command, 확인된 업무 변경의 owner 간 계약 | 동일 Component 내부 상태 전이와 module API |
| 변경 책임 | 업무 상태와 대화 흐름의 변경을 port로 제한 | 질문·취소·업무 상태가 얽힌 변경을 한 단위로 관리 |

## 어느 쪽이 설득력 있는가

방안 1은 장기 업무 관리가 대화 흐름에 종속되지 않게 경계를 드러낸다. 대신 결과 저장·질문 종료·응답 intent 사이의 owner 간 정합성을 설계해야 한다. 기준선의 두 Component는 같은 Core process에 있으므로 이 분리가 process crash 격리까지 제공한다고 설명하지 않는다.

방안 2는 취소와 완료, Agent 질문과 사용자 답변처럼 대화·업무를 함께 바꾸는 사건을 한 owner가 다루기 좋다. 같은 local transaction과 내구 기록으로 복구할 수 있고, Task별 비동기 처리로 긴 업무가 대화를 막지 않게 할 수 있다. 대신 대화 기능 변경과 Agent 업무 lifecycle 변화가 같은 Component에 모인다.

## 자체 검토와 남은 질문

- Agent별 thread에 VIA Task identity를 맡기거나 대화 삭제 시 업무를 함께 지우는 대안은 제외한다.
- 통합해도 Conversation·Request·Task·Execution 개념은 합치지 않는다. 긴 외부 호출 중 전역 lock을 잡는 구조도 강요하지 않는다.
- 비교에서는 같은 State Store와 내구 전송·수신 보장을 유지한다. shared DB 대 owner별 journal까지 동시에 바꾸지 않는다.
- **후보로 남길 이유:** 사용자 interaction과 지속 업무의 상태 권위 및 Component 간 인계가 달라진다.
- 통합 구조가 독립 lifecycle과 변경 경계를 간결하게 유지한다면 별도 Task Manager의 필요성이 약해진다. 통합 후 업무 상태 변화가 대화 전반에 퍼진다면 분리 이유가 강해진다. 양쪽 모두 검토 가설이다.
- [해석 경계 후보](./request-resolution-boundary.md)와는 독립 선택이다. 여기서는 Request Interpreter의 책임을 그대로 둔다.

기준선 근거: [전체 구조 §5·11·12·14](../target-architecture/architecture.md), [제어 계약 §4·5·7](../target-architecture/control-and-lifecycle.md). 주요 사용자 상황: UC-06~10·12~16·18.
