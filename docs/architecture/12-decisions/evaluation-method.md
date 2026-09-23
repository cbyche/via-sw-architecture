# Architecture Evaluation Method — QA Catalog Sensitivity Sweep

> 작성일: 2026-09-21
> 상태: **USER REVIEW DRAFT / QA catalog 승인 및 실제 DP 후보 점수 산출 전**
> 목적: QA Catalog metric을 모두 유지한 상태에서 구조 대안의 실제 trade-off를 탐색하되, 사후 해석·cherry-picking을 방지한다.
> Current measurement contract: category-range QA catalog와 metric은 초안이며 target/score, fixture, 반복 수와 DP applicability는 새 Measurement Freeze 전에 다시 고정한다.

## 1. 핵심 원칙

최종 발표에서 구조 대안별 QA trade-off가 명확히 보여야 한다. 그러나 후보 점수를 본 뒤 유리한 QA만 고르거나 target/score band를 바꾸면 Architecture 평가가 아니라 결과 맞추기가 된다.

평가 전에 [전체 QA 사고실험 비교표](./dp-review-protocol.md#73-전체-qa-사고실험-비교표)를 작성한다. 상호 배타적인 steelman A/B에 대해 전체 QA의 예상 우세와 차이 크기, 판단 확실성, 구조적 이유와 조건을 남긴다. 이는 측정 전 가설이며 아래 Evaluation A/B의 실행 결과나 score를 대체하지 않는다. 이후 결과와 대조할 수 있도록 예상표 version을 보존한다.

현재 문서 검토의 [최종 요약](./dp-executive-summary.md)과 [전체 후보 분류](./dp-review-synthesis.md)는 13개 보고서를 우선 핵심 검증·조건부 핵심·선행 범위/기능·보조 설계로 나눈다. 이 분류는 아래 sensitivity sweep을 실행한 결과가 아니다. 기능 부적합 후보나 비교 구간이 다른 QA를 수치 점수로 강제 비교하지 않고, 양방향 trade-off가 약한 후보도 핵심 평가표의 수를 채우기 위해 유지하지 않는다.

실제 평가는 두 단계로 나눈다.

### Evaluation A — Architecture Sensitivity Sweep

- QA Catalog metric을 모두 확인
- 각 DP의 모든 합리적 후보를 같은 Metric/Target/Score Boundary로 평가
- weighted total / winner를 만들지 않음
- 구조적으로 해당 DP와 인과관계가 없는 QA는 N/A로 기록
- 동일 점수도 그대로 공개
- 후보별 trade-off pattern과 실제 sensitivity를 관찰

### Evaluation B — Decision Evaluation

- Evaluation A 결과와 구조 인과분석을 Differentiation Criteria에 적용
- criteria를 충족한 QA를 해당 DP의 Primary Architecture Driver로 확정. 발표 편의를 위해 개수를 강제하지 않음
- 나머지는 regression / constraint / secondary evidence로 유지
- 전체 DP를 가로지르는 구조 영향과 위험을 [Quality Model](../08-quality-attributes/quality-model.md)의 기준으로 검토하여 QA별 ASR 상태를 기록
- Primary set 고정 후 final decision narrative와 weakness/tactic evaluation 수행

## 1-A. 상호 배타적인 Steelman A/B 구성

상세 QA 비교표와 Candidate Implementation에 앞서 [DP 공통 검토 절차](./dp-review-protocol.md)를 적용한다. 핵심 요건은 다음과 같다.

1. 같은 목표·필수 기능·완료 조건을 충족하는 두 정상 Architecture를 구성한다.
2. **같은 결정 범위에서 두 안은 상호 배타적(mutually exclusive)이며 서로에게 steelman이어야 한다.** 각 안의 채택 이유, 가능한 보완책과 보완 후 남는 약점을 설명한다.
3. 같은 대상·조건의 최종 권한, 계약, 기준 상태 또는 Process 경계를 어떻게 서로 양립할 수 없게 결정하는지 명시한다. 공통 기술의 공존만으로 상호 배타성 여부를 판정하지 않는다.
4. hybrid가 양쪽 장점을 취할 수 있으면 이를 새 A로 구성하고, 이에 대등하면서 상호 배타적인 새 B를 도출한다. B가 성립하지 않으면 DP를 재정의·분할·통합하거나 제외한다.
5. 정상적인 snapshot·cache·retry·로그 등은 양쪽에서 검토한다. 후보 결정을 유지하는 보완책이면 해당 안에 포함하고, 최종 권한·기준 기록을 바꾸면 새 후보 version으로 처리한다.
6. 유력한 제3안을 배제한 이유, 외부 기능의 실현 가능성, 다른 DP의 고정 조건과 QA 인과를 기록한다. 점수 차이를 만들기 위해 기능·보완책을 한쪽에서 제거하지 않는다.

이 기준을 통과하지 못한 비교는 QA 차이가 크게 보여도 Architecture Decision 근거로 사용하지 않는다. 아래 평가에서 확인되는 동점이나 한쪽의 지배적 결과도 그대로 남긴다.

## 2. 사후 논리 만들기와 허용되는 해석의 경계

허용:
- 결과가 왜 발생했는지 candidate 구조/trace/change ledger로 설명
- 예상과 다른 결과가 나온 원인을 분석
- 발표용 문구·시각화를 결과에 맞게 명확하게 다듬기

금지:
- 특정 후보가 유리하도록 QA target/score boundary 변경
- 후보 결과를 본 뒤 새 ASR을 즉석 추가
- 점수 차이가 난 QA만 골라 중요했다고 주장
- 구조 인과관계가 없는 우연한 차이를 Architecture trade-off로 주장
- 의도적으로 약한 후보를 만들어 점수 차이 생성

즉 **storytelling은 결과 후에 다듬을 수 있지만, 평가 논리와 기준은 결과 전에 고정한다.**

## 3. QA Catalog 전수 Sweep

| ID | QA metric | Sweep role |
| --- | --- | --- |
| QA-01 | Delegated Path VIA Responsiveness | strong candidate / target pending |
| QA-02 | VIA Direct Voice Response Responsiveness | strong candidate / target pending |
| QA-03 | Agent Progress Voice Feedback Responsiveness | strong candidate / target pending |
| QA-04 | Voice Interruption Responsiveness | strong/conditional; target pending |
| QA-05 | Task Control Responsiveness | strong/conditional; target pending |
| QA-11 | VIA Request Handling Correctness | previous QA-05 metric preserved; integrated outcome; conditional |
| QA-12 | Request Semantic Resolution Correctness | non-additive driver; conditional |
| QA-13 | Task & Interaction Binding Correctness | non-additive driver; strong/conditional |
| QA-14 | Async State Convergence Correctness | non-additive driver; strong/conditional |
| QA-15 | Interaction & Task Continuity Correctness | non-additive driver; strong/conditional |
| QA-21 | Agent Change Locality | strong candidate |
| QA-22 | Model, Context & State Change Locality | strong candidate |
| QA-23 | Experiment & Logging Change Locality | strong/conditional; target pending |
| QA-31 | Correct Task Recovery Time | strong/conditional |
| QA-32 | Fault Blast Radius | strong/conditional |
| QA-41 | Target-device Memory Footprint | strong/conditional; target pending |
| QA-51 | Protected Data Exposure Minimization | strong candidate |
| QA-61 | Execution Trace Completeness | strong/conditional; target pending |
| QA-62 | Evidence Reproducibility | strong/conditional; target pending |

Evaluation A에서는 이 역할 label 때문에 점수를 제외하지 않는다. 모든 applicable active QA를 계산한다. 단, QA-11과 QA-12~15는 같은 성공을 중복 보상하는 weighted total로 합산하지 않는다. 동시성은 workload stratum, action/access safety는 필수 회귀로 별도 기록한다.

## 4. Differentiation Criteria

어떤 QA metric을 특정 DP의 Primary Architecture Driver로 인정하려면 다음을 모두 만족해야 한다.

### G1. Product Relevance

해당 DP가 다루는 사용자/제품 상황과 직접 관련된 QA여야 한다.

### G2. Natural Structural Causality

합리적으로 구현한 후보 사이의 책임 배치, call graph, state authority, contract, deployment, persistence 등의 차이가 Metric에 자연스럽게 영향을 주어야 한다.

### G3. Observable Sensitivity

Evaluation A 결과에서 다음 중 하나 이상이 나타난다.

- 후보 최고/최저가 **최소 1개 score band 이상** 차이
- 동일 target 기준에서 후보들의 **target pass/fail이 갈림**
- 다른 Primary candidate metric과 **명확한 반대 방향 trade-off**가 나타남

단, 작은 measurement noise는 sensitivity로 인정하지 않는다.

### G4. Non-Redundancy

다른 Primary QA와 사실상 동일한 response를 중복 측정하지 않는다.

### G5. Evidence Traceability

점수 차이를 candidate component/contract/state/call graph/change ledger/trace로 설명할 수 있어야 한다.

위 조건을 통과하지 못하면 해당 DP에서는 Primary가 아니며, System-level 중요성과 무관하게 regression/secondary로 둔다.

## 5. 왜 결과를 본 뒤 criteria를 적용해도 cherry-picking이 아닌가

Criteria 자체를 후보 결과 전에 고정하고, '어느 후보에게 유리한가'가 아니라 **실제 구조 sensitivity가 존재하는가**만 판단한다.

예를 들어 세 후보가 필수 안전 회귀를 모두 통과하면 이를 숨기지 않고 `non-discriminating constraint`로 기록한다. 반대로 QA-21에서 점수가 갈리고 contract boundary 차이로 설명 가능하면 해당 DP의 Primary driver가 될 수 있다.

이 criteria는 후보의 승자를 고르는 규칙이 아니라 **어떤 QA가 이 DP에서 실제 trade-off 축인지 식별하는 규칙**이다.

## 6. 발표 자료에 보여줄 형태

각 DP는 다음 순서로 제시한다.

1. DP가 다루는 구조 질문
2. hybrid·제3안 검토를 거쳐 구성한 상호 배타적인 steelman A/B
3. 승인된 active QA sensitivity sweep mini-heatmap
4. Differentiation Criteria를 충족한 Primary QA 확대
5. Primary QA의 raw metric + 0~5 score
6. trade-off 설명
7. 선택
8. 선택안의 weakness
9. 후보 정의 때 포함한 보완책과 남는 약점 설명. 결과 후 새 보완책을 발견하면 양쪽 적용 가능성을 검토하고 후보 version을 갱신하여 동일 metric 재평가

이렇게 하면 '왜 이 QA만 비교했는가?'라는 질문에 active QA 전수 sweep과 사전 정의한 criteria로 답할 수 있고, 결과가 평평한 QA도 숨기지 않는다.

## 7. 12 진입 전 남은 작업

실제 DP 후보 점수를 계산하기 전에 QA Catalog metric 모두에 대해 다음을 동결한다.

- 대표 Metric
- workload / stimulus
- Target
- 0~5 Score Boundary
- evidence level
- N/A 판정 규칙

ASR은 아직 선정하지 않았다. 사용자 검토 중인 active QA의 scoring baseline과 DP별 구조 인과를 먼저 정리한 뒤, 별도 ID를 만들지 않고 해당 QA에 ASR 상태를 기록한다.
