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
| 5 | [Representative Use Cases](./05-representative-use-cases.md) | 검토 완료 · 기준선 확정 |
| 6 | [Fixed Assumptions](./06-fixed-assumptions.md) | 작성 완료 · 06/07 공동 검토본 |
| 7 | [Intentional Variables](./07-intentional-variables.md) | 작성 완료 · 06/07 공동 검토본 |
| 8 | Architecture Significant Requirements | 예정 |
| 9 | ASR ↔ UC / Evolution Scenario Mapping | 예정 |
| 10 | Architecture Element Definition | 예정 |
| 11 | Test Case Catalog | 이후 작성 |
| 12 | Architecture Decision Points | 이후 작성 |

## 06·07 공동 검토 안내

**06은 같은 비교에서 고정할 조건, 07은 Architecture가 대응해야 할 변화이다.** 제품 요구와 내부 설계 선택은 두 문서에서 임의로 바꾸지 않는다.

| 읽을 부분 | 확인할 내용 |
| --- | --- |
| 06 §6.2~6.3 | 고정/변화 짝 비교표, Windows·언어·사용자·작업량 등의 평가 제안 |
| 06 §6.5~6.7 | 모델의 공정한 비교, VIA-only 지연의 한계, Agent 기능과 재시작 조건 |
| 07 §7.3~7.4 | 채택·제외한 변화와 모델 8개 / Agent 7개 / Context·Memory 4개 변경 시나리오 |
| 07 §7.5~7.6 | 전체 집합 적용, 0개·적용 없음·처리 불가 구분, 변경 후 기능 유지 |

06·07은 검토본이며 아직 사용자 승인이나 실험 완료를 뜻하지 않는다. 숫자가 있는 시험 조건은 제품의 성능 목표·사용 통계·최대 용량이 아니다. 실제 장비·모델 속도·녹음·정답 파일을 확보했다고 가정하지 않는다.

이번 작성에서는 01~05를 다시 대조했으며 제품 범위를 바꿀 필요가 없어 해당 파일은 수정하지 않았다. 기존 ASR이나 주요 DP도 미리 확정하지 않았다.

## 05 검토 안내

05는 대표 UC 18개에 사용자 목표, 시작 상황, VIA의 책임, 성공 완료 조건, 필수 변형, 실패·보류 행동을 정리하고 사용자 검토를 통해 제품 범위를 확정했다.

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
