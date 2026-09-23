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
| 8 | [Architecture Significant Requirements](./08-architecture-significant-requirements.md) | 7개 ASR historical baseline · Working-12 후속 계약 사용 |
| 9 | [ASR ↔ UC / Evolution Scenario Mapping](./09-asr-uc-change-mapping.md) | 작성 완료 · 09/10 공동 검토본 |
| 10 | [Architecture Element Definition](./10-architecture-element-definition.md) | 작성 완료 · 09/10 공동 검토본 |
| 11 | [Test Case & Measurement](./11-test-case-catalog.md) | W-01~W-03 Voice 정의 재작성 · 구현/target/score 재동결 전 |
| 12 | [Architecture Evaluation Method](./12-00-evaluation-method.md) | W12-G1 결과 archive 완료 · 새 DP×W mapping 재동결 전 |

## 현재 리뷰 지점 — W-01~W-03 Voice 측정 재정의

> W12-G2에서 W-01~W-03은 각각 Agent delegation 결과, VIA direct Voice response, Agent progress Voice feedback으로 재정의됐다. 현재 source of truth는 [11-E](./11e-voice-responsiveness-measurement-redefinition.md)이며 측정 코드는 아직 변경하지 않았다. 아래 11-A/B 안내는 이전 7-ASR 기준선의 historical context다.

08의7개 ASR과09/10의 연결·집계 원칙을 유지한다. ASR-01은 VIA 모델 비용의 근거·계산을 구체화하고, ASR-07은 고정 분모를 보완했다. 11-A/B는 검토본이며 11-C 목표/0~5점과12의 후보 비교는 진행하지 않았다.

| 읽을 문서 | 확인할 것 |
| --- | --- |
| [11-E Voice 반응성 재정의](./11e-voice-responsiveness-measurement-redefinition.md) | 새 W-01~W-03 경계·Voice endpoint·LLM/S2S evidence 사용 규칙 |
| [11-A 측정 계약](./11a-measurement-baseline.md) | 모델 포함 시간·pass/fail·변경량·안전 분모 |
| [Qwen 근거](./11-evidence/asr01-qwen-evidence.md) | 확인한 공개값과 계산 가정, tokenization 미실행 구분 |
| [11-B 시험 목록](./11b-test-case-catalog.md) | 94개 UC·24개 변경·24개 안전 판단 기회 |
| [참조자료 적용](./11-evidence/reference-presentation-application.md) | 본문 비교와 부록 증거의 연결 |

공식 tokenizer 실행·실제 모델 추론·녹음·실제 VIA 후보 측정은 아직 수행되지 않았다. 계산 보조 코드와 catalog 자체를 검증한 것을 제품 성능 실측으로 부르지 않는다.

**다음 순서: 11-E 검토 → DP×W applicability와 fixture 확정 → target·score·machine contract 재동결 → 측정 코드 변경.**

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
