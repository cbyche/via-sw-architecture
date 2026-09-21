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
| 6 | [Fixed Assumptions](./06-fixed-assumptions.md) | 검토 완료 · 공통 비교 조건 확정 |
| 7 | [Intentional Variables](./07-intentional-variables.md) | 검토 완료 · 24개 변경 집합 확정 |
| 8 | [Architecture Significant Requirements](./08-architecture-significant-requirements.md) | 재작성 검토본 · QA scoring 검토 필요 |
| 9 | ASR ↔ UC / Evolution Scenario Mapping | 예정 |
| 10 | Architecture Element Definition | 예정 |
| 11 | Test Case Catalog | 이후 작성 |
| 12 | Architecture Decision Points | 이후 작성 |

## 08 검토 안내

08은 01~07에서 품질 후보를 도출하고 중요도와 SW 구조적 난이도를 각각 판단한다. 특정 후보 Architecture나 기존 QA 목록을 정답으로 놓지 않는다.

| 읽을 부분 | 확인할 내용 |
| --- | --- |
| 08 §8.2~8.4 | 제품 관심사, 중요도/난이도 기준, 전체 QA 후보의 선정 이유 |
| 08 §8.5 | 정량 비교 5개와 대표 지표 요약 |
| 08 §8.6~8.9 | 지연·화면/업무 정확도·모델/Agent 변경 요소의 채점 범위와 공정성 |
| 08 §8.10 | 복합 요청·연속성·복구·권한·상태 정합성의 필수 설계 요구 |
| 08 §8.11~8.12 | 사고실험의 반례와 한계, 09~12의 평가 준비 및 동결 순서 |

**08은 선정·측정 정의의 검토본이다.** Architecture별 점수, 후보 승패, 제품 목표 수치 또는 Test Case 실행 완료를 선언하지 않는다. ASR이 모두 점수표 항목인 것은 아니며, 필수 설계 요구도 책임과 검증 근거로 다룬다.

## 06·07 확정 내용

**06은 같은 비교에서 고정할 조건, 07은 Architecture가 대응해야 할 변화이다.** 제품 요구와 내부 설계 선택은 두 문서에서 임의로 바꾸지 않는다.

| 읽을 부분 | 내용 |
| --- | --- |
| 06 §6.2~6.3 | 고정/변화 짝 비교표, Windows·언어·사용자·작업량 등의 평가 기준 |
| 06 §6.5~6.7 | 모델의 공정한 비교, VIA-only 지연의 한계, Agent 기능과 재시작 조건 |
| 07 §7.3~7.4 | 모델 9개 / Agent 9개 / Context·저장 정보 6개 — 전체 24개 변경 시나리오 |
| 07 §7.5~7.6 | 전체 집합 적용, 0개·적용 없음·처리 불가 구분, 변경 후 기능 유지 |
| 07 §7.9 | M-09, A-08·09, C-05·06 추가 및 M-07·A-06 보완 기록 |

06·07의 범위와 비교 규칙은 사용자 검토 및 보완 진행 승인에 따라 확정했다. 숫자가 있는 시험 조건은 제품의 성능 목표·사용 통계·최대 용량이 아니다. 실제 장비·모델 속도·녹음·정답 파일을 확보했다고 가정하지 않는다.

이번 07 보완 및 08 작성에서는 01~05의 제품 범위를 바꾸지 않았다. 변경 집합 24개는 새 DP 24개를 뜻하지 않으며, 기존 설계 질문에서 주요 DP를 선정하는 원칙을 유지한다.

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
