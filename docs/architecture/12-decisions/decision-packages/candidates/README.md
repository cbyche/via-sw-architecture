# 구조적 Decision Point 후보

> 상태: **DISCUSSION_DRAFT / 구조 후보 자체 검토 완료·사용자 검토 대기**
> 작성일: 2026-09-30 · 기준: [검토 완료 Target Architecture](../../target-architecture/README.md)

이 문서는 VIA의 핵심 문제를 해결하는 SW 구조적 선택을 후보로 정리한다. **대안에 따라 Component의 책임·구성·연결이 실질적으로 달라지는 선택을 우선한다.** 발표에서는 해결할 문제와 두 구조의 차이를 먼저 설명하고, 그 차이가 필요한 이유와 비용을 설명한다.

사용자는 구조 후보를 먼저 검토하고 ASR은 뒤에 논의하도록 요청했다. 따라서 여기서는 ASR 선정·변경·우선순위·metric을 정하지 않는다. 메모리를 포함한 자원 비용은 구조의 비용으로 설명하되 QA-41의 지위를 바꾸지 않는다. 정식 package, 기준선 변경, 기존 DP 매핑, 구현·모델 실행·성능 측정·freeze도 수행하지 않았다.

## 폴더 구성과 읽는 순서

`12-decisions/decision-packages/candidates/`를 후보 작업 공간으로 사용한다. 기존 저장소의 후보·package 위치를 지키면서 검토 완료 설계와 논의 중 대안을 분리한다. 아직 정식 DP 번호를 부여하지 않으며 파일명은 해결 문제를 따른다.

1. [선발 원칙](./selection-principles.md): 사용자와 합의한 선발 기준 및 이를 적용하는 방법.
2. 아래 후보 표: 문제와 두 구조의 차이를 한 번에 비교.
3. 각 후보 파일: 구체적인 상황, 두 방안, 비교 그림, 비용과 자체 검토.
4. [후보 검토 기록](./review-notes.md): 약한 대안의 보완, 후보 간 경계, 보류·제외 이유.

본문의 `owner`는 상태를 최종 변경할 책임자, `admission`은 실행·게시의 허용 판단, `publication`은 사용자에게 게시하는 응답 단위, `revision`은 변경 버전을 뜻한다. Component 이름은 기준선과 대조할 수 있도록 정식 명칭을 유지한다.

## 먼저 논의할 구조 후보

아래 다섯 항목은 **선발 원칙에 따른 우선 검토 제안**이다. 채택 개수나 순서는 확정하지 않았다. 방안 1은 기준선의 구조, 방안 2는 이를 설명·재검토하기 위한 대안이다.

| 문제와 후보 | 방안 1 — 기준선 | 방안 2 — steelman 초안 | 그림에서 보이는 실제 변화 |
| --- | --- | --- | --- |
| [요청의 해석과 확정을 어떻게 나눌 것인가](./request-resolution-boundary.md) | Request Interpreter가 의미 제안, Request Controller가 흐름·확정 소유 | Request Controller에 의미 해석 orchestration을 통합하고 내부 검증 경계를 유지 | Request Interpreter의 독립 Component 경계 제거, proposal 계약의 내부화 |
| [여러 곳에서 필요한 Context를 누가 준비할 것인가](./context-preparation-ownership.md) | Context Manager가 요청 목적에 맞는 공통 근거 package 구성 | 소비 Component별 Context 준비, Context Manager는 공통 source 접근·기억 관리 | 공통 구성 책임이 소비자 쪽으로 이동하고 조회 연결이 분기 |
| [대화와 오래가는 업무 상태를 어디서 관리할 것인가](./conversation-task-ownership.md) | Request Controller와 Task Manager가 별도 상태 권위 | Request Controller에 Task Manager의 상태 관리 책임 통합 | Task Manager 경계 제거, 두 owner의 인계가 한 owner의 상태 전이로 변경 |
| [응답 전달을 한곳에서 관리할 것인가](./response-publication-ownership.md) | Response Manager가 직접 답변·업무 알림의 전달을 공통 관리 | 직접 답변과 업무 알림을 별도 응답 Component가 관리 | Response Manager가 둘로 분리, Interaction Manager의 공통 발화 조정 계약 확대 |
| [서로 다른 Agent 연동을 어디서 흡수할 것인가](./agent-integration-boundary.md) | Agent Gateway가 공통 전송·수신 lifecycle과 adapter 관리 | Agent별 독립 연동 Component가 전송·수신 lifecycle 소유 | Agent Gateway의 공통 실행 경계가 Agent별 경계로 분리 |

## 조건을 더 확인할 후보

| 후보 | 남겨두는 이유 | 우선 후보와 구분한 이유 |
| --- | --- | --- |
| [음성 입력 근거 경로를 독립시킬 것인가](./speech-evidence-boundary.md) | 별도 ASR dependency와 입력 worker의 유무가 실제 구조를 바꿈 | VIA 최상위 Component 증감보다 dependency·하위 모듈 경계의 변화다. 대안의 Omni 입력 기능 확보 여부도 미확인 |
| [실시간 입출력과 상태 처리를 프로세스로 격리할 것인가](./runtime-isolation.md) | blocking·process crash를 어디까지 함께 겪는지 달라짐 | 논리 Component는 유지되고 runtime 배치가 달라진다. Component 변화 우선 원칙에서는 후순위 |

스케줄링 우선순위·호출 횟수·보관 기간은 이 표의 독립 후보가 아니다. Agent 선택 알고리즘과 DB 제품 선택도 같은 이유로 별도 후보로 올리지 않았다. 공유 transaction 대 owner별 journal은 구조 차이가 있으나 현재는 업무 상태 소유권 후보와의 중복을 먼저 정리하도록 보류했다. 상세 이유는 [검토 기록](./review-notes.md)에 있다.

## 그림의 읽기 규칙

각 파일은 **구체적인 사용자 상황을 두 구조가 어떻게 처리하는지** 나란히 보여준다. Component 경계 안의 책임, 핵심 계약, 상태 원장과 주요 흐름을 함께 표시한다. 전체 시스템의 모든 호출을 그린 그림은 아니며, 공통 서비스 접근 중 생략한 연결은 그림 주석과 본문으로 밝힌다.

- **검정은 두 방안의 공통 부분, 파랑은 양쪽에서 달라지는 부분**이다. 방안 1도 차이에 해당하는 경계·상태 owner·연결을 파랑으로 표시한다. 색은 선정 여부나 우열을 뜻하지 않는다.
- 파란 Component 경계 안에 검정 내부 블록이 있으면 기능은 유지되면서 소속·책임 경계가 달라진다는 뜻이다. 내부 책임 블록을 모두 새 Component로 세지 않는다.
- `«component»`는 논리 책임 경계, `«process / runtime»` 점선은 실행 경계, 이름이 있는 점선 dependency는 외부 모델·서비스, 원통은 저장 기록이다. `공개 조회 계약`은 owner의 공개 port를 모은 표현이다.
- 번호는 주요 처리 흐름을 읽는 순서다. 실선은 호출·event, 점선 화살표는 상태 접근이다. 선의 교차 자체는 연결을 뜻하지 않는다. 정확한 thread·시간 순서를 확정한 sequence diagram은 아니다.
- 기준선 Component는 [정식 명칭](../../target-architecture/architecture.md#4-component와-상태-소유권)을 유지한다. 대안에서 새로 도입한 Component에는 `대안 전용`을 표시한다.
- 각 그림 아래에 공통 조건과 방안별 이점·비용·검토 질문을 둔다. 아직 검증하지 않은 수치나 승자를 넣지 않는다.
- SVG preview와 같은 이름의 편집 가능한 draw.io 원본을 함께 제공한다. 상세 표기법과 수정 방법은 [그림 작성 규칙](./diagrams/README.md)에 있다.

## 이번 검토의 범위

기준선·상세 계약과의 대조, 동일 사용자 행동 유지 여부, Component 변화의 실질성, 강한 대안과 비용, 후보 간 중복을 자체 검토했다. 별도 심사위원이나 독립 에이전트가 검증한 결과가 아니며, 실제 구현 가능성·성능을 실험으로 입증한 상태도 아니다. 다음 논의에서는 후보를 채택·통합·보류한 뒤 관련 ASR을 검토한다.
