# VIA 시스템 정의 및 평가 기준선

> 상태: Architecture Rebaseline 작업 기준선  
> 작업 브랜치: `architecture-rebaseline-20260918`

이 디렉터리는 VIA Architecture를 다시 정의하기 위한 기준선 문서를 관리한다.

원칙은 다음과 같다.

> **먼저 시스템과 사용자 요구를 정의하고, 그 정의에서 품질 속성·ASR·평가 시나리오·Architecture Decision Point를 도출한다. 기존 Architecture 결론에 맞추어 요구사항을 역으로 구성하지 않는다.**

## 문서 구조

| 번호 | 문서 | 상태 |
| --- | --- | --- |
| 1 | [System Mission & Boundary](./01-system-mission-and-boundary.md) | 작성 완료 |
| 2 | [Terms](./02-terms.md) | 작성 완료 |
| 3 | [Fixed Architecture Scope — R1](./03-fixed-architecture-scope-r1.md) | 작성 완료 |
| 4 | [Canonical Interaction Flow](./04-canonical-interaction-flow.md) | 작성 완료 |
| 5 | Representative Use Cases | 예정 |
| 6 | Fixed Assumptions | 예정 |
| 7 | Intentional Variables | 예정 |
| 8 | Architecture Significant Requirements | 예정 |
| 9 | ASR ↔ UC / Evolution Scenario Mapping | 예정 |
| 10 | Architecture Element Definition | 예정 |
| 11 | Test Case Catalog | 이후 작성 |
| 12 | Architecture Decision Points | 이후 작성 |

## 작성 원칙

- 각 번호는 독립 Markdown 파일로 관리한다.
- 문서 본문은 한글을 기본으로 작성한다.
- Architecture 용어는 해석 차이를 줄이기 위해 필요한 경우 영어 용어를 병기한다.
- 구조·흐름·lifecycle·책임 경계는 가능한 경우 Mermaid Diagram을 함께 제공한다.
- 아직 결정하지 않은 내용을 장기간 TBD로 남기지 않는다. 해당 절에서 결정해야 하는 내용은 그 절을 닫기 전에 결정한다.
- ASR은 미리 고정하지 않고 System Definition, Use Case, Assumption, Evolution Scenario를 먼저 정의한 뒤 도출한다.
