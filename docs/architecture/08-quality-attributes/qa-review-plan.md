# QA Catalog Review Plan and Decision Log

> 상태: **ONGOING USER REVIEW**
> 목적: 이번 대화에서 합의한 방향과 아직 합의하지 않은 일을 분리하여 다음 세션에서도 같은 지점에서 검토를 계속한다.

## 1. 2026-09-23까지 합의한 방향

| 항목 | 현재 방향 | 아직 확정 아님 |
| --- | --- | --- |
| QA-01~03 | Voice 사용자 관점 responsiveness를 유지 | case, model profile, workload, target·band |
| 이전 QA-04 | 독립 QA에서 제외하고 concurrency workload condition으로 이동 | 대표 동시 Task 수와 load shape |
| QA-05 | Downstream Agent 품질이 아닌 VIA request handling correctness | selected corpus, predicate JSON, 반복 수, critical-failure cap |
| 이전 QA-06 | 정상 continuity는 QA-05, 장애 후 continuity는 QA-09에 병합 | 각 selected case의 binding predicate |
| QA-07/08 | change당 변경 Architecture Element 평균 수 | 후보별 element ledger, applicability, target·band 최종 승인 |
| QA-09 | 모든 영향 Task의 정확한 복구 시간 | fault strata, workload, repeat, timeout·percentile |
| 이전 QA-10 | 독립 점수를 없애고 QA-09 blast-radius secondary trace로 이동 | QA-09 원인 분석 trace의 최소 필드 |
| QA-11 | 정상 기능의 최소 필요량을 넘겨 외부에 노출한 보호정보 단위 수 | protected unit inventory, recipient/purpose별 최소 집합, band |
| 이전 QA-12 | 독립 QA에서 제외하되 action/access 위반 0건 필수 회귀 유지 | 회귀 fixture 최종 범위 |

QA catalog 자체는 승인되지 않았다. 위 표는 이후 검토의 출발점이지 변경 금지 baseline이 아니다.

## 2. 다음 검토 순서

1. 각 active QA가 정말 Architecture 대안을 가르는지 DP별 구조 인과를 검토한다.
2. metric이 심사위원에게 한 문장으로 설명되는지 확인한다.
3. canonical case/change/fault/workload와 machine-readable oracle을 고정한다.
4. 근거가 있는 target과 0~5 band를 승인한다.
5. QA별 `APPLICABLE / REGRESSION_ONLY / NOT_APPLICABLE` DP mapping을 고정한다.
6. 그 뒤에만 machine measurement contract와 harness를 수정한다.

## 3. 변경 관리

QA를 바꿀 때는 다음을 한 변경으로 처리한다.

- [Quality Model](./quality-model.md)과 [machine-readable registry](./qa-catalog-draft.json) 갱신
- [Scoring Contract](../11-measurement/scoring-contract.md), traceability와 candidate mapping 갱신
- superseded 문서·implementation·result가 있으면 같은 세대로 archive
- `check_qa_catalog.py`, link, terminology 검사를 통과
- 아직 승인하지 않은 target이나 결과를 current evidence로 표현하지 않음

ID 공백은 review 중 이력을 보존하기 위한 것이다. Catalog 전체 승인 뒤 필요하면 한 번만 재번호화한다.

## 4. Catalog 승인 완료 조건

- 각 QA의 중요성·구조 인과·직관적 metric에 사용자 동의
- canonical workload와 oracle이 결과 전에 고정됨
- target과 0~5 band의 근거가 문서화됨
- DP별 applicability가 실제 responsibility/contract/state/deployment 차이로 설명됨
- 필수 회귀와 scored QA가 구분됨
- ASR은 그 다음 별도 판정이며 catalog 승인만으로 자동 선정되지 않음
