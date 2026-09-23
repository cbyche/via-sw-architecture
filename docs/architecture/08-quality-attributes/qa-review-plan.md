# QA Catalog Review Plan and Decision Log

> 상태: **ONGOING USER REVIEW — category와 single-metric 구조 합의**
> 목적: 합의한 QA 구조와 아직 승인하지 않은 measurement 작업을 분리한다.

## 1. 2026-09-23 합의 사항

1. QA ID를 품질 계열별 번호 범위로 한 번 재구성한다.
2. 하나의 QA는 하나의 대표 metric만 가진다.
3. 기존 request handling QA는 통합 outcome인 QA-11로 유지한다.
4. semantic resolution, interaction binding, async state convergence, normal continuity를 QA-12~15로 분리한다.
5. QA-11과 QA-12~15는 함께 보고하되 weighted total에 중복 가중하지 않는다.
6. recovery time과 fault blast radius를 QA-31과 QA-32로 분리한다.
7. Voice interruption을 QA-04로 추가한다.
8. target device resource는 우선 peak committed memory인 QA-41로 측정한다.
9. Privacy는 QA-51의 단일 exposure metric만 유지한다.
10. QA-21~23은 평균 changed Architecture Element count를 대표 metric으로 유지한다.
11. Action/access safety는 scored QA가 아니라 zero-violation qualification gate로 유지한다.
12. Task cancel·correction 같은 사용자 제어의 응답시간을 QA-05로 분리한다.
13. 연구 실험·로그 변경의 파급 범위를 QA-23으로 측정한다.
14. 실행 로그의 완전성과 평가 결과 재현성을 QA-61과 QA-62로 분리한다.
15. Resource Efficiency에는 현재 QA-41 memory만 유지하고, 직관적이지 않은 합성 compute/energy metric은 추가하지 않는다.

## 2. 아직 승인하지 않은 것

| QA | 남은 결정 |
| --- | --- |
| QA-01~04 | canonical case, Voice fixture, workload, target와 score band |
| QA-05 | canonical control case, authoritative disposition, Voice/Text 비중, target와 score band |
| QA-11~15 | selected corpus, predicate schema, isolation fixture, 반복 수, target와 score band |
| QA-21~23 | 후보별 element ledger, applicability, experiment/logging change pack과 target·score band 최종 승인 |
| QA-31/32 | fault strata, affected-unit registry, 반복·timeout·target·score band |
| QA-41 | target PC, process/model accounting boundary, sampling과 target·score band |
| QA-51 | protected unit inventory, recipient/purpose별 최소 집합과 score band |
| QA-61/62 | required trace relation, evidence package, sample, target와 score band |

새 QA의 target은 근거 없이 기존 QA에서 복사하지 않는다. 현재 결과는 모두 `NOT_RUN`이다.

## 3. 다음 검토 순서

1. 각 active QA가 정상적인 Architecture 대안을 가르는 구조 인과를 검토한다.
2. metric의 stimulus, terminal observable과 단위를 확정한다.
3. canonical case/change/fault/workload와 machine-readable oracle을 고정한다.
4. 근거가 있는 target과 0~5 band를 승인한다.
5. QA별 `APPLICABLE / REGRESSION_ONLY / NOT_APPLICABLE` DP mapping을 고정한다.
6. 그 뒤에만 machine measurement contract와 harness를 구현한다.

## 4. 변경 관리

QA를 바꿀 때는 다음을 한 변경으로 처리한다.

- [Quality Model](./quality-model.md)과 [machine-readable registry](./qa-catalog-draft.json) 갱신
- [Scoring Contract](../11-measurement/scoring-contract.md), traceability와 candidate mapping 갱신
- superseded 문서·implementation·result가 있으면 generation과 evidence 상태를 명확히 표시
- QA catalog, link와 terminology 검사를 통과
- 아직 승인하지 않은 target이나 결과를 current evidence로 표현하지 않음

이번 category-range 재번호화 이후에는 QA가 추가·제거돼도 기존 ID를 당겨 붙이지 않는다. Archive의 historical ID는 변경하지 않는다.

## 5. Catalog 승인 완료 조건

- 각 QA의 중요성·구조 인과·단일 metric에 사용자 동의
- canonical workload와 oracle이 결과 전에 고정됨
- target과 0~5 band의 근거가 문서화됨
- DP별 applicability가 실제 responsibility/contract/state/deployment 차이로 설명됨
- integrated outcome과 driver QA의 중복 가중이 방지됨
- 필수 qualification gate와 scored QA가 구분됨
- ASR은 그 다음 별도 판정이며 catalog 승인만으로 자동 선정되지 않음

## 6. 전체 coverage 재검토 결과

| Category | 결론 |
| --- | --- |
| Responsiveness | QA-01~04에 없던 Task control 응답시간을 QA-05로 보완 |
| Correctness & Continuity | QA-11 integrated outcome과 QA-12~15 driver 구조 유지 |
| Modifiability | 외부 생태계 QA-21/22에 연구 실험·로그 변화 QA-23 추가 |
| Reliability & Availability | recovery time QA-31과 blast radius QA-32 분리 유지 |
| Resource Efficiency | 직관적인 memory QA-41만 유지. QA-42 합성 compute/energy metric은 추가하지 않음 |
| Privacy & Security | QA-51 단일 exposure metric과 mandatory safety gate 유지 |
| Observability | 실행 trace 완전성 QA-61과 평가 결과 재현성 QA-62 추가 |

새 QA를 추가한 것은 metric과 structural causality를 검토할 대상으로 승인한 것이며 target, score band, fixture 또는 ASR을 확정한 것은 아니다.
