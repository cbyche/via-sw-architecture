# QA Catalog Review Plan and Decision Log

> 상태: **CORE ASR SELECTION APPROVED — measurement freeze pending**
> 목적: 2026-09-28 승인한 네 core ASR과 아직 승인하지 않은 target·fixture·반복·score 작업을 분리한다.

## 0. 2026-09-28 핵심 ASR 결정

1. QA-09는 QA-01/02/03/05의 VIA-attributable 시간을 모든 applicable trial에 대해 단순 평균한다. QA-04는 core 평균에서 제외하고 Voice 회귀로 유지한다.
2. QA-19는 QA-11/12의 중복 없는 applicable machine field를 전체 단순 accuracy로 집계한다. QA-13~15는 회귀 진단이다.
3. QA-29는 QA-21~23 change superset 중 DP별 applicable item의 changed Architecture Element 수를 단순 평균한다.
4. QA-39는 containment·correct recovery·deadline·no-duplicate 조건을 모두 만족한 fault trial의 단순 성공률이다.
5. 네 QA를 `CONFIRMED_ASR`로 분류하고 VIA-DP-01~18의 공통 평가 항목으로 지정한다.
6. QA-41은 target-device memory diagnostic으로 유지하고 core ASR에서 제외한다.
7. QA-51과 action/access rule은 safety qualification, QA-61/62는 evidence qualification으로 유지한다.
8. 기존 상세 QA와 reference 결과를 새 core QA 결과로 소급 변환하지 않는다.

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
| QA-09 | DP별 interaction case, equal repetition, timeout imputation, precision, target와 score band |
| QA-19 | 중복 없는 field registry, granularity, corpus, target와 score band |
| QA-29 | DP별 applicable change set, element ledger, target와 score band |
| QA-39 | DP별 fault set, necessary closure, recovery deadline, 반복, target와 score band |
| QA-41 | target PC, process/model accounting boundary, sampling과 target·score band |
| QA-51 | protected unit inventory, recipient/purpose별 최소 집합과 score band |
| QA-61/62 | required trace relation, evidence package, sample, target와 score band |

새 QA의 target은 근거 없이 기존 QA에서 복사하지 않는다. 현재 결과는 모두 `NOT_RUN`이다.

## 3. 다음 검토 순서

1. Core ASR Contract의 VIA-DP-01~18 초기 원장을 DP별 `PRIMARY / REGRESSION_ONLY / NOT_APPLICABLE / UNRESOLVED`로 동결한다.
2. QA-09 case·반복·timeout, QA-19 field, QA-29 change, QA-39 fault 모집단을 고정한다.
3. 근거가 있는 target과 0~5 band를 승인한다.
4. machine-readable contract와 공통 analyzer를 구현하고 sentinel로 검증한다.
5. 그 뒤에만 새 generation A/B campaign을 실행한다.

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
- QA-09/19/29/39의 ASR 분류는 승인됐지만 target·fixture·harness·실행 완료를 별도로 확인함

## 6. 전체 coverage 재검토 결과

| Category | 결론 |
| --- | --- |
| Responsiveness | QA-01/02/03/05를 QA-09 평균의 case stratum으로 통합; QA-04 regression |
| Correctness & Continuity | QA-11/12를 QA-19의 중복 없는 field registry로 통합; QA-13~15는 회귀 진단 |
| Modifiability | QA-21/22/23 applicable change를 QA-29 단순 평균으로 통합 |
| Reliability & Availability | QA-31/32 evidence를 QA-39 binary fault PASS로 통합 |
| Resource Efficiency | 직관적인 memory QA-41만 유지. QA-42 합성 compute/energy metric은 추가하지 않음 |
| Privacy & Security | QA-51 단일 exposure metric과 mandatory safety gate 유지 |
| Observability | 실행 trace 완전성 QA-61과 평가 결과 재현성 QA-62 추가 |

네 core QA의 metric과 ASR 분류는 승인됐다. target, score band, fixture, 반복, timeout, machine contract와 실행 결과는 아직 확정되지 않았다.
