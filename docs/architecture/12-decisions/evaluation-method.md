# 12-00. Architecture Evaluation Method — 12-ASR Sensitivity Sweep

> 작성일: 2026-09-21
> 상태: **12 Architecture Decision 평가 방법 초안 — 실제 DP 후보 점수 산출 전**
> 목적: Working ASR 12개를 모두 유지한 상태에서 구조 대안의 실제 trade-off를 탐색하되, 사후 해석·cherry-picking을 방지한다.
> W12-G2: W-01~W-03의 명칭·endpoint는 [Voice responsiveness 정의](../08-quality-attributes/voice-responsiveness.md)로 재정의되었다. 세 지표의 target/score와 DP applicability는 새 Measurement Freeze 전에 다시 고정한다.

## 1. 핵심 원칙

최종 발표에서 구조 대안별 QA trade-off가 명확히 보여야 한다. 그러나 후보 점수를 본 뒤 유리한 QA만 고르거나 target/score band를 바꾸면 Architecture 평가가 아니라 결과 맞추기가 된다.

따라서 12는 두 단계로 나눈다.

### 12-A Architecture Sensitivity Sweep

- Working ASR 12개를 모두 확인
- 각 DP의 모든 합리적 후보를 같은 Metric/Target/Score Boundary로 평가
- weighted total / winner를 만들지 않음
- 구조적으로 해당 DP와 인과관계가 없는 ASR은 N/A로 기록
- 동일 점수도 그대로 공개
- 후보별 trade-off pattern과 실제 sensitivity를 관찰

### 12-B Decision Evaluation

- 12-A 결과와 구조 인과분석을 Differentiation Gate에 적용
- Gate를 통과한 약 3~4개를 해당 DP의 Primary Architecture Driver로 고정
- 나머지는 regression / constraint / secondary evidence로 유지
- Primary set 고정 후 final decision narrative와 weakness/tactic evaluation 수행

## 1-A. Candidate Mutual-Exclusivity Gate

Gate 2에서 상세 후보를 만들기 전에 각 DP의 대안은 다음을 만족해야 한다.

1. 두 대안 모두 필수 기능을 충족하는 정상 Architecture다.
2. 같은 authoritative owner / canonical contract / primary state path / process fault boundary를 서로 다르게 결정한다.
3. A+B를 단순 결합하면 decision이 사라지는 경우 두 안을 독립 Architecture alternative로 인정하지 않는다.
4. 공통 tactic은 허용하지만 authority를 바꾸면 candidate identity 변경으로 처리한다.
5. Hybrid는 단순 장점 결합이 아니라 새로운 authority split과 독립적인 비용/변경면을 가질 때만 별도 family로 인정한다.

이 Gate를 통과하지 못한 비교는 ASR sensitivity가 크게 보여도 발표용 Architecture Decision으로 사용하지 않는다.

## 2. 사후 논리 만들기와 허용되는 해석의 경계

허용:
- 결과가 왜 발생했는지 candidate 구조/trace/change ledger로 설명
- 예상과 다른 결과가 나온 원인을 분석
- 발표용 문구·시각화를 결과에 맞게 명확하게 다듬기

금지:
- 특정 후보가 유리하도록 ASR target/score boundary 변경
- 후보 결과를 본 뒤 새 ASR을 즉석 추가
- 점수 차이가 난 QA만 골라 중요했다고 주장
- 구조 인과관계가 없는 우연한 차이를 Architecture trade-off로 주장
- 의도적으로 약한 후보를 만들어 점수 차이 생성

즉 **storytelling은 결과 후에 다듬을 수 있지만, 평가 논리와 기준은 결과 전에 고정한다.**

## 3. 12개 Working ASR 전수 Sweep

| ID | Working ASR | Sweep role |
| --- | --- | --- |
| W-01 | Delegated Task Result Responsiveness | strong candidate / target pending |
| W-02 | VIA Direct Voice Response Responsiveness | strong candidate / target pending |
| W-03 | Agent Progress Voice Feedback Responsiveness | strong candidate / target pending |
| W-04 | Concurrent Task Performance Isolation | strong candidate |
| W-05 | Task Completion Effectiveness | conditional |
| W-06 | Interaction & Task Continuity | conditional |
| W-07 | Agent Ecosystem Interoperability & Substitutability | strong candidate |
| W-08 | Evolvability & Maintainability | strong candidate |
| W-09 | Recovery Timeliness & Recoverability | strong/conditional |
| W-10 | Dependency Failure Containment & Graceful Degradation | strong candidate |
| W-11 | Privacy Exposure Minimization | strong candidate |
| W-12 | Action & Access Safety | constraint-style |

12-A에서는 이 역할 label 때문에 점수를 제외하지 않는다. 모든 applicable ASR을 계산한다.

## 4. Differentiation Gate

어떤 Working ASR을 특정 DP의 Primary Architecture Driver로 인정하려면 다음을 모두 만족해야 한다.

### G1. Product Relevance

해당 DP가 다루는 사용자/제품 상황과 직접 관련된 QA여야 한다.

### G2. Natural Structural Causality

합리적으로 구현한 후보 사이의 책임 배치, call graph, state authority, contract, deployment, persistence 등의 차이가 Metric에 자연스럽게 영향을 주어야 한다.

### G3. Observable Sensitivity

12-A 결과에서 다음 중 하나 이상이 나타난다.

- 후보 최고/최저가 **최소 1개 score band 이상** 차이
- 동일 target 기준에서 후보들의 **target pass/fail이 갈림**
- 다른 Primary candidate metric과 **명확한 반대 방향 trade-off**가 나타남

단, 작은 measurement noise는 sensitivity로 인정하지 않는다.

### G4. Non-Redundancy

다른 Primary ASR과 사실상 동일한 response를 중복 측정하지 않는다.

### G5. Evidence Traceability

점수 차이를 candidate component/contract/state/call graph/change ledger/trace로 설명할 수 있어야 한다.

위 조건을 통과하지 못하면 해당 DP에서는 Primary가 아니며, System-level 중요성과 무관하게 regression/secondary로 둔다.

## 5. 왜 결과를 본 뒤 Gate를 적용해도 cherry-picking이 아닌가

Gate 규칙 자체를 후보 결과 전에 고정하고, '어느 후보에게 유리한가'가 아니라 **실제 구조 sensitivity가 존재하는가**만 판단한다.

예를 들어 세 후보가 ASR-06에서 모두 100%를 얻으면 이를 숨기지 않고 'non-discriminating constraint'로 기록한다. 반대로 ASR-01에서 5/3/2점으로 갈리고 call graph 차이로 설명 가능하면 Primary driver가 될 수 있다.

Gate는 후보의 승자를 고르는 규칙이 아니라 **어떤 QA가 이 DP에서 실제 trade-off 축인지 식별하는 규칙**이다.

## 6. 발표 자료에 보여줄 형태

각 DP는 다음 순서로 제시한다.

1. DP가 다루는 구조 질문
2. 합리적 후보 A/B/C
3. 12개 ASR sensitivity sweep mini-heatmap
4. Differentiation Gate를 통과한 Primary 3~4개 확대
5. Primary QA의 raw metric + 0~5 score
6. trade-off 설명
7. 선택
8. 선택안의 weakness
9. tactic 적용 후 동일 metric 재평가

이렇게 하면 '왜 이 QA만 비교했는가?'라는 질문에 12개 전수 sweep과 Gate로 답할 수 있고, 결과가 평평한 ASR도 숨기지 않는다.

## 7. 12 진입 전 남은 작업

실제 DP 후보 점수를 계산하기 전에 Working ASR 12개 모두에 대해 다음을 동결한다.

- 대표 Metric
- workload / stimulus
- Target
- 0~5 Score Boundary
- evidence level
- N/A 판정 규칙

현재 기존 7개 중심의 11-C는 Working ASR rebaseline 때문에 최종 freeze 상태가 아니다. 12개의 scoring baseline을 먼저 재작성한다.
