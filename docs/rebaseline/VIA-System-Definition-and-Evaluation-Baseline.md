# VIA 시스템 정의 및 평가 기준선

> 상태: Architecture Rebaseline 작업 기준선  
> 작업 브랜치: `architecture-rebaseline-20260918`

이 디렉터리는 VIA Architecture를 다시 정의하기 위한 기준선 문서를 관리한다.

원칙은 다음과 같다.

> **먼저 시스템과 사용자 요구를 정의하고, 그 정의에서 품질 속성·ASR·평가 시나리오·Architecture Decision Point를 도출한다. 기존 Architecture 결론에 맞추어 요구사항을 역으로 구성하지 않는다.**

## 문서 구조

| 번호 | 문서 | 상태 |
| --- | --- | --- |
| 1 | [System Mission & Boundary](./01-system-mission-and-boundary.md) | 작성 완료 · 05와 정합성 보완 |
| 2 | [Terms](./02-terms.md) | 작성 완료 · 05와 정합성 보완 |
| 3 | [Fixed Architecture Scope](./03-fixed-architecture-scope.md) | 작성 완료 · 05와 정합성 보완 |
| 4 | [Canonical Interaction Flow](./04-canonical-interaction-flow.md) | 작성 완료 · 흐름과 제어 의미 보완 |
| 5 | [Representative Use Cases](./05-representative-use-cases.md) | 작성 완료 · 사용자 검토본 |
| 6 | Fixed Assumptions | 예정 |
| 7 | Intentional Variables | 예정 |
| 8 | Architecture Significant Requirements | 예정 |
| 9 | ASR ↔ UC / Evolution Scenario Mapping | 예정 |
| 10 | Architecture Element Definition | 예정 |
| 11 | Test Case Catalog | 이후 작성 |
| 12 | Architecture Decision Points | 이후 작성 |

## 05 검토 안내

05는 대표 UC 18개에 사용자 목표, 시작 상황, VIA의 책임, 성공 완료 조건, 필수 변형, 실패·보류 행동을 정리한다.

먼저 전체 UC 목록을 읽고 다음을 확인한다.

- [UC-03: 먼저 지정하고 말하기](./05-representative-use-cases.md#uc-03), [UC-04: 말하면서 지정하기](./05-representative-use-cases.md#uc-04)
- [UC-07: 직접 응답에서 업무로 이어가기](./05-representative-use-cases.md#uc-07), [UC-09: 복합 요청](./05-representative-use-cases.md#uc-09)
- [UC-10: 기존 업무](./05-representative-use-cases.md#uc-10), [UC-14: 여러 업무](./05-representative-use-cases.md#uc-14)

UC는 사용자 요구와 완료 조건을 고정한다. 구체적인 녹음·화면·정답 데이터는 11번, Model/Agent 변경은 별도의 Evolution Scenario에서 다룬다. 작성 완료는 Test Case 실행이나 Architecture 평가 완료를 뜻하지 않는다.

## 작성 원칙

- 각 번호는 독립 Markdown 파일로 관리한다.
- 문서 본문은 한글을 기본으로 작성한다.
- Architecture 용어는 해석 차이를 줄이기 위해 필요한 경우 영어 용어를 병기한다.
- 구조·흐름·lifecycle·책임 경계는 가능한 경우 Mermaid Diagram을 함께 제공한다.
- 아직 결정하지 않은 내용을 장기간 TBD로 남기지 않는다. 해당 절에서 결정해야 하는 내용은 그 절을 닫기 전에 결정한다.
- ASR은 미리 고정하지 않고 System Definition, Use Case, Assumption, Evolution Scenario를 먼저 정의한 뒤 도출한다.
- UC의 완료 조건과 내부 처리 구조를 구분한다. 특정 후보가 유리하도록 사용자 요구나 정답을 바꾸지 않는다.
