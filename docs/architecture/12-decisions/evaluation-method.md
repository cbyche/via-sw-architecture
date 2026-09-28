# Architecture Evaluation Method — Four-Core-ASR A/B Evaluation

> 작성일: 2026-09-21
> 상태: **CORE ASR APPROVED / measurement freeze 및 실제 DP 후보 점수 산출 전**
> 목적: QA-09·19·29·39를 모든 DP의 공통 Architecture 평가축으로 사용하고 상세 진단을 보존하면서 사후 해석·cherry-picking을 방지한다.
> Current measurement contract: core 정의와 집계는 승인됐고 target/score, fixture, 반복 수와 DP applicability는 새 Measurement Freeze 전에 고정한다.

## 1. 핵심 원칙

최종 발표에서 구조 대안별 QA trade-off가 명확히 보여야 한다. 그러나 후보 점수를 본 뒤 유리한 QA만 고르거나 target/score band를 바꾸면 Architecture 평가가 아니라 결과 맞추기가 된다.

평가 전에 [전체 QA 사고실험 비교표](./dp-review-protocol.md#73-전체-qa-사고실험-비교표)를 작성한다. 상호 배타적인 steelman A/B에 대해 전체 QA의 예상 우세와 차이 크기, 판단 확실성, 구조적 이유와 조건을 남긴다. 이는 측정 전 가설이며 아래 Evaluation A/B의 실행 결과나 score를 대체하지 않는다. 이후 결과와 대조할 수 있도록 예상표 version을 보존한다.

현재 [전체 요약](./dp-executive-summary.md)은 VIA-DP-01~18 전수 inventory와 미완료 사항을 설명한다. 과거의 우선·보조 분류는 현행 선정 결론이 아니다. 후보 보존과 최종 평가 채택은 구분하며, 기능 부적합·비참여 경로·약한 trade-off를 숨기지 않는다.

실제 평가는 두 단계로 나눈다.

### Evaluation A — Core-ASR Measurement

- QA-09·19·29·39를 모두 산출하고 상세 input·diagnostic·qualification을 함께 확인
- 각 DP의 모든 합리적 후보를 같은 Metric/Target/Score Boundary로 평가
- weighted total / winner를 만들지 않음
- 구조적으로 해당 DP와 인과관계가 없는 QA는 N/A로 기록
- 동일 점수도 그대로 공개
- 후보별 trade-off pattern과 실제 sensitivity를 관찰

### Evaluation B — Decision Evaluation

- Evaluation A 결과와 구조 인과분석을 Differentiation Criteria에 적용
- criteria를 충족한 QA를 해당 DP의 Primary Architecture Driver로 확정. 발표 편의를 위해 개수를 강제하지 않음
- 나머지는 regression / constraint / secondary evidence로 유지
- 네 core ASR의 `PRIMARY`/`REGRESSION_ONLY` 역할과 상세 원인을 [Core ASR Contract](../08-quality-attributes/core-asr-contract.md)의 기준으로 기록
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

## 3. 네 core ASR과 상세 evidence

| ID | Core metric | DP package role |
| --- | --- | --- |
| QA-09 | applicable interaction trial의 평균 VIA 책임시간 | QA-01/02/03/05 input; QA-04 제외 |
| QA-19 | applicable QA-11/12 oracle field accuracy | 중복 없는 field micro-average; QA-13~15 회귀 진단 |
| QA-29 | applicable change당 평균 changed Architecture Element 수 | QA-21~23의 DP별 사전 동결 합집합 |
| QA-39 | fault containment·correct recovery 성공률 | QA-31/32 evidence로 binary trial PASS 판정 |

모든 DP는 네 행을 결과 package에 포함한다. 구조 차이의 자연 인과가 예상되면 `PRIMARY`, 같은 기능 보존 확인이면 `REGRESSION_ONLY`로 사전 분류한다. 물리적으로 참여하지 않는 모집단은 근거와 함께 `NOT_APPLICABLE`, 설계·endpoint 미완료는 `UNRESOLVED`다. QA-41은 memory diagnostic, QA-51과 action/access rule은 safety qualification, QA-61/62는 evidence qualification으로 별도 기록한다.

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
3. 네 core ASR 결과와 상세 breakdown mini-heatmap
4. Differentiation Criteria를 충족한 Primary QA 확대
5. Primary QA의 raw metric + 0~5 score
6. trade-off 설명
7. 선택
8. 선택안의 weakness
9. 후보 정의 때 포함한 보완책과 남는 약점 설명. 결과 후 새 보완책을 발견하면 양쪽 적용 가능성을 검토하고 후보 version을 갱신하여 동일 metric 재평가

QA-09는 합계·sample count·산술평균과 class breakdown을, QA-19는 correct/total field와 family breakdown을, QA-29는 changed-element 합계/change 수를, QA-39는 successful/all fault trial을 보여준다. p95, strict run pass, recovery-time 분포와 maximum blast radius는 진단이다. 선택안에 tactic을 적용한 B′를 실제 실행했다면 complete table에 함께 두고, 실행하지 않은 값을 A/B에서 복사하지 않는다.

이렇게 하면 '왜 이 QA만 비교했는가?'라는 질문에 네 핵심 사용자 질문과 사전 정의한 상세 evidence로 답할 수 있고, 결과가 평평한 ASR이나 regression failure도 숨기지 않는다.

## 7. A/B 측정 진입 전 남은 작업

실제 DP 후보 점수를 계산하기 전에 네 core ASR 각각에 대해 다음을 동결한다.

- 대표 Metric
- workload / stimulus
- Target
- 0~5 Score Boundary
- evidence level
- N/A 판정 규칙

QA-09·19·29·39의 `CONFIRMED_ASR` 분류는 확정됐다. 아직 확정하지 않은 것은 DP별 모집단·반복·target·score band와 실제 결과다. 기존 reference campaign을 새 core QA 결과로 소급 집계하지 않는다.
