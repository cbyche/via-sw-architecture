# 08-D. Working ASR Candidate Catalog

> **W12-G2 historical notice:** 이 문서의 responsiveness 후보 명칭과 endpoint는 초기 탐색 기록이다. 현재 W-01~W-03 Voice 정의는 [11-E](./11e-voice-responsiveness-measurement-redefinition.md)를 따른다.

> 작성일: 2026-09-21
> 상태: **Working candidate set — 12-A Architecture Sensitivity Sweep에 12개 전수 반입. 정식 rebaseline/최종 ASR 선정 전**
> 원칙: 최종 확정 시 각 QA/ASR은 대표 Metric 하나만 사용하고 모든 DP에서 동일 target/score rule을 적용한다.

## 1. Working candidate set

| 후보 | 역할 | Representative Metric 방향 |
| --- | --- | --- |
| Conversational Reaction Responsiveness | Strong discriminator | p95 user-input-end → first meaningful response start, lower better |
| Task Handoff Responsiveness | Strong discriminator | p95 user-input-end → durable Agent handoff, lower better |
| Task Feedback Responsiveness | Strong discriminator | p95 Agent event available → user-visible feedback start, lower better |
| Concurrent Task Performance Isolation | Strong discriminator | 4-task / 1-task foreground latency degradation ratio, lower better |
| Task Completion Effectiveness | Conditional discriminator | obligation satisfaction rate, higher better |
| Interaction & Task Continuity | Conditional discriminator | continuity obligation preservation rate, higher better |
| Agent Ecosystem Interoperability & Substitutability | Strong discriminator | avg changed architecture elements / Agent change, lower better |
| Evolvability & Maintainability | Strong discriminator | avg changed architecture elements / non-Agent change, lower better |
| Recovery Timeliness & Recoverability | Strong/conditional discriminator | p95 time to correct user-visible recovery, lower better |
| Dependency Failure Containment & Graceful Degradation | Strong discriminator | essential capability retention under dependency failure, higher better |
| Privacy Exposure Minimization | Strong discriminator | remote sensitive-context exposure ratio, lower better |
| Action & Access Safety | Constraint-style ASR | safety violation rate V/N, target 0 |

## 2. Role grouping

### A. Strong structural drivers

- Conversational Reaction Responsiveness
- Task Handoff Responsiveness
- Task Feedback Responsiveness
- Concurrent Task Performance Isolation
- Agent Ecosystem Interoperability & Substitutability
- Evolvability & Maintainability
- Recovery Timeliness
- Dependency Failure Containment & Graceful Degradation
- Privacy Exposure Minimization

이들은 정상적인 합리적 Architecture 대안 사이에서도 구조 차이가 representative metric 차이로 자연스럽게 나타날 가능성이 높다.

### B. Conditional structural drivers

- Task Completion Effectiveness
- Interaction & Task Continuity

제품 중요도는 높지만 동일한 강한 Model/Agent와 충분한 Context/State를 제공하는 정상 대안끼리는 100%에 가까운 동점이 가능하다. 실제 information-flow / semantic-pipeline 구조가 completion/continuity를 자연스럽게 바꾸는 DP에서만 Primary로 사용한다.

### C. Constraint-style ASR

- Action & Access Safety

정상 후보는 모두 0 violation이어야 한다. Architecture에 큰 영향을 주므로 system-level ASR/constraint로 유지할 수 있지만, 대부분의 DP에서 discriminator로 기대하지 않는다.

Recovery correctness 27-TC strict pass도 별도 ASR이 아니라 Recovery Timeliness의 regression invariant로 유지한다.

## 3. Secondary / Reserve only

- Voice Barge-in Latency
- Voice Streaming Jitter / Underrun
- Compute & Resource Efficiency
- Diagnosability & Operability
- Task State Freshness
- Cross-device Portability

현재 제품 요구 또는 독립성 근거가 H/H 승격에 충분하지 않거나 다른 metric과 중복되어 secondary/reserve로 유지한다.

## 4. QA/ASR 구조 원칙

기존의 `QA-01 Responsiveness`를 세 개 이상의 서로 다른 response measure로 유지하면 'QA별 대표 Metric 하나' 원칙과 충돌한다. 정식 rebaseline에서는 다음 중 하나를 명시적으로 선택해야 한다.

권고안: **각 responsiveness dimension을 독립 QA/ASR로 분리**한다.

- Conversational Reaction Responsiveness
- Task Handoff Responsiveness
- Task Feedback Responsiveness
- Concurrent Task Performance Isolation

이렇게 하면 각 Quality Attribute가 하나의 representative metric/target/score rule을 갖는 기존 프로젝트 원칙을 유지할 수 있다.

Recovery와 Privacy도 같은 원칙을 적용한다.

- Reliability/Recoverability → Recovery Timeliness를 대표 QA/ASR metric으로 사용하고 strict correctness는 invariant
- Privacy/Security/Action Safety → Privacy Exposure와 Action & Access Safety를 분리

## 5. Next rebaseline gate

정식 확정 전 다음을 순서대로 수행한다.

1. 위 12개 후보의 Importance / Architecture Difficulty를 동일 1~10 규칙으로 재평가
2. H/H 통과 여부와 중복 여부 확인
3. 최종 QA/ASR catalog renumbering
4. 09 UC/change mapping 재작성
5. 11-A/B/C metric/TC/target/score rule 재동결
6. 그 이후 12 DP 도출


## 6. 12-A에서의 사용 방식

이 12개 후보는 모두 12-A의 Architecture Sensitivity Sweep에 가져간다. 목적은 후보 결과에 맞춰 ASR을 사후 발명하는 것이 아니라, **동일한 사전 정의 Metric으로 합리적 Architecture 대안의 구조 민감성을 전수 관찰**하는 것이다.

- 12개 모두 후보별 raw Metric과 0~5 score를 산출할 수 있도록 12 진입 전에 Metric/Target/Score Boundary를 동결한다.
- 12-A에서는 12개 점수를 단일 weighted total로 합치지 않는다.
- 각 DP에서 구조적으로 적용되지 않는 후보는 억지 숫자를 만들지 않고 `N/A — no causal applicability`로 기록한다.
- 결과가 모두 동일한 QA는 숨기지 않는다. 해당 DP에서 non-discriminating임을 명시한다.
- 결과를 본 뒤 특정 후보에 유리한 target/score boundary를 수정하지 않는다.
- 어떤 ASR을 최종 DP Primary Driver로 인정할지는 사전에 고정한 Differentiation Gate를 따른다.

상세 절차는 [12-00 Evaluation Method](./12-00-evaluation-method.md)를 따른다.
